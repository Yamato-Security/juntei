//! Allow/block classification, adapted from PyrsistenceSniper (MIT).
//! A name in signature metadata is never proof of a valid signature.

use super::{Finding, checks::CheckDefinition};
use regex::{Regex, RegexBuilder};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    #[serde(default)]
    reason: String,
    #[serde(default)]
    value_matches: String,
    #[serde(default)]
    path_matches: String,
    #[serde(default)]
    signer: String,
    #[serde(default)]
    hash: String,
    #[serde(default)]
    not_lolbin: bool,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCheck {
    enabled: Option<bool>,
    #[serde(default)]
    allow: Vec<RawRule>,
    #[serde(default)]
    block: Vec<RawRule>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProfile {
    #[serde(default)]
    allow: Vec<RawRule>,
    #[serde(default)]
    block: Vec<RawRule>,
    #[serde(default)]
    checks: BTreeMap<String, RawCheck>,
}

struct Rule {
    raw: RawRule,
    value: Option<Regex>,
    path: Option<Regex>,
}

#[derive(Default)]
struct Policy {
    enabled: bool,
    allow: Vec<Rule>,
    block: Vec<Rule>,
}

pub struct Profile {
    global: Policy,
    checks: BTreeMap<String, Policy>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Match {
    None,
    Partial,
    Full,
}

impl Rule {
    fn compile(raw: RawRule) -> Result<Self, String> {
        let compile = |s: &str| {
            if s.is_empty() {
                Ok(None)
            } else {
                RegexBuilder::new(s)
                    .case_insensitive(true)
                    .size_limit(8 * 1024 * 1024)
                    .build()
                    .map(Some)
                    .map_err(|e| format!("Invalid persistence profile expression {s:?}: {e}"))
            }
        };
        let value = compile(&raw.value_matches)?;
        let path = compile(&raw.path_matches)?;
        if !raw.hash.is_empty()
            && (raw.hash.len() != 64 || !raw.hash.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err(
                "A persistence profile hash must contain 64 SHA256 hexadecimal digits".into(),
            );
        }
        Ok(Self { raw, value, path })
    }

    fn matches(&self, f: &Finding) -> Match {
        let mut hard = 0;
        if self.raw.not_lolbin {
            hard += 1;
            if f.is_lolbin != Some(false) {
                return Match::None;
            }
        }
        for (regex, value) in [(&self.value, &f.value), (&self.path, &f.path)] {
            if let Some(r) = regex {
                hard += 1;
                if !r.is_match(value) {
                    return Match::None;
                }
            }
        }
        if !self.raw.hash.is_empty() {
            hard += 1;
            if !self.raw.hash.eq_ignore_ascii_case(&f.sha256) {
                return Match::None;
            }
        }
        if hard == 0 && self.raw.signer.is_empty() {
            return Match::None;
        }
        if self.raw.signer.is_empty()
            || (!f.signer.is_empty()
                && f.signer
                    .to_lowercase()
                    .contains(&self.raw.signer.to_lowercase()))
        {
            Match::Full
        } else if hard > 0 {
            Match::Partial
        } else {
            Match::None
        }
    }
}

impl Profile {
    pub fn load(path: Option<&Path>, definitions: &[CheckDefinition]) -> Result<Self, String> {
        let text = if let Some(path) = path {
            let meta = path
                .metadata()
                .map_err(|e| format!("{}: {e}", path.display()))?;
            if meta.len() > 8 * 1024 * 1024 {
                return Err("Persistence profile exceeds the 8 MiB limit".into());
            }
            std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?
        } else {
            include_str!("default_profile.yaml").into()
        };
        Self::parse(&text, definitions)
    }

    fn parse(text: &str, definitions: &[CheckDefinition]) -> Result<Self, String> {
        let raw: RawProfile = serde_yaml_ng::from_str(text)
            .map_err(|e| format!("Invalid persistence profile: {e}"))?;
        fn policy(c: RawCheck) -> Result<Policy, String> {
            Ok(Policy {
                enabled: c.enabled.unwrap_or(true),
                allow: c
                    .allow
                    .into_iter()
                    .map(Rule::compile)
                    .collect::<Result<_, _>>()?,
                block: c
                    .block
                    .into_iter()
                    .map(Rule::compile)
                    .collect::<Result<_, _>>()?,
            })
        }
        let global = policy(RawCheck {
            enabled: Some(true),
            allow: raw.allow,
            block: raw.block,
        })?;
        let mut checks = BTreeMap::new();
        for (id, check) in raw.checks {
            if !definitions.iter().any(|d| d.id == id) {
                return Err(format!(
                    "Unknown persistence check {id:?} in profile; use --list-checks"
                ));
            }
            checks.insert(id, policy(check)?);
        }
        Ok(Self { global, checks })
    }

    pub fn enabled(&self, check: &str) -> bool {
        self.checks.get(check).is_none_or(|p| p.enabled)
    }

    pub fn classify(&self, finding: &mut Finding) {
        let policies = [Some(&self.global), self.checks.get(&finding.check_id)];
        for p in policies.into_iter().flatten() {
            if let Some(rule) = p.block.iter().find(|r| r.matches(finding) == Match::Full) {
                finding.severity = "HIGH".into();
                finding.policy_reason = rule.raw.reason.clone();
                if !rule.raw.signer.is_empty() {
                    finding
                        .policy_reason
                        .push_str(" (signer metadata matched; signature validity not verified)");
                }
                return;
            }
        }
        let mut best = Match::None;
        let mut reason = String::new();
        let mut signer_condition = false;
        for p in policies.into_iter().flatten() {
            for r in &p.allow {
                let m = r.matches(finding);
                if m > best {
                    best = m;
                    reason.clone_from(&r.raw.reason);
                    signer_condition = !r.raw.signer.is_empty();
                }
            }
        }
        let base = finding.base_classification.unwrap_or(("MEDIUM", ""));
        finding.severity = match best {
            Match::None => base.0,
            // An allow rule can lower severity, never promote INFO inventory.
            Match::Partial => {
                if base.0 == "INFO" {
                    "INFO"
                } else {
                    "LOW"
                }
            }
            Match::Full => "INFO",
        }
        .into();
        finding.policy_reason = if best == Match::None {
            base.1.into()
        } else {
            reason
        };
        if best == Match::Partial {
            finding
                .policy_reason
                .push_str(" (partial allow match; signer metadata missing or different)");
        } else if best == Match::Full && signer_condition {
            finding
                .policy_reason
                .push_str(" (signer metadata matched; signature validity not verified)");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_profile_is_valid() {
        Profile::load(None, super::super::checks::catalogue()).unwrap();
    }

    #[test]
    fn block_wins_and_partial_allow_is_low() {
        let defs = super::super::checks::catalogue();
        let profile = Profile::parse("allow:\n  - value_matches: '^example\\.exe$'\n    signer: Microsoft\n    reason: expected\nblock:\n  - hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n    reason: blocked\n", defs).unwrap();
        let mut f = Finding {
            value: "EXAMPLE.exe".into(),
            ..Default::default()
        };
        profile.classify(&mut f);
        assert_eq!(f.severity, "LOW");
        assert!(f.policy_reason.contains("partial allow match"));
        f.signer = "Microsoft Corporation".into();
        profile.classify(&mut f);
        assert_eq!(f.severity, "INFO");
        assert!(f.policy_reason.contains("signature validity not verified"));
        f.sha256 = "a".repeat(64);
        profile.classify(&mut f);
        assert_eq!(f.severity, "HIGH");
        assert_eq!(f.policy_reason, "blocked");
    }

    #[test]
    fn partial_allow_never_raises_a_checks_base_classification() {
        let defs = super::super::checks::catalogue();
        let profile = Profile::parse("allow:\n  - value_matches: '^example\\.exe$'\n    signer: Microsoft\n    reason: expected\nblock:\n  - hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n    reason: blocked\n", defs).unwrap();
        for (base, expected) in [
            ("INFO", "INFO"),
            ("LOW", "LOW"),
            ("MEDIUM", "LOW"),
            ("HIGH", "LOW"),
        ] {
            let mut f = Finding {
                value: "example.exe".into(),
                base_classification: Some((base, "check default")),
                resolution_unavailable: true,
                ..Default::default()
            };
            profile.classify(&mut f);
            assert_eq!(f.severity, expected, "base {base}");
            assert!(f.policy_reason.contains("partial allow match"));
            assert!(f.signer.is_empty());
            f.sha256 = "a".repeat(64);
            profile.classify(&mut f);
            assert_eq!(f.severity, "HIGH", "explicit block still wins");
        }
    }

    #[test]
    fn unknown_or_invalid_profile_is_error() {
        let defs = super::super::checks::catalogue();
        for text in [
            "checks:\n  typo: {}",
            "allow:\n  - value_matches: '['",
            "allow:\n  - hash: abc",
            "checks:\n  run_keys:\n    typo: true",
        ] {
            assert!(Profile::parse(text, defs).is_err(), "{text}");
        }
    }

    #[test]
    fn custom_profile_replaces_bundled_and_empty_rules_do_not_allow() {
        let defs = super::super::checks::catalogue();
        let p = Profile::parse("allow: [{}]", defs).unwrap();
        let mut f = Finding {
            check_id: "winlogon_shell".into(),
            value: "explorer.exe".into(),
            ..Default::default()
        };
        p.classify(&mut f);
        assert_eq!(f.severity, "MEDIUM");
    }
}
