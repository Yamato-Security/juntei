//! `parse-registry`: registry hives (SYSTEM, SOFTWARE, SAM, SECURITY, NTUSER.DAT, UsrClass.dat,
//! Amcache.hve, …) read by ys-registry-parser, one subcommand per artifact. What they share is
//! here: finding the hives of a collection, opening each with its transaction logs replayed when
//! it is dirty, naming its type and the host it comes from, and writing CSV, JSON or JSONL.

pub mod persistence;
pub mod read;
pub mod recentdocs;
pub mod sam;
pub mod services;
pub mod taskcache;

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
/// it, the logs given (`--log1`, `--log2`), and the SYSTEM and SOFTWARE hives of its system with
/// their logs (read for its host, and by `persistence` for the machine's classes and profiles).
pub fn files_read(files: &[PathBuf], given_logs: &[PathBuf]) -> Vec<PathBuf> {
    let mut hives: Vec<PathBuf> = files.to_vec();
    for path in files {
        let dir = path
            .parent()
            .filter(|d| !d.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        for name in ["SYSTEM", "SOFTWARE"] {
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

/// Where the records go: the file of `-o` (created, with its directory), stdout (`-t` without
/// `-o`), or nowhere (the summary only). An existing `-o` that is one of the files `reads` gives
/// (the same file by path, symbolic link or hard link) is refused, even with `-C`, before
/// anything is written.
pub fn sink(
    output: Option<&PathBuf>,
    clobber: bool,
    data_on_stdout: bool,
    reads: impl FnOnce() -> Vec<PathBuf>,
) -> Result<Option<Box<dyn Write>>, String> {
    let Some(path) = output else {
        return Ok(data_on_stdout.then(|| Box::new(BufWriter::new(io::stdout())) as Box<dyn Write>));
    };
    if let Ok(out) = same_file::Handle::from_path(path)
        && let Some(input) = reads()
            .into_iter()
            .find(|r| same_file::Handle::from_path(r).is_ok_and(|h| h == out))
    {
        let also = if input == *path {
            String::new()
        } else {
            format!(" (the same file as {})", input.display())
        };
        return Err(format!(
            "The output file is also an input file and cannot be overwritten: {}{also}",
            path.display()
        ));
    }
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
