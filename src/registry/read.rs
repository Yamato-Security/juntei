//! `parse-registry read`: the keys and values of registry hives as they are, with Hayabusa's input
//! options, progress bar and results summary. One record per value, and one per key without
//! values or with damage: the key's last-write time, the value's type, its data decoded (strings,
//! multi-strings, numbers, binary data in hex) and, when it doesn't decode or `--raw-data` asks,
//! its bytes; numbers also as `value_u64`, and with `--qword-as-filetime` a `REG_QWORD` also as a
//! FILETIME. Each record says where it came from: allocated, or recovered (`--recover-deleted`:
//! from free cells, cell slack, the data past the hive bins, or what only the transaction logs
//! kept; complete, partial, a value without its key, or only in the logs, and the places it was
//! found), and whether the transaction logs' replay wrote it. Filters on the key path,
//! value names and data (regular expressions ignoring case, so that a GUID matches in any case)
//! and last-write times keep a targeted query from writing the whole hive; `--value` with a key
//! filter also says which keys lack the value.

use super::{
    Hosts, Opened, RecordWriter, choose_logs, filetime, find_hives, input_files, open, sink,
};
use crate::cli::RegistryReadOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use indicatif::ProgressBar;
use regex::{Regex, RegexBuilder};
use registry::system::{ControlSets, control_set_name};
use registry::{
    Data, Key, RecoveredStatus, RecoveredValue, Source, Status, Value as RegValue, ValueType,
    decode,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io;
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
    let given_logs: Vec<PathBuf> = opts.log1.iter().chain(&opts.log2).cloned().collect();
    let inputs = input_files(
        opts.filepath.as_ref(),
        opts.directory.as_ref(),
        "registry hives",
        find_hives,
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
                    written = out.recovered(&opened, &filters);
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
            && let Some(n) = ControlSets::read(&opened.hive).current
        {
            parts[0] = control_set_name(n);
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
    /// For a recovered record: how complete it is, and where it was found.
    recovered: Option<Recovered>,
    /// Whether the logs' replay wrote its cells.
    replayed: bool,
    problems: Vec<String>,
}

/// A recovered record's status and the places it was found (`free_cell`, `SYSTEM.LOG1#1234`,
/// …).
#[derive(Clone)]
struct Recovered {
    status: RecoveredStatus,
    from: Vec<String>,
}

impl Recovered {
    fn of(status: RecoveredStatus, sources: &[Source]) -> Self {
        Recovered {
            status,
            from: sources.iter().map(Source::name).collect(),
        }
    }
}

const CSV_HEADER: [&str; 18] = [
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
    "RecoveredFrom",
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

    /// `allocated`, or how complete a recovered record is.
    fn allocation(&self) -> &'static str {
        self.recovered
            .as_ref()
            .map_or("allocated", |r| r.status.name())
    }

    fn recovered_from(&self) -> &[String] {
        self.recovered.as_ref().map_or(&[], |r| &r.from)
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
            self.recovered_from().join("|"),
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
            recovered_from: self.recovered_from(),
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
    recovered_from: &'a [String],
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
        s.recovered_records += usize::from(r.recovered.is_some());
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
                recovered: None,
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
                None,
                value.status() == Status::Replayed,
            )?;
        }
        if filters.absent() && !named {
            self.write(&Record {
                opened,
                key_path: Some(path),
                key_written: Some(written),
                body: Body::Absent(filters.value.clone().unwrap_or_default()),
                recovered: None,
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
        recovered: Option<Recovered>,
        replayed: bool,
    ) -> io::Result<()> {
        if problem.is_some() && recovered.is_some() {
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
            recovered,
            replayed,
            problems: problem.into_iter().collect(),
        })
    }

    /// The keys and values recovered from `opened` (its free cells, the slack of its cells,
    /// the data past its hive bins, and what only its transaction logs kept) that pass
    /// `filters`.
    fn recovered(&mut self, opened: &Opened, filters: &Filters) -> io::Result<()> {
        let found = opened.hive.recover();
        let key = filters.key_path(opened).map(|k| k.to_uppercase());
        for k in &found.keys {
            if !k.in_use {
                self.stats.recovered_key(k.status, k.problems.len());
            }
            for v in &k.values {
                self.stats.recovered_value(v.status);
            }
            let path_matches = key.as_ref().is_none_or(|p| k.path.to_uppercase() == *p);
            if !path_matches || !filters.key_matches(&k.path, k.last_written) {
                continue;
            }
            // an earlier version of a key in use: only the values it held otherwise
            let lone = k.values.is_empty() || !k.problems.is_empty();
            if !k.in_use && lone && !filters.on_values() {
                self.write(&Record {
                    opened,
                    key_path: Some(&k.path),
                    key_written: Some(k.last_written),
                    body: Body::Key,
                    recovered: Some(Recovered::of(k.status, &k.sources)),
                    replayed: k.replayed,
                    problems: k.problems.clone(),
                })?;
            }
            for v in &k.values {
                let (record, problem) = recovered_record(v);
                self.value(
                    opened,
                    filters,
                    Some(&k.path),
                    Some(k.last_written),
                    record,
                    problem,
                    Some(Recovered::of(v.status, &v.sources)),
                    v.replayed,
                )?;
            }
        }
        for v in &found.values {
            self.stats.recovered_value(v.status);
        }
        // values without their key: no key filter takes them
        if filters.on_keys() {
            return Ok(());
        }
        for v in &found.values {
            let (record, problem) = recovered_record(v);
            self.value(
                opened,
                filters,
                None,
                None,
                record,
                problem,
                Some(Recovered::of(v.status, &v.sources)),
                v.replayed,
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

fn recovered_record(value: &RecoveredValue) -> (ValueRecord, Option<String>) {
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
    /// Recovered keys (not earlier versions of keys in use) and values, by status.
    recovered_keys: BTreeMap<RecoveredStatus, usize>,
    recovered_values: BTreeMap<RecoveredStatus, usize>,
    /// Parts of recovered keys and values that were lost.
    lost: usize,
    records: usize,
    replayed: usize,
    recovered_records: usize,
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

    fn recovered_key(&mut self, status: RecoveredStatus, lost: usize) {
        *self.recovered_keys.entry(status).or_default() += 1;
        self.lost += lost;
    }

    fn recovered_value(&mut self, status: RecoveredStatus) {
        *self.recovered_values.entry(status).or_default() += 1;
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
            let count = |map: &BTreeMap<RecoveredStatus, usize>| {
                let total: usize = map.values().sum();
                let each: Vec<String> = map
                    .iter()
                    .map(|(status, n)| format!("{} {}", status.name(), fmt_num(*n)))
                    .collect();
                if each.is_empty() {
                    fmt_num(total)
                } else {
                    format!("{} ({})", fmt_num(total), each.join(", "))
                }
            };
            p_label(no_color, "Recovered keys: ", &count(&self.recovered_keys));
            p_label(
                no_color,
                "Recovered values: ",
                &count(&self.recovered_values),
            );
            if self.lost > 0 {
                p_label(
                    no_color,
                    "Value lists, values or data of recovered records lost: ",
                    &fmt_num(self.lost),
                );
            }
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
        if self.recovered_records > 0 {
            parts.push(format!("recovered: {}", fmt_num(self.recovered_records)));
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
