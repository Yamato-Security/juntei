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

//! `parse-ese windows-search`: the Windows Search index, `Windows.edb`.
//!
//! Windows Search keeps what it indexed in one ESE database: the gatherer's record of every
//! crawled item (`SystemIndex_Gthr`, with its scope tree in `SystemIndex_GthrPth`) and the
//! property store with the item's indexed properties (`SystemIndex_PropertyStore` from Windows 8
//! on, `SystemIndex_0A` on Windows 7). One record is written per indexed item, joining the two by
//! the document identifier. Windows 11 22H2 moved the index to SQLite (`Windows.db`); those files
//! are recognized and reported, not read.
//!
//! The times are the ones Windows Search stored: the item's file system times as they were when
//! it was indexed, and the moment the gatherer processed it. None of them says that a user opened
//! the item, and an index can be stale or filtered by the indexing settings.

use crate::cli::{WindowsSearchOptions, WindowsSearchOutputType};
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use ese::{Column, ColumnType, Database, DatabaseState, Value};
use indicatif::ProgressBar;
use serde_json::{Map, Value as Json, json};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// Larger databases are refused: Windows Search indexes of many gigabytes exist, but the file is
/// mapped whole and a corrupt size field must not become an unbounded read.
const MAX_FILE_SIZE: u64 = 32 * 1024 * 1024 * 1024;
/// The most one row's long values (indexed text, summaries) may decompress to.
const ROW_LONG_VALUE_LIMIT: usize = 64 * 1024 * 1024;
/// The deepest scope tree that is followed when a gatherer path is rebuilt.
const MAX_SCOPE_DEPTH: usize = 128;
const MAX_PRINTED_ERRORS: usize = 10;
const ESE_SIGNATURE: [u8; 4] = [0xEF, 0xCD, 0xAB, 0x89];
const SQLITE_SIGNATURE: &[u8] = b"SQLite format 3\0";

/// The ESE index and the SQLite databases of Windows 11 22H2 and later.
/// SQLite files are found so that they can be reported as unsupported.
pub fn is_database_name(name: &str) -> bool {
    [
        "Windows.edb",
        "Windows.db",
        "Windows-gather.db",
        "Windows-usn.db",
    ]
    .iter()
    .any(|n| n.eq_ignore_ascii_case(name))
}

/// What disk-image staging copies for this parser: the databases and the ESE companions written
/// beside `Windows.edb` (`Windows.jfm`, `MSS*.log`, `MSS.chk`, `MSS*.jrs`), so that a staged
/// folder shows the same files as the source.
pub fn is_staged_name(name: &str) -> bool {
    if is_database_name(name) || name.eq_ignore_ascii_case("Windows.jfm") {
        return true;
    }
    let lower = name.to_ascii_lowercase();
    lower.starts_with("mss")
        && [".log", ".chk", ".jrs"]
            .iter()
            .any(|suffix| lower.ends_with(suffix))
}

/// The candidate files under `dir`, in path order.
fn find_databases(dir: &Path) -> Vec<Source> {
    util::find_files(dir, is_database_name)
        .into_iter()
        .map(|path| Source::under(dir, path))
        .collect()
}

/// One database to read.
struct Source {
    path: PathBuf,
    /// `SourceFile`: the path below the input directory with `/` between its names, or the name
    /// of a file given with `-f`. Never an absolute path.
    shown: String,
}

impl Source {
    fn under(root: &Path, path: PathBuf) -> Self {
        let shown =
            crate::powershell_history::parts(path.strip_prefix(root).unwrap_or(&path)).join("/");
        Self { path, shown }
    }

    fn given(path: PathBuf) -> Self {
        let shown = crate::powershell_history::parts(&path)
            .last()
            .cloned()
            .unwrap_or_default();
        Self { path, shown }
    }
}

// --- reading ------------------------------------------------------------------------------------

/// Why a file produced no records.
enum Skip {
    /// The SQLite index of Windows 11 22H2 and later.
    Sqlite,
    /// Not an ESE database, or not a Windows Search one.
    NotIndex(String),
    Unreadable(String),
}

/// The file's bytes: mapped when the operating system allows it, read otherwise.
enum Mapped {
    Map(memmap2::Mmap),
    Read(Vec<u8>),
}

impl Mapped {
    fn bytes(&self) -> &[u8] {
        match self {
            Mapped::Map(map) => map,
            Mapped::Read(data) => data,
        }
    }
}

fn open(path: &Path) -> Result<Mapped, Skip> {
    let file = File::open(path).map_err(|e| Skip::Unreadable(e.to_string()))?;
    let size = file
        .metadata()
        .map_err(|e| Skip::Unreadable(e.to_string()))?
        .len();
    if size == 0 {
        return Err(Skip::NotIndex("the file is empty".into()));
    }
    if size > MAX_FILE_SIZE {
        return Err(Skip::Unreadable(format!(
            "the file is {} and the limit is {}",
            util::size_display(size),
            util::size_display(MAX_FILE_SIZE)
        )));
    }
    let length = usize::try_from(size)
        .map_err(|_| Skip::Unreadable("the file is too large for this platform".into()))?;
    // SAFETY: this parser operates on immutable evidence copies. The backing file must not be
    // modified or truncated while mapped; a read-only mapping does not protect against another
    // process truncating it. Fix the mapping length to the size checked above.
    match unsafe { memmap2::MmapOptions::new().len(length).map(&file) } {
        Ok(map) => Ok(Mapped::Map(map)),
        // Read the same open handle and enforce the limit again: the path may have been
        // replaced or the file may have grown since metadata was read.
        Err(_) => read_bounded(file, MAX_FILE_SIZE).map(Mapped::Read),
    }
}

fn read_bounded(reader: impl Read, limit: u64) -> Result<Vec<u8>, Skip> {
    let mut bytes = Vec::new();
    reader
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|e| Skip::Unreadable(e.to_string()))?;
    if bytes.len() as u64 > limit {
        return Err(Skip::Unreadable(
            "the file grew beyond the size limit while being read".into(),
        ));
    }
    Ok(bytes)
}

/// Which property store the database has. The names of its date columns and the byte order of
/// their values differ between the two.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Schema {
    /// `SystemIndex_0A`, keyed by `DocID`, big-endian times (Windows 7).
    Windows7,
    /// `SystemIndex_PropertyStore`, keyed by `WorkID`, columns prefixed with the property ID,
    /// little-endian times (Windows 8 through Windows 11 21H2).
    PropertyStore,
}

impl Schema {
    fn table(self) -> &'static str {
        match self {
            Schema::Windows7 => "SystemIndex_0A",
            Schema::PropertyStore => "SystemIndex_PropertyStore",
        }
    }
    fn key(self) -> &'static str {
        match self {
            Schema::Windows7 => "DocID",
            Schema::PropertyStore => "WorkID",
        }
    }
    fn big_endian(self) -> bool {
        self == Schema::Windows7
    }
    fn name(self) -> &'static str {
        match self {
            Schema::Windows7 => "Windows 7 (SystemIndex_0A)",
            Schema::PropertyStore => "Windows 8 to 11 (SystemIndex_PropertyStore)",
        }
    }
}

/// One `SystemIndex_Gthr` row: the gatherer's record of an item.
#[derive(Default)]
struct Gather {
    scope_id: Option<i64>,
    sdid: Option<i64>,
    /// `LastModified`: the item's last-modified time when the gatherer saw it (FILETIME ticks).
    last_modified: Option<u64>,
    file_name: Option<String>,
    transaction_flags: Option<i64>,
    crawl_number: Option<i64>,
    priority: Option<i64>,
    deleted_count: Option<i64>,
    /// Every column of the row, for the JSON `Gather` object.
    columns: Vec<(String, Json)>,
}

/// One indexed property of an item.
struct Property {
    /// The property ID that prefixes the column name from Windows 8 on.
    id: Option<u32>,
    /// The property name as the schema spells it, e.g. `System_ItemPathDisplay`.
    name: String,
    column_type: ColumnType,
    value: Json,
    /// For a stored FILETIME: its RFC 3339 spelling (the `value` holds the ticks).
    time: Option<String>,
}

/// One indexed item: its gatherer row, its property-store row, or both.
struct Item {
    document_id: i64,
    /// Table membership is independent of whether any non-key property is non-NULL.
    in_property_store: bool,
    gather: Option<Gather>,
    properties: Vec<Property>,
    problems: Vec<String>,
}

