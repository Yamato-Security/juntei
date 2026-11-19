// Copyright (C) 2026 Shirofune Security Co., Ltd.
// SPDX-License-Identifier: RPL-1.5
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

//! `parse-ese windows-search` against the public indexes in `test_files/windows_search/` (its
//! README names the sources and the independent readings every value below was checked against).

use flate2::read::GzDecoder;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    /// Messages go to stdout, unless the records do.
    fn all(&self) -> String {
        plain(&format!("{}{}", self.stdout, self.stderr))
    }
}

/// The text without its terminal colors.
fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn run(args: &[&str]) -> Run {
    let out = Command::new(env!("CARGO_BIN_EXE_juntei"))
        .args(args)
        .output()
        .expect("juntei runs");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    }
}

const SEARCH_DIR: &str = "C/ProgramData/Microsoft/Search/Data/Applications/Windows";

/// `test_files/windows_search/<name>.gz`, decompressed to `directory`/`target`.
fn unpack(name: &str, directory: &Path, target: &str) -> PathBuf {
    std::fs::create_dir_all(directory).unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files/windows_search")
        .join(format!("{name}.gz"));
    let mut data = Vec::new();
    GzDecoder::new(std::fs::File::open(source).unwrap())
        .read_to_end(&mut data)
        .unwrap();
    let path = directory.join(target);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, data).unwrap();
    path
}

fn sha256(path: &Path) -> String {
    let digest = Sha256::digest(std::fs::read(path).unwrap());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn csv_rows(path: &Path) -> (Vec<String>, Vec<BTreeMap<String, String>>) {
    let mut reader = csv::Reader::from_path(path).unwrap();
    let header: Vec<String> = reader.headers().unwrap().iter().map(String::from).collect();
    let rows = reader
        .records()
        .map(|r| {
            let r = r.unwrap();
            header
                .iter()
                .cloned()
                .zip(r.iter().map(String::from))
                .collect()
        })
        .collect();
    (header, rows)
}

fn jsonl(text: &str) -> Vec<Value> {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn row<'a>(rows: &'a [BTreeMap<String, String>], id: &str) -> &'a BTreeMap<String, String> {
    rows.iter()
        .find(|r| r["DocumentID"] == id)
        .unwrap_or_else(|| panic!("no item {id}"))
}

