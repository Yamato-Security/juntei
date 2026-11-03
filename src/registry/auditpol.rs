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

//! `parse-registry auditpol`: the audit policy in force in `SECURITY` hives
//! (`Policy\PolAdtEv`), read by ys-registry-parser's `audit_policy` module, with Hayabusa's
//! input options, progress bar and results summary. One record per subcategory (Windows Vista
//! and later) or category (Windows XP and 2003): what it was set to log, and the main events
//! it logs. The summary lists disabled audit options in the captured policy, not a history of policy changes.

use super::{
    Hosts, Opened, RecordWriter, choose_logs, files_read, filetime, find_hives_of_type,
    input_files, open, sink,
};
use crate::cli::RegistryAuditpolOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::audit_policy::{AuditPolicy, AuditSetting, Layout, Setting, audit_policy};
use serde::Serialize;
use std::collections::BTreeSet;
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Hives, gaps and problems listed in the summary.
const MAX_PRINTED: usize = 20;

/// The main events each subcategory logs (Microsoft's documentation of the advanced audit
/// policy settings), for what an analyst looks for; not every event it logs.
const EVENTS: [(&str, &[u32]); 47] = [
    ("Security State Change", &[4608, 4616]),
    ("Security System Extension", &[4697]),
    ("System Integrity", &[5038, 6281]),
    ("Logon", &[4624, 4625, 4648]),
    ("Logoff", &[4634, 4647]),
    ("Account Lockout", &[4625]),
    ("Special Logon", &[4672, 4964]),
    ("Other Logon/Logoff Events", &[4778, 4779, 4800, 4801]),
    ("Network Policy Server", &[6272, 6273]),
    ("User / Device Claims", &[4626]),
    ("Group Membership", &[4627]),
    ("File System", &[4656, 4663]),
    ("Registry", &[4657, 4663]),
    ("Kernel Object", &[4656, 4663]),
    ("SAM", &[4661]),
    (
        "Other Object Access Events",
        &[4698, 4699, 4700, 4701, 4702],
    ),
    ("Certification Services", &[4886, 4887]),
    ("Handle Manipulation", &[4690]),
    ("File Share", &[5140, 5142, 5144]),
    ("Filtering Platform Packet Drop", &[5152, 5153]),
    ("Filtering Platform Connection", &[5156, 5157, 5158]),
    ("Detailed File Share", &[5145]),
    ("Removable Storage", &[4663]),
    ("Sensitive Privilege Use", &[4673, 4674]),
    ("Non Sensitive Privilege Use", &[4673, 4674]),
    ("Process Creation", &[4688]),
    ("Process Termination", &[4689]),
    ("DPAPI Activity", &[4692, 4693]),
    ("RPC Events", &[5712]),
    ("Plug and Play Events", &[6416]),
    ("Token Right Adjusted Events", &[4703]),
    ("Audit Policy Change", &[4719, 4907]),
    ("Authentication Policy Change", &[4706, 4713, 4716, 4739]),
    ("Authorization Policy Change", &[4704, 4705]),
    ("MPSSVC Rule-Level Policy Change", &[4946, 4947, 4948]),
    ("Filtering Platform Policy Change", &[5441, 5447]),
    (
        "User Account Management",
        &[4720, 4722, 4724, 4725, 4726, 4738, 4740],
    ),
    ("Computer Account Management", &[4741, 4742, 4743]),
    ("Security Group Management", &[4728, 4732, 4756, 4799]),
    ("Distribution Group Management", &[4749, 4753]),
    ("Application Group Management", &[4783, 4791]),
    ("Directory Service Access", &[4662]),
    ("Directory Service Changes", &[5136, 5137, 5141]),
    ("Directory Service Replication", &[4932, 4933]),
    ("Credential Validation", &[4776]),
    ("Kerberos Service Ticket Operations", &[4769]),
    ("Kerberos Authentication Service", &[4768, 4771]),
];

