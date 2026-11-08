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

//! `parse-file outlook`: Outlook's PST and OST stores and its `.msg` and `.oft` files, read by
//! Yamato Security's ys-outlook-parser without Outlook or MAPI, read-only and leniently: the
//! stores themselves, their folders, every item (messages, contacts, calendar items, tasks and the
//! hidden items of the folders' associated contents tables, with the messages embedded in their
//! attachments), the items' recipients and attachments, and with `--properties` every property
//! as stored. One CSV table per kind of row, or one JSON object per row; each item's rows are
//! written as soon as it has been read, so that a store of many gigabytes is never held in memory.
//! Hayabusa's input options, progress bar and results summary around it.

use crate::browser::{optional, path_user, text, time};
use crate::cli::{OutlookOptions, OutlookOutputType};
use crate::color::JunteiColor;
use crate::progress;
use crate::srum::cell;
use crate::util::{self, check_output, error_msg, fmt_num, p, p_label};
use chrono::{DateTime, Datelike, Utc};
use common::time::Ts;
use indicatif::ProgressBar;
use outlook_parser::item::Props;
use outlook_parser::mapi::{NameId, PropertyName, guid_text, lids, tags, types};
use outlook_parser::msg::{self, FileKind};
use outlook_parser::pst::{self, Encryption, Format, PstStore, StoreKind};
use outlook_parser::{Attachment, Item, MsgItem, Property, Value};
use serde_json::{Map, Value as Json, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

/// Problems printed per file; the rest are only counted.
const MAX_PRINTED_PROBLEMS: usize = 10;

/// Problems an item's or attachment's `Problems` cell lists; the rest are only counted there.
const MAX_CELL_PROBLEMS: usize = 10;

/// Characters of an item's plain-text body in `BodyPreview`.
const PREVIEW_CHARS: usize = 256;

/// Bytes of a binary property value written in hex; the rest are only counted.
const MAX_HEX_BYTES: usize = 4096;

/// Senders listed in the summary.
const MAX_SENDERS: usize = 10;

/// Items read between two updates of the progress bar's message.
const PROGRESS_EVERY: usize = 64;

/// The names in messages that say what is looked for.
const NAMES_SHOWN: &str = "*.pst, *.ost, *.msg, *.oft";

/// What `--extract-attachments` warns before it writes anything.
const EXTRACT_WARNING: &str = "WARNING: --extract-attachments writes the attachments of e-mail \
and other items to disk. They can be malware: never run them, keep them on an isolated analysis \
system, and expect antivirus software to remove them. They are named by their SHA-256 and created \
without execute permission.";

/// The signature PST and OST files start with ([MS-PST] 2.2.2.6 `dwMagic`).
const STORE_SIGNATURE: &[u8; 4] = b"!BDN";

/// The signature compound files (`.msg`, `.oft`) start with.
const CFB_SIGNATURE: &[u8; 8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// The tables written, in the order their CSV files are listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Table {
    Files,
    Folders,
    Items,
    Recipients,
    Attachments,
    Contacts,
    Calendar,
    Tasks,
    Properties,
}

impl Table {
    const ALL: [Table; 9] = [
        Table::Files,
        Table::Folders,
        Table::Items,
        Table::Recipients,
        Table::Attachments,
        Table::Contacts,
        Table::Calendar,
        Table::Tasks,
        Table::Properties,
    ];

    fn name(self) -> &'static str {
        match self {
            Table::Files => "Files",
            Table::Folders => "Folders",
            Table::Items => "Items",
            Table::Recipients => "Recipients",
            Table::Attachments => "Attachments",
            Table::Contacts => "Contacts",
            Table::Calendar => "Calendar",
            Table::Tasks => "Tasks",
            Table::Properties => "Properties",
        }
    }

    fn file_name(self) -> String {
        format!("Outlook-{}.csv", self.name())
    }

    /// Every column of the table, in order; each row has them all.
    fn columns(self) -> &'static [&'static str] {
        match self {
            Table::Files => &[
                "SourceFile",
                "User",
                "FileType",
                "Format",
                "Version",
                "ClientVersion",
                "Encryption",
                "DisplayName",
                "RecordKey",
                "Folders",
                "Items",
                "HiddenItems",
                "EmbeddedItems",
                "Attachments",
                "Problems",
            ],
            Table::Folders => &[
                "SourceFile",
                "User",
                "FolderID",
                "ParentID",
                "FolderPath",
                "Name",
                "ContainerClass",
                "ContentCount",
                "UnreadCount",
                "Messages",
                "HiddenItems",
                "Subfolders",
                "CreationTime",
                "LastModifiedTime",
                "Problem",
            ],
            Table::Items => &[
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
                "Problems",
            ],
            Table::Recipients => &[
                "SourceFile",
                "User",
                "ItemID",
                "Index",
                "RecipientType",
                "Name",
                "AddressType",
                "Address",
                "SMTP-Address",
            ],
            Table::Attachments => &[
                "SourceFile",
                "User",
                "ItemID",
                "Index",
                "Method",
                "Filename",
                "Extension",
                "MimeType",
                "ContentID",
                "Inline",
                "Hidden",
                "Size",
                "DataSize",
                "SHA256",
                "Pathname",
                "CreationTime",
                "LastModifiedTime",
                "EmbeddedSubject",
                "ExtractedFile",
                "Problems",
            ],
            Table::Contacts => &[
                "SourceFile",
                "User",
                "FolderPath",
                "ItemID",
                "ItemType",
                "DisplayName",
                "GivenName",
                "Surname",
                "Nickname",
                "FileUnder",
                "Company",
                "Title",
                "Department",
                "Email1Name",
                "Email1Address",
                "Email2Name",
                "Email2Address",
                "Email3Name",
                "Email3Address",
                "BusinessPhone",
                "HomePhone",
                "MobilePhone",
                "InstantMessaging",
                "CreationTime",
                "LastModifiedTime",
            ],
            Table::Calendar => &[
                "SourceFile",
                "User",
                "FolderPath",
                "ItemID",
                "ItemType",
                "Subject",
                "StartTime",
                "EndTime",
                "DurationMinutes",
                "Location",
                "AllDay",
                "BusyStatus",
                "Recurring",
                "ResponseStatus",
                "Organizer",
                "AllAttendees",
                "ToAttendees",
                "CcAttendees",
                "GlobalObjectID",
                "ReminderSet",
                "ReminderTime",
                "CreationTime",
                "LastModifiedTime",
            ],
            Table::Tasks => &[
                "SourceFile",
                "User",
                "FolderPath",
                "ItemID",
                "ItemType",
                "Subject",
                "Status",
                "PercentComplete",
                "StartTime",
                "DueTime",
                "CompletedTime",
                "Complete",
                "Owner",
                "Assigner",
                "ReminderSet",
                "ReminderTime",
                "CreationTime",
                "LastModifiedTime",
            ],
            Table::Properties => &[
                "SourceFile",
                "User",
                "ItemID",
                "Scope",
                "Index",
                "PropertyTag",
                "PropertyName",
                "Type",
                "Value",
            ],
        }
    }
}

/// One row: its table and a value for each of the table's columns, in order.
struct Row {
    table: Table,
    values: Vec<Json>,
}

impl Row {
    /// A row of `table` from `source`, its other columns empty.
    fn new(table: Table, source: &Source) -> Row {
        let mut values = vec![Json::Null; table.columns().len()];
        values[0] = json!(source.path);
        values[1] = optional(source.user.as_deref());
        Row { table, values }
    }

    fn set(&mut self, column: &str, value: Json) {
        let at = self
            .table
            .columns()
            .iter()
            .position(|&c| c == column)
            .unwrap_or_else(|| panic!("{column} is not a column of {:?}", self.table));
        self.values[at] = value;
    }
}

/// The file rows come from: its path, and the Windows user whose profile its path is in.
struct Source {
    path: String,
    user: Option<String>,
}

/// Where an item is: its folder (none in a `.msg` file), its id, whether it is folder associated
/// information, and the item its message is attached to.
struct Place<'a> {
    folder: Option<&'a str>,
    id: String,
    hidden: bool,
    embedded_in: Option<&'a str>,
}

/// Why a file was not read through: it could not be read (the next file is), or the rows could
/// not be written (nothing more is).
enum Failure {
    Read(String),
    Write(io::Error),
}

impl From<io::Error> for Failure {
    fn from(e: io::Error) -> Self {
        Failure::Write(e)
    }
}

