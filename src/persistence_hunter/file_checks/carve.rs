//! Bounded offline record recovery. Carved records may be stale/deleted: never
//! claim that a recovered WMI consumer or BITS job was active at acquisition.
use super::Check;
use crate::persistence_hunter::TimeEvidence;
use regex::bytes::Regex;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{self, Read};

const WINDOW: usize = 4 * 1024 * 1024;
const MAX_RECORDS: usize = 10_000;
const MAX_TEXT: usize = 32 * 1024 * 1024;
const CLASSES: &[&str] = &[
    "ActiveScriptEventConsumer",
    "CommandLineEventConsumer",
    "LogFileEventConsumer",
    "NTEventLogEventConsumer",
    "SMTPEventConsumer",
];

fn windows(
    path: &std::path::Path,
    overlap: usize,
    mut inspect: impl FnMut(&[u8], usize) -> bool,
) -> io::Result<()> {
    let mut handle = File::open(path)?;
    let mut window = Vec::new();
    handle
        .by_ref()
        .take(WINDOW as u64)
        .read_to_end(&mut window)?;
    while !window.is_empty() {
        let mut next = Vec::new();
        handle.by_ref().take(WINDOW as u64).read_to_end(&mut next)?;
        let cutoff = if next.is_empty() {
            window.len()
        } else {
            window.len().saturating_sub(overlap)
        };
        if !inspect(&window, cutoff) {
            return Ok(());
        }
        if next.is_empty() {
            break;
        }
        let retained = window.split_off(cutoff);
        window = retained;
        window.extend(next);
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct Heap {
    class: usize,
    values: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Binding {
    class: String,
    name: String,
    filter: String,
}

fn class_header(class: &str) -> Vec<u8> {
    let raw: Vec<_> = class
        .to_uppercase()
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    format!("{:X}", Sha256::digest(&raw))
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect()
}

fn heap_values(window: &[u8], start: usize, class: &str) -> Vec<String> {
    if !window
        .get(start..)
        .is_some_and(|data| data.starts_with(&class_header(class)))
    {
        return Vec::new();
    }
    let marker = format!("\0{class}\0").into_bytes();
    let search_end = (start + 1024).min(window.len());
    let Some(index) = window[start..search_end]
        .windows(marker.len())
        .position(|b| b == marker)
    else {
        return Vec::new();
    };
    let index = start + index;
    let mut end = (index + 64 * 1024 - 1024).min(window.len());
    if index >= 4 {
        let declared = u32::from_le_bytes(window[index - 4..index].try_into().expect("four bytes"));
        if declared & 0x8000_0000 != 0 && (declared & 0x7fff_ffff) as usize >= marker.len() {
            end = end.min(index + (declared & 0x7fff_ffff) as usize);
        }
    }
    let mut values = Vec::new();
    let mut at = index + marker.len();
    while at < end && values.len() < 6 {
        while at < end && !(0x20..=0x7e).contains(&window[at]) {
            at += 1;
        }
        let first = at;
        while at < end && (0x20..=0x7e).contains(&window[at]) {
            at += 1;
        }
        if at - first < 4 {
            continue;
        }
        let text = String::from_utf8_lossy(&window[first..at]);
        if text.starts_with("__") {
            break;
        }
        values.push(text.into_owned());
    }
    values
}

pub(super) fn wmi(check: &mut Check<'_>) {
    let binding_re=Regex::new(r#"((?:ActiveScript|CommandLine|LogFile|NTEventLog|SMTP)EventConsumer)\.Name="((?-u:[^"\x00]){1,256})""#).expect("fixed expression");
    let filter_re =
        Regex::new(r#"__EventFilter\.Name="((?-u:[^"\x00]){1,256})""#).expect("fixed expression");
    let headers: Vec<_> = CLASSES
        .iter()
        .map(|name| {
            let header = class_header(name);
            Regex::new(&regex::escape(
                std::str::from_utf8(&header).expect("ASCII UTF-16 class hash"),
            ))
            .expect("escaped class hash")
        })
        .collect();
    for file in check.files(|p| {
        matches!(
            p,
            "windows/system32/wbem/repository/objects.data"
                | "windows/system32/wbem/repository/fs/objects.data"
        )
    }) {
        check.sources += 1;
        if !check.safe_file(&file) {
            continue;
        }
        let mut bindings = BTreeSet::<Binding>::new();
        let mut heaps = Vec::<Heap>::new();
        let mut heap_seen = BTreeSet::new();
        let mut text_bytes = 0usize;
        let mut limited = false;
        let scanned = windows(&file.physical, 64 * 1024, |data, cutoff| {
            for captures in binding_re.captures_iter(data) {
                let whole = captures.get(0).expect("capture");
                if whole.start() >= cutoff {
                    break;
                }
                let filter = filter_re
                    .captures(&data[whole.end()..(whole.end() + 512).min(data.len())])
                    .map(|c| {
                        String::from_utf8_lossy(c.get(1).expect("capture").as_bytes()).into_owned()
                    })
                    .unwrap_or_default();
                bindings.insert(Binding {
                    class: String::from_utf8_lossy(captures.get(1).expect("capture").as_bytes())
                        .into_owned(),
                    name: String::from_utf8_lossy(captures.get(2).expect("capture").as_bytes())
                        .into_owned(),
                    filter,
                });
                if bindings.len() + heaps.len() >= MAX_RECORDS {
                    limited = true;
                    return false;
                }
            }
            for (class, header) in headers.iter().enumerate() {
                for matched in header.find_iter(data) {
                    let offset = matched.start();
                    if offset >= cutoff {
                        break;
                    }
                    let values = heap_values(data, offset, CLASSES[class]);
                    if !values.is_empty() && heap_seen.insert((class, values.clone())) {
                        text_bytes += values.iter().map(String::len).sum::<usize>();
                        if bindings.len() + heaps.len() >= MAX_RECORDS || text_bytes > MAX_TEXT {
                            limited = true;
                            return false;
                        }
                        heaps.push(Heap { class, values });
                    }
                }
            }
            true
        });
        if let Err(e) = scanned {
            check.issue("unreadable", format!("{}: {e}", file.physical.display()));
        }
        if limited {
            check.issue(
                "limit_reached",
                format!(
                    "{}: WMI recovery stopped at {MAX_RECORDS} records or {MAX_TEXT} text bytes",
                    file.physical.display()
                ),
            );
        }
        let mut index = BTreeMap::new();
        for (n, heap) in heaps.iter().enumerate() {
            for value in &heap.values {
                index
                    .entry((CLASSES[heap.class].to_string(), value.clone()))
                    .or_insert(n);
            }
        }
        let mut bound_heaps = BTreeSet::new();
        for binding in bindings {
            let mut value = format!("{} \"{}\"", binding.class, binding.name);
            if !binding.filter.is_empty() {
                value.push_str(&format!(" bound to filter \"{}\"", binding.filter));
            }
            let mut target = String::new();
            if let Some(n) = index.get(&(binding.class.clone(), binding.name.clone())) {
                bound_heaps.insert(*n);
                let payload = heaps[*n]
                    .values
                    .iter()
                    .filter(|v| *v != &binding.name)
                    .max_by_key(|v| v.len())
                    .cloned()
                    .unwrap_or_default();
                if !payload.is_empty() {
                    value.push_str(": ");
                    value.push_str(&payload);
                }
                if binding.class == "CommandLineEventConsumer" {
                    target = payload;
                }
            }
            let mut f = check.file_finding(&file, &value, "SYSTEM");
            if !target.is_empty() {
                let (payload, launcher) =
                    crate::persistence_hunter::resolver::command_target(&target);
                f.resolve_target = payload;
                f.launcher = launcher;
            } else {
                f.resolve_target.clear();
            }
            f.record_status = "carved".into();
            f.time_evidence = vec![TimeEvidence::FileWrite {
                path: file.logical.clone(),
                weak: true,
            }];
            f.description="Recovered WMI consumer/binding reference. The binding and payload are heuristically associated; stale/deleted repository content may survive, and carving does not establish activation or execution.".into();
            check.scan.findings.push(f);
        }
        for (n, heap) in heaps.into_iter().enumerate() {
            if bound_heaps.contains(&n) {
                continue;
            }
            let value = format!(
                "{} instance: {}",
                CLASSES[heap.class],
                heap.values.join(" | ")
            );
            let mut f = check.file_finding(&file, &value, "SYSTEM");
            f.resolve_target.clear();
            f.record_status = "carved_unbound".into();
            f.time_evidence = vec![TimeEvidence::FileWrite {
                path: file.logical.clone(),
                weak: true,
            }];
            f.description="Recovered WMI consumer instance without a matching recovered binding; presence does not prove an active persistent subscription or execution.".into();
            check.scan.findings.push(f);
        }
        check.issue("carving",format!("{}: bounded ASCII WMI record recovery; allocated/deleted state and complete repository semantics are not validated",file.physical.display()));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct BitsJob {
    guid: [u8; 16],
    display: String,
    program: String,
    parameters: String,
    owner: String,
}

fn counted_at(data: &[u8], offset: usize) -> Option<(String, usize)> {
    let n = u32::from_le_bytes(data.get(offset..offset.checked_add(4)?)?.try_into().ok()?) as usize;
    if !(1..=8192).contains(&n) {
        return None;
    }
    let end = offset.checked_add(4)?.checked_add(n.checked_mul(2)?)?;
    let bytes = data.get(offset + 4..end)?;
    if !bytes.ends_with(&[0, 0]) {
        return None;
    }
    let words: Vec<_> = bytes[..bytes.len() - 2]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u16::from_le_bytes([p[0], p[1]]))
        .collect();
    let value = String::from_utf16_lossy(&words);
    if value.contains('\0') {
        return None;
    }
    Some((value, end))
}

fn counted_before(data: &[u8], end: usize) -> Option<(String, usize)> {
    if end < 6 || data.get(end - 2..end)? != [0, 0] {
        return None;
    }
    let mut position = end - 4;
    let limit = end.saturating_sub(2 * 8192);
    while position >= limit && data.get(position..position + 2)? != [0, 0] {
        position = position.checked_sub(2)?;
    }
    if position < limit {
        return None;
    }
    let length_offset = position.checked_sub(2)?;
    let (value, actual_end) = counted_at(data, length_offset)?;
    if actual_end != end {
        return None;
    }
    Some((value, length_offset))
}

fn bits_job(data: &[u8], owner_start: usize) -> Option<BitsJob> {
    let (owner, _) = counted_at(data, owner_start.checked_sub(4)?)?;
    if !owner.starts_with("S-1-") || !owner[4..].bytes().all(|c| c.is_ascii_digit() || c == b'-') {
        return None;
    }
    let mut end = owner_start - 4;
    let mut fields = Vec::new();
    for _ in 0..4 {
        let (value, start) = counted_before(data, end)?;
        fields.push(value);
        end = start;
    }
    let guid = data.get(end.checked_sub(16)?..end)?.try_into().ok()?;
    Some(BitsJob {
        guid,
        display: fields[3].clone(),
        program: fields[1].clone(),
        parameters: fields[0].clone(),
        owner,
    })
}

fn guid(raw: &[u8; 16]) -> String {
    format!(
        "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        u32::from_le_bytes(raw[..4].try_into().expect("slice")),
        u16::from_le_bytes(raw[4..6].try_into().expect("slice")),
        u16::from_le_bytes(raw[6..8].try_into().expect("slice")),
        raw[8],
        raw[9],
        raw[10],
        raw[11],
        raw[12],
        raw[13],
        raw[14],
        raw[15]
    )
}

pub(super) fn bits(check: &mut Check<'_>) {
    let mut jobs_seen = BTreeSet::new();
    for file in check.files(|p| {
        matches!(
            p,
            "programdata/microsoft/network/downloader/qmgr.db"
                | "programdata/microsoft/network/downloader/qmgr0.dat"
                | "programdata/microsoft/network/downloader/qmgr1.dat"
        )
    }) {
        check.sources += 1;
        if !check.safe_file(&file) {
            continue;
        }
        let mut jobs = Vec::new();
        let mut limited = false;
        let anchor = b"S\0-\x001\0-\0";
        let scanned = windows(&file.physical, 256 * 1024, |data, _cutoff| {
            for (offset, value) in data.windows(anchor.len()).enumerate() {
                if value != anchor {
                    continue;
                }
                if let Some(job) = bits_job(data, offset)
                    && jobs_seen.insert((job.guid, job.program.clone(), job.parameters.clone()))
                {
                    jobs.push(job);
                    if jobs_seen.len() >= MAX_RECORDS {
                        limited = true;
                        return false;
                    }
                }
            }
            true
        });
        if let Err(e) = scanned {
            check.issue("unreadable", format!("{}: {e}", file.physical.display()));
        }
        if limited {
            check.issue(
                "limit_reached",
                format!(
                    "{}: stopped after {MAX_RECORDS} BITS records",
                    file.physical.display()
                ),
            );
        }
        for job in jobs {
            if [&job.display, &job.program, &job.parameters, &job.owner]
                .iter()
                .any(|s| s.contains('\u{fffd}'))
            {
                check.issue("lossy_text", format!("BITS job {} contains Unicode replacement characters; inspect its original counted UTF-16 source bytes", guid(&job.guid)));
            }
            let program = job.program.trim();
            if program.is_empty() {
                continue;
            }
            let parameters = job.parameters.trim();
            let command = if parameters
                .to_lowercase()
                .starts_with(&program.to_lowercase())
            {
                parameters.to_string()
            } else {
                format!("{program} {parameters}").trim().to_string()
            };
            let access = bits_owner_access(&job.owner);
            let mut f = check.file_finding(&file, &command, access);
            f.path = format!("{}\\{}", f.path, guid(&job.guid));
            f.resolve_target = program.to_string();
            f.record_status = "carved".into();
            f.description = format!(
                "Recovered BITS notification command; job {} ({}), owner {}. Carving may include stale/deleted jobs and does not establish current state or execution.",
                guid(&job.guid),
                job.display,
                job.owner
            );
            f.time_evidence = vec![TimeEvidence::FileWrite {
                path: file.logical.clone(),
                weak: true,
            }];
            check.scan.findings.push(f);
        }
        check.issue("carving",format!("{}: bounded counted-string record recovery; ESE allocation/current job state is not validated",file.physical.display()));
    }
}

fn bits_owner_access(owner: &str) -> &'static str {
    match owner {
        "S-1-5-18" => "SYSTEM",
        "S-1-5-19" => "LOCAL_SERVICE",
        "S-1-5-20" => "NETWORK_SERVICE",
        _ => "USER",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn counted(value: &str) -> Vec<u8> {
        let chars: Vec<_> = value.encode_utf16().chain([0]).collect();
        let mut out = (chars.len() as u32).to_le_bytes().to_vec();
        out.extend(chars.into_iter().flat_map(u16::to_le_bytes));
        out
    }
    #[test]
    fn bits_notification_fields_and_truncation_are_validated() {
        let mut raw = vec![1; 16];
        for field in [
            "Test",
            "Description",
            r"C:\Tools\test.exe",
            "-flag",
            "S-1-5-18",
        ] {
            raw.extend(counted(field));
        }
        let owner = raw.windows(6).position(|w| w == b"S\0-\x001\0").unwrap();
        let job = bits_job(&raw, owner).unwrap();
        assert_eq!(job.program, r"C:\Tools\test.exe");
        assert_eq!(job.parameters, "-flag");
        assert_eq!(job.owner, "S-1-5-18");
        assert!(bits_job(&raw[..raw.len() - 1], owner).is_none());
    }

    #[test]
    fn bits_bad_display_utf16_does_not_discard_its_notification_command() {
        let mut raw = vec![1; 16];
        raw.extend(3u32.to_le_bytes());
        raw.extend([0x78, 0, 0, 0xd8, 0, 0]); // x + unpaired surrogate + NUL
        for field in ["Description", r"C:\Tools\test.exe", "-flag", "S-1-5-18"] {
            raw.extend(counted(field));
        }
        let owner = raw.windows(6).position(|w| w == b"S\0-\x001\0").unwrap();
        let job = bits_job(&raw, owner).unwrap();
        assert_eq!(job.display, "x\u{fffd}");
        assert_eq!(job.program, r"C:\Tools\test.exe");
    }

    #[test]
    fn bits_owners_preserve_local_service_and_network_service_privileges() {
        assert_eq!(bits_owner_access("S-1-5-18"), "SYSTEM");
        assert_eq!(bits_owner_access("S-1-5-19"), "LOCAL_SERVICE");
        assert_eq!(bits_owner_access("S-1-5-20"), "NETWORK_SERVICE");
        assert_eq!(bits_owner_access("S-1-5-21-1-2-3-1001"), "USER");
    }
    #[test]
    fn wmi_class_schema_is_not_a_consumer_instance() {
        assert!(
            heap_values(
                b"CommandLineTemplate\0CommandLineEventConsumer\0ScriptText",
                0,
                "CommandLineEventConsumer"
            )
            .is_empty()
        );
        let mut instance = class_header("CommandLineEventConsumer");
        instance.extend([0; 32]);
        instance.extend(b"\0CommandLineEventConsumer\0cmd.exe /c test.exe\0ConsumerName\0");
        assert_eq!(
            heap_values(&instance, 0, "CommandLineEventConsumer"),
            vec!["cmd.exe /c test.exe", "ConsumerName"]
        );
    }
    #[test]
    fn windows_keep_cross_boundary_records() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("bytes");
        let mut data = vec![0; WINDOW - 3];
        data.extend(b"crossing-boundary");
        data.resize(WINDOW + 100, 0);
        std::fs::write(&file, &data).unwrap();
        let mut found = 0;
        windows(&file, 64 * 1024, |data, cutoff| {
            for (at, b) in data.windows(17).enumerate() {
                if at < cutoff && b == b"crossing-boundary" {
                    found += 1;
                }
            }
            true
        })
        .unwrap();
        assert_eq!(found, 1);
    }
}
