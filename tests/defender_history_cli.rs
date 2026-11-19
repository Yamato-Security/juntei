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

//! `parse-file defender-history` and its `auto-parse` and `super-timeline` integration, checked
//! against what other tools read from the same public files (`test_files/defender_history/`):
//! WMI's `MSFT_MpThreatDetection` for Orange Cyberdefense's samples, plaso's
//! `windefender_history` parser for plaso's.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::process::Command;

const FIXTURES: &str = "test_files/defender_history";
const ORANGE: &str = "94BBE9CF-CDEB-4885-9178-CC93FB10822D";
/// Where Defender keeps its detection history, below a volume's root.
const HISTORY: &str =
    "ProgramData/Microsoft/Windows Defender/Scans/History/Service/DetectionHistory";

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    fn all(&self) -> String {
        format!("{}{}", self.stdout, self.stderr)
    }
}

fn run_at(current_dir: &Path, args: &[&str]) -> Run {
    let out = Command::new(env!("CARGO_BIN_EXE_juntei"))
        .current_dir(current_dir)
        .args(args)
        .output()
        .expect("juntei runs");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    }
}

fn run(args: &[&str]) -> Run {
    run_at(Path::new(env!("CARGO_MANIFEST_DIR")), args)
}

fn history(args: &[&str]) -> Run {
    let mut all = vec!["parse-file", "defender-history"];
    all.extend(args);
    all.extend(["-K", "-q"]);
    run(&all)
}

/// The records of `-t jsonl` on stdout.
fn rows(args: &[&str]) -> Vec<Value> {
    let mut all = args.to_vec();
    all.extend(["-t", "jsonl"]);
    let r = history(&all);
    assert_eq!(r.code, 0, "{}", r.all());
    r.stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn csv_rows(path: &Path) -> (Vec<String>, Vec<HashMap<String, String>>) {
    let mut reader = csv::Reader::from_path(path).unwrap();
    let header: Vec<String> = reader.headers().unwrap().iter().map(String::from).collect();
    let rows = reader
        .records()
        .map(|r| {
            header
                .iter()
                .cloned()
                .zip(r.unwrap().iter().map(String::from))
                .collect()
        })
        .collect();
    (header, rows)
}

fn text<'a>(row: &'a Value, field: &str) -> &'a str {
    row[field].as_str().unwrap_or_default()
}

/// Every file under `dir` with its SHA-256.
fn hashes(dir: &Path) -> BTreeMap<PathBuf, String> {
    let mut out = BTreeMap::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let digest = Sha256::digest(std::fs::read(&path).unwrap());
                out.insert(path, digest.iter().map(|b| format!("{b:02x}")).collect());
            }
        }
    }
    out
}

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES)
        .join(path)
}

#[test]
fn every_resource_of_every_detection_is_a_record_with_a_relative_source() {
    let all = rows(&["-d", FIXTURES]);
    // 18 detections, each with resources: one record per resource.
    assert_eq!(all.len(), 32);
    let mut by_detection: BTreeMap<(String, String), Vec<&Value>> = BTreeMap::new();
    for row in &all {
        assert_eq!(row["Record"], "resource");
        let source = text(row, "SourceFile");
        assert!(
            source.starts_with("museum/DetectionHistory/")
                || source.starts_with("orange/DetectionHistory/")
                || source.starts_with("plaso/DetectionHistory/"),
            "{source}"
        );
        assert!(!source.contains('\\') && !Path::new(source).is_absolute());
        by_detection
            .entry((source.to_owned(), text(row, "DetectionID").to_owned()))
            .or_default()
            .push(row);
    }
    assert_eq!(by_detection.len(), 18);
    // A detection's resources stay together, numbered in the order stored.
    for rows in by_detection.values() {
        let count = rows.len() as u64;
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(row["ResourceIndex"], i as u64 + 1);
            assert_eq!(row["ResourceCount"], count);
        }
    }
    // The same detection kept twice (Orange's original and a copy it changed) is two
    // detections: repeated records are never merged.
    let orange: Vec<&Value> = all
        .iter()
        .filter(|row| row["DetectionID"] == ORANGE)
        .collect();
    assert_eq!(orange.len(), 2);
    assert_ne!(orange[0]["SourceFile"], orange[1]["SourceFile"]);
}

