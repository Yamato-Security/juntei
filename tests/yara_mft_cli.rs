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

//! Synthetic MFTs exercise real fixups, deleted entries, ADS, damaged records and the record-by-record
//! resident scan that does not depend on raw matches.
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn u16_at(b: &mut [u8], at: usize, v: u16) {
    b[at..at + 2].copy_from_slice(&v.to_le_bytes());
}
fn u32_at(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}
fn u64_at(b: &mut [u8], at: usize, v: u64) {
    b[at..at + 8].copy_from_slice(&v.to_le_bytes());
}
fn align(n: usize) -> usize {
    (n + 7) & !7
}
fn resident(kind: u32, name: &str, data: &[u8], instance: u16) -> Vec<u8> {
    let name: Vec<_> = name.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let offset = align(24 + name.len());
    let mut b = vec![0; align(offset + data.len())];
    let length = b.len();
    u32_at(&mut b, 0, kind);
    u32_at(&mut b, 4, length as u32);
    b[9] = (name.len() / 2) as u8;
    if !name.is_empty() {
        u16_at(&mut b, 10, 24);
    }
    u16_at(&mut b, 14, instance);
    u32_at(&mut b, 16, data.len() as u32);
    u16_at(&mut b, 20, offset as u16);
    b[24..24 + name.len()].copy_from_slice(&name);
    b[offset..offset + data.len()].copy_from_slice(data);
    b
}
fn nonresident(instance: u16) -> Vec<u8> {
    let mut b = vec![0; 72];
    u32_at(&mut b, 0, 0x80);
    u32_at(&mut b, 4, 72);
    b[8] = 1;
    u16_at(&mut b, 14, instance);
    u64_at(&mut b, 24, 0);
    u16_at(&mut b, 32, 64);
    u64_at(&mut b, 40, 4096);
    u64_at(&mut b, 48, 4096);
    u64_at(&mut b, 56, 4096);
    // one cluster at LCN 1, then the terminator
    b[64..68].copy_from_slice(&[0x11, 1, 1, 0]);
    b
}
fn record(
    id: u32,
    name: &str,
    streams: &[(&str, &[u8])],
    allocated: bool,
    nonres: bool,
) -> Vec<u8> {
    let mut b = vec![0; 1024];
    b[..4].copy_from_slice(b"FILE");
    u16_at(&mut b, 4, 0x30);
    u16_at(&mut b, 6, 3);
    u16_at(&mut b, 16, if id == 5 { 1 } else { 3 });
    u16_at(&mut b, 18, 1);
    u16_at(&mut b, 20, 0x38);
    u16_at(&mut b, 22, if id == 5 { 3 } else { u16::from(allocated) });
    u32_at(&mut b, 28, 1024);
    u32_at(&mut b, 44, id);
    let utf16: Vec<_> = name.encode_utf16().collect();
    let mut filename = vec![0; 66 + utf16.len() * 2];
    u64_at(&mut filename, 0, 5 | (1u64 << 48));
    filename[64] = utf16.len() as u8;
    filename[65] = 1;
    for (i, c) in utf16.iter().enumerate() {
        u16_at(&mut filename, 66 + i * 2, *c);
    }
    let mut attrs = vec![resident(0x30, "", &filename, 0)];
    for (i, (name, data)) in streams.iter().enumerate() {
        attrs.push(resident(0x80, name, data, (i + 1) as u16));
    }
    if nonres {
        attrs.push(nonresident(9));
    }
    let mut at = 0x38;
    for a in attrs {
        b[at..at + a.len()].copy_from_slice(&a);
        at += a.len();
    }
    u32_at(&mut b, at, u32::MAX);
    u32_at(&mut b, 24, (at + 8) as u32);
    for (index, end) in [510, 1022].into_iter().enumerate() {
        let original = u16::from_le_bytes(b[end..end + 2].try_into().unwrap());
        u16_at(&mut b, 0x32 + index * 2, original);
        u16_at(&mut b, end, 0xAABB);
    }
    u16_at(&mut b, 0x30, 0xAABB);
    b
}
fn mft(dir: &Path, name: &str, records: &[(usize, Vec<u8>)]) -> PathBuf {
    let mut bytes =
        vec![0; 1024 * (records.iter().map(|(id, _)| *id).max().unwrap_or(0).max(5) + 1)];
    bytes[..1024].copy_from_slice(&record(0, "$MFT", &[], true, false));
    bytes[5 * 1024..6 * 1024].copy_from_slice(&record(5, ".", &[], true, false));
    for (id, r) in records {
        bytes[id * 1024..(id + 1) * 1024].copy_from_slice(r);
    }
    let p = dir.join(name);
    fs::write(&p, bytes).unwrap();
    p
}
fn run(input: &Path, rules: &Path, out: &Path, more: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_juntei"))
        .args([
            "yara-scan",
            "-f",
            input.to_str().unwrap(),
            "--rules",
            rules.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "-t",
            "jsonl",
            "-q",
            "-K",
        ])
        .args(more)
        .output()
        .unwrap()
}
fn rows(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}
fn rules(dir: &Path, source: &str) -> PathBuf {
    let p = dir.join("test.yar");
    fs::write(&p, source).unwrap();
    p
}
fn summary(out: &Path) -> Value {
    serde_json::from_slice(&fs::read(out.join("Summary.json")).unwrap()).unwrap()
}
fn statuses(rows: &[Value], status: &str) -> usize {
    rows.iter().filter(|r| r["Status"] == status).count()
}
fn sources(hits: &[Value], source: &str) -> usize {
    hits.iter().filter(|r| r["SourceType"] == source).count()
}

