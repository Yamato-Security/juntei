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

use chrono::{
    DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, SecondsFormat, TimeZone, Utc,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct Point {
    pub seconds: i64,
    pub nanos: u32,
}

#[derive(Clone, Debug)]
pub struct ParsedTime {
    pub point: Point,
    pub text: String,
    pub precision: String,
    pub basis: String,
}

/// Numeric counters are deliberately not interpreted as arbitrary epoch/FILETIME values.
pub fn parse_time(
    raw: &str,
    known_utc: bool,
    assumed: Option<FixedOffset>,
) -> Result<ParsedTime, String> {
    let raw = raw.trim();
    let fractions = raw
        .split_once('.')
        .map(|(_, tail)| tail.bytes().take_while(u8::is_ascii_digit).count())
        .unwrap_or(0);
    if fractions > 9 {
        return Err("timestamp precision exceeds nanoseconds".into());
    }
    let mut basis = "Explicit UTC offset".to_string();
    let mut day = false;
    let time = if let Ok(value) = DateTime::parse_from_rfc3339(raw) {
        value.with_timezone(&Utc)
    } else if let Ok(value) = DateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S%.f %:z") {
        value.with_timezone(&Utc)
    } else {
        let naive = [
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%d %H:%M:%S%.f",
            "%m/%d/%Y %H:%M:%S%.f",
        ]
        .into_iter()
        .find_map(|format| NaiveDateTime::parse_from_str(raw, format).ok())
        .or_else(|| {
            ["%Y-%m-%d", "%Y%m%d", "%m/%d/%Y", "%m-%d-%Y"]
                .into_iter()
                .find_map(|format| {
                    NaiveDate::parse_from_str(raw, format)
                        .ok()
                        .and_then(|date| {
                            day = true;
                            date.and_hms_opt(0, 0, 0)
                        })
                })
        })
        .ok_or_else(|| "unrecognized or out-of-range timestamp".to_string())?;
        let offset = if known_utc && !day {
            basis = "UTC defined by artifact/field".into();
            FixedOffset::east_opt(0).unwrap()
        } else {
            let offset = assumed.ok_or_else(|| {
                "timestamp has no known time zone (use --assume-offset when justified)".to_string()
            })?;
            basis = format!(
                "Assumed offset {offset}{}",
                if day {
                    "; date-only midnight anchor"
                } else {
                    ""
                }
            );
            offset
        };
        offset
            .from_local_datetime(&naive)
            .single()
            .ok_or("ambiguous timestamp")?
            .with_timezone(&Utc)
    };
    if !(1..=9999).contains(&time.year()) || time.timestamp_subsec_nanos() >= 1_000_000_000 {
        return Err("unsupported calendar range or leap-second representation".into());
    }
    Ok(ParsedTime {
        point: Point {
            seconds: time.timestamp(),
            nanos: time.timestamp_subsec_nanos(),
        },
        text: time.to_rfc3339_opts(SecondsFormat::Nanos, true),
        precision: if day {
            "day".into()
        } else if fractions == 0 {
            "second".into()
        } else {
            format!("{fractions} fractional decimal digits")
        },
        basis,
    })
}

pub fn offset(value: Option<&str>) -> Option<FixedOffset> {
    let value = value?;
    let sign = if value.starts_with('-') { -1 } else { 1 };
    let hours: i32 = value.get(1..3)?.parse().ok()?;
    let minutes: i32 = value.get(4..6)?.parse().ok()?;
    FixedOffset::east_opt(sign * (hours * 3600 + minutes * 60))
}

