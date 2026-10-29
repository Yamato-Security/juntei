//! `parse-registry read`: the keys and values of registry hives as they are, with Hayabusa's input
//! options, progress bar and results summary. One record per value, and one per key without
//! values or with damage: the key's last-write time, the value's type, its data decoded (strings,
//! multi-strings, numbers, binary data in hex) and, when it doesn't decode or `--raw-data` asks,
//! its bytes; numbers also as `value_u64`, and with `--qword-as-filetime` a `REG_QWORD` also as a
//! FILETIME. Each record says where it came from: allocated or deleted (`--recover-deleted`: left
//! in a free cell), and whether the transaction logs' replay wrote it. Filters on the key path,
//! value names and data (regular expressions ignoring case, so that a GUID matches in any case)
//! and last-write times keep a targeted query from writing the whole hive; `--value` with a key
//! filter also says which keys lack the value.

use super::{Hosts, Logs, Opened, RecordWriter, current_control_set, filetime, find_hives, open};
use crate::cli::RegistryReadOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, check_output, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use regex::{Regex, RegexBuilder};
use registry::{Data, DeletedValue, Key, Status, Value as RegValue, ValueType, decode};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems listed in the summary.
const MAX_PRINTED_PROBLEMS: usize = 20;

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryReadOptions) -> i32 {
    let no_color = opts.common.no_color;
    let data_on_stdout = opts.data_on_stdout();
    let report_error = |bar: Option<&ProgressBar>, msg: &str| {
        if data_on_stdout {
            eprintln!("{msg}");
        } else {
            progress::suspended(bar, || error_msg(no_color, msg));
        }
    };
    let filters = match Filters::new(opts) {
        Ok(f) => f,
        Err(msg) => {
            report_error(None, &msg);
            return 1;
        }
    };

    // --- inputs ---------------------------------------------------------------------------
    let files = match (&opts.filepath, &opts.directory) {
        (Some(f), _) => {
            if !f.is_file() {
                report_error(None, &format!("The file does not exist: {}", f.display()));
                return 1;
            }
            vec![f.clone()]
        }
        (None, Some(d)) => {
            if !d.is_dir() {
                report_error(
                    None,
                    &format!("The directory does not exist: {}", d.display()),
                );
                return 1;
            }
            let found = find_hives(d);
            if found.is_empty() {
                report_error(
                    None,
                    &format!("No registry hives were found in {}", d.display()),
                );
                return 1;
            }
            found
        }
        (None, None) => unreachable!("clap requires -f or -d"),
    };
    let given_logs: Vec<PathBuf> = opts.log1.iter().chain(&opts.log2).cloned().collect();
    for log in &given_logs {
        if !log.is_file() {
            report_error(None, &format!("The file does not exist: {}", log.display()));
            return 1;
        }
    }
    let logs = if opts.no_logs {
        Logs::Without
    } else if given_logs.is_empty() {
        Logs::NextToIt
    } else {
        Logs::Given(&given_logs)
    };
    if let Some(out) = &opts.output
        && let Err(msg) = check_output(out, false, opts.clobber)
    {
        report_error(None, &msg);
        return 1;
    }
    let sink: Option<Box<dyn Write>> = match (&opts.output, data_on_stdout) {
        (Some(path), _) => {
            if let Some(parent) = path.parent().filter(|d| !d.as_os_str().is_empty())
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                report_error(None, &format!("Could not create {}: {e}", parent.display()));
                return 1;
            }
            match File::create(path) {
                Ok(f) => Some(Box::new(BufWriter::new(f))),
                Err(e) => {
                    report_error(None, &format!("Could not create {}: {e}", path.display()));
                    return 1;
                }
            }
        }
        (None, true) => Some(Box::new(BufWriter::new(io::stdout()))),
        (None, false) => None,
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
    let output = Output {
        raw: opts.raw_data,
        qword_as_filetime: opts.qword_as_filetime,
    };
    let mut hosts = Hosts::new(!opts.no_logs);
    let mut stats = Stats::default();
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        match open(path, logs) {
            Ok(mut opened) => {
                opened.host = hosts.of(path, &opened);
                let mut out = Out {
                    writer: writer.as_mut(),
                    output,
                    stats: &mut stats,
                };
                let mut written = out.hive(&opened, &filters);
                if opts.recover_deleted && written.is_ok() {
                    written = out.deleted(&opened, &filters);
                }
                if let Err(e) = written {
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
        stats.print(&filters, opts.recover_deleted, no_color);
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

// --- filters ----------------------------------------------------------------------------------

/// What to write.
struct Filters {
    /// `--key`: that key only.
    key: Option<String>,
    key_regex: Option<Regex>,
    /// `--value`: the value of that name only (in any case).
    value: Option<String>,
    value_regex: Option<Regex>,
    data_regex: Option<Regex>,
    start: Option<u64>,
    end: Option<u64>,
}

impl Filters {
    fn new(opts: &RegistryReadOptions) -> Result<Self, String> {
        let regex = |option: &str, pattern: &Option<String>| {
            pattern
                .as_deref()
                .map(|p| {
                    RegexBuilder::new(p)
                        .case_insensitive(true)
                        .build()
                        .map_err(|e| format!("{option}: {e}"))
                })
                .transpose()
        };
        let time = |t: Option<i64>| t.and_then(|t| u64::try_from(t).ok());
        Ok(Filters {
            key: opts.key.clone(),
            key_regex: regex("--key-regex", &opts.key_regex)?,
            value: opts.value.clone(),
            value_regex: regex("--value-regex", &opts.value_regex)?,
            data_regex: regex("--data-regex", &opts.data_regex)?,
            start: time(opts.timeline_start),
            end: time(opts.timeline_end),
        })
    }

    /// Whether values are filtered: then keys without values aren't written.
    fn on_values(&self) -> bool {
        self.value.is_some() || self.value_regex.is_some() || self.data_regex.is_some()
    }

    /// Whether keys are filtered.
    fn on_keys(&self) -> bool {
        self.key.is_some() || self.key_regex.is_some() || self.start.is_some() || self.end.is_some()
    }

    /// Whether a key without the value of `--value` is written as such: when keys are chosen.
    fn absent(&self) -> bool {
        self.value.is_some() && (self.key.is_some() || self.key_regex.is_some())
    }

    fn key_matches(&self, path: &str, written: u64) -> bool {
        self.key_regex.as_ref().is_none_or(|r| r.is_match(path))
            && self.start.is_none_or(|s| written >= s)
            && self.end.is_none_or(|e| written <= e)
    }

    /// Whether `name` is the name `--value` gives.
    fn is_value(&self, name: &str) -> bool {
        self.value
            .as_ref()
            .is_some_and(|v| v.to_uppercase() == name.to_uppercase())
    }

    fn value_matches(&self, name: &str, text: &str) -> bool {
        (self.value.is_none() || self.is_value(name))
            && self.value_regex.as_ref().is_none_or(|r| r.is_match(name))
            && self.data_regex.as_ref().is_none_or(|r| r.is_match(text))
    }

    /// The path `--key` names in `opened`'s hive: without a leading `HKLM\` or `HKCU\` (in full
    /// too) or the hive's name, and with `CurrentControlSet` as the control set Windows used
    /// last, where the hive has no key of those names.
    fn key_path(&self, opened: &Opened) -> Option<String> {
        let key = self.key.as_ref()?;
        let mut parts: Vec<String> = key
            .split('\\')
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect();
        let root = opened.hive.root();
        let is_key = |name: &str| root.as_ref().is_some_and(|r| r.subkey(name).is_some());
        let first = |parts: &[String], names: &[&str]| {
            parts
                .first()
                .is_some_and(|p| names.iter().any(|n| p.eq_ignore_ascii_case(n)) && !is_key(p))
        };
        let hives = ["HKLM", "HKEY_LOCAL_MACHINE", "HKCU", "HKEY_CURRENT_USER"];
        if first(&parts, &hives) {
            parts.remove(0);
        }
        if !opened.hive_type.is_empty() && first(&parts, &[opened.hive_type.as_str()]) {
            parts.remove(0);
        }
        if first(&parts, &["CurrentControlSet"])
            && let Some(n) = current_control_set(&opened.hive)
        {
            parts[0] = format!("ControlSet{n:03}");
        }
        Some(parts.join("\\"))
    }
}

// --- records ----------------------------------------------------------------------------------

/// `--raw-data` and `--qword-as-filetime`.
#[derive(Clone, Copy)]
struct Output {
    raw: bool,
    qword_as_filetime: bool,
}

/// A value's name, type and data.
struct ValueRecord {
    name: String,
    kind: u32,
    /// The size of its data, as it records it.
    size: u32,
    /// `None` when it couldn't all be read.
    data: Option<Data>,
    /// Its bytes (as far as they could be read).
    bytes: Vec<u8>,
}

impl ValueRecord {
    /// A value and what keeps it from being read.
    fn new(
        name: String,
        kind: u32,
        size: u32,
        read: (Vec<u8>, Option<String>),
    ) -> (Self, Option<String>) {
        let (bytes, problem) = read;
        let (data, problem) = match problem {
            Some(problem) => (None, Some(problem)),
            None => {
                let data = decode(kind, &bytes);
                let problem = match &data {
                    Data::Malformed(why) => Some(why.clone()),
                    _ => None,
                };
                (Some(data), problem)
            }
        };
        let record = ValueRecord {
            name,
            kind,
            size,
            data,
            bytes,
        };
        (record, problem)
    }

    /// Its data as JSON, and as text (for CSV and `--data-regex`).
    fn data(&self) -> Option<(Value, String)> {
        Some(match self.data.as_ref()? {
            Data::Text { value, .. } => (json!(value), value.clone()),
            Data::MultiText { values, .. } => (
                json!(values),
                serde_json::to_string(values).unwrap_or_default(),
            ),
            Data::Number(n) => (json!(n), n.to_string()),
            Data::Bytes => (json!(hex(&self.bytes)), hex(&self.bytes)),
            Data::Malformed(_) | Data::Undecoded => return None,
        })
    }

    /// Its number, for `REG_DWORD`, `REG_DWORD_BIG_ENDIAN` and `REG_QWORD`.
    fn number(&self) -> Option<u64> {
        match self.data {
            Some(Data::Number(n)) => Some(n),
            _ => None,
        }
    }

    /// A `REG_QWORD` as a FILETIME, when asked for.
    fn filetime(&self, output: Output) -> Option<String> {
        (output.qword_as_filetime && ValueType::from_code(self.kind) == ValueType::Qword)
            .then(|| self.number().and_then(filetime))
            .flatten()
    }

    /// Whether its text was decoded with a byte left over or an invalid UTF-16 code unit
    /// replaced.
    fn lossy(&self) -> bool {
        match &self.data {
            Some(Data::Text { lossy, .. } | Data::MultiText { lossy, .. }) => *lossy,
            _ => false,
        }
    }

    /// Its bytes in hex: when its data doesn't decode or decodes with losses, or when asked for.
    fn raw(&self, output: Output) -> Option<String> {
        let undecoded = self.data().is_none() && !self.bytes.is_empty();
        (output.raw || undecoded || self.lossy()).then(|| hex(&self.bytes))
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// What a record is.
enum Body {
    /// A key without values, or with damage.
    Key,
    /// A value (the default value when its name is empty).
    Value(ValueRecord),
    /// The value `--value` names, which the key hasn't: its name as asked for.
    Absent(String),
}

/// One record to write.
struct Record<'a> {
    opened: &'a Opened,
    /// `None` for a deleted value whose key is unknown.
    key_path: Option<&'a str>,
    key_written: Option<u64>,
    body: Body,
    deleted: bool,
    /// Whether the logs' replay wrote its cells.
    replayed: bool,
    problems: Vec<String>,
}

const CSV_HEADER: [&str; 17] = [
    "Host",
    "HiveType",
    "SourcePath",
    "RecordType",
    "KeyPath",
    "KeyLastWrittenUTC",
    "ValueName",
    "ValueType",
    "ValueData",
    "DataSize",
    "ValueDataRaw",
    "ValueU64",
    "ValueFiletimeUTC",
    "AllocationStatus",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

impl Record<'_> {
    fn record_type(&self) -> &'static str {
        match &self.body {
            Body::Key => "key",
            Body::Value(v) if v.name.is_empty() => "default_value",
            Body::Value(_) => "value",
            Body::Absent(_) => "absent_value",
        }
    }

    fn value(&self) -> Option<&ValueRecord> {
        match &self.body {
            Body::Value(v) => Some(v),
            _ => None,
        }
    }

    fn value_name(&self) -> Option<&str> {
        match &self.body {
            Body::Key => None,
            Body::Value(v) => Some(&v.name),
            Body::Absent(name) => Some(name),
        }
    }

    fn allocation(&self) -> &'static str {
        if self.deleted { "deleted" } else { "allocated" }
    }

    fn csv(&self, output: Output) -> Vec<String> {
        let v = self.value();
        let o = self.opened;
        let text = |s: Option<String>| s.unwrap_or_default();
        vec![
            text(o.host.clone()),
            o.hive_type.clone(),
            o.source.clone(),
            self.record_type().to_string(),
            text(self.key_path.map(str::to_string)),
            text(self.key_written.and_then(filetime)),
            text(self.value_name().map(str::to_string)),
            text(v.map(|v| ValueType::from_code(v.kind).name())),
            text(v.and_then(ValueRecord::data).map(|(_, t)| t)),
            text(v.map(|v| v.size.to_string())),
            text(v.and_then(|v| v.raw(output))),
            text(v.and_then(ValueRecord::number).map(|n| n.to_string())),
            text(v.and_then(|v| v.filetime(output))),
            self.allocation().to_string(),
            o.log_status(self.replayed).to_string(),
            o.recovery.logs_applied.join("|"),
            self.problems.join("; "),
        ]
    }

    fn json(&self, output: Output) -> JsonRecord<'_> {
        let v = self.value();
        let o = self.opened;
        JsonRecord {
            host: o.host.as_deref(),
            hive_type: Some(o.hive_type.as_str()).filter(|t| !t.is_empty()),
            source_path: &o.source,
            record_type: self.record_type(),
            key_path: self.key_path,
            key_last_written: self.key_written.and_then(filetime),
            value_name: self.value_name(),
            value_type: v.map(|v| ValueType::from_code(v.kind).name()),
            value_data: v.and_then(ValueRecord::data).map(|(d, _)| d),
            data_size: v.map(|v| v.size),
            value_data_raw: v.and_then(|v| v.raw(output)),
            value_u64: v.and_then(ValueRecord::number),
            value_filetime_utc: v.and_then(|v| v.filetime(output)),
            allocation_status: self.allocation(),
            transaction_log_status: o.log_status(self.replayed),
            logs_applied: &o.recovery.logs_applied,
            problems: &self.problems,
        }
    }
}

/// A record in JSON, its fields in this order.
#[derive(Serialize)]
struct JsonRecord<'a> {
    host: Option<&'a str>,
    hive_type: Option<&'a str>,
    source_path: &'a str,
    record_type: &'static str,
    key_path: Option<&'a str>,
    key_last_written: Option<String>,
    value_name: Option<&'a str>,
    value_type: Option<String>,
    value_data: Option<Value>,
    data_size: Option<u32>,
    value_data_raw: Option<String>,
    value_u64: Option<u64>,
    value_filetime_utc: Option<String>,
    allocation_status: &'static str,
    transaction_log_status: &'static str,
    logs_applied: &'a [String],
    problems: &'a [String],
}

/// Where records go, and what is counted of them.
struct Out<'w, 's> {
    writer: Option<&'w mut RecordWriter>,
    output: Output,
    stats: &'s mut Stats,
}

impl Out<'_, '_> {
    fn write(&mut self, r: &Record) -> io::Result<()> {
        let s = &mut *self.stats;
        s.records += 1;
        s.replayed += usize::from(r.replayed && r.opened.recovery.recovered);
        s.deleted_records += usize::from(r.deleted);
        s.absent += usize::from(matches!(r.body, Body::Absent(_)));
        if let Some(w) = r.key_written {
            s.first = Some(s.first.map_or(w, |f| f.min(w)));
            s.last = Some(s.last.map_or(w, |l| l.max(w)));
        }
        match self.writer.as_deref_mut() {
            Some(w) if w.is_csv() => w.csv(&r.csv(self.output)),
            Some(w) => w.json(&r.json(self.output)),
            None => Ok(()),
        }
    }

    /// The keys and values of `opened` that pass `filters`.
    fn hive(&mut self, opened: &Opened, filters: &Filters) -> io::Result<()> {
        self.stats.hive(opened);
        if let Some(path) = filters.key_path(opened) {
            // one key: no walk
            let Some(key) = opened.hive.key(&path) else {
                self.stats.without_key += 1;
                return Ok(());
            };
            return self.key(opened, filters, &key, &path, Vec::new());
        }
        for entry in opened.hive.walk() {
            self.key(opened, filters, &entry.key, &entry.path, entry.problems)?;
        }
        Ok(())
    }

    /// A key in use and its values.
    fn key(
        &mut self,
        opened: &Opened,
        filters: &Filters,
        key: &Key,
        path: &str,
        mut problems: Vec<String>,
    ) -> io::Result<()> {
        self.stats.keys += 1;
        let written = key.last_written();
        let (values, value_problems) = key.values();
        problems.extend(value_problems);
        for problem in &problems {
            self.stats.problem(&opened.source, Some(path), problem);
        }
        self.stats.values += values.len();
        if !filters.key_matches(path, written) {
            return Ok(());
        }
        let key_replayed = key.status() == Status::Replayed;
        if (values.is_empty() || !problems.is_empty()) && !filters.on_values() {
            self.write(&Record {
                opened,
                key_path: Some(path),
                key_written: Some(written),
                body: Body::Key,
                deleted: false,
                replayed: key_replayed,
                problems,
            })?;
        }
        let mut named = false;
        for value in values {
            let (record, problem) = value_record(&value);
            named |= filters.is_value(&record.name);
            self.value(
                opened,
                filters,
                Some(path),
                Some(written),
                record,
                problem,
                false,
                value.status() == Status::Replayed,
            )?;
        }
        if filters.absent() && !named {
            self.write(&Record {
                opened,
                key_path: Some(path),
                key_written: Some(written),
                body: Body::Absent(filters.value.clone().unwrap_or_default()),
                deleted: false,
                replayed: key_replayed,
                problems: Vec::new(),
            })?;
        }
        Ok(())
    }

    /// A value, written when it passes the value filters.
    #[allow(clippy::too_many_arguments)]
    fn value(
        &mut self,
        opened: &Opened,
        filters: &Filters,
        key_path: Option<&str>,
        key_written: Option<u64>,
        record: ValueRecord,
        problem: Option<String>,
        deleted: bool,
        replayed: bool,
    ) -> io::Result<()> {
        if problem.is_some() && deleted {
            self.stats.lost += 1;
        } else if let Some(problem) = &problem {
            let name = if record.name.is_empty() {
                "(default)"
            } else {
                &record.name
            };
            self.stats.problem(
                &opened.source,
                key_path,
                &format!("value {name}: {problem}"),
            );
        }
        let text = record
            .data()
            .map_or_else(|| hex(&record.bytes), |(_, text)| text);
        if !filters.value_matches(&record.name, &text) {
            return Ok(());
        }
        self.write(&Record {
            opened,
            key_path,
            key_written,
            body: Body::Value(record),
            deleted,
            replayed,
            problems: problem.into_iter().collect(),
        })
    }

    /// The deleted keys and values of `opened` that pass `filters`.
    fn deleted(&mut self, opened: &Opened, filters: &Filters) -> io::Result<()> {
        let deleted = opened.hive.deleted();
        let key = filters.key_path(opened).map(|k| k.to_uppercase());
        for k in &deleted.keys {
            self.stats.deleted_keys += 1;
            self.stats.deleted_values += k.values.len();
            self.stats.lost += k.problems.len();
            let path_matches = key.as_ref().is_none_or(|p| k.path.to_uppercase() == *p);
            if !path_matches || !filters.key_matches(&k.path, k.last_written) {
                continue;
            }
            if (k.values.is_empty() || !k.problems.is_empty()) && !filters.on_values() {
                self.write(&Record {
                    opened,
                    key_path: Some(&k.path),
                    key_written: Some(k.last_written),
                    body: Body::Key,
                    deleted: true,
                    replayed: k.replayed,
                    problems: k.problems.clone(),
                })?;
            }
            for v in &k.values {
                let (record, problem) = deleted_record(v);
                self.value(
                    opened,
                    filters,
                    Some(&k.path),
                    Some(k.last_written),
                    record,
                    problem,
                    true,
                    v.replayed,
                )?;
            }
        }
        self.stats.deleted_values += deleted.values.len();
        // values no deleted key lists: their key is unknown, so no key filter takes them
        if filters.on_keys() {
            return Ok(());
        }
        for v in &deleted.values {
            let (record, problem) = deleted_record(v);
            self.value(
                opened, filters, None, None, record, problem, true, v.replayed,
            )?;
        }
        Ok(())
    }
}

fn value_record(value: &RegValue) -> (ValueRecord, Option<String>) {
    ValueRecord::new(
        value.name(),
        value.data_type(),
        value.size(),
        value.read_data(),
    )
}

fn deleted_record(value: &DeletedValue) -> (ValueRecord, Option<String>) {
    ValueRecord::new(
        value.name.clone(),
        value.data_type,
        value.size,
        (value.data.clone(), value.problem.clone()),
    )
}

// --- summary ----------------------------------------------------------------------------------

/// Counts for the results summary.
#[derive(Default)]
struct Stats {
    hives: usize,
    failed: usize,
    types: BTreeMap<String, usize>,
    hosts: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    without_key: usize,
    keys: usize,
    values: usize,
    deleted_keys: usize,
    deleted_values: usize,
    /// Parts of deleted keys and values whose cells are in use again.
    lost: usize,
    records: usize,
    replayed: usize,
    deleted_records: usize,
    absent: usize,
    first: Option<u64>,
    last: Option<u64>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    fn hive(&mut self, opened: &Opened) {
        self.hives += 1;
        let kind = if opened.hive_type.is_empty() {
            "other"
        } else {
            opened.hive_type.as_str()
        };
        *self.types.entry(kind.to_string()).or_default() += 1;
        if let Some(host) = &opened.host {
            self.hosts.insert(host.clone());
        }
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
    }

    fn problem(&mut self, source: &str, path: Option<&str>, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED_PROBLEMS {
            let key = match path {
                Some("") => "(root)",
                Some(path) => path,
                None => "(unknown key)",
            };
            self.problems.push(format!("{source}: {key}: {problem}"));
        }
    }

    fn print(&self, filters: &Filters, deleted: bool, no_color: bool) {
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        p_label(no_color, "Registry hives parsed: ", &fmt_num(self.hives));
        if self.failed > 0 {
            p_label(no_color, "Could not parse: ", &fmt_num(self.failed));
        }
        if !self.types.is_empty() {
            let list: Vec<String> = self
                .types
                .iter()
                .map(|(t, n)| format!("{t} {}", fmt_num(*n)))
                .collect();
            p_label(no_color, "Hive types: ", &list.join(", "));
        }
        if !self.hosts.is_empty() {
            let list: Vec<&str> = self.hosts.iter().map(String::as_str).collect();
            p_label(no_color, "Hosts: ", &list.join(", "));
        }
        p_label(
            no_color,
            "Dirty hives recovered with their transaction logs: ",
            &fmt_num(self.recovered),
        );
        if !self.dirty.is_empty() {
            p_label(
                no_color,
                "Dirty hives whose transaction logs were not applied (keys or values may be missing): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED_PROBLEMS) {
                p(None, &format!("  {source}"), true);
            }
        }
        if self.without_key > 0 {
            p_label(
                no_color,
                "Hives without the key of --key: ",
                &fmt_num(self.without_key),
            );
        }
        p_label(
            no_color,
            "Keys: ",
            &format!("{}, values: {}", fmt_num(self.keys), fmt_num(self.values)),
        );
        if deleted {
            let mut found = format!(
                "{}, values: {}",
                fmt_num(self.deleted_keys),
                fmt_num(self.deleted_values)
            );
            if self.lost > 0 {
                found.push_str(&format!(
                    " (incomplete deleted value lists, values or data: {})",
                    fmt_num(self.lost)
                ));
            }
            p_label(no_color, "Deleted keys: ", &found);
        }
        let label = if filters.on_keys() || filters.on_values() {
            "Records matching the filters: "
        } else {
            "Records: "
        };
        let mut records = fmt_num(self.records);
        let mut parts = Vec::new();
        if self.replayed > 0 {
            parts.push(format!(
                "replayed from the transaction logs: {}",
                fmt_num(self.replayed)
            ));
        }
        if self.deleted_records > 0 {
            parts.push(format!("deleted: {}", fmt_num(self.deleted_records)));
        }
        if self.absent > 0 {
            parts.push(format!(
                "keys without the value of --value: {}",
                fmt_num(self.absent)
            ));
        }
        if !parts.is_empty() {
            records.push_str(&format!(" ({})", parts.join(", ")));
        }
        p_label(no_color, label, &records);
        if let (Some(first), Some(last)) = (self.first, self.last)
            && let (Some(first), Some(last)) = (filetime(first), filetime(last))
        {
            p_label(
                no_color,
                "Key last-write times: ",
                &format!("{first} - {last}"),
            );
        }
        p_label(
            no_color,
            "Damaged keys and values: ",
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

    fn record(kind: u32, bytes: &[u8]) -> ValueRecord {
        ValueRecord::new("v".into(), kind, bytes.len() as u32, (bytes.to_vec(), None)).0
    }

    #[test]
    fn values_by_type() {
        let utf16 = |s: &str| -> Vec<u8> { s.encode_utf16().flat_map(u16::to_le_bytes).collect() };
        let output = Output {
            raw: false,
            qword_as_filetime: true,
        };
        let sz = record(1, &utf16("abc\0junk"));
        assert_eq!(sz.data(), Some((json!("abc"), "abc".into())));
        assert_eq!(sz.number(), None);
        let multi = record(7, &utf16("a\0b c\0\0"));
        assert_eq!(
            multi.data(),
            Some((json!(["a", "b c"]), r#"["a","b c"]"#.into()))
        );
        let dword = record(4, &[1, 0, 0, 0]);
        assert_eq!(dword.number(), Some(1));
        // only a REG_QWORD is taken for a FILETIME
        assert_eq!(dword.filetime(output), None);
        let qword = record(11, &132_888_278_320_000_000u64.to_le_bytes());
        assert_eq!(qword.number(), Some(132_888_278_320_000_000));
        assert_eq!(
            qword.filetime(output).as_deref(),
            Some("2022-02-08T21:03:52Z")
        );
        let off = Output {
            raw: false,
            qword_as_filetime: false,
        };
        assert_eq!(qword.filetime(off), None);
        let binary = record(3, &[0xab, 1]);
        assert_eq!(binary.data(), Some((json!("ab01"), "ab01".into())));
        assert_eq!(binary.raw(off), None);
        let short = ValueRecord::new("v".into(), 4, 3, (vec![1, 2, 3], None));
        assert_eq!(short.1.as_deref(), Some("REG_DWORD of 3 bytes"));
        assert_eq!(short.0.data(), None);
        assert_eq!(short.0.raw(off).as_deref(), Some("010203"));
        // text decoded with losses keeps its bytes
        let odd = record(1, b"A\0B");
        assert_eq!(odd.data(), Some((json!("A"), "A".into())));
        assert_eq!(odd.raw(off).as_deref(), Some("410042"));
        let unpaired = record(1, &[0x00, 0xd8]);
        assert_eq!(unpaired.raw(off).as_deref(), Some("00d8"));
        let replacement = record(1, &utf16("\u{fffd}"));
        assert_eq!(
            replacement.data(),
            Some((json!("\u{fffd}"), "\u{fffd}".into()))
        );
        assert_eq!(replacement.raw(off), None);
    }
}
