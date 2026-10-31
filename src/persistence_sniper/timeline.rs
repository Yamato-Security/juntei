//! Native, artifact-backed change-time correlation.

use super::{
    Finding, TimeEvidence,
    context::{Context, EvidenceFile, explicit_drive, normalize_windows},
};
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use evtx::{EvtxParser, ParserSettings};
use mft::{MftParser, attribute::MftAttributeType, csv::FlatMftEntryWithName};
use rayon::prelude::*;
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
struct Candidate {
    when: DateTime<Utc>,
    source: String,
    detail: String,
    weak: bool,
    mft: bool,
}

#[derive(Default)]
struct State {
    declared: bool,
    available: bool,
    rejected: bool,
    candidates: [Option<Candidate>; 4],
}

impl State {
    fn add(&mut self, candidate: Candidate, now: DateTime<Utc>) {
        if candidate.when.timestamp() < 631_152_000 || candidate.when > now + Duration::hours(48) {
            self.rejected = true;
            return;
        }
        let slot = usize::from(candidate.weak) * 2 + usize::from(!candidate.mft);
        let old = &mut self.candidates[slot];
        if old.as_ref().is_none_or(|old| {
            candidate.when > old.when || candidate.when == old.when && candidate.source < old.source
        }) {
            *old = Some(candidate);
        }
    }
    fn winner(&self) -> Option<&Candidate> {
        for pair in self.candidates.as_chunks::<2>().0 {
            match (&pair[0], &pair[1]) {
                (Some(mft), Some(event)) => {
                    return Some(if mft.when + Duration::seconds(2) >= event.when {
                        mft
                    } else {
                        event
                    });
                }
                (Some(mft), None) => return Some(mft),
                (None, Some(event)) => return Some(event),
                (None, None) => {}
            }
        }
        None
    }
}

fn empty_states(count: usize) -> Vec<State> {
    (0..count).map(|_| State::default()).collect()
}

fn merge_states(states: &mut [State], incoming: Vec<State>, now: DateTime<Utc>) {
    for (state, incoming) in states.iter_mut().zip(incoming) {
        state.available |= incoming.available;
        state.rejected |= incoming.rejected;
        for candidate in incoming.candidates.into_iter().flatten() {
            state.add(candidate, now);
        }
    }
}

struct SourceScan {
    states: Vec<State>,
    issues: Vec<String>,
    unreadable: usize,
}

impl SourceScan {
    fn new(count: usize) -> Self {
        Self {
            states: empty_states(count),
            issues: Vec::new(),
            unreadable: 0,
        }
    }
}

#[derive(Hash, PartialEq, Eq)]
struct EventKey {
    channel: String,
    event_id: u64,
    field: String,
    value: String,
}

const MAX_RENAME_PROOFS: usize = 4096;
const MAX_RENAME_CHAIN: usize = 64;

#[derive(Clone, Debug)]
struct Rename {
    old: String,
    new: String,
    source: String,
    record_id: u64,
}

struct HostAliases {
    current: String,
    /// Historical name -> the next name toward the captured host and its proof event.
    parents: HashMap<String, (String, usize)>,
    proofs: Vec<Rename>,
}

impl HostAliases {
    fn new(current: &str, proofs: Vec<Rename>) -> Self {
        let current = host_key(current).unwrap_or_default();
        let mut aliases = Self {
            current: current.clone(),
            parents: HashMap::new(),
            proofs,
        };
        let mut pending = VecDeque::from([(current.clone(), 0usize)]);
        let mut known = HashSet::from([current]);
        while let Some((name, depth)) = pending.pop_front() {
            if name.is_empty() || depth >= MAX_RENAME_CHAIN {
                continue;
            }
            for (i, proof) in aliases.proofs.iter().enumerate() {
                let adjacent = if proof.new == name {
                    &proof.old
                } else if proof.old == name {
                    &proof.new
                } else {
                    continue;
                };
                if known.insert(adjacent.clone()) {
                    aliases.parents.insert(adjacent.clone(), (name.clone(), i));
                    pending.push_back((adjacent.clone(), depth + 1));
                }
            }
        }
        aliases
    }

    fn accepts(&self, computer: &str) -> bool {
        host_key(computer).is_some_and(|name| {
            !self.current.is_empty() && (name == self.current || self.parents.contains_key(&name))
        })
    }

