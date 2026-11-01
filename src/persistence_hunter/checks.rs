//! Native registry checks adapted from PyrsistenceSniper (MIT).
//! Copyright (c) Hexastrike. See THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt.

use super::context::{Context, HiveInput};
use super::{Coverage, Finding, Scan};
use registry::{Data, Key, Value};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::OnceLock;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegistryTarget {
    pub path: String,
    pub values: String,
    pub scope: String,
    pub recurse: bool,
    pub include_wow64: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CheckDefinition {
    pub id: String,
    pub technique: String,
    pub mitre_id: String,
    pub description: String,
    pub references: Vec<String>,
    pub implementation: String,
    pub targets: Vec<RegistryTarget>,
    pub hives: Vec<String>,
    pub builtin_allow: Vec<serde_json::Value>,
    pub builtin_block: Vec<serde_json::Value>,
}

pub fn catalogue() -> &'static [CheckDefinition] {
    static CHECKS: OnceLock<Vec<CheckDefinition>> = OnceLock::new();
    CHECKS.get_or_init(|| {
        serde_json::from_str(include_str!("catalogue.json"))
            .expect("embedded check catalogue is valid")
    })
}

pub(crate) const FILE_CHECKS: &[&str] = &[
    "start_menu_com",
    "gp_scripts",
    "scheduled_task_files",
    "ghost_task",
    "hidden_scheduled_task",
    "bits_notify_command",
    "accessibility_tools",
    "power_automate",
    "powershell_profiles",
    "windows_terminal",
    "wmi_event_subscription",
    "error_handler_cmd",
    "startup_folder",
    "shell_folders_startup",
    "office_templates",
];

/// Execute registry checks once per host. Missing evidence remains visible in coverage.
#[cfg_attr(not(test), allow(dead_code))]
pub fn run(ctx: &Context) -> Scan {
    run_definitions(ctx, catalogue())
}

pub fn run_definitions(ctx: &Context, definitions: &[CheckDefinition]) -> Scan {
    let mut result = Scan::default();
    for check in definitions {
        if FILE_CHECKS.contains(&check.id.as_str()) {
            continue;
        }
        let mut run = CheckRun::new(ctx, check);
        if check.implementation == "declarative"
            || ["winlogon_userinit", "winlogon_appsetup", "winlogon_system"]
                .contains(&check.id.as_str())
        {
            run.declarative();
        } else {
            super::custom::execute(&mut run);
        }
        let (mut findings, coverage) = run.finish();
        result.findings.append(&mut findings);
        result.coverage.push(coverage);
    }
    result
}

/// A decoded registry value. Binary values stay binary for SAM/account checks.
#[derive(Clone, Debug)]
pub(crate) enum RegData {
    Text(String),
    Multi(Vec<String>),
    Number(u64),
    Bytes(Vec<u8>),
}
impl RegData {
    pub fn text(&self) -> String {
        match self {
            Self::Text(v) => v.clone(),
            Self::Number(v) => v.to_string(),
            Self::Multi(v) => v.join("; "),
            Self::Bytes(v) => v.iter().map(|b| format!("{b:02x}")).collect(),
        }
    }
    pub fn strings(&self) -> Vec<String> {
        match self {
            Self::Multi(v) => v.clone(),
            _ => vec![self.text()],
        }
    }
    pub fn number(&self) -> Option<u64> {
        match self {
            Self::Number(n) => Some(*n),
            Self::Text(s) => parse_number(s),
            _ => None,
        }
    }
}
pub(crate) fn parse_number(s: &str) -> Option<u64> {
    let s = s.trim();
    if let Some(s) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(s, 16).ok()
    } else {
        s.parse().ok()
    }
}

