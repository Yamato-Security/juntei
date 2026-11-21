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

//! Opt-in, portable YARA scanning of collected files, never executing evidence.

#[path = "yara_mft.rs"]
mod mft_scan;

use crate::{cli::CommonOptions, interrupt, progress, util};
use anyhow::{Context, Result, ensure};
use boreal::{
    MetadataValue, Scanner,
    compiler::{CompilerBuilder, CompilerParams},
    scanner::ScanParams,
};
use clap::{ArgGroup, Args, ValueEnum};
use rayon::prelude::*;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

const MAX_ARCHIVE: u64 = 64 * 1024 * 1024;
const MAX_RULES: u64 = 128 * 1024 * 1024;
const ENGINE_VERSION: &str = "1.4.0";
const DETECTION_HEADERS: &[&str] = &[
    "Computer",
    "FilePath",
    "Size",
    "SHA256",
    "ScanStatus",
    "Rule",
    "Namespace",
    "Tags",
    "Metadata",
    "MatchLocations",
    "SourceType",
    "RecoveredFile",
];
const FILE_HEADERS: &[&str] = &[
    "Computer",
    "FilePath",
    "Size",
    "SHA256",
    "Status",
    "Matches",
    "ElapsedMs",
    "Note",
    "SourceType",
];

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum Ruleset {
    #[default]
    Core,
    Extended,
    Full,
}
impl Ruleset {
    fn name(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Extended => "extended",
            Self::Full => "full",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum OutputType {
    #[default]
    Csv,
    Jsonl,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("input").required(true).args(["directory", "file"])))]
pub struct Options {
    /// Recursively scan all regular files, including .quarantined files
    #[arg(short = 'd', long, value_name = "DIR")]
    pub directory: Option<PathBuf>,
    /// Scan one file
    #[arg(short = 'f', long, value_name = "FILE")]
    pub file: Option<PathBuf>,
    /// Create a new results directory outside the evidence
    #[arg(short = 'o', long, value_name = "DIR")]
    pub output: PathBuf,
    /// Local rule file or directory (.yar/.yara); no download
    #[arg(short='r', long, value_name="PATH", conflicts_with_all=["ruleset", "rules_release"])]
    pub rules: Option<PathBuf>,
    /// YARA Forge package to download when --rules is absent
    #[arg(long, value_enum, default_value = "core")]
    pub ruleset: Ruleset,
    /// YARA Forge release: latest or YYYYMMDD
    #[arg(long, default_value="latest", value_parser=parse_release)]
    pub rules_release: String,
    /// Results format
    #[arg(short = 't', long, value_enum, default_value = "csv")]
    pub output_type: OutputType,
    /// Parallel file scans (default: CPUs, capped at 8)
    #[arg(short='T', long, default_value_t=default_threads(), value_parser=clap::value_parser!(u16).range(1..=256))]
    pub threads: u16,
    /// Per-file scan timeout in seconds (does not include reading)
    #[arg(long, default_value_t=60, value_parser=clap::value_parser!(u64).range(1..=86400))]
    pub timeout: u64,
    /// Optional maximum file size in MiB (default: unlimited; includes MFTs)
    #[arg(long, value_name="MIB", value_parser=parse_size_limit)]
    pub max_file_mib: Option<u64>,
    /// Override the file-size cap for MFTs (default: inherit --max-file-mib)
    #[arg(long, value_name="MIB", value_parser=parse_size_limit)]
    pub max_mft_mib: Option<u64>,
    /// Maximum matched resident streams recovered per run
    #[arg(long, default_value_t=1000, value_parser=clap::value_parser!(u64).range(1..=100000))]
    pub max_mft_recoveries: u64,
    /// Evidence computer name (otherwise read nearby client_info.json; empty if unknown)
    #[arg(long)]
    pub computer: Option<String>,
    #[command(flatten)]
    pub common: CommonOptions,
}
fn default_threads() -> u16 {
    std::thread::available_parallelism().map_or(1, |n| n.get().min(8) as u16)
}
const MIB: u64 = 1024 * 1024;
fn parse_size_limit(s: &str) -> std::result::Result<u64, String> {
    s.parse::<u64>().ok().filter(|n| *n > 0 && n.checked_mul(MIB).is_some())
        .ok_or_else(|| "Use a positive whole number of MiB (example: --max-file-mib 1024) that fits in a u64 byte count".into())
}
fn size_limit_bytes(mib: Option<u64>) -> u64 {
    // CLI validation guarantees multiplication cannot overflow; None is an unlimited read.
    mib.map_or(u64::MAX, |value| value * MIB)
}
fn parse_release(s: &str) -> std::result::Result<String, String> {
    if s == "latest"
        || (s.len() == 8
            && s.bytes().all(|b| b.is_ascii_digit())
            && chrono::NaiveDate::parse_from_str(s, "%Y%m%d").is_ok())
    {
        Ok(s.into())
    } else {
        Err("Use latest or a release date in YYYYMMDD format (example: 20261115)".into())
    }
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
fn bounded_read(reader: impl Read, limit: u64) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "Size exceeds the configured limit"
    );
    Ok(bytes)
}
fn exclusive(path: &Path) -> Result<File> {
    Ok(OpenOptions::new().write(true).create_new(true).open(path)?)
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    exclusive(path)?.write_all(bytes)?;
    Ok(())
}