/// What WMI (`MSFT_MpThreatDetection`) gave Orange Cyberdefense for its samples, after each
/// change was put back on the machine (`orange/README` of ys-defender-history-parser).
#[test]
fn orange_samples_read_as_wmi_reported_them() {
    let all = rows(&["-d", &format!("{FIXTURES}/orange")]);
    assert_eq!(all.len(), 2);
    let original = &all[0];
    assert_eq!(
        text(original, "SourceFile"),
        format!("DetectionHistory/original/{ORANGE}")
    );
    assert_eq!(original["DetectionID"], ORANGE);
    assert_eq!(original["ThreatID"], 2_147_686_744_u64);
    assert_eq!(original["ThreatName"], "HackTool:Win32/Mimikatz");
    assert_eq!(original["User"], r"DESKTOP-O8964S4\RaptorSniper");
    assert_eq!(original["UserSource"], "DomainUser");
    assert_eq!(
        original["Process"],
        r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
    );
    assert_eq!(original["DetectionSourceTypeID"], 3);
    assert_eq!(original["DetectionSource"], "Real-time");
    assert_eq!(original["ThreatStatusID"], 3);
    assert_eq!(original["ThreatStatus"], "Quarantined");
    assert_eq!(original["ThreatStatusErrorCode"], 0);
    assert_eq!(original["ExecutionStatusID"], 1);
    assert_eq!(original["ExecutionStatus"], "Blocked");
    // WMI gives milliseconds; the file keeps the FILETIME's 100 ns.
    for (field, wmi) in [
        ("InitialDetectionTime", "2025-01-28T16:44:51.243"),
        ("LastThreatStatusChangeTime", "2025-01-28T16:45:06.220"),
        ("RemediationTime", "2025-01-28T16:45:06.220"),
    ] {
        let time = text(original, field);
        assert!(
            time.starts_with(wmi) && time.ends_with('Z'),
            "{field}: {time}"
        );
    }
    // `Resources {file:_C:\Users\RaptorSniper\Downloads\a.zip}`
    assert_eq!(original["ResourceType"], "file");
    assert_eq!(original["Path"], r"C:\Users\RaptorSniper\Downloads\a.zip");
    assert_eq!(
        original["SHA256"],
        "7accd179e8a6b2fc907e7e8d087c52a7f48084852724b03d25bebcada1acbca5"
    );
    assert_eq!(
        original["QuarantineID"],
        "80031958-0000-0000-862B-597C89800A50"
    );
    assert_eq!(original["MagicVersion"], "Magic.Version:1.2");
    assert_eq!(original["Problems"], serde_json::json!([]));

    // Orange changed the status to 18, which WMI then reported: kept as stored, with no name.
    let changed = &all[1];
    assert_eq!(changed["ThreatStatusID"], 18);
    assert!(changed["ThreatStatus"].is_null());
    assert_eq!(changed["DetectionID"], ORANGE);
}

