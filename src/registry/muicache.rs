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

//! `parse-registry muicache`: the programs users ran, with the names and companies Windows read
//! from their version resources, from `MuiCache` in `UsrClass.dat` (Windows Vista and later) and
//! `NTUSER.DAT` (Windows XP) hives, read by ys-registry-parser's `ntuser` module, with Hayabusa's
//! input options, progress bar and results summary. One record per program: its path, its
//! friendly app name and its company. There is no time per program: the key's last-write time
//! dates only its latest change. The summary lists programs outside the Windows and Program Files
//! folders.

use super::persistence::profile_of_classes_path;
use super::{
    Hosts, Opened, Profiles, RecordWriter, choose_logs, files_read, filetime, find_hives,
    hive_type_of_file, input_files, open, profile_of_path, sink,
};
use crate::cli::RegistryMuiCacheOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::ntuser::{MuiCacheEntry, classes_sid, mui_cache, profile_name};
use serde::Serialize;
use std::collections::BTreeSet;
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems and programs listed in the summary.
const MAX_PRINTED: usize = 20;
/// The hives read, as their base blocks name them.
const HIVE_TYPES: [&str; 2] = ["USRCLASS.DAT", "NTUSER.DAT"];
/// The folders, right under a drive, that Windows and installed programs are in.
const USUAL_FOLDERS: [&str; 3] = ["Windows", "Program Files", "Program Files (x86)"];

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryMuiCacheOptions) -> i32 {
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
    let inputs = input_files(
        opts.filepath.as_ref(),
        opts.directory.as_ref(),
        "UsrClass.dat or NTUSER.DAT hives",
        |d| {
            find_hives(d)
                .into_iter()
                .filter(|p| hive_type_of_file(p).is_some_and(|t| HIVE_TYPES.contains(&t.as_str())))
                .collect()
        },
    )
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
        p_label(
            no_color,
            "UsrClass.dat and NTUSER.DAT hives: ",
            &fmt_num(files.len()),
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
            if !opened.hive_type.is_empty() && !HIVE_TYPES.contains(&opened.hive_type.as_str()) {
                return Err(format!(
                    "a {} hive, not a UsrClass.dat or NTUSER.DAT hive",
                    opened.hive_type
                ));
            }
            opened.host = hosts.of(path, &opened);
            let (username, mut sid) = if opened.hive_type == "USRCLASS.DAT" {
                (profile_of_classes_path(path), classes_sid(&opened.hive))
            } else {
                (
                    profile_of_path(path).or_else(|| profile_name(&opened.hive)),
                    None,
                )
            };
            if sid.is_none() {
                sid = username.as_deref().and_then(|u| profiles.sid(path, u));
            }
            Ok((opened, User { username, sid }))
        });
        match parsed {
            Ok((opened, user)) => {
                if let Err(e) = stats.hive(&opened, &user, writer.as_mut()) {
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

/// Whether `path` is in a folder Windows or installed programs are in (`C:\Windows\…`,
/// `D:\Program Files (x86)\…`, `%windir%\…`, `%ProgramFiles%\…`).
fn in_usual_folder(path: &str) -> bool {
    let mut parts = path.split(['\\', '/']);
    let first = parts.next().unwrap_or_default();
    let drive =
        first.len() == 2 && first.as_bytes()[0].is_ascii_alphabetic() && first.ends_with(':');
    let variable = [
        "%windir%",
        "%SystemRoot%",
        "%ProgramFiles%",
        "%ProgramFiles(x86)%",
    ]
    .iter()
    .any(|variable| first.eq_ignore_ascii_case(variable));
    if !drive && !variable {
        return false;
    }

    // Resolve lexical components without touching the examiner's file system. A path that
    // escapes an environment-variable root cannot be assumed to remain in that folder.
    let mut components = Vec::new();
    for part in parts {
        match part {
            "" | "." => {}
            ".." => {
                if components.pop().is_none() && variable {
                    return false;
                }
            }
            _ => components.push(part),
        }
    }
    variable
        || components.first().is_some_and(|folder| {
            USUAL_FOLDERS
                .iter()
                .any(|usual| folder.eq_ignore_ascii_case(usual))
        })
}

/// A hive type as Windows names the file: `UsrClass.dat`, not `USRCLASS.DAT`.
fn hive_file_name(hive_type: &str) -> &str {
    if hive_type == "USRCLASS.DAT" {
        "UsrClass.dat"
    } else {
        hive_type
    }
}

// --- records ----------------------------------------------------------------------------------

/// The user a hive belongs to: its profile folder's name, and its SID (a `UsrClass.dat`'s root
/// key names it; else from its system's `ProfileList`).
struct User {
    username: Option<String>,
    sid: Option<String>,
}

/// A program's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    #[serde(rename = "Host")]
    host: Option<&'a str>,
    #[serde(rename = "Username")]
    username: Option<&'a str>,
    #[serde(rename = "UserSID")]
    user_sid: Option<&'a str>,
    #[serde(rename = "SourcePath")]
    source_path: &'a str,
    #[serde(rename = "HiveType")]
    hive_type: &'a str,
    #[serde(rename = "Path")]
    path: &'a str,
    #[serde(rename = "FriendlyAppName")]
    friendly_app_name: Option<&'a str>,
    #[serde(rename = "ApplicationCompany")]
    application_company: Option<&'a str>,
    #[serde(rename = "KeyPath")]
    key_path: &'a str,
    #[serde(rename = "KeyLastWritten")]
    key_last_written: Option<String>,
    #[serde(rename = "FriendlyAppNameRaw")]
    friendly_app_name_raw: Option<String>,
    #[serde(rename = "ApplicationCompanyRaw")]
    application_company_raw: Option<String>,
    #[serde(rename = "TransactionLogStatus")]
    transaction_log_status: &'static str,
    #[serde(rename = "LogsApplied")]
    logs_applied: &'a [String],
    #[serde(rename = "Problems")]
    problems: &'a [String],
}

const CSV_HEADER: [&str; 15] = [
    "Host",
    "Username",
    "UserSID",
    "SourcePath",
    "HiveType",
    "Path",
    "FriendlyAppName",
    "ApplicationCompany",
    "KeyPath",
    "KeyLastWrittenUTC",
    "FriendlyAppNameRaw",
    "ApplicationCompanyRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

impl<'a> Record<'a> {
    fn new(opened: &'a Opened, user: &'a User, entry: &'a MuiCacheEntry) -> Self {
        Record {
            host: opened.host.as_deref(),
            username: user.username.as_deref(),
            user_sid: user.sid.as_deref(),
            source_path: &opened.source,
            hive_type: hive_file_name(&opened.hive_type),
            path: &entry.path,
            friendly_app_name: entry.friendly_app_name.as_deref(),
            application_company: entry.application_company.as_deref(),
            key_path: entry.key_path,
            key_last_written: filetime(entry.key_last_written),
            friendly_app_name_raw: entry.friendly_app_name_raw.as_deref().map(hex),
            application_company_raw: entry.application_company_raw.as_deref().map(hex),
            transaction_log_status: opened.log_status(entry.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &entry.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        vec![
            text(self.host),
            text(self.username),
            text(self.user_sid),
            self.source_path.to_string(),
            self.hive_type.to_string(),
            self.path.to_string(),
            text(self.friendly_app_name),
            text(self.application_company),
            self.key_path.to_string(),
            text(self.key_last_written.as_deref()),
            text(self.friendly_app_name_raw.as_deref()),
            text(self.application_company_raw.as_deref()),
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
    /// Whether the records go to stdout (`-t` without `-o`): no summary, so what concerns a hive
    /// rather than one program goes to stderr.
    to_stderr: bool,
    hives: usize,
    failed: usize,
    hosts: BTreeSet<String>,
    users: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    without: usize,
    programs: usize,
    resource_strings: usize,
    /// Programs outside the Windows and Program Files folders: (user, path, name).
    elsewhere: Vec<String>,
    elsewhere_count: usize,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened`'s MuiCache programs (when there is a writer) and count them.
    fn hive(
        &mut self,
        opened: &Opened,
        user: &User,
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
        let Some(cache) = mui_cache(&opened.hive) else {
            self.without += 1;
            return Ok(());
        };
        self.resource_strings += cache.resource_strings;
        for problem in &cache.problems {
            if self.to_stderr {
                eprintln!("{}: MuiCache: {problem}", opened.source);
            }
            self.problem(&opened.source, "MuiCache", problem);
        }
        for entry in &cache.entries {
            self.programs += 1;
            for problem in &entry.problems {
                self.problem(
                    &opened.source,
                    &format!(r"{}\{}", entry.key_path, entry.path),
                    problem,
                );
            }
            if !in_usual_folder(&entry.path) {
                self.elsewhere_count += 1;
                if self.elsewhere.len() < MAX_PRINTED {
                    let name = entry
                        .friendly_app_name
                        .as_deref()
                        .map_or_else(String::new, |n| format!(" ({n})"));
                    let who = user.username.as_deref().unwrap_or("?");
                    self.elsewhere.push(format!("{who}  {}{name}", entry.path));
                }
            }
            let record = Record::new(opened, user, entry);
            match writer.as_deref_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv())?,
                Some(w) => w.json(&record)?,
                None => {}
            }
        }
        Ok(())
    }

    fn problem(&mut self, source: &str, at: &str, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED {
            self.problems.push(format!("{source}: {at}: {problem}"));
        }
    }

    fn print(&self, no_color: bool) {
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        p_label(
            no_color,
            "UsrClass.dat and NTUSER.DAT hives parsed: ",
            &fmt_num(self.hives),
        );
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
                "Dirty hives whose transaction logs were not applied (programs may be missing): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if self.without > 0 {
            p_label(
                no_color,
                "Hives without MuiCache (NTUSER.DAT of Windows Vista and later keeps none): ",
                &fmt_num(self.without),
            );
        }
        p_label(no_color, "Programs: ", &fmt_num(self.programs));
        p_label(
            no_color,
            "Resource strings left out (@<file>,-<number>): ",
            &fmt_num(self.resource_strings),
        );
        p_label(
            no_color,
            "Programs outside the Windows and Program Files folders: ",
            &fmt_num(self.elsewhere_count),
        );
        for line in &self.elsewhere {
            p(None, &format!("  {line}"), true);
        }
        if self.elsewhere_count > self.elsewhere.len() {
            p(
                None,
                &format!(
                    "  … {} more (-o <FILE>)",
                    fmt_num(self.elsewhere_count - self.elsewhere.len())
                ),
                true,
            );
        }
        p_label(
            no_color,
            "Programs or keys that could not be read: ",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usual_folders() {
        for path in [
            r"C:\Windows\System32\cmd.exe",
            r"C:\Program Files\Internet Explorer\iexplore.exe",
            r"d:\program files (x86)\Tool\tool.exe",
            r"%windir%\system32\notepad.exe",
            r"%ProgramFiles%\Tool\tool.exe",
            r"C:\Windows\System32\..\notepad.exe",
            r"%windir%\System32\..\notepad.exe",
        ] {
            assert!(in_usual_folder(path), "{path}");
        }
        for path in [
            r"C:\Users\alice\Downloads\setup.exe",
            r"D:\setup64.exe",
            r"C:\ProgramData\x.exe",
            r"\\server\share\tool.exe",
            r"C:\Windows.old\x.exe",
            r"C:\Temp\Windows\x.exe",
            r"Windows\relative.exe",
            r"Program Files\relative.exe",
            r"1:\Windows\x.exe",
            r"C:\Windows\..\Users\alice\payload.exe",
            r"%windir%\..\Temp\payload.exe",
            "",
        ] {
            assert!(!in_usual_folder(path), "{path}");
        }
        assert_eq!(hive_file_name("USRCLASS.DAT"), "UsrClass.dat");
        assert_eq!(hive_file_name("NTUSER.DAT"), "NTUSER.DAT");
    }
}
