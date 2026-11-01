//! Native offline persistence hunting, adapted from Hexastrike's MIT-licensed
//! PyrsistenceSniper. See THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt.

pub mod checks;
mod com;
mod command;
pub mod context;
pub mod custom;
pub mod file_checks;
pub mod profile;
pub mod resolver;
pub mod timeline;

pub use command::run;

use serde::{Deserialize, Serialize};

/// A candidate persistence entry. This is evidence of configuration, not a
/// verdict that the configured program is malicious or has executed.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Finding {
    pub computer: String,
    pub check_id: String,
    pub technique: String,
    pub mitre_id: String,
    pub path: String,
    pub value: String,
    pub description: String,
    pub access_gained: String,
    pub severity: String,
    pub policy_reason: String,
    pub source_file: String,
    pub record_status: String,
    pub key_last_write: String,
    pub last_change: String,
    pub change_source: String,
    pub change_evidence: String,
    pub resolved_path: String,
    pub resolution_status: String,
    pub exists: Option<bool>,
    pub sha256: String,
    pub signer: String,
    pub signer_status: String,
    pub is_lolbin: Option<bool>,
    pub is_builtin: Option<bool>,
    pub is_in_os_directory: Option<bool>,
    pub launcher: String,
    pub references: String,
    #[serde(skip)]
    pub resolve_target: String,
    #[serde(skip)]
    pub time_evidence: Vec<TimeEvidence>,
}

#[derive(Clone, Debug)]
pub enum TimeEvidence {
    FileWrite {
        path: String,
        weak: bool,
    },
    EventLog {
        channel: String,
        event_ids: Vec<u64>,
        match_field: String,
        match_value: String,
    },
}

impl Finding {
    pub fn new(
        check_id: impl Into<String>,
        technique: impl Into<String>,
        mitre_id: impl Into<String>,
        path: impl Into<String>,
        value: impl Into<String>,
        access: impl Into<String>,
    ) -> Self {
        Self {
            check_id: check_id.into(),
            technique: technique.into(),
            mitre_id: mitre_id.into(),
            path: path.into(),
            value: value.into(),
            access_gained: access.into(),
            severity: "MEDIUM".into(),
            change_evidence: "NOT_RUN".into(),
            ..Default::default()
        }
    }
}

/// Each check reports its coverage, even when it produces no findings.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Coverage {
    pub computer: String,
    pub check_id: String,
    pub status: String,
    pub detail: String,
}

impl Coverage {
    pub fn new(
        check_id: impl Into<String>,
        status: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            computer: String::new(),
            check_id: check_id.into(),
            status: status.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Default)]
pub struct Scan {
    pub findings: Vec<Finding>,
    pub coverage: Vec<Coverage>,
}
