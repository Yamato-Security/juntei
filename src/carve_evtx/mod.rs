// Copyright (C) 2026 Shirofune Security Co., Ltd.
//
// Unless explicitly acquired and licensed from Licensor under another license, the contents of this
// file are subject to the Reciprocal Public License ("RPL") Version 1.5, or subsequent versions as
// allowed by the RPL, and You may not copy or use this file in either source code or executable
// form, except in compliance with the terms and conditions of the RPL.
//
// All software distributed under the RPL is provided strictly on an "AS IS" basis, WITHOUT WARRANTY
// OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR HEREBY DISCLAIMS ALL SUCH WARRANTIES,
// INCLUDING WITHOUT LIMITATION, ANY WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE, QUIET ENJOYMENT, OR NON-INFRINGEMENT. See the RPL for specific language governing rights
// and limitations under the RPL.

//! `carve-evtx`: event-log chunks left on a disk after a log was cleared or deleted, in the
//! pagefile, or in a memory image, rebuilt into `.evtx` files that Hayabusa and Event Viewer open.
//!
//! Each 64 KiB chunk carries its own string and template tables, so its records can be read without
//! the rest of its log. Chunks are found by their signature at any byte offset (also inside
//! NTFS-compressed data). On a disk image or a raw NTFS disk or volume, each chunk is also placed:
//! in free space, in a deleted log whose `$MFT` record still lists the cluster, in a live log, or in
//! another file. Deleted logs whose clusters were not reused are rebuilt whole first. Chunks of the
//! live logs are left out, and so is a chunk whose records are all in the live logs or in another
//! carved chunk, so the output holds what is gone from the live logs, once.

mod chunk;
mod volume;

use crate::color::JunteiColor;
use crate::disk_image::{self, Kind};
use crate::lnk::filetime;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use chunk::{CHUNK_SIZE, Header, MAGIC, Records};
use clap::{Args, ValueEnum};
use evtx::{EvtxChunkData, ParserSettings};
use indicatif::ProgressBar;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vhdx_mount::image::Image;
use volume::{Location, Volume};

/// How much of the input is searched at a time.
const BLOCK: usize = 4 << 20;
/// What is read before a signature for an NTFS-compressed chunk: its LZNT1 block header and the
/// flag byte.
const LZNT1_PREFIX: u64 = 3;
/// A rebuilt log holds at most this many chunks: the file header counts them in 16 bits.
const MAX_CHUNKS_PER_FILE: usize = u16::MAX as usize;
/// Problems printed; the rest are counted.
const MAX_PRINTED: usize = 20;

#[derive(Args, Clone, Debug)]
pub struct Options {
    /// Disk image (E01, VHDX), raw disk or volume, pagefile, memory image, or any other file
    #[arg(help_heading = Some("Input"), short = 'f', long = "file", value_name = "FILE", display_order = 100)]
    pub file: PathBuf,

    /// Live .evtx logs (searched recursively) to leave out when the input has no NTFS volume
    #[arg(help_heading = Some("Input"), short = 'l', long, value_name = "DIR", display_order = 101)]
    pub live_logs: Option<PathBuf>,

    /// Read the input as plain bytes even if it holds NTFS volumes
    #[arg(help_heading = Some("Input"), long, display_order = 102)]
    pub no_file_system: bool,

    /// Cluster size for NTFS-compressed chunks in an input without an NTFS volume
    #[arg(help_heading = Some("Input"), long, value_name = "BYTES", default_value_t = 4096, value_parser = parse_cluster_size, display_order = 103)]
    pub cluster_size: u32,

    /// Search only the free clusters of NTFS volumes (faster, skips the pagefile and other files)
    #[arg(help_heading = Some("Filtering"), long, display_order = 200)]
    pub free_space_only: bool,

    /// Also write the chunks of live logs, and chunks whose records the live logs still hold
    #[arg(help_heading = Some("Filtering"), long, display_order = 201)]
    pub keep_live: bool,

    /// Save the rebuilt logs, recovered files and reports to a new directory
    #[arg(help_heading = Some("Output"), short = 'o', long, value_name = "DIR", display_order = 300)]
    pub output: Option<PathBuf>,

    /// Format of the chunk and record reports
    #[arg(help_heading = Some("Output"), short = 't', long = "output-type", value_name = "TYPE", value_enum, default_value = "csv", display_order = 301)]
    pub output_type: ReportType,

    #[command(flatten)]
    pub common: crate::cli::CommonOptions,
}

/// What the chunk and record reports are written as.
#[derive(ValueEnum, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReportType {
    /// chunks.csv and records.csv
    #[default]
    Csv,
    /// chunks.jsonl and records.jsonl
    Jsonl,
}

fn parse_cluster_size(value: &str) -> Result<u32, String> {
    value
        .parse::<u32>()
        .ok()
        .filter(|n| n.is_power_of_two() && (512..=4096).contains(n))
        .ok_or_else(|| "the cluster size must be 512, 1024, 2048 or 4096 bytes".into())
}

/// The input's bytes.
trait Disk: Read + Seek + Send {}
impl<T: Read + Seek + Send> Disk for T {}

struct Input {
    disk: Box<dyn Disk>,
    size: u64,
    kind: &'static str,
    image: Option<Arc<Image>>,
    warnings: Vec<String>,
}

