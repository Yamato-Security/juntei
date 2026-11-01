//! File-backed persistence checks. Adapted from PyrsistenceSniper (MIT).
//! Collection paths are resolved through Context; never through the analyst's OS.

mod accessibility;
mod carve;
mod start_menu;
mod tasks;
mod text;

use super::checks::CheckDefinition;
use super::context::{Context, EvidenceFile};
use super::{Coverage, Finding, Scan, TimeEvidence};
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::Read;

use super::checks::FILE_CHECKS;

pub fn run(ctx: &Context, definitions: &[CheckDefinition]) -> Scan {
    let mut scan = Scan::default();
    for definition in definitions
        .iter()
        .filter(|d| FILE_CHECKS.contains(&d.id.as_str()))
    {
        let mut check = Check {
            ctx,
            definition,
            scan: &mut scan,
            sources: 0,
            issues: Vec::new(),
        };
        match definition.id.as_str() {
            "start_menu_com" => start_menu::run(&mut check),
            "scheduled_task_files" => tasks::scheduled(&mut check),
            "ghost_task" | "hidden_scheduled_task" => tasks::cache(&mut check),
            "wmi_event_subscription" => carve::wmi(&mut check),
            "bits_notify_command" => carve::bits(&mut check),
            "gp_scripts" => text::gp_scripts(&mut check),
            "windows_terminal" => text::terminal(&mut check),
            "startup_folder" | "shell_folders_startup" => startup(&mut check),
            "powershell_profiles" => powershell(&mut check),
            "office_templates" => office(&mut check),
            "power_automate" => power_automate(&mut check),
            "error_handler_cmd" => {
                for file in check.files(|p| p == "windows/system32/errorhandler.cmd") {
                    check.sources += 1;
                    let mut finding = check.file_finding(&file, &file.logical, "SYSTEM");
                    finding.description = "Non-default Windows Error Reporting error-handler script; existence does not prove execution".into();
                    check.scan.findings.push(finding);
                }
            }
            "accessibility_tools" => accessibility::run(&mut check),
            _ => unreachable!(),
        }
        let status = if check.sources == 0 {
            "not_collected"
        } else if check.issues.iter().any(|(status, _)| status != "carving") {
            "partial"
        } else {
            "complete"
        };
        let mut detail = format!(
            "{} applicable source artifacts examined; a sparse collection cannot prove that uncollected files were absent on the source system",
            check.sources
        );
        if !check.issues.is_empty() {
            detail.push_str(&format!(". {} issue(s): ", check.issues.len()));
            detail.push_str(
                &check
                    .issues
                    .iter()
                    .take(20)
                    .map(|(status, detail)| format!("{status}: {detail}"))
                    .collect::<Vec<_>>()
                    .join("; "),
            );
            if check.issues.len() > 20 {
                detail.push_str("; remaining issues omitted from this summary");
            }
        }
        let mut coverage = Coverage::new(&check.definition.id, status, detail);
        coverage.computer.clone_from(&ctx.computer);
        check.scan.coverage.push(coverage);
    }
    scan
}

pub(super) struct Check<'a> {
    ctx: &'a Context,
    definition: &'a CheckDefinition,
    scan: &'a mut Scan,
    sources: usize,
    issues: Vec<(String, String)>,
}

