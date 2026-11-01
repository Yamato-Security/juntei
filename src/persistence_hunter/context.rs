//! Offline evidence belonging to one computer. Nothing here reads the analyst's registry,
//! environment variables or executable search path.

use crate::registry::{Logs, Opened};
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

const MAX_HIVE_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_DEPTH: usize = 128;

pub struct HiveInput {
    pub opened: Opened,
    pub kind: String,
    pub user: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct EvidenceFile {
    pub logical: String,
    pub physical: PathBuf,
    pub len: u64,
    pub(super) volume: char,
}

#[derive(Debug, Clone)]
pub enum Resolution {
    Found(EvidenceFile),
    NotCollected,
    Invalid(String),
    Ambiguous(Vec<PathBuf>),
}

pub struct Context {
    pub root: PathBuf,
    pub computer: String,
    pub(super) service_event_correlation: bool,
    pub hives: Vec<HiveInput>,
    pub files: Vec<EvidenceFile>,
    /// Collected directories, including empty CLSID-named Start Menu folders.
    /// Kept separate so file-backed parsers never read a directory as a file.
    pub(super) directories: Vec<EvidenceFile>,
    pub issues: Vec<String>,
    index: HashMap<String, Vec<usize>>,
    drive_index: HashMap<(char, String), Vec<usize>>,
    source_drives: HashMap<PathBuf, char>,
    root_drive: char,
    system_volume: OnceLock<char>,
    original_drive: OnceLock<char>,
    pub(super) metadata_cache: Mutex<HashMap<String, Arc<OnceLock<super::resolver::Metadata>>>>,
}

#[derive(Default)]
struct Inventory {
    paths: Vec<PathBuf>,
    directories: Vec<PathBuf>,
    issues: Vec<String>,
}

fn inventory_directory(directory: &Path, depth: usize) -> Inventory {
    let mut inventory = Inventory::default();
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(e) => {
            inventory
                .issues
                .push(format!("Cannot enumerate {}: {e}", directory.display()));
            return inventory;
        }
    };
    let mut directories = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                inventory
                    .issues
                    .push(format!("Cannot enumerate {}: {e}", directory.display()));
                continue;
            }
        };
        let path = entry.path();
        let kind = match entry.file_type() {
            Ok(kind) => kind,
            Err(e) => {
                inventory
                    .issues
                    .push(format!("Cannot inspect {}: {e}", path.display()));
                continue;
            }
        };
        if kind.is_symlink() {
            inventory.issues.push(format!(
                "Not following evidence symlink: {}",
                path.display()
            ));
        } else if kind.is_dir() {
            inventory.directories.push(path.clone());
            if depth < MAX_DEPTH {
                directories.push(path);
            } else {
                inventory
                    .issues
                    .push(format!("Directory depth limit reached: {}", path.display()));
            }
        } else if kind.is_file() {
            inventory.paths.push(path);
        }
    }
    // Preserve the old depth-first stack order for diagnostics. Do not create a
    // global pool when Context is used by metadata discovery outside this command.
    directories.reverse();
    let children: Vec<_> = if rayon::current_thread_index().is_some() {
        directories
            .par_iter()
            .map(|dir| inventory_directory(dir, depth + 1))
            .collect()
    } else {
        directories
            .iter()
            .map(|dir| inventory_directory(dir, depth + 1))
            .collect()
    };
    for mut child in children {
        inventory.paths.append(&mut child.paths);
        inventory.directories.append(&mut child.directories);
        inventory.issues.append(&mut child.issues);
    }
    inventory
}

impl Context {
    pub fn open(root: &Path, no_logs: bool) -> Result<Self, String> {
        Self::open_with_budget(root, no_logs, MAX_HIVE_BYTES)
    }

