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

//! `parse-file defender-history`: Microsoft Defender's detection history
//! (`ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>`),
//! read with Yamato Security's ys-defender-history-parser. Defender writes one file per detection,
//! also for threats it never quarantined: the threat (identifier, name, severity, category), its
//! status, the error its last status change ended with, whether it was running, the user and
//! process involved, what detected it, the quarantine entry, its times, and every resource it was
//! found in (files, containers, downloads, processes, registry values, command lines, behaviors)
//! with that resource's tracking data (hashes, size, tracking time).
//!
//! One record per resource of each detection (one for a detection without resources), so that a
//! detection's resources stay together under its `DetectionID` and repeated detections of one
//! threat stay separate. Every value is kept: the values whose meaning isn't known by their
//! section and position, the tracking data with its own keys and types. Nothing is guessed: the
//! times are the ones Defender wrote, never the file's own times.

use crate::cli::{DefenderHistoryOptions, DefenderHistoryOutputType};
use crate::color::JunteiColor;
use crate::powershell_history::parts;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use defender::{Detection, Resource, Time, Value};
use indicatif::ProgressBar;
use serde_json::{Value as Json, json};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// Files that could not be read are printed up to this many; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Threats listed in the summary.
const MAX_PRINTED_THREATS: usize = 10;
/// Larger files are not detection history files (those seen are a few KiB), and are not read.
const MAX_FILE_SIZE: u64 = 64 * 1024 * 1024;
/// The folder Defender keeps its detection history files in, one subfolder per number.
const HISTORY_FOLDER: &str = "DetectionHistory";
/// The bytes [`defender::detect`] needs to recognize a detection history file.
const HEAD_BYTES: usize = 0x30;

/// The tracking values that have a CSV column of their own; the others are in `OtherTracking`.
const TRACKING_COLUMNS: [&str; 6] = [
    "ThreatTrackingStartTime",
    "ThreatTrackingSha256",
    "ThreatTrackingSha1",
    "ThreatTrackingMD5",
    "ThreatTrackingSize",
    "ThreatTrackingId",
];

/// The files `-d`, `auto-parse` and disk-image staging select: files in a `DetectionHistory`
/// folder, at any depth below it. The file's content decides whether it is read as one.
pub(crate) fn is_history_path(path: &Path) -> bool {
    let parts = parts(path);
    parts.split_last().is_some_and(|(_, folders)| {
        folders
            .iter()
            .any(|f| f.eq_ignore_ascii_case(HISTORY_FOLDER))
    })
}

/// One detection history file to read.
struct Source {
    path: PathBuf,
    /// `SourceFile`: the path below the input directory with `/` between its names, or the name
    /// of a file given with `-f`. Never an absolute path.
    shown: String,
}

impl Source {
    fn under(root: &Path, path: PathBuf) -> Self {
        let shown = parts(path.strip_prefix(root).unwrap_or(&path)).join("/");
        Self { path, shown }
    }

    fn given(path: PathBuf) -> Self {
        let shown = parts(&path).last().cloned().unwrap_or_default();
        Self { path, shown }
    }
}

/// The candidate files under `dir`, in path order. The `DetectionHistory` folder is looked for in
/// the absolute path, so that `-d` given that folder or one of its numbered subfolders finds its
/// files too. A directory index that image staging writes as `$I30` into a folder is no
/// detection history file (Defender names them by ID) and is left out.
fn find_histories(dir: &Path) -> Vec<Source> {
    util::find_files(dir, |name| !name.eq_ignore_ascii_case("$I30"))
        .into_iter()
        .filter(|path| is_history_path(&std::path::absolute(path).unwrap_or(path.clone())))
        .map(|path| Source::under(dir, path))
        .collect()
}

/// Why a file gave no records.
enum Skip {
    /// Not a detection history file (another file in the folder, an empty placeholder).
    NotHistory(String),
    /// It could not be read.
    Unreadable(String),
}

