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

//! `parse-file powershell-history`: the command history PSReadLine saves for each user, one row
//! per line of each history file.
//!
//! - `Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt`
//!   (Windows PowerShell 5.1 and PowerShell 7 share it): `ConsoleHost_history.txt` for the
//!   console, `Visual Studio Code Host_history.txt` for the editor's integrated console.
//!   PowerShell on Linux and macOS keeps the same files in
//!   `~/.local/share/powershell/PSReadLine`.
//! - The file holds command lines only, oldest first, with no time for any of them. A command of
//!   several lines is saved with a backtick at the end of every line but its last.
//!
//! Nothing is guessed: every line is a row with its own line number and its text as it is in
//! the file, and the only time is the file's own modification time (`FileLastWriteTime`), which
//! is not when any command ran. The file is read and its rows are written as a stream, so a
//! history of any size needs little memory.

use crate::cli::{PowerShellHistoryOptions, PowerShellHistoryOutputType};
use crate::color::JunteiColor;
use crate::image_metadata::SourceTimes;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use serde_json::json;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};

/// Files that could not be read are printed up to this many; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// What every PSReadLine history file's name ends with, after the name of its host.
const NAME_SUFFIX: &str = "_history.txt";
/// The folder PSReadLine keeps its history files in.
const HISTORY_FOLDER: &str = "PSReadLine";
/// How much of a file is read at a time.
const CHUNK_BYTES: usize = 64 * 1024;
/// The bytes of one line that are kept. A longer line (a file that is not a history, or one
/// without line ends) is cut here with a problem, so that no line needs unbounded memory.
const MAX_LINE_BYTES: usize = 1024 * 1024;

/// A PSReadLine history file's name: `<Host>_history.txt` in any case, with a host name.
pub fn is_history_name(name: &str) -> bool {
    host_of(name).is_some()
}

/// The host a history file's name gives (`ConsoleHost`), when it is a history file's name.
fn host_of(name: &str) -> Option<&str> {
    let at = name.len().checked_sub(NAME_SUFFIX.len())?;
    (at > 0 && name.is_char_boundary(at) && name[at..].eq_ignore_ascii_case(NAME_SUFFIX))
        .then(|| &name[..at])
}

/// The history files `-d`, `auto-parse` and disk-image staging select: a history file's name
/// directly in a `PSReadLine` folder. Other `*_history.txt` files are not PowerShell's.
pub(crate) fn is_history_path(path: &Path) -> bool {
    let parts = parts(path);
    match parts.as_slice() {
        [.., folder, name] => folder.eq_ignore_ascii_case(HISTORY_FOLDER) && is_history_name(name),
        _ => false,
    }
}

