//! Native, offline payload resolution. Authenticode data is descriptive metadata only:
//! neither certificate trust nor the image's signature is verified by this module.

use super::Finding;
use super::context::{Context, Resolution, normalize_windows};
use cms::cert::CertificateChoices;
use cms::content_info::ContentInfo;
use cms::signed_data::{SignedData, SignerIdentifier};
use der::{Decode, asn1::OctetString};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, OnceLock};

const MAX_CERTIFICATE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_COMMAND_BYTES: usize = 128 * 1024;
const MAX_METADATA_CACHE: usize = 4096;
const MAX_SIGNER_TEXT_BYTES: usize = 16 * 1024;

#[derive(Clone)]
pub(super) struct Metadata {
    sha256: Result<String, String>,
    signer: String,
    signer_status: String,
    original_filename: Option<String>,
}

pub fn enrich(ctx: &Context, finding: &mut Finding) {
    if finding.resolution_unavailable || finding.resolution_status == "unresolved_com_class" {
        // A CLSID or AppID executable basename is not a proven payload path.
        finding.signer_status = "not_available".into();
        return;
    }
    let (payload, launcher) = command_target(&finding.value);
    let user = ctx.user_for_source(&finding.source_file).unwrap_or("");
    let target = expand_environment(
        ctx,
        if finding.resolve_target.is_empty() {
            &payload
        } else {
            &finding.resolve_target
        },
        user,
    );
    if finding.launcher.is_empty() {
        finding.launcher = launcher;
    }
    if finding.is_lolbin.is_none() {
        finding.is_lolbin = Some(is_lolbin(&target) || is_lolbin(&finding.launcher));
    }
    if finding.is_builtin.is_none() {
        finding.is_builtin = Some(BUILTIN.contains(&basename(&target).as_str()));
    }
    let normalized = normalize_windows(&target).unwrap_or_default();
    finding.resolved_path = normalized;
    finding.signer_status = "not_available".to_string();
    match ctx.resolve(&target) {
        Resolution::Found(file) => {
            if finding.exists.is_none() {
                finding.exists = Some(true);
            }
            finding.resolved_path = file.logical;
            let metadata = metadata(ctx, &file.physical);
            match metadata.sha256 {
                Ok(hash) => {
                    finding.sha256 = hash;
                    finding.resolution_status = "collected".to_string();
                }
                Err(e) => {
                    finding.resolution_status = format!("unreadable: {e}");
                }
            }
            finding.signer = metadata.signer;
            finding.signer_status = metadata.signer_status;
        }
        Resolution::NotCollected => {
            // An incomplete triage collection cannot prove that a file was absent on the host.
            finding.resolution_status = "not_collected".to_string();
        }
        Resolution::Invalid(reason) => finding.resolution_status = format!("unresolved: {reason}"),
        Resolution::Ambiguous(paths) => {
            finding.resolution_status = format!("ambiguous: {} collected copies", paths.len())
        }
    }
    if finding.is_in_os_directory.is_none() {
        let lower = finding.resolved_path.to_ascii_lowercase();
        finding.is_in_os_directory =
            Some(lower.starts_with("windows/system32/") || lower.starts_with("windows/syswow64/"));
    }
}