pub fn empty_time(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "-" | "n/a" | "null" | "none" | "never" | "not set" | "0"
    )
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Event {
    pub timestamp: String,
    #[serde(rename = "Comp", alias = "Computer")]
    pub computer: String,
    pub artifact: String,
    pub timestamp_type: String,
    pub user: String,
    pub path: String,
    pub message: String,
    pub source_file: String,
    pub source_field: String,
    #[serde(rename = "RecordID")]
    pub record_id: String,
    pub details: Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SortRow {
    pub time: Point,
    pub identity: String,
    pub origin: String,
    pub event: Event,
}

impl SortRow {
    pub fn key(&self) -> (Point, &str, &str) {
        (self.time, &self.identity, &self.origin)
    }
}

pub fn fingerprint(parts: &[&str]) -> String {
    let mut h = Sha256::new();
    for part in parts {
        h.update((part.len() as u64).to_be_bytes());
        h.update(part.as_bytes());
    }
    format!("{:x}", h.finalize())
}

#[derive(Clone, Debug)]
pub struct ImageSource {
    pub image: String,
    pub staged_root: String,
}

#[derive(Clone, Debug)]
pub struct Context {
    pub parsed_file: PathBuf,
    pub artifact: String,
    pub computer: String,
    pub raw_source: Option<String>,
    pub image: Option<ImageSource>,
    pub input_index: usize,
    pub record: u64,
    pub location: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Diagnostic {
    pub parsed_file: String,
    pub input_record: u64,
    pub location: String,
    #[serde(rename = "Comp", alias = "Computer")]
    pub computer: String,
    pub artifact: String,
    pub source_file: String,
    pub reason: String,
    pub source_field: String,
    pub original_value: String,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Counts {
    pub input_records: u64,
    pub emitted_events: u64,
    pub filtered_events: u64,
    pub untimed_records: u64,
    pub invalid_timestamps: u64,
    pub unsupported_records: u64,
    pub interpreted_times_omitted: u64,
    pub duplicate_events: u64,
}

impl Counts {
    pub fn add(&mut self, other: &Self) {
        self.input_records += other.input_records;
        self.emitted_events += other.emitted_events;
        self.filtered_events += other.filtered_events;
        self.untimed_records += other.untimed_records;
        self.invalid_timestamps += other.invalid_timestamps;
        self.unsupported_records += other.unsupported_records;
        self.interpreted_times_omitted += other.interpreted_times_omitted;
        self.duplicate_events += other.duplicate_events;
    }
}

pub type ArtifactCounts = BTreeMap<String, Counts>;

/// Keep exact source spelling in SourceField; accept the UTC suffix used by CSV exports.
pub fn field<'a>(value: &'a Value, name: &str) -> Option<(&'a str, &'a Value)> {
    let object = value.as_object()?;
    object
        .get_key_value(crate::output_schema::field_name(name))
        .or_else(|| object.get_key_value(name))
        .or_else(|| {
            // UAL's column normalizer preserves acronyms and separates the next word. Keep
            // accepting the database's original spelling without allocating on every lookup.
            let current = match name {
                "RoleGuid" => "RoleGUID",
                "TenantId" => "TenantID",
                "OSLastBootUpTime" => "OS-LastBootUpTime",
                "OSVersion" => "OS-Version",
                "OSProductType" => "OS-ProductType",
                "VmGuid" => "VmGUID",
                "BiosGuid" => "BIOS-GUID",
                _ => return None,
            };
            object.get_key_value(current)
        })
        .or_else(|| {
            name.strip_suffix("UTC").and_then(|key| {
                object
                    .get_key_value(crate::output_schema::field_name(key))
                    .or_else(|| object.get_key_value(key))
            })
        })
        .map(|(key, value)| (key.as_str(), value))
}

pub fn get<'a>(value: &'a Value, names: &[&str]) -> Option<&'a Value> {
    names.iter().find_map(|name| {
        field(value, name)
            .map(|(_, value)| value)
            .filter(|v| !v.is_null())
    })
}

pub fn text(value: &Value, names: &[&str]) -> String {
    names
        .iter()
        .filter_map(|name| field(value, name).map(|(_, value)| string(value)))
        .find(|value| !value.is_empty())
        .unwrap_or_default()
}

pub fn string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

pub fn truth(value: &Value, names: &[&str]) -> bool {
    matches!(
        text(value, names).to_ascii_lowercase().as_str(),
        "true" | "1" | "yes"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamps_preserve_precision_offsets_and_pre_epoch_order() {
        let a = parse_time("2026-10-20T08:46:38.200166800Z", false, None).unwrap();
        let b = parse_time("2026-10-20T17:46:38.2001668+09:00", false, None).unwrap();
        assert_eq!(a.point, b.point);
        assert_eq!(a.text, "2026-10-20T08:46:38.200166800Z");
        assert!(
            parse_time("1969-12-31T23:59:59.999999900Z", false, None)
                .unwrap()
                .point
                < parse_time("1970-01-01T00:00:00Z", false, None)
                    .unwrap()
                    .point
        );
        assert!(parse_time("9999-12-31T23:59:59.999999900Z", false, None).is_ok());
        assert!(parse_time("2026-02-30T00:00:00Z", false, None).is_err());
        assert!(parse_time("2026-10-20T08:46:38.2001668001Z", false, None).is_err());
    }

    #[test]
    fn naive_times_and_dates_require_a_documented_basis() {
        assert!(parse_time("2026-10-20 12:00:00", false, None).is_err());
        let time = parse_time("2026-10-20 12:00:00", false, offset(Some("+09:00"))).unwrap();
        assert_eq!(time.text, "2026-10-20T03:00:00.000000000Z");
        assert!(time.basis.starts_with("Assumed"));
        assert!(parse_time("20261020", true, None).is_err());
        assert_eq!(
            parse_time("20261020", false, offset(Some("+00:00")))
                .unwrap()
                .precision,
            "day"
        );
        assert!(
            parse_time("10/20/2026 12:09:16", true, None)
                .unwrap()
                .basis
                .contains("artifact")
        );
        // A valid epoch-valued string is evidence; do not blacklist dates.
        assert!(parse_time("1601-01-01T00:00:00Z", false, None).is_ok());
    }

    #[test]
    fn identity_components_cannot_collide_on_delimiters() {
        assert_ne!(fingerprint(&["a|b", "c"]), fingerprint(&["a", "b|c"]));
    }
}