#[test]
fn recovers_fixup_corrected_data_and_reports_stream_provenance() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let mut payload = vec![b'A'; 400];
    payload[..b"MFT_INERT_PAYLOAD!".len()].copy_from_slice(b"MFT_INERT_PAYLOAD!");
    let input = mft(
        root,
        "$MFT",
        &[(1, record(1, "payload.exe", &[("", &payload)], true, false))],
    );
    let r = rules(
        root,
        "rule marker { strings: $a=\"MFT_INERT_PAYLOAD!\" condition: $a } rule correct_size { strings: $a=\"MFT_INERT_PAYLOAD!\" condition: filesize == 400 and $a }",
    );
    let out = root.join("out");
    let result = run(&input, &r, &out, &[]);
    assert!(
        result.status.success(),
        "{} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let recovery = rows(&out.join("YaraMFTResults.jsonl"));
    assert_eq!(recovery.len(), 1);
    let row = &recovery[0];
    assert_eq!(row["Status"], "Recovered");
    assert_eq!(row["EntryNum"], 1);
    assert_eq!(row["SN"], 3);
    assert_eq!(row["Deleted"], false);
    assert!(
        row["OriginalPath"]
            .as_str()
            .unwrap()
            .ends_with("payload.exe")
    );
    assert_eq!(row["Rules"], "marker; correct_size");
    let relative = row["RecoveredFile"].as_str().unwrap();
    assert!(relative.ends_with(".quarantined"));
    assert_eq!(
        fs::read(out.join(relative)).unwrap(),
        payload,
        "fixup-restored bytes, not sector-tail USA words"
    );
    let hits = rows(&out.join("YaraDetections.jsonl"));
    assert_eq!(hits.len(), 3);
    assert_eq!(
        sources(&hits, "MFTRaw"),
        1,
        "filesize == 400 cannot match the MFT"
    );
    assert_eq!(sources(&hits, "MFTResident"), 2);
    assert!(
        hits.iter()
            .filter(|r| r["SourceType"] == "MFTResident")
            .all(|h| h["RecoveredFile"] == relative)
    );
    let s = summary(&out);
    assert_eq!(s["MFT"]["RecoveredFiles"], 1);
    assert_eq!(s["MFT"]["ResidentStreams"], 1);
    assert_eq!(s["MFT"]["Records"], 6);
    assert_eq!(s["MFT"]["UnusedRecords"], 3);
    assert_eq!(s["MFT"]["DamagedRecords"], 0);
}

