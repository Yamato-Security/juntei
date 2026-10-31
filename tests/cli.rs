//! End-to-end tests: run the built binary against the test hives in `test_files/amcache/`.
//!
//! The expected counts are those of the `ys-amcache-parser` command, which was cross-validated
//! against regipy and a second, independent Amcache parser (see `test_files/amcache/README.md`).

use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

const WIN10: &str = "test_files/amcache/amcache_win10.hve";
const LEGACY: &str = "test_files/amcache/amcache_2012r2_old.hve";

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Run {
    run_at(Path::new(env!("CARGO_MANIFEST_DIR")), args)
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

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// A triage-like tree: two hosts, a transaction log and a hive that is not a hive.
fn triage() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let programs = dir.path().join("HOST1/C/Windows/AppCompat/Programs");
    std::fs::create_dir_all(&programs).unwrap();
    std::fs::copy(repo(WIN10), programs.join("Amcache.hve")).unwrap();
    std::fs::write(programs.join("Amcache.hve.LOG1"), b"log").unwrap();
    std::fs::create_dir_all(dir.path().join("HOST2")).unwrap();
    std::fs::copy(repo(LEGACY), dir.path().join("HOST2/Amcache.hve")).unwrap();
    std::fs::write(dir.path().join("HOST2/Broken_Amcache.hve"), b"not a hive").unwrap();
    dir
}

/// A minimal Velociraptor-like collection used to exercise `auto-parse` end to end.
fn auto_triage() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let host = dir.path().join("HOST1_F.TEST");
    let windows = host.join("uploads/auto/C%3A/Windows");
    let programs = windows.join("AppCompat/Programs");
    let prefetch = windows.join("Prefetch");
    let srum = windows.join("System32/sru");
    let ntfs = host.join("uploads/ntfs/C");
    std::fs::create_dir_all(&programs).unwrap();
    std::fs::create_dir_all(&prefetch).unwrap();
    std::fs::create_dir_all(windows.join("System32")).unwrap();
    std::fs::create_dir_all(&srum).unwrap();
    std::fs::create_dir_all(ntfs.join("$Extend")).unwrap();
    std::fs::write(
        host.join("client_info.json"),
        r#"{"hostname":"HOST1","fqdn":"host1.example.test"}"#,
    )
    .unwrap();
    std::fs::copy(repo(WIN10), programs.join("Amcache.hve")).unwrap();
    std::fs::copy(
        repo("test_files/prefetch/CMD.EXE-6D6290C5.pf"),
        prefetch.join("CMD.EXE-6D6290C5.pf"),
    )
    .unwrap();
    std::fs::copy(repo("test_files/mft/MFT"), ntfs.join("$MFT")).unwrap();
    std::fs::copy(
        repo("test_files/usn/win11_rathbunvm_$J"),
        ntfs.join("$Extend/$J"),
    )
    .unwrap();
    std::fs::copy(
        repo("test_files/srum/win10_rathbunvm_SRUDB.dat"),
        srum.join("SRUDB.dat"),
    )
    .unwrap();
    dir
}

/// Every JSON document in a stream of newline-separated (pretty) documents.
fn json_documents(text: &str) -> Vec<Value> {
    serde_json::Deserializer::from_str(text)
        .into_iter::<Value>()
        .map(|v| v.unwrap())
        .collect()
}

#[test]
fn banner_and_help() {
    let r = run(&[]);
    assert_eq!(r.code, 0);
    assert!(
        r.stdout.contains("┗━━┻━━━┻┛ ┗━┛ ┗┛ ┗━━━┻━━┛"),
        "{}",
        r.stdout
    );
    assert!(r.stdout.contains("by Yamato Security"));
    assert!(r.stdout.contains("parse-registry"));
    let r = run(&["parse-registry", "amcache", "-h", "-K"]);
    assert_eq!(r.code, 0);
    assert!(
        r.stdout
            .contains("juntei parse-registry amcache <INPUT> [OPTIONS]"),
        "{}",
        r.stdout
    );
}

#[test]
fn source_help_lists_artifacts_and_leaf_help_shows_the_full_command() {
    let assert_alphabetical = |help: &str| {
        let commands: Vec<_> = help
            .lines()
            .skip_while(|line| *line != "Commands:")
            .skip(1)
            .take_while(|line| !line.is_empty())
            .map(|line| line.split_whitespace().next().unwrap())
            .collect();
        assert!(!commands.is_empty(), "{help}");
        let mut sorted = commands.clone();
        sorted.sort_unstable();
        assert_eq!(commands, sorted, "{help}");
    };
    for (source, description, artifacts) in [
        (
            "parse-ntfs",
            "Parse NTFS artifacts",
            &["mft", "usn-journal", "logfile", "i30"][..],
        ),
        (
            "parse-registry",
            "Parse registry hives",
            &[
                "amcache",
                "shimcache",
                "read",
                "services",
                "recentdocs",
                "taskcache",
                "sam",
                "persistence",
                "userassist",
            ][..],
        ),
        ("parse-ese", "Parse ESE databases", &["srum"][..]),
        (
            "parse-file",
            "Parse file artifacts",
            &["prefetch", "lnk", "jump-lists", "defender-quarantine"][..],
        ),
    ] {
        let top = run(&["--help"]);
        assert_eq!(top.code, 0);
        assert!(top.stdout.contains(source) && top.stdout.contains(description));
        assert_alphabetical(&top.stdout);

        for help in ["-h", "--help"] {
            let group = run(&[source, help]);
            assert_eq!(group.code, 0, "{}", group.stderr);
            assert!(group.stdout.contains(&format!("juntei {source} <COMMAND>")));
            assert_alphabetical(&group.stdout);
            for artifact in artifacts {
                assert!(group.stdout.contains(artifact), "{}", group.stdout);
                let leaf = run(&[source, artifact, help]);
                assert_eq!(leaf.code, 0, "{}", leaf.stderr);
                assert!(
                    leaf.stdout
                        .contains(&format!("juntei {source} {artifact} <INPUT> [OPTIONS]")),
                    "{}",
                    leaf.stdout
                );
                assert!(leaf.stdout.contains("--output-type") && leaf.stdout.contains("--file"));
            }
        }
        let help_subcommand = run(&[source, "help", artifacts[0]]);
        assert_eq!(help_subcommand.code, 0, "{}", help_subcommand.stderr);
        assert!(
            help_subcommand
                .stdout
                .contains(&format!("juntei {source} {}", artifacts[0]))
        );
    }
}

#[test]
fn auto_parse_creates_host_csvs_and_one_siem_ready_jsonl() {
    let evidence = auto_triage();
    let output_root = tempfile::tempdir().unwrap();
    let csv = output_root.path().join("csv");
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "-o",
        csv.to_str().unwrap(),
        "-t",
        "csv",
        "-T",
        "2",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    assert!(
        r.stdout.contains(
            "Parsing in progress. Per-parser messages are saved in *-logs.txt files beside the results.\n\n"
        ),
        "{}",
        r.stdout
    );
    assert!(csv.join("HOST1/file_existence/amcache").is_dir());
    assert!(
        csv.join("HOST1/file_existence/amcache/Amcache_summary.csv")
            .is_file()
    );
    assert!(
        csv.join("HOST1/file_existence/amcache/Amcache_files.csv")
            .is_file()
    );
    assert!(
        std::fs::read_dir(csv.join("HOST1/file_existence/amcache"))
            .unwrap()
            .flatten()
            .all(|entry| entry.path().is_file())
    );
    assert!(csv.join("HOST1/process_execution/srum").is_dir());
    assert!(
        csv.join("HOST1/process_execution/srum/IdentityMap.csv")
            .is_file()
    );
    assert!(
        csv.join("HOST1/process_execution/srum/VfuProvider.csv")
            .is_file()
    );
    assert!(
        std::fs::read_dir(csv.join("HOST1/process_execution/srum"))
            .unwrap()
            .flatten()
            .all(|entry| entry.path().is_file())
    );
    assert!(csv.join("HOST1/file_system/mft.csv").is_file());
    assert!(csv.join("HOST1/file_system/usn-journal.csv").is_file());
    assert!(!csv.join("HOST1/file_system/mft").exists());
    assert!(!csv.join("HOST1/file_system/usn-journal").exists());
    assert!(csv.join("HOST1/process_execution/prefetch.csv").is_file());
    assert!(
        csv.join("HOST1/process_execution/prefetch-timeline.csv")
            .is_file()
    );
    assert!(
        csv.join("HOST1/process_execution/prefetch-logs.txt")
            .is_file()
    );
    assert!(csv.join("HOST1/file_system/mft-logs.txt").is_file());
    assert!(!csv.join("HOST1/logs").exists());
    for category in [
        "accounts",
        "file_and_folder_opening",
        "file_existence",
        "file_system",
        "persistence",
        "process_execution",
        "registry",
        "security_products",
    ] {
        assert!(csv.join("HOST1").join(category).is_dir());
        assert!(!csv.join("HOST1").join(category).join("logs").exists());
    }
    assert!(!csv.join("HOST1/_logs").exists());
    let manifest: Value =
        serde_json::from_reader(std::fs::File::open(csv.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["hosts"][0]["Computer"], "HOST1");
    assert_eq!(manifest["hosts"][0]["hostname"], "HOST1");
    assert_eq!(manifest["hosts"][0]["hostname_source"], "client_info.json");
    assert!(
        manifest["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|task| task["status"] != "failed")
    );
    assert!(manifest["tasks"].as_array().unwrap().iter().all(|task| {
        task["log"].as_str().is_some_and(|path| {
            let category = task["category"].as_str().unwrap();
            Path::new(path).parent() == Some(csv.join("HOST1").join(category).as_path())
                && path.ends_with("-logs.txt")
                && Path::new(path).is_file()
        })
    }));
    assert_eq!(manifest["tasks"][0]["artifact"], "mft");
    assert_eq!(manifest["tasks"][1]["artifact"], "usn-journal");
    for task in manifest["tasks"].as_array().unwrap() {
        let artifact = task["artifact"].as_str().unwrap();
        let category = match artifact {
            "sam" => "accounts",
            "jump-lists" | "lnk" | "recentdocs" => "file_and_folder_opening",
            "amcache" | "shimcache" => "file_existence",
            "i30" | "logfile" | "mft" | "usn-journal" => "file_system",
            "persistence" | "services" | "taskcache" => "persistence",
            "prefetch" | "prefetch-timeline" | "srum" | "userassist" => "process_execution",
            "registry" => "registry",
            "defender-quarantine" => "security_products",
            _ => panic!("unexpected artifact: {artifact}"),
        };
        assert_eq!(task["category"], category);
        if let Some(output) = task["output"].as_str() {
            assert!(Path::new(output).starts_with(csv.join("HOST1").join(category)));
        }
        let (source, command) = match artifact {
            "mft" | "usn-journal" | "logfile" | "i30" => ("parse-ntfs", artifact),
            "srum" => ("parse-ese", artifact),
            "prefetch-timeline" => ("parse-file", "prefetch"),
            "prefetch" | "lnk" | "jump-lists" | "defender-quarantine" => ("parse-file", artifact),
            "registry" => ("parse-registry", "read"),
            _ => ("parse-registry", artifact),
        };
        assert_eq!(task["command"][1], source, "{task}");
        assert_eq!(task["command"][2], command, "{task}");
    }
    let (_, csv_tasks) = csv_rows(&csv.join("manifest.csv"));
    assert_eq!(csv_tasks.len(), manifest["tasks"].as_array().unwrap().len());
    for (row, task) in csv_tasks.iter().zip(manifest["tasks"].as_array().unwrap()) {
        assert_eq!(row["Computer"], task["Computer"].as_str().unwrap());
        assert_eq!(row["Hostname"], "HOST1");
        assert_eq!(row["Category"], task["category"].as_str().unwrap());
        assert_eq!(row["Status"], task["status"].as_str().unwrap());
        assert_eq!(row["Log"], task["log"].as_str().unwrap());
        assert_eq!(
            serde_json::from_str::<Value>(&row["Command"]).unwrap(),
            task["command"]
        );
    }
    assert!(!r.stdout.contains("Manifest: "), "{}", r.stdout);
    let summary = r
        .stdout
        .lines()
        .find(|line| line.starts_with("Results saved: "))
        .unwrap();
    assert!(
        summary.starts_with(&format!("Results saved: {}/ ( ", csv.display())),
        "{summary}"
    );
    assert!(
        summary.contains(" files / ") && summary.ends_with(" MB )"),
        "{summary}"
    );

    // A previously generated target must not survive when the corresponding artifact is absent.
    let stale = csv.join("HOST1/file_system/i30.csv");
    std::fs::write(&stale, b"stale result\n").unwrap();
    let notes = csv.join("HOST1/file_system/notes.txt");
    std::fs::write(&notes, b"examiner notes\n").unwrap();
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "-o",
        csv.to_str().unwrap(),
        "-t",
        "csv",
        "-T",
        "2",
        "-C",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    assert!(!stale.exists());
    assert_eq!(std::fs::read(notes).unwrap(), b"examiner notes\n");

    // A Windows directory is a valid scope on its own; parsers must not receive its parent.
    let windows = evidence
        .path()
        .join("HOST1_F.TEST/uploads/auto/C%3A/Windows");
    let direct = output_root.path().join("direct-windows");
    let r = run_at(
        windows.parent().unwrap(),
        &[
            "auto-parse",
            "-d",
            "Windows",
            "-o",
            direct.to_str().unwrap(),
            "-t",
            "csv",
            "-T",
            "2",
            "-K",
            "-q",
        ],
    );
    assert_eq!(r.code, 0, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    let direct_manifest: Value =
        serde_json::from_reader(std::fs::File::open(direct.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(
        direct_manifest["hosts"][0]["evidence_root"],
        std::fs::canonicalize(&windows)
            .unwrap()
            .to_string_lossy()
            .as_ref()
    );
    assert!(
        direct
            .join("Windows/process_execution/prefetch.csv")
            .is_file()
    );

    let jsonl = output_root.path().join("jsonl");
    let combined = jsonl.join("siem.jsonl");
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "-o",
        jsonl.to_str().unwrap(),
        "-t",
        "jsonl",
        "-j",
        combined.to_str().unwrap(),
        "-T",
        "2",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    assert!(jsonl.join("manifest.csv").is_file());
    let values: Vec<Value> = std::fs::read_to_string(&combined)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(!values.is_empty());
    assert!(values.iter().all(|value| value["Computer"] == "HOST1"));
    assert!(values.iter().all(|value| value.get("Artifact").is_some()));
    assert!(values.iter().any(|value| value["Artifact"] == "amcache"));
    assert!(values.iter().any(|value| value["Artifact"] == "prefetch"));
    assert!(
        jsonl
            .join("HOST1/process_execution/prefetch-logs.txt")
            .is_file()
    );
    assert!(
        jsonl
            .join("HOST1/file_existence/amcache-logs.txt")
            .is_file()
    );
    assert!(
        jsonl
            .join("HOST1/process_execution/prefetch.jsonl")
            .is_file()
    );
    assert!(std::fs::read_dir(&jsonl).unwrap().flatten().all(|entry| {
        !entry
            .file_name()
            .to_string_lossy()
            .starts_with(".juntei-work-")
    }));
}

#[test]
fn auto_parse_json_and_jsonl_keep_category_outputs_and_logs() {
    let evidence = auto_triage();
    let root = tempfile::tempdir().unwrap();
    for format in ["json", "jsonl"] {
        let output = root.path().join(format);
        let result = run(&[
            "auto-parse",
            "-d",
            evidence.path().to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-t",
            format,
            "-T",
            "2",
            "-K",
            "-q",
        ]);
        assert_eq!(result.code, 0, "{}\n{}", result.stdout, result.stderr);
        let host = output.join("HOST1");
        assert!(
            host.join(format!("file_existence/amcache.{format}"))
                .is_file()
        );
        assert!(host.join(format!("file_system/mft.{format}")).is_file());
        assert!(
            host.join(format!("file_system/usn-journal.{format}"))
                .is_file()
        );
        assert!(host.join("process_execution/srum").is_dir());
        let prefetch = host.join(format!("process_execution/prefetch.{format}"));
        let records = json_documents(&std::fs::read_to_string(prefetch).unwrap());
        assert!(!records.is_empty());
        assert!(
            records.iter().all(|record| {
                record["Computer"] == "HOST1" && record["Artifact"] == "prefetch"
            })
        );
        let manifest: Value =
            serde_json::from_reader(std::fs::File::open(output.join("manifest.json")).unwrap())
                .unwrap();
        for task in manifest["tasks"].as_array().unwrap() {
            let category = task["category"].as_str().unwrap();
            let log = Path::new(task["log"].as_str().unwrap());
            assert_eq!(log.parent(), Some(host.join(category).as_path()));
            assert!(
                log.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .ends_with("-logs.txt")
            );
            assert!(!host.join(category).join("logs").exists());
            assert!(log.is_file());
            if let Some(path) = task["output"].as_str() {
                assert!(Path::new(path).starts_with(host.join(category)));
                assert!(Path::new(path).exists());
            }
        }
        assert!(!host.join("logs").exists());
        assert!(!output.join(format!("all-artifacts.{format}")).exists());
    }
}

#[test]
fn auto_parse_csv_json_and_jsonl_export_the_same_native_siem_records() {
    let evidence = auto_triage();
    let root = tempfile::tempdir().unwrap();
    let mut previous = None;
    for format in ["csv", "json", "jsonl"] {
        let output = root.path().join(format);
        // Include a space and a non-default destination outside the results directory.
        let timeline = root.path().join(format!("SIEM export {format}.jsonl"));
        let result = run(&[
            "auto-parse",
            "-d",
            evidence.path().to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-t",
            format,
            "-j",
            timeline.to_str().unwrap(),
            "-T",
            "2",
            "-K",
            "-q",
        ]);
        assert_eq!(result.code, 0, "{}\n{}", result.stdout, result.stderr);
        let records: Vec<Value> = std::fs::read_to_string(&timeline)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert!(!records.is_empty());
        assert!(records.iter().all(|record| record["Computer"] == "HOST1"));
        assert!(
            records
                .iter()
                .any(|record| record["Artifact"] == "amcache" && record["record_type"] == "file")
        );
        assert!(records.iter().any(|record| record["Artifact"] == "srum"));
        // Value equality ignores object key order but checks native types and nested fields.
        if let Some(expected) = &previous {
            assert_eq!(&records, expected, "{format} SIEM records changed");
        }
        previous = Some(records);
        let manifest: Value =
            serde_json::from_reader(std::fs::File::open(output.join("manifest.json")).unwrap())
                .unwrap();
        let canonical_timeline = std::fs::canonicalize(&timeline).unwrap();
        assert_eq!(
            manifest["json_timeline"],
            canonical_timeline.to_string_lossy().as_ref()
        );
        assert!(manifest["json_timeline_error"].is_null());
        assert!(
            output
                .join("HOST1/process_execution/prefetch-logs.txt")
                .is_file()
        );
        let extension = format!("HOST1/process_execution/prefetch.{format}");
        assert!(output.join(extension).is_file());
        let tasks = manifest["tasks"].as_array().unwrap();
        if format == "csv" {
            assert_eq!(
                tasks
                    .iter()
                    .filter(|task| task["purpose"] == "json_timeline")
                    .count(),
                19
            );
            assert!(
                output
                    .join("HOST1/process_execution/prefetch-json-timeline-logs.txt")
                    .is_file()
            );
            assert!(
                output
                    .join("HOST1/file_existence/amcache/Amcache_files.csv")
                    .is_file()
            );
        } else {
            assert!(tasks.iter().all(|task| task["purpose"] == "artifact"));
        }
        for task in tasks.iter().filter(|task| task["status"] == "succeeded") {
            let path = Path::new(task["output"].as_str().unwrap());
            assert!(path.exists(), "{task}");
            if task["purpose"] == "json_timeline" {
                assert_eq!(path, canonical_timeline);
            }
        }
        assert!(std::fs::read_dir(&output).unwrap().flatten().all(|entry| {
            !entry
                .file_name()
                .to_string_lossy()
                .starts_with(".juntei-work-")
        }));
    }
}

#[test]
fn auto_parse_timeline_rejects_collisions_before_changing_any_files() {
    let evidence = auto_triage();
    let root = tempfile::tempdir().unwrap();
    let output = root.path().join("parsed");
    let source = evidence
        .path()
        .join("HOST1_F.TEST/uploads/auto/C%3A/Windows/AppCompat/Programs/Amcache.hve");
    let original = std::fs::read(&source).unwrap();
    for target in [
        source.clone(),
        evidence.path().join("timeline.jsonl"),
        output.clone(),
        output.join("manifest.json"),
        output.join("manifest.csv"),
        output.join("HOST1/process_execution/prefetch.jsonl"),
        output.join("HOST1/process_execution/prefetch-logs.txt"),
        output.join(".juntei-work-custom/extra.jsonl"),
    ] {
        let result = run(&[
            "auto-parse",
            "-d",
            evidence.path().to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-j",
            target.to_str().unwrap(),
            "-C",
            "-K",
            "-q",
        ]);
        assert_eq!(result.code, 1, "{}\n{}", result.stdout, result.stderr);
        assert_eq!(std::fs::read(&source).unwrap(), original);
        assert!(!output.exists(), "{target:?} changed the output directory");
    }
    let existing = root.path().join("existing.jsonl");
    std::fs::write(&existing, b"previous results\n").unwrap();
    let result = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-j",
        existing.to_str().unwrap(),
        "-K",
        "-q",
    ]);
    assert_eq!(result.code, 1);
    assert_eq!(std::fs::read(existing).unwrap(), b"previous results\n");
    assert!(!output.exists());
}

#[test]
fn auto_parse_timeline_rejects_case_insensitive_output_aliases() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("CaseProbe")).unwrap();
    if !root.path().join("caseprobe").is_dir() {
        return; // Only a case-insensitive filesystem can alias these two output directories.
    }
    let evidence = auto_triage();
    let output = root.path().join("Parsed");
    let alias = root
        .path()
        .join("parsed/HOST1/process_execution/prefetch.jsonl");
    let result = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-t",
        "jsonl",
        "-j",
        alias.to_str().unwrap(),
        "-C",
        "-K",
        "-q",
    ]);
    assert_eq!(result.code, 1, "{}\n{}", result.stdout, result.stderr);
    assert!(!alias.exists());
    assert!(!output.join("manifest.json").exists());
}

#[test]
fn auto_parse_csv_manifest_includes_failed_and_skipped_tasks() {
    let evidence = tempfile::tempdir().unwrap();
    let prefetch = evidence.path().join("HOST1/C/Windows/Prefetch");
    std::fs::create_dir_all(&prefetch).unwrap();
    std::fs::write(
        prefetch.join("BROKEN.EXE-12345678.pf"),
        b"not a prefetch file",
    )
    .unwrap();
    let programs = evidence.path().join("HOST1/C/Windows/AppCompat/Programs");
    std::fs::create_dir_all(&programs).unwrap();
    std::fs::copy(repo(WIN10), programs.join("Amcache.hve")).unwrap();
    let root = tempfile::tempdir().unwrap();
    let output = root.path().join("parsed");
    let timeline = output.join("partial.jsonl");
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-j",
        timeline.to_str().unwrap(),
        "-T",
        "2",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 1, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    let (_, tasks) = csv_rows(&output.join("manifest.csv"));
    assert_eq!(tasks.len(), 38);
    assert!(tasks.iter().any(|task| task["Status"] == "skipped"));
    let failed = tasks
        .iter()
        .find(|task| task["Artifact"] == "prefetch" && task["Purpose"] == "artifact")
        .unwrap();
    assert_eq!(failed["Status"], "failed");
    assert_eq!(failed["Category"], "process_execution");
    assert_eq!(failed["ExitCode"], "1");
    assert!(!failed["Error"].is_empty());
    assert!(Path::new(&failed["Log"]).is_file());
    let records = json_documents(&std::fs::read_to_string(timeline).unwrap());
    assert!(records.iter().any(|record| record["Artifact"] == "amcache"));
    assert!(std::fs::read_dir(&output).unwrap().flatten().any(|entry| {
        entry
            .file_name()
            .to_string_lossy()
            .starts_with(".juntei-work-")
    }));
}

#[test]
fn auto_parse_refuses_an_output_parent_of_the_evidence() {
    let container = tempfile::tempdir().unwrap();
    let evidence = container.path().join("evidence");
    std::fs::create_dir_all(evidence.join("HOST1/C/Windows/System32")).unwrap();
    let marker = container.path().join("keep.txt");
    std::fs::write(&marker, b"must remain\n").unwrap();
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.to_str().unwrap(),
        "-o",
        container.path().to_str().unwrap(),
        "-C",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 1, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    assert!(
        r.stdout.contains("must not contain one another"),
        "{}",
        r.stdout
    );
    assert_eq!(std::fs::read(&marker).unwrap(), b"must remain\n");
    assert!(!container.path().join("manifest.json").exists());
}

