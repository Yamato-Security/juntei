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

//! End-to-end timeline tests exercise file protection, chronology, profiles and provenance.
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}
fn run(args: &[&str]) {
    let r = Command::new(env!("CARGO_BIN_EXE_juntei"))
        .args(args)
        .output()
        .unwrap();
    assert!(
        r.status.success(),
        "{args:?}\n{}\n{}",
        String::from_utf8_lossy(&r.stdout),
        String::from_utf8_lossy(&r.stderr)
    );
}
fn unpack(source: &Path, dest: &Path) {
    if source.extension().is_some_and(|x| x == "gz") {
        std::io::copy(
            &mut flate2::read::GzDecoder::new(fs::File::open(source).unwrap()),
            &mut fs::File::create(dest).unwrap(),
        )
        .unwrap();
    } else {
        fs::copy(source, dest).unwrap();
    }
}
fn events(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}
fn keys(rows: &[Value]) -> BTreeSet<String> {
    rows.iter()
        .map(|v| {
            serde_json::to_string(&(
                &v["Timestamp"],
                &v["Artifact"],
                &v["TimestampType"],
                &v["SourceFile"],
                &v["SourceField"],
                &v["RecordID"],
            ))
            .unwrap()
        })
        .collect()
}

/// Real parser exports, rather than invented records, exercise table detection in CSV and JSONL.
/// All fixture files are public and retain their original license notices in test_files.
#[test]
fn newer_parsers_csv_and_jsonl_have_the_same_timestamp_events() {
    for (artifact, group, name, filename) in [
        (
            "browser",
            "parse-sqlite",
            "test_files/browser/chromium150/History.gz",
            "History",
        ),
        (
            "browser",
            "parse-sqlite",
            "test_files/browser/chromium150/Cookies.gz",
            "Cookies",
        ),
        (
            "browser",
            "parse-sqlite",
            "test_files/browser/chromium150/Web Data.gz",
            "Web Data",
        ),
        (
            "browser",
            "parse-sqlite",
            "test_files/browser/firefox140/formhistory.sqlite.gz",
            "formhistory.sqlite",
        ),
        (
            "webcache",
            "parse-ese",
            "test_files/webcache/plaso/WebCacheV01.dat.gz",
            "WebCacheV01.dat",
        ),
        (
            "webcache",
            "parse-ese",
            "test_files/webcache/plaso/PartitionsEx-WebCacheV01.dat.gz",
            "WebCacheV01.dat",
        ),
        (
            "outlook",
            "parse-file",
            "test_files/outlook/apache-poi/msgClassAppointment.msg",
            "appointment.msg",
        ),
        (
            "outlook",
            "parse-file",
            "test_files/outlook/apache-poi/msgClassTask.msg",
            "task.msg",
        ),
        (
            "outlook",
            "parse-file",
            "test_files/outlook/msgreader-hiraoka/contactAnsi.msg",
            "contact.msg",
        ),
        (
            "outlook",
            "parse-file",
            "test_files/outlook/msgreader-hiraoka/msgInMsgInMsg.msg",
            "embedded.msg",
        ),
        (
            "outlook",
            "parse-file",
            "test_files/outlook/java-libpst/example-2013.ost.gz",
            "example.ost",
        ),
        (
            "outlook",
            "parse-file",
            "test_files/outlook/libyal/outlook.pst.gz",
            "example.pst",
        ),
        (
            "ual",
            "parse-ese",
            "test_files/ual/server2019/Current.mdb",
            "Current.mdb",
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let evidence = dir.path().join("evidence");
        fs::create_dir(&evidence).unwrap();
        let source = evidence.join(filename);
        unpack(&repo(name), &source);
        if artifact == "ual" {
            fs::copy(
                repo("test_files/ual/server2019/SystemIdentity.mdb"),
                evidence.join("SystemIdentity.mdb"),
            )
            .unwrap();
        }
        // Bring the SQLite WAL when this public database has one.
        let wal = repo(&format!("{}-wal.gz", name.trim_end_matches(".gz")));
        if wal.exists() {
            unpack(&wal, &evidence.join(format!("{filename}-wal")));
        }
        let json = dir.path().join("native.jsonl");
        let csv = dir.path().join("tables");
        for (format, out) in [("jsonl", &json), ("csv", &csv)] {
            let mut args = vec![
                group,
                artifact,
                "-f",
                source.to_str().unwrap(),
                "-o",
                out.to_str().unwrap(),
                "-t",
                format,
                "-K",
                "-q",
            ];
            if artifact == "outlook" {
                args.push("--properties");
            }
            run(&args);
        }
        let native = dir.path().join("native-timeline.jsonl");
        run(&[
            "super-timeline",
            "-f",
            json.to_str().unwrap(),
            "--input-artifact",
            artifact,
            "--computer",
            "TEST",
            "-o",
            native.to_str().unwrap(),
            "-t",
            "jsonl",
            "-K",
            "-q",
        ]);
        let a = events(&native);
        assert!(!a.is_empty(), "No timestamp events from {name}");
        let mut b = Vec::new();
        for (index, file) in fs::read_dir(&csv).unwrap().map(Result::unwrap).enumerate() {
            let output = dir.path().join(format!("table-{index}.jsonl"));
            run(&[
                "super-timeline",
                "-f",
                file.path().to_str().unwrap(),
                "--input-artifact",
                artifact,
                "--computer",
                "TEST",
                "-o",
                output.to_str().unwrap(),
                "-t",
                "jsonl",
                "-K",
                "-q",
            ]);
            b.extend(events(&output));
        }
        let ka = keys(&a);
        let kb = keys(&b);
        assert_eq!(ka, kb, "CSV/JSONL timestamp projection differs for {name}");
    }
}

#[test]
fn current_auto_parse_manifests_and_combined_native_records_support_new_parsers() {
    let root = tempfile::tempdir().unwrap();
    let evidence = root.path().join("Evidence");
    let windows = evidence.join("HOST/C/Windows");
    fs::create_dir_all(windows.join("System32/LogFiles/Sum")).unwrap();
    let user = evidence.join("HOST/C/Users/alice");
    let browser = user.join("AppData/Local/Google/Chrome/User Data/Default");
    fs::create_dir_all(&browser).unwrap();
    unpack(
        &repo("test_files/browser/chromium150/History.gz"),
        &browser.join("History"),
    );
    let webcache = user.join("AppData/Local/Microsoft/Windows/WebCache");
    fs::create_dir_all(&webcache).unwrap();
    unpack(
        &repo("test_files/webcache/plaso/WebCacheV01.dat.gz"),
        &webcache.join("WebCacheV01.dat"),
    );
    fs::create_dir_all(user.join("Documents")).unwrap();
    fs::copy(
        repo("test_files/outlook/apache-poi/msgClassAppointment.msg"),
        user.join("Documents/example.msg"),
    )
    .unwrap();
    for name in ["Current.mdb", "SystemIdentity.mdb"] {
        fs::copy(
            repo(&format!("test_files/ual/server2019/{name}")),
            windows.join("System32/LogFiles/Sum").join(name),
        )
        .unwrap();
    }
    let parsed = root.path().join("Parsed");
    let native = root.path().join("Native.jsonl");
    run(&[
        "auto-parse",
        "-d",
        evidence.to_str().unwrap(),
        "-o",
        parsed.to_str().unwrap(),
        "-j",
        native.to_str().unwrap(),
        "-T",
        "2",
        "-K",
        "-q",
    ]);
    let output = root.path().join("FromManifest.jsonl");
    run(&[
        "super-timeline",
        "-d",
        parsed.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-t",
        "jsonl",
        "-K",
        "-q",
    ]);
    let rows = events(&output);
    for artifact in ["browser", "outlook", "ual", "webcache"] {
        assert!(
            rows.iter().any(|row| row["Artifact"] == artifact),
            "No {artifact} events"
        );
    }
    let output = root.path().join("FromNative.jsonl");
    run(&[
        "super-timeline",
        "-f",
        native.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-t",
        "jsonl",
        "-K",
        "-q",
    ]);
    let rows = events(&output);
    for artifact in ["browser", "outlook", "ual", "webcache"] {
        assert!(
            rows.iter().any(|row| row["Artifact"] == artifact),
            "No {artifact} native events"
        );
    }
}

#[test]
fn portable_v2_manifest_resolves_a_unique_srum_source_without_using_the_cwd() {
    let root = tempfile::tempdir().unwrap();
    let evidence = root.path().join("Evidence/HOST/C/Windows/System32/sru");
    fs::create_dir_all(&evidence).unwrap();
    let db = evidence.join("SRUDB.dat");
    fs::write(&db, []).unwrap();
    let parsed = root.path().join("Exports/Parsed");
    fs::create_dir_all(parsed.join("HOST")).unwrap();
    fs::write(
        parsed.join("HOST/srum.csv"),
        "Timestamp,ID,BytesSent\n2026-10-20T08:00:00Z,1,42\n",
    )
    .unwrap();
    fs::write(parsed.join("manifest.json"),serde_json::to_vec(&serde_json::json!({
        "ManifestVersion":2,"OutputDirectory":"Parsed","EvidenceDirectory":"Evidence",
        "Hosts":[{"Comp":"HOST","EvidenceRoot":"./HOST/C","WindowsDirectories":["./HOST/C/Windows"]}],
        "Tasks":[{"Status":"succeeded","Purpose":"artifact","Artifact":"srum","Comp":"HOST",
            "OutputType":"csv","OutputFiles":["./HOST/srum.csv"],"Command":["juntei","parse-ese","srum","-d","./HOST/C"]}]
    })).unwrap()).unwrap();
    let output = root.path().join("timeline.jsonl");
    run(&[
        "super-timeline",
        "-d",
        parsed.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-t",
        "jsonl",
        "-K",
        "-q",
    ]);
    let rows = events(&output);
    assert!(!rows.is_empty());
    assert_eq!(
        rows[0]["SourceFile"],
        db.canonicalize().unwrap().to_string_lossy().as_ref()
    );
    assert_eq!(rows[0]["Details"]["RawSourceAvailable"], true);
}

#[test]
fn parsed_vhdx_and_e01_keep_image_provenance_without_the_staging_workspace() {
    for (kind, extension) in [("--vhdx", "vhdx"), ("--e01", "E01")] {
        let root = tempfile::tempdir().unwrap();
        let evidence = root.path().join("Evidence");
        fs::create_dir(&evidence).unwrap();
        let source = evidence.join(format!("sample.{extension}"));
        if kind == "--vhdx" {
            unpack(&repo("test_files/vhdx/sample.vhdx.gz"), &source);
        } else {
            for suffix in ["E01", "E02"] {
                fs::copy(
                    repo(&format!("test_files/e01/sample.{suffix}")),
                    evidence.join(format!("sample.{suffix}")),
                )
                .unwrap();
            }
        }
        let parsed = root.path().join("Parsed");
        run(&[
            "auto-parse",
            "-d",
            source.to_str().unwrap(),
            kind,
            "-o",
            parsed.to_str().unwrap(),
            "-T",
            "2",
            "-K",
            "-q",
        ]);
        let output = root.path().join("timeline.jsonl");
        run(&[
            "super-timeline",
            "-d",
            parsed.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-t",
            "jsonl",
            "-K",
            "-q",
        ]);
        let rows = events(&output);
        assert!(!rows.is_empty());
        let expected = format!("./sample.{extension}!/");
        for row in rows {
            assert!(
                row["SourceFile"].as_str().unwrap().starts_with(&expected),
                "{}",
                row["SourceFile"]
            );
        }
    }
}