#[test]
fn rules_that_cannot_match_the_raw_mft_still_recover_resident_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let mut payload = b"MZ\0\0MFT_SMALL_PE_ONLY".to_vec();
    payload.resize(200, 0);
    let input = mft(
        root,
        "$MFT",
        &[(7, record(7, "dropper.exe", &[("", &payload)], true, false))],
    );
    // Both conditions are false for the whole MFT: it is far larger than 1024 bytes and starts
    // with FILE. They are true for the resident file, which must still be found.
    let r = rules(
        root,
        "rule small_pe { strings: $a=\"MFT_SMALL_PE_ONLY\" condition: uint16(0) == 0x5a4d and filesize < 1024 and $a }",
    );
    let out = root.join("out");
    let result = run(&input, &r, &out, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let hits = rows(&out.join("YaraDetections.jsonl"));
    assert_eq!(sources(&hits, "MFTRaw"), 0);
    assert_eq!(sources(&hits, "MFTResident"), 1);
    let recovery = rows(&out.join("YaraMFTResults.jsonl"));
    assert_eq!(statuses(&recovery, "Recovered"), 1);
    assert_eq!(recovery[0]["EntryNum"], 7);
    assert_eq!(
        fs::read(out.join(recovery[0]["RecoveredFile"].as_str().unwrap())).unwrap(),
        payload
    );
}

#[test]
fn metadata_nonresident_and_cross_record_matches_are_not_exported() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let input = mft(
        root,
        "$MFT",
        &[
            (
                1,
                record(
                    1,
                    "MFT_META_ONLY.exe",
                    &[("", b"ordinary content")],
                    true,
                    false,
                ),
            ),
            (2, record(2, "MFT_NONRES.exe", &[], true, true)),
            (
                3,
                record(3, "first", &[("", b"MFT_SPLIT_LEFT")], true, false),
            ),
            (
                4,
                record(4, "second", &[("", b"MFT_SPLIT_RIGHT")], true, false),
            ),
        ],
    );
    let r = rules(
        root,
        "rule metadata { strings: $a=\"MFT_META_ONLY.exe\" wide condition:$a } rule nonresident {strings: $a=\"MFT_NONRES.exe\" wide condition:$a } rule together {strings: $a=\"MFT_SPLIT_LEFT\" $b=\"MFT_SPLIT_RIGHT\" condition:all of them}",
    );
    let out = root.join("out");
    let result = run(&input, &r, &out, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    // The file names and the split pattern match the raw MFT and are reported as such, but no
    // resident stream matches on its own, so nothing is recovered and nothing is invented for
    // the nonresident file.
    let hits = rows(&out.join("YaraDetections.jsonl"));
    assert_eq!(sources(&hits, "MFTRaw"), 3);
    assert_eq!(sources(&hits, "MFTResident"), 0);
    assert!(rows(&out.join("YaraMFTResults.jsonl")).is_empty());
    assert!(!out.join("RecoveredMFT").exists());
    let s = summary(&out);
    assert_eq!(s["MFT"]["ResidentStreams"], 3);
    assert_eq!(s["MFT"]["NonresidentStreams"], 1);
    assert_eq!(s["MFT"]["RecoveredFiles"], 0);
    assert_eq!(s["Status"], "Complete");
}

#[test]
fn deleted_ads_and_hostile_names_stay_inside_output() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let payload = b"MFT_ADS_MATCH";
    let input = mft(
        root,
        "renamed.bin",
        &[(
            1,
            record(
                1,
                "..\\..\\outside.exe",
                &[("", b"safe"), ("evil:stream", payload)],
                false,
                false,
            ),
        )],
    );
    let r = rules(root, "rule ads {strings:$a=\"MFT_ADS_MATCH\" condition:$a}");
    let out = root.join("out");
    let result = run(&input, &r, &out, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let rows = rows(&out.join("YaraMFTResults.jsonl"));
    let found = rows.iter().find(|r| r["Status"] == "Recovered").unwrap();
    assert_eq!(found["Deleted"], true);
    assert_eq!(found["Stream"], "evil:stream");
    let path = out
        .join(found["RecoveredFile"].as_str().unwrap())
        .canonicalize()
        .unwrap();
    assert!(path.starts_with(out.canonicalize().unwrap()));
    assert_eq!(fs::read(path).unwrap(), payload);
    assert!(!root.join("outside.exe").exists());
}

