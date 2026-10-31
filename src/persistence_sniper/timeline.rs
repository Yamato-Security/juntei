//! Native, artifact-backed change-time correlation.

use super::{
    Finding, TimeEvidence,
    context::{Context, explicit_drive, normalize_windows},
};
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use evtx::{EvtxParser, ParserSettings};
use mft::{MftParser, attribute::MftAttributeType, csv::FlatMftEntryWithName};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
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

#[derive(Hash, PartialEq, Eq)]
struct EventKey {
    channel: String,
    event_id: u64,
    field: String,
    value: String,
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
    if !file_targets.is_empty() {
        scan_mft(
            ctx,
            explicit_mft,
            &file_targets,
            &mut states,
            now,
            &mut issues,
        );
    }
    if !event_targets.is_empty() {
        scan_events(ctx, &event_targets, &mut states, now, &mut issues);
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
    let mut failures = 0usize;
    for source in sources {
        let drive = ctx.drive_of(&source);
        let mut parser = match MftParser::from_path(&source) {
            Ok(parser) => parser,
            Err(e) => {
                issues.push(format!(
                    "Cannot read timeline $MFT {}: {e}",
                    source.display()
                ));
                continue;
            }
        };
        let mut readable = false;
        for i in 0..parser.get_entry_count() {
            let entry = match parser.get_entry(i) {
                Ok(entry) if &entry.header.signature == b"FILE" => entry,
                Ok(_) => continue,
                Err(_) => {
                    failures += 1;
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
                    a.data.as_file_name().is_some_and(|name| {
                        names.contains(&name.name.to_utf8_string().to_lowercase())
                    })
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
                states[*index].add(
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
                    states[*index].available = true;
                }
            }
        }
    }
    if failures > 0 {
        issues.push(format!(
            "$MFT timeline skipped {failures} unreadable records; correlation is incomplete"
        ));
    }
}

fn scan_events(
    ctx: &Context,
    targets: &HashMap<EventKey, Vec<usize>>,
    states: &mut [State],
    now: DateTime<Utc>,
    issues: &mut Vec<String>,
) {
    let channels: HashSet<_> = targets.keys().map(|key| key.channel.clone()).collect();
    let mut channel_fields: HashMap<(String, u64), HashSet<String>> = HashMap::new();
    let mut channel_indices: HashMap<String, HashSet<usize>> = HashMap::new();
    for (key, indices) in targets {
        channel_fields
            .entry((key.channel.clone(), key.event_id))
            .or_default()
            .insert(key.field.clone());
        channel_indices
            .entry(key.channel.clone())
            .or_default()
            .extend(indices);
    }
    let sources: Vec<_> = ctx
        .files
        .iter()
        .filter(|file| {
            file.logical.to_lowercase().ends_with(".evtx")
                && channels
                    .iter()
                    .any(|channel| event_filename_matches(&file.logical, channel))
        })
        .collect();
    for source in sources {
        let mut parser = match EvtxParser::from_path(&source.physical) {
            Ok(parser) => parser.with_configuration(ParserSettings::new().num_threads(1)),
            Err(e) => {
                issues.push(format!(
                    "Cannot read timeline EVTX {}: {e}",
                    source.physical.display()
                ));
                continue;
            }
        };
        let probable_channels: Vec<_> = channels
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
            if !channels.contains(&channel) {
                continue;
            }
            let computer = scalar(system.get("Computer")).unwrap_or_default();
            if !computer.is_empty()
                && !ctx.computer.is_empty()
                && !computer
                    .split('.')
                    .next()
                    .unwrap_or_default()
                    .eq_ignore_ascii_case(ctx.computer.split('.').next().unwrap_or_default())
            {
                continue;
            }
            if available_channels.insert(channel.clone())
                && let Some(indices) = channel_indices.get(&channel)
            {
                for index in indices {
                    states[*index].available = true;
                }
            }
            let Some(id) = scalar(system.get("EventID")).and_then(|id| id.parse::<u64>().ok())
            else {
                continue;
            };
            let Some(fields) = channel_fields.get(&(channel.clone(), id)) else {
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
                    if let Some(indices) = targets.get(&key) {
                        for index in indices {
                            states[*index].add(
                                Candidate {
                                    when,
                                    source: format!(
                                        "event log {} RecordID {}",
                                        source.physical.display(),
                                        record.event_record_id
                                    ),
                                    detail: format!("{channel} EID {id}; {field}={value}"),
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
                if let Some(indices) = channel_indices.get(channel) {
                    for index in indices {
                        states[*index].available = true;
                    }
                }
            }
        }
        if unreadable > 0 {
            issues.push(format!(
                "Timeline EVTX {} skipped {unreadable} unreadable records",
                source.physical.display()
            ));
        }
    }
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
