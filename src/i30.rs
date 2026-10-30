//! `parse-i30`: directory indexes (`$I30`), read by ys-mft-parser's `i30` module, with Hayabusa's
//! input options, progress bar and results summary. One record per index entry: the file's name
//! in the directory, its MFT reference, its parent, the four times and two sizes its `$FILE_NAME`
//! keeps, and whether it was found in the slack of an INDX record, where entries of files deleted
//! or renamed out of the directory are left (and copies of entries moved when the index was
//! rebalanced, marked as such). A file holding INDX records (an exported `$I30`, a disk image,
//! unallocated space) is read in chunks.

use crate::cli::RegistryOutputType;
use crate::cli::{I30Options, I30OutputType};
use crate::color::JunteiColor;
use crate::lnk::filetime;
use crate::progress;
use crate::registry::{RecordWriter, sink};
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use mft_i30::attribute::FileAttributeFlags;
use mft_i30::attribute::x30::FileNamespace;
use mft_i30::i30::{IndexEntry, IndexRecord, index_records};
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Names and problems listed in the summary.
const MAX_PRINTED: usize = 20;
/// How much of a file is read at a time; records starting in a chunk are read from it.
const CHUNK: u64 = 64 << 20;
/// What is read past a chunk, for a record that starts in it and ends in the next (INDX records
/// are at most 64 KiB).
const LOOKAHEAD: u64 = 64 << 10;

/// Run the command; returns the exit status.
pub fn run(opts: &I30Options) -> i32 {
    let no_color = opts.common.no_color;
    let data_on_stdout = opts.data_on_stdout();
    let report_error = |bar: Option<&ProgressBar>, msg: &str| {
        if data_on_stdout {
            eprintln!("{msg}");
        } else {
            progress::suspended(bar, || error_msg(no_color, msg));
        }
    };

    // --- inputs ---------------------------------------------------------------------------
    let files = match (&opts.filepath, &opts.directory) {
        (Some(f), _) if f.is_file() => vec![f.clone()],
        (Some(f), _) => {
            report_error(None, &format!("The file does not exist: {}", f.display()));
            return 1;
        }
        (None, Some(d)) if !d.is_dir() => {
            report_error(
                None,
                &format!("The directory does not exist: {}", d.display()),
            );
            return 1;
        }
        (None, Some(d)) => {
            let found = find_i30_files(d);
            if found.is_empty() {
                report_error(
                    None,
                    &format!("No $I30 files were found in {}", d.display()),
                );
                return 1;
            }
            found
        }
        (None, None) => unreachable!("clap requires -f or -d"),
    };
    let sink = match sink(opts.output.as_ref(), opts.clobber, data_on_stdout) {
        Ok(sink) => sink,
        Err(msg) => {
            report_error(None, &msg);
            return 1;
        }
    };
    let output_type = match opts.output_type() {
        I30OutputType::Csv => RegistryOutputType::Csv,
        I30OutputType::Json => RegistryOutputType::Json,
        I30OutputType::Jsonl => RegistryOutputType::Jsonl,
    };
    let mut writer = match sink.map(|w| RecordWriter::new(w, output_type, &CSV_HEADER)) {
        Some(Ok(w)) => Some(w),
        Some(Err(e)) => {
            report_error(None, &format!("Could not write the entries: {e}"));
            return 1;
        }
        None => None,
    };
    let total: u64 = files
        .iter()
        .map(|p| p.metadata().map_or(0, |m| m.len()))
        .sum();
    if !data_on_stdout {
        p_label(no_color, "$I30 files: ", &fmt_num(files.len()));
        p_label(no_color, "Total file size: ", &util::size_display(total));
        p(None, "", true);
    }
    let bar = (!data_on_stdout).then(|| {
        p(
            JunteiColor::Orange.rdg(no_color),
            "Parsing in progress. Please wait.",
            true,
        );
        p(None, "", true);
        progress::progress_bar(files.len(), no_color)
    });

    // --- parse and write -------------------------------------------------------------------
    let mut stats = Stats::default();
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        match read_file(path, opts, &mut stats, writer.as_mut()) {
            Ok(0) => {
                stats.failed += 1;
                if stats.failed <= MAX_PRINTED_ERRORS {
                    report_error(
                        bar.as_ref(),
                        &format!("Could not parse {}: no INDX records", path.display()),
                    );
                }
            }
            Ok(_) => stats.files += 1,
            Err(Failure::Read(e)) => {
                stats.failed += 1;
                if stats.failed <= MAX_PRINTED_ERRORS {
                    report_error(
                        bar.as_ref(),
                        &format!("Could not read {}: {e}", path.display()),
                    );
                }
            }
            Err(Failure::Write(e)) => {
                report_error(
                    bar.as_ref(),
                    &format!("Could not write the entries of {}: {e}", path.display()),
                );
                return 1;
            }
        }
        if let Some(b) = &bar {
            b.inc(1);
        }
    }
    if stats.failed > MAX_PRINTED_ERRORS {
        report_error(
            bar.as_ref(),
            &format!(
                "{} more files could not be parsed",
                fmt_num(stats.failed - MAX_PRINTED_ERRORS)
            ),
        );
    }
    if let Some(w) = writer
        && let Err(e) = w.finish()
    {
        report_error(None, &format!("Could not write the entries: {e}"));
        return 1;
    }
    if let Some(b) = &bar {
        b.finish_with_message(progress::finished_message(no_color));
    }

    // --- summary ---------------------------------------------------------------------------
    if !data_on_stdout {
        stats.print(no_color);
        if let Some(out) = &opts.output {
            p_label(
                no_color,
                "Results saved: ",
                &format!(
                    "{} ({} entries, {})",
                    out.display(),
                    fmt_num(stats.written),
                    util::file_size_display(out)
                ),
            );
        } else {
            p(
                None,
                "To save every entry, add -o <FILE> (-t csv, json or jsonl).",
                true,
            );
        }
    }
    i32::from(stats.files == 0)
}

