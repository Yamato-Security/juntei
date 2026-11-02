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

//! `parse-registry office`: Microsoft Office in `NTUSER.DAT` hives, read by ys-registry-parser's
//! `office` module, with Hayabusa's input options, progress bar and results summary. One record
//! per document or folder each version's applications opened last (`File MRU`, `Place MRU`, and
//! Microsoft 365 accounts' `User MRU`), with the time it was last opened; per document whose
//! editing or macros the user enabled (`Trusted Documents\TrustRecords`), with when it was
//! trusted and whether its macros were enabled; and per macro setting (`VBAWarnings`,
//! `BlockContentExecutionFromInternet`, `AccessVBOM`) and trusted location, the user's and those
//! set by policy. The summary lists the documents whose macros were enabled, the settings that
//! let macros run, trusted locations users added, and the documents opened last.

use super::{
    Hosts, Opened, Profiles, RecordWriter, choose_logs, files_read, filetime, find_hives_of_type,
    input_files, open, profile_of_path, sink,
};
use crate::cli::RegistryOfficeOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::ntuser::profile_name;
use registry::office::{OfficeEntry, OfficeList, office};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems, documents, settings and locations listed in the summary.
const MAX_PRINTED: usize = 20;

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryOfficeOptions) -> i32 {
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
        "NTUSER.DAT hives",
        |d| find_hives_of_type(d, "NTUSER.DAT"),
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

// --- records ----------------------------------------------------------------------------------

/// The user a hive belongs to: its profile folder's name, and its SID from its system's
/// `ProfileList`.
struct User {
    username: Option<String>,
    sid: Option<String>,
}

/// An entry's record, its fields in this order.
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
    #[serde(rename = "OfficeVersion")]
    office_version: &'a str,
    #[serde(rename = "Application")]
    application: &'a str,
    #[serde(rename = "List")]
    list: &'static str,
    #[serde(rename = "Account")]
    account: Option<&'a str>,
    #[serde(rename = "Policy")]
    policy: bool,
    #[serde(rename = "MRU-Position")]
    mru_position: Option<u32>,
    #[serde(rename = "IsMostRecent")]
    is_most_recent: bool,
    #[serde(rename = "Path")]
    path: Option<&'a str>,
    #[serde(rename = "LastOpened")]
    last_opened: Option<String>,
    #[serde(rename = "TrustedAt")]
    trusted_at: Option<String>,
    #[serde(rename = "MacrosEnabled")]
    macros_enabled: Option<bool>,
    #[serde(rename = "TrustFlags")]
    trust_flags: Option<String>,
    #[serde(rename = "Setting")]
    setting: Option<&'a str>,
    #[serde(rename = "Value")]
    value: Option<&'a str>,
    #[serde(rename = "KeyPath")]
    key_path: &'a str,
    #[serde(rename = "KeyLastWritten")]
    key_last_written: Option<String>,
    #[serde(rename = "DataRaw")]
    data_raw: Option<String>,
    #[serde(rename = "TransactionLogStatus")]
    transaction_log_status: &'static str,
    #[serde(rename = "LogsApplied")]
    logs_applied: &'a [String],
    #[serde(rename = "Problems")]
    problems: &'a [String],
}