/// The main events of the subcategory `name`.
fn events_of(name: &str) -> &'static [u32] {
    EVENTS
        .iter()
        .find(|(n, _)| *n == name)
        .map_or(&[], |(_, events)| events)
}

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryAuditpolOptions) -> i32 {
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
    let mut stats = Stats::default();
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

/// A setting's record (or, for a value that doesn't decode, the hive's), its fields in this
/// order.
#[derive(Serialize)]
struct Record<'a> {
    #[serde(rename = "Host")]
    host: Option<&'a str>,
    #[serde(rename = "SourcePath")]
    source_path: &'a str,
    #[serde(rename = "Layout")]
    layout: Option<&'static str>,
    #[serde(rename = "Category")]
    category: Option<&'static str>,
    #[serde(rename = "Subcategory")]
    subcategory: Option<&'static str>,
    #[serde(rename = "SubcategoryGUID")]
    subcategory_guid: Option<&'static str>,
    #[serde(rename = "SubcategoryID")]
    subcategory_id: Option<String>,
    #[serde(rename = "Setting")]
    setting: Option<String>,
    #[serde(rename = "Success")]
    success: Option<bool>,
    #[serde(rename = "Failure")]
    failure: Option<bool>,
    #[serde(rename = "AuditingEnabled")]
    auditing_enabled: Option<bool>,
    #[serde(rename = "Events")]
    events: &'static [u32],
    #[serde(rename = "KeyLastWritten")]
    key_last_written: Option<String>,
    #[serde(rename = "DataRaw")]
    data_raw: Option<String>,
    #[serde(rename = "TransactionLogStatus")]
    transaction_log_status: &'static str,
    #[serde(rename = "LogsApplied")]
    logs_applied: &'a [String],
    #[serde(rename = "Problems")]
    problems: Vec<String>,
}