    fn proof(&self, computer: &str) -> Option<String> {
        let mut name = host_key(computer)?;
        if name == self.current {
            return None;
        }
        let mut proof_lines = Vec::new();
        for _ in 0..MAX_RENAME_CHAIN {
            let (next, proof) = self.parents.get(&name)?;
            let proof = &self.proofs[*proof];
            proof_lines.push(format!(
                "{} RecordID {} ({} -> {})",
                proof.source, proof.record_id, proof.old, proof.new
            ));
            name = next.clone();
            if name == self.current {
                return Some(format!(
                    "original event Computer={computer}; captured-host rename chain: {}",
                    proof_lines.join("; ")
                ));
            }
        }
        None
    }
}

fn host_key(name: &str) -> Option<String> {
    let name = name.trim().trim_end_matches('.');
    if name.is_empty()
        || name.encode_utf16().count() > 255
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return None;
    }
    let first = name.split('.').next()?;
    (!first.is_empty()).then(|| first.to_lowercase())
}

/// Only the primary installation's live channel files establish local-host identity.
/// Windows.old, copied exports and forwarded/archived logs cannot introduce aliases.
fn active_channel_log(ctx: &Context, file: &EvidenceFile, channel: &str) -> bool {
    let lower = file.logical.to_lowercase();
    lower
        .strip_prefix("windows/system32/winevt/logs/")
        .is_some_and(|name| !name.contains('/') && event_filename_matches(name, channel))
        && ctx.drive_of(&file.physical) == ctx.system_drive()
}

fn historical_hosts(ctx: &Context, now: DateTime<Utc>, issues: &mut Vec<String>) -> HostAliases {
    let mut proofs = Vec::new();
    for source in ctx
        .files
        .iter()
        .filter(|file| active_channel_log(ctx, file, "System"))
    {
        let mut parser = match EvtxParser::from_path(&source.physical) {
            Ok(parser) => parser.with_configuration(ParserSettings::new().num_threads(1)),
            Err(e) => {
                issues.push(format!(
                    "Cannot read host-rename evidence {}: {e}",
                    source.physical.display()
                ));
                continue;
            }
        };
        let mut unreadable = 0usize;
        for record in parser.records_json_value() {
            let record = match record {
                Ok(record) => record,
                Err(_) => {
                    unreadable += 1;
                    continue;
                }
            };
            let event = record.data.get("Event").unwrap_or(&record.data);
            let Some(when) = DateTime::from_timestamp(
                record.timestamp.as_second(),
                record.timestamp.subsec_nanosecond() as u32,
            ) else {
                continue;
            };
            if let Some(proof) = rename_event(
                event,
                when,
                now,
                source.physical.display().to_string(),
                record.event_record_id,
            ) {
                if proofs.len() >= MAX_RENAME_PROOFS {
                    issues.push(format!("Host rename evidence exceeds {MAX_RENAME_PROOFS} events; additional rename proofs are not used"));
                    break;
                }
                proofs.push(proof);
            }
        }
        if unreadable > 0 {
            issues.push(format!(
                "Host-rename EVTX {} skipped {unreadable} unreadable records",
                source.physical.display()
            ));
        }
    }
    HostAliases::new(&ctx.computer, proofs)
}

fn rename_event(
    event: &Value,
    when: DateTime<Utc>,
    now: DateTime<Utc>,
    source: String,
    record_id: u64,
) -> Option<Rename> {
    if when.timestamp() < 631_152_000 || when > now + Duration::hours(48) {
        return None;
    }
    let system = event.get("System")?;
    if scalar(system.get("EventID"))?.parse::<u64>().ok()? != 6011
        || !scalar(system.get("Channel"))?.eq_ignore_ascii_case("System")
    {
        return None;
    }
    let provider = system
        .get("Provider")
        .and_then(|value| value.get("#attributes"))
        .and_then(|value| value.get("Name"))
        .or_else(|| {
            system
                .get("Provider_attributes")
                .and_then(|value| value.get("Name"))
        })?
        .as_str()?;
    if !provider.eq_ignore_ascii_case("EventLog") {
        return None;
    }
    let data = event.get("EventData")?.get("Data")?;
    // Native EVTX rendering represents the two unnamed insertion strings as a
    // #text array; XML renderers may instead preserve the <string> fragment.
    let data = data.get("#text").unwrap_or(data);
    let names = if let Value::Array(values) = data {
        let names: Vec<_> = values
            .iter()
            .map(|value| scalar(Some(value)))
            .collect::<Option<_>>()?;
        if names.len() == 2 && names.iter().all(|name| host_key(name).is_some()) {
            names
        } else if names.len() == 1 {
            rename_strings(&names[0])?
        } else {
            return None;
        }
    } else {
        rename_strings(&scalar(Some(data))?)?
    };
    let old = host_key(names.first()?)?;
    let new = host_key(names.get(1)?)?;
    let computer = host_key(&scalar(system.get("Computer"))?)?;
    if old == new || computer != old && computer != new {
        return None;
    }
    Some(Rename {
        old,
        new,
        source,
        record_id,
    })
}

