//! `parse-registry services`: the services of `SYSTEM` hives, read by ys-registry-parser's
//! `system` module, with Hayabusa's input options, progress bar and results summary. One record
//! per service of the control set Windows used last (`Select\Current`), or of every control set
//! (`--all-control-sets`): its image path, service DLL, start and type (named, with their
//! numbers), account, and what Windows does when it fails (`FailureActions` decoded, and the
//! `FailureCommand`). The summary lists the services whose failure runs a command.

use super::{
    Hosts, Opened, RecordWriter, choose_logs, filetime, find_hives_of_type, input_files, open, sink,
};
use crate::cli::RegistryServicesOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::system::{
    Action, ControlSets, FailureActions, Service, control_set_name, error_control_name, services,
    start_name, type_names,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems and failure commands listed in the summary.
const MAX_PRINTED: usize = 20;

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryServicesOptions) -> i32 {
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
    let mut stats = Stats::default();
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        let parsed = open(path, logs).and_then(|mut opened| {
            opened.host = hosts.of(path, &opened);
            let sets = ControlSets::read(&opened.hive);
            if sets.present.is_empty() {
                return Err("no control set (ControlSet001, …): not a SYSTEM hive".to_string());
            }
            Ok((opened, sets))
        });
        match parsed {
            Ok((opened, sets)) => {
                if let Err(e) = stats.hive(&opened, &sets, opts, writer.as_mut()) {
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

// --- records ----------------------------------------------------------------------------------

/// One failure action in JSON.
#[derive(Serialize)]
struct ActionRecord {
    #[serde(rename = "type")]
    kind: String,
    delay_ms: u32,
}

/// A service's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    host: Option<&'a str>,
    source_path: &'a str,
    control_set: String,
    is_current_control_set: bool,
    service_name: &'a str,
    display_name: Option<&'a str>,
    description: Option<&'a str>,
    image_path: Option<&'a str>,
    service_dll: Option<&'a str>,
    start_type: Option<String>,
    start_value: Option<u32>,
    delayed_autostart: Option<bool>,
    service_type: Option<String>,
    type_value: Option<u32>,
    error_control: Option<String>,
    object_name: Option<&'a str>,
    group: Option<&'a str>,
    failure_reset_period: Option<u32>,
    failure_command: Option<&'a str>,
    failure_actions: Option<Vec<ActionRecord>>,
    failure_action_type: Option<String>,
    failure_action_delay_ms: Option<u32>,
    failure_actions_raw: Option<String>,
    key_last_written: Option<String>,
    transaction_log_status: &'static str,
    logs_applied: &'a [String],
    problems: &'a [String],
}

const CSV_HEADER: [&str; 27] = [
    "Host",
    "SourcePath",
    "ControlSet",
    "IsCurrentControlSet",
    "ServiceName",
    "DisplayName",
    "Description",
    "ImagePath",
    "ServiceDll",
    "StartType",
    "StartValue",
    "DelayedAutostart",
    "ServiceType",
    "TypeValue",
    "ErrorControl",
    "ObjectName",
    "Group",
    "FailureResetPeriod",
    "FailureCommand",
    "FailureActions",
    "FailureActionType",
    "FailureActionDelayMs",
    "FailureActionsRaw",
    "KeyLastWrittenUTC",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

impl<'a> Record<'a> {
    fn new(opened: &'a Opened, set: u32, current: bool, s: &'a Service) -> Self {
        let actions = s.failure_actions.as_ref();
        let first = actions.and_then(|f: &FailureActions| f.actions.first());
        Record {
            host: opened.host.as_deref(),
            source_path: &opened.source,
            control_set: control_set_name(set),
            is_current_control_set: current,
            service_name: &s.name,
            display_name: s.display_name.as_deref(),
            description: s.description.as_deref(),
            image_path: s.image_path.as_deref(),
            service_dll: s.service_dll.as_deref(),
            start_type: s.start.map(start_name),
            start_value: s.start,
            delayed_autostart: s.delayed_autostart,
            service_type: s.service_type.map(|t| type_names(t).join("|")),
            type_value: s.service_type,
            error_control: s.error_control.map(error_control_name),
            object_name: s.object_name.as_deref(),
            group: s.group.as_deref(),
            failure_reset_period: actions.map(|f| f.reset_period),
            failure_command: s.failure_command.as_deref(),
            failure_actions: actions.map(|f| {
                f.actions
                    .iter()
                    .map(|a| ActionRecord {
                        kind: a.action.name(),
                        delay_ms: a.delay_ms,
                    })
                    .collect()
            }),
            failure_action_type: first.map(|a| a.action.name()),
            failure_action_delay_ms: first.map(|a| a.delay_ms),
            failure_actions_raw: s.failure_actions_raw.as_deref().map(hex),
            key_last_written: filetime(s.last_written),
            transaction_log_status: opened.log_status(s.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &s.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let num = |v: Option<u32>| v.map(|n| n.to_string()).unwrap_or_default();
        let flag = |b: bool| if b { "True" } else { "False" }.to_string();
        vec![
            text(self.host),
            self.source_path.to_string(),
            self.control_set.clone(),
            flag(self.is_current_control_set),
            self.service_name.to_string(),
            text(self.display_name),
            text(self.description),
            text(self.image_path),
            text(self.service_dll),
            text(self.start_type.as_deref()),
            num(self.start_value),
            self.delayed_autostart.map(flag).unwrap_or_default(),
            text(self.service_type.as_deref()),
            num(self.type_value),
            text(self.error_control.as_deref()),
            text(self.object_name),
            text(self.group),
            num(self.failure_reset_period),
            text(self.failure_command),
            self.failure_actions
                .as_ref()
                .map(|a| serde_json::to_string(a).unwrap_or_default())
                .unwrap_or_default(),
            text(self.failure_action_type.as_deref()),
            num(self.failure_action_delay_ms),
            text(self.failure_actions_raw.as_deref()),
            text(self.key_last_written.as_deref()),
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
    hosts: Vec<String>,
    recovered: usize,
    dirty: Vec<String>,
    without_select: Vec<String>,
    control_sets: usize,
    without_service: usize,
    services: usize,
    /// By `Start` value (`u32::MAX`: none).
    starts: BTreeMap<u32, usize>,
    with_actions: usize,
    /// The services whose failure runs a command, with it.
    commands: Vec<String>,
    command_count: usize,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened`'s services (when there is a writer) and count them.
    fn hive(
        &mut self,
        opened: &Opened,
        sets: &ControlSets,
        opts: &RegistryServicesOptions,
        mut writer: Option<&mut RecordWriter>,
    ) -> io::Result<()> {
        self.hives += 1;
        if let Some(host) = &opened.host
            && !self.hosts.contains(host)
        {
            self.hosts.push(host.clone());
        }
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
        let mut found = false;
        for set in read {
            let Some((list, problems)) = services(&opened.hive, set) else {
                self.problem(
                    &opened.source,
                    &control_set_name(set),
                    "it has no Services key",
                );
                continue;
            };
            self.control_sets += 1;
            for problem in &problems {
                self.problem(&opened.source, &control_set_name(set), problem);
            }
            for service in &list {
                if opts
                    .service
                    .as_ref()
                    .is_some_and(|name| !name.eq_ignore_ascii_case(&service.name))
                {
                    continue;
                }
                found = true;
                self.service(opened, set, service);
                let record = Record::new(opened, set, sets.current == Some(set), service);
                match writer.as_deref_mut() {
                    Some(w) if w.is_csv() => w.csv(&record.csv())?,
                    Some(w) => w.json(&record)?,
                    None => {}
                }
            }
        }
        if opts.service.is_some() && !found {
            self.without_service += 1;
        }
        Ok(())
    }

    /// Count a service written.
    fn service(&mut self, opened: &Opened, set: u32, s: &Service) {
        self.services += 1;
        *self.starts.entry(s.start.unwrap_or(u32::MAX)).or_default() += 1;
        let where_ = format!(r"{}\{}", control_set_name(set), s.name);
        for problem in &s.problems {
            self.problem(&opened.source, &where_, problem);
        }
        let Some(actions) = &s.failure_actions else {
            return;
        };
        self.with_actions += 1;
        if actions
            .actions
            .iter()
            .any(|a| a.action == Action::RunCommand)
        {
            self.command_count += 1;
            if self.commands.len() < MAX_PRINTED {
                let command = s
                    .failure_command
                    .as_deref()
                    .unwrap_or("(no FailureCommand)");
                self.commands
                    .push(format!("{}: {where_}: {command}", opened.source));
            }
        }
    }

    fn problem(&mut self, source: &str, at: &str, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED {
            self.problems.push(format!("{source}: {at}: {problem}"));
        }
    }

    fn print(&self, opts: &RegistryServicesOptions, no_color: bool) {
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        p_label(no_color, "SYSTEM hives parsed: ", &fmt_num(self.hives));
        if self.failed > 0 {
            p_label(no_color, "Could not parse: ", &fmt_num(self.failed));
        }
        if !self.hosts.is_empty() {
            p_label(no_color, "Hosts: ", &self.hosts.join(", "));
        }
        p_label(
            no_color,
            "Dirty hives recovered with their transaction logs: ",
            &fmt_num(self.recovered),
        );
        if !self.dirty.is_empty() {
            p_label(
                no_color,
                "Dirty hives whose transaction logs were not applied (services may be missing or out of date): ",
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
        if self.without_service > 0 {
            p_label(
                no_color,
                "Hives without the service of --service: ",
                &fmt_num(self.without_service),
            );
        }
        let starts: Vec<String> = self
            .starts
            .iter()
            .map(|(&s, n)| {
                let name = match s {
                    u32::MAX => "no Start value".to_string(),
                    s => start_name(s),
                };
                format!("{name} {}", fmt_num(*n))
            })
            .collect();
        let mut services = fmt_num(self.services);
        if !starts.is_empty() {
            services.push_str(&format!(" ({})", starts.join(", ")));
        }
        p_label(no_color, "Services: ", &services);
        p_label(
            no_color,
            "With failure actions: ",
            &fmt_num(self.with_actions),
        );
        p_label(
            no_color,
            "Failure actions that run a command: ",
            &fmt_num(self.command_count),
        );
        for command in &self.commands {
            p(None, &format!("  {command}"), true);
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
