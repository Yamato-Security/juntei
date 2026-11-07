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

//! `parse-ese ual`: Windows Server User Access Logging (UAL) databases in
//! `System32\LogFiles\Sum`, read with the ESE reader of Yamato Security's ys-srum-parser.
//! `Current.mdb` and the yearly `{GUID}.mdb` files say which server roles were used, from which
//! client addresses, by which accounts and how often on each day; they also hold the DNS names
//! and virtual machines seen. `SystemIdentity.mdb` describes the server and names the roles and
//! the yearly files. Every row of every table, as one CSV table per UAL table or one JSON object
//! per row. Hayabusa's input options, progress bar and results summary around it.

use crate::cli::{UalOptions, UalOutputType};
use crate::color::JunteiColor;
use crate::progress;
use crate::srum::{CsvTable, Ordered, cell, guid, json_value, safe_file_name};
use crate::util::{self, check_output, error_msg, fmt_num, p, p_label};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, SecondsFormat, TimeDelta, Utc};
use ese::{Database, DatabaseState, Table, Value};
use indicatif::ProgressBar;
use serde_json::{Value as Json, json};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};

/// Problems printed per database; the rest are only counted.
const MAX_PRINTED_PROBLEMS: usize = 10;

/// The tables of the UAL databases, in the order their CSV files are listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Kind {
    Clients,
    RoleAccess,
    Dns,
    VirtualMachines,
    SystemIdentity,
    ChainedDatabases,
    Roles,
}

impl Kind {
    const ALL: [Kind; 7] = [
        Kind::Clients,
        Kind::RoleAccess,
        Kind::Dns,
        Kind::VirtualMachines,
        Kind::SystemIdentity,
        Kind::ChainedDatabases,
        Kind::Roles,
    ];

    /// The table's name in the database.
    fn table(self) -> &'static str {
        match self {
            Kind::Clients => "CLIENTS",
            Kind::RoleAccess => "ROLE_ACCESS",
            Kind::Dns => "DNS",
            Kind::VirtualMachines => "VIRTUALMACHINES",
            Kind::SystemIdentity => "SYSTEM_IDENTITY",
            Kind::ChainedDatabases => "CHAINED_DATABASES",
            Kind::Roles => "ROLE_IDS",
        }
    }

    /// The table of this name (ignoring case, as ESE does).
    fn of_table(name: &str) -> Option<Kind> {
        Kind::ALL
            .into_iter()
            .find(|k| k.table().eq_ignore_ascii_case(name))
    }

    /// The name of its CSV file (`UAL-Clients.csv`) and of its rows in the summary.
    fn output_name(self) -> &'static str {
        match self {
            Kind::Clients => "Clients",
            Kind::RoleAccess => "RoleAccess",
            Kind::Dns => "DNS",
            Kind::VirtualMachines => "VirtualMachines",
            Kind::SystemIdentity => "SystemIdentity",
            Kind::ChainedDatabases => "ChainedDatabases",
            Kind::Roles => "Roles",
        }
    }
}

/// A table of the output: a known UAL table, or another table under its own name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TableKey {
    Known(Kind),
    Other(String),
}

impl TableKey {
    fn of_table(name: &str) -> Self {
        Kind::of_table(name).map_or_else(|| TableKey::Other(name.to_string()), TableKey::Known)
    }

    fn output_name(&self) -> &str {
        match self {
            TableKey::Known(kind) => kind.output_name(),
            TableKey::Other(name) => name,
        }
    }
}

/// The roles Windows Server defines, for databases without their `SystemIdentity.mdb`
/// (`ROLE_IDS` names the same roles in every version).
const ROLE_NAMES: [(&str, &str); 14] = [
    (
        "{C50FCC83-BC8D-4DF5-8A3D-89D7F80F074B}",
        "Active Directory Certificate Services",
    ),
    (
        "{AD495FC3-0EAA-413D-BA7D-8B13FA7EC598}",
        "Active Directory Domain Services",
    ),
    (
        "{B4CDD739-089C-417E-878D-855F90081BE7}",
        "Active Directory Rights Management Service",
    ),
    ("{910CBAF9-B612-4782-A21F-F7C75105434A}", "BranchCache"),
    ("{48EED6B2-9CDC-4358-B5A5-8DEA3B2F3F6A}", "DHCP Server"),
    ("{7CC4B071-292C-4732-97A1-CF9A7301195D}", "FAX Server"),
    ("{C23F1C6A-30A8-41B6-BBF7-F266563DFCD6}", "FTP Server"),
    ("{10A9226F-50EE-49D8-A393-9A501D47CE04}", "File Server"),
    (
        "{BBD85B29-9DCC-4FD9-865D-3846DCBA75C7}",
        "Network Policy and Access Services",
    ),
    (
        "{7FB09BD3-7FE6-435E-8348-7D8AEFB6CEA3}",
        "Print and Document Services",
    ),
    ("{952285D9-EDB7-4B6B-9D85-0C09E3DA0BBD}", "Remote Access"),
    ("{D6256CF7-98FB-4EB4-AA18-303F1DA1F770}", "Web Server"),
    (
        "{4116A14D-3840-4F42-A67F-F2F9FF46EB4C}",
        "Windows Deployment Services",
    ),
    (
        "{D8DC1C8E-EA13-49CE-9A68-C9DCA8DB8B33}",
        "Windows Server Update Services",
    ),
];

/// What a server's `SystemIdentity.mdb` says about the other databases of its directory.
#[derive(Default)]
struct Identity {
    /// Role names by role GUID (`ROLE_IDS`).
    roles: HashMap<String, String>,
    /// The year of each yearly database, by lower-case file name (`CHAINED_DATABASES`).
    years: HashMap<String, i32>,
    /// Damage in lookup metadata must not disappear when only Current.mdb was requested.
    problems: Vec<String>,
}

