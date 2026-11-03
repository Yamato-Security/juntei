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

//! Convert only generated, parsed timestamp fields. Raw evidence text and unknown wall clocks
//! are not interpreted. UTC output needs no rewrite, preserving the default scan's performance.
use super::{AutoOutputType, Task, json_files, replace_generated_file, unused_sibling};
use crate::time_display::OutputOffset;
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;

fn timestamp_field(key: &str) -> bool {
    key == "UTC"
        || [
            "UTC",
            "Time",
            "Times",
            "Date",
            "Dates",
            "LastWrite",
            "LastWritten",
            "Created",
            "Modified",
            "Accessed",
            "Changed",
            "Timestamp",
            "LastRun",
        ]
        .iter()
        .any(|suffix| key.ends_with(suffix))
        || matches!(
            key,
            "Created (SI)"
                | "Created (FN)"
                | "DataModified (SI)"
                | "DataModified (FN)"
                | "MetadataChanged (SI)"
                | "MetadataChanged (FN)"
                | "Accessed (SI)"
                | "Accessed (FN)"
                | "LastOpened"
                | "LastSeen"
                | "LastChange"
                | "LastShutdown"
                | "FirstUsed"
                | "LastUsed"
                | "LastRegistered"
                | "LeaseObtained"
                | "LeaseExpires"
                | "FirstConnected"
                | "LastConnected"
                | "LastArrival"
                | "LastRemoval"
                | "EXE-Timestamp"
                | "LastRunTimes"
                | "CreatedOn"
                | "ContentModifiedOn"
                | "RecordModifiedOn"
                | "LastAccessedOn"
                | "TrackerCreatedOn"
                | "PasswordLastSet"
                | "AccountExpiration"
                | "LastLogon"
                | "LastFailedLogon"
                | "LastLogonWholeSeconds"
                | "LastSuccessfulRun"
                | "DeviceFirstInstall"
                | "DriverLastInstalled"
                | "CurrentSet"
                | "OldSet"
                | "SdbInstalled"
        )
        || ["LastRun", "PreviousRun"].iter().any(|prefix| {
            key.strip_prefix(prefix)
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        })
}

fn raw_field(key: &str) -> bool {
    matches!(
        key,
        "Details"
            | "Raw"
            | "DataRaw"
            | "ValueState"
            | "ZoneIdentifier"
            | "RegistryValues"
            | "RegistryValuesJSON"
            | "ValueData"
            | "ValueDataRaw"
            // FAT shell-item clocks carry no UTC offset. Their legacy UTC labels/Z suffix
            // are rendering conventions; don't turn them into inferred absolute instants.
            | "ItemCreated"
            | "ItemCreatedUTC"
            | "ItemModified"
            | "ItemModifiedUTC"
            | "ItemAccessed"
            | "ItemAccessedUTC"
            | "TargetItems"
    )
}

/// Keep formatting limitations local to the affected value. Bound the warning to one example
/// per file, so crafted evidence cannot turn a streaming conversion into an unbounded report.
#[derive(Default)]
struct RetainedTimes {
    count: u64,
    first: Option<String>,
}

impl RetainedTimes {
    fn timestamp(&mut self, value: &str, offset: OutputOffset) -> io::Result<Option<String>> {
        match offset.timestamp(value) {
            Err(error) if error.kind() == io::ErrorKind::InvalidData => {
                self.count = self.count.saturating_add(1);
                self.first.get_or_insert_with(|| {
                    format!("{} ({error})", value.chars().take(128).collect::<String>())
                });
                Ok(None)
            }
            result => result,
        }
    }
}

/// A CSV can retain several volume-creation timestamps in one column. Convert individual
/// timestamps only within timestamp columns; a filename that looks like a date stays intact.
fn timestamp_text(
    text: &str,
    offset: OutputOffset,
    retained: &mut RetainedTimes,
) -> io::Result<String> {
    if let Some(new) = retained.timestamp(text, offset)? {
        return Ok(new);
    }
    if text.contains(", ") || text.contains('|') {
        let mut result = String::with_capacity(text.len());
        for part in text.split_inclusive([',', '|']) {
            let (value, separator) = if part.ends_with([',', '|']) {
                part.split_at(part.len() - 1)
            } else {
                (part, "")
            };
            let trimmed = value.trim();
            if let Some(new) = retained.timestamp(trimmed, offset)? {
                result.push_str(&value[..value.len() - value.trim_start().len()]);
                result.push_str(&new);
                result.push_str(&value[value.trim_end().len()..]);
            } else {
                result.push_str(value);
            }
            result.push_str(separator);
        }
        return Ok(result);
    }
    Ok(text.to_owned())
}

