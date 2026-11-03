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

//! `parse-registry app-compat`: application compatibility data beyond the ShimCache and Amcache,
//! in SOFTWARE and NTUSER.DAT hives, read by ys-registry-parser's `appcompat` module, with
//! Hayabusa's input options, progress bar and results summary. One record per program with
//! compatibility layers, program the Program Compatibility Assistant saw run (`Store`,
//! `Persisted`), program that loaded a CIT module, program path or command line in a CIT
//! database, and custom shim database (`InstalledSDB`, `Custom`: a persistence and injection
//! technique). The summary lists the custom shim databases, the programs with layers, and the
//! CIT modules' last uses.

use super::{
    Hosts, Opened, Profiles, RecordWriter, choose_logs, files_read, filetime, find_hives,
    hive_type_of_file, input_files, open, profile_of_path, sink,
};
use crate::cli::RegistryAppCompatOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::appcompat::{self, AppCompatEntry, AppCompatSource};
use registry::ntuser::profile_name;
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems and records listed in the summary.
const MAX_PRINTED: usize = 20;
/// The hives read, as their base blocks name them.
const HIVE_TYPES: [&str; 2] = ["SOFTWARE", "NTUSER.DAT"];

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryAppCompatOptions) -> i32 {
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
                let (entries, problems) = appcompat::software(&opened.hive);
                Read {
                    entries,
                    problems,
                    username: None,
                    sid: None,
                }
            } else {
                let (entries, problems) = appcompat::ntuser(&opened.hive);
                let username = profile_of_path(path).or_else(|| profile_name(&opened.hive));
                let sid = username.as_deref().and_then(|u| profiles.sid(path, u));
                Read {
                    entries,
                    problems,
                    username,
                    sid,
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

/// What one hive holds, with a user hive's user: its profile folder's name, and its SID from
/// its system's `ProfileList`.
struct Read {
    entries: Vec<AppCompatEntry>,
    problems: Vec<String>,
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

/// An entry's record, its fields in this order.
#[derive(serde::Serialize)]
struct Record<'a> {
    #[serde(rename = "Host")]
    host: Option<&'a str>,
    #[serde(rename = "SourcePath")]
    source_path: &'a str,
    #[serde(rename = "HiveType")]
    hive_type: &'a str,
    #[serde(rename = "User")]
    username: Option<&'a str>,
    #[serde(rename = "UserSID")]
    user_sid: Option<&'a str>,
    #[serde(rename = "Source")]
    source: &'static str,
    #[serde(rename = "Scope")]
    scope: &'static str,
    #[serde(rename = "Path")]
    path: Option<&'a str>,
    #[serde(rename = "Layers")]
    layers: Option<&'a str>,
    #[serde(rename = "LastSeen")]
    last_seen: Option<String>,
    #[serde(rename = "SdbGUID")]
    sdb_guid: Option<&'a str>,
    #[serde(rename = "SdbPath")]
    sdb_path: Option<&'a str>,
    #[serde(rename = "SdbInstalled")]
    sdb_installed: Option<String>,
    #[serde(rename = "Details")]
    details: Pairs<'a>,
    #[serde(rename = "KeyPath")]
    key_path: &'a str,
    #[serde(rename = "KeyLastWritten")]
    key_last_written: Option<String>,
    #[serde(rename = "ValueName")]
    value_name: Option<&'a str>,
    #[serde(rename = "WOW64")]
    wow64: bool,
    #[serde(rename = "DataRaw")]
    data_raw: Option<String>,
    #[serde(rename = "TransactionLogStatus")]
    transaction_log_status: &'static str,
    #[serde(rename = "LogsApplied")]
    logs_applied: &'a [String],
    #[serde(rename = "Problems")]
    problems: &'a [String],
}

const CSV_HEADER: [&str; 22] = [
    "Host",
    "SourcePath",
    "HiveType",
    "Username",
    "UserSID",
    "Source",
    "Scope",
    "Path",
    "Layers",
    "LastSeenUTC",
    "SdbGUID",
    "SdbPath",
    "SdbInstalledUTC",
    "Details",
    "KeyPath",
    "KeyLastWrittenUTC",
    "ValueName",
    "WOW64",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

impl<'a> Record<'a> {
    fn new(opened: &'a Opened, read: &'a Read, entry: &'a AppCompatEntry) -> Self {
        Record {
            host: opened.host.as_deref(),
            source_path: &opened.source,
            hive_type: &opened.hive_type,
            username: read.username.as_deref(),
            user_sid: read.sid.as_deref(),
            source: entry.source.name(),
            scope: if entry.machine { "machine" } else { "user" },
            path: entry.path.as_deref(),
            layers: entry.layers.as_deref(),
            last_seen: entry.last_seen.and_then(filetime),
            sdb_guid: entry.sdb_guid.as_deref(),
            sdb_path: entry.sdb_path.as_deref(),
            sdb_installed: entry.sdb_installed.and_then(filetime),
            details: Pairs(&entry.details),
            key_path: &entry.key_path,
            key_last_written: filetime(entry.key_last_written),
            value_name: entry.value_name.as_deref(),
            wow64: entry.wow64,
            data_raw: entry.data_raw.as_deref().map(hex),
            transaction_log_status: opened.log_status(entry.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &entry.problems,
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
            text(self.path),
            text(self.layers),
            text(self.last_seen.as_deref()),
            text(self.sdb_guid),
            text(self.sdb_path),
            text(self.sdb_installed.as_deref()),
            self.details.csv(),
            self.key_path.to_string(),
            text(self.key_last_written.as_deref()),
            text(self.value_name),
            flag(self.wow64),
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
    /// Hives parsed, by type.
    types: BTreeMap<&'static str, usize>,
    failed: usize,
    hosts: BTreeSet<String>,
    users: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    records: usize,
    sources: BTreeMap<&'static str, usize>,
    /// Custom shim databases: installed ones and those applied to programs.
    shims: Vec<String>,
    shim_count: usize,
    /// Programs with compatibility layers.
    layers: Vec<String>,
    layer_count: usize,
    /// Programs that loaded a CIT module: (last use, line).
    modules: Vec<(u64, String)>,
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
        self.users.extend(read.username.clone());
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
        for entry in &read.entries {
            self.entry(opened, read, entry);
            let record = Record::new(opened, read, entry);
            match writer.as_deref_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv())?,
                Some(w) => w.json(&record)?,
                None => {}
            }
        }
        Ok(())
    }

    /// Count an entry written.
    fn entry(&mut self, opened: &Opened, read: &Read, entry: &AppCompatEntry) {
        self.records += 1;
        *self.sources.entry(entry.source.name()).or_default() += 1;
        for problem in &entry.problems {
            let at = entry.value_name.as_ref().map_or_else(
                || entry.key_path.clone(),
                |v| format!(r"{}\{v}", entry.key_path),
            );
            self.problem(&opened.source, &format!("{at}: {problem}"));
        }
        let host = opened.host.as_deref().unwrap_or(&opened.source);
        let path = entry.path.as_deref().unwrap_or("-");
        match entry.source {
            AppCompatSource::InstalledSdb | AppCompatSource::CustomSdb => {
                self.shim_count += 1;
                if self.shims.len() < MAX_PRINTED {
                    let installed = entry
                        .sdb_installed
                        .and_then(filetime)
                        .unwrap_or_else(|| "-".to_owned());
                    let what = if entry.source == AppCompatSource::CustomSdb {
                        format!("applied to {path}")
                    } else {
                        "installed".to_owned()
                    };
                    self.shims.push(format!(
                        "{host}  {installed}  {}  {}  {what}",
                        entry.sdb_guid.as_deref().unwrap_or("-"),
                        entry.sdb_path.as_deref().unwrap_or("-"),
                    ));
                }
            }
            AppCompatSource::Layers => {
                self.layer_count += 1;
                if self.layers.len() < MAX_PRINTED {
                    let who = read.username.as_deref().unwrap_or("(machine)");
                    self.layers.push(format!(
                        "{host}  {who}  {path}  {}",
                        entry.layers.as_deref().unwrap_or("-")
                    ));
                }
            }
            AppCompatSource::CitModule => {
                if let Some((when, at)) = entry
                    .last_seen
                    .and_then(|when| Some((when, filetime(when)?)))
                {
                    let module = entry
                        .details
                        .iter()
                        .find(|(k, _)| k == "Module")
                        .map_or("-", |(_, v)| v.as_str());
                    self.modules
                        .push((when, format!("{at}  {host}  {path}  ({module})")));
                    if self.modules.len() > MAX_PRINTED * 4 {
                        self.trim_modules();
                    }
                }
            }
            AppCompatSource::PcaStore
            | AppCompatSource::PcaPersisted
            | AppCompatSource::CitSystem => {}
        }
    }

    /// The most recent module uses, newest first, at most [`MAX_PRINTED`].
    fn trim_modules(&mut self) {
        self.modules.sort_by(|a, b| b.cmp(a));
        self.modules.truncate(MAX_PRINTED);
    }

    fn problem(&mut self, source: &str, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED {
            self.problems.push(format!("{source}: {problem}"));
        }
    }

    fn print(&mut self, no_color: bool) {
        self.trim_modules();
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
        p_label(
            no_color,
            "Custom shim databases (host, installed UTC, GUID, path): ",
            &fmt_num(self.shim_count),
        );
        for line in &self.shims {
            p(None, &format!("  {line}"), true);
        }
        p_label(
            no_color,
            "Programs with compatibility layers (host, user, program, layers): ",
            &fmt_num(self.layer_count),
        );
        for line in &self.layers {
            p(None, &format!("  {line}"), true);
        }
        if !self.modules.is_empty() {
            p_label(
                no_color,
                "CIT module uses, newest first (UTC, host, program, module): ",
                "",
            );
            for (_, line) in &self.modules {
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
    fn details_as_json_and_csv() {
        let details = vec![
            ("Module".to_owned(), "System32/mrt100.dll".to_owned()),
            ("SignMedia".to_owned(), "1FF3254".to_owned()),
        ];
        assert_eq!(
            serde_json::to_string(&Pairs(&details)).unwrap(),
            r#"{"Module":"System32/mrt100.dll","SignMedia":"1FF3254"}"#
        );
        assert_eq!(
            Pairs(&details).csv(),
            "Module=System32/mrt100.dll; SignMedia=1FF3254"
        );
    }
}
