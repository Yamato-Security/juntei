// Copyright (C) 2026 Shirofune Security Co., Ltd.
//
// Unless explicitly acquired and licensed from Licensor under another license, the contents of this
// file are subject to the Reciprocal Public License ("RPL") Version 1.5, or subsequent versions as
// allowed by the RPL, and You may not copy or use this file in either source code or executable
// form, except in compliance with the terms and conditions of the RPL.
//
// All software distributed under the RPL is provided strictly on an "AS IS" basis, WITHOUT WARRANTY
// OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR HEREBY DISCLAIMS ALL SUCH WARRANTIES,
// INCLUDING WITHOUT LIMITATION, ANY WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE, QUIET ENJOYMENT, OR NON-INFRINGEMENT. See the RPL for specific language governing rights
// and limitations under the RPL.

//! Export references separately from canonical paths used for reading and overwrite protection.
use super::model::{Diagnostic, Event, fingerprint};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct References {
    parsed: BTreeMap<PathBuf, String>,
    evidence: Vec<String>,
}

fn absolute(value: &str) -> bool {
    value.starts_with('/')
        || value.starts_with("\\\\")
        || (value.as_bytes().get(1) == Some(&b':')
            && matches!(value.as_bytes().get(2), Some(b'/' | b'\\')))
}

fn normalized(value: &str) -> String {
    let value = value.replace('\\', "/");
    // Windows canonicalization adds a verbatim prefix; manifests/native exports can instead
    // retain the ordinary drive or UNC spelling of the same evidence directory.
    let value = if value
        .get(..8)
        .is_some_and(|p| p.eq_ignore_ascii_case("//?/UNC/"))
    {
        format!("//{}", &value[8..])
    } else if let Some(drive) = value.strip_prefix("//?/")
        && drive.as_bytes().get(1) == Some(&b':')
    {
        drive.to_string()
    } else {
        value
    };
    value.trim_end_matches('/').to_string()
}

fn external(kind: &str, value: &str) -> String {
    let normalized = normalized(value);
    let leaf = normalized.rsplit('/').next().unwrap_or("source");
    format!("./{kind}/{}/{leaf}", &fingerprint(&[&normalized])[..32])
}

impl References {
    pub fn new(inputs: &[PathBuf], parsed_root: Option<&Path>, evidence: &[String]) -> Self {
        let mut result = Self::default();
        for input in inputs {
            let relative = parsed_root.and_then(|root| input.strip_prefix(root).ok());
            let display = relative.map_or_else(
                || {
                    format!(
                        "./{}",
                        input.file_name().unwrap_or_default().to_string_lossy()
                    )
                },
                |path| format!("./{}", path.to_string_lossy().replace('\\', "/")),
            );
            result.parsed.insert(input.clone(), display);
        }
        let mut seen = BTreeSet::new();
        let collisions: BTreeSet<_> = result
            .parsed
            .values()
            .filter(|v| !seen.insert(*v))
            .cloned()
            .collect();
        for (path, display) in &mut result.parsed {
            if collisions.contains(display) {
                *display = external("Inputs", &path.to_string_lossy());
            }
        }
        result.evidence = evidence
            .iter()
            .filter(|p| absolute(p))
            .map(|p| normalized(p))
            .collect();
        result.evidence.sort_by_key(|p| std::cmp::Reverse(p.len()));
        result.evidence.dedup();
        result
    }

    pub fn parsed(&self, path: &Path) -> String {
        self.parsed.get(path).cloned().unwrap_or_else(|| {
            let value = path.to_string_lossy();
            if absolute(&value) {
                external("Inputs", &value)
            } else {
                value.into_owned()
            }
        })
    }

    pub fn source(&self, value: &str) -> String {
        if let Some((image, within)) = value.split_once("!/") {
            return format!("{}!/{within}", self.source(image));
        }
        if !absolute(value) {
            return value.into();
        }
        if let Some(path) = self.parsed.get(Path::new(value)) {
            return path.clone();
        }
        let path = normalized(value);
        for root in &self.evidence {
            let prefix = format!("{root}/");
            // Windows drive and UNC paths are case-insensitive even when read on Linux/macOS.
            let windows = root.as_bytes().get(1) == Some(&b':') || root.starts_with("//");
            let matches = if windows {
                path.get(..prefix.len())
                    .is_some_and(|p| p.eq_ignore_ascii_case(&prefix))
            } else {
                path.starts_with(&prefix)
            };
            if matches {
                return format!("./{}", &path[prefix.len()..]);
            }
        }
        external("Sources", &path)
    }

    pub fn event(&self, event: &mut Event) {
        event.source_file = self.source(&event.source_file);
        if let Some(value) = event.details.get_mut("ParsedFile")
            && let Some(path) = value.as_str()
        {
            *value = Value::String(self.parsed(Path::new(path)));
        }
        if let Some(value) = event.details.get_mut("Image")
            && let Some(path) = value.as_str()
        {
            *value = Value::String(self.source(path));
        }
        // Other fields are evidence, including absolute Windows process/target paths. They are
        // deliberately left intact rather than recursively rewriting captured event details.
    }

    pub fn diagnostic(&self, diagnostic: &mut Diagnostic) {
        diagnostic.parsed_file = self.parsed(Path::new(&diagnostic.parsed_file));
        diagnostic.source_file = self.source(&diagnostic.source_file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_references_are_relative_and_do_not_collapse_distinct_captures() {
        let refs = References::new(
            &[],
            None,
            &["/cases/Evidence".into(), r"C:\Cases\Evidence".into()],
        );
        assert_eq!(
            refs.source("/cases/Evidence/HOST/C/SRUDB.dat"),
            "./HOST/C/SRUDB.dat"
        );
        assert_eq!(
            refs.source(r"c:\cases\evidence\HOST\Security.evtx"),
            "./HOST/Security.evtx"
        );
        assert_eq!(
            refs.source(r"\\?\C:\Cases\Evidence\HOST\Security.evtx"),
            "./HOST/Security.evtx"
        );
        let unc = References::new(&[], None, &[r"\\?\UNC\server\share\Evidence".into()]);
        assert_eq!(
            unc.source(r"\\server\share\Evidence\HOST\Security.evtx"),
            "./HOST/Security.evtx"
        );
        assert_eq!(
            refs.source("./disk.E01!/Volume1/Windows/Logs/Security.evtx"),
            "./disk.E01!/Volume1/Windows/Logs/Security.evtx"
        );
        let first = refs.source("/analyst/case1/Security.evtx");
        let second = refs.source("/analyst/case2/Security.evtx");
        assert!(first.starts_with("./Sources/"));
        assert!(!first.contains("analyst"));
        assert_ne!(first, second);
        assert_eq!(
            refs.source("/cases/Evidence/disk.E01!/Volume1/Windows/Logs/Security.evtx"),
            "./disk.E01!/Volume1/Windows/Logs/Security.evtx"
        );
        let first = PathBuf::from("/parsed/one/records.jsonl");
        let second = PathBuf::from("/parsed/two/records.jsonl");
        let inputs = References::new(&[first.clone(), second.clone()], None, &[]);
        assert_ne!(inputs.parsed(&first), inputs.parsed(&second));
        assert!(inputs.parsed(&first).starts_with("./Inputs/"));
    }
}
