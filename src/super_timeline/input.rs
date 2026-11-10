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

use super::model::{Context, ImageSource, get, text};
use crate::cli::SuperTimelineOptions;
use serde_json::{Map, Value};
use std::cell::Cell;
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
pub enum Format {
    Csv,
    Json,
    Jsonl,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub path: PathBuf,
    pub artifact: String,
    pub computer: String,
    pub raw_source: Option<String>,
    pub image: Option<ImageSource>,
    pub format: Format,
    pub index: usize,
}

pub struct Inventory {
    pub inputs: Vec<Input>,
    pub protected: Vec<PathBuf>,
    pub evidence_roots: Vec<PathBuf>,
    pub references: super::paths::References,
}

fn format(path: &Path, hint: &str) -> io::Result<Format> {
    let name = if hint.is_empty() {
        path.extension().and_then(|s| s.to_str()).unwrap_or("")
    } else {
        hint
    };
    match name.to_ascii_lowercase().as_str() {
        "csv" => Ok(Format::Csv),
        "json" => Ok(Format::Json),
        "jsonl" | "ndjson" => Ok(Format::Jsonl),
        _ => Err(io::Error::other(format!(
            "Unsupported parsed input format: {}",
            path.display()
        ))),
    }
}

fn contained(root: &Path, recorded: &Path, value: &str) -> io::Result<PathBuf> {
    let path = Path::new(value);
    let relative = if path.is_absolute() {
        path.strip_prefix(root)
            .or_else(|_| path.strip_prefix(recorded))
            .map_err(|_| {
                io::Error::other("Manifest output path is outside its recorded output directory")
            })?
    } else {
        path
    };
    if relative
        .components()
        .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(io::Error::other("Invalid relative path in parsed manifest"));
    }
    let result = fs::canonicalize(root.join(relative))?;
    if !result.starts_with(root) {
        return Err(io::Error::other(
            "Parsed input symlink points outside the output directory",
        ));
    }
    if !result.is_file() {
        return Err(io::Error::other("Expected a parsed data file"));
    }
    Ok(result)
}

fn child(parent: &Path, name: &str) -> Option<PathBuf> {
    fs::read_dir(parent)
        .ok()?
        .filter_map(Result::ok)
        .find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(name))
        .map(|e| e.path())
}

/// A CSV provider table has no SourceFile. Resolve only a unique existing database; multiple
/// captures cannot safely be assigned by a SourceN filename without a capture/source mapping.
fn unique_srum(host: &Value, evidence: &[PathBuf]) -> Option<String> {
    let mut sources = BTreeSet::new();
    for windows in get(host, &["WindowsDirectories"])?.as_array()? {
        let windows = PathBuf::from(windows.as_str()?);
        let locations = if windows.is_absolute() {
            vec![windows]
        } else {
            evidence.iter().map(|root| root.join(&windows)).collect()
        };
        for windows in locations {
            if let Some(path) = child(&windows, "System32")
                .and_then(|p| child(&p, "sru"))
                .and_then(|p| child(&p, "SRUDB.dat"))
                .filter(|p| p.is_file())
            {
                sources.insert(fs::canonicalize(path).ok()?);
            }
        }
    }
    (sources.len() == 1).then(|| sources.first().unwrap().to_string_lossy().into_owned())
}

/// Current portable manifests record directory names and evidence-relative host paths. Resolve
/// only existing, structurally matching evidence beside an ancestor (or in the current working
/// directory); never recursively search for a hive/database or choose between two captures.
fn evidence_locations(root: &Path, recorded: &str, hosts: &[Value]) -> Vec<PathBuf> {
    let mut locations = BTreeSet::new();
    let recorded = Path::new(recorded);
    let candidates = std::iter::once(recorded.to_path_buf()).chain(
        root.ancestors()
            .take(10)
            .map(|parent| parent.join(recorded)),
    );
    for path in candidates {
        let Ok(path) = path.canonicalize() else {
            continue;
        };
        if !path.is_dir() {
            continue;
        }
        if hosts.iter().any(|host| {
            let image = text(host, &["Image"]);
            if !image.is_empty() {
                return path.join(image).is_file();
            }
            let within = text(host, &["EvidenceRoot"]);
            !within.is_empty() && path.join(within).is_dir()
        }) {
            locations.insert(path);
        }
    }
    locations.into_iter().collect()
}