fn sha256(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = [0u8; 65536];
    let mut hasher = Sha256::new();
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[derive(Clone, Debug)]
pub(super) struct PeMetadata {
    pub signer: String,
    pub signer_status: String,
    pub original_filename: Option<String>,
}

pub(super) fn pe_metadata(ctx: &Context, path: &Path) -> PeMetadata {
    let metadata = metadata(ctx, path);
    PeMetadata {
        signer: metadata.signer,
        signer_status: metadata.signer_status,
        original_filename: metadata.original_filename,
    }
}

fn metadata(ctx: &Context, path: &Path) -> Metadata {
    let stat = std::fs::metadata(path).ok();
    let cache_key = format!(
        "{}\0{}\0{:?}",
        path.display(),
        stat.as_ref().map_or(0, |m| m.len()),
        stat.and_then(|m| m.modified().ok())
    );
    let slot = ctx.metadata_cache.lock().ok().and_then(|mut cache| {
        if let Some(slot) = cache.get(&cache_key) {
            return Some(Arc::clone(slot));
        }
        if cache.len() >= MAX_METADATA_CACHE {
            return None;
        }
        let slot = Arc::new(OnceLock::new());
        cache.insert(cache_key, Arc::clone(&slot));
        Some(slot)
    });
    // Checks and metadata workers can refer to the same payload concurrently.
    // Initialize each cache entry once, without holding the cache map lock
    // during file I/O, hashing, or certificate parsing.
    match slot {
        Some(slot) => slot.get_or_init(|| read_metadata(ctx, path)).clone(),
        None => read_metadata(ctx, path),
    }
}

fn read_metadata(ctx: &Context, path: &Path) -> Metadata {
    let (mut signer, mut signer_status) = embedded_signer(path);
    let mut original_filename = None;
    let content = File::open(path).ok().and_then(|file| {
        let len = file.metadata().ok()?.len();
        if len > 64 * 1024 * 1024 {
            return None;
        }
        let mut bytes = Vec::with_capacity(len as usize);
        file.take(64 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .ok()?;
        (bytes.len() <= 64 * 1024 * 1024).then_some(bytes)
    });
    if let Some(bytes) = content {
        if let Ok(pe) = goblin::pe::PE::parse(&bytes) {
            original_filename = version_original_filename(&pe, &bytes);
            if signer.is_empty() && signer_status.starts_with("no_embedded_signature") {
                if safe_authenticode_bounds(&pe, &bytes) {
                    let mut sha256 = Sha256::new();
                    let mut sha1 = sha1::Sha1::new();
                    for range in pe.authenticode_ranges() {
                        sha256.update(range);
                        sha1.update(range);
                    }
                    let hashes = [sha256.finalize().to_vec(), sha1.finalize().to_vec()];
                    (signer, signer_status) = catalog_signer(ctx, &hashes);
                } else {
                    signer_status = "damaged_pe_authenticode_bounds".to_string();
                }
            }
        }
    } else if signer_status.starts_with("no_embedded_signature") {
        signer_status = "pe_metadata_exceeds_64MiB_limit_or_unreadable".to_string();
    }
    Metadata {
        sha256: sha256(path),
        signer,
        signer_status,
        original_filename,
    }
}

fn safe_authenticode_bounds(pe: &goblin::pe::PE<'_>, bytes: &[u8]) -> bool {
    let Some(optional) = &pe.header.optional_header else {
        return false;
    };
    let Some(pointer) = bytes.get(60..64) else {
        return false;
    };
    let optional_at = u32::from_le_bytes(pointer.try_into().unwrap()) as usize + 24;
    let directory_at = match optional.standard_fields.magic {
        0x10b => 96,
        0x20b => 112,
        _ => return false,
    };
    let headers = optional.windows_fields.size_of_headers as usize;
    if headers < optional_at + directory_at + 40 || headers > bytes.len() {
        return false;
    }
    let certificate_size = optional
        .data_directories
        .get_certificate_table()
        .map_or(0, |table| table.size as usize);
    let Some(max_hashed) = bytes.len().checked_sub(certificate_size) else {
        return false;
    };
    let mut sum = headers;
    let mut sections: Vec<_> = pe
        .sections
        .iter()
        .filter(|section| section.size_of_raw_data != 0)
        .collect();
    sections.sort_by_key(|section| section.pointer_to_raw_data);
    let mut previous = headers;
    for section in sections {
        let start = section.pointer_to_raw_data as usize;
        let Some(end) = start.checked_add(section.size_of_raw_data as usize) else {
            return false;
        };
        if start < previous || end > bytes.len() {
            return false;
        }
        previous = end;
        let Some(total) = sum.checked_add(section.size_of_raw_data as usize) else {
            return false;
        };
        sum = total;
    }
    sum <= max_hashed
}

fn catalog_signer(ctx: &Context, hashes: &[Vec<u8>]) -> (String, String) {
    let catalogs: Vec<_> = ctx
        .files
        .iter()
        .filter(|file| {
            let lower = file.logical.to_lowercase();
            lower.starts_with("windows/system32/catroot/") && lower.ends_with(".cat")
        })
        .collect();
    if catalogs.is_empty() {
        return (
            String::new(),
            "no_embedded_signature_catalog_not_collected".to_string(),
        );
    }
    let mut damaged = 0;
    for file in catalogs {
        if file.len > MAX_CERTIFICATE_BYTES {
            damaged += 1;
            continue;
        }
        let bytes = match File::open(&file.physical).and_then(|file| {
            let mut bytes = Vec::new();
            file.take(MAX_CERTIFICATE_BYTES + 1)
                .read_to_end(&mut bytes)?;
            Ok(bytes)
        }) {
            Ok(bytes) if bytes.len() as u64 <= MAX_CERTIFICATE_BYTES => bytes,
            _ => {
                damaged += 1;
                continue;
            }
        };
        let parsed = (|| -> Result<bool, String> {
            let content =
                ContentInfo::from_der(der_document(&bytes)?).map_err(|e| e.to_string())?;
            if content.content_type.to_string() != "1.2.840.113549.1.7.2" {
                return Err("not SignedData".into());
            }
            let signed: SignedData = content.content.decode_as().map_err(|e| e.to_string())?;
            if signed.encap_content_info.econtent_type.to_string() != "1.3.6.1.4.1.311.10.1" {
                return Err("not a Microsoft catalog CTL".into());
            }
            let data = signed
                .encap_content_info
                .econtent
                .ok_or("catalog has no CTL")?;
            Ok(ctl_contains_hash(data.value(), hashes, 0))
        })();
        match parsed {
            Ok(true) => match cms_signer(&bytes) {
                Ok(signer) => return (signer, "catalog_certificate_unverified".to_string()),
                Err(_) => damaged += 1,
            },
            Ok(false) => {}
            Err(_) => damaged += 1,
        }
    }
    (
        String::new(),
        if damaged == 0 {
            "no_embedded_signature_no_catalog_match"
        } else {
            "catalog_unreadable_or_damaged"
        }
        .to_string(),
    )
}

/// Search CTL subject entries structurally, rather than grepping arbitrary certificate
/// bytes for a digest. Each subject entry starts with its OCTET STRING identifier.
fn ctl_contains_hash(data: &[u8], hashes: &[Vec<u8>], depth: usize) -> bool {
    if depth > 5 {
        return false;
    }
    let mut rest = data;
    while let Some((tag, value, tail)) = der_element(rest) {
        if tag == 0x30 {
            if let Some((0x04, hash, _)) = der_element(value)
                && hashes.iter().any(|wanted| wanted.as_slice() == hash)
            {
                return true;
            }
            if ctl_contains_hash(value, hashes, depth + 1) {
                return true;
            }
        }
        rest = tail;
    }
    false
}

fn der_element(data: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let tag = *data.first()?;
    let length = *data.get(1)?;
    let (length, start) = if length & 0x80 == 0 {
        (length as usize, 2)
    } else {
        let count = (length & 0x7f) as usize;
        if count == 0 || count > std::mem::size_of::<usize>() {
            return None;
        }
        let mut length = 0usize;
        for byte in data.get(2..2 + count)? {
            length = length.checked_mul(256)?.checked_add(*byte as usize)?;
        }
        (length, 2 + count)
    };
    let end = start.checked_add(length)?;
    Some((tag, data.get(start..end)?, data.get(end..)?))
}

fn der_document(bytes: &[u8]) -> Result<&[u8], String> {
    let (_, _, padding) = der_element(bytes).ok_or("damaged_pkcs7")?;
    // WIN_CERTIFICATE commonly counts its zero alignment bytes in dwLength.
    // Preserve strict DER inside the envelope, allowing only its <=7 outer pad bytes.
    if padding.len() > 7 || padding.iter().any(|byte| *byte != 0) {
        return Err("damaged_pkcs7_trailing_data".into());
    }
    Ok(&bytes[..bytes.len() - padding.len()])
}

fn rva_offset(pe: &goblin::pe::PE<'_>, rva: u32, size: u32, bytes_len: usize) -> Option<usize> {
    for section in &pe.sections {
        let Some(delta) = rva.checked_sub(section.virtual_address) else {
            continue;
        };
        if delta.checked_add(size)? <= section.size_of_raw_data {
            let offset = section.pointer_to_raw_data.checked_add(delta)? as usize;
            if offset.checked_add(size as usize)? <= bytes_len {
                return Some(offset);
            }
        }
    }
    None
}

// Bound total work as well as tree depth: resource directories/data may alias each other.
const MAX_RESOURCE_ENTRIES: usize = 16_384;
const MAX_VERSION_BYTES: usize = 16 * 1024 * 1024;

#[derive(Default)]
struct ResourceWalk {
    directories: std::collections::HashSet<usize>,
    data_entries: std::collections::HashSet<usize>,
    blobs: std::collections::HashSet<(usize, usize)>,
    entries: usize,
    bytes: usize,
}
impl ResourceWalk {
    fn entries(&mut self, data: &[u8], offset: usize) -> Vec<(u32, u32)> {
        if !self.directories.insert(offset) || self.entries >= MAX_RESOURCE_ENTRIES {
            return Vec::new();
        }
        let Some(header) = data.get(offset..offset.saturating_add(16)) else {
            return Vec::new();
        };
        let named = u16::from_le_bytes(header[12..14].try_into().unwrap()) as usize;
        let ids = u16::from_le_bytes(header[14..16].try_into().unwrap()) as usize;
        let count = (named + ids).min(MAX_RESOURCE_ENTRIES - self.entries);
        let Some(entries) = data.get(offset + 16..(offset + 16).saturating_add(count * 8)) else {
            return Vec::new();
        };
        self.entries += count;
        entries
            .as_chunks::<8>()
            .0
            .iter()
            .map(|entry| {
                (
                    u32::from_le_bytes(entry[..4].try_into().unwrap()),
                    u32::from_le_bytes(entry[4..8].try_into().unwrap()),
                )
            })
            .collect()
    }
}

fn version_original_filename(pe: &goblin::pe::PE<'_>, bytes: &[u8]) -> Option<String> {
    let optional = pe.header.optional_header.as_ref()?;
    let resource = optional.data_directories.get_resource_table()?;
    if resource.size > 16 * 1024 * 1024 {
        return None;
    }
    let root = rva_offset(pe, resource.virtual_address, resource.size, bytes.len())?;
    let data = bytes.get(root..root + resource.size as usize)?;
    version_resource(pe, bytes, data, &mut ResourceWalk::default())
}

fn version_resource(
    pe: &goblin::pe::PE<'_>,
    bytes: &[u8],
    data: &[u8],
    walk: &mut ResourceWalk,
) -> Option<String> {
    for (_, type_entry) in walk
        .entries(data, 0)
        .into_iter()
        .filter(|(id, _)| *id == 16)
    {
        if type_entry & 0x8000_0000 == 0 {
            continue;
        }
        for (_, name_entry) in walk.entries(data, (type_entry & 0x7fff_ffff) as usize) {
            if name_entry & 0x8000_0000 == 0 {
                continue;
            }
            for (_, language) in walk.entries(data, (name_entry & 0x7fff_ffff) as usize) {
                if language & 0x8000_0000 != 0 || !walk.data_entries.insert(language as usize) {
                    continue;
                }
                let Some(entry) =
                    data.get(language as usize..(language as usize).saturating_add(16))
                else {
                    continue;
                };
                let rva = u32::from_le_bytes(entry[..4].try_into().ok()?);
                let size = u32::from_le_bytes(entry[4..8].try_into().ok()?);
                if size > 4 * 1024 * 1024 {
                    continue;
                }
                let Some(offset) = rva_offset(pe, rva, size, bytes.len()) else {
                    continue;
                };
                if !walk.blobs.insert((offset, size as usize)) {
                    continue;
                }
                walk.bytes = walk.bytes.saturating_add(size as usize);
                if walk.bytes > MAX_VERSION_BYTES {
                    return None;
                }
                if let Some(name) = version_string(bytes.get(offset..offset + size as usize)?, 0) {
                    return Some(name);
                }
            }
        }
    }
    None
}

fn version_string(data: &[u8], depth: usize) -> Option<String> {
    if depth > 8 || data.len() < 6 {
        return None;
    }
    let length = u16::from_le_bytes(data[..2].try_into().ok()?) as usize;
    let value_length = u16::from_le_bytes(data[2..4].try_into().ok()?) as usize;
    let kind = u16::from_le_bytes(data[4..6].try_into().ok()?);
    let block = data.get(..length)?;
    let mut key = Vec::new();
    let mut cursor = 6;
    loop {
        let word = u16::from_le_bytes(block.get(cursor..cursor + 2)?.try_into().ok()?);
        cursor += 2;
        if word == 0 {
            break;
        }
        key.push(word);
    }
    let key = String::from_utf16(&key).ok()?;
    cursor = (cursor + 3) & !3;
    let size = value_length.checked_mul(if kind == 1 { 2 } else { 1 })?;
    if key == "OriginalFilename" && kind == 1 {
        let value = block.get(cursor..cursor.checked_add(size)?)?;
        let words: Vec<_> = value
            .as_chunks::<2>()
            .0
            .iter()
            .map(|w| u16::from_le_bytes([w[0], w[1]]))
            .take_while(|word| *word != 0)
            .collect();
        return String::from_utf16(&words)
            .ok()
            .filter(|s| !s.is_empty() && s.len() <= 4096);
    }
    cursor = (cursor.checked_add(size)? + 3) & !3;
    while cursor + 6 <= block.len() {
        let length = u16::from_le_bytes(block[cursor..cursor + 2].try_into().ok()?) as usize;
        if length < 6 || cursor.checked_add(length)? > block.len() {
            break;
        }
        if let Some(value) = version_string(&block[cursor..cursor + length], depth + 1) {
            return Some(value);
        }
        cursor = (cursor + length + 3) & !3;
    }
    None
}

fn basename(path: &str) -> String {
    path.trim_matches(['\'', '"'])
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or_default()
        .to_lowercase()
}

/// Windows quoting, rather than POSIX backslash escaping: a path such as C:\temp
/// must retain its backslashes. An unmatched quote leaves a token available for reporting.
fn tokens(command: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut token = String::new();
    let mut quoted = false;
    let mut slash_run = 0;
    for ch in command.chars() {
        if ch == '\\' {
            slash_run += 1;
            continue;
        }
        if ch == '"' {
            token.extend(std::iter::repeat_n('\\', slash_run / 2));
            if slash_run % 2 == 1 {
                token.push('"');
            } else {
                quoted = !quoted;
            }
            slash_run = 0;
        } else {
            token.extend(std::iter::repeat_n('\\', slash_run));
            slash_run = 0;
            if ch.is_whitespace() && !quoted {
                if !token.is_empty() {
                    out.push(std::mem::take(&mut token));
                }
            } else {
                token.push(ch);
            }
        }
    }
    token.extend(std::iter::repeat_n('\\', slash_run));
    if !token.is_empty() {
        out.push(token);
    }
    out
}

/// Return the payload and its proxy launcher, preserving a proxy with a placeholder-only
/// registered command instead of treating `%1` or an encoded command as a filename.
pub(super) fn command_target(command: &str) -> (String, String) {
    command_target_inner(command, 0)
}

fn command_target_inner(command: &str, depth: usize) -> (String, String) {
    if command.len() > MAX_COMMAND_BYTES {
        return (String::new(), String::new());
    }
    let parts = tokens(command);
    let Some(first) = parts.first() else {
        return (String::new(), String::new());
    };
    let name = basename(first);
    let bare = name.strip_suffix(".exe").unwrap_or(&name);
    let mut selected = None;
    match bare {
        "rundll32" => {
            selected = parts
                .get(1)
                .map(|v| v.split(',').next().unwrap_or(v).to_string())
        }
        "cmd" => {
            if let Some(at) = parts
                .iter()
                .position(|v| v.eq_ignore_ascii_case("/c") || v.eq_ignore_ascii_case("/k"))
                && at + 1 < parts.len()
                && depth < 16
            {
                let nested = parts[at + 1..].join(" ");
                selected = Some(command_target_inner(&nested, depth + 1).0);
            }
        }
        "powershell" | "pwsh" => {
            let mut at = 1;
            while at < parts.len() {
                let lower = parts[at].to_lowercase();
                if ["-encodedcommand", "-enc", "-e", "-ec"].contains(&lower.as_str()) {
                    break;
                }
                if ["-file", "-f"].contains(&lower.as_str()) {
                    selected = parts.get(at + 1).cloned();
                    break;
                }
                if ["-command", "-c"].contains(&lower.as_str()) {
                    // Inline script is not a PE path. Only a path-looking single token can
                    // be resolved; the launcher still carries the LOLBin classification.
                    selected = parts.get(at + 1).filter(|v| path_like(v)).cloned();
                    break;
                }
                if !lower.starts_with('-') {
                    selected = Some(parts[at].clone());
                    break;
                }
                let name = lower.trim_start_matches('-');
                let takes_value = [
                    "executionpolicy",
                    "windowstyle",
                    "inputformat",
                    "outputformat",
                    "version",
                    "configurationname",
                    "psconsolefile",
                ]
                .iter()
                .any(|v| v.starts_with(name))
                    || ["ep", "of"].contains(&name);
                at += if takes_value { 2 } else { 1 };
            }
        }
        "wscript" | "cscript" | "mshta" => {
            selected = parts
                .iter()
                .skip(1)
                .find(|v| !v.starts_with("//") && !placeholder(v))
                .cloned()
        }
        _ => {}
    }
    if let Some(target) = selected.filter(|v| !placeholder(v) && !v.is_empty()) {
        return (target, name);
    }
    if matches!(
        bare,
        "powershell" | "pwsh" | "cmd" | "wscript" | "cscript" | "mshta" | "rundll32"
    ) && parts.len() > 1
    {
        return (first.to_string(), name);
    }
    // Unquoted service paths can contain spaces. This is a candidate configured image,
    // not a claim about CreateProcess's actual choice if C:\Program.exe exists.
    if !command.trim_start().starts_with('"') && first.contains(['\\', '/']) {
        let mut combined = String::new();
        for part in &parts {
            if !combined.is_empty() {
                combined.push(' ');
            }
            combined.push_str(part);
            if executable_extension(&combined) {
                return (combined, String::new());
            }
        }
    }
    (first.to_string(), String::new())
}

fn executable_extension(value: &str) -> bool {
    let lower = value.to_lowercase();
    [
        ".exe", ".dll", ".com", ".bat", ".cmd", ".ps1", ".vbs", ".js", ".scr", ".msi",
    ]
    .iter()
    .any(|ext| lower.ends_with(ext))
}
fn placeholder(value: &str) -> bool {
    value.len() == 2 && value.starts_with('%') && value.as_bytes()[1].is_ascii_alphanumeric()
        || value == "%*"
}
fn path_like(value: &str) -> bool {
    !value.contains([';', '{', '}', '\n']) && executable_extension(value)
}

pub(super) fn expand_environment(ctx: &Context, value: &str, user: &str) -> String {
    let mut variables = std::collections::HashMap::from([
        ("systemroot".to_string(), "Windows".to_string()),
        ("windir".into(), "Windows".into()),
        ("systemdrive".into(), "C:".into()),
        ("homedrive".into(), "C:".into()),
        ("programfiles".into(), "Program Files".into()),
        ("programfiles(x86)".into(), "Program Files (x86)".into()),
        (
            "commonprogramfiles".into(),
            "Program Files\\Common Files".into(),
        ),
        (
            "commonprogramfiles(x86)".into(),
            "Program Files (x86)\\Common Files".into(),
        ),
        ("programdata".into(), "ProgramData".into()),
        ("allusersprofile".into(), "ProgramData".into()),
        ("public".into(), "Users\\Public".into()),
    ]);
    let mut system_drive = ctx.system_drive();
    if let Some(software) = ctx.machine_hive("SOFTWARE") {
        if let Some(key) = software
            .opened
            .hive
            .key(r"Microsoft\Windows NT\CurrentVersion")
            && let Some(value) = key.value("SystemRoot")
            && let registry::Data::Text { value, .. } =
                registry::decode(value.data_type(), &value.read_data().0)
        {
            if let Some(drive) = super::context::explicit_drive(&value) {
                system_drive = drive;
            }
            variables.insert("systemroot".into(), value.clone());
            variables.insert("windir".into(), value);
        }
        if let Some(key) = software
            .opened
            .hive
            .key(r"Microsoft\Windows\CurrentVersion")
        {
            for (field, name) in [
                ("ProgramFilesDir", "programfiles"),
                ("ProgramFilesDir (x86)", "programfiles(x86)"),
                ("CommonFilesDir", "commonprogramfiles"),
                ("CommonFilesDir (x86)", "commonprogramfiles(x86)"),
            ] {
                if let Some(value) = key.value(field)
                    && let registry::Data::Text { value, .. } =
                        registry::decode(value.data_type(), &value.read_data().0)
                {
                    variables.insert(name.into(), value);
                }
            }
        }
    }
    variables.insert("systemdrive".into(), format!("{system_drive}:"));
    variables.insert("homedrive".into(), format!("{system_drive}:"));
    if variables
        .get("systemroot")
        .is_some_and(|root| !root.contains(':'))
    {
        variables.insert("systemroot".into(), format!("{system_drive}:\\Windows"));
        variables.insert("windir".into(), format!("{system_drive}:\\Windows"));
    }
    if let Some(system) = ctx.machine_hive("SYSTEM") {
        let key = format!(
            "{}\\Control\\Session Manager\\Environment",
            ctx.active_controlset()
        );
        if let Some(key) = system.opened.hive.key(&key) {
            for value in key.values().0 {
                if let registry::Data::Text { value: text, .. } =
                    registry::decode(value.data_type(), &value.read_data().0)
                {
                    variables.insert(value.name().to_lowercase(), text);
                }
            }
        }
    }
    if !user.is_empty() {
        let profile = ctx
            .machine_hive("SOFTWARE")
            .and_then(|h| {
                registry::software::profile_list(&h.opened.hive)
                    .into_iter()
                    .find(|(_, path)| {
                        path.rsplit(['\\', '/'])
                            .next()
                            .is_some_and(|name| name.eq_ignore_ascii_case(user))
                    })
                    .map(|(_, path)| path)
            })
            .or_else(|| ctx.captured_user_profile(user))
            .unwrap_or_else(|| format!("{system_drive}:\\Users\\{user}"));
        for (name, tail) in [
            ("userprofile", ""),
            ("appdata", "\\AppData\\Roaming"),
            ("localappdata", "\\AppData\\Local"),
            ("temp", "\\AppData\\Local\\Temp"),
            ("tmp", "\\AppData\\Local\\Temp"),
        ] {
            variables.insert(name.into(), format!("{profile}{tail}"));
        }
        let profile_drive = super::context::split_drive(&profile);
        let homepath = profile_drive.map_or(profile.as_str(), |(_, tail)| tail);
        variables.insert("homepath".into(), homepath.to_string());
        if let Some((drive, _)) = profile_drive {
            variables.insert("homedrive".into(), format!("{drive}:"));
        }
        if let Some(hive) = ctx
            .hives
            .iter()
            .find(|h| h.kind == "NTUSER.DAT" && h.user.eq_ignore_ascii_case(user))
        {
            for key in ["Environment", "Volatile Environment"]
                .iter()
                .filter_map(|key| hive.opened.hive.key(key))
            {
                for value in key.values().0 {
                    if let registry::Data::Text { value: text, .. } =
                        registry::decode(value.data_type(), &value.read_data().0)
                    {
                        variables.insert(value.name().to_lowercase(), text);
                    }
                }
            }
        }
    }
    // One left-to-right pass per variable, with an explicit recursion chain. A profile
    // literally named %appdata% must stay literal instead of recursively expanding itself.
    // The shared budget bounds both emitted bytes and substitutions, including empty ones.
    let mut budget = MAX_COMMAND_BYTES;
    expand_tokens(value, &variables, &mut Vec::new(), &mut budget)
        .unwrap_or_else(|| value.to_string())
}

fn expand_tokens(
    value: &str,
    variables: &std::collections::HashMap<String, String>,
    chain: &mut Vec<String>,
    budget: &mut usize,
) -> Option<String> {
    let mut result = String::new();
    let mut rest = value;
    while let Some(start) = rest.find('%') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('%') else {
            break;
        };
        let prefix = &rest[..start];
        *budget = budget.checked_sub(prefix.len())?;
        result.push_str(prefix);
        let token = &rest[start..start + end + 2];
        // Charge before recursion: repeated references ending in empty values can
        // otherwise perform exponential work without emitting a single byte.
        *budget = budget.checked_sub(token.len())?;
        let name = after[..end].to_lowercase();
        if chain.len() < 8
            && !chain.contains(&name)
            && let Some(replacement) = variables.get(&name)
        {
            chain.push(name);
            let expanded = expand_tokens(replacement, variables, chain, budget)?;
            chain.pop();
            result.push_str(&expanded);
        } else {
            result.push_str(token);
        }
        rest = &after[end + 1..];
    }
    *budget = budget.checked_sub(rest.len())?;
    result.push_str(rest);
    Some(result)
}

fn embedded_signer(path: &Path) -> (String, String) {
    match embedded_signer_inner(path) {
        Ok(Some(name)) => (name, "embedded_certificate_unverified".to_string()),
        Ok(None) => (
            String::new(),
            "no_embedded_signature_catalog_not_checked".to_string(),
        ),
        Err(e) => (String::new(), e),
    }
}

fn embedded_signer_inner(path: &Path) -> Result<Option<String>, String> {
    let mut file = File::open(path).map_err(|e| format!("unreadable: {e}"))?;
    let file_len = file.metadata().map_err(|e| e.to_string())?.len();
    let mut dos = [0u8; 64];
    if file.read_exact(&mut dos).is_err() || &dos[..2] != b"MZ" {
        return Err("not_pe".to_string());
    }
    let pe = u32::from_le_bytes(dos[60..64].try_into().unwrap()) as u64;
    if pe.checked_add(24).is_none_or(|end| end > file_len) {
        return Err("damaged_pe_header".to_string());
    }
    file.seek(SeekFrom::Start(pe)).map_err(|e| e.to_string())?;
    let mut header = [0u8; 24];
    file.read_exact(&mut header)
        .map_err(|_| "damaged_pe_header".to_string())?;
    if &header[..4] != b"PE\0\0" {
        return Err("not_pe".to_string());
    }
    let length = u16::from_le_bytes(header[20..22].try_into().unwrap()) as usize;
    if length < 2 || pe + 24 + length as u64 > file_len {
        return Err("damaged_pe_optional_header".to_string());
    }
    let mut optional = vec![0u8; length];
    file.read_exact(&mut optional)
        .map_err(|_| "damaged_pe_optional_header".to_string())?;
    let magic = u16::from_le_bytes(optional[..2].try_into().unwrap());
    let (count_at, directories) = match magic {
        0x10b => (92, 96),
        0x20b => (108, 112),
        _ => return Err("unsupported_pe_format".to_string()),
    };
    if optional.len() < count_at + 4 {
        return Err("damaged_pe_optional_header".to_string());
    }
    let count = u32::from_le_bytes(optional[count_at..count_at + 4].try_into().unwrap());
    if count <= 4 {
        return Ok(None);
    }
    if optional.len() < directories + 40 {
        return Err("damaged_pe_certificate_directory".to_string());
    }
    let at = directories + 32;
    let offset = u32::from_le_bytes(optional[at..at + 4].try_into().unwrap()) as u64;
    let size = u32::from_le_bytes(optional[at + 4..at + 8].try_into().unwrap()) as u64;
    if offset == 0 && size == 0 {
        return Ok(None);
    }
    if offset == 0 || size < 8 || offset.checked_add(size).is_none_or(|end| end > file_len) {
        return Err("damaged_pe_certificate_bounds".to_string());
    }
    if size > MAX_CERTIFICATE_BYTES {
        return Err("certificate_table_exceeds_16MiB_limit".to_string());
    }
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| e.to_string())?;
    let mut table = vec![0u8; size as usize];
    file.read_exact(&mut table)
        .map_err(|_| "damaged_pe_certificate_table".to_string())?;
    let mut at = 0;
    let mut names = Vec::new();
    for _ in 0..128 {
        if at == table.len() {
            break;
        }
        let entry = table
            .get(at..at + 8)
            .ok_or("damaged_win_certificate_header")?;
        let length = u32::from_le_bytes(entry[..4].try_into().unwrap()) as usize;
        if length < 8 || length > table.len() - at {
            return Err("damaged_win_certificate_length".to_string());
        }
        let revision = u16::from_le_bytes(entry[4..6].try_into().unwrap());
        let kind = u16::from_le_bytes(entry[6..8].try_into().unwrap());
        if revision == 0x200 && kind == 2 {
            let name = cms_signer(&table[at + 8..at + length])?;
            if !name.is_empty() && !names.contains(&name) {
                names.push(name);
            }
        }
        at = at
            .checked_add((length + 7) & !7)
            .ok_or("damaged_win_certificate_length")?;
        if at > table.len() {
            if at - table.len() <= 7 {
                break;
            } else {
                return Err("damaged_win_certificate_padding".to_string());
            }
        }
    }
    if at < table.len() {
        return Err("certificate_entry_count_exceeds_128_limit".to_string());
    }
    if names.is_empty() {
        return Err("embedded_signer_not_identified".to_string());
    }
    if names.iter().map(String::len).sum::<usize>() > MAX_SIGNER_TEXT_BYTES {
        return Err("signer_metadata_exceeds_16KiB_limit".to_string());
    }
    Ok(Some(names.join(" | ")))
}