fn rename_strings(fragment: &str) -> Option<Vec<String>> {
    if fragment.len() > 4096 {
        return None;
    }
    let mut reader = quick_xml::Reader::from_str(fragment);
    let mut names = Vec::new();
    loop {
        match reader.read_event().ok()? {
            quick_xml::events::Event::Start(tag)
                if tag.name().as_ref() == b"string" && tag.attributes().next().is_none() =>
            {
                names.push(reader.read_text(tag.name()).ok()?.into_owned());
                if names.len() > 2 {
                    return None;
                }
            }
            quick_xml::events::Event::Text(text) if text.decode().ok()?.trim().is_empty() => {}
            quick_xml::events::Event::Eof => break,
            _ => return None,
        }
    }
    (names.len() == 2).then_some(names)
}

/// Parse each relevant channel and MFT once. The indices contain finding identifiers and
/// four best candidates per finding; event records and the entire MFT are never retained.
pub fn enrich(ctx: &Context, findings: &mut [Finding], explicit_mft: Option<&Path>) -> Vec<String> {
    let mut issues = Vec::new();
    let mut states: Vec<State> = (0..findings.len()).map(|_| State::default()).collect();
    let mut file_targets: HashMap<String, Vec<(usize, bool)>> = HashMap::new();
    let mut event_targets: HashMap<EventKey, Vec<usize>> = HashMap::new();
    for (i, finding) in findings.iter().enumerate() {
        let descriptors = if finding.time_evidence.is_empty() {
            fallback(ctx, finding)
        } else {
            finding.time_evidence.clone()
        };
        states[i].declared = !descriptors.is_empty();
        for descriptor in descriptors {
            match descriptor {
                TimeEvidence::FileWrite { path, weak } => {
                    if let Ok(key) = normalize_windows(&path) {
                        let drive = explicit_drive(&path)
                            .unwrap_or_else(|| ctx.drive_of(Path::new(&finding.source_file)));
                        file_targets
                            .entry(format!("{drive}:{}", key.to_lowercase()))
                            .or_default()
                            .push((i, weak));
                    }
                }
                TimeEvidence::EventLog {
                    channel,
                    event_ids,
                    match_field,
                    match_value,
                } => {
                    for event_id in event_ids {
                        if match_value.is_empty() || match_field.is_empty() {
                            continue;
                        }
                        event_targets
                            .entry(EventKey {
                                channel: channel.to_lowercase(),
                                event_id,
                                field: match_field.to_lowercase(),
                                value: normalize_match(&match_value),
                            })
                            .or_default()
                            .push(i);
                    }
                }
            }
        }
    }
    let now = Utc::now();
    let (file_result, event_result) = rayon::join(
        || {
            let mut result = SourceScan::new(findings.len());
            if !file_targets.is_empty() {
                scan_mft(
                    ctx,
                    explicit_mft,
                    &file_targets,
                    &mut result.states,
                    now,
                    &mut result.issues,
                );
            }
            result
        },
        || {
            let mut result = SourceScan::new(findings.len());
            if !event_targets.is_empty() {
                scan_events(
                    ctx,
                    &event_targets,
                    &mut result.states,
                    now,
                    &mut result.issues,
                );
            }
            result
        },
    );
    // Merge in source order rather than completion order: thread count must
    // not change candidate tie-breaking, missing-source states, or warnings.
    for result in [file_result, event_result] {
        merge_states(&mut states, result.states, now);
        issues.extend(result.issues);
    }
    for (finding, state) in findings.iter_mut().zip(states) {
        finding.last_change.clear();
        finding.change_source.clear();
        if let Some(candidate) = state.winner() {
            finding.last_change = candidate.when.to_rfc3339_opts(SecondsFormat::Nanos, true);
            finding.change_source = format!(
                "{}{}; {}",
                candidate.source,
                if candidate.weak { " (weak)" } else { "" },
                candidate.detail
            );
            finding.change_evidence = "RESOLVED".to_string();
        } else {
            finding.change_evidence = if !state.declared {
                "NOT_APPLICABLE"
            } else if state.rejected {
                "REJECTED"
            } else if state.available {
                "NO_MATCH"
            } else {
                "NO_ARTIFACT"
            }
            .to_string();
        }
    }
    issues
}