fn value_times(
    value: &mut Value,
    key: &str,
    offset: OutputOffset,
    retained: &mut RetainedTimes,
) -> io::Result<()> {
    if raw_field(key) {
        return Ok(());
    }
    match value {
        Value::Object(fields) => {
            for (name, child) in fields {
                value_times(child, name, offset, retained)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                value_times(child, key, offset, retained)?;
            }
        }
        Value::String(text) if timestamp_field(key) => {
            *text = timestamp_text(text, offset, retained)?
        }
        _ => (),
    }
    Ok(())
}

fn csv_cell(
    value: &str,
    key: &str,
    artifact: &str,
    offset: OutputOffset,
    retained: &mut RetainedTimes,
) -> io::Result<String> {
    if raw_field(key) {
        return Ok(value.to_owned());
    }
    // Parsed metadata rendered as JSON remains structured. Raw registry-value dictionaries
    // nested inside it are excluded by value_times.
    if (key.ends_with("JSON")
        || matches!(
            key,
            "RedoTimes"
                | "UndoTimes"
                | "RedoFileNames"
                | "UndoFileNames"
                | "RedoDecoded"
                | "UndoDecoded"
                | "ReconstructedFile"
                | "ReconstructedIndex"
        ))
        && value.starts_with(['{', '['])
    {
        let mut json: Value = serde_json::from_str(value)?;
        value_times(&mut json, "", offset, retained)?;
        return serde_json::to_string(&json).map_err(io::Error::other);
    }
    if timestamp_field(key) {
        return timestamp_text(value, offset, retained);
    }
    // USB's CSV renders a parsed user's MountPoints2 key time after the name in parentheses.
    if artifact == "usb" && key == "User" {
        return value
            .split('|')
            .map(|user| {
                if let Some((name, at)) = user.rsplit_once(" (")
                    && let Some(at) = at.strip_suffix(')')
                    && let Some(new) = retained.timestamp(at, offset)?
                {
                    return Ok(format!("{name} ({new})"));
                }
                Ok(user.to_owned())
            })
            .collect::<io::Result<Vec<_>>>()
            .map(|users| users.join("|"));
    }
    Ok(value.to_owned())
}

pub(super) fn rewrite(task: &Task, offset: OutputOffset, output: &Path) -> io::Result<Vec<String>> {
    let mut warnings = Vec::new();
    if offset.is_utc() {
        return Ok(warnings);
    }
    for path in json_files(&task.output, task.output_type) {
        let retained = rewrite_file(&path, task.output_type, task.artifact, offset)?;
        if let Some(first) = retained.first {
            warnings.push(format!(
                "Warning: retained {} timestamp(s) unchanged in {} because timezone {offset} cannot represent them without losing data. Other timestamps were converted. First retained value: {first}",
                retained.count, super::dot_relative(output, &path).display()
            ));
        }
    }
    Ok(warnings)
}

