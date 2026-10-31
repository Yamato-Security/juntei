//! Registry checks needing joins, typed predicates, or custom payload extraction.
//! Adapted from PyrsistenceSniper, copyright Hexastrike, MIT license.

use super::TimeEvidence;
use super::checks::{CheckRun, RegData, join};
use super::context::HiveInput;
use registry::Key;
use std::collections::{BTreeMap, BTreeSet};

/// All non-declarative registry checks. File/repository checks live in file_checks.
pub(crate) fn execute(r: &mut CheckRun<'_>) {
    match r.check.id.as_str() {
        "run_keys" | "ts_initial_program" => run_sections(r),
        "active_setup" => active_setup(r),
        "windows_service_image_path"
        | "windows_service_dll"
        | "service_failure_command"
        | "network_provider_dll" => services(r),
        "appinit_dlls" => appinit(r),
        "assistive_technology" => assistive(r),
        "app_paths" => child_values(
            r,
            "SOFTWARE",
            r"Microsoft\Windows\CurrentVersion\App Paths",
            &["(Default)"],
        ),
        "amsi_providers" | "explorer_bho" | "disk_cleanup_handler" => registered_com(r),
        "explorer_app_key" => child_values(
            r,
            "SOFTWARE",
            r"Microsoft\Windows\CurrentVersion\Explorer\AppKey",
            &["ShellExecute", "Association"],
        ),
        "explorer_context_menu" => context_menus(r),
        "telemetry_controller" => telemetry(r),
        "ifeo_debugger" | "ifeo_silent_process_exit" | "ifeo_delegated_ntdll" => ifeo(r),
        "screensaver" => user_values(r, r"Control Panel\Desktop", &["SCRNSAVE.EXE"]),
        "content_index_dll" => child_values(
            r,
            "SYSTEM",
            &format!(
                r"{}\Control\ContentIndex\Language",
                r.ctx.active_controlset()
            ),
            &["DLLOverridePath"],
        ),
        "known_dlls" | "exclude_from_known_dlls" | "dll_search_mode" => known_dlls(r),
        "dotnet_startup_hooks" | "cor_profiler" | "coreclr_profiler" => environment(r),
        "dsrm_backdoor" => dsrm(r),
        "lsa_password_filter" => lsa_password_filter(r),
        "rdp_virtual_channel" => rdp_channels(r),
        "minidump_auxiliary_dlls" => minidump(r),
        "wer_runtime_exception" => wer_runtime(r),
        "shim_custom" => shim(r),
        "office_ai_hijack" => all_values(
            r,
            "SOFTWARE",
            r"Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI",
        ),
        "office_dll_override" => office_overrides(r),
        "office_addins" => office_addins(r),
        "outlook_home_page" => outlook(r),
        "vba_monitors" => vba(r),
        "file_association_hijack" => file_associations(r),
        "protocol_handler_hijack" => protocols(r),
        "com_treat_as" => com_hijacks(r),
        "typelib_hijack" => typelib(r),
        "hidden_account" => hidden_accounts(r),
        "rid_hijacking" | "rid_suborner" => rid_checks(r),
        "admin_group_membership" => admin_members(r),
        id => r.issue(format!("No registry implementation for check {id}")),
    }
}

fn child_values(r: &mut CheckRun<'_>, kind: &str, path: &str, values: &[&str]) {
    let Some(h) = r.machine(kind) else {
        return;
    };
    let Some(key) = r.key(h, path) else {
        return;
    };
    for c in r.children(key) {
        for n in values {
            if let Some(v) = r.text(c, n) {
                r.emit(
                    h,
                    c,
                    join(&[
                        "HKLM",
                        kind,
                        path,
                        &c.name(),
                        if *n == "(Default)" { "" } else { n },
                    ]),
                    v,
                    "SYSTEM",
                );
            }
        }
    }
}
fn all_values(r: &mut CheckRun<'_>, kind: &str, path: &str) {
    let Some(h) = r.machine(kind) else {
        return;
    };
    let Some(key) = r.key(h, path) else {
        return;
    };
    for (n, v) in r.values(key) {
        r.emit(h, key, join(&["HKLM", kind, path, &n]), v.text(), "SYSTEM");
    }
}
fn user_values(r: &mut CheckRun<'_>, path: &str, values: &[&str]) {
    for h in r.users("NTUSER.DAT") {
        let Some(key) = r.key(h, path) else {
            continue;
        };
        for n in values {
            if let Some(v) = r.text(key, n) {
                r.emit(h, key, join(&["HKU", &h.user, path, n]), v, "USER");
            }
        }
    }
}

