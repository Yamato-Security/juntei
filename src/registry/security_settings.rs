//! `parse-registry security-settings`: the security settings attackers change, in SOFTWARE,
//! SYSTEM and NTUSER.DAT hives, read by ys-registry-parser's `security_settings` module, with
//! Hayabusa's input options, progress bar and results summary. One record per value found:
//! Microsoft Defender (each exclusion a record), UAC, credential protection (WDigest, LSA
//! protection, Credential Guard, Restricted Admin, NTLM), Remote Desktop, the firewall and SMBv1,
//! PowerShell (execution policy, logging, AMSI) and what keeps evidence (last access times, System
//! Restore, event logs), each with what its value means and whether it weakens what Windows does
//! by default. The summary lists the weakened settings and the Defender exclusions.

use super::{
    Hosts, Opened, Profiles, RecordWriter, choose_logs, files_read, filetime, find_hives,
    hive_type_of_file, input_files, open, profile_of_path, sink,
};
use crate::cli::RegistrySecuritySettingsOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::ntuser::profile_name;
use registry::security_settings::{self, Setting};
use registry::system::{ControlSets, control_set_name};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems, weakened settings and exclusions listed in the summary.
const MAX_PRINTED: usize = 20;
/// The hives read, as their base blocks name them.
const HIVE_TYPES: [&str; 3] = ["SOFTWARE", "SYSTEM", "NTUSER.DAT"];

