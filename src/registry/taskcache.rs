//! `parse-registry taskcache`: the Task Scheduler's cache in `SOFTWARE` hives, read by
//! ys-registry-parser's `software` module, with Hayabusa's input options, progress bar and
//! results summary. One record per action of each task (one for a task without actions): the
//! task's name, GUID, lists and times from `TaskCache\Tree` and `TaskCache\Tasks`, its actions
//! decoded by their structure (program, arguments, working directory; COM handler), and its last
//! registration and runs (`DynamicInfo`). A task the Task Scheduler doesn't show (no `Tree`
//! entry, or no security descriptor where the hive's other tasks have one) is `hidden`; the
//! summary lists those, and the programs of the tasks outside `\Microsoft\`.

use super::{
    Hosts, Opened, RecordWriter, choose_logs, files_read, filetime, find_hives_of_type,
    input_files, open, sink,
};
use crate::cli::RegistryTaskCacheOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::software::{ScheduledTask, TaskAction, task_cache};
use serde::Serialize;
use std::collections::BTreeSet;
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems, hidden tasks and programs listed in the summary.
const MAX_PRINTED: usize = 20;

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryTaskCacheOptions) -> i32 {
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
        "SOFTWARE hives",
        |d| find_hives_of_type(d, "SOFTWARE"),
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
        p_label(no_color, "SOFTWARE hives: ", &fmt_num(files.len()));
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
            if !opened.hive_type.is_empty() && opened.hive_type != "SOFTWARE" {
                return Err(format!("a {} hive, not a SOFTWARE hive", opened.hive_type));
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

/// Whether `task` is the one `--task` names: its whole name, or its last part.
fn named(task: &ScheduledTask, name: &str) -> bool {
    let whole = format!(r"\{}", name.trim_start_matches('\\'));
    let last = task.name.rsplit('\\').next().unwrap_or_default();
    if name.contains('\\') {
        task.name.eq_ignore_ascii_case(&whole)
    } else {
        last.eq_ignore_ascii_case(name)
    }
}

/// What an action does, as fields of a record.
#[derive(Default)]
struct ActionFields<'a> {
    kind: Option<&'static str>,
    executable: Option<&'a str>,
    arguments: Option<&'a str>,
    working_directory: Option<&'a str>,
    com_class_id: Option<&'a str>,
    data: Option<String>,
}

impl<'a> ActionFields<'a> {
    fn of(action: &'a TaskAction) -> Self {
        let kind = Some(action.kind());
        match action {
            TaskAction::Exec {
                command,
                arguments,
                working_directory,
                ..
            } => ActionFields {
                kind,
                executable: Some(command),
                arguments: Some(arguments),
                working_directory: Some(working_directory),
                ..ActionFields::default()
            },
            TaskAction::ComHandler { class_id, data, .. } => ActionFields {
                kind,
                com_class_id: Some(class_id),
                data: Some(data.clone()),
                ..ActionFields::default()
            },
            TaskAction::SendEmail { fields, .. } => ActionFields {
                kind,
                // From, To, Subject
                data: Some(format!(
                    "{} -> {}: {}",
                    fields.first().map_or("", String::as_str),
                    fields.get(1).map_or("", String::as_str),
                    fields.get(6).map_or("", String::as_str)
                )),
                ..ActionFields::default()
            },
            TaskAction::ShowMessage { title, message, .. } => ActionFields {
                kind,
                data: Some(format!("{title}: {message}")),
                ..ActionFields::default()
            },
        }
    }
}

/// One action's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    host: Option<&'a str>,
    source_path: &'a str,
    task_name: &'a str,
    task_guid: Option<&'a str>,
    task_index: Option<u32>,
    task_lists: &'a [String],
    hidden: bool,
    in_tree: bool,
    in_tasks: bool,
    has_security_descriptor: Option<bool>,
    tree_key_last_written: Option<String>,
    task_key_last_written: Option<String>,
    action_index: Option<usize>,
    action_type: Option<&'static str>,
    executable: Option<&'a str>,
    arguments: Option<&'a str>,
    working_directory: Option<&'a str>,
    com_class_id: Option<&'a str>,
    action_data: Option<String>,
    context: Option<&'a str>,
    author: Option<&'a str>,
    uri: Option<&'a str>,
    description: Option<&'a str>,
    last_registered: Option<String>,
    last_run: Option<String>,
    last_successful_run: Option<String>,
    last_result: Option<String>,
    actions_raw: Option<String>,
    transaction_log_status: &'static str,
    logs_applied: &'a [String],
    problems: &'a [String],
}

