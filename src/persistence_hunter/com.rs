//! Offline COM registration joins. No COM activation or analyst-machine lookups.
//! Resolution describes candidates, not the Windows activation decision (which
//! also depends on architecture, elevation, policy and the activation context).
use super::context::{Context, HiveInput};
use super::resolver::{command_target, expand_environment};
use std::collections::BTreeSet;

#[derive(Default)]
pub(super) struct Resolution {
    pub servers: Vec<Server>,
    pub notes: Vec<String>,
    pub issues: Vec<String>,
}
pub(super) struct Server {
    pub association_only: bool,
    pub command: String,
    pub target: String,
    pub launcher: String,
    pub source: String,
}
#[derive(Clone, Copy)]
pub(super) enum Identity<'a> {
    Machine,
    User(&'a str),
    Principal(&'a str),
    Group(&'a str),
    Unknown,
}

pub(super) fn guid(text: &str) -> Option<String> {
    let text = text.trim();
    let text = if text.starts_with('{') && text.ends_with('}') {
        &text[1..text.len() - 1]
    } else {
        text
    };
    (text.len() == 36
        && text.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        }))
    .then(|| format!("{{{}}}", text.to_ascii_uppercase()))
}

#[derive(Clone, Copy)]
enum TargetKind {
    File,
    Command,
    Association,
}

impl Resolution {
    fn key<'h>(&mut self, hive: &'h HiveInput, path: &str) -> Option<registry::Key<'h>> {
        let mut key = hive.opened.hive.root()?;
        for part in path.split('\\').filter(|p| !p.is_empty()) {
            let (children, issues) = key.subkeys();
            self.issues.extend(
                issues
                    .into_iter()
                    .map(|e| format!("{} {path}: {e}", hive.path.display())),
            );
            key = children
                .into_iter()
                .find(|k| k.name().eq_ignore_ascii_case(part))?;
        }
        Some(key)
    }
    fn text(&mut self, hive: &HiveInput, path: &str, name: &str) -> Option<String> {
        let key = self.key(hive, path)?;
        self.text_key(hive, key, path, name)
    }
    fn text_key(
        &mut self,
        hive: &HiveInput,
        key: registry::Key<'_>,
        path: &str,
        name: &str,
    ) -> Option<String> {
        let (values, issues) = key.values();
        self.issues.extend(
            issues
                .into_iter()
                .map(|e| format!("{} {path}: {e}", hive.path.display())),
        );
        let value = values
            .into_iter()
            .find(|v| v.name().eq_ignore_ascii_case(name))?;
        if !matches!(value.data_type(), 1 | 2) {
            self.issues.push(format!(
                "{} {path} {name}: COM value is not a registry string",
                hive.path.display()
            ));
            return None;
        }
        let (data, issue) = value.read_data();
        if let Some(issue) = issue {
            self.issues
                .push(format!("{} {path} {name}: {issue}", hive.path.display()));
        }
        if let registry::Data::Text { value, lossy } = registry::decode(value.data_type(), &data) {
            if lossy {
                self.issues.push(format!(
                    "{} {path} {name}: lossy UTF-16",
                    hive.path.display()
                ));
            }
            let text = value.trim_end_matches('\0').trim();
            return (!text.is_empty()).then(|| text.to_string());
        }
        None
    }
    fn add(
        &mut self,
        ctx: &Context,
        command: String,
        user: &str,
        source: String,
        kind: TargetKind,
    ) {
        let (target, launcher) = match kind {
            TargetKind::File => (command.trim().trim_matches('"').to_string(), String::new()),
            TargetKind::Command => command_target(&command),
            TargetKind::Association => (String::new(), String::new()),
        };
        self.servers.push(Server {
            association_only: matches!(kind, TargetKind::Association),
            command,
            target: expand_environment(ctx, &target, user),
            launcher,
            source,
        });
    }
}