#[test]
fn every_matching_stream_is_recovered_up_to_the_limit() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let recs: Vec<_> = (6..26)
        .map(|id| {
            (
                id,
                record(
                    id as u32,
                    "same.exe",
                    &[("", b"MFT_MANY_MATCH")],
                    true,
                    false,
                ),
            )
        })
        .collect();
    let input = mft(root, "$MFT", &recs);
    let r = rules(
        root,
        "rule many {strings:$a=\"MFT_MANY_MATCH\" condition:$a}",
    );
    for (name, limit, recovered, capped, code) in
        [("all", "1000", 20, 0, 0), ("limited", "2", 2, 18, 3)]
    {
        let out = root.join(name);
        let result = run(&input, &r, &out, &["--max-mft-recoveries", limit]);
        assert_eq!(
            result.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&result.stdout)
        );
        let rows = rows(&out.join("YaraMFTResults.jsonl"));
        assert_eq!(statuses(&rows, "Recovered"), recovered);
        assert_eq!(statuses(&rows, "RecoveryLimitReached"), capped);
        // The limit stops the export, not the detection.
        let hits = self::rows(&out.join("YaraDetections.jsonl"));
        assert_eq!(sources(&hits, "MFTResident"), 20);
    }
}

#[test]
fn damaged_records_are_reported_and_make_coverage_incomplete() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let mut bad = record(1, "bad.exe", &[("", b"MFT_BAD_FIXUP")], true, false);
    bad[510] = 0;
    let mut baad = record(2, "baad.exe", &[("", b"MFT_BAAD_SIG")], true, false);
    baad[..4].copy_from_slice(b"BAAD");
    let input = mft(root, "$MFT", &[(1, bad), (2, baad)]);
    let r = rules(
        root,
        "rule bad {strings:$a=\"MFT_BAD_FIXUP\" condition:$a} rule baad {strings:$a=\"MFT_BAAD_SIG\" condition:$a}",
    );
    let out = root.join("bad");
    let result = run(&input, &r, &out, &[]);
    assert_eq!(
        result.status.code(),
        Some(3),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let rows = rows(&out.join("YaraMFTResults.jsonl"));
    assert_eq!(statuses(&rows, "InvalidRecord"), 2);
    assert_eq!(rows[0]["EntryNum"], 1);
    assert_eq!(rows[1]["EntryNum"], 2);
    assert!(!out.join("RecoveredMFT").exists());
    // The raw bytes still carry the strings, so the raw scan reports them.
    let hits = self::rows(&out.join("YaraDetections.jsonl"));
    assert_eq!(sources(&hits, "MFTRaw"), 2);
    let s = summary(&out);
    assert_eq!(s["MFT"]["DamagedRecords"], 2);
    assert_eq!(s["MFT"]["Incomplete"], true);
    assert_eq!(s["Status"], "Incomplete");
}