fn run_sections(r: &mut CheckRun<'_>) {
    r.declarative();
    for t in &r.check.targets {
        if !["runex", "runonceex"].contains(
            &t.path
                .rsplit('\\')
                .next()
                .unwrap_or("")
                .to_lowercase()
                .as_str(),
        ) {
            continue;
        }
        if t.scope == "HKLM" || t.scope == "BOTH" {
            let (kind, path) = t.path.split_once('\\').unwrap_or((&t.path, ""));
            if let Some(h) = r.machine(kind)
                && let Some(key) = r.key(h, path)
            {
                section_walk(
                    r,
                    h,
                    key,
                    &join(&["HKLM", &t.path]),
                    "SYSTEM",
                    0,
                    &mut BTreeSet::new(),
                );
            }
        }
        if t.scope == "HKU" || t.scope == "BOTH" {
            for h in r.users("NTUSER.DAT") {
                if let Some(key) = r.key(h, &t.path) {
                    section_walk(
                        r,
                        h,
                        key,
                        &join(&["HKU", &h.user, &t.path]),
                        "USER",
                        0,
                        &mut BTreeSet::new(),
                    );
                }
            }
        }
    }
}
fn section_walk<'a>(
    r: &mut CheckRun<'a>,
    h: &HiveInput,
    key: Key<'a>,
    path: &str,
    access: &str,
    depth: usize,
    seen: &mut BTreeSet<u32>,
) {
    if depth > 128 {
        r.issue("Run-section key recursion exceeds 128 levels");
        return;
    }
    if !seen.insert(key.offset()) {
        r.issue("Run-section key is linked more than once or forms a cycle");
        return;
    }
    for child in r.children(key) {
        if seen.contains(&child.offset()) {
            r.issue("Run-section key is linked more than once or forms a cycle");
            continue;
        }
        let p = join(&[path, &child.name()]);
        for (n, v) in r.values(child) {
            if n.is_empty() {
                continue;
            }
            for s in v.strings() {
                if !s.trim().is_empty() {
                    r.emit(h, child, join(&[&p, &n]), s, access);
                }
            }
        }
        section_walk(r, h, child, &p, access, depth + 1, seen);
    }
}
fn active_setup(r: &mut CheckRun<'_>) {
    let Some(h) = r.machine("SOFTWARE") else {
        return;
    };
    for path in [
        r"Microsoft\Active Setup\Installed Components",
        r"Wow6432Node\Microsoft\Active Setup\Installed Components",
    ] {
        let Some(key) = r.key(h, path) else {
            continue;
        };
        for c in r.children(key) {
            if let Some(v) = r.text(c, "StubPath")
                && v != "U"
                && v != "/UserInstall"
            {
                r.emit(
                    h,
                    c,
                    join(&[r"HKLM\SOFTWARE", path, &c.name(), "StubPath"]),
                    v,
                    "SYSTEM",
                );
            }
        }
    }
}
fn services(r: &mut CheckRun<'_>) {
    let Some(h) = r.machine("SYSTEM") else {
        return;
    };
    let path = format!(r"{}\Services", r.ctx.active_controlset());
    let Some(key) = r.key(h, &path) else {
        return;
    };
    let service_nodes = r.children(key);
    let mut image_counts = BTreeMap::new();
    for service in &service_nodes {
        if let Some(image) = r.text(*service, "ImagePath") {
            *image_counts
                .entry(normalize_service_image(&image))
                .or_insert(0usize) += 1;
        }
    }
    for s in service_nodes {
        let name = s.name();
        let base = join(&[r"HKLM\SYSTEM", &path, &name]);
        match r.check.id.as_str() {
            "windows_service_image_path" => {
                if let Some(v) = r.text(s, "ImagePath") {
                    let i = r.emit(h, s, join(&[&base, "ImagePath"]), v, "SYSTEM");
                    service_evidence(r, s, &name, i, &image_counts);
                } else if let Some(t) = r.number(s, "Type")
                    && [1, 2, 8].contains(&t)
                {
                    let value = format!("{name}.sys");
                    let i = r.emit(h, s, &base, &value, "SYSTEM");
                    r.findings[i].resolve_target = format!(r"\SystemRoot\System32\drivers\{value}");
                    r.findings[i].description = format!(
                        "Driver service registered without ImagePath; Windows can load the implied driver from \\SystemRoot\\System32\\drivers\\{value}."
                    );
                    service_evidence(r, s, &name, i, &image_counts);
                }
            }
            "windows_service_dll" => {
                if let Some(p) = r.child(s, "Parameters")
                    && let Some(v) = r.text(p, "ServiceDll")
                {
                    let i = r.emit(h, p, join(&[&base, r"Parameters\ServiceDll"]), v, "SYSTEM");
                    service_evidence(r, s, &name, i, &image_counts);
                }
            }
            "network_provider_dll" => {
                if let Some(p) = r.child(s, "NetworkProvider")
                    && let Some(v) = r.text(p, "ProviderPath")
                {
                    r.emit(
                        h,
                        p,
                        join(&[&base, r"NetworkProvider\ProviderPath"]),
                        v,
                        "SYSTEM",
                    );
                }
            }
            _ => {
                if let Some(v) = r.text(s, "FailureCommand") {
                    r.emit(h, s, join(&[&base, "FailureCommand"]), v, "SYSTEM");
                }
            }
        }
    }
}
fn service_evidence<'a>(
    r: &mut CheckRun<'a>,
    key: Key<'a>,
    name: &str,
    finding: usize,
    image_counts: &BTreeMap<String, usize>,
) {
    let mut names = vec![name.to_string()];
    if let Some(display) = r.text(key, "DisplayName")
        && display != name
    {
        names.push(display);
    }
    let mut evidence = vec![];
    for name in names {
        evidence.push(TimeEvidence::EventLog {
            channel: "System".into(),
            event_ids: vec![7045],
            match_field: "ServiceName".into(),
            match_value: name,
        });
    }
    // Many Windows services share a svchost command line. An ImagePath-only event
    // can date another service's installation, so retain it only as a fallback
    // when this command identifies exactly one service in the active control set.
    if let Some(image) = r.text(key, "ImagePath")
        && image_counts.get(&normalize_service_image(&image)) == Some(&1)
    {
        evidence.push(TimeEvidence::EventLog {
            channel: "System".into(),
            event_ids: vec![7045],
            match_field: "ImagePath".into(),
            match_value: image,
        });
    }
    r.findings[finding].time_evidence = evidence;
}