fn fallback(ctx: &Context, finding: &Finding) -> Vec<TimeEvidence> {
    if finding.path.to_uppercase().starts_with("HKLM\\")
        || finding.path.to_uppercase().starts_with("HKU\\")
    {
        registry_spellings(ctx, &finding.path)
            .into_iter()
            .map(|path| TimeEvidence::EventLog {
                channel: "Microsoft-Windows-Sysmon/Operational".to_string(),
                event_ids: vec![13],
                match_field: "TargetObject".to_string(),
                match_value: path,
            })
            .collect()
    } else if !finding.path.is_empty() {
        vec![TimeEvidence::FileWrite {
            path: if !finding.resolve_target.is_empty() {
                finding.resolve_target.clone()
            } else {
                finding.path.clone()
            },
            weak: false,
        }]
    } else {
        Vec::new()
    }
}

fn registry_spellings(ctx: &Context, path: &str) -> Vec<String> {
    let mut path = path.to_string();
    if path.to_uppercase().starts_with("HKU\\") {
        let (owner, tail) = path[4..].split_once('\\').unwrap_or((&path[4..], ""));
        if !owner.to_uppercase().starts_with("S-1-") {
            let sid = ctx.machine_hive("SOFTWARE").and_then(|h| {
                registry::software::profile_list(&h.opened.hive)
                    .into_iter()
                    .find(|(_, profile)| {
                        profile
                            .rsplit(['\\', '/'])
                            .next()
                            .is_some_and(|user| user.eq_ignore_ascii_case(owner))
                    })
                    .map(|(sid, _)| sid)
            });
            let Some(sid) = sid else { return Vec::new() };
            path = format!("HKU\\{sid}\\{tail}");
        }
    }
    if path.ends_with("\\(Default)") {
        path.truncate(path.len() - "(Default)".len());
        path.push_str("(Default)");
    }
    let mut paths = vec![path.clone()];
    let lower = path.to_lowercase();
    if let Some(at) = lower.find("\\controlset") {
        let start = at + "\\controlset".len();
        if path
            .as_bytes()
            .get(start..start + 3)
            .is_some_and(|s| s.iter().all(u8::is_ascii_digit))
            && path.as_bytes().get(start + 3) == Some(&b'\\')
        {
            paths.push(format!(
                "{}\\CurrentControlSet{}",
                &path[..at],
                &path[start + 3..]
            ));
        }
    } else if let Some(at) = lower.find("\\currentcontrolset\\") {
        let end = at + "\\currentcontrolset".len();
        paths.push(format!(
            "{}\\{}{}",
            &path[..at],
            ctx.active_controlset(),
            &path[end..]
        ));
    }
    paths
}

fn scan_mft(
    ctx: &Context,
    explicit: Option<&Path>,
    targets: &HashMap<String, Vec<(usize, bool)>>,
    states: &mut [State],
    now: DateTime<Utc>,
    issues: &mut Vec<String>,
) {
    let sources: Vec<PathBuf> = if let Some(path) = explicit {
        vec![path.to_path_buf()]
    } else {
        ctx.files
            .iter()
            .filter(|file| {
                file.logical
                    .rsplit('/')
                    .next()
                    .is_some_and(crate::mft::is_mft_name)
            })
            .map(|file| file.physical.clone())
            .collect()
    };
    if sources.is_empty() {
        return;
    }
    let names: HashSet<_> = targets
        .keys()
        .filter_map(|path| path.split_once(':').map(|(_, path)| path))
        .filter_map(|path| path.rsplit('/').next().map(str::to_string))
        .collect();
    let results: Vec<_> = sources
        .par_iter()
        .map(|source| scan_mft_source(ctx, source, targets, &names, states.len(), now))
        .collect();
    let mut failures = 0;
    for result in results {
        failures += result.unreadable;
        merge_states(states, result.states, now);
        issues.extend(result.issues);
    }
    if failures > 0 {
        issues.push(format!(
            "$MFT timeline skipped {failures} unreadable records; correlation is incomplete"
        ));
    }
}