#[test]
fn optional_size_caps_and_partial_tail_do_not_shift_records() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let input = mft(
        root,
        "$MFT",
        &[(
            1,
            record(1, "small.exe", &[("", b"MFT_SIZE_MATCH")], true, false),
        )],
    );
    let mut bytes = fs::read(&input).unwrap();
    bytes.resize(2 * 1024 * 1024 + 17, 0);
    fs::write(&input, bytes).unwrap();
    let r = rules(
        root,
        "rule sized {strings:$a=\"MFT_SIZE_MATCH\" condition:$a}",
    );
    let out = root.join("default-mft-limit");
    let result = run(&input, &r, &out, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let rows = rows(&out.join("YaraMFTResults.jsonl"));
    let recovered = rows.iter().find(|r| r["Status"] == "Recovered").unwrap();
    assert_eq!(recovered["EntryNum"], 1);
    assert_eq!(statuses(&rows, "TruncatedTail"), 1);
    let s = summary(&out);
    assert_eq!(s["MFT"]["Records"], 2048);
    assert_eq!(s["MFT"]["UnusedRecords"], 2045);
    let out = root.join("global-limit");
    let result = run(&input, &r, &out, &["--max-file-mib", "1"]);
    assert_eq!(result.status.code(), Some(3));
    let out = root.join("override-limit");
    let result = run(
        &input,
        &r,
        &out,
        &["--max-file-mib", "1", "--max-mft-mib", "3"],
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let s = summary(&out);
    assert_eq!(s["MaxFileMiB"], 1);
    assert_eq!(s["MaxMFTMiB"], 3);
    let out = root.join("small-mft-limit");
    let result = run(&input, &r, &out, &["--max-mft-mib", "1"]);
    assert_eq!(result.status.code(), Some(3));
    assert_eq!(
        self::rows(&out.join("YaraFiles.jsonl"))[0]["Status"],
        "SkippedTooLarge"
    );
    // A skipped MFT gets no record pass either.
    assert_eq!(summary(&out)["MFT"]["Records"], 0);
}

#[test]
fn stale_extension_reference_is_not_attached_to_a_recycled_base() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let mut extension = record(2, "", &[("ads", b"MFT_EXTENSION_MATCH")], true, false);
    // The real base in slot 1 has generation 3. Generation 2 is stale.
    u64_at(&mut extension, 32, 1 | (2u64 << 48));
    let input = mft(
        root,
        "$MFT",
        &[
            (
                1,
                record(1, "innocent-new-generation.exe", &[], true, false),
            ),
            (2, extension),
        ],
    );
    let r = rules(
        root,
        "rule extension {strings:$a=\"MFT_EXTENSION_MATCH\" condition:$a}",
    );
    let out = root.join("out");
    let result = run(&input, &r, &out, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let rows = rows(&out.join("YaraMFTResults.jsonl"));
    let recovered = rows.iter().find(|r| r["Status"] == "Recovered").unwrap();
    assert!(recovered["BaseEntryNum"].is_null());
    assert!(
        !recovered["OriginalPath"]
            .as_str()
            .unwrap()
            .contains("innocent-new-generation")
    );
    assert!(
        recovered["Note"]
            .as_str()
            .unwrap()
            .contains("Unverified/stale")
    );
}

#[test]
fn csv_recovery_and_jsonl_report_the_same_fields() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let input = mft(
        root,
        "$MFT",
        &[(
            1,
            record(1, "match.ps1", &[("", b"MFT_CSV_MATCH")], true, false),
        )],
    );
    let r = rules(
        root,
        "rule csv_match {strings:$a=\"MFT_CSV_MATCH\" condition:$a}",
    );
    let mut records = Vec::new();
    for kind in ["csv", "jsonl"] {
        let out = root.join(kind);
        let result = Command::new(env!("CARGO_BIN_EXE_juntei"))
            .args([
                "yara-scan",
                "-f",
                input.to_str().unwrap(),
                "--rules",
                r.to_str().unwrap(),
                "-o",
                out.to_str().unwrap(),
                "-t",
                kind,
                "-q",
                "-K",
            ])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stdout)
        );
        if kind == "csv" {
            let mut reader = csv::Reader::from_path(out.join("YaraMFTResults.csv")).unwrap();
            let headers = reader.headers().unwrap().clone();
            let row = reader.records().next().unwrap().unwrap();
            let fields: std::collections::BTreeMap<String, String> = headers
                .iter()
                .zip(row.iter())
                .map(|(k, v)| (k.into(), v.into()))
                .collect();
            records.push(fields);
        } else {
            let row = rows(&out.join("YaraMFTResults.jsonl")).remove(0);
            let fields: std::collections::BTreeMap<String, String> = row
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        match v {
                            Value::Null => String::new(),
                            Value::String(s) => s.clone(),
                            _ => v.to_string(),
                        },
                    )
                })
                .collect();
            records.push(fields);
        }
    }
    assert_eq!(records[0], records[1]);
    assert_eq!(records[0]["Status"], "Recovered");
}

#[test]
fn a_truncated_tail_is_reported_and_only_raw_scanned() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let input = mft(root, "$MFT", &[]);
    let mut b = fs::read(&input).unwrap();
    b.extend_from_slice(b"MFT_TAIL_MATCH");
    fs::write(&input, b).unwrap();
    let r = rules(
        root,
        "rule tail {strings:$a=\"MFT_TAIL_MATCH\" condition:$a}",
    );
    let out = root.join("out");
    let result = run(&input, &r, &out, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let hits = rows(&out.join("YaraDetections.jsonl"));
    assert_eq!(sources(&hits, "MFTRaw"), 1);
    let rows = rows(&out.join("YaraMFTResults.jsonl"));
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["Status"], "TruncatedTail");
    assert!(rows[0]["Note"].as_str().unwrap().starts_with("14 bytes"));
    assert!(!out.join("RecoveredMFT").exists());
}

