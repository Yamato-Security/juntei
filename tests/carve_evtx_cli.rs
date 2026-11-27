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

//! `carve-evtx` end to end: the NTFS disk of test_files/carve_evtx/sample.raw.gz (see its
//! make_sample.py for what is on it) and plain files built from the sample logs.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

const FIXTURES: &str = "test_files/carve_evtx";
const CHUNK: usize = 0x10000;

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Run {
    let out = Command::new(env!("CARGO_BIN_EXE_juntei"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(args)
        .args(["-q", "-K"])
        .output()
        .expect("juntei runs");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    }
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES)
        .join(name)
}

fn sample_log(name: &str) -> Vec<u8> {
    fs::read(fixture("evtx").join(name)).unwrap()
}

fn task_log() -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/persistence/task-creation.evtx"),
    )
    .unwrap()
}

fn first_chunk(log: &[u8]) -> Vec<u8> {
    log[0x1000..0x1000 + CHUNK].to_vec()
}

/// The sample disk, unpacked into `dir`.
fn sample_disk(dir: &Path) -> PathBuf {
    let mut raw = Vec::new();
    flate2::read::GzDecoder::new(fs::File::open(fixture("sample.raw.gz")).unwrap())
        .read_to_end(&mut raw)
        .unwrap();
    let path = dir.join("sample.raw");
    fs::write(&path, raw).unwrap();
    path
}

/// The rows of a CSV report.
fn rows(path: &Path) -> Vec<BTreeMap<String, String>> {
    let mut reader = csv::Reader::from_path(path).unwrap();
    let headers = reader.headers().unwrap().clone();
    reader
        .records()
        .map(|r| {
            let r = r.unwrap();
            headers
                .iter()
                .zip(r.iter())
                .map(|(h, v)| (h.to_string(), v.to_string()))
                .collect()
        })
        .collect()
}

fn count(rows: &[BTreeMap<String, String>], key: &str, value: &str) -> usize {
    rows.iter().filter(|r| r[key] == value).count()
}

/// Records each rebuilt log holds, read with the evtx crate.
fn evtx_records(path: &Path) -> usize {
    let mut parser = evtx::EvtxParser::from_path(path).unwrap();
    parser.records().filter(Result::is_ok).count()
}