const CSV_HEADER: [&str; 24] = [
    "Host",
    "Username",
    "UserSID",
    "SourcePath",
    "OfficeVersion",
    "Application",
    "List",
    "Account",
    "Policy",
    "MRU-Position",
    "IsMostRecent",
    "Path",
    "LastOpenedUTC",
    "TrustedAtUTC",
    "MacrosEnabled",
    "TrustFlags",
    "Setting",
    "Value",
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

impl<'a> Record<'a> {
    fn new(opened: &'a Opened, user: &'a User, entry: &'a OfficeEntry) -> Self {
        // the setting's name: a value of Security or Trusted Locations, or a location's key
        let setting = matches!(
            entry.list,
            OfficeList::Security | OfficeList::TrustedLocations
        )
        .then_some(entry.value_name.as_str());
        Record {
            host: opened.host.as_deref(),
            username: user.username.as_deref(),
            user_sid: user.sid.as_deref(),
            source_path: &opened.source,
            office_version: &entry.office_version,
            application: &entry.application,
            list: entry.list.name(),
            account: entry.account.as_deref(),
            policy: entry.policy,
            mru_position: entry.mru_position,
            is_most_recent: entry.mru_position == Some(0),
            path: entry.path.as_deref(),
            last_opened: entry.last_opened.and_then(filetime),
            trusted_at: entry.trusted_at.and_then(filetime),
            macros_enabled: entry.macros_enabled,
            trust_flags: entry.trust_flags.map(|f| format!("0x{f:08X}")),
            setting,
            value: entry.value.as_deref(),
            key_path: &entry.key_path,
            key_last_written: filetime(entry.key_last_written),
            data_raw: entry.data_raw.as_deref().map(hex),
            transaction_log_status: opened.log_status(entry.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &entry.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let flag = |v: Option<bool>| v.map(|b| b.to_string()).unwrap_or_default();
        vec![
            text(self.host),
            text(self.username),
            text(self.user_sid),
            self.source_path.to_string(),
            self.office_version.to_string(),
            self.application.to_string(),
            self.list.to_string(),
            text(self.account),
            self.policy.to_string(),
            self.mru_position.map(|n| n.to_string()).unwrap_or_default(),
            self.is_most_recent.to_string(),
            text(self.path),
            text(self.last_opened.as_deref()),
            text(self.trusted_at.as_deref()),
            flag(self.macros_enabled),
            text(self.trust_flags.as_deref()),
            text(self.setting),
            text(self.value),
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

/// Whether a setting's value lets macros run more freely than Office's defaults: `VBAWarnings`
/// 1 (all macros enabled), `BlockContentExecutionFromInternet` 0, `AccessVBOM` 1, or
/// `AllowNetworkLocations` 1.
fn weakens_macros(entry: &OfficeEntry) -> bool {
    if !entry.problems.is_empty() {
        return false;
    }
    let number = entry
        .value
        .as_deref()
        .and_then(|v| v.split(' ').next())
        .and_then(|n| n.parse::<u64>().ok());
    matches!(
        (entry.value_name.to_ascii_lowercase().as_str(), number),
        ("vbawarnings", Some(1))
            | ("blockcontentexecutionfrominternet", Some(0))
            | ("accessvbom", Some(1))
            | ("allownetworklocations", Some(1))
    )
}

/// Counts for the results summary.
#[derive(Default)]
struct Stats {
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
    versions: BTreeSet<String>,
    /// Documents whose macros were enabled: (time trusted, line).
    macros: Vec<(u64, String)>,
    /// Settings that let macros run more freely.
    weak: Vec<String>,
    /// Trusted locations users added (with a `Date`), or set by policy.
    added_locations: Vec<String>,
    /// The documents opened last: (time, line).
    latest: Vec<(u64, String)>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened`'s Office entries (when there is a writer) and count them.
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
        let Some((entries, problems)) = office(&opened.hive) else {
            self.without += 1;
            return Ok(());
        };
        for problem in &problems {
            if self.to_stderr {
                eprintln!("{}: {problem}", opened.source);
            }
            self.problem(&opened.source, problem);
        }
        let who = user.username.as_deref().unwrap_or("?");
        for entry in &entries {
            self.entries += 1;
            *self.lists.entry(entry.list.name()).or_default() += 1;
            self.versions.insert(entry.office_version.clone());
            for problem in &entry.problems {
                self.problem(
                    &opened.source,
                    &format!(r"{}\{}: {problem}", entry.key_path, entry.value_name),
                );
            }
            let app = format!("{} {}", entry.application, entry.office_version);
            let path = entry.path.as_deref().unwrap_or("(no path)");
            match entry.list {
                OfficeList::TrustRecords if entry.macros_enabled == Some(true) => {
                    let when = entry.trusted_at.and_then(filetime);
                    self.macros.push((
                        entry.trusted_at.unwrap_or(0),
                        format!("{}  {who}  {app}: {path}", when.as_deref().unwrap_or("?")),
                    ));
                }
                OfficeList::Security | OfficeList::TrustedLocations if weakens_macros(entry) => {
                    let by = if entry.policy { "policy" } else { who };
                    self.weak.push(format!(
                        "{}: {by}  {app}: {} = {}",
                        opened.source,
                        entry.value_name,
                        entry.value.as_deref().unwrap_or("")
                    ));
                }
                OfficeList::TrustedLocations
                    if entry.path.is_some()
                        && (entry.policy
                            || entry.value.as_deref().is_some_and(|v| v.contains("Date="))) =>
                {
                    let by = if entry.policy { "policy" } else { who };
                    self.added_locations.push(format!(
                        "{}: {by}  {app}: {path} ({})",
                        opened.source,
                        entry.value.as_deref().unwrap_or("")
                    ));
                }
                OfficeList::FileMru => {
                    if let Some((when, at)) = entry
                        .last_opened
                        .and_then(|when| Some((when, filetime(when)?)))
                    {
                        self.latest
                            .push((when, format!("{at}  {who}  {app}: {path}")));
                        if self.latest.len() > MAX_PRINTED * 4 {
                            self.trim_latest();
                        }
                    }
                }
                _ => {}
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

    /// The documents opened last, newest first, at most [`MAX_PRINTED`].
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

    /// `lines` under `label`, at most [`MAX_PRINTED`], with how many more there are.
    fn list(no_color: bool, label: &str, lines: &[String]) {
        p_label(no_color, label, &fmt_num(lines.len()));
        for line in lines.iter().take(MAX_PRINTED) {
            p(None, &format!("  {line}"), true);
        }
        if lines.len() > MAX_PRINTED {
            p(
                None,
                &format!("  … {} more", fmt_num(lines.len() - MAX_PRINTED)),
                true,
            );
        }
    }

    fn print(&mut self, no_color: bool) {
        self.trim_latest();
        self.macros.sort_by(|a, b| b.cmp(a));
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
            p_label(no_color, "Hives without Office: ", &fmt_num(self.without));
        }
        if !self.versions.is_empty() {
            p_label(no_color, "Office versions: ", &list(&self.versions));
        }
        let lists: Vec<String> = self
            .lists
            .iter()
            .map(|(name, n)| format!("{name} {}", fmt_num(*n)))
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
        let macros: Vec<String> = self.macros.iter().map(|(_, l)| l.clone()).collect();
        Self::list(
            no_color,
            "Documents whose macros were enabled (UTC trusted, user, application): ",
            &macros,
        );
        Self::list(
            no_color,
            "Settings that let macros run more freely: ",
            &self.weak,
        );
        Self::list(
            no_color,
            "Trusted locations added by users or set by policy: ",
            &self.added_locations,
        );
        if !self.latest.is_empty() {
            p_label(
                no_color,
                "Documents opened last (UTC, user, application): ",
                "",
            );
            for (_, line) in &self.latest {
                p(None, &format!("  {line}"), true);
            }
        }
        p_label(
            no_color,
            "Entries or keys that could not be read: ",
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