impl Check<'_> {
    fn files(&self, predicate: impl Fn(&str) -> bool) -> Vec<EvidenceFile> {
        self.ctx
            .files
            .iter()
            .filter(|f| predicate(&f.logical.to_lowercase()))
            .cloned()
            .collect()
    }

    fn issue(&mut self, status: &str, detail: impl Into<String>) {
        self.issues.push((status.to_string(), detail.into()));
    }

    fn finding(&self, path: &str, value: &str, access: &str) -> Finding {
        let mut result = Finding::new(
            &self.definition.id,
            &self.definition.technique,
            &self.definition.mitre_id,
            path,
            value,
            access,
        );
        result.computer.clone_from(&self.ctx.computer);
        result.description.clone_from(&self.definition.description);
        result.references = self.definition.references.join(" | ");
        result.record_status = "collected".into();
        result
    }

    fn file_finding(&self, file: &EvidenceFile, value: &str, access: &str) -> Finding {
        let mut result = self.finding(&file.logical.replace('/', "\\"), value, access);
        result.source_file = file.physical.display().to_string();
        result.resolve_target.clone_from(&file.logical);
        result.time_evidence.push(TimeEvidence::FileWrite {
            path: file.logical.clone(),
            weak: false,
        });
        result
    }

    fn read(&mut self, file: &EvidenceFile, max_bytes: u64) -> Option<Vec<u8>> {
        if !self.safe_file(file) {
            return None;
        }
        // Bound both advertised and actual reads (the source may change after discovery).
        if file.len > max_bytes {
            self.issue(
                "limit_reached",
                format!(
                    "{} exceeds the {max_bytes}-byte parser limit",
                    file.physical.display()
                ),
            );
            return None;
        }
        let mut data = Vec::new();
        match File::open(&file.physical).and_then(|f| f.take(max_bytes + 1).read_to_end(&mut data))
        {
            Ok(_) if data.len() as u64 <= max_bytes => Some(data),
            Ok(_) => {
                self.issue(
                    "limit_reached",
                    format!("{} grew beyond the parser limit", file.physical.display()),
                );
                None
            }
            Err(e) => {
                self.issue("unreadable", format!("{}: {e}", file.physical.display()));
                None
            }
        }
    }

    fn safe_file(&mut self, file: &EvidenceFile) -> bool {
        match std::fs::canonicalize(&file.physical) {
            Ok(path) if path.starts_with(&self.ctx.root) => true,
            Ok(_) => {
                self.issue(
                    "unreadable",
                    format!(
                        "{} now resolves outside the evidence root",
                        file.physical.display()
                    ),
                );
                false
            }
            Err(e) => {
                self.issue("unreadable", format!("{}: {e}", file.physical.display()));
                false
            }
        }
    }

    fn folder(&mut self, folder: &str, access: &str, limit: usize, shortcut: bool) {
        let drive = match super::context::explicit_drive(folder) {
            Some(d) => match self.ctx.captured_drive(d) {
                Some(d) => d,
                None => {
                    self.issue(
                        "not_collected",
                        format!("Startup volume not collected: {folder}"),
                    );
                    return;
                }
            },
            None => self.ctx.system_drive(),
        };
        let Ok(folder) = super::context::normalize_windows(folder) else {
            self.issue(
                "unresolved",
                format!("Unsafe or remote folder path: {folder}"),
            );
            return;
        };
        let prefix = format!("{}/", folder.to_lowercase());
        let files = self.files(|p| {
            p.starts_with(&prefix)
                && !p[prefix.len()..].contains('/')
                && !p.ends_with("/desktop.ini")
        });
        let files: Vec<_> = files.into_iter().filter(|f| f.volume == drive).collect();
        if files.is_empty() {
            self.issue("not_collected",format!("No collected files in {folder}; the directory may be empty or omitted from collection"));
        }
        if files.len() > limit {
            self.issue(
                "limit_reached",
                format!(
                    "{folder} contains {} collected files; examining only the first {limit}",
                    files.len()
                ),
            );
        }
        for file in files.into_iter().take(limit) {
            self.sources += 1;
            let mut finding = self.file_finding(&file, &file.logical, access);
            if shortcut
                && file.logical.to_lowercase().ends_with(".lnk")
                && let Some(data) = self.read(&file, 16 * 1024 * 1024)
            {
                match shell::lnk::parse(&data) {
                    Ok(link) => {
                        let target = shortcut_target(self.ctx, &link, &file.logical);
                        if !target.is_empty() {
                            finding.value =
                                format!("{} {}", target, link.arguments).trim().to_string();
                            finding.resolve_target = target;
                            finding.description = format!(
                                "Startup shortcut target; arguments: {}. Link presence does not prove execution.",
                                link.arguments
                            );
                        } else {
                            self.issue(
                                "unresolved",
                                format!(
                                    "{} has no resolved shortcut target",
                                    file.physical.display()
                                ),
                            );
                        }
                    }
                    Err(e) => self.issue("malformed", format!("{}: {e}", file.physical.display())),
                }
            }
            self.scan.findings.push(finding);
        }
    }
}