fn open_input(path: &Path, no_file_system: bool) -> io::Result<Input> {
    let invalid = |e: String| io::Error::new(io::ErrorKind::InvalidData, e);
    // The NTFS volumes of a disk image, unless the input is read as plain bytes. A disk without
    // a readable NTFS volume is still searched, without file system information.
    let volumes = || -> (Option<Arc<Image>>, Vec<String>) {
        if no_file_system {
            return (None, Vec::new());
        }
        match disk_image::open(path) {
            Ok(opened) => {
                let mut warnings = opened.status.warnings();
                warnings.extend(opened.image.problems.clone());
                (Some(Arc::new(opened.image)), warnings)
            }
            Err(e) => (
                None,
                vec![format!(
                    "No NTFS volume was read ({e}); searching the whole disk"
                )],
            ),
        }
    };
    match disk_image::kind(path) {
        Ok(Kind::Vhdx) => {
            let reader = vhdx::VhdxReader::open(path).map_err(|e| invalid(e.to_string()))?;
            let size = reader.virtual_disk_size();
            let (image, warnings) = volumes();
            return Ok(Input {
                disk: Box::new(reader),
                size,
                kind: "VHDX disk image",
                warnings,
                image,
            });
        }
        Ok(Kind::E01) => {
            let ewf = ewf_image::Image::open(path).map_err(|e| invalid(e.to_string()))?;
            let size = ewf.media_size();
            let (image, warnings) = volumes();
            return Ok(Input {
                disk: Box::new(ewf.cursor()),
                size,
                kind: "E01 disk image",
                warnings,
                image,
            });
        }
        Err(_) => {}
    }
    let size = fs::metadata(path)?.len();
    let image = if no_file_system {
        None
    } else {
        Image::from_disk(Box::new(File::open(path)?), size, 512).ok()
    };
    Ok(Input {
        disk: Box::new(File::open(path)?),
        size,
        kind: if image.is_some() {
            "raw disk or volume"
        } else {
            "file"
        },
        warnings: image
            .as_ref()
            .map(|i| i.problems.clone())
            .unwrap_or_default(),
        image: image.map(Arc::new),
    })
}

/// `len` bytes from `offset`, zero-filled past the end of the input, and how many were read.
fn read_at(
    disk: &mut dyn Disk,
    size: u64,
    offset: u64,
    len: usize,
) -> io::Result<(Vec<u8>, usize)> {
    let mut buf = vec![0_u8; len];
    if offset >= size {
        return Ok((buf, 0));
    }
    let want = usize::try_from((size - offset).min(len as u64)).unwrap_or(len);
    disk.seek(SeekFrom::Start(offset))?;
    let mut got = 0;
    while got < want {
        match disk.read(&mut buf[got..want]) {
            Ok(0) => break,
            Ok(n) => got += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok((buf, got))
}

/// Records of the live logs, which carving does not report again.
#[derive(Default)]
struct Live {
    records: HashSet<[u8; 16]>,
    chunks: HashSet<[u8; 32]>,
    logs: usize,
    unreadable: Vec<String>,
}

impl Live {
    /// Add a log's chunks and allocated records (records in a live chunk's slack are old data the
    /// log no longer shows, so carved copies of them are still reported).
    fn add(&mut self, mut log: impl Read) -> io::Result<()> {
        let mut header = vec![0_u8; 0x1000];
        log.read_exact(&mut header)?;
        if &header[..8] != b"ElfFile\0" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "not an EVTX file",
            ));
        }
        let mut data = vec![0_u8; CHUNK_SIZE];
        loop {
            let mut got = 0;
            while got < CHUNK_SIZE {
                match log.read(&mut data[got..])? {
                    0 => break,
                    n => got += n,
                }
            }
            if got < CHUNK_SIZE {
                break;
            }
            if let Some(header) = Header::parse(&data) {
                self.chunks.insert(Sha256::digest(&data).into());
                let found = chunk::records(&data, &header);
                self.records.extend(found.allocated().map(|r| r.hash));
            }
        }
        self.logs += 1;
        Ok(())
    }
}

/// Strings that many records share, stored once.
#[derive(Default)]
struct Strings {
    values: Vec<String>,
    index: HashMap<String, u32>,
}

impl Strings {
    fn id(&mut self, value: &str) -> u32 {
        if let Some(id) = self.index.get(value) {
            return *id;
        }
        let id = self.values.len() as u32;
        self.values.push(value.to_string());
        self.index.insert(value.to_string(), id);
        id
    }

    fn get(&self, id: u32) -> &str {
        &self.values[id as usize]
    }
}

/// What a rendered record says about itself.
#[derive(Clone, Copy, Debug)]
struct Info {
    event_id: Option<u32>,
    channel: u32,
    provider: u32,
    computer: u32,
}

/// What became of a chunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Status {
    Written,
    LiveLog,
    DuplicateOfLiveLog,
    DuplicateChunk,
    RecordsInLiveLogs,
    RecordsInOtherChunks,
    NoRecords,
}

impl Status {
    fn name(self) -> &'static str {
        match self {
            Status::Written => "Written",
            Status::LiveLog => "SkippedLiveLog",
            Status::DuplicateOfLiveLog => "DuplicateOfLiveLog",
            Status::DuplicateChunk => "DuplicateChunk",
            Status::RecordsInLiveLogs => "RecordsInLiveLogs",
            Status::RecordsInOtherChunks => "RecordsInOtherChunks",
            Status::NoRecords => "NoRecords",
        }
    }
}

/// A chunk that was found.
struct Found {
    source: u32,
    /// Found in the input itself (not in a log rebuilt from it).
    source_is_input: bool,
    offset: u64,
    location: Location,
    volume: Option<usize>,
    compressed: bool,
    truncated: bool,
    header: Header,
    sha: [u8; 32],
    records: Records,
    info: Vec<Option<Info>>,
    channel: u32,
    computer: u32,
    /// Where its bytes are in the spool file.
    spool: Option<u64>,
    status: Status,
    output: Option<u32>,
    repaired: bool,
}

