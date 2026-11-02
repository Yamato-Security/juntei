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

//! `parse-registry bam`: when each user last ran each program, as the Background and Desktop
//! Activity Moderators (BAM, DAM) recorded it in `SYSTEM` hives, read by ys-registry-parser's
//! `system` module, with Hayabusa's input options, progress bar and results summary. One record
//! per value of each user's key under `Services\bam` and `Services\dam` (Windows 10 1709 and
//! later; both key layouts), in the control set Windows used last (`Select\Current`) or in every
//! control set (`--all-control-sets`): the program, the device its path is on, and the time it
//! last ran. A user's name comes from the system's `SAM` (local accounts) or `ProfileList`
//! (profile folders), or from the well-known SIDs. The summary lists the users and the programs
//! run last.

use super::{
    Hosts, Logs, Opened, RecordWriter, choose_logs, files_read, filetime, find_hives_of_type,
    input_files, open, sink, software_of, system_hive_of,
};
use crate::cli::RegistryBamOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::system::{BamEntry, ControlSets, Moderator, bam, control_set_name, device_path};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io;
use std::path::{Path, PathBuf};

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems, users and programs listed in the summary.
const MAX_PRINTED: usize = 20;

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryBamOptions) -> i32 {
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
        "SYSTEM hives",
        |d| find_hives_of_type(d, "SYSTEM"),
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
        p_label(no_color, "SYSTEM hives: ", &fmt_num(files.len()));
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
    let mut users = UserNames::new(!opts.no_logs);
    let mut stats = Stats::default();
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        let parsed = open(path, logs).and_then(|mut opened| {
            if !opened.hive_type.is_empty() && opened.hive_type != "SYSTEM" {
                return Err(format!("a {} hive, not a SYSTEM hive", opened.hive_type));
            }
            opened.host = hosts.of(path, &opened);
            let sets = ControlSets::read(&opened.hive);
            if sets.present.is_empty() {
                return Err("no control set (ControlSet001, …): not a SYSTEM hive".to_string());
            }
            Ok((opened, sets))
        });
        match parsed {
            Ok((opened, sets)) => {
                let names = users.of(path);
                if let Err(e) = stats.hive(&opened, &sets, names, opts, writer.as_mut()) {
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
        stats.print(opts, no_color);
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

// --- user names ---------------------------------------------------------------------------------

/// The name of a well-known SID a BAM key can be named after: the accounts services and the
/// sign-in screen's processes run as (Microsoft's well-known SIDs: `S-1-5-18` Local System,
/// `S-1-5-19` Local Service, `S-1-5-20` Network Service; `S-1-5-90-0-<session>` the Desktop
/// Window Manager's `DWM-<session>`, `S-1-5-96-0-<session>` the font driver host's
/// `UMFD-<session>`).
fn well_known(sid: &str) -> Option<String> {
    match sid.to_ascii_uppercase().as_str() {
        "S-1-5-18" => return Some("SYSTEM".to_owned()),
        "S-1-5-19" => return Some("LOCAL SERVICE".to_owned()),
        "S-1-5-20" => return Some("NETWORK SERVICE".to_owned()),
        _ => {}
    }
    [("S-1-5-90-0-", "DWM-"), ("S-1-5-96-0-", "UMFD-")]
        .iter()
        .find_map(|(prefix, name)| {
            let session = sid.get(prefix.len()..)?;
            (sid[..prefix.len()].eq_ignore_ascii_case(prefix)
                && !session.is_empty()
                && session.bytes().all(|b| b.is_ascii_digit()))
            .then(|| format!("{name}{session}"))
        })
}

/// The names one system gives its users' SIDs: its local accounts (`SAM`: the computer's SID and
/// each account's RID) and its profiles' folders (`SOFTWARE`'s `ProfileList`).
#[derive(Default)]
struct Names {
    /// Local accounts: SID (upper case) → user name.
    accounts: HashMap<String, String>,
    /// Profiles: SID (upper case) → profile folder name.
    profiles: HashMap<String, String>,
}

impl Names {
    /// The name of `sid`, and where it comes from: `well_known`, `sam`, `profile_list`.
    fn of(&self, sid: &str) -> Option<(String, &'static str)> {
        let key = sid.to_ascii_uppercase();
        well_known(sid)
            .map(|name| (name, "well_known"))
            .or_else(|| self.accounts.get(&key).map(|name| (name.clone(), "sam")))
            .or_else(|| {
                self.profiles
                    .get(&key)
                    .map(|name| (name.clone(), "profile_list"))
            })
    }
}

/// The `SAM` and `SOFTWARE` hives of the systems `SYSTEM` hives come from (found as [`Hosts`]
/// finds a system's hives), each system's names read once.
struct UserNames {
    logs: bool,
    read: HashMap<(Option<PathBuf>, Option<PathBuf>), Names>,
}

impl UserNames {
    /// `logs`: whether `SAM` and `SOFTWARE` hives get their logs replayed.
    fn new(logs: bool) -> Self {
        UserNames {
            logs,
            read: HashMap::new(),
        }
    }

    /// The names of the users of the system the `SYSTEM` hive at `path` comes from.
    fn of(&mut self, path: &Path) -> &Names {
        let sam = system_hive_of(path, "SAM");
        let software = software_of(path);
        let logs = if self.logs {
            Logs::NextToIt
        } else {
            Logs::Without
        };
        self.read
            .entry((sam.clone(), software.clone()))
            .or_insert_with(|| {
                let opened = |path: Option<PathBuf>, kind: &str| {
                    path.and_then(|p| open(&p, logs).ok())
                        .filter(|o| o.hive_type == kind)
                };
                let mut names = Names::default();
                if let Some(sam) = opened(sam, "SAM")
                    && let Some(machine) = registry::sam::machine_sid(&sam.hive)
                {
                    for account in registry::sam::accounts(&sam.hive)
                        .map(|a| a.0)
                        .unwrap_or_default()
                    {
                        if let (Some(rid), Some(name)) = (account.rid, account.username) {
                            names
                                .accounts
                                .insert(format!("{machine}-{rid}").to_ascii_uppercase(), name);
                        }
                    }
                }
                if let Some(software) = opened(software, "SOFTWARE") {
                    for (sid, folder) in registry::software::profile_list(&software.hive) {
                        if let Some(name) =
                            folder.rsplit(['\\', '/']).next().filter(|n| !n.is_empty())
                        {
                            names
                                .profiles
                                .insert(sid.to_ascii_uppercase(), name.to_owned());
                        }
                    }
                }
                names
            })
    }
}

// --- records ----------------------------------------------------------------------------------

/// An entry's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    #[serde(rename = "Host")]
    host: Option<&'a str>,
    #[serde(rename = "SourcePath")]
    source_path: &'a str,
    #[serde(rename = "ControlSet")]
    control_set: String,
    #[serde(rename = "IsCurrentControlSet")]
    is_current_control_set: bool,
    #[serde(rename = "Moderator")]
    moderator: &'static str,
    #[serde(rename = "UserSID")]
    user_sid: &'a str,
    #[serde(rename = "UserName")]
    user_name: Option<String>,
    #[serde(rename = "UserNameSource")]
    user_name_source: Option<&'static str>,
    #[serde(rename = "Program")]
    program: &'a str,
    #[serde(rename = "DevicePath")]
    device_path: Option<&'a str>,
    #[serde(rename = "Path")]
    path: Option<&'a str>,
    #[serde(rename = "LastRun")]
    last_run: Option<String>,
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

const CSV_HEADER: [&str; 18] = [
    "Host",
    "SourcePath",
    "ControlSet",
    "IsCurrentControlSet",
    "Moderator",
    "UserSID",
    "UserName",
    "UserNameSource",
    "Program",
    "DevicePath",
    "Path",
    "LastRunUTC",
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
    fn new(
        opened: &'a Opened,
        set: u32,
        current: bool,
        names: &Names,
        entry: &'a BamEntry,
    ) -> Self {
        let (user_name, user_name_source) = names
            .of(&entry.user_sid)
            .map_or((None, None), |(name, from)| (Some(name), Some(from)));
        let split = device_path(&entry.program);
        Record {
            host: opened.host.as_deref(),
            source_path: &opened.source,
            control_set: control_set_name(set),
            is_current_control_set: current,
            moderator: entry.moderator.name(),
            user_sid: &entry.user_sid,
            user_name,
            user_name_source,
            program: &entry.program,
            device_path: split.map(|(device, _)| device),
            path: split.map(|(_, path)| path),
            last_run: entry.last_run.and_then(filetime),
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
        let flag = |b: bool| if b { "True" } else { "False" }.to_string();
        vec![
            text(self.host),
            self.source_path.to_string(),
            self.control_set.clone(),
            flag(self.is_current_control_set),
            self.moderator.to_string(),
            self.user_sid.to_string(),
            text(self.user_name.as_deref()),
            text(self.user_name_source),
            self.program.to_string(),
            text(self.device_path),
            text(self.path),
            text(self.last_run.as_deref()),
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
    hives: usize,
    failed: usize,
    hosts: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    without_select: Vec<String>,
    control_sets: usize,
    /// Control sets read that have no BAM or DAM key (Windows before 10 1709, or turned off).
    without_keys: usize,
    entries: usize,
    by_moderator: BTreeMap<&'static str, usize>,
    /// Each user's SID (with its host), its name and how many entries it has.
    users: BTreeMap<(String, String), (Option<String>, usize)>,
    /// The programs run last: (last run, line), each once even when several control sets have it.
    latest: BTreeSet<(u64, String, String, String)>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened`'s BAM and DAM entries (when there is a writer) and count
    /// them.
    fn hive(
        &mut self,
        opened: &Opened,
        sets: &ControlSets,
        names: &Names,
        opts: &RegistryBamOptions,
        mut writer: Option<&mut RecordWriter>,
    ) -> io::Result<()> {
        self.hives += 1;
        self.hosts.extend(opened.host.clone());
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
        // without a usable Select\Current, the first control set is the best available fallback
        if sets
            .current
            .is_none_or(|current| !sets.present.contains(&current))
        {
            self.without_select.push(opened.source.clone());
        }
        let read: Vec<u32> = if opts.all_control_sets {
            sets.present.clone()
        } else {
            sets.current_or_first().into_iter().collect()
        };
        for set in read {
            self.control_sets += 1;
            let Some((mut entries, problems)) = bam(&opened.hive, set) else {
                self.without_keys += 1;
                continue;
            };
            for problem in &problems {
                self.problem(&opened.source, problem);
            }
            for entry in &mut entries {
                if let Some(when) = entry.last_run
                    && when != 0
                    && filetime(when).is_none()
                {
                    entry.problems.push(format!("last-run FILETIME {when} is outside the supported date range; original data retained"));
                    if let Some(value) = opened
                        .hive
                        .key(&entry.key_path)
                        .and_then(|k| k.value(&entry.program))
                    {
                        entry.data_raw = Some(value.read_data().0);
                    }
                    entry.last_run = None;
                }
                self.entry(opened, names, entry);
                let record = Record::new(opened, set, sets.current == Some(set), names, entry);
                match writer.as_deref_mut() {
                    Some(w) if w.is_csv() => w.csv(&record.csv())?,
                    Some(w) => w.json(&record)?,
                    None => {}
                }
            }
        }
        Ok(())
    }

    /// Count an entry written.
    fn entry(&mut self, opened: &Opened, names: &Names, entry: &BamEntry) {
        self.entries += 1;
        *self.by_moderator.entry(entry.moderator.name()).or_default() += 1;
        let name = names.of(&entry.user_sid).map(|(name, _)| name);
        let host = opened.host.clone().unwrap_or_default();
        let user = self
            .users
            .entry((host, entry.user_sid.clone()))
            .or_insert((None, 0));
        user.0 = name.clone();
        user.1 += 1;
        for problem in &entry.problems {
            self.problem(
                &opened.source,
                &format!(r"{}\{}: {problem}", entry.key_path, entry.program),
            );
        }
        // only times that are times
        if let Some((when, at)) = entry
            .last_run
            .and_then(|when| Some((when, filetime(when)?)))
        {
            let who = name.unwrap_or_else(|| entry.user_sid.clone());
            let moderator = if entry.moderator == Moderator::Dam {
                " (dam)"
            } else {
                ""
            };
            let host = opened.host.as_deref().unwrap_or(&opened.source);
            self.latest.insert((
                when,
                opened.source.clone(),
                entry.user_sid.clone(),
                format!("{host}  {at}  {who}  {}{moderator}", entry.program),
            ));
            // keep the most recent only
            while self.latest.len() > MAX_PRINTED {
                self.latest.pop_first();
            }
        }
    }

    fn problem(&mut self, source: &str, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED {
            self.problems.push(format!("{source}: {problem}"));
        }
    }

    fn print(&self, opts: &RegistryBamOptions, no_color: bool) {
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        p_label(no_color, "SYSTEM hives parsed: ", &fmt_num(self.hives));
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
                "Dirty hives whose transaction logs were not applied (entries may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if !self.without_select.is_empty() {
            p_label(
                no_color,
                "Hives without usable Select\\Current (the first control set was read): ",
                &fmt_num(self.without_select.len()),
            );
        }
        p_label(
            no_color,
            "Control sets read: ",
            &format!(
                "{}{}",
                fmt_num(self.control_sets),
                if opts.all_control_sets {
                    ""
                } else {
                    " (the one in use; --all-control-sets for every one)"
                }
            ),
        );
        if self.without_keys > 0 {
            p_label(
                no_color,
                "Control sets without BAM or DAM keys (before Windows 10 1709, or turned off): ",
                &fmt_num(self.without_keys),
            );
        }
        let moderators: Vec<String> = self
            .by_moderator
            .iter()
            .map(|(m, n)| format!("{m} {}", fmt_num(*n)))
            .collect();
        p_label(
            no_color,
            "Entries: ",
            &if moderators.is_empty() {
                fmt_num(self.entries)
            } else {
                format!("{} ({})", fmt_num(self.entries), moderators.join(", "))
            },
        );
        if !self.users.is_empty() {
            p_label(no_color, "Users (SID, name, entries): ", "");
            for ((host, sid), (name, n)) in self.users.iter().take(MAX_PRINTED) {
                let host = if host.is_empty() {
                    String::new()
                } else {
                    format!("{host}  ")
                };
                let name = name.as_deref().unwrap_or("(unknown)");
                p(
                    None,
                    &format!("  {host}{sid}  {name}  {}", fmt_num(*n)),
                    true,
                );
            }
            if self.users.len() > MAX_PRINTED {
                p(
                    None,
                    &format!("  … {} more", fmt_num(self.users.len() - MAX_PRINTED)),
                    true,
                );
            }
        }
        if !self.latest.is_empty() {
            p_label(no_color, "Run last (UTC, user, program): ", "");
            for (_, _, _, line) in self.latest.iter().rev() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_known_sids() {
        assert_eq!(well_known("S-1-5-18").as_deref(), Some("SYSTEM"));
        assert_eq!(well_known("s-1-5-19").as_deref(), Some("LOCAL SERVICE"));
        assert_eq!(well_known("S-1-5-20").as_deref(), Some("NETWORK SERVICE"));
        assert_eq!(well_known("S-1-5-90-0-2").as_deref(), Some("DWM-2"));
        assert_eq!(well_known("S-1-5-96-0-11").as_deref(), Some("UMFD-11"));
        assert_eq!(well_known("S-1-5-90-0-"), None);
        assert_eq!(well_known("S-1-5-90-0-1x"), None);
        assert_eq!(well_known("S-1-5-21-1-2-3-500"), None);
        assert_eq!(well_known("S-1-5-9"), None);
    }

    #[test]
    fn names_prefer_local_accounts_to_profile_folders() {
        let mut names = Names::default();
        names
            .accounts
            .insert("S-1-5-21-1-2-3-1001".into(), "alice".into());
        names
            .profiles
            .insert("S-1-5-21-1-2-3-1001".into(), "alice.OLD".into());
        names
            .profiles
            .insert("S-1-5-21-9-9-9-500".into(), "Administrator".into());
        names
            .profiles
            .insert("S-1-5-18".into(), "systemprofile".into());
        assert_eq!(
            names.of("s-1-5-21-1-2-3-1001"),
            Some(("alice".into(), "sam"))
        );
        assert_eq!(
            names.of("S-1-5-21-9-9-9-500"),
            Some(("Administrator".into(), "profile_list"))
        );
        assert_eq!(names.of("S-1-5-18"), Some(("SYSTEM".into(), "well_known")));
        assert_eq!(names.of("S-1-5-21-1-2-3-1000"), None);
    }
}
