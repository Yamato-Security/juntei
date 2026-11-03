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

//! `parse-ntfs secure-sds`: the security descriptors of an NTFS volume (`$Secure:$SDS`), read by
//! the secure crate of ys-ntfs-parser. One record per descriptor: its security ID (the
//! `SecurityID` of `parse-ntfs mft`), owner, group, control flags, DACL and SACL (readable, as
//! icacls writes ACEs, and as SDDL), and what in it could not be decoded.

use crate::cli::SecureSdsOptions;
use crate::color::JunteiColor;
use crate::registry::{RecordWriter, sink};
use crate::util::{self, error_msg, fmt_num, p, p_label};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use ys_ntfs_parser::secure::sds::{self, Entry};
use ys_ntfs_parser::secure::{Ace, Acl, Error, SecurityDescriptor, Sid};

/// Errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Owners, descriptors and problems listed in the summary.
const MAX_PRINTED: usize = 20;

/// Run the command; returns the exit status.
pub fn run(opts: &SecureSdsOptions) -> i32 {
    let no_color = opts.common.no_color;
    let data_on_stdout = opts.data_on_stdout();
    let report_error = |msg: &str| {
        if data_on_stdout {
            eprintln!("{msg}");
        } else {
            error_msg(no_color, msg);
        }
    };

    let files = match (&opts.filepath, &opts.directory) {
        (Some(f), _) if f.is_file() => vec![f.clone()],
        (Some(f), _) => {
            report_error(&format!("The file does not exist: {}", f.display()));
            return 1;
        }
        (None, Some(d)) if !d.is_dir() => {
            report_error(&format!("The directory does not exist: {}", d.display()));
            return 1;
        }
        (None, Some(d)) => {
            let found = find_sds_files(d);
            if found.is_empty() {
                report_error(&format!(
                    "No $SDS streams ($Secure_$SDS) were found in {}",
                    d.display()
                ));
                return 1;
            }
            found
        }
        (None, None) => unreachable!("clap requires -f or -d"),
    };
    let sink = match sink(opts.output.as_ref(), opts.clobber, data_on_stdout, || {
        files.clone()
    }) {
        Ok(sink) => sink,
        Err(msg) => {
            report_error(&msg);
            return 1;
        }
    };
    let mut writer = match sink.map(|w| RecordWriter::new(w, opts.output_type(), &CSV_HEADER)) {
        Some(Ok(w)) => Some(w),
        Some(Err(e)) => {
            report_error(&format!("Could not write the security descriptors: {e}"));
            return 1;
        }
        None => None,
    };
    let total: u64 = files
        .iter()
        .map(|p| p.metadata().map_or(0, |m| m.len()))
        .sum();
    if !data_on_stdout {
        p_label(no_color, "$SDS streams: ", &fmt_num(files.len()));
        p_label(no_color, "Total file size: ", &util::size_display(total));
        p(None, "", true);
    }

    let mut stats = Stats::default();
    for path in &files {
        let source = path.to_string_lossy().to_string();
        let data = match std::fs::read(path) {
            Ok(data) => data,
            Err(e) => {
                stats.fail(&report_error, &format!("Could not read {source}: {e}"));
                continue;
            }
        };
        let stream = sds::read(&data);
        if stream.entries.is_empty() {
            stats.fail(
                &report_error,
                &format!("Could not parse {source}: no $SDS entries"),
            );
            continue;
        }
        stats.streams += 1;
        for problem in &stream.problems {
            stats.stream_problem(format!("{source}: {problem}"));
        }
        for entry in &stream.entries {
            let record = Record::new(&source, entry);
            stats.add(&record);
            let written = match writer.as_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv()),
                Some(w) => w.json(&record),
                None => Ok(()),
            };
            if let Err(e) = written {
                report_error(&format!("Could not write the security descriptors: {e}"));
                return 1;
            }
        }
    }
    if stats.failed > MAX_PRINTED_ERRORS {
        report_error(&format!(
            "{} more files could not be parsed",
            fmt_num(stats.failed - MAX_PRINTED_ERRORS)
        ));
    }
    if let Some(w) = writer
        && let Err(e) = w.finish()
    {
        report_error(&format!("Could not write the security descriptors: {e}"));
        return 1;
    }

    if !data_on_stdout {
        stats.print(no_color, files.len() > 1);
        if let Some(out) = &opts.output {
            p_label(
                no_color,
                "Results saved: ",
                &format!(
                    "{} ({} security descriptors, {})",
                    out.display(),
                    fmt_num(stats.descriptors),
                    util::file_size_display(out)
                ),
            );
        } else {
            p(
                None,
                "To save every security descriptor, add -o <FILE> (-t csv, json or jsonl).",
                true,
            );
        }
    }
    i32::from(stats.streams == 0)
}