/// What plaso's `windefender_history` parser reads from its own test files (`plaso.tsv` of
/// ys-defender-history-parser): the file detected, the time it dates the detection by (the
/// last tracking data's start time, in microseconds), the threat, the user, the process, the
/// SHA-256 and the container and web files.
#[test]
fn plaso_files_read_as_plaso_reads_them() {
    let all = rows(&["-d", &format!("{FIXTURES}/plaso")]);
    for (id, file, micros, threat, process, sha256, container, web) in [
        (
            "6AFE33A0-19BA-4FFF-892F-B700539D7D63",
            r"C:\Users\testuser\Downloads\eicar_com.zip->eicar.com",
            "2022-07-22T05:36:18.909442",
            "Virus:DOS/EICAR_Test_File",
            "Unknown",
            "275a021bbfb6489e54d471899f7db9d1663fc695ec2fe2a2c4538aabf651fd0f",
            Some(r"C:\Users\testuser\Downloads\eicar_com.zip"),
            r"C:\Users\testuser\Downloads\eicar_com.zip|https://secure.eicar.org/eicar_com.zip|pid:20580,ProcessStart:133029417782902463",
        ),
        (
            "FC380697-A68D-4C94-B67F-9B6449039463",
            r"C:\Users\testuser\Downloads\PotentiallyUnwanted.exe",
            "2022-07-22T05:32:27.830278",
            "PUA:Win32/EICAR_Test_File",
            r"C:\Windows\explorer.exe",
            "d6f6c6b9fde37694e12b12009ad11ab9ec8dd0f193e7319c523933bdad8a50ad",
            None,
            r"C:\Users\testuser\Downloads\PotentiallyUnwanted.exe|http://amtso.eicar.org/PotentiallyUnwanted.exe|pid:19588,ProcessStart:133029415464710822",
        ),
    ] {
        let detection: Vec<&Value> = all.iter().filter(|r| r["DetectionID"] == id).collect();
        let of = |kind: &str| {
            detection
                .iter()
                .filter(|r| r["ResourceType"] == kind)
                .map(|r| text(r, "Path"))
                .collect::<Vec<_>>()
        };
        assert_eq!(of("file"), [file], "{id}");
        assert_eq!(
            of("containerfile"),
            container.into_iter().collect::<Vec<_>>()
        );
        assert_eq!(of("webfile"), [web], "{id}");
        let last = detection.last().unwrap();
        assert!(text(last, "TrackingStartTime").starts_with(micros), "{id}");
        for row in &detection {
            assert_eq!(row["ThreatName"], threat);
            assert_eq!(row["User"], r"FRYPTOP\testuser");
            assert_eq!(row["Process"], process);
        }
        let file_row = detection
            .iter()
            .find(|r| r["ResourceType"] == "file")
            .unwrap();
        assert_eq!(file_row["SHA256"], sha256);
    }
    // Remediation that failed is not removal: the status, its error and the times stay apart.
    let failed = all
        .iter()
        .find(|r| r["DetectionID"] == "6AFE33A0-19BA-4FFF-892F-B700539D7D63")
        .unwrap();
    assert_eq!(failed["ThreatStatus"], "RemoveFailed");
    assert_eq!(failed["ThreatStatusErrorCode"], -2_142_207_965_i64);
    assert!(failed["QuarantineID"].is_null());
}

#[test]
fn every_tracking_and_unknown_value_is_kept_with_its_type() {
    let all = rows(&[
        "-f",
        &format!("{FIXTURES}/orange/DetectionHistory/original/{ORANGE}"),
    ]);
    assert_eq!(all.len(), 1);
    let row = &all[0];
    // A file given with -f is named without its folders.
    assert_eq!(row["SourceFile"], ORANGE);
    let tracking = row["Tracking"].as_array().unwrap();
    assert_eq!(tracking.len(), 14);
    assert_eq!(
        tracking[0],
        serde_json::json!({"Key": "ThreatTrackingSha256", "Type": "Text", "Value": "7accd179e8a6b2fc907e7e8d087c52a7f48084852724b03d25bebcada1acbca5"})
    );
    // The start time is an integer in the file: kept as one, and only interpreted in its column.
    assert_eq!(
        tracking[3],
        serde_json::json!({"Key": "ThreatTrackingStartTime", "Type": "Integer", "Value": 133_825_562_912_428_256_u64})
    );
    assert_eq!(row["TrackingStartTime"], "2025-01-28T16:44:51.2428256Z");
    assert_eq!(
        tracking[10],
        serde_json::json!({"Key": "ThreatTrackingIsEsuSig", "Type": "Bool", "Value": false})
    );
    assert_eq!(row["CountedList"], serde_json::json!([2, 3, 6]));
    let unknown = row["UnknownValues"].as_array().unwrap();
    assert_eq!(unknown.len(), 19);
    assert!(unknown.contains(
        &serde_json::json!({"Position": "6.13", "Type": "Text", "Value": r"NT AUTHORITY\SYSTEM"})
    ));
}

