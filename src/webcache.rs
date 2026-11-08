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

//! `parse-ese webcache`: Internet Explorer and legacy Edge's `WebCacheV01.dat`, the ESE database
//! of the Windows Internet API (WinINet) in `AppData\Local\Microsoft\Windows\WebCache`, read whole
//! by Yamato Security's ys-browser-parser. Its history containers' entries (the pages visited, and
//! the local files File Explorer opened), the downloads of its `iedownload` containers, the cookies
//! Windows 10 and later keep in it, the entries of every other container (cached files, cookie
//! files, web storage, compatibility lists, …) and the containers themselves. One CSV table per
//! kind of record, or one JSON object per row. Hayabusa's input options, progress bar and results
//! summary around it.

use crate::browser::{host, optional, path_user, text, time};
use crate::cli::{WebCacheOptions, WebCacheOutputType};
use crate::color::JunteiColor;
use crate::progress;
use crate::srum::{Ordered, cell};
use crate::util::{self, check_output, error_msg, fmt_num, p, p_label};
use browser::{
    WebCache, WebCacheContainer, WebCacheCookie, WebCacheDownload, WebCacheEntry, WebCacheVisit,
};
use chrono::{DateTime, Utc};
use common::time::{Precision, Semantic, Ts};
use indicatif::ProgressBar;
use serde_json::{Value as Json, json};
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

/// Problems printed per database; the rest are only counted.
const MAX_PRINTED_PROBLEMS: usize = 10;

/// Hosts listed in the summary.
const MAX_HOSTS: usize = 10;

/// The names in messages that say what is looked for.
const NAMES_SHOWN: &str = "WebCacheV01.dat";

/// How the ESE reader's problem for a database not shut down cleanly begins.
const DIRTY: &str = "dirty shutdown";

/// The tables written, in the order their CSV files are listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Table {
    History,
    Downloads,
    Cookies,
    Entries,
    Containers,
}

impl Table {
    const ALL: [Table; 5] = [
        Table::History,
        Table::Downloads,
        Table::Cookies,
        Table::Entries,
        Table::Containers,
    ];

    fn name(self) -> &'static str {
        match self {
            Table::History => "History",
            Table::Downloads => "Downloads",
            Table::Cookies => "Cookies",
            Table::Entries => "Entries",
            Table::Containers => "Containers",
        }
    }

    fn file_name(self) -> String {
        format!("WebCache-{}.csv", self.name())
    }

    /// Every column of the table, in order; each row has them all.
    fn columns(self) -> &'static [&'static str] {
        match self {
            Table::History => &[
                "SourceFile",
                "User",
                "Package",
                "Container",
                "AccessedTime",
                "ModifiedTime",
                "CreationTime",
                "ExpiryTime",
                "SyncTime",
                "Account",
                "URL",
                "Title",
                "AccessCount",
                "SyncCount",
                "EntryType",
                "Flags",
                "EntryID",
                "ContainerID",
            ],
            Table::Downloads => &[
                "SourceFile",
                "User",
                "Package",
                "StartTime",
                "URL",
                "TargetPath",
                "CurrentPath",
                "TotalBytes",
                "Referrer",
                "MimeType",
                "Publisher",
                "AccessedTime",
                "DownloadID",
                "ContainerID",
            ],
            Table::Cookies => &[
                "SourceFile",
                "User",
                "Package",
                "LastModified",
                "Expires",
                "Host",
                "Name",
                "Value",
                "Path",
                "Flags",
                "Directory",
                "CookieTable",
                "EntryID",
            ],
            Table::Entries => &[
                "SourceFile",
                "User",
                "Package",
                "Container",
                "AccessedTime",
                "ModifiedTime",
                "CreationTime",
                "ExpiryTime",
                "SyncTime",
                "URL",
                "Account",
                "Filename",
                "FilePath",
                "FileSize",
                "AccessCount",
                "SyncCount",
                "EntryType",
                "Flags",
                "RedirectURL",
                "ResponseHeaders",
                "RequestHeaders",
                "EntryID",
                "ContainerID",
            ],
            Table::Containers => &[
                "SourceFile",
                "User",
                "Package",
                "ContainerID",
                "Name",
                "Directory",
                "SecureDirectories",
                "PartitionID",
                "LastAccessTime",
                "LastScavengeTime",
            ],
        }
    }
}