/// The names in a path, without its root, `.` and `..`. A backslash separates names too: a
/// collection copied without its folders can keep a Windows path in a single file name.
fn parts(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            _ => None,
        })
        .flat_map(|name| {
            name.split('\\')
                .filter(|part| !part.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The user a history file is in the profile of, from where PSReadLine keeps it: the folder
/// above `AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine` (or above
/// `.local/share/powershell/PSReadLine`).
fn profile_user(parts: &[String]) -> Option<String> {
    const WINDOWS: [&str; 6] = [
        "AppData",
        "Roaming",
        "Microsoft",
        "Windows",
        "PowerShell",
        HISTORY_FOLDER,
    ];
    const UNIX: [&str; 4] = [".local", "share", "powershell", HISTORY_FOLDER];
    let folders = parts.split_last()?.1;
    [&WINDOWS[..], &UNIX[..]].into_iter().find_map(|layout| {
        let at = folders.len().checked_sub(layout.len() + 1)?;
        folders[at + 1..]
            .iter()
            .zip(layout)
            .all(|(folder, expected)| folder.eq_ignore_ascii_case(expected))
            .then(|| folders[at].clone())
    })
}

/// One history file to read.
struct Source {
    path: PathBuf,
    /// `SourceFile`: the path below the input directory with `/` between its names, or the name
    /// of a file given with `-f`. Never an absolute path.
    shown: String,
    user: Option<String>,
    /// The host of the file's name; empty for a file given with `-f` under another name.
    host: String,
}

impl Source {
    /// A file found under the directory `root`. Its user is read from the path below `root`
    /// only, never from the analyst's own folders above it.
    fn under(root: &Path, path: PathBuf) -> Self {
        let parts = parts(path.strip_prefix(root).unwrap_or(&path));
        let user = profile_user(&parts).or_else(|| {
            crate::browser::path_user(&parts.iter().collect::<PathBuf>())
                .filter(|user| !user.is_empty())
        });
        Self::new(path, parts.join("/"), user, &parts)
    }

    /// A file given with `-f`: shown by its name. Its user is named only by the folders
    /// PSReadLine keeps the file in, since any other folder may be the analyst's own.
    fn given(path: PathBuf) -> Self {
        let parts = parts(&path);
        let name = parts.last().cloned().unwrap_or_default();
        Self::new(path, name, profile_user(&parts), &parts)
    }

    fn new(path: PathBuf, shown: String, user: Option<String>, parts: &[String]) -> Self {
        let host = parts
            .last()
            .and_then(|name| host_of(name))
            .unwrap_or_default()
            .to_owned();
        Self {
            path,
            shown,
            user,
            host,
        }
    }
}

/// The history files under `dir`, recursively, in path order.
fn find_histories(dir: &Path) -> Vec<Source> {
    util::find_files(dir, is_history_name)
        .into_iter()
        .filter(|path| is_history_path(path))
        .map(|path| Source::under(dir, path))
        .collect()
}

/// Run the command; returns the exit status.
pub fn run(opts: &PowerShellHistoryOptions) -> i32 {
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
    let sources = match (&opts.filepath, &opts.directory) {
        (Some(f), _) => {
            if !f.is_file() {
                report_error(None, &format!("The file does not exist: {}", f.display()));
                return 1;
            }
            vec![Source::given(f.clone())]
        }
        (None, Some(d)) => {
            if !d.is_dir() {
                report_error(
                    None,
                    &format!("The directory does not exist: {}", d.display()),
                );
                return 1;
            }
            let found = find_histories(d);
            if found.is_empty() {
                report_error(
                    None,
                    &format!(
                        "No PowerShell history files (PSReadLine\\*_history.txt) were found in {}",
                        d.display()
                    ),
                );
                return 1;
            }
            found
        }
        (None, None) => unreachable!("clap requires -f or -d"),
    };
    let source_times = match SourceTimes::load(opts.image_source_metadata.as_deref()) {
        Ok(times) => times,
        Err(e) => {
            report_error(None, &e);
            return 1;
        }
    };
    // An output that is one of the history files (by path, symbolic link or hard link) is
    // refused before anything is written, also with --clobber.
    let sink =
        match crate::registry::sink(opts.output.as_ref(), opts.clobber, data_on_stdout, || {
            sources.iter().map(|source| source.path.clone()).collect()
        }) {
            Ok(sink) => sink,
            Err(msg) => {
                report_error(None, &msg);
                return 1;
            }
        };
    let mut records = match sink
        .map(|sink| Records::new(sink, opts.output_type()))
        .transpose()
    {
        Ok(records) => records,
        Err(e) => {
            report_error(None, &format!("Could not write the records: {e}"));
            return 1;
        }
    };
    if !data_on_stdout {
        let total: u64 = sources
            .iter()
            .map(|source| source.path.metadata().map_or(0, |m| m.len()))
            .sum();
        p_label(
            no_color,
            "PowerShell history files: ",
            &fmt_num(sources.len()),
        );
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
        progress::progress_bar(sources.len(), no_color)
    });

    // --- read and write ---------------------------------------------------------------------
    let mut stats = Stats::default();
    for source in &sources {
        if let Some(b) = &bar {
            b.set_message(progress::message(&source.path));
        }
        match parse_file(source, &source_times, records.as_mut(), &mut stats) {
            Ok(problems) => {
                stats.files += 1;
                for problem in problems {
                    report_error(bar.as_ref(), &format!("{}: {problem}", source.shown));
                }
            }
            Err(Stop::Unreadable(e)) => {
                stats.unreadable += 1;
                if stats.unreadable <= MAX_PRINTED_ERRORS {
                    report_error(
                        bar.as_ref(),
                        &format!("Could not read {}: {e}", source.shown),
                    );
                }
            }
            // The output is gone (a full disk, a closed pipe): reading on cannot save anything.
            Err(Stop::Write(e)) => {
                if let Some(b) = &bar {
                    b.abandon();
                }
                report_error(None, &format!("Could not write the records: {e}"));
                return 1;
            }
        }
        if let Some(b) = &bar {
            b.inc(1);
        }
    }
    if stats.unreadable > MAX_PRINTED_ERRORS {
        report_error(
            bar.as_ref(),
            &format!(
                "{} more files could not be read",
                fmt_num(stats.unreadable - MAX_PRINTED_ERRORS)
            ),
        );
    }
    if let Some(b) = &bar {
        b.finish_with_message(progress::finished_message(no_color));
    }
    if let Some(records) = records
        && let Err(e) = records.finish()
    {
        report_error(None, &format!("Could not write the records: {e}"));
        return 1;
    }

    // --- summary ---------------------------------------------------------------------------
    if !data_on_stdout {
        print_summary(&stats, no_color);
        if stats.files > 0 {
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
    }
    i32::from(stats.files == 0)
}

/// Why a history file gave nothing more.
enum Stop {
    /// The file could not be opened; the other files are still read.
    Unreadable(String),
    /// The records could not be written; the run ends.
    Write(io::Error),
}

/// Read one history file and write its rows as they are read. Gives the file's problems, which
/// are printed: damage in a line is also in that line's `Problems`.
fn parse_file(
    source: &Source,
    source_times: &SourceTimes,
    mut records: Option<&mut Records>,
    stats: &mut Stats,
) -> Result<Vec<String>, Stop> {
    let meta = source
        .path
        .metadata()
        .map_err(|e| Stop::Unreadable(e.to_string()))?;
    let file = File::open(&source.path).map_err(|e| Stop::Unreadable(e.to_string()))?;
    let mut problems = Vec::new();
    // With a disk image the time is the image's, never that of the copy made to read it.
    let last_write = match source_times.for_file(&source.path, &meta) {
        Ok([_, modified, _]) => crate::lnk::system_time(modified),
        Err(e) => {
            problems.push(format!("FileLastWriteTime is unavailable: {e}"));
            String::new()
        }
    };
    if source.path.to_str().is_none() {
        problems.push(
            "The source path contains non-Unicode bytes; its displayed spelling uses U+FFFD".into(),
        );
    }
    let file_info = FileInfo {
        source,
        last_write: &last_write,
        size: meta.len(),
    };
    let before = stats.lines;
    let result = read_history(file, CHUNK_BYTES, |line| {
        stats.lines += 1;
        if line.entry_start == line.number {
            stats.entries += 1;
        }
        if !line.problems.is_empty() {
            stats.with_problems += 1;
        }
        match records.as_deref_mut() {
            Some(records) => records.write(&file_info, &line),
            None => Ok(()),
        }
    });
    let user = source.user.clone().unwrap_or_else(|| "(unknown)".into());
    *stats.by_user.entry(user).or_default() += stats.lines - before;
    let outcome = result.map_err(Stop::Write)?;
    stats.empty_lines += outcome.empty_lines;
    problems.extend(outcome.problems);
    Ok(problems)
}

// --- reading --------------------------------------------------------------------------------

/// How a history file's bytes are text, from its byte order mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Encoding {
    Utf8,
    Utf8Bom,
    Utf16Le,
    Utf16Be,
}

impl Encoding {
    fn name(self) -> &'static str {
        match self {
            Encoding::Utf8 => "UTF-8",
            Encoding::Utf8Bom => "UTF-8-BOM",
            Encoding::Utf16Le => "UTF-16LE",
            Encoding::Utf16Be => "UTF-16BE",
        }
    }

    fn is_utf16(self) -> bool {
        matches!(self, Encoding::Utf16Le | Encoding::Utf16Be)
    }

    /// The encoding a file's first bytes give, and the length of its byte order mark. PSReadLine
    /// writes UTF-8 without one; a file saved by an editor can have one. Without a mark the
    /// file is read as UTF-8, also when fewer bytes than a whole mark are there.
    fn detect(head: &[u8]) -> (Self, usize) {
        match head {
            [0xEF, 0xBB, 0xBF, ..] => (Encoding::Utf8Bom, 3),
            [0xFF, 0xFE, ..] => (Encoding::Utf16Le, 2),
            [0xFE, 0xFF, ..] => (Encoding::Utf16Be, 2),
            _ => (Encoding::Utf8, 0),
        }
    }
}

/// One line of a history file.
#[derive(Debug, PartialEq, Eq)]
struct Line {
    /// The line's number in the file, from 1, counting every line (also the empty ones).
    number: u64,
    /// The number of the first line of the command this line is part of: the line itself, or
    /// the start of the lines before it that each end with a backtick.
    entry_start: u64,
    /// The line as it is in the file, without its line end.
    text: String,
    encoding: Encoding,
    problems: Vec<String>,
}

/// What reading a whole file found beside its lines.
#[derive(Debug, Default)]
struct Outcome {
    /// Lines without any character: counted in the line numbers, not given as rows.
    empty_lines: u64,
    /// Damage that is the file's rather than one line's (a read error, half a code unit).
    problems: Vec<String>,
}

/// Splits a file's bytes into lines as they arrive, keeping only the line being read.
struct Splitter {
    encoding: Encoding,
    /// The bytes of the line being read, up to [`MAX_LINE_BYTES`].
    line: Vec<u8>,
    /// The bytes of the line being read that were not kept.
    dropped: u64,
    /// UTF-16: the first byte of a code unit whose second byte is in the next chunk.
    half: Option<u8>,
    number: u64,
    entry_start: u64,
    /// Whether the line before ended with a backtick.
    continues: bool,
    empty_lines: u64,
}

impl Splitter {
    fn new(encoding: Encoding) -> Self {
        Self {
            encoding,
            line: Vec::new(),
            dropped: 0,
            half: None,
            number: 0,
            entry_start: 0,
            continues: false,
            empty_lines: 0,
        }
    }

    fn keep(&mut self, bytes: &[u8]) {
        let room = MAX_LINE_BYTES - self.line.len();
        let kept = bytes.len().min(room);
        self.line.extend_from_slice(&bytes[..kept]);
        self.dropped += (bytes.len() - kept) as u64;
    }

    /// Take the next bytes of the file, giving every line they complete.
    fn push(
        &mut self,
        mut bytes: &[u8],
        emit: &mut impl FnMut(Line) -> io::Result<()>,
    ) -> io::Result<()> {
        if !self.encoding.is_utf16() {
            // A line feed byte is never part of another UTF-8 character.
            while let Some(at) = bytes.iter().position(|b| *b == b'\n') {
                self.keep(&bytes[..at]);
                self.finish(Vec::new(), false, emit)?;
                bytes = &bytes[at + 1..];
            }
            self.keep(bytes);
            return Ok(());
        }
        if let Some(first) = self.half.take() {
            let Some((second, rest)) = bytes.split_first() else {
                self.half = Some(first);
                return Ok(());
            };
            self.unit([first, *second], emit)?;
            bytes = rest;
        }
        let (units, rest) = bytes.as_chunks::<2>();
        for unit in units {
            self.unit(*unit, emit)?;
        }
        self.half = rest.first().copied();
        Ok(())
    }

    fn unit(
        &mut self,
        unit: [u8; 2],
        emit: &mut impl FnMut(Line) -> io::Result<()>,
    ) -> io::Result<()> {
        let line_feed = match self.encoding {
            Encoding::Utf16Be => [0, b'\n'],
            _ => [b'\n', 0],
        };
        if unit == line_feed {
            self.finish(Vec::new(), false, emit)
        } else {
            self.keep(&unit);
            Ok(())
        }
    }

    /// The file ended, or could not be read further (`read_error`): give what is left of its
    /// last line. Gives the file's problems.
    fn end(
        &mut self,
        read_error: Option<String>,
        emit: &mut impl FnMut(Line) -> io::Result<()>,
    ) -> io::Result<Vec<String>> {
        let mut problems = Vec::new();
        let mut line_problems = Vec::new();
        let half = self.half.take().is_some();
        if half {
            let problem = "the file ends with half a UTF-16 code unit (an odd number of bytes), shown as U+FFFD";
            problems.push(problem.to_owned());
            line_problems.push(problem.to_owned());
        }
        if let Some(error) = read_error {
            problems.push(error.clone());
            line_problems.push(format!(
                "{error}; this line may be incomplete and the lines after it are missing"
            ));
        }
        if !self.line.is_empty() || self.dropped > 0 || !line_problems.is_empty() {
            self.finish(line_problems, half, emit)?;
        }
        Ok(problems)
    }

    /// The line being read is complete: decode it, and give it unless it is empty.
    fn finish(
        &mut self,
        mut problems: Vec<String>,
        half_unit: bool,
        emit: &mut impl FnMut(Line) -> io::Result<()>,
    ) -> io::Result<()> {
        self.number += 1;
        let mut dropped = std::mem::take(&mut self.dropped);
        let mut raw = std::mem::take(&mut self.line);
        if dropped > 0 {
            // The cut must not leave part of a character behind to be reported as damage.
            let whole = if self.encoding.is_utf16() {
                whole_utf16(&raw, self.encoding)
            } else {
                whole_utf8(&raw)
            };
            dropped += (raw.len() - whole) as u64;
            raw.truncate(whole);
        }
        let (mut text, invalid) = if self.encoding.is_utf16() {
            decode_utf16(&raw, self.encoding)
        } else {
            decode_utf8(&raw)
        };
        raw.clear();
        // Keep the allocation for the next line.
        self.line = raw;
        if dropped == 0 && text.ends_with('\r') {
            text.pop();
        }
        if half_unit {
            text.push(char::REPLACEMENT_CHARACTER);
        }
        if invalid > 0 {
            problems.push(if self.encoding.is_utf16() {
                format!("{invalid} unpaired UTF-16 surrogate(s) shown as U+FFFD")
            } else {
                format!("{invalid} invalid UTF-8 byte sequence(s) shown as U+FFFD")
            });
        }
        let nuls = text.bytes().filter(|b| *b == 0).count();
        if nuls > 0 {
            problems.push(if self.encoding == Encoding::Utf8 {
                format!(
                    "{nuls} NUL character(s): the file may be UTF-16 without a byte order mark, or not text"
                )
            } else {
                format!("{nuls} NUL character(s)")
            });
        }
        if dropped > 0 {
            problems.push(format!(
                "the line is longer than {MAX_LINE_BYTES} bytes: its last {dropped} bytes are not shown"
            ));
        }
        if text.is_empty() && problems.is_empty() {
            // PSReadLine saves no empty command, and a line it continues ends with a backtick.
            self.empty_lines += 1;
            self.continues = false;
            return Ok(());
        }
        if !self.continues {
            self.entry_start = self.number;
        }
        self.continues = text.ends_with('`');
        emit(Line {
            number: self.number,
            entry_start: self.entry_start,
            text,
            encoding: self.encoding,
            problems,
        })
    }
}

/// The length of `bytes` without a UTF-8 character that a cut at their end left unfinished.
fn whole_utf8(bytes: &[u8]) -> usize {
    for back in 1..=bytes.len().min(3) {
        let byte = bytes[bytes.len() - back];
        if byte & 0xC0 == 0x80 {
            continue;
        }
        let needed = match byte {
            0xF0.. => 4,
            0xE0.. => 3,
            0xC0.. => 2,
            _ => 1,
        };
        return if needed > back {
            bytes.len() - back
        } else {
            bytes.len()
        };
    }
    bytes.len()
}

/// The length of `bytes` without a high surrogate that a cut at their end left unpaired.
fn whole_utf16(bytes: &[u8], encoding: Encoding) -> usize {
    match bytes.as_chunks::<2>().0.last() {
        Some(unit) if (0xD800..=0xDBFF).contains(&code_unit(*unit, encoding)) => bytes.len() - 2,
        _ => bytes.len(),
    }
}

fn code_unit(unit: [u8; 2], encoding: Encoding) -> u16 {
    if encoding == Encoding::Utf16Be {
        u16::from_be_bytes(unit)
    } else {
        u16::from_le_bytes(unit)
    }
}

/// `bytes` as UTF-8 text, and how many invalid byte sequences became U+FFFD.
fn decode_utf8(bytes: &[u8]) -> (String, usize) {
    let mut text = String::with_capacity(bytes.len());
    let mut invalid = 0;
    for chunk in bytes.utf8_chunks() {
        text.push_str(chunk.valid());
        if !chunk.invalid().is_empty() {
            text.push(char::REPLACEMENT_CHARACTER);
            invalid += 1;
        }
    }
    (text, invalid)
}

/// `bytes` as UTF-16 text, and how many unpaired surrogates became U+FFFD.
fn decode_utf16(bytes: &[u8], encoding: Encoding) -> (String, usize) {
    let units = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|unit| code_unit(*unit, encoding));
    let mut invalid = 0;
    let text = char::decode_utf16(units)
        .map(|c| {
            c.unwrap_or_else(|_| {
                invalid += 1;
                char::REPLACEMENT_CHARACTER
            })
        })
        .collect();
    (text, invalid)
}

