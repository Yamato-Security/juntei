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

//! `parse-file recycle-bin`: the Recycle Bin's index files, one row per deleted item with when
//! it was deleted, where it was, its size, and whose bin it is in.
//!
//! - `$Recycle.Bin\<SID>\$I…` (Windows Vista and later): format version 1 (Vista to 8.1) holds a
//!   fixed 260-character path, version 2 (Windows 10 and later) a path of any length. The item
//!   itself is the `$R…` file or folder of the same name.
//! - `RECYCLER\<SID>\INFO2` (Windows NT 4.0 to XP and Server 2003): 800-byte records with an ANSI
//!   and a Unicode path. `RECYCLED\INFO2` (Windows 95 to ME; `INFO` on Windows 95): 280-byte
//!   records with only the ANSI path. The item itself is `D<drive><index><extension>`. When an
//!   item leaves the bin (restored, or deleted from it), Windows clears the first byte of its ANSI
//!   path and keeps the record.
//!
//! The user is the bin folder's SID, named from `ProfileList` in the SOFTWARE hive of the volume's
//! Windows installation (or of the only one found, for bins on other volumes).

use crate::cli::{RecycleBinOptions, RecycleBinOutputType};
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, check_output, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Larger files are not index files: a `$I` file is at most 28 + 2 × 32,767 bytes, and INFO2
/// grows by 800 bytes per item in the bin.
const MAX_INDEX_SIZE: u64 = 64 * 1024 * 1024;
/// `$I`: format version, size and deletion time.
const I_HEADER: usize = 24;
/// `$I` version 1: the path's 260 UTF-16 characters after the header.
const V1_PATH_BYTES: usize = 520;
const V1_SIZE: usize = I_HEADER + V1_PATH_BYTES;
/// `$I` version 2: the path's length in characters (with its NUL), then the path.
const V2_PATH_OFFSET: usize = 28;
/// The longest Windows path, with its NUL.
const MAX_PATH_CHARS: usize = 32_768;
/// INFO2: version, two counts, record size and a size total; then the records.
const INFO2_HEADER: usize = 20;
/// INFO2 record: 260-byte ANSI path, index, drive, deletion time and size.
const LEGACY_RECORD: usize = 280;
/// INFO2 record of Windows NT: the legacy record and a 260-character Unicode path.
const UNICODE_RECORD: usize = 800;

/// Recycle Bin folders, by the Windows versions that use them.
const BIN_FOLDERS: [&str; 3] = ["$Recycle.Bin", "RECYCLER", "RECYCLED"];

/// The index files `auto-parse` and disk-image staging select by name: `$I` followed by six
/// letters or digits and any extension (`$IKEGS1G`, `$IQ7LAXT.png`), and `INFO2`. Windows 95's
/// `INFO` is found only in a `RECYCLED` folder ([`find_inputs`]).
pub fn is_index_name(name: &str) -> bool {
    is_i_name(name) || name.eq_ignore_ascii_case("INFO2")
}

fn is_i_name(name: &str) -> bool {
    let b = name.as_bytes();
    b.len() >= 8
        && b[0] == b'$'
        && b[1].eq_ignore_ascii_case(&b'I')
        && b[2..8].iter().all(u8::is_ascii_alphanumeric)
        && (b.len() == 8 || b[8] == b'.')
}

fn is_candidate_name(name: &str) -> bool {
    is_index_name(name) || name.eq_ignore_ascii_case("INFO")
}