#[test]
fn csv_json_and_jsonl_hold_the_same_records_and_leave_the_sources_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let before = hashes(&fixture(""));
    let mut outputs = HashMap::new();
    for output_type in ["csv", "json", "jsonl"] {
        let out = dir.path().join(format!("history.{output_type}"));
        let r = history(&[
            "-d",
            FIXTURES,
            "-o",
            out.to_str().unwrap(),
            "-t",
            output_type,
        ]);
        assert_eq!(r.code, 0, "{}", r.all());
        outputs.insert(output_type, out);
    }
    assert_eq!(hashes(&fixture("")), before);

    let read_json = |path: &Path| -> Vec<Value> {
        serde_json::Deserializer::from_str(&std::fs::read_to_string(path).unwrap())
            .into_iter()
            .map(Result::unwrap)
            .collect()
    };
    let json = read_json(&outputs["json"]);
    assert_eq!(json, read_json(&outputs["jsonl"]));
    let (header, csv) = csv_rows(&outputs["csv"]);
    assert_eq!(csv.len(), json.len());
    for (row, record) in csv.iter().zip(&json) {
        for column in &header {
            let cell = &row[column];
            let expected = match (column.as_str(), &record[column]) {
                // The CSV joins what JSON keeps as lists of typed values.
                ("OtherTracking" | "UnknownValues", _) => continue,
                ("ResourceFlags", Value::Number(n)) => format!("0x{:x}", n.as_u64().unwrap()),
                ("Problems", Value::Array(items)) => items
                    .iter()
                    .map(|item| item.as_str().unwrap())
                    .collect::<Vec<_>>()
                    .join("; "),
                (_, Value::Null) => String::new(),
                (_, Value::String(text)) => text.clone(),
                (_, other) => other.to_string(),
            };
            assert_eq!(cell, &expected, "{column}");
        }
        // Every tracking value without a column of its own is in OtherTracking, in order.
        let others: Vec<String> = record["Tracking"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|pair| {
                ![
                    "ThreatTrackingStartTime",
                    "ThreatTrackingSha256",
                    "ThreatTrackingSha1",
                    "ThreatTrackingMD5",
                    "ThreatTrackingSize",
                    "ThreatTrackingId",
                ]
                .contains(&text(pair, "Key"))
            })
            .map(|pair| {
                let value = match &pair["Value"] {
                    Value::String(text) => text.clone(),
                    other => other.to_string(),
                };
                format!("{}={value}", text(pair, "Key"))
            })
            .collect();
        assert_eq!(row["OtherTracking"], others.join("; "));
    }
}

/// Write `bytes` as `<dir>/DetectionHistory/01/<name>`.
fn put(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let folder = dir.join("DetectionHistory/01");
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Orange's sample without its header's `Magic.Version:1.2`, as some Windows Server 2016 files
/// are written: the value at 0x28 (its size, type, data and padding to 8 bytes) taken out.
fn without_version(data: &[u8]) -> Vec<u8> {
    let size = u32::from_le_bytes(data[0x28..0x2C].try_into().unwrap()) as usize;
    let end = (0x30 + size).next_multiple_of(8);
    [&data[..0x28], &data[end..]].concat()
}

#[test]
fn version_differences_damage_and_other_files_are_reported_without_losing_the_rest() {
    let dir = tempfile::tempdir().unwrap();
    let original = std::fs::read(fixture(&format!(
        "orange/DetectionHistory/original/{ORANGE}"
    )))
    .unwrap();
    put(dir.path(), ORANGE, &without_version(&original));
    put(
        dir.path(),
        "11111111-0000-0000-0000-000000000000",
        &original[..600],
    );
    put(dir.path(), "notes.txt", b"not a detection\n");
    put(dir.path(), "empty", b"");
    let r = history(&["-d", dir.path().to_str().unwrap(), "-t", "jsonl"]);
    assert_eq!(r.code, 0, "{}", r.all());
    assert!(
        r.stderr.contains(
            "DetectionHistory/01/notes.txt: does not start like a detection history file"
        ),
        "{}",
        r.stderr
    );
    assert!(
        r.stderr.contains("DetectionHistory/01/empty: empty"),
        "{}",
        r.stderr
    );
    let all: Vec<Value> = r
        .stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(all.len(), 2);
    // Cut short: what was read is kept, with what is missing and the name that doesn't match.
    let cut = &all[0];
    assert_eq!(cut["ThreatName"], "HackTool:Win32/Mimikatz");
    assert_eq!(cut["Path"], r"C:\Users\RaptorSniper\Downloads\a.zip");
    assert!(cut["RemediationTime"].is_null());
    let problems: Vec<&str> = cut["Problems"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap())
        .collect();
    assert!(
        problems.iter().any(|p| p.contains("past the end")),
        "{problems:?}"
    );
    assert!(
        problems.contains(&format!("its DetectionID ({ORANGE}) isn't its file's name").as_str()),
        "{problems:?}"
    );
    // Without the header's version: the same detection, no problem.
    let old = &all[1];
    assert!(old["MagicVersion"].is_null());
    assert_eq!(old["ThreatName"], "HackTool:Win32/Mimikatz");
    assert_eq!(old["User"], r"DESKTOP-O8964S4\RaptorSniper");
    assert_eq!(old["RemediationTime"], "2025-01-28T16:45:06.2208881Z");
    assert_eq!(old["Problems"], serde_json::json!([]));

    // The summary counts what was read and what was not.
    let r = history(&["-d", dir.path().to_str().unwrap()]);
    assert_eq!(r.code, 0, "{}", r.all());
    for line in [
        "Detections parsed: 2",
        "Not detection history files: 2",
        "Detections with problems: 1",
    ] {
        assert!(r.stdout.contains(line), "{line}: {}", r.stdout);
    }
}

#[test]
fn input_and_output_errors() {
    let dir = tempfile::tempdir().unwrap();
    let notes = put(dir.path(), "notes.txt", b"not a detection\n");
    // -f given a file that isn't one: nothing read, an error.
    let r = history(&["-f", notes.to_str().unwrap()]);
    assert_eq!(r.code, 1);
    assert!(
        r.all()
            .contains("does not start like a detection history file"),
        "{}",
        r.all()
    );
    // A directory without a DetectionHistory folder: skipped by auto-parse's rule.
    let empty = tempfile::tempdir().unwrap();
    let r = history(&["-d", empty.path().to_str().unwrap()]);
    assert_eq!(r.code, 1);
    assert!(
        r.all().contains("No Defender detection history files"),
        "{}",
        r.all()
    );
    assert!(r.all().contains(" were found in "), "{}", r.all());
    let r = history(&["-d", "no-such-directory"]);
    assert_eq!(r.code, 1);
    assert!(r.all().contains("The directory does not exist"));
    // The output is never one of the inputs, also with --clobber.
    let input = put(
        dir.path(),
        ORANGE,
        &std::fs::read(fixture(&format!(
            "orange/DetectionHistory/original/{ORANGE}"
        )))
        .unwrap(),
    );
    let before = std::fs::read(&input).unwrap();
    let r = history(&[
        "-f",
        input.to_str().unwrap(),
        "-o",
        input.to_str().unwrap(),
        "-C",
    ]);
    assert_eq!(r.code, 1, "{}", r.all());
    assert_eq!(std::fs::read(&input).unwrap(), before);
    // An existing output needs --clobber.
    let out = dir.path().join("out.csv");
    std::fs::write(&out, "keep").unwrap();
    let r = history(&["-f", input.to_str().unwrap(), "-o", out.to_str().unwrap()]);
    assert_eq!(r.code, 1, "{}", r.all());
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "keep");
}

