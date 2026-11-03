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

use super::model::*;
use crate::cli::SuperTimelineOptions;
use serde_json::{Map, Value, json};
use std::io;

pub struct Policy {
    pub assumed: Option<chrono::FixedOffset>,
    pub start: Option<Point>,
    pub end: Option<Point>,
    pub collection: bool,
    pub interpreted: bool,
}

impl Policy {
    pub fn new(opts: &SuperTimelineOptions) -> Result<Self, String> {
        let start = opts
            .timeline_start
            .as_deref()
            .map(|v| parse_time(v, false, None).map(|t| t.point))
            .transpose()?;
        let end = opts
            .timeline_end
            .as_deref()
            .map(|v| parse_time(v, false, None).map(|t| t.point))
            .transpose()?;
        if start.zip(end).is_some_and(|(start, end)| start > end) {
            return Err("--timeline-start is later than --timeline-end".into());
        }
        Ok(Self {
            assumed: offset(opts.assume_offset.as_deref()),
            start,
            end,
            collection: opts.include_collection_times,
            interpreted: opts.include_interpreted_times,
        })
    }
}

pub enum Item {
    Event(SortRow),
    Diagnostic(Diagnostic),
}

pub struct Projection {
    pub events: Vec<SortRow>,
    pub diagnostics: Vec<Diagnostic>,
    pub counts: Counts,
}

pub const ARTIFACTS: &[&str] = &[
    "amcache",
    "app-history",
    "appcompat",
    "auditpol",
    "bam",
    "bcd",
    "boot",
    "defender-quarantine",
    "featureusage",
    "hayabusa",
    "i30",
    "installed-programs",
    "jump-lists",
    "lnk",
    "logfile",
    "lsa-secrets",
    "mft",
    "mru",
    "muicache",
    "networks",
    "office",
    "opensavemru",
    "persistence",
    "persistence-hunter",
    "prefetch",
    "prefetch-timeline",
    "recentdocs",
    "registry",
    "remote-access",
    "sam",
    "secure-sds",
    "security-settings",
    "services",
    "shellbags",
    "shimcache",
    "srum",
    "system-info",
    "taskcache",
    "usb",
    "userassist",
    "usn-journal",
];

pub fn source(value: &Value, ctx: &Context) -> (String, bool) {
    let mut path = if ctx.artifact == "hayabusa" {
        text(value, &["EvtxFile"])
    } else {
        text(value, &["SourceFile", "SourcePath", "EntryFile"])
    };
    if path.is_empty() && ctx.artifact == "amcache" {
        path = text(value, &["Source"]);
    }
    if path.is_empty() {
        path = ctx.raw_source.clone().unwrap_or_default();
    }
    if path.is_empty() {
        (ctx.parsed_file.to_string_lossy().into_owned(), false)
    } else {
        (path, true)
    }
}

fn join(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.into()
    } else if name.is_empty() {
        parent.into()
    } else {
        format!("{}\\{name}", parent.trim_end_matches(['\\', '/']))
    }
}

fn compact(value: &Value, depth: usize) -> Value {
    if depth > 3 {
        return json!({"ProjectionOmitted":true});
    }
    match value {
        Value::String(s) if s.chars().count() > 1024 => {
            json!({"Prefix":s.chars().take(1024).collect::<String>(),"OriginalBytes":s.len(),"ProjectionOmitted":true})
        }
        Value::Array(a) if a.len() > 16 => {
            json!({"Prefix":a.iter().take(16).map(|v| compact(v,depth+1)).collect::<Vec<_>>(),"OriginalItems":a.len(),"ProjectionOmitted":true})
        }
        Value::Array(a) => Value::Array(a.iter().map(|v| compact(v, depth + 1)).collect()),
        Value::Object(m) => {
            let mut result: Map<String, Value> = m
                .iter()
                .take(32)
                .map(|(k, v)| (k.clone(), compact(v, depth + 1)))
                .collect();
            if m.len() > 32 {
                result.insert("ProjectionOmitted".into(), Value::Bool(true));
            }
            Value::Object(result)
        }
        _ => value.clone(),
    }
}

struct Builder<'a, 'b> {
    value: &'a Value,
    ctx: &'a Context,
    policy: &'a Policy,
    source: String,
    raw: bool,
    computer: String,
    artifact: String,
    user: String,
    path: String,
    id: String,
    valid: usize,
    projection: Projection,
    sink: Option<&'b mut dyn FnMut(Item) -> io::Result<()>>,
    failure: Option<io::Error>,
}

impl<'a, 'b> Builder<'a, 'b> {
    fn new(
        value: &'a Value,
        ctx: &'a Context,
        policy: &'a Policy,
        sink: Option<&'b mut dyn FnMut(Item) -> io::Result<()>>,
    ) -> Self {
        let (source, raw) = source(value, ctx);
        let artifact = if ctx.artifact == "prefetch-timeline" {
            "prefetch".into()
        } else {
            ctx.artifact.clone()
        };
        let path = text(
            value,
            &[
                "FullPath",
                "Path",
                "ExecutablePath",
                "Program",
                "TargetPath",
                "DevicePath",
                "Name",
                "FileName",
                "Filename",
                "KeyPath",
                "TaskName",
                "ServiceName",
            ],
        );
        let id = [
            "ID",
            "RecordID",
            "EntryNum",
            "EntryNumber",
            "LSN",
            "USN",
            "ResourceID",
            "KeyName",
            "KeyPath",
            "ValueName",
            "RID",
            "Secret",
            "TaskGUID",
            "ServiceName",
            "ControlSet",
            "RegistryPath",
            "SID",
            "NodeSlot",
            "GUID",
            "Application",
            "Name",
            "Path",
            "Program",
        ]
        .into_iter()
        .filter_map(|k| field(value, k).map(|(_, v)| (k, string(v))))
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(";");
        Self {
            value,
            ctx,
            policy,
            source,
            raw,
            artifact,
            computer: ctx.computer.clone(),
            user: text(value, &["User", "Username", "UserName", "UserSID", "SID"]),
            path,
            id: if id.is_empty() {
                format!("Record:{}:{}", ctx.record, ctx.location)
            } else {
                id
            },
            valid: 0,
            sink,
            failure: None,
            projection: Projection {
                events: Vec::new(),
                diagnostics: Vec::new(),
                counts: Counts {
                    input_records: 1,
                    ..Default::default()
                },
            },
        }
    }