fn shortcut_target(ctx: &Context, link: &shell::lnk::Link, logical: &str) -> String {
    let user = logical.split('/').nth(1).unwrap_or("");
    if link.flags & 0x100 == 0 && (!link.local_path.is_empty() || !link.network_path.is_empty()) {
        return super::resolver::expand_environment(ctx, &crate::lnk::link_target(link), user);
    }
    if !link.environment_target.is_empty() {
        return super::resolver::expand_environment(ctx, &link.environment_target, user);
    }
    if !link.relative_path.is_empty() {
        let relative = super::resolver::expand_environment(ctx, &link.relative_path, user);
        if relative.as_bytes().get(1) == Some(&b':') || relative.starts_with("\\\\") {
            return relative;
        }
        let parent = logical.rsplit_once('/').map(|v| v.0).unwrap_or("");
        return super::context::normalize_windows(&format!("{parent}/{relative}"))
            .unwrap_or_default();
    }
    let target = crate::lnk::link_target(link);
    // Shell item lists may begin with a display namespace such as "This PC".
    if let Some(colon) = target.find(":\\")
        && colon > 0
        && target.as_bytes()[colon - 1].is_ascii_alphabetic()
    {
        return target[colon - 1..].to_string();
    }
    target
}

pub(super) fn registry_text(
    hive: &super::context::HiveInput,
    key: &str,
    name: &str,
) -> Option<String> {
    registry_text_status(hive, key, name).0
}

fn registry_text_status(
    hive: &super::context::HiveInput,
    key: &str,
    name: &str,
) -> (Option<String>, bool) {
    registry_text_inner(hive, key, name).unwrap_or((None, false))
}
fn registry_text_inner(
    hive: &super::context::HiveInput,
    key: &str,
    name: &str,
) -> Option<(Option<String>, bool)> {
    let value = hive.opened.hive.key(key)?.value(name)?;
    if !matches!(value.data_type(), 1 | 2) {
        return None;
    }
    let bytes = value.data().ok()?;
    let registry::Data::Text { value: text, lossy } = registry::decode(value.data_type(), &bytes)
    else {
        return None;
    };
    let text = text.trim_end_matches('\0').trim();
    Some(((!text.is_empty()).then(|| text.to_string()), lossy))
}

fn users(ctx: &Context) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = ctx
        .hives
        .iter()
        .filter(|h| !h.user.is_empty() && h.user != "standalone_user")
        .map(|h| h.user.clone())
        .collect();
    for file in &ctx.files {
        let mut parts = file.logical.split('/');
        if parts
            .next()
            .is_some_and(|p| p.eq_ignore_ascii_case("Users"))
            && let Some(user) = parts.next()
            && parts.next().is_some()
        {
            names.insert(user.to_string());
        }
    }
    names
}

