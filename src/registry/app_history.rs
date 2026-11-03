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

//! `parse-registry app-history`: traces third-party tools leave in `NTUSER.DAT` hives, read by
//! ys-registry-parser's `app_history` module, with Hayabusa's input options, progress bar and
//! results summary. One record per Sysinternals tool (with a first-use estimate), WinRAR,
//! 7-Zip and WinZip archive and folder history entry, OneDrive account and synced folder, Adobe
//! Acrobat and Reader recent file and folder, and TeamViewer and CCleaner value. The summary
//! lists the Sysinternals tools with the newest first-use estimates, the archives and folders used last, and the
//! OneDrive accounts.

use super::{
    Hosts, Opened, Profiles, RecordWriter, choose_logs, files_read, filetime, find_hives_of_type,
    input_files, open, profile_of_path, sink,
};
use crate::cli::RegistryAppHistoryOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::app_history::{App, AppTrace, ntuser};
use registry::ntuser::profile_name;
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems and traces listed in the summary.
const MAX_PRINTED: usize = 20;

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryAppHistoryOptions) -> i32 {
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

/// Names and values written as a JSON object, in their order.
struct Pairs<'a>(&'a [(String, String)]);

impl Serialize for Pairs<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (name, value) in self.0 {
            map.serialize_entry(name, value)?;
        }
        map.end()
    }
}

impl Pairs<'_> {
    /// `name=value` pairs, for CSV.
    fn csv(&self) -> String {
        self.0
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

/// A trace's record, its fields in this order.
#[derive(serde::Serialize)]
struct Record<'a> {
    #[serde(rename = "Host")]
    host: Option<&'a str>,
    #[serde(rename = "SourcePath")]
    source_path: &'a str,
    #[serde(rename = "User")]
    username: Option<&'a str>,
    #[serde(rename = "UserSID")]
    user_sid: Option<&'a str>,
    #[serde(rename = "App")]
    application: &'static str,
    #[serde(rename = "Item")]
    item: &'a str,
    #[serde(rename = "MRU-Position")]
    mru_position: Option<u32>,
    #[serde(rename = "Path")]
    path: Option<&'a str>,
    #[serde(rename = "Value")]
    value: Option<&'a str>,
    #[serde(rename = "FirstUsed")]
    first_used: Option<String>,
    #[serde(rename = "Details")]
    details: Pairs<'a>,
    #[serde(rename = "KeyPath")]
    key_path: &'a str,
    #[serde(rename = "KeyLastWritten")]
    key_last_written: Option<String>,
    #[serde(rename = "ValueName")]
    value_name: Option<&'a str>,
    #[serde(rename = "DataRaw")]
    data_raw: Option<String>,
    #[serde(rename = "TransactionLogStatus")]
    transaction_log_status: &'static str,
    #[serde(rename = "LogsApplied")]
    logs_applied: &'a [String],
    #[serde(rename = "Problems")]
    problems: &'a [String],
}

