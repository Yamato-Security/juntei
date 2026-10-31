//! `parse-registry persistence`: autostart locations in SOFTWARE, SYSTEM, NTUSER.DAT and
//! UsrClass.dat hives, read by ys-registry-parser's `persistence` module, with Hayabusa's input
//! options, progress bar and results summary. One record per program, DLL or package a key or
//! value starts or loads: Run keys (with whether the Task Manager disabled an entry), Winlogon,
//! Image File Execution Options, AppInit and AppCert DLLs, Session Manager, LSA packages, print
//! monitors, netsh helpers, time providers, Active Setup, .NET profilers, a user's COM classes
//! and shell commands, and more, each with its MITRE ATT&CK technique. A user's COM class or
//! command is checked against the SOFTWARE hive of its system (as the host is found from its
//! SYSTEM hive): whether it overrides the machine's registration of the same one.

use super::{
    Hosts, Logs, Opened, RecordWriter, choose_logs, filetime, find_hives, find_in,
    hive_type_of_file, input_files, open, profile_of_path, sink, system_root,
};
use crate::cli::RegistryPersistenceOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::ntuser::{classes_sid, profile_name};
use registry::persistence::{self, Autostart, MachineClasses};
use registry::software::profile_list;
use registry::system::ControlSets;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems and notable records listed in the summary.
const MAX_PRINTED: usize = 20;
/// The hives read, as their base blocks name them.
const HIVE_TYPES: [&str; 4] = ["SOFTWARE", "SYSTEM", "NTUSER.DAT", "USRCLASS.DAT"];

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryPersistenceOptions) -> i32 {
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
        "SOFTWARE, SYSTEM, NTUSER.DAT or UsrClass.dat hives",
        |d| {
            find_hives(d)
                .into_iter()
                .filter(|p| hive_type_of_file(p).is_some_and(|t| HIVE_TYPES.contains(&t.as_str())))
                .collect()
        },
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
    let mut machines = Machines::new(!opts.no_logs);
    let mut stats = Stats::default();
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        let parsed = open(path, logs).and_then(|mut opened| {
            if !HIVE_TYPES.contains(&opened.hive_type.as_str()) {
                return Err(if opened.hive_type.is_empty() {
                    "not a SOFTWARE, SYSTEM, NTUSER.DAT or UsrClass.dat hive (neither its base block nor its file name says which)".to_string()
                } else {
                    format!(
                        "a {} hive, not a SOFTWARE, SYSTEM, NTUSER.DAT or UsrClass.dat hive",
                        opened.hive_type
                    )
                });
            }
            opened.host = hosts.of(path, &opened);
            Ok(opened)
        });
        let written = parsed.and_then(|opened| {
            let read = Read::of(path, &opened, opts, &mut machines)?;
            Ok(stats.hive(&opened, &read, writer.as_mut()))
        });
        match written {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                report_error(
                    bar.as_ref(),
                    &format!("Could not write the records of {}: {e}", path.display()),
                );
                return 1;
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

// --- the machine's SOFTWARE hive ----------------------------------------------------------------

/// What a user hive's records take from the SOFTWARE hive of its system.
struct Machine {
    /// Its COM classes and commands: which a user's override.
    classes: MachineClasses,
    /// `ProfileList`: each SID with its profile's folder.
    profiles: Vec<(String, String)>,
}

impl Machine {
    /// The SID of the profile whose folder is named `user`.
    fn sid_of(&self, user: &str) -> Option<String> {
        self.profiles
            .iter()
            .find(|(_, path)| {
                path.rsplit(['\\', '/'])
                    .next()
                    .is_some_and(|folder| folder.eq_ignore_ascii_case(user))
            })
            .map(|(sid, _)| sid.clone())
    }
}

/// The SOFTWARE hives of the systems user hives come from, each read once: the system's
/// `Windows\System32\config\SOFTWARE` when the user hive is where Windows keeps it, else a
/// `SOFTWARE` hive next to it (hives exported to one directory), as [`Hosts`] finds SYSTEM hives.
struct Machines {
    logs: bool,
    read: HashMap<PathBuf, Option<Rc<Machine>>>,
}

impl Machines {
    fn new(logs: bool) -> Self {
        Machines {
            logs,
            read: HashMap::new(),
        }
    }

    /// The machine of the user hive at `path`.
    fn of(&mut self, path: &Path) -> Option<Rc<Machine>> {
        let software = system_root(path)
            .and_then(|root| find_in(&root, &["Windows", "System32", "config", "SOFTWARE"]))
            .or_else(|| find_in(path.parent()?, &["SOFTWARE"]))?;
        if let Some(machine) = self.read.get(&software) {
            return machine.clone();
        }
        let logs = if self.logs {
            Logs::NextToIt
        } else {
            Logs::Without
        };
        let machine = open(&software, logs)
            .ok()
            .filter(|o| o.hive_type == "SOFTWARE")
            .map(|o| {
                Rc::new(Machine {
                    classes: MachineClasses::read(&o.hive),
                    profiles: profile_list(&o.hive),
                })
            });
        self.read.insert(software, machine.clone());
        machine
    }
}

/// The name of the profile folder a `UsrClass.dat` is in, when it is where Windows keeps it
/// (`…\Users\alice\AppData\Local\Microsoft\Windows\UsrClass.dat`).
fn profile_of_classes_path(path: &Path) -> Option<String> {
    let root = system_root(path)?;
    let rest = path.strip_prefix(&root).ok()?;
    let mut parts = rest.components().map(|c| c.as_os_str().to_string_lossy());
    let top = parts.next()?;
    (top.eq_ignore_ascii_case("Users") || top.eq_ignore_ascii_case("Documents and Settings"))
        .then(|| parts.next().map(|user| user.to_string()))
        .flatten()
}

// --- records ----------------------------------------------------------------------------------

/// The autostarts of one hive, with the user and SID of a user hive.
struct Read {
    records: Vec<Autostart>,
    problems: Vec<String>,
    user: Option<String>,
    user_sid: Option<String>,
    /// For a user hive: whether its system's SOFTWARE hive was found.
    machine_found: Option<bool>,
}

impl Read {
    fn of(
        path: &Path,
        opened: &Opened,
        opts: &RegistryPersistenceOptions,
        machines: &mut Machines,
    ) -> Result<Self, String> {
        let hive = &opened.hive;
        let mut read = Read {
            records: Vec::new(),
            problems: Vec::new(),
            user: None,
            user_sid: None,
            machine_found: None,
        };
        match opened.hive_type.as_str() {
            "SOFTWARE" => (read.records, read.problems) = persistence::software(hive),
            "SYSTEM" => {
                let sets = ControlSets::read(hive);
                if sets.present.is_empty() {
                    return Err("no control set (ControlSet001, …): not a SYSTEM hive".to_string());
                }
                let chosen: Vec<u32> = if opts.all_control_sets {
                    sets.present.clone()
                } else {
                    sets.current_or_first().into_iter().collect()
                };
                for set in chosen {
                    let (records, problems) = persistence::system(hive, set);
                    read.records.extend(records);
                    read.problems.extend(problems);
                }
            }
            kind => {
                let machine = machines.of(path);
                read.machine_found = Some(machine.is_some());
                let classes = machine.as_ref().map(|m| &m.classes);
                if kind == "NTUSER.DAT" {
                    read.user = profile_of_path(path).or_else(|| profile_name(hive));
                    (read.records, read.problems) = persistence::ntuser(hive, classes);
                } else {
                    read.user = profile_of_classes_path(path);
                    read.user_sid = classes_sid(hive);
                    (read.records, read.problems) = persistence::usrclass(hive, classes);
                }
                if read.user_sid.is_none()
                    && let (Some(machine), Some(user)) = (&machine, &read.user)
                {
                    read.user_sid = machine.sid_of(user);
                }
            }
        }
        Ok(read)
    }
}

/// One autostart's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    host: Option<&'a str>,
    source_path: &'a str,
    hive_type: &'a str,
    user: Option<&'a str>,
    user_sid: Option<&'a str>,
    mechanism: &'static str,
    attack_technique: Option<&'static str>,
    key_path: &'a str,
    value_name: &'a str,
    data: Option<&'a str>,
    image_path: Option<&'a str>,
    arguments: Option<&'a str>,
    enabled: Option<bool>,
    disabled_at: Option<String>,
    details: Option<&'a str>,
    overrides_machine_registration: Option<bool>,
    wow64: bool,
    key_last_written: Option<String>,
    data_raw: Option<String>,
    transaction_log_status: &'static str,
    logs_applied: &'a [String],
    problems: &'a [String],
}

const CSV_HEADER: [&str; 22] = [
    "Host",
    "SourcePath",
    "HiveType",
    "User",
    "UserSid",
    "Mechanism",
    "AttackTechnique",
    "KeyPath",
    "ValueName",
    "Data",
    "ImagePath",
    "Arguments",
    "Enabled",
    "DisabledAtUTC",
    "Details",
    "OverridesMachineRegistration",
    "Wow64",
    "KeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// A hive type as Windows names the file: `UsrClass.dat`, not `USRCLASS.DAT`.
fn hive_type_name(hive_type: &str) -> &str {
    if hive_type == "USRCLASS.DAT" {
        "UsrClass.dat"
    } else {
        hive_type
    }
}

impl<'a> Record<'a> {
    fn new(opened: &'a Opened, read: &'a Read, autostart: &'a Autostart) -> Self {
        Record {
            host: opened.host.as_deref(),
            source_path: &opened.source,
            hive_type: hive_type_name(&opened.hive_type),
            user: read.user.as_deref(),
            user_sid: read.user_sid.as_deref(),
            mechanism: autostart.mechanism.name(),
            attack_technique: autostart.mechanism.attack_technique(),
            key_path: &autostart.key_path,
            value_name: &autostart.value_name,
            data: autostart.data.as_deref(),
            image_path: autostart.image_path.as_deref(),
            arguments: autostart.arguments.as_deref(),
            enabled: autostart.enabled,
            disabled_at: autostart.disabled_at.and_then(filetime),
            details: autostart.details.as_deref(),
            overrides_machine_registration: autostart.overrides_machine,
            wow64: autostart.wow64,
            key_last_written: filetime(autostart.last_written),
            data_raw: autostart.data_raw.as_deref().map(hex),
            transaction_log_status: opened.log_status(autostart.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &autostart.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let flag = |b: bool| if b { "True" } else { "False" }.to_string();
        vec![
            text(self.host),
            self.source_path.to_string(),
            self.hive_type.to_string(),
            text(self.user),
            text(self.user_sid),
            self.mechanism.to_string(),
            text(self.attack_technique),
            self.key_path.to_string(),
            self.value_name.to_string(),
            text(self.data),
            text(self.image_path),
            text(self.arguments),
            self.enabled.map(flag).unwrap_or_default(),
            text(self.disabled_at.as_deref()),
            text(self.details),
            self.overrides_machine_registration
                .map(flag)
                .unwrap_or_default(),
            flag(self.wow64),
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
    /// Hives parsed, by type.
    types: BTreeMap<&'static str, usize>,
    failed: usize,
    hosts: BTreeSet<String>,
    users: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    /// User hives whose system's SOFTWARE hive wasn't found.
    without_machine: Vec<String>,
    records: usize,
    mechanisms: BTreeMap<&'static str, usize>,
    disabled: usize,
    /// `StartupApproved` entries without their `Run` value.
    approved_only: Vec<String>,
    approved_only_count: usize,
    /// A user's classes and commands that override the machine's.
    overrides: Vec<String>,
    override_count: usize,
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
            .map_or("", |t| hive_type_name(t));
        *self.types.entry(kind).or_default() += 1;
        self.hosts.extend(opened.host.clone());
        self.users.extend(read.user.clone());
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
        if read.machine_found == Some(false) {
            self.without_machine.push(opened.source.clone());
        }
        for problem in &read.problems {
            self.problem(&opened.source, problem);
        }
        for autostart in &read.records {
            let record = Record::new(opened, read, autostart);
            match writer.as_deref_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv())?,
                Some(w) => w.json(&record)?,
                None => {}
            }
            self.autostart(opened, autostart);
        }
        Ok(())
    }

    /// Count an autostart written.
    fn autostart(&mut self, opened: &Opened, autostart: &Autostart) {
        self.records += 1;
        *self
            .mechanisms
            .entry(autostart.mechanism.name())
            .or_default() += 1;
        let at = format!(
            r"{}: {}\{}",
            opened.source, autostart.key_path, autostart.value_name
        );
        for problem in &autostart.problems {
            self.problem(&opened.source, &format!("{at}: {problem}"));
        }
        if autostart.enabled == Some(false) {
            self.disabled += 1;
        }
        if autostart.data.is_none() && autostart.key_path.contains("StartupApproved") {
            self.approved_only_count += 1;
            if self.approved_only.len() < MAX_PRINTED {
                self.approved_only.push(at.clone());
            }
        }
        if autostart.overrides_machine == Some(true) {
            self.override_count += 1;
            if self.overrides.len() < MAX_PRINTED {
                let target = autostart
                    .image_path
                    .as_deref()
                    .or(autostart.data.as_deref())
                    .unwrap_or_default();
                self.overrides.push(format!("{at} = {target}"));
            }
        }
    }

    fn problem(&mut self, source: &str, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED {
            if problem.starts_with(source) {
                self.problems.push(problem.to_string());
            } else {
                self.problems.push(format!("{source}: {problem}"));
            }
        }
    }

    fn print(&self, no_color: bool) {
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
                "Dirty hives whose transaction logs were not applied (autostarts may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if !self.without_machine.is_empty() {
            p_label(
                no_color,
                "User hives without their system's SOFTWARE hive (overrides of the machine's classes unknown): ",
                &fmt_num(self.without_machine.len()),
            );
            for source in self.without_machine.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        p_label(no_color, "Autostarts: ", &fmt_num(self.records));
        for (mechanism, n) in &self.mechanisms {
            p(None, &format!("  {mechanism}: {}", fmt_num(*n)), true);
        }
        p_label(no_color, "Disabled: ", &fmt_num(self.disabled));
        p_label(
            no_color,
            "Disabled in StartupApproved without their Run value: ",
            &fmt_num(self.approved_only_count),
        );
        for entry in &self.approved_only {
            p(None, &format!("  {entry}"), true);
        }
        p_label(
            no_color,
            "A user's COM classes and commands overriding the machine's: ",
            &fmt_num(self.override_count),
        );
        for entry in &self.overrides {
            p(None, &format!("  {entry}"), true);
        }
        p_label(
            no_color,
            "Values that could not be read: ",
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
    fn profiles_of_classes_hives() {
        let profile = |p: &str| profile_of_classes_path(Path::new(p));
        assert_eq!(
            profile("/case/C/Users/alice/AppData/Local/Microsoft/Windows/UsrClass.dat").as_deref(),
            Some("alice")
        );
        assert_eq!(
            profile(
                "/case/C/Documents and Settings/bob/Local Settings/Application Data/Microsoft/Windows/UsrClass.dat"
            )
            .as_deref(),
            Some("bob")
        );
        assert_eq!(profile("/case/hives/UsrClass.dat"), None);
    }

    #[test]
    fn sids_by_profile_folder() {
        let machine = Machine {
            classes: MachineClasses::default(),
            profiles: vec![
                (
                    "S-1-5-18".into(),
                    r"%systemroot%\system32\config\systemprofile".into(),
                ),
                ("S-1-5-21-1-2-3-1001".into(), r"C:\Users\Alice".into()),
            ],
        };
        assert_eq!(
            machine.sid_of("alice").as_deref(),
            Some("S-1-5-21-1-2-3-1001")
        );
        assert_eq!(machine.sid_of("bob"), None);
    }
}