/// Whether a file name is an `$SDS` stream's (`$Secure_$SDS`, `$SDS`, `$Secure:$SDS`, …).
pub(crate) fn is_sds_name(name: &str) -> bool {
    name.to_uppercase().contains("$SDS")
}

/// Whether the file starts with an `$SDS` entry holding a security descriptor.
fn starts_with_sds_entry(path: &Path) -> bool {
    let mut head = Vec::with_capacity(4096);
    File::open(path)
        .and_then(|f| f.take(4096).read_to_end(&mut head))
        .is_ok()
        && sds::looks_like_sds(&head)
}

/// The `$SDS` streams under `dir`, recursively, in path order.
fn find_sds_files(dir: &Path) -> Vec<PathBuf> {
    util::find_files(dir, |_| true)
        .into_iter()
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| is_sds_name(&n.to_string_lossy()))
                || starts_with_sds_entry(p)
        })
        .collect()
}

// --- records ----------------------------------------------------------------------------------

/// A security descriptor's record, its fields in this order.
#[derive(Serialize)]
struct Record {
    #[serde(rename = "SourcePath")]
    source_path: String,
    #[serde(rename = "SecurityID")]
    security_id: u32,
    #[serde(rename = "Hash")]
    hash: String,
    #[serde(rename = "EntryOffset")]
    entry_offset: u64,
    #[serde(rename = "FromMirror")]
    from_mirror: bool,
    #[serde(rename = "Size")]
    size: u32,
    #[serde(rename = "Owner")]
    owner: Option<String>,
    #[serde(rename = "OwnerName")]
    owner_name: Option<String>,
    #[serde(rename = "Group")]
    group: Option<String>,
    #[serde(rename = "GroupName")]
    group_name: Option<String>,
    #[serde(rename = "Control")]
    control: String,
    #[serde(rename = "DaclAceCount")]
    dacl_ace_count: Option<usize>,
    #[serde(rename = "Dacl")]
    dacl: Option<String>,
    #[serde(rename = "SaclAceCount")]
    sacl_ace_count: Option<usize>,
    #[serde(rename = "Sacl")]
    sacl: Option<String>,
    #[serde(rename = "SDDL")]
    sddl: Option<String>,
    #[serde(rename = "Problems")]
    problems: Vec<String>,
    /// For the summary.
    #[serde(skip)]
    null_dacl: bool,
}

const CSV_HEADER: [&str; 17] = [
    "SourcePath",
    "SecurityID",
    "Hash",
    "EntryOffset",
    "FromMirror",
    "Size",
    "Owner",
    "OwnerName",
    "Group",
    "GroupName",
    "Control",
    "DaclAceCount",
    "Dacl",
    "SaclAceCount",
    "Sacl",
    "SDDL",
    "Problems",
];

impl Record {
    fn new(source: &str, entry: &Entry<'_>) -> Self {
        let mut record = Record {
            source_path: source.to_string(),
            security_id: entry.security_id,
            hash: format!("{:08X}", entry.hash),
            entry_offset: entry.position,
            from_mirror: entry.from_mirror,
            size: entry.size,
            owner: None,
            owner_name: None,
            group: None,
            group_name: None,
            control: String::new(),
            dacl_ace_count: None,
            dacl: None,
            sacl_ace_count: None,
            sacl: None,
            sddl: None,
            problems: Vec::new(),
            null_dacl: false,
        };
        if !entry.hash_matches() {
            record
                .problems
                .push("the recorded hash is not the descriptor's".into());
        }
        match entry.security_descriptor() {
            Ok(sd) => record.describe(&sd),
            Err(e) => record
                .problems
                .push(format!("the security descriptor cannot be read: {e}")),
        }
        record
    }