fn rewrite_file(
    path: &Path,
    kind: AutoOutputType,
    artifact: &str,
    offset: OutputOffset,
) -> io::Result<RetainedTimes> {
    let temporary = unused_sibling(path, "timezone")?;
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let mut retained = RetainedTimes::default();
    let result = (|| {
        let input = BufReader::new(File::open(path)?);
        if kind == AutoOutputType::Csv {
            let mut input = csv::Reader::from_reader(input);
            let header = input.headers()?.clone();
            let mut writer = csv::Writer::from_writer(BufWriter::new(file));
            writer.write_record(&header)?;
            for record in input.records() {
                let record = record?;
                let row = record
                    .iter()
                    .zip(header.iter())
                    .map(|(value, key)| csv_cell(value, key, artifact, offset, &mut retained))
                    .collect::<io::Result<Vec<_>>>()?;
                writer.write_record(row)?;
            }
            writer.flush()?;
        } else {
            let mut writer = BufWriter::new(file);
            for record in serde_json::Deserializer::from_reader(input).into_iter::<Value>() {
                let mut record = record.map_err(io::Error::other)?;
                value_times(&mut record, "", offset, &mut retained)?;
                if kind == AutoOutputType::Json {
                    serde_json::to_writer_pretty(&mut writer, &record)?;
                } else {
                    serde_json::to_writer(&mut writer, &record)?;
                }
                writer.write_all(b"\n")?;
            }
            writer.flush()?;
        }
        replace_generated_file(path, &temporary)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map(|()| retained)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn shift_only_parsed_instants_preserving_precision_raw_data_and_local_clocks() {
        let offset = "+09:00".parse().unwrap();
        let mut retained = RetainedTimes::default();
        let at = "2026-10-20T08:46:38.2001668Z";
        let mut value = json!({"LastRunTimes":[at],"Attr":[{"Data":{"Created":at}}],"KeyLastWritten":at,"LastShutdown":at,"Name":at,
            "ValueData":at,"DataRaw":{"Created":at},"RegistryValues":[{"Name":"Created","Decoded":at}],
            "FirstConnectedLocal":"2026-10-20T08:46:38.200", "ItemCreated":at, "TargetItems":[{"Created":at}], "Raw":{"Timestamp":at}, "ZoneIdentifier":{"LastRun":at},
            "Nested":{"UTC":at,"Filetime":134000000000000000u64}});
        value_times(&mut value, "", offset, &mut retained).unwrap();
        assert_eq!(
            value["LastRunTimes"][0],
            "2026-10-20T17:46:38.2001668+09:00"
        );
        assert_eq!(
            value["Attr"][0]["Data"]["Created"],
            value["LastRunTimes"][0]
        );
        assert_eq!(value["KeyLastWritten"], value["LastRunTimes"][0]);
        assert_eq!(value["LastShutdown"], value["LastRunTimes"][0]);
        assert_eq!(value["ItemCreated"], at);
        assert_eq!(value["TargetItems"][0]["Created"], at);
        assert_eq!(value["Name"], at);
        assert_eq!(value["ValueData"], at);
        assert_eq!(value["DataRaw"]["Created"], at);
        assert_eq!(value["Raw"]["Timestamp"], at);
        assert_eq!(value["RegistryValues"][0]["Decoded"], at);
        assert_eq!(value["ZoneIdentifier"]["LastRun"], at);
        assert_eq!(value["FirstConnectedLocal"], "2026-10-20T08:46:38.200");
        assert_eq!(value["Nested"]["Filetime"], 134000000000000000u64);
        assert_eq!(
            csv_cell(at, "Filename", "mft", offset, &mut retained).unwrap(),
            at
        );
        assert_eq!(
            csv_cell(at, "ItemCreatedUTC", "shellbags", offset, &mut retained).unwrap(),
            at
        );
        assert_eq!(
            csv_cell(
                &format!("alice ({at})"),
                "User",
                "usb",
                offset,
                &mut retained
            )
            .unwrap(),
            "alice (2026-10-20T17:46:38.2001668+09:00)"
        );
        assert_eq!(
            timestamp_text(&format!("{at}, {at}"), offset, &mut retained).unwrap(),
            "2026-10-20T17:46:38.2001668+09:00, 2026-10-20T17:46:38.2001668+09:00"
        );
    }

    #[test]
    fn failed_conversion_never_replaces_the_original_file() {
        let root = tempfile::tempdir().unwrap();
        for (kind, original) in [
            (
                AutoOutputType::Csv,
                "Filename,Created (SI)\nx.exe,2026-10-20T08:46:38Z\ny.exe,2026-10-20T08:46:38Z,unexpected column\n",
            ),
            (
                AutoOutputType::Json,
                "{\"Created\":\"2026-10-20T08:46:38Z\"}\n{malformed",
            ),
            (
                AutoOutputType::Jsonl,
                "{\"Created\":\"2026-10-20T08:46:38Z\"}\n{malformed",
            ),
        ] {
            let path = root.path().join(format!("data.{}", kind.as_str()));
            fs::write(&path, original).unwrap();
            assert!(rewrite_file(&path, kind, "mft", "+09:00".parse().unwrap()).is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), original);
        }
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 3);
    }

    #[test]
    fn display_limits_preserve_only_affected_values_in_all_formats() {
        let root = tempfile::tempdir().unwrap();
        let at = "2026-10-20T08:46:38.2001668Z";
        let precise = "2026-10-20T08:46:38.1234567890Z";
        for (offset, boundary, expected) in [
            (
                "+09:00",
                "9999-12-31T23:00:00Z",
                "2026-10-20T17:46:38.2001668+09:00",
            ),
            (
                "-01:00",
                "0000-01-01T00:00:00Z",
                "2026-10-20T07:46:38.2001668-01:00",
            ),
        ] {
            for kind in [
                AutoOutputType::Csv,
                AutoOutputType::Json,
                AutoOutputType::Jsonl,
            ] {
                let path = root.path().join(format!("data.{}", kind.as_str()));
                if kind == AutoOutputType::Csv {
                    fs::write(
                        &path,
                        format!(
                            "Created,Modified,Accessed,Filename\n{boundary},{at},{precise},{at}\n"
                        ),
                    )
                    .unwrap();
                } else {
                    fs::write(&path, json!({"Created":boundary,"Modified":at,"Accessed":precise,"Filename":at,"Raw":{"Created":boundary}}).to_string()).unwrap();
                }
                let retained = rewrite_file(&path, kind, "mft", offset.parse().unwrap()).unwrap();
                assert_eq!(retained.count, 2);
                assert!(retained.first.unwrap().contains(boundary));
                if kind == AutoOutputType::Csv {
                    let mut reader = csv::Reader::from_path(&path).unwrap();
                    assert_eq!(
                        reader.records().next().unwrap().unwrap(),
                        vec![boundary, expected, precise, at]
                    );
                } else {
                    let after: Value = serde_json::from_reader(File::open(&path).unwrap()).unwrap();
                    assert_eq!(after["Created"], boundary);
                    assert_eq!(after["Modified"], expected);
                    assert_eq!(after["Accessed"], precise);
                    assert_eq!(after["Filename"], at);
                    assert_eq!(after["Raw"]["Created"], boundary);
                }
            }
        }
    }
}