/// What one file holds, for its row of the Files table.
#[derive(Default)]
struct FileCounts {
    items: usize,
    hidden: usize,
    embedded: usize,
    attachments: usize,
    problems: usize,
    printed: usize,
}

/// Run the command; returns the exit status.
pub fn run(opts: &OutlookOptions) -> i32 {
    let no_color = opts.common.no_color;
    let data_on_stdout = opts.data_on_stdout();
    let report_error = |bar: Option<&ProgressBar>, msg: &str| {
        if data_on_stdout {
            eprintln!("{msg}");
        } else {
            progress::suspended(bar, || error_msg(no_color, msg));
        }
    };
    let output_type = opts.output_type();
    if output_type == OutlookOutputType::Csv && opts.output.is_none() && opts.output_type.is_some()
    {
        report_error(
            None,
            "CSV output needs -o <DIR>: files, folders, items, recipients, attachments, contacts, calendar items and tasks are each a CSV file.",
        );
        return 1;
    }

    // --- inputs ---------------------------------------------------------------------------
    let files = match (&opts.filepath, &opts.directory) {
        (Some(f), _) => {
            if !f.is_file() {
                report_error(None, &format!("The file does not exist: {}", f.display()));
                return 1;
            }
            vec![f.clone()]
        }
        (None, Some(d)) => {
            if !d.is_dir() {
                report_error(
                    None,
                    &format!("The directory does not exist: {}", d.display()),
                );
                return 1;
            }
            let found = find_files(d);
            if found.is_empty() {
                report_error(
                    None,
                    &format!(
                        "No Outlook files ({NAMES_SHOWN}) were found in {}",
                        d.display()
                    ),
                );
                return 1;
            }
            found
        }
        (None, None) => unreachable!("clap requires -f or -d"),
    };
    if let Some(out) = &opts.output
        && let Err(msg) = check_output(out, output_type == OutlookOutputType::Csv, opts.clobber)
    {
        report_error(None, &msg);
        return 1;
    }
    let extractor = match &opts.extract_attachments {
        Some(dir) => {
            if let Err(msg) = check_output(dir, true, opts.clobber) {
                report_error(None, &msg);
                return 1;
            }
            if let Err(e) = std::fs::create_dir_all(dir) {
                report_error(None, &format!("Could not create {}: {e}", dir.display()));
                return 1;
            }
            if data_on_stdout {
                eprintln!("{EXTRACT_WARNING}");
            } else {
                p(JunteiColor::Red.rdg(no_color), EXTRACT_WARNING, true);
                p(None, "", true);
            }
            Some(Extractor::new(dir, opts.clobber, files.clone()))
        }
        None => None,
    };

    let total: u64 = files
        .iter()
        .map(|f| f.metadata().map_or(0, |m| m.len()))
        .sum();
    if !data_on_stdout {
        p_label(no_color, "Outlook files: ", &fmt_num(files.len()));
        p_label(no_color, "Total file size: ", &util::size_display(total));
        p(None, "", true);
    }
    let bar = (!data_on_stdout).then(|| {
        p(
            JunteiColor::Orange.rdg(no_color),
            "Parsing in progress. Please wait.",
            true,
        );
        p(None, "", true);
        progress::progress_bar(files.len(), no_color)
    });

    // --- parse and write ---------------------------------------------------------------------
    let mut parser = Parser {
        code_page: u32::from(opts.code_page),
        properties: opts.properties,
        sink: Sink::new(opts, output_type, files.clone()),
        extractor,
        stats: Stats::default(),
        bar,
        data_on_stdout,
        no_color,
    };
    for file in &files {
        if let Some(b) = &parser.bar {
            b.set_message(progress::message(file));
        }
        match parser.file(file) {
            Ok(()) => parser.stats.files += 1,
            Err(Failure::Read(e)) => {
                parser.stats.failed += 1;
                parser.error(&format!("Could not parse {}: {e}", file.display()));
            }
            Err(Failure::Write(e)) => {
                parser.error(&format!("Could not write the rows: {e}"));
                return 1;
            }
        }
        if let Some(b) = &parser.bar {
            b.inc(1);
        }
    }
    let Parser {
        sink,
        stats,
        bar,
        extractor,
        ..
    } = parser;
    if let Some(b) = &bar {
        b.finish_with_message(progress::finished_message(no_color));
    }
    let saved = match sink.finish(stats.files > 0) {
        Ok(saved) => saved,
        Err(e) => {
            report_error(None, &format!("Could not write the rows: {e}"));
            return 1;
        }
    };

    // --- summary ---------------------------------------------------------------------------
    if !data_on_stdout {
        print_summary(&stats, opts.properties, extractor.as_ref(), no_color);
        match (&opts.output, saved) {
            (Some(out), Some(Saved::Tables(files))) => p_label(
                no_color,
                "Results saved: ",
                &format!("{} ({} CSV files)", out.display(), fmt_num(files)),
            ),
            (Some(out), Some(Saved::File)) => p_label(
                no_color,
                "Results saved: ",
                &format!("{} ({})", out.display(), util::file_size_display(out)),
            ),
            (Some(_), None) => p(None, "Nothing was saved: no file could be read.", true),
            (None, _) => p(
                None,
                "To save every row, add -o <DIR> (CSV tables) or -o <FILE> -t json or jsonl.",
                true,
            ),
        }
    }
    i32::from(stats.files == 0)
}

// --- inputs -------------------------------------------------------------------------------------

/// Whether a file name is one `-d` looks for: `*.pst`, `*.ost`, `*.msg` or `*.oft`, case ignored.
pub fn is_outlook_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    [".pst", ".ost", ".msg", ".oft"]
        .iter()
        .any(|extension| name.len() > extension.len() && name.ends_with(extension))
}

/// The files under `dir` named like Outlook files, recursively, in path order.
pub fn find_files(dir: &Path) -> Vec<PathBuf> {
    util::find_files(dir, is_outlook_name)
}

/// What a file is, from its first bytes.
enum Kind {
    Store,
    Msg,
}