fn normalize_service_image(image: &str) -> String {
    let mut value = image
        .trim()
        .to_lowercase()
        .replace('/', "\\")
        .replace('"', "");
    for alias in ["%systemroot%", "%windir%", r"\systemroot"] {
        value = value.replace(alias, "windows");
    }
    // The Windows directory may be stored using the drive, a root-relative path,
    // or a system-root variable. Treat equivalent spellings conservatively: a
    // possible shared image disables the fallback rather than guessing a date.
    if value.as_bytes().get(1) == Some(&b':')
        && value.get(2..).is_some_and(|s| s.starts_with(r"\windows\"))
    {
        value = value[3..].to_string();
    }
    value = value.trim_start_matches('\\').to_string();
    if value.starts_with(r"system32\") || value.starts_with(r"syswow64\") {
        value = format!(r"windows\{value}");
    }
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
fn appinit(r: &mut CheckRun<'_>) {
    let Some(h) = r.machine("SOFTWARE") else {
        return;
    };
    for path in [
        r"Microsoft\Windows NT\CurrentVersion\Windows",
        r"Wow6432Node\Microsoft\Windows NT\CurrentVersion\Windows",
    ] {
        let Some(k) = r.key(h, path) else {
            continue;
        };
        let Some(v) = r.text(k, "AppInit_DLLs") else {
            continue;
        };
        let mut context = vec![];
        if let Some(n) = r.number(k, "LoadAppInit_DLLs") {
            context.push(format!(
                "LoadAppInit_DLLs={n}{}",
                if n == 0 { " (INACTIVE)" } else { "" }
            ));
        }
        if let Some(n) = r.number(k, "RequireSignedAppInit_DLLs") {
            context.push(format!("RequireSignedAppInit_DLLs={n}"));
        }
        for dll in v
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|v| !v.is_empty())
        {
            let i = r.emit(
                h,
                k,
                join(&[r"HKLM\SOFTWARE", path, "AppInit_DLLs"]),
                if context.is_empty() {
                    dll.to_string()
                } else {
                    format!("{dll} [{}]", context.join(", "))
                },
                "SYSTEM",
            );
            r.findings[i].resolve_target = dll.to_string();
        }
    }
}
fn assistive(r: &mut CheckRun<'_>) {
    if let Some(h) = r.machine("SOFTWARE") {
        for path in [
            r"Microsoft\Windows NT\CurrentVersion\Accessibility\ATs",
            r"Wow6432Node\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs",
        ] {
            let Some(k) = r.key(h, path) else {
                continue;
            };
            for c in r.children(k) {
                let Some(data) = r.value(c, "StartExe") else {
                    continue;
                };
                if !matches!(data, RegData::Text(_)) {
                    continue;
                }
                let mut v = data.text().trim().to_string();
                if v.is_empty() || v.chars().all(|c| c.is_ascii_digit()) {
                    continue;
                }
                if let Some(RegData::Text(p)) = r.value(c, "StartParams")
                    && !p.trim().is_empty()
                {
                    v.push(' ');
                    v.push_str(p.trim());
                }
                r.emit(
                    h,
                    c,
                    join(&[r"HKLM\SOFTWARE", path, &c.name(), "StartExe"]),
                    v,
                    "SYSTEM",
                );
            }
        }
    }
    let config = r"Software\Microsoft\Windows NT\CurrentVersion\Accessibility";
    for h in r.users("NTUSER.DAT") {
        if let Some(k) = r.key(h, config)
            && let Some(v) = r.text(k, "Configuration")
        {
            for n in v.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                r.emit(
                    h,
                    k,
                    join(&["HKU", &h.user, config, "Configuration"]),
                    n,
                    "USER",
                );
            }
        }
    }
    if let Some(h) = r.machine("DEFAULT")
        && let Some(k) = r.key(h, config)
        && let Some(v) = r.text(k, "Configuration")
    {
        for n in v.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            r.emit(
                h,
                k,
                join(&[r"HKU\.DEFAULT", config, "Configuration"]),
                n,
                "SYSTEM",
            );
        }
    }
}
fn registered_com(r: &mut CheckRun<'_>) {
    let path = match r.check.id.as_str() {
        "amsi_providers" => r"Microsoft\AMSI\Providers",
        "explorer_bho" => r"Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects",
        _ => r"Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches",
    };
    let Some(h) = r.machine("SOFTWARE") else {
        return;
    };
    let Some(k) = r.key(h, path) else {
        return;
    };
    for c in r.children(k) {
        let clsid = if r.check.id == "disk_cleanup_handler" {
            r.text(c, "(Default)").unwrap_or_default()
        } else {
            c.name()
        };
        if !clsid.starts_with('{') {
            continue;
        }
        let dll = if r.check.id == "amsi_providers" {
            r.clsid_server(h, "Classes", &clsid)
        } else {
            r.default_server(h, &join(&[r"Classes\CLSID", &clsid, "InprocServer32"]))
        };
        if dll.is_empty() && r.check.id == "disk_cleanup_handler" {
            continue;
        }
        let i = r.emit(
            h,
            c,
            join(&[r"HKLM\SOFTWARE", path, &c.name()]),
            if dll.is_empty() { clsid } else { dll.clone() },
            "SYSTEM",
        );
        if dll.is_empty() {
            r.findings[i].description.push_str(
                " Registered COM server could not be resolved from collected registry views.",
            );
        }
    }
}
fn telemetry(r: &mut CheckRun<'_>) {
    let path = r"Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController";
    let Some(h) = r.machine("SOFTWARE") else {
        return;
    };
    let Some(k) = r.key(h, path) else {
        return;
    };
    let mut nodes = vec![(k, path.to_string())];
    nodes.extend(
        r.children(k)
            .into_iter()
            .map(|c| (c, join(&[path, &c.name()]))),
    );
    for (n, p) in nodes {
        for name in ["Binary", "Command", "MaintenanceCommand"] {
            if let Some(v) = r.text(n, name) {
                r.emit(h, n, join(&[r"HKLM\SOFTWARE", &p, name]), v, "SYSTEM");
            }
        }
    }
}
fn ifeo(r: &mut CheckRun<'_>) {
    let (surface, value) = match r.check.id.as_str() {
        "ifeo_debugger" => ("Image File Execution Options", "Debugger"),
        "ifeo_silent_process_exit" => ("SilentProcessExit", "MonitorProcess"),
        _ => ("Image File Execution Options", "VerifierDlls"),
    };
    let Some(h) = r.machine("SOFTWARE") else {
        return;
    };
    for prefix in [
        r"Microsoft\Windows NT\CurrentVersion",
        r"Wow6432Node\Microsoft\Windows NT\CurrentVersion",
    ] {
        let p = join(&[prefix, surface]);
        if let Some(k) = r.key(h, &p) {
            walk_named(
                r,
                h,
                k,
                &join(&[r"HKLM\SOFTWARE", &p]),
                value,
                0,
                &mut BTreeSet::new(),
            );
        }
    }
}
fn walk_named<'a>(
    r: &mut CheckRun<'a>,
    h: &HiveInput,
    k: Key<'a>,
    path: &str,
    value: &str,
    depth: usize,
    seen: &mut BTreeSet<u32>,
) {
    if depth > 128 {
        r.issue("Registry traversal exceeds 128 levels");
        return;
    }
    if !seen.insert(k.offset()) {
        r.issue("Registry key is linked more than once or forms a cycle");
        return;
    }
    for child in r.children(k) {
        if seen.contains(&child.offset()) {
            r.issue("Registry key is linked more than once or forms a cycle");
            continue;
        }
        let p = join(&[path, &child.name()]);
        if let Some(v) = r.text(child, value) {
            r.emit(h, child, join(&[&p, value]), v, "SYSTEM");
        }
        walk_named(r, h, child, &p, value, depth + 1, seen);
    }
}
fn known_dlls(r: &mut CheckRun<'_>) {
    let Some(h) = r.machine("SYSTEM") else {
        return;
    };
    let base = format!(r"{}\Control\Session Manager", r.ctx.active_controlset());
    if r.check.id == "known_dlls" {
        let p = join(&[&base, "KnownDLLs"]);
        if let Some(k) = r.key(h, &p) {
            for (n, v) in r.values(k) {
                if !n.trim().is_empty() {
                    r.emit(h, k, join(&[r"HKLM\SYSTEM", &p, &n]), v.text(), "SYSTEM");
                }
            }
        }
    } else if let Some(k) = r.key(h, &base) {
        if r.check.id == "exclude_from_known_dlls" {
            if let Some(v) = r.value(k, "ExcludeFromKnownDlls") {
                for s in v.strings() {
                    if !s.trim().is_empty() {
                        r.emit(
                            h,
                            k,
                            join(&[r"HKLM\SYSTEM", &base, "ExcludeFromKnownDlls"]),
                            s.trim(),
                            "SYSTEM",
                        );
                    }
                }
            }
        } else {
            for n in ["SafeDllSearchMode", "CWDIllegalInDllSearch"] {
                if r.number(k, n) == Some(0) {
                    r.emit(h, k, join(&[r"HKLM\SYSTEM", &base, n]), "0", "SYSTEM");
                }
            }
        }
    }
}
fn environment(r: &mut CheckRun<'_>) {
    let names: &[&str] = match r.check.id.as_str() {
        "dotnet_startup_hooks" => &["DOTNET_STARTUP_HOOKS"],
        "cor_profiler" => &[
            "COR_ENABLE_PROFILING",
            "COR_PROFILER",
            "COR_PROFILER_PATH",
            "COR_PROFILER_PATH_32",
            "COR_PROFILER_PATH_64",
        ],
        _ => &[
            "CORECLR_ENABLE_PROFILING",
            "CORECLR_PROFILER",
            "CORECLR_PROFILER_PATH",
            "CORECLR_PROFILER_PATH_32",
            "CORECLR_PROFILER_PATH_64",
        ],
    };
    if let Some(h) = r.machine("SYSTEM") {
        let p = format!(
            r"{}\Control\Session Manager\Environment",
            r.ctx.active_controlset()
        );
        if let Some(k) = r.key(h, &p) {
            for n in names {
                if let Some(v) = r.text(k, n) {
                    r.emit(h, k, join(&[r"HKLM\SYSTEM", &p, n]), v, "SYSTEM");
                }
            }
        }
        if r.check.id != "dotnet_startup_hooks" {
            let p = format!(r"{}\Services", r.ctx.active_controlset());
            if let Some(k) = r.key(h, &p) {
                for c in r.children(k) {
                    if let Some(v) = r.value(c, "Environment") {
                        let mut seen = BTreeSet::new();
                        for entry in v.strings() {
                            if let Some((name, val)) = entry.split_once('=')
                                && names.iter().any(|n| n.eq_ignore_ascii_case(name.trim()))
                                && !val.trim().is_empty()
                                && seen.insert(name.trim().to_lowercase())
                            {
                                r.emit(
                                    h,
                                    c,
                                    join(&[
                                        r"HKLM\SYSTEM",
                                        &p,
                                        &c.name(),
                                        "Environment",
                                        name.trim(),
                                    ]),
                                    val.trim(),
                                    "SYSTEM",
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    user_values(r, "Environment", names);
}
fn dsrm(r: &mut CheckRun<'_>) {
    let Some(h) = r.machine("SYSTEM") else {
        return;
    };
    for c in ["ControlSet001", "ControlSet002", "CurrentControlSet"] {
        let p = format!(r"{c}\Control\Lsa");
        if let Some(k) = r.key(h, &p)
            && matches!(
                r.value(k, "DsrmAdminLogonBehavior"),
                Some(RegData::Number(2))
            )
        {
            r.emit(
                h,
                k,
                join(&[r"HKLM\SYSTEM", &p, "DsrmAdminLogonBehavior"]),
                "2",
                "SYSTEM",
            );
        }
    }
}
fn lsa_password_filter(r: &mut CheckRun<'_>) {
    r.declarative();
    for f in &mut r.findings {
        let s = f.value.trim().trim_matches('"');
        if !s.contains(['\\', '/', ':']) && !s.is_empty() {
            f.resolve_target = format!(
                r"Windows\System32\{s}{}",
                if s.to_lowercase().ends_with(".dll") {
                    ""
                } else {
                    ".dll"
                }
            );
        }
    }
}
fn rdp_channels(r: &mut CheckRun<'_>) {
    let Some(h) = r.machine("SOFTWARE") else {
        return;
    };
    let p = r"Microsoft\Terminal Server Client\Default\AddIns";
    if let Some(k) = r.key(h, p) {
        for c in r.children(k) {
            for (n, v) in r.values(c) {
                let text = if let RegData::Bytes(b) = v {
                    String::from_utf16_lossy(
                        &b.as_chunks::<2>()
                            .0
                            .iter()
                            .map(|b| u16::from_le_bytes([b[0], b[1]]))
                            .collect::<Vec<_>>(),
                    )
                    .trim_matches('\0')
                    .to_string()
                } else {
                    v.text()
                };
                if !text.trim().is_empty() {
                    r.emit(
                        h,
                        c,
                        join(&[r"HKLM\SOFTWARE", p, &c.name(), &n]),
                        text,
                        "SYSTEM",
                    );
                }
            }
        }
    }
}
fn minidump(r: &mut CheckRun<'_>) {
    let Some(h) = r.machine("SOFTWARE") else {
        return;
    };
    let p = r"Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls";
    if let Some(k) = r.key(h, p) {
        for (n, v) in r.values(k) {
            if !n.trim().is_empty() && !v.text().trim().is_empty() {
                let i = r.emit(h, k, join(&[r"HKLM\SOFTWARE", p, &n]), v.text(), "SYSTEM");
                r.findings[i]
                    .description
                    .push_str(&format!(" (trigger module: {n})"));
            }
        }
    }
}
fn wer_runtime(r: &mut CheckRun<'_>) {
    let Some(h) = r.machine("SOFTWARE") else {
        return;
    };
    for prefix in ["", r"Wow6432Node"] {
        let p = join(&[
            prefix,
            r"Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules",
        ]);
        if let Some(k) = r.key(h, &p) {
            for (n, _) in r.values(k) {
                if !n.trim().is_empty() {
                    r.emit(h, k, join(&[r"HKLM\SOFTWARE", &p, &n]), n, "SYSTEM");
                }
            }
        }
    }
}
fn shim(r: &mut CheckRun<'_>) {
    let Some(h) = r.machine("SOFTWARE") else {
        return;
    };
    let p = r"Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom";
    if let Some(k) = r.key(h, p) {
        for c in r.children(k) {
            for (n, _) in r.values(c) {
                if n.to_lowercase().ends_with(".sdb") {
                    r.emit(
                        h,
                        c,
                        join(&[r"HKLM\SOFTWARE", p, &c.name()]),
                        format!("{} -> {n}", c.name()),
                        "SYSTEM",
                    );
                }
            }
        }
    }
}

struct ClassView<'a> {
    h: &'a HiveInput,
    root: String,
    display: String,
    access: &'static str,
}
fn classes<'a>(r: &mut CheckRun<'a>, ntuser: bool, wow: bool) -> Vec<ClassView<'a>> {
    let mut views = vec![];
    if let Some(h) = r.machine("SOFTWARE") {
        views.push(ClassView {
            h,
            root: "Classes".into(),
            display: r"HKLM\SOFTWARE\Classes".into(),
            access: "SYSTEM",
        });
        if wow {
            views.push(ClassView {
                h,
                root: r"Classes\Wow6432Node".into(),
                display: r"HKLM\SOFTWARE\Classes\Wow6432Node".into(),
                access: "SYSTEM",
            });
        }
    }
    for h in r.users("USRCLASS.DAT") {
        views.push(ClassView {
            h,
            root: String::new(),
            display: join(&["HKU", &h.user, r"Software\Classes"]),
            access: "USER",
        });
        if wow {
            views.push(ClassView {
                h,
                root: "Wow6432Node".into(),
                display: join(&["HKU", &h.user, r"Software\Classes\Wow6432Node"]),
                access: "USER",
            });
        }
    }
    if ntuser {
        for h in r.users("NTUSER.DAT") {
            views.push(ClassView {
                h,
                root: r"Software\Classes".into(),
                display: join(&["HKU", &h.user, r"Software\Classes"]),
                access: "USER",
            });
            if wow {
                views.push(ClassView {
                    h,
                    root: r"Software\Classes\Wow6432Node".into(),
                    display: join(&["HKU", &h.user, r"Software\Classes\Wow6432Node"]),
                    access: "USER",
                });
            }
        }
    }
    views
}
fn context_menus(r: &mut CheckRun<'_>) {
    let views = classes(r, false, false);
    for v in &views {
        for subject in [
            "Directory",
            r"Directory\Background",
            "Folder",
            "Drive",
            "AllFilesystemObjects",
            "*",
        ] {
            let p = join(&[&v.root, subject, r"shellex\ContextMenuHandlers"]);
            let Some(k) = r.key(v.h, &p) else {
                continue;
            };
            for c in r.children(k) {
                let Some(mut value) = r.text(c, "(Default)") else {
                    continue;
                };
                let mut dll = r.clsid_server(v.h, &v.root, &value);
                if dll.is_empty() && v.access == "USER" {
                    for m in views.iter().filter(|v| v.access == "SYSTEM") {
                        dll = r.clsid_server(m.h, &m.root, &value);
                        if !dll.is_empty() {
                            break;
                        }
                    }
                }
                if !dll.is_empty() {
                    value = dll;
                } else if !value.contains('\\') && !value.starts_with('{') {
                    continue;
                }
                r.emit(
                    v.h,
                    c,
                    join(&[
                        &v.display,
                        subject,
                        r"shellex\ContextMenuHandlers",
                        &c.name(),
                    ]),
                    value,
                    v.access,
                );
            }
        }
    }
}
fn file_associations(r: &mut CheckRun<'_>) {
    for v in classes(r, false, false) {
        for ext in [
            ".txt", ".pdf", ".doc", ".docx", ".html", ".htm", ".js", ".vbs", ".hta", ".exe",
            ".bat", ".cmd", ".ps1",
        ] {
            let mut keys = vec![ext.to_string()];
            if let Some(k) = r.key(v.h, &join(&[&v.root, ext]))
                && let Some(progid) = r.text(k, "(Default)")
                && !progid.contains('\\')
                && !progid.starts_with('"')
            {
                keys.push(progid);
            }
            for name in keys {
                let p = join(&[&v.root, &name, r"shell\open\command"]);
                if let Some(k) = r.key(v.h, &p)
                    && let Some(cmd) = r.text(k, "(Default)")
                {
                    r.emit(
                        v.h,
                        k,
                        join(&[&v.display, &name, r"shell\open\command"]),
                        cmd,
                        v.access,
                    );
                }
            }
        }
    }
}
fn protocols(r: &mut CheckRun<'_>) {
    let known = ["http", "https", "mailto", "ms-msdt", "ms-officecmd"];
    let mut machine_protocols = BTreeSet::new();
    for v in classes(r, false, false) {
        let mut names: BTreeSet<String> = known.iter().map(|s| s.to_string()).collect();
        if let Some(root) = r.key(v.h, &v.root) {
            for c in r.children(root) {
                let n = c.name();
                let marker = r.value(c, "URL Protocol").is_some();
                if marker {
                    names.insert(n.clone());
                    if v.access == "SYSTEM" {
                        machine_protocols.insert(n.to_lowercase());
                    }
                } else if v.access == "USER" && machine_protocols.contains(&n.to_lowercase()) {
                    names.insert(n);
                }
            }
        }
        for n in names {
            let p = join(&[&v.root, &n, r"shell\open\command"]);
            if let Some(k) = r.key(v.h, &p)
                && let Some(cmd) = r.text(k, "(Default)")
            {
                r.emit(
                    v.h,
                    k,
                    join(&[&v.display, &n, r"shell\open\command"]),
                    cmd,
                    v.access,
                );
            }
        }
    }
}
fn com_hijacks(r: &mut CheckRun<'_>) {
    let views = classes(r, true, true);
    let mut machine_clsids = BTreeSet::new();
    for v in views.iter().filter(|v| v.access == "SYSTEM") {
        if let Some(k) = r.key(v.h, &join(&[&v.root, "CLSID"])) {
            for c in r.children(k) {
                machine_clsids.insert(c.name().to_lowercase());
            }
        }
    }
    for v in &views {
        let Some(root) = r.key(v.h, &join(&[&v.root, "CLSID"])) else {
            continue;
        };
        for clsid in r.children(root) {
            let class_name = clsid.name();
            for name in ["TreatAs", "InprocServer32", "LocalServer32"] {
                if name != "TreatAs"
                    && (v.access != "USER" || !machine_clsids.contains(&class_name.to_lowercase()))
                {
                    continue;
                }
                let Some(k) = r.child(clsid, name) else {
                    continue;
                };
                let Some(value) = r.text(k, "(Default)") else {
                    continue;
                };
                let mut target = String::new();
                if name == "TreatAs" {
                    // A user's registration may only resolve through that profile and machine stores.
                    // Never attach another user's private DLL to the current user's finding.
                    let mut eligible: Vec<_> = views
                        .iter()
                        .filter(|x| {
                            x.access == "SYSTEM"
                                || (v.access == "USER" && x.h.user.eq_ignore_ascii_case(&v.h.user))
                        })
                        .collect();
                    eligible.sort_by_key(|x| if x.access == "USER" { 0 } else { 1 });
                    for x in eligible {
                        if let Some(c) = r.key(x.h, &join(&[&x.root, "CLSID", &value])) {
                            for server in ["InprocServer32", "LocalServer32"] {
                                if let Some(s) = r.child(c, server)
                                    && let Some(image) = r.text(s, "(Default)")
                                {
                                    target = image;
                                    break;
                                }
                            }
                        }
                        if !target.is_empty() {
                            break;
                        }
                    }
                }
                let i = r.emit(
                    v.h,
                    k,
                    join(&[&v.display, "CLSID", &class_name, name]),
                    value,
                    v.access,
                );
                r.findings[i].resolve_target = target;
                if name != "TreatAs" {
                    r.findings[i].description="Per-user COM server shadows a machine-wide registration. The override needs no administrator access; configuration does not prove that activation occurred.".into();
                }
            }
        }
    }
}
fn typelib(r: &mut CheckRun<'_>) {
    for h in r.users("USRCLASS.DAT") {
        let Some(root) = r.key(h, "TypeLib") else {
            continue;
        };
        for guid in r.children(root) {
            for version in r.children(guid) {
                for lcid in r.children(version) {
                    for platform in ["win32", "win64"] {
                        let Some(k) = r.child(lcid, platform) else {
                            continue;
                        };
                        let Some(value) = r.text(k, "(Default)") else {
                            continue;
                        };
                        let i = r.emit(
                            h,
                            k,
                            join(&[
                                "HKU",
                                &h.user,
                                r"Software\Classes\TypeLib",
                                &guid.name(),
                                &version.name(),
                                &lcid.name(),
                                platform,
                            ]),
                            &value,
                            "USER",
                        );
                        let path = if let Some((left, tail)) = value.rsplit_once('\\') {
                            if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) {
                                left
                            } else {
                                &value
                            }
                        } else {
                            &value
                        };
                        if !path.to_lowercase().starts_with("script:") {
                            r.findings[i].resolve_target = expand_user_env(path, &h.user);
                        }
                    }
                }
            }
        }
    }
}
fn office_overrides(r: &mut CheckRun<'_>) {
    let mut roots = vec![];
    if let Some(h) = r.machine("SOFTWARE") {
        for p in [
            r"Microsoft\Office",
            r"Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office",
        ] {
            roots.push((h, p.to_string(), r"HKLM\SOFTWARE".to_string(), "SYSTEM"));
        }
    }
    for h in r.users("NTUSER.DAT") {
        roots.push((
            h,
            r"Software\Microsoft\Office".into(),
            join(&["HKU", &h.user]),
            "USER",
        ));
    }
    for (h, root, display, access) in roots {
        for ver in ["", "14.0", "15.0", "16.0"] {
            for app in ["Word", "PowerPoint"] {
                let p = join(&[&root, ver, app]);
                if let Some(k) = r.key(h, &p) {
                    for n in ["WwlibtDll", "PPCoreTDLL"] {
                        if let Some(v) = r.text(k, n) {
                            r.emit(h, k, join(&[&display, &p, n]), v, access);
                        }
                    }
                }
            }
        }
    }
}
fn office_addins(r: &mut CheckRun<'_>) {
    const C2R: &str = r"Microsoft\Office\ClickToRun\REGISTRY\MACHINE";
    let mut roots = vec![];
    let mut machine_classes = vec![];
    if let Some(h) = r.machine("SOFTWARE") {
        for p in [
            "Classes".to_string(),
            r"Classes\Wow6432Node".into(),
            join(&[C2R, r"Software\Classes"]),
            join(&[C2R, r"Software\Classes\Wow6432Node"]),
        ] {
            machine_classes.push((h, p));
        }
        for p in [
            r"Microsoft\Office".to_string(),
            join(&[C2R, r"Software\Microsoft\Office"]),
        ] {
            roots.push((h, p, r"HKLM\SOFTWARE".to_string(), "SYSTEM"));
        }
    }
    let user_classes = r.users("USRCLASS.DAT");
    for h in r.users("NTUSER.DAT") {
        roots.push((
            h,
            r"Software\Microsoft\Office".into(),
            join(&["HKU", &h.user]),
            "USER",
        ));
    }
    for (h, root, display, access) in roots {
        let mut class_roots = vec![];
        if access == "USER" {
            for p in [r"Software\Classes", r"Software\Classes\Wow6432Node"] {
                class_roots.push((h, p.to_string()));
            }
            for uh in user_classes
                .iter()
                .filter(|u| u.user.eq_ignore_ascii_case(&h.user))
            {
                for p in ["", "Wow6432Node"] {
                    class_roots.push((*uh, p.to_string()));
                }
            }
        }
        class_roots.extend(machine_classes.iter().cloned());
        for app in ["Word", "Excel", "PowerPoint", "Outlook", "Access"] {
            for version in ["", "14.0", "15.0", "16.0"] {
                let p = join(&[&root, version, app, "Addins"]);
                let Some(k) = r.key(h, &p) else {
                    continue;
                };
                for c in r.children(k) {
                    let name = c.name();
                    let behavior = r.value(c, "LoadBehavior");
                    let desc = format!(
                        "{} (LoadBehavior: {})",
                        r.check.description,
                        behavior
                            .as_ref()
                            .map(|v| v.text())
                            .unwrap_or_else(|| "absent".into())
                    );
                    let mut explicit = false;
                    for n in ["Manifest", "FileName", "Path"] {
                        if let Some(v) = r.value(c, n) {
                            explicit = true;
                            let i = r.emit(h, c, join(&[&display, &p, &name, n]), v.text(), access);
                            r.findings[i].description = desc.clone();
                        }
                    }
                    if explicit || behavior.is_none() {
                        continue;
                    }
                    let mut clsid = if name.starts_with('{') {
                        name.clone()
                    } else {
                        String::new()
                    };
                    if clsid.is_empty() {
                        for (ch, cr) in &class_roots {
                            let v = r.default_server(ch, &join(&[cr, &name, "CLSID"]));
                            if v.starts_with('{') {
                                clsid = v;
                                break;
                            }
                        }
                    }
                    if clsid.is_empty() {
                        continue;
                    }
                    for (ch, cr) in &class_roots {
                        let v =
                            r.default_server(ch, &join(&[cr, "CLSID", &clsid, "InprocServer32"]));
                        if !v.trim().is_empty() {
                            let i = r.emit(h, c, join(&[&display, &p, &name]), v, access);
                            r.findings[i].description = format!("{desc}; COM server {clsid}");
                            break;
                        }
                    }
                }
            }
        }
    }
}
fn outlook(r: &mut CheckRun<'_>) {
    for h in r.users("NTUSER.DAT") {
        for ver in ["14.0", "15.0", "16.0"] {
            let p = format!(r"Software\Microsoft\Office\{ver}\Outlook\WebView");
            if let Some(k) = r.key(h, &p) {
                for c in r.children(k) {
                    if let Some(v) = r.text(c, "URL") {
                        r.emit(
                            h,
                            c,
                            join(&["HKU", &h.user, &p, &c.name(), "URL"]),
                            v,
                            "USER",
                        );
                    }
                }
            }
        }
    }
}
fn vba(r: &mut CheckRun<'_>) {
    for view in classes(r, false, false) {
        for clsid in [
            "{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}",
            "{0002E157-0000-0000-C000-000000000046}",
        ] {
            let p = join(&[&view.root, "CLSID", clsid, "InprocServer32"]);
            if let Some(k) = r.key(view.h, &p)
                && let Some(v) = r.text(k, "(Default)")
            {
                r.emit(
                    view.h,
                    k,
                    join(&[&view.display, "CLSID", clsid, "InprocServer32"]),
                    v,
                    view.access,
                );
            }
        }
    }
}

fn hidden_names(r: &mut CheckRun<'_>) -> BTreeSet<String> {
    let Some(h) = r.machine("SOFTWARE") else {
        return BTreeSet::new();
    };
    let Some(k) = r.key(h, HIDDEN_KEY) else {
        return BTreeSet::new();
    };
    r.values(k)
        .into_iter()
        .filter_map(|(n, v)| {
            if !n.is_empty() && matches!(v, RegData::Number(0)) {
                Some(n)
            } else {
                None
            }
        })
        .collect()
}
const HIDDEN_KEY: &str = r"Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList";
fn hidden_accounts(r: &mut CheckRun<'_>) {
    let names = hidden_names(r);
    let Some(h) = r.machine("SOFTWARE") else {
        return;
    };
    let Some(k) = r.key(h, HIDDEN_KEY) else {
        return;
    };
    for n in names {
        r.emit(
            h,
            k,
            join(&[r"HKLM\SOFTWARE", HIDDEN_KEY, &n]),
            format!("{n} is hidden from the sign-in screen"),
            "SYSTEM",
        );
    }
}
fn rid_checks(r: &mut CheckRun<'_>) {
    let Some(h) = r.machine("SAM") else {
        return;
    };
    let p = r"SAM\Domains\Account\Users";
    let Some(k) = r.key(h, p) else {
        return;
    };
    for c in r.children(k) {
        let name = c.name();
        let Ok(actual) = u32::from_str_radix(&name, 16) else {
            continue;
        };
        let Some(RegData::Bytes(f)) = r.value(c, "F") else {
            continue;
        };
        let Some(effective) = dword(&f, 0x30) else {
            r.issue(format!("SAM account {name} has a truncated F value"));
            continue;
        };
        let value = if r.check.id == "rid_hijacking" && actual != effective {
            Some(format!(
                "RID mismatch: subkey=0x{actual:X} ({actual}), F value=0x{effective:X} ({effective})"
            ))
        } else if r.check.id == "rid_suborner" && actual != 500 && effective == 500 {
            Some(format!(
                "Potential Suborner: account 0x{actual:X} has F-value RID=500"
            ))
        } else {
            None
        };
        if let Some(value) = value {
            r.emit(h, c, join(&["HKLM", p, &name, "F"]), value, "SYSTEM");
        }
    }
}
fn dword(b: &[u8], p: usize) -> Option<u32> {
    let v = b.get(p..p.checked_add(4)?)?;
    Some(u32::from_le_bytes(v.try_into().ok()?))
}
fn sid(b: &[u8], p: usize) -> Option<(String, usize)> {
    let hdr = b.get(p..p.checked_add(8)?)?;
    let n = usize::from(hdr[1]);
    if hdr[0] != 1 || !(1..=15).contains(&n) {
        return None;
    }
    let end = p.checked_add(8)?.checked_add(n.checked_mul(4)?)?;
    b.get(..end)?;
    let mut auth = 0u64;
    for c in &hdr[2..8] {
        auth = (auth << 8) | u64::from(*c);
    }
    let mut s = format!("S-1-{auth}");
    for i in 0..n {
        s.push_str(&format!("-{}", dword(b, p + 8 + i * 4)?));
    }
    Some((s, end))
}
fn admin_members(r: &mut CheckRun<'_>) {
    let hidden: BTreeSet<_> = hidden_names(r)
        .into_iter()
        .map(|n| n.to_lowercase())
        .collect();
    let Some(h) = r.machine("SAM") else {
        return;
    };
    let account = r"SAM\Domains\Account";
    let Some(a) = r.key(h, account) else {
        return;
    };
    let Some(RegData::Bytes(v)) = r.value(a, "V") else {
        return;
    };
    let Some(start) = v.len().checked_sub(24) else {
        r.issue("SAM Account V value is too short for the machine SID");
        return;
    };
    let Some((domain, _)) = sid(&v, start) else {
        r.issue("SAM machine SID could not be decoded");
        return;
    };
    let users = join(&[account, "Users"]);
    let mut names = BTreeMap::new();
    let mut groups = BTreeSet::new();
    if let Some(k) = r.key(h, &users) {
        for c in r.children(k) {
            let Ok(rid) = u32::from_str_radix(&c.name(), 16) else {
                continue;
            };
            if let Some(RegData::Bytes(v)) = r.value(c, "V") {
                let parsed = registry::sam::user_v(&v);
                for p in parsed.1 {
                    r.issue(p);
                }
                names.insert(rid, parsed.0.username.unwrap_or_default());
            }
        }
    }
    for container in ["Aliases", "Groups"] {
        if let Some(k) = r.key(h, &join(&[account, container])) {
            for c in r.children(k) {
                if let Ok(rid) = u32::from_str_radix(&c.name(), 16) {
                    groups.insert(rid);
                }
            }
        }
    }
    let path = r"SAM\Domains\Builtin\Aliases\00000220";
    let Some(k) = r.key(h, path) else {
        return;
    };
    let Some(RegData::Bytes(c)) = r.value(k, "C") else {
        return;
    };
    let (Some(offset), Some(len)) = (dword(&c, 0x28), dword(&c, 0x2c)) else {
        r.issue("Administrators alias C value is truncated");
        return;
    };
    let Some(start) = 0x34usize.checked_add(offset as usize) else {
        return;
    };
    let Some(end) = start.checked_add(len as usize) else {
        return;
    };
    let Some(members) = c.get(start..end) else {
        r.issue("Administrators alias member list is truncated");
        return;
    };
    let mut p = 0;
    while p < members.len() {
        let Some((member, next)) = sid(members, p) else {
            r.issue("Invalid SID in Administrators alias member list");
            break;
        };
        p = next;
        let Some(rid) = member
            .strip_prefix(&format!("{domain}-"))
            .and_then(|s| s.parse::<u32>().ok())
        else {
            continue;
        };
        let reason = if let Some(name) = names.get(&rid) {
            if hidden.contains(&name.to_lowercase()) {
                format!(
                    "{name} ({member}) is a member of Administrators and is hidden from the sign-in screen"
                )
            } else {
                String::new()
            }
        } else if !groups.contains(&rid) {
            format!(
                "{member} is a member of Administrators but names no account or group in the SAM"
            )
        } else {
            String::new()
        };
        if !reason.is_empty() {
            r.emit(h, k, join(&["HKLM", path, "C"]), reason, "SYSTEM");
        }
    }
}

fn expand_user_env(s: &str, user: &str) -> String {
    let mut result = s.to_string();
    for (k, v) in [
        ("%appdata%", format!(r"Users\{user}\AppData\Roaming")),
        ("%localappdata%", format!(r"Users\{user}\AppData\Local")),
        ("%userprofile%", format!(r"Users\{user}")),
    ] {
        loop {
            let lower = result.to_lowercase();
            let Some(i) = lower.find(k) else {
                break;
            };
            result.replace_range(i..i + k.len(), &v);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sam_sid_bounds() {
        let b = [1, 2, 0, 0, 0, 0, 0, 5, 21, 0, 0, 0, 0xf4, 1, 0, 0];
        assert_eq!(sid(&b, 0), Some(("S-1-5-21-500".into(), 16)));
        assert_eq!(sid(&b[..15], 0), None);
        assert_eq!(sid(&b, usize::MAX), None);
        assert_eq!(dword(&b, usize::MAX), None);
    }
    #[test]
    fn user_expansion_is_case_insensitive() {
        assert_eq!(
            expand_user_env(r"%USERPROFILE%\run.dll", "Alice"),
            r"Users\Alice\run.dll"
        );
    }
    #[test]
    fn service_image_aliases_are_compared_conservatively() {
        let expected = normalize_service_image(r"%SystemRoot%\System32\svchost.exe -k netsvcs");
        for command in [
            r#""C:\Windows\System32\SVCHOST.EXE"   -k NETSVCS"#,
            r"\SystemRoot\System32\svchost.exe -k netsvcs",
            r"System32\svchost.exe -k netsvcs",
        ] {
            assert_eq!(normalize_service_image(command), expected);
        }
        assert_ne!(
            normalize_service_image(r"C:\Tools\service.exe -k unique"),
            expected
        );
    }
}