fn scan_mft_source(
    ctx: &Context,
    source: &Path,
    targets: &HashMap<String, Vec<(usize, bool)>>,
    names: &HashSet<String>,
    state_count: usize,
    now: DateTime<Utc>,
) -> SourceScan {
    let mut scan = SourceScan::new(state_count);
    let drive = ctx.drive_of(source);
    let mut parser = match MftParser::from_path(source) {
        Ok(parser) => parser,
        Err(e) => {
            scan.issues.push(format!(
                "Cannot read timeline $MFT {}: {e}",
                source.display()
            ));
            return scan;
        }
    };
    let mut readable = false;
    for i in 0..parser.get_entry_count() {
        let entry = match parser.get_entry(i) {
            Ok(entry) if &entry.header.signature == b"FILE" => entry,
            Ok(_) => continue,
            Err(_) => {
                scan.unreadable += 1;
                continue;
            }
        };
        readable = true;
        if !entry.header.is_base_record() {
            continue;
        }
        let name_matches = entry
            .iter_attributes_matching(Some(vec![MftAttributeType::FileName]))
            .filter_map(Result::ok)
            .any(|a| {
                a.data
                    .as_file_name()
                    .is_some_and(|name| names.contains(&name.name.to_utf8_string().to_lowercase()))
            });
        if !name_matches {
            continue;
        }
        let Some(path) = parser.get_full_path_for_entry(&entry).ok().flatten() else {
            continue;
        };
        let key = match normalize_windows(&path.to_string_lossy()) {
            Ok(path) => path.to_lowercase(),
            Err(_) => continue,
        };
        let Some(indices) = targets.get(&format!("{drive}:{key}")) else {
            continue;
        };
        let extensions = parser.get_extension_entries(&entry);
        let row = FlatMftEntryWithName::from_entry_with_path(&entry, &extensions, path);
        let Some(modified) = row.standard_info_last_modified else {
            continue;
        };
        let Some(when) =
            DateTime::from_timestamp(modified.as_second(), modified.subsec_nanosecond() as u32)
        else {
            continue;
        };
        let mut detail = format!(
            "$SI Modified of {key}; MFT entry {} sequence {}{}",
            row.entry_id,
            row.sequence,
            if row.is_deleted { " (deleted)" } else { "" }
        );
        if let Some(fn_time) = row.file_name_last_modified
            && modified.as_second() + 2 < fn_time.as_second()
        {
            detail.push_str(&format!(
                "; timestomping hint: $SI predates $FN Modified {}",
                fn_time
            ));
        }
        for (index, weak) in indices {
            scan.states[*index].add(
                Candidate {
                    when,
                    source: format!("$MFT {}", source.display()),
                    detail: detail.clone(),
                    weak: *weak,
                    mft: true,
                },
                now,
            );
        }
    }
    if readable {
        for (_, indices) in targets
            .iter()
            .filter(|(path, _)| path.starts_with(&format!("{drive}:")))
        {
            for (index, _) in indices {
                scan.states[*index].available = true;
            }
        }
    }
    scan
}

struct EventPlan<'a> {
    targets: &'a HashMap<EventKey, Vec<usize>>,
    host_aliases: HostAliases,
    channels: HashSet<String>,
    channel_fields: HashMap<(String, u64), HashSet<String>>,
    channel_indices: HashMap<String, HashSet<usize>>,
}

fn scan_events(
    ctx: &Context,
    targets: &HashMap<EventKey, Vec<usize>>,
    states: &mut [State],
    now: DateTime<Utc>,
    issues: &mut Vec<String>,
) {
    // Rename evidence must be complete before channel workers use it.
    let mut plan = EventPlan {
        targets,
        host_aliases: historical_hosts(ctx, now, issues),
        channels: targets.keys().map(|key| key.channel.clone()).collect(),
        channel_fields: HashMap::new(),
        channel_indices: HashMap::new(),
    };
    for (key, indices) in targets {
        plan.channel_fields
            .entry((key.channel.clone(), key.event_id))
            .or_default()
            .insert(key.field.clone());
        plan.channel_indices
            .entry(key.channel.clone())
            .or_default()
            .extend(indices);
    }
    let sources: Vec<_> = ctx
        .files
        .iter()
        .filter(|file| {
            plan.channels
                .iter()
                .any(|channel| active_channel_log(ctx, file, channel))
        })
        .collect();
    let results: Vec<_> = sources
        .par_iter()
        .map(|source| scan_event_source(source, &plan, states.len(), now))
        .collect();
    for result in results {
        merge_states(states, result.states, now);
        issues.extend(result.issues);
    }
}