    fn describe(&mut self, sd: &SecurityDescriptor<'_>) {
        (self.owner, self.owner_name) = account(&sd.owner);
        (self.group, self.group_name) = account(&sd.group);
        self.control = sd.control.names().join("|");
        self.null_dacl = matches!(sd.dacl, Ok(Acl::Null));
        (self.dacl_ace_count, self.dacl) = acl_text(&sd.dacl, "DACL", &mut self.problems);
        (self.sacl_ace_count, self.sacl) = acl_text(&sd.sacl, "SACL", &mut self.problems);
        self.sddl = sd.sddl();
        let mut problems = sd.problems();
        problems.append(&mut self.problems);
        self.problems = problems;
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: &Option<String>| v.clone().unwrap_or_default();
        let num = |v: Option<usize>| v.map(|n| n.to_string()).unwrap_or_default();
        vec![
            self.source_path.clone(),
            self.security_id.to_string(),
            self.hash.clone(),
            self.entry_offset.to_string(),
            if self.from_mirror { "True" } else { "False" }.to_string(),
            self.size.to_string(),
            text(&self.owner),
            text(&self.owner_name),
            text(&self.group),
            text(&self.group_name),
            self.control.clone(),
            num(self.dacl_ace_count),
            text(&self.dacl),
            num(self.sacl_ace_count),
            text(&self.sacl),
            text(&self.sddl),
            self.problems.join("; "),
        ]
    }
}

/// An owner or group: its SID and well-known name.
fn account(sid: &Result<Option<Sid>, Error>) -> (Option<String>, Option<String>) {
    match sid {
        Ok(Some(sid)) => (
            Some(sid.to_string()),
            sid.well_known_name().map(str::to_string),
        ),
        _ => (None, None),
    }
}

/// An ACL as text, its entries separated by `; `, and how many entries it has. `None` if absent
/// or not decoded.
fn acl_text(
    acl: &Result<Acl<'_>, Error>,
    name: &str,
    problems: &mut Vec<String>,
) -> (Option<usize>, Option<String>) {
    match acl {
        Ok(Acl::Absent) | Err(_) => (None, None),
        Ok(Acl::Null) => (
            None,
            Some(if name == "DACL" {
                "NULL DACL: no restriction".to_string()
            } else {
                "NULL SACL".to_string()
            }),
        ),
        Ok(Acl::Entries { aces, .. }) => {
            let entries: Vec<String> = aces
                .iter()
                .enumerate()
                .map(|(i, ace)| {
                    let (text, problem) = ace_text(ace);
                    if let Some(problem) = problem {
                        problems.push(format!("{name} entry {}: {problem}", i + 1));
                    }
                    text
                })
                .collect();
            (Some(aces.len()), Some(entries.join("; ")))
        }
    }
}

/// File rights icacls names, from the most to the least.
const SIMPLE_RIGHTS: [(u32, &str); 5] = [
    (0x001f_01ff, "F"),
    (0x0013_01bf, "M"),
    (0x0012_00a9, "RX"),
    (0x0012_0089, "R"),
    (0x0010_0116, "W"),
];

const SPECIFIC_RIGHTS: [(u32, &str); 20] = [
    (0x0000_0001, "RD"),
    (0x0000_0002, "WD"),
    (0x0000_0004, "AD"),
    (0x0000_0008, "REA"),
    (0x0000_0010, "WEA"),
    (0x0000_0020, "X"),
    (0x0000_0040, "DC"),
    (0x0000_0080, "RA"),
    (0x0000_0100, "WA"),
    (0x0001_0000, "DE"),
    (0x0002_0000, "RC"),
    (0x0004_0000, "WDAC"),
    (0x0008_0000, "WO"),
    (0x0010_0000, "S"),
    (0x0100_0000, "AS"),
    (0x0200_0000, "MA"),
    (0x1000_0000, "GA"),
    (0x2000_0000, "GE"),
    (0x4000_0000, "GW"),
    (0x8000_0000, "GR"),
];

/// A file access mask as icacls writes it: `F`, `M`, `RX`, `R`, `W`, or the specific rights
/// (`RX,WD`, `GA`); `N` for none.
fn file_rights(mask: u32) -> String {
    if mask == 0 {
        return "N".into();
    }
    let mut names = Vec::new();
    let mut covered = 0;
    for (bits, name) in SIMPLE_RIGHTS {
        if mask & bits == bits && covered & bits != bits {
            names.push(name.to_string());
            covered |= bits;
        }
    }
    let mut rest = mask & !covered;
    for (bit, name) in SPECIFIC_RIGHTS {
        if rest & bit != 0 {
            names.push(name.to_string());
            rest &= !bit;
        }
    }
    if rest != 0 {
        names.push(format!("0x{rest:X}"));
    }
    names.join(",")
}