fn kind_of(path: &Path) -> Result<Kind, String> {
    let mut head = [0u8; 8];
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut read = 0;
    while read < head.len() {
        match file.read(&mut head[read..]) {
            Ok(0) => break,
            Ok(n) => read += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    if head[..4] == *STORE_SIGNATURE {
        Ok(Kind::Store)
    } else if head == *CFB_SIGNATURE {
        Ok(Kind::Msg)
    } else {
        Err("not a PST, OST, MSG or OFT file (no !BDN or compound file signature)".into())
    }
}

// --- reading ------------------------------------------------------------------------------------

struct Parser {
    code_page: u32,
    properties: bool,
    sink: Sink,
    extractor: Option<Extractor>,
    stats: Stats,
    bar: Option<ProgressBar>,
    data_on_stdout: bool,
    no_color: bool,
}

impl Parser {
    fn error(&self, msg: &str) {
        if self.data_on_stdout {
            eprintln!("{msg}");
        } else {
            progress::suspended(self.bar.as_ref(), || error_msg(self.no_color, msg));
        }
    }

    /// Count a problem of a file, and print it while fewer than [`MAX_PRINTED_PROBLEMS`] of the
    /// file's have been.
    fn problem(&mut self, source: &Source, counts: &mut FileCounts, problem: &str) {
        counts.problems += 1;
        self.stats.problems += 1;
        if counts.printed < MAX_PRINTED_PROBLEMS {
            counts.printed += 1;
            self.error(&format!("{}: {problem}", source.path));
        }
    }

    fn write(&mut self, row: &Row) -> Result<(), Failure> {
        self.sink.write(row).map_err(Failure::Write)
    }

    /// Read one file and write its rows.
    fn file(&mut self, path: &Path) -> Result<(), Failure> {
        let kind = kind_of(path).map_err(Failure::Read)?;
        let source = Source {
            path: path.to_string_lossy().to_string(),
            user: path_user(path),
        };
        match kind {
            Kind::Store => self.store(path, &source)?,
            Kind::Msg => self.msg(path, &source)?,
        }
        if let Some(user) = &source.user {
            self.stats.users.insert(user.clone());
        }
        Ok(())
    }

    fn store(&mut self, path: &Path, source: &Source) -> Result<(), Failure> {
        let options = pst::Options {
            default_code_page: self.code_page,
            ..pst::Options::default()
        };
        let store = PstStore::open(path, &options).map_err(|e| Failure::Read(e.to_string()))?;
        self.stats.stores += 1;
        let mut counts = FileCounts::default();
        if self.properties {
            self.property_rows(source, "store", "Store", None, &store.info().properties)?;
        }
        let folders = store.folders();
        self.stats.folders += folders.len();
        for folder in &folders {
            let mut row = Row::new(Table::Folders, source);
            row.set("FolderID", json!(folder.node));
            row.set("ParentID", json!(folder.parent));
            row.set("FolderPath", text(&folder.path));
            row.set("Name", optional(folder.name.as_deref()));
            row.set(
                "ContainerClass",
                optional(folder.container_class.as_deref()),
            );
            row.set("ContentCount", json!(folder.content_count));
            row.set("UnreadCount", json!(folder.unread_count));
            row.set("Messages", json!(folder.messages.len()));
            row.set("HiddenItems", json!(folder.associated.len()));
            row.set("Subfolders", json!(folder.subfolders.len()));
            row.set("CreationTime", filetime(folder.created));
            row.set("LastModifiedTime", filetime(folder.modified));
            row.set("Problem", optional(folder.problem.as_deref()));
            self.write(&row)?;
            if let Some(problem) = &folder.problem {
                self.problem(
                    source,
                    &mut counts,
                    &format!("folder {} ({}): {problem}", folder.node, folder.path),
                );
            }
            if self.properties {
                let id = format!("folder:{}", folder.node);
                self.property_rows(source, &id, "Folder", None, &folder.properties)?;
            }
        }
        let total: usize = folders
            .iter()
            .map(|f| f.messages.len() + f.associated.len())
            .sum();
        let shown = self.bar.as_ref().map(|_| progress::message(path));
        let mut read = 0usize;
        for folder in &folders {
            for stored in store.items(folder) {
                read += 1;
                if let (Some(bar), Some(shown)) = (&self.bar, &shown)
                    && read.is_multiple_of(PROGRESS_EVERY)
                {
                    bar.set_message(format!(
                        "{shown} {} / {} items",
                        fmt_num(read),
                        fmt_num(total)
                    ));
                }
                if stored.associated {
                    counts.hidden += 1;
                    self.stats.hidden += 1;
                } else {
                    counts.items += 1;
                    self.stats.items += 1;
                }
                let place = Place {
                    folder: Some(&folder.path),
                    id: stored.node.to_string(),
                    hidden: stored.associated,
                    embedded_in: None,
                };
                match &stored.item {
                    Ok(item) => self.item(source, &mut counts, &place, item)?,
                    Err(e) => {
                        self.stats.unreadable += 1;
                        let problem = format!("could not be read: {e}");
                        let mut row = Row::new(Table::Items, source);
                        row.set("FolderPath", text(&folder.path));
                        row.set("ItemID", json!(place.id));
                        row.set("Hidden", json!(place.hidden));
                        row.set("Problems", json!(problem));
                        self.write(&row)?;
                        self.problem(
                            source,
                            &mut counts,
                            &format!("item {}: {problem}", place.id),
                        );
                    }
                }
            }
        }
        // the file's own damage, found while reading it all
        for problem in store.problems() {
            self.problem(source, &mut counts, &problem);
        }
        let info = store.info();
        let mut row = Row::new(Table::Files, source);
        row.set(
            "FileType",
            json!(match &info.kind {
                StoreKind::Pst => "PST",
                StoreKind::Ost => "OST",
                StoreKind::Other(magic) => magic.as_str(),
            }),
        );
        row.set(
            "Format",
            json!(match info.format {
                Format::Ansi => "ANSI",
                Format::Unicode => "Unicode",
                Format::Unicode4k => "Unicode 4K",
            }),
        );
        row.set("Version", json!(info.version));
        row.set("ClientVersion", json!(info.client_version));
        row.set(
            "Encryption",
            json!(match info.encryption {
                Encryption::None => "None",
                Encryption::Permute => "Permute",
                Encryption::Cyclic => "Cyclic",
            }),
        );
        row.set("DisplayName", optional(info.display_name.as_deref()));
        row.set(
            "RecordKey",
            optional(info.record_key.as_deref().map(hex).as_deref()),
        );
        row.set("Folders", json!(folders.len()));
        self.file_row(row, &counts)
    }

    fn msg(&mut self, path: &Path, source: &Source) -> Result<(), Failure> {
        let options = msg::Options {
            default_code_page: self.code_page,
            ..msg::Options::default()
        };
        let reader = BufReader::new(File::open(path).map_err(|e| Failure::Read(e.to_string()))?);
        let msg =
            MsgItem::from_reader(reader, &options).map_err(|e| Failure::Read(e.to_string()))?;
        self.stats.msgs += 1;
        self.stats.items += 1;
        let mut counts = FileCounts {
            items: 1,
            ..FileCounts::default()
        };
        let place = Place {
            folder: None,
            id: "0".into(),
            hidden: false,
            embedded_in: None,
        };
        self.item(source, &mut counts, &place, &msg.item)?;
        for problem in &msg.problems {
            self.problem(source, &mut counts, problem);
        }
        let mut row = Row::new(Table::Files, source);
        row.set(
            "FileType",
            json!(match msg.kind {
                FileKind::Message => "MSG",
                FileKind::Template => "OFT",
                FileKind::Other => "CFB",
            }),
        );
        row.set("Format", json!(format!("CFB v{}", msg.cfb_version)));
        self.file_row(row, &counts)
    }

    /// Write a file's row, with what was counted while reading it.
    fn file_row(&mut self, mut row: Row, counts: &FileCounts) -> Result<(), Failure> {
        row.set("Items", json!(counts.items));
        row.set("HiddenItems", json!(counts.hidden));
        row.set("EmbeddedItems", json!(counts.embedded));
        row.set("Attachments", json!(counts.attachments));
        row.set("Problems", json!(counts.problems));
        self.write(&row)
    }

    /// Write an item's rows, and those of the messages embedded in its attachments after them.
    fn item(
        &mut self,
        source: &Source,
        counts: &mut FileCounts,
        place: &Place<'_>,
        item: &Item,
    ) -> Result<(), Failure> {
        let id = place.id.as_str();
        for problem in &item.problems {
            self.problem(source, counts, &format!("item {id}: {problem}"));
        }
        self.write(&item_row(source, place, item))?;
        if !place.hidden {
            self.stats.add_item(item);
        }
        for recipient in &item.recipients {
            let mut row = Row::new(Table::Recipients, source);
            row.set("ItemID", json!(id));
            row.set("Index", json!(recipient.index));
            row.set("RecipientType", json!(recipient.kind.map(|k| k.name())));
            row.set("Name", optional(recipient.party.name.as_deref()));
            row.set(
                "AddressType",
                optional(recipient.party.address_type.as_deref()),
            );
            row.set("Address", optional(recipient.party.address.as_deref()));
            row.set("SMTP-Address", optional(recipient.party.smtp.as_deref()));
            self.write(&row)?;
            self.stats.recipients += 1;
        }
        for attachment in &item.attachments {
            counts.attachments += 1;
            self.stats.attachments += 1;
            for problem in &attachment.problems {
                self.problem(
                    source,
                    counts,
                    &format!("item {id}, attachment {}: {problem}", attachment.index),
                );
            }
            let row = self.attachment_row(source, counts, id, attachment);
            self.write(&row)?;
        }
        if let Some(row) = contact_row(source, place, item) {
            self.write(&row)?;
            self.stats.contacts += 1;
        }
        if let Some(row) = calendar_row(source, place, item) {
            self.write(&row)?;
            self.stats.calendar += 1;
        }
        if let Some(row) = task_row(source, place, item) {
            self.write(&row)?;
            self.stats.tasks += 1;
        }
        if self.properties {
            self.property_rows(source, id, "Item", None, &item.properties)?;
            for recipient in &item.recipients {
                let index = Some(recipient.index);
                self.property_rows(source, id, "Recipient", index, &recipient.properties)?;
            }
            for attachment in &item.attachments {
                let index = Some(attachment.index);
                self.property_rows(source, id, "Attachment", index, &attachment.properties)?;
            }
        }
        for attachment in &item.attachments {
            if let Some(embedded) = &attachment.embedded {
                counts.embedded += 1;
                self.stats.embedded += 1;
                let place = Place {
                    folder: place.folder,
                    id: format!("{id}/{}", attachment.index),
                    hidden: place.hidden,
                    embedded_in: Some(id),
                };
                self.item(source, counts, &place, embedded)?;
            }
        }
        Ok(())
    }

    fn attachment_row(
        &mut self,
        source: &Source,
        counts: &mut FileCounts,
        id: &str,
        attachment: &Attachment,
    ) -> Row {
        let mut row = Row::new(Table::Attachments, source);
        let props = Props(&attachment.properties);
        let extension = extension(attachment);
        row.set("ItemID", json!(id));
        row.set("Index", json!(attachment.index));
        row.set("Method", json!(attachment.method.map(method)));
        row.set("Filename", optional(attachment.name()));
        row.set("Extension", optional(extension.as_deref()));
        row.set("MimeType", optional(attachment.mime_type.as_deref()));
        row.set("ContentID", optional(attachment.content_id.as_deref()));
        row.set("Inline", json!(attachment.is_inline()));
        row.set("Hidden", json!(attachment.hidden));
        row.set("Size", json!(attachment.size));
        row.set("Pathname", optional(attachment.pathname.as_deref()));
        row.set("CreationTime", filetime(props.time(tags::CREATION_TIME)));
        row.set(
            "LastModifiedTime",
            filetime(props.time(tags::LAST_MODIFICATION_TIME)),
        );
        row.set(
            "EmbeddedSubject",
            optional(
                attachment
                    .embedded
                    .as_ref()
                    .and_then(|e| e.subject.as_deref()),
            ),
        );
        let mut problems = attachment.problems.clone();
        if let Some(data) = &attachment.data {
            let sha256 = sha256(data);
            row.set("DataSize", json!(data.len()));
            let extracted = self
                .extractor
                .as_mut()
                .map(|x| x.extract(data, &sha256, extension.as_deref()));
            match extracted {
                Some(Ok(name)) => row.set("ExtractedFile", json!(name)),
                Some(Err(e)) => {
                    let index = attachment.index;
                    self.problem(
                        source,
                        counts,
                        &format!("item {id}, attachment {index}: {e}"),
                    );
                    problems.push(e);
                }
                None => {}
            }
            row.set("SHA256", json!(sha256));
        }
        row.set("Problems", problems_cell(&problems));
        row
    }

    /// The Properties rows of one object's properties.
    fn property_rows(
        &mut self,
        source: &Source,
        id: &str,
        scope: &str,
        index: Option<usize>,
        properties: &[Property],
    ) -> Result<(), Failure> {
        for property in properties {
            let mut row = Row::new(Table::Properties, source);
            row.set("ItemID", json!(id));
            row.set("Scope", json!(scope));
            row.set("Index", json!(index));
            row.set("PropertyTag", json!(format!("0x{:08X}", property.tag())));
            row.set(
                "PropertyName",
                json!(property.name.as_ref().map(property_name)),
            );
            row.set("Type", json!(type_name(property.kind)));
            row.set("Value", property_value(&property.value));
            self.write(&row)?;
            self.stats.properties += 1;
        }
        Ok(())
    }
}

// --- rows -----------------------------------------------------------------------------------

fn item_row(source: &Source, place: &Place<'_>, item: &Item) -> Row {
    let mut row = Row::new(Table::Items, source);
    row.set("FolderPath", optional(place.folder));
    row.set("ItemID", json!(place.id));
    row.set("Hidden", json!(place.hidden));
    row.set("EmbeddedIn", optional(place.embedded_in));
    row.set("ItemType", json!(item.kind.name()));
    row.set("MessageClass", optional(item.class.as_deref()));
    row.set("SubmitTime", filetime(item.submitted));
    row.set("DeliveryTime", filetime(item.delivered));
    row.set("CreationTime", filetime(item.created));
    row.set("LastModifiedTime", filetime(item.modified));
    row.set("Subject", optional(item.subject.as_deref()));
    row.set("SenderName", optional(item.sender.name.as_deref()));
    row.set(
        "SenderAddressType",
        optional(item.sender.address_type.as_deref()),
    );
    row.set("SenderAddress", optional(item.sender.address.as_deref()));
    row.set("SenderSMTP-Address", optional(item.sender.smtp.as_deref()));
    row.set(
        "SentRepresentingName",
        optional(item.sent_representing.name.as_deref()),
    );
    // the SMTP address where one is stored besides the address of its type
    row.set(
        "SentRepresentingAddress",
        optional(
            item.sent_representing
                .smtp
                .as_deref()
                .or(item.sent_representing.address.as_deref()),
        ),
    );
    row.set("To", optional(item.display_to.as_deref()));
    row.set("Cc", optional(item.display_cc.as_deref()));
    row.set("Bcc", optional(item.display_bcc.as_deref()));
    row.set("InternetMessageID", optional(item.message_id.as_deref()));
    row.set("InReplyTo", optional(item.in_reply_to.as_deref()));
    row.set(
        "ConversationTopic",
        optional(item.conversation_topic.as_deref()),
    );
    row.set("Importance", json!(item.importance.map(importance)));
    row.set("Sensitivity", json!(item.sensitivity.map(sensitivity)));
    // MSGFLAG_READ
    row.set("Read", json!(item.flags.map(|f| f & 0x1 != 0)));
    row.set(
        "Flags",
        json!(item.flags.map(|f| format!("0x{:08X}", f as u32))),
    );
    row.set("Size", json!(item.size));
    row.set("Recipients", json!(item.recipients.len()));
    row.set("Attachments", json!(item.attachments.len()));
    row.set(
        "Categories",
        if item.categories.is_empty() {
            Json::Null
        } else {
            json!(item.categories.join("; "))
        },
    );
    row.set(
        "BodyPreview",
        optional(item.body.text.as_deref().map(preview).as_deref()),
    );
    row.set("Problems", problems_cell(&item.problems));
    row
}

/// The columns contacts, calendar items and tasks start with.
fn kind_row(table: Table, source: &Source, place: &Place<'_>, item: &Item) -> Row {
    let mut row = Row::new(table, source);
    row.set("FolderPath", optional(place.folder));
    row.set("ItemID", json!(place.id));
    row.set("ItemType", json!(item.kind.name()));
    row.set("CreationTime", filetime(item.created));
    row.set("LastModifiedTime", filetime(item.modified));
    row
}

fn contact_row(source: &Source, place: &Place<'_>, item: &Item) -> Option<Row> {
    let contact = item.contact.as_ref()?;
    let mut row = kind_row(Table::Contacts, source, place, item);
    row.set("DisplayName", optional(contact.display_name.as_deref()));
    row.set("GivenName", optional(contact.given_name.as_deref()));
    row.set("Surname", optional(contact.surname.as_deref()));
    row.set("Nickname", optional(contact.nickname.as_deref()));
    row.set("FileUnder", optional(contact.file_under.as_deref()));
    row.set("Company", optional(contact.company.as_deref()));
    row.set("Title", optional(contact.title.as_deref()));
    row.set("Department", optional(contact.department.as_deref()));
    // each e-mail slot as stored (the model lists only those that are set)
    let props = Props(&item.properties);
    let named = |lid| optional(props.named(lid).and_then(Value::as_str));
    row.set("Email1Name", named(lids::EMAIL1_DISPLAY_NAME));
    row.set("Email1Address", named(lids::EMAIL1_EMAIL_ADDRESS));
    row.set("Email2Name", named(lids::EMAIL2_DISPLAY_NAME));
    row.set("Email2Address", named(lids::EMAIL2_EMAIL_ADDRESS));
    row.set("Email3Name", named(lids::EMAIL3_DISPLAY_NAME));
    row.set("Email3Address", named(lids::EMAIL3_EMAIL_ADDRESS));
    row.set("BusinessPhone", optional(contact.business_phone.as_deref()));
    row.set("HomePhone", optional(contact.home_phone.as_deref()));
    row.set("MobilePhone", optional(contact.mobile_phone.as_deref()));
    row.set(
        "InstantMessaging",
        optional(contact.instant_messaging.as_deref()),
    );
    Some(row)
}

fn calendar_row(source: &Source, place: &Place<'_>, item: &Item) -> Option<Row> {
    let appointment = item.appointment.as_ref()?;
    let mut row = kind_row(Table::Calendar, source, place, item);
    row.set("Subject", optional(item.subject.as_deref()));
    row.set("StartTime", filetime(appointment.start));
    row.set("EndTime", filetime(appointment.end));
    row.set("DurationMinutes", json!(appointment.duration_minutes));
    row.set("Location", optional(appointment.location.as_deref()));
    row.set("AllDay", json!(appointment.all_day));
    row.set(
        "BusyStatus",
        json!(appointment.busy_status.map(busy_status)),
    );
    row.set("Recurring", json!(appointment.recurring));
    row.set(
        "ResponseStatus",
        json!(appointment.response_status.map(response_status)),
    );
    row.set(
        "Organizer",
        optional(
            item.sender
                .name
                .as_deref()
                .or(item.sent_representing.name.as_deref()),
        ),
    );
    row.set(
        "AllAttendees",
        optional(appointment.all_attendees.as_deref()),
    );
    row.set("ToAttendees", optional(appointment.to_attendees.as_deref()));
    row.set("CcAttendees", optional(appointment.cc_attendees.as_deref()));
    row.set(
        "GlobalObjectID",
        optional(appointment.global_object_id.as_deref().map(hex).as_deref()),
    );
    row.set("ReminderSet", json!(item.reminder.set));
    row.set("ReminderTime", filetime(item.reminder.time));
    Some(row)
}

fn task_row(source: &Source, place: &Place<'_>, item: &Item) -> Option<Row> {
    let task = item.task.as_ref()?;
    let mut row = kind_row(Table::Tasks, source, place, item);
    row.set("Subject", optional(item.subject.as_deref()));
    row.set("Status", json!(task.status.map(task_status)));
    row.set(
        "PercentComplete",
        task.percent_complete.map_or(Json::Null, percent),
    );
    // dates in the user's time zone, kept as the wall clock they are
    row.set("StartTime", local_filetime(task.start));
    row.set("DueTime", local_filetime(task.due));
    row.set("CompletedTime", filetime(task.completed));
    row.set("Complete", json!(task.complete));
    row.set("Owner", optional(task.owner.as_deref()));
    row.set("Assigner", optional(task.assigner.as_deref()));
    row.set("ReminderSet", json!(item.reminder.set));
    row.set("ReminderTime", filetime(item.reminder.time));
    Some(row)
}

// --- values ---------------------------------------------------------------------------------

/// A FILETIME (UTC) as the other commands write times; nothing for zero or a time outside the
/// years 0000–9999.
fn filetime(value: Option<i64>) -> Json {
    value
        .filter(|&v| v > 0)
        .map_or(Json::Null, |v| time(Some(Ts::from_filetime(v as u64))))
}

/// A FILETIME that holds a wall-clock time of the user's time zone (a task's start and due
/// dates, midnight of the day): written without an offset (`2026-11-08T00:00:00`), which `-Z`
/// leaves as it is.
fn local_filetime(value: Option<i64>) -> Json {
    value
        .filter(|&v| v > 0)
        .and_then(|v| Ts::from_filetime(v as u64).ticks())
        .and_then(|ticks| {
            DateTime::<Utc>::from_timestamp(
                ticks.div_euclid(10_000_000),
                (ticks.rem_euclid(10_000_000) * 100) as u32,
            )
        })
        .filter(|t| (0..=9999).contains(&t.year()))
        .map_or(Json::Null, |t| {
            json!(t.naive_utc().format("%Y-%m-%dT%H:%M:%S%.f").to_string())
        })
}

/// Bytes in hex, as binary values are written.
fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02X}"));
    }
    out
}