/// Collects the chunks: their bytes go to a spool file in the output directory (carving a disk
/// can find far more chunks than fit in memory) and their records are summarized in memory.
struct Collector {
    chunks: Vec<Found>,
    strings: Strings,
    spool: Option<(File, PathBuf)>,
    spool_len: u64,
    settings: Arc<ParserSettings>,
    seen: HashSet<(u32, u64)>,
}

impl Collector {
    fn new(spool: Option<PathBuf>) -> io::Result<Self> {
        let spool = match spool {
            Some(path) => Some((
                File::options()
                    .read(true)
                    .write(true)
                    .create_new(true)
                    .open(&path)?,
                path,
            )),
            None => None,
        };
        let mut strings = Strings::default();
        strings.id("");
        Ok(Collector {
            chunks: Vec::new(),
            strings,
            spool,
            spool_len: 0,
            settings: Arc::new(ParserSettings::default()),
            seen: HashSet::new(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn add(
        &mut self,
        data: Vec<u8>,
        source: &str,
        offset: u64,
        location: Location,
        volume: Option<usize>,
        compressed: bool,
        truncated: bool,
        from_input: bool,
    ) -> io::Result<()> {
        let Some(header) = Header::parse(&data) else {
            return Ok(());
        };
        let source = self.strings.id(source);
        if !self.seen.insert((source, offset)) {
            return Ok(());
        }
        let records = chunk::records(&data, &header);
        let info = self.render(&data, &records);
        let channel = most_common(info.iter().flatten().map(|i| i.channel))
            .or_else(|| {
                // An empty page has no allocated records to read the channel from: use the name
                // of the log it is in (Windows names a log after its channel, `/` as `%4`).
                let owner = match &location {
                    Location::DeletedLog(path) | Location::LiveLog(path) => path,
                    _ => return None,
                };
                let name = owner.rsplit('/').next()?;
                let stem = name.get(..name.len().checked_sub(5)?)?;
                name.to_ascii_lowercase()
                    .ends_with(".evtx")
                    .then(|| self.strings.id(&stem.replace("%4", "/")))
            })
            .unwrap_or(0);
        let computer = most_common(info.iter().flatten().map(|i| i.computer)).unwrap_or(0);
        let spool = match &mut self.spool {
            Some((file, _)) => {
                file.seek(SeekFrom::Start(self.spool_len))?;
                file.write_all(&data)?;
                let at = self.spool_len;
                self.spool_len += CHUNK_SIZE as u64;
                Some(at)
            }
            None => None,
        };
        self.chunks.push(Found {
            source,
            source_is_input: from_input,
            offset,
            location,
            volume,
            compressed,
            truncated,
            header,
            sha: Sha256::digest(&data).into(),
            records,
            info,
            channel,
            computer,
            spool,
            status: Status::NoRecords,
            output: None,
            repaired: false,
        });
        Ok(())
    }

    /// The channel, provider, computer and event ID of each allocated record, read with the evtx
    /// crate. Carved bytes can be damaged, so a parser panic only loses this chunk's details.
    fn render(&mut self, data: &[u8], records: &Records) -> Vec<Option<Info>> {
        let settings = self.settings.clone();
        let rendered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut out: Vec<(u64, serde_json::Value)> = Vec::new();
            let Ok(mut chunk_data) = EvtxChunkData::new(data.to_vec(), false) else {
                return out;
            };
            let Ok(mut chunk) = chunk_data.parse(settings) else {
                return out;
            };
            for record in chunk.iter() {
                let Ok(record) = record else {
                    break;
                };
                let id = record.event_record_id;
                if let Ok(json) = record.into_json_value() {
                    out.push((id, json.data));
                }
            }
            out
        }))
        .unwrap_or_default();
        let mut by_id: HashMap<u64, Vec<Info>> = HashMap::new();
        for (id, json) in rendered.into_iter().rev() {
            let system = &json["Event"]["System"];
            let text = |v: &serde_json::Value| match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => String::new(),
                other => other.to_string(),
            };
            let event_id = match &system["EventID"] {
                serde_json::Value::Number(n) => n.as_u64(),
                serde_json::Value::String(s) => s.parse().ok(),
                serde_json::Value::Object(o) => o.get("#text").and_then(|t| {
                    t.as_u64()
                        .or_else(|| t.as_str().and_then(|s| s.parse().ok()))
                }),
                _ => None,
            }
            .and_then(|n| u32::try_from(n).ok());
            let info = Info {
                event_id,
                channel: self.strings.id(&text(&system["Channel"])),
                provider: self
                    .strings
                    .id(&text(&system["Provider"]["#attributes"]["Name"])),
                computer: self.strings.id(&text(&system["Computer"])),
            };
            by_id.entry(id).or_default().push(info);
        }
        records
            .records
            .iter()
            .map(|r| {
                if r.slack {
                    None
                } else {
                    by_id.get_mut(&r.id).and_then(Vec::pop)
                }
            })
            .collect()
    }

    fn spooled(&mut self, at: u64) -> io::Result<Vec<u8>> {
        let mut data = vec![0_u8; CHUNK_SIZE];
        let (file, _) = self
            .spool
            .as_mut()
            .ok_or_else(|| io::Error::other("no spool"))?;
        file.seek(SeekFrom::Start(at))?;
        file.read_exact(&mut data)?;
        Ok(data)
    }
}

impl Drop for Collector {
    /// The spool is scratch space: remove it even when the run stops on an error.
    fn drop(&mut self) {
        if let Some((file, path)) = self.spool.take() {
            drop(file);
            let _ = fs::remove_file(path);
        }
    }
}