#[cfg(test)]
mod format_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn csv_json_and_jsonl_convert_nested_parsed_times_but_preserve_raw_values() {
        let dir = tempfile::tempdir().unwrap();
        let original = "2026-10-20T08:46:38.200166800Z";
        let expected = "2026-10-20T17:46:38.200166800+09:00";
        let offset = "+09:00".parse().unwrap();
        for kind in [AutoOutputType::Json, AutoOutputType::Jsonl] {
            let path = dir.path().join(format!("data.{}", kind.as_str()));
            let value = json!({"Attr":[{"Data":{"Created":original}}], "LastRunTimes":[original],
                "PasswordLastSet":original, "ValueData":original, "Filename":original,
                "FirstConnectedLocal":"2026-10-20T08:46:38.2"});
            fs::write(&path, format!("{value}\n")).unwrap();
            rewrite_file(&path, kind, "mft", offset).unwrap();
            let after: Value = serde_json::from_reader(File::open(path).unwrap()).unwrap();
            assert_eq!(after["Attr"][0]["Data"]["Created"], expected);
            assert_eq!(after["LastRunTimes"][0], expected);
            assert_eq!(after["PasswordLastSet"], expected);
            assert_eq!(after["ValueData"], original);
            assert_eq!(after["Filename"], original);
            assert_eq!(after["FirstConnectedLocal"], "2026-10-20T08:46:38.2");
        }
        let path = dir.path().join("Prefetch.csv");
        let mut writer = csv::Writer::from_path(&path).unwrap();
        writer
            .write_record([
                "Filename",
                "LastRun",
                "PreviousRun1",
                "VolumeCreated",
                "RedoTimes",
                "RegistryValuesJSON",
            ])
            .unwrap();
        let raw = json!([{"Name":"Created", "Decoded":original}]).to_string();
        writer
            .write_record([
                original,
                original,
                original,
                &format!("{original}, {original}"),
                &json!({"Created":{"UTC":original,"Filetime":123}}).to_string(),
                &raw,
            ])
            .unwrap();
        writer.flush().unwrap();
        drop(writer);
        rewrite_file(&path, AutoOutputType::Csv, "prefetch", offset).unwrap();
        let mut reader = csv::Reader::from_path(path).unwrap();
        let row = reader.records().next().unwrap().unwrap();
        assert_eq!(&row[0], original);
        assert_eq!(&row[1], expected);
        assert_eq!(&row[2], expected);
        assert_eq!(&row[3], format!("{expected}, {expected}"));
        let nested: Value = serde_json::from_str(&row[4]).unwrap();
        assert_eq!(nested["Created"]["UTC"], expected);
        assert_eq!(nested["Created"]["Filetime"], 123);
        assert_eq!(&row[5], raw);
    }
}

#[cfg(test)]
mod numeric_tests {
    use super::*;

    #[test]
    fn timezone_conversion_keeps_exact_float_scores_and_large_identifiers() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("JumpLists.jsonl");
        let original = r#"{"Created":"2026-10-20T08:46:38.2001668Z","Score":3.9000000953674316,"ID":18446744073709551615,"Raw":{"Score":3.9000000953674316}}"#;
        fs::write(&path, format!("{original}\n")).unwrap();
        rewrite_file(
            &path,
            AutoOutputType::Jsonl,
            "jump-lists",
            "+09:00".parse().unwrap(),
        )
        .unwrap();
        let result = fs::read_to_string(path).unwrap();
        assert_eq!(result.matches("3.9000000953674316").count(), 2, "{result}");
        assert!(result.contains("18446744073709551615"), "{result}");
        assert!(
            result.contains("2026-10-20T17:46:38.2001668+09:00"),
            "{result}"
        );
    }
}
