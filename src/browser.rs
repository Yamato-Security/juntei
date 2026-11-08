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

//! `parse-sqlite browser`: web browser databases, read without SQLite by Yamato Security's
//! ys-browser-parser. The visits and downloads of Chromium-family browsers' `History`, Firefox's
//! `places.sqlite` (and `downloads.sqlite` up to Firefox 25) and Safari's `History.db`; the
//! cookies of Chromium's `Cookies` and Firefox's `cookies.sqlite`; the form history of Chromium's
//! `Web Data` and Firefox's `formhistory.sqlite`. Each database is read with the committed
//! changes of its `-wal` file, and the deleted visits, pages, downloads and form entries still in
//! free space, on the freelist and in older versions of pages are recovered and marked. One CSV
//! table per kind of record, or one JSON object per row. Hayabusa's input options, progress bar
//! and results summary around it.

use crate::cli::{BrowserOptions, BrowserOutputType};
use crate::color::JunteiColor;
use crate::progress;
use crate::srum::{Ordered, cell};
use crate::util::{self, check_output, error_msg, fmt_num, p, p_label};
use browser::{
    Area, AutofillEntry, Confidence, Cookie, CookieStore, Download, DownloadState, Evidence,
    FormStore, GoogleAnalytics, Kind, Page, PageSource, PageState, Provenance, Visit,
};
use chrono::{DateTime, SecondsFormat, Utc};
use common::time::Ts;
use indicatif::ProgressBar;
use serde_json::{Value as Json, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::{Component, Path, PathBuf};

/// Problems printed per database; the rest are only counted.
const MAX_PRINTED_PROBLEMS: usize = 10;

/// Hosts listed in the summary.
const MAX_HOSTS: usize = 10;

/// The file names `-d` looks for (case ignored); each file found is then told by its contents.
const DATABASE_NAMES: [&str; 10] = [
    "History",
    "Archived History",
    "Cookies",
    "Extension Cookies",
    "Web Data",
    "places.sqlite",
    "downloads.sqlite",
    "cookies.sqlite",
    "formhistory.sqlite",
    "History.db",
];

/// The names in messages that list what is looked for.
const NAMES_SHOWN: &str =
    "History, Cookies, Web Data, places.sqlite, cookies.sqlite, formhistory.sqlite";

/// The tables written, in the order their CSV files are listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Table {
    History,
    Downloads,
    Cookies,
    FormHistory,
}

impl Table {
    const ALL: [Table; 4] = [
        Table::History,
        Table::Downloads,
        Table::Cookies,
        Table::FormHistory,
    ];

    fn name(self) -> &'static str {
        match self {
            Table::History => "History",
            Table::Downloads => "Downloads",
            Table::Cookies => "Cookies",
            Table::FormHistory => "FormHistory",
        }
    }

    fn file_name(self) -> String {
        format!("Browser-{}.csv", self.name())
    }

    /// Every column of the table, in order; each row has them all.
    fn columns(self) -> &'static [&'static str] {
        match self {
            Table::History => &[
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
            ],
            Table::Downloads => &[
                "SourceFile",
                "User",
                "Browser",
                "Profile",
                "Status",
                "StartTime",
                "EndTime",
                "URL",
                "RedirectChain",
                "TargetPath",
                "CurrentPath",
                "ReceivedBytes",
                "TotalBytes",
                "State",
                "DangerType",
                "InterruptReason",
                "Referrer",
                "TabURL",
                "MimeType",
                "Opened",
                "FileDeletedByUser",
                "DownloadID",
                "Confidence",
                "RecoveredFrom",
                "LostColumns",
            ],
            Table::Cookies => &[
                "SourceFile",
                "User",
                "Browser",
                "Profile",
                "Status",
                "Created",
                "LastAccessed",
                "Expires",
                "Host",
                "Name",
                "Value",
                "Path",
                "Secure",
                "HttpOnly",
                "Persistent",
                "GoogleAnalytics",
                "RowID",
            ],
            Table::FormHistory => &[
                "SourceFile",
                "User",
                "Browser",
                "Profile",
                "Status",
                "FirstUsed",
                "LastUsed",
                "RemovedTime",
                "FieldName",
                "Value",
                "Count",
                "GUID",
                "Sources",
                "RowID",
                "Confidence",
                "RecoveredFrom",
                "LostColumns",
            ],
        }
    }
}

/// One row: its table and its columns, in the table's order.
struct Row {
    table: Table,
    columns: Vec<(String, Json)>,
}

/// Where a database is: the Windows user, the browser and the profile its path names.
struct Context {
    source: String,
    user: Option<String>,
    browser: String,
    profile: Option<String>,
}

impl Context {
    /// The columns every row starts with.
    fn start(&self, table: Table, status: &str) -> Row {
        let mut columns = Vec::with_capacity(table.columns().len());
        columns.push(("SourceFile".to_string(), json!(self.source)));
        columns.push(("User".to_string(), optional(self.user.as_deref())));
        columns.push(("Browser".to_string(), json!(self.browser)));
        columns.push(("Profile".to_string(), optional(self.profile.as_deref())));
        columns.push(("Status".to_string(), json!(status)));
        Row { table, columns }
    }
}

impl Row {
    fn push(&mut self, name: &str, value: Json) {
        self.columns.push((name.to_string(), value));
    }

    /// How a deleted entry was recovered; empty for a live one.
    fn recovery(&mut self, provenance: Option<&Provenance>) {
        self.push(
            "Confidence",
            provenance.map_or(Json::Null, |p| json!(confidence(p.confidence))),
        );
        self.push(
            "RecoveredFrom",
            provenance.map_or(Json::Null, |p| json!(recovered_from(p))),
        );
        self.push(
            "LostColumns",
            provenance
                .filter(|p| !p.lost.is_empty())
                .map_or(Json::Null, |p| json!(p.lost)),
        );
    }
}

