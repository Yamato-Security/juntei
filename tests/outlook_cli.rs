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

//! `parse-file outlook` and its `auto-parse` task, on the public PST, OST, `.msg` and `.oft` files
//! of test_files/outlook: libyal's `outlook.pst`, java-libpst's `dist-list.pst` and Outlook 2013
//! OST, and `.msg` and `.oft` files of msgreader, Apache POI and Aspose.Email.

use flate2::read::GzDecoder;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::{Read, Write};
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
    run_in(None, args)
}

fn run_in(directory: Option<&Path>, args: &[&str]) -> Run {
    let mut command = Command::new(env!("CARGO_BIN_EXE_juntei"));
    if let Some(directory) = directory {
        command.current_dir(directory);
    }
    let out = command.args(args).output().expect("juntei runs");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    }
}

const OST: &str = "C/Users/alice/AppData/Local/Microsoft/Outlook";
const PST: &str = "C/Users/bob/Documents/Outlook Files";
const MSG: &str = "C/Users/alice/Desktop";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files/outlook")
        .join(name)
}

/// test_files/outlook/`name`, decompressed when it ends in `.gz`, as `directory`/`as_name`.
fn unpack(name: &str, directory: &Path, as_name: &str) -> PathBuf {
    std::fs::create_dir_all(directory).unwrap();
    let mut data = Vec::new();
    if name.ends_with(".gz") {
        GzDecoder::new(std::fs::File::open(fixture(name)).unwrap())
            .read_to_end(&mut data)
            .unwrap();
    } else {
        data = std::fs::read(fixture(name)).unwrap();
    }
    let path = directory.join(as_name);
    std::fs::write(&path, data).unwrap();
    path
}

/// A host with alice's OST and her saved messages, bob's two PSTs, and Outlook's other files,
/// which aren't read.
fn collection(root: &Path) -> PathBuf {
    let host = root.join("HOST1");
    unpack(
        "java-libpst/example-2013.ost.gz",
        &host.join(OST),
        "example-2013.ost",
    );
    unpack("libyal/outlook.pst.gz", &host.join(PST), "outlook.pst");
    unpack(
        "java-libpst/dist-list.pst.gz",
        &host.join(PST),
        "dist-list.pst",
    );
    for name in [
        "msgreader-hiraoka/msgInMsgInMsg.msg",
        "msgreader-hiraoka/contactAnsi.msg",
        "apache-poi/msgClassTask.msg",
        "apache-poi/msgClassAppointment.msg",
        "aspose-email-java/mapiToOft.oft",
    ] {
        let file = Path::new(name).file_name().unwrap().to_str().unwrap();
        unpack(name, &host.join(MSG), file);
    }
    // not read yet
    for name in [
        "Stream_Autocomplete_0_1234.dat",
        "Outlook.nk2",
        "Offline.oab",
    ] {
        std::fs::write(host.join(OST).join(name), b"not an item").unwrap();
    }
    std::fs::create_dir_all(host.join("C/Windows/System32")).unwrap();
    host
}

type Rows = Vec<BTreeMap<String, String>>;

fn csv_rows(path: &Path) -> (Vec<String>, Rows) {
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

/// The rows of a file, by the end of its path.
fn of<'a>(rows: &'a Rows, file: &str) -> Vec<&'a BTreeMap<String, String>> {
    rows.iter()
        .filter(|r| r["SourceFile"].replace('\\', "/").ends_with(file))
        .collect()
}

fn table_counts(rows: &[Value]) -> BTreeMap<String, usize> {
    rows.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts
            .entry(row["Table"].as_str().unwrap().to_string())
            .or_default() += 1;
        counts
    })
}

/// The rows of the eight files, without properties, by table.
fn expected_counts() -> BTreeMap<String, usize> {
    BTreeMap::from([
        ("Attachments".to_string(), 28),
        ("Calendar".to_string(), 2),
        ("Contacts".to_string(), 3),
        ("Files".to_string(), 8),
        ("Folders".to_string(), 92),
        ("Items".to_string(), 35),
        ("Recipients".to_string(), 8),
        ("Tasks".to_string(), 1),
    ])
}

