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

//! `parse-registry`: registry hives (SYSTEM, SOFTWARE, SAM, SECURITY, NTUSER.DAT, UsrClass.dat,
//! Amcache.hve, …) read by ys-registry-parser, one subcommand per artifact. What they share is
//! here: finding the hives of a collection, opening each with its transaction logs replayed when
//! it is dirty, naming its type and the host it comes from, and writing CSV, JSON or JSONL.

pub mod app_history;
pub mod appcompat;
pub mod auditpol;
pub mod bam;
pub mod bcd;
pub mod featureusage;
pub mod installed_programs;
pub mod lsa_secrets;
pub mod mru;
pub mod muicache;
pub mod networks;
pub mod office;
pub mod opensavemru;
pub mod persistence;
pub mod read;
pub mod recentdocs;
pub mod remote_access;
pub mod sam;
pub mod security_settings;
pub mod services;
pub mod shellbags;
pub mod system_info;
pub mod taskcache;
pub mod usb;
pub mod userassist;

use crate::cli::RegistryOutputType;
use crate::util::{self, check_output};
use registry::system::computer_name;
use registry::{Hive, LogFile, Recovery, find_logs};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

/// The hives Windows keeps, as their base blocks name them (in upper case).
const HIVE_TYPES: [&str; 14] = [
    "SYSTEM",
    "SOFTWARE",
    "SAM",
    "SECURITY",
    "DEFAULT",
    "COMPONENTS",
    "DRIVERS",
    "ELAM",
    "BBI",
    "BCD",
    "NTUSER.DAT",
    "USRCLASS.DAT",
    "AMCACHE.HVE",
    "SYSCACHE.HVE",
];

// --- hives ------------------------------------------------------------------------------------

/// Whether a file is a primary hive file: the `regf` signature and file type 0 (transaction logs
/// start with a base block too), and not named as a log.
pub(crate) fn is_hive_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_uppercase())
        .unwrap_or_default();
    if [".LOG", ".LOG1", ".LOG2"].iter().any(|x| name.ends_with(x)) {
        return false;
    }
    let mut head = [0u8; 32];
    File::open(path)
        .and_then(|mut f| io::Read::read_exact(&mut f, &mut head))
        .is_ok()
        && &head[..4] == b"regf"
        && head[28..32] == [0; 4]
}

/// The primary hive files under `dir`, recursively, in path order.
pub fn find_hives(dir: &Path) -> Vec<PathBuf> {
    util::find_files(dir, |_| true)
        .into_iter()
        .filter(|p| is_hive_file(p))
        .collect()
}

/// The type of the hive file at `path` from its base block alone: the name it records (else the
/// file's), when Windows keeps a hive of that name.
pub(crate) fn hive_type_of_file(path: &Path) -> Option<String> {
    let mut base = [0u8; 0x70];
    File::open(path)
        .and_then(|mut f| io::Read::read_exact(&mut f, &mut base))
        .ok()?;
    let units: Vec<u16> = base[0x30..0x70]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|u| u16::from_le_bytes(*u))
        .take_while(|&u| u != 0)
        .collect();
    let recorded = String::from_utf16_lossy(&units);
    let last = recorded.rsplit(['\\', '/']).next().unwrap_or_default();
    hive_type(last).or_else(|| hive_type(&path.file_name()?.to_string_lossy()))
}

/// `name` in upper case, when Windows keeps a hive of that name.
fn hive_type(name: &str) -> Option<String> {
    let upper = name.to_uppercase();
    HIVE_TYPES.contains(&upper.as_str()).then_some(upper)
}

/// The primary hive files of type `kind` (`SYSTEM`, …) under `dir`, recursively, in path order.
pub fn find_hives_of_type(dir: &Path, kind: &str) -> Vec<PathBuf> {
    find_hives(dir)
        .into_iter()
        .filter(|p| hive_type_of_file(p).as_deref() == Some(kind))
        .collect()
}