/// Read until `buffer` is full or the input ends: the bytes read, and the error that ended the
/// reading early.
fn fill(reader: &mut impl Read, buffer: &mut [u8]) -> (usize, Option<io::Error>) {
    let mut filled = 0;
    while filled < buffer.len() {
        match reader.read(&mut buffer[filled..]) {
            Ok(0) => break,
            Ok(count) => filled += count,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return (filled, Some(e)),
        }
    }
    (filled, None)
}

/// Read a history file `chunk` bytes at a time, giving `emit` each line when it is complete.
/// A read error ends the file with what was read (and a problem); only an error of `emit`,
/// which writes the line, is returned as an error.
fn read_history(
    mut reader: impl Read,
    chunk: usize,
    mut emit: impl FnMut(Line) -> io::Result<()>,
) -> io::Result<Outcome> {
    // A byte order mark is up to three bytes, whatever the size of a read.
    let mut buffer = vec![0u8; chunk.max(3)];
    let (mut filled, mut error) = fill(&mut reader, &mut buffer);
    let (encoding, mark) = Encoding::detect(&buffer[..filled]);
    let mut splitter = Splitter::new(encoding);
    let mut offset = filled as u64;
    splitter.push(&buffer[mark..filled], &mut emit)?;
    while error.is_none() && filled == buffer.len() {
        (filled, error) = fill(&mut reader, &mut buffer);
        offset += filled as u64;
        splitter.push(&buffer[..filled], &mut emit)?;
    }
    let read_error = error.map(|e| format!("read error after {offset} bytes: {e}"));
    let problems = splitter.end(read_error, &mut emit)?;
    Ok(Outcome {
        empty_lines: splitter.empty_lines,
        problems,
    })
}

