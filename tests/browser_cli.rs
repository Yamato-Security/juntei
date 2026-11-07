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

//! `parse-sqlite browser` and its `auto-parse` task, on the databases of test_files/browser.

use flate2::read::GzDecoder;
use serde_json::Value;
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
    /// Messages go to stdout, unless the rows do.
    fn all(&self) -> String {
        format!("{}{}", self.stdout, self.stderr)
    }
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

/// The databases of test_files/browser/`set`, decompressed into `directory`.
fn unpack(set: &str, directory: &Path) {
    std::fs::create_dir_all(directory).unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files/browser")
        .join(set);
    for entry in std::fs::read_dir(source).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let Some(name) = name.strip_suffix(".gz") else {
            continue;
        };
        let mut data = Vec::new();
        GzDecoder::new(std::fs::File::open(entry.path()).unwrap())
            .read_to_end(&mut data)
            .unwrap();
        std::fs::write(directory.join(name), data).unwrap();
    }
}

const EDGE: &str = "C/Users/alice/AppData/Local/Microsoft/Edge/User Data/Default";
const FIREFOX: &str = "C/Users/alice/AppData/Roaming/Mozilla/Firefox/Profiles/x1y2.default-release";

/// A collection with the Chromium 150 databases in an Edge profile (the cookies in `Network`,
/// as on Windows), the Firefox 140 profile, and a `History` file of another program.
fn collection(root: &Path) -> PathBuf {
    let host = root.join("HOST1");
    let edge = host.join(EDGE);
    unpack("chromium150", &edge);
    std::fs::create_dir_all(edge.join("Network")).unwrap();
    std::fs::rename(edge.join("Cookies"), edge.join("Network/Cookies")).unwrap();
    unpack("firefox140", &host.join(FIREFOX));
    let other = host.join("C/ProgramData/Other");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join("History"), b"not a database").unwrap();
    std::fs::create_dir_all(host.join("C/Windows/System32")).unwrap();
    host
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

#[test]
fn browser_summary_on_the_terminal() {
    let dir = tempfile::tempdir().unwrap();
    let host = collection(dir.path());
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-d",
        host.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}\n{}", r.stdout, r.stderr);
    for line in [
        "Browser database files: 7\n",
        "Browser databases parsed: 6\n  History: 2\n  Cookies: 2\n  Form history: 2\n",
        "Files skipped (not browser databases): 1\n",
        "Read with their write-ahead log (-wal): 2\n",
        "Profiles: 2\n  Edge: Default (user alice)\n  Firefox: x1y2.default-release (user alice)\n",
        "Visits: 26 (deleted, recovered: 2)\n",
        "Deleted pages recovered: 2\n",
        "Downloads: 2\n",
        "Cookies: 16\n",
        "Form entries: 6\n",
        "Visit times: 2026-11-07T12:32:09.882399Z - 2026-11-07T12:37:09.265272Z\n",
        "Most visited hosts (live visits):\n        23  127.0.0.1:8765\n         3  localhost:8765\n",
        "Problems: 0\n",
        "To save every row, add -o <DIR>",
    ] {
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    }
}