    pub(super) fn open_with_budget(
        root: &Path,
        no_logs: bool,
        budget: u64,
    ) -> Result<Self, String> {
        let input = fs::canonicalize(root).map_err(|e| format!("{}: {e}", root.display()))?;
        let standalone = input.is_file();
        let root = if standalone {
            input
                .parent()
                .ok_or("hive has no parent directory")?
                .to_path_buf()
        } else {
            input.clone()
        };
        let mut issues = Vec::new();
        let mut paths = Vec::new();
        let mut directory_paths = Vec::new();
        if standalone {
            paths.push(input);
        } else {
            let inventory = inventory_directory(&root, 0);
            paths = inventory.paths;
            directory_paths = inventory.directories;
            issues = inventory.issues;
        }
        paths.sort();
        let mut files = Vec::with_capacity(paths.len());
        let mut index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut drive_index: HashMap<(char, String), Vec<usize>> = HashMap::new();
        // A mount point called "windows" may contain a full volume. Inspect its
        // children instead of trusting the analyst's choice of directory name.
        let windows_root = paths.iter().any(|p| {
            p.strip_prefix(&root)
                .ok()
                .and_then(|p| p.components().next())
                .is_some_and(|p| {
                    ["System32", "Prefetch", "AppCompat"]
                        .iter()
                        .any(|n| p.as_os_str().to_string_lossy().eq_ignore_ascii_case(n))
                })
        }) && !paths.iter().any(|p| {
            p.strip_prefix(&root)
                .ok()
                .and_then(|p| p.components().next())
                .is_some_and(|p| {
                    p.as_os_str()
                        .to_string_lossy()
                        .eq_ignore_ascii_case("Windows")
                })
        });
        let flat_exports = paths.iter().any(|p| {
            p.strip_prefix(&root).ok().is_some_and(|p| {
                p.components().count() == 1
                    && ["SYSTEM", "SOFTWARE", "SAM", "SECURITY", "DEFAULT"]
                        .iter()
                        .any(|n| p.to_string_lossy().eq_ignore_ascii_case(n))
            })
        });
        let windows_root = windows_root && !flat_exports;
        let root_drive = root
            .file_name()
            .and_then(|p| {
                let p = decode_component(&p.to_string_lossy());
                drive_component(&p).or_else(|| {
                    (p.len() == 1 && p.as_bytes()[0].is_ascii_alphabetic())
                        .then(|| (p.as_bytes()[0] as char).to_ascii_uppercase())
                })
            })
            .unwrap_or('C');
        let describe = |physical: &PathBuf| {
            let relative = physical.strip_prefix(&root).unwrap_or(physical);
            let logical = if windows_root {
                format!("Windows/{}", logical_path(relative))
            } else {
                logical_path(relative)
            };
            fs::metadata(physical)
                .map(|metadata| EvidenceFile {
                    logical,
                    physical: physical.clone(),
                    len: metadata.len(),
                    volume: collection_drive(relative).unwrap_or(root_drive),
                })
                .map_err(|e| format!("Cannot inspect {}: {e}", physical.display()))
        };
        let described: Vec<_> = if rayon::current_thread_index().is_some() {
            paths.par_iter().map(describe).collect()
        } else {
            paths.iter().map(describe).collect()
        };
        for file in described {
            let file = match file {
                Ok(file) => file,
                Err(issue) => {
                    issues.push(issue);
                    continue;
                }
            };
            index
                .entry(file.logical.to_lowercase())
                .or_default()
                .push(files.len());
            let drive = file.volume;
            drive_index
                .entry((drive, file.logical.to_lowercase()))
                .or_default()
                .push(files.len());
            files.push(file);
        }

        // Only canonical locations (or explicitly supplied hive exports) establish
        // identity. Sort canonical copies ahead of flat exports, never by an arbitrary
        // path containing Users/.../C/Windows, and try a valid fallback if parsing fails.
        let mut selected: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, file) in files.iter().enumerate() {
            if let Some((kind, user)) = identity_of(file, standalone) {
                selected
                    .entry(format!("{}\0{kind}", user.to_ascii_lowercase()))
                    .or_default()
                    .push(i);
            }
        }
        let mut hives = Vec::new();
        let mut input_bytes = 0u64;
        for candidates in selected.values_mut() {
            candidates.sort_by_key(|i| {
                let f = &files[*i];
                (
                    !f.logical.contains('/'),
                    f.volume != root_drive,
                    f.physical.clone(),
                )
            });
            let mut chosen: Option<usize> = None;
            for &i in candidates.iter() {
                let file = &files[i];
                if let Some(previous) = chosen {
                    issues.push(format!(
                        "Multiple copies of {}; using {} instead of {}",
                        file.logical,
                        files[previous].physical.display(),
                        file.physical.display()
                    ));
                    continue;
                }
                let Some((kind, user)) = identity_of(file, standalone) else {
                    continue;
                };
                if file.len > budget.saturating_sub(input_bytes) {
                    issues.push(format!("Skipped {kind} hive {}: {} bytes exceed remaining per-computer input memory budget of {} bytes; other hives and file checks are retained", file.physical.display(), file.len, budget.saturating_sub(input_bytes)));
                    continue;
                }
                let mut cost = file.len;
                let mut logs = Vec::new();
                if !no_logs {
                    for log in adjacent_logs(&file.physical, &mut issues) {
                        match fs::metadata(&log) {
                            Ok(m) if m.len() <= budget.saturating_sub(input_bytes).saturating_sub(cost) => {
                                cost += m.len(); logs.push(log);
                            }
                            Ok(_) => issues.push(format!("Skipped transaction log {}: per-computer input memory budget reached; hive retained without this log", log.display())),
                            Err(e) => issues.push(format!("Cannot inspect transaction log {}: {e}", log.display())),
                        }
                    }
                }
                match crate::registry::open(&file.physical, Logs::Given(&logs)) {
                    Ok(opened) => {
                        if !opened.hive_type.is_empty()
                            && !opened.hive_type.eq_ignore_ascii_case(&kind)
                        {
                            issues.push(format!("Hive type mismatch: {} is stored as {kind} but its recorded type is {}; not scanning it under the wrong hive", file.physical.display(), opened.hive_type));
                            continue;
                        }
                        if opened.recovery.dirty && !opened.recovery.recovered {
                            issues.push(format!(
                                "Dirty hive without applicable transaction-log replay: {}",
                                file.physical.display()
                            ));
                        }
                        input_bytes += cost;
                        hives.push(HiveInput {
                            opened,
                            kind,
                            user,
                            path: file.physical.clone(),
                        });
                        chosen = Some(i);
                    }
                    Err(e) => issues.push(format!(
                        "Cannot parse hive {}: {e}",
                        file.physical.display()
                    )),
                }
            }
        }
        let computer = hives
            .iter()
            .find(|h| h.kind == "SYSTEM")
            .and_then(|h| registry::system::computer_name(&h.opened.hive))
            .or_else(|| client_computer(&root))
            .unwrap_or_else(|| {
                root.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .split("_F.")
                    .next()
                    .unwrap_or("UNKNOWN")
                    .to_string()
            });
        let source_drives = files
            .iter()
            .map(|f| (f.physical.clone(), f.volume))
            .collect();
        directory_paths.sort();
        let directories = directory_paths
            .iter()
            .filter_map(|path| match describe(path) {
                Ok(directory) => Some(directory),
                Err(e) => {
                    issues.push(e);
                    None
                }
            })
            .collect();
        Ok(Self {
            root,
            computer,
            service_event_correlation: true,
            hives,
            files,
            directories,
            issues,
            index,
            drive_index,
            source_drives,
            root_drive,
            system_volume: OnceLock::new(),
            original_drive: OnceLock::new(),
            metadata_cache: Mutex::new(HashMap::new()),
        })
    }

    pub fn machine_hive(&self, kind: &str) -> Option<&HiveInput> {
        self.hives
            .iter()
            .find(|h| h.user.is_empty() && h.kind.eq_ignore_ascii_case(kind))
    }

    pub fn user_for_source(&self, source: &str) -> Option<&str> {
        // Inventory paths originate under the canonical evidence root. A caller may still
        // use its original spelling (/var instead of /private/var, a Windows path without
        // the verbatim prefix, or an explicitly supplied directory alias).
        let source = self.canonical_source(Path::new(source))?;
        if let Some(hive) = self
            .hives
            .iter()
            .find(|h| h.path == source && !h.user.is_empty())
        {
            return Some(&hive.user);
        }
        let file = self.files.iter().find(|file| file.physical == source)?;
        let mut parts = file.logical.split('/');
        while let Some(part) = parts.next() {
            if ["Users", "Documents and Settings", "ServiceProfiles"]
                .iter()
                .any(|name| part.eq_ignore_ascii_case(name))
            {
                return parts.next();
            }
        }
        if let Some(software) = self.machine_hive("SOFTWARE") {
            for (_, profile) in registry::software::profile_list(&software.opened.hive) {
                let Ok(profile) = normalize_windows(&profile) else {
                    continue;
                };
                if file
                    .logical
                    .to_lowercase()
                    .starts_with(&(profile.to_lowercase() + "/"))
                {
                    return file.logical.split('/').nth(profile.split('/').count() - 1);
                }
            }
        }
        None
    }

    fn canonical_source(&self, source: &Path) -> Option<PathBuf> {
        let source = fs::canonicalize(source).ok()?;
        source.starts_with(&self.root).then_some(source)
    }

    pub(super) fn drive_of(&self, path: &Path) -> char {
        // Inventoried paths are already canonical. Avoid network-filesystem stats
        // for every payload; canonicalize only external spellings such as aliases.
        self.source_drives
            .get(path)
            .copied()
            .or_else(|| {
                self.canonical_source(path)
                    .and_then(|p| self.source_drives.get(&p).copied())
            })
            .unwrap_or(self.root_drive)
    }

    /// Captured volume letter; it can differ from the original host's system drive.
    pub(super) fn system_drive(&self) -> char {
        *self.system_volume.get_or_init(|| {
            self.machine_hive("SYSTEM")
                .or_else(|| self.machine_hive("SOFTWARE"))
                .map(|h| self.drive_of(&h.path))
                .or_else(|| {
                    self.files
                        .iter()
                        .find(|f| f.logical.to_ascii_lowercase().starts_with("windows/"))
                        .map(|f| f.volume)
                })
                .unwrap_or(self.root_drive)
        })
    }

    pub(super) fn original_system_drive(&self) -> char {
        *self.original_drive.get_or_init(|| {
            self.machine_hive("SOFTWARE")
                .and_then(|h| h.opened.hive.key(r"Microsoft\Windows NT\CurrentVersion"))
                .and_then(|key| key.value("SystemRoot"))
                .and_then(
                    |v| match registry::decode(v.data_type(), &v.read_data().0) {
                        registry::Data::Text { value, .. } => explicit_drive(&value),
                        _ => None,
                    },
                )
                .unwrap_or_else(|| self.system_drive())
        })
    }

    pub(super) fn captured_user_profile(&self, user: &str) -> Option<String> {
        let hive = self
            .hives
            .iter()
            .find(|h| h.kind == "NTUSER.DAT" && h.user.eq_ignore_ascii_case(user))?;
        let file = self.files.iter().find(|f| f.physical == hive.path)?;
        let (parent, _) = file.logical.rsplit_once('/')?;
        let drive = if file.volume == self.system_drive() {
            self.original_system_drive()
        } else {
            file.volume
        };
        Some(format!("{drive}:\\{}", parent.replace('/', "\\")))
    }

    /// Registry/client identity is independent of analyst-chosen evidence folder names.
    pub(super) fn host_identities(&self) -> Vec<(String, String)> {
        let mut names = Vec::new();
        if let Some(h) = self.machine_hive("SYSTEM") {
            let set = self.active_controlset();
            for (key, field) in [
                (
                    format!(r"{set}\Control\ComputerName\ComputerName"),
                    "ComputerName",
                ),
                (
                    format!(r"{set}\Control\ComputerName\ActiveComputerName"),
                    "ComputerName",
                ),
                (format!(r"{set}\Services\Tcpip\Parameters"), "Hostname"),
                (format!(r"{set}\Services\Tcpip\Parameters"), "NV Hostname"),
            ] {
                if let Some(v) = h.opened.hive.key(&key).and_then(|k| k.value(field))
                    && let registry::Data::Text { value, .. } =
                        registry::decode(v.data_type(), &v.read_data().0)
                    && !value.trim().is_empty()
                {
                    names.push((value, format!("captured SYSTEM {key} {field}")));
                }
            }
        }
        names.extend(
            client_names(&self.root)
                .into_iter()
                .map(|(name, field)| (name, format!("captured client_info.json {field}"))),
        );
        names
    }

    pub(super) fn captured_drive(&self, original: char) -> Option<char> {
        let host = self.original_system_drive();
        let captured = self.system_drive();
        if original == host {
            Some(captured)
        } else if original == captured && host != captured {
            None
        } else {
            Some(original)
        }
    }

    pub fn active_controlset(&self) -> String {
        self.machine_hive("SYSTEM")
            .and_then(|h| registry::system::ControlSets::read(&h.opened.hive).current_or_first())
            .map(registry::system::control_set_name)
            .unwrap_or_else(|| "ControlSet001".to_string())
    }

    pub fn resolve(&self, windows_path: &str) -> Resolution {
        let normalized = match normalize_windows(windows_path) {
            Ok(path) => path,
            Err(e) => return Resolution::Invalid(e),
        };
        let key = normalized.to_lowercase();
        let requested_drive = match explicit_drive(windows_path) {
            Some(drive) => match self.captured_drive(drive) {
                Some(drive) => Some(drive),
                None => return Resolution::NotCollected,
            },
            None => None,
        };
        let lookup = |key: &str| match requested_drive {
            Some(drive) => self.drive_index.get(&(drive, key.to_string())),
            None => self.index.get(key),
        };
        let candidates = lookup(&key).or_else(|| {
            (!key.contains('/'))
                .then(|| {
                    [format!("windows/system32/{key}"), format!("windows/{key}")]
                        .iter()
                        .find_map(|candidate| lookup(candidate))
                })
                .flatten()
        });
        match candidates {
            None => Resolution::NotCollected,
            Some(ids) if ids.len() == 1 => {
                let file = &self.files[ids[0]];
                // Check again when used: evidence can have changed after inventory.
                match fs::canonicalize(&file.physical) {
                    Ok(real) if real.starts_with(&self.root) => Resolution::Found(file.clone()),
                    Ok(_) => Resolution::Invalid("resolved file escapes evidence root".to_string()),
                    Err(e) => Resolution::Invalid(format!("cannot resolve evidence file: {e}")),
                }
            }
            Some(ids) => Resolution::Ambiguous(
                ids.iter()
                    .map(|i| self.files[*i].physical.clone())
                    .collect(),
            ),
        }
    }
}