fn most_common(values: impl Iterator<Item = u32>) -> Option<u32> {
    let mut counts: HashMap<u32, usize> = HashMap::new();
    for v in values.filter(|v| *v != 0) {
        *counts.entry(v).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(v, n)| (*n, std::cmp::Reverse(*v)))
        .map(|(v, _)| v)
}

/// Statistics of a run.
#[derive(Default)]
struct Stats {
    scanned: u64,
    signatures: usize,
    rebuilt_logs: usize,
    problems: Vec<String>,
}

/// Decide which chunks to write: not the live logs' chunks, not a chunk seen before, and not a
/// chunk all of whose records are in the live logs or in a chunk already chosen. Chunks with more
/// records go first, so an older, shorter copy of a chunk adds nothing.
fn choose(chunks: &mut [Found], live: &Live, keep_live: bool) {
    let mut order: Vec<usize> = (0..chunks.len()).collect();
    order.sort_by_key(|&i| (std::cmp::Reverse(chunks[i].records.records.len()), i));
    let mut seen_chunks: HashSet<[u8; 32]> = HashSet::new();
    let mut carved: HashSet<[u8; 16]> = HashSet::new();
    for i in order {
        let found = &mut chunks[i];
        let in_live_log = matches!(found.location, Location::LiveLog(_));
        found.status = if in_live_log && !keep_live {
            Status::LiveLog
        } else if !seen_chunks.insert(found.sha) {
            Status::DuplicateChunk
        } else if live.chunks.contains(&found.sha) && !keep_live {
            Status::DuplicateOfLiveLog
        } else if found.records.records.is_empty() {
            Status::NoRecords
        } else {
            let new = found
                .records
                .records
                .iter()
                .filter(|r| {
                    (keep_live || !live.records.contains(&r.hash)) && !carved.contains(&r.hash)
                })
                .count();
            if new > 0 {
                Status::Written
            } else if found
                .records
                .records
                .iter()
                .all(|r| live.records.contains(&r.hash))
            {
                Status::RecordsInLiveLogs
            } else {
                Status::RecordsInOtherChunks
            }
        };
        if found.status == Status::Written {
            carved.extend(found.records.records.iter().map(|r| r.hash));
        }
    }
}

/// A channel name as a file name, as Windows names logs (`/` becomes `%4`).
fn log_file_name(channel: &str) -> String {
    let mut safe: String = channel
        .replace('/', "%4")
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || " -_.%".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect();
    safe = safe.trim_matches(['.', ' ']).to_string();
    if safe.is_empty() {
        safe = "Unknown".into();
    }
    let upper = safe.to_ascii_uppercase();
    if matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((upper.starts_with("COM") || upper.starts_with("LPT"))
            && upper.len() == 4
            && upper.as_bytes()[3].is_ascii_digit())
    {
        safe.insert(0, '_');
    }
    safe.chars().take(200).collect()
}

/// A path from an NTFS volume as a relative path under the output directory.
fn safe_relative(path: &str) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.split(['/', '\\']).filter(|p| !p.is_empty()) {
        let safe: String = part
            .chars()
            .map(|c| {
                if c.is_control() || "<>:\"|?*".contains(c) {
                    '_'
                } else {
                    c
                }
            })
            .collect();
        let safe = safe.trim_end_matches(['.', ' ']);
        out.push(match safe {
            "" | "." | ".." => "_",
            s => s,
        });
    }
    if out.as_os_str().is_empty() {
        out.push("_");
    }
    out
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct ChunkRow<'a> {
    source: &'a str,
    offset: u64,
    location: &'a str,
    volume: String,
    owner: &'a str,
    status: &'a str,
    output_file: &'a str,
    channel: &'a str,
    computer: &'a str,
    first_record_id: String,
    last_record_id: String,
    first_time: String,
    last_time: String,
    records: usize,
    slack_records: usize,
    damaged_records: usize,
    new_records: usize,
    empty_page: bool,
    compressed: bool,
    truncated: bool,
    header_checksum_valid: bool,
    data_checksum_valid: bool,
    repaired: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct RecordRow<'a> {
    output_file: &'a str,
    source: &'a str,
    chunk_offset: u64,
    location: &'a str,
    record_offset: usize,
    allocation: &'a str,
    event_record_id: u64,
    timestamp: String,
    channel: &'a str,
    event_id: String,
    provider: &'a str,
    computer: &'a str,
    only_in_carved_data: bool,
}

/// CSV or JSON Lines rows.
enum Report {
    Csv(Box<csv::Writer<BufWriter<File>>>),
    Jsonl(BufWriter<File>),
}

impl Report {
    fn create(dir: &Path, stem: &str, kind: ReportType) -> io::Result<Report> {
        Ok(match kind {
            ReportType::Csv => Report::Csv(Box::new(csv::Writer::from_writer(BufWriter::new(
                File::create_new(dir.join(format!("{stem}.csv")))?,
            )))),
            ReportType::Jsonl => Report::Jsonl(BufWriter::new(File::create_new(
                dir.join(format!("{stem}.jsonl")),
            )?)),
        })
    }

    fn write(&mut self, row: &impl Serialize) -> io::Result<()> {
        match self {
            Report::Csv(w) => w.serialize(row).map_err(io::Error::other),
            Report::Jsonl(w) => {
                serde_json::to_writer(&mut *w, row)?;
                w.write_all(b"\n")
            }
        }
    }