/// Bytes in hex, at most [`MAX_HEX_BYTES`] of them, then how many there are.
fn hex_capped(bytes: &[u8]) -> String {
    if bytes.len() <= MAX_HEX_BYTES {
        hex(bytes)
    } else {
        format!("{}…({} bytes)", hex(&bytes[..MAX_HEX_BYTES]), bytes.len())
    }
}

fn sha256(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The first [`PREVIEW_CHARS`] characters of a text, its runs of white space each one space.
fn preview(text: &str) -> String {
    let mut out = String::new();
    let mut chars = 0;
    'words: for word in text.split_whitespace() {
        if chars > 0 {
            // a space with nothing after it is left out
            if chars + 1 >= PREVIEW_CHARS {
                break;
            }
            out.push(' ');
            chars += 1;
        }
        for c in word.chars() {
            if chars == PREVIEW_CHARS {
                break 'words;
            }
            out.push(c);
            chars += 1;
        }
    }
    out
}

/// Problems in one cell, at most [`MAX_CELL_PROBLEMS`] of them.
fn problems_cell(problems: &[String]) -> Json {
    match problems.len() {
        0 => Json::Null,
        n if n <= MAX_CELL_PROBLEMS => json!(problems.join("; ")),
        n => json!(format!(
            "{}; … ({} more)",
            problems[..MAX_CELL_PROBLEMS].join("; "),
            n - MAX_CELL_PROBLEMS
        )),
    }
}

