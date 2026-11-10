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
    copy_ual(&windows.join("System32/LogFiles/Sum"));
    dir
}

/// The UAL databases of test_files/ual/server2019 copied to `sum` (a `Sum` directory).
fn copy_ual(sum: &Path) {
    std::fs::create_dir_all(sum).unwrap();
    for entry in std::fs::read_dir(repo(UAL_SERVER)).unwrap().flatten() {
        std::fs::copy(entry.path(), sum.join(entry.file_name())).unwrap();
    }
}

/// Fail if a file under `output` names one of `directories` or Juntei's executable by its absolute
/// path, or `stdout` names any absolute path: auto-parse shows paths relative to the evidence and
/// output directories.
fn assert_no_absolute_paths(output: &Path, stdout: &str, directories: &[&Path]) {
    let mut absolute = vec![PathBuf::from(env!("CARGO_BIN_EXE_juntei"))];
    for directory in directories {
        absolute.push(directory.to_path_buf());
        absolute.push(std::fs::canonicalize(directory).unwrap());
    }
    // As written, and as JSON writes them (with `\\` on Windows)
    let needles: Vec<String> = absolute
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .flat_map(|path| {
            let json = serde_json::to_string(&path).unwrap();
            [path, json.trim_matches('"').to_string()]
        })
        .collect();
    // The terminal shows a path given as absolute from the current directory (`../results`).
    for token in stdout.split_whitespace() {
        assert!(
            token.len() == 1 || !Path::new(token).is_absolute(),
            "stdout names {token}"
        );
    }
    let mut pending = vec![output.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let text = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
            for needle in &needles {
                assert!(
                    !text.contains(needle.as_str()),
                    "{} names {needle}",
                    path.display()
                );
            }
        }
    }
}

/// Every regular file under `dir`.
fn walk_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files
}

/// Every JSON document in a stream of newline-separated (pretty) documents.
fn json_documents(text: &str) -> Vec<Value> {
    serde_json::Deserializer::from_str(text)
        .into_iter::<Value>()
        .map(|v| v.unwrap())
        .collect()
}

#[test]
fn summaries_have_no_blank_lines_from_their_heading_to_the_elapsed_time() {
    let output = tempfile::tempdir().unwrap();
    let saved = output.path().join("saved.csv");
    let runs: [&[&str]; 6] = [
        &["parse-ntfs", "boot", "-f", "test_files/boot/Boot"],
        &["parse-ntfs", "mft", "-f", "test_files/mft/MFT"],
        &["parse-file", "lnk", "-d", "test_files/lnk"],
        &["parse-file", "prefetch", "-d", "test_files/prefetch"],
        &["parse-registry", "amcache", "-f", WIN10],
        &[
            "parse-registry",
            "amcache",
            "-f",
            WIN10,
            "-o",
            saved.to_str().unwrap(),
            "-t",
            "jsonl",
        ],
    ];
    for args in runs {
        let r = run(&[args, &["-K", "-q"]].concat());
        assert_eq!(r.code, 0, "{args:?}\n{}\n{}", r.stdout, r.stderr);
        let lines: Vec<&str> = r.stdout.lines().collect();
        let heading = lines
            .iter()
            .position(|line| *line == "Results Summary:")
            .unwrap();
        assert!(!lines[heading + 1].is_empty(), "{args:?}\n{}", r.stdout);
        let elapsed = lines
            .iter()
            .position(|line| line.starts_with("Elapsed time: "))
            .unwrap();
        assert!(!lines[elapsed - 1].is_empty(), "{args:?}\n{}", r.stdout);
        // Where the records were saved, or how to save them, follows the summary directly.
        if let Some(save) = lines
            .iter()
            .position(|line| line.starts_with("Results saved: ") || line.starts_with("To save "))
        {
            assert!(!lines[save - 1].is_empty(), "{args:?}\n{}", r.stdout);
        }
    }
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
    // The release name follows the version, and the usage follows them.
    let version = format!("Version: {} (Dev Build)", env!("CARGO_PKG_VERSION"));
    assert!(
        r.stdout
            .contains(&format!("{version}\n\nUsage: juntei <COMMAND>")),
        "{}",
        r.stdout
    );
    let r = run(&["parse-registry", "amcache", "-h", "-K"]);
    assert_eq!(r.code, 0);
    assert!(
        r.stdout
            .contains("juntei parse-registry amcache <INPUT> [OPTIONS]"),
        "{}",
        r.stdout
    );
    assert!(r.stdout.contains(&version), "{}", r.stdout);
    let r = run(&["--version"]);
    assert_eq!(
        r.stdout,
        format!("juntei {} (Dev Build)\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn banner_is_logo_blank_version_and_start_time_then_blank() {
    // The help screen is always in color: drop the escape sequences.
    let plain = |text: &str| {
        let mut out = String::new();
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            if c == '\u{1b}' {
                chars.by_ref().find(|c| c.is_ascii_alphabetic());
            } else {
                out.push(c);
            }
        }
        out
    };
    let version = format!("Version: {} (Dev Build)", env!("CARGO_PKG_VERSION"));
    // The help screen: the logo, a blank line, the version, a blank line and the usage.
    let help = plain(&run(&[]).stdout);
    assert!(
        help.contains(&format!("   by Yamato Security\n\n{version}\n\nUsage: ")),
        "{help}"
    );
    // A command: the logo, a blank line, the version and the start time, then a blank line
    // before the command's own output.
    for (args, next) in [
        (
            vec!["parse-registry", "amcache", "-f", WIN10, "-K"],
            "Amcache hives: ",
        ),
        (
            vec!["parse-file", "prefetch", "-f", PREFETCH_CMD10, "-K"],
            "Prefetch files: ",
        ),
    ] {
        let r = run(&args);
        assert_eq!(r.code, 0, "{}", r.stderr);
        let banner = format!("   by Yamato Security\n\n{version}\nStart time: ");
        let start = r
            .stdout
            .find(&banner)
            .unwrap_or_else(|| panic!("{}", r.stdout));
        let after: Vec<&str> = r.stdout[start + banner.len()..].lines().collect();
        assert!(after[1].is_empty(), "{args:?}\n{}", r.stdout);
        assert!(after[2].starts_with(next), "{args:?}\n{}", r.stdout);
    }
}

#[test]
fn start_and_end_times_are_local_and_the_elapsed_time_is_in_whole_seconds() {
    let r = run(&["parse-registry", "amcache", "-f", WIN10, "-K", "-q"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let lines: Vec<&str> = r.stdout.lines().collect();
    let time = |label: &str| {
        let line = lines
            .iter()
            .position(|line| line.starts_with(label))
            .unwrap_or_else(|| panic!("{label}\n{}", r.stdout));
        let value = &lines[line][label.len()..];
        let parsed = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
            .unwrap_or_else(|e| panic!("{label}{value}: {e}"));
        (line, parsed)
    };
    let (_, start) = time("Start time: ");
    let (end_line, end) = time("End time: ");
    assert!(start <= end, "{}", r.stdout);
    // The elapsed time follows, in whole seconds.
    let elapsed = lines[end_line + 1]
        .strip_prefix("Elapsed time: ")
        .unwrap_or_else(|| panic!("{}", r.stdout));
    assert!(
        chrono::NaiveTime::parse_from_str(elapsed, "%H:%M:%S").is_ok() && elapsed.len() == 8,
        "{}",
        r.stdout
    );
    // Local time, as the computer's clock shows it.
    let local = chrono::Local::now().naive_local();
    assert!((local - end).num_seconds().abs() < 120, "{}", r.stdout);
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
            &["mft", "usn-journal", "logfile", "i30", "boot", "secure-sds"][..],
        ),
        (
            "parse-registry",
            "Parse registry hives",
            &[
                "amcache",
                "shimcache",
                "read",
                "services",
                "shell-bags",
                "recent-docs",
                "taskcache",
                "sam",
                "persistence",
                "usb",
                "user-assist",
                "networks",
                "lsa-secrets",
                "bcd",
                "app-compat",
                "auditpol",
                "office",
                "security-settings",
                "feature-usage",
                "app-history",
                "mru",
                "system-info",
                "remote-access",
                "open-save-mru",
                "bam",
                "installed-programs",
                "muicache",
            ][..],
        ),
        ("parse-ese", "Parse ESE databases", &["srum", "ual"][..]),
        (
            "parse-file",
            "Parse file artifacts",
            &[
                "prefetch",
                "lnk",
                "jump-lists",
                "defender-quarantine",
                "outlook",
            ][..],
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
        r.stdout.contains("Parsing in progress.\n\n"),
        "{}",
        r.stdout
    );
    // The size of every file of the computers found, after the number of processes
    let total: u64 = walk_files(&evidence.path().join("HOST1_F.TEST"))
        .iter()
        .map(|path| path.metadata().unwrap().len())
        .sum();
    let processes = r
        .stdout
        .lines()
        .position(|line| line.starts_with("Parallel processes: "))
        .unwrap();
    assert_eq!(
        r.stdout.lines().nth(processes + 1).unwrap(),
        format!("Total filesize: {}", bytesize::ByteSize::b(total).display()),
        "{}",
        r.stdout
    );
    assert!(!r.stdout.contains("Per-parser messages are saved"));
    // The summary has no blank line after its heading or before the elapsed time.
    assert!(
        r.stdout.contains("Results Summary:\nSucceeded: "),
        "{}",
        r.stdout
    );
    let lines: Vec<&str> = r.stdout.lines().collect();
    let elapsed = lines
        .iter()
        .position(|line| line.starts_with("Elapsed time: "))
        .unwrap();
    assert!(!lines[elapsed - 1].is_empty(), "{}", r.stdout);
    assert!(
        csv.join("HOST1/Persistence/PersistenceHunter.csv")
            .is_file()
    );
    assert!(
        csv.join("HOST1/Persistence/PersistenceHunter-logs.txt")
            .is_file()
    );
    assert!(!csv.join("HOST1/FileExistence/Registry-Amcache").exists());
    assert!(
        csv.join("HOST1/FileExistence/Registry-Amcache-Summary.csv")
            .is_file()
    );
    assert!(
        csv.join("HOST1/FileExistence/Registry-Amcache-Files.csv")
            .is_file()
    );
    assert!(
        std::fs::read_dir(csv.join("HOST1/FileExistence"))
            .unwrap()
            .flatten()
            .all(|entry| entry.path().is_file())
    );
    assert!(!csv.join("HOST1/ProcessExecution/SRUM").exists());
    assert!(
        csv.join("HOST1/SystemInformation/SRUM-IdentityMap.csv")
            .is_file()
    );
    assert!(
        csv.join("HOST1/SystemInformation/SRUM-VfuProvider.csv")
            .is_file()
    );
    assert!(
        csv.join("HOST1/NetworkActivity/SRUM-NetworkConnections.csv")
            .is_file()
    );
    assert!(
        csv.join("HOST1/NetworkActivity/SRUM-NetworkUsages.csv")
            .is_file()
    );
    assert!(
        csv.join("HOST1/ProcessExecution/SRUM-AppResourceUseInfo.csv")
            .is_file()
    );
    assert!(!csv.join("HOST1/SystemInformation/SRUM").exists());
    // UAL: one file per table, in LateralMovement, no directory left
    for table in [
        "Clients",
        "RoleAccess",
        "DNS",
        "VirtualMachines",
        "SystemIdentity",
        "ChainedDatabases",
        "Roles",
    ] {
        assert!(
            csv.join(format!("HOST1/LateralMovement/UAL-{table}.csv"))
                .is_file(),
            "{table}"
        );
    }
    assert!(!csv.join("HOST1/LateralMovement/UAL").exists());
    assert!(csv.join("HOST1/LateralMovement/UAL-logs.txt").is_file());
    assert!(csv.join("HOST1/FileSystem/MFT.csv").is_file());
    assert!(csv.join("HOST1/Registry/Registry-Dump.csv").is_file());
    assert!(csv.join("HOST1/Registry/Registry-Dump-logs.txt").is_file());
    assert!(!csv.join("HOST1/Registry/Registry-Read.csv").exists());
    assert!(!csv.join("HOST1/Registry/Registry-Read-logs.txt").exists());
    assert!(csv.join("HOST1/FileSystem/UsnJournal.csv").is_file());
    assert!(!csv.join("HOST1/FileSystem/mft").exists());
    assert!(!csv.join("HOST1/FileSystem/usn-journal").exists());
    assert!(csv.join("HOST1/ProcessExecution/Prefetch.csv").is_file());
    assert!(
        csv.join("HOST1/ProcessExecution/PrefetchTimeline.csv")
            .is_file()
    );
    assert!(
        csv.join("HOST1/ProcessExecution/Prefetch-logs.txt")
            .is_file()
    );
    assert!(csv.join("HOST1/FileSystem/MFT-logs.txt").is_file());
    assert!(!csv.join("HOST1/logs").exists());
    for category in [
        "Accounts",
        "Applications",
        "BrowserActivity",
        "Communication",
        "ExternalDevices",
        "FileAndFolderOpening",
        "FileExistence",
        "FileSystem",
        "NetworkActivity",
        "LateralMovement",
        "Persistence",
        "ProcessExecution",
        "Registry",
        "SecurityProducts",
        "SystemInformation",
    ] {
        assert!(csv.join("HOST1").join(category).is_dir());
        assert!(!csv.join("HOST1").join(category).join("logs").exists());
    }
    assert!(!csv.join("HOST1/_logs").exists());
    let manifest: Value =
        serde_json::from_reader(std::fs::File::open(csv.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["Hosts"][0]["Comp"], "HOST1");
    assert_eq!(manifest["Hosts"][0]["Hostname"], "HOST1");
    assert_eq!(manifest["Hosts"][0]["HostnameSource"], "client_info.json");
    // The parsers run in the evidence directory and are given paths relative to it, so the host
    // list, the manifests and the results show where files are in the evidence.
    let host_root = Path::new(".").join("HOST1_F.TEST");
    assert!(
        r.stdout.contains(&format!(": {}\n", host_root.display())),
        "{}",
        r.stdout
    );
    assert_eq!(
        manifest["Hosts"][0]["EvidenceRoot"],
        host_root.to_string_lossy().as_ref()
    );
    assert_eq!(
        manifest["Hosts"][0]["WindowsDirectories"][0],
        host_root
            .join("uploads")
            .join("auto")
            .join("C%3A")
            .join("Windows")
            .to_string_lossy()
            .as_ref()
    );
    let mft_task = manifest["Tasks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|task| task["Artifact"] == "mft")
        .unwrap();
    let mft = host_root
        .join("uploads")
        .join("ntfs")
        .join("C")
        .join("$MFT");
    assert!(
        mft_task["Command"]
            .as_array()
            .unwrap()
            .windows(2)
            .any(|pair| pair[0] == "-f" && pair[1] == mft.to_string_lossy().as_ref()),
        "{mft_task}"
    );
    // Names in the slack of FILE records are recovered only with --mft-slack.
    assert!(
        !mft_task["Command"]
            .as_array()
            .unwrap()
            .iter()
            .any(|arg| arg == "--recover-slack"),
        "{mft_task}"
    );
    let (mft_header, _) = csv_rows(&csv.join("HOST1/FileSystem/MFT.csv"));
    assert!(!mft_header.iter().any(|column| column == "FromSlack"));
    // Every parser that reads ANSI text gets the code page, here the default.
    for artifact in ["lnk", "jump-lists", "opensavemru", "shellbags"] {
        let task = manifest["Tasks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|task| task["Artifact"] == artifact)
            .unwrap();
        assert!(
            task["Command"]
                .as_array()
                .unwrap()
                .windows(2)
                .any(|pair| pair[0] == "--code-page" && pair[1] == "1252"),
            "{task}"
        );
    }
    let (_, prefetch) = csv_rows(&csv.join("HOST1/ProcessExecution/Prefetch.csv"));
    assert!(!prefetch.is_empty());
    assert!(
        prefetch
            .iter()
            .all(|row| Path::new(&row["SourceFile"]).starts_with(&host_root)),
        "{prefetch:?}"
    );
    let (_, hunter) = csv_rows(&csv.join("HOST1/Persistence/PersistenceHunter.csv"));
    assert!(
        hunter
            .iter()
            .all(|row| Path::new(&row["SourceFile"]).starts_with(&host_root)),
        "{hunter:?}"
    );
    // No output, log, manifest or terminal line says where the evidence, the results or Juntei
    // are on this system.
    assert_eq!(
        manifest["EvidenceDirectory"],
        evidence
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .as_ref()
    );
    assert_eq!(manifest["OutputDirectory"], "csv");
    assert_eq!(
        manifest["Tasks"][0]["Command"][0],
        Path::new(env!("CARGO_BIN_EXE_juntei"))
            .file_name()
            .unwrap()
            .to_string_lossy()
            .as_ref()
    );
    assert_no_absolute_paths(&csv, &r.stdout, &[evidence.path(), output_root.path()]);
    assert!(
        manifest["Tasks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|task| task["Status"] != "failed")
    );
    assert!(manifest["Tasks"].as_array().unwrap().iter().all(|task| {
        task["Log"].as_str().is_some_and(|path| {
            let category = task["Category"].as_str().unwrap();
            Path::new(path).parent() == Some(Path::new(".").join("HOST1").join(category).as_path())
                && path.ends_with("-logs.txt")
                && csv.join(path).is_file()
        })
    }));
    assert_eq!(manifest["Tasks"][0]["Artifact"], "mft");
    assert_eq!(manifest["Tasks"][1]["Artifact"], "usn-journal");
    for task in manifest["Tasks"].as_array().unwrap() {
        let artifact = task["Artifact"].as_str().unwrap();
        if artifact == "registry" {
            assert_eq!(
                csv.join(task["Output"].as_str().unwrap()),
                csv.join("HOST1/Registry/Registry-Dump.csv")
            );
            assert_eq!(
                csv.join(task["Log"].as_str().unwrap()),
                csv.join("HOST1/Registry/Registry-Dump-logs.txt")
            );
        }
        let category = match artifact {
            "lsa-secrets" | "sam" => "Accounts",
            "app-history" => "Applications",
            "installed-programs" => "Applications",
            "browser" | "webcache" => "BrowserActivity",
            "outlook" => "Communication",
            "usb" => "ExternalDevices",
            "jump-lists" | "lnk" | "mru" | "office" | "opensavemru" | "recentdocs"
            | "shellbags" => "FileAndFolderOpening",
            "amcache" | "shimcache" => "FileExistence",
            "boot" | "i30" | "logfile" | "mft" | "secure-sds" | "usn-journal" => "FileSystem",
            "networks" => "NetworkActivity",
            "remote-access" | "ual" => "LateralMovement",
            "bcd" | "persistence" | "persistence-hunter" | "services" | "taskcache" => {
                "Persistence"
            }
            "appcompat" | "bam" | "featureusage" | "muicache" | "prefetch"
            | "prefetch-timeline" | "userassist" => "ProcessExecution",
            "registry" => "Registry",
            "defender-quarantine" | "security-settings" => "SecurityProducts",
            "system-info" | "srum" => "SystemInformation",
            "auditpol" => "SystemInformation",
            _ => panic!("unexpected artifact: {artifact}"),
        };
        assert_eq!(task["Category"], category);
        if let Some(output) = task["Output"].as_str() {
            let host = Path::new(".").join("HOST1");
            if artifact == "srum" {
                assert_eq!(Path::new(output), host);
            } else {
                assert!(Path::new(output).starts_with(host.join(category)));
            }
        }
        let (source, command) = match artifact {
            "persistence-hunter" => {
                assert_eq!(task["Command"][1], "persistence-hunter", "{task}");
                assert_eq!(task["Command"][2], "-d", "{task}");
                let args = task["Command"].as_array().unwrap();
                assert!(
                    args.windows(2)
                        .any(|pair| pair[0] == "--min-severity" && pair[1] == "info"),
                    "{task}"
                );
                assert!(
                    args.windows(2)
                        .any(|pair| pair[0] == "--threads" && pair[1] == "1"),
                    "{task}"
                );
                continue;
            }
            "mft" | "usn-journal" | "logfile" | "i30" | "boot" | "secure-sds" => {
                ("parse-ntfs", artifact)
            }
            "srum" | "ual" | "webcache" => ("parse-ese", artifact),
            "browser" => ("parse-sqlite", artifact),
            "prefetch-timeline" => ("parse-file", "prefetch"),
            "prefetch" | "lnk" | "jump-lists" | "defender-quarantine" | "outlook" => {
                ("parse-file", artifact)
            }
            "registry" => ("parse-registry", "read"),
            "appcompat" => ("parse-registry", "app-compat"),
            "featureusage" => ("parse-registry", "feature-usage"),
            "opensavemru" => ("parse-registry", "open-save-mru"),
            "recentdocs" => ("parse-registry", "recent-docs"),
            "userassist" => ("parse-registry", "user-assist"),
            "shellbags" => ("parse-registry", "shell-bags"),
            _ => ("parse-registry", artifact),
        };
        assert_eq!(task["Command"][1], source, "{task}");
        assert_eq!(task["Command"][2], command, "{task}");
    }
    let (_, csv_tasks) = csv_rows(&csv.join("manifest.csv"));
    assert_eq!(csv_tasks.len(), manifest["Tasks"].as_array().unwrap().len());
    for (row, task) in csv_tasks.iter().zip(manifest["Tasks"].as_array().unwrap()) {
        assert_eq!(row["Comp"], task["Comp"].as_str().unwrap());
        assert_eq!(row["Hostname"], "HOST1");
        assert_eq!(row["Category"], task["Category"].as_str().unwrap());
        assert_eq!(row["Status"], task["Status"].as_str().unwrap());
        assert_eq!(row["Log"], task["Log"].as_str().unwrap());
        assert_eq!(
            serde_json::from_str::<Value>(&row["Command"]).unwrap(),
            task["Command"]
        );
    }
    assert!(!r.stdout.contains("Manifest: "), "{}", r.stdout);
    let summary = r
        .stdout
        .lines()
        .find(|line| line.starts_with("Results saved: "))
        .unwrap();
    // An output directory given as absolute is shown from the current directory.
    assert!(
        summary.starts_with("Results saved: ") && summary.contains("csv/ ( "),
        "{summary}"
    );
    assert!(
        summary.contains(" files / ") && summary.ends_with(" MiB )"),
        "{summary}"
    );

    // A previously generated target must not survive when the corresponding artifact is absent.
    let stale = csv.join("HOST1/FileSystem/I30.csv");
    std::fs::write(&stale, b"stale result\n").unwrap();
    let notes = csv.join("HOST1/FileSystem/notes.txt");
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
    assert_eq!(direct_manifest["EvidenceDirectory"], "Windows");
    assert_eq!(direct_manifest["Hosts"][0]["EvidenceRoot"], ".");
    assert_eq!(
        direct_manifest["Hosts"][0]["WindowsDirectories"],
        json!(["."])
    );
    assert!(
        direct
            .join("Windows/ProcessExecution/Prefetch.csv")
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
    assert!(values.iter().all(|value| value["Comp"] == "HOST1"));
    assert!(values.iter().all(|value| value.get("Artifact").is_some()));
    assert!(values.iter().any(|value| value["Artifact"] == "amcache"));
    assert!(values.iter().any(|value| value["Artifact"] == "prefetch"));
    assert!(
        jsonl
            .join("HOST1/ProcessExecution/Prefetch-logs.txt")
            .is_file()
    );
    assert!(
        jsonl
            .join("HOST1/FileExistence/Registry-Amcache-logs.txt")
            .is_file()
    );
    assert!(
        jsonl
            .join("HOST1/ProcessExecution/Prefetch.jsonl")
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
        let registry_dump = host.join(format!("Registry/Registry-Dump.{format}"));
        assert!(registry_dump.is_file());
        assert!(host.join("Registry/Registry-Dump-logs.txt").is_file());
        assert!(
            !host
                .join(format!("Registry/Registry-Read.{format}"))
                .exists()
        );
        assert!(!host.join("Registry/Registry-Read-logs.txt").exists());
        assert!(
            host.join(format!("FileExistence/Registry-Amcache.{format}"))
                .is_file()
        );
        assert!(host.join(format!("FileSystem/MFT.{format}")).is_file());
        assert!(
            host.join(format!("FileSystem/UsnJournal.{format}"))
                .is_file()
        );
        assert!(!host.join("ProcessExecution/SRUM").exists());
        assert!(!host.join("SystemInformation/SRUM").exists());
        assert!(
            host.join(format!("NetworkActivity/SRUM-NetworkConnections.{format}"))
                .is_file()
        );
        assert!(
            host.join(format!("ProcessExecution/SRUM-AppResourceUseInfo.{format}"))
                .is_file()
        );
        let prefetch = host.join(format!("ProcessExecution/Prefetch.{format}"));
        let records = json_documents(&std::fs::read_to_string(prefetch).unwrap());
        assert!(!records.is_empty());
        assert!(
            records
                .iter()
                .all(|record| { record["Comp"] == "HOST1" && record["Artifact"] == "prefetch" })
        );
        let manifest: Value =
            serde_json::from_reader(std::fs::File::open(output.join("manifest.json")).unwrap())
                .unwrap();
        for task in manifest["Tasks"].as_array().unwrap() {
            if task["Artifact"] == "registry" {
                assert_eq!(output.join(task["Output"].as_str().unwrap()), registry_dump);
                assert_eq!(
                    output.join(task["Log"].as_str().unwrap()),
                    host.join("Registry/Registry-Dump-logs.txt")
                );
            }
            let category = task["Category"].as_str().unwrap();
            // The manifest's paths are relative to the output directory.
            let log = output.join(task["Log"].as_str().unwrap());
            assert_eq!(log.parent(), Some(host.join(category).as_path()));
            assert!(
                log.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .ends_with("-logs.txt")
            );
            assert!(!host.join(category).join("logs").exists());
            assert!(log.is_file());
            if let Some(path) = task["Output"].as_str() {
                let path = output.join(path);
                if task["Artifact"] == "srum" {
                    assert_eq!(path, host);
                    let files = task["OutputFiles"].as_array().unwrap();
                    assert!(!files.is_empty());
                    for file in files {
                        assert!(output.join(file.as_str().unwrap()).is_file());
                    }
                } else {
                    assert!(path.starts_with(host.join(category)));
                }
                assert!(path.exists());
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
    let mut previous: Option<Vec<Value>> = None;
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
        let mut records: Vec<Value> = std::fs::read_to_string(&timeline)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert!(!records.is_empty());
        assert!(records.iter().all(|record| record["Comp"] == "HOST1"));
        assert!(
            records
                .iter()
                .any(|record| record["Artifact"] == "amcache" && record["RecordType"] == "file")
        );
        assert!(records.iter().any(|record| record["Artifact"] == "srum"));
        assert_eq!(
            records
                .iter()
                .filter(|record| record["Artifact"] == "ual")
                .count(),
            17
        );
        fn canonical(value: &mut Value) {
            match value {
                Value::Object(fields) => {
                    fields.sort_keys();
                    for child in fields.values_mut() {
                        canonical(child);
                    }
                }
                Value::Array(values) => values.iter_mut().for_each(canonical),
                _ => (),
            }
        }
        records.iter_mut().for_each(canonical);
        records.sort_by_cached_key(|record| serde_json::to_string(record).unwrap());
        if let Some(expected) = &previous {
            assert_eq!(
                records.len(),
                expected.len(),
                "{format} SIEM record count changed"
            );
            for (index, (actual, expected)) in records.iter().zip(expected).enumerate() {
                assert_eq!(actual, expected, "{format} SIEM record {index} changed");
            }
        }
        previous = Some(records);
        let manifest: Value =
            serde_json::from_reader(std::fs::File::open(output.join("manifest.json")).unwrap())
                .unwrap();
        let canonical_timeline = std::fs::canonicalize(&timeline).unwrap();
        // From the output directory, like the manifest's other paths
        assert_eq!(
            manifest["JSON-Timeline"],
            Path::new("..")
                .join(format!("SIEM export {format}.jsonl"))
                .to_string_lossy()
                .as_ref()
        );
        assert!(manifest["JSON-TimelineError"].is_null());
        assert!(
            output
                .join("HOST1/ProcessExecution/Prefetch-logs.txt")
                .is_file()
        );
        let extension = format!("HOST1/ProcessExecution/Prefetch.{format}");
        assert!(output.join(extension).is_file());
        let tasks = manifest["Tasks"].as_array().unwrap();
        if format == "csv" {
            assert_eq!(
                tasks
                    .iter()
                    .filter(|task| task["Purpose"] == "json_timeline")
                    .count(),
                44
            );
            assert!(
                output
                    .join("HOST1/ProcessExecution/Prefetch-json-timeline-logs.txt")
                    .is_file()
            );
            assert!(
                output
                    .join("HOST1/FileExistence/Registry-Amcache-Files.csv")
                    .is_file()
            );
        } else {
            assert!(tasks.iter().all(|task| task["Purpose"] == "artifact"));
        }
        for task in tasks.iter().filter(|task| task["Status"] == "succeeded") {
            let path = output.join(task["Output"].as_str().unwrap());
            assert!(path.exists(), "{task}");
            if task["Purpose"] == "json_timeline" {
                assert_eq!(std::fs::canonicalize(path).unwrap(), canonical_timeline);
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
        output.join("HOST1/ProcessExecution/Prefetch.jsonl"),
        output.join("HOST1/ProcessExecution/Prefetch-logs.txt"),
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
        .join("parsed/HOST1/ProcessExecution/Prefetch.jsonl");
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
    assert_eq!(tasks.len(), 88);
    assert!(tasks.iter().any(|task| task["Status"] == "skipped"));
    let failed = tasks
        .iter()
        .find(|task| task["Artifact"] == "prefetch" && task["Purpose"] == "artifact")
        .unwrap();
    assert_eq!(failed["Status"], "failed");
    assert_eq!(failed["Category"], "ProcessExecution");
    assert_eq!(failed["ExitCode"], "1");
    assert!(!failed["Error"].is_empty());
    assert!(output.join(&failed["Log"]).is_file());
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

/// `-o evidence/link/../results` names, through the file system, the directory beside the link's
/// target, which the containment check approves. The parsers must write there, not to
/// `evidence/results`, which removing `link/..` from the text would name.
#[cfg(unix)]
#[test]
fn auto_parse_writes_where_a_symbolic_link_in_the_output_path_resolves() {
    let root = tempfile::tempdir().unwrap();
    let prefetch = root.path().join("evidence/HOST/C/Windows/Prefetch");
    std::fs::create_dir_all(&prefetch).unwrap();
    std::fs::copy(
        repo("test_files/prefetch/CMD.EXE-087B4001.pf"),
        prefetch.join("CMD.EXE-087B4001.pf"),
    )
    .unwrap();
    let outside = root.path().join("outside");
    std::fs::create_dir_all(outside.join("child")).unwrap();
    std::os::unix::fs::symlink(outside.join("child"), root.path().join("evidence/link")).unwrap();
    let sentinel = root
        .path()
        .join("evidence/results/HOST/ProcessExecution/Prefetch.csv");
    std::fs::create_dir_all(sentinel.parent().unwrap()).unwrap();
    std::fs::write(&sentinel, "sentinel\n").unwrap();
    let r = run_at(
        root.path(),
        &[
            "auto-parse",
            "-d",
            "evidence",
            "-o",
            "evidence/link/../results",
            "-T",
            "2",
            "-K",
            "-q",
        ],
    );
    assert_eq!(r.code, 0, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    assert_eq!(std::fs::read_to_string(&sentinel).unwrap(), "sentinel\n");
    let (_, rows) = csv_rows(&outside.join("results/HOST/ProcessExecution/Prefetch.csv"));
    assert_eq!(rows.len(), 1);
    assert_no_absolute_paths(&outside.join("results"), &r.stdout, &[root.path()]);
}

/// A `-C` rerun removes the table files an earlier run recorded, from a manifest of this version
/// (`ManifestVersion` 2, paths from the output directory) or of an earlier one (no version,
/// paths as given to `-o`, which can also start with `./`).
#[test]
fn auto_parse_clobber_reruns_read_current_and_earlier_manifest_paths() {
    let root = tempfile::tempdir().unwrap();
    let programs = root
        .path()
        .join("evidence/HOST/C/Windows/AppCompat/Programs");
    std::fs::create_dir_all(&programs).unwrap();
    std::fs::copy(repo(WIN10), programs.join("Amcache.hve")).unwrap();
    let args = [
        "auto-parse",
        "-d",
        "./evidence",
        "-o",
        "./results",
        "-C",
        "-T",
        "2",
        "-K",
        "-q",
    ];
    let results = root.path().join("results");
    let r = run_at(root.path(), &args);
    assert_eq!(r.code, 0, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    for earlier in [false, true] {
        let manifest_path = results.join("manifest.json");
        let mut manifest: Value =
            serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
        assert_eq!(manifest["ManifestVersion"], 2);
        // As earlier versions recorded the paths: as given to `-o` (`./results/HOST/...`)
        let recorded = |path: &str| -> String {
            if earlier {
                let path = Path::new(path).strip_prefix(".").unwrap();
                Path::new(".")
                    .join("results")
                    .join(path)
                    .to_string_lossy()
                    .into_owned()
            } else {
                path.to_string()
            }
        };
        if earlier {
            manifest.as_object_mut().unwrap().remove("ManifestVersion");
        }
        let stale_name = Path::new(".")
            .join("HOST")
            .join("FileExistence")
            .join("Registry-Amcache-Stale.csv");
        for task in manifest["Tasks"].as_array_mut().unwrap() {
            for field in ["Output", "Log"] {
                if let Some(path) = task[field].as_str() {
                    task[field] = Value::from(recorded(path));
                }
            }
            let files: Vec<Value> = task["OutputFiles"]
                .as_array()
                .unwrap()
                .iter()
                .map(|path| Value::from(recorded(path.as_str().unwrap())))
                .collect();
            task["OutputFiles"] = Value::from(files);
            if task["Artifact"] == "amcache" && task["Purpose"] == "artifact" {
                task["OutputFiles"]
                    .as_array_mut()
                    .unwrap()
                    .push(Value::from(recorded(&stale_name.to_string_lossy())));
            }
        }
        std::fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        let stale = results.join(&stale_name);
        std::fs::write(&stale, "stale\n").unwrap();

        let r = run_at(root.path(), &args);
        assert_eq!(
            r.code, 0,
            "earlier: {earlier}\nstdout:\n{}\nstderr:\n{}",
            r.stdout, r.stderr
        );
        assert!(!stale.exists(), "earlier: {earlier}");
        assert!(
            results
                .join("HOST/FileExistence/Registry-Amcache-Files.csv")
                .is_file()
        );
    }
}

#[test]
fn auto_parse_passes_mft_slack_and_the_code_page_to_their_parsers() {
    let evidence = auto_triage();
    let output_root = tempfile::tempdir().unwrap();
    let csv = output_root.path().join("csv");
    let timeline = output_root.path().join("timeline.jsonl");
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "-o",
        csv.to_str().unwrap(),
        "-j",
        timeline.to_str().unwrap(),
        "--mft-slack",
        "--code-page",
        "932",
        "-T",
        "2",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    let manifest: Value =
        serde_json::from_reader(std::fs::File::open(csv.join("manifest.json")).unwrap()).unwrap();
    let command = |artifact: &str| -> Vec<String> {
        manifest["Tasks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|task| task["Artifact"] == artifact)
            .unwrap()["Command"]
            .as_array()
            .unwrap()
            .iter()
            .map(|arg| arg.as_str().unwrap().to_string())
            .collect()
    };
    let mft = command("mft");
    assert!(mft.iter().any(|arg| arg == "--recover-slack"), "{mft:?}");
    for artifact in ["lnk", "jump-lists", "opensavemru", "shellbags"] {
        let args = command(artifact);
        assert!(
            args.windows(2)
                .any(|pair| pair[0] == "--code-page" && pair[1] == "932"),
            "{artifact}: {args:?}"
        );
    }
    // MFT.csv gets the slack rows that parse-ntfs mft -x writes for the same $MFT.
    let (header, rows) = csv_rows(&csv.join("HOST1/FileSystem/MFT.csv"));
    assert_eq!(
        header[header.len() - 4..],
        [
            "FromSlack",
            "SlackRecord",
            "SlackOffset",
            "CopyOfEntryInUse"
        ]
    );
    let slack = |rows: &[std::collections::HashMap<String, String>]| {
        rows.iter().filter(|row| row["FromSlack"] == "True").count()
    };
    let direct = output_root.path().join("direct.csv");
    let r = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        repo("test_files/mft/MFT").to_str().unwrap(),
        "-x",
        "-a",
        "-s",
        "-o",
        direct.to_str().unwrap(),
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let (_, direct_rows) = csv_rows(&direct);
    assert!(slack(&rows) > 0);
    assert_eq!(slack(&rows), slack(&direct_rows));
    // The JSONL timeline comes from separate JSONL runs of the parsers, with the same options.
    let slack_names = std::fs::read_to_string(&timeline)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .filter(|record| record["Artifact"] == "mft")
        .filter_map(|record| record["SlackFileNames"].as_array().map(Vec::len))
        .sum::<usize>();
    assert!(slack_names > 0);
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
        .join("HOST1/SecurityProducts/DefenderQuarantined")
        .join("tool.exe.quarantined");
    assert!(extracted.is_file(), "{}", extracted.display());
    assert!(
        std::fs::read(extracted)
            .unwrap()
            .starts_with(b"A harmless line of text")
    );
    let security_text = std::fs::read_to_string(
        output.join("HOST1/SecurityProducts/DefenderQuarantined/tool.exe.security_descriptor.txt"),
    )
    .unwrap();
    assert!(security_text.contains("Binary descriptor: tool.exe.security.quarantined"));
    assert!(security_text.contains("Owner: S-1-5-21-1111-2222-3333-1001"));
    assert!(security_text.contains("Decode status: complete"));

    let manifest: Value =
        serde_json::from_reader(std::fs::File::open(output.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["ExtractDefenderQuarantined"], true);
    let defender = manifest["Tasks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|task| task["Artifact"] == "defender-quarantine")
        .unwrap();
    assert!(
        defender["Command"]
            .as_array()
            .unwrap()
            .iter()
            .any(|argument| argument == "--extract")
    );
    // ExtractedFile is from the output directory, as the manifest's paths are.
    let (_, rows) = csv_rows(&output.join("HOST1/SecurityProducts/DefenderQuarantine.csv"));
    let shown = Path::new(".")
        .join("HOST1")
        .join("SecurityProducts")
        .join("DefenderQuarantined")
        .join("tool.exe.quarantined");
    assert!(
        rows.iter()
            .any(|row| row["ExtractedFile"] == shown.to_string_lossy()),
        "{rows:?}"
    );
    assert_no_absolute_paths(&output, &r.stdout, &[evidence.path(), output_root.path()]);

    let timeline_defender = manifest["Tasks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|task| {
            task["Artifact"] == "defender-quarantine" && task["Purpose"] == "json_timeline"
        })
        .unwrap();
    assert!(
        !timeline_defender["Command"]
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
    let legacy_category_extraction = output.join("HOST1/security_products/defender-quarantined");
    std::fs::create_dir_all(&legacy_category_extraction).unwrap();
    std::fs::write(
        legacy_category_extraction.join("old.quarantined"),
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
            .join("HOST1/SecurityProducts/DefenderQuarantined")
            .exists()
    );
    let manifest: Value =
        serde_json::from_reader(std::fs::File::open(output.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["ExtractDefenderQuarantined"], false);
    assert!(!legacy_extraction.exists());
    assert!(!legacy_category_extraction.exists());
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
    std::fs::create_dir_all(amcache.join("older")).unwrap();
    std::fs::copy(repo(LEGACY), amcache.join("older/Amcache.hve")).unwrap();
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
        "csv",
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
    assert_eq!(manifest["Hosts"].as_array().unwrap().len(), 2);
    assert_eq!(manifest["Hosts"][0]["Comp"], "HOST2");
    assert_eq!(manifest["Hosts"][0]["Hostname"], "HOST2");
    assert_eq!(manifest["Hosts"][1]["Comp"], "HOST3");
    assert_eq!(manifest["Hosts"][1]["Hostname"], "HOST3");
    assert!(
        manifest["Hosts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|host| host["HostnameSource"] == "directory name")
    );
    let category = output.join("HOST3/FileExistence");
    assert!(!category.join("Registry-Amcache").exists());
    for source in [1, 2] {
        for table in ["Devices", "Files", "Findings", "Summary"] {
            assert!(
                category
                    .join(format!("Registry-Amcache-{table}-Source{source}.csv"))
                    .is_file()
            );
        }
    }
    let records: Vec<Value> = std::fs::read_to_string(combined)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        records
            .iter()
            .any(|record| record["Comp"] == "HOST2" && record["Artifact"] == "prefetch")
    );
    assert!(
        records
            .iter()
            .any(|record| record["Comp"] == "HOST3" && record["Artifact"] == "amcache")
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
    assert_eq!(doc["AmcacheSchema"], "modern");
    assert_eq!(doc["Summary"]["FileCount"], 123);
    assert_eq!(doc["Files"].as_array().unwrap().len(), 123);
    assert_eq!(doc["Devices"].as_array().unwrap().len(), 189);
    assert_eq!(doc["Findings"].as_array().unwrap().len(), 4);
    let sevenzip = doc["Files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["FullPath"] == "c:\\program files\\7-zip\\7z.exe")
        .expect("7z.exe is inventoried");
    assert_eq!(sevenzip["SHA1"], "1189cebeb8ffed7316f98b895ff949a726f4026f");

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
    assert_eq!(docs[0]["AmcacheSchema"], "modern");
    assert_eq!(docs[1]["AmcacheSchema"], "legacy");
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
    let id = headers.iter().position(|h| h == "EntryID").unwrap();
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
            "EntryNum",
            "SN",
            "InUse",
            "ParentEntryNum",
            "ParentSN",
            "ParentPath"
        ]
    );
    assert_eq!(header.len(), 35);
    assert_eq!(
        &header[19..27],
        [
            "DataModified (SI)",
            "DataModified (FN)",
            "Accessed (SI)",
            "Accessed (FN)",
            "MetadataChanged (SI)",
            "MetadataChanged (FN)",
            "Created (SI)",
            "Created (FN)",
        ]
    );
    assert_eq!(rows.len(), 1014);
    let row = |entry: &str, name: &str| {
        rows.iter()
            .find(|r| r["EntryNum"] == entry && r["File"] == name)
            .unwrap_or_else(|| panic!("{entry} {name}"))
    };
    assert_eq!(row("30", "config")["ParentPath"], r".\WINDOWS\system32");
    // $STANDARD_INFORMATION created before $FILE_NAME created, on a whole second
    let bootvid = row("178", "bootvid.dll");
    for (column, value) in [
        ("Ext", ".dll"),
        ("Size", "12288"),
        ("SI<FN", "True"),
        ("uSecZeros", "True"),
        ("Copied", "False"),
        ("SI-Flags", "Archive"),
        ("Type", "DosWindows"),
        ("Created (SI)", "2006-02-28T12:00:00.0000000Z"),
        ("Created (FN)", "2007-06-30T12:51:00.8147072Z"),
        ("Accessed (SI)", "2007-06-30T22:32:20.5674272Z"),
        ("Accessed (FN)", "2007-06-30T12:51:00.8147072Z"),
        ("DataModified (SI)", "2006-02-28T12:00:00.0000000Z"),
        ("MetadataChanged (SI)", "2007-06-30T13:00:28.7156064Z"),
    ] {
        assert_eq!(bootvid[column], value, "{column}");
    }
    let ads = row("8", "$BadClus:$Bad");
    assert_eq!(ads["IsADS"], "True");
    assert_eq!(ads["Size"], "1261821952");
    assert_eq!(row("8", "$BadClus")["HasADS"], "True");

    // The ys-ntfs-parser command's columns: one row per entry (`ys-mft-parser-csv`, the command's
    // former name, and `mft-dump-csv`, upstream's, still work)
    let mut texts = Vec::new();
    for output_type in ["ys-ntfs-parser-csv", "ys-mft-parser-csv", "mft-dump-csv"] {
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
    for text in &texts[1..] {
        assert_eq!(text, &texts[0]);
    }
    let text = &texts[0];
    assert!(text.starts_with(
        "Signature,EntryID,Sequence,BaseEntryID,BaseEntrySequence,HardLinkCount,Flags,"
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
    assert_eq!(docs[0]["Header"]["RecordNumber"], 0);
    assert_eq!(docs[0]["FullPath"], "$MFT");
    assert!(docs[0]["Attr"].as_array().unwrap().len() > 2);
    let config = docs
        .iter()
        .find(|d| d["Header"]["RecordNumber"] == 30)
        .unwrap();
    assert_eq!(config["FullPath"], r"WINDOWS\system32\config");

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
            serde_json::from_str::<Value>(l).unwrap()["Header"]["RecordNumber"]
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
            .contains("Output type: csv (default), ys-ntfs-parser-csv, json, jsonl"),
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
        rows.iter().find(|r| r["File"] == "$MFT").unwrap().clone()
    };
    assert_eq!(mft(&csv(&[]))["Created (FN)"], "");
    assert_eq!(
        mft(&csv(&["-a"]))["Created (FN)"],
        "2007-06-30T12:50:52.2523952Z"
    );
    // DOS (8.3) names get rows of their own with -s/--short-names
    let rows = csv(&["-s"]);
    assert_eq!(rows.len(), 1041);
    assert_eq!(rows.iter().filter(|r| r["Type"] == "Dos").count(), 27);
    // only records created or modified on or after the cutoff (the reference gives the same 8 rows)
    let rows = csv(&["--cutoff", "2007-07-01"]);
    assert_eq!(rows.len(), 8);
    assert_eq!(rows[0]["DataModified (SI)"], "2009-01-19T04:17:44.7268320Z");
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
    // Distinct FN times catch a header-only reorder or a missed reorder in the slack writer.
    let driver = slack
        .iter()
        .find(|row| row["EntryNum"] == "2050" && row["File"] == "swenum.sys")
        .unwrap();
    for (column, expected) in [
        ("Created (FN)", "2004-08-03T22:58:42.0000000Z"),
        ("Accessed (FN)", "2007-06-30T12:58:47.1896192Z"),
        ("DataModified (FN)", "2006-02-28T12:00:00.0000000Z"),
        ("MetadataChanged (FN)", "2007-06-30T13:00:28.8157504Z"),
        ("Created (SI)", ""),
        ("Accessed (SI)", ""),
        ("DataModified (SI)", ""),
        ("MetadataChanged (SI)", ""),
    ] {
        assert_eq!(driver[column], expected, "{column}");
    }
    assert_eq!(
        slack
            .iter()
            .filter(|r| r["CopyOfEntryInUse"] == "True")
            .count(),
        33
    );
    // An entry of the index root of WINDOWS (entry 28), left in its record's slack: the file it
    // pointed to, the directory, the name and its $FILE_NAME times.
    let found = slack.iter().find(|r| r["File"] == "setuperr.log").unwrap();
    for (column, value) in [
        ("EntryNum", "3522"),
        ("SN", "1"),
        ("InUse", "False"),
        ("ParentEntryNum", "28"),
        ("ParentPath", r".\WINDOWS"),
        ("Created (SI)", ""),
        ("Created (FN)", "2007-06-30T12:59:00.7391024Z"),
        ("SlackRecord", "28"),
        ("SlackOffset", "29208"),
        ("CopyOfEntryInUse", "False"),
    ] {
        assert_eq!(found[column], value, "{column}");
    }
    let normal = rows.iter().find(|r| r["File"] == "$MFT").unwrap();
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
        .flat_map(|e| e["SlackFileNames"].as_array().unwrap())
        .collect();
    assert_eq!(names.len(), 117);
    let windows = entries
        .iter()
        .find(|e| e["Header"]["RecordNumber"] == 28)
        .unwrap();
    assert_eq!(windows["SlackFileNames"][0]["Name"], "setuperr.log");
    assert_eq!(windows["SlackFileNames"][0]["Offset"], 29208);
    assert_eq!(
        names
            .iter()
            .filter(|n| n["CopyOfEntryInUse"] == true)
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
        "ys-ntfs-parser-csv",
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
            .find(|r| r["FromSlack"] == "True" && r["File"] == name)
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
    let ini: Vec<_> = rows.iter().filter(|r| r["File"] == "system.ini").collect();
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
    assert_eq!(entry["SlackFileNames"], serde_json::json!([]));
    assert!(
        entry["SlackProblem"]
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
        .find(|r| r["File"] == "system.ini")
        .unwrap();
    assert!(ini["ResidentDataASCII"].starts_with("; for 16-bit app support\r\n"));
    assert!(ini["ResidentDataHex"].starts_with("3B-20-66-6F-72-20"));
    assert!(ini["ResidentDataBase64"].starts_with("OyBmb3Ig"));
    // Binary content: no text.
    let pif = with_data
        .iter()
        .find(|r| r["File"] == "_default.pif")
        .unwrap();
    assert_eq!(pif["ResidentDataASCII"], "");
    assert!(pif["ResidentDataHex"].starts_with("00-C9"));
    // By extension and size: win.ini (477 bytes) is larger than 300.
    let (_, rows) = csv(&["-i", "-E", ".INI", "-M", "300"]);
    let names: Vec<&str> = rows
        .iter()
        .filter(|r| !r["ResidentDataBase64"].is_empty())
        .map(|r| r["File"].as_str())
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
        "\"SlackFileNames\": [",
        "\"Name\": \"setuperr.log\"",
        "0000  46 49 4C 45",
    ] {
        assert!(r.stdout.contains(line), "{line}: {}", r.stdout);
    }
    let shown = |stdout: &str| {
        stdout
            .lines()
            .filter(|l| {
                !l.starts_with("Start time")
                    && !l.starts_with("End time")
                    && !l.starts_with("Elapsed time")
            })
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
    let names: Vec<&str> = rows.iter().map(|r| r["File"].as_str()).collect();
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
    assert_eq!(rows[1]["Size"], "219");
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
    assert!(rows.iter().all(|r| r["EntryNum"] == "209"), "{rows:?}");
    assert_eq!(rows.len(), 2);
    for row in &rows {
        assert_eq!(
            row["ParseErrors"],
            "extension record 226: $DATA attribute at 0x100 left out: its value (0x1DD bytes at 0x3F0) runs past the end of the attribute",
            "{}",
            row["File"]
        );
        // the base record's own $DATA gives the size
        assert_eq!(row["Size"], "231");
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
        assert!(text.starts_with("EntryNum,SN,InUse,"), "{name}");
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
        .filter(|r| r["EntryNum"] == a.to_string() || r["EntryNum"] == b.to_string())
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
        .find(|d| d["ExecutableName"] == "RUST_OUT.EXE")
        .unwrap();
    assert_eq!(rust_out["Findings"][0]["Code"], "SuspiciousExecutionPath");
    assert_eq!(rust_out["Findings"][0]["Severity"], "medium");
    let consent = docs
        .iter()
        .find(|d| d["ExecutableName"] == "CONSENT.EXE")
        .unwrap();
    assert_eq!(consent["WindowsVersion"], "Windows Vista/7");
    assert!(
        consent["Files"][0]["MFT-Entry"].is_u64(),
        "version 23 has file references"
    );
    assert!(!consent["Volumes"][0]["Dir"].as_array().unwrap().is_empty());

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
    assert_eq!(doc["LastRunTimes"].as_array().unwrap().len(), 4);
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

/// A triage collection that keeps the volume's layout: the journal at `HOST1/C/$Extend/$J`, the volume's `$MFT` at `HOST1/C/$MFT`.
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
    assert_eq!(docs[0]["File"]["Entry"], 900);
    assert_eq!(docs[0]["Reasons"][0], "FILE_CREATE");
    assert_eq!(docs[9]["Version"], "4.0");
    assert_eq!(docs[9]["Extents"][0]["Length"], 4096);
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
    assert_eq!(docs[0]["Offset"], 80_000);
    assert_eq!(
        docs[0]["ParentPath"],
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

fn hunter_software_hive() -> Vec<u8> {
    let mut key = reg_key(
        "Run",
        vec![(
            "ExampleAgent",
            REG_SZ,
            utf16z(r"C:\Example\agent.exe --start"),
        )],
        vec![],
    );
    for parent in ["CurrentVersion", "Windows", "Microsoft", "SOFTWARE"] {
        key = reg_key(parent, vec![], vec![key]);
    }
    regf(&key)
}

#[test]
fn persistence_hunter_reports_failed_image_reads_without_losing_readable_findings() {
    let evidence = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    let config = evidence.path().join("Windows/System32/config");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::create_dir(evidence.path().join("Example")).unwrap();
    std::fs::write(config.join("SOFTWARE"), hunter_software_hive()).unwrap();
    let payload = evidence.path().join("Example/agent.exe");
    std::fs::write(&payload, []).unwrap();
    let sources = output.path().join("ImageFiles.json");
    std::fs::write(
        &sources,
        serde_json::to_vec(&json!({
            "Workspace": output.path().canonicalize().unwrap(),
            "Volumes": [{
                "Root": evidence.path().canonicalize().unwrap(),
                "Image": output.path().join("unavailable.E01"),
                "Source": "./evidence.E01",
                "PathPrefix": "Volume2",
                "Volume": 0
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    let findings = output.path().join("findings.jsonl");
    let r = run(&[
        "persistence-hunter",
        "-d",
        evidence.path().to_str().unwrap(),
        "--check",
        "run_keys",
        "--no-timeline",
        "--image-files",
        sources.to_str().unwrap(),
        "-o",
        findings.to_str().unwrap(),
        "-t",
        "jsonl",
        "-K",
        "-q",
    ]);
    let errors: Vec<Value> = r
        .stderr
        .lines()
        .filter_map(|line| {
            line.strip_prefix("JUNTEI_IMAGE_READ_PROBLEM: ")
                .map(|json| serde_json::from_str(json).unwrap())
        })
        .collect();
    assert!(
        errors.iter().any(|error| error["Image"] == "./evidence.E01"
            && error["Path"] == "Volume2/Example/agent.exe"),
        "{}",
        r.stderr
    );
    let records = jsonl_records(&std::fs::read_to_string(findings).unwrap());
    assert!(
        records
            .iter()
            .any(|record| record["Value"] == r"C:\Example\agent.exe --start")
    );
    let retained = records
        .iter()
        .find(|record| record["Value"] == r"C:\Example\agent.exe --start")
        .unwrap();
    assert_eq!(retained["SHA256"], "");
    assert!(
        retained["ResolutionStatus"]
            .as_str()
            .unwrap()
            .starts_with("unreadable:")
    );
    assert!(records.iter().all(|record| record["SHA256"]
        != "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"));
    assert_eq!(std::fs::metadata(payload).unwrap().len(), 0);
}

#[test]
fn persistence_hunter_selected_check_has_csv_json_and_jsonl_parity() {
    let source = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    let hive = source.path().join("SOFTWARE");
    std::fs::write(&hive, hunter_software_hive()).unwrap();
    let mut expected = None;
    for format in ["csv", "json", "jsonl"] {
        let result = output.path().join(format!("findings.{format}"));
        let coverage = output.path().join(format!("coverage-{format}.csv"));
        let r = run(&[
            "persistence-hunter",
            "-f",
            hive.to_str().unwrap(),
            "--check",
            "run_keys",
            "--no-timeline",
            "-o",
            result.to_str().unwrap(),
            "-t",
            format,
            "--coverage",
            coverage.to_str().unwrap(),
            "-q",
            "-K",
        ]);
        assert_eq!(r.code, 0, "{}\n{}", r.stdout, r.stderr);
        let (headers, rows) = csv_rows(&coverage);
        assert!(headers.iter().any(|name| name == "Status"));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["CheckID"], "run_keys");
        if format == "csv" {
            let (_, rows) = csv_rows(&result);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0]["Value"], r"C:\Example\agent.exe --start");
            expected = Some(rows[0]["Path"].clone());
        } else {
            let f: Value = serde_json::from_slice(&std::fs::read(result).unwrap()).unwrap();
            assert_eq!(f["Path"], expected.as_ref().unwrap().as_str());
            assert_eq!(f["CheckID"], "run_keys");
            assert_eq!(f["Value"], r"C:\Example\agent.exe --start");
            assert!(f["Exists"].is_null());
        }
    }
    let r = run(&[
        "persistence-hunter",
        "-f",
        hive.to_str().unwrap(),
        "--technique",
        "T1547.001",
        "--no-timeline",
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let f: Value = serde_json::from_str(&r.stdout).unwrap();
    assert_eq!(f["CheckID"], "run_keys");
    assert!(!r.stdout.contains("JUNTEI"));
}

#[test]
fn persistence_hunter_names_com_registrations_as_the_input_was_given() {
    // a task's COM handler registered in the machine's SOFTWARE hive: the finding names the hive,
    // from the evidence directory as given, not where it is on the analyst's system
    let evidence = tempfile::tempdir().unwrap();
    let windows = evidence.path().join("HOST/C/Windows");
    let config = windows.join("System32/config");
    let tasks = windows.join("System32/Tasks");
    for dir in [&config, &tasks] {
        std::fs::create_dir_all(dir).unwrap();
    }
    let clsid = "{00112233-4455-6677-8899-AABBCCDDEEFF}";
    std::fs::write(
        config.join("SOFTWARE"),
        reg_tree(
            "ROOT",
            &[(
                &format!(r"Classes\CLSID\{clsid}\InprocServer32"),
                "",
                REG_SZ,
                utf16z(r"C:\Example\handler.dll"),
            )],
        ),
    )
    .unwrap();
    std::fs::write(tasks.join("ExampleTask"), format!(r#"<Task xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task"><Principals><Principal id="Author"><UserId>S-1-5-18</UserId></Principal></Principals><Actions Context="Author"><ComHandler><ClassId>{clsid}</ClassId></ComHandler></Actions></Task>"#)).unwrap();
    let r = run_at(
        evidence.path(),
        &[
            "persistence-hunter",
            "-d",
            "./HOST/C",
            "--min-severity",
            "info",
            "--no-timeline",
            "-t",
            "jsonl",
        ],
    );
    assert_eq!(r.code, 0, "{}", r.stderr);
    let findings = jsonl_records(&r.stdout);
    let registration = findings
        .iter()
        .filter_map(|f| f["Description"].as_str())
        .find(|d| d.contains("Registration: "))
        .unwrap_or_else(|| panic!("{}", r.stdout));
    let hive = [
        ".", "HOST", "C", "Windows", "System32", "config", "SOFTWARE",
    ]
    .join(std::path::MAIN_SEPARATOR_STR);
    assert!(
        registration.contains(&format!("Registration: {hive}")),
        "{registration}"
    );
    let canonical = std::fs::canonicalize(evidence.path()).unwrap();
    assert!(
        !r.stdout.contains(canonical.to_str().unwrap()),
        "{}",
        r.stdout
    );
}

#[test]
fn persistence_workers_preserve_records_coverage_and_errors_in_every_format() {
    let evidence = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    let windows = evidence.path().join("Windows");
    let config = windows.join("System32/config");
    let tasks = windows.join("System32/Tasks");
    let logs = windows.join("System32/winevt/Logs");
    for dir in [&config, &tasks, &logs, &evidence.path().join("Example")] {
        std::fs::create_dir_all(dir).unwrap();
    }
    std::fs::write(config.join("SOFTWARE"), hunter_software_hive()).unwrap();
    std::fs::write(config.join("SYSTEM"), services_hive(1)).unwrap();
    std::fs::write(
        evidence.path().join("Example/agent.exe"),
        b"example payload bytes",
    )
    .unwrap();
    std::fs::copy(repo(MFT), evidence.path().join("$MFT")).unwrap();
    std::fs::write(tasks.join("ExampleTask"), br#"<Task xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task"><Principals><Principal id="Author"><UserId>S-1-5-18</UserId></Principal></Principals><Actions Context="Author"><Exec><Command>C:\Example\agent.exe</Command><Arguments>--task</Arguments></Exec></Actions></Task>"#).unwrap();
    // Independent channel workers must preserve both failures and their order;
    // other successfully parsed artifacts must remain in the output.
    std::fs::write(logs.join("System.evtx"), b"damaged system channel").unwrap();
    std::fs::write(logs.join("Security.evtx"), b"damaged security channel").unwrap();
    #[cfg(unix)]
    {
        let outside = output.path().join("not-evidence");
        std::fs::create_dir_all(&outside).unwrap();
        for branch in ["First/nested", "Second"] {
            let path = evidence.path().join(branch);
            std::fs::create_dir_all(&path).unwrap();
            std::os::unix::fs::symlink(&outside, path.join("outside-link")).unwrap();
        }
    }

    for format in ["csv", "json", "jsonl"] {
        let mut first = None;
        for threads in ["1", "4", "2"] {
            let findings = output.path().join(format!("findings-{format}-{threads}"));
            let coverage = output
                .path()
                .join(format!("coverage-{format}-{threads}.csv"));
            let r = run(&[
                "persistence-hunter",
                "-d",
                evidence.path().to_str().unwrap(),
                "--min-severity",
                "info",
                "-T",
                threads,
                "-t",
                format,
                "-o",
                findings.to_str().unwrap(),
                "--coverage",
                coverage.to_str().unwrap(),
                "-q",
                "-K",
            ]);
            assert_eq!(r.code, 0, "{}\n{}", r.stdout, r.stderr);
            let result = (
                std::fs::read(&findings).unwrap(),
                std::fs::read(&coverage).unwrap(),
            );
            if let Some(first) = &first {
                assert_eq!(
                    &result, first,
                    "{format}: -T {threads} changed findings or coverage"
                );
            } else {
                let (_, rows) = csv_rows(&coverage);
                assert_eq!(
                    rows.iter()
                        .filter(|r| r["CheckID"] != "context"
                            && r["CheckID"] != "timestamp_correlation")
                        .count(),
                    131
                );
                assert!(rows.iter().any(|r| r["CheckID"] == "timestamp_correlation"
                    && r["Detail"].contains("System.evtx")));
                assert!(rows.iter().any(|r| r["CheckID"] == "timestamp_correlation"
                    && r["Detail"].contains("Security.evtx")));
                if format == "csv" {
                    let (_, rows) = csv_rows(&findings);
                    assert!(
                        rows.iter()
                            .any(|r| r["CheckID"] == "run_keys" && !r["SHA256"].is_empty())
                    );
                    assert!(rows.iter().any(|r| r["CheckID"] == "scheduled_task_files"));
                }
                first = Some(result);
            }
        }
    }
    for threads in ["0", "invalid", "-1"] {
        assert_ne!(
            run(&[
                "persistence-hunter",
                "-d",
                evidence.path().to_str().unwrap(),
                "--threads",
                threads
            ])
            .code,
            0
        );
    }
}

#[test]
fn persistence_hunter_rejects_bad_inputs_and_preserves_evidence() {
    let source = tempfile::tempdir().unwrap();
    let hive = source.path().join("SOFTWARE");
    let original = hunter_software_hive();
    std::fs::write(&hive, &original).unwrap();
    for args in [
        vec!["--check", "does_not_exist"],
        vec!["-o", hive.to_str().unwrap(), "-C"],
        vec!["-o", "same.csv", "--coverage", "./same.csv", "-C"],
    ] {
        let mut command = vec![
            "persistence-hunter",
            "-f",
            hive.to_str().unwrap(),
            "--no-timeline",
        ];
        command.extend(args);
        let r = run(&command);
        assert_ne!(r.code, 0, "{}", r.stdout);
        assert_eq!(std::fs::read(&hive).unwrap(), original);
    }
    let fake = source.path().join("fake.bin");
    std::fs::write(&fake, b"not a hive").unwrap();
    assert_ne!(
        run(&["persistence-hunter", "-f", fake.to_str().unwrap()]).code,
        0
    );
    let r = run(&["persistence-hunter", "--list-checks"]);
    assert_eq!(r.code, 0);
    assert_eq!(r.stdout.lines().count(), 131);
}

#[test]
fn persistence_hunter_protects_adjacent_logs_and_conflicting_destinations() {
    let source = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    let hive = source.path().join("SOFTWARE");
    std::fs::write(&hive, hunter_software_hive()).unwrap();
    let log = hive.with_extension("LOG1");
    std::fs::write(&log, b"original transaction-log evidence").unwrap();
    for flag in ["-o", "--coverage"] {
        let r = run(&[
            "persistence-hunter",
            "-f",
            hive.to_str().unwrap(),
            "--check",
            "run_keys",
            "--no-timeline",
            flag,
            log.to_str().unwrap(),
            "-C",
        ]);
        assert_ne!(r.code, 0, "{}", r.stdout);
        assert_eq!(
            std::fs::read(&log).unwrap(),
            b"original transaction-log evidence"
        );
    }
    let traversal = source.path().join("missing/../SOFTWARE");
    let original = std::fs::read(&hive).unwrap();
    let r = run(&[
        "persistence-hunter",
        "-f",
        hive.to_str().unwrap(),
        "--check",
        "run_keys",
        "--no-timeline",
        "-o",
        traversal.to_str().unwrap(),
        "-C",
    ]);
    assert_ne!(r.code, 0);
    assert_eq!(std::fs::read(&hive).unwrap(), original);
    assert!(!source.path().join("missing").exists());
    #[cfg(unix)]
    {
        let alias = output.path().join("evidence-link");
        std::os::unix::fs::symlink(source.path(), &alias).unwrap();
        let traversal = output.path().join("missing/../evidence-link/SOFTWARE");
        let r = run(&[
            "persistence-hunter",
            "-f",
            hive.to_str().unwrap(),
            "--check",
            "run_keys",
            "--no-timeline",
            "-o",
            traversal.to_str().unwrap(),
            "-C",
        ]);
        assert_ne!(r.code, 0);
        assert_eq!(std::fs::read(&hive).unwrap(), original);
        assert!(!output.path().join("missing").exists());
    }
    let findings = output.path().join("result");
    let child_coverage = findings.join("coverage.csv");
    std::fs::write(&findings, b"previous results").unwrap();
    let r = run(&[
        "persistence-hunter",
        "-f",
        hive.to_str().unwrap(),
        "--check",
        "run_keys",
        "--no-timeline",
        "-o",
        findings.to_str().unwrap(),
        "--coverage",
        child_coverage.to_str().unwrap(),
        "-C",
    ]);
    assert_ne!(r.code, 0);
    assert_eq!(std::fs::read(&findings).unwrap(), b"previous results");
    let a = output.path().join("new-result");
    let b = a.join("coverage.csv");
    let r = run(&[
        "persistence-hunter",
        "-f",
        hive.to_str().unwrap(),
        "--check",
        "run_keys",
        "--no-timeline",
        "-o",
        a.to_str().unwrap(),
        "--coverage",
        b.to_str().unwrap(),
    ]);
    assert_ne!(r.code, 0);
    assert!(!a.exists());
    let mixed = output.path().join("Results.csv");
    let lower = output.path().join("results.csv");
    let r = run(&[
        "persistence-hunter",
        "-f",
        hive.to_str().unwrap(),
        "--check",
        "run_keys",
        "--no-timeline",
        "-o",
        mixed.to_str().unwrap(),
        "--coverage",
        lower.to_str().unwrap(),
        "-C",
    ]);
    assert_ne!(r.code, 0);
    assert!(!mixed.exists());
    assert!(!lower.exists());
}

#[test]
fn persistence_hunter_disabled_profile_retains_coverage_without_findings() {
    let source = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    let hive = source.path().join("SOFTWARE");
    let profile = source.path().join("profile.yaml");
    let findings = output.path().join("findings.csv");
    let coverage = output.path().join("coverage.csv");
    std::fs::write(&hive, hunter_software_hive()).unwrap();
    std::fs::write(&profile, "checks:\n  run_keys:\n    enabled: false\n").unwrap();
    let r = run(&[
        "persistence-hunter",
        "-f",
        hive.to_str().unwrap(),
        "--check",
        "run_keys",
        "--no-timeline",
        "--profile",
        profile.to_str().unwrap(),
        "-o",
        findings.to_str().unwrap(),
        "--coverage",
        coverage.to_str().unwrap(),
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(csv_rows(&findings).1.len(), 0);
    let rows = csv_rows(&coverage).1;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["Status"], "disabled");
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
            "SRUM-AppResourceUseInfo.csv",
            "SRUM-AppTimelineProvider.csv",
            "SRUM-IdentityMap.csv",
            "SRUM-NetworkConnections.csv",
            "SRUM-NetworkUsages.csv",
            "SRUM-VfuProvider.csv"
        ]
    );

    let (header, rows) = csv_rows(&out.join("SRUM-NetworkUsages.csv"));
    assert_eq!(
        header,
        [
            "ID",
            "Timestamp",
            "EXE-Info",
            "EXE-InfoDescription",
            "EXE-Timestamp",
            "AppType",
            "SID",
            "User",
            "UserID",
            "AppID",
            "InterfaceLUID",
            "InterfaceType",
            "L2ProfileID",
            "ProfileName",
            "L2ProfileFlags",
            "BytesSent",
            "BytesReceived"
        ]
    );
    assert_eq!(rows.len(), 23);
    let first = &rows[0];
    for (column, value) in [
        ("ID", "1"),
        ("Timestamp", "2022-02-05T22:19:00Z"),
        ("EXE-Info", "wlidsvc"),
        ("AppType", "Service"),
        ("SID", "S-1-5-19"),
        ("User", "LocalService"),
        ("InterfaceType", "IF_TYPE_ETHERNET_CSMACD"),
        ("ProfileName", "CoffeeShop"),
        ("BytesSent", "11988"),
        ("BytesReceived", "12755"),
    ] {
        assert_eq!(first[column], value, "{column}");
    }
    let rathbun: Vec<_> = rows.iter().filter(|r| r["SID"] == RATHBUN_SID).collect();
    assert!(!rathbun.is_empty());
    assert!(rathbun.iter().all(|r| r["User"] == "rathbun"));

    // `!!name!time!x!description` names split; FILETIME columns as times
    let (_, timeline) = csv_rows(&out.join("SRUM-AppTimelineProvider.csv"));
    let split = timeline
        .iter()
        .find(|r| r["EXE-InfoDescription"] == "[LocalService] [WdiServiceHost]")
        .unwrap();
    assert_eq!(split["EXE-Info"], "svchost.exe");
    assert_eq!(split["EXE-Timestamp"], "1972-12-14T16:22:50Z");
    assert_eq!(split["EndTime"], "2022-02-05T22:18:49.336199700Z");
    assert!(
        timeline
            .iter()
            .all(|r| r["EndTime"].starts_with("2022-02-"))
    );
    let (_, ids) = csv_rows(&out.join("SRUM-IdentityMap.csv"));
    assert_eq!(ids.len(), 288);
    assert_eq!(ids[3]["ID-Blob"], "S-1-5-19");
    let (_, vfu) = csv_rows(&out.join("SRUM-VfuProvider.csv"));
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
    assert!(connection["User"].is_null());

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
        let states = e["ValueState"].as_object().unwrap();
        // the database's values, not what is made from them
        for column in ["ID", "Timestamp", "UserID", "AppID"] {
            assert_eq!(states[column], "parsed", "{column}");
        }
        for made in ["EXE-Info", "SID", "AppType", "SourceFile"] {
            assert!(!states.contains_key(made), "{made}");
        }
        // the id's bytes: a Long, little-endian
        let id = i32::try_from(e["ID"].as_i64().unwrap()).unwrap();
        let hex: String = id
            .to_le_bytes()
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect();
        assert_eq!(e["Raw"]["ID"], hex);
    }
    // a NULL in the database: null, no bytes
    let timeline = entries
        .iter()
        .find(|e| e["Provider"] == "AppTimelineProvider" && e["AudioInS"].is_null())
        .unwrap();
    assert_eq!(timeline["ValueState"]["AudioInS"], "null");
    assert!(timeline["Raw"].get("AudioInS").is_none());
    let nulls = entries
        .iter()
        .flat_map(|e| e["ValueState"].as_object().unwrap().values())
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
            "SRUM-AppResourceUseInfo.csv",
            "SRUM-IdentityMap.csv",
            "SRUM-NetworkBytes.csv",
            "SRUM-NetworkConnections.csv",
            "SRUM-ProcessorTime.csv",
            "SRUM-SizeInBytes.csv",
            "SRUM-TotalUsed.csv"
        ]
    );
    let (_, used) = csv_rows(&server.join("SRUM-TotalUsed.csv"));
    assert_eq!(used.len(), 4);
    assert!(used.iter().all(|r| r["AppType"] == "Object"));
    let (_, network) = csv_rows(
        &out.path()
            .join("HOST1_C_Windows_System32_sru_SRUDB/SRUM-NetworkUsages.csv"),
    );
    assert_eq!(network[0]["User"], "LocalService");

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

// --- parse-ese ual ---------------------------------------------------------------------------------

const UAL_SERVER: &str = "test_files/ual/server2019";
const UAL_CURRENT: &str = "test_files/ual/server2019/Current.mdb";

#[test]
fn ual_summary_on_the_terminal() {
    let r = run(&["parse-ese", "ual", "-d", UAL_SERVER, "-K"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for line in [
        "UAL databases: 3\n",
        "UAL databases parsed: 3\n",
        "Rows: 17\n",
        "  RoleAccess: 1\n  SystemIdentity: 1\n  ChainedDatabases: 2\n  Roles: 13\n",
        "Client addresses / accounts: 0 / 0\n",
        "Role names: from SystemIdentity.mdb where available; otherwise built in\n",
        "Problems: 0\n",
        "To save every row, add -o <DIR>",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    assert!(!r.stdout.contains("Left dirty"));
    assert!(!r.stdout.contains("Client access:"));
}

#[test]
fn ual_csv_tables() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("ual");
    let args = [
        "parse-ese",
        "ual",
        "-d",
        UAL_SERVER,
        "-o",
        out.to_str().unwrap(),
        "-K",
    ];
    let r = run(&args);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(r.stdout.contains("(7 CSV files)"), "{}", r.stdout);
    let mut files: Vec<String> = std::fs::read_dir(&out)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    files.sort();
    assert_eq!(
        files,
        [
            "UAL-ChainedDatabases.csv",
            "UAL-Clients.csv",
            "UAL-DNS.csv",
            "UAL-RoleAccess.csv",
            "UAL-Roles.csv",
            "UAL-SystemIdentity.csv",
            "UAL-VirtualMachines.csv"
        ]
    );
    // a table without rows has its header
    let (header, rows) = csv_rows(&out.join("UAL-Clients.csv"));
    assert_eq!(
        header,
        [
            "SourceFile",
            "RoleGUID",
            "RoleName",
            "TenantID",
            "TotalAccesses",
            "InsertDate",
            "LastAccess",
            "Address",
            "AuthenticatedUserName",
            "ClientName",
            "DailyAccesses"
        ]
    );
    assert!(rows.is_empty());
    let (header, _) = csv_rows(&out.join("UAL-VirtualMachines.csv"));
    assert_eq!(
        header,
        [
            "SourceFile",
            "VmGUID",
            "BIOS-GUID",
            "CreationTime",
            "LastSeenActive",
            "SerialNumber"
        ]
    );
    let (header, rows) = csv_rows(&out.join("UAL-RoleAccess.csv"));
    assert_eq!(
        header,
        [
            "SourceFile",
            "RoleGUID",
            "RoleName",
            "FirstSeen",
            "LastSeen"
        ]
    );
    assert_eq!(rows.len(), 1);
    // as found under -d: Windows separates the names found with `\`
    assert_eq!(Path::new(&rows[0]["SourceFile"]), Path::new(UAL_CURRENT));
    assert_eq!(
        rows[0]["RoleGUID"],
        "{7FB09BD3-7FE6-435E-8348-7D8AEFB6CEA3}"
    );
    assert_eq!(rows[0]["RoleName"], "Print and Document Services");
    assert_eq!(rows[0]["FirstSeen"], "2023-01-11T03:54:53.611605300Z");
    assert_eq!(rows[0]["LastSeen"], "2023-01-11T03:54:53.611605300Z");

    let (header, rows) = csv_rows(&out.join("UAL-SystemIdentity.csv"));
    assert_eq!(header.len(), 26);
    let system = &rows[0];
    for (field, value) in [
        ("CreationTime", "2022-11-23T02:38:33.374Z"),
        ("OS-BuildNumber", "17763"),
        ("OS-ProductType", "3"),
        ("OS-ProductTypeName", "Server"),
        ("OS-CurrentTimeZone", "0"),
        ("OS-DaylightInEffect", "false"),
        ("SystemDNS-HostName", "researcher-vm"),
        ("SystemDomainName", "WORKGROUP"),
        ("OS-LastBootUpTime", "2023-01-11T03:54:11.500000Z"),
    ] {
        assert_eq!(system[field], value, "{field}");
    }
    let (_, years) = csv_rows(&out.join("UAL-ChainedDatabases.csv"));
    assert_eq!(years[1]["Year"], "2023");
    assert_eq!(
        years[1]["Filename"],
        "{32D77B8F-140E-4668-8289-F964D6E7C2F3}.mdb"
    );
    let (header, roles) = csv_rows(&out.join("UAL-Roles.csv"));
    assert_eq!(
        header,
        ["SourceFile", "RoleGUID", "ProductName", "RoleName"]
    );
    assert_eq!(roles.len(), 13);
    assert!(roles.iter().any(|r| r["RoleName"] == "File Server"));

    // an existing directory needs -C
    let r = run(&args);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("already exists"), "{}", r.stdout);
}

#[test]
fn ual_json_and_jsonl() {
    let r = run(&["parse-ese", "ual", "-f", UAL_CURRENT, "-t", "jsonl"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(!r.stdout.contains("Results Summary"));
    assert_eq!(
        r.stdout,
        concat!(
            r#"{"SourceFile":"test_files/ual/server2019/Current.mdb","Table":"ROLE_ACCESS","#,
            r#""RoleGUID":"{7FB09BD3-7FE6-435E-8348-7D8AEFB6CEA3}","#,
            r#""RoleName":"Print and Document Services","#,
            r#""FirstSeen":"2023-01-11T03:54:53.611605300Z","#,
            r#""LastSeen":"2023-01-11T03:54:53.611605300Z"}"#,
            "\n"
        )
    );
    // every database of a directory on stdout: the rows say where they come from
    let r = run(&["parse-ese", "ual", "-d", UAL_SERVER, "-t", "jsonl"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let rows = jsonl_records(&r.stdout);
    assert_eq!(rows.len(), 17);
    assert!(
        rows.iter()
            .all(|row| row["SourceFile"] != UAL_SERVER && row["Table"].is_string())
    );
    let identity = rows
        .iter()
        .find(|row| row["Table"] == "SYSTEM_IDENTITY")
        .unwrap();
    assert_eq!(identity["OS-LastBootUpTime"], "2023-01-11T03:54:11.500000Z");
    assert_eq!(identity["MaximumMemory"], 4_294_967_296u64);

    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("ual.json");
    let r = run(&[
        "parse-ese",
        "ual",
        "-d",
        UAL_SERVER,
        "-t",
        "json",
        "-o",
        file.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(r.stdout.contains("Results saved: "), "{}", r.stdout);
    let documents = json_documents(&std::fs::read_to_string(&file).unwrap());
    assert_eq!(documents, rows);
}

#[test]
fn ual_role_names_without_system_identity() {
    // Current.mdb alone: the roles Windows Server defines are still named
    let dir = tempfile::tempdir().unwrap();
    let current = dir.path().join("Current.mdb");
    std::fs::copy(repo(UAL_CURRENT), &current).unwrap();
    let r = run(&["parse-ese", "ual", "-f", current.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout
            .contains("Role names: built in (no readable SystemIdentity.mdb role names)"),
        "{}",
        r.stdout
    );
    let r = run(&[
        "parse-ese",
        "ual",
        "-f",
        current.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(
        jsonl_records(&r.stdout)[0]["RoleName"],
        "Print and Document Services"
    );
    // a SystemIdentity.mdb that can't be read: the rows are still written
    std::fs::write(dir.path().join("systemidentity.MDB"), b"not ESE").unwrap();
    let r = run(&["parse-ese", "ual", "-f", current.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stdout.contains("(built-in role names are used)"),
        "{}",
        r.stdout
    );
    assert!(r.stdout.contains("Rows: 1\n"), "{}", r.stdout);
    assert!(
        r.stdout
            .contains("Role names: built in (no readable SystemIdentity.mdb role names)"),
        "{}",
        r.stdout
    );
    assert!(r.stdout.contains("Problems: 1\n"), "{}", r.stdout);
}

#[test]
fn ual_input_and_output_errors() {
    let r = run(&["parse-ese", "ual", "-f", "test_files/ual/missing.mdb", "-K"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("The file does not exist"), "{}", r.stdout);

    let r = run(&["parse-ese", "ual", "-f", "test_files/ual/README.md", "-K"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("Could not parse"), "{}", r.stdout);

    let r = run(&["parse-ese", "ual", "-d", "test_files/amcache", "-K"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains(
            "No UAL databases (Current.mdb, {GUID}.mdb, SystemIdentity.mdb) were found in"
        ),
        "{}",
        r.stdout
    );

    let r = run(&["parse-ese", "ual", "-f", UAL_CURRENT, "-t", "csv"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("CSV output needs -o <DIR>"),
        "{}",
        r.stdout
    );

    // the output is never one of the databases read, even with -C
    let dir = tempfile::tempdir().unwrap();
    let sum = dir.path().join("Sum");
    copy_ual(&sum);
    let identity = sum.join("SystemIdentity.mdb");
    let before = std::fs::read(&identity).unwrap();
    let r = run(&[
        "parse-ese",
        "ual",
        "-f",
        sum.join("Current.mdb").to_str().unwrap(),
        "-t",
        "jsonl",
        "-o",
        identity.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1, "{}", r.stdout);
    assert!(
        r.stdout.contains("The output file is also an input file"),
        "{}",
        r.stdout
    );
    assert_eq!(std::fs::read(&identity).unwrap(), before);
}

#[test]
fn ual_reports_damaged_identity_lookup_and_keeps_access_rows() {
    let dir = tempfile::tempdir().unwrap();
    copy_ual(dir.path());
    let identity = dir.path().join("SystemIdentity.mdb");
    let mut bytes = std::fs::read(&identity).unwrap();
    let database = ese::Database::open(&bytes).unwrap();
    let role_page = database.table("ROLE_IDS").unwrap().root_page as usize;
    let page_size = database.header.page_size as usize;
    // Keep the catalog and other tables intact, but make the role-name lookup unreadable.
    let at = (role_page + 1) * page_size;
    bytes[at..at + page_size].fill(0);
    std::fs::write(&identity, bytes).unwrap();
    let current = dir.path().join("Current.mdb");
    let r = run(&["parse-ese", "ual", "-f", current.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 0, "{}\n{}", r.stdout, r.stderr);
    assert!(r.stdout.contains("ROLE_IDS: page"), "{}", r.stdout);
    assert!(r.stdout.contains("is blank"), "{}", r.stdout);
    assert!(r.stdout.contains("Problems: 1\n"), "{}", r.stdout);
    assert!(r.stdout.contains("Rows: 1\n"), "{}", r.stdout);
    let r = run(&[
        "parse-ese",
        "ual",
        "-f",
        current.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let rows = jsonl_records(&r.stdout);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["RoleName"], "Print and Document Services");
    assert!(r.stderr.contains("ROLE_IDS: page"), "{}", r.stderr);

    // With directory input, the metadata file is also parsed directly. Report its damage
    // once, and retain its other tables as well as Current.mdb's access row.
    let r = run(&["parse-ese", "ual", "-d", dir.path().to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 0, "{}\n{}", r.stdout, r.stderr);
    assert_eq!(
        r.stdout.matches("ROLE_IDS: page").count(),
        1,
        "{}",
        r.stdout
    );
    assert!(r.stdout.contains("Rows: 4\n"), "{}", r.stdout);
    assert!(r.stdout.contains("Problems: 1\n"), "{}", r.stdout);
}

#[test]
fn auto_parse_puts_ual_tables_in_lateral_movement_with_their_times_in_the_offset() {
    let evidence = tempfile::tempdir().unwrap();
    copy_ual(&evidence.path().join("HOST/C/Windows/System32/LogFiles/Sum"));
    let target = tempfile::tempdir().unwrap();
    for kind in ["csv", "json", "jsonl"] {
        let output = target.path().join(kind);
        let result = run(&[
            "auto-parse",
            "-d",
            evidence.path().to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-t",
            kind,
            "-j",
            output.join("all.jsonl").to_str().unwrap(),
            "-Z",
            "+09:00",
            "-T",
            "2",
            "-K",
            "-q",
        ]);
        assert_eq!(result.code, 0, "{}\n{}", result.stdout, result.stderr);
        let manifest: Value =
            serde_json::from_slice(&std::fs::read(output.join("manifest.json")).unwrap()).unwrap();
        let task = manifest["Tasks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["Artifact"] == "ual" && t["Purpose"] == "artifact")
            .unwrap();
        assert_eq!(task["Status"], "succeeded", "{task}");
        assert_eq!(task["Category"], "LateralMovement");
        assert_eq!(task["Command"][1], "parse-ese");
        assert_eq!(task["Command"][2], "ual");
        let lateral = output.join("HOST/LateralMovement");
        assert!(lateral.join("UAL-logs.txt").is_file());
        if kind == "csv" {
            assert!(!lateral.join("UAL").exists());
            let files = task["OutputFiles"].as_array().unwrap();
            assert_eq!(files.len(), 7, "{task}");
            let (_, access) = csv_rows(&lateral.join("UAL-RoleAccess.csv"));
            assert_eq!(
                access[0]["FirstSeen"],
                "2023-01-11T12:54:53.611605300+09:00"
            );
            let (_, identity) = csv_rows(&lateral.join("UAL-SystemIdentity.csv"));
            assert_eq!(
                identity[0]["OS-LastBootUpTime"],
                "2023-01-11T12:54:11.500000+09:00"
            );
            assert_eq!(identity[0]["CreationTime"], "2022-11-23T11:38:33.374+09:00");
        } else {
            let rows = json_documents(
                &std::fs::read_to_string(lateral.join(format!("UAL.{kind}"))).unwrap(),
            );
            assert_eq!(rows.len(), 17);
            assert!(
                rows.iter()
                    .all(|r| r["Comp"] == "HOST" && r["Artifact"] == "ual")
            );
        }
        let timeline = jsonl_records(&std::fs::read_to_string(output.join("all.jsonl")).unwrap());
        let ual: Vec<&Value> = timeline.iter().filter(|r| r["Artifact"] == "ual").collect();
        assert_eq!(ual.len(), 17);
        let access = ual.iter().find(|r| r["Table"] == "ROLE_ACCESS").unwrap();
        assert_eq!(access["FirstSeen"], "2023-01-11T12:54:53.611605300+09:00");
        assert_eq!(access["LastSeen"], "2023-01-11T12:54:53.611605300+09:00");
        assert_eq!(access["RoleName"], "Print and Document Services");
    }
}

// --- mount and auto-parse --vhdx ------------------------------------------------------------------

/// The sample VHDX (test_files/vhdx), decompressed to `path`.
fn sample_vhdx(path: &Path) {
    let gz = std::fs::File::open(repo("test_files/vhdx/sample.vhdx.gz")).unwrap();
    let mut out = std::fs::File::create(path).unwrap();
    std::io::copy(&mut flate2::read::GzDecoder::new(gz), &mut out).unwrap();
}

fn add_unreadable_partition_to_sample(path: &Path) {
    let mut bytes = std::fs::read(path).unwrap();
    // The first logical MiB is fully present. Resolve its BAT entry in this fixture only.
    let bat = u64::from_le_bytes(bytes[3 << 20..(3 << 20) + 8].try_into().unwrap());
    assert_eq!(bat & 7, 6);
    let payload = (bat & !((1 << 20) - 1)) as usize;
    let entry = payload + 446 + 16;
    bytes[entry + 4] = 0x07;
    bytes[entry + 8..entry + 12].copy_from_slice(&64u32.to_le_bytes());
    bytes[entry + 12..entry + 16].copy_from_slice(&1u32.to_le_bytes());
    bytes[payload + 64 * 512..payload + 65 * 512].fill(0);
    bytes[payload + 64 * 512 + 3..payload + 64 * 512 + 11].copy_from_slice(b"NTFS    ");
    std::fs::write(path, bytes).unwrap();
}

/// Whether this user can mount with FUSE here (Linux with /dev/fuse and fusermount3). Without it
/// the tests that mount say so and stop.
#[cfg(target_os = "linux")]
fn fuse_available() -> bool {
    let usable = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/fuse")
        .is_ok()
        && Command::new("fusermount3")
            .arg("-V")
            .output()
            .is_ok_and(|o| o.status.success());
    if !usable {
        // CI sets this where the mount must be tested
        assert!(
            std::env::var("JUNTEI_REQUIRE_FUSE").as_deref() != Ok("1"),
            "FUSE is not available, and JUNTEI_REQUIRE_FUSE is 1"
        );
        eprintln!("FUSE is not available: the mount is not tested");
    }
    usable
}

/// Whether anything is mounted from an image named `name`.
#[cfg(target_os = "linux")]
fn mounted_from(name: &str) -> bool {
    std::fs::read_to_string("/proc/mounts")
        .unwrap_or_default()
        .lines()
        .any(|line| line.starts_with(&format!("juntei:{name} ")))
}

#[cfg(target_os = "linux")]
#[test]
fn mount_serves_a_vhdx_read_only_until_interrupted() {
    if !fuse_available() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let image = dir.path().join("mount-test.vhdx");
    sample_vhdx(&image);
    let point = dir.path().join("mnt");
    std::fs::create_dir(&point).unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_juntei"))
        .args([
            "mount",
            image.to_str().unwrap(),
            point.to_str().unwrap(),
            "-K",
            "-q",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let data = point.join("C/Users/sample/Data");
    for _ in 0..100 {
        if data.is_dir() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    assert!(data.is_dir(), "not mounted");
    assert!(mounted_from("mount-test.vhdx"));
    let sha256 = |path: &Path| {
        use sha2::{Digest, Sha256};
        Sha256::digest(std::fs::read(path).unwrap())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    assert_eq!(
        std::fs::read(data.join("small.txt")).unwrap(),
        b"resident content\n"
    );
    assert_eq!(
        sha256(&data.join("fragmented.bin")),
        "dad53fd22d62e1acbf48ababc172c8eb3ffb45486b261fb960a2990199a8e459"
    );
    assert_eq!(
        sha256(&data.join("compressed/text.log")),
        "cc3c52b090233f44ecfc2dbd90adc6c787e071872ca488fa96eae24af8162f98"
    );
    assert!(data.join("日本語のファイル.txt").is_file());
    // read-only: no new file, no writing to a file, no deleting
    assert!(std::fs::write(data.join("new.txt"), b"x").is_err());
    assert!(
        std::fs::OpenOptions::new()
            .write(true)
            .open(data.join("small.txt"))
            .is_err()
    );
    assert!(std::fs::remove_file(data.join("small.txt")).is_err());
    assert!(
        std::fs::metadata(data.join("small.txt"))
            .unwrap()
            .permissions()
            .readonly()
    );
    // Ctrl+C unmounts
    let killed = Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(killed.success());
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{stdout}");
    assert!(stdout.contains("NTFS volumes: 1 ("), "{stdout}");
    assert!(stdout.contains("Mounted read-only at "), "{stdout}");
    assert!(stdout.contains("Unmounted: "), "{stdout}");
    assert!(!mounted_from("mount-test.vhdx"));
    assert!(!data.exists());
}

/// Without FUSE support (Windows, macOS), mounting says what to do instead.
#[cfg(not(target_os = "linux"))]
#[test]
fn only_os_visible_mounting_needs_fuse() {
    let dir = tempfile::tempdir().unwrap();
    let r = run(&["mount", "image.vhdx", dir.path().to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("auto-parse --vhdx"), "{}", r.stdout);
    let r = run(&[
        "auto-parse",
        "-d",
        dir.path().to_str().unwrap(),
        "--vhdx",
        "-o",
        dir.path().join("out").to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("No VHDX disk images"), "{}", r.stdout);
}

#[cfg(target_os = "linux")]
#[test]
fn mount_input_errors() {
    let dir = tempfile::tempdir().unwrap();
    let image = dir.path().join("errors.vhdx");
    sample_vhdx(&image);
    let point = dir.path().join("mnt");
    std::fs::create_dir(&point).unwrap();
    let r = run(&["mount", "missing.vhdx", point.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("The image does not exist"),
        "{}",
        r.stdout
    );
    let r = run(&["mount", image.to_str().unwrap(), "missing-dir", "-K"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("The mount point is not a directory"),
        "{}",
        r.stdout
    );
    let r = run(&[
        "mount",
        "test_files/ual/README.md",
        point.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("Could not open test_files/ual/README.md"),
        "{}",
        r.stdout
    );
}

#[test]
fn auto_parse_vhdx_parses_the_systems_inside_every_image() {
    let evidence = tempfile::tempdir().unwrap();
    // no image: nothing to do
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "--vhdx",
        "-o",
        evidence.path().join("unused").to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout
            .contains("No VHDX disk images (*.vhdx) were found in"),
        "{}",
        r.stdout
    );
    sample_vhdx(&evidence.path().join("first-auto.vhdx"));
    std::fs::create_dir(evidence.path().join("sub")).unwrap();
    sample_vhdx(&evidence.path().join("sub/second-auto.vhdx"));
    std::fs::write(evidence.path().join("broken-auto.vhdx"), b"not a VHDX").unwrap();
    let partial = evidence.path().join("partial-auto.vhdx");
    sample_vhdx(&partial);
    add_unreadable_partition_to_sample(&partial);
    let root = tempfile::tempdir().unwrap();
    let output = root.path().join("results");
    let timeline = root.path().join("timeline.jsonl");
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "--vhdx",
        "-o",
        output.to_str().unwrap(),
        "-j",
        timeline.to_str().unwrap(),
        "-T",
        "2",
        "-K",
        "-q",
    ]);
    // the broken image is a failure; the others are parsed
    assert_eq!(r.code, 1, "{}\n{}", r.stdout, r.stderr);
    let stdout = r.stdout.replace('\\', "/");
    for line in [
        "VHDX images: 4\n",
        "Could not read ./broken-auto.vhdx: ",
        "Incomplete VHDX input:",
        "Could not read ./partial-auto.vhdx: volume 2 at byte 32768: ",
        "  ./first-auto.vhdx: reading natively (1 NTFS volume)\n",
        "  ./sub/second-auto.vhdx: reading natively (1 NTFS volume)\n",
        "Windows systems: 3\n",
        "  first-auto: ./first-auto/C (./first-auto.vhdx)\n",
        "  sub_second-auto: ./sub_second-auto/C (./sub/second-auto.vhdx)\n",
        "Failed: 0\n",
    ] {
        assert!(stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(output.join("manifest.json")).unwrap()).unwrap();
    let evidence_name = evidence.path().file_name().unwrap().to_str().unwrap();
    assert_eq!(manifest["EvidenceDirectory"], evidence_name);
    let hosts = manifest["Hosts"].as_array().unwrap();
    assert_eq!(hosts.len(), 3);
    assert_eq!(hosts[0]["Image"], "./first-auto.vhdx");
    assert_eq!(
        hosts[0]["EvidenceRoot"]
            .as_str()
            .unwrap()
            .replace('\\', "/"),
        "./first-auto/C"
    );
    assert_eq!(hosts[1]["Image"], "./partial-auto.vhdx");
    assert_eq!(hosts[2]["Image"], "./sub/second-auto.vhdx");
    let errors = manifest["ImageErrors"].as_array().unwrap();
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(errors.iter().any(|e| e["Image"] == "./broken-auto.vhdx"));
    assert!(errors.iter().any(|e| e["Image"] == "./partial-auto.vhdx"));
    let (_, rows) = csv_rows(&output.join("manifest.csv"));
    assert_eq!(
        rows.iter()
            .filter(|row| row["Artifact"] == "disk-image" && row["Status"] == "failed")
            .count(),
        2
    );
    // the files inside were parsed
    for host in ["first-auto", "partial-auto", "sub_second-auto"] {
        let (_, rows) = csv_rows(&output.join(host).join("ProcessExecution/Prefetch.csv"));
        assert_eq!(rows.len(), 1, "{host}");
        assert!(
            output
                .join(host)
                .join("LateralMovement/UAL-Roles.csv")
                .is_file()
        );
    }
    let records = jsonl_records(&std::fs::read_to_string(&timeline).unwrap());
    assert!(
        records
            .iter()
            .any(|r| r["Artifact"] == "prefetch" && r["Comp"] == "first-auto")
    );
    // nothing names the mount points, and nothing stays mounted
    assert_no_absolute_paths(&output, &r.stdout, &[evidence.path(), root.path()]);
    for file in walk_files(&output) {
        let text = String::from_utf8_lossy(&std::fs::read(&file).unwrap()).into_owned();
        assert!(
            !text.contains("juntei-vhdx-") && !text.contains("juntei-images-"),
            "{}",
            file.display()
        );
    }
}

#[test]
fn auto_parse_vhdx_bare_filename_keeps_original_directory_provenance() {
    let evidence = tempfile::tempdir().unwrap();
    // A backslash is part of a filename on Unix, never a directory separator.
    let filename = if cfg!(unix) {
        r"single\image.vhdx"
    } else {
        "single.vhdx"
    };
    let image = evidence.path().join(filename);
    sample_vhdx(&image);
    let original = std::fs::read(&image).unwrap();
    let permissions = std::fs::metadata(&image).unwrap().permissions();
    let mut read_only = permissions.clone();
    read_only.set_readonly(true);
    std::fs::set_permissions(&image, read_only).unwrap();
    let output_root = tempfile::tempdir().unwrap();
    let output = output_root.path().join("parsed");
    let result = Command::new(env!("CARGO_BIN_EXE_juntei"))
        .args([
            "auto-parse",
            "--vhdx",
            "-d",
            filename,
            "-o",
            output.to_str().unwrap(),
            "-T",
            "2",
            "-K",
            "-q",
        ])
        .current_dir(evidence.path())
        // Native parsing needs no fusermount, system mount utility or helper.
        .env("PATH", evidence.path())
        .output()
        .unwrap();
    assert_eq!(std::fs::read(&image).unwrap(), original);
    std::fs::set_permissions(&image, permissions).unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(output.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        manifest["EvidenceDirectory"],
        evidence.path().file_name().unwrap().to_str().unwrap()
    );
    assert_eq!(manifest["Hosts"][0]["Image"], format!("./{filename}"));
}

#[test]
fn auto_parse_vhdx_refuses_input_as_timeline_even_through_a_hard_link() {
    let dir = tempfile::tempdir().unwrap();
    let image = dir.path().join("input.vhdx");
    sample_vhdx(&image);
    let original = std::fs::read(&image).unwrap();
    let alias = dir.path().join("alias.jsonl");
    std::fs::hard_link(&image, &alias).unwrap();
    for target in [&image, &alias] {
        let output = dir.path().join("output");
        let r = run(&[
            "auto-parse",
            "--vhdx",
            "-d",
            image.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-j",
            target.to_str().unwrap(),
            "-C",
            "-K",
            "-q",
        ]);
        assert_eq!(r.code, 1, "{}", r.stdout);
        assert!(
            r.stdout.contains("overlaps an input image or segment"),
            "{}",
            r.stdout
        );
        assert!(!output.exists());
        assert_eq!(std::fs::read(&image).unwrap(), original);
    }
}

#[cfg(unix)]
#[test]
fn auto_parse_vhdx_keeps_image_provenance_through_a_temporary_directory_alias() {
    let dir = tempfile::tempdir().unwrap();
    let actual = dir.path().join("actual");
    let alias = dir.path().join("alias");
    std::fs::create_dir(&actual).unwrap();
    std::os::unix::fs::symlink(&actual, &alias).unwrap();
    let image = dir.path().join("alias-case.vhdx");
    sample_vhdx(&image);
    let output = dir.path().join("output");
    let r = Command::new(env!("CARGO_BIN_EXE_juntei"))
        .args([
            "auto-parse",
            "--vhdx",
            "--temp-dir",
            alias.to_str().unwrap(),
            "-d",
            image.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-T",
            "2",
            "-K",
            "-q",
        ])
        .env("TMPDIR", &alias)
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stdout));
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(output.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["Hosts"][0]["Image"], "./alias-case.vhdx");
    assert_no_absolute_paths(&output, &String::from_utf8_lossy(&r.stdout), &[dir.path()]);
    assert_eq!(std::fs::read_dir(&actual).unwrap().count(), 0);
}

#[cfg(unix)]
#[test]
fn auto_parse_vhdx_skips_an_unreadable_subdirectory_but_not_the_input_root() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let evidence = dir.path().join("evidence");
    std::fs::create_dir(&evidence).unwrap();
    sample_vhdx(&evidence.join("readable.vhdx"));
    let locked = evidence.join("locked");
    std::fs::create_dir(&locked).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o0)).unwrap();
    if std::fs::read_dir(&locked).is_ok() {
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o700)).unwrap();
        return; // Permission checks do not apply to a privileged test process.
    }
    let output = dir.path().join("parsed");
    let r = run(&[
        "auto-parse",
        "--vhdx",
        "-d",
        evidence.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-T",
        "2",
        "-K",
        "-q",
    ]);
    let blocked_root = run(&[
        "auto-parse",
        "--vhdx",
        "-d",
        locked.to_str().unwrap(),
        "-o",
        dir.path().join("blocked").to_str().unwrap(),
        "-K",
        "-q",
    ]);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(r.code, 0, "{}\n{}", r.stdout, r.stderr);
    assert!(
        output
            .join("readable/ProcessExecution/Prefetch.csv")
            .is_file()
    );
    assert_eq!(blocked_root.code, 1);
    assert!(blocked_root.stdout.contains("Could not search"));
}

#[test]
fn auto_parse_vhdx_uses_the_output_filesystem_unless_a_staging_directory_is_given() {
    let dir = tempfile::tempdir().unwrap();
    let image = dir.path().join("image.vhdx");
    sample_vhdx(&image);
    let unused_system_temp = dir.path().join("not-a-directory");
    std::fs::write(&unused_system_temp, b"not a directory").unwrap();
    let staging = dir.path().join("staging");
    std::fs::create_dir(&staging).unwrap();
    for explicit in [false, true] {
        let output = dir
            .path()
            .join(if explicit { "explicit" } else { "default" });
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_juntei"));
        cmd.args([
            "auto-parse",
            "--vhdx",
            "-d",
            image.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-T",
            "2",
            "-K",
            "-q",
        ])
        .env("TMPDIR", &unused_system_temp)
        .env("TMP", &unused_system_temp)
        .env("TEMP", &unused_system_temp);
        if explicit {
            cmd.args(["--temp-dir", staging.to_str().unwrap()]);
        }
        let r = cmd.output().unwrap();
        assert!(
            r.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&r.stdout),
            String::from_utf8_lossy(&r.stderr)
        );
        assert!(output.join("image/ProcessExecution/Prefetch.csv").is_file());
        assert_eq!(std::fs::read_dir(&staging).unwrap().count(), 0);
        assert!(!std::fs::read_dir(dir.path()).unwrap().any(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".juntei-image-control-")
        }));
    }
    let invalid = run(&[
        "auto-parse",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        dir.path().join("invalid").to_str().unwrap(),
        "--temp-dir",
        staging.to_str().unwrap(),
        "-K",
        "-q",
    ]);
    assert_eq!(invalid.code, 1);
    assert!(invalid.stdout.contains("--temp-dir requires --vhdx"));
}

#[test]
fn auto_parse_native_images_reject_staging_inside_output_before_opening_images() {
    for (flag, extension) in [("--vhdx", "vhdx"), ("--e01", "E01")] {
        let dir = tempfile::tempdir().unwrap();
        let image = dir.path().join(format!("invalid.{extension}"));
        // An early validation failure must take priority over an unreadable image.
        std::fs::write(&image, b"not a VHDX image").unwrap();
        let output = dir.path().join("output");
        std::fs::create_dir(&output).unwrap();
        let nested = output.join("staging");
        std::fs::create_dir(&nested).unwrap();
        let marker = output.join("keep.txt");
        std::fs::write(&marker, b"existing output").unwrap();
        let staging_paths = vec![output.clone(), nested, output.join("not-created")];
        #[cfg(unix)]
        let staging_paths = {
            let mut paths = staging_paths;
            let alias = dir.path().join("alias");
            std::os::unix::fs::symlink(&output, &alias).unwrap();
            paths.push(alias);
            paths
        };
        for staging in staging_paths {
            let result = run(&[
                "auto-parse",
                flag,
                "-d",
                image.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
                "--temp-dir",
                staging.to_str().unwrap(),
                "--clobber",
                "-K",
                "-q",
            ]);
            assert_eq!(result.code, 1, "{}\n{}", result.stdout, result.stderr);
            let messages = format!("{}\n{}", result.stdout, result.stderr);
            assert!(messages.contains("--temp-dir must be outside the output directory"));
            assert!(!messages.contains("VHDX images:"), "{messages}");
            assert!(!messages.contains("E01 images:"), "{messages}");
            assert!(
                !messages.contains(dir.path().to_str().unwrap()),
                "{messages}"
            );
            assert_eq!(std::fs::read(&marker).unwrap(), b"existing output");
            assert!(!output.join("manifest.json").exists());
            assert!(!std::fs::read_dir(dir.path()).unwrap().any(|entry| {
                let name = entry.unwrap().file_name();
                let name = name.to_string_lossy();
                name.starts_with("juntei-vhdx-") || name.starts_with(".juntei-image-control-")
            }));
        }
    }
}

#[cfg(unix)]
#[test]
fn auto_parse_vhdx_signals_clean_staged_evidence_during_copying_and_parsing() {
    use std::time::{Duration, Instant};
    for (signal, parsing) in ["-INT", "-TERM", "-HUP"]
        .into_iter()
        .flat_map(|signal| [false, true].map(|parsing| (signal, parsing)))
    {
        let dir = tempfile::tempdir().unwrap();
        let evidence = dir.path().join("evidence");
        let staging = dir.path().join("staging");
        std::fs::create_dir(&evidence).unwrap();
        std::fs::create_dir(&staging).unwrap();
        let image = evidence.join("00.vhdx");
        sample_vhdx(&image);
        let original = std::fs::read(&image).unwrap();
        for n in 1..24 {
            std::fs::hard_link(&image, evidence.join(format!("{n:02}.vhdx"))).unwrap();
        }
        let output = dir.path().join("output");
        let mut child = Command::new(env!("CARGO_BIN_EXE_juntei"))
            .args([
                "auto-parse",
                "--vhdx",
                "-d",
                evidence.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
                "--temp-dir",
                staging.to_str().unwrap(),
                "-T",
                "2",
                "-K",
                "-q",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let start = Instant::now();
        loop {
            let ready = if parsing {
                output.exists()
                    && walk_files(&output)
                        .iter()
                        .any(|p| p.extension().is_some_and(|e| e == "csv"))
            } else {
                !walk_files(&staging).is_empty()
            };
            if ready {
                break;
            }
            if start.elapsed() > Duration::from_secs(30) || child.try_wait().unwrap().is_some() {
                let _ = child.kill();
                let result = child.wait_with_output().unwrap();
                panic!(
                    "Image run ended before the interrupt test: {}\n{}",
                    String::from_utf8_lossy(&result.stdout),
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        // Signal only the parent: it must also stop and reap its parser processes.
        assert!(
            Command::new("kill")
                .args([signal, &child.id().to_string()])
                .status()
                .unwrap()
                .success()
        );
        let interrupted = Instant::now();
        while child.try_wait().unwrap().is_none() {
            if interrupted.elapsed() > Duration::from_secs(15) {
                let _ = child.kill();
                let _ = child.wait();
                panic!("{signal} did not stop the image parser");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let result = child.wait_with_output().unwrap();
        assert_eq!(
            result.status.code(),
            Some(130),
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        // SIGTERM and SIGHUP are not Ctrl+C, so the message must not name it.
        let messages = format!(
            "{}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(messages.contains("Cancelled."), "{messages}");
        assert!(!messages.contains("Ctrl+C"), "{messages}");
        assert_eq!(std::fs::read_dir(&staging).unwrap().count(), 0);
        assert!(!std::fs::read_dir(dir.path()).unwrap().any(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".juntei-image-control-")
        }));
        assert_eq!(std::fs::read(&image).unwrap(), original);
    }
}

#[cfg(target_os = "linux")]
#[test]
fn auto_parse_vhdx_retains_readable_results_when_the_staging_filesystem_fills() {
    let dir = tempfile::tempdir().unwrap();
    let evidence = dir.path().join("evidence");
    let staging = dir.path().join("small");
    std::fs::create_dir(&evidence).unwrap();
    std::fs::create_dir(&staging).unwrap();
    let image = evidence.join("00.vhdx");
    sample_vhdx(&image);
    for n in 1..12 {
        std::fs::hard_link(&image, evidence.join(format!("{n:02}.vhdx"))).unwrap();
    }
    let output = dir.path().join("output");
    let result = Command::new("unshare").args(["--user", "--map-root-user", "--mount", "sh", "-c",
        "mount -t tmpfs -o size=12m tmpfs \"$1\" || exit 77; TMPDIR=\"$1\" \"$2\" auto-parse --vhdx -d \"$3\" -o \"$4\" --temp-dir \"$1\" -T 2 -K -q",
        "_", staging.to_str().unwrap(), env!("CARGO_BIN_EXE_juntei"), evidence.to_str().unwrap(), output.to_str().unwrap()]).output();
    let Ok(result) = result else { return }; // Namespace tools are optional on developer systems.
    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);
    if result.status.code() == Some(77) || stderr.contains("unshare:") {
        eprintln!("Bounded-filesystem test unavailable: {stderr}");
        return;
    }
    assert_eq!(result.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stdout.contains("No space left"), "{stdout}");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(output.join("manifest.json")).unwrap()).unwrap();
    let errors = manifest["ImageErrors"].as_array().unwrap();
    assert_eq!(
        errors
            .iter()
            .filter(|e| e["Error"]
                .as_str()
                .is_some_and(|s| s.contains("No space left")))
            .count(),
        1,
        "{errors:?}"
    );
    assert!(
        walk_files(&output)
            .iter()
            .any(|p| p.file_name().is_some_and(|name| name == "Prefetch.csv"))
    );
    assert_eq!(std::fs::read_dir(&staging).unwrap().count(), 0);
}

#[test]
fn staged_shell_files_use_original_source_times_in_every_format() {
    let dir = tempfile::tempdir().unwrap();
    for (command, fixture) in [
        ("lnk", "test_files/lnk/nero_infotool.lnk"),
        (
            "jump-lists",
            "test_files/jump_lists/f01b4d95cf55d32a.automaticDestinations-ms",
        ),
    ] {
        let path = dir.path().join(Path::new(fixture).file_name().unwrap());
        std::fs::copy(repo(fixture), &path).unwrap();
        let mut files = serde_json::Map::new();
        files.insert(
            path.canonicalize().unwrap().to_string_lossy().into_owned(),
            json!([
                132_223_104_000_000_000u64,
                132_223_104_010_000_000u64,
                132_223_104_020_000_000u64
            ]),
        );
        let snapshot = dir.path().join("metadata.json");
        std::fs::write(
            &snapshot,
            serde_json::to_vec(&json!({"Files": files})).unwrap(),
        )
        .unwrap();
        for kind in ["csv", "json", "jsonl"] {
            let output = dir.path().join(format!("{command}-{kind}"));
            let r = run(&[
                "parse-file",
                command,
                "-f",
                path.to_str().unwrap(),
                "--image-source-metadata",
                snapshot.to_str().unwrap(),
                "-t",
                kind,
                "-o",
                output.to_str().unwrap(),
                "-K",
                "-q",
            ]);
            assert_eq!(r.code, 0, "{}\n{}", r.stdout, r.stderr);
            let record: Value = if kind == "csv" {
                let csv = if command == "jump-lists" {
                    output.join("AutomaticDestinations.csv")
                } else {
                    output.clone()
                };
                let (_, rows) = csv_rows(&csv);
                serde_json::to_value(&rows[0]).unwrap()
            } else {
                let text = std::fs::read_to_string(&output).unwrap();
                let value: Value = serde_json::from_str(&text).unwrap();
                if value.is_array() {
                    value[0].clone()
                } else {
                    value
                }
            };
            assert_eq!(record["SourceCreated"], "2020-01-01T00:00:00Z");
            assert_eq!(record["SourceModified"], "2020-01-01T00:00:01Z");
            assert_eq!(record["SourceAccessed"], "2020-01-01T00:00:02Z");
        }
        // Missing metadata must be an error, not a fallback to the cache's times.
        std::fs::write(&snapshot, br#"{"Files":{}}"#).unwrap();
        let r = run(&[
            "parse-file",
            command,
            "-f",
            path.to_str().unwrap(),
            "--image-source-metadata",
            snapshot.to_str().unwrap(),
            "-t",
            "jsonl",
        ]);
        assert_eq!(r.code, 1);
        assert!(
            r.stderr.contains("Original image metadata is missing")
                || r.stdout.contains("Original image metadata is missing")
        );
    }
}

/// The sample E01 image (test_files/e01, two segments) copied to `dir` as `<stem>.E01` and
/// `<stem>.E02`; returns the first segment.
fn sample_e01(dir: &Path, stem: &str) -> PathBuf {
    for segment in ["E01", "E02"] {
        std::fs::copy(
            repo(&format!("test_files/e01/sample.{segment}")),
            dir.join(format!("{stem}.{segment}")),
        )
        .unwrap();
    }
    dir.join(format!("{stem}.E01"))
}

#[cfg(target_os = "linux")]
#[test]
fn mount_serves_an_e01_image_from_its_segments() {
    let dir = tempfile::tempdir().unwrap();
    let image = sample_e01(dir.path(), "mount-test");
    let point = dir.path().join("mnt");
    std::fs::create_dir(&point).unwrap();
    // a logical evidence file holds files, not a disk
    let logical = dir.path().join("files.L01");
    std::fs::write(&logical, b"LVF\x09\x0d\x0a\xff\x00").unwrap();
    let r = run(&[
        "mount",
        logical.to_str().unwrap(),
        point.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout
            .contains("it is a logical evidence file (L01 or Lx01), which holds files, not a disk"),
        "{}",
        r.stdout
    );
    if !fuse_available() {
        return;
    }
    let child = Command::new(env!("CARGO_BIN_EXE_juntei"))
        .args([
            "mount",
            image.to_str().unwrap(),
            point.to_str().unwrap(),
            "-K",
            "-q",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let data = point.join("C/Users/sample/Data");
    for _ in 0..100 {
        if data.is_dir() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    assert!(data.is_dir(), "not mounted");
    assert!(mounted_from("mount-test.E01"));
    let sha256 = |path: &Path| {
        use sha2::{Digest, Sha256};
        Sha256::digest(std::fs::read(path).unwrap())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    // the same files as in the VHDX
    assert_eq!(
        std::fs::read(data.join("small.txt")).unwrap(),
        b"resident content\n"
    );
    assert_eq!(
        sha256(&data.join("fragmented.bin")),
        "dad53fd22d62e1acbf48ababc172c8eb3ffb45486b261fb960a2990199a8e459"
    );
    assert_eq!(
        sha256(&data.join("compressed/text.log")),
        "cc3c52b090233f44ecfc2dbd90adc6c787e071872ca488fa96eae24af8162f98"
    );
    assert!(std::fs::write(data.join("new.txt"), b"x").is_err());
    let killed = Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(killed.success());
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{stdout}");
    assert!(stdout.contains("Format: E01 (2 segments)\n"), "{stdout}");
    assert!(stdout.contains("Disk size: 33.2 MiB\n"), "{stdout}");
    assert!(stdout.contains("NTFS volumes: 1 ("), "{stdout}");
    assert!(!mounted_from("mount-test.E01"));
    assert!(!data.exists());
}

#[test]
fn auto_parse_e01_parses_the_systems_inside_every_image() {
    let evidence = tempfile::tempdir().unwrap();
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "--e01",
        "-o",
        evidence.path().join("unused").to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout
            .contains("No E01 disk images (*.E01, *.Ex01) were found in"),
        "{}",
        r.stdout
    );
    sample_e01(evidence.path(), "first-e01");
    std::fs::create_dir(evidence.path().join("sub")).unwrap();
    sample_e01(&evidence.path().join("sub"), "second-e01");
    std::fs::write(evidence.path().join("broken-e01.E01"), b"not an E01").unwrap();
    sample_vhdx(&evidence.path().join("third.vhdx"));
    let root = tempfile::tempdir().unwrap();

    // --e01: the E01 images only, each once (not their second segments)
    let output = root.path().join("e01");
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "--e01",
        "-o",
        output.to_str().unwrap(),
        "-T",
        "2",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 1, "{}\n{}", r.stdout, r.stderr);
    let stdout = r.stdout.replace('\\', "/");
    for line in [
        "E01 images: 3\n",
        "Could not read ./broken-e01.E01: it is not a VHDX or E01 disk image",
        "  ./first-e01.E01: reading natively (1 NTFS volume)\n",
        "  ./sub/second-e01.E01: reading natively (1 NTFS volume)\n",
        "Windows systems: 2\n",
        "  first-e01: ./first-e01/C (./first-e01.E01)\n",
        "Failed: 0\n",
    ] {
        assert!(stdout.contains(line), "{line}\n{}", r.stdout);
    }
    assert!(!r.stdout.contains("third.vhdx"), "{}", r.stdout);
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(output.join("manifest.json")).unwrap()).unwrap();
    let hosts = manifest["Hosts"].as_array().unwrap();
    assert_eq!(hosts.len(), 2);
    assert_eq!(hosts[0]["Image"], "./first-e01.E01");
    assert_eq!(hosts[1]["Image"], "./sub/second-e01.E01");
    for host in ["first-e01", "sub_second-e01"] {
        let (_, rows) = csv_rows(&output.join(host).join("ProcessExecution/Prefetch.csv"));
        assert_eq!(rows.len(), 1, "{host}");
    }
    assert_no_absolute_paths(&output, &r.stdout, &[evidence.path(), root.path()]);

    // --vhdx and --e01 together: both kinds
    std::fs::remove_file(evidence.path().join("broken-e01.E01")).unwrap();
    let output = root.path().join("both");
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "--vhdx",
        "--e01",
        "-o",
        output.to_str().unwrap(),
        "-T",
        "2",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "{}\n{}", r.stdout, r.stderr);
    assert!(r.stdout.contains("VHDX or E01 images: 3\n"), "{}", r.stdout);
    assert!(r.stdout.contains("Windows systems: 3\n"), "{}", r.stdout);
}

/// A damaged E01 image is read as far as it can be checked: the damage is reported, what is
/// readable is parsed, and the run fails, as for an image with an unreadable volume.
#[test]
fn auto_parse_e01_reads_a_damaged_image_and_reports_it() {
    let evidence = tempfile::tempdir().unwrap();
    sample_e01(evidence.path(), "damaged");
    // cut the second segment short: it holds the random bytes after the disk's partition
    let second = evidence.path().join("damaged.E02");
    let bytes = std::fs::read(&second).unwrap();
    std::fs::write(&second, &bytes[..400_000]).unwrap();
    let root = tempfile::tempdir().unwrap();
    let output = root.path().join("results");
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "--e01",
        "-o",
        output.to_str().unwrap(),
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 1, "{}\n{}", r.stdout, r.stderr);
    let stdout = r.stdout.replace('\\', "/");
    for line in [
        "Could not read ./damaged.E01: the image is incomplete: its acquisition did not finish\n",
        "Could not read ./damaged.E01: EWF1 section data exceeds file",
        "  ./damaged.E01: reading natively (1 NTFS volume)\n",
        "  damaged: ./damaged/C (./damaged.E01)\n",
        "Failed: 0\n",
    ] {
        assert!(stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let (_, rows) = csv_rows(&output.join("damaged/ProcessExecution/Prefetch.csv"));
    assert_eq!(rows.len(), 1);
}

#[test]
fn auto_parse_e01_protects_secondary_segments_even_when_damaged() {
    let dir = tempfile::tempdir().unwrap();
    let image = sample_e01(dir.path(), "protected");
    let second = dir.path().join("protected.E02");
    let mut damaged = std::fs::read(&second).unwrap();
    damaged[..16].fill(0);
    std::fs::write(&second, &damaged).unwrap();
    let alias = dir.path().join("timeline.jsonl");
    std::fs::hard_link(&second, &alias).unwrap();
    let original = std::fs::read(&image).unwrap();
    for target in [&second, &alias, &image] {
        let output = dir.path().join("output");
        let r = run(&[
            "auto-parse",
            "--e01",
            "-d",
            image.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-j",
            target.to_str().unwrap(),
            "-C",
            "-K",
            "-q",
        ]);
        assert_eq!(r.code, 1, "{}", r.stdout);
        assert!(
            r.stdout.contains("overlaps an input image or segment"),
            "{}",
            r.stdout
        );
        assert!(!output.exists());
        assert_eq!(std::fs::read(&second).unwrap(), damaged);
        assert_eq!(std::fs::read(&image).unwrap(), original);
    }
}

#[test]
fn auto_parse_e01_and_vhdx_selection_checks_the_actual_signature() {
    let dir = tempfile::tempdir().unwrap();
    let wrong_e01 = dir.path().join("spoof.E01");
    sample_vhdx(&wrong_e01);
    let wrong_vhdx = dir.path().join("spoof.vhdx");
    std::fs::copy(repo("test_files/e01/sample.E01"), &wrong_vhdx).unwrap();
    for (flag, path, message) in [
        ("--e01", &wrong_e01, "Expected E01, found VHDX"),
        ("--vhdx", &wrong_vhdx, "Expected VHDX, found E01"),
    ] {
        let output = dir.path().join("output");
        let r = run(&[
            "auto-parse",
            flag,
            "-d",
            path.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-K",
            "-q",
        ]);
        assert_eq!(r.code, 1, "{}", r.stdout);
        assert!(r.stdout.contains(message), "{}", r.stdout);
        assert!(!output.exists());
    }
}

#[test]
fn auto_parse_e01_rejects_logical_evidence_and_continuation_inputs() {
    let dir = tempfile::tempdir().unwrap();
    let logical = dir.path().join("logical.E01");
    std::fs::write(&logical, b"LVF\x09\x0d\x0a\xff\x00").unwrap();
    let first = sample_e01(dir.path(), "split");
    for (input, expected) in [
        (logical, "logical evidence file"),
        (first.with_extension("E02"), "first segment"),
    ] {
        let output = dir.path().join("output");
        let r = run(&[
            "auto-parse",
            "--e01",
            "-d",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-K",
            "-q",
        ]);
        assert_eq!(r.code, 1, "{}\n{}", r.stdout, r.stderr);
        assert!(r.stdout.contains(expected), "{}", r.stdout);
        assert!(!output.exists());
    }
}

#[test]
fn auto_parse_images_refuse_outputs_containing_source_segments() {
    let dir = tempfile::tempdir().unwrap();
    let image = sample_e01(dir.path(), "protected");
    let before = std::fs::read(&image).unwrap();
    let r = run(&[
        "auto-parse",
        "--e01",
        "-d",
        image.to_str().unwrap(),
        "-o",
        dir.path().to_str().unwrap(),
        "-C",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 1, "{}\n{}", r.stdout, r.stderr);
    assert!(
        r.stdout
            .contains("output directory contains an input image or segment"),
        "{}",
        r.stdout
    );
    assert_eq!(std::fs::read(&image).unwrap(), before);
    assert!(!dir.path().join("manifest.json").exists());
}

#[test]
fn auto_parse_e01_native_inputs_work_without_helpers_in_csv_and_jsonl() {
    let dir = tempfile::tempdir().unwrap();
    let staging = dir.path().join("staging");
    std::fs::create_dir(&staging).unwrap();
    let first = sample_e01(dir.path(), "split");
    let direct = run(&[
        "parse-file",
        "prefetch",
        "-f",
        PREFETCH_CMD10,
        "-t",
        "jsonl",
    ]);
    assert_eq!(direct.code, 0, "{}", direct.stderr);
    let expected: Value = serde_json::from_str(&direct.stdout).unwrap();
    for name in ["sample.Ex01", "volume.Ex01"] {
        std::fs::copy(
            repo(&format!("test_files/e01/{name}")),
            dir.path().join(name),
        )
        .unwrap();
    }
    let sources: Vec<_> = ["split.E01", "split.E02", "sample.Ex01", "volume.Ex01"]
        .into_iter()
        .map(|name| {
            let path = dir.path().join(name);
            let permissions = path.metadata().unwrap().permissions();
            let bytes = std::fs::read(&path).unwrap();
            let mut readonly = permissions.clone();
            readonly.set_readonly(true);
            std::fs::set_permissions(&path, readonly).unwrap();
            (path, permissions, bytes)
        })
        .collect();
    for image in [
        &first,
        &dir.path().join("sample.Ex01"),
        &dir.path().join("volume.Ex01"),
    ] {
        for kind in ["csv", "json", "jsonl"] {
            let output = dir.path().join(format!(
                "{}-{kind}",
                image.file_name().unwrap().to_string_lossy()
            ));
            let timeline = output.with_extension(format!("{kind}.timeline.jsonl"));
            let r = Command::new(env!("CARGO_BIN_EXE_juntei"))
                .args([
                    "auto-parse",
                    "--e01",
                    "--temp-dir",
                    staging.to_str().unwrap(),
                    "-d",
                    image.to_str().unwrap(),
                    "-o",
                    output.to_str().unwrap(),
                    "-t",
                    kind,
                    "-j",
                    timeline.to_str().unwrap(),
                    "-T",
                    "2",
                    "-K",
                    "-q",
                ])
                .env("PATH", dir.path())
                .output()
                .unwrap();
            assert!(
                r.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&r.stdout),
                String::from_utf8_lossy(&r.stderr)
            );
            let manifest: Value =
                serde_json::from_slice(&std::fs::read(output.join("manifest.json")).unwrap())
                    .unwrap();
            assert_eq!(manifest["Hosts"].as_array().unwrap().len(), 1);
            assert_eq!(
                manifest["Hosts"][0]["Image"],
                format!("./{}", image.file_name().unwrap().to_string_lossy())
            );
            let records = jsonl_records(&std::fs::read_to_string(&timeline).unwrap());
            let actual = records
                .iter()
                .find(|r| r["Artifact"] == "prefetch")
                .unwrap();
            for (key, value) in expected.as_object().unwrap() {
                if key != "SourceFile" {
                    assert_eq!(&actual[key], value, "{key}: {} ({kind})", image.display());
                }
            }
            assert_no_absolute_paths(&output, &String::from_utf8_lossy(&r.stdout), &[dir.path()]);
            assert_eq!(std::fs::read_dir(&staging).unwrap().count(), 0);
        }
    }
    for (path, permissions, bytes) in sources {
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        std::fs::set_permissions(path, permissions).unwrap();
    }
}

#[test]
fn auto_parse_e01_resolves_a_case_alias_of_the_first_segment() {
    let dir = tempfile::tempdir().unwrap();
    sample_e01(dir.path(), "MiXeD");
    let alias = dir.path().join("mixed.e01");
    if !alias.is_file() {
        return;
    } // Relevant only on a case-insensitive filesystem.
    let output = dir.path().join("output");
    let r = run(&[
        "auto-parse",
        "--e01",
        "-d",
        alias.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-T",
        "2",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "{}\n{}", r.stdout, r.stderr);
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(output.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["Hosts"][0]["Image"], "./mixed.e01");
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
            "TargetID-AbsolutePath",
            r"This PC\C:\Program Files (x86)\Nero\Nero 9\Nero InfoTool\InfoTool.exe",
        ),
        ("Args", "-ScParameter=30002  "),
        ("TargetMFT-EntryNumber", "81349"),
        ("TargetMFT-SequenceNumber", "1"),
        ("MachineID", "win-3qmdjgkf3g9"),
        ("MachineMAC-Address", "70:5a:b6:16:a5:f3"),
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
    assert_eq!(padded["Args"].chars().count(), 1431);
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
    assert_eq!(v["Tracker"]["MachineID"], "win-3qmdjgkf3g9");
    assert_eq!(
        v["Tracker"]["VolumeDroidBirth"],
        "c989f572-4deb-4c64-b0d1-dc4959bbff28"
    );
    assert_eq!(v["TargetMFT-Entry"], 81_349);
    assert_eq!(v["Header"]["ShowCommand"], "SW_SHOWNORMAL");
    let items = v["TargetItems"].as_array().unwrap();
    assert_eq!(items[0]["Name"], "This PC");
    assert_eq!(items.last().unwrap()["Name"], "InfoTool.exe");
    assert_eq!(v["ExtraBlocks"][0], "IconEnvironmentDataBlock");
    assert!(
        v["IconEnvironment"]
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
            d["SourceFile"]
                .as_str()
                .unwrap()
                .ends_with("padded_cli_arguments.lnk")
        })
        .unwrap();
    assert_eq!(padded["Problems"].as_array().unwrap().len(), 2);
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
    assert_eq!(v["Header"]["TargetCreated"], "");
    assert_eq!(v["Header"]["TargetModified"], "2009-06-05T20:13:20Z");
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
            "AppID",
            "AppID-Description",
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
        ("AppID", "f01b4d95cf55d32a"),
        ("AppID-Description", "Windows Explorer Windows 8.1"),
        ("DestListVersion", "4"),
        ("LastUsedEntryNumber", "9"),
        ("EntryNumber", "7"),
        ("CreationTime", "2022-02-05T19:30:37.504034700Z"),
        ("LastModified", "2022-02-05T19:39:12.024818800Z"),
        ("Hostname", "desktop-gnccvrk"),
        ("MAC-Address", "ec:63:d7:72:83:36"),
        ("Path", r"C:\Users\AndrewRathbun\Desktop\Targets"),
        ("InteractionCount", "1"),
        ("PinStatus", "False"),
        ("FileDroid", "1884d55b-86ba-11ec-9e53-ec63d7728336"),
        ("VolumeBirthDroid", "a0eda9fe-dbef-4d91-bf7a-eabd0971f8df"),
        ("TargetCreated", "2022-02-05T19:39:11.619072400Z"),
        ("LocalPath", r"C:\Users\AndrewRathbun\Desktop\Targets"),
        (
            "TargetID-AbsolutePath",
            r"This PC\C:\Users\AndrewRathbun\Desktop\Targets",
        ),
        ("TargetMFT-EntryNumber", "105479"),
        ("TargetMFT-SequenceNumber", "2"),
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
        ("TargetMFT-EntryNumber", "87111"),
    ] {
        assert_eq!(notepad[column], value, "{column}");
    }
    // A Windows 10 version 6 list that ends inside its last sector: entry 1 is a file staged
    // for a CD burn, named as the reference names it
    let burn = row("5f7b5f1e01b83767.automaticDestinations-ms", "36");
    assert_eq!(burn["DestListVersion"], "6");
    assert_eq!(burn["TargetID-AbsolutePath"], r"This PC\D:\autorun.inf");
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
        chrome[7]["Args"],
        "--win-jumplist-action=recently-closed http://192.168.50.33:3000/"
    );
    assert_eq!(chrome[5]["Args"], "--incognito");
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
    assert_eq!(v["Type"], "automatic");
    assert_eq!(v["AppID"], "f01b4d95cf55d32a");
    assert_eq!(v["DestList"]["Version"], 4);
    assert_eq!(v["DestList"]["LastEntryNumber"], 9);
    let entries = v["Entries"].as_array().unwrap();
    assert_eq!(entries.len(), 9);
    assert_eq!(entries[3]["Stream"], "7");
    assert_eq!(entries[3]["InteractionCount"], 1);
    assert_eq!(
        entries[3]["Link"]["TargetPath"],
        r"This PC\C:\Users\AndrewRathbun\Desktop\Targets"
    );
    assert_eq!(
        entries[3]["Link"]["Tracker"]["MachineID"],
        "desktop-gnccvrk"
    );
    assert_eq!(v["Streams"].as_array().unwrap().len(), 10);
    assert_eq!(v["Problems"].as_array().unwrap().len(), 0);

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
    assert_eq!(chrome["Type"], "custom");
    assert_eq!(chrome["FormatVersion"], 2);
    let categories = chrome["Categories"].as_array().unwrap();
    assert_eq!(categories[0]["Name"], "Most visited");
    assert_eq!(categories[2]["Kind"], "tasks");
    assert_eq!(chrome["TrailingBytes"], 3250);
    assert_eq!(chrome["Leftover"][0]["Offset"], 11472);
    assert_eq!(
        chrome["Leftover"][0]["Link"]["Args"],
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
    assert_eq!(v["Entries"][0]["LastUsed"], "");
    assert_eq!(
        v["Entries"][1]["LastUsed"],
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
    let second = entries.iter().find(|e| e["ControlSet"] == 2).unwrap();
    assert_eq!(second["IsCurrent"], false);
    assert_eq!(second["CacheValueSize"], 56_256);
    assert!(second["DataSize"].as_u64().unwrap() < 56_256);
    assert_eq!(second["RegistryValueName"], "AppCompatCache");
    assert_eq!(
        (
            second["SelectCurrent"].as_u64(),
            second["SelectDefault"].as_u64(),
            second["SelectLastKnownGood"].as_u64(),
            second["SelectFailed"].as_u64()
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
    assert_eq!(xp["CacheFormat"], "WindowsXP");
    assert_eq!(xp["ControlSet"], Value::Null);
    assert_eq!(xp["Path"], r"C:\WINDOWS\system32\wscntfy.exe");
    assert_eq!(xp["LastModified"], "2008-04-14T12:00:00Z");
    assert_eq!(xp["Executed"], Value::Null);
    assert_eq!(xp["FileSize"], 13824);
    // a value on its own: its size is the file's, and no control set is known to be current
    assert_eq!(xp["CacheValueSize"], 53_392);
    assert_eq!(xp["IsCurrent"], Value::Null);
    assert_eq!(xp["RegistryPath"], Value::Null);
    assert_eq!(xp["SelectCurrent"], Value::Null);
    assert_eq!(xp["LastUpdate"], "2016-01-13T22:20:03.265625Z");

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
        let ran = entries.iter().filter(|e| e["Executed"] == true).count();
        assert_eq!(ran, executed, "{file}");
        assert_eq!(entries[0]["CacheFormat"], format, "{file}");
        assert_eq!(entries[0]["Path"], path, "{file}");
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
    assert_eq!(w11["Executed"], false);
    assert_eq!(w11["Machine"], "x64");
    assert_eq!(w11["DataSize"], 136);
    assert_eq!(w11["LastModified"], "2022-06-18T22:26:50.670515700Z");

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
        ("ThreatID", "2147700000"),
        ("Path", r"\\?\C:\Users\alice\Downloads\tool\tool.exe"),
        ("PhysicalPath", r"C:\Users\alice\Downloads\tool\tool.exe"),
        ("FileSize", "61"),
        ("CreationTime", "2023-12-19T09:29:59.0000000Z"),
        ("LastAccessTime", "2023-12-19T09:36:30.5000000Z"),
        ("LastWriteTime", "2023-12-19T09:30:00.0000000Z"),
        ("Attr", "0x20"),
        ("Owner", "S-1-5-21-1111-2222-3333-1001"),
        ("ZoneID", "3"),
        ("HostURL", "https://example.com/tool.zip"),
        ("ContentSize", "61"),
        ("SHA256", FILE_SHA256),
        (
            "Streams",
            "SecurityData, Data, AlternateData :Zone.Identifier:$DATA",
        ),
        ("ResourceID", QUARANTINED_FILE),
        ("EntryID", "80012345-0000-0000-1A2B-3C4D5E6F7081"),
        ("ScanID", "6B1E0C2D-3F4A-4B5C-8D9E-0F1A2B3C4D5E"),
        ("ExtractedFile", ""),
        ("Problems", ""),
    ] {
        assert_eq!(file[column], value, "{column}");
    }
    assert!(file["ResourceRecordFile"].ends_with(QUARANTINED_FILE));
    let service = &rows[1];
    assert_eq!(service["Streams"], "Hive");
    assert_eq!(service["ContentSize"], "4096");
    assert_eq!(service["ResourceID"], QUARANTINED_HIVE);
    assert_eq!(rows[2]["ResourceID"], "");
    let orphan = &rows[3];
    assert_eq!(orphan["ResourceID"], ORPHAN_CONTENT);
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
    assert_eq!(file["Record"], "resource");
    assert_eq!(file["ResourceType"], "file");
    assert_eq!(file["Fields"].as_array().unwrap().len(), 8);
    assert_eq!(file["Fields"][0]["Name"], "ResourceId");
    assert_eq!(file["Content"]["SHA256"], FILE_SHA256);
    assert_eq!(file["Content"]["Streams"].as_array().unwrap().len(), 3);
    assert_eq!(file["Content"]["ZoneIdentifier"]["ZoneId"], "3");
    assert_eq!(
        file["ResourceRecordEntries"],
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
    assert_eq!(lines[3]["Record"], "orphan-content");
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
        .find(|row| row["ResourceID"] == QUARANTINED_FILE)
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
    "Host",
    "HiveType",
    "SourcePath",
    "RecordType",
    "KeyPath",
    "KeyLastWritten",
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
fn hyphenated_registry_names_keep_hidden_legacy_aliases() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["NTUSER.DAT"], dir.path());
    let hive = dir.path().join("NTUSER.DAT");
    let menu = run(&["parse-registry", "--help"]);
    assert_eq!(menu.code, 0, "{}", menu.stderr);
    for (current, previous) in [
        ("app-compat", "appcompat"),
        ("feature-usage", "featureusage"),
        ("open-save-mru", "opensavemru"),
        ("recent-docs", "recentdocs"),
        ("user-assist", "userassist"),
        ("shell-bags", "shellbags"),
    ] {
        assert!(
            menu.stdout
                .lines()
                .any(|line| line.split_whitespace().next() == Some(current))
        );
        assert!(
            !menu
                .stdout
                .lines()
                .any(|line| line.split_whitespace().next() == Some(previous))
        );
        let help = run(&["parse-registry", current, "-h"]);
        let legacy_help = run(&["parse-registry", previous, "-h"]);
        assert_eq!(help.code, 0, "{current}: {}", help.stderr);
        assert_eq!(legacy_help.code, 0, "{previous}: {}", legacy_help.stderr);
        assert!(
            help.stdout
                .contains(&format!("juntei parse-registry {current} <INPUT>"))
        );
        assert_eq!(help.stdout, legacy_help.stdout);
        let parse = |command| {
            run(&[
                "parse-registry",
                command,
                "-f",
                hive.to_str().unwrap(),
                "-t",
                "jsonl",
                "--no-logs",
                "-K",
                "-q",
            ])
        };
        let new = parse(current);
        let old = parse(previous);
        assert_eq!(new.code, 0, "{current}: {}", new.stderr);
        assert_eq!(old.code, 0, "{previous}: {}", old.stderr);
        assert_eq!(
            new.stdout, old.stdout,
            "{current} and {previous} must parse the same records"
        );
    }
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
    assert!(help.contains("recent-docs "), "{help}");
    assert!(help.contains("taskcache "), "{help}");
    assert!(help.contains("sam "), "{help}");
    assert!(help.contains("persistence "), "{help}");
    assert!(help.contains("user-assist "), "{help}");
    assert!(help.contains("networks "), "{help}");
    assert!(help.contains("lsa-secrets "), "{help}");
    assert!(help.contains("bcd "), "{help}");
    assert!(help.contains("app-compat "), "{help}");
    assert!(help.contains("auditpol "), "{help}");
    assert!(help.contains("office "), "{help}");
    assert!(help.contains("security-settings "), "{help}");
    assert!(help.contains("feature-usage "), "{help}");
    assert!(help.contains("app-history "), "{help}");
    assert!(help.contains("mru "), "{help}");
    assert!(help.contains("system-info "), "{help}");
    assert!(help.contains("remote-access "), "{help}");
    assert!(help.contains("open-save-mru "), "{help}");
    assert!(help.contains("bam "), "{help}");
    assert!(help.contains("installed-programs "), "{help}");
    assert!(help.contains("muicache "), "{help}");
    assert!(help.contains("shell-bags "), "{help}");
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
            .find(|r| r["KeyPath"] == seven && r["ValueName"] == name)
            .unwrap()
    };
    assert_eq!(find("Publisher")["ValueData"], "igor pavlov");
    assert_eq!(find("Publisher")["Host"], Value::Null);
    assert_eq!(find("Publisher")["TransactionLogStatus"], "not_applied");
    assert_eq!(find("Size")["ValueData"], 535_040);
    assert_eq!(find("Size")["ValueU64"], 535_040);
    assert_eq!(find("Size")["ValueFiletimeUTC"], Value::Null);
    assert_eq!(
        find("Size")["KeyLastWritten"],
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
                assert_eq!(v["Host"], "GOLDPAPERW11");
                assert_eq!(v["ValueType"], "REG_DWORD");
                assert_eq!(v["TransactionLogStatus"], "unchanged");
                assert_eq!(v["LogsApplied"], json!(["SYSTEM.LOG1", "SYSTEM.LOG2"]));
                assert_eq!(v["ValueData"], v["ValueU64"]);
                (
                    v["ValueName"].as_str().unwrap().to_string(),
                    v["ValueU64"].as_u64().unwrap(),
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
        name[0]["KeyPath"],
        r"ControlSet001\Control\ComputerName\ComputerName"
    );
    assert_eq!(name[0]["ValueName"], "ComputerName");
    assert_eq!(name[0]["ValueData"], "GOLDPAPERW11");
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
    assert_eq!(replayed["ValueType"], "REG_QWORD");
    assert_eq!(replayed["ValueData"], 132_891_424_373_386_666_u64);
    assert_eq!(replayed["ValueU64"], 132_891_424_373_386_666_u64);
    assert_eq!(
        replayed["ValueFiletimeUTC"],
        "2022-02-12T12:27:17.338666600Z"
    );
    assert_eq!(replayed["TransactionLogStatus"], "replayed");
    assert_eq!(replayed["KeyLastWritten"], "2022-02-08T21:03:52.520572Z");
    let as_it_is = secure_time(&["--no-logs"]);
    assert_eq!(as_it_is["ValueU64"], 132_888_271_724_338_846_u64);
    assert_eq!(
        as_it_is["ValueFiletimeUTC"],
        "2022-02-08T20:52:52.433884600Z"
    );
    assert_eq!(as_it_is["TransactionLogStatus"], "not_applied");
    assert_eq!(as_it_is["LogsApplied"], json!([]));
    assert_eq!(as_it_is["KeyLastWritten"], "2022-02-08T20:52:54.093671200Z");
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
        .find(|r| r["ValueName"] == "BusType" && r["AllocationStatus"] != "allocated")
        .unwrap();
    assert_eq!(
        bus["KeyPath"],
        r"[unknown]\Configurations\Stornvme_Inst\Services\stornvme\Parameters"
    );
    assert_eq!(bus["AllocationStatus"], "deleted_partial");
    assert_eq!(bus["RecoveredFrom"], json!(["free_cell", "before_replay"]));
    assert_eq!(bus["KeyLastWritten"], "2022-02-07T07:38:03.052139100Z");
    assert_eq!(bus["ValueType"], "REG_DWORD");
    assert_eq!(bus["ValueU64"], 17);
    assert_eq!(bus["TransactionLogStatus"], "unchanged");
    let allocated = records
        .iter()
        .filter(|r| r["AllocationStatus"] == "allocated")
        .collect::<Vec<_>>();
    assert!(allocated.iter().all(|r| r["RecoveredFrom"] == json!([])));
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
        .find(|r| r["ValueName"] == "EventMessageFile")
        .unwrap();
    assert_eq!(
        message["KeyPath"],
        r"ControlSet001\Services\EventLog\System\Microsoft Edge Etw"
    );
    assert_eq!(message["AllocationStatus"], "transaction_log_recovered");
    assert_eq!(message["RecoveredFrom"], json!(["SYSTEM.LOG2#344"]));
    assert_eq!(
        message["ValueData"],
        r"C:\Program Files (x86)\Microsoft\Edge\Application\90.0.818.66\msedge.dll"
    );
    // without --recover-deleted: only what is in use
    let records = registry_read(&[
        "-f",
        system,
        "--key-regex",
        r"\\Services\\stornvme\\Parameters$",
    ]);
    assert!(records.iter().all(|r| r["AllocationStatus"] == "allocated"));
    // values without their key: no key filter takes them
    let all = registry_read(&["-f", system, "--recover-deleted", "--value-regex", "^"]);
    let residual: Vec<&Value> = all.iter().filter(|r| r["KeyPath"].is_null()).collect();
    let count = |status: &str| {
        residual
            .iter()
            .filter(|r| r["AllocationStatus"] == status)
            .count()
    };
    assert_eq!(
        (
            count("residual_value_only"),
            count("transaction_log_recovered")
        ),
        (53, 47)
    );
    assert!(residual.iter().all(|r| r["KeyLastWritten"].is_null()));
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
            .find(|r| r["ValueName"] == name && r["RecordType"] != "key")
            .unwrap()
    };
    assert_eq!(find("Multi")["ValueData"], json!(["a", "b c"]));
    assert_eq!(find("Qword")["ValueData"], json!(u64::MAX));
    assert_eq!(find("Qword")["ValueU64"], json!(u64::MAX));
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
    assert_eq!(qword[0]["ValueFiletimeUTC"], Value::Null);
    assert_eq!(qword[0]["ValueU64"], json!(u64::MAX));
    assert_eq!(find("Dword")["ValueDataRaw"], "07000000");
    assert_eq!(find("ShortDword")["ValueData"], Value::Null);
    assert_eq!(
        find("ShortDword")["Problems"],
        json!(["REG_DWORD of 3 bytes"])
    );
    assert_eq!(find("")["ValueData"], "the default");
    assert_eq!(find("")["RecordType"], "default_value");

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
                r["KeyPath"].as_str().unwrap(),
                r["RecordType"].as_str().unwrap(),
                r["ValueName"].as_str().unwrap(),
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
    assert_eq!(absent[0]["RecordType"], "absent_value");
    assert_eq!(absent[0]["ValueName"], "Missing");
    assert_eq!(absent[0]["ValueType"], Value::Null);
    assert_eq!(absent[0]["ValueData"], Value::Null);
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
        .map(|r| r["KeyPath"].as_str().unwrap())
        .collect();
    assert_eq!(
        paths,
        [
            format!(r"{state}\Machine\GPLink-List\0"),
            format!(r"{state}\Machine\GPLink-List\1"),
            format!(r"{state}\S-1-5-21-1-2-3-1001\GPLink-List\0"),
        ]
    );
    assert_eq!(links[0]["HiveType"], "SOFTWARE");
    assert_eq!(links[0]["KeyLastWritten"], "2022-02-08T21:03:52Z");
    assert_eq!(links[1]["KeyLastWritten"], "2022-02-09T21:03:52Z");
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
            .all(|r| r["KeyPath"] == format!(r"{state}\Machine\GPLink-List\1"))
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
        .map(|r| r["ValueData"].as_str().unwrap())
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
            .all(|r| r["KeyPath"] == format!(r"{state}\Machine\GPLink-List\1"))
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
    assert_eq!(records[0]["Host"], "GOLDPAPERW11");
    assert_eq!(records[0]["HiveType"], "AMCACHE.HVE");
    // hives exported to one directory: the SYSTEM hive next to it
    let flat = tempfile::tempdir().unwrap();
    let config = hive.path().join("HOST/C/Windows/System32/config");
    for name in ["SYSTEM", "SYSTEM.LOG1", "SYSTEM.LOG2"] {
        std::fs::copy(config.join(name), flat.path().join(name)).unwrap();
    }
    std::fs::copy(repo(AMCACHE_HIVE), flat.path().join("Amcache.hve")).unwrap();
    let amcache = flat.path().join("Amcache.hve");
    let records = registry_read(&["-f", amcache.to_str().unwrap(), "--key-regex", "^Root$"]);
    assert_eq!(records[0]["Host"], "GOLDPAPERW11");
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
    "ServiceDLL",
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
        ("ServiceDLL", r"%SystemRoot%\System32\wevtsvc.dll"),
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
            r#"[{"Type":"restart","DelayMs":60000},{"Type":"restart","DelayMs":120000},{"Type":"none","DelayMs":0}]"#,
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
    assert_eq!(wu[0]["ServiceName"], "wuauserv");
    assert_eq!(wu[0]["DelayedAutostart"], true);
    assert_eq!(
        wu[0]["FailureActions"][0],
        json!({"Type": "restart", "DelayMs": 60000})
    );
    // a driver: no failure actions
    let tcpip = services_read(&["-f", system, "--service", "tcpip"]);
    assert_eq!(tcpip[0]["StartType"], "boot");
    assert_eq!(tcpip[0]["ServiceType"], "kernel_driver");
    assert_eq!(tcpip[0]["FailureActions"], Value::Null);
    assert_eq!(tcpip[0]["FailureActionType"], Value::Null);

    // every control set: the one not in use said so
    let all = services_read(&["-f", system, "--all-control-sets"]);
    assert_eq!(all.len(), 933);
    let second: Vec<&Value> = all
        .iter()
        .filter(|r| r["ControlSet"] == "ControlSet002")
        .collect();
    assert_eq!(second.len(), 466);
    assert!(second.iter().all(|r| r["IsCurrentControlSet"] == false));
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
    assert_eq!(s["FailureResetPeriod"], 60);
    assert_eq!(
        s["FailureCommand"],
        r#""C:\Windows\System32\Spectrum.exe" -safemode"#
    );
    let actions: Vec<&str> = s["FailureActions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["Type"].as_str().unwrap())
        .collect();
    assert_eq!(
        actions,
        ["restart", "restart", "restart", "restart", "run_command"]
    );
    assert_eq!(s["TransactionLogStatus"], "unchanged");
    assert_eq!(s["LogsApplied"], json!(["SYSTEM.LOG1", "SYSTEM.LOG2"]));
    // user services: their type's flags
    let all = services_read(&["-f", system]);
    assert!(
        all.iter()
            .any(|r| r["ServiceType"] == "win32_share_process|user_service|user_service_instance")
    );
    // the logs wrote some services
    assert!(all.iter().any(|r| r["TransactionLogStatus"] == "replayed"));
    let without = services_read(&["-f", system, "--no-logs", "--service", "spectrum"]);
    assert_eq!(without[0]["TransactionLogStatus"], "not_applied");
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
        .find(|record| record["ServiceName"] == "ExampleSvc")
        .unwrap();
    assert_eq!(good["ServiceName"], "ExampleSvc");
    assert_eq!(good["ServiceDLL"], r"C:\ProgramData\example\svc.dll");
    assert_eq!(good["FailureActionType"], "restart");
    assert_eq!(good["FailureActionDelayMs"], 60000);
    assert_eq!(good["FailureActions"][1]["Type"], "run_command");
    assert_eq!(good["KeyLastWritten"], "2022-02-08T21:03:52Z");
    // what can't be read is a problem, its bytes kept; the rest is still read
    let short = records
        .iter()
        .find(|record| record["ServiceName"] == "Short")
        .unwrap();
    assert_eq!(short["StartType"], Value::Null);
    assert_eq!(short["FailureActions"], Value::Null);
    assert_eq!(
        short["Problems"],
        json!([
            "Start: is REG_SZ, not REG_DWORD",
            "FailureActions counts 3 actions, its 28 bytes hold 1"
        ])
    );
    assert!(
        short["FailureActionsRaw"]
            .as_str()
            .unwrap()
            .starts_with("100e0000")
    );
    let wrong = records
        .iter()
        .find(|record| record["ServiceName"] == "WrongTypes")
        .unwrap();
    assert_eq!(wrong["StartType"], Value::Null);
    assert_eq!(wrong["FailureActions"], Value::Null);
    assert_eq!(
        wrong["Problems"],
        json!([
            "Start: is REG_QWORD, not REG_DWORD",
            "FailureActions: is REG_SZ, not REG_BINARY"
        ])
    );
    assert!(
        wrong["FailureActionsRaw"]
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
            .all(|record| record["ControlSet"] == "ControlSet001")
    );
    assert!(
        records
            .iter()
            .all(|record| record["IsCurrentControlSet"] == false)
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

// --- parse-registry recent-docs ------------------------------------------------------------------

/// `parse-registry recent-docs`'s CSV columns.
const RECENTDOCS_COLUMNS: [&str; 14] = [
    "Host",
    "User",
    "SourcePath",
    "Extension",
    "MRU-Position",
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

/// `parse-registry recent-docs` with `args`, as JSON Lines.
fn recentdocs_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "recent-docs"];
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
    let r = run(&["parse-registry", "recent-docs", "-d", dir, "-K"]);
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
        ("Host", json!("GOLDPAPERW11")),
        ("User", json!("AndrewRathbun")),
        ("Extension", Value::Null),
        ("MRU-Position", json!(0)),
        ("ValueIndex", json!(10)),
        ("Filename", json!("The Internet")),
        ("LinkName", json!("The Internet.lnk")),
        ("KeyLastWritten", json!("2022-02-13T03:54:53.963087500Z")),
        ("LastOpened", json!("2022-02-13T03:54:53.963087500Z")),
        ("ValueDataRaw", Value::Null),
        ("TransactionLogStatus", json!("unchanged")),
        ("Problems", json!([])),
    ] {
        assert_eq!(first[field], expected, "{field}");
    }
    // the most recent item of all, as the issue's jq query finds it
    let latest: Vec<&Value> = records
        .iter()
        .filter(|r| r["Extension"].is_null() && r["MRU-Position"] == 0)
        .collect();
    assert_eq!(latest.len(), 1);
    // one extension's list: only its most recent item has the time it was opened
    let txt: Vec<(&str, &str, &Value)> = records
        .iter()
        .filter(|r| r["Extension"] == ".txt")
        .map(|r| {
            (
                r["Filename"].as_str().unwrap(),
                r["LinkName"].as_str().unwrap(),
                &r["LastOpened"],
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
        .filter(|r| r["Extension"] == "Folder")
        .map(|r| r["Filename"].as_str().unwrap())
        .collect();
    assert_eq!(folders, ["The Internet", "UsageLogs", "OneDrive", "logs"]);
    // without the logs: the same items, said to be possibly out of date
    let without = recentdocs_read(&["-d", dir, "--no-logs"]);
    assert_eq!(without.len(), 22);
    assert!(
        without
            .iter()
            .all(|r| r["TransactionLogStatus"] == "not_applied")
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
        "recent-docs",
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
            .find(|r| r["Extension"] == ext && r["MRU-Position"] == position)
            .unwrap()
    };
    for (column, expected) in [
        ("Host", ""),
        ("User", "vibranium"),
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
                &r["MRU-Position"],
                &r["ValueIndex"],
                &r["Filename"],
                &r["Problems"],
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
    assert_eq!(records[0]["User"], "bob");
    assert_eq!(records[0]["LastOpened"], "2022-02-08T21:03:52Z");
    assert!(records[3]["ValueDataRaw"].is_string());
    let r = run(&[
        "parse-registry",
        "recent-docs",
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
        "recent-docs",
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
        "recent-docs",
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
    let r = run(&[
        "parse-registry",
        "recent-docs",
        "-d",
        "test_files/lnk",
        "-K",
    ]);
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
    "TaskGUID",
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
    "Args",
    "WorkingDirectory",
    "COM-ClassID",
    "ActionData",
    "Context",
    "Author",
    "URI",
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
        ("TaskGUID", "{13ABE1C7-3F21-4017-9BF6-899BE8C1BA89}"),
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
        ("Args", r"sekurlsa::LogonPasswords > C:\TMP\o.txt"),
        ("WorkingDirectory", ""),
        ("Context", "Author"),
        ("Author", "Example"),
        ("URI", r"\GameOver"),
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
            .find(|r| r["TaskName"] == name)
            .unwrap()
            .clone()
    };
    // the system task: its lists, its last successful run
    let sync = find(r"\Microsoft\Windows\Time Synchronization\SynchronizeTime");
    assert_eq!(sync["TaskLists"], json!(["Plain", "Maintenance"]));
    assert_eq!(sync["Context"], "LocalService");
    assert_eq!(sync["Executable"], r"%windir%\system32\sc.exe");
    assert_eq!(sync["Args"], "start w32time task_started");
    assert_eq!(sync["LastSuccessfulRun"], "2022-02-13T03:54:43.724066300Z");
    assert_eq!(sync["LastResult"], "0x00000420");
    // hidden: no SD
    let falsh = find(r"\falshupdate22");
    assert_eq!(falsh["Hidden"], true);
    assert_eq!(falsh["HasSecurityDescriptor"], false);
    assert_eq!(falsh["TaskLists"], json!(["Logon"]));
    // an Index 0 entry without a Tasks key: one record, no action
    let defrag = find(r"\Microsoft\Windows\Defrag\ScheduledDefrag");
    assert_eq!(
        (&defrag["TaskIndex"], &defrag["InTasks"], &defrag["Hidden"]),
        (&json!(0), &json!(false), &json!(false))
    );
    assert_eq!(defrag["ActionType"], Value::Null);
    // no Tree entry: hidden, its cut Actions kept
    let orphan = find(r"\Orphan");
    assert_eq!(
        (&orphan["InTree"], &orphan["Hidden"]),
        (&json!(false), &json!(true))
    );
    assert_eq!(orphan["HasSecurityDescriptor"], Value::Null);
    assert_eq!(
        orphan["Problems"],
        json!(["Actions: a string of 48 bytes at 36 is cut short (20 bytes left)"])
    );
    assert_eq!(orphan["ActionsRaw"].as_str().unwrap().len(), 120);

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
    "RID",
    "User",
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
        ("Host", json!("GOLDPAPERW11")),
        ("RID", json!(1001)),
        ("User", json!("AndrewRathbun")),
        (
            "AccountFlags",
            json!([
                "password_not_required",
                "normal_account",
                "dont_expire_password"
            ]),
        ),
        ("AccountFlagsValue", json!(532)),
        ("CountryCode", json!(0)),
        ("CodePage", json!(0)),
        ("LastLogon", json!("2022-02-08T20:43:51.966977200Z")),
        ("LastLogonWholeSeconds", json!("2022-02-08T20:43:51Z")),
        // never set, never expires: null, the numbers kept
        ("PasswordLastSet", Value::Null),
        ("PasswordLastSetFiletime", json!(0)),
        ("AccountExpiration", Value::Null),
        (
            "AccountExpirationFiletime",
            json!(0x7FFF_FFFF_FFFF_FFFF_u64),
        ),
        ("LogonCount", json!(4)),
        ("FailedLogonCount", json!(0)),
        (
            "NamesKeyLastWritten",
            json!("2022-02-05T18:59:49.448358500Z"),
        ),
        ("HiddenFromLogonUi", Value::Null),
        ("TransactionLogStatus", json!("clean")),
        ("Problems", json!([])),
    ] {
        assert_eq!(user[0][field], expected, "{field}");
    }
    // the built-in accounts the sign-in screen hides
    let all = sam_read(&["-d", dir]);
    let hidden: Vec<&Value> = all
        .iter()
        .filter(|r| r["HiddenFromLogonUi"] == true)
        .map(|r| &r["RID"])
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
        ("RID", "500"),
        ("User", "Administrator"),
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
    assert_eq!(guest[0]["User"], "Guest");
    assert_eq!(
        guest[0]["AccountExpirationFiletime"],
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
    assert_eq!(support["HiddenFromLogonUi"], true);
    assert_eq!(support["PasswordHint"], "the usual");
    assert_eq!(support["InternetUserName"], "support@example.com");
    assert_eq!(support["HomeDrive"], "H:");
    assert_eq!(support["LastLogon"], "2022-02-08T21:03:52Z");
    assert_eq!(
        support["AccountFlags"],
        json!(["normal_account", "dont_expire_password"])
    );
    let partial = &records[2];
    assert_eq!(partial["RID"], 1003);
    assert_eq!(partial["AccountFlags"], Value::Null);
    assert_eq!(partial["AccountFlagsValue"], Value::Null);
    assert_eq!(partial["Problems"], json!(["F is not there"]));

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
            "RecordVCN",
            "RecordLSN",
            "EntryOffset",
            "FromSlack",
            "CopyOfEntryInUse",
            "Filename",
            "Namespace",
            "FileEntry",
            "FileSequence",
            "ParentEntry",
            "ParentSequence",
            "CreatedUTC",
            "ModifiedUTC",
            "MFT-ModifiedUTC",
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
        .find(|r| r["Filename"] == "app-022.log")
        .unwrap();
    // the fields in their order
    let text = std::fs::read_to_string(&json).unwrap();
    let first = &text[..text.find("\n}").unwrap()];
    let at = |key: &str| first.find(&format!("\"{key}\":")).unwrap();
    let order = [
        "SourcePath",
        "RecordOffset",
        "RecordVCN",
        "RecordLSN",
        "EntryOffset",
        "FromSlack",
        "CopyOfEntryInUse",
        "Filename",
        "Namespace",
        "FileEntry",
        "FileSequence",
        "ParentEntry",
        "ParentSequence",
        "Created",
        "Modified",
        "MFT-Modified",
        "Accessed",
        "AllocatedSize",
        "RealSize",
        "FileAttributes",
        "ReparseValue",
        "Problems",
    ];
    assert!(order.windows(2).all(|w| at(w[0]) < at(w[1])), "{first}");
    assert_eq!(deleted["FromSlack"], true);
    assert_eq!(deleted["CopyOfEntryInUse"], false);
    assert_eq!(deleted["FileEntry"], 87);
    assert_eq!(deleted["ParentEntry"], 64);
    assert_eq!(
        (&deleted["RealSize"], &deleted["AllocatedSize"]),
        (&json!(8), &json!(8))
    );
    assert_eq!(deleted["Namespace"], "posix");
    assert_eq!(deleted["FileAttributes"], "archive");
    assert_eq!(deleted["Problems"], json!([]));
    for time in ["Created", "Modified", "MFT-Modified", "Accessed"] {
        assert!(
            deleted[time].as_str().unwrap().ends_with('Z'),
            "{time}: {}",
            deleted[time]
        );
    }
    // the password list's copies, their parent written over
    let passwords: Vec<&Value> = records
        .iter()
        .filter(|r| r["Filename"] == "passwords.txt")
        .collect();
    assert_eq!(passwords.len(), 11);
    assert!(
        passwords
            .iter()
            .all(|r| r["ParentEntry"].is_null() && r["FileEntry"].is_null())
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
                r["Filename"].as_str().unwrap(),
                r["FromSlack"].as_bool().unwrap(),
                r["CopyOfEntryInUse"].as_bool().unwrap(),
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
    assert!(lines.iter().all(|r| r["RecordOffset"] == 0x30200));
    assert_eq!(lines[5]["EntryOffset"], 0x30200 + 576);

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
    "UserSID",
    "Mechanism",
    "AttackTechnique",
    "KeyPath",
    "ValueName",
    "Data",
    "ImagePath",
    "Args",
    "Enabled",
    "DisabledAtUTC",
    "Details",
    "OverridesMachineRegistration",
    "WOW64",
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
                r["HiveType"].as_str().unwrap(),
                r["Mechanism"].as_str().unwrap(),
                r["KeyPath"].as_str().unwrap(),
                r["ValueName"].as_str().unwrap(),
                r["Data"].as_str().unwrap_or("-")
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
    assert!(records.iter().all(|r| r["Host"] == "HOST1"));
    // the user hives' user and SID: UsrClass.dat's root name, NTUSER.DAT's profile in ProfileList
    for r in &records[..6] {
        assert_eq!(r["User"], "alice");
        assert_eq!(r["UserSID"], "S-1-5-21-1-2-3-1001");
    }
    assert!(records[6..].iter().all(|r| r["User"].is_null()));
    // the user's class {AAAA} overrides the machine's
    assert_eq!(records[0]["OverridesMachineRegistration"], true);
    assert_eq!(records[1]["OverridesMachineRegistration"], false);
    assert_eq!(records[1]["ImagePath"], r"C:\Users\alice\own.exe");
    assert_eq!(records[1]["Args"], "-Embedding");
    assert_eq!(records[0]["AttackTechnique"], "T1546.015");
    // StartupApproved: Startup folder files enabled and disabled; Updater has no entry, Off is
    // disabled, Gone has no Run value
    assert_eq!(records[3]["Enabled"], true);
    assert!(records[3]["DisabledAt"].is_null());
    assert_eq!(records[3]["Details"], "a file in the Startup folder");
    assert_eq!(records[4]["Enabled"], false);
    assert_eq!(records[4]["DisabledAt"], "2022-02-08T21:03:52Z");
    assert!(records[6]["Enabled"].is_null());
    assert_eq!(records[7]["Enabled"], false);
    assert_eq!(records[7]["DisabledAt"], "2022-02-08T21:03:52Z");
    assert_eq!(records[8]["Enabled"], false);
    assert_eq!(
        records[8]["Details"],
        "only in StartupApproved: the Run key has no value of this name"
    );
    assert!(
        records
            .iter()
            .all(|r| r["TransactionLogStatus"] == "clean" && r["Problems"] == json!([]))
    );
    assert!(
        records
            .iter()
            .all(|r| r["KeyLastWritten"] == "2022-02-08T21:03:52Z")
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
            .all(|r| r["OverridesMachineRegistration"].is_null()
                && r["UserSID"] == "S-1-5-21-1-2-3-1001")
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
        records[0]["ImagePath"],
        r"%ProgramFiles%\Windows Sidebar\Sidebar.exe"
    );
    assert_eq!(records[0]["Args"], "/autoRun");

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
            .filter(|r| r["Mechanism"] == mechanism)
            .map(|r| r["Data"].as_str().unwrap())
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
        ["replayed", "unchanged"].contains(&r["TransactionLogStatus"].as_str().unwrap())
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

// --- parse-registry usb -------------------------------------------------------------------------

/// `parse-registry usb`'s CSV columns.
const USB_COLUMNS: [&str; 33] = [
    "Host",
    "DeviceClass",
    "ControlSet",
    "Vendor",
    "Product",
    "Revision",
    "Vid",
    "PID",
    "Serial",
    "Instance",
    "FriendlyName",
    "DeviceDesc",
    "Service",
    "ContainerID",
    "DeviceFirstInstallUTC",
    "DriverLastInstalledUTC",
    "LastArrivalUTC",
    "LastRemovalUTC",
    "DriveLetters",
    "VolumeGUID",
    "VolumeData",
    "VolumeNames",
    "VolumeSerials",
    "User",
    "FoundIn",
    "SourceKeys",
    "KeyPath",
    "KeyLastWrittenUTC",
    "SourcePath",
    "SoftwarePath",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// A device path as `MountedDevices` keeps it (UTF-16, no NUL).
fn mounted_path(path: &str) -> Vec<u8> {
    path.encode_utf16().flat_map(u16::to_le_bytes).collect()
}

/// One system's SYSTEM, SOFTWARE and users' hives with USB history: a USB stick (its USB
/// device, its portable device, its volume as E:, its ReadyBoost serial), C:'s MBR volume, the
/// volume of a removed stick whose device key is gone, a phone only SOFTWARE names; alice had the
/// stick mounted, bob a volume no SYSTEM value has.
fn usb_collection(root: &Path) {
    let stick = r"ControlSet001\Enum\USBSTOR\Disk&Ven_Acme&Prod_Stick&Rev_1.00\SER123&0";
    let stick_path = r"_??_USBSTOR#Disk&Ven_Acme&Prod_Stick&Rev_1.00#SER123&0#{53f56307-b6bf-11d0-94f2-00a0c91efb8b}";
    let gone_path = r"_??_USBSTOR#Disk&Ven_Gone&Prod_Drive&Rev_9#GONE77&0#{53f56307-b6bf-11d0-94f2-00a0c91efb8b}";
    let props = format!(r"{stick}\Properties\{{83da6326-97a6-4088-9453-a1923f573b29}}");
    let time = |at: u64| at.to_le_bytes().to_vec();
    let container = utf16z("{11111111-2222-3333-4444-555555555555}");
    let mut mbr = 0x1234_5678_u32.to_le_bytes().to_vec();
    mbr.extend(1_048_576_u64.to_le_bytes());
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
                stick,
                "FriendlyName",
                REG_SZ,
                utf16z("Acme Stick USB Device"),
            ),
            (stick, "ContainerID", REG_SZ, container.clone()),
            (stick, "Service", REG_SZ, utf16z("disk")),
            (&format!(r"{props}\0064"), "", 0xffff_0010, time(LINKED)),
            (
                &format!(r"{props}\0065"),
                "",
                0xffff_0010,
                time(LINKED - DAY),
            ),
            (
                &format!(r"{props}\0066"),
                "",
                0xffff_0010,
                time(LINKED + DAY),
            ),
            (&format!(r"{props}\0067"), "", 0xffff_0010, vec![1, 2, 3, 4]),
            (
                r"ControlSet001\Enum\USB\VID_1234&PID_5678\SER123",
                "ContainerID",
                REG_SZ,
                container,
            ),
            (
                &format!(r"ControlSet001\Enum\SWD\WPDBUSENUM\{stick_path}"),
                "FriendlyName",
                REG_SZ,
                utf16z("ACME_VOL"),
            ),
            (
                "MountedDevices",
                r"\??\Volume{aaaaaaaa-0000-0000-0000-000000000001}",
                REG_BINARY,
                mounted_path(stick_path),
            ),
            (
                "MountedDevices",
                r"\DosDevices\E:",
                REG_BINARY,
                mounted_path(stick_path),
            ),
            (
                "MountedDevices",
                r"\??\Volume{bbbbbbbb-0000-0000-0000-000000000002}",
                REG_BINARY,
                mbr.clone(),
            ),
            ("MountedDevices", r"\DosDevices\C:", REG_BINARY, mbr),
            (
                "MountedDevices",
                r"\??\Volume{cccccccc-0000-0000-0000-000000000003}",
                REG_BINARY,
                mounted_path(gone_path),
            ),
        ],
    );
    let emd = r"Microsoft\Windows NT\CurrentVersion\EMDMgmt";
    let software = reg_tree(
        "ROOT",
        &[
            (
                &format!(r"{emd}\{stick_path}ACME_VOL_305419896"),
                "CacheSizeInMB",
                REG_DWORD,
                0u32.to_le_bytes().to_vec(),
            ),
            (
                r"Microsoft\Windows Portable Devices\Devices\USB#VID_0781&PID_7400#0781005B300304C7",
                "FriendlyName",
                REG_SZ,
                utf16z("Sansa m240"),
            ),
        ],
    );
    let mp2 = r"Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2";
    let user = |guid: &str| {
        reg_tree(
            "ROOT",
            &[(
                &format!(r"{mp2}\{guid}"),
                "BaseClass",
                REG_DWORD,
                0u32.to_le_bytes().to_vec(),
            )],
        )
    };
    let config = root.join("C/Windows/System32/config");
    for dir in [
        &config,
        &root.join("C/Users/alice"),
        &root.join("C/Users/bob"),
    ] {
        std::fs::create_dir_all(dir).unwrap();
    }
    std::fs::write(config.join("SYSTEM"), system).unwrap();
    std::fs::write(config.join("SOFTWARE"), software).unwrap();
    std::fs::write(
        root.join("C/Users/alice/NTUSER.DAT"),
        user("{aaaaaaaa-0000-0000-0000-000000000001}"),
    )
    .unwrap();
    std::fs::write(
        root.join("C/Users/bob/NTUSER.DAT"),
        user("{dddddddd-0000-0000-0000-000000000004}"),
    )
    .unwrap();
}

/// `parse-registry usb` with `args`, as JSON Lines.
fn usb_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "usb"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_usb_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    usb_collection(dir.path());
    let records = usb_read(&["-d", dir.path().to_str().unwrap()]);
    let text = |v: &Value| v.as_str().unwrap_or("-").to_owned();
    let list = |v: &Value| {
        v.as_array()
            .unwrap()
            .iter()
            .map(|x| {
                x.as_str()
                    .map_or_else(|| x["User"].to_string(), str::to_owned)
            })
            .collect::<Vec<_>>()
            .join(",")
    };
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} {}/{}/{} {}:{} {} [{}] {} [{}] [{}] [{}]",
                text(&r["DeviceClass"]),
                text(&r["Vendor"]),
                text(&r["Product"]),
                text(&r["Revision"]),
                text(&r["Vid"]),
                text(&r["PID"]),
                text(&r["Serial"]),
                list(&r["DriveLetters"]),
                text(&r["VolumeGUID"]),
                list(&r["VolumeNames"]),
                list(&r["VolumeSerials"]),
                list(&r["User"]),
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            r#"usbstor Acme/Stick/1.00 1234:5678 SER123 [E:] {aaaaaaaa-0000-0000-0000-000000000001} [ACME_VOL] [1234-5678] ["alice"]"#,
            "usb -/-/- 1234:5678 SER123 [] - [] [] []",
            r#"wpdbusenum Acme/Stick/1.00 -:- SER123 [E:] {aaaaaaaa-0000-0000-0000-000000000001} [ACME_VOL] [1234-5678] ["alice"]"#,
            "volume -/-/- -:- - [C:] {bbbbbbbb-0000-0000-0000-000000000002} [] [] []",
            "volume Gone/Drive/9 -:- GONE77 [] {cccccccc-0000-0000-0000-000000000003} [] [] []",
            r#"volume -/-/- -:- - [] {dddddddd-0000-0000-0000-000000000004} [] [] ["bob"]"#,
            "portable_device -/-/- 0781:7400 0781005B300304C7 [] - [Sansa m240] [] []",
        ]
    );
    let stick = &records[0];
    assert_eq!(stick["Host"], "HOST1");
    assert_eq!(stick["ControlSet"], 1);
    assert_eq!(
        (
            &stick["DeviceFirstInstall"],
            &stick["DriverLastInstalled"],
            &stick["LastArrival"],
            &stick["LastRemoval"]
        ),
        (
            &json!("2022-02-07T21:03:52Z"),
            &json!("2022-02-08T21:03:52Z"),
            &json!("2022-02-09T21:03:52Z"),
            &Value::Null
        )
    );
    // a removal time that isn't 8 bytes keeps them in a problem
    assert_eq!(
        stick["Problems"],
        json!([
            r"Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0067: type 0xffff0010 of 4 bytes, not a FILETIME (01020304)"
        ])
    );
    assert_eq!(
        stick["FoundIn"],
        json!(["Enum", "EMDMgmt", "MountedDevices", "MountPoints2"])
    );
    assert_eq!(
        stick["User"][0]["MountPoints2LastWritten"],
        "2022-02-08T21:03:52Z"
    );
    assert!(
        stick["User"][0]["SourcePath"]
            .as_str()
            .unwrap()
            .ends_with("NTUSER.DAT")
    );
    assert!(
        stick["SoftwarePath"]
            .as_str()
            .unwrap()
            .ends_with("SOFTWARE")
    );
    assert_eq!(
        records[3]["VolumeData"],
        "MBR disk signature 12345678, partition offset 1048576"
    );
    assert!(records.iter().all(|r| r["TransactionLogStatus"] == "clean"));
}

#[test]
fn registry_usb_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    usb_collection(dir.path());
    let out = dir.path().join("usb.csv");
    let r = run(&[
        "parse-registry",
        "usb",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "SYSTEM hives parsed: 1",
        "Hosts: HOST1",
        "User hives read: 2",
        "Users with volumes in MountPoints2: alice, bob",
        "Records: 7 (portable_device 1, usb 1, usbstor 1, volume 3, wpdbusenum 1)",
        "Volumes of USB storage devices without a device key (removed devices): 1",
        "2022-02-09T21:03:52Z  Acme Stick USB Device (serial SER123) E:",
        "Records or keys that could not be read: 1",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let header: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, USB_COLUMNS);
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(rows.len(), 7);
    assert_eq!(&rows[0][18], "E:");
    assert_eq!(&rows[0][23], "alice (2022-02-08T21:03:52Z)");
    assert_eq!(&rows[0][24], "Enum|EMDMgmt|MountedDevices|MountPoints2");
    assert!(rows[0][25].starts_with(r"SYSTEM ControlSet001\Enum\USBSTOR\Disk&Ven_Acme"));
}

#[test]
fn registry_usb_of_public_hives() {
    // regipy's Windows 7 SYSTEM and NTUSER.DAT, of one computer: the HP drive regipy's
    // usbstor_plugin and Plaso's windows_usbstor_devices read, its volume as E:, and the volumes
    // the user had mounted
    let dir = regipy_system();
    unpack_registry(
        "regipy",
        &["NTUSER.DAT"],
        &dir.path().join("W7/C/Users/user"),
    );
    let records = usb_read(&["-d", dir.path().to_str().unwrap()]);
    let hp = records
        .iter()
        .find(|r| r["DeviceClass"] == "usbstor")
        .unwrap();
    assert_eq!(
        (
            &hp["Vendor"],
            &hp["Product"],
            &hp["Revision"],
            &hp["Serial"],
            &hp["FriendlyName"]
        ),
        (
            &json!("HP"),
            &json!("v100w"),
            &json!("1024"),
            &json!("AA951D0000007252"),
            &json!("HP v100w USB Device")
        )
    );
    // Windows 7 keeps the install times in 00000064\00000000 and 00000065\00000000
    assert_eq!(hp["DriverLastInstalled"], "2011-04-01T04:52:38.686Z");
    assert_eq!(hp["DeviceFirstInstall"], "2011-04-01T04:52:38.686Z");
    assert_eq!(hp["DriveLetters"], json!(["E:"]));
    assert_eq!(hp["VolumeGUID"], "{eba74da6-5bb2-11e0-95d1-000c2971073c}");
    let mounted: Vec<String> = records
        .iter()
        .filter(|r| !r["User"].as_array().unwrap().is_empty())
        .map(|r| format!("{} {}", r["DriveLetters"], r["User"][0]["User"]))
        .collect();
    assert_eq!(mounted, [r#"["C:"] "user""#, r#"["D:"] "user""#]);
    assert_eq!(records.len(), 14);
    // the other control set too
    let all = usb_read(&["-d", dir.path().to_str().unwrap(), "--all-control-sets"]);
    let sets: Vec<&Value> = all
        .iter()
        .filter(|r| r["DeviceClass"] == "usbstor")
        .map(|r| &r["ControlSet"])
        .collect();
    assert_eq!(sets, [&json!(1), &json!(2)]);

    // the DFIR Artifact Museum's dirty Windows 11 SYSTEM and NTUSER.DAT, with their logs
    let w11 = system_hive();
    unpack_registry(
        "rathbun_w11",
        &["NTUSER.DAT", "ntuser.dat.LOG1", "ntuser.dat.LOG2"],
        &w11.path().join("HOST/C/Users/user"),
    );
    let records = usb_read(&["-d", w11.path().to_str().unwrap()]);
    let volumes: Vec<String> = records
        .iter()
        .filter(|r| r["DeviceClass"] == "volume")
        .map(|r| {
            format!(
                "{} {} {}",
                r["DriveLetters"],
                r["User"].as_array().unwrap().len(),
                r["TransactionLogStatus"]
            )
        })
        .collect();
    assert_eq!(
        volumes,
        [
            // the CD-ROM's volume, which the user had mounted
            r#"["D:"] 1 "unchanged""#,
            // C:, a GPT partition with no volume GUID value
            r#"["C:"] 0 "unchanged""#,
            // a volume only the user's MountPoints2 has
            r#"[] 1 "unchanged""#,
        ]
    );
    assert!(records.iter().all(|r| r["Host"] == "GOLDPAPERW11"));
}

#[test]
fn registry_usb_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&["parse-registry", "usb", "-f", sam.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("a SAM hive, not a SYSTEM hive"), "{said}");
    let r = run(&[
        "parse-registry",
        "usb",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("No SYSTEM hives were found"), "{said}");
    // the output is never one of the files read: a user hive of the system included
    let collection = tempfile::tempdir().unwrap();
    usb_collection(collection.path());
    let alice = collection.path().join("C/Users/alice/NTUSER.DAT");
    let original = std::fs::read(&alice).unwrap();
    let r = run(&[
        "parse-registry",
        "usb",
        "-d",
        collection.path().to_str().unwrap(),
        "-o",
        alice.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(&alice).unwrap(), original);
}

// --- parse-registry user-assist ------------------------------------------------------------------

/// `parse-registry user-assist`'s CSV columns.
const USERASSIST_COLUMNS: [&str; 20] = [
    "Host",
    "User",
    "UserSID",
    "SourcePath",
    "GUID",
    "Kind",
    "NameRot13",
    "Name",
    "Path",
    "RunCount",
    "FocusCount",
    "FocusTimeMs",
    "LastRunUTC",
    "Format",
    "SessionID",
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

/// `parse-registry user-assist` with `args`, as JSON Lines.
fn userassist_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "user-assist"];
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
                r["Kind"].as_str().unwrap(),
                r["Name"].as_str().unwrap(),
                r["Path"].as_str().unwrap_or("-"),
                r["RunCount"],
                r["FocusCount"],
                r["FocusTimeMs"],
                r["LastRun"].as_str().unwrap_or("-"),
                r["Format"]
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
        assert_eq!(r["Host"], "HOST1");
        assert_eq!(r["User"], "alice");
        assert_eq!(r["UserSID"], "S-1-5-21-1-2-3-1001");
        assert_eq!(r["TransactionLogStatus"], "clean");
        assert_eq!(r["KeyLastWritten"], "2022-02-08T21:03:52Z");
    }
    // the {GUID} keys in their list's order (by name), each Count's values in theirs
    assert_eq!(records[1]["GUID"], "{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}");
    assert_eq!(
        records[1]["NameRot13"],
        r"{1NP14R77-02R7-4R5Q-O744-2RO1NR5198O7}\pzq.rkr"
    );
    // the value cut short keeps its bytes
    assert_eq!(records[3]["DataRaw"], "010203");
    assert_eq!(
        records[3]["Problems"],
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
        "user-assist",
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
            r["Name"]
                .as_str()
                .unwrap()
                .ends_with(r"\Welcome Center.lnk")
        })
        .unwrap();
    assert_eq!(welcome["Kind"], "shortcut");
    assert_eq!(
        welcome["Path"],
        r"%ALLUSERSPROFILE%\Microsoft\Windows\Start Menu\Programs\Accessories\Welcome Center.lnk"
    );
    assert_eq!(
        (
            &welcome["RunCount"],
            &welcome["FocusCount"],
            &welcome["FocusTimeMs"]
        ),
        (&json!(14), &json!(0), &json!(14))
    );
    assert_eq!(welcome["LastRun"], "2012-04-03T22:06:58.124282300Z");

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
        .find(|r| r["Name"] == "Microsoft.WindowsNotepad_8wekyb3d8bbwe!App")
        .unwrap();
    assert_eq!(
        (
            &notepad["RunCount"],
            &notepad["FocusCount"],
            &notepad["FocusTimeMs"]
        ),
        (&json!(16), &json!(9), &json!(83515))
    );
    assert_eq!(notepad["LastRun"], "2022-02-06T20:15:41.701Z");
    let cmd = records
        .iter()
        .find(|r| r["Path"] == r"%windir%\system32\cmd.exe")
        .unwrap();
    assert_eq!(
        (&cmd["RunCount"], &cmd["FocusTimeMs"]),
        (&json!(0), &json!(156))
    );
    assert!(records.iter().all(|r| r["Problems"] == json!([])));
}

#[test]
fn registry_userassist_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&[
        "parse-registry",
        "user-assist",
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
        "user-assist",
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
        "user-assist",
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
        "user-assist",
        "-f",
        hive.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let records = jsonl_records(&r.stdout);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["Name"], r"P:\Gbbyf\cnlybnq.rkr");
    assert_eq!(records[0]["Problems"], json!([no_encrypt]));
    // with the records on stdout, the hive's warning goes to stderr
    assert!(r.stderr.contains("NoEncrypt is 1"), "{}", r.stderr);
    let out = dir.path().join("plain.csv");
    let r = run(&[
        "parse-registry",
        "user-assist",
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
        assert!(r["LastRun"].is_null());
        assert_eq!(
            r["Problems"],
            json!(["last run FILETIME 0xffffffffffffffff is past 9999"])
        );
        assert_eq!(r["DataRaw"].as_str().unwrap().len(), 144);
    }
    assert_eq!(records[21]["LastRun"], "2022-02-08T21:03:52Z");
    let out = dir.path().join("corrupt.csv");
    let r = run(&[
        "parse-registry",
        "user-assist",
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

#[test]
fn persistence_hunter_start_menu_unresolved_candidates_survive_all_output_formats() {
    let evidence = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    let clsid = "{00112233-4455-6677-8899-AABBCCDDEEFF}";
    let folder = evidence
        .path()
        .join("Users/alice/AppData/Roaming/Microsoft/Windows/Start Menu")
        .join(format!("Programs/Example.{clsid}"));
    std::fs::create_dir_all(&folder).unwrap();
    for kind in ["csv", "json", "jsonl"] {
        let path = output.path().join(format!("findings.{kind}"));
        let r = run(&[
            "persistence-hunter",
            "-d",
            evidence.path().to_str().unwrap(),
            "--check",
            "start_menu_com",
            "--no-timeline",
            "--min-severity",
            "info",
            "-t",
            kind,
            "-o",
            path.to_str().unwrap(),
            "-q",
            "-K",
        ]);
        assert_eq!(r.code, 0, "{}\n{}", r.stdout, r.stderr);
        if kind == "csv" {
            let (_, rows) = csv_rows(&path);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0]["Value"], clsid);
            assert_eq!(rows[0]["ResolutionStatus"], "unresolved_com_class");
            assert_eq!(rows[0]["ResolvedPath"], "");
            assert_eq!(rows[0]["Exists"], "");
        } else {
            let row: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            assert_eq!(row["Value"], clsid);
            assert_eq!(row["ResolutionStatus"], "unresolved_com_class");
            assert_eq!(row["ResolvedPath"], "");
            assert!(row["Exists"].is_null());
        }
    }
}

#[test]
fn persistence_hunter_has_consistent_command_help() {
    let root = run(&["-h"]);
    assert_eq!(root.code, 0);
    assert!(root.stdout.contains("persistence-hunter"));
    let help = run(&["persistence-hunter", "-h"]);
    assert_eq!(help.code, 0, "{}", help.stderr);
    assert!(
        help.stdout
            .contains("juntei persistence-hunter -d <EVIDENCE>")
    );
    assert!(help.stdout.contains("--min-severity") && help.stdout.contains("--coverage"));
}

#[test]
fn registry_usb_duplicate_user_names_preserve_each_source_hive() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("SYSTEM"),
        reg_tree(
            "ROOT",
            &[
                ("Select", "Current", REG_DWORD, 1u32.to_le_bytes().to_vec()),
                (
                    r"ControlSet001\Control\ComputerName\ComputerName",
                    "ComputerName",
                    REG_SZ,
                    utf16z("TESTHOST"),
                ),
            ],
        ),
    )
    .unwrap();
    let guid = "{aaaaaaaa-0000-0000-0000-000000000001}";
    let mut sources = Vec::new();
    for folder in ["export-one", "export-two"] {
        let profile = dir.path().join(folder);
        std::fs::create_dir(&profile).unwrap();
        let ntuser = profile.join("NTUSER.DAT");
        let hive = reg_tree(
            "ROOT",
            &[
                (
                    r"Software\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders",
                    "Local AppData",
                    REG_SZ,
                    utf16z(r"C:\Users\alice\AppData\Local"),
                ),
                (
                    &format!(
                        r"Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2\{guid}"
                    ),
                    "BaseClass",
                    REG_DWORD,
                    0u32.to_le_bytes().to_vec(),
                ),
            ],
        );
        std::fs::write(&ntuser, hive).unwrap();
        sources.push(ntuser.to_string_lossy().to_string());
    }
    let records = usb_read(&["-f", dir.path().join("SYSTEM").to_str().unwrap()]);
    let volume = records.iter().find(|r| r["VolumeGUID"] == guid).unwrap();
    let users = volume["User"].as_array().unwrap();
    assert_eq!(users.len(), 2);
    assert!(users.iter().all(|u| u["User"] == "alice"));
    let paths: std::collections::BTreeSet<_> = users
        .iter()
        .map(|u| u["SourcePath"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(paths, sources.into_iter().collect());
}

#[cfg(unix)]
#[test]
fn registry_usb_does_not_discover_user_hives_through_external_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    usb_collection(dir.path());
    let outside = tempfile::tempdir().unwrap();
    let ntuser = dir.path().join("C/Users/bob/NTUSER.DAT");
    std::fs::copy(&ntuser, outside.path().join("NTUSER.DAT")).unwrap();
    let link = dir.path().join("C/Users/outside-profile");
    std::os::unix::fs::symlink(outside.path(), &link).unwrap();
    let rows = usb_read(&["-d", dir.path().to_str().unwrap()]);
    assert!(
        !rows.iter().any(
            |r| r["User"].as_array().unwrap().iter().any(|u| u["SourcePath"]
                .as_str()
                .is_some_and(|p| p.contains("outside-profile")))
        )
    );
}

// --- parse-registry bam -------------------------------------------------------------------------

/// `parse-registry bam`'s CSV columns.
const BAM_COLUMNS: [&str; 18] = [
    "Host",
    "SourcePath",
    "ControlSet",
    "IsCurrentControlSet",
    "Moderator",
    "UserSID",
    "User",
    "UserSource",
    "Program",
    "DevicePath",
    "Path",
    "LastRunUTC",
    "KeyPath",
    "KeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// BAM's data: the time a program last ran, then 16 bytes Windows writes after it.
fn bam_data(when: u64) -> Vec<u8> {
    let mut data = when.to_le_bytes().to_vec();
    data.extend([0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0]);
    data
}

/// One system's SYSTEM hive (its name; BAM and DAM entries of alice, SYSTEM and a domain
/// administrator in two control sets and both layouts, a value cut short) and its SOFTWARE hive
/// (alice's and the administrator's profiles).
fn bam_collection(root: &Path) {
    let alice = r"ControlSet001\Services\bam\State\UserSettings\S-1-5-21-1-2-3-1001";
    let system_user = r"ControlSet001\Services\bam\State\UserSettings\S-1-5-18";
    let dam = r"ControlSet001\Services\dam\State\UserSettings\S-1-5-21-1-2-3-1001";
    let admin = r"ControlSet002\Services\bam\UserSettings\S-1-5-21-9-9-9-500";
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
            (alice, "Version", REG_DWORD, 1u32.to_le_bytes().to_vec()),
            (
                alice,
                "SequenceNumber",
                REG_DWORD,
                7u32.to_le_bytes().to_vec(),
            ),
            (
                alice,
                r"\Device\HarddiskVolume3\Users\alice\Downloads\tool.exe",
                REG_BINARY,
                bam_data(LINKED),
            ),
            (
                alice,
                "Microsoft.WindowsCalculator_8wekyb3d8bbwe",
                REG_BINARY,
                bam_data(LINKED - DAY),
            ),
            (
                alice,
                r"\Device\HarddiskVolume3\Users\alice\cut.exe",
                REG_BINARY,
                vec![1, 2, 3],
            ),
            (
                system_user,
                r"\Device\HarddiskVolume3\Windows\System32\oobe\FirstLogonAnim.exe",
                REG_BINARY,
                bam_data(LINKED - 2 * DAY),
            ),
            (
                dam,
                r"\Device\HarddiskVolume3\Windows\notepad.exe",
                REG_BINARY,
                bam_data(LINKED + DAY),
            ),
            (
                admin,
                r"\Device\HarddiskVolume2\Tools\old.exe",
                REG_BINARY,
                bam_data(LINKED - 3 * DAY),
            ),
        ],
    );
    let software = reg_tree(
        "ROOT",
        &[
            (
                r"Microsoft\Windows NT\CurrentVersion\ProfileList\S-1-5-21-1-2-3-1001",
                "ProfileImagePath",
                REG_EXPAND_SZ,
                utf16z(r"C:\Users\alice"),
            ),
            (
                r"Microsoft\Windows NT\CurrentVersion\ProfileList\S-1-5-21-9-9-9-500",
                "ProfileImagePath",
                REG_EXPAND_SZ,
                utf16z(r"C:\Users\Administrator.CORP"),
            ),
        ],
    );
    let config = root.join("C/Windows/System32/config");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("SYSTEM"), system).unwrap();
    std::fs::write(config.join("SOFTWARE"), software).unwrap();
}

/// `parse-registry bam` with `args`, as JSON Lines.
fn bam_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "bam"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_bam_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    bam_collection(dir.path());
    let records = bam_read(&["-d", dir.path().to_str().unwrap()]);
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} {} {} ({}) {} | {} | {}",
                r["Moderator"].as_str().unwrap(),
                r["UserSID"].as_str().unwrap(),
                r["User"].as_str().unwrap_or("-"),
                r["UserSource"].as_str().unwrap_or("-"),
                r["Program"].as_str().unwrap(),
                r["Path"].as_str().unwrap_or("-"),
                r["LastRun"].as_str().unwrap_or("-"),
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            r"bam S-1-5-18 SYSTEM (well_known) \Device\HarddiskVolume3\Windows\System32\oobe\FirstLogonAnim.exe | \Windows\System32\oobe\FirstLogonAnim.exe | 2022-02-06T21:03:52Z",
            r"bam S-1-5-21-1-2-3-1001 alice (profile_list) \Device\HarddiskVolume3\Users\alice\Downloads\tool.exe | \Users\alice\Downloads\tool.exe | 2022-02-08T21:03:52Z",
            "bam S-1-5-21-1-2-3-1001 alice (profile_list) Microsoft.WindowsCalculator_8wekyb3d8bbwe | - | 2022-02-07T21:03:52Z",
            r"bam S-1-5-21-1-2-3-1001 alice (profile_list) \Device\HarddiskVolume3\Users\alice\cut.exe | \Users\alice\cut.exe | -",
            r"dam S-1-5-21-1-2-3-1001 alice (profile_list) \Device\HarddiskVolume3\Windows\notepad.exe | \Windows\notepad.exe | 2022-02-09T21:03:52Z",
        ]
    );
    for r in &records {
        assert_eq!(r["Host"], "HOST1");
        assert_eq!(r["ControlSet"], "ControlSet001");
        assert_eq!(r["IsCurrentControlSet"], true);
        assert_eq!(r["TransactionLogStatus"], "clean");
        assert_eq!(r["KeyLastWritten"], "2022-02-08T21:03:52Z");
    }
    assert_eq!(records[1]["DevicePath"], r"\Device\HarddiskVolume3");
    assert_eq!(records[2]["DevicePath"], Value::Null);
    assert_eq!(
        records[1]["KeyPath"],
        r"ControlSet001\Services\bam\State\UserSettings\S-1-5-21-1-2-3-1001"
    );
    assert_eq!(
        records[4]["KeyPath"],
        r"ControlSet001\Services\dam\State\UserSettings\S-1-5-21-1-2-3-1001"
    );
    // the value cut short keeps its bytes, and no time is guessed
    assert_eq!(records[3]["DataRaw"], "010203");
    assert_eq!(
        records[3]["Problems"],
        json!(["3 data bytes, fewer than the 8 of a time it last ran"])
    );
    assert_eq!(records[1]["DataRaw"], Value::Null);

    // every control set: the domain administrator's entry in ControlSet002's older layout, named
    // from its profile folder
    let records = bam_read(&["-d", dir.path().to_str().unwrap(), "--all-control-sets"]);
    assert_eq!(records.len(), 6);
    let old = &records[5];
    assert_eq!(
        (
            &old["ControlSet"],
            &old["IsCurrentControlSet"],
            &old["User"],
            &old["KeyPath"]
        ),
        (
            &json!("ControlSet002"),
            &json!(false),
            &json!("Administrator.CORP"),
            &json!(r"ControlSet002\Services\bam\UserSettings\S-1-5-21-9-9-9-500")
        )
    );
    assert_eq!(old["DevicePath"], r"\Device\HarddiskVolume2");
}

#[test]
fn registry_bam_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    bam_collection(dir.path());
    let out = dir.path().join("bam.csv");
    let r = run(&[
        "parse-registry",
        "bam",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "SYSTEM hives parsed: 1",
        "Hosts: HOST1",
        "Control sets read: 1 (the one in use; --all-control-sets for every one)",
        "Entries: 5 (bam 4, dam 1)",
        "HOST1  S-1-5-18  SYSTEM  1",
        "HOST1  S-1-5-21-1-2-3-1001  alice  4",
        r"2022-02-09T21:03:52Z  alice  \Device\HarddiskVolume3\Windows\notepad.exe (dam)",
        r"2022-02-08T21:03:52Z  alice  \Device\HarddiskVolume3\Users\alice\Downloads\tool.exe",
        "Entries or keys that could not be read: 1",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    // the most recent first
    let notepad = r.stdout.find("notepad.exe (dam)").unwrap();
    let tool = r.stdout.find(r"Downloads\tool.exe").unwrap();
    assert!(notepad < tool);
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let header: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, BAM_COLUMNS);
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(rows.len(), 5);
    assert_eq!(&rows[1][4], "bam");
    assert_eq!(&rows[1][6], "alice");
    assert_eq!(&rows[1][9], r"\Device\HarddiskVolume3");
    assert_eq!(&rows[1][10], r"\Users\alice\Downloads\tool.exe");
    assert_eq!(&rows[1][11], "2022-02-08T21:03:52Z");
    assert_eq!(&rows[3][14], "010203");
}

#[test]
fn registry_bam_of_public_hives() {
    // the DFIR Artifact Museum's dirty Windows 11 SYSTEM with its logs, and its SAM: the entries
    // regipy's and Plaso's bam read, local users named from the SAM
    let hive = system_hive();
    let config = hive.path().join("HOST/C/Windows/System32/config");
    unpack_registry("rathbun_w11", &["SAM"], &config);
    let records = bam_read(&["-f", config.join("SYSTEM").to_str().unwrap()]);
    assert_eq!(records.len(), 19);
    assert!(records.iter().all(|r| r["Problems"] == json!([])
        && r["Moderator"] == "bam"
        && r["ControlSet"] == "ControlSet001"));
    let named = |sid_end: &str| {
        records
            .iter()
            .filter(|r| r["UserSID"].as_str().unwrap().ends_with(sid_end))
            .map(|r| format!("{} {}", r["User"], r["UserSource"]))
            .collect::<std::collections::BTreeSet<_>>()
    };
    assert_eq!(
        named("-632671957-1001"),
        [r#""AndrewRathbun" "sam""#.to_string()].into()
    );
    // a RID the SAM no longer has, and no ProfileList to name it
    assert_eq!(named("-632671957-1000"), ["null null".to_string()].into());
    assert_eq!(
        named("S-1-5-90-0-2"),
        [r#""DWM-2" "well_known""#.to_string()].into()
    );
    let cmd = records
        .iter()
        .find(|r| r["Path"] == r"\Windows\System32\cmd.exe")
        .unwrap();
    assert_eq!(cmd["DevicePath"], r"\Device\HarddiskVolume3");
    assert_eq!(cmd["LastRun"], "2022-02-07T07:38:40.992105700Z");
    assert_eq!(cmd["TransactionLogStatus"], "replayed");
    assert_eq!(
        records
            .iter()
            .filter(|r| r["DevicePath"] == Value::Null)
            .count(),
        8
    );
}

#[test]
fn registry_bam_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&["parse-registry", "bam", "-f", sam.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("not a SYSTEM hive"), "{said}");
    let r = run(&[
        "parse-registry",
        "bam",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("No SYSTEM hives were found"), "{said}");
    // the output is never one of the files read: the system's SAM included
    let collection = tempfile::tempdir().unwrap();
    bam_collection(collection.path());
    let config = collection.path().join("C/Windows/System32/config");
    std::fs::copy(&sam, config.join("SAM")).unwrap();
    let original = std::fs::read(config.join("SAM")).unwrap();
    let r = run(&[
        "parse-registry",
        "bam",
        "-f",
        config.join("SYSTEM").to_str().unwrap(),
        "-o",
        config.join("SAM").to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(config.join("SAM")).unwrap(), original);
}

#[test]
fn registry_bam_preserves_out_of_range_times_and_rejects_wrong_hive_types() {
    let dir = tempfile::tempdir().unwrap();
    let bytes = reg_tree(
        "ROOT",
        &[
            ("Select", "Current", REG_DWORD, 1u32.to_le_bytes().to_vec()),
            (
                r"ControlSet001\Services\bam\State\UserSettings\S-1-5-18",
                r"\Device\HarddiskVolume1\bad.exe",
                REG_BINARY,
                bam_data(u64::MAX),
            ),
        ],
    );
    let system = dir.path().join("SYSTEM");
    std::fs::write(&system, &bytes).unwrap();
    let rows = bam_read(&["-f", system.to_str().unwrap()]);
    assert_eq!(rows.len(), 1);
    assert!(rows[0]["LastRun"].is_null());
    assert_eq!(
        rows[0]["DataRaw"],
        bam_data(u64::MAX)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    assert!(
        rows[0]["Problems"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains("FILETIME"))
    );
    let sam = dir.path().join("SAM");
    std::fs::write(&sam, &bytes).unwrap();
    let result = run(&[
        "parse-registry",
        "bam",
        "-f",
        sam.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(result.code, 1);
    assert!(result.stderr.contains("not a SYSTEM hive"));
}

#[test]
fn registry_bam_recent_summary_keeps_identical_activity_on_distinct_hosts() {
    let dir = tempfile::tempdir().unwrap();
    for host in ["HOST_A", "HOST_B"] {
        let root = dir.path().join(host);
        std::fs::create_dir(&root).unwrap();
        let bytes = reg_tree(
            "ROOT",
            &[
                ("Select", "Current", REG_DWORD, 1u32.to_le_bytes().to_vec()),
                (
                    r"ControlSet001\Control\ComputerName\ComputerName",
                    "ComputerName",
                    REG_SZ,
                    utf16z(host),
                ),
                (
                    r"ControlSet001\Services\bam\State\UserSettings\S-1-5-18",
                    r"\Device\HarddiskVolume1\same.exe",
                    REG_BINARY,
                    bam_data(LINKED),
                ),
            ],
        );
        std::fs::write(root.join("SYSTEM"), bytes).unwrap();
    }
    let result = run(&[
        "parse-registry",
        "bam",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(result.code, 0, "{}", result.stderr);
    let recent = result
        .stdout
        .split("Run last (UTC, user, program):")
        .nth(1)
        .unwrap()
        .split("Entries or keys")
        .next()
        .unwrap();
    assert_eq!(recent.matches("same.exe").count(), 2, "{recent}");
    assert!(
        recent.contains("HOST_A") && recent.contains("HOST_B"),
        "{recent}"
    );
}

// --- parse-registry shell-bags --------------------------------------------------------------------

/// `parse-registry shell-bags`'s CSV columns.
const SHELLBAGS_COLUMNS: [&str; 26] = [
    "Host",
    "User",
    "UserSID",
    "SourcePath",
    "HiveType",
    "BagKey",
    "MRU-Position",
    "IsMostRecentInParent",
    "NodeSlot",
    "Path",
    "ItemType",
    "ItemClass",
    "ItemName",
    "GUID",
    "ItemModifiedUTC",
    "ItemCreatedUTC",
    "ItemAccessedUTC",
    "MFT-Entry",
    "MFT-Sequence",
    "ExtensionBlocks",
    "KeyLastWrittenUTC",
    "ParentKeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// `parse-registry shell-bags` with `args`, as JSON Lines.
fn shellbags_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "shell-bags"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

/// regipy's dirty `UsrClass.dat` and its logs, where Windows keeps jcloudy's.
fn shellbags_collection(root: &Path) -> std::path::PathBuf {
    let dir = [
        "C",
        "Users",
        "jcloudy",
        "AppData",
        "Local",
        "Microsoft",
        "Windows",
    ]
    .iter()
    .fold(root.to_path_buf(), |dir, part| dir.join(part));
    unpack_registry(
        "regipy",
        &["UsrClass.dat", "UsrClass.dat.LOG1", "UsrClass.dat.LOG2"],
        &dir,
    );
    dir.join("UsrClass.dat")
}

const BAG_MRU: &str = r"Local Settings\Software\Microsoft\Windows\Shell\BagMRU";

#[test]
fn registry_shellbags_of_a_public_hive() {
    let dir = tempfile::tempdir().unwrap();
    let hive = shellbags_collection(dir.path());
    let records = shellbags_read(&["-d", dir.path().to_str().unwrap()]);
    assert_eq!(records.len(), 38);
    assert_eq!(
        records
            .iter()
            .filter(|r| r["TransactionLogStatus"] == "replayed")
            .count(),
        24
    );
    for r in &records {
        assert_eq!(r["User"], "jcloudy");
        assert_eq!(
            r["UserSID"],
            "S-1-5-21-2734969515-1644526556-1039763013-1001"
        );
        assert_eq!(r["HiveType"], "UsrClass.dat");
        assert_eq!(r["SourcePath"], hive.to_str().unwrap());
        assert_eq!(r["Problems"], json!([]));
    }
    let at = |key: &str| {
        records
            .iter()
            .find(|r| r["BagKey"] == format!(r"{BAG_MRU}\{key}"))
            .unwrap()
    };
    // This PC, opened last: its parent key's time dates it.
    let this_pc = &records[0];
    assert_eq!(this_pc["BagKey"], format!(r"{BAG_MRU}\4"));
    assert_eq!(
        (&this_pc["Path"], &this_pc["ItemType"], &this_pc["GUID"]),
        (
            &json!("This PC"),
            &json!("root_folder"),
            &json!("{20D04FE0-3AEA-1069-A2D8-08002B30309D}")
        )
    );
    assert_eq!(
        (
            &this_pc["MRU-Position"],
            &this_pc["IsMostRecentInParent"],
            &this_pc["NodeSlot"]
        ),
        (&json!(0), &json!(true), &json!(7))
    );
    assert_eq!(
        this_pc["ParentKeyLastWritten"],
        "2018-04-06T12:42:02.778956500Z"
    );
    // A folder: its long name, FAT times and NTFS reference from its shell item.
    let box_sync = at(r"4\3\0\0\3");
    assert_eq!(box_sync["Path"], r"This PC\C:\Users\jcloudy\Box Sync");
    assert_eq!(
        (
            &box_sync["ItemType"],
            &box_sync["ItemClass"],
            &box_sync["MRU-Position"]
        ),
        (&json!("directory"), &json!("0x31"), &json!(1))
    );
    assert_eq!(
        (
            &box_sync["ItemModified"],
            &box_sync["ItemCreated"],
            &box_sync["ItemAccessed"]
        ),
        (
            &json!("2018-04-05T02:11:16Z"),
            &json!("2018-03-28T00:53:58Z"),
            &json!("2018-04-05T02:11:16Z")
        )
    );
    assert_eq!(
        (
            &box_sync["MFT-Entry"],
            &box_sync["MFT-Sequence"],
            &box_sync["ExtensionBlocks"]
        ),
        (&json!(140782), &json!(2), &json!(1))
    );
    assert_eq!(box_sync["KeyLastWritten"], "2018-04-05T02:12:11.004647700Z");
    assert_eq!(box_sync["TransactionLogStatus"], "unchanged");
    assert!(box_sync["DataRaw"].is_null());
    // Removable drives (D: in the hive, F: only in its logs) and search results.
    for (key, name) in [("1", "D:"), ("10", "F:")] {
        let drive = at(key);
        assert_eq!(
            (&drive["Path"], &drive["ItemType"], &drive["GUID"]),
            (
                &json!(name),
                &json!("volume"),
                &json!("{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")
            )
        );
    }
    assert_eq!(at(r"1\0\0\0")["Path"], r"D:\AKMonitor\logs\pic");
    let search = at("3");
    assert_eq!(
        (&search["Path"], &search["ItemType"]),
        (&json!("Search Folder"), &json!("property_view"))
    );

    // as the hive is, without its logs
    let records = shellbags_read(&["-f", hive.to_str().unwrap(), "--no-logs"]);
    assert_eq!(records.len(), 29);
    assert!(
        records
            .iter()
            .all(|r| r["TransactionLogStatus"] == "not_applied")
    );
}

#[test]
fn registry_shellbags_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    shellbags_collection(dir.path());
    let out = dir.path().join("shellbags.csv");
    let r = run(&[
        "parse-registry",
        "shell-bags",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "UsrClass.dat and NTUSER.DAT hives parsed: 1",
        "Users: jcloudy",
        "Dirty hives recovered with their transaction logs: 1",
        "Items: 38 (control_panel_category 2, control_panel_item 2, directory 15, property_view 3, root_folder 10, users_files_folder 3, volume 3)",
        "Opened last (UTC from the parent key's last write, user, folder):",
        r"2018-04-06T12:42:02.778956500Z  jcloudy  This PC\Downloads",
        "Items or keys that could not be read: 0",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let header: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, SHELLBAGS_COLUMNS);
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(rows.len(), 38);
    assert_eq!(&rows[0][9], "This PC");
    assert_eq!(&rows[0][7], "true");
}

/// An `NTUSER.DAT` with Windows XP's `ShellNoRoam\BagMRU`: This PC, C:, a folder whose class no
/// parser knows, and a value its list names but the key hasn't.
fn shellbags_ntuser() -> Vec<u8> {
    let bag = r"Software\Microsoft\Windows\ShellNoRoam\BagMRU";
    let mru = |list: &[u32]| -> Vec<u8> {
        list.iter()
            .chain(&[u32::MAX])
            .flat_map(|n| n.to_le_bytes())
            .collect()
    };
    let mut this_pc = vec![0x14, 0, 0x1f, 0x50];
    this_pc.extend([
        0xe0, 0x4f, 0xd0, 0x20, 0xea, 0x3a, 0x69, 0x10, 0xa2, 0xd8, 0x08, 0x00, 0x2b, 0x30, 0x30,
        0x9d, 0, 0,
    ]);
    let mut volume = vec![0x19, 0, 0x2f];
    volume.extend(b"C:\\");
    volume.resize(0x19 + 2, 0);
    let unknown = vec![6, 0, 0x99, 1, 2, 3, 0, 0];
    reg_tree(
        "ROOT",
        &[
            (bag, "MRUListEx", REG_BINARY, mru(&[0])),
            (bag, "0", REG_BINARY, this_pc),
            (&format!(r"{bag}\0"), "MRUListEx", REG_BINARY, mru(&[0, 1])),
            (&format!(r"{bag}\0"), "0", REG_BINARY, volume),
            (
                &format!(r"{bag}\0\0"),
                "NodeSlot",
                REG_DWORD,
                3u32.to_le_bytes().to_vec(),
            ),
            (&format!(r"{bag}\0\0"), "MRUListEx", REG_BINARY, mru(&[0])),
            (&format!(r"{bag}\0\0"), "0", REG_BINARY, unknown),
        ],
    )
}

#[test]
fn registry_shellbags_of_windows_xp_and_damage() {
    let dir = tempfile::tempdir().unwrap();
    let profile = dir.path().join("Documents and Settings").join("bob");
    std::fs::create_dir_all(&profile).unwrap();
    std::fs::write(profile.join("NTUSER.DAT"), shellbags_ntuser()).unwrap();
    let records = shellbags_read(&["-d", dir.path().to_str().unwrap()]);
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} | {} | {} | {} | {}",
                r["BagKey"].as_str().unwrap(),
                r["Path"].as_str().unwrap(),
                r["ItemType"].as_str().unwrap_or("-"),
                r["NodeSlot"],
                r["Problems"]
            )
        })
        .collect();
    let bag = r"Software\Microsoft\Windows\ShellNoRoam\BagMRU";
    assert_eq!(
        lines,
        [
            format!(r#"{bag}\0 | This PC | root_folder | null | []"#),
            format!(r#"{bag}\0\0 | This PC\C: | volume | 3 | []"#),
            format!(r#"{bag}\0\0\0 | This PC\C:\[class 0x99] | unknown | null | []"#),
            format!(
                r#"{bag}\0\1 | This PC\[unknown] | - | null | ["MRUListEx lists value 1, which is not there"]"#
            ),
        ]
    );
    assert!(
        records
            .iter()
            .all(|r| r["User"] == "bob" && r["HiveType"] == "NTUSER.DAT")
    );
    // the unknown item keeps its bytes
    assert_eq!(records[2]["DataRaw"], "0600990102030000");
    assert_eq!(records[2]["ItemClass"], "0x99");

    // a SAM hive is not read
    unpack_registry("regipy", &["SAM"], dir.path());
    let r = run(&[
        "parse-registry",
        "shell-bags",
        "-f",
        dir.path().join("SAM").to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        format!("{}{}", r.stdout, r.stderr)
            .contains("a SAM hive, not a UsrClass.dat or NTUSER.DAT hive"),
        "{}{}",
        r.stdout,
        r.stderr
    );
}

#[test]
fn registry_shellbags_never_guesses_an_ambiguous_or_backup_profile_sid() {
    let dir = tempfile::tempdir().unwrap();
    let user = dir.path().join("C/Users/alice");
    let config = dir.path().join("C/Windows/System32/config");
    std::fs::create_dir_all(&user).unwrap();
    std::fs::create_dir_all(&config).unwrap();
    let location = r"Software\Microsoft\Windows\Shell\BagMRU";
    let volume = [
        vec![0x19, 0, 0x2f],
        b"C:\\".to_vec(),
        vec![0; 19],
        vec![0, 0],
    ]
    .concat();
    let hive = reg_tree(
        "ROOT",
        &[
            (
                location,
                "MRUListEx",
                REG_BINARY,
                [0u32.to_le_bytes(), u32::MAX.to_le_bytes()].concat(),
            ),
            (location, "0", REG_BINARY, volume),
        ],
    );
    let ntuser = user.join("NTUSER.DAT");
    std::fs::write(&ntuser, hive).unwrap();
    let profile = r"Microsoft\Windows NT\CurrentVersion\ProfileList";
    for sids in [
        vec!["S-1-5-21-1-2-3-1001", "S-1-5-21-4-5-6-1001"],
        vec!["S-1-5-21-1-2-3-1001.bak"],
    ] {
        let paths: Vec<_> = sids.iter().map(|sid| format!("{profile}\\{sid}")).collect();
        let values: Vec<_> = paths
            .iter()
            .map(|p| {
                (
                    p.as_str(),
                    "ProfileImagePath",
                    REG_SZ,
                    utf16z(r"C:\Users\alice"),
                )
            })
            .collect();
        std::fs::write(config.join("SOFTWARE"), reg_tree("ROOT", &values)).unwrap();
        let rows = shellbags_read(&["-f", ntuser.to_str().unwrap()]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["User"], "alice");
        assert!(rows[0]["UserSID"].is_null(), "{rows:?}");
    }
}

// --- parse-registry muicache --------------------------------------------------------------------

/// `parse-registry muicache`'s CSV columns.
const MUICACHE_COLUMNS: [&str; 15] = [
    "Host",
    "User",
    "UserSID",
    "SourcePath",
    "HiveType",
    "Path",
    "FriendlyAppName",
    "AppCompany",
    "KeyPath",
    "KeyLastWrittenUTC",
    "FriendlyAppNameRaw",
    "AppCompanyRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// One system's SYSTEM and SOFTWARE hives (its name, alice's and bob's profiles); alice's
/// UsrClass.dat with MuiCache in the Windows 10 form (a download and cmd.exe), the Windows 7 form
/// (a program on a removable drive) and a value that isn't a string; her NTUSER.DAT, without
/// MuiCache; and bob's Windows XP NTUSER.DAT with a resource string.
fn muicache_collection(root: &Path) {
    let mui = r"Local Settings\Software\Microsoft\Windows\Shell\MuiCache";
    let xp = r"Software\Microsoft\Windows\ShellNoRoam\MUICache";
    let tool = r"C:\Users\alice\Downloads\tool.exe";
    let cmd = r"C:\Windows\System32\cmd.exe";
    let classes = reg_tree(
        "S-1-5-21-1-2-3-1001_Classes",
        &[
            (mui, "LangID", REG_BINARY, vec![9, 4]),
            (
                mui,
                &format!("{tool}.FriendlyAppName"),
                REG_SZ,
                utf16z("Tool Setup"),
            ),
            (
                mui,
                &format!("{tool}.ApplicationCompany"),
                REG_SZ,
                utf16z("Example Ltd"),
            ),
            (
                mui,
                &format!("{cmd}.FriendlyAppName"),
                REG_SZ,
                utf16z("Windows Command Processor"),
            ),
            (
                mui,
                &format!("{cmd}.ApplicationCompany"),
                REG_SZ,
                utf16z("Microsoft Corporation"),
            ),
            (mui, r"E:\run.exe", REG_SZ, utf16z("Run")),
            (
                mui,
                r"C:\Users\alice\bad.exe.FriendlyAppName",
                REG_BINARY,
                vec![1, 2, 3],
            ),
        ],
    );
    let alice = reg_tree(
        "ROOT",
        &[(
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
            "Start_TrackProgs",
            REG_DWORD,
            1u32.to_le_bytes().to_vec(),
        )],
    );
    let bob = reg_tree(
        "ROOT",
        &[
            (xp, "LangID", REG_BINARY, vec![9, 4]),
            (xp, "@shell32.dll,-22019", REG_SZ, utf16z("Calculator")),
            (
                xp,
                r"C:\WINDOWS\system32\notepad.exe",
                REG_SZ,
                utf16z("Notepad"),
            ),
        ],
    );
    let list = r"Microsoft\Windows NT\CurrentVersion\ProfileList";
    let software = reg_tree(
        "ROOT",
        &[
            (
                &format!(r"{list}\S-1-5-21-1-2-3-1001"),
                "ProfileImagePath",
                REG_EXPAND_SZ,
                utf16z(r"C:\Users\alice"),
            ),
            (
                &format!(r"{list}\S-1-5-21-1-2-3-1002"),
                "ProfileImagePath",
                REG_EXPAND_SZ,
                utf16z(r"C:\Documents and Settings\bob"),
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
        ],
    );
    let config = root.join("C/Windows/System32/config");
    let windows = root.join("C/Users/alice/AppData/Local/Microsoft/Windows");
    let bob_dir = root.join("C/Documents and Settings/bob");
    for dir in [&config, &windows, &bob_dir] {
        std::fs::create_dir_all(dir).unwrap();
    }
    std::fs::write(config.join("SOFTWARE"), software).unwrap();
    std::fs::write(config.join("SYSTEM"), system).unwrap();
    std::fs::write(windows.join("UsrClass.dat"), classes).unwrap();
    std::fs::write(root.join("C/Users/alice/NTUSER.DAT"), alice).unwrap();
    std::fs::write(bob_dir.join("NTUSER.DAT"), bob).unwrap();
}

/// `parse-registry muicache` with `args`, as JSON Lines.
fn muicache_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "muicache"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_muicache_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    muicache_collection(dir.path());
    let records = muicache_read(&["-d", dir.path().to_str().unwrap()]);
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} {} {} | {} | {} | {}",
                r["User"].as_str().unwrap(),
                r["UserSID"].as_str().unwrap(),
                r["HiveType"].as_str().unwrap(),
                r["Path"].as_str().unwrap(),
                r["FriendlyAppName"].as_str().unwrap_or("-"),
                r["AppCompany"].as_str().unwrap_or("-"),
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            r"bob S-1-5-21-1-2-3-1002 NTUSER.DAT | C:\WINDOWS\system32\notepad.exe | Notepad | -",
            r"alice S-1-5-21-1-2-3-1001 UsrClass.dat | C:\Users\alice\Downloads\tool.exe | Tool Setup | Example Ltd",
            r"alice S-1-5-21-1-2-3-1001 UsrClass.dat | C:\Windows\System32\cmd.exe | Windows Command Processor | Microsoft Corporation",
            r"alice S-1-5-21-1-2-3-1001 UsrClass.dat | E:\run.exe | Run | -",
            r"alice S-1-5-21-1-2-3-1001 UsrClass.dat | C:\Users\alice\bad.exe | - | -",
        ]
    );
    for r in &records {
        assert_eq!(r["Host"], "HOST1");
        assert_eq!(r["TransactionLogStatus"], "clean");
        assert_eq!(r["KeyLastWritten"], "2022-02-08T21:03:52Z");
    }
    assert_eq!(
        records[0]["KeyPath"],
        r"Software\Microsoft\Windows\ShellNoRoam\MUICache"
    );
    assert_eq!(
        records[1]["KeyPath"],
        r"Local Settings\Software\Microsoft\Windows\Shell\MuiCache"
    );
    // the value that isn't a string keeps its bytes
    assert_eq!(records[4]["FriendlyAppNameRaw"], "010203");
    assert_eq!(records[4]["AppCompanyRaw"], Value::Null);
    assert_eq!(
        records[4]["Problems"],
        json!(["FriendlyAppName: is REG_BINARY, not a string"])
    );
    assert!(records[..4].iter().all(|r| r["Problems"] == json!([])));
}

#[test]
fn registry_muicache_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    muicache_collection(dir.path());
    let out = dir.path().join("muicache.csv");
    let r = run(&[
        "parse-registry",
        "muicache",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "UsrClass.dat and NTUSER.DAT hives parsed: 3",
        "Hosts: HOST1",
        "Users: alice, bob",
        "Hives without MuiCache (NTUSER.DAT of Windows Vista and later keeps none): 1",
        "Programs: 5",
        "Resource strings left out (@<file>,-<number>): 1",
        "Programs outside the Windows and Program Files folders: 3",
        r"  alice  C:\Users\alice\Downloads\tool.exe (Tool Setup)",
        r"  alice  E:\run.exe (Run)",
        r"  alice  C:\Users\alice\bad.exe",
        "Programs or keys that could not be read: 1",
        "FriendlyAppName: is REG_BINARY, not a string",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    assert!(!r.stdout.contains(r"  alice  C:\Windows\System32\cmd.exe"));
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let header: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, MUICACHE_COLUMNS);
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(rows.len(), 5);
    assert_eq!(&rows[1][5], r"C:\Users\alice\Downloads\tool.exe");
    assert_eq!(&rows[1][6], "Tool Setup");
    assert_eq!(&rows[1][7], "Example Ltd");
    assert_eq!(&rows[4][10], "010203");
}

#[test]
fn registry_muicache_of_public_hives() {
    // the DFIR Artifact Museum's Windows 11 system: its SYSTEM, and the user's dirty
    // UsrClass.dat and NTUSER.DAT with their logs, where they were. The programs and their names
    // are those regipy and dissect.target read from the hive.
    let collection = system_hive();
    let profile = collection.path().join("HOST/C/Users/AndrewRathbun");
    let windows = profile.join("AppData/Local/Microsoft/Windows");
    unpack_registry(
        "rathbun_w11",
        &["UsrClass.dat", "UsrClass.dat.LOG1", "UsrClass.dat.LOG2"],
        &windows,
    );
    unpack_registry(
        "rathbun_w11",
        &["NTUSER.DAT", "ntuser.dat.LOG1", "ntuser.dat.LOG2"],
        &profile,
    );
    let dir = collection.path().to_str().unwrap();
    let records = muicache_read(&["-d", dir]);
    assert_eq!(records.len(), 12);
    for r in &records {
        assert_eq!(r["Host"], "GOLDPAPERW11");
        assert_eq!(r["User"], "AndrewRathbun");
        assert_eq!(
            r["UserSID"],
            "S-1-5-21-4063860464-1508573791-632671957-1001"
        );
        assert_eq!(r["HiveType"], "UsrClass.dat");
        assert_eq!(r["KeyLastWritten"], "2022-02-07T05:41:02.220813100Z");
        assert_eq!(r["TransactionLogStatus"], "unchanged");
        assert_eq!(r["Problems"], json!([]));
    }
    let program = |path: &str| {
        let r = records.iter().find(|r| r["Path"] == path).unwrap();
        (
            r["FriendlyAppName"].as_str().unwrap(),
            r["AppCompany"].as_str().unwrap(),
        )
    };
    assert_eq!(
        program(r"C:\Windows\system32\OpenWith.exe"),
        ("Pick an app", "Microsoft Corporation")
    );
    assert_eq!(
        program(r"D:\setup64.exe"),
        ("VMware installation launcher", "VMware, Inc.")
    );
    assert_eq!(
        program(r"C:\Windows\System32\PickerHost.exe"),
        ("File Picker UI Host", "Microsoft Corporation")
    );
    let r = run(&["parse-registry", "muicache", "-d", dir, "-K"]);
    for expected in [
        "UsrClass.dat and NTUSER.DAT hives parsed: 2",
        "Hosts: GOLDPAPERW11",
        "Dirty hives recovered with their transaction logs: 2",
        "Hives without MuiCache (NTUSER.DAT of Windows Vista and later keeps none): 1",
        "Programs: 12",
        "Programs outside the Windows and Program Files folders: 2",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }

    // regipy's Windows 7 NTUSER.DAT: MuiCache is in its UsrClass.dat
    let regipy = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["NTUSER.DAT"], regipy.path());
    let records = muicache_read(&["-f", regipy.path().join("NTUSER.DAT").to_str().unwrap()]);
    assert!(records.is_empty());
}

#[test]
fn registry_muicache_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&[
        "parse-registry",
        "muicache",
        "-f",
        sam.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(
        said.contains("a SAM hive, not a UsrClass.dat or NTUSER.DAT hive"),
        "{said}"
    );
    let r = run(&[
        "parse-registry",
        "muicache",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(
        said.contains("No UsrClass.dat or NTUSER.DAT hives were found"),
        "{said}"
    );
    // the output is never one of the files read
    let original = std::fs::read(&sam).unwrap();
    let r = run(&[
        "parse-registry",
        "muicache",
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
fn registry_muicache_summary_keeps_relative_and_escaping_paths_visible() {
    let directory = tempfile::tempdir().unwrap();
    let hive = directory.path().join("NTUSER.DAT");
    let key = r"Local Settings\Software\Microsoft\Windows\Shell\MuiCache";
    let paths = [
        r"Windows\relative.exe",
        r"Program Files\relative.exe",
        r"C:\Windows\..\Users\alice\payload.exe",
        r"%windir%\..\Temp\payload.exe",
    ];
    let mut entries = paths
        .iter()
        .map(|path| {
            (
                key.to_string(),
                format!("{path}.FriendlyAppName"),
                REG_SZ,
                utf16z("Example"),
            )
        })
        .collect::<Vec<_>>();
    entries.push((
        key.to_string(),
        r"C:\Windows\System32\notepad.exe.FriendlyAppName".to_string(),
        REG_SZ,
        utf16z("Notepad"),
    ));
    let borrowed = entries
        .iter()
        .map(|(key, name, kind, data)| (key.as_str(), name.as_str(), *kind, data.clone()))
        .collect::<Vec<_>>();
    std::fs::write(&hive, reg_tree("ROOT", &borrowed)).unwrap();
    let result = run(&[
        "parse-registry",
        "muicache",
        "-f",
        hive.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(result.code, 0, "{}", result.stderr);
    assert!(
        result
            .stdout
            .contains("Programs outside the Windows and Program Files folders: 4"),
        "{}",
        result.stdout
    );
    for path in paths {
        assert!(result.stdout.contains(path), "{path}\n{}", result.stdout);
    }
}

// --- parse-registry installed-programs ----------------------------------------------------------

/// `parse-registry installed-programs`' CSV columns.
const INSTALLED_PROGRAMS_COLUMNS: [&str; 30] = [
    "Host",
    "SourcePath",
    "HiveType",
    "User",
    "UserSID",
    "Source",
    "Scope",
    "Name",
    "Version",
    "Publisher",
    "InstallDate",
    "InstallDateRaw",
    "KeyLastWrittenUTC",
    "InstallLocation",
    "InstallSource",
    "Program",
    "UninstallCommand",
    "QuietUninstallCommand",
    "SizeKB",
    "SystemComponent",
    "WindowsInstaller",
    "ProductCode",
    "Architecture",
    "WOW64",
    "Details",
    "KeyPath",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// One system's SYSTEM hive (its name), SOFTWARE hive (alice's profile; an MSI-installed
/// program, one with a date written otherwise and a size written as text, a hidden 32-bit one,
/// its Windows Installer product, an App Path and a package listed for alice) and alice's
/// NTUSER.DAT (a program she installed for herself and a WSL 2 distribution).
fn installed_programs_collection(root: &Path) {
    let cv = r"Microsoft\Windows\CurrentVersion";
    let agent = format!(r"{cv}\Uninstall\{{8E9D8F3C-6B7A-4D5C-3E2F-1A0B9C8D7E6F}}");
    let tool = format!(r"{cv}\Uninstall\Tool");
    let old = format!(r"Wow6432Node\{cv}\Uninstall\Old32");
    let product = r"Classes\Installer\Products\c3f8d9e8a7b6c5d4e3f2a1b0c9d8e7f6";
    let sz = |path: &str, name: &str, text: &str| {
        (path.to_owned(), name.to_owned(), REG_SZ, utf16z(text))
    };
    let dword = |path: &str, name: &str, n: u32| {
        (
            path.to_owned(),
            name.to_owned(),
            REG_DWORD,
            n.to_le_bytes().to_vec(),
        )
    };
    let software: Vec<(String, String, u32, Vec<u8>)> = vec![
        (
            r"Microsoft\Windows NT\CurrentVersion\ProfileList\S-1-5-21-1-2-3-1001".to_owned(),
            "ProfileImagePath".to_owned(),
            REG_EXPAND_SZ,
            utf16z(r"C:\Users\alice"),
        ),
        sz(&agent, "DisplayName", "Agent"),
        sz(&agent, "DisplayVersion", "2.1.0"),
        sz(&agent, "Publisher", "Example Corp"),
        sz(&agent, "InstallDate", "20220208"),
        sz(&agent, "InstallLocation", r"C:\Program Files\Agent\"),
        sz(
            &agent,
            "UninstallString",
            "MsiExec.exe /X{8E9D8F3C-6B7A-4D5C-3E2F-1A0B9C8D7E6F}",
        ),
        dword(&agent, "EstimatedSize", 2048),
        dword(&agent, "WindowsInstaller", 1),
        sz(&tool, "DisplayName", "Tool"),
        sz(&tool, "InstallDate", "2022-02-08"),
        sz(&tool, "EstimatedSize", "12"),
        sz(&tool, "Inno Setup: User", "alice"),
        sz(&old, "DisplayName", "Old"),
        sz(&old, "InstallDate", "20220101"),
        dword(&old, "SystemComponent", 1),
        sz(product, "ProductName", "Agent"),
        dword(product, "Version", 0x0201_0000),
        sz(
            &format!(r"{product}\SourceList"),
            "PackageName",
            "agent.msi",
        ),
        sz(
            &format!(r"{product}\SourceList"),
            "LastUsedSource",
            r"n;1;C:\Users\alice\Downloads\",
        ),
        sz(
            &format!(r"{cv}\App Paths\tool.exe"),
            "",
            r"C:\Tool\tool.exe",
        ),
        sz(&format!(r"{cv}\App Paths\tool.exe"), "Path", r"C:\Tool"),
        sz(
            &format!(
                r"{cv}\Appx\AppxAllUserStore\S-1-5-21-1-2-3-1001\Tool.Package_1.0.0.0_x64__abcdefghijkmn"
            ),
            "Path",
            r"C:\Program Files\WindowsApps\Tool.Package_1.0.0.0_x64__abcdefghijkmn\AppxManifest.xml",
        ),
    ];
    let ucv = r"Software\Microsoft\Windows\CurrentVersion";
    let portable = format!(r"{ucv}\Uninstall\Portable");
    let ubuntu = format!(r"{ucv}\Lxss\{{70E495E1-D4D3-4B8D-BBE8-79FDC01E50EA}}");
    let ntuser: Vec<(String, String, u32, Vec<u8>)> = vec![
        sz(&portable, "DisplayName", "Portable Tool"),
        sz(&portable, "DisplayVersion", "3.0"),
        sz(&portable, "InstallDate", "20220209"),
        sz(
            &portable,
            "UninstallString",
            r"C:\Users\alice\AppData\Local\Portable\uninstall.exe",
        ),
        sz(&ubuntu, "DistributionName", "Ubuntu"),
        dword(&ubuntu, "Flags", 15),
        sz(
            &ubuntu,
            "BasePath",
            r"C:\Users\alice\AppData\Local\Packages\Ubuntu\LocalState",
        ),
    ];
    let tree = |values: &[(String, String, u32, Vec<u8>)]| {
        let values: Vec<(&str, &str, u32, Vec<u8>)> = values
            .iter()
            .map(|(p, n, k, d)| (p.as_str(), n.as_str(), *k, d.clone()))
            .collect();
        reg_tree("ROOT", &values)
    };
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
    let profile = root.join("C/Users/alice");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::create_dir_all(&profile).unwrap();
    std::fs::write(config.join("SYSTEM"), system).unwrap();
    std::fs::write(config.join("SOFTWARE"), tree(&software)).unwrap();
    std::fs::write(profile.join("NTUSER.DAT"), tree(&ntuser)).unwrap();
}

/// `parse-registry installed-programs` with `args`, as JSON Lines.
fn installed_programs_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "installed-programs"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_installed_programs_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    installed_programs_collection(dir.path());
    let records = installed_programs_read(&["-d", dir.path().to_str().unwrap()]);
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} {} {} {} | {} {} | {} | {} | {}",
                r["HiveType"].as_str().unwrap(),
                r["Source"].as_str().unwrap(),
                r["Scope"].as_str().unwrap(),
                r["User"].as_str().unwrap_or("-"),
                r["Name"].as_str().unwrap_or("-"),
                r["Version"].as_str().unwrap_or("-"),
                r["InstallDate"].as_str().unwrap_or("-"),
                r["InstallLocation"].as_str().unwrap_or("-"),
                r["Architecture"].as_str().unwrap_or("-"),
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            // the hives in path order (C/Users before C/Windows), the keys in the hive's (sorted)
            "NTUSER.DAT uninstall user alice | Portable Tool 3.0 | 2022-02-09 | - | -",
            r"NTUSER.DAT wsl user alice | Ubuntu 2 | - | C:\Users\alice\AppData\Local\Packages\Ubuntu\LocalState | -",
            "SOFTWARE uninstall machine - | Tool - | - | - | -",
            r"SOFTWARE uninstall machine - | Agent 2.1.0 | 2022-02-08 | C:\Program Files\Agent\ | -",
            "SOFTWARE uninstall machine - | Old - | 2022-01-01 | - | x86",
            "SOFTWARE msi_product machine - | Agent 2.1.0 | - | - | -",
            r"SOFTWARE app_path machine - | tool.exe - | - | C:\Tool | -",
            r"SOFTWARE package user alice | Tool.Package 1.0.0.0 | - | C:\Program Files\WindowsApps\Tool.Package_1.0.0.0_x64__abcdefghijkmn\AppxManifest.xml | x64",
        ]
    );
    for r in &records {
        assert_eq!(r["Host"], "HOST1");
        assert_eq!(r["TransactionLogStatus"], "clean");
        assert_eq!(r["KeyLastWritten"], "2022-02-08T21:03:52Z");
    }
    let agent = &records[3];
    assert_eq!(
        (
            &agent["ProductCode"],
            &agent["SizeKB"],
            &agent["WindowsInstaller"],
            &agent["Publisher"]
        ),
        (
            &json!("{8E9D8F3C-6B7A-4D5C-3E2F-1A0B9C8D7E6F}"),
            &json!(2048),
            &json!(true),
            &json!("Example Corp")
        )
    );
    assert_eq!(agent["UserSID"], Value::Null);
    assert_eq!(agent["Details"], json!({}));
    assert_eq!(agent["DataRaw"], json!({}));
    // a date written otherwise is kept as written; a size written as text keeps its bytes
    let tool = &records[2];
    assert_eq!(tool["InstallDateRaw"], "2022-02-08");
    assert_eq!(tool["Details"], json!({"Inno Setup: User": "alice"}));
    assert_eq!(tool["DataRaw"], json!({"EstimatedSize": "310032000000"}));
    assert_eq!(
        tool["Problems"],
        json!(["EstimatedSize: is REG_SZ, not REG_DWORD"])
    );
    assert_eq!(
        (&records[4]["WOW64"], &records[4]["SystemComponent"]),
        (&json!(true), &json!(true))
    );
    let product = &records[5];
    assert_eq!(product["InstallSource"], r"C:\Users\alice\Downloads\");
    assert_eq!(
        product["ProductCode"],
        "{8E9D8F3C-6B7A-4D5C-3E2F-1A0B9C8D7E6F}"
    );
    assert_eq!(product["Details"], json!({"PackageName": "agent.msi"}));
    assert_eq!(records[6]["Program"], r"C:\Tool\tool.exe");
    // a package SOFTWARE lists for alice, named from its ProfileList; her own hive's records
    // take her SID from it too
    for r in [&records[0], &records[1], &records[7]] {
        assert_eq!(r["UserSID"], "S-1-5-21-1-2-3-1001");
    }
    assert_eq!(
        records[7]["Details"],
        json!({"List": "User", "PublisherId": "abcdefghijkmn"})
    );
    assert_eq!(records[1]["Details"], json!({"Flags": "0xf"}));
    assert_eq!(
        records[1]["KeyPath"],
        r"Software\Microsoft\Windows\CurrentVersion\Lxss\{70E495E1-D4D3-4B8D-BBE8-79FDC01E50EA}"
    );
}

#[test]
fn registry_installed_programs_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    installed_programs_collection(dir.path());
    let out = dir.path().join("installed.csv");
    let r = run(&[
        "parse-registry",
        "installed-programs",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "Hives parsed: 2 (NTUSER.DAT 1, SOFTWARE 1)",
        "Hosts: HOST1",
        "Users: alice",
        "Records: 8 (app_path 1, msi_product 1, package 1, uninstall 4, wsl 1)",
        "2022-02-09  HOST1  Portable Tool 3.0  [alice]",
        "2022-02-08  HOST1  Agent 2.1.0 (Example Corp)",
        "2022-01-01  HOST1  Old",
        "Installed by users for themselves (NTUSER.DAT Uninstall entries and Windows Installer products): 1",
        "HOST1  alice  Portable Tool 3.0",
        r"HOST1  alice  Ubuntu (WSL 2)  C:\Users\alice\AppData\Local\Packages\Ubuntu\LocalState",
        "Records or keys that could not be read: 1",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    // newest first
    let portable = r.stdout.find("2022-02-09  HOST1").unwrap();
    let agent = r.stdout.find("2022-02-08  HOST1").unwrap();
    let old = r.stdout.find("2022-01-01  HOST1").unwrap();
    assert!(portable < agent && agent < old);
    let (header, rows) = csv_rows(&out);
    assert_eq!(header, INSTALLED_PROGRAMS_COLUMNS);
    assert_eq!(rows.len(), 8);
    assert_eq!(rows[2]["Details"], "Inno Setup: User=alice");
    assert_eq!(rows[2]["DataRaw"], "EstimatedSize=310032000000");
    assert_eq!(rows[2]["InstallDateRaw"], "2022-02-08");
    assert_eq!(rows[3]["SizeKB"], "2048");
    assert_eq!(rows[3]["WindowsInstaller"], "True");
    assert_eq!(rows[3]["WOW64"], "False");
    assert_eq!(rows[4]["Architecture"], "x86");
    assert_eq!(rows[0]["User"], "alice");
    assert_eq!(rows[1]["Details"], "Flags=0xf");
}

#[test]
fn registry_installed_programs_of_public_hives() {
    // regipy's NTUSER-WSL.DAT: three WSL 1 distributions (Ubuntu-20.04 the default), four App
    // Paths and OneDrive, as regipy's wsl, installed_programs_ntuser give them
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["NTUSER-WSL.DAT"], dir.path());
    let hive = dir.path().join("NTUSER.DAT");
    std::fs::rename(dir.path().join("NTUSER-WSL.DAT"), &hive).unwrap();
    let records = installed_programs_read(&["-f", hive.to_str().unwrap()]);
    let counts = records
        .iter()
        .fold(std::collections::BTreeMap::new(), |mut m, r| {
            *m.entry(r["Source"].as_str().unwrap().to_owned())
                .or_insert(0) += 1;
            m
        });
    assert_eq!(
        counts,
        std::collections::BTreeMap::from([
            ("app_path".to_owned(), 4),
            ("uninstall".to_owned(), 1),
            ("wsl".to_owned(), 3)
        ])
    );
    let wsl: Vec<String> = records
        .iter()
        .filter(|r| r["Source"] == "wsl")
        .map(|r| {
            format!(
                "{} {} {}",
                r["Name"].as_str().unwrap(),
                r["Version"].as_str().unwrap(),
                r["Details"]["Default"].as_str().unwrap_or("-")
            )
        })
        .collect();
    assert_eq!(wsl, ["Debian 1 -", "Ubuntu 1 -", "Ubuntu-20.04 1 true"]);
    let onedrive = &records[0];
    assert_eq!(
        (&onedrive["Name"], &onedrive["Version"], &onedrive["SizeKB"]),
        (
            &json!("Microsoft OneDrive"),
            &json!("21.220.1024.0005"),
            &json!(174_061)
        )
    );
    assert!(records.iter().all(|r| r["Problems"] == json!([])
        && r["Scope"] == "user"
        && r["HiveType"] == "NTUSER.DAT"));

    // the DFIR Artifact Museum's dirty Windows 11 NTUSER.DAT, with its logs
    let dir = tempfile::tempdir().unwrap();
    unpack_registry(
        "rathbun_w11",
        &["NTUSER.DAT", "ntuser.dat.LOG1", "ntuser.dat.LOG2"],
        dir.path(),
    );
    let records = installed_programs_read(&["-f", dir.path().join("NTUSER.DAT").to_str().unwrap()]);
    let names: Vec<&str> = records
        .iter()
        .map(|r| r["Name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "Microsoft OneDrive",
            "mspaint.exe",
            "msteams.exe",
            "notepad.exe",
            "OneDriveFileLauncher.exe",
            "wt.exe"
        ]
    );
    assert_eq!(records[0]["Version"], "22.002.0103.0004");
    assert_eq!(
        records[0]["KeyLastWritten"],
        "2022-02-06T21:36:22.847509700Z"
    );
    assert_eq!(
        records[5]["Program"],
        r"C:\Program Files\WindowsApps\Microsoft.WindowsTerminal_1.11.3471.0_x64__8wekyb3d8bbwe\wt.exe"
    );
    assert!(
        records
            .iter()
            .all(|r| r["TransactionLogStatus"] == "unchanged" && r["Problems"] == json!([]))
    );
}

#[test]
fn registry_installed_programs_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&[
        "parse-registry",
        "installed-programs",
        "-f",
        sam.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("not a SOFTWARE or NTUSER.DAT hive"), "{said}");
    let r = run(&[
        "parse-registry",
        "installed-programs",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(
        said.contains("No SOFTWARE or NTUSER.DAT hives were found"),
        "{said}"
    );
    // the output is never one of the files read: a user hive's system's SOFTWARE included
    let collection = tempfile::tempdir().unwrap();
    installed_programs_collection(collection.path());
    let software = collection.path().join("C/Windows/System32/config/SOFTWARE");
    let original = std::fs::read(&software).unwrap();
    let r = run(&[
        "parse-registry",
        "installed-programs",
        "-f",
        collection
            .path()
            .join("C/Users/alice/NTUSER.DAT")
            .to_str()
            .unwrap(),
        "-o",
        software.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(&software).unwrap(), original);
}

#[test]
fn registry_installed_programs_resolves_profiles_with_trailing_separators() {
    let directory = tempfile::tempdir().unwrap();
    let hive = directory.path().join("SOFTWARE");
    let sid = "S-1-5-21-1-2-3-1001";
    let profile = format!(r"Microsoft\Windows NT\CurrentVersion\ProfileList\{sid}");
    let package = format!(
        r"Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{sid}\Example.App_1.2.3.4_x64__8wekyb3d8bbwe"
    );
    for folder in [r"C:\Users\alice\", "C:/Users/alice/"] {
        std::fs::write(
            &hive,
            reg_tree(
                "ROOT",
                &[
                    (&profile, "ProfileImagePath", REG_SZ, utf16z(folder)),
                    (
                        &package,
                        "Path",
                        REG_SZ,
                        utf16z(r"C:\Program Files\WindowsApps\Example.App"),
                    ),
                ],
            ),
        )
        .unwrap();
        let rows = installed_programs_read(&["-f", hive.to_str().unwrap()]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["User"], "alice", "{folder}");
        assert_eq!(rows[0]["UserSID"], sid);
    }
}

#[test]
fn registry_installed_programs_data_stdout_reports_hive_damage_on_stderr() {
    let directory = tempfile::tempdir().unwrap();
    let hive = directory.path().join("SOFTWARE");
    let mut bytes = reg_tree(
        "ROOT",
        &[
            (
                r"Microsoft\Windows\CurrentVersion\Uninstall\GoodProgram",
                "DisplayName",
                REG_SZ,
                utf16z("Good program"),
            ),
            (
                r"Microsoft\Windows\CurrentVersion\Uninstall\DamagedProgram",
                "DisplayName",
                REG_SZ,
                utf16z("Damaged program"),
            ),
        ],
    );
    let name = b"DamagedProgram";
    let at = bytes
        .windows(name.len())
        .position(|data| data == name)
        .unwrap();
    let cell = at - 0x4c;
    assert_eq!(&bytes[cell..cell + 2], b"nk");
    bytes[cell..cell + 2].copy_from_slice(b"xx");
    std::fs::write(&hive, bytes).unwrap();
    let result = run(&[
        "parse-registry",
        "installed-programs",
        "-f",
        hive.to_str().unwrap(),
        "-t",
        "jsonl",
        "-K",
    ]);
    assert_eq!(result.code, 0, "{}", result.stderr);
    let rows = result
        .stdout
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["Name"], "Good program");
    assert!(result.stderr.contains("Uninstall"), "{}", result.stderr);
}

// --- parse-registry open-save-mru -----------------------------------------------------------------

/// `parse-registry open-save-mru`'s CSV columns.
const OPENSAVEMRU_COLUMNS: [&str; 24] = [
    "Host",
    "User",
    "UserSID",
    "SourcePath",
    "List",
    "Extension",
    "MRU-Position",
    "IsMostRecent",
    "ValueName",
    "Program",
    "Path",
    "ItemType",
    "ItemModifiedUTC",
    "ItemCreatedUTC",
    "ItemAccessedUTC",
    "MFT-Entry",
    "MFT-Sequence",
    "LastUsedUTC",
    "KeyPath",
    "KeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// The common dialogs' key, under the hive's root.
const COMDLG32: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32";

/// A shell item list of `items` (each with its size), ended by a size of 0.
fn shell_list(items: &[Vec<u8>]) -> Vec<u8> {
    let mut out: Vec<u8> = items.concat();
    out.extend([0, 0]);
    out
}

/// This PC, as a root folder item.
fn this_pc() -> Vec<u8> {
    let mut item = vec![0x14, 0x00, 0x1f, 0x50];
    // {20D04FE0-3AEA-1069-A2D8-08002B30309D}
    item.extend([
        0xe0, 0x4f, 0xd0, 0x20, 0xea, 0x3a, 0x69, 0x10, 0xa2, 0xd8, 0x08, 0x00, 0x2b, 0x30, 0x30,
        0x9d,
    ]);
    item
}

/// The volume `C:\`.
fn volume_c() -> Vec<u8> {
    let mut item = vec![0x19, 0x00, 0x2f];
    item.extend(b"C:\\");
    item.resize(0x19, 0);
    item
}

/// A file entry (`0x31` a folder, `0x32` a file) with its 8.3 name and, in a version 9
/// `0xBEEF0004` block, its long name and NTFS file reference; modified 2022-02-08 12:00:00,
/// created the same, accessed 2022-02-09 12:00:00.
fn file_entry(class: u8, short: &str, long: &str, mft: (u64, u16)) -> Vec<u8> {
    let fat = |day: u16| {
        let date = ((2022 - 1980) << 9) | (2 << 5) | day;
        let time: u16 = 12 << 11;
        [date.to_le_bytes(), time.to_le_bytes()].concat()
    };
    let mut item = vec![class, 0];
    item.extend(0u32.to_le_bytes());
    item.extend(fat(8));
    item.extend(0x10u16.to_le_bytes());
    item.extend(short.as_bytes());
    item.push(0);
    if item.len() % 2 == 1 {
        item.push(0);
    }
    // where the block starts in the item, with the item's size
    let block_at = item.len() + 2;
    let mut block = vec![0, 0];
    block.extend(9u16.to_le_bytes());
    block.extend(0xbeef_0004u32.to_le_bytes());
    block.extend(fat(8));
    block.extend(fat(9));
    block.extend(0x2eu16.to_le_bytes());
    block.extend(0u16.to_le_bytes());
    block.extend((mft.0 | (u64::from(mft.1) << 48)).to_le_bytes());
    block.extend([0; 8 + 2 + 4 + 4]);
    block.extend(utf16z(long));
    block.extend((block_at as u16).to_le_bytes());
    let size = block.len() as u16;
    block[..2].copy_from_slice(&size.to_le_bytes());
    item.extend(block);
    let mut out = ((item.len() + 2) as u16).to_le_bytes().to_vec();
    out.extend(item);
    out
}

/// A program's name (UTF-16, with its NUL), then `rest`.
fn program_then(program: &str, rest: &[u8]) -> Vec<u8> {
    let mut b = utf16z(program);
    b.extend(rest);
    b
}

/// One system's SYSTEM and SOFTWARE hives (its name, alice's profile), alice's NTUSER.DAT with
/// the common dialogs' history (files opened and saved, the programs and their folders, an item
/// of a type the shell item parser doesn't know, a value cut short, Windows XP's lists), and
/// bob's NTUSER.DAT without it.
fn opensavemru_collection(root: &Path) {
    let at = |list: &str| format!(r"{COMDLG32}\{list}");
    let report = shell_list(&[
        this_pc(),
        volume_c(),
        file_entry(0x31, "Users", "Users", (512, 1)),
        file_entry(0x32, "REPORT~1.DOC", "report 2022.docx", (98_765, 3)),
    ]);
    let notes = shell_list(&[
        this_pc(),
        volume_c(),
        file_entry(0x32, "notes.txt", "notes.txt", (77, 2)),
    ]);
    // a class 0x52 item: not one ys-lnk-parser knows
    let odd = shell_list(&[this_pc(), vec![0x08, 0x00, 0x52, 1, 2, 3, 4, 5]]);
    let users = shell_list(&[
        this_pc(),
        volume_c(),
        file_entry(0x31, "Users", "Users", (512, 1)),
    ]);
    let mru = |list: &[u32]| -> Vec<u8> {
        list.iter()
            .chain(&[u32::MAX])
            .flat_map(|n| n.to_le_bytes())
            .collect()
    };
    let mut folder = utf16z(r"C:\Program Files\Office\WINWORD.EXE");
    folder.extend(utf16z(r"C:\Users\alice\Documents"));
    let mut xp_folder = utf16z("notepad.exe");
    xp_folder.extend(utf16z(r"C:\Temp"));
    let alice = reg_tree(
        "ROOT",
        &[
            (
                &at(r"OpenSavePidlMRU\*"),
                "MRUListEx",
                REG_BINARY,
                mru(&[1, 0, 2]),
            ),
            (&at(r"OpenSavePidlMRU\*"), "0", REG_BINARY, notes.clone()),
            (&at(r"OpenSavePidlMRU\*"), "1", REG_BINARY, report.clone()),
            (&at(r"OpenSavePidlMRU\*"), "2", REG_BINARY, odd),
            (
                &at(r"OpenSavePidlMRU\docx"),
                "MRUListEx",
                REG_BINARY,
                mru(&[0]),
            ),
            (&at(r"OpenSavePidlMRU\docx"), "0", REG_BINARY, report),
            (
                &at("LastVisitedPidlMRU"),
                "MRUListEx",
                REG_BINARY,
                mru(&[0, 1, 2]),
            ),
            (
                &at("LastVisitedPidlMRU"),
                "0",
                REG_BINARY,
                program_then("WINWORD.EXE", &users),
            ),
            (
                &at("LastVisitedPidlMRU"),
                "1",
                REG_BINARY,
                program_then("notepad.exe", &[0, 0]),
            ),
            // cut short: no NUL after the program's name
            (
                &at("LastVisitedPidlMRU"),
                "2",
                REG_BINARY,
                utf16z("cut.exe")[..6].to_vec(),
            ),
            (&at("CIDSizeMRU"), "MRUListEx", REG_BINARY, mru(&[0])),
            (
                &at("CIDSizeMRU"),
                "0",
                REG_BINARY,
                program_then("WINWORD.EXE", &[0; 16]),
            ),
            (&at("FirstFolder"), "MRUListEx", REG_BINARY, mru(&[0])),
            (&at("FirstFolder"), "0", REG_BINARY, folder),
            (&at(r"OpenSaveMRU\txt"), "MRUList", REG_SZ, utf16z("a")),
            (&at(r"OpenSaveMRU\txt"), "a", REG_SZ, utf16z(r"C:\old.txt")),
            (&at("LastVisitedMRU"), "MRUList", REG_SZ, utf16z("a")),
            (&at("LastVisitedMRU"), "a", REG_BINARY, xp_folder),
        ],
    );
    let bob = reg_tree(
        "ROOT",
        &[(
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
            "Start_TrackProgs",
            REG_DWORD,
            1u32.to_le_bytes().to_vec(),
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

/// `parse-registry open-save-mru` with `args`, as JSON Lines.
fn opensavemru_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "open-save-mru"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_opensavemru_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    opensavemru_collection(dir.path());
    let records = opensavemru_read(&["-d", dir.path().to_str().unwrap()]);
    let text = |v: &Value| v.as_str().unwrap_or("-").to_string();
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} {} {} {} {} | {} | {} | {}",
                text(&r["List"]),
                text(&r["Extension"]),
                r["MRU-Position"],
                text(&r["ValueName"]),
                r["IsMostRecent"],
                text(&r["Program"]),
                text(&r["Path"]),
                text(&r["ItemType"]),
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            r"OpenSavePidlMRU * 0 1 true | - | This PC\C:\Users\report 2022.docx | file",
            r"OpenSavePidlMRU * 1 0 false | - | This PC\C:\notes.txt | file",
            r"OpenSavePidlMRU * 2 2 false | - | This PC\[class 0x52] | unknown",
            r"OpenSavePidlMRU docx 0 0 true | - | This PC\C:\Users\report 2022.docx | file",
            r"LastVisitedPidlMRU - 0 0 true | WINWORD.EXE | This PC\C:\Users | directory",
            r"LastVisitedPidlMRU - 1 1 false | notepad.exe |  | -",
            r"LastVisitedPidlMRU - 2 2 false | - | - | -",
            r"CIDSizeMRU - 0 0 true | WINWORD.EXE | - | -",
            r"FirstFolder - 0 0 true | C:\Program Files\Office\WINWORD.EXE | C:\Users\alice\Documents | -",
            r"OpenSaveMRU txt 0 a true | - | C:\old.txt | -",
            r"LastVisitedMRU - 0 a true | notepad.exe | C:\Temp | -",
        ]
    );
    for r in &records {
        assert_eq!(r["Host"], "HOST1");
        assert_eq!(r["User"], "alice");
        assert_eq!(r["UserSID"], "S-1-5-21-1-2-3-1001");
        assert_eq!(r["TransactionLogStatus"], "clean");
        assert_eq!(r["KeyLastWritten"], "2022-02-08T21:03:52Z");
        // the key's time dates its most recent entry only
        let last_used = if r["IsMostRecent"] == true {
            json!("2022-02-08T21:03:52Z")
        } else {
            Value::Null
        };
        assert_eq!(r["LastUsed"], last_used, "{r}");
    }
    // the file's own times and NTFS file reference, from its shell item
    let report = &records[0];
    assert_eq!(
        (
            &report["ItemModified"],
            &report["ItemCreated"],
            &report["ItemAccessed"],
            &report["MFT-Entry"],
            &report["MFT-Sequence"]
        ),
        (
            &json!("2022-02-08T12:00:00Z"),
            &json!("2022-02-08T12:00:00Z"),
            &json!("2022-02-09T12:00:00Z"),
            &json!(98_765),
            &json!(3)
        )
    );
    assert_eq!(report["KeyPath"], format!(r"{COMDLG32}\OpenSavePidlMRU\*"));
    assert!(report["DataRaw"].is_null());
    assert_eq!(records[4]["MFT-Entry"], 512);
    // an item of an unknown type keeps the list's bytes; a value cut short keeps its own
    let odd = records[2]["DataRaw"].as_str().unwrap();
    assert!(
        odd.starts_with("14001f50") && odd.ends_with("08005201020304050000"),
        "{odd}"
    );
    assert_eq!(records[2]["Problems"], json!([]));
    assert_eq!(records[6]["DataRaw"], "630075007400");
    assert_eq!(
        records[6]["Problems"],
        json!(["its program's name has no NUL: the data is cut short"])
    );
}

#[test]
fn registry_opensavemru_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    opensavemru_collection(dir.path());
    let out = dir.path().join("opensavemru.csv");
    let r = run(&[
        "parse-registry",
        "open-save-mru",
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
        "Hives without ComDlg32: 1",
        "Entries: 11 (OpenSavePidlMRU 4, LastVisitedPidlMRU 3, CIDSizeMRU 1, FirstFolder 1, OpenSaveMRU 1, LastVisitedMRU 1)",
        "Extensions: 2",
        "Programs that used the dialogs: winword.exe (3), notepad.exe (2)",
        r"NTUSER.DAT: This PC\C:\Users\report 2022.docx (2022-02-08T21:03:52Z, OpenSavePidlMRU)",
        "Entries with shell items of an unknown type (bytes in DataRaw): 1",
        "Entries or lists that could not be read: 1",
        "its program's name has no NUL: the data is cut short",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let header: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, OPENSAVEMRU_COLUMNS);
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(rows.len(), 11);
    assert_eq!(&rows[0][4], "OpenSavePidlMRU");
    assert_eq!(&rows[0][5], "*");
    assert_eq!(&rows[0][7], "true");
    assert_eq!(&rows[0][10], r"This PC\C:\Users\report 2022.docx");
    assert_eq!(&rows[0][15], "98765");
    assert_eq!(&rows[0][17], "2022-02-08T21:03:52Z");
    assert_eq!(&rows[1][17], "");
    assert_eq!(
        &rows[6][23],
        "its program's name has no NUL: the data is cut short"
    );
}

#[test]
fn registry_opensavemru_of_a_public_hive() {
    // Plaso's NTUSER.DAT: the entries, order and programs Plaso's mrulistex plugins give, and
    // the files' names in their shell item paths
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("plaso", &["NTUSER.DAT"], dir.path());
    let records = opensavemru_read(&["-f", dir.path().join("NTUSER.DAT").to_str().unwrap()]);
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} {} {} {} | {} | {}",
                r["List"].as_str().unwrap(),
                r["Extension"].as_str().unwrap_or("-"),
                r["MRU-Position"],
                r["ValueName"].as_str().unwrap(),
                r["Program"].as_str().unwrap_or("-"),
                r["Path"].as_str().unwrap_or("-"),
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            r"OpenSavePidlMRU * 0 1 | - | This PC\CLSID_ThisPCMyPicturesRegFolder\ash_0.png",
            r"OpenSavePidlMRU * 1 0 | - | This PC\CLSID_ThisPCMyPicturesRegFolder\ash.png",
            r"OpenSavePidlMRU png 0 1 | - | This PC\CLSID_ThisPCMyPicturesRegFolder\ash_0.png",
            r"OpenSavePidlMRU png 1 0 | - | This PC\CLSID_ThisPCMyPicturesRegFolder\ash.png",
            r"LastVisitedPidlMRU - 0 0 | iexplore.exe | This PC\CLSID_ThisPCMyPicturesRegFolder",
            "CIDSizeMRU - 0 0 | iexplore.exe | -",
            r"FirstFolder - 0 0 | C:\Program Files\Internet Explorer\iexplore.exe | -",
        ]
    );
    assert_eq!(records[0]["User"], "bperry");
    assert_eq!(records[0]["ItemType"], "file");
    assert_eq!(records[0]["LastUsed"], "2016-10-09T19:59:17.734597Z");
    assert_eq!(records[4]["ItemType"], "root_folder");
    assert!(records.iter().all(|r| r["Problems"] == json!([])
        && r["DataRaw"].is_null()
        && r["TransactionLogStatus"] == "clean"));
}

#[test]
fn registry_opensavemru_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM", "NTUSER.DAT"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&[
        "parse-registry",
        "open-save-mru",
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
    // a hive without ComDlg32 is parsed, with nothing to report
    let ntuser = dir.path().join("NTUSER.DAT");
    let r = run(&[
        "parse-registry",
        "open-save-mru",
        "-f",
        ntuser.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(
        r.stdout.contains("Hives without ComDlg32: 1"),
        "{}",
        r.stdout
    );
    let r = run(&[
        "parse-registry",
        "open-save-mru",
        "-f",
        ntuser.to_str().unwrap(),
        "--code-page",
        "1",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("Unsupported code page: 1"), "{said}");
    let empty = tempfile::tempdir().unwrap();
    let r = run(&[
        "parse-registry",
        "open-save-mru",
        "-d",
        empty.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("No NTUSER.DAT hives were found"), "{said}");
    // the output is never one of the files read
    let original = std::fs::read(&ntuser).unwrap();
    let r = run(&[
        "parse-registry",
        "open-save-mru",
        "-f",
        ntuser.to_str().unwrap(),
        "-o",
        ntuser.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(&ntuser).unwrap(), original);
}

#[test]
fn registry_opensavemru_keeps_key_warnings_on_stderr_for_stdout_formats() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("NTUSER.DAT");
    let at = format!(r"{COMDLG32}\OpenSavePidlMRU\*");
    let bytes = reg_tree(
        "ROOT",
        &[
            // Its readable prefix lists value 0, but the MRUListEx has no end marker.
            (&at, "MRUListEx", REG_BINARY, vec![0, 0, 0, 0]),
            (&at, "0", REG_BINARY, shell_list(&[this_pc(), volume_c()])),
        ],
    );
    std::fs::write(&hive, bytes).unwrap();
    for format in ["csv", "json", "jsonl"] {
        let r = run(&[
            "parse-registry",
            "open-save-mru",
            "-f",
            hive.to_str().unwrap(),
            "-t",
            format,
        ]);
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert!(
            r.stderr.contains("MRUListEx") && r.stderr.contains("end marker"),
            "{}",
            r.stderr
        );
        assert!(!r.stdout.contains("MRUListEx:"), "{}", r.stdout);
        if format == "csv" {
            assert_eq!(
                csv::Reader::from_reader(r.stdout.as_bytes())
                    .records()
                    .count(),
                1
            );
        } else {
            assert_eq!(json_documents(&r.stdout).len(), 1);
        }
    }
}

#[test]
fn registry_opensavemru_decodes_nested_zip_items_with_their_parents() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("NTUSER.DAT");
    let at = format!(r"{COMDLG32}\OpenSavePidlMRU\*");
    let zipped = |name: &str| {
        let mut raw = vec![0u8; 92];
        raw[2] = 0x52;
        raw[84..88].copy_from_slice(&(name.encode_utf16().count() as u32).to_le_bytes());
        raw.extend(utf16z(name));
        let len = raw.len() as u16;
        raw[..2].copy_from_slice(&len.to_le_bytes());
        raw
    };
    let bytes = reg_tree(
        "ROOT",
        &[
            (
                &at,
                "MRUListEx",
                REG_BINARY,
                vec![0, 0, 0, 0, 255, 255, 255, 255],
            ),
            (
                &at,
                "0",
                REG_BINARY,
                shell_list(&[
                    this_pc(),
                    volume_c(),
                    file_entry(0x32, "bundle.zip", "bundle.zip", (98_765, 3)),
                    zipped("docs"),
                    zipped("note.txt"),
                ]),
            ),
        ],
    );
    std::fs::write(&hive, bytes).unwrap();
    let records = opensavemru_read(&["-f", hive.to_str().unwrap()]);
    assert_eq!(records.len(), 1);
    assert!(
        records[0]["Path"]
            .as_str()
            .unwrap()
            .ends_with(r"bundle.zip\docs\note.txt"),
        "{}",
        records[0]
    );
    assert_eq!(records[0]["ItemType"], "compressed_folder");
    assert_eq!(records[0]["DataRaw"], Value::Null);
}

#[test]
fn registry_opensavemru_damaged_first_letter_does_not_date_the_next_file() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("NTUSER.DAT");
    let at = format!(r"{COMDLG32}\OpenSaveMRU\txt");
    let order: Vec<u8> = [0xd800, 0x0061, 0x0000]
        .into_iter()
        .flat_map(u16::to_le_bytes)
        .collect();
    let bytes = reg_tree(
        "ROOT",
        &[
            (&at, "MRUList", REG_SZ, order),
            (&at, "a", REG_SZ, utf16z(r"C:\report.txt")),
        ],
    );
    std::fs::write(&hive, bytes).unwrap();
    let r = run(&[
        "parse-registry",
        "open-save-mru",
        "-f",
        hive.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let records = jsonl_records(&r.stdout);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["MRU-Position"], 1);
    assert_eq!(records[0]["IsMostRecent"], false);
    assert_eq!(records[0]["LastUsed"], Value::Null);
    assert!(r.stderr.contains("not valid UTF-16"), "{}", r.stderr);
}

// --- parse-registry remote-access ---------------------------------------------------------------

/// `parse-registry remote-access`'s CSV columns.
const REMOTE_ACCESS_COLUMNS: [&str; 25] = [
    "Host",
    "User",
    "UserSID",
    "SourcePath",
    "Client",
    "Kind",
    "Target",
    "Port",
    "RemoteUsername",
    "Protocol",
    "MRU-Position",
    "SessionName",
    "HostKeyType",
    "HostKeyFingerprint",
    "HostKey",
    "DriveLetter",
    "PasswordStored",
    "Details",
    "KeyPath",
    "ValueName",
    "KeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// A generated Ed25519 host key in PuTTY's `SshHostKeys` form, and its fingerprint as OpenSSH's
/// `ssh-keygen -l` gives it.
const ED25519_HOST_KEY: &str = "0x1dc1835f6c2412ce0471e8e93f9bc28c107b658a7c09d1b77cf8328a9727bcbf,0x6a13e584ed673b1b8640774399fbb872c00e7ff64f4e24ea5870315b315b7fd3";
const ED25519_FINGERPRINT: &str = "SHA256:FHMF6w9trn7uMSI+bN+z7+Biyw+8SJgBfXa9ie8UtuY";

/// One system (HOST1, alice's profile) and alice's `NTUSER.DAT` with a connection of each kind: two
/// Remote Desktop computers and a damaged MRU value, the account used on one, a PuTTY session
/// through a proxy with its password and its host key, a WinSCP session with a stored password,
/// a mapped drive, the path typed to map it, and its share.
fn remote_access_collection(root: &Path) {
    let tsc = r"Software\Microsoft\Terminal Server Client";
    let default = format!(r"{tsc}\Default");
    let server = format!(r"{tsc}\Servers\server01.example.com");
    let web = r"Software\SimonTatham\PuTTY\Sessions\web%20server";
    let keys = r"Software\SimonTatham\PuTTY\SshHostKeys";
    let files = r"Software\Martin Prikryl\WinSCP 2\Sessions\alice@files.example.com";
    let mru = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Map Network Drive MRU";
    let share = r"Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2\##fs01#share";
    let dword = |n: u32| n.to_le_bytes().to_vec();
    let alice = reg_tree(
        "ROOT",
        &[
            (&default, "MRU0", REG_SZ, utf16z("server01.example.com")),
            (&default, "MRU1", REG_SZ, utf16z("10.0.0.9:3390")),
            (&default, "MRU2", REG_BINARY, vec![1, 2, 3]),
            (&server, "UsernameHint", REG_SZ, utf16z(r"EXAMPLE\alice")),
            (web, "HostName", REG_SZ, utf16z("admin@203.0.113.10")),
            (web, "PortNumber", REG_DWORD, dword(22)),
            (web, "Protocol", REG_SZ, utf16z("ssh")),
            (web, "ProxyMethod", REG_DWORD, dword(3)),
            (web, "ProxyHost", REG_SZ, utf16z("proxy.example.com")),
            (web, "ProxyPassword", REG_SZ, utf16z("not output")),
            (
                keys,
                "ssh-ed25519@22:203.0.113.10",
                REG_SZ,
                utf16z(ED25519_HOST_KEY),
            ),
            (files, "HostName", REG_SZ, utf16z("files.example.com")),
            (files, "UserName", REG_SZ, utf16z("alice")),
            (files, "FSProtocol", REG_DWORD, dword(2)),
            (files, "Password", REG_SZ, utf16z("A35C4E")),
            (r"Network\Z", "RemotePath", REG_SZ, utf16z(r"\\fs01\share")),
            (r"Network\Z", "UserName", REG_DWORD, dword(0)),
            (
                r"Network\Z",
                "ProviderName",
                REG_SZ,
                utf16z("Microsoft Windows Network"),
            ),
            (mru, "MRUList", REG_SZ, utf16z("a")),
            (mru, "a", REG_SZ, utf16z(r"\\fs01\share")),
            (share, "_LabelFromReg", REG_SZ, utf16z("Team")),
        ],
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
            ("Select", "Current", REG_DWORD, dword(1)),
            (
                r"ControlSet001\Control\ComputerName\ComputerName",
                "ComputerName",
                REG_SZ,
                utf16z("HOST1"),
            ),
        ],
    );
    let config = root.join("C/Windows/System32/config");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::create_dir_all(root.join("C/Users/alice")).unwrap();
    std::fs::write(config.join("SOFTWARE"), software).unwrap();
    std::fs::write(config.join("SYSTEM"), system).unwrap();
    std::fs::write(root.join("C/Users/alice/NTUSER.DAT"), alice).unwrap();
}

/// `parse-registry remote-access` with `args`, as JSON Lines.
fn remote_access_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "remote-access"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

/// Each record as `client kind target:port user | position | drive`.
fn remote_access_lines(records: &[Value]) -> Vec<String> {
    records
        .iter()
        .map(|r| {
            let mut line = format!(
                "{} {} {}",
                r["Client"].as_str().unwrap(),
                r["Kind"].as_str().unwrap(),
                r["Target"].as_str().unwrap_or("-")
            );
            if let Some(port) = r["Port"].as_u64() {
                line += &format!(":{port}");
            }
            if let Some(user) = r["RemoteUsername"].as_str() {
                line += &format!(" {user}");
            }
            if let Some(n) = r["MRU-Position"].as_u64() {
                line += &format!(" | {n}");
            }
            if let Some(drive) = r["DriveLetter"].as_str() {
                line += &format!(" | {drive}:");
            }
            line
        })
        .collect()
}

#[test]
fn registry_remote_access_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    remote_access_collection(dir.path());
    let records = remote_access_read(&["-d", dir.path().to_str().unwrap()]);
    assert_eq!(
        remote_access_lines(&records),
        [
            "rdp rdp_mru server01.example.com | 0",
            "rdp rdp_mru 10.0.0.9:3390 | 1",
            "rdp rdp_mru - | 2",
            r"rdp rdp_server server01.example.com EXAMPLE\alice",
            "putty putty_session 203.0.113.10:22 admin",
            "putty putty_host_key 203.0.113.10:22",
            "winscp winscp_session files.example.com alice",
            r"mapped_drive network_drive \\fs01\share | Z:",
            r"mapped_drive map_network_drive_mru \\fs01\share | 0",
            r"network_share mount_point \\fs01\share",
        ]
    );
    for r in &records {
        assert_eq!(r["Host"], "HOST1");
        assert_eq!(r["User"], "alice");
        assert_eq!(r["UserSID"], "S-1-5-21-1-2-3-1001");
        assert_eq!(r["TransactionLogStatus"], "clean");
        assert_eq!(r["KeyLastWritten"], "2022-02-08T21:03:52Z");
    }
    // the damaged MRU value keeps its bytes
    assert_eq!(records[2]["DataRaw"], "010203");
    assert_eq!(
        records[2]["Problems"],
        json!(["is REG_BINARY, not a string"])
    );
    assert_eq!(records[2]["ValueName"], "MRU2");
    // PuTTY: the session's proxy and that it keeps the proxy's password, not the password
    let session = &records[4];
    assert_eq!(
        (
            &session["SessionName"],
            &session["Protocol"],
            &session["PasswordStored"]
        ),
        (&json!("web server"), &json!("ssh"), &json!(true))
    );
    assert_eq!(
        session["Details"],
        json!({
            "ProxyMethod": "3 (http)",
            "ProxyHost": "proxy.example.com",
            "StoredPasswords": "ProxyPassword"
        })
    );
    assert!(!session.to_string().contains("not output"));
    let key = &records[5];
    assert_eq!(key["HostKeyType"], "ssh-ed25519");
    assert_eq!(key["HostKeyFingerprint"], ED25519_FINGERPRINT);
    assert_eq!(key["HostKey"], ED25519_HOST_KEY);
    assert_eq!(key["ValueName"], "ssh-ed25519@22:203.0.113.10");
    let winscp = &records[6];
    assert_eq!(
        (&winscp["Protocol"], &winscp["PasswordStored"]),
        (&json!("sftp"), &json!(true))
    );
    assert!(!winscp.to_string().contains("A35C4E"));
    assert_eq!(records[7]["Protocol"], "Microsoft Windows Network");
    assert_eq!(records[9]["Details"], json!({"Label": "Team"}));
    assert_eq!(
        records[9]["KeyPath"],
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2\##fs01#share"
    );
}

#[test]
fn registry_remote_access_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    remote_access_collection(dir.path());
    let out = dir.path().join("remote.csv");
    let r = run(&[
        "parse-registry",
        "remote-access",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "NTUSER.DAT hives parsed: 1",
        "Hosts: HOST1",
        "Users: alice",
        "Connections: 10 (mapped_drive 2, network_share 1, putty 2, rdp 4, winscp 1)",
        "HOST1  2022-02-08T21:03:52Z  alice  rdp  server01.example.com",
        "HOST1  2022-02-08T21:03:52Z  alice  putty  203.0.113.10:22",
        "Sessions with a stored password (not decoded): 2",
        "HOST1  alice  putty  web server (ProxyPassword)",
        "HOST1  alice  winscp  alice@files.example.com (Password)",
        "Keys or values that could not be read: 1",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    // only a list's first item has the list's time as its own
    assert!(!r.stdout.contains("10.0.0.9"), "{}", r.stdout);
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let header: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, REMOTE_ACCESS_COLUMNS);
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(rows.len(), 10);
    assert_eq!(&rows[4][4], "putty");
    assert_eq!(&rows[4][6], "203.0.113.10");
    assert_eq!(&rows[4][16], "True");
    assert_eq!(
        &rows[4][17],
        "ProxyMethod=3 (http); ProxyHost=proxy.example.com; StoredPasswords=ProxyPassword"
    );
    assert_eq!(&rows[5][13], ED25519_FINGERPRINT);
    assert_eq!(&rows[7][15], "Z");
    assert_eq!(&rows[2][21], "010203");
}

#[test]
fn registry_remote_access_of_public_hives() {
    // notatin's Windows 7 NTUSER.DAT: Remote Desktop, a PuTTY and a WinSCP host key, a WinSCP
    // session with a stored password, mapped drives and shares; what regipy's and Plaso's plugins
    // read
    let dir = tempfile::tempdir().unwrap();
    let profile = dir.path().join("C/Users/jmroberts");
    unpack_registry("notatin", &["win7_ntuser.dat"], &profile);
    let records = remote_access_read(&["-f", profile.join("win7_ntuser.dat").to_str().unwrap()]);
    assert_eq!(records.len(), 32);
    let by_client = |client: &str| records.iter().filter(|r| r["Client"] == client).count();
    assert_eq!(
        [
            by_client("rdp"),
            by_client("putty"),
            by_client("winscp"),
            by_client("mapped_drive"),
            by_client("network_share")
        ],
        [10, 1, 3, 11, 7]
    );
    assert!(records.iter().all(|r| r["Problems"] == json!([])
        && r["User"] == "jmroberts"
        && r["TransactionLogStatus"] == "clean"));
    let lines = remote_access_lines(&records);
    assert_eq!(lines[2], "rdp rdp_mru 10.150.16.30:63389 | 2");
    assert_eq!(
        lines[9],
        r"rdp rdp_server nyfkws0009015 strozllc.com\jmroberts"
    );
    assert_eq!(
        records[5]["Details"]["CertHash"],
        "f0cab303aeef9d77871d07953bcb60dc86e0f5e0"
    );
    assert_eq!(
        records[5]["KeyLastWritten"],
        "2014-12-12T19:03:26.217191100Z"
    );
    for key in [&records[10], &records[13]] {
        assert_eq!(
            key["HostKeyFingerprint"],
            "SHA256:NIWlL38s6iTDvDtHtTXpRipSrsGvuFRtIXjJS2JU7WQ"
        );
    }
    assert_eq!(records[12]["SessionName"], "malware@10.1.200.20");
    assert_eq!(records[12]["PasswordStored"], true);
    assert_eq!(
        records[12]["Details"]["LocalDirectory"],
        r"C:\Users\jmroberts\Desktop"
    );
    assert_eq!(
        lines[18],
        r"mapped_drive map_network_drive_mru \\NYFKWS0009015\Scratchy | 0"
    );

    // regipy's NTUSER.DAT: one drive mapped, and its share
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["NTUSER.DAT"], dir.path());
    let records = remote_access_read(&["-f", dir.path().join("NTUSER.DAT").to_str().unwrap()]);
    assert_eq!(
        remote_access_lines(&records),
        [
            r"mapped_drive network_drive \\controller\public | p:",
            r"network_share mount_point \\controller\public",
        ]
    );
}

#[test]
fn registry_remote_access_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&[
        "parse-registry",
        "remote-access",
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
        "remote-access",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("No NTUSER.DAT hives were found"), "{said}");
    // the output is never one of the files read: the user's hive, nor its system's SOFTWARE
    let collection = tempfile::tempdir().unwrap();
    remote_access_collection(collection.path());
    let ntuser = collection.path().join("C/Users/alice/NTUSER.DAT");
    let software = collection.path().join("C/Windows/System32/config/SOFTWARE");
    for target in [&ntuser, &software] {
        let original = std::fs::read(target).unwrap();
        let r = run(&[
            "parse-registry",
            "remote-access",
            "-f",
            ntuser.to_str().unwrap(),
            "-o",
            target.to_str().unwrap(),
            "-C",
            "-K",
        ]);
        assert_eq!(r.code, 1);
        assert_eq!(std::fs::read(target).unwrap(), original);
    }
}

#[test]
fn registry_remote_access_keeps_password_bytes_private_in_all_stdout_formats() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("NTUSER.DAT");
    let canary = b"private-canary-never-export";
    let tree = reg_tree(
        "ROOT",
        &[
            (
                r"Software\SimonTatham\PuTTY\Sessions\test",
                "ProxyPassword",
                REG_BINARY,
                canary.to_vec(),
            ),
            (
                r"Software\Martin Prikryl\WinSCP 2\Sessions\test",
                "Password",
                REG_BINARY,
                canary.to_vec(),
            ),
            (
                r"Software\SimonTatham\PuTTY\Jumplist",
                "Recent sessions",
                REG_BINARY,
                vec![1, 2],
            ),
        ],
    );
    std::fs::write(&hive, tree).unwrap();
    let canary_hex = "707269766174652d63616e6172792d6e657665722d6578706f7274";
    for format in ["csv", "json", "jsonl"] {
        let r = run(&[
            "parse-registry",
            "remote-access",
            "-f",
            hive.to_str().unwrap(),
            "-t",
            format,
        ]);
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert!(r.stderr.contains("Recent sessions"), "{}", r.stderr);
        for text in [&r.stdout, &r.stderr] {
            assert!(
                !text.contains("private-canary") && !text.contains(canary_hex),
                "{text}"
            );
        }
        if format == "csv" {
            assert_eq!(
                csv::Reader::from_reader(r.stdout.as_bytes())
                    .records()
                    .count(),
                2
            );
        } else {
            let records = json_documents(&r.stdout);
            assert_eq!(records.len(), 2);
            assert!(records.iter().all(|r| r["PasswordStored"] == true));
        }
    }
}

// --- parse-registry system-info -----------------------------------------------------------------

/// A `TIME_FIELDS`: year, month, day, hour, minute, second, milliseconds, weekday.
fn time_fields(fields: [u16; 8]) -> Vec<u8> {
    fields.iter().flat_map(|f| f.to_le_bytes()).collect()
}

/// A `UNICODE_STRING` as LSA keeps it: length, maximum length, the text's offset, the text.
fn lsa_string(text: &str) -> Vec<u8> {
    let units: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut b = (units.len() as u16).to_le_bytes().to_vec();
    b.extend(((units.len() + 2) as u16).to_le_bytes());
    b.extend(8u32.to_le_bytes());
    b.extend(&units);
    b.extend([0, 0]);
    b
}

/// A SID `S-1-5-<subs>`.
fn sid_bytes(subs: &[u32]) -> Vec<u8> {
    let mut b = vec![1, subs.len() as u8, 0, 0, 0, 0, 0, 5];
    b.extend(subs.iter().flat_map(|s| s.to_le_bytes()));
    b
}

/// One domain-joined system's SYSTEM, SOFTWARE and SECURITY hives under `root`, where Windows
/// keeps them: a Tokyo time zone, an upgrade from 1803, two profiles and the last user.
fn system_info_collection(root: &Path) {
    let zone = r"ControlSet001\Control\TimeZoneInformation";
    let upgrade = r"Setup\Source OS (Updated on 5/16/2019 00:55:20)";
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
                r"ControlSet001\Services\Tcpip\Parameters",
                "Hostname",
                REG_SZ,
                utf16z("host1"),
            ),
            (
                r"ControlSet001\Services\Tcpip\Parameters",
                "Domain",
                REG_SZ,
                utf16z("corp.example.com"),
            ),
            (
                zone,
                "TimeZoneKeyName",
                REG_SZ,
                utf16z("Tokyo Standard Time"),
            ),
            (zone, "Bias", REG_DWORD, (-540i32).to_le_bytes().to_vec()),
            (
                zone,
                "ActiveTimeBias",
                REG_DWORD,
                (-540i32).to_le_bytes().to_vec(),
            ),
            (zone, "StandardBias", REG_DWORD, 0u32.to_le_bytes().to_vec()),
            (
                zone,
                "DaylightBias",
                REG_DWORD,
                (-60i32).to_le_bytes().to_vec(),
            ),
            (zone, "StandardStart", REG_BINARY, time_fields([0; 8])),
            (zone, "DaylightStart", REG_BINARY, time_fields([0; 8])),
            (
                r"ControlSet001\Control\Windows",
                "ShutdownTime",
                REG_BINARY,
                LINKED.to_le_bytes().to_vec(),
            ),
            (
                r"ControlSet001\Control\Session Manager\Environment",
                "PROCESSOR_ARCHITECTURE",
                REG_SZ,
                utf16z("AMD64"),
            ),
            (
                r"ControlSet001\Control\Nls\CodePage",
                "ACP",
                REG_SZ,
                utf16z("932"),
            ),
            (upgrade, "ProductName", REG_SZ, utf16z("Windows 10 Pro")),
            (upgrade, "ReleaseId", REG_SZ, utf16z("1803")),
            (upgrade, "CurrentBuild", REG_SZ, utf16z("17134")),
            (upgrade, "UBR", REG_DWORD, 765u32.to_le_bytes().to_vec()),
        ],
    );
    let cv = r"Microsoft\Windows NT\CurrentVersion";
    let alice = format!(r"{cv}\ProfileList\S-1-5-21-7-8-9-1105");
    let logon = r"Microsoft\Windows\CurrentVersion\Authentication\LogonUI";
    let software = reg_tree(
        "ROOT",
        &[
            (cv, "ProductName", REG_SZ, utf16z("Windows 10 Enterprise")),
            (cv, "EditionID", REG_SZ, utf16z("Enterprise")),
            (cv, "DisplayVersion", REG_SZ, utf16z("22H2")),
            (cv, "CurrentBuild", REG_SZ, utf16z("19045")),
            (cv, "UBR", REG_DWORD, 4170u32.to_le_bytes().to_vec()),
            (
                cv,
                "CurrentMajorVersionNumber",
                REG_DWORD,
                10u32.to_le_bytes().to_vec(),
            ),
            (
                cv,
                "CurrentMinorVersionNumber",
                REG_DWORD,
                0u32.to_le_bytes().to_vec(),
            ),
            // 2022-02-01T00:00:00Z
            (
                cv,
                "InstallDate",
                REG_DWORD,
                1_643_673_600u32.to_le_bytes().to_vec(),
            ),
            (cv, "RegisteredOwner", REG_SZ, utf16z("IT")),
            (
                &format!(r"{cv}\ProfileList\S-1-5-18"),
                "ProfileImagePath",
                REG_EXPAND_SZ,
                utf16z(r"%systemroot%\system32\config\systemprofile"),
            ),
            (
                &alice,
                "ProfileImagePath",
                REG_EXPAND_SZ,
                utf16z(r"C:\Users\alice"),
            ),
            (
                &alice,
                "LocalProfileLoadTimeHigh",
                REG_DWORD,
                ((LINKED >> 32) as u32).to_le_bytes().to_vec(),
            ),
            (
                &alice,
                "LocalProfileLoadTimeLow",
                REG_DWORD,
                (LINKED as u32).to_le_bytes().to_vec(),
            ),
            (logon, "LastLoggedOnUser", REG_SZ, utf16z(r"CORP\alice")),
            (
                logon,
                "LastLoggedOnUserSID",
                REG_SZ,
                utf16z("S-1-5-21-7-8-9-1105"),
            ),
        ],
    );
    let security = reg_tree(
        "ROOT",
        &[
            (r"Policy\PolAcDmN", "", 0, lsa_string("HOST1")),
            (r"Policy\PolAcDmS", "", 0, sid_bytes(&[21, 1, 2, 3])),
            (r"Policy\PolPrDmN", "", 0, lsa_string("CORP")),
            (r"Policy\PolPrDmS", "", 0, sid_bytes(&[21, 7, 8, 9])),
            (r"Policy\PolDnDDN", "", 0, lsa_string("corp.example.com")),
        ],
    );
    let config = root.join("C/Windows/System32/config");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("SYSTEM"), system).unwrap();
    std::fs::write(config.join("SOFTWARE"), software).unwrap();
    std::fs::write(config.join("SECURITY"), security).unwrap();
}

/// `parse-registry system-info` with `args`, as JSON Lines.
fn system_info_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "system-info"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_system_info_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    system_info_collection(dir.path());
    let records = system_info_read(&["-d", dir.path().to_str().unwrap()]);
    assert_eq!(records.len(), 1);
    let r = &records[0];
    for (field, value) in [
        ("Host", json!("HOST1")),
        ("CompName", json!("HOST1")),
        ("Hostname", json!("host1")),
        ("Domain", json!("corp.example.com")),
        ("PrimaryDomain", json!("CORP")),
        ("DNS-Domain", json!("corp.example.com")),
        ("ProductName", json!("Windows 10 Enterprise")),
        ("Edition", json!("Enterprise")),
        ("DisplayVersion", json!("22H2")),
        ("Version", json!("10.0")),
        ("Build", json!("19045")),
        ("Ubr", json!(4170)),
        ("InstallDate", json!("2022-02-01T00:00:00Z")),
        ("RegisteredOwner", json!("IT")),
        ("TimeZone", json!("Tokyo Standard Time")),
        (
            "TimeZoneDescription",
            json!("Tokyo Standard Time, UTC+09:00; no daylight saving time"),
        ),
        ("UTC-Offset", json!("UTC+09:00")),
        ("DaylightUTC-Offset", Value::Null),
        ("BiasMinutes", json!(-540)),
        ("ActiveBiasMinutes", json!(-540)),
        ("StandardStart", Value::Null),
        ("LastShutdown", json!("2022-02-08T21:03:52Z")),
        ("CurrentControlSet", json!("ControlSet001")),
        ("ProcessorArchitecture", json!("AMD64")),
        ("AnsiCodePage", json!("932")),
        ("LastLoggedOnUser", json!(r"CORP\alice")),
        ("LastLoggedOnUserSID", json!("S-1-5-21-7-8-9-1105")),
        // no SAM: the computer's SID from LSA's account domain
        ("MachineSID", json!("S-1-5-21-1-2-3")),
        ("DomainSID", json!("S-1-5-21-7-8-9")),
        ("SamPath", Value::Null),
        ("TransactionLogStatus", json!("clean")),
        ("Problems", json!([])),
    ] {
        assert_eq!(r[field], value, "{field}");
    }
    assert!(
        r["SourcePath"]
            .as_str()
            .unwrap()
            .ends_with(&format!("config{}SYSTEM", std::path::MAIN_SEPARATOR))
    );
    assert!(r["SecurityPath"].as_str().unwrap().ends_with("SECURITY"));
    assert_eq!(
        r["PreviousVersions"],
        json!([{
            "KeyName": "Source OS (Updated on 5/16/2019 00:55:20)",
            "UpdatedOn": "2019-05-16T00:55:20",
            "ProductName": "Windows 10 Pro",
            "Edition": null,
            "DisplayVersion": "1803",
            "Build": "17134",
            "Ubr": 765,
            "InstallDate": null,
            "InstallTime": null,
            "KeyLastWritten": "2022-02-08T21:03:52Z"
        }])
    );
    let profiles = r["Profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 2);
    assert_eq!(profiles[1]["SID"], "S-1-5-21-7-8-9-1105");
    assert_eq!(profiles[1]["Path"], r"C:\Users\alice");
    assert_eq!(profiles[1]["LocalLoadTime"], "2022-02-08T21:03:52Z");
    assert_eq!(profiles[1]["LocalUnloadTime"], Value::Null);
}

#[test]
fn registry_system_info_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    system_info_collection(dir.path());
    let out = dir.path().join("system.csv");
    let r = run(&[
        "parse-registry",
        "system-info",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "Systems: 1",
        "Hives parsed: SYSTEM 1, SOFTWARE 1, SECURITY 1",
        "HOST1  (",
        "Windows:        Windows 10 Enterprise (Enterprise) 22H2, build 19045.4170",
        "Installed:      2022-02-01T00:00:00Z",
        "Time zone:      Tokyo Standard Time, UTC+09:00; no daylight saving time",
        "Last shutdown:  2022-02-08T21:03:52Z",
        "Domain:         CORP, corp.example.com (joined, S-1-5-21-7-8-9)",
        r"Last logged on: CORP\alice",
        "Computer SID:   S-1-5-21-1-2-3",
        "Profiles:       2",
        "Upgraded from:  Windows 10 Pro 1803, build 17134.765, upgraded on 2019-05-16T00:55:20",
        "Values that could not be read or don't match: 0",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    let (header, rows) = csv_rows(&out);
    assert_eq!(header.len(), 59);
    assert_eq!(
        header[..6],
        [
            "Host",
            "SourcePath",
            "SoftwarePath",
            "SamPath",
            "SecurityPath",
            "CompName"
        ]
    );
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row["TimeZone"], "Tokyo Standard Time");
    assert_eq!(row["UTC-Offset"], "UTC+09:00");
    assert_eq!(row["LastShutdownUTC"], "2022-02-08T21:03:52Z");
    assert_eq!(row["DomainSID"], "S-1-5-21-7-8-9");
    assert_eq!(
        row["PreviousVersions"],
        "Windows 10 Pro 1803, build 17134.765, upgraded on 2019-05-16T00:55:20"
    );
    assert_eq!(
        row["Profiles"],
        r"S-1-5-18 %systemroot%\system32\config\systemprofile (key 2022-02-08T21:03:52Z) | S-1-5-21-7-8-9-1105 C:\Users\alice (key 2022-02-08T21:03:52Z, loaded 2022-02-08T21:03:52Z)"
    );
    assert_eq!(row["Problems"], "");
}

#[test]
fn registry_system_info_of_public_hives() {
    // the DFIR Artifact Museum's dirty Windows 11 SYSTEM with its logs, and its SAM: the values
    // regipy's and Plaso's plugins read, and the SAM's SID
    let hive = system_hive();
    let config = hive.path().join("HOST/C/Windows/System32/config");
    unpack_registry("rathbun_w11", &["SAM"], &config);
    let records = system_info_read(&["-f", config.join("SAM").to_str().unwrap()]);
    assert_eq!(records.len(), 1);
    let r = &records[0];
    for (field, value) in [
        ("Host", json!("GOLDPAPERW11")),
        ("CompName", json!("GOLDPAPERW11")),
        ("Hostname", json!("GoldPaperW11")),
        ("Domain", Value::Null),
        ("TimeZone", json!("Eastern Standard Time")),
        ("UTC-Offset", json!("UTC-05:00")),
        ("DaylightUTC-Offset", json!("UTC-04:00")),
        ("BiasMinutes", json!(300)),
        ("ActiveBiasMinutes", json!(300)),
        ("StandardStart", json!("first Sunday of November, 02:00")),
        ("DaylightStart", json!("second Sunday of March, 02:00")),
        ("DynamicDaylightTimeDisabled", json!(false)),
        ("LastShutdown", json!("2022-02-07T07:37:47.374974700Z")),
        ("ProcessorArchitecture", json!("AMD64")),
        ("AnsiCodePage", json!("1252")),
        ("OemCodePage", json!("437")),
        (
            "MachineSID",
            json!("S-1-5-21-4063860464-1508573791-632671957"),
        ),
        ("ProductName", Value::Null),
        ("TransactionLogStatus", json!("replayed")),
        ("Problems", json!([])),
    ] {
        assert_eq!(r[field], value, "{field}");
    }
    // -f SAM found the SYSTEM next to it
    assert!(r["SourcePath"].as_str().unwrap().ends_with("SYSTEM"));
    assert!(r["SamPath"].as_str().unwrap().ends_with("SAM"));
    assert!(
        r["LogsApplied"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l.as_str().unwrap().starts_with("SYSTEM.LOG"))
    );
}

#[test]
fn registry_system_info_errors() {
    let dir = tempfile::tempdir().unwrap();
    system_info_collection(dir.path());
    let config = dir.path().join("C/Windows/System32/config");
    let copy = dir.path().join("SYSTEM.copy");
    std::fs::copy(config.join("SYSTEM"), &copy).unwrap();
    // two SYSTEM hives: the base block names the copy
    std::fs::write(
        &copy,
        reg_tree(
            "ROOT",
            &[("Select", "Current", REG_DWORD, 1u32.to_le_bytes().to_vec())],
        ),
    )
    .unwrap();
    let mut bytes = std::fs::read(&copy).unwrap();
    let name: Vec<u8> = "SYSTEM".encode_utf16().flat_map(u16::to_le_bytes).collect();
    bytes[0x30..0x30 + name.len()].copy_from_slice(&name);
    std::fs::write(&copy, bytes).unwrap();
    let r = run(&[
        "parse-registry",
        "system-info",
        "-f",
        config.join("SYSTEM").to_str().unwrap(),
        "-f",
        copy.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr.contains("Two SYSTEM hives were given")
            || r.stdout.contains("Two SYSTEM hives were given"),
        "{}{}",
        r.stdout,
        r.stderr
    );
    // not one of the four
    let ntuser = dir.path().join("NTUSER.DAT");
    std::fs::write(&ntuser, reg_tree("ROOT", &[])).unwrap();
    let r = run(&[
        "parse-registry",
        "system-info",
        "-f",
        ntuser.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr
            .contains("is not a SYSTEM, SOFTWARE, SAM or SECURITY hive"),
        "{}",
        r.stderr
    );
    let empty = tempfile::tempdir().unwrap();
    let r = run(&[
        "parse-registry",
        "system-info",
        "-d",
        empty.path().to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr
            .contains("No SYSTEM, SOFTWARE, SAM or SECURITY hives were found")
    );
    // the output is never one of the hives read, even with -C
    let original = std::fs::read(config.join("SECURITY")).unwrap();
    let r = run(&[
        "parse-registry",
        "system-info",
        "-f",
        config.join("SYSTEM").to_str().unwrap(),
        "-o",
        config.join("SECURITY").to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(config.join("SECURITY")).unwrap(), original);
}

#[test]
fn registry_system_info_preserves_sam_replay_provenance() {
    let dir = tempfile::tempdir().unwrap();
    let mut sid = vec![1, 4, 0, 0, 0, 0, 0, 5];
    for number in [21u32, 1, 2, 3] {
        sid.extend(number.to_le_bytes());
    }
    let mut hive = reg_tree("ROOT", &[(r"SAM\Domains\Account", "V", REG_BINARY, sid)]);
    // An old-format transaction log writing all hive-bin pages. The SAM value is
    // recovered from those pages, so the exported SID must retain that provenance.
    let bins = hive.len() - 4096;
    let mut log = hive[..512].to_vec();
    put_u32(&mut log, 28, 1);
    let checksum = log[..508]
        .as_chunks::<4>()
        .0
        .iter()
        .fold(0u32, |x, w| x ^ u32::from_le_bytes(*w));
    put_u32(&mut log, 508, checksum);
    log.extend(b"DIRT");
    log.extend(vec![0xff; bins / 4096]);
    log.resize(log.len().next_multiple_of(512), 0);
    log.extend_from_slice(&hive[4096..]);
    // The primary still has an older SID; the log replaces its last subauthority.
    let sid_start = hive
        .windows(12)
        .position(|w| w == [1, 4, 0, 0, 0, 0, 0, 5, 21, 0, 0, 0])
        .unwrap();
    put_u32(&mut hive, sid_start + 20, 4);
    put_u32(&mut hive, 4, 2);
    let checksum = hive[..508]
        .as_chunks::<4>()
        .0
        .iter()
        .fold(0u32, |x, w| x ^ u32::from_le_bytes(*w));
    put_u32(&mut hive, 508, checksum);
    std::fs::write(dir.path().join("SAM"), hive).unwrap();
    std::fs::write(dir.path().join("SAM.LOG"), log).unwrap();
    let records = system_info_read(&["-f", dir.path().join("SAM").to_str().unwrap()]);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["MachineSID"], "S-1-5-21-1-2-3");
    assert_eq!(records[0]["TransactionLogStatus"], "replayed");
}

// --- parse-registry mru -------------------------------------------------------------------------

/// `parse-registry mru`'s CSV columns.
const MRU_COLUMNS: [&str; 18] = [
    "Host",
    "User",
    "UserSID",
    "SourcePath",
    "List",
    "Subkey",
    "MRU-Position",
    "IsMostRecent",
    "ValueName",
    "Value",
    "TypedTimeUTC",
    "LastUsedUTC",
    "KeyPath",
    "KeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// One system's SYSTEM and SOFTWARE hives (its name, alice's profile), alice's NTUSER.DAT with
/// every list (a Run command, typed paths, searches with one cut short, typed addresses with
/// their times, Paint's and WordPad's recent files, Windows XP's search terms), and bob's
/// NTUSER.DAT without any.
fn mru_collection(root: &Path) {
    let explorer = r"Software\Microsoft\Windows\CurrentVersion\Explorer";
    let run = format!(r"{explorer}\RunMRU");
    let paths = format!(r"{explorer}\TypedPaths");
    let words = format!(r"{explorer}\WordWheelQuery");
    let urls = r"Software\Microsoft\Internet Explorer\TypedURLs";
    let times = r"Software\Microsoft\Internet Explorer\TypedURLsTime";
    let applets = r"Software\Microsoft\Windows\CurrentVersion\Applets";
    let acmru = r"Software\Microsoft\Search Assistant\ACMru\5603";
    let mru_ex = |list: &[u32]| -> Vec<u8> {
        list.iter()
            .chain(&[u32::MAX])
            .flat_map(|n| n.to_le_bytes())
            .collect()
    };
    let alice = reg_tree(
        "ROOT",
        &[
            (&run, "MRUList", REG_SZ, utf16z("ba")),
            (&run, "a", REG_SZ, utf16z(r"cmd\1")),
            (&run, "b", REG_SZ, utf16z(r"powershell -ep bypass\1")),
            (&paths, "url1", REG_SZ, utf16z(r"\\fileserver\share")),
            (&paths, "url2", REG_SZ, utf16z(r"C:\Users\alice\Downloads")),
            (&words, "MRUListEx", REG_BINARY, mru_ex(&[1, 0])),
            (&words, "0", REG_BINARY, utf16z("invoice")),
            (&words, "1", REG_BINARY, utf16z("password")[..6].to_vec()),
            (urls, "url1", REG_SZ, utf16z("http://example.com/")),
            (urls, "url2", REG_SZ, utf16z("http://example.org/")),
            (
                times,
                "url1",
                REG_BINARY,
                (LINKED + DAY).to_le_bytes().to_vec(),
            ),
            (
                times,
                "url2",
                REG_BINARY,
                (LINKED - DAY).to_le_bytes().to_vec(),
            ),
            (
                &format!(r"{applets}\Paint\Recent File List"),
                "File1",
                REG_SZ,
                utf16z(r"C:\Users\alice\Pictures\screen.png"),
            ),
            (
                &format!(r"{applets}\Wordpad\Recent File List"),
                "File1",
                REG_SZ,
                utf16z(r"C:\Users\alice\notes.rtf"),
            ),
            (acmru, "000", REG_SZ, utf16z("*.kdbx")),
        ],
    );
    let bob = reg_tree(
        "ROOT",
        &[(
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
            "Start_TrackProgs",
            REG_DWORD,
            1u32.to_le_bytes().to_vec(),
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

/// `parse-registry mru` with `args`, as JSON Lines.
fn mru_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "mru"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

/// Each record's list, subkey, place, value name, value and typed time.
fn mru_lines(records: &[Value]) -> Vec<String> {
    let text = |v: &Value| v.as_str().unwrap_or("-").to_string();
    records
        .iter()
        .map(|r| {
            format!(
                "{} {} {} {} | {} | {}",
                text(&r["List"]),
                text(&r["Subkey"]),
                r["MRU-Position"],
                text(&r["ValueName"]),
                text(&r["Value"]),
                text(&r["TypedTime"]),
            )
        })
        .collect()
}

#[test]
fn registry_mru_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    mru_collection(dir.path());
    let records = mru_read(&["-d", dir.path().to_str().unwrap()]);
    assert_eq!(
        mru_lines(&records),
        [
            "RunMRU - 0 b | powershell -ep bypass | -",
            "RunMRU - 1 a | cmd | -",
            r"TypedPaths - 0 url1 | \\fileserver\share | -",
            r"TypedPaths - 1 url2 | C:\Users\alice\Downloads | -",
            "WordWheelQuery - 0 1 | - | -",
            "WordWheelQuery - 1 0 | invoice | -",
            "TypedURLs - 0 url1 | http://example.com/ | 2022-02-09T21:03:52Z",
            "TypedURLs - 1 url2 | http://example.org/ | 2022-02-07T21:03:52Z",
            r"Paint - 0 File1 | C:\Users\alice\Pictures\screen.png | -",
            r"Wordpad - 0 File1 | C:\Users\alice\notes.rtf | -",
            "ACMru 5603 0 000 | *.kdbx | -",
        ]
    );
    for r in &records {
        assert_eq!(r["Host"], "HOST1");
        assert_eq!(r["User"], "alice");
        assert_eq!(r["UserSID"], "S-1-5-21-1-2-3-1001");
        assert_eq!(r["TransactionLogStatus"], "clean");
        assert_eq!(r["KeyLastWritten"], "2022-02-08T21:03:52Z");
        // the key's time dates its most recent entry only
        let last_used = if r["IsMostRecent"] == true {
            json!("2022-02-08T21:03:52Z")
        } else {
            Value::Null
        };
        assert_eq!(r["LastUsed"], last_used, "{r}");
    }
    // a term cut short keeps its bytes
    assert_eq!(records[4]["DataRaw"], "700061007300");
    assert_eq!(
        records[4]["Problems"],
        json!(["its term has no NUL: the data is cut short"])
    );
    assert!(records[5]["DataRaw"].is_null());
    assert_eq!(
        records[10]["KeyPath"],
        r"Software\Microsoft\Search Assistant\ACMru\5603"
    );

    // --list keeps some lists, in any case
    let records = mru_read(&[
        "-d",
        dir.path().to_str().unwrap(),
        "--list",
        "TypedURLs,acmru",
    ]);
    assert_eq!(
        mru_lines(&records),
        [
            "TypedURLs - 0 url1 | http://example.com/ | 2022-02-09T21:03:52Z",
            "TypedURLs - 1 url2 | http://example.org/ | 2022-02-07T21:03:52Z",
            "ACMru 5603 0 000 | *.kdbx | -",
        ]
    );
}

#[test]
fn registry_mru_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    mru_collection(dir.path());
    let out = dir.path().join("mru.csv");
    let r = run(&[
        "parse-registry",
        "mru",
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
        "Hives without these lists: 1",
        "Entries: 11 (RunMRU 2, TypedPaths 2, WordWheelQuery 2, TypedURLs 2, Paint 1, Wordpad 1, ACMru 1)",
        "2022-02-09T21:03:52Z  alice  TypedURLs: http://example.com/",
        "2022-02-08T21:03:52Z  alice  RunMRU: powershell -ep bypass",
        r"2022-02-08T21:03:52Z  alice  ACMru\5603: *.kdbx",
        "2022-02-07T21:03:52Z  alice  TypedURLs: http://example.org/",
        "Entries or lists that could not be read: 1",
        "its term has no NUL: the data is cut short",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    // the typed address first: it is the newest
    let typed = r.stdout.find("TypedURLs: http://example.com/").unwrap();
    let run_mru = r.stdout.find("RunMRU: powershell").unwrap();
    assert!(typed < run_mru);
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let header: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, MRU_COLUMNS);
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(rows.len(), 11);
    assert_eq!(&rows[0][4], "RunMRU");
    assert_eq!(&rows[0][7], "true");
    assert_eq!(&rows[0][9], "powershell -ep bypass");
    assert_eq!(&rows[0][11], "2022-02-08T21:03:52Z");
    assert_eq!(&rows[1][11], "");
    assert_eq!(&rows[6][10], "2022-02-09T21:03:52Z");
    assert_eq!(&rows[10][5], "5603");
}

#[test]
fn registry_mru_of_public_hives() {
    let dir = tempfile::tempdir().unwrap();
    // regipy's Windows 7 NTUSER.DAT: Explorer's searches and typed addresses, as Plaso's and
    // regipy's plugins read them
    unpack_registry("regipy", &["NTUSER.DAT"], dir.path());
    let records = mru_read(&["-f", dir.path().join("NTUSER.DAT").to_str().unwrap()]);
    assert_eq!(
        mru_lines(&records),
        [
            "WordWheelQuery - 0 1 | alloy | -",
            "WordWheelQuery - 1 5 | test-plan | -",
            "WordWheelQuery - 2 4 | vibranium | -",
            "WordWheelQuery - 3 3 | accounts | -",
            "WordWheelQuery - 4 2 | myron maclain | -",
            "WordWheelQuery - 5 0 | adamantium | -",
            "TypedURLs - 0 url1 | http://199.73.28.114:53/ | -",
            "TypedURLs - 1 url2 | http://go.microsoft.com/fwlink/?LinkId=69157 | -",
        ]
    );
    assert_eq!(records[0]["LastUsed"], "2012-04-04T15:45:18.551339700Z");

    // Plaso's NTUSER.DAT: typed addresses with their times from TypedURLsTime
    let plaso = dir.path().join("plaso");
    unpack_registry("plaso", &["NTUSER.DAT"], &plaso);
    let records = mru_read(&["-f", plaso.join("NTUSER.DAT").to_str().unwrap()]);
    assert_eq!(records.len(), 5);
    assert_eq!(records[0]["Value"], "http://www.catnews.com/");
    assert_eq!(records[0]["TypedTime"], "2016-10-05T09:47:50.967417100Z");
    assert_eq!(records[3]["TypedTime"], "2016-10-05T09:35:04.641445600Z");
    assert!(records[4]["TypedTime"].is_null());

    // the DFIR Artifact Museum's dirty Windows 11 NTUSER.DAT, with its logs
    let w11 = dir.path().join("w11");
    unpack_registry(
        "rathbun_w11",
        &["NTUSER.DAT", "ntuser.dat.LOG1", "ntuser.dat.LOG2"],
        &w11,
    );
    let records = mru_read(&["-f", w11.join("NTUSER.DAT").to_str().unwrap()]);
    assert_eq!(
        mru_lines(&records),
        [
            r"TypedPaths - 0 url1 | C:\Users\AndrewRathbun\AppData | -",
            "TypedURLs - 0 url1 | http://go.microsoft.com/fwlink/p/?LinkId=255141 | -",
        ]
    );
    assert!(records.iter().all(|r| r["Problems"] == json!([])));
}

#[test]
fn registry_mru_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM", "NTUSER.DAT"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&["parse-registry", "mru", "-f", sam.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(
        said.contains("a SAM hive, not an NTUSER.DAT hive"),
        "{said}"
    );
    let ntuser = dir.path().join("NTUSER.DAT");
    let r = run(&[
        "parse-registry",
        "mru",
        "-f",
        ntuser.to_str().unwrap(),
        "--list",
        "runmru,bogus",
        "-K",
    ]);
    assert_ne!(r.code, 0);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("bogus"), "{said}");
    let empty = tempfile::tempdir().unwrap();
    let r = run(&[
        "parse-registry",
        "mru",
        "-d",
        empty.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("No NTUSER.DAT hives were found"), "{said}");
    // the output is never one of the files read
    let original = std::fs::read(&ntuser).unwrap();
    let r = run(&[
        "parse-registry",
        "mru",
        "-f",
        ntuser.to_str().unwrap(),
        "-o",
        ntuser.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(&ntuser).unwrap(), original);
}

#[test]
fn registry_mru_keeps_list_warnings_in_stdout_exports() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("NTUSER.DAT");
    std::fs::write(
        &path,
        reg_tree(
            "ROOT",
            &[
                (
                    r"Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU",
                    "MRUList",
                    REG_SZ,
                    utf16z("aa"),
                ),
                (
                    r"Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU",
                    "a",
                    REG_SZ,
                    utf16z(r"cmd\1"),
                ),
            ],
        ),
    )
    .unwrap();
    for format in ["csv", "json", "jsonl"] {
        let r = run(&[
            "parse-registry",
            "mru",
            "-f",
            path.to_str().unwrap(),
            "-t",
            format,
        ]);
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert!(r.stderr.contains("listed twice"), "{}", r.stderr);
        assert!(
            !r.stdout.contains("listed twice"),
            "diagnostic polluted records"
        );
        if format == "jsonl" {
            assert_eq!(jsonl_records(&r.stdout).len(), 1);
        }
    }
}

// --- parse-registry app-history -----------------------------------------------------------------

/// `parse-registry app-history`'s CSV columns.
const APP_HISTORY_COLUMNS: [&str; 18] = [
    "Host",
    "SourcePath",
    "User",
    "UserSID",
    "App",
    "Item",
    "MRU-Position",
    "Path",
    "Value",
    "FirstUsedUTC",
    "Details",
    "KeyPath",
    "KeyLastWrittenUTC",
    "ValueName",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// One system's SYSTEM hive (its name), SOFTWARE hive (alice's profile) and alice's NTUSER.DAT
/// with a trace of each kind: Sysinternals tools, WinRAR and 7-Zip history (one list cut short),
/// a OneDrive account and a TeamViewer account name.
fn app_history_collection(root: &Path) {
    let utf16_list = |items: &[&str]| -> Vec<u8> {
        let mut b: Vec<u8> = items.iter().flat_map(|i| utf16z(i)).collect();
        b.extend([0, 0]);
        b
    };
    let mut cut = utf16_list(&[r"C:\Users\alice\Desktop"]);
    cut.push(0x41);
    let business = r"Software\Microsoft\OneDrive\Accounts\Business1";
    let ntuser = reg_tree(
        "ROOT",
        &[
            (
                r"Software\Sysinternals\PsExec",
                "EulaAccepted",
                REG_DWORD,
                1u32.to_le_bytes().to_vec(),
            ),
            (
                r"Software\Sysinternals\Process Explorer",
                "EulaAccepted",
                REG_DWORD,
                1u32.to_le_bytes().to_vec(),
            ),
            (
                r"Software\Sysinternals\Process Explorer",
                "ShowLowerpane",
                REG_DWORD,
                0u32.to_le_bytes().to_vec(),
            ),
            (
                r"Software\WinRAR\ArcHistory",
                "1",
                REG_SZ,
                utf16z(r"C:\Users\alice\old.rar"),
            ),
            (
                r"Software\WinRAR\ArcHistory",
                "0",
                REG_SZ,
                utf16z(r"C:\Users\alice\loot.rar"),
            ),
            (
                r"Software\7-Zip\Compression",
                "ArcHistory",
                REG_BINARY,
                utf16_list(&[r"C:\ProgramData\out.7z", r"D:\stage.zip"]),
            ),
            (r"Software\7-Zip\FM", "FolderHistory", REG_BINARY, cut),
            (business, "UserEmail", REG_SZ, utf16z("alice@example.com")),
            (
                business,
                "UserFolder",
                REG_SZ,
                utf16z(r"C:\Users\alice\OneDrive - Example"),
            ),
            (
                r"Software\TeamViewer",
                "BuddyLoginName",
                REG_SZ,
                utf16z("alice@example.com"),
            ),
        ],
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
    let profile = root.join("C/Users/alice");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::create_dir_all(&profile).unwrap();
    std::fs::write(config.join("SYSTEM"), system).unwrap();
    std::fs::write(config.join("SOFTWARE"), software).unwrap();
    std::fs::write(profile.join("NTUSER.DAT"), ntuser).unwrap();
}

/// `parse-registry app-history` with `args`, as JSON Lines.
fn app_history_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "app-history"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_app_history_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    app_history_collection(dir.path());
    let records = app_history_read(&["-d", dir.path().to_str().unwrap()]);
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} {} {} | {} | {} | {}",
                r["App"].as_str().unwrap(),
                r["Item"].as_str().unwrap(),
                r["MRU-Position"],
                r["Path"].as_str().unwrap_or("-"),
                r["Value"].as_str().unwrap_or("-"),
                r["FirstUsed"].as_str().unwrap_or("-"),
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            // the hive's (sorted) keys: Process Explorer keeps settings, so no first run
            "Sysinternals Process Explorer null | - | 1 | -",
            "Sysinternals PsExec null | - | 1 | 2022-02-08T21:03:52Z",
            r"WinRAR ArcHistory 0 | C:\Users\alice\loot.rar | - | -",
            r"WinRAR ArcHistory 1 | C:\Users\alice\old.rar | - | -",
            r"7-Zip ArcHistory 0 | C:\ProgramData\out.7z | - | -",
            r"7-Zip ArcHistory 1 | D:\stage.zip | - | -",
            "7-Zip FolderHistory null | - | - | -",
            r"7-Zip FolderHistory 0 | C:\Users\alice\Desktop | - | -",
            r"OneDrive Business1 null | C:\Users\alice\OneDrive - Example | alice@example.com | -",
            "TeamViewer BuddyLoginName null | - | alice@example.com | -",
        ]
    );
    for r in &records {
        assert_eq!(r["Host"], "HOST1");
        assert_eq!(r["User"], "alice");
        assert_eq!(r["UserSID"], "S-1-5-21-1-2-3-1001");
        assert_eq!(r["TransactionLogStatus"], "clean");
        assert_eq!(r["KeyLastWritten"], "2022-02-08T21:03:52Z");
    }
    assert_eq!(records[1]["KeyPath"], r"Software\Sysinternals\PsExec");
    assert_eq!(records[1]["ValueName"], Value::Null);
    assert_eq!(records[2]["ValueName"], "0");
    assert_eq!(records[2]["Details"], json!({}));
    // the list cut short: its entries as far as they go, and its bytes
    let cut = &records[6];
    assert_eq!(
        cut["Problems"],
        json!(["is not a list of valid UTF-16 strings"])
    );
    assert!(cut["DataRaw"].as_str().unwrap().ends_with("000041"));
}

#[test]
fn registry_app_history_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    app_history_collection(dir.path());
    let out = dir.path().join("apps.csv");
    let r = run(&[
        "parse-registry",
        "app-history",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "NTUSER.DAT hives parsed: 1",
        "Hosts: HOST1",
        "Users: alice",
        "Traces: 10 (7-Zip 4, OneDrive 1, Sysinternals 2, TeamViewer 1, WinRAR 2)",
        "2022-02-08T21:03:52Z  HOST1  alice  PsExec",
        r"2022-02-08T21:03:52Z  HOST1  alice  WinRAR ArcHistory  C:\Users\alice\loot.rar",
        r"2022-02-08T21:03:52Z  HOST1  alice  7-Zip ArcHistory  C:\ProgramData\out.7z",
        r"HOST1  alice  Business1  alice@example.com  C:\Users\alice\OneDrive - Example",
        "Traces or keys that could not be read: 1",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    assert!(
        !r.stdout.contains("alice  Process Explorer"),
        "{}",
        r.stdout
    );
    let (header, rows) = csv_rows(&out);
    assert_eq!(header, APP_HISTORY_COLUMNS);
    assert_eq!(rows.len(), 10);
    assert_eq!(rows[1]["FirstUsedUTC"], "2022-02-08T21:03:52Z");
    assert_eq!(rows[0]["FirstUsedUTC"], "");
    assert_eq!(rows[3]["MRU-Position"], "1");
    assert_eq!(rows[3]["ValueName"], "1");
    assert_eq!(rows[6]["MRU-Position"], "");
    assert_eq!(rows[8]["Value"], "alice@example.com");
}

#[test]
fn registry_app_history_of_public_hives() {
    // the DFIR Artifact Museum's dirty Windows 10 APT simulation user, with its logs: PsExec and
    // Sysmon, as regipy's sysinternals gives them
    let dir = tempfile::tempdir().unwrap();
    unpack_registry(
        "aptsimulator_w10",
        &["NTUSER.DAT", "ntuser.dat.LOG1", "ntuser.dat.LOG2"],
        dir.path(),
    );
    let records = app_history_read(&["-f", dir.path().join("NTUSER.DAT").to_str().unwrap()]);
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} {} {} {} {}",
                r["App"].as_str().unwrap(),
                r["Item"].as_str().unwrap(),
                r["Value"].as_str().unwrap_or("-"),
                r["FirstUsed"].as_str().unwrap_or("-"),
                r["Path"].as_str().unwrap_or("-"),
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            "Sysinternals PsExec 1 2022-03-10T19:57:43.579822600Z -",
            "Sysinternals System Monitor 1 2022-03-10T19:51:17.013712800Z -",
            r"OneDrive Personal - - C:\Users\TestUser\OneDrive",
        ]
    );
    assert!(records.iter().all(|r| r["Problems"] == json!([])
        && r["TransactionLogStatus"] == "unchanged"
        && r["User"] == "TestUser"));

    // regipy's NTUSER.DAT: WinRAR's history, as regipy's winrar_plugin and Plaso's winrar_mru
    // give it
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["NTUSER.DAT"], dir.path());
    let records = app_history_read(&["-f", dir.path().join("NTUSER.DAT").to_str().unwrap()]);
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} {} {}",
                r["Item"].as_str().unwrap(),
                r["MRU-Position"],
                r["Path"].as_str().unwrap()
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            r"ArcHistory 0 C:\Users\tony\Downloads\RegistryFinder64.zip",
            r"ArcHistory 1 C:\temp\token.zip",
            "ArcName 0 Tools.zip",
            "ArcName 1 data.zip",
            r"ExtrPath 0 C:\Users\tony\Downloads",
            r"ExtrPath 1 C:\temp",
        ]
    );
    assert_eq!(
        records[0]["KeyLastWritten"],
        "2021-11-18T13:59:04.888952700Z"
    );
}

#[test]
fn registry_app_history_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&[
        "parse-registry",
        "app-history",
        "-f",
        sam.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("not an NTUSER.DAT hive"), "{said}");
    let r = run(&[
        "parse-registry",
        "app-history",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("No NTUSER.DAT hives were found"), "{said}");
    // the output is never one of the files read: the hive's system's SOFTWARE included
    let collection = tempfile::tempdir().unwrap();
    app_history_collection(collection.path());
    let software = collection.path().join("C/Windows/System32/config/SOFTWARE");
    let original = std::fs::read(&software).unwrap();
    let r = run(&[
        "parse-registry",
        "app-history",
        "-f",
        collection
            .path()
            .join("C/Users/alice/NTUSER.DAT")
            .to_str()
            .unwrap(),
        "-o",
        software.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(&software).unwrap(), original);
}

#[test]
fn registry_app_history_keeps_damaged_list_warnings_in_exports() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("NTUSER.DAT");
    let mut bytes = reg_tree(
        "ROOT",
        &[(
            r"Software\Sysinternals\PsExec",
            "EulaAccepted",
            REG_DWORD,
            1u32.to_le_bytes().to_vec(),
        )],
    );
    let name = bytes.windows(6).position(|w| w == b"PsExec").unwrap();
    // The value list holds one offset, but its key claims two.
    put_u32(&mut bytes, name - 76 + 36, 2);
    std::fs::write(&hive, bytes).unwrap();
    for format in ["csv", "json", "jsonl"] {
        let r = run(&[
            "parse-registry",
            "app-history",
            "-f",
            hive.to_str().unwrap(),
            "-t",
            format,
        ]);
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert!(
            r.stderr.contains("Software\\Sysinternals\\PsExec"),
            "{}",
            r.stderr
        );
        if format == "csv" {
            assert_eq!(
                csv::Reader::from_reader(r.stdout.as_bytes())
                    .records()
                    .count(),
                1
            );
        } else {
            let records = json_documents(&r.stdout);
            assert_eq!(records.len(), 1);
            assert!(records[0]["FirstUsed"].is_null());
        }
    }
}

// --- parse-registry feature-usage ----------------------------------------------------------------

/// `parse-registry feature-usage`'s CSV columns.
const FEATUREUSAGE_COLUMNS: [&str; 20] = [
    "Host",
    "User",
    "UserSID",
    "SourcePath",
    "Source",
    "App",
    "AppPath",
    "Count",
    "LastAccessedUTC",
    "ItemPath",
    "DisplayName",
    "PinType",
    "Index",
    "ValueName",
    "KeyPath",
    "KeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// One system's SYSTEM and SOFTWARE hives (its name, alice's profile) and alice's NTUSER.DAT
/// with FeatureUsage counts (one of them a string), a recent app with a file it opened, and a
/// Windows 7 ProgramsCache whose format version is unknown; and bob's NTUSER.DAT, without any.
fn featureusage_collection(root: &Path) {
    let usage = r"Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage";
    let switched = format!(r"{usage}\AppSwitched");
    let tray = format!(r"{usage}\TrayButtonClicked");
    let app = r"Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{6153EB14-4709-4178-8D85-8DC8A345B6FC}";
    let item = format!(r"{app}\RecentItems\{{0B2C3D4E-0000-0000-0000-000000000001}}");
    let alice = reg_tree(
        "ROOT",
        &[
            (
                usage,
                "KeyCreationTime",
                REG_QWORD,
                (LINKED - DAY).to_le_bytes().to_vec(),
            ),
            (
                &switched,
                "Microsoft.Windows.Explorer",
                REG_DWORD,
                12u32.to_le_bytes().to_vec(),
            ),
            (
                &switched,
                r"{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\cmd.exe",
                REG_DWORD,
                5u32.to_le_bytes().to_vec(),
            ),
            (&switched, "Damaged", REG_SZ, utf16z("3")),
            (&tray, "StartButton", REG_DWORD, 3u32.to_le_bytes().to_vec()),
            (
                app,
                "AppID",
                REG_SZ,
                utf16z(r"C:\Users\alice\Downloads\tool.exe"),
            ),
            (app, "LaunchCount", REG_DWORD, 2u32.to_le_bytes().to_vec()),
            (
                app,
                "LastAccessedTime",
                REG_QWORD,
                LINKED.to_le_bytes().to_vec(),
            ),
            (&item, "DisplayName", REG_SZ, utf16z("report.docx")),
            (&item, "Path", REG_SZ, utf16z(r"C:\Users\alice\report.docx")),
            (
                &item,
                "LastAccessedTime",
                REG_QWORD,
                (LINKED + DAY).to_le_bytes().to_vec(),
            ),
            (
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2",
                "ProgramsCache",
                REG_BINARY,
                vec![7, 0, 0, 0, 1],
            ),
        ],
    );
    let bob = reg_tree(
        "ROOT",
        &[(
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
            "Start_TrackProgs",
            REG_DWORD,
            1u32.to_le_bytes().to_vec(),
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

/// `parse-registry feature-usage` with `args`, as JSON Lines.
fn featureusage_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "feature-usage"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_featureusage_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    featureusage_collection(dir.path());
    let records = featureusage_read(&["-d", dir.path().to_str().unwrap()]);
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} | {} | {} | {} | {} | {}",
                r["Source"].as_str().unwrap(),
                r["App"].as_str().unwrap_or("-"),
                r["AppPath"].as_str().unwrap_or("-"),
                r["Count"],
                r["LastAccessed"].as_str().unwrap_or("-"),
                r["ItemPath"].as_str().unwrap_or("-"),
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            "AppSwitched | Microsoft.Windows.Explorer | - | 12 | - | -",
            r"AppSwitched | {1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\cmd.exe | %windir%\system32\cmd.exe | 5 | - | -",
            "AppSwitched | Damaged | - | null | - | -",
            "TrayButtonClicked | StartButton | - | 3 | - | -",
            r"RecentApps | C:\Users\alice\Downloads\tool.exe | C:\Users\alice\Downloads\tool.exe | 2 | 2022-02-08T21:03:52Z | -",
            r"RecentItems | C:\Users\alice\Downloads\tool.exe | C:\Users\alice\Downloads\tool.exe | null | 2022-02-09T21:03:52Z | C:\Users\alice\report.docx",
            "ProgramsCache | - | - | null | - | -",
        ]
    );
    for r in &records {
        assert_eq!(r["Host"], "HOST1");
        assert_eq!(r["User"], "alice");
        assert_eq!(r["UserSID"], "S-1-5-21-1-2-3-1001");
        assert_eq!(r["TransactionLogStatus"], "clean");
    }
    assert_eq!(records[5]["DisplayName"], "report.docx");
    // what doesn't decode keeps its bytes
    assert_eq!(records[2]["DataRaw"], "33000000");
    assert_eq!(records[2]["Problems"], json!(["is REG_SZ, not REG_DWORD"]));
    assert_eq!(records[6]["DataRaw"], "0700000001");
    assert_eq!(
        records[6]["Problems"],
        json!(["format version 7 is not known"])
    );
}

#[test]
fn registry_featureusage_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    featureusage_collection(dir.path());
    let out = dir.path().join("featureusage.csv");
    let r = run(&[
        "parse-registry",
        "feature-usage",
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
        "Hives without FeatureUsage, RecentApps, Taskband or ProgramsCache: 1",
        "Records: 7 (AppSwitched 3, ProgramsCache 1, RecentApps 1, RecentItems 1, TrayButtonClicked 1)",
        "2022-02-07T21:03:52Z  alice",
        "      12  alice  Microsoft.Windows.Explorer",
        r"       5  alice  %windir%\system32\cmd.exe",
        r"2022-02-09T21:03:52Z  alice  C:\Users\alice\Downloads\tool.exe: C:\Users\alice\report.docx",
        r"2022-02-08T21:03:52Z  alice  C:\Users\alice\Downloads\tool.exe",
        "Pinned to the taskbar (Favorites): 0",
        "Records or keys that could not be read: 2",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let header: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, FEATUREUSAGE_COLUMNS);
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(rows.len(), 7);
    assert_eq!(&rows[1][4], "AppSwitched");
    assert_eq!(&rows[1][6], r"%windir%\system32\cmd.exe");
    assert_eq!(&rows[1][7], "5");
    assert_eq!(&rows[5][9], r"C:\Users\alice\report.docx");
    assert_eq!(&rows[5][10], "report.docx");
}

#[test]
fn registry_featureusage_of_public_hives() {
    // the DFIR Artifact Museum's Windows 11 system: its SYSTEM, and the user's dirty
    // NTUSER.DAT with its logs, where they were
    let collection = system_hive();
    let profile = collection.path().join("HOST/C/Users/AndrewRathbun");
    unpack_registry(
        "rathbun_w11",
        &["NTUSER.DAT", "ntuser.dat.LOG1", "ntuser.dat.LOG2"],
        &profile,
    );
    let dir = collection.path().to_str().unwrap();
    let records = featureusage_read(&["-d", dir]);
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} {} {} | {} | {} | {}",
                r["Source"].as_str().unwrap(),
                r["ValueName"].as_str().unwrap(),
                r["Index"],
                r["App"].as_str().unwrap_or("-"),
                r["Count"],
                r["ItemPath"].as_str().unwrap_or("-"),
            )
        })
        .collect();
    let link = r"C:\Users\AndrewRathbun\AppData\Roaming\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar";
    assert_eq!(
        lines,
        [
            "AppLaunch Microsoft.Windows.Explorer null | Microsoft.Windows.Explorer | 2 | -".to_string(),
            "AppSwitched Microsoft.Windows.Explorer null | Microsoft.Windows.Explorer | 2 | -".to_string(),
            format!(r"Taskband FavoritesResolve 0 | Microsoft.Windows.Explorer | null | {link}\File Explorer.lnk"),
            format!(r"Taskband FavoritesResolve 1 | MSEdge | null | {link}\Microsoft Edge.lnk"),
            r"Taskband FavoritesResolve 2 | Microsoft.WindowsStore_8wekyb3d8bbwe!App | null | Applications\Microsoft.WindowsStore_8wekyb3d8bbwe!App".to_string(),
            r"Taskband Favorites 0 | Microsoft.Windows.Explorer | null | User Pinned\TaskBar\File Explorer.lnk".to_string(),
            r"Taskband Favorites 1 | MSEdge | null | User Pinned\TaskBar\Microsoft Edge.lnk".to_string(),
            r"Taskband Favorites 2 | Microsoft.WindowsStore_8wekyb3d8bbwe!App | null | Applications\Microsoft.WindowsStore_8wekyb3d8bbwe!App".to_string(),
        ]
    );
    for r in &records {
        assert_eq!(r["Host"], "GOLDPAPERW11");
        assert_eq!(r["User"], "AndrewRathbun");
        assert_eq!(r["Problems"], json!([]));
    }
    assert_eq!(records[5]["PinType"], "SystemPinned");
    assert_eq!(
        records[0]["KeyLastWritten"],
        "2022-02-05T19:49:18.979827400Z"
    );

    // regipy's Windows 7 NTUSER.DAT: the taskbar's pins and the Start menu's programs cache
    let regipy = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["NTUSER.DAT"], regipy.path());
    let records = featureusage_read(&["-f", regipy.path().join("NTUSER.DAT").to_str().unwrap()]);
    let count = |source: &str, value: &str| {
        records
            .iter()
            .filter(|r| r["Source"] == source && r["ValueName"] == value)
            .count()
    };
    assert_eq!(
        [
            count("Taskband", "Favorites"),
            count("Taskband", "FavoritesResolve"),
            count("ProgramsCache", "ProgramsCache"),
            count("ProgramsCache", "ProgramsCacheTBP"),
        ],
        [3, 3, 19, 3]
    );
    let accessories = records
        .iter()
        .find(|r| r["ItemPath"] == r"Programs\Accessories")
        .unwrap();
    assert_eq!(
        accessories["KeyPath"],
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2"
    );
    assert!(records.iter().all(|r| r["Problems"] == json!([])));
}

#[test]
fn registry_featureusage_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&[
        "parse-registry",
        "feature-usage",
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
        "feature-usage",
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
        "feature-usage",
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

// --- parse-registry security-settings -----------------------------------------------------------

/// `parse-registry security-settings`' CSV columns.
const SECURITY_SETTINGS_COLUMNS: [&str; 20] = [
    "Host",
    "SourcePath",
    "HiveType",
    "ControlSet",
    "IsCurrentControlSet",
    "User",
    "UserSID",
    "Area",
    "Setting",
    "Value",
    "Meaning",
    "WeakensDefault",
    "Policy",
    "KeyPath",
    "ValueName",
    "KeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// One system (HOST1) with weakened settings in each hive: WDigest, Remote Desktop and a
/// firewall profile in SYSTEM (and LSA protection on in its other control set); Defender's
/// real-time protection turned off by policy, two exclusions, remote UAC off and a Bypass
/// execution policy in SOFTWARE; and alice's NTUSER.DAT with AMSI off for scripts.
fn security_settings_collection(root: &Path) {
    let dword = |n: u32| n.to_le_bytes().to_vec();
    let system = reg_tree(
        "ROOT",
        &[
            ("Select", "Current", REG_DWORD, dword(1)),
            (
                r"ControlSet001\Control\ComputerName\ComputerName",
                "ComputerName",
                REG_SZ,
                utf16z("HOST1"),
            ),
            (
                r"ControlSet001\Control\SecurityProviders\WDigest",
                "UseLogonCredential",
                REG_DWORD,
                dword(1),
            ),
            (
                r"ControlSet001\Control\Terminal Server",
                "fDenyTSConnections",
                REG_DWORD,
                dword(0),
            ),
            (
                r"ControlSet001\Services\SharedAccess\Parameters\FirewallPolicy\PublicProfile",
                "EnableFirewall",
                REG_DWORD,
                dword(0),
            ),
            (
                r"ControlSet002\Control\Lsa",
                "RunAsPPL",
                REG_DWORD,
                dword(1),
            ),
        ],
    );
    let software = reg_tree(
        "ROOT",
        &[
            (
                r"Microsoft\Windows Defender\Exclusions\Paths",
                r"C:\Users\Public\Tools",
                REG_DWORD,
                dword(0),
            ),
            (
                r"Microsoft\Windows Defender\Exclusions\Extensions",
                ".ps1",
                REG_DWORD,
                dword(0),
            ),
            (
                r"Microsoft\Windows NT\CurrentVersion\ProfileList\S-1-5-21-1-2-3-1001",
                "ProfileImagePath",
                REG_EXPAND_SZ,
                utf16z(r"C:\Users\alice"),
            ),
            (
                r"Microsoft\Windows\CurrentVersion\Policies\System",
                "EnableLUA",
                REG_DWORD,
                dword(1),
            ),
            (
                r"Microsoft\Windows\CurrentVersion\Policies\System",
                "LocalAccountTokenFilterPolicy",
                REG_DWORD,
                dword(1),
            ),
            (
                r"Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell",
                "ExecutionPolicy",
                REG_SZ,
                utf16z("Bypass"),
            ),
            (
                r"Policies\Microsoft\Windows Defender\Real-Time Protection",
                "DisableRealtimeMonitoring",
                REG_DWORD,
                dword(1),
            ),
        ],
    );
    let alice = reg_tree(
        "ROOT",
        &[(
            r"Software\Microsoft\Windows Script\Settings",
            "AmsiEnable",
            REG_DWORD,
            dword(0),
        )],
    );
    let config = root.join("C/Windows/System32/config");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::create_dir_all(root.join("C/Users/alice")).unwrap();
    std::fs::write(config.join("SOFTWARE"), software).unwrap();
    std::fs::write(config.join("SYSTEM"), system).unwrap();
    std::fs::write(root.join("C/Users/alice/NTUSER.DAT"), alice).unwrap();
}

/// `parse-registry security-settings` with `args`, as JSON Lines.
fn security_settings_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "security-settings"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

/// Each record as `hive area setting value weakened`.
fn security_settings_lines(records: &[Value]) -> Vec<String> {
    records
        .iter()
        .map(|r| {
            format!(
                "{} {} {} {} {}",
                r["HiveType"].as_str().unwrap(),
                r["Area"].as_str().unwrap(),
                r["Setting"].as_str().unwrap(),
                r["Value"].as_str().unwrap_or("-"),
                match r["WeakensDefault"].as_bool() {
                    Some(true) => "weakened",
                    Some(false) => "ok",
                    None => "-",
                }
            )
        })
        .collect()
}

#[test]
fn registry_security_settings_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    security_settings_collection(dir.path());
    let records = security_settings_read(&["-d", dir.path().to_str().unwrap()]);
    assert_eq!(
        security_settings_lines(&records),
        [
            "NTUSER.DAT powershell wsh_amsi_enable 0 weakened",
            "SOFTWARE defender defender_disable_realtime_monitoring 1 weakened",
            "SOFTWARE uac uac_enable_lua 1 ok",
            "SOFTWARE uac uac_local_account_token_filter_policy 1 weakened",
            "SOFTWARE powershell powershell_execution_policy Bypass weakened",
            r"SOFTWARE defender defender_exclusion_path C:\Users\Public\Tools weakened",
            "SOFTWARE defender defender_exclusion_extension .ps1 weakened",
            "SYSTEM credentials wdigest_use_logon_credential 1 weakened",
            "SYSTEM remote_access rdp_deny_connections 0 weakened",
            "SYSTEM network firewall_public_profile 0 weakened",
        ]
    );
    for r in &records {
        assert_eq!(r["Host"], "HOST1");
        assert_eq!(r["TransactionLogStatus"], "clean");
        assert_eq!(r["KeyLastWritten"], "2022-02-08T21:03:52Z");
    }
    // the user of NTUSER.DAT, from its folder and the system's ProfileList
    assert_eq!(
        (&records[0]["User"], &records[0]["UserSID"]),
        (&json!("alice"), &json!("S-1-5-21-1-2-3-1001"))
    );
    assert_eq!(records[1]["User"], Value::Null);
    // a policy, its meaning; the control set of SYSTEM's
    assert_eq!(records[1]["Policy"], true);
    assert_eq!(
        records[1]["KeyPath"],
        r"Policies\Microsoft\Windows Defender\Real-Time Protection"
    );
    assert_eq!(records[1]["Meaning"], "real-time protection turned off");
    assert_eq!(records[4]["Policy"], false);
    assert_eq!(
        records[5]["Meaning"],
        "Microsoft Defender doesn't scan files under this path"
    );
    assert_eq!(records[5]["ValueName"], r"C:\Users\Public\Tools");
    assert_eq!(
        (
            &records[7]["ControlSet"],
            &records[7]["IsCurrentControlSet"]
        ),
        (&json!("ControlSet001"), &json!(true))
    );
    assert_eq!(records[2]["ControlSet"], Value::Null);

    // only what weakens the default; every control set
    let weakened = security_settings_read(&[
        "-d",
        dir.path().to_str().unwrap(),
        "--weakened-only",
        "--all-control-sets",
    ]);
    assert_eq!(weakened.len(), 9);
    assert!(weakened.iter().all(|r| r["WeakensDefault"] == true));
    let all = security_settings_read(&["-d", dir.path().to_str().unwrap(), "--all-control-sets"]);
    let other = all.last().unwrap();
    assert_eq!(
        (
            &other["ControlSet"],
            &other["IsCurrentControlSet"],
            &other["Setting"]
        ),
        (
            &json!("ControlSet002"),
            &json!(false),
            &json!("lsa_run_as_ppl")
        )
    );
}

#[test]
fn registry_security_settings_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    security_settings_collection(dir.path());
    let out = dir.path().join("settings.csv");
    let r = run(&[
        "parse-registry",
        "security-settings",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "Hives parsed: 3 (NTUSER.DAT 1, SOFTWARE 1, SYSTEM 1)",
        "Hosts: HOST1",
        "Users: alice",
        "SYSTEM control sets read: 1 (the one in use; --all-control-sets for every one)",
        "Settings: 10 (credentials 1, defender 3, network 1, powershell 2, remote_access 1, uac 2)",
        "Settings that weaken the default (exclusions below; not proof of compromise): 7",
        r"HOST1  credentials  Control\SecurityProviders\WDigest\UseLogonCredential = 1: WDigest keeps cleartext passwords in LSASS memory",
        r"HOST1  alice  powershell  Software\Microsoft\Windows Script\Settings\AmsiEnable = 0: AMSI for Windows Script Host (VBScript and JScript) turned off",
        "Microsoft Defender exclusions: 2",
        r"HOST1  path  C:\Users\Public\Tools",
        "HOST1  extension  .ps1",
        "Keys or values that could not be read: 0",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let header: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, SECURITY_SETTINGS_COLUMNS);
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(rows.len(), 10);
    assert_eq!(&rows[7][2], "SYSTEM");
    assert_eq!(&rows[7][3], "ControlSet001");
    assert_eq!(&rows[7][4], "True");
    assert_eq!(&rows[7][8], "wdigest_use_logon_credential");
    assert_eq!(&rows[7][11], "True");
    assert_eq!(&rows[2][11], "False");
    assert_eq!(&rows[1][12], "True");
    assert_eq!(&rows[0][5], "alice");
}

#[test]
fn registry_security_settings_of_public_hives() {
    // regipy's Windows 7 SYSTEM: WDigest keeping passwords, Remote Desktop without NLA, small
    // event logs; what regipy's wdigest and disable_last_access give
    let regipy = regipy_system();
    let records =
        security_settings_read(&["-d", regipy.path().to_str().unwrap(), "--weakened-only"]);
    assert_eq!(
        security_settings_lines(&records),
        [
            "SYSTEM credentials wdigest_use_logon_credential 1 weakened",
            "SYSTEM remote_access rdp_deny_connections 0 weakened",
            "SYSTEM remote_access rdp_network_level_authentication 0 weakened",
            "SYSTEM evidence event_log_max_size 4194304 weakened",
            "SYSTEM evidence event_log_max_size 4194304 weakened",
            "SYSTEM evidence event_log_max_size 4194304 weakened",
        ]
    );
    assert_eq!(
        records[3]["Meaning"],
        "the Application log keeps up to 4 MiB, less than the default 20 MiB"
    );
    // the DFIR Artifact Museum's dirty Windows 11 SYSTEM with its logs: Windows' defaults
    let w11 = system_hive();
    let records = security_settings_read(&["-d", w11.path().to_str().unwrap()]);
    assert_eq!(records.len(), 13);
    assert!(records.iter().all(|r| r["WeakensDefault"] != true
        && r["Host"] == "GOLDPAPERW11"
        && r["TransactionLogStatus"] == "unchanged"
        && r["Problems"] == json!([])));
    let last_access = records
        .iter()
        .find(|r| r["Setting"] == "ntfs_disable_last_access_update")
        .unwrap();
    assert_eq!(last_access["Value"], "2147483650");
    assert_eq!(
        last_access["Meaning"],
        "last access times updated (system managed, the default since Windows 10 1803)"
    );
}

#[test]
fn registry_security_settings_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&[
        "parse-registry",
        "security-settings",
        "-f",
        sam.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(
        said.contains("a SAM hive, not a SOFTWARE, SYSTEM or NTUSER.DAT hive"),
        "{said}"
    );
    let r = run(&[
        "parse-registry",
        "security-settings",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(
        said.contains("No SOFTWARE, SYSTEM or NTUSER.DAT hives were found"),
        "{said}"
    );
    // the output is never one of the files read
    let collection = tempfile::tempdir().unwrap();
    security_settings_collection(collection.path());
    let software = collection.path().join("C/Windows/System32/config/SOFTWARE");
    let original = std::fs::read(&software).unwrap();
    let r = run(&[
        "parse-registry",
        "security-settings",
        "-f",
        software.to_str().unwrap(),
        "-o",
        software.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(&software).unwrap(), original);
}

/// Write an old-format log that replaces every bin page of a dirty primary.
fn write_replayed_registry_hive(
    directory: &Path,
    name: &str,
    mut primary: Vec<u8>,
    current: &[u8],
) {
    let bins = current.len() - 4096;
    let mut log = current[..512].to_vec();
    put_u32(&mut log, 28, 1);
    let checksum = log[..508]
        .as_chunks::<4>()
        .0
        .iter()
        .fold(0u32, |a, b| a ^ u32::from_le_bytes(*b));
    put_u32(&mut log, 508, checksum);
    log.extend(b"DIRT");
    log.extend(vec![0xff; bins / 4096]);
    log.resize(log.len().next_multiple_of(512), 0);
    log.extend_from_slice(&current[4096..]);
    put_u32(&mut primary, 4, 2);
    let checksum = primary[..508]
        .as_chunks::<4>()
        .0
        .iter()
        .fold(0u32, |a, b| a ^ u32::from_le_bytes(*b));
    put_u32(&mut primary, 508, checksum);
    std::fs::write(directory.join(name), primary).unwrap();
    std::fs::write(directory.join(format!("{name}.LOG")), log).unwrap();
}

#[test]
fn registry_security_settings_keep_amsi_com_replay_provenance() {
    let dir = tempfile::tempdir().unwrap();
    let clsid = "{00000000-0000-0000-0000-000000000001}";
    let provider = format!(r"Microsoft\AMSI\Providers\{clsid}");
    let com = format!(r"Classes\CLSID\{clsid}\InprocServer32");
    let build = |dll: &str| {
        reg_tree(
            "ROOT",
            &[
                (&provider, "Marker", REG_DWORD, 1u32.to_le_bytes().to_vec()),
                (&com, "", REG_SZ, utf16z(dll)),
            ],
        )
    };
    write_replayed_registry_hive(
        dir.path(),
        "SOFTWARE",
        build(r"C:\old.dll"),
        &build(r"C:\new.dll"),
    );
    let records = security_settings_read(&["-f", dir.path().join("SOFTWARE").to_str().unwrap()]);
    let r = records
        .iter()
        .find(|r| r["Setting"] == "amsi_provider")
        .unwrap();
    assert!(r["Meaning"].as_str().unwrap().contains("new.dll"));
    assert_eq!(r["TransactionLogStatus"], "replayed");
}

#[test]
fn registry_security_settings_keeps_list_warnings_on_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("SOFTWARE");
    let mut bytes = reg_tree(
        "ROOT",
        &[(
            r"Microsoft\Windows Defender\Exclusions\Paths",
            "C:\\Data",
            REG_DWORD,
            0u32.to_le_bytes().to_vec(),
        )],
    );
    let name = bytes.windows(5).position(|b| b == b"Paths").unwrap();
    put_u32(&mut bytes, name - 76 + 36, 2);
    std::fs::write(&hive, bytes).unwrap();
    for format in ["csv", "json", "jsonl"] {
        let r = run(&[
            "parse-registry",
            "security-settings",
            "-f",
            hive.to_str().unwrap(),
            "-t",
            format,
        ]);
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert!(r.stderr.contains("Exclusions\\Paths"), "{}", r.stderr);
        if format != "csv" {
            assert_eq!(json_documents(&r.stdout).len(), 1);
        }
    }
}

#[test]
fn registry_security_settings_leaves_unknown_current_control_set_null() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("SYSTEM");
    std::fs::write(
        &hive,
        reg_tree(
            "ROOT",
            &[(
                r"ControlSet001\Control\SecurityProviders\WDigest",
                "UseLogonCredential",
                REG_DWORD,
                1u32.to_le_bytes().to_vec(),
            )],
        ),
    )
    .unwrap();
    let r = run(&[
        "parse-registry",
        "security-settings",
        "-f",
        hive.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let rows = jsonl_records(&r.stdout);
    assert_eq!(rows.len(), 1);
    assert!(rows[0]["IsCurrentControlSet"].is_null());
    assert!(r.stderr.contains("Select\\Current"), "{}", r.stderr);
}

// --- parse-registry office ----------------------------------------------------------------------

/// `parse-registry office`'s CSV columns.
const OFFICE_COLUMNS: [&str; 24] = [
    "Host",
    "User",
    "UserSID",
    "SourcePath",
    "OfficeVersion",
    "App",
    "List",
    "Account",
    "Policy",
    "MRU-Position",
    "IsMostRecent",
    "Path",
    "LastOpenedUTC",
    "TrustedAtUTC",
    "MacrosEnabled",
    "TrustFlags",
    "Setting",
    "Value",
    "KeyPath",
    "KeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// A trust record: trusted at `when`, with `flags`.
fn office_trust(when: u64, flags: u32) -> Vec<u8> {
    let mut b = when.to_le_bytes().to_vec();
    b.extend([0, 0xd0, 0x88, 0xc3, 0x10, 0, 0, 0, 0x32, 0x82, 0xa2, 0x01]);
    b.extend(flags.to_le_bytes());
    b
}

/// An Office MRU item opened at `when`.
fn office_item(when: u64, path: &str) -> Vec<u8> {
    utf16z(&format!("[F00000000][T{when:016X}][O00000000]*{path}"))
}

/// One system's SYSTEM and SOFTWARE hives (its name, alice's profile), alice's NTUSER.DAT with
/// Word 2016's recent documents and folder, a Microsoft 365 account's document, a document whose
/// macros she enabled and one she only edited, all macros enabled, a trusted location she added,
/// a policy that blocks macros in Excel, and an Excel 2013 item cut short; and bob's NTUSER.DAT
/// without Office.
fn office_collection(root: &Path) {
    let word = r"Software\Microsoft\Office\16.0\Word";
    let records = format!(r"{word}\Security\Trusted Documents\TrustRecords");
    let location = format!(r"{word}\Security\Trusted Locations\Location99");
    let policy = r"Software\Policies\Microsoft\Office\16.0\Excel\Security";
    let alice = reg_tree(
        "ROOT",
        &[
            (
                &format!(r"{word}\File MRU"),
                "Item 1",
                REG_SZ,
                office_item(LINKED, r"C:\Users\alice\Downloads\invoice.docm"),
            ),
            (
                &format!(r"{word}\File MRU"),
                "Item 2",
                REG_SZ,
                office_item(LINKED - DAY, r"C:\Users\alice\notes.docx"),
            ),
            (
                &format!(r"{word}\File MRU"),
                "Max Display",
                REG_DWORD,
                25u32.to_le_bytes().to_vec(),
            ),
            (
                &format!(r"{word}\Place MRU"),
                "Item 1",
                REG_SZ,
                office_item(LINKED, r"C:\Users\alice\Downloads\"),
            ),
            (
                &format!(r"{word}\User MRU\ADAL_0123\File MRU"),
                "Item 1",
                REG_SZ,
                office_item(LINKED + DAY, "https://example.sharepoint.com/report.docx"),
            ),
            (
                &records,
                "%USERPROFILE%/Downloads/invoice.docm",
                REG_BINARY,
                office_trust(LINKED, 0x7fff_ffff),
            ),
            (
                &records,
                "%USERPROFILE%/notes.docx",
                REG_BINARY,
                office_trust(LINKED - DAY, 1),
            ),
            (
                &format!(r"{word}\Security"),
                "VBAWarnings",
                REG_DWORD,
                1u32.to_le_bytes().to_vec(),
            ),
            (&location, "Path", REG_SZ, utf16z(r"C:\Users\Public\")),
            (&location, "Date", REG_SZ, utf16z("2/8/2022 9:03 PM")),
            (
                policy,
                "VBAWarnings",
                REG_DWORD,
                4u32.to_le_bytes().to_vec(),
            ),
            (
                r"Software\Microsoft\Office\15.0\Excel\File MRU",
                "Item 1",
                REG_SZ,
                utf16z(r"[F00000000][T01D0000000000000]C:\cut.xlsx"),
            ),
        ],
    );
    let bob = reg_tree(
        "ROOT",
        &[(
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
            "Start_TrackProgs",
            REG_DWORD,
            1u32.to_le_bytes().to_vec(),
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

/// `parse-registry office` with `args`, as JSON Lines.
fn office_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "office"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

/// Each record's list, version, application, account, policy, place, path or setting, and
/// times, macros and value.
fn office_lines(records: &[Value]) -> Vec<String> {
    let text = |v: &Value| v.as_str().map_or("-".to_string(), str::to_string);
    records
        .iter()
        .map(|r| {
            format!(
                "{} {} {} {} {} {} | {} {} | {} {} {} {}",
                text(&r["List"]),
                text(&r["OfficeVersion"]),
                text(&r["App"]),
                text(&r["Account"]),
                r["Policy"],
                r["MRU-Position"],
                text(&r["Path"]),
                text(&r["Setting"]),
                text(&r["LastOpened"]),
                text(&r["TrustedAt"]),
                r["MacrosEnabled"],
                text(&r["Value"]),
            )
        })
        .collect()
}

#[test]
fn registry_office_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    office_collection(dir.path());
    let records = office_read(&["-d", dir.path().to_str().unwrap()]);
    assert_eq!(
        office_lines(&records),
        [
            r"File MRU 15.0 Excel - false 0 | - - | - - null -",
            r"File MRU 16.0 Word - false 0 | C:\Users\alice\Downloads\invoice.docm - | 2022-02-08T21:03:52Z - null -",
            r"File MRU 16.0 Word - false 1 | C:\Users\alice\notes.docx - | 2022-02-07T21:03:52Z - null -",
            r"Place MRU 16.0 Word - false 0 | C:\Users\alice\Downloads\ - | 2022-02-08T21:03:52Z - null -",
            "File MRU 16.0 Word ADAL_0123 false 0 | https://example.sharepoint.com/report.docx - | 2022-02-09T21:03:52Z - null -",
            "TrustRecords 16.0 Word - false null | %USERPROFILE%/Downloads/invoice.docm - | - 2022-02-08T21:03:52Z true -",
            "TrustRecords 16.0 Word - false null | %USERPROFILE%/notes.docx - | - 2022-02-07T21:03:52Z false -",
            "Security 16.0 Word - false null | - VBAWarnings | - - null 1 (all macros enabled)",
            r"Trusted Locations 16.0 Word - false null | C:\Users\Public\ Location99 | - - null Date=2/8/2022 9:03 PM",
            "Security 16.0 Excel - true null | - VBAWarnings | - - null 4 (macros disabled without notification)",
        ]
    );
    for r in &records {
        assert_eq!(r["Host"], "HOST1");
        assert_eq!(r["User"], "alice");
        assert_eq!(r["UserSID"], "S-1-5-21-1-2-3-1001");
        assert_eq!(r["TransactionLogStatus"], "clean");
    }
    assert_eq!(records[1]["IsMostRecent"], true);
    assert_eq!(records[2]["IsMostRecent"], false);
    assert_eq!(records[5]["TrustFlags"], "0x7FFFFFFF");
    assert_eq!(records[6]["TrustFlags"], "0x00000001");
    // an item without its * keeps its bytes
    assert_eq!(records[0]["Problems"], json!(["no * before the path"]));
    assert!(
        records[0]["DataRaw"]
            .as_str()
            .unwrap()
            .starts_with("5b0046003000")
    );
    assert_eq!(
        records[9]["KeyPath"],
        r"Software\Policies\Microsoft\Office\16.0\Excel\Security"
    );
}

#[test]
fn registry_office_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    office_collection(dir.path());
    let out = dir.path().join("office.csv");
    let r = run(&[
        "parse-registry",
        "office",
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
        "Hives without Office: 1",
        "Office versions: 15.0, 16.0",
        "Entries: 10 (File MRU 4, Place MRU 1, Security 2, TrustRecords 2, Trusted Locations 1)",
        "Documents whose macros were enabled (UTC trusted, user, application): 1",
        "2022-02-08T21:03:52Z  alice  Word 16.0: %USERPROFILE%/Downloads/invoice.docm",
        "Settings that let macros run more freely: 1",
        "alice  Word 16.0: VBAWarnings = 1 (all macros enabled)",
        "Trusted locations added by users or set by policy: 1",
        r"alice  Word 16.0: C:\Users\Public\ (Date=2/8/2022 9:03 PM)",
        "2022-02-09T21:03:52Z  alice  Word 16.0: https://example.sharepoint.com/report.docx",
        "Entries or keys that could not be read: 1",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let header: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, OFFICE_COLUMNS);
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(rows.len(), 10);
    assert_eq!(&rows[1][6], "File MRU");
    assert_eq!(&rows[1][11], r"C:\Users\alice\Downloads\invoice.docm");
    assert_eq!(&rows[1][12], "2022-02-08T21:03:52Z");
    assert_eq!(&rows[5][14], "true");
    assert_eq!(&rows[7][16], "VBAWarnings");
    assert_eq!(&rows[9][8], "true");
}

#[test]
fn registry_office_of_public_hives() {
    let dir = tempfile::tempdir().unwrap();
    // Plaso's Windows 7 NTUSER.DAT: Word 2010's documents (as Plaso's officemru and
    // dissect.target's Office MRU parser read them) and three documents trusted for editing
    unpack_registry("plaso", &["NTUSER-WIN7.DAT"], dir.path());
    let hive = dir.path().join("NTUSER-WIN7.DAT");
    let records = office_read(&["-f", hive.to_str().unwrap()]);
    let lists: Vec<String> = office_lines(&records)
        .into_iter()
        .filter(|l| !l.starts_with("Trusted Locations"))
        .collect();
    let docs = r"C:\Users\nfury\Documents";
    assert_eq!(
        lists,
        [
            format!(r"File MRU 14.0 Word - false 0 | {docs}\StarFury\StarFury\SA-23E Mitchell-Hyundyne Starfury.docx - | 2012-03-13T18:27:15.083Z - null -"),
            format!(r"File MRU 14.0 Word - false 1 | {docs}\StarFury\StarFury\Earthforce SA-26 Thunderbolt Star Fury.docx - | 2012-03-12T20:53:06.159Z - null -"),
            format!(r"File MRU 14.0 Word - false 2 | {docs}\StarFury\StarFury\StarFury.docx - | 2012-03-12T20:52:27.092Z - null -"),
            format!(r"File MRU 14.0 Word - false 3 | {docs}\VIBRANIUM.docx - | 2012-03-09T15:41:46.975Z - null -"),
            format!(r"File MRU 14.0 Word - false 4 | {docs}\ADAMANTIUM-Background.docx - | 2012-03-07T23:30:58.035Z - null -"),
            format!(r"Place MRU 14.0 Word - false 0 | {docs}\StarFury\StarFury\ - | 2012-03-13T18:27:15.080Z - null -"),
            format!(r"Place MRU 14.0 Word - false 1 | {docs}\ - | 2012-03-09T15:41:46.972Z - null -"),
            "TrustRecords 14.0 Word - false null | %USERPROFILE%/Documents/StarFury/StarFury/StarFury.docx - | - 2012-03-12T23:50:22Z false -".to_string(),
            "TrustRecords 14.0 Word - false null | %USERPROFILE%/Documents/StarFury/StarFury/Earthforce%20SA-26%20Thunderbolt%20Star%20Fury.docx - | - 2012-03-12T20:52:00Z false -".to_string(),
            "TrustRecords 14.0 Word - false null | %USERPROFILE%/Documents/StarFury/StarFury/SA-23E%20Mitchell-Hyundyne%20Starfury.docx - | - 2012-03-12T20:52:00Z false -".to_string(),
        ]
    );
    assert_eq!(records.len(), 23);
    assert!(records.iter().all(|r| r["Problems"] == json!([])));

    // regipy's Windows 7 NTUSER.DAT: Excel 2010's lists
    unpack_registry("regipy", &["NTUSER.DAT"], dir.path());
    let records = office_read(&["-f", dir.path().join("NTUSER.DAT").to_str().unwrap()]);
    let excel: Vec<&Value> = records.iter().filter(|r| r["List"] == "File MRU").collect();
    assert_eq!(excel.len(), 4);
    assert_eq!(excel[0]["App"], "Excel");
    assert_eq!(excel[0]["LastOpened"], "2012-04-04T15:43:16.848Z");
}

#[test]
fn registry_office_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM", "NTUSER.DAT"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&[
        "parse-registry",
        "office",
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
    let empty = tempfile::tempdir().unwrap();
    let r = run(&[
        "parse-registry",
        "office",
        "-d",
        empty.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("No NTUSER.DAT hives were found"), "{said}");
    // the output is never one of the files read
    let ntuser = dir.path().join("NTUSER.DAT");
    let original = std::fs::read(&ntuser).unwrap();
    let r = run(&[
        "parse-registry",
        "office",
        "-f",
        ntuser.to_str().unwrap(),
        "-o",
        ntuser.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(&ntuser).unwrap(), original);
}

#[test]
fn registry_office_keeps_list_warnings_on_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("NTUSER.DAT");
    let mut bytes = reg_tree(
        "ROOT",
        &[(
            r"Software\Microsoft\Office\16.0\Word\File MRU",
            "Item 1",
            REG_SZ,
            utf16z("[T0]*C:\\file.docx"),
        )],
    );
    let name = bytes.windows(8).position(|b| b == b"File MRU").unwrap();
    put_u32(&mut bytes, name - 76 + 36, 2);
    std::fs::write(&hive, bytes).unwrap();
    for format in ["csv", "json", "jsonl"] {
        let r = run(&[
            "parse-registry",
            "office",
            "-f",
            hive.to_str().unwrap(),
            "-t",
            format,
        ]);
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert!(r.stderr.contains("File MRU"), "{}", r.stderr);
        if format != "csv" {
            assert_eq!(json_documents(&r.stdout).len(), 1);
        }
    }
}

#[test]
fn registry_office_does_not_classify_wrong_type_macro_settings_as_weak() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("NTUSER.DAT");
    std::fs::write(
        &hive,
        reg_tree(
            "ROOT",
            &[(
                r"Software\Microsoft\Office\16.0\Word\Security",
                "VBAWarnings",
                REG_SZ,
                utf16z("1"),
            )],
        ),
    )
    .unwrap();
    let r = run(&[
        "parse-registry",
        "office",
        "-f",
        hive.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(!r.stdout.contains("VBAWarnings = 1"), "{}", r.stdout);
    let rows = office_read(&["-f", hive.to_str().unwrap()]);
    assert_eq!(rows.len(), 1);
    assert!(
        rows[0]["Problems"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p.as_str().unwrap().contains("REG_SZ"))
    );
    assert!(rows[0]["DataRaw"].is_string());
}

// --- parse-registry auditpol --------------------------------------------------------------------

/// `parse-registry auditpol` with `args`, as JSON Lines.
fn auditpol_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "auditpol"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

/// The DFIR Artifact Museum's Windows 11 system: its dirty SYSTEM and SECURITY with their logs,
/// where Windows keeps them.
fn auditpol_collection() -> tempfile::TempDir {
    let hive = system_hive();
    let config = hive.path().join("HOST/C/Windows/System32/config");
    unpack_registry(
        "rathbun_w11",
        &["SECURITY", "SECURITY.LOG1", "SECURITY.LOG2"],
        &config,
    );
    hive
}

#[test]
fn registry_auditpol_of_a_public_hive() {
    let dir = auditpol_collection();
    let records = auditpol_read(&["-d", dir.path().to_str().unwrap()]);
    assert_eq!(records.len(), 59);
    for r in &records {
        assert_eq!(r["Host"], "GOLDPAPERW11");
        assert_eq!(r["Layout"], "subcategories");
        assert_eq!(r["Problems"], json!([]));
        assert_eq!(r["DataRaw"], Value::Null);
        assert_eq!(r["AuditingEnabled"], Value::Null);
    }
    let find = |name: &str| {
        records
            .iter()
            .find(|r| r["Subcategory"] == name)
            .unwrap_or_else(|| panic!("{name}"))
    };
    // Windows' order: Special Logon is Logon/Logoff's index 4, before IPsec Quick Mode
    let special = find("Special Logon");
    assert_eq!(special["Category"], "Logon/Logoff");
    assert_eq!(special["Setting"], "success");
    assert_eq!(
        special["SubcategoryGUID"],
        "0CCE921B-69AE-11D9-BED3-505054503030"
    );
    assert_eq!(special["SubcategoryID"], "%%12548");
    assert_eq!(special["Events"], json!([4672, 4964]));
    assert_eq!(find("IPsec Quick Mode")["Setting"], "none");
    assert_eq!(
        find("Network Policy Server")["Setting"],
        "success_and_failure"
    );
    let process = find("Process Creation");
    assert_eq!(
        (&process["Setting"], &process["Success"], &process["Events"]),
        (&json!("none"), &json!(false), &json!([4688]))
    );
    let audited: Vec<&str> = records
        .iter()
        .filter(|r| r["Setting"] != "none")
        .map(|r| r["Subcategory"].as_str().unwrap())
        .collect();
    assert_eq!(
        audited,
        [
            "Security State Change",
            "System Integrity",
            "Other System Events",
            "Logon",
            "Logoff",
            "Account Lockout",
            "Special Logon",
            "Network Policy Server",
            "Audit Policy Change",
            "Authentication Policy Change",
            "User Account Management",
            "Security Group Management",
        ]
    );
    assert_eq!(
        records
            .iter()
            .map(|r| r["Category"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        9
    );
    assert!(
        records[0]["KeyLastWritten"]
            .as_str()
            .unwrap()
            .starts_with("2022-02-05T21:49:55")
    );
    assert_ne!(records[0]["TransactionLogStatus"], "clean");
}

#[test]
fn registry_auditpol_csv_and_summary() {
    let dir = auditpol_collection();
    let security = dir.path().join("HOST/C/Windows/System32/config/SECURITY");
    let out = dir.path().join("auditpol.csv");
    let r = run(&[
        "parse-registry",
        "auditpol",
        "-f",
        security.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "SECURITY hives parsed: 1",
        "Hosts: GOLDPAPERW11",
        "Settings: 59 (12 audited)",
        "Layout: subcategories; 12 of 59 audited; last written 2022-02-05T21:49:55",
        "Success only: Security State Change (4608, 4616); Logoff (4634, 4647)",
        "Not audited in captured policy (past policy may differ): Security System Extension (4697)",
        "Process Creation (4688)",
        "Other Object Access Events (4698, 4699, 4700, 4701, 4702)",
        "Values that could not be read or decoded: 0",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    let (header, rows) = csv_rows(&out);
    assert_eq!(
        header,
        [
            "Host",
            "SourcePath",
            "Layout",
            "Category",
            "Subcategory",
            "SubcategoryGUID",
            "SubcategoryID",
            "Setting",
            "Success",
            "Failure",
            "AuditingEnabled",
            "Events",
            "KeyLastWrittenUTC",
            "DataRaw",
            "TransactionLogStatus",
            "LogsApplied",
            "Problems",
        ]
    );
    assert_eq!(rows.len(), 59);
    let logon = rows.iter().find(|r| r["Subcategory"] == "Logon").unwrap();
    assert_eq!(logon["Setting"], "success_and_failure");
    assert_eq!(
        (logon["Success"].as_str(), logon["Failure"].as_str()),
        ("True", "True")
    );
    assert_eq!(logon["Events"], "4624|4625|4648");
    assert_eq!(logon["SubcategoryID"], "%%12544");
}

#[test]
fn registry_auditpol_layouts_and_errors() {
    let dir = tempfile::tempdir().unwrap();
    // Windows XP's layout with auditing off; an unknown layout; no PolAdtEv
    let mut xp = 0u32.to_le_bytes().to_vec();
    for v in [3u32, 1, 2, 0, 0, 3, 3, 0, 1, 9] {
        xp.extend(v.to_le_bytes());
    }
    for (name, data) in [
        ("xp", Some(xp)),
        ("odd", Some(vec![1u8, 2, 3])),
        ("none", None),
    ] {
        let at = dir.path().join(name);
        std::fs::create_dir_all(&at).unwrap();
        let values: Vec<(&str, &str, u32, Vec<u8>)> = match data {
            Some(data) => vec![(r"Policy\PolAdtEv", "", 0, data)],
            None => vec![(r"Policy\PolRevision", "", 0, vec![0x12, 0, 1, 0])],
        };
        std::fs::write(at.join("SECURITY"), reg_tree("ROOT", &values)).unwrap();
    }
    let records = auditpol_read(&["-d", dir.path().to_str().unwrap()]);
    let odd: Vec<&Value> = records
        .iter()
        .filter(|r| r["SourcePath"].as_str().unwrap().contains("odd"))
        .collect();
    assert_eq!(odd.len(), 1);
    assert_eq!(
        (&odd[0]["Layout"], &odd[0]["Category"], &odd[0]["DataRaw"]),
        (&Value::Null, &Value::Null, &json!("010203"))
    );
    assert_eq!(
        odd[0]["Problems"],
        json!(["an unknown layout of 3 bytes: shorter than the 12-byte header"])
    );
    let xp: Vec<&Value> = records
        .iter()
        .filter(|r| r["SourcePath"].as_str().unwrap().contains("xp"))
        .collect();
    assert_eq!(xp.len(), 9);
    assert!(xp.iter().all(|r| r["Layout"] == "categories"
        && r["AuditingEnabled"] == false
        && r["Subcategory"] == Value::Null));
    assert_eq!(
        (&xp[0]["Category"], &xp[0]["Setting"]),
        (&json!("System"), &json!("success_and_failure"))
    );
    assert_eq!(xp[2]["Setting"], "failure");
    let r = run(&[
        "parse-registry",
        "auditpol",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "SECURITY hives parsed: 3",
        r"Hives without Policy\PolAdtEv: 1",
        "Auditing is off: no category is audited",
        "Layout: categories; 0 of 9 audited",
        "Values that could not be read or decoded: 1",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    // a SAM given as SECURITY, a directory without SECURITY hives
    let sams = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], sams.path());
    let r = run(&[
        "parse-registry",
        "auditpol",
        "-f",
        sams.path().join("SAM").to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stderr.contains("not a SECURITY hive"), "{}", r.stderr);
    let r = run(&[
        "parse-registry",
        "auditpol",
        "-d",
        sams.path().to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr.contains("No SECURITY hives were found"),
        "{}",
        r.stderr
    );
    // the output is never the hive read, even with -C
    let security = dir.path().join("xp/SECURITY");
    let original = std::fs::read(&security).unwrap();
    let r = run(&[
        "parse-registry",
        "auditpol",
        "-f",
        security.to_str().unwrap(),
        "-o",
        security.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(&security).unwrap(), original);
}

#[test]
fn registry_auditpol_does_not_claim_unknown_flags_are_audited() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("SECURITY");
    let mut data = Vec::new();
    for n in [0x100u32, 1, 16] {
        data.extend(n.to_le_bytes());
    }
    for n in [9u16, 0, 1] {
        data.extend(n.to_le_bytes());
    }
    std::fs::write(
        &hive,
        reg_tree("ROOT", &[(r"Policy\PolAdtEv", "", 0, data)]),
    )
    .unwrap();
    let rows = auditpol_read(&["-f", hive.to_str().unwrap()]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["Setting"], "0x9");
    assert!(rows[0]["Success"].is_null());
    assert!(rows[0]["Failure"].is_null());
    let r = run(&[
        "parse-registry",
        "auditpol",
        "-f",
        hive.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.contains("1 (0 audited)"), "{}", r.stdout);
}

// --- parse-registry app-compat -------------------------------------------------------------------

/// `parse-registry app-compat`'s CSV columns.
const APPCOMPAT_COLUMNS: [&str; 22] = [
    "Host",
    "SourcePath",
    "HiveType",
    "User",
    "UserSID",
    "Source",
    "Scope",
    "Path",
    "Layers",
    "LastSeenUTC",
    "SdbGUID",
    "SdbPath",
    "SdbInstalledUTC",
    "Details",
    "KeyPath",
    "KeyLastWrittenUTC",
    "ValueName",
    "WOW64",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// One system's SYSTEM hive (its name), SOFTWARE hive (alice's profile; a program run as
/// administrator; a program that loaded a CIT module; a custom shim database installed and
/// applied to notepad.exe) and alice's NTUSER.DAT (her own layers, and two programs the Program
/// Compatibility Assistant saw: one on removable media).
fn appcompat_collection(root: &Path) {
    let flags = r"Microsoft\Windows NT\CurrentVersion\AppCompatFlags";
    let guid = "{8E9D8F3C-6B7A-4D5C-3E2F-1A0B9C8D7E6F}";
    let sdb = format!(r"{flags}\InstalledSDB\{guid}");
    let software = reg_tree(
        "ROOT",
        &[
            (
                r"Microsoft\Windows NT\CurrentVersion\ProfileList\S-1-5-21-1-2-3-1001",
                "ProfileImagePath",
                REG_EXPAND_SZ,
                utf16z(r"C:\Users\alice"),
            ),
            (
                &format!(r"{flags}\Layers"),
                r"C:\Tools\agent.exe",
                REG_SZ,
                utf16z("~ RUNASADMIN"),
            ),
            (
                &format!(r"{flags}\CIT\Module\System32/mrt100.dll"),
                r"\Device\HarddiskVolume1\Windows\System32\backgroundTaskHost.exe",
                REG_QWORD,
                LINKED.to_le_bytes().to_vec(),
            ),
            (
                &format!(r"{flags}\CIT\Module\System32/mrt100.dll"),
                "OverflowQuota",
                REG_QWORD,
                LINKED.to_le_bytes().to_vec(),
            ),
            (
                &sdb,
                "DatabasePath",
                REG_SZ,
                utf16z(r"C:\ProgramData\shim.sdb"),
            ),
            (&sdb, "DatabaseDescription", REG_SZ, utf16z("Updater")),
            (
                &sdb,
                "DatabaseInstallTimeStamp",
                REG_QWORD,
                (LINKED - DAY).to_le_bytes().to_vec(),
            ),
            (
                &sdb,
                "DatabaseType",
                REG_DWORD,
                0x0001_0000u32.to_le_bytes().to_vec(),
            ),
            (
                &format!(r"{flags}\Custom\notepad.exe"),
                &format!("{guid}.sdb"),
                REG_QWORD,
                (LINKED - DAY).to_le_bytes().to_vec(),
            ),
        ],
    );
    let user_flags = format!(r"Software\{flags}");
    let store = format!(r"{user_flags}\Compatibility Assistant\Store");
    let mut sacp = b"SACP".to_vec();
    sacp.extend([1, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0x28, 0, 0, 0]);
    let ntuser = reg_tree(
        "ROOT",
        &[
            (
                &format!(r"{user_flags}\Layers"),
                r"C:\Users\alice\game.exe",
                REG_SZ,
                utf16z("~ HIGHDPIAWARE"),
            ),
            (
                &store,
                r"C:\Users\alice\Downloads\tool.exe",
                REG_BINARY,
                sacp.clone(),
            ),
            (&store, "SIGN.MEDIA=1FF3254 setup64.exe", REG_BINARY, sacp),
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
        ],
    );
    let config = root.join("C/Windows/System32/config");
    let profile = root.join("C/Users/alice");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::create_dir_all(&profile).unwrap();
    std::fs::write(config.join("SYSTEM"), system).unwrap();
    std::fs::write(config.join("SOFTWARE"), software).unwrap();
    std::fs::write(profile.join("NTUSER.DAT"), ntuser).unwrap();
}

/// `parse-registry app-compat` with `args`, as JSON Lines.
fn appcompat_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "app-compat"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_appcompat_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    appcompat_collection(dir.path());
    let records = appcompat_read(&["-d", dir.path().to_str().unwrap()]);
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} {} {} {} | {} | {} | {} {} {}",
                r["HiveType"].as_str().unwrap(),
                r["Source"].as_str().unwrap(),
                r["Scope"].as_str().unwrap(),
                r["User"].as_str().unwrap_or("-"),
                r["Path"].as_str().unwrap_or("-"),
                r["Layers"].as_str().unwrap_or("-"),
                r["LastSeen"].as_str().unwrap_or("-"),
                r["SdbPath"].as_str().unwrap_or("-"),
                r["SdbInstalled"].as_str().unwrap_or("-"),
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            // the hives in path order (C/Users before C/Windows)
            r"NTUSER.DAT layers user alice | C:\Users\alice\game.exe | ~ HIGHDPIAWARE | - - -",
            r"NTUSER.DAT pca_store user alice | C:\Users\alice\Downloads\tool.exe | - | - - -",
            "NTUSER.DAT pca_store user alice | setup64.exe | - | - - -",
            r"SOFTWARE layers machine - | C:\Tools\agent.exe | ~ RUNASADMIN | - - -",
            r"SOFTWARE cit_module machine - | \Device\HarddiskVolume1\Windows\System32\backgroundTaskHost.exe | - | 2022-02-08T21:03:52Z - -",
            r"SOFTWARE installed_sdb machine - | - | - | - C:\ProgramData\shim.sdb 2022-02-07T21:03:52Z",
            r"SOFTWARE custom_sdb machine - | notepad.exe | - | - C:\ProgramData\shim.sdb 2022-02-07T21:03:52Z",
        ]
    );
    for r in &records {
        assert_eq!(r["Host"], "HOST1");
        assert_eq!(r["TransactionLogStatus"], "clean");
        assert_eq!(r["KeyLastWritten"], "2022-02-08T21:03:52Z");
    }
    for r in &records[..3] {
        assert_eq!(r["UserSID"], "S-1-5-21-1-2-3-1001");
    }
    assert_eq!(records[3]["UserSID"], Value::Null);
    // the Store data is kept, and a program on removable media keeps its media id
    assert!(
        records[1]["DataRaw"]
            .as_str()
            .unwrap()
            .starts_with("53414350")
    );
    assert_eq!(records[2]["Details"], json!({"SignMedia": "1FF3254"}));
    assert_eq!(records[2]["ValueName"], "SIGN.MEDIA=1FF3254 setup64.exe");
    assert_eq!(
        records[4]["Details"],
        json!({"Module": "System32/mrt100.dll"})
    );
    let guid = "{8E9D8F3C-6B7A-4D5C-3E2F-1A0B9C8D7E6F}";
    assert_eq!(records[5]["SdbGUID"], guid);
    assert_eq!(
        records[5]["Details"],
        json!({"DatabaseDescription": "Updater", "DatabaseType": "0x10000"})
    );
    assert_eq!(records[6]["SdbGUID"], guid);
    assert_eq!(records[6]["ValueName"], format!("{guid}.sdb"));
}

#[test]
fn registry_appcompat_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    appcompat_collection(dir.path());
    let out = dir.path().join("appcompat.csv");
    let r = run(&[
        "parse-registry",
        "app-compat",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "Hives parsed: 2 (NTUSER.DAT 1, SOFTWARE 1)",
        "Hosts: HOST1",
        "Users: alice",
        "Records: 7 (cit_module 1, custom_sdb 1, installed_sdb 1, layers 2, pca_store 2)",
        "Custom shim databases (host, installed UTC, GUID, path): 2",
        r"HOST1  2022-02-07T21:03:52Z  {8E9D8F3C-6B7A-4D5C-3E2F-1A0B9C8D7E6F}  C:\ProgramData\shim.sdb  applied to notepad.exe",
        "Programs with compatibility layers (host, user, program, layers): 2",
        r"HOST1  alice  C:\Users\alice\game.exe  ~ HIGHDPIAWARE",
        r"HOST1  (machine)  C:\Tools\agent.exe  ~ RUNASADMIN",
        r"2022-02-08T21:03:52Z  HOST1  \Device\HarddiskVolume1\Windows\System32\backgroundTaskHost.exe  (System32/mrt100.dll)",
        "Records or keys that could not be read: 0",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    let (header, rows) = csv_rows(&out);
    assert_eq!(header, APPCOMPAT_COLUMNS);
    assert_eq!(rows.len(), 7);
    assert_eq!(rows[2]["Details"], "SignMedia=1FF3254");
    assert_eq!(rows[4]["LastSeenUTC"], "2022-02-08T21:03:52Z");
    assert_eq!(rows[5]["SdbInstalledUTC"], "2022-02-07T21:03:52Z");
    assert_eq!(rows[6]["Scope"], "machine");
    assert_eq!(rows[0]["WOW64"], "False");
}

#[test]
fn registry_appcompat_of_public_hives() {
    // the DFIR Artifact Museum's dirty Windows 11 NTUSER.DAT with its logs: 11 programs the
    // Program Compatibility Assistant saw run, each with its SACP data, one on removable media
    let dir = tempfile::tempdir().unwrap();
    unpack_registry(
        "rathbun_w11",
        &["NTUSER.DAT", "ntuser.dat.LOG1", "ntuser.dat.LOG2"],
        dir.path(),
    );
    let records = appcompat_read(&["-f", dir.path().join("NTUSER.DAT").to_str().unwrap()]);
    assert_eq!(records.len(), 11);
    assert!(records.iter().all(|r| r["Source"] == "pca_store"
        && r["Problems"] == json!([])
        && r["LastSeen"].is_null()
        && r["DataRaw"].as_str().unwrap().starts_with("53414350")));
    let media: Vec<&Value> = records
        .iter()
        .filter(|r| r["Details"]["SignMedia"].is_string())
        .collect();
    assert_eq!(media.len(), 1);
    assert_eq!(
        (&media[0]["Path"], &media[0]["Details"]["SignMedia"]),
        (&json!("setup64.exe"), &json!("1FF3254"))
    );
    assert_eq!(
        records
            .iter()
            .filter(|r| r["Path"]
                .as_str()
                .unwrap()
                .ends_with(r"\WindowsTerminal.exe"))
            .count(),
        2
    );
    assert_eq!(
        records[0]["KeyLastWritten"],
        "2022-02-07T07:37:00.507738100Z"
    );
}

#[test]
fn registry_appcompat_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&[
        "parse-registry",
        "app-compat",
        "-f",
        sam.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("not a SOFTWARE or NTUSER.DAT hive"), "{said}");
    let r = run(&[
        "parse-registry",
        "app-compat",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(
        said.contains("No SOFTWARE or NTUSER.DAT hives were found"),
        "{said}"
    );
    // the output is never one of the files read: a user hive's system's SOFTWARE included
    let collection = tempfile::tempdir().unwrap();
    appcompat_collection(collection.path());
    let software = collection.path().join("C/Windows/System32/config/SOFTWARE");
    let original = std::fs::read(&software).unwrap();
    let r = run(&[
        "parse-registry",
        "app-compat",
        "-f",
        collection
            .path()
            .join("C/Users/alice/NTUSER.DAT")
            .to_str()
            .unwrap(),
        "-o",
        software.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(&software).unwrap(), original);
}

#[test]
fn registry_appcompat_custom_shims_keep_installed_path_replay() {
    let dir = tempfile::tempdir().unwrap();
    let base = r"Microsoft\Windows NT\CurrentVersion\AppCompatFlags";
    let guid = "{00000000-0000-0000-0000-000000000001}";
    let installed = format!(r"{base}\InstalledSDB\{guid}");
    let custom = format!(r"{base}\Custom\target.exe");
    let name = format!("{guid}.sdb");
    let build = |path: &str| {
        reg_tree(
            "ROOT",
            &[
                (&installed, "DatabasePath", REG_SZ, utf16z(path)),
                (&custom, &name, REG_BINARY, vec![]),
            ],
        )
    };
    write_replayed_registry_hive(
        dir.path(),
        "SOFTWARE",
        build(r"C:\old.sdb"),
        &build(r"C:\new.sdb"),
    );
    let rows = appcompat_read(&["-f", dir.path().join("SOFTWARE").to_str().unwrap()]);
    let row = rows.iter().find(|r| r["Source"] == "custom_sdb").unwrap();
    assert_eq!(row["SdbPath"], r"C:\new.sdb");
    assert_eq!(row["TransactionLogStatus"], "replayed");
}

#[test]
fn registry_appcompat_keeps_list_warnings_on_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("SOFTWARE");
    let mut bytes = reg_tree(
        "ROOT",
        &[(
            r"Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers",
            r"C:\tool.exe",
            REG_SZ,
            utf16z("~ RUNASADMIN"),
        )],
    );
    let name = bytes.windows(6).position(|b| b == b"Layers").unwrap();
    put_u32(&mut bytes, name - 76 + 36, 2);
    std::fs::write(&hive, bytes).unwrap();
    for format in ["csv", "json", "jsonl"] {
        let r = run(&[
            "parse-registry",
            "app-compat",
            "-f",
            hive.to_str().unwrap(),
            "-t",
            format,
        ]);
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert!(r.stderr.contains("Layers"), "{}", r.stderr);
        if format != "csv" {
            assert_eq!(json_documents(&r.stdout).len(), 1);
        }
    }
}

// --- parse-registry bcd -------------------------------------------------------------------------

/// `parse-registry bcd`'s CSV columns.
const BCD_COLUMNS: [&str; 22] = [
    "Host",
    "SourcePath",
    "HiveType",
    "ObjectGUID",
    "ObjectType",
    "ObjectTypeName",
    "Description",
    "IsDefault",
    "DisplayOrder",
    "Device",
    "Path",
    "RecoveryEnabled",
    "TestSigning",
    "NoIntegrityChecks",
    "SafeBoot",
    "BootStatusPolicy",
    "Inherited",
    "Elements",
    "KeyLastWrittenUTC",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

const BCD_MANAGER: &str = "{9dea862c-5cdd-4e70-acc1-f32b344d4795}";
const BCD_WINDOWS: &str = "{11111111-2222-3333-4444-555555555555}";
const BCD_SETTINGS: &str = "{6efb52bf-1766-41db-a6b3-0ee5eff72bd7}";

/// A BCD store as ransomware and a rootkit leave it, and a SYSTEM hive next to it (exported to
/// one directory): the boot manager's default entry with recovery off, all failures ignored and
/// integrity checks off, test signing on in the boot loader settings it inherits, and a boolean
/// that isn't 0 or 1.
fn bcd_collection(root: &Path) {
    let element = |object: &str, element: &str| format!(r"Objects\{object}\Elements\{element}");
    let kind = |object: &str| format!(r"Objects\{object}\Description");
    let multi = |items: &[&str]| {
        let mut data: Vec<u8> = items.iter().flat_map(|i| utf16z(i)).collect();
        data.extend([0, 0]);
        data
    };
    let mut gpt = vec![0u8; 88];
    gpt[16] = 6;
    gpt[24] = 0x48;
    gpt[32..48].copy_from_slice(&[0x11; 16]);
    gpt[56..72].copy_from_slice(&[0x22; 16]);
    let store = reg_tree(
        "NewStoreRoot",
        &[
            (
                &kind(BCD_MANAGER),
                "Type",
                REG_DWORD,
                0x1010_0002u32.to_le_bytes().to_vec(),
            ),
            (
                &element(BCD_MANAGER, "12000004"),
                "Element",
                REG_SZ,
                utf16z("Windows Boot Manager"),
            ),
            (
                &element(BCD_MANAGER, "23000003"),
                "Element",
                REG_SZ,
                utf16z(BCD_WINDOWS),
            ),
            (
                &element(BCD_MANAGER, "24000001"),
                "Element",
                REG_MULTI_SZ,
                multi(&[BCD_WINDOWS]),
            ),
            (
                &kind(BCD_WINDOWS),
                "Type",
                REG_DWORD,
                0x1020_0003u32.to_le_bytes().to_vec(),
            ),
            (
                &element(BCD_WINDOWS, "11000001"),
                "Element",
                REG_BINARY,
                gpt,
            ),
            (
                &element(BCD_WINDOWS, "12000002"),
                "Element",
                REG_SZ,
                utf16z(r"\Windows\system32\winload.efi"),
            ),
            (
                &element(BCD_WINDOWS, "12000004"),
                "Element",
                REG_SZ,
                utf16z("Windows 11"),
            ),
            (
                &element(BCD_WINDOWS, "14000006"),
                "Element",
                REG_MULTI_SZ,
                multi(&[BCD_SETTINGS]),
            ),
            (
                &element(BCD_WINDOWS, "16000009"),
                "Element",
                REG_BINARY,
                vec![0],
            ),
            (
                &element(BCD_WINDOWS, "16000048"),
                "Element",
                REG_BINARY,
                vec![1],
            ),
            (
                &element(BCD_WINDOWS, "16000040"),
                "Element",
                REG_BINARY,
                vec![2],
            ),
            (
                &element(BCD_WINDOWS, "250000e0"),
                "Element",
                REG_BINARY,
                1u64.to_le_bytes().to_vec(),
            ),
            (
                &kind(BCD_SETTINGS),
                "Type",
                REG_DWORD,
                0x2020_0003u32.to_le_bytes().to_vec(),
            ),
            (
                &element(BCD_SETTINGS, "16000049"),
                "Element",
                REG_BINARY,
                vec![1],
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
        ],
    );
    let dir = root.join("HOST1");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("BCD"), store).unwrap();
    std::fs::write(dir.join("SYSTEM"), system).unwrap();
}

/// `parse-registry bcd` with `args`, as JSON Lines.
fn bcd_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "bcd"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_bcd_of_a_compromised_store() {
    let dir = tempfile::tempdir().unwrap();
    bcd_collection(dir.path());
    let records = bcd_read(&["-d", dir.path().to_str().unwrap()]);
    assert_eq!(records.len(), 3);
    let find = |guid: &str| records.iter().find(|r| r["ObjectGUID"] == guid).unwrap();
    let windows = find(BCD_WINDOWS);
    for (field, expected) in [
        ("Host", json!("HOST1")),
        ("HiveType", json!("BCD")),
        ("ObjectType", json!("0x10200003")),
        ("ObjectTypeName", json!("Windows boot loader")),
        ("Description", json!("Windows 11")),
        ("IsDefault", json!(true)),
        ("DisplayOrder", json!(0)),
        (
            "Device",
            json!(
                "partition {11111111-1111-1111-1111-111111111111} on disk {22222222-2222-2222-2222-222222222222}"
            ),
        ),
        ("Path", json!(r"\Windows\system32\winload.efi")),
        ("RecoveryEnabled", json!(false)),
        ("TestSigning", json!(true)),
        ("NoIntegrityChecks", json!(true)),
        ("SafeBoot", Value::Null),
        ("BootStatusPolicy", json!("IgnoreAllFailures (1)")),
        (
            "Inherited",
            json!([format!("test_signing from {BCD_SETTINGS}")]),
        ),
        ("TransactionLogStatus", json!("clean")),
        (
            "Problems",
            json!([r"Elements\16000040: boolean 2 is neither 0 nor 1"]),
        ),
    ] {
        assert_eq!(windows[field], expected, "{field}");
    }
    let elements = windows["Elements"].as_array().unwrap();
    assert_eq!(elements.len(), 8);
    let element = |t: &str| elements.iter().find(|e| e["ElementType"] == t).unwrap();
    assert_eq!(
        element("16000009")["Name"],
        "BcdLibraryBoolean_AutoRecoveryEnabled"
    );
    assert_eq!(element("16000009")["Value"], false);
    assert_eq!(
        element("250000e0")["Name"],
        "BcdOSLoaderInteger_BootStatusPolicy"
    );
    assert_eq!(element("250000e0")["Value"], 1);
    assert_eq!(element("14000006")["Value"], json!([BCD_SETTINGS]));
    assert_eq!(element("16000040")["Value"], Value::Null);
    assert_eq!(element("16000040")["DataRaw"], "02");
    assert_eq!(
        element("16000009")["KeyLastWritten"],
        "2022-02-08T21:03:52Z"
    );
    assert_eq!(find(BCD_SETTINGS)["TestSigning"], true);
    assert_eq!(find(BCD_SETTINGS)["Inherited"], json!([]));
    assert_eq!(find(BCD_MANAGER)["IsDefault"], false);
}

#[test]
fn registry_bcd_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    bcd_collection(dir.path());
    let out = dir.path().join("bcd.csv");
    let r = run(&[
        "parse-registry",
        "bcd",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let store = dir.path().join("HOST1").join("BCD");
    let store = store.to_str().unwrap();
    for expected in [
        "BCD hives parsed: 1".to_string(),
        "Hosts: HOST1".to_string(),
        "Objects: 3 (Windows boot loader 1, Windows boot manager 1, inherited settings for Windows boot loader 1)".to_string(),
        format!(r"{store}: {BCD_WINDOWS} Windows 11, \Windows\system32\winload.efi, partition"),
        "Test signing, integrity checks off, recovery off, failures ignored, safe mode: 5".to_string(),
        format!("{store}: {BCD_WINDOWS} (Windows 11): test signing on (inherited from {BCD_SETTINGS})"),
        format!("{store}: {BCD_WINDOWS} (Windows 11): integrity checks off (set 2022-02-08T21:03:52Z)"),
        format!("{store}: {BCD_WINDOWS} (Windows 11): recovery off (set 2022-02-08T21:03:52Z)"),
        format!("{store}: {BCD_WINDOWS} (Windows 11): boot status policy IgnoreAllFailures (1) (set 2022-02-08T21:03:52Z)"),
        format!("{store}: {BCD_SETTINGS} (inherited settings for Windows boot loader): test signing on (set 2022-02-08T21:03:52Z)"),
        "Objects or elements that could not be read: 1".to_string(),
    ] {
        assert!(r.stdout.contains(&expected), "{expected}\n{}", r.stdout);
    }
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let header: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, BCD_COLUMNS);
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(rows.len(), 3);
    let windows = rows.iter().find(|r| &r[3] == BCD_WINDOWS).unwrap();
    assert_eq!(&windows[11], "false");
    assert_eq!(&windows[12], "true");
    assert_eq!(&windows[15], "IgnoreAllFailures (1)");
    assert!(
        windows[17].contains("16000009 BcdLibraryBoolean_AutoRecoveryEnabled=false"),
        "{}",
        &windows[17]
    );
    assert!(windows[17].contains("16000040 BcdLibraryBoolean_DisplayAdvancedOptions= [02]"));
}

#[test]
fn registry_bcd_of_public_hives() {
    // regipy's Windows 10 UEFI store: the boot entries its boot_entry_list reads
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["BCD"], dir.path());
    let records = bcd_read(&["-f", dir.path().join("BCD").to_str().unwrap()]);
    assert_eq!(records.len(), 17);
    let default: Vec<&Value> = records.iter().filter(|r| r["IsDefault"] == true).collect();
    assert_eq!(default.len(), 1);
    for (field, expected) in [
        (
            "ObjectGUID",
            json!("{733b62e5-f608-11eb-825c-c112f60133ab}"),
        ),
        ("Description", json!("Windows 10")),
        ("Path", json!(r"\Windows\system32\winload.efi")),
        (
            "Device",
            json!(
                "partition {8e0f2c38-e4ea-47ba-b7fc-9d8c74dccf0b} on disk {0b2394a9-095e-487d-8d48-719ecd4d78ca}"
            ),
        ),
        ("RecoveryEnabled", json!(true)),
        ("TestSigning", Value::Null),
        ("NoIntegrityChecks", Value::Null),
        ("BootStatusPolicy", Value::Null),
    ] {
        assert_eq!(default[0][field], expected, "{field}");
    }
    let manager = records
        .iter()
        .find(|r| r["ObjectTypeName"] == "Windows boot manager")
        .unwrap();
    assert_eq!(manager["Path"], r"\EFI\Microsoft\Boot\bootmgfw.efi");
    assert!(records.iter().all(|r| r["Problems"] == json!([])));

    // the DFIR Artifact Museum's Windows 11 BCD-Template, where Windows keeps it
    let collection = system_hive();
    let config = collection.path().join("HOST/C/Windows/System32/config");
    unpack_registry("rathbun_w11", &["BCD-Template"], &config);
    let records = bcd_read(&["-d", collection.path().to_str().unwrap()]);
    assert_eq!(records.len(), 16);
    assert!(
        records
            .iter()
            .all(|r| r["Host"] == "GOLDPAPERW11" && r["HiveType"] == "BCD-Template")
    );
    let default = records.iter().find(|r| r["IsDefault"] == true).unwrap();
    assert_eq!(
        default["ObjectGUID"],
        "{cbd971bf-b7b8-4885-951a-fa03044f5d71}"
    );
    assert_eq!(default["ObjectTypeName"], "Windows boot loader");
}

#[test]
fn registry_bcd_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&["parse-registry", "bcd", "-f", sam.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("a SAM hive, not a BCD hive"), "{said}");
    // a hive of no known type without Objects
    let other = dir.path().join("store");
    std::fs::write(
        &other,
        reg_tree("ROOT", &[("Description", "KeyName", REG_SZ, utf16z("x"))]),
    )
    .unwrap();
    let r = run(&["parse-registry", "bcd", "-f", other.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(
        said.contains("it has no Objects key: not a BCD store"),
        "{said}"
    );
    let r = run(&[
        "parse-registry",
        "bcd",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(said.contains("No BCD hives were found"), "{said}");
    // the output is never one of the files read
    let original = std::fs::read(&sam).unwrap();
    let r = run(&[
        "parse-registry",
        "bcd",
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

// --- parse-registry lsa-secrets -----------------------------------------------------------------

/// `parse-registry lsa-secrets` with `args`, as JSON Lines.
fn lsa_secrets_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "lsa-secrets"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

/// What every secret value of the built hive holds: never to be seen in the output.
const SECRET_CANARY: &[u8] = b"CANARY-SECRET-VALUE";

/// A cached logon slot: in use (a user name of `user_length` bytes) or empty.
fn cache_slot(user_length: u16) -> Vec<u8> {
    let mut slot = vec![0u8; 168];
    slot[..2].copy_from_slice(&user_length.to_le_bytes());
    if user_length > 0 {
        slot[96..96 + SECRET_CANARY.len()].copy_from_slice(SECRET_CANARY);
    }
    slot
}

/// A SECURITY hive with a service's secret, security questions, an autologon password without
/// an old value, and a cache with one slot of two in use.
fn lsa_secrets_hive(dir: &Path) -> PathBuf {
    let service = r"Policy\Secrets\_SC_ExampleSvc";
    let questions = r"Policy\Secrets\L$_SQSA_S-1-5-21-1-2-3-1001";
    let autologon = r"Policy\Secrets\DefaultPassword";
    let secret: Vec<u8> = SECRET_CANARY.repeat(3);
    let hive = reg_tree(
        "ROOT",
        &[
            (&format!(r"{service}\CurrVal"), "", 0, secret.clone()),
            (&format!(r"{service}\OldVal"), "", 0, secret.clone()),
            (
                &format!(r"{service}\CupdTime"),
                "",
                0,
                LINKED.to_le_bytes().to_vec(),
            ),
            (
                &format!(r"{service}\OupdTime"),
                "",
                0,
                (LINKED - DAY).to_le_bytes().to_vec(),
            ),
            (&format!(r"{questions}\CurrVal"), "", 0, secret.clone()),
            (
                &format!(r"{questions}\CupdTime"),
                "",
                0,
                LINKED.to_le_bytes().to_vec(),
            ),
            (&format!(r"{autologon}\CurrVal"), "", 0, secret),
            (&format!(r"{autologon}\OldVal"), "", 0, Vec::new()),
            (
                &format!(r"{autologon}\CupdTime"),
                "",
                0,
                (LINKED + DAY).to_le_bytes().to_vec(),
            ),
            ("Cache", "NL$1", REG_BINARY, cache_slot(10)),
            ("Cache", "NL$2", REG_BINARY, cache_slot(0)),
            (
                "Cache",
                "NL$Control",
                REG_BINARY,
                vec![4, 0, 1, 0, 2, 0, 0, 0],
            ),
        ],
    );
    let path = dir.join("SECURITY");
    std::fs::write(&path, hive).unwrap();
    path
}

#[test]
fn registry_lsa_secrets_of_a_public_hive() {
    // the DFIR Artifact Museum's dirty Windows 11 SECURITY with its logs, and its SYSTEM
    let dir = system_hive();
    let config = dir.path().join("HOST/C/Windows/System32/config");
    unpack_registry(
        "rathbun_w11",
        &["SECURITY", "SECURITY.LOG1", "SECURITY.LOG2"],
        &config,
    );
    let records = lsa_secrets_read(&["-f", config.join("SECURITY").to_str().unwrap()]);
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} {} {} {} {}",
                r["Secret"].as_str().unwrap(),
                r["Kind"].as_str().unwrap(),
                r["CurrentSet"].as_str().unwrap_or("-"),
                r["HasCurrentValue"],
                r["HasOldValue"],
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            "DefaultPassword autologon 2022-02-05T19:00:06.835826800Z true true",
            "DPAPI_SYSTEM dpapi_system 2022-02-05T21:50:29.394285500Z true true",
            "NL$KM cached_logon_key 2022-02-05T21:49:55.130252900Z true true",
            "Cache cached_logons - null null",
        ]
    );
    for r in &records {
        assert_eq!(r["Host"], "GOLDPAPERW11");
        assert_eq!(r["Problems"], json!([]));
        assert_eq!(r["TransactionLogStatus"], "unchanged");
    }
    assert_eq!(records[0]["OldSet"], "2022-02-05T19:00:06.070794800Z");
    let cache = &records[3];
    assert_eq!(
        (
            &cache["CachedLogons"],
            &cache["CachedLogonsInUse"],
            &cache["CachedLogonsKept"]
        ),
        (&json!(10), &json!(0), &json!(10))
    );
}

#[test]
fn registry_lsa_secrets_never_write_secret_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let hive = lsa_secrets_hive(dir.path());
    let records = lsa_secrets_read(&["-f", hive.to_str().unwrap()]);
    let got: Vec<(&str, &str, &Value, &Value)> = records
        .iter()
        .map(|r| {
            (
                r["Secret"].as_str().unwrap(),
                r["Kind"].as_str().unwrap(),
                &r["ServiceName"],
                &r["UserSID"],
            )
        })
        .collect();
    assert_eq!(
        got,
        // in the order of the keys' names, as Windows sorts them
        [
            ("DefaultPassword", "autologon", &Value::Null, &Value::Null),
            (
                "L$_SQSA_S-1-5-21-1-2-3-1001",
                "security_questions",
                &Value::Null,
                &json!("S-1-5-21-1-2-3-1001")
            ),
            (
                "_SC_ExampleSvc",
                "service_account",
                &json!("ExampleSvc"),
                &Value::Null
            ),
            ("Cache", "cached_logons", &Value::Null, &Value::Null),
        ]
    );
    assert_eq!(records[2]["CurrentSet"], "2022-02-08T21:03:52Z");
    assert_eq!(records[2]["OldSet"], "2022-02-07T21:03:52Z");
    assert_eq!(
        records[2]["CurrentValueLastWritten"],
        "2022-02-08T21:03:52Z"
    );
    // security questions without an old value; autologon with an empty one
    assert_eq!(
        (
            &records[1]["HasOldValue"],
            &records[1]["OldValueLastWritten"]
        ),
        (&json!(false), &Value::Null)
    );
    assert_eq!(
        (&records[0]["HasCurrentValue"], &records[0]["HasOldValue"]),
        (&json!(true), &json!(false))
    );
    assert_eq!(
        (
            &records[3]["CachedLogons"],
            &records[3]["CachedLogonsInUse"],
            &records[3]["CachedLogonsKept"]
        ),
        (&json!(2), &json!(1), &json!(2))
    );
    // no output holds a secret's bytes, as text or in hex
    let out = dir.path().join("secrets.csv");
    let r = run(&[
        "parse-registry",
        "lsa-secrets",
        "-f",
        hive.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let jsonl = run(&[
        "parse-registry",
        "lsa-secrets",
        "-f",
        hive.to_str().unwrap(),
        "-t",
        "json",
    ]);
    let canary_hex: String = SECRET_CANARY.iter().map(|b| format!("{b:02x}")).collect();
    let csv_text = std::fs::read_to_string(&out).unwrap();
    for text in [&r.stdout, &r.stderr, &csv_text, &jsonl.stdout] {
        assert!(!text.contains("CANARY"), "{text}");
        assert!(!text.to_lowercase().contains(&canary_hex[..16]), "{text}");
    }
    for expected in [
        "Secrets (names and times only; nothing is decrypted): 3 (autologon 1, security_questions 1, service_account 1)",
        "_SC_ExampleSvc  (service_account for ExampleSvc; set 2022-02-08T21:03:52Z)",
        "Services whose account passwords LSA stores: ExampleSvc",
        "Cached domain logons: 1 of 2 slots in use, 2 kept (NL$Control)",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    let (header, rows) = csv_rows(&out);
    assert_eq!(header.len(), 19);
    assert_eq!(header[2..4], ["Secret", "Kind"]);
    assert_eq!(rows[2]["ServiceName"], "ExampleSvc");
    assert_eq!(rows[2]["HasCurrentValue"], "True");
    assert_eq!(rows[3]["CachedLogonsInUse"], "1");
}

#[test]
fn registry_lsa_secrets_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let r = run(&[
        "parse-registry",
        "lsa-secrets",
        "-f",
        dir.path().join("SAM").to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stderr.contains("not a SECURITY hive"), "{}", r.stderr);
    let r = run(&[
        "parse-registry",
        "lsa-secrets",
        "-d",
        dir.path().to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stderr.contains("No SECURITY hives were found"),
        "{}",
        r.stderr
    );
    // the output is never the hive read, even with -C
    let hive = lsa_secrets_hive(dir.path());
    let original = std::fs::read(&hive).unwrap();
    let r = run(&[
        "parse-registry",
        "lsa-secrets",
        "-f",
        hive.to_str().unwrap(),
        "-o",
        hive.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(&hive).unwrap(), original);
}

#[test]
fn registry_lsa_secrets_keeps_list_warnings_on_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("SECURITY");
    let mut bytes = reg_tree(
        "ROOT",
        &[(
            r"Policy\Secrets\Example\CupdTime",
            "",
            REG_BINARY,
            132_726_344_757_163_260u64.to_le_bytes().to_vec(),
        )],
    );
    let name = bytes.windows(7).position(|b| b == b"Secrets").unwrap();
    put_u32(&mut bytes, name - 76 + 20, 2);
    std::fs::write(&hive, bytes).unwrap();
    for format in ["csv", "json", "jsonl"] {
        let r = run(&[
            "parse-registry",
            "lsa-secrets",
            "-f",
            hive.to_str().unwrap(),
            "-t",
            format,
        ]);
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert!(r.stderr.contains("Policy\\Secrets"), "{}", r.stderr);
        if format != "csv" {
            assert_eq!(json_documents(&r.stdout).len(), 1);
        }
    }
}

// --- parse-registry networks --------------------------------------------------------------------

/// `parse-registry networks`' CSV columns.
const NETWORKS_COLUMNS: [&str; 35] = [
    "Host",
    "SourcePath",
    "ControlSet",
    "IsCurrentControlSet",
    "Source",
    "ProfileName",
    "Category",
    "ConnectionType",
    "FirstConnectedUTC",
    "LastConnectedUTC",
    "FirstConnectedLocal",
    "LastConnectedLocal",
    "TimeZone",
    "GatewayMAC",
    "DNS-Suffix",
    "InterfaceGUID",
    "IP-Address",
    "DhcpServer",
    "LeaseObtainedUTC",
    "LeaseExpiresUTC",
    "ShareName",
    "SharePath",
    "ListenAddress",
    "ListenPort",
    "ConnectAddress",
    "ConnectPort",
    "Route",
    "Details",
    "KeyPath",
    "ValueName",
    "KeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

/// A `SYSTEMTIME`: year, month, weekday, day, hour, minute, second, millisecond.
fn systemtime(fields: [u16; 8]) -> Vec<u8> {
    fields.iter().flat_map(|n| n.to_le_bytes()).collect()
}

/// One system (HOST1, Eastern time) with a domain network and a café's Wi-Fi in SOFTWARE, its
/// network card, and in SYSTEM a DHCP interface, a staging share, a port proxy and a persistent
/// route.
fn networks_collection(root: &Path) {
    let dword = |n: u32| n.to_le_bytes().to_vec();
    let list = r"Microsoft\Windows NT\CurrentVersion\NetworkList";
    let corp = format!(r"{list}\Profiles\{{8148D564-B67F-423F-9C2C-9CA1D61F18AE}}");
    let cafe = format!(r"{list}\Profiles\{{D4BA51F6-8576-40A9-953A-42027C41ED4F}}");
    let signature = format!(r"{list}\Signatures\Managed\010103000F0000F0A000");
    let software = reg_tree(
        "ROOT",
        &[
            (
                r"Microsoft\Windows NT\CurrentVersion\NetworkCards\2",
                "ServiceName",
                REG_SZ,
                utf16z("{DAE79B04-8A4E-46D0-A8DB-EE10ABBFFE64}"),
            ),
            (
                r"Microsoft\Windows NT\CurrentVersion\NetworkCards\2",
                "Description",
                REG_SZ,
                utf16z("Example Network Adapter"),
            ),
            (
                &signature,
                "ProfileGuid",
                REG_SZ,
                utf16z("{8148D564-B67F-423F-9C2C-9CA1D61F18AE}"),
            ),
            (&signature, "DnsSuffix", REG_SZ, utf16z("corp.example")),
            (
                &signature,
                "DefaultGatewayMac",
                REG_BINARY,
                vec![0, 0x0c, 0x29, 0x67, 0x10, 0x19],
            ),
            (&corp, "ProfileName", REG_SZ, utf16z("corp.example")),
            (&corp, "Category", REG_DWORD, dword(2)),
            (&corp, "NameType", REG_DWORD, dword(6)),
            // winter (EST) and summer (EDT)
            (
                &corp,
                "DateCreated",
                REG_BINARY,
                systemtime([2021, 2, 3, 10, 14, 50, 37, 716]),
            ),
            (
                &corp,
                "DateLastConnected",
                REG_BINARY,
                systemtime([2021, 7, 0, 4, 12, 0, 0, 0]),
            ),
            (&cafe, "ProfileName", REG_SZ, utf16z("Cafe WiFi")),
            (&cafe, "Category", REG_DWORD, dword(0)),
            (&cafe, "NameType", REG_DWORD, dword(71)),
            // damaged: 15 bytes
            (&cafe, "DateCreated", REG_BINARY, vec![1; 15]),
        ],
    );
    let interface = r"ControlSet001\Services\Tcpip\Parameters\Interfaces\{dae79b04-8a4e-46d0-a8db-ee10abbffe64}";
    let tz = r"ControlSet001\Control\TimeZoneInformation";
    // TIME_FIELDS: year, month, day (the week), hour, minute, second, millisecond, weekday
    let rule = |month: u16, week: u16| systemtime([0, month, week, 2, 0, 0, 0, 0]);
    let system = reg_tree(
        "ROOT",
        &[
            ("Select", "Current", REG_DWORD, dword(1)),
            (
                r"ControlSet001\Control\ComputerName\ComputerName",
                "ComputerName",
                REG_SZ,
                utf16z("HOST1"),
            ),
            (
                tz,
                "TimeZoneKeyName",
                REG_SZ,
                utf16z("Eastern Standard Time"),
            ),
            (tz, "Bias", REG_DWORD, dword(300)),
            (tz, "DaylightBias", REG_DWORD, dword(0xffff_ffc4)),
            (tz, "StandardStart", REG_BINARY, rule(11, 1)),
            (tz, "DaylightStart", REG_BINARY, rule(3, 2)),
            (interface, "EnableDHCP", REG_DWORD, dword(1)),
            (interface, "DhcpIPAddress", REG_SZ, utf16z("192.0.2.25")),
            (interface, "DhcpServer", REG_SZ, utf16z("192.0.2.1")),
            (interface, "DhcpSubnetMask", REG_SZ, utf16z("255.255.255.0")),
            (
                interface,
                "LeaseObtainedTime",
                REG_DWORD,
                dword(1_625_414_400),
            ),
            (
                interface,
                "LeaseTerminatesTime",
                REG_DWORD,
                dword(1_625_500_800),
            ),
            (
                r"ControlSet001\Services\LanmanServer\Shares",
                "Stage$",
                REG_MULTI_SZ,
                [
                    "CSCFlags=0",
                    "MaxUses=4294967295",
                    r"Path=C:\ProgramData\stage",
                    "Permissions=0",
                    "Remark=",
                    "ShareName=Stage$",
                    "Type=0",
                ]
                .iter()
                .flat_map(|s| utf16z(s))
                .chain([0, 0])
                .collect(),
            ),
            (
                r"ControlSet001\Services\PortProxy\v4tov4\tcp",
                "0.0.0.0/8443",
                REG_SZ,
                utf16z("10.0.0.5/3389"),
            ),
            (
                r"ControlSet001\Services\Tcpip\Parameters\PersistentRoutes",
                "10.10.0.0,255.255.0.0,192.0.2.254,1",
                REG_SZ,
                utf16z(""),
            ),
        ],
    );
    let config = root.join("C/Windows/System32/config");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join("SOFTWARE"), software).unwrap();
    std::fs::write(config.join("SYSTEM"), system).unwrap();
}

/// `parse-registry networks` with `args`, as JSON Lines.
fn networks_read(args: &[&str]) -> Vec<Value> {
    let mut all = vec!["parse-registry", "networks"];
    all.extend(args);
    all.extend(["-t", "jsonl"]);
    let r = run(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl_records(&r.stdout)
}

#[test]
fn registry_networks_of_a_collection() {
    let dir = tempfile::tempdir().unwrap();
    networks_collection(dir.path());
    let records = networks_read(&["-d", dir.path().to_str().unwrap()]);
    let sources: Vec<&str> = records
        .iter()
        .map(|r| r["Source"].as_str().unwrap())
        .collect();
    assert_eq!(
        sources,
        [
            "network_profile",
            "network_profile",
            "network_card",
            "interface",
            "share",
            "port_proxy",
            "route"
        ]
    );
    for r in &records {
        assert_eq!(r["Host"], "HOST1");
        assert_eq!(r["KeyLastWritten"], "2022-02-08T21:03:52Z");
    }
    // the domain network: its local times, in UTC with the system's Eastern time
    let corp = &records[0];
    assert_eq!(
        (
            &corp["ProfileName"],
            &corp["Category"],
            &corp["ConnectionType"],
            &corp["GatewayMAC"],
            &corp["DNS-Suffix"]
        ),
        (
            &json!("corp.example"),
            &json!("domain"),
            &json!("wired"),
            &json!("00:0c:29:67:10:19"),
            &json!("corp.example")
        )
    );
    assert_eq!(corp["FirstConnectedLocal"], "2021-02-10T14:50:37.716");
    assert_eq!(corp["FirstConnected"], "2021-02-10T19:50:37.716Z");
    assert_eq!(corp["LastConnectedLocal"], "2021-07-04T12:00:00.000");
    assert_eq!(corp["LastConnected"], "2021-07-04T16:00:00Z");
    assert_eq!(corp["TimeZone"], "Eastern Standard Time");
    assert_eq!(
        corp["Details"]["Signatures"]
            .as_str()
            .unwrap()
            .matches('|')
            .count(),
        0
    );
    // the café: wireless, a damaged time kept as bytes
    let cafe = &records[1];
    assert_eq!(
        (&cafe["Category"], &cafe["ConnectionType"]),
        (&json!("public"), &json!("wireless"))
    );
    assert_eq!(cafe["FirstConnected"], Value::Null);
    assert_eq!(cafe["DataRaw"], "01".repeat(15));
    assert_eq!(
        cafe["Problems"],
        json!(["DateCreated: 15 bytes, not the 16 of a SYSTEMTIME"])
    );
    // the card, and its interface with the adapter's name
    assert_eq!(
        records[2]["InterfaceGUID"],
        "{DAE79B04-8A4E-46D0-A8DB-EE10ABBFFE64}"
    );
    let interface = &records[3];
    assert_eq!(
        (
            &interface["IP-Address"],
            &interface["DhcpServer"],
            &interface["LeaseObtained"],
            &interface["LeaseExpires"],
            &interface["ControlSet"]
        ),
        (
            &json!("192.0.2.25"),
            &json!("192.0.2.1"),
            &json!("2021-07-04T16:00:00Z"),
            &json!("2021-07-05T16:00:00Z"),
            &json!("ControlSet001")
        )
    );
    assert_eq!(interface["Details"]["Adapter"], "Example Network Adapter");
    let share = &records[4];
    assert_eq!(
        (&share["ShareName"], &share["SharePath"]),
        (&json!("Stage$"), &json!(r"C:\ProgramData\stage"))
    );
    assert_eq!(share["Details"]["Type"], "0 (disk)");
    let proxy = &records[5];
    assert_eq!(
        (
            &proxy["ListenAddress"],
            &proxy["ListenPort"],
            &proxy["ConnectAddress"],
            &proxy["ConnectPort"]
        ),
        (
            &json!("0.0.0.0"),
            &json!(8443),
            &json!("10.0.0.5"),
            &json!(3389)
        )
    );
    assert_eq!(records[6]["Route"], "10.10.0.0,255.255.0.0,192.0.2.254,1");
    assert_eq!(records[6]["Details"]["Gateway"], "192.0.2.254");
}

#[test]
fn registry_networks_csv_and_summary() {
    let dir = tempfile::tempdir().unwrap();
    networks_collection(dir.path());
    let out = dir.path().join("networks.csv");
    let r = run(&[
        "parse-registry",
        "networks",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for expected in [
        "Hives parsed: 2 (SOFTWARE 1, SYSTEM 1)",
        "Hosts: HOST1",
        "Records: 7 (interface 1, network_card 1, network_profile 2, port_proxy 1, route 1, share 1)",
        "HOST1  2021-07-04T16:00:00Z  corp.example  wired  gateway 00:0c:29:67:10:19",
        r"HOST1  Stage$  C:\ProgramData\stage",
        "HOST1  v4tov4  0.0.0.0:8443 -> 10.0.0.5:3389",
        "HOST1  10.10.0.0,255.255.0.0,192.0.2.254,1",
        "Keys or values that could not be read: 1",
    ] {
        assert!(r.stdout.contains(expected), "{expected}\n{}", r.stdout);
    }
    let mut reader = csv::Reader::from_path(&out).unwrap();
    let header: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_string)
        .collect();
    assert_eq!(header, NETWORKS_COLUMNS);
    let rows: Vec<csv::StringRecord> = reader.records().map(Result::unwrap).collect();
    assert_eq!(rows.len(), 7);
    assert_eq!(&rows[0][4], "network_profile");
    assert_eq!(&rows[0][9], "2021-07-04T16:00:00Z");
    assert_eq!(&rows[0][11], "2021-07-04T12:00:00.000");
    assert_eq!(&rows[3][16], "192.0.2.25");
    assert_eq!(&rows[5][23], "8443");
}

#[test]
fn registry_networks_of_public_hives() {
    // regipy's Windows 7 SYSTEM: two static interfaces, one leased before, and two routes; what
    // regipy's network_data and routes give
    let regipy = regipy_system();
    let records = networks_read(&["-d", regipy.path().to_str().unwrap()]);
    let lines: Vec<String> = records
        .iter()
        .map(|r| {
            format!(
                "{} {} {}",
                r["Source"].as_str().unwrap(),
                r["IP-Address"].as_str().unwrap_or("-"),
                r["Route"].as_str().unwrap_or("-")
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            "interface 10.3.58.5 -",
            "interface 192.168.1.123 -",
            "interface - -",
            "route - 0.0.0.0,0.0.0.0,192.168.1.1,-1",
            "route - 0.0.0.0,0.0.0.0,10.3.58.1,-1",
        ]
    );
    assert_eq!(records[1]["LeaseObtained"], "2010-11-10T17:06:43Z");
    assert_eq!(records[0]["LeaseObtained"], Value::Null);
    // the DFIR Artifact Museum's dirty Windows 11 SYSTEM with its logs: its DHCP interface
    let w11 = system_hive();
    let records = networks_read(&["-d", w11.path().to_str().unwrap()]);
    assert!(records.iter().all(|r| r["Host"] == "GOLDPAPERW11"
        && r["Source"] == "interface"
        && r["Problems"] == json!([])));
    assert_eq!(
        records
            .iter()
            .filter(|r| r["DhcpServer"].is_string())
            .count(),
        1
    );
}

#[test]
fn registry_networks_errors() {
    let dir = tempfile::tempdir().unwrap();
    unpack_registry("regipy", &["SAM"], dir.path());
    let sam = dir.path().join("SAM");
    let r = run(&[
        "parse-registry",
        "networks",
        "-f",
        sam.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(
        said.contains("a SAM hive, not a SOFTWARE or SYSTEM hive"),
        "{said}"
    );
    let r = run(&[
        "parse-registry",
        "networks",
        "-d",
        dir.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    let said = format!("{}{}", r.stdout, r.stderr);
    assert!(
        said.contains("No SOFTWARE or SYSTEM hives were found"),
        "{said}"
    );
    // the output is never one of the files read: the SOFTWARE's SYSTEM
    let collection = tempfile::tempdir().unwrap();
    networks_collection(collection.path());
    let config = collection.path().join("C/Windows/System32/config");
    let original = std::fs::read(config.join("SYSTEM")).unwrap();
    let r = run(&[
        "parse-registry",
        "networks",
        "-f",
        config.join("SOFTWARE").to_str().unwrap(),
        "-o",
        config.join("SYSTEM").to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(config.join("SYSTEM")).unwrap(), original);
}

#[test]
fn registry_networks_missing_select_remains_unknown_and_warns_on_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let hive = dir.path().join("SYSTEM");
    std::fs::write(
        &hive,
        reg_tree(
            "ROOT",
            &[(
                r"ControlSet001\Services\Tcpip\Parameters\Interfaces\{interface}",
                "NameServer",
                REG_SZ,
                utf16z("192.0.2.1"),
            )],
        ),
    )
    .unwrap();
    for format in ["csv", "json", "jsonl"] {
        let r = run(&[
            "parse-registry",
            "networks",
            "-f",
            hive.to_str().unwrap(),
            "-t",
            format,
        ]);
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert!(r.stderr.contains("Select"), "{}", r.stderr);
        if format != "csv" {
            let records = json_documents(&r.stdout);
            assert_eq!(records.len(), 1);
            assert!(records[0]["IsCurrentControlSet"].is_null());
        }
    }
}

#[test]
fn srum_category_reruns_remove_recorded_outputs_and_keep_unrelated_files() {
    let root = tempfile::tempdir().unwrap();
    let evidence = root.path().join("evidence");
    let sru = evidence.join("HOST1/C/Windows/System32/sru");
    std::fs::create_dir_all(&sru).unwrap();
    let database = sru.join("SRUDB.dat");
    std::fs::copy(repo(SRUM_WIN10), &database).unwrap();
    let output = root.path().join("parsed");
    let scan = |format: &str, clobber: bool| {
        let mut args = vec![
            "auto-parse",
            "-d",
            evidence.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-t",
            format,
            "-T",
            "2",
            "-q",
            "-K",
        ];
        if clobber {
            args.push("-C");
        }
        let result = run(&args);
        assert_eq!(result.code, 0, "{}\n{}", result.stdout, result.stderr);
    };
    scan("csv", false);
    let network = output.join("HOST1/NetworkActivity");
    let previous = network.join("SRUM-NetworkUsages.csv");
    assert!(previous.is_file());
    let notes = network.join("SRUM-personal-notes.csv");
    std::fs::write(&notes, b"keep").unwrap();
    scan("jsonl", true);
    assert!(!previous.exists());
    assert!(network.join("SRUM-NetworkUsages.jsonl").is_file());
    assert_eq!(std::fs::read(&notes).unwrap(), b"keep");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(output.join("manifest.json")).unwrap()).unwrap();
    let task = manifest["Tasks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["Artifact"] == "srum")
        .unwrap();
    let files = task["OutputFiles"].as_array().unwrap();
    assert!(!files.is_empty());
    for file in files {
        assert!(output.join(file.as_str().unwrap()).is_file());
    }
    std::fs::rename(&database, sru.join("not-collected.bin")).unwrap();
    scan("jsonl", true);
    assert!(!network.join("SRUM-NetworkUsages.jsonl").exists());
    assert_eq!(std::fs::read(notes).unwrap(), b"keep");
}

#[test]
fn srum_exports_cannot_overwrite_inputs_even_with_new_table_names() {
    let root = tempfile::tempdir().unwrap();
    let database = root.path().join("SRUM-NetworkUsages.csv");
    let original = std::fs::read(repo(SRUM_WIN10)).unwrap();
    std::fs::write(&database, &original).unwrap();
    let result = run(&[
        "parse-ese",
        "srum",
        "-f",
        database.to_str().unwrap(),
        "-o",
        root.path().to_str().unwrap(),
        "-C",
        "-q",
        "-K",
    ]);
    assert_ne!(result.code, 0);
    assert_eq!(std::fs::read(&database).unwrap(), original);
    let alias = root.path().join("output.jsonl");
    std::fs::hard_link(&database, &alias).unwrap();
    let result = run(&[
        "parse-ese",
        "srum",
        "-f",
        database.to_str().unwrap(),
        "-t",
        "jsonl",
        "-o",
        alias.to_str().unwrap(),
        "-C",
        "-q",
        "-K",
    ]);
    assert_ne!(result.code, 0);
    assert_eq!(std::fs::read(database).unwrap(), original);
}

#[test]
fn auto_parse_timezone_is_utc_by_default_and_offset_is_shared_by_csv_json_and_timeline() {
    let evidence = auto_triage();
    let target = tempfile::tempdir().unwrap();
    let parse = |output: &Path, kind: &str, offset: Option<&str>, tz: &str| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_juntei"));
        cmd.args(["auto-parse", "-d"])
            .arg(evidence.path())
            .arg("-o")
            .arg(output)
            .args(["-t", kind, "-j"])
            .arg(output.join("all.jsonl"))
            .args(["-T", "2", "-K", "-q"])
            .env("TZ", tz);
        if let Some(offset) = offset {
            cmd.args(["--timezone", offset]);
        }
        let result = cmd.output().unwrap();
        assert!(
            result.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        let stdout = String::from_utf8(result.stdout).unwrap();
        // The console start time is the computer's local time whatever --timezone says.
        let start = stdout
            .lines()
            .find_map(|line| line.strip_prefix("Start time: "))
            .unwrap();
        let start = chrono::NaiveDateTime::parse_from_str(start, "%Y-%m-%d %H:%M:%S").unwrap();
        // Windows takes its time zone from the system, not from TZ.
        if cfg!(unix) {
            let hours = ((start - chrono::Utc::now().naive_utc()).num_seconds() as f64 / 3600.0)
                .round() as i64;
            match tz {
                "Asia/Tokyo" => assert_eq!(hours, 9, "{stdout}"),
                _ => assert!(hours == -7 || hours == -8, "{stdout}"),
            }
        }
        let manifest: Value =
            serde_json::from_reader(std::fs::File::open(output.join("manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["OutputTimeZone"], offset.unwrap_or("+00:00"));
        assert_eq!(manifest["ManifestVersion"], 2);
        assert_eq!(
            std::fs::canonicalize(output.join(manifest["JSON-Timeline"].as_str().unwrap()))
                .unwrap(),
            std::fs::canonicalize(output.join("all.jsonl")).unwrap()
        );
        for task in manifest["Tasks"].as_array().unwrap() {
            assert!(output.join(task["Log"].as_str().unwrap()).is_file());
            if task["Status"] == "succeeded" {
                assert!(output.join(task["Output"].as_str().unwrap()).exists());
            }
        }
        assert_no_absolute_paths(output, &stdout, &[evidence.path(), target.path()]);
        assert!(manifest["GeneratedUTC"].as_str().unwrap().ends_with('Z'));
        assert!(
            manifest["Tasks"]
                .as_array()
                .unwrap()
                .iter()
                .all(|task| task["Status"] != "failed")
        );
        let (_, rows) = csv_rows(&output.join("manifest.csv"));
        assert!(
            rows.iter()
                .all(|r| r["OutputTimeZone"] == offset.unwrap_or("+00:00"))
        );
    };
    let utc = target.path().join("utc");
    parse(&utc, "csv", None, "Asia/Tokyo");
    let (_, before) = csv_rows(&utc.join("HOST1/ProcessExecution/Prefetch.csv"));
    let original = &before[0]["LastRun"];
    assert!(original.ends_with('Z'));
    let time = chrono::DateTime::parse_from_rfc3339(original).unwrap();
    for kind in ["csv", "json", "jsonl"] {
        let output = target.path().join(kind);
        parse(&output, kind, Some("+09:00"), "America/Los_Angeles");
        let data: Vec<Value> = std::fs::read_to_string(output.join("all.jsonl"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let prefetch = data.iter().find(|v| v["Artifact"] == "prefetch").unwrap();
        let shown = prefetch["LastRunTimes"][0].as_str().unwrap();
        let shown_time = chrono::DateTime::parse_from_rfc3339(shown).unwrap();
        assert_eq!(shown_time, time);
        assert!(shown.ends_with("+09:00"));
        assert_eq!(shown_time.offset().local_minus_utc(), 32400);
        assert_eq!(prefetch["Comp"], "HOST1");
        if kind == "csv" {
            let (_, rows) = csv_rows(&output.join("HOST1/ProcessExecution/Prefetch.csv"));
            assert_eq!(rows[0]["LastRun"], shown);
            for key in [
                "PreviousRun1",
                "PreviousRun2",
                "PreviousRun3",
                "PreviousRun4",
                "PreviousRun5",
                "PreviousRun6",
                "PreviousRun7",
            ] {
                if !rows[0][key].is_empty() {
                    assert!(rows[0][key].ends_with("+09:00"));
                }
            }
        } else {
            let text = std::fs::read_to_string(
                output.join(format!("HOST1/ProcessExecution/Prefetch.{kind}")),
            )
            .unwrap();
            let individual: Vec<Value> = serde_json::Deserializer::from_str(&text)
                .into_iter::<Value>()
                .map(Result::unwrap)
                .collect();
            assert_eq!(individual[0]["LastRunTimes"], prefetch["LastRunTimes"]);
        }
    }
}

#[test]
fn auto_parse_timezone_converts_lsa_and_sdb_timestamps_in_all_exports() {
    let evidence = tempfile::tempdir().unwrap();
    let host = evidence.path().join("HOST");
    appcompat_collection(&host);
    lsa_secrets_hive(&host.join("C/Windows/System32/config"));
    unpack_registry(
        "rathbun_w11",
        &["SECURITY", "SECURITY.LOG1", "SECURITY.LOG2"],
        &evidence.path().join("PUBLIC/C/Windows/System32/config"),
    );
    let target = tempfile::tempdir().unwrap();
    for kind in ["csv", "json", "jsonl"] {
        let output = target.path().join(kind);
        let result = run(&[
            "auto-parse",
            "-d",
            evidence.path().to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-t",
            kind,
            "-j",
            output.join("all.jsonl").to_str().unwrap(),
            "-Z",
            "+09:00",
            "-T",
            "2",
            "-K",
            "-q",
        ]);
        assert_eq!(result.code, 0, "{}\n{}", result.stdout, result.stderr);
        let timeline = jsonl_records(&std::fs::read_to_string(output.join("all.jsonl")).unwrap());
        let secret = timeline
            .iter()
            .find(|r| r["Artifact"] == "lsa-secrets" && r["Secret"] == "_SC_ExampleSvc")
            .unwrap();
        assert_eq!(secret["CurrentSet"], "2022-02-09T06:03:52+09:00");
        assert_eq!(secret["OldSet"], "2022-02-08T06:03:52+09:00");
        let public_secret = timeline
            .iter()
            .find(|r| {
                r["Artifact"] == "lsa-secrets"
                    && r["Secret"] == "DefaultPassword"
                    && r["Comp"] == "PUBLIC"
            })
            .unwrap();
        assert_eq!(
            public_secret["CurrentSet"],
            "2022-02-06T04:00:06.835826800+09:00"
        );
        assert_eq!(
            public_secret["OldSet"],
            "2022-02-06T04:00:06.070794800+09:00"
        );
        let sdb = timeline
            .iter()
            .find(|r| r["Artifact"] == "appcompat" && r["Source"] == "installed_sdb")
            .unwrap();
        assert_eq!(sdb["SdbInstalled"], "2022-02-08T06:03:52+09:00");
        let manifest: Value =
            serde_json::from_slice(&std::fs::read(output.join("manifest.json")).unwrap()).unwrap();
        for (artifact, selector, value, fields) in [
            (
                "lsa-secrets",
                "Secret",
                "_SC_ExampleSvc",
                vec![
                    ("CurrentSet", "2022-02-09T06:03:52+09:00"),
                    ("OldSet", "2022-02-08T06:03:52+09:00"),
                ],
            ),
            (
                "appcompat",
                "Source",
                "installed_sdb",
                vec![("SdbInstalled", "2022-02-08T06:03:52+09:00")],
            ),
        ] {
            let task = manifest["Tasks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["Artifact"] == artifact && t["Purpose"] == "artifact")
                .unwrap();
            assert_eq!(task["Status"], "succeeded");
            let path = output.join(task["Output"].as_str().unwrap());
            if kind == "csv" {
                let (_, rows) = csv_rows(&path);
                let row = rows.iter().find(|r| r[selector] == value).unwrap();
                for (field, expected) in fields {
                    assert_eq!(row[&format!("{field}UTC")], expected);
                }
            } else {
                let records = json_documents(&std::fs::read_to_string(path).unwrap());
                let row = records.iter().find(|r| r[selector] == value).unwrap();
                for (field, expected) in fields {
                    assert_eq!(row[field], expected);
                }
            }
        }
    }
}

#[test]
fn auto_parse_timezone_retains_boundary_registry_time_and_converts_remaining_records() {
    let evidence = tempfile::tempdir().unwrap();
    let config = evidence.path().join("HOST/C/Windows/System32/config");
    std::fs::create_dir_all(&config).unwrap();
    let instant = chrono::DateTime::parse_from_rfc3339("9999-12-31T23:00:00Z").unwrap();
    let filetime = (instant.timestamp() + 11_644_473_600) as u64 * 10_000_000;
    let hive = regf(&reg_key(
        "SOFTWARE",
        vec![],
        vec![
            reg_key(
                "Boundary",
                vec![("Example", REG_SZ, utf16z("unchanged data"))],
                vec![],
            )
            .written(filetime),
            reg_key(
                "Ordinary",
                vec![("Example", REG_SZ, utf16z("unchanged data"))],
                vec![],
            )
            .written(LINKED),
        ],
    ));
    let input = config.join("SOFTWARE");
    std::fs::write(&input, &hive).unwrap();
    let target = tempfile::tempdir().unwrap();
    let mut expected_records = None;
    for (kind, offset, relative) in [
        ("csv", "+00:00", false),
        ("csv", "+09:00", false),
        ("json", "+09:00", false),
        ("jsonl", "+09:00", false),
        ("csv", "+09:00", true),
        ("json", "+09:00", true),
        ("jsonl", "+09:00", true),
    ] {
        let name = format!("{kind}-{offset}-{relative}").replace(':', "");
        let output = target.path().join(&name);
        let given = if relative {
            Path::new(".").join(name)
        } else {
            output.clone()
        };
        let result = run_at(
            target.path(),
            &[
                "auto-parse",
                "-d",
                evidence.path().to_str().unwrap(),
                "-o",
                given.to_str().unwrap(),
                "-t",
                kind,
                "-j",
                output.join("all.jsonl").to_str().unwrap(),
                "-Z",
                offset,
                "-T",
                "2",
                "-K",
                "-q",
            ],
        );
        assert_eq!(result.code, 0, "{}\n{}", result.stdout, result.stderr);
        let records = jsonl_records(&std::fs::read_to_string(output.join("all.jsonl")).unwrap());
        let records: Vec<_> = records
            .iter()
            .filter(|r| r["Artifact"] == "registry")
            .collect();
        assert_eq!(
            records.len(),
            *expected_records.get_or_insert(records.len())
        );
        let text = serde_json::to_string(&records).unwrap();
        assert!(
            text.contains("9999-12-31T23:00:00Z"),
            "{kind}: boundary value lost"
        );
        let manifest: Value =
            serde_json::from_slice(&std::fs::read(output.join("manifest.json")).unwrap()).unwrap();
        assert!(
            manifest["Tasks"]
                .as_array()
                .unwrap()
                .iter()
                .all(|t| t["Status"] != "failed")
        );
        let task = manifest["Tasks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["Artifact"] == "registry" && t["Purpose"] == "artifact")
            .unwrap();
        assert_eq!(task["Status"], "succeeded");
        if offset != "+00:00" {
            assert!(text.contains("+09:00"));
            let log = std::fs::read_to_string(output.join(task["Log"].as_str().unwrap())).unwrap();
            assert!(log.contains("Warning: retained"), "{log}");
            assert!(
                log.contains(&format!(
                    "unchanged in {} because",
                    task["Output"].as_str().unwrap()
                )),
                "{log}"
            );
            assert!(log.contains("9999-12-31T23:00:00Z"), "{log}");
            if kind == "csv" {
                let (_, rows) = csv_rows(&output.join(task["Output"].as_str().unwrap()));
                assert!(
                    rows.iter()
                        .any(|r| r["KeyLastWrittenUTC"] == "9999-12-31T23:00:00Z")
                );
                assert!(
                    rows.iter()
                        .any(|r| r["KeyLastWrittenUTC"].ends_with("+09:00"))
                );
            }
        }
        assert_eq!(std::fs::read(&input).unwrap(), hive);
    }
}

#[test]
fn auto_parse_timezone_errors_give_quoted_examples_and_negative_offsets_are_accepted() {
    for offset in ["JST", "UTC", "+9", "+09", "09:00", "+24:00", "+00:60"] {
        let result = run(&["auto-parse", "-d", "missing", "-o", "unused", "-Z", offset]);
        assert_ne!(result.code, 0);
        assert!(
            result.stderr.contains("--timezone \"+09:00\""),
            "{}",
            result.stderr
        );
        assert!(result.stderr.contains("-Z \"-01:00\""), "{}", result.stderr);
    }
    let result = run(&[
        "auto-parse",
        "-d",
        "missing",
        "-o",
        "unused",
        "-Z",
        "-01:00",
    ]);
    assert_eq!(result.code, 1);
    assert!(
        result.stdout.contains("evidence directory does not exist"),
        "{}",
        result.stdout
    );
    assert!(!result.stderr.contains("unexpected argument"));
}
