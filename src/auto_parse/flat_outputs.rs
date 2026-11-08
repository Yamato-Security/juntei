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

//! Flatten multi-table CSV exports without changing their bytes or discarding source identity.
use super::{Task, artifact_category, artifact_filename, remove_generated_path};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, Write};
use std::path::{Component, Path, PathBuf};

fn check_category(root: &Path, directory: &Path) -> io::Result<()> {
    for path in [root, directory] {
        if fs::symlink_metadata(path)?.file_type().is_symlink() {
            return Err(io::Error::other(
                "Output directories must not be symbolic links",
            ));
        }
    }
    Ok(())
}

fn table_name<'a>(artifact: &str, name: &'a str) -> Option<&'a str> {
    match (artifact, name) {
        ("amcache", "Amcache_devices.csv") => Some("Devices"),
        ("amcache", "Amcache_files.csv") => Some("Files"),
        ("amcache", "Amcache_findings.csv") => Some("Findings"),
        ("amcache", "Amcache_summary.csv") => Some("Summary"),
        ("jump-lists", "AutomaticDestinations.csv") => Some("AutomaticDestinations"),
        ("jump-lists", "CustomDestinations.csv") => Some("CustomDestinations"),
        ("browser", "Browser-History.csv") => Some("History"),
        ("browser", "Browser-Downloads.csv") => Some("Downloads"),
        ("browser", "Browser-Cookies.csv") => Some("Cookies"),
        ("browser", "Browser-FormHistory.csv") => Some("FormHistory"),
        // `parse-ese ual` names each table's file `UAL-<table>.csv`, a table it doesn't know
        // under its own name, in letters, digits and `-_{}` only
        ("ual", name) => name
            .strip_prefix("UAL-")
            .and_then(|table| table.strip_suffix(".csv"))
            .filter(|table| {
                !table.is_empty()
                    && table
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || "-_{}".contains(c))
            }),
        _ => None,
    }
}

/// The log names the tables from `output`, the output directory, as the manifests do.
pub(super) fn distribute(task: &Task, output: &Path) -> io::Result<Vec<PathBuf>> {
    let directory = task
        .output
        .parent()
        .ok_or_else(|| io::Error::other("Missing category"))?;
    let root = directory
        .parent()
        .ok_or_else(|| io::Error::other("Missing computer"))?;
    check_category(root, directory)?;
    let files = crate::util::find_files(&task.output, |name| name.ends_with(".csv"));
    if files.is_empty() {
        return Err(io::Error::other("Parser produced no CSV tables"));
    }
    let parents: BTreeSet<_> = files.iter().filter_map(|p| p.parent()).collect();
    let indexes: BTreeMap<_, _> = parents
        .iter()
        .enumerate()
        .map(|(i, p)| (*p, i + 1))
        .collect();
    let mut destinations = BTreeSet::new();
    let mut transfers = Vec::new();
    for source in &files {
        let table = source
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| table_name(task.artifact, name))
            .ok_or_else(|| {
                io::Error::other("Unexpected parser CSV table; staging output retained")
            })?;
        let suffix = if parents.len() > 1 {
            format!("-Source{}", indexes[source.parent().unwrap()])
        } else {
            String::new()
        };
        let destination = directory.join(format!(
            "{}-{table}{suffix}.csv",
            artifact_filename(task.artifact)
        ));
        if fs::symlink_metadata(&destination)
            .is_ok_and(|m| m.file_type().is_symlink() || !m.is_file())
        {
            return Err(io::Error::other("Unsafe CSV output target"));
        }
        if !destinations.insert(destination.clone()) {
            return Err(io::Error::other("CSV output filename collision"));
        }
        transfers.push((source.clone(), destination));
    }
    let mut log = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&task.log)?;
    for (source, destination) in transfers {
        fs::rename(&source, &destination)?;
        writeln!(
            log,
            "Table: {} -> {}",
            super::dot_relative(output, &source).display(),
            super::dot_relative(output, &destination).display()
        )?;
    }
    log.flush()?;
    fs::remove_dir_all(&task.output)?;
    Ok(destinations.into_iter().collect())
}

fn flat_table_path(host: &Path, path: &Path, artifact: &str) -> bool {
    let category = host.join(artifact_category(artifact));
    if path.parent() != Some(category.as_path())
        || path.extension().and_then(|s| s.to_str()) != Some("csv")
    {
        return false;
    }
    path.file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|name| name.starts_with(&format!("{}-", artifact_filename(artifact))))
}