/// One row of a database table, as written.
struct Record {
    /// The table's name in the database.
    table: String,
    key: TableKey,
    /// The columns, in the table's order: times in UTC, GUIDs in braces, client addresses as IP
    /// addresses. `RoleName` follows `RoleGuid`; a client's day counters end the row as
    /// `DailyAccesses`.
    columns: Vec<(String, Json)>,
}

impl Record {
    fn get(&self, name: &str) -> Option<&Json> {
        self.columns.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }
}

/// The rows of one database.
struct Parsed {
    /// Each table's columns as written (as [`Record::columns`] names them), for the header of a
    /// table without rows.
    tables: Vec<(TableKey, Vec<String>)>,
    records: Vec<Record>,
    dirty: bool,
    problems: Vec<String>,
}

/// Run the command; returns the exit status.
pub fn run(opts: &UalOptions) -> i32 {
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
    if output_type == UalOutputType::Csv && opts.output.is_none() && opts.output_type.is_some() {
        report_error(
            None,
            "CSV output needs -o <DIR>: every UAL table is its own CSV file.",
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
                        "No UAL databases (Current.mdb, {{GUID}}.mdb, SystemIdentity.mdb) were found in {}",
                        d.display()
                    ),
                );
                return 1;
            }
            found
        }
        (None, None) => unreachable!("clap requires -f or -d"),
    };
    let mut protected = databases.clone();
    protected.extend(databases.iter().filter_map(|d| identity_database(d)));
    let out_is_dir = output_type == UalOutputType::Csv;
    if let Some(out) = &opts.output
        && let Err(msg) = check_output(out, out_is_dir, opts.clobber)
    {
        report_error(None, &msg);
        return 1;
    }

    let total: u64 = databases
        .iter()
        .map(|d| d.metadata().map_or(0, |m| m.len()))
        .sum();
    if !data_on_stdout {
        p_label(no_color, "UAL databases: ", &fmt_num(databases.len()));
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

    // --- parse -----------------------------------------------------------------------------
    let mut stats = Stats::default();
    let mut identities: HashMap<PathBuf, Option<Identity>> = HashMap::new();
    let mut parsed = Vec::new();
    for database in &databases {
        let source = database.to_string_lossy().to_string();
        if let Some(b) = &bar {
            b.set_message(progress::message(database));
        }
        // the role names and years of the SystemIdentity.mdb beside it, read once per directory
        let directory = database.parent().unwrap_or(Path::new("")).to_path_buf();
        let identity = identities.entry(directory).or_insert_with(|| {
            let path = identity_database(database)?;
            match std::fs::read(&path)
                .map_err(|e| e.to_string())
                .and_then(|data| read_identity(&data))
            {
                Ok(identity) => {
                    // Directory input will report these problems when parsing this same database
                    // below. A sidecar read for -f must report its diagnostics here instead.
                    if !databases.contains(&path) {
                        stats.problems += identity.problems.len();
                        for problem in identity.problems.iter().take(MAX_PRINTED_PROBLEMS) {
                            report_error(bar.as_ref(), &format!("{}: {problem}", path.display()));
                        }
                    }
                    Some(identity)
                }
                Err(e) => {
                    if !databases.contains(&path) {
                        stats.problems += 1;
                    }
                    report_error(
                        bar.as_ref(),
                        &format!(
                            "Could not read {} (built-in role names are used): {e}",
                            path.display()
                        ),
                    );
                    None
                }
            }
        });
        let name = database
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let result = std::fs::read(database)
            .map_err(|e| e.to_string())
            .and_then(|data| read(&data, &name, identity.as_ref()).map_err(|e| e.to_string()));
        match result {
            Ok(database) => {
                for problem in database.problems.iter().take(MAX_PRINTED_PROBLEMS) {
                    report_error(bar.as_ref(), &format!("{source}: {problem}"));
                }
                stats.add(
                    &database,
                    identity.as_ref().is_some_and(|i| !i.roles.is_empty()),
                );
                parsed.push((source, database));
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

    // --- write -----------------------------------------------------------------------------
    let written = match &opts.output {
        Some(out) if output_type == UalOutputType::Csv => {
            write_csv_tables(out, &parsed, &protected).map(Some)
        }
        Some(out) => crate::registry::check_not_read(out, || protected.clone())
            .map_err(io::Error::other)
            .and_then(|()| {
                if let Some(parent) = out.parent().filter(|d| !d.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent)?;
                }
                let file = Box::new(BufWriter::new(File::create(out)?));
                write_records(file, &parsed, output_type)
            })
            .map(|()| None),
        None if data_on_stdout => {
            write_records(Box::new(BufWriter::new(io::stdout())), &parsed, output_type)
                .map(|()| None)
        }
        None => Ok(None),
    };
    let saved = match written {
        Ok(saved) => saved,
        Err(e) => {
            report_error(bar.as_ref(), &format!("Could not write the rows: {e}"));
            return 1;
        }
    };
    if let Some(b) = &bar {
        b.finish_with_message(progress::finished_message(no_color));
    }

    // --- summary ---------------------------------------------------------------------------
    if !data_on_stdout {
        print_summary(&stats, no_color);
        match (&opts.output, saved) {
            (Some(out), Some(files)) => p_label(
                no_color,
                "Results saved: ",
                &format!("{} ({} CSV files)", out.display(), fmt_num(files)),
            ),
            (Some(out), None) => p_label(
                no_color,
                "Results saved: ",
                &format!("{} ({})", out.display(), util::file_size_display(out)),
            ),
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

/// Whether a file name is a UAL database: `Current.mdb`, `SystemIdentity.mdb` or a yearly
/// `{GUID}.mdb`. Other `.mdb` files (Access databases) are not.
pub fn is_database_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower == "current.mdb" || lower == "systemidentity.mdb" || is_yearly_name(&lower)
}

/// `{xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}.mdb`, in lower case.
fn is_yearly_name(lower: &str) -> bool {
    let Some(inner) = lower
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix("}.mdb"))
    else {
        return false;
    };
    let groups: Vec<&str> = inner.split('-').collect();
    groups.len() == 5
        && groups
            .iter()
            .zip([8, 4, 4, 4, 12])
            .all(|(g, n)| g.len() == n && g.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// The UAL databases under `dir`, recursively, in path order.
pub fn find_databases(dir: &Path) -> Vec<PathBuf> {
    util::find_files(dir, is_database_name)
}

/// The `SystemIdentity.mdb` beside a database (the name in any case), the database itself when it
/// is one.
fn identity_database(database: &Path) -> Option<PathBuf> {
    let directory = database.parent()?;
    let directory = if directory.as_os_str().is_empty() {
        Path::new(".")
    } else {
        directory
    };
    directory
        .read_dir()
        .ok()?
        .flatten()
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("SystemIdentity.mdb")
        })
        .map(|e| database.with_file_name(e.file_name()))
        .filter(|p| p.is_file())
}

/// The role names and the years of the yearly databases in a `SystemIdentity.mdb`.
fn read_identity(data: &[u8]) -> Result<Identity, String> {
    let db = Database::open(data).map_err(|e| e.to_string())?;
    let mut identity = Identity {
        problems: db.problems.clone(),
        ..Identity::default()
    };
    if let (Some(table), Ok(mut rows)) = (db.table("ROLE_IDS"), db.rows("ROLE_IDS")) {
        for row in &mut rows {
            if let (Some(Value::Guid(g)), Some(Value::Text(name))) =
                (row.get(table, "RoleGuid"), row.get(table, "RoleName"))
            {
                identity.roles.insert(guid(g), name.clone());
            }
        }
        identity.problems.extend(
            rows.problems()
                .iter()
                .map(|problem| format!("ROLE_IDS: {problem}")),
        );
    }
    if let (Some(table), Ok(mut rows)) =
        (db.table("CHAINED_DATABASES"), db.rows("CHAINED_DATABASES"))
    {
        for row in &mut rows {
            if let (Some(year), Some(Value::Text(file))) = (
                row.get(table, "Year").and_then(Value::as_i64),
                row.get(table, "FileName"),
            ) && let Ok(year) = i32::try_from(year)
            {
                identity.years.insert(file.to_ascii_lowercase(), year);
            }
        }
        identity.problems.extend(
            rows.problems()
                .iter()
                .map(|problem| format!("CHAINED_DATABASES: {problem}")),
        );
    }
    Ok(identity)
}

// --- rows ---------------------------------------------------------------------------------------

/// Every row of every table of a database (not ESE's own `MSys` tables). `name` is its file name,
/// for the year `identity` gives a yearly database.
fn read(data: &[u8], name: &str, identity: Option<&Identity>) -> Result<Parsed, ese::Error> {
    let db = Database::open(data)?;
    let mut problems = db.problems.clone();
    let year = identity.and_then(|i| i.years.get(&name.to_ascii_lowercase()).copied());
    let mut records = Vec::new();
    let mut tables = Vec::new();
    for table in &db.tables {
        if table
            .name
            .get(..4)
            .is_some_and(|p| p.eq_ignore_ascii_case("MSys"))
        {
            continue;
        }
        let key = TableKey::of_table(&table.name);
        let nothing = vec![Value::Null; table.columns.len()];
        let header = columns(table, &key, &nothing, None, None);
        tables.push((
            key.clone(),
            header.into_iter().map(|(name, _)| name).collect(),
        ));
        let mut rows = db.rows(&table.name)?;
        for row in &mut rows {
            records.push(Record {
                table: table.name.clone(),
                key: key.clone(),
                columns: columns(table, &key, &row.values, identity, year),
            });
        }
        problems.extend(
            rows.problems()
                .iter()
                .map(|problem| format!("{}: {problem}", table.name)),
        );
    }
    Ok(Parsed {
        tables,
        records,
        dirty: db.header.state == DatabaseState::DirtyShutdown,
        problems,
    })
}

/// A row's columns, as [`Record::columns`] says. `year` is the year of a yearly database, for
/// the day counters of a client without access times.
fn columns(
    table: &Table,
    key: &TableKey,
    values: &[Value],
    identity: Option<&Identity>,
    year: Option<i32>,
) -> Vec<(String, Json)> {
    let kind = match key {
        TableKey::Known(kind) => Some(*kind),
        TableKey::Other(_) => None,
    };
    let mut out = Vec::with_capacity(table.columns.len());
    let mut days = Vec::new();
    let mut has_days = false;
    let (mut inserted, mut last_access) = (None, None);
    for (column, value) in table.columns.iter().zip(values) {
        let name = column.name.as_str();
        if kind == Some(Kind::Clients)
            && let Some(day) = day_number(name)
        {
            has_days = true;
            if let Some(count) = value.as_i64().and_then(|n| u64::try_from(n).ok())
                && count > 0
            {
                days.push((day, count));
            }
            continue;
        }
        let json = match value {
            // UAL keeps FILETIMEs in its date and time columns; the reader decodes the 8 bytes as
            // an OLE date, whose bits are those bytes.
            Value::DateTime(date) => {
                let filetime = date.to_bits();
                if name.eq_ignore_ascii_case("InsertDate") {
                    inserted = Some(filetime);
                } else if name.eq_ignore_ascii_case("LastAccess") {
                    last_access = Some(filetime);
                }
                crate::registry::filetime(filetime).map_or(Json::Null, |t| json!(t))
            }
            Value::Binary(bytes)
                if matches!(kind, Some(Kind::Clients)) && name.eq_ignore_ascii_case("Address") =>
            {
                address(bytes).map_or_else(|| json_value(value), |a| json!(a))
            }
            Value::U16(minutes)
                if kind == Some(Kind::SystemIdentity)
                    && name.eq_ignore_ascii_case("OSCurrentTimeZone") =>
            {
                // minutes from UTC, stored as 16 bits: 65116 is -420 (UTC-07:00)
                json!(*minutes as i16)
            }
            Value::Text(text)
                if kind == Some(Kind::SystemIdentity)
                    && name.eq_ignore_ascii_case("OSLastBootUpTime") =>
            {
                json!(cim_datetime(text).unwrap_or_else(|| text.clone()))
            }
            _ => json_value(value),
        };
        out.push((name.to_string(), json));
        if matches!(kind, Some(Kind::Clients | Kind::RoleAccess))
            && name.eq_ignore_ascii_case("RoleGuid")
        {
            let role = match value {
                Value::Guid(g) => role_name(&guid(g), identity),
                _ => None,
            };
            out.push((
                "RoleName".to_string(),
                role.map_or(Json::Null, |r| json!(r)),
            ));
        }
        if kind == Some(Kind::SystemIdentity) && name.eq_ignore_ascii_case("OSProductType") {
            let product = value.as_i64().and_then(product_type_name);
            out.push((
                "OSProductTypeName".to_string(),
                product.map_or(Json::Null, |p| json!(p)),
            ));
        }
    }
    if has_days {
        // Prefer a valid access timestamp over database metadata. A present but unusable
        // LastAccess must not hide a valid InsertDate.
        let year = last_access
            .and_then(filetime_year)
            .or_else(|| inserted.and_then(filetime_year))
            .or(year);
        out.push(("DailyAccesses".to_string(), daily_accesses(&days, year)));
    }
    out
}

/// The day of the year of a client's day counter column (`Day1` … `Day366`).
fn day_number(column: &str) -> Option<u16> {
    let digits = column
        .get(..3)
        .filter(|p| p.eq_ignore_ascii_case("Day"))
        .map(|_| &column[3..])?;
    if digits.is_empty() || digits.starts_with('0') || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok().filter(|day| (1..=366).contains(day))
}

/// The UTC year of a FILETIME.
fn filetime_year(filetime: u64) -> Option<i32> {
    crate::registry::filetime(filetime)
        .and_then(|t| DateTime::parse_from_rfc3339(&t).ok())
        .map(|t| t.year())
}

/// The days a client used the role and how many times: `{"Date": "2023-01-12", "Accesses": 5}`
/// for each day with accesses, `DayOfYear` for a day whose year is unknown.
fn daily_accesses(days: &[(u16, u64)], year: Option<i32>) -> Json {
    Json::Array(
        days.iter()
            .map(|&(day, count)| {
                match year.and_then(|y| NaiveDate::from_yo_opt(y, u32::from(day))) {
                    Some(date) => {
                        json!({"Date": date.format("%Y-%m-%d").to_string(), "Accesses": count})
                    }
                    None => json!({"DayOfYear": day, "Accesses": count}),
                }
            })
            .collect(),
    )
}

/// `DailyAccesses` in a CSV cell: `2023-01-12: 5, 2023-01-13: 2`.
fn daily_accesses_cell(value: &Json) -> String {
    let Some(days) = value.as_array() else {
        return cell(value);
    };
    days.iter()
        .map(|day| {
            let accesses = cell(&day["Accesses"]);
            match day.get("Date") {
                Some(date) => format!("{}: {accesses}", cell(date)),
                None => format!("day {}: {accesses}", cell(&day["DayOfYear"])),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// A client's address: 4 bytes of IPv4, 16 of IPv6.
fn address(bytes: &[u8]) -> Option<String> {
    match bytes.len() {
        4 => Some(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]).to_string()),
        16 => {
            let mut octets = [0u8; 16];
            octets.copy_from_slice(bytes);
            Some(Ipv6Addr::from(octets).to_string())
        }
        _ => None,
    }
}

/// The name of a role: from `SystemIdentity.mdb`, else the name Windows Server gives it.
fn role_name(role: &str, identity: Option<&Identity>) -> Option<String> {
    identity
        .and_then(|i| i.roles.get(role).cloned())
        .or_else(|| {
            ROLE_NAMES
                .iter()
                .find(|(id, _)| id.eq_ignore_ascii_case(role))
                .map(|(_, name)| (*name).to_string())
        })
}

/// What `OSProductType` (as in Win32_OperatingSystem) stands for.
fn product_type_name(product: i64) -> Option<&'static str> {
    match product {
        1 => Some("Workstation"),
        2 => Some("Domain Controller"),
        3 => Some("Server"),
        _ => None,
    }
}

/// A WMI (CIM) date and time, `yyyymmddHHMMSS.mmmmmm` local time and `+UUU`/`-UUU` minutes from
/// UTC (`20200917100320.485021-420`), in UTC (`2020-09-17T17:03:20.485021Z`); `None` for any
/// other text.
fn cim_datetime(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    if bytes.len() != 25
        || bytes[14] != b'.'
        || !matches!(bytes[21], b'+' | b'-')
        || !bytes[..14]
            .iter()
            .chain(&bytes[15..21])
            .chain(&bytes[22..])
            .all(u8::is_ascii_digit)
    {
        return None;
    }
    let local = NaiveDateTime::parse_from_str(&text[..21], "%Y%m%d%H%M%S%.f").ok()?;
    let minutes: i64 = text[22..].parse().ok()?;
    let offset = if bytes[21] == b'-' { -minutes } else { minutes };
    let utc = local.checked_sub_signed(TimeDelta::minutes(offset))?;
    Some(utc.and_utc().to_rfc3339_opts(SecondsFormat::Micros, true))
}

// --- writers ------------------------------------------------------------------------------------

/// The CSV file name of each table, `UAL-Clients.csv` …; other tables under their own name, kept
/// in the directory and apart from the known names.
fn csv_file_names<'a>(keys: impl Iterator<Item = &'a TableKey>) -> BTreeMap<TableKey, String> {
    let mut used: HashSet<String> = Kind::ALL
        .iter()
        .map(|k| k.output_name().to_ascii_lowercase())
        .collect();
    let mut names = BTreeMap::new();
    for key in keys {
        if names.contains_key(key) {
            continue;
        }
        let name = match key {
            TableKey::Known(kind) => kind.output_name().to_string(),
            TableKey::Other(table) => safe_file_name(table, &mut used),
        };
        names.insert(key.clone(), format!("UAL-{name}.csv"));
    }
    names
}

/// One CSV file per table in `dir`, with the rows of every database (`SourceFile` first), and
/// a header for a table without rows; returns the number of files written.
fn write_csv_tables(
    dir: &Path,
    parsed: &[(String, Parsed)],
    protected: &[PathBuf],
) -> io::Result<usize> {
    let names = csv_file_names(
        parsed
            .iter()
            .flat_map(|(_, p)| p.tables.iter().map(|(key, _)| key)),
    );
    let mut tables: BTreeMap<&TableKey, CsvTable> = BTreeMap::new();
    for (key, header) in parsed.iter().flat_map(|(_, p)| &p.tables) {
        let columns = std::iter::once("SourceFile".to_string()).chain(header.iter().cloned());
        tables.entry(key).or_default().add_columns(columns);
    }
    let records = parsed
        .iter()
        .flat_map(|(source, p)| p.records.iter().map(move |r| (source, r)));
    for (source, record) in records {
        let mut row = vec![("SourceFile".to_string(), json!(source))];
        row.extend(record.columns.iter().map(|(name, value)| {
            if name == "DailyAccesses" {
                (name.clone(), json!(daily_accesses_cell(value)))
            } else {
                (name.clone(), value.clone())
            }
        }));
        tables.entry(&record.key).or_default().push(row);
    }
    std::fs::create_dir_all(dir)?;
    for name in names.values() {
        crate::registry::check_not_read(&dir.join(name), || protected.to_vec())
            .map_err(io::Error::other)?;
    }
    for (key, table) in &tables {
        table.write(&dir.join(&names[*key]))?;
    }
    Ok(tables.len())
}

/// Every row as JSON (pretty) or JSONL: where it comes from, then its columns.
fn write_records(
    mut out: Box<dyn Write>,
    parsed: &[(String, Parsed)],
    output_type: UalOutputType,
) -> io::Result<()> {
    let pretty = output_type == UalOutputType::Json;
    for (source, database) in parsed {
        for record in &database.records {
            let mut columns = vec![
                ("SourceFile".to_string(), json!(source)),
                ("Table".to_string(), json!(record.table)),
            ];
            columns.extend(record.columns.iter().cloned());
            crate::output_schema::write_json(&mut out, &Ordered(&columns), pretty)?;
            out.write_all(b"\n")?;
        }
    }
    out.flush()
}

// --- summary ------------------------------------------------------------------------------------

/// Counts for the results summary.
#[derive(Default)]
struct Stats {
    databases: usize,
    failed: usize,
    dirty: usize,
    problems: usize,
    with_identity: usize,
    without_identity: usize,
    rows: BTreeMap<TableKey, usize>,
    first: Option<DateTime<chrono::FixedOffset>>,
    last: Option<DateTime<chrono::FixedOffset>>,
    addresses: HashSet<String>,
    accounts: HashSet<String>,
    /// Client accesses per role name (the GUID of a role without one).
    roles: HashMap<String, u64>,
}

impl Stats {
    fn add(&mut self, database: &Parsed, identity: bool) {
        self.databases += 1;
        self.dirty += usize::from(database.dirty);
        self.problems += database.problems.len();
        if identity {
            self.with_identity += 1;
        } else {
            self.without_identity += 1;
        }
        for record in &database.records {
            *self.rows.entry(record.key.clone()).or_default() += 1;
            if record.key != TableKey::Known(Kind::Clients) {
                continue;
            }
            let time = |name| {
                record
                    .get(name)
                    .and_then(Json::as_str)
                    .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
            };
            if let Some(t) = time("InsertDate") {
                self.first = Some(self.first.map_or(t, |f| f.min(t)));
            }
            if let Some(t) = time("LastAccess") {
                self.last = Some(self.last.map_or(t, |l| l.max(t)));
            }
            if let Some(a) = record.get("Address").and_then(Json::as_str) {
                self.addresses.insert(a.to_string());
            }
            if let Some(a) = record.get("AuthenticatedUserName").and_then(Json::as_str) {
                self.accounts.insert(a.to_ascii_lowercase());
            }
            let role = record
                .get("RoleName")
                .and_then(Json::as_str)
                .or_else(|| record.get("RoleGuid").and_then(Json::as_str))
                .unwrap_or("(no role)");
            let accesses = record
                .get("TotalAccesses")
                .and_then(Json::as_u64)
                .unwrap_or(0);
            let total = self.roles.entry(role.to_string()).or_default();
            *total = total.saturating_add(accesses);
        }
    }
}

fn print_summary(s: &Stats, no_color: bool) {
    p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
    let mut parsed = fmt_num(s.databases);
    if s.failed > 0 {
        parsed.push_str(&format!(" (could not parse: {})", fmt_num(s.failed)));
    }
    p_label(no_color, "UAL databases parsed: ", &parsed);
    if s.dirty > 0 {
        p_label(
            no_color,
            "Left dirty (changes still only in the transaction logs are missing): ",
            &fmt_num(s.dirty),
        );
    }
    let total: usize = s.rows.values().sum();
    p_label(no_color, "Rows: ", &fmt_num(total));
    for (table, n) in &s.rows {
        p(
            None,
            &format!("  {}: {}", table.output_name(), fmt_num(*n)),
            true,
        );
    }
    if let (Some(first), Some(last)) = (s.first, s.last) {
        let utc = |t: DateTime<chrono::FixedOffset>| {
            t.with_timezone(&Utc)
                .to_rfc3339_opts(SecondsFormat::Secs, true)
        };
        p_label(
            no_color,
            "Client access: ",
            &format!("{} - {}", utc(first), utc(last)),
        );
    }
    p_label(
        no_color,
        "Client addresses / accounts: ",
        &format!(
            "{} / {}",
            fmt_num(s.addresses.len()),
            fmt_num(s.accounts.len())
        ),
    );
    let mut roles: Vec<(&String, &u64)> = s.roles.iter().collect();
    roles.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    if !roles.is_empty() {
        p(
            JunteiColor::Green.rdg(no_color),
            "Client accesses by role (sum of source rows; not deduplicated):",
            true,
        );
        for (role, accesses) in roles {
            p(
                None,
                &format!(
                    "  {:>10}  {role}",
                    fmt_num(usize::try_from(*accesses).unwrap_or(usize::MAX))
                ),
                true,
            );
        }
    }
    let names = match (s.with_identity, s.without_identity) {
        (_, 0) => "from SystemIdentity.mdb where available; otherwise built in".to_string(),
        (0, _) => "built in (no readable SystemIdentity.mdb role names)".to_string(),
        (_, n) => format!(
            "from SystemIdentity.mdb where available ({} databases without readable role names: built in)",
            fmt_num(n)
        ),
    };
    p_label(no_color, "Role names: ", &names);
    p_label(no_color, "Problems: ", &fmt_num(s.problems));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_names() {
        assert!(is_database_name("Current.mdb"));
        assert!(is_database_name("current.MDB"));
        assert!(is_database_name("SystemIdentity.mdb"));
        assert!(is_database_name(
            "{32D77B8F-140E-4668-8289-F964D6E7C2F3}.mdb"
        ));
        assert!(is_database_name(
            "{32d77b8f-140e-4668-8289-f964d6e7c2f3}.mdb"
        ));
        assert!(!is_database_name("Current.jfm"));
        assert!(!is_database_name("Svc.log"));
        assert!(!is_database_name("customers.mdb"));
        assert!(!is_database_name(
            "{32D77B8F-140E-4668-8289-F964D6E7C2F}.mdb"
        ));
        assert!(!is_database_name(
            "{32D77B8F-140E-4668-8289-F964D6E7C2FG}.mdb"
        ));
        assert!(!is_database_name(
            "32D77B8F-140E-4668-8289-F964D6E7C2F3.mdb"
        ));
    }

    #[test]
    fn day_counter_columns() {
        assert_eq!(day_number("Day1"), Some(1));
        assert_eq!(day_number("Day366"), Some(366));
        assert_eq!(day_number("DAY12"), Some(12));
        assert_eq!(day_number("Day0"), None);
        assert_eq!(day_number("Day367"), None);
        assert_eq!(day_number("Day01"), None);
        assert_eq!(day_number("Day"), None);
        assert_eq!(day_number("Days"), None);
        assert_eq!(day_number("Day+1"), None);
        assert_eq!(day_number("Da"), None);
    }

    #[test]
    fn daily_accesses_are_dated_in_the_year() {
        let days = [(1, 3), (60, 1), (366, 2)];
        // 2024 is a leap year: day 60 is February 29, day 366 December 31
        assert_eq!(
            daily_accesses(&days, Some(2024)),
            json!([
                {"Date": "2024-01-01", "Accesses": 3},
                {"Date": "2024-02-29", "Accesses": 1},
                {"Date": "2024-12-31", "Accesses": 2}
            ])
        );
        // 2023 has no day 366
        let year_2023 = daily_accesses(&days, Some(2023));
        assert_eq!(year_2023[1], json!({"Date": "2023-03-01", "Accesses": 1}));
        assert_eq!(year_2023[2], json!({"DayOfYear": 366, "Accesses": 2}));
        assert_eq!(
            daily_accesses_cell(&year_2023),
            "2023-01-01: 3, 2023-03-01: 1, day 366: 2"
        );
        let unknown = daily_accesses(&days[..1], None);
        assert_eq!(unknown, json!([{"DayOfYear": 1, "Accesses": 3}]));
        assert_eq!(daily_accesses_cell(&daily_accesses(&[], Some(2023))), "");
    }

    #[test]
    fn client_addresses() {
        assert_eq!(address(&[192, 168, 1, 20]).as_deref(), Some("192.168.1.20"));
        let mut v6 = [0u8; 16];
        v6[0] = 0xfe;
        v6[1] = 0x80;
        v6[15] = 1;
        assert_eq!(address(&v6).as_deref(), Some("fe80::1"));
        assert_eq!(address(&[1, 2, 3]), None);
    }

    #[test]
    fn wmi_times_in_utc() {
        assert_eq!(
            cim_datetime("20200917100320.485021-420").as_deref(),
            Some("2020-09-17T17:03:20.485021Z")
        );
        assert_eq!(
            cim_datetime("20230111035411.500000+000").as_deref(),
            Some("2023-01-11T03:54:11.500000Z")
        );
        assert_eq!(
            cim_datetime("20220716164333.452772+120").as_deref(),
            Some("2022-07-16T14:43:33.452772Z")
        );
        assert_eq!(cim_datetime("2020091710032.485021-420"), None);
        assert_eq!(cim_datetime("20200917100320.485021*420"), None);
        assert_eq!(cim_datetime("2020091710032*.485021-420"), None);
        assert_eq!(cim_datetime("20201317100320.485021-420"), None);
        assert_eq!(cim_datetime("20200917100320.485021-42a"), None);
        assert_eq!(cim_datetime(""), None);
    }

    #[test]
    fn role_names_from_the_server_first() {
        let file_server = "{10A9226F-50EE-49D8-A393-9A501D47CE04}";
        assert_eq!(role_name(file_server, None).as_deref(), Some("File Server"));
        assert_eq!(
            role_name(&file_server.to_ascii_lowercase(), None).as_deref(),
            Some("File Server")
        );
        let identity = Identity {
            roles: HashMap::from([(file_server.to_string(), "Files".to_string())]),
            years: HashMap::new(),
            problems: Vec::new(),
        };
        assert_eq!(
            role_name(file_server, Some(&identity)).as_deref(),
            Some("Files")
        );
        assert_eq!(
            role_name("{00000000-0000-0000-0000-000000000000}", Some(&identity)),
            None
        );
        assert_eq!(product_type_name(2), Some("Domain Controller"));
        assert_eq!(product_type_name(9), None);
    }

    /// A table of `columns` (name and type), in that order.
    fn table(name: &str, columns: &[(&str, ese::ColumnType)]) -> Table {
        Table {
            object_id: 2,
            name: name.to_string(),
            root_page: 4,
            flags: 0,
            template: None,
            columns: columns
                .iter()
                .zip(1u32..)
                .map(|(&(name, column_type), id)| ese::Column {
                    id,
                    name: name.to_string(),
                    column_type,
                    codepage: 1200,
                    flags: 0,
                    max_size: 0,
                    record_offset: 0,
                    default: None,
                    from_template: false,
                })
                .collect(),
            indexes: Vec::new(),
            long_value_root: None,
        }
    }

    /// A FILETIME as the reader gives it: its 8 bytes as an OLE date.
    fn date(filetime: u64) -> Value {
        Value::DateTime(f64::from_bits(filetime))
    }

    #[test]
    fn a_client_row() {
        use ese::ColumnType::{Binary, DateTime, Guid, LongText, UnsignedLong, UnsignedShort};
        let mut columns = vec![
            ("RoleGuid", Guid),
            ("TenantId", Guid),
            ("TotalAccesses", UnsignedLong),
            ("InsertDate", DateTime),
            ("LastAccess", DateTime),
        ];
        let days: Vec<String> = (1..=366).map(|d| format!("Day{d}")).collect();
        columns.extend(days.iter().map(|d| (d.as_str(), UnsignedShort)));
        columns.extend([
            ("Address", Binary),
            ("AuthenticatedUserName", LongText),
            ("ClientName", LongText),
        ]);
        let clients = table("CLIENTS", &columns);
        // the File Server role, as stored: the first three fields little-endian
        let file_server = [
            0x6f, 0x22, 0xa9, 0x10, 0xee, 0x50, 0xd8, 0x49, 0xa3, 0x93, 0x9a, 0x50, 0x1d, 0x47,
            0xce, 0x04,
        ];
        let mut values = vec![
            Value::Guid(file_server),
            Value::Guid([0; 16]),
            Value::U32(7),
            // 2024-02-28T23:59:59.9999999Z and 2024-12-31T12:00:00Z
            date(133_536_383_999_999_999),
            date(133_801_200_000_000_000),
        ];
        values.extend((1..=366).map(|d| match d {
            59 => Value::U16(4),
            60 => Value::U16(2),
            366 => Value::U16(1),
            1 => Value::U16(0),
            _ => Value::Null,
        }));
        values.extend([
            Value::Binary(vec![10, 0, 0, 5]),
            Value::Text(r"CORPlice".to_string()),
            Value::Null,
        ]);
        let row = columns_of(&clients, &values, None, None);
        assert_eq!(
            row.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
            [
                "RoleGuid",
                "RoleName",
                "TenantId",
                "TotalAccesses",
                "InsertDate",
                "LastAccess",
                "Address",
                "AuthenticatedUserName",
                "ClientName",
                "DailyAccesses"
            ]
        );
        let value = |name: &str| row.iter().find(|(n, _)| n == name).unwrap().1.clone();
        assert_eq!(value("RoleGuid"), "{10A9226F-50EE-49D8-A393-9A501D47CE04}");
        assert_eq!(value("RoleName"), "File Server");
        assert_eq!(value("TenantId"), "{00000000-0000-0000-0000-000000000000}");
        assert_eq!(value("InsertDate"), "2024-02-28T23:59:59.999999900Z");
        assert_eq!(value("LastAccess"), "2024-12-31T12:00:00Z");
        assert_eq!(value("Address"), "10.0.0.5");
        assert_eq!(value("ClientName"), Json::Null);
        // the year of the last access, a leap year: day 60 is February 29
        assert_eq!(
            value("DailyAccesses"),
            json!([
                {"Date": "2024-02-28", "Accesses": 4},
                {"Date": "2024-02-29", "Accesses": 2},
                {"Date": "2024-12-31", "Accesses": 1}
            ])
        );
        // without access times, the year of the yearly database
        let mut undated = values.clone();
        undated[3] = date(0);
        undated[4] = Value::Null;
        let row = columns_of(&clients, &undated, None, Some(2023));
        let daily = &row.last().unwrap().1;
        assert_eq!(daily[0], json!({"Date": "2023-02-28", "Accesses": 4}));
        assert_eq!(daily[2], json!({"DayOfYear": 366, "Accesses": 1}));
        assert_eq!(row[4].1, Json::Null);
        // an address of another size stays in hex
        let mut odd = values.clone();
        let at = values.len() - 3;
        odd[at] = Value::Binary(vec![1, 2, 3]);
        let row = columns_of(&clients, &odd, None, None);
        assert_eq!(
            row.iter().find(|(n, _)| n == "Address").unwrap().1,
            "010203"
        );
    }

    #[test]
    fn daily_accesses_use_insert_date_when_last_access_is_unusable() {
        use ese::ColumnType::{DateTime, UnsignedShort};
        let clients = table(
            "CLIENTS",
            &[
                ("InsertDate", DateTime),
                ("LastAccess", DateTime),
                ("Day60", UnsignedShort),
            ],
        );
        for last in [date(0), date(u64::MAX), Value::Null] {
            let values = [
                date(133_536_383_999_999_999), // 2024-02-28 UTC
                last,
                Value::U16(2),
            ];
            // Even without SystemIdentity.mdb, the valid InsertDate establishes the year.
            for database_year in [None, Some(2023)] {
                let row = columns_of(&clients, &values, None, database_year);
                assert_eq!(row[1].1, Json::Null);
                assert_eq!(
                    row.last().unwrap().1,
                    json!([{"Date": "2024-02-29", "Accesses": 2}]),
                    "LastAccess {:?}, database year {database_year:?}",
                    values[1]
                );
            }
        }
    }

    #[test]
    fn identity_and_other_tables() {
        use ese::ColumnType::{Bit, DateTime, Guid, LongText, UnsignedByte, UnsignedShort};
        let identity = table(
            "SYSTEM_IDENTITY",
            &[
                ("CreationTime", DateTime),
                ("OSProductType", UnsignedByte),
                ("OSCurrentTimeZone", UnsignedShort),
                ("OSDaylightInEffect", Bit),
                ("OSLastBootUpTime", LongText),
            ],
        );
        let row = columns_of(
            &identity,
            &[
                date(132_448_316_944_240_000),
                Value::U8(2),
                Value::U16(65116),
                Value::Bool(true),
                Value::Text("20200917100320.485021-420".to_string()),
            ],
            None,
            None,
        );
        assert_eq!(
            row,
            [
                (
                    "CreationTime".to_string(),
                    json!("2020-09-17T15:54:54.424Z")
                ),
                ("OSProductType".to_string(), json!(2)),
                ("OSProductTypeName".to_string(), json!("Domain Controller")),
                ("OSCurrentTimeZone".to_string(), json!(-420)),
                ("OSDaylightInEffect".to_string(), json!(true)),
                (
                    "OSLastBootUpTime".to_string(),
                    json!("2020-09-17T17:03:20.485021Z")
                ),
            ]
        );
        // a boot time WMI couldn't write stays as it is
        let row = columns_of(
            &identity,
            &[
                Value::Null,
                Value::U8(9),
                Value::U16(0),
                Value::Null,
                Value::Text("unknown".to_string()),
            ],
            None,
            None,
        );
        assert_eq!(row[2].1, Json::Null);
        assert_eq!(row[5].1, "unknown");

        // a role named by the server's SystemIdentity.mdb; DNS addresses are text
        let access = table(
            "ROLE_ACCESS",
            &[("RoleGuid", Guid), ("FirstSeen", DateTime)],
        );
        let server = Identity {
            roles: HashMap::from([(
                "{00000000-0000-0000-0000-000000000001}".to_string(),
                "Custom Role".to_string(),
            )]),
            years: HashMap::new(),
            problems: Vec::new(),
        };
        let mut role = [0u8; 16];
        role[15] = 1;
        let row = columns_of(&access, &[Value::Guid(role), date(0)], Some(&server), None);
        assert_eq!(row[1], ("RoleName".to_string(), json!("Custom Role")));
        assert_eq!(row[2], ("FirstSeen".to_string(), Json::Null));
        let dns = table("DNS", &[("Address", LongText), ("HostName", LongText)]);
        let row = columns_of(
            &dns,
            &[
                Value::Text("10.0.0.5".to_string()),
                Value::Text("host.corp.example".to_string()),
            ],
            None,
            None,
        );
        assert_eq!(row[0].1, "10.0.0.5");
        // another table: every column as it is, nothing added
        let other = table("NEW_TABLE", &[("RoleGuid", Guid), ("Day1", UnsignedShort)]);
        let row = columns_of(&other, &[Value::Guid(role), Value::U16(3)], None, None);
        assert_eq!(
            row,
            [
                (
                    "RoleGuid".to_string(),
                    json!("{00000000-0000-0000-0000-000000000001}")
                ),
                ("Day1".to_string(), json!(3)),
            ]
        );
    }

    /// The columns of a row of `table`, as written.
    fn columns_of(
        table: &Table,
        values: &[Value],
        identity: Option<&Identity>,
        year: Option<i32>,
    ) -> Vec<(String, Json)> {
        columns(
            table,
            &TableKey::of_table(&table.name),
            values,
            identity,
            year,
        )
    }

    #[test]
    fn table_files_stay_in_the_directory_and_apart() {
        let keys = [
            TableKey::Other("../../evil".to_string()),
            TableKey::Known(Kind::Clients),
            TableKey::Other("Clients ".to_string()),
            TableKey::Known(Kind::Dns),
        ];
        let names = csv_file_names(keys.iter());
        assert_eq!(names[&TableKey::Known(Kind::Clients)], "UAL-Clients.csv");
        assert_eq!(names[&TableKey::Known(Kind::Dns)], "UAL-DNS.csv");
        assert_eq!(
            names[&TableKey::Other("../../evil".to_string())],
            "UAL-______evil.csv"
        );
        assert_eq!(
            names[&TableKey::Other("Clients ".to_string())],
            "UAL-Clients_.csv"
        );
        assert_eq!(Kind::of_table("clients"), Some(Kind::Clients));
        assert_eq!(Kind::of_table("ROLE_IDS"), Some(Kind::Roles));
        assert_eq!(Kind::of_table("SruDbIdMapTable"), None);
    }
}