fn cms_signer(bytes: &[u8]) -> Result<String, String> {
    let content =
        ContentInfo::from_der(der_document(bytes)?).map_err(|_| "damaged_pkcs7".to_string())?;
    if content.content_type.to_string() != "1.2.840.113549.1.7.2" {
        return Err("unsupported_pkcs7_content".to_string());
    }
    let signed: SignedData = content
        .content
        .decode_as()
        .map_err(|_| "damaged_pkcs7_signed_data".to_string())?;
    let Some(certificates) = signed.certificates else {
        return Err("signer_certificate_not_embedded".to_string());
    };
    let mut names = Vec::new();
    for signer in signed.signer_infos.0.iter() {
        for choice in certificates.0.iter() {
            let CertificateChoices::Certificate(cert) = choice else {
                continue;
            };
            let tbs = &cert.tbs_certificate;
            let matches = match &signer.sid {
                SignerIdentifier::IssuerAndSerialNumber(sid) => {
                    sid.issuer == tbs.issuer && sid.serial_number == tbs.serial_number
                }
                SignerIdentifier::SubjectKeyIdentifier(sid) => {
                    tbs.extensions.as_ref().is_some_and(|extensions| {
                        extensions.iter().any(|ext| {
                            ext.extn_id.to_string() == "2.5.29.14"
                                && OctetString::from_der(ext.extn_value.as_bytes())
                                    .is_ok_and(|id| id.as_bytes() == sid.0.as_bytes())
                        })
                    })
                }
            };
            if matches {
                // Match the upstream metadata convention when SpcSpOpusInfo names the
                // program. It is just as unverified as the certificate subject and is
                // accepted only after linking this SignerInfo to its own certificate.
                let name = signer
                    .signed_attrs
                    .as_ref()
                    .and_then(|attributes| {
                        attributes.iter().find_map(|attribute| {
                            if attribute.oid.to_string() != "1.3.6.1.4.1.311.2.1.12" {
                                return None;
                            }
                            attribute
                                .values
                                .iter()
                                .find_map(|value| opus_program_name(value.value()))
                        })
                    })
                    .unwrap_or_else(|| tbs.subject.to_string());
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }
    }
    if names.is_empty() {
        return Err("signer_identifier_has_no_matching_certificate".to_string());
    }
    if names.iter().map(String::len).sum::<usize>() > MAX_SIGNER_TEXT_BYTES {
        return Err("signer_metadata_exceeds_16KiB_limit".to_string());
    }
    Ok(names.join(" | "))
}

fn opus_program_name(data: &[u8]) -> Option<String> {
    let mut rest = data;
    while let Some((tag, value, tail)) = der_element(rest) {
        if tag == 0xa0 {
            let (kind, name, _) = der_element(value)?;
            return match kind {
                0x80 if name.len() % 2 == 0 => String::from_utf16(
                    &name
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|w| u16::from_be_bytes(*w))
                        .collect::<Vec<_>>(),
                )
                .ok(),
                0x81 => std::str::from_utf8(name).ok().map(str::to_string),
                _ => None,
            }
            .map(|name| name.trim_end_matches('\0').to_string())
            .filter(|name| !name.is_empty());
        }
        rest = tail;
    }
    None
}