impl Item {
    fn property(&self, name: &str) -> Option<&Property> {
        self.properties
            .iter()
            .find(|property| property.name.eq_ignore_ascii_case(name))
    }

    fn text(&self, name: &str) -> Option<String> {
        self.property(name)
            .and_then(|property| match &property.value {
                Json::String(s) => Some(s.clone()),
                Json::Array(items) => Some(
                    items
                        .iter()
                        .map(|item| match item {
                            Json::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join("|"),
                ),
                Json::Null => None,
                other => Some(other.to_string()),
            })
    }

    fn time(&self, name: &str) -> Option<String> {
        self.property(name)
            .and_then(|property| property.time.clone())
    }

    fn json(&self, name: &str) -> Json {
        self.property(name)
            .map_or(Json::Null, |property| property.value.clone())
    }

    fn in_tables(&self) -> &'static str {
        match (&self.gather, self.in_property_store) {
            (Some(_), true) => "Gthr+PropertyStore",
            (Some(_), false) => "Gthr",
            (None, _) => "PropertyStore",
        }
    }
}

/// The scope tree of `SystemIndex_GthrPth`: a scope's parent and its own name.
#[derive(Default)]
struct Scopes(HashMap<i64, (i64, String)>);

impl Scopes {
    /// The names from the root down to `scope`, concatenated as the gatherer spells them
    /// (`file:C:/Users/…/`), or nothing for an unknown scope.
    fn path(&self, scope: i64) -> Option<String> {
        let mut names = Vec::new();
        let mut seen = HashSet::new();
        let mut current = scope;
        while let Some((parent, name)) = self.0.get(&current) {
            if !seen.insert(current) || names.len() >= MAX_SCOPE_DEPTH {
                return None;
            }
            names.push(name.as_str());
            if *parent == current || *parent <= 0 {
                break;
            }
            current = *parent;
        }
        if names.is_empty() {
            return None;
        }
        names.reverse();
        Some(names.concat())
    }
}

/// What one database held.
#[derive(Default)]
struct Parsed {
    schema: Option<Schema>,
    dirty: bool,
    problems: Vec<String>,
}

/// Counts for the summary.
#[derive(Default)]
struct Stats {
    databases: usize,
    windows7: usize,
    property_store: usize,
    sqlite: usize,
    not_index: usize,
    unreadable: usize,
    dirty: usize,
    items: usize,
    both: usize,
    gather_only: usize,
    property_only: usize,
    deleted: usize,
    folders: usize,
    untimed: usize,
    with_problems: usize,
    stores: BTreeMap<String, usize>,
    problems: usize,
}

/// Reads one database and writes its items. The property store is streamed row by row: the
/// gatherer tables, which are small, are kept in memory to join against.
fn parse(
    source: &Source,
    records: &mut Option<Records>,
    stats: &mut Stats,
) -> Result<Parsed, Skip> {
    let mapped = open(&source.path)?;
    let data = mapped.bytes();
    if data.starts_with(SQLITE_SIGNATURE) {
        return Err(Skip::Sqlite);
    }
    if data.len() < 8 || data[4..8] != ESE_SIGNATURE {
        return Err(Skip::NotIndex(
            "not an ESE database (no ESE signature at offset 4)".into(),
        ));
    }
    let db = Database::open(data).map_err(|e| Skip::NotIndex(format!("{e:?}")))?;
    let schema = [Schema::PropertyStore, Schema::Windows7]
        .into_iter()
        .find(|schema| db.table(schema.table()).is_some())
        .ok_or_else(|| {
            Skip::NotIndex(
                "no Windows Search property store (SystemIndex_PropertyStore or SystemIndex_0A)"
                    .into(),
            )
        })?;
    let mut parsed = Parsed {
        schema: Some(schema),
        dirty: db.header.state == DatabaseState::DirtyShutdown,
        problems: db.problems.clone(),
    };
    let scopes = read_scopes(&db, &mut parsed.problems);
    let mut gathers = read_gathers(&db, &mut parsed.problems);
    let table = db
        .table(schema.table())
        .expect("the schema was chosen by this table");
    let key = table.column_index(schema.key()).ok_or_else(|| {
        Skip::NotIndex(format!("{} has no {} column", schema.table(), schema.key()))
    })?;
    let mut rows = db
        .rows(&table.name)
        .map_err(|e| Skip::Unreadable(format!("{}: {e:?}", table.name)))?
        .long_value_limit(ROW_LONG_VALUE_LIMIT);
    let mut seen = HashSet::new();
    for row in &mut rows {
        let Some(document_id) = row.values.get(key).and_then(integer) else {
            parsed
                .problems
                .push(format!("{}: a row without a {}", table.name, schema.key()));
            continue;
        };
        let mut item = Item {
            document_id,
            in_property_store: true,
            gather: gathers.remove(&document_id),
            properties: Vec::new(),
            problems: Vec::new(),
        };
        if !seen.insert(document_id) {
            item.problems.push(format!(
                "{} holds more than one row for {} {document_id}",
                table.name,
                schema.key()
            ));
        }
        for (index, (column, value)) in table.columns.iter().zip(&row.values).enumerate() {
            if index == key || matches!(value, Value::Null) {
                continue;
            }
            item.properties
                .push(property(column, value, schema, &mut item.problems));
        }
        write_item(records, source, &item, &scopes, stats)?;
    }
    parsed.problems.extend(
        rows.problems()
            .iter()
            .map(|problem| format!("{}: {problem}", table.name)),
    );
    // Items the gatherer knows that have no property-store row: crawled but not (or no longer)
    // in the property store.
    let mut rest: Vec<_> = gathers.into_iter().collect();
    rest.sort_by_key(|(id, _)| *id);
    for (document_id, gather) in rest {
        let item = Item {
            document_id,
            in_property_store: false,
            gather: Some(gather),
            properties: Vec::new(),
            problems: Vec::new(),
        };
        write_item(records, source, &item, &scopes, stats)?;
    }
    Ok(parsed)
}

fn read_scopes(db: &Database<'_>, problems: &mut Vec<String>) -> Scopes {
    let mut scopes = Scopes::default();
    let Some(table) = db.table("SystemIndex_GthrPth") else {
        problems.push("no SystemIndex_GthrPth table: gatherer paths cannot be rebuilt".into());
        return scopes;
    };
    let mut rows = match db.rows(&table.name) {
        Ok(rows) => rows,
        Err(e) => {
            problems.push(format!("SystemIndex_GthrPth: {e:?}"));
            return scopes;
        }
    };
    for row in &mut rows {
        let scope = row.get(table, "Scope").and_then(integer);
        let parent = row.get(table, "Parent").and_then(integer);
        let name = row.get(table, "Name").and_then(Value::as_text);
        if let (Some(scope), Some(parent), Some(name)) = (scope, parent, name) {
            scopes.0.insert(scope, (parent, name.to_string()));
        }
    }
    problems.extend(
        rows.problems()
            .iter()
            .map(|problem| format!("SystemIndex_GthrPth: {problem}")),
    );
    scopes
}

/// Any integer column value, whatever its width or sign.
fn integer(value: &Value) -> Option<i64> {
    match value {
        Value::Bool(b) => Some(i64::from(*b)),
        other => other.as_i64(),
    }
}

fn read_gathers(db: &Database<'_>, problems: &mut Vec<String>) -> HashMap<i64, Gather> {
    let mut gathers = HashMap::new();
    let Some(table) = db.table("SystemIndex_Gthr") else {
        problems.push("no SystemIndex_Gthr table: items have no gatherer record".into());
        return gathers;
    };
    let mut rows = match db.rows(&table.name) {
        Ok(rows) => rows,
        Err(e) => {
            problems.push(format!("SystemIndex_Gthr: {e:?}"));
            return gathers;
        }
    };
    for row in &mut rows {
        let Some(document_id) = row.get(table, "DocumentID").and_then(integer) else {
            problems.push("SystemIndex_Gthr: a row without a DocumentID".into());
            continue;
        };
        let mut gather = Gather::default();
        for (column, value) in table.columns.iter().zip(&row.values) {
            if matches!(value, Value::Null) {
                continue;
            }
            let name = column.name.as_str();
            let json = if name.eq_ignore_ascii_case("LastModified") {
                // The gatherer writes this FILETIME big-endian, in every version.
                match value.as_bytes().and_then(filetime_be) {
                    Some(ticks) => {
                        gather.last_modified = Some(ticks);
                        json!(crate::registry::filetime(ticks))
                    }
                    None => json_value(value),
                }
            } else {
                json_value(value)
            };
            match name.to_ascii_lowercase().as_str() {
                "scopeid" => gather.scope_id = integer(value),
                "sdid" => gather.sdid = integer(value),
                "filename" => gather.file_name = value.as_text().map(str::to_string),
                "transactionflags" => gather.transaction_flags = integer(value),
                "crawlnumbercrawled" => gather.crawl_number = integer(value),
                "priority" => gather.priority = integer(value),
                "deletedcount" => gather.deleted_count = integer(value),
                _ => {}
            }
            gather.columns.push((name.to_string(), json));
        }
        if gathers.insert(document_id, gather).is_some() {
            problems.push(format!(
                "SystemIndex_Gthr holds more than one row for DocumentID {document_id}; the last one is used"
            ));
        }
    }
    problems.extend(
        rows.problems()
            .iter()
            .map(|problem| format!("SystemIndex_Gthr: {problem}")),
    );
    gathers
}

/// A property-store column's name, split into the property ID Windows 8 and later prefix it
/// with (`4447-System_ItemPathDisplay`, `15F-System_DateModified`) and the property name.
fn property_name(column: &str) -> (Option<u32>, &str) {
    if let Some((prefix, name)) = column.split_once('-') {
        let digits = prefix.trim_end_matches(|c: char| c.is_ascii_alphabetic());
        if !digits.is_empty()
            && digits.bytes().all(|b| b.is_ascii_digit())
            && prefix.len() - digits.len() <= 1
            && let Ok(id) = digits.parse()
        {
            return (Some(id), name);
        }
    }
    (None, column)
}

/// Whether a property holds a point in time: Windows Search stores them as 8-byte FILETIMEs in
/// binary columns, so the name decides. Durations (`System_Document_TotalEditingTime`,
/// `System_Media_Duration`) are integers in 100 ns units, and
/// `System_Search_LastIndexedTotalTime` is a floating-point column, which the type keeps apart.
fn is_time_property(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    !is_integer_property(name)
        && (lower.contains("date")
            || lower.ends_with("time")
            || lower.ends_with("visited")
            || lower.ends_with("_lastvisit"))
}

/// Whether an 8-byte binary property is an integer, stored in the schema's byte order: sizes,
/// counts and durations (100 ns units).
fn is_integer_property(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower == "system_size"
        || lower.ends_with("_size")
        || lower.ends_with("filecount")
        || lower.contains("duration")
        || lower.ends_with("totaleditingtime")
}

/// Ticks between 1980 and 2100: what a real index timestamp looks like.
fn plausible_filetime(ticks: u64) -> bool {
    (119_600_064_000_000_000..=157_469_472_000_000_000).contains(&ticks)
}

fn filetime_be(bytes: &[u8]) -> Option<u64> {
    bytes.try_into().ok().map(u64::from_be_bytes)
}

fn filetime_le(bytes: &[u8]) -> Option<u64> {
    bytes.try_into().ok().map(u64::from_le_bytes)
}

/// One property of a row, decoded by its name and column type.
fn property(
    column: &Column,
    value: &Value,
    schema: Schema,
    problems: &mut Vec<String>,
) -> Property {
    let (id, name) = property_name(&column.name);
    let mut prop = Property {
        id,
        name: name.to_string(),
        column_type: column.column_type,
        value: Json::Null,
        time: None,
    };
    match value {
        // Unset times and sizes are stored as eight asterisks.
        Value::Binary(bytes)
            if bytes.len() == 8
                && bytes.iter().all(|b| *b == b'*')
                && (is_time_property(name) || is_integer_property(name)) => {}
        Value::Binary(bytes) if bytes.len() == 8 && is_time_property(name) => {
            let (first, second) = if schema.big_endian() {
                (filetime_be(bytes), filetime_le(bytes))
            } else {
                (filetime_le(bytes), filetime_be(bytes))
            };
            // The schema's byte order, unless only the other order gives a believable time. Zero
            // is unset; a few ticks (the index writes 1 for an item that has no time) are no
            // time either, and stay a number.
            let ticks = match (first, second) {
                (Some(first), Some(second))
                    if !plausible_filetime(first) && first != 0 && plausible_filetime(second) =>
                {
                    problems.push(format!(
                        "{name}: the time is stored in the other byte order than the schema's"
                    ));
                    second
                }
                (Some(first), _) => first,
                (None, _) => unreachable!("eight bytes"),
            };
            prop.value = json!(ticks);
            if ticks == 0 || plausible_filetime(ticks) {
                prop.time = crate::registry::filetime(ticks);
            } else {
                problems.push(format!(
                    "{name}: {ticks} FILETIME ticks is not a time between 1980 and 2099, kept as a number"
                ));
            }
        }
        Value::Binary(bytes) if bytes.len() == 8 && is_integer_property(name) => {
            let n = if schema.big_endian() {
                u64::from_be_bytes(bytes[..].try_into().expect("8 bytes"))
            } else {
                u64::from_le_bytes(bytes[..].try_into().expect("8 bytes"))
            };
            prop.value = json!(n);
        }
        Value::Binary(bytes) if schema == Schema::Windows7 && windows7_text_property(name) => {
            prop.value = match windows7_text(bytes) {
                Ok(text) => json!(text),
                Err(why) => {
                    problems.push(format!("{name}: {why}; kept as hex"));
                    json!(hex(bytes))
                }
            };
        }
        Value::Binary(bytes) => {
            prop.value = match utf16_strings(name, bytes) {
                Some(mut strings) if strings.len() == 1 => json!(strings.remove(0)),
                Some(strings) => json!(strings),
                None => json!(hex(bytes)),
            };
        }
        Value::Text(text) => prop.value = json!(text),
        Value::MultiValue(values) => {
            prop.value = Json::Array(
                values
                    .iter()
                    .map(|value| property(column, value, schema, problems).value)
                    .collect(),
            );
        }
        other => prop.value = json_value(other),
    }
    prop
}

/// Properties stored as a vector of strings: UTF-16LE strings separated by NULs in a binary
/// column (Windows 8 and later; Windows 7 stores them as text).
const STRING_VECTOR_PROPERTIES: &[&str] = &[
    "System_Kind",
    "System_Keywords",
    "System_Category",
    "System_Author",
    "System_ItemAuthors",
    "System_ItemParticipants",
    "System_Media_Producer",
    "System_Media_Writer",
    "System_Music_Artist",
    "System_Music_Genre",
    "System_Music_Composer",
    "System_Music_Conductor",
    "System_Photo_TagViewAggregate",
    "System_Photo_PeopleNames",
    "System_Message_ToName",
    "System_Message_CcName",
    "System_Message_BccName",
    "System_Message_ToAddress",
    "System_Message_CcAddress",
    "System_Message_BccAddress",
    "System_Contact_EmailAddresses",
    "System_Activity_AppID-List",
    "System_ActivityHistory_DaysActive",
    "System_ActivityHistory_HoursActive",
];

fn string_vector_property(name: &str) -> bool {
    STRING_VECTOR_PROPERTIES
        .iter()
        .any(|n| n.eq_ignore_ascii_case(name))
}

/// A binary value as the UTF-16LE strings of a string vector. Known vector properties are
/// decoded whatever their text; any other binary value only when it is at least six characters
/// of plain ASCII, since random bytes such as timestamps and identifiers decode as "valid" UTF-16
/// far too often. Everything else stays hexadecimal.
fn utf16_strings(name: &str, bytes: &[u8]) -> Option<Vec<String>> {
    if bytes.is_empty() || !bytes.len().is_multiple_of(2) {
        return None;
    }
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair))
        .collect();
    let text = String::from_utf16(&units).ok()?;
    if text
        .chars()
        .any(|c| c != '\0' && (c.is_control() || c == '\u{FFFD}'))
    {
        return None;
    }
    if !string_vector_property(name)
        && (bytes.len() < 12 || !text.chars().all(|c| c == '\0' || c.is_ascii()))
    {
        return None;
    }
    let strings: Vec<String> = text
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    (!strings.is_empty()).then_some(strings)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}

