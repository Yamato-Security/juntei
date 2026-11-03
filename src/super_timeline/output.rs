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

use super::model::{Event, SortRow};
use crate::cli::{TimelineOutputType, TimelineProfile};
use serde_json::Value;
use std::fs::File;
use std::io::{self, BufWriter, Write};

pub const HEADERS: [&str; 11] = [
    "Timestamp",
    "Computer",
    "Artifact",
    "TimestampType",
    "User",
    "Path",
    "Message",
    "SourceFile",
    "SourceField",
    "RecordID",
    "Details",
];

pub struct Writer {
    csv: Option<csv::Writer<BufWriter<File>>>,
    jsonl: Option<BufWriter<File>>,
    profile: TimelineProfile,
}

impl Writer {
    pub fn new(
        file: File,
        format: TimelineOutputType,
        profile: TimelineProfile,
    ) -> io::Result<Self> {
        let mut writer = Self {
            csv: None,
            jsonl: None,
            profile,
        };
        if format == TimelineOutputType::Csv {
            let mut csv = csv::Writer::from_writer(BufWriter::new(file));
            let mut header = HEADERS;
            if profile == TimelineProfile::Timesketch {
                header[0] = "datetime";
                header[3] = "timestamp_desc";
                header[6] = "message";
            }
            csv.write_record(header)?;
            writer.csv = Some(csv);
        } else {
            writer.jsonl = Some(BufWriter::new(file));
        }
        Ok(writer)
    }

    pub fn event(&mut self, event: &Event) -> io::Result<()> {
        if let Some(csv) = &mut self.csv {
            csv.write_record([
                event.timestamp.clone(),
                event.computer.clone(),
                event.artifact.clone(),
                event.timestamp_type.clone(),
                event.user.clone(),
                event.path.clone(),
                event.message.clone(),
                event.source_file.clone(),
                event.source_field.clone(),
                event.record_id.clone(),
                serde_json::to_string(&event.details)?,
            ])?;
        } else if let Some(out) = &mut self.jsonl {
            if self.profile == TimelineProfile::Timesketch {
                let mut value = serde_json::to_value(event)?;
                let object = value.as_object_mut().unwrap();
                for (old, new) in [
                    ("Timestamp", "datetime"),
                    ("TimestampType", "timestamp_desc"),
                    ("Message", "message"),
                ] {
                    let value = object.remove(old).unwrap();
                    object.insert(new.into(), value);
                }
                serde_json::to_writer(&mut *out, &value)?;
            } else {
                serde_json::to_writer(&mut *out, event)?;
            }
            out.write_all(b"\n")?;
        }
        Ok(())
    }

    pub fn finish(mut self) -> io::Result<()> {
        if let Some(csv) = &mut self.csv {
            csv.flush()?;
        }
        if let Some(out) = &mut self.jsonl {
            out.flush()?;
        }
        Ok(())
    }
}

/// Keep bounded alternative context when SI/ADS or Prefetch projections describe one event.
pub fn duplicate(first: &mut SortRow, other: &SortRow) {
    let count = first
        .event
        .details
        .get("DuplicateCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    first
        .event
        .details
        .insert("DuplicateCount".into(), Value::from(count + 1));
    for (key, candidate) in [
        ("AlternativePaths", &other.event.path),
        ("EquivalentSourceFields", &other.event.source_field),
    ] {
        let original = if key == "AlternativePaths" {
            &first.event.path
        } else {
            &first.event.source_field
        };
        if candidate == original || candidate.is_empty() {
            continue;
        }
        let array = first
            .event
            .details
            .entry(key)
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .unwrap();
        let candidate = Value::String(candidate.clone());
        if !array.contains(&candidate) {
            if array.len() < 128 {
                array.push(candidate);
            } else {
                first.event.details.insert(
                    "AdditionalDuplicateContextInNativeRecords".into(),
                    Value::Bool(true),
                );
            }
        }
    }
}