fn startup(check: &mut Check<'_>) {
    let common = "ProgramData/Microsoft/Windows/Start Menu/Programs/Startup";
    let mut folders: Vec<(String, String, &str)> =
        vec![(String::new(), common.to_string(), "SYSTEM")];
    for user in users(check.ctx) {
        folders.push((
            user.clone(),
            super::resolver::expand_environment(
                check.ctx,
                r"%USERPROFILE%\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup",
                &user,
            ),
            "USER",
        ));
    }
    for (user, default, access) in folders {
        let hive = if user.is_empty() {
            check.ctx.machine_hive("SOFTWARE")
        } else {
            check
                .ctx
                .hives
                .iter()
                .find(|h| h.kind == "NTUSER.DAT" && h.user == user)
        };
        let prefix = if user.is_empty() { "" } else { "Software\\" };
        let name = if user.is_empty() {
            "Common Startup"
        } else {
            "Startup"
        };
        let mut redirected = Vec::new();
        if let Some(hive) = hive {
            check.sources += 1;
            for suffix in ["User Shell Folders", "Shell Folders"] {
                let key = format!("{prefix}Microsoft\\Windows\\CurrentVersion\\Explorer\\{suffix}");
                let (value, lossy) = registry_text_status(hive, &key, name);
                if lossy {
                    check.issue("lossy_text", format!("{} {key} {name}: malformed UTF-16 decoded with replacement/trailing-byte recovery", hive.path.display()));
                }
                if let Some(value) = value {
                    redirected.push((
                        key,
                        value.clone(),
                        super::resolver::expand_environment(check.ctx, &value, &user),
                    ));
                }
            }
        }
        if check.definition.id == "startup_folder" {
            let folder = redirected
                .first()
                .map(|(_, _, path)| path.as_str())
                .unwrap_or(&default);
            check.folder(folder, access, 256, true);
        } else {
            let normalize = |p: &str| {
                let volume = super::context::explicit_drive(p)
                    .map(|d| check.ctx.captured_drive(d))
                    .unwrap_or(Some(check.ctx.system_drive()));
                super::context::normalize_windows(p).map(|s| (volume, s.to_lowercase()))
            };
            let mut scanned = BTreeSet::new();
            for (key, raw, folder) in redirected {
                if normalize(&folder).ok() == normalize(&default).ok() {
                    continue;
                }
                let hive = hive.expect("redirect from a hive");
                let prefix = if user.is_empty() {
                    "HKLM\\SOFTWARE".to_string()
                } else {
                    format!("HKU\\{user}")
                };
                let mut f = check.finding(&format!("{prefix}\\{key}\\{name}"), &raw, access);
                f.source_file = hive.path.display().to_string();
                f.description = "Startup folder redirected to a non-default path; this does not prove execution".into();
                f.resolve_target.clear();
                check.scan.findings.push(f);
                if scanned.insert(folder.to_lowercase()) {
                    check.folder(&folder, access, 256, true);
                }
            }
        }
    }
}

fn powershell(check: &mut Check<'_>) {
    let mut directories: BTreeSet<(String, &str)> = BTreeSet::new();
    directories.insert(("windows/system32/windowspowershell/v1.0".into(), "SYSTEM"));
    directories.insert(("windows/syswow64/windowspowershell/v1.0".into(), "SYSTEM"));
    for user in users(check.ctx) {
        let mut docs = BTreeSet::from([format!("Users/{user}/Documents")]);
        if let Some(hive) = check
            .ctx
            .hives
            .iter()
            .find(|h| h.kind == "NTUSER.DAT" && h.user == user)
        {
            for suffix in ["User Shell Folders", "Shell Folders"] {
                if let Some(raw) = registry_text(
                    hive,
                    &format!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\{suffix}"),
                    "Personal",
                ) {
                    if let Ok(path) = super::context::normalize_windows(
                        &super::resolver::expand_environment(check.ctx, &raw, &user),
                    ) {
                        docs.insert(path);
                    }
                    break;
                }
            }
        }
        for file in &check.ctx.files {
            let parts: Vec<_> = file.logical.split('/').collect();
            if parts.len() > 3
                && parts[0].eq_ignore_ascii_case("Users")
                && parts[1].eq_ignore_ascii_case(&user)
                && parts[2].to_lowercase().starts_with("onedrive")
            {
                docs.insert(format!("Users/{user}/{}/Documents", parts[2]));
            }
        }
        for doc in docs {
            for engine in ["WindowsPowerShell", "PowerShell"] {
                directories.insert((format!("{doc}/{engine}").to_lowercase(), "USER"));
            }
        }
    }
    let mut seen = BTreeSet::new();
    for file in check.files(|p| p.ends_with("profile.ps1")) {
        let lower = file.logical.to_lowercase();
        let parent = lower.rsplit_once('/').map(|p| p.0).unwrap_or("");
        let core = [
            "program files/powershell/",
            "program files (x86)/powershell/",
        ]
        .iter()
        .any(|p| {
            parent
                .strip_prefix(p)
                .is_some_and(|v| !v.is_empty() && !v.contains('/'))
        });
        let access = directories
            .iter()
            .find(|(p, _)| p == parent)
            .map(|(_, access)| *access)
            .or(core.then_some("SYSTEM"));
        if let Some(access) = access
            && seen.insert(lower)
        {
            check.sources += 1;
            let mut f = check.file_finding(&file, &file.logical, access);
            f.description =
                "PowerShell auto-loaded profile script; presence does not prove it ran".into();
            check.scan.findings.push(f);
        }
    }
}

