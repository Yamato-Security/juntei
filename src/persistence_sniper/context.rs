//! Offline evidence belonging to one computer. Nothing here reads the analyst's registry,
//! environment variables or executable search path.

use crate::registry::{Logs, Opened};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

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
    pub hives: Vec<HiveInput>,
    pub files: Vec<EvidenceFile>,
    pub issues: Vec<String>,
    index: HashMap<String, Vec<usize>>,
    drive_index: HashMap<(char, String), Vec<usize>>,
    pub(super) metadata_cache: Mutex<HashMap<String, super::resolver::Metadata>>,
}

impl Context {
    pub fn open(root: &Path, no_logs: bool) -> Result<Self, String> {
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
        if standalone {
            paths.push(input);
        } else {
            let mut pending = vec![(root.clone(), 0usize)];
            while let Some((directory, depth)) = pending.pop() {
                let entries = match fs::read_dir(&directory) {
                    Ok(entries) => entries,
                    Err(e) => {
                        issues.push(format!("Cannot enumerate {}: {e}", directory.display()));
                        continue;
                    }
                };
                for entry in entries {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(e) => {
                            issues.push(format!("Cannot enumerate {}: {e}", directory.display()));
                            continue;
                        }
                    };
                    let path = entry.path();
                    let kind = match entry.file_type() {
                        Ok(kind) => kind,
                        Err(e) => {
                            issues.push(format!("Cannot inspect {}: {e}", path.display()));
                            continue;
                        }
                    };
                    if kind.is_symlink() {
                        issues.push(format!(
                            "Not following evidence symlink: {}",
                            path.display()
                        ));
                    } else if kind.is_dir() {
                        if depth < MAX_DEPTH {
                            pending.push((path, depth + 1));
                        } else {
                            issues
                                .push(format!("Directory depth limit reached: {}", path.display()));
                        }
                    } else if kind.is_file() {
                        paths.push(path);
                    }
                }
            }
        }
        paths.sort();
        let mut files = Vec::with_capacity(paths.len());
        let mut index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut drive_index: HashMap<(char, String), Vec<usize>> = HashMap::new();
        let windows_root = root
            .file_name()
            .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("Windows"));
        for physical in paths {
            let relative = physical.strip_prefix(&root).unwrap_or(&physical);
            let logical = if windows_root {
                format!("Windows/{}", logical_path(relative))
            } else {
                logical_path(relative)
            };
            let len = match fs::metadata(&physical) {
                Ok(meta) => meta.len(),
                Err(e) => {
                    issues.push(format!("Cannot inspect {}: {e}", physical.display()));
                    continue;
                }
            };
            index
                .entry(logical.to_lowercase())
                .or_default()
                .push(files.len());
            let drive = path_drive(&physical).unwrap_or('C');
            drive_index
                .entry((drive, logical.to_lowercase()))
                .or_default()
                .push(files.len());
            files.push(EvidenceFile {
                logical,
                physical,
                len,
            });
        }

        // Prefer the live config directory to RegBack/Windows.old. When an exported-hive
        // directory is supplied, named hives at its root remain usable.
        let mut selected: BTreeMap<String, usize> = BTreeMap::new();
        for (i, file) in files.iter().enumerate() {
            if let Some((kind, user)) = identity_of(file, standalone) {
                let identity = format!("{kind}\0{user}");
                if let Some(previous) = selected.get(&identity) {
                    issues.push(format!(
                        "Multiple copies of {kind}{}; using {} instead of {}",
                        if user.is_empty() {
                            String::new()
                        } else {
                            format!(" for {user}")
                        },
                        files[*previous].physical.display(),
                        file.physical.display()
                    ));
                } else {
                    selected.insert(identity, i);
                }
            }
        }
        let hive_bytes = selected
            .values()
            .try_fold(0u64, |total, i| total.checked_add(files[*i].len))
            .ok_or("selected hive sizes overflowed the size counter")?;
        if hive_bytes > MAX_HIVE_BYTES {
            return Err(format!(
                "The selected hives total {hive_bytes} bytes, exceeding the {MAX_HIVE_BYTES}-byte per-computer memory budget; scan fewer user hives together"
            ));
        }
        let mut hive_inputs = Vec::new();
        let mut input_bytes = hive_bytes;
        for i in selected.values() {
            let logs = if no_logs {
                Vec::new()
            } else {
                adjacent_logs(&files[*i].physical, &mut issues)
            };
            for log in &logs {
                let size = fs::metadata(log)
                    .map_err(|e| format!("{}: {e}", log.display()))?
                    .len();
                input_bytes = input_bytes
                    .checked_add(size)
                    .ok_or("registry input sizes overflowed the size counter")?;
            }
            if input_bytes > MAX_HIVE_BYTES {
                return Err(format!(
                    "Hives and transaction logs total {input_bytes} bytes, exceeding the {MAX_HIVE_BYTES}-byte per-computer input memory budget"
                ));
            }
            hive_inputs.push((*i, logs));
        }
        let mut hives = Vec::new();
        for (i, logs) in hive_inputs {
            let file = &files[i];
            let Some((kind, user)) = identity_of(file, standalone) else {
                continue;
            };
            match crate::registry::open(&file.physical, Logs::Given(&logs)) {
                Ok(opened) => {
                    if !opened.hive_type.is_empty() && !opened.hive_type.eq_ignore_ascii_case(&kind)
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
                    hives.push(HiveInput {
                        opened,
                        kind,
                        user,
                        path: file.physical.clone(),
                    });
                }
                Err(e) => issues.push(format!(
                    "Cannot parse hive {}: {e}",
                    file.physical.display()
                )),
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
        Ok(Self {
            root,
            computer,
            hives,
            files,
            issues,
            index,
            drive_index,
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
        let canonical = self.canonical_source(path);
        path_drive(canonical.as_deref().unwrap_or(path)).unwrap_or('C')
    }

    pub(super) fn system_drive(&self) -> char {
        self.machine_hive("SYSTEM")
            .map(|h| self.drive_of(&h.path))
            .or_else(|| {
                self.files
                    .iter()
                    .find(|f| f.logical.to_lowercase().starts_with("windows/"))
                    .map(|f| self.drive_of(&f.physical))
            })
            .unwrap_or_else(|| self.drive_of(&self.root))
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
        let requested_drive = explicit_drive(windows_path);
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

fn path_drive(path: &Path) -> Option<char> {
    let parts: Vec<_> = path
        .components()
        .filter_map(|part| match part {
            Component::Normal(part) => Some(decode_component(&part.to_string_lossy())),
            _ => None,
        })
        .collect();
    parts
        .iter()
        .enumerate()
        .filter_map(|(i, part)| {
            drive_component(part).or_else(|| {
                (part.len() == 1
                    && part.as_bytes()[0].is_ascii_alphabetic()
                    && parts.get(i + 1).is_some_and(|next| volume_root_name(next)))
                .then(|| (part.as_bytes()[0] as char).to_ascii_uppercase())
            })
        })
        .next_back()
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
    let mut path = path.trim().trim_matches(['\'', '"']);
    for prefix in ["\\\\?\\", "\\??\\", "\\\\.\\"] {
        if let Some(rest) = path.strip_prefix(prefix) {
            path = rest;
            break;
        }
    }
    let bytes = path.as_bytes();
    (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
        .then(|| (bytes[0] as char).to_ascii_uppercase())
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

fn client_computer(root: &Path) -> Option<String> {
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
    value
        .get("hostname")?
        .as_str()
        .filter(|name| !name.is_empty())
        .map(str::to_string)
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
        let user = components
            .windows(2)
            .find(|p| {
                p[0].eq_ignore_ascii_case("Users")
                    || p[0].eq_ignore_ascii_case("Documents and Settings")
                    || p[0].eq_ignore_ascii_case("ServiceProfiles")
            })
            .map(|p| p[1].to_string())
            .or_else(|| {
                components
                    .iter()
                    .position(|part| part.eq_ignore_ascii_case("AppData"))
                    .and_then(|at| at.checked_sub(1))
                    .map(|at| components[at].to_string())
            })
            .or_else(|| {
                (name == "NTUSER.DAT" && components.len() > 1)
                    .then(|| components[components.len() - 2].to_string())
            })
            .unwrap_or_else(|| "standalone_user".to_string());
        if standalone || components.len() == 1 || !logical.to_lowercase().contains("windows.old/") {
            return Some((name, user));
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

fn logical_path(relative: &Path) -> String {
    let parts: Vec<_> = relative
        .components()
        .filter_map(|part| match part {
            Component::Normal(part) => Some(decode_component(&part.to_string_lossy())),
            _ => None,
        })
        .collect();
    let drive = parts.iter().enumerate().find_map(|(i, part)| {
        (drive_component(part).is_some()
            || part.len() == 1
                && part.as_bytes()[0].is_ascii_alphabetic()
                && parts.get(i + 1).is_some_and(|next| volume_root_name(next)))
        .then_some(i)
    });
    let start = drive.map(|i| i + 1).unwrap_or(0);
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