fn scan_event_source(
    source: &EvidenceFile,
    plan: &EventPlan<'_>,
    state_count: usize,
    now: DateTime<Utc>,
) -> SourceScan {
    let mut scan = SourceScan::new(state_count);
    let mut parser = match EvtxParser::from_path(&source.physical) {
        Ok(parser) => parser.with_configuration(ParserSettings::new().num_threads(1)),
        Err(e) => {
            scan.issues.push(format!(
                "Cannot read timeline EVTX {}: {e}",
                source.physical.display()
            ));
            return scan;
        }
    };
    let probable_channels: Vec<_> = plan
        .channels
        .iter()
        .filter(|channel| event_filename_matches(&source.logical, channel))
        .collect();
    let mut unreadable = 0usize;
    let mut any_record = false;
    let mut available_channels = HashSet::new();
    for record in parser.records_json_value() {
        let record = match record {
            Ok(record) => record,
            Err(_) => {
                unreadable += 1;
                continue;
            }
        };
        any_record = true;
        let event = record.data.get("Event").unwrap_or(&record.data);
        let Some(system) = event.get("System") else {
            unreadable += 1;
            continue;
        };
        let channel = scalar(system.get("Channel"))
            .unwrap_or_default()
            .to_lowercase();
        if !plan.channels.contains(&channel) {
            continue;
        }
        let computer = scalar(system.get("Computer")).unwrap_or_default();
        if !plan.host_aliases.accepts(&computer) {
            continue;
        }
        if available_channels.insert(channel.clone())
            && let Some(indices) = plan.channel_indices.get(&channel)
        {
            for index in indices {
                scan.states[*index].available = true;
            }
        }
        let Some(id) = scalar(system.get("EventID")).and_then(|id| id.parse::<u64>().ok()) else {
            continue;
        };
        let Some(fields) = plan.channel_fields.get(&(channel.clone(), id)) else {
            continue;
        };
        let event_time = system
            .get("TimeCreated")
            .and_then(|value| value.get("#attributes"))
            .and_then(|value| value.get("SystemTime"))
            .and_then(Value::as_str)
            .or_else(|| {
                system
                    .get("TimeCreated_attributes")
                    .and_then(|v| v.get("SystemTime"))
                    .and_then(Value::as_str)
            });
        let when = event_time
            .and_then(|text| DateTime::parse_from_rfc3339(text).ok())
            .map(|time| time.with_timezone(&Utc))
            .or_else(|| {
                DateTime::from_timestamp(
                    record.timestamp.as_second(),
                    record.timestamp.subsec_nanosecond() as u32,
                )
            });
        let Some(when) = when else { continue };
        for field in fields {
            let values = event
                .get("EventData")
                .into_iter()
                .chain(event.get("UserData"))
                .flat_map(|root| named_fields(root, field, 0));
            for value in values {
                let key = EventKey {
                    channel: channel.clone(),
                    event_id: id,
                    field: field.clone(),
                    value: normalize_match(&value),
                };
                if let Some(indices) = plan.targets.get(&key) {
                    let alias_proof = plan
                        .host_aliases
                        .proof(&computer)
                        .map(|proof| format!("; {proof}"))
                        .unwrap_or_default();
                    for index in indices {
                        scan.states[*index].add(
                            Candidate {
                                when,
                                source: format!(
                                    "event log {} RecordID {}",
                                    source.physical.display(),
                                    record.event_record_id
                                ),
                                detail: format!("{channel} EID {id}; {field}={value}{alias_proof}"),
                                weak: channel == "system"
                                    && id == 7045
                                    && field.eq_ignore_ascii_case("ImagePath"),
                                mft: false,
                            },
                            now,
                        );
                    }
                }
            }
        }
    }
    // A valid empty EVTX is collected evidence even though there are no wanted events.
    if !any_record && unreadable == 0 {
        for channel in probable_channels {
            if let Some(indices) = plan.channel_indices.get(channel) {
                for index in indices {
                    scan.states[*index].available = true;
                }
            }
        }
    }
    if unreadable > 0 {
        scan.issues.push(format!(
            "Timeline EVTX {} skipped {unreadable} unreadable records",
            source.physical.display()
        ));
    }
    scan
}

fn scalar(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Object(value) => scalar(value.get("#text")).or_else(|| scalar(value.get("value"))),
        _ => None,
    }
}

fn named_fields(value: &Value, field: &str, depth: usize) -> Vec<String> {
    if depth > 16 {
        return Vec::new();
    }
    let mut found = Vec::new();
    match value {
        Value::Object(object) => {
            for (name, child) in object {
                if name.eq_ignore_ascii_case(field)
                    && let Some(value) = scalar(Some(child))
                {
                    found.push(value);
                }
            }
            if object
                .get("#attributes")
                .and_then(|a| a.get("Name"))
                .and_then(Value::as_str)
                .is_some_and(|name| name.eq_ignore_ascii_case(field))
                && let Some(value) = scalar(object.get("#text"))
            {
                found.push(value);
            }
            for (name, child) in object {
                if name != "#attributes" {
                    found.extend(named_fields(child, field, depth + 1));
                }
            }
        }
        Value::Array(array) => {
            for item in array {
                found.extend(named_fields(item, field, depth + 1));
            }
        }
        _ => {}
    }
    found
}