    fn finish(self) -> io::Result<()> {
        match self {
            Report::Csv(mut w) => w.flush(),
            Report::Jsonl(mut w) => w.flush(),
        }
    }
}

/// Write the chosen chunks, one log per channel, and the reports. Returns the logs written.
fn write_outputs(
    out: &Path,
    collector: &mut Collector,
    live: &Live,
    kind: ReportType,
) -> io::Result<Vec<(String, usize, usize)>> {
    let carved = out.join("Carved");
    fs::create_dir(&carved)?;
    // Group by channel, in time order.
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, found) in collector.chunks.iter().enumerate() {
        if found.status == Status::Written {
            let channel = collector.strings.get(found.channel);
            groups
                .entry(if channel.is_empty() {
                    "Unknown".into()
                } else {
                    channel.to_string()
                })
                .or_default()
                .push(i);
        }
    }
    let mut used = HashSet::new();
    let mut logs = Vec::new();
    for (channel, mut members) in groups {
        members.sort_by_key(|&i| {
            let found = &collector.chunks[i];
            let first = found.records.allocated().map(|r| r.filetime).min();
            (first.unwrap_or(u64::MAX), found.source, found.offset)
        });
        let stem = log_file_name(&channel);
        for (part, group) in members.chunks(MAX_CHUNKS_PER_FILE).enumerate() {
            let mut name = if part == 0 {
                format!("{stem}.evtx")
            } else {
                format!("{stem}-{}.evtx", part + 1)
            };
            let mut n = 1;
            while !used.insert(name.to_lowercase()) {
                n += 1;
                name = format!("{stem}_{n}.evtx");
            }
            let mut file = BufWriter::new(File::create_new(carved.join(&name))?);
            let next_id = group
                .iter()
                .flat_map(|&i| collector.chunks[i].records.allocated().map(|r| r.id))
                .max()
                .map_or(1, |id| id.saturating_add(1));
            file.write_all(&chunk::file_header(group.len() as u16, next_id))?;
            let mut number = 1;
            let mut records = 0;
            let output = collector.strings.id(&format!("Carved/{name}"));
            for &i in group {
                let at = collector.chunks[i]
                    .spool
                    .ok_or_else(|| io::Error::other("no spool"))?;
                let mut data = collector.spooled(at)?;
                let (next, repaired) =
                    chunk::renumber(&mut data, &collector.chunks[i].records, number);
                number = next;
                file.write_all(&data)?;
                let found = &mut collector.chunks[i];
                found.repaired = repaired;
                found.output = Some(output);
                records += found.records.records.len();
            }
            file.flush()?;
            logs.push((format!("Carved/{name}"), group.len(), records));
        }
    }

    let mut chunk_report = Report::create(out, "chunks", kind)?;
    let mut record_report = Report::create(out, "records", kind)?;
    let strings = &collector.strings;
    for found in &collector.chunks {
        let allocated: Vec<_> = found.records.allocated().collect();
        let times: Vec<u64> = allocated.iter().map(|r| r.filetime).collect();
        let source = strings.get(found.source);
        let output = found.output.map_or("", |o| strings.get(o));
        let new_records = found
            .records
            .records
            .iter()
            .filter(|r| !live.records.contains(&r.hash))
            .count();
        chunk_report.write(&ChunkRow {
            source,
            offset: found.offset,
            location: found.location.name(),
            volume: found.volume.map(|v| v.to_string()).unwrap_or_default(),
            owner: found.location.owner(),
            status: found.status.name(),
            output_file: output,
            channel: strings.get(found.channel),
            computer: strings.get(found.computer),
            first_record_id: allocated
                .iter()
                .map(|r| r.id)
                .min()
                .map(|v| v.to_string())
                .unwrap_or_default(),
            last_record_id: allocated
                .iter()
                .map(|r| r.id)
                .max()
                .map(|v| v.to_string())
                .unwrap_or_default(),
            first_time: times.iter().min().map(|t| filetime(*t)).unwrap_or_default(),
            last_time: times.iter().max().map(|t| filetime(*t)).unwrap_or_default(),
            records: allocated.len(),
            slack_records: found.records.records.len() - allocated.len(),
            damaged_records: found.records.damaged,
            new_records,
            empty_page: found.header.empty,
            compressed: found.compressed,
            truncated: found.truncated,
            header_checksum_valid: found.header.header_checksum_valid,
            data_checksum_valid: found.header.data_checksum_valid,
            repaired: found.repaired,
        })?;
        if found.status != Status::Written {
            continue;
        }
        for (record, info) in found.records.records.iter().zip(&found.info) {
            record_report.write(&RecordRow {
                output_file: output,
                source,
                chunk_offset: found.offset,
                location: found.location.name(),
                record_offset: record.offset,
                allocation: if record.slack { "Slack" } else { "Allocated" },
                event_record_id: record.id,
                timestamp: filetime(record.filetime),
                channel: info.map_or("", |i| strings.get(i.channel)),
                event_id: info
                    .and_then(|i| i.event_id)
                    .map(|e| e.to_string())
                    .unwrap_or_default(),
                provider: info.map_or("", |i| strings.get(i.provider)),
                computer: info.map_or("", |i| strings.get(i.computer)),
                only_in_carved_data: !live.records.contains(&record.hash),
            })?;
        }
    }
    chunk_report.finish()?;
    record_report.finish()?;
    Ok(logs)
}

/// The live logs of a directory.
fn evtx_files(dir: &Path) -> Vec<PathBuf> {
    util::find_files(dir, |name| name.to_ascii_lowercase().ends_with(".evtx"))
}