/// Read one file, bounded in size.
fn read_file(path: &Path) -> Result<Vec<u8>, Skip> {
    let size = path
        .metadata()
        .map_err(|e| Skip::Unreadable(e.to_string()))?
        .len();
    if size > MAX_FILE_SIZE {
        return Err(Skip::NotHistory(format!(
            "too large for a detection history file ({size} bytes)"
        )));
    }
    let mut data = Vec::with_capacity(size as usize);
    File::open(path)
        .and_then(|f| f.take(MAX_FILE_SIZE).read_to_end(&mut data))
        .map_err(|e| Skip::Unreadable(e.to_string()))?;
    if !defender::detect(&data[..data.len().min(HEAD_BYTES)]) {
        return Err(Skip::NotHistory(if data.is_empty() {
            "empty".into()
        } else {
            "does not start like a detection history file".into()
        }));
    }
    Ok(data)
}

/// Read one file's detection; its problems include the file's own.
fn read_detection(source: &Source) -> Result<Detection, Skip> {
    let data = read_file(&source.path)?;
    let mut detection = defender::read(&data).map_err(|e| Skip::NotHistory(e.to_string()))?;
    let name = source
        .path
        .file_name()
        .map(|n| {
            n.to_string_lossy()
                .trim_matches(['{', '}'])
                .to_ascii_uppercase()
        })
        .unwrap_or_default();
    if let Some(id) = &detection.id
        && *id != name
    {
        detection
            .problems
            .push(format!("its DetectionID ({id}) isn't its file's name"));
    }
    if source.path.to_str().is_none() {
        detection.problems.push(
            "The source path contains non-Unicode bytes; its displayed spelling uses U+FFFD".into(),
        );
    }
    Ok(detection)
}

/// Run the command; returns the exit status.
pub fn run(opts: &DefenderHistoryOptions) -> i32 {
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
    let sources = match (&opts.filepath, &opts.directory) {
        (Some(f), _) => {
            if !f.is_file() {
                report_error(None, &format!("The file does not exist: {}", f.display()));
                return 1;
            }
            vec![Source::given(f.clone())]
        }
        (None, Some(d)) => {
            if !d.is_dir() {
                report_error(
                    None,
                    &format!("The directory does not exist: {}", d.display()),
                );
                return 1;
            }
            let found = find_histories(d);
            if found.is_empty() {
                report_error(
                    None,
                    &format!(
                        "No Defender detection history files (DetectionHistory\\<nn>\\<GUID>) were found in {}",
                        d.display()
                    ),
                );
                return 1;
            }
            found
        }
        (None, None) => unreachable!("clap requires -f or -d"),
    };
    // An output that is one of the inputs (by path, symbolic link or hard link) is refused
    // before anything is written, also with --clobber.
    let sink =
        match crate::registry::sink(opts.output.as_ref(), opts.clobber, data_on_stdout, || {
            sources.iter().map(|source| source.path.clone()).collect()
        }) {
            Ok(sink) => sink,
            Err(msg) => {
                report_error(None, &msg);
                return 1;
            }
        };
    let mut records = match sink
        .map(|sink| Records::new(sink, opts.output_type()))
        .transpose()
    {
        Ok(records) => records,
        Err(e) => {
            report_error(None, &format!("Could not write the records: {e}"));
            return 1;
        }
    };
    if !data_on_stdout {
        let total: u64 = sources
            .iter()
            .map(|source| source.path.metadata().map_or(0, |m| m.len()))
            .sum();
        p_label(
            no_color,
            "Detection history files: ",
            &fmt_num(sources.len()),
        );
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
        progress::progress_bar(sources.len(), no_color)
    });

    // --- read and write ---------------------------------------------------------------------
    let mut stats = Stats::default();
    let mut printed = 0usize;
    for source in &sources {
        if let Some(b) = &bar {
            b.set_message(progress::message(&source.path));
        }
        match read_detection(source) {
            Ok(detection) => {
                stats.count(&detection);
                if let Some(records) = records.as_mut()
                    && let Err(e) = records.write(source, &detection)
                {
                    // The output is gone (a full disk, a closed pipe): reading on cannot save
                    // anything.
                    if let Some(b) = &bar {
                        b.abandon();
                    }
                    report_error(None, &format!("Could not write the records: {e}"));
                    return 1;
                }
            }
            Err(skip) => {
                let message = match skip {
                    Skip::NotHistory(why) => {
                        stats.not_history += 1;
                        format!(
                            "Not read as a detection history file: {}: {why}",
                            source.shown
                        )
                    }
                    Skip::Unreadable(why) => {
                        stats.unreadable += 1;
                        format!("Could not read {}: {why}", source.shown)
                    }
                };
                printed += 1;
                if printed <= MAX_PRINTED_ERRORS {
                    report_error(bar.as_ref(), &message);
                }
            }
        }
        if let Some(b) = &bar {
            b.inc(1);
        }
    }
    if printed > MAX_PRINTED_ERRORS {
        report_error(
            bar.as_ref(),
            &format!(
                "{} more files could not be read",
                fmt_num(printed - MAX_PRINTED_ERRORS)
            ),
        );
    }
    if let Some(b) = &bar {
        b.finish_with_message(progress::finished_message(no_color));
    }
    if let Some(records) = records
        && let Err(e) = records.finish()
    {
        report_error(None, &format!("Could not write the records: {e}"));
        return 1;
    }

    // --- summary ---------------------------------------------------------------------------
    if !data_on_stdout {
        print_summary(&stats, no_color);
        if stats.detections > 0 {
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
    }
    i32::from(stats.detections == 0)
}