/// One row: its table and its columns, in the table's order.
struct Row {
    table: Table,
    columns: Vec<(String, Json)>,
}

impl Row {
    fn push(&mut self, name: &str, value: Json) {
        self.columns.push((name.to_string(), value));
    }
}

/// Where a database is: the file, and the Windows user whose profile its path is in.
struct Context {
    source: String,
    user: Option<String>,
}

impl Context {
    /// The columns every row starts with; `directory` is the directory of the row's container
    /// (or of its cookie table), which names the app package whose data it is.
    fn start(&self, table: Table, directory: Option<&str>) -> Row {
        let mut columns = Vec::with_capacity(table.columns().len());
        columns.push(("SourceFile".to_string(), json!(self.source)));
        columns.push(("User".to_string(), optional(self.user.as_deref())));
        columns.push(("Package".to_string(), optional(package(directory))));
        Row { table, columns }
    }
}

/// Run the command; returns the exit status.
pub fn run(opts: &WebCacheOptions) -> i32 {
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
    if output_type == WebCacheOutputType::Csv && opts.output.is_none() && opts.output_type.is_some()
    {
        report_error(
            None,
            "CSV output needs -o <DIR>: history, downloads, cookies, entries and containers are each a CSV file.",
        );
        return 1;
    }

    // --- inputs ---------------------------------------------------------------------------
    let databases = match (&opts.filepath, &opts.directory) {
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
            let found = find_databases(d);
            if found.is_empty() {
                report_error(
                    None,
                    &format!(
                        "No WebCache databases ({NAMES_SHOWN}) were found in {}",
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
        && let Err(msg) = check_output(out, output_type == WebCacheOutputType::Csv, opts.clobber)
    {
        report_error(None, &msg);
        return 1;
    }

    let total: u64 = databases
        .iter()
        .map(|f| f.metadata().map_or(0, |m| m.len()))
        .sum();
    if !data_on_stdout {
        p_label(
            no_color,
            "WebCache database files: ",
            &fmt_num(databases.len()),
        );
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
        progress::progress_bar(databases.len(), no_color)
    });

    // --- parse and write ---------------------------------------------------------------------
    let mut sink = Sink::new(opts, output_type, databases.clone());
    let mut stats = Stats::default();
    for database in &databases {
        if let Some(b) = &bar {
            b.set_message(progress::message(database));
        }
        let source = database.to_string_lossy().to_string();
        match parse(database, &mut stats) {
            Ok(parsed) => {
                stats.problems += parsed.problems.len();
                for problem in parsed.problems.iter().take(MAX_PRINTED_PROBLEMS) {
                    report_error(bar.as_ref(), &format!("{source}: {problem}"));
                }
                if let Err(e) = parsed.rows.iter().try_for_each(|row| sink.write(row)) {
                    report_error(bar.as_ref(), &format!("Could not write the rows: {e}"));
                    return 1;
                }
            }
            Err(e) => {
                stats.failed += 1;
                report_error(bar.as_ref(), &format!("Could not parse {source}: {e}"));
            }
        }
        if let Some(b) = &bar {
            b.inc(1);
        }
    }
    if let Some(b) = &bar {
        b.finish_with_message(progress::finished_message(no_color));
    }
    let saved = match sink.finish(stats.databases > 0) {
        Ok(saved) => saved,
        Err(e) => {
            report_error(None, &format!("Could not write the rows: {e}"));
            return 1;
        }
    };

    // --- summary ---------------------------------------------------------------------------
    if !data_on_stdout {
        print_summary(&stats, no_color);
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
            (Some(_), None) => p(None, "Nothing was saved: no database could be read.", true),
            (None, _) => p(
                None,
                "To save every row, add -o <DIR> (CSV tables) or -o <FILE> -t json or jsonl.",
                true,
            ),
        }
    }
    i32::from(stats.databases == 0)
}

// --- inputs -------------------------------------------------------------------------------------

/// Whether a file name is one `-d` looks for: `WebCacheV01.dat` (`WebCacheV24.dat` in some
/// versions of Internet Explorer 10), case ignored. Not its transaction logs (`V01.log`) or
/// checkpoint file (`V01.chk`).
pub fn is_database_name(name: &str) -> bool {
    name.to_ascii_lowercase()
        .strip_prefix("webcachev")
        .and_then(|rest| rest.strip_suffix(".dat"))
        .is_some_and(|version| !version.is_empty() && version.bytes().all(|b| b.is_ascii_digit()))
}

/// The files under `dir` named like WebCache databases, recursively, in path order.
pub fn find_databases(dir: &Path) -> Vec<PathBuf> {
    util::find_files(dir, is_database_name)
}

/// The rows of one database and the damage met.
struct Parsed {
    rows: Vec<Row>,
    problems: Vec<String>,
}

/// Read one database.
fn parse(database: &Path, stats: &mut Stats) -> Result<Parsed, String> {
    let data = std::fs::read(database).map_err(|e| e.to_string())?;
    let cache = browser::read_webcache(&data).map_err(|e| e.to_string())?;
    let context = Context {
        source: database.to_string_lossy().to_string(),
        user: path_user(database),
    };
    stats.add(&context, &cache);
    Ok(Parsed {
        rows: rows(&context, &cache),
        problems: cache.problems,
    })
}

/// Every row of a database, table by table.
fn rows(context: &Context, cache: &WebCache) -> Vec<Row> {
    let containers: HashMap<i64, &WebCacheContainer> =
        cache.containers.iter().map(|c| (c.id, c)).collect();
    let directory = |id: Option<i64>| {
        id.and_then(|id| containers.get(&id))
            .and_then(|c| c.directory.as_deref())
    };
    let mut rows = Vec::new();
    for visit in &cache.visits {
        let name = containers.get(&visit.container).map(|c| c.name.as_str());
        rows.push(history_row(
            context,
            visit,
            name,
            directory(Some(visit.container)),
        ));
    }
    for download in &cache.downloads {
        rows.push(download_row(
            context,
            download,
            directory(Some(download.container)),
        ));
    }
    for cookie in &cache.cookies {
        rows.push(cookie_row(context, cookie));
    }
    for entry in &cache.entries {
        rows.push(entry_row(context, entry, directory(Some(entry.container))));
    }
    for container in &cache.containers {
        rows.push(container_row(context, container));
    }
    rows
}

/// The app package whose data a directory holds: the folder after `Packages` (legacy Edge's
/// `…\Packages\microsoft.microsoftedge_8wekyb3d8bbwe\AC\…`, Internet Explorer's app container
/// `windows_ie_ac_001`, another app's); none for the user's own folders.
fn package(directory: Option<&str>) -> Option<&str> {
    let mut parts = directory?.split(['\\', '/']).filter(|p| !p.is_empty());
    parts.find(|p| p.eq_ignore_ascii_case("Packages"))?;
    parts.next()
}

// --- rows -----------------------------------------------------------------------------------

fn history_row(
    context: &Context,
    entry: &WebCacheVisit,
    name: Option<&str>,
    directory: Option<&str>,
) -> Row {
    let visit = &entry.visit;
    let mut row = context.start(Table::History, directory);
    row.push("Container", optional(name));
    row.push("AccessedTime", time(visit.time));
    // local time in a day's or a week's container
    row.push("ModifiedTime", any_time(entry.modified));
    row.push("CreationTime", time(entry.created));
    row.push("ExpiryTime", time(entry.expires));
    row.push("SyncTime", time(entry.synced));
    row.push("Account", optional(visit.user.as_deref()));
    row.push("URL", text(&visit.url));
    row.push("Title", text(&visit.title));
    row.push("AccessCount", json!(visit.visit_count));
    row.push("SyncCount", json!(entry.sync_count));
    row.push("EntryType", json!(entry.entry_type));
    row.push("Flags", json!(entry.flags));
    row.push("EntryID", json!(visit.id));
    row.push("ContainerID", json!(entry.container));
    row
}

/// A time as [`time`] writes it, or, for the wall clock of an unknown zone, the same without an
/// offset (`2014-05-12T09:30:41.686`), which `-Z` leaves as it is.
fn any_time(t: Option<Ts>) -> Json {
    match t {
        Some(t) if t.semantic() == Semantic::LocalUnknownZone => t
            .ticks()
            .and_then(|ticks| {
                DateTime::<Utc>::from_timestamp(
                    ticks.div_euclid(10_000_000),
                    (ticks.rem_euclid(10_000_000) * 100) as u32,
                )
            })
            .map_or(Json::Null, |t| {
                json!(t.naive_utc().format("%Y-%m-%dT%H:%M:%S%.f").to_string())
            }),
        _ => time(t),
    }
}

fn download_row(context: &Context, entry: &WebCacheDownload, directory: Option<&str>) -> Row {
    let download = &entry.download;
    let mut row = context.start(Table::Downloads, directory);
    row.push("StartTime", time(download.start));
    row.push("URL", text(&download.url));
    row.push("TargetPath", text(&download.target_path));
    row.push("CurrentPath", optional(download.current_path.as_deref()));
    row.push("TotalBytes", json!(download.total_bytes));
    row.push("Referrer", optional(download.referrer.as_deref()));
    row.push("MimeType", optional(download.mime_type.as_deref()));
    row.push("Publisher", optional(entry.publisher.as_deref()));
    row.push("AccessedTime", time(entry.accessed));
    row.push("DownloadID", json!(download.id));
    row.push("ContainerID", json!(entry.container));
    row
}

fn cookie_row(context: &Context, cookie: &WebCacheCookie) -> Row {
    let mut row = context.start(Table::Cookies, cookie.directory.as_deref());
    row.push("LastModified", time(cookie.modified));
    row.push("Expires", time(cookie.expires));
    // A cookie's name and value can be empty, as set: kept as stored, as its host and path.
    row.push("Host", json!(cookie.host));
    row.push("Name", json!(cookie.name));
    row.push("Value", json!(cookie.value));
    row.push("Path", json!(cookie.path));
    row.push("Flags", json!(cookie.flags));
    row.push("Directory", optional(cookie.directory.as_deref()));
    row.push("CookieTable", text(&cookie.table));
    row.push("EntryID", json!(cookie.id));
    row
}

fn entry_row(context: &Context, entry: &WebCacheEntry, directory: Option<&str>) -> Row {
    let mut row = context.start(Table::Entries, directory);
    row.push("Container", text(&entry.container_name));
    row.push("AccessedTime", time(entry.accessed));
    row.push("ModifiedTime", time(entry.modified));
    row.push("CreationTime", time(entry.created));
    row.push("ExpiryTime", time(entry.expires));
    row.push("SyncTime", time(entry.synced));
    row.push("URL", text(&entry.url));
    row.push("Account", optional(entry.user.as_deref()));
    row.push("Filename", optional(entry.file_name.as_deref()));
    row.push("FilePath", optional(entry.file_path.as_deref()));
    row.push("FileSize", json!(entry.file_size));
    row.push("AccessCount", json!(entry.access_count));
    row.push("SyncCount", json!(entry.sync_count));
    row.push("EntryType", json!(entry.entry_type));
    row.push("Flags", json!(entry.flags));
    row.push("RedirectURL", optional(entry.redirect_url.as_deref()));
    row.push(
        "ResponseHeaders",
        optional(entry.response_headers.as_deref()),
    );
    row.push("RequestHeaders", optional(entry.request_headers.as_deref()));
    row.push("EntryID", json!(entry.id));
    row.push("ContainerID", json!(entry.container));
    row
}

fn container_row(context: &Context, container: &WebCacheContainer) -> Row {
    let mut row = context.start(Table::Containers, container.directory.as_deref());
    row.push("ContainerID", json!(container.id));
    row.push("Name", text(&container.name));
    row.push("Directory", optional(container.directory.as_deref()));
    row.push(
        "SecureDirectories",
        if container.secure_directories.is_empty() {
            Json::Null
        } else {
            json!(container.secure_directories)
        },
    );
    row.push("PartitionID", optional(container.partition.as_deref()));
    row.push("LastAccessTime", time(container.last_access));
    row.push("LastScavengeTime", time(container.last_scavenge));
    row
}

// --- writers --------------------------------------------------------------------------------

/// What was saved with `-o`.
enum Saved {
    /// The number of CSV files.
    Tables(usize),
    File,
}

/// Where the rows go, opened when the first database has been read: five CSV files (each with
/// its header, rows or not), one JSON or JSONL file, stdout, or nowhere (the summary only).
struct Sink {
    output: Option<PathBuf>,
    output_type: WebCacheOutputType,
    stdout: bool,
    protected: Vec<PathBuf>,
    csv: Option<Vec<csv::Writer<BufWriter<File>>>>,
    json: Option<Box<dyn Write>>,
}

impl Sink {
    fn new(
        opts: &WebCacheOptions,
        output_type: WebCacheOutputType,
        protected: Vec<PathBuf>,
    ) -> Self {
        Sink {
            output: opts.output.clone(),
            output_type,
            stdout: opts.data_on_stdout(),
            protected,
            csv: None,
            json: None,
        }
    }

    fn open(&mut self) -> io::Result<()> {
        if self.csv.is_some() || self.json.is_some() {
            return Ok(());
        }
        match (&self.output, self.output_type) {
            (Some(dir), WebCacheOutputType::Csv) => {
                std::fs::create_dir_all(dir)?;
                let mut writers = Vec::new();
                for table in Table::ALL {
                    let path = dir.join(table.file_name());
                    crate::registry::check_not_read(&path, || self.protected.clone())
                        .map_err(io::Error::other)?;
                    let mut writer = csv::Writer::from_writer(BufWriter::new(File::create(&path)?));
                    writer.write_record(crate::output_schema::headers(
                        table.columns().iter().copied(),
                    )?)?;
                    writers.push(writer);
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
        self.open()?;
        if let Some(writers) = &mut self.csv {
            let writer = &mut writers[Table::ALL.iter().position(|&t| t == row.table).unwrap()];
            writer.write_record(row.columns.iter().map(|(_, value)| cell(value)))?;
        } else if let Some(out) = &mut self.json {
            let mut columns = row.columns.clone();
            columns.insert(1, ("Table".to_string(), json!(row.table.name())));
            crate::output_schema::write_json(
                &mut *out,
                &Ordered(&columns),
                self.output_type == WebCacheOutputType::Json,
            )?;
            out.write_all(b"\n")?;
        }
        Ok(())
    }

    /// Finish the files; what was saved, if anything. `parsed`: whether any database was read.
    /// One without rows still makes its tables' headers; when none could be read, nothing is
    /// written.
    fn finish(mut self, parsed: bool) -> io::Result<Option<Saved>> {
        if self.output.is_some() && parsed {
            self.open()?;
        }
        if let Some(writers) = &mut self.csv {
            for writer in writers.iter_mut() {
                writer.flush()?;
            }
            return Ok(Some(Saved::Tables(writers.len())));
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
    databases: usize,
    failed: usize,
    dirty: usize,
    problems: usize,
    users: BTreeSet<String>,
    history: usize,
    downloads: usize,
    cookies: usize,
    entries: usize,
    containers: usize,
    /// The earliest and latest `AccessedTime` of the history entries, in ticks.
    first: Option<i64>,
    last: Option<i64>,
    hosts: HashMap<String, usize>,
}

impl Stats {
    fn add(&mut self, context: &Context, cache: &WebCache) {
        self.databases += 1;
        self.dirty += usize::from(cache.problems.iter().any(|p| p.starts_with(DIRTY)));
        if let Some(user) = &context.user {
            self.users.insert(user.clone());
        }
        self.history += cache.visits.len();
        self.downloads += cache.downloads.len();
        self.cookies += cache.cookies.len();
        self.entries += cache.entries.len();
        self.containers += cache.containers.len();
        for visit in cache.visits.iter().map(|v| &v.visit) {
            if let Some(ticks) = visit.time.and_then(|t| t.ticks()) {
                self.first = Some(self.first.map_or(ticks, |f| f.min(ticks)));
                self.last = Some(self.last.map_or(ticks, |l| l.max(ticks)));
            }
            if let Some(host) = host(&visit.url) {
                *self.hosts.entry(host.to_ascii_lowercase()).or_default() += 1;
            }
        }
    }
}

fn print_summary(s: &Stats, no_color: bool) {
    p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
    let mut parsed = fmt_num(s.databases);
    if s.failed > 0 {
        parsed.push_str(&format!(" (could not parse: {})", fmt_num(s.failed)));
    }
    p_label(no_color, "WebCache databases parsed: ", &parsed);
    if s.dirty > 0 {
        p_label(
            no_color,
            "Left dirty (changes still only in the transaction logs are missing): ",
            &fmt_num(s.dirty),
        );
    }
    p_label(no_color, "Users: ", &fmt_num(s.users.len()));
    for user in &s.users {
        p(None, &format!("  {user}"), true);
    }
    p_label(no_color, "History entries: ", &fmt_num(s.history));
    p_label(no_color, "Downloads: ", &fmt_num(s.downloads));
    p_label(no_color, "Cookies: ", &fmt_num(s.cookies));
    p_label(no_color, "Other entries: ", &fmt_num(s.entries));
    p_label(no_color, "Containers: ", &fmt_num(s.containers));
    let when = |ticks: i64| {
        time(Some(Ts::from_ticks(ticks, Precision::Tick)))
            .as_str()
            .map(str::to_string)
    };
    if let (Some(first), Some(last)) = (s.first.and_then(when), s.last.and_then(when)) {
        p_label(
            no_color,
            "History access times: ",
            &format!("{first} - {last}"),
        );
    }
    let mut hosts: Vec<(&String, &usize)> = s.hosts.iter().collect();
    hosts.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    if !hosts.is_empty() {
        p(
            JunteiColor::Green.rdg(no_color),
            "Hosts with the most history entries:",
            true,
        );
        for (host, entries) in hosts.into_iter().take(MAX_HOSTS) {
            p(None, &format!("  {:>8}  {host}", fmt_num(*entries)), true);
        }
    }
    p_label(no_color, "Problems: ", &fmt_num(s.problems));
}

#[cfg(test)]
mod tests {
    use super::*;
    use browser::{Download, Transition, Visit};

    #[test]
    fn database_names() {
        for name in ["WebCacheV01.dat", "webcachev01.dat", "WEBCACHEV24.DAT"] {
            assert!(is_database_name(name), "{name}");
        }
        for name in [
            "WebCacheV01.jfm",
            "V01.log",
            "V01.chk",
            "WebCacheV.dat",
            "WebCacheV01.dat.bak",
            "WebCacheVxx.dat",
            "PartitionsEx-WebCacheV01.dat",
            "History",
        ] {
            assert!(!is_database_name(name), "{name}");
        }
    }

    #[test]
    fn packages() {
        assert_eq!(
            package(Some(
                r"C:\Users\a\AppData\Local\Packages\microsoft.microsoftedge_8wekyb3d8bbwe\AC\#!001\MicrosoftEdge\History\"
            )),
            Some("microsoft.microsoftedge_8wekyb3d8bbwe")
        );
        assert_eq!(
            package(Some(
                r"C:\Users\a\AppData\Local\packages\windows_ie_ac_001\AC\INetCache\"
            )),
            Some("windows_ie_ac_001")
        );
        assert_eq!(
            package(Some(
                r"C:\Users\a\AppData\Local\Microsoft\Windows\INetCache\IE\"
            )),
            None
        );
        assert_eq!(package(Some(r"C:\Packages\")), None);
        assert_eq!(package(None), None);
    }

    #[test]
    fn every_row_has_its_tables_columns_in_order() {
        let context = Context {
            source: "WebCacheV01.dat".into(),
            user: Some("alice".into()),
        };
        let edge = r"C:\Users\alice\AppData\Local\Packages\microsoft.microsoftedge_8wekyb3d8bbwe\AC\MicrosoftEdge\User\Default\DownloadHistory\";
        let container = WebCacheContainer {
            id: 21,
            name: "iedownload".into(),
            directory: Some(edge.into()),
            partition: Some("S-1-15-2-1".into()),
            secure_directories: vec!["ABCDEFGH".into(), "IJKLMNOP".into()],
            last_access: Some(Ts::from_unix_seconds(1_576_530_227)),
            last_scavenge: None,
        };
        let visit = Visit {
            id: 4,
            table: "Container_2".into(),
            time: Some(Ts::from_unix_seconds(1_576_530_227)),
            url: "https://example.com/".into(),
            title: String::new(),
            user: Some("alice".into()),
            transition: Transition::NotRecorded,
            from_visit: None,
            duration: None,
            visit_count: Some(3),
            typed: false,
            typed_count: None,
            hidden: false,
            frecency: None,
        };
        let visit = WebCacheVisit {
            visit,
            container: 23,
            created: None,
            modified: Some(Ts::from_local_ticks(
                15_765_374_270_010_000,
                Precision::Tick,
            )),
            expires: None,
            synced: None,
            sync_count: Some(0),
            entry_type: Some(0x20_0004),
            flags: Some(4),
        };
        let download = WebCacheDownload {
            download: Download {
                id: 1,
                url: "https://example.com/a.exe".into(),
                url_chain: Vec::new(),
                target_path: r"C:\Users\alice\Downloads\a.exe".into(),
                current_path: None,
                start: None,
                end: None,
                received_bytes: None,
                total_bytes: Some(10),
                state: None,
                danger_type: None,
                interrupt_reason: None,
                referrer: None,
                tab_url: None,
                mime_type: Some("application/octet-stream".into()),
                opened: None,
                deleted: None,
            },
            container: 21,
            publisher: Some("Example".into()),
            accessed: None,
        };
        let cookie = WebCacheCookie {
            table: "CookieEntryEx_13".into(),
            directory: None,
            id: 7,
            host: "www.example.com".into(),
            path: "/".into(),
            name: String::new(),
            value: "1".into(),
            expires: None,
            modified: None,
            flags: Some(0x2000),
        };
        let entry = WebCacheEntry {
            container: 21,
            container_name: "Content".into(),
            id: 9,
            url: "https://example.com/a.js".into(),
            user: None,
            file_name: Some("a[1].js".into()),
            file_path: None,
            file_size: Some(12),
            entry_type: None,
            flags: None,
            access_count: Some(1),
            created: None,
            modified: None,
            accessed: None,
            expires: None,
            synced: None,
            sync_count: None,
            request_headers: None,
            response_headers: Some("HTTP/1.1 200 OK\r\n\r\n".into()),
            redirect_url: None,
        };
        let names = |row: &Row| -> Vec<String> {
            row.columns.iter().map(|(name, _)| name.clone()).collect()
        };
        let value = |row: &Row, name: &str| -> Json {
            row.columns
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
                .unwrap()
        };
        let row = history_row(&context, &visit, Some("MSHist012019121620191217"), None);
        assert_eq!(names(&row), Table::History.columns());
        assert_eq!(value(&row, "Package"), Json::Null);
        assert_eq!(value(&row, "AccessedTime"), json!("2019-12-16T21:03:47Z"));
        // a day's container's local time: no offset, so -Z leaves it
        assert_eq!(
            value(&row, "ModifiedTime"),
            json!("2019-12-16T23:03:47.001")
        );
        assert_eq!(value(&row, "ContainerID"), json!(23));
        let row = download_row(&context, &download, Some(edge));
        assert_eq!(names(&row), Table::Downloads.columns());
        assert_eq!(
            value(&row, "Package"),
            json!("microsoft.microsoftedge_8wekyb3d8bbwe")
        );
        let row = cookie_row(&context, &cookie);
        assert_eq!(names(&row), Table::Cookies.columns());
        assert_eq!(value(&row, "Name"), json!(""), "an empty name as stored");
        let row = entry_row(&context, &entry, None);
        assert_eq!(names(&row), Table::Entries.columns());
        let row = container_row(&context, &container);
        assert_eq!(names(&row), Table::Containers.columns());
        assert_eq!(
            cell(&value(&row, "SecureDirectories")),
            "ABCDEFGH, IJKLMNOP"
        );
        // the CSV headers are the column names as written
        for table in Table::ALL {
            assert_eq!(
                crate::output_schema::headers(table.columns().iter().copied()).unwrap(),
                table.columns()
            );
        }
    }
}