/// Whether a file is a `$I30` file: named so (`$I30`, `Users_$I30`, …), or starting with an
/// INDX record.
fn is_i30_file(path: &Path) -> bool {
    let named = path
        .file_name()
        .is_some_and(|n| n.to_string_lossy().to_uppercase().contains("$I30"));
    named || {
        let mut head = [0u8; 4];
        File::open(path)
            .and_then(|mut f| f.read_exact(&mut head))
            .is_ok()
            && &head == mft_i30::i30::INDX_SIGNATURE
    }
}

/// The `$I30` files under `dir`, recursively, in path order.
fn find_i30_files(dir: &Path) -> Vec<PathBuf> {
    util::find_files(dir, |_| true)
        .into_iter()
        .filter(|p| is_i30_file(p))
        .collect()
}

/// Why a file was not read through.
enum Failure {
    Read(io::Error),
    Write(io::Error),
}

/// Read the INDX records of the file at `path`, writing their entries; how many records it has.
fn read_file(
    path: &Path,
    opts: &I30Options,
    stats: &mut Stats,
    mut writer: Option<&mut RecordWriter>,
) -> Result<usize, Failure> {
    let mut file = File::open(path).map_err(Failure::Read)?;
    let len = file.metadata().map_err(Failure::Read)?.len();
    let source = path.to_string_lossy().to_string();
    let mut found = 0;
    each_chunk(&mut file, len, CHUNK, |base, records| {
        for record in &records {
            found += 1;
            stats.record(&source, base, record);
            for entry in record.entries.iter().chain(&record.slack) {
                // the summary counts every entry; --slack-only and --no-copies choose those written
                stats.entry(&source, entry);
                if (opts.slack_only && !entry.from_slack)
                    || (opts.no_copies && entry.copy_of_entry_in_use)
                {
                    continue;
                }
                stats.written += 1;
                let r = Record::new(&source, base, record, entry);
                match writer.as_deref_mut() {
                    Some(w) if w.is_csv() => w.csv(&r.csv()).map_err(Failure::Write)?,
                    Some(w) => w.json(&r).map_err(Failure::Write)?,
                    None => {}
                }
            }
        }
        Ok(())
    })?;
    Ok(found)
}

/// Read `file` (of `len` bytes) `chunk` bytes at a time, handing `each` a chunk's offset and the
/// INDX records that start in it (read with the bytes after it, for those that end in the next).
fn each_chunk(
    file: &mut (impl Read + Seek),
    len: u64,
    chunk: u64,
    mut each: impl FnMut(u64, Vec<IndexRecord>) -> Result<(), Failure>,
) -> Result<(), Failure> {
    let mut base = 0;
    while base < len {
        let want = (chunk + LOOKAHEAD).min(len - base);
        let mut data = vec![0u8; usize::try_from(want).unwrap_or(0)];
        file.seek(SeekFrom::Start(base)).map_err(Failure::Read)?;
        file.read_exact(&mut data).map_err(Failure::Read)?;
        let records = index_records(&data)
            .into_iter()
            .filter(|r| r.offset < chunk)
            .collect();
        each(base, records)?;
        base += chunk;
    }
    Ok(())
}

// --- records ----------------------------------------------------------------------------------

