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

//! `parse-ntfs boot` and `parse-ntfs secure-sds` on the DFIR Artifact Museum's Windows 11 `$Boot`
//! and `$Secure:$SDS` (test_files/boot, test_files/secure_sds; MIT, see their NOTICE files).

use flate2::read::GzDecoder;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Run {
    let out = Command::new(env!("CARGO_BIN_EXE_juntei"))
        .args(args)
        .output()
        .unwrap();
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn sds_bytes() -> Vec<u8> {
    let mut data = Vec::new();
    GzDecoder::new(std::fs::File::open(repo("test_files/secure_sds/SDS.gz")).unwrap())
        .read_to_end(&mut data)
        .unwrap();
    data
}

fn csv_rows(text: &str) -> (Vec<String>, Vec<HashMap<String, String>>) {
    let mut reader = csv::Reader::from_reader(text.as_bytes());
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

fn json_lines(text: &str) -> Vec<Value> {
    text.lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// A CSV cell for a JSON value: numbers and strings as they are, `True`/`False`, lists joined by
/// `; `, empty for null.
fn as_cell(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(items) => items.iter().map(as_cell).collect::<Vec<_>>().join("; "),
        other => other.to_string(),
    }
}

/// CSV and JSONL say the same, field for field.
fn same_in_csv_and_jsonl(csv: &str, jsonl: &str) -> usize {
    let (header, rows) = csv_rows(csv);
    let docs = json_lines(jsonl);
    assert_eq!(rows.len(), docs.len());
    for (row, doc) in rows.iter().zip(&docs) {
        let keys: Vec<&String> = doc.as_object().unwrap().keys().collect();
        assert_eq!(keys, header.iter().collect::<Vec<_>>());
        for field in &header {
            assert_eq!(row[field], as_cell(&doc[field]), "{field}");
        }
    }
    rows.len()
}

// --- parse-ntfs boot -----------------------------------------------------------------------------

#[test]
fn boot_summary_and_fields() {
    let boot = repo("test_files/boot/Boot");
    let r = run(&["parse-ntfs", "boot", "-f", boot.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for line in [
        "$Boot files: 1",
        "Boot sectors parsed: 1",
        "volume serial number 180B-1E67 (180B1E67), 59.3 GiB, clusters of 4,096 bytes, $MFT at cluster 786,432",
        "Boot code (SHA-256): 2354794a213560f40d3eb095ba35808108faf2c27c1bd849839921e469d5f13a",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    assert!(!r.stdout.contains("problems"), "{}", r.stdout);

    let csv = run(&[
        "parse-ntfs",
        "boot",
        "-f",
        boot.to_str().unwrap(),
        "-t",
        "csv",
        "-K",
    ]);
    let jsonl = run(&[
        "parse-ntfs",
        "boot",
        "-f",
        boot.to_str().unwrap(),
        "-t",
        "jsonl",
        "-K",
    ]);
    assert_eq!(same_in_csv_and_jsonl(&csv.stdout, &jsonl.stdout), 1);
    let (_, rows) = csv_rows(&csv.stdout);
    let row = &rows[0];
    for (field, value) in [
        ("VolumeSerialNumber", "180B1E67"),
        ("VolumeSerialNumber64", "EA180B53180B1E67"),
        ("OemId", "NTFS"),
        ("BytesPerSector", "512"),
        ("SectorsPerCluster", "8"),
        ("ClusterSize", "4096"),
        ("TotalSectors", "124370943"),
        ("VolumeSize", "63677922816"),
        ("HiddenSectors", "239616"),
        ("MftCluster", "786432"),
        ("MftOffset", "3221225472"),
        ("MftMirrorCluster", "2"),
        ("MftMirrorOffset", "8192"),
        ("FileRecordSize", "1024"),
        ("IndexRecordSize", "4096"),
        ("MediaDescriptor", "F8"),
        ("JumpInstruction", "EB 52 90"),
        ("EndSignature", "55 AA"),
        (
            "BootCodeSHA256",
            "2354794a213560f40d3eb095ba35808108faf2c27c1bd849839921e469d5f13a",
        ),
        ("Problems", ""),
    ] {
        assert_eq!(row[field], value, "{field}");
    }
}

#[test]
fn boot_files_in_a_collection_and_damage() {
    let sample = std::fs::read(repo("test_files/boot/Boot")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let host = dir.path().join("HOST/C");
    std::fs::create_dir_all(&host).unwrap();
    std::fs::write(host.join("$Boot"), &sample).unwrap();
    // a volume image: found by its first sector, read for its first 8 KiB
    let mut volume = sample.clone();
    volume.resize(64 * 1024, 0x5a);
    std::fs::write(dir.path().join("volume.dd"), &volume).unwrap();
    // a copy cut off after its first sector, with another boot code and a FAT field set
    let mut damaged = sample[..512].to_vec();
    damaged[0x0e] = 1;
    damaged[0x100] ^= 0xff;
    std::fs::write(dir.path().join("C_$Boot"), &damaged).unwrap();
    std::fs::write(host.join("notes.txt"), b"not a boot sector").unwrap();

    let out = tempfile::tempdir().unwrap();
    let csv = out.path().join("boot.csv");
    let r = run(&[
        "parse-ntfs",
        "boot",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        csv.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for line in [
        "$Boot files: 3",
        "Boot sectors parsed: 3",
        "Boot code (SHA-256), by number of boot sectors: 2",
        "  2354794a213560f40d3eb095ba35808108faf2c27c1bd849839921e469d5f13a: ",
        "Boot sectors with problems: 1",
        "a FAT field that NTFS keeps at zero is not zero; only 512 of $Boot's 8,192 bytes: BootCodeSHA256 covers those",
        "(3 boot sectors, ",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let (_, rows) = csv_rows(&std::fs::read_to_string(&csv).unwrap());
    let by_name: HashMap<String, &HashMap<String, String>> = rows
        .iter()
        .map(|row| {
            let name = Path::new(&row["SourcePath"])
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            (name, row)
        })
        .collect();
    assert_eq!(by_name.len(), 3);
    assert_eq!(
        by_name["volume.dd"]["BootCodeSHA256"],
        by_name["$Boot"]["BootCodeSHA256"]
    );
    assert_eq!(by_name["volume.dd"]["Problems"], "");
    assert_ne!(
        by_name["C_$Boot"]["BootCodeSHA256"],
        by_name["$Boot"]["BootCodeSHA256"]
    );
    assert_eq!(by_name["C_$Boot"]["VolumeSerialNumber"], "180B1E67");

    // nothing to read
    let empty = tempfile::tempdir().unwrap();
    std::fs::write(empty.path().join("notes.txt"), b"nothing").unwrap();
    let r = run(&[
        "parse-ntfs",
        "boot",
        "-d",
        empty.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout.contains("No $Boot files were found"),
        "{}",
        r.stdout
    );
    let short = empty.path().join("$Boot");
    std::fs::write(&short, b"too short").unwrap();
    let r = run(&["parse-ntfs", "boot", "-f", short.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout
            .contains("a boot sector is 512 bytes; only 9 were read"),
        "{}",
        r.stdout
    );
}

// --- parse-ntfs secure-sds ------------------------------------------------------------------------

#[test]
fn secure_sds_summary_and_fields() {
    let dir = tempfile::tempdir().unwrap();
    let sds = dir.path().join("$Secure_$SDS");
    std::fs::write(&sds, sds_bytes()).unwrap();
    let r = run(&[
        "parse-ntfs",
        "secure-sds",
        "-f",
        sds.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for line in [
        "$SDS streams: 1",
        "$SDS streams parsed: 1",
        "Security descriptors: 1,716\n",
        "  NT AUTHORITY\\SYSTEM (S-1-5-18): 761\n",
        "  BUILTIN\\Administrators (S-1-5-32-544): 593\n",
        "  S-1-5-21-4063860464-1508573791-632671957-1001: 215\n",
        "To save every security descriptor, add -o <FILE>",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    for absent in ["NULL DACL", "problems", "Problems in streams", "mirror"] {
        assert!(!r.stdout.contains(absent), "{absent}\n{}", r.stdout);
    }

    let csv = run(&[
        "parse-ntfs",
        "secure-sds",
        "-f",
        sds.to_str().unwrap(),
        "-t",
        "csv",
        "-K",
    ]);
    let jsonl = run(&[
        "parse-ntfs",
        "secure-sds",
        "-f",
        sds.to_str().unwrap(),
        "-t",
        "jsonl",
        "-K",
    ]);
    assert_eq!(same_in_csv_and_jsonl(&csv.stdout, &jsonl.stdout), 1716);
    let (header, rows) = csv_rows(&csv.stdout);
    assert_eq!(
        header,
        [
            "SourcePath",
            "SecurityID",
            "Hash",
            "EntryOffset",
            "FromMirror",
            "Size",
            "Owner",
            "OwnerName",
            "Group",
            "GroupName",
            "Control",
            "DaclAceCount",
            "Dacl",
            "SaclAceCount",
            "Sacl",
            "SDDL",
            "Problems"
        ]
    );
    // every descriptor decoded, with SDDL
    assert!(
        rows.iter()
            .all(|r| r["Problems"].is_empty() && !r["SDDL"].is_empty())
    );
    let by_id: HashMap<&str, &HashMap<String, String>> =
        rows.iter().map(|r| (r["SecurityID"].as_str(), r)).collect();
    let first = by_id["256"];
    for (field, value) in [
        ("Hash", "32FEC6CB"),
        ("EntryOffset", "0"),
        ("FromMirror", "False"),
        ("Size", "120"),
        ("Owner", "S-1-5-18"),
        ("OwnerName", "NT AUTHORITY\\SYSTEM"),
        ("Group", "S-1-5-32-544"),
        ("GroupName", "BUILTIN\\Administrators"),
        ("Control", "SE_DACL_PRESENT|SE_SELF_RELATIVE"),
        ("DaclAceCount", "2"),
        (
            "Dacl",
            "NT AUTHORITY\\SYSTEM:(R); BUILTIN\\Administrators:(R)",
        ),
        ("SaclAceCount", ""),
        ("Sacl", ""),
        (
            "SDDL",
            "O:S-1-5-18G:S-1-5-32-544D:(A;;0x00120089;;;S-1-5-18)(A;;0x00120089;;;S-1-5-32-544)",
        ),
    ] {
        assert_eq!(first[field], value, "{field}");
    }
    // a mandatory label, inherited-only ACEs and generic rights
    let labelled = by_id["264"];
    assert_eq!(
        labelled["Dacl"],
        "BUILTIN\\Administrators:(OI)(CI)(F); NT AUTHORITY\\SYSTEM:(OI)(CI)(F); \
         BUILTIN\\Users:(OI)(CI)(RX); NT AUTHORITY\\Authenticated Users:(OI)(CI)(IO)(DE,GE,GW,GR); \
         NT AUTHORITY\\Authenticated Users:(AD)"
    );
    assert_eq!(
        labelled["Sacl"],
        "Mandatory Label\\High Mandatory Level:(OI)(IO)(NP)(NW)"
    );
    // an app container condition, a trust label and a resource attribute claim
    let callback = rows
        .iter()
        .find(|r| r["Dacl"].contains("Microsoft.549981C3F5F10_8wekyb3d8bbwe"))
        .unwrap();
    assert!(
        callback["Dacl"].contains(
            "BUILTIN\\Users:(ACCESS_ALLOWED_CALLBACK)(OI)(CI)(RX) \
             if (WIN://SYSAPPID Contains \"Microsoft.549981C3F5F10_8wekyb3d8bbwe\")"
        ),
        "{}",
        callback["Dacl"]
    );
    assert!(
        callback["SDDL"].contains(
            "(XA;OICI;0x001200a9;;;S-1-5-32-545;\
             (WIN://SYSAPPID Contains \"Microsoft.549981C3F5F10_8wekyb3d8bbwe\"))"
        ),
        "{}",
        callback["SDDL"]
    );
    let claim = by_id["1695"];
    assert_eq!(
        claim["Sacl"],
        "Everyone:(SYSTEM_RESOURCE_ATTRIBUTE)(OI)(CI)(IO)(0x00000000) (\"IMAGELOAD\",TU,0x0,1)"
    );
    assert!(
        rows.iter()
            .any(|r| r["Sacl"].contains("S-1-19-512-4096:(SYSTEM_PROCESS_TRUST_LABEL)"))
    );
}

#[test]
fn secure_sds_in_a_collection_and_damage() {
    let mut data = sds_bytes();
    let dir = tempfile::tempdir().unwrap();
    let host = dir.path().join("HOST/C");
    std::fs::create_dir_all(&host).unwrap();
    std::fs::write(host.join("$Secure_$SDS"), &data).unwrap();
    // renamed: found by its first entry
    std::fs::write(dir.path().join("exported.bin"), &data).unwrap();
    // a damaged copy: the first entry's offset, and a descriptor changed in both copies
    data[8] ^= 0xff;
    let second = 128 + 20 + 40;
    data[second] ^= 0xff;
    data[0x40000 + second] ^= 0xff;
    std::fs::write(dir.path().join("damaged_$SDS"), &data).unwrap();
    std::fs::write(host.join("notes.txt"), b"not an SDS stream at all").unwrap();

    let out = tempfile::tempdir().unwrap();
    let json = out.path().join("sds.json");
    let r = run(&[
        "parse-ntfs",
        "secure-sds",
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        json.to_str().unwrap(),
        "-t",
        "json",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for line in [
        "$SDS streams: 3",
        "$SDS streams parsed: 3",
        "Security descriptors: 5,148 (copies from the mirror that differ, or of damaged entries: 1)",
        "Descriptors whose recorded hash is not theirs: 1",
        "Descriptors with problems: 1",
        "Problems in streams: 1",
        "0x0: the entry is damaged; its mirror copy at 0x40000 is read instead",
        "(5,148 security descriptors, ",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
    let text = std::fs::read_to_string(&json).unwrap();
    let docs: Vec<Value> = serde_json::Deserializer::from_str(&text)
        .into_iter::<Value>()
        .map(Result::unwrap)
        .collect();
    assert_eq!(docs.len(), 5148);
    let damaged: Vec<&Value> = docs
        .iter()
        .filter(|d| d["SourcePath"].as_str().unwrap().ends_with("damaged_$SDS"))
        .collect();
    assert_eq!(damaged[0]["FromMirror"], true);
    assert_eq!(damaged[0]["EntryOffset"], 0x40000);
    assert_eq!(damaged[1]["SecurityID"], 257);
    assert_eq!(
        damaged[1]["Problems"],
        serde_json::json!(["the recorded hash is not the descriptor's"])
    );
    assert_eq!(damaged[1]["Sacl"], Value::Null);

    // nothing to read
    let empty = tempfile::tempdir().unwrap();
    std::fs::write(empty.path().join("notes.txt"), b"nothing").unwrap();
    let r = run(&[
        "parse-ntfs",
        "secure-sds",
        "-d",
        empty.path().to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.stdout
            .contains("No $SDS streams ($Secure_$SDS) were found"),
        "{}",
        r.stdout
    );
    let zeros = empty.path().join("$Secure_$SDS");
    std::fs::write(&zeros, vec![0u8; 4096]).unwrap();
    let r = run(&[
        "parse-ntfs",
        "secure-sds",
        "-f",
        zeros.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.stdout.contains("no $SDS entries"), "{}", r.stdout);
}

// --- auto-parse ----------------------------------------------------------------------------------

#[test]
fn auto_parse_reads_boot_and_sds() {
    let evidence = tempfile::tempdir().unwrap();
    let volume = evidence.path().join("HOST1/C");
    // a Windows directory makes HOST1 a host to auto-parse
    std::fs::create_dir_all(volume.join("Windows/System32")).unwrap();
    std::fs::copy(repo("test_files/boot/Boot"), volume.join("$Boot")).unwrap();
    std::fs::write(volume.join("$Secure_$SDS"), sds_bytes()).unwrap();
    let root = tempfile::tempdir().unwrap();
    let output = root.path().join("parsed");
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "stdout:\n{}\nstderr:\n{}", r.stdout, r.stderr);
    let file_system = output.join("HOST1/FileSystem");
    let (_, boot) = csv_rows(&std::fs::read_to_string(file_system.join("Boot.csv")).unwrap());
    assert_eq!(boot.len(), 1);
    assert_eq!(boot[0]["VolumeSerialNumber"], "180B1E67");
    let (_, sds) = csv_rows(&std::fs::read_to_string(file_system.join("SecureSDS.csv")).unwrap());
    assert_eq!(sds.len(), 1716);
    assert!(file_system.join("Boot-logs.txt").is_file());
    assert!(file_system.join("SecureSDS-logs.txt").is_file());
    let manifest: Value =
        serde_json::from_reader(std::fs::File::open(output.join("manifest.json")).unwrap())
            .unwrap();
    for artifact in ["boot", "secure-sds"] {
        let task = manifest["Tasks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["Artifact"] == artifact)
            .unwrap();
        assert_eq!(task["Status"], "succeeded", "{task}");
        assert_eq!(task["Category"], "FileSystem", "{task}");
    }
}
