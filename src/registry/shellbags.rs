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

//! `parse-registry shellbags`: the folders users opened in Explorer and in open and save
//! dialogs, from the `BagMRU` trees of `UsrClass.dat` and `NTUSER.DAT` hives, read by
//! ys-registry-parser's `shellbags` module, each shell item decoded by ys-lnk-parser (a link's
//! target ID list has the same items), with Hayabusa's input options, progress bar and results
//! summary. One record per item: its full path (the items from the top of its tree down to it),
//! its type and name, the folder's times and NTFS file reference the item recorded, its place in
//! its parent's `MRUListEx`, and the last-write times of its own key and of its parent's key. The
//! summary lists the folders opened last.

use super::persistence::profile_of_classes_path;
use super::{
    Hosts, Opened, Profiles, RecordWriter, choose_logs, files_read, filetime, find_hives,
    hive_type_of_file, input_files, open, profile_of_path, sink,
};
use crate::cli::RegistryShellBagsOptions;
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, error_msg, fmt_num, p, p_label};
use chrono::{DateTime, SecondsFormat, Utc};
use indicatif::ProgressBar;
use registry::ntuser::{classes_sid, profile_name};
use registry::shellbags::{ShellBag, shell_bags};
use serde::Serialize;
use shell::item::{self, Item, Kind};
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Problems and folders listed in the summary.
const MAX_PRINTED: usize = 20;
/// The hives read, as their base blocks name them.
const HIVE_TYPES: [&str; 2] = ["NTUSER.DAT", "USRCLASS.DAT"];

/// Run the command; returns the exit status.
pub fn run(opts: &RegistryShellBagsOptions) -> i32 {
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
    let given_logs: Vec<PathBuf> = opts.log1.iter().chain(&opts.log2).cloned().collect();
    let inputs = input_files(
        opts.filepath.as_ref(),
        opts.directory.as_ref(),
        "UsrClass.dat or NTUSER.DAT hives",
        |d| {
            find_hives(d)
                .into_iter()
                .filter(|p| hive_type_of_file(p).is_some_and(|t| HIVE_TYPES.contains(&t.as_str())))
                .collect()
        },
    )
    .and_then(|files| Ok((files, choose_logs(opts.no_logs, &given_logs)?)))
    .and_then(|(files, logs)| {
        let sink = sink(opts.output.as_ref(), opts.clobber, data_on_stdout, || {
            files_read(&files, &given_logs)
        })?;
        Ok((files, logs, sink))
    });
    let (files, logs, sink) = match inputs {
        Ok(inputs) => inputs,
        Err(msg) => {
            report_error(None, &msg);
            return 1;
        }
    };
    let mut writer = match sink.map(|w| RecordWriter::new(w, opts.output_type(), &CSV_HEADER)) {
        Some(Ok(w)) => Some(w),
        Some(Err(e)) => {
            report_error(None, &format!("Could not write the records: {e}"));
            return 1;
        }
        None => None,
    };
    if !data_on_stdout {
        let total: u64 = files
            .iter()
            .map(|p| p.metadata().map_or(0, |m| m.len()))
            .sum();
        p_label(
            no_color,
            "UsrClass.dat and NTUSER.DAT hives: ",
            &fmt_num(files.len()),
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
        progress::progress_bar(files.len(), no_color)
    });

    // --- parse and write -------------------------------------------------------------------
    let mut hosts = Hosts::new(!opts.no_logs);
    let mut profiles = Profiles::new(!opts.no_logs);
    let mut stats = Stats {
        to_stderr: data_on_stdout,
        ..Stats::default()
    };
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        let parsed = open(path, logs).and_then(|mut opened| {
            if !HIVE_TYPES.contains(&opened.hive_type.as_str()) {
                return Err(if opened.hive_type.is_empty() {
                    "not a UsrClass.dat or NTUSER.DAT hive (neither its base block nor its file name says which)".to_string()
                } else {
                    format!(
                        "a {} hive, not a UsrClass.dat or NTUSER.DAT hive",
                        opened.hive_type
                    )
                });
            }
            opened.host = hosts.of(path, &opened);
            let user = User::of(path, &opened, &mut profiles);
            Ok((opened, user))
        });
        match parsed {
            Ok((opened, user)) => {
                if let Err(e) = stats.hive(&opened, &user, opts.code_page, writer.as_mut()) {
                    report_error(
                        bar.as_ref(),
                        &format!("Could not write the records of {}: {e}", path.display()),
                    );
                    return 1;
                }
            }
            Err(e) => {
                stats.failed += 1;
                if stats.failed <= MAX_PRINTED_ERRORS {
                    report_error(
                        bar.as_ref(),
                        &format!("Could not parse {}: {e}", path.display()),
                    );
                }
            }
        }
        if let Some(b) = &bar {
            b.inc(1);
        }
    }
    if stats.failed > MAX_PRINTED_ERRORS {
        report_error(
            bar.as_ref(),
            &format!(
                "{} more files could not be parsed",
                fmt_num(stats.failed - MAX_PRINTED_ERRORS)
            ),
        );
    }
    if let Some(w) = writer
        && let Err(e) = w.finish()
    {
        report_error(None, &format!("Could not write the records: {e}"));
        return 1;
    }
    if let Some(b) = &bar {
        b.finish_with_message(progress::finished_message(no_color));
    }

    // --- summary ---------------------------------------------------------------------------
    if !data_on_stdout {
        stats.print(no_color);
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
    i32::from(stats.hives == 0)
}