/// An attachment's extension as stored (`.pdf`), else its name's.
fn extension(attachment: &Attachment) -> Option<String> {
    attachment
        .extension
        .clone()
        .filter(|e| !e.trim_matches('.').is_empty())
        .or_else(|| {
            let name = attachment.name()?;
            let name = name.rsplit(['\\', '/']).next()?;
            let at = name.rfind('.')?;
            (at + 1 < name.len()).then(|| name[at..].to_string())
        })
}

/// The extension of an extracted file: an attachment's, without its dot, lowercased, when it is
/// 1 to 10 ASCII letters and digits; none otherwise, so that no name from the file reaches a path.
fn safe_extension(extension: Option<&str>) -> Option<String> {
    let extension = extension?.trim_start_matches('.');
    ((1..=10).contains(&extension.len()) && extension.bytes().all(|b| b.is_ascii_alphanumeric()))
        .then(|| extension.to_ascii_lowercase())
}

/// `PidTagAttachMethod` ([MS-OXCMSG] 2.2.2.9).
fn method(value: i32) -> String {
    match value {
        0 => "None".into(),
        1 => "ByValue".into(),
        2 => "ByReference".into(),
        3 => "ByReferenceResolve".into(),
        4 => "ByReferenceOnly".into(),
        5 => "EmbeddedMessage".into(),
        6 => "OLE".into(),
        7 => "ByWebReference".into(),
        other => other.to_string(),
    }
}

/// `PidTagImportance`.
fn importance(value: i32) -> String {
    match value {
        0 => "Low".into(),
        1 => "Normal".into(),
        2 => "High".into(),
        other => other.to_string(),
    }
}

/// `PidTagSensitivity`.
fn sensitivity(value: i32) -> String {
    match value {
        0 => "Normal".into(),
        1 => "Personal".into(),
        2 => "Private".into(),
        3 => "Confidential".into(),
        other => other.to_string(),
    }
}

/// `PidLidBusyStatus`.
fn busy_status(value: i64) -> String {
    match value {
        0 => "Free".into(),
        1 => "Tentative".into(),
        2 => "Busy".into(),
        3 => "OutOfOffice".into(),
        4 => "WorkingElsewhere".into(),
        other => other.to_string(),
    }
}

/// `PidLidResponseStatus`.
fn response_status(value: i64) -> String {
    match value {
        0 => "None".into(),
        1 => "Organized".into(),
        2 => "Tentative".into(),
        3 => "Accepted".into(),
        4 => "Declined".into(),
        5 => "NotResponded".into(),
        other => other.to_string(),
    }
}

/// `PidLidTaskStatus`.
fn task_status(value: i64) -> String {
    match value {
        0 => "NotStarted".into(),
        1 => "InProgress".into(),
        2 => "Complete".into(),
        3 => "Waiting".into(),
        4 => "Deferred".into(),
        other => other.to_string(),
    }
}

/// `PidLidPercentComplete` (0.0 to 1.0) as a percentage.
fn percent(fraction: f64) -> Json {
    let percent = fraction * 100.0;
    if percent.is_finite() && percent.fract() == 0.0 && percent.abs() < 1e15 {
        json!(percent as i64)
    } else {
        json!(percent)
    }
}

