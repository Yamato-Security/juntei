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

//! End-to-end timeline tests exercise file protection, chronology, profiles and provenance.
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_juntei"))
        .arg("super-timeline")
        .args(args)
        .args(["-q", "-K"])
        .output()
        .unwrap()
}
fn ok(result: Output) {
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
fn bad(result: Output, expected: &str) {
    assert!(!result.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(combined.contains(expected), "{combined}");
}
fn write_jsonl(path: &Path, rows: &[Value]) {
    fs::write(
        path,
        rows.iter().map(|v| format!("{v}\n")).collect::<String>(),
    )
    .unwrap();
}
fn jsonl(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}
fn csv(path: &Path) -> Vec<Value> {
    let mut reader = csv::Reader::from_path(path).unwrap();
    let headers = reader.headers().unwrap().clone();
    reader
        .records()
        .map(|record| {
            let record = record.unwrap();
            let fields: serde_json::Map<String, Value> = headers
                .iter()
                .zip(record.iter())
                .map(|(k, v)| {
                    (
                        k.into(),
                        if k == "Details" {
                            serde_json::from_str(v).unwrap()
                        } else {
                            json!(v)
                        },
                    )
                })
                .collect();
            Value::Object(fields)
        })
        .collect()
}
fn seed(dir: &Path) -> std::path::PathBuf {
    let input = dir.join("records.jsonl");
    write_jsonl(
        &input,
        &[
            json!({"Artifact":"prefetch","Computer":"HOST","SourceFile":"capture/first.pf","ExecutableName":"example.exe","LastRunTimes":["2026-10-20T08:46:38.200166800Z","1969-01-01T00:00:00Z"]}),
            json!({"Artifact":"prefetch-timeline","Computer":"HOST","SourceFile":"capture/first.pf","ExecutableName":"example.exe","RunTime":"2026-10-20T08:46:38.200166800Z"}),
            json!({"Artifact":"mft","Computer":"HOST","SourceFile":"capture/$MFT","EntryNum":10,"SequenceNumber":3,"FileName":"example.exe","NameType":"Win32","Created (SI)":"2026-10-20T08:46:38.200166800Z","Created (FN)":"2026-10-20T08:46:38.200166800Z"}),
            json!({"Artifact":"registry","Computer":"HOST","SourcePath":"capture/SYSTEM","RecordType":"key","KeyPath":"ControlSet001\\Services\\example","KeyLastWritten":"2026-10-20T09:00:00Z"}),
            json!({"Artifact":"registry","Computer":"HOST","SourcePath":"capture/SYSTEM","RecordType":"value","KeyPath":"ControlSet001\\Services\\example","ValueName":"ImagePath","KeyLastWritten":"2026-10-20T09:00:00Z"}),
            json!({"Artifact":"amcache","Computer":"HOST","Source":"capture/Amcache.hve","KeyName":"example","LinkDate":"invalid","KeyLastWritten":""}),
        ],
    );
    input
}
#[test]
fn csv_jsonl_timesketch_and_threads_have_identical_events() {
    let dir = tempfile::tempdir().unwrap();
    let input = seed(dir.path());
    let mut native = Vec::new();
    for profile in ["native", "timesketch"] {
        let mut output_rows = Vec::new();
        for format in ["jsonl", "csv"] {
            let output = dir.path().join(format!("{profile}-{format}.{format}"));
            ok(run(&[
                "-f",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
                "-t",
                format,
                "--profile",
                profile,
                "-T",
                if format == "csv" { "1" } else { "4" },
                "--memory-limit",
                "8",
            ]));
            output_rows.push(if format == "csv" {
                csv(&output)
            } else {
                jsonl(&output)
            });
        }
        assert_eq!(output_rows[0], output_rows[1]);
        if profile == "native" {
            native = output_rows.remove(0);
        } else {
            for row in &mut output_rows[0] {
                let map = row.as_object_mut().unwrap();
                for (old, new) in [
                    ("datetime", "Timestamp"),
                    ("timestamp_desc", "TimestampType"),
                    ("message", "Message"),
                ] {
                    let value = map.remove(old).unwrap();
                    map.insert(new.into(), value);
                }
            }
            assert_eq!(native, output_rows[0]);
        }
    }
    assert_eq!(native.len(), 5); // two executions, SI, FN and one key time
    assert_eq!(native[0]["Timestamp"], "1969-01-01T00:00:00.000000000Z");
    assert!(native.iter().any(|r| r["Details"]["DuplicateCount"] == 1));
    let summary: Value =
        serde_json::from_slice(&fs::read(dir.path().join("native-jsonl-Summary.json")).unwrap())
            .unwrap();
    assert_eq!(summary["Counts"]["DuplicateEvents"], 1);
    assert_eq!(summary["Counts"]["InvalidTimestamps"], 1);
    assert_eq!(
        jsonl(&dir.path().join("native-jsonl-Untimed.jsonl")).len(),
        2
    );
}
#[test]
fn hayabusa_string_details_and_correlations_merge_without_losing_events() {
    let dir = tempfile::tempdir().unwrap();
    let input = seed(dir.path());
    let haya = dir.path().join("alerts.jsonl");
    write_jsonl(
        &haya,
        &[
            json!({"Timestamp":"2026-10-20 06:00:00.123 +00:00","Computer":"DC.domain.local","Details":"Informational native message","RuleID":"rule","RecordID":1,"EvtxFile":"capture/Security.evtx"}),
            json!({"Timestamp":"2026-10-20T06:00:00.123Z","Computer":"DC.domain.local","Details":{"Count":5},"RuleID":"correlation","EvtxFile":"capture/Security.evtx"}),
            json!({"Timestamp":"2026-10-20T06:00:00.123Z","Computer":"DC.domain.local","Details":{"Count":6},"RuleID":"correlation","EvtxFile":"capture/Security.evtx"}),
        ],
    );
    let output = dir.path().join("timeline.jsonl");
    ok(run(&[
        "-f",
        input.to_str().unwrap(),
        "--hayabusa",
        haya.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-t",
        "jsonl",
    ]));
    let rows = jsonl(&output);
    assert_eq!(rows.len(), 8);
    let haya: Vec<_> = rows
        .iter()
        .filter(|r| r["Artifact"] == "hayabusa")
        .collect();
    assert_eq!(haya.len(), 3);
    assert!(haya.iter().any(|r| r["Details"]["Details"].is_string()));
    assert!(
        haya.iter()
            .all(|r| r["SourceFile"] == "capture/Security.evtx")
    );
}
fn manifest(root: &Path, files: &[(&str, &str)]) {
    fs::create_dir_all(root).unwrap();
    let recorded = root.parent().unwrap().join("original").join("parsed");
    let tasks:Vec<_>=files.iter().map(|(name,artifact)|json!({"Computer":"HOST","Artifact":artifact,"Status":"succeeded","Purpose":"artifact","OutputType":"csv","OutputFiles":[recorded.join(name)]})).collect();
    fs::write(
        root.join("manifest.json"),
        serde_json::to_vec(
            &json!({"OutputDirectory":recorded,"Hosts":[{"Computer":"HOST"}],"Tasks":tasks}),
        )
        .unwrap(),
    )
    .unwrap();
}
#[test]
fn relocated_manifest_discovers_csv_by_artifact_and_reports_unresolved_sources() {
    let dir = tempfile::tempdir().unwrap();
    let parsed = dir.path().join("relocated");
    manifest(&parsed, &[("renamed.csv", "prefetch")]);
    fs::write(
        parsed.join("renamed.csv"),
        "ExecutableName,LastRun\nexample.exe,2026-10-20T08:00:00Z\n",
    )
    .unwrap();
    let output = dir.path().join("timeline.jsonl");
    ok(run(&[
        "-d",
        parsed.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-t",
        "jsonl",
    ]));
    let rows = jsonl(&output);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["Computer"], "HOST");
    assert_eq!(rows[0]["Details"]["RawSourceAvailable"], false);
    assert_eq!(
        rows[0]["SourceFile"],
        fs::canonicalize(parsed.join("renamed.csv"))
            .unwrap()
            .to_str()
            .unwrap()
    );
}
#[test]
fn manifest_traversal_is_rejected_before_reading() {
    let dir = tempfile::tempdir().unwrap();
    let parsed = dir.path().join("parsed");
    manifest(&parsed, &[("../outside.csv", "prefetch")]);
    fs::write(
        dir.path().join("outside.csv"),
        "LastRun\n2026-10-20T08:00:00Z\n",
    )
    .unwrap();
    let output = dir.path().join("timeline.csv");
    bad(
        run(&[
            "-d",
            parsed.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ]),
        "Invalid relative path",
    );
    assert!(!output.exists());
}
#[cfg(unix)]
#[test]
fn symlinked_manifest_input_cannot_escape_the_parsed_directory() {
    let dir = tempfile::tempdir().unwrap();
    let parsed = dir.path().join("parsed");
    manifest(&parsed, &[("link.csv", "prefetch")]);
    let outside = dir.path().join("outside.csv");
    fs::write(&outside, "LastRun\n2026-10-20T08:00:00Z\n").unwrap();
    std::os::unix::fs::symlink(outside, parsed.join("link.csv")).unwrap();
    bad(
        run(&[
            "-d",
            parsed.to_str().unwrap(),
            "-o",
            dir.path().join("timeline.csv").to_str().unwrap(),
        ]),
        "symlink points outside",
    );
}
#[test]
fn malformed_input_never_replaces_existing_timeline_or_companions() {
    let dir = tempfile::tempdir().unwrap();
    let input = seed(dir.path());
    let output = dir.path().join("timeline.csv");
    for name in [
        "timeline.csv",
        "timeline-Summary.json",
        "timeline-Diagnostics.jsonl",
        "timeline-Untimed.jsonl",
    ] {
        fs::write(dir.path().join(name), "old content").unwrap();
    }
    use std::io::Write;
    fs::OpenOptions::new()
        .append(true)
        .open(&input)
        .unwrap()
        .write_all(b"{broken JSON\n")
        .unwrap();
    bad(
        run(&[
            "-f",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-C",
        ]),
        "no timeline or companions were replaced",
    );
    for name in [
        "timeline.csv",
        "timeline-Summary.json",
        "timeline-Diagnostics.jsonl",
        "timeline-Untimed.jsonl",
    ] {
        assert_eq!(
            fs::read_to_string(dir.path().join(name)).unwrap(),
            "old content"
        );
    }
}
#[test]
fn input_hardlinks_and_referenced_raw_sources_cannot_be_clobbered() {
    let dir = tempfile::tempdir().unwrap();
    let input = seed(dir.path());
    let alias = dir.path().join("hardlink.jsonl");
    fs::hard_link(&input, &alias).unwrap();
    assert!(
        !run(&[
            "-f",
            input.to_str().unwrap(),
            "-o",
            alias.to_str().unwrap(),
            "-C"
        ])
        .status
        .success()
    );
    let raw = dir.path().join("raw.pf");
    fs::write(&raw, "evidence").unwrap();
    write_jsonl(
        &input,
        &[json!({"Artifact":"prefetch","SourceFile":raw,"LastRun":"2026-10-20T08:00:00Z"})],
    );
    assert!(
        !run(&[
            "-f",
            input.to_str().unwrap(),
            "-o",
            raw.to_str().unwrap(),
            "-C"
        ])
        .status
        .success()
    );
    assert_eq!(fs::read_to_string(&raw).unwrap(), "evidence");
}
#[test]
fn timezone_bounds_and_date_precision_are_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.jsonl");
    write_jsonl(
        &input,
        &[
            json!({"Artifact":"amcache","Computer":"HOST","Name":"program.exe","InstallDate":"2026-10-20"}),
            json!({"Artifact":"prefetch","LastRun":"2026-10-20T00:00:00Z"}),
        ],
    );
    let output = dir.path().join("timeline.jsonl");
    ok(run(&[
        "-f",
        input.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-t",
        "jsonl",
    ]));
    assert_eq!(jsonl(&output).len(), 1);
    assert!(
        fs::read_to_string(dir.path().join("timeline-Diagnostics.jsonl"))
            .unwrap()
            .contains("no known time zone")
    );
    ok(run(&[
        "-f",
        input.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-t",
        "jsonl",
        "-C",
        "--assume-offset",
        "+09:00",
        "--timeline-start",
        "2026-10-19T15:00:00Z",
        "--timeline-end",
        "2026-10-19T15:00:00Z",
    ]));
    let rows = jsonl(&output);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["Details"]["TimePrecision"], "day");
    assert_eq!(rows[0]["Timestamp"], "2026-10-19T15:00:00.000000000Z");
    bad(
        run(&[
            "-f",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-C",
            "--assume-offset",
            "JST",
        ]),
        "quoted signed offset",
    );
    bad(
        run(&[
            "-f",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-C",
            "--timeline-start",
            "2026-10-20 00:00:00",
        ]),
        "offset",
    );
}
#[test]
fn empty_and_untimed_records_do_not_create_epoch_events() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.jsonl");
    write_jsonl(
        &input,
        &[json!({"Artifact":"muicache","Name":"No time","KeyLastWrittenUTC":null})],
    );
    let output = dir.path().join("timeline.csv");
    ok(run(&[
        "-f",
        input.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
    ]));
    assert!(csv(&output).is_empty());
    assert_eq!(jsonl(&dir.path().join("timeline-Untimed.jsonl")).len(), 1);
    fs::write(&input, "").unwrap();
    ok(run(&[
        "-f",
        input.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-C",
    ]));
    assert!(csv(&output).is_empty());
}
#[test]
fn oversized_lines_and_already_normalized_inputs_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.jsonl");
    let output = dir.path().join("timeline.csv");
    write_jsonl(
        &input,
        &[json!({"Artifact":"registry","Payload":"x".repeat(1024*1024+10)})],
    );
    bad(
        run(&[
            "-f",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--max-record-size",
            "1",
        ]),
        "max-record-size",
    );
    for row in [
        json!({"TimestampType":"x","Message":"x","SourceField":"x"}),
        json!({"timestamp_desc":"x","message":"x","SourceField":"x"}),
    ] {
        write_jsonl(&input, &[row]);
        bad(
            run(&[
                "-f",
                input.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ]),
            "already a normalized",
        );
    }
    assert!(!output.exists());
}
#[test]
fn standalone_input_needs_an_artifact_and_filter_names_are_validated() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("table.csv");
    let output = dir.path().join("timeline.csv");
    fs::write(&input, "LastRun\n2026-10-20T00:00:00Z\n").unwrap();
    bad(
        run(&[
            "-f",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ]),
        "needs --input-artifact",
    );
    bad(
        run(&[
            "-f",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--input-artifact",
            "not-an-artifact",
        ]),
        "Unknown --artifact",
    );
    ok(run(&[
        "-f",
        input.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--input-artifact",
        "prefetch",
        "--computer",
        "TEST",
    ]));
    assert_eq!(csv(&output)[0]["Computer"], "TEST");
}

