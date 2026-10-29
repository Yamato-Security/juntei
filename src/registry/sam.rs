//! `parse-registry sam`: the local accounts in `SAM` hives, read by ys-registry-parser's `sam`
//! module, with Hayabusa's input options, progress bar and results summary. One record per
//! account: its RID and user name, its account control flags, its last logon (to the 100 ns,
//! and to the second), when its password was last set, when it expires, its last failed logon,
//! its failed and total logon counts, its password hint, the Microsoft account it is linked to,
//! and whether the sign-in screen hides it. Password hashes are not read.

use super::{
    Hosts, Opened, RecordWriter, choose_logs, filetime, find_hives_of_type, input_files, open, sink,
};
use crate::cli::RegistrySamOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::sam::{Account, account_flag_names, accounts};
use serde::Serialize;
use std::collections::BTreeSet;
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Accounts and problems listed in the summary.
const MAX_PRINTED: usize = 20;
/// FILETIME ticks in a second.
const SECOND: u64 = 10_000_000;
/// The first RID of the accounts people make (the built-in ones are below).
const FIRST_USER_RID: u32 = 1000;
/// The `disabled` account control flag.
const DISABLED: u16 = 0x0001;

/// Run the command; returns the exit status.
pub fn run(opts: &RegistrySamOptions) -> i32 {
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
        "SAM hives",
        |d| find_hives_of_type(d, "SAM"),
    )
    .and_then(|files| Ok((files, choose_logs(opts.no_logs, &given_logs)?)))
    .and_then(|(files, logs)| {
        Ok((
            files,
            logs,
            sink(opts.output.as_ref(), opts.clobber, data_on_stdout)?,
        ))
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
        p_label(no_color, "SAM hives: ", &fmt_num(files.len()));
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
    let mut stats = Stats::default();
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        let parsed = open(path, logs).and_then(|mut opened| {
            if !opened.hive_type.is_empty() && opened.hive_type != "SAM" {
                return Err(format!("a {} hive, not a SAM hive", opened.hive_type));
            }
            opened.host = hosts.of(path, &opened);
            Ok(opened)
        });
        match parsed {
            Ok(opened) => {
                if let Err(e) = stats.hive(&opened, opts, writer.as_mut()) {
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

/// An account's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    host: Option<&'a str>,
    source_path: &'a str,
    rid: Option<u32>,
    username: Option<&'a str>,
    full_name: Option<&'a str>,
    comment: Option<&'a str>,
    user_comment: Option<&'a str>,
    account_flags: Vec<String>,
    account_flags_value: Option<u16>,
    last_logon: Option<String>,
    last_logon_whole_seconds: Option<String>,
    password_last_set: Option<String>,
    account_expiration: Option<String>,
    last_failed_logon: Option<String>,
    failed_logon_count: Option<u16>,
    logon_count: Option<u16>,
    password_hint: Option<&'a str>,
    internet_user_name: Option<&'a str>,
    hidden_from_logon_ui: Option<bool>,
    home_directory: Option<&'a str>,
    profile_path: Option<&'a str>,
    logon_script: Option<&'a str>,
    key_last_written: Option<String>,
    names_key_last_written: Option<String>,
    last_logon_filetime: Option<u64>,
    password_last_set_filetime: Option<u64>,
    account_expiration_filetime: Option<u64>,
    last_failed_logon_filetime: Option<u64>,
    transaction_log_status: &'static str,
    logs_applied: &'a [String],
    problems: &'a [String],
}

const CSV_HEADER: [&str; 31] = [
    "Host",
    "SourcePath",
    "Rid",
    "Username",
    "FullName",
    "Comment",
    "UserComment",
    "AccountFlags",
    "AccountFlagsValue",
    "LastLogonUTC",
    "LastLogonWholeSecondsUTC",
    "PasswordLastSetUTC",
    "AccountExpirationUTC",
    "LastFailedLogonUTC",
    "FailedLogonCount",
    "LogonCount",
    "PasswordHint",
    "InternetUserName",
    "HiddenFromLogonUI",
    "HomeDirectory",
    "ProfilePath",
    "LogonScript",
    "KeyLastWrittenUTC",
    "NamesKeyLastWrittenUTC",
    "LastLogonFiletime",
    "PasswordLastSetFiletime",
    "AccountExpirationFiletime",
    "LastFailedLogonFiletime",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

impl<'a> Record<'a> {
    fn new(opened: &'a Opened, a: &'a Account) -> Self {
        let f = a.f.as_ref();
        Record {
            host: opened.host.as_deref(),
            source_path: &opened.source,
            rid: a.rid,
            username: a.username.as_deref(),
            full_name: a.v.full_name.as_deref(),
            comment: a.v.comment.as_deref(),
            user_comment: a.v.user_comment.as_deref(),
            account_flags: f.map(|f| account_flag_names(f.flags)).unwrap_or_default(),
            account_flags_value: f.map(|f| f.flags),
            // 0 is never; an account that doesn't expire has 0 or 0x7FFFFFFFFFFFFFFF, past 9999
            last_logon: f.and_then(|f| filetime(f.last_logon)),
            last_logon_whole_seconds: f
                .and_then(|f| filetime(f.last_logon - f.last_logon % SECOND)),
            password_last_set: f.and_then(|f| filetime(f.password_last_set)),
            account_expiration: f.and_then(|f| filetime(f.account_expires)),
            last_failed_logon: f.and_then(|f| filetime(f.last_failed_logon)),
            failed_logon_count: f.map(|f| f.failed_logon_count),
            logon_count: f.map(|f| f.logon_count),
            password_hint: a.password_hint.as_deref(),
            internet_user_name: a.internet_user_name.as_deref(),
            hidden_from_logon_ui: a.hidden_from_logon_ui,
            home_directory: a.v.home_directory.as_deref(),
            profile_path: a.v.profile_path.as_deref(),
            logon_script: a.v.logon_script.as_deref(),
            key_last_written: a.last_written.and_then(filetime),
            names_key_last_written: a.names_last_written.and_then(filetime),
            last_logon_filetime: f.map(|f| f.last_logon),
            password_last_set_filetime: f.map(|f| f.password_last_set),
            account_expiration_filetime: f.map(|f| f.account_expires),
            last_failed_logon_filetime: f.map(|f| f.last_failed_logon),
            transaction_log_status: opened.log_status(a.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &a.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let num = |v: Option<u64>| v.map(|n| n.to_string()).unwrap_or_default();
        let flag = |b: bool| if b { "True" } else { "False" }.to_string();
        vec![
            text(self.host),
            self.source_path.to_string(),
            num(self.rid.map(u64::from)),
            text(self.username),
            text(self.full_name),
            text(self.comment),
            text(self.user_comment),
            self.account_flags.join("|"),
            num(self.account_flags_value.map(u64::from)),
            text(self.last_logon.as_deref()),
            text(self.last_logon_whole_seconds.as_deref()),
            text(self.password_last_set.as_deref()),
            text(self.account_expiration.as_deref()),
            text(self.last_failed_logon.as_deref()),
            num(self.failed_logon_count.map(u64::from)),
            num(self.logon_count.map(u64::from)),
            text(self.password_hint),
            text(self.internet_user_name),
            self.hidden_from_logon_ui.map(flag).unwrap_or_default(),
            text(self.home_directory),
            text(self.profile_path),
            text(self.logon_script),
            text(self.key_last_written.as_deref()),
            text(self.names_key_last_written.as_deref()),
            num(self.last_logon_filetime),
            num(self.password_last_set_filetime),
            num(self.account_expiration_filetime),
            num(self.last_failed_logon_filetime),
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
    hives: usize,
    failed: usize,
    hosts: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    without: usize,
    without_rid: usize,
    accounts: usize,
    disabled: usize,
    /// Each account: RID, name, state, last logon, logons.
    listed: Vec<String>,
    /// The accounts people made that the sign-in screen hides.
    hidden: Vec<String>,
    hidden_count: usize,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened`'s accounts (when there is a writer) and count them.
    fn hive(
        &mut self,
        opened: &Opened,
        opts: &RegistrySamOptions,
        mut writer: Option<&mut RecordWriter>,
    ) -> io::Result<()> {
        self.hives += 1;
        self.hosts.extend(opened.host.clone());
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
        let Some((found, problems)) = accounts(&opened.hive) else {
            self.without += 1;
            return Ok(());
        };
        for problem in &problems {
            self.problem(&opened.source, "Users", problem);
        }
        let mut any = false;
        for account in &found {
            if opts.rid.is_some_and(|rid| account.rid != Some(rid)) {
                continue;
            }
            any = true;
            self.account(opened, account);
            let record = Record::new(opened, account);
            match writer.as_deref_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv())?,
                Some(w) => w.json(&record)?,
                None => {}
            }
        }
        if opts.rid.is_some() && !any {
            self.without_rid += 1;
        }
        Ok(())
    }

    /// Count an account written.
    fn account(&mut self, opened: &Opened, a: &Account) {
        self.accounts += 1;
        let name = a.username.as_deref().unwrap_or("(no name)");
        let rid = a.rid.map_or_else(|| "?".to_string(), |r| r.to_string());
        for problem in &a.problems {
            self.problem(&opened.source, &format!("{rid} {name}"), problem);
        }
        let disabled = a.f.is_some_and(|f| f.flags & DISABLED != 0);
        self.disabled += usize::from(disabled);
        if self.listed.len() < MAX_PRINTED {
            let state = if disabled { "disabled" } else { "enabled" };
            let logon =
                a.f.and_then(|f| filetime(f.last_logon))
                    .unwrap_or_else(|| "never".to_string());
            let logons = a.f.map_or(0, |f| f.logon_count);
            self.listed.push(format!(
                "{}: {rid} {name} ({state}, last logon {logon}, {} logons)",
                opened.source,
                fmt_num(usize::from(logons))
            ));
        }
        if a.hidden_from_logon_ui == Some(true) && a.rid.is_some_and(|r| r >= FIRST_USER_RID) {
            self.hidden_count += 1;
            if self.hidden.len() < MAX_PRINTED {
                self.hidden.push(format!("{}: {rid} {name}", opened.source));
            }
        }
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
        p_label(no_color, "SAM hives parsed: ", &fmt_num(self.hives));
        if self.failed > 0 {
            p_label(no_color, "Could not parse: ", &fmt_num(self.failed));
        }
        if !self.hosts.is_empty() {
            let hosts: Vec<&str> = self.hosts.iter().map(String::as_str).collect();
            p_label(no_color, "Hosts: ", &hosts.join(", "));
        }
        p_label(
            no_color,
            "Dirty hives recovered with their transaction logs: ",
            &fmt_num(self.recovered),
        );
        if !self.dirty.is_empty() {
            p_label(
                no_color,
                "Dirty hives whose transaction logs were not applied (accounts may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if self.without > 0 {
            p_label(no_color, "Hives without accounts: ", &fmt_num(self.without));
        }
        if self.without_rid > 0 {
            p_label(
                no_color,
                "Hives without the account of --rid: ",
                &fmt_num(self.without_rid),
            );
        }
        p_label(
            no_color,
            "Accounts: ",
            &format!(
                "{} (enabled {}, disabled {})",
                fmt_num(self.accounts),
                fmt_num(self.accounts - self.disabled),
                fmt_num(self.disabled)
            ),
        );
        for account in &self.listed {
            p(None, &format!("  {account}"), true);
        }
        p_label(
            no_color,
            "Accounts people made that the sign-in screen hides: ",
            &fmt_num(self.hidden_count),
        );
        for hidden in &self.hidden {
            p(None, &format!("  {hidden}"), true);
        }
        p_label(
            no_color,
            "Values that could not be read or don't match: ",
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
