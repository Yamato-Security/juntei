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

//! `parse-file powershell-history` and its `auto-parse` task, on history files written by the
//! tests themselves (no collected data).

use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

/// Where PSReadLine keeps a user's history files, below the profile folder.
const PSREADLINE: &str = "AppData/Roaming/Microsoft/Windows/PowerShell/PSReadLine";
const CONSOLE: &str = "ConsoleHost_history.txt";
const VSCODE: &str = "Visual Studio Code Host_history.txt";
/// 2026-10-20T08:46:38Z, in seconds since 1970.
const MODIFIED: u64 = 1_792_485_998;
const MODIFIED_UTC: &str = "2026-10-20T08:46:38Z";

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

fn run(args: &[&str]) -> Run {
    run_at(Path::new(env!("CARGO_MANIFEST_DIR")), args)
}

fn history(args: &[&str]) -> Run {
    let mut all = vec!["parse-file", "powershell-history"];
    all.extend(args);
    all.extend(["-K", "-q"]);
    run(&all)
}

/// The JSONL rows of `parse-file powershell-history` with `args`.
fn rows(args: &[&str]) -> Vec<Value> {
    let mut all = args.to_vec();
    all.extend(["-t", "jsonl"]);
    let r = history(&all);
    assert_eq!(r.code, 0, "{}", r.stderr);
    jsonl(&r.stdout)
}

fn jsonl(text: &str) -> Vec<Value> {
    text.lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn csv_rows(path: &Path) -> (Vec<String>, Vec<HashMap<String, String>>) {
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

fn utf16(text: &str, big_endian: bool) -> Vec<u8> {
    std::iter::once(0xFEFFu16)
        .chain(text.encode_utf16())
        .flat_map(|unit| {
            if big_endian {
                unit.to_be_bytes()
            } else {
                unit.to_le_bytes()
            }
        })
        .collect()
}

/// Write `user`'s history file `name` under the volume folder `volume`, last modified at
/// [`MODIFIED`] plus `later` seconds.
fn write_history(volume: &Path, user: &str, name: &str, bytes: &[u8], later: u64) -> PathBuf {
    let folder = volume.join("Users").join(user).join(PSREADLINE);
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join(name);
    std::fs::write(&path, bytes).unwrap();
    set_modified(&path, later);
    path
}

fn set_modified(path: &Path, later: u64) {
    let time = SystemTime::UNIX_EPOCH + Duration::from_secs(MODIFIED + later);
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(time)
        .unwrap();
}

/// One volume with the histories of several users in every encoding, an empty one, a damaged
/// one, and files that are not PowerShell's.
fn collection(volume: &Path) {
    write_history(
        volume,
        "alice",
        CONSOLE,
        b"whoami\r\nGet-Process |`\r\n  Sort-Object CPU\r\n\r\nipconfig /all\r\n",
        0,
    );
    write_history(
        volume,
        "alice",
        VSCODE,
        "Write-Output 'héllo'\n".as_bytes(),
        60,
    );
    write_history(
        volume,
        "bob",
        CONSOLE,
        &utf16("Get-Date\r\necho 日本語 😀\r\n", false),
        3600,
    );
    write_history(
        volume,
        "carol",
        CONSOLE,
        &utf16("hostname\nnet user\n", true),
        0,
    );
    let mut marked = vec![0xEF, 0xBB, 0xBF];
    marked.extend(b"Get-ChildItem\n");
    write_history(volume, "dave", CONSOLE, &marked, 0);
    write_history(volume, "empty", CONSOLE, b"", 0);
    write_history(
        volume,
        "erin",
        CONSOLE,
        b"good\nbad \xff\xfe bytes\ncut \xe6\x97",
        0,
    );
    // Not in a PSReadLine folder, or not a history file's name: never read.
    let documents = volume.join("Users/alice/Documents");
    std::fs::create_dir_all(&documents).unwrap();
    std::fs::write(documents.join("bash_history.txt"), b"not powershell\n").unwrap();
    std::fs::write(documents.join(CONSOLE), b"a copy elsewhere\n").unwrap();
    std::fs::write(
        volume
            .join("Users/alice")
            .join(PSREADLINE)
            .join("notes.txt"),
        b"not a history\n",
    )
    .unwrap();
}

fn text<'a>(row: &'a Value, field: &str) -> &'a str {
    row[field].as_str().unwrap_or_default()
}