// --- items ------------------------------------------------------------------------------------

/// The user a hive belongs to: its profile folder's name, and its SID (a `UsrClass.dat`'s root
/// key names it, else its system's `ProfileList`).
struct User {
    username: Option<String>,
    sid: Option<String>,
}

impl User {
    fn of(path: &Path, opened: &Opened, profiles: &mut Profiles) -> Self {
        let (username, sid) = if opened.hive_type == "USRCLASS.DAT" {
            (profile_of_classes_path(path), classes_sid(&opened.hive))
        } else {
            (
                profile_of_path(path).or_else(|| profile_name(&opened.hive)),
                None,
            )
        };
        let sid = sid.or_else(|| username.as_deref().and_then(|u| profiles.sid(path, u)));
        User { username, sid }
    }
}

/// An item of a tree, decoded, with its full path.
struct Decoded {
    item: Option<Item>,
    path: String,
    problems: Vec<String>,
    /// The value's bytes, kept when the item isn't fully understood.
    raw: Option<Vec<u8>>,
}

/// What a shell item is, as the records name it.
fn kind_name(kind: &Kind) -> &'static str {
    match kind {
        Kind::RootFolder => "root_folder",
        Kind::Volume => "volume",
        Kind::Directory => "directory",
        Kind::File => "file",
        Kind::Network => "network_location",
        Kind::ControlPanel => "control_panel_item",
        Kind::ControlPanelCategory => "control_panel_category",
        Kind::UsersFilesFolder => "users_files_folder",
        Kind::Uri => "uri",
        Kind::PropertyView => "property_view",
        Kind::CdBurn => "cd_burn",
        Kind::PortableDevice => "portable_device",
        Kind::PortableDeviceFolder => "portable_device_folder",
        Kind::CompressedFolder => "compressed_folder",
        Kind::FtpFolder => "ftp_folder",
        Kind::Other(_) => "unknown",
    }
}

/// The item in a `BagMRU` value, decoded (an item under a ZIP archive's, by its `parent`): its
/// shell item list holds one item, then a size of 0.
fn decode(data: &[u8], parent: Option<&Item>, code_page: u16) -> (Option<Item>, Vec<String>) {
    let items = item::items(data);
    let mut problems = Vec::new();
    let used: usize = items.iter().map(|i| i.len()).sum();
    match data.get(used..) {
        Some([0, 0, rest @ ..]) if rest.iter().all(|&b| b == 0) => {}
        Some([0, 0, rest @ ..]) => problems.push(format!(
            "{} bytes after the end of its shell item list",
            rest.len()
        )),
        _ => problems.push("its shell item list has no end (a size of 0)".to_string()),
    }
    if items.len() > 1 {
        problems.push(format!(
            "{} shell items, not one: only the first is read",
            items.len()
        ));
    }
    let Some(first) = items.first() else {
        problems.push("no shell item".to_string());
        return (None, problems);
    };
    (Some(item::decode_child(first, parent, code_page)), problems)
}