/// The Windows 7 property store's binary text columns. Windows Search stored these strings
/// obfuscated and compressed (`System_Search_AutoSummary` in practice; the list is libesedb's
/// esedbtools' for the `SystemIndex_0A` table, less the time and size properties decoded above).
const WINDOWS7_TEXT_PROPERTIES: &[&str] = &[
    "System_Author",
    "System_Calendar_Location",
    "System_Calendar_ShowTimeAsText",
    "System_Comment",
    "System_ComputerName",
    "System_Copyright",
    "System_FileExtension",
    "System_FileName",
    "System_FileOwner",
    "System_FlagStatusText",
    "System_Identity",
    "System_Image_Dimensions",
    "System_ImportanceText",
    "System_ItemAuthors",
    "System_ItemFolderNameDisplay",
    "System_ItemFolderPathDisplay",
    "System_ItemFolderPathDisplayNarrow",
    "System_ItemName",
    "System_ItemNameDisplay",
    "System_ItemNamePrefix",
    "System_ItemParticipants",
    "System_ItemPathDisplay",
    "System_ItemPathDisplayNarrow",
    "System_ItemType",
    "System_ItemTypeText",
    "System_ItemUrl",
    "System_Kind",
    "System_KindText",
    "System_Link_TargetParsingPath",
    "System_Media_ClassPrimaryID",
    "System_Media_ClassSecondaryID",
    "System_Media_CollectionGroupID",
    "System_Media_CollectionID",
    "System_Media_ContentID",
    "System_Media_MCDI",
    "System_Media_Publisher",
    "System_Media_SubTitle",
    "System_Message_CcAddress",
    "System_Message_ConversationID",
    "System_Message_FromAddress",
    "System_Message_FromName",
    "System_Message_MessageClass",
    "System_Message_SenderAddress",
    "System_Message_SenderName",
    "System_Message_Store",
    "System_Message_ToAddress",
    "System_Message_ToName",
    "System_MIMEType",
    "System_Music_AlbumArtist",
    "System_Music_AlbumTitle",
    "System_Music_Artist",
    "System_Music_Composer",
    "System_Music_Genre",
    "System_Music_PartOfSet",
    "System_ParsingName",
    "System_RatingText",
    "System_RecordedTV_EpisodeName",
    "System_RecordedTV_ProgramDescription",
    "System_Search_AutoSummary",
    "System_Search_Store",
    "System_Subject",
    "System_Title",
];