const BUILTIN: &[&str] = &[
    "explorer.exe",
    "svchost.exe",
    "lsass.exe",
    "csrss.exe",
    "smss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "spoolsv.exe",
    "taskhostw.exe",
    "sihost.exe",
    "ctfmon.exe",
    "conhost.exe",
    "dwm.exe",
    "fontdrvhost.exe",
    "dllhost.exe",
    "searchindexer.exe",
    "searchprotocolhost.exe",
    "searchfilterhost.exe",
    "runtimebroker.exe",
    "securityhealthservice.exe",
    "securityhealthsystray.exe",
    "sgrmbroker.exe",
    "smartscreen.exe",
];

pub(super) fn is_lolbin(path: &str) -> bool {
    let name = basename(path);
    LOLBINS.contains(&name.as_str())
        || [
            "powershell",
            "powershell.exe",
            "pwsh",
            "pwsh.exe",
            "cmd",
            "rundll32",
            "wscript",
            "cscript",
            "mshta",
        ]
        .contains(&name.as_str())
}

// Bundled LOLBAS name snapshot from Hexastrike/PyrsistenceSniper (MIT); the
// original project obtains these names from https://lolbas-project.github.io/.
const LOLBINS: &[&str] = &[
    "acccheckconsole.exe",
    "addinutil.exe",
    "adplus.exe",
    "advpack.dll",
    "agentexecutor.exe",
    "appcert.exe",
    "appinstaller.exe",
    "applauncher.exe",
    "appvlp.exe",
    "aspnet_compiler.exe",
    "at.exe",
    "atbroker.exe",
    "bash.exe",
    "bcp.exe",
    "bginfo.exe",
    "bitsadmin.exe",
    "cdb.exe",
    "certoc.exe",
    "certreq.exe",
    "certutil.exe",
    "change.exe",
    "cipher.exe",
    "cl_invocation.ps1",
    "cl_loadassembly.ps1",
    "cl_mutexverifiers.ps1",
    "cmd.exe",
    "cmdkey.exe",
    "cmdl32.exe",
    "cmstp.exe",
    "colorcpl.exe",
    "computerdefaults.exe",
    "comsvcs.dll",
    "configsecuritypolicy.exe",
    "conhost.exe",
    "control.exe",
    "coregen.exe",
    "createdump.exe",
    "csc.exe",
    "cscript.exe",
    "csi.exe",
    "customshellhost.exe",
    "datasvcutil.exe",
    "defaultpack.exe",
    "desk.cpl",
    "desktopimgdownldr.exe",
    "devicecredentialdeployment.exe",
    "devinit.exe",
    "devtoolslauncher.exe",
    "devtunnel.exe",
    "dfshim.dll",
    "dfsvc.exe",
    "diantz.exe",
    "diskshadow.exe",
    "dnscmd.exe",
    "dnx.exe",
    "dotnet.exe",
    "dsdbutil.exe",
    "dtutil.exe",
    "dump64.exe",
    "dumpminitool.exe",
    "dxcap.exe",
    "ecmangen.exe",
    "esentutl.exe",
    "eudcedit.exe",
    "eventvwr.exe",
    "excel.exe",
    "expand.exe",
    "explorer.exe",
    "extexport.exe",
    "extrac32.exe",
    "findstr.exe",
    "finger.exe",
    "fltmc.exe",
    "forfiles.exe",
    "fsi.exe",
    "fsianycpu.exe",
    "fsutil.exe",
    "ftp.exe",
    "gpscript.exe",
    "hh.exe",
    "ie4uinit.exe",
    "ieadvpack.dll",
    "iediagcmd.exe",
    "ieexec.exe",
    "ieframe.dll",
    "ilasm.exe",
    "imewdbld.exe",
    "infdefaultinstall.exe",
    "installutil.exe",
    "intellitrace.exe",
    "iscsicpl.exe",
    "jsc.exe",
    "launch-vsdevshell.ps1",
    "ldifde.exe",
    "makecab.exe",
    "manage-bde.wsf",
    "mavinject.exe",
    "mftrace.exe",
    "microsoft.nodejstools.pressanykey.exe",
    "microsoft.workflow.compiler.exe",
    "mmc.exe",
    "mpcmdrun.exe",
    "mpiexec.exe",
    "msaccess.exe",
    "msbuild.exe",
    "msconfig.exe",
    "msdeploy.exe",
    "msdt.exe",
    "msedge.exe",
    "msedge_proxy.exe",
    "msedgewebview2.exe",
    "mshta.exe",
    "mshtml.dll",
    "msiexec.exe",
    "msohtmed.exe",
    "mspub.exe",
    "msxsl.exe",
    "netsh.exe",
    "ngen.exe",
    "ntdsutil.exe",
    "ntsd.exe",
    "odbcconf.exe",
    "offlinescannershell.exe",
    "onedrivestandaloneupdater.exe",
    "openconsole.exe",
    "pcalua.exe",
    "pcwrun.exe",
    "pcwutl.dll",
    "pester.bat",
    "photoviewer.dll",
    "pixtool.exe",
    "pktmon.exe",
    "pnputil.exe",
    "powerpnt.exe",
    "presentationhost.exe",
    "print.exe",
    "printbrm.exe",
    "procdump.exe",
    "protocolhandler.exe",
    "provlaunch.exe",
    "psr.exe",
    "pubprn.vbs",
    "query.exe",
    "rasautou.exe",
    "rcsi.exe",
    "rdrleakdiag.exe",
    "reg.exe",
    "regasm.exe",
    "regedit.exe",
    "regini.exe",
    "register-cimprovider.exe",
    "regsvcs.exe",
    "regsvr32.exe",
    "remote.exe",
    "replace.exe",
    "reset.exe",
    "rpcping.exe",
    "rundll32.exe",
    "runexehelper.exe",
    "runonce.exe",
    "runscripthelper.exe",
    "sc.exe",
    "schtasks.exe",
    "scriptrunner.exe",
    "scrobj.dll",
    "setres.exe",
    "settingsynchost.exe",
    "setupapi.dll",
    "sftp.exe",
    "shdocvw.dll",
    "shell32.dll",
    "shimgvw.dll",
    "sqldumper.exe",
    "sqlps.exe",
    "sqltoolsps.exe",
    "squirrel.exe",
    "ssh.exe",
    "stordiag.exe",
    "syncappvpublishingserver.exe",
    "syncappvpublishingserver.vbs",
    "syssetup.dll",
    "tar.exe",
    "te.exe",
    "teams.exe",
    "testwindowremoteagent.exe",
    "tracker.exe",
    "ttdinject.exe",
    "tttracer.exe",
    "unregmp2.exe",
    "update.exe",
    "url.dll",
    "utilityfunctions.ps1",
    "vbc.exe",
    "verclsid.exe",
    "visio.exe",
    "visualuiaverifynative.exe",
    "vsdiagnostics.exe",
    "vshadow.exe",
    "vsiisexelauncher.exe",
    "vsjitdebugger.exe",
    "vslaunchbrowser.exe",
    "vsls-agent.exe",
    "vstest.console.exe",
    "wab.exe",
    "wbadmin.exe",
    "wbemtest.exe",
    "wfc.exe",
    "wfmformat.exe",
    "windbg.exe",
    "winfile.exe",
    "winget.exe",
    "winproj.exe",
    "winrm.vbs",
    "winword.exe",
    "wlrmdr.exe",
    "wmic.exe",
    "workfolders.exe",
    "wscript.exe",
    "wsl.exe",
    "wsreset.exe",
    "wt.exe",
    "wuauclt.exe",
    "xbootmgr.exe",
    "xbootmgrsleep.exe",
    "xsd.exe",
    "xwizard.exe",
    "zipfldr.dll",
];

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_CMS_HEX: &str = "3082039506092a864886f70d010702a082038630820382020101310d300b0609608648016503040201300b06092a864886f70d010701a082025e3082025a308201c3a00302010202143b4eece9d57600a709979e872d1125b5ebb5b14b300d06092a864886f70d01010b0500303f3129302706035504030c204a756e746569206e6174697665207061727365722074657374207369676e657231123010060355040a0c0954657374206f6e6c79301e170d3236313033313132333231345a170d3336313032383132333231345a303f3129302706035504030c204a756e746569206e6174697665207061727365722074657374207369676e657231123010060355040a0c0954657374206f6e6c7930819f300d06092a864886f70d010101050003818d0030818902818100d99cc49f242d8438e4819078bbf122b9389314d4125daf6db72c32f64d1706e5f66d06d1b83a754f08f37538d5e9f60424b444a57ce83c621194c06fcf1fef9025798ce9ea1fa75437eaacbb05216ceb185268478b0341135f3e59125ea72709cae0ec196ba42c22a52e8081b4085a433dcc93cae09fe21e3e014849ef2e0ed30203010001a3533051301d0603551d0e041604146395e1617f9e48328e69a1bd72179a649bda8338301f0603551d230418301680146395e1617f9e48328e69a1bd72179a649bda8338300f0603551d130101ff040530030101ff300d06092a864886f70d01010b050003818100b98a8aa23275f23264278d63700a8eed9b1673d82f99b50d1ec5a7986a04e8b14eec664b6d73e87f84d0560cff1541cb37461460f574ecb8b76638e7592b89ec249ec1c1e55e353fbfad4aab5277e538680ecd03f0a03cb786e8a4fc44ee7f5950d62c98177a92a3f8bd2ff774287751e014ae3af1f68ffec69baca10ab970133181fe3081fb0201013057303f3129302706035504030c204a756e746569206e6174697665207061727365722074657374207369676e657231123010060355040a0c0954657374206f6e6c7902143b4eece9d57600a709979e872d1125b5ebb5b14b300b0609608648016503040201300d06092a864886f70d010101050004818008a29fdf00cffbba75d39527527146f6ef779d21e5d79d66e509abab93fad921393c29aa5b84a552ade13706a6d2efccbda8bb11d6b0bd106e823978642fe178a6ad3ee4f78ed9fe099cab3598b050f31e1c4d890283a110db669fe5415e552fb37e1f31dcc97613fb01081a290806525b0668bb50a16e8b9659527e97f0a3c0";
    #[test]
    fn windows_paths_and_proxy_payloads_retain_backslashes() {
        assert_eq!(
            command_target(r#"rundll32.exe "C:\evil folder\evil.dll",Start"#),
            (r"C:\evil folder\evil.dll".into(), "rundll32.exe".into())
        );
        assert_eq!(
            command_target(r"C:\Program Files\Agent\agent.exe --service").0,
            r"C:\Program Files\Agent\agent.exe"
        );
        assert_eq!(
            command_target(r#"powershell.exe -ExecutionPolicy Bypass -File "C:\scripts\a.ps1""#).0,
            r"C:\scripts\a.ps1"
        );
        assert_eq!(command_target(r#"wscript.exe "%1""#).0, "wscript.exe");
        assert_eq!(
            command_target(r"powershell.exe -enc YQBiAA==").0,
            "powershell.exe"
        );
    }
    #[test]
    fn malformed_pe_offsets_are_bounded() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.exe");
        let mut dos = [0u8; 64];
        dos[..2].copy_from_slice(b"MZ");
        dos[60..64].copy_from_slice(&u32::MAX.to_le_bytes());
        std::fs::write(&path, dos).unwrap();
        assert_eq!(embedded_signer(&path).1, "damaged_pe_header");
    }
    #[test]
    fn streamed_hash_and_non_pe_status() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample");
        std::fs::write(&path, b"abc").unwrap();
        assert_eq!(
            sha256(&path).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(embedded_signer(&path).1, "not_pe");
    }
    #[test]
    fn missing_collection_is_not_proof_of_absence() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = Context::open(dir.path(), true).unwrap();
        let mut finding = Finding {
            value: r"C:\missing.exe".into(),
            ..Default::default()
        };
        enrich(&ctx, &mut finding);
        assert_eq!(finding.exists, None);
        assert_eq!(finding.resolution_status, "not_collected");
    }

    #[test]
    fn file_backed_user_commands_use_the_collected_users_drive() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join("D/Users/alice/AppData/Local/settings.json");
        let payload = dir.path().join("D/Users/alice/AppData/Roaming/agent.ps1");
        std::fs::create_dir_all(dir.path().join("D/Windows/System32")).unwrap();
        std::fs::write(dir.path().join("D/Windows/System32/marker"), b"marker").unwrap();
        std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
        std::fs::write(&settings, b"{}").unwrap();
        std::fs::create_dir_all(payload.parent().unwrap()).unwrap();
        std::fs::write(&payload, b"test").unwrap();
        let ctx = Context::open(dir.path(), true).unwrap();
        let mut finding = Finding {
            source_file: settings.display().to_string(),
            value: "powershell.exe -File %APPDATA%\\agent.ps1".into(),
            ..Default::default()
        };
        enrich(&ctx, &mut finding);
        assert_eq!(finding.resolution_status, "collected");
        assert_eq!(
            finding.resolved_path,
            "Users/alice/AppData/Roaming/agent.ps1"
        );
        assert_eq!(finding.exists, Some(true));
        assert!(matches!(
            ctx.resolve("C:\\Users\\alice\\AppData\\Roaming\\agent.ps1"),
            Resolution::NotCollected
        ));
    }

    #[cfg(unix)]
    #[test]
    fn file_backed_user_commands_accept_the_original_evidence_directory_alias() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("evidence");
        let settings = root.join("D/Users/alice/AppData/Local/settings.json");
        let payload = root.join("D/Users/alice/AppData/Roaming/agent.ps1");
        std::fs::create_dir_all(root.join("D/Windows/System32")).unwrap();
        std::fs::write(root.join("D/Windows/System32/marker"), b"marker").unwrap();
        std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
        std::fs::write(&settings, b"{}").unwrap();
        std::fs::create_dir_all(payload.parent().unwrap()).unwrap();
        std::fs::write(&payload, b"test").unwrap();
        let alias = dir.path().join("capture-alias");
        std::os::unix::fs::symlink(&root, &alias).unwrap();
        let ctx = Context::open(&alias, true).unwrap();
        let mut finding = Finding {
            source_file: alias
                .join("D/Users/alice/AppData/Local/settings.json")
                .display()
                .to_string(),
            value: "powershell.exe -File %APPDATA%\\agent.ps1".into(),
            ..Default::default()
        };
        enrich(&ctx, &mut finding);
        assert_eq!(finding.resolution_status, "collected");
        assert_eq!(
            finding.resolved_path,
            "Users/alice/AppData/Roaming/agent.ps1"
        );
        assert_eq!(finding.exists, Some(true));
    }

    #[test]
    fn environment_expansion_is_unicode_safe_bounded_and_uses_windows_homepath() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = Context::open(dir.path(), true).unwrap();
        assert_eq!(
            expand_environment(&ctx, "K%APPDATA%\\evil.dll", "Alice"),
            "KC:\\Users\\Alice\\AppData\\Roaming\\evil.dll"
        );
        assert_eq!(
            expand_environment(&ctx, r"%HOMEDRIVE%%HOMEPATH%\x.exe", "Alice"),
            r"C:\Users\Alice\x.exe"
        );
        for (token, expected) in [
            ("%TEMP%", r"C:\Users\Alice\AppData\Local\Temp"),
            ("%TMP%", r"C:\Users\Alice\AppData\Local\Temp"),
            ("%PUBLIC%", r"Users\Public"),
            ("%ALLUSERSPROFILE%", "ProgramData"),
            ("%COMMONPROGRAMFILES%", r"Program Files\Common Files"),
        ] {
            assert_eq!(expand_environment(&ctx, token, "Alice"), expected);
        }
        let recursive = expand_environment(&ctx, r"%APPDATA%\evil.dll", "%appdata%");
        assert!(recursive.len() < MAX_COMMAND_BYTES);
        assert_eq!(recursive, r"C:\Users\%appdata%\AppData\Roaming\evil.dll");
        assert_eq!(
            expand_environment(&ctx, "K%Unknown%", "Alice"),
            "K%Unknown%"
        );
    }

    #[test]
    fn empty_environment_substitutions_exhaust_the_work_budget() {
        let mut variables = std::collections::HashMap::new();
        for n in 1..8 {
            variables.insert(format!("v{n}"), format!("%v{}%", n + 1).repeat(4));
        }
        variables.insert("v8".into(), String::new());
        let mut budget = 128;
        assert!(expand_tokens("%v1%", &variables, &mut Vec::new(), &mut budget).is_none());
    }

    #[test]
    fn bounded_empty_substitutions_can_still_resolve_normal_commands() {
        let variables = std::collections::HashMap::from([
            ("empty".into(), String::new()),
            ("root".into(), r"C:\Windows".into()),
        ]);
        let mut budget = MAX_COMMAND_BYTES;
        assert_eq!(
            expand_tokens(
                r"%empty%%root%\agent.exe",
                &variables,
                &mut Vec::new(),
                &mut budget
            ),
            Some(r"C:\Windows\agent.exe".into())
        );
        assert!(budget < MAX_COMMAND_BYTES - r"C:\Windows\agent.exe".len());
    }

    #[test]
    fn os_directory_flag_uses_the_resolved_target_not_the_bare_command() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("Windows/System32")).unwrap();
        for name in ["userinit.exe", "evil.dll"] {
            std::fs::write(dir.path().join("Windows/System32").join(name), b"test").unwrap();
        }
        let ctx = Context::open(dir.path(), true).unwrap();
        for value in ["userinit.exe", "rundll32.exe evil.dll,Start"] {
            let mut finding = Finding {
                value: value.into(),
                ..Default::default()
            };
            enrich(&ctx, &mut finding);
            assert_eq!(finding.resolution_status, "collected");
            assert_eq!(finding.is_in_os_directory, Some(true), "{value}");
        }
    }

    #[test]
    fn aliased_version_resources_are_visited_once_with_a_total_work_budget() {
        let count = 1024usize;
        let dir_bytes = 16 + count * 8;
        let data_entry = 3 * dir_bytes;
        let mut resource = vec![0; data_entry + 32];
        for level in 0..3 {
            let base = level * dir_bytes;
            resource[base + 14..base + 16].copy_from_slice(&(count as u16).to_le_bytes());
            for n in 0..count {
                let at = base + 16 + n * 8;
                resource[at..at + 4]
                    .copy_from_slice(&(if level == 0 { 16u32 } else { n as u32 }).to_le_bytes());
                let target = if level < 2 {
                    0x8000_0000 | ((level + 1) * dir_bytes) as u32
                } else {
                    data_entry as u32
                };
                resource[at + 4..at + 8].copy_from_slice(&target.to_le_bytes());
            }
        }
        resource[data_entry..data_entry + 4]
            .copy_from_slice(&(0x1000 + data_entry as u32 + 16).to_le_bytes());
        resource[data_entry + 4..data_entry + 8].copy_from_slice(&16u32.to_le_bytes());
        let mut bytes = vec![0u8; 512 + resource.len()];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[60..64].copy_from_slice(&64u32.to_le_bytes());
        bytes[64..68].copy_from_slice(b"PE\0\0");
        bytes[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        bytes[70..72].copy_from_slice(&1u16.to_le_bytes());
        bytes[84..86].copy_from_slice(&240u16.to_le_bytes());
        bytes[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        bytes[120..124].copy_from_slice(&4096u32.to_le_bytes());
        bytes[124..128].copy_from_slice(&512u32.to_le_bytes());
        bytes[144..148].copy_from_slice(&65536u32.to_le_bytes());
        bytes[148..152].copy_from_slice(&512u32.to_le_bytes());
        bytes[196..200].copy_from_slice(&16u32.to_le_bytes());
        bytes[216..220].copy_from_slice(&0x1000u32.to_le_bytes());
        bytes[220..224].copy_from_slice(&(resource.len() as u32).to_le_bytes());
        bytes[328..333].copy_from_slice(b".rsrc");
        bytes[336..340].copy_from_slice(&(resource.len() as u32).to_le_bytes());
        bytes[340..344].copy_from_slice(&0x1000u32.to_le_bytes());
        bytes[344..348].copy_from_slice(&(resource.len() as u32).to_le_bytes());
        bytes[348..352].copy_from_slice(&512u32.to_le_bytes());
        bytes[512..].copy_from_slice(&resource);
        let pe = goblin::pe::PE::parse(&bytes).unwrap();
        let mut walk = ResourceWalk::default();
        assert!(version_resource(&pe, &bytes, &resource, &mut walk).is_none());
        assert_eq!(walk.entries, 3 * count);
        assert_eq!(walk.directories.len(), 3);
        assert_eq!(walk.data_entries.len(), 1);
        assert!(version_original_filename(&pe, &bytes).is_none());
        let mut walk = ResourceWalk {
            entries: MAX_RESOURCE_ENTRIES - 1,
            ..Default::default()
        };
        assert_eq!(walk.entries(&resource, 0).len(), 1);
        assert!(walk.entries(&resource, dir_bytes).is_empty());
    }

    #[test]
    fn nested_command_proxies_have_a_depth_limit() {
        let command = format!("{}C:\\payload.exe", "cmd.exe /c ".repeat(2000));
        let (payload, launcher) = command_target(&command);
        assert_eq!(payload, "cmd.exe");
        assert_eq!(launcher, "cmd.exe");
    }
    #[test]
    fn embedded_certificate_subject_is_reported_as_unverified() {
        // Self-signed public test certificate and CMS envelope; no private key is shipped.
        // The envelope deliberately signs unrelated data: extraction must never claim trust.
        let encoded = TEST_CMS_HEX;
        let cms: Vec<u8> = encoded
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();
        assert!(
            cms_signer(&cms)
                .unwrap()
                .contains("Juntei native parser test signer")
        );
        let mut padded = cms.clone();
        padded.extend([0, 0, 0]);
        assert!(
            cms_signer(&padded)
                .unwrap()
                .contains("Juntei native parser test signer")
        );
        padded.push(1);
        assert!(cms_signer(&padded).is_err());
        let mut pe = vec![0u8; 512];
        pe[..2].copy_from_slice(b"MZ");
        pe[60..64].copy_from_slice(&64u32.to_le_bytes());
        pe[64..68].copy_from_slice(b"PE\0\0");
        pe[84..86].copy_from_slice(&224u16.to_le_bytes());
        pe[88..90].copy_from_slice(&0x10bu16.to_le_bytes());
        pe[180..184].copy_from_slice(&16u32.to_le_bytes());
        pe[216..220].copy_from_slice(&512u32.to_le_bytes());
        let certificate_len = cms.len() + 8;
        let aligned_len = (certificate_len + 7) & !7;
        pe[220..224].copy_from_slice(&(aligned_len as u32).to_le_bytes());
        pe.extend_from_slice(&(certificate_len as u32).to_le_bytes());
        pe.extend_from_slice(&0x200u16.to_le_bytes());
        pe.extend_from_slice(&2u16.to_le_bytes());
        pe.extend_from_slice(&cms);
        pe.resize(512 + aligned_len, 0);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("signed.exe");
        std::fs::write(&path, &pe).unwrap();
        let (name, status) = embedded_signer(&path);
        assert!(
            name.contains("Juntei native parser test signer"),
            "{status}: {name}"
        );
        assert_eq!(status, "embedded_certificate_unverified");
        pe[220..224].copy_from_slice(&u32::MAX.to_le_bytes());
        std::fs::write(&path, pe).unwrap();
        assert_eq!(embedded_signer(&path).1, "damaged_pe_certificate_bounds");
    }

    #[test]
    fn catalog_requires_structural_hash_membership_and_matching_signer() {
        use der::{Any, Encode, Tag, asn1::ObjectIdentifier};
        let bytes: Vec<_> = TEST_CMS_HEX
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();
        let mut outer = ContentInfo::from_der(&bytes).unwrap();
        let mut signed: SignedData = outer.content.decode_as().unwrap();
        let hash = vec![0x42u8; 32];
        let mut ctl = vec![0x30, 38, 0x30, 36, 0x30, 34, 0x04, 32];
        ctl.extend_from_slice(&hash);
        signed.encap_content_info.econtent_type =
            ObjectIdentifier::new_unwrap("1.3.6.1.4.1.311.10.1");
        signed.encap_content_info.econtent = Some(Any::new(Tag::OctetString, ctl).unwrap());
        outer.content = Any::from_der(&signed.to_der().unwrap()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Windows/System32/CatRoot/example/test.cat");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, outer.to_der().unwrap()).unwrap();
        let ctx = Context::open(dir.path(), true).unwrap();
        let (signer, status) = catalog_signer(&ctx, &[hash]);
        assert!(
            signer.contains("Juntei native parser test signer"),
            "{status}: {signer}"
        );
        assert_eq!(status, "catalog_certificate_unverified");
        assert_eq!(
            catalog_signer(&ctx, &[vec![0x43; 32]]).1,
            "no_embedded_signature_no_catalog_match"
        );
    }

    #[test]
    fn version_string_is_structured_and_length_checked() {
        let key: Vec<u8> = "OriginalFilename\0"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        let value: Vec<u8> = "real.exe\0"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        let mut block = vec![0, 0, 9, 0, 1, 0];
        block.extend(key);
        while block.len() % 4 != 0 {
            block.push(0);
        }
        block.extend(value);
        let len = block.len() as u16;
        block[..2].copy_from_slice(&len.to_le_bytes());
        assert_eq!(version_string(&block, 0).unwrap(), "real.exe");
        block[2..4].copy_from_slice(&u16::MAX.to_le_bytes());
        assert!(version_string(&block, 0).is_none());
    }

    #[test]
    #[ignore = "Set JUNTEI_PE_SAMPLE to an independently collected signed PE"]
    fn independent_signed_pe_metadata() {
        let path = std::path::PathBuf::from(
            std::env::var_os("JUNTEI_PE_SAMPLE").expect("JUNTEI_PE_SAMPLE"),
        );
        let ctx = Context::open(path.parent().unwrap(), true).unwrap();
        let metadata = pe_metadata(&ctx, &path);
        println!("{metadata:?}");
        assert_eq!(metadata.signer_status, "embedded_certificate_unverified");
        assert!(!metadata.signer.is_empty());
        assert!(metadata.original_filename.is_some());
    }
}