/// Use saved ProfileList data for SIDs and unique captured folder names for
/// account-name heuristics. Ambiguous candidates are never picked arbitrarily.
fn principal_user(ctx: &Context, name: &str, out: &mut Resolution) -> Option<String> {
    let mut matches = BTreeSet::new();
    let profiles = ctx
        .machine_hive("SOFTWARE")
        .map(|h| registry::software::profile_list(&h.opened.hive))
        .unwrap_or_default();
    if name.to_ascii_uppercase().starts_with("S-1-") {
        for (sid, profile) in profiles {
            if sid.eq_ignore_ascii_case(name) {
                let expanded = expand_environment(ctx, &profile, "");
                if let super::context::Resolution::Found(file) = ctx.resolve(&format!(
                    "{}\\NTUSER.DAT",
                    expanded.trim_end_matches(['\\', '/'])
                )) && let Some(h) = ctx.hives.iter().find(|h| h.path == file.physical)
                {
                    matches.insert(h.user.clone());
                }
                // UsrClass.dat may be the only collected hive for this profile.
                if let super::context::Resolution::Found(file) = ctx.resolve(&format!(
                    "{}\\AppData\\Local\\Microsoft\\Windows\\UsrClass.dat",
                    expanded.trim_end_matches(['\\', '/'])
                )) && let Some(h) = ctx.hives.iter().find(|h| h.path == file.physical)
                {
                    matches.insert(h.user.clone());
                }
            }
        }
    } else {
        let (domain, account) = name
            .split_once('\\')
            .map_or((None, name), |(d, a)| (Some(d), a));
        // Volatile Environment is normally memory-only and absent from saved
        // hives. Profile folder names are usable offline, but are an inference,
        // not an authoritative domain/account-to-SID mapping.
        if !account.is_empty()
            && !account.contains(['\\', '/', '\0'])
            && domain.is_none_or(|d| !d.is_empty() && !d.contains(['/', '\0']))
        {
            let qualified = domain
                .filter(|d| *d != ".")
                .map(|d| format!("{account}.{d}"));
            let hosts = ctx.host_identities();
            let local = domain
                .is_some_and(|d| d == "." || hosts.iter().any(|(h, _)| h.eq_ignore_ascii_case(d)));
            let mut exact_matches = BTreeSet::new();
            let mut local_matches = BTreeSet::new();
            let mut plain_matches = BTreeSet::new();
            for h in ctx
                .hives
                .iter()
                .filter(|h| matches!(h.kind.as_str(), "NTUSER.DAT" | "USRCLASS.DAT"))
            {
                if qualified
                    .as_deref()
                    .is_some_and(|q| h.user.eq_ignore_ascii_case(q))
                {
                    exact_matches.insert(h.user.clone());
                }
                if local
                    && hosts
                        .iter()
                        .any(|(host, _)| h.user.eq_ignore_ascii_case(&format!("{account}.{host}")))
                {
                    local_matches.insert(h.user.clone());
                }
                if h.user.eq_ignore_ascii_case(account) {
                    plain_matches.insert(h.user.clone());
                }
            }
            // Domain-qualified profile folders are more specific than a plain
            // account name. Local aliases are eligible only from captured host
            // identities. Never hide ambiguity at one rank by using a lower rank.
            matches = if !exact_matches.is_empty() {
                exact_matches
            } else if !local_matches.is_empty() {
                local_matches
            } else {
                plain_matches
            };
        }
        if matches.len() == 1 {
            let user = matches.first().expect("one matching profile");
            let qualifier = if let Some(domain) = domain {
                if domain == "."
                    || ctx
                        .host_identities()
                        .iter()
                        .any(|(n, _)| n.eq_ignore_ascii_case(domain))
                {
                    "qualifier matches a captured host name; account ownership is unverified"
                } else {
                    "domain membership and account ownership are unverified"
                }
            } else {
                "account ownership is unverified"
            };
            out.notes.push(format!("Principal {name} mapped to collected profile {user} by folder-name heuristic ({qualifier})."));
        }
    }
    if matches.len() == 1 {
        matches.pop_first()
    } else {
        out.issues.push(format!("Cannot map principal {name} uniquely to a collected user profile; per-user COM lookup unavailable"));
        out.notes.push(format!("Principal {name}: per-user COM lookup unavailable (missing or ambiguous captured profile mapping)."));
        None
    }
}

fn file_uri(text: &str) -> Result<String, String> {
    if !text
        .get(..5)
        .is_some_and(|p| p.eq_ignore_ascii_case("file:"))
    {
        if text.contains("://") {
            return Err("remote managed CodeBase retained without accessing it".into());
        }
        return Ok(text.to_string());
    }
    let tail = &text[5..];
    // Only local file:///C:/... or file:/C:/..., never file://server/share.
    let tail = if let Some(local) = tail.strip_prefix("///") {
        local
    } else if tail.starts_with("//") {
        return Err("remote file CodeBase retained without accessing it".into());
    } else {
        tail.strip_prefix('/').unwrap_or(tail)
    };
    let mut bytes = Vec::new();
    let input = tail.as_bytes();
    let mut at = 0;
    while at < input.len() {
        if input[at] == b'%' {
            let hex = input
                .get(at + 1..at + 3)
                .and_then(|s| std::str::from_utf8(s).ok())
                .and_then(|s| u8::from_str_radix(s, 16).ok())
                .ok_or("invalid managed CodeBase percent escape")?;
            bytes.push(hex);
            at += 3;
        } else {
            bytes.push(input[at]);
            at += 1;
        }
    }
    let path = String::from_utf8(bytes).map_err(|_| "invalid UTF-8 managed CodeBase")?;
    if super::context::explicit_drive(&path).is_none() || path.contains(['\0', '"']) {
        return Err("managed CodeBase is not a local absolute Windows path".into());
    }
    Ok(path)
}

