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

//! Route one SRUM parse to its investigative categories; never duplicate or coerce evidence rows.
use super::{Task, json_files, remove_generated_path};
use crate::cli::AutoOutputType;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};

pub(super) fn category(table: &str) -> &'static str {
    match table {
        "NetworkConnections" | "NetworkUsages" | "NetworkBytes" | "PushNotifications" => {
            "NetworkActivity"
        }
        "AppResourceUseInfo" | "AppTimelineProvider" | "ProcessorTime" => "ProcessExecution",
        _ => "SystemInformation",
    }
}

fn valid_path(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    let parts: Vec<_> = relative.components().collect();
    parts.len() == 2
        && parts.iter().all(|p| matches!(p, Component::Normal(_)))
        && matches!(
            parts[0].as_os_str().to_str(),
            Some("NetworkActivity" | "ProcessExecution" | "SystemInformation")
        )
        && path
            .file_name()
            .is_some_and(|p| p.to_string_lossy().starts_with("SRUM-"))
        && path
            .extension()
            .is_some_and(|p| matches!(p.to_str(), Some("csv" | "json" | "jsonl")))
}

/// Generated files from a previous run are owned by that manifest, not by a wildcard pattern.
pub(super) fn clear_previous(root: &Path, tasks: &[Task]) -> io::Result<()> {
    let current: HashSet<_> = tasks
        .iter()
        .filter(|t| t.artifact == "srum" && !t.timeline_only)
        .map(|t| t.computer.as_str())
        .collect();
    let path = root.join("manifest.json");
    let old: Value = match File::open(path) {
        Ok(file) => serde_json::from_reader(BufReader::new(file))?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };
    if let Some(tasks) = old.get("Tasks").and_then(Value::as_array) {
        for task in tasks
            .iter()
            .filter(|t| t.get("Artifact").and_then(Value::as_str) == Some("srum"))
        {
            if !task
                .get("Comp")
                .or_else(|| task.get("Computer"))
                .and_then(Value::as_str)
                .is_some_and(|name| current.contains(name))
            {
                continue;
            }
            let Some(hosts) = old.get("Hosts").and_then(Value::as_array) else {
                continue;
            };
            let Some(host) = hosts.iter().find(|h| {
                crate::output_schema::field(h, "Computer")
                    == crate::output_schema::field(task, "Computer")
            }) else {
                continue;
            };
            let Some(folder) = host.get("OutputDirectory").and_then(Value::as_str) else {
                continue;
            };
            if Path::new(folder).components().count() != 1
                || !matches!(
                    Path::new(folder).components().next(),
                    Some(Component::Normal(_))
                )
            {
                continue;
            }
            let host_root = root.join(folder);
            if let Some(files) = task.get("OutputFiles").and_then(Value::as_array) {
                for file in files.iter().filter_map(Value::as_str).map(PathBuf::from) {
                    if !valid_path(&host_root, &file) {
                        return Err(io::Error::other(
                            "Previous SRUM manifest contains a path outside its categories",
                        ));
                    }
                    ensure_category(&host_root, file.parent().unwrap())?;
                    remove_generated_path(&file, root)?;
                }
            }
            for category in ["ProcessExecution", "SystemInformation"] {
                let log = host_root.join(category).join("SRUM-logs.txt");
                if task
                    .get("Log")
                    .and_then(Value::as_str)
                    .is_some_and(|s| Path::new(s) == log)
                {
                    ensure_category(&host_root, &host_root.join(category))?;
                    remove_generated_path(&log, root)?;
                }
            }
            // The old layout is an artifact-owned directory, never a whole category.
            for category in ["ProcessExecution", "SystemInformation"] {
                let legacy = host_root.join(category).join("SRUM");
                if task
                    .get("Output")
                    .and_then(Value::as_str)
                    .is_some_and(|s| Path::new(s) == legacy)
                {
                    ensure_category(&host_root, &host_root.join(category))?;
                    remove_generated_path(&legacy, root)?;
                }
            }
        }
    }
    Ok(())
}

fn ensure_category(root: &Path, directory: &Path) -> io::Result<()> {
    if fs::symlink_metadata(root)?.file_type().is_symlink()
        || fs::symlink_metadata(directory)?.file_type().is_symlink()
    {
        return Err(io::Error::other(
            "SRUM output directories must not be symbolic links",
        ));
    }
    Ok(())
}