fn windows7_text_property(name: &str) -> bool {
    WINDOWS7_TEXT_PROPERTIES
        .iter()
        .any(|n| n.eq_ignore_ascii_case(name))
}

/// A Windows 7 property-store string: the bytes are XORed with a mask derived from their length,
/// then a type byte says how the text is stored: `0` run-length UTF-16, `1` an 8-bit code page,
/// `4` plain bytes; bit `0x02` wraps the rest in LZXPRESS Huffman compression first.
fn windows7_text(bytes: &[u8]) -> Result<String, String> {
    if bytes.is_empty() {
        return Ok(String::new());
    }
    let mask = 0x0500_0113u32 ^ bytes.len() as u32;
    let decoded: Vec<u8> = bytes
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ ((mask >> (8 * (i & 3))) & 0xff) as u8 ^ (i & 0xff) as u8)
        .collect();
    windows7_decoded_text(&decoded, 0)
}

fn windows7_decoded_text(decoded: &[u8], depth: u8) -> Result<String, String> {
    let kind = *decoded.first().ok_or("text shorter than its header")?;
    if kind & 0x02 != 0 {
        if depth > 0 {
            return Err("compression nested more than once".into());
        }
        let size = decoded
            .get(1..3)
            .map(|s| usize::from(u16::from_le_bytes([s[0], s[1]])))
            .ok_or("compressed text shorter than its header")?;
        let mut out = Vec::with_capacity(size + 1);
        out.push(kind - 2);
        let mut plain = Vec::with_capacity(size);
        frnsc_prefetch::decompress::xpress_huff::decompress(&decoded[3..], &mut plain, size)
            .map_err(|e| format!("LZXPRESS Huffman: {e:?}"))?;
        if plain.len() != size {
            return Err("LZXPRESS Huffman text shorter than its declared size".into());
        }
        out.extend_from_slice(&plain);
        return windows7_decoded_text(&out, depth + 1);
    }
    let data = &decoded[1..];
    match kind {
        0 => {
            // [count][high byte] then `count` low bytes
            let mut units = Vec::new();
            let mut at = 0;
            while at < data.len() {
                let header = data
                    .get(at..at + 2)
                    .ok_or("run-length text has an incomplete run header")?;
                let count = usize::from(header[0]);
                let high = header[1];
                at += 2;
                let end = at + count;
                if end > data.len() {
                    return Err("run-length text is shorter than its declared run".into());
                }
                units.extend(
                    data[at..end]
                        .iter()
                        .map(|low| u16::from_le_bytes([*low, high])),
                );
                at = end;
            }
            String::from_utf16(&units).map_err(|_| "run-length text is not UTF-16".to_string())
        }
        1 | 4 => Ok(data.iter().map(|b| windows_1252(*b)).collect()),
        other => Err(format!("unknown text encoding {other:#04x}")),
    }
}

/// Windows-1252: Latin-1 but for the 0x80..=0x9F range.
fn windows_1252(byte: u8) -> char {
    const HIGH: [char; 32] = [
        '\u{20AC}', '\u{FFFD}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}',
        '\u{2021}', '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{FFFD}',
        '\u{017D}', '\u{FFFD}', '\u{FFFD}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}',
        '\u{2022}', '\u{2013}', '\u{2014}', '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}',
        '\u{0153}', '\u{FFFD}', '\u{017E}', '\u{0178}',
    ];
    match byte {
        0x80..=0x9F => HIGH[usize::from(byte - 0x80)],
        _ => char::from(byte),
    }
}

/// A column value as JSON: numbers, booleans and text as themselves, GUIDs as text, binary as hex.
fn json_value(value: &Value) -> Json {
    match value {
        Value::Binary(bytes) => json!(hex(bytes)),
        Value::MultiValue(values) => Json::Array(values.iter().map(json_value).collect()),
        other => crate::srum::json_value(other),
    }
}

// --- records ------------------------------------------------------------------------------------

/// The properties the fixed columns hold; every other one goes to `OtherProperties`.
const COLUMN_PROPERTIES: &[(&str, &str)] = &[
    ("ItemPath", "System_ItemPathDisplay"),
    ("ItemURL", "System_ItemUrl"),
    ("Name", "System_ItemName"),
    ("ItemType", "System_ItemType"),
    ("ItemTypeText", "System_ItemTypeText"),
    ("Kind", "System_Kind"),
    ("MimeType", "System_MIMEType"),
    ("Store", "System_Search_Store"),
    ("IsFolder", "System_IsFolder"),
    ("IsDeleted", "System_IsDeleted"),
    ("Size", "System_Size"),
    ("FileAttributes", "System_FileAttributes"),
    ("FileOwner", "System_FileOwner"),
    ("DateCreated", "System_DateCreated"),
    ("DateModified", "System_DateModified"),
    ("DateAccessed", "System_DateAccessed"),
    ("ItemDate", "System_ItemDate"),
    ("GatherTime", "System_Search_GatherTime"),
    ("Title", "System_Title"),
    ("Author", "System_Author"),
    ("AutoSummary", "System_Search_AutoSummary"),
    ("ComputerName", "System_ComputerName"),
    ("TargetURL", "System_Link_TargetUrl"),
    ("DateVisited", "System_Link_DateVisited"),
    ("ActivityApp", "System_Activity_AppDisplayName"),
    ("ActivityText", "System_Activity_DisplayText"),
    ("ActivityContentURI", "System_Activity_ContentUri"),
    ("ActivityStartTime", "System_ActivityHistory_StartTime"),
    ("ActivityEndTime", "System_ActivityHistory_EndTime"),
];