/// The index files under `dir`, recursively, in path order, and the SOFTWARE hives of the
/// Windows installations there (`Windows\System32\config\SOFTWARE`), for user names.
fn find_inputs(dir: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let (software, files): (Vec<PathBuf>, Vec<PathBuf>) = util::find_files(dir, |n| {
        is_candidate_name(n) || n.eq_ignore_ascii_case("SOFTWARE")
    })
    .into_iter()
    .partition(|path| file_name(path).eq_ignore_ascii_case("SOFTWARE"));
    let files = files
        .into_iter()
        .filter(|path| {
            let name = file_name(path);
            !name.eq_ignore_ascii_case("INFO")
                || path
                    .parent()
                    .and_then(Path::file_name)
                    .is_some_and(|d| d.to_string_lossy().eq_ignore_ascii_case("RECYCLED"))
        })
        .collect();
    let software = software
        .into_iter()
        .filter(|path| {
            let names: Vec<String> = path
                .components()
                .rev()
                .skip(1)
                .take(3)
                .map(|c| c.as_os_str().to_string_lossy().to_ascii_lowercase())
                .collect();
            names == ["config", "system32", "windows"]
        })
        .collect();
    (files, software)
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Run the command; returns the exit status.
pub fn run(opts: &RecycleBinOptions) -> i32 {
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
    let mut installations = Vec::new();
    let files = match (&opts.filepath, &opts.directory) {
        (Some(f), _) => {
            if !f.is_file() {
                report_error(None, &format!("The file does not exist: {}", f.display()));
                return 1;
            }
            vec![f.clone()]
        }
        (None, Some(d)) => {
            if !d.is_dir() {
                report_error(
                    None,
                    &format!("The directory does not exist: {}", d.display()),
                );
                return 1;
            }
            let (found, software) = find_inputs(d);
            installations = software;
            if found.is_empty() {
                report_error(
                    None,
                    &format!(
                        "No Recycle Bin index files ($I files, INFO2) were found in {}",
                        d.display()
                    ),
                );
                return 1;
            }
            found
        }
        (None, None) => unreachable!("clap requires -f or -d"),
    };
    if !shell::codepage::is_supported(opts.code_page) {
        report_error(
            None,
            &format!(
                "Code page {} is not supported (ex: 1252 Western European, 932 Japanese, 936 Simplified Chinese).",
                opts.code_page
            ),
        );
        return 1;
    }
    let users = Users::find(&files, &installations);
    if let Some(out) = &opts.output {
        if let Err(msg) = check_output(out, false, opts.clobber) {
            report_error(None, &msg);
            return 1;
        }
        // The index files, and the SOFTWARE hives (with their logs) read for user names.
        let inputs = || {
            let mut inputs = crate::registry::files_read(&users.hives(), &[]);
            inputs.extend(files.iter().cloned());
            inputs
        };
        if let Err(msg) = crate::registry::check_not_read(out, inputs) {
            report_error(None, &msg);
            return 1;
        }
    }
    if !data_on_stdout {
        let total: u64 = files
            .iter()
            .map(|p| p.metadata().map_or(0, |m| m.len()))
            .sum();
        p_label(no_color, "Recycle Bin index files: ", &fmt_num(files.len()));
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

    // --- parse -----------------------------------------------------------------------------
    let mut users = users;
    let mut parsed: Vec<Index> = Vec::new();
    let mut failed = 0usize;
    let mut listings = Listings::default();
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        match parse_file(path, opts.code_page) {
            Ok(mut index) => {
                let (sid, user) = users.of(path);
                for item in &mut index.items {
                    item.sid.clone_from(&sid);
                    item.user.clone_from(&user);
                    if let Some(name) = item.recycled_name.as_deref() {
                        let found = listings.find(path, name);
                        item.recycled = Some(sibling(path, found.as_deref().unwrap_or(name)));
                        item.recycled_found = Some(found.is_some());
                    }
                }
                for problem in &index.problems {
                    report_error(bar.as_ref(), &format!("{}: {problem}", path.display()));
                }
                parsed.push(index);
            }
            Err(e) => {
                failed += 1;
                if failed <= MAX_PRINTED_ERRORS {
                    report_error(
                        bar.as_ref(),
                        &format!("Could not parse {}: {e}", path.display()),
                    );
                }
            }
        }
        if let Some(b) = &bar {
            b.inc(1);
        }
    }
    if failed > MAX_PRINTED_ERRORS {
        report_error(
            bar.as_ref(),
            &format!(
                "{} more files could not be parsed",
                fmt_num(failed - MAX_PRINTED_ERRORS)
            ),
        );
    }
    if let Some(b) = &bar {
        b.finish_with_message(progress::finished_message(no_color));
    }

    // --- write -----------------------------------------------------------------------------
    let sink: Option<Box<dyn Write>> = match (&opts.output, data_on_stdout) {
        (Some(path), _) => {
            if let Some(parent) = path.parent().filter(|d| !d.as_os_str().is_empty())
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                report_error(None, &format!("Could not create {}: {e}", parent.display()));
                return 1;
            }
            match File::create(path) {
                Ok(f) => Some(Box::new(BufWriter::new(f))),
                Err(e) => {
                    report_error(None, &format!("Could not create {}: {e}", path.display()));
                    return 1;
                }
            }
        }
        (None, true) => Some(Box::new(BufWriter::new(io::stdout()))),
        (None, false) => None,
    };
    if let Some(sink) = sink
        && let Err(e) = write_records(sink, opts.output_type(), &parsed)
    {
        report_error(None, &format!("Could not write the records: {e}"));
        return 1;
    }

    // --- summary ---------------------------------------------------------------------------
    if !data_on_stdout {
        print_summary(&parsed, failed, no_color);
        if let Some(out) = &opts.output {
            p_label(
                no_color,
                "Results saved: ",
                &format!("{} ({})", out.display(), util::file_size_display(out)),
            );
        } else {
            p(
                None,
                "To save every record, add -o <FILE> (-t csv, json or jsonl).",
                true,
            );
        }
    }
    i32::from(parsed.is_empty())
}

// --- index files ----------------------------------------------------------------------------

/// The two index file formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Format {
    /// `$Recycle.Bin`'s `$I` files.
    I,
    /// `RECYCLER` and `RECYCLED`'s INFO2 (and Windows 95's `INFO`).
    Info2,
}

impl Format {
    fn name(self) -> &'static str {
        match self {
            Format::I => "$I",
            Format::Info2 => "INFO2",
        }
    }
}

/// One parsed index file.
#[derive(Debug)]
struct Index {
    format: Format,
    items: Vec<Item>,
    /// Damage outside any record (data after INFO2's last record).
    problems: Vec<String>,
}

/// One deleted item.
#[derive(Debug, Default)]
struct Item {
    source: String,
    version: u32,
    /// FILETIME, as recorded.
    deleted: u64,
    original_path: String,
    /// `None` where the size field can't be read (a 543-byte `$I` file).
    file_size: Option<u64>,
    sid: Option<String>,
    user: Option<String>,
    /// The name of the item in the bin (`$R…`, `Dc1.txt`), when it has one.
    recycled_name: Option<String>,
    /// Its path, beside the index file, and whether it is there.
    recycled: Option<String>,
    recycled_found: Option<bool>,
    // INFO2 only
    record_index: Option<u32>,
    drive: Option<String>,
    removed: Option<bool>,
    short_path: Option<String>,
    record_offset: Option<u64>,
    problems: Vec<String>,
}