pub fn discover(opts: &SuperTimelineOptions) -> io::Result<Inventory> {
    let mut result = Inventory {
        inputs: Vec::new(),
        protected: Vec::new(),
        evidence_roots: Vec::new(),
        references: super::paths::References::default(),
    };
    let mut parsed_root = None;
    let mut reference_roots = Vec::new();
    if let Some(root) = &opts.directory {
        let logical_root = std::path::absolute(root)?;
        let root = fs::canonicalize(root)?;
        parsed_root = Some(root.clone());
        let manifest = root.join("manifest.json");
        let size = fs::metadata(&manifest)
            .map_err(|e| {
                io::Error::other(format!(
                    "{}: {e}; -d needs auto-parse output containing manifest.json",
                    manifest.display()
                ))
            })?
            .len();
        if size > 32 * 1024 * 1024 {
            return Err(io::Error::other("Parsed manifest is larger than 32 MiB"));
        }
        let value: Value = serde_json::from_reader(BufReader::new(File::open(&manifest)?))?;
        result.protected.push(manifest);
        let recorded = PathBuf::from(text(&value, &["OutputDirectory"]));
        if recorded.as_os_str().is_empty() {
            return Err(io::Error::other("Manifest has no OutputDirectory"));
        }
        if let Some(path) = get(&value, &["EvidenceDirectory"]).and_then(Value::as_str) {
            result.evidence_roots.push(PathBuf::from(path));
            reference_roots.push(path.to_string());
        }
        let hosts = get(&value, &["Hosts"])
            .and_then(Value::as_array)
            .ok_or_else(|| io::Error::other("Manifest has no Hosts array"))?;
        let evidence = evidence_locations(&root, &text(&value, &["EvidenceDirectory"]), hosts);
        // More than one physical candidate is ambiguous. Keep separate source identifiers
        // instead of giving distinct captures the same evidence-relative name.
        if evidence.len() == 1 {
            reference_roots.extend(evidence.iter().map(|p| p.to_string_lossy().into_owned()));
            // Preserve verified spellings beside the directory supplied by the caller too. On
            // macOS, for example, canonical /private/var paths can have /var source references.
            // Accept an alias only when it resolves to a verified evidence directory.
            let recorded_evidence = text(&value, &["EvidenceDirectory"]);
            if !recorded_evidence.is_empty() {
                for parent in logical_root.ancestors().take(10) {
                    let alias = parent.join(&recorded_evidence);
                    if alias
                        .canonicalize()
                        .is_ok_and(|path| evidence.contains(&path))
                    {
                        reference_roots.push(alias.to_string_lossy().into_owned());
                    }
                }
            }
        }
        result.evidence_roots.extend(evidence.iter().cloned());
        let tasks = get(&value, &["Tasks"])
            .and_then(Value::as_array)
            .ok_or_else(|| io::Error::other("Manifest has no Tasks array"))?;
        for task in tasks {
            if text(task, &["Status"]) != "succeeded" || text(task, &["Purpose"]) == "json_timeline"
            {
                continue;
            }
            let artifact = text(task, &["Artifact"]);
            let computer = text(task, &["Computer"]);
            if artifact.is_empty() || computer.is_empty() {
                return Err(io::Error::other(
                    "Manifest task has no artifact/computer identity",
                ));
            }
            let host = hosts.iter().find(|h| text(h, &["Computer"]) == computer);
            if let Some(host) = host
                && let Some(path) = get(host, &["EvidenceRoot"]).and_then(Value::as_str)
            {
                result.evidence_roots.push(PathBuf::from(path));
            }
            let image = host.and_then(|host| {
                let image = text(host, &["Image"]);
                let root = text(host, &["EvidenceRoot"]).replace('\\', "/");
                let staged_root = root.split('/').find(|p| !p.is_empty() && *p != ".")?;
                (!image.is_empty()).then(|| ImageSource {
                    image,
                    staged_root: staged_root.into(),
                })
            });
            let raw = get(task, &["Command"])
                .and_then(Value::as_array)
                .and_then(|command| {
                    command
                        .windows(2)
                        .find(|p| matches!(p[0].as_str(), Some("-f" | "--file")))
                        .and_then(|p| p[1].as_str())
                        .map(str::to_owned)
                })
                .or_else(|| {
                    (artifact == "srum")
                        .then(|| host.and_then(|host| unique_srum(host, &evidence)))
                        .flatten()
                });
            let hint = text(task, &["OutputType"]);
            let files: Vec<String> = get(task, &["OutputFiles"])
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default();
            let files = if files.is_empty() {
                let output = text(task, &["Output"]);
                if output.is_empty() {
                    return Err(io::Error::other("Succeeded manifest task has no output"));
                }
                let path = Path::new(&output);
                // Legacy multi-table artifact directories are permitted; categories/host roots
                // must carry OutputFiles and are never scanned as if they were one artifact.
                let relative = if path.is_absolute() {
                    path.strip_prefix(&root)
                        .or_else(|_| path.strip_prefix(&recorded))
                        .map_err(|_| {
                            io::Error::other(
                                "Manifest output path is outside its recorded output directory",
                            )
                        })?
                } else {
                    path
                };
                if relative
                    .components()
                    .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
                {
                    return Err(io::Error::other("Invalid relative path in parsed manifest"));
                }
                let local = fs::canonicalize(root.join(relative))?;
                if !local.starts_with(&root) {
                    return Err(io::Error::other(
                        "Parsed input symlink points outside the output directory",
                    ));
                }
                if local.is_dir() {
                    let leaf = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if !matches!(
                        leaf,
                        "Amcache"
                            | "Registry-Amcache"
                            | "JumpLists"
                            | "SRUM"
                            | "amcache"
                            | "jump-lists"
                            | "srum"
                    ) {
                        return Err(io::Error::other(
                            "Manifest category output requires explicit OutputFiles",
                        ));
                    }
                    crate::util::find_files(&local, |s| {
                        s.ends_with(".csv") || s.ends_with(".json") || s.ends_with(".jsonl")
                    })
                    .iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect()
                } else {
                    vec![output]
                }
            } else {
                files
            };
            for path in files {
                let path = contained(&root, &recorded, &path)?;
                let kind = format(&path, &hint)?;
                result.inputs.push(Input {
                    path,
                    artifact: artifact.clone(),
                    computer: computer.clone(),
                    raw_source: raw.clone(),
                    image: image.clone(),
                    format: kind,
                    index: 0,
                });
            }
        }
    }
    if let Some(path) = &opts.filepath {
        let path = fs::canonicalize(path)?;
        let kind = format(&path, "")?;
        if matches!(kind, Format::Csv) && opts.input_artifact.is_none() {
            return Err(io::Error::other(
                "Standalone CSV needs --input-artifact, for example --input-artifact mft",
            ));
        }
        result.inputs.push(Input {
            path,
            artifact: opts.input_artifact.clone().unwrap_or_default(),
            computer: opts.computer.clone().unwrap_or_default(),
            raw_source: None,
            image: None,
            format: kind,
            index: 0,
        });
    }
    for path in &opts.hayabusa {
        result.inputs.push(Input {
            path: fs::canonicalize(path)?,
            artifact: "hayabusa".into(),
            computer: opts.computer.clone().unwrap_or_default(),
            raw_source: None,
            image: None,
            format: Format::Jsonl,
            index: 0,
        });
    }
    result.inputs.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.artifact.cmp(&b.artifact))
            .then(a.computer.cmp(&b.computer))
    });
    result
        .inputs
        .dedup_by(|a, b| a.path == b.path && a.artifact == b.artifact && a.computer == b.computer);
    for pair in result.inputs.windows(2) {
        if pair[0].path == pair[1].path {
            return Err(io::Error::other(
                "One parsed file has conflicting artifact/computer mappings",
            ));
        }
    }
    for (index, input) in result.inputs.iter_mut().enumerate() {
        input.index = index;
        result.protected.push(input.path.clone());
        if let Some(raw) = &input.raw_source {
            result.protected.push(PathBuf::from(raw));
        }
    }
    if result.inputs.is_empty() {
        return Err(io::Error::other("No succeeded artifact inputs were found"));
    }
    result.references = super::paths::References::new(
        &result
            .inputs
            .iter()
            .map(|input| input.path.clone())
            .collect::<Vec<_>>(),
        parsed_root.as_deref(),
        &reference_roots,
    );
    Ok(result)
}