#[test]
fn a_whole_container_regex_does_not_trigger_bulk_recovery() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let input = mft(
        root,
        "$MFT",
        &[
            (
                1,
                record(1, "first", &[("", b"MFT_SPAN_START")], true, false),
            ),
            (2, record(2, "last", &[("", b"MFT_SPAN_END")], true, false)),
        ],
    );
    let r = rules(
        root,
        "rule span {strings:$a=/MFT_SPAN_START.*MFT_SPAN_END/s condition:$a}",
    );
    let out = root.join("out");
    let result = run(&input, &r, &out, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let hits = rows(&out.join("YaraDetections.jsonl"));
    assert_eq!(sources(&hits, "MFTRaw"), 1);
    assert_eq!(sources(&hits, "MFTResident"), 0);
    assert!(rows(&out.join("YaraMFTResults.jsonl")).is_empty());
    assert!(!out.join("RecoveredMFT").exists());
}

#[test]
fn verified_extension_uses_its_exact_base_generation_and_path() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let mut base = record(1, "logical.txt", &[], true, false);
    let mut list = [0; 32];
    u32_at(&mut list, 0, 0x80);
    u16_at(&mut list, 4, 32);
    u64_at(&mut list, 16, 2 | (3u64 << 48));
    u16_at(&mut list, 24, 1);
    let attr = resident(0x20, "", &list, 2);
    let at = u32::from_le_bytes(base[24..28].try_into().unwrap()) as usize - 8;
    base[at..at + attr.len()].copy_from_slice(&attr);
    u32_at(&mut base, at + attr.len(), u32::MAX);
    u32_at(&mut base, 24, (at + attr.len() + 8) as u32);
    let mut extension = record(2, "", &[("", b"MFT_VALID_EXTENSION")], true, false);
    u64_at(&mut extension, 32, 1 | (3u64 << 48));
    let input = mft(root, "$MFT", &[(1, base), (2, extension)]);
    let r = rules(
        root,
        "rule extension {strings:$a=\"MFT_VALID_EXTENSION\" condition:$a}",
    );
    let out = root.join("out");
    let result = run(&input, &r, &out, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let rows = rows(&out.join("YaraMFTResults.jsonl"));
    let row = rows.iter().find(|r| r["Status"] == "Recovered").unwrap();
    assert_eq!(row["EntryNum"], 2);
    assert_eq!(row["BaseEntryNum"], 1);
    assert_eq!(row["BaseSN"], 3);
    assert!(
        row["OriginalPath"]
            .as_str()
            .unwrap()
            .ends_with("logical.txt")
    );
}

#[test]
fn default_scans_an_mft_larger_than_the_former_512_mib_limit() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let input = mft(
        root,
        "$MFT",
        &[(
            1,
            record(
                1,
                "large-source.txt",
                &[("", b"MFT_NO_DEFAULT_LIMIT")],
                true,
                false,
            ),
        )],
    );
    fs::OpenOptions::new()
        .write(true)
        .open(&input)
        .unwrap()
        .set_len(513 * 1024 * 1024 + 17)
        .unwrap();
    let r = rules(
        root,
        "rule no_default_limit {strings:$a=\"MFT_NO_DEFAULT_LIMIT\" condition:$a}",
    );
    let out = root.join("out");
    let result = run(&input, &r, &out, &[]);
    assert!(
        result.status.success(),
        "{} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let s = summary(&out);
    assert!(s["MaxFileMiB"].is_null());
    assert!(s["MaxMFTMiB"].is_null());
    assert_eq!(s["MFT"]["RecoveredFiles"], 1);
    assert_eq!(s["MFT"]["Records"], 513 * 1024);
    assert!(
        !s["Statuses"]
            .as_object()
            .unwrap()
            .contains_key("SkippedTooLarge")
    );
}