impl Item {
    fn deleted_time(&self) -> String {
        crate::lnk::filetime(self.deleted)
    }
}

fn parse_file(path: &Path, code_page: u16) -> Result<Index, String> {
    let meta = path.metadata().map_err(|e| e.to_string())?;
    if meta.len() > MAX_INDEX_SIZE {
        return Err(format!(
            "too large for a Recycle Bin index file ({} bytes)",
            meta.len()
        ));
    }
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let name = file_name(path);
    let source = path.to_string_lossy().into_owned();
    let mut index = match detect(&name, &data) {
        Some(Format::I) => parse_i(&data, &name)?,
        Some(Format::Info2) => parse_info2(&data, code_page)?,
        None => return Err(not_an_index(&data)),
    };
    for item in &mut index.items {
        item.source.clone_from(&source);
    }
    Ok(index)
}

/// The format of an index file: from its name when it has an index file's name, else from
/// its content.
fn detect(name: &str, data: &[u8]) -> Option<Format> {
    let upper = name.to_ascii_uppercase();
    if upper.starts_with("$I") {
        return Some(Format::I);
    }
    if upper == "INFO2" || upper == "INFO" {
        return Some(Format::Info2);
    }
    if info2_header(data).is_some() {
        return Some(Format::Info2);
    }
    match (u64_at(data, 0), data.len()) {
        (Some(1), n) if n == V1_SIZE || n == V1_SIZE - 1 => Some(Format::I),
        (Some(2), n) if n >= V2_PATH_OFFSET => {
            let chars = u32_at(data, I_HEADER)? as usize;
            (n == V2_PATH_OFFSET + 2 * chars).then_some(Format::I)
        }
        _ => None,
    }
}

fn not_an_index(data: &[u8]) -> String {
    match u64_at(data, 0) {
        Some(version) => format!(
            "not a Recycle Bin index file (it starts with 0x{version:016x} and is {} bytes long)",
            data.len()
        ),
        None => format!(
            "too short for a Recycle Bin index file ({} bytes)",
            data.len()
        ),
    }
}

/// The version and record size of an INFO2 header that Windows writes: 280-byte records on
/// Windows 95 (version 0), 98 (4) and ME (5), 800-byte records on NT 4.0 (2) and 2000 to 2003 (5).
fn info2_header(data: &[u8]) -> Option<(u32, usize)> {
    let version = u32_at(data, 0)?;
    let size = u32_at(data, 12)? as usize;
    matches!(
        (size, version),
        (LEGACY_RECORD, 0 | 4 | 5) | (UNICODE_RECORD, 2 | 5)
    )
    .then_some((version, size))
}

fn parse_i(data: &[u8], name: &str) -> Result<Index, String> {
    if data.len() < I_HEADER {
        return Err(format!(
            "too short for a $I file ({} bytes, the header is {I_HEADER})",
            data.len()
        ));
    }
    let version = u64_at(data, 0).unwrap_or_default();
    let mut item = Item {
        recycled_name: recycled_i_name(name),
        ..Item::default()
    };
    let path = match version {
        1 => {
            item.version = 1;
            if data.len() == V1_SIZE - 1 {
                // Some Windows Vista systems write the size in 7 bytes: everything after it is a
                // byte early, and the size is not reliable (as rifiuti2 documents).
                item.deleted = u64_at(data, 15).unwrap_or_default();
                item.problems.push(
                    "543 bytes instead of 544: the size field is 7 bytes long and unreliable, so it is not reported".into(),
                );
                &data[I_HEADER - 1..]
            } else {
                item.file_size = u64_at(data, 8);
                item.deleted = u64_at(data, 16).unwrap_or_default();
                if data.len() > V1_SIZE {
                    item.problems.push(format!(
                        "{} bytes after the 260-character path",
                        data.len() - V1_SIZE
                    ));
                } else if data.len() < V1_SIZE {
                    item.problems.push(format!(
                        "truncated: {} of {V1_SIZE} bytes, so the path may be incomplete",
                        data.len()
                    ));
                }
                &data[I_HEADER..data.len().min(V1_SIZE)]
            }
        }
        2 => {
            item.version = 2;
            item.file_size = u64_at(data, 8);
            item.deleted = u64_at(data, 16).unwrap_or_default();
            let Some(chars) = u32_at(data, I_HEADER) else {
                return Err(format!(
                    "truncated: a version 2 $I file of {} bytes has no path length",
                    data.len()
                ));
            };
            let chars = chars as usize;
            let expected = V2_PATH_OFFSET.saturating_add(chars.saturating_mul(2));
            if chars == 0 || chars > MAX_PATH_CHARS {
                item.problems
                    .push(format!("the path length ({chars} characters) is not valid"));
            }
            if data.len() > expected {
                item.problems.push(format!(
                    "{} bytes after the {chars}-character path",
                    data.len() - expected
                ));
            } else if data.len() < expected {
                item.problems.push(format!(
                    "truncated: {} of {expected} bytes, so the path may be incomplete",
                    data.len()
                ));
            }
            &data[V2_PATH_OFFSET..data.len().min(expected)]
        }
        other => {
            return Err(format!(
                "not a Recycle Bin $I file (format version {other}; Windows writes 1 or 2)"
            ));
        }
    };
    let (text, problem) = utf16(path);
    item.original_path = text;
    item.problems.extend(problem);
    time_problem(&mut item);
    Ok(Index {
        format: Format::I,
        items: vec![item],
        problems: Vec::new(),
    })
}