/// Migrate only manifest-owned outputs, never a whole investigative category or a wildcard.
pub(super) fn clear_previous(root: &Path, tasks: &[Task]) -> io::Result<()> {
    let old: Value = match File::open(root.join("manifest.json")) {
        Ok(file) => serde_json::from_reader(BufReader::new(file))?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let from_output = super::paths_from_output(&old);
    let Some(previous) = old.get("Tasks").and_then(Value::as_array) else {
        return Ok(());
    };
    let Some(hosts) = old.get("Hosts").and_then(Value::as_array) else {
        return Ok(());
    };
    for task in tasks.iter().filter(|task| !task.timeline_only) {
        let stem = artifact_filename(task.artifact);
        if !matches!(task.artifact, "amcache" | "browser" | "jump-lists" | "ual")
            && !stem.starts_with("Registry-")
        {
            continue;
        }
        let Some(folder) = hosts
            .iter()
            .find(|h| {
                crate::output_schema::field(h, "Computer").and_then(Value::as_str)
                    == Some(task.computer.as_str())
            })
            .and_then(|h| h.get("OutputDirectory"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        if Path::new(folder).components().count() != 1
            || !matches!(
                Path::new(folder).components().next(),
                Some(Component::Normal(_))
            )
        {
            return Err(io::Error::other(
                "Invalid computer directory in previous manifest",
            ));
        }
        let host = root.join(folder);
        let category = host.join(artifact_category(task.artifact));
        check_category(&host, &category)?;
        let old_stem = if task.artifact == "registry" {
            "Registry"
        } else {
            stem.strip_prefix("Registry-").unwrap_or(stem)
        };
        for previous in previous.iter().filter(|p| {
            crate::output_schema::field(p, "Computer").and_then(Value::as_str)
                == Some(task.computer.as_str())
                && p.get("Artifact").and_then(Value::as_str) == Some(task.artifact)
        }) {
            if previous.get("Purpose").and_then(Value::as_str) != Some("json_timeline") {
                if let Some(files) = previous.get("OutputFiles").and_then(Value::as_array) {
                    for path in files
                        .iter()
                        .filter_map(Value::as_str)
                        .map(|path| super::recorded_path(root, path, from_output))
                    {
                        if !flat_table_path(&host, &path, task.artifact) {
                            return Err(io::Error::other(
                                "Previous table manifest contains an invalid output path",
                            ));
                        }
                        remove_generated_path(&path, root)?;
                    }
                }
                if let Some(path) = previous
                    .get("Output")
                    .and_then(Value::as_str)
                    .map(|path| super::recorded_path(root, path, from_output))
                {
                    for name in [old_stem, stem] {
                        if matches!(task.artifact, "amcache" | "browser" | "jump-lists" | "ual")
                            && path == category.join(name)
                        {
                            remove_generated_path(&path, root)?;
                        }
                        for extension in ["csv", "json", "jsonl"] {
                            if path == category.join(format!("{name}.{extension}")) {
                                remove_generated_path(&path, root)?;
                            }
                        }
                    }
                }
            }
            if let Some(path) = previous
                .get("Log")
                .and_then(Value::as_str)
                .map(|path| super::recorded_path(root, path, from_output))
            {
                for name in [old_stem, stem] {
                    for suffix in ["-logs.txt", "-json-timeline-logs.txt"] {
                        if path == category.join(format!("{name}{suffix}")) {
                            remove_generated_path(&path, root)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::AutoOutputType;
    use serde_json::json;

    fn setup(artifact: &'static str) -> (tempfile::TempDir, Task) {
        let temp = tempfile::tempdir().unwrap();
        let category = temp.path().join("HOST1").join(artifact_category(artifact));
        fs::create_dir_all(&category).unwrap();
        let output = category.join(artifact_filename(artifact));
        fs::create_dir(&output).unwrap();
        let task = Task {
            computer: "HOST1".into(),
            artifact,
            output_type: AutoOutputType::Csv,
            timeline_only: false,
            input_bytes: 0,
            args: Vec::new(),
            output,
            log: category.join(format!("{}-logs.txt", artifact_filename(artifact))),
        };
        (temp, task)
    }

    #[test]
    fn flattening_retains_all_tables_and_distinguishes_capture_sources() {
        for (artifact, names) in [
            (
                "amcache",
                vec![
                    "Amcache_files.csv",
                    "Amcache_devices.csv",
                    "Amcache_summary.csv",
                    "Amcache_findings.csv",
                ],
            ),
            (
                "jump-lists",
                vec!["AutomaticDestinations.csv", "CustomDestinations.csv"],
            ),
        ] {
            let (temp, task) = setup(artifact);
            for (index, capture) in ["a", "b"].iter().enumerate() {
                let source = task.output.join(capture);
                fs::create_dir(&source).unwrap();
                for name in &names {
                    fs::write(
                        source.join(name),
                        format!("Field,Note\r\n{index},\"a,b\"\r\n"),
                    )
                    .unwrap();
                }
            }
            let files = distribute(&task, temp.path()).unwrap();
            assert_eq!(files.len(), names.len() * 2);
            for source in [1, 2] {
                for name in &names {
                    let destination = task.output.parent().unwrap().join(format!(
                        "{}-{}-Source{source}.csv",
                        artifact_filename(artifact),
                        table_name(artifact, name).unwrap()
                    ));
                    assert_eq!(
                        fs::read(destination).unwrap(),
                        format!("Field,Note\r\n{},\"a,b\"\r\n", source - 1).as_bytes()
                    );
                }
            }
            assert!(!task.output.exists());
            let log = fs::read_to_string(&task.log).unwrap();
            assert!(log.contains("/a/") || log.contains("\\a\\"));
            assert!(log.contains("-Source2.csv"));
        }
    }

    #[test]
    fn unknown_tables_do_not_disappear_or_partially_replace_outputs() {
        let (temp, task) = setup("jump-lists");
        fs::write(task.output.join("AutomaticDestinations.csv"), b"new\n").unwrap();
        fs::write(task.output.join("Unexpected.csv"), b"retain\n").unwrap();
        let destination = task
            .output
            .parent()
            .unwrap()
            .join("JumpLists-AutomaticDestinations.csv");
        fs::write(&destination, b"previous\n").unwrap();
        assert!(distribute(&task, temp.path()).is_err());
        assert_eq!(fs::read(destination).unwrap(), b"previous\n");
        assert!(task.output.join("AutomaticDestinations.csv").is_file());
        assert!(task.output.join("Unexpected.csv").is_file());
    }

    #[test]
    fn cleanup_migrates_only_manifest_owned_legacy_outputs() {
        let (temp, task) = setup("amcache");
        let category = task.output.parent().unwrap().to_path_buf();
        let legacy = category.join("Amcache");
        fs::create_dir(&legacy).unwrap();
        fs::write(legacy.join("Files.csv"), b"old\n").unwrap();
        let notes = category.join("Amcache-notes.csv");
        fs::write(&notes, b"keep\n").unwrap();
        let manifest = json!({"Hosts":[{"Computer":"HOST1","OutputDirectory":"HOST1"}], "Tasks":[
            {"Computer":"HOST1","Artifact":"amcache","Purpose":"artifact","Output":legacy,"OutputFiles":[]}
        ]});
        fs::write(
            temp.path().join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        clear_previous(temp.path(), std::slice::from_ref(&task)).unwrap();
        assert!(!legacy.exists());
        assert_eq!(fs::read(notes).unwrap(), b"keep\n");
        fs::write(task.output.join("Amcache_files.csv"), b"same\n").unwrap();
        let files = distribute(&task, temp.path()).unwrap();
        let manifest = json!({"Hosts":[{"Computer":"HOST1","OutputDirectory":"HOST1"}], "Tasks":[
            {"Computer":"HOST1","Artifact":"amcache","Purpose":"artifact","Output":category,"OutputFiles":files}
        ]});
        fs::write(
            temp.path().join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        clear_previous(temp.path(), &[task]).unwrap();
        assert!(files.iter().all(|file| !file.exists()));
        assert!(category.is_dir());
    }

    #[test]
    fn cleanup_rejects_manifest_paths_outside_the_artifact_category() {
        let (temp, task) = setup("amcache");
        let outside = temp.path().join("keep.csv");
        fs::write(&outside, b"evidence\n").unwrap();
        let manifest = json!({"Hosts":[{"Computer":"HOST1","OutputDirectory":"HOST1"}], "Tasks":[
            {"Computer":"HOST1","Artifact":"amcache","Purpose":"artifact","OutputFiles":[outside]}
        ]});
        fs::write(
            temp.path().join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(clear_previous(temp.path(), &[task]).is_err());
        assert_eq!(fs::read(outside).unwrap(), b"evidence\n");
    }
}