/// Search `len` bytes from `start` for chunk signatures, and hand each chunk to `found`.
fn scan(
    disk: &mut dyn Disk,
    size: u64,
    start: u64,
    len: u64,
    bar: Option<&ProgressBar>,
    stats: &mut Stats,
    found: &mut dyn FnMut(&mut dyn Disk, u64) -> io::Result<()>,
) -> io::Result<()> {
    let end = start.saturating_add(len).min(size);
    let mut offset = start;
    let overlap = MAGIC.len() - 1;
    while offset < end {
        let want = usize::try_from((end - offset).min(BLOCK as u64)).unwrap_or(BLOCK);
        let (block, got) = read_at(disk, size, offset, want + overlap)?;
        if got == 0 {
            break;
        }
        let scan_len = got.min(want + overlap);
        let hits: Vec<u64> = memchr::memmem::find_iter(&block[..scan_len], MAGIC)
            .filter(|&i| i < want)
            .map(|i| offset + i as u64)
            .collect();
        for hit in hits {
            stats.signatures += 1;
            found(disk, hit)?;
        }
        let step = want.min(got) as u64;
        offset += step;
        stats.scanned += step;
        if let Some(b) = bar {
            b.inc(step);
        }
        if (got as u64) < want as u64 {
            break;
        }
    }
    Ok(())
}

/// The chunk at a signature: as it is, or decompressed from NTFS compression around it.
fn extract(
    disk: &mut dyn Disk,
    size: u64,
    offset: u64,
    cluster: u64,
    grid: u64,
) -> io::Result<Option<(Vec<u8>, bool, bool)>> {
    let (data, got) = read_at(disk, size, offset, CHUNK_SIZE)?;
    if Header::parse(&data).is_some() {
        return Ok(Some((data, false, got < CHUNK_SIZE)));
    }
    if offset < LZNT1_PREFIX {
        return Ok(None);
    }
    let start = offset - LZNT1_PREFIX;
    let window = CHUNK_SIZE + 2 * cluster as usize + 0x2000;
    let (raw, got) = read_at(disk, size, start, window)?;
    let grid_offset = usize::try_from(start.saturating_sub(grid)).unwrap_or(0);
    Ok(
        chunk::decompress_chunk(&raw[..got], cluster as usize, grid_offset)
            .filter(|(data, _)| Header::parse(data).is_some())
            .map(|(data, present)| (data, true, present < CHUNK_SIZE)),
    )
}

/// Run the command; returns the exit status.
pub fn run(opts: &Options) -> i32 {
    let no_color = opts.common.no_color;
    let report = |bar: Option<&ProgressBar>, msg: &str| {
        progress::suspended(bar, || error_msg(no_color, msg));
    };
    match carve(opts, &report) {
        Ok(()) => 0,
        Err(e) => {
            report(None, &e);
            1
        }
    }
}