/// `$R` and the rest of the name of a `$I` file.
fn recycled_i_name(name: &str) -> Option<String> {
    name.get(..2)
        .filter(|prefix| prefix.eq_ignore_ascii_case("$I"))
        .and_then(|_| name.get(2..))
        .map(|rest| format!("$R{rest}"))
}

fn parse_info2(data: &[u8], code_page: u16) -> Result<Index, String> {
    if data.len() < INFO2_HEADER {
        return Err(format!(
            "too short for an INFO2 file ({} bytes, the header is {INFO2_HEADER})",
            data.len()
        ));
    }
    let Some((version, size)) = info2_header(data) else {
        return Err(format!(
            "not an INFO2 file (version {}, record size {}; Windows writes 280-byte records with version 0, 4 or 5, and 800-byte records with version 2 or 5)",
            u32_at(data, 0).unwrap_or_default(),
            u32_at(data, 12).unwrap_or_default()
        ));
    };
    let mut index = Index {
        format: Format::Info2,
        items: Vec::new(),
        problems: Vec::new(),
    };
    let mut offset = INFO2_HEADER;
    while offset < data.len() {
        let rest = data.len() - offset;
        if rest < LEGACY_RECORD {
            index.problems.push(format!(
                "{rest} bytes after the last complete record (records are {size} bytes)"
            ));
            break;
        }
        let record = &data[offset..offset + rest.min(size)];
        index
            .items
            .push(info2_record(record, version, size, offset, code_page));
        offset += size;
    }
    Ok(index)
}

fn info2_record(record: &[u8], version: u32, size: usize, offset: usize, code_page: u16) -> Item {
    let mut item = Item {
        version,
        record_offset: Some(offset as u64),
        ..Item::default()
    };
    let index = u32_at(record, 260).unwrap_or_default();
    let drive = u32_at(record, 264).unwrap_or_default();
    item.record_index = Some(index);
    item.deleted = u64_at(record, 268).unwrap_or_default();
    item.file_size = u32_at(record, 276).map(u64::from);
    // 0–25: A: to Z:. 26 is a UNC path's backslash.
    let letter = match drive {
        0..=25 => Some(char::from(b'A' + drive as u8)),
        26 => Some('\\'),
        _ => None,
    };
    match letter {
        Some('\\') | None => {}
        Some(l) => item.drive = Some(format!("{l}:")),
    }
    if letter.is_none() {
        item.problems
            .push(format!("the drive number ({drive}) is not a drive"));
    }
    // An item no longer in the bin keeps its record with the first byte of its ANSI path
    // cleared: the drive letter (or a UNC path's first backslash) it is put back from.
    let mut ansi = record[..260].to_vec();
    let removed = ansi[0] == 0;
    item.removed = Some(removed);
    if removed && let Some(l) = letter {
        ansi[0] = l as u8;
    }
    let short = shell::codepage::decode(&ansi, code_page);
    if size == UNICODE_RECORD {
        if record.len() < UNICODE_RECORD {
            item.problems.push(format!(
                "truncated: {} of {UNICODE_RECORD} bytes, so the Unicode path may be incomplete",
                record.len()
            ));
        }
        let (unicode, problem) = utf16(&record[LEGACY_RECORD..]);
        item.problems.extend(problem);
        if unicode.is_empty() {
            item.original_path = short;
        } else {
            if short != unicode {
                item.short_path = Some(short);
            }
            item.original_path = unicode;
        }
    } else {
        item.original_path = short;
    }
    if let Some(l) = letter.filter(char::is_ascii_alphabetic) {
        item.recycled_name = Some(format!(
            "D{}{index}{}",
            l.to_ascii_lowercase(),
            extension(&item.original_path)
        ));
    }
    time_problem(&mut item);
    item
}

/// The extension Windows gives an INFO2 item's file in the bin: its original name's, with
/// the dot.
fn extension(path: &str) -> &str {
    let name = path.rsplit(['\\', '/']).next().unwrap_or_default();
    name.rfind('.').map_or("", |dot| &name[dot..])
}

fn time_problem(item: &mut Item) {
    if item.deleted != 0 && item.deleted_time().is_empty() {
        item.problems.push(format!(
            "the deletion time (FILETIME 0x{:016x}) is not a time",
            item.deleted
        ));
    }
}