/// Each item of `bags` decoded, its path built from its parent's.
fn decode_all(bags: &[ShellBag], code_page: u16) -> Vec<Decoded> {
    let mut out: Vec<Decoded> = Vec::with_capacity(bags.len());
    for bag in bags {
        let parent = bag.parent.and_then(|p| out.get(p));
        let (item, mut problems) = match &bag.data {
            Some(data) => decode(data, parent.and_then(|p| p.item.as_ref()), code_page),
            None => (None, Vec::new()),
        };
        let name = match &item {
            Some(i) if !i.name.is_empty() => i.name.clone(),
            Some(i) => format!("[class 0x{:02x}]", i.class),
            None => "[unknown]".to_string(),
        };
        let path = match parent {
            Some(parent) => format!("{}\\{name}", parent.path),
            None => name,
        };
        let unknown = item
            .as_ref()
            .is_none_or(|i| matches!(i.kind, Kind::Other(_)) || i.name.is_empty());
        let raw = (unknown || !problems.is_empty() || !bag.problems.is_empty())
            .then(|| bag.data.clone())
            .flatten();
        problems.splice(0..0, bag.problems.iter().cloned());
        out.push(Decoded {
            item,
            path,
            problems,
            raw,
        });
    }
    out
}

// --- records ----------------------------------------------------------------------------------

/// An item's record, its fields in this order.
#[derive(Serialize)]
struct Record<'a> {
    host: Option<&'a str>,
    username: Option<&'a str>,
    user_sid: Option<&'a str>,
    source_path: &'a str,
    hive_type: &'static str,
    bag_key: String,
    mru_position: Option<u32>,
    is_most_recent_in_parent: bool,
    node_slot: Option<u32>,
    path: &'a str,
    item_type: Option<&'static str>,
    item_class: Option<String>,
    item_name: Option<&'a str>,
    guid: Option<&'a str>,
    item_modified: Option<String>,
    item_created: Option<String>,
    item_accessed: Option<String>,
    mft_entry: Option<u64>,
    mft_sequence: Option<u16>,
    extension_blocks: Option<usize>,
    key_last_written: Option<String>,
    parent_key_last_written: Option<String>,
    data_raw: Option<String>,
    transaction_log_status: &'static str,
    logs_applied: &'a [String],
    problems: &'a [String],
}

const CSV_HEADER: [&str; 26] = [
    "Host",
    "Username",
    "UserSid",
    "SourcePath",
    "HiveType",
    "BagKey",
    "MruPosition",
    "IsMostRecentInParent",
    "NodeSlot",
    "Path",
    "ItemType",
    "ItemClass",
    "ItemName",
    "Guid",
    "ItemModifiedUTC",
    "ItemCreatedUTC",
    "ItemAccessedUTC",
    "MftEntry",
    "MftSequence",
    "ExtensionBlocks",
    "KeyLastWrittenUTC",
    "ParentKeyLastWrittenUTC",
    "DataRaw",
    "TransactionLogStatus",
    "LogsApplied",
    "Problems",
];

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// A shell item's FAT time (seconds since 1970, as recorded) as RFC 3339.
fn fat_time(secs: Option<i64>) -> Option<String> {
    secs.and_then(|s| DateTime::<Utc>::from_timestamp(s, 0))
        .map(|t| t.to_rfc3339_opts(SecondsFormat::AutoSi, true))
}

/// A hive type as Windows names the file: `UsrClass.dat`, not `USRCLASS.DAT`.
fn hive_file_name(hive_type: &str) -> &'static str {
    if hive_type == "USRCLASS.DAT" {
        "UsrClass.dat"
    } else {
        "NTUSER.DAT"
    }
}