/// Run the command; returns the exit status.
pub fn run(opts: &RegistrySecuritySettingsOptions) -> i32 {
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
        "SOFTWARE, SYSTEM or NTUSER.DAT hives",
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
        p_label(no_color, "Registry hives: ", &fmt_num(files.len()));
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
    let mut stats = Stats::default();
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        let parsed = open(path, logs).and_then(|mut opened| {
            if !HIVE_TYPES.contains(&opened.hive_type.as_str()) {
                return Err(if opened.hive_type.is_empty() {
                    "not a SOFTWARE, SYSTEM or NTUSER.DAT hive (neither its base block nor its file name says which)".to_string()
                } else {
                    format!(
                        "a {} hive, not a SOFTWARE, SYSTEM or NTUSER.DAT hive",
                        opened.hive_type
                    )
                });
            }
            opened.host = hosts.of(path, &opened);
            let read = Read::of(path, &opened, opts, &mut profiles)?;
            Ok((opened, read))
        });
        match parsed {
            Ok((opened, read)) => {
                if let Err(e) = stats.hive(&opened, &read, opts, writer.as_mut()) {
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
    i32::from(stats.hives.values().sum::<usize>() == 0)
}

// --- reading ----------------------------------------------------------------------------------

/// The settings of one hive: per control set for a SYSTEM hive, with the user of an NTUSER.DAT.
struct Read {
    /// Each control set read (`None` for other hives), whether it is the current one, and its
    /// settings.
    sets: Vec<(Option<u32>, bool, Vec<Setting>)>,
    problems: Vec<String>,
    user: Option<String>,
    user_sid: Option<String>,
    /// A SYSTEM hive without a usable `Select\Current`: its first control set was read.
    without_select: bool,
}

impl Read {
    fn of(
        path: &Path,
        opened: &Opened,
        opts: &RegistrySecuritySettingsOptions,
        profiles: &mut Profiles,
    ) -> Result<Self, String> {
        let hive = &opened.hive;
        let mut read = Read {
            sets: Vec::new(),
            problems: Vec::new(),
            user: None,
            user_sid: None,
            without_select: false,
        };
        match opened.hive_type.as_str() {
            "SOFTWARE" => {
                let (settings, problems) = security_settings::software(hive);
                read.sets.push((None, false, settings));
                read.problems = problems;
            }
            "SYSTEM" => {
                let sets = ControlSets::read(hive);
                if sets.present.is_empty() {
                    return Err("no control set (ControlSet001, …): not a SYSTEM hive".to_string());
                }
                read.without_select = sets
                    .current
                    .is_none_or(|current| !sets.present.contains(&current));
                let chosen: Vec<u32> = if opts.all_control_sets {
                    sets.present.clone()
                } else {
                    sets.current_or_first().into_iter().collect()
                };
                for set in chosen {
                    if let Some((settings, problems)) = security_settings::system(hive, set) {
                        read.sets
                            .push((Some(set), sets.current == Some(set), settings));
                        read.problems.extend(problems);
                    }
                }
            }
            _ => {
                read.user = profile_of_path(path).or_else(|| profile_name(hive));
                read.user_sid = read.user.as_deref().and_then(|u| profiles.sid(path, u));
                read.sets
                    .push((None, false, security_settings::ntuser(hive)));
            }
        }
        Ok(read)
    }
}

// --- records ----------------------------------------------------------------------------------

/// A setting's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    host: Option<&'a str>,
    source_path: &'a str,
    hive_type: &'a str,
    control_set: Option<String>,
    is_current_control_set: Option<bool>,
    username: Option<&'a str>,
    user_sid: Option<&'a str>,
    area: &'static str,
    setting: &'static str,
    value: Option<&'a str>,
    meaning: &'a str,
    weakens_default: Option<bool>,
    policy: bool,
    key_path: &'a str,
    value_name: &'a str,
    key_last_written: Option<String>,
    data_raw: Option<String>,
    transaction_log_status: &'static str,
    logs_applied: &'a [String],
    problems: &'a [String],
}

const CSV_HEADER: [&str; 20] = [
    "Host",
    "SourcePath",
    "HiveType",
    "ControlSet",
    "IsCurrentControlSet",
    "Username",
    "UserSid",
    "Area",
    "Setting",
    "Value",
    "Meaning",
    "WeakensDefault",
    "Policy",
    "KeyPath",
    "ValueName",
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
        read: &'a Read,
        set: Option<u32>,
        current: bool,
        setting: &'a Setting,
    ) -> Self {
        Record {
            host: opened.host.as_deref(),
            source_path: &opened.source,
            hive_type: &opened.hive_type,
            control_set: set.map(control_set_name),
            is_current_control_set: set.map(|_| current),
            username: read.user.as_deref(),
            user_sid: read.user_sid.as_deref(),
            area: setting.area.name(),
            setting: setting.setting,
            value: setting.value.as_deref(),
            meaning: &setting.meaning,
            weakens_default: setting.weakens_default,
            policy: setting.policy,
            key_path: &setting.key_path,
            value_name: &setting.value_name,
            key_last_written: filetime(setting.key_last_written),
            data_raw: setting.data_raw.as_deref().map(hex),
            transaction_log_status: opened.log_status(setting.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &setting.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let flag = |b: bool| if b { "True" } else { "False" }.to_string();
        let maybe = |b: Option<bool>| b.map(flag).unwrap_or_default();
        vec![
            text(self.host),
            self.source_path.to_string(),
            self.hive_type.to_string(),
            text(self.control_set.as_deref()),
            maybe(self.is_current_control_set),
            text(self.username),
            text(self.user_sid),
            self.area.to_string(),
            self.setting.to_string(),
            text(self.value),
            self.meaning.to_string(),
            maybe(self.weakens_default),
            flag(self.policy),
            self.key_path.to_string(),
            self.value_name.to_string(),
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
    /// Hives read, by type.
    hives: BTreeMap<String, usize>,
    failed: usize,
    hosts: BTreeSet<String>,
    users: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    without_select: Vec<String>,
    control_sets: usize,
    settings: usize,
    by_area: BTreeMap<&'static str, usize>,
    /// Weakened settings: lines, each once (the same value in several control sets once).
    weakened: BTreeSet<String>,
    /// Defender exclusions: lines, each once.
    exclusions: BTreeSet<String>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `read` (when there is a writer) and count them.
    fn hive(
        &mut self,
        opened: &Opened,
        read: &Read,
        opts: &RegistrySecuritySettingsOptions,
        mut writer: Option<&mut RecordWriter>,
    ) -> io::Result<()> {
        *self.hives.entry(opened.hive_type.clone()).or_default() += 1;
        self.hosts.extend(opened.host.clone());
        self.users.extend(read.user.clone());
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
        if read.without_select {
            self.without_select.push(opened.source.clone());
        }
        for problem in &read.problems {
            self.problem(&opened.source, problem);
        }
        let host = opened.host.as_deref().unwrap_or(&opened.source);
        let user = read
            .user
            .as_deref()
            .map_or_else(String::new, |u| format!("{u}  "));
        for (set, current, settings) in &read.sets {
            if set.is_some() {
                self.control_sets += 1;
            }
            for setting in settings {
                if opts.weakened_only && setting.weakens_default != Some(true) {
                    continue;
                }
                self.settings += 1;
                *self.by_area.entry(setting.area.name()).or_default() += 1;
                for problem in &setting.problems {
                    self.problem(
                        &opened.source,
                        &format!(r"{}\{}: {problem}", setting.key_path, setting.value_name),
                    );
                }
                let value = setting.value.as_deref().unwrap_or("?");
                // the key path without its control set: one line for the same value in several
                let key = set.map_or(setting.key_path.as_str(), |_| {
                    setting
                        .key_path
                        .split_once('\\')
                        .map_or(setting.key_path.as_str(), |(_, rest)| rest)
                });
                if setting.setting.starts_with("defender_exclusion") {
                    self.exclusions.insert(format!(
                        "{host}  {}  {value}{}",
                        setting.setting.trim_start_matches("defender_exclusion_"),
                        if setting.policy { "  (policy)" } else { "" }
                    ));
                } else if setting.weakens_default == Some(true) {
                    self.weakened.insert(format!(
                        r"{host}  {user}{}  {key}\{} = {value}: {}",
                        setting.area.name(),
                        setting.value_name,
                        setting.meaning
                    ));
                }
                let record = Record::new(opened, read, *set, *current, setting);
                match writer.as_deref_mut() {
                    Some(w) if w.is_csv() => w.csv(&record.csv())?,
                    Some(w) => w.json(&record)?,
                    None => {}
                }
            }
        }
        Ok(())
    }

    fn problem(&mut self, source: &str, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED {
            self.problems.push(format!("{source}: {problem}"));
        }
    }

    fn print(&self, opts: &RegistrySecuritySettingsOptions, no_color: bool) {
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        let hives: Vec<String> = self
            .hives
            .iter()
            .map(|(kind, n)| format!("{kind} {}", fmt_num(*n)))
            .collect();
        p_label(
            no_color,
            "Hives parsed: ",
            &format!(
                "{}{}",
                fmt_num(self.hives.values().sum::<usize>()),
                if hives.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", hives.join(", "))
                }
            ),
        );
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
                "Dirty hives whose transaction logs were not applied (settings may be missing or out of date): ",
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
        if self.control_sets > 0 {
            p_label(
                no_color,
                "SYSTEM control sets read: ",
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
        }
        let areas: Vec<String> = self
            .by_area
            .iter()
            .map(|(area, n)| format!("{area} {}", fmt_num(*n)))
            .collect();
        p_label(
            no_color,
            if opts.weakened_only {
                "Weakened settings: "
            } else {
                "Settings: "
            },
            &if areas.is_empty() {
                fmt_num(self.settings)
            } else {
                format!("{} ({})", fmt_num(self.settings), areas.join(", "))
            },
        );
        p_label(
            no_color,
            "Settings that weaken the default (exclusions below; not proof of compromise): ",
            &fmt_num(self.weakened.len()),
        );
        for line in self.weakened.iter().take(MAX_PRINTED) {
            p(None, &format!("  {line}"), true);
        }
        if self.weakened.len() > MAX_PRINTED {
            p(
                None,
                &format!(
                    "  … {} more (WeakensDefault in -o <FILE>)",
                    fmt_num(self.weakened.len() - MAX_PRINTED)
                ),
                true,
            );
        }
        p_label(
            no_color,
            "Microsoft Defender exclusions: ",
            &fmt_num(self.exclusions.len()),
        );
        for line in self.exclusions.iter().take(MAX_PRINTED) {
            p(None, &format!("  {line}"), true);
        }
        if self.exclusions.len() > MAX_PRINTED {
            p(
                None,
                &format!(
                    "  … {} more (-o <FILE>)",
                    fmt_num(self.exclusions.len() - MAX_PRINTED)
                ),
                true,
            );
        }
        p_label(
            no_color,
            "Keys or values that could not be read: ",
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
