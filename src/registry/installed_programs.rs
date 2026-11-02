// Copyright (C) 2026 Yamato Security
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

//! `parse-registry installed-programs`: the software Windows lists as installed, machine-wide in
//! SOFTWARE hives and per user in NTUSER.DAT hives, read by ys-registry-parser's `installed`
//! module, with Hayabusa's input options, progress bar and results summary. One record per key:
//! `Uninstall` entries (and `Wow6432Node`'s), Windows Installer products, App Paths, WSL
//! distributions and packaged apps, with the key's last-write time beside the install date. A
//! user hive's user comes from its profile folder and its system's `ProfileList`; a package
//! SOFTWARE lists for a user, from the same hive's `ProfileList`. The summary lists what was
//! installed last, what users installed for themselves, and WSL distributions.

use super::{
    Hosts, Opened, Profiles, RecordWriter, choose_logs, files_read, filetime, find_hives,
    hive_type_of_file, input_files, open, profile_of_path, sink,
};
use crate::cli::RegistryInstalledProgramsOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::installed::{self, Program, ProgramSource, Scope};
use registry::ntuser::profile_name;
use registry::software::profile_list;
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems and records listed in the summary.
const MAX_PRINTED: usize = 20;
/// The hives read, as their base blocks name them.
const HIVE_TYPES: [&str; 2] = ["SOFTWARE", "NTUSER.DAT"];

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryInstalledProgramsOptions) -> i32 {
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
        "SOFTWARE or NTUSER.DAT hives",
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
            "SOFTWARE and NTUSER.DAT hives: ",
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
            if !HIVE_TYPES.contains(&opened.hive_type.as_str()) {
                return Err(if opened.hive_type.is_empty() {
                    "not a SOFTWARE or NTUSER.DAT hive (neither its base block nor its file name says which)".to_string()
                } else {
                    format!(
                        "a {} hive, not a SOFTWARE or NTUSER.DAT hive",
                        opened.hive_type
                    )
                });
            }
            opened.host = hosts.of(path, &opened);
            let read = if opened.hive_type == "SOFTWARE" {
                let (programs, problems) = installed::software(&opened.hive);
                let names = profile_list(&opened.hive)
                    .into_iter()
                    .filter_map(|(sid, folder)| {
                        let name = folder
                            .trim_end_matches(['\\', '/'])
                            .rsplit(['\\', '/'])
                            .next()?
                            .to_owned();
                        (!name.is_empty()).then(|| (sid.to_ascii_uppercase(), name))
                    })
                    .collect();
                Read {
                    programs,
                    problems,
                    user: None,
                    names,
                }
            } else {
                let (programs, problems) = installed::ntuser(&opened.hive);
                let username = profile_of_path(path).or_else(|| profile_name(&opened.hive));
                let sid = username.as_deref().and_then(|u| profiles.sid(path, u));
                Read {
                    programs,
                    problems,
                    user: Some(User { username, sid }),
                    names: HashMap::new(),
                }
            };
            Ok((opened, read))
        });
        match parsed {
            Ok((opened, read)) => {
                if let Err(e) = stats.hive(&opened, &read, writer.as_mut()) {
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
    i32::from(stats.hives() == 0)
}

// --- records ----------------------------------------------------------------------------------

/// The user a user hive belongs to: its profile folder's name, and its SID from its system's
/// `ProfileList`.
struct User {
    username: Option<String>,
    sid: Option<String>,
}

/// What one hive lists.
struct Read {
    programs: Vec<Program>,
    problems: Vec<String>,
    /// A user hive's user.
    user: Option<User>,
    /// A SOFTWARE hive's profiles: SID (upper case) → profile folder name, for the users its
    /// packages are listed for.
    names: HashMap<String, String>,
}

impl Read {
    /// The user name and SID of `program`'s record: the user hive's, or the user a package in
    /// SOFTWARE is listed for.
    fn user_of<'a>(&'a self, program: &'a Program) -> (Option<&'a str>, Option<&'a str>) {
        match (&self.user, &program.user_sid) {
            (Some(user), _) => (user.username.as_deref(), user.sid.as_deref()),
            (None, Some(sid)) => (
                self.names
                    .get(&sid.to_ascii_uppercase())
                    .map(String::as_str),
                Some(sid),
            ),
            (None, None) => (None, None),
        }
    }
}

/// Names and values written as a JSON object, in their order.
struct Pairs<'a, T>(&'a [(String, T)], fn(&T) -> String);

