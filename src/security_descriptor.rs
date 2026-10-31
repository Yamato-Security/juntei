//! Offline, bounded decoding of self-relative Windows security descriptors. No account lookup
//! or Windows APIs: unknown SIDs stay unresolved, and unsupported ACEs stay explicitly raw.
//! Layouts: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/

use std::fmt::Write;

pub(crate) struct Report {
    pub text: String,
    pub problems: Vec<String>,
}

/// Produce deterministic text, including a diagnostic report for damaged descriptors.
pub(crate) fn report(bytes: &[u8]) -> Report {
    let mut report = Report {
        text: format!(
            "Windows security descriptor\nSize: {} bytes\nSHA-256: {}\n\n",
            bytes.len(),
            defender::quarantine::hashes(bytes).sha256,
        ),
        problems: Vec::new(),
    };
    if let Err(error) = decode(bytes, &mut report) {
        report.problems.push(error);
    }
    if report.problems.is_empty() {
        report.text.push_str("\nDecode status: complete\n");
    } else {
        report
            .text
            .push_str("\nDecode status: incomplete\nProblems:\n");
        for problem in &report.problems {
            let _ = writeln!(report.text, "  - {problem}");
        }
        report
            .text
            .push_str("Consult the preserved binary descriptor for undecoded data.\n");
    }
    report.text.push_str(
        "\nInterpretation: recorded ownership and permissions are not proof of who created\n\
         or executed the file. ACEs are listed in their original order; this report\n\
         does not calculate effective access. SACL entries describe configuration,\n\
         not access history or the host's complete audit policy. File/directory rights\n\
         share bit values; object ACEs retain object-specific rights as hexadecimal.\n",
    );
    report
}

fn decode(bytes: &[u8], report: &mut Report) -> Result<(), String> {
    let header = bytes
        .get(..20)
        .ok_or("descriptor header is shorter than 20 bytes")?;
    if header[0] != 1 {
        return Err(format!("unsupported descriptor revision {}", header[0]));
    }
    let control = u16_at(header, 2)?;
    if control & 0x8000 == 0 {
        return Err(
            "descriptor is not self-relative; absolute pointers cannot be decoded offline".into(),
        );
    }
    let _ = writeln!(report.text, "Revision: 1\nControl: 0x{control:04X}");
    for (flag, name) in CONTROL_FLAGS {
        if control & flag != 0 {
            let _ = writeln!(report.text, "  {name}");
        }
    }
    if control & 0x4000 != 0 {
        let _ = writeln!(report.text, "Resource manager control: 0x{:02X}", header[1]);
    }
    let mut sddl = Some(String::new());
    for (name, token, at) in [("Owner", "O:", 4), ("Primary group", "G:", 8)] {
        match optional_sid(bytes, u32_at(header, at)?) {
            Ok(Some(sid)) => {
                let _ = writeln!(report.text, "{name}: {}", sid.label());
                if let Some(sddl) = &mut sddl {
                    let _ = write!(sddl, "{token}{}", sid.value);
                }
            }
            Ok(None) => {
                let _ = writeln!(report.text, "{name}: Not recorded");
            }
            Err(error) => {
                let _ = writeln!(report.text, "{name}: Unable to decode");
                report.problems.push(format!("{name}: {error}"));
                sddl = None;
            }
        }
    }
    for (name, token, flag, at, shift) in
        [("DACL", "D:", 0x0004, 16, 0), ("SACL", "S:", 0x0010, 12, 1)]
    {
        report.text.push('\n');
        match acl(bytes, control & flag != 0, u32_at(header, at)?) {
            Ok(acl) => {
                let mut part = token.to_owned();
                for (bit, token) in [(0x1000, "P"), (0x0100, "AR"), (0x0400, "AI")] {
                    if control & (bit << shift) != 0 {
                        part.push_str(token);
                    }
                }
                let complete = render_acl(
                    &acl,
                    name,
                    &mut report.text,
                    &mut part,
                    &mut report.problems,
                );
                if matches!(acl, Acl::Absent) {
                    // An absent ACL has no SDDL component, even if unrelated control bits are set.
                } else if complete {
                    if let Some(sddl) = &mut sddl {
                        sddl.push_str(&part);
                    }
                } else {
                    sddl = None;
                }
            }
            Err(error) => {
                let _ = writeln!(report.text, "{name}: Unable to decode");
                report.problems.push(format!("{name}: {error}"));
                sddl = None;
            }
        }
    }
    match sddl {
        Some(value) => {
            let _ = writeln!(
                report.text,
                "\nSDDL (owner, group and ACL entries):\n{value}"
            );
        }
        None => report
            .text
            .push_str("\nSDDL: unavailable; damaged or unsupported data would be omitted.\n"),
    }
    Ok(())
}