const CSV_HEADER: &[&str] = &[
    "DocumentID",
    "InTables",
    "ItemPath",
    "ItemURL",
    "GatherPath",
    "Name",
    "ItemType",
    "ItemTypeText",
    "Kind",
    "MimeType",
    "Store",
    "IsFolder",
    "IsDeleted",
    "DeletedCount",
    "Size",
    "FileAttributes",
    "FileOwner",
    "DateCreated",
    "DateModified",
    "DateAccessed",
    "ItemDate",
    "GatherTime",
    "GatherLastModified",
    "Title",
    "Author",
    "AutoSummary",
    "ComputerName",
    "TargetURL",
    "DateVisited",
    "ActivityApp",
    "ActivityText",
    "ActivityContentURI",
    "ActivityStartTime",
    "ActivityEndTime",
    "User",
    "UserSource",
    "ScopeID",
    "SDID",
    "CrawlNumber",
    "GatherFlags",
    "Priority",
    "PropertyCount",
    "OtherProperties",
    "SourceFile",
    "Problems",
];

/// Index-internal columns that are not properties of the item.
fn internal_property(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.starts_with("invertedonly") || lower == "system_null"
}

/// The user whose profile holds the item, from its path (`C:\\Users\\<name>\\…`): the name
/// after `Users` when something follows it, so that `C:\\Users\\desktop.ini` names no user.
fn path_user(path: &str) -> Option<String> {
    let mut parts = path
        .split(['\\', '/'])
        .filter(|part| !part.is_empty())
        .peekable();
    while let Some(part) = parts.next() {
        if part.eq_ignore_ascii_case("Users") {
            let user = parts.next()?;
            return parts.peek().is_some().then(|| user.to_string());
        }
    }
    None
}

/// The item's columns, in `CSV_HEADER` order, plus the JSON-only `Properties` and `Gather`.
struct Row<'a> {
    source: &'a Source,
    item: &'a Item,
    scopes: &'a Scopes,
}