const CSV_HEADER: [&str; 18] = [
    "Host",
    "SourcePath",
    "Username",
    "UserSID",
    "Application",
    "Item",
    "MRU-Position",
    "Path",
    "Value",
    "FirstUsedUTC",
    "Details",
    "KeyPath",
    "KeyLastWrittenUTC",
    "ValueName",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

impl<'a> Record<'a> {
    fn new(opened: &'a Opened, user: &'a User, trace: &'a AppTrace) -> Self {
        Record {
            host: opened.host.as_deref(),
            source_path: &opened.source,
            username: user.username.as_deref(),
            user_sid: user.sid.as_deref(),
            application: trace.app.name(),
            item: &trace.item,
            mru_position: trace.mru_position,
            path: trace.path.as_deref(),
            value: trace.value.as_deref(),
            first_used: trace.first_used.and_then(filetime),
            details: Pairs(&trace.details),
            key_path: &trace.key_path,
            key_last_written: filetime(trace.key_last_written),
            value_name: trace.value_name.as_deref(),
            data_raw: trace.data_raw.as_deref().map(hex),
            transaction_log_status: opened.log_status(trace.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &trace.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        vec![
            text(self.host),
            self.source_path.to_string(),
            text(self.username),
            text(self.user_sid),
            self.application.to_string(),
            self.item.to_string(),
            self.mru_position.map(|n| n.to_string()).unwrap_or_default(),
            text(self.path),
            text(self.value),
            text(self.first_used.as_deref()),
            self.details.csv(),
            self.key_path.to_string(),
            text(self.key_last_written.as_deref()),
            text(self.value_name),
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
    to_stderr: bool,
    hives: usize,
    failed: usize,
    hosts: BTreeSet<String>,
    users: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    /// Hives without any trace.
    without: usize,
    records: usize,
    apps: BTreeMap<&'static str, usize>,
    /// Sysinternals tools with first-use estimates: (estimate, line).
    tools: Vec<(u64, String)>,
    /// Each archive and folder list's most recent entry, by its key's last write: (time, line).
    archives: Vec<(u64, String)>,
    /// OneDrive accounts.
    accounts: Vec<String>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened`'s traces (when there is a writer) and count them.
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
        let (traces, problems) = ntuser(&opened.hive);
        if traces.is_empty() {
            self.without += 1;
        }
        for problem in &problems {
            if self.to_stderr {
                eprintln!("{}: {problem}", opened.source);
            }
            self.problem(&opened.source, problem);
        }
        for trace in &traces {
            self.trace(opened, user, trace);
            let record = Record::new(opened, user, trace);
            match writer.as_deref_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv())?,
                Some(w) => w.json(&record)?,
                None => {}
            }
        }
        Ok(())
    }

    /// Count a trace written.
    fn trace(&mut self, opened: &Opened, user: &User, trace: &AppTrace) {
        self.records += 1;
        *self.apps.entry(trace.app.name()).or_default() += 1;
        for problem in &trace.problems {
            let at = trace.value_name.as_ref().map_or_else(
                || trace.key_path.clone(),
                |v| format!(r"{}\{v}", trace.key_path),
            );
            self.problem(&opened.source, &format!("{at}: {problem}"));
        }
        let host = opened.host.as_deref().unwrap_or(&opened.source);
        let who = user.username.as_deref().unwrap_or("?");
        match trace.app {
            App::Sysinternals => {
                if let Some((when, at)) = trace
                    .first_used
                    .and_then(|when| Some((when, filetime(when)?)))
                {
                    self.tools
                        .push((when, format!("{at}  {host}  {who}  {}", trace.item)));
                }
            }
            App::WinRar | App::SevenZip | App::WinZip if trace.mru_position == Some(0) => {
                if let Some(path) = &trace.path {
                    let at = filetime(trace.key_last_written).unwrap_or_default();
                    self.archives.push((
                        trace.key_last_written,
                        format!(
                            "{at}  {host}  {who}  {} {}  {path}",
                            trace.app.name(),
                            trace.item
                        ),
                    ));
                }
            }
            App::OneDrive if trace.value_name.is_none() => {
                let email = trace.value.as_deref().unwrap_or("-");
                let folder = trace.path.as_deref().unwrap_or("-");
                self.accounts
                    .push(format!("{host}  {who}  {}  {email}  {folder}", trace.item));
            }
            _ => {}
        }
    }

    fn problem(&mut self, source: &str, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED {
            self.problems.push(format!("{source}: {problem}"));
        }
    }

    fn print(&mut self, no_color: bool) {
        let list = |set: &BTreeSet<String>| set.iter().cloned().collect::<Vec<_>>().join(", ");
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p_label(no_color, "NTUSER.DAT hives parsed: ", &fmt_num(self.hives));
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
                "Dirty hives whose transaction logs were not applied (traces may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if self.without > 0 {
            p_label(
                no_color,
                "Hives without these tools' traces: ",
                &fmt_num(self.without),
            );
        }
        let apps: Vec<String> = self
            .apps
            .iter()
            .map(|(app, n)| format!("{app} {}", fmt_num(*n)))
            .collect();
        p_label(
            no_color,
            "Traces: ",
            &if apps.is_empty() {
                fmt_num(self.records)
            } else {
                format!("{} ({})", fmt_num(self.records), apps.join(", "))
            },
        );
        let newest = |items: &mut Vec<(u64, String)>| {
            items.sort_by(|a, b| b.cmp(a));
            items.truncate(MAX_PRINTED);
        };
        newest(&mut self.tools);
        if !self.tools.is_empty() {
            p_label(
                no_color,
                "Sysinternals first-use estimates (UTC, host, user, tool; newest first): ",
                "",
            );
            for (_, line) in &self.tools {
                p(None, &format!("  {line}"), true);
            }
        }
        newest(&mut self.archives);
        if !self.archives.is_empty() {
            p_label(
                no_color,
                "Archives and folders used last (list's last write UTC, host, user, list, most recent entry): ",
                "",
            );
            for (_, line) in &self.archives {
                p(None, &format!("  {line}"), true);
            }
        }
        if !self.accounts.is_empty() {
            p_label(
                no_color,
                "OneDrive accounts (host, user, account, e-mail, folder): ",
                "",
            );
            for line in self.accounts.iter().take(MAX_PRINTED) {
                p(None, &format!("  {line}"), true);
            }
        }
        p_label(
            no_color,
            "Traces or keys that could not be read: ",
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn details_as_json_and_csv() {
        let details = vec![
            ("Version".to_owned(), "DC".to_owned()),
            ("tFileName".to_owned(), "a.pdf".to_owned()),
        ];
        assert_eq!(
            serde_json::to_string(&Pairs(&details)).unwrap(),
            r#"{"Version":"DC","tFileName":"a.pdf"}"#
        );
        assert_eq!(Pairs(&details).csv(), "Version=DC; tFileName=a.pdf");
        assert_eq!(Pairs(&[]).csv(), "");
    }
}