const CONTROL_FLAGS: &[(u16, &str)] = &[
    (0x0001, "SE_OWNER_DEFAULTED"),
    (0x0002, "SE_GROUP_DEFAULTED"),
    (0x0004, "SE_DACL_PRESENT"),
    (0x0008, "SE_DACL_DEFAULTED"),
    (0x0010, "SE_SACL_PRESENT"),
    (0x0020, "SE_SACL_DEFAULTED"),
    (0x0040, "SE_DACL_UNTRUSTED"),
    (0x0080, "SE_SERVER_SECURITY"),
    (0x0100, "SE_DACL_AUTO_INHERIT_REQ"),
    (0x0200, "SE_SACL_AUTO_INHERIT_REQ"),
    (0x0400, "SE_DACL_AUTO_INHERITED"),
    (0x0800, "SE_SACL_AUTO_INHERITED"),
    (0x1000, "SE_DACL_PROTECTED"),
    (0x2000, "SE_SACL_PROTECTED"),
    (0x4000, "SE_RM_CONTROL_VALID"),
    (0x8000, "SE_SELF_RELATIVE"),
];

fn u16_at(bytes: &[u8], at: usize) -> Result<u16, String> {
    let data = bytes
        .get(at..at.saturating_add(2))
        .ok_or("truncated 16-bit field")?;
    Ok(u16::from_le_bytes([data[0], data[1]]))
}