#[test]
fn every_users_history_is_read_with_relative_sources_and_line_numbers() {
    let dir = tempfile::tempdir().unwrap();
    let volume = dir.path().join("evidence/HOST1/C");
    collection(&volume);
    // An absolute input: where the evidence is on the analyst's system is in no row.
    let evidence = dir.path().join("evidence");
    let all = rows(&["-d", evidence.to_str().unwrap()]);
    let source = |user: &str, name: &str| format!("HOST1/C/Users/{user}/{PSREADLINE}/{name}");

    let found: Vec<(&str, &str, u64, u64, &str)> = all
        .iter()
        .map(|row| {
            (
                text(row, "User"),
                text(row, "ShellHost"),
                row["LineNumber"].as_u64().unwrap(),
                row["EntryStartLine"].as_u64().unwrap(),
                text(row, "Command"),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            ("alice", "ConsoleHost", 1, 1, "whoami"),
            ("alice", "ConsoleHost", 2, 2, "Get-Process |`"),
            ("alice", "ConsoleHost", 3, 2, "  Sort-Object CPU"),
            // Line 4 is empty: counted, not a row.
            ("alice", "ConsoleHost", 5, 5, "ipconfig /all"),
            (
                "alice",
                "Visual Studio Code Host",
                1,
                1,
                "Write-Output 'héllo'"
            ),
            ("bob", "ConsoleHost", 1, 1, "Get-Date"),
            ("bob", "ConsoleHost", 2, 2, "echo 日本語 😀"),
            ("carol", "ConsoleHost", 1, 1, "hostname"),
            ("carol", "ConsoleHost", 2, 2, "net user"),
            ("dave", "ConsoleHost", 1, 1, "Get-ChildItem"),
            ("erin", "ConsoleHost", 1, 1, "good"),
            ("erin", "ConsoleHost", 2, 2, "bad \u{fffd}\u{fffd} bytes"),
            ("erin", "ConsoleHost", 3, 3, "cut \u{fffd}"),
        ]
    );
    assert_eq!(text(&all[0], "SourceFile"), source("alice", CONSOLE));
    assert_eq!(text(&all[4], "SourceFile"), source("alice", VSCODE));
    assert_eq!(text(&all[5], "SourceFile"), source("bob", CONSOLE));
    let encodings: Vec<&str> = [0, 5, 7, 9, 10]
        .iter()
        .map(|&row| text(&all[row], "Encoding"))
        .collect();
    assert_eq!(
        encodings,
        ["UTF-8", "UTF-16LE", "UTF-16BE", "UTF-8-BOM", "UTF-8"]
    );

    // The only time is the file's own modification time, the same on every row of a file.
    for row in &all[..4] {
        assert_eq!(row["FileLastWriteTime"], MODIFIED_UTC);
        assert_eq!(row["FileSize"], 60);
    }
    assert_eq!(all[4]["FileLastWriteTime"], "2026-10-20T08:47:38Z");
    assert_eq!(all[5]["FileLastWriteTime"], "2026-10-20T09:46:38Z");
    for row in &all {
        let fields: Vec<&String> = row.as_object().unwrap().keys().collect();
        assert_eq!(
            fields,
            [
                "SourceFile",
                "User",
                "ShellHost",
                "LineNumber",
                "EntryStartLine",
                "Command",
                "FileLastWriteTime",
                "FileSize",
                "Encoding",
                "Problems"
            ]
        );
    }

    // Damage is on its line; the lines around it are untouched.
    assert_eq!(
        all[11]["Problems"],
        serde_json::json!(["2 invalid UTF-8 byte sequence(s) shown as U+FFFD"])
    );
    assert_eq!(all[12]["Problems"].as_array().unwrap().len(), 1);
    assert!(
        all.iter()
            .enumerate()
            .all(|(index, row)| matches!(index, 11 | 12)
                || row["Problems"].as_array().unwrap().is_empty())
    );

    // Nothing printed or saved names the analyst's directory.
    let root = dir.path().to_string_lossy().into_owned();
    let canonical = dir
        .path()
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    for output_type in ["csv", "json", "jsonl"] {
        let out = dir.path().join(format!("out/history.{output_type}"));
        // Given as a relative path: the summary names the output as it was given.
        let r = run_at(
            dir.path(),
            &[
                "parse-file",
                "powershell-history",
                "-d",
                evidence.to_str().unwrap(),
                "-o",
                &format!("out/history.{output_type}"),
                "-t",
                output_type,
                "-K",
                "-q",
            ],
        );
        assert_eq!(r.code, 0, "{}", r.all());
        let saved = std::fs::read_to_string(&out).unwrap();
        for needle in [&root, &canonical] {
            assert!(!saved.contains(needle.as_str()), "{output_type}");
        }
        for line in r.all().lines() {
            // The input directory is named once, as the analyst gave it; no source file is.
            assert!(
                !line.contains(PSREADLINE) || !line.contains(root.as_str()),
                "{line}"
            );
        }
        assert!(
            r.stdout.contains("PowerShell history files: 7"),
            "{}",
            r.stdout
        );
        assert!(r.stdout.contains("History files read: 7"), "{}", r.stdout);
        assert!(
            r.stdout
                .contains("Command lines: 13 (commands: 12, continuation lines: 1)"),
            "{}",
            r.stdout
        );
        assert!(r.stdout.contains("Empty lines skipped: 1"), "{}", r.stdout);
        assert!(
            r.stdout
                .contains("Lines by user: alice: 5, bob: 2, carol: 2, dave: 1, empty: 0, …"),
            "{}",
            r.stdout
        );
        assert!(r.stdout.contains("With problems: 2"), "{}", r.stdout);
        assert!(
            r.stdout.contains("Results saved: out/history."),
            "{}",
            r.stdout
        );
    }
    let (header, csv) = csv_rows(&dir.path().join("out/history.csv"));
    assert_eq!(
        header,
        [
            "SourceFile",
            "User",
            "ShellHost",
            "LineNumber",
            "EntryStartLine",
            "Command",
            "FileLastWriteTime",
            "FileSize",
            "Encoding",
            "Problems"
        ]
    );
    assert_eq!(csv.len(), all.len());
    for (row, json) in csv.iter().zip(&all) {
        assert_eq!(row["SourceFile"], text(json, "SourceFile"));
        assert_eq!(row["User"], text(json, "User"));
        assert_eq!(row["LineNumber"], json["LineNumber"].to_string());
        assert_eq!(row["Command"], text(json, "Command"));
        assert_eq!(row["FileLastWriteTime"], text(json, "FileLastWriteTime"));
    }
    // Pretty JSON holds the same records.
    let pretty: Vec<Value> = serde_json::Deserializer::from_str(
        &std::fs::read_to_string(dir.path().join("out/history.json")).unwrap(),
    )
    .into_iter()
    .map(Result::unwrap)
    .collect();
    assert_eq!(pretty, all);
}

#[test]
fn a_single_file_is_named_without_its_folders() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_history(dir.path(), "frank", CONSOLE, b"Get-Help\n", 0);
    let all = rows(&["-f", path.to_str().unwrap()]);
    assert_eq!(all.len(), 1);
    assert_eq!(all[0]["SourceFile"], CONSOLE);
    // The profile layout PSReadLine uses names the user.
    assert_eq!(all[0]["User"], "frank");
    assert_eq!(all[0]["FileLastWriteTime"], MODIFIED_UTC);

    // A copy under any other name and folder: read, without a user or a host.
    let copy = dir.path().join("Users/analyst/Desktop/export.txt");
    std::fs::create_dir_all(copy.parent().unwrap()).unwrap();
    std::fs::write(&copy, b"Get-Help\n").unwrap();
    let all = rows(&["-f", copy.to_str().unwrap()]);
    assert_eq!(all[0]["SourceFile"], "export.txt");
    assert!(all[0]["User"].is_null());
    assert!(all[0]["ShellHost"].is_null());
    assert_eq!(all[0]["Command"], "Get-Help");

    // Without -o and -t only the summary is printed.
    let r = history(&["-f", path.to_str().unwrap()]);
    assert_eq!(r.code, 0, "{}", r.all());
    assert!(r.stdout.contains("Command lines: 1"), "{}", r.stdout);
    assert!(r.stdout.contains("Lines by user: frank: 1"), "{}", r.stdout);
    assert!(
        r.stdout.contains("To save every record, add -o <FILE>"),
        "{}",
        r.stdout
    );
}