fn download_url(set: Ruleset, release: &str) -> String {
    let release_path = if release == "latest" {
        "latest/download".to_owned()
    } else {
        format!("download/{release}")
    };
    format!(
        "https://github.com/YARAHQ/yara-forge/releases/{release_path}/yara-forge-rules-{}.zip",
        set.name()
    )
}
fn unpack_rules(bytes: &[u8], set: Ruleset) -> Result<Vec<u8>> {
    ensure!(bytes.len() as u64 <= MAX_ARCHIVE, "Rule archive too large");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
    let mut member = archive.by_name(&format!("packages/{0}/yara-rules-{0}.yar", set.name()))?;
    ensure!(member.size() <= MAX_RULES, "Rule source too large");
    bounded_read(&mut member, MAX_RULES)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct RuleSource {
    file_path: String,
    #[serde(rename = "SHA256")]
    sha256: String,
    size: usize,
}
#[derive(Debug)]
struct Rules {
    source: String,
    sources: Vec<RuleSource>,
    paths: HashSet<PathBuf>,
    url: Option<String>,
    archive_sha256: Option<String>,
}

/// Resolve includes using YARA's parser, not a regex: comments and quoted include-like text
/// remain untouched. All local files must stay within the explicitly selected rules directory.
struct LocalRules {
    root: PathBuf,
    sources: BTreeMap<PathBuf, String>,
    includes: HashMap<PathBuf, Vec<(std::ops::Range<usize>, PathBuf)>>,
    provenance: BTreeMap<PathBuf, (String, usize)>,
    active: HashSet<PathBuf>,
    total: u64,
}
impl LocalRules {
    fn load(&mut self, path: &Path, depth: usize) -> Result<()> {
        ensure!(depth <= 32, "Rule include depth exceeds 32");
        let path = path.canonicalize()?;
        ensure!(
            path.starts_with(&self.root),
            "Rule include escapes the local rules directory"
        );
        ensure!(!self.active.contains(&path), "Circular rule include");
        if self.sources.contains_key(&path) {
            return Ok(());
        }
        ensure!(self.sources.len() < 10000, "Too many local rule files");
        let remaining = MAX_RULES.saturating_sub(self.total);
        let bytes = bounded_read(File::open(&path)?, remaining)?;
        self.total += bytes.len() as u64;
        self.provenance
            .insert(path.clone(), (hash(&bytes), bytes.len()));
        let text = String::from_utf8(bytes).context("Rules must be UTF-8")?;
        let text = text.strip_prefix('\u{feff}').unwrap_or(&text).to_owned();
        let parsed = boreal_parser::parse(&text).map_err(|e| {
            anyhow::anyhow!(
                "Invalid local rule source {}: {e:?}",
                relative(&self.root, &path)
            )
        })?;
        self.active.insert(path.clone());
        let mut includes = Vec::new();
        for component in parsed.components {
            if let boreal_parser::file::YaraFileComponent::Include(i) = component {
                let child = path
                    .parent()
                    .context("Rule has no parent")?
                    .join(&i.path)
                    .canonicalize()
                    .with_context(|| format!("Cannot resolve include {}", i.path))?;
                self.load(&child, depth + 1)?;
                includes.push((i.span, child));
            }
        }
        self.active.remove(&path);
        self.includes.insert(path.clone(), includes);
        self.sources.insert(path, text);
        Ok(())
    }
    fn expand(&self, path: &Path, seen: &mut HashSet<PathBuf>, out: &mut String) {
        if !seen.insert(path.to_owned()) {
            return;
        }
        let text = &self.sources[path];
        let mut offset = 0;
        for (span, child) in &self.includes[path] {
            out.push_str(&text[offset..span.start]);
            out.push('\n');
            self.expand(child, seen, out);
            out.push('\n');
            offset = span.end;
        }
        out.push_str(&text[offset..]);
        out.push('\n');
    }
}
fn local_rules(input: &Path) -> Result<Rules> {
    let input = input.canonicalize()?;
    let root = if input.is_dir() {
        input.clone()
    } else {
        input.parent().context("Rules have no parent")?.to_owned()
    };
    let mut candidates = Vec::new();
    if input.is_dir() {
        for e in walkdir::WalkDir::new(&input)
            .follow_links(false)
            .sort_by_file_name()
        {
            let e = e?;
            if e.file_type().is_file()
                && e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| {
                        s.eq_ignore_ascii_case("yar") || s.eq_ignore_ascii_case("yara")
                    })
            {
                candidates.push(e.path().canonicalize()?);
            }
        }
    } else {
        candidates.push(input);
    }
    ensure!(!candidates.is_empty(), "No .yar/.yara files found");
    let mut local = LocalRules {
        root,
        sources: BTreeMap::new(),
        includes: HashMap::new(),
        provenance: BTreeMap::new(),
        active: HashSet::new(),
        total: 0,
    };
    for path in &candidates {
        local.load(path, 0)?;
    }
    let included: HashSet<_> = local.includes.values().flatten().map(|(_, p)| p).collect();
    let mut seen = HashSet::new();
    let mut source = String::new();
    // Expand the entry points first so a helper discovered earlier alphabetically does not get
    // moved ahead of the rules/imports that precede its include directive.
    for path in candidates.iter().filter(|p| !included.contains(p)) {
        local.expand(path, &mut seen, &mut source);
    }
    ensure!(source.len() as u64 <= MAX_RULES, "Expanded rules too large");
    let sources = local
        .provenance
        .iter()
        .map(|(p, (sha, size))| RuleSource {
            file_path: relative(&local.root, p),
            sha256: sha.clone(),
            size: *size,
        })
        .collect();
    Ok(Rules {
        source,
        sources,
        paths: local.sources.into_keys().collect(),
        url: None,
        archive_sha256: None,
    })
}
fn load_rules(opts: &Options) -> Result<Rules> {
    if let Some(path) = &opts.rules {
        return local_rules(path);
    }
    let url = download_url(opts.ruleset, &opts.rules_release);
    if !opts.common.quiet {
        util::p_label(opts.common.no_color, "Downloading rules: ", &url);
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .timeout_global(Some(Duration::from_secs(120)))
        .build()
        .into();
    let mut response = agent
        .get(&url)
        .call()
        .context("Could not download YARA Forge rules; use --rules for offline scanning")?;
    let archive = bounded_read(response.body_mut().as_reader(), MAX_ARCHIVE)?;
    let source = String::from_utf8(unpack_rules(&archive, opts.ruleset)?)?;
    Ok(Rules {
        source,
        sources: vec![],
        paths: HashSet::new(),
        url: Some(url),
        archive_sha256: Some(hash(&archive)),
    })
}
fn compile(source: &str, timeout: u64) -> Result<(Scanner, Vec<String>)> {
    let mut compiler = CompilerBuilder::new()
        .add_module(boreal::module::Console::with_callback(|_| {}))
        .build();
    compiler.set_params(CompilerParams::default().disable_includes(true));
    let status = compiler
        .add_rules_str(source.strip_prefix('\u{feff}').unwrap_or(source))
        .map_err(|e| {
            anyhow::anyhow!("YARA compilation failed; no rules were silently dropped: {e}")
        })?;
    let warnings = status.warnings().map(ToString::to_string).collect();
    let mut scanner = compiler.finalize();
    ensure!(
        scanner.rules().any(|r| !r.is_private),
        "No public rules to scan"
    );
    scanner.set_scan_params(
        ScanParams::default()
            .compute_full_matches(true)
            .match_max_length(0)
            .timeout_duration(Some(Duration::from_secs(timeout))),
    );
    Ok((scanner, warnings))
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct FileResult {
    computer: String,
    file_path: String,
    size: u64,
    #[serde(rename = "SHA256")]
    sha256: String,
    status: String,
    matches: usize,
    elapsed_ms: u128,
    note: String,
    source_type: String,
}
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Detection {
    computer: String,
    file_path: String,
    size: u64,
    #[serde(rename = "SHA256")]
    sha256: String,
    scan_status: String,
    rule: String,
    namespace: String,
    tags: String,
    metadata: String,
    match_locations: String,
    source_type: String,
    recovered_file: String,
}
struct Candidate {
    path: PathBuf,
    relative: String,
    size: u64,
    pre_status: Option<&'static str>,
    note: String,
}
fn discover(root: &Path, input: &Path, paths: &HashSet<PathBuf>) -> Vec<Candidate> {
    walkdir::WalkDir::new(input)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(e) => {
                if e.file_type().is_dir() {
                    return None;
                }
                let status = if e.file_type().is_symlink() {
                    Some("SkippedSymlink")
                } else if !e.file_type().is_file() {
                    Some("SkippedSpecialFile")
                } else if paths.contains(e.path()) {
                    Some("SkippedRuleSource")
                } else {
                    None
                };
                Some(Candidate {
                    relative: relative(root, e.path()),
                    path: e.path().to_owned(),
                    size: e.metadata().map_or(0, |m| m.len()),
                    pre_status: status,
                    note: String::new(),
                })
            }
            Err(e) => Some(Candidate {
                relative: e.path().map_or_else(|| ".".into(), |p| relative(root, p)),
                path: e.path().unwrap_or(input).to_owned(),
                size: 0,
                pre_status: Some("DiscoveryError"),
                note: e
                    .io_error()
                    .map_or_else(|| "Directory traversal error".into(), ToString::to_string),
            }),
        })
        .collect()
}
fn computer_for(path: &Path, root: &Path, override_name: &Option<String>) -> String {
    if let Some(name) = override_name {
        return name.clone();
    }
    for dir in path
        .parent()
        .into_iter()
        .flat_map(Path::ancestors)
        .take_while(|d| d.starts_with(root))
    {
        let info = dir.join("client_info.json");
        if let Ok(f) = File::open(info)
            && let Ok(bytes) = bounded_read(f, 1024 * 1024)
            && let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes)
        {
            for key in ["hostname", "Hostname", "HostName", "Computer", "computer"] {
                if let Some(s) = v
                    .get(key)
                    .and_then(|x| x.as_str())
                    .filter(|s| !s.is_empty())
                {
                    return s.into();
                }
            }
        }
    }
    String::new()
}
#[derive(Debug)]
struct RawLocation {
    rule: String,
    offset: usize,
    length: usize,
}
struct ScanOutcome {
    file: FileResult,
    hits: Vec<Detection>,
    snapshot: Option<Vec<u8>>,
    locations: Vec<RawLocation>,
    locations_limited: bool,
}
#[cfg(test)]
fn scan(
    scanner: &Scanner,
    c: &Candidate,
    computer: String,
    limit: u64,
) -> (FileResult, Vec<Detection>) {
    let outcome = scan_file(scanner, c, computer, limit, false);
    (outcome.file, outcome.hits)
}
fn scan_file(
    scanner: &Scanner,
    c: &Candidate,
    computer: String,
    limit: u64,
    capture_mft: bool,
) -> ScanOutcome {
    let start = Instant::now();
    let mut row = FileResult {
        computer,
        file_path: c.relative.clone(),
        size: c.size,
        sha256: String::new(),
        status: String::new(),
        matches: 0,
        elapsed_ms: 0,
        note: c.note.clone(),
        source_type: if capture_mft { "MFTRaw" } else { "File" }.into(),
    };
    if let Some(status) = c.pre_status {
        row.status = status.into();
        return ScanOutcome {
            file: row,
            hits: vec![],
            snapshot: None,
            locations: vec![],
            locations_limited: false,
        };
    }
    if interrupt::requested() {
        row.status = "Cancelled".into();
        return ScanOutcome {
            file: row,
            hits: vec![],
            snapshot: None,
            locations: vec![],
            locations_limited: false,
        };
    }
    let data = (|| -> Result<Vec<u8>> {
        let meta = fs::symlink_metadata(&c.path)?;
        ensure!(
            meta.is_file() && !meta.file_type().is_symlink(),
            "Input is no longer a regular file"
        );
        ensure!(
            meta.len() <= limit,
            "File exceeds the configured scan size limit"
        );
        let f = File::open(&c.path)?;
        let before = f.metadata()?;
        ensure!(before.is_file(), "Input is no longer a regular file");
        let bytes = bounded_read(&f, limit)?;
        let after = f.metadata()?;
        ensure!(
            before.len() == after.len()
                && after.len() == bytes.len() as u64
                && before.modified().ok() == after.modified().ok(),
            "File changed while being read; retry from a stable copy"
        );
        Ok(bytes)
    })();
    let bytes = match data {
        Ok(b) => b,
        Err(e) => {
            row.status = if c.size > limit {
                "SkippedTooLarge"
            } else {
                "ReadError"
            }
            .into();
            row.note = format!("{e}");
            row.elapsed_ms = start.elapsed().as_millis();
            return ScanOutcome {
                file: row,
                hits: vec![],
                snapshot: None,
                locations: vec![],
                locations_limited: false,
            };
        }
    };
    row.size = bytes.len() as u64;
    row.sha256 = hash(&bytes);
    let (hits, locations, locations_limited) = evaluate(scanner, &bytes, &mut row, capture_mft);
    row.elapsed_ms = start.elapsed().as_millis();
    let snapshot = (capture_mft && !hits.is_empty()).then_some(bytes);
    ScanOutcome {
        file: row,
        hits,
        snapshot,
        locations,
        locations_limited,
    }
}

