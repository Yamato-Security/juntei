//! Additional SIEM export. Retain native types and provenance, and commit only complete JSONL.

use super::unused_sibling;
use super::{DataFile, Host, WORK_DIR_PREFIX, absolute_candidate, output_separate_from_evidence};
use crate::cli::AutoOutputType;
use crate::util;
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

/// Validate before creating output directories, clearing previous outputs, or starting a parser.
pub(super) fn validate(
    path: &Path,
    evidence: &Path,
    output: &Path,
    hosts: &[Host],
    clobber: bool,
) -> Result<PathBuf, String> {
    if path.as_os_str().is_empty() {
        return Err("--json-timeline requires a filename, for example -j timeline.jsonl".into());
    }
    output_separate_from_evidence(evidence, path)?;
    for host in hosts {
        output_separate_from_evidence(&host.root, path)?;
    }
    let target = absolute_candidate(path)
        .map_err(|e| format!("Could not resolve JSONL timeline {}: {e}", path.display()))?;
    for evidence_root in
        std::iter::once(evidence).chain(hosts.iter().map(|host| host.root.as_path()))
    {
        if relative_to_directory(&target, evidence_root)?.is_some() {
            return Err("The JSONL timeline must be outside the evidence directories.".into());
        }
    }
    let root = absolute_candidate(output).map_err(|e| {
        format!(
            "Could not resolve output directory {}: {e}",
            output.display()
        )
    })?;
    if root.starts_with(&target) {
        return Err("The JSONL timeline must be a file separate from the output directory.".into());
    }
    if let Some(relative) = relative_to_directory(&target, &root)? {
        let first = relative
            .components()
            .next()
            .ok_or("The JSONL timeline must be a file separate from the output directory.")?
            .as_os_str()
            .to_string_lossy();
        if first.eq_ignore_ascii_case("manifest.json")
            || first.eq_ignore_ascii_case("manifest.csv")
            || first.to_ascii_lowercase().starts_with(WORK_DIR_PREFIX)
            || hosts
                .iter()
                .any(|host| first.eq_ignore_ascii_case(&host.folder))
        {
            return Err("The JSONL timeline must not overlap manifests, computer results, or temporary parser outputs. Use -j <OUTPUT>/timeline.jsonl or a file outside <OUTPUT>.".into());
        }
    }
    check_destination(&target, clobber)?;
    Ok(target)
}

/// `realpath` need not normalize filename case on every filesystem. Compare directory identity
/// for aliases as well as textual prefixes, including Windows short names and macOS case aliases.
fn relative_to_directory(path: &Path, directory: &Path) -> Result<Option<PathBuf>, String> {
    if let Ok(relative) = path.strip_prefix(directory) {
        return Ok(Some(relative.to_path_buf()));
    }
    match fs::metadata(directory) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Could not inspect {}: {e}", directory.display())),
        Ok(_) => {}
    }
    for ancestor in path.ancestors() {
        match fs::metadata(ancestor) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("Could not inspect {}: {e}", ancestor.display())),
            Ok(metadata) if !metadata.is_dir() => continue,
            Ok(_) => {}
        }
        if same_file::is_same_file(directory, ancestor)
            .map_err(|e| format!("Could not compare output directory paths: {e}"))?
        {
            return Ok(Some(path.strip_prefix(ancestor).unwrap().to_path_buf()));
        }
    }
    Ok(None)
}