#[test]
fn outlook_summary_on_the_terminal() {
    let evidence = tempfile::tempdir().unwrap();
    let host = collection(evidence.path());
    let r = run(&["parse-file", "outlook", "-d", host.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 0, "{}", r.all());
    for line in [
        "Outlook files: 8",
        "Outlook files parsed: 8",
        "PST and OST stores: 3",
        "MSG and OFT files: 5",
        "Users: 2",
        "  alice",
        "  bob",
        "Folders: 92",
        "Items: 14",
        "Hidden items (folder associated information): 17",
        "Embedded items: 4",
        "Recipients: 8",
        "Attachments: 28",
        "Contacts: 3",
        "Calendar items: 2",
        "Tasks: 1",
        "Submit and delivery times: 2007-04-29T22:27:33.593Z - 2022-07-20T17:36:38.015Z",
        "Senders with the most items:",
        "         2  olteam@microsoft.com",
        "Problems: 0",
        "To save every row, add -o <DIR> (CSV tables) or -o <FILE> -t json or jsonl.",
    ] {
        assert!(
            r.stdout.lines().any(|l| l == line),
            "{line:?}\n{}",
            r.stdout
        );
    }
    assert!(!r.stdout.contains("Properties: "), "{}", r.stdout);
}

#[test]
fn outlook_csv_tables() {
    let evidence = tempfile::tempdir().unwrap();
    let host = collection(evidence.path());
    let out = evidence.path().join("csv");
    let r = run(&[
        "parse-file",
        "outlook",
        "-d",
        host.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    assert!(r.stdout.contains("(8 CSV files)"), "{}", r.stdout);
    let mut names: Vec<String> = std::fs::read_dir(&out)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "Outlook-Attachments.csv",
            "Outlook-Calendar.csv",
            "Outlook-Contacts.csv",
            "Outlook-Files.csv",
            "Outlook-Folders.csv",
            "Outlook-Items.csv",
            "Outlook-Recipients.csv",
            "Outlook-Tasks.csv"
        ]
    );
    for (table, count) in expected_counts() {
        let (header, rows) = csv_rows(&out.join(format!("Outlook-{table}.csv")));
        assert_eq!(rows.len(), count, "{table}");
        assert_eq!(&header[..2], ["SourceFile", "User"], "{table}");
    }

    // --- files ---
    let (_, files) = csv_rows(&out.join("Outlook-Files.csv"));
    let ost = of(&files, "example-2013.ost")[0];
    assert_eq!(ost["User"], "alice");
    assert_eq!(ost["FileType"], "OST");
    assert_eq!(ost["Format"], "Unicode 4K");
    assert_eq!((&*ost["Version"], &*ost["ClientVersion"]), ("36", "12"));
    assert_eq!(ost["Encryption"], "None");
    assert_eq!(
        (&*ost["Folders"], &*ost["Items"], &*ost["HiddenItems"]),
        ("46", "3", "14")
    );
    assert_eq!(ost["RecordKey"], "B18D258814BE384892F7428367355A6E");
    let pst = of(&files, "outlook.pst")[0];
    assert_eq!(pst["User"], "bob");
    assert_eq!(
        (&*pst["FileType"], &*pst["Format"], &*pst["Encryption"]),
        ("PST", "Unicode", "Permute")
    );
    assert_eq!(pst["DisplayName"], "Personal Folders");
    assert_eq!((&*pst["Attachments"], &*pst["Problems"]), ("22", "0"));
    let list = of(&files, "dist-list.pst")[0];
    assert_eq!(
        (
            &*list["Items"],
            &*list["HiddenItems"],
            &*list["EmbeddedItems"]
        ),
        ("4", "3", "2")
    );
    let nested = of(&files, "msgInMsgInMsg.msg")[0];
    assert_eq!(
        (
            &*nested["FileType"],
            &*nested["Format"],
            &*nested["Folders"]
        ),
        ("MSG", "CFB v3", "")
    );
    assert_eq!(
        (&*nested["EmbeddedItems"], &*nested["Attachments"]),
        ("2", "4")
    );
    assert_eq!(of(&files, "mapiToOft.oft")[0]["FileType"], "OFT");

    // --- folders ---
    let (_, folders) = csv_rows(&out.join("Outlook-Folders.csv"));
    // Unicode folder names
    let inbox = folders
        .iter()
        .find(|f| f["FolderPath"] == "/Top of Personal Folders/收件箱")
        .unwrap();
    assert_eq!(inbox["Name"], "收件箱");
    let sent = folders
        .iter()
        .find(|f| f["FolderPath"] == "/Root - Mailbox/IPM_SUBTREE/Sent Items")
        .unwrap();
    assert_eq!(sent["Messages"], "1");
    assert_eq!(sent["ContainerClass"], "IPF.Imap");
    let root = folders
        .iter()
        .find(|f| f["SourceFile"].ends_with("example-2013.ost") && f["FolderPath"] == "/")
        .unwrap();
    assert_eq!(root["ParentID"], "");
    assert_eq!(root["CreationTime"], "2014-06-05T16:18:29.398Z");

    // --- items ---
    let (header, items) = csv_rows(&out.join("Outlook-Items.csv"));
    assert_eq!(
        header,
        [
            "SourceFile",
            "User",
            "FolderPath",
            "ItemID",
            "Hidden",
            "EmbeddedIn",
            "ItemType",
            "MessageClass",
            "SubmitTime",
            "DeliveryTime",
            "CreationTime",
            "LastModifiedTime",
            "Subject",
            "SenderName",
            "SenderAddressType",
            "SenderAddress",
            "SenderSMTP-Address",
            "SentRepresentingName",
            "SentRepresentingAddress",
            "To",
            "Cc",
            "Bcc",
            "InternetMessageID",
            "InReplyTo",
            "ConversationTopic",
            "Importance",
            "Sensitivity",
            "Read",
            "Flags",
            "Size",
            "Recipients",
            "Attachments",
            "Categories",
            "BodyPreview",
            "Problems"
        ]
    );
    let welcome = items
        .iter()
        .find(|i| i["FolderPath"] == "/Top of Personal Folders/Inbox")
        .unwrap();
    assert_eq!(welcome["ItemID"], "2097188");
    assert_eq!(
        welcome["Subject"],
        "Welcome to Microsoft Office Outlook 2003"
    );
    assert_eq!(welcome["SubmitTime"], "2007-04-29T22:27:33.593Z");
    assert_eq!(welcome["SenderAddress"], "olteam@microsoft.com");
    assert_eq!(welcome["To"], "New Outlook User");
    assert_eq!(
        (&*welcome["Read"], &*welcome["Flags"]),
        ("true", "0x00000011")
    );
    assert_eq!(
        (&*welcome["Recipients"], &*welcome["Attachments"]),
        ("1", "11")
    );
    assert!(
        welcome["BodyPreview"].starts_with("Thank you for using Microsoft® Office Outlook® 2003!"),
        "{}",
        welcome["BodyPreview"]
    );
    assert!(welcome["BodyPreview"].chars().count() <= 256);
    let test2 = items.iter().find(|i| i["Subject"] == "Test 2").unwrap();
    assert_eq!(
        test2["FolderPath"],
        "/Root - Mailbox/IPM_SUBTREE/Sent Items"
    );
    assert_eq!(
        test2["InternetMessageID"],
        "<7D1139ED-A46E-49DF-9570-7EC8A06108DD@apogeephysicians.com>"
    );
    assert_eq!(test2["DeliveryTime"], "2014-04-09T16:38:32Z");
    // folder associated information: hidden items
    let hidden: Vec<_> = items.iter().filter(|i| i["Hidden"] == "true").collect();
    assert_eq!(hidden.len(), 17);
    assert!(
        hidden
            .iter()
            .any(|i| i["MessageClass"] == "IPM.Microsoft.WunderBar.Link" && i["Subject"] == "Inbox")
    );
    // messages in a message in a message: their ids from the item down
    let nested = of(&items, "msgInMsgInMsg.msg");
    let ids: Vec<(&str, &str, &str)> = nested
        .iter()
        .map(|i| (&*i["ItemID"], &*i["EmbeddedIn"], &*i["Subject"]))
        .collect();
    assert_eq!(
        ids,
        [
            ("0", "", "I have sub attachments!"),
            ("0/0", "0", "I have attachments!"),
            // Japanese text, decoded
            ("0/0/0", "0/0", "Microsoft Outlook テスト メッセージ")
        ]
    );
    assert!(nested.iter().all(|i| i["FolderPath"].is_empty()));
    // a recurring appointment's exceptions, embedded in it
    let exceptions: Vec<_> = items
        .iter()
        .filter(|i| i["EmbeddedIn"] == "2097348")
        .collect();
    assert_eq!(exceptions.len(), 2);
    assert_eq!(exceptions[0]["ItemID"], "2097348/0");
    assert_eq!(
        exceptions[0]["FolderPath"],
        "/Top of Personal Folders/Calendar"
    );
    assert_eq!(exceptions[0]["BodyPreview"], "This is the appointment at 9");

    // --- recipients ---
    let (_, recipients) = csv_rows(&out.join("Outlook-Recipients.csv"));
    let tim = of(&recipients, "msgClassAppointment.msg")[0];
    assert_eq!(
        (&*tim["ItemID"], &*tim["Index"], &*tim["RecipientType"]),
        ("0", "0", "To")
    );
    assert_eq!(
        (&*tim["AddressType"], &*tim["SMTP-Address"]),
        ("EX", "tallison@mitre.org")
    );
    assert_eq!(of(&recipients, "msgInMsgInMsg.msg")[0]["ItemID"], "0/0/0");

    // --- attachments ---
    let (_, attachments) = csv_rows(&out.join("Outlook-Attachments.csv"));
    let jpeg = attachments
        .iter()
        .find(|a| a["SourceFile"].ends_with("outlook.pst") && a["Index"] == "0")
        .unwrap();
    assert_eq!(jpeg["ItemID"], "2097188");
    assert_eq!(
        (&*jpeg["Method"], &*jpeg["Filename"], &*jpeg["Extension"]),
        ("ByValue", "1.jpg", ".jpg")
    );
    assert_eq!(
        (&*jpeg["MimeType"], &*jpeg["Inline"]),
        ("image/jpeg", "true")
    );
    assert_eq!((&*jpeg["Size"], &*jpeg["DataSize"]), ("2069", "1969"));
    assert_eq!(
        jpeg["SHA256"],
        "0c7da5374fe7e65b84521e111a584c5508eb40e1f8f044e4ff9416740d601311"
    );
    assert_eq!(jpeg["ExtractedFile"], "");
    let embedded = of(&attachments, "msgInMsgInMsg.msg");
    assert_eq!(embedded[0]["Method"], "EmbeddedMessage");
    assert_eq!(embedded[0]["EmbeddedSubject"], "I have attachments!");
    assert_eq!(embedded[0]["DataSize"], "");
    let green = embedded
        .iter()
        .find(|a| a["Filename"] == "green.png")
        .unwrap();
    assert_eq!(green["ItemID"], "0/0");
    assert_eq!(green["LastModifiedTime"], "2020-09-10T10:17:47.638260900Z");

    // --- contacts, calendar items and tasks ---
    let (_, contacts) = csv_rows(&out.join("Outlook-Contacts.csv"));
    // an ANSI contact in code page 932
    let ansi = of(&contacts, "contactAnsi.msg")[0];
    assert_eq!(ansi["Surname"], "イグザンプル");
    assert_eq!(ansi["GivenName"], "コム");
    assert_eq!(ansi["Company"], "My work company");
    assert_eq!(ansi["Email1Address"], "test@example.com");
    assert_eq!(ansi["MobilePhone"], "080-0001-0001");
    let kinds: Vec<&str> = of(&contacts, "dist-list.pst")
        .iter()
        .map(|c| &*c["ItemType"])
        .collect();
    assert_eq!(kinds, ["Contact", "DistributionList"]);
    let one = contacts
        .iter()
        .find(|c| c["DisplayName"] == "contact name 1")
        .unwrap();
    assert_eq!(one["FileUnder"], "1, contact name");
    assert_eq!(one["Email1Address"], "contact1@rjohnson.id.au");
    assert_eq!(one["FolderPath"], "/Top of Personal Folders/Contacts");

    let (_, calendar) = csv_rows(&out.join("Outlook-Calendar.csv"));
    let fox = of(&calendar, "msgClassAppointment.msg")[0];
    assert_eq!(fox["Subject"], "Quick brown fox");
    assert_eq!(
        (
            &*fox["StartTime"],
            &*fox["EndTime"],
            &*fox["DurationMinutes"]
        ),
        ("2017-02-28T18:30:00Z", "2017-02-28T19:00:00Z", "30")
    );
    assert_eq!(
        (
            &*fox["Location"],
            &*fox["BusyStatus"],
            &*fox["ResponseStatus"]
        ),
        ("under lazy dog", "Busy", "None")
    );
    assert_eq!(fox["Organizer"], "Allison, Timothy B.");
    assert!(fox["GlobalObjectID"].starts_with("040000008200E00074C5B7101A82E008"));
    let recurring = of(&calendar, "dist-list.pst")[0];
    assert_eq!(
        (&*recurring["Recurring"], &*recurring["ReminderSet"]),
        ("true", "true")
    );
    assert_eq!(recurring["StartTime"], "2016-08-02T15:00:00Z");

    let (_, tasks) = csv_rows(&out.join("Outlook-Tasks.csv"));
    let task = &tasks[0];
    assert_eq!(task["Subject"], "Must jump over the lazy dog");
    assert_eq!(
        (
            &*task["Status"],
            &*task["PercentComplete"],
            &*task["Complete"]
        ),
        ("InProgress", "0", "false")
    );
    // dates in the user's time zone, without an offset
    assert_eq!(
        (&*task["StartTime"], &*task["DueTime"]),
        ("2017-02-19T00:00:00", "2017-03-10T00:00:00")
    );
    assert_eq!(task["Owner"], "Allison, Timothy B.");
    assert_eq!(task["CreationTime"], "2017-02-28T18:44:08.031Z");
}

#[test]
fn outlook_json_and_jsonl() {
    let evidence = tempfile::tempdir().unwrap();
    let host = collection(evidence.path());
    // JSONL on stdout: the rows only
    let r = run(&[
        "parse-file",
        "outlook",
        "-d",
        host.to_str().unwrap(),
        "-t",
        "jsonl",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    let rows = jsonl(&r.stdout);
    assert_eq!(table_counts(&rows), expected_counts());
    for row in &rows {
        let keys: Vec<&String> = row.as_object().unwrap().keys().collect();
        assert_eq!(keys[0], "SourceFile");
        assert_eq!(keys[1], "Table");
        assert_eq!(keys[2], "User");
    }
    let nested = rows
        .iter()
        .find(|r| r["Table"] == "Items" && r["ItemID"] == "0/0/0")
        .unwrap();
    assert_eq!(nested["EmbeddedIn"], "0/0");
    assert_eq!(nested["Hidden"], false);
    assert_eq!(nested["Recipients"], 1);
    assert_eq!(nested["FolderPath"], Value::Null);
    let task = rows.iter().find(|r| r["Table"] == "Tasks").unwrap();
    assert_eq!(task["PercentComplete"], 0);
    assert_eq!(task["Complete"], false);
    let file = rows
        .iter()
        .find(|r| r["Table"] == "Files" && r["FileType"] == "OST")
        .unwrap();
    assert_eq!(file["Version"], 36);
    assert_eq!(file["Folders"], 46);

    // one JSON file, one file read by -f
    let out = evidence.path().join("out/outlook.json");
    let pst = host.join(PST).join("dist-list.pst");
    let r = run(&[
        "parse-file",
        "outlook",
        "-f",
        pst.to_str().unwrap(),
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
    assert_eq!(
        table_counts(&objects),
        BTreeMap::from([
            ("Attachments".to_string(), 2),
            ("Calendar".to_string(), 1),
            ("Contacts".to_string(), 2),
            ("Files".to_string(), 1),
            ("Folders".to_string(), 24),
            ("Items".to_string(), 9),
        ])
    );
    // pretty-printed
    assert!(text.starts_with("{\n  \"SourceFile\": "), "{}", &text[..40]);
}

#[test]
fn outlook_properties_and_attachment_extraction() {
    let evidence = tempfile::tempdir().unwrap();
    let host = collection(evidence.path());
    let out = evidence.path().join("csv");
    let extracted = evidence.path().join("attachments");
    // run in the evidence directory, as auto-parse does: no absolute path in the rows
    let r = run_in(
        Some(&host),
        &[
            "parse-file",
            "outlook",
            "-d",
            ".",
            "-o",
            out.to_str().unwrap(),
            "--properties",
            "--extract-attachments",
            extracted.to_str().unwrap(),
            "-K",
        ],
    );
    assert_eq!(r.code, 0, "{}", r.all());
    assert!(r.stdout.contains("(9 CSV files)"), "{}", r.stdout);
    assert!(
        r.stdout.contains("WARNING: --extract-attachments"),
        "{}",
        r.stdout
    );
    assert!(
        r.stdout.lines().any(|l| l == "Properties: 3,496"),
        "{}",
        r.stdout
    );
    assert!(
        r.stdout
            .lines()
            .any(|l| l == "Attachment files extracted: 13"),
        "{}",
        r.stdout
    );

    // --- properties ---
    let (header, properties) = csv_rows(&out.join("Outlook-Properties.csv"));
    assert_eq!(
        header,
        [
            "SourceFile",
            "User",
            "ItemID",
            "Scope",
            "Index",
            "PropertyTag",
            "PropertyName",
            "Type",
            "Value"
        ]
    );
    assert_eq!(properties.len(), 3_496);
    assert!(
        properties
            .iter()
            .all(|p| p["SourceFile"].starts_with("./C/") || p["SourceFile"].starts_with(".\\C\\"))
    );
    let scopes: BTreeMap<&str, usize> = properties.iter().fold(BTreeMap::new(), |mut m, p| {
        *m.entry(p["Scope"].as_str()).or_default() += 1;
        m
    });
    assert_eq!(
        scopes.keys().copied().collect::<Vec<_>>(),
        ["Attachment", "Folder", "Item", "Recipient", "Store"]
    );
    // a named property of a contact, by its property set's name
    let email = properties
        .iter()
        .find(|p| {
            p["SourceFile"].ends_with("contactAnsi.msg")
                && p["PropertyName"] == "PSETID_Address:0x8083"
        })
        .unwrap();
    assert_eq!(
        (&*email["Scope"], &*email["Type"], &*email["Value"]),
        ("Item", "PtypString8", "test@example.com")
    );
    // a time, in UTC
    let submitted = properties
        .iter()
        .find(|p| {
            p["SourceFile"].ends_with("outlook.pst")
                && p["ItemID"] == "2097188"
                && p["PropertyTag"] == "0x00390040"
        })
        .unwrap();
    assert_eq!(submitted["Value"], "2007-04-29T22:27:33.593Z");
    // the store's and its folders' properties
    assert!(properties.iter().any(|p| p["ItemID"] == "store"
        && p["PropertyTag"] == "0x0FF90102"
        && p["Value"] == "4E887873E29EC9448EC2346DFA6DF858"));
    assert!(properties.iter().any(|p| p["Scope"] == "Folder"
        && p["ItemID"].starts_with("folder:")
        && p["Value"] == "收件箱"));
    // an attachment's properties with its index; its data is not repeated
    let data = properties
        .iter()
        .find(|p| {
            p["Scope"] == "Attachment"
                && p["ItemID"] == "2097188"
                && p["PropertyTag"] == "0x37010102"
        })
        .unwrap();
    assert_eq!((&*data["Index"], &*data["Value"]), ("0", "(object)"));

    // --- extraction ---
    let (_, attachments) = csv_rows(&out.join("Outlook-Attachments.csv"));
    let mut files: Vec<String> = std::fs::read_dir(&extracted)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    files.sort();
    assert_eq!(files.len(), 13);
    for attachment in &attachments {
        if attachment["SHA256"].is_empty() {
            assert_eq!(attachment["ExtractedFile"], "");
            continue;
        }
        let name = &attachment["ExtractedFile"];
        assert!(name.starts_with(&attachment["SHA256"]), "{attachment:?}");
        let data = std::fs::read(extracted.join(name)).unwrap();
        assert_eq!(data.len().to_string(), attachment["DataSize"]);
    }
    assert!(files.contains(
        &"0c7da5374fe7e65b84521e111a584c5508eb40e1f8f044e4ff9416740d601311.jpg".to_string()
    ));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for name in &files {
            let mode = std::fs::metadata(extracted.join(name))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o111, 0, "{name}");
        }
    }
    // a directory with files in it needs -C, which -o alone does not
    let jsonl_out = evidence.path().join("rows.jsonl");
    let args = [
        "parse-file",
        "outlook",
        "-d",
        host.to_str().unwrap(),
        "-t",
        "jsonl",
        "-o",
        jsonl_out.to_str().unwrap(),
        "--extract-attachments",
        extracted.to_str().unwrap(),
        "-K",
    ];
    let r = run(&args);
    assert_eq!(r.code, 1, "{}", r.all());
    assert!(r.all().contains("already exists"), "{}", r.all());
    assert!(!jsonl_out.exists());
    let r = run(&[&args[..], &["-C"]].concat());
    assert_eq!(r.code, 0, "{}", r.all());
    assert!(
        r.stdout
            .lines()
            .any(|l| l == "Attachment files extracted: 0 (already there: 13)"),
        "{}",
        r.stdout
    );
    // -C with --extract-attachments alone
    let r = run(&[
        "parse-file",
        "outlook",
        "-f",
        host.join(MSG).join("msgInMsgInMsg.msg").to_str().unwrap(),
        "--extract-attachments",
        extracted.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
}

#[test]
fn outlook_reads_files_by_their_contents_and_changes_nothing() {
    let evidence = tempfile::tempdir().unwrap();
    let host = collection(evidence.path());
    // any name: told apart by its signature
    let renamed = unpack(
        "msgreader-hiraoka/contactAnsi.msg",
        &evidence.path().join("other"),
        "evidence-item-17.bin",
    );
    let r = run(&[
        "parse-file",
        "outlook",
        "-f",
        renamed.to_str().unwrap(),
        "-t",
        "jsonl",
        "--code-page",
        "932",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    let rows = jsonl(&r.stdout);
    let contact = rows.iter().find(|r| r["Table"] == "Contacts").unwrap();
    assert_eq!(contact["Surname"], "イグザンプル");
    let renamed = unpack(
        "libyal/outlook.pst.gz",
        &evidence.path().join("other"),
        "export",
    );
    let r = run(&[
        "parse-file",
        "outlook",
        "-f",
        renamed.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    assert!(r.stdout.contains("PST and OST stores: 1"), "{}", r.stdout);

    // nothing is written beside the files, and they are unchanged
    let files: Vec<PathBuf> = [
        host.join(OST).join("example-2013.ost"),
        host.join(PST).join("outlook.pst"),
        host.join(PST).join("dist-list.pst"),
        host.join(MSG).join("msgInMsgInMsg.msg"),
    ]
    .into();
    let before: Vec<Vec<u8>> = files.iter().map(|f| std::fs::read(f).unwrap()).collect();
    let listing = |dir: &Path| -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        names.sort();
        names
    };
    let listed: Vec<Vec<String>> = [OST, PST, MSG]
        .iter()
        .map(|d| listing(&host.join(d)))
        .collect();
    let out = evidence.path().join("csv");
    let r = run(&[
        "parse-file",
        "outlook",
        "-d",
        host.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "--properties",
        "-q",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    let after: Vec<Vec<u8>> = files.iter().map(|f| std::fs::read(f).unwrap()).collect();
    assert!(before == after, "the files are unchanged");
    let relisted: Vec<Vec<String>> = [OST, PST, MSG]
        .iter()
        .map(|d| listing(&host.join(d)))
        .collect();
    assert_eq!(listed, relisted);
}

#[test]
fn outlook_input_and_output_errors() {
    let dir = tempfile::tempdir().unwrap();
    // nothing to find: the message auto-parse takes for a skipped task
    let empty = dir.path().join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    std::fs::write(empty.join("Outlook.nk2"), b"x").unwrap();
    let r = run(&["parse-file", "outlook", "-d", empty.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 1);
    assert!(
        r.all()
            .contains("No Outlook files (*.pst, *.ost, *.msg, *.oft) were found in"),
        "{}",
        r.all()
    );
    let r = run(&[
        "parse-file",
        "outlook",
        "-f",
        dir.path().join("missing.pst").to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.all().contains("The file does not exist"), "{}", r.all());
    // a file that is not an Outlook file: could not be parsed, nothing saved
    let text = dir.path().join("notes.txt");
    std::fs::write(&text, b"not an Outlook file").unwrap();
    let out = dir.path().join("nothing");
    let r = run(&[
        "parse-file",
        "outlook",
        "-f",
        text.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(
        r.all()
            .contains("not a PST, OST, MSG or OFT file (no !BDN or compound file signature)"),
        "{}",
        r.all()
    );
    assert!(r.all().contains("(could not parse: 1)"), "{}", r.all());
    assert!(r.all().contains("Nothing was saved"), "{}", r.all());
    assert!(!out.exists());
    // named as one, but not one, beside one that is: the rest is read
    let mixed = dir.path().join("mixed");
    std::fs::create_dir_all(&mixed).unwrap();
    std::fs::write(mixed.join("broken.pst"), b"!BDN and nothing else").unwrap();
    std::fs::write(mixed.join("text.msg"), b"From: someone\r\n\r\nhello").unwrap();
    unpack("apache-poi/msgClassTask.msg", &mixed, "task.MSG");
    let r = run(&["parse-file", "outlook", "-d", mixed.to_str().unwrap(), "-K"]);
    assert_eq!(r.code, 0, "{}", r.all());
    assert!(
        r.all()
            .contains("Outlook files parsed: 1 (could not parse: 2)"),
        "{}",
        r.all()
    );
    assert!(r.all().contains("Tasks: 1"), "{}", r.all());
    // CSV needs a directory
    let r = run(&[
        "parse-file",
        "outlook",
        "-d",
        mixed.to_str().unwrap(),
        "-t",
        "csv",
        "-K",
    ]);
    assert_eq!(r.code, 1);
    assert!(r.all().contains("CSV output needs -o <DIR>"), "{}", r.all());
    // -C needs something to overwrite
    let r = run(&[
        "parse-file",
        "outlook",
        "-d",
        mixed.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 2, "{}", r.all());
    // an existing output is kept without -C
    let out = dir.path().join("rows.jsonl");
    std::fs::write(&out, b"keep").unwrap();
    let task = mixed.join("task.MSG");
    let args = [
        "parse-file",
        "outlook",
        "-f",
        task.to_str().unwrap(),
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
    assert_eq!(jsonl(&std::fs::read_to_string(&out).unwrap()).len(), 3);
    // the file read is never the output
    let r = run(&[
        "parse-file",
        "outlook",
        "-f",
        task.to_str().unwrap(),
        "-t",
        "jsonl",
        "-o",
        task.to_str().unwrap(),
        "-C",
        "-K",
    ]);
    assert_eq!(r.code, 1, "{}", r.all());
    assert_eq!(
        std::fs::read(&task).unwrap(),
        std::fs::read(fixture("apache-poi/msgClassTask.msg")).unwrap()
    );
    // the extraction directory must be one
    let r = run(&[
        "parse-file",
        "outlook",
        "-f",
        task.to_str().unwrap(),
        "--extract-attachments",
        text.to_str().unwrap(),
        "-K",
    ]);
    assert_eq!(r.code, 1, "{}", r.all());
    assert!(
        r.all().contains("is a file, not a directory"),
        "{}",
        r.all()
    );
}

#[test]
fn auto_parse_puts_outlook_tables_in_communication_with_their_times_in_the_offset() {
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
            .find(|t| t["Artifact"] == "outlook" && t["Purpose"] == "artifact")
            .unwrap();
        assert_eq!(task["Status"], "succeeded", "{task}");
        assert_eq!(task["Category"], "Communication");
        assert_eq!(task["Command"][1], "parse-file");
        assert_eq!(task["Command"][2], "outlook");
        let communication = output.join("HOST1/Communication");
        assert!(communication.join("Outlook-logs.txt").is_file());
        if kind == "csv" {
            assert!(!communication.join("Outlook").exists());
            assert_eq!(task["OutputFiles"].as_array().unwrap().len(), 8, "{task}");
            for (table, count) in expected_counts() {
                let (_, rows) = csv_rows(&communication.join(format!("Outlook-{table}.csv")));
                assert_eq!(rows.len(), count, "{table}");
            }
            let (_, items) = csv_rows(&communication.join("Outlook-Items.csv"));
            // the parser runs in the evidence directory: paths are relative to it
            assert!(
                items
                    .iter()
                    .all(|i| Path::new(&i["SourceFile"]).starts_with(Path::new(".").join("HOST1"))),
                "{}",
                items[0]["SourceFile"]
            );
            let welcome = items
                .iter()
                .find(|i| i["FolderPath"] == "/Top of Personal Folders/Inbox")
                .unwrap();
            assert_eq!(welcome["SubmitTime"], "2007-04-30T07:27:33.593+09:00");
            assert_eq!(welcome["User"], "bob");
            let (_, calendar) = csv_rows(&communication.join("Outlook-Calendar.csv"));
            let fox = calendar
                .iter()
                .find(|c| c["Subject"] == "Quick brown fox")
                .unwrap();
            assert_eq!(fox["StartTime"], "2017-03-01T03:30:00+09:00");
            // a task's dates are the user's wall clock: left as they are
            let (_, tasks) = csv_rows(&communication.join("Outlook-Tasks.csv"));
            assert_eq!(tasks[0]["StartTime"], "2017-02-19T00:00:00");
            assert_eq!(tasks[0]["CreationTime"], "2017-03-01T03:44:08.031+09:00");
        } else {
            let rows =
                jsonl(&std::fs::read_to_string(communication.join("Outlook.jsonl")).unwrap());
            assert_eq!(table_counts(&rows), expected_counts());
            assert!(
                rows.iter()
                    .all(|r| r["Comp"] == "HOST1" && r["Artifact"] == "outlook")
            );
        }
        let timeline = jsonl(&std::fs::read_to_string(output.join("all.jsonl")).unwrap());
        let rows: Vec<&Value> = timeline
            .iter()
            .filter(|r| r["Artifact"] == "outlook")
            .collect();
        assert_eq!(rows.len(), 177);
        let test2 = rows
            .iter()
            .find(|r| r["Table"] == "Items" && r["Subject"] == "Test 2")
            .unwrap();
        assert_eq!(test2["DeliveryTime"], "2014-04-10T01:38:32+09:00");
    }
}

#[test]
fn outlook_precise_properties_and_party_fallbacks_survive_all_exports() {
    let evidence = tempfile::tempdir().unwrap();
    let host = evidence.path().join("HOST1");
    std::fs::create_dir_all(host.join("C/Windows/System32")).unwrap();
    let input = host.join(MSG).join("precise.msg");
    std::fs::create_dir_all(input.parent().unwrap()).unwrap();
    let mut file = cfb::CompoundFile::create(std::io::Cursor::new(Vec::new())).unwrap();
    let mut properties = vec![0; 32];
    for (tag, value) in [
        (0x6000_0007u32, 0.0f64.to_bits()),
        (0x6001_0007, 25_569.0f64.to_bits() + 1),
        (0x0039_0040, 116_444_736_000_000_003),
    ] {
        properties.extend(tag.to_le_bytes());
        properties.extend(6u32.to_le_bytes());
        properties.extend(value.to_le_bytes());
    }
    file.create_stream("/__properties_version1.0")
        .unwrap()
        .write_all(&properties)
        .unwrap();
    for (tag, text) in [
        (0x001Au16, "IPM.Appointment"),
        (0x0C1A, ""),
        (0x5D01, ""),
        (0x0C1F, "sender@example.test"),
        (0x0042, "Organizer"),
        (0x5D02, ""),
        (0x0065, "organizer@example.test"),
    ] {
        file.create_stream(format!("/__substg1.0_{tag:04X}001E"))
            .unwrap()
            .write_all(text.as_bytes())
            .unwrap();
    }
    std::fs::write(&input, file.into_inner().into_inner()).unwrap();
    let input_before = std::fs::read(&input).unwrap();

    let check_json = |rows: &[Value], submitted: &str| {
        let property = |tag: &str| {
            rows.iter()
                .find(|r| r["Table"] == "Properties" && r["PropertyTag"] == tag)
                .unwrap()["Value"]
                .clone()
        };
        assert_eq!(property("0x60000007"), "1899-12-30T00:00:00Z");
        assert_eq!(property("0x60010007"), "1970-01-01T00:00:00.000000300Z");
        let item = rows.iter().find(|r| r["Table"] == "Items").unwrap();
        assert_eq!(item["SentRepresentingAddress"], "organizer@example.test");
        assert_eq!(item["SubmitTime"], submitted);
        assert_eq!(
            rows.iter().find(|r| r["Table"] == "Calendar").unwrap()["Organizer"],
            "Organizer"
        );
    };
    for format in ["csv", "json", "jsonl"] {
        let output = evidence.path().join(format!("direct-{format}"));
        let r = run(&[
            "parse-file",
            "outlook",
            "-f",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-t",
            format,
            "--properties",
            "-K",
        ]);
        assert_eq!(r.code, 0, "{}", r.all());
        assert!(r.stdout.contains("sender@example.test"), "{}", r.stdout);
        if format == "csv" {
            let (_, properties) = csv_rows(&output.join("Outlook-Properties.csv"));
            let property = |tag: &str| {
                properties.iter().find(|r| r["PropertyTag"] == tag).unwrap()["Value"].clone()
            };
            assert_eq!(property("0x60000007"), "1899-12-30T00:00:00Z");
            assert_eq!(property("0x60010007"), "1970-01-01T00:00:00.000000300Z");
            let (_, items) = csv_rows(&output.join("Outlook-Items.csv"));
            assert_eq!(
                items[0]["SentRepresentingAddress"],
                "organizer@example.test"
            );
            let (_, calendar) = csv_rows(&output.join("Outlook-Calendar.csv"));
            assert_eq!(calendar[0]["Organizer"], "Organizer");
        } else {
            let contents = std::fs::read_to_string(output).unwrap();
            let rows = serde_json::Deserializer::from_str(&contents)
                .into_iter::<Value>()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            check_json(&rows, "1970-01-01T00:00:00.000000300Z");
        }
    }
    let auto_output = tempfile::tempdir().unwrap();
    let output = auto_output.path().join("auto");
    let timeline = output.join("all.jsonl");
    let r = run(&[
        "auto-parse",
        "-d",
        evidence.path().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-j",
        timeline.to_str().unwrap(),
        "-Z",
        "+09:00",
        "-K",
    ]);
    assert_eq!(r.code, 0, "{}", r.all());
    let timeline = jsonl(&std::fs::read_to_string(timeline).unwrap());
    let item = timeline
        .iter()
        .find(|r| r["Table"] == "Items" && r["Artifact"] == "outlook")
        .unwrap();
    assert_eq!(item["SubmitTime"], "1970-01-01T09:00:00.000000300+09:00");
    assert_eq!(item["SentRepresentingAddress"], "organizer@example.test");
    assert_eq!(std::fs::read(input).unwrap(), input_before);
}