/// A mandatory label's policy: `NW` (no write up), `NR`, `NX`.
fn label_policy(mask: u32) -> String {
    let mut names: Vec<String> = [(1, "NW"), (2, "NR"), (4, "NX")]
        .iter()
        .filter(|(bit, _)| mask & bit != 0)
        .map(|(_, name)| (*name).to_string())
        .collect();
    if mask & !7 != 0 {
        names.push(format!("0x{:X}", mask & !7));
    }
    names.join(",")
}

/// ACE flags as icacls writes them: `(I)(OI)(CI)`, and the audit flags `(SA)(FA)`.
const ACE_FLAGS: [(u8, &str); 8] = [
    (0x10, "I"),
    (0x01, "OI"),
    (0x02, "CI"),
    (0x08, "IO"),
    (0x04, "NP"),
    (0x40, "SA"),
    (0x80, "FA"),
    (0x20, "CR"),
];

/// An ACE as icacls writes it, `BUILTIN\Users:(I)(OI)(CI)(RX)`, with its type when not
/// `ACCESS_ALLOWED` (`(DENY)`, `(AUDIT)`, `(ACCESS_ALLOWED_CALLBACK)`, …), a callback ACE's
/// condition after `if`, and what in it is not decoded.
fn ace_text(ace: &Ace<'_>) -> (String, Option<String>) {
    let Some(data) = &ace.data else {
        return (
            format!(
                "{} (type 0x{:02X}, flags 0x{:02X}): not decoded",
                ace.type_name(),
                ace.ace_type,
                ace.flags
            ),
            Some(format!(
                "{} ACE (type 0x{:02X}) not decoded",
                ace.type_name(),
                ace.ace_type
            )),
        );
    };
    let mut text = data
        .sid
        .well_known_name()
        .map_or_else(|| data.sid.to_string(), str::to_string);
    text.push(':');
    match ace.ace_type {
        0 | 17 => {}
        1 => text.push_str("(DENY)"),
        2 => text.push_str("(AUDIT)"),
        3 => text.push_str("(ALARM)"),
        _ => text.push_str(&format!("({})", ace.type_name())),
    }
    for (bit, flag) in ACE_FLAGS {
        if ace.flags & bit != 0 {
            text.push_str(&format!("({flag})"));
        }
    }
    let rights = match ace.ace_type {
        0..=3 | 9 | 10 | 13 | 14 => file_rights(data.mask),
        17 => label_policy(data.mask),
        _ => format!("0x{:08X}", data.mask),
    };
    text.push_str(&format!("({rights})"));
    if let Some(object) = data.object_type {
        text.push_str(&format!(" object {object}"));
    }
    if let Some(inherited) = data.inherited_object_type {
        text.push_str(&format!(" inherited object {inherited}"));
    }
    let mut problem = None;
    match ace.ace_type {
        9..=16 => match ace.condition() {
            Some(condition) => text.push_str(&format!(" if {condition}")),
            None => {
                text.push_str(" if <condition not decoded>");
                problem = Some("its condition is not decoded".to_string());
            }
        },
        18 => match ace.resource_attribute() {
            Some(claim) => text.push_str(&format!(" {claim}")),
            None => {
                text.push_str(" <claim not decoded>");
                problem = Some("its claim is not decoded".to_string());
            }
        },
        _ if !data.application_data.is_empty() => {
            text.push_str(&format!(
                " +{} bytes of application data",
                data.application_data.len()
            ));
        }
        _ => {}
    }
    (text, problem)
}

// --- summary ----------------------------------------------------------------------------------

/// Counts for the results summary.
#[derive(Default)]
struct Stats {
    streams: usize,
    failed: usize,
    descriptors: usize,
    from_mirror: usize,
    hash_mismatches: usize,
    with_problems: usize,
    /// (source, security ID) of descriptors with a NULL DACL.
    null_dacls: Vec<(String, u32)>,
    /// Owner (SID, name) → descriptors.
    owners: HashMap<(String, Option<String>), usize>,
    stream_problems: Vec<String>,
}

