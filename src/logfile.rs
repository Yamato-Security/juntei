//! NTFS $LogFile export. One row per LSN; redo and undo remain separate. There is no
//! transaction timestamp: the times below belong to embedded file metadata.

use crate::cli::{LogfileOptions, RegistryOutputType};
use crate::progress;
use crate::registry::{RecordWriter, input_files, sink};
use crate::util::{self, fmt_num, p, p_label};
use mft::attribute::{MftAttributeType, x30::FileNamespace};
use mft::entry::EntryFlags;
use ntfslog::{Body, Decoded, FileName, Filetime, LogRecord, StandardTimes};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};

const MAX_ERRORS: usize = 20;

pub fn run(opts: &LogfileOptions) -> i32 {
    let stdout = opts.data_on_stdout();
    let report = |message: &str| {
        // Diagnostic output must never contaminate CSV or JSON on stdout.
        eprintln!("{message}");
    };
    let files = match input_files(
        opts.filepath.as_ref(),
        opts.directory.as_ref(),
        "$LogFile files",
        |dir| util::find_files(dir, is_logfile_name),
    ) {
        Ok(files) => files,
        Err(e) => {
            report(&e);
            return 1;
        }
    };
    if let Some(mft) = &opts.mft
        && !mft.is_file()
    {
        report(&format!("The $MFT file does not exist: {}", mft.display()));
        return 1;
    }
    // Resolve auxiliary inputs before opening the sink, including automatically discovered MFTs.
    let mfts: Vec<_> = files
        .iter()
        .map(|f| opts.mft.clone().or_else(|| adjacent_mft(f)))
        .collect();
    let reads = || files.iter().chain(mfts.iter().flatten()).cloned().collect();
    let mut writer =
        match sink(opts.output.as_ref(), opts.clobber, stdout, reads).and_then(|sink| {
            sink.map(|w| {
                RecordWriter::new(
                    w,
                    opts.output_type.unwrap_or(RegistryOutputType::Csv),
                    CSV_HEADER,
                )
            })
            .transpose()
            .map_err(|e| e.to_string())
        }) {
            Ok(w) => w,
            Err(e) => {
                report(&e);
                return 1;
            }
        };
    if !stdout {
        p_label(
            opts.common.no_color,
            "$LogFile files: ",
            &fmt_num(files.len()),
        );
        p_label(
            opts.common.no_color,
            "Total file size: ",
            &util::size_display(
                files
                    .iter()
                    .map(|f| f.metadata().map_or(0, |m| m.len()))
                    .sum(),
            ),
        );
        p(None, "\nParsing in progress. Please wait.\n", true);
    }
    let bar = (!stdout).then(|| progress::progress_bar(files.len(), opts.common.no_color));
    let mut stats = Stats::default();
    for (path, mft) in files.iter().zip(&mfts) {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        let result = read_one(
            path,
            mft.as_deref(),
            opts,
            &mut writer,
            &mut stats,
            &|message| {
                progress::suspended(bar.as_ref(), || report(message));
            },
        );
        if let Err(e) = result {
            progress::suspended(bar.as_ref(), || {
                report(&format!("Could not parse {}: {e}", path.display()))
            });
            stats.failed += 1;
            if e.kind() == io::ErrorKind::WriteZero || e.kind() == io::ErrorKind::BrokenPipe {
                return 1;
            }
        }
        if let Some(b) = &bar {
            b.inc(1);
        }
    }
    if let Some(w) = writer
        && let Err(e) = w.finish()
    {
        report(&format!("Could not finish writing records: {e}"));
        return 1;
    }
    if let Some(b) = &bar {
        b.finish_with_message(progress::finished_message(opts.common.no_color));
    }
    if !stdout {
        stats.print(opts.common.no_color);
        if let Some(out) = &opts.output {
            p_label(
                opts.common.no_color,
                "Results saved: ",
                &format!(
                    "{} ({} records, {})",
                    out.display(),
                    fmt_num(stats.records),
                    util::file_size_display(out)
                ),
            );
        } else {
            p(
                None,
                "To save every record, add -o <FILE> (-t csv, json or jsonl).",
                true,
            );
        }
    }
    i32::from(stats.failed != 0)
}

