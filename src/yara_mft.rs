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

//! Scan every resident `$DATA` stream of a collected MFT as a file of its own, reading one FILE
//! record at a time, and save only the streams that match. Nonresident data is counted, never
//! invented, and a damaged record is reported rather than skipped silently.
use super::{Candidate, Detection, FileResult, Scanner, evaluate, hash};
use crate::interrupt;
use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    time::Instant,
};
use ys_ntfs_parser::mft::{MftParser, attribute::MftAttributeType};

pub(super) const HEADERS: &[&str] = &[
    "Computer",
    "MFTPath",
    "EntryNum",
    "SN",
    "BaseEntryNum",
    "BaseSN",
    "OriginalPath",
    "Stream",
    "AttributeID",
    "AttributeOffset",
    "Deleted",
    "Rules",
    "Status",
    "Size",
    "SHA256",
    "RecoveredFile",
    "Note",
];
/// Damaged records beyond this many get no row of their own; `Summary.json` keeps the count.
const DAMAGED_ROWS: u64 = 100;
/// How often the record loop looks for Ctrl+C.
const INTERRUPT_EVERY: u64 = 4096;

#[derive(Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Stats {
    pub files: usize,
    pub records: u64,
    pub unused_records: u64,
    pub damaged_records: u64,
    pub resident_streams: u64,
    pub nonresident_streams: u64,
    pub recovered_files: u64,
    pub issues: usize,
    pub elapsed_ms: u128,
    pub incomplete: bool,
}
#[derive(Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Recovery {
    computer: String,
    #[serde(rename = "MFTPath")]
    mft_path: String,
    entry_num: Option<u64>,
    #[serde(rename = "SN")]
    sn: Option<u16>,
    base_entry_num: Option<u64>,
    #[serde(rename = "BaseSN")]
    base_sn: Option<u16>,
    original_path: String,
    stream: String,
    #[serde(rename = "AttributeID")]
    attribute_id: Option<u16>,
    attribute_offset: Option<u64>,
    deleted: Option<bool>,
    rules: String,
    status: String,
    size: Option<u64>,
    #[serde(rename = "SHA256")]
    sha256: String,
    recovered_file: String,
    note: String,
}
impl Recovery {
    fn source(file: &FileResult, status: &str, note: &str) -> Self {
        Self {
            computer: file.computer.clone(),
            mft_path: file.file_path.clone(),
            status: status.into(),
            note: note.into(),
            ..Self::default()
        }
    }
}
/// Standard collected names are recognized even with damaged leading records. A renamed MFT
/// can also be identified from a plausible FILE header, without parsing every input file.
pub(super) fn is_mft(c: &Candidate) -> bool {
    if c.pre_status.is_some() {
        return false;
    }
    let name = c
        .path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();
    if crate::mft::is_mft_name(&name)
        || name.ends_with("%24mft")
        || name == "mft"
        || name.ends_with(".mft")
    {
        return true;
    }
    if c.size < 512 {
        return false;
    }
    let mut header = [0u8; 32];
    if std::fs::File::open(&c.path)
        .and_then(|mut f| f.read_exact(&mut header))
        .is_err()
    {
        return false;
    }
    let size = u32::from_le_bytes(header[28..32].try_into().expect("header"));
    let usa = u16::from_le_bytes(header[6..8].try_into().expect("header"));
    &header[..4] == b"FILE"
        && size.is_power_of_two()
        && (512..=1024 * 1024).contains(&size)
        && u32::from(usa.saturating_sub(1)) * 512 == size
}
fn safe_name(name: &str, limit: usize) -> String {
    let mut out = String::new();
    for c in name.chars() {
        let c = if c.is_control()
            || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        {
            '_'
        } else {
            c
        };
        if out.len() + c.len_utf8() > limit {
            break;
        }
        out.push(c);
    }
    out
}
fn write_recovered(out: &Path, row: &Recovery, instance: u16, data: &[u8]) -> Result<String> {
    let entry = row.entry_num.context("Missing mapped MFT entry")?;
    let sn = row.sn.context("Missing mapped MFT generation")?;
    let id = hash(format!("{}\0{}", row.computer, row.mft_path).as_bytes());
    let directory = format!("RecoveredMFT/{}", &id[..20]);
    let directory_path = out.join(&directory);
    fs::create_dir_all(&directory_path)?;
    // Evidence paths/ADS names are labels only: never recreate their directories or interpret
    // drive letters, '..', separators or reserved Windows names as an output path.
    let basename = row
        .original_path
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("unknown");
    let offset = row
        .attribute_offset
        .context("Missing resident attribute offset")?;
    let name = format!(
        "{entry}-{sn}-{instance}-{offset}_{}_{}.quarantined",
        safe_name(basename, 100),
        safe_name(&row.stream, 60)
    );
    let mut temporary = tempfile::NamedTempFile::new_in(&directory_path)?;
    temporary.write_all(data)?;
    temporary.flush()?;
    temporary
        .persist_noclobber(directory_path.join(&name))
        .map_err(|e| anyhow::anyhow!("Could not save recovered MFT content: {}", e.error))?;
    Ok(format!("{directory}/{name}"))
}

