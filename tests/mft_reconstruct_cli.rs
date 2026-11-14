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

//! End-to-end reconstruction from the public MFT and generated directory-index samples.
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo(s: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(s)
}
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_juntei"))
        .args(args)
        .args(["-K", "-q"])
        .output()
        .unwrap()
}
fn rows(file: &Path) -> Vec<HashMap<String, String>> {
    let mut reader = csv::Reader::from_path(file).unwrap();
    let header = reader.headers().unwrap().clone();
    reader
        .records()
        .map(|r| {
            header
                .iter()
                .zip(r.unwrap().iter())
                .map(|(k, v)| (k.into(), v.into()))
                .collect()
        })
        .collect()
}
fn success(out: &Output) {
    assert!(
        out.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn public_mft_reconstruction_bytes_agree_with_independent_csv_resident_exports() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("C");
    let csv = tmp.path().join("mft.csv");
    success(&run(&[
        "parse-ntfs",
        "mft",
        "-f",
        repo("test_files/mft/MFT").to_str().unwrap(),
        "-o",
        csv.to_str().unwrap(),
        "-i",
        "--reconstruct-filesystem",
        root.to_str().unwrap(),
    ]));
    let records = rows(&root.join("_Juntei-Reconstruction.csv"));
    let native = rows(&csv);
    let mut recovered = 0;
    for record in records
        .iter()
        .filter(|r| r["DataStatus"] == "resident_recovered")
    {
        let row = native
            .iter()
            .find(|r| {
                r["EntryNum"] == record["EntryNum"]
                    && r["IsADS"] == "False"
                    && !r["ResidentDataHex"].is_empty()
            })
            .unwrap();
        let expected: Vec<u8> = row["ResidentDataHex"]
            .split('-')
            .map(|b| u8::from_str_radix(b, 16).unwrap())
            .collect();
        assert_eq!(
            fs::read(root.join(&record["OutputPath"])).unwrap(),
            expected
        );
        recovered += 1;
    }
    assert_eq!(recovered, 22);
    assert!(
        records
            .iter()
            .any(|r| r["Source"].contains("INDEX_ROOT") && r["DataStatus"] == "i30_placeholder")
    );
    assert!(
        records
            .iter()
            .filter(|r| r["DataStatus"] == "non_resident_placeholder")
            .all(|r| fs::metadata(root.join(&r["OutputPath"])).unwrap().len() == 0)
    );
}

#[test]
fn external_directory_indexes_contribute_only_provenanced_empty_placeholders() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("C");
    success(&run(&[
        "parse-ntfs",
        "mft",
        "-f",
        repo("test_files/mft/MFT").to_str().unwrap(),
        "--reconstruct-filesystem",
        root.to_str().unwrap(),
        "--reconstruction-i30",
        repo("test_files/i30/Logs_I30").to_str().unwrap(),
    ]));
    let records = rows(&root.join("_Juntei-Reconstruction.csv"));
    let indexed: Vec<_> = records
        .iter()
        .filter(|r| r["Source"].ends_with("Logs_I30"))
        .collect();
    assert!(indexed.len() > 30);
    assert!(indexed.iter().any(|r| r["FromSlack"] == "true"));
    assert!(
        indexed
            .iter()
            .all(|r| r["RecoveredBytes"] == "0" && r["DeletionStatus"] == "unconfirmed")
    );
    for row in indexed
        .iter()
        .filter(|r| r["DataStatus"] == "i30_placeholder" && r["Dir"] == "false")
    {
        assert_eq!(
            fs::metadata(root.join(&row["OutputPath"])).unwrap().len(),
            0
        );
    }
}

#[test]
fn bad_fixup_cannot_be_exported_as_recovered_resident_content() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("C");
    let input = tmp.path().join("$MFT");
    let mut bytes = fs::read(repo("test_files/mft/MFT")).unwrap();
    // Public sample record 422 contains a resident SCF. Corrupt a sector's fixup marker.
    let offset = 422 * 1024 + 510;
    bytes[offset] ^= 0xff;
    fs::write(&input, bytes).unwrap();
    success(&run(&[
        "parse-ntfs",
        "mft",
        "-f",
        input.to_str().unwrap(),
        "--reconstruct-filesystem",
        root.to_str().unwrap(),
    ]));
    let records = rows(&root.join("_Juntei-Reconstruction.csv"));
    assert!(
        !records
            .iter()
            .any(|r| r["EntryNum"] == "422" && r["DataStatus"] == "resident_recovered")
    );
}

