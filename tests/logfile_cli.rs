//! CLI regressions using public Windows/NTFS evidence, with licenses next to the fixtures.
use flate2::read::GzDecoder;
use serde_json::Value;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Output};

fn unpack(name: &str, destination: &Path) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files/logfile")
        .join(name);
    let mut data = Vec::new();
    GzDecoder::new(std::fs::File::open(source).unwrap())
        .read_to_end(&mut data)
        .unwrap();
    std::fs::write(destination, data).unwrap();
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_juntei"))
        .args(args)
        .output()
        .unwrap()
}

fn rows(output: &Output) -> Vec<Value> {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout.clone())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn real_log_all_formats_agree_and_preserve_redo_and_undo() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("$LogFile");
    unpack("go-ntfs/LogFile.gz", &input);
    unpack("go-ntfs/MFT.gz", &dir.path().join("$MFT"));
    let args = [
        "parse-ntfs",
        "logfile",
        "-f",
        input.to_str().unwrap(),
        "-t",
        "jsonl",
        "--raw-data",
    ];
    let values = rows(&run(&args));
    assert_eq!(values.len(), 2129);
    assert!(
        values
            .windows(2)
            .all(|w| w[0]["Lsn"].as_u64() < w[1]["Lsn"].as_u64())
    );
    assert_eq!(
        values
            .iter()
            .filter(|v| v["RecordType"] == "ClientRestart")
            .count(),
        17
    );
    assert_eq!(
        values
            .iter()
            .map(|v| v["RedoFileNames"].as_array().map_or(0, Vec::len)
                + v["UndoFileNames"].as_array().map_or(0, Vec::len))
            .sum::<usize>(),
        79
    );
    let quota = values.iter().find(|v| v["Lsn"] == 1050704_u64).unwrap();
    assert_eq!(quota["FileName"], "$Quota");
    assert_eq!(quota["MetadataSide"], "Undo");
    assert_eq!(quota["ParentPath"], ".");
    assert_eq!(quota["CreatedOn"], "2018-09-24T07:53:17.2748380Z");
    assert_eq!(quota["UndoFileNames"][0]["Name"], "$Quota");
    // The directory whose inline index changes is not the child described by its metadata.
    let directory = values.iter().find(|v| v["Lsn"] == 1058043_u64).unwrap();
    assert_eq!(directory["TargetEntryNumber"], 34);
    assert!(directory["TargetSequenceNumber"].is_null());
    assert!(directory["TargetPath"].is_null());
    assert!(directory["EntryNumber"].is_null());
    assert_eq!(directory["OpenAttributeEntryNumber"], 0);
    assert_eq!(directory["OpenAttributePath"], r".\$MFT");
    assert_eq!(directory["MetadataSide"], "Redo");
    assert_eq!(directory["FileSize"], 12);
    assert_eq!(directory["FileAttributes"], 2080);
    assert_eq!(directory["CreatedOn"], "2018-09-24T07:53:20.0721062Z");
    assert_eq!(
        directory["ContentModifiedOn"],
        "2018-09-24T07:53:20.0877325Z"
    );
    let data = values.iter().find(|v| v["Lsn"] == 1052517_u64).unwrap();
    assert_eq!(data["FileSize"], 8);
    assert_eq!(data["AttributeType"], 128);
    for lsn in [1051174_u64, 1051191] {
        let update = values.iter().find(|v| v["Lsn"] == lsn).unwrap();
        assert!(
            update["EntryNumber"].is_null(),
            "LSN {lsn}: target is not a decoded file identity"
        );
    }
    let partial_times = values.iter().find(|v| v["Lsn"] == 1053039_u64).unwrap();
    assert_eq!(partial_times["EntryNumber"], 0);
    assert_eq!(partial_times["AttributeType"], 16);
    assert!(
        quota["UndoFileNames"][0]["Created"]["Filetime"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(values.iter().any(|v| !v["RedoTimes"].is_null()
        && !v["UndoTimes"].is_null()
        && v["RedoTimes"] != v["UndoTimes"]));
    for v in &values {
        if v["RecordType"] == "ClientRecord" {
            assert_eq!(
                v["RedoData"].as_str().unwrap().len(),
                v["RedoBytesAvailable"].as_u64().unwrap() as usize * 2
            );
            assert_eq!(
                v["UndoData"].as_str().unwrap().len(),
                v["UndoBytesAvailable"].as_u64().unwrap() as usize * 2
            );
        }
    }
    let pretty = run(&[
        "parse-ntfs",
        "logfile",
        "-f",
        input.to_str().unwrap(),
        "-t",
        "json",
        "--raw-data",
    ]);
    assert!(pretty.status.success());
    let pretty: Vec<Value> = serde_json::Deserializer::from_slice(&pretty.stdout)
        .into_iter()
        .map(Result::unwrap)
        .collect();
    assert_eq!(pretty, values);
    let csv = run(&[
        "parse-ntfs",
        "logfile",
        "-f",
        input.to_str().unwrap(),
        "-t",
        "csv",
        "--raw-data",
    ]);
    assert!(csv.status.success());
    let mut reader = csv::Reader::from_reader(&csv.stdout[..]);
    let header = reader.headers().unwrap().clone();
    let records: Vec<_> = reader.records().map(Result::unwrap).collect();
    assert_eq!(records.len(), values.len());
    for (row, value) in records.iter().zip(&values) {
        for (field, cell) in header.iter().zip(row) {
            let expected = match &value[field] {
                Value::Null => String::new(),
                Value::String(s) => s.clone(),
                v => v.to_string(),
            };
            assert_eq!(cell, expected, "LSN {} field {field}", value["Lsn"]);
        }
    }
}

#[test]
fn version_two_log_exposes_journal_writes_and_damage() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("$LogFile");
    unpack("museum-win11/LogFile.gz", &input);
    let output = run(&[
        "parse-ntfs",
        "logfile",
        "-f",
        input.to_str().unwrap(),
        "-t",
        "jsonl",
        "--raw-data",
    ]);
    let values = rows(&output);
    assert!(values.iter().all(|v| v["LogVersion"] == "2.0"));
    assert!(
        values
            .iter()
            .any(|v| v["JournalWriteLength"].as_u64().unwrap_or(0) > 0)
    );
    assert!(values.iter().all(|v| v["MftSource"].is_null()));
    let mut data = std::fs::read(&input).unwrap();
    data[..4].copy_from_slice(b"BAD!");
    std::fs::write(&input, data).unwrap();
    let damaged = run(&[
        "parse-ntfs",
        "logfile",
        "-f",
        input.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    let salvaged = rows(&damaged);
    assert_eq!(salvaged.len(), values.len());
    assert!(
        !damaged.stderr.is_empty(),
        "damage must be reported even for stdout export"
    );
    assert!(salvaged.iter().all(|v| v["RedoData"].is_null()));
}

#[test]
fn current_paths_do_not_follow_a_reused_mft_record_number() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("$LogFile");
    let mft = dir.path().join("$MFT");
    unpack("go-ntfs/LogFile.gz", &input);
    unpack("go-ntfs/MFT.gz", &mft);
    // $Extend is entry 11, sequence 11 in this volume. Simulate its reuse.
    let mut bytes = std::fs::read(&mft).unwrap();
    bytes[11 * 1024 + 0x10..11 * 1024 + 0x12].copy_from_slice(&12_u16.to_le_bytes());
    std::fs::write(&mft, bytes).unwrap();
    let values = rows(&run(&[
        "parse-ntfs",
        "logfile",
        "-f",
        input.to_str().unwrap(),
        "-t",
        "jsonl",
    ]));
    let quota = values.iter().find(|v| v["Lsn"] == 1051364_u64).unwrap();
    assert_eq!(quota["ParentEntryNumber"], 11);
    assert_eq!(quota["ParentSequenceNumber"], 11);
    assert_eq!(quota["ParentPath"], r".\[Unknown 11-11]");
}

// APFS rejects illegal UTF-8 before Juntei can read it. The abstract serialization is
// separately tested on every Unix target in logfile::tests.
#[cfg(target_os = "linux")]
#[test]
fn non_utf8_source_paths_do_not_panic() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let dir = tempfile::tempdir().unwrap();
    // Discovery can encounter non-UTF-8 ancestors even when -d itself is UTF-8.
    let nested = dir.path().join(OsString::from_vec(b"volume-\xff".to_vec()));
    std::fs::create_dir(&nested).unwrap();
    unpack("go-ntfs/LogFile.gz", &nested.join("$LogFile"));
    unpack("go-ntfs/MFT.gz", &nested.join("$MFT"));
    let output = run(&[
        "parse-ntfs",
        "logfile",
        "-d",
        dir.path().to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    let values = rows(&output);
    assert_eq!(values.len(), 2129);
    assert!(
        values[0]["SourceFile"]
            .as_str()
            .unwrap()
            .contains('\u{fffd}')
    );
    assert!(
        values[0]["MftSource"]
            .as_str()
            .unwrap()
            .contains('\u{fffd}')
    );
}

#[test]
fn discovery_combines_sources_and_distinguishes_unused_from_invalid() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    std::fs::create_dir(&a).unwrap();
    std::fs::create_dir(&b).unwrap();
    unpack("go-ntfs/LogFile.gz", &a.join("$LogFile"));
    unpack("go-ntfs/LogFile.gz", &b.join("c_$logfile.bin"));
    std::fs::write(dir.path().join("LogFile.txt"), b"unrelated").unwrap();
    let values = rows(&run(&[
        "parse-ntfs",
        "logfile",
        "-d",
        dir.path().to_str().unwrap(),
        "-t",
        "jsonl",
    ]));
    assert_eq!(values.len(), 4258);
    let sources: std::collections::HashSet<_> = values
        .iter()
        .map(|v| v["SourceFile"].as_str().unwrap())
        .collect();
    assert_eq!(sources.len(), 2);
    let unused = dir.path().join("unused");
    std::fs::write(&unused, [0xff; 65536]).unwrap();
    assert!(
        rows(&run(&[
            "parse-ntfs",
            "logfile",
            "-f",
            unused.to_str().unwrap(),
            "-t",
            "jsonl"
        ]))
        .is_empty()
    );
    for data in [b"not an NTFS log".as_slice(), b""] {
        std::fs::write(&unused, data).unwrap();
        assert!(
            !run(&[
                "parse-ntfs",
                "logfile",
                "-f",
                unused.to_str().unwrap(),
                "-t",
                "jsonl"
            ])
            .status
            .success()
        );
    }
    let empty = tempfile::tempdir().unwrap();
    let output = run(&[
        "parse-ntfs",
        "logfile",
        "-d",
        empty.path().to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("No $LogFile files were found"));
}

#[test]
fn prefixed_logs_pair_only_with_their_own_mft() {
    let dir = tempfile::tempdir().unwrap();
    let c_log = dir.path().join("C_$LogFile");
    let c_mft = dir.path().join("C_$MFT");
    let d_log = dir.path().join("D_$LogFile.bin");
    unpack("go-ntfs/LogFile.gz", &c_log);
    unpack("go-ntfs/MFT.gz", &c_mft);
    unpack("go-ntfs/LogFile.gz", &d_log);
    // A plain MFT must not be used for a differently prefixed volume.
    unpack("go-ntfs/MFT.gz", &dir.path().join("$MFT"));
    let output = run(&[
        "parse-ntfs",
        "logfile",
        "-d",
        dir.path().to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    let values = rows(&output);
    assert_eq!(values.len(), 4258);
    let quota = |source: &Path| {
        values
            .iter()
            .find(|v| {
                v["Lsn"] == 1051364_u64 && v["SourceFile"] == source.to_string_lossy().as_ref()
            })
            .unwrap()
    };
    assert_eq!(quota(&c_log)["MftSource"], c_mft.to_string_lossy().as_ref());
    assert_eq!(quota(&c_log)["ParentPath"], r".\$Extend");
    assert!(quota(&d_log)["MftSource"].is_null());
    assert!(quota(&d_log)["ParentPath"].is_null());
    assert!(String::from_utf8_lossy(&output.stderr).contains("No unambiguous matching $MFT for"));
    let d_mft = dir.path().join("d_$mft.BIN");
    unpack("go-ntfs/MFT.gz", &d_mft);
    let mut bytes = std::fs::read(&d_mft).unwrap();
    bytes[11 * 1024 + 0x10..11 * 1024 + 0x12].copy_from_slice(&12_u16.to_le_bytes());
    std::fs::write(&d_mft, bytes).unwrap();
    let values = rows(&run(&[
        "parse-ntfs",
        "logfile",
        "-d",
        dir.path().to_str().unwrap(),
        "-t",
        "jsonl",
    ]));
    let quota = |source: &Path| {
        values
            .iter()
            .find(|v| {
                v["Lsn"] == 1051364_u64 && v["SourceFile"] == source.to_string_lossy().as_ref()
            })
            .unwrap()
    };
    assert_eq!(quota(&c_log)["ParentPath"], r".\$Extend");
    assert_eq!(quota(&d_log)["MftSource"], d_mft.to_string_lossy().as_ref());
    assert_eq!(quota(&d_log)["ParentPath"], r".\[Unknown 11-11]");
    let before = std::fs::read(&c_mft).unwrap();
    assert!(
        !run(&[
            "parse-ntfs",
            "logfile",
            "-f",
            c_log.to_str().unwrap(),
            "-o",
            c_mft.to_str().unwrap(),
            "-C"
        ])
        .status
        .success()
    );
    assert_eq!(std::fs::read(&c_mft).unwrap(), before);
}

#[test]
fn protects_input_mft_and_existing_outputs() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("$LogFile");
    let mft = dir.path().join("$MFT");
    unpack("go-ntfs/LogFile.gz", &input);
    unpack("go-ntfs/MFT.gz", &mft);
    for out in [&input, &mft] {
        let before = std::fs::read(out).unwrap();
        assert!(
            !run(&[
                "parse-ntfs",
                "logfile",
                "-f",
                input.to_str().unwrap(),
                "-o",
                out.to_str().unwrap(),
                "-C"
            ])
            .status
            .success()
        );
        assert_eq!(std::fs::read(out).unwrap(), before);
    }
    let alias = dir.path().join("alias");
    std::fs::hard_link(&mft, &alias).unwrap();
    assert!(
        !run(&[
            "parse-ntfs",
            "logfile",
            "-f",
            input.to_str().unwrap(),
            "-o",
            alias.to_str().unwrap(),
            "-C"
        ])
        .status
        .success()
    );
    let out = dir.path().join("out.csv");
    std::fs::write(&out, b"keep this").unwrap();
    assert!(
        !run(&[
            "parse-ntfs",
            "logfile",
            "-f",
            input.to_str().unwrap(),
            "-o",
            out.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert_eq!(std::fs::read(&out).unwrap(), b"keep this");
    assert!(
        run(&[
            "parse-ntfs",
            "logfile",
            "-f",
            input.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "-C"
        ])
        .status
        .success()
    );
    assert!(
        !run(&[
            "parse-ntfs",
            "logfile",
            "-f",
            input.to_str().unwrap(),
            "--cluster-size",
            "513"
        ])
        .status
        .success()
    );
    let override_values = rows(&run(&[
        "parse-ntfs",
        "logfile",
        "-f",
        input.to_str().unwrap(),
        "--cluster-size",
        "4096",
        "--record-size",
        "1024",
        "-t",
        "jsonl",
    ]));
    assert!(
        override_values
            .iter()
            .all(|v| v["ClusterSize"] == 4096 && v["ClusterSizeSource"] == "Caller")
    );
    // Broken auxiliary evidence must not discard valid log records or pretend paths loaded.
    std::fs::write(&mft, b"not a valid MFT").unwrap();
    let output = run(&[
        "parse-ntfs",
        "logfile",
        "-f",
        input.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    let salvaged = rows(&output);
    assert_eq!(salvaged.len(), 2129);
    assert!(salvaged.iter().all(|v| v["MftSource"].is_null()));
    assert!(String::from_utf8_lossy(&output.stderr).contains("without current paths"));
}

#[test]
fn auto_parse_outputs_and_jsonl_timeline_include_logfile_and_provenance() {
    let evidence = tempfile::tempdir().unwrap();
    let host = evidence.path().join("HOST1");
    std::fs::create_dir_all(host.join("C/Windows/System32")).unwrap();
    std::fs::create_dir_all(host.join("D")).unwrap();
    std::fs::write(host.join("client_info.json"), r#"{"hostname":"HOST1"}"#).unwrap();
    unpack("go-ntfs/LogFile.gz", &host.join("C/$LogFile"));
    // Both collected volumes must survive as separate outputs.
    unpack("go-ntfs/LogFile.gz", &host.join("D/$LogFile"));
    let root = tempfile::tempdir().unwrap();
    for format in ["csv", "json", "jsonl"] {
        let out = root.path().join(format);
        let combined = out.join("timeline.jsonl");
        let args = vec![
            "auto-parse",
            "-d",
            evidence.path().to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "-t",
            format,
            "-j",
            combined.to_str().unwrap(),
            "-T",
            "2",
            "-K",
            "-q",
        ];
        let output = run(&args);
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let manifest: Value =
            serde_json::from_slice(&std::fs::read(out.join("manifest.json")).unwrap()).unwrap();
        let tasks: Vec<_> = manifest["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|t| t["artifact"] == "logfile" && t["purpose"] == "artifact")
            .collect();
        assert_eq!(tasks.len(), 2);
        assert!(tasks.iter().all(|t| t["status"] == "succeeded"));
        assert!(tasks.iter().all(|t| t["category"] == "file_system"));
        assert!(out.join("HOST1/file_system/logs/logfile.txt").is_file());
        assert!(out.join("HOST1/file_system/logs/logfile-2.txt").is_file());
        if format == "csv" {
            for name in ["logfile.csv", "logfile-2.csv"] {
                assert_eq!(
                    csv::Reader::from_path(out.join("HOST1/file_system").join(name))
                        .unwrap()
                        .records()
                        .count(),
                    2129
                );
            }
        }
        {
            let data = std::fs::read_to_string(combined).unwrap();
            let values: Vec<Value> = data
                .lines()
                .map(|l| serde_json::from_str(l).unwrap())
                .collect();
            assert_eq!(values.len(), 4258);
            assert!(
                values
                    .iter()
                    .all(|v| v["Computer"] == "HOST1" && v["Artifact"] == "logfile")
            );
        }
    }
}
