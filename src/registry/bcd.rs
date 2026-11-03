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

//! `parse-registry bcd`: the boot configuration of Boot Configuration Data (`BCD`) hives, read by
//! ys-registry-parser's `bcd` module, with Hayabusa's input options, progress bar and results
//! summary. One record per object of the store: the boot manager, each boot entry and each
//! settings object, with its type, description, device and path, whether it is the default
//! entry, the settings ransomware and rootkits change (recovery, boot status policy, test
//! signing, integrity checks, safe mode; own or inherited), and every element decoded. The
//! summary lists the default entry and those settings wherever they are set.

use super::{
    Hosts, Opened, RecordWriter, choose_logs, files_read, filetime, find_hives, hive_type_of_file,
    input_files, open, sink,
};
use crate::cli::RegistryBcdOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::bcd::{
    BcdElement, BcdObject, BcdValue, bcd, boot_status_policy_name, safe_boot_name,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems and findings listed in the summary.
const MAX_PRINTED: usize = 20;

/// Whether the file at `path` is named `BCD-Template` (`Windows\System32\config`): the template
/// Windows makes stores from, a hive of the same format.
fn is_template(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("BCD-Template"))
}

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryBcdOptions) -> i32 {
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
        "BCD hives",
        |d| {
            find_hives(d)
                .into_iter()
                .filter(|p| hive_type_of_file(p).as_deref() == Some("BCD") || is_template(p))
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
        p_label(no_color, "BCD hives: ", &fmt_num(files.len()));
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
            if !opened.hive_type.is_empty() && opened.hive_type != "BCD" {
                return Err(format!("a {} hive, not a BCD hive", opened.hive_type));
            }
            let Some(store) = bcd(&opened.hive) else {
                return Err("it has no Objects key: not a BCD store".to_string());
            };
            opened.host = hosts.of(path, &opened);
            Ok((opened, store, is_template(path)))
        });
        match parsed {
            Ok((opened, (objects, problems), template)) => {
                if let Err(e) = stats.hive(&opened, template, &objects, &problems, writer.as_mut())
                {
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

/// An object's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    #[serde(rename = "Host")]
    host: Option<&'a str>,
    #[serde(rename = "SourcePath")]
    source_path: &'a str,
    #[serde(rename = "HiveType")]
    hive_type: &'static str,
    #[serde(rename = "ObjectGUID")]
    object_guid: &'a str,
    #[serde(rename = "ObjectType")]
    object_type: Option<String>,
    #[serde(rename = "ObjectTypeName")]
    object_type_name: &'a str,
    #[serde(rename = "Description")]
    description: Option<&'a str>,
    #[serde(rename = "IsDefault")]
    is_default: bool,
    #[serde(rename = "DisplayOrder")]
    display_order: Option<usize>,
    #[serde(rename = "Device")]
    device: Option<&'a str>,
    #[serde(rename = "Path")]
    path: Option<&'a str>,
    #[serde(rename = "RecoveryEnabled")]
    recovery_enabled: Option<bool>,
    #[serde(rename = "TestSigning")]
    test_signing: Option<bool>,
    #[serde(rename = "NoIntegrityChecks")]
    no_integrity_checks: Option<bool>,
    #[serde(rename = "SafeBoot")]
    safe_boot: Option<String>,
    #[serde(rename = "BootStatusPolicy")]
    boot_status_policy: Option<String>,
    #[serde(rename = "Inherited")]
    inherited: &'a [String],
    #[serde(rename = "Elements")]
    elements: Vec<Value>,
    #[serde(rename = "KeyLastWritten")]
    key_last_written: Option<String>,
    #[serde(rename = "TransactionLogStatus")]
    transaction_log_status: &'static str,
    #[serde(rename = "LogsApplied")]
    logs_applied: &'a [String],
    #[serde(rename = "Problems")]
    problems: Vec<String>,
}

const CSV_HEADER: [&str; 22] = [
    "Host",
    "SourcePath",
    "HiveType",
    "ObjectGUID",
    "ObjectType",
    "ObjectTypeName",
    "Description",
    "IsDefault",
    "DisplayOrder",
    "Device",
    "Path",
    "RecoveryEnabled",
    "TestSigning",
    "NoIntegrityChecks",
    "SafeBoot",
    "BootStatusPolicy",
    "Inherited",
    "Elements",
    "KeyLastWrittenUTC",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// A number with its name, when it has one: `IgnoreAllFailures (1)`.
fn named(n: u64, name: Option<&str>) -> String {
    name.map_or_else(|| n.to_string(), |name| format!("{name} ({n})"))
}

/// An element's value as JSON.
fn value_json(value: &BcdValue) -> Value {
    match value {
        BcdValue::Device(d) => json!(d),
        BcdValue::String(s) | BcdValue::Object(s) => json!(s),
        BcdValue::ObjectList(list) => json!(list),
        BcdValue::Integer(n) => json!(n),
        BcdValue::Boolean(b) => json!(b),
        BcdValue::IntegerList(list) => json!(list),
        BcdValue::Undecoded => Value::Null,
    }
}

/// An element's value as text.
fn value_text(value: &BcdValue) -> String {
    match value {
        BcdValue::Device(d) => d.clone().unwrap_or_default(),
        BcdValue::String(s) | BcdValue::Object(s) => s.clone(),
        BcdValue::ObjectList(list) => list.join(","),
        BcdValue::Integer(n) => n.to_string(),
        BcdValue::Boolean(b) => b.to_string(),
        BcdValue::IntegerList(list) => list
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(","),
        BcdValue::Undecoded => String::new(),
    }
}

/// An element as JSON.
fn element_json(e: &BcdElement) -> Value {
    json!({
        "ElementType": e.key_name,
        "Name": e.name,
        "Value": value_json(&e.value),
        "DataRaw": e.data_raw.as_deref().map(hex),
        "KeyLastWritten": filetime(e.key_last_written),
        "Problems": e.problems,
    })
}

/// An element as text, for CSV: `16000009 BcdLibraryBoolean_AutoRecoveryEnabled=true`, with
/// its bytes when kept (`[0102…]`).
fn element_text(e: &BcdElement) -> String {
    let name = e.name.map_or_else(String::new, |name| format!(" {name}"));
    let raw = e
        .data_raw
        .as_deref()
        .map_or_else(String::new, |raw| format!(" [{}]", hex(raw)));
    format!("{}{name}={}{raw}", e.key_name, value_text(&e.value))
}

impl<'a> Record<'a> {
    fn new(opened: &'a Opened, template: bool, o: &'a BcdObject) -> Self {
        Record {
            host: opened.host.as_deref(),
            source_path: &opened.source,
            hive_type: if template { "BCD-Template" } else { "BCD" },
            object_guid: &o.guid,
            object_type: o.object_type.map(|t| format!("0x{t:08x}")),
            object_type_name: &o.type_name,
            description: o.description.as_deref(),
            is_default: o.is_default,
            display_order: o.display_order,
            device: o.device.as_deref(),
            path: o.path.as_deref(),
            recovery_enabled: o.recovery_enabled,
            test_signing: o.test_signing,
            no_integrity_checks: o.no_integrity_checks,
            safe_boot: o.safe_boot.map(|n| named(n, safe_boot_name(n))),
            boot_status_policy: o
                .boot_status_policy
                .map(|n| named(n, boot_status_policy_name(n))),
            inherited: &o.inherited,
            elements: o.elements.iter().map(element_json).collect(),
            key_last_written: filetime(o.key_last_written),
            transaction_log_status: opened.log_status(o.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: object_problems(o),
        }
    }

    fn csv(&self, o: &BcdObject) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let flag = |v: Option<bool>| v.map(|b| b.to_string()).unwrap_or_default();
        vec![
            text(self.host),
            self.source_path.to_string(),
            self.hive_type.to_string(),
            self.object_guid.to_string(),
            text(self.object_type.as_deref()),
            self.object_type_name.to_string(),
            text(self.description),
            self.is_default.to_string(),
            self.display_order
                .map(|n| n.to_string())
                .unwrap_or_default(),
            text(self.device),
            text(self.path),
            flag(self.recovery_enabled),
            flag(self.test_signing),
            flag(self.no_integrity_checks),
            text(self.safe_boot.as_deref()),
            text(self.boot_status_policy.as_deref()),
            self.inherited.join("; "),
            o.elements
                .iter()
                .map(element_text)
                .collect::<Vec<_>>()
                .join("; "),
            text(self.key_last_written.as_deref()),
            self.transaction_log_status.to_string(),
            self.logs_applied.join("|"),
            self.problems.join("; "),
        ]
    }
}

/// An object's problems and its elements', each element's with its key's name.
fn object_problems(o: &BcdObject) -> Vec<String> {
    o.problems
        .iter()
        .cloned()
        .chain(o.elements.iter().flat_map(|e| {
            e.problems
                .iter()
                .map(move |p| format!(r"Elements\{}: {p}", e.key_name))
        }))
        .collect()
}

// --- summary ----------------------------------------------------------------------------------

/// Counts for the results summary.
#[derive(Default)]
struct Stats {
    /// Whether the records go to stdout (`-t` without `-o`): no summary, so what concerns a hive
    /// rather than one object goes to stderr.
    to_stderr: bool,
    hives: usize,
    templates: usize,
    failed: usize,
    hosts: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    objects: usize,
    types: BTreeMap<String, usize>,
    /// Each store's default entry.
    defaults: Vec<String>,
    /// Settings that turn recovery, signing or integrity checks off, or force safe mode.
    findings: Vec<String>,
    finding_count: usize,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `objects`, the store of `opened`, and count them.
    fn hive(
        &mut self,
        opened: &Opened,
        template: bool,
        objects: &[BcdObject],
        problems: &[String],
        mut writer: Option<&mut RecordWriter>,
    ) -> io::Result<()> {
        self.hives += 1;
        self.templates += usize::from(template);
        self.hosts.extend(opened.host.clone());
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
        for problem in problems {
            if self.to_stderr {
                eprintln!("{}: Objects: {problem}", opened.source);
            }
            self.problem(&opened.source, "Objects", problem);
        }
        for o in objects {
            self.objects += 1;
            *self.types.entry(o.type_name.clone()).or_default() += 1;
            for problem in object_problems(o) {
                self.problem(&opened.source, &o.guid, &problem);
            }
            if o.is_default {
                let mut line = format!(
                    "{}: {} {}",
                    opened.source,
                    o.guid,
                    o.description.as_deref().unwrap_or(&o.type_name)
                );
                if let Some(path) = &o.path {
                    line.push_str(&format!(", {path}"));
                }
                if let Some(device) = &o.device {
                    line.push_str(&format!(", {device}"));
                }
                self.defaults.push(line);
            }
            self.note_findings(opened, template, o);
            let record = Record::new(opened, template, o);
            match writer.as_deref_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv(o))?,
                Some(w) => w.json(&record)?,
                None => {}
            }
        }
        Ok(())
    }

    /// The settings of `o` that turn recovery, signing or integrity checks off, or force safe
    /// mode, each with the time its element was set (its own) or where it comes from.
    fn note_findings(&mut self, opened: &Opened, template: bool, o: &BcdObject) {
        let when = |element: u32, setting: &str| {
            o.elements
                .iter()
                .find(|e| e.element_type == Some(element))
                .and_then(|e| filetime(e.key_last_written))
                .map(|t| format!("set {t}"))
                .or_else(|| {
                    o.inherited
                        .iter()
                        .find(|i| i.starts_with(&format!("{setting} from ")))
                        .map(|i| format!("inherited {}", &i[setting.len() + 1..]))
                })
                .unwrap_or_default()
        };
        let mut found = Vec::new();
        if o.test_signing == Some(true) {
            let at = [0x1600_0049, 0x2600_0027]
                .into_iter()
                .map(|e| when(e, "test_signing"))
                .find(|s| s.starts_with("set"))
                .unwrap_or_else(|| when(0x1600_0049, "test_signing"));
            found.push(format!("test signing on ({at})"));
        }
        if o.no_integrity_checks == Some(true) {
            found.push(format!(
                "integrity checks off ({})",
                when(0x1600_0048, "no_integrity_checks")
            ));
        }
        if o.recovery_enabled == Some(false) {
            found.push(format!(
                "recovery off ({})",
                when(0x1600_0009, "recovery_enabled")
            ));
        }
        if let Some(policy) = o.boot_status_policy {
            let name = boot_status_policy_name(policy);
            if name.is_some_and(|n| n.starts_with("Ignore")) || name.is_none() {
                found.push(format!(
                    "boot status policy {} ({})",
                    named(policy, name),
                    when(0x2500_00e0, "boot_status_policy")
                ));
            }
        }
        if let Some(mode) = o.safe_boot {
            found.push(format!(
                "safe mode {} ({})",
                named(mode, safe_boot_name(mode)),
                when(0x2500_0080, "safe_boot")
            ));
        }
        for finding in found {
            self.finding_count += 1;
            if self.findings.len() < MAX_PRINTED {
                let what = o.description.as_deref().unwrap_or(&o.type_name);
                let template = if template { " [template]" } else { "" };
                self.findings.push(format!(
                    "{}{template}: {} ({what}): {finding}",
                    opened.source, o.guid
                ));
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
        p_label(no_color, "BCD hives parsed: ", &fmt_num(self.hives));
        if self.templates > 0 {
            p_label(
                no_color,
                "Of them templates (BCD-Template): ",
                &fmt_num(self.templates),
            );
        }
        if self.failed > 0 {
            p_label(no_color, "Could not parse: ", &fmt_num(self.failed));
        }
        if !self.hosts.is_empty() {
            p_label(
                no_color,
                "Hosts: ",
                &self.hosts.iter().cloned().collect::<Vec<_>>().join(", "),
            );
        }
        p_label(
            no_color,
            "Dirty hives recovered with their transaction logs: ",
            &fmt_num(self.recovered),
        );
        if !self.dirty.is_empty() {
            p_label(
                no_color,
                "Dirty hives whose transaction logs were not applied (objects may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        let types: Vec<String> = self
            .types
            .iter()
            .map(|(t, n)| format!("{t} {}", fmt_num(*n)))
            .collect();
        p_label(
            no_color,
            "Objects: ",
            &if types.is_empty() {
                fmt_num(self.objects)
            } else {
                format!("{} ({})", fmt_num(self.objects), types.join(", "))
            },
        );
        if !self.defaults.is_empty() {
            p_label(no_color, "Default entries: ", "");
            for line in self.defaults.iter().take(MAX_PRINTED) {
                p(None, &format!("  {line}"), true);
            }
        }
        p_label(
            no_color,
            "Test signing, integrity checks off, recovery off, failures ignored, safe mode: ",
            &fmt_num(self.finding_count),
        );
        for line in &self.findings {
            p(None, &format!("  {line}"), true);
        }
        if self.finding_count > self.findings.len() {
            p(
                None,
                &format!(
                    "  … {} more (-o <FILE>)",
                    fmt_num(self.finding_count - self.findings.len())
                ),
                true,
            );
        }
        p_label(
            no_color,
            "Objects or elements that could not be read: ",
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
    fn templates_and_values() {
        assert!(is_template(Path::new(
            "/c/Windows/System32/config/BCD-Template"
        )));
        assert!(is_template(Path::new("bcd-template")));
        assert!(!is_template(Path::new("/c/Boot/BCD")));
        assert_eq!(named(1, Some("IgnoreAllFailures")), "IgnoreAllFailures (1)");
        assert_eq!(named(9, None), "9");
        assert_eq!(value_text(&BcdValue::IntegerList(vec![1, 2])), "1,2");
        assert_eq!(value_json(&BcdValue::Undecoded), Value::Null);
        assert_eq!(value_json(&BcdValue::Boolean(true)), json!(true));
    }
}