/// A namespace's name: `posix`, `win32`, `dos`, `win32_and_dos`.
fn namespace(n: FileNamespace) -> &'static str {
    match n {
        FileNamespace::POSIX => "posix",
        FileNamespace::Win32 => "win32",
        FileNamespace::DOS => "dos",
        FileNamespace::Win32AndDos => "win32_and_dos",
    }
}

/// File attribute flags' names (`archive`, `hidden`, `is_directory`, …), joined with `|`.
fn attributes(flags: &FileAttributeFlags) -> String {
    flags
        .iter_names()
        .map(|(name, _)| name.trim_start_matches("FILE_ATTRIBUTE_").to_lowercase())
        .collect::<Vec<_>>()
        .join("|")
}

/// A FILETIME in UTC (to the 100 ns), or `None`.
fn time(ft: u64) -> Option<String> {
    Some(filetime(ft)).filter(|t| !t.is_empty())
}

/// An index entry's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    source_path: &'a str,
    record_offset: u64,
    record_vcn: u64,
    record_lsn: u64,
    entry_offset: u64,
    from_slack: bool,
    copy_of_entry_in_use: bool,
    file_name: &'a str,
    namespace: &'static str,
    file_entry: Option<u64>,
    file_sequence: Option<u16>,
    parent_entry: Option<u64>,
    parent_sequence: Option<u16>,
    created: Option<String>,
    modified: Option<String>,
    mft_modified: Option<String>,
    accessed: Option<String>,
    allocated_size: u64,
    real_size: u64,
    file_attributes: String,
    reparse_value: u32,
    problems: &'a [String],
}

const CSV_HEADER: [&str; 22] = [
    "SourcePath",
    "RecordOffset",
    "RecordVcn",
    "RecordLsn",
    "EntryOffset",
    "FromSlack",
    "CopyOfEntryInUse",
    "FileName",
    "Namespace",
    "FileEntry",
    "FileSequence",
    "ParentEntry",
    "ParentSequence",
    "CreatedUTC",
    "ModifiedUTC",
    "MftModifiedUTC",
    "AccessedUTC",
    "AllocatedSize",
    "RealSize",
    "FileAttributes",
    "ReparseValue",
    "Problems",
];

impl<'a> Record<'a> {
    fn new(source: &'a str, base: u64, record: &'a IndexRecord, e: &'a IndexEntry) -> Self {
        Record {
            source_path: source,
            record_offset: base + record.offset,
            record_vcn: record.vcn,
            record_lsn: record.lsn,
            entry_offset: base + e.offset,
            from_slack: e.from_slack,
            copy_of_entry_in_use: e.copy_of_entry_in_use,
            file_name: &e.name,
            namespace: namespace(e.namespace),
            file_entry: e.file_reference.map(|f| f.entry),
            file_sequence: e.file_reference.map(|f| f.sequence),
            parent_entry: e.parent.map(|p| p.entry),
            parent_sequence: e.parent.map(|p| p.sequence),
            created: time(e.created),
            modified: time(e.modified),
            mft_modified: time(e.mft_modified),
            accessed: time(e.accessed),
            allocated_size: e.allocated_size,
            real_size: e.real_size,
            file_attributes: attributes(&e.file_flags),
            reparse_value: e.reparse_value,
            problems: &record.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: &Option<String>| v.clone().unwrap_or_default();
        let num = |v: Option<u64>| v.map(|n| n.to_string()).unwrap_or_default();
        let flag = |b: bool| if b { "True" } else { "False" }.to_string();
        vec![
            self.source_path.to_string(),
            self.record_offset.to_string(),
            self.record_vcn.to_string(),
            self.record_lsn.to_string(),
            self.entry_offset.to_string(),
            flag(self.from_slack),
            flag(self.copy_of_entry_in_use),
            self.file_name.to_string(),
            self.namespace.to_string(),
            num(self.file_entry),
            num(self.file_sequence.map(u64::from)),
            num(self.parent_entry),
            num(self.parent_sequence.map(u64::from)),
            text(&self.created),
            text(&self.modified),
            text(&self.mft_modified),
            text(&self.accessed),
            self.allocated_size.to_string(),
            self.real_size.to_string(),
            self.file_attributes.clone(),
            self.reparse_value.to_string(),
            self.problems.join("; "),
        ]
    }
}

// --- summary ----------------------------------------------------------------------------------

/// Counts for the results summary.
#[derive(Default)]
struct Stats {
    files: usize,
    failed: usize,
    records: usize,
    written_in_part: usize,
    in_use: usize,
    slack: usize,
    copies: usize,
    headerless: usize,
    without_parent: usize,
    written: usize,
    /// The names in slack that are not copies of entries in use: deleted or renamed files.
    gone: BTreeSet<(String, String)>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    fn record(&mut self, source: &str, base: u64, record: &IndexRecord) {
        self.records += 1;
        self.written_in_part += usize::from(!record.valid_fixup);
        for problem in &record.problems {
            self.problem_count += 1;
            if self.problems.len() < MAX_PRINTED {
                self.problems.push(format!(
                    "{source}: the record at 0x{:x}: {problem}",
                    base + record.offset
                ));
            }
        }
    }

