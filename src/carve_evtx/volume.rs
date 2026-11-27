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

//! The NTFS volumes of a disk, as carving needs them: which clusters are free (`$Bitmap`), which
//! file owns an allocated cluster (the data runs in `$MFT`), the live event logs, and deleted ones
//! whose `$MFT` record still lists clusters that are free.

use std::collections::HashMap;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::sync::Arc;
use vhdx_mount::image::{FileReader, Image};
use ys_ntfs_parser::mft::MftParser;
use ys_ntfs_parser::mft::attribute::header::ResidentialHeader;
use ys_ntfs_parser::mft::attribute::{AttributeDataFlags, MftAttributeType, data_run::RunType};

/// The `$MFT`, `$Bitmap` and `$Boot` records.
const MFT_RECORD: u64 = 0;
const BITMAP_RECORD: u64 = 6;
const BOOT_RECORD: u64 = 7;
/// A deleted log larger than this is not rebuilt (its chunks are still carved).
const MAX_REBUILT: u64 = 4 << 30;

/// Where a chunk was found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Location {
    /// A cluster that no file uses.
    FreeSpace,
    /// A free cluster that a deleted log's `$MFT` record still lists, or a log rebuilt from them.
    DeletedLog(String),
    /// A cluster of an event log that is in use.
    LiveLog(String),
    /// A cluster of another file (a pagefile, a shadow copy's store file, a copied log, ...).
    AllocatedFile(String),
    /// An allocated cluster that no `$MFT` record lists (file system metadata or damage).
    AllocatedUnknown,
    /// Outside every NTFS volume of the disk.
    OutsideVolumes,
    /// The input has no file system to tell.
    Unclassified,
}

impl Location {
    pub fn name(&self) -> &'static str {
        match self {
            Location::FreeSpace => "FreeSpace",
            Location::DeletedLog(_) => "DeletedLog",
            Location::LiveLog(_) => "LiveLog",
            Location::AllocatedFile(_) => "AllocatedFile",
            Location::AllocatedUnknown => "AllocatedUnknown",
            Location::OutsideVolumes => "OutsideVolumes",
            Location::Unclassified => "Unclassified",
        }
    }

    pub fn owner(&self) -> &str {
        match self {
            Location::DeletedLog(p) | Location::LiveLog(p) | Location::AllocatedFile(p) => p,
            _ => "",
        }
    }
}

/// Clusters of one stream.
#[derive(Clone, Copy, Debug)]
struct Extent {
    lcn: u64,
    len: u64,
    owner: u32,
}

/// A stream that owns clusters.
#[derive(Debug)]
struct Owner {
    record: u64,
    stream: String,
    path: Option<String>,
    log: bool,
}

/// A deleted event log whose `$MFT` record is still readable.
#[derive(Clone, Debug)]
pub struct DeletedLog {
    pub record: u64,
    pub path: String,
    /// Its data runs in VCN order: `None` for a sparse run.
    runs: Vec<(Option<u64>, u64)>,
    pub size: u64,
    /// Clusters per compression unit, if the stream was compressed.
    compression_unit: Option<u64>,
    /// Why it can't be rebuilt, if it can't.
    pub problem: Option<String>,
}

/// One NTFS volume of a disk.
pub struct Volume {
    /// 1-based, as listed.
    pub number: usize,
    /// Its first byte on the disk, and its length.
    pub start: u64,
    pub len: u64,
    pub cluster: u64,
    bitmap: Vec<u8>,
    extents: Vec<Extent>,
    deleted_extents: Vec<Extent>,
    owners: Vec<Owner>,
    /// `$MFT` records of the event logs in use.
    pub live_logs: Vec<u64>,
    pub deleted_logs: Vec<DeletedLog>,
    parser: MftParser<FileReader>,
    image: Arc<Image>,
    index: usize,
}

fn is_log_name(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".evtx")
}