/// Exact conventional names, optional volume prefix, and .bin exports. Avoid arbitrary
/// log files and registry transaction logs, which are unrelated formats.
pub(crate) fn is_logfile_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    let base = upper.strip_suffix(".BIN").unwrap_or(&upper);
    matches!(base, "$LOGFILE" | "LOGFILE") || base.ends_with("_$LOGFILE")
}

fn adjacent_mft(log: &Path) -> Option<PathBuf> {
    let parent = log.parent()?;
    let name = log.file_name()?.to_string_lossy();
    let bin = name.to_ascii_uppercase().ends_with(".BIN");
    let base = if bin { &name[..name.len() - 4] } else { &name };
    let upper = base.to_ascii_uppercase();
    let stems = if upper == "$LOGFILE" || upper.ends_with("_$LOGFILE") {
        // A prefixed export must never fall back to a different volume's plain $MFT.
        vec![format!("{}$MFT", &base[..base.len() - 8])]
    } else if upper == "LOGFILE" {
        vec!["MFT".to_string(), "$MFT".to_string()]
    } else {
        // -f also permits arbitrary filenames, in a volume's original directory layout.
        vec!["$MFT".to_string()]
    };
    let mut candidates = Vec::new();
    if bin {
        candidates.extend(stems.iter().map(|s| format!("{s}.bin")));
    }
    candidates.extend(stems);
    let files: Vec<_> = std::fs::read_dir(parent)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    for candidate in candidates {
        let mut matches = files.iter().filter(|p| {
            p.file_name()
                .is_some_and(|n| n.eq_ignore_ascii_case(&candidate))
        });
        if let Some(path) = matches.next() {
            // Case-sensitive evidence storage can contain names Windows would treat as equal.
            return matches.next().is_none().then(|| path.clone());
        }
    }
    None
}

#[derive(Default)]
struct Stats {
    files: usize,
    failed: usize,
    unused: usize,
    records: usize,
    updates: usize,
    checkpoints: usize,
    names: usize,
    journal_writes: usize,
    problems: usize,
    inferred: usize,
    mft_failures: usize,
    without_mft: usize,
    operations: BTreeMap<String, usize>,
}

impl Stats {
    fn print(&self, no_color: bool) {
        p(None, "", true);
        for (label, count) in [
            ("Files parsed: ", self.files),
            ("Files failed: ", self.failed),
            ("Unused logs: ", self.unused),
            ("Records: ", self.records),
            ("Updates: ", self.updates),
            ("Checkpoints: ", self.checkpoints),
            ("Embedded filenames (redo + undo): ", self.names),
            ("USN journal writes: ", self.journal_writes),
            ("Inferred targets: ", self.inferred),
            ("Parse problems: ", self.problems),
            ("MFT path failures: ", self.mft_failures),
            ("Logs without a matching MFT: ", self.without_mft),
        ] {
            p_label(no_color, label, &fmt_num(count));
        }
        for (op, count) in &self.operations {
            p_label(no_color, &format!("  {op}: "), &fmt_num(*count));
        }
        p(
            None,
            "File metadata times are not transaction times. Paths from $MFT describe its collected state.",
            true,
        );
    }
}