#[test]
fn browser_csv_tables() {
    let dir = tempfile::tempdir().unwrap();
    let host = collection(dir.path());
    let out = dir.path().join("out");
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-d",
        host.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "{}\n{}", r.stdout, r.stderr);
    assert!(r.stdout.contains("(4 CSV files)"), "{}", r.stdout);
    let mut files: Vec<String> = std::fs::read_dir(&out)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    files.sort();
    assert_eq!(
        files,
        [
            "Browser-Cookies.csv",
            "Browser-Downloads.csv",
            "Browser-FormHistory.csv",
            "Browser-History.csv"
        ]
    );

    let (header, history) = csv_rows(&out.join("Browser-History.csv"));
    assert_eq!(
        header,
        [
            "SourceFile",
            "User",
            "Browser",
            "Profile",
            "Status",
            "RecordType",
            "VisitTime",
            "URL",
            "Title",
            "Transition",
            "VisitDuration",
            "VisitCount",
            "Typed",
            "TypedCount",
            "Hidden",
            "Frecency",
            "VisitID",
            "FromVisitID",
            "LastVisitTime",
            "PageSource",
            "Confidence",
            "RecoveredFrom",
            "LostColumns",
        ]
    );
    assert_eq!(history.len(), 30);
    let first = &history[0];
    assert_eq!(
        Path::new(&first["SourceFile"]),
        host.join(EDGE).join("History")
    );
    assert_eq!(first["User"], "alice");
    assert_eq!(first["Browser"], "Edge");
    assert_eq!(first["Profile"], "Default");
    assert_eq!(first["Status"], "Live");
    assert_eq!(first["RecordType"], "Visit");
    assert_eq!(first["VisitTime"], "2026-11-07T12:32:09.882399Z");
    assert_eq!(first["URL"], "http://127.0.0.1:8765/page1");
    assert_eq!(first["Title"], "Test page 1 日本語");
    assert_eq!(first["Transition"], "TYPED|FROM_API|CHAIN_START|CHAIN_END");
    assert_eq!(first["VisitDuration"], "1.206218");
    assert_eq!(first["Typed"], "true");
    assert_eq!(first["TypedCount"], "1");
    assert_eq!(first["VisitID"], "1");
    assert_eq!(first["Confidence"], "");

    // The two pages deleted from places.sqlite in its write-ahead log, with their visits: from the
    // database file's copies of the pages the log replaced.
    let deleted: Vec<_> = history
        .iter()
        .filter(|r| r["Status"] == "Deleted")
        .collect();
    assert_eq!(deleted.len(), 4);
    let visits: Vec<_> = deleted
        .iter()
        .filter(|r| r["RecordType"] == "Visit")
        .map(|r| (r["VisitTime"].as_str(), r["URL"].as_str()))
        .collect();
    assert_eq!(
        visits,
        [
            ("2026-11-07T12:36:45.954264Z", "http://localhost:8765/page9"),
            ("2026-11-07T12:36:43.233828Z", "http://127.0.0.1:8765/page2"),
        ]
    );
    for row in &deleted {
        assert_eq!(row["Browser"], "Firefox");
        assert_eq!(row["Confidence"], "High");
        assert!(
            row["RecoveredFrom"].contains("(replaced by the WAL)"),
            "{row:?}"
        );
        assert!(row["Title"].starts_with("Test page "), "{row:?}");
    }
    let pages: Vec<_> = deleted
        .iter()
        .filter(|r| r["RecordType"] == "Page")
        .collect();
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0]["LastVisitTime"], "2026-11-07T12:36:45.954264Z");
    assert_eq!(pages[0]["VisitTime"], "");
    assert!(
        deleted
            .iter()
            .filter(|r| r["RecordType"] == "Visit")
            .all(|r| r["PageSource"] == "Recovered")
    );

    let (_, downloads) = csv_rows(&out.join("Browser-Downloads.csv"));
    assert_eq!(downloads.len(), 2);
    let edge = &downloads[0];
    assert_eq!(edge["Browser"], "Edge");
    assert_eq!(edge["StartTime"], "2026-11-07T12:32:18.323966Z");
    assert_eq!(
        edge["URL"],
        "http://127.0.0.1:8765/download/report-2026.bin"
    );
    assert_eq!(
        edge["RedirectChain"],
        "http://127.0.0.1:8765/download/report-2026.bin"
    );
    assert!(edge["TargetPath"].ends_with("report-2026.bin"));
    assert_eq!(edge["ReceivedBytes"], "102400");
    assert_eq!(edge["State"], "Complete");
    assert_eq!(edge["TabURL"], "http://localhost:8765/cookies");
    let firefox = &downloads[1];
    assert_eq!(firefox["Browser"], "Firefox");
    assert!(firefox["TargetPath"].starts_with("file:///"));
    assert_eq!(firefox["FileDeletedByUser"], "false");

    let (_, cookies) = csv_rows(&out.join("Browser-Cookies.csv"));
    assert_eq!(cookies.len(), 16);
    assert_eq!(
        cookies.iter().filter(|c| c["Browser"] == "Edge").count(),
        10
    );
    let cookie = &cookies[0];
    assert!(
        Path::new(&cookie["SourceFile"]).ends_with("Default/Network/Cookies"),
        "{cookie:?}"
    );
    assert_eq!(cookie["Profile"], "Default");
    assert_eq!(cookie["Host"], "127.0.0.1");
    assert_eq!(cookie["Created"], "2026-11-07T12:32:15.909358Z");
    assert_eq!(cookie["Expires"], "2026-11-17T12:32:15.909358Z");
    assert_eq!(cookie["HttpOnly"], "true");

    let (_, forms) = csv_rows(&out.join("Browser-FormHistory.csv"));
    assert_eq!(forms.len(), 6);
    let values: Vec<(&str, &str, &str)> = forms
        .iter()
        .map(|f| {
            (
                f["Browser"].as_str(),
                f["FieldName"].as_str(),
                f["Value"].as_str(),
            )
        })
        .collect();
    assert_eq!(
        values,
        [
            ("Edge", "email", "alice@example.com"),
            ("Edge", "city", "Kagoshima"),
            ("Edge", "email", "bob@example.org"),
            ("Firefox", "email", "alice@example.com"),
            ("Firefox", "city", "Kagoshima"),
            ("Firefox", "email", "bob@example.org"),
        ]
    );
    assert_eq!(forms[3]["GUID"], "JZn8Ia0IQbebMgS0");
}