struct Limited<R> {
    inner: R,
    used: Rc<Cell<usize>>,
    limit: usize,
}
impl<R: Read> Read for Limited<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        // Read::bytes (used by serde_json) retries Interrupted indefinitely. Cancellation is
        // persistent, so use a non-retryable read error; the command still returns exit 130.
        crate::interrupt::check().map_err(|_| io::Error::other("Cancelled"))?;
        let left = self.limit.saturating_sub(self.used.get());
        if left == 0 {
            return Err(io::Error::other("input record exceeds --max-record-size"));
        }
        let length = bytes.len().min(left);
        let read = self.inner.read(&mut bytes[..length])?;
        self.used.set(self.used.get() + read);
        Ok(read)
    }
}

fn context(input: &Input, record: u64) -> Context {
    Context {
        parsed_file: input.path.clone(),
        artifact: input.artifact.clone(),
        computer: input.computer.clone(),
        raw_source: input.raw_source.clone(),
        image: input.image.clone(),
        input_index: input.index,
        record,
        location: String::new(),
    }
}

fn dispatch(
    value: Value,
    ctx: &Context,
    sink: &mut impl FnMut(&Value, &Context) -> io::Result<()>,
) -> io::Result<()> {
    crate::interrupt::check()?;
    if let Value::Array(values) = value {
        for (index, value) in values.into_iter().enumerate() {
            let mut child = ctx.clone();
            child.location = format!("{}[{index}]", ctx.location);
            dispatch(value, &child, sink)?;
        }
        return Ok(());
    }
    if !value.is_object() {
        return Err(io::Error::other(
            "Native parsed records must be JSON objects",
        ));
    }
    if value.get("SourceField").is_some()
        && ((value.get("TimestampType").is_some() && value.get("Message").is_some())
            || (value.get("timestamp_desc").is_some() && value.get("message").is_some()))
    {
        return Err(io::Error::other(
            "Input is already a normalized super-timeline",
        ));
    }
    let mut ctx = ctx.clone();
    if ctx.artifact != "hayabusa" {
        let artifact = text(&value, &["Artifact"]);
        if !artifact.is_empty() {
            if !ctx.artifact.is_empty() && ctx.artifact != artifact {
                return Err(io::Error::other(
                    "Embedded artifact conflicts with the input artifact mapping",
                ));
            }
            ctx.artifact = artifact;
        }
    }
    let computer = text(&value, &["Computer"]);
    if !computer.is_empty() {
        ctx.computer = computer;
    }
    if ctx.computer.is_empty() {
        ctx.computer = text(&value, &["Host"]);
    }
    if ctx.computer.is_empty() {
        ctx.computer = "Unknown".into();
    }
    // The pretty Amcache report contains record arrays; native JSONL is already flattened.
    if ctx.artifact == "amcache" && value.get("Summary").is_some() && value.get("Files").is_some() {
        for (name, kind) in [
            ("Summary", "summary"),
            ("Files", "file"),
            ("Devices", "device"),
            ("Findings", "finding"),
        ] {
            if let Some(group) = value.get(name) {
                let records: Vec<&Value> = if name == "Summary" {
                    vec![group]
                } else {
                    group
                        .as_array()
                        .map(|a| a.iter().collect())
                        .unwrap_or_default()
                };
                for (index, record) in records.into_iter().enumerate() {
                    let mut record = record.clone();
                    let object = record
                        .as_object_mut()
                        .ok_or_else(|| io::Error::other("Amcache group member is not an object"))?;
                    object.insert("RecordType".into(), Value::String(kind.into()));
                    if let Some(source) = value.get("Source") {
                        object.insert("Source".into(), source.clone());
                    }
                    let mut child = ctx.clone();
                    child.location = format!("{name}[{index}]");
                    sink(&record, &child)?;
                }
            }
        }
        return Ok(());
    }
    sink(&value, &ctx)
}