fn collection(tmp: &Path) -> PathBuf {
    let root = tmp.join("evidence");
    let host = root.join("HOST_F.TEST");
    fs::create_dir_all(host.join("uploads/auto/C%3A/Windows/System32")).unwrap();
    let ntfs = host.join("uploads/ntfs/%5C%5C.%5CC%3A");
    fs::create_dir_all(&ntfs).unwrap();
    fs::write(
        host.join("client_info.json"),
        br#"{"hostname":"HOST","fqdn":"host.example.test"}"#,
    )
    .unwrap();
    fs::copy(repo("test_files/mft/MFT"), ntfs.join("$MFT")).unwrap();
    // Cross-branch matching: collected index and MFT are under auto and ntfs respectively.
    fs::copy(
        repo("test_files/i30/Photos_I30"),
        host.join("uploads/auto/C%3A/Photos_I30"),
    )
    .unwrap();
    root
}

#[test]
fn automatic_csv_json_and_jsonl_build_one_tree_and_leave_raw_files_out_of_siem() {
    let tmp = tempfile::tempdir().unwrap();
    let evidence = collection(tmp.path());
    for format in ["csv", "json", "jsonl"] {
        let output = tmp.path().join(format);
        let timeline = output.join("timeline.jsonl");
        success(&run(&[
            "auto-parse",
            "-d",
            evidence.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-t",
            format,
            "-j",
            timeline.to_str().unwrap(),
            "-T",
            "2",
        ]));
        let manifest: Value =
            serde_json::from_slice(&fs::read(output.join("manifest.json")).unwrap()).unwrap();
        let mft = manifest["Tasks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["Artifact"] == "mft" && r["Purpose"] == "artifact")
            .unwrap();
        assert_eq!(mft["Status"], "succeeded");
        let inventory = mft["ReconstructionInventory"].as_str().unwrap();
        assert!(
            inventory
                .replace('\\', "/")
                .ends_with("/FileSystem/C/_Juntei-Reconstruction.csv")
        );
        assert!(!inventory.contains(tmp.path().to_str().unwrap()));
        let csv_manifest = rows(&output.join("manifest.csv"));
        let csv_mft = csv_manifest
            .iter()
            .find(|r| r["Artifact"] == "mft" && r["Purpose"] == "artifact")
            .unwrap();
        assert_eq!(csv_mft["ReconstructionInventory"], inventory);
        let records = rows(&output.join(inventory));
        assert!(records.iter().any(|r| r["Source"].ends_with("Photos_I30")));
        assert!(
            manifest["Tasks"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["Purpose"] == "json_timeline")
                .all(|r| r.get("ReconstructionRoot").is_none())
        );
        assert!(!output.join("HOST/FileSystem/C-MFT-2").exists());
        assert!(fs::read_to_string(&timeline).unwrap().lines().all(|s| {
            serde_json::from_str::<Value>(s)
                .unwrap()
                .get("DataStatus")
                .is_none()
        }));
    }
}

#[test]
fn auto_parse_can_opt_out_and_refuses_preexisting_trees_before_clobbering() {
    let tmp = tempfile::tempdir().unwrap();
    let evidence = collection(tmp.path());
    let output = tmp.path().join("results");
    success(&run(&[
        "auto-parse",
        "-d",
        evidence.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-T",
        "2",
        "--no-reconstruct-filesystem",
    ]));
    assert!(!output.join("HOST/FileSystem/C").exists());
    let mft = output.join("HOST/FileSystem/MFT.csv");
    let before = fs::read(&mft).unwrap();
    fs::create_dir_all(output.join("HOST/FileSystem/C")).unwrap();
    let sentinel = output.join("HOST/FileSystem/C/keep.txt");
    fs::write(&sentinel, b"keep").unwrap();
    let out = run(&[
        "auto-parse",
        "-d",
        evidence.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-T",
        "2",
        "-C",
    ]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("requires a new directory"));
    assert_eq!(fs::read(sentinel).unwrap(), b"keep");
    assert_eq!(fs::read(mft).unwrap(), before);
}

#[test]
fn reconstruction_requires_a_whole_single_mft_and_refuses_output_overlap() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("C");
    for extra in [["-r", "0-5"], ["-D", "5"], ["--cutoff", "2026-01-01"]] {
        let out = run(&[
            "parse-ntfs",
            "mft",
            "-f",
            repo("test_files/mft/MFT").to_str().unwrap(),
            "--reconstruct-filesystem",
            root.to_str().unwrap(),
            extra[0],
            extra[1],
        ]);
        assert!(!out.status.success());
        assert!(!root.exists());
    }
    let input = tmp.path().join("$MFT");
    fs::copy(repo("test_files/mft/MFT"), &input).unwrap();
    let before = fs::read(&input).unwrap();
    let out = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        input.to_str().unwrap(),
        "-o",
        input.to_str().unwrap(),
        "-C",
        "--reconstruct-filesystem",
        input.to_str().unwrap(),
    ]);
    assert!(!out.status.success());
    assert_eq!(fs::read(input).unwrap(), before);
}