impl Stats {
    fn fail(&mut self, report: &impl Fn(&str), msg: &str) {
        self.failed += 1;
        if self.failed <= MAX_PRINTED_ERRORS {
            report(msg);
        }
    }

    fn stream_problem(&mut self, problem: String) {
        self.stream_problems.push(problem);
    }

    fn add(&mut self, r: &Record) {
        self.descriptors += 1;
        self.from_mirror += usize::from(r.from_mirror);
        self.hash_mismatches += usize::from(
            r.problems
                .iter()
                .any(|p| p == "the recorded hash is not the descriptor's"),
        );
        self.with_problems += usize::from(!r.problems.is_empty());
        if r.null_dacl {
            self.null_dacls.push((r.source_path.clone(), r.security_id));
        }
        if let Some(owner) = &r.owner {
            *self
                .owners
                .entry((owner.clone(), r.owner_name.clone()))
                .or_default() += 1;
        }
    }

    fn print(&self, no_color: bool, several: bool) {
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        p_label(no_color, "$SDS streams parsed: ", &fmt_num(self.streams));
        if self.failed > 0 {
            p_label(no_color, "Could not parse: ", &fmt_num(self.failed));
        }
        let mut descriptors = fmt_num(self.descriptors);
        if self.from_mirror > 0 {
            descriptors.push_str(&format!(
                " (copies from the mirror that differ, or of damaged entries: {})",
                fmt_num(self.from_mirror)
            ));
        }
        p_label(no_color, "Security descriptors: ", &descriptors);
        if self.hash_mismatches > 0 {
            p_label(
                no_color,
                "Descriptors whose recorded hash is not theirs: ",
                &fmt_num(self.hash_mismatches),
            );
        }
        let mut owners: Vec<_> = self.owners.iter().collect();
        owners.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        p_label(no_color, "Owners: ", &fmt_num(owners.len()));
        for ((sid, name), count) in owners.iter().take(MAX_PRINTED) {
            let who = name
                .as_ref()
                .map_or_else(|| sid.clone(), |name| format!("{name} ({sid})"));
            p(None, &format!("  {who}: {}", fmt_num(**count)), true);
        }
        if owners.len() > MAX_PRINTED {
            p(
                None,
                &format!("  … {} more", fmt_num(owners.len() - MAX_PRINTED)),
                true,
            );
        }
        if !self.null_dacls.is_empty() {
            p_label(
                no_color,
                "Descriptors with a NULL DACL (no restriction): ",
                &fmt_num(self.null_dacls.len()),
            );
            for (source, id) in self.null_dacls.iter().take(MAX_PRINTED) {
                let line = if several {
                    format!("  {source}: security ID {id}")
                } else {
                    format!("  security ID {id}")
                };
                p(None, &line, true);
            }
        }
        if self.with_problems > 0 {
            p_label(
                no_color,
                "Descriptors with problems: ",
                &fmt_num(self.with_problems),
            );
        }
        if !self.stream_problems.is_empty() {
            p_label(
                no_color,
                "Problems in streams: ",
                &fmt_num(self.stream_problems.len()),
            );
            for problem in self.stream_problems.iter().take(MAX_PRINTED) {
                p(None, &format!("  {problem}"), true);
            }
        }
        p(None, "", true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rights_as_icacls_writes_them() {
        assert_eq!(file_rights(0x001f_01ff), "F");
        assert_eq!(file_rights(0x0013_01bf), "M");
        assert_eq!(file_rights(0x0012_00a9), "RX");
        assert_eq!(file_rights(0x0012_0089), "R");
        assert_eq!(file_rights(0x0012_01bf), "RX,W");
        assert_eq!(file_rights(0x1000_0000), "GA");
        assert_eq!(file_rights(0xa000_0000), "GE,GR");
        assert_eq!(file_rights(0x0010_0004), "AD,S");
        assert_eq!(file_rights(0x0000_0200), "0x200");
        assert_eq!(file_rights(0), "N");
        assert_eq!(label_policy(1), "NW");
        assert_eq!(label_policy(0x13), "NW,NR,0x10");
    }

    #[test]
    fn sds_names() {
        assert!(is_sds_name("$Secure_$SDS"));
        assert!(is_sds_name("$sds"));
        assert!(is_sds_name("$Secure:$SDS"));
        assert!(!is_sds_name("$Secure_$SDH"));
    }
}
