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

//! A browsable, explicitly incomplete reconstruction, never a restored live NTFS volume.
//! Names are individual evidence components, not host paths. No links, ACLs, original timestamps
//! or executable permissions are restored. The CSV inventory distinguishes content from placeholders.
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use ys_ntfs_parser::mft::attribute::x30::FileNamespace;
use ys_ntfs_parser::mft::i30::IndexEntry;
use ys_ntfs_parser::mft::{MftEntry, attribute::MftAttributeType};

pub(crate) const INVENTORY: &str = "_Juntei-Reconstruction.csv";
const INVENTORY_HEADER: &[&str] = &[
    "Volume",
    "DriveLetter",
    "Source",
    "EntryNum",
    "SN",
    "ParentEntryNum",
    "ParentSN",
    "OriginalPath",
    "OutputPath",
    "Dir",
    "DeletionStatus",
    "DataStatus",
    "LogicalSize",
    "RecoveredBytes",
    "PathStatus",
    "FromSlack",
    "Offset",
    "Notes",
    "File",
];
const UNRESOLVED: &str = "_Juntei-Unresolved";
const PARTIAL_INVENTORY: &str = "_Juntei-Reconstruction.partial.csv";
const MAX_DEPTH: usize = 128;
// macOS PATH_MAX is 1024 bytes; Linux permits 4096. Leave space for collision suffixes.
const MAX_HOST_PATH_BYTES: usize = if cfg!(target_os = "macos") { 900 } else { 3500 };
type Reference = (u64, u16);