// --- output ---------------------------------------------------------------------------------

const CSV_HEADER: &[&str] = &[
    "Record",
    "InitialDetectionTime",
    "LastThreatStatusChangeTime",
    "RemediationTime",
    "ThreatName",
    "ThreatID",
    "Severity",
    "SeverityID",
    "Category",
    "CategoryID",
    "ThreatStatus",
    "ThreatStatusID",
    "ThreatStatusErrorCode",
    "ExecutionStatus",
    "ExecutionStatusID",
    "DetectionSource",
    "DetectionSourceTypeID",
    "User",
    "UserSource",
    "Process",
    "ResourceIndex",
    "ResourceCount",
    "ResourceType",
    "Path",
    "ResourceFlags",
    "TrackingStartTime",
    "SHA256",
    "SHA1",
    "MD5",
    "TrackingSize",
    "TrackingID",
    "OtherTracking",
    "QuarantineID",
    "DetectionID",
    "MagicVersion",
    "UnknownValues",
    "SourceFile",
    "Problems",
];

/// One record: a detection and one of its resources (`None` for a detection without any).
struct Row<'a> {
    source: &'a Source,
    detection: &'a Detection,
    resource: Option<(usize, &'a Resource)>,
}

impl Row<'_> {
    fn kind(&self) -> &'static str {
        if self.resource.is_some() {
            "resource"
        } else {
            "detection"
        }
    }

    fn tracking(&self, key: &str) -> Option<&Value> {
        self.resource?
            .1
            .tracking
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// The tracking data's start time, read as a FILETIME, as plaso dates a detection.
    fn tracking_start(&self) -> Option<Time> {
        Time::from_filetime(self.tracking("ThreatTrackingStartTime")?.as_u64()?)
    }

    /// The user Defender recorded (`DomainUser`), when it recorded one.
    fn user(&self) -> Option<&str> {
        self.detection.user.as_deref().filter(|u| !u.is_empty())
    }
}

fn time(t: Option<Time>) -> String {
    t.map(Time::to_iso8601).unwrap_or_default()
}

fn opt<T: ToString>(v: Option<T>) -> String {
    v.map(|v| v.to_string()).unwrap_or_default()
}

fn flags(v: Option<u32>) -> String {
    v.map_or_else(String::new, |f| format!("0x{f:x}"))
}

