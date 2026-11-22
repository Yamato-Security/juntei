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

//! End-to-end file scanning, offline rules, both output formats and evidence protection.
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_juntei"))
        .args(args)
        .output()
        .unwrap()
}
fn fixture(root: &Path) {
    fs::create_dir_all(root.join("evidence/HOST/C")).unwrap();
    fs::write(
        root.join("evidence/HOST/client_info.json"),
        r#"{"hostname":"HOST"}"#,
    )
    .unwrap();
    fs::write(
        root.join("evidence/HOST/C/inert.exe.quarantined"),
        b"JUNTEI_INERT_FIXTURE",
    )
    .unwrap();
    fs::write(root.join("evidence/HOST/C/no-match.bin"), b"ordinary file").unwrap();
    fs::write(root.join("evidence/HOST/C/empty.bin"), b"").unwrap();
    fs::write(
        root.join("local.yar"),
        r#"rule InertFixture : validation {
        meta: author = "Juntei test" score = 1
        strings: $marker = "JUNTEI_INERT_FIXTURE"
        condition: $marker and filesize == 20
    }"#,
    )
    .unwrap();
}
fn text(path: &Path) -> &str {
    path.to_str().unwrap()
}
fn summary(root: &Path) -> Value {
    serde_json::from_slice(&fs::read(root.join("Summary.json")).unwrap()).unwrap()
}

#[test]
fn offline_csv_and_jsonl_agree_with_relative_provenance() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fixture(root);
    let evidence = root.join("evidence");
    let rules = root.join("local.yar");
    let original = fs::read(evidence.join("HOST/C/inert.exe.quarantined")).unwrap();
    for format in ["csv", "jsonl"] {
        let out = root.join(format);
        let result = run(&[
            "yara-scan",
            "-d",
            text(&evidence),
            "-r",
            text(&rules),
            "-o",
            text(&out),
            "-t",
            format,
            "-T",
            "2",
            "-q",
            "-K",
        ]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stdout)
        );
        let s = summary(&out);
        assert_eq!(s["RuleMatches"], 1);
        assert_eq!(s["MatchingFiles"], 1);
        assert_eq!(s["Statuses"]["Scanned"], 4);
        assert_eq!(s["Ruleset"], "local");
        assert!(s["DownloadURL"].is_null());
        assert_eq!(s["PublicRules"], 1);
        assert!(
            !fs::read_to_string(out.join("Summary.json"))
                .unwrap()
                .contains(text(root))
        );
        if format == "csv" {
            let mut reader = csv::Reader::from_path(out.join("YaraDetections.csv")).unwrap();
            let headers = reader.headers().unwrap().clone();
            assert_eq!(headers.get(0), Some("Computer"));
            assert_eq!(headers.get(3), Some("SHA256"));
            let rows: Vec<_> = reader.records().map(Result::unwrap).collect();
            assert_eq!(rows.len(), 1);
            assert_eq!(&rows[0][0], "HOST");
            assert_eq!(&rows[0][1], "HOST/C/inert.exe.quarantined");
            assert_eq!(&rows[0][5], "InertFixture");
            let mut files = csv::Reader::from_path(out.join("YaraFiles.csv")).unwrap();
            let empty = files
                .records()
                .map(Result::unwrap)
                .find(|r| &r[1] == "HOST/C/empty.bin")
                .unwrap();
            assert_eq!(&empty[2], "0");
            assert_eq!(
                &empty[3],
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            );
            assert_eq!(&empty[4], "Scanned");
            assert_eq!(
                serde_json::from_str::<Value>(&rows[0][8]).unwrap()["score"],
                1
            );
        } else {
            let data = fs::read_to_string(out.join("YaraDetections.jsonl")).unwrap();
            let rows: Vec<Value> = data
                .lines()
                .map(|l| serde_json::from_str(l).unwrap())
                .collect();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0]["Computer"], "HOST");
            assert_eq!(rows[0]["FilePath"], "HOST/C/inert.exe.quarantined");
        }
    }
    assert_eq!(
        fs::read(evidence.join("HOST/C/inert.exe.quarantined")).unwrap(),
        original
    );
}

#[test]
fn single_file_zero_matches_has_csv_header_and_refuses_overwrite() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fixture(root);
    let input = root.join("evidence/HOST/C/no-match.bin");
    let rules = root.join("local.yar");
    let out = root.join("evidence/HOST/C/scan-results");
    let args = [
        "yara-scan",
        "-f",
        text(&input),
        "-r",
        text(&rules),
        "-o",
        text(&out),
        "-q",
        "-K",
    ];
    assert!(run(&args).status.success());
    let csv = fs::read_to_string(out.join("YaraDetections.csv")).unwrap();
    assert_eq!(csv.lines().count(), 1);
    assert!(csv.starts_with("Computer,FilePath,Size,SHA256,"));
    assert_eq!(summary(&out)["RuleMatches"], 0);
    let original = fs::read(out.join("Summary.json")).unwrap();
    assert_eq!(run(&args).status.code(), Some(2));
    assert_eq!(fs::read(out.join("Summary.json")).unwrap(), original);
}