fn u32_at(bytes: &[u8], at: usize) -> Result<u32, String> {
    let data = bytes
        .get(at..at.saturating_add(4))
        .ok_or("truncated 32-bit field")?;
    Ok(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
}

fn component(bytes: &[u8], offset: u32) -> Result<&[u8], String> {
    let at = usize::try_from(offset).map_err(|_| "component offset is too large")?;
    if at < 20 || at % 4 != 0 {
        return Err(format!(
            "component offset {offset} is inside the header or not DWORD-aligned"
        ));
    }
    bytes
        .get(at..)
        .ok_or_else(|| format!("component offset {offset} is outside the descriptor"))
}

struct Sid {
    value: String,
    size: usize,
}

impl Sid {
    fn label(&self) -> String {
        let name = match self.value.as_str() {
            "S-1-0-0" => "Nobody",
            "S-1-1-0" => "Everyone",
            "S-1-2-0" => "LOCAL",
            "S-1-3-0" => "CREATOR OWNER",
            "S-1-3-1" => "CREATOR GROUP",
            "S-1-3-4" => "OWNER RIGHTS",
            "S-1-5-7" => "NT AUTHORITY\\ANONYMOUS LOGON",
            "S-1-5-11" => "NT AUTHORITY\\Authenticated Users",
            "S-1-5-18" => "NT AUTHORITY\\SYSTEM",
            "S-1-5-19" => "NT AUTHORITY\\LOCAL SERVICE",
            "S-1-5-20" => "NT AUTHORITY\\NETWORK SERVICE",
            "S-1-5-32-544" => "BUILTIN\\Administrators",
            "S-1-5-32-545" => "BUILTIN\\Users",
            "S-1-5-32-546" => "BUILTIN\\Guests",
            "S-1-16-0" => "Untrusted integrity",
            "S-1-16-4096" => "Low integrity",
            "S-1-16-8192" => "Medium integrity",
            "S-1-16-8448" => "Medium Plus integrity",
            "S-1-16-12288" => "High integrity",
            "S-1-16-16384" => "System integrity",
            _ => "account name not resolved",
        };
        format!("{} ({name})", self.value)
    }
}

fn sid(bytes: &[u8]) -> Result<Sid, String> {
    let header = bytes.get(..8).ok_or("truncated SID header")?;
    if header[0] != 1 || header[1] > 15 {
        return Err("invalid SID revision or subauthority count".into());
    }
    let size = 8 + usize::from(header[1]) * 4;
    let bytes = bytes.get(..size).ok_or("truncated SID subauthorities")?;
    let authority = header[2..8]
        .iter()
        .fold(0u64, |a, &b| (a << 8) | u64::from(b));
    let mut value = if authority <= u64::from(u32::MAX) {
        format!("S-1-{authority}")
    } else {
        format!("S-1-0x{authority:012x}")
    };
    for at in (8..size).step_by(4) {
        let _ = write!(value, "-{}", u32_at(bytes, at)?);
    }
    Ok(Sid { value, size })
}

fn optional_sid(bytes: &[u8], offset: u32) -> Result<Option<Sid>, String> {
    if offset == 0 {
        return Ok(None);
    }
    sid(component(bytes, offset)?).map(Some)
}

enum Acl<'a> {
    Absent,
    Null,
    Entries(Vec<Ace<'a>>),
}
struct Ace<'a> {
    kind: u8,
    flags: u8,
    raw: &'a [u8],
    data: Option<AceData>,
}
struct AceData {
    mask: u32,
    sid: Sid,
    object: Option<String>,
    inherited: Option<String>,
}

fn acl(bytes: &[u8], present: bool, offset: u32) -> Result<Acl<'_>, String> {
    if !present {
        return Ok(Acl::Absent);
    }
    if offset == 0 {
        return Ok(Acl::Null);
    }
    let bytes = component(bytes, offset)?;
    let header = bytes.get(..8).ok_or("truncated ACL header")?;
    if !matches!(header[0], 2 | 4) {
        return Err(format!("unsupported ACL revision {}", header[0]));
    }
    let size = usize::from(u16_at(header, 2)?);
    if size < 8 {
        return Err("ACL size is smaller than its header".into());
    }
    let bytes = bytes.get(..size).ok_or("ACL size exceeds the descriptor")?;
    let count = usize::from(u16_at(header, 4)?);
    if count > (size - 8) / 4 {
        return Err("ACE count cannot fit inside the ACL".into());
    }
    let mut entries = Vec::new();
    let mut at = 8;
    for index in 0..count {
        let header = bytes
            .get(at..at + 4)
            .ok_or_else(|| format!("ACE {} header is truncated", index + 1))?;
        let length = usize::from(u16_at(header, 2)?);
        if length < 4 || length % 4 != 0 {
            return Err(format!("ACE {} has an invalid size {length}", index + 1));
        }
        let raw = bytes
            .get(at..at + length)
            .ok_or_else(|| format!("ACE {} extends beyond the ACL", index + 1))?;
        let kind = header[0];
        let data = if matches!(kind, 0..=3 | 5..=8 | 17) {
            if matches!(kind, 5..=8) && bytes[0] != 4 {
                return Err("object ACE requires ACL revision 4".into());
            }
            decode_ace(raw).map_err(|e| format!("ACE {}: {e}", index + 1))?
        } else {
            None
        };
        entries.push(Ace {
            kind,
            flags: header[1],
            raw,
            data,
        });
        at += length;
    }
    Ok(Acl::Entries(entries))
}