/// The values whose meaning isn't known, as `position=value`; the counted list after the
/// threat's status first, as `2.list`.
fn unknown_text(d: &Detection) -> String {
    let mut parts = Vec::with_capacity(d.unknown.len() + 1);
    if !d.list.is_empty() {
        let list: Vec<String> = d.list.iter().map(u32::to_string).collect();
        parts.push(format!("2.list={}", list.join(",")));
    }
    parts.extend(d.unknown.iter().map(|(k, v)| format!("{k}={v}")));
    parts.join("; ")
}

fn csv_row(r: &Row<'_>) -> Vec<String> {
    let d = r.detection;
    let res = r.resource.map(|(_, res)| res);
    let tracked = |key: &str| r.tracking(key).map(Value::to_string).unwrap_or_default();
    let other: Vec<String> = res
        .map(|res| {
            res.tracking
                .iter()
                .filter(|(k, _)| !TRACKING_COLUMNS.contains(&k.as_str()))
                .map(|(k, v)| format!("{k}={v}"))
                .collect()
        })
        .unwrap_or_default();
    vec![
        r.kind().to_owned(),
        time(d.initial_detection),
        time(d.status_changed),
        time(d.remediation),
        opt(d.threat_name.as_deref()),
        opt(d.threat_id),
        opt(d.severity_name()),
        opt(d.severity_id),
        opt(d.category_name()),
        opt(d.category_id),
        opt(d.threat_status_name()),
        opt(d.threat_status_id),
        opt(d.status_error_code),
        opt(d.execution_status_name()),
        opt(d.execution_status_id),
        opt(d.source_type_name()),
        opt(d.source_type_id),
        opt(r.user()),
        opt(r.user().map(|_| "DomainUser")),
        opt(d.process.as_deref()),
        opt(r.resource.map(|(i, _)| i + 1)),
        d.resources.len().to_string(),
        opt(res.map(|x| x.kind.as_str())),
        opt(res.map(|x| x.location.as_str())),
        flags(res.and_then(|x| x.flags)),
        time(r.tracking_start()),
        tracked("ThreatTrackingSha256"),
        tracked("ThreatTrackingSha1"),
        tracked("ThreatTrackingMD5"),
        tracked("ThreatTrackingSize"),
        tracked("ThreatTrackingId"),
        other.join("; "),
        opt(d.quarantine_id.as_deref()),
        opt(d.id.as_deref()),
        opt(d.version.as_deref()),
        unknown_text(d),
        r.source.shown.clone(),
        d.problems.join("; "),
    ]
}

/// A value with its type, as read: an integer, a signed integer, a FILETIME (its ticks, with the
/// time they read as when they are one), a flag, text, a GUID, or bytes in hexadecimal.
fn typed(v: &Value) -> Json {
    match v {
        Value::Integer(n) => json!({"Type": "Integer", "Value": n}),
        Value::Signed(n) => json!({"Type": "Signed", "Value": n}),
        Value::Time(ticks) => json!({
            "Type": "FILETIME",
            "Value": ticks,
            "Time": Time::from_filetime(*ticks).map(Time::to_iso8601),
        }),
        Value::Bool(b) => json!({"Type": "Bool", "Value": b}),
        Value::Text(t) => json!({"Type": "Text", "Value": t}),
        Value::Guid(t) => json!({"Type": "GUID", "Value": t}),
        Value::Bytes(_) => json!({"Type": "Bytes", "Value": v.to_string()}),
    }
}

/// `typed`, with the name or position it was read under first.
fn keyed(label: &str, name: &str, v: &Value) -> Json {
    let mut object = serde_json::Map::new();
    object.insert(label.into(), json!(name));
    if let Json::Object(fields) = typed(v) {
        object.extend(fields);
    }
    Json::Object(object)
}