#[test]
fn external_merge_is_deterministic_across_memory_and_worker_counts() {
    let dir = tempfile::tempdir().unwrap();
    let parsed = dir.path().join("parsed");
    let names: Vec<_> = (0..36).map(|i| format!("part-{i:02}.csv")).collect();
    let files: Vec<_> = names.iter().map(|n| (n.as_str(), "prefetch")).collect();
    manifest(&parsed, &files);
    for (i, name) in names.iter().enumerate() {
        let mut w = csv::Writer::from_path(parsed.join(name)).unwrap();
        w.write_record(["SourceFile", "ExecutableName", "LastRun"])
            .unwrap();
        for row in (0..400).rev() {
            w.write_record([
                format!("capture/{i}-{row}.pf"),
                format!("program-{row}.exe"),
                format!(
                    "2026-10-20T08:{:02}:{:02}.{:09}Z",
                    row % 60,
                    (row * 7) % 60,
                    row % 11
                ),
            ])
            .unwrap();
        }
        w.flush().unwrap();
    }
    let mut outputs = Vec::new();
    for (budget, threads) in [("8", "2"), ("64", "4")] {
        let output = dir.path().join(format!("timeline-{budget}.jsonl"));
        ok(run(&[
            "-d",
            parsed.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-t",
            "jsonl",
            "--memory-limit",
            budget,
            "-T",
            threads,
        ]));
        let summary: Value = serde_json::from_slice(
            &fs::read(dir.path().join(format!("timeline-{budget}-Summary.json"))).unwrap(),
        )
        .unwrap();
        assert!(summary["MergePasses"].as_u64().unwrap() > 0);
        outputs.push(output);
    }
    assert_eq!(
        fs::read(&outputs[0]).unwrap(),
        fs::read(&outputs[1]).unwrap()
    );
    let rows = jsonl(&outputs[0]);
    assert_eq!(rows.len(), 36 * 400);
    assert!(
        rows.windows(2)
            .all(|pair| pair[0]["Timestamp"].as_str() <= pair[1]["Timestamp"].as_str())
    );
}
#[test]
fn real_public_prefetch_exports_feed_the_timeline_in_csv_json_and_jsonl() {
    let dir = tempfile::tempdir().unwrap();
    let raw = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/prefetch/CMD.EXE-6D6290C5.pf");
    let mut counts = Vec::new();
    for format in ["csv", "json", "jsonl"] {
        let input = dir.path().join(format!("prefetch.{format}"));
        let result = Command::new(env!("CARGO_BIN_EXE_juntei"))
            .args([
                "parse-file",
                "prefetch",
                "-f",
                raw.to_str().unwrap(),
                "-t",
                format,
                "-o",
                input.to_str().unwrap(),
                "-q",
                "-K",
            ])
            .output()
            .unwrap();
        ok(result);
        let output = dir.path().join(format!("timeline-{format}.jsonl"));
        ok(run(&[
            "-f",
            input.to_str().unwrap(),
            "--input-artifact",
            "prefetch",
            "--computer",
            "PUBLIC-TEST",
            "-o",
            output.to_str().unwrap(),
            "-t",
            "jsonl",
        ]));
        let rows = jsonl(&output);
        assert!(!rows.is_empty());
        assert!(
            rows.iter()
                .all(|r| r["SourceFile"] == raw.to_str().unwrap())
        );
        counts.push(
            rows.iter()
                .filter(|r| r["Details"]["TimeRole"] == "execution")
                .map(|r| r["Timestamp"].clone())
                .collect::<Vec<_>>(),
        );
    }
    assert!(!counts[0].is_empty());
    assert_eq!(counts[0], counts[1]);
    assert_eq!(counts[1], counts[2]);
}