/// The files to read: `-f`'s, or those `find` finds under `-d` (`what` names them in errors).
pub fn input_files(
    file: Option<&PathBuf>,
    dir: Option<&PathBuf>,
    what: &str,
    find: impl Fn(&Path) -> Vec<PathBuf>,
) -> Result<Vec<PathBuf>, String> {
    match (file, dir) {
        (Some(f), _) if f.is_file() => Ok(vec![f.clone()]),
        (Some(f), _) => Err(format!("The file does not exist: {}", f.display())),
        (None, Some(d)) if !d.is_dir() => {
            Err(format!("The directory does not exist: {}", d.display()))
        }
        (None, Some(d)) => {
            let found = find(d);
            if found.is_empty() {
                Err(format!("No {what} were found in {}", d.display()))
            } else {
                Ok(found)
            }
        }
        (None, None) => unreachable!("clap requires -f or -d"),
    }
}

/// The transaction logs to replay over a dirty hive.
#[derive(Clone, Copy)]
pub enum Logs<'a> {
    /// `<hive>.LOG`, `.LOG1` and `.LOG2` next to it.
    NextToIt,
    /// These files.
    Given(&'a [PathBuf]),
    /// None: the hive as it is.
    Without,
}

/// A hive read for output.
pub struct Opened {
    pub source: String,
    /// `SYSTEM`, `NTUSER.DAT`, …: the name its base block records, when Windows keeps a hive of
    /// that name (else the file's, else empty).
    pub hive_type: String,
    /// The computer name of the system the hive comes from: see [`Hosts`].
    pub host: Option<String>,
    pub recovery: Recovery,
    pub hive: Hive,
}

impl Opened {
    /// How the transaction logs bear on a record whose cells the replay wrote (`replayed`) or
    /// not: `clean` (the hive wasn't dirty), `replayed`, `unchanged`, or `not_applied` (a dirty
    /// hive without logs that apply: the record may be out of date).
    pub fn log_status(&self, replayed: bool) -> &'static str {
        if self.recovery.recovered {
            if replayed { "replayed" } else { "unchanged" }
        } else if self.recovery.dirty {
            "not_applied"
        } else {
            "clean"
        }
    }
}

/// The logs `--no-logs`, `--log1` and `--log2` choose; an error when a log given doesn't exist.
pub fn choose_logs(no_logs: bool, given: &[PathBuf]) -> Result<Logs<'_>, String> {
    if let Some(missing) = given.iter().find(|l| !l.is_file()) {
        return Err(format!("The file does not exist: {}", missing.display()));
    }
    Ok(if no_logs {
        Logs::Without
    } else if given.is_empty() {
        Logs::NextToIt
    } else {
        Logs::Given(given)
    })
}