/// File and resident-stream evaluation use the same complete byte buffer and file semantics.
fn evaluate(
    scanner: &Scanner,
    bytes: &[u8],
    row: &mut FileResult,
    capture: bool,
) -> (Vec<Detection>, Vec<RawLocation>, bool) {
    let result = match scanner.scan_mem(bytes) {
        Ok(r) => {
            row.status = "Scanned".into();
            r
        }
        Err((e, r)) => {
            row.status = "ScanError".into();
            row.note = e.to_string();
            r
        }
    };
    const MAX_LOCATIONS: usize = 100_000;
    let mut locations = Vec::new();
    let mut limited = false;
    if capture {
        for rule in result.rules.iter().filter(|r| r.matched) {
            for pattern in &rule.matches {
                // Boreal stores at most 1000 occurrences per pattern by default. Reaching
                // that ceiling makes recovery coverage uncertain even if raw detection succeeded.
                limited |= pattern.matches.len() >= 1000;
                for matched in &pattern.matches {
                    if locations.len() == MAX_LOCATIONS {
                        limited = true;
                        break;
                    }
                    if let Some(offset) = matched.base.checked_add(matched.offset) {
                        locations.push(RawLocation {
                            rule: rule.name.into(),
                            offset,
                            length: matched.length,
                        });
                    } else {
                        limited = true;
                    }
                }
            }
        }
    }
    let detections = result.rules.into_iter().filter(|r| r.matched).map(|r| {
        let metadata: BTreeMap<_, _> = r.metadatas.iter().map(|m| (scanner.get_string_symbol(m.name), match m.value {
            MetadataValue::Bytes(s) => serde_json::Value::String(String::from_utf8_lossy(scanner.get_bytes_symbol(s)).into_owned()),
            MetadataValue::Integer(i) => i.into(), MetadataValue::Boolean(b) => b.into(),
        })).collect();
        let samples: Vec<_> = r.matches.iter().flat_map(|s| s.matches.iter().take(1).map(move |m| serde_json::json!({"Pattern": format!("${}", s.name), "Offset": m.offset, "Length": m.length, "XorKey": m.xor_key}))).take(16).collect();
        Detection { computer: row.computer.clone(), file_path: row.file_path.clone(), size: row.size, sha256: row.sha256.clone(), scan_status: row.status.clone(), rule: r.name.into(), namespace: r.namespace.into(), tags: r.tags.iter().map(|s| scanner.get_string_symbol(*s)).collect::<Vec<_>>().join(" "), metadata: serde_json::to_string(&metadata).expect("metadata serialization"), match_locations: serde_json::to_string(&samples).expect("location serialization"), source_type: row.source_type.clone(), recovered_file: String::new() }
    }).collect::<Vec<_>>();
    row.matches = detections.len();
    (detections, locations, limited)
}