fn read_one(
    path: &Path,
    mft: Option<&Path>,
    opts: &LogfileOptions,
    writer: &mut Option<RecordWriter>,
    stats: &mut Stats,
    report: &impl Fn(&str),
) -> io::Result<()> {
    let data = std::fs::read(path)?;
    if data.is_empty() {
        return Err(io::Error::other("empty file"));
    }
    let log = ntfslog::parse_with(
        &data,
        &ntfslog::Options {
            cluster_size: opts.cluster_size,
            record_size: opts.record_size,
        },
    );
    drop(data);
    if log.restart_pages.is_empty() && log.records.is_empty() && !log.is_unused() {
        return Err(io::Error::other(
            "no readable NTFS log records or restart pages",
        ));
    }
    stats.files += 1;
    stats.unused += usize::from(log.is_unused());
    stats.problems += log.problems.len();
    for problem in log.problems.iter().take(MAX_ERRORS) {
        report(&format!("{}: {problem}", path.display()));
    }
    if log.problems.len() > MAX_ERRORS {
        report(&format!(
            "{}: {} further parse problems",
            path.display(),
            log.problems.len() - MAX_ERRORS
        ));
    }
    if mft.is_none() {
        stats.without_mft += 1;
        report(&format!(
            "No unambiguous matching $MFT for {}; exporting without current paths (use -m with -f to select one explicitly)",
            path.display()
        ));
    }
    let paths = match mft.map(load_paths).transpose() {
        Ok(paths) => paths,
        Err(e) => {
            // Enrichment must not prevent recovery of the primary artifact.
            stats.mft_failures += 1;
            report(&format!(
                "Could not read $MFT {}: {e}; exporting $LogFile without current paths",
                mft.unwrap().display()
            ));
            None
        }
    };
    let truncated: HashSet<_> = log
        .problems
        .iter()
        .filter_map(|p| match p.kind {
            ntfslog::ProblemKind::RecordCutShort { lsn, .. } => Some(lsn),
            _ => None,
        })
        .collect();
    for record in &log.records {
        stats.records += 1;
        if let Some(u) = record.update() {
            stats.updates += 1;
            stats.names += u.redo_names.len() + u.undo_names.len();
            stats.journal_writes += usize::from(u.journal_write().is_some());
            stats.inferred += usize::from(u.target_inferred);
            *stats.operations.entry(u.redo.to_string()).or_default() += 1;
        } else if matches!(record.body, Body::Checkpoint(_)) {
            stats.checkpoints += 1;
        }
        if let Some(w) = writer.as_mut() {
            let value = record_value(
                record,
                path,
                mft.filter(|_| paths.is_some()),
                paths.as_ref(),
                &log,
                truncated.contains(&record.lsn),
                opts.raw_data,
            );
            let result = if w.is_csv() {
                w.csv(
                    &CSV_HEADER
                        .iter()
                        .map(|key| csv_cell(&value[*key]))
                        .collect::<Vec<_>>(),
                )
            } else {
                w.json(&value)
            };
            // Preserve IO failures, including ENOSPC, rather than treating them as damaged input.
            result.map_err(|e| io::Error::new(io::ErrorKind::WriteZero, e))?;
        }
    }
    Ok(())
}

/// Current MFT paths, keyed by the entire file reference, including its sequence. There is no
/// replay and no number-only fallback: that would misidentify a reused record as its predecessor.
fn load_paths(path: &Path) -> io::Result<usn::Paths> {
    let mut parser = mft::MftParser::from_path(path).map_err(io::Error::other)?;
    let mut builder = usn::PathBuilder::new();
    let mut seeded = 0;
    for number in 0..parser.get_entry_count() {
        let Ok(entry) = parser.get_entry(number) else {
            continue;
        };
        if !entry.header.is_base_record() {
            continue;
        }
        let sequence = if entry.header.flags.contains(EntryFlags::ALLOCATED) {
            entry.header.sequence
        } else if let Some(s) = entry.header.sequence.checked_sub(1) {
            s
        } else {
            continue;
        };
        let extensions = parser.get_extension_entries(&entry);
        let mut best = None;
        for record in std::iter::once(&entry).chain(&extensions) {
            for attr in record
                .iter_attributes_matching(Some(vec![MftAttributeType::FileName]))
                .filter_map(Result::ok)
            {
                let Some(name) = attr.data.as_file_name() else {
                    continue;
                };
                let rank = match name.namespace {
                    FileNamespace::Win32 | FileNamespace::Win32AndDos => 3,
                    FileNamespace::POSIX => 2,
                    FileNamespace::DOS => 1,
                };
                if best.as_ref().is_none_or(|(r, _, _)| rank > *r) {
                    best = Some((
                        rank,
                        name.name.to_utf8_string(),
                        usn::Reference::new(
                            (u128::from(name.parent.sequence) << 48)
                                | u128::from(name.parent.entry),
                        ),
                    ));
                }
            }
        }
        if let Some((_, name, parent)) = best {
            builder.seed(
                usn::Reference::new((u128::from(sequence) << 48) | u128::from(number)),
                &name,
                parent,
            );
            seeded += 1;
        }
    }
    if seeded == 0 {
        return Err(io::Error::other("no readable file names in the MFT"));
    }
    Ok(builder.finish())
}