fn json_record(r: &Row<'_>) -> Json {
    let d = r.detection;
    let res = r.resource.map(|(_, res)| res);
    let t = |x: Option<Time>| x.map_or(Json::Null, |x| json!(x.to_iso8601()));
    let tracked = |key: &str| {
        r.tracking(key)
            .map_or(Json::Null, |v| typed(v)["Value"].clone())
    };
    json!({
        "Record": r.kind(),
        "InitialDetectionTime": t(d.initial_detection),
        "LastThreatStatusChangeTime": t(d.status_changed),
        "RemediationTime": t(d.remediation),
        "ThreatName": d.threat_name,
        "ThreatID": d.threat_id,
        "Severity": d.severity_name(),
        "SeverityID": d.severity_id,
        "Category": d.category_name(),
        "CategoryID": d.category_id,
        "ThreatStatus": d.threat_status_name(),
        "ThreatStatusID": d.threat_status_id,
        "ThreatStatusErrorCode": d.status_error_code,
        "ExecutionStatus": d.execution_status_name(),
        "ExecutionStatusID": d.execution_status_id,
        "DetectionSource": d.source_type_name(),
        "DetectionSourceTypeID": d.source_type_id,
        "User": r.user(),
        "UserSource": r.user().map(|_| "DomainUser"),
        "Process": d.process,
        "ResourceIndex": r.resource.map(|(i, _)| i + 1),
        "ResourceCount": d.resources.len(),
        "ResourceType": res.map(|x| x.kind.as_str()),
        "Path": res.map(|x| x.location.as_str()),
        "ResourceFlags": res.and_then(|x| x.flags),
        "TrackingStartTime": t(r.tracking_start()),
        "SHA256": tracked("ThreatTrackingSha256"),
        "SHA1": tracked("ThreatTrackingSha1"),
        "MD5": tracked("ThreatTrackingMD5"),
        "TrackingSize": tracked("ThreatTrackingSize"),
        "TrackingID": tracked("ThreatTrackingId"),
        // Every tracking value in the order stored, with Defender's own key.
        "Tracking": res.map_or_else(Vec::new, |x| {
            x.tracking.iter().map(|(k, v)| keyed("Key", k, v)).collect()
        }),
        "QuarantineID": d.quarantine_id,
        "DetectionID": d.id,
        "MagicVersion": d.version,
        "CountedList": d.list,
        "UnknownValues": d
            .unknown
            .iter()
            .map(|(k, v)| keyed("Position", k, v))
            .collect::<Vec<_>>(),
        "SourceFile": r.source.shown,
        "Problems": d.problems,
    })
}

/// Where the records go, written one detection at a time.
enum Records {
    Csv(Box<csv::Writer<Box<dyn Write>>>),
    /// Pretty JSON objects when `true`, else one object per line.
    Json(Box<dyn Write>, bool),
}

impl Records {
    /// A CSV gets its header now, so that it has one also without records.
    fn new(out: Box<dyn Write>, output_type: DefenderHistoryOutputType) -> io::Result<Self> {
        Ok(match output_type {
            DefenderHistoryOutputType::Csv => {
                let mut csv = csv::Writer::from_writer(out);
                csv.write_record(crate::output_schema::labels(CSV_HEADER.iter().copied())?)?;
                Records::Csv(Box::new(csv))
            }
            DefenderHistoryOutputType::Json => Records::Json(out, true),
            DefenderHistoryOutputType::Jsonl => Records::Json(out, false),
        })
    }

    /// A detection's records: one per resource, or one for the detection when it has none.
    fn write(&mut self, source: &Source, detection: &Detection) -> io::Result<()> {
        let rows: Vec<Row<'_>> = if detection.resources.is_empty() {
            vec![Row {
                source,
                detection,
                resource: None,
            }]
        } else {
            detection
                .resources
                .iter()
                .enumerate()
                .map(|resource| Row {
                    source,
                    detection,
                    resource: Some(resource),
                })
                .collect()
        };
        for row in &rows {
            match self {
                Records::Csv(csv) => csv.write_record(csv_row(row))?,
                Records::Json(out, pretty) => {
                    crate::output_schema::write_json(&mut *out, &json_record(row), *pretty)?;
                    out.write_all(b"\n")?;
                }
            }
        }
        Ok(())
    }

    fn finish(self) -> io::Result<()> {
        match self {
            Records::Csv(mut csv) => csv.flush(),
            Records::Json(mut out, _) => out.flush(),
        }
    }
}