enum Table {
    Csv(Box<csv::Writer<BufWriter<File>>>),
    Jsonl(BufWriter<File>),
}
impl Table {
    fn new(out: &Path, name: &str, kind: OutputType, headers: &[&str]) -> Result<Self> {
        Ok(match kind {
            OutputType::Csv => {
                let mut w = csv::WriterBuilder::new()
                    .has_headers(false)
                    .from_writer(BufWriter::new(exclusive(&out.join(format!("{name}.csv")))?));
                w.write_record(headers)?;
                Self::Csv(Box::new(w))
            }
            OutputType::Jsonl => Self::Jsonl(BufWriter::new(exclusive(
                &out.join(format!("{name}.jsonl")),
            )?)),
        })
    }
    fn row(&mut self, row: &impl Serialize) -> Result<()> {
        match self {
            Self::Csv(w) => w.serialize(row)?,
            Self::Jsonl(w) => {
                serde_json::to_writer(&mut *w, row)?;
                w.write_all(b"\n")?;
            }
        }
        Ok(())
    }
    fn flush(&mut self) -> Result<()> {
        match self {
            Self::Csv(w) => w.flush()?,
            Self::Jsonl(w) => w.flush()?,
        }
        Ok(())
    }
}
/// Canonicalize the existing ancestor as well, so symlinked output parents cannot evade checks.
fn proposed_output(path: &Path) -> Result<PathBuf> {
    let abs = std::path::absolute(path)?;
    let mut ancestor = abs.as_path();
    let mut rest = Vec::new();
    while !ancestor.exists() {
        rest.push(
            ancestor
                .file_name()
                .context("Invalid output path")?
                .to_owned(),
        );
        ancestor = ancestor
            .parent()
            .context("Output has no existing ancestor")?;
    }
    let mut resolved = ancestor.canonicalize()?;
    for part in rest.into_iter().rev() {
        ensure!(
            part != "..",
            "Output must not contain .. after a missing directory"
        );
        resolved.push(part);
    }
    Ok(resolved)
}
fn execute(opts: &Options) -> Result<i32> {
    interrupt::install()?;
    let input_arg = opts
        .directory
        .as_ref()
        .or(opts.file.as_ref())
        .context("Input required")?;
    ensure!(
        !fs::symlink_metadata(input_arg)?.file_type().is_symlink(),
        "Input must not be a symlink"
    );
    let input = input_arg.canonicalize()?;
    ensure!(
        if opts.directory.is_some() {
            input.is_dir()
        } else {
            input.is_file()
        },
        "Input type does not match -d/-f"
    );
    let root = if input.is_dir() {
        input.clone()
    } else {
        input.parent().context("Input has no parent")?.to_owned()
    };
    let output = proposed_output(&opts.output)?;
    ensure!(
        !output.exists(),
        "Output already exists; use a new results directory"
    );
    ensure!(
        !output.starts_with(&input) && !input.starts_with(&output),
        "Results must be outside the evidence directory"
    );
    if let Some(rules) = &opts.rules {
        let rules = rules.canonicalize()?;
        ensure!(
            !output.starts_with(&rules) && !rules.starts_with(&output),
            "Results must be separate from local rules"
        );
    }
    // Construct the pool before creating results, so invalid resource settings leave no partial directory.
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(opts.threads as usize)
        .build()?;
    fs::create_dir_all(&output)?;
    write_new(
        &output.join("RunStarted.json"),
        &serde_json::to_vec_pretty(
            &serde_json::json!({"Status":"Started", "StartedUTC":chrono::Utc::now().to_rfc3339(), "JunteiVersion":env!("CARGO_PKG_VERSION")}),
        )?,
    )?;
    let rules = load_rules(opts)?;
    write_new(&output.join("Rules.yar"), rules.source.as_bytes())?;
    if !opts.common.quiet {
        println!("Compiling rules. Please wait.");
    }
    let (scanner, warnings) = match compile(&rules.source, opts.timeout) {
        Ok(v) => v,
        Err(e) => {
            write_new(
                &output.join("CompilationError.txt"),
                e.to_string().as_bytes(),
            )?;
            return Err(e);
        }
    };
    write_new(
        &output.join("CompilationWarnings.txt"),
        warnings.join("\n").as_bytes(),
    )?;
    let candidates = discover(&root, &input, &rules.paths);
    let (mfts, ordinary): (Vec<_>, Vec<_>) = candidates.iter().partition(|c| mft_scan::is_mft(c));
    let mut mft_table = if mfts.is_empty() {
        None
    } else {
        Some(Table::new(
            &output,
            "YaraMFTResults",
            opts.output_type,
            mft_scan::HEADERS,
        )?)
    };
    let mut mft_stats = mft_scan::Stats::default();
    let mut statuses = BTreeMap::<String, usize>::new();
    let files = Table::new(&output, "YaraFiles", opts.output_type, FILE_HEADERS)?;
    let detections = Table::new(
        &output,
        "YaraDetections",
        opts.output_type,
        DETECTION_HEADERS,
    )?;
    let state = Mutex::new((files, detections, &mut statuses, 0usize, 0usize));
    if !opts.common.quiet {
        util::p_label(
            opts.common.no_color,
            "Rule source: ",
            if opts.rules.is_some() {
                "Local (offline)"
            } else {
                opts.ruleset.name()
            },
        );
        util::p_label(
            opts.common.no_color,
            "Compiled rules: ",
            &scanner.rules().count().to_string(),
        );
        util::p_label(
            opts.common.no_color,
            "Input files: ",
            &candidates.len().to_string(),
        );
        util::p_label(
            opts.common.no_color,
            "Parallel scans: ",
            &opts.threads.to_string(),
        );
        println!("\nScanning in progress. Please wait.\n");
    }
    let bar = (!opts.common.quiet)
        .then(|| progress::progress_bar(candidates.len(), opts.common.no_color));
    let scan_result: Result<()> = pool.install(|| {
        ordinary.par_iter().try_for_each(|c| {
            if let Some(bar) = &bar {
                bar.set_message(progress::message(&c.path));
            }
            let outcome = scan_file(
                &scanner,
                c,
                computer_for(&c.path, &root, &opts.computer),
                size_limit_bytes(opts.max_file_mib),
                false,
            );
            let file = outcome.file;
            let hits = outcome.hits;
            let mut s = state
                .lock()
                .map_err(|_| anyhow::anyhow!("Results writer failed"))?;
            s.0.row(&file)?;
            for hit in &hits {
                s.1.row(hit)?;
            }
            *s.2.entry(file.status.clone()).or_default() += 1;
            s.3 += hits.len();
            s.4 += usize::from(!hits.is_empty());
            if let Some(bar) = &bar {
                bar.inc(1);
            }
            Ok(())
        })
    });
    scan_result?;
    let (mut files, mut detections, _, mut matches, mut matching_files) = state
        .into_inner()
        .map_err(|_| anyhow::anyhow!("Results writer failed"))?;
    for c in mfts {
        if let Some(bar) = &bar {
            bar.set_message(progress::message(&c.path));
        }
        let outcome = scan_file(
            &scanner,
            c,
            computer_for(&c.path, &root, &opts.computer),
            size_limit_bytes(opts.max_mft_mib.or(opts.max_file_mib)),
            true,
        );
        files.row(&outcome.file)?;
        for hit in &outcome.hits {
            detections.row(hit)?;
        }
        *statuses.entry(outcome.file.status.clone()).or_default() += 1;
        matches += outcome.hits.len();
        matching_files += usize::from(!outcome.hits.is_empty());
        mft_stats.files += 1;
        if let Some(snapshot) = outcome.snapshot {
            mft_scan::inspect(
                mft_scan::Inspection {
                    scanner: &scanner,
                    snapshot: &snapshot,
                    file: &outcome.file,
                    raw_hits: &outcome.hits,
                    locations: &outcome.locations,
                    limited: outcome.locations_limited,
                    output: &output,
                    maximum: opts.max_mft_recoveries,
                },
                &mut mft_stats,
                |recovery, stream| {
                    if let Some(table) = &mut mft_table {
                        table.row(&recovery)?;
                    }
                    if let Some((file, hits)) = stream {
                        files.row(&file)?;
                        for hit in &hits {
                            detections.row(hit)?;
                        }
                        *statuses.entry(file.status.clone()).or_default() += 1;
                        matches += hits.len();
                        matching_files += usize::from(!hits.is_empty());
                    }
                    Ok(())
                },
            )?;
        }
        if let Some(bar) = &bar {
            bar.inc(1);
        }
    }
    if let Some(bar) = &bar {
        bar.finish_and_clear();
    }
    if let Some(table) = &mut mft_table {
        table.flush()?;
    }
    files.flush()?;
    detections.flush()?;
    let incomplete = mft_stats.incomplete
        || statuses
            .iter()
            .any(|(k, v)| *v > 0 && !["Scanned", "SkippedRuleSource"].contains(&k.as_str()));
    let exit_code = if interrupt::requested() {
        130
    } else if incomplete {
        3
    } else {
        0
    };
    let summary = serde_json::json!({
        "Status":if exit_code == 0 { "Complete" } else { "Incomplete" }, "FinishedUTC":chrono::Utc::now().to_rfc3339(),
        "JunteiVersion":env!("CARGO_PKG_VERSION"), "Engine":"boreal", "EngineVersion":ENGINE_VERSION,
        "Ruleset":if opts.rules.is_some() { "local" } else { opts.ruleset.name() }, "RequestedRelease":if opts.rules.is_some() { None } else { Some(&opts.rules_release) },
        "DownloadURL":rules.url, "ArchiveSHA256":rules.archive_sha256, "RulesSHA256":hash(rules.source.as_bytes()), "RuleSources":rules.sources,
        "CompiledRules":scanner.rules().count(), "PublicRules":scanner.rules().filter(|r| !r.is_private).count(), "CompilationWarnings":warnings.len(),
        "Threads":opts.threads, "TimeoutSeconds":opts.timeout, "MaxFileMiB":opts.max_file_mib, "MaxMFTMiB":opts.max_mft_mib.or(opts.max_file_mib), "MFTLimitOverrideMiB":opts.max_mft_mib, "MaxMFTRecoveries":opts.max_mft_recoveries, "MFT":mft_stats,
        "Statuses":statuses, "RuleMatches":matches, "MatchingFiles":matching_files, "ExitCode":exit_code,
        "MatchLocations":"First occurrence per pattern, up to 16 patterns; no matched bytes exported",
        "Interpretation":"A match is a hunting lead, not a verdict. A scanned file with no matches is not proven safe. No archive/container unpacking or live-memory scan is performed."
    });
    write_new(
        &output.join("Summary.json"),
        &serde_json::to_vec_pretty(&summary)?,
    )?;
    if mft_stats.files > 0 {
        util::p_label(
            opts.common.no_color,
            "Recovered MFT streams: ",
            &mft_stats.recovered_files.to_string(),
        );
    }
    util::p_label(opts.common.no_color, "Rule matches: ", &matches.to_string());
    util::p_label(
        opts.common.no_color,
        "Matching files: ",
        &matching_files.to_string(),
    );
    util::p_label(
        opts.common.no_color,
        "Coverage: ",
        if incomplete {
            "Incomplete (see YaraFiles and Summary.json)"
        } else {
            "Complete for the selected regular files"
        },
    );
    util::p_label(
        opts.common.no_color,
        "Results saved: ",
        &opts.output.display().to_string(),
    );
    Ok(exit_code)
}
pub fn run(opts: &Options) -> i32 {
    match execute(opts) {
        Ok(code) => code,
        Err(e) => {
            util::error_msg(opts.common.no_color, &format!("YARA scan failed: {e:#}"));
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Commands};
    use clap::Parser;

    #[test]
    fn cli_defaults_and_offline_conflicts() {
        let cli = Cli::try_parse_from(["juntei", "yara-scan", "-f", "sample.bin", "-o", "results"])
            .unwrap();
        let Commands::YaraScan(opts) = cli.cmd else {
            panic!()
        };
        assert_eq!(opts.ruleset.name(), "core");
        assert!((1..=8).contains(&opts.threads));
        assert_eq!(opts.max_file_mib, None);
        assert_eq!(opts.max_mft_mib, None);
        assert_eq!(size_limit_bytes(opts.max_file_mib), u64::MAX);
        for set in ["core", "extended", "full"] {
            assert!(
                Cli::try_parse_from([
                    "juntei",
                    "yara-scan",
                    "-d",
                    "evidence",
                    "-o",
                    "out",
                    "--ruleset",
                    set
                ])
                .is_ok()
            );
        }
        assert!(
            Cli::try_parse_from([
                "juntei",
                "yara-scan",
                "-f",
                "sample",
                "-o",
                "out",
                "--rules",
                "local.yar"
            ])
            .is_ok()
        );
        for extra in [["--ruleset", "full"], ["--rules-release", "20261115"]] {
            let mut args = vec![
                "juntei",
                "yara-scan",
                "-f",
                "sample",
                "-o",
                "out",
                "--rules",
                "local.yar",
            ];
            args.extend(extra);
            assert!(Cli::try_parse_from(args).is_err());
        }
        for bad in ["../x", "20269999", "202611150", "v20261115"] {
            assert!(parse_release(bad).is_err());
        }
        assert!(parse_release("20261115").is_ok());
    }

    #[test]
    fn packages_are_exact_and_bounded() {
        for set in [Ruleset::Core, Ruleset::Extended, Ruleset::Full] {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            zip.start_file(
                format!("packages/{0}/yara-rules-{0}.yar", set.name()),
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            zip.write_all(b"rule test { condition: true }").unwrap();
            let bytes = zip.finish().unwrap().into_inner();
            assert_eq!(
                unpack_rules(&bytes, set).unwrap(),
                b"rule test { condition: true }"
            );
            assert!(download_url(set, "latest").contains("/latest/download/"));
            assert!(download_url(set, "20261115").contains("/download/20261115/"));
        }
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        zip.start_file("../../escape.yar", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"rule test { condition: true }").unwrap();
        assert!(unpack_rules(&zip.finish().unwrap().into_inner(), Ruleset::Core).is_err());
        assert!(bounded_read(&b"abcd"[..], 3).is_err());
        assert_eq!(bounded_read(&b"abc"[..], 3).unwrap(), b"abc");
        assert_eq!(bounded_read(&b"abc"[..], u64::MAX).unwrap(), b"abc");
        assert_eq!(size_limit_bytes(Some(8192)), 8192 * MIB);
        for bad in ["0", "-1", "1.5", "18446744073709551615", "17592186044416"] {
            assert!(parse_size_limit(bad).is_err());
        }
        assert_eq!(parse_size_limit("17592186044415").unwrap(), u64::MAX / MIB);
        assert_eq!(
            hash(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn local_directory_includes_are_expanded_in_place_once() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("helpers")).unwrap();
        fs::write(
            root.join("helpers/a.yar"),
            "rule helper { condition: prefix }",
        )
        .unwrap();
        fs::write(root.join("z.yara"), "\u{feff}rule prefix { condition: true }\ninclude \"helpers/a.yar\"\n// include \"nonexistent\"\nrule final { strings: $s = \"include \\\"fake\\\"\" condition: helper and $s }").unwrap();
        let rules = local_rules(root).unwrap();
        assert_eq!(rules.paths.len(), 2);
        assert_eq!(rules.sources.len(), 2);
        let original = fs::read(root.join("z.yara")).unwrap();
        let provenance = rules
            .sources
            .iter()
            .find(|s| s.file_path == "z.yara")
            .unwrap();
        assert_eq!(provenance.sha256, hash(&original));
        assert_eq!(provenance.size, original.len());
        let (scanner, _) = compile(&rules.source, 1).unwrap();
        assert_eq!(scanner.rules().count(), 3);
        assert!(
            scanner
                .scan_mem(b"include \"fake\"")
                .unwrap()
                .rules
                .iter()
                .any(|r| r.name == "final")
        );
    }

    #[test]
    fn invalid_and_external_includes_never_silently_drop_rules() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("rules");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("bad.yar"), "include \"../outside.yar\"").unwrap();
        fs::write(
            tmp.path().join("outside.yar"),
            "rule outside { condition: true }",
        )
        .unwrap();
        assert!(
            local_rules(&root)
                .unwrap_err()
                .to_string()
                .contains("escapes")
        );
        fs::write(root.join("bad.yar"), "include \"bad.yar\"").unwrap();
        assert!(
            local_rules(&root)
                .unwrap_err()
                .to_string()
                .contains("Circular")
        );
        fs::write(root.join("bad.yar"), "this is invalid").unwrap();
        assert!(local_rules(&root).is_err());
        assert!(compile("rule invalid { condition: missing_external }", 1).is_err());
        assert!(compile("private rule hidden { condition: true }", 1).is_err());
    }

    #[test]
    fn normal_file_semantics_offsets_and_coverage() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("example.exe.quarantined");
        fs::write(&path, b"MZ_JUNTEI_TEST_ONLY").unwrap();
        let c = Candidate {
            path,
            relative: "example.exe.quarantined".into(),
            size: 19,
            pre_status: None,
            note: String::new(),
        };
        let (scanner, _) = compile("rule example : fixture { meta: author = \"test\" strings: $a = \"JUNTEI_TEST_ONLY\" condition: uint16(0) == 0x5a4d and filesize == 20 and $a }", 1).unwrap();
        // The sample is 19 bytes; filesize must be evaluated as a real file, not process memory.
        assert_eq!(scan(&scanner, &c, "HOST".into(), 1024).1.len(), 0);
        let (scanner, _) = compile("rule example : fixture { meta: author = \"test\" strings: $a = \"JUNTEI_TEST_ONLY\" condition: uint16(0) == 0x5a4d and filesize == 19 and $a }", 1).unwrap();
        let (row, hits) = scan(&scanner, &c, "HOST".into(), 1024);
        assert_eq!(row.status, "Scanned");
        assert_eq!(row.matches, 1);
        assert_eq!(hits[0].file_path, "example.exe.quarantined");
        assert!(hits[0].metadata.contains("test"));
        let locations: serde_json::Value = serde_json::from_str(&hits[0].match_locations).unwrap();
        assert_eq!(locations[0]["Offset"], 3);
        assert!(!hits[0].match_locations.contains("JUNTEI_TEST_ONLY"));
        assert_eq!(
            scan(&scanner, &c, String::new(), 1).0.status,
            "SkippedTooLarge"
        );
        fs::remove_file(&c.path).unwrap();
        assert_eq!(
            scan(&scanner, &c, String::new(), 1024).0.status,
            "ReadError"
        );
    }

    #[test]
    fn evaluation_timeout_is_reported_as_incomplete() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("sample.bin");
        fs::write(&path, vec![b'A'; 1024 * 1024]).unwrap();
        let candidate = Candidate {
            path,
            relative: "sample.bin".into(),
            size: 1024 * 1024,
            pre_status: None,
            note: String::new(),
        };
        let (mut scanner, _) = compile(
            "rule example { condition: for all i in (0..1000000000) : (i >= 0) }",
            1,
        )
        .unwrap();
        scanner
            .set_scan_params(ScanParams::default().timeout_duration(Some(Duration::from_nanos(1))));
        let (row, _) = scan(&scanner, &candidate, String::new(), 2 * 1024 * 1024);
        assert_eq!(row.status, "ScanError");
        assert_eq!(row.note, "timeout");
        assert!(!row.sha256.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn traversal_does_not_follow_symlinks() {
        use std::os::unix::fs::symlink;
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("evidence");
        fs::create_dir(&root).unwrap();
        fs::write(tmp.path().join("outside"), b"secret").unwrap();
        symlink(tmp.path().join("outside"), root.join("link")).unwrap();
        symlink(tmp.path(), root.join("loop")).unwrap();
        let candidates = discover(&root, &root, &HashSet::new());
        assert_eq!(candidates.len(), 2);
        assert!(
            candidates
                .iter()
                .all(|c| c.pre_status == Some("SkippedSymlink"))
        );
    }
}