impl Volume {
    /// Read the volume's `$Boot`, `$Bitmap` and `$MFT`.
    pub fn open(image: &Arc<Image>, index: usize) -> io::Result<Volume> {
        let volume = &image.volumes[index];
        let boot = volume.read(BOOT_RECORD, 0, 512)?;
        if boot.len() < 512 {
            return Err(io::Error::other("$Boot is too short"));
        }
        let sector = u64::from(u16::from_le_bytes([boot[0x0b], boot[0x0c]]));
        let per_cluster = boot[0x0d];
        let clusters = if per_cluster > 0x80 {
            1_u64.checked_shl(u32::from(256 - u16::from(per_cluster)))
        } else {
            Some(u64::from(per_cluster))
        };
        let cluster = clusters
            .and_then(|c| c.checked_mul(sector))
            .filter(|c| c.is_power_of_two() && (512..=2 << 20).contains(c))
            .ok_or_else(|| io::Error::other("$Boot has an invalid cluster size"))?;
        let mut bitmap = Vec::new();
        image
            .file_reader(index, BITMAP_RECORD)?
            .read_to_end(&mut bitmap)?;
        let mft = image.file_reader(index, MFT_RECORD)?;
        let len = mft.len();
        let parser = MftParser::from_read_seek(mft, Some(len)).map_err(io::Error::other)?;
        let mut out = Volume {
            number: index + 1,
            start: volume.start,
            len: volume.len,
            cluster,
            bitmap,
            extents: Vec::new(),
            deleted_extents: Vec::new(),
            owners: Vec::new(),
            live_logs: Vec::new(),
            deleted_logs: Vec::new(),
            parser,
            image: image.clone(),
            index,
        };
        out.map_streams();
        Ok(out)
    }