#[test]
fn browser_json_and_jsonl() {
    let dir = tempfile::tempdir().unwrap();
    let host = collection(dir.path());
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-d",
        host.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let rows = jsonl(&r.stdout);
    assert_eq!(rows.len(), 54);
    let first = rows[0].as_object().unwrap();
    let keys: Vec<&str> = first.keys().map(String::as_str).take(4).collect();
    assert_eq!(keys, ["SourceFile", "Table", "User", "Browser"]);
    assert_eq!(first["Table"], "History");
    assert_eq!(first["VisitDuration"], 1.206218);
    assert_eq!(first["Typed"], true);
    assert_eq!(first["Frecency"], Value::Null);
    let tables: BTreeMap<String, usize> = rows.iter().fold(BTreeMap::new(), |mut m, r| {
        *m.entry(r["Table"].as_str().unwrap().to_string())
            .or_default() += 1;
        m
    });
    assert_eq!(
        tables,
        BTreeMap::from([
            ("Cookies".to_string(), 16),
            ("Downloads".to_string(), 2),
            ("FormHistory".to_string(), 6),
            ("History".to_string(), 30),
        ])
    );
    let deleted = rows
        .iter()
        .find(|r| r["Status"] == "Deleted" && r["RecordType"] == "Visit")
        .unwrap();
    assert_eq!(deleted["Confidence"], "High");

    let out = dir.path().join("browser.json");
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-d",
        host.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-t",
        "json",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let text = std::fs::read_to_string(&out).unwrap();
    let pretty: Vec<Value> = serde_json::Deserializer::from_str(&text)
        .into_iter::<Value>()
        .map(Result::unwrap)
        .collect();
    assert_eq!(pretty, rows);
    assert!(text.starts_with("{\n  \"SourceFile\""));
}

#[test]
fn browser_form_history_deleted_and_removed_entries() {
    let dir = tempfile::tempdir().unwrap();
    let profile = dir.path().join("profile");
    unpack("formhistory-wal", &profile);
    let database = profile.join("formhistory.sqlite");
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-f",
        database.to_str().unwrap(),
        "-t",
        "jsonl",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let rows = jsonl(&r.stdout);
    let status: Vec<&str> = rows.iter().map(|r| r["Status"].as_str().unwrap()).collect();
    assert_eq!(
        status,
        ["Live", "Live", "Live", "Live", "Deleted", "Removed"]
    );
    assert!(rows.iter().all(|r| r["Browser"] == "Firefox-based"));
    assert_eq!(
        rows[1]["Sources"],
        serde_json::json!(["Example Search", "Example Encyclopedia"])
    );
    let deleted = &rows[4];
    assert_eq!(deleted["FieldName"], "city");
    assert_eq!(deleted["Value"], "Springfield");
    assert_eq!(deleted["GUID"], "GuIdCiTy00000004");
    assert_eq!(deleted["FirstUsed"], "2026-09-01T08:02:00Z");
    assert_eq!(deleted["Confidence"], "High");
    assert_eq!(
        deleted["RecoveredFrom"],
        "Cell of the database file's copy of page 2 (replaced by the WAL), offset 3822"
    );
    let removed = &rows[5];
    assert_eq!(removed["GUID"], "GuIdCiTy00000004");
    assert_eq!(removed["RemovedTime"], "2026-09-01T08:03:00Z");
    assert_eq!(removed["FieldName"], Value::Null);

    // CSV joins the sources with `|`
    let out = dir.path().join("out");
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-f",
        database.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let (_, forms) = csv_rows(&out.join("Browser-FormHistory.csv"));
    assert_eq!(forms[1]["Sources"], "Example Search | Example Encyclopedia");
    // the other tables are written with their headers
    let (header, history) = csv_rows(&out.join("Browser-History.csv"));
    assert!(history.is_empty());
    assert_eq!(header[0], "SourceFile");
}