const CSV_HEADER: [&str; 31] = [
    "Host",
    "SourcePath",
    "TaskName",
    "TaskGuid",
    "TaskIndex",
    "TaskLists",
    "Hidden",
    "InTree",
    "InTasks",
    "HasSecurityDescriptor",
    "TreeKeyLastWrittenUTC",
    "TaskKeyLastWrittenUTC",
    "ActionIndex",
    "ActionType",
    "Executable",
    "Arguments",
    "WorkingDirectory",
    "ComClassId",
    "ActionData",
    "Context",
    "Author",
    "Uri",
    "Description",
    "LastRegisteredUTC",
    "LastRunUTC",
    "LastSuccessfulRunUTC",
    "LastResult",
    "ActionsRaw",
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
        task: &'a ScheduledTask,
        hidden: bool,
        action: Option<(usize, ActionFields<'a>)>,
    ) -> Self {
        let (index, fields) = action.unzip();
        let fields = fields.unwrap_or_default();
        let dynamic = task.dynamic_info.as_ref();
        Record {
            host: opened.host.as_deref(),
            source_path: &opened.source,
            task_name: &task.name,
            task_guid: task.guid.as_deref(),
            task_index: task.index,
            task_lists: &task.lists,
            hidden,
            in_tree: task.in_tree,
            in_tasks: task.in_tasks,
            has_security_descriptor: task.has_security_descriptor,
            tree_key_last_written: task.tree_last_written.and_then(filetime),
            task_key_last_written: task.task_last_written.and_then(filetime),
            action_index: index,
            action_type: fields.kind,
            executable: fields.executable,
            arguments: fields.arguments,
            working_directory: fields.working_directory,
            com_class_id: fields.com_class_id,
            action_data: fields.data,
            context: task.actions.as_ref().and_then(|a| a.context.as_deref()),
            author: task.author.as_deref(),
            uri: task.uri.as_deref(),
            description: task.description.as_deref(),
            last_registered: dynamic.and_then(|d| filetime(d.last_registered)),
            last_run: dynamic.and_then(|d| filetime(d.last_run)),
            last_successful_run: dynamic
                .and_then(|d| d.last_successful_run)
                .and_then(filetime),
            last_result: dynamic.map(|d| format!("0x{:08x}", d.last_result)),
            actions_raw: task.actions_raw.as_deref().map(hex),
            transaction_log_status: opened.log_status(task.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &task.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let flag = |b: bool| if b { "True" } else { "False" }.to_string();
        vec![
            text(self.host),
            self.source_path.to_string(),
            self.task_name.to_string(),
            text(self.task_guid),
            self.task_index.map(|n| n.to_string()).unwrap_or_default(),
            self.task_lists.join("|"),
            flag(self.hidden),
            flag(self.in_tree),
            flag(self.in_tasks),
            self.has_security_descriptor.map(flag).unwrap_or_default(),
            text(self.tree_key_last_written.as_deref()),
            text(self.task_key_last_written.as_deref()),
            self.action_index.map(|n| n.to_string()).unwrap_or_default(),
            text(self.action_type),
            text(self.executable),
            text(self.arguments),
            text(self.working_directory),
            text(self.com_class_id),
            text(self.action_data.as_deref()),
            text(self.context),
            text(self.author),
            text(self.uri),
            text(self.description),
            text(self.last_registered.as_deref()),
            text(self.last_run.as_deref()),
            text(self.last_successful_run.as_deref()),
            text(self.last_result.as_deref()),
            text(self.actions_raw.as_deref()),
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
    without_task: usize,
    tasks: usize,
    exec: usize,
    com: usize,
    other_actions: usize,
    /// Tasks the Task Scheduler doesn't show, and why.
    hidden: Vec<String>,
    hidden_count: usize,
    /// The programs of the tasks outside `\Microsoft\`.
    programs: Vec<String>,
    program_count: usize,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened`'s tasks (when there is a writer) and count them.
    fn hive(
        &mut self,
        opened: &Opened,
        opts: &RegistryTaskCacheOptions,
        mut writer: Option<&mut RecordWriter>,
    ) -> io::Result<()> {
        self.hives += 1;
        self.hosts.extend(opened.host.clone());
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
        let Some((tasks, problems)) = task_cache(&opened.hive) else {
            self.without += 1;
            return Ok(());
        };
        for problem in &problems {
            self.problem(&opened.source, "TaskCache", problem);
        }
        // a task without its SD is hidden only where the others have one (older Windows keeps
        // none)
        let hive_has_sd = tasks
            .iter()
            .any(|t| t.has_security_descriptor == Some(true));
        let mut found = false;
        for task in &tasks {
            if opts.task.as_ref().is_some_and(|name| !named(task, name)) {
                continue;
            }
            found = true;
            let no_sd = hive_has_sd && task.has_security_descriptor == Some(false);
            let hidden = !task.in_tree || no_sd;
            self.task(opened, task, hidden, no_sd);
            let actions = task.actions.as_ref().map(|a| &a.actions[..]).unwrap_or(&[]);
            let mut write = |record: &Record| match writer.as_deref_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv()),
                Some(w) => w.json(record),
                None => Ok(()),
            };
            if actions.is_empty() {
                write(&Record::new(opened, task, hidden, None))?;
            }
            for (i, action) in actions.iter().enumerate() {
                write(&Record::new(
                    opened,
                    task,
                    hidden,
                    Some((i, ActionFields::of(action))),
                ))?;
            }
        }
        if opts.task.is_some() && !found {
            self.without_task += 1;
        }
        Ok(())
    }

    /// Count a task written.
    fn task(&mut self, opened: &Opened, task: &ScheduledTask, hidden: bool, no_sd: bool) {
        self.tasks += 1;
        for problem in &task.problems {
            self.problem(&opened.source, &task.name, problem);
        }
        if hidden {
            self.hidden_count += 1;
            if self.hidden.len() < MAX_PRINTED {
                let why = if no_sd {
                    "its Tree entry has no SD"
                } else {
                    "no Tree entry names it"
                };
                self.hidden
                    .push(format!("{}: {} ({why})", opened.source, task.name));
            }
        }
        let outside = !task.name.to_lowercase().starts_with(r"\microsoft\");
        for action in task.actions.iter().flat_map(|a| &a.actions) {
            match action {
                TaskAction::Exec {
                    command, arguments, ..
                } => {
                    self.exec += 1;
                    if outside {
                        self.program_count += 1;
                        if self.programs.len() < MAX_PRINTED {
                            self.programs.push(
                                format!("{}: {}: {command} {arguments}", opened.source, task.name)
                                    .trim_end()
                                    .to_string(),
                            );
                        }
                    }
                }
                TaskAction::ComHandler { .. } => self.com += 1,
                _ => self.other_actions += 1,
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
        p_label(no_color, "SOFTWARE hives parsed: ", &fmt_num(self.hives));
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
                "Dirty hives whose transaction logs were not applied (tasks may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if self.without > 0 {
            p_label(
                no_color,
                "Hives without TaskCache: ",
                &fmt_num(self.without),
            );
        }
        if self.without_task > 0 {
            p_label(
                no_color,
                "Hives without the task of --task: ",
                &fmt_num(self.without_task),
            );
        }
        p_label(no_color, "Tasks: ", &fmt_num(self.tasks));
        let mut actions = format!(
            "{} (exec {}, COM handler {}",
            fmt_num(self.exec + self.com + self.other_actions),
            fmt_num(self.exec),
            fmt_num(self.com)
        );
        if self.other_actions > 0 {
            actions.push_str(&format!(
                ", e-mail or message {}",
                fmt_num(self.other_actions)
            ));
        }
        actions.push(')');
        p_label(no_color, "Actions: ", &actions);
        p_label(
            no_color,
            "Tasks hidden from the Task Scheduler: ",
            &fmt_num(self.hidden_count),
        );
        for hidden in &self.hidden {
            p(None, &format!("  {hidden}"), true);
        }
        p_label(
            no_color,
            "Programs of tasks outside \\Microsoft\\: ",
            &fmt_num(self.program_count),
        );
        for program in &self.programs {
            p(None, &format!("  {program}"), true);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tasks_by_name() {
        let task = ScheduledTask {
            name: r"\Microsoft\Windows\Defrag\ScheduledDefrag".into(),
            ..ScheduledTask::default()
        };
        assert!(named(&task, "scheduleddefrag"));
        assert!(named(&task, r"\Microsoft\Windows\Defrag\ScheduledDefrag"));
        assert!(named(&task, r"microsoft\windows\defrag\scheduleddefrag"));
        assert!(!named(&task, "Defrag"));
        assert!(!named(&task, r"Defrag\ScheduledDefrag"));
    }
}
