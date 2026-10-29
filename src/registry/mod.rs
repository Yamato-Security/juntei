//! `parse-registry`: registry hives (SYSTEM, SOFTWARE, SAM, SECURITY, NTUSER.DAT, UsrClass.dat,
//! Amcache.hve, …) read by ys-registry-parser, one subcommand per artifact. What they share is
//! here: finding the hives of a collection, opening each with its transaction logs replayed when
//! it is dirty, naming its type and the host it comes from, and writing CSV, JSON or JSONL.

pub mod read;

use crate::cli::RegistryOutputType;
use crate::util;
use registry::{Data, Hive, LogFile, Recovery, decode, find_logs};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
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
fn is_hive_file(path: &Path) -> bool {
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
    let named = |name: &str| {
        let upper = name.to_uppercase();
        HIVE_TYPES.contains(&upper.as_str()).then_some(upper)
    };
    let hive_type = named(hive.header().name())
        .or_else(|| named(&path.file_name()?.to_string_lossy()))
        .unwrap_or_default();
    Ok(Opened {
        source: path.to_string_lossy().to_string(),
        hive_type,
        host: None,
        recovery,
        hive,
    })
}

// --- SYSTEM -----------------------------------------------------------------------------------

/// The text of a string value, when it is one.
fn text(hive: &Hive, key: &str, name: &str) -> Option<String> {
    let value = hive.key(key)?.value(name)?;
    match decode(value.data_type(), &value.data().ok()?) {
        Data::Text(s) if !s.is_empty() => Some(s),
        _ => None,
    }
}

/// The number of the control set Windows used last (`Select\Current`) in a SYSTEM hive.
pub fn current_control_set(hive: &Hive) -> Option<u32> {
    let value = hive.key("Select")?.value("Current")?;
    match decode(value.data_type(), &value.data().ok()?) {
        Data::Number(n) => u32::try_from(n).ok().filter(|&n| n > 0),
        _ => None,
    }
}

/// A SYSTEM hive's computer name: `ComputerName` in the control set Windows used last (else the
/// first), or the TCP/IP host name.
fn computer_name(hive: &Hive) -> Option<String> {
    let set = format!("ControlSet{:03}", current_control_set(hive).unwrap_or(1));
    text(
        hive,
        &format!(r"{set}\Control\ComputerName\ComputerName"),
        "ComputerName",
    )
    .or_else(|| {
        text(
            hive,
            &format!(r"{set}\Services\Tcpip\Parameters"),
            "Hostname",
        )
    })
}

/// The hosts hives come from. A SYSTEM hive names its own; another hive takes the name of the
/// SYSTEM hive of its system: `Windows\System32\config\SYSTEM` under the directory that holds
/// its `Windows` or `Users` directory (a collection keeps a system's files as they were), else
/// a `SYSTEM` hive next to it (hives exported to one directory). Without one, the host is
/// unknown: a hive is never given the name of a system it may not come from.
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

/// The directory of the system a file at `path` comes from: the one holding the `Windows`
/// directory (`Windows\System32\…`, `Windows\AppCompat\…`, `Windows\ServiceProfiles\…`) or the
/// `Users` directory (`Users\<name>\…`, also `Documents and Settings`) nearest the file.
fn system_root(path: &Path) -> Option<PathBuf> {
    let parts: Vec<_> = path.components().collect();
    let name = |i: usize| {
        parts
            .get(i)
            .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
            .unwrap_or_default()
    };
    (0..parts.len()).rev().find_map(|i| {
        let windows = name(i) == "windows"
            && ["system32", "appcompat", "serviceprofiles"].contains(&name(i + 1).as_str());
        let users =
            ["users", "documents and settings"].contains(&name(i).as_str()) && i + 2 < parts.len();
        (windows || users).then(|| parts[..i].iter().collect())
    })
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
                "/case/H1/C/Documents and Settings/bob/NTUSER.DAT",
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
        ] {
            assert_eq!(root(path), Some(PathBuf::from(expected)), "{path}");
        }
        assert_eq!(root("hives/NTUSER.DAT"), None);
    }
}