fn office(check: &mut Check<'_>) {
    let mut folder_counts: HashMap<String, usize> = HashMap::new();
    for file in check.files(|p| {
        p.starts_with("users/")
            || p.starts_with("program files/microsoft office/")
            || p.starts_with("program files (x86)/microsoft office/")
    }) {
        let p = file.logical.to_lowercase();
        if p.ends_with("/desktop.ini") {
            continue;
        }
        let user_template = p.ends_with("/appdata/roaming/microsoft/templates/normal.dotm")
            || p.ends_with("/appdata/roaming/microsoft/templates/normalemail.dotm");
        let (parent, _) = p.rsplit_once('/').unwrap_or(("", ""));
        let user_startup = parent.ends_with("/appdata/roaming/microsoft/word/startup")
            || parent.ends_with("/appdata/roaming/microsoft/excel/xlstart");
        let machine_startup = [
            "program files/microsoft office/",
            "program files (x86)/microsoft office/",
        ]
        .iter()
        .any(|prefix| {
            let Some(tail) = parent.strip_prefix(prefix) else {
                return false;
            };
            let parts: Vec<_> = tail.split('/').collect();
            (parts.len() == 2 || parts.len() == 3 && parts[0] == "root")
                && matches!(parts.last(), Some(&"startup" | &"xlstart"))
        });
        if !(user_template || user_startup || machine_startup) {
            continue;
        }
        check.sources += 1;
        let count = folder_counts.entry(parent.to_string()).or_default();
        *count += 1;
        if *count > 128 {
            if *count == 129 {
                check.issue(
                    "limit_reached",
                    format!("Office global template folder {parent} exceeds 128 files"),
                );
            }
            continue;
        }
        let mut f = check.file_finding(
            &file,
            &file.logical,
            if machine_startup { "SYSTEM" } else { "USER" },
        );
        f.description = "Office auto-load template/startup file; presence alone does not establish malicious macros, execution or the application's token privileges".into();
        check.scan.findings.push(f);
    }
}

fn power_automate(check: &mut Check<'_>) {
    let mut stores = std::collections::BTreeMap::new();
    for file in &check.ctx.files {
        let lower = file.logical.to_lowercase();
        let Some((prefix, rest)) =
            lower.split_once("/appdata/local/microsoft/power automate desktop/")
        else {
            continue;
        };
        if !prefix.starts_with("users/") {
            continue;
        }
        let mut parts = rest.split('/');
        let store = parts.next().unwrap_or("");
        let flow = parts.next().unwrap_or("");
        if matches!(store, "flows" | "scripts") && !flow.is_empty() && parts.next().is_some() {
            let path = file
                .logical
                .split('/')
                .take(prefix.split('/').count() + 4 + 2)
                .collect::<Vec<_>>()
                .join("/");
            stores.entry(path).or_insert(file.physical.clone());
        }
    }
    for (store, source) in stores {
        check.sources += 1;
        let mut f = check.finding(
            &store.replace('/', "\\"),
            store.rsplit('/').next().unwrap_or(""),
            "USER",
        );
        f.description = "Collected Power Automate Desktop flow/script directory; a configured flow is not proof of execution".into();
        f.source_file = source.display().to_string();
        check.scan.findings.push(f);
    }
}

pub(super) fn decode_utf16(raw: &[u8], be: bool) -> Result<String, String> {
    if !raw.len().is_multiple_of(2) {
        return Err("odd-length UTF-16 input".into());
    }
    let words: Vec<_> = raw
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| {
            if be {
                u16::from_be_bytes([p[0], p[1]])
            } else {
                u16::from_le_bytes([p[0], p[1]])
            }
        })
        .collect();
    String::from_utf16(&words).map_err(|e| e.to_string())
}