#[test]
fn auto_parse_can_extract_defender_quarantined_files() {
    let evidence = auto_triage();
    quarantine_host(evidence.path(), "HOST1_F.TEST");
    let output_root = tempfile::tempdir().unwrap();
    let output = output_root.path().join("parsed");
    let timeline = output.join("siem.jsonl");
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-j",
        timeline.to_str().unwrap(),
        "-x",
        "-T",
        "2",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    let extracted = output
        .join("HOST1/security_products/defender-quarantined")
        .join("tool.exe.quarantined");
    assert!(extracted.is_file(), "{}", extracted.display());
    assert!(
        std::fs::read(extracted)
            .unwrap()
            .starts_with(b"A harmless line of text")
    );
    let security_text = std::fs::read_to_string(
        output
            .join("HOST1/security_products/defender-quarantined/tool.exe.security_descriptor.txt"),
    )
    .unwrap();
    assert!(security_text.contains("Binary descriptor: tool.exe.security.quarantined"));
    assert!(security_text.contains("Owner: S-1-5-21-1111-2222-3333-1001"));
    assert!(security_text.contains("Decode status: complete"));

    let manifest: Value =
        serde_json::from_reader(std::fs::File::open(output.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["extract_defender_quarantined"], true);
    let defender = manifest["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|task| task["artifact"] == "defender-quarantine")
        .unwrap();
    assert!(
        defender["command"]
            .as_array()
            .unwrap()
            .iter()
            .any(|argument| argument == "--extract")
    );

    let timeline_defender = manifest["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|task| {
            task["artifact"] == "defender-quarantine" && task["purpose"] == "json_timeline"
        })
        .unwrap();
    assert!(
        !timeline_defender["command"]
            .as_array()
            .unwrap()
            .iter()
            .any(|arg| arg == "--extract")
    );
    assert!(
        json_documents(&std::fs::read_to_string(timeline).unwrap())
            .iter()
            .any(|record| record["Artifact"] == "defender-quarantine")
    );

    // A later clobber run also clears the reserved extraction directory of older versions.
    let legacy_extraction = output.join("HOST1/defender-quarantined");
    std::fs::create_dir_all(&legacy_extraction).unwrap();
    std::fs::write(
        legacy_extraction.join("old.quarantined"),
        b"harmless fixture",
    )
    .unwrap();
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-C",
        "-T",
        "2",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    assert!(
        !output
            .join("HOST1/security_products/defender-quarantined")
            .exists()
    );
    let manifest: Value =
        serde_json::from_reader(std::fs::File::open(output.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["extract_defender_quarantined"], false);
    assert!(!legacy_extraction.exists());
}

#[test]
fn auto_parse_accepts_targeted_prefetch_without_system32() {
    let evidence = tempfile::tempdir().unwrap();
    let prefetch = evidence.path().join("HOST2/C/Windows/Prefetch");
    let amcache = evidence
        .path()
        .join("HOST3/uploads/auto/C%3A/Windows/AppCompat/Programs");
    std::fs::create_dir_all(&prefetch).unwrap();
    std::fs::create_dir_all(&amcache).unwrap();
    std::fs::copy(
        repo("test_files/prefetch/CMD.EXE-6D6290C5.pf"),
        prefetch.join("CMD.EXE-6D6290C5.pf"),
    )
    .unwrap();
    std::fs::copy(repo(WIN10), amcache.join("Amcache.hve")).unwrap();
    let output_root = tempfile::tempdir().unwrap();
    let output = output_root.path().join("parsed");
    let combined = output.join("timeline.jsonl");
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-t",
        "jsonl",
        "--json-timeline",
        combined.to_str().unwrap(),
        "-T",
        "2",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    let manifest: Value =
        serde_json::from_reader(std::fs::File::open(output.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["hosts"].as_array().unwrap().len(), 2);
    assert_eq!(manifest["hosts"][0]["Computer"], "HOST2");
    assert_eq!(manifest["hosts"][0]["hostname"], "HOST2");
    assert_eq!(manifest["hosts"][1]["Computer"], "HOST3");
    assert_eq!(manifest["hosts"][1]["hostname"], "HOST3");
    assert!(
        manifest["hosts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|host| host["hostname_source"] == "directory name")
    );
    let records: Vec<Value> = std::fs::read_to_string(combined)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        records
            .iter()
            .any(|record| record["Computer"] == "HOST2" && record["Artifact"] == "prefetch")
    );
    assert!(
        records
            .iter()
            .any(|record| record["Computer"] == "HOST3" && record["Artifact"] == "amcache")
    );
}

#[test]
fn auto_parse_accepts_a_new_relative_output_directory() {
    let container = tempfile::tempdir().unwrap();
    let evidence = container.path().join("evidence");
    let windows = evidence.join("HOST1/C/Windows");
    std::fs::create_dir_all(windows.join("System32")).unwrap();
    std::fs::create_dir_all(windows.join("Prefetch")).unwrap();
    std::fs::copy(
        repo("test_files/prefetch/CMD.EXE-6D6290C5.pf"),
        windows.join("Prefetch/CMD.EXE-6D6290C5.pf"),
    )
    .unwrap();
    let r = run_at(
        container.path(),
        &[
            "auto-parse",
            "-d",
            evidence.to_str().unwrap(),
            "-o",
            "test",
            "-K",
            "-q",
        ],
    );
    assert_eq!(r.code, 0, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    assert!(container.path().join("test/manifest.json").is_file());
}

#[test]
fn terminal_report_has_the_summary_and_findings() {
    let r = run(&["parse-registry", "amcache", "-f", WIN10, "-K"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.contains("Start time: "));
    assert!(
        r.stdout
            .contains("Amcache: 123 file entries, 189 device entries")
    );
    assert!(r.stdout.contains("Findings (4):"));
    assert!(r.stdout.contains(
        "[MEDIUM] AMCACHE-SUSPICIOUS-PATH  c:\\users\\testuser\\downloads\\sysmon\\sysmon64.exe"
    ));
    // no detailed records on the terminal unless asked for
    assert!(
        !r.stdout.contains("Inventoried executables:\n"),
        "{}",
        r.stdout
    );
    assert!(r.stdout.contains("Results Summary:"));
    assert!(r.stdout.contains("Inventoried executables: 123"));
    assert!(r.stdout.contains("Inventoried devices: 189"));
    assert!(r.stdout.contains("Findings: 4 (medium: 4)"));
    assert!(r.stdout.contains("Elapsed time: "));
    let r = run(&[
        "parse-registry",
        "amcache",
        "-f",
        WIN10,
        "-K",
        "-q",
        "--files",
    ]);
    assert!(
        !r.stdout.contains("by Yamato Security"),
        "-q hides the banner"
    );
    assert!(r.stdout.contains("7z.exe"), "--files adds the executables");
}

#[test]
fn legacy_hives_are_read() {
    let r = run(&["parse-registry", "amcache", "-f", LEGACY, "-K", "-q"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(
        r.stdout
            .contains("Amcache: 136 file entries, 0 device entries")
    );
    assert!(
        r.stdout
            .contains("Schemas: modern (Windows 10/11) 0, legacy (Windows 8/8.1/2012 R2) 1")
    );
    assert!(r.stdout.contains("Findings: 2 (medium: 2)"));
}

#[test]
fn json_and_jsonl_files_hold_every_record() {
    let dir = tempfile::tempdir().unwrap();
    let json = dir.path().join("amcache.json");
    let r = run(&[
        "parse-registry",
        "amcache",
        "-f",
        WIN10,
        "-o",
        json.to_str().unwrap(),
        "-t",
        "json",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.contains("Parsing in progress. Please wait."));
    assert!(r.stdout.contains("Results saved: "));
    let docs = json_documents(&std::fs::read_to_string(&json).unwrap());
    assert_eq!(docs.len(), 1);
    let doc = &docs[0];
    assert_eq!(doc["amcache_schema"], "modern");
    assert_eq!(doc["summary"]["file_count"], 123);
    assert_eq!(doc["files"].as_array().unwrap().len(), 123);
    assert_eq!(doc["devices"].as_array().unwrap().len(), 189);
    assert_eq!(doc["findings"].as_array().unwrap().len(), 4);
    let sevenzip = doc["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["full_path"] == "c:\\program files\\7-zip\\7z.exe")
        .expect("7z.exe is inventoried");
    assert_eq!(sevenzip["sha1"], "1189cebeb8ffed7316f98b895ff949a726f4026f");

    let jsonl = dir.path().join("amcache.jsonl");
    let r = run(&[
        "parse-registry",
        "amcache",
        "-f",
        WIN10,
        "-o",
        jsonl.to_str().unwrap(),
        "-t",
        "jsonl",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let text = std::fs::read_to_string(&jsonl).unwrap();
    // summary + 4 findings + 123 files + 189 devices
    assert_eq!(text.lines().count(), 317);
    // --files narrows the default of every record
    let r = run(&[
        "parse-registry",
        "amcache",
        "-f",
        WIN10,
        "-o",
        jsonl.to_str().unwrap(),
        "-t",
        "jsonl",
        "-C",
        "--files",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(
        std::fs::read_to_string(&jsonl).unwrap().lines().count(),
        128
    );
}

#[test]
fn csv_tables_go_into_one_directory_per_hive() {
    let tree = triage();
    let out = tempfile::tempdir().unwrap();
    let out_dir = out.path().join("csv");
    let r = run(&[
        "parse-registry",
        "amcache",
        "-d",
        tree.path().to_str().unwrap(),
        "-o",
        out_dir.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(
        r.stdout.contains("Amcache hives: 3"),
        "the .LOG1 file is not a hive"
    );
    assert!(r.stdout.contains("Could not parse "), "{}", r.stdout);
    assert!(
        r.stdout
            .contains("Amcache hives parsed: 2 (could not parse: 1)")
    );
    assert!(r.stdout.contains("Inventoried executables: 259"));
    assert!(r.stdout.contains("Findings: 6 (medium: 6)"));
    assert!(r.stdout.contains("(8 CSV files)"));
    for (sub, rows) in [
        ("HOST1_C_Windows_AppCompat_Programs_Amcache", 123),
        ("HOST2_Amcache", 136),
    ] {
        let files = out_dir.join(sub).join("Amcache_files.csv");
        let text =
            std::fs::read_to_string(&files).unwrap_or_else(|e| panic!("{}: {e}", files.display()));
        let mut rdr = csv::Reader::from_reader(text.as_bytes());
        assert_eq!(rdr.records().count(), rows, "{sub}");
        for table in ["summary", "findings", "devices"] {
            assert!(
                out_dir
                    .join(sub)
                    .join(format!("Amcache_{table}.csv"))
                    .is_file()
            );
        }
    }
    // the results are not overwritten without -C
    let r = run(&[
        "parse-registry",
        "amcache",
        "-d",
        tree.path().to_str().unwrap(),
        "-o",
        out_dir.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("already exists"), "{}", r.stdout);
    let r = run(&[
        "parse-registry",
        "amcache",
        "-d",
        tree.path().to_str().unwrap(),
        "-o",
        out_dir.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
}

#[test]
fn structured_stdout_holds_only_the_data() {
    let tree = triage();
    let r = run(&[
        "parse-registry",
        "amcache",
        "-d",
        tree.path().to_str().unwrap(),
        "-t",
        "json",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let docs = json_documents(&r.stdout);
    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0]["amcache_schema"], "modern");
    assert_eq!(docs[1]["amcache_schema"], "legacy");
    assert!(
        r.stderr.contains("Could not parse "),
        "errors go to stderr: {}",
        r.stderr
    );
    let r = run(&["parse-registry", "amcache", "-f", WIN10, "-t", "jsonl"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for line in r.stdout.lines() {
        serde_json::from_str::<Value>(line).unwrap_or_else(|e| panic!("{e}: {line}"));
    }
}

#[test]
fn text_reports_can_be_saved() {
    let tree = triage();
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("amcache.txt");
    let r = run(&[
        "parse-registry",
        "amcache",
        "-d",
        tree.path().to_str().unwrap(),
        "-o",
        file.to_str().unwrap(),
        "-t",
        "text",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let text = std::fs::read_to_string(&file).unwrap();
    assert_eq!(text.matches("Amcache hive: ").count(), 2);
    assert!(text.contains("Amcache: 123 file entries, 189 device entries"));
    assert!(text.contains("Amcache: 136 file entries, 0 device entries"));
    assert!(
        text.contains("Inventoried executables:"),
        "-o includes every record"
    );
}

#[test]
fn input_and_output_errors() {
    let r = run(&[
        "parse-registry",
        "amcache",
        "-f",
        "no/such/Amcache.hve",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("The file does not exist"), "{}", r.stdout);
    let r = run(&["parse-registry", "amcache", "-f", "README.md", "-K", "-q"]);
    assert_eq!(r.code, 1, "nothing could be parsed");
    assert!(
        r.stdout.contains("Could not parse README.md"),
        "{}",
        r.stdout
    );
    let empty = tempfile::tempdir().unwrap();
    let r = run(&[
        "parse-registry",
        "amcache",
        "-d",
        empty.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout
            .contains("No Amcache hives (Amcache.hve) were found"),
        "{}",
        r.stdout
    );
    let r = run(&["parse-registry", "amcache", "-f", WIN10, "-t", "csv", "-K"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("CSV output needs -o <DIR>"),
        "{}",
        r.stdout
    );
}

// ----- parse-ntfs mft --------------------------------------------------------------------------------

const MFT: &str = "test_files/mft/MFT";

/// (EntryId, FullPath) of every CSV row (paths use `\` on every OS, as Windows shows them).
fn full_paths(csv_text: &str) -> Vec<(String, String)> {
    let mut rdr = csv::Reader::from_reader(csv_text.as_bytes());
    let headers = rdr.headers().unwrap().clone();
    let id = headers.iter().position(|h| h == "EntryId").unwrap();
    let path = headers.iter().position(|h| h == "FullPath").unwrap();
    rdr.records()
        .map(|r| {
            let r = r.unwrap();
            (r[id].to_string(), r[path].to_string())
        })
        .collect()
}

#[test]
fn mft_summary_on_the_terminal() {
    let r = run(&["parse-ntfs", "mft", "-f", MFT, "-K"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.contains("$MFT files: 1"), "{}", r.stdout);
    assert!(r.stdout.contains("Parsing in progress. Please wait."));
    assert!(
        r.stdout
            .contains("Entries: 1,016 (in use: 1,016, deleted: 0)")
    );
    assert!(r.stdout.contains("Directories / files: 153 / 863"));
    assert!(r.stdout.contains("Entries with alternate data streams: 2"));
    assert!(r.stdout.contains("Empty entries (skipped): 8"));
    assert!(r.stdout.contains("To save every entry, add -o <FILE>"));
}

#[test]
fn mft_csv_json_and_jsonl() {
    let dir = tempfile::tempdir().unwrap();
    let csv_path = dir.path().join("mft.csv");
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        MFT,
        "-o",
        csv_path.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.contains("Results saved: "));
    // One row per file name and per alternate data stream
    assert!(
        r.stdout
            .contains("CSV rows (file names and alternate data streams): 1,014"),
        "{}",
        r.stdout
    );
    let (header, rows) = csv_rows(&csv_path);
    assert_eq!(
        header[..6],
        [
            "EntryNumber",
            "SequenceNumber",
            "InUse",
            "ParentEntryNumber",
            "ParentSequenceNumber",
            "ParentPath"
        ]
    );
    assert_eq!(header.len(), 35);
    assert_eq!(rows.len(), 1014);
    let row = |entry: &str, name: &str| {
        rows.iter()
            .find(|r| r["EntryNumber"] == entry && r["FileName"] == name)
            .unwrap_or_else(|| panic!("{entry} {name}"))
    };
    assert_eq!(row("30", "config")["ParentPath"], r".\WINDOWS\system32");
    // $STANDARD_INFORMATION created before $FILE_NAME created, on a whole second
    let bootvid = row("178", "bootvid.dll");
    for (column, value) in [
        ("Extension", ".dll"),
        ("FileSize", "12288"),
        ("SI<FN", "True"),
        ("uSecZeros", "True"),
        ("Copied", "False"),
        ("SiFlags", "Archive"),
        ("NameType", "DosWindows"),
        ("Created0x10", "2006-02-28T12:00:00.0000000Z"),
        ("Created0x30", "2007-06-30T12:51:00.8147072Z"),
    ] {
        assert_eq!(bootvid[column], value, "{column}");
    }
    let ads = row("8", "$BadClus:$Bad");
    assert_eq!(ads["IsAds"], "True");
    assert_eq!(ads["FileSize"], "1261821952");
    assert_eq!(row("8", "$BadClus")["HasAds"], "True");

    // ys-mft-parser's columns: one row per entry (`mft-dump-csv`, upstream's name, still works)
    let mut texts = Vec::new();
    for output_type in ["ys-mft-parser-csv", "mft-dump-csv"] {
        let r = run(&[
            "parse-ntfs",
            "mft",
            "-f",
            MFT,
            "-o",
            csv_path.to_str().unwrap(),
            "-t",
            output_type,
            "-C",
            "-K",
        ]);
        assert_eq!(r.code, 0, "{output_type}: {}", r.stderr);
        texts.push(std::fs::read_to_string(&csv_path).unwrap());
    }
    assert_eq!(texts[0], texts[1]);
    let text = &texts[0];
    assert!(text.starts_with(
        "Signature,EntryId,Sequence,BaseEntryId,BaseEntrySequence,HardLinkCount,Flags,"
    ));
    let paths = full_paths(text);
    assert_eq!(paths.len(), 1016);
    for (id, path) in [
        ("0", "$MFT"),
        ("5", "."),
        ("30", r"WINDOWS\system32\config"),
    ] {
        assert!(
            paths.contains(&(id.to_string(), path.to_string())),
            "{id} {path}"
        );
    }

    let json_path = dir.path().join("mft.json");
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        MFT,
        "-o",
        json_path.to_str().unwrap(),
        "-t",
        "json",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let docs = json_documents(&std::fs::read_to_string(&json_path).unwrap());
    assert_eq!(docs.len(), 1016);
    assert_eq!(docs[0]["header"]["record_number"], 0);
    assert_eq!(docs[0]["full_path"], "$MFT");
    assert!(docs[0]["attributes"].as_array().unwrap().len() > 2);
    let config = docs
        .iter()
        .find(|d| d["header"]["record_number"] == 30)
        .unwrap();
    assert_eq!(config["full_path"], r"WINDOWS\system32\config");

    // -t without -o: only the entries on stdout
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        MFT,
        "-t",
        "jsonl",
        "-r",
        "0-5,30",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let ids: Vec<u64> = r
        .stdout
        .lines()
        .map(|l| {
            serde_json::from_str::<Value>(l).unwrap()["header"]["record_number"]
                .as_u64()
                .unwrap()
        })
        .collect();
    assert_eq!(ids, [0, 1, 2, 3, 4, 5, 30]);
}

#[test]
fn mft_csv_options() {
    let help = run(&["parse-ntfs", "mft", "-h", "-K"]);
    assert_eq!(help.code, 0, "{}", help.stderr);
    assert!(help.stdout.contains("-a, --all-times"), "{}", help.stdout);
    assert!(help.stdout.contains("-s, --short-names"), "{}", help.stdout);
    assert!(
        help.stdout
            .contains("Output type: csv (default), ys-mft-parser-csv, json, jsonl"),
        "{}",
        help.stdout
    );

    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("mft.csv");
    let csv = |extra: &[&str]| {
        let mut args = vec![
            "parse-ntfs",
            "mft",
            "-f",
            MFT,
            "-o",
            out.to_str().unwrap(),
            "-C",
            "-K",
        ];
        args.extend_from_slice(extra);
        let r = run(&args);
        assert_eq!(r.code, 0, "{}", r.stdout);
        csv_rows(&out).1
    };
    // $FILE_NAME times only where they differ from $STANDARD_INFORMATION, unless -a/--all-times
    let mft = |rows: &[std::collections::HashMap<String, String>]| {
        rows.iter()
            .find(|r| r["FileName"] == "$MFT")
            .unwrap()
            .clone()
    };
    assert_eq!(mft(&csv(&[]))["Created0x30"], "");
    assert_eq!(
        mft(&csv(&["-a"]))["Created0x30"],
        "2007-06-30T12:50:52.2523952Z"
    );
    // DOS (8.3) names get rows of their own with -s/--short-names
    let rows = csv(&["-s"]);
    assert_eq!(rows.len(), 1041);
    assert_eq!(rows.iter().filter(|r| r["NameType"] == "Dos").count(), 27);
    // only records created or modified on or after the cutoff (the reference gives the same 8 rows)
    let rows = csv(&["--cutoff", "2007-07-01"]);
    assert_eq!(rows.len(), 8);
    assert_eq!(rows[0]["LastModified0x10"], "2009-01-19T04:17:44.7268320Z");
    assert_eq!(csv(&["--cutoff", "2007-07-01T09:00:00+09:00"]).len(), 8);

    let r = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        MFT,
        "-t",
        "jsonl",
        "--short-names",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr.contains("options of the CSV output"),
        "{}",
        r.stderr
    );
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        MFT,
        "--cutoff",
        "yesterday",
        "-K",
    ]);
    assert_ne!(r.code, 0);
    assert!(r.stderr.contains("is not a date or time"), "{}", r.stderr);
    // a year chrono accepts but a FILETIME can't hold: an error, not an overflow
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        MFT,
        "--cutoff",
        "+262142-12-31T23:59:59",
        "-K",
    ]);
    assert_ne!(r.code, 0);
    assert!(
        r.stderr.contains("outside the times NTFS can hold"),
        "{}",
        r.stderr
    );
}

#[test]
fn mft_recover_slack() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("mft.csv");
    let csv = |extra: &[&str]| {
        let mut args = vec![
            "parse-ntfs",
            "mft",
            "-f",
            MFT,
            "-o",
            out.to_str().unwrap(),
            "-C",
            "-K",
        ];
        args.extend_from_slice(extra);
        let r = run(&args);
        assert_eq!(r.code, 0, "{}", r.stdout);
        (r.stdout, csv_rows(&out))
    };
    let (stdout, (header, rows)) = csv(&["-x"]);
    assert!(
        stdout.contains("File names in FILE record slack: 117 (84 that no file in use has)"),
        "{stdout}"
    );
    assert_eq!(
        header[header.len() - 5..],
        [
            "ParseErrors",
            "FromSlack",
            "SlackRecord",
            "SlackOffset",
            "CopyOfEntryInUse"
        ]
    );
    let slack: Vec<_> = rows.iter().filter(|r| r["FromSlack"] == "True").collect();
    assert_eq!(slack.len(), 117);
    assert_eq!(
        slack
            .iter()
            .filter(|r| r["CopyOfEntryInUse"] == "True")
            .count(),
        33
    );
    // An entry of the index root of WINDOWS (entry 28), left in its record's slack: the file it
    // pointed to, the directory, the name and its $FILE_NAME times.
    let found = slack
        .iter()
        .find(|r| r["FileName"] == "setuperr.log")
        .unwrap();
    for (column, value) in [
        ("EntryNumber", "3522"),
        ("SequenceNumber", "1"),
        ("InUse", "False"),
        ("ParentEntryNumber", "28"),
        ("ParentPath", r".\WINDOWS"),
        ("Created0x10", ""),
        ("Created0x30", "2007-06-30T12:59:00.7391024Z"),
        ("SlackRecord", "28"),
        ("SlackOffset", "29208"),
        ("CopyOfEntryInUse", "False"),
    ] {
        assert_eq!(found[column], value, "{column}");
    }
    let normal = rows.iter().find(|r| r["FileName"] == "$MFT").unwrap();
    assert_eq!(
        (normal["FromSlack"].as_str(), normal["SlackRecord"].as_str()),
        ("False", "")
    );
    // Only the records in -r.
    let (_, (_, rows)) = csv(&["-x", "-r", "28"]);
    assert_eq!(rows.iter().filter(|r| r["FromSlack"] == "True").count(), 2);
    // Without -x, no slack rows and no slack columns.
    let (_, (header, rows)) = csv(&[]);
    assert_eq!(header.last().unwrap(), "ParseErrors");
    assert_eq!(rows.len(), 1014);

    // JSON: each entry's slack_file_names, with their offsets in the $MFT.
    let r = run(&["parse-ntfs", "mft", "-f", MFT, "-t", "jsonl", "-x"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let entries: Vec<serde_json::Value> = r
        .stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let names: Vec<&serde_json::Value> = entries
        .iter()
        .flat_map(|e| e["slack_file_names"].as_array().unwrap())
        .collect();
    assert_eq!(names.len(), 117);
    let windows = entries
        .iter()
        .find(|e| e["header"]["record_number"] == 28)
        .unwrap();
    assert_eq!(windows["slack_file_names"][0]["name"], "setuperr.log");
    assert_eq!(windows["slack_file_names"][0]["offset"], 29208);
    assert_eq!(
        names
            .iter()
            .filter(|n| n["copy_of_entry_in_use"] == true)
            .count(),
        33
    );
    let r = run(&["parse-ntfs", "mft", "-f", MFT, "-t", "jsonl"]);
    assert!(!r.stdout.contains("slack_file_names"));

    let r = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        MFT,
        "-t",
        "ys-mft-parser-csv",
        "-x",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr.contains("--recover-slack adds rows"),
        "{}",
        r.stderr
    );
}

/// The test $MFT with `change` applied, in `dir`.
fn changed_mft(dir: &Path, name: &str, change: impl Fn(&mut Vec<u8>)) -> std::path::PathBuf {
    let mut data = std::fs::read(MFT).unwrap();
    change(&mut data);
    let path = dir.join(name);
    std::fs::write(&path, data).unwrap();
    path
}

#[test]
fn mft_slack_provenance_and_damage() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("mft.csv");
    let export = |mft: &Path, extra: &[&str]| {
        let mut args = vec![
            "parse-ntfs",
            "mft",
            "-f",
            mft.to_str().unwrap(),
            "-x",
            "-o",
            out.to_str().unwrap(),
            "-C",
            "-K",
        ];
        args.extend_from_slice(extra);
        let r = run(&args);
        assert_eq!(r.code, 0, "{}", r.stdout);
        csv_rows(&out).1
    };
    let slack = |rows: &[std::collections::HashMap<String, String>], name: &str| {
        rows.iter()
            .find(|r| r["FromSlack"] == "True" && r["FileName"] == name)
            .cloned()
            .unwrap()
    };

    // The names in an extension record's slack keep that record as their source, not its base.
    let extension = changed_mft(dir.path(), "extension.MFT", |d| {
        d[28 * 1024 + 0x20..28 * 1024 + 0x28].copy_from_slice(&(5u64 | 5 << 48).to_le_bytes());
    });
    let rows = export(&extension, &[]);
    for (name, offset) in [("setuperr.log", "29208"), ("twunk_32.exe", "29328")] {
        let row = slack(&rows, name);
        assert_eq!(
            (row["SlackRecord"].as_str(), row["SlackOffset"].as_str()),
            ("28", offset),
            "{name}"
        );
    }

    // The source is the record's place in the $MFT, where SlackOffset points, whatever its own
    // record number field says.
    let renumbered = changed_mft(dir.path(), "renumbered.MFT", |d| {
        d[28 * 1024 + 0x2C..28 * 1024 + 0x30].copy_from_slice(&999u32.to_le_bytes());
    });
    let rows = export(&renumbered, &["-r", "28"]);
    for (name, offset) in [("setuperr.log", "29208"), ("twunk_32.exe", "29328")] {
        let row = slack(&rows, name);
        assert_eq!(
            (row["SlackRecord"].as_str(), row["SlackOffset"].as_str()),
            ("28", offset),
            "{name}"
        );
    }

    // A record written in part: its slack rows say so too.
    let torn = changed_mft(dir.path(), "torn.MFT", |d| d[28 * 1024 + 510] ^= 0xFF);
    let rows = export(&torn, &["-r", "28"]);
    for name in ["setuperr.log", "twunk_32.exe"] {
        assert!(
            slack(&rows, name)["ParseErrors"].contains("fixup values do not match"),
            "{name}"
        );
    }

    // A size in use of zero: the record's own attributes are not slack, and its row says why its
    // slack was not read.
    let zeroed = changed_mft(dir.path(), "zeroed.MFT", |d| {
        d[209 * 1024 + 0x18..209 * 1024 + 0x1C].copy_from_slice(&0u32.to_le_bytes());
    });
    let rows = export(&zeroed, &["-r", "209"]);
    let ini: Vec<_> = rows
        .iter()
        .filter(|r| r["FileName"] == "system.ini")
        .collect();
    assert_eq!(ini.len(), 1, "no recovered copy of the live name");
    assert_eq!(ini[0]["FromSlack"], "False");
    assert!(
        ini[0]["ParseErrors"]
            .contains("slack not read: its size in use (0x0) ends before its attributes do"),
        "{}",
        ini[0]["ParseErrors"]
    );
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        zeroed.to_str().unwrap(),
        "-x",
        "-t",
        "jsonl",
        "-r",
        "209",
    ]);
    let entry: serde_json::Value = serde_json::from_str(r.stdout.lines().next().unwrap()).unwrap();
    assert_eq!(entry["slack_file_names"], serde_json::json!([]));
    assert!(
        entry["slack_problem"]
            .as_str()
            .unwrap()
            .starts_with("its size in use (0x0)")
    );
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        zeroed.to_str().unwrap(),
        "-D",
        "209",
        "-K",
        "-q",
    ]);
    assert!(
        r.stdout
            .contains("Names in slack:     not read: its size in use (0x0)"),
        "{}",
        r.stdout
    );
}

#[test]
fn mft_resident_data() {
    let help = run(&["parse-ntfs", "mft", "-h", "-K"]);
    for option in [
        "-x, --recover-slack",
        "-i, --include-resident-data",
        "-E, --resident-extensions <EXT>",
        "-M, --resident-max-size <BYTES>",
        "-D, --dump-entry <ENTRY>",
        "-L, --list-directory <ENTRY|PATH>",
    ] {
        assert!(help.stdout.contains(option), "{option}: {}", help.stdout);
    }

    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("mft.csv");
    let csv = |extra: &[&str]| {
        let mut args = vec![
            "parse-ntfs",
            "mft",
            "-f",
            MFT,
            "-o",
            out.to_str().unwrap(),
            "-C",
            "-K",
        ];
        args.extend_from_slice(extra);
        let r = run(&args);
        assert_eq!(r.code, 0, "{}", r.stdout);
        csv_rows(&out)
    };
    let (header, rows) = csv(&["-i"]);
    assert_eq!(
        header[header.len() - 5..],
        [
            "SourceFile",
            "ResidentDataBase64",
            "ResidentDataHex",
            "ResidentDataASCII",
            "ParseErrors"
        ]
    );
    let with_data: Vec<_> = rows
        .iter()
        .filter(|r| !r["ResidentDataBase64"].is_empty())
        .collect();
    assert_eq!(with_data.len(), 22);
    let ini = with_data
        .iter()
        .find(|r| r["FileName"] == "system.ini")
        .unwrap();
    assert!(ini["ResidentDataASCII"].starts_with("; for 16-bit app support\r\n"));
    assert!(ini["ResidentDataHex"].starts_with("3B-20-66-6F-72-20"));
    assert!(ini["ResidentDataBase64"].starts_with("OyBmb3Ig"));
    // Binary content: no text.
    let pif = with_data
        .iter()
        .find(|r| r["FileName"] == "_default.pif")
        .unwrap();
    assert_eq!(pif["ResidentDataASCII"], "");
    assert!(pif["ResidentDataHex"].starts_with("00-C9"));
    // By extension and size: win.ini (477 bytes) is larger than 300.
    let (_, rows) = csv(&["-i", "-E", ".INI", "-M", "300"]);
    let names: Vec<&str> = rows
        .iter()
        .filter(|r| !r["ResidentDataBase64"].is_empty())
        .map(|r| r["FileName"].as_str())
        .collect();
    assert_eq!(names, ["system.ini", "desktop.ini"]);

    let r = run(&["parse-ntfs", "mft", "-f", MFT, "-E", "ini", "-K"]);
    assert_ne!(r.code, 0, "-E needs -i");
    let r = run(&["parse-ntfs", "mft", "-f", MFT, "-t", "jsonl", "-i"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr.contains("always has the resident data"),
        "{}",
        r.stderr
    );
}

#[test]
fn mft_dump_entry_and_list_directory() {
    let r = run(&["parse-ntfs", "mft", "-f", MFT, "-D", "28", "-K", "-q"]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "Entry 28-1 at offset 0x7000 of the $MFT (1024 bytes)",
        "  Flags:              0x0003 (in use, directory)",
        "  Size in use:        472 of 1024 bytes",
        "  Names in slack:     2 (2 that no file in use has)",
        "\"slack_file_names\": [",
        "\"name\": \"setuperr.log\"",
        "0000  46 49 4C 45",
    ] {
        assert!(r.stdout.contains(line), "{line}: {}", r.stdout);
    }
    let shown = |stdout: &str| {
        stdout
            .lines()
            .filter(|l| !l.starts_with("Start time") && !l.starts_with("Elapsed time"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let hex = run(&["parse-ntfs", "mft", "-f", MFT, "-D", "0x1C-0x1", "-K", "-q"]);
    assert_eq!(shown(&hex.stdout), shown(&r.stdout));
    for (args, message) in [
        (
            &["-D", "28-9"][..],
            "Entry 28 has sequence number 1, not 9.",
        ),
        (
            &["-D", "99999"],
            "The $MFT has 1,024 entries: there is no entry 99999.",
        ),
        (&["-L", "209"], "Entry 209 is a file, not a directory."),
        (
            &["-L", "windows/nope"],
            r".\WINDOWS has no directory named nope.",
        ),
        (&["-D", "5", "-o", "x.csv"], "they don't take -o"),
    ] {
        let mut all = vec!["parse-ntfs", "mft", "-f", MFT, "-K", "-q"];
        all.extend_from_slice(args);
        let r = run(&all);
        assert_eq!(r.code, 1, "{args:?}");
        assert!(r.stdout.contains(message), "{args:?}: {}", r.stdout);
    }
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-d",
        "test_files/mft",
        "-D",
        "5",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("give it with -f"), "{}", r.stdout);
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        MFT,
        "-D",
        "5",
        "-L",
        "5",
        "-K",
        "-q",
    ]);
    assert_ne!(r.code, 0, "one inspection at a time");

    // A directory by path (case-insensitive, either separator) or by entry.
    for target in [r"\WINDOWS", "windows", "28", "28-1"] {
        let r = run(&["parse-ntfs", "mft", "-f", MFT, "-L", target, "-K", "-q"]);
        assert_eq!(r.code, 0, "{target}: {}", r.stdout);
        assert!(
            r.stdout
                .contains(r"Directory .\WINDOWS (entry 28-1, in use)"),
            "{target}: {}",
            r.stdout
        );
        assert!(
            r.stdout
                .contains("33 names: directories 29, files 4, deleted 0")
        );
    }
    let r = run(&["parse-ntfs", "mft", "-f", MFT, "-L", "windows", "-K", "-q"]);
    let line = r
        .stdout
        .lines()
        .find(|l| l.ends_with("_default.pif"))
        .unwrap();
    assert!(line.starts_with("285-1"), "{line}");
    assert!(line.contains("File") && line.contains("707"), "{line}");
}

#[test]
fn mft_csv_reports_damaged_attributes() {
    let dir = tempfile::tempdir().unwrap();
    let mut data = std::fs::read(repo(MFT)).unwrap();
    // entry 62: the value of its first $FILE_NAME (at 0x98) starts past the end of the record
    data[62 * 1024 + 0xAC] = 0xF0;
    // entry 209: so does the value of its resident $DATA (at 0x108)
    data[209 * 1024 + 0x108 + 0x14..][..2].copy_from_slice(&0x3F0u16.to_le_bytes());
    let damaged = dir.path().join("damaged_$MFT");
    std::fs::write(&damaged, &data).unwrap();
    let out = dir.path().join("mft.csv");
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        damaged.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-r",
        "62,209",
        "--short-names",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    let (_, rows) = csv_rows(&out);
    // the damaged attributes are left out and said so
    let names: Vec<&str> = rows.iter().map(|r| r["FileName"].as_str()).collect();
    assert_eq!(
        names,
        ["Repository", "system.ini"],
        "REPOSI~1 was in the damaged $FILE_NAME"
    );
    assert_eq!(
        rows[0]["ParseErrors"],
        "$FILE_NAME attribute at 0x98 left out: its value (0x52 bytes at 0xF0) runs past the end of the attribute"
    );
    assert!(
        rows[1]["ParseErrors"].starts_with("$DATA attribute at 0x108 left out: "),
        "{}",
        rows[1]["ParseErrors"]
    );
    // without its $DATA, the size is the $FILE_NAME's (it was 231)
    assert_eq!(rows[1]["FileSize"], "219");
}

#[test]
fn mft_csv_reports_damaged_extension_records() {
    let dir = tempfile::tempdir().unwrap();
    let mut data = std::fs::read(repo(MFT)).unwrap();
    // entry 226 (a small file with a resident $DATA at 0x100) becomes an extension record of
    // entry 209 (system.ini, sequence 1), and the value of its $DATA runs past the end
    let extension = 226 * 1024;
    data[extension + 0x20..][..8].copy_from_slice(&(209u64 | (1 << 48)).to_le_bytes());
    data[extension + 0x100 + 0x14..][..2].copy_from_slice(&0x3F0u16.to_le_bytes());
    let damaged = dir.path().join("damaged_$MFT");
    std::fs::write(&damaged, &data).unwrap();
    let out = dir.path().join("mft.csv");
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        damaged.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-r",
        "209,226",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    let (_, rows) = csv_rows(&out);
    // the extension record has no rows of its own; its name is a name of entry 209
    assert!(rows.iter().all(|r| r["EntryNumber"] == "209"), "{rows:?}");
    assert_eq!(rows.len(), 2);
    for row in &rows {
        assert_eq!(
            row["ParseErrors"],
            "extension record 226: $DATA attribute at 0x100 left out: its value (0x1DD bytes at 0x3F0) runs past the end of the attribute",
            "{}",
            row["FileName"]
        );
        // the base record's own $DATA gives the size
        assert_eq!(row["FileSize"], "231");
    }
}

#[test]
fn mft_resident_streams_and_directory_mode() {
    let tree = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tree.path().join("HOST1/C")).unwrap();
    std::fs::create_dir_all(tree.path().join("HOST2")).unwrap();
    std::fs::copy(repo(MFT), tree.path().join("HOST1/C/$MFT")).unwrap();
    std::fs::copy(repo(MFT), tree.path().join("HOST2/$MFT")).unwrap();
    std::fs::copy(repo(MFT), tree.path().join("HOST2/$MFTMirr")).unwrap();
    let out = tempfile::tempdir().unwrap();
    let csv_dir = out.path().join("csv");
    let streams = out.path().join("streams");
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-d",
        tree.path().to_str().unwrap(),
        "-o",
        csv_dir.to_str().unwrap(),
        "-e",
        streams.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(
        r.stdout.contains("$MFT files: 2"),
        "$MFTMirr is not an $MFT: {}",
        r.stdout
    );
    assert!(r.stdout.contains("(2 files)"));
    assert!(
        // 24 resident streams in each, of which 2 are empty and not written
        r.stdout.contains("Resident streams extracted: 44"),
        "{}",
        r.stdout
    );
    for name in ["HOST1_C__MFT", "HOST2__MFT"] {
        let text = std::fs::read_to_string(csv_dir.join(format!("{name}.csv"))).unwrap();
        assert!(
            text.starts_with("EntryNumber,SequenceNumber,InUse,"),
            "{name}"
        );
        assert_eq!(
            csv_rows(&csv_dir.join(format!("{name}.csv"))).1.len(),
            1014,
            "{name}"
        );
        assert_eq!(
            std::fs::read_dir(streams.join(name)).unwrap().count(),
            22,
            "{name}"
        );
    }
    // stream files are named after the entry and its path
    let names: Vec<String> = std::fs::read_dir(streams.join("HOST2__MFT"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert!(names.iter().all(|n| n.ends_with(".dontrun")), "{names:?}");
    // the entries of several $MFT files cannot share stdout
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-d",
        tree.path().to_str().unwrap(),
        "-t",
        "csv",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr.contains("With -d, save the entries with -o <DIR>"),
        "{}",
        r.stderr
    );
}

#[test]
fn mft_damaged_inputs_do_not_stop_the_run() {
    let dir = tempfile::tempdir().unwrap();
    // two directories that name each other as parent (the upstream crate overflows the stack)
    let mut data = std::fs::read(repo(MFT)).unwrap();
    let mut dirs: Vec<(usize, Vec<usize>)> = Vec::new();
    for i in 16..data.len() / 1024 {
        let base = i * 1024;
        if &data[base..base + 4] != b"FILE" || data[base + 0x16] & 2 == 0 {
            continue;
        }
        let mut off = usize::from(u16::from_le_bytes([data[base + 0x14], data[base + 0x15]]));
        let mut fn_offsets = Vec::new();
        while off + 8 < 1024 {
            let t = u32::from_le_bytes(data[base + off..base + off + 4].try_into().unwrap());
            let len = u32::from_le_bytes(data[base + off + 4..base + off + 8].try_into().unwrap());
            if t == 0xFFFF_FFFF || len == 0 {
                break;
            }
            if t == 0x30 && data[base + off + 8] == 0 {
                let content = usize::from(u16::from_le_bytes([
                    data[base + off + 0x14],
                    data[base + off + 0x15],
                ]));
                fn_offsets.push(base + off + content);
            }
            off += len as usize;
        }
        if !fn_offsets.is_empty() {
            dirs.push((i, fn_offsets));
        }
        if dirs.len() == 2 {
            break;
        }
    }
    let (a, b) = (dirs[0].0, dirs[1].0);
    for ((_, offsets), parent) in dirs.iter().zip([b, a]) {
        for &o in offsets {
            data[o..o + 6].copy_from_slice(&(parent as u64).to_le_bytes()[..6]);
            data[o + 6..o + 8].copy_from_slice(&[0, 0]);
        }
    }
    let cycle = dir.path().join("cycle_$MFT");
    std::fs::write(&cycle, &data).unwrap();
    let zero = dir.path().join("zero_$MFT");
    std::fs::write(&zero, vec![0u8; 4096]).unwrap();
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "{}\n{}", r.stdout, r.stderr);
    assert!(
        r.stdout
            .contains("$MFT files parsed: 1 (could not parse: 1)"),
        "{}",
        r.stdout
    );
    assert!(
        r.stdout.contains("no valid FILE record header was found"),
        "{}",
        r.stdout
    );
    assert!(r.stdout.contains("Entries: 1,016"), "{}", r.stdout);
    // the CSV's parent paths of the loop end in PathUnknown
    let out = dir.path().join("cycle.csv");
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        cycle.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    let (_, rows) = csv_rows(&out);
    let looped: Vec<_> = rows
        .iter()
        .filter(|r| r["EntryNumber"] == a.to_string() || r["EntryNumber"] == b.to_string())
        .collect();
    assert_eq!(looped.len(), 2);
    assert!(
        looped
            .iter()
            .all(|r| r["ParentPath"].starts_with(r".\PathUnknown\Directory with ID 0x")),
        "{looped:?}"
    );

    let r = run(&["parse-ntfs", "mft", "-f", "README.md", "-K", "-q"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout
            .contains("Could not parse README.md: Failed to determine the entry size"),
        "{}",
        r.stdout
    );
}

const PREFETCH_DIR: &str = "test_files/prefetch";
const PREFETCH_CMD10: &str = "test_files/prefetch/CMD.EXE-6D6290C5.pf";

#[test]
fn prefetch_summary_on_the_terminal() {
    let r = run(&["parse-file", "prefetch", "-d", PREFETCH_DIR, "-K"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.contains("Prefetch files: 6"), "{}", r.stdout);
    assert!(r.stdout.contains("Parsing in progress. Please wait."));
    assert!(r.stdout.contains("Could not parse "));
    assert!(r.stdout.contains("SEARCHPROTOCOLHOST.EXE-AFAD3EF9.pf"));
    assert!(
        r.stdout
            .contains("Prefetch files parsed: 5 (could not parse: 1)")
    );
    assert!(r.stdout.contains(
        "Format versions: 30 (Windows 10/11) 2, 26 (Windows 8/8.1) 1, 23 (Windows Vista/7) 1, 17 (Windows XP/2003) 1"
    ));
    assert!(r.stdout.contains("Executables: 3"));
    assert!(r.stdout.contains("Executions (run counts): 25"));
    assert!(r.stdout.contains(
        "Run times recorded: 15 (2013-03-10T10:11:49.281250Z - 2024-02-05T06:17:41.144014200Z)"
    ));
    assert!(r.stdout.contains("Findings: 1 (medium: 1)"));
    assert!(r.stdout.contains("To save every record, add -o <FILE>"));
}

#[test]
fn prefetch_csv_json_and_jsonl() {
    let dir = tempfile::tempdir().unwrap();
    let csv_path = dir.path().join("prefetch.csv");
    let r = run(&[
        "parse-file",
        "prefetch",
        "-f",
        PREFETCH_CMD10,
        "-o",
        csv_path.to_str().unwrap(),
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.contains("Results saved: "));
    let mut csv = csv::Reader::from_path(&csv_path).unwrap();
    let header = csv.headers().unwrap().clone();
    let rows: Vec<csv::StringRecord> = csv.records().map(|r| r.unwrap()).collect();
    assert_eq!(rows.len(), 1);
    let field = |name: &str| rows[0][header.iter().position(|h| h == name).unwrap()].to_string();
    assert_eq!(field("ExecutableName"), "CMD.EXE");
    assert_eq!(field("PrefetchHash"), "6D6290C5");
    assert_eq!(field("FormatVersion"), "30");
    assert_eq!(field("RunCount"), "4");
    assert_eq!(field("LastRun"), "2024-02-05T06:17:41.144014200Z");
    assert_eq!(field("PreviousRun3"), "2024-02-04T04:15:17.060262400Z");
    assert_eq!(field("PreviousRun4"), "");
    assert!(field("ExecutablePath").ends_with(r"\WINDOWS\SYSWOW64\CMD.EXE"));
    assert_eq!(field("FileCount"), "25");
    assert_eq!(field("Findings"), "");

    // one pretty JSON document per file, with every field
    let json_path = dir.path().join("prefetch.json");
    let r = run(&[
        "parse-file",
        "prefetch",
        "-d",
        PREFETCH_DIR,
        "-o",
        json_path.to_str().unwrap(),
        "-t",
        "json",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let docs = json_documents(&std::fs::read_to_string(&json_path).unwrap());
    assert_eq!(docs.len(), 5);
    let rust_out = docs
        .iter()
        .find(|d| d["executable_name"] == "RUST_OUT.EXE")
        .unwrap();
    assert_eq!(rust_out["findings"][0]["code"], "SuspiciousExecutionPath");
    assert_eq!(rust_out["findings"][0]["severity"], "medium");
    let consent = docs
        .iter()
        .find(|d| d["executable_name"] == "CONSENT.EXE")
        .unwrap();
    assert_eq!(consent["windows_version"], "Windows Vista/7");
    assert!(
        consent["files"][0]["mft_entry"].is_u64(),
        "version 23 has file references"
    );
    assert!(
        !consent["volumes"][0]["directories"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    // JSON Lines on stdout: only the data
    let r = run(&[
        "parse-file",
        "prefetch",
        "-f",
        PREFETCH_CMD10,
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let lines: Vec<&str> = r.stdout.lines().collect();
    assert_eq!(lines.len(), 1, "{}", r.stdout);
    let doc: Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(doc["last_run_times"].as_array().unwrap().len(), 4);
}

#[test]
fn prefetch_timeline_is_one_record_per_execution_sorted_by_time() {
    let r = run(&[
        "parse-file",
        "prefetch",
        "-d",
        PREFETCH_DIR,
        "-T",
        "-t",
        "csv",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stderr.contains("Could not parse "), "errors go to stderr");
    let mut csv = csv::Reader::from_reader(r.stdout.as_bytes());
    assert_eq!(
        csv.headers().unwrap().iter().collect::<Vec<_>>(),
        [
            "RunTime",
            "ExecutableName",
            "ExecutablePath",
            "PrefetchHash",
            "RunCount",
            "SourceFile"
        ]
    );
    let times: Vec<String> = csv.records().map(|r| r.unwrap()[0].to_string()).collect();
    assert_eq!(times.len(), 15);
    let mut sorted = times.clone();
    sorted.sort();
    assert_eq!(times, sorted);
    assert_eq!(times[0], "2013-03-10T10:11:49.281250Z");
}

/// A Windows 7 prefetch file of CONSENT.EXE turned into one of SVCHOST.EXE loaded from outside
/// System32 (the names have the same length, so every offset stays valid).
#[test]
fn prefetch_system_binary_outside_system32_is_a_high_finding() {
    fn utf16(s: &str) -> Vec<u8> {
        s.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }
    fn replace_all(data: &mut [u8], from: &[u8], to: &[u8]) {
        let mut i = 0;
        while i + from.len() <= data.len() {
            if &data[i..i + from.len()] == from {
                data[i..i + from.len()].copy_from_slice(to);
                i += from.len();
            } else {
                i += 1;
            }
        }
    }
    let mut data = std::fs::read(repo("test_files/prefetch/CONSENT.EXE-531BD9EA.pf")).unwrap();
    replace_all(
        &mut data,
        &utf16(r"SYSTEM32\CONSENT.EXE"),
        &utf16(r"TEMPDIR1\SVCHOST.EXE"),
    );
    replace_all(&mut data, &utf16("CONSENT.EXE"), &utf16("SVCHOST.EXE"));
    let dir = tempfile::tempdir().unwrap();
    let pf = dir.path().join("SVCHOST.EXE-531BD9EA.pf");
    std::fs::write(&pf, data).unwrap();

    let r = run(&["parse-file", "prefetch", "-f", pf.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.contains("Findings: 1 (high: 1)"), "{}", r.stdout);
    assert!(r.stdout.contains(
        r"[HIGH] SVCHOST.EXE is a Windows system binary but was loaded from \DEVICE\HARDDISKVOLUME2\WINDOWS\TEMPDIR1\SVCHOST.EXE"
    ), "{}", r.stdout);
    assert!(r.stdout.contains("(T1036.005)"));
}

#[test]
fn prefetch_input_and_output_errors() {
    let dir = tempfile::tempdir().unwrap();
    let r = run(&[
        "parse-file",
        "prefetch",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("No prefetch files (*.pf) were found"),
        "{}",
        r.stdout
    );

    let r = run(&[
        "parse-file",
        "prefetch",
        "-f",
        "test_files/prefetch/corrupt/SEARCHPROTOCOLHOST.EXE-AFAD3EF9.pf",
        "-K",
    ]);
    assert_eq!(r.code, 1, "nothing was parsed");
    assert!(
        r.stdout
            .contains("Prefetch files parsed: 0 (could not parse: 1)")
    );

    let out = dir.path().join("exists.csv");
    std::fs::write(&out, b"x").unwrap();
    let r = run(&[
        "parse-file",
        "prefetch",
        "-f",
        PREFETCH_CMD10,
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("The output file already exists"));
    let r = run(&[
        "parse-file",
        "prefetch",
        "-f",
        PREFETCH_CMD10,
        "-o",
        out.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
}

const USN_SLICE: &str = "test_files/usn/win11_rathbunvm_$J";

/// A `USN_RECORD_V2` at `usn` (also its offset), for MFT entry `file` in directory `parent`.
fn usn_v2(
    usn: u64,
    file: (u64, u16),
    parent: (u64, u16),
    reasons: u32,
    attributes: u32,
    name: &str,
) -> Vec<u8> {
    let reference =
        |(entry, sequence): (u64, u16)| (u64::from(sequence) << 48 | entry).to_le_bytes();
    let name: Vec<u8> = name.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut r = vec![0; 4];
    r.extend(2_u16.to_le_bytes());
    r.extend(0_u16.to_le_bytes());
    r.extend(reference(file));
    r.extend(reference(parent));
    r.extend(usn.to_le_bytes());
    r.extend(133_537_680_000_000_000_u64.to_le_bytes()); // 2024-03-01T12:00:00Z
    r.extend(reasons.to_le_bytes());
    r.extend(0_u32.to_le_bytes());
    r.extend(0_u32.to_le_bytes());
    r.extend(attributes.to_le_bytes());
    r.extend((name.len() as u16).to_le_bytes());
    r.extend(60_u16.to_le_bytes());
    r.extend(name);
    r.resize(r.len().next_multiple_of(8), 0);
    let length = (r.len() as u32).to_le_bytes();
    r[..4].copy_from_slice(&length);
    r
}

const DIRECTORY: u32 = 0x10;
const ARCHIVE: u32 = 0x20;
const CREATE: u32 = 0x100;
const DELETE: u32 = 0x200;
const OLD_NAME: u32 = 0x1000;
const NEW_NAME: u32 = 0x2000;
const CLOSE: u32 = 0x8000_0000;

/// A journal for the volume of `test_files/mft/MFT` (where entry 36 is
/// `WINDOWS/system32/spool/drivers/w32x86`, 37 its subdirectory `3`, 30
/// `WINDOWS/system32/config`): a file in a directory the MFT knows, a directory
/// renamed to the name the MFT has, a directory created and deleted, a parent nobody
/// knows, and a range-tracking (V4) record.
fn usn_journal() -> Vec<u8> {
    let mut j = Vec::new();
    let mut add = |r: &dyn Fn(u64) -> Vec<u8>| {
        let record = r(j.len() as u64);
        j.extend(record);
    };
    add(&|u| usn_v2(u, (900, 1), (36, 1), CREATE, ARCHIVE, "evil.dll"));
    add(&|u| usn_v2(u, (901, 1), (37, 1), CREATE | CLOSE, ARCHIVE, "before.txt"));
    add(&|u| usn_v2(u, (37, 1), (36, 1), OLD_NAME, DIRECTORY, "old3"));
    add(&|u| usn_v2(u, (37, 1), (36, 1), NEW_NAME, DIRECTORY, "3"));
    add(&|u| usn_v2(u, (902, 1), (37, 1), CREATE | CLOSE, ARCHIVE, "after.txt"));
    add(&|u| usn_v2(u, (950, 2), (30, 1), CREATE, DIRECTORY, "Temp1"));
    add(&|u| {
        usn_v2(
            u,
            (903, 1),
            (950, 2),
            CREATE | CLOSE,
            ARCHIVE,
            "dropped.exe",
        )
    });
    add(&|u| usn_v2(u, (950, 2), (30, 1), DELETE | CLOSE, DIRECTORY, "Temp1"));
    add(&|u| usn_v2(u, (904, 1), (999, 9), CREATE, ARCHIVE, "orphan.txt"));
    // USN_RECORD_V4: header, references, USN, reasons, source, remaining, 1 extent of 16 bytes
    let mut v4 = 80_u32.to_le_bytes().to_vec();
    v4.extend(4_u16.to_le_bytes());
    v4.extend(0_u16.to_le_bytes());
    v4.extend(u128::from(1_u64 << 48 | 900).to_le_bytes());
    v4.extend(u128::from(1_u64 << 48 | 36).to_le_bytes());
    v4.extend((j.len() as u64).to_le_bytes());
    v4.extend(2_u32.to_le_bytes());
    v4.extend(0_u32.to_le_bytes());
    v4.extend(0_u32.to_le_bytes());
    v4.extend(1_u16.to_le_bytes());
    v4.extend(16_u16.to_le_bytes());
    v4.extend(0_u64.to_le_bytes());
    v4.extend(4096_u64.to_le_bytes());
    j.extend(v4);
    j
}

/// A KAPE-like collection: the journal at `HOST1/C/$Extend/$J`, the volume's `$MFT` at `HOST1/C/$MFT`.
fn usn_triage() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let extend = dir.path().join("HOST1/C/$Extend");
    std::fs::create_dir_all(&extend).unwrap();
    std::fs::write(extend.join("$J"), usn_journal()).unwrap();
    std::fs::copy(repo(MFT), dir.path().join("HOST1/C/$MFT")).unwrap();
    dir
}

#[test]
fn usn_journal_paths_are_those_of_each_records_time() {
    let dir = usn_triage();
    let journal = dir.path().join("HOST1/C/$Extend/$J");
    let r = run(&[
        "parse-ntfs",
        "usn-journal",
        "-f",
        journal.to_str().unwrap(),
        "-t",
        "csv",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let mut csv = csv::Reader::from_reader(r.stdout.as_bytes());
    let header = csv.headers().unwrap().clone();
    assert_eq!(
        &header.iter().take(13).collect::<Vec<_>>(),
        &[
            "Name",
            "Extension",
            "EntryNumber",
            "SequenceNumber",
            "ParentEntryNumber",
            "ParentSequenceNumber",
            "ParentPath",
            "UpdateSequenceNumber",
            "UpdateTimestamp",
            "UpdateReasons",
            "FileAttributes",
            "OffsetToData",
            "SourceFile",
        ],
        "the common columns first"
    );
    let col = |name: &str| header.iter().position(|h| h == name).unwrap();
    let rows: Vec<csv::StringRecord> = csv.records().map(|r| r.unwrap()).collect();
    assert_eq!(rows.len(), 9, "the V4 record is not a CSV row");
    let path = |name: &str| {
        let row = rows.iter().find(|r| &r[col("Name")] == name).unwrap();
        row[col("ParentPath")].to_string()
    };
    assert_eq!(path("evil.dll"), r".\WINDOWS\system32\spool\drivers\w32x86");
    assert_eq!(
        path("before.txt"),
        r".\WINDOWS\system32\spool\drivers\w32x86\old3",
        "before the rename"
    );
    assert_eq!(
        path("after.txt"),
        r".\WINDOWS\system32\spool\drivers\w32x86\3"
    );
    assert_eq!(
        path("dropped.exe"),
        r".\WINDOWS\system32\config\Temp1",
        "a directory the MFT no longer has"
    );
    assert_eq!(path("orphan.txt"), r".\[Unknown 999-9]");
    let evil = rows.iter().find(|r| &r[col("Name")] == "evil.dll").unwrap();
    assert_eq!(&evil[col("Extension")], ".dll");
    assert_eq!(&evil[col("EntryNumber")], "900");
    assert_eq!(&evil[col("UpdateTimestamp")], "2024-03-01T12:00:00Z");
    assert_eq!(&evil[col("UpdateReasons")], "FILE_CREATE");
    assert_eq!(&evil[col("FileAttributes")], "ARCHIVE");
    assert_eq!(&evil[col("Version")], "2.0");
}

#[test]
fn usn_journal_summary_and_directory_mode() {
    let dir = usn_triage();
    let out = dir.path().join("out");
    let r = run(&[
        "parse-ntfs",
        "usn-journal",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for line in [
        "USN journals: 1",
        "USN journals parsed: 1",
        "Records: 9 (V2: 9, V3: 0)",
        "Range-tracking records (V4): 1",
        "Files created / deleted / renamed: 6 / 1 / 1",
        "Paths: rewound from the journal, seeded from the $MFT",
        "Records with an unknown directory: 1",
        "Damaged data: 0 spans (0 B)",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let saved = std::fs::read_to_string(out.join("HOST1_C__Extend__J.csv")).unwrap();
    assert_eq!(saved.lines().count(), 10);

    // JSON Lines: every field, and the range-tracking record with its extents
    let journal = dir.path().join("HOST1/C/$Extend/$J");
    let r = run(&[
        "parse-ntfs",
        "usn-journal",
        "-f",
        journal.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let docs: Vec<Value> = r
        .stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(docs.len(), 10);
    assert_eq!(docs[0]["file"]["entry"], 900);
    assert_eq!(docs[0]["reasons"][0], "FILE_CREATE");
    assert_eq!(docs[9]["version"], "4.0");
    assert_eq!(docs[9]["extents"][0]["length"], 4096);
}

#[test]
fn usn_journal_without_an_mft_and_real_records() {
    let r = run(&["parse-ntfs", "usn-journal", "-f", USN_SLICE, "-K"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for line in [
        "Records: 199 (V2: 199, V3: 0)",
        "Time range: 2022-02-05T18:56:53.653148700Z",
        "Paths: rewound from the journal (no $MFT found)",
        "To save every record, add -o <FILE>",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
}

#[test]
fn usn_records_are_carved_from_other_data() {
    let dir = tempfile::tempdir().unwrap();
    let image = dir.path().join("disk.raw");
    let mut data: Vec<u8> = (0..20_000_u32)
        .flat_map(|i| i.wrapping_mul(2_654_435_761).to_le_bytes())
        .collect();
    data.extend(usn_journal());
    data.extend([0xA5; 4096]);
    std::fs::write(&image, &data).unwrap();
    let r = run(&[
        "parse-ntfs",
        "usn-journal",
        "-f",
        image.to_str().unwrap(),
        "--carve",
        "-m",
        MFT,
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let docs: Vec<Value> = r
        .stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(
        docs.len(),
        9,
        "every V2 record, nothing from the other data"
    );
    assert_eq!(docs[0]["offset"], 80_000);
    assert_eq!(
        docs[0]["parent_path"],
        r".\WINDOWS\system32\spool\drivers\w32x86"
    );

    let r = run(&[
        "parse-ntfs",
        "usn-journal",
        "-f",
        image.to_str().unwrap(),
        "--carve",
        "-K",
    ]);
    assert!(
        r.stdout.contains("Records carved: 9 (V2: 9, V3: 0)"),
        "{}",
        r.stdout
    );
    assert!(r.stdout.contains("Paths: none (no $MFT given)"));
}

// --- parse-ese srum --------------------------------------------------------------------------------

const SRUM_WIN10: &str = "test_files/srum/win10_rathbunvm_SRUDB.dat";
const SRUM_SERVER: &str = "test_files/srum/server2022_aptsimulatorvm_SRUDB.dat";
/// The account most of the Windows 10 database's entries are for.
const RATHBUN_SID: &str = "S-1-5-21-2290304894-658537038-505560459-1000";

/// A registry key of the hives the SRUM and registry tests build: its name, values (name, type,
/// data), subkeys and last-write time.
struct RegKey {
    name: String,
    values: Vec<(String, u32, Vec<u8>)>,
    subkeys: Vec<RegKey>,
    written: u64,
}

fn reg_key(name: &str, values: Vec<(&str, u32, Vec<u8>)>, subkeys: Vec<RegKey>) -> RegKey {
    RegKey {
        name: name.to_string(),
        values: values
            .into_iter()
            .map(|(n, t, d)| (n.to_string(), t, d))
            .collect(),
        subkeys,
        written: 0,
    }
}

impl RegKey {
    /// Last written at `filetime`.
    fn written(mut self, filetime: u64) -> Self {
        self.written = filetime;
        self
    }
}

const REG_SZ: u32 = 1;
const REG_EXPAND_SZ: u32 = 2;
const REG_BINARY: u32 = 3;
const REG_DWORD: u32 = 4;
const REG_MULTI_SZ: u32 = 7;
const REG_QWORD: u32 = 11;

fn utf16z(s: &str) -> Vec<u8> {
    s.encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect()
}

fn put_u32(buf: &mut [u8], at: usize, v: u32) {
    buf[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

/// Append a cell (its size, negative: allocated) holding `data`; its offset in the hive bins.
fn alloc_cell(cells: &mut Vec<u8>, data: &[u8]) -> u32 {
    let at = cells.len();
    let size = (4 + data.len()).next_multiple_of(8);
    cells.extend_from_slice(&(-(size as i32)).to_le_bytes());
    cells.extend_from_slice(data);
    cells.resize(at + size, 0);
    at as u32
}

/// Write a key node with its values and subkeys (an `lf` list, sorted); its offset.
fn write_key(cells: &mut Vec<u8>, key: &RegKey, parent: u32, root: bool) -> u32 {
    let mut nk = vec![0u8; 76];
    nk[0..2].copy_from_slice(b"nk");
    // KEY_COMP_NAME (an ASCII name), and for the root KEY_HIVE_ENTRY | KEY_NO_DELETE
    nk[2..4].copy_from_slice(&(if root { 0x2cu16 } else { 0x20 }).to_le_bytes());
    nk[4..12].copy_from_slice(&key.written.to_le_bytes());
    put_u32(&mut nk, 16, parent);
    for unset in [28, 32, 40, 44, 48] {
        put_u32(&mut nk, unset, u32::MAX);
    }
    nk[72..74].copy_from_slice(&(key.name.len() as u16).to_le_bytes());
    nk.extend_from_slice(key.name.as_bytes());
    let at = alloc_cell(cells, &nk);
    let field = at as usize + 4;
    if !key.values.is_empty() {
        let mut list = Vec::new();
        for (name, kind, data) in &key.values {
            let mut vk = vec![0u8; 20];
            vk[0..2].copy_from_slice(b"vk");
            vk[2..4].copy_from_slice(&(name.len() as u16).to_le_bytes());
            if data.len() <= 4 {
                // resident: the data is in the offset field
                put_u32(&mut vk, 4, data.len() as u32 | 0x8000_0000);
                vk[8..8 + data.len()].copy_from_slice(data);
            } else {
                put_u32(&mut vk, 4, data.len() as u32);
                let data_at = alloc_cell(cells, data);
                put_u32(&mut vk, 8, data_at);
            }
            put_u32(&mut vk, 12, *kind);
            vk[16..18].copy_from_slice(&1u16.to_le_bytes()); // VALUE_COMP_NAME
            vk.extend_from_slice(name.as_bytes());
            list.extend_from_slice(&alloc_cell(cells, &vk).to_le_bytes());
        }
        let list_at = alloc_cell(cells, &list);
        put_u32(cells, field + 36, key.values.len() as u32);
        put_u32(cells, field + 40, list_at);
    }
    if !key.subkeys.is_empty() {
        let mut subkeys: Vec<&RegKey> = key.subkeys.iter().collect();
        subkeys.sort_by_key(|k| k.name.to_ascii_uppercase());
        let mut lf = b"lf".to_vec();
        lf.extend_from_slice(&(subkeys.len() as u16).to_le_bytes());
        for sub in subkeys {
            let sub_at = write_key(cells, sub, at, false);
            lf.extend_from_slice(&sub_at.to_le_bytes());
            let mut hint = [0u8; 4];
            for (h, b) in hint.iter_mut().zip(sub.name.bytes()) {
                *h = b;
            }
            lf.extend_from_slice(&hint);
        }
        let lf_at = alloc_cell(cells, &lf);
        put_u32(cells, field + 20, key.subkeys.len() as u32);
        put_u32(cells, field + 28, lf_at);
    }
    at
}

/// A registry hive (REGF 1.5, one hive bin) holding `root`.
fn regf(root: &RegKey) -> Vec<u8> {
    let mut cells = vec![0u8; 32]; // the hive bin's header
    let root_at = write_key(&mut cells, root, 0, true);
    let size = (cells.len() + 8).next_multiple_of(4096);
    let free = size - cells.len();
    cells.extend_from_slice(&(free as i32).to_le_bytes());
    cells.resize(size, 0);
    cells[0..4].copy_from_slice(b"hbin");
    put_u32(&mut cells, 8, size as u32);
    let mut base = vec![0u8; 4096];
    base[0..4].copy_from_slice(b"regf");
    for (at, v) in [(4, 1), (8, 1), (20, 1), (24, 5), (32, 1), (44, 1)] {
        put_u32(&mut base, at, v);
    }
    put_u32(&mut base, 36, root_at);
    put_u32(&mut base, 40, size as u32);
    let checksum = base[..508]
        .as_chunks::<4>()
        .0
        .iter()
        .fold(0u32, |x, w| x ^ u32::from_le_bytes(*w));
    put_u32(&mut base, 508, checksum);
    base.extend_from_slice(&cells);
    base
}

/// A Wi-Fi profile key: its index and a `Channel Hints` value declaring `length` bytes of SSID.
fn wifi_profile(guid: &str, index: u32, length: i32, ssid: &[u8]) -> RegKey {
    let mut hints = length.to_le_bytes().to_vec();
    hints.extend_from_slice(ssid);
    hints.resize(hints.len().max(36), 0);
    reg_key(
        guid,
        vec![("ProfileIndex", REG_DWORD, index.to_le_bytes().to_vec())],
        vec![reg_key(
            "MetaData",
            vec![("Channel Hints", REG_BINARY, hints)],
            vec![],
        )],
    )
}

/// A SOFTWARE hive naming two accounts and two Wi-Fi networks, with a profile left without
/// metadata, one whose index another profile already took (the first one names it), and
/// one whose `Channel Hints` is damaged. Index 0 is the `L2ProfileId` of the Windows
/// 10 database's network entries.
fn software_hive() -> Vec<u8> {
    let profile = |sid: &str, image: &str| {
        reg_key(
            sid,
            vec![("ProfileImagePath", REG_EXPAND_SZ, utf16z(image))],
            vec![],
        )
    };
    let profiles = reg_key(
        "ProfileList",
        vec![],
        vec![
            profile("S-1-5-19", r"%systemroot%\ServiceProfiles\LocalService"),
            profile(RATHBUN_SID, r"C:\Users\rathbun"),
        ],
    );
    let wlan_profiles = reg_key(
        "Profiles",
        vec![],
        vec![
            wifi_profile("{1A}", 0, 10, b"CoffeeShop"),
            wifi_profile("{1B}", 0, 6, b"Second"),
            wifi_profile("{1C}", 8, 4, b"Home"),
            wifi_profile("{1D}", 9, 40, b"Broken"),
            reg_key(
                "{1E}",
                vec![("ProfileIndex", REG_DWORD, 10u32.to_le_bytes().to_vec())],
                vec![],
            ),
        ],
    );
    let microsoft = reg_key(
        "Microsoft",
        vec![],
        vec![
            reg_key(
                "Windows NT",
                vec![],
                vec![reg_key("CurrentVersion", vec![], vec![profiles])],
            ),
            reg_key(
                "WlanSvc",
                vec![],
                vec![reg_key(
                    "Interfaces",
                    vec![],
                    vec![reg_key("{0F00}", vec![], vec![wlan_profiles])],
                )],
            ),
        ],
    );
    regf(&reg_key("ROOT", vec![], vec![microsoft]))
}

/// The rows of a CSV file, by column name.
fn csv_rows(path: &Path) -> (Vec<String>, Vec<std::collections::HashMap<String, String>>) {
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

#[test]
fn srum_summary_on_the_terminal() {
    let r = run(&["parse-ese", "srum", "-f", SRUM_WIN10, "-K"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for line in [
        "SRUM databases parsed: 1",
        "Entries: 309",
        "  AppResourceUseInfo: 118",
        "  AppTimelineProvider: 163",
        "  NetworkConnections: 1",
        "  NetworkUsages: 23",
        "  vfuprov: 4",
        "Time range: 2022-02-05T22:19:00Z - 2022-02-05T22:19:00Z",
        "Network: 1.8 MiB sent, 141.8 MiB received",
        "Top senders:",
        "   262.1 KiB  LicenseManager",
        "Account and Wi-Fi names: none (no SOFTWARE hive found; give one with -r)",
        "6 AppId / UserId values are not in SruDbIdMapTable",
        "To save every entry, add -o <DIR>",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    assert!(!r.stdout.contains("Left dirty"));
}

#[test]
fn srum_csv_tables_with_names_from_the_software_hive() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("SOFTWARE");
    std::fs::write(&hive, software_hive()).unwrap();
    let out = dir.path().join("out");
    let hive_arg = hive.to_str().unwrap();
    let r = run(&[
        "parse-ese",
        "srum",
        "-f",
        SRUM_WIN10,
        "-r",
        hive_arg,
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains(
            "Account and Wi-Fi names: from the SOFTWARE hive (2 accounts, 2 Wi-Fi networks)"
        ),
        "{}",
        r.stdout
    );
    assert!(
        r.stdout
            .contains(r"Wi-Fi profile {0F00}\{1D}: Channel Hints says the name has 40 bytes"),
        "{}",
        r.stdout
    );
    assert!(r.stdout.contains("(6 CSV files)"), "{}", r.stdout);
    let mut files: Vec<String> = std::fs::read_dir(&out)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    files.sort();
    assert_eq!(
        files,
        [
            "AppResourceUseInfo.csv",
            "AppTimelineProvider.csv",
            "IdentityMap.csv",
            "NetworkConnections.csv",
            "NetworkUsages.csv",
            "VfuProvider.csv"
        ]
    );

    let (header, rows) = csv_rows(&out.join("NetworkUsages.csv"));
    assert_eq!(
        header,
        [
            "Id",
            "Timestamp",
            "ExeInfo",
            "ExeInfoDescription",
            "ExeTimestamp",
            "AppType",
            "Sid",
            "UserName",
            "UserId",
            "AppId",
            "InterfaceLuid",
            "InterfaceType",
            "L2ProfileId",
            "ProfileName",
            "L2ProfileFlags",
            "BytesSent",
            "BytesRecvd"
        ]
    );
    assert_eq!(rows.len(), 23);
    let first = &rows[0];
    for (column, value) in [
        ("Id", "1"),
        ("Timestamp", "2022-02-05T22:19:00Z"),
        ("ExeInfo", "wlidsvc"),
        ("AppType", "Service"),
        ("Sid", "S-1-5-19"),
        ("UserName", "LocalService"),
        ("InterfaceType", "IF_TYPE_ETHERNET_CSMACD"),
        ("ProfileName", "CoffeeShop"),
        ("BytesSent", "11988"),
        ("BytesRecvd", "12755"),
    ] {
        assert_eq!(first[column], value, "{column}");
    }
    let rathbun: Vec<_> = rows.iter().filter(|r| r["Sid"] == RATHBUN_SID).collect();
    assert!(!rathbun.is_empty());
    assert!(rathbun.iter().all(|r| r["UserName"] == "rathbun"));

    // `!!name!time!x!description` names split; FILETIME columns as times
    let (_, timeline) = csv_rows(&out.join("AppTimelineProvider.csv"));
    let split = timeline
        .iter()
        .find(|r| r["ExeInfoDescription"] == "[LocalService] [WdiServiceHost]")
        .unwrap();
    assert_eq!(split["ExeInfo"], "svchost.exe");
    assert_eq!(split["ExeTimestamp"], "1972-12-14T16:22:50Z");
    assert_eq!(split["EndTime"], "2022-02-05T22:18:49.336199700Z");
    assert!(
        timeline
            .iter()
            .all(|r| r["EndTime"].starts_with("2022-02-"))
    );
    let (_, ids) = csv_rows(&out.join("IdentityMap.csv"));
    assert_eq!(ids.len(), 288);
    assert_eq!(ids[3]["IdBlob"], "S-1-5-19");
    let (_, vfu) = csv_rows(&out.join("VfuProvider.csv"));
    assert_eq!(vfu.len(), 4);

    // existing results are kept without -C
    let again = run(&[
        "parse-ese",
        "srum",
        "-f",
        SRUM_WIN10,
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(again.code, 1);
    assert!(again.stdout.contains("already exists"), "{}", again.stdout);
}

#[test]
fn srum_json_and_jsonl() {
    let r = run(&["parse-ese", "srum", "-f", SRUM_WIN10, "-t", "jsonl"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(!r.stdout.contains("Results Summary"));
    let lines: Vec<&str> = r.stdout.lines().collect();
    assert_eq!(lines.len(), 309);
    // the columns keep their order
    assert!(
        lines[0]
            .starts_with(r#"{"SourceFile":"test_files/srum/win10_rathbunvm_SRUDB.dat","Table":"#)
    );
    let entries: Vec<Value> = lines
        .iter()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let connection = entries
        .iter()
        .find(|e| e["Provider"] == "NetworkConnections")
        .unwrap();
    assert_eq!(
        connection["Table"],
        "{DD6636C4-8929-4683-974E-22C046A43763}"
    );
    assert!(
        connection["ConnectStartTime"]
            .as_str()
            .unwrap()
            .starts_with("20")
    );
    assert!(connection["ConnectedTime"].is_u64());
    assert!(connection["UserName"].is_null());

    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("srum.json");
    let r = run(&[
        "parse-ese",
        "srum",
        "-f",
        SRUM_WIN10,
        "-t",
        "json",
        "-o",
        file.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    let documents = json_documents(&std::fs::read_to_string(&file).unwrap());
    assert_eq!(documents.len(), 309);
    assert_eq!(documents, entries);
}

#[test]
fn srum_value_states_and_bytes() {
    let r = run(&[
        "parse-ese",
        "srum",
        "-f",
        SRUM_WIN10,
        "-t",
        "jsonl",
        "--include-value-state",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let entries: Vec<Value> = r
        .stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(entries.len(), 309);
    for e in &entries {
        let states = e["_value_state"].as_object().unwrap();
        // the database's values, not what is made from them
        for column in ["Id", "Timestamp", "UserId", "AppId"] {
            assert_eq!(states[column], "parsed", "{column}");
        }
        for made in ["ExeInfo", "Sid", "AppType", "SourceFile"] {
            assert!(!states.contains_key(made), "{made}");
        }
        // the id's bytes: a Long, little-endian
        let id = i32::try_from(e["Id"].as_i64().unwrap()).unwrap();
        let hex: String = id
            .to_le_bytes()
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect();
        assert_eq!(e["_raw"]["Id"], hex);
    }
    // a NULL in the database: null, no bytes
    let timeline = entries
        .iter()
        .find(|e| e["Provider"] == "AppTimelineProvider" && e["AudioInS"].is_null())
        .unwrap();
    assert_eq!(timeline["_value_state"]["AudioInS"], "null");
    assert!(timeline["_raw"].get("AudioInS").is_none());
    let nulls = entries
        .iter()
        .flat_map(|e| e["_value_state"].as_object().unwrap().values())
        .filter(|s| *s == "null")
        .count();
    assert_eq!(nulls, 4333);

    // not asked for: as before
    let r = run(&["parse-ese", "srum", "-f", SRUM_WIN10, "-t", "jsonl"]);
    assert!(!r.stdout.contains("_value_state"));
    assert!(!r.stdout.contains("_raw"));
    // a CSV cell can't say it
    let dir = tempfile::tempdir().unwrap();
    let r = run(&[
        "parse-ese",
        "srum",
        "-f",
        SRUM_WIN10,
        "-o",
        dir.path().join("csv").to_str().unwrap(),
        "--include-value-state",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout
            .contains("--include-value-state needs -t json or jsonl"),
        "{}",
        r.stdout
    );
}

#[test]
fn srum_triage_directory_with_server_tables() {
    let dir = tempfile::tempdir().unwrap();
    let host1 = dir.path().join("HOST1/C/Windows/System32");
    std::fs::create_dir_all(host1.join("sru")).unwrap();
    std::fs::create_dir_all(host1.join("config")).unwrap();
    std::fs::copy(repo(SRUM_WIN10), host1.join("sru/SRUDB.dat")).unwrap();
    std::fs::write(host1.join("config/SOFTWARE"), software_hive()).unwrap();
    let host2 = dir.path().join("HOST2/C/Windows/System32/SRU");
    std::fs::create_dir_all(&host2).unwrap();
    std::fs::copy(repo(SRUM_SERVER), host2.join("SRUDB.dat")).unwrap();
    std::fs::write(dir.path().join("HOST3_SRUDB.dat"), b"not a database").unwrap();
    let triage = dir.path().to_str().unwrap();

    let out = tempfile::tempdir().unwrap();
    let r = run(&[
        "parse-ese",
        "srum",
        "-d",
        triage,
        "-o",
        out.path().to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "SRUM databases parsed: 2 (could not parse: 1)",
        "Entries: 476",
        "  ProcessorTime: 2",
        "  TotalUsed: 4",
        "Account and Wi-Fi names: from the SOFTWARE hive (2 accounts, 2 Wi-Fi networks)",
        "Databases without a SOFTWARE hive: 1",
        "Could not parse",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let server = out.path().join("HOST2_C_Windows_System32_SRU_SRUDB");
    let mut files: Vec<String> = std::fs::read_dir(&server)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    files.sort();
    assert_eq!(
        files,
        [
            "AppResourceUseInfo.csv",
            "IdentityMap.csv",
            "NetworkBytes.csv",
            "NetworkConnections.csv",
            "ProcessorTime.csv",
            "SizeInBytes.csv",
            "TotalUsed.csv"
        ]
    );
    let (_, used) = csv_rows(&server.join("TotalUsed.csv"));
    assert_eq!(used.len(), 4);
    assert!(used.iter().all(|r| r["AppType"] == "Object"));
    let (_, network) = csv_rows(
        &out.path()
            .join("HOST1_C_Windows_System32_sru_SRUDB/NetworkUsages.csv"),
    );
    assert_eq!(network[0]["UserName"], "LocalService");

    // one JSONL file per database
    let jsonl = tempfile::tempdir().unwrap();
    let r = run(&[
        "parse-ese",
        "srum",
        "-d",
        triage,
        "-t",
        "jsonl",
        "-o",
        jsonl.path().to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    let server = std::fs::read_to_string(
        jsonl
            .path()
            .join("HOST2_C_Windows_System32_SRU_SRUDB.jsonl"),
    )
    .unwrap();
    assert_eq!(server.lines().count(), 167);
}

#[test]
fn srum_input_and_output_errors() {
    let r = run(&[
        "parse-ese",
        "srum",
        "-f",
        "test_files/srum/missing.dat",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("The file does not exist"), "{}", r.stdout);

    let r = run(&["parse-ese", "srum", "-f", "test_files/srum/README.md", "-K"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("Could not parse"), "{}", r.stdout);

    let r = run(&["parse-ese", "srum", "-d", "test_files/amcache", "-K"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("No SRUM databases"), "{}", r.stdout);

    let r = run(&["parse-ese", "srum", "-f", SRUM_WIN10, "-t", "csv"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("CSV output needs -o <DIR>"),
        "{}",
        r.stdout
    );

    let r = run(&["parse-ese", "srum", "-d", "test_files/srum", "-t", "jsonl"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr.contains("With -d, save the entries with -o"),
        "{}",
        r.stderr
    );

    // a SOFTWARE hive that can't be read: the entries are still written, without names
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("SOFTWARE");
    std::fs::write(&hive, b"regf but not a hive").unwrap();
    let r = run(&[
        "parse-ese",
        "srum",
        "-f",
        SRUM_WIN10,
        "-r",
        hive.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("Could not read the SOFTWARE hive"),
        "{}",
        r.stdout
    );
    assert!(r.stdout.contains("Entries: 309"), "{}", r.stdout);
}

// --- parse-file lnk ---------------------------------------------------------------------------------

const LNK_DIR: &str = "test_files/lnk";

#[test]
fn lnk_summary_on_the_terminal() {
    let r = run(&["parse-file", "lnk", "-d", LNK_DIR, "-K"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    // found under -d, so with the platform's separator
    let found = |name: &str| format!("  {} -> ", Path::new(LNK_DIR).join(name).display());
    for line in [
        "Shortcuts parsed: 6".to_owned(),
        "Targets: fixed drives 2, removable drives 1, CD/DVD 0, network shares 1".to_owned(),
        "Machines that made them: 2 (pc, win-3qmdjgkf3g9)".to_owned(),
        "With problems: 1".to_owned(),
        "Hidden command line (string data over 260 characters): 1".to_owned(),
        found("padded_cli_arguments.lnk") + r#"This PC\C:\Windows\System32\cmd.exe /c "set PATH="#,
        "Data after the end of the link: 1".to_owned(),
        found("appended_data.lnk") + r"This PC\All Control Panel Items\" + "\n",
        "Shell or script host started with arguments: 2".to_owned(),
        found("cmd_powershell.lnk")
            + r#"C:\Windows\System32\cmd.exe /c powershell.exe -c "Write-Host This is a test""#,
        "To save every record, add -o <FILE>".to_owned(),
    ] {
        assert!(r.stdout.contains(&line), "{line}\n{}", r.stdout);
    }
}

#[test]
fn lnk_csv_has_the_common_columns() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("lnk.csv");
    let r = run(&[
        "parse-file",
        "lnk",
        "-d",
        LNK_DIR,
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    let (header, rows) = csv_rows(&out);
    assert_eq!(
        &header[..5],
        [
            "SourceFile",
            "SourceCreated",
            "SourceModified",
            "SourceAccessed",
            "TargetCreated"
        ]
    );
    assert_eq!(header.len(), 46);
    assert_eq!(rows.len(), 6);
    let row = |name: &str| {
        rows.iter()
            .find(|r| r["SourceFile"].ends_with(name))
            .unwrap()
    };
    // Nero InfoTool, from plaso's test data: as the reference reads it
    let nero = row("nero_infotool.lnk");
    for (column, value) in [
        ("TargetCreated", "2009-06-05T20:13:20Z"),
        ("TargetModified", "2009-06-05T20:13:20Z"),
        ("FileSize", "4635160"),
        (
            "LocalPath",
            r"C:\Program Files (x86)\Nero\Nero 9\Nero InfoTool\InfoTool.exe",
        ),
        (
            "TargetIDAbsolutePath",
            r"This PC\C:\Program Files (x86)\Nero\Nero 9\Nero InfoTool\InfoTool.exe",
        ),
        ("Arguments", "-ScParameter=30002  "),
        ("TargetMFTEntryNumber", "81349"),
        ("TargetMFTSequenceNumber", "1"),
        ("MachineID", "win-3qmdjgkf3g9"),
        ("MachineMACAddress", "70:5a:b6:16:a5:f3"),
        ("TrackerCreatedOn", "2010-01-29T21:02:17.840171700Z"),
        ("VolumeSerialNumber", "70ECFA33"),
        ("DriveType", "DRIVE_FIXED"),
        ("FileAttributes", "ARCHIVE"),
        (
            "ExtraBlocksPresent",
            "IconEnvironmentDataBlock|SpecialFolderDataBlock|KnownFolderDataBlock|PropertyStoreDataBlock|TrackerDataBlock",
        ),
        ("ShowWindow", "SW_SHOWNORMAL"),
        ("VolumeDroid", "c989f572-4deb-4c64-b0d1-dc4959bbff28"),
        ("FileDroid", "958705b5-0d19-11df-9fe4-705ab616a5f3"),
        ("Problems", ""),
        ("AppendedBytes", "0"),
    ] {
        assert_eq!(nero[column], value, "{column}");
    }
    assert!(nero["HeaderFlags"].starts_with("HasLinkTargetIDList|HasLinkInfo|"));
    assert_eq!(
        row("network_share_unicode.lnk")["NetworkPath"],
        r"\\127.0.0.1\TEST"
    );
    assert_eq!(row("removable_drive.lnk")["DriveType"], "DRIVE_REMOVABLE");
    assert_eq!(row("appended_data.lnk")["AppendedBytes"], "67649");
    let padded = row("padded_cli_arguments.lnk");
    assert_eq!(padded["Arguments"].chars().count(), 1431);
    assert!(padded["Problems"].starts_with("string data longer than 260 characters"));
}

#[test]
fn lnk_json_and_jsonl() {
    let r = run(&[
        "parse-file",
        "lnk",
        "-f",
        "test_files/lnk/nero_infotool.lnk",
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(!r.stdout.contains("Results Summary"));
    let v: Value = serde_json::from_str(r.stdout.trim()).unwrap();
    assert_eq!(v["tracker"]["machine_id"], "win-3qmdjgkf3g9");
    assert_eq!(
        v["tracker"]["volume_droid_birth"],
        "c989f572-4deb-4c64-b0d1-dc4959bbff28"
    );
    assert_eq!(v["target_mft_entry"], 81_349);
    assert_eq!(v["header"]["show_command"], "SW_SHOWNORMAL");
    let items = v["target_items"].as_array().unwrap();
    assert_eq!(items[0]["name"], "This PC");
    assert_eq!(items.last().unwrap()["name"], "InfoTool.exe");
    assert_eq!(v["extra_blocks"][0], "IconEnvironmentDataBlock");
    assert!(
        v["icon_environment"]
            .as_str()
            .unwrap()
            .starts_with("%ProgramFiles%")
    );

    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("lnk.json");
    let r = run(&[
        "parse-file",
        "lnk",
        "-d",
        LNK_DIR,
        "-t",
        "json",
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    let docs = json_documents(&std::fs::read_to_string(&out).unwrap());
    assert_eq!(docs.len(), 6);
    let padded = docs
        .iter()
        .find(|d| {
            d["source_file"]
                .as_str()
                .unwrap()
                .ends_with("padded_cli_arguments.lnk")
        })
        .unwrap();
    assert_eq!(padded["problems"].as_array().unwrap().len(), 2);
}

#[test]
fn lnk_times_that_are_not_times() {
    // a link whose target creation time is 2^63 (100 ns units), past any time: empty, no panic
    let dir = tempfile::tempdir().unwrap();
    let mut data = std::fs::read(repo("test_files/lnk/nero_infotool.lnk")).unwrap();
    data[28..36].copy_from_slice(&0x8000_0000_0000_0000u64.to_le_bytes());
    let file = dir.path().join("nero_infotool.lnk");
    std::fs::write(&file, &data).unwrap();
    let r = run(&[
        "parse-file",
        "lnk",
        "-f",
        file.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let v: Value = serde_json::from_str(r.stdout.trim()).unwrap();
    assert_eq!(v["header"]["target_created"], "");
    assert_eq!(v["header"]["target_modified"], "2009-06-05T20:13:20Z");
}

#[test]
fn lnk_input_and_output_errors() {
    let r = run(&[
        "parse-file",
        "lnk",
        "-f",
        "test_files/lnk/missing.lnk",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("The file does not exist"), "{}", r.stdout);

    let r = run(&[
        "parse-file",
        "lnk",
        "-f",
        "test_files/lnk/../amcache/README.md",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("Could not parse"), "{}", r.stdout);
    assert!(r.stdout.contains("not a shell link"), "{}", r.stdout);

    let r = run(&["parse-file", "lnk", "-d", "test_files/amcache", "-K"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("No shortcuts (*.lnk)"), "{}", r.stdout);

    let r = run(&[
        "parse-file",
        "lnk",
        "-f",
        "test_files/lnk/nero_infotool.lnk",
        "--code-page",
        "12345",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("Code page 12345 is not supported"),
        "{}",
        r.stdout
    );

    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("x.csv");
    std::fs::write(&out, b"keep").unwrap();
    let r = run(&[
        "parse-file",
        "lnk",
        "-f",
        "test_files/lnk/nero_infotool.lnk",
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("already exists"), "{}", r.stdout);
}

// --- parse-file jump-lists --------------------------------------------------------------------------

const JUMP_LIST_DIR: &str = "test_files/jump_lists";
const EXPLORER_WIN10: &str = "test_files/jump_lists/f01b4d95cf55d32a.automaticDestinations-ms";
const CHROME_WIN11: &str = "test_files/jump_lists/5d696d521de238c3.customDestinations-ms";

#[test]
fn jump_list_summary_on_the_terminal() {
    let r = run(&["parse-file", "jump-lists", "-d", JUMP_LIST_DIR, "-K"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    // found under -d, so with the platform's separator
    let found = |name: &str| format!("  {} (", Path::new(JUMP_LIST_DIR).join(name).display());
    for line in [
        "Jump lists parsed: 4 (automatic 3, custom 1)".to_owned(),
        "Entries: 51 (pinned 4), custom list links 6".to_owned(),
        "Applications: 4 (4 named from their AppID)".to_owned(),
        "  Windows Explorer Windows 8.1 (f01b4d95cf55d32a): 9 links".to_owned(),
        "Machines in the entries: 3 (citadel-dc01, desktop-gnccvrk, pc)".to_owned(),
        "Targets: fixed drives 58, removable drives 0, CD/DVD 1, network shares 5".to_owned(),
        "With problems: 0".to_owned(),
        "Links left over from an earlier version of a custom list: 2".to_owned(),
        found("5d696d521de238c3.customDestinations-ms")
            + r"at 11472) -> C:\Program Files\Google\Chrome\Application\chrome.exe --win-jumplist-action=most-visited http://192.168.50.33:5000/",
        "Link streams no DestList entry names (removed or damaged entries): 0".to_owned(),
        "Items staged for a CD or DVD burn: 1".to_owned(),
        found("5f7b5f1e01b83767.automaticDestinations-ms") + r"entry 1) -> D:\autorun.inf",
        "Shell or script host started with arguments: 0".to_owned(),
        "To save every entry, add -o <DIR> (CSV tables)".to_owned(),
    ] {
        assert!(r.stdout.contains(&line), "{line}\n{}", r.stdout);
    }
}

#[test]
fn jump_list_csv_has_the_common_columns() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("jl");
    let r = run(&[
        "parse-file",
        "jump-lists",
        "-d",
        JUMP_LIST_DIR,
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(r.stdout.contains("(2 CSV files)"), "{}", r.stdout);
    let (header, rows) = csv_rows(&out.join("AutomaticDestinations.csv"));
    assert_eq!(
        &header[..7],
        [
            "SourceFile",
            "SourceCreated",
            "SourceModified",
            "SourceAccessed",
            "AppId",
            "AppIdDescription",
            "HasSps"
        ]
    );
    assert_eq!(header[43], "Notes");
    assert_eq!(header.len(), 50);
    assert_eq!(rows.len(), 51);
    let row = |file: &str, mru: &str| {
        rows.iter()
            .find(|r| r["SourceFile"].ends_with(file) && r["MRU"] == mru)
            .unwrap()
    };
    // Windows 10 Explorer, from the DFIR Artifact Museum: as the reference reads it
    let targets = row("f01b4d95cf55d32a.automaticDestinations-ms", "3");
    for (column, value) in [
        ("AppId", "f01b4d95cf55d32a"),
        ("AppIdDescription", "Windows Explorer Windows 8.1"),
        ("DestListVersion", "4"),
        ("LastUsedEntryNumber", "9"),
        ("EntryNumber", "7"),
        ("CreationTime", "2022-02-05T19:30:37.504034700Z"),
        ("LastModified", "2022-02-05T19:39:12.024818800Z"),
        ("Hostname", "desktop-gnccvrk"),
        ("MacAddress", "ec:63:d7:72:83:36"),
        ("Path", r"C:\Users\AndrewRathbun\Desktop\Targets"),
        ("InteractionCount", "1"),
        ("PinStatus", "False"),
        ("FileDroid", "1884d55b-86ba-11ec-9e53-ec63d7728336"),
        ("VolumeBirthDroid", "a0eda9fe-dbef-4d91-bf7a-eabd0971f8df"),
        ("TargetCreated", "2022-02-05T19:39:11.619072400Z"),
        ("LocalPath", r"C:\Users\AndrewRathbun\Desktop\Targets"),
        (
            "TargetIDAbsolutePath",
            r"This PC\C:\Users\AndrewRathbun\Desktop\Targets",
        ),
        ("TargetMFTEntryNumber", "105479"),
        ("TargetMFTSequenceNumber", "2"),
        ("DriveType", "DRIVE_FIXED"),
        (
            "ExtraBlocksPresent",
            "TrackerDataBlock|PropertyStoreDataBlock",
        ),
        ("Notes", ""),
        ("Problems", ""),
    ] {
        assert_eq!(targets[column], value, "{column}");
    }
    // a pinned known folder, named
    let documents = row("f01b4d95cf55d32a.automaticDestinations-ms", "6");
    assert_eq!(
        documents["Path"],
        "knownfolder:{FDD39AD0-238F-46AF-ADB4-6C85480369C7}"
    );
    assert_eq!(documents["DisplayPath"], "Personal");
    assert_eq!(documents["PinStatus"], "True");
    // Notepad on Windows Server 2012 R2, a version 1 list: no interaction count is recorded: 0
    let notepad = row("9b9cdc69c1c24e2b.automaticDestinations-ms", "0");
    for (column, value) in [
        ("DestListVersion", "1"),
        ("EntryNumber", "5"),
        ("InteractionCount", "0"),
        ("Hostname", "citadel-dc01"),
        ("Path", r"C:\FileShare\Secret\Beth_Secret.txt"),
        ("LastModified", "2020-09-19T03:35:07.829277600Z"),
        ("NetworkPath", r"\\CITADEL-DC01\FileShare"),
        ("TargetMFTEntryNumber", "87111"),
    ] {
        assert_eq!(notepad[column], value, "{column}");
    }
    // A Windows 10 version 6 list that ends inside its last sector: entry 1 is a file staged
    // for a CD burn, named as the reference names it
    let burn = row("5f7b5f1e01b83767.automaticDestinations-ms", "36");
    assert_eq!(burn["DestListVersion"], "6");
    assert_eq!(burn["TargetIDAbsolutePath"], r"This PC\D:\autorun.inf");
    assert_eq!(burn["DriveType"], "DRIVE_CDROM");
    assert_eq!(
        rows.iter()
            .filter(|r| r["SourceFile"].ends_with("5f7b5f1e01b83767.automaticDestinations-ms"))
            .count(),
        37
    );

    let (header, rows) = csv_rows(&out.join("CustomDestinations.csv"));
    assert_eq!(header[6], "EntryName");
    assert_eq!(header.len(), 33);
    assert_eq!(rows.len(), 8);
    let chrome: Vec<_> = rows
        .iter()
        .filter(|r| r["SourceFile"].ends_with("5d696d521de238c3.customDestinations-ms"))
        .collect();
    let names: Vec<&str> = chrome.iter().map(|r| r["EntryName"].as_str()).collect();
    assert_eq!(
        names,
        [
            "Most visited",
            "Most visited",
            "Recently closed",
            "Recently closed",
            "Tasks",
            "Tasks",
            "",
            ""
        ]
    );
    assert_eq!(
        chrome[6]["Notes"],
        "Left over after the last category, at offset 11472"
    );
    assert_eq!(
        chrome[7]["Arguments"],
        "--win-jumplist-action=recently-closed http://192.168.50.33:3000/"
    );
    assert_eq!(chrome[5]["Arguments"], "--incognito");
}

#[test]
fn jump_list_json_and_jsonl() {
    let r = run(&[
        "parse-file",
        "jump-lists",
        "-f",
        EXPLORER_WIN10,
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(!r.stdout.contains("Results Summary"));
    let v: Value = serde_json::from_str(r.stdout.trim()).unwrap();
    assert_eq!(v["type"], "automatic");
    assert_eq!(v["app_id"], "f01b4d95cf55d32a");
    assert_eq!(v["dest_list"]["version"], 4);
    assert_eq!(v["dest_list"]["last_entry_number"], 9);
    let entries = v["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 9);
    assert_eq!(entries[3]["stream"], "7");
    assert_eq!(entries[3]["interaction_count"], 1);
    assert_eq!(
        entries[3]["link"]["target_path"],
        r"This PC\C:\Users\AndrewRathbun\Desktop\Targets"
    );
    assert_eq!(
        entries[3]["link"]["tracker"]["machine_id"],
        "desktop-gnccvrk"
    );
    assert_eq!(v["streams"].as_array().unwrap().len(), 10);
    assert_eq!(v["problems"].as_array().unwrap().len(), 0);

    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("jl.json");
    let r = run(&[
        "parse-file",
        "jump-lists",
        "-f",
        CHROME_WIN11,
        "-t",
        "json",
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    let docs = json_documents(&std::fs::read_to_string(&out).unwrap());
    assert_eq!(docs.len(), 1);
    let chrome = &docs[0];
    assert_eq!(chrome["type"], "custom");
    assert_eq!(chrome["format_version"], 2);
    let categories = chrome["categories"].as_array().unwrap();
    assert_eq!(categories[0]["name"], "Most visited");
    assert_eq!(categories[2]["kind"], "tasks");
    assert_eq!(chrome["trailing_bytes"], 3250);
    assert_eq!(chrome["leftover"][0]["offset"], 11472);
    assert_eq!(
        chrome["leftover"][0]["link"]["arguments"],
        "--win-jumplist-action=most-visited http://192.168.50.33:5000/"
    );
}

/// `data` with the compound file directory entry of stream `from` renamed `to` (names of the
/// same length or longer, up to 31 characters).
fn rename_stream(data: &mut [u8], from: &str, to: &str) {
    let name = |s: &str| -> Vec<u8> {
        s.encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect()
    };
    let (from, to) = (name(from), name(to));
    // Directory entries are 128 bytes, sector-aligned: the name, its size in bytes at 64, the
    // type at 66 (2 = stream).
    let at = (0..data.len() - 128)
        .step_by(128)
        .find(|&o| {
            data[o..o + from.len()] == from[..]
                && data[o + 64..o + 66] == (from.len() as u16).to_le_bytes()
                && data[o + 66] == 2
        })
        .unwrap();
    data[at..at + 64].fill(0);
    data[at..at + to.len()].copy_from_slice(&to);
    data[at + 64..at + 66].copy_from_slice(&(to.len() as u16).to_le_bytes());
}

#[test]
fn jump_list_streams_no_entry_names() {
    // Explorer's list with entry 7's stream renamed: entry 7 has no link, stream 99 no entry.
    let dir = tempfile::tempdir().unwrap();
    let mut data = std::fs::read(repo(EXPLORER_WIN10)).unwrap();
    rename_stream(&mut data, "7", "99");
    let file = dir.path().join("f01b4d95cf55d32a.automaticDestinations-ms");
    std::fs::write(&file, &data).unwrap();
    let out = dir.path().join("jl");
    let r = run(&[
        "parse-file",
        "jump-lists",
        "-f",
        file.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "entry 7 has no stream",
        "With problems: 1",
        "Link streams no DestList entry names (removed or damaged entries): 1",
        r"(stream 99) -> C:\Users\AndrewRathbun\Desktop\Targets",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let (_, rows) = csv_rows(&out.join("AutomaticDestinations.csv"));
    assert_eq!(rows.len(), 10);
    let entry = rows.iter().find(|r| r["EntryNumber"] == "7").unwrap();
    assert_eq!(entry["Path"], r"C:\Users\AndrewRathbun\Desktop\Targets");
    assert_eq!(entry["LocalPath"], "");
    // a link stream no entry names
    let unlisted = rows.iter().find(|r| r["EntryNumber"] == "99").unwrap();
    assert_eq!(unlisted["MRU"], "");
    assert_eq!(unlisted["Notes"], "Found in Directory, not DestList");
    assert_eq!(
        unlisted["LocalPath"],
        r"C:\Users\AndrewRathbun\Desktop\Targets"
    );
    assert_eq!(unlisted["Path"], "");
}

/// `data` with the one occurrence of `from` (a little-endian u64) replaced with `to`.
fn replace_u64(data: &mut [u8], from: u64, to: u64) {
    let from = from.to_le_bytes();
    let found: Vec<usize> = (0..data.len() - 8)
        .filter(|&o| data[o..o + 8] == from)
        .collect();
    assert_eq!(found.len(), 1);
    data[found[0]..found[0] + 8].copy_from_slice(&to.to_le_bytes());
}

#[test]
fn jump_list_times_that_are_not_times() {
    // Explorer's most recent entry last used at 2^63 (100 ns units), past any time: empty, no
    // panic (it overflowed the conversion)
    let dir = tempfile::tempdir().unwrap();
    let mut data = std::fs::read(repo(EXPLORER_WIN10)).unwrap();
    // 2022-02-08T20:44:20.6000878Z
    replace_u64(&mut data, 0x01d8_1d2c_a5de_3aee, 0x8000_0000_0000_0000);
    let file = dir.path().join("f01b4d95cf55d32a.automaticDestinations-ms");
    std::fs::write(&file, &data).unwrap();
    let r = run(&[
        "parse-file",
        "jump-lists",
        "-f",
        file.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let v: Value = serde_json::from_str(r.stdout.trim()).unwrap();
    assert_eq!(v["entries"][0]["last_used"], "");
    assert_eq!(
        v["entries"][1]["last_used"],
        "2022-02-06T22:09:00.177299400Z"
    );
    let out = dir.path().join("jl");
    let r = run(&[
        "parse-file",
        "jump-lists",
        "-f",
        file.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    let (_, rows) = csv_rows(&out.join("AutomaticDestinations.csv"));
    assert_eq!(rows[0]["LastModified"], "");
    assert_eq!(
        rows[0]["Path"],
        r"C:\temp\tout\c\ProgramData\Microsoft\search\data\applications\windows"
    );
}

#[test]
fn jump_list_input_and_output_errors() {
    let r = run(&[
        "parse-file",
        "jump-lists",
        "-f",
        "test_files/jump_lists/missing",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("The file does not exist"), "{}", r.stdout);

    let r = run(&[
        "parse-file",
        "jump-lists",
        "-f",
        "test_files/lnk/README.md",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("Could not parse") && r.stdout.contains("not a custom jump list"),
        "{}",
        r.stdout
    );

    let r = run(&["parse-file", "jump-lists", "-d", "test_files/lnk", "-K"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("No jump lists"), "{}", r.stdout);

    let r = run(&[
        "parse-file",
        "jump-lists",
        "-f",
        EXPLORER_WIN10,
        "--code-page",
        "12345",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("Code page 12345 is not supported"),
        "{}",
        r.stdout
    );

    // CSV tables need an empty directory (or -C); JSON a new file
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("keep"), b"keep").unwrap();
    let r = run(&[
        "parse-file",
        "jump-lists",
        "-f",
        EXPLORER_WIN10,
        "-o",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("already exists"), "{}", r.stdout);
    let file = dir.path().join("keep");
    let r = run(&[
        "parse-file",
        "jump-lists",
        "-f",
        EXPLORER_WIN10,
        "-o",
        file.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("is a file, not a directory"),
        "{}",
        r.stdout
    );
}

const SHIMCACHE_DIR: &str = "test_files/shimcache";

/// The Windows 11 SYSTEM hive of `test_files/shimcache/` and its transaction logs, unpacked
/// where a collection has them.
fn system_hive() -> tempfile::TempDir {
    use std::io::Read as _;
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("HOST/C/Windows/System32/config");
    std::fs::create_dir_all(&config).unwrap();
    for name in ["SYSTEM", "SYSTEM.LOG1", "SYSTEM.LOG2"] {
        let gz = std::fs::File::open(repo(&format!("{SHIMCACHE_DIR}/{name}.gz"))).unwrap();
        let mut bytes = Vec::new();
        flate2::read::GzDecoder::new(gz)
            .read_to_end(&mut bytes)
            .unwrap();
        std::fs::write(config.join(name), bytes).unwrap();
    }
    dir
}

#[test]
fn shimcache_summary_on_the_terminal() {
    let hive = system_hive();
    let r = run(&[
        "parse-registry",
        "shimcache",
        "-d",
        hive.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for line in [
        "SYSTEM hives: 1",
        "SYSTEM hives parsed: 1",
        "Dirty hives recovered with their transaction logs: 1",
        "Caches: 1 (Windows10C_11 1)",
        "Entries: 359 (executed 30, not executed 329, no execution flag 0)",
        "Duplicates (in an earlier control set): 0",
        "Recovered from slack: 0",
        "Last-modified times: 2021-05-20T08:51:57.791182Z - 2022-02-07T07:37:13.434150800Z",
        "Damaged entries: 0",
        "Findings: 27 (medium: 27)",
        r"  [MEDIUM] SHIMCACHE-SUSPICIOUS-PATH C:\Users\ANDREW~1\AppData\Local\Temp\DA15D340-2806-4EF5-BB3C-C787E4CD384B\dismhost.exe",
        "  … 17 more in the Findings column",
        "To save every entry, add -o <FILE>",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    assert!(!r.stdout.contains("were not applied"), "{}", r.stdout);

    // the hive as it is: dirty, its logs not applied (they don't change this cache)
    let r = run(&[
        "parse-registry",
        "shimcache",
        "-d",
        hive.path().to_str().unwrap(),
        "--no-logs",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for line in [
        "Dirty hives recovered with their transaction logs: 0",
        "Dirty hives whose transaction logs were not applied (entries may be missing): 1",
        "Entries: 359 (executed 30",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
}

/// Changes one byte of a file in place.
fn flip_byte(path: &Path, at: usize) {
    let mut bytes = std::fs::read(path).unwrap();
    bytes[at] ^= 0xFF;
    std::fs::write(path, bytes).unwrap();
}

#[test]
fn shimcache_hive_with_a_damaged_base_block_is_recovered_from_its_logs() {
    let hive = system_hive();
    let config = hive.path().join("HOST/C/Windows/System32/config");
    // no `regf` signature: the base block comes from the log with the latest entries
    flip_byte(&config.join("SYSTEM"), 0);
    let r = run(&[
        "parse-registry",
        "shimcache",
        "-d",
        hive.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "SYSTEM hives parsed: 1",
        "Dirty hives recovered with their transaction logs: 1",
        "Entries: 359 (executed 30, not executed 329, no execution flag 0)",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
}

#[test]
fn shimcache_logs_whose_entries_are_damaged_are_not_applied() {
    let hive = system_hive();
    let config = hive.path().join("HOST/C/Windows/System32/config");
    // a byte of each log's first entry: its hashes no longer match, so nothing can be applied
    for log in ["SYSTEM.LOG1", "SYSTEM.LOG2"] {
        flip_byte(&config.join(log), 512 + 100);
    }
    let r = run(&[
        "parse-registry",
        "shimcache",
        "-d",
        hive.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "Dirty hives recovered with their transaction logs: 0",
        "Dirty hives whose transaction logs were not applied (entries may be missing): 1",
        "Entries: 359",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
}

#[test]
fn shimcache_csv_has_the_common_columns() {
    let hive = system_hive();
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("shimcache.csv");
    let r = run(&[
        "parse-registry",
        "shimcache",
        "-d",
        hive.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    let (header, rows) = csv_rows(&out);
    let common_columns = [
        "ControlSet",
        "CacheEntryPosition",
        "Path",
        "LastModifiedTimeUTC",
        "Executed",
        "Duplicate",
        "SourceFile",
        "FromSlack",
    ];
    assert_eq!(&header[..8], common_columns);
    assert_eq!(header.len(), 28);
    // every entry as the reference writes it (its times to the second)
    let (_, expected) = csv_rows(&repo(&format!("{SHIMCACHE_DIR}/SYSTEM.expected.csv")));
    assert_eq!(rows.len(), 359);
    assert_eq!(rows.len(), expected.len());
    for (ours, theirs) in rows.iter().zip(&expected) {
        for column in common_columns {
            let value = match column {
                "LastModifiedTimeUTC" => {
                    ours[column].get(..19).unwrap_or_default().replace('T', " ")
                }
                "SourceFile" => {
                    assert!(ours[column].ends_with("SYSTEM"));
                    continue;
                }
                _ => ours[column].clone(),
            };
            assert_eq!(value, theirs[column], "{column}: {ours:?}");
        }
    }
    let first = &rows[0];
    for (column, value) in [
        ("CacheFormat", "Windows10C_11"),
        ("CacheLastWrittenUTC", "2022-02-07T07:37:59.317450900Z"),
        ("LastModifiedTimeUTC", "2022-02-07T07:37:13.434150800Z"),
        ("DataSize", "72"),
        ("Machine", "x64"),
        ("InsertFlags", ""),
        ("Findings", ""),
        (
            "RegistryPath",
            r"ControlSet001\Control\Session Manager\AppCompatCache",
        ),
        ("RegistryValueName", "AppCompatCache"),
        // the whole cache, not the entry's data
        ("CacheValueSize", "80260"),
        ("IsCurrent", "True"),
        ("SelectCurrent", "1"),
        ("SelectDefault", "1"),
        ("SelectLastKnownGood", "1"),
        ("SelectFailed", "0"),
    ] {
        assert_eq!(first[column], value, "{column}");
    }
    let flagged: Vec<_> = rows
        .iter()
        .filter(|r| r["Findings"] == "SHIMCACHE-SUSPICIOUS-PATH")
        .collect();
    assert_eq!(flagged.len(), 27);
}

/// regipy's 32-bit Windows 7 SYSTEM hive: two control sets, `Select\Current` 1 and
/// `LastKnownGood` 2.
fn regipy_hive() -> tempfile::TempDir {
    use std::io::Read as _;
    let dir = tempfile::tempdir().unwrap();
    let gz = std::fs::File::open(repo(&format!("{SHIMCACHE_DIR}/regipy/SYSTEM.gz"))).unwrap();
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(gz)
        .read_to_end(&mut bytes)
        .unwrap();
    std::fs::write(dir.path().join("SYSTEM"), bytes).unwrap();
    dir
}

#[test]
fn shimcache_select_and_the_size_of_each_value() {
    let hive = regipy_hive();
    let system = hive.path().join("SYSTEM");
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("shimcache.csv");
    let r = run(&[
        "parse-registry",
        "shimcache",
        "-f",
        system.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(!r.stdout.contains("Select missing"), "{}", r.stdout);
    let (_, rows) = csv_rows(&out);
    assert_eq!(rows.len(), 660);
    // only Current's control set is current, not LastKnownGood's
    for (set, current) in [("1", "True"), ("2", "False")] {
        let of_set: Vec<_> = rows.iter().filter(|r| r["ControlSet"] == set).collect();
        assert_eq!(of_set.len(), 330, "{set}");
        for row in of_set {
            assert_eq!(row["IsCurrent"], current, "{set}");
            assert_eq!(row["CacheValueSize"], "56256", "{set}");
            assert_eq!(
                row["RegistryPath"],
                format!(r"ControlSet00{set}\Control\Session Manager\AppCompatCache")
            );
            assert_eq!(row["SelectCurrent"], "1");
            assert_eq!(row["SelectLastKnownGood"], "2");
        }
    }
    // the same in JSON Lines
    let r = run(&[
        "parse-registry",
        "shimcache",
        "-f",
        system.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let entries: Vec<Value> = r
        .stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let second = entries.iter().find(|e| e["control_set"] == 2).unwrap();
    assert_eq!(second["is_current"], false);
    assert_eq!(second["cache_value_size"], 56_256);
    assert!(second["data_size"].as_u64().unwrap() < 56_256);
    assert_eq!(second["registry_value_name"], "AppCompatCache");
    assert_eq!(
        (
            second["select_current"].as_u64(),
            second["select_default"].as_u64(),
            second["select_last_known_good"].as_u64(),
            second["select_failed"].as_u64()
        ),
        (Some(1), Some(1), Some(2), Some(0))
    );
}

#[test]
fn shimcache_values_as_json() {
    // AppCompatCache values saved on their own: Windows XP, 7 (32-bit), Server 2012 R2 (its
    // first dword 0) and 11
    let r = run(&[
        "parse-registry",
        "shimcache",
        "-f",
        &format!("{SHIMCACHE_DIR}/WinXPx86.bin"),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(!r.stdout.contains("Results Summary"));
    let entries: Vec<Value> = r
        .stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(entries.len(), 17);
    let xp = &entries[0];
    assert_eq!(xp["cache_format"], "WindowsXP");
    assert_eq!(xp["control_set"], Value::Null);
    assert_eq!(xp["path"], r"C:\WINDOWS\system32\wscntfy.exe");
    assert_eq!(xp["last_modified"], "2008-04-14T12:00:00Z");
    assert_eq!(xp["executed"], Value::Null);
    assert_eq!(xp["file_size"], 13824);
    // a value on its own: its size is the file's, and no control set is known to be current
    assert_eq!(xp["cache_value_size"], 53_392);
    assert_eq!(xp["is_current"], Value::Null);
    assert_eq!(xp["registry_path"], Value::Null);
    assert_eq!(xp["select_current"], Value::Null);
    assert_eq!(xp["last_update"], "2016-01-13T22:20:03.265625Z");

    for (file, count, executed, format, path) in [
        (
            "regipy_win7x86.bin",
            330,
            238,
            "Windows7x86",
            r"C:\Program Files\McAfee\VirusScan Enterprise\mfeann.exe",
        ),
        (
            "Win2012R2-StolenSzechuan-ControlSet001.bin",
            140,
            101,
            "Windows81_Windows2012R2",
            r"SYSVOL\Windows\System32\LogonUI.exe",
        ),
        (
            "win11shim.raw",
            2,
            0,
            "Windows10C_11",
            r"C:\WINDOWS\system32\wbem\WmiApSrv.exe",
        ),
    ] {
        let r = run(&[
            "parse-registry",
            "shimcache",
            "-f",
            &format!("{SHIMCACHE_DIR}/{file}"),
            "-t",
            "json",
        ]);
        assert_eq!(r.code, 0, "{}", r.stderr);
        let entries = json_documents(&r.stdout);
        assert_eq!(entries.len(), count, "{file}");
        let ran = entries.iter().filter(|e| e["executed"] == true).count();
        assert_eq!(ran, executed, "{file}");
        assert_eq!(entries[0]["cache_format"], format, "{file}");
        assert_eq!(entries[0]["path"], path, "{file}");
    }
    let r = run(&[
        "parse-registry",
        "shimcache",
        "-f",
        &format!("{SHIMCACHE_DIR}/win11shim.raw"),
        "-t",
        "jsonl",
    ]);
    let w11: Value = serde_json::from_str(r.stdout.lines().next().unwrap()).unwrap();
    assert_eq!(w11["executed"], false);
    assert_eq!(w11["machine"], "x64");
    assert_eq!(w11["data_size"], 136);
    assert_eq!(w11["last_modified"], "2022-06-18T22:26:50.670515700Z");

    // the summary of a value
    let r = run(&[
        "parse-registry",
        "shimcache",
        "-f",
        &format!("{SHIMCACHE_DIR}/WinXPx86.bin"),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for line in [
        "AppCompatCache values parsed: 1",
        "Caches: 1 (WindowsXP 1)",
        "Entries: 17 (executed 0, not executed 0, no execution flag 17)",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    assert!(!r.stdout.contains("SYSTEM hives parsed"), "{}", r.stdout);
}

#[test]
fn shimcache_input_and_output_errors() {
    let r = run(&[
        "parse-registry",
        "shimcache",
        "-f",
        "test_files/shimcache/missing",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("The file does not exist"), "{}", r.stdout);

    let r = run(&[
        "parse-registry",
        "shimcache",
        "-f",
        "test_files/lnk/README.md",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("Could not parse")
            && r.stdout
                .contains("neither a registry hive nor an AppCompatCache value"),
        "{}",
        r.stdout
    );

    let r = run(&["parse-registry", "shimcache", "-d", "test_files/lnk", "-K"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("No SYSTEM hives"), "{}", r.stdout);

    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("keep.csv");
    std::fs::write(&file, b"keep").unwrap();
    let value = format!("{SHIMCACHE_DIR}/regipy_win7x86.bin");
    let r = run(&[
        "parse-registry",
        "shimcache",
        "-f",
        &value,
        "-o",
        file.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("already exists"), "{}", r.stdout);
    let r = run(&[
        "parse-registry",
        "shimcache",
        "-f",
        &value,
        "-o",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("is a directory"), "{}", r.stdout);
}

// --- parse-file defender-quarantine ---------------------------------------------------------------

const QUARANTINE_DIR: &str = "test_files/defender_quarantine";
const QUARANTINED_FILE: &str = "AAB535B2BF6E319A41A914FDF71F7509D42E847F";
const QUARANTINED_HIVE: &str = "E2600EF2CC97428CF2377419502B3D3330483F01";
const ORPHAN_CONTENT: &str = "5CA5633C0E2AE818B85C5695677170EA42D55131";
const FILE_SHA256: &str = "828804761c959b70327e6b05c4cab83c30aafdcc193bc29209d41b15e6d9c358";

/// A copy of the test quarantine folder at `dir/<host>/C/ProgramData/Microsoft/Windows
/// Defender/Quarantine`.
fn quarantine_host(dir: &Path, host: &str) -> PathBuf {
    let root = dir
        .join(host)
        .join("C/ProgramData/Microsoft/Windows Defender/Quarantine");
    let from = repo(&format!("{QUARANTINE_DIR}/Quarantine"));
    let mut stack = vec![from.clone()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let target = root.join(e.path().strip_prefix(&from).unwrap());
            if e.path().is_dir() {
                std::fs::create_dir_all(&target).unwrap();
                stack.push(e.path());
            } else {
                std::fs::create_dir_all(target.parent().unwrap()).unwrap();
                std::fs::copy(e.path(), target).unwrap();
            }
        }
    }
    root
}

fn quarantine_csv(args: &[&str]) -> (Run, Vec<std::collections::HashMap<String, String>>) {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("quarantine.csv");
    let mut argv = vec!["parse-file", "defender-quarantine", "-K"];
    argv.extend_from_slice(args);
    argv.extend(["-o", out.to_str().unwrap()]);
    let r = run(&argv);
    let rows = if out.exists() {
        let mut reader = csv::Reader::from_path(&out).unwrap();
        let header: Vec<String> = reader
            .headers()
            .unwrap()
            .iter()
            .map(str::to_owned)
            .collect();
        reader
            .records()
            .map(|row| {
                header
                    .iter()
                    .cloned()
                    .zip(row.unwrap().iter().map(str::to_owned))
                    .collect()
            })
            .collect()
    } else {
        Vec::new()
    };
    (r, rows)
}

#[test]
fn defender_quarantine_summary_on_the_terminal() {
    let r = run(&[
        "parse-file",
        "defender-quarantine",
        "-K",
        "-d",
        QUARANTINE_DIR,
        "-q",
    ]);
    assert_eq!(r.code, 0, "{}{}", r.stdout, r.stderr);
    for line in [
        "Quarantine folders: 1",
        "Quarantine entries parsed: 2",
        "Resources quarantined: 3 (file: 1, service: 1, startup: 1)",
        "Contents or records no entry holds: 1",
        "HackTool:Win32/Example!MSR (1)",
        "Trojan:Win32/Example.A (1)",
    ] {
        assert!(r.stdout.contains(line), "{line}: {}", r.stdout);
    }
    assert!(!r.stdout.contains("WARNING"), "{}", r.stdout);
}

#[test]
fn defender_quarantine_csv_has_every_resource() {
    let (r, rows) = quarantine_csv(&["-d", QUARANTINE_DIR]);
    assert_eq!(r.code, 0, "{}{}", r.stdout, r.stderr);
    let kinds: Vec<(&str, &str)> = rows
        .iter()
        .map(|row| (row["Record"].as_str(), row["ResourceType"].as_str()))
        .collect();
    assert_eq!(
        kinds,
        [
            ("resource", "file"),
            ("resource", "service"),
            ("resource", "startup"),
            ("orphan-content", "")
        ]
    );
    let file = &rows[0];
    for (column, value) in [
        ("QuarantineTime", "2023-12-19T09:36:40.1234567Z"),
        ("ThreatName", "HackTool:Win32/Example!MSR"),
        ("ThreatId", "2147700000"),
        ("Path", r"\\?\C:\Users\alice\Downloads\tool\tool.exe"),
        ("PhysicalPath", r"C:\Users\alice\Downloads\tool\tool.exe"),
        ("FileSize", "61"),
        ("CreationTime", "2023-12-19T09:29:59.0000000Z"),
        ("LastAccessTime", "2023-12-19T09:36:30.5000000Z"),
        ("LastWriteTime", "2023-12-19T09:30:00.0000000Z"),
        ("Attributes", "0x20"),
        ("Owner", "S-1-5-21-1111-2222-3333-1001"),
        ("ZoneId", "3"),
        ("HostUrl", "https://example.com/tool.zip"),
        ("ContentSize", "61"),
        ("SHA256", FILE_SHA256),
        (
            "Streams",
            "SecurityData, Data, AlternateData :Zone.Identifier:$DATA",
        ),
        ("ResourceId", QUARANTINED_FILE),
        ("EntryId", "80012345-0000-0000-1A2B-3C4D5E6F7081"),
        ("ScanId", "6B1E0C2D-3F4A-4B5C-8D9E-0F1A2B3C4D5E"),
        ("ExtractedFile", ""),
        ("Problems", ""),
    ] {
        assert_eq!(file[column], value, "{column}");
    }
    assert!(file["ResourceRecordFile"].ends_with(QUARANTINED_FILE));
    let service = &rows[1];
    assert_eq!(service["Streams"], "Hive");
    assert_eq!(service["ContentSize"], "4096");
    assert_eq!(service["ResourceId"], QUARANTINED_HIVE);
    assert_eq!(rows[2]["ResourceId"], "");
    let orphan = &rows[3];
    assert_eq!(orphan["ResourceId"], ORPHAN_CONTENT);
    assert_eq!(orphan["Problems"], "no entry holds this content");
}

#[test]
fn defender_quarantine_json_and_jsonl() {
    let dir = tempfile::tempdir().unwrap();
    let json_out = dir.path().join("q.json");
    let r = run(&[
        "parse-file",
        "defender-quarantine",
        "-K",
        "-d",
        QUARANTINE_DIR,
        "-t",
        "json",
        "-o",
        json_out.to_str().unwrap(),
    ]);
    assert_eq!(r.code, 0, "{}{}", r.stdout, r.stderr);
    let docs = json_documents(&std::fs::read_to_string(&json_out).unwrap());
    assert_eq!(docs.len(), 4);
    let file = &docs[0];
    assert_eq!(file["record"], "resource");
    assert_eq!(file["resource_type"], "file");
    assert_eq!(file["fields"].as_array().unwrap().len(), 8);
    assert_eq!(file["fields"][0]["name"], "ResourceId");
    assert_eq!(file["content"]["sha256"], FILE_SHA256);
    assert_eq!(file["content"]["streams"].as_array().unwrap().len(), 3);
    assert_eq!(file["content"]["zone_identifier"]["ZoneId"], "3");
    assert_eq!(
        file["resource_record_entries"],
        serde_json::json!(["80012345-0000-0000-1A2B-3C4D5E6F7081"])
    );
    // JSONL on stdout: the data alone
    let r = run(&[
        "parse-file",
        "defender-quarantine",
        "-K",
        "-d",
        QUARANTINE_DIR,
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let lines: Vec<Value> = r
        .stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[3]["record"], "orphan-content");
}

#[test]
fn defender_quarantine_hosts_damage_and_missing_contents() {
    let dir = tempfile::tempdir().unwrap();
    quarantine_host(dir.path(), "HOST1");
    let host2 = quarantine_host(dir.path(), "HOST2");
    // HOST2: the file's content gone, the service's entry changed, and a file that isn't an entry
    std::fs::remove_file(host2.join(format!(
        "ResourceData/{}/{QUARANTINED_FILE}",
        &QUARANTINED_FILE[..2]
    )))
    .unwrap();
    let service_entry = host2.join("Entries/{80054321-0000-0000-9A8B-7C6D5E4F3021}");
    let mut bytes = std::fs::read(&service_entry).unwrap();
    bytes[70] ^= 1;
    std::fs::write(&service_entry, bytes).unwrap();
    std::fs::write(host2.join("Entries/notes.txt"), b"not an entry").unwrap();

    let (r, rows) = quarantine_csv(&["-d", dir.path().to_str().unwrap()]);
    assert_eq!(r.code, 0, "{}{}", r.stdout, r.stderr);
    assert!(r.stdout.contains("Quarantine folders: 2"), "{}", r.stdout);
    assert!(r.stdout.contains("Could not parse: 1"), "{}", r.stdout);
    let host = |row: &std::collections::HashMap<String, String>| {
        if row["QuarantineFolder"].contains("HOST1") {
            1
        } else {
            2
        }
    };
    let host1: Vec<_> = rows.iter().filter(|r| host(r) == 1).collect();
    let host2_rows: Vec<_> = rows.iter().filter(|r| host(r) == 2).collect();
    assert_eq!(host1.len(), 4);
    assert!(
        host1
            .iter()
            .all(|r| r["Problems"].is_empty() || r["Record"] == "orphan-content")
    );
    let file = host2_rows
        .iter()
        .find(|r| r["ResourceType"] == "file")
        .unwrap();
    assert!(
        file["Problems"].starts_with("its content isn't there (ResourceData"),
        "{}",
        file["Problems"]
    );
    assert_eq!(file["SHA256"], "");
    let service = host2_rows
        .iter()
        .find(|r| r["ResourceType"] == "service")
        .unwrap();
    assert!(
        service["Problems"].contains("CRC-32"),
        "{}",
        service["Problems"]
    );
    // the file's resource record is still there, beside the missing content
    assert!(file["ResourceRecordFile"].ends_with(QUARANTINED_FILE));
    assert!(host2_rows.iter().all(|r| r["Record"] != "orphan-record"));
}

#[test]
fn defender_quarantine_extracts_only_when_asked() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("extracted");
    let report = dir.path().join("quarantine.csv");
    let r = run(&[
        "parse-file",
        "defender-quarantine",
        "-K",
        "-d",
        QUARANTINE_DIR,
        "--extract",
        out.to_str().unwrap(),
        "-o",
        report.to_str().unwrap(),
    ]);
    assert_eq!(r.code, 0, "{}{}", r.stdout, r.stderr);
    assert!(r.stdout.contains("WARNING: --extract"), "{}", r.stdout);
    let mut names: Vec<String> = std::fs::read_dir(&out)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            format!("{ORPHAN_CONTENT}.quarantined"),
            format!("{QUARANTINED_HIVE}.hive.quarantined"),
            "tool.exe.Zone.Identifier.quarantined".to_string(),
            "tool.exe.quarantined".to_string(),
            "tool.exe.security.quarantined".to_string(),
            "tool.exe.security_descriptor.txt".to_string(),
        ]
    );
    assert!(
        names.iter().all(
            |name| name.ends_with(".quarantined") || name.ends_with(".security_descriptor.txt")
        )
    );
    let security_file = out.join("tool.exe.security_descriptor.txt");
    let security_text = std::fs::read_to_string(&security_file).unwrap();
    assert!(security_text.contains("Binary descriptor: tool.exe.security.quarantined"));
    assert!(security_text.contains("Owner: S-1-5-21-1111-2222-3333-1001"));
    assert!(security_text.contains("SDDL (owner, group and ACL entries):"));
    assert!(security_text.contains("Decode status: complete"));
    let file = out.join("tool.exe.quarantined");
    let data = std::fs::read(&file).unwrap();
    assert!(data.starts_with(b"A harmless line of text"));
    assert_eq!(data.len(), 61);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&file).unwrap().permissions().mode();
        assert_eq!(mode & 0o111, 0, "executable: {mode:o}");
        let mode = std::fs::metadata(&security_file)
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "text report permissions: {mode:o}");
    }
    assert!(r.stdout.contains("Files extracted: 6"), "{}", r.stdout);
    let (_, rows) = csv_rows(&report);
    let extracted = rows
        .iter()
        .find(|row| row["ResourceId"] == QUARANTINED_FILE)
        .unwrap();
    assert_eq!(extracted["ExtractedFile"], file.to_string_lossy());

    // Again: the same files are there, nothing written.
    let r = run(&[
        "parse-file",
        "defender-quarantine",
        "-K",
        "-d",
        QUARANTINE_DIR,
        "--extract",
        out.to_str().unwrap(),
    ]);
    assert!(r.stdout.contains("Files extracted: 0"), "{}", r.stdout);
    assert!(r.stdout.contains("6 already there"), "{}", r.stdout);

    // A file holding something else isn't overwritten, unless -C.
    std::fs::write(&file, b"something else").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let (r, rows) = quarantine_csv(&["-d", QUARANTINE_DIR, "--extract", out.to_str().unwrap()]);
    assert!(r.stdout.contains("1 not overwritten"), "{}", r.stdout);
    assert!(
        rows[0]["Problems"].contains("not overwritten"),
        "{:?}",
        rows[0]
    );
    assert_eq!(std::fs::read(&file).unwrap(), b"something else");
    let dir2 = tempfile::tempdir().unwrap();
    let csv = dir2.path().join("q.csv");
    let r = run(&[
        "parse-file",
        "defender-quarantine",
        "-K",
        "-d",
        QUARANTINE_DIR,
        "--extract",
        out.to_str().unwrap(),
        "-o",
        csv.to_str().unwrap(),
        "-C",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert_eq!(std::fs::read(&file).unwrap(), data);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&file).unwrap().permissions().mode();
        assert_eq!(mode & 0o111, 0, "executable after -C: {mode:o}");
    }
}

#[test]
fn defender_security_text_obeys_overwrite_protection() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("extracted");
    let (r, _) = quarantine_csv(&["-d", QUARANTINE_DIR, "--extract", out.to_str().unwrap()]);
    assert_eq!(r.code, 0, "{}{}", r.stdout, r.stderr);
    let text = out.join("tool.exe.security_descriptor.txt");
    let expected = std::fs::read(&text).unwrap();
    let payload = std::fs::read(out.join("tool.exe.quarantined")).unwrap();
    let raw_security = std::fs::read(out.join("tool.exe.security.quarantined")).unwrap();
    std::fs::write(&text, b"analyst's existing file\n").unwrap();
    let (r, rows) = quarantine_csv(&["-d", QUARANTINE_DIR, "--extract", out.to_str().unwrap()]);
    assert!(r.stdout.contains("1 not overwritten"), "{}", r.stdout);
    assert!(rows.iter().any(|row| {
        row["Problems"].contains("tool.exe.security_descriptor.txt holds something else")
    }));
    assert_eq!(std::fs::read(&text).unwrap(), b"analyst's existing file\n");
    assert_eq!(
        std::fs::read(out.join("tool.exe.quarantined")).unwrap(),
        payload
    );
    assert_eq!(
        std::fs::read(out.join("tool.exe.security.quarantined")).unwrap(),
        raw_security
    );
    let (r, _) = quarantine_csv(&[
        "-d",
        QUARANTINE_DIR,
        "--extract",
        out.to_str().unwrap(),
        "-C",
    ]);
    assert_eq!(r.code, 0, "{}{}", r.stdout, r.stderr);
    assert_eq!(std::fs::read(&text).unwrap(), expected);
}

#[test]
fn defender_quarantine_input_and_output_errors() {
    let r = run(&[
        "parse-file",
        "defender-quarantine",
        "-K",
        "-d",
        "no/such/dir",
    ]);
    assert_eq!(r.code, 1);
    let empty = tempfile::tempdir().unwrap();
    let r = run(&[
        "parse-file",
        "defender-quarantine",
        "-K",
        "-d",
        empty.path().to_str().unwrap(),
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("No Defender quarantine folders"),
        "{}",
        r.stdout
    );
    // -C needs somewhere to write
    let r = run(&[
        "parse-file",
        "defender-quarantine",
        "-K",
        "-d",
        QUARANTINE_DIR,
        "-C",
    ]);
    assert_ne!(r.code, 0);
    // one entry with -f, and a file that isn't one
    let entry =
        format!("{QUARANTINE_DIR}/Quarantine/Entries/{{80012345-0000-0000-1A2B-3C4D5E6F7081}}");
    let (r, rows) = quarantine_csv(&["-f", &entry]);
    assert_eq!(r.code, 0, "{}{}", r.stdout, r.stderr);
    assert_eq!(rows.len(), 1, "no orphans with -f");
    assert_eq!(rows[0]["SHA256"], FILE_SHA256);
    let r = run(&[
        "parse-file",
        "defender-quarantine",
        "-K",
        "-f",
        "test_files/defender_quarantine/README.md",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("Could not parse"), "{}", r.stdout);
}

// --- parse-registry ----------------------------------------------------------------------------

const AMCACHE_HIVE: &str = "test_files/amcache/amcache_win10.hve";
/// `parse-registry read`'s CSV columns.
const REGISTRY_COLUMNS: [&str; 18] = [
    "Host",
    "HiveType",
    "SourcePath",
    "RecordType",
    "KeyPath",
    "KeyLastWrittenUTC",
    "ValueName",
    "ValueType",
    "ValueData",
    "DataSize",
    "ValueDataRaw",
    "ValueU64",
    "ValueFiletimeUTC",
    "AllocationStatus",
    "RecoveredFrom",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];
/// `parse-registry read`'s JSON fields, in their order.
const REGISTRY_FIELDS: [&str; 18] = [
    "host",
    "hive_type",
    "source_path",
    "record_type",
    "key_path",
    "key_last_written",
    "value_name",
    "value_type",
    "value_data",
    "data_size",
    "value_data_raw",
    "value_u64",
    "value_filetime_utc",
    "allocation_status",
    "recovered_from",
    "transaction_log_status",
    "logs_applied",
    "problems",
];

fn jsonl_records(text: &str) -> Vec<Value> {
    text.lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// `parse-registry read` with `args`, as JSON Lines.
fn registry_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "read"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_subcommands() {
    // parse-registry alone lists its subcommands
    let r = run(&["parse-registry"]);
    assert_ne!(r.code, 0);
    let help = format!("{}{}", r.stdout, r.stderr);
    assert!(help.contains("juntei parse-registry <COMMAND>"), "{help}");
    assert!(help.contains("read "), "{help}");
    assert!(help.contains("services "), "{help}");
    assert!(help.contains("recentdocs "), "{help}");
    assert!(help.contains("taskcache "), "{help}");
    assert!(help.contains("sam "), "{help}");
    assert!(help.contains("persistence "), "{help}");
    assert!(help.contains("userassist "), "{help}");
    let r = run(&["parse-registry", "services", "-h"]);
    assert_eq!(r.code, 0);
    for option in [
        "--service <NAME>",
        "--all-control-sets",
        "--log1 <FILE>",
        "--no-logs",
    ] {
        assert!(r.stdout.contains(option), "{option}\n{}", r.stdout);
    }
    let r = run(&["parse-registry", "read", "-h"]);
    assert_eq!(r.code, 0);
    for option in [
        "--key <PATH>",
        "--key-regex <REGEX>",
        "--value <NAME>",
        "--value-regex <REGEX>",
        "--data-regex <REGEX>",
        "--log1 <FILE>",
        "--log2 <FILE>",
        "--no-logs",
        "--raw-data",
        "--qword-as-filetime",
        "--recover-deleted",
    ] {
        assert!(r.stdout.contains(option), "{option}\n{}", r.stdout);
    }
}

#[test]
fn registry_every_key_and_value_of_a_hive() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("registry.csv");
    let r = run(&[
        "parse-registry",
        "read",
        "-f",
        AMCACHE_HIVE,
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "Registry hives parsed: 1",
        "Hive types: AMCACHE.HVE 1",
        // the test hive was copied from a running system, without its logs
        "Dirty hives whose transaction logs were not applied (keys or values may be missing): 1",
        "Keys: 470, values: 10,942",
        "Records: 10,944",
        "Damaged keys and values: 0",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let (header, rows) = csv_rows(&out);
    assert_eq!(header, REGISTRY_COLUMNS);
    assert_eq!(rows.len(), 10_944);
    // the keys without values, with their last-write times
    let keys: Vec<(&str, &str)> = rows
        .iter()
        .filter(|r| r["RecordType"] == "key")
        .map(|r| (r["KeyPath"].as_str(), r["KeyLastWrittenUTC"].as_str()))
        .collect();
    assert_eq!(
        keys,
        [
            ("Root", "2022-03-10T20:12:08.418697200Z"),
            (r"Root\DeviceCensus", "2022-03-10T16:34:26.058381300Z")
        ]
    );
    let value = |key: &str, name: &str| {
        rows.iter()
            .find(|r| r["KeyPath"] == key && r["ValueName"] == name)
            .unwrap()
    };
    let seven = r"Root\InventoryApplicationFile\7z.exe|afe683e0fa522625";
    for (column, expected) in [
        // an Amcache hive alone: no SYSTEM hive names its host
        ("Host", ""),
        ("HiveType", "AMCACHE.HVE"),
        ("SourcePath", AMCACHE_HIVE),
        ("RecordType", "value"),
        ("KeyLastWrittenUTC", "2022-03-10T19:49:31.658344400Z"),
        ("ValueType", "REG_SZ"),
        ("ValueData", "igor pavlov"),
        ("DataSize", "24"),
        ("ValueDataRaw", ""),
        ("ValueU64", ""),
        ("ValueFiletimeUTC", ""),
        ("AllocationStatus", "allocated"),
        ("TransactionLogStatus", "not_applied"),
        ("LogsApplied", ""),
        ("Problems", ""),
    ] {
        assert_eq!(value(seven, "Publisher")[column], expected, "{column}");
    }
    assert_eq!(value(seven, "Size")["ValueType"], "REG_QWORD");
    assert_eq!(value(seven, "Size")["ValueData"], "535040");
    assert_eq!(value(seven, "Size")["ValueU64"], "535040");
    // the root key's own values: an empty path
    assert_eq!(value("", "CreatingModule")["RecordType"], "value");

    // JSON Lines, the same every time, the fields in their order
    let first = run(&["parse-registry", "read", "-f", AMCACHE_HIVE, "-t", "jsonl"]);
    let again = run(&["parse-registry", "read", "-f", AMCACHE_HIVE, "-t", "jsonl"]);
    assert_eq!(first.code, 0, "{}", first.stderr);
    assert_eq!(first.stdout, again.stdout);
    let line = first.stdout.lines().next().unwrap();
    let at: Vec<usize> = REGISTRY_FIELDS
        .iter()
        .map(|f| line.find(&format!("\"{f}\":")).unwrap())
        .collect();
    assert!(at.is_sorted(), "{line}");
    assert_eq!(
        serde_json::from_str::<Value>(line)
            .unwrap()
            .as_object()
            .unwrap()
            .len(),
        REGISTRY_FIELDS.len()
    );
    let records = jsonl_records(&first.stdout);
    assert_eq!(records.len(), 10_944);
    let find = |name: &str| {
        records
            .iter()
            .find(|r| r["key_path"] == seven && r["value_name"] == name)
            .unwrap()
    };
    assert_eq!(find("Publisher")["value_data"], "igor pavlov");
    assert_eq!(find("Publisher")["host"], Value::Null);
    assert_eq!(find("Publisher")["transaction_log_status"], "not_applied");
    assert_eq!(find("Size")["value_data"], 535_040);
    assert_eq!(find("Size")["value_u64"], 535_040);
    assert_eq!(find("Size")["value_filetime_utc"], Value::Null);
    assert_eq!(
        find("Size")["key_last_written"],
        "2022-03-10T19:49:31.658344400Z"
    );
}

#[test]
fn registry_dirty_hive_with_its_logs() {
    let hive = system_hive();
    let system = hive.path().join("HOST/C/Windows/System32/config/SYSTEM");
    let system = system.to_str().unwrap();
    // found under -d: the hive, not its logs
    let r = run(&[
        "parse-registry",
        "read",
        "-d",
        hive.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "Registry hives: 1",
        "Hive types: SYSTEM 1",
        "Hosts: GOLDPAPERW11",
        "Dirty hives recovered with their transaction logs: 1",
        "Keys: 32,881, values: 75,061",
        "Records: 81,006 (replayed from the transaction logs: 253)",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    // the control sets Windows chose, the path given as in regedit
    let select: Vec<(String, u64)> =
        registry_read(&["-f", system, "--key", r"HKEY_LOCAL_MACHINE\SYSTEM\Select"])
            .iter()
            .map(|v| {
                assert_eq!(v["host"], "GOLDPAPERW11");
                assert_eq!(v["value_type"], "REG_DWORD");
                assert_eq!(v["transaction_log_status"], "unchanged");
                assert_eq!(v["logs_applied"], json!(["SYSTEM.LOG1", "SYSTEM.LOG2"]));
                assert_eq!(v["value_data"], v["value_u64"]);
                (
                    v["value_name"].as_str().unwrap().to_string(),
                    v["value_u64"].as_u64().unwrap(),
                )
            })
            .collect();
    let select: Vec<(&str, u64)> = select.iter().map(|(n, v)| (n.as_str(), *v)).collect();
    assert_eq!(
        select,
        [
            ("Current", 1),
            ("Default", 1),
            ("Failed", 0),
            ("LastKnownGood", 1)
        ]
    );
    // CurrentControlSet: the control set Select\Current names
    let name = registry_read(&[
        "-f",
        system,
        "--key",
        r"CurrentControlSet\Control\ComputerName\ComputerName",
        "--value",
        "computername",
    ]);
    assert_eq!(name.len(), 1);
    assert_eq!(
        name[0]["key_path"],
        r"ControlSet001\Control\ComputerName\ComputerName"
    );
    assert_eq!(name[0]["value_name"], "ComputerName");
    assert_eq!(name[0]["value_data"], "GOLDPAPERW11");
    // a FILETIME the logs changed, kept as a number too; and the same without the logs
    let secure_time = |extra: &[&str]| {
        let mut args = vec![
            "-f",
            system,
            "--key",
            r"ControlSet001\Services\W32Time\SecureTimeLimits",
            "--value",
            "SecureTimeEstimated",
            "--qword-as-filetime",
        ];
        args.extend(extra);
        let records = registry_read(&args);
        assert_eq!(records.len(), 1);
        records[0].clone()
    };
    let replayed = secure_time(&[]);
    assert_eq!(replayed["value_type"], "REG_QWORD");
    assert_eq!(replayed["value_data"], 132_891_424_373_386_666_u64);
    assert_eq!(replayed["value_u64"], 132_891_424_373_386_666_u64);
    assert_eq!(
        replayed["value_filetime_utc"],
        "2022-02-12T12:27:17.338666600Z"
    );
    assert_eq!(replayed["transaction_log_status"], "replayed");
    assert_eq!(replayed["key_last_written"], "2022-02-08T21:03:52.520572Z");
    let as_it_is = secure_time(&["--no-logs"]);
    assert_eq!(as_it_is["value_u64"], 132_888_271_724_338_846_u64);
    assert_eq!(
        as_it_is["value_filetime_utc"],
        "2022-02-08T20:52:52.433884600Z"
    );
    assert_eq!(as_it_is["transaction_log_status"], "not_applied");
    assert_eq!(as_it_is["logs_applied"], json!([]));
    assert_eq!(
        as_it_is["key_last_written"],
        "2022-02-08T20:52:54.093671200Z"
    );
    let r = run(&["parse-registry", "read", "-f", system, "--no-logs", "-K"]);
    for line in [
        "Dirty hives whose transaction logs were not applied (keys or values may be missing): 1",
        "Keys: 32,857, values: 74,907",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    // logs renamed in a collection, given by hand
    let other = tempfile::tempdir().unwrap();
    let config = hive.path().join("HOST/C/Windows/System32/config");
    std::fs::copy(config.join("SYSTEM"), other.path().join("hive")).unwrap();
    std::fs::copy(config.join("SYSTEM.LOG1"), other.path().join("first")).unwrap();
    std::fs::copy(config.join("SYSTEM.LOG2"), other.path().join("second")).unwrap();
    let path = |name: &str| other.path().join(name).to_str().unwrap().to_string();
    let r = run(&[
        "parse-registry",
        "read",
        "-f",
        &path("hive"),
        "--log1",
        &path("first"),
        "--log2",
        &path("second"),
        "-K",
    ]);
    for line in [
        "Hive types: SYSTEM 1",
        "Dirty hives recovered with their transaction logs: 1",
        "Keys: 32,881, values: 75,061",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
}

#[test]
fn registry_deleted_keys_and_values() {
    let hive = system_hive();
    let system = hive.path().join("HOST/C/Windows/System32/config/SYSTEM");
    let system = system.to_str().unwrap();
    let r = run(&[
        "parse-registry",
        "read",
        "-f",
        system,
        "--recover-deleted",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "Recovered keys: 429 (deleted_complete 36, deleted_partial 241, transaction_log_recovered 152)",
        "Recovered values: 1,379 (deleted_complete 48, deleted_partial 577, residual_value_only 53, transaction_log_recovered 701)",
        "Value lists, values or data of recovered records lost: 65",
        "Records: 82,514 (replayed from the transaction logs: 262, recovered: 1,508)",
        // what deleted records lost is not damage
        "Damaged keys and values: 0",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    // a deleted service's values, under parents that are deleted too or lost: partial, in the
    // free cells of the hive and of the file before the replay
    let records = registry_read(&[
        "-f",
        system,
        "--recover-deleted",
        "--key-regex",
        r"\\Services\\stornvme\\Parameters$",
    ]);
    let bus = records
        .iter()
        .find(|r| r["value_name"] == "BusType" && r["allocation_status"] != "allocated")
        .unwrap();
    assert_eq!(
        bus["key_path"],
        r"[unknown]\Configurations\Stornvme_Inst\Services\stornvme\Parameters"
    );
    assert_eq!(bus["allocation_status"], "deleted_partial");
    assert_eq!(bus["recovered_from"], json!(["free_cell", "before_replay"]));
    assert_eq!(bus["key_last_written"], "2022-02-07T07:38:03.052139100Z");
    assert_eq!(bus["value_type"], "REG_DWORD");
    assert_eq!(bus["value_u64"], 17);
    assert_eq!(bus["transaction_log_status"], "unchanged");
    let allocated = records
        .iter()
        .filter(|r| r["allocation_status"] == "allocated")
        .collect::<Vec<_>>();
    assert!(allocated.iter().all(|r| r["recovered_from"] == json!([])));
    // a key only an older log entry holds: an earlier Edge's event provider
    let edge = registry_read(&[
        "-f",
        system,
        "--recover-deleted",
        "--key-regex",
        "Microsoft Edge Etw$",
    ]);
    let message = edge
        .iter()
        .find(|r| r["value_name"] == "EventMessageFile")
        .unwrap();
    assert_eq!(
        message["key_path"],
        r"ControlSet001\Services\EventLog\System\Microsoft Edge Etw"
    );
    assert_eq!(message["allocation_status"], "transaction_log_recovered");
    assert_eq!(message["recovered_from"], json!(["SYSTEM.LOG2#344"]));
    assert_eq!(
        message["value_data"],
        r"C:\Program Files (x86)\Microsoft\Edge\Application\90.0.818.66\msedge.dll"
    );
    // without --recover-deleted: only what is in use
    let records = registry_read(&[
        "-f",
        system,
        "--key-regex",
        r"\\Services\\stornvme\\Parameters$",
    ]);
    assert!(
        records
            .iter()
            .all(|r| r["allocation_status"] == "allocated")
    );
    // values without their key: no key filter takes them
    let all = registry_read(&["-f", system, "--recover-deleted", "--value-regex", "^"]);
    let residual: Vec<&Value> = all.iter().filter(|r| r["key_path"].is_null()).collect();
    let count = |status: &str| {
        residual
            .iter()
            .filter(|r| r["allocation_status"] == status)
            .count()
    };
    assert_eq!(
        (
            count("residual_value_only"),
            count("transaction_log_recovered")
        ),
        (53, 47)
    );
    assert!(residual.iter().all(|r| r["key_last_written"].is_null()));
}

/// A hive with a value of every common type, the default value, values that don't decode, a key
/// without values, and a damaged value and subkey (their signatures broken).
fn types_hive() -> Vec<u8> {
    let values = reg_key(
        "Values",
        vec![
            ("", REG_SZ, utf16z("the default")),
            ("Sz", REG_SZ, utf16z("text")),
            ("Expand", REG_EXPAND_SZ, utf16z("%SystemRoot%")),
            ("Multi", REG_MULTI_SZ, utf16z("a\0b c\0")),
            ("Dword", REG_DWORD, 7u32.to_le_bytes().to_vec()),
            ("Qword", REG_QWORD, u64::MAX.to_le_bytes().to_vec()),
            ("Binary", REG_BINARY, vec![0xde, 0xad]),
            ("EmptyExternal", REG_BINARY, Vec::new()),
            ("OddSz", REG_SZ, vec![b'A']),
            ("ShortDword", REG_DWORD, vec![1, 2, 3]),
            ("Unknown", 0x0020_0000, vec![1, 2, 3, 4, 5]),
            ("BadValue", REG_DWORD, 9u32.to_le_bytes().to_vec()),
        ],
        Vec::new(),
    );
    let root = reg_key(
        "ROOT",
        Vec::new(),
        vec![
            values,
            reg_key("Empty", Vec::new(), Vec::new()),
            reg_key("BadKey", vec![("Lost", REG_DWORD, vec![0; 4])], Vec::new()),
            reg_key("GoodKey", vec![("Kept", REG_SZ, utf16z("yes"))], Vec::new()),
        ],
    );
    let mut hive = regf(&root);
    // A zero-length value can be stored without the resident-data flag. Its data offset is
    // irrelevant and may be zero; it is still a valid empty value.
    let empty = b"EmptyExternal";
    let empty_at = hive.windows(empty.len()).position(|w| w == empty).unwrap();
    hive[empty_at - 16..empty_at - 12].copy_from_slice(&0u32.to_le_bytes());
    // a value cell starts 20 bytes before its name, a key cell 76
    for (name, before) in [(&b"BadValue"[..], 20), (b"BadKey", 76)] {
        let at = hive.windows(name.len()).position(|w| w == name).unwrap();
        hive[at - before..at - before + 2].copy_from_slice(b"xx");
    }
    hive
}

#[test]
fn registry_value_types_and_damage() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("SOFTWARE");
    std::fs::write(&hive, types_hive()).unwrap();
    let out = dir.path().join("registry.csv");
    let r = run(&[
        "parse-registry",
        "read",
        "-f",
        hive.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("Damaged keys and values: 3"),
        "{}",
        r.stdout
    );
    let (_, rows) = csv_rows(&out);
    let row = |key: &str, name: &str, kind: &str| {
        rows.iter()
            .find(|r| r["KeyPath"] == key && r["ValueName"] == name && r["RecordType"] == kind)
            .unwrap_or_else(|| panic!("{key} {name} {kind}"))
    };
    for (name, kind, data, number) in [
        ("Sz", "REG_SZ", "text", ""),
        ("Expand", "REG_EXPAND_SZ", "%SystemRoot%", ""),
        ("Multi", "REG_MULTI_SZ", r#"["a","b c"]"#, ""),
        ("Dword", "REG_DWORD", "7", "7"),
        (
            "Qword",
            "REG_QWORD",
            "18446744073709551615",
            "18446744073709551615",
        ),
        ("Binary", "REG_BINARY", "dead", ""),
        ("EmptyExternal", "REG_BINARY", "", ""),
    ] {
        let r = row("Values", name, "value");
        assert_eq!(
            (
                r["ValueType"].as_str(),
                r["ValueData"].as_str(),
                r["ValueU64"].as_str()
            ),
            (kind, data, number)
        );
        assert_eq!(r["ValueDataRaw"], "", "{name}");
        assert_eq!(r["Problems"], "", "{name}");
    }
    // the default value: an empty name, its own record type
    let default = row("Values", "", "default_value");
    assert_eq!(default["ValueData"], "the default");
    // what doesn't decode keeps its bytes
    let short = row("Values", "ShortDword", "value");
    assert_eq!(short["ValueData"], "");
    assert_eq!(short["ValueU64"], "");
    assert_eq!(short["ValueDataRaw"], "010203");
    assert_eq!(short["Problems"], "REG_DWORD of 3 bytes");
    // a string of an odd number of bytes: the characters there are, and its bytes
    let odd = row("Values", "OddSz", "value");
    assert_eq!(odd["ValueData"], "");
    assert_eq!(odd["ValueDataRaw"], "41");
    assert_eq!(odd["Problems"], "");
    let unknown = row("Values", "Unknown", "value");
    assert_eq!(unknown["ValueType"], "0x200000");
    assert_eq!(unknown["ValueDataRaw"], "0102030405");
    assert_eq!(unknown["Problems"], "");
    // a damaged value and a damaged subkey are reported, and the rest is still read
    assert!(!rows.iter().any(|r| r["ValueName"] == "BadValue"));
    assert!(
        row("Values", "", "key")["Problems"].ends_with("is not a value"),
        "{}",
        row("Values", "", "key")["Problems"]
    );
    assert!(
        row("", "", "key")["Problems"].ends_with("is not a key"),
        "{}",
        row("", "", "key")["Problems"]
    );
    assert!(!rows.iter().any(|r| r["ValueName"] == "Lost"));
    assert_eq!(row("GoodKey", "Kept", "value")["ValueData"], "yes");
    assert_eq!(row("Empty", "", "key")["Problems"], "");
    assert!(rows.iter().all(|r| r["AllocationStatus"] == "allocated"));
    // a hive that wasn't dirty
    assert!(rows.iter().all(|r| r["TransactionLogStatus"] == "clean"));

    // in JSON: typed, and every value's bytes with --raw-data
    let hive = hive.to_str().unwrap();
    let records = registry_read(&["-f", hive, "-k", "Values", "--raw-data"]);
    let find = |name: &str| {
        records
            .iter()
            .find(|r| r["value_name"] == name && r["record_type"] != "key")
            .unwrap()
    };
    assert_eq!(find("Multi")["value_data"], json!(["a", "b c"]));
    assert_eq!(find("Qword")["value_data"], json!(u64::MAX));
    assert_eq!(find("Qword")["value_u64"], json!(u64::MAX));
    // past year 9999: no FILETIME, but the number is kept
    let qword = registry_read(&[
        "-f",
        hive,
        "-k",
        "Values",
        "--value",
        "qword",
        "--qword-as-filetime",
    ]);
    assert_eq!(qword[0]["value_filetime_utc"], Value::Null);
    assert_eq!(qword[0]["value_u64"], json!(u64::MAX));
    assert_eq!(find("Dword")["value_data_raw"], "07000000");
    assert_eq!(find("ShortDword")["value_data"], Value::Null);
    assert_eq!(
        find("ShortDword")["problems"],
        json!(["REG_DWORD of 3 bytes"])
    );
    assert_eq!(find("")["value_data"], "the default");
    assert_eq!(find("")["record_type"], "default_value");

    // the default value, and a value a key hasn't (subkeys come in the order of their names)
    let default = registry_read(&[
        "-f",
        hive,
        "--key-regex",
        "^(Values|GoodKey)$",
        "--value",
        "",
    ]);
    let found: Vec<(&str, &str, &str)> = default
        .iter()
        .map(|r| {
            (
                r["key_path"].as_str().unwrap(),
                r["record_type"].as_str().unwrap(),
                r["value_name"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            ("GoodKey", "absent_value", ""),
            ("Values", "default_value", "")
        ]
    );
    let absent = registry_read(&["-f", hive, "--key", "GoodKey", "--value", "Missing"]);
    assert_eq!(absent.len(), 1);
    assert_eq!(absent[0]["record_type"], "absent_value");
    assert_eq!(absent[0]["value_name"], "Missing");
    assert_eq!(absent[0]["value_type"], Value::Null);
    assert_eq!(absent[0]["value_data"], Value::Null);
    // without a key filter, keys without the value are not listed
    let r = registry_read(&["-f", hive, "--value", "Missing"]);
    assert!(r.is_empty());
}

/// 2022-02-08T21:03:52Z and a day after, as FILETIMEs.
const LINKED: u64 = 132_888_278_320_000_000;
const DAY: u64 = 864_000_000_000;

/// A SOFTWARE hive with Group Policy's state: the machine's and an account's GPO links, the
/// default domain policy's GUID in upper case for the machine and in lower case for the account.
fn group_policy_hive() -> Vec<u8> {
    let link = |n: &str, written: u64, guid: &str| {
        reg_key(
            n,
            vec![
                (
                    "DsPath",
                    REG_SZ,
                    utf16z(&format!(
                        "LDAP://cn={guid},cn=policies,cn=system,DC=corp,DC=example"
                    )),
                ),
                ("Options", REG_DWORD, 0u32.to_le_bytes().to_vec()),
            ],
            Vec::new(),
        )
        .written(written)
    };
    let machine = reg_key(
        "Machine",
        Vec::new(),
        vec![reg_key(
            "GPLink-List",
            Vec::new(),
            vec![
                link("0", LINKED, "{31B2F340-016D-11D2-945F-00C04FB984F9}"),
                link("1", LINKED + DAY, "{6AC1786C-016F-11D2-945F-00C04FB984F9}"),
            ],
        )],
    );
    let account = reg_key(
        "S-1-5-21-1-2-3-1001",
        Vec::new(),
        vec![reg_key(
            "GPLink-List",
            Vec::new(),
            vec![link("0", LINKED, "{31b2f340-016d-11d2-945f-00c04fb984f9}")],
        )],
    );
    let state = reg_key("State", Vec::new(), vec![machine, account]);
    let mut key = state;
    for name in ["Group Policy", "CurrentVersion", "Windows", "Microsoft"] {
        key = reg_key(name, Vec::new(), vec![key]);
    }
    regf(&reg_key("ROOT", Vec::new(), vec![key]))
}

#[test]
fn registry_group_policy_state() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("SOFTWARE");
    std::fs::write(&hive, group_policy_hive()).unwrap();
    let hive = hive.to_str().unwrap();
    let state = r"Microsoft\Windows\CurrentVersion\Group Policy\State";
    // the state's links, by a regular expression on their paths
    let links = registry_read(&[
        "-f",
        hive,
        "--key-regex",
        r"\\Group Policy\\State\\.*\\GPLink-List\\\d+$",
        "--value",
        "dspath",
    ]);
    let paths: Vec<&str> = links
        .iter()
        .map(|r| r["key_path"].as_str().unwrap())
        .collect();
    assert_eq!(
        paths,
        [
            format!(r"{state}\Machine\GPLink-List\0"),
            format!(r"{state}\Machine\GPLink-List\1"),
            format!(r"{state}\S-1-5-21-1-2-3-1001\GPLink-List\0"),
        ]
    );
    assert_eq!(links[0]["hive_type"], "SOFTWARE");
    assert_eq!(links[0]["key_last_written"], "2022-02-08T21:03:52Z");
    assert_eq!(links[1]["key_last_written"], "2022-02-09T21:03:52Z");
    // one key, the path also given with the hive's name
    let one = registry_read(&[
        "-f",
        hive,
        "--key",
        &format!(r"HKLM\SOFTWARE\{state}\Machine\GPLink-List\1"),
    ]);
    assert_eq!(one.len(), 2);
    assert!(
        one.iter()
            .all(|r| r["key_path"] == format!(r"{state}\Machine\GPLink-List\1"))
    );
    // a GUID matches in any case; the evidence keeps its own
    let default_policy = registry_read(&[
        "-f",
        hive,
        "--data-regex",
        "31b2f340-016d-11d2-945f-00c04fb984f9",
    ]);
    let found: Vec<&str> = default_policy
        .iter()
        .map(|r| r["value_data"].as_str().unwrap())
        .collect();
    assert_eq!(
        found,
        [
            "LDAP://cn={31B2F340-016D-11D2-945F-00C04FB984F9},cn=policies,cn=system,DC=corp,DC=example",
            "LDAP://cn={31b2f340-016d-11d2-945f-00c04fb984f9},cn=policies,cn=system,DC=corp,DC=example",
        ]
    );
    // value names by a regular expression
    let options = registry_read(&["-f", hive, "--value-regex", "^opt"]);
    assert_eq!(options.len(), 3);
    // the numbered links last written on the second day
    let later = registry_read(&[
        "-f",
        hive,
        "--key-regex",
        r"GPLink-List\\\d+$",
        "--timeline-start",
        "2022-02-09",
        "--timeline-end",
        "2022-02-09T23:59:59Z",
    ]);
    assert_eq!(later.len(), 2);
    assert!(
        later
            .iter()
            .all(|r| r["key_path"] == format!(r"{state}\Machine\GPLink-List\1"))
    );
    // a key the hive hasn't
    let r = run(&["parse-registry", "read", "-f", hive, "-k", "Policies", "-K"]);
    assert!(
        r.stdout.contains("Hives without the key of --key: 1"),
        "{}",
        r.stdout
    );
}

#[test]
fn registry_triage_directory() {
    let dir = tempfile::tempdir().unwrap();
    for host in ["HOST1", "HOST2"] {
        let programs = dir.path().join(host).join("C/Windows/AppCompat/Programs");
        std::fs::create_dir_all(&programs).unwrap();
        std::fs::copy(repo(AMCACHE_HIVE), programs.join("Amcache.hve")).unwrap();
        std::fs::write(programs.join("notes.txt"), b"not a hive").unwrap();
    }
    let out = dir.path().join("registry.csv");
    let r = run(&[
        "parse-registry",
        "read",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "Registry hives: 2",
        "Hive types: AMCACHE.HVE 2",
        "Records: 21,888",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let (_, rows) = csv_rows(&out);
    for host in ["HOST1", "HOST2"] {
        let of_host = rows
            .iter()
            .filter(|r| Path::new(&r["SourcePath"]).starts_with(dir.path().join(host)))
            .count();
        assert_eq!(of_host, 10_944, "{host}");
    }
}

#[test]
fn registry_hosts_from_system_hives() {
    // a collection: the SYSTEM hive names the host of the system's other hives
    let hive = system_hive();
    let users = hive.path().join("HOST/C/Users/alice");
    std::fs::create_dir_all(&users).unwrap();
    std::fs::copy(repo(AMCACHE_HIVE), users.join("NTUSER.DAT")).unwrap();
    let records = registry_read(&["-d", hive.path().to_str().unwrap(), "--key-regex", "^Root$"]);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["host"], "GOLDPAPERW11");
    assert_eq!(records[0]["hive_type"], "AMCACHE.HVE");
    // hives exported to one directory: the SYSTEM hive next to it
    let flat = tempfile::tempdir().unwrap();
    let config = hive.path().join("HOST/C/Windows/System32/config");
    for name in ["SYSTEM", "SYSTEM.LOG1", "SYSTEM.LOG2"] {
        std::fs::copy(config.join(name), flat.path().join(name)).unwrap();
    }
    std::fs::copy(repo(AMCACHE_HIVE), flat.path().join("Amcache.hve")).unwrap();
    let amcache = flat.path().join("Amcache.hve");
    let records = registry_read(&["-f", amcache.to_str().unwrap(), "--key-regex", "^Root$"]);
    assert_eq!(records[0]["host"], "GOLDPAPERW11");
}

#[test]
fn registry_input_and_output_errors() {
    let r = run(&["parse-registry", "read", "-f", "test_files/missing", "-K"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("The file does not exist"), "{}", r.stdout);
    let r = run(&[
        "parse-registry",
        "read",
        "-f",
        "test_files/lnk/README.md",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("Could not parse"), "{}", r.stdout);
    let r = run(&["parse-registry", "read", "-d", "test_files/lnk", "-K"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("No registry hives"), "{}", r.stdout);
    for option in ["--key-regex", "--value-regex", "--data-regex"] {
        let r = run(&[
            "parse-registry",
            "read",
            "-f",
            AMCACHE_HIVE,
            option,
            "(",
            "-K",
        ]);
        assert_eq!(r.code, 1);
        assert!(r.stdout.contains(&format!("{option}: ")), "{}", r.stdout);
    }
    for option in ["--log1", "--log2"] {
        let r = run(&[
            "parse-registry",
            "read",
            "-d",
            "test_files",
            option,
            AMCACHE_HIVE,
        ]);
        assert_ne!(r.code, 0, "{option} needs -f");
        let r = run(&[
            "parse-registry",
            "read",
            "-f",
            AMCACHE_HIVE,
            option,
            AMCACHE_HIVE,
            "--no-logs",
        ]);
        assert_ne!(r.code, 0, "{option} and --no-logs");
    }
    let r = run(&[
        "parse-registry",
        "read",
        "-f",
        AMCACHE_HIVE,
        "--log1",
        "test_files/missing",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("The file does not exist"), "{}", r.stdout);
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("keep.csv");
    std::fs::write(&file, b"keep").unwrap();
    let r = run(&[
        "parse-registry",
        "read",
        "-f",
        AMCACHE_HIVE,
        "-o",
        file.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("already exists"), "{}", r.stdout);
    assert_eq!(std::fs::read(&file).unwrap(), b"keep");
}

// --- parse-registry services --------------------------------------------------------------------

/// `parse-registry services`'s CSV columns.
const SERVICES_COLUMNS: [&str; 27] = [
    "Host",
    "SourcePath",
    "ControlSet",
    "IsCurrentControlSet",
    "ServiceName",
    "DisplayName",
    "Description",
    "ImagePath",
    "ServiceDll",
    "StartType",
    "StartValue",
    "DelayedAutostart",
    "ServiceType",
    "TypeValue",
    "ErrorControl",
    "ObjectName",
    "Group",
    "FailureResetPeriod",
    "FailureCommand",
    "FailureActions",
    "FailureActionType",
    "FailureActionDelayMs",
    "FailureActionsRaw",
    "KeyLastWrittenUTC",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// regipy's 32-bit Windows 7 SYSTEM hive (two control sets), where a collection has it.
fn regipy_system() -> tempfile::TempDir {
    use std::io::Read as _;
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("W7/C/Windows/System32/config");
    std::fs::create_dir_all(&config).unwrap();
    let gz = std::fs::File::open(repo("test_files/shimcache/regipy/SYSTEM.gz")).unwrap();
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(gz)
        .read_to_end(&mut bytes)
        .unwrap();
    std::fs::write(config.join("SYSTEM"), bytes).unwrap();
    dir
}

/// `parse-registry services` with `args`, as JSON Lines.
fn services_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "services"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_services_of_a_system_hive() {
    let hive = regipy_system();
    let system = hive.path().join("W7/C/Windows/System32/config/SYSTEM");
    let system = system.to_str().unwrap();
    let out = hive.path().join("services.csv");
    let r = run(&[
        "parse-registry",
        "services",
        "-f",
        system,
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "SYSTEM hives parsed: 1",
        "Hosts: WKS-WIN732BITA",
        "Control sets read: 1 (the one in use; --all-control-sets for every one)",
        "Services: 467 (boot 36, system 28, automatic 61, manual 283, disabled 9, no Start value 50)",
        "With failure actions: 133",
        "Failure actions that run a command: 0",
        "Values that could not be read: 0",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let (header, rows) = csv_rows(&out);
    assert_eq!(header, SERVICES_COLUMNS);
    assert_eq!(rows.len(), 467);
    let event_log = rows
        .iter()
        .find(|r| r["ServiceName"] == "eventlog")
        .unwrap();
    for (column, expected) in [
        ("Host", "WKS-WIN732BITA"),
        ("ControlSet", "ControlSet001"),
        ("IsCurrentControlSet", "True"),
        ("DisplayName", r"@%SystemRoot%\system32\wevtsvc.dll,-200"),
        (
            "ImagePath",
            r"%SystemRoot%\System32\svchost.exe -k LocalServiceNetworkRestricted",
        ),
        ("ServiceDll", r"%SystemRoot%\System32\wevtsvc.dll"),
        ("StartType", "automatic"),
        ("StartValue", "2"),
        ("DelayedAutostart", ""),
        ("ServiceType", "win32_share_process"),
        ("TypeValue", "32"),
        ("ErrorControl", "normal"),
        ("ObjectName", r"NT AUTHORITY\LocalService"),
        ("Group", "Event Log"),
        ("FailureResetPeriod", "86400"),
        ("FailureCommand", ""),
        (
            "FailureActions",
            r#"[{"type":"restart","delay_ms":60000},{"type":"restart","delay_ms":120000},{"type":"none","delay_ms":0}]"#,
        ),
        ("FailureActionType", "restart"),
        ("FailureActionDelayMs", "60000"),
        ("FailureActionsRaw", ""),
        ("TransactionLogStatus", "clean"),
        ("Problems", ""),
    ] {
        assert_eq!(event_log[column], expected, "{column}");
    }

    // one service, in any case; started after the other automatic ones
    let wu = services_read(&["-f", system, "--service", "WUAUSERV"]);
    assert_eq!(wu.len(), 1);
    assert_eq!(wu[0]["service_name"], "wuauserv");
    assert_eq!(wu[0]["delayed_autostart"], true);
    assert_eq!(
        wu[0]["failure_actions"][0],
        json!({"type": "restart", "delay_ms": 60000})
    );
    // a driver: no failure actions
    let tcpip = services_read(&["-f", system, "--service", "tcpip"]);
    assert_eq!(tcpip[0]["start_type"], "boot");
    assert_eq!(tcpip[0]["service_type"], "kernel_driver");
    assert_eq!(tcpip[0]["failure_actions"], Value::Null);
    assert_eq!(tcpip[0]["failure_action_type"], Value::Null);

    // every control set: the one not in use said so
    let all = services_read(&["-f", system, "--all-control-sets"]);
    assert_eq!(all.len(), 933);
    let second: Vec<&Value> = all
        .iter()
        .filter(|r| r["control_set"] == "ControlSet002")
        .collect();
    assert_eq!(second.len(), 466);
    assert!(second.iter().all(|r| r["is_current_control_set"] == false));
    // a service the hive hasn't
    let r = run(&[
        "parse-registry",
        "services",
        "-f",
        system,
        "--service",
        "nosuch",
        "-K",
    ]);
    assert!(
        r.stdout
            .contains("Hives without the service of --service: 1"),
        "{}",
        r.stdout
    );
}

#[test]
fn registry_services_with_logs() {
    let hive = system_hive();
    let system = hive.path().join("HOST/C/Windows/System32/config/SYSTEM");
    let system = system.to_str().unwrap();
    let r = run(&["parse-registry", "services", "-f", system, "-K"]);
    for line in [
        "Hosts: GOLDPAPERW11",
        "Dirty hives recovered with their transaction logs: 1",
        "Services: 718 (boot 97, system 31, automatic 78, manual 448, disabled 16, no Start value 48)",
        "With failure actions: 234",
        "Failure actions that run a command: 1",
        r#"ControlSet001\spectrum: "C:\Windows\System32\Spectrum.exe" -safemode"#,
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let spectrum = services_read(&["-f", system, "--service", "Spectrum"]);
    let s = &spectrum[0];
    assert_eq!(s["failure_reset_period"], 60);
    assert_eq!(
        s["failure_command"],
        r#""C:\Windows\System32\Spectrum.exe" -safemode"#
    );
    let actions: Vec<&str> = s["failure_actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["type"].as_str().unwrap())
        .collect();
    assert_eq!(
        actions,
        ["restart", "restart", "restart", "restart", "run_command"]
    );
    assert_eq!(s["transaction_log_status"], "unchanged");
    assert_eq!(s["logs_applied"], json!(["SYSTEM.LOG1", "SYSTEM.LOG2"]));
    // user services: their type's flags
    let all = services_read(&["-f", system]);
    assert!(
        all.iter()
            .any(|r| r["service_type"] == "win32_share_process|user_service|user_service_instance")
    );
    // the logs wrote some services
    assert!(
        all.iter()
            .any(|r| r["transaction_log_status"] == "replayed")
    );
    let without = services_read(&["-f", system, "--no-logs", "--service", "spectrum"]);
    assert_eq!(without[0]["transaction_log_status"], "not_applied");
}

/// A SYSTEM hive whose services have a FailureActions value that is cut short, a Start value
/// that is a string, and a DLL under Parameters.
fn services_hive(current: u32) -> Vec<u8> {
    let actions = |count: u32, list: &[(u32, u32)]| {
        let mut b = Vec::new();
        for v in [3600u32, 0, 0, count, 0x700f_0000] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        for (kind, delay) in list {
            b.extend_from_slice(&kind.to_le_bytes());
            b.extend_from_slice(&delay.to_le_bytes());
        }
        b
    };
    let dword = |n: u32| n.to_le_bytes().to_vec();
    let good = reg_key(
        "ExampleSvc",
        vec![
            (
                "ImagePath",
                REG_EXPAND_SZ,
                utf16z(r"C:\ProgramData\example\svc.exe"),
            ),
            ("Start", REG_DWORD, dword(2)),
            ("Type", REG_DWORD, dword(0x10)),
            ("ObjectName", REG_SZ, utf16z("LocalSystem")),
            (
                "FailureCommand",
                REG_SZ,
                utf16z(r"C:\ProgramData\example\again.exe"),
            ),
            (
                "FailureActions",
                REG_BINARY,
                actions(2, &[(1, 60_000), (3, 0)]),
            ),
        ],
        vec![reg_key(
            "Parameters",
            vec![(
                "ServiceDll",
                REG_EXPAND_SZ,
                utf16z(r"C:\ProgramData\example\svc.dll"),
            )],
            Vec::new(),
        )],
    )
    .written(LINKED);
    let short = reg_key(
        "Short",
        vec![
            ("Start", REG_SZ, utf16z("2")),
            ("FailureActions", REG_BINARY, actions(3, &[(1, 1)])),
        ],
        Vec::new(),
    );
    let wrong_types = reg_key(
        "WrongTypes",
        vec![
            ("Start", REG_QWORD, 2u64.to_le_bytes().to_vec()),
            ("FailureActions", REG_SZ, actions(1, &[(1, 1)])),
        ],
        Vec::new(),
    );
    let services = reg_key("Services", Vec::new(), vec![good, short, wrong_types]);
    let name = reg_key(
        "ComputerName",
        Vec::new(),
        vec![reg_key(
            "ComputerName",
            vec![("ComputerName", REG_SZ, utf16z("EXAMPLE-PC"))],
            Vec::new(),
        )],
    );
    let control = reg_key("Control", Vec::new(), vec![name]);
    let set = reg_key("ControlSet001", Vec::new(), vec![control, services]);
    let select = reg_key(
        "Select",
        vec![("Current", REG_DWORD, dword(current))],
        Vec::new(),
    );
    regf(&reg_key("ROOT", Vec::new(), vec![set, select]))
}

#[test]
fn registry_services_damage_and_discovery() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("H/C/Windows/System32/config");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("SYSTEM"), services_hive(1)).unwrap();
    let programs = dir.path().join("H/C/Windows/AppCompat/Programs");
    std::fs::create_dir_all(&programs).unwrap();
    std::fs::copy(repo(AMCACHE_HIVE), programs.join("Amcache.hve")).unwrap();
    // under -d: the SYSTEM hive, not the Amcache hive next to it
    let r = run(&[
        "parse-registry",
        "services",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "SYSTEM hives: 1",
        "Hosts: EXAMPLE-PC",
        "Services: 3 (automatic 1, no Start value 2)",
        "Failure actions that run a command: 1",
        r"ControlSet001\ExampleSvc: C:\ProgramData\example\again.exe",
        "Values that could not be read: 4",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let records = services_read(&["-d", dir.path().to_str().unwrap()]);
    let good = records
        .iter()
        .find(|record| record["service_name"] == "ExampleSvc")
        .unwrap();
    assert_eq!(good["service_name"], "ExampleSvc");
    assert_eq!(good["service_dll"], r"C:\ProgramData\example\svc.dll");
    assert_eq!(good["failure_action_type"], "restart");
    assert_eq!(good["failure_action_delay_ms"], 60000);
    assert_eq!(good["failure_actions"][1]["type"], "run_command");
    assert_eq!(good["key_last_written"], "2022-02-08T21:03:52Z");
    // what can't be read is a problem, its bytes kept; the rest is still read
    let short = records
        .iter()
        .find(|record| record["service_name"] == "Short")
        .unwrap();
    assert_eq!(short["start_type"], Value::Null);
    assert_eq!(short["failure_actions"], Value::Null);
    assert_eq!(
        short["problems"],
        json!([
            "Start: is REG_SZ, not REG_DWORD",
            "FailureActions counts 3 actions, its 28 bytes hold 1"
        ])
    );
    assert!(
        short["failure_actions_raw"]
            .as_str()
            .unwrap()
            .starts_with("100e0000")
    );
    let wrong = records
        .iter()
        .find(|record| record["service_name"] == "WrongTypes")
        .unwrap();
    assert_eq!(wrong["start_type"], Value::Null);
    assert_eq!(wrong["failure_actions"], Value::Null);
    assert_eq!(
        wrong["problems"],
        json!([
            "Start: is REG_QWORD, not REG_DWORD",
            "FailureActions: is REG_SZ, not REG_BINARY"
        ])
    );
    assert!(
        wrong["failure_actions_raw"]
            .as_str()
            .unwrap()
            .starts_with("100e0000")
    );

    // a hive that is not a SYSTEM hive
    let r = run(&["parse-registry", "services", "-f", AMCACHE_HIVE, "-K"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("not a SYSTEM hive"), "{}", r.stdout);
    let r = run(&["parse-registry", "services", "-d", "test_files/lnk", "-K"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("No SYSTEM hives were found"),
        "{}",
        r.stdout
    );
    let r = run(&[
        "parse-registry",
        "services",
        "-d",
        "test_files",
        "--log1",
        AMCACHE_HIVE,
    ]);
    assert_ne!(r.code, 0, "--log1 needs -f");
}

#[test]
fn registry_services_falls_back_when_select_current_is_unusable() {
    let dir = tempfile::tempdir().unwrap();
    let system = dir.path().join("SYSTEM");
    std::fs::write(&system, services_hive(99)).unwrap();
    let system = system.to_str().unwrap();

    let records = services_read(&["-f", system]);
    assert_eq!(records.len(), 3);
    assert!(
        records
            .iter()
            .all(|record| record["control_set"] == "ControlSet001")
    );
    assert!(
        records
            .iter()
            .all(|record| record["is_current_control_set"] == false)
    );

    let r = run(&["parse-registry", "services", "-f", system, "-K"]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout
            .contains("Hives without usable Select\\Current (the first control set was read): 1"),
        "{}",
        r.stdout
    );
}

// --- parse-registry recentdocs ------------------------------------------------------------------

/// `parse-registry recentdocs`'s CSV columns.
const RECENTDOCS_COLUMNS: [&str; 14] = [
    "Host",
    "Username",
    "SourcePath",
    "Extension",
    "MruPosition",
    "ValueIndex",
    "Filename",
    "LinkName",
    "KeyLastWrittenUTC",
    "LastOpenedUTC",
    "ValueDataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// The gzipped files `names` of `test_files/registry/<dir>`, unpacked into `to`.
fn unpack_registry(dir: &str, names: &[&str], to: &Path) {
    use std::io::Read as _;
    std::fs::create_dir_all(to).unwrap();
    for name in names {
        let gz =
            std::fs::File::open(repo(&format!("test_files/registry/{dir}/{name}.gz"))).unwrap();
        let mut bytes = Vec::new();
        flate2::read::GzDecoder::new(gz)
            .read_to_end(&mut bytes)
            .unwrap();
        std::fs::write(to.join(name), bytes).unwrap();
    }
}

/// `parse-registry recentdocs` with `args`, as JSON Lines.
fn recentdocs_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "recentdocs"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_recentdocs_in_a_collection() {
    // a Windows 11 system's SYSTEM hive and a user's dirty NTUSER.DAT, where they were
    let collection = system_hive();
    let profile = collection.path().join("HOST/C/Users/AndrewRathbun");
    unpack_registry(
        "rathbun_w11",
        &["NTUSER.DAT", "ntuser.dat.LOG1", "ntuser.dat.LOG2"],
        &profile,
    );
    let dir = collection.path().to_str().unwrap();
    let r = run(&["parse-registry", "recentdocs", "-d", dir, "-K"]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "NTUSER.DAT hives: 1",
        "Hosts: GOLDPAPERW11",
        "Users: AndrewRathbun",
        "Dirty hives recovered with their transaction logs: 1",
        "Recent items: 22 (in 5 lists: RecentDocs and its extensions)",
        "NTUSER.DAT: The Internet (2022-02-13T03:54:53.963087500Z)",
        "Items or lists that could not be read: 0",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let records = recentdocs_read(&["-d", dir]);
    assert_eq!(records.len(), 22);
    let first = &records[0];
    for (field, expected) in [
        ("host", json!("GOLDPAPERW11")),
        ("username", json!("AndrewRathbun")),
        ("extension", Value::Null),
        ("mru_position", json!(0)),
        ("value_index", json!(10)),
        ("filename", json!("The Internet")),
        ("link_name", json!("The Internet.lnk")),
        ("key_last_written", json!("2022-02-13T03:54:53.963087500Z")),
        ("last_opened", json!("2022-02-13T03:54:53.963087500Z")),
        ("value_data_raw", Value::Null),
        ("transaction_log_status", json!("unchanged")),
        ("problems", json!([])),
    ] {
        assert_eq!(first[field], expected, "{field}");
    }
    // the most recent item of all, as the issue's jq query finds it
    let latest: Vec<&Value> = records
        .iter()
        .filter(|r| r["extension"].is_null() && r["mru_position"] == 0)
        .collect();
    assert_eq!(latest.len(), 1);
    // one extension's list: only its most recent item has the time it was opened
    let txt: Vec<(&str, &str, &Value)> = records
        .iter()
        .filter(|r| r["extension"] == ".txt")
        .map(|r| {
            (
                r["filename"].as_str().unwrap(),
                r["link_name"].as_str().unwrap(),
                &r["last_opened"],
            )
        })
        .collect();
    assert_eq!(
        txt,
        [
            (
                "New Text Document.txt",
                "New Text Document.lnk",
                &json!("2022-02-06T20:15:40.621827400Z")
            ),
            ("deletedfile.txt", "deletedfile.lnk", &Value::Null),
        ]
    );
    let folders: Vec<&str> = records
        .iter()
        .filter(|r| r["extension"] == "Folder")
        .map(|r| r["filename"].as_str().unwrap())
        .collect();
    assert_eq!(folders, ["The Internet", "UsageLogs", "OneDrive", "logs"]);
    // without the logs: the same items, said to be possibly out of date
    let without = recentdocs_read(&["-d", dir, "--no-logs"]);
    assert_eq!(without.len(), 22);
    assert!(
        without
            .iter()
            .all(|r| r["transaction_log_status"] == "not_applied")
    );
}

#[test]
fn registry_recentdocs_of_an_exported_hive() {
    // a hive on its own: its user from the paths of its shell folders
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["NTUSER.DAT"], dir.path());
    let hive = dir.path().join("NTUSER.DAT");
    let out = dir.path().join("recentdocs.csv");
    let r = run(&[
        "parse-registry",
        "recentdocs",
        "-f",
        hive.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "Users: vibranium",
        "Recent items: 22 (in 5 lists: RecentDocs and its extensions)",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let (header, rows) = csv_rows(&out);
    assert_eq!(header, RECENTDOCS_COLUMNS);
    assert_eq!(rows.len(), 22);
    let row = |ext: &str, position: &str| {
        rows.iter()
            .find(|r| r["Extension"] == ext && r["MruPosition"] == position)
            .unwrap()
    };
    for (column, expected) in [
        ("Host", ""),
        ("Username", "vibranium"),
        ("ValueIndex", "10"),
        ("Filename", "Undercover-Agents-List-For-United-Kingdom.xls"),
        // the shortcut was named without the document's extension
        ("LinkName", "Undercover-Agents-List-For-United-Kingdom.lnk"),
        ("KeyLastWrittenUTC", "2012-04-04T15:43:17.160714700Z"),
        ("LastOpenedUTC", ""),
        ("TransactionLogStatus", "clean"),
        ("Problems", ""),
    ] {
        assert_eq!(row("", "1")[column], expected, "{column}");
    }
    assert_eq!(
        row(".xls", "0")["LastOpenedUTC"],
        "2012-04-04T15:43:17.160714700Z"
    );
    let jpg: Vec<&str> = ["0", "1", "2"]
        .iter()
        .map(|p| row(".JPG", p)["Filename"].as_str())
        .collect();
    assert_eq!(jpg, ["hq-1.JPG", "clp-2 - Fuel Hookup.JPG", "clp-1.JPG"]);
}

/// An NTUSER.DAT whose RecentDocs list names a value that isn't there and leaves one out, with
/// an item of the wrong type.
fn recentdocs_hive() -> Vec<u8> {
    let mru = |list: &[u32]| -> Vec<u8> {
        list.iter()
            .chain(&[u32::MAX])
            .flat_map(|n| n.to_le_bytes())
            .collect()
    };
    let recent = reg_key(
        "RecentDocs",
        vec![
            ("MRUListEx", REG_BINARY, mru(&[1, 4, 0])),
            ("0", REG_BINARY, utf16z("notes.txt")),
            ("1", REG_BINARY, utf16z("plan.docx")),
            ("2", REG_SZ, utf16z("left out.pdf")),
        ],
        Vec::new(),
    )
    .written(LINKED);
    let mut key = recent;
    for name in [
        "Explorer",
        "CurrentVersion",
        "Windows",
        "Microsoft",
        "Software",
    ] {
        key = reg_key(name, Vec::new(), vec![key]);
    }
    regf(&reg_key("ROOT", Vec::new(), vec![key]))
}

#[test]
fn registry_recentdocs_damage_and_errors() {
    let dir = tempfile::tempdir().unwrap();
    let profile = dir.path().join("C/Users/bob");
    std::fs::create_dir_all(&profile).unwrap();
    std::fs::write(profile.join("NTUSER.DAT"), recentdocs_hive()).unwrap();
    let records = recentdocs_read(&["-d", dir.path().to_str().unwrap()]);
    let items: Vec<(&Value, &Value, &Value, &Value)> = records
        .iter()
        .map(|r| {
            (
                &r["mru_position"],
                &r["value_index"],
                &r["filename"],
                &r["problems"],
            )
        })
        .collect();
    assert_eq!(
        items,
        [
            (&json!(0), &json!(1), &json!("plan.docx"), &json!([])),
            (
                &json!(1),
                &json!(4),
                &Value::Null,
                &json!(["MRUListEx lists value 4, which is not there"])
            ),
            (&json!(2), &json!(0), &json!("notes.txt"), &json!([])),
            (
                &Value::Null,
                &json!(2),
                &json!("left out.pdf"),
                &json!(["MRUListEx doesn't list it", "is REG_SZ, not REG_BINARY"])
            ),
        ]
    );
    assert_eq!(records[0]["username"], "bob");
    assert_eq!(records[0]["last_opened"], "2022-02-08T21:03:52Z");
    assert!(records[3]["value_data_raw"].is_string());
    let r = run(&[
        "parse-registry",
        "recentdocs",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert!(
        r.stdout
            .contains("Items or lists that could not be read: 3"),
        "{}",
        r.stdout
    );

    // a SYSTEM hive is not an NTUSER.DAT; a hive without RecentDocs is said so
    let hive = system_hive();
    let system = hive.path().join("HOST/C/Windows/System32/config/SYSTEM");
    let r = run(&[
        "parse-registry",
        "recentdocs",
        "-f",
        system.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("not an NTUSER.DAT hive"), "{}", r.stdout);
    let empty = tempfile::tempdir().unwrap();
    let users = empty.path().join("C/Users/carol");
    std::fs::create_dir_all(&users).unwrap();
    std::fs::write(
        users.join("NTUSER.DAT"),
        regf(&reg_key("ROOT", Vec::new(), Vec::new())),
    )
    .unwrap();
    let r = run(&[
        "parse-registry",
        "recentdocs",
        "-d",
        empty.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("Hives without RecentDocs: 1"),
        "{}",
        r.stdout
    );
    let r = run(&["parse-registry", "recentdocs", "-d", "test_files/lnk", "-K"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("No NTUSER.DAT hives were found"),
        "{}",
        r.stdout
    );
}

// --- parse-registry taskcache -------------------------------------------------------------------

/// `parse-registry taskcache`'s CSV columns.
const TASKCACHE_COLUMNS: [&str; 31] = [
    "Host",
    "SourcePath",
    "TaskName",
    "TaskGuid",
    "TaskIndex",
    "TaskLists",
    "Hidden",
    "InTree",
    "InTasks",
    "HasSecurityDescriptor",
    "TreeKeyLastWrittenUTC",
    "TaskKeyLastWrittenUTC",
    "ActionIndex",
    "ActionType",
    "Executable",
    "Arguments",
    "WorkingDirectory",
    "ComClassId",
    "ActionData",
    "Context",
    "Author",
    "Uri",
    "Description",
    "LastRegisteredUTC",
    "LastRunUTC",
    "LastSuccessfulRunUTC",
    "LastResult",
    "ActionsRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

fn unhex(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

/// `Actions` and `DynamicInfo` of tasks of the DFIR Artifact Museum's SOFTWARE hives (MIT): a
/// Windows 11 system task, and two tasks an attack simulator left on Windows 10 and Server 2022.
const SYNC_ACTIONS: &str = "0300180000004c006f00630061006c005300650072007600690063006500666600000000300000002500770069006e0064006900720025005c00730079007300740065006d00330032005c00730063002e0065007800650034000000730074006100720074002000770033003200740069006d00650020007400610073006b005f007300740061007200740065006400000000000000";
const SYNC_DYNAMIC: &str =
    "0300000042bed253da1ad801d6e9726c8d20d8010000000020040000573f4d6f8d20d801";
const GAMEOVER_ACTIONS: &str = "03000c00000041007500740068006f0072006666000000001c00000043003a005c0054004d0050005c006d0069006d002e006500780065004e000000730065006b00750072006c00730061003a003a004c006f0067006f006e00500061007300730077006f0072006400730020003e00200043003a005c0054004d0050005c006f002e00740078007400000000000000";
const GAMEOVER_DYNAMIC: &str =
    "03000000493c8facc134d80197874d41c734d80100000000020007800000000000000000";
const FALSH_ACTIONS: &str = "03000c00000041007500740068006f0072006666000000001c00000070006f007700650072007300680065006c006c002e006500780065003c0000002d00570069006e0064006f0077005300740079006c0065002000480069006400640065006e0020002d006e006f00700020002d006300200022002200000000000000";
const FALSH_DYNAMIC: &str =
    "030000004d0bf2cb6f01da018b9d78137001da010000000000000000d7f3f6137001da01";

/// A SOFTWARE hive's Task Scheduler cache: a system task, a task outside `\Microsoft\`, one
/// whose Tree entry has lost its SD (hidden), an Index 0 entry without its Tasks key, and a
/// Tasks key no Tree entry names, its Actions cut short.
fn taskcache_hive() -> Vec<u8> {
    const SYNC: &str = "{CC75EC31-F3E4-4F10-8279-0ECC1C1C37CD}";
    const GAMEOVER: &str = "{13ABE1C7-3F21-4017-9BF6-899BE8C1BA89}";
    const FALSH: &str = "{A50EF102-2914-473C-95B9-315F908F707E}";
    const DEFRAG: &str = "{8C1BA5F6-61DE-43AE-A3F2-37AA03E817CD}";
    const ORPHAN: &str = "{11111111-2222-3333-4444-555555555555}";
    let entry = |name: &str, guid: &str, index: u32, sd: bool| {
        let mut values = vec![
            ("Id", REG_SZ, utf16z(guid)),
            ("Index", REG_DWORD, index.to_le_bytes().to_vec()),
        ];
        if sd {
            values.push(("SD", REG_BINARY, vec![1, 0, 4, 0x80, 0, 0, 0, 0]));
        }
        reg_key(name, values, Vec::new()).written(LINKED)
    };
    let task = |guid: &str, path: &str, actions: &[u8], dynamic: Option<&str>| {
        let mut values = vec![
            ("Path", REG_SZ, utf16z(path)),
            ("URI", REG_SZ, utf16z(path)),
            ("Author", REG_SZ, utf16z("Example")),
            ("Actions", REG_BINARY, actions.to_vec()),
        ];
        if let Some(d) = dynamic {
            values.push(("DynamicInfo", REG_BINARY, unhex(d)));
        }
        reg_key(guid, values, Vec::new()).written(LINKED + DAY)
    };
    let list = |name: &str, guids: &[&str]| {
        reg_key(
            name,
            Vec::new(),
            guids
                .iter()
                .map(|g| reg_key(g, Vec::new(), Vec::new()))
                .collect(),
        )
    };
    let windows = reg_key(
        "Windows",
        Vec::new(),
        vec![
            reg_key(
                "Defrag",
                Vec::new(),
                vec![entry("ScheduledDefrag", DEFRAG, 0, true)],
            ),
            reg_key(
                "Time Synchronization",
                Vec::new(),
                vec![entry("SynchronizeTime", SYNC, 3, true)],
            ),
        ],
    );
    let tree = reg_key(
        "Tree",
        Vec::new(),
        vec![
            entry("falshupdate22", FALSH, 2, false),
            entry("GameOver", GAMEOVER, 3, true),
            reg_key("Microsoft", Vec::new(), vec![windows]),
        ],
    );
    let tasks = reg_key(
        "Tasks",
        Vec::new(),
        vec![
            task(ORPHAN, r"\Orphan", &unhex(SYNC_ACTIONS)[..60], None),
            task(
                GAMEOVER,
                r"\GameOver",
                &unhex(GAMEOVER_ACTIONS),
                Some(GAMEOVER_DYNAMIC),
            ),
            task(
                FALSH,
                r"\falshupdate22",
                &unhex(FALSH_ACTIONS),
                Some(FALSH_DYNAMIC),
            ),
            task(
                SYNC,
                r"\Microsoft\Windows\Time Synchronization\SynchronizeTime",
                &unhex(SYNC_ACTIONS),
                Some(SYNC_DYNAMIC),
            ),
        ],
    );
    let cache = reg_key(
        "TaskCache",
        Vec::new(),
        vec![
            list("Logon", &[FALSH]),
            list("Maintenance", &[SYNC]),
            list("Plain", &[GAMEOVER, SYNC]),
            tasks,
            tree,
        ],
    );
    let mut key = cache;
    for name in ["Schedule", "CurrentVersion", "Windows NT", "Microsoft"] {
        key = reg_key(name, Vec::new(), vec![key]);
    }
    regf(&reg_key("ROOT", Vec::new(), vec![key]))
}

/// `parse-registry taskcache` with `args`, as JSON Lines.
fn taskcache_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "taskcache"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_taskcache_of_a_software_hive() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("SOFTWARE");
    std::fs::write(&hive, taskcache_hive()).unwrap();
    let hive = hive.to_str().unwrap();
    let out = dir.path().join("tasks.csv");
    let r = run(&[
        "parse-registry",
        "taskcache",
        "-f",
        hive,
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "SOFTWARE hives parsed: 1",
        "Tasks: 5",
        "Actions: 3 (exec 3, COM handler 0)",
        "Tasks hidden from the Task Scheduler: 2",
        r"\falshupdate22 (its Tree entry has no SD)",
        r"\Orphan (no Tree entry names it)",
        r"Programs of tasks outside \Microsoft\: 2",
        r"\GameOver: C:\TMP\mim.exe sekurlsa::LogonPasswords > C:\TMP\o.txt",
        r#"\falshupdate22: powershell.exe -WindowStyle Hidden -nop -c """#,
        "Values that could not be read or don't match: 1",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let (header, rows) = csv_rows(&out);
    assert_eq!(header, TASKCACHE_COLUMNS);
    let names: Vec<&str> = rows.iter().map(|r| r["TaskName"].as_str()).collect();
    assert_eq!(
        names,
        [
            r"\falshupdate22",
            r"\GameOver",
            r"\Microsoft\Windows\Defrag\ScheduledDefrag",
            r"\Microsoft\Windows\Time Synchronization\SynchronizeTime",
            r"\Orphan",
        ]
    );
    let game_over = &rows[1];
    for (column, expected) in [
        ("TaskGuid", "{13ABE1C7-3F21-4017-9BF6-899BE8C1BA89}"),
        ("TaskIndex", "3"),
        ("TaskLists", "Plain"),
        ("Hidden", "False"),
        ("InTree", "True"),
        ("InTasks", "True"),
        ("HasSecurityDescriptor", "True"),
        ("TreeKeyLastWrittenUTC", "2022-02-08T21:03:52Z"),
        ("TaskKeyLastWrittenUTC", "2022-02-09T21:03:52Z"),
        ("ActionIndex", "0"),
        ("ActionType", "exec"),
        ("Executable", r"C:\TMP\mim.exe"),
        ("Arguments", r"sekurlsa::LogonPasswords > C:\TMP\o.txt"),
        ("WorkingDirectory", ""),
        ("Context", "Author"),
        ("Author", "Example"),
        ("Uri", r"\GameOver"),
        ("LastRegisteredUTC", "2022-03-10T20:59:03.583034500Z"),
        ("LastRunUTC", "2022-03-10T21:39:00.616591100Z"),
        ("LastSuccessfulRunUTC", ""),
        ("LastResult", "0x80070002"),
        ("ActionsRaw", ""),
        ("TransactionLogStatus", "clean"),
        ("Problems", ""),
    ] {
        assert_eq!(game_over[column], expected, "{column}");
    }

    let records = taskcache_read(&["-f", hive]);
    let find = |name: &str| {
        records
            .iter()
            .find(|r| r["task_name"] == name)
            .unwrap()
            .clone()
    };
    // the system task: its lists, its last successful run
    let sync = find(r"\Microsoft\Windows\Time Synchronization\SynchronizeTime");
    assert_eq!(sync["task_lists"], json!(["Plain", "Maintenance"]));
    assert_eq!(sync["context"], "LocalService");
    assert_eq!(sync["executable"], r"%windir%\system32\sc.exe");
    assert_eq!(sync["arguments"], "start w32time task_started");
    assert_eq!(
        sync["last_successful_run"],
        "2022-02-13T03:54:43.724066300Z"
    );
    assert_eq!(sync["last_result"], "0x00000420");
    // hidden: no SD
    let falsh = find(r"\falshupdate22");
    assert_eq!(falsh["hidden"], true);
    assert_eq!(falsh["has_security_descriptor"], false);
    assert_eq!(falsh["task_lists"], json!(["Logon"]));
    // an Index 0 entry without a Tasks key: one record, no action
    let defrag = find(r"\Microsoft\Windows\Defrag\ScheduledDefrag");
    assert_eq!(
        (
            &defrag["task_index"],
            &defrag["in_tasks"],
            &defrag["hidden"]
        ),
        (&json!(0), &json!(false), &json!(false))
    );
    assert_eq!(defrag["action_type"], Value::Null);
    // no Tree entry: hidden, its cut Actions kept
    let orphan = find(r"\Orphan");
    assert_eq!(
        (&orphan["in_tree"], &orphan["hidden"]),
        (&json!(false), &json!(true))
    );
    assert_eq!(orphan["has_security_descriptor"], Value::Null);
    assert_eq!(
        orphan["problems"],
        json!(["Actions: a string of 48 bytes at 36 is cut short (20 bytes left)"])
    );
    assert_eq!(orphan["actions_raw"].as_str().unwrap().len(), 120);

    // one task, by its last name or its whole name
    let one = taskcache_read(&["-f", hive, "--task", "gameover"]);
    assert_eq!(one.len(), 1);
    let one = taskcache_read(&[
        "-f",
        hive,
        "--task",
        r"\Microsoft\Windows\Time Synchronization\SynchronizeTime",
    ]);
    assert_eq!(one.len(), 1);
    let r = run(&[
        "parse-registry",
        "taskcache",
        "-f",
        hive,
        "--task",
        "nosuch",
        "-K",
    ]);
    assert!(
        r.stdout.contains("Hives without the task of --task: 1"),
        "{}",
        r.stdout
    );
}

#[test]
fn registry_taskcache_discovery_and_errors() {
    // under -d: the SOFTWARE hive, not the user's hive next to it
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("C/Windows/System32/config");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("SOFTWARE"), taskcache_hive()).unwrap();
    unpack_registry(
        "regipy",
        &["NTUSER.DAT"],
        &dir.path().join("C/Users/vibranium"),
    );
    let r = run(&[
        "parse-registry",
        "taskcache",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(r.stdout.contains("SOFTWARE hives: 1"), "{}", r.stdout);
    // a user's hive is not a SOFTWARE hive
    let ntuser = dir.path().join("C/Users/vibranium/NTUSER.DAT");
    let r = run(&[
        "parse-registry",
        "taskcache",
        "-f",
        ntuser.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("not a SOFTWARE hive"), "{}", r.stdout);
    let r = run(&["parse-registry", "taskcache", "-d", "test_files/lnk", "-K"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("No SOFTWARE hives were found"),
        "{}",
        r.stdout
    );
    // a SOFTWARE hive without a task cache
    let empty = tempfile::tempdir().unwrap();
    let hive = empty.path().join("SOFTWARE");
    std::fs::write(&hive, regf(&reg_key("ROOT", Vec::new(), Vec::new()))).unwrap();
    let r = run(&[
        "parse-registry",
        "taskcache",
        "-f",
        hive.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("Hives without TaskCache: 1"),
        "{}",
        r.stdout
    );
}

// --- parse-registry sam -------------------------------------------------------------------------

/// `parse-registry sam`'s CSV columns.
const SAM_COLUMNS: [&str; 34] = [
    "Host",
    "SourcePath",
    "Rid",
    "Username",
    "FullName",
    "Comment",
    "UserComment",
    "AccountFlags",
    "AccountFlagsValue",
    "CountryCode",
    "CodePage",
    "LastLogonUTC",
    "LastLogonWholeSecondsUTC",
    "PasswordLastSetUTC",
    "AccountExpirationUTC",
    "LastFailedLogonUTC",
    "FailedLogonCount",
    "LogonCount",
    "PasswordHint",
    "InternetUserName",
    "HiddenFromLogonUI",
    "HomeDirectory",
    "HomeDrive",
    "ProfilePath",
    "LogonScript",
    "KeyLastWrittenUTC",
    "NamesKeyLastWrittenUTC",
    "LastLogonFiletime",
    "PasswordLastSetFiletime",
    "AccountExpirationFiletime",
    "LastFailedLogonFiletime",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// `parse-registry sam` with `args`, as JSON Lines.
fn sam_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "sam"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_sam_in_a_collection() {
    // a Windows 11 system's SYSTEM and SAM hives, where they were
    let collection = system_hive();
    let config = collection.path().join("HOST/C/Windows/System32/config");
    unpack_registry("rathbun_w11", &["SAM"], &config);
    let dir = collection.path().to_str().unwrap();
    let r = run(&["parse-registry", "sam", "-d", dir, "-K"]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "SAM hives: 1",
        "Hosts: GOLDPAPERW11",
        "Accounts: 5 (enabled 1, disabled 4)",
        "SAM: 1001 AndrewRathbun (enabled, last logon 2022-02-08T20:43:51.966977200Z, 4 logons)",
        "SAM: 500 Administrator (disabled, last logon never, 0 logons)",
        // the built-in accounts it hides are not counted
        "Accounts people made that the sign-in screen hides: 0",
        "Values that could not be read or don't match: 0",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let user = sam_read(&["-d", dir, "--rid", "1001"]);
    assert_eq!(user.len(), 1);
    for (field, expected) in [
        ("host", json!("GOLDPAPERW11")),
        ("rid", json!(1001)),
        ("username", json!("AndrewRathbun")),
        (
            "account_flags",
            json!([
                "password_not_required",
                "normal_account",
                "dont_expire_password"
            ]),
        ),
        ("account_flags_value", json!(532)),
        ("country_code", json!(0)),
        ("code_page", json!(0)),
        ("last_logon", json!("2022-02-08T20:43:51.966977200Z")),
        ("last_logon_whole_seconds", json!("2022-02-08T20:43:51Z")),
        // never set, never expires: null, the numbers kept
        ("password_last_set", Value::Null),
        ("password_last_set_filetime", json!(0)),
        ("account_expiration", Value::Null),
        (
            "account_expiration_filetime",
            json!(0x7FFF_FFFF_FFFF_FFFF_u64),
        ),
        ("logon_count", json!(4)),
        ("failed_logon_count", json!(0)),
        (
            "names_key_last_written",
            json!("2022-02-05T18:59:49.448358500Z"),
        ),
        ("hidden_from_logon_ui", Value::Null),
        ("transaction_log_status", json!("clean")),
        ("problems", json!([])),
    ] {
        assert_eq!(user[0][field], expected, "{field}");
    }
    // the built-in accounts the sign-in screen hides
    let all = sam_read(&["-d", dir]);
    let hidden: Vec<&Value> = all
        .iter()
        .filter(|r| r["hidden_from_logon_ui"] == true)
        .map(|r| &r["rid"])
        .collect();
    assert_eq!(hidden, [&json!(503), &json!(504)]);
}

#[test]
fn registry_sam_of_an_exported_hive() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let hive = dir.path().join("SAM");
    let hive = hive.to_str().unwrap();
    let out = dir.path().join("sam.csv");
    let r = run(&[
        "parse-registry",
        "sam",
        "-f",
        hive,
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("Accounts: 3 (enabled 1, disabled 2)"),
        "{}",
        r.stdout
    );
    let (header, rows) = csv_rows(&out);
    assert_eq!(header, SAM_COLUMNS);
    let admin = &rows[0];
    for (column, expected) in [
        ("Host", ""),
        ("Rid", "500"),
        ("Username", "Administrator"),
        (
            "Comment",
            "Built-in account for administering the computer/domain",
        ),
        (
            "AccountFlags",
            "disabled|normal_account|dont_expire_password",
        ),
        ("AccountFlagsValue", "529"),
        ("CountryCode", "0"),
        ("CodePage", "0"),
        ("LastLogonUTC", "2010-11-20T21:48:12.569244Z"),
        ("LastLogonWholeSecondsUTC", "2010-11-20T21:48:12Z"),
        ("PasswordLastSetUTC", "2010-11-20T21:56:34.743687Z"),
        ("AccountExpirationUTC", ""),
        ("AccountExpirationFiletime", "0"),
        ("LastFailedLogonUTC", ""),
        ("FailedLogonCount", "0"),
        ("LogonCount", "6"),
        ("HiddenFromLogonUI", ""),
        ("HomeDrive", ""),
        ("KeyLastWrittenUTC", "2014-09-24T06:32:50.378042400Z"),
        ("NamesKeyLastWrittenUTC", "2014-09-24T03:36:06.358837400Z"),
        ("TransactionLogStatus", "clean"),
        ("Problems", ""),
    ] {
        assert_eq!(admin[column], expected, "{column}");
    }
    let guest = sam_read(&["-f", hive, "--rid", "501"]);
    assert_eq!(guest[0]["username"], "Guest");
    assert_eq!(
        guest[0]["account_expiration_filetime"],
        json!(0x7FFF_FFFF_FFFF_FFFF_u64)
    );
    let r = run(&["parse-registry", "sam", "-f", hive, "--rid", "9999", "-K"]);
    assert!(
        r.stdout.contains("Hives without the account of --rid: 1"),
        "{}",
        r.stdout
    );
}

/// A SAM hive with an account hidden from the sign-in screen, with a password hint and a
/// Microsoft account, and an account whose Names entry has another name.
fn sam_hive() -> Vec<u8> {
    let f = |last_logon: u64, rid: u32, flags: u16, logons: u16| {
        let mut f = vec![0u8; 80];
        f[0x08..0x10].copy_from_slice(&last_logon.to_le_bytes());
        f[0x20..0x28].copy_from_slice(&0x7FFF_FFFF_FFFF_FFFF_u64.to_le_bytes());
        f[0x30..0x34].copy_from_slice(&rid.to_le_bytes());
        f[0x38..0x3A].copy_from_slice(&flags.to_le_bytes());
        f[0x42..0x44].copy_from_slice(&logons.to_le_bytes());
        f
    };
    let v = |name: &str, home_drive: Option<&str>| {
        let name: Vec<u8> = name.encode_utf16().flat_map(u16::to_le_bytes).collect();
        let mut v = vec![0u8; 0xCC];
        v[16..20].copy_from_slice(&(name.len() as u32).to_le_bytes());
        v.extend(&name);
        if let Some(home_drive) = home_drive {
            let home_drive: Vec<u8> = home_drive
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect();
            v[7 * 12..7 * 12 + 4].copy_from_slice(&(name.len() as u32).to_le_bytes());
            v[7 * 12 + 4..7 * 12 + 8].copy_from_slice(&(home_drive.len() as u32).to_le_bytes());
            v.extend(home_drive);
        }
        v
    };
    let wide = |s: &str| -> Vec<u8> { s.encode_utf16().flat_map(u16::to_le_bytes).collect() };
    let support = reg_key(
        "000003E9",
        vec![
            ("F", REG_BINARY, f(LINKED, 1001, 0x0210, 3)),
            ("V", REG_BINARY, v("support", Some("H:"))),
            ("UserPasswordHint", REG_BINARY, wide("the usual")),
            ("InternetUserName", REG_BINARY, wide("support@example.com")),
            (
                "UserDontShowInLogonUI",
                REG_BINARY,
                1u32.to_le_bytes().to_vec(),
            ),
        ],
        Vec::new(),
    );
    let bob = reg_key(
        "000003EA",
        vec![
            ("F", REG_BINARY, f(0, 1002, 0x0011, 0)),
            ("V", REG_BINARY, v("bob", None)),
        ],
        Vec::new(),
    );
    // A partial account whose F value is missing: its state must not be called enabled.
    let partial = reg_key(
        "000003EB",
        vec![("V", REG_BINARY, v("partial", None))],
        Vec::new(),
    );
    let names = reg_key(
        "Names",
        Vec::new(),
        vec![
            reg_key("robert", vec![("", 1002, Vec::new())], Vec::new()),
            reg_key("support", vec![("", 1001, Vec::new())], Vec::new()),
            reg_key("partial", vec![("", 1003, Vec::new())], Vec::new()),
        ],
    );
    let mut key = reg_key("Users", Vec::new(), vec![support, bob, partial, names]);
    for name in ["Account", "Domains", "SAM"] {
        key = reg_key(name, Vec::new(), vec![key]);
    }
    regf(&reg_key("ROOT", Vec::new(), vec![key]))
}

#[test]
fn registry_sam_hidden_accounts_and_errors() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("C/Windows/System32/config");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("SAM"), sam_hive()).unwrap();
    unpack_registry("regipy", &["NTUSER.DAT"], &dir.path().join("C/Users/bob"));
    let dir = dir.path().to_str().unwrap();
    // under -d: the SAM hive, not the user's hive
    let r = run(&["parse-registry", "sam", "-d", dir, "-K"]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    for line in [
        "SAM hives: 1",
        "Accounts: 3 (enabled 1, disabled 1, unknown 1)",
        "Accounts people made that the sign-in screen hides: 1",
        "SAM: 1001 support",
        "its Names entry is robert, not its user name",
        "Values that could not be read or don't match: 2",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let records = sam_read(&["-d", dir]);
    let support = &records[0];
    assert_eq!(support["hidden_from_logon_ui"], true);
    assert_eq!(support["password_hint"], "the usual");
    assert_eq!(support["internet_user_name"], "support@example.com");
    assert_eq!(support["home_drive"], "H:");
    assert_eq!(support["last_logon"], "2022-02-08T21:03:52Z");
    assert_eq!(
        support["account_flags"],
        json!(["normal_account", "dont_expire_password"])
    );
    let partial = &records[2];
    assert_eq!(partial["rid"], 1003);
    assert_eq!(partial["account_flags"], Value::Null);
    assert_eq!(partial["account_flags_value"], Value::Null);
    assert_eq!(partial["problems"], json!(["F is not there"]));

    let ntuser = std::path::Path::new(dir).join("C/Users/bob/NTUSER.DAT");
    let r = run(&[
        "parse-registry",
        "sam",
        "-f",
        ntuser.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("not a SAM hive"), "{}", r.stdout);
    let r = run(&["parse-registry", "sam", "-d", "test_files/lnk", "-K"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("No SAM hives were found"), "{}", r.stdout);
}

// --- parse-ntfs i30 --------------------------------------------------------------------------------

const LOGS_I30: &str = "test_files/i30/Logs_I30";
const PHOTOS_I30: &str = "test_files/i30/Photos_I30";

/// A triage-like tree: the two indexes, named as they are exported, and a file that is not one.
fn i30_triage() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let host = dir.path().join("HOST/C");
    std::fs::create_dir_all(&host).unwrap();
    std::fs::copy(repo(LOGS_I30), host.join("Logs_$I30")).unwrap();
    std::fs::copy(repo(PHOTOS_I30), host.join("写真_$I30")).unwrap();
    std::fs::write(host.join("notes.txt"), b"not an index").unwrap();
    dir
}

#[test]
fn i30_summary_on_the_terminal() {
    let r = run(&["parse-ntfs", "i30", "-f", LOGS_I30, "-K"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for line in [
        "$I30 files: 1",
        "$I30 files parsed: 1",
        "INDX records: 7\n",
        "Entries in use: 60",
        "Entries in slack: 157 (copies of entries in use: 96, without their header: 67, without their parent: 11)",
        "Non-copy names in slack (possible deleted or renamed files): 51",
        "  app-020.log\n",
        "  app-058.log\n  … 31 more\n",
        "To save every entry, add -o <FILE>",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    assert!(!r.stdout.contains("Problems in records"), "{}", r.stdout);

    // several files: each name with its file
    let dir = i30_triage();
    let r = run(&[
        "parse-ntfs",
        "i30",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    // as the directory is read: with the platform's separators
    let logs = dir.path().join("HOST").join("C").join("Logs_$I30");
    for line in [
        "$I30 files: 2".to_string(),
        "$I30 files parsed: 2".to_string(),
        "INDX records: 8\n".to_string(),
        "Entries in use: 64".to_string(),
        "Entries in slack: 159 (copies of entries in use: 97, without their header: 69, without their parent: 12)".to_string(),
        "Non-copy names in slack (possible deleted or renamed files): 52".to_string(),
        format!("  {}: app-020.log\n", logs.display()),
        "  … 32 more\n".to_string(),
    ] {
        assert!(r.stdout.contains(&line), "{line}\n{}", r.stdout);
    }
}

#[test]
fn i30_csv_json_and_jsonl() {
    let dir = i30_triage();
    let out = tempfile::tempdir().unwrap();
    let csv = out.path().join("i30.csv");
    let r = run(&[
        "parse-ntfs",
        "i30",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        csv.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(
        r.stdout
            .contains(&format!("Results saved: {} (223 entries, ", csv.display())),
        "{}",
        r.stdout
    );
    let text = std::fs::read_to_string(&csv).unwrap();
    let mut rows = csv::Reader::from_reader(text.as_bytes());
    assert_eq!(
        rows.headers().unwrap().iter().collect::<Vec<_>>(),
        [
            "SourcePath",
            "RecordOffset",
            "RecordVcn",
            "RecordLsn",
            "EntryOffset",
            "FromSlack",
            "CopyOfEntryInUse",
            "FileName",
            "Namespace",
            "FileEntry",
            "FileSequence",
            "ParentEntry",
            "ParentSequence",
            "CreatedUTC",
            "ModifiedUTC",
            "MftModifiedUTC",
            "AccessedUTC",
            "AllocatedSize",
            "RealSize",
            "FileAttributes",
            "ReparseValue",
            "Problems",
        ]
    );
    let rows: Vec<csv::StringRecord> = rows.records().map(|r| r.unwrap()).collect();
    assert_eq!(rows.len(), 223);
    // the renamed photo's old name: its entry header written over, its parent kept
    let sakura = rows.iter().find(|r| &r[7] == "桜.jpg").unwrap();
    assert!(sakura[0].ends_with("写真_$I30"));
    assert_eq!(
        sakura.iter().skip(4).take(9).collect::<Vec<_>>(),
        ["576", "True", "False", "桜.jpg", "posix", "", "", "65", "1"]
    );
    assert_eq!(
        (&sakura[17], &sakura[18], &sakura[19]),
        ("8", "4", "archive")
    );

    // JSON: one pretty document per entry
    let json = out.path().join("i30.json");
    let r = run(&[
        "parse-ntfs",
        "i30",
        "-f",
        LOGS_I30,
        "-t",
        "json",
        "-o",
        json.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let records = json_documents(&std::fs::read_to_string(&json).unwrap());
    assert_eq!(records.len(), 217);
    let deleted = records
        .iter()
        .find(|r| r["file_name"] == "app-022.log")
        .unwrap();
    // the fields in their order
    let text = std::fs::read_to_string(&json).unwrap();
    let first = &text[..text.find("\n}").unwrap()];
    let at = |key: &str| first.find(&format!("\"{key}\":")).unwrap();
    let order = [
        "source_path",
        "record_offset",
        "record_vcn",
        "record_lsn",
        "entry_offset",
        "from_slack",
        "copy_of_entry_in_use",
        "file_name",
        "namespace",
        "file_entry",
        "file_sequence",
        "parent_entry",
        "parent_sequence",
        "created",
        "modified",
        "mft_modified",
        "accessed",
        "allocated_size",
        "real_size",
        "file_attributes",
        "reparse_value",
        "problems",
    ];
    assert!(order.windows(2).all(|w| at(w[0]) < at(w[1])), "{first}");
    assert_eq!(deleted["from_slack"], true);
    assert_eq!(deleted["copy_of_entry_in_use"], false);
    assert_eq!(deleted["file_entry"], 87);
    assert_eq!(deleted["parent_entry"], 64);
    assert_eq!(
        (&deleted["real_size"], &deleted["allocated_size"]),
        (&json!(8), &json!(8))
    );
    assert_eq!(deleted["namespace"], "posix");
    assert_eq!(deleted["file_attributes"], "archive");
    assert_eq!(deleted["problems"], json!([]));
    for time in ["created", "modified", "mft_modified", "accessed"] {
        assert!(
            deleted[time].as_str().unwrap().ends_with('Z'),
            "{time}: {}",
            deleted[time]
        );
    }
    // the password list's copies, their parent written over
    let passwords: Vec<&Value> = records
        .iter()
        .filter(|r| r["file_name"] == "passwords.txt")
        .collect();
    assert_eq!(passwords.len(), 11);
    assert!(
        passwords
            .iter()
            .all(|r| r["parent_entry"].is_null() && r["file_entry"].is_null())
    );

    // JSONL on stdout: nothing but the entries
    let r = run(&["parse-ntfs", "i30", "-f", PHOTOS_I30, "-t", "jsonl", "-K"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let lines: Vec<Value> = r
        .stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let names: Vec<(&str, bool, bool)> = lines
        .iter()
        .map(|r| {
            (
                r["file_name"].as_str().unwrap(),
                r["from_slack"].as_bool().unwrap(),
                r["copy_of_entry_in_use"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        names,
        [
            ("さくら.jpg", false, false),
            ("京都の秋.jpg", false, false),
            ("大阪城.jpg", false, false),
            ("東京タワー.jpg", false, false),
            ("東京タワー.jpg", true, true),
            ("桜.jpg", true, false),
        ]
    );
}

#[test]
fn i30_slack_only_and_no_copies() {
    let count = |extra: &[&str]| {
        let mut args = vec!["parse-ntfs", "i30", "-f", LOGS_I30, "-t", "jsonl", "-K"];
        args.extend_from_slice(extra);
        let r = run(&args);
        assert_eq!(r.code, 0, "{}", r.stderr);
        r.stdout.lines().count()
    };
    assert_eq!(count(&[]), 217);
    assert_eq!(count(&["--slack-only"]), 157);
    assert_eq!(count(&["--no-copies"]), 121);
    assert_eq!(count(&["--slack-only", "--no-copies"]), 61);

    // the summary is the index's, whatever is written
    let out = tempfile::tempdir().unwrap();
    let csv = out.path().join("slack.csv");
    let r = run(&[
        "parse-ntfs",
        "i30",
        "-f",
        LOGS_I30,
        "--slack-only",
        "-o",
        csv.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.contains("Entries in use: 60"), "{}", r.stdout);
    assert!(r.stdout.contains("(157 entries, "), "{}", r.stdout);
}

#[test]
fn i30_records_in_other_data_and_records_written_in_part() {
    // the photos' record 0x30200 bytes into other data, as in a disk image
    let dir = tempfile::tempdir().unwrap();
    let image = dir.path().join("disk.img");
    let mut data = vec![0x5Au8; 0x30200];
    data.extend(std::fs::read(repo(PHOTOS_I30)).unwrap());
    data.extend(vec![0u8; 0x1000]);
    std::fs::write(&image, &data).unwrap();
    let r = run(&[
        "parse-ntfs",
        "i30",
        "-f",
        image.to_str().unwrap(),
        "-t",
        "jsonl",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let lines: Vec<Value> = r
        .stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 6);
    assert!(lines.iter().all(|r| r["record_offset"] == 0x30200));
    assert_eq!(lines[5]["entry_offset"], 0x30200 + 576);

    // the end of the second record's first sector not written
    let torn = dir.path().join("Logs_$I30");
    let mut logs = std::fs::read(repo(LOGS_I30)).unwrap();
    logs[4096 + 510] ^= 0xFF;
    std::fs::write(&torn, &logs).unwrap();
    let r = run(&["parse-ntfs", "i30", "-f", torn.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for line in [
        "INDX records: 7 (written in part: 1)".to_string(),
        "Problems in records: 1".to_string(),
        format!(
            "  {}: the record at 0x1000: sectors 0 don't end with the update sequence value: the record was written in part",
            torn.display()
        ),
    ] {
        assert!(r.stdout.contains(&line), "{line}\n{}", r.stdout);
    }
}

#[test]
fn i30_input_and_output_errors() {
    let r = run(&["parse-ntfs", "i30", "-f", "test_files/i30/missing", "-K"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("The file does not exist"), "{}", r.stdout);

    let r = run(&["parse-ntfs", "i30", "-f", "test_files/lnk/README.md", "-K"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("Could not parse") && r.stdout.contains("no INDX records"),
        "{}",
        r.stdout
    );

    let r = run(&["parse-ntfs", "i30", "-d", "test_files/lnk", "-K"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("No $I30 files were found"),
        "{}",
        r.stdout
    );

    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("evidence_$I30");
    std::fs::copy(repo(LOGS_I30), &input).unwrap();
    let original = std::fs::read(&input).unwrap();
    let r = run(&[
        "parse-ntfs",
        "i30",
        "-f",
        input.to_str().unwrap(),
        "-o",
        input.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("also an input file"), "{}", r.stdout);
    assert_eq!(std::fs::read(&input).unwrap(), original);

    // A different path to the same file is just as destructive.
    let linked_output = dir.path().join("linked.csv");
    std::fs::hard_link(&input, &linked_output).unwrap();
    let r = run(&[
        "parse-ntfs",
        "i30",
        "-f",
        input.to_str().unwrap(),
        "-o",
        linked_output.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("also an input file"), "{}", r.stdout);
    assert_eq!(std::fs::read(&input).unwrap(), original);

    let file = dir.path().join("keep.csv");
    std::fs::write(&file, b"keep").unwrap();
    let r = run(&[
        "parse-ntfs",
        "i30",
        "-f",
        LOGS_I30,
        "-o",
        file.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("already exists"), "{}", r.stdout);
    assert_eq!(std::fs::read(&file).unwrap(), b"keep");
    let r = run(&[
        "parse-ntfs",
        "i30",
        "-f",
        LOGS_I30,
        "-o",
        file.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
}

// --- parse-registry persistence -----------------------------------------------------------------

/// `parse-registry persistence`'s CSV columns.
const PERSISTENCE_COLUMNS: [&str; 22] = [
    "Host",
    "SourcePath",
    "HiveType",
    "User",
    "UserSid",
    "Mechanism",
    "AttackTechnique",
    "KeyPath",
    "ValueName",
    "Data",
    "ImagePath",
    "Arguments",
    "Enabled",
    "DisabledAtUTC",
    "Details",
    "OverridesMachineRegistration",
    "Wow64",
    "KeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// A hive whose root is named `root`, with the values `(key path, name, type, data)` and the
/// keys along their paths, each last written at `LINKED`.
fn reg_tree(root: &str, values: &[(&str, &str, u32, Vec<u8>)]) -> Vec<u8> {
    #[derive(Default)]
    struct Node {
        keys: Vec<(String, Node)>,
        values: Vec<(String, u32, Vec<u8>)>,
    }
    fn build(name: &str, node: &Node) -> RegKey {
        RegKey {
            name: name.to_string(),
            values: node.values.clone(),
            subkeys: node.keys.iter().map(|(n, c)| build(n, c)).collect(),
            written: LINKED,
        }
    }
    let mut tree = Node::default();
    for (path, name, kind, data) in values {
        let mut node = &mut tree;
        for part in path.split('\\') {
            let i = if let Some(i) = node.keys.iter().position(|(n, _)| n == part) {
                i
            } else {
                node.keys.push((part.to_string(), Node::default()));
                node.keys.len() - 1
            };
            node = &mut node.keys[i].1;
        }
        node.values.push((name.to_string(), *kind, data.clone()));
    }
    regf(&build(root, &tree))
}

fn multi_sz(items: &[&str]) -> Vec<u8> {
    let mut data: Vec<u8> = items.iter().flat_map(|i| utf16z(i)).collect();
    data.extend([0, 0]);
    data
}

/// A `StartupApproved` entry: enabled, or disabled at `LINKED`.
fn approved(enabled: bool) -> Vec<u8> {
    let mut b = vec![0u8; 12];
    if enabled {
        b[0] = 2;
    } else {
        b[0] = 3;
        b[4..].copy_from_slice(&LINKED.to_le_bytes());
    }
    b
}

/// One system's SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat, where Windows keeps them under
/// `root`: Run keys with a disabled entry and one only `StartupApproved` names, Startup folder
/// files enabled and disabled, an IFEO debugger,
/// LSA packages in two control sets, and a user's COM classes, one of them overriding the
/// machine's.
fn persistence_collection(root: &Path) {
    let cv = r"Microsoft\Windows\CurrentVersion";
    let approved_run = format!(r"{cv}\Explorer\StartupApproved\Run");
    let run = format!(r"{cv}\Run");
    let software = reg_tree(
        "ROOT",
        &[
            (
                &run,
                "Updater",
                REG_SZ,
                utf16z(r#""C:\Program Files\Up\up.exe" /bg"#),
            ),
            (&run, "Off", REG_SZ, utf16z(r"C:\Tools\off.exe")),
            (&approved_run, "Off", REG_BINARY, approved(false)),
            (&approved_run, "Gone", REG_BINARY, approved(false)),
            (
                r"Microsoft\Windows NT\CurrentVersion\Image File Execution Options\sethc.exe",
                "Debugger",
                REG_SZ,
                utf16z(r"C:\Windows\System32\cmd.exe"),
            ),
            (r"Classes\CLSID\{AAAA}", "", REG_SZ, utf16z("Shell thing")),
            (
                r"Microsoft\Windows NT\CurrentVersion\ProfileList\S-1-5-21-1-2-3-1001",
                "ProfileImagePath",
                REG_EXPAND_SZ,
                utf16z(r"C:\Users\alice"),
            ),
        ],
    );
    let system = reg_tree(
        "ROOT",
        &[
            ("Select", "Current", REG_DWORD, 1u32.to_le_bytes().to_vec()),
            (
                r"ControlSet001\Control\ComputerName\ComputerName",
                "ComputerName",
                REG_SZ,
                utf16z("HOST1"),
            ),
            (
                r"ControlSet001\Control\Lsa",
                "Notification Packages",
                REG_MULTI_SZ,
                multi_sz(&["scecli", "pwfilter"]),
            ),
            (
                r"ControlSet002\Control\Lsa",
                "Notification Packages",
                REG_MULTI_SZ,
                multi_sz(&["scecli"]),
            ),
        ],
    );
    let ntuser = reg_tree(
        "ROOT",
        &[
            (
                r"Software\Microsoft\Windows\CurrentVersion\Run",
                "Sync",
                REG_SZ,
                utf16z(r"C:\Users\alice\sync.exe /background"),
            ),
            (
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder",
                "Good.lnk",
                REG_BINARY,
                approved(true),
            ),
            (
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder",
                "Off.lnk",
                REG_BINARY,
                approved(false),
            ),
            (
                "Environment",
                "UserInitMprLogonScript",
                REG_SZ,
                utf16z(r"C:\Users\alice\logon.bat"),
            ),
        ],
    );
    let classes = reg_tree(
        "S-1-5-21-1-2-3-1001_Classes",
        &[
            (
                r"CLSID\{AAAA}\InprocServer32",
                "",
                REG_EXPAND_SZ,
                utf16z(r"C:\Users\alice\h.dll"),
            ),
            (
                r"CLSID\{BBBB}\LocalServer32",
                "",
                REG_SZ,
                utf16z(r#""C:\Users\alice\own.exe" -Embedding"#),
            ),
        ],
    );
    let config = root.join("C/Windows/System32/config");
    let profile = root.join("C/Users/alice");
    let local = profile.join("AppData/Local/Microsoft/Windows");
    for dir in [&config, &local] {
        std::fs::create_dir_all(dir).unwrap();
    }
    std::fs::write(config.join("SOFTWARE"), software).unwrap();
    std::fs::write(config.join("SYSTEM"), system).unwrap();
    std::fs::write(profile.join("NTUSER.DAT"), ntuser).unwrap();
    std::fs::write(local.join("UsrClass.dat"), classes).unwrap();
}

/// `parse-registry persistence` with `args`, as JSON Lines.
fn persistence_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "persistence"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

/// Each record as `hive mechanism key\value = data`.
fn persistence_lines(records: &[Value]) -> Vec<String> {
    records
        .iter()
        .map(|r| {
            format!(
                r"{} {} {}\{} = {}",
                r["hive_type"].as_str().unwrap(),
                r["mechanism"].as_str().unwrap(),
                r["key_path"].as_str().unwrap(),
                r["value_name"].as_str().unwrap(),
                r["data"].as_str().unwrap_or("-")
            )
        })
        .collect()
}

#[test]
fn registry_persistence_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    persistence_collection(dir.path());
    let records = persistence_read(&["-d", dir.path().to_str().unwrap()]);
    assert_eq!(
        persistence_lines(&records),
        [
            r"UsrClass.dat com_server CLSID\{AAAA}\InprocServer32\ = C:\Users\alice\h.dll",
            r#"UsrClass.dat com_server CLSID\{BBBB}\LocalServer32\ = "C:\Users\alice\own.exe" -Embedding"#,
            r"NTUSER.DAT run Software\Microsoft\Windows\CurrentVersion\Run\Sync = C:\Users\alice\sync.exe /background",
            r"NTUSER.DAT startup_folder Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder\Good.lnk = -",
            r"NTUSER.DAT startup_folder Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder\Off.lnk = -",
            r"NTUSER.DAT logon_script Environment\UserInitMprLogonScript = C:\Users\alice\logon.bat",
            r#"SOFTWARE run Microsoft\Windows\CurrentVersion\Run\Updater = "C:\Program Files\Up\up.exe" /bg"#,
            r"SOFTWARE run Microsoft\Windows\CurrentVersion\Run\Off = C:\Tools\off.exe",
            r"SOFTWARE run Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run\Gone = -",
            r"SOFTWARE ifeo_debugger Microsoft\Windows NT\CurrentVersion\Image File Execution Options\sethc.exe\Debugger = C:\Windows\System32\cmd.exe",
            r"SYSTEM lsa_notification_package ControlSet001\Control\Lsa\Notification Packages = scecli",
            r"SYSTEM lsa_notification_package ControlSet001\Control\Lsa\Notification Packages = pwfilter",
        ]
    );
    assert!(records.iter().all(|r| r["host"] == "HOST1"));
    // the user hives' user and SID: UsrClass.dat's root name, NTUSER.DAT's profile in ProfileList
    for r in &records[..6] {
        assert_eq!(r["user"], "alice");
        assert_eq!(r["user_sid"], "S-1-5-21-1-2-3-1001");
    }
    assert!(records[6..].iter().all(|r| r["user"].is_null()));
    // the user's class {AAAA} overrides the machine's
    assert_eq!(records[0]["overrides_machine_registration"], true);
    assert_eq!(records[1]["overrides_machine_registration"], false);
    assert_eq!(records[1]["image_path"], r"C:\Users\alice\own.exe");
    assert_eq!(records[1]["arguments"], "-Embedding");
    assert_eq!(records[0]["attack_technique"], "T1546.015");
    // StartupApproved: Startup folder files enabled and disabled; Updater has no entry, Off is
    // disabled, Gone has no Run value
    assert_eq!(records[3]["enabled"], true);
    assert!(records[3]["disabled_at"].is_null());
    assert_eq!(records[3]["details"], "a file in the Startup folder");
    assert_eq!(records[4]["enabled"], false);
    assert_eq!(records[4]["disabled_at"], "2022-02-08T21:03:52Z");
    assert!(records[6]["enabled"].is_null());
    assert_eq!(records[7]["enabled"], false);
    assert_eq!(records[7]["disabled_at"], "2022-02-08T21:03:52Z");
    assert_eq!(records[8]["enabled"], false);
    assert_eq!(
        records[8]["details"],
        "only in StartupApproved: the Run key has no value of this name"
    );
    assert!(
        records
            .iter()
            .all(|r| r["transaction_log_status"] == "clean" && r["problems"] == json!([]))
    );
    assert!(
        records
            .iter()
            .all(|r| r["key_last_written"] == "2022-02-08T21:03:52Z")
    );

    // every control set
    let records = persistence_read(&["-d", dir.path().to_str().unwrap(), "--all-control-sets"]);
    assert_eq!(
        persistence_lines(&records)[10..],
        [
            r"SYSTEM lsa_notification_package ControlSet001\Control\Lsa\Notification Packages = scecli",
            r"SYSTEM lsa_notification_package ControlSet001\Control\Lsa\Notification Packages = pwfilter",
            r"SYSTEM lsa_notification_package ControlSet002\Control\Lsa\Notification Packages = scecli",
        ]
    );
}

#[test]
fn registry_persistence_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    persistence_collection(dir.path());
    let out = dir.path().join("persistence.csv");
    let r = run(&[
        "parse-registry",
        "persistence",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "Hives parsed: 4 (NTUSER.DAT 1, SOFTWARE 1, SYSTEM 1, UsrClass.dat 1)",
        "Hosts: HOST1",
        "Users: alice",
        "Autostarts: 12",
        "  com_server: 2",
        "  run: 4",
        "  startup_folder: 2",
        "Disabled: 3",
        "StartupApproved Run entries without their Run value: 1\n",
        r"StartupApproved\Run\Gone",
        "A user's COM classes and commands overriding the machine's: 1",
        r"CLSID\{AAAA}\InprocServer32\ = C:\Users\alice\h.dll",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    // Startup folder files have no Run value to lose
    assert!(!r.stdout.contains("Good.lnk"), "{}", r.stdout);
    assert!(!r.stdout.contains("Off.lnk"), "{}", r.stdout);
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let header: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, PERSISTENCE_COLUMNS);
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(rows.len(), 12);
    assert_eq!(&rows[0][2], "UsrClass.dat");
    assert_eq!(&rows[0][15], "True");
    assert_eq!(&rows[7][12], "False");

    // an exported user hive without its system's SOFTWARE hive: overrides unknown
    let alone = dir.path().join("alone");
    std::fs::create_dir_all(&alone).unwrap();
    std::fs::copy(
        dir.path()
            .join("C/Users/alice/AppData/Local/Microsoft/Windows/UsrClass.dat"),
        alone.join("UsrClass.dat"),
    )
    .unwrap();
    let records = persistence_read(&["-f", alone.join("UsrClass.dat").to_str().unwrap()]);
    assert_eq!(records.len(), 2);
    assert!(
        records
            .iter()
            .all(|r| r["overrides_machine_registration"].is_null()
                && r["user_sid"] == "S-1-5-21-1-2-3-1001")
    );
    let r = run(&[
        "parse-registry",
        "persistence",
        "-f",
        alone.join("UsrClass.dat").to_str().unwrap(),
        "-K",
    ]);
    assert!(
        r.stdout
            .contains("User hives without their system's SOFTWARE hive"),
        "{}",
        r.stdout
    );
}

#[test]
fn registry_persistence_of_public_hives() {
    let dir = tempfile::tempdir().unwrap();
    // regipy's Windows 7 NTUSER.DAT: the Run value regipy's ntuser_persistence reads
    unpack_registry("regipy", &["NTUSER.DAT"], dir.path());
    let records = persistence_read(&["-f", dir.path().join("NTUSER.DAT").to_str().unwrap()]);
    assert_eq!(
        persistence_lines(&records),
        [
            r"NTUSER.DAT run Software\Microsoft\Windows\CurrentVersion\Run\Sidebar = %ProgramFiles%\Windows Sidebar\Sidebar.exe /autoRun"
        ]
    );
    assert_eq!(
        records[0]["image_path"],
        r"%ProgramFiles%\Windows Sidebar\Sidebar.exe"
    );
    assert_eq!(records[0]["arguments"], "/autoRun");

    // regipy's Windows 7 SYSTEM: the LSA packages regipy's lsa_packages reads
    use std::io::Read as _;
    let gz = std::fs::File::open(repo("test_files/shimcache/regipy/SYSTEM.gz")).unwrap();
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(gz)
        .read_to_end(&mut bytes)
        .unwrap();
    let system = dir.path().join("SYSTEM");
    std::fs::write(&system, bytes).unwrap();
    let records = persistence_read(&["-f", system.to_str().unwrap()]);
    let packages = |mechanism: &str| -> Vec<&str> {
        records
            .iter()
            .filter(|r| r["mechanism"] == mechanism)
            .map(|r| r["data"].as_str().unwrap())
            .collect()
    };
    assert_eq!(packages("lsa_authentication_package"), ["msv1_0"]);
    assert_eq!(
        packages("lsa_security_package"),
        [
            "kerberos", "msv1_0", "schannel", "wdigest", "tspkg", "pku2u"
        ]
    );
    assert_eq!(packages("lsa_notification_package"), ["scecli"]);
    assert_eq!(packages("known_dlls").len(), 28);

    // the DFIR Artifact Museum's dirty Windows 11 NTUSER.DAT, with its logs
    let w11 = dir.path().join("w11");
    unpack_registry(
        "rathbun_w11",
        &["NTUSER.DAT", "ntuser.dat.LOG1", "ntuser.dat.LOG2"],
        &w11,
    );
    let records = persistence_read(&["-f", w11.join("NTUSER.DAT").to_str().unwrap()]);
    assert!(!records.is_empty());
    assert!(records.iter().all(|r| {
        ["replayed", "unchanged"].contains(&r["transaction_log_status"].as_str().unwrap())
    }));
}

#[test]
fn registry_persistence_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let r = run(&[
        "parse-registry",
        "persistence",
        "-f",
        dir.path().join("SAM").to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr
            .contains("a SAM hive, not a SOFTWARE, SYSTEM, NTUSER.DAT or UsrClass.dat hive")
            || r.stdout
                .contains("a SAM hive, not a SOFTWARE, SYSTEM, NTUSER.DAT or UsrClass.dat hive"),
        "{}{}",
        r.stdout,
        r.stderr
    );
    // a directory with a SAM hive only
    let r = run(&[
        "parse-registry",
        "persistence",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let all = format!("{}{}", r.stdout, r.stderr);
    assert!(
        all.contains("No SOFTWARE, SYSTEM, NTUSER.DAT or UsrClass.dat hives were found"),
        "{all}"
    );
    let r = run(&["parse-registry", "persistence", "-h"]);
    assert_eq!(r.code, 0);
    assert!(r.stdout.contains("--all-control-sets"), "{}", r.stdout);
}

#[test]
fn registry_output_never_overwrites_a_file_read() {
    let dir = tempfile::tempdir().unwrap();
    persistence_collection(dir.path());
    let config = dir.path().join("C/Windows/System32/config");
    let software = config.join("SOFTWARE");
    let ntuser = dir.path().join("C/Users/alice/NTUSER.DAT");
    let log = config.join("SOFTWARE.LOG1");
    std::fs::write(&log, vec![0u8; 512]).unwrap();
    let given_log = dir.path().join("given.LOG1");
    std::fs::write(&given_log, vec![0u8; 512]).unwrap();
    let hard_link = dir.path().join("hard.csv");
    std::fs::hard_link(&software, &hard_link).unwrap();
    let originals: Vec<(PathBuf, Vec<u8>)> = [&software, &config.join("SYSTEM"), &log, &given_log]
        .iter()
        .map(|p| ((*p).clone(), std::fs::read(p).unwrap()))
        .collect();
    let refused = |args: &[&str]| {
        let mut all = args.to_vec();
        all.extend(["-C", "-K", "-q"]);
        let r = run(&all);
        assert_eq!(r.code, 1, "{all:?}\n{}{}", r.stdout, r.stderr);
        let said = format!("{}{}", r.stdout, r.stderr);
        assert!(
            said.contains("The output file is also an input file and cannot be overwritten"),
            "{all:?}\n{said}"
        );
        for (path, bytes) in &originals {
            assert_eq!(&std::fs::read(path).unwrap(), bytes, "{}", path.display());
        }
    };
    let s = |p: &Path| p.to_str().unwrap().to_owned();
    let (software_s, ntuser_s) = (s(&software), s(&ntuser));
    // the hive itself, by its path and by a hard link
    refused(&[
        "parse-registry",
        "persistence",
        "-f",
        &software_s,
        "-o",
        &software_s,
    ]);
    refused(&[
        "parse-registry",
        "persistence",
        "-f",
        &software_s,
        "-o",
        &s(&hard_link),
    ]);
    // its log next to it, and a log given
    refused(&[
        "parse-registry",
        "persistence",
        "-f",
        &software_s,
        "-o",
        &s(&log),
    ]);
    refused(&[
        "parse-registry",
        "persistence",
        "-f",
        &ntuser_s,
        "--log1",
        &s(&given_log),
        "-o",
        &s(&given_log),
    ]);
    // the system's SOFTWARE and SYSTEM hives a user hive is read with
    refused(&[
        "parse-registry",
        "persistence",
        "-f",
        &ntuser_s,
        "-o",
        &software_s,
    ]);
    refused(&[
        "parse-registry",
        "persistence",
        "-f",
        &ntuser_s,
        "-o",
        &s(&config.join("SYSTEM")),
    ]);
    // every registry command shares the check
    refused(&[
        "parse-registry",
        "taskcache",
        "-f",
        &software_s,
        "-o",
        &software_s,
    ]);
    refused(&[
        "parse-registry",
        "read",
        "-d",
        &s(dir.path()),
        "-o",
        &s(&hard_link),
    ]);
    #[cfg(unix)]
    {
        let link = dir.path().join("link.csv");
        std::os::unix::fs::symlink(&software, &link).unwrap();
        refused(&[
            "parse-registry",
            "persistence",
            "-f",
            &software_s,
            "-o",
            &s(&link),
        ]);
        refused(&[
            "parse-registry",
            "services",
            "-d",
            &s(dir.path()),
            "-o",
            &s(&link),
        ]);
    }
    // an output that is no input is written
    let out = dir.path().join("out.csv");
    std::fs::write(&out, b"old").unwrap();
    let r = run(&[
        "parse-registry",
        "persistence",
        "-f",
        &software_s,
        "-o",
        &s(&out),
        "-C",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "{}{}", r.stdout, r.stderr);
    assert!(std::fs::read_to_string(&out).unwrap().starts_with("Host,"));
}

/// A write-only input can't be opened to tell what it is, yet could be truncated: it is told
/// apart without opening it, and an output that can't be told apart is not written.
#[cfg(unix)]
#[test]
fn registry_output_never_overwrites_a_write_only_input() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    persistence_collection(dir.path());
    let software = dir.path().join("C/Windows/System32/config/SOFTWARE");
    let hard_link = dir.path().join("hard.csv");
    std::fs::hard_link(&software, &hard_link).unwrap();
    let original = std::fs::read(&software).unwrap();
    let set_mode = |path: &Path, mode: u32| {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    };
    let software_s = software.to_str().unwrap();
    set_mode(&software, 0o200);
    let mut said = Vec::new();
    for out in [&software, &hard_link] {
        let r = run(&[
            "parse-registry",
            "persistence",
            "-f",
            software_s,
            "-o",
            out.to_str().unwrap(),
            "-C",
            "-K",
            "-q",
        ]);
        said.push((r.code, format!("{}{}", r.stdout, r.stderr)));
    }
    set_mode(&software, 0o600);
    assert_eq!(std::fs::read(&software).unwrap(), original);
    for (code, said) in said {
        assert_eq!(code, 1, "{said}");
        assert!(
            said.contains("The output file is also an input file and cannot be overwritten"),
            "{said}"
        );
    }

    // an output whose identity can't be read (its directory can't be searched) is not written
    let locked = dir.path().join("locked");
    std::fs::create_dir(&locked).unwrap();
    let out = locked.join("out.csv");
    std::fs::write(&out, b"kept").unwrap();
    set_mode(&locked, 0o600);
    // root can search it anyway: nothing to test then
    let privileged = std::fs::metadata(&out).is_ok();
    let r = run(&[
        "parse-registry",
        "persistence",
        "-f",
        software_s,
        "-o",
        out.to_str().unwrap(),
        "-C",
        "-K",
        "-q",
    ]);
    set_mode(&locked, 0o700);
    if !privileged {
        let said = format!("{}{}", r.stdout, r.stderr);
        assert_eq!(r.code, 1, "{said}");
        assert!(
            said.contains("Could not tell whether the output file"),
            "{said}"
        );
        assert_eq!(std::fs::read(&out).unwrap(), b"kept");
    }
}

// --- parse-registry userassist ------------------------------------------------------------------

/// `parse-registry userassist`'s CSV columns.
const USERASSIST_COLUMNS: [&str; 20] = [
    "Host",
    "Username",
    "UserSid",
    "SourcePath",
    "Guid",
    "Kind",
    "NameRot13",
    "Name",
    "Path",
    "RunCount",
    "FocusCount",
    "FocusTimeMs",
    "LastRunUTC",
    "Format",
    "SessionId",
    "KeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

fn rot13(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'a'..='z' => char::from((c as u8 - b'a' + 13) % 26 + b'a'),
            'A'..='Z' => char::from((c as u8 - b'A' + 13) % 26 + b'A'),
            _ => c,
        })
        .collect()
}

/// UserAssist version 5 data: run count, focus count and time, last run.
fn ua5(count: u32, focus: u32, focus_ms: u32, last_run: u64) -> Vec<u8> {
    let mut b = vec![0u8; 72];
    b[4..8].copy_from_slice(&count.to_le_bytes());
    b[8..12].copy_from_slice(&focus.to_le_bytes());
    b[12..16].copy_from_slice(&focus_ms.to_le_bytes());
    b[0x3c..0x44].copy_from_slice(&last_run.to_le_bytes());
    b
}

/// One system's SYSTEM and SOFTWARE hives (its name, alice's profile) and alice's NTUSER.DAT with
/// UserAssist: a program in System32, an app, a shortcut, the session entry, a value cut short,
/// and a version 3 key; and bob's NTUSER.DAT with UserAssist turned off.
fn userassist_collection(root: &Path) {
    let ua = r"Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist";
    let programs = format!(r"{ua}\{{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}}");
    let shortcuts = format!(r"{ua}\{{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}}");
    let old = format!(r"{ua}\{{75048700-EF1F-11D0-9888-006097DEACF9}}");
    let count = |key: &str| format!(r"{key}\Count");
    let mut v3 = vec![0u8; 16];
    v3[4..8].copy_from_slice(&9u32.to_le_bytes());
    v3[8..16].copy_from_slice(&(LINKED - DAY).to_le_bytes());
    let alice = reg_tree(
        "ROOT",
        &[
            (&programs, "Version", REG_DWORD, 5u32.to_le_bytes().to_vec()),
            (
                &count(&programs),
                &rot13(r"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\cmd.exe"),
                REG_BINARY,
                ua5(3, 5, 61_000, LINKED),
            ),
            (
                &count(&programs),
                &rot13("Microsoft.Windows.Explorer"),
                REG_BINARY,
                ua5(0, 2, 900, 0),
            ),
            (
                &count(&programs),
                &rot13("UEME_CTLSESSION"),
                REG_BINARY,
                vec![0; 1612],
            ),
            (
                &count(&programs),
                &rot13(r"C:\Users\alice\Downloads\cut.exe"),
                REG_BINARY,
                vec![1, 2, 3],
            ),
            (
                &shortcuts,
                "Version",
                REG_DWORD,
                5u32.to_le_bytes().to_vec(),
            ),
            (
                &count(&shortcuts),
                &rot13(r"{9E3995AB-1F9C-4F13-B827-48B24B6C7174}\TaskBar\File Explorer.lnk"),
                REG_BINARY,
                ua5(4, 0, 4, LINKED + DAY),
            ),
            (&old, "Version", REG_DWORD, 3u32.to_le_bytes().to_vec()),
            (
                &count(&old),
                &rot13(r"UEME_RUNPATH:C:\WINDOWS\system32\notepad.exe"),
                REG_BINARY,
                v3,
            ),
        ],
    );
    let bob = reg_tree(
        "ROOT",
        &[(
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
            "Start_TrackProgs",
            REG_DWORD,
            0u32.to_le_bytes().to_vec(),
        )],
    );
    let software = reg_tree(
        "ROOT",
        &[(
            r"Microsoft\Windows NT\CurrentVersion\ProfileList\S-1-5-21-1-2-3-1001",
            "ProfileImagePath",
            REG_EXPAND_SZ,
            utf16z(r"C:\Users\alice"),
        )],
    );
    let system = reg_tree(
        "ROOT",
        &[
            ("Select", "Current", REG_DWORD, 1u32.to_le_bytes().to_vec()),
            (
                r"ControlSet001\Control\ComputerName\ComputerName",
                "ComputerName",
                REG_SZ,
                utf16z("HOST1"),
            ),
        ],
    );
    let config = root.join("C/Windows/System32/config");
    for dir in [
        &config,
        &root.join("C/Users/alice"),
        &root.join("C/Users/bob"),
    ] {
        std::fs::create_dir_all(dir).unwrap();
    }
    std::fs::write(config.join("SOFTWARE"), software).unwrap();
    std::fs::write(config.join("SYSTEM"), system).unwrap();
    std::fs::write(root.join("C/Users/alice/NTUSER.DAT"), alice).unwrap();
    std::fs::write(root.join("C/Users/bob/NTUSER.DAT"), bob).unwrap();
}

/// `parse-registry userassist` with `args`, as JSON Lines.
fn userassist_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "userassist"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_userassist_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    userassist_collection(dir.path());
    let records = userassist_read(&["-d", dir.path().to_str().unwrap()]);
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} {} | {} | {} {} {} {} v{}",
                r["kind"].as_str().unwrap(),
                r["name"].as_str().unwrap(),
                r["path"].as_str().unwrap_or("-"),
                r["run_count"],
                r["focus_count"],
                r["focus_time_ms"],
                r["last_run"].as_str().unwrap_or("-"),
                r["format"]
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            r"active_desktop UEME_RUNPATH:C:\WINDOWS\system32\notepad.exe | C:\WINDOWS\system32\notepad.exe | 4 null null 2022-02-07T21:03:52Z v3",
            r"executable {1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\cmd.exe | %windir%\system32\cmd.exe | 3 5 61000 2022-02-08T21:03:52Z v5",
            "executable Microsoft.Windows.Explorer | - | 0 2 900 - v5",
            r"executable C:\Users\alice\Downloads\cut.exe | C:\Users\alice\Downloads\cut.exe | null null null - vnull",
            r"shortcut {9E3995AB-1F9C-4F13-B827-48B24B6C7174}\TaskBar\File Explorer.lnk | %APPDATA%\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\File Explorer.lnk | 4 0 4 2022-02-09T21:03:52Z v5",
        ]
    );
    for r in &records {
        assert_eq!(r["host"], "HOST1");
        assert_eq!(r["username"], "alice");
        assert_eq!(r["user_sid"], "S-1-5-21-1-2-3-1001");
        assert_eq!(r["transaction_log_status"], "clean");
        assert_eq!(r["key_last_written"], "2022-02-08T21:03:52Z");
    }
    // the {GUID} keys in their list's order (by name), each Count's values in theirs
    assert_eq!(records[1]["guid"], "{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}");
    assert_eq!(
        records[1]["name_rot13"],
        r"{1NP14R77-02R7-4R5Q-O744-2RO1NR5198O7}\pzq.rkr"
    );
    // the value cut short keeps its bytes
    assert_eq!(records[3]["data_raw"], "010203");
    assert_eq!(
        records[3]["problems"],
        json!(["3 bytes: neither 72 (version 5) nor 16 (version 3)"])
    );
}

#[test]
fn registry_userassist_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    userassist_collection(dir.path());
    let out = dir.path().join("userassist.csv");
    let r = run(&[
        "parse-registry",
        "userassist",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "NTUSER.DAT hives parsed: 2",
        "Hosts: HOST1",
        "Users: alice, bob",
        "Hives without UserAssist: 1",
        "Hives where UserAssist is turned off: 1",
        r"NTUSER.DAT: Explorer\Advanced Start_TrackProgs is 0",
        "Items: 5 (active_desktop 1, executable 3, shortcut 1)",
        r"2022-02-09T21:03:52Z  alice  %APPDATA%\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\File Explorer.lnk (runs 4)",
        r"2022-02-08T21:03:52Z  alice  %windir%\system32\cmd.exe (runs 3)",
        "Items or keys that could not be read: 1",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    // the most recent first
    let shortcut = r.stdout.find("File Explorer.lnk (runs 4)").unwrap();
    let cmd = r.stdout.find(r"cmd.exe (runs 3)").unwrap();
    assert!(shortcut < cmd);
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let header: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, USERASSIST_COLUMNS);
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(rows.len(), 5);
    assert_eq!(&rows[1][8], r"%windir%\system32\cmd.exe");
    assert_eq!(&rows[1][9], "3");
    assert_eq!(&rows[1][12], "2022-02-08T21:03:52Z");
}

#[test]
fn registry_userassist_of_public_hives() {
    let dir = tempfile::tempdir().unwrap();
    // regipy's Windows 7 NTUSER.DAT: the items Plaso's userassist reads
    unpack_registry("regipy", &["NTUSER.DAT"], dir.path());
    let records = userassist_read(&["-f", dir.path().join("NTUSER.DAT").to_str().unwrap()]);
    assert_eq!(records.len(), 29);
    let welcome = records
        .iter()
        .find(|r| {
            r["name"]
                .as_str()
                .unwrap()
                .ends_with(r"\Welcome Center.lnk")
        })
        .unwrap();
    assert_eq!(welcome["kind"], "shortcut");
    assert_eq!(
        welcome["path"],
        r"%ALLUSERSPROFILE%\Microsoft\Windows\Start Menu\Programs\Accessories\Welcome Center.lnk"
    );
    assert_eq!(
        (
            &welcome["run_count"],
            &welcome["focus_count"],
            &welcome["focus_time_ms"]
        ),
        (&json!(14), &json!(0), &json!(14))
    );
    assert_eq!(welcome["last_run"], "2012-04-03T22:06:58.124282300Z");

    // the DFIR Artifact Museum's dirty Windows 11 NTUSER.DAT, with its logs
    let w11 = dir.path().join("w11");
    unpack_registry(
        "rathbun_w11",
        &["NTUSER.DAT", "ntuser.dat.LOG1", "ntuser.dat.LOG2"],
        &w11,
    );
    let records = userassist_read(&["-f", w11.join("NTUSER.DAT").to_str().unwrap()]);
    assert_eq!(records.len(), 22);
    let notepad = records
        .iter()
        .find(|r| r["name"] == "Microsoft.WindowsNotepad_8wekyb3d8bbwe!App")
        .unwrap();
    assert_eq!(
        (
            &notepad["run_count"],
            &notepad["focus_count"],
            &notepad["focus_time_ms"]
        ),
        (&json!(16), &json!(9), &json!(83515))
    );
    assert_eq!(notepad["last_run"], "2022-02-06T20:15:41.701Z");
    let cmd = records
        .iter()
        .find(|r| r["path"] == r"%windir%\system32\cmd.exe")
        .unwrap();
    assert_eq!(
        (&cmd["run_count"], &cmd["focus_time_ms"]),
        (&json!(0), &json!(156))
    );
    assert!(records.iter().all(|r| r["problems"] == json!([])));
}

#[test]
fn registry_userassist_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&[
        "parse-registry",
        "userassist",
        "-f",
        sam.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(
        said.contains("a SAM hive, not an NTUSER.DAT hive"),
        "{said}"
    );
    let r = run(&[
        "parse-registry",
        "userassist",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("No NTUSER.DAT hives were found"), "{said}");
    // the output is never one of the files read
    let original = std::fs::read(&sam).unwrap();
    let r = run(&[
        "parse-registry",
        "userassist",
        "-f",
        sam.to_str().unwrap(),
        "-o",
        sam.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(&sam).unwrap(), original);
}

#[test]
fn registry_userassist_no_encrypt_and_corrupt_times() {
    let ua = r"Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist";
    let count = format!(r"{ua}\{{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}}\Count");
    let dir = tempfile::tempdir().unwrap();

    // NoEncrypt: a name written in plain text decodes wrongly; every record says so
    let plain = dir.path().join("plain");
    std::fs::create_dir_all(&plain).unwrap();
    std::fs::write(
        plain.join("NTUSER.DAT"),
        reg_tree(
            "ROOT",
            &[
                (
                    &count,
                    r"C:\Tools\payload.exe",
                    REG_BINARY,
                    ua5(1, 1, 10, LINKED),
                ),
                (
                    &format!(r"{ua}\Settings"),
                    "NoEncrypt",
                    REG_DWORD,
                    1u32.to_le_bytes().to_vec(),
                ),
            ],
        ),
    )
    .unwrap();
    let hive = plain.join("NTUSER.DAT");
    let no_encrypt = r"UserAssist\Settings NoEncrypt is 1: a name written while it was set is not ROT13, and decodes wrongly";
    let r = run(&[
        "parse-registry",
        "userassist",
        "-f",
        hive.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let records = jsonl_records(&r.stdout);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["name"], r"P:\Gbbyf\cnlybnq.rkr");
    assert_eq!(records[0]["problems"], json!([no_encrypt]));
    // with the records on stdout, the hive's warning goes to stderr
    assert!(r.stderr.contains("NoEncrypt is 1"), "{}", r.stderr);
    let out = dir.path().join("plain.csv");
    let r = run(&[
        "parse-registry",
        "userassist",
        "-f",
        hive.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(&rows[0][19], no_encrypt);

    // last runs past 9999 are no times: they don't push the real run out of the summary
    let corrupt = dir.path().join("corrupt");
    std::fs::create_dir_all(&corrupt).unwrap();
    let names: Vec<String> = (0..21)
        .map(|i| rot13(&format!(r"C:\Tools\broken{i:02}.exe")))
        .collect();
    let mut values: Vec<(&str, &str, u32, Vec<u8>)> = names
        .iter()
        .map(|n| {
            (
                count.as_str(),
                n.as_str(),
                REG_BINARY,
                ua5(1, 1, 1, u64::MAX),
            )
        })
        .collect();
    let real = rot13(r"C:\Tools\real.exe");
    values.push((&count, &real, REG_BINARY, ua5(2, 1, 10, LINKED)));
    std::fs::write(corrupt.join("NTUSER.DAT"), reg_tree("ROOT", &values)).unwrap();
    let hive = corrupt.join("NTUSER.DAT");
    let records = userassist_read(&["-f", hive.to_str().unwrap()]);
    assert_eq!(records.len(), 22);
    for r in &records[..21] {
        assert!(r["last_run"].is_null());
        assert_eq!(
            r["problems"],
            json!(["last run FILETIME 0xffffffffffffffff is past 9999"])
        );
        assert_eq!(r["data_raw"].as_str().unwrap().len(), 144);
    }
    assert_eq!(records[21]["last_run"], "2022-02-08T21:03:52Z");
    let out = dir.path().join("corrupt.csv");
    let r = run(&[
        "parse-registry",
        "userassist",
        "-f",
        hive.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let start = r.stdout.find("Run last").unwrap();
    let end = r
        .stdout
        .find("Items or keys that could not be read")
        .unwrap();
    let ran: Vec<&str> = r.stdout[start..end].lines().skip(1).collect();
    assert_eq!(
        ran,
        [r"  2022-02-08T21:03:52Z  ?  C:\Tools\real.exe (runs 2)"],
        "{}",
        r.stdout
    );
    assert!(
        r.stdout
            .contains("Items or keys that could not be read: 21"),
        "{}",
        r.stdout
    );
}