type StreamResult = Option<(FileResult, Vec<Detection>)>;
pub(super) struct Inspection<'a> {
    pub scanner: &'a Scanner,
    pub path: &'a Path,
    pub file: &'a FileResult,
    pub output: &'a Path,
    pub maximum: u64,
}
fn damaged(
    stats: &mut Stats,
    file: &FileResult,
    id: u64,
    note: String,
    emit: &mut impl FnMut(Recovery, StreamResult) -> Result<()>,
) -> Result<()> {
    stats.damaged_records += 1;
    stats.issues += 1;
    stats.incomplete = true;
    if stats.damaged_records <= DAMAGED_ROWS {
        let mut row = Recovery::source(file, "InvalidRecord", &note);
        row.entry_num = Some(id);
        emit(row, None)?;
    } else if stats.damaged_records == DAMAGED_ROWS + 1 {
        emit(
            Recovery::source(
                file,
                "InvalidRecord",
                "Further damaged records are only counted in Summary.json.",
            ),
            None,
        )?;
    }
    Ok(())
}
/// Read the MFT one FILE record at a time from the file itself (never from a buffer of the
/// whole MFT) and scan every resident `$DATA` stream as an independent file. This is what makes
/// `filesize`, header checks and other compound conditions refer to the file a rule was written
/// for, and it covers every record rather than only those under a raw match.
pub(super) fn inspect(
    context: Inspection<'_>,
    stats: &mut Stats,
    mut emit: impl FnMut(Recovery, StreamResult) -> Result<()>,
) -> Result<()> {
    let Inspection {
        scanner,
        path,
        file,
        output,
        maximum,
    } = context;
    let start = Instant::now();
    let parsed = (|| -> Result<(u64, MftParser<File>)> {
        let handle = File::open(path)?;
        let len = handle.metadata()?.len();
        Ok((len, MftParser::from_read_seek(handle, Some(len))?))
    })();
    let (len, mut parser) = match parsed {
        Ok(v) => v,
        Err(e) => {
            stats.issues += 1;
            stats.incomplete = true;
            emit(
                Recovery::source(
                    file,
                    "InvalidMFT",
                    &format!("Cannot identify MFT records: {e:#}"),
                ),
                None,
            )?;
            return Ok(());
        }
    };
    let count = parser.get_entry_count();
    if count == 0 {
        stats.issues += 1;
        stats.incomplete = true;
        emit(
            Recovery::source(file, "InvalidMFT", "MFT has no complete records"),
            None,
        )?;
        return Ok(());
    }
    // The parser accepts power-of-two FILE record sizes. If L = n*S+r, n>=1 and 0<=r<S,
    // then S <= L/n < 2*S. Rounding L/n down to a power of two therefore yields exactly S,
    // including when a partial record remains at EOF. Each parsed header is validated below.
    let ratio = len / count;
    let record_size = 1u64 << (63 - ratio.leading_zeros());
    for id in 0..count {
        if id % INTERRUPT_EVERY == 0 && interrupt::requested() {
            stats.incomplete = true;
            emit(
                Recovery::source(file, "Cancelled", "Resident scan cancelled"),
                None,
            )?;
            break;
        }
        stats.records += 1;
        let entry = match parser.get_entry(id) {
            Ok(e) if !e.header.is_valid() => {
                if e.header.signature == [0; 4] {
                    stats.unused_records += 1;
                } else {
                    damaged(
                        stats,
                        file,
                        id,
                        format!(
                            "Record signature {:?} is not FILE; content not scanned.",
                            String::from_utf8_lossy(&e.header.signature)
                        ),
                        &mut emit,
                    )?;
                }
                continue;
            }
            Ok(e)
                if u64::from(e.header.total_entry_size) != record_size
                    || e.valid_fixup != Some(true) =>
            {
                damaged(
                    stats,
                    file,
                    id,
                    "Record size or update-sequence fixups are invalid; content not scanned."
                        .into(),
                    &mut emit,
                )?;
                continue;
            }
            Ok(e) => e,
            Err(e) => {
                damaged(stats, file, id, e.to_string(), &mut emit)?;
                continue;
            }
        };
        let mut row = Recovery::source(file, "", "");
        row.entry_num = Some(id);
        row.sn = Some(entry.header.sequence);
        row.deleted = Some(!entry.is_allocated());
        // Only use an extension record's base path when its reference generation and the
        // base's ATTRIBUTE_LIST both identify this exact extension. Never attach stale data
        // to the current generation of a recycled base record.
        let mut owner = None;
        if !entry.header.is_base_record() {
            if let Ok(base) = parser.get_entry(entry.header.base_reference.entry)
                && base.header.sequence == entry.header.base_reference.sequence
                && base.header.is_base_record()
                && base.valid_fixup == Some(true)
                && parser.get_extension_entries(&base).iter().any(|e| {
                    e.header.record_number == entry.header.record_number
                        && e.header.sequence == entry.header.sequence
                })
            {
                row.base_entry_num = Some(base.header.record_number);
                row.base_sn = Some(base.header.sequence);
                owner = Some(base);
            } else {
                row.note =
                    "Unverified/stale extension owner; base-file path is not asserted.".into();
            }
        }
        let path = if entry.header.is_base_record() || owner.is_some() {
            parser
                .get_full_path_for_entry(owner.as_ref().unwrap_or(&entry))
                .ok()
                .flatten()
        } else {
            None
        };
        row.original_path = path
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|| format!("[Unresolved]/{id}-{}", entry.header.sequence));
        for attribute in entry.iter_attributes() {
            let attribute = match attribute {
                Ok(a) => a,
                Err(e) => {
                    stats.issues += 1;
                    stats.incomplete = true;
                    let mut r = Recovery::source(file, "AttributeError", &e.to_string());
                    r.entry_num = Some(id);
                    r.sn = row.sn;
                    emit(r, None)?;
                    continue;
                }
            };
            if attribute.header.type_code != MftAttributeType::DATA {
                continue;
            }
            // Nonresident DATA refers to volume clusters; its bytes are not in a standalone
            // MFT, so it is counted and never replaced by a placeholder.
            let Some(data) = attribute.data.as_data() else {
                stats.nonresident_streams += 1;
                continue;
            };
            let data = data.data();
            stats.resident_streams += 1;
            let mut stream_row = Recovery {
                computer: row.computer.clone(),
                mft_path: row.mft_path.clone(),
                entry_num: row.entry_num,
                sn: row.sn,
                base_entry_num: row.base_entry_num,
                base_sn: row.base_sn,
                original_path: row.original_path.clone(),
                stream: attribute.header.name.to_utf8_string(),
                attribute_id: Some(attribute.header.instance),
                attribute_offset: Some(attribute.header.start_offset),
                deleted: row.deleted,
                size: Some(data.len() as u64),
                sha256: hash(data),
                note: row.note.clone(),
                ..Recovery::default()
            };
            let mut result = FileResult {
                computer: file.computer.clone(),
                file_path: format!(
                    "{}::MFT({id}-{})::{}:{}",
                    file.file_path,
                    entry.header.sequence,
                    stream_row.original_path,
                    stream_row.stream
                ),
                size: data.len() as u64,
                sha256: stream_row.sha256.clone(),
                status: String::new(),
                matches: 0,
                elapsed_ms: 0,
                note: String::new(),
                source_type: "MFTResident".into(),
            };
            let began = Instant::now();
            let mut hits = evaluate(scanner, data, &mut result);
            result.elapsed_ms = began.elapsed().as_millis();
            stream_row.rules = hits
                .iter()
                .map(|h| h.rule.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            if result.status != "Scanned" {
                stats.issues += 1;
                stats.incomplete = true;
                stream_row.status = "StreamScanError".into();
                stream_row.note.push_str(&format!(" {}", result.note));
                emit(stream_row, Some((result, hits)))?;
                continue;
            }
            if hits.is_empty() {
                continue;
            }
            if stats.recovered_files >= maximum {
                stats.incomplete = true;
                stream_row.status = "RecoveryLimitReached".into();
                stream_row.note.push_str(
                    " Maximum recovered streams per run reached; matched bytes were not exported.",
                );
            } else {
                let saved = write_recovered(output, &stream_row, attribute.header.instance, data)
                    .context("Saving matched resident DATA")?;
                for h in &mut hits {
                    h.recovered_file = saved.clone();
                }
                stream_row.recovered_file = saved;
                stream_row.status = "Recovered".into();
                stats.recovered_files += 1;
            }
            emit(stream_row, Some((result, hits)))?;
        }
    }
    let tail = len - count * record_size;
    if tail > 0 {
        emit(
            Recovery::source(
                file,
                "TruncatedTail",
                &format!(
                    "{tail} bytes after the last complete FILE record were scanned only as raw MFT bytes; no record can be read from them."
                ),
            ),
            None,
        )?;
    }
    stats.elapsed_ms += start.elapsed().as_millis();
    Ok(())
}
