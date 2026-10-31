//! `parse-registry userassist`: the programs and shortcuts users started from Explorer, from the
//! `UserAssist` keys of `NTUSER.DAT` hives, read by ys-registry-parser's `ntuser` module, with
//! Hayabusa's input options, progress bar and results summary. One record per item: its name
//! decoded from ROT13, its path with a known folder's GUID resolved, its run count, focus count
//! and time, and when it last ran. The summary lists the items run last, and the hives where
//! UserAssist is turned off.

use super::{
    Hosts, Opened, Profiles, RecordWriter, choose_logs, files_read, filetime, find_hives_of_type,
    input_files, open, profile_of_path, sink,
};
use crate::cli::RegistryUserAssistOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::ntuser::{UserAssistEntry, profile_name, user_assist, user_assist_settings};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems and items listed in the summary.
const MAX_PRINTED: usize = 20;

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryUserAssistOptions) -> i32 {
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

/// An item's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    host: Option<&'a str>,
    username: Option<&'a str>,
    user_sid: Option<&'a str>,
    source_path: &'a str,
    guid: &'a str,
    kind: &'static str,
    name_rot13: &'a str,
    name: &'a str,
    path: Option<&'a str>,
    run_count: Option<u32>,
    focus_count: Option<u32>,
    focus_time_ms: Option<u32>,
    last_run: Option<String>,
    format: Option<u32>,
    session_id: Option<u32>,
    key_last_written: Option<String>,
    data_raw: Option<String>,
    transaction_log_status: &'static str,
    logs_applied: &'a [String],
    problems: &'a [String],
}

const CSV_HEADER: [&str; 20] = [
    "Host",
    "Username",
    "UserSid",
    "SourcePath",
    "Guid",
    "Kind",
    "NameRot13",
    "Name",
    "Path",
    "RunCount",
    "FocusCount",
    "FocusTimeMs",
    "LastRunUTC",
    "Format",
    "SessionId",
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
    fn new(opened: &'a Opened, user: &'a User, entry: &'a UserAssistEntry) -> Self {
        Record {
            host: opened.host.as_deref(),
            username: user.username.as_deref(),
            user_sid: user.sid.as_deref(),
            source_path: &opened.source,
            guid: &entry.guid,
            kind: entry.kind,
            name_rot13: &entry.name_rot13,
            name: &entry.name,
            path: entry.path.as_deref(),
            run_count: entry.run_count,
            focus_count: entry.focus_count,
            focus_time_ms: entry.focus_time_ms,
            last_run: entry.last_run.and_then(filetime),
            format: entry.format,
            session_id: entry.session_id,
            key_last_written: filetime(entry.last_written),
            data_raw: entry.data_raw.as_deref().map(hex),
            transaction_log_status: opened.log_status(entry.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &entry.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let num = |v: Option<u32>| v.map(|n| n.to_string()).unwrap_or_default();
        vec![
            text(self.host),
            text(self.username),
            text(self.user_sid),
            self.source_path.to_string(),
            self.guid.to_string(),
            self.kind.to_string(),
            self.name_rot13.to_string(),
            self.name.to_string(),
            text(self.path),
            num(self.run_count),
            num(self.focus_count),
            num(self.focus_time_ms),
            text(self.last_run.as_deref()),
            num(self.format),
            num(self.session_id),
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
    /// rather than one item goes to stderr.
    to_stderr: bool,
    hives: usize,
    failed: usize,
    hosts: BTreeSet<String>,
    users: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    without: usize,
    /// Hives where UserAssist is turned off, and why.
    turned_off: Vec<String>,
    items: usize,
    kinds: BTreeMap<&'static str, usize>,
    /// The items run last: (last run, line).
    latest: Vec<(u64, String)>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened`'s UserAssist items (when there is a writer) and count them.
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
        let settings = user_assist_settings(&opened.hive);
        let off = settings.tracking_off();
        if !off.is_empty() {
            self.turned_off
                .push(format!("{}: {}", opened.source, off.join(", ")));
        }
        let Some((entries, problems)) = user_assist(&opened.hive) else {
            self.without += 1;
            return Ok(());
        };
        if self.to_stderr && settings.no_encrypt == Some(1) && !entries.is_empty() {
            eprintln!(
                r"{}: UserAssist\Settings NoEncrypt is 1: names written while it was set are not ROT13 and decode wrongly (see each item's problems)",
                opened.source
            );
        }
        for problem in &problems {
            if self.to_stderr {
                eprintln!("{}: UserAssist: {problem}", opened.source);
            }
            self.problem(&opened.source, "UserAssist", problem);
        }
        for entry in &entries {
            self.items += 1;
            *self.kinds.entry(entry.kind).or_default() += 1;
            for problem in &entry.problems {
                self.problem(
                    &opened.source,
                    &format!(r"UserAssist\{}\Count\{}", entry.guid, entry.name_rot13),
                    problem,
                );
            }
            // only times that are times: a corrupt one must not push real runs out of the list
            if let Some((when, at)) = entry
                .last_run
                .and_then(|when| Some((when, filetime(when)?)))
            {
                let user = user.username.as_deref().unwrap_or("?");
                let what = entry.path.as_deref().unwrap_or(&entry.name);
                let runs = entry
                    .run_count
                    .map_or_else(String::new, |n| format!(" (runs {n})"));
                self.latest
                    .push((when, format!("{at}  {user}  {what}{runs}")));
                // keep the most recent only
                if self.latest.len() > MAX_PRINTED * 4 {
                    self.trim_latest();
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

    /// The most recent items, newest first, at most [`MAX_PRINTED`].
    fn trim_latest(&mut self) {
        self.latest.sort_by(|a, b| b.cmp(a));
        self.latest.truncate(MAX_PRINTED);
    }

    fn problem(&mut self, source: &str, at: &str, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED {
            self.problems.push(format!("{source}: {at}: {problem}"));
        }
    }

    fn print(&mut self, no_color: bool) {
        self.trim_latest();
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
                "Dirty hives whose transaction logs were not applied (items may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if self.without > 0 {
            p_label(
                no_color,
                "Hives without UserAssist: ",
                &fmt_num(self.without),
            );
        }
        p_label(
            no_color,
            "Hives where UserAssist is turned off: ",
            &fmt_num(self.turned_off.len()),
        );
        for off in self.turned_off.iter().take(MAX_PRINTED) {
            p(None, &format!("  {off}"), true);
        }
        let kinds: Vec<String> = self
            .kinds
            .iter()
            .map(|(kind, n)| format!("{kind} {}", fmt_num(*n)))
            .collect();
        p_label(
            no_color,
            "Items: ",
            &if kinds.is_empty() {
                fmt_num(self.items)
            } else {
                format!("{} ({})", fmt_num(self.items), kinds.join(", "))
            },
        );
        if !self.latest.is_empty() {
            p_label(no_color, "Run last (UTC, user, item): ", "");
            for (_, line) in &self.latest {
                p(None, &format!("  {line}"), true);
            }
        }
        p_label(
            no_color,
            "Items or keys that could not be read: ",
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