#[test]
fn output_cannot_pollute_input_and_bad_rules_cannot_produce_clean_summary() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fixture(root);
    let input = root.join("evidence");
    let rules = root.join("local.yar");
    let out = input.join("output");
    assert_eq!(
        run(&[
            "yara-scan",
            "-d",
            text(&input),
            "-r",
            text(&rules),
            "-o",
            text(&out),
            "-q"
        ])
        .status
        .code(),
        Some(2)
    );
    assert!(!out.exists());
    fs::write(&rules, "rule bad { condition: nonexistent }").unwrap();
    let out = root.join("bad-results");
    assert_eq!(
        run(&[
            "yara-scan",
            "-d",
            text(&input),
            "-r",
            text(&rules),
            "-o",
            text(&out),
            "-q"
        ])
        .status
        .code(),
        Some(2)
    );
    assert!(!out.join("Summary.json").exists());
    assert!(out.join("CompilationError.txt").exists());
    assert!(out.join("RunStarted.json").exists());
}

#[test]
fn oversized_file_is_explicitly_incomplete_and_rule_source_is_excluded() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fixture(root);
    let input = root.join("evidence");
    let rules = input.join("local.yar");
    fs::copy(root.join("local.yar"), &rules).unwrap();
    fs::File::create(input.join("too-large.bin"))
        .unwrap()
        .set_len(1024 * 1024 + 1)
        .unwrap();
    let out = root.join("results");
    let result = run(&[
        "yara-scan",
        "-d",
        text(&input),
        "-r",
        text(&rules),
        "-o",
        text(&out),
        "--max-file-mib",
        "1",
        "-q",
    ]);
    assert_eq!(result.status.code(), Some(3));
    let s = summary(&out);
    assert_eq!(s["Status"], "Incomplete");
    assert_eq!(s["Statuses"]["SkippedTooLarge"], 1);
    assert_eq!(s["Statuses"]["SkippedRuleSource"], 1);
}

#[test]
#[ignore = "downloads and compiles all three official YARA Forge 20261115 packages"]
fn real_forge_packages_detect_an_inert_marker() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let evidence = root.join("evidence");
    fs::create_dir(&evidence).unwrap();
    // An inert string fixture, not an executable or an actual C2 agent.
    fs::write(
        evidence.join("marker.exe.quarantined"),
        b"MZ\0github.com/bishopfox/sliver/protobuf/sliverpbb.",
    )
    .unwrap();
    fs::write(
        evidence.join("ordinary.txt"),
        b"Plain text for a small negative-control smoke test.",
    )
    .unwrap();
    for set in ["core", "extended", "full"] {
        let out = root.join(set);
        let result = run(&[
            "yara-scan",
            "-d",
            text(&evidence),
            "--ruleset",
            set,
            "--rules-release",
            "20261115",
            "-o",
            text(&out),
            "-t",
            "jsonl",
            "-q",
            "-K",
        ]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stdout)
        );
        let hits: Vec<Value> = fs::read_to_string(out.join("YaraDetections.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert!(
            hits.iter()
                .any(|h| h["Rule"] == "DITEKSHEN_INDICATOR_TOOL_Sliver"
                    && h["FilePath"] == "marker.exe.quarantined")
        );
        assert_eq!(summary(&out)["Statuses"]["Scanned"], 2);
    }
}

#[test]
fn default_scans_ordinary_files_above_the_former_64_mib_limit() {
    use std::io::Write;
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let input = root.join("large.bin");
    let mut file = fs::File::create(&input).unwrap();
    file.write_all(b"JUNTEI_NO_DEFAULT_SIZE_LIMIT").unwrap();
    file.set_len(65 * 1024 * 1024 + 1).unwrap();
    drop(file);
    let rules = root.join("local.yar");
    fs::write(&rules,"rule unlimited { strings: $a=\"JUNTEI_NO_DEFAULT_SIZE_LIMIT\" condition: filesize > 64MB and $a }").unwrap();
    let out = root.join("out");
    let result = run(&[
        "yara-scan",
        "-f",
        text(&input),
        "--rules",
        text(&rules),
        "-o",
        text(&out),
        "-q",
        "-K",
    ]);
    assert!(
        result.status.success(),
        "{} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let s = summary(&out);
    assert_eq!(s["RuleMatches"], 1);
    assert!(s["MaxFileMiB"].is_null());
    assert!(s["MaxMFTMiB"].is_null());
}