/// A property type's name ([MS-OXCDATA] 2.11.1).
fn type_name(kind: u16) -> String {
    let base = |kind: u16| -> Option<&'static str> {
        Some(match kind {
            0x0000 => "Unspecified",
            types::NULL => "Null",
            types::INTEGER16 => "Integer16",
            types::INTEGER32 => "Integer32",
            types::FLOATING32 => "Floating32",
            types::FLOATING64 => "Floating64",
            types::CURRENCY => "Currency",
            types::FLOATING_TIME => "FloatingTime",
            types::ERROR_CODE => "ErrorCode",
            types::BOOLEAN => "Boolean",
            types::OBJECT => "Object",
            types::INTEGER64 => "Integer64",
            types::STRING8 => "String8",
            types::STRING => "String",
            types::TIME => "Time",
            types::GUID => "Guid",
            0x00FB => "ServerId",
            0x00FD => "Restriction",
            0x00FE => "RuleAction",
            types::BINARY => "Binary",
            _ => return None,
        })
    };
    match (kind & types::MULTIPLE != 0, base(kind & !types::MULTIPLE)) {
        (false, Some(name)) => format!("Ptyp{name}"),
        (true, Some(name)) => format!("PtypMultiple{name}"),
        (_, None) => format!("0x{kind:04X}"),
    }
}

/// A named property's set and name: `PSETID_Address:0x8083`, `PS_PUBLIC_STRINGS:Keywords`, or
/// the set's GUID when it is not one of the well-known sets ([MS-OXPROPS] 1.3.2).
fn property_name(name: &PropertyName) -> String {
    let set = guid_text(&name.set);
    let known = match set.as_str() {
        "{00062008-0000-0000-C000-000000000046}" => Some("PSETID_Common"),
        "{00062004-0000-0000-C000-000000000046}" => Some("PSETID_Address"),
        "{00062002-0000-0000-C000-000000000046}" => Some("PSETID_Appointment"),
        "{6ED8DA90-450B-101B-98DA-00AA003F1305}" => Some("PSETID_Meeting"),
        "{0006200A-0000-0000-C000-000000000046}" => Some("PSETID_Log"),
        "{41F28F13-83F4-4114-A584-EEDB5A6B0BFF}" => Some("PSETID_Messaging"),
        "{0006200E-0000-0000-C000-000000000046}" => Some("PSETID_Note"),
        "{00062041-0000-0000-C000-000000000046}" => Some("PSETID_PostRss"),
        "{00062003-0000-0000-C000-000000000046}" => Some("PSETID_Task"),
        "{4442858E-A9E3-4E80-B900-317A210CC15B}" => Some("PSETID_UnifiedMessaging"),
        "{00020328-0000-0000-C000-000000000046}" => Some("PS_MAPI"),
        "{71035549-0739-4DCB-9163-00F0580DBBDF}" => Some("PSETID_AirSync"),
        "{00062040-0000-0000-C000-000000000046}" => Some("PSETID_Sharing"),
        "{23239608-685D-4732-9C55-4C95CB4E8E33}" => Some("PSETID_XmlExtractedEntities"),
        "{96357F7F-59E1-47D0-99A7-46515C183B54}" => Some("PSETID_Attachment"),
        "{11000E07-B51B-40D6-AF21-CAA85EDAB1D0}" => Some("PSETID_CalendarAssistant"),
        "{00020329-0000-0000-C000-000000000046}" => Some("PS_PUBLIC_STRINGS"),
        "{00020386-0000-0000-C000-000000000046}" => Some("PS_INTERNET_HEADERS"),
        _ => None,
    };
    let set = known.map_or(set.clone(), str::to_string);
    match &name.id {
        NameId::Number(lid) => format!("{set}:0x{lid:04X}"),
        NameId::Text(text) => format!("{set}:{text}"),
    }
}

/// A property's value: strings as text, times in UTC, binary values in hex (capped), the values
/// of a multi-valued property as a list, a damaged value as what could be recovered of it with a
/// note and its bytes, and an undecoded value's bytes with a note.
fn property_value(value: &Value) -> Json {
    match value {
        Value::Null => Json::Null,
        Value::Integer16(v) => json!(v),
        Value::Integer32(v) => json!(v),
        Value::Integer64(v) => json!(v),
        Value::Floating32(v) => json!(f64::from(*v)),
        Value::Floating64(v) => json!(v),
        Value::Currency(v) => {
            let sign = if *v < 0 { "-" } else { "" };
            let v = v.unsigned_abs();
            json!(format!("{sign}{}.{:04}", v / 10_000, v % 10_000))
        }
        Value::FloatingTime(days) => time(Some(Ts::from_ole_date(*days))),
        Value::ErrorCode(code) => json!(format!("0x{code:08X}")),
        Value::Boolean(v) => json!(v),
        Value::String(text) => json!(text),
        Value::String8 { text, .. } => json!(text),
        Value::Time(v) => filetime(Some(*v)),
        Value::Guid(guid) => json!(guid_text(guid)),
        Value::Binary(bytes) => json!(hex_capped(bytes)),
        Value::Object => json!("(object)"),
        Value::Multiple(values) => Json::Array(values.iter().map(property_value).collect()),
        Value::Unsupported(raw) => json!(format!("{} (unsupported type)", hex_capped(raw))),
        Value::Damaged {
            recovered,
            raw,
            reason,
        } => {
            let bytes = hex_capped(raw);
            match recovered.as_deref().map(property_value) {
                None | Some(Json::Null) => json!(format!("{bytes} (damaged: {reason})")),
                Some(value) => json!(format!(
                    "{} (damaged: {reason}; stored as {bytes})",
                    cell(&value)
                )),
            }
        }
    }
}

// --- extraction -----------------------------------------------------------------------------

/// Writes each attachment's data once, named by its SHA-256, without execute permission.
struct Extractor {
    dir: PathBuf,
    clobber: bool,
    /// The input files, never overwritten.
    inputs: Vec<PathBuf>,
    /// The name each SHA-256 was written under in this run.
    names: HashMap<String, String>,
    written: usize,
    already: usize,
}

impl Extractor {
    fn new(dir: &Path, clobber: bool, inputs: Vec<PathBuf>) -> Self {
        Extractor {
            dir: dir.to_path_buf(),
            clobber,
            inputs,
            names: HashMap::new(),
            written: 0,
            already: 0,
        }
    }

    /// Write `data` as `<sha256>` with the attachment's extension when it is safe; the name it
    /// is under in the directory.
    fn extract(
        &mut self,
        data: &[u8],
        sha256: &str,
        extension: Option<&str>,
    ) -> Result<String, String> {
        if let Some(name) = self.names.get(sha256) {
            return Ok(name.clone());
        }
        let name = match safe_extension(extension) {
            Some(extension) => format!("{sha256}.{extension}"),
            None => sha256.to_string(),
        };
        let path = self.dir.join(&name);
        if let Ok(metadata) = std::fs::symlink_metadata(&path) {
            if metadata.is_file() && std::fs::read(&path).is_ok_and(|have| have == data) {
                self.already += 1;
                self.names.insert(sha256.to_string(), name.clone());
                return Ok(name);
            }
            if !self.clobber {
                return Err(format!(
                    "{} holds something else: not overwritten (add -C to overwrite)",
                    path.display()
                ));
            }
            if metadata.is_dir() {
                return Err(format!(
                    "{} is a directory: not overwritten",
                    path.display()
                ));
            }
            crate::registry::check_not_read(&path, || self.inputs.clone())?;
            // a link is replaced, not followed
            std::fs::remove_file(&path)
                .map_err(|e| format!("could not replace {}: {e}", path.display()))?;
        }
        write_private(&path, data)
            .map_err(|e| format!("could not write {}: {e}", path.display()))?;
        self.written += 1;
        self.names.insert(sha256.to_string(), name.clone());
        Ok(name)
    }
}

/// A new file without execute permission.
fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.flush()
}

// --- writers --------------------------------------------------------------------------------

/// What was saved with `-o`.
enum Saved {
    /// The number of CSV files.
    Tables(usize),
    File,
}

/// Where the rows go, opened when the first file has been read: a CSV file per table (each with
/// its header, rows or not; Properties only with `--properties`), one JSON or JSONL file,
/// stdout, or nowhere (the summary only).
struct Sink {
    output: Option<PathBuf>,
    output_type: OutlookOutputType,
    stdout: bool,
    properties: bool,
    protected: Vec<PathBuf>,
    csv: Option<Vec<Option<csv::Writer<BufWriter<File>>>>>,
    json: Option<Box<dyn Write>>,
}

impl Sink {
    fn new(opts: &OutlookOptions, output_type: OutlookOutputType, protected: Vec<PathBuf>) -> Self {
        Sink {
            output: opts.output.clone(),
            output_type,
            stdout: opts.data_on_stdout(),
            properties: opts.properties,
            protected,
            csv: None,
            json: None,
        }
    }