#[test]
fn a_folder_kept_in_one_file_name_is_normalized() {
    // A path with backslashes is one file name outside Windows, and nested folders on it:
    // either way the source is written with `/` and the user is found.
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("flat");
    std::fs::create_dir_all(&folder).unwrap();
    let name = format!(
        "C\\Users\\grace\\{}\\{CONSOLE}",
        PSREADLINE.replace('/', "\\")
    );
    let path = folder.join(&name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, b"Get-Service\n").unwrap();
    let all = rows(&["-d", folder.to_str().unwrap()]);
    assert_eq!(all.len(), 1);
    assert_eq!(
        all[0]["SourceFile"],
        format!("C/Users/grace/{PSREADLINE}/{CONSOLE}")
    );
    assert_eq!(all[0]["User"], "grace");
    assert_eq!(all[0]["ShellHost"], "ConsoleHost");
}

#[test]
fn formula_cells_are_guarded_in_csv_and_unchanged_in_json() {
    let dir = tempfile::tempdir().unwrap();
    let commands = [
        "=cmd|' /C calc'!A0",
        "+1+cmd|' /C calc'!A0",
        "-2+3",
        "@SUM(1+1)",
        "\t=1+1",
        "  =HYPERLINK(\"http://example.test\")",
        "\u{feff}=1",
        "＝1+1",
        "\tGet-Date",
        "a,b;\"quoted\" = fine",
        "'=typed with an apostrophe",
        "Get-Content -Path x",
    ];
    // A user folder and a host name are evidence too.
    let path = write_history(
        dir.path(),
        "=mallory",
        "@host_history.txt",
        // The first line starts after a real byte order mark, so the one in the list is text.
        format!("\u{feff}{}\n", commands.join("\n")).as_bytes(),
        0,
    );
    assert!(path.is_file());
    let out = dir.path().join("out.csv");
    let r = history(&[
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    let (_, csv) = csv_rows(&out);
    let cells: Vec<&str> = csv.iter().map(|row| row["Command"].as_str()).collect();
    assert_eq!(
        cells,
        [
            "'=cmd|' /C calc'!A0",
            "'+1+cmd|' /C calc'!A0",
            "'-2+3",
            "'@SUM(1+1)",
            "'\t=1+1",
            "'  =HYPERLINK(\"http://example.test\")",
            "'\u{feff}=1",
            "'＝1+1",
            "'\tGet-Date",
            "a,b;\"quoted\" = fine",
            "'=typed with an apostrophe",
            "Get-Content -Path x",
        ]
    );
    for row in &csv {
        assert_eq!(row["User"], "'=mallory");
        assert_eq!(row["ShellHost"], "'@host");
        assert!(row["SourceFile"].starts_with("Users/=mallory/"));
    }
    // No cell of the file starts with a character a spreadsheet takes for a formula.
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(&out)
        .unwrap();
    for record in reader.records() {
        for cell in record.unwrap().iter() {
            let shown = cell.trim_start_matches(|c: char| c.is_whitespace() || c.is_control());
            assert!(!shown.starts_with(['=', '+', '-', '@', '＝']), "{cell:?}");
            assert!(!cell.starts_with(['\t', '\r', '\n']), "{cell:?}");
        }
    }

    for output_type in ["jsonl", "json"] {
        let r = history(&["-d", dir.path().to_str().unwrap(), "-t", output_type]);
        assert_eq!(r.code, 0, "{}", r.stderr);
        let records: Vec<Value> = serde_json::Deserializer::from_str(&r.stdout)
            .into_iter()
            .map(Result::unwrap)
            .collect();
        let original: Vec<&str> = records.iter().map(|row| text(row, "Command")).collect();
        assert_eq!(original, commands, "{output_type}");
        assert!(records.iter().all(|row| row["User"] == "=mallory"));
        assert!(records.iter().all(|row| row["ShellHost"] == "@host"));
    }
}

#[test]
fn empty_histories_give_a_header_and_no_rows() {
    let dir = tempfile::tempdir().unwrap();
    write_history(dir.path(), "empty", CONSOLE, b"", 0);
    write_history(dir.path(), "mark", CONSOLE, &[0xFF, 0xFE], 0);
    write_history(dir.path(), "blank", CONSOLE, b"\r\n\r\n\n", 0);
    let input = dir.path().to_str().unwrap();
    let out = dir.path().join("out.csv");
    let r = history(&["-d", input, "-o", out.to_str().unwrap()]);
    assert_eq!(r.code, 0, "{}", r.all());
    assert!(r.stdout.contains("History files read: 3"), "{}", r.stdout);
    assert!(
        r.stdout
            .contains("Command lines: 0 (commands: 0, continuation lines: 0)"),
        "{}",
        r.stdout
    );
    assert!(r.stdout.contains("Empty lines skipped: 3"), "{}", r.stdout);
    assert_eq!(
        std::fs::read_to_string(&out).unwrap(),
        "SourceFile,User,ShellHost,LineNumber,EntryStartLine,Command,FileLastWriteTime,FileSize,Encoding,Problems\n"
    );
    let r = history(&["-d", input, "-t", "jsonl"]);
    assert_eq!((r.code, r.stdout.as_str()), (0, ""), "{}", r.stderr);
}

#[test]
fn input_and_output_errors() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing");
    let r = history(&["-d", missing.to_str().unwrap()]);
    assert_eq!(r.code, 1);
    assert!(
        r.all().contains("The directory does not exist"),
        "{}",
        r.all()
    );
    let r = history(&["-f", missing.to_str().unwrap()]);
    assert_eq!(r.code, 1);
    assert!(r.all().contains("The file does not exist"), "{}", r.all());

    // Only files that are not PowerShell's: nothing to parse, and nothing is created.
    let other = dir.path().join("other/Users/alice/Documents");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join(CONSOLE), b"elsewhere\n").unwrap();
    let out = dir.path().join("none.csv");
    let r = history(&[
        "-d",
        dir.path().join("other").to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.all()
            .contains("No PowerShell history files (PSReadLine\\*_history.txt) were found in "),
        "{}",
        r.all()
    );
    assert!(!out.exists());

    // -f and -d exclude each other, and one of them is needed.
    let both = history(&["-d", ".", "-f", "x"]);
    assert_ne!(both.code, 0);
    assert_ne!(history(&[]).code, 0);

    // On stdout nothing but the records is printed; messages go to stderr.
    let path = write_history(dir.path(), "alice", CONSOLE, b"whoami\n", 0);
    let r = history(&["-f", path.to_str().unwrap(), "-t", "csv"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(
        r.stdout,
        format!(
            "SourceFile,User,ShellHost,LineNumber,EntryStartLine,Command,FileLastWriteTime,FileSize,Encoding,Problems\n\
             {CONSOLE},alice,ConsoleHost,1,1,whoami,{MODIFIED_UTC},7,UTF-8,\n"
        )
    );
    assert_eq!(r.stderr, "");
}

#[test]
fn existing_outputs_and_aliases_of_the_input_are_not_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let original = b"whoami\nhostname\n";
    let path = write_history(dir.path(), "alice", CONSOLE, original, 0);
    let input = path.to_str().unwrap();

    let out = dir.path().join("out.csv");
    std::fs::write(&out, b"earlier results").unwrap();
    let r = history(&["-f", input, "-o", out.to_str().unwrap()]);
    assert_eq!(r.code, 1);
    assert!(r.all().contains("already exists"), "{}", r.all());
    assert_eq!(std::fs::read(&out).unwrap(), b"earlier results");
    let r = history(&["-f", input, "-o", out.to_str().unwrap(), "-C"]);
    assert_eq!(r.code, 0, "{}", r.all());
    assert_eq!(csv_rows(&out).1.len(), 2);

    // --clobber needs an output, and a directory is not one.
    assert_ne!(history(&["-f", input, "-C"]).code, 0);
    let r = history(&["-f", input, "-o", dir.path().to_str().unwrap(), "-C"]);
    assert_eq!(r.code, 1);
    assert!(r.all().contains("is a directory"), "{}", r.all());

    // The history file itself, also with --clobber, for one file and for a directory.
    for args in [
        vec!["-f", input, "-o", input, "-C"],
        vec!["-d", dir.path().to_str().unwrap(), "-o", input, "-C"],
    ] {
        let r = history(&args);
        assert_eq!(r.code, 1, "{}", r.all());
        assert!(
            r.all()
                .contains("The output file is also an input file and cannot be overwritten"),
            "{}",
            r.all()
        );
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }

    #[cfg(unix)]
    {
        let link = dir.path().join("link.csv");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        let hard = dir.path().join("hard.csv");
        std::fs::hard_link(&path, &hard).unwrap();
        for alias in [&link, &hard] {
            for extra in [&[][..], &["-t", "jsonl"][..]] {
                let mut args = vec!["-f", input, "-o", alias.to_str().unwrap(), "-C"];
                args.extend(extra);
                let r = history(&args);
                assert_eq!(r.code, 1, "{}", r.all());
                assert!(
                    r.all().contains("The output file is also an input file"),
                    "{}",
                    r.all()
                );
                assert_eq!(std::fs::read(&path).unwrap(), original);
            }
        }
        // A symbolic link to a history file in a PSReadLine folder is read as the file is.
        let linked = dir.path().join("linked/Users/heidi").join(PSREADLINE);
        std::fs::create_dir_all(&linked).unwrap();
        std::os::unix::fs::symlink(&path, linked.join(CONSOLE)).unwrap();
        // A linked directory is not followed (a loop would never end).
        std::os::unix::fs::symlink(dir.path(), dir.path().join("linked/loop")).unwrap();
        let all = rows(&["-d", dir.path().join("linked").to_str().unwrap()]);
        assert_eq!(all.len(), 2);
        assert_eq!(all[0]["User"], "heidi");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn a_full_output_device_is_a_write_error_not_a_parse_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_history(dir.path(), "alice", CONSOLE, b"whoami\nhostname\n", 0);
    for output_type in ["csv", "jsonl"] {
        let r = history(&[
            "-f",
            path.to_str().unwrap(),
            "-o",
            "/dev/full",
            "-t",
            output_type,
            "-C",
        ]);
        assert_eq!(r.code, 1, "{}", r.all());
        assert!(
            r.all().contains("Could not write the records: "),
            "{}",
            r.all()
        );
        assert!(!r.all().contains("Could not read"), "{}", r.all());
        assert!(!r.all().contains("Results saved"), "{}", r.all());
    }
}

#[cfg(unix)]
#[test]
fn an_unreadable_history_does_not_stop_the_others() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let locked = write_history(dir.path(), "alice", CONSOLE, b"secret\n", 0);
    write_history(dir.path(), "bob", CONSOLE, b"whoami\n", 0);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
    if std::fs::File::open(&locked).is_ok() {
        // An administrator can read it anyway.
        return;
    }
    let out = dir.path().join("out.csv");
    let r = history(&[
        "-d",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(r.code, 0, "{}", r.all());
    assert!(
        r.stdout.contains(&format!(
            "Could not read Users/alice/{PSREADLINE}/{CONSOLE}: "
        )),
        "{}",
        r.stdout
    );
    assert!(
        r.stdout
            .contains("History files read: 1 (could not read: 1)"),
        "{}",
        r.stdout
    );
    let (_, csv) = csv_rows(&out);
    assert_eq!(csv.len(), 1);
    assert_eq!(csv[0]["User"], "bob");
}

#[test]
fn help_names_the_command_and_its_options() {
    let group = run(&["parse-file", "--help"]);
    assert_eq!(group.code, 0, "{}", group.stderr);
    assert!(
        group.stdout.contains("powershell-history"),
        "{}",
        group.stdout
    );
    for help in ["-h", "--help"] {
        let leaf = run(&["parse-file", "powershell-history", help]);
        assert_eq!(leaf.code, 0, "{}", leaf.stderr);
        assert!(
            leaf.stdout
                .contains("juntei parse-file powershell-history <INPUT> [OPTIONS]"),
            "{}",
            leaf.stdout
        );
        for option in [
            "--directory",
            "--file",
            "--output",
            "--output-type",
            "--clobber",
        ] {
            assert!(leaf.stdout.contains(option), "{}", leaf.stdout);
        }
        assert!(!leaf.stdout.contains("image-source-metadata"));
    }
}

/// Two collected computers: HOST1 with two users' histories and Prefetch files (which do have
/// times), HOST2 with one user's history, HOST3 without any.
fn evidence(root: &Path) -> PathBuf {
    let evidence = root.join("evidence");
    for host in ["HOST1", "HOST2", "HOST3"] {
        std::fs::create_dir_all(evidence.join(host).join("C/Windows/System32")).unwrap();
    }
    let one = evidence.join("HOST1/C");
    write_history(
        &one,
        "alice",
        CONSOLE,
        // A command that looks like a time, and one a spreadsheet would run.
        b"whoami\r\nGet-Date 2026-10-20T08:46:38Z\r\n=cmd|' /C calc'!A0\r\n",
        0,
    );
    write_history(
        &one,
        "bob",
        VSCODE,
        &utf16("Get-Process |`\n  Out-Null\n", false),
        3600,
    );
    write_history(
        &evidence.join("HOST2/C"),
        "carol",
        CONSOLE,
        b"hostname\n",
        0,
    );
    let prefetch = one.join("Windows/Prefetch");
    std::fs::create_dir_all(&prefetch).unwrap();
    let samples = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/prefetch");
    for name in ["CMD.EXE-6D6290C5.pf", "CONSENT.EXE-531BD9EA.pf"] {
        std::fs::copy(samples.join(name), prefetch.join(name)).unwrap();
    }
    evidence
}

fn manifest(output: &Path) -> Value {
    serde_json::from_reader(std::fs::File::open(output.join("manifest.json")).unwrap()).unwrap()
}

fn tasks<'a>(manifest: &'a Value, artifact: &str) -> Vec<&'a Value> {
    manifest["Tasks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|task| task["Artifact"] == artifact)
        .collect()
}

/// No file under `output` names one of `directories`.
fn assert_no_absolute_paths(output: &Path, directories: &[&Path]) {
    let mut needles = Vec::new();
    for directory in directories {
        needles.push(directory.to_string_lossy().into_owned());
        needles.push(
            directory
                .canonicalize()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
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
            let content = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
            for needle in &needles {
                assert!(
                    !content.contains(needle.as_str()),
                    "{} names {needle}",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn auto_parse_saves_histories_under_shell_history_and_keeps_them_off_the_timelines() {
    let dir = tempfile::tempdir().unwrap();
    let evidence = evidence(dir.path());
    let output = dir.path().join("parsed");
    let timeline = dir.path().join("timeline.jsonl");
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-j",
        timeline.to_str().unwrap(),
        "-Z",
        "+09:00",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());

    // --- the category output -------------------------------------------------------------
    for host in ["HOST1", "HOST2", "HOST3"] {
        assert!(output.join(host).join("ShellHistory").is_dir(), "{host}");
    }
    let (header, rows) = csv_rows(&output.join("HOST1/ShellHistory/PowerShellHistory.csv"));
    assert_eq!(header[0], "SourceFile");
    let found: Vec<(&str, &str, &str, &str, &str)> = rows
        .iter()
        .map(|row| {
            (
                row["User"].as_str(),
                row["ShellHost"].as_str(),
                row["LineNumber"].as_str(),
                row["EntryStartLine"].as_str(),
                row["Command"].as_str(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            ("alice", "ConsoleHost", "1", "1", "whoami"),
            // Typed text that looks like a time is not a time: the output timezone leaves it.
            (
                "alice",
                "ConsoleHost",
                "2",
                "2",
                "Get-Date 2026-10-20T08:46:38Z"
            ),
            ("alice", "ConsoleHost", "3", "3", "'=cmd|' /C calc'!A0"),
            ("bob", "Visual Studio Code Host", "1", "1", "Get-Process |`"),
            ("bob", "Visual Studio Code Host", "2", "1", "  Out-Null"),
        ]
    );
    // The file time is shown in the output timezone, converted from UTC exactly once.
    for row in &rows[..3] {
        assert_eq!(row["FileLastWriteTime"], "2026-10-20T17:46:38+09:00");
        assert_eq!(
            row["SourceFile"],
            format!("Users/alice/{PSREADLINE}/{CONSOLE}")
        );
    }
    assert_eq!(rows[3]["FileLastWriteTime"], "2026-10-20T18:46:38+09:00");
    let (_, second) = csv_rows(&output.join("HOST2/ShellHistory/PowerShellHistory.csv"));
    assert_eq!(second.len(), 1);
    assert_eq!(
        (second[0]["User"].as_str(), second[0]["Command"].as_str()),
        ("carol", "hostname")
    );
    assert!(
        !output
            .join("HOST3/ShellHistory/PowerShellHistory.csv")
            .exists()
    );

    // --- manifests and logs ---------------------------------------------------------------
    let manifest = manifest(&output);
    assert_eq!(manifest["OutputTimeZone"], "+09:00");
    let history = tasks(&manifest, "powershell-history");
    // One task per computer, and none that exports to the JSONL timeline.
    assert_eq!(history.len(), 3);
    for task in &history {
        assert_eq!(task["Category"], "ShellHistory");
        assert_eq!(task["Purpose"], "artifact");
        assert_eq!(task["OutputType"], "csv");
        let command: Vec<&str> = task["Command"]
            .as_array()
            .unwrap()
            .iter()
            .map(|arg| arg.as_str().unwrap())
            .collect();
        assert_eq!(command[1..3], ["parse-file", "powershell-history"]);
        let host = task["Comp"].as_str().unwrap();
        // The parser is given the computer's evidence root, which SourceFile is relative to.
        assert_eq!(command[3..5], ["-d", &format!("./{host}/C")]);
        assert_eq!(
            task["Log"],
            format!("./{host}/ShellHistory/PowerShellHistory-logs.txt")
        );
        assert!(output.join(task["Log"].as_str().unwrap()).is_file());
        if host == "HOST3" {
            assert_eq!(task["Status"], "skipped");
            assert!(task["Output"].is_null());
        } else {
            assert_eq!(task["Status"], "succeeded", "{task}");
            assert_eq!(
                task["Output"],
                format!("./{host}/ShellHistory/PowerShellHistory.csv")
            );
        }
    }
    // Other parsers do get a timeline task: the exclusion is this artifact's alone.
    assert!(
        tasks(&manifest, "prefetch")
            .iter()
            .any(|task| task["Purpose"] == "json_timeline")
    );
    let (_, index) = csv_rows(&output.join("manifest.csv"));
    let listed: Vec<_> = index
        .iter()
        .filter(|row| row["Artifact"] == "powershell-history")
        .collect();
    assert_eq!(listed.len(), 3);
    assert!(listed.iter().all(|row| row["Category"] == "ShellHistory"
        && row["Purpose"] == "artifact"
        && row["OutputTimeZone"] == "+09:00"));
    let log = std::fs::read_to_string(output.join("HOST1/ShellHistory/PowerShellHistory-logs.txt"))
        .unwrap();
    assert!(
        log.starts_with("Command: juntei parse-file powershell-history -d ./HOST1/C -o ./HOST1/ShellHistory/PowerShellHistory.csv -t csv"),
        "{log}"
    );
    assert!(log.contains("History files read: 2"), "{log}");
    assert_no_absolute_paths(&output, &[dir.path()]);

    // --- the timelines ----------------------------------------------------------------------
    let events = std::fs::read_to_string(&timeline).unwrap();
    assert!(events.contains("\"Artifact\":\"prefetch\""), "{events}");
    for needle in [
        "powershell-history",
        "whoami",
        "hostname",
        "Out-Null",
        PSREADLINE,
    ] {
        assert!(!events.contains(needle), "{needle}");
    }
    let super_timeline = dir.path().join("super.jsonl");
    let r = run(&[
        "super-timeline",
        "-d",
        output.to_str().unwrap(),
        "-o",
        super_timeline.to_str().unwrap(),
        "-t",
        "jsonl",
        "-K",
        "-q",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    let events = std::fs::read_to_string(&super_timeline).unwrap();
    assert!(events.contains("prefetch"), "{events}");
    for needle in [
        "powershell-history",
        "PowerShellHistory",
        "whoami",
        PSREADLINE,
    ] {
        assert!(!events.contains(needle), "{needle}");
    }
    // Asked for by name, the history is refused rather than dated by its file's time.
    let csv = output.join("HOST1/ShellHistory/PowerShellHistory.csv");
    let refused = dir.path().join("refused.csv");
    for args in [
        vec![
            "super-timeline",
            "-f",
            csv.to_str().unwrap(),
            "--input-artifact",
            "powershell-history",
            "-o",
            refused.to_str().unwrap(),
        ],
        vec![
            "super-timeline",
            "-d",
            output.to_str().unwrap(),
            "--artifact",
            "powershell-history",
            "-o",
            refused.to_str().unwrap(),
        ],
    ] {
        let r = run(&args);
        assert_ne!(r.code, 0, "{}", r.all());
        assert!(
            r.all().contains(
                "'powershell-history' records have no time of their own and are not timeline events"
            ),
            "{}",
            r.all()
        );
        assert!(!refused.exists());
    }
}

#[test]
fn auto_parse_json_outputs_keep_the_commands_and_stay_off_the_timeline() {
    let dir = tempfile::tempdir().unwrap();
    evidence(dir.path());
    for output_type in ["jsonl", "json"] {
        let output = dir.path().join(format!("parsed-{output_type}"));
        let timeline = dir.path().join(format!("timeline-{output_type}.jsonl"));
        // A relative evidence directory, as an analyst in the case folder gives it.
        let r = run_at(
            dir.path(),
            &[
                "auto-parse",
                "-d",
                "evidence",
                "-o",
                output.to_str().unwrap(),
                "-t",
                output_type,
                "-j",
                timeline.to_str().unwrap(),
                "-Z",
                "-05:00",
                "-K",
                "-q",
            ],
        );
        assert_eq!(r.code, 0, "{}", r.all());
        let saved = std::fs::read_to_string(output.join(format!(
            "HOST1/ShellHistory/PowerShellHistory.{output_type}"
        )))
        .unwrap();
        let records: Vec<Value> = serde_json::Deserializer::from_str(&saved)
            .into_iter()
            .map(Result::unwrap)
            .collect();
        assert_eq!(records.len(), 5);
        for record in &records {
            assert_eq!(record["Comp"], "HOST1");
            assert_eq!(record["Artifact"], "powershell-history");
        }
        // The command as typed: no added apostrophe, and no converted time.
        assert_eq!(records[1]["Command"], "Get-Date 2026-10-20T08:46:38Z");
        assert_eq!(records[2]["Command"], "=cmd|' /C calc'!A0");
        assert_eq!(records[0]["FileLastWriteTime"], "2026-10-20T03:46:38-05:00");
        assert_eq!(records[3]["FileLastWriteTime"], "2026-10-20T04:46:38-05:00");
        assert_eq!(records[4]["EntryStartLine"], 1);
        assert_eq!(
            records[0]["SourceFile"],
            format!("Users/alice/{PSREADLINE}/{CONSOLE}")
        );

        let manifest = manifest(&output);
        let history = tasks(&manifest, "powershell-history");
        assert_eq!(history.len(), 3);
        assert!(history.iter().all(|task| task["Purpose"] == "artifact"));
        let events = std::fs::read_to_string(&timeline).unwrap();
        assert!(events.contains("\"Artifact\":\"prefetch\""), "{events}");
        for needle in ["powershell-history", "whoami", PSREADLINE] {
            assert!(!events.contains(needle), "{output_type}: {needle}");
        }
        assert_no_absolute_paths(&output, &[dir.path()]);

        // The saved records, given to super-timeline as a file, are refused too.
        let refused = dir.path().join(format!("refused-{output_type}.csv"));
        let r = run(&[
            "super-timeline",
            "-f",
            output
                .join(format!(
                    "HOST1/ShellHistory/PowerShellHistory.{output_type}"
                ))
                .to_str()
                .unwrap(),
            "-o",
            refused.to_str().unwrap(),
        ]);
        assert_ne!(r.code, 0, "{}", r.all());
        assert!(r.all().contains("are not timeline events"), "{}", r.all());
    }
}

#[test]
fn auto_parse_clobber_replaces_an_earlier_history_output() {
    let dir = tempfile::tempdir().unwrap();
    let evidence = evidence(dir.path());
    let output = dir.path().join("parsed");
    let args = [
        "auto-parse",
        "-d",
        evidence.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-K",
        "-q",
    ];
    assert_eq!(run(&args).code, 0);
    let csv = output.join("HOST2/ShellHistory/PowerShellHistory.csv");
    assert_eq!(csv_rows(&csv).1.len(), 1);
    // Without a timezone the time stays UTC.
    assert_eq!(csv_rows(&csv).1[0]["FileLastWriteTime"], MODIFIED_UTC);
    let refused = run(&args);
    assert_eq!(refused.code, 1, "{}", refused.all());

    // The history is gone from the evidence: its earlier output must not survive the rerun.
    std::fs::remove_file(
        evidence
            .join("HOST2/C/Users/carol")
            .join(PSREADLINE)
            .join(CONSOLE),
    )
    .unwrap();
    let mut again = args.to_vec();
    again.push("-C");
    assert_eq!(run(&again).code, 0);
    assert!(!csv.exists());
    let manifest = manifest(&output);
    let second = tasks(&manifest, "powershell-history")
        .into_iter()
        .find(|task| task["Comp"] == "HOST2")
        .unwrap();
    assert_eq!(second["Status"], "skipped");
}