/// Open the hive at `path`, with `logs` replayed when it is dirty.
pub fn open(path: &Path, logs: Logs) -> Result<Opened, String> {
    let logs = match logs {
        Logs::NextToIt => find_logs(path),
        Logs::Without => Vec::new(),
        Logs::Given(files) => files
            .iter()
            .map(|l| {
                Ok(LogFile {
                    name: l.file_name().map_or_else(
                        || l.to_string_lossy().to_string(),
                        |n| n.to_string_lossy().to_string(),
                    ),
                    data: std::fs::read(l).map_err(|e| format!("{}: {e}", l.display()))?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
    };
    let (hive, recovery) = registry::open_with(path, &logs).map_err(|e| e.0)?;
    let hive_type = hive_type(hive.header().name())
        .or_else(|| hive_type(&path.file_name()?.to_string_lossy()))
        .unwrap_or_default();
    Ok(Opened {
        source: path.to_string_lossy().to_string(),
        hive_type,
        host: None,
        recovery,
        hive,
    })
}

// --- hosts ------------------------------------------------------------------------------------

/// The hosts hives come from. A SYSTEM hive names its own; another hive takes the name of the
/// SYSTEM hive of its system: `Windows\System32\config\SYSTEM` of the system whose files it is
/// among, when it is where Windows keeps hives (a collection keeps a system's files as they
/// were), else a `SYSTEM` hive next to it (hives exported to one directory). Without one, the
/// host is unknown: a hive is never given the name of a system it may not come from.
pub struct Hosts {
    logs: bool,
    names: HashMap<PathBuf, Option<String>>,
}

impl Hosts {
    /// `logs`: whether SYSTEM hives read for their names get their logs replayed.
    pub fn new(logs: bool) -> Self {
        Hosts {
            logs,
            names: HashMap::new(),
        }
    }

    /// The host of `opened`, read from the file at `path`.
    pub fn of(&mut self, path: &Path, opened: &Opened) -> Option<String> {
        if opened.hive_type == "SYSTEM" {
            let name = computer_name(&opened.hive);
            self.names.insert(path.to_path_buf(), name.clone());
            return name;
        }
        let system = system_root(path)
            .and_then(|root| find_in(&root, &["Windows", "System32", "config", "SYSTEM"]))
            .or_else(|| find_in(path.parent()?, &["SYSTEM"]))
            .filter(|s| s != path)?;
        if let Some(name) = self.names.get(&system) {
            return name.clone();
        }
        let logs = if self.logs {
            Logs::NextToIt
        } else {
            Logs::Without
        };
        let name = open(&system, logs)
            .ok()
            .filter(|o| o.hive_type == "SYSTEM")
            .and_then(|o| computer_name(&o.hive));
        self.names.insert(system, name.clone());
        name
    }
}

/// Where Windows keeps hives, under the directory that holds its `Windows` and `Users`
/// directories (`*`: any name).
const HIVE_PLACES: [&[&str]; 9] = [
    &["Windows", "System32", "config", "*"],
    &["Windows", "System32", "config", "RegBack", "*"],
    &[
        "Windows",
        "System32",
        "config",
        "systemprofile",
        "NTUSER.DAT",
    ],
    &["Windows", "ServiceProfiles", "*", "NTUSER.DAT"],
    &["Windows", "AppCompat", "Programs", "Amcache.hve"],
    &["Users", "*", "NTUSER.DAT"],
    &[
        "Users",
        "*",
        "AppData",
        "Local",
        "Microsoft",
        "Windows",
        "UsrClass.dat",
    ],
    &["Documents and Settings", "*", "NTUSER.DAT"],
    &[
        "Documents and Settings",
        "*",
        "Local Settings",
        "Application Data",
        "Microsoft",
        "Windows",
        "UsrClass.dat",
    ],
];

/// The directory of the system the hive at `path` comes from, when it is where Windows keeps
/// hives ([`HIVE_PLACES`]). Only those places: a hive copied somewhere under the analyst's own
/// `Users` directory must not take the name of the analyst's system.
fn system_root(path: &Path) -> Option<PathBuf> {
    let parts: Vec<_> = path.components().collect();
    HIVE_PLACES.iter().find_map(|place| {
        let start = parts.len().checked_sub(place.len())?;
        let matches = place.iter().zip(&parts[start..]).all(|(want, part)| {
            *want == "*"
                || part
                    .as_os_str()
                    .to_string_lossy()
                    .eq_ignore_ascii_case(want)
        });
        matches.then(|| match start {
            0 => PathBuf::from("."),
            _ => parts[..start].iter().collect(),
        })
    })
}

/// The SOFTWARE hive of the system the user hive at `path` comes from, as [`Hosts`] finds SYSTEM
/// hives: the system's `Windows\System32\config\SOFTWARE` when the user hive is where Windows
/// keeps it, else a `SOFTWARE` hive next to it (hives exported to one directory).
pub fn software_of(path: &Path) -> Option<PathBuf> {
    system_root(path)
        .and_then(|root| find_in(&root, &["Windows", "System32", "config", "SOFTWARE"]))
        .or_else(|| find_in(path.parent()?, &["SOFTWARE"]))
}

/// The `NTUSER.DAT` hives of the system whose SYSTEM hive is at `system`: each profile's under
/// the system's `Users` and `Documents and Settings` when SYSTEM is where Windows keeps it, else
/// those next to it and one directory down (hives exported to one directory). In path order.
pub fn users_of(system: &Path) -> Vec<PathBuf> {
    let children = |dir: &Path| -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| e.file_type().is_ok_and(|t| !t.is_symlink()))
                    .map(|e| e.path())
                    .collect()
            })
            .unwrap_or_default();
        paths.sort();
        paths
    };
    let is_ntuser = |p: &PathBuf| {
        p.is_file()
            && p.file_name()
                .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("NTUSER.DAT"))
    };
    let ntuser_in = |dir: &Path| children(dir).into_iter().find(|p| is_ntuser(p));
    if let Some(root) = system_root(system) {
        return children(&root)
            .iter()
            .filter(|d| {
                d.file_name().is_some_and(|n| {
                    let n = n.to_string_lossy();
                    n.eq_ignore_ascii_case("Users")
                        || n.eq_ignore_ascii_case("Documents and Settings")
                })
            })
            .flat_map(|d| children(d))
            .filter(|profile| profile.is_dir())
            .filter_map(|profile| ntuser_in(&profile))
            .collect();
    }
    let dir = system
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut found: Vec<PathBuf> = children(dir).into_iter().filter(is_ntuser).collect();
    found.extend(
        children(dir)
            .iter()
            .filter(|d| d.is_dir())
            .filter_map(|d| ntuser_in(d)),
    );
    found
}