#[test]
fn help_names_the_command_and_its_options() {
    let r = run(&["parse-file", "defender-history", "-h"]);
    assert_eq!(r.code, 0, "{}", r.all());
    for needle in [
        "juntei parse-file defender-history <INPUT> [OPTIONS]",
        "--directory <DIR>",
        "--file <FILE>",
        "--output <FILE>",
        "--output-type <TYPE>",
        "--clobber",
    ] {
        assert!(r.stdout.contains(needle), "{needle}: {}", r.stdout);
    }
}

/// HOST1 with the museum's and plaso's detection history files where Defender keeps them, and
/// HOST2 without any.
fn evidence(root: &Path) -> PathBuf {
    let evidence = root.join("evidence");
    for host in ["HOST1", "HOST2"] {
        std::fs::create_dir_all(evidence.join(host).join("C/Windows/System32")).unwrap();
    }
    let history = evidence.join("HOST1/C").join(HISTORY);
    for (from, to) in [
        ("museum/DetectionHistory", ""),
        ("plaso/DetectionHistory", "30"),
    ] {
        let from = fixture(from);
        for path in hashes(&from).into_keys() {
            let target = history.join(to).join(path.strip_prefix(&from).unwrap());
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(&path, target).unwrap();
        }
    }
    evidence
}

fn tasks<'a>(manifest: &'a Value, artifact: &str) -> Vec<&'a Value> {
    manifest["Tasks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|task| task["Artifact"] == artifact)
        .collect()
}