    fn entry(&mut self, source: &str, e: &IndexEntry) {
        if !e.from_slack {
            self.in_use += 1;
            return;
        }
        self.slack += 1;
        self.copies += usize::from(e.copy_of_entry_in_use);
        self.headerless += usize::from(e.file_reference.is_none());
        self.without_parent += usize::from(e.parent.is_none());
        if !e.copy_of_entry_in_use {
            self.gone.insert((source.to_string(), e.name.clone()));
        }
    }

    fn print(&self, no_color: bool) {
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        p_label(no_color, "$I30 files parsed: ", &fmt_num(self.files));
        if self.failed > 0 {
            p_label(no_color, "Could not parse: ", &fmt_num(self.failed));
        }
        let mut records = fmt_num(self.records);
        if self.written_in_part > 0 {
            records.push_str(&format!(
                " (written in part: {})",
                fmt_num(self.written_in_part)
            ));
        }
        p_label(no_color, "INDX records: ", &records);
        p_label(no_color, "Entries in use: ", &fmt_num(self.in_use));
        p_label(
            no_color,
            "Entries in slack: ",
            &format!(
                "{} (copies of entries in use: {}, without their header: {}, without their parent: {})",
                fmt_num(self.slack),
                fmt_num(self.copies),
                fmt_num(self.headerless),
                fmt_num(self.without_parent)
            ),
        );
        p_label(
            no_color,
            "Names in slack no longer in use (deleted or renamed files): ",
            &fmt_num(self.gone.len()),
        );
        // the file each name is in, when there are several
        let several = self.files + self.failed > 1;
        for (source, name) in self.gone.iter().take(MAX_PRINTED) {
            let line = if several {
                format!("  {source}: {name}")
            } else {
                format!("  {name}")
            };
            p(None, &line, true);
        }
        if self.gone.len() > MAX_PRINTED {
            p(
                None,
                &format!("  … {} more", fmt_num(self.gone.len() - MAX_PRINTED)),
                true,
            );
        }
        if self.problem_count > 0 {
            p_label(
                no_color,
                "Problems in records: ",
                &fmt_num(self.problem_count),
            );
            for problem in &self.problems {
                p(None, &format!("  {problem}"), true);
            }
        }
        p(None, "", true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_of_namespaces_and_attributes() {
        assert_eq!(namespace(FileNamespace::Win32AndDos), "win32_and_dos");
        let flags = FileAttributeFlags::from_bits_truncate(0x1000_0022);
        assert_eq!(attributes(&flags), "hidden|archive|is_directory");
        assert_eq!(time(0), None);
    }

    #[test]
    fn records_across_chunks() {
        // the sample's 7 records, 512 bytes in: those at 0x1200, 0x3200 and 0x5200 cross the ends
        // of 8 KiB chunks
        let sample = std::fs::read("test_files/i30/Logs_I30").unwrap();
        let mut data = vec![0u8; 512];
        data.extend_from_slice(&sample);
        let whole: Vec<(u64, usize, usize)> = index_records(&data)
            .iter()
            .map(|r| (r.offset, r.entries.len(), r.slack.len()))
            .collect();
        assert_eq!(whole.len(), 7);
        let mut chunked = Vec::new();
        let mut cursor = io::Cursor::new(&data);
        let read = each_chunk(&mut cursor, data.len() as u64, 8192, |base, records| {
            chunked.extend(
                records
                    .iter()
                    .map(|r| (base + r.offset, r.entries.len(), r.slack.len())),
            );
            Ok(())
        });
        assert!(read.is_ok());
        assert_eq!(chunked, whole);
    }

    #[test]
    fn i30_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("$I30"), b"anything").unwrap();
        std::fs::write(dir.path().join("Users_$i30"), b"anything").unwrap();
        std::fs::write(dir.path().join("exported.bin"), b"INDX....").unwrap();
        std::fs::write(dir.path().join("notes.txt"), b"not an index").unwrap();
        let mut found: Vec<String> = find_i30_files(dir.path())
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        found.sort();
        assert_eq!(found, ["$I30", "Users_$i30", "exported.bin"]);
    }
}
