use super::Check;
use crate::persistence_sniper::context::Resolution;
use crate::persistence_sniper::resolver::pe_metadata;

const TOOLS: &[(&str, &[&str])] = &[
    ("windows/system32/sethc.exe", &["sethc.exe"]),
    ("windows/system32/osk.exe", &["osk.exe"]),
    ("windows/system32/narrator.exe", &["narrator.exe", "sr.exe"]),
    (
        "windows/system32/magnify.exe",
        &["magnify.exe", "screenmagnifier.exe"],
    ),
    (
        "windows/system32/utilman.exe",
        &["utilman.exe", "utilman2.exe"],
    ),
    ("windows/system32/atbroker.exe", &["atbroker.exe"]),
    ("windows/system32/displayswitch.exe", &["displayswitch.exe"]),
];

pub(super) fn run(check: &mut Check<'_>) {
    let files = check.files(|path| TOOLS.iter().any(|(p, _)| *p == path));
    if files.is_empty() {
        return;
    }
    check.sources += files.len();
    let reference = [
        "Windows/System32/winlogon.exe",
        "Windows/System32/cmd.exe",
        "Windows/System32/notepad.exe",
    ]
    .iter()
    .any(|path| {
        let Resolution::Found(file) = check.ctx.resolve(path) else {
            return false;
        };
        let metadata = pe_metadata(check.ctx, &file.physical);
        metadata.signer_status == "catalog_certificate_unverified"
            && metadata
                .signer
                .trim()
                .to_lowercase()
                .starts_with("microsoft")
    });
    if !reference {
        check.issue("unresolved","Accessibility replacement comparison was not run: none of the collected reference binaries resolved to Microsoft catalog signer metadata. A sparse collection without catalogs must not produce unsigned-replacement false positives.");
        return;
    }
    check.issue("metadata_only","Accessibility comparisons use unverified certificate/catalog metadata and PE OriginalFilename for upstream parity. This is not signature/trust-chain verification.");
    for file in files {
        if !check.safe_file(&file) {
            continue;
        }
        let metadata = pe_metadata(check.ctx, &file.physical);
        let allowed = TOOLS
            .iter()
            .find(|(p, _)| p.eq_ignore_ascii_case(&file.logical))
            .map(|(_, names)| *names)
            .unwrap_or(&[]);
        let reason = match replacement_reason(&metadata, allowed) {
            Ok(reason) => reason,
            Err(reason) => {
                check.issue(
                    "unresolved",
                    format!("{}: {reason}", file.physical.display()),
                );
                None
            }
        };
        if let Some(reason) = reason {
            let mut finding = check.file_finding(&file, &reason, "SYSTEM");
            finding.description = format!(
                "Accessibility tool replacement candidate: {reason}. Certificate display metadata is unverified; this finding does not prove tampering or execution."
            );
            finding.signer = metadata.signer;
            finding.signer_status = metadata.signer_status;
            check.scan.findings.push(finding);
        }
    }
}

fn replacement_reason(
    metadata: &crate::persistence_sniper::resolver::PeMetadata,
    allowed: &[&str],
) -> Result<Option<String>, &'static str> {
    if metadata
        .signer
        .trim()
        .to_ascii_lowercase()
        .starts_with("microsoft")
    {
        match metadata.original_filename.as_deref() {
            Some(name) if !allowed.iter().any(|n| n.eq_ignore_ascii_case(name.trim())) => {
                Ok(Some(format!(
                    "Microsoft-named certificate metadata but unexpected PE OriginalFilename: {name}"
                )))
            }
            None => Err("no readable OriginalFilename identity"),
            _ => Ok(None),
        }
    } else {
        Ok(Some(format!(
            "No matching Microsoft embedded/catalog signer metadata (status: {}, signer: {})",
            metadata.signer_status,
            if metadata.signer.is_empty() {
                "not recorded"
            } else {
                &metadata.signer
            }
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence_sniper::resolver::PeMetadata;

    #[test]
    fn missing_or_damaged_metadata_cannot_hide_a_replacement_after_reference_validation() {
        for status in [
            "pe_metadata_exceeds_64MiB_limit_or_unreadable",
            "damaged_pe_certificate_bounds",
            "damaged_pe_authenticode_bounds",
            "catalog_unreadable_or_damaged",
            "not_pe",
            "no_embedded_signature_no_catalog_match",
        ] {
            let metadata = PeMetadata {
                signer: String::new(),
                signer_status: status.into(),
                original_filename: None,
            };
            assert!(
                replacement_reason(&metadata, &["sethc.exe"])
                    .unwrap()
                    .unwrap()
                    .contains(status)
            );
        }
        let mut metadata = PeMetadata {
            signer: "Microsoft Windows".into(),
            signer_status: "catalog_certificate_unverified".into(),
            original_filename: Some("sethc.exe".into()),
        };
        assert!(
            replacement_reason(&metadata, &["sethc.exe"])
                .unwrap()
                .is_none()
        );
        metadata.original_filename = Some("cmd.exe".into());
        assert!(
            replacement_reason(&metadata, &["sethc.exe"])
                .unwrap()
                .is_some()
        );
        metadata.original_filename = None;
        assert!(replacement_reason(&metadata, &["sethc.exe"]).is_err());
    }
}