impl Row<'_> {
    fn gather_path(&self) -> Option<String> {
        let gather = self.item.gather.as_ref()?;
        let scope = gather.scope_id.and_then(|scope| self.scopes.path(scope));
        match (scope, &gather.file_name) {
            (Some(scope), Some(name)) => Some(format!("{scope}{name}")),
            (Some(scope), None) => Some(scope),
            (None, Some(name)) => Some(name.clone()),
            (None, None) => None,
        }
    }

    fn name(&self) -> Option<String> {
        self.item
            .text("System_ItemName")
            .or_else(|| self.item.text("System_ItemNameDisplay"))
            .or_else(|| self.item.text("System_FileName"))
            .or_else(|| {
                self.item
                    .gather
                    .as_ref()
                    .and_then(|gather| gather.file_name.clone())
            })
    }

    fn item_path(&self) -> Option<String> {
        self.item.text("System_ItemPathDisplay")
    }

    /// The profile named in the item's path, and which path it was read from.
    fn user(&self) -> Option<(String, &'static str)> {
        [
            (self.item_path(), "ItemPath"),
            (self.item.text("System_ItemUrl"), "ItemURL"),
            (self.gather_path(), "GatherPath"),
        ]
        .into_iter()
        .find_map(|(path, source)| path_user(&path?).map(|user| (user, source)))
    }

    fn other_properties(&self) -> Vec<&Property> {
        self.item
            .properties
            .iter()
            .filter(|property| {
                !internal_property(&property.name)
                    && !COLUMN_PROPERTIES
                        .iter()
                        .any(|(_, name)| name.eq_ignore_ascii_case(&property.name))
            })
            .collect()
    }

    fn cells(&self) -> Vec<String> {
        let item = self.item;
        let gather = item.gather.as_ref();
        let text = |name: &str| item.text(name).unwrap_or_default();
        let time = |name: &str| item.time(name).unwrap_or_default();
        let number = |n: Option<i64>| n.map(|n| n.to_string()).unwrap_or_default();
        let (user, user_source) = self
            .user()
            .map_or((None, None), |(u, s)| (Some(u), Some(s)));
        let other = self
            .other_properties()
            .iter()
            .map(|property| {
                format!(
                    "{}={}",
                    property.name,
                    property
                        .time
                        .clone()
                        .unwrap_or_else(|| cell(&property.value))
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        vec![
            item.document_id.to_string(),
            item.in_tables().to_string(),
            self.item_path().unwrap_or_default(),
            text("System_ItemUrl"),
            self.gather_path().unwrap_or_default(),
            self.name().unwrap_or_default(),
            text("System_ItemType"),
            text("System_ItemTypeText"),
            text("System_Kind"),
            text("System_MIMEType"),
            text("System_Search_Store"),
            text("System_IsFolder"),
            text("System_IsDeleted"),
            number(gather.and_then(|g| g.deleted_count)),
            text("System_Size"),
            text("System_FileAttributes"),
            text("System_FileOwner"),
            time("System_DateCreated"),
            time("System_DateModified"),
            time("System_DateAccessed"),
            time("System_ItemDate"),
            time("System_Search_GatherTime"),
            gather
                .and_then(|g| g.last_modified)
                .and_then(crate::registry::filetime)
                .unwrap_or_default(),
            text("System_Title"),
            text("System_Author"),
            text("System_Search_AutoSummary"),
            text("System_ComputerName"),
            text("System_Link_TargetUrl"),
            time("System_Link_DateVisited"),
            text("System_Activity_AppDisplayName"),
            text("System_Activity_DisplayText"),
            text("System_Activity_ContentUri"),
            time("System_ActivityHistory_StartTime"),
            time("System_ActivityHistory_EndTime"),
            user.clone().unwrap_or_default(),
            user_source.unwrap_or_default().to_string(),
            number(gather.and_then(|g| g.scope_id)),
            number(gather.and_then(|g| g.sdid)),
            number(gather.and_then(|g| g.crawl_number)),
            number(gather.and_then(|g| g.transaction_flags)),
            number(gather.and_then(|g| g.priority)),
            item.properties.len().to_string(),
            other,
            self.source.shown.clone(),
            item.problems.join("; "),
        ]
    }

    fn json(&self) -> Json {
        let item = self.item;
        let gather = item.gather.as_ref();
        let time = |name: &str| item.time(name).map_or(Json::Null, |t| json!(t));
        let text = |name: &str| item.json(name);
        let user = self.user();
        let mut map = Map::new();
        let mut put = |key: &str, value: Json| {
            map.insert(key.to_string(), value);
        };
        put("DocumentID", json!(item.document_id));
        put("InTables", json!(item.in_tables()));
        put("ItemPath", opt(self.item_path()));
        put("ItemURL", text("System_ItemUrl"));
        put("GatherPath", opt(self.gather_path()));
        put("Name", opt(self.name()));
        put("ItemType", text("System_ItemType"));
        put("ItemTypeText", text("System_ItemTypeText"));
        put("Kind", text("System_Kind"));
        put("MimeType", text("System_MIMEType"));
        put("Store", text("System_Search_Store"));
        put("IsFolder", text("System_IsFolder"));
        put("IsDeleted", text("System_IsDeleted"));
        put("DeletedCount", json!(gather.and_then(|g| g.deleted_count)));
        put("Size", text("System_Size"));
        put("FileAttributes", text("System_FileAttributes"));
        put("FileOwner", text("System_FileOwner"));
        put("DateCreated", time("System_DateCreated"));
        put("DateModified", time("System_DateModified"));
        put("DateAccessed", time("System_DateAccessed"));
        put("ItemDate", time("System_ItemDate"));
        put("GatherTime", time("System_Search_GatherTime"));
        put(
            "GatherLastModified",
            json!(
                gather
                    .and_then(|g| g.last_modified)
                    .and_then(crate::registry::filetime)
            ),
        );
        put("Title", text("System_Title"));
        put("Author", text("System_Author"));
        put("AutoSummary", text("System_Search_AutoSummary"));
        put("ComputerName", text("System_ComputerName"));
        put("TargetURL", text("System_Link_TargetUrl"));
        put("DateVisited", time("System_Link_DateVisited"));
        put("ActivityApp", text("System_Activity_AppDisplayName"));
        put("ActivityText", text("System_Activity_DisplayText"));
        put("ActivityContentURI", text("System_Activity_ContentUri"));
        put(
            "ActivityStartTime",
            time("System_ActivityHistory_StartTime"),
        );
        put("ActivityEndTime", time("System_ActivityHistory_EndTime"));
        put("User", opt(user.as_ref().map(|(user, _)| user.clone())));
        put(
            "UserSource",
            json!(user.as_ref().map(|(_, source)| *source)),
        );
        put("ScopeID", json!(gather.and_then(|g| g.scope_id)));
        put("SDID", json!(gather.and_then(|g| g.sdid)));
        put("CrawlNumber", json!(gather.and_then(|g| g.crawl_number)));
        put(
            "GatherFlags",
            json!(gather.and_then(|g| g.transaction_flags)),
        );
        put("Priority", json!(gather.and_then(|g| g.priority)));
        put("PropertyCount", json!(item.properties.len()));
        put(
            "Properties",
            Json::Array(
                item.properties
                    .iter()
                    .map(|property| {
                        let mut object = Map::new();
                        object.insert("ID".into(), json!(property.id));
                        object.insert("Name".into(), json!(property.name));
                        object.insert("Type".into(), json!(format!("{:?}", property.column_type)));
                        if let Some(time) = &property.time {
                            object.insert("Time".into(), json!(time));
                            object.insert("Ticks".into(), property.value.clone());
                        } else {
                            object.insert("Value".into(), property.value.clone());
                        }
                        Json::Object(object)
                    })
                    .collect(),
            ),
        );
        put(
            "Gather",
            gather.map_or(Json::Null, |gather| {
                Json::Object(gather.columns.iter().cloned().collect())
            }),
        );
        put("SourceFile", json!(self.source.shown));
        put("Problems", json!(item.problems.join("; ")));
        Json::Object(map)
    }
}

fn opt(value: Option<String>) -> Json {
    value.map_or(Json::Null, |v| json!(v))
}

/// A JSON value as one CSV cell: lists joined with `|`, text as is.
fn cell(value: &Json) -> String {
    match value {
        Json::Null => String::new(),
        Json::String(s) => s.clone(),
        Json::Array(items) => items.iter().map(cell).collect::<Vec<_>>().join("|"),
        other => other.to_string(),
    }
}

fn write_item(
    records: &mut Option<Records>,
    source: &Source,
    item: &Item,
    scopes: &Scopes,
    stats: &mut Stats,
) -> Result<(), Skip> {
    stats.items += 1;
    match item.in_tables() {
        "Gthr+PropertyStore" => stats.both += 1,
        "Gthr" => stats.gather_only += 1,
        _ => stats.property_only += 1,
    }
    if item.text("System_IsDeleted").as_deref() == Some("true")
        || item
            .gather
            .as_ref()
            .and_then(|g| g.deleted_count)
            .is_some_and(|n| n > 0)
    {
        stats.deleted += 1;
    }
    if item.text("System_IsFolder").as_deref() == Some("true") {
        stats.folders += 1;
    }
    if let Some(store) = item.text("System_Search_Store") {
        *stats.stores.entry(store).or_default() += 1;
    }
    if item.time("System_DateModified").is_none()
        && item.time("System_Search_GatherTime").is_none()
        && item.gather.as_ref().and_then(|g| g.last_modified).is_none()
    {
        stats.untimed += 1;
    }
    if !item.problems.is_empty() {
        stats.with_problems += 1;
    }
    if let Some(records) = records {
        records
            .write(&Row {
                source,
                item,
                scopes,
            })
            .map_err(|e| Skip::Unreadable(format!("could not write the records: {e}")))?;
    }
    Ok(())
}

/// Where the records go, written one item at a time.
enum Records {
    Csv(Box<csv::Writer<Box<dyn Write>>>),
    /// Pretty JSON objects when `true`, else one object per line.
    Json(Box<dyn Write>, bool),
}

impl Records {
    /// A CSV gets its header now, so that it has one also without records.
    fn new(out: Box<dyn Write>, output_type: WindowsSearchOutputType) -> io::Result<Self> {
        Ok(match output_type {
            WindowsSearchOutputType::Csv => {
                let mut csv = csv::Writer::from_writer(out);
                csv.write_record(crate::output_schema::labels(CSV_HEADER.iter().copied())?)?;
                Records::Csv(Box::new(csv))
            }
            WindowsSearchOutputType::Json => Records::Json(out, true),
            WindowsSearchOutputType::Jsonl => Records::Json(out, false),
        })
    }

    fn write(&mut self, row: &Row<'_>) -> io::Result<()> {
        match self {
            Records::Csv(csv) => csv.write_record(row.cells()).map_err(io::Error::other),
            Records::Json(out, pretty) => {
                crate::output_schema::write_json(&mut *out, &row.json(), *pretty)?;
                out.write_all(b"\n")
            }
        }
    }

    fn finish(self) -> io::Result<()> {
        match self {
            Records::Csv(mut csv) => csv.flush(),
            Records::Json(mut out, _) => out.flush(),
        }
    }
}

// --- command ------------------------------------------------------------------------------------

/// Run the command; returns the exit status.
pub fn run(opts: &WindowsSearchOptions) -> i32 {
    let no_color = opts.common.no_color;
    let data_on_stdout = opts.data_on_stdout();
    let report_error = |bar: Option<&ProgressBar>, msg: &str| {
        if data_on_stdout {
            eprintln!("{msg}");
        } else {
            progress::suspended(bar, || error_msg(no_color, msg));
        }
    };

    // --- inputs ---------------------------------------------------------------------------
    let sources = match (&opts.filepath, &opts.directory) {
        (Some(f), _) => {
            if !f.is_file() {
                report_error(None, &format!("The file does not exist: {}", f.display()));
                return 1;
            }
            vec![Source::given(f.clone())]
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
                        "No Windows Search databases (Windows.edb, Windows.db) were found in {}",
                        d.display()
                    ),
                );
                return 1;
            }
            found
        }
        (None, None) => unreachable!("clap requires -f or -d"),
    };
    let sink =
        match crate::registry::sink(opts.output.as_ref(), opts.clobber, data_on_stdout, || {
            sources.iter().map(|source| source.path.clone()).collect()
        }) {
            Ok(sink) => sink,
            Err(msg) => {
                report_error(None, &msg);
                return 1;
            }
        };
    let mut records = match sink
        .map(|sink| Records::new(sink, opts.output_type()))
        .transpose()
    {
        Ok(records) => records,
        Err(e) => {
            report_error(None, &format!("Could not write the records: {e}"));
            return 1;
        }
    };
    if !data_on_stdout {
        let total: u64 = sources
            .iter()
            .map(|source| source.path.metadata().map_or(0, |m| m.len()))
            .sum();
        p_label(
            no_color,
            "Windows Search databases: ",
            &fmt_num(sources.len()),
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
        progress::progress_bar(sources.len(), no_color)
    });

    // --- read and write ---------------------------------------------------------------------
    let mut stats = Stats::default();
    let mut printed = 0usize;
    let report = |bar: Option<&ProgressBar>, printed: &mut usize, message: &str| {
        *printed += 1;
        if *printed <= MAX_PRINTED_ERRORS {
            report_error(bar, message);
        }
    };
    for source in &sources {
        if let Some(b) = &bar {
            b.set_message(progress::message(&source.path));
        }
        match parse(source, &mut records, &mut stats) {
            Ok(parsed) => {
                stats.databases += 1;
                match parsed.schema {
                    Some(Schema::Windows7) => stats.windows7 += 1,
                    Some(Schema::PropertyStore) => stats.property_store += 1,
                    None => {}
                }
                if parsed.dirty {
                    stats.dirty += 1;
                }
                stats.problems += parsed.problems.len();
                for problem in &parsed.problems {
                    report(
                        bar.as_ref(),
                        &mut printed,
                        &format!("{}: {problem}", source.shown),
                    );
                }
            }
            Err(Skip::Sqlite) => {
                stats.sqlite += 1;
                report(
                    bar.as_ref(),
                    &mut printed,
                    &format!(
                        "Unsupported newer format, not read: {} is the SQLite index of Windows 11 22H2 or later",
                        source.shown
                    ),
                );
            }
            Err(Skip::NotIndex(why)) => {
                stats.not_index += 1;
                report(
                    bar.as_ref(),
                    &mut printed,
                    &format!(
                        "Not read as a Windows Search index: {}: {why}",
                        source.shown
                    ),
                );
            }
            Err(Skip::Unreadable(why)) => {
                if why.starts_with("could not write the records") {
                    // The output is gone (a full disk, a closed pipe): reading on cannot save
                    // anything.
                    if let Some(b) = &bar {
                        b.abandon();
                    }
                    report_error(None, &format!("Could not write the records: {why}"));
                    return 1;
                }
                stats.unreadable += 1;
                report(
                    bar.as_ref(),
                    &mut printed,
                    &format!("Could not read {}: {why}", source.shown),
                );
            }
        }
        if let Some(b) = &bar {
            b.inc(1);
        }
    }
    if printed > MAX_PRINTED_ERRORS {
        report_error(
            bar.as_ref(),
            &format!(
                "{} more messages were not printed",
                fmt_num(printed - MAX_PRINTED_ERRORS)
            ),
        );
    }
    if let Some(b) = &bar {
        b.finish_with_message(progress::finished_message(no_color));
    }
    if let Some(records) = records
        && let Err(e) = records.finish()
    {
        report_error(None, &format!("Could not write the records: {e}"));
        return 1;
    }

    // --- summary ---------------------------------------------------------------------------
    if !data_on_stdout {
        print_summary(&stats, no_color);
        if stats.items > 0 {
            if let Some(out) = &opts.output {
                p_label(
                    no_color,
                    "Results saved: ",
                    &format!("{} ({})", out.display(), util::file_size_display(out)),
                );
            } else {
                p(
                    None,
                    "To save every record, add -o <FILE> (-t csv, json or jsonl).",
                    true,
                );
            }
        }
    }
    i32::from(stats.items == 0)
}