fn path_of(paths: Option<&usn::Paths>, reference: Option<ntfslog::Reference>) -> Option<String> {
    Some(paths?.resolve(usn::Reference::new(u128::from(reference?.raw()))))
}

const CSV_HEADER: &[&str] = &[
    "Lsn",
    "TransactionId",
    "TransactionFirstLsn",
    "RedoOperation",
    "UndoOperation",
    "TargetEntryNumber",
    "TargetSequenceNumber",
    "TargetAttributeType",
    "TargetAttributeName",
    "TargetPath",
    "EntryNumber",
    "SequenceNumber",
    "FileName",
    "ParentEntryNumber",
    "ParentSequenceNumber",
    "ParentPath",
    "CreatedOn",
    "ContentModifiedOn",
    "RecordModifiedOn",
    "LastAccessedOn",
    "FileSize",
    "FileAttributes",
    "AttributeType",
    "PreviousLsn",
    "UndoNextLsn",
    "RecordType",
    "TargetAttribute",
    "TargetInferred",
    "TargetVcn",
    "Lcns",
    "RecordOffset",
    "AttributeOffset",
    "ClusterBlockOffset",
    "TargetBlockSize",
    "OffsetInTarget",
    "RedoLength",
    "UndoLength",
    "RedoBytesAvailable",
    "UndoBytesAvailable",
    "Truncated",
    "FileOffset",
    "SourceFile",
    "MetadataSide",
    "ClientIndex",
    "ClientSequence",
    "Flags",
    "SpansPages",
    "ClusterSize",
    "ClusterSizeSource",
    "LogVersion",
    "MftSource",
    "RedoFileNames",
    "UndoFileNames",
    "RedoTimes",
    "UndoTimes",
    "RedoDecoded",
    "UndoDecoded",
    "JournalWriteOffset",
    "JournalWriteLength",
    "Checkpoint",
    "OtherRecordType",
    "RedoData",
    "UndoData",
    "OtherData",
    "OpenAttributeEntryNumber",
    "OpenAttributeSequenceNumber",
    "OpenAttributePath",
];

fn csv_cell(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        _ => value.to_string(),
    }
}