pub(super) fn write(files: &[DataFile], output: &Path, clobber: bool) -> io::Result<()> {
    // Recheck after parsing, before opening anything for writing. Never truncate an existing
    // destination or follow a hard link into evidence; write a sibling and then replace the name.
    check_destination(output, clobber).map_err(io::Error::other)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = unused_sibling(output, "timeline")?;
    let result = (|| -> io::Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        let mut writer = BufWriter::new(file);
        let mut ordered: Vec<_> = files.iter().collect();
        ordered.sort_by_key(|file| &file.path);
        for data in ordered {
            let mut input = BufReader::new(File::open(&data.path)?);
            if data.output_type == AutoOutputType::Jsonl {
                // These inputs have already been validated and annotated one record per line.
                io::copy(&mut input, &mut writer)?;
            } else {
                for value in serde_json::Deserializer::from_reader(input).into_iter::<Value>() {
                    let value = value.map_err(io::Error::other)?;
                    if data.artifact == "amcache" {
                        amcache_records(value, &mut writer)?;
                    } else {
                        line(&value, &mut writer)?;
                    }
                }
            }
        }
        writer.flush()?;
        drop(writer);
        check_destination(output, clobber).map_err(io::Error::other)?;
        fs::rename(&temporary, output)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn check_destination(output: &Path, clobber: bool) -> Result<(), String> {
    util::check_output(output, false, clobber)?;
    if !clobber && fs::symlink_metadata(output).is_ok() {
        return Err(format!(
            "The JSONL timeline already exists: {}. Use -C to overwrite.",
            output.display()
        ));
    }
    Ok(())
}

fn line(value: &Value, writer: &mut impl Write) -> io::Result<()> {
    serde_json::to_writer(&mut *writer, value).map_err(io::Error::other)?;
    writer.write_all(b"\n")
}

/// Amcache's pretty JSON is a report containing record arrays; its native JSONL flattens those
/// arrays into individual summary/finding/file/device envelopes. Preserve that SIEM schema in
/// JSON mode too, without reparsing the source hive.
fn amcache_records(value: Value, writer: &mut impl Write) -> io::Result<()> {
    let Value::Object(mut common) = value else {
        return Err(io::Error::other("Amcache JSON report is not an object"));
    };
    let mut groups = Vec::new();
    for (field, kind) in [
        ("summary", "summary"),
        ("findings", "finding"),
        ("files", "file"),
        ("devices", "device"),
    ] {
        let value = common
            .remove(field)
            .ok_or_else(|| io::Error::other(format!("Amcache JSON report is missing {field}")))?;
        let records = if field == "summary" {
            vec![value]
        } else if let Value::Array(records) = value {
            records
        } else {
            return Err(io::Error::other(format!(
                "Amcache JSON {field} is not an array"
            )));
        };
        groups.push((kind, records));
    }
    for (kind, records) in groups {
        for record in records {
            let Value::Object(fields) = record else {
                return Err(io::Error::other("Amcache JSON record is not an object"));
            };
            let mut envelope = common.clone();
            envelope.insert("record_type".into(), kind.into());
            envelope.extend(fields);
            line(&Value::Object(envelope), writer)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_timeline_survives_a_failed_export_and_requires_clobber() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source.json");
        let output = root.path().join("timeline.jsonl");
        fs::write(&source, "{\"value\":1}\nbroken").unwrap();
        fs::write(&output, b"previous results\n").unwrap();
        let files = [DataFile {
            path: source.clone(),
            output_type: AutoOutputType::Json,
            artifact: "registry",
        }];
        assert!(write(&files, &output, false).is_err());
        assert!(write(&files, &output, true).is_err());
        assert_eq!(fs::read(&output).unwrap(), b"previous results\n");
        assert_eq!(
            fs::read_to_string(&source).unwrap(),
            "{\"value\":1}\nbroken"
        );
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
        fs::write(source, "{\"value\":1}\n").unwrap();
        write(&files, &output, true).unwrap();
        assert_eq!(fs::read_to_string(output).unwrap(), "{\"value\":1}\n");
    }

    #[cfg(unix)]
    #[test]
    fn replacing_a_hard_link_does_not_change_evidence() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("evidence.json");
        let output = root.path().join("timeline.jsonl");
        let original = "{\n  \"value\": 7\n}\n";
        fs::write(&source, original).unwrap();
        fs::hard_link(&source, &output).unwrap();
        write(
            &[DataFile {
                path: source.clone(),
                output_type: AutoOutputType::Json,
                artifact: "registry",
            }],
            &output,
            true,
        )
        .unwrap();
        assert_eq!(fs::read_to_string(source).unwrap(), original);
        assert_eq!(fs::read_to_string(output).unwrap(), "{\"value\":7}\n");
    }
}