fn adjacent_logs(hive: &Path, issues: &mut Vec<String>) -> Vec<PathBuf> {
    let Some(parent) = hive.parent() else {
        return Vec::new();
    };
    let name = hive
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_uppercase();
    let wanted = [
        format!("{name}.LOG"),
        format!("{name}.LOG1"),
        format!("{name}.LOG2"),
    ];
    let Ok(entries) = fs::read_dir(parent) else {
        issues.push(format!(
            "Cannot enumerate transaction logs next to {}",
            hive.display()
        ));
        return Vec::new();
    };
    let mut logs = Vec::new();
    for entry in entries
        .flatten()
        .filter(|entry| wanted.contains(&entry.file_name().to_string_lossy().to_uppercase()))
    {
        match entry.file_type() {
            Ok(kind) if kind.is_file() => logs.push(entry.path()),
            _ => issues.push(format!(
                "Not opening non-regular transaction log: {}",
                entry.path().display()
            )),
        }
    }
    logs.sort();
    logs
}

fn drive_component(component: &str) -> Option<char> {
    let last = component.rsplit(['\\', '/']).next()?;
    (last.len() == 2 && last.as_bytes()[0].is_ascii_alphabetic() && last.as_bytes()[1] == b':')
        .then(|| (last.as_bytes()[0] as char).to_ascii_uppercase())
}