fn output_path(
    root: &Path,
    table: &str,
    source: usize,
    multiple: bool,
    extension: &str,
) -> io::Result<PathBuf> {
    let dir = root.join(category(table));
    ensure_category(root, &dir)?;
    let suffix = if multiple {
        format!("-Source{}", source + 1)
    } else {
        String::new()
    };
    let path = dir.join(format!("SRUM-{table}{suffix}.{extension}"));
    if fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink() || !m.is_file()) {
        return Err(io::Error::other(format!(
            "Unsafe SRUM output target {}",
            path.display()
        )));
    }
    Ok(path)
}

pub(super) fn distribute(task: &Task) -> io::Result<Vec<PathBuf>> {
    let root = task
        .output
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| io::Error::other("Invalid SRUM staging path"))?;
    let files = if task.output_type == AutoOutputType::Csv {
        crate::util::find_files(&task.output, |name| name.ends_with(".csv"))
    } else {
        json_files(&task.output, task.output_type)
    };
    let mut outputs = BTreeSet::new();
    let mut transfers = BTreeSet::new();
    if task.output_type == AutoOutputType::Csv {
        let parents: BTreeSet<_> = files
            .iter()
            .filter_map(|p| p.parent().map(Path::to_path_buf))
            .collect();
        let indexes: BTreeMap<_, _> = parents
            .iter()
            .enumerate()
            .map(|(i, p)| (p.clone(), i))
            .collect();
        for file in files {
            let stem = file
                .file_stem()
                .and_then(|p| p.to_str())
                .and_then(|s| s.strip_prefix("SRUM-"))
                .ok_or_else(|| io::Error::other("Unexpected SRUM CSV name"))?;
            let index = indexes[file.parent().unwrap()];
            let destination = output_path(root, stem, index, parents.len() > 1, "csv")?;
            if !outputs.insert(destination.clone()) {
                return Err(io::Error::other("SRUM output filename collision"));
            }
            transfers.insert((file.clone(), destination.clone()));
            fs::rename(file, destination)?;
        }
    } else {
        let multiple = files.len() > 1;
        for (source, file) in files.iter().enumerate() {
            let mut names = HashMap::<String, String>::new();
            let mut used = HashSet::new();
            let mut writers = BTreeMap::<PathBuf, BufWriter<File>>::new();
            let mut stages = BTreeMap::<PathBuf, PathBuf>::new();
            let mut count = 0;
            for record in serde_json::Deserializer::from_reader(BufReader::new(File::open(file)?))
                .into_iter::<Value>()
            {
                let mut value = record?;
                let table = value
                    .get("Table")
                    .and_then(Value::as_str)
                    .ok_or_else(|| io::Error::other("SRUM JSON row has no Table"))?;
                let kind = srum::Kind::of_table(table);
                let provider = kind.map(|k| k.output_name()).unwrap_or(table);
                let provider = if provider == "vfuprov" {
                    "VfuProvider"
                } else {
                    provider
                };
                let key = format!("{}:{provider}", kind.is_some());
                let filename = names
                    .entry(key)
                    .or_insert_with(|| crate::srum::table_filename(kind, table, &mut used));
                let destination =
                    output_path(root, filename, source, multiple, task.output_type.as_str())?;
                // Bound open file handles even for a database with many unknown providers.
                if !writers.contains_key(&destination) {
                    if writers.len() >= 32
                        && let Some((_, mut writer)) = writers.pop_first()
                    {
                        writer.flush()?;
                    }
                    let fresh = !stages.contains_key(&destination);
                    if fresh {
                        transfers.insert((file.clone(), destination.clone()));
                    }
                    let temporary = if let Some(path) = stages.get(&destination) {
                        path.clone()
                    } else {
                        super::unused_sibling(&destination, "srum")?
                    };
                    let handle = OpenOptions::new()
                        .write(true)
                        .create_new(fresh)
                        .append(!fresh)
                        .open(&temporary)?;
                    stages.insert(destination.clone(), temporary);
                    writers.insert(destination.clone(), BufWriter::new(handle));
                }
                super::annotate_value(&mut value, &task.computer, task.artifact);
                let writer = writers.get_mut(&destination).unwrap();
                if task.output_type == AutoOutputType::Json {
                    serde_json::to_writer_pretty(&mut *writer, &value)?;
                } else {
                    serde_json::to_writer(&mut *writer, &value)?;
                }
                writer.write_all(b"\n")?;
                count += 1;
            }
            for (_, mut writer) in writers {
                writer.flush()?;
            }
            for (destination, temporary) in stages {
                fs::rename(temporary, &destination)?;
                outputs.insert(destination);
            }
            if count == 0 {
                let destination =
                    output_path(root, "Empty", source, multiple, task.output_type.as_str())?;
                transfers.insert((file.clone(), destination.clone()));
                let temporary = super::unused_sibling(&destination, "srum")?;
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&temporary)?;
                fs::rename(temporary, &destination)?;
                outputs.insert(destination);
            }
        }
    }
    let mut log = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&task.log)?;
    for (source, destination) in transfers {
        writeln!(
            log,
            "SRUM table: {} -> {}",
            source.display(),
            destination.display()
        )?;
    }
    log.flush()?;
    fs::remove_dir_all(&task.output)?;
    Ok(outputs.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn setup(format: AutoOutputType) -> (tempfile::TempDir, Task, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("HOST1");
        for category in ["NetworkActivity", "ProcessExecution", "SystemInformation"] {
            fs::create_dir_all(root.join(category)).unwrap();
        }
        let output = root.join("SystemInformation/SRUM");
        fs::create_dir(&output).unwrap();
        let task = Task {
            computer: "HOST1".into(),
            artifact: "srum",
            output_type: format,
            timeline_only: false,
            input_bytes: 0,
            args: Vec::new(),
            output,
            log: root.join("SystemInformation/SRUM-logs.txt"),
        };
        (temp, task, root)
    }

    #[test]
    fn csv_sources_are_flat_distinct_and_not_reencoded() {
        let (_temp, task, root) = setup(AutoOutputType::Csv);
        for capture in ["capture-a", "capture-b"] {
            let source = task.output.join(capture);
            fs::create_dir(&source).unwrap();
            fs::write(
                source.join("SRUM-NetworkConnections.csv"),
                b"ID,Note\r\n1,\"x,y\"\r\n",
            )
            .unwrap();
            fs::write(source.join("SRUM-AppTimelineProvider.csv"), b"ID\n2\n").unwrap();
            fs::write(source.join("SRUM-IdentityMap.csv"), b"ID\n3\n").unwrap();
        }
        let files = distribute(&task).unwrap();
        assert_eq!(files.len(), 6);
        assert_eq!(
            fs::read(root.join("NetworkActivity/SRUM-NetworkConnections-Source2.csv")).unwrap(),
            b"ID,Note\r\n1,\"x,y\"\r\n"
        );
        assert!(
            root.join("ProcessExecution/SRUM-AppTimelineProvider-Source1.csv")
                .is_file()
        );
        assert!(
            root.join("SystemInformation/SRUM-IdentityMap-Source2.csv")
                .is_file()
        );
        assert!(!task.output.exists());
    }

    #[test]
    fn json_routes_known_and_unknown_providers_preserving_native_values() {
        for format in [AutoOutputType::Json, AutoOutputType::Jsonl] {
            let (_temp, task, root) = setup(format);
            let records = [
                json!({"Table":srum::Kind::NetworkUsage.table(),"Provider":"NetworkUsages","SourceFile":"a","BytesSent":u64::MAX,"Empty":null}),
                json!({"Table":srum::Kind::ApplicationUsage.table(),"Provider":"AppResourceUseInfo","SourceFile":"a","Usage":[1,2]}),
                json!({"Table":"NetworkUsages","Provider":"NetworkUsages","SourceFile":"a","Unknown":true}),
            ];
            for source in ["a", "b"] {
                let mut writer =
                    File::create(task.output.join(format!("{source}.{}", format.as_str())))
                        .unwrap();
                for value in &records {
                    serde_json::to_writer(&mut writer, value).unwrap();
                    writeln!(writer).unwrap();
                }
            }
            let files = distribute(&task).unwrap();
            assert_eq!(files.len(), 6);
            for suffix in [1, 2] {
                let path = root.join(format!(
                    "NetworkActivity/SRUM-NetworkUsages-Source{suffix}.{}",
                    format.as_str()
                ));
                let values: Vec<Value> =
                    serde_json::Deserializer::from_reader(File::open(path).unwrap())
                        .into_iter()
                        .map(Result::unwrap)
                        .collect();
                let mut expected = records[0].clone();
                super::super::annotate_value(&mut expected, "HOST1", "srum");
                assert_eq!(values, vec![expected]);
                assert!(
                    root.join(format!(
                        "SystemInformation/SRUM-Unknown-NetworkUsages-Source{suffix}.{}",
                        format.as_str()
                    ))
                    .is_file()
                );
            }
        }
    }

    #[test]
    fn failed_json_split_keeps_previous_outputs_and_source() {
        let (_temp, task, root) = setup(AutoOutputType::Jsonl);
        let out = root.join("NetworkActivity/SRUM-NetworkUsages.jsonl");
        fs::write(&out, b"previous output\n").unwrap();
        let source = task.output.join("source.jsonl");
        fs::write(
            &source,
            format!(
                "{}\ninvalid\n",
                json!({"Table":srum::Kind::NetworkUsage.table()})
            ),
        )
        .unwrap();
        assert!(distribute(&task).is_err());
        assert_eq!(fs::read(out).unwrap(), b"previous output\n");
        assert!(source.is_file());
    }

    #[test]
    fn empty_json_database_does_not_fabricate_timeline_records() {
        for format in [AutoOutputType::Json, AutoOutputType::Jsonl] {
            let (_temp, task, root) = setup(format);
            fs::write(task.output.join(format!("empty.{}", format.as_str())), b"").unwrap();
            let files = distribute(&task).unwrap();
            assert_eq!(
                files,
                vec![root.join(format!("SystemInformation/SRUM-Empty.{}", format.as_str()))]
            );
            assert_eq!(fs::metadata(&files[0]).unwrap().len(), 0);
            assert!(!task.output.exists());
        }
    }

    #[test]
    fn output_hard_link_does_not_modify_its_original() {
        let (temp, task, root) = setup(AutoOutputType::Jsonl);
        let evidence = temp.path().join("evidence");
        fs::write(&evidence, b"original evidence").unwrap();
        fs::hard_link(
            &evidence,
            root.join("NetworkActivity/SRUM-NetworkUsages.jsonl"),
        )
        .unwrap();
        fs::write(
            task.output.join("source.jsonl"),
            format!("{}\n", json!({"Table":srum::Kind::NetworkUsage.table()})),
        )
        .unwrap();
        distribute(&task).unwrap();
        assert_eq!(fs::read(evidence).unwrap(), b"original evidence");
    }

    #[test]
    fn manifest_cleanup_removes_only_recorded_category_outputs() {
        let (temp, task, root) = setup(AutoOutputType::Csv);
        let generated = root.join("NetworkActivity/SRUM-NetworkUsages.csv");
        let unrelated = root.join("NetworkActivity/SRUM-personal-notes.csv");
        fs::write(&generated, b"old").unwrap();
        fs::write(&unrelated, b"keep").unwrap();
        let manifest = json!({"Hosts":[{"Computer":"HOST1","OutputDirectory":"HOST1"}],
            "Tasks":[{"Computer":"HOST1","Artifact":"srum","OutputFiles":[generated]}]});
        fs::write(
            temp.path().join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        clear_previous(temp.path(), std::slice::from_ref(&task)).unwrap();
        assert!(!generated.exists());
        assert_eq!(fs::read(&unrelated).unwrap(), b"keep");
        let mut unsafe_manifest = manifest;
        unsafe_manifest["Tasks"][0]["OutputFiles"] =
            json!([unrelated.parent().unwrap().join("../outside")]);
        fs::write(
            temp.path().join("manifest.json"),
            serde_json::to_vec(&unsafe_manifest).unwrap(),
        )
        .unwrap();
        assert!(clear_previous(temp.path(), std::slice::from_ref(&task)).is_err());
        assert!(unrelated.exists());
    }

    #[test]
    fn manifest_cleanup_preserves_computers_not_in_this_run() {
        let (temp, task, _root) = setup(AutoOutputType::Csv);
        let other = temp.path().join("OTHER/NetworkActivity");
        fs::create_dir_all(&other).unwrap();
        let generated = other.join("SRUM-NetworkUsages.csv");
        fs::write(&generated, b"keep other host\n").unwrap();
        let manifest = json!({"Hosts":[{"Computer":"OTHER","OutputDirectory":"OTHER"}],
            "Tasks":[{"Computer":"OTHER","Artifact":"srum","OutputFiles":[generated]}]});
        fs::write(
            temp.path().join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        clear_previous(temp.path(), std::slice::from_ref(&task)).unwrap();
        assert_eq!(fs::read(generated).unwrap(), b"keep other host\n");
    }

    #[test]
    #[cfg(unix)]
    fn category_symlinks_are_rejected() {
        let (temp, task, root) = setup(AutoOutputType::Csv);
        let network = root.join("NetworkActivity");
        fs::remove_dir(&network).unwrap();
        let outside = temp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, &network).unwrap();
        fs::write(task.output.join("SRUM-NetworkUsages.csv"), b"ID\n1\n").unwrap();
        assert!(distribute(&task).is_err());
        assert_eq!(fs::read_dir(outside).unwrap().count(), 0);
    }
}