#[test]
fn multiple_volumes_and_duplicate_computer_names_never_share_reconstruction_tasks() {
    let tmp = tempfile::tempdir().unwrap();
    let evidence = collection(tmp.path());
    let first = evidence.join("HOST_F.TEST");
    for dir in [
        first.join("uploads/ntfs/D%3A"),
        first.join("ExtraVolume"),
        evidence.join("SECOND_F.TEST/C"),
    ] {
        fs::create_dir_all(&dir).unwrap();
        fs::copy(repo("test_files/mft/MFT"), dir.join("$MFT")).unwrap();
    }
    let second = evidence.join("SECOND_F.TEST");
    fs::create_dir_all(second.join("C/Windows/System32")).unwrap();
    fs::write(second.join("client_info.json"), br#"{"hostname":"HOST"}"#).unwrap();
    let output = tmp.path().join("parsed");
    success(&run(&[
        "auto-parse",
        "-d",
        evidence.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-T",
        "2",
    ]));
    let manifest: Value =
        serde_json::from_slice(&fs::read(output.join("manifest.json")).unwrap()).unwrap();
    let tasks: Vec<_> = manifest["Tasks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["Artifact"] == "mft")
        .collect();
    assert_eq!(tasks.len(), 4);
    let mut roots = std::collections::HashSet::new();
    for task in tasks {
        assert_eq!(task["Status"], "succeeded");
        assert_eq!(
            task["Command"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| v.as_str() == Some("--reconstruct-filesystem"))
                .count(),
            1
        );
        let root = task["ReconstructionRoot"].as_str().unwrap();
        assert!(roots.insert(root));
        assert!(
            output
                .join(root)
                .join("_Juntei-Reconstruction.csv")
                .is_file()
        );
    }
    assert!(
        roots
            .iter()
            .any(|r| r.replace('\\', "/").ends_with("/UnknownVolume"))
    );
    assert!(roots.iter().any(|r| r.replace('\\', "/").ends_with("/D")));
}

#[test]
fn an_unreadable_or_damaged_optional_index_does_not_discard_valid_mft_results() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("C");
    let damaged = tmp.path().join("$I30");
    fs::write(&damaged, b"damaged").unwrap();
    let out = run(&[
        "parse-ntfs",
        "mft",
        "-f",
        repo("test_files/mft/MFT").to_str().unwrap(),
        "--reconstruct-filesystem",
        root.to_str().unwrap(),
        "--reconstruction-i30",
        damaged.to_str().unwrap(),
    ]);
    success(&out);
    assert!(String::from_utf8_lossy(&out.stdout).contains("Skipped reconstruction index"));
    let inventory = rows(&root.join("_Juntei-Reconstruction.csv"));
    assert_eq!(
        inventory
            .iter()
            .filter(|r| r["DataStatus"] == "resident_recovered")
            .count(),
        22
    );
    let problem = inventory
        .iter()
        .find(|r| r["DataStatus"] == "i30_input_error")
        .unwrap();
    assert!(problem["Notes"].contains("No INDX records"));
    assert!(problem["OutputPath"].is_empty());
}
