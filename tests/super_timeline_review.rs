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

//! Regressions for portable references, report isolation, timestamp diagnostics and cancellation.
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_juntei"))
        .arg("super-timeline")
        .args(args)
        .args(["-q", "-K"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn jsonl(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn all_exported_provenance_is_relative_and_csv_jsonl_exports_coexist() {
    let tmp = tempfile::tempdir().unwrap();
    let physical = tmp.path().join("analyst-private-parent");
    fs::create_dir(&physical).unwrap();
    #[cfg(unix)]
    let case = {
        let alias = tmp.path().join("logical-analyst-private-parent");
        std::os::unix::fs::symlink(&physical, &alias).unwrap();
        alias
    };
    #[cfg(not(unix))]
    let case = physical;
    let evidence = case.join("Evidence");
    let parsed = case.join("Parsed");
    let reports = case.join("Reports");
    fs::create_dir_all(evidence.join("HOST/C/Windows/System32/sru")).unwrap();
    fs::write(evidence.join("HOST/C/Windows/System32/sru/SRUDB.dat"), []).unwrap();
    fs::create_dir_all(parsed.join("HOST/ProcessExecution")).unwrap();
    fs::create_dir_all(&reports).unwrap();
    fs::write(
        parsed.join("HOST/ProcessExecution/SRUM.csv"),
        "Timestamp,ID,BytesSent\n2026-10-20T08:00:00Z,1,42\n",
    )
    .unwrap();
    let mut prefetch =
        csv::Writer::from_path(parsed.join("HOST/ProcessExecution/Prefetch.csv")).unwrap();
    prefetch
        .write_record(["SourceFile", "ExecutablePath", "LastRun"])
        .unwrap();
    prefetch
        .write_record([
            evidence
                .join("HOST/C/Windows/Prefetch/APP.pf")
                .to_str()
                .unwrap(),
            r"C:\Windows\Temp\app.exe",
            "2026-10-20T08:01:00Z",
        ])
        .unwrap();
    prefetch.flush().unwrap();
    fs::write(
        parsed.join("HOST/ProcessExecution/Amcache.csv"),
        "Name,InstallDate,LinkDate\napp.exe,03-10-2022,corrupt\n",
    )
    .unwrap();
    let tasks: Vec<_> = [("srum", "SRUM.csv"), ("prefetch", "Prefetch.csv"), ("amcache", "Amcache.csv")].into_iter().map(|(artifact,file)|json!({
        "Status":"succeeded","Purpose":"artifact","Artifact":artifact,"Comp":"HOST","OutputType":"csv",
        "OutputFiles":[format!("./HOST/ProcessExecution/{file}")],"Command":["juntei", "parser", "-d", "./HOST/C"]
    })).collect();
    fs::write(parsed.join("manifest.json"),serde_json::to_vec(&json!({
        "ManifestVersion":2,"OutputDirectory":"Parsed","EvidenceDirectory":"Evidence",
        "Hosts":[{"Comp":"HOST","EvidenceRoot":"./HOST/C","WindowsDirectories":["./HOST/C/Windows"]}],"Tasks":tasks
    })).unwrap()).unwrap();
    let haya = case.join("alerts.jsonl");
    fs::write(&haya, format!("{}\n{}\n",json!({"Timestamp":"2026-10-20T08:02:00Z","Computer":"HOST","RuleTitle":"Empty path alert","EvtxFile":r"C:\private-analyst\old-case\Security.evtx", "Details":"Informational text"}),json!({"Timestamp":"2026-10-20T08:03:00Z","Computer":"HOST","RuleTitle":"Process path alert","EvtxFile":r"C:\private-analyst\old-case\Security.evtx", "Details":{"Image":r"C:\Windows\System32\cmd.exe"}}))).unwrap();
    let mut tables = Vec::new();
    for format in ["csv", "jsonl"] {
        let output = reports.join(format!("Timeline.{format}"));
        let result = run(&[
            "-d",
            parsed.to_str().unwrap(),
            "--hayabusa",
            haya.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-t",
            format,
        ]);
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("Unresolved time zones: 1"), "{stdout}");
        assert!(stdout.contains("Invalid timestamps: 1"), "{stdout}");
        for suffix in ["-Summary.json", "-Untimed.jsonl", "-Diagnostics.jsonl"] {
            assert!(!reports.join(format!("Timeline.{format}{suffix}")).exists());
        }
        let bytes = fs::read_to_string(&output).unwrap();
        assert!(!bytes.contains("analyst-private-parent"), "{bytes}");
        assert!(!bytes.contains("private-analyst"), "{bytes}");
        let rows = if format == "jsonl" {
            jsonl(&output)
        } else {
            let mut reader = csv::Reader::from_path(&output).unwrap();
            let headers = reader.headers().unwrap().clone();
            reader
                .records()
                .map(|record| {
                    let record = record.unwrap();
                    let mut object = serde_json::Map::new();
                    for (key, value) in headers.iter().zip(record.iter()) {
                        object.insert(
                            key.into(),
                            if key == "Details" {
                                serde_json::from_str(value).unwrap()
                            } else {
                                json!(value)
                            },
                        );
                    }
                    Value::Object(object)
                })
                .collect()
        };
        let srum = rows.iter().find(|r| r["Artifact"] == "srum").unwrap();
        assert_eq!(
            srum["SourceFile"],
            "./HOST/C/Windows/System32/sru/SRUDB.dat"
        );
        assert_eq!(
            srum["Details"]["ParsedFile"],
            "./HOST/ProcessExecution/SRUM.csv"
        );
        let prefetch = rows.iter().find(|r| r["Artifact"] == "prefetch").unwrap();
        assert_eq!(prefetch["Path"], r"C:\Windows\Temp\app.exe");
        assert_eq!(prefetch["SourceFile"], "./HOST/C/Windows/Prefetch/APP.pf");
        let alert = rows
            .iter()
            .find(|r| r["Details"]["RuleTitle"] == "Empty path alert")
            .unwrap();
        assert_eq!(alert["Message"], "Empty path alert");
        let process = rows
            .iter()
            .find(|r| r["Details"]["RuleTitle"] == "Process path alert")
            .unwrap();
        assert_eq!(
            process["Details"]["Details"]["Image"],
            r"C:\Windows\System32\cmd.exe"
        );
        assert!(
            alert["SourceFile"]
                .as_str()
                .unwrap()
                .starts_with("./Sources/")
        );
        assert_eq!(alert["Details"]["ParsedFile"], "./alerts.jsonl");
        tables.push(rows);
    }
    assert_eq!(tables[0], tables[1]);
}

#[cfg(unix)]
#[test]
fn signals_clean_normalization_and_merge_workspaces_and_preserve_existing_outputs() {
    use std::time::{Duration, Instant};
    let tmp = tempfile::tempdir().unwrap();
    let parsed = tmp.path().join("parsed");
    fs::create_dir(&parsed).unwrap();
    let mut tasks = Vec::new();
    for index in 0..36 {
        let name = format!("part-{index}.csv");
        let mut writer = csv::Writer::from_path(parsed.join(&name)).unwrap();
        writer
            .write_record(["SourceFile", "ExecutableName", "LastRun"])
            .unwrap();
        for record in 0..1000 {
            writer
                .write_record([
                    format!("capture/{index}-{record}.pf"),
                    format!("program-{record}.exe"),
                    "2026-10-20T08:00:00Z".into(),
                ])
                .unwrap();
        }
        for record in 0..1000 {
            writer
                .write_record([
                    format!("capture/untimed-{index}-{record}.pf"),
                    "untimed.exe".into(),
                    String::new(),
                ])
                .unwrap();
        }
        writer.flush().unwrap();
        tasks.push(json!({"Status":"succeeded","Purpose":"artifact","Artifact":"prefetch","Comp":"HOST","OutputType":"csv","OutputFiles":[name]}));
    }
    fs::write(
        parsed.join("manifest.json"),
        serde_json::to_vec(
            &json!({"OutputDirectory":"parsed","Hosts":[{"Comp":"HOST"}],"Tasks":tasks}),
        )
        .unwrap(),
    )
    .unwrap();
    for signal in ["-INT", "-TERM", "-HUP"] {
        for (sort, phase) in [
            (true, "normalize"),
            (true, "merge"),
            (true, "output"),
            (false, "normalize"),
            (false, "output"),
        ] {
            if phase == "output" && signal != "-INT" {
                continue;
            }
            let scratch = tmp.path().join(format!("scratch-{sort}-{signal}-{phase}"));
            fs::create_dir(&scratch).unwrap();
            let output = tmp
                .path()
                .join(format!("output-{sort}-{signal}-{phase}.jsonl"));
            let companions = [
                output.clone(),
                output.with_file_name(format!(
                    "{}-Summary.json",
                    output.file_name().unwrap().to_string_lossy()
                )),
                output.with_file_name(format!(
                    "{}-Untimed.jsonl",
                    output.file_name().unwrap().to_string_lossy()
                )),
                output.with_file_name(format!(
                    "{}-Diagnostics.jsonl",
                    output.file_name().unwrap().to_string_lossy()
                )),
            ];
            for path in &companions {
                fs::write(path, "previous complete result").unwrap();
            }
            let selected = if !sort && phase == "output" {
                // Concatenating small fragments can finish before an external `kill` starts
                // on fast machines. Use a substantial output for this copy-phase check.
                let large = tmp.path().join("large-copy-input");
                fs::create_dir(&large).unwrap();
                let mut writer = csv::Writer::from_path(large.join("Prefetch.csv")).unwrap();
                writer
                    .write_record(["SourceFile", "ExecutablePath", "LastRun"])
                    .unwrap();
                let path = format!("C:\\Temp\\{}\\program.exe", "a".repeat(4096));
                for index in 0..20_000 {
                    writer
                        .write_record([
                            format!("capture/{index}.pf"),
                            path.clone(),
                            "2026-10-20T08:00:00Z".into(),
                        ])
                        .unwrap();
                }
                writer.flush().unwrap();
                fs::write(large.join("manifest.json"), serde_json::to_vec(&json!({
                    "OutputDirectory":"large-copy-input","Hosts":[{"Comp":"HOST"}],
                    "Tasks":[{"Status":"succeeded","Artifact":"prefetch","Comp":"HOST","OutputType":"csv","OutputFiles":["Prefetch.csv"]}]
                })).unwrap()).unwrap();
                large
            } else {
                parsed.clone()
            };
            let mut child = Command::new(env!("CARGO_BIN_EXE_juntei"))
                .args([
                    "super-timeline",
                    "-d",
                    selected.to_str().unwrap(),
                    "-o",
                    output.to_str().unwrap(),
                    "-t",
                    "jsonl",
                    "--memory-limit",
                    "8",
                    "-T",
                    "2",
                    "--temp-dir",
                    scratch.to_str().unwrap(),
                    "-C",
                    "-q",
                    "-K",
                ])
                .args(sort.then_some("--sort"))
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            let start = Instant::now();
            loop {
                let ready = fs::read_dir(if phase == "output" {
                    tmp.path()
                } else {
                    &scratch
                })
                .unwrap()
                .filter_map(Result::ok)
                .any(|folder| {
                    if phase == "output"
                        && !folder
                            .file_name()
                            .to_string_lossy()
                            .starts_with(".juntei-super-timeline-")
                    {
                        return false;
                    }
                    fs::read_dir(folder.path()).is_ok_and(|files| {
                        files.filter_map(Result::ok).any(|file| {
                            let name = file.file_name().to_string_lossy().into_owned();
                            assert!(
                                !name.contains("diagnostic"),
                                "Unexpected diagnostic spool: {name}"
                            );
                            if phase == "output" {
                                name == "timeline" && file.metadata().is_ok_and(|m| m.len() > 0)
                            } else if phase == "merge" {
                                name.starts_with("merge-")
                            } else if sort {
                                name.contains("-chunk-")
                            } else {
                                name.ends_with("-events")
                                    && file.metadata().is_ok_and(|m| m.len() > 0)
                            }
                        })
                    })
                });
                if ready {
                    break;
                }
                if start.elapsed() > Duration::from_secs(30) || child.try_wait().unwrap().is_some()
                {
                    let _ = child.kill();
                    let output = child.wait_with_output().unwrap();
                    panic!(
                        "{phase} did not start: {} {}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            assert!(
                Command::new("kill")
                    .args([signal, &child.id().to_string()])
                    .status()
                    .unwrap()
                    .success()
            );
            let stopped = Instant::now();
            while child.try_wait().unwrap().is_none() {
                if stopped.elapsed() > Duration::from_secs(10) {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("{signal} did not stop {phase}");
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            let result = child.wait_with_output().unwrap();
            assert_eq!(
                result.status.code(),
                Some(130),
                "{} {}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
            assert!(String::from_utf8_lossy(&result.stdout).contains("Cancelled."));
            assert_eq!(fs::read_dir(&scratch).unwrap().count(), 0);
            for path in &companions {
                assert_eq!(
                    fs::read_to_string(path).unwrap(),
                    "previous complete result"
                );
            }
            assert!(
                !fs::read_dir(tmp.path())
                    .unwrap()
                    .filter_map(Result::ok)
                    .any(|e| e
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".juntei-super-timeline-"))
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn cancellation_during_json_deserialization_does_not_retry_interrupted_reads_forever() {
    use std::time::{Duration, Instant};
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("records.json");
    let record =
        r#"{"Artifact":"prefetch","SourceFile":"capture/APP.pf","LastRun":"2026-10-20T08:00:00Z"}"#;
    let mut data = String::from("[");
    for i in 0..120_000 {
        if i > 0 {
            data.push(',');
        }
        data.push_str(record);
    }
    data.push(']');
    fs::write(&input, data).unwrap();
    let scratch = dir.path().join("scratch");
    fs::create_dir(&scratch).unwrap();
    let output = dir.path().join("Timeline.csv");
    let mut child = Command::new(env!("CARGO_BIN_EXE_juntei"))
        .args([
            "super-timeline",
            "-f",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--temp-dir",
            scratch.to_str().unwrap(),
            "-q",
            "-K",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let start = Instant::now();
    loop {
        let reading = fs::read_dir(&scratch)
            .unwrap()
            .filter_map(Result::ok)
            .any(|e| e.path().join("input-0-events").is_file());
        if reading {
            break;
        }
        if start.elapsed() > Duration::from_secs(10) || child.try_wait().unwrap().is_some() {
            let _ = child.kill();
            let _ = child.wait();
            panic!("JSON reader did not start");
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    std::thread::sleep(Duration::from_millis(10));
    assert!(
        Command::new("kill")
            .args(["-INT", &child.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    let stopped = Instant::now();
    while child.try_wait().unwrap().is_none() {
        if stopped.elapsed() > Duration::from_secs(10) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("JSON reader kept retrying cancellation");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let result = child.wait_with_output().unwrap();
    assert_eq!(
        result.status.code(),
        Some(130),
        "{} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(fs::read_dir(&scratch).unwrap().count(), 0);
    assert!(!output.exists());
}

#[test]
fn ambiguous_evidence_roots_keep_distinct_absolute_captures_distinct() {
    let tmp = tempfile::tempdir().unwrap();
    let case = tmp.path().join("case");
    let parsed = case.join("exports/Parsed");
    fs::create_dir_all(&parsed).unwrap();
    let mut sources = Vec::new();
    for root in [case.join("Evidence"), case.join("exports/Evidence")] {
        fs::create_dir_all(root.join("HOST/C/Windows")).unwrap();
        let source = root.join("HOST/C/Windows/APP.pf");
        fs::write(&source, []).unwrap();
        sources.push(source);
    }
    let mut writer = csv::Writer::from_path(parsed.join("Prefetch.csv")).unwrap();
    writer
        .write_record(["SourceFile", "ExecutableName", "LastRun"])
        .unwrap();
    for source in &sources {
        writer
            .write_record([source.to_str().unwrap(), "APP.EXE", "2026-10-20T08:00:00Z"])
            .unwrap();
    }
    writer.flush().unwrap();
    fs::write(parsed.join("manifest.json"), serde_json::to_vec(&json!({
        "OutputDirectory":"Parsed","EvidenceDirectory":"Evidence",
        "Hosts":[{"Comp":"HOST","EvidenceRoot":"./HOST/C","WindowsDirectories":["./HOST/C/Windows"]}],
        "Tasks":[{"Artifact":"prefetch","Comp":"HOST","Status":"succeeded","OutputType":"csv","OutputFiles":["Prefetch.csv"]}]
    })).unwrap()).unwrap();
    let output = tmp.path().join("Timeline.jsonl");
    run(&[
        "-d",
        parsed.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    let rows = jsonl(&output);
    assert_eq!(rows.len(), 2);
    assert_ne!(rows[0]["SourceFile"], rows[1]["SourceFile"]);
    assert!(
        rows.iter()
            .all(|r| r["SourceFile"].as_str().unwrap().starts_with("./Sources/"))
    );
}