fn normalize_match(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('\\')
        .replace('/', "\\")
        .to_lowercase()
}

fn event_filename_matches(path: &str, channel: &str) -> bool {
    let lower = path.rsplit('/').next().unwrap_or(path).to_lowercase();
    let filename = lower.trim_end_matches(".evtx");
    let expected = channel.to_lowercase().replace('/', "%4");
    filename == expected || filename == expected.replace('%', "%25")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_rename(old: &str, new: &str, computer: &str) -> Value {
        serde_json::json!({
            "System": {"Provider": {"#attributes": {"Name": "EventLog"}}, "EventID": 6011, "Channel": "System", "Computer": computer},
            "EventData": {"Data": format!("<string>{old}</string>\n<string>{new}</string>\n")}
        })
    }

    #[test]
    fn verified_rename_chain_accepts_historical_names_and_preserves_provenance() {
        let now = Utc::now();
        let first = rename_event(
            &synthetic_rename("INITIAL", "MIDDLE", "INITIAL"),
            now,
            now,
            "captured System.evtx".into(),
            11,
        )
        .unwrap();
        let second = rename_event(
            &synthetic_rename("MIDDLE", "CURRENT", "CURRENT"),
            now,
            now,
            "captured System.evtx".into(),
            12,
        )
        .unwrap();
        let unrelated = rename_event(
            &synthetic_rename("FOREIGN", "OTHER", "OTHER"),
            now,
            now,
            "captured System.evtx".into(),
            13,
        )
        .unwrap();
        let aliases = HostAliases::new("CURRENT.example.test", vec![first, second, unrelated]);
        assert!(aliases.accepts("current.EXAMPLE.TEST"));
        assert!(aliases.accepts("INITIAL.example.test"));
        assert!(aliases.accepts("MIDDLE"));
        assert!(!aliases.accepts("FOREIGN"));
        assert!(!aliases.accepts("UNPROVEN.example.test"));
        assert!(!aliases.accepts(""));
        let proof = aliases.proof("INITIAL.example.test").unwrap();
        assert!(proof.contains("original event Computer=INITIAL.example.test"));
        assert!(proof.contains("RecordID 11 (initial -> middle)"));
        assert!(proof.contains("RecordID 12 (middle -> current)"));
        assert!(aliases.proof("CURRENT").is_none());
    }

    #[test]
    fn rename_proof_requires_eventlog_provider_and_an_endpoint_computer() {
        let now = Utc::now();
        let mut event = synthetic_rename("INITIAL", "CURRENT", "FOREIGN");
        assert!(rename_event(&event, now, now, "System.evtx".into(), 1).is_none());
        event["System"]["Computer"] = Value::String("CURRENT".into());
        event["System"]["Provider"]["#attributes"]["Name"] =
            Value::String("UnrelatedProvider".into());
        assert!(rename_event(&event, now, now, "System.evtx".into(), 1).is_none());
        assert!(rename_strings("<!DOCTYPE string [<!ENTITY x SYSTEM 'file:///outside'>]><string>&x;</string><string>CURRENT</string>").is_none());
        assert!(
            rename_strings(
                "<string>INITIAL</string><string>CURRENT</string><string>FOREIGN</string>"
            )
            .is_none()
        );
        let mut native = synthetic_rename("INITIAL", "CURRENT", "CURRENT");
        native["EventData"]["Data"] = serde_json::json!({"#text": ["INITIAL", "CURRENT"]});
        assert!(rename_event(&native, now, now, "System.evtx".into(), 1).is_some());
    }

    #[test]
    fn host_aliases_and_timeline_records_use_only_live_primary_channel_paths() {
        let dir = tempfile::tempdir().unwrap();
        let paths = [
            "C/Windows/System32/winevt/Logs/System.evtx",
            "C/Windows.old/Windows/System32/winevt/Logs/System.evtx",
            "C/archive/Windows/System32/winevt/Logs/System.evtx",
            "C/Windows/System32/winevt/Logs/ForwardedEvents.evtx",
            "C/Windows/System32/winevt/Logs/Archive-System-0001.evtx",
            "D/Windows/System32/winevt/Logs/System.evtx",
        ];
        for path in paths {
            let path = dir.path().join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, b"test").unwrap();
        }
        let ctx = Context::open(dir.path(), true).unwrap();
        let sources: Vec<_> = ctx
            .files
            .iter()
            .filter(|file| active_channel_log(&ctx, file, "System"))
            .collect();
        assert_eq!(sources.len(), 1);
        assert!(sources[0].physical.ends_with(Path::new(paths[0])));
    }

    #[test]
    #[ignore = "Set JUNTEI_RENAME_EVIDENCE to a private collection containing System 6011 rename evidence"]
    fn independent_local_host_rename_evidence() {
        let path = std::path::PathBuf::from(
            std::env::var_os("JUNTEI_RENAME_EVIDENCE").expect("JUNTEI_RENAME_EVIDENCE"),
        );
        let ctx = Context::open(&path, true).unwrap();
        let mut issues = Vec::new();
        let aliases = historical_hosts(&ctx, Utc::now(), &mut issues);
        assert!(
            !aliases.parents.is_empty(),
            "No verified local rename chain: {issues:?}"
        );
    }

    #[test]
    fn candidate_precision_and_mft_tie_preference_match_upstream() {
        let now = Utc::now();
        let mut state = State::default();
        let base = Candidate {
            when: now - Duration::seconds(10),
            source: "$MFT".into(),
            detail: String::new(),
            weak: false,
            mft: true,
        };
        state.add(base.clone(), now);
        state.add(
            Candidate {
                when: base.when + Duration::seconds(1),
                mft: false,
                source: "event".into(),
                ..base.clone()
            },
            now,
        );
        assert!(state.winner().unwrap().mft);
        state.add(
            Candidate {
                when: now,
                weak: true,
                ..base.clone()
            },
            now,
        );
        assert!(!state.winner().unwrap().weak);
        state.add(
            Candidate {
                when: base.when + Duration::seconds(3),
                mft: false,
                source: "event".into(),
                ..base
            },
            now,
        );
        assert!(!state.winner().unwrap().mft);
    }
    #[test]
    fn damaged_future_timestamp_is_rejected() {
        let now = Utc::now();
        let mut state = State::default();
        state.add(
            Candidate {
                when: now + Duration::days(3),
                source: "x".into(),
                detail: "x".into(),
                weak: false,
                mft: false,
            },
            now,
        );
        assert!(state.rejected);
        assert!(state.winner().is_none());
    }
    #[test]
    fn named_data_and_nested_userdata_are_supported() {
        let event = serde_json::json!({"Data":[{"#attributes":{"Name":"TargetObject"},"#text":"HKLM\\X"}],"Operation":{"TargetObject":"HKLM\\Y"}});
        let values = named_fields(&event, "targetobject", 0);
        assert!(values.contains(&"HKLM\\X".into()));
        assert!(values.contains(&"HKLM\\Y".into()));
    }
    #[test]
    fn missing_data_never_uses_hive_key_time_or_host_copy_time() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = Context::open(dir.path(), true).unwrap();
        let mut findings = [Finding {
            path: "HKLM\\SOFTWARE\\Example\\Value".into(),
            key_last_write: "2026-10-31T01:00:00Z".into(),
            ..Default::default()
        }];
        enrich(&ctx, &mut findings, None);
        assert!(findings[0].last_change.is_empty());
        assert_eq!(findings[0].change_evidence, "NO_ARTIFACT");
    }

    #[test]
    fn file_evidence_retains_its_explicit_volume() {
        let dir = tempfile::tempdir().unwrap();
        let mft = dir.path().join("uploads/ntfs/%5C%5C.%5CC%3A/$MFT");
        std::fs::create_dir_all(mft.parent().unwrap()).unwrap();
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/mft/MFT"),
            &mft,
        )
        .unwrap();
        let ctx = Context::open(dir.path(), true).unwrap();
        let mut findings = [
            Finding {
                path: "C:\\$MFT".into(),
                source_file: mft.display().to_string(),
                ..Default::default()
            },
            Finding {
                path: "D:\\$MFT".into(),
                source_file: mft.display().to_string(),
                ..Default::default()
            },
        ];
        enrich(&ctx, &mut findings, None);
        assert_eq!(
            findings[0].change_evidence, "RESOLVED",
            "{}",
            findings[0].change_source
        );
        assert!(findings[0].change_source.contains("MFT entry 0 sequence"));
        assert_eq!(findings[1].change_evidence, "NO_ARTIFACT");
        assert!(findings[1].last_change.is_empty());
    }
}