/// Only collection metadata can establish a drive letter; an MFT does not contain one.
/// The anchor also prevents merging indexes from unrelated volumes with the same drive letter.
pub(crate) fn volume_hint(path: &Path) -> (Option<char>, PathBuf) {
    let mut anchor = PathBuf::new();
    let mut result = None;
    let parts: Vec<_> = path
        .parent()
        .unwrap_or(Path::new(""))
        .components()
        .collect();
    let mut previous = String::new();
    for (i, part) in parts.iter().enumerate() {
        anchor.push(part.as_os_str());
        // A Windows analyst's local drive prefix is not an evidence volume label.
        if !matches!(part, std::path::Component::Normal(_)) {
            continue;
        }
        let raw = part.as_os_str().to_string_lossy();
        let decoded = percent_decode(&raw);
        let name = decoded
            .strip_prefix(r"\\.\")
            .or_else(|| decoded.strip_prefix(r"\\?\"))
            .unwrap_or(&decoded);
        let bytes = name.as_bytes();
        let plain_label = bytes.len() == 1
            && (i + 1 == parts.len() || matches!(previous.as_str(), "auto" | "ntfs"));
        if (plain_label || (bytes.len() == 2 && matches!(bytes[1], b':' | b'$')))
            && bytes[0].is_ascii_alphabetic()
        {
            result = Some((bytes[0].to_ascii_uppercase() as char, anchor.clone()));
        }
        previous = raw.to_ascii_lowercase();
    }
    result.map_or_else(
        || (None, path.parent().unwrap_or(Path::new("")).to_owned()),
        |(c, p)| (Some(c), p),
    )
}

fn percent_decode(s: &str) -> String {
    let mut out = Vec::new();
    let mut bytes = s.bytes();
    while let Some(c) = bytes.next() {
        if c == b'%' {
            let a = bytes.next();
            let b = bytes.next();
            if let (Some(a), Some(b)) = (a, b)
                && let (Some(a), Some(b)) = ((a as char).to_digit(16), (b as char).to_digit(16))
            {
                out.push((a * 16 + b) as u8);
                continue;
            }
            out.push(c);
            out.extend(a);
            out.extend(b);
        } else {
            out.push(c);
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Name {
    parent: Reference,
    text: String,
}

#[derive(Debug)]
enum Content {
    Resident(Vec<u8>),
    NonResident(u64),
    Unavailable,
}

#[derive(Debug)]
struct Node {
    sequence: u16,
    directory: bool,
    deleted: bool,
    names: Vec<Name>,
    content: Content,
    notes: String,
}

impl Node {
    fn matches(&self, sequence: u16) -> bool {
        sequence != 0
            && (self.sequence == sequence
                || (self.deleted && self.sequence == sequence.wrapping_add(1)))
    }
}

#[derive(Debug)]
struct IndexedName {
    name: Name,
    reference: Option<Reference>,
    directory: bool,
    size: u64,
    slack: bool,
    source: String,
    offset: u64,
}

/// Keep only metadata and resident bytes, not entire FILE records. Filled during the MFT parser's
/// existing first pass; the directory graph is complete before any generated paths are chosen.
#[derive(Default)]
pub(crate) struct Builder {
    nodes: BTreeMap<u64, Node>,
    indexes: Vec<IndexedName>,
    rejected: u64,
}

impl Builder {
    pub(crate) fn add(&mut self, id: u64, entry: &MftEntry, extensions: &[MftEntry]) {
        if !entry.header.is_valid() || !entry.header.is_base_record() {
            return;
        }
        if entry.valid_fixup == Some(false) {
            self.rejected += 1;
            return;
        }
        let mut names = Vec::new();
        let mut dos_names = Vec::new();
        let mut content = Content::Unavailable;
        let mut streams = 0;
        let mut damaged = false;
        let mut named_streams = 0;
        for part in std::iter::once(entry).chain(extensions) {
            if !part.header.is_valid() || part.valid_fixup == Some(false) {
                damaged = true;
                continue;
            }
            for attr in part.iter_attributes() {
                let attr = match attr {
                    Ok(attr) => attr,
                    Err(_) => {
                        damaged = true;
                        continue;
                    }
                };
                if let Some(name) = attr.data.as_file_name() {
                    let value = Name {
                        parent: (name.parent.entry, name.parent.sequence),
                        text: name.name.to_utf8_string(),
                    };
                    if name.namespace == FileNamespace::DOS {
                        dos_names.push(value);
                    } else {
                        names.push(value);
                    }
                }
                if attr.header.type_code == MftAttributeType::DATA {
                    if !attr.header.name.to_utf8_string().is_empty() {
                        named_streams += 1;
                        continue;
                    }
                    if attr.header.is_continuation() {
                        continue;
                    }
                    streams += 1;
                    if attr.header.form_code == 0 && (attr.header.data_flags.bits() & 0x40ff) != 0 {
                        damaged = true;
                        continue;
                    }
                    content = match attr.data.as_data() {
                        Some(data) if data.data().len() as u64 == attr.header.value_size() => {
                            Content::Resident(data.data().to_vec())
                        }
                        _ if attr.header.form_code != 0 => {
                            Content::NonResident(attr.header.value_size())
                        }
                        _ => {
                            damaged = true;
                            Content::Unavailable
                        }
                    };
                }
                // Small directories keep their $I30 index inside the MFT. Large directories
                // need a separately collected INDEX_ALLOCATION; no disk clusters are read here.
                if entry.is_dir()
                    && attr
                        .header
                        .name
                        .to_utf8_string()
                        .eq_ignore_ascii_case("$I30")
                    && let Some(index) = attr.data.as_index_root()
                    && index.attribute_type == 0x30
                {
                    for indexed in &index.index_entries.index_entries {
                        let n = &indexed.fname_info;
                        if n.namespace == FileNamespace::DOS {
                            continue;
                        }
                        self.indexes.push(IndexedName {
                            name: Name { parent: (n.parent.entry, n.parent.sequence), text: n.name.to_utf8_string() },
                            reference: Some((indexed.mft_reference.entry, indexed.mft_reference.sequence)),
                            directory: n.flags.contains(ys_ntfs_parser::mft::attribute::FileAttributeFlags::FILE_ATTRIBUTE_IS_DIRECTORY),
                            size: n.logical_size,
                            slack: false,
                            source: format!("$MFT/$I30-INDEX_ROOT/{}", part.header.record_number),
                            offset: attr.header.start_offset,
                        });
                    }
                }
            }
        }
        if names.is_empty() {
            names = dos_names;
        }
        names.sort();
        names.dedup();
        let mut notes = Vec::new();
        if damaged || streams > 1 {
            // A partially parsed or conflicting attribute set cannot establish complete content.
            content = Content::Unavailable;
            notes.push("Damaged or conflicting attributes; content not recovered");
        }
        if named_streams > 0 {
            notes.push(
                "Named alternate streams are not materialized; use --extract-resident-streams",
            );
        }
        self.nodes.insert(
            id,
            Node {
                sequence: entry.header.sequence,
                directory: entry.is_dir(),
                deleted: !entry.is_allocated(),
                names,
                content,
                notes: notes.join("; "),
            },
        );
    }

    pub(crate) fn add_i30(&mut self, path: &Path, source: &str) -> io::Result<()> {
        crate::i30::visit_records(path, |base, record| {
            crate::interrupt::check()?;
            if !record.valid_fixup || !record.problems.is_empty() {
                self.rejected += 1;
                return Ok(());
            }
            for entry in record.entries.iter().chain(&record.slack) {
                if entry.copy_of_entry_in_use || entry.namespace == FileNamespace::DOS {
                    continue;
                }
                self.indexes.push(indexed_name(entry, source, base));
            }
            Ok(())
        })
    }

    pub(crate) fn write(
        self,
        destination: &Path,
        source: &Path,
        i30_files: &[PathBuf],
        clobber: bool,
    ) -> io::Result<Summary> {
        let mut this = self;
        let mut input_errors = Vec::new();
        for file in i30_files {
            let shown =
                crate::auto_parse::relative_path(source.parent().unwrap_or(Path::new(".")), file)
                    .to_string_lossy()
                    .into_owned();
            if let Err(error) = this.add_i30(file, &shown) {
                if error.kind() == io::ErrorKind::Interrupted {
                    return Err(error);
                }
                this.rejected += 1;
                input_errors.push((shown, error.to_string()));
            }
        }
        let mut disk = Disk::new(destination, clobber)?;
        let mut manifest = csv::WriterBuilder::new().has_headers(false).from_writer(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(destination.join(PARTIAL_INVENTORY))?,
        );
        manifest.write_record(INVENTORY_HEADER)?;
        let mut paths = Paths::default();
        let mut summary = Summary {
            rejected: this.rejected,
            ..Summary::default()
        };
        let (drive, _) = volume_hint(source);
        let volume = destination
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let source_name = source.file_name().unwrap_or_default().to_string_lossy();
        // Index-only directory references can supply otherwise missing ancestors. Never replace
        // a current MFT generation with a historical index reference.
        let mut indexed_directories = BTreeMap::new();
        for entry in &this.indexes {
            if entry.directory
                && let Some(reference) = entry.reference
            {
                indexed_directories
                    .entry(reference)
                    .or_insert_with(|| entry.name.clone());
            }
        }
        let mut inventory =
            |row: Row<'_>| -> io::Result<()> { manifest.serialize(row).map_err(io::Error::other) };
        for (source, message) in &input_errors {
            summary
                .warnings
                .push(format!("Skipped reconstruction index {source}: {message}"));
            inventory(Row {
                volume: &volume,
                drive,
                source,
                entry: None,
                sequence: None,
                parent_entry: None,
                parent_sequence: None,
                original: "",
                output: "",
                directory: false,
                deletion: "unconfirmed",
                data: "i30_input_error",
                logical_size: None,
                recovered: 0,
                path_status: "unavailable",
                from_slack: false,
                offset: None,
                notes: message,
                file: "",
            })?;
        }
        for (&id, node) in &this.nodes {
            crate::interrupt::check()?;
            if id == 5 {
                continue;
            }
            if node.names.is_empty() {
                summary.unnamed += 1;
                let location = if node.directory {
                    paths.directory(
                        (id, node.sequence),
                        &this.nodes,
                        &indexed_directories,
                        &mut disk,
                    )?
                } else {
                    let parent = disk.unresolved_parent()?;
                    disk.child(
                        &parent,
                        &format!("MFT-{id}-{}", node.sequence),
                        if node.deleted { "-deleted" } else { "" },
                        false,
                    )?
                };
                let (status, size, bytes) = if node.directory {
                    ("directory", None, &[][..])
                } else {
                    content(&node.content)
                };
                if !node.directory {
                    disk.file(&location.path, bytes)?;
                    summary.files += 1;
                    if !bytes.is_empty() {
                        summary.resident += 1;
                    }
                    summary.bytes += bytes.len() as u64;
                }
                inventory(Row {
                    volume: &volume,
                    drive,
                    source: &source_name,
                    entry: Some(id),
                    sequence: Some(node.sequence),
                    parent_entry: None,
                    parent_sequence: None,
                    original: "",
                    output: &portable(&location.path),
                    directory: node.directory,
                    deletion: deletion(node.deleted),
                    data: status,
                    logical_size: size,
                    recovered: bytes.len() as u64,
                    path_status: "unnamed",
                    from_slack: false,
                    offset: None,
                    notes: "No surviving file name; generated identifier in unresolved directory",
                    file: "",
                })?;
                continue;
            }
            for name in &node.names {
                let (location, data_status, logical_size, recovered) = if node.directory {
                    let location = if node.names.first() == Some(name) {
                        paths.directory(
                            (id, node.sequence),
                            &this.nodes,
                            &indexed_directories,
                            &mut disk,
                        )?
                    } else {
                        let parent = paths.directory(
                            name.parent,
                            &this.nodes,
                            &indexed_directories,
                            &mut disk,
                        )?;
                        disk.child(
                            &parent,
                            &name.text,
                            if node.deleted { "-deleted" } else { "" },
                            true,
                        )?
                    };
                    (location, "directory", None, 0)
                } else {
                    let parent = paths.directory(
                        name.parent,
                        &this.nodes,
                        &indexed_directories,
                        &mut disk,
                    )?;
                    let location = disk.child(
                        &parent,
                        &name.text,
                        if node.deleted { "-deleted" } else { "" },
                        false,
                    )?;
                    let (status, size, bytes) = content(&node.content);
                    disk.file(&location.path, bytes)?;
                    summary.files += 1;
                    if !bytes.is_empty() {
                        summary.resident += 1;
                    }
                    summary.bytes += bytes.len() as u64;
                    (location, status, size, bytes.len() as u64)
                };
                inventory(Row {
                    volume: &volume,
                    drive,
                    source: &source_name,
                    entry: Some(id),
                    sequence: Some(node.sequence),
                    parent_entry: Some(name.parent.0),
                    parent_sequence: Some(name.parent.1),
                    original: &location.original,
                    output: &portable(&location.path),
                    directory: node.directory,
                    deletion: deletion(node.deleted),
                    data: data_status,
                    logical_size,
                    recovered,
                    path_status: location.status,
                    from_slack: false,
                    offset: None,
                    notes: &node.notes,
                    file: &name.text,
                })?;
            }
        }
        let mut seen = HashSet::new();
        for entry in &this.indexes {
            crate::interrupt::check()?;
            // Duplicate current names add provenance, not another file. In particular a slack
            // copy of a current file must never acquire a spurious '-deleted' counterpart.
            let matched = entry
                .reference
                .and_then(|r| this.nodes.get(&r.0).filter(|n| n.matches(r.1)))
                .is_some_and(|n| n.directory == entry.directory && n.names.contains(&entry.name));
            if !seen.insert((entry.reference, entry.name.clone(), entry.slack)) {
                continue;
            }
            let parent = paths.directory(
                entry.name.parent,
                &this.nodes,
                &indexed_directories,
                &mut disk,
            )?;
            let location = if matched {
                None
            } else {
                let path = if entry.directory {
                    if let Some(r) = entry.reference
                        && !this
                            .nodes
                            .get(&r.0)
                            .is_some_and(|n| n.directory && n.matches(r.1))
                        && indexed_directories.get(&r) == Some(&entry.name)
                    {
                        paths.directory(r, &this.nodes, &indexed_directories, &mut disk)?
                    } else {
                        disk.child(&parent, &entry.name.text, "-i30", true)?
                    }
                } else {
                    let location = disk.child(&parent, &entry.name.text, "-i30", false)?;
                    disk.file(&location.path, &[])?;
                    summary.files += 1;
                    location
                };
                summary.i30 += 1;
                Some(path)
            };
            let original = join_original(&parent.original, &entry.name.text);
            inventory(Row {
                volume: &volume,
                drive,
                source: &entry.source,
                entry: entry.reference.map(|r| r.0),
                sequence: entry.reference.map(|r| r.1),
                parent_entry: (entry.name.parent.0 != 0).then_some(entry.name.parent.0),
                parent_sequence: (entry.name.parent.1 != 0).then_some(entry.name.parent.1),
                original: &original,
                output: &location
                    .as_ref()
                    .map_or_else(String::new, |p| portable(&p.path)),
                directory: entry.directory,
                deletion: "unconfirmed",
                data: if matched {
                    "mft_name_already_present"
                } else {
                    "i30_placeholder"
                },
                logical_size: Some(entry.size),
                recovered: 0,
                path_status: location.as_ref().map_or(parent.status, |p| p.status),
                from_slack: entry.slack,
                offset: Some(entry.offset),
                notes: "I30 names can reflect renames or index rebalancing; deletion and historical content are not established",
                file: &entry.name.text,
            })?;
        }
        manifest.flush()?;
        drop(manifest);
        fs::rename(
            destination.join(PARTIAL_INVENTORY),
            destination.join(INVENTORY),
        )?;
        summary.directories = disk.directories;
        Ok(summary)
    }
}

fn indexed_name(entry: &IndexEntry, source: &str, base: u64) -> IndexedName {
    let parent = entry
        .parent
        .map(|r| (r.entry, r.sequence))
        .unwrap_or((0, 0));
    IndexedName {
        name: Name {
            parent,
            text: entry.name.clone(),
        },
        reference: entry.file_reference.map(|r| (r.entry, r.sequence)),
        directory: entry.file_flags.contains(
            ys_ntfs_parser::mft::attribute::FileAttributeFlags::FILE_ATTRIBUTE_IS_DIRECTORY,
        ),
        size: entry.real_size,
        slack: entry.from_slack,
        source: source.to_string(),
        offset: base.saturating_add(entry.offset),
    }
}

fn content(data: &Content) -> (&'static str, Option<u64>, &[u8]) {
    match data {
        Content::Resident(bytes) => (
            if bytes.is_empty() {
                "resident_empty"
            } else {
                "resident_recovered"
            },
            Some(bytes.len() as u64),
            bytes,
        ),
        Content::NonResident(size) => ("non_resident_placeholder", Some(*size), &[]),
        Content::Unavailable => ("unavailable_placeholder", None, &[]),
    }
}

fn deletion(deleted: bool) -> &'static str {
    if deleted { "deleted" } else { "allocated" }
}

#[derive(Default, Debug)]
pub(crate) struct Summary {
    pub(crate) files: u64,
    pub(crate) directories: u64,
    pub(crate) resident: u64,
    pub(crate) bytes: u64,
    pub(crate) i30: u64,
    pub(crate) rejected: u64,
    pub(crate) unnamed: u64,
    pub(crate) warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Row<'a> {
    volume: &'a str,
    drive: Option<char>,
    source: &'a str,
    entry: Option<u64>,
    sequence: Option<u16>,
    parent_entry: Option<u64>,
    parent_sequence: Option<u16>,
    original: &'a str,
    output: &'a str,
    directory: bool,
    deletion: &'a str,
    data: &'a str,
    logical_size: Option<u64>,
    recovered: u64,
    path_status: &'a str,
    from_slack: bool,
    offset: Option<u64>,
    notes: &'a str,
    file: &'a str,
}

#[derive(Clone, Default)]
struct Location {
    path: PathBuf,
    original: String,
    status: &'static str,
}
#[derive(Default)]
struct Paths {
    directories: HashMap<Reference, Location>,
}

impl Paths {
    fn directory(
        &mut self,
        reference: Reference,
        nodes: &BTreeMap<u64, Node>,
        indexes: &BTreeMap<Reference, Name>,
        disk: &mut Disk,
    ) -> io::Result<Location> {
        let canonical = |r: Reference| {
            nodes
                .get(&r.0)
                .filter(|n| n.directory && n.matches(r.1))
                .map_or(r, |n| (r.0, n.sequence))
        };
        let reference = canonical(reference);
        if let Some(path) = self.directories.get(&reference) {
            return Ok(path.clone());
        }
        let mut chain = Vec::new();
        let mut seen = HashSet::new();
        let mut at = reference;
        let mut base = loop {
            at = canonical(at);
            if let Some(found) = self.directories.get(&at) {
                break found.clone();
            }
            if at.0 == 5
                && nodes
                    .get(&5)
                    .is_some_and(|n| n.directory && n.matches(at.1))
            {
                break Location {
                    status: "resolved",
                    ..Location::default()
                };
            }
            if !seen.insert(at) || chain.len() >= MAX_DEPTH || at.1 == 0 {
                let fallback = disk.unresolved(
                    at,
                    nodes
                        .get(&at.0)
                        .is_some_and(|n| n.directory && n.matches(at.1) && n.deleted),
                )?;
                self.directories.insert(at, fallback.clone());
                break fallback;
            }
            if let Some(node) = nodes.get(&at.0).filter(|n| n.directory && n.matches(at.1))
                && let Some(name) = node.names.first()
            {
                chain.push((
                    at,
                    name.clone(),
                    if node.deleted { "-deleted" } else { "" },
                    false,
                ));
                at = name.parent;
            } else if let Some(name) = indexes.get(&at) {
                chain.push((at, name.clone(), "-i30", true));
                at = name.parent;
            } else {
                let fallback = disk.unresolved(
                    at,
                    nodes
                        .get(&at.0)
                        .is_some_and(|n| n.directory && n.matches(at.1) && n.deleted),
                )?;
                self.directories.insert(at, fallback.clone());
                break fallback;
            }
        };
        for (reference, name, suffix, i30) in chain.into_iter().rev() {
            if let Some(cached) = self.directories.get(&reference) {
                base = cached.clone();
                continue;
            }
            base = disk.child(&base, &name.text, suffix, true)?;
            if i30 && base.status == "resolved" {
                base.status = "i30_parent";
            }
            self.directories.insert(reference, base.clone());
        }
        self.directories.insert(reference, base.clone());
        Ok(base)
    }
}

struct Disk {
    root: PathBuf,
    used: HashSet<String>,
    directories: u64,
}
impl Disk {
    fn new(root: &Path, clobber: bool) -> io::Result<Self> {
        // Never write through a populated tree. Explicit clobber can replace only a complete,
        // inventoried Juntei tree with no untracked files, links or special filesystem objects.
        if fs::symlink_metadata(root).is_ok() {
            if !clobber {
                return Err(existing_tree());
            }
            owned_tree(root)?;
            fs::remove_dir_all(root)?;
        }
        fs::create_dir(root)?;
        Ok(Self {
            root: fs::canonicalize(root)?,
            used: HashSet::from([
                INVENTORY.to_lowercase(),
                PARTIAL_INVENTORY.to_lowercase(),
                UNRESOLVED.to_lowercase(),
            ]),
            directories: 0,
        })
    }
    fn child(
        &mut self,
        parent: &Location,
        name: &str,
        suffix: &str,
        directory: bool,
    ) -> io::Result<Location> {
        let original = name;
        let name = component(name);
        let mut stem = format!("{name}{suffix}");
        let mut parent = parent.clone();
        if self.root.as_os_str().len() + parent.path.as_os_str().len() + stem.len()
            > MAX_HOST_PATH_BYTES
        {
            let full = join_original(&parent.original, original);
            let hash = full.bytes().fold(0xcbf29ce484222325u64, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
            });
            let bucket = self.root.join(UNRESOLVED);
            if !bucket.exists() {
                fs::create_dir(&bucket)?;
                self.directories += 1;
            }
            parent.path = PathBuf::from(UNRESOLVED);
            parent.status = "unresolved";
            stem.push_str(&format!("-path-{hash:016x}"));
        }
        let mut path = parent.path.join(&stem);
        let mut collision = 1u64;
        loop {
            let key = portable(&path).to_lowercase();
            // The destination can have additional equivalences, such as Unicode normalization
            // on macOS. Never assume Rust string folding fully describes the host filesystem.
            let exists = match fs::symlink_metadata(self.root.join(&path)) {
                Ok(_) => true,
                Err(e) if e.kind() == io::ErrorKind::NotFound => false,
                Err(e) => return Err(e),
            };
            if !exists && self.used.insert(key) {
                break;
            }
            collision += 1;
            path = parent.path.join(format!("{stem}-collision-{collision}"));
        }
        if directory {
            fs::create_dir(self.root.join(&path))?;
            self.directories += 1;
        }
        Ok(Location {
            path,
            original: join_original(&parent.original, original),
            status: parent.status,
        })
    }
    fn file(&self, relative: &Path, content: &[u8]) -> io::Result<()> {
        let mut opts = OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut out = opts.open(self.root.join(relative))?;
        out.write_all(content)
    }
    fn unresolved_parent(&mut self) -> io::Result<Location> {
        let bucket = self.root.join(UNRESOLVED);
        if !bucket.exists() {
            fs::create_dir(&bucket)?;
            self.directories += 1;
        }
        Ok(Location {
            path: PathBuf::from(UNRESOLVED),
            original: "[Unresolved]".into(),
            status: "unresolved",
        })
    }
    fn unresolved(&mut self, reference: Reference, deleted: bool) -> io::Result<Location> {
        let parent = self.unresolved_parent()?;
        self.child(
            &parent,
            &format!("MFT-{}-{}", reference.0, reference.1),
            if deleted { "-deleted" } else { "" },
            true,
        )
    }
}

fn join_original(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.into()
    } else {
        format!("{parent}/{name}")
    }
}
fn portable(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Reversible escaping for illegal Windows components; long names gain a stable hash. Percent
/// itself is escaped, so '%2F' never aliases an escaped slash. The inventory retains raw names.
fn component(name: &str) -> String {
    let reserved = name
        .split('.')
        .next()
        .unwrap_or("")
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    let reserved = matches!(
        reserved.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "CONIN$"
            | "CONOUT$"
            | "COM¹"
            | "COM²"
            | "COM³"
            | "LPT¹"
            | "LPT²"
            | "LPT³"
    ) || (reserved.len() == 4
        && (reserved.starts_with("COM") || reserved.starts_with("LPT"))
        && matches!(reserved.as_bytes()[3], b'1'..=b'9'));
    let mut encoded = String::new();
    for (at, c) in name.char_indices() {
        if c.is_control()
            || matches!(
                c,
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '%'
            )
            || (at == 0 && reserved)
            || (c == '.' && matches!(name, "." | ".."))
            || (at + c.len_utf8() == name.len() && matches!(c, '.' | ' '))
        {
            for byte in c.to_string().bytes() {
                encoded.push_str(&format!("%{byte:02X}"));
            }
        } else {
            encoded.push(c);
        }
    }
    if encoded.is_empty() {
        encoded = "%00".into();
    }
    if encoded.len() > 160 {
        let hash = name.bytes().fold(0xcbf29ce484222325u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
        });
        let mut end = 140;
        while !encoded.is_char_boundary(end) {
            end -= 1;
        }
        encoded.truncate(end);
        encoded.push_str(&format!("-name-{hash:016x}"));
    }
    encoded
}

fn existing_tree() -> io::Error {
    io::Error::new(
        io::ErrorKind::AlreadyExists,
        "Filesystem reconstruction requires a new directory or --clobber with a complete matching Juntei inventory and no unrelated files or links",
    )
}

fn reparse(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    metadata.file_type().is_symlink()
}

/// A tree is disposable only when every object is covered by the completed inventory. Reject
/// unexpected files, missing files, links (including Windows junctions), malformed manifests and
/// special objects. Walk with symlink_metadata; never traverse an untrusted link.
fn owned_tree(root: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(root)?;
    if reparse(&metadata) || !metadata.is_dir() {
        return Err(existing_tree());
    }
    let manifest = root.join(INVENTORY);
    let metadata = fs::symlink_metadata(&manifest).map_err(|_| existing_tree())?;
    if reparse(&metadata) || !metadata.is_file() {
        return Err(existing_tree());
    }
    let mut reader = csv::Reader::from_path(&manifest)?;
    if reader
        .headers()?
        .iter()
        .ne(INVENTORY_HEADER.iter().copied())
    {
        return Err(existing_tree());
    }
    let mut expected = BTreeMap::<PathBuf, bool>::from([(PathBuf::from(INVENTORY), false)]);
    for record in reader.records() {
        crate::interrupt::check()?;
        let row = record?;
        let path = row.get(8).ok_or_else(existing_tree)?;
        if path.is_empty() {
            continue;
        }
        let path = Path::new(path);
        if path.as_os_str().is_empty()
            || path.to_string_lossy().contains('\\')
            || !path
                .components()
                .all(|c| matches!(c, std::path::Component::Normal(_)))
        {
            return Err(existing_tree());
        }
        let directory = match row.get(9) {
            Some("true") => true,
            Some("false") => false,
            _ => return Err(existing_tree()),
        };
        if expected
            .insert(path.to_path_buf(), directory)
            .is_some_and(|old| old != directory)
        {
            return Err(existing_tree());
        }
        for parent in path
            .ancestors()
            .skip(1)
            .filter(|p| !p.as_os_str().is_empty())
        {
            if expected.insert(parent.to_path_buf(), true) == Some(false) {
                return Err(existing_tree());
            }
        }
    }
    let mut pending = vec![root.to_path_buf()];
    let mut found = BTreeSet::new();
    while let Some(dir) = pending.pop() {
        crate::interrupt::check()?;
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            let metadata = fs::symlink_metadata(&path)?;
            if reparse(&metadata) || (!metadata.is_dir() && !metadata.is_file()) {
                return Err(existing_tree());
            }
            let relative = path.strip_prefix(root).map_err(|_| existing_tree())?;
            if expected.get(relative) != Some(&metadata.is_dir()) {
                return Err(existing_tree());
            }
            found.insert(relative.to_path_buf());
            if metadata.is_dir() {
                pending.push(path);
            }
        }
    }
    if found.len() != expected.len() {
        return Err(existing_tree());
    }
    Ok(())
}

/// Validate before the normal CSV writer is opened. Source aliases/containment are forbidden.
/// Clobber recognizes only complete inventoried trees, and repeats that check before removal.
pub(crate) fn validate(
    destination: &Path,
    sources: &[PathBuf],
    other_outputs: &[&Path],
    clobber: bool,
) -> io::Result<()> {
    let resolved = crate::auto_parse::absolute_candidate(destination)?;
    for source in sources
        .iter()
        .map(PathBuf::as_path)
        .chain(other_outputs.iter().copied())
    {
        let source = crate::auto_parse::absolute_candidate(source)?;
        if source.starts_with(&resolved) || resolved.starts_with(&source) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Filesystem reconstruction must not contain or overlap input or other output paths",
            ));
        }
    }
    if fs::symlink_metadata(destination).is_ok() {
        if !clobber {
            return Err(existing_tree());
        }
        owned_tree(destination)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(parent: Reference, text: &str) -> Name {
        Name {
            parent,
            text: text.into(),
        }
    }
    fn node(
        sequence: u16,
        directory: bool,
        deleted: bool,
        names: Vec<Name>,
        content: Content,
    ) -> Node {
        Node {
            sequence,
            directory,
            deleted,
            names,
            content,
            notes: String::new(),
        }
    }
    fn tree() -> Builder {
        let mut b = Builder::default();
        b.nodes
            .insert(5, node(5, true, false, vec![], Content::Unavailable));
        b
    }
    fn rows(root: &Path) -> Vec<HashMap<String, String>> {
        let mut csv = csv::Reader::from_path(root.join(INVENTORY)).unwrap();
        let fields = csv.headers().unwrap().clone();
        csv.records()
            .map(|r| {
                fields
                    .iter()
                    .zip(r.unwrap().iter())
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect()
            })
            .collect()
    }
    fn write(b: Builder, root: &Path) -> Summary {
        b.write(root, Path::new("collection/C/$MFT"), &[], false)
            .unwrap()
    }

    #[test]
    fn recovers_binary_and_empty_resident_data_but_never_nonresident_contents() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("C");
        let mut b = tree();
        b.nodes.insert(
            10,
            node(
                1,
                true,
                false,
                vec![name((5, 5), "Users")],
                Content::Unavailable,
            ),
        );
        b.nodes.insert(
            11,
            node(
                2,
                false,
                false,
                vec![name((10, 1), "binary.dat")],
                Content::Resident(vec![0, 255, 1, 0]),
            ),
        );
        b.nodes.insert(
            12,
            node(
                3,
                false,
                false,
                vec![name((10, 1), "large.bin")],
                Content::NonResident(9_000_000_000),
            ),
        );
        b.nodes.insert(
            13,
            node(
                4,
                false,
                false,
                vec![name((10, 1), "empty.txt")],
                Content::Resident(vec![]),
            ),
        );
        let summary = write(b, &root);
        assert_eq!(
            fs::read(root.join("Users/binary.dat")).unwrap(),
            [0, 255, 1, 0]
        );
        assert_eq!(fs::metadata(root.join("Users/large.bin")).unwrap().len(), 0);
        assert_eq!((summary.files, summary.resident, summary.bytes), (3, 1, 4));
        let records = rows(&root);
        let large = records.iter().find(|r| r["EntryNum"] == "12").unwrap();
        assert_eq!(large["DataStatus"], "non_resident_placeholder");
        assert_eq!(large["LogicalSize"], "9000000000");
        assert_eq!(
            records.iter().find(|r| r["EntryNum"] == "13").unwrap()["DataStatus"],
            "resident_empty"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(root.join("Users/binary.dat"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn deleted_parent_sequence_increment_resolves_to_one_deleted_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("C");
        let mut b = tree();
        b.nodes.insert(
            10,
            node(
                8,
                true,
                true,
                vec![name((5, 5), "Old")],
                Content::Unavailable,
            ),
        );
        b.nodes.insert(
            11,
            node(
                2,
                false,
                true,
                vec![name((10, 7), "note.txt")],
                Content::Resident(b"deleted content".to_vec()),
            ),
        );
        b.nodes.insert(
            12,
            node(
                3,
                false,
                false,
                vec![name((10, 8), "other.txt")],
                Content::NonResident(10),
            ),
        );
        write(b, &root);
        assert_eq!(
            fs::read(root.join("Old-deleted/note.txt-deleted")).unwrap(),
            b"deleted content"
        );
        assert!(root.join("Old-deleted/other.txt").is_file());
        assert!(!root.join("Old-deleted-collision-2").exists());
        assert_eq!(
            rows(&root).iter().find(|r| r["EntryNum"] == "11").unwrap()["DeletionStatus"],
            "deleted"
        );
    }

    #[test]
    fn reused_parent_never_receives_a_previous_generations_child() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("C");
        let mut b = tree();
        b.nodes.insert(
            10,
            node(
                8,
                true,
                false,
                vec![name((5, 5), "NewOwner")],
                Content::Unavailable,
            ),
        );
        b.nodes.insert(
            11,
            node(
                2,
                false,
                true,
                vec![name((10, 6), "old.txt")],
                Content::Resident(b"old".to_vec()),
            ),
        );
        write(b, &root);
        assert!(!root.join("NewOwner/old.txt-deleted").exists());
        assert_eq!(
            fs::read(root.join(UNRESOLVED).join("MFT-10-6/old.txt-deleted")).unwrap(),
            b"old"
        );
    }

    #[test]
    fn i30_adds_historical_names_without_borrowing_resident_bytes_or_asserting_deletion() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("C");
        let mut b = tree();
        b.nodes.insert(
            30,
            node(
                4,
                false,
                false,
                vec![name((5, 5), "current.txt")],
                Content::Resident(b"new generation".to_vec()),
            ),
        );
        for (text, seq, slack) in [
            ("older.txt", 2, true),
            ("renamed.txt", 4, false),
            ("current.txt", 4, true),
        ] {
            b.indexes.push(IndexedName {
                name: name((5, 5), text),
                reference: Some((30, seq)),
                directory: false,
                size: 999,
                slack,
                source: "$I30".into(),
                offset: 1024,
            });
        }
        write(b, &root);
        assert_eq!(
            fs::read(root.join("current.txt")).unwrap(),
            b"new generation"
        );
        assert_eq!(fs::metadata(root.join("older.txt-i30")).unwrap().len(), 0);
        assert_eq!(fs::metadata(root.join("renamed.txt-i30")).unwrap().len(), 0);
        assert!(!root.join("current.txt-deleted").exists());
        assert!(!root.join("current.txt-i30").exists());
        for row in rows(&root).iter().filter(|r| r["Source"] == "$I30") {
            assert_eq!(row["RecoveredBytes"], "0");
            assert_eq!(row["DeletionStatus"], "unconfirmed");
        }
    }

    #[test]
    fn i30_can_supply_a_missing_directory_but_not_change_a_current_directory_name() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("C");
        let mut b = tree();
        b.nodes.insert(
            21,
            node(
                2,
                false,
                false,
                vec![name((20, 1), "child.txt")],
                Content::Resident(b"child".to_vec()),
            ),
        );
        b.nodes.insert(
            30,
            node(
                1,
                true,
                false,
                vec![name((5, 5), "Current")],
                Content::Unavailable,
            ),
        );
        for (id, text) in [(20, "Missing"), (30, "OlderName")] {
            b.indexes.push(IndexedName {
                name: name((5, 5), text),
                reference: Some((id, 1)),
                directory: true,
                size: 0,
                slack: true,
                source: "$I30".into(),
                offset: id,
            });
        }
        write(b, &root);
        assert_eq!(
            fs::read(root.join("Missing-i30/child.txt")).unwrap(),
            b"child"
        );
        assert!(root.join("Current").is_dir());
        assert!(root.join("OlderName-i30").is_dir());
        assert_eq!(
            rows(&root).iter().find(|r| r["EntryNum"] == "21").unwrap()["PathStatus"],
            "i30_parent"
        );
    }

    #[test]
    fn path_components_cannot_escape_and_collisions_preserve_both_contents() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("C");
        let mut b = tree();
        let names = [
            "../outside",
            r"..\outside",
            ".",
            "..",
            "CON",
            "con.txt",
            "NUL",
            "COM1.txt",
            "trail.",
            "trail ",
            "A.txt",
            "a.txt",
            "victim.txt",
            "victim.txt-deleted",
            INVENTORY,
            PARTIAL_INVENTORY,
            UNRESOLVED,
            "%2F",
            "/",
            "é中",
        ];
        for (i, text) in names.iter().enumerate() {
            b.nodes.insert(
                20 + i as u64,
                node(
                    1,
                    false,
                    i == 12,
                    vec![name((5, 5), text)],
                    Content::Resident(text.as_bytes().to_vec()),
                ),
            );
        }
        write(b, &root);
        assert!(!tmp.path().join("outside").exists());
        let records = rows(&root);
        let mut outputs = HashSet::new();
        for row in &records {
            assert!(outputs.insert(row["OutputPath"].to_lowercase()));
            assert_eq!(
                fs::read(root.join(&row["OutputPath"])).unwrap(),
                row["OriginalPath"].as_bytes()
            );
        }
        assert_eq!(records.len(), names.len());
        assert!(root.join("victim.txt-deleted-collision-2").exists());
    }

    #[test]
    fn cyclic_parent_graph_terminates_in_an_unresolved_bucket() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("C");
        let mut b = tree();
        b.nodes.insert(
            10,
            node(
                1,
                true,
                false,
                vec![name((11, 1), "One")],
                Content::Unavailable,
            ),
        );
        b.nodes.insert(
            11,
            node(
                1,
                true,
                false,
                vec![name((10, 1), "Two")],
                Content::Unavailable,
            ),
        );
        b.nodes.insert(
            12,
            node(
                1,
                false,
                false,
                vec![name((10, 1), "child")],
                Content::Resident(b"ok".to_vec()),
            ),
        );
        write(b, &root);
        let records = rows(&root);
        assert!(records.iter().all(|r| r["PathStatus"] == "unresolved"));
        assert_eq!(
            fs::read(
                root.join(&records.iter().find(|r| r["EntryNum"] == "12").unwrap()["OutputPath"])
            )
            .unwrap(),
            b"ok"
        );
    }

    #[test]
    fn unrepresentable_names_keep_the_original_in_the_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("C");
        let mut b = tree();
        let long = "中".repeat(250);
        b.nodes.insert(
            30,
            node(
                1,
                false,
                false,
                vec![name((5, 5), &long)],
                Content::Resident(vec![1]),
            ),
        );
        write(b, &root);
        let records = rows(&root);
        assert_eq!(records[0]["OriginalPath"], long);
        assert!(records[0]["OutputPath"].len() < 200);
        assert_eq!(fs::read(root.join(&records[0]["OutputPath"])).unwrap(), [1]);
    }

    #[test]
    fn source_aliases_and_preexisting_trees_are_refused_before_writes() {
        let tmp = tempfile::tempdir().unwrap();
        let mft = tmp.path().join("$MFT");
        fs::write(&mft, b"evidence").unwrap();
        assert!(validate(tmp.path(), std::slice::from_ref(&mft), &[], false).is_err());
        assert!(validate(&mft, std::slice::from_ref(&mft), &[], false).is_err());
        let fresh = tmp.path().join("new");
        assert!(validate(&fresh, std::slice::from_ref(&mft), &[], false).is_ok());
        assert!(validate(&fresh, &[], &[&fresh.join("output.csv")], false).is_err());
        fs::create_dir(&fresh).unwrap();
        fs::write(fresh.join("keep"), b"keep").unwrap();
        assert!(Builder::default().write(&fresh, &mft, &[], false).is_err());
        assert_eq!(fs::read(fresh.join("keep")).unwrap(), b"keep");
    }

    #[test]
    #[cfg(unix)]
    fn destination_links_are_not_followed() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("evidence");
        fs::create_dir(&source).unwrap();
        let mft = source.join("$MFT");
        fs::write(&mft, b"private").unwrap();
        let link = tmp.path().join("link");
        std::os::unix::fs::symlink(&source, &link).unwrap();
        assert!(validate(&link, std::slice::from_ref(&mft), &[], false).is_err());
        assert!(validate(&link.join("new"), &[], &[&mft], false).is_ok());
        let nested_input = link.join("new/$MFT");
        assert!(validate(&source.join("new"), &[nested_input], &[], false).is_err());
        assert_eq!(fs::read(&mft).unwrap(), b"private");
    }

    #[test]
    fn volume_letters_come_from_collection_paths_and_indexes_stay_on_their_volume() {
        for path in [
            "host/C/$MFT",
            "host/uploads/ntfs/%5C%5C.%5CC%3A/$MFT",
            "host/uploads/auto/c%3a/Windows/$MFT",
            "host/C$/$MFT",
        ] {
            assert_eq!(volume_hint(Path::new(path)).0, Some('C'));
        }
        assert_eq!(volume_hint(Path::new("host/$MFT")).0, None);
        assert_eq!(volume_hint(Path::new("case/a/another/$MFT")).0, None);
        assert_eq!(volume_hint(Path::new("host/Volume1/$MFT")).0, None);
        assert_ne!(
            volume_hint(Path::new("one/C/$MFT")).1,
            volume_hint(Path::new("two/C/$I30")).1
        );
    }

    #[test]
    fn deep_trees_fall_back_before_the_host_path_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("C");
        let mut b = tree();
        let mut parent = (5, 5);
        for id in 10..60 {
            b.nodes.insert(
                id,
                node(
                    1,
                    true,
                    false,
                    vec![name(parent, &format!("{id}{}", "x".repeat(145)))],
                    Content::Unavailable,
                ),
            );
            parent = (id, 1);
        }
        b.nodes.insert(
            61,
            node(
                1,
                false,
                false,
                vec![name(parent, "deep.txt")],
                Content::Resident(b"deep".to_vec()),
            ),
        );
        write(b, &root);
        let records = rows(&root);
        let deep = records.iter().find(|r| r["EntryNum"] == "61").unwrap();
        assert_eq!(deep["PathStatus"], "unresolved");
        assert_eq!(fs::read(root.join(&deep["OutputPath"])).unwrap(), b"deep");
    }

    #[test]
    fn unnamed_resident_content_is_saved_under_an_explicit_generated_identifier() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("C");
        let mut b = tree();
        b.nodes
            .insert(10, node(2, true, true, vec![], Content::Unavailable));
        b.nodes.insert(
            11,
            node(
                3,
                false,
                true,
                vec![],
                Content::Resident(b"unnamed".to_vec()),
            ),
        );
        b.nodes.insert(
            12,
            node(
                1,
                false,
                false,
                vec![name((10, 1), "child")],
                Content::Resident(b"child".to_vec()),
            ),
        );
        write(b, &root);
        assert_eq!(
            fs::read(root.join(UNRESOLVED).join("MFT-11-3-deleted")).unwrap(),
            b"unnamed"
        );
        assert_eq!(
            fs::read(root.join(UNRESOLVED).join("MFT-10-2-deleted/child")).unwrap(),
            b"child"
        );
        let records = rows(&root);
        assert_eq!(
            records.iter().find(|r| r["EntryNum"] == "11").unwrap()["PathStatus"],
            "unnamed"
        );
    }

    #[test]
    fn explicit_clobber_replaces_only_a_complete_owned_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("C");
        let mut b = tree();
        b.nodes.insert(
            10,
            node(
                1,
                false,
                false,
                vec![name((5, 5), "old.txt")],
                Content::Resident(b"old".to_vec()),
            ),
        );
        write(b, &root);
        assert!(validate(&root, &[], &[], true).is_ok());
        let mut b = tree();
        b.nodes.insert(
            10,
            node(
                1,
                false,
                false,
                vec![name((5, 5), "new.txt")],
                Content::Resident(b"new".to_vec()),
            ),
        );
        b.write(&root, Path::new("C/$MFT"), &[], true).unwrap();
        assert!(!root.join("old.txt").exists());
        assert_eq!(fs::read(root.join("new.txt")).unwrap(), b"new");
        fs::write(root.join("analyst-note.txt"), b"do not erase").unwrap();
        assert!(validate(&root, &[], &[], true).is_err());
        assert!(
            Builder::default()
                .write(&root, Path::new("C/$MFT"), &[], true)
                .is_err()
        );
        assert_eq!(
            fs::read(root.join("analyst-note.txt")).unwrap(),
            b"do not erase"
        );
        assert_eq!(fs::read(root.join("new.txt")).unwrap(), b"new");
    }

    #[test]
    #[cfg(unix)]
    fn a_link_substituted_inside_an_owned_tree_is_never_followed_or_clobbered() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("C");
        let mut b = tree();
        b.nodes.insert(
            10,
            node(
                1,
                true,
                false,
                vec![name((5, 5), "Directory")],
                Content::Unavailable,
            ),
        );
        write(b, &root);
        fs::remove_dir(root.join("Directory")).unwrap();
        let outside = tmp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("keep.txt"), b"evidence").unwrap();
        std::os::unix::fs::symlink(&outside, root.join("Directory")).unwrap();
        assert!(validate(&root, &[], &[], true).is_err());
        assert!(
            Builder::default()
                .write(&root, Path::new("C/$MFT"), &[], true)
                .is_err()
        );
        assert_eq!(fs::read(outside.join("keep.txt")).unwrap(), b"evidence");
    }

    #[test]
    #[cfg(windows)]
    fn the_analysts_windows_drive_is_not_the_evidence_drive() {
        assert_eq!(volume_hint(Path::new(r"D:\Cases\Evidence\$MFT")).0, None);
        assert_eq!(
            volume_hint(Path::new(r"D:\Cases\Evidence\C%3A\$MFT")).0,
            Some('C')
        );
    }

    #[test]
    fn destination_name_collisions_are_detected_even_without_a_string_cache_match() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("C");
        let mut disk = Disk::new(&root, false).unwrap();
        fs::write(root.join("name.txt"), b"existing").unwrap();
        let location = disk
            .child(
                &Location {
                    status: "resolved",
                    ..Location::default()
                },
                "name.txt",
                "",
                false,
            )
            .unwrap();
        assert_eq!(location.path, Path::new("name.txt-collision-2"));
        disk.file(&location.path, b"new").unwrap();
        assert_eq!(fs::read(root.join("name.txt")).unwrap(), b"existing");
        assert_eq!(fs::read(root.join(location.path)).unwrap(), b"new");
    }

    #[test]
    fn empty_reconstruction_still_has_a_parseable_inventory() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("C");
        write(Builder::default(), &root);
        let mut reader = csv::Reader::from_path(root.join(INVENTORY)).unwrap();
        assert_eq!(reader.headers().unwrap().get(3), Some("EntryNum"));
        assert_eq!(reader.records().count(), 0);
    }
}