// --- output ---------------------------------------------------------------------------------

const CSV_HEADER: &[&str] = &[
    "SourceFile",
    "User",
    "ShellHost",
    "LineNumber",
    "EntryStartLine",
    "Command",
    "FileLastWriteTime",
    "FileSize",
    "Encoding",
    "Problems",
];

/// What every row of one history file shares.
struct FileInfo<'a> {
    source: &'a Source,
    /// The file's own modification time (RFC 3339, UTC), or empty when it is not known.
    last_write: &'a str,
    size: u64,
}

/// Characters a spreadsheet takes as the start of a formula, with the full-width forms that
/// East Asian versions accept.
const FORMULA_STARTS: [char; 8] = ['=', '+', '-', '@', '＝', '＋', '－', '＠'];

/// Characters a spreadsheet may skip before deciding that a cell is a formula.
fn skipped_by_spreadsheets(c: char) -> bool {
    c.is_whitespace()
        || c.is_control()
        || matches!(
            c,
            '\u{00AD}' | '\u{200B}'..='\u{200F}' | '\u{2060}' | '\u{FEFF}'
        )
}

/// A CSV cell that a spreadsheet shows as text. A command line is whatever was typed, and a
/// path or user name whatever the evidence holds: a cell that starts with `=`, `+`, `-` or `@`
/// (after any spaces, tabs or control characters), or with a control character such as a tab or
/// carriage return, would be run as a formula by a spreadsheet opening the CSV. Such a cell gets
/// a `'` in front. JSON and JSONL always hold the text unchanged.
pub(crate) fn csv_cell(text: &str) -> Cow<'_, str> {
    let Some(first) = text.chars().next() else {
        return Cow::Borrowed(text);
    };
    let formula = text
        .trim_start_matches(skipped_by_spreadsheets)
        .starts_with(FORMULA_STARTS);
    if formula || first.is_control() {
        Cow::Owned(format!("'{text}"))
    } else {
        Cow::Borrowed(text)
    }
}

/// Where the rows go, written one at a time.
enum Records {
    Csv(Box<csv::Writer<Box<dyn Write>>>),
    /// Pretty JSON objects when `true`, else one object per line.
    Json(Box<dyn Write>, bool),
}

impl Records {
    /// A CSV gets its header now, so that it has one also without rows.
    fn new(out: Box<dyn Write>, output_type: PowerShellHistoryOutputType) -> io::Result<Self> {
        Ok(match output_type {
            PowerShellHistoryOutputType::Csv => {
                let mut csv = csv::Writer::from_writer(out);
                csv.write_record(crate::output_schema::labels(CSV_HEADER.iter().copied())?)?;
                Records::Csv(Box::new(csv))
            }
            PowerShellHistoryOutputType::Json => Records::Json(out, true),
            PowerShellHistoryOutputType::Jsonl => Records::Json(out, false),
        })
    }