pub(super) fn resolve(ctx: &Context, clsid: &str, identity: Identity<'_>) -> Resolution {
    let mut out = Resolution::default();
    let Some(clsid) = guid(clsid) else {
        out.issues
            .push(format!("Invalid COM class identifier: {clsid}"));
        return out;
    };
    let user = match identity {
        Identity::Machine => None,
        Identity::User(user) => Some(user.to_string()),
        Identity::Principal(name) => principal_user(ctx, name, &mut out),
        Identity::Group(group) => {
            out.notes.push(format!("Group principal {group}: machine COM candidates only; an individual user's overrides cannot be determined from the group alone."));
            None
        }
        Identity::Unknown => {
            out.issues.push(
                "No unambiguous individual task principal; per-user COM lookup unavailable".into(),
            );
            None
        }
    };
    if let Some(user) = &user
        && !ctx.hives.iter().any(|h| h.user.eq_ignore_ascii_case(user))
    {
        out.issues.push(format!(
            "No collected user hives for {user}; per-user COM lookup unavailable"
        ));
    }
    // Enumerate both architectures independently. Never resolve a 32-bit CLSID
    // through an unrelated 64-bit override, or one user's hive through another.
    for wow in [false, true] {
        let mut views = Vec::new();
        if let Some(user) = &user {
            for kind in ["USRCLASS.DAT", "NTUSER.DAT"] {
                for h in ctx
                    .hives
                    .iter()
                    .filter(|h| h.kind == kind && h.user.eq_ignore_ascii_case(user))
                {
                    let root = match (kind, wow) {
                        ("USRCLASS.DAT", false) => "",
                        ("USRCLASS.DAT", true) => "Wow6432Node",
                        (_, false) => "Software\\Classes",
                        (_, true) => "Software\\Classes\\Wow6432Node",
                    };
                    views.push((h, root.to_string()));
                }
            }
        }
        if let Some(h) = ctx.machine_hive("SOFTWARE") {
            views.push((
                h,
                if wow {
                    "Classes\\Wow6432Node"
                } else {
                    "Classes"
                }
                .to_string(),
            ));
        }
        let arch = if wow { "32-bit view" } else { "native view" };
        // HKCR does not merge the children of a duplicate per-user CLSID.
        let selected = views
            .iter()
            .find(|(h, root)| out.key(h, &format!("{root}\\CLSID\\{clsid}")).is_some());
        let Some((hive, root)) = selected else {
            continue;
        };
        let base = format!("{root}\\CLSID\\{clsid}");
        let mut commands = Vec::new();
        if let Some(command) = out.text(hive, &format!("{base}\\InprocServer32"), "") {
            let server = command
                .trim_matches('"')
                .replace('/', "\\")
                .rsplit('\\')
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            if server == "mscoree.dll" {
                let managed_key = format!("{base}\\InprocServer32");
                let mut codebases = Vec::new();
                if let Some(codebase) = out.text(hive, &managed_key, "CodeBase") {
                    codebases.push((managed_key.clone(), codebase));
                }
                if let Some(key) = out.key(hive, &managed_key) {
                    let (children, issues) = key.subkeys();
                    out.issues.extend(issues);
                    if children.len() > 256 {
                        out.issues
                            .push("Managed COM version limit reached (256)".into());
                    }
                    for child in children.into_iter().take(256) {
                        let path = format!("{managed_key}\\{}", child.name());
                        if let Some(codebase) = out.text_key(hive, child, &path, "CodeBase") {
                            codebases.push((path, codebase));
                        }
                    }
                }
                if codebases.is_empty() {
                    out.notes.push(format!("{base}: managed CLR loader found; managed assembly CodeBase not collected (GAC activation may apply)"));
                    out.issues.push(format!(
                        "{base}: managed assembly path unresolved; only CLR loader is known"
                    ));
                    commands.push((command, managed_key, TargetKind::File));
                } else {
                    for (path, codebase) in codebases {
                        match file_uri(&codebase) {
                            Ok(path_value) => commands.push((
                                format!("\"{path_value}\""),
                                format!("{path} CodeBase ({codebase})"),
                                TargetKind::File,
                            )),
                            Err(e) => {
                                out.issues.push(e);
                                out.notes.push(format!("{path} CodeBase: {codebase}"));
                            }
                        }
                    }
                }
            } else {
                commands.push((command, format!("{base}\\InprocServer32"), TargetKind::File));
            }
        }
        if let Some(command) = out.text(hive, &format!("{base}\\LocalServer32"), "") {
            commands.push((
                command,
                format!("{base}\\LocalServer32"),
                TargetKind::Command,
            ));
        }
        if let Some(appid_raw) = out.text(hive, &base, "AppID") {
            if let Some(appid) = guid(&appid_raw) {
                let appkey = format!("AppID\\{appid}");
                // AppID configuration is machine-wide; never enumerate unrelated users.
                if let Some(machine) = ctx.machine_hive("SOFTWARE") {
                    for app_root in [
                        if wow {
                            "Classes\\Wow6432Node"
                        } else {
                            "Classes"
                        },
                        "Classes",
                    ] {
                        let app = format!("{app_root}\\{appkey}");
                        if let Some(service) = out.text(machine, &app, "LocalService") {
                            if service.contains(['\\', '/']) || service == "." || service == ".." {
                                out.issues
                                    .push(format!("Invalid AppID service name: {service}"));
                            } else if let Some(system) = ctx.machine_hive("SYSTEM") {
                                let key =
                                    format!("{}\\Services\\{service}", ctx.active_controlset());
                                if let Some(command) = out.text(system, &key, "ImagePath") {
                                    commands.push((
                                        command,
                                        format!(
                                            "{} {app} LocalService={service}; {} {key} ImagePath",
                                            machine.path.display(),
                                            system.path.display()
                                        ),
                                        TargetKind::Command,
                                    ));
                                } else {
                                    out.issues.push(format!(
                                        "AppID service {service} has no collected ImagePath"
                                    ));
                                }
                            } else {
                                out.issues.push(format!(
                                    "AppID service {service} requires collected SYSTEM"
                                ));
                            }
                        }
                        // The original proposal joins AppID\\program.exe back to its
                        // GUID. Keep that association explicit; it is not a full path.
                        if commands.is_empty()
                            && let Some(key) = out.key(machine, &format!("{app_root}\\AppID"))
                        {
                            let (children, issues) = key.subkeys();
                            out.issues.extend(issues);
                            if children.len() > 4096 {
                                out.issues
                                    .push("AppID association limit reached (4096)".into());
                            }
                            for entry in children.into_iter().take(4096) {
                                let name = entry.name();
                                if !name.to_ascii_lowercase().ends_with(".exe")
                                    || name.contains(['\\', '/'])
                                {
                                    continue;
                                }
                                let path = format!("{app_root}\\AppID\\{name}");
                                if out
                                    .text_key(machine, entry, &path, "AppID")
                                    .and_then(|v| guid(&v))
                                    .as_deref()
                                    == Some(&appid)
                                {
                                    out.notes.push(format!("{} {path} associates executable {name} with AppID {appid}; basename alone does not establish its path", machine.path.display()));
                                    // Report association as a candidate without guessing a directory.
                                    {
                                        commands.push((
                                            name,
                                            format!(
                                                "{} {path} AppID={appid}; executable association only", machine.path.display()
                                            ),
                                            TargetKind::Association,
                                        ));
                                    }
                                }
                            }
                        }
                        if app_root == "Classes" {
                            break;
                        }
                    }
                }
            } else {
                out.issues.push(format!("Invalid AppID: {appid_raw}"));
            }
        }
        for (command, path, association_only) in commands {
            out.add(
                ctx,
                command,
                user.as_deref().unwrap_or(""),
                format!("{} {path} ({arch})", hive.path.display()),
                association_only,
            );
        }
    }
    out.servers
        .sort_by(|a, b| (&a.command, &a.source).cmp(&(&b.command, &b.source)));
    out.servers
        .dedup_by(|a, b| a.command == b.command && a.source == b.source);
    if out.servers.is_empty() {
        out.issues
            .push(format!("No collected COM server resolved for {clsid}"));
    }
    out.issues.sort();
    out.issues.dedup();
    out
}