pub fn records(
    input: &Input,
    limit: usize,
    mut sink: impl FnMut(&Value, &Context) -> io::Result<()>,
) -> io::Result<()> {
    crate::interrupt::check()?;
    let mut file = BufReader::new(File::open(&input.path)?);
    if file.fill_buf()?.starts_with(&[0xef, 0xbb, 0xbf]) {
        file.consume(3);
    }
    match input.format {
        Format::Jsonl => {
            let mut index = 0;
            loop {
                crate::interrupt::check()?;
                let mut line = Vec::new();
                loop {
                    crate::interrupt::check()?;
                    let buffer = file.fill_buf()?;
                    if buffer.is_empty() {
                        break;
                    }
                    let end = buffer
                        .iter()
                        .position(|b| *b == b'\n')
                        .map(|i| i + 1)
                        .unwrap_or(buffer.len());
                    if line.len().saturating_add(end) > limit {
                        return Err(io::Error::other(format!(
                            "line {} exceeds --max-record-size",
                            index + 1
                        )));
                    }
                    line.extend_from_slice(&buffer[..end]);
                    file.consume(end);
                    if line.last() == Some(&b'\n') {
                        break;
                    }
                }
                if line.is_empty() {
                    break;
                }
                index += 1;
                if line.iter().all(u8::is_ascii_whitespace) {
                    continue;
                }
                let value = serde_json::from_slice(&line)
                    .map_err(|e| io::Error::other(format!("JSONL line {index}: {e}")))?;
                dispatch(value, &context(input, index), &mut sink)?;
            }
        }
        Format::Json => {
            let used = Rc::new(Cell::new(0));
            let limited = Limited {
                inner: file,
                used: used.clone(),
                limit,
            };
            let mut records = serde_json::Deserializer::from_reader(limited).into_iter::<Value>();
            let mut index = 0;
            loop {
                used.set(0);
                let Some(record) = records.next() else {
                    break;
                };
                index += 1;
                dispatch(record?, &context(input, index), &mut sink)?;
            }
        }
        Format::Csv => {
            let used = Rc::new(Cell::new(0));
            let limited = Limited {
                inner: file,
                used: used.clone(),
                limit,
            };
            let mut csv = csv::Reader::from_reader(limited);
            let headers = csv.headers()?.clone();
            let unique: BTreeSet<_> = headers.iter().collect();
            if unique.len() != headers.len() {
                return Err(io::Error::other("CSV has duplicate field names"));
            }
            let mut record = csv::StringRecord::new();
            let mut index = 0;
            loop {
                used.set(0);
                if !csv.read_record(&mut record)? {
                    break;
                }
                index += 1;
                let row: Map<String, Value> = headers
                    .iter()
                    .zip(record.iter())
                    .filter(|(name, _)| {
                        !matches!(
                            *name,
                            "DataRaw"
                                | "ValueDataRaw"
                                | "RegistryValuesJSON"
                                | "ZoneID-Contents"
                                | "ResidentDataBase64"
                                | "ResidentDataHex"
                        )
                    })
                    .map(|(name, value)| (name.into(), Value::String(value.into())))
                    .collect();
                dispatch(Value::Object(row), &context(input, index), &mut sink)?;
            }
        }
    }
    Ok(())
}