impl<T> Serialize for Pairs<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (name, value) in self.0 {
            map.serialize_entry(name, &(self.1)(value))?;
        }
        map.end()
    }
}

impl<T> Pairs<'_, T> {
    /// `name=value` pairs, for CSV.
    fn csv(&self) -> String {
        self.0
            .iter()
            .map(|(name, value)| format!("{name}={}", (self.1)(value)))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

/// A program's record, its fields in this order.
#[derive(serde::Serialize)]
struct Record<'a> {
    host: Option<&'a str>,
    source_path: &'a str,
    hive_type: &'a str,
    username: Option<&'a str>,
    user_sid: Option<&'a str>,
    source: &'static str,
    scope: &'static str,
    name: Option<&'a str>,
    version: Option<&'a str>,
    publisher: Option<&'a str>,
    install_date: Option<&'a str>,
    install_date_raw: Option<&'a str>,
    key_last_written: Option<String>,
    install_location: Option<&'a str>,
    install_source: Option<&'a str>,
    program: Option<&'a str>,
    uninstall_command: Option<&'a str>,
    quiet_uninstall_command: Option<&'a str>,
    size_kb: Option<u32>,
    system_component: Option<bool>,
    windows_installer: Option<bool>,
    product_code: Option<&'a str>,
    architecture: Option<&'a str>,
    wow64: bool,
    details: Pairs<'a, String>,
    key_path: &'a str,
    data_raw: Pairs<'a, Vec<u8>>,
    transaction_log_status: &'static str,
    logs_applied: &'a [String],
    problems: &'a [String],
}

