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

//! `parse-registry feature-usage`: how users used applications in Explorer, from `NTUSER.DAT`
//! hives, read by ys-registry-parser's `ntuser` module, with Hayabusa's input options, progress
//! bar and results summary. One record per application, button or item of:
//!
//! - `FeatureUsage` (Windows 10 and later): how many times each application was switched to
//!   (`AppSwitched`), launched from the taskbar (`AppLaunch`), had its jump list opened
//!   (`ShowJumpView`) or its badge updated (`AppBadgeUpdated`), and each taskbar button clicked
//!   (`TrayButtonClicked`);
//! - `RecentApps` (Windows 10 up to 1709): applications run recently, with their launch counts,
//!   last access and the files they opened (`RecentItems`);
//! - `Taskband`: the programs pinned to the taskbar (`Favorites`, `FavoritesResolve`);
//! - `ProgramsCache`: the Start menu's cached programs and, on Windows 7, its and the taskbar's
//!   pinned ones.

use super::{
    Hosts, Opened, Profiles, RecordWriter, choose_logs, files_read, filetime, find_hives_of_type,
    input_files, open, profile_of_path, sink,
};
use crate::cli::RegistryFeatureUsageOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::ntuser::{
    AppUsage, feature_usage, feature_usage_created, profile_name, programs_cache, recent_apps,
    taskband, user_assist_path,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems and items listed in the summary.
const MAX_PRINTED: usize = 20;
/// Applications listed among those switched to most.
const MAX_SWITCHED: usize = 10;

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryFeatureUsageOptions) -> i32 {
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

/// A record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    #[serde(rename = "Host")]
    host: Option<&'a str>,
    #[serde(rename = "User")]
    username: Option<&'a str>,
    #[serde(rename = "UserSID")]
    user_sid: Option<&'a str>,
    #[serde(rename = "SourcePath")]
    source_path: &'a str,
    #[serde(rename = "Source")]
    source: &'a str,
    #[serde(rename = "App")]
    application: Option<&'a str>,
    #[serde(rename = "AppPath")]
    application_path: Option<String>,
    #[serde(rename = "Count")]
    count: Option<u32>,
    #[serde(rename = "LastAccessed")]
    last_accessed: Option<String>,
    #[serde(rename = "ItemPath")]
    item_path: Option<&'a str>,
    #[serde(rename = "DisplayName")]
    display_name: Option<&'a str>,
    #[serde(rename = "PinType")]
    pin_type: Option<&'a str>,
    #[serde(rename = "Index")]
    index: Option<usize>,
    #[serde(rename = "ValueName")]
    value_name: &'a str,
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

const CSV_HEADER: [&str; 20] = [
    "Host",
    "Username",
    "UserSID",
    "SourcePath",
    "Source",
    "Application",
    "ApplicationPath",
    "Count",
    "LastAccessedUTC",
    "ItemPath",
    "DisplayName",
    "PinType",
    "Index",
    "ValueName",
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
    fn new(opened: &'a Opened, user: &'a User, entry: &'a AppUsage) -> Self {
        Record {
            host: opened.host.as_deref(),
            username: user.username.as_deref(),
            user_sid: user.sid.as_deref(),
            source_path: &opened.source,
            source: &entry.source,
            application: entry.application.as_deref(),
            application_path: entry.application.as_deref().and_then(user_assist_path),
            count: entry.count,
            last_accessed: entry.last_accessed.and_then(filetime),
            item_path: entry.item_path.as_deref(),
            display_name: entry.display_name.as_deref(),
            pin_type: entry.pin_type.as_deref(),
            index: entry.index,
            value_name: &entry.value_name,
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
        vec![
            text(self.host),
            text(self.username),
            text(self.user_sid),
            self.source_path.to_string(),
            self.source.to_string(),
            text(self.application),
            text(self.application_path.as_deref()),
            self.count.map(|n| n.to_string()).unwrap_or_default(),
            text(self.last_accessed.as_deref()),
            text(self.item_path),
            text(self.display_name),
            text(self.pin_type),
            self.index.map(|n| n.to_string()).unwrap_or_default(),
            self.value_name.to_string(),
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
    /// Whether the records go to stdout (`-t` without `-o`): no summary, so what concerns a hive
    /// rather than one record goes to stderr.
    to_stderr: bool,
    hives: usize,
    failed: usize,
    hosts: BTreeSet<String>,
    users: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    without: usize,
    records: usize,
    sources: BTreeMap<String, usize>,
    /// When each hive's `FeatureUsage` was made: (time, line).
    created: Vec<(u64, String)>,
    /// Applications switched to: (count, line).
    switched: Vec<(u32, String)>,
    /// Recent apps and items: (last access, line).
    recent: Vec<(u64, String)>,
    /// Taskbar pins: lines.
    pinned: Vec<String>,
    pinned_count: usize,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened` (when there is a writer) and count them.
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
        let hive = &opened.hive;
        let who = user.username.as_deref().unwrap_or("?").to_string();
        if let Some(created) = feature_usage_created(hive)
            && let Some(at) = filetime(created)
        {
            self.created.push((created, format!("{at}  {who}")));
        }
        let parts = [
            ("FeatureUsage", feature_usage(hive)),
            ("RecentApps", recent_apps(hive)),
            ("Taskband", taskband(hive)),
            ("ProgramsCache", programs_cache(hive)),
        ];
        if parts.iter().all(|(_, part)| part.is_none()) {
            self.without += 1;
            return Ok(());
        }
        for (name, part) in parts {
            let Some((entries, problems)) = part else {
                continue;
            };
            for problem in &problems {
                if self.to_stderr {
                    eprintln!("{}: {name}: {problem}", opened.source);
                }
                self.problem(&opened.source, name, problem);
            }
            for entry in &entries {
                self.entry(opened, &who, entry);
                let record = Record::new(opened, user, entry);
                match writer.as_deref_mut() {
                    Some(w) if w.is_csv() => w.csv(&record.csv())?,
                    Some(w) => w.json(&record)?,
                    None => {}
                }
            }
        }
        Ok(())
    }

    /// Count `entry` of `opened`, the hive of user `who`.
    fn entry(&mut self, opened: &Opened, who: &str, entry: &AppUsage) {
        self.records += 1;
        *self.sources.entry(entry.source.clone()).or_default() += 1;
        for problem in &entry.problems {
            let at = match (&entry.index, entry.value_name.is_empty()) {
                (Some(i), _) => format!(r"{}\{}[{i}]", entry.key_path, entry.value_name),
                (None, false) => format!(r"{}\{}", entry.key_path, entry.value_name),
                (None, true) => entry.key_path.clone(),
            };
            self.problem(&opened.source, &at, problem);
        }
        let application = entry.application.as_deref().unwrap_or("?");
        match entry.source.as_str() {
            "AppSwitched" => {
                if let Some(count) = entry.count {
                    let shown =
                        user_assist_path(application).unwrap_or_else(|| application.to_string());
                    self.switched
                        .push((count, format!("{count:>6}  {who}  {shown}")));
                    if self.switched.len() > MAX_SWITCHED * 4 {
                        trim(&mut self.switched, MAX_SWITCHED);
                    }
                }
            }
            "RecentApps" | "RecentItems" => {
                if let Some((when, at)) = entry
                    .last_accessed
                    .and_then(|when| Some((when, filetime(when)?)))
                {
                    let what = match &entry.item_path {
                        Some(item) => format!("{application}: {item}"),
                        None => application.to_string(),
                    };
                    self.recent.push((when, format!("{at}  {who}  {what}")));
                    if self.recent.len() > MAX_PRINTED * 4 {
                        trim(&mut self.recent, MAX_PRINTED);
                    }
                }
            }
            "Taskband" if entry.value_name.eq_ignore_ascii_case("Favorites") => {
                self.pinned_count += 1;
                if self.pinned.len() < MAX_PRINTED {
                    let item = entry.item_path.as_deref().unwrap_or("?");
                    let how = entry
                        .pin_type
                        .as_deref()
                        .map_or_else(String::new, |t| format!(" ({t})"));
                    self.pinned.push(format!("{who}  {item}{how}"));
                }
            }
            _ => {}
        }
    }

    fn problem(&mut self, source: &str, at: &str, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED {
            self.problems.push(format!("{source}: {at}: {problem}"));
        }
    }

    fn print(&mut self, no_color: bool) {
        trim(&mut self.switched, MAX_SWITCHED);
        trim(&mut self.recent, MAX_PRINTED);
        trim(&mut self.created, MAX_PRINTED);
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
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
                "Dirty hives whose transaction logs were not applied (records may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if self.without > 0 {
            p_label(
                no_color,
                "Hives without FeatureUsage, RecentApps, Taskband or ProgramsCache: ",
                &fmt_num(self.without),
            );
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
        if !self.created.is_empty() {
            p_label(
                no_color,
                "FeatureUsage counting since (KeyCreationTime, newest first): ",
                "",
            );
            for (_, line) in &self.created {
                p(None, &format!("  {line}"), true);
            }
        }
        if !self.switched.is_empty() {
            p_label(no_color, "Switched to most (AppSwitched, user): ", "");
            for (_, line) in &self.switched {
                p(None, &format!("  {line}"), true);
            }
        }
        if !self.recent.is_empty() {
            p_label(
                no_color,
                "Recent apps and items, newest first (UTC, user): ",
                "",
            );
            for (_, line) in &self.recent {
                p(None, &format!("  {line}"), true);
            }
        }
        p_label(
            no_color,
            "Pinned to the taskbar (Favorites): ",
            &fmt_num(self.pinned_count),
        );
        for line in &self.pinned {
            p(None, &format!("  {line}"), true);
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
    }
}

/// The `max` largest of `items`, largest first.
fn trim<T: Ord>(items: &mut Vec<(T, String)>, max: usize) {
    items.sort_by(|a, b| b.cmp(a));
    items.truncate(max);
}