fn decode_ace(raw: &[u8]) -> Result<Option<AceData>, String> {
    let mask = u32_at(raw, 4)?;
    let (mut at, mut object, mut inherited) = (8, None, None);
    if matches!(raw[0], 5..=8) {
        let flags = u32_at(raw, 8)?;
        if flags & !3 != 0 {
            return Ok(None);
        }
        at = 12;
        for (flag, value) in [(1, &mut object), (2, &mut inherited)] {
            if flags & flag != 0 {
                let bytes = raw.get(at..at + 16).ok_or("truncated object GUID")?;
                *value = Some(guid(bytes)?);
                at += 16;
            }
        }
    }
    let principal = sid(raw.get(at..).ok_or("missing ACE SID")?)?;
    // Callback conditions, resource attributes and unexpected trailing data must never be
    // silently dropped from an apparently complete SDDL/access interpretation.
    if at + principal.size != raw.len() {
        return Ok(None);
    }
    Ok(Some(AceData {
        mask,
        sid: principal,
        object,
        inherited,
    }))
}

fn guid(bytes: &[u8]) -> Result<String, String> {
    Ok(format!(
        "{:08x}-{:04x}-{:04x}-{}-{}",
        u32_at(bytes, 0)?,
        u16_at(bytes, 4)?,
        u16_at(bytes, 6)?,
        hex(&bytes[8..10]),
        hex(&bytes[10..16])
    ))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

const ACE_FLAGS: &[(u8, &str, &str)] = &[
    (1, "OI", "OBJECT_INHERIT_ACE"),
    (2, "CI", "CONTAINER_INHERIT_ACE"),
    (4, "NP", "NO_PROPAGATE_INHERIT_ACE"),
    (8, "IO", "INHERIT_ONLY_ACE"),
    (16, "ID", "INHERITED_ACE"),
    (32, "CR", "CRITICAL_ACE_FLAG"),
    (64, "SA", "SUCCESSFUL_ACCESS_ACE_FLAG"),
    (128, "FA", "FAILED_ACCESS_ACE_FLAG"),
];

fn ace_type(kind: u8) -> (&'static str, &'static str) {
    match kind {
        0 => ("ALLOW", "A"),
        1 => ("DENY", "D"),
        2 => ("AUDIT", "AU"),
        3 => ("ALARM", "AL"),
        5 => ("OBJECT ALLOW", "OA"),
        6 => ("OBJECT DENY", "OD"),
        7 => ("OBJECT AUDIT", "OU"),
        8 => ("OBJECT ALARM", "OL"),
        17 => ("MANDATORY INTEGRITY LABEL", "ML"),
        _ => ("UNSUPPORTED", ""),
    }
}

fn render_acl(
    acl: &Acl<'_>,
    name: &str,
    text: &mut String,
    sddl: &mut String,
    problems: &mut Vec<String>,
) -> bool {
    let entries = match acl {
        Acl::Absent => {
            let _ = writeln!(text, "{name}: Not present");
            return true;
        }
        Acl::Null => {
            let _ = writeln!(
                text,
                "{name}: NULL{}",
                if name == "DACL" {
                    " (no discretionary access restriction)"
                } else {
                    " (no audit entries recorded)"
                }
            );
            sddl.push_str("NO_ACCESS_CONTROL");
            return true;
        }
        Acl::Entries(entries) => entries,
    };
    let _ = writeln!(
        text,
        "{name}: {} entries{}",
        entries.len(),
        if entries.is_empty() {
            " (empty ACL)"
        } else {
            ""
        }
    );
    if name == "DACL" && entries.is_empty() {
        text.push_str("  No access is granted by DACL entries.\n");
    }
    let mut complete = true;
    for (index, ace) in entries.iter().enumerate() {
        let (kind, token) = ace_type(ace.kind);
        let _ = writeln!(
            text,
            "  Entry {}: {kind} (type 0x{:02X})",
            index + 1,
            ace.kind
        );
        let flags: Vec<_> = ACE_FLAGS
            .iter()
            .filter(|(bit, _, _)| ace.flags & bit != 0)
            .collect();
        let _ = writeln!(
            text,
            "    Flags: 0x{:02X} ({})",
            ace.flags,
            if flags.is_empty() {
                "none".into()
            } else {
                flags
                    .iter()
                    .map(|(_, _, name)| *name)
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        );
        if let Some(data) = &ace.data {
            let _ = writeln!(
                text,
                "    Account: {}\n    Access mask: 0x{:08X}\n    Rights: {}",
                data.sid.label(),
                data.mask,
                rights(data.mask, ace.kind)
            );
            if let Some(object) = &data.object {
                let _ = writeln!(text, "    Object type GUID: {object}");
            }
            if let Some(inherited) = &data.inherited {
                let _ = writeln!(text, "    Inherited object type GUID: {inherited}");
            }
            let flags = flags.iter().map(|(_, token, _)| *token).collect::<String>();
            let _ = write!(
                sddl,
                "({token};{flags};0x{:08x};{};{};{})",
                data.mask,
                data.object.as_deref().unwrap_or(""),
                data.inherited.as_deref().unwrap_or(""),
                data.sid.value
            );
        } else {
            text.push_str("    Unsupported ACE type or payload; no access interpretation.\n");
            let _ = writeln!(text, "    Raw ACE (hex): {}", hex(ace.raw));
            problems.push(format!(
                "{name} entry {}: unsupported ACE type 0x{:02X} or payload",
                index + 1,
                ace.kind
            ));
            complete = false;
        }
    }
    complete
}

fn rights(mask: u32, kind: u8) -> String {
    if mask == 0 {
        return "None".into();
    }
    let specific: &[(u32, &str)] = if kind == 17 {
        &[(1, "NO_WRITE_UP"), (2, "NO_READ_UP"), (4, "NO_EXECUTE_UP")]
    } else if matches!(kind, 5..=8) {
        &[]
    } else {
        if mask == 0x001f01ff {
            return "Full control (FILE_ALL_ACCESS)".into();
        }
        &[
            (1, "Read data / list directory"),
            (2, "Write data / add file"),
            (4, "Append data / add subdirectory"),
            (8, "Read extended attributes"),
            (16, "Write extended attributes"),
            (32, "Execute / traverse"),
            (64, "Delete child"),
            (128, "Read attributes"),
            (256, "Write attributes"),
        ]
    };
    let common: &[(u32, &str)] = &[
        (0x00010000, "DELETE"),
        (0x00020000, "READ_CONTROL"),
        (0x00040000, "WRITE_DAC"),
        (0x00080000, "WRITE_OWNER"),
        (0x00100000, "SYNCHRONIZE"),
        (0x01000000, "ACCESS_SYSTEM_SECURITY"),
        (0x02000000, "MAXIMUM_ALLOWED"),
        (0x10000000, "GENERIC_ALL"),
        (0x20000000, "GENERIC_EXECUTE"),
        (0x40000000, "GENERIC_WRITE"),
        (0x80000000, "GENERIC_READ"),
    ];
    let mut names = Vec::new();
    let mut remaining = mask;
    for &(bit, name) in specific.iter().chain(common) {
        if remaining & bit != 0 {
            names.push(name.to_owned());
            remaining &= !bit;
        }
    }
    if remaining != 0 {
        names.push(format!(
            "0x{remaining:08X} (object-specific or unrecognized bits)"
        ));
    }
    names.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sid_bytes(authority: u64, subauthorities: &[u32]) -> Vec<u8> {
        let mut out = vec![1, subauthorities.len() as u8];
        out.extend_from_slice(&authority.to_be_bytes()[2..]);
        for value in subauthorities {
            out.extend_from_slice(&value.to_le_bytes());
        }
        out
    }

    fn ace(kind: u8, flags: u8, mask: u32, extra: &[u8], principal: &[u8]) -> Vec<u8> {
        let mut out = vec![kind, flags, 0, 0];
        out.extend_from_slice(&mask.to_le_bytes());
        out.extend_from_slice(extra);
        out.extend_from_slice(principal);
        let size = out.len() as u16;
        out[2..4].copy_from_slice(&size.to_le_bytes());
        out
    }

    fn acl_bytes(revision: u8, entries: &[Vec<u8>]) -> Vec<u8> {
        let mut out = vec![revision, 0, 0, 0];
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&[0, 0]);
        for entry in entries {
            out.extend_from_slice(entry);
        }
        let size = out.len() as u16;
        out[2..4].copy_from_slice(&size.to_le_bytes());
        out
    }

    fn descriptor(control: u16, components: [Option<&[u8]>; 4]) -> Vec<u8> {
        let mut out = vec![0; 20];
        out[0] = 1;
        out[2..4].copy_from_slice(&control.to_le_bytes());
        for (i, bytes) in components.into_iter().enumerate() {
            if let Some(bytes) = bytes {
                let offset = out.len() as u32;
                out[4 + i * 4..8 + i * 4].copy_from_slice(&offset.to_le_bytes());
                out.extend_from_slice(bytes);
            }
        }
        out
    }

    fn admin_owned() -> Vec<u8> {
        let admin = sid_bytes(5, &[32, 544]);
        let system = sid_bytes(5, &[18]);
        let group = sid_bytes(5, &[21, 1111, 2222, 3333, 513]);
        let dacl = acl_bytes(
            2,
            &[
                ace(0, 0x10, 0x1f01ff, &[], &admin),
                ace(0, 0x10, 0x1f01ff, &[], &system),
            ],
        );
        descriptor(0x8404, [Some(&admin), Some(&group), None, Some(&dacl)])
    }

    #[test]
    fn inherited_full_control_and_unresolved_domain_sid() {
        let decoded = report(&admin_owned());
        assert!(decoded.problems.is_empty(), "{:?}", decoded.problems);
        assert!(
            decoded
                .text
                .contains("Owner: S-1-5-32-544 (BUILTIN\\Administrators)")
        );
        assert!(
            decoded
                .text
                .contains("Primary group: S-1-5-21-1111-2222-3333-513 (account name not resolved)")
        );
        assert!(!decoded.text.contains("Domain Users"));
        assert_eq!(
            decoded
                .text
                .matches("Full control (FILE_ALL_ACCESS)")
                .count(),
            2
        );
        assert_eq!(decoded.text.matches("INHERITED_ACE").count(), 2);
        assert!(decoded.text.contains("SACL: Not present"));
        assert!(decoded.text.contains("O:S-1-5-32-544G:S-1-5-21-1111-2222-3333-513D:AI(A;ID;0x001f01ff;;;S-1-5-32-544)(A;ID;0x001f01ff;;;S-1-5-18)"));
        assert!(decoded.text.contains("Decode status: complete"));
    }

    #[test]
    fn absent_null_and_empty_acl_are_distinct() {
        let absent = report(&descriptor(0x8000, [None; 4]));
        assert!(absent.text.contains("DACL: Not present"));
        assert!(!absent.text.contains("D:NO_ACCESS_CONTROL"));
        let null = report(&descriptor(0x8014, [None; 4]));
        assert!(null.problems.is_empty());
        assert!(
            null.text
                .contains("DACL: NULL (no discretionary access restriction)")
        );
        assert!(null.text.contains("D:NO_ACCESS_CONTROLS:NO_ACCESS_CONTROL"));
        let empty_acl = acl_bytes(2, &[]);
        let empty = report(&descriptor(0x8004, [None, None, None, Some(&empty_acl)]));
        assert!(empty.problems.is_empty());
        assert!(empty.text.contains("DACL: 0 entries (empty ACL)"));
        assert!(empty.text.contains("\nD:\n"));
        assert!(!empty.text.contains("NO_ACCESS_CONTROL"));
    }

    #[test]
    fn audit_and_integrity_rights_are_not_file_rights() {
        let user = sid_bytes(1, &[0]);
        let high = sid_bytes(16, &[12288]);
        let sacl = acl_bytes(2, &[ace(2, 0xc0, 1, &[], &user), ace(17, 0, 7, &[], &high)]);
        let decoded = report(&descriptor(0xaa10, [None, None, Some(&sacl), None]));
        assert!(decoded.problems.is_empty(), "{:?}", decoded.problems);
        assert!(decoded.text.contains("Entry 1: AUDIT"));
        assert!(
            decoded
                .text
                .contains("SUCCESSFUL_ACCESS_ACE_FLAG, FAILED_ACCESS_ACE_FLAG")
        );
        assert!(
            decoded
                .text
                .contains("NO_WRITE_UP, NO_READ_UP, NO_EXECUTE_UP")
        );
        assert!(decoded.text.contains("S-1-16-12288 (High integrity)"));
        assert!(
            decoded
                .text
                .contains("S:PARAI(AU;SAFA;0x00000001;;;S-1-1-0)(ML;;0x00000007;;;S-1-16-12288)")
        );
    }

    #[test]
    fn object_ace_optional_guids_and_deny_order_are_preserved() {
        let principal = sid_bytes(5, &[18]);
        let mut entries = vec![ace(1, 8, 0x80000000, &[], &principal)];
        let object = [
            0x33, 0x22, 0x11, 0, 0x55, 0x44, 0x77, 0x66, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
            0xff,
        ];
        for flags in 0u32..=3 {
            let mut extra = flags.to_le_bytes().to_vec();
            if flags & 1 != 0 {
                extra.extend_from_slice(&object);
            }
            if flags & 2 != 0 {
                extra.extend_from_slice(&object);
            }
            entries.push(ace(5, 3, 0x100, &extra, &principal));
        }
        let dacl = acl_bytes(4, &entries);
        let decoded = report(&descriptor(0x9104, [None, None, None, Some(&dacl)]));
        assert!(decoded.problems.is_empty(), "{:?}", decoded.problems);
        assert!(decoded.text.contains("Entry 1: DENY"));
        assert!(decoded.text.contains("INHERIT_ONLY_ACE"));
        assert!(
            decoded
                .text
                .contains("00112233-4455-6677-8899-aabbccddeeff")
        );
        assert!(
            decoded
                .text
                .contains("D:PAR(D;IO;0x80000000;;;S-1-5-18)(OA;OICI;0x00000100;;;S-1-5-18)")
        );
        assert!(
            decoded
                .text
                .contains("0x00000100 (object-specific or unrecognized bits)")
        );
    }

    #[test]
    fn unsupported_aces_keep_raw_bytes_and_suppress_incomplete_sddl() {
        let principal = sid_bytes(5, &[18]);
        let mut callback = ace(9, 0, 1, &[], &principal);
        callback.extend_from_slice(b"artx");
        let size = callback.len() as u16;
        callback[2..4].copy_from_slice(&size.to_le_bytes());
        let dacl = acl_bytes(2, &[callback.clone(), ace(0, 0, 1, &[], &principal)]);
        let decoded = report(&descriptor(0x8004, [None, None, None, Some(&dacl)]));
        assert_eq!(decoded.problems.len(), 1);
        assert!(
            decoded
                .text
                .contains(&format!("Raw ACE (hex): {}", hex(&callback)))
        );
        assert!(decoded.text.contains("Entry 2: ALLOW"));
        assert!(decoded.text.contains("SDDL: unavailable"));
        assert!(decoded.text.contains("Decode status: incomplete"));
        assert!(
            !decoded
                .text
                .contains("SDDL (owner, group and ACL entries):")
        );
        let unknown = acl_bytes(2, &[vec![0xff, 0, 4, 0]]);
        assert!(
            report(&descriptor(0x8004, [None, None, None, Some(&unknown)])).problems[0]
                .contains("0xFF")
        );
    }

    #[test]
    fn damaged_offsets_and_lengths_are_reported_without_panicking() {
        let bytes = admin_owned();
        for end in 0..bytes.len() {
            let decoded = report(&bytes[..end]);
            assert!(
                !decoded.problems.is_empty(),
                "accepted truncated length {end}"
            );
        }
        for at in [4, 8, 16] {
            for offset in [1u32, 4, 19, 21, u32::MAX] {
                let mut damaged = bytes.clone();
                damaged[at..at + 4].copy_from_slice(&offset.to_le_bytes());
                assert!(
                    !report(&damaged).problems.is_empty(),
                    "offset {at}: {offset}"
                );
            }
        }
        let dacl_at = u32_at(&bytes, 16).unwrap() as usize;
        for (at, value) in [
            (dacl_at + 2, 4u16),
            (dacl_at + 2, u16::MAX),
            (dacl_at + 4, u16::MAX),
            (dacl_at + 10, 0),
            (dacl_at + 10, 7),
            (dacl_at + 10, u16::MAX),
        ] {
            let mut damaged = bytes.clone();
            damaged[at..at + 2].copy_from_slice(&value.to_le_bytes());
            assert!(
                !report(&damaged).problems.is_empty(),
                "ACL field {at}: {value}"
            );
        }
        let mut absolute = bytes.clone();
        absolute[3] &= 0x7f;
        assert!(report(&absolute).problems[0].contains("not self-relative"));
        let mut bad_sid = bytes.clone();
        bad_sid[21] = 255;
        assert!(report(&bad_sid).problems[0].contains("SID"));
        let mut bad_revision = bytes.clone();
        bad_revision[0] = 2;
        assert!(report(&bad_revision).problems[0].contains("revision"));
    }

    #[test]
    fn unexpected_ace_payload_is_never_silently_discarded() {
        let principal = sid_bytes(5, &[18]);
        let mut entry = ace(0, 0, 1, &[], &principal);
        entry.extend_from_slice(&[0; 4]);
        let length = entry.len() as u16;
        entry[2..4].copy_from_slice(&length.to_le_bytes());
        let dacl = acl_bytes(2, &[entry]);
        let decoded = report(&descriptor(0x8004, [None, None, None, Some(&dacl)]));
        assert!(!decoded.problems.is_empty());
        assert!(decoded.text.contains("SDDL: unavailable"));
        assert!(decoded.text.contains("Raw ACE (hex):"));
    }

    #[test]
    fn unusual_sid_authorities_and_access_bits_remain_visible() {
        let principal = sid_bytes(0xffff_ffff_ffff, &[u32::MAX]);
        assert_eq!(
            sid(&principal).unwrap().value,
            "S-1-0xffffffffffff-4294967295"
        );
        let dacl = acl_bytes(2, &[ace(0, 0, 0x80000200, &[], &principal)]);
        let decoded = report(&descriptor(0x8004, [None, None, None, Some(&dacl)]));
        assert!(decoded.problems.is_empty());
        assert!(
            decoded
                .text
                .contains("GENERIC_READ, 0x00000200 (object-specific or unrecognized bits)")
        );
    }

    #[test]
    fn adversarial_mutations_do_not_panic() {
        let original = admin_owned();
        for i in 0..original.len() {
            for value in [0, 1, 4, 16, 0x80, 0xff] {
                let mut bytes = original.clone();
                bytes[i] = value;
                let _ = report(&bytes);
            }
        }
    }
}