/// UTF-16LE text up to its first NUL, with a problem when it isn't valid UTF-16 (unpaired
/// surrogates become U+FFFD).
fn utf16(bytes: &[u8]) -> (String, Option<String>) {
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .take_while(|&u| u != 0)
        .collect();
    let mut invalid = 0usize;
    let text = char::decode_utf16(units)
        .map(|c| {
            c.unwrap_or_else(|_| {
                invalid += 1;
                char::REPLACEMENT_CHARACTER
            })
        })
        .collect();
    let problem = (invalid > 0).then(|| {
        format!(
            "the path is not valid UTF-16: {invalid} unpaired surrogates were replaced with U+FFFD"
        )
    });
    (text, problem)
}

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    data.get(at..at + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

fn u64_at(data: &[u8], at: usize) -> Option<u64> {
    data.get(at..at + 8).map(|b| {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(b);
        u64::from_le_bytes(bytes)
    })
}

// --- the bins' users and items ----------------------------------------------------------------

/// A bin's SID and its user's name, from `ProfileList`.
struct Users {
    /// Per index file: the SID of its folder and the SOFTWARE hive of its volume.
    bins: HashMap<PathBuf, (Option<String>, Option<PathBuf>)>,
    /// The only SOFTWARE hive found (beside the bins, or in the directory searched), for bins
    /// on volumes without Windows.
    only: Option<PathBuf>,
    profiles: HashMap<PathBuf, HashMap<String, String>>,
}

impl Users {
    fn find(files: &[PathBuf], installations: &[PathBuf]) -> Self {
        let mut bins = HashMap::new();
        let mut hives: BTreeSet<PathBuf> = installations.iter().cloned().collect();
        for path in files {
            let folder = path.parent();
            let sid = folder
                .and_then(Path::file_name)
                .map(|n| n.to_string_lossy().to_ascii_uppercase())
                .filter(|n| crate::registry::valid_sid(n));
            let software = volume_of(path).and_then(|volume| software_of(&volume));
            hives.extend(software.clone());
            bins.insert(path.clone(), (sid, software));
        }
        Users {
            bins,
            only: (hives.len() == 1)
                .then(|| hives.into_iter().next())
                .flatten(),
            profiles: HashMap::new(),
        }
    }

    /// The SOFTWARE hives read.
    fn hives(&self) -> Vec<PathBuf> {
        let mut hives: BTreeSet<PathBuf> = self
            .bins
            .values()
            .filter_map(|(sid, software)| sid.as_ref().and(software.clone()))
            .collect();
        if self
            .bins
            .values()
            .any(|(sid, software)| sid.is_some() && software.is_none())
        {
            hives.extend(self.only.clone());
        }
        hives.into_iter().collect()
    }

    fn of(&mut self, path: &Path) -> (Option<String>, Option<String>) {
        let Some((Some(sid), software)) = self.bins.get(path).cloned() else {
            return (None, None);
        };
        let user = software.or_else(|| self.only.clone()).and_then(|software| {
            self.profiles
                .entry(software)
                .or_insert_with_key(|software| profile_users(software))
                .get(&sid)
                .cloned()
        });
        (Some(sid), user)
    }
}

/// The volume a bin is on: the folder holding `$Recycle.Bin`, `RECYCLER` or `RECYCLED`.
fn volume_of(path: &Path) -> Option<PathBuf> {
    path.ancestors().skip(1).find_map(|dir| {
        let name = dir.file_name()?.to_string_lossy();
        BIN_FOLDERS
            .iter()
            .any(|bin| name.eq_ignore_ascii_case(bin))
            .then(|| {
                dir.parent()
                    .filter(|d| !d.as_os_str().is_empty())
                    .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
            })
    })
}

/// `Windows\System32\config\SOFTWARE` under `volume`, each name in any case.
fn software_of(volume: &Path) -> Option<PathBuf> {
    let mut at = volume.to_path_buf();
    for name in ["Windows", "System32", "config", "SOFTWARE"] {
        at = std::fs::read_dir(&at)
            .ok()?
            .flatten()
            .find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(name))?
            .path();
    }
    at.is_file().then_some(at)
}

/// SID (upper case) → user: the last folder of each profile's `ProfileImagePath`.
fn profile_users(software: &Path) -> HashMap<String, String> {
    crate::registry::open(software, crate::registry::Logs::NextToIt)
        .ok()
        .filter(|o| o.hive_type == "SOFTWARE")
        .map(|o| registry::software::profile_list(&o.hive))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(sid, path)| {
            let user = path
                .trim_end_matches(['\\', '/'])
                .rsplit(['\\', '/'])
                .next()?
                .to_owned();
            (!user.is_empty()).then(|| (sid.to_ascii_uppercase(), user))
        })
        .collect()
}

/// The names in each bin folder, read once, to find items in any case.
#[derive(Default)]
struct Listings(HashMap<PathBuf, HashMap<String, String>>);