const CSV_HEADER: [&str; 30] = [
    "Host",
    "SourcePath",
    "HiveType",
    "Username",
    "UserSid",
    "Source",
    "Scope",
    "Name",
    "Version",
    "Publisher",
    "InstallDate",
    "InstallDateRaw",
    "KeyLastWrittenUTC",
    "InstallLocation",
    "InstallSource",
    "Program",
    "UninstallCommand",
    "QuietUninstallCommand",
    "SizeKb",
    "SystemComponent",
    "WindowsInstaller",
    "ProductCode",
    "Architecture",
    "Wow64",
    "Details",
    "KeyPath",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

impl<'a> Record<'a> {
    fn new(opened: &'a Opened, read: &'a Read, program: &'a Program) -> Self {
        let (username, user_sid) = read.user_of(program);
        Record {
            host: opened.host.as_deref(),
            source_path: &opened.source,
            hive_type: &opened.hive_type,
            username,
            user_sid,
            source: program.source.name(),
            scope: program.scope.name(),
            name: program.name.as_deref(),
            version: program.version.as_deref(),
            publisher: program.publisher.as_deref(),
            install_date: program.install_date.as_deref(),
            install_date_raw: program.install_date_raw.as_deref(),
            key_last_written: filetime(program.key_last_written),
            install_location: program.install_location.as_deref(),
            install_source: program.install_source.as_deref(),
            program: program.program.as_deref(),
            uninstall_command: program.uninstall_command.as_deref(),
            quiet_uninstall_command: program.quiet_uninstall_command.as_deref(),
            size_kb: program.size_kb,
            system_component: program.system_component,
            windows_installer: program.windows_installer,
            product_code: program.product_code.as_deref(),
            architecture: program.architecture.as_deref(),
            wow64: program.wow64,
            details: Pairs(&program.details, String::clone),
            key_path: &program.key_path,
            data_raw: Pairs(&program.data_raw, |bytes| hex(bytes)),
            transaction_log_status: opened.log_status(program.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &program.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let flag = |b: bool| if b { "True" } else { "False" }.to_string();
        vec![
            text(self.host),
            self.source_path.to_string(),
            self.hive_type.to_string(),
            text(self.username),
            text(self.user_sid),
            self.source.to_string(),
            self.scope.to_string(),
            text(self.name),
            text(self.version),
            text(self.publisher),
            text(self.install_date),
            text(self.install_date_raw),
            text(self.key_last_written.as_deref()),
            text(self.install_location),
            text(self.install_source),
            text(self.program),
            text(self.uninstall_command),
            text(self.quiet_uninstall_command),
            self.size_kb.map(|n| n.to_string()).unwrap_or_default(),
            self.system_component.map(flag).unwrap_or_default(),
            self.windows_installer.map(flag).unwrap_or_default(),
            text(self.product_code),
            text(self.architecture),
            flag(self.wow64),
            self.details.csv(),
            self.key_path.to_string(),
            self.data_raw.csv(),
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
    /// Records are written to stdout; hive-level diagnostics must remain on stderr.
    to_stderr: bool,
    /// Hives parsed, by type.
    types: BTreeMap<&'static str, usize>,
    failed: usize,
    hosts: BTreeSet<String>,
    users: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    records: usize,
    sources: BTreeMap<&'static str, usize>,
    /// `Uninstall` entries with an install date: (date, key time, line).
    latest: Vec<(String, u64, String)>,
    /// What users installed for themselves (their hives' `Uninstall` entries and products).
    per_user: Vec<String>,
    per_user_count: usize,
    /// WSL distributions.
    distributions: Vec<String>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    fn hives(&self) -> usize {
        self.types.values().sum()
    }

    /// Write the records of `read` (when there is a writer) and count them.
    fn hive(
        &mut self,
        opened: &Opened,
        read: &Read,
        mut writer: Option<&mut RecordWriter>,
    ) -> io::Result<()> {
        let kind = HIVE_TYPES
            .iter()
            .find(|t| **t == opened.hive_type)
            .copied()
            .unwrap_or_default();
        *self.types.entry(kind).or_default() += 1;
        self.hosts.extend(opened.host.clone());
        if let Some(user) = read.user.as_ref().and_then(|u| u.username.clone()) {
            self.users.insert(user);
        }
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
        for problem in &read.problems {
            if self.to_stderr {
                eprintln!("{}: {problem}", opened.source);
            }
            self.problem(&opened.source, problem);
        }
        for program in &read.programs {
            let record = Record::new(opened, read, program);
            match writer.as_deref_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv())?,
                Some(w) => w.json(&record)?,
                None => {}
            }
            self.program(opened, &record, program);
        }
        Ok(())
    }

    /// Count a program written.
    fn program(&mut self, opened: &Opened, record: &Record, program: &Program) {
        self.records += 1;
        *self.sources.entry(program.source.name()).or_default() += 1;
        for problem in &program.problems {
            self.problem(&opened.source, &format!("{}: {problem}", program.key_path));
        }
        let host = opened.host.as_deref().unwrap_or(&opened.source);
        let name = program
            .name
            .as_deref()
            .or_else(|| program.key_path.rsplit('\\').next())
            .unwrap_or_default();
        let version = program
            .version
            .as_deref()
            .map_or_else(String::new, |v| format!(" {v}"));
        let user = record.username.or(record.user_sid).unwrap_or("?");
        match program.source {
            ProgramSource::Uninstall => {
                if let Some(date) = &program.install_date {
                    let publisher = program
                        .publisher
                        .as_deref()
                        .map_or_else(String::new, |p| format!(" ({p})"));
                    let scope = if program.scope == Scope::User {
                        format!("  [{user}]")
                    } else {
                        String::new()
                    };
                    self.latest.push((
                        date.clone(),
                        program.key_last_written,
                        format!("{date}  {host}  {name}{version}{publisher}{scope}"),
                    ));
                    if self.latest.len() > MAX_PRINTED * 4 {
                        self.trim_latest();
                    }
                }
            }
            ProgramSource::Wsl => {
                let at = program.install_location.as_deref().unwrap_or("?");
                let wsl = program
                    .version
                    .as_deref()
                    .map_or_else(String::new, |v| format!(" (WSL {v})"));
                self.distributions
                    .push(format!("{host}  {user}  {name}{wsl}  {at}"));
            }
            ProgramSource::MsiProduct | ProgramSource::AppPath | ProgramSource::Package => {}
        }
        if program.scope == Scope::User
            && matches!(
                program.source,
                ProgramSource::Uninstall | ProgramSource::MsiProduct
            )
        {
            self.per_user_count += 1;
            if self.per_user.len() < MAX_PRINTED {
                self.per_user
                    .push(format!("{host}  {user}  {name}{version}"));
            }
        }
    }

    /// The most recent installs, newest first, at most [`MAX_PRINTED`].
    fn trim_latest(&mut self) {
        self.latest.sort_by(|a, b| b.cmp(a));
        self.latest.truncate(MAX_PRINTED);
    }

    fn problem(&mut self, source: &str, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED {
            self.problems.push(format!("{source}: {problem}"));
        }
    }

    fn print(&mut self, no_color: bool) {
        self.trim_latest();
        let list = |items: &BTreeSet<String>| {
            items
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        };
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        let types: Vec<String> = self
            .types
            .iter()
            .map(|(kind, n)| format!("{kind} {}", fmt_num(*n)))
            .collect();
        p_label(
            no_color,
            "Hives parsed: ",
            &format!("{} ({})", fmt_num(self.hives()), types.join(", ")),
        );
        if self.failed > 0 {
            p_label(no_color, "Could not parse: ", &fmt_num(self.failed));
        }
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
                "Dirty hives whose transaction logs were not applied (records may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        let sources: Vec<String> = self
            .sources
            .iter()
            .map(|(source, n)| format!("{source} {}", fmt_num(*n)))
            .collect();
        p_label(
            no_color,
            "Records: ",
            &if sources.is_empty() {
                fmt_num(self.records)
            } else {
                format!("{} ({})", fmt_num(self.records), sources.join(", "))
            },
        );
        if !self.latest.is_empty() {
            p_label(
                no_color,
                "Installed last (InstallDate, host, program): ",
                "",
            );
            for (_, _, line) in &self.latest {
                p(None, &format!("  {line}"), true);
            }
        }
        p_label(
            no_color,
            "Installed by users for themselves (NTUSER.DAT Uninstall entries and Windows Installer products): ",
            &fmt_num(self.per_user_count),
        );
        for line in &self.per_user {
            p(None, &format!("  {line}"), true);
        }
        if !self.distributions.is_empty() {
            p_label(no_color, "WSL distributions: ", "");
            for line in self.distributions.iter().take(MAX_PRINTED) {
                p(None, &format!("  {line}"), true);
            }
        }
        p_label(
            no_color,
            "Records or keys that could not be read: ",
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
    fn pairs_as_json_and_csv() {
        let details = vec![
            ("DisplayIcon".to_owned(), r"C:\a.exe,0".to_owned()),
            ("Inno Setup: User".to_owned(), "alice".to_owned()),
        ];
        let pairs = Pairs(&details, String::clone);
        assert_eq!(
            serde_json::to_string(&pairs).unwrap(),
            r#"{"DisplayIcon":"C:\\a.exe,0","Inno Setup: User":"alice"}"#
        );
        assert_eq!(
            pairs.csv(),
            r"DisplayIcon=C:\a.exe,0; Inno Setup: User=alice"
        );
        let raw = vec![("EstimatedSize".to_owned(), vec![0x31, 0, 0x32, 0])];
        assert_eq!(Pairs(&raw, |b| hex(b)).csv(), "EstimatedSize=31003200");
        assert_eq!(
            serde_json::to_string(&Pairs(&[] as &[(String, String)], String::clone)).unwrap(),
            "{}"
        );
    }
}