// --- summary --------------------------------------------------------------------------------

/// What the summary counts.
#[derive(Default)]
struct Stats {
    detections: usize,
    not_history: usize,
    unreadable: usize,
    resources: BTreeMap<String, usize>,
    statuses: BTreeMap<String, usize>,
    threats: BTreeMap<String, usize>,
    /// Detections with no InitialDetectionTime, LastThreatStatusChangeTime or RemediationTime.
    untimed: usize,
    with_problems: usize,
}

impl Stats {
    fn count(&mut self, d: &Detection) {
        self.detections += 1;
        for resource in &d.resources {
            *self.resources.entry(resource.kind.clone()).or_default() += 1;
        }
        let status = d
            .threat_status_name()
            .map(str::to_owned)
            .or_else(|| d.threat_status_id.map(|n| format!("{n} (undocumented)")))
            .unwrap_or_else(|| "(none)".into());
        *self.statuses.entry(status).or_default() += 1;
        let threat = d.threat_name.clone().unwrap_or_else(|| "(none)".into());
        *self.threats.entry(threat).or_default() += 1;
        if d.initial_detection.is_none() && d.status_changed.is_none() && d.remediation.is_none() {
            self.untimed += 1;
        }
        if !d.problems.is_empty() {
            self.with_problems += 1;
        }
    }
}

