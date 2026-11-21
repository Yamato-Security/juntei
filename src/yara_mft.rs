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

//! Resolve raw MFT matches without pretending metadata or nonresident data is recovered content.
use super::{Candidate, Detection, FileResult, RawLocation, Scanner, evaluate, hash};
use crate::interrupt;
use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Read, Write},
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
    "RawRules",
    "StreamRules",
    "Status",
    "Size",
    "SHA256",
    "RecoveredFile",
    "Note",
];
#[derive(Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Stats {
    pub files: usize,
    pub candidate_records: usize,
    pub resident_streams_scanned: usize,
    pub recovered_files: u64,
    pub nonresident_unavailable: usize,
    pub unconfirmed_streams: usize,
    pub raw_locations_limited: usize,
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
    raw_rules: String,
    stream_rules: String,
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
    pub snapshot: &'a [u8],
    pub file: &'a FileResult,
    pub raw_hits: &'a [Detection],
    pub locations: &'a [RawLocation],
    pub limited: bool,
    pub output: &'a Path,
    pub maximum: u64,
}
pub(super) fn inspect(
    context: Inspection<'_>,
    stats: &mut Stats,
    mut emit: impl FnMut(Recovery, StreamResult) -> Result<()>,
) -> Result<()> {
    let Inspection {
        scanner,
        snapshot,
        file,
        raw_hits,
        locations,
        limited,
        output,
        maximum,
    } = context;
    let start = Instant::now();
    if limited {
        stats.raw_locations_limited += 1;
        stats.incomplete = true;
        emit(
            Recovery::source(
                file,
                "RawLocationLimit",
                "Raw match-location storage reached its limit; more resident files may match. Raw detections remain available.",
            ),
            None,
        )?;
    }
    let mut parser =
        match MftParser::from_read_seek(Cursor::new(snapshot), Some(snapshot.len() as u64)) {
            Ok(p) => p,
            Err(e) => {
                stats.issues += 1;
                stats.incomplete = true;
                emit(
                    Recovery::source(
                        file,
                        "InvalidMFT",
                        &format!("Cannot identify MFT records: {e}"),
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
    // including when a partial record remains at EOF. Validate each parsed header below.
    let ratio = (snapshot.len() as u64) / count;
    let record_size = 1u64 << (63 - ratio.leading_zeros());
    let mut records: BTreeMap<u64, BTreeSet<String>> = BTreeMap::new();
    for location in locations {
        let Some(end) = location.offset.checked_add(location.length) else {
            continue;
        };
        if location.length == 0 || end > snapshot.len() {
            continue;
        }
        let first = location.offset as u64 / record_size;
        let last = (end as u64 - 1) / record_size;
        if last >= count {
            stats.issues += 1;
            stats.incomplete = true;
            let mut row = Recovery::source(
                file,
                "TruncatedRecord",
                "Raw match touches the incomplete record at EOF; no complete file stream can be recovered from that tail.",
            );
            row.entry_num = Some(count);
            row.raw_rules = location.rule.clone();
            emit(row, None)?;
        }
        // A resident value cannot span FILE records. In particular, do not let a broad
        // whole-container regex nominate hundreds of thousands of records for recovery.
        if first != last {
            let mut row = Recovery::source(
                file,
                "CrossRecordSpan",
                "The matched pattern spans multiple FILE records and cannot belong to one resident DATA value.",
            );
            row.raw_rules = location.rule.clone();
            emit(row, None)?;
            continue;
        }
        for id in first..=last.min(count - 1) {
            if id < count {
                records.entry(id).or_default().insert(location.rule.clone());
            }
        }
    }
    if records.is_empty() {
        let mut row = Recovery::source(
            file,
            "NoMatchOffsets",
            "Raw rule matched without offsets attributable to a complete FILE record; no file content can safely be attributed to this match.",
        );
        row.raw_rules = raw_hits
            .iter()
            .map(|r| r.rule.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        emit(row, None)?;
    }
    for (id, raw_rules) in records {
        if interrupt::requested() {
            stats.incomplete = true;
            emit(
                Recovery::source(file, "Cancelled", "Resident recovery cancelled"),
                None,
            )?;
            break;
        }
        stats.candidate_records += 1;
        let mut row = Recovery::source(file, "", "");
        row.entry_num = Some(id);
        row.raw_rules = raw_rules.into_iter().collect::<Vec<_>>().join("; ");
        let entry = match parser.get_entry(id) {
            Ok(e)
                if e.header.is_valid()
                    && u64::from(e.header.total_entry_size) == record_size
                    && e.valid_fixup == Some(true) =>
            {
                e
            }
            Ok(_) => {
                stats.issues += 1;
                stats.incomplete = true;
                row.status = "InvalidRecord".into();
                row.note = "Record signature, size or update-sequence fixups are invalid; content not exported.".into();
                emit(row, None)?;
                continue;
            }
            Err(e) => {
                stats.issues += 1;
                stats.incomplete = true;
                row.status = "InvalidRecord".into();
                row.note = e.to_string();
                emit(row, None)?;
                continue;
            }
        };
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
        let mut had_data = false;
        for attribute in entry.iter_attributes() {
            let attribute = match attribute {
                Ok(a) => a,
                Err(e) => {
                    stats.issues += 1;
                    stats.incomplete = true;
                    let mut r = Recovery::source(file, "AttributeError", &e.to_string());
                    r.entry_num = Some(id);
                    emit(r, None)?;
                    continue;
                }
            };
            if attribute.header.type_code != MftAttributeType::DATA {
                continue;
            }
            had_data = true;
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
                raw_rules: row.raw_rules.clone(),
                note: row.note.clone(),
                ..Recovery::default()
            };
            let Some(data) = attribute.data.as_data() else {
                stats.nonresident_unavailable += 1;
                stream_row.status = "NonResidentUnavailable".into();
                stream_row.size = Some(attribute.header.value_size());
                stream_row.note.push_str(" Nonresident DATA refers to volume clusters; bytes cannot be recovered from a standalone MFT. No placeholder is exported.");
                emit(stream_row, None)?;
                continue;
            };
            let data = data.data();
            stream_row.size = Some(data.len() as u64);
            stream_row.sha256 = hash(data);
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
            let (mut hits, _, _) = evaluate(scanner, data, &mut result, false);
            result.elapsed_ms = began.elapsed().as_millis();
            stats.resident_streams_scanned += 1;
            stream_row.stream_rules = hits
                .iter()
                .map(|h| h.rule.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            if result.status != "Scanned" {
                stats.incomplete = true;
                stream_row.status = "StreamScanError".into();
                stream_row.note.push_str(&format!(" {}", result.note));
            } else if hits.is_empty() {
                stats.unconfirmed_streams += 1;
                stream_row.status = "ResidentNotConfirmed".into();
            } else if stats.recovered_files >= maximum {
                stats.incomplete = true;
                stream_row.status = "RecoveryLimitReached".into();
                stream_row.note.push_str(
                    " Maximum recovered streams per run reached; matched bytes were not exported.",
                );
            } else {
                let path = write_recovered(output, &stream_row, attribute.header.instance, data)
                    .context("Saving matched resident DATA")?;
                for h in &mut hits {
                    h.recovered_file = path.clone();
                }
                stream_row.recovered_file = path;
                stream_row.status = "Recovered".into();
                stats.recovered_files += 1;
            }
            emit(stream_row, Some((result, hits)))?;
        }
        if !had_data {
            row.status = "MetadataOnly".into();
            row.note.push_str(
                " No resident DATA in the matched record; metadata is not a recovered file.",
            );
            emit(row, None)?;
        }
    }
    stats.elapsed_ms += start.elapsed().as_millis();
    Ok(())
}