fn volume_root_name(name: &str) -> bool {
    [
        "Windows",
        "Users",
        "Program Files",
        "Program Files (x86)",
        "ProgramData",
        "$MFT",
        "$Extend",
    ]
    .iter()
    .any(|root| name.eq_ignore_ascii_case(root))
}

pub(super) fn explicit_drive(path: &str) -> Option<char> {
    split_drive(path).map(|(drive, _)| drive)
}

/// Keep the drive and remainder relative to the same normalized string. Offsets
/// into a trimmed/prefix-stripped path cannot be applied to the original input.
pub(super) fn split_drive(path: &str) -> Option<(char, &str)> {
    let mut path = path.trim().trim_matches(['\'', '"']);
    for prefix in ["\\\\?\\", "\\??\\", "\\\\.\\"] {
        if let Some(rest) = path.strip_prefix(prefix) {
            path = rest;
            break;
        }
    }
    let bytes = path.as_bytes();
    (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
        .then(|| ((bytes[0] as char).to_ascii_uppercase(), &path[2..]))
}

fn identity_of(file: &EvidenceFile, standalone: bool) -> Option<(String, String)> {
    if standalone && let Some(kind) = crate::registry::hive_type_of_file(&file.physical) {
        let user = if matches!(kind.as_str(), "NTUSER.DAT" | "USRCLASS.DAT") {
            "standalone_user"
        } else {
            ""
        };
        return Some((kind, user.to_string()));
    }
    hive_identity(&file.logical, standalone)
}

fn client_names_inner(root: &Path) -> Option<Vec<(String, String)>> {
    let path = root.join("client_info.json");
    let metadata = fs::symlink_metadata(&path).ok()?;
    if !metadata.is_file()
        || metadata.len() > 1024 * 1024
        || !fs::canonicalize(&path).ok()?.starts_with(root)
    {
        return None;
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .ok()?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > 1024 * 1024 {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    Some(
        ["hostname", "fqdn"]
            .into_iter()
            .filter_map(|field| {
                value
                    .get(field)?
                    .as_str()
                    .filter(|name| !name.trim().is_empty())
                    .map(|name| (name.to_string(), field.to_string()))
            })
            .collect(),
    )
}

fn client_names(root: &Path) -> Vec<(String, String)> {
    client_names_inner(root).unwrap_or_default()
}
fn client_computer(root: &Path) -> Option<String> {
    client_names(root).into_iter().next().map(|(name, _)| name)
}

fn hive_identity(logical: &str, standalone: bool) -> Option<(String, String)> {
    let components: Vec<_> = logical.split('/').collect();
    let name = components.last()?.to_uppercase();
    let machine = ["SYSTEM", "SOFTWARE", "SAM", "SECURITY", "DEFAULT"];
    if machine.contains(&name.as_str())
        && (components.len() == 1
            || logical
                .to_lowercase()
                .starts_with("windows/system32/config/")
                && components.len() == 4)
    {
        return Some((name, String::new()));
    }
    if matches!(name.as_str(), "NTUSER.DAT" | "USRCLASS.DAT") {
        if standalone || components.len() == 1 {
            return Some((name, "standalone_user".into()));
        }
        let lower: Vec<_> = components.iter().map(|s| s.to_ascii_lowercase()).collect();
        let user_at = if matches!(lower[0].as_str(), "users" | "documents and settings") {
            Some(1)
        } else if lower.starts_with(&["windows".into(), "serviceprofiles".into()]) {
            Some(2)
        } else if lower.starts_with(&["windows".into(), "system32".into(), "config".into()])
            && lower.get(3).is_some_and(|s| s == "systemprofile")
        {
            Some(3)
        } else {
            None
        };
        if let Some(at) = user_at {
            let tail = &lower[at + 1..];
            let canonical = if name == "NTUSER.DAT" {
                tail == ["ntuser.dat"]
            } else {
                tail == ["appdata", "local", "microsoft", "windows", "usrclass.dat"]
            };
            if canonical {
                return Some((name, components[at].to_string()));
            }
        }
    }
    None
}

/// Decode collection filename escaping once. `%254` represents a literal `%4`, so a
/// repeated URL-decode would silently change the evidence path.
fn decode_component(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'%'
            && at + 2 < bytes.len()
            && let (Some(a), Some(b)) = (
                (bytes[at + 1] as char).to_digit(16),
                (bytes[at + 2] as char).to_digit(16),
            )
        {
            out.push((a * 16 + b) as u8);
            at += 3;
        } else {
            out.push(bytes[at]);
            at += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn collection_parts(relative: &Path) -> Vec<String> {
    relative
        .components()
        .filter_map(|part| match part {
            Component::Normal(part) => Some(decode_component(&part.to_string_lossy())),
            _ => None,
        })
        .collect()
}

/// Drive prefixes are only meaningful at the root of known collection layouts.
/// A nested Users/Public/.../C/Windows directory is ordinary evidence, not a volume.
fn volume_prefix(parts: &[String]) -> Option<(usize, char)> {
    let at = if parts.first()?.eq_ignore_ascii_case("uploads") {
        if !parts.get(1).is_some_and(|s| {
            ["auto", "ntfs", "file", "lazy_ntfs"]
                .iter()
                .any(|a| s.eq_ignore_ascii_case(a))
        }) {
            return None;
        }
        2
    } else {
        0
    };
    let part = parts.get(at)?;
    let drive = drive_component(part).or_else(|| {
        (part.len() == 1
            && part.as_bytes()[0].is_ascii_alphabetic()
            && parts.get(at + 1).is_some_and(|next| volume_root_name(next)))
        .then(|| (part.as_bytes()[0] as char).to_ascii_uppercase())
    })?;
    Some((at + 1, drive))
}
fn collection_drive(relative: &Path) -> Option<char> {
    volume_prefix(&collection_parts(relative)).map(|(_, drive)| drive)
}
fn logical_path(relative: &Path) -> String {
    let parts = collection_parts(relative);
    let start = volume_prefix(&parts).map(|(start, _)| start).unwrap_or(0);
    parts[start..].join("/")
}

pub(super) fn normalize_windows(value: &str) -> Result<String, String> {
    let mut path = value.trim().trim_matches(['\'', '"']).replace('\\', "/");
    if path.len() > 131_068 || path.contains('\0') {
        return Err("invalid Windows path length or NUL".to_string());
    }
    for prefix in ["//?/", "/??/", "//./"] {
        if path.starts_with(prefix) {
            path = path[prefix.len()..].to_string();
            break;
        }
    }
    if path.starts_with("//") || path.to_lowercase().starts_with("unc/") {
        return Err("UNC target is outside this computer's collected volume".to_string());
    }
    if path.as_bytes().get(1) == Some(&b':') {
        if path.as_bytes().get(2) != Some(&b'/') {
            return Err("drive-relative path has no captured working directory".to_string());
        }
        path = path[2..].to_string();
    }
    path = path.trim_start_matches('/').to_string();
    let lower = path.to_lowercase();
    for prefix in ["%systemroot%/", "%windir%/", "systemroot/"] {
        if lower.starts_with(prefix) {
            path = format!("Windows/{}", &path[prefix.len()..]);
            break;
        }
    }
    if lower.starts_with("system32/") || lower.starts_with("syswow64/") {
        path = format!("Windows/{path}");
    }
    let mut components = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if components.pop().is_none() {
                    return Err("path escapes the evidence volume".to_string());
                }
            }
            _ if part.encode_utf16().count() > 255 || part.contains(':') => {
                return Err("invalid or alternate-stream Windows path".to_string());
            }
            _ => components.push(part),
        }
    }
    if components.is_empty() {
        return Err("empty file path".to_string());
    }
    Ok(components.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn drive_and_remainder_share_normalized_prefixes() {
        for path in [
            "\u{3000}d:\\Users\\fixture",
            r#""d:\Users\fixture""#,
            r"'d:\Users\fixture'",
            r"\\?\d:\Users\fixture",
            r"\??\d:\Users\fixture",
            r"\\.\d:\Users\fixture",
        ] {
            assert_eq!(
                split_drive(path),
                Some(('D', r"\Users\fixture")),
                "{path:?}"
            );
            assert_eq!(explicit_drive(path), Some('D'));
        }
        for path in ["", "\u{3000}", "K:", r"Users\fixture", r"\\server\share"] {
            assert_eq!(split_drive(path), None, "{path:?}");
            assert_eq!(explicit_drive(path), None);
        }
    }

    #[test]
    fn collection_paths_decode_once() {
        assert_eq!(
            logical_path(Path::new(
                "uploads/auto/C%3A/Windows/System32/winevt/Logs/Test%254Operational.evtx"
            )),
            "Windows/System32/winevt/Logs/Test%4Operational.evtx"
        );
        assert_eq!(
            logical_path(Path::new("uploads/ntfs/%5C%5C.%5CC%3A/$MFT")),
            "$MFT"
        );
    }
    #[test]
    fn resolver_rejects_escapes_and_keeps_unknown_distinct() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("Windows/System32")).unwrap();
        fs::write(dir.path().join("Windows/System32/A.exe"), b"a").unwrap();
        let context = Context::open(dir.path(), true).unwrap();
        assert!(matches!(
            context.resolve("C:\\WINDOWS\\system32\\a.exe"),
            Resolution::Found(_)
        ));
        assert!(matches!(
            context.resolve("../outside"),
            Resolution::Invalid(_)
        ));
        assert!(matches!(
            context.resolve("C:\\not-collected.exe"),
            Resolution::NotCollected
        ));
        assert!(matches!(
            context.resolve("\\\\server\\share\\x.exe"),
            Resolution::Invalid(_)
        ));
    }
    #[test]
    fn case_collisions_are_not_chosen_arbitrarily() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("A.exe"), b"a").unwrap();
        fs::write(dir.path().join("a.EXE"), b"b").unwrap();
        let context = Context::open(dir.path(), true).unwrap();
        if context.files.len() == 2 {
            assert!(matches!(context.resolve("a.exe"), Resolution::Ambiguous(_)));
        }
    }

    #[test]
    fn windows_directory_input_retains_its_volume_relative_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let windows = dir.path().join("Windows");
        fs::create_dir_all(windows.join("System32")).unwrap();
        fs::write(windows.join("System32/A.exe"), b"a").unwrap();
        let context = Context::open(&windows, true).unwrap();
        assert!(matches!(
            context.resolve("C:\\Windows\\System32\\A.exe"),
            Resolution::Found(_)
        ));
    }

    #[test]
    fn an_uncollected_drive_cannot_resolve_a_file_on_c() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("uploads/auto/C%3A/Windows")).unwrap();
        fs::write(dir.path().join("uploads/auto/C%3A/Windows/tool.exe"), b"c").unwrap();
        let context = Context::open(dir.path(), true).unwrap();
        assert!(matches!(
            context.resolve("C:\\Windows\\tool.exe"),
            Resolution::Found(_)
        ));
        assert!(matches!(
            context.resolve("D:\\Windows\\tool.exe"),
            Resolution::NotCollected
        ));
    }

    #[test]
    fn user_context_is_inferred_from_collected_file_sources() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir
            .path()
            .join("uploads/auto/C%3A/Users/alice/AppData/Local/settings.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"{}").unwrap();
        let context = Context::open(dir.path(), true).unwrap();
        assert_eq!(
            context.user_for_source(&path.to_string_lossy()),
            Some("alice")
        );
        assert_eq!(
            context.user_for_source("/outside/Users/attacker/settings.json"),
            None
        );
    }

    #[cfg(unix)]
    #[test]
    fn source_directory_aliases_keep_captured_identity_and_reject_outside_targets() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("evidence");
        let settings = root.join("D/Users/alice/AppData/Local/settings.json");
        fs::create_dir_all(settings.parent().unwrap()).unwrap();
        fs::write(&settings, b"{}").unwrap();
        let alias = dir.path().join("capture-alias");
        std::os::unix::fs::symlink(&root, &alias).unwrap();
        // The alias includes a misleading C/Users component; drive inference must use
        // the inventoried D path rather than the alias's directory names.
        let drive_alias = dir.path().join("C/Users");
        fs::create_dir_all(drive_alias.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(root.join("D/Users"), &drive_alias).unwrap();
        let context = Context::open(&alias, true).unwrap();
        let aliased_settings = alias.join("D/Users/alice/AppData/Local/settings.json");
        assert_eq!(
            context.user_for_source(&aliased_settings.to_string_lossy()),
            Some("alice")
        );
        assert_eq!(
            context.drive_of(&drive_alias.join("alice/AppData/Local/settings.json")),
            'D'
        );

        let outside = dir.path().join("outside/Users/alice/settings.json");
        fs::create_dir_all(outside.parent().unwrap()).unwrap();
        fs::write(&outside, b"{}").unwrap();
        let linked_outside = root.join("outside-source.json");
        std::os::unix::fs::symlink(&outside, &linked_outside).unwrap();
        assert!(
            context
                .user_for_source(&outside.to_string_lossy())
                .is_none()
        );
        assert!(
            context
                .user_for_source(&linked_outside.to_string_lossy())
                .is_none()
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_do_not_read_outside_the_evidence() {
        let evidence = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("hidden.exe"), b"outside").unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("hidden.exe"),
            evidence.path().join("evil.exe"),
        )
        .unwrap();
        let context = Context::open(evidence.path(), true).unwrap();
        assert!(matches!(
            context.resolve("evil.exe"),
            Resolution::NotCollected
        ));
        assert!(context.issues.iter().any(|issue| issue.contains("symlink")));
    }

    #[cfg(unix)]
    #[test]
    fn adjacent_transaction_log_symlinks_are_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("SYSTEM"), b"test").unwrap();
        std::os::unix::fs::symlink("/etc/passwd", dir.path().join("SYSTEM.LOG1")).unwrap();
        let mut issues = Vec::new();
        assert!(adjacent_logs(&dir.path().join("SYSTEM"), &mut issues).is_empty());
        assert!(
            issues
                .iter()
                .any(|issue| issue.contains("non-regular transaction log"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn client_metadata_symlinks_are_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        fs::write(
            external.path().join("client_info.json"),
            br#"{"hostname":"OUTSIDE"}"#,
        )
        .unwrap();
        std::os::unix::fs::symlink(
            external.path().join("client_info.json"),
            dir.path().join("client_info.json"),
        )
        .unwrap();
        assert!(client_computer(dir.path()).is_none());
        let ctx = Context::open(dir.path(), true).unwrap();
        assert_ne!(ctx.computer, "OUTSIDE");
    }

    #[test]
    fn standalone_header_type_wins_and_structured_type_mismatches_are_visible() {
        use std::io::Read;
        let mut bytes = Vec::new();
        flate2::read::GzDecoder::new(
            std::fs::File::open(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/shimcache/SYSTEM.gz"),
            )
            .unwrap(),
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let standalone = dir.path().join("SOFTWARE");
        fs::write(&standalone, &bytes).unwrap();
        let context = Context::open(&standalone, true).unwrap();
        assert!(context.machine_hive("SYSTEM").is_some());
        assert!(context.machine_hive("SOFTWARE").is_none());
        let structured = dir.path().join("Windows/System32/config/SOFTWARE");
        fs::create_dir_all(structured.parent().unwrap()).unwrap();
        fs::write(structured, bytes).unwrap();
        let context = Context::open(&dir.path().join("Windows"), true).unwrap();
        assert!(context.machine_hive("SOFTWARE").is_none());
        assert!(
            context
                .issues
                .iter()
                .any(|issue| issue.contains("Hive type mismatch"))
        );
    }
}