/// The kinds of database read, as the summary counts them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Group {
    History,
    Cookies,
    FormHistory,
}

impl Group {
    fn of(kind: Kind) -> Option<Self> {
        match kind {
            Kind::ChromiumHistory
            | Kind::FirefoxPlaces
            | Kind::FirefoxDownloads
            | Kind::SafariHistory => Some(Group::History),
            Kind::Cookies => Some(Group::Cookies),
            Kind::Autofill => Some(Group::FormHistory),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Group::History => "History",
            Group::Cookies => "Cookies",
            Group::FormHistory => "Form history",
        }
    }
}

/// Run the command; returns the exit status.
pub fn run(opts: &BrowserOptions) -> i32 {
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
    if output_type == BrowserOutputType::Csv && opts.output.is_none() && opts.output_type.is_some()
    {
        report_error(
            None,
            "CSV output needs -o <DIR>: history, downloads, cookies and form history are each a CSV file.",
        );
        return 1;
    }

    // --- inputs ---------------------------------------------------------------------------
    let (candidates, searched) = match (&opts.filepath, &opts.directory) {
        (Some(f), _) => {
            if !f.is_file() {
                report_error(None, &format!("The file does not exist: {}", f.display()));
                return 1;
            }
            (vec![f.clone()], None)
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
                report_error(None, &not_found(d));
                return 1;
            }
            (found, Some(d.clone()))
        }
        (None, None) => unreachable!("clap requires -f or -d"),
    };
    let wals: Vec<Option<PathBuf>> = candidates.iter().map(|c| wal_of(c)).collect();
    let mut protected = candidates.clone();
    protected.extend(wals.iter().flatten().cloned());
    if let Some(out) = &opts.output
        && let Err(msg) = check_output(out, output_type == BrowserOutputType::Csv, opts.clobber)
    {
        report_error(None, &msg);
        return 1;
    }

    let total: u64 = protected
        .iter()
        .map(|f| f.metadata().map_or(0, |m| m.len()))
        .sum();
    if !data_on_stdout {
        p_label(
            no_color,
            "Browser database files: ",
            &fmt_num(candidates.len()),
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
        progress::progress_bar(candidates.len(), no_color)
    });

    // --- parse and write ---------------------------------------------------------------------
    let mut sink = Sink::new(opts, output_type, protected);
    let mut stats = Stats::default();
    let mut not_read = None;
    for (database, wal) in candidates.iter().zip(&wals) {
        if let Some(b) = &bar {
            b.set_message(progress::message(database));
        }
        let source = database.to_string_lossy().to_string();
        match parse(database, wal.as_deref(), &mut stats) {
            Ok(Outcome::Parsed(parsed)) => {
                stats.problems += parsed.problems.len();
                for problem in parsed.problems.iter().take(MAX_PRINTED_PROBLEMS) {
                    report_error(bar.as_ref(), &format!("{source}: {problem}"));
                }
                if let Err(e) = parsed.rows.iter().try_for_each(|row| sink.write(row)) {
                    report_error(bar.as_ref(), &format!("Could not write the rows: {e}"));
                    return 1;
                }
            }
            Ok(Outcome::NotRead(kind)) => {
                stats.skipped += 1;
                not_read = kind;
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
    if stats.databases == 0 && stats.failed == 0 {
        // every file found by name is something else: nothing to parse, as when none is found
        let message = match &searched {
            Some(d) => not_found(d),
            None => match not_read {
                Some(kind) => format!(
                    "{} is a {kind:?} file, which parse-sqlite browser does not read ({NAMES_SHOWN}).",
                    candidates[0].display()
                ),
                None => format!(
                    "{} is not a browser database that parse-sqlite browser reads ({NAMES_SHOWN}).",
                    candidates[0].display()
                ),
            },
        };
        report_error(None, &message);
        return 1;
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

fn not_found(directory: &Path) -> String {
    format!(
        "No browser databases ({NAMES_SHOWN}) were found in {}",
        directory.display()
    )
}

// --- inputs -------------------------------------------------------------------------------------

/// Whether a file name is one `-d` looks for: `History`, `Cookies`, `places.sqlite`, … (case
/// ignored). The files found are told by their contents; a `History` of another program is
/// skipped.
pub fn is_database_name(name: &str) -> bool {
    DATABASE_NAMES.iter().any(|n| n.eq_ignore_ascii_case(name))
}

/// The files under `dir` named like browser databases, recursively, in path order.
pub fn find_databases(dir: &Path) -> Vec<PathBuf> {
    util::find_files(dir, is_database_name)
}

/// The `-wal` file of a database: its name with `-wal` after it, beside it (case ignored).
fn wal_of(database: &Path) -> Option<PathBuf> {
    let name = database.file_name()?.to_string_lossy().to_string();
    let wanted = format!("{name}-wal");
    let exact = database.with_file_name(&wanted);
    if exact.is_file() {
        return Some(exact);
    }
    let directory = match database.parent() {
        Some(d) if !d.as_os_str().is_empty() => d,
        _ => Path::new("."),
    };
    directory
        .read_dir()
        .ok()?
        .flatten()
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(&wanted)
                && entry.path().is_file()
        })
        .map(|entry| database.with_file_name(entry.file_name()))
}

/// The rows of one database and the damage met.
struct Parsed {
    rows: Vec<Row>,
    problems: Vec<String>,
}

/// What became of a file found or given.
enum Outcome {
    Parsed(Parsed),
    /// Not a database this command reads: another kind of browser file, or none.
    NotRead(Option<Kind>),
}

/// Read one database and its log.
fn parse(database: &Path, wal: Option<&Path>, stats: &mut Stats) -> Result<Outcome, String> {
    let data = std::fs::read(database).map_err(|e| e.to_string())?;
    let name = database
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let kind = browser::detect(&name, &data);
    let Some(group) = kind.and_then(Group::of) else {
        return Ok(Outcome::NotRead(kind));
    };
    let mut problems = Vec::new();
    let log = match wal {
        Some(path) => match std::fs::read(path) {
            Ok(log) => log,
            Err(e) => {
                problems.push(format!(
                    "could not read {} (read without it): {e}",
                    path.display()
                ));
                Vec::new()
            }
        },
        None => Vec::new(),
    };
    let mut context = context(database);
    let mut rows = Vec::new();
    match group {
        Group::History => {
            let history = browser::read(&data, &log).map_err(|e| e.to_string())?;
            context.browser = browser_name(database).unwrap_or_else(|| {
                match history.kind {
                    Kind::FirefoxPlaces | Kind::FirefoxDownloads => "Firefox-based",
                    Kind::SafariHistory => "Safari",
                    _ => "Chromium-based",
                }
                .to_string()
            });
            for visit in &history.visits {
                stats.visit(visit);
                rows.push(visit_row(&context, "Live", visit, None, None));
            }
            for recovered in &history.deleted_visits {
                stats.deleted_visits += 1;
                rows.push(visit_row(
                    &context,
                    "Deleted",
                    &recovered.visit,
                    Some(recovered.page),
                    Some(&recovered.provenance),
                ));
            }
            for recovered in &history.deleted_pages {
                stats.deleted_pages += 1;
                rows.push(page_row(&context, &recovered.page, &recovered.provenance));
            }
            for download in &history.downloads {
                stats.downloads += 1;
                rows.push(download_row(&context, "Live", download, None));
            }
            for recovered in &history.deleted_downloads {
                stats.deleted_downloads += 1;
                rows.push(download_row(
                    &context,
                    "Deleted",
                    &recovered.download,
                    Some(&recovered.provenance),
                ));
            }
            problems.extend(history.problems);
        }
        Group::Cookies => {
            let cookies = browser::read_cookies(&data, &log).map_err(|e| e.to_string())?;
            let firefox = cookies
                .rows
                .first()
                .map_or(name.to_ascii_lowercase().ends_with(".sqlite"), |c| {
                    c.store == CookieStore::Firefox
                });
            context.browser = browser_name(database).unwrap_or_else(|| {
                if firefox {
                    "Firefox-based"
                } else {
                    "Chromium-based"
                }
                .to_string()
            });
            for cookie in &cookies.rows {
                stats.cookies += 1;
                rows.push(cookie_row(&context, cookie));
            }
            problems.extend(cookies.problems);
        }
        Group::FormHistory => {
            let history = browser::read_form_history(&data, &log).map_err(|e| e.to_string())?;
            context.browser = browser_name(database).unwrap_or_else(|| {
                match history.store {
                    FormStore::Chromium => "Chromium-based",
                    FormStore::Firefox => "Firefox-based",
                }
                .to_string()
            });
            for entry in &history.entries {
                stats.form_entries += 1;
                rows.push(form_row(&context, "Live", entry, None));
            }
            for recovered in &history.deleted {
                stats.deleted_form_entries += 1;
                rows.push(form_row(
                    &context,
                    "Deleted",
                    &recovered.entry,
                    Some(&recovered.provenance),
                ));
            }
            for removal in &history.removals {
                stats.removals += 1;
                let mut row = context.start(Table::FormHistory, "Removed");
                row.push("FirstUsed", Json::Null);
                row.push("LastUsed", Json::Null);
                row.push("RemovedTime", time(removal.time));
                row.push("FieldName", Json::Null);
                row.push("Value", Json::Null);
                row.push("Count", Json::Null);
                row.push("GUID", text(&removal.guid));
                row.push("Sources", Json::Null);
                row.push("RowID", json!(removal.rowid));
                row.recovery(None);
                rows.push(row);
            }
            problems.extend(history.problems);
        }
    }
    stats.databases += 1;
    *stats.groups.entry(group).or_default() += 1;
    stats.with_wal += usize::from(!log.is_empty());
    stats.profiles.insert(context.profile_line());
    Ok(Outcome::Parsed(Parsed { rows, problems }))
}

// --- where a database is --------------------------------------------------------------------

/// Browsers by a folder of their profiles' path (lower case, `/`-separated), the more specific
/// first. Windows, then the macOS and Linux spellings where they differ.
const BROWSERS: [(&str, &str); 26] = [
    ("/google/chrome sxs/", "Chrome Canary"),
    ("/google/chrome beta/", "Chrome Beta"),
    ("/google/chrome dev/", "Chrome Dev"),
    ("/google/chrome/", "Chrome"),
    ("/google-chrome/", "Chrome"),
    ("/microsoft/edge sxs/", "Edge Canary"),
    ("/microsoft/edge beta/", "Edge Beta"),
    ("/microsoft/edge dev/", "Edge Dev"),
    ("/microsoft/edge/", "Edge"),
    ("/microsoft edge/", "Edge"),
    ("/microsoft-edge/", "Edge"),
    ("/bravesoftware/brave-browser/", "Brave"),
    ("/opera software/opera gx stable/", "Opera GX"),
    ("/opera software/opera stable/", "Opera"),
    ("/com.operasoftware.opera/", "Opera"),
    ("/vivaldi/", "Vivaldi"),
    ("/yandex/yandexbrowser/", "Yandex Browser"),
    ("/chromium/", "Chromium"),
    ("/tor browser/", "Tor Browser"),
    ("/waterfox/", "Waterfox"),
    ("/librewolf/", "LibreWolf"),
    ("/thunderbird/", "Thunderbird"),
    ("/firefox/profiles/", "Firefox"),
    ("/mozilla/firefox/", "Firefox"),
    ("/apple computer/safari/", "Safari"),
    ("/library/safari/", "Safari"),
];

/// The browser a database's path names, from the path as given and else where it really is.
fn browser_name(database: &Path) -> Option<String> {
    let named = |path: &Path| {
        let text = format!(
            "/{}/",
            path.to_string_lossy().replace('\\', "/").to_lowercase()
        );
        BROWSERS
            .iter()
            .find(|(folder, _)| text.contains(folder))
            .map(|(_, name)| (*name).to_string())
    };
    named(database).or_else(|| {
        std::fs::canonicalize(database)
            .ok()
            .and_then(|path| named(&path))
    })
}

/// The user and profile a database's path names. The user is the folder after the last `Users`
/// (or `Documents and Settings`) of the path as given, never of the analyst's own directories;
/// the profile is the database's folder (the one above `Network`, where Chromium keeps its
/// cookies since version 96).
fn context(database: &Path) -> Context {
    let names: Vec<String> = database
        .components()
        .filter_map(|c| match c {
            Component::Normal(name) => Some(name.to_string_lossy().to_string()),
            _ => None,
        })
        .collect();
    let user = names
        .iter()
        .rposition(|n| {
            n.eq_ignore_ascii_case("Users") || n.eq_ignore_ascii_case("Documents and Settings")
        })
        .filter(|&at| at + 2 < names.len())
        .map(|at| names[at + 1].clone());
    let folders = |path: &Path| -> Vec<String> {
        path.parent()
            .map(|parent| {
                parent
                    .components()
                    .filter_map(|c| match c {
                        Component::Normal(name) => Some(name.to_string_lossy().to_string()),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut parents = folders(database);
    if parents.len() < 2 {
        // a database in the current directory: its folder's own name
        if let Ok(path) = std::fs::canonicalize(database) {
            parents = folders(&path);
        }
    }
    let profile = match parents.as_slice() {
        [.., profile, network] if network.eq_ignore_ascii_case("Network") => Some(profile.clone()),
        [.., profile] => Some(profile.clone()),
        [] => None,
    };
    Context {
        source: database.to_string_lossy().to_string(),
        user,
        browser: String::new(),
        profile,
    }
}

impl Context {
    /// How the summary lists the profile: `Chrome: Default (alice)`.
    fn profile_line(&self) -> String {
        let mut line = format!(
            "{}: {}",
            self.browser,
            self.profile.as_deref().unwrap_or("(no folder)")
        );
        if let Some(user) = &self.user {
            line.push_str(&format!(" (user {user})"));
        }
        line
    }
}

// --- rows -----------------------------------------------------------------------------------

fn visit_row(
    context: &Context,
    status: &str,
    visit: &Visit,
    page: Option<PageSource>,
    provenance: Option<&Provenance>,
) -> Row {
    let mut row = context.start(Table::History, status);
    row.push("RecordType", json!("Visit"));
    row.push("VisitTime", time(visit.time));
    row.push("URL", text(&visit.url));
    row.push("Title", text(&visit.title));
    row.push("Transition", text(&visit.transition.to_string()));
    row.push(
        "VisitDuration",
        visit
            .duration
            .map_or(Json::Null, |d| json!(d.as_micros() as f64 / 1_000_000.0)),
    );
    row.push("VisitCount", json!(visit.visit_count));
    row.push("Typed", json!(visit.typed));
    row.push("TypedCount", json!(visit.typed_count));
    row.push("Hidden", json!(visit.hidden));
    row.push("Frecency", json!(visit.frecency));
    row.push("VisitID", id(Some(visit.id)));
    row.push("FromVisitID", id(visit.from_visit));
    row.push("LastVisitTime", Json::Null);
    row.push(
        "PageSource",
        page.map_or(Json::Null, |page| {
            json!(match page {
                PageSource::Live => "Live",
                PageSource::Recovered => "Recovered",
                PageSource::NotFound => "NotFound",
            })
        }),
    );
    row.recovery(provenance);
    row
}

/// A deleted page whose URL no live page has: its visits may be gone too.
fn page_row(context: &Context, page: &Page, provenance: &Provenance) -> Row {
    let mut row = context.start(Table::History, "Deleted");
    row.push("RecordType", json!("Page"));
    row.push("VisitTime", Json::Null);
    row.push("URL", text(&page.url));
    row.push("Title", text(&page.title));
    row.push("Transition", Json::Null);
    row.push("VisitDuration", Json::Null);
    row.push("VisitCount", json!(page.visit_count));
    row.push("Typed", json!(page.typed));
    row.push("TypedCount", json!(page.typed_count));
    row.push("Hidden", json!(page.hidden));
    row.push("Frecency", json!(page.frecency));
    row.push("VisitID", Json::Null);
    row.push("FromVisitID", Json::Null);
    row.push("LastVisitTime", time(page.last_visit));
    row.push("PageSource", Json::Null);
    row.recovery(Some(provenance));
    row
}

fn download_row(
    context: &Context,
    status: &str,
    download: &Download,
    provenance: Option<&Provenance>,
) -> Row {
    let mut row = context.start(Table::Downloads, status);
    row.push("StartTime", time(download.start));
    row.push("EndTime", time(download.end));
    row.push("URL", text(&download.url));
    row.push(
        "RedirectChain",
        if download.url_chain.is_empty() {
            Json::Null
        } else {
            json!(download.url_chain)
        },
    );
    row.push("TargetPath", text(&download.target_path));
    row.push("CurrentPath", optional(download.current_path.as_deref()));
    row.push("ReceivedBytes", json!(download.received_bytes));
    row.push("TotalBytes", json!(download.total_bytes));
    row.push(
        "State",
        download
            .state
            .map_or(Json::Null, |state| json!(state_name(state))),
    );
    row.push("DangerType", json!(download.danger_type));
    row.push("InterruptReason", json!(download.interrupt_reason));
    row.push("Referrer", optional(download.referrer.as_deref()));
    row.push("TabURL", optional(download.tab_url.as_deref()));
    row.push("MimeType", optional(download.mime_type.as_deref()));
    row.push("Opened", json!(download.opened));
    row.push("FileDeletedByUser", json!(download.deleted));
    row.push("DownloadID", id(Some(download.id)));
    row.recovery(provenance);
    row
}

fn state_name(state: DownloadState) -> String {
    match state {
        DownloadState::InProgress => "InProgress".into(),
        DownloadState::Complete => "Complete".into(),
        DownloadState::Cancelled => "Cancelled".into(),
        DownloadState::Interrupted => "Interrupted".into(),
        DownloadState::Paused => "Paused".into(),
        DownloadState::Blocked => "Blocked".into(),
        DownloadState::Dirty => "Dirty".into(),
        DownloadState::Other(value) => value.to_string(),
    }
}

fn cookie_row(context: &Context, cookie: &Cookie) -> Row {
    let mut row = context.start(Table::Cookies, "Live");
    row.push("Created", time(cookie.created));
    row.push("LastAccessed", time(cookie.last_accessed));
    row.push("Expires", time(cookie.expires));
    // A cookie's name can be empty, as set: kept as stored, as its host and path. An empty
    // value is most often one Chromium encrypted: nothing.
    row.push("Host", json!(cookie.host));
    row.push("Name", json!(cookie.name));
    row.push("Value", text(&cookie.value));
    row.push("Path", json!(cookie.path));
    row.push("Secure", json!(cookie.secure));
    row.push("HttpOnly", json!(cookie.http_only));
    row.push("Persistent", json!(cookie.persistent));
    row.push(
        "GoogleAnalytics",
        cookie
            .analytics
            .as_ref()
            .map_or(Json::Null, |a| json!(analytics(a))),
    );
    row.push("RowID", json!(cookie.rowid));
    row
}

/// A Google Analytics cookie's decoded fields: `name: value; …`.
fn analytics(value: &GoogleAnalytics) -> String {
    let mut fields: Vec<(&str, String)> = Vec::new();
    let mut add_text = |name, value: &Option<String>| {
        if let Some(value) = value {
            fields.push((name, value.clone()));
        }
    };
    let when = |t: &Option<Ts>| time(*t).as_str().map(str::to_string);
    match value {
        GoogleAnalytics::Utma {
            domain_hash,
            visitor_id,
            first_visit,
            previous_visit,
            last_visit,
            sessions,
        } => {
            add_text("DomainHash", domain_hash);
            add_text("VisitorID", visitor_id);
            add_text("FirstVisit", &when(first_visit));
            add_text("PreviousVisit", &when(previous_visit));
            add_text("LastVisit", &when(last_visit));
            add_text("Sessions", &sessions.map(|n| n.to_string()));
        }
        GoogleAnalytics::Utmb {
            domain_hash,
            pages_viewed,
            last_visit,
        } => {
            add_text("DomainHash", domain_hash);
            add_text("PagesViewed", &pages_viewed.map(|n| n.to_string()));
            add_text("LastVisit", &when(last_visit));
        }
        GoogleAnalytics::Utmt { last_visit } => add_text("LastVisit", &when(last_visit)),
        GoogleAnalytics::Utmz {
            domain_hash,
            last_visit,
            sessions,
            sources,
            variables,
        } => {
            add_text("DomainHash", domain_hash);
            add_text("LastVisit", &when(last_visit));
            add_text("Sessions", &sessions.map(|n| n.to_string()));
            add_text("Sources", &sources.map(|n| n.to_string()));
            for (name, value) in variables {
                fields.push((name.as_str(), value.clone()));
            }
        }
    }
    fields
        .iter()
        .map(|(name, value)| format!("{name}: {value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

fn form_row(
    context: &Context,
    status: &str,
    entry: &AutofillEntry,
    provenance: Option<&Provenance>,
) -> Row {
    let mut row = context.start(Table::FormHistory, status);
    row.push("FirstUsed", time(entry.created));
    row.push("LastUsed", time(entry.last_used));
    row.push("RemovedTime", Json::Null);
    row.push("FieldName", text(&entry.field));
    row.push("Value", text(&entry.value));
    row.push("Count", json!(entry.count));
    row.push("GUID", optional(entry.guid.as_deref()));
    row.push(
        "Sources",
        if entry.sources.is_empty() {
            Json::Null
        } else {
            json!(entry.sources)
        },
    );
    row.push("RowID", json!(entry.rowid));
    row.recovery(provenance);
    row
}

// --- values ---------------------------------------------------------------------------------

/// A UTC time as RFC 3339, like the timestamps of the other commands; nothing for a time not
/// set (zero) or out of range.
fn time(t: Option<Ts>) -> Json {
    t.and_then(|t| t.ticks())
        .and_then(|ticks| {
            DateTime::<Utc>::from_timestamp(
                ticks.div_euclid(10_000_000),
                (ticks.rem_euclid(10_000_000) * 100) as u32,
            )
        })
        .map_or(Json::Null, |t| {
            json!(t.to_rfc3339_opts(SecondsFormat::AutoSi, true))
        })
}

fn text(value: &str) -> Json {
    if value.is_empty() {
        Json::Null
    } else {
        json!(value)
    }
}

fn optional(value: Option<&str>) -> Json {
    value.map_or(Json::Null, text)
}

/// A row id: nothing when a recovered record lost it (0) or there is none.
fn id(value: Option<i64>) -> Json {
    value.filter(|&v| v != 0).map_or(Json::Null, |v| json!(v))
}

fn confidence(value: Confidence) -> &'static str {
    match value {
        Confidence::High => "High",
        Confidence::Medium => "Medium",
        Confidence::Low => "Low",
    }
}

/// Where a deleted entry's record was found, and how much of it was read: `Freeblock of in-use
/// page 12, offset 3456`. WAL frames count from 0.
fn recovered_from(p: &Provenance) -> String {
    let area = match p.area {
        Area::Cell => "Cell",
        Area::Freeblock => "Freeblock",
        Area::Unallocated => "Unallocated space",
        Area::FreePage => "Free page",
    };
    let page = match p.page_state {
        PageState::InUse => format!("in-use page {}", p.page),
        PageState::FreelistTrunk => format!("freelist trunk page {}", p.page),
        PageState::FreelistLeaf => format!("freelist leaf page {}", p.page),
        PageState::Superseded { frame } => {
            format!("superseded WAL frame {frame} (page {})", p.page)
        }
        PageState::Uncommitted { frame } => {
            format!("uncommitted WAL frame {frame} (page {})", p.page)
        }
        PageState::Invalid { frame } => {
            format!("invalid WAL frame {frame} (page {})", p.page)
        }
        PageState::ReplacedInFile => format!(
            "the database file's copy of page {} (replaced by the WAL)",
            p.page
        ),
    };
    let mut text = format!("{area} of {page}, offset {}", p.offset);
    match p.evidence {
        Evidence::CellPointer | Evidence::Cell => {}
        Evidence::Record => text.push_str("; cell header overwritten"),
        Evidence::Shape => text.push_str("; record header rebuilt"),
    }
    if p.truncated {
        text.push_str("; overflow pages lost");
    }
    if !p.also_fits.is_empty() {
        text.push_str(&format!("; also fits {}", p.also_fits.join(", ")));
    }
    text
}

/// A CSV cell: lists of URLs joined so that commas in them stay readable.
fn csv_cell(name: &str, value: &Json) -> String {
    match (name, value) {
        ("RedirectChain", Json::Array(urls)) => {
            urls.iter().map(cell).collect::<Vec<_>>().join(" -> ")
        }
        ("Sources", Json::Array(sources)) => {
            sources.iter().map(cell).collect::<Vec<_>>().join(" | ")
        }
        _ => cell(value),
    }
}

// --- writers --------------------------------------------------------------------------------

/// What was saved with `-o`.
enum Saved {
    /// The number of CSV files.
    Tables(usize),
    File,
}

/// Where the rows go, opened when the first database has been read: four CSV files (each with
/// its header, rows or not), one JSON or JSONL file, stdout, or nowhere (the summary only).
struct Sink {
    output: Option<PathBuf>,
    output_type: BrowserOutputType,
    stdout: bool,
    protected: Vec<PathBuf>,
    csv: Option<Vec<csv::Writer<BufWriter<File>>>>,
    json: Option<Box<dyn Write>>,
}

impl Sink {
    fn new(opts: &BrowserOptions, output_type: BrowserOutputType, protected: Vec<PathBuf>) -> Self {
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
            (Some(dir), BrowserOutputType::Csv) => {
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
            writer.write_record(
                row.columns
                    .iter()
                    .map(|(name, value)| csv_cell(name, value)),
            )?;
        } else if let Some(out) = &mut self.json {
            let mut columns = row.columns.clone();
            columns.insert(1, ("Table".to_string(), json!(row.table.name())));
            crate::output_schema::write_json(
                &mut *out,
                &Ordered(&columns),
                self.output_type == BrowserOutputType::Json,
            )?;
            out.write_all(b"\n")?;
        }
        Ok(())
    }

    /// Finish the files; the number of CSV files written, if any.
    /// `parsed`: whether any database was read. One without rows still makes its tables'
    /// headers; when none could be read, nothing is written.
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
    skipped: usize,
    with_wal: usize,
    problems: usize,
    groups: BTreeMap<Group, usize>,
    profiles: BTreeSet<String>,
    visits: usize,
    deleted_visits: usize,
    deleted_pages: usize,
    downloads: usize,
    deleted_downloads: usize,
    cookies: usize,
    form_entries: usize,
    deleted_form_entries: usize,
    removals: usize,
    first: Option<String>,
    last: Option<String>,
    hosts: HashMap<String, usize>,
}

impl Stats {
    fn visit(&mut self, visit: &Visit) {
        self.visits += 1;
        if let Some(t) = time(visit.time).as_str().map(str::to_string) {
            let earlier = |a: &str, b: &str| {
                DateTime::parse_from_rfc3339(a).ok() < DateTime::parse_from_rfc3339(b).ok()
            };
            if self.first.as_deref().is_none_or(|f| earlier(&t, f)) {
                self.first = Some(t.clone());
            }
            if self.last.as_deref().is_none_or(|l| earlier(l, &t)) {
                self.last = Some(t);
            }
        }
        if let Some(host) = host(&visit.url) {
            *self.hosts.entry(host.to_ascii_lowercase()).or_default() += 1;
        }
    }
}

/// A URL's host (and port): between `scheme://` (and any `user@`) and the path.
fn host(url: &str) -> Option<&str> {
    let (_, rest) = url.split_once("://")?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..end];
    let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    (!host.is_empty()).then_some(host)
}

fn print_summary(s: &Stats, no_color: bool) {
    p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
    let mut parsed = fmt_num(s.databases);
    if s.failed > 0 {
        parsed.push_str(&format!(" (could not parse: {})", fmt_num(s.failed)));
    }
    p_label(no_color, "Browser databases parsed: ", &parsed);
    for (group, n) in &s.groups {
        p(None, &format!("  {}: {}", group.label(), fmt_num(*n)), true);
    }
    if s.skipped > 0 {
        p_label(
            no_color,
            "Files skipped (not browser databases): ",
            &fmt_num(s.skipped),
        );
    }
    p_label(
        no_color,
        "Read with their write-ahead log (-wal): ",
        &fmt_num(s.with_wal),
    );
    p_label(no_color, "Profiles: ", &fmt_num(s.profiles.len()));
    for profile in &s.profiles {
        p(None, &format!("  {profile}"), true);
    }
    let with_deleted = |live: usize, deleted: usize| {
        if deleted == 0 {
            fmt_num(live)
        } else {
            format!(
                "{} (deleted, recovered: {})",
                fmt_num(live),
                fmt_num(deleted)
            )
        }
    };
    p_label(
        no_color,
        "Visits: ",
        &with_deleted(s.visits, s.deleted_visits),
    );
    if s.deleted_pages > 0 {
        p_label(
            no_color,
            "Deleted pages recovered: ",
            &fmt_num(s.deleted_pages),
        );
    }
    p_label(
        no_color,
        "Downloads: ",
        &with_deleted(s.downloads, s.deleted_downloads),
    );
    p_label(no_color, "Cookies: ", &fmt_num(s.cookies));
    let mut forms = with_deleted(s.form_entries, s.deleted_form_entries);
    if s.removals > 0 {
        forms.push_str(&format!(" (removal records: {})", fmt_num(s.removals)));
    }
    p_label(no_color, "Form entries: ", &forms);
    if let (Some(first), Some(last)) = (&s.first, &s.last) {
        p_label(no_color, "Visit times: ", &format!("{first} - {last}"));
    }
    let mut hosts: Vec<(&String, &usize)> = s.hosts.iter().collect();
    hosts.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    if !hosts.is_empty() {
        p(
            JunteiColor::Green.rdg(no_color),
            "Most visited hosts (live visits):",
            true,
        );
        for (host, visits) in hosts.into_iter().take(MAX_HOSTS) {
            p(None, &format!("  {:>8}  {host}", fmt_num(*visits)), true);
        }
    }
    p_label(no_color, "Problems: ", &fmt_num(s.problems));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_names() {
        for name in [
            "History",
            "history",
            "Archived History",
            "Cookies",
            "Extension Cookies",
            "Web Data",
            "places.sqlite",
            "PLACES.SQLITE",
            "downloads.sqlite",
            "cookies.sqlite",
            "formhistory.sqlite",
            "History.db",
        ] {
            assert!(is_database_name(name), "{name}");
        }
        for name in [
            "History-journal",
            "places.sqlite-wal",
            "Login Data",
            "WebCacheV01.dat",
            "History.txt",
            "Current.mdb",
        ] {
            assert!(!is_database_name(name), "{name}");
        }
    }

    #[test]
    fn browsers_users_and_profiles_from_the_path() {
        let cases = [
            (
                "C/Users/alice/AppData/Local/Google/Chrome/User Data/Default/History",
                Some("Chrome"),
                Some("alice"),
                Some("Default"),
            ),
            (
                "C/Users/bob/AppData/Local/Microsoft/Edge/User Data/Profile 1/Network/Cookies",
                Some("Edge"),
                Some("bob"),
                Some("Profile 1"),
            ),
            (
                "C/Users/carol/AppData/Roaming/Mozilla/Firefox/Profiles/x1y2.default-release/places.sqlite",
                Some("Firefox"),
                Some("carol"),
                Some("x1y2.default-release"),
            ),
            (
                "C/Documents and Settings/dave/Local Settings/Application Data/Google/Chrome/User Data/Default/History",
                Some("Chrome"),
                Some("dave"),
                Some("Default"),
            ),
            (
                "C/Users/erin/AppData/Local/BraveSoftware/Brave-Browser/User Data/Default/Web Data",
                Some("Brave"),
                Some("erin"),
                Some("Default"),
            ),
            (
                "C/Users/frank/AppData/Roaming/Opera Software/Opera Stable/Default/History",
                Some("Opera"),
                Some("frank"),
                Some("Default"),
            ),
            (
                "C/Users/grace/AppData/Local/Google/Chrome SxS/User Data/Default/History",
                Some("Chrome Canary"),
                Some("grace"),
                Some("Default"),
            ),
            (
                "export/Users/ignored/case/Users/heidi/AppData/Local/Packages/App/EBWebView/Default/History",
                None,
                Some("heidi"),
                Some("Default"),
            ),
        ];
        for (path, browser, user, profile) in cases {
            let path = Path::new(path);
            let found = context(path);
            assert_eq!(found.user.as_deref(), user, "{}", path.display());
            assert_eq!(found.profile.as_deref(), profile, "{}", path.display());
            assert_eq!(browser_name(path).as_deref(), browser, "{}", path.display());
        }
        // a Users folder must have a folder after it, below which the database is
        assert_eq!(context(Path::new("Users/History")).user, None);
    }

    #[test]
    fn hosts_of_urls() {
        assert_eq!(host("https://example.com/a?b"), Some("example.com"));
        assert_eq!(host("http://127.0.0.1:8765"), Some("127.0.0.1:8765"));
        assert_eq!(
            host("ftp://user:pw@files.example.org/x"),
            Some("files.example.org")
        );
        assert_eq!(host("https://example.com#top"), Some("example.com"));
        assert_eq!(host("file:///C:/x.html"), None);
        assert_eq!(host("about:blank"), None);
    }

    #[test]
    fn times_and_ids() {
        assert_eq!(time(None), Json::Null);
        assert_eq!(time(Some(Ts::from_unix_micros(0))), Json::Null);
        assert_eq!(
            time(Some(Ts::from_unix_micros(1_794_055_003_233_828))),
            json!("2026-11-07T12:36:43.233828Z")
        );
        assert_eq!(
            time(Some(Ts::from_unix_seconds(1_794_055_003))),
            json!("2026-11-07T12:36:43Z")
        );
        assert_eq!(id(Some(0)), Json::Null);
        assert_eq!(id(None), Json::Null);
        assert_eq!(id(Some(7)), json!(7));
    }

    #[test]
    fn where_recovered_records_were() {
        let mut provenance = Provenance {
            table: "moz_historyvisits".into(),
            rowid: Some(4),
            page: 12,
            offset: 3456,
            page_state: PageState::InUse,
            area: Area::Freeblock,
            evidence: Evidence::Cell,
            confidence: Confidence::High,
            also_fits: Vec::new(),
            lost: Vec::new(),
            truncated: false,
        };
        assert_eq!(
            recovered_from(&provenance),
            "Freeblock of in-use page 12, offset 3456"
        );
        provenance.page_state = PageState::Superseded { frame: 3 };
        provenance.area = Area::Cell;
        provenance.evidence = Evidence::Shape;
        provenance.truncated = true;
        provenance.also_fits = vec!["moz_places".into()];
        assert_eq!(
            recovered_from(&provenance),
            "Cell of superseded WAL frame 3 (page 12), offset 3456; record header rebuilt; overflow pages lost; also fits moz_places"
        );
        provenance.page_state = PageState::ReplacedInFile;
        provenance.evidence = Evidence::Record;
        provenance.truncated = false;
        provenance.also_fits.clear();
        assert_eq!(
            recovered_from(&provenance),
            "Cell of the database file's copy of page 12 (replaced by the WAL), offset 3456; cell header overwritten"
        );
    }

    #[test]
    fn lists_in_csv_cells() {
        assert_eq!(
            csv_cell("RedirectChain", &json!(["http://a/?x=1,2", "https://b/"])),
            "http://a/?x=1,2 -> https://b/"
        );
        assert_eq!(csv_cell("Sources", &json!(["a", "b"])), "a | b");
        assert_eq!(
            csv_cell("LostColumns", &json!(["title", "url"])),
            "title, url"
        );
        assert_eq!(csv_cell("Typed", &json!(true)), "true");
        assert_eq!(csv_cell("Title", &Json::Null), "");
    }

    #[test]
    fn every_row_has_its_tables_columns_in_order() {
        let context = Context {
            source: "History".into(),
            user: None,
            browser: "Chrome".into(),
            profile: Some("Default".into()),
        };
        let visit = Visit {
            id: 1,
            table: "visits".into(),
            time: Some(Ts::from_unix_micros(1)),
            url: "https://example.com/".into(),
            title: String::new(),
            user: None,
            transition: browser::Transition::NotRecorded,
            from_visit: Some(0),
            duration: Some(std::time::Duration::from_micros(5_123_456)),
            visit_count: Some(1),
            typed: false,
            typed_count: None,
            hidden: false,
            frecency: None,
        };
        let row = visit_row(&context, "Live", &visit, None, None);
        let names: Vec<&str> = row.columns.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, Table::History.columns());
        assert_eq!(row.columns[10].1, json!(5.123456));
        assert_eq!(row.columns[17].1, Json::Null, "from_visit 0 is no visit");
        let provenance = Provenance {
            table: "urls".into(),
            rowid: None,
            page: 2,
            offset: 100,
            page_state: PageState::FreelistLeaf,
            area: Area::Cell,
            evidence: Evidence::CellPointer,
            confidence: Confidence::Low,
            also_fits: Vec::new(),
            lost: vec!["title".into()],
            truncated: false,
        };
        let row = page_row(&context, &Page::default(), &provenance);
        let names: Vec<&str> = row.columns.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, Table::History.columns());
        let download = Download {
            id: 0,
            url: String::new(),
            url_chain: Vec::new(),
            target_path: String::new(),
            current_path: None,
            start: None,
            end: None,
            received_bytes: None,
            total_bytes: None,
            state: Some(DownloadState::Other(9)),
            danger_type: None,
            interrupt_reason: None,
            referrer: None,
            tab_url: None,
            mime_type: None,
            opened: None,
            deleted: None,
        };
        let row = download_row(&context, "Deleted", &download, Some(&provenance));
        let names: Vec<&str> = row.columns.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, Table::Downloads.columns());
        let cookie = Cookie {
            store: CookieStore::Chromium,
            rowid: 1,
            host: ".example.com".into(),
            name: "__utmb".into(),
            value: "1.2.3.4".into(),
            path: "/".into(),
            created: None,
            last_accessed: None,
            expires: None,
            secure: true,
            http_only: false,
            persistent: Some(true),
            analytics: Some(GoogleAnalytics::Utmb {
                domain_hash: Some("1".into()),
                pages_viewed: Some(2),
                last_visit: Some(Ts::from_unix_seconds(1_000_000_000)),
            }),
        };
        let row = cookie_row(&context, &cookie);
        let names: Vec<&str> = row.columns.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, Table::Cookies.columns());
        assert_eq!(
            row.columns[15].1,
            json!("DomainHash: 1; PagesViewed: 2; LastVisit: 2001-09-09T01:46:40Z")
        );
        let entry = AutofillEntry {
            store: FormStore::Firefox,
            rowid: 3,
            field: "q".into(),
            value: "forensics".into(),
            count: Some(2),
            created: None,
            last_used: None,
            guid: Some("abc".into()),
            sources: vec!["https://example.com/".into()],
        };
        let row = form_row(&context, "Live", &entry, None);
        let names: Vec<&str> = row.columns.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, Table::FormHistory.columns());
        // the CSV headers are the column names as written
        for table in Table::ALL {
            assert_eq!(
                crate::output_schema::headers(table.columns().iter().copied()).unwrap(),
                table.columns()
            );
        }
    }
}