    fn tables(&self) -> impl Iterator<Item = Table> + '_ {
        Table::ALL
            .into_iter()
            .filter(|&t| t != Table::Properties || self.properties)
    }

    fn open(&mut self) -> io::Result<()> {
        if self.csv.is_some() || self.json.is_some() {
            return Ok(());
        }
        match (&self.output, self.output_type) {
            (Some(dir), OutlookOutputType::Csv) => {
                std::fs::create_dir_all(dir)?;
                let mut writers: Vec<Option<csv::Writer<BufWriter<File>>>> =
                    Table::ALL.iter().map(|_| None).collect();
                for table in self.tables().collect::<Vec<_>>() {
                    let path = dir.join(table.file_name());
                    crate::registry::check_not_read(&path, || self.protected.clone())
                        .map_err(io::Error::other)?;
                    let mut writer = csv::Writer::from_writer(BufWriter::new(File::create(&path)?));
                    writer.write_record(crate::output_schema::headers(
                        table.columns().iter().copied(),
                    )?)?;
                    writers[table as usize] = Some(writer);
                }
                self.csv = Some(writers);
            }
            (Some(out), _) => {
                crate::registry::check_not_read(out, || self.protected.clone())
                    .map_err(io::Error::other)?;
                if let Some(parent) = out.parent().filter(|d| !d.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent)?;
                }
                self.json = Some(Box::new(BufWriter::new(File::create(out)?)));
            }
            (None, _) if self.stdout => self.json = Some(Box::new(BufWriter::new(io::stdout()))),
            (None, _) => {}
        }
        Ok(())
    }

    fn write(&mut self, row: &Row) -> io::Result<()> {
        if self.output.is_none() && !self.stdout {
            return Ok(());
        }
        self.open()?;
        if let Some(writers) = &mut self.csv {
            if let Some(writer) = &mut writers[row.table as usize] {
                writer.write_record(row.values.iter().map(cell))?;
            }
        } else if let Some(out) = &mut self.json {
            let mut object = Map::with_capacity(row.values.len() + 1);
            for (i, (name, value)) in row.table.columns().iter().zip(&row.values).enumerate() {
                if i == 1 {
                    object.insert("Table".into(), json!(row.table.name()));
                }
                object.insert((*name).into(), value.clone());
            }
            crate::output_schema::write_json(
                &mut *out,
                &Json::Object(object),
                self.output_type == OutlookOutputType::Json,
            )?;
            out.write_all(b"\n")?;
        }
        Ok(())
    }

    /// Finish the files; what was saved, if anything. `parsed`: whether any file was read. One
    /// without rows still makes its tables' headers; when none could be read, nothing is
    /// written.
    fn finish(mut self, parsed: bool) -> io::Result<Option<Saved>> {
        if self.output.is_some() && parsed {
            self.open()?;
        }
        if let Some(writers) = &mut self.csv {
            let mut files = 0;
            for writer in writers.iter_mut().flatten() {
                writer.flush()?;
                files += 1;
            }
            return Ok(Some(Saved::Tables(files)));
        }
        if let Some(out) = &mut self.json {
            out.flush()?;
            if self.output.is_some() {
                return Ok(Some(Saved::File));
            }
        }
        Ok(None)
    }
}

// --- summary --------------------------------------------------------------------------------

/// Counts for the results summary.
#[derive(Default)]
struct Stats {
    files: usize,
    failed: usize,
    stores: usize,
    msgs: usize,
    users: BTreeSet<String>,
    folders: usize,
    items: usize,
    hidden: usize,
    embedded: usize,
    unreadable: usize,
    recipients: usize,
    attachments: usize,
    contacts: usize,
    calendar: usize,
    tasks: usize,
    properties: usize,
    problems: usize,
    /// The earliest and latest submit or delivery time of the items that aren't hidden.
    first: Option<i64>,
    last: Option<i64>,
    senders: HashMap<String, usize>,
}

impl Stats {
    fn add_item(&mut self, item: &Item) {
        for time in [item.submitted, item.delivered].into_iter().flatten() {
            if filetime(Some(time)).is_null() {
                continue;
            }
            self.first = Some(self.first.map_or(time, |f| f.min(time)));
            self.last = Some(self.last.map_or(time, |l| l.max(time)));
        }
        let sender = item
            .sender
            .smtp
            .as_deref()
            .or(item.sender.address.as_deref())
            .map(str::to_lowercase)
            .or_else(|| item.sender.name.clone())
            .filter(|s| !s.is_empty());
        if let Some(sender) = sender {
            *self.senders.entry(sender).or_default() += 1;
        }
    }
}

fn print_summary(s: &Stats, properties: bool, extractor: Option<&Extractor>, no_color: bool) {
    p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
    let mut parsed = fmt_num(s.files);
    if s.failed > 0 {
        parsed.push_str(&format!(" (could not parse: {})", fmt_num(s.failed)));
    }
    p_label(no_color, "Outlook files parsed: ", &parsed);
    p_label(no_color, "PST and OST stores: ", &fmt_num(s.stores));
    p_label(no_color, "MSG and OFT files: ", &fmt_num(s.msgs));
    p_label(no_color, "Users: ", &fmt_num(s.users.len()));
    for user in &s.users {
        p(None, &format!("  {user}"), true);
    }
    p_label(no_color, "Folders: ", &fmt_num(s.folders));
    p_label(no_color, "Items: ", &fmt_num(s.items));
    p_label(
        no_color,
        "Hidden items (folder associated information): ",
        &fmt_num(s.hidden),
    );
    p_label(no_color, "Embedded items: ", &fmt_num(s.embedded));
    if s.unreadable > 0 {
        p_label(
            no_color,
            "Items that could not be read: ",
            &fmt_num(s.unreadable),
        );
    }
    p_label(no_color, "Recipients: ", &fmt_num(s.recipients));
    p_label(no_color, "Attachments: ", &fmt_num(s.attachments));
    if let Some(x) = extractor {
        let mut text = fmt_num(x.written);
        if x.already > 0 {
            text.push_str(&format!(" (already there: {})", fmt_num(x.already)));
        }
        p_label(no_color, "Attachment files extracted: ", &text);
    }
    p_label(no_color, "Contacts: ", &fmt_num(s.contacts));
    p_label(no_color, "Calendar items: ", &fmt_num(s.calendar));
    p_label(no_color, "Tasks: ", &fmt_num(s.tasks));
    if properties {
        p_label(no_color, "Properties: ", &fmt_num(s.properties));
    }
    let when = |ticks: i64| filetime(Some(ticks)).as_str().map(str::to_string);
    if let (Some(first), Some(last)) = (s.first.and_then(when), s.last.and_then(when)) {
        p_label(
            no_color,
            "Submit and delivery times: ",
            &format!("{first} - {last}"),
        );
    }
    let mut senders: Vec<(&String, &usize)> = s.senders.iter().collect();
    senders.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    if !senders.is_empty() {
        p(
            JunteiColor::Green.rdg(no_color),
            "Senders with the most items:",
            true,
        );
        for (sender, items) in senders.into_iter().take(MAX_SENDERS) {
            p(None, &format!("  {:>8}  {sender}", fmt_num(*items)), true);
        }
    }
    p_label(no_color, "Problems: ", &fmt_num(s.problems));
}

#[cfg(test)]
mod tests {
    use super::*;
    use outlook_parser::item::AttachmentParts;
    use outlook_parser::mapi::sets;

    fn string(id: u16, value: &str) -> Property {
        Property::new(id, types::STRING, Value::String(value.into()))
    }

    fn named(set: [u8; 16], lid: u32, kind: u16, value: Value) -> Property {
        Property {
            id: 0x8000,
            kind,
            value,
            name: Some(PropertyName {
                set,
                id: NameId::Number(lid),
            }),
        }
    }

    #[test]
    fn outlook_names() {
        for name in [
            "a.pst",
            "Outlook.OST",
            "mail.msg",
            "Template.OFT",
            "x.y.pst",
        ] {
            assert!(is_outlook_name(name), "{name}");
        }
        for name in [".pst", "a.pst.bak", "a.nk2", "a.oab", "pst", "a.ps"] {
            assert!(!is_outlook_name(name), "{name}");
        }
    }

