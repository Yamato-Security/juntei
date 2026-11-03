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

//! `parse-registry lsa-secrets`: which LSA secrets and cached domain logons `SECURITY` hives
//! hold, and when they were set, read by ys-registry-parser's `lsa_secrets` module, with
//! Hayabusa's input options, progress bar and results summary. Names and times only: nothing is
//! decrypted, and no secret's bytes are read or written. One record per secret
//! (`Policy\Secrets\<name>`): its kind, the service or user it names, when its current and old
//! values were set and whether they hold data; and one record for the cached logons (`Cache`):
//! how many slots Windows keeps and how many are in use.

use super::{
    Hosts, Opened, RecordWriter, choose_logs, files_read, filetime, find_hives_of_type,
    input_files, open, sink,
};
use crate::cli::RegistryLsaSecretsOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::lsa_secrets::{CachedLogons, Secret, SecretKind, cached_logons, secrets};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Hives, secrets and problems listed in the summary.
const MAX_PRINTED: usize = 20;

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryLsaSecretsOptions) -> i32 {
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
        "SECURITY hives",
        |d| find_hives_of_type(d, "SECURITY"),
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
        p_label(no_color, "SECURITY hives: ", &fmt_num(files.len()));
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
    let mut stats = Stats {
        to_stderr: data_on_stdout,
        ..Stats::default()
    };
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        let parsed = open(path, logs).and_then(|mut opened| {
            if !opened.hive_type.is_empty() && opened.hive_type != "SECURITY" {
                return Err(format!("a {} hive, not a SECURITY hive", opened.hive_type));
            }
            opened.host = hosts.of(path, &opened);
            Ok(opened)
        });
        match parsed {
            Ok(opened) => {
                if let Err(e) = stats.hive(&opened, writer.as_mut()) {
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

/// A secret's record, or the cached logons', its fields in this order.
#[derive(Serialize, Default)]
struct Record<'a> {
    #[serde(rename = "Host")]
    host: Option<&'a str>,
    #[serde(rename = "SourcePath")]
    source_path: &'a str,
    #[serde(rename = "Secret")]
    secret: &'a str,
    #[serde(rename = "Kind")]
    kind: &'static str,
    #[serde(rename = "ServiceName")]
    service_name: Option<&'a str>,
    #[serde(rename = "UserSID")]
    user_sid: Option<&'a str>,
    #[serde(rename = "CurrentSet")]
    current_set: Option<String>,
    #[serde(rename = "OldSet")]
    old_set: Option<String>,
    #[serde(rename = "HasCurrentValue")]
    has_current_value: Option<bool>,
    #[serde(rename = "HasOldValue")]
    has_old_value: Option<bool>,
    #[serde(rename = "CurrentValueLastWritten")]
    current_value_last_written: Option<String>,
    #[serde(rename = "OldValueLastWritten")]
    old_value_last_written: Option<String>,
    #[serde(rename = "KeyLastWritten")]
    key_last_written: Option<String>,
    #[serde(rename = "CachedLogons")]
    cached_logons: Option<usize>,
    #[serde(rename = "CachedLogonsInUse")]
    cached_logons_in_use: Option<usize>,
    #[serde(rename = "CachedLogonsKept")]
    cached_logons_kept: Option<u32>,
    #[serde(rename = "TransactionLogStatus")]
    transaction_log_status: &'static str,
    #[serde(rename = "LogsApplied")]
    logs_applied: &'a [String],
    #[serde(rename = "Problems")]
    problems: &'a [String],
}

const CSV_HEADER: [&str; 19] = [
    "Host",
    "SourcePath",
    "Secret",
    "Kind",
    "ServiceName",
    "UserSID",
    "CurrentSetUTC",
    "OldSetUTC",
    "HasCurrentValue",
    "HasOldValue",
    "CurrentValueLastWrittenUTC",
    "OldValueLastWrittenUTC",
    "KeyLastWrittenUTC",
    "CachedLogons",
    "CachedLogonsInUse",
    "CachedLogonsKept",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

impl<'a> Record<'a> {
    fn secret(opened: &'a Opened, s: &'a Secret) -> Self {
        Record {
            host: opened.host.as_deref(),
            source_path: &opened.source,
            secret: &s.name,
            kind: s.kind.name(),
            service_name: s.service_name.as_deref(),
            user_sid: s.user_sid.as_deref(),
            current_set: s.current.set.and_then(filetime),
            old_set: s.old.set.and_then(filetime),
            has_current_value: Some(s.current.has_data()),
            has_old_value: Some(s.old.has_data()),
            current_value_last_written: s.current.key_last_written.and_then(filetime),
            old_value_last_written: s.old.key_last_written.and_then(filetime),
            key_last_written: filetime(s.key_last_written),
            transaction_log_status: opened.log_status(s.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &s.problems,
            ..Record::default()
        }
    }

    fn cache(opened: &'a Opened, c: &'a CachedLogons) -> Self {
        Record {
            host: opened.host.as_deref(),
            source_path: &opened.source,
            secret: "Cache",
            kind: "cached_logons",
            key_last_written: filetime(c.key_last_written),
            cached_logons: Some(c.slots),
            cached_logons_in_use: Some(c.in_use),
            cached_logons_kept: c.control_count,
            transaction_log_status: opened.log_status(c.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &c.problems,
            ..Record::default()
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let num = |v: Option<u64>| v.map(|n| n.to_string()).unwrap_or_default();
        let flag = |b: Option<bool>| {
            b.map(|b| if b { "True" } else { "False" }.to_string())
                .unwrap_or_default()
        };
        vec![
            text(self.host),
            self.source_path.to_string(),
            self.secret.to_string(),
            self.kind.to_string(),
            text(self.service_name),
            text(self.user_sid),
            text(self.current_set.as_deref()),
            text(self.old_set.as_deref()),
            flag(self.has_current_value),
            flag(self.has_old_value),
            text(self.current_value_last_written.as_deref()),
            text(self.old_value_last_written.as_deref()),
            text(self.key_last_written.as_deref()),
            num(self.cached_logons.map(|n| n as u64)),
            num(self.cached_logons_in_use.map(|n| n as u64)),
            num(self.cached_logons_kept.map(u64::from)),
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
    recovered: usize,
    dirty: Vec<String>,
    /// Hives without `Policy\Secrets`.
    without: Vec<String>,
    secrets: usize,
    by_kind: BTreeMap<&'static str, usize>,
    /// Each hive's lines.
    listed: Vec<Vec<String>>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened`'s secrets and cached logons (when there is a writer) and
    /// count them.
    fn hive(&mut self, opened: &Opened, mut writer: Option<&mut RecordWriter>) -> io::Result<()> {
        self.hives += 1;
        self.hosts.extend(opened.host.clone());
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
        let mut write = |record: Record| -> io::Result<()> {
            match writer.as_deref_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv()),
                Some(w) => w.json(&record),
                None => Ok(()),
            }
        };
        let found = secrets(&opened.hive);
        if found.is_none() {
            self.without.push(opened.source.clone());
        }
        let (found, problems) = found.unwrap_or_default();
        for problem in &problems {
            if self.to_stderr {
                eprintln!("{}: {problem}", opened.source);
            }
            self.problem(&opened.source, problem);
        }
        for secret in &found {
            write(Record::secret(opened, secret))?;
            self.secrets += 1;
            *self.by_kind.entry(secret.kind.name()).or_default() += 1;
            for problem in &secret.problems {
                self.problem(&opened.source, problem);
            }
        }
        let cache = cached_logons(&opened.hive);
        if let Some(cache) = &cache {
            write(Record::cache(opened, cache))?;
            for problem in &cache.problems {
                self.problem(&opened.source, problem);
            }
        }
        if self.listed.len() < MAX_PRINTED {
            self.listed.push(lines(opened, &found, cache.as_ref()));
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
        p_label(no_color, "SECURITY hives parsed: ", &fmt_num(self.hives));
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
                "Dirty hives whose transaction logs were not applied (secrets may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if !self.without.is_empty() {
            p_label(
                no_color,
                r"Hives without Policy\Secrets: ",
                &fmt_num(self.without.len()),
            );
        }
        let kinds: Vec<String> = self
            .by_kind
            .iter()
            .map(|(kind, n)| format!("{kind} {}", fmt_num(*n)))
            .collect();
        p_label(
            no_color,
            "Secrets (names and times only; nothing is decrypted): ",
            &if kinds.is_empty() {
                fmt_num(self.secrets)
            } else {
                format!("{} ({})", fmt_num(self.secrets), kinds.join(", "))
            },
        );
        p(None, "", true);
        for lines in &self.listed {
            for line in lines {
                p(None, line, true);
            }
            p(None, "", true);
        }
        p_label(
            no_color,
            "Keys and values that could not be read: ",
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

/// A hive's summary: each secret with its kind and when its value was set, the services whose
/// account passwords are stored, and the cached logons.
fn lines(opened: &Opened, found: &[Secret], cache: Option<&CachedLogons>) -> Vec<String> {
    let host = opened
        .host
        .as_deref()
        .map_or_else(String::new, |h| format!("{h}  "));
    let mut lines = vec![format!("{host}{}", opened.source)];
    for secret in found.iter().take(MAX_PRINTED) {
        let set = secret
            .current
            .set
            .and_then(filetime)
            .map_or_else(|| "-".to_string(), |t| format!("set {t}"));
        let value = if secret.current.has_data() {
            ""
        } else {
            ", no current value"
        };
        let about = secret
            .service_name
            .as_deref()
            .or(secret.user_sid.as_deref())
            .map_or_else(String::new, |s| format!(" for {s}"));
        lines.push(format!(
            "  {}  ({}{about}{value}; {set})",
            secret.name,
            secret.kind.name()
        ));
    }
    if found.len() > MAX_PRINTED {
        lines.push(format!(
            "  … {} more secrets in the records (-o <FILE>)",
            fmt_num(found.len() - MAX_PRINTED)
        ));
    }
    let services: Vec<&str> = found
        .iter()
        .filter(|s| s.kind == SecretKind::ServiceAccount)
        .filter_map(|s| s.service_name.as_deref())
        .collect();
    if !services.is_empty() {
        lines.push(format!(
            "  Services whose account passwords LSA stores: {}",
            services.join(", ")
        ));
    }
    if let Some(cache) = cache {
        let kept = cache.control_count.map_or_else(String::new, |n| {
            format!(", {} kept (NL$Control)", fmt_num(n as usize))
        });
        lines.push(format!(
            "  Cached domain logons: {} of {} slots in use{kept}; Cache last written {}",
            fmt_num(cache.in_use),
            fmt_num(cache.slots),
            filetime(cache.key_last_written).unwrap_or_else(|| "-".into())
        ));
    }
    lines
}