    fn diagnostic(&mut self, reason: &str, source_field: &str, original: &str) {
        let diagnostic = Diagnostic {
            parsed_file: self.ctx.parsed_file.to_string_lossy().into_owned(),
            input_record: self.ctx.record,
            location: self.ctx.location.clone(),
            computer: self.computer.clone(),
            artifact: self.artifact.clone(),
            source_file: self.source.clone(),
            reason: reason.into(),
            source_field: source_field.into(),
            original_value: original.chars().take(256).collect(),
        };
        self.deliver(Item::Diagnostic(diagnostic));
    }

    fn deliver(&mut self, item: Item) {
        if self.failure.is_some() {
            return;
        }
        if let Some(sink) = &mut self.sink {
            if let Err(error) = sink(item) {
                self.failure = Some(error);
            }
        } else {
            match item {
                Item::Event(event) => self.projection.events.push(event),
                Item::Diagnostic(diagnostic) => self.projection.diagnostics.push(diagnostic),
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit(
        &mut self,
        value: &Value,
        source_field: &str,
        label: &str,
        role: &str,
        path: &str,
        id: &str,
        utc: bool,
    ) {
        if self.failure.is_some() {
            return;
        }
        let (value, source_field) = if value.is_object() {
            if let Some(v) = get(value, &["UTC", "Utc"]) {
                (v, format!("{source_field}.UTC"))
            } else {
                return;
            }
        } else {
            (value, source_field.to_string())
        };
        let raw = string(value);
        if empty_time(&raw) {
            return;
        }
        // Only this mapped native field is a numeric FILETIME. Never guess units for counters.
        let parsed = if role == "recovered_file_metadata" && value.is_u64() {
            let ticks = value.as_u64().unwrap();
            let seconds = (ticks / 10_000_000) as i64 - 11_644_473_600;
            let nanos = ((ticks % 10_000_000) * 100) as u32;
            chrono::DateTime::from_timestamp(seconds, nanos)
                .ok_or_else(|| "FILETIME is outside the supported calendar range".to_string())
                .and_then(|date| {
                    parse_time(
                        &date.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
                        true,
                        None,
                    )
                })
                .map(|mut time| {
                    time.precision = "7 fractional decimal digits (FILETIME)".into();
                    time.basis = "UTC defined by FILETIME field".into();
                    time
                })
        } else {
            parse_time(&raw, utc, self.policy.assumed)
        };
        let time = match parsed {
            Ok(time) => time,
            Err(error) => {
                self.projection.counts.invalid_timestamps += 1;
                self.diagnostic(&error, &source_field, &raw);
                return;
            }
        };
        self.valid += 1;
        if self.policy.start.is_some_and(|start| time.point < start)
            || self.policy.end.is_some_and(|end| time.point > end)
        {
            self.projection.counts.filtered_events += 1;
            return;
        }
        let mut details = Map::new();
        for key in [
            "RecordType",
            "Header",
            "Record",
            "RecordStatus",
            "AllocationStatus",
            "RecoveredFrom",
            "FromSlack",
            "InUse",
            "ValidFixup",
            "IsCurrent",
            "KeyPath",
            "ValueName",
            "RegistryPath",
            "Provider",
            "Table",
            "ID",
            "EntryNum",
            "EntryNumber",
            "SequenceNumber",
            "NameType",
            "FileAttributes",
            "SI-Flags",
            "UpdateReasons",
            "Reasons",
            "USN",
            "UpdateSequenceNumber",
            "LSN",
            "RedoOperation",
            "UndoOperation",
            "MetadataSide",
            "Truncated",
            "BytesSent",
            "BytesRecvd",
            "ConnectedTime",
            "DurationMS",
            "SpanMS",
            "RunCount",
            "ExecutableName",
            "ExecutablePath",
            "ThreatName",
            "SHA1",
            "SHA256",
            "RuleID",
            "RuleTitle",
            "EventID",
            "Channel",
            "Level",
            "Details",
            "ExtraFieldInfo",
            "ChangeSource",
            "ChangeEvidence",
            "Technique",
            "MitreID",
        ] {
            if let Some(v) = get(self.value, &[key]) {
                details.insert(key.into(), compact(v, 0));
            }
        }
        details.insert("TimeRole".into(), json!(role));
        if role == "recovered_file_metadata" {
            details.insert("FromSlack".into(), json!(true));
            details.insert("RecoveredFrom".into(), json!("MFT FILE record slack"));
        }
        details.insert("OriginalTimestamp".into(), json!(raw));
        details.insert("TimePrecision".into(), json!(time.precision));
        details.insert("TimeBasis".into(), json!(time.basis));
        details.insert("ParsedFile".into(), json!(self.ctx.parsed_file));
        details.insert("InputRecord".into(), json!(self.ctx.record));
        details.insert("Location".into(), json!(self.ctx.location));
        details.insert("RawSourceAvailable".into(), json!(self.raw));
        let id = if self.raw {
            id.to_string()
        } else {
            format!(
                "{id};UnresolvedSourceRecord={}:{}",
                self.ctx.record, self.ctx.location
            )
        };
        let state = ["AllocationStatus", "RecordStatus", "FromSlack", "Truncated"]
            .into_iter()
            .map(|key| text(self.value, &[key]).to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join("|");
        let recovery_record = if state.contains("deleted") || truth(self.value, &["FromSlack"]) {
            format!("{}:{}", self.ctx.record, self.ctx.location)
        } else {
            String::new()
        };
        let identity = fingerprint(&[
            &self.computer,
            &self.artifact,
            &self.source,
            &id,
            label,
            &time.text,
            &state,
            &recovery_record,
        ]);
        let title = if self.artifact == "hayabusa" {
            let rule = text(self.value, &["RuleTitle"]);
            if rule.is_empty() {
                label.to_string()
            } else {
                rule.chars().take(1024).collect()
            }
        } else {
            label.to_string()
        };
        let message = format!("{title}: {}", path.chars().take(1024).collect::<String>());
        let row = SortRow {
            time: time.point,
            identity,
            origin: format!(
                "{:012}:{:020}:{}:{source_field}",
                self.ctx.input_index, self.ctx.record, self.ctx.location
            ),
            event: Event {
                timestamp: time.text,
                computer: self.computer.clone(),
                artifact: self.artifact.clone(),
                timestamp_type: label.into(),
                user: self.user.clone(),
                path: path.into(),
                message,
                source_file: self.source.clone(),
                source_field,
                record_id: id,
                details,
            },
        };
        self.deliver(Item::Event(row));
        self.projection.counts.emitted_events += 1;
    }

    fn named(&mut self, keys: &[&str], label: &str, role: &str, utc: bool) {
        let found = keys.iter().find_map(|key| field(self.value, key));
        if let Some((key, value)) = found {
            self.emit(
                value,
                key,
                label,
                role,
                &self.path.clone(),
                &self.id.clone(),
                utc,
            );
        }
    }

    fn finish(mut self) -> (Projection, Option<io::Error>) {
        if self.valid == 0 {
            self.projection.counts.untimed_records += 1;
            self.diagnostic(
                "No usable mapped event timestamp; native record retained in parsed input",
                "",
                "",
            );
        }
        (self.projection, self.failure)
    }
}

const MACB: [(&str, &str); 4] = [
    ("Modified", "Data modified"),
    ("Accessed", "Accessed"),
    ("MFT-Modified", "Metadata changed"),
    ("Created", "Created"),
];

fn mft(b: &mut Builder<'_, '_>) {
    if truth(b.value, &["IsADS", "IsAds"]) {
        b.diagnostic(
            "ADS row shares its base file metadata; projected from the file row",
            "",
            "",
        );
        return;
    }
    let header = get(b.value, &["Header"]).unwrap_or(b.value);
    let entry = text(header, &["RecordNumber", "EntryNum", "EntryNumber"]);
    let seq = text(header, &["Sequence", "SequenceNumber"]);
    let entry_id = format!("MFT:{entry}:{seq}");
    let path = if b.path.is_empty() || get(b.value, &["FullPath"]).is_none() {
        join(
            &text(b.value, &["ParentPath"]),
            &text(b.value, &["FileName", "Name"]),
        )
    } else {
        b.path.clone()
    };
    if let Some((attributes_field, attrs)) = field(b.value, "Attributes")
        .and_then(|(key, value)| value.as_array().map(|attrs| (key, attrs)))
    {
        for (index, attr) in attrs.iter().enumerate() {
            let h = get(attr, &["Header"]).unwrap_or(&Value::Null);
            let kind = text(h, &["TypeCode"]);
            let data = get(attr, &["Data"]).unwrap_or(&Value::Null);
            let group = match kind.as_str() {
                "StandardInformation" => "SI",
                "FileName" => "FN",
                _ => continue,
            };
            let id = if group == "SI" {
                entry_id.clone()
            } else {
                format!(
                    "{entry_id}:FN:{}:{}:{}",
                    text(data, &["Name"]),
                    text(data, &["Namespace"]),
                    text(get(data, &["Parent"]).unwrap_or(&Value::Null), &["Entry"])
                )
            };
            for (timestamp_field, label) in MACB {
                if let Some((actual, value)) = field(data, timestamp_field) {
                    b.emit(
                        value,
                        &format!("{attributes_field}[{index}].Data.{actual}"),
                        &format!("MFT {group} {label}"),
                        "file_metadata",
                        &path,
                        &id,
                        true,
                    );
                }
            }
        }
        if let Some(slack) = get(b.value, &["SlackFileNames"]).and_then(Value::as_array) {
            for (i, name) in slack.iter().enumerate() {
                let path = text(name, &["Name"]);
                for (key, label) in MACB {
                    if let Some(value) =
                        get(name, &[key]).filter(|v| v.as_u64().is_some_and(|v| v != 0))
                    {
                        b.emit(
                            value,
                            &format!("SlackFileNames[{i}].{key}"),
                            &format!("MFT recovered slack FN {label}"),
                            "recovered_file_metadata",
                            &path,
                            &format!("{entry_id}:Slack:{}:{i}", text(name, &["Offset"])),
                            true,
                        );
                    }
                }
            }
        }
        return;
    }
    for (group, suffix) in [("SI", "0x10"), ("FN", "0x30")] {
        if group == "SI" && text(b.value, &["NameType"]).eq_ignore_ascii_case("DOS") {
            continue;
        }
        let id = if group == "SI" {
            entry_id.clone()
        } else {
            format!(
                "{entry_id}:FN:{}:{}:{}",
                text(b.value, &["FileName"]),
                text(b.value, &["NameType"]),
                text(b.value, &["ParentEntryNumber"])
            )
        };
        for (old, new, label) in [
            ("LastModified", "DataModified", "Data modified"),
            ("LastAccess", "Accessed", "Accessed"),
            ("LastRecordChange", "MetadataChanged", "Metadata changed"),
            ("Created", "Created", "Created"),
        ] {
            let current = format!("{new} ({group})");
            let legacy = format!("{old}{suffix}");
            if let Some((key, value)) = field(b.value, &current).or_else(|| field(b.value, &legacy))
            {
                b.emit(
                    value,
                    key,
                    &format!("MFT {group} {label}"),
                    "file_metadata",
                    &path,
                    &id,
                    true,
                );
            }
        }
    }
}

fn prefetch(b: &mut Builder<'_, '_>) {
    b.path = text(b.value, &["ExecutablePath", "ExecutableName"]);
    let path = b.path.clone();
    if let Some(times) = get(b.value, &["LastRunTimes"]).and_then(Value::as_array) {
        for (i, time) in times.iter().enumerate() {
            b.emit(
                time,
                &format!("LastRunTimes[{i}]"),
                "Prefetch recorded execution",
                "execution",
                &path,
                "Execution",
                true,
            );
        }
    } else {
        for key in [
            "RunTime",
            "LastRun",
            "PreviousRun1",
            "PreviousRun2",
            "PreviousRun3",
            "PreviousRun4",
            "PreviousRun5",
            "PreviousRun6",
            "PreviousRun7",
        ] {
            if let Some((field, time)) = field(b.value, key) {
                b.emit(
                    time,
                    field,
                    "Prefetch recorded execution",
                    "execution",
                    &path,
                    "Execution",
                    true,
                );
            }
        }
    }
    if let Some(volumes) = get(b.value, &["Volumes"]).and_then(Value::as_array) {
        for (i, volume) in volumes.iter().enumerate() {
            if let Some(time) = get(volume, &["CreationTime"]) {
                let path = text(volume, &["DevicePath"]);
                b.emit(
                    time,
                    &format!("Volumes[{i}].CreationTime"),
                    "Prefetch referenced volume creation",
                    "file_metadata",
                    &path,
                    &format!("Volume:{path}"),
                    true,
                );
            }
        }
    } else if let Some((key, values)) = field(b.value, "VolumeCreated") {
        let times = string(values);
        let paths = text(b.value, &["VolumeDevicePaths"]);
        let paths: Vec<_> = paths.split(", ").collect();
        for (i, time) in times.split(", ").enumerate() {
            let path = paths.get(i).copied().unwrap_or("");
            b.emit(
                &json!(time),
                &format!("{key}[{i}]"),
                "Prefetch referenced volume creation",
                "file_metadata",
                path,
                &format!("Volume:{path}"),
                true,
            );
        }
    }
}

fn amcache(b: &mut Builder<'_, '_>) {
    let kind = text(b.value, &["RecordType"]);
    if matches!(kind.as_str(), "summary" | "finding") {
        return;
    }
    b.path = text(
        b.value,
        &["FullPath", "LongPathHash", "Name", "Description"],
    );
    b.id = text(b.value, &["SourceKeyPath", "KeyName", "FileID", "Name"]);
    b.named(
        &["KeyLastWrittenUTC"],
        "Amcache inventory key last write",
        "inventory_metadata",
        true,
    );
    b.named(
        &["SourceVolumeKeyLastWrittenUTC"],
        "Amcache parent volume key last write",
        "key_context",
        true,
    );
    b.named(
        &["LinkDate"],
        "Amcache PE link timestamp (metadata)",
        "pe_metadata",
        true,
    );
    for key in ["FirstInstallDate", "InstallDate", "DriverVerDate"] {
        b.named(
            &[key],
            &format!("Amcache {key} (recorded date)"),
            "inventory_metadata",
            false,
        );
    }
}

fn srum(b: &mut Builder<'_, '_>) {
    let mut provider = text(b.value, &["Provider"]);
    if provider.is_empty() {
        let stem = b
            .ctx
            .parsed_file
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        provider = stem
            .strip_prefix("SRUM-")
            .unwrap_or(&stem)
            .split("-Source")
            .next()
            .unwrap_or("")
            .into();
    }
    b.path = text(b.value, &["EXE-Info"]);
    b.id = format!("{provider}:{}", text(b.value, &["ID", "Id"]));
    b.named(
        &["Timestamp", "TimeStamp"],
        &format!("SRUM {provider} provider timestamp"),
        "usage_bucket",
        true,
    );
    b.named(
        &["ConnectStartTime"],
        "SRUM recorded connection start",
        "recorded_activity",
        true,
    );
    b.named(
        &["StartTime"],
        "SRUM recorded interval start",
        "usage_interval",
        true,
    );
    b.named(
        &["EndTime"],
        "SRUM recorded interval end",
        "usage_interval",
        true,
    );
    b.named(
        &["EXE-Timestamp"],
        "SRUM application identifier timestamp (metadata)",
        "application_metadata",
        true,
    );
}

fn link_times(
    b: &mut Builder<'_, '_>,
    link: &Value,
    prefix: &str,
    path: &str,
    id: &str,
    family: &str,
) {
    let header = get(link, &["Header"]).unwrap_or(link);
    for (key, label) in [
        ("TargetCreated", "target creation"),
        ("TargetModified", "target modification"),
        ("TargetAccessed", "target access"),
    ] {
        if let Some((field, time)) = field(header, key) {
            b.emit(
                time,
                &format!(
                    "{prefix}{}{field}",
                    if header == link { "" } else { "Header." }
                ),
                &format!("{family} stored {label}"),
                "target_metadata",
                path,
                id,
                true,
            );
        }
    }
    if let Some(tracker) = get(link, &["Tracker"]) {
        for key in ["Created", "BirthCreated"] {
            if let Some(time) = get(tracker, &[key]) {
                b.emit(
                    time,
                    &format!("{prefix}Tracker.{key}"),
                    &format!("{family} tracking identifier {key}"),
                    "identifier_metadata",
                    path,
                    id,
                    true,
                );
            }
        }
    } else if let Some(time) = get(link, &["TrackerCreatedOn"]) {
        b.emit(
            time,
            &format!("{prefix}TrackerCreatedOn"),
            &format!("{family} tracking identifier Created"),
            "identifier_metadata",
            path,
            id,
            true,
        );
    }
    if let Some(items) = get(link, &["TargetItems"]).and_then(Value::as_array) {
        for (i, item) in items.iter().enumerate() {
            for key in ["Created", "Modified", "Accessed"] {
                if let Some(time) = get(item, &[key]) {
                    b.emit(
                        time,
                        &format!("{prefix}TargetItems[{i}].{key}"),
                        &format!("{family} shell item {key} (recorded)"),
                        "shell_item_metadata",
                        path,
                        &format!("{id}:Item:{i}"),
                        false,
                    );
                }
            }
        }
    }
}

fn links(b: &mut Builder<'_, '_>) {
    let family = if b.artifact == "lnk" {
        "LNK"
    } else {
        "Jump List"
    };
    if b.artifact == "jump-lists" && get(b.value, &["Entries"]).is_some() {
        if let Some(entries) = get(b.value, &["Entries"]).and_then(Value::as_array) {
            for (i, entry) in entries.iter().enumerate() {
                let id = format!("Entry:{}", text(entry, &["EntryNumber"]));
                let path = text(entry, &["Path", "DisplayPath"]);
                for (key, label, role) in [
                    (
                        "LastUsed",
                        "Jump List entry last use (recorded)",
                        "interaction_metadata",
                    ),
                    (
                        "Created",
                        "Jump List tracking identifier creation",
                        "identifier_metadata",
                    ),
                ] {
                    if let Some(time) = get(entry, &[key]) {
                        b.emit(
                            time,
                            &format!("Entries[{i}].{key}"),
                            label,
                            role,
                            &path,
                            &id,
                            true,
                        );
                    }
                }
                if let Some(link) = get(entry, &["Link"]) {
                    link_times(b, link, &format!("Entries[{i}].Link."), &path, &id, family);
                }
            }
        }
        if let Some(entries) = get(b.value, &["Unlisted"]).and_then(Value::as_array) {
            for (i, entry) in entries.iter().enumerate() {
                if let Some(link) = get(entry, &["Link"]) {
                    let path = text(link, &["TargetPath", "RelativePath"]);
                    link_times(
                        b,
                        link,
                        &format!("Unlisted[{i}].Link."),
                        &path,
                        &format!("Unlisted:{}", text(entry, &["Stream"])),
                        family,
                    );
                }
            }
        }
    } else if b.artifact == "jump-lists" && get(b.value, &["Categories"]).is_some() {
        if let Some(categories) = get(b.value, &["Categories"]).and_then(Value::as_array) {
            for (i, category) in categories.iter().enumerate() {
                if let Some(links) = get(category, &["Links"]).and_then(Value::as_array) {
                    for (j, link) in links.iter().enumerate() {
                        let path = text(link, &["TargetPath", "RelativePath"]);
                        link_times(
                            b,
                            link,
                            &format!("Categories[{i}].Links[{j}]."),
                            &path,
                            &format!("Category:{i}:Link:{j}"),
                            family,
                        );
                    }
                }
            }
        }
        if let Some(leftover) = get(b.value, &["Leftover"]).and_then(Value::as_array) {
            for (i, entry) in leftover.iter().enumerate() {
                if let Some(link) = get(entry, &["Link"]) {
                    let path = text(link, &["TargetPath", "RelativePath"]);
                    link_times(
                        b,
                        link,
                        &format!("Leftover[{i}].Link."),
                        &path,
                        &format!("Leftover:{i}"),
                        family,
                    );
                }
            }
        }
    } else {
        let path = text(
            b.value,
            &["Path", "TargetPath", "LocalPath", "RelativePath"],
        );
        let id = if b.artifact == "lnk" {
            "Shortcut".into()
        } else {
            format!(
                "Entry:{}:{}:{}",
                text(b.value, &["EntryNumber"]),
                text(b.value, &["EntryName"]),
                path
            )
        };
        if b.artifact == "jump-lists" {
            for (key, label, role) in [
                (
                    "LastModified",
                    "Jump List entry last use (recorded)",
                    "interaction_metadata",
                ),
                (
                    "CreationTime",
                    "Jump List tracking identifier creation",
                    "identifier_metadata",
                ),
            ] {
                if let Some(time) = get(b.value, &[key]) {
                    b.emit(time, key, label, role, &path, &id, true);
                }
            }
        }
        link_times(b, b.value, "", &path, &id, family);
    }
    if b.policy.collection {
        let path = b.source.clone();
        for key in ["SourceCreated", "SourceModified", "SourceAccessed"] {
            if let Some(time) = get(b.value, &[key]) {
                b.emit(
                    time,
                    key,
                    &format!("{family} collected-container {key}"),
                    "collection_filesystem",
                    &path,
                    "Container",
                    true,
                );
            }
        }
    }
}

fn logfile(b: &mut Builder<'_, '_>) {
    b.path = text(b.value, &["TargetPath", "FileName"]);
    b.id = format!(
        "LSN:{}:Offset:{}",
        text(b.value, &["LSN"]),
        text(b.value, &["FileOffset"])
    );
    let mut expanded = false;
    for side in ["Redo", "Undo"] {
        let key = format!("{side}Times");
        if let Some(value) = get(b.value, &[&key]) {
            let decoded: Value;
            let times = if let Value::String(s) = value {
                decoded = serde_json::from_str(s).unwrap_or(Value::Null);
                &decoded
            } else {
                value
            };
            for field_name in ["Created", "Modified", "Changed", "Accessed"] {
                if let Some(time) = get(times, &[field_name]).filter(|v| !v.is_null()) {
                    b.emit(
                        time,
                        &format!("{key}.{field_name}"),
                        &format!("LogFile {side} stored {field_name}"),
                        "journal_metadata_snapshot",
                        &b.path.clone(),
                        &b.id.clone(),
                        true,
                    );
                    expanded = true;
                }
            }
        }
        let key = format!("{side}FileNames");
        if let Some(value) = get(b.value, &[&key]) {
            let decoded: Value;
            let names = if let Value::String(s) = value {
                decoded = serde_json::from_str(s).unwrap_or(Value::Null);
                &decoded
            } else {
                value
            };
            if let Some(names) = names.as_array() {
                for (i, name) in names.iter().enumerate() {
                    let path = text(name, &["Name", "FileName"]);
                    for key_name in [
                        "Created",
                        "Modified",
                        "Changed",
                        "Accessed",
                        "CreatedOn",
                        "ContentModifiedOn",
                        "RecordModifiedOn",
                        "LastAccessedOn",
                    ] {
                        if let Some(time) = get(name, &[key_name]) {
                            b.emit(
                                time,
                                &format!("{key}[{i}].{key_name}"),
                                &format!("LogFile {side} filename stored {key_name}"),
                                "journal_metadata_snapshot",
                                &path,
                                &format!("{}:Name:{i}", b.id),
                                true,
                            );
                            expanded = true;
                        }
                    }
                }
            }
        }
    }
    if !expanded {
        let side = text(b.value, &["MetadataSide"]);
        for key in [
            "CreatedOn",
            "ContentModifiedOn",
            "RecordModifiedOn",
            "LastAccessedOn",
        ] {
            b.named(
                &[key],
                &format!("LogFile {side} stored {key}"),
                "journal_metadata_snapshot",
                true,
            );
        }
    }
}

/// A curated map of exported registry times, rather than a search for date-looking strings.
const REGISTRY_TIMES: &[(&str, &str, &str)] = &[
    ("LastRunUTC", "last recorded activity", "recorded_activity"),
    (
        "FirstUsedUTC",
        "recorded first-use metadata",
        "activity_metadata",
    ),
    (
        "TypedTimeUTC",
        "stored typed-item timestamp",
        "activity_metadata",
    ),
    (
        "LastUsedUTC",
        "most-recent item time (key-derived)",
        "key_context",
    ),
    (
        "LastOpenedUTC",
        "recorded last-opened metadata",
        "interaction_metadata",
    ),
    (
        "TrustedAtUTC",
        "Office trust-record timestamp",
        "configuration_metadata",
    ),
    (
        "LastAccessedUTC",
        "recorded access metadata",
        "activity_metadata",
    ),
    (
        "ItemCreatedUTC",
        "stored shell-item creation",
        "target_metadata",
    ),
    (
        "ItemModifiedUTC",
        "stored shell-item modification",
        "target_metadata",
    ),
    (
        "ItemAccessedUTC",
        "stored shell-item access",
        "target_metadata",
    ),
    (
        "LastRegisteredUTC",
        "task registration metadata",
        "configuration_metadata",
    ),
    (
        "LastSuccessfulRunUTC",
        "task last successful run",
        "recorded_activity",
    ),
    (
        "LastLogonUTC",
        "SAM last-logon metadata",
        "account_metadata",
    ),
    (
        "PasswordLastSetUTC",
        "password last-set metadata",
        "account_metadata",
    ),
    (
        "AccountExpirationUTC",
        "configured account expiration",
        "configuration_metadata",
    ),
    (
        "LastFailedLogonUTC",
        "SAM last-failed-logon metadata",
        "account_metadata",
    ),
    (
        "CurrentSetUTC",
        "LSA current-secret set time",
        "account_metadata",
    ),
    ("OldSetUTC", "LSA old-secret set time", "account_metadata"),
    (
        "CurrentValueLastWrittenUTC",
        "LSA current-value last write",
        "key_context",
    ),
    (
        "OldValueLastWrittenUTC",
        "LSA old-value last write",
        "key_context",
    ),
    (
        "FirstConnectedUTC",
        "network profile first connection metadata",
        "network_metadata",
    ),
    (
        "LastConnectedUTC",
        "network profile last connection metadata",
        "network_metadata",
    ),
    (
        "LeaseObtainedUTC",
        "DHCP lease obtained metadata",
        "network_metadata",
    ),
    (
        "LeaseExpiresUTC",
        "configured DHCP lease expiration",
        "configuration_metadata",
    ),
    (
        "DeviceFirstInstallUTC",
        "device first-install metadata",
        "device_metadata",
    ),
    (
        "DriverLastInstalledUTC",
        "device driver-install metadata",
        "device_metadata",
    ),
    (
        "LastArrivalUTC",
        "device last-arrival metadata",
        "device_metadata",
    ),
    (
        "LastRemovalUTC",
        "device last-removal metadata",
        "device_metadata",
    ),
    (
        "LastSeenUTC",
        "application compatibility last-seen metadata",
        "inventory_metadata",
    ),
    (
        "SdbInstalledUTC",
        "compatibility database install metadata",
        "configuration_metadata",
    ),
    (
        "InstallDateUTC",
        "recorded installation date",
        "installation_metadata",
    ),
    (
        "InstallTimeUTC",
        "recorded installation time",
        "installation_metadata",
    ),
    (
        "LastShutdownUTC",
        "recorded system shutdown",
        "system_metadata",
    ),
    (
        "TimeZoneLastWrittenUTC",
        "time-zone key last write",
        "key_context",
    ),
    (
        "LogonUiLastWrittenUTC",
        "logon-UI key last write",
        "key_context",
    ),
    (
        "ComputerNameLastWrittenUTC",
        "computer-name key last write",
        "key_context",
    ),
    (
        "PrimaryDomainLastWrittenUTC",
        "primary-domain key last write",
        "key_context",
    ),
    (
        "VersionLastWrittenUTC",
        "Windows-version key last write",
        "key_context",
    ),
];

fn registry(b: &mut Builder<'_, '_>) {
    let mut keypath = text(b.value, &["KeyPath", "BagKey", "Subkey"]);
    if keypath.is_empty() {
        keypath = b.id.clone();
    }
    let record_type = text(b.value, &["RecordType"]);
    let is_value = b.artifact == "registry" && record_type != "key";
    if !is_value {
        for key in [
            "KeyLastWrittenUTC",
            "NamesKeyLastWrittenUTC",
            "TreeKeyLastWrittenUTC",
            "TaskKeyLastWrittenUTC",
            "ParentKeyLastWrittenUTC",
        ] {
            if let Some((actual, time)) = field(b.value, key) {
                let path = if key == "ParentKeyLastWrittenUTC" {
                    keypath
                        .rsplit_once('\\')
                        .map(|(p, _)| p)
                        .unwrap_or(&keypath)
                } else {
                    &keypath
                };
                b.emit(
                    time,
                    actual,
                    &format!("{} {key}", b.artifact),
                    "registry_key_last_write",
                    path,
                    &format!("Key:{path}:{key}"),
                    true,
                );
            }
        }
    }
    if b.artifact == "registry" {
        if let Some((key, time)) =
            field(b.value, "ValueFiletimeUTC").filter(|(_, v)| !empty_time(&string(v)))
        {
            if b.policy.interpreted {
                b.emit(
                    time,
                    key,
                    "Registry QWORD interpreted as FILETIME (metadata)",
                    "interpreted_value",
                    &keypath,
                    &format!("Value:{keypath}:{}", text(b.value, &["ValueName"])),
                    true,
                );
            } else {
                b.projection.counts.interpreted_times_omitted += 1;
            }
        }
        return;
    }
    for (key, label, role) in REGISTRY_TIMES {
        b.named(&[key], &format!("{} {label}", b.artifact), role, true);
    }
    if b.artifact == "usb"
        && let Some(users) = get(b.value, &["Users"]).and_then(Value::as_array)
    {
        for (i, user) in users.iter().enumerate() {
            if let Some(time) = get(user, &["MountPoints2LastWritten"]) {
                let previous_source = b.source.clone();
                let previous_user = b.user.clone();
                let previous_raw = b.raw;
                let source = text(user, &["SourcePath"]);
                if !source.is_empty() {
                    b.source = source;
                    b.raw = true;
                }
                b.user = text(user, &["Username"]);
                b.emit(
                    time,
                    &format!("Users[{i}].MountPoints2LastWritten"),
                    "USB user MountPoints2 key last write",
                    "registry_key_last_write",
                    &b.path.clone(),
                    &format!("{}:User:{}", b.id, b.user),
                    true,
                );
                b.source = previous_source;
                b.user = previous_user;
                b.raw = previous_raw;
            }
        }
    }
    if b.artifact == "system-info" {
        for array in ["Profiles", "PreviousVersions"] {
            if let Some(value) = get(b.value, &[array]) {
                let decoded: Value;
                let values = if let Value::String(s) = value {
                    decoded = serde_json::from_str(s).unwrap_or(Value::Null);
                    &decoded
                } else {
                    value
                };
                if !values.is_array() && !empty_time(&string(value)) {
                    b.diagnostic("Rendered CSV metadata does not preserve nested timestamps; use native JSON/JSONL for this field", array, "");
                }
                if let Some(values) = values.as_array() {
                    for (i, row) in values.iter().enumerate() {
                        for key in [
                            "KeyLastWritten",
                            "LoadTime",
                            "LocalLoadTime",
                            "LocalUnloadTime",
                            "InstallDate",
                            "InstallTime",
                        ] {
                            if let Some(time) = get(row, &[key]) {
                                let path = text(row, &["ProfilePath", "Path", "SID"]);
                                b.emit(
                                    time,
                                    &format!("{array}[{i}].{key}"),
                                    &format!("SystemInfo {array} {key}"),
                                    "system_metadata",
                                    &path,
                                    &format!("{array}:{i}"),
                                    true,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
pub fn project(value: &Value, ctx: &Context, policy: &Policy) -> Projection {
    build(value, ctx, policy, None).0
}

pub fn project_into(
    value: &Value,
    ctx: &Context,
    policy: &Policy,
    mut sink: impl FnMut(Item) -> io::Result<()>,
) -> io::Result<Counts> {
    let (projection, error) = build(value, ctx, policy, Some(&mut sink));
    match error {
        Some(error) => Err(error),
        None => Ok(projection.counts),
    }
}

fn build(
    value: &Value,
    ctx: &Context,
    policy: &Policy,
    sink: Option<&mut dyn FnMut(Item) -> io::Result<()>>,
) -> (Projection, Option<io::Error>) {
    let mut b = Builder::new(value, ctx, policy, sink);
    if !ARTIFACTS.contains(&ctx.artifact.as_str()) {
        b.projection.counts.unsupported_records = 1;
        b.diagnostic("Unsupported or missing Artifact identifier", "", "");
        return b.finish();
    }
    match ctx.artifact.as_str() {
        "mft" => mft(&mut b),
        "prefetch" | "prefetch-timeline" => prefetch(&mut b),
        "amcache" => amcache(&mut b),
        "srum" => srum(&mut b),
        "lnk" | "jump-lists" => links(&mut b),
        "logfile" => logfile(&mut b),
        "usn-journal" => {
            b.path = join(&text(value, &["ParentPath"]), &text(value, &["Name"]));
            b.id = format!(
                "USN:{}:Offset:{}",
                text(value, &["USN", "UpdateSequenceNumber"]),
                text(value, &["Offset", "OffsetToData"])
            );
            b.named(
                &["Timestamp", "UpdateTimestamp"],
                "USN journal record timestamp",
                "journal_record",
                true,
            );
        }
        "i30" => {
            b.path = text(value, &["Name", "FileName"]);
            b.id = format!(
                "Index:{}:Entry:{}:{}",
                text(value, &["RecordOffset", "IndexRecordOffset"]),
                text(value, &["EntryOffset", "Offset"]),
                b.id
            );
            for (fields, label) in [
                (&["CreatedUTC", "CreatedOn", "Created"][..], "creation"),
                (
                    &["ModifiedUTC", "ContentModifiedOn", "Modified"][..],
                    "data modification",
                ),
                (
                    &["MFT-ModifiedUTC", "RecordModifiedOn", "MFT-Modified"][..],
                    "metadata change",
                ),
                (&["AccessedUTC", "LastAccessedOn", "Accessed"][..], "access"),
            ] {
                b.named(
                    fields,
                    &format!("I30 stored FN {label}"),
                    "index_metadata",
                    true,
                );
            }
        }
        "shimcache" => {
            b.named(
                &["LastModifiedTimeUTC", "LastModified"],
                "ShimCache stored file modification",
                "file_metadata",
                true,
            );
            b.named(
                &["LastUpdateTimeUTC", "LastUpdate"],
                "ShimCache stored last-update metadata",
                "cache_metadata",
                true,
            );
            b.path = text(value, &["RegistryPath"]);
            b.id = format!("CacheKey:{}", b.path);
            b.named(
                &["CacheLastWrittenUTC", "CacheLastWritten"],
                "ShimCache cache key last write",
                "registry_key_last_write",
                true,
            );
        }
        "defender-quarantine" => {
            b.id = format!(
                "Entry:{}:Resource:{}:{}",
                text(value, &["EntryID"]),
                text(value, &["ResourceID"]),
                b.path
            );
            b.named(
                &["QuarantineTime"],
                "Defender quarantine-entry timestamp",
                "quarantine_metadata",
                true,
            );
            b.named(
                &["CreationTime"],
                "Defender stored resource creation",
                "file_metadata",
                true,
            );
            b.named(
                &["LastAccessTime"],
                "Defender stored resource access",
                "file_metadata",
                true,
            );
            b.named(
                &["LastWriteTime"],
                "Defender stored resource modification",
                "file_metadata",
                true,
            );
        }
        "hayabusa" => {
            let detail = get(value, &["Details"]).unwrap_or(&Value::Null);
            b.path = text(detail, &["Path", "Proc", "Image", "CommandLine"]);
            b.user = text(detail, &["User", "SrcUser", "SubjectUserName"]);
            b.id = format!(
                "Rule:{}:Record:{}",
                text(value, &["RuleID"]),
                text(value, &["RecordID"])
            );
            if text(value, &["RecordID"]).is_empty() {
                b.id.push_str(&format!(":InputRecord:{}", ctx.record));
            }
            b.named(
                &["Timestamp", "TimeStamp"],
                "Hayabusa event/alert timestamp",
                "event_log_detection",
                false,
            );
        }
        "persistence-hunter" => {
            b.named(
                &["KeyLastWrite"],
                "Persistence finding supporting key last write",
                "registry_key_last_write",
                true,
            );
            b.named(
                &["LastChange"],
                "Persistence finding correlated change time",
                "correlated_evidence",
                true,
            );
        }
        _ => registry(&mut b),
    }
    b.finish()
}
