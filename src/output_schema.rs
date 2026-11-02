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

//! Public output labels. Evidence values, flags and source-provided dictionary keys are unchanged.
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use std::io::{self, Write};

pub fn name(input: &str) -> String {
    match input {
        "SI<FN" => return "SI-BeforeFN".into(),
        "uSecZeros" => return "MicrosecondZeros".into(),
        "_value_state" => return "ValueState".into(),
        "_raw" => return "Raw".into(),
        _ => (),
    }
    let mut words = Vec::new();
    for chunk in input.split(['_', '-', ' ']).filter(|s| !s.is_empty()) {
        if acronym(chunk) {
            words.push(chunk);
            continue;
        }
        let chars: Vec<_> = chunk.char_indices().collect();
        let mut start = 0;
        for i in 1..chars.len() {
            let before = chars[i - 1].1;
            let current = chars[i].1;
            let next = chars.get(i + 1).map(|c| c.1);
            if current.is_uppercase()
                && (before.is_lowercase()
                    || before.is_ascii_digit()
                    || (before.is_uppercase() && next.is_some_and(char::is_lowercase)))
            {
                words.push(&chunk[start..chars[i].0]);
                start = chars[i].0;
            }
        }
        words.push(&chunk[start..]);
    }
    let mut out = String::with_capacity(input.len() + 4);
    let mut previous_acronym = false;
    for word in words {
        if previous_acronym && !out.is_empty() {
            out.push('-');
        }
        let lower = word.to_ascii_lowercase();
        let is_acronym = acronym(word)
            || (word.chars().filter(|c| c.is_alphabetic()).count() > 1
                && word.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()));
        match lower.as_str() {
            "ipv4" => out.push_str("IPv4"),
            "ipv6" => out.push_str("IPv6"),
            "netbios" => out.push_str("NetBIOS"),
            _ if is_acronym => out.push_str(&word.to_uppercase()),
            _ => {
                let mut chars = word.chars();
                if let Some(first) = chars.next() {
                    out.extend(first.to_uppercase());
                    out.extend(chars);
                }
            }
        }
        previous_acronym = is_acronym;
    }
    out
}

fn acronym(word: &str) -> bool {
    matches!(
        word.to_ascii_lowercase().as_str(),
        "ace"
            | "acl"
            | "ads"
            | "amsi"
            | "appx"
            | "ascii"
            | "av"
            | "bam"
            | "com"
            | "cpu"
            | "crc"
            | "csv"
            | "dacl"
            | "dam"
            | "dcom"
            | "dll"
            | "dns"
            | "dpapi"
            | "dps"
            | "dword"
            | "edr"
            | "efs"
            | "eula"
            | "exe"
            | "fn"
            | "fqdn"
            | "gb"
            | "gpo"
            | "guid"
            | "hid"
            | "hwid"
            | "id"
            | "ids"
            | "inf"
            | "ip"
            | "ipv4"
            | "ipv6"
            | "json"
            | "jsonl"
            | "kb"
            | "lcn"
            | "lolbin"
            | "lsn"
            | "luid"
            | "mac"
            | "mb"
            | "md5"
            | "mft"
            | "mru"
            | "mui"
            | "netbios"
            | "ntfs"
            | "os"
            | "pe"
            | "pid"
            | "pnp"
            | "qword"
            | "rdp"
            | "rdpdr"
            | "rid"
            | "rpc"
            | "sacl"
            | "sddl"
            | "sha1"
            | "sha256"
            | "si"
            | "sid"
            | "smb"
            | "srum"
            | "tid"
            | "uac"
            | "unc"
            | "uri"
            | "url"
            | "usb"
            | "usn"
            | "utc"
            | "vcn"
            | "vfu"
            | "wmi"
            | "wow64"
            | "xml"
    )
}

/// Check before writing: never merge two source columns that normalize to the same label.
pub fn headers<'a>(labels: impl IntoIterator<Item = &'a str>) -> io::Result<Vec<String>> {
    let mut seen = HashSet::new();
    labels
        .into_iter()
        .map(|original| {
            let label = name(original);
            if !seen.insert(label.clone()) {
                return Err(io::Error::other(format!(
                    "Output field collision: {original:?} becomes {label:?}"
                )));
            }
            Ok(label)
        })
        .collect()
}

/// Rename record fields without interpreting or changing any of their values. Dictionaries
/// containing registry-value/Zone.Identifier names retain their original keys and spelling.
pub fn normalize(value: &mut Value) -> io::Result<()> {
    match value {
        Value::Array(values) => values.iter_mut().try_for_each(normalize),
        Value::Object(fields) => {
            let mut renamed = serde_json::Map::with_capacity(fields.len());
            for (key, mut child) in std::mem::take(fields) {
                let label = name(&key);
                if !matches!(label.as_str(), "Details" | "DataRaw" | "ZoneIdentifier") {
                    normalize(&mut child)?;
                }
                if renamed.insert(label.clone(), child).is_some() {
                    return Err(io::Error::other(format!(
                        "Output field collision at {key:?}: {label:?}"
                    )));
                }
            }
            *fields = renamed;
            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn to_value(value: &impl Serialize) -> io::Result<Value> {
    let mut value = serde_json::to_value(value)?;
    normalize(&mut value)?;
    Ok(value)
}

pub fn write_json(mut out: impl Write, value: &impl Serialize, pretty: bool) -> io::Result<()> {
    let value = to_value(value)?;
    if pretty {
        serde_json::to_writer_pretty(&mut out, &value)?;
    } else {
        serde_json::to_writer(&mut out, &value)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn acronym_boundaries_and_pascal_case() {
        for (before, after) in [
            ("IP_Address", "IP-Address"),
            ("MFTEntry", "MFT-Entry"),
            ("USNJournal", "USN-Journal"),
            ("ProcessId", "ProcessID"),
            ("timestamp_utc", "TimestampUTC"),
            ("is_pe_file", "IsPE-File"),
            (
                "source_volume_key_last_written_utc",
                "SourceVolumeKeyLastWrittenUTC",
            ),
            ("sha256", "SHA256"),
            ("SHA1OfContent", "SHA1-OfContent"),
            ("Created0x10", "Created0x10"),
            ("SI<FN", "SI-BeforeFN"),
        ] {
            assert_eq!(name(before), after, "{before}");
            assert_eq!(name(after), after);
        }
    }
    #[test]
    fn preserve_values_and_evidence_dictionaries() {
        let mut v = json!({"process_id":42,"details":{"raw_name":"x","ProcessId":"y"},
            "zone_identifier":{"HostUrl":"https://example.test"},
            "registry_values":[{"name":"some_name","raw_data_hex":"00"}],
            "unknown_flags":["SOME_FLAG"],"data_raw":{"some_key":"00"}});
        normalize(&mut v).unwrap();
        assert_eq!(v["ProcessID"], 42);
        assert_eq!(v["Details"]["raw_name"], "x");
        assert_eq!(v["Details"]["ProcessId"], "y");
        assert_eq!(v["ZoneIdentifier"]["HostUrl"], "https://example.test");
        assert_eq!(v["RegistryValues"][0]["Name"], "some_name");
        assert_eq!(v["UnknownFlags"][0], "SOME_FLAG");
        assert_eq!(v["DataRaw"]["some_key"], "00");
    }
    #[test]
    fn collisions_fail_instead_of_discarding_data() {
        assert!(headers(["UserId", "UserID"]).is_err());
        assert!(normalize(&mut json!({"user_id":1,"UserID":2})).is_err());
    }
}