fn print_summary(stats: &Stats, no_color: bool) {
    p(None, "", true);
    let mut read = fmt_num(stats.databases);
    let mut versions = Vec::new();
    if stats.windows7 > 0 {
        versions.push(format!(
            "{} {}",
            fmt_num(stats.windows7),
            Schema::Windows7.name()
        ));
    }
    if stats.property_store > 0 {
        versions.push(format!(
            "{} {}",
            fmt_num(stats.property_store),
            Schema::PropertyStore.name()
        ));
    }
    if !versions.is_empty() {
        read = format!("{read} ({})", versions.join(", "));
    }
    p_label(no_color, "Databases read: ", &read);
    if stats.sqlite > 0 {
        p_label(
            no_color,
            "Unsupported newer format (SQLite, Windows 11 22H2+): ",
            &fmt_num(stats.sqlite),
        );
    }
    if stats.not_index > 0 {
        p_label(
            no_color,
            "Not Windows Search indexes: ",
            &fmt_num(stats.not_index),
        );
    }
    if stats.unreadable > 0 {
        p_label(no_color, "Could not read: ", &fmt_num(stats.unreadable));
    }
    if stats.dirty > 0 {
        p_label(
            no_color,
            "Dirty shutdown (transaction logs not replayed): ",
            &fmt_num(stats.dirty),
        );
    }
    p_label(no_color, "Indexed items: ", &fmt_num(stats.items));
    p_label(
        no_color,
        "In both tables / gatherer only / property store only: ",
        &format!(
            "{} / {} / {}",
            fmt_num(stats.both),
            fmt_num(stats.gather_only),
            fmt_num(stats.property_only)
        ),
    );
    if stats.folders > 0 {
        p_label(no_color, "Folders: ", &fmt_num(stats.folders));
    }
    if stats.deleted > 0 {
        p_label(no_color, "Marked deleted: ", &fmt_num(stats.deleted));
    }
    if !stats.stores.is_empty() {
        p_label(
            no_color,
            "Stores: ",
            &stats
                .stores
                .iter()
                .map(|(store, n)| format!("{store} {}", fmt_num(*n)))
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    if stats.untimed > 0 {
        p_label(
            no_color,
            "Items without a stored time: ",
            &fmt_num(stats.untimed),
        );
    }
    if stats.with_problems > 0 {
        p_label(
            no_color,
            "Items with problems: ",
            &fmt_num(stats.with_problems),
        );
    }
    if stats.problems > 0 {
        p_label(no_color, "Database problems: ", &fmt_num(stats.problems));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_names_and_staged_companions_are_recognized() {
        for name in [
            "Windows.edb",
            "windows.EDB",
            "Windows.db",
            "Windows-gather.db",
            "Windows-usn.db",
        ] {
            assert!(is_database_name(name), "{name}");
            assert!(is_staged_name(name), "{name}");
        }
        for name in [
            "Windows.jfm",
            "MSS.log",
            "MSS00012.log",
            "MSS.chk",
            "MSSres00001.jrs",
        ] {
            assert!(!is_database_name(name), "{name}");
            assert!(is_staged_name(name), "{name}");
        }
        for name in [
            "Windows.edb.bak",
            "mss.txt",
            "Windows",
            "SystemIndex.1.gthr",
        ] {
            assert!(!is_database_name(name), "{name}");
            assert!(!is_staged_name(name), "{name}");
        }
    }

    #[test]
    fn property_store_column_names_split_into_id_and_property() {
        assert_eq!(
            property_name("4447-System_ItemPathDisplay"),
            (Some(4447), "System_ItemPathDisplay")
        );
        assert_eq!(
            property_name("15F-System_DateModified"),
            (Some(15), "System_DateModified")
        );
        assert_eq!(
            property_name("0F-InvertedOnlyMD5"),
            (Some(0), "InvertedOnlyMD5")
        );
        assert_eq!(
            property_name("System_ItemPathDisplay"),
            (None, "System_ItemPathDisplay")
        );
        assert_eq!(
            property_name("Microsoft_IE-Title"),
            (None, "Microsoft_IE-Title")
        );
        assert_eq!(property_name("ab-c"), (None, "ab-c"));
    }

    #[test]
    fn times_are_decoded_in_the_schema_byte_order_with_a_checked_fallback() {
        let column = |name: &str| Column {
            id: 1,
            name: name.into(),
            column_type: ColumnType::Binary,
            codepage: 0,
            flags: 0,
            max_size: 8,
            record_offset: 0,
            default: None,
            from_template: false,
        };
        // 2023-02-08T22:13:58.617646Z as Windows 10 writes it (little-endian)
        let le = Value::Binary(vec![0xCC, 0x55, 0x31, 0xA4, 0x0A, 0x3C, 0xD9, 0x01]);
        let mut problems = Vec::new();
        let p = property(
            &column("15F-System_DateModified"),
            &le,
            Schema::PropertyStore,
            &mut problems,
        );
        assert_eq!(p.time.as_deref(), Some("2023-02-08T22:13:58.617646Z"));
        assert_eq!(p.value, json!(0x01D93C0AA43155CCu64));
        assert!(problems.is_empty());
        // The same instant big-endian, as Windows 7 writes it
        let be = Value::Binary(vec![0x01, 0xD9, 0x3C, 0x0A, 0xA4, 0x31, 0x55, 0xCC]);
        let p = property(
            &column("System_DateModified"),
            &be,
            Schema::Windows7,
            &mut problems,
        );
        assert_eq!(p.time.as_deref(), Some("2023-02-08T22:13:58.617646Z"));
        assert!(problems.is_empty());
        // A Windows 7 schema with a little-endian value is read, with a note
        let p = property(
            &column("System_DateCreated"),
            &le,
            Schema::Windows7,
            &mut problems,
        );
        assert_eq!(p.time.as_deref(), Some("2023-02-08T22:13:58.617646Z"));
        assert_eq!(problems.len(), 1);
        // Eight bytes that are no time stay hexadecimal, unset times stay unset
        let p = property(
            &column("4678-System_ThumbnailCacheID"),
            &Value::Binary(vec![0xBD, 0xAE, 0xFA, 0x18, 0xD1, 0x9D, 0x62, 0xE1]),
            Schema::PropertyStore,
            &mut problems,
        );
        assert_eq!(p.value, json!("BDAEFA18D19D62E1"));
        assert!(p.time.is_none());
        let p = property(
            &column("System_DateAccessed"),
            &Value::Binary(vec![0; 8]),
            Schema::Windows7,
            &mut problems,
        );
        assert_eq!(p.value, json!(0));
        assert!(p.time.is_none());
        assert_eq!(problems.len(), 1);
        // One tick (what the index writes for an item without a time) is not 1601-01-01
        let p = property(
            &column("System_DateModified"),
            &Value::Binary(vec![0, 0, 0, 0, 0, 0, 0, 1]),
            Schema::Windows7,
            &mut problems,
        );
        assert_eq!(p.value, json!(1));
        assert!(p.time.is_none());
        assert_eq!(problems.len(), 2);
        assert!(problems[1].contains("1 FILETIME ticks"), "{problems:?}");
        problems.pop();
        for name in ["System_DateAccessed", "13F-System_Size"] {
            let p = property(
                &column(name),
                &Value::Binary(b"********".to_vec()),
                Schema::PropertyStore,
                &mut problems,
            );
            assert_eq!(p.value, Json::Null, "{name}");
            assert!(p.time.is_none());
        }
        assert_eq!(problems.len(), 1);
    }

    #[test]
    fn multi_valued_text_and_sizes_are_decoded_and_scopes_are_rebuilt() {
        assert_eq!(
            utf16_strings("System_Kind", b"f\0o\0l\0d\0e\0r\0"),
            Some(vec!["folder".to_string()])
        );
        assert_eq!(
            utf16_strings("System_Keywords", b"a\0\0\0b\0"),
            Some(vec!["a".to_string(), "b".to_string()])
        );
        // An unlisted property: long plain ASCII only
        assert_eq!(
            utf16_strings("System_Other", b"A\0d\0o\0b\0e\0 \0P\0D\0F\0"),
            Some(vec!["Adobe PDF".to_string()])
        );
        assert_eq!(utf16_strings("System_Other", b"a\0b\0"), None);
        assert_eq!(
            utf16_strings("System_Other", &[0xBD, 0xAE, 0xFA, 0x18]),
            None
        );
        assert_eq!(
            utf16_strings(
                "System_Other",
                &[0xCC, 0x55, 0x31, 0xA4, 0x0A, 0x3C, 0xD9, 0x01]
            ),
            None
        );
        assert_eq!(utf16_strings("System_Kind", &[1, 2, 3]), None);
        let mut scopes = Scopes::default();
        for (scope, parent, name) in [
            (2, 1, "file:"),
            (3, 2, "C:"),
            (4, 3, "/"),
            (5, 4, "Users/"),
            (6, 5, "test/"),
            (9, 9, "loop/"),
        ] {
            scopes.0.insert(scope, (parent, name.into()));
        }
        assert_eq!(scopes.path(6).as_deref(), Some("file:C:/Users/test/"));
        assert_eq!(scopes.path(2).as_deref(), Some("file:"));
        assert_eq!(scopes.path(9).as_deref(), Some("loop/"));
        assert_eq!(scopes.path(99), None);
        assert_eq!(
            path_user("C:\\Users\\John Doe\\Documents\\a.txt").as_deref(),
            Some("John Doe")
        );
        assert_eq!(path_user("file:C:/Users/test/x").as_deref(), Some("test"));
        assert_eq!(path_user("C:\\Windows\\Users"), None);
        assert_eq!(path_user("C:\\Users\\desktop.ini"), None);
        assert_eq!(
            path_user("C:\\Users\\Public\\Desktop\\a.lnk").as_deref(),
            Some("Public")
        );
        assert!(is_time_property("System_Search_GatherTime"));
        assert!(is_time_property("Microsoft_IE_LastVisited"));
        assert!(!is_time_property("System_Document_TotalEditingTime"));
        assert!(!is_time_property("System_Media_Duration"));
        assert!(is_integer_property("System_Size"));
        assert!(is_integer_property("System_Document_TotalEditingTime"));
    }

    #[test]
    fn windows_7_strings_are_deobfuscated_and_decoded() {
        // DocID 3 of plaso's Windows.edb: an 8-bit string (type 1), as esedbexport reads it
        let encoded: Vec<u8> = (0..131u8).collect::<Vec<_>>();
        let _ = encoded;
        let text = "Choose which programs you want Windows to use for activities like web browsing, editing photos, sending e-mail, and playing music.";
        // Encode: type byte 1 + the text, then the XOR mask, to check the round trip exactly.
        let mut decoded = vec![1u8];
        decoded.extend_from_slice(text.as_bytes());
        let mask = 0x0500_0113u32 ^ decoded.len() as u32;
        let encoded: Vec<u8> = decoded
            .iter()
            .enumerate()
            .map(|(i, b)| b ^ ((mask >> (8 * (i & 3))) & 0xff) as u8 ^ (i & 0xff) as u8)
            .collect();
        assert_eq!(encoded.len(), 131);
        assert_eq!(windows7_text(&encoded).unwrap(), text);
        // Run-length UTF-16: [count][high] then the low bytes
        let mut decoded = vec![0u8, 3, 0, b'a', b'b', b'c', 1, 0x4e, 0x2d];
        let mask = 0x0500_0113u32 ^ decoded.len() as u32;
        for (i, b) in decoded.iter_mut().enumerate() {
            *b ^= ((mask >> (8 * (i & 3))) & 0xff) as u8 ^ (i & 0xff) as u8;
        }
        assert_eq!(windows7_text(&decoded).unwrap(), "abc\u{4e2d}");
        assert!(windows7_text_property("System_Search_AutoSummary"));
        assert!(!windows7_text_property("System_DateModified"));
        assert_eq!(windows_1252(0x80), '\u{20AC}');
        assert_eq!(windows_1252(0xE9), 'é');
    }

    #[test]
    fn null_properties_do_not_hide_property_store_membership() {
        let mut item = Item {
            document_id: 1,
            in_property_store: true,
            gather: Some(Gather::default()),
            properties: Vec::new(),
            problems: Vec::new(),
        };
        assert_eq!(item.in_tables(), "Gthr+PropertyStore");
        item.in_property_store = false;
        assert_eq!(item.in_tables(), "Gthr");
        item.in_property_store = true;
        item.gather = None;
        assert_eq!(item.in_tables(), "PropertyStore");
    }

    #[test]
    fn fallback_read_rejects_growth_beyond_the_limit() {
        assert_eq!(read_bounded(&b"abc"[..], 3).ok().unwrap(), b"abc");
        assert!(matches!(
            read_bounded(&b"abcd"[..], 3),
            Err(Skip::Unreadable(_))
        ));
    }

    #[test]
    fn damaged_windows_7_run_length_text_keeps_the_original_bytes() {
        let column = Column {
            id: 1,
            name: "System_Search_AutoSummary".into(),
            column_type: ColumnType::Binary,
            codepage: 0,
            flags: 0,
            max_size: 255,
            record_offset: 0,
            default: None,
            from_template: false,
        };
        for plain in [vec![0, 3, 0, b'a'], vec![0, 1, 0, b'a', 2]] {
            let mask = 0x0500_0113u32 ^ plain.len() as u32;
            let encoded: Vec<u8> = plain
                .iter()
                .enumerate()
                .map(|(i, b)| b ^ ((mask >> (8 * (i & 3))) & 0xff) as u8 ^ (i & 0xff) as u8)
                .collect();
            let mut problems = Vec::new();
            let value = property(
                &column,
                &Value::Binary(encoded.clone()),
                Schema::Windows7,
                &mut problems,
            );
            assert_eq!(value.value, json!(hex(&encoded)));
            assert!(
                problems
                    .iter()
                    .any(|p| p.contains("run-length text") && p.contains("kept as hex"))
            );
        }
    }

    #[test]
    fn csv_header_and_json_record_hold_the_same_fields() {
        let item = Item {
            document_id: 7,
            in_property_store: true,
            gather: None,
            properties: Vec::new(),
            problems: Vec::new(),
        };
        let source = Source::given(PathBuf::from("Windows.edb"));
        let scopes = Scopes::default();
        let row = Row {
            source: &source,
            item: &item,
            scopes: &scopes,
        };
        assert_eq!(row.cells().len(), CSV_HEADER.len());
        let Json::Object(map) = row.json() else {
            panic!("object")
        };
        // JSON carries every property in `Properties` instead of the CSV's `OtherProperties`
        // summary, and the gatherer row as `Gather`.
        for key in CSV_HEADER.iter().filter(|key| **key != "OtherProperties") {
            assert!(map.contains_key(*key), "{key}");
        }
        assert!(map.contains_key("Properties") && map.contains_key("Gather"));
        assert_eq!(map.len(), CSV_HEADER.len() + 1);
        assert_eq!(map["InTables"], "PropertyStore");
        assert_eq!(map["SourceFile"], "Windows.edb");
    }
}