impl<'a> Record<'a> {
    fn new(opened: &'a Opened, user: &'a User, bag: &'a ShellBag, decoded: &'a Decoded) -> Self {
        let item = decoded.item.as_ref();
        Record {
            host: opened.host.as_deref(),
            username: user.username.as_deref(),
            user_sid: user.sid.as_deref(),
            source_path: &opened.source,
            hive_type: hive_file_name(&opened.hive_type),
            bag_key: bag.key_path(),
            mru_position: bag.mru_position,
            is_most_recent_in_parent: bag.mru_position == Some(0),
            node_slot: bag.node_slot,
            path: &decoded.path,
            item_type: item.map(|i| kind_name(&i.kind)),
            item_class: item.map(|i| format!("0x{:02x}", i.class)),
            item_name: item.map(|i| i.name.as_str()),
            guid: item.and_then(|i| i.guid.as_deref()),
            item_modified: fat_time(item.and_then(|i| i.modified)),
            item_created: fat_time(item.and_then(|i| i.created)),
            item_accessed: fat_time(item.and_then(|i| i.accessed)),
            mft_entry: item.and_then(|i| i.mft).map(|m| m.0),
            mft_sequence: item.and_then(|i| i.mft).map(|m| m.1),
            extension_blocks: item.map(|i| i.extension_blocks),
            key_last_written: bag.last_written.and_then(filetime),
            parent_key_last_written: filetime(bag.parent_last_written),
            data_raw: decoded.raw.as_deref().map(hex),
            transaction_log_status: opened.log_status(bag.replayed),
            logs_applied: &opened.recovery.logs_applied,
            problems: &decoded.problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let text = |v: Option<&str>| v.unwrap_or_default().to_string();
        let num = |v: Option<u64>| v.map(|n| n.to_string()).unwrap_or_default();
        vec![
            text(self.host),
            text(self.username),
            text(self.user_sid),
            self.source_path.to_string(),
            self.hive_type.to_string(),
            self.bag_key.clone(),
            num(self.mru_position.map(u64::from)),
            self.is_most_recent_in_parent.to_string(),
            num(self.node_slot.map(u64::from)),
            self.path.to_string(),
            text(self.item_type),
            text(self.item_class.as_deref()),
            text(self.item_name),
            text(self.guid),
            text(self.item_modified.as_deref()),
            text(self.item_created.as_deref()),
            text(self.item_accessed.as_deref()),
            num(self.mft_entry),
            num(self.mft_sequence.map(u64::from)),
            num(self.extension_blocks.map(|n| n as u64)),
            text(self.key_last_written.as_deref()),
            text(self.parent_key_last_written.as_deref()),
            text(self.data_raw.as_deref()),
            self.transaction_log_status.to_string(),
            self.logs_applied.join("|"),
            self.problems.join("; "),
        ]
    }
}

// --- summary ----------------------------------------------------------------------------------

/// Counts for the results summary.
#[derive(Default)]
struct Stats {
    /// Whether the records go to stdout (`-t` without `-o`): no summary, so what concerns a hive
    /// rather than one item goes to stderr.
    to_stderr: bool,
    hives: usize,
    failed: usize,
    hosts: BTreeSet<String>,
    users: BTreeSet<String>,
    recovered: usize,
    dirty: Vec<String>,
    without: usize,
    items: usize,
    kinds: BTreeMap<&'static str, usize>,
    /// The folders opened last: (their parent key's last write, line), for the items at
    /// `MRUListEx` position 0 only.
    latest: Vec<(u64, String)>,
    problems: Vec<String>,
    problem_count: usize,
}

impl Stats {
    /// Write the records of `opened`'s `BagMRU` items (when there is a writer) and count them.
    fn hive(
        &mut self,
        opened: &Opened,
        user: &User,
        code_page: u16,
        mut writer: Option<&mut RecordWriter>,
    ) -> io::Result<()> {
        self.hives += 1;
        self.hosts.extend(opened.host.clone());
        self.users.extend(user.username.clone());
        if opened.recovery.recovered {
            self.recovered += 1;
        } else if opened.recovery.dirty {
            self.dirty.push(opened.source.clone());
        }
        let Some((bags, problems)) = shell_bags(&opened.hive) else {
            self.without += 1;
            return Ok(());
        };
        for problem in &problems {
            if self.to_stderr {
                eprintln!("{}: {problem}", opened.source);
            }
            self.problem(&opened.source, problem);
        }
        let decoded = decode_all(&bags, code_page);
        for (bag, item) in bags.iter().zip(&decoded) {
            self.items += 1;
            *self
                .kinds
                .entry(item.item.as_ref().map_or("none", |i| kind_name(&i.kind)))
                .or_default() += 1;
            for problem in &item.problems {
                self.problem(&opened.source, &format!("{}: {problem}", bag.key_path()));
            }
            if bag.mru_position == Some(0)
                && let Some(at) = filetime(bag.parent_last_written)
            {
                let who = user.username.as_deref().unwrap_or("?");
                self.latest.push((
                    bag.parent_last_written,
                    format!("{at}  {who}  {}", item.path),
                ));
                if self.latest.len() > MAX_PRINTED * 4 {
                    self.trim_latest();
                }
            }
            let record = Record::new(opened, user, bag, item);
            match writer.as_deref_mut() {
                Some(w) if w.is_csv() => w.csv(&record.csv())?,
                Some(w) => w.json(&record)?,
                None => {}
            }
        }
        Ok(())
    }

