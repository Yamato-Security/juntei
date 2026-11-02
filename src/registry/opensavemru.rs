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

//! `parse-registry opensavemru`: the files users opened and saved through the common Open and
//! Save As dialogs, and the programs that showed the dialogs with the folders they used, from the
//! `ComDlg32` key of `NTUSER.DAT` hives, read by ys-registry-parser's `comdlg32` module, each
//! shell item list decoded by ys-lnk-parser (a link's target ID list has the same items), with
//! Hayabusa's input options, progress bar and results summary. One record per entry of
//! `OpenSavePidlMRU` (per extension), `LastVisitedPidlMRU`, `LastVisitedPidlMRULegacy`,
//! `CIDSizeMRU` and `FirstFolder`, and of Windows XP's `OpenSaveMRU` and `LastVisitedMRU`, in the
//! order of its key's list: the program, the path, and the last shell item's type, times and
//! NTFS file reference. A key's last-write time dates its most recent entry only. The summary
//! lists each hive's most recent file and the programs that used the dialogs.

use super::{
    Hosts, Opened, Profiles, RecordWriter, choose_logs, files_read, filetime, find_hives_of_type,
    input_files, open, profile_of_path, sink,
};
use crate::cli::RegistryOpenSaveMruOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use chrono::{DateTime, SecondsFormat, Utc};
use indicatif::ProgressBar;
use registry::comdlg32::{DialogEntry, DialogList, dialogs};
use registry::ntuser::profile_name;
use serde::Serialize;
use shell::item::{self, Item, Kind};
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems, files and programs listed in the summary.
const MAX_PRINTED: usize = 20;

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryOpenSaveMruOptions) -> i32 {
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
    let given_logs: Vec<PathBuf> = opts.log1.iter().chain(&opts.log2).cloned().collect();
    let inputs = if shell::codepage::is_supported(opts.code_page) {
        input_files(
            opts.filepath.as_ref(),
            opts.directory.as_ref(),
            "NTUSER.DAT hives",
            |d| find_hives_of_type(d, "NTUSER.DAT"),
        )
    } else {
        Err(format!("Unsupported code page: {}", opts.code_page))
    }
    .and_then(|files| Ok((files, choose_logs(opts.no_logs, &given_logs)?)))
    .and_then(|(files, logs)| {
        let sink = sink(opts.output.as_ref(), opts.clobber, data_on_stdout, || {
            files_read(&files, &given_logs)
        })?;
        Ok((files, logs, sink))
    });
    let (files, logs, sink) = match inputs {
        Ok(inputs) => inputs,
        Err(msg) => {
            report_error(None, &msg);
            return 1;
        }
    };
    let mut writer = match sink.map(|w| RecordWriter::new(w, opts.output_type(), &CSV_HEADER)) {
        Some(Ok(w)) => Some(w),
        Some(Err(e)) => {
            report_error(None, &format!("Could not write the records: {e}"));
            return 1;
        }
        None => None,
    };
    if !data_on_stdout {
        let total: u64 = files
            .iter()
            .map(|p| p.metadata().map_or(0, |m| m.len()))
            .sum();
        p_label(no_color, "NTUSER.DAT hives: ", &fmt_num(files.len()));
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
    let mut hosts = Hosts::new(!opts.no_logs);
    let mut profiles = Profiles::new(!opts.no_logs);
    let mut stats = Stats {
        to_stderr: data_on_stdout,
        ..Stats::default()
    };
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        let parsed = open(path, logs).and_then(|mut opened| {
            if !opened.hive_type.is_empty() && opened.hive_type != "NTUSER.DAT" {
                return Err(format!(
                    "a {} hive, not an NTUSER.DAT hive",
                    opened.hive_type
                ));
            }
            opened.host = hosts.of(path, &opened);
            let username = profile_of_path(path).or_else(|| profile_name(&opened.hive));
            let sid = username.as_deref().and_then(|u| profiles.sid(path, u));
            Ok((opened, User { username, sid }))
        });
        match parsed {
            Ok((opened, user)) => {
                if let Err(e) = stats.hive(&opened, &user, opts.code_page, writer.as_mut()) {
                    report_error(
                        bar.as_ref(),
                        &format!("Could not write the records of {}: {e}", path.display()),
                    );
                    return 1;
                }
            }
            Err(e) => {
                stats.failed += 1;
                if stats.failed <= MAX_PRINTED_ERRORS {
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
        report_error(None, &format!("Could not write the records: {e}"));
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
    i32::from(stats.hives == 0)
}

// --- shell items ------------------------------------------------------------------------------

/// The user a hive belongs to: its profile folder's name, and its SID from its system's
/// `ProfileList`.
struct User {
    username: Option<String>,
    sid: Option<String>,
}

/// What a shell item is, as the records name it.
fn kind_name(kind: &Kind) -> &'static str {
    match kind {
        Kind::RootFolder => "root_folder",
        Kind::Volume => "volume",
        Kind::Directory => "directory",
        Kind::File => "file",
        Kind::Network => "network_location",
        Kind::ControlPanel => "control_panel_item",
        Kind::ControlPanelCategory => "control_panel_category",
        Kind::UsersFilesFolder => "users_files_folder",
        Kind::Uri => "uri",
        Kind::PropertyView => "property_view",
        Kind::CdBurn => "cd_burn",
        Kind::PortableDevice => "portable_device",
        Kind::PortableDeviceFolder => "portable_device_folder",
        Kind::CompressedFolder => "compressed_folder",
        Kind::FtpFolder => "ftp_folder",
        Kind::Other(_) => "unknown",
    }
}

/// An entry's shell item list, decoded: the path its items make (each item's name, from the
/// desktop down) and its last item, the file or folder.
struct Decoded {
    path: String,
    last: Option<Item>,
    /// Items of a type ys-lnk-parser doesn't know, or without a name: the entry keeps its bytes.
    unknown: bool,
}

/// The items of `list`, decoded and joined into a path. An item without a name is shown by its
/// class (`[class 0x52]`); a network item's leading backslashes are dropped after the first
/// item, as for a link's target.
fn decode_items(list: &[u8], code_page: u16) -> Decoded {
    let mut parts: Vec<String> = Vec::new();
    let mut last = None;
    let mut unknown = false;
    for raw in item::items(list) {
        let decoded = item::decode_child(raw, last.as_ref(), code_page);
        let name = if decoded.name.is_empty() {
            unknown = true;
            format!("[class 0x{:02x}]", decoded.class)
        } else if parts.is_empty() {
            decoded.name.clone()
        } else {
            decoded.name.trim_start_matches('\\').to_string()
        };
        unknown |= matches!(decoded.kind, Kind::Other(_));
        parts.push(name);
        last = Some(decoded);
    }
    Decoded {
        path: parts.join("\\"),
        last,
        unknown,
    }
}

// --- records ----------------------------------------------------------------------------------

/// An entry's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    host: Option<&'a str>,
    username: Option<&'a str>,
    user_sid: Option<&'a str>,
    source_path: &'a str,
    list: &'static str,
    extension: Option<&'a str>,
    mru_position: Option<u32>,
    is_most_recent: bool,
    value_name: &'a str,
    program: Option<&'a str>,
    path: Option<String>,
    item_type: Option<&'static str>,
    item_modified: Option<String>,
    item_created: Option<String>,
    item_accessed: Option<String>,
    mft_entry: Option<u64>,
    mft_sequence: Option<u16>,
    last_used: Option<String>,
    key_path: &'a str,
    key_last_written: Option<String>,
    data_raw: Option<String>,
    transaction_log_status: &'static str,
    logs_applied: &'a [String],
    problems: &'a [String],
}

const CSV_HEADER: [&str; 24] = [
    "Host",
    "Username",
    "UserSid",
    "SourcePath",
    "List",
    "Extension",
    "MruPosition",
    "IsMostRecent",
    "ValueName",
    "Program",
    "Path",
    "ItemType",
    "ItemModifiedUTC",
    "ItemCreatedUTC",
    "ItemAccessedUTC",
    "MftEntry",
    "MftSequence",
    "LastUsedUTC",
    "KeyPath",
    "KeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// A shell item's FAT time (seconds since 1970, as recorded) as RFC 3339.
fn fat_time(secs: Option<i64>) -> Option<String> {
    secs.and_then(|s| DateTime::<Utc>::from_timestamp(s, 0))
        .map(|t| t.to_rfc3339_opts(SecondsFormat::AutoSi, true))
}

impl<'a> Record<'a> {
    fn new(
        opened: &'a Opened,
        user: &'a User,
        entry: &'a DialogEntry,
        decoded: Option<&Decoded>,
    ) -> Self {
        let last = decoded.and_then(|d| d.last.as_ref());
        let written = filetime(entry.key_last_written);
        let most_recent = entry.mru_position == Some(0);
        // the shell items' path, else the path the value holds as text
        let path = decoded
            .map(|d| d.path.clone())
            .or_else(|| entry.path.clone());
        let unknown = decoded.is_some_and(|d| d.unknown);
        Record {
            host: opened.host.as_deref(),
            username: user.username.as_deref(),
            user_sid: user.sid.as_deref(),
            source_path: &opened.source,
            list: entry.list.name(),
            extension: entry.extension.as_deref(),
            mru_position: entry.mru_position,
            is_most_recent: most_recent,
            value_name: &entry.value_name,
            program: entry.program.as_deref(),
            path,
            item_type: last.map(|i| kind_name(&i.kind)),
            item_modified: fat_time(last.and_then(|i| i.modified)),
            item_created: fat_time(last.and_then(|i| i.created)),
            item_accessed: fat_time(last.and_then(|i| i.accessed)),
            mft_entry: last.and_then(|i| i.mft.map(|m| m.0)),
            mft_sequence: last.and_then(|i| i.mft.map(|m| m.1)),
            // the key is written when an entry moves to the front: its time is the most
            // recent entry's
            last_used: most_recent.then(|| written.clone()).flatten(),
            key_path: &entry.key_path,
            key_last_written: written,
            data_raw: entry
                .data_raw
                .as_deref()
                .or_else(|| unknown.then_some(entry.shell_items.as_deref()).flatten())
                .map(hex),
            transaction_log_status: opened.log_status(entry.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &entry.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let num = |v: Option<u64>| v.map(|n| n.to_string()).unwrap_or_default();
        vec![
            text(self.host),
            text(self.username),
            text(self.user_sid),
            self.source_path.to_string(),
            self.list.to_string(),
            text(self.extension),
            num(self.mru_position.map(u64::from)),
            self.is_most_recent.to_string(),
            self.value_name.to_string(),
            text(self.program),
            text(self.path.as_deref()),
            text(self.item_type),
            text(self.item_modified.as_deref()),
            text(self.item_created.as_deref()),
            text(self.item_accessed.as_deref()),
            num(self.mft_entry),
            num(self.mft_sequence.map(u64::from)),
            text(self.last_used.as_deref()),
            self.key_path.to_string(),
            text(self.key_last_written.as_deref()),
            text(self.data_raw.as_deref()),
            self.transaction_log_status.to_string(),
            self.logs_applied.join("|"),
            self.problems.join("; "),
        ]
    }
}

// --- summary ----------------------------------------------------------------------------------

/// Counts for the results summary.
#[derive(Default)]
struct Stats {
    /// Hive/list warnings must remain visible when stdout carries only records.
    to_stderr: bool,
    hives: usize,
    failed: usize,
    hosts: BTreeSet<String>,
    users: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    without: usize,
    entries: usize,
    lists: BTreeMap<&'static str, usize>,
    extensions: BTreeSet<String>,
    /// Programs that used the dialogs, and in how many entries.
    programs: BTreeMap<String, usize>,
    /// Entries whose shell items include one of a type ys-lnk-parser doesn't know.
    unknown_items: usize,
    /// Each hive's most recent file (of every extension), and when it was opened or saved.
    latest: Vec<String>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened`'s dialog entries (when there is a writer) and count them.
    fn hive(
        &mut self,
        opened: &Opened,
        user: &User,
        code_page: u16,
        mut writer: Option<&mut RecordWriter>,
    ) -> io::Result<()> {
        self.hives += 1;
        self.hosts.extend(opened.host.clone());
        self.users.extend(user.username.clone());
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
        let Some((entries, problems)) = dialogs(&opened.hive) else {
            self.without += 1;
            return Ok(());
        };
        for problem in &problems {
            if self.to_stderr {
                eprintln!("{}: {problem}", opened.source);
            }
            self.problem(&opened.source, problem);
        }
        for entry in &entries {
            self.entries += 1;
            *self.lists.entry(entry.list.name()).or_default() += 1;
            if let Some(ext) = &entry.extension {
                self.extensions.insert(ext.to_lowercase());
            }
            if let Some(program) = entry.program.as_deref().filter(|p| !p.is_empty()) {
                // a path (FirstFolder) counts as its file name
                let name = program.rsplit(['\\', '/']).next().unwrap_or(program);
                *self.programs.entry(name.to_lowercase()).or_default() += 1;
            }
            for problem in &entry.problems {
                self.problem(
                    &opened.source,
                    &format!(r"{}\{}: {problem}", entry.key_path, entry.value_name),
                );
            }
            let decoded = entry
                .shell_items
                .as_deref()
                .map(|list| decode_items(list, code_page));
            if decoded.as_ref().is_some_and(|d| d.unknown) {
                self.unknown_items += 1;
            }
            let record = Record::new(opened, user, entry, decoded.as_ref());
            if entry.extension.as_deref() == Some("*")
                && record.is_most_recent
                && self.latest.len() < MAX_PRINTED
            {
                let when = record.key_last_written.as_deref().unwrap_or("?");
                let what = record.path.as_deref().unwrap_or("(no path)");
                self.latest.push(format!(
                    "{}: {what} ({when}, {})",
                    opened.source,
                    entry.list.name()
                ));
            }
            match writer.as_deref_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv())?,
                Some(w) => w.json(&record)?,
                None => {}
            }
        }
        Ok(())
    }

    fn problem(&mut self, source: &str, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED {
            self.problems.push(format!("{source}: {problem}"));
        }
    }

    fn print(&self, no_color: bool) {
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        p_label(no_color, "NTUSER.DAT hives parsed: ", &fmt_num(self.hives));
        if self.failed > 0 {
            p_label(no_color, "Could not parse: ", &fmt_num(self.failed));
        }
        let list = |set: &BTreeSet<String>| set.iter().cloned().collect::<Vec<_>>().join(", ");
        if !self.hosts.is_empty() {
            p_label(no_color, "Hosts: ", &list(&self.hosts));
        }
        if !self.users.is_empty() {
            p_label(no_color, "Users: ", &list(&self.users));
        }
        p_label(
            no_color,
            "Dirty hives recovered with their transaction logs: ",
            &fmt_num(self.recovered),
        );
        if !self.dirty.is_empty() {
            p_label(
                no_color,
                "Dirty hives whose transaction logs were not applied (entries may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if self.without > 0 {
            p_label(no_color, "Hives without ComDlg32: ", &fmt_num(self.without));
        }
        let lists: Vec<String> = DialogList::ALL
            .iter()
            .filter_map(|l| {
                let n = self.lists.get(l.name())?;
                Some(format!("{} {}", l.name(), fmt_num(*n)))
            })
            .collect();
        p_label(
            no_color,
            "Entries: ",
            &if lists.is_empty() {
                fmt_num(self.entries)
            } else {
                format!("{} ({})", fmt_num(self.entries), lists.join(", "))
            },
        );
        if !self.extensions.is_empty() {
            p_label(
                no_color,
                "Extensions: ",
                &fmt_num(self.extensions.iter().filter(|e| *e != "*").count()),
            );
        }
        if !self.programs.is_empty() {
            let mut programs: Vec<(&String, &usize)> = self.programs.iter().collect();
            programs.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            let shown: Vec<String> = programs
                .iter()
                .take(MAX_PRINTED)
                .map(|(name, n)| format!("{name} ({n})"))
                .collect();
            let more = programs.len().saturating_sub(MAX_PRINTED);
            p_label(
                no_color,
                "Programs that used the dialogs: ",
                &if more > 0 {
                    format!("{}, … {} more", shown.join(", "), fmt_num(more))
                } else {
                    shown.join(", ")
                },
            );
        }
        if !self.latest.is_empty() {
            p_label(
                no_color,
                "Most recent file of each hive (opened or saved; UTC): ",
                "",
            );
            for latest in &self.latest {
                p(None, &format!("  {latest}"), true);
            }
        }
        if self.unknown_items > 0 {
            p_label(
                no_color,
                "Entries with shell items of an unknown type (bytes in DataRaw): ",
                &fmt_num(self.unknown_items),
            );
        }
        p_label(
            no_color,
            "Entries or lists that could not be read: ",
            &fmt_num(self.problem_count),
        );
        for problem in &self.problems {
            p(None, &format!("  {problem}"), true);
        }
        if self.problem_count > self.problems.len() {
            p(
                None,
                &format!(
                    "  … {} more in the Problems column (-o <FILE>)",
                    fmt_num(self.problem_count - self.problems.len())
                ),
                true,
            );
        }
        p(None, "", true);
    }
}