/// The hive `name` (`SOFTWARE`, `SAM`, …) of the system the hive at `path` comes from: the
/// system's `Windows\System32\config\<name>` when the hive is where Windows keeps it, else a
/// hive of that name next to it (hives exported to one directory); never the hive itself.
pub fn system_hive_of(path: &Path, name: &str) -> Option<PathBuf> {
    system_root(path)
        .and_then(|root| find_in(&root, &["Windows", "System32", "config", name]))
        .or_else(|| find_in(path.parent()?, &[name]))
        .filter(|found| found != path)
}

/// The SID of the profile whose folder is named `user`, among `profiles` (`ProfileList`'s SIDs
/// and folders: `C:\Users\alice`).
pub fn sid_of_profile(profiles: &[(String, String)], user: &str) -> Option<String> {
    let matches: std::collections::BTreeSet<_> = profiles
        .iter()
        .filter(|(sid, path)| {
            valid_sid(sid)
                && path
                    .trim_end_matches(['\\', '/'])
                    .rsplit(['\\', '/'])
                    .next()
                    .is_some_and(|folder| folder.eq_ignore_ascii_case(user))
        })
        .map(|(sid, _)| sid.to_ascii_uppercase())
        .collect();
    (matches.len() == 1)
        .then(|| matches.into_iter().next())
        .flatten()
}

/// A complete Windows SID, excluding ProfileList backup names such as `S-1-...bak`.
fn valid_sid(sid: &str) -> bool {
    let mut parts = sid.split('-');
    if !parts.next().is_some_and(|s| s.eq_ignore_ascii_case("S")) || parts.next() != Some("1") {
        return false;
    }
    let Some(authority) = parts.next() else {
        return false;
    };
    if authority.is_empty()
        || !authority.bytes().all(|b| b.is_ascii_digit())
        || !authority.parse::<u64>().is_ok_and(|n| n < (1u64 << 48))
    {
        return false;
    }
    let sub: Vec<_> = parts.collect();
    (1..=15).contains(&sub.len())
        && sub.iter().all(|s| {
            !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) && s.parse::<u32>().is_ok()
        })
}

/// The profiles (`ProfileList`) of the systems user hives come from, each SOFTWARE hive
/// ([`software_of`]) read once.
pub struct Profiles {
    logs: bool,
    read: HashMap<PathBuf, Vec<(String, String)>>,
}

impl Profiles {
    /// `logs`: whether SOFTWARE hives read for their profiles get their logs replayed.
    pub fn new(logs: bool) -> Self {
        Profiles {
            logs,
            read: HashMap::new(),
        }
    }

    /// The SID of `user`, whose hive is at `path`: from its system's `ProfileList`.
    pub fn sid(&mut self, path: &Path, user: &str) -> Option<String> {
        let software = software_of(path)?;
        let logs = if self.logs {
            Logs::NextToIt
        } else {
            Logs::Without
        };
        let profiles = self.read.entry(software).or_insert_with_key(|software| {
            open(software, logs)
                .ok()
                .filter(|o| o.hive_type == "SOFTWARE")
                .map(|o| registry::software::profile_list(&o.hive))
                .unwrap_or_default()
        });
        sid_of_profile(profiles, user)
    }
}