impl Listings {
    /// The name of the entry beside `index` called `name` in any case.
    fn find(&mut self, index: &Path, name: &str) -> Option<String> {
        let dir = index
            .parent()
            .filter(|d| !d.as_os_str().is_empty())
            .unwrap_or(Path::new("."))
            .to_path_buf();
        self.0
            .entry(dir)
            .or_insert_with_key(|dir| {
                std::fs::read_dir(dir)
                    .map(|entries| {
                        entries
                            .flatten()
                            .map(|e| {
                                let name = e.file_name().to_string_lossy().into_owned();
                                (name.to_lowercase(), name)
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            })
            .get(&name.to_lowercase())
            .cloned()
    }
}

/// `name` beside the file at `path`, written as `path` is.
fn sibling(path: &Path, name: &str) -> String {
    path.parent()
        .filter(|d| !d.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from(name), |d| d.join(name))
        .to_string_lossy()
        .into_owned()
}

// --- output ---------------------------------------------------------------------------------

const CSV_HEADER: &[&str] = &[
    "SourceFile",
    "DeletedTime",
    "OriginalPath",
    "FileSize",
    "User",
    "SID",
    "RecycledFile",
    "RecycledFileFound",
    "Format",
    "FormatVersion",
    "RecordIndex",
    "Drive",
    "RemovedFromBin",
    "ShortPath",
    "RecordOffset",
    "Problems",
];

fn text<T: ToString>(value: Option<T>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}

fn csv_row(format: Format, item: &Item) -> Vec<String> {
    vec![
        item.source.clone(),
        item.deleted_time(),
        item.original_path.clone(),
        text(item.file_size),
        text(item.user.as_ref()),
        text(item.sid.as_ref()),
        text(item.recycled.as_ref()),
        text(item.recycled_found),
        format.name().to_owned(),
        item.version.to_string(),
        text(item.record_index),
        text(item.drive.as_ref()),
        text(item.removed),
        text(item.short_path.as_ref()),
        text(item.record_offset),
        item.problems.join("; "),
    ]
}

fn json_record(format: Format, item: &Item) -> Value {
    let deleted = item.deleted_time();
    json!({
        "SourceFile": item.source,
        "DeletedTime": (!deleted.is_empty()).then_some(deleted),
        "OriginalPath": item.original_path,
        "FileSize": item.file_size,
        "User": item.user,
        "SID": item.sid,
        "RecycledFile": item.recycled,
        "RecycledFileFound": item.recycled_found,
        "Format": format.name(),
        "FormatVersion": item.version,
        "RecordIndex": item.record_index,
        "Drive": item.drive,
        "RemovedFromBin": item.removed,
        "ShortPath": item.short_path,
        "RecordOffset": item.record_offset,
        "Problems": item.problems,
    })
}

fn write_records(
    mut w: Box<dyn Write>,
    output_type: RecycleBinOutputType,
    parsed: &[Index],
) -> io::Result<()> {
    let items = parsed
        .iter()
        .flat_map(|index| index.items.iter().map(move |item| (index.format, item)));
    if output_type == RecycleBinOutputType::Csv {
        let mut csv = csv::Writer::from_writer(w);
        csv.write_record(crate::output_schema::labels(CSV_HEADER.iter().copied())?)?;
        for (format, item) in items {
            csv.write_record(csv_row(format, item))?;
        }
        return csv.flush();
    }
    for (format, item) in items {
        let v = json_record(format, item);
        crate::output_schema::write_json(&mut w, &v, output_type == RecycleBinOutputType::Json)?;
        w.write_all(b"\n")?;
    }
    w.flush()
}

fn print_summary(parsed: &[Index], failed: usize, no_color: bool) {
    p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
    let count = |format: Format| parsed.iter().filter(|i| i.format == format).count();
    let mut files = format!(
        "{} ($I files: {}, INFO2: {})",
        fmt_num(parsed.len()),
        fmt_num(count(Format::I)),
        fmt_num(count(Format::Info2))
    );
    if failed > 0 {
        files.push_str(&format!(" (could not parse: {})", fmt_num(failed)));
    }
    p_label(no_color, "Index files parsed: ", &files);
    let items: Vec<&Item> = parsed.iter().flat_map(|i| &i.items).collect();
    p_label(no_color, "Deleted items: ", &fmt_num(items.len()));
    let times: Vec<u64> = items
        .iter()
        .filter(|i| !i.deleted_time().is_empty())
        .map(|i| i.deleted)
        .collect();
    if let (Some(first), Some(last)) = (times.iter().min(), times.iter().max()) {
        p_label(
            no_color,
            "Deletion times: ",
            &format!(
                "{} - {}",
                crate::lnk::filetime(*first),
                crate::lnk::filetime(*last)
            ),
        );
    }
    let mut bins: BTreeMap<String, usize> = BTreeMap::new();
    for item in &items {
        let owner = item
            .user
            .clone()
            .or_else(|| item.sid.clone())
            .unwrap_or_else(|| "(no SID)".into());
        *bins.entry(owner).or_default() += 1;
    }
    let mut shown: Vec<String> = bins
        .iter()
        .take(5)
        .map(|(owner, n)| format!("{owner}: {}", fmt_num(*n)))
        .collect();
    if bins.len() > shown.len() {
        shown.push("…".into());
    }
    if !bins.is_empty() {
        p_label(no_color, "Items by user: ", &shown.join(", "));
    }
    let with_name = items.iter().filter(|i| i.recycled.is_some()).count();
    let found = items
        .iter()
        .filter(|i| i.recycled_found == Some(true))
        .count();
    p_label(
        no_color,
        "Deleted content collected ($R / D files): ",
        &format!("{} of {}", fmt_num(found), fmt_num(with_name)),
    );
    let removed = items.iter().filter(|i| i.removed == Some(true)).count();
    if removed > 0 {
        p_label(
            no_color,
            "INFO2 records of items restored or deleted from the bin: ",
            &fmt_num(removed),
        );
    }
    let damaged = items.iter().filter(|i| !i.problems.is_empty()).count()
        + parsed.iter().filter(|i| !i.problems.is_empty()).count();
    p_label(no_color, "With problems: ", &fmt_num(damaged));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16z(text: &str) -> Vec<u8> {
        text.encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect()
    }

    fn i_v1(size: u64, time: u64, path: &str) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(1u64.to_le_bytes());
        data.extend(size.to_le_bytes());
        data.extend(time.to_le_bytes());
        let mut name = utf16z(path);
        name.resize(V1_PATH_BYTES, 0);
        data.extend(name);
        data
    }

    fn i_v2(size: u64, time: u64, path: &str) -> Vec<u8> {
        let name = utf16z(path);
        let mut data = Vec::new();
        data.extend(2u64.to_le_bytes());
        data.extend(size.to_le_bytes());
        data.extend(time.to_le_bytes());
        data.extend(((name.len() / 2) as u32).to_le_bytes());
        data.extend(name);
        data
    }

    /// ANSI path, index, drive, deletion time, size and Unicode path.
    type Record<'a> = (&'a [u8], u32, u32, u64, u32, &'a str);

    fn info2(records: &[Record<'_>]) -> Vec<u8> {
        let mut data = Vec::new();
        for value in [5u32, records.len() as u32, records.len() as u32, 800, 0] {
            data.extend(value.to_le_bytes());
        }
        for (ansi, index, drive, time, size, unicode) in records {
            let mut legacy = ansi.to_vec();
            legacy.resize(260, 0);
            data.extend(legacy);
            data.extend(index.to_le_bytes());
            data.extend(drive.to_le_bytes());
            data.extend(time.to_le_bytes());
            data.extend(size.to_le_bytes());
            let mut name = utf16z(unicode);
            name.resize(520, 0);
            data.extend(name);
        }
        data
    }

    /// 2015-04-04T17:19:52Z
    const TIME: u64 = 130_726_415_920_000_000;

    #[test]
    fn index_names_are_six_characters_after_dollar_i() {
        for name in [
            "$IKEGS1G",
            "$IQ7LAXT.png",
            "$iabc123.tar.gz",
            "INFO2",
            "info2",
        ] {
            assert!(is_index_name(name), "{name}");
        }
        for name in [
            "$I30",
            "$I30:$INDEX_ALLOCATION",
            "Users_$I30",
            "$IABC12",
            "$IABC1234",
            "$RKEGS1G",
            "INFO",
            "desktop.ini",
            "$IABC12é",
        ] {
            assert!(!is_index_name(name), "{name}");
        }
    }

    #[test]
    fn version_one_and_two_give_size_time_and_path() {
        let v1 = parse_i(&i_v1(6455, TIME, r"C:\Users\a\x.png"), "$IQ7LAXT.png").unwrap();
        let item = &v1.items[0];
        assert_eq!(
            (item.version, item.file_size, item.deleted_time().as_str()),
            (1, Some(6455), "2015-04-04T17:19:52Z")
        );
        assert_eq!(item.original_path, r"C:\Users\a\x.png");
        assert_eq!(item.recycled_name.as_deref(), Some("$RQ7LAXT.png"));
        assert!(item.problems.is_empty());

        let long = format!(r"C:\{}", "d".repeat(400));
        let v2 = parse_i(&i_v2(12_884_901_888, TIME, &long), "$IHO61YT").unwrap();
        let item = &v2.items[0];
        assert_eq!((item.version, item.file_size), (2, Some(12_884_901_888)));
        assert_eq!(item.original_path, long);
        assert!(item.problems.is_empty(), "{:?}", item.problems);
    }

    #[test]
    fn damaged_i_files_keep_what_can_be_read() {
        // 543 bytes: the size field is 7 bytes long (a Windows Vista quirk).
        let mut quirk = i_v1(0, 0, "");
        quirk.truncate(15);
        quirk.extend(TIME.to_le_bytes());
        let mut name = utf16z(r"C:\Temp\dd.exe");
        name.resize(V1_PATH_BYTES, 0);
        quirk.extend(name);
        assert_eq!(quirk.len(), 543);
        let item = &parse_i(&quirk, "$IROMPZ0.exe").unwrap().items[0];
        assert_eq!(item.file_size, None);
        assert_eq!(item.deleted_time(), "2015-04-04T17:19:52Z");
        assert_eq!(item.original_path, r"C:\Temp\dd.exe");
        assert!(item.problems[0].starts_with("543 bytes"));

        let mut truncated = i_v2(1, TIME, r"C:\Temp\long name.txt");
        truncated.truncate(40);
        let item = &parse_i(&truncated, "$IABCDEF.txt").unwrap().items[0];
        assert_eq!(item.original_path, r"C:\Tem");
        assert!(item.problems[0].starts_with("truncated"));

        let mut extra = i_v2(1, TIME, r"C:\a");
        extra.extend([0xAA; 3]);
        let item = &parse_i(&extra, "$IABCDEF").unwrap().items[0];
        assert_eq!(item.original_path, r"C:\a");
        assert_eq!(item.problems, ["3 bytes after the 5-character path"]);

        let mut bad = i_v2(1, u64::MAX, "");
        bad.truncate(V2_PATH_OFFSET);
        bad.extend([0x00, 0xD8, b'a', 0, 0, 0]);
        bad[I_HEADER..V2_PATH_OFFSET].copy_from_slice(&3u32.to_le_bytes());
        let item = &parse_i(&bad, "$IABCDEF").unwrap().items[0];
        assert_eq!(item.original_path, "\u{fffd}a");
        assert_eq!(item.deleted_time(), "");
        assert_eq!(item.problems.len(), 2, "{:?}", item.problems);

        assert!(parse_i(&[0u8; 10], "$IABCDEF").is_err());
        let mut three = i_v2(1, TIME, "x");
        three[0] = 3;
        assert!(
            parse_i(&three, "$IABCDEF")
                .unwrap_err()
                .contains("format version 3")
        );
    }

    #[test]
    fn info2_records_restore_the_first_byte_of_removed_items() {
        let data = info2(&[
            (
                br"C:\DOCUME~1\alice\Desktop\REPORT~1.DOC",
                1,
                2,
                TIME,
                4096,
                r"C:\Documents and Settings\alice\Desktop\Report final.doc",
            ),
            (b"\0:\\temp\\a.txt", 2, 3, TIME, 0, r"D:\temp\a.txt"),
            (b"\0\\server\\share\\b.txt", 3, 26, 0, 0, ""),
        ]);
        let index = parse_info2(&data, 1252).unwrap();
        assert!(index.problems.is_empty());
        let [a, b, c] = &index.items[..] else {
            panic!("{:?}", index.items)
        };
        assert_eq!(
            a.original_path,
            r"C:\Documents and Settings\alice\Desktop\Report final.doc"
        );
        assert_eq!(
            a.short_path.as_deref(),
            Some(r"C:\DOCUME~1\alice\Desktop\REPORT~1.DOC")
        );
        assert_eq!(a.recycled_name.as_deref(), Some("Dc1.doc"));
        assert_eq!(
            (a.removed, a.drive.as_deref(), a.record_offset),
            (Some(false), Some("C:"), Some(20))
        );
        assert_eq!((b.removed, b.short_path.as_deref()), (Some(true), None));
        assert_eq!(b.recycled_name.as_deref(), Some("Dd2.txt"));
        assert_eq!(c.original_path, r"\\server\share\b.txt");
        assert_eq!(
            (c.drive.as_deref(), c.recycled_name.as_deref()),
            (None, None)
        );
        assert_eq!(c.record_offset, Some(1620));
    }

    #[test]
    fn info2_damage_is_reported() {
        let mut data = info2(&[(b"C:\\a.txt", 1, 99, TIME, 1, r"C:\a.txt")]);
        data.extend([1u8; 100]);
        let index = parse_info2(&data, 1252).unwrap();
        assert_eq!(
            index.items[0].problems,
            ["the drive number (99) is not a drive"]
        );
        assert_eq!(
            index.problems,
            ["100 bytes after the last complete record (records are 800 bytes)"]
        );

        let mut truncated = info2(&[(b"C:\\a.txt", 1, 2, TIME, 1, r"C:\abc.txt")]);
        truncated.truncate(INFO2_HEADER + 290);
        let index = parse_info2(&truncated, 1252).unwrap();
        assert_eq!(index.items[0].original_path, r"C:\ab");
        assert!(index.items[0].problems[0].starts_with("truncated"));

        let mut wrong = info2(&[]);
        wrong[12] = 0x10;
        assert!(parse_info2(&wrong, 1252).is_err());
        assert!(parse_info2(&info2(&[]), 1252).unwrap().items.is_empty());
    }

    #[test]
    fn legacy_info2_paths_use_the_code_page() {
        let mut data = Vec::new();
        for value in [4u32, 1, 1, 280, 0] {
            data.extend(value.to_le_bytes());
        }
        let mut ansi = b"C:\\\x83e\x83X\x83g.txt".to_vec();
        ansi.resize(260, 0);
        data.extend(ansi);
        data.extend(7u32.to_le_bytes());
        data.extend(2u32.to_le_bytes());
        data.extend(TIME.to_le_bytes());
        data.extend(10u32.to_le_bytes());
        let item = &parse_info2(&data, 932).unwrap().items[0];
        assert_eq!(item.original_path, "C:\\テスト.txt");
        assert_eq!(item.recycled_name.as_deref(), Some("Dc7.txt"));
        assert_eq!(item.short_path, None);
    }

    #[test]
    fn formats_are_detected_by_name_then_content() {
        let v2 = i_v2(1, TIME, r"C:\a");
        assert_eq!(detect("exported.bin", &v2), Some(Format::I));
        assert_eq!(detect("exported.bin", &i_v1(1, TIME, "x")), Some(Format::I));
        assert_eq!(detect("copy-of-info", &info2(&[])), Some(Format::Info2));
        assert_eq!(detect("notes.txt", b"hello world, not an index"), None);
        assert_eq!(detect("$IABCDEF", b""), Some(Format::I));
    }

    #[test]
    fn bins_give_volume_and_sid() {
        let path = Path::new("HOST/C/$Recycle.Bin/S-1-5-21-1-2-3-1001/$IABCDEF.txt");
        assert_eq!(volume_of(path), Some(PathBuf::from("HOST/C")));
        assert_eq!(
            volume_of(Path::new("RECYCLER/S-1-5-21-1-2-3-500/INFO2")),
            Some(PathBuf::from("."))
        );
        assert_eq!(volume_of(Path::new("exported/$IABCDEF")), None);
        assert_eq!(extension(r"C:\a\b.tar.gz"), ".gz");
        assert_eq!(extension(r"C:\a.d\b"), "");
    }
}