#[test]
fn browser_reads_the_write_ahead_log_and_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let profile = dir.path().join("profile");
    unpack("firefox140", &profile);
    let before: Vec<(PathBuf, Vec<u8>)> = std::fs::read_dir(&profile)
        .unwrap()
        .map(|e| {
            let path = e.unwrap().path();
            let data = std::fs::read(&path).unwrap();
            (path, data)
        })
        .collect();
    let summary = |r: &Run, line: &str| {
        assert_eq!(r.code, 0, "{}\n{}", r.stdout, r.stderr);
        assert!(r.stdout.contains(line), "{line}\n{}", r.stdout);
    };
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-d",
        profile.to_str().unwrap(),
        "-K",
    ]);
    summary(&r, "Visits: 13 (deleted, recovered: 2)\n");
    summary(&r, "Cookies: 6\n");
    for (path, data) in &before {
        assert_eq!(&std::fs::read(path).unwrap(), data, "{}", path.display());
    }
    assert!(!profile.join("places.sqlite-shm").exists());
    // Without the logs, the visits deleted in the log are still live and the cookies are not
    // there yet.
    for wal in ["places.sqlite-wal", "cookies.sqlite-wal"] {
        std::fs::remove_file(profile.join(wal)).unwrap();
    }
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-d",
        profile.to_str().unwrap(),
        "-K",
    ]);
    summary(&r, "Visits: 15\n");
    summary(&r, "Cookies: 0\n");
    summary(&r, "Read with their write-ahead log (-wal): 0\n");
}

#[test]
fn browser_input_and_output_errors() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing");
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-f",
        missing.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.all().contains("The file does not exist"), "{}", r.all());
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-d",
        missing.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.all().contains("The directory does not exist"),
        "{}",
        r.all()
    );

    let empty = dir.path().join("empty");
    std::fs::create_dir(&empty).unwrap();
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-d",
        empty.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    // the message auto-parse takes for "not collected"
    assert!(
        r.all().contains("No browser databases (") && r.all().contains(" were found in "),
        "{}",
        r.all()
    );
    // a History of another program alone is nothing to parse, as well
    std::fs::write(empty.join("History"), b"plain text").unwrap();
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-d",
        empty.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.all().contains(" were found in "), "{}", r.all());
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-f",
        empty.join("History").to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.all()
            .contains("is not a browser database that parse-sqlite browser reads"),
        "{}",
        r.all()
    );
    // a damaged database is reported, not skipped
    let damaged = dir.path().join("damaged");
    std::fs::create_dir(&damaged).unwrap();
    std::fs::write(damaged.join("History"), b"SQLite format 3\0").unwrap();
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-d",
        damaged.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.all().contains("Could not parse"), "{}", r.all());
    assert!(r.all().contains("(could not parse: 1)"), "{}", r.all());
    // another kind of browser file: an ESE database named as a WebCache
    let mut ese = vec![0u8; 4096];
    ese[4..8].copy_from_slice(&[0xef, 0xcd, 0xab, 0x89]);
    let webcache = dir.path().join("WebCacheV01.dat");
    std::fs::write(&webcache, ese).unwrap();
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-f",
        webcache.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.all()
            .contains("is a WebCache file, which parse-sqlite browser does not read"),
        "{}",
        r.all()
    );
    assert!(!r.stdout.contains("Results Summary"), "{}", r.stdout);

    let profile = dir.path().join("profile");
    unpack("chromium150", &profile);
    let history = profile.join("History");
    let r = run(&[
        "parse-sqlite",
        "browser",
        "-f",
        history.to_str().unwrap(),
        "-t",
        "csv",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.all().contains("CSV output needs -o <DIR>"), "{}", r.all());

    // existing results are kept without -C, and an input is never overwritten
    let out = dir.path().join("rows.jsonl");
    std::fs::write(&out, b"keep").unwrap();
    let args = |o: &Path, clobber: bool| {
        let mut a = vec![
            "parse-sqlite".to_string(),
            "browser".into(),
            "-d".into(),
            profile.to_string_lossy().to_string(),
            "-o".into(),
            o.to_string_lossy().to_string(),
            "-t".into(),
            "jsonl".into(),
            "-K".into(),
        ];
        if clobber {
            a.push("-C".into());
        }
        a
    };
    let r = run(&args(&out, false)
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>());
    assert_eq!(r.code, 1);
    assert_eq!(std::fs::read(&out).unwrap(), b"keep");
    let original = std::fs::read(&history).unwrap();
    let r = run(&args(&history, true)
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>());
    assert_eq!(r.code, 1, "{}", r.stdout);
    assert_eq!(std::fs::read(&history).unwrap(), original);
}