#[test]
fn ntfs_disk_places_chunks_and_rebuilds_deleted_logs() {
    let dir = tempfile::tempdir().unwrap();
    let disk = sample_disk(dir.path());
    let out = dir.path().join("out");
    let result = run(&[
        "carve-evtx",
        "-f",
        disk.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(result.code, 0, "{}{}", result.stdout, result.stderr);
    assert!(
        result.stdout.contains("NTFS volumes: 1"),
        "{}",
        result.stdout
    );
    assert!(result.stdout.contains("Live logs: 2"), "{}", result.stdout);
    assert!(
        result.stdout.contains("Deleted logs rebuilt: 2"),
        "{}",
        result.stdout
    );

    let chunks = rows(&out.join("chunks.csv"));
    assert_eq!(chunks.len(), 10);
    let by: Vec<(&str, &str, &str)> = chunks
        .iter()
        .map(|r| {
            (
                r["Location"].as_str(),
                r["Status"].as_str(),
                r["Channel"].as_str(),
            )
        })
        .collect();
    // Both compressed live logs are found and left out.
    assert_eq!(count(&chunks, "Status", "SkippedLiveLog"), 2);
    assert!(
        chunks
            .iter()
            .filter(|r| r["Location"] == "LiveLog")
            .all(|r| r["Compressed"] == "true" && r["Owner"].contains("winevt/Logs/"))
    );
    // Each deleted log is rebuilt from its $MFT record (the compressed one through LZNT1), and
    // its chunk found by the search on the disk is the same chunk.
    let deleted: Vec<_> = chunks
        .iter()
        .filter(|r| r["Location"] == "DeletedLog")
        .collect();
    assert_eq!(deleted.len(), 4);
    assert_eq!(count(&chunks, "Status", "DuplicateChunk"), 2);
    assert!(by.contains(&(
        "DeletedLog",
        "Written",
        "Microsoft-Windows-HelloForBusiness/Operational"
    )));
    // An empty page has no records to name its channel; its log's file name does.
    assert!(by.contains(&("DeletedLog", "Written", "Microsoft-Windows-WorkFolders/WHC")));
    for name in [
        "Windows/System32/winevt/Logs/Microsoft-Windows-HelloForBusiness%4Operational.evtx",
        "Logs/Microsoft-Windows-WorkFolders%4WHC.evtx",
    ] {
        let rebuilt = fs::read(out.join("Recovered/Volume1/C").join(name)).unwrap();
        let original = sample_log(name.rsplit('/').next().unwrap());
        assert_eq!(rebuilt, original, "{name}");
    }
    // In free space: an older copy of a live chunk (all its records are in the live log), a
    // chunk that is gone from the live logs, and an NTFS-compressed one; in the pagefile, another.
    let free: Vec<_> = chunks
        .iter()
        .filter(|r| r["Location"] == "FreeSpace")
        .collect();
    assert_eq!(free.len(), 3);
    assert_eq!(count(&chunks, "Status", "RecordsInLiveLogs"), 1);
    assert!(by.contains(&("FreeSpace", "Written", "Application")));
    assert!(
        free.iter()
            .any(|r| r["Status"] == "Written" && r["Compressed"] == "true")
    );
    let page: Vec<_> = chunks
        .iter()
        .filter(|r| r["Location"] == "AllocatedFile")
        .collect();
    assert_eq!(page.len(), 1);
    assert_eq!(page[0]["Owner"], "C/pagefile.sys");
    assert_eq!(page[0]["Status"], "Written");
    assert_eq!(count(&chunks, "Status", "Written"), 5);

    // One rebuilt log per channel, which an independent parser reads.
    let mut carved: Vec<String> = fs::read_dir(out.join("Carved"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    carved.sort();
    assert_eq!(
        carved,
        [
            "Application.evtx",
            "Microsoft-Windows-HelloForBusiness%4Operational.evtx",
            "Microsoft-Windows-WorkFolders%4WHC.evtx",
            "Security.evtx"
        ]
    );
    let carved_dir = out.join("Carved");
    assert_eq!(evtx_records(&carved_dir.join("Application.evtx")), 17);
    assert_eq!(evtx_records(&carved_dir.join("Security.evtx")), 8);
    assert_eq!(
        evtx_records(&carved_dir.join("Microsoft-Windows-HelloForBusiness%4Operational.evtx")),
        5
    );
    // The records report lists every record of the chunks written, slack ones included.
    let records = rows(&out.join("records.csv"));
    assert_eq!(records.len(), 66);
    assert_eq!(count(&records, "Allocation", "Slack"), 36);
    assert!(records.iter().all(|r| r["OnlyInCarvedData"] == "true"));
    assert!(
        records
            .iter()
            .any(|r| r["Channel"] == "Application" && !r["EventId"].is_empty())
    );
    // The spool file is gone.
    assert!(!out.join(".carve-evtx-spool").exists());
}

#[test]
fn free_space_only_and_keep_live() {
    let dir = tempfile::tempdir().unwrap();
    let disk = sample_disk(dir.path());
    let disk = disk.to_str().unwrap();

    let free = dir.path().join("free");
    let result = run(&[
        "carve-evtx",
        "-f",
        disk,
        "--free-space-only",
        "-o",
        free.to_str().unwrap(),
    ]);
    assert_eq!(result.code, 0, "{}{}", result.stdout, result.stderr);
    let chunks = rows(&free.join("chunks.csv"));
    assert!(
        chunks
            .iter()
            .all(|r| r["Location"] == "FreeSpace" || r["Location"] == "DeletedLog")
    );
    assert_eq!(count(&chunks, "Status", "Written"), 4);

    let keep = dir.path().join("keep");
    let result = run(&[
        "carve-evtx",
        "-f",
        disk,
        "--keep-live",
        "-o",
        keep.to_str().unwrap(),
    ]);
    assert_eq!(result.code, 0, "{}{}", result.stdout, result.stderr);
    let chunks = rows(&keep.join("chunks.csv"));
    let live: Vec<_> = chunks
        .iter()
        .filter(|r| r["Location"] == "LiveLog")
        .collect();
    assert_eq!(live.len(), 2);
    assert!(live.iter().all(|r| r["Status"] == "Written"));
    // The older copy of the live Security chunk still adds nothing: its records are in the live
    // chunk, which is now written.
    assert_eq!(count(&chunks, "Status", "RecordsInLiveLogs"), 1);
    assert_eq!(count(&chunks, "Status", "Written"), 7);
}

/// `file` as NTFS stores it compressed on 4 KiB clusters: each 64 KiB unit LZNT1-compressed when
/// that saves a cluster, padded to its last cluster.
fn ntfs_compressed(file: &[u8]) -> Vec<u8> {
    let mut disk = Vec::new();
    for unit in file.chunks(0x10000) {
        let mut compressed = Vec::new();
        lznt1::compress(unit, &mut compressed);
        if compressed.len().next_multiple_of(0x1000) < unit.len() {
            disk.extend_from_slice(&compressed);
            disk.resize(disk.len().next_multiple_of(0x1000), 0);
        } else {
            disk.extend_from_slice(unit);
        }
    }
    disk
}

#[test]
fn plain_file_with_unaligned_compressed_and_truncated_chunks() {
    let dir = tempfile::tempdir().unwrap();
    let task = first_chunk(&task_log());
    let security = first_chunk(&sample_log("new-user-security.evtx"));
    let hello = first_chunk(&sample_log(
        "Microsoft-Windows-HelloForBusiness%4Operational.evtx",
    ));
    let folders = first_chunk(&sample_log("Microsoft-Windows-WorkFolders%4WHC.evtx"));

    let mut blob = vec![0x5a_u8; 1001];
    let task_at = blob.len();
    blob.extend_from_slice(&task);
    blob.extend_from_slice(&[0x11; 4999]);
    let security_at = blob.len();
    blob.extend_from_slice(&security);
    blob.extend_from_slice(&task); // the same chunk again
    blob.resize(blob.len().next_multiple_of(0x1000), 0x22);
    // A compressed log file: its header, then the WorkFolders chunk.
    let mut log = vec![0_u8; 0x1000];
    log.extend_from_slice(&folders);
    log.extend_from_slice(&[0x33; 0x1000]);
    let packed_at = blob.len();
    blob.extend_from_slice(&ntfs_compressed(&log));
    blob.extend_from_slice(&[0x44; 333]);
    // The input ends 30,000 bytes into a chunk (its records end before that).
    let truncated_at = blob.len();
    blob.extend_from_slice(&hello[..30_000]);
    let input = dir.path().join("pagefile.sys");
    fs::write(&input, &blob).unwrap();
    let input = input.to_str().unwrap();

    let out = dir.path().join("out");
    let result = run(&["carve-evtx", "-f", input, "-o", out.to_str().unwrap()]);
    assert_eq!(result.code, 0, "{}{}", result.stdout, result.stderr);
    assert!(result.stdout.contains("(file)"), "{}", result.stdout);
    let chunks = rows(&out.join("chunks.csv"));
    let at = |offset: usize| {
        chunks
            .iter()
            .find(|r| r["Offset"] == offset.to_string())
            .unwrap_or_else(|| panic!("no chunk at {offset}: {chunks:?}"))
    };
    assert_eq!(chunks.len(), 5);
    assert!(chunks.iter().all(|r| r["Location"] == "Unclassified"));
    assert_eq!(at(task_at)["Status"], "Written");
    assert_eq!(at(security_at + CHUNK)["Status"], "DuplicateChunk");
    assert_eq!(at(security_at)["Status"], "Written");
    let packed = at(packed_at + chunks_lznt1_offset(&blob[packed_at..]));
    assert_eq!(packed["Compressed"], "true");
    assert_eq!(packed["Channel"], "");
    assert_eq!(packed["SlackRecords"], "2");
    let truncated = at(truncated_at);
    assert_eq!(truncated["Truncated"], "true");
    assert_eq!(truncated["Records"], "5");
    assert_eq!(truncated["Status"], "Written");

    // With the live logs given, a chunk they hold is not written again.
    let live = dir.path().join("live");
    fs::create_dir(&live).unwrap();
    fs::write(
        live.join("Security.evtx"),
        sample_log("new-user-security.evtx"),
    )
    .unwrap();
    let out = dir.path().join("out-live");
    let result = run(&[
        "carve-evtx",
        "-f",
        input,
        "--live-logs",
        live.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(result.code, 0, "{}{}", result.stdout, result.stderr);
    assert!(result.stdout.contains("Live logs: 1"), "{}", result.stdout);
    let lines = fs::read_to_string(out.join("chunks.jsonl")).unwrap();
    let security_row: serde_json::Value = lines
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .find(|v| v["Offset"] == security_at as u64)
        .unwrap();
    assert_eq!(security_row["Status"], "DuplicateOfLiveLog");
    assert!(out.join("records.jsonl").is_file());
}

/// Where the chunk's LZNT1 block starts in compressed data that begins with a 4 KiB header block.
fn chunks_lznt1_offset(data: &[u8]) -> usize {
    let signature = data.windows(8).position(|w| w == b"ElfChnk\0").unwrap();
    signature - 3
}

#[test]
fn summary_only_and_input_errors() {
    let dir = tempfile::tempdir().unwrap();
    let disk = sample_disk(dir.path());
    let disk = disk.to_str().unwrap();
    let result = run(&["carve-evtx", "-f", disk]);
    assert_eq!(result.code, 0, "{}{}", result.stdout, result.stderr);
    assert!(
        result.stdout.contains("Chunks to keep: 5"),
        "{}",
        result.stdout
    );
    assert!(result.stdout.contains("add -o <DIR>"), "{}", result.stdout);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);

    let result = run(&["carve-evtx", "-f", "does-not-exist.raw"]);
    assert_eq!(result.code, 1);
    assert!(result.stderr.contains("does not exist") || result.stdout.contains("does not exist"));

    // Results go to a new or empty directory.
    let used = dir.path().join("used");
    fs::create_dir(&used).unwrap();
    fs::write(used.join("keep.txt"), b"x").unwrap();
    let result = run(&["carve-evtx", "-f", disk, "-o", used.to_str().unwrap()]);
    assert_eq!(result.code, 1);
    assert_eq!(fs::read_dir(&used).unwrap().count(), 1);

    // Free space needs a file system.
    let plain = dir.path().join("plain.bin");
    fs::write(&plain, vec![0_u8; 100_000]).unwrap();
    let result = run(&[
        "carve-evtx",
        "-f",
        plain.to_str().unwrap(),
        "--free-space-only",
    ]);
    assert_eq!(result.code, 1);
}

#[test]
fn e01_and_vhdx_images_skip_their_live_log() {
    let dir = tempfile::tempdir().unwrap();
    let mut vhdx = Vec::new();
    flate2::read::GzDecoder::new(
        fs::File::open(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/vhdx/sample.vhdx.gz"),
        )
        .unwrap(),
    )
    .read_to_end(&mut vhdx)
    .unwrap();
    let vhdx_path = dir.path().join("sample.vhdx");
    fs::write(&vhdx_path, vhdx).unwrap();
    let e01 = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/e01/sample.E01");
    for (image, kind) in [
        (vhdx_path.as_path(), "VHDX disk image"),
        (e01.as_path(), "E01 disk image"),
    ] {
        let image = image.to_str().unwrap();
        // The TaskScheduler log on the image is live: its chunk is found and left out.
        let result = run(&["carve-evtx", "-f", image]);
        assert_eq!(result.code, 0, "{}{}", result.stdout, result.stderr);
        assert!(result.stdout.contains(kind), "{}", result.stdout);
        assert!(result.stdout.contains("Live logs: 1"), "{}", result.stdout);
        assert!(
            result.stdout.contains("SkippedLiveLog: 1"),
            "{}",
            result.stdout
        );
        // Read as plain bytes, the container is still decoded, but nothing tells the log is live.
        let result = run(&["carve-evtx", "-f", image, "--no-file-system"]);
        assert_eq!(result.code, 0, "{}{}", result.stdout, result.stderr);
        assert!(
            result.stdout.contains("Unclassified: 1"),
            "{}",
            result.stdout
        );
        assert!(
            result.stdout.contains("Chunks to keep: 1"),
            "{}",
            result.stdout
        );
    }
}

#[test]
fn a_bad_input_leaves_no_results_directory() {
    let dir = tempfile::tempdir().unwrap();
    let plain = dir.path().join("plain.bin");
    fs::write(&plain, vec![0_u8; 100_000]).unwrap();
    let out = dir.path().join("out");
    let result = run(&[
        "carve-evtx",
        "-f",
        plain.to_str().unwrap(),
        "--free-space-only",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(result.code, 1);
    assert!(!out.exists());
}