/// The name of the profile folder a user hive is in: `alice` for `…\Users\alice\NTUSER.DAT`
/// (also `Documents and Settings`, and `Windows\ServiceProfiles\<service>`).
pub fn profile_of_path(path: &Path) -> Option<String> {
    let parts: Vec<String> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();
    let n = parts.len();
    (n >= 3
        && ["users", "documents and settings", "serviceprofiles"]
            .contains(&parts[n - 3].to_lowercase().as_str()))
    .then(|| parts[n - 2].clone())
}

/// The file at `dir` + `names`, each name in any case.
fn find_in(dir: &Path, names: &[&str]) -> Option<PathBuf> {
    let mut at = dir.to_path_buf();
    for name in names {
        at = std::fs::read_dir(&at)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .find(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case(name))
            })?;
    }
    at.is_file().then_some(at)
}

// --- output -----------------------------------------------------------------------------------

/// The transaction logs next to the hive at `path`: `<hive>.LOG`, `.LOG1` and `.LOG2`, in any
/// case (the files `registry::find_logs` reads).
fn logs_next_to(path: &Path) -> Vec<PathBuf> {
    let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_uppercase()) else {
        return Vec::new();
    };
    let dir = path
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let wanted: Vec<String> = [".LOG", ".LOG1", ".LOG2"]
        .iter()
        .map(|x| format!("{name}{x}"))
        .collect();
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| wanted.contains(&e.file_name().to_string_lossy().to_uppercase()))
                .map(|e| e.path())
                .collect()
        })
        .unwrap_or_default()
}

/// Every file a registry command may read for the hives `files`: each hive and the logs next to
/// it, the logs given (`--log1`, `--log2`), and the SYSTEM, SOFTWARE and SAM hives of its system
/// with their logs (read for its host, by `persistence` for the machine's classes and profiles,
/// and by `bam` for its users' names).
pub fn files_read(files: &[PathBuf], given_logs: &[PathBuf]) -> Vec<PathBuf> {
    let mut hives: Vec<PathBuf> = files.to_vec();
    for path in files {
        let dir = path
            .parent()
            .filter(|d| !d.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        for name in ["SYSTEM", "SOFTWARE", "SAM"] {
            hives.extend(
                system_root(path)
                    .and_then(|root| find_in(&root, &["Windows", "System32", "config", name])),
            );
            hives.extend(find_in(dir, &[name]));
        }
    }
    let mut read: Vec<PathBuf> = given_logs.to_vec();
    for hive in hives {
        read.extend(logs_next_to(&hive));
        read.push(hive);
    }
    read.sort();
    read.dedup();
    read
}

/// What tells files apart: the same for every path, symbolic link or hard link to one file. On
/// Unix the device and inode `stat` gives, which needs no permission on the file itself (opening
/// it would: a write-only file couldn't be told apart, yet could be truncated); on Windows
/// `same_file`'s handle, opened without asking for access.
#[cfg(unix)]
type Identity = (u64, u64);
#[cfg(not(unix))]
type Identity = same_file::Handle;

#[cfg(unix)]
fn identity(path: &Path) -> io::Result<Identity> {
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::metadata(path)?;
    Ok((meta.dev(), meta.ino()))
}

#[cfg(not(unix))]
fn identity(path: &Path) -> io::Result<Identity> {
    same_file::Handle::from_path(path)
}

/// An error when the output file `path` is one of the files `reads` gives, or when that can't be
/// told (fail closed: it is not overwritten). A file that isn't there can't be one of them.
fn check_not_read(path: &Path, reads: impl FnOnce() -> Vec<PathBuf>) -> Result<(), String> {
    let unsure = |of: &Path, e: &io::Error| {
        format!(
            "Could not tell whether the output file {} is the input file {}, so it is not overwritten: {e}",
            path.display(),
            of.display()
        )
    };
    let out = match identity(path) {
        Ok(out) => out,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(format!(
                "Could not tell whether the output file {} is one of the input files, so it is not overwritten: {e}",
                path.display()
            ));
        }
    };
    for input in reads() {
        match identity(&input) {
            Ok(id) if id == out => {
                let also = if input == path {
                    String::new()
                } else {
                    format!(" (the same file as {})", input.display())
                };
                return Err(format!(
                    "The output file is also an input file and cannot be overwritten: {}{also}",
                    path.display()
                ));
            }
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(unsure(&input, &e)),
        }
    }
    Ok(())
}