    fn write(&mut self, file: &FileInfo<'_>, line: &Line) -> io::Result<()> {
        match self {
            Records::Csv(csv) => {
                let row = csv_row(file, line);
                csv.write_record(row.iter().map(|cell| csv_cell(cell).into_owned()))?;
                Ok(())
            }
            Records::Json(out, pretty) => {
                crate::output_schema::write_json(&mut *out, &json_record(file, line), *pretty)?;
                out.write_all(b"\n")
            }
        }
    }

    fn finish(self) -> io::Result<()> {
        match self {
            Records::Csv(mut csv) => csv.flush(),
            Records::Json(mut out, _) => out.flush(),
        }
    }
}

fn csv_row(file: &FileInfo<'_>, line: &Line) -> Vec<String> {
    vec![
        file.source.shown.clone(),
        file.source.user.clone().unwrap_or_default(),
        file.source.host.clone(),
        line.number.to_string(),
        line.entry_start.to_string(),
        line.text.clone(),
        file.last_write.to_owned(),
        file.size.to_string(),
        line.encoding.name().to_owned(),
        line.problems.join("; "),
    ]
}

fn json_record(file: &FileInfo<'_>, line: &Line) -> serde_json::Value {
    json!({
        "SourceFile": file.source.shown,
        "User": file.source.user,
        "ShellHost": (!file.source.host.is_empty()).then_some(&file.source.host),
        "LineNumber": line.number,
        "EntryStartLine": line.entry_start,
        "Command": line.text,
        "FileLastWriteTime": (!file.last_write.is_empty()).then_some(file.last_write),
        "FileSize": file.size,
        "Encoding": line.encoding.name(),
        "Problems": line.problems,
    })
}

/// What the summary counts. Nothing here grows with the size of a history file.
#[derive(Default)]
struct Stats {
    files: usize,
    unreadable: usize,
    lines: usize,
    /// Commands: lines that do not continue the line before them.
    entries: usize,
    empty_lines: u64,
    with_problems: usize,
    by_user: BTreeMap<String, usize>,
}