fn carve(opts: &Options, report: &dyn Fn(Option<&ProgressBar>, &str)) -> Result<(), String> {
    let no_color = opts.common.no_color;
    if !opts.file.is_file() {
        return Err(format!("The file does not exist: {}", opts.file.display()));
    }
    if let Some(dir) = &opts.live_logs
        && !dir.is_dir()
    {
        return Err(format!("The directory does not exist: {}", dir.display()));
    }
    if let Some(out) = &opts.output {
        util::check_output(out, true, false)?;
    }
    let mut input = open_input(&opts.file, opts.no_file_system)
        .map_err(|e| format!("Could not open {}: {e}", opts.file.display()))?;
    if opts.free_space_only && input.image.is_none() {
        return Err("--free-space-only needs an input with an NTFS volume".into());
    }
    // Only now that the input opened, so that a bad input leaves no results directory behind.
    if let Some(out) = &opts.output {
        fs::create_dir_all(out).map_err(|e| format!("Could not create {}: {e}", out.display()))?;
    }
    let source_name = opts
        .file
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    p_label(
        no_color,
        "Input: ",
        &format!("{} ({})", opts.file.display(), input.kind),
    );
    p_label(no_color, "Size: ", &util::size_display(input.size));
    for warning in &input.warnings {
        report(None, &format!("{}: {warning}", opts.file.display()));
    }

    // --- volumes and live logs -------------------------------------------------------------
    let mut volumes = Vec::new();
    let mut live = Live::default();
    if let Some(image) = &input.image {
        for index in 0..image.volumes.len() {
            match Volume::open(image, index) {
                Ok(v) => volumes.push(v),
                Err(e) => report(None, &format!("NTFS volume {}: {e}", index + 1)),
            }
        }
        for volume in &mut volumes {
            for (record, path) in volume.live_log_paths() {
                let added = volume.open_file(record).and_then(|f| live.add(f));
                if let Err(e) = added {
                    live.unreadable
                        .push(format!("Volume {} {path}: {e}", volume.number));
                }
            }
        }
        p_label(no_color, "NTFS volumes: ", &fmt_num(volumes.len()));
    }
    if let Some(dir) = &opts.live_logs {
        for path in evtx_files(dir) {
            let added = File::open(&path).and_then(|f| live.add(io::BufReader::new(f)));
            if let Err(e) = added {
                live.unreadable.push(format!("{}: {e}", path.display()));
            }
        }
    }
    p_label(no_color, "Live logs: ", &fmt_num(live.logs));
    for problem in live.unreadable.iter().take(MAX_PRINTED) {
        report(None, &format!("Could not read the live log {problem}"));
    }
    let deleted: usize = volumes.iter().map(|v| v.deleted_logs.len()).sum();
    if input.image.is_some() {
        p_label(no_color, "Deleted logs in $MFT: ", &fmt_num(deleted));
    }
    p(None, "", true);

    let spool = opts.output.as_ref().map(|o| o.join(".carve-evtx-spool"));
    let mut collector = Collector::new(spool.clone())
        .map_err(|e| format!("Could not create the spool file: {e}"))?;
    let mut stats = Stats::default();

    // --- deleted logs, rebuilt whole --------------------------------------------------------
    for volume in &volumes {
        for log in &volume.deleted_logs {
            if let Some(problem) = &log.problem {
                stats.problems.push(format!(
                    "Volume {} {} ($MFT record {}): not rebuilt: {problem}",
                    volume.number, log.path, log.record
                ));
                continue;
            }
            let relative =
                PathBuf::from(format!("Volume{}", volume.number)).join(safe_relative(&log.path));
            let (mut file, label) = match &opts.output {
                Some(out) => {
                    let mut path = out.join("Recovered").join(&relative);
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    let mut n = 1;
                    while path.exists() {
                        n += 1;
                        path.set_file_name(format!(
                            "{}_{n}.evtx",
                            relative.file_stem().unwrap_or_default().to_string_lossy()
                        ));
                    }
                    let file = File::options()
                        .read(true)
                        .write(true)
                        .create_new(true)
                        .open(&path)
                        .map_err(|e| format!("Could not create {}: {e}", path.display()))?;
                    let label = path
                        .strip_prefix(out)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .replace('\\', "/");
                    (file, label)
                }
                None => (
                    tempfile::tempfile().map_err(|e| e.to_string())?,
                    format!(
                        "Recovered/{}",
                        relative.to_string_lossy().replace('\\', "/")
                    ),
                ),
            };
            let rebuilt = volume.rebuild(log, &mut input.disk, &mut file);
            match rebuilt {
                Ok(len) => {
                    stats.rebuilt_logs += 1;
                    let location = Location::DeletedLog(log.path.clone());
                    let mut file_disk: Box<dyn Disk> = Box::new(file);
                    let mut offset = 0x1000;
                    while offset + CHUNK_SIZE as u64 <= len {
                        let (data, _) = read_at(&mut *file_disk, len, offset, CHUNK_SIZE)
                            .map_err(|e| e.to_string())?;
                        collector
                            .add(
                                data,
                                &label,
                                offset,
                                location.clone(),
                                Some(volume.number),
                                false,
                                false,
                                false,
                            )
                            .map_err(|e| format!("Could not write the spool file: {e}"))?;
                        offset += CHUNK_SIZE as u64;
                    }
                }
                Err(e) => stats.problems.push(format!(
                    "Volume {} {} ($MFT record {}): could not be rebuilt: {e}",
                    volume.number, log.path, log.record
                )),
            }
        }
    }

    // --- the signature search ----------------------------------------------------------------
    let ranges: Vec<(u64, u64)> = if opts.free_space_only {
        volumes.iter().flat_map(Volume::free_ranges).collect()
    } else {
        vec![(0, input.size)]
    };
    let total: u64 = ranges.iter().map(|r| r.1).sum();
    p(
        JunteiColor::Orange.rdg(no_color),
        "Carving in progress. Please wait.",
        true,
    );
    p(None, "", true);
    let bar = progress::progress_bar_bytes(total, no_color);
    let size = input.size;
    let default_cluster = u64::from(opts.cluster_size);
    let mut add_error = None;
    {
        let volumes = &mut volumes;
        let collector = &mut collector;
        let add_error = &mut add_error;
        let mut on_hit = |disk: &mut dyn Disk, offset: u64| -> io::Result<()> {
            let volume = volumes.iter_mut().find(|v| v.contains(offset));
            let (cluster, grid) = volume
                .as_ref()
                .map_or((default_cluster, 0), |v| (v.cluster, v.start));
            let Some((data, compressed, truncated)) = extract(disk, size, offset, cluster, grid)?
            else {
                return Ok(());
            };
            // A compressed chunk starts at its LZNT1 block header.
            let at = if compressed {
                offset - LZNT1_PREFIX
            } else {
                offset
            };
            let (location, number) = match volume {
                Some(v) => (v.classify(at), Some(v.number)),
                None if input.image.is_some() => (Location::OutsideVolumes, None),
                None => (Location::Unclassified, None),
            };
            if let Err(e) = collector.add(
                data,
                &source_name,
                at,
                location,
                number,
                compressed,
                truncated,
                true,
            ) {
                *add_error = Some(e.to_string());
                return Err(e);
            }
            Ok(())
        };
        for (start, len) in &ranges {
            if let Err(e) = scan(
                &mut *input.disk,
                size,
                *start,
                *len,
                Some(&bar),
                &mut stats,
                &mut on_hit,
            ) {
                bar.abandon();
                return Err(match add_error.take() {
                    Some(e) => format!("Could not write the spool file: {e}"),
                    None => format!("Could not read {}: {e}", opts.file.display()),
                });
            }
        }
    }
    bar.finish_with_message(progress::finished_message(no_color));

    choose(&mut collector.chunks, &live, opts.keep_live);

    // --- output --------------------------------------------------------------------------------
    let logs = match &opts.output {
        Some(out) => {
            let logs = write_outputs(out, &mut collector, &live, opts.output_type)
                .map_err(|e| format!("Could not write the results: {e}"))?;
            if let Some((file, path)) = collector.spool.take() {
                drop(file); // closed before it is removed, as Windows requires
                fs::remove_file(&path)
                    .map_err(|e| format!("Could not remove {}: {e}", path.display()))?;
            }
            logs
        }
        None => Vec::new(),
    };

    // --- summary -------------------------------------------------------------------------------
    for problem in stats.problems.iter().take(MAX_PRINTED) {
        report(None, problem);
    }
    if stats.problems.len() > MAX_PRINTED {
        report(
            None,
            &format!(
                "{} more problems",
                fmt_num(stats.problems.len() - MAX_PRINTED)
            ),
        );
    }
    print_summary(&collector, &stats, &logs, opts, no_color);
    Ok(())
}