/// Where the records go: the file of `-o` (created, with its directory), stdout (`-t` without
/// `-o`), or nowhere (the summary only). An existing `-o` that is one of the files `reads` gives
/// (the same file by path, symbolic link or hard link), or that can't be told apart from them, is
/// refused, even with `-C`, before anything is written.
pub fn sink(
    output: Option<&PathBuf>,
    clobber: bool,
    data_on_stdout: bool,
    reads: impl FnOnce() -> Vec<PathBuf>,
) -> Result<Option<Box<dyn Write>>, String> {
    let Some(path) = output else {
        return Ok(data_on_stdout.then(|| Box::new(BufWriter::new(io::stdout())) as Box<dyn Write>));
    };
    check_not_read(path, reads)?;
    check_output(path, false, clobber)?;
    if let Some(parent) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create {}: {e}", parent.display()))?;
    }
    let file =
        File::create(path).map_err(|e| format!("Could not create {}: {e}", path.display()))?;
    Ok(Some(Box::new(BufWriter::new(file))))
}

/// The CSV, JSON or JSONL writer of a subcommand's records.
pub enum RecordWriter {
    Csv(Box<csv::Writer<Box<dyn Write>>>),
    Json(Box<dyn Write>, bool),
}

impl RecordWriter {
    /// A writer; a CSV gets its `header` now, so that it has one also without records.
    pub fn new(
        out: Box<dyn Write>,
        output_type: RegistryOutputType,
        header: &[&str],
    ) -> io::Result<Self> {
        Ok(match output_type {
            RegistryOutputType::Csv => {
                let mut w = csv::Writer::from_writer(out);
                w.write_record(header)?;
                RecordWriter::Csv(Box::new(w))
            }
            RegistryOutputType::Json => RecordWriter::Json(out, true),
            RegistryOutputType::Jsonl => RecordWriter::Json(out, false),
        })
    }

    /// Whether records are written as CSV rows (else as JSON).
    pub fn is_csv(&self) -> bool {
        matches!(self, RecordWriter::Csv(_))
    }

    pub fn csv(&mut self, row: &[String]) -> io::Result<()> {
        if let RecordWriter::Csv(w) = self {
            w.write_record(row)?;
        }
        Ok(())
    }

    pub fn json(&mut self, record: &impl Serialize) -> io::Result<()> {
        if let RecordWriter::Json(w, pretty) = self {
            // JSON: pretty objects separated by newlines, like the other commands; JSONL: one per
            // line
            if *pretty {
                serde_json::to_writer_pretty(&mut *w, record)?;
            } else {
                serde_json::to_writer(&mut *w, record)?;
            }
            w.write_all(b"\n")?;
        }
        Ok(())
    }

    pub fn finish(self) -> io::Result<()> {
        match self {
            RecordWriter::Csv(mut w) => w.flush(),
            RecordWriter::Json(mut w, _) => w.flush(),
        }
    }
}