fn print_summary(stats: &Stats, no_color: bool) {
    p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
    let mut files = fmt_num(stats.files);
    if stats.unreadable > 0 {
        files.push_str(&format!(" (could not read: {})", fmt_num(stats.unreadable)));
    }
    p_label(no_color, "History files read: ", &files);
    p_label(
        no_color,
        "Command lines: ",
        &format!(
            "{} (commands: {}, continuation lines: {})",
            fmt_num(stats.lines),
            fmt_num(stats.entries),
            fmt_num(stats.lines - stats.entries)
        ),
    );
    if stats.empty_lines > 0 {
        p_label(
            no_color,
            "Empty lines skipped: ",
            &fmt_num(usize::try_from(stats.empty_lines).unwrap_or(usize::MAX)),
        );
    }
    let mut shown: Vec<String> = stats
        .by_user
        .iter()
        .take(5)
        .map(|(user, lines)| format!("{user}: {}", fmt_num(*lines)))
        .collect();
    if stats.by_user.len() > shown.len() {
        shown.push("…".into());
    }
    if !shown.is_empty() {
        p_label(no_color, "Lines by user: ", &shown.join(", "));
    }
    p_label(no_color, "With problems: ", &fmt_num(stats.with_problems));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines_of(bytes: &[u8], chunk: usize) -> (Vec<Line>, Outcome) {
        let mut lines = Vec::new();
        let outcome = read_history(bytes, chunk, |line| {
            lines.push(line);
            Ok(())
        })
        .unwrap();
        (lines, outcome)
    }

    /// The numbers, entry starts and texts of the lines, read with several chunk sizes that
    /// must all agree (a chunk can end inside a character, a line end or a byte order mark).
    fn rows(bytes: &[u8]) -> Vec<(u64, u64, String)> {
        let expected = lines_of(bytes, CHUNK_BYTES).0;
        for chunk in [1, 2, 3, 4, 5, 7] {
            assert_eq!(lines_of(bytes, chunk).0, expected, "chunk size {chunk}");
        }
        expected
            .into_iter()
            .map(|line| (line.number, line.entry_start, line.text))
            .collect()
    }

    fn utf16(text: &str, big_endian: bool) -> Vec<u8> {
        let mark: u16 = 0xFEFF;
        std::iter::once(mark)
            .chain(text.encode_utf16())
            .flat_map(|unit| {
                if big_endian {
                    unit.to_be_bytes()
                } else {
                    unit.to_le_bytes()
                }
            })
            .collect()
    }

    fn row(number: u64, entry_start: u64, text: &str) -> (u64, u64, String) {
        (number, entry_start, text.to_owned())
    }

    #[test]
    fn history_names_end_with_history_txt_and_are_in_a_psreadline_folder() {
        for name in [
            "ConsoleHost_history.txt",
            "consolehost_HISTORY.TXT",
            "Visual Studio Code Host_history.txt",
            "Windows PowerShell ISE Host_history.txt",
            "ホスト_history.txt",
        ] {
            assert!(is_history_name(name), "{name}");
        }
        for name in [
            "_history.txt",
            "history.txt",
            "ConsoleHost_history.txt.bak",
            "ConsoleHost_history",
            "é",
            "",
        ] {
            assert!(!is_history_name(name), "{name}");
        }
        assert_eq!(
            host_of("Visual Studio Code Host_history.txt"),
            Some("Visual Studio Code Host")
        );
        assert!(is_history_path(Path::new(
            "C/Users/a/AppData/Roaming/Microsoft/Windows/PowerShell/PSReadLine/ConsoleHost_history.txt"
        )));
        assert!(is_history_path(Path::new(
            "psreadline/ConsoleHost_history.txt"
        )));
        assert!(!is_history_path(Path::new("ConsoleHost_history.txt")));
        assert!(!is_history_path(Path::new(
            "Users/a/Documents/bash_history.txt"
        )));
        assert!(!is_history_path(Path::new(
            "PSReadLine/backup/ConsoleHost_history.txt"
        )));
    }

    #[test]
    fn users_come_from_the_profile_below_the_input_and_paths_use_forward_slashes() {
        let root = Path::new("/cases/Users/analyst/evidence");
        let tail = "AppData/Roaming/Microsoft/Windows/PowerShell/PSReadLine";
        let source = Source::under(
            root,
            root.join("HOST/C/Users/alice")
                .join(tail)
                .join("ConsoleHost_history.txt"),
        );
        assert_eq!(
            source.shown,
            format!("HOST/C/Users/alice/{tail}/ConsoleHost_history.txt")
        );
        assert_eq!(source.user.as_deref(), Some("alice"));
        assert_eq!(source.host, "ConsoleHost");

        // The system account's profile is not under Users.
        let system = Source::under(
            root,
            root.join("C/Windows/System32/config/systemprofile")
                .join(tail)
                .join("ConsoleHost_history.txt"),
        );
        assert_eq!(system.user.as_deref(), Some("systemprofile"));

        // A redirected AppData: the folder after Users is still the user.
        let moved = Source::under(
            root,
            root.join("D/Users/bob/Roaming/PSReadLine/Visual Studio Code Host_history.txt"),
        );
        assert_eq!(moved.user.as_deref(), Some("bob"));
        assert_eq!(moved.host, "Visual Studio Code Host");

        // PowerShell on Linux.
        let unix = Source::under(
            root,
            root.join("home/carol/.local/share/powershell/PSReadLine/ConsoleHost_history.txt"),
        );
        assert_eq!(unix.user.as_deref(), Some("carol"));

        // Nothing below the input names a user: the analyst's own folders never do.
        let bare = Source::under(root, root.join("PSReadLine/ConsoleHost_history.txt"));
        assert_eq!(bare.user, None);
        assert_eq!(bare.shown, "PSReadLine/ConsoleHost_history.txt");
        let profile = Path::new("/cases/Users/analyst");
        let inside = Source::under(profile, profile.join(tail).join("ConsoleHost_history.txt"));
        assert_eq!(inside.user, None);
        assert!(!inside.shown.contains("analyst"));
    }

    #[test]
    fn windows_separators_are_normalized() {
        // A Windows path kept in one name, as a copy made without its folders can have it.
        let name = r"C\Users\dave\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt";
        assert!(is_history_name(name));
        let root = Path::new("evidence");
        assert!(is_history_path(&root.join(name)));
        let source = Source::under(root, root.join(name));
        assert_eq!(
            source.shown,
            "C/Users/dave/AppData/Roaming/Microsoft/Windows/PowerShell/PSReadLine/ConsoleHost_history.txt"
        );
        assert_eq!(source.user.as_deref(), Some("dave"));
        assert_eq!(source.host, "ConsoleHost");
        assert!(!source.shown.contains('\\'));
    }

    #[test]
    fn a_given_file_is_shown_by_name_and_only_a_profile_layout_names_its_user() {
        let given = Source::given(PathBuf::from(
            "/Users/analyst/cases/export/ConsoleHost_history.txt",
        ));
        assert_eq!(given.shown, "ConsoleHost_history.txt");
        assert_eq!(given.user, None);
        assert_eq!(given.host, "ConsoleHost");

        let kept = Source::given(PathBuf::from(
            "mnt/Users/erin/AppData/Roaming/Microsoft/Windows/PowerShell/PSReadLine/ConsoleHost_history.txt",
        ));
        assert_eq!(kept.shown, "ConsoleHost_history.txt");
        assert_eq!(kept.user.as_deref(), Some("erin"));

        let renamed = Source::given(PathBuf::from("notes.txt"));
        assert_eq!((renamed.host.as_str(), renamed.user), ("", None));
    }

    #[test]
    fn every_line_keeps_its_number_and_its_text() {
        assert_eq!(
            rows(b"whoami\r\nGet-ChildItem -Force\nipconfig /all"),
            [
                row(1, 1, "whoami"),
                row(2, 2, "Get-ChildItem -Force"),
                row(3, 3, "ipconfig /all"),
            ]
        );
        // Leading and trailing spaces, tabs and a lone carriage return inside a line stay.
        assert_eq!(
            rows(b"  cd ..  \n\tls\ra\n \n"),
            [row(1, 1, "  cd ..  "), row(2, 2, "\tls\ra"), row(3, 3, " ")]
        );
        // The last line needs no line end; one after it adds no line.
        assert_eq!(rows(b"a\n"), [row(1, 1, "a")]);
        assert_eq!(rows(b"a\r\n"), [row(1, 1, "a")]);
        assert_eq!(rows(b"a\r"), [row(1, 1, "a")]);
    }

    #[test]
    fn continuation_lines_stay_separate_and_name_the_line_their_command_starts_on() {
        let history = b"Get-Process |`\n  Where-Object CPU -gt 1 |`\n  Sort-Object CPU\nwhoami\n\
            echo `\n\nhostname\nlast`";
        assert_eq!(
            rows(history),
            [
                row(1, 1, "Get-Process |`"),
                row(2, 1, "  Where-Object CPU -gt 1 |`"),
                row(3, 1, "  Sort-Object CPU"),
                row(4, 4, "whoami"),
                row(5, 5, "echo `"),
                // Line 6 is empty: it ends the command, and is counted but not given.
                row(7, 7, "hostname"),
                row(8, 8, "last`"),
            ]
        );
        // An empty line of a command is saved as a lone backtick, and continues it.
        assert_eq!(
            rows(b"if ($x) {`\n`\n}\n"),
            [row(1, 1, "if ($x) {`"), row(2, 1, "`"), row(3, 1, "}")]
        );
    }

    #[test]
    fn empty_lines_are_counted_in_the_numbers_but_are_not_rows() {
        let (lines, outcome) = lines_of(b"\n\r\na\n\n\nb\n\n", 3);
        assert_eq!(
            lines
                .iter()
                .map(|l| (l.number, l.text.as_str()))
                .collect::<Vec<_>>(),
            [(3, "a"), (6, "b")]
        );
        assert_eq!(outcome.empty_lines, 5);
        assert!(outcome.problems.is_empty());
    }

    #[test]
    fn empty_files_and_lone_byte_order_marks_have_no_lines() {
        for bytes in [
            &b""[..],
            &[0xEF, 0xBB, 0xBF],
            &[0xFF, 0xFE],
            &[0xFE, 0xFF],
            b"\n",
            b"\r\n",
            &[0xFF, 0xFE, b'\n', 0],
        ] {
            for chunk in [1, 2, 3, 4, 64] {
                let (lines, outcome) = lines_of(bytes, chunk);
                assert!(lines.is_empty(), "{bytes:?} {lines:?}");
                assert!(outcome.problems.is_empty(), "{bytes:?}");
            }
        }
    }

    #[test]
    fn byte_order_marks_select_the_encoding_and_are_not_text() {
        let text = "echo 'héllo wörld' # 日本語 😀\r\nnext`\r\nline";
        let expected = [
            row(1, 1, "echo 'héllo wörld' # 日本語 😀"),
            row(2, 2, "next`"),
            row(3, 2, "line"),
        ];
        assert_eq!(rows(text.as_bytes()), expected);
        let mut marked = vec![0xEF, 0xBB, 0xBF];
        marked.extend(text.as_bytes());
        assert_eq!(rows(&marked), expected);
        assert_eq!(rows(&utf16(text, false)), expected);
        assert_eq!(rows(&utf16(text, true)), expected);

        for (bytes, encoding) in [
            (text.as_bytes().to_vec(), Encoding::Utf8),
            (marked, Encoding::Utf8Bom),
            (utf16(text, false), Encoding::Utf16Le),
            (utf16(text, true), Encoding::Utf16Be),
        ] {
            let (lines, outcome) = lines_of(&bytes, 5);
            assert!(lines.iter().all(|line| line.encoding == encoding));
            assert!(lines.iter().all(|line| line.problems.is_empty()));
            assert!(outcome.problems.is_empty());
        }
        // A mark only starts a file: later it is a character of the line.
        assert_eq!(
            rows("a\n\u{feff}b".as_bytes()),
            [row(1, 1, "a"), row(2, 2, "\u{feff}b")]
        );
    }

    #[test]
    fn truncated_byte_order_marks_are_damaged_utf8_not_another_encoding() {
        for (bytes, text) in [
            (&[0xEF][..], "\u{fffd}"),
            (&[0xEF, 0xBB], "\u{fffd}"),
            (&[0xFF], "\u{fffd}"),
            (&[0xFE], "\u{fffd}"),
            (&[0xEF, 0xBB, b'a'], "\u{fffd}a"),
        ] {
            for chunk in [1, 2, 3, 64] {
                let (lines, outcome) = lines_of(bytes, chunk);
                assert_eq!(lines.len(), 1, "{bytes:?}");
                assert_eq!(lines[0].text, text, "{bytes:?}");
                assert_eq!(lines[0].encoding, Encoding::Utf8);
                assert_eq!(
                    lines[0].problems,
                    ["1 invalid UTF-8 byte sequence(s) shown as U+FFFD"]
                );
                assert!(outcome.problems.is_empty());
            }
        }
    }

    #[test]
    fn damaged_utf8_is_read_lossily_and_reported_on_its_line() {
        let (lines, outcome) = lines_of(b"ok\nbad \xff\xfe bytes \xc3\nfine\ncut \xe6\x97", 4);
        assert!(outcome.problems.is_empty());
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[1].text, "bad \u{fffd}\u{fffd} bytes \u{fffd}");
        assert_eq!(
            lines[1].problems,
            ["3 invalid UTF-8 byte sequence(s) shown as U+FFFD"]
        );
        assert!(lines[0].problems.is_empty() && lines[2].problems.is_empty());
        // A character cut off by the end of the file.
        assert_eq!(lines[3].text, "cut \u{fffd}");
        assert_eq!(lines[3].problems.len(), 1);

        // UTF-16 without a byte order mark is not guessed: its NULs are reported.
        let unmarked: Vec<u8> = "dir\n".encode_utf16().flat_map(u16::to_le_bytes).collect();
        let (lines, _) = lines_of(&unmarked, 64);
        assert_eq!(lines[0].text, "d\0i\0r\0");
        assert!(lines[0].problems[0].starts_with("3 NUL character(s): the file may be UTF-16"));
        assert_eq!(lines[1].text, "\0");
    }

    #[test]
    fn damaged_utf16_is_read_lossily_and_reported() {
        // An unpaired surrogate, and a file that ends in the middle of a code unit.
        let mut bytes = vec![0xFF, 0xFE];
        for unit in [0x0061u16, 0xD800, 0x0062, 0x000A, 0x0063] {
            bytes.extend(unit.to_le_bytes());
        }
        bytes.push(0x64);
        for chunk in [1, 2, 3, 5, 64] {
            let (lines, outcome) = lines_of(&bytes, chunk);
            assert_eq!(lines.len(), 2);
            assert_eq!(lines[0].text, "a\u{fffd}b");
            assert_eq!(
                lines[0].problems,
                ["1 unpaired UTF-16 surrogate(s) shown as U+FFFD"]
            );
            assert_eq!(lines[1].text, "c\u{fffd}");
            assert_eq!(lines[1].problems.len(), 1);
            assert!(lines[1].problems[0].contains("half a UTF-16 code unit"));
            assert_eq!(outcome.problems.len(), 1);
        }
        // Only the half unit after the mark, and one after a complete last line.
        let (lines, outcome) = lines_of(&[0xFE, 0xFF, 0x00], 64);
        assert_eq!(lines.len(), 1);
        assert_eq!((lines[0].number, lines[0].text.as_str()), (1, "\u{fffd}"));
        assert_eq!(outcome.problems.len(), 1);
        let (lines, _) = lines_of(&[0xFF, 0xFE, b'a', 0, b'\n', 0, 0x41], 64);
        assert_eq!(
            lines
                .iter()
                .map(|l| (l.number, l.text.as_str()))
                .collect::<Vec<_>>(),
            [(1, "a"), (2, "\u{fffd}")]
        );
    }

    /// Gives `data`, then fails.
    struct Failing<'a>(&'a [u8]);

    impl Read for Failing<'_> {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if self.0.is_empty() {
                return Err(io::Error::other("device not ready"));
            }
            let count = buffer.len().min(self.0.len()).min(4);
            buffer[..count].copy_from_slice(&self.0[..count]);
            self.0 = &self.0[count..];
            Ok(count)
        }
    }

    #[test]
    fn a_read_error_keeps_the_lines_read_and_marks_where_the_file_stops() {
        let mut lines = Vec::new();
        let outcome = read_history(Failing(b"first\nsecond\nthi"), 8, |line| {
            lines.push(line);
            Ok(())
        })
        .unwrap();
        assert_eq!(
            lines
                .iter()
                .map(|l| (l.number, l.text.as_str()))
                .collect::<Vec<_>>(),
            [(1, "first"), (2, "second"), (3, "thi")]
        );
        assert!(lines[0].problems.is_empty() && lines[1].problems.is_empty());
        assert_eq!(
            outcome.problems,
            ["read error after 16 bytes: device not ready"]
        );
        assert_eq!(
            lines[2].problems,
            [
                "read error after 16 bytes: device not ready; this line may be incomplete and the lines after it are missing"
            ]
        );

        // Between two lines, and before anything was read: a row still says where it stopped.
        for (data, number) in [(&b"first\n"[..], 2), (b"", 1)] {
            let mut lines = Vec::new();
            let outcome = read_history(Failing(data), 8, |line| {
                lines.push(line);
                Ok(())
            })
            .unwrap();
            let last = lines.last().unwrap();
            assert_eq!((last.number, last.text.as_str()), (number, ""));
            assert!(last.problems[0].starts_with("read error after"));
            assert_eq!(outcome.problems.len(), 1);
        }
    }

    #[test]
    fn a_write_error_stops_the_reading_and_is_not_a_read_problem() {
        let mut seen = 0;
        let error = read_history(&b"one\ntwo\nthree\n"[..], 64, |_| {
            seen += 1;
            if seen == 2 {
                Err(io::Error::new(io::ErrorKind::StorageFull, "disk full"))
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::StorageFull);
        assert_eq!(seen, 2);
    }

    #[test]
    fn very_long_lines_are_cut_with_a_problem_and_whole_characters() {
        // The cut falls inside the three bytes of the last character kept.
        let mut history = vec![b'a'; MAX_LINE_BYTES - 1];
        history.extend("日本".as_bytes());
        history.extend(b"\nnext\n");
        let (lines, outcome) = lines_of(&history, CHUNK_BYTES);
        assert!(outcome.problems.is_empty());
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text.len(), MAX_LINE_BYTES - 1);
        assert!(lines[0].text.bytes().all(|b| b == b'a'));
        assert_eq!(
            lines[0].problems,
            [format!(
                "the line is longer than {MAX_LINE_BYTES} bytes: its last 6 bytes are not shown"
            )]
        );
        assert_eq!((lines[1].number, lines[1].text.as_str()), (2, "next"));

        // UTF-16: the cut does not leave half of a surrogate pair.
        let mut text = "b".repeat(MAX_LINE_BYTES / 2 - 1);
        text.push('😀');
        let (lines, _) = lines_of(&utf16(&text, false), CHUNK_BYTES);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text.len(), MAX_LINE_BYTES / 2 - 1);
        assert_eq!(
            lines[0].problems,
            [format!(
                "the line is longer than {MAX_LINE_BYTES} bytes: its last 4 bytes are not shown"
            )]
        );
    }

    #[test]
    fn cells_a_spreadsheet_would_run_get_an_apostrophe() {
        for text in [
            "=cmd|' /C calc'!A0",
            "+1+1",
            "-2+3+cmd|' /C calc'!A0",
            "@SUM(1+1)*cmd|' /C calc'!A0",
            "-Path C:\\Temp `",
            " =1+1",
            "\t=1+1",
            "\r\n=1+1",
            "\u{a0}\u{3000}@x",
            "\u{feff}=x",
            "\u{200b}+x",
            "\0=x",
            "＝1+1",
            "＋1",
            "－1",
            "＠x",
            "\tdir",
            "\rdir",
            "\ndir",
            "\u{1b}[31m",
            "\0",
        ] {
            assert_eq!(csv_cell(text), format!("'{text}"), "{text:?}");
        }
        for text in [
            "",
            "whoami",
            "  cd ..",
            "'=already quoted",
            "a=b",
            "Get-Content -Path x",
            "1+1",
            "$x = 1",
            "& cmd /c x",
            "|=",
            "`",
            "\u{fffd}",
            "日本語",
        ] {
            assert!(matches!(csv_cell(text), Cow::Borrowed(_)), "{text:?}");
        }
    }

    #[test]
    fn csv_guards_every_text_cell_and_json_keeps_the_original() {
        let source = Source::under(
            Path::new("in"),
            PathBuf::from(
                "in/=evil/Users/@admin/AppData/Roaming/Microsoft/Windows/PowerShell/PSReadLine/+calc_history.txt",
            ),
        );
        let file = FileInfo {
            source: &source,
            last_write: "2026-10-20T08:46:38.5Z",
            size: 12,
        };
        let line = Line {
            number: 7,
            entry_start: 6,
            text: "=HYPERLINK(\"http://example.test\",\"x\")".into(),
            encoding: Encoding::Utf8,
            problems: vec!["-odd".into(), "second".into()],
        };
        let mut out = Vec::new();
        {
            let mut csv = csv::Writer::from_writer(&mut out);
            let row = csv_row(&file, &line);
            csv.write_record(row.iter().map(|cell| csv_cell(cell).into_owned()))
                .unwrap();
            csv.flush().unwrap();
        }
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(out.as_slice());
        let row = reader.records().next().unwrap().unwrap();
        assert_eq!(row.len(), CSV_HEADER.len());
        let cells: Vec<&str> = row.iter().collect();
        assert_eq!(
            cells,
            [
                "'=evil/Users/@admin/AppData/Roaming/Microsoft/Windows/PowerShell/PSReadLine/+calc_history.txt",
                "'@admin",
                "'+calc",
                "7",
                "6",
                "'=HYPERLINK(\"http://example.test\",\"x\")",
                "2026-10-20T08:46:38.5Z",
                "12",
                "UTF-8",
                "'-odd; second",
            ]
        );

        let json = crate::output_schema::to_value(&json_record(&file, &line)).unwrap();
        assert_eq!(json["Command"], line.text);
        assert_eq!(json["User"], "@admin");
        assert_eq!(json["ShellHost"], "+calc");
        assert_eq!(json["SourceFile"], source.shown);
        assert_eq!(json["LineNumber"], 7);
        assert_eq!(json["EntryStartLine"], 6);
        assert_eq!(json["FileLastWriteTime"], "2026-10-20T08:46:38.5Z");
        assert_eq!(json["Problems"], json!(["-odd", "second"]));
    }

    #[test]
    fn field_names_are_the_same_in_csv_and_json() {
        let source = Source::given(PathBuf::from("notes.txt"));
        let file = FileInfo {
            source: &source,
            last_write: "",
            size: 0,
        };
        let line = Line {
            number: 1,
            entry_start: 1,
            text: "x".into(),
            encoding: Encoding::Utf16Be,
            problems: Vec::new(),
        };
        let json = crate::output_schema::to_value(&json_record(&file, &line)).unwrap();
        let object = json.as_object().unwrap();
        assert_eq!(object.len(), CSV_HEADER.len());
        for name in crate::output_schema::labels(CSV_HEADER.iter().copied()).unwrap() {
            assert!(object.contains_key(name), "{name}");
        }
        // What is not known is null, not an empty or invented value.
        assert!(json["User"].is_null());
        assert!(json["ShellHost"].is_null());
        assert!(json["FileLastWriteTime"].is_null());
        assert_eq!(json["Encoding"], "UTF-16BE");
        assert_eq!(csv_row(&file, &line)[6], "");
    }
}