fn record_value(
    r: &LogRecord,
    source: &Path,
    mft: Option<&Path>,
    paths: Option<&usn::Paths>,
    log: &ntfslog::LogFile,
    truncated: bool,
    raw: bool,
) -> Value {
    let mut v = json!({
        "Lsn": r.lsn, "TransactionId": r.transaction_id, "TransactionFirstLsn": r.transaction_first_lsn,
        "PreviousLsn": r.previous_lsn, "UndoNextLsn": r.undo_next_lsn, "FileOffset": r.offset,
        "SourceFile": source.to_string_lossy(), "MftSource": mft.map(|p| p.to_string_lossy()), "ClientIndex": r.client.index, "ClientSequence": r.client.sequence,
        "Flags": r.flags, "SpansPages": r.spans_pages(), "Truncated": truncated,
        "ClusterSize": log.cluster_size, "ClusterSizeSource": format!("{:?}", log.cluster_size_source),
        "LogVersion": log.restart().map(|p| p.version.to_string()),
    });
    // A consistent schema in JSON as well as CSV: absent is null, never an invented 0 or time.
    for key in CSV_HEADER {
        v.as_object_mut()
            .unwrap()
            .entry(*key)
            .or_insert(Value::Null);
    }
    match &r.body {
        Body::Update(u) => {
            v["RecordType"] = json!("ClientRecord");
            let targets_record = u.redo.targets_file_record() || u.undo.targets_file_record();
            for (key, value) in [
                ("RedoOperation", json!(u.redo.to_string())),
                ("UndoOperation", json!(u.undo.to_string())),
                (
                    "TargetEntryNumber",
                    json!(if targets_record {
                        u.mft_record
                    } else {
                        u.target.as_ref().map(|t| t.file.entry())
                    }),
                ),
                (
                    "TargetSequenceNumber",
                    json!(if targets_record {
                        None
                    } else {
                        u.target.as_ref().map(|t| t.file.sequence())
                    }),
                ),
                (
                    "TargetAttributeType",
                    json!(u.target.as_ref().map(|t| t.attribute_type)),
                ),
                (
                    "TargetAttributeName",
                    json!(u.target.as_ref().and_then(|t| t.name.as_deref())),
                ),
                (
                    "TargetPath",
                    // A record number alone cannot identify the historical generation.
                    json!(if targets_record {
                        None
                    } else {
                        path_of(paths, u.target.as_ref().map(|t| t.file))
                    }),
                ),
                (
                    "OpenAttributeEntryNumber",
                    json!(u.target.as_ref().map(|t| t.file.entry())),
                ),
                (
                    "OpenAttributeSequenceNumber",
                    json!(u.target.as_ref().map(|t| t.file.sequence())),
                ),
                (
                    "OpenAttributePath",
                    json!(path_of(paths, u.target.as_ref().map(|t| t.file))),
                ),
                ("TargetAttribute", json!(u.target_attribute)),
                ("TargetInferred", json!(u.target_inferred)),
                ("TargetVcn", json!(u.target_vcn)),
                ("Lcns", json!(u.lcns)),
                ("RecordOffset", json!(u.record_offset)),
                ("AttributeOffset", json!(u.attribute_offset)),
                ("ClusterBlockOffset", json!(u.cluster_block_offset)),
                ("TargetBlockSize", json!(u.target_block_size)),
                ("OffsetInTarget", json!(u.offset_in_target)),
                ("RedoLength", json!(u.redo_length)),
                ("UndoLength", json!(u.undo_length)),
                ("RedoBytesAvailable", json!(u.redo_data.len())),
                ("UndoBytesAvailable", json!(u.undo_data.len())),
                (
                    "RedoFileNames",
                    json!(
                        u.redo_names
                            .iter()
                            .map(|n| name_value(n, paths))
                            .collect::<Vec<_>>()
                    ),
                ),
                (
                    "UndoFileNames",
                    json!(
                        u.undo_names
                            .iter()
                            .map(|n| name_value(n, paths))
                            .collect::<Vec<_>>()
                    ),
                ),
                (
                    "RedoTimes",
                    u.redo_times.map(times_value).unwrap_or(Value::Null),
                ),
                (
                    "UndoTimes",
                    u.undo_times.map(times_value).unwrap_or(Value::Null),
                ),
                (
                    "RedoDecoded",
                    u.redo_decoded
                        .as_ref()
                        .map(|d| decoded_value(d, paths))
                        .unwrap_or(Value::Null),
                ),
                (
                    "UndoDecoded",
                    u.undo_decoded
                        .as_ref()
                        .map(|d| decoded_value(d, paths))
                        .unwrap_or(Value::Null),
                ),
                (
                    "JournalWriteOffset",
                    json!(u.journal_write().map(|w| w.offset)),
                ),
                (
                    "JournalWriteLength",
                    json!(u.journal_write().map(|w| w.data.len())),
                ),
                (
                    "RedoData",
                    if raw {
                        json!(hex(&u.redo_data))
                    } else {
                        Value::Null
                    },
                ),
                (
                    "UndoData",
                    if raw {
                        json!(hex(&u.undo_data))
                    } else {
                        Value::Null
                    },
                ),
            ] {
                v[key] = value;
            }
            let preferred = |names: &[FileName]| {
                names
                    .iter()
                    .find(|n| n.namespace != ntfslog::Namespace::Dos)
                    .cloned()
                    .or_else(|| names.first().cloned())
            };
            let name = preferred(&u.redo_names)
                .map(|n| ("Redo", n))
                .or_else(|| preferred(&u.undo_names).map(|n| ("Undo", n)));
            if let Some((side, n)) = name {
                v["MetadataSide"] = json!(side);
                v["EntryNumber"] = json!(n.file.map(|f| f.entry()).or(u.mft_record));
                v["SequenceNumber"] = json!(n.file.map(|f| f.sequence()));
                v["FileName"] = json!(n.name);
                v["ParentEntryNumber"] = json!(n.parent.entry());
                v["ParentSequenceNumber"] = json!(n.parent.sequence());
                v["ParentPath"] = json!(path_of(paths, Some(n.parent)));
                v["FileSize"] = json!(n.size);
                v["FileAttributes"] = json!(n.attributes);
                set_times(
                    &mut v,
                    [
                        Some(n.created),
                        Some(n.modified),
                        Some(n.changed),
                        Some(n.accessed),
                    ],
                );
            } else if let Some((side, times)) = u
                .redo_times
                .map(|t| ("Redo", t))
                .or(u.undo_times.map(|t| ("Undo", t)))
            {
                v["MetadataSide"] = json!(side);
                // The partial $STANDARD_INFORMATION value identifies its owner, even
                // though UpdateResidentValue does not carry a whole attribute record.
                v["EntryNumber"] = json!(u.mft_record);
                v["AttributeType"] = json!(0x10);
                set_times(
                    &mut v,
                    [times.created, times.modified, times.changed, times.accessed],
                );
            }
            let decoded = u
                .redo_decoded
                .as_ref()
                .map(|d| ("Redo", d))
                .or_else(|| u.undo_decoded.as_ref().map(|d| ("Undo", d)));
            match decoded {
                Some((side, Decoded::DirectoryEntryInfo(i))) => {
                    v["MetadataSide"] = json!(side);
                    set_times(
                        &mut v,
                        [
                            Some(i.created),
                            Some(i.modified),
                            Some(i.changed),
                            Some(i.accessed),
                        ],
                    );
                    v["FileSize"] = json!(i.size);
                    v["FileAttributes"] = json!(i.attributes);
                    v["AttributeType"] = json!(0x30);
                }
                Some((side, Decoded::Attribute(a))) => {
                    v["AttributeType"] = json!(a.attribute_type);
                    v["EntryNumber"] = json!(u.mft_record);
                    if let Some(info) = &a.standard_information {
                        v["FileAttributes"] = json!(info.attributes);
                    }
                    if a.attribute_type == 0x80 {
                        v["MetadataSide"] = json!(side);
                        v["FileSize"] = json!(a.size);
                    }
                }
                Some((side, Decoded::FileRecord(f))) => {
                    if v["MetadataSide"].is_null() {
                        v["MetadataSide"] = json!(side);
                    }
                    if v["EntryNumber"].is_null() {
                        v["EntryNumber"] = json!(f.number.or(u.mft_record));
                    }
                    if v["SequenceNumber"].is_null() {
                        v["SequenceNumber"] = json!(f.sequence);
                    }
                    if let Some(size) = f.data_size() {
                        v["FileSize"] = json!(size);
                    }
                    if v["FileAttributes"].is_null()
                        && let Some(info) = f.standard_information()
                    {
                        v["FileAttributes"] = json!(info.attributes);
                    }
                }
                _ => {}
            }
        }
        Body::Checkpoint(c) => {
            v["RecordType"] = json!("ClientRestart");
            v["Checkpoint"] = json!({"MajorVersion": c.major_version, "MinorVersion": c.minor_version, "StartLsn": c.start_lsn,
                "OpenAttributeTable": table_value(c.open_attribute_table), "AttributeNames": table_value(c.attribute_names),
                "DirtyPageTable": table_value(c.dirty_page_table), "TransactionTable": table_value(c.transaction_table),
                "BytesPerCluster": c.bytes_per_cluster});
        }
        Body::Other { record_type, data } => {
            v["RecordType"] = json!("Other");
            v["OtherRecordType"] = json!(record_type);
            if raw {
                v["OtherData"] = json!(hex(data));
            }
        }
    }
    v
}

