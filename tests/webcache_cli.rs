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

//! `parse-ese webcache` and its `auto-parse` task, on plaso's two WebCache databases in
//! test_files/webcache: Internet Explorer 11's (user `test`) and a dirty one of Windows 10 with
//! legacy Edge's history, downloads and cookies (user `John Doe`).

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

const IE: &str = "C/Users/test/AppData/Local/Microsoft/Windows/WebCache";
const EDGE: &str = "C/Users/John Doe/AppData/Local/Microsoft/Windows/WebCache";
const EDGE_PACKAGE: &str = "microsoft.microsoftedge_8wekyb3d8bbwe";

/// test_files/webcache/plaso/`name`.gz, decompressed to `directory`/WebCacheV01.dat.
fn unpack(name: &str, directory: &Path) -> PathBuf {
    std::fs::create_dir_all(directory).unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files/webcache/plaso")
        .join(format!("{name}.gz"));
    let mut data = Vec::new();
    GzDecoder::new(std::fs::File::open(source).unwrap())
        .read_to_end(&mut data)
        .unwrap();
    let database = directory.join("WebCacheV01.dat");
    std::fs::write(&database, data).unwrap();
    database
}

/// A host with both databases in their users' profiles, beside their transaction logs' names.
fn collection(root: &Path) -> PathBuf {
    let host = root.join("HOST1");
    unpack("WebCacheV01.dat", &host.join(IE));
    unpack("PartitionsEx-WebCacheV01.dat", &host.join(EDGE));
    // what else is in the folder: not databases
    for name in ["V01.log", "V01.chk", "WebCacheV01.jfm"] {
        std::fs::write(host.join(EDGE).join(name), b"not a database").unwrap();
    }
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
fn webcache_summary_on_the_terminal() {
    let evidence = tempfile::tempdir().unwrap();
    let host = collection(evidence.path());
    let r = run(&["parse-ese", "webcache", "-d", host.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 0, "{}", r.all());
    for line in [
        "WebCache database files: 2",
        "WebCache databases parsed: 2",
        "Left dirty (changes still only in the transaction logs are missing): 1",
        "Users: 2",
        "  John Doe",
        "  test",
        "History entries: 182",
        "Downloads: 6",
        "Cookies: 170",
        "Other entries: 3,754",
        "Containers: 44",
        "History access times: 2014-03-15T16:12:59.093227900Z - 2019-12-16T21:03:37.866515200Z",
        "Hosts with the most history entries:",
        "        22  www.forensicswiki.org",
        "Problems: 1",
        "To save every row, add -o <DIR> (CSV tables) or -o <FILE> -t json or jsonl.",
    ] {
        assert!(
            r.stdout.lines().any(|l| l == line),
            "{line:?}\n{}",
            r.stdout
        );
    }
    // the dirty database's problem, with its file
    assert!(
        r.all()
            .contains("WebCacheV01.dat: dirty shutdown: changes still only in the transaction log"),
        "{}",
        r.all()
    );
}

#[test]
fn webcache_csv_tables() {
    let evidence = tempfile::tempdir().unwrap();
    let host = collection(evidence.path());
    let out = evidence.path().join("csv");
    let r = run(&[
        "parse-ese",
        "webcache",
        "-d",
        host.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    assert!(r.stdout.contains("(5 CSV files)"), "{}", r.stdout);

    let (header, history) = csv_rows(&out.join("WebCache-History.csv"));
    assert_eq!(
        header,
        [
            "SourceFile",
            "User",
            "Package",
            "Container",
            "AccessedTime",
            "Account",
            "URL",
            "AccessCount",
            "EntryID",
            "ContainerID"
        ]
    );
    assert_eq!(history.len(), 182);
    // the day's container of Internet Explorer's history, and the visit in the main one
    let superfetch: Vec<_> = history
        .iter()
        .filter(|h| {
            h["User"] == "test"
                && h["URL"] == "http://www.forensicswiki.org/wiki/Windows_SuperFetch_Format"
        })
        .collect();
    assert_eq!(superfetch.len(), 3, "{superfetch:#?}");
    let daily = superfetch
        .iter()
        .find(|h| h["Container"] == "MSHist012014051220140513")
        .unwrap();
    assert_eq!(daily["AccessedTime"], "2014-05-12T07:31:38.359991100Z");
    assert_eq!(daily["Account"], "test");
    assert_eq!(daily["Package"], "");
    assert_eq!(daily["ContainerID"], "23");
    assert!(
        superfetch
            .iter()
            .any(|h| h["Container"] == "History" && h["AccessCount"] == "5")
    );
    // legacy Edge's history is its package's
    let edge: Vec<_> = history.iter().filter(|h| h["User"] == "John Doe").collect();
    assert_eq!(edge.len(), 69);
    assert!(edge.iter().all(|h| h["Package"] == EDGE_PACKAGE));
    assert!(
        history.iter().any(|h| h["Package"] == "windows_ie_ac_001"),
        "Internet Explorer's app container"
    );

    let (_, downloads) = csv_rows(&out.join("WebCache-Downloads.csv"));
    assert_eq!(downloads.len(), 6);
    let pdf = downloads
        .iter()
        .find(|d| d["User"] == "test" && d["DownloadID"] == "1")
        .unwrap();
    assert_eq!(
        pdf["URL"],
        "https://googledrive.com/host/0B3fBvzttpiiSWFFkSTBOUHdjOVU/Windows%20SuperFetch%20(DB)%20format.pdf"
    );
    assert_eq!(
        pdf["TargetPath"],
        r"C:\Users\test\Downloads\Windows SuperFetch (DB) format.pdf"
    );
    assert_eq!(
        pdf["CurrentPath"],
        r"C:\Users\test\AppData\Local\Microsoft\Windows\INetCache\Low\IE\0H5ICNJW\Windows%20SuperFetch%20(DB)%20format[1].pdf"
    );
    assert_eq!(pdf["TotalBytes"], "241651");
    assert_eq!(pdf["MimeType"], "application/pdf");
    assert_eq!(
        pdf["Referrer"],
        "http://www.forensicswiki.org/wiki/Windows_SuperFetch_Format"
    );
    assert_eq!(pdf["StartTime"], "2014-05-12T07:33:03.037773200Z");
    assert_eq!(pdf["Publisher"], "");
    let putty = downloads
        .iter()
        .find(|d| d["TargetPath"].ends_with(r"\putty.exe"))
        .unwrap();
    assert_eq!(
        putty["URL"],
        "https://the.earth.li/~sgtatham/putty/0.73/w64/putty.exe"
    );
    assert_eq!(putty["Package"], EDGE_PACKAGE);
    assert_eq!(putty["Publisher"], "Simon Tatham");
    assert_eq!(putty["TotalBytes"], "1179024");
    assert_eq!(putty["StartTime"], "2019-12-16T21:03:47.476744500Z");
    assert!(
        putty["CurrentPath"].ends_with(r"TempState\Downloads\putty.exe.j11qakb.partial"),
        "{}",
        putty["CurrentPath"]
    );

    let (_, cookies) = csv_rows(&out.join("WebCache-Cookies.csv"));
    assert_eq!(cookies.len(), 170);
    let market = cookies
        .iter()
        .find(|c| c["CookieTable"] == "CookieEntryEx_18" && c["Name"] == "sptmarket")
        .unwrap();
    assert_eq!(market["Host"], "www.msn.com");
    assert_eq!(market["Path"], "/");
    assert_eq!(market["Value"], "en-us|fr|FR|en-us|en-us|en");
    assert_eq!(market["LastModified"], "2019-12-16T20:54:41.888898Z");
    assert_eq!(market["Expires"], "2021-12-16T20:54:41.888898Z");
    let edge = cookies
        .iter()
        .find(|c| c["CookieTable"] == "CookieEntryEx_13")
        .unwrap();
    assert!(
        edge["Directory"].ends_with(r"\AC\#!001\MicrosoftEdge\Cookies\ESE\"),
        "{}",
        edge["Directory"]
    );
    assert_eq!(edge["Package"], EDGE_PACKAGE);

    let (header, entries) = csv_rows(&out.join("WebCache-Entries.csv"));
    assert!(header.contains(&"Filename".to_string()), "{header:?}");
    assert_eq!(entries.len(), 3_754);
    let widgets = entries
        .iter()
        .find(|e| e["URL"] == "http://platform.twitter.com/widgets.js")
        .unwrap();
    assert_eq!(widgets["Container"], "Content");
    assert_eq!(widgets["Filename"], "widgets[1].js");
    assert_eq!(
        widgets["FilePath"],
        r"C:\Users\test\AppData\Local\Microsoft\Windows\INetCache\Low\IE\SLTWV5F0\widgets[1].js"
    );
    assert_eq!(widgets["FileSize"], "98240");
    assert_eq!(widgets["AccessedTime"], "2014-05-12T07:32:18.224844100Z");
    assert_eq!(widgets["ModifiedTime"], "2014-05-09T01:44:18Z");
    assert_eq!(widgets["CreationTime"], "2014-05-12T07:32:17.709226900Z");
    // the headers as stored, their lines and all
    assert!(
        widgets["ResponseHeaders"]
            .starts_with("HTTP/1.1 200 OK\r\nContent-Type: application/javascript\r\n"),
        "{:?}",
        widgets["ResponseHeaders"]
    );
    let cookie_file = entries
        .iter()
        .find(|e| e["URL"] == "Cookie:test@microsoft.com/")
        .unwrap();
    assert_eq!(cookie_file["Container"], "Cookies");
    assert_eq!(cookie_file["Account"], "test");
    assert_eq!(
        cookie_file["FilePath"],
        r"C:\Users\test\AppData\Local\Microsoft\Windows\INetCookies\VLR48FYL.txt"
    );

    let (_, containers) = csv_rows(&out.join("WebCache-Containers.csv"));
    assert_eq!(containers.len(), 44);
    let content = containers
        .iter()
        .find(|c| c["User"] == "test" && c["ContainerID"] == "1")
        .unwrap();
    assert_eq!(content["Name"], "Content");
    assert_eq!(
        content["SecureDirectories"],
        "WFV5DJED, 9KIDZSRE, 961394QI, MB16RAJJ"
    );
    assert_eq!(
        content["Directory"],
        r"C:\Users\test\AppData\Local\Microsoft\Windows\INetCache\IE\"
    );
}

#[test]
fn webcache_json_and_jsonl() {
    let evidence = tempfile::tempdir().unwrap();
    let host = collection(evidence.path());
    let database = host.join(EDGE).join("WebCacheV01.dat");
    // JSONL on stdout: the rows only, the problems on stderr
    let r = run(&[
        "parse-ese",
        "webcache",
        "-f",
        database.to_str().unwrap(),
        "-t",
        "jsonl",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    let rows = jsonl(&r.stdout);
    assert_eq!(rows.len(), 69 + 4 + 170 + 2_320 + 23);
    assert!(r.stderr.contains("dirty shutdown"), "{}", r.stderr);
    for row in &rows {
        let keys: Vec<&String> = row.as_object().unwrap().keys().collect();
        assert_eq!(keys[0], "SourceFile");
        assert_eq!(keys[1], "Table");
        assert_eq!(row["User"], "John Doe");
    }
    let tables: BTreeMap<String, usize> = rows.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts
            .entry(row["Table"].as_str().unwrap().to_string())
            .or_default() += 1;
        counts
    });
    assert_eq!(
        tables,
        BTreeMap::from([
            ("Containers".to_string(), 23),
            ("Cookies".to_string(), 170),
            ("Downloads".to_string(), 4),
            ("Entries".to_string(), 2_320),
            ("History".to_string(), 69),
        ])
    );
    let putty = rows
        .iter()
        .find(|r| r["Table"] == "Downloads" && r["DownloadID"] == 4)
        .unwrap();
    assert_eq!(putty["TotalBytes"], 1_179_024);
    assert_eq!(putty["MimeType"], Value::Null);
    let container = rows.iter().find(|r| r["Table"] == "Containers").unwrap();
    assert!(container["SecureDirectories"].is_array(), "{container}");

    // one JSON file
    let out = evidence.path().join("out/webcache.json");
    let r = run(&[
        "parse-ese",
        "webcache",
        "-f",
        database.to_str().unwrap(),
        "-t",
        "json",
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    assert!(r.stdout.contains("Results saved: "), "{}", r.stdout);
    let text = std::fs::read_to_string(&out).unwrap();
    let objects: Vec<Value> = serde_json::Deserializer::from_str(&text)
        .into_iter::<Value>()
        .map(Result::unwrap)
        .collect();
    assert_eq!(objects.len(), rows.len());
    assert_eq!(objects[0], rows[0]);
}

#[test]
fn webcache_reads_and_changes_nothing() {
    let evidence = tempfile::tempdir().unwrap();
    let host = collection(evidence.path());
    let databases = [host.join(IE), host.join(EDGE)].map(|d| d.join("WebCacheV01.dat"));
    let before: Vec<Vec<u8>> = databases
        .iter()
        .map(|d| std::fs::read(d).unwrap())
        .collect();
    let out = evidence.path().join("csv");
    let r = run(&[
        "parse-ese",
        "webcache",
        "-d",
        host.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-q",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    let after: Vec<Vec<u8>> = databases
        .iter()
        .map(|d| std::fs::read(d).unwrap())
        .collect();
    assert!(before == after, "the databases are unchanged");
    // nothing written beside them either
    for directory in [host.join(IE), host.join(EDGE)] {
        let names: Vec<String> = std::fs::read_dir(&directory)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert!(
            names.iter().all(
                |n| ["WebCacheV01.dat", "V01.log", "V01.chk", "WebCacheV01.jfm"]
                    .contains(&n.as_str())
            ),
            "{names:?}"
        );
    }
}

#[test]
fn webcache_input_and_output_errors() {
    let dir = tempfile::tempdir().unwrap();
    // nothing to find: the message auto-parse takes for a skipped task
    let empty = dir.path().join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    let r = run(&["parse-ese", "webcache", "-d", empty.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 1);
    assert!(
        r.all()
            .contains("No WebCache databases (WebCacheV01.dat) were found in"),
        "{}",
        r.all()
    );
    let r = run(&[
        "parse-ese",
        "webcache",
        "-f",
        dir.path().join("missing.dat").to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.all().contains("The file does not exist"), "{}", r.all());
    // a file named as one that is not one: could not be parsed, nothing saved
    let damaged = dir.path().join("damaged");
    std::fs::create_dir_all(&damaged).unwrap();
    std::fs::write(damaged.join("WebCacheV01.dat"), b"not a database").unwrap();
    let out = dir.path().join("nothing");
    let r = run(&[
        "parse-ese",
        "webcache",
        "-d",
        damaged.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.all().contains("Could not parse"), "{}", r.all());
    assert!(r.all().contains("(could not parse: 1)"), "{}", r.all());
    assert!(r.all().contains("Nothing was saved"), "{}", r.all());
    assert!(!out.exists());
    // an ESE database of another kind
    let srum = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/srum");
    if let Some(other) = std::fs::read_dir(&srum)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "dat"))
    {
        let r = run(&["parse-ese", "webcache", "-f", other.to_str().unwrap(), "-K"]);
        assert_eq!(r.code, 1);
        assert!(
            r.all().contains("an ESE database, but not a WebCache"),
            "{}",
            r.all()
        );
    }
    // CSV needs a directory
    let host = collection(dir.path());
    let r = run(&[
        "parse-ese",
        "webcache",
        "-d",
        host.to_str().unwrap(),
        "-t",
        "csv",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.all().contains("CSV output needs -o <DIR>"), "{}", r.all());
    // an existing output is kept without -C
    let out = dir.path().join("rows.jsonl");
    std::fs::write(&out, b"keep").unwrap();
    let database = host.join(IE).join("WebCacheV01.dat");
    let args = [
        "parse-ese",
        "webcache",
        "-f",
        database.to_str().unwrap(),
        "-t",
        "jsonl",
        "-o",
        out.to_str().unwrap(),
        "-K",
    ];
    let r = run(&args);
    assert_eq!(r.code, 1, "{}", r.all());
    assert_eq!(std::fs::read(&out).unwrap(), b"keep");
    let r = run(&[&args[..], &["-C"]].concat());
    assert_eq!(r.code, 0, "{}", r.all());
    assert_eq!(
        jsonl(&std::fs::read_to_string(&out).unwrap()).len(),
        113 + 2 + 1_434 + 21
    );
    // the database itself is never the output
    let r = run(&[
        "parse-ese",
        "webcache",
        "-f",
        database.to_str().unwrap(),
        "-t",
        "jsonl",
        "-o",
        database.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1, "{}", r.all());
    assert!(std::fs::metadata(&database).unwrap().len() > 1_000_000);
}

#[test]
fn auto_parse_puts_webcache_tables_in_browser_activity_with_their_times_in_the_offset() {
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
            .find(|t| t["Artifact"] == "webcache" && t["Purpose"] == "artifact")
            .unwrap();
        assert_eq!(task["Status"], "succeeded", "{task}");
        assert_eq!(task["Category"], "BrowserActivity");
        assert_eq!(task["Command"][1], "parse-ese");
        assert_eq!(task["Command"][2], "webcache");
        let activity = output.join("HOST1/BrowserActivity");
        assert!(activity.join("WebCache-logs.txt").is_file());
        if kind == "csv" {
            assert!(!activity.join("WebCache").exists());
            assert_eq!(task["OutputFiles"].as_array().unwrap().len(), 5, "{task}");
            let (_, history) = csv_rows(&activity.join("WebCache-History.csv"));
            assert_eq!(history.len(), 182);
            // the parser runs in the evidence directory: paths are relative to it
            assert!(
                history
                    .iter()
                    .all(|h| Path::new(&h["SourceFile"]).starts_with(Path::new(".").join("HOST1"))),
                "{}",
                history[0]["SourceFile"]
            );
            let daily = history
                .iter()
                .find(|h| h["Container"] == "MSHist012014051220140513")
                .unwrap();
            assert!(
                daily["AccessedTime"].ends_with("+09:00"),
                "{}",
                daily["AccessedTime"]
            );
            let (_, downloads) = csv_rows(&activity.join("WebCache-Downloads.csv"));
            let putty = downloads
                .iter()
                .find(|d| d["Publisher"] == "Simon Tatham")
                .unwrap();
            assert_eq!(putty["StartTime"], "2019-12-17T06:03:47.476744500+09:00");
            let (_, cookies) = csv_rows(&activity.join("WebCache-Cookies.csv"));
            let market = cookies.iter().find(|c| c["Name"] == "sptmarket").unwrap();
            assert_eq!(market["Expires"], "2021-12-17T05:54:41.888898+09:00");
            for table in ["Entries", "Containers"] {
                assert!(activity.join(format!("WebCache-{table}.csv")).is_file());
            }
        } else {
            let rows = jsonl(&std::fs::read_to_string(activity.join("WebCache.jsonl")).unwrap());
            assert_eq!(rows.len(), 4_156);
            assert!(
                rows.iter()
                    .all(|r| r["Comp"] == "HOST1" && r["Artifact"] == "webcache")
            );
        }
        let timeline = jsonl(&std::fs::read_to_string(output.join("all.jsonl")).unwrap());
        let rows: Vec<&Value> = timeline
            .iter()
            .filter(|r| r["Artifact"] == "webcache")
            .collect();
        assert_eq!(rows.len(), 4_156);
        let widgets = rows
            .iter()
            .find(|r| r["URL"] == "http://platform.twitter.com/widgets.js")
            .unwrap();
        assert_eq!(widgets["ModifiedTime"], "2014-05-09T10:44:18+09:00");
    }
}