    #[test]
    fn extensions_reach_paths_only_when_safe() {
        assert_eq!(safe_extension(Some(".PDF")).as_deref(), Some("pdf"));
        assert_eq!(safe_extension(Some("jpg")).as_deref(), Some("jpg"));
        assert_eq!(safe_extension(Some(".tar.gz")), None);
        assert_eq!(safe_extension(Some("./../x")), None);
        assert_eq!(safe_extension(Some(".abcdefghijk")), None);
        assert_eq!(safe_extension(Some(".")), None);
        assert_eq!(safe_extension(Some(".é")), None);
        assert_eq!(safe_extension(None), None);
    }

    #[test]
    fn previews_collapse_white_space_and_stop() {
        assert_eq!(
            preview("  Hello,\r\n\r\n\tworld \u{3000}!  "),
            "Hello, world !"
        );
        let long = "あ".repeat(300);
        assert_eq!(preview(&long).chars().count(), PREVIEW_CHARS);
        // the words up to the limit, the last one cut
        let words = "ab ".repeat(200);
        let shown = preview(&words);
        assert_eq!(shown.chars().count(), PREVIEW_CHARS);
        assert!(shown.ends_with(" a"), "{shown}");
        // never a space at the end
        let words = format!("{} b", "a".repeat(PREVIEW_CHARS - 1));
        assert_eq!(preview(&words), "a".repeat(PREVIEW_CHARS - 1));
    }

    #[test]
    fn values_as_written() {
        assert_eq!(hex_capped(&[0xAB, 0x01]), "AB01");
        let long = vec![0u8; MAX_HEX_BYTES + 1];
        let shown = hex_capped(&long);
        assert!(shown.ends_with(&format!("…({} bytes)", MAX_HEX_BYTES + 1)));
        assert_eq!(shown.len(), MAX_HEX_BYTES * 2 + "…(4097 bytes)".len());
        assert_eq!(
            property_value(&Value::Time(128_223_592_535_930_000)),
            json!("2007-04-29T22:27:33.593Z")
        );
        assert_eq!(property_value(&Value::Time(0)), Json::Null);
        assert_eq!(property_value(&Value::Currency(-12_345)), json!("-1.2345"));
        assert_eq!(
            property_value(&Value::ErrorCode(0x8004_010F)),
            json!("0x8004010F")
        );
        assert_eq!(
            property_value(&Value::Multiple(vec![
                Value::String("a".into()),
                Value::Integer32(2)
            ])),
            json!(["a", 2])
        );
        assert_eq!(
            property_value(&Value::Damaged {
                recovered: None,
                raw: vec![1, 2],
                reason: "short".into()
            }),
            json!("0102 (damaged: short)")
        );
        assert_eq!(
            property_value(&Value::Damaged {
                recovered: Some(Box::new(Value::String("A\u{FFFD}B".into()))),
                raw: vec![0x41, 0, 0, 0xD8, 0x42, 0],
                reason: "text is not valid UTF-16LE".into()
            }),
            json!("A\u{FFFD}B (damaged: text is not valid UTF-16LE; stored as 410000D84200)")
        );
        assert_eq!(type_name(types::STRING), "PtypString");
        assert_eq!(
            type_name(types::STRING | types::MULTIPLE),
            "PtypMultipleString"
        );
        assert_eq!(type_name(0x0033), "0x0033");
        assert_eq!(
            property_name(&PropertyName {
                set: sets::ADDRESS,
                id: NameId::Number(0x8083)
            }),
            "PSETID_Address:0x8083"
        );
        assert_eq!(
            property_name(&PropertyName {
                set: sets::PUBLIC_STRINGS,
                id: NameId::Text("Keywords".into())
            }),
            "PS_PUBLIC_STRINGS:Keywords"
        );
        assert_eq!(percent(0.5), json!(50));
        assert_eq!(percent(0.255), json!(25.5));
        // a task's date: the wall clock, without an offset
        assert_eq!(
            local_filetime(Some(131_146_236_000_000_000)),
            json!("2016-08-02T15:00:00")
        );
        assert_eq!(filetime(Some(i64::MAX)), Json::Null);
        assert_eq!(filetime(Some(-1)), Json::Null);
    }

    #[test]
    fn every_row_has_its_tables_columns_in_order() {
        let source = Source {
            path: "mail.msg".into(),
            user: Some("alice".into()),
        };
        let properties = vec![
            string(tags::MESSAGE_CLASS, "IPM.Task"),
            string(tags::SUBJECT, "Report"),
            Property::new(
                tags::MESSAGE_FLAGS,
                types::INTEGER32,
                Value::Integer32(0x11),
            ),
            named(sets::TASK, 0x8101, types::INTEGER32, Value::Integer32(2)),
            named(
                sets::TASK,
                0x8102,
                types::FLOATING64,
                Value::Floating64(1.0),
            ),
        ];
        let attachments = vec![AttachmentParts {
            properties: vec![
                Property::new(tags::ATTACH_METHOD, types::INTEGER32, Value::Integer32(1)),
                string(tags::ATTACH_LONG_FILENAME, "../a.TXT"),
            ],
            data: Some(b"x".to_vec()),
            ..AttachmentParts::default()
        }];
        let item = Item::build(properties, Vec::new(), attachments, Vec::new());
        let place = Place {
            folder: None,
            id: "0".into(),
            hidden: false,
            embedded_in: None,
        };
        let row = item_row(&source, &place, &item);
        assert_eq!(row.values.len(), Table::Items.columns().len());
        let value = |row: &Row, name: &str| -> Json {
            let at = row.table.columns().iter().position(|c| *c == name).unwrap();
            row.values[at].clone()
        };
        assert_eq!(value(&row, "SourceFile"), json!("mail.msg"));
        assert_eq!(value(&row, "User"), json!("alice"));
        assert_eq!(value(&row, "ItemType"), json!("Task"));
        assert_eq!(value(&row, "Read"), json!(true));
        assert_eq!(value(&row, "Flags"), json!("0x00000011"));
        assert_eq!(value(&row, "FolderPath"), Json::Null);
        let row = task_row(&source, &place, &item).unwrap();
        assert_eq!(value(&row, "Status"), json!("Complete"));
        assert_eq!(value(&row, "PercentComplete"), json!(100));
        assert!(contact_row(&source, &place, &item).is_none());
        assert!(calendar_row(&source, &place, &item).is_none());
        let attachment = &item.attachments[0];
        assert_eq!(extension(attachment).as_deref(), Some(".TXT"));
        assert_eq!(method(5), "EmbeddedMessage");
        assert_eq!(method(9), "9");
        // the CSV headers are the column names as written
        for table in Table::ALL {
            assert_eq!(
                crate::output_schema::headers(table.columns().iter().copied()).unwrap(),
                table.columns()
            );
        }
    }

    #[test]
    fn extraction_names_files_by_their_hash_and_keeps_others() {
        let dir = tempfile::tempdir().unwrap();
        let mut extractor = Extractor::new(dir.path(), false, Vec::new());
        let data = b"attached";
        let hash = sha256(data);
        let name = extractor.extract(data, &hash, Some(".PDF")).unwrap();
        assert_eq!(name, format!("{hash}.pdf"));
        assert_eq!(std::fs::read(dir.path().join(&name)).unwrap(), data);
        // once per content, whatever the next attachment's extension
        assert_eq!(extractor.extract(data, &hash, Some(".exe")).unwrap(), name);
        assert_eq!(extractor.written, 1);
        // an unsafe extension is left out
        let other = b"other";
        let other_hash = sha256(other);
        assert_eq!(
            extractor.extract(other, &other_hash, Some(".a/b")).unwrap(),
            other_hash
        );
        // a file of that name holding something else is kept without -C
        let third = b"third";
        let third_hash = sha256(third);
        std::fs::write(dir.path().join(&third_hash), b"not it").unwrap();
        assert!(extractor.extract(third, &third_hash, None).is_err());
        assert_eq!(
            std::fs::read(dir.path().join(&third_hash)).unwrap(),
            b"not it"
        );
        let mut clobbering = Extractor::new(dir.path(), true, Vec::new());
        clobbering.extract(third, &third_hash, None).unwrap();
        assert_eq!(std::fs::read(dir.path().join(&third_hash)).unwrap(), third);
        // the same content already there is not written again
        let mut again = Extractor::new(dir.path(), false, Vec::new());
        again.extract(data, &hash, Some("pdf")).unwrap();
        assert_eq!((again.written, again.already), (0, 1));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(dir.path().join(&name))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o111, 0, "no execute permission");
        }
    }
}