/// The total size of the files under a directory.
fn dir_size(dir: &Path) -> u64 {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

fn print_summary(
    collector: &Collector,
    stats: &Stats,
    logs: &[(String, usize, usize)],
    opts: &Options,
    no_color: bool,
) {
    let chunks = &collector.chunks;
    let count = |f: &dyn Fn(&Found) -> bool| fmt_num(chunks.iter().filter(|c| f(c)).count());
    p(None, "", true);
    p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
    p(None, "", true);
    p_label(
        no_color,
        "Bytes searched: ",
        &util::size_display(stats.scanned),
    );
    p_label(no_color, "Chunk signatures: ", &fmt_num(stats.signatures));
    p_label(
        no_color,
        "Chunks found by the search: ",
        &count(&|c| !matches!(c.location, Location::DeletedLog(_)) || c.source_is_input),
    );
    p_label(no_color, "NTFS-compressed: ", &count(&|c| c.compressed));
    p_label(
        no_color,
        "Damaged (checksum mismatch): ",
        &count(&|c| !c.header.header_checksum_valid || !c.header.data_checksum_valid),
    );
    if stats.rebuilt_logs > 0 {
        p_label(
            no_color,
            "Deleted logs rebuilt: ",
            &fmt_num(stats.rebuilt_logs),
        );
        p_label(
            no_color,
            "Chunks in rebuilt logs: ",
            &count(&|c| matches!(c.location, Location::DeletedLog(_)) && !c.source_is_input),
        );
    }
    p(None, "", true);
    let mut by_location: BTreeMap<&str, usize> = BTreeMap::new();
    for c in chunks {
        *by_location.entry(c.location.name()).or_default() += 1;
    }
    p(
        JunteiColor::Green.rdg(no_color),
        "Where the chunks were:",
        true,
    );
    for (name, n) in &by_location {
        p_label(no_color, &format!("  {name}: "), &fmt_num(*n));
    }
    p(None, "", true);
    let mut by_status: BTreeMap<Status, usize> = BTreeMap::new();
    for c in chunks {
        *by_status.entry(c.status).or_default() += 1;
    }
    p(
        JunteiColor::Green.rdg(no_color),
        "What became of them:",
        true,
    );
    for (status, n) in &by_status {
        p_label(no_color, &format!("  {}: ", status.name()), &fmt_num(*n));
    }
    p(None, "", true);
    let written: Vec<&Found> = chunks
        .iter()
        .filter(|c| c.status == Status::Written)
        .collect();
    let records: usize = written.iter().map(|c| c.records.records.len()).sum();
    let mut channels: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for c in &written {
        let entry = channels
            .entry(collector.strings.get(c.channel))
            .or_default();
        entry.0 += 1;
        entry.1 += c.records.records.len();
    }
    p_label(no_color, "Chunks to keep: ", &fmt_num(written.len()));
    p_label(no_color, "Records in them: ", &fmt_num(records));
    if !channels.is_empty() {
        p(JunteiColor::Green.rdg(no_color), "By channel:", true);
        let mut list: Vec<_> = channels.into_iter().collect();
        list.sort_by_key(|(name, (_, n))| (std::cmp::Reverse(*n), *name));
        for (name, (c, n)) in list.iter().take(MAX_PRINTED) {
            let name = if name.is_empty() { "Unknown" } else { name };
            p_label(
                no_color,
                &format!("  {name}: "),
                &format!("{} records in {} chunks", fmt_num(*n), fmt_num(*c)),
            );
        }
    }
    p(None, "", true);
    match &opts.output {
        Some(out) => {
            p_label(
                no_color,
                "Results saved: ",
                &format!(
                    "{} ({} logs in Carved, {})",
                    out.display(),
                    fmt_num(logs.len()),
                    util::size_display(dir_size(out))
                ),
            );
            if !logs.is_empty() {
                p(
                    None,
                    &format!(
                        "Analyze the rebuilt logs with Hayabusa: hayabusa csv-timeline -d {} -a -o timeline.csv",
                        out.join("Carved").display()
                    ),
                    true,
                );
            }
        }
        None => p(
            None,
            "To save the rebuilt logs and the chunk and record reports, add -o <DIR>.",
            true,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_file_names() {
        assert_eq!(log_file_name("Security"), "Security");
        assert_eq!(
            log_file_name("Microsoft-Windows-Sysmon/Operational"),
            "Microsoft-Windows-Sysmon%4Operational"
        );
        assert_eq!(log_file_name("Windows PowerShell"), "Windows PowerShell");
        assert_eq!(log_file_name("../x:y"), "%4x_y");
        assert_eq!(log_file_name(""), "Unknown");
        assert_eq!(log_file_name("CON"), "_CON");
    }

    #[test]
    fn safe_relative_paths_stay_inside() {
        assert_eq!(
            safe_relative("C/Windows/System32/winevt/Logs/Security.evtx"),
            PathBuf::from("C/Windows/System32/winevt/Logs/Security.evtx")
        );
        assert_eq!(
            safe_relative("../../etc/passwd"),
            PathBuf::from("_/_/etc/passwd")
        );
        assert_eq!(safe_relative("a:b/c."), PathBuf::from("a_b/c"));
        assert_eq!(safe_relative(""), PathBuf::from("_"));
    }
}