    /// Record which stream owns every allocated run, the live logs, and the deleted ones.
    fn map_streams(&mut self) {
        let mut owners: HashMap<(u64, String), u32> = HashMap::new();
        let mut deleted = Vec::new();
        let count = self.parser.get_entry_count();
        for number in 0..count {
            let Ok(entry) = self.parser.get_entry(number) else {
                continue;
            };
            if entry.valid_fixup != Some(true) || &entry.header.signature != b"FILE" {
                continue;
            }
            let in_use = entry.is_allocated();
            let base = entry.header.is_base_record();
            let owner_record = if base {
                entry.header.record_number
            } else {
                entry.header.base_reference.entry
            };
            let log_name = base
                && entry
                    .find_best_name_attribute()
                    .is_some_and(|n| is_log_name(&n.name.to_utf8_string()));
            for attribute in entry.iter_attributes().filter_map(Result::ok) {
                if attribute.header.type_code != MftAttributeType::DATA {
                    continue;
                }
                let ResidentialHeader::NonResident(header) = &attribute.header.residential_header
                else {
                    continue;
                };
                let Some(runs) = attribute.data.as_data_runs() else {
                    continue;
                };
                let stream = attribute.header.name.to_utf8_string();
                if in_use {
                    let key = (owner_record, stream.clone());
                    let next = self.owners.len() as u32;
                    let owner = *owners.entry(key).or_insert_with(|| {
                        self.owners.push(Owner {
                            record: owner_record,
                            stream: stream.clone(),
                            path: None,
                            log: false,
                        });
                        next
                    });
                    if log_name && stream.is_empty() {
                        self.owners[owner as usize].log = true;
                        if header.vnc_first == 0 {
                            self.live_logs.push(owner_record);
                        }
                    }
                    for run in &runs.data_runs {
                        if run.run_type == RunType::Standard && run.lcn_length > 0 {
                            self.extents.push(Extent {
                                lcn: run.lcn_offset,
                                len: run.lcn_length,
                                owner,
                            });
                        }
                    }
                } else if log_name && stream.is_empty() && header.vnc_first == 0 {
                    let runs: Vec<_> = runs
                        .data_runs
                        .iter()
                        .map(|r| {
                            let lcn = (r.run_type == RunType::Standard).then_some(r.lcn_offset);
                            (lcn, r.lcn_length)
                        })
                        .collect();
                    let compressed = attribute
                        .header
                        .data_flags
                        .contains(AttributeDataFlags::IS_COMPRESSED);
                    let covered: u64 = runs.iter().map(|r| r.1).sum();
                    let needed = header.allocated_length.div_ceil(self.cluster);
                    deleted.push((
                        entry.header.record_number,
                        runs,
                        header.file_size,
                        (compressed && header.unit_compression_size > 0)
                            .then(|| 1_u64 << header.unit_compression_size.min(16)),
                        covered < needed,
                        compressed && header.unit_compression_size == 0,
                    ));
                }
            }
        }
        self.extents.sort_by_key(|e| e.lcn);
        for (record, runs, size, compression_unit, incomplete, odd) in deleted {
            let path = self
                .parser
                .get_entry(record)
                .ok()
                .and_then(|e| self.parser.get_full_path_for_entry(&e).ok().flatten())
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|| format!("[Unresolved]/{record}.evtx"));
            let reused = runs
                .iter()
                .filter_map(|(lcn, len)| lcn.map(|l| (l, *len)))
                .any(|(lcn, len)| (lcn..lcn.saturating_add(len)).any(|c| self.allocated(c)));
            let problem = if incomplete {
                Some("Its data runs continue in other $MFT records".to_string())
            } else if reused {
                Some("Some of its clusters were reused".to_string())
            } else if odd {
                Some("It is compressed without a compression unit size".to_string())
            } else if size > MAX_REBUILT {
                Some("It is larger than 4 GiB".to_string())
            } else {
                None
            };
            let owner = self.owners.len() as u32;
            self.owners.push(Owner {
                record,
                stream: String::new(),
                path: Some(path.clone()),
                log: true,
            });
            for (lcn, len) in &runs {
                if let Some(lcn) = lcn {
                    for (start, run) in self.free_parts(*lcn, *len) {
                        self.deleted_extents.push(Extent {
                            lcn: start,
                            len: run,
                            owner,
                        });
                    }
                }
            }
            self.deleted_logs.push(DeletedLog {
                record,
                path,
                runs,
                size,
                compression_unit,
                problem,
            });
        }
        self.deleted_extents.sort_by_key(|e| e.lcn);
    }

    /// Whether a cluster is allocated (clusters past the bitmap count as allocated).
    pub fn allocated(&self, cluster: u64) -> bool {
        usize::try_from(cluster / 8)
            .ok()
            .and_then(|i| self.bitmap.get(i))
            .is_none_or(|byte| byte & (1 << (cluster % 8)) != 0)
    }

    /// The free parts of `len` clusters from `lcn`.
    fn free_parts(&self, lcn: u64, len: u64) -> Vec<(u64, u64)> {
        let mut parts = Vec::new();
        let mut start = None;
        for c in lcn..lcn.saturating_add(len) {
            match (self.allocated(c), start) {
                (false, None) => start = Some(c),
                (true, Some(s)) => {
                    parts.push((s, c - s));
                    start = None;
                }
                _ => {}
            }
        }
        if let Some(s) = start {
            parts.push((s, lcn + len - s));
        }
        parts
    }

    /// The volume's free clusters as byte ranges on the disk.
    pub fn free_ranges(&self) -> Vec<(u64, u64)> {
        let clusters = self.len / self.cluster;
        let mut ranges = Vec::new();
        let mut start = None;
        for c in 0..clusters {
            match (self.allocated(c), start) {
                (false, None) => start = Some(c),
                (true, Some(s)) => {
                    ranges.push((self.start + s * self.cluster, (c - s) * self.cluster));
                    start = None;
                }
                _ => {}
            }
        }
        if let Some(s) = start {
            ranges.push((self.start + s * self.cluster, (clusters - s) * self.cluster));
        }
        ranges
    }

    pub fn contains(&self, offset: u64) -> bool {
        offset >= self.start && offset - self.start < self.len
    }

    fn find(extents: &[Extent], cluster: u64) -> Option<Extent> {
        let i = extents.partition_point(|e| e.lcn <= cluster);
        let e = *extents.get(i.checked_sub(1)?)?;
        (cluster - e.lcn < e.len).then_some(e)
    }

    /// Where a byte of the disk inside this volume is.
    pub fn classify(&mut self, offset: u64) -> Location {
        let cluster = (offset - self.start) / self.cluster;
        if !self.allocated(cluster) {
            return match Self::find(&self.deleted_extents, cluster) {
                Some(e) => Location::DeletedLog(self.owner_path(e.owner)),
                None => Location::FreeSpace,
            };
        }
        match Self::find(&self.extents, cluster) {
            Some(e) if self.owners[e.owner as usize].log => {
                Location::LiveLog(self.owner_path(e.owner))
            }
            Some(e) => Location::AllocatedFile(self.owner_path(e.owner)),
            None => Location::AllocatedUnknown,
        }
    }

    /// The path of a stream's file (`path:stream` for a named stream), resolved when first used.
    fn owner_path(&mut self, owner: u32) -> String {
        let index = owner as usize;
        if let Some(path) = &self.owners[index].path {
            return path.clone();
        }
        let record = self.owners[index].record;
        let mut path = self
            .parser
            .get_entry(record)
            .ok()
            .and_then(|e| self.parser.get_full_path_for_entry(&e).ok().flatten())
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|| format!("[Unresolved]/{record}"));
        if !self.owners[index].stream.is_empty() {
            path = format!("{path}:{}", self.owners[index].stream);
        }
        self.owners[index].path = Some(path.clone());
        path
    }

    /// The live logs and their paths.
    pub fn live_log_paths(&mut self) -> Vec<(u64, String)> {
        let records = self.live_logs.clone();
        records
            .into_iter()
            .map(|record| {
                let path = self
                    .parser
                    .get_entry(record)
                    .ok()
                    .and_then(|e| self.parser.get_full_path_for_entry(&e).ok().flatten())
                    .map(|p| p.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|| format!("[Unresolved]/{record}.evtx"));
                (record, path)
            })
            .collect()
    }

    /// A live log's content.
    pub fn open_file(&self, record: u64) -> io::Result<FileReader> {
        self.image.file_reader(self.index, record)
    }

    /// Write a deleted log's content, read from its clusters on `disk`, to `out`.
    pub fn rebuild(
        &self,
        log: &DeletedLog,
        disk: &mut (impl Read + Seek),
        out: &mut impl Write,
    ) -> io::Result<u64> {
        let cluster = self.cluster;
        // The VCN-ordered clusters: Some(lcn) or None (sparse).
        let lcns = log
            .runs
            .iter()
            .flat_map(|(lcn, len)| (0..*len).map(move |i| lcn.map(|l| l + i)));
        let mut written = 0_u64;
        let mut write = |out: &mut dyn Write, data: &[u8]| -> io::Result<bool> {
            let take =
                usize::try_from((log.size - written).min(data.len() as u64)).unwrap_or(data.len());
            out.write_all(&data[..take])?;
            written += take as u64;
            Ok(written < log.size)
        };
        let read_cluster = |disk: &mut dyn ReadSeekDyn, lcn: u64| -> io::Result<Vec<u8>> {
            let mut buf = vec![0_u8; cluster as usize];
            disk.seek(SeekFrom::Start(self.start + lcn * cluster))?;
            disk.read_exact(&mut buf)?;
            Ok(buf)
        };
        let unit = log.compression_unit.unwrap_or(1);
        let lcns: Vec<Option<u64>> = lcns.collect();
        for group in lcns.chunks(unit as usize) {
            let allocated: Vec<u64> = group.iter().flatten().copied().collect();
            let data = if log.compression_unit.is_some()
                && !allocated.is_empty()
                && allocated.len() < group.len()
            {
                // A compressed unit: the allocated clusters hold its LZNT1 data.
                let mut raw = Vec::with_capacity(allocated.len() * cluster as usize);
                for lcn in &allocated {
                    raw.extend_from_slice(&read_cluster(disk, *lcn)?);
                }
                let mut data = Vec::with_capacity((unit * cluster) as usize);
                lznt1::decompress(&raw, &mut data)
                    .map_err(|e| io::Error::other(format!("compressed unit: {e:?}")))?;
                data.resize((unit * cluster) as usize, 0);
                data
            } else {
                let mut data = Vec::with_capacity(group.len() * cluster as usize);
                for lcn in group {
                    match lcn {
                        Some(lcn) => data.extend_from_slice(&read_cluster(disk, *lcn)?),
                        None => data.resize(data.len() + cluster as usize, 0),
                    }
                }
                data
            };
            if !write(out, &data)? {
                break;
            }
        }
        Ok(written)
    }
}

/// `Read + Seek` as a trait object.
trait ReadSeekDyn: Read + Seek {}
impl<T: Read + Seek + ?Sized> ReadSeekDyn for T {}