fn counts(map: &BTreeMap<String, usize>) -> String {
    map.iter()
        .map(|(k, n)| format!("{k}: {}", fmt_num(*n)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn print_summary(stats: &Stats, no_color: bool) {
    p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
    p_label(no_color, "Detections parsed: ", &fmt_num(stats.detections));
    if stats.not_history > 0 {
        p_label(
            no_color,
            "Not detection history files: ",
            &fmt_num(stats.not_history),
        );
    }
    if stats.unreadable > 0 {
        p_label(no_color, "Could not read: ", &fmt_num(stats.unreadable));
    }
    let total: usize = stats.resources.values().sum();
    p_label(
        no_color,
        "Resources: ",
        &if total == 0 {
            "0".to_owned()
        } else {
            format!("{} ({})", fmt_num(total), counts(&stats.resources))
        },
    );
    if !stats.statuses.is_empty() {
        p_label(no_color, "Threat statuses: ", &counts(&stats.statuses));
    }
    if stats.untimed > 0 {
        p_label(
            no_color,
            "Detections without a detection, status or remediation time: ",
            &fmt_num(stats.untimed),
        );
    }
    if stats.with_problems > 0 {
        p_label(
            no_color,
            "Detections with problems: ",
            &fmt_num(stats.with_problems),
        );
    }
    if !stats.threats.is_empty() {
        p(None, "", true);
        p(JunteiColor::Orange.rdg(no_color), "Threats detected:", true);
        let mut list: Vec<(&str, usize)> = stats
            .threats
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();
        list.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        for (name, n) in list.iter().take(MAX_PRINTED_THREATS) {
            p(None, &format!("  {name} ({})", fmt_num(*n)), true);
        }
        if list.len() > MAX_PRINTED_THREATS {
            p(
                None,
                &format!("  … and {} more", fmt_num(list.len() - MAX_PRINTED_THREATS)),
                true,
            );
        }
    }
    p(None, "", true);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_paths_need_a_detection_history_folder() {
        for path in [
            "ProgramData/Microsoft/Windows Defender/Scans/History/Service/DetectionHistory/02/6AFE33A0-19BA-4FFF-892F-B700539D7D63",
            "C/ProgramData/Microsoft/Windows Defender/Scans/History/Service/detectionhistory/x",
            "DetectionHistory/a",
            r"C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\02\x",
        ] {
            assert!(is_history_path(Path::new(path)), "{path}");
        }
        for path in [
            "ProgramData/Microsoft/Windows Defender/Quarantine/Entries/x",
            "DetectionHistory",
            "Service/DetectionHistory.bak/x",
            "",
        ] {
            assert!(!is_history_path(Path::new(path)), "{path}");
        }
    }

    #[test]
    fn staged_directory_indexes_are_not_detection_history_files() {
        let dir = tempfile::tempdir().unwrap();
        let history = dir.path().join("DetectionHistory");
        let numbered = history.join("02");
        std::fs::create_dir_all(&numbered).unwrap();
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_files/defender_history/museum/DetectionHistory/02/0F3D5CDF-677A-4AF9-BDE9-54E0ACD9900D");
        let id = numbered.join("0F3D5CDF-677A-4AF9-BDE9-54E0ACD9900D");
        std::fs::copy(fixture, &id).unwrap();
        // image staging writes a directory's index as `$I30` into it
        std::fs::write(history.join("$I30"), b"INDX").unwrap();
        std::fs::write(numbered.join("$i30"), b"INDX").unwrap();
        let found: Vec<PathBuf> = find_histories(dir.path())
            .into_iter()
            .map(|s| s.path)
            .collect();
        assert_eq!(found, [id]);
    }

    #[test]
    fn csv_and_json_name_the_same_fields_in_the_same_order() {
        let detection = Detection {
            id: Some("X".into()),
            resources: vec![Resource {
                kind: "file".into(),
                location: r"C:\a.exe".into(),
                flags: Some(1),
                tracking: vec![
                    ("ThreatTrackingSha256".into(), Value::Text("ab".into())),
                    ("ThreatTrackingScanType".into(), Value::Integer(4)),
                ],
            }],
            ..Detection::default()
        };
        let source = Source {
            path: PathBuf::from("X"),
            shown: "X".into(),
        };
        let row = Row {
            source: &source,
            detection: &detection,
            resource: Some((0, &detection.resources[0])),
        };
        let mut json = crate::output_schema::to_value(&json_record(&row)).unwrap();
        let object = json.as_object_mut().unwrap();
        // JSON only: the typed values the CSV joins into text.
        for key in ["Tracking", "CountedList"] {
            assert!(object.remove(key).is_some(), "{key}");
        }
        let keys: Vec<&str> = object.keys().map(String::as_str).collect();
        let mut csv: Vec<&str> = crate::output_schema::labels(CSV_HEADER.iter().copied())
            .unwrap()
            .into_iter()
            .filter(|k| *k != "OtherTracking")
            .collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        csv.sort_unstable();
        assert_eq!(sorted, csv);
        let cells = csv_row(&row);
        assert_eq!(cells.len(), CSV_HEADER.len());
        assert_eq!(
            cells[CSV_HEADER.iter().position(|h| *h == "SHA256").unwrap()],
            "ab"
        );
        assert_eq!(
            cells[CSV_HEADER
                .iter()
                .position(|h| *h == "OtherTracking")
                .unwrap()],
            "ThreatTrackingScanType=4"
        );
        assert_eq!(object["SHA256"], "ab");
        assert_eq!(object["ResourceFlags"], 1);
    }

    #[test]
    fn typed_values_keep_their_types_and_raw_filetimes() {
        assert_eq!(
            typed(&Value::Time(132_914_198_084_356_762)),
            json!({"Type": "FILETIME", "Value": 132_914_198_084_356_762_u64, "Time": "2022-03-10T21:03:28.4356762Z"})
        );
        // A FILETIME that is no time keeps its ticks, with no time.
        assert_eq!(
            typed(&Value::Time(u64::MAX)),
            json!({"Type": "FILETIME", "Value": u64::MAX, "Time": null})
        );
        assert_eq!(
            typed(&Value::Bytes(vec![0, 0xab])),
            json!({"Type": "Bytes", "Value": "00ab"})
        );
        assert_eq!(
            keyed(
                "Position",
                "6.13",
                &Value::Text("NT AUTHORITY\\SYSTEM".into())
            ),
            json!({"Position": "6.13", "Type": "Text", "Value": "NT AUTHORITY\\SYSTEM"})
        );
    }
}