pub(super) fn decode_text(raw: &[u8], ansi: bool) -> Result<String, String> {
    if let Some(raw) = raw.strip_prefix(&[0xff, 0xfe]) {
        return decode_utf16(raw, false);
    }
    if let Some(raw) = raw.strip_prefix(&[0xfe, 0xff]) {
        return decode_utf16(raw, true);
    }
    let raw = raw.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(raw);
    match std::str::from_utf8(raw) {
        Ok(s) => Ok(s.into()),
        Err(_) if ansi => {
            const CP1252: [char; 32] = [
                '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8d}',
                'Ž', '\u{8f}', '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›',
                'œ', '\u{9d}', 'ž', 'Ÿ',
            ];
            Ok(raw
                .iter()
                .map(|&b| {
                    if (0x80..=0x9f).contains(&b) {
                        CP1252[(b - 0x80) as usize]
                    } else {
                        b as char
                    }
                })
                .collect())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_encodings_are_strict_and_bounded() {
        assert_eq!(decode_text(&[0xff, 0xfe, b'a', 0], false).unwrap(), "a");
        assert!(decode_text(&[0xff, 0xfe, b'a'], false).is_err());
        assert_eq!(decode_text(&[0x80], true).unwrap(), "€");
        assert!(decode_text(&[0x80], false).is_err());
    }
    #[test]
    fn office_profiles_and_errors_use_collected_sources_only() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join(
            "Users/User Name/OneDrive - Org/Documents/PowerShell/Microsoft.VSCode_profile.ps1",
        );
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, b"# inert test").unwrap();
        let ctx = Context::open(dir.path(), true).unwrap();
        let defs = super::super::checks::catalogue();
        let mut scan = Scan::default();
        {
            let definition = defs
                .iter()
                .find(|d| d.id == "powershell_profiles")
                .expect("registered check");
            let mut check = Check {
                ctx: &ctx,
                definition,
                scan: &mut scan,
                sources: 0,
                issues: Vec::new(),
            };
            powershell(&mut check);
            assert_eq!(scan.findings.len(), 1);
            assert!(
                scan.findings[0]
                    .source_file
                    .ends_with("Microsoft.VSCode_profile.ps1")
            );
            assert!(scan.findings[0].last_change.is_empty());
        }
    }
    #[test]
    fn collection_only_coverage_and_jsonc_ini_office_sources_are_explicit() {
        let dir = tempfile::tempdir().unwrap();
        let artifacts = [
            (
                "Windows/System32/GroupPolicy/Machine/Scripts/scripts.ini",
                "[Startup]\n0CmdLine=C:\\Tools\\command.cmd\n0Parameters=-silent\n",
            ),
            (
                "Users/alice/AppData/Local/Microsoft/Windows Terminal/settings.json",
                r#"{"profiles":{"defaults":{"commandline":"pwsh.exe"},"list":[{"commandline":"cmd.exe /c test",},],},}"#,
            ),
            (
                "Users/alice/AppData/Roaming/Microsoft/Templates/Normal.dotm",
                "inert test fixture",
            ),
            (
                "Users/alice/AppData/Roaming/Microsoft/Templates/Other.dotx",
                "not an auto-load template",
            ),
            (
                "Program Files/Microsoft Office/root/Office16/STARTUP/addin.dotm",
                "inert test fixture",
            ),
            (
                "Users/alice/AppData/Local/Microsoft/Power Automate Desktop/Flows/TestFlow/definition.json",
                "{}",
            ),
        ];
        for (path, text) in artifacts {
            let p = dir.path().join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        }
        let ctx = Context::open(dir.path(), true).unwrap();
        let ids = [
            "gp_scripts",
            "windows_terminal",
            "office_templates",
            "power_automate",
            "accessibility_tools",
        ];
        let defs: Vec<_> = super::super::checks::catalogue()
            .iter()
            .filter(|d| ids.contains(&d.id.as_str()))
            .cloned()
            .collect();
        let scan = run(&ctx, &defs);
        let count = |id: &str| scan.findings.iter().filter(|f| f.check_id == id).count();
        assert_eq!(count("gp_scripts"), 1);
        assert_eq!(count("windows_terminal"), 2);
        assert_eq!(count("office_templates"), 2);
        assert_eq!(count("power_automate"), 1);
        assert_eq!(count("accessibility_tools"), 0);
        assert_eq!(
            scan.findings
                .iter()
                .find(|f| f.check_id == "power_automate")
                .unwrap()
                .value,
            "TestFlow"
        );
        assert_eq!(scan.coverage.len(), ids.len());
        assert_eq!(
            scan.coverage
                .iter()
                .find(|c| c.check_id == "accessibility_tools")
                .unwrap()
                .status,
            "not_collected"
        );
        assert!(
            scan.findings
                .iter()
                .all(|f| f.last_change.is_empty() && !f.source_file.is_empty())
        );
    }
    #[test]
    fn startup_shortcut_relative_targets_are_resolved_from_link_directory() {
        let dir = tempfile::tempdir().unwrap();
        let relative = r"..\..\payload.exe";
        let mut bytes = vec![0; 0x4c];
        bytes[..4].copy_from_slice(&0x4cu32.to_le_bytes());
        bytes[4..20].copy_from_slice(&[1, 0x14, 2, 0, 0, 0, 0, 0, 0xc0, 0, 0, 0, 0, 0, 0, 0x46]);
        bytes[20..24].copy_from_slice(&0x88u32.to_le_bytes());
        bytes.extend((relative.encode_utf16().count() as u16).to_le_bytes());
        bytes.extend(relative.encode_utf16().flat_map(u16::to_le_bytes));
        bytes.extend([0; 4]);
        let link = shell::lnk::parse(&bytes).unwrap();
        assert_eq!(
            shortcut_target(
                &Context::open(dir.path(), true).unwrap(),
                &link,
                "Users/alice/Startup/test.lnk"
            ),
            "Users/payload.exe"
        );
    }
    #[test]
    #[ignore = "requires evidence and a private expectations manifest"]
    fn collected_sources_match_external_expectations() {
        let evidence = std::env::var_os("JUNTEI_CTF_EVIDENCE").expect("set JUNTEI_CTF_EVIDENCE");
        let expectations =
            std::env::var_os("JUNTEI_CTF_EXPECTATIONS").expect("set JUNTEI_CTF_EXPECTATIONS");
        let expected: serde_json::Value =
            serde_json::from_slice(&std::fs::read(expectations).unwrap()).unwrap();
        let check_ids = expected["checks"].as_array().expect("checks array");
        let defs: Vec<_> = super::super::checks::catalogue()
            .iter()
            .filter(|d| check_ids.iter().any(|id| id.as_str() == Some(&d.id)))
            .cloned()
            .collect();
        let mut hosts = 0;
        let mut findings = Vec::new();
        let mut coverage = Vec::new();
        for entry in std::fs::read_dir(evidence).unwrap() {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_dir() {
                continue;
            }
            let ctx = Context::open(&entry.path(), false).unwrap();
            let scan = run(&ctx, &defs);
            hosts += 1;
            findings.extend(scan.findings);
            coverage.extend(scan.coverage);
        }
        assert_eq!(
            hosts,
            expected["hosts"].as_u64().expect("host count") as usize
        );
        for assertion in expected["assertions"].as_array().expect("assertions array") {
            let matching = findings
                .iter()
                .filter(|f| {
                    assertion
                        .get("computer")
                        .and_then(serde_json::Value::as_str)
                        .is_none_or(|name| f.computer.eq_ignore_ascii_case(name))
                        && assertion
                            .get("check")
                            .and_then(serde_json::Value::as_str)
                            .is_none_or(|id| f.check_id == id)
                        && assertion
                            .get("contains")
                            .and_then(serde_json::Value::as_array)
                            .is_none_or(|items| {
                                items.iter().all(|item| {
                                    let term = item.as_str().expect("contains text");
                                    f.value
                                        .replace("\\\\", "\\")
                                        .to_lowercase()
                                        .contains(&term.to_lowercase())
                                        || f.path.to_lowercase().contains(&term.to_lowercase())
                                })
                            })
                })
                .count();
            assert_eq!(
                matching,
                assertion["count"].as_u64().expect("expected count") as usize,
                "private expectation: {assertion}"
            );
        }
        for id in expected["no_malformed_coverage"]
            .as_array()
            .expect("coverage check IDs")
        {
            assert!(
                !coverage
                    .iter()
                    .any(|c| Some(c.check_id.as_str()) == id.as_str()
                        && c.detail.contains("malformed:")),
                "malformed coverage for {id}"
            );
        }
    }
}