/// A FILETIME in UTC (RFC 3339, to the 100 ns), or `None` for 0 and times out of range.
pub fn filetime(ft: u64) -> Option<String> {
    Some(crate::lnk::filetime(ft)).filter(|t| !t.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_sid_requires_a_unique_valid_identity() {
        let a = "S-1-5-21-1-2-3-1001";
        let b = "S-1-5-21-4-5-6-1001";
        let profiles = vec![
            (a.into(), r"C:\Users\alice".into()),
            (b.into(), r"D:\Profiles\ALICE".into()),
        ];
        assert_eq!(sid_of_profile(&profiles, "alice"), None);
        assert_eq!(
            sid_of_profile(&[(format!("{a}.bak"), r"C:\Users\alice".into())], "alice"),
            None
        );
        assert_eq!(
            sid_of_profile(&[(a.into(), r"C:\Users\Alice\".into())], "alice"),
            Some(a.into())
        );
        assert_eq!(
            sid_of_profile(
                &[
                    (a.into(), r"C:\Users\alice".into()),
                    (a.to_lowercase(), r"C:\Users\ALICE".into())
                ],
                "alice"
            ),
            Some(a.into())
        );
        for sid in ["S-1-5-18", a] {
            assert!(valid_sid(sid));
        }
        for sid in [
            "S-1-5",
            "S-1-5-+18",
            "S-1-5-4294967296",
            "S-1-281474976710656-1",
            "S-1-5-18.bak",
        ] {
            assert!(!valid_sid(sid), "{sid}");
        }
    }

    #[test]
    fn hive_files_and_logs() {
        let dir = tempfile::tempdir().unwrap();
        let mut hive = vec![0u8; 4096];
        hive[..4].copy_from_slice(b"regf");
        std::fs::write(dir.path().join("NTUSER.DAT"), &hive).unwrap();
        std::fs::write(dir.path().join("NTUSER.DAT.LOG1"), &hive).unwrap();
        hive[28] = 1;
        std::fs::write(dir.path().join("renamed_log"), &hive).unwrap();
        std::fs::write(dir.path().join("notes.txt"), b"not a hive").unwrap();
        let found = find_hives(dir.path());
        assert_eq!(found, [dir.path().join("NTUSER.DAT")]);
    }

    #[test]
    fn profiles_of_user_hives() {
        let profile = |p: &str| profile_of_path(Path::new(p));
        assert_eq!(
            profile("/case/C/Users/alice/NTUSER.DAT").as_deref(),
            Some("alice")
        );
        assert_eq!(
            profile("/case/C/Documents and Settings/bob/NTUSER.DAT").as_deref(),
            Some("bob")
        );
        assert_eq!(
            profile("/case/C/Windows/ServiceProfiles/LocalService/NTUSER.DAT").as_deref(),
            Some("LocalService")
        );
        assert_eq!(profile("/case/hives/NTUSER.DAT"), None);
        assert_eq!(profile("NTUSER.DAT"), None);
    }

    #[test]
    fn systems_of_collected_files() {
        let root = |p: &str| system_root(Path::new(p));
        for (path, expected) in [
            ("/case/H1/C/Windows/System32/config/SOFTWARE", "/case/H1/C"),
            (
                "/case/H1/C/Windows/System32/config/RegBack/SAM",
                "/case/H1/C",
            ),
            (
                "/case/H1/C/Windows/AppCompat/Programs/Amcache.hve",
                "/case/H1/C",
            ),
            ("/case/H1/C/Users/alice/NTUSER.DAT", "/case/H1/C"),
            (
                "/case/H1/C/Users/alice/AppData/Local/Microsoft/Windows/UsrClass.dat",
                "/case/H1/C",
            ),
            (
                "/case/H1/C/Documents and Settings/bob/ntuser.dat",
                "/case/H1/C",
            ),
            (
                "/case/H1/C/Windows/ServiceProfiles/LocalService/NTUSER.DAT",
                "/case/H1/C",
            ),
            (
                "/Users/analyst/case/C/Users/alice/NTUSER.DAT",
                "/Users/analyst/case/C",
            ),
            (
                "/Users/analyst/case/C/Windows/System32/config/SYSTEM",
                "/Users/analyst/case/C",
            ),
            ("Windows/System32/config/SOFTWARE", "."),
        ] {
            assert_eq!(root(path), Some(PathBuf::from(expected)), "{path}");
        }
        // hives elsewhere, the analyst's own Users directory among them
        for path in [
            "hives/NTUSER.DAT",
            "/Users/analyst/case/SOFTWARE",
            "/Users/analyst/AppData/Local/Temp/t/Amcache.hve",
            "/case/H1/C/Users/alice/Documents/NTUSER.DAT",
            "/case/H1/C/Windows/Temp/SYSTEM",
        ] {
            assert_eq!(root(path), None, "{path}");
        }
    }
}