fn table_value(t: ntfslog::TableDump) -> Value {
    json!({"Lsn": t.lsn, "Length": t.length})
}
fn set_times(v: &mut Value, times: [Option<Filetime>; 4]) {
    for (key, time) in [
        "CreatedOn",
        "ContentModifiedOn",
        "RecordModifiedOn",
        "LastAccessedOn",
    ]
    .into_iter()
    .zip(times)
    {
        v[key] = json!(time.and_then(Filetime::to_iso8601));
    }
}
fn time_value(time: Filetime) -> Value {
    json!({"Utc": time.to_iso8601(), "Filetime": time.raw()})
}
fn times_value(t: StandardTimes) -> Value {
    json!({"Created": t.created.map(time_value), "Modified": t.modified.map(time_value), "Changed": t.changed.map(time_value), "Accessed": t.accessed.map(time_value)})
}
fn ref_value(r: ntfslog::Reference) -> Value {
    json!({"Raw": r.raw(), "EntryNumber": r.entry(), "SequenceNumber": r.sequence()})
}
fn name_value(n: &FileName, paths: Option<&usn::Paths>) -> Value {
    json!({"File": n.file.map(ref_value), "Parent": ref_value(n.parent), "Name": n.name, "NameEscaped": n.name_escaped,
        "Namespace": format!("{:?}", n.namespace), "ParentPath": path_of(paths, Some(n.parent)),
        "Created": time_value(n.created), "Modified": time_value(n.modified), "Changed": time_value(n.changed), "Accessed": time_value(n.accessed),
        "AllocatedSize": n.allocated_size, "Size": n.size, "Attributes": n.attributes, "ReparseValue": n.reparse_value})
}
fn info_value(s: &ntfslog::StandardInformation) -> Value {
    json!({"Created": time_value(s.created), "Modified": time_value(s.modified), "Changed": time_value(s.changed), "Accessed": time_value(s.accessed),
        "Attributes": s.attributes, "OwnerId": s.owner_id, "SecurityId": s.security_id, "QuotaCharged": s.quota_charged, "Usn": s.usn})
}
fn attribute_value(a: &ntfslog::Attribute, paths: Option<&usn::Paths>) -> Value {
    json!({"Type": a.attribute_type, "Name": a.name, "Resident": a.resident, "Flags": a.flags, "Id": a.id, "Size": a.size,
        "AllocatedSize": a.allocated_size, "StandardInformation": a.standard_information.as_ref().map(info_value), "FileName": a.file_name.as_ref().map(|n| name_value(n, paths))})
}
fn decoded_value(d: &Decoded, paths: Option<&usn::Paths>) -> Value {
    match d {
        Decoded::FileRecord(f) => {
            json!({"Kind": "FileRecord", "Number": f.number, "Sequence": f.sequence, "HardLinks": f.hard_links,
            "Flags": f.flags, "Base": f.base.map(ref_value), "HeaderOnly": f.header_only,
            "Attributes": f.attributes.iter().map(|a| attribute_value(a, paths)).collect::<Vec<_>>()})
        }
        Decoded::Attribute(a) => {
            json!({"Kind": "Attribute", "Attribute": attribute_value(a, paths)})
        }
        Decoded::IndexEntry(e) => {
            json!({"Kind": "IndexEntry", "Flags": e.flags, "KeyLength": e.key_length, "FileName": e.file_name.as_ref().map(|n| name_value(n, paths))})
        }
        Decoded::DirectoryEntryInfo(i) => {
            json!({"Kind": "DirectoryEntryInfo", "Created": time_value(i.created), "Modified": time_value(i.modified),
            "Changed": time_value(i.changed), "Accessed": time_value(i.accessed), "AllocatedSize": i.allocated_size, "Size": i.size, "Attributes": i.attributes, "ReparseValue": i.reparse_value})
        }
    }
}
fn hex(data: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(data.len().saturating_mul(2));
    for byte in data {
        write!(s, "{byte:02x}").unwrap();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn non_utf8_provenance_serializes_without_touching_the_filesystem() {
        use std::os::unix::ffi::OsStringExt;
        let path = PathBuf::from(std::ffi::OsString::from_vec(b"evidence-\xff".to_vec()));
        let record = LogRecord {
            lsn: 1,
            previous_lsn: 0,
            undo_next_lsn: 0,
            offset: 0,
            client: ntfslog::ClientId {
                sequence: 0,
                index: 0,
            },
            transaction_id: 0,
            transaction_first_lsn: 1,
            flags: 0,
            body: Body::Other {
                record_type: 0,
                data: vec![],
            },
        };
        let log = ntfslog::parse(&[0xff; 4096]);
        let value = record_value(&record, &path, Some(&path), None, &log, false, false);
        assert_eq!(value["SourceFile"], "evidence-\u{fffd}");
        assert_eq!(value["MftSource"], "evidence-\u{fffd}");
    }

    #[test]
    fn flat_metadata_uses_undo_when_redo_has_no_decoded_body() {
        use std::io::Read;
        let mut data = Vec::new();
        flate2::read::GzDecoder::new(
            &include_bytes!("../test_files/logfile/go-ntfs/LogFile.gz")[..],
        )
        .read_to_end(&mut data)
        .unwrap();
        let mut log = ntfslog::parse(&data);
        for (lsn, size, attribute_type) in [(1058043, 12, 0x30), (1052517, 8, 0x80)] {
            let record = log.records.iter_mut().find(|r| r.lsn == lsn).unwrap();
            let Body::Update(update) = &mut record.body else {
                panic!("an update");
            };
            update.undo_decoded = update.redo_decoded.take();
            update.redo = ntfslog::Operation::Noop;
            let record = record.clone();
            let value = record_value(
                &record,
                Path::new("$LogFile"),
                None,
                None,
                &log,
                false,
                false,
            );
            assert_eq!(value["MetadataSide"], "Undo");
            assert_eq!(value["FileSize"], size);
            assert_eq!(value["AttributeType"], attribute_type);
        }
        // NTFS can initialize a FILE record with SI and DATA before adding its filename.
        let record = log.records.iter_mut().find(|r| r.update().is_some_and(|u| {
            u.redo == ntfslog::Operation::InitializeFileRecordSegment
                && u.undo_names.is_empty()
                && matches!(&u.redo_decoded, Some(Decoded::FileRecord(f)) if f.standard_information().is_some())
        })).unwrap();
        let Body::Update(update) = &mut record.body else {
            unreachable!()
        };
        update.redo_names.clear();
        let Some(Decoded::FileRecord(file)) = &mut update.redo_decoded else {
            unreachable!()
        };
        file.attributes.retain(|a| a.attribute_type != 0x30);
        let attributes = file.standard_information().unwrap().attributes;
        let record = record.clone();
        let value = record_value(
            &record,
            Path::new("$LogFile"),
            None,
            None,
            &log,
            false,
            false,
        );
        assert!(value["FileName"].is_null());
        assert_eq!(value["FileAttributes"], attributes);
    }

    #[test]
    fn exported_names_are_specific() {
        for name in [
            "$LogFile",
            "LogFile",
            "C_$LogFile",
            "$logfile.bin",
            "volume_$LogFile.bin",
        ] {
            assert!(is_logfile_name(name));
        }
        for name in [
            "LogFile.csv",
            "LogFile.gz",
            "SYSTEM.LOG1",
            "logfile.txt",
            "$LogFile.bak",
            "application-logfile.bin",
        ] {
            assert!(!is_logfile_name(name));
        }
    }

    #[test]
    fn csv_cells_preserve_types_and_nested_data() {
        assert_eq!(csv_cell(&Value::Null), "");
        assert_eq!(csv_cell(&json!(false)), "false");
        assert_eq!(csv_cell(&json!(u64::MAX)), u64::MAX.to_string());
        assert_eq!(csv_cell(&json!(["a", "b"])), "[\"a\",\"b\"]");
    }
}