#[test]
fn auto_parse_saves_detection_history_under_security_products_and_the_timelines() {
    let dir = tempfile::tempdir().unwrap();
    let evidence = evidence(dir.path());
    let before = hashes(&evidence);
    let output = dir.path().join("parsed");
    let timeline = dir.path().join("timeline.jsonl");
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-j",
        timeline.to_str().unwrap(),
        "-Z",
        "+09:00",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    assert_eq!(hashes(&evidence), before);

    let (header, rows) = csv_rows(&output.join("HOST1/SecurityProducts/DefenderHistory.csv"));
    assert_eq!(header[0], "Record");
    assert_eq!(rows.len(), 30);
    for row in &rows {
        assert!(
            row["SourceFile"].starts_with(HISTORY),
            "{}",
            row["SourceFile"]
        );
    }
    // Defender's UTC times in the output timezone, converted once; text is left as it is.
    let plaso = rows
        .iter()
        .find(|row| row["DetectionID"] == "FC380697-A68D-4C94-B67F-9B6449039463")
        .unwrap();
    assert_eq!(
        plaso["InitialDetectionTime"],
        "2022-07-22T14:32:27.6558959+09:00"
    );
    assert!(plaso["Path"].ends_with("PotentiallyUnwanted.exe"));
    assert!(
        !output
            .join("HOST2/SecurityProducts/DefenderHistory.csv")
            .exists()
    );

    let manifest: Value =
        serde_json::from_str(&std::fs::read_to_string(output.join("manifest.json")).unwrap())
            .unwrap();
    let (artifact, exports): (Vec<_>, Vec<_>) = tasks(&manifest, "defender-history")
        .into_iter()
        .partition(|task| task["Purpose"] == "artifact");
    assert_eq!(artifact.len(), 2);
    assert_eq!(exports.len(), 2);
    for task in &artifact {
        assert_eq!(task["Category"], "SecurityProducts");
        let command: Vec<&str> = task["Command"]
            .as_array()
            .unwrap()
            .iter()
            .map(|arg| arg.as_str().unwrap())
            .collect();
        assert_eq!(command[1..4], ["parse-file", "defender-history", "-d"]);
        let host = task["Comp"].as_str().unwrap();
        assert_eq!(Path::new(command[4]), Path::new(".").join(host).join("C"));
        if host == "HOST2" {
            assert_eq!(task["Status"], "skipped");
        } else {
            assert_eq!(task["Status"], "succeeded", "{task}");
        }
    }
    let log =
        std::fs::read_to_string(output.join("HOST1/SecurityProducts/DefenderHistory-logs.txt"))
            .unwrap();
    assert!(log.contains("Detections parsed: 16"), "{log}");

    // The combined timeline has every record, with its computer.
    let combined: Vec<Value> = std::fs::read_to_string(&timeline)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .filter(|record: &Value| record["Artifact"] == "defender-history")
        .collect();
    assert_eq!(combined.len(), 30);
    assert!(combined.iter().all(|record| record["Comp"] == "HOST1"));

    // No output names where the evidence or the output is.
    for path in hashes(&output).into_keys() {
        let content = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
        let root = dir.path().to_string_lossy().into_owned();
        assert!(!content.contains(&root), "{} names {root}", path.display());
    }

    // super-timeline: Defender's three times on each record. The tracking start time is an
    // integer read as a FILETIME, projected only when interpreted times are asked for.
    let mut counts = Vec::new();
    for extra in [None, Some("--include-interpreted-times")] {
        let out = dir.path().join("super.jsonl");
        let mut args = vec![
            "super-timeline",
            "-d",
            output.to_str().unwrap(),
            "--artifact",
            "defender-history",
            "-o",
            out.to_str().unwrap(),
            "-t",
            "jsonl",
            "-C",
            "-K",
            "-q",
        ];
        args.extend(extra);
        let r = run(&args);
        assert_eq!(r.code, 0, "{}", r.all());
        let events: Vec<Value> = std::fs::read_to_string(&out)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let mut types: BTreeMap<String, usize> = BTreeMap::new();
        for event in &events {
            assert_eq!(event["Artifact"], "defender-history");
            *types
                .entry(text(event, "TimestampType").to_owned())
                .or_default() += 1;
        }
        counts.push(types);
    }
    let (plain, interpreted) = (&counts[0], &counts[1]);
    assert!(!plain.keys().any(|k| k.contains("tracking")), "{plain:?}");
    assert_eq!(plain["Defender threat status change"], 30);
    assert_eq!(plain["Defender remediation"], 27);
    assert_eq!(plain["Defender initial detection"], 22);
    assert_eq!(
        interpreted["Defender resource tracking start, integer interpreted as FILETIME"],
        29
    );
    assert_eq!(
        plain["Defender initial detection"],
        interpreted["Defender initial detection"]
    );
}