    /// The most recent folders, newest first, at most [`MAX_PRINTED`].
    fn trim_latest(&mut self) {
        self.latest.sort_by(|a, b| b.cmp(a));
        self.latest.truncate(MAX_PRINTED);
    }

    fn problem(&mut self, source: &str, problem: &str) {
        self.problem_count += 1;
        if self.problems.len() < MAX_PRINTED {
            self.problems.push(format!("{source}: {problem}"));
        }
    }

    fn print(&mut self, no_color: bool) {
        self.trim_latest();
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        p_label(
            no_color,
            "UsrClass.dat and NTUSER.DAT hives parsed: ",
            &fmt_num(self.hives),
        );
        if self.failed > 0 {
            p_label(no_color, "Could not parse: ", &fmt_num(self.failed));
        }
        let list = |set: &BTreeSet<String>| set.iter().cloned().collect::<Vec<_>>().join(", ");
        if !self.hosts.is_empty() {
            p_label(no_color, "Hosts: ", &list(&self.hosts));
        }
        if !self.users.is_empty() {
            p_label(no_color, "Users: ", &list(&self.users));
        }
        p_label(
            no_color,
            "Dirty hives recovered with their transaction logs: ",
            &fmt_num(self.recovered),
        );
        if !self.dirty.is_empty() {
            p_label(
                no_color,
                "Dirty hives whose transaction logs were not applied (items may be missing or out of date): ",
                &fmt_num(self.dirty.len()),
            );
            for source in self.dirty.iter().take(MAX_PRINTED) {
                p(None, &format!("  {source}"), true);
            }
        }
        if self.without > 0 {
            p_label(
                no_color,
                "Hives without ShellBags: ",
                &fmt_num(self.without),
            );
        }
        let kinds: Vec<String> = self
            .kinds
            .iter()
            .map(|(kind, n)| format!("{kind} {}", fmt_num(*n)))
            .collect();
        p_label(
            no_color,
            "Items: ",
            &if kinds.is_empty() {
                fmt_num(self.items)
            } else {
                format!("{} ({})", fmt_num(self.items), kinds.join(", "))
            },
        );
        if !self.latest.is_empty() {
            p_label(
                no_color,
                "Opened last (UTC from the parent key's last write, user, folder): ",
                "",
            );
            for (_, line) in &self.latest {
                p(None, &format!("  {line}"), true);
            }
        }
        p_label(
            no_color,
            "Items or keys that could not be read: ",
            &fmt_num(self.problem_count),
        );
        for problem in &self.problems {
            p(None, &format!("  {problem}"), true);
        }
        if self.problem_count > self.problems.len() {
            p(
                None,
                &format!(
                    "  … {} more in the Problems column (-o <FILE>)",
                    fmt_num(self.problem_count - self.problems.len())
                ),
                true,
            );
        }
        p(None, "", true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file entry for a folder: class 0x31, its short name, and a `BEEF0004` block (version 9)
    /// with the long name and the NTFS file reference.
    fn folder(short: &str, long: &str, entry: u64, sequence: u16) -> Vec<u8> {
        let mut item = vec![0, 0, 0x31, 0];
        item.extend(0u32.to_le_bytes()); // file size
        item.extend([0x21, 0x5a, 0x00, 0x60]); // 2025-01-01 12:00:00 (FAT)
        item.extend(0x10u16.to_le_bytes()); // directory
        item.extend(short.bytes());
        item.push(0);
        if item.len() % 2 == 1 {
            item.push(0);
        }
        let block_at = item.len();
        let mut block = vec![0, 0, 9, 0];
        block.extend(0xbeef_0004u32.to_le_bytes());
        block.extend([0x21, 0x5a, 0x00, 0x60, 0x21, 0x5a, 0x00, 0x60]); // created, accessed
        block.extend(0x2eu16.to_le_bytes()); // identifier
        block.extend(0u16.to_le_bytes());
        block.extend((entry | (u64::from(sequence) << 48)).to_le_bytes());
        block.extend([0; 8]);
        block.extend(0u16.to_le_bytes()); // long string size
        block.extend([0; 4]);
        block.extend([0; 4]);
        block.extend(long.encode_utf16().chain([0]).flat_map(u16::to_le_bytes));
        block.extend((block_at as u16).to_le_bytes());
        let size = block.len() as u16;
        block[..2].copy_from_slice(&size.to_le_bytes());
        item.extend(block);
        let size = item.len() as u16;
        item[..2].copy_from_slice(&size.to_le_bytes());
        item
    }

    fn list(item: &[u8]) -> Vec<u8> {
        [item, &[0, 0]].concat()
    }

    fn bag(numbers: &[u32], parent: Option<usize>, data: Option<Vec<u8>>) -> ShellBag {
        ShellBag {
            location: r"Local Settings\Software\Microsoft\Windows\Shell\BagMRU",
            numbers: numbers.to_vec(),
            parent,
            mru_position: Some(0),
            data,
            ..ShellBag::default()
        }
    }

    #[test]
    fn paths_join_each_item_to_its_parents_and_unknown_items_keep_their_bytes() {
        let this_pc = list(&[
            0x14, 0, 0x1f, 0x50, 0xe0, 0x4f, 0xd0, 0x20, 0xea, 0x3a, 0x69, 0x10, 0xa2, 0xd8, 0x08,
            0x00, 0x2b, 0x30, 0x30, 0x9d,
        ]);
        let mut volume = vec![0x19, 0, 0x2f];
        volume.extend(b"C:\\");
        volume.resize(0x19, 0);
        let unknown = list(&[6, 0, 0x99, 1, 2, 3]);
        let bags = [
            bag(&[0], None, Some(this_pc)),
            bag(&[0, 0], Some(0), Some(list(&volume))),
            bag(
                &[0, 0, 0],
                Some(1),
                Some(list(&folder("USERS", "Users", 512, 3))),
            ),
            bag(&[0, 0, 0, 0], Some(2), Some(unknown.clone())),
            bag(&[0, 0, 0, 0, 0], Some(3), None),
        ];
        let decoded = decode_all(&bags, 1252);
        let paths: Vec<&str> = decoded.iter().map(|d| d.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "This PC",
                r"This PC\C:",
                r"This PC\C:\Users",
                r"This PC\C:\Users\[class 0x99]",
                r"This PC\C:\Users\[class 0x99]\[unknown]",
            ]
        );
        let users = decoded[2].item.as_ref().unwrap();
        assert_eq!(users.kind, Kind::Directory);
        assert_eq!(users.mft, Some((512, 3)));
        assert_eq!(
            fat_time(users.modified).as_deref(),
            Some("2025-01-01T12:00:00Z")
        );
        assert!(decoded[2].raw.is_none() && decoded[2].problems.is_empty());
        assert_eq!(decoded[3].raw.as_deref(), Some(unknown.as_slice()));
        assert_eq!(
            kind_name(&decoded[3].item.as_ref().unwrap().kind),
            "unknown"
        );
    }

    #[test]
    fn a_list_without_its_end_or_with_more_items_says_so() {
        let one = folder("A", "A", 1, 1);
        let (item, problems) = decode(&one, None, 1252);
        assert!(item.is_some());
        assert_eq!(problems, ["its shell item list has no end (a size of 0)"]);
        let two = [one.as_slice(), &one, &[0, 0]].concat();
        let (_, problems) = decode(&two, None, 1252);
        assert_eq!(problems, ["2 shell items, not one: only the first is read"]);
        let (item, problems) = decode(&[0, 0], None, 1252);
        assert!(item.is_none());
        assert_eq!(problems, ["no shell item"]);
    }
}