const CSV_HEADER: [&str; 17] = [
    "Host",
    "SourcePath",
    "Layout",
    "Category",
    "Subcategory",
    "SubcategoryGUID",
    "SubcategoryID",
    "Setting",
    "Success",
    "Failure",
    "AuditingEnabled",
    "Events",
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
    /// The record of `setting` (`None`: the hive's, when nothing decodes).
    fn new(opened: &'a Opened, policy: &AuditPolicy, setting: Option<&AuditSetting>) -> Self {
        // what is wrong with the whole value goes with every setting; a setting's own with it
        let mut problems: Vec<String> = policy
            .problems
            .iter()
            .filter(|p| !p.contains("a setting of"))
            .cloned()
            .collect();
        if let Some(Setting::Other(n)) = setting.map(|s| s.setting) {
            problems.push(format!("a setting of 0x{n:x}, not 0 to 3"));
        }
        let layout = policy.events.layout;
        Record {
            host: opened.host.as_deref(),
            source_path: &opened.source,
            layout: layout.map(Layout::name),
            category: setting.and_then(|s| s.category),
            subcategory: setting.and_then(|s| s.subcategory),
            subcategory_guid: setting.and_then(|s| s.subcategory_guid),
            subcategory_id: setting
                .and_then(AuditSetting::subcategory_id)
                .map(|id| format!("%%{id}")),
            setting: setting.map(|s| s.setting.name()),
            success: setting
                .filter(|s| !matches!(s.setting, Setting::Other(_)))
                .map(|s| s.setting.success()),
            failure: setting
                .filter(|s| !matches!(s.setting, Setting::Other(_)))
                .map(|s| s.setting.failure()),
            auditing_enabled: policy.events.auditing_mode,
            events: setting.and_then(|s| s.subcategory).map_or(&[], events_of),
            key_last_written: filetime(policy.key_last_written),
            data_raw: policy.data_raw.as_deref().map(hex),
            transaction_log_status: opened.log_status(policy.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let flag = |b: Option<bool>| {
            b.map(|b| if b { "True" } else { "False" }.to_string())
                .unwrap_or_default()
        };
        vec![
            text(self.host),
            self.source_path.to_string(),
            text(self.layout),
            text(self.category),
            text(self.subcategory),
            text(self.subcategory_guid),
            text(self.subcategory_id.as_deref()),
            text(self.setting.as_deref()),
            flag(self.success),
            flag(self.failure),
            flag(self.auditing_enabled),
            self.events
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join("|"),
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
    hives: usize,
    failed: usize,
    hosts: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    /// Hives without `Policy\PolAdtEv`.
    without: Vec<String>,
    settings: usize,
    audited: usize,
    /// Each hive's lines.
    listed: Vec<Vec<String>>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened`'s policy (when there is a writer) and count them.
    fn hive(&mut self, opened: &Opened, mut writer: Option<&mut RecordWriter>) -> io::Result<()> {
        self.hives += 1;
        self.hosts.extend(opened.host.clone());
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
        let Some(policy) = audit_policy(&opened.hive) else {
            self.without.push(opened.source.clone());
            return Ok(());
        };
        for problem in &policy.problems {
            self.problem_count += 1;
            if self.problems.len() < MAX_PRINTED {
                self.problems.push(format!("{}: {problem}", opened.source));
            }
        }
        let settings = &policy.events.settings;
        let mut write = |record: Record| -> io::Result<()> {
            match writer.as_deref_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv()),
                Some(w) => w.json(&record),
                None => Ok(()),
            }
        };
        if settings.is_empty() {
            write(Record::new(opened, &policy, None))?;
        }
        for setting in settings {
            write(Record::new(opened, &policy, Some(setting)))?;
        }
        self.settings += settings.len();
        // XP's categories are audited only with auditing on
        let off = policy.events.auditing_mode == Some(false);
        let audited = settings
            .iter()
            .filter(|s| {
                !off && matches!(
                    s.setting,
                    Setting::Success | Setting::Failure | Setting::SuccessAndFailure
                )
            })
            .count();
        self.audited += audited;
        if self.listed.len() < MAX_PRINTED {
            self.listed.push(lines(opened, &policy, audited));
        }
        Ok(())
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
                "Dirty hives whose transaction logs were not applied (the policy may be out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if !self.without.is_empty() {
            p_label(
                no_color,
                r"Hives without Policy\PolAdtEv: ",
                &fmt_num(self.without.len()),
            );
            for source in self.without.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        p_label(
            no_color,
            "Settings: ",
            &format!(
                "{} ({} audited)",
                fmt_num(self.settings),
                fmt_num(self.audited)
            ),
        );
        p(None, "", true);
        for lines in &self.listed {
            for line in lines {
                p(None, line, true);
            }
            p(None, "", true);
        }
        if self.hives > self.listed.len() + self.without.len() {
            p(
                None,
                &format!(
                    "… more hives in the records (-o <FILE>): {} listed",
                    fmt_num(self.listed.len())
                ),
                true,
            );
        }
        p_label(
            no_color,
            "Values that could not be read or decoded: ",
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

/// A hive's summary: its layout and when it was written, what is audited, and the
/// subcategories with main events disabled in this captured policy.
fn lines(opened: &Opened, policy: &AuditPolicy, audited: usize) -> Vec<String> {
    let settings = &policy.events.settings;
    let host = opened
        .host
        .as_deref()
        .map_or_else(String::new, |h| format!("{h}  "));
    let mut lines = vec![format!("{host}{}", opened.source)];
    let layout = policy.events.layout.map_or("unknown", Layout::name);
    lines.push(format!(
        "  Layout: {layout}; {} of {} audited; last written {}",
        fmt_num(audited),
        fmt_num(settings.len()),
        filetime(policy.key_last_written).unwrap_or_else(|| "-".into())
    ));
    if policy.events.auditing_mode == Some(false) {
        lines.push("  Auditing is off: no category is audited".to_string());
    }
    let mut shown = |label: &str, pick: &dyn Fn(Setting) -> bool| {
        let names: Vec<String> = settings
            .iter()
            .filter(|s| pick(s.setting))
            .filter_map(|s| {
                let name = s.subcategory.or(s.category)?;
                let events = events_of(name);
                Some(if events.is_empty() {
                    name.to_string()
                } else {
                    let ids: Vec<String> = events.iter().map(u32::to_string).collect();
                    format!("{name} ({})", ids.join(", "))
                })
            })
            .collect();
        if !names.is_empty() {
            lines.push(format!("  {label}: {}", names.join("; ")));
        }
    };
    shown("Success and failure", &|s| s == Setting::SuccessAndFailure);
    shown("Success only", &|s| s == Setting::Success);
    shown("Failure only", &|s| s == Setting::Failure);
    // what investigations look for, and isn't logged
    let missing: Vec<String> = settings
        .iter()
        .filter(|s| s.setting == Setting::None)
        .filter_map(|s| {
            let name = s.subcategory?;
            let events = events_of(name);
            (!events.is_empty()).then(|| {
                let ids: Vec<String> = events.iter().map(u32::to_string).collect();
                format!("{name} ({})", ids.join(", "))
            })
        })
        .collect();
    if !missing.is_empty() {
        lines.push(format!(
            "  Not audited in captured policy (past policy may differ): {}",
            missing.join("; ")
        ));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_subcategory_with_events_is_one_windows_has() {
        let known: Vec<&str> = (0..9)
            .flat_map(|c| (0..16).filter_map(move |i| registry::audit_policy::subcategory(c, i)))
            .map(|(name, _)| name)
            .collect();
        for (name, events) in EVENTS {
            assert!(known.contains(&name), "{name}");
            assert!(!events.is_empty(), "{name}");
        }
        assert_eq!(events_of("Process Creation"), [4688]);
        assert!(events_of("Other System Events").is_empty());
    }
}
