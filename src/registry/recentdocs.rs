//! `parse-registry recentdocs`: Explorer's recent documents in `NTUSER.DAT` hives, read by
//! ys-registry-parser's `ntuser` module, with Hayabusa's input options, progress bar and results
//! summary. One record per item of `RecentDocs` and of each of its subkeys (one per extension,
//! and `Folder`), in the order of the key's `MRUListEx`: the item's name, the name of its
//! shortcut in the `Recent` folder, and the key's last-write time, which is when its most recent
//! item (`mru_position` 0) was opened. The summary lists each hive's most recent item.

use super::{
    Hosts, Opened, RecordWriter, choose_logs, filetime, find_hives_of_type, input_files, open,
    profile_of_path, sink,
};
use crate::cli::RegistryRecentDocsOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use registry::ntuser::{RecentDoc, RecentDocs, profile_name, recent_docs};
use serde::Serialize;
use std::collections::BTreeSet;
use std::io;
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems and recent items listed in the summary.
const MAX_PRINTED: usize = 20;

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryRecentDocsOptions) -> i32 {
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
        "NTUSER.DAT hives",
        |d| find_hives_of_type(d, "NTUSER.DAT"),
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
        p_label(no_color, "NTUSER.DAT hives: ", &fmt_num(files.len()));
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
            if !opened.hive_type.is_empty() && opened.hive_type != "NTUSER.DAT" {
                return Err(format!(
                    "a {} hive, not an NTUSER.DAT hive",
                    opened.hive_type
                ));
            }
            opened.host = hosts.of(path, &opened);
            let username = profile_of_path(path).or_else(|| profile_name(&opened.hive));
            Ok((opened, username))
        });
        match parsed {
            Ok((opened, username)) => {
                if let Err(e) = stats.hive(&opened, username.as_deref(), writer.as_mut()) {
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

/// A recent document's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    host: Option<&'a str>,
    username: Option<&'a str>,
    source_path: &'a str,
    extension: Option<&'a str>,
    mru_position: Option<u32>,
    value_index: u32,
    filename: Option<&'a str>,
    link_name: Option<&'a str>,
    key_last_written: Option<String>,
    last_opened: Option<String>,
    value_data_raw: Option<String>,
    transaction_log_status: &'static str,
    logs_applied: &'a [String],
    problems: &'a [String],
}

const CSV_HEADER: [&str; 14] = [
    "Host",
    "Username",
    "SourcePath",
    "Extension",
    "MruPosition",
    "ValueIndex",
    "Filename",
    "LinkName",
    "KeyLastWrittenUTC",
    "LastOpenedUTC",
    "ValueDataRaw",
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
        username: Option<&'a str>,
        list: &'a RecentDocs,
        item: &'a RecentDoc,
    ) -> Self {
        let written = filetime(list.last_written);
        Record {
            host: opened.host.as_deref(),
            username,
            source_path: &opened.source,
            extension: list.extension.as_deref(),
            mru_position: item.mru_position,
            value_index: item.value_index,
            filename: item.name.as_deref(),
            link_name: item.link_name.as_deref(),
            // the key is written when an item is opened: its time is the most recent item's
            last_opened: (item.mru_position == Some(0))
                .then(|| written.clone())
                .flatten(),
            key_last_written: written,
            value_data_raw: item.raw.as_deref().map(hex),
            transaction_log_status: opened.log_status(item.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &item.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let num = |v: Option<u32>| v.map(|n| n.to_string()).unwrap_or_default();
        vec![
            text(self.host),
            text(self.username),
            self.source_path.to_string(),
            text(self.extension),
            num(self.mru_position),
            self.value_index.to_string(),
            text(self.filename),
            text(self.link_name),
            text(self.key_last_written.as_deref()),
            text(self.last_opened.as_deref()),
            text(self.value_data_raw.as_deref()),
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
    users: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    without: usize,
    lists: usize,
    items: usize,
    /// Each hive's most recent item, and when it was opened.
    latest: Vec<String>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened`'s recent documents (when there is a writer) and count them.
    fn hive(
        &mut self,
        opened: &Opened,
        username: Option<&str>,
        mut writer: Option<&mut RecordWriter>,
    ) -> io::Result<()> {
        self.hives += 1;
        self.hosts.extend(opened.host.clone());
        self.users.extend(username.map(str::to_string));
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
        let Some((lists, problems)) = recent_docs(&opened.hive) else {
            self.without += 1;
            return Ok(());
        };
        for problem in &problems {
            self.problem(&opened.source, "RecentDocs", problem);
        }
        for list in &lists {
            self.lists += 1;
            let at = match &list.extension {
                Some(ext) => format!(r"RecentDocs\{ext}"),
                None => "RecentDocs".to_string(),
            };
            for problem in &list.problems {
                self.problem(&opened.source, &at, problem);
            }
            for item in &list.items {
                self.items += 1;
                for problem in &item.problems {
                    self.problem(
                        &opened.source,
                        &format!("{at}\\{}", item.value_index),
                        problem,
                    );
                }
                if list.extension.is_none()
                    && item.mru_position == Some(0)
                    && self.latest.len() < MAX_PRINTED
                {
                    let when = filetime(list.last_written).unwrap_or_default();
                    let name = item.name.as_deref().unwrap_or("(no name)");
                    self.latest
                        .push(format!("{}: {name} ({when})", opened.source));
                }
                let record = Record::new(opened, username, list, item);
                match writer.as_deref_mut() {
                    Some(w) if w.is_csv() => w.csv(&record.csv())?,
                    Some(w) => w.json(&record)?,
                    None => {}
                }
            }
        }
        Ok(())
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
        p_label(no_color, "NTUSER.DAT hives parsed: ", &fmt_num(self.hives));
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
                "Dirty hives whose transaction logs were not applied (recent items may be missing): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if self.without > 0 {
            p_label(
                no_color,
                "Hives without RecentDocs: ",
                &fmt_num(self.without),
            );
        }
        p_label(
            no_color,
            "Recent items: ",
            &format!(
                "{} (in {} lists: RecentDocs and its extensions)",
                fmt_num(self.items),
                fmt_num(self.lists)
            ),
        );
        if !self.latest.is_empty() {
            p_label(no_color, "Most recent item of each hive: ", "");
            for latest in &self.latest {
                p(None, &format!("  {latest}"), true);
            }
        }
        p_label(
            no_color,
            "Items or lists that could not be read: ",
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