fn arg(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[test]
fn windows_7_index_items_match_libesedb_including_its_string_encoding() {
    let temp = tempfile::tempdir().unwrap();
    let edb = unpack("plaso/Windows.edb", temp.path(), "Windows.edb");
    let out = temp.path().join("items.csv");
    let r = run(&[
        "parse-ese",
        "windows-search",
        "-f",
        arg(&edb),
        "-o",
        arg(&out),
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    for line in [
        "Databases read: 1 (1 Windows 7 (SystemIndex_0A))",
        "Dirty shutdown (transaction logs not replayed): 1",
        "Indexed items: 339",
        "In both tables / gatherer only / property store only: 322 / 17 / 0",
        "Folders: 77",
        "Stores: csc 2, file 291, iehistory 29",
        "Items without a stored time: 17",
    ] {
        assert!(r.all().contains(line), "{line}\n{}", r.all());
    }
    let (header, rows) = csv_rows(&out);
    assert_eq!(header[0], "DocumentID");
    assert_eq!(header.last().unwrap(), "Problems");
    assert_eq!(rows.len(), 339);
    // esedbexport's SystemIndex_0A row 3 and SystemIndex_Gthr row 3
    let item = row(&rows, "3");
    let expected = [
        ("InTables", "Gthr+PropertyStore"),
        (
            "ItemPath",
            r"C:\ProgramData\Microsoft\Windows\Start Menu\Default Programs.lnk",
        ),
        (
            "ItemURL",
            "file:C:/ProgramData/Microsoft/Windows/Start Menu/Default Programs.lnk",
        ),
        (
            "GatherPath",
            "file:C:/ProgramData/Microsoft/Windows/Start Menu/Default Programs.lnk",
        ),
        ("Name", "Default Programs.lnk"),
        ("ItemType", ".lnk"),
        ("Kind", "link|program"),
        ("Store", "file"),
        ("IsFolder", "false"),
        ("Size", "1282"),
        ("FileAttributes", "32"),
        ("FileOwner", r"NT AUTHORITY\SYSTEM"),
        ("DateCreated", "2009-07-14T05:01:14.046440500Z"),
        ("DateModified", "2009-07-14T05:01:14.046440500Z"),
        ("DateAccessed", "2009-07-14T05:01:14.046440500Z"),
        ("ItemDate", "2009-07-14T05:01:16Z"),
        ("GatherTime", "2015-08-24T11:23:40.594705200Z"),
        ("GatherLastModified", "2015-08-24T11:23:25.047831Z"),
        ("CompName", "STUDENT-PC1"),
        ("User", ""),
        ("UserSource", ""),
        ("ScopeID", "8"),
        ("SDID", "97"),
        ("CrawlNumber", "1"),
        ("GatherFlags", "373"),
        ("SourceFile", "Windows.edb"),
        ("Problems", ""),
    ];
    for (key, value) in expected {
        assert_eq!(item[key], value, "{key}");
    }
    // The 8-bit summary encoding (type 1) and the run-length UTF-16 one (type 0, with U+2019)
    assert!(
        item["AutoSummary"].starts_with(
            "Choose which programs you want Windows to use for activities like web browsing"
        ),
        "{}",
        item["AutoSummary"]
    );
    let document = row(&rows, "229");
    assert_eq!(
        document["AutoSummary"],
        "A chicken is a type of bird. It’s tasty."
    );
    assert_eq!(
        document["ItemPath"],
        r"C:\Users\bperry\My Documents\Poultry Studies\Introduction to Chicken.docx"
    );
    assert_eq!(document["User"], "bperry");
    assert_eq!(document["UserSource"], "ItemPath");
    assert_eq!(document["FileOwner"], r"INTERNAL\bperry");
    assert_eq!(document["Size"], "12530");
    assert_eq!(document["DateCreated"], "2015-08-25T10:58:40.676386800Z");
    assert_eq!(document["DateAccessed"], "2015-08-25T10:58:41.363886800Z");
    assert_eq!(document["GatherTime"], "2015-08-25T10:59:23.988886800Z");
    assert_eq!(
        rows.iter().filter(|r| !r["AutoSummary"].is_empty()).count(),
        81
    );
    // Every property-store row has its summary decoded without a note
    assert!(
        rows.iter().all(|r| !r["Problems"].contains("kept as hex")),
        "{:?}",
        rows.iter()
            .filter(|r| r["Problems"].contains("kept as hex"))
            .map(|r| &r["Problems"])
            .collect::<Vec<_>>()
    );
    // Browser history items the index also holds
    let history: Vec<_> = rows.iter().filter(|r| r["Store"] == "iehistory").collect();
    assert_eq!(history.len(), 29);
    assert!(
        history
            .iter()
            .all(|r| r["ItemURL"].starts_with("iehistory://"))
    );
    // Items the gatherer crawled without a property-store row keep their gatherer path
    let gathered: Vec<_> = rows.iter().filter(|r| r["InTables"] == "Gthr").collect();
    assert_eq!(gathered.len(), 17);
    let page = row(&rows, "256");
    assert_eq!(
        page["GatherPath"],
        "iehistory://{S-1-5-21-226059406-2984137831-1201299043-1107}/http://webmail.student.greendale.xyz/src/redirect.php"
    );
    assert_eq!(page["Name"], "redirect.php");
    assert_eq!(page["ItemPath"], "");
    assert_eq!(page["PropertyCount"], "0");
    // No profile file is a user
    assert!(rows.iter().all(|r| r["User"] != "desktop.ini"));
    // The index writes one tick for the times of an item that has none: a number, not 1601
    let root = row(&rows, "149");
    assert_eq!(root["DateModified"], "");
    assert_eq!(root["DateCreated"], "");
    assert!(
        root["Problems"].contains("System_DateModified: 1 FILETIME ticks is not a time"),
        "{}",
        root["Problems"]
    );
    assert_eq!(rows.iter().filter(|r| !r["Problems"].is_empty()).count(), 2);
    // Gatherer fields ESE marks NULL stay empty. The record bytes still hold the priority of the
    // last crawl (45 for item 3), which readers that skip the fixed-column NULL bitmap print.
    assert_eq!(item["Priority"], "");
    assert_eq!(
        rows.iter().filter(|r| !r["Priority"].is_empty()).count(),
        18
    );
    assert!(
        rows.iter()
            .filter(|r| !r["Priority"].is_empty())
            .all(|r| r["Priority"].parse::<u8>().is_ok())
    );
}

#[test]
fn dirty_windows_10_index_reads_every_row_libesedb_and_sidr_read() {
    let temp = tempfile::tempdir().unwrap();
    let edb = unpack("museum/Win10-Windows.edb", temp.path(), "Windows.edb");
    let out = temp.path().join("items.csv");
    let r = run(&[
        "parse-ese",
        "windows-search",
        "-f",
        arg(&edb),
        "-o",
        arg(&out),
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    for line in [
        "Databases read: 1 (1 Windows 8 to 11 (SystemIndex_PropertyStore))",
        "Dirty shutdown (transaction logs not replayed): 1",
        "Indexed items: 935",
        "In both tables / gatherer only / property store only: 930 / 5 / 0",
        "Folders: 505",
        "Stores: file 929, winrt 1",
        "a leaf page no branch page reaches",
        "8 leaf pages were read through the sibling chain",
    ] {
        assert!(r.all().contains(line), "{line}\n{}", r.all());
    }
    let (_, rows) = csv_rows(&out);
    assert_eq!(rows.len(), 935);
    // WorkID 701 sits in the first leaf page only the sibling chain reaches: SIDR's values
    let item = row(&rows, "701");
    let expected = [
        (
            "ItemPath",
            r"C:\Users\TestUser\Desktop\Targets\Antivirus\VIPRE.tkape",
        ),
        (
            "GatherPath",
            "file:C:/[3b9ead37-f2cd-4bda-ae00-1242c51c4248]/Users/TestUser/Desktop/Targets/Antivirus/VIPRE.tkape",
        ),
        ("ItemType", ".tkape"),
        ("Size", "845"),
        ("DateModified", "2022-03-09T13:48:18Z"),
        ("DateCreated", "2022-03-10T21:40:41.753962800Z"),
        ("DateAccessed", "2022-03-10T21:40:41.753962800Z"),
        ("GatherTime", "2022-03-10T21:40:42.553567300Z"),
        ("User", "TestUser"),
        ("UserSource", "ItemPath"),
    ];
    for (key, value) in expected {
        assert_eq!(item[key], value, "{key}");
    }
    let last = row(&rows, "935");
    assert_eq!(
        last["ItemPath"],
        r"C:\Users\TestUser\Desktop\Targets\Windows\WSA\WindowsSubsystemforAndroid.tkape"
    );
    assert_eq!(last["Size"], "2101");
    assert_eq!(last["GatherTime"], "2022-03-10T21:40:42.732586700Z");
    assert_eq!(rows.iter().filter(|r| r["InTables"] == "Gthr").count(), 5);
    assert!(rows.iter().all(|r| r["Problems"].is_empty()));
}

#[test]
fn windows_11_revision_200_index_is_read_with_its_computer_name() {
    let temp = tempfile::tempdir().unwrap();
    let edb = unpack("museum/Win11-Windows.edb", temp.path(), "Windows.edb");
    let out = temp.path().join("items.jsonl");
    let r = run(&[
        "parse-ese",
        "windows-search",
        "-f",
        arg(&edb),
        "-o",
        arg(&out),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    assert!(r.all().contains("Indexed items: 855"), "{}", r.all());
    assert!(
        r.all()
            .contains("In both tables / gatherer only / property store only: 825 / 27 / 3"),
        "{}",
        r.all()
    );
    let records = jsonl(&std::fs::read_to_string(&out).unwrap());
    assert_eq!(records.len(), 855);
    // esedbexport's SystemIndex_PropertyStore row 72 (little-endian FILETIMEs and size)
    let item = records
        .iter()
        .find(|r| r["DocumentID"] == 72)
        .expect("WorkID 72");
    assert_eq!(
        item["ItemPath"],
        r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs\Windows System\Task Manager.lnk"
    );
    assert_eq!(item["Size"], 1120);
    assert_eq!(item["DateModified"], "2021-06-05T12:05:21.968641600Z");
    assert_eq!(item["DateCreated"], "2021-06-05T12:05:21.968641600Z");
    assert_eq!(item["GatherTime"], "2022-02-05T21:03:53.107481Z");
    assert_eq!(item["FileOwner"], r"NT AUTHORITY\SYSTEM");
    assert_eq!(item["CompName"], "GOLDPAPERW11");
    assert_eq!(item["ItemType"], ".lnk");
    // Every property is kept with its name, ID and type; times carry their ticks
    let properties = item["Properties"].as_array().unwrap();
    assert_eq!(item["PropertyCount"], properties.len());
    let modified = properties
        .iter()
        .find(|p| p["Name"] == "System_DateModified")
        .unwrap();
    assert_eq!(modified["ID"], 15);
    assert_eq!(modified["Type"], "Binary");
    assert_eq!(modified["Time"], "2021-06-05T12:05:21.968641600Z");
    assert_eq!(modified["Ticks"], 0x01D75A030F5A2410u64);
    let path = properties
        .iter()
        .find(|p| p["Name"] == "System_ItemPathDisplay")
        .unwrap();
    assert_eq!(path["ID"], 4447);
    assert_eq!(path["Type"], "LongText");
    assert_eq!(item["Gather"]["Filename"], "Task Manager.lnk");
    // Items in the property store that the gatherer has not recorded
    let only: Vec<_> = records
        .iter()
        .filter(|r| r["InTables"] == "PropertyStore")
        .collect();
    assert_eq!(only.len(), 3);
    assert!(only.iter().all(|r| r["Gather"].is_null()));
}

#[test]
fn csv_json_and_jsonl_hold_the_same_records_and_leave_the_sources_unchanged() {
    let temp = tempfile::tempdir().unwrap();
    let sources = [
        ("plaso/Windows.edb", "w7/Windows.edb"),
        ("museum/Win10-Windows.edb", "w10/Windows.edb"),
        ("museum/Win11-Windows.edb", "w11/Windows.edb"),
    ]
    .map(|(name, target)| unpack(name, temp.path(), target));
    let before: Vec<_> = sources.iter().map(|p| sha256(p)).collect();
    let csv = temp.path().join("items.csv");
    let json = temp.path().join("items.json");
    let jsonl_path = temp.path().join("items.jsonl");
    for (path, kind) in [(&csv, "csv"), (&json, "json"), (&jsonl_path, "jsonl")] {
        let r = run(&[
            "parse-ese",
            "windows-search",
            "-d",
            arg(temp.path()),
            "-o",
            arg(path),
            "-t",
            kind,
        ]);
        assert_eq!(r.code, 0, "{kind}: {}", r.all());
        assert!(
            r.all().contains("Indexed items: 2,129"),
            "{kind}: {}",
            r.all()
        );
    }
    let (header, rows) = csv_rows(&csv);
    assert_eq!(rows.len(), 339 + 935 + 855);
    let lines = jsonl(&std::fs::read_to_string(&jsonl_path).unwrap());
    let pretty: Vec<Value> =
        serde_json::Deserializer::from_str(&std::fs::read_to_string(&json).unwrap())
            .into_iter::<Value>()
            .map(Result::unwrap)
            .collect();
    assert_eq!(lines.len(), rows.len());
    assert_eq!(pretty, lines);
    for (csv_row, record) in rows.iter().zip(&lines) {
        let object = record.as_object().unwrap();
        // The JSON keys: the CSV's columns (every property in `Properties` instead of
        // `OtherProperties`) plus `Properties` and `Gather`.
        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();
        let mut expected: Vec<&str> = header
            .iter()
            .map(String::as_str)
            .filter(|k| *k != "OtherProperties")
            .chain(["Properties", "Gather"])
            .collect();
        expected.sort_unstable();
        assert_eq!(keys, expected);
        for (key, cell) in csv_row {
            if key == "OtherProperties" {
                continue;
            }
            let value = &object[key];
            let text = match value {
                Value::Null => String::new(),
                Value::String(s) => s.clone(),
                Value::Array(items) => items
                    .iter()
                    .map(|i| i.as_str().map_or_else(|| i.to_string(), str::to_string))
                    .collect::<Vec<_>>()
                    .join("|"),
                other => other.to_string(),
            };
            assert_eq!(cell, &text, "{key} of item {}", csv_row["DocumentID"]);
        }
        // Relative provenance under the input directory
        assert!(
            ["w7/Windows.edb", "w10/Windows.edb", "w11/Windows.edb"]
                .contains(&csv_row["SourceFile"].as_str()),
            "{}",
            csv_row["SourceFile"]
        );
    }
    let after: Vec<_> = sources.iter().map(|p| sha256(p)).collect();
    assert_eq!(before, after);
    assert_eq!(
        before[0],
        "e96837a0ad37be5a55222fa98991bec6b5c5ba956610f9d241aa8c65f2e0714c"
    );
    assert_eq!(
        before[1],
        "ff024095f2880817e64e0413d8b5760965bd8e5247ad163045426354aa589b5b"
    );
    assert_eq!(
        before[2],
        "35ca37a022869e9311defe4211253948b68f55599f79258c16390ee4aa14dbf9"
    );
    // JSON on stdout holds nothing but the records
    let r = run(&[
        "parse-ese",
        "windows-search",
        "-f",
        arg(&sources[0]),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0);
    assert_eq!(jsonl(&r.stdout).len(), 339);
    assert!(r.stderr.contains("dirty shutdown"), "{}", r.stderr);
}

#[test]
fn newer_sqlite_indexes_damaged_and_other_files_are_reported_without_losing_the_rest() {
    let temp = tempfile::tempdir().unwrap();
    let newer = temp.path().join("newer");
    unpack("museum/Win11-22H2-Windows.db", &newer, "Windows.db");
    unpack(
        "museum/Win11-22H2-Windows-gather.db",
        &newer,
        "Windows-gather.db",
    );
    // Only the SQLite index of Windows 11 22H2: found, named, not read
    let r = run(&["parse-ese", "windows-search", "-d", arg(&newer)]);
    assert_eq!(r.code, 1, "{}", r.all());
    assert!(
        r.all().contains("Windows Search databases: 2"),
        "{}",
        r.all()
    );
    assert!(
        r.all().contains("Unsupported newer format, not read: Windows.db is the SQLite index of Windows 11 22H2 or later"),
        "{}",
        r.all()
    );
    assert!(
        r.all().contains("Windows-gather.db is the SQLite index"),
        "{}",
        r.all()
    );
    assert!(
        r.all()
            .contains("Unsupported newer format (SQLite, Windows 11 22H2+): 2"),
        "{}",
        r.all()
    );
    assert!(r.all().contains("Indexed items: 0"), "{}", r.all());
    // Beside readable indexes: a truncated copy, an empty file, a text file and the SQLite pair
    let mixed = temp.path().join("mixed");
    let good = unpack("plaso/Windows.edb", &mixed.join("a"), "Windows.edb");
    let data = std::fs::read(&good).unwrap();
    std::fs::create_dir_all(mixed.join("cut")).unwrap();
    std::fs::write(mixed.join("cut/Windows.edb"), &data[..6 * 1024 * 1024]).unwrap();
    std::fs::create_dir_all(mixed.join("empty")).unwrap();
    std::fs::write(mixed.join("empty/Windows.edb"), b"").unwrap();
    std::fs::create_dir_all(mixed.join("text")).unwrap();
    std::fs::write(mixed.join("text/Windows.edb"), b"not a database at all\n").unwrap();
    std::fs::create_dir_all(mixed.join("srum")).unwrap();
    let mut srum = Vec::new();
    GzDecoder::new(
        std::fs::File::open(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("test_files/webcache/plaso/WebCacheV01.dat.gz"),
        )
        .unwrap(),
    )
    .read_to_end(&mut srum)
    .unwrap();
    std::fs::write(mixed.join("srum/Windows.edb"), &srum).unwrap();
    unpack(
        "museum/Win11-22H2-Windows.db",
        &mixed.join("new"),
        "Windows.db",
    );
    let out = temp.path().join("mixed.csv");
    let r = run(&[
        "parse-ese",
        "windows-search",
        "-d",
        arg(&mixed),
        "-o",
        arg(&out),
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    assert!(
        r.all().contains("Windows Search databases: 6"),
        "{}",
        r.all()
    );
    assert!(
        r.all()
            .contains("Not read as a Windows Search index: empty/Windows.edb: the file is empty"),
        "{}",
        r.all()
    );
    assert!(
        r.all()
            .contains("Not read as a Windows Search index: text/Windows.edb: not an ESE database"),
        "{}",
        r.all()
    );
    assert!(
        r.all().contains(
            "Not read as a Windows Search index: srum/Windows.edb: no Windows Search property store"
        ),
        "{}",
        r.all()
    );
    assert!(
        r.all().contains("new/Windows.db is the SQLite index"),
        "{}",
        r.all()
    );
    assert!(
        r.all().contains("Not Windows Search indexes: 3"),
        "{}",
        r.all()
    );
    assert!(
        r.all()
            .contains("Unsupported newer format (SQLite, Windows 11 22H2+): 1"),
        "{}",
        r.all()
    );
    let (_, rows) = csv_rows(&out);
    let whole = rows
        .iter()
        .filter(|r| r["SourceFile"] == "a/Windows.edb")
        .count();
    assert_eq!(whole, 339);
    // The cut copy: whatever its first pages hold is read, and what is missing is said
    let cut = rows
        .iter()
        .filter(|r| r["SourceFile"] == "cut/Windows.edb")
        .count();
    assert!(cut <= 339, "{cut}");
    assert!(
        r.all().contains("cut/Windows.edb: ") && r.all().contains("Database problems: "),
        "{}",
        r.all()
    );
    assert!(
        r.all()
            .contains("Databases read: 2 (2 Windows 7 (SystemIndex_0A))"),
        "{}",
        r.all()
    );
}

#[test]
fn sqlite_usn_companion_is_reported_instead_of_silently_ignored() {
    let temp = tempfile::tempdir().unwrap();
    // Discovery and SQLite-signature handling, not a fixture for SQLite parsing.
    std::fs::write(temp.path().join("Windows-usn.db"), b"SQLite format 3\0").unwrap();
    let r = run(&["parse-ese", "windows-search", "-d", arg(temp.path())]);
    assert_eq!(r.code, 1, "{}", r.all());
    assert!(
        r.all().contains("Windows-usn.db is the SQLite index"),
        "{}",
        r.all()
    );
    assert!(
        r.all()
            .contains("Unsupported newer format (SQLite, Windows 11 22H2+): 1"),
        "{}",
        r.all()
    );
}

#[test]
fn input_and_output_errors() {
    let temp = tempfile::tempdir().unwrap();
    let edb = unpack("plaso/Windows.edb", temp.path(), "Windows.edb");
    let r = run(&[
        "parse-ese",
        "windows-search",
        "-f",
        arg(&temp.path().join("absent.edb")),
    ]);
    assert_eq!(r.code, 1);
    assert!(r.all().contains("The file does not exist"), "{}", r.all());
    let r = run(&[
        "parse-ese",
        "windows-search",
        "-d",
        arg(&temp.path().join("absent")),
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.all().contains("The directory does not exist"),
        "{}",
        r.all()
    );
    let empty = temp.path().join("nothing");
    std::fs::create_dir_all(&empty).unwrap();
    let r = run(&["parse-ese", "windows-search", "-d", arg(&empty)]);
    assert_eq!(r.code, 1);
    assert!(
        r.all()
            .contains("No Windows Search databases (Windows.edb, Windows.db) were found in"),
        "{}",
        r.all()
    );
    // CSV needs a file; the summary alone says so
    let r = run(&["parse-ese", "windows-search", "-f", arg(&edb)]);
    assert_eq!(r.code, 0, "{}", r.all());
    assert!(
        r.all().contains("To save every record, add -o <FILE>"),
        "{}",
        r.all()
    );
    // No overwrite without -C, never over an input
    let out = temp.path().join("items.csv");
    std::fs::write(&out, "keep").unwrap();
    let r = run(&[
        "parse-ese",
        "windows-search",
        "-f",
        arg(&edb),
        "-o",
        arg(&out),
    ]);
    assert_eq!(r.code, 1, "{}", r.all());
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "keep");
    let r = run(&[
        "parse-ese",
        "windows-search",
        "-f",
        arg(&edb),
        "-o",
        arg(&edb),
        "-C",
    ]);
    assert_eq!(r.code, 1, "{}", r.all());
    assert_eq!(
        sha256(&edb),
        "e96837a0ad37be5a55222fa98991bec6b5c5ba956610f9d241aa8c65f2e0714c"
    );
    let r = run(&["parse-ese", "windows-search", "-f", arg(&edb), "-C"]);
    assert_eq!(r.code, 2, "{}", r.all());
    assert!(r.all().contains("--output"), "{}", r.all());
}

#[test]
fn help_names_the_command_and_its_options() {
    let r = run(&["parse-ese", "--help"]);
    assert!(r.all().contains("windows-search"), "{}", r.stdout);
    let r = run(&["parse-ese", "windows-search", "--help"]);
    assert_eq!(r.code, 0);
    for text in [
        "juntei parse-ese windows-search <INPUT> [OPTIONS]",
        "-d, --directory <DIR>",
        "-f, --file <FILE>",
        "-o, --output <FILE>",
        "-t, --output-type <TYPE>",
        "-C, --clobber",
        "Windows.db",
    ] {
        assert!(r.all().contains(text), "{text}\n{}", r.stdout);
    }
}

#[test]
fn auto_parse_saves_the_index_under_file_existence_and_the_timelines() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("evidence");
    let host = root.join("HOST1");
    let search = host.join(SEARCH_DIR);
    unpack("plaso/Windows.edb", &search, "Windows.edb");
    for name in [
        "Windows.jfm",
        "MSS.log",
        "MSS00012.log",
        "MSS.chk",
        "MSSres00001.jrs",
    ] {
        std::fs::write(search.join(name), b"companion, not a database").unwrap();
    }
    std::fs::create_dir_all(host.join("C/Windows/System32")).unwrap();
    // A second host without an index: the task is skipped there
    std::fs::create_dir_all(root.join("HOST2/C/Windows/System32")).unwrap();
    let output = temp.path().join("out");
    let combined = temp.path().join("all.jsonl");
    let r = run(&[
        "auto-parse",
        "-d",
        arg(&root),
        "-o",
        arg(&output),
        "-t",
        "csv",
        "-j",
        arg(&combined),
        "-Z",
        "+09:00",
        "-T",
        "2",
        "-q",
    ]);
    assert!(r.code == 0 || r.code == 1, "{}", r.all());
    let manifest: Value =
        serde_json::from_str(&std::fs::read_to_string(output.join("manifest.json")).unwrap())
            .unwrap();
    let tasks: Vec<&Value> = manifest["Tasks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|task| task["Artifact"] == "windows-search" && task["Purpose"] == "artifact")
        .collect();
    assert_eq!(tasks.len(), 2, "{tasks:?}");
    for task in &tasks {
        assert_eq!(task["Category"], "FileExistence");
        let command: Vec<&str> = task["Command"]
            .as_array()
            .unwrap()
            .iter()
            .map(|arg| arg.as_str().unwrap())
            .collect();
        assert_eq!(command[1..4], ["parse-ese", "windows-search", "-d"]);
        let host = task["Comp"].as_str().unwrap();
        assert_eq!(Path::new(command[4]), Path::new(".").join(host).join("C"));
        if host == "HOST2" {
            assert_eq!(task["Status"], "skipped", "{task}");
        } else {
            assert_eq!(task["Status"], "succeeded", "{task}");
            assert!(
                task["Output"]
                    .as_str()
                    .unwrap()
                    .replace('\\', "/")
                    .ends_with("HOST1/FileExistence/WindowsSearch.csv"),
                "{}",
                task["Output"]
            );
        }
    }
    let (_, rows) = csv_rows(&output.join("HOST1/FileExistence/WindowsSearch.csv"));
    assert_eq!(rows.len(), 339);
    let item = row(&rows, "3");
    // Provenance relative to the parser's input directory (the host's volume), never absolute
    assert_eq!(
        item["SourceFile"],
        "ProgramData/Microsoft/Search/Data/Applications/Windows/Windows.edb"
    );
    assert_eq!(item["DateModified"], "2009-07-14T14:01:14.046440500+09:00");
    assert_eq!(item["GatherTime"], "2015-08-24T20:23:40.594705200+09:00");
    assert!(
        !std::fs::read_to_string(output.join("HOST1/FileExistence/WindowsSearch.csv"))
            .unwrap()
            .contains(arg(&root)),
        "an absolute path leaked"
    );
    // The combined timeline carries the records with the host and artifact
    let all = jsonl(&std::fs::read_to_string(&combined).unwrap());
    let items: Vec<_> = all
        .iter()
        .filter(|r| r["Artifact"] == "windows-search")
        .collect();
    assert_eq!(items.len(), 339);
    assert!(items.iter().all(|r| r["Comp"] == "HOST1"));
    assert!(items.iter().all(|r| r["Properties"].is_array()));
    // super-timeline from the native output: one event per stored time
    let timeline = temp.path().join("timeline.jsonl");
    let r = run(&[
        "super-timeline",
        "-d",
        arg(&output),
        "--artifact",
        "windows-search",
        "-o",
        arg(&timeline),
        "-t",
        "jsonl",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    let events = jsonl(&std::fs::read_to_string(&timeline).unwrap());
    let mut types: BTreeMap<&str, usize> = BTreeMap::new();
    for event in &events {
        assert_eq!(event["Artifact"], "windows-search");
        assert_eq!(event["Comp"], "HOST1");
        assert!(event["Path"].is_string(), "{event}");
        assert_eq!(
            event["SourceFile"],
            "ProgramData/Microsoft/Search/Data/Applications/Windows/Windows.edb"
        );
        *types
            .entry(event["TimestampType"].as_str().unwrap())
            .or_default() += 1;
    }
    let expected = [
        ("Item indexed by Windows Search", 322),
        // 322 property-store rows less the two whose times are one tick, and the two folders
        // the property store holds no time for
        (
            "Indexed item last modified (file system time recorded by Windows Search)",
            318,
        ),
        (
            "Indexed item created (file system time recorded by Windows Search)",
            291,
        ),
        (
            "Indexed item last accessed (file system time recorded by Windows Search)",
            291,
        ),
        // only for an item the property store has no modification time for
        ("Indexed item last modified (gatherer record)", 3),
    ];
    for (label, count) in expected {
        assert_eq!(
            types.get(label).copied().unwrap_or(0),
            count,
            "{label}\n{types:?}"
        );
    }
    assert_eq!(events.len(), 322 + 318 + 291 + 291 + 3, "{types:?}");
    let first = events
        .iter()
        .find(|e| e["SourceField"] == "DateModified" && e["Details"]["InputRecord"] == 1)
        .expect("the first item's modification event");
    assert_eq!(first["Timestamp"], "2009-07-14T05:01:14.046440500Z");
    assert_eq!(first["Details"]["TimeRole"], "file_metadata");
    assert_eq!(first["Details"]["Store"], "file");
    assert_eq!(first["Details"]["IsFolder"], "true");
    assert_eq!(
        first["Details"]["ItemURL"],
        "file:C:/ProgramData/Microsoft/Windows/Start Menu"
    );
}