#[test]
fn auto_parse_puts_browser_tables_in_browser_activity_with_their_times_in_the_offset() {
    let evidence = tempfile::tempdir().unwrap();
    collection(evidence.path());
    let target = tempfile::tempdir().unwrap();
    for kind in ["csv", "jsonl"] {
        let output = target.path().join(kind);
        let r = run(&[
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
        assert_eq!(r.code, 0, "{}\n{}", r.stdout, r.stderr);
        let manifest: Value =
            serde_json::from_slice(&std::fs::read(output.join("manifest.json")).unwrap()).unwrap();
        let task = manifest["Tasks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["Artifact"] == "browser" && t["Purpose"] == "artifact")
            .unwrap();
        assert_eq!(task["Status"], "succeeded", "{task}");
        assert_eq!(task["Category"], "BrowserActivity");
        assert_eq!(task["Command"][1], "parse-sqlite");
        assert_eq!(task["Command"][2], "browser");
        let browser = output.join("HOST1/BrowserActivity");
        assert!(browser.join("Browser-logs.txt").is_file());
        if kind == "csv" {
            assert!(!browser.join("Browser").exists());
            assert_eq!(task["OutputFiles"].as_array().unwrap().len(), 4, "{task}");
            let (_, history) = csv_rows(&browser.join("Browser-History.csv"));
            assert_eq!(history.len(), 30);
            assert_eq!(history[0]["VisitTime"], "2026-11-07T21:32:09.882399+09:00");
            // the parser runs in the evidence directory: paths are relative to it
            assert!(
                Path::new(&history[0]["SourceFile"]).starts_with(Path::new(".").join("HOST1")),
                "{}",
                history[0]["SourceFile"]
            );
            let (_, cookies) = csv_rows(&browser.join("Browser-Cookies.csv"));
            assert_eq!(cookies[0]["Expires"], "2026-11-17T21:32:15.909358+09:00");
        } else {
            let rows = jsonl(&std::fs::read_to_string(browser.join("Browser.jsonl")).unwrap());
            assert_eq!(rows.len(), 54);
            assert!(
                rows.iter()
                    .all(|r| r["Comp"] == "HOST1" && r["Artifact"] == "browser")
            );
        }
        let timeline = jsonl(&std::fs::read_to_string(output.join("all.jsonl")).unwrap());
        let rows: Vec<&Value> = timeline
            .iter()
            .filter(|r| r["Artifact"] == "browser")
            .collect();
        assert_eq!(rows.len(), 54);
        let form = rows.iter().find(|r| r["Table"] == "FormHistory").unwrap();
        assert_eq!(form["LastUsed"], "2026-11-07T21:32:25+09:00");
    }
}