pub(crate) struct CheckRun<'a> {
    pub ctx: &'a Context,
    pub check: &'a CheckDefinition,
    pub findings: Vec<Finding>,
    pub issues: Vec<String>,
    pub inputs: BTreeSet<String>,
}
impl<'a> CheckRun<'a> {
    fn new(ctx: &'a Context, check: &'a CheckDefinition) -> Self {
        Self {
            ctx,
            check,
            findings: vec![],
            issues: vec![],
            inputs: BTreeSet::new(),
        }
    }
    pub fn machine(&mut self, kind: &str) -> Option<&'a HiveInput> {
        let h = self.ctx.machine_hive(kind)?;
        self.inputs.insert(h.path.display().to_string());
        if h.opened.recovery.dirty && !h.opened.recovery.recovered {
            self.issue(format!(
                "{} is dirty without successfully replayed transaction logs",
                h.path.display()
            ));
        }
        Some(h)
    }
    pub fn users(&mut self, kind: &str) -> Vec<&'a HiveInput> {
        let hs: Vec<_> = self
            .ctx
            .hives
            .iter()
            .filter(|h| h.kind.eq_ignore_ascii_case(kind))
            .collect();
        for h in &hs {
            self.inputs.insert(h.path.display().to_string());
            if h.opened.recovery.dirty && !h.opened.recovery.recovered {
                self.issue(format!(
                    "{} is dirty without successfully replayed transaction logs",
                    h.path.display()
                ));
            }
        }
        hs
    }
    pub fn issue(&mut self, problem: impl Into<String>) {
        let p = problem.into();
        if !self.issues.contains(&p) {
            self.issues.push(p);
        }
    }
    pub fn children(&mut self, key: Key<'a>) -> Vec<Key<'a>> {
        let (v, issues) = key.subkeys();
        for p in issues {
            self.issue(p);
        }
        v
    }
    pub fn child(&mut self, key: Key<'a>, name: &str) -> Option<Key<'a>> {
        self.children(key)
            .into_iter()
            .find(|k| k.name().eq_ignore_ascii_case(name))
    }
    pub fn key(&mut self, h: &'a HiveInput, path: &str) -> Option<Key<'a>> {
        let Some(mut key) = h.opened.hive.root() else {
            self.issue(format!("{} has no readable root key", h.path.display()));
            return None;
        };
        for part in path.split('\\').filter(|p| !p.is_empty()) {
            key = self.child(key, part)?;
        }
        Some(key)
    }
    pub fn values(&mut self, key: Key<'a>) -> Vec<(String, RegData)> {
        let (values, issues) = key.values();
        for p in issues {
            self.issue(p);
        }
        values
            .into_iter()
            .filter_map(|v| self.decode(v).map(|data| (v.name(), data)))
            .collect()
    }
    pub fn value(&mut self, key: Key<'a>, name: &str) -> Option<RegData> {
        // Reading the full value list also exposes damage rather than silently equating it with absence.
        let actual = if name.eq_ignore_ascii_case("(Default)") {
            ""
        } else {
            name
        };
        self.values(key)
            .into_iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(actual))
            .map(|(_, v)| v)
    }
    pub fn text(&mut self, key: Key<'a>, name: &str) -> Option<String> {
        self.value(key, name)
            .map(|v| v.text().trim().to_owned())
            .filter(|v| !v.is_empty())
    }
    pub fn number(&mut self, key: Key<'a>, name: &str) -> Option<u64> {
        self.value(key, name).and_then(|v| v.number())
    }
    fn decode(&mut self, value: Value<'a>) -> Option<RegData> {
        let (bytes, problem) = value.read_data();
        if let Some(p) = problem {
            self.issue(p);
        }
        match registry::decode(value.data_type(), &bytes) {
            Data::Text { value, lossy } => {
                if lossy {
                    self.issue("Lossy registry string decoding");
                }
                Some(RegData::Text(value))
            }
            Data::MultiText { values, lossy } => {
                if lossy {
                    self.issue("Lossy registry multi-string decoding");
                }
                Some(RegData::Multi(values))
            }
            Data::Number(n) => Some(RegData::Number(n)),
            Data::Bytes | Data::Undecoded => Some(RegData::Bytes(bytes)),
            Data::Malformed(problem) => {
                self.issue(problem);
                None
            }
        }
    }
    pub fn emit(
        &mut self,
        h: &HiveInput,
        key: Key<'_>,
        path: impl Into<String>,
        value: impl Into<String>,
        access: &str,
    ) -> usize {
        let mut f = Finding::new(
            &self.check.id,
            &self.check.technique,
            &self.check.mitre_id,
            path,
            value,
            access,
        );
        f.computer = self.ctx.computer.clone();
        f.description = self.check.description.clone();
        f.source_file = h.path.display().to_string();
        f.key_last_write = crate::registry::filetime(key.last_written()).unwrap_or_default();
        f.record_status = format!("{:?}", key.status()).to_lowercase();
        f.references = self.check.references.join("; ");
        self.findings.push(f);
        self.findings.len() - 1
    }
    pub fn default_server(&mut self, h: &'a HiveInput, path: &str) -> String {
        h.opened
            .hive
            .key(path)
            .and_then(|n| self.text(n, "(Default)"))
            .unwrap_or_default()
    }
    pub fn clsid_server(&mut self, h: &'a HiveInput, root: &str, clsid: &str) -> String {
        if !clsid.starts_with('{') {
            return String::new();
        }
        for prefix in ["CLSID", r"Wow6432Node\CLSID"] {
            let value = self.default_server(h, &join(&[root, prefix, clsid, "InprocServer32"]));
            if !value.is_empty() {
                return value;
            }
        }
        String::new()
    }
    pub(crate) fn declarative(&mut self) {
        for target in &self.check.targets {
            if target.scope == "HKLM" || target.scope == "BOTH" {
                let (hive_name, key_path) =
                    target.path.split_once('\\').unwrap_or((&target.path, ""));
                if let Some(h) = self.machine(hive_name) {
                    let key_path = key_path.replace("{controlset}", &self.ctx.active_controlset());
                    for view in machine_views(hive_name, &key_path, target.include_wow64) {
                        self.target(
                            h,
                            &view,
                            &join(&["HKLM", hive_name, &view]),
                            target,
                            "SYSTEM",
                        );
                    }
                }
            }
            if target.scope == "HKU" || target.scope == "BOTH" {
                for h in self.users("NTUSER.DAT") {
                    for view in user_views(&target.path, target.include_wow64) {
                        self.target(h, &view, &join(&["HKU", &h.user, &view]), target, "USER");
                    }
                }
                for h in self.users("USRCLASS.DAT") {
                    for view in user_views(&target.path, target.include_wow64) {
                        if let Some(classes) = strip_prefix_case(&view, r"Software\Classes") {
                            self.target(
                                h,
                                classes,
                                &join(&["HKU", &h.user, &view]),
                                target,
                                "USER",
                            );
                        }
                    }
                }
            }
        }
    }
    fn target(
        &mut self,
        h: &'a HiveInput,
        path: &str,
        display: &str,
        target: &RegistryTarget,
        access: &str,
    ) {
        let Some(key) = self.key(h, path) else {
            return;
        };
        if target.recurse {
            for child in self.children(key) {
                if let Some(text) = self.text(child, &target.values) {
                    self.emit(
                        h,
                        child,
                        join(&[display, &child.name(), &target.values]),
                        text,
                        access,
                    );
                }
            }
        } else {
            let pairs = if target.values == "*" {
                self.values(key)
            } else {
                self.value(key, &target.values)
                    .map(|v| (target.values.clone(), v))
                    .into_iter()
                    .collect()
            };
            for (name, data) in pairs {
                for value in data.strings() {
                    if value.trim().trim_matches('"').is_empty() {
                        continue;
                    }
                    let values = if ["winlogon_userinit", "winlogon_appsetup", "winlogon_system"]
                        .contains(&self.check.id.as_str())
                    {
                        value
                            .split(',')
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(str::to_owned)
                            .collect()
                    } else {
                        vec![value]
                    };
                    for value in values {
                        self.emit(
                            h,
                            key,
                            join(&[display, if name == "(Default)" { "" } else { &name }]),
                            value,
                            access,
                        );
                    }
                }
            }
        }
    }
    fn finish(mut self) -> (Vec<Finding>, Coverage) {
        for kind in &self.check.hives {
            if !self
                .ctx
                .hives
                .iter()
                .any(|h| h.kind.eq_ignore_ascii_case(kind))
            {
                self.issue(format!(
                    "{kind} hive not collected; its registry view could not be assessed"
                ));
            }
        }
        let mut seen = BTreeSet::new();
        self.findings
            .retain(|f| seen.insert((f.path.to_lowercase(), f.value.clone())));
        let (status, detail) = if self.inputs.is_empty() {
            (
                "not_collected",
                "No applicable registry hives were collected".to_string(),
            )
        } else if self.issues.is_empty() {
            (
                "complete",
                format!(
                    "Read {} applicable registry hive(s); configuration evidence does not prove execution",
                    self.inputs.len()
                ),
            )
        } else {
            ("partial", self.issues.join("; "))
        };
        let mut c = Coverage::new(&self.check.id, status, detail);
        c.computer = self.ctx.computer.clone();
        (self.findings, c)
    }
}

pub(crate) fn join(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .map(|p| p.trim_matches('\\'))
        .collect::<Vec<_>>()
        .join("\\")
}
pub(crate) fn strip_prefix_case<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    if path.eq_ignore_ascii_case(prefix) {
        Some("")
    } else if path.len() > prefix.len()
        && path.get(..prefix.len())?.eq_ignore_ascii_case(prefix)
        && path.as_bytes().get(prefix.len()) == Some(&b'\\')
    {
        Some(&path[prefix.len() + 1..])
    } else {
        None
    }
}
fn machine_views(hive: &str, path: &str, wow: bool) -> Vec<String> {
    let mut views = vec![path.to_string()];
    if wow && hive.eq_ignore_ascii_case("SOFTWARE") {
        views.push(if let Some(tail) = strip_prefix_case(path, "Classes") {
            join(&["Classes", "Wow6432Node", tail])
        } else {
            join(&["Wow6432Node", path])
        });
    }
    views
}
fn user_views(path: &str, wow: bool) -> Vec<String> {
    let mut views = vec![path.to_string()];
    if wow && let Some(tail) = strip_prefix_case(path, r"Software\Classes") {
        views.push(join(&[r"Software\Classes", "Wow6432Node", tail]));
    }
    views
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_catalogue() {
        let c = catalogue();
        assert_eq!(c.len(), 131);
        assert_eq!(c.iter().map(|c| &c.id).collect::<BTreeSet<_>>().len(), 131);
        assert_eq!(c.iter().map(|c| c.builtin_allow.len()).sum::<usize>(), 115);
        assert!(
            c.iter()
                .all(|c| !c.technique.is_empty() && !c.mitre_id.is_empty())
        );
    }
    #[test]
    fn views_preserve_class_store_layout() {
        assert_eq!(
            machine_views("SOFTWARE", r"Classes\CLSID", true),
            vec![r"Classes\CLSID", r"Classes\Wow6432Node\CLSID"]
        );
        assert_eq!(
            machine_views("SOFTWARE", r"Microsoft\Foo", true),
            vec![r"Microsoft\Foo", r"Wow6432Node\Microsoft\Foo"]
        );
        assert_eq!(
            user_views(r"Software\Classes\CLSID", true),
            vec![
                r"Software\Classes\CLSID",
                r"Software\Classes\Wow6432Node\CLSID"
            ]
        );
        assert_eq!(
            user_views(r"Software\Microsoft", true),
            vec![r"Software\Microsoft"]
        );
    }
    #[test]
    fn prefix_boundaries() {
        assert_eq!(
            strip_prefix_case(r"software\classes\x", r"Software\Classes"),
            Some("x")
        );
        assert_eq!(strip_prefix_case("ClassesUnexpected", "Classes"), None);
    }
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod registry_tests;
