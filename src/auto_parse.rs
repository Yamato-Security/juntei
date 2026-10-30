//! `auto-parse`: discover collected Windows systems, run every Juntei parser in parallel, and
//! organize the results by computer. Parser commands run as child processes so one damaged
//! artifact cannot take down the whole collection and CPU-heavy parsers can run concurrently.

use crate::cli::{AutoOutputType, AutoParseOptions};
use crate::color::JunteiColor;
use crate::progress;
use crate::registry::{Hosts, Logs, open};
use crate::util::{self, error_msg, fmt_num, p, p_label};
use chrono::Utc;
use serde::Serialize;
use serde::de::IgnoredAny;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashSet};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

const WORK_DIR_PREFIX: &str = ".juntei-work-";

#[derive(Clone)]
struct Host {
    hostname: String,
    computer: String,
    fqdn: Option<String>,
    source: &'static str,
    root: PathBuf,
    windows: Vec<PathBuf>,
    folder: String,
}

#[derive(Clone, Copy)]
enum OutputShape {
    File,
    Directory,
    CsvDirectory,
}

#[derive(Clone)]
struct Task {
    computer: String,
    artifact: &'static str,
    input_bytes: u64,
    args: Vec<OsString>,
    output: PathBuf,
    log: PathBuf,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum TaskStatus {
    Succeeded,
    Skipped,
    Failed,
}

impl TaskStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
        }
    }
}

#[derive(Serialize)]
struct TaskResult {
    #[serde(rename = "Computer")]
    computer: String,
    artifact: String,
    status: TaskStatus,
    duration_ms: u128,
    exit_code: Option<i32>,
    command: Vec<String>,
    output: Option<String>,
    log: String,
    error: Option<String>,
}

#[derive(Serialize)]
struct ManifestHost {
    #[serde(rename = "Computer")]
    computer: String,
    hostname: String,
    fqdn: Option<String>,
    hostname_source: &'static str,
    evidence_root: String,
    windows_directories: Vec<String>,
    output_directory: String,
}

#[derive(Serialize)]
struct Manifest<'a> {
    generated_utc: String,
    juntei_version: &'static str,
    evidence_directory: String,
    output_directory: String,
    output_type: &'static str,
    single_file: bool,
    extract_defender_quarantined: bool,
    threads: usize,
    max_depth: usize,
    hosts: Vec<ManifestHost>,
    tasks: &'a [TaskResult],
}

struct DataFile {
    path: PathBuf,
}

/// Run the command; returns the exit status.
pub fn run(opts: &AutoParseOptions) -> i32 {
    let no_color = opts.common.no_color;
    if !opts.directory.is_dir() {
        error_msg(
            no_color,
            &format!(
                "The evidence directory does not exist: {}",
                opts.directory.display()
            ),
        );
        return 1;
    }
    let evidence = match fs::canonicalize(&opts.directory) {
        Ok(path) => path,
        Err(e) => {
            error_msg(
                no_color,
                &format!(
                    "Could not resolve the evidence directory {}: {e}",
                    opts.directory.display()
                ),
            );
            return 1;
        }
    };
    if opts.single_file && !opts.output_type.is_json() {
        error_msg(
            no_color,
            "-s, --single-file can only be used with -t json or -t jsonl.",
        );
        return 1;
    }
    if let Err(msg) = output_separate_from_evidence(&evidence, &opts.output) {
        error_msg(no_color, &msg);
        return 1;
    }
    if let Err(msg) = util::check_output(&opts.output, true, opts.clobber) {
        error_msg(no_color, &msg);
        return 1;
    }

    let mut hosts = match discover_hosts(&evidence, opts.max_depth) {
        Ok(hosts) if !hosts.is_empty() => hosts,
        Ok(_) => {
            error_msg(
                no_color,
                &format!(
                    "No Windows directories were found within depth {} of {}.",
                    opts.max_depth,
                    evidence.display()
                ),
            );
            return 1;
        }
        Err(e) => {
            error_msg(
                no_color,
                &format!("Could not search {}: {e}", evidence.display()),
            );
            return 1;
        }
    };
    for host in &hosts {
        if let Err(msg) = output_separate_from_evidence(&host.root, &opts.output) {
            error_msg(no_color, &msg);
            return 1;
        }
    }
    assign_output_folders(&mut hosts);

    if let Err(e) = fs::create_dir_all(&opts.output) {
        error_msg(
            no_color,
            &format!("Could not create {}: {e}", opts.output.display()),
        );
        return 1;
    }
    let work = if opts.single_file {
        match unused_child(
            &opts.output,
            &format!("{WORK_DIR_PREFIX}{}", std::process::id()),
        ) {
            Ok(path) => path,
            Err(e) => {
                error_msg(
                    no_color,
                    &format!("Could not choose a temporary output directory: {e}"),
                );
                return 1;
            }
        }
    } else {
        opts.output.clone()
    };
    if let Err(e) = fs::create_dir_all(&work) {
        error_msg(
            no_color,
            &format!("Could not create {}: {e}", work.display()),
        );
        return 1;
    }

    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            error_msg(
                no_color,
                &format!("Could not locate the Juntei executable: {e}"),
            );
            return 1;
        }
    };
    let tasks = match build_tasks(&hosts, opts, &work) {
        Ok(tasks) => tasks,
        Err(e) => {
            error_msg(
                no_color,
                &format!("Could not prepare the parser tasks: {e}"),
            );
            return 1;
        }
    };
    if opts.clobber
        && let Err(e) = clear_generated_outputs(&tasks, &work)
    {
        error_msg(
            no_color,
            &format!("Could not clear an earlier generated output: {e}"),
        );
        return 1;
    }
    if opts.clobber
        && let Err(e) = clear_defender_extractions(&hosts, &opts.output)
    {
        error_msg(
            no_color,
            &format!("Could not clear earlier Defender extraction output: {e}"),
        );
        return 1;
    }
    if opts.clobber {
        let mut root_files = vec![
            opts.output.join("manifest.json"),
            opts.output.join("manifest.csv"),
        ];
        if opts.single_file {
            root_files.push(
                opts.output
                    .join(format!("all-artifacts.{}", opts.output_type.as_str())),
            );
        }
        for path in root_files {
            if let Err(e) = remove_generated_file(&path) {
                error_msg(
                    no_color,
                    &format!(
                        "Could not clear earlier generated file {}: {e}",
                        path.display()
                    ),
                );
                return 1;
            }
        }
    }
    let available = std::thread::available_parallelism().map_or(1, usize::from);
    let threads = opts
        .threads
        .unwrap_or(available)
        .max(1)
        .min(tasks.len().max(1));

    p_label(no_color, "Windows systems: ", &fmt_num(hosts.len()));
    for host in &hosts {
        let fqdn = host
            .fqdn
            .as_deref()
            .filter(|fqdn| *fqdn != host.computer)
            .map_or_else(String::new, |fqdn| format!(" ({fqdn})"));
        p(
            None,
            &format!("  {}{}: {}", host.computer, fqdn, host.root.display()),
            true,
        );
    }
    p_label(no_color, "Parser tasks: ", &fmt_num(tasks.len()));
    p_label(no_color, "Parallel processes: ", &fmt_num(threads));
    p(None, "", true);
    p(
        JunteiColor::Orange.rdg(no_color),
        "Parsing in progress. Per-parser output is saved under each computer's logs directory.",
        true,
    );
    p(None, "", true);

    let bar = progress::progress_bar(tasks.len(), no_color);
    let mut results = run_tasks(&tasks, &executable, threads, &bar);
    bar.finish_with_message(progress::finished_message(no_color));
    let mut data_files = Vec::new();
    if opts.output_type.is_json() {
        for (task, result) in tasks.iter().zip(&mut results) {
            if result.status != TaskStatus::Succeeded {
                continue;
            }
            let files = json_files(&task.output, opts.output_type);
            if files.is_empty() {
                result.status = TaskStatus::Failed;
                result.error = Some("the parser succeeded but produced no JSON data file".into());
                continue;
            }
            let mut annotated = Vec::new();
            for path in files {
                if let Err(e) =
                    annotate_json_file(&path, &task.computer, task.artifact, opts.output_type)
                {
                    result.status = TaskStatus::Failed;
                    result.output = None;
                    result.error = Some(format!(
                        "could not add Computer and Artifact to {}: {e}",
                        path.display()
                    ));
                    break;
                }
                annotated.push(DataFile { path });
            }
            if result.status == TaskStatus::Succeeded {
                data_files.extend(annotated);
            }
        }
    }

    let mut combined = None;
    if opts.single_file {
        let name = format!("all-artifacts.{}", opts.output_type.as_str());
        let path = opts.output.join(name);
        match consolidate(&data_files, &path, opts.output_type) {
            Ok(()) => {
                combined = Some(path.clone());
                let parser_failed = results
                    .iter()
                    .any(|result| result.status == TaskStatus::Failed);
                if !parser_failed
                    && temporary_work_directory(&opts.output, &work)
                    && let Err(e) = fs::remove_dir_all(&work)
                {
                    error_msg(
                        no_color,
                        &format!(
                            "Could not remove temporary parser outputs {}: {e}",
                            work.display()
                        ),
                    );
                    return 1;
                }
                if parser_failed {
                    p(
                        JunteiColor::Orange.rdg(no_color),
                        &format!(
                            "Temporary parser outputs were kept in {} because at least one parser failed.",
                            work.display()
                        ),
                        true,
                    );
                }
                for result in &mut results {
                    if result.status == TaskStatus::Succeeded {
                        result.output = Some(path.to_string_lossy().to_string());
                    }
                }
            }
            Err(e) => {
                error_msg(
                    no_color,
                    &format!("Could not create {}: {e}", path.display()),
                );
                error_msg(
                    no_color,
                    &format!("Temporary parser outputs were kept in {}.", work.display()),
                );
                return 1;
            }
        }
    }

    let manifest = Manifest {
        generated_utc: Utc::now().to_rfc3339(),
        juntei_version: env!("CARGO_PKG_VERSION"),
        evidence_directory: evidence.to_string_lossy().to_string(),
        output_directory: opts.output.to_string_lossy().to_string(),
        output_type: opts.output_type.as_str(),
        single_file: opts.single_file,
        extract_defender_quarantined: opts.extract_defender_quarantined,
        threads,
        max_depth: opts.max_depth,
        hosts: hosts.iter().map(manifest_host).collect(),
        tasks: &results,
    };
    let manifest_path = opts.output.join("manifest.json");
    if let Err(e) = write_pretty_json(&manifest_path, &manifest) {
        error_msg(
            no_color,
            &format!("Could not write {}: {e}", manifest_path.display()),
        );
        return 1;
    }
    let manifest_csv_path = opts.output.join("manifest.csv");
    if let Err(e) = write_manifest_csv(&manifest_csv_path, &manifest) {
        error_msg(
            no_color,
            &format!("Could not write {}: {e}", manifest_csv_path.display()),
        );
        return 1;
    }
    let (file_count, total_bytes) = match output_totals(&opts.output) {
        Ok(totals) => totals,
        Err(e) => {
            error_msg(no_color, &format!("Could not summarize output files: {e}"));
            return 1;
        }
    };

    let succeeded = results
        .iter()
        .filter(|r| r.status == TaskStatus::Succeeded)
        .count();
    let skipped = results
        .iter()
        .filter(|r| r.status == TaskStatus::Skipped)
        .count();
    let failed = results
        .iter()
        .filter(|r| r.status == TaskStatus::Failed)
        .count();
    p(None, "", true);
    p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
    p(None, "", true);
    p_label(no_color, "Succeeded: ", &fmt_num(succeeded));
    p_label(no_color, "Not collected / skipped: ", &fmt_num(skipped));
    p_label(no_color, "Failed: ", &fmt_num(failed));
    p_label(
        no_color,
        "Results saved: ",
        &format!(
            "{}/ ( {} files / {:.2} MB )",
            opts.output.to_string_lossy().trim_end_matches(['/', '\\']),
            fmt_num(file_count),
            total_bytes as f64 / 1_000_000.0,
        ),
    );
    if let Some(path) = combined {
        p_label(no_color, "Combined data: ", &path.to_string_lossy());
    }
    if failed > 0 {
        for result in results.iter().filter(|r| r.status == TaskStatus::Failed) {
            p(
                JunteiColor::Red.rdg(no_color),
                &format!(
                    "  {} / {}: {} (log: {})",
                    result.computer,
                    result.artifact,
                    result.error.as_deref().unwrap_or("parser failed"),
                    result.log
                ),
                true,
            );
        }
    }
    p(None, "", true);

    i32::from(failed > 0 || succeeded == 0)
}

fn discover_hosts(input: &Path, max_depth: usize) -> io::Result<Vec<Host>> {
    let windows = windows_directories(input, max_depth)?;
    let mut roots: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for windows_dir in windows {
        let root = if windows_dir == input {
            input.to_path_buf()
        } else {
            collection_root(input, &windows_dir)
                .or_else(|| windows_dir.parent().map(Path::to_path_buf))
                .unwrap_or_else(|| input.to_path_buf())
        };
        roots.entry(root).or_default().push(windows_dir);
    }
    let roots = merge_nested_roots(roots);
    let mut hosts = Vec::new();
    for (root, mut windows) in roots {
        windows.sort();
        windows.dedup();
        let client = client_info(&root);
        let (computer, fqdn, source) = if let Some((hostname, fqdn)) = client {
            (hostname, fqdn, "client_info.json")
        } else if let Some(name) = system_computer_name(&windows) {
            (name, None, "SYSTEM hive")
        } else {
            (fallback_computer(&root), None, "directory name")
        };
        hosts.push(Host {
            folder: safe_component(&computer),
            hostname: computer.clone(),
            computer,
            fqdn,
            source,
            root,
            windows,
        });
    }
    hosts.sort_by(|a, b| {
        a.computer
            .to_ascii_lowercase()
            .cmp(&b.computer.to_ascii_lowercase())
            .then_with(|| a.root.cmp(&b.root))
    });
    Ok(hosts)
}

/// A legacy installation such as `C\Windows.old\Windows` belongs to the same collected volume as
/// `C\Windows`. Merge its candidate root into the outer root instead of parsing it a second time.
/// A nested collection with its own `client_info.json` remains a distinct system.
fn merge_nested_roots(roots: BTreeMap<PathBuf, Vec<PathBuf>>) -> BTreeMap<PathBuf, Vec<PathBuf>> {
    let mut entries: Vec<_> = roots.into_iter().collect();
    entries.sort_by(|(a, _), (b, _)| {
        a.components()
            .count()
            .cmp(&b.components().count())
            .then_with(|| a.cmp(b))
    });
    let mut merged: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for (root, windows) in entries {
        let ancestor = (!root.join("client_info.json").is_file())
            .then(|| {
                merged
                    .keys()
                    .filter(|candidate| root != **candidate && root.starts_with(candidate))
                    .max_by_key(|candidate| candidate.components().count())
                    .cloned()
            })
            .flatten();
        if let Some(ancestor) = ancestor {
            merged.get_mut(&ancestor).unwrap().extend(windows);
        } else {
            merged.insert(root, windows);
        }
    }
    merged
}

fn windows_directories(root: &Path, max_depth: usize) -> io::Result<Vec<PathBuf>> {
    if is_windows_system_directory(root) {
        return Ok(vec![root.to_path_buf()]);
    }
    let mut found = Vec::new();
    let mut pending = vec![(root.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = pending.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) if depth == 0 => return Err(e),
            Err(_) => continue,
        };
        let mut children = Vec::new();
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if !kind.is_dir() {
                continue;
            }
            let path = entry.path();
            let child_depth = depth + 1;
            if is_windows_system_directory(&path) {
                if child_depth <= max_depth {
                    found.push(path);
                }
                continue;
            }
            if child_depth < max_depth {
                children.push(path);
            }
        }
        children.sort();
        pending.extend(children.into_iter().rev().map(|p| (p, depth + 1)));
    }
    found.sort();
    found.dedup();
    Ok(found)
}

fn is_windows_system_directory(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("Windows"))
        && ["System32", "Prefetch", "AppCompat"]
            .iter()
            .any(|child| child_ci(path, child).is_some_and(|candidate| candidate.is_dir()))
}

fn collection_root(input: &Path, windows: &Path) -> Option<PathBuf> {
    let mut at = Some(windows);
    while let Some(dir) = at {
        if dir.join("client_info.json").is_file() {
            return Some(dir.to_path_buf());
        }
        if dir == input {
            break;
        }
        at = dir.parent().filter(|parent| parent.starts_with(input));
    }
    None
}

fn client_info(root: &Path) -> Option<(String, Option<String>)> {
    let file = File::open(root.join("client_info.json")).ok()?;
    let value: Value = serde_json::from_reader(BufReader::new(file)).ok()?;
    let hostname = value.get("hostname")?.as_str()?.trim();
    if hostname.is_empty() {
        return None;
    }
    let fqdn = value
        .get("fqdn")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    Some((hostname.to_string(), fqdn))
}

fn system_computer_name(windows: &[PathBuf]) -> Option<String> {
    for windows in windows {
        let config = child_ci(windows, "System32").and_then(|p| child_ci(&p, "config"));
        let system = config.and_then(|p| child_ci(&p, "SYSTEM"));
        let Some(system) = system else {
            continue;
        };
        let Ok(opened) = open(&system, Logs::NextToIt) else {
            continue;
        };
        let mut hosts = Hosts::new(true);
        if let Some(name) = hosts.of(&system, &opened) {
            return Some(name);
        }
    }
    None
}

fn child_ci(dir: &Path, name: &str) -> Option<PathBuf> {
    fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case(name))
        })
}

fn fallback_computer(root: &Path) -> String {
    let name = root
        .ancestors()
        .filter_map(Path::file_name)
        .map(|name| name.to_string_lossy().to_string())
        .find(|name| !name.is_empty() && !collection_scaffolding(name))
        .unwrap_or_else(|| "UNKNOWN".to_string());
    name.split_once("_F.")
        .map_or(name.clone(), |(computer, _)| computer.to_string())
}

/// Directory names used to lay out a collected volume are not useful computer names. Walk above
/// them so `HOST2/C/Windows` becomes `HOST2`, and a Velociraptor path such as
/// `HOST2/uploads/auto/C%3A/Windows` does too.
fn collection_scaffolding(name: &str) -> bool {
    if name.eq_ignore_ascii_case("auto") || name.eq_ignore_ascii_case("uploads") {
        return true;
    }
    let bytes = name.as_bytes();
    (bytes.len() == 1 && bytes[0].is_ascii_alphabetic())
        || (bytes.len() == 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
        || (bytes.len() == 4
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b'%'
            && bytes[2] == b'3'
            && bytes[3].eq_ignore_ascii_case(&b'a'))
}

fn safe_component(name: &str) -> String {
    let safe: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let safe = safe.trim_matches('.');
    let safe = if safe.is_empty() { "UNKNOWN" } else { safe };
    let upper = safe.to_ascii_uppercase();
    let reserved = matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || upper
            .strip_prefix("COM")
            .or_else(|| upper.strip_prefix("LPT"))
            .is_some_and(|number| {
                matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            });
    if reserved {
        format!("_{safe}")
    } else {
        safe.to_string()
    }
}

fn assign_output_folders(hosts: &mut [Host]) {
    let mut used = HashSet::new();
    for host in hosts {
        let base = host.folder.clone();
        let mut candidate = base.clone();
        let mut number = 2usize;
        while !used.insert(candidate.to_ascii_lowercase()) {
            candidate = format!("{base}_{number}");
            number += 1;
        }
        if candidate != base {
            // Keep SIEM records from separate collections distinguishable as well as their output
            // directories. Otherwise duplicate fallback names would remain identical in the JSON
            // `Computer` field even though their folders were suffixed.
            host.computer.clone_from(&candidate);
        }
        host.folder = candidate;
    }
}

fn build_tasks(hosts: &[Host], opts: &AutoParseOptions, work: &Path) -> io::Result<Vec<Task>> {
    let mut tasks = Vec::new();
    for host in hosts {
        let inputs = InputInventory::scan(&host.root);
        let data_dir = work.join(&host.folder);
        let log_dir = opts.output.join(&host.folder).join("logs");
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(&log_dir)?;
        let mut add =
            |artifact: &'static str, command: &[&str], shape: OutputShape, extra: &[&str]| {
                let output = task_output(&data_dir, artifact, opts.output_type, shape);
                let mut args: Vec<OsString> = command.iter().map(OsString::from).collect();
                args.push("-d".into());
                args.push(host.root.as_os_str().to_owned());
                args.push("-o".into());
                args.push(output.as_os_str().to_owned());
                args.push("-t".into());
                args.push(opts.output_type.as_str().into());
                args.extend(extra.iter().map(OsString::from));
                if artifact == "defender-quarantine" && opts.extract_defender_quarantined {
                    args.push("--extract".into());
                    args.push(
                        opts.output
                            .join(&host.folder)
                            .join("defender-quarantined")
                            .into_os_string(),
                    );
                }
                args.push("-C".into());
                args.push("-K".into());
                args.push("-q".into());
                tasks.push(Task {
                    computer: host.computer.clone(),
                    artifact,
                    input_bytes: inputs.sizes.get(artifact).copied().unwrap_or(0),
                    args,
                    output,
                    log: log_dir.join(format!("{artifact}.txt")),
                });
            };

        add(
            "amcache",
            &["parse-amcache"],
            OutputShape::CsvDirectory,
            &["-a"],
        );
        add("prefetch", &["parse-prefetch"], OutputShape::File, &[]);
        add(
            "prefetch-timeline",
            &["parse-prefetch"],
            OutputShape::File,
            &["--timeline"],
        );
        add("i30", &["parse-i30"], OutputShape::File, &[]);
        let srum_extra: &[&str] = if opts.output_type.is_json() {
            &["--include-value-state"]
        } else {
            &[]
        };
        add("srum", &["parse-srum"], OutputShape::Directory, srum_extra);
        let code_page = opts.code_page.to_string();
        add(
            "lnk",
            &["parse-lnk"],
            OutputShape::File,
            &["--code-page", &code_page],
        );
        add(
            "jump-lists",
            &["parse-jump-lists"],
            OutputShape::CsvDirectory,
            &["--code-page", &code_page],
        );
        add("shimcache", &["parse-shimcache"], OutputShape::File, &[]);
        add(
            "defender-quarantine",
            &["parse-defender-quarantine"],
            OutputShape::File,
            &[],
        );
        add(
            "registry",
            &["parse-registry", "read"],
            OutputShape::File,
            &["--recover-deleted", "--raw-data", "--qword-as-filetime"],
        );
        add(
            "services",
            &["parse-registry", "services"],
            OutputShape::File,
            &["--all-control-sets"],
        );
        add(
            "recentdocs",
            &["parse-registry", "recentdocs"],
            OutputShape::File,
            &[],
        );
        add(
            "taskcache",
            &["parse-registry", "taskcache"],
            OutputShape::File,
            &[],
        );
        add("sam", &["parse-registry", "sam"], OutputShape::File, &[]);
        drop(add);

        let mft_extra: &[&str] = if opts.output_type == AutoOutputType::Csv {
            &["--all-times", "--short-names"]
        } else {
            &[]
        };
        add_file_artifact_tasks(
            &mut tasks,
            host,
            opts,
            &data_dir,
            &log_dir,
            "mft",
            &["parse-mft"],
            inputs.mfts,
            mft_extra,
        );
        add_file_artifact_tasks(
            &mut tasks,
            host,
            opts,
            &data_dir,
            &log_dir,
            "usn-journal",
            &["parse-usn-journal"],
            inputs.journals,
            &[],
        );
    }
    prioritize_long_tasks(&mut tasks);
    Ok(tasks)
}

/// Inventory each host once. Sizes describe primary inputs, excluding transaction logs and
/// enrichment hives, and are only estimates of the work a parser will perform.
#[derive(Default)]
struct InputInventory {
    sizes: BTreeMap<&'static str, u64>,
    mfts: Vec<PathBuf>,
    journals: Vec<PathBuf>,
}

impl InputInventory {
    fn scan(root: &Path) -> Self {
        let mut found = Self::default();
        for path in util::find_files(root, |_| true) {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let size = path.metadata().map_or(0, |meta| meta.len());
            for (artifact, matches) in [
                ("amcache", crate::amcache::is_amcache_name(&name)),
                ("prefetch", crate::prefetch::is_prefetch_name(&name)),
                (
                    "prefetch-timeline",
                    crate::prefetch::is_prefetch_name(&name),
                ),
                ("srum", crate::srum::is_database_name(&name)),
                ("lnk", crate::lnk::is_lnk_name(&name)),
                ("jump-lists", crate::jump_lists::is_jump_list_name(&name)),
                ("shimcache", crate::shimcache::is_system_name(&name)),
                ("i30", crate::i30::is_i30_file(&path)),
            ] {
                if matches {
                    found.add_size(artifact, size);
                }
            }
            if crate::registry::is_hive_file(&path) {
                found.add_size("registry", size);
                let artifact = match crate::registry::hive_type_of_file(&path).as_deref() {
                    Some("SYSTEM") => Some("services"),
                    Some("NTUSER.DAT") => Some("recentdocs"),
                    Some("SOFTWARE") => Some("taskcache"),
                    Some("SAM") => Some("sam"),
                    _ => None,
                };
                if let Some(artifact) = artifact {
                    found.add_size(artifact, size);
                }
            }
            // Defender reads entry metadata, resource records, and encrypted content.
            if path.ancestors().any(|ancestor| {
                ancestor
                    .file_name()
                    .is_some_and(|name| name.eq_ignore_ascii_case("Quarantine"))
            }) {
                found.add_size("defender-quarantine", size);
            }
            if crate::mft::is_mft_name(&name) {
                found.mfts.push(path.clone());
            }
            if crate::usn_journal::is_journal_name(&name) {
                found.journals.push(path);
            }
        }
        found
    }

    fn add_size(&mut self, artifact: &'static str, bytes: u64) {
        let size = self.sizes.entry(artifact).or_default();
        *size = size.saturating_add(bytes);
    }
}

/// Start the usually longest parsers across every host before filling worker slots with short
/// tasks. Stable sorting preserves host and per-volume order inside each priority class.
fn prioritize_long_tasks(tasks: &mut [Task]) {
    tasks.sort_by_key(|task| match task.artifact {
        "mft" => 0,
        "usn-journal" => 1,
        _ => 2,
    });
}

/// File-shaped artifacts are written directly under the computer directory. A normal collected
/// volume has one of each (`mft.csv`, `usn-journal.csv`). If several volumes were collected for
/// one computer, keep every artifact as a numbered sibling instead of creating another directory.
#[allow(clippy::too_many_arguments)]
fn add_file_artifact_tasks(
    tasks: &mut Vec<Task>,
    host: &Host,
    opts: &AutoParseOptions,
    data_dir: &Path,
    log_dir: &Path,
    artifact: &'static str,
    command: &[&str],
    inputs: Vec<PathBuf>,
    extra: &[&str],
) {
    let inputs: Vec<(OsString, PathBuf)> = if inputs.is_empty() {
        vec![("-d".into(), host.root.clone())]
    } else {
        inputs.into_iter().map(|path| ("-f".into(), path)).collect()
    };
    for (index, (input_flag, input)) in inputs.into_iter().enumerate() {
        let input_bytes = if input_flag == "-f" {
            input.metadata().map_or(0, |meta| meta.len())
        } else {
            0
        };
        let stem = if index == 0 {
            artifact.to_string()
        } else {
            format!("{artifact}-{}", index + 1)
        };
        let output = data_dir.join(format!("{stem}.{}", opts.output_type.as_str()));
        let mut args: Vec<OsString> = command.iter().map(OsString::from).collect();
        args.push(input_flag);
        args.push(input.into_os_string());
        args.push("-o".into());
        args.push(output.as_os_str().to_owned());
        args.push("-t".into());
        args.push(opts.output_type.as_str().into());
        args.extend(extra.iter().map(OsString::from));
        args.push("-C".into());
        args.push("-K".into());
        args.push("-q".into());
        tasks.push(Task {
            computer: host.computer.clone(),
            artifact,
            input_bytes,
            args,
            output,
            log: log_dir.join(format!("{stem}.txt")),
        });
    }
}

fn task_output(
    host_dir: &Path,
    artifact: &str,
    output_type: AutoOutputType,
    shape: OutputShape,
) -> PathBuf {
    let directory = matches!(shape, OutputShape::Directory)
        || (matches!(shape, OutputShape::CsvDirectory) && output_type == AutoOutputType::Csv);
    if directory {
        host_dir.join(artifact)
    } else {
        host_dir.join(format!("{artifact}.{}", output_type.as_str()))
    }
}

/// With explicit `--clobber`, remove only the exact task targets under this run's work directory.
/// Otherwise a skipped parser could leave a stale artifact from an earlier run.
fn clear_generated_outputs(tasks: &[Task], work: &Path) -> io::Result<()> {
    for task in tasks {
        remove_generated_path(&task.output, work)?;
    }
    Ok(())
}

/// Defender extraction is an opt-in side output rather than a parser task target. Clear it on
/// every clobber run—even when `-x` is absent—so malware from an earlier run cannot survive as an
/// unrecorded stale result.
fn clear_defender_extractions(hosts: &[Host], output: &Path) -> io::Result<()> {
    for host in hosts {
        remove_generated_path(
            &output.join(&host.folder).join("defender-quarantined"),
            output,
        )?;
    }
    Ok(())
}

fn remove_generated_path(path: &Path, root: &Path) -> io::Result<()> {
    if !path.starts_with(root) || path == root {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "generated output {} is outside {}",
                path.display(),
                root.display()
            ),
        ));
    }
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn remove_generated_file(path: &Path) -> io::Result<()> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if metadata.is_file() || metadata.file_type().is_symlink() {
        fs::remove_file(path)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a generated file but found a directory",
        ))
    }
}

fn run_tasks(
    tasks: &[Task],
    executable: &Path,
    threads: usize,
    bar: &indicatif::ProgressBar,
) -> Vec<TaskResult> {
    let next = AtomicUsize::new(0);
    let results = Mutex::new(Vec::with_capacity(tasks.len()));
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(task) = tasks.get(index) else {
                        break;
                    };
                    bar.set_message(task_progress_message(task));
                    let result = run_task(task, executable);
                    results.lock().unwrap().push((index, result));
                    bar.inc(1);
                }
            });
        }
    });
    let mut results = results.into_inner().unwrap();
    results.sort_by_key(|(index, _)| *index);
    results.into_iter().map(|(_, result)| result).collect()
}

fn task_progress_message(task: &Task) -> String {
    format!(
        "{} / {} (~{})",
        task.computer,
        task.artifact,
        util::size_display(task.input_bytes)
    )
}

fn run_task(task: &Task, executable: &Path) -> TaskResult {
    let start = Instant::now();
    let command: Vec<String> =
        std::iter::once(executable.as_os_str().to_string_lossy().to_string())
            .chain(task.args.iter().map(|a| a.to_string_lossy().to_string()))
            .collect();
    let output = Command::new(executable).args(&task.args).output();
    let mut error = None;
    let (status, exit_code, stdout, stderr) = match output {
        Ok(output) => {
            let mut combined = String::from_utf8_lossy(&output.stdout).to_string();
            combined.push_str(&String::from_utf8_lossy(&output.stderr));
            let skipped = !output.status.success() && no_artifact_message(&combined);
            let status = if output.status.success() {
                TaskStatus::Succeeded
            } else if skipped {
                TaskStatus::Skipped
            } else {
                error = Some(
                    useful_error_line(&combined)
                        .unwrap_or("parser failed")
                        .to_string(),
                );
                TaskStatus::Failed
            };
            (status, output.status.code(), output.stdout, output.stderr)
        }
        Err(e) => {
            error = Some(format!("could not start parser: {e}"));
            (TaskStatus::Failed, None, Vec::new(), Vec::new())
        }
    };
    let mut log = Vec::new();
    let _ = writeln!(log, "Command: {}", command.join(" "));
    log.extend_from_slice(&stdout);
    if !stdout.is_empty() && !stdout.ends_with(b"\n") {
        log.push(b'\n');
    }
    log.extend_from_slice(&stderr);
    if let Some(parent) = task.log.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Err(e) = fs::write(&task.log, log) {
        error = Some(format!("could not write {}: {e}", task.log.display()));
    }
    TaskResult {
        computer: task.computer.clone(),
        artifact: task.artifact.to_string(),
        status: if error.is_some() && status != TaskStatus::Skipped {
            TaskStatus::Failed
        } else {
            status
        },
        duration_ms: start.elapsed().as_millis(),
        exit_code,
        command,
        output: (status == TaskStatus::Succeeded)
            .then(|| task.output.to_string_lossy().to_string()),
        log: task.log.to_string_lossy().to_string(),
        error,
    }
}

fn no_artifact_message(output: &str) -> bool {
    output.lines().any(|line| {
        let line = line.trim();
        line.starts_with("No ") && line.contains(" were found")
    })
}

fn useful_error_line(output: &str) -> Option<&str> {
    if let Some(error) = output
        .lines()
        .map(str::trim)
        .find(|line| line.to_ascii_lowercase().starts_with("error:"))
    {
        return Some(error);
    }
    output
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())
}

fn json_files(output: &Path, output_type: AutoOutputType) -> Vec<PathBuf> {
    let extension = output_type.as_str();
    if output.is_file() {
        return (output
            .extension()
            .is_some_and(|e| e.to_string_lossy().eq_ignore_ascii_case(extension)))
        .then(|| output.to_path_buf())
        .into_iter()
        .collect();
    }
    let mut files = Vec::new();
    let mut pending = vec![output.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|e| e.to_string_lossy().eq_ignore_ascii_case(extension))
            {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn annotate_json_file(
    path: &Path,
    computer: &str,
    artifact: &str,
    output_type: AutoOutputType,
) -> io::Result<()> {
    let input = BufReader::new(File::open(path)?);
    let temporary = unused_sibling(path, "annotated")?;
    let output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let mut writer = BufWriter::new(output);
    let write_result = (|| -> io::Result<()> {
        if output_type == AutoOutputType::Jsonl {
            return annotate_jsonl(input, &mut writer, computer, artifact);
        }
        for value in serde_json::Deserializer::from_reader(input).into_iter::<Value>() {
            let mut value = value.map_err(io::Error::other)?;
            annotate_value(&mut value, computer, artifact);
            serde_json::to_writer_pretty(&mut writer, &value).map_err(io::Error::other)?;
            writer.write_all(b"\n")?;
        }
        writer.flush()
    })();
    if let Err(e) = write_result {
        let _ = fs::remove_file(&temporary);
        return Err(e);
    }
    replace_generated_file(path, &temporary)
}

/// Validate and label JSONL without building and serializing every object again. Juntei's JSONL
/// writers emit one object per line, so the existing object can safely become the tail of a new
/// object after the two provenance fields. A non-object is rare, but is still wrapped as `Data`.
fn annotate_jsonl(
    mut input: impl BufRead,
    writer: &mut impl Write,
    computer: &str,
    artifact: &str,
) -> io::Result<()> {
    let mut prefix = b"{\"Computer\":".to_vec();
    serde_json::to_writer(&mut prefix, computer).map_err(io::Error::other)?;
    prefix.extend_from_slice(b",\"Artifact\":");
    serde_json::to_writer(&mut prefix, artifact).map_err(io::Error::other)?;

    let mut line = Vec::new();
    loop {
        line.clear();
        if input.read_until(b'\n', &mut line)? == 0 {
            break;
        }
        let start = line
            .iter()
            .position(|byte| !byte.is_ascii_whitespace())
            .unwrap_or(line.len());
        let end = line
            .iter()
            .rposition(|byte| !byte.is_ascii_whitespace())
            .map_or(start, |at| at + 1);
        let value = &line[start..end];
        if value.is_empty() {
            continue;
        }
        serde_json::from_slice::<IgnoredAny>(value).map_err(io::Error::other)?;
        if value.first() == Some(&b'{') && value.last() == Some(&b'}') {
            writer.write_all(&prefix)?;
            let inner = &value[1..value.len() - 1];
            if inner.iter().any(|byte| !byte.is_ascii_whitespace()) {
                writer.write_all(b",")?;
                writer.write_all(inner)?;
            }
            writer.write_all(b"}\n")?;
        } else {
            let mut parsed: Value = serde_json::from_slice(value).map_err(io::Error::other)?;
            annotate_value(&mut parsed, computer, artifact);
            serde_json::to_writer(&mut *writer, &parsed).map_err(io::Error::other)?;
            writer.write_all(b"\n")?;
        }
    }
    writer.flush()
}

fn annotate_value(value: &mut Value, computer: &str, artifact: &str) {
    if let Value::Object(object) = value {
        object.remove("Computer");
        object.remove("Artifact");
        object.shift_insert(
            0,
            "Artifact".to_string(),
            Value::String(artifact.to_string()),
        );
        object.shift_insert(
            0,
            "Computer".to_string(),
            Value::String(computer.to_string()),
        );
    } else {
        let data = std::mem::take(value);
        let mut object = Map::new();
        object.insert("Computer".to_string(), Value::String(computer.to_string()));
        object.insert("Artifact".to_string(), Value::String(artifact.to_string()));
        object.insert("Data".to_string(), data);
        *value = Value::Object(object);
    }
}

fn consolidate(files: &[DataFile], output: &Path, output_type: AutoOutputType) -> io::Result<()> {
    let out = File::create(output)?;
    let mut writer = BufWriter::new(out);
    let mut first = true;
    if output_type == AutoOutputType::Json {
        writer.write_all(b"[\n")?;
    }
    let mut ordered: Vec<&Path> = files.iter().map(|f| f.path.as_path()).collect();
    ordered.sort();
    for path in ordered {
        let mut input = BufReader::new(File::open(path)?);
        if output_type == AutoOutputType::Jsonl {
            io::copy(&mut input, &mut writer)?;
            continue;
        }
        for value in serde_json::Deserializer::from_reader(input).into_iter::<Value>() {
            let value = value.map_err(io::Error::other)?;
            if !first {
                writer.write_all(b",\n")?;
            }
            serde_json::to_writer_pretty(&mut writer, &value).map_err(io::Error::other)?;
            first = false;
        }
    }
    if output_type == AutoOutputType::Json {
        writer.write_all(b"\n]\n")?;
    }
    writer.flush()
}

fn unused_sibling(path: &Path, label: &str) -> io::Result<PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .map_or_else(|| "output".to_string(), |n| n.to_string_lossy().to_string());
    for number in 0..1000 {
        let candidate = parent.join(format!(".{name}.juntei-{label}-{number}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!("no unused temporary name next to {}", path.display()),
    ))
}

fn unused_child(parent: &Path, name: &str) -> io::Result<PathBuf> {
    for number in 0..1000 {
        let candidate = if number == 0 {
            parent.join(name)
        } else {
            parent.join(format!("{name}-{number}"))
        };
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!(
            "no unused temporary directory name under {}",
            parent.display()
        ),
    ))
}

fn temporary_work_directory(output: &Path, work: &Path) -> bool {
    work.parent() == Some(output)
        && work
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with(WORK_DIR_PREFIX))
}

fn replace_generated_file(path: &Path, replacement: &Path) -> io::Result<()> {
    let backup = unused_sibling(path, "original")?;
    fs::rename(path, &backup)?;
    if let Err(e) = fs::rename(replacement, path) {
        let _ = fs::rename(&backup, path);
        return Err(e);
    }
    fs::remove_file(backup)
}

fn manifest_host(host: &Host) -> ManifestHost {
    ManifestHost {
        computer: host.computer.clone(),
        hostname: host.hostname.clone(),
        fqdn: host.fqdn.clone(),
        hostname_source: host.source,
        evidence_root: host.root.to_string_lossy().to_string(),
        windows_directories: host
            .windows
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        output_directory: host.folder.clone(),
    }
}

fn write_pretty_json(path: &Path, value: &impl Serialize) -> io::Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    serde_json::to_writer_pretty(&mut writer, value).map_err(io::Error::other)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

/// A spreadsheet-friendly task index; the full run metadata remains in manifest.json.
fn write_manifest_csv(path: &Path, manifest: &Manifest<'_>) -> io::Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record([
        "Computer",
        "Hostname",
        "Fqdn",
        "HostnameSource",
        "EvidenceRoot",
        "Artifact",
        "Status",
        "DurationMs",
        "ExitCode",
        "Output",
        "Log",
        "Error",
        "Command",
    ])?;
    for task in manifest.tasks {
        let host = manifest
            .hosts
            .iter()
            .find(|host| host.computer == task.computer);
        writer.write_record([
            task.computer.clone(),
            host.map_or_else(String::new, |host| host.hostname.clone()),
            host.and_then(|host| host.fqdn.clone()).unwrap_or_default(),
            host.map_or_else(String::new, |host| host.hostname_source.to_string()),
            host.map_or_else(String::new, |host| host.evidence_root.clone()),
            task.artifact.clone(),
            task.status.as_str().to_string(),
            task.duration_ms.to_string(),
            task.exit_code
                .map_or_else(String::new, |code| code.to_string()),
            task.output.clone().unwrap_or_default(),
            task.log.clone(),
            task.error.clone().unwrap_or_default(),
            serde_json::to_string(&task.command).map_err(io::Error::other)?,
        ])?;
    }
    writer.flush()
}

/// Count the regular files currently under the results directory without following symlinks.
fn output_totals(root: &Path) -> io::Result<(usize, u64)> {
    let mut files = 0usize;
    let mut bytes = 0u64;
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                pending.push(entry.path());
            } else if kind.is_file() {
                files += 1;
                bytes = bytes.checked_add(entry.metadata()?.len()).ok_or_else(|| {
                    io::Error::other("output size exceeds the supported byte count")
                })?;
            }
        }
    }
    Ok((files, bytes))
}

fn output_separate_from_evidence(evidence: &Path, output: &Path) -> Result<(), String> {
    let evidence = fs::canonicalize(evidence)
        .map_err(|e| format!("Could not resolve {}: {e}", evidence.display()))?;
    let output = absolute_candidate(output)
        .map_err(|e| format!("Could not resolve {}: {e}", output.display()))?;
    if output.starts_with(&evidence) || evidence.starts_with(&output) {
        return Err(format!(
            "The output and evidence directories must not contain one another: {} and {}",
            output.display(),
            evidence.display()
        ));
    }
    Ok(())
}

fn absolute_candidate(path: &Path) -> io::Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    if path.exists() {
        return fs::canonicalize(&path);
    }
    let mut suffix = Vec::new();
    let mut at = path.as_path();
    while !at.exists() {
        suffix.push(at.file_name().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "path has no existing ancestor")
        })?);
        at = at.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "path has no existing ancestor")
        })?;
    }
    let mut absolute = fs::canonicalize(at)?;
    for part in suffix.into_iter().rev() {
        absolute.push(part);
    }
    Ok(normalize_absolute(&absolute))
}

fn normalize_absolute(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totals_include_nested_results_and_logs_without_following_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("HOST/logs")).unwrap();
        fs::write(dir.path().join("manifest.csv"), b"a,b\n").unwrap();
        fs::write(dir.path().join("HOST/result.csv"), b"x\n1\n").unwrap();
        fs::write(dir.path().join("HOST/logs/parser.txt"), b"log\n").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(dir.path(), dir.path().join("HOST/loop")).unwrap();
            std::os::unix::fs::symlink(
                dir.path().join("manifest.csv"),
                dir.path().join("link.csv"),
            )
            .unwrap();
        }
        assert_eq!(output_totals(dir.path()).unwrap(), (3, 12));
    }

    #[test]
    fn discovers_client_collections_and_respects_depth() {
        let dir = tempfile::tempdir().unwrap();
        let collection = dir.path().join("case");
        let windows = collection.join("uploads/auto/C/Windows");
        fs::create_dir_all(windows.join("System32")).unwrap();
        fs::write(
            collection.join("client_info.json"),
            r#"{"hostname":"HOST-1","fqdn":"host-1.example.test"}"#,
        )
        .unwrap();
        let hosts = discover_hosts(dir.path(), 6).unwrap();
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].computer, "HOST-1");
        assert_eq!(hosts[0].fqdn.as_deref(), Some("host-1.example.test"));
        assert_eq!(hosts[0].root, collection);
        assert!(discover_hosts(dir.path(), 3).unwrap().is_empty());

        let direct = windows_directories(&windows, 1).unwrap();
        assert_eq!(direct, vec![windows.clone()]);
        let direct_hosts = discover_hosts(&windows, 1).unwrap();
        assert_eq!(direct_hosts.len(), 1);
        assert_eq!(direct_hosts[0].root, windows);
    }

    #[test]
    fn ignores_profile_windows_and_merges_windows_old() {
        let dir = tempfile::tempdir().unwrap();
        let volume = dir.path().join("HOST1/C");
        let current = volume.join("Windows");
        let old = volume.join("Windows.old/Windows");
        let profile = volume.join("Users/alice/AppData/Roaming/Microsoft/Windows");
        fs::create_dir_all(current.join("System32")).unwrap();
        fs::create_dir_all(old.join("System32")).unwrap();
        fs::create_dir_all(profile.join("Recent")).unwrap();

        let hosts = discover_hosts(dir.path(), 10).unwrap();
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].root, volume);
        assert_eq!(hosts[0].windows, vec![current, old]);
        assert!(!hosts[0].windows.contains(&profile));
    }

    #[test]
    fn discovers_targeted_windows_artifact_directories() {
        let dir = tempfile::tempdir().unwrap();
        let prefetch = dir.path().join("PREFETCH_ONLY/C/Windows");
        let amcache = dir.path().join("AMCACHE_ONLY/uploads/auto/C%3A/Windows");
        let profile = dir
            .path()
            .join("PROFILE/Users/alice/AppData/Roaming/Microsoft/Windows");
        fs::create_dir_all(prefetch.join("Prefetch")).unwrap();
        fs::create_dir_all(amcache.join("AppCompat/Programs")).unwrap();
        fs::create_dir_all(profile.join("Recent")).unwrap();

        let found = windows_directories(dir.path(), 10).unwrap();
        assert_eq!(found, vec![amcache.clone(), prefetch.clone()]);
        assert!(!found.contains(&profile));
        let hosts = discover_hosts(dir.path(), 10).unwrap();
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0].computer, "AMCACHE_ONLY");
        assert_eq!(hosts[0].root, amcache.parent().unwrap());
        assert_eq!(hosts[1].computer, "PREFETCH_ONLY");
        assert_eq!(hosts[1].root, prefetch.parent().unwrap());
    }

    #[test]
    fn output_names_follow_format_and_shape() {
        let root = Path::new("out/HOST");
        assert_eq!(
            task_output(
                root,
                "amcache",
                AutoOutputType::Csv,
                OutputShape::CsvDirectory
            ),
            root.join("amcache")
        );
        assert_eq!(
            task_output(
                root,
                "amcache",
                AutoOutputType::Jsonl,
                OutputShape::CsvDirectory
            ),
            root.join("amcache.jsonl")
        );
        assert_eq!(
            task_output(root, "srum", AutoOutputType::Json, OutputShape::Directory),
            root.join("srum")
        );
    }

    #[test]
    fn annotates_and_consolidates_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let one = dir.path().join("one.jsonl");
        let two = dir.path().join("two.jsonl");
        fs::write(&one, "{\"value\":1}\n").unwrap();
        fs::write(&two, "[1,2]\n").unwrap();
        annotate_json_file(&one, "HOST1", "mft", AutoOutputType::Jsonl).unwrap();
        annotate_json_file(&two, "HOST2", "usn-journal", AutoOutputType::Jsonl).unwrap();
        let first: Value = serde_json::from_str(&fs::read_to_string(&one).unwrap()).unwrap();
        assert_eq!(first["Computer"], "HOST1");
        assert_eq!(first["Artifact"], "mft");
        let wrapped: Value = serde_json::from_str(&fs::read_to_string(&two).unwrap()).unwrap();
        assert_eq!(wrapped["Computer"], "HOST2");
        assert_eq!(wrapped["Data"], serde_json::json!([1, 2]));

        let combined = dir.path().join("all.jsonl");
        consolidate(
            &[DataFile { path: one }, DataFile { path: two }],
            &combined,
            AutoOutputType::Jsonl,
        )
        .unwrap();
        let lines: Vec<Value> = fs::read_to_string(combined)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0]["Computer"], "HOST1");
        assert_eq!(lines[1]["Computer"], "HOST2");
    }

    #[test]
    fn annotates_and_consolidates_json_array() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.json");
        fs::write(&source, "{\n  \"z_field\": 7,\n  \"a_field\": 8\n}\n").unwrap();
        annotate_json_file(&source, "HOST1", "registry", AutoOutputType::Json).unwrap();
        let annotated = fs::read_to_string(&source).unwrap();
        let positions: Vec<_> = ["Computer", "Artifact", "z_field", "a_field"]
            .map(|field| annotated.find(field).unwrap())
            .into();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        let combined = dir.path().join("all.json");
        consolidate(
            &[DataFile { path: source }],
            &combined,
            AutoOutputType::Json,
        )
        .unwrap();
        let values: Vec<Value> = serde_json::from_reader(File::open(combined).unwrap()).unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0]["Computer"], "HOST1");
        assert_eq!(values[0]["Artifact"], "registry");
        assert_eq!(values[0]["z_field"], 7);
        assert_eq!(values[0]["a_field"], 8);
    }

    #[test]
    fn malformed_jsonl_is_rejected_without_changing_the_parser_output() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.jsonl");
        let original = "{\"valid\":true}\nnot-json\n";
        fs::write(&source, original).unwrap();
        assert!(annotate_json_file(&source, "HOST1", "mft", AutoOutputType::Jsonl).is_err());
        assert_eq!(fs::read_to_string(source).unwrap(), original);
    }

    #[test]
    fn refuses_output_that_contains_or_is_inside_evidence() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("case")).unwrap();
        assert!(output_separate_from_evidence(dir.path(), &dir.path().join("results")).is_err());
        assert!(output_separate_from_evidence(dir.path(), dir.path().parent().unwrap()).is_err());
        assert!(
            output_separate_from_evidence(
                dir.path(),
                &dir.path()
                    .parent()
                    .unwrap()
                    .join("missing")
                    .join("..")
                    .join(dir.path().file_name().unwrap())
                    .join("results")
            )
            .is_err()
        );
        assert!(
            output_separate_from_evidence(
                dir.path(),
                &dir.path().parent().unwrap().join("elsewhere")
            )
            .is_ok()
        );
    }

    #[test]
    fn reports_the_useful_child_error() {
        let clap = "error: a value is required\n\nFor more information, try '--help'.\n";
        assert_eq!(useful_error_line(clap), Some("error: a value is required"));
        assert_eq!(useful_error_line("first\nlast\n"), Some("last"));
    }

    #[test]
    fn progress_identifies_the_computer_and_artifact() {
        let task = Task {
            computer: "HOST1".into(),
            artifact: "prefetch",
            input_bytes: 2048,
            args: Vec::new(),
            output: PathBuf::new(),
            log: PathBuf::new(),
        };
        assert_eq!(task_progress_message(&task), "HOST1 / prefetch (~2.0 KiB)");
    }

    #[test]
    fn progress_sizes_group_inputs_and_keep_file_tasks_separate() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("Windows/Prefetch")).unwrap();
        fs::create_dir_all(dir.path().join("Windows/System32/config")).unwrap();
        fs::create_dir_all(dir.path().join("Quarantine/ResourceData/AA")).unwrap();
        fs::write(dir.path().join("Windows/Prefetch/A.pf"), vec![0u8; 1024]).unwrap();
        fs::write(dir.path().join("Windows/Prefetch/B.pf"), vec![0u8; 2048]).unwrap();
        fs::write(dir.path().join("unrelated.bin"), vec![0u8; 4096]).unwrap();
        fs::write(
            dir.path().join("Quarantine/ResourceData/AA/content"),
            vec![0u8; 256],
        )
        .unwrap();
        let mut hive = [0u8; 0x70];
        hive[..4].copy_from_slice(b"regf");
        fs::write(dir.path().join("Windows/System32/config/SOFTWARE"), hive).unwrap();
        fs::write(
            dir.path().join("Windows/System32/config/SOFTWARE.LOG1"),
            vec![0u8; 512],
        )
        .unwrap();
        fs::write(dir.path().join("$MFT"), vec![0u8; 64]).unwrap();
        fs::write(dir.path().join("$J"), vec![0u8; 128]).unwrap();
        let inputs = InputInventory::scan(dir.path());
        assert_eq!(inputs.sizes["prefetch"], 3072);
        assert_eq!(inputs.sizes["prefetch-timeline"], 3072);
        assert_eq!(inputs.sizes["registry"], 0x70);
        assert_eq!(inputs.sizes["taskcache"], 0x70);
        assert_eq!(inputs.sizes["defender-quarantine"], 256);
        assert_eq!(inputs.mfts, vec![dir.path().join("$MFT")]);
        assert_eq!(inputs.journals, vec![dir.path().join("$J")]);
    }

    #[test]
    fn long_tasks_are_queued_across_hosts_before_short_tasks() {
        let task = |computer: &str, artifact| Task {
            computer: computer.into(),
            artifact,
            input_bytes: 0,
            args: Vec::new(),
            output: PathBuf::new(),
            log: PathBuf::new(),
        };
        let mut tasks = vec![
            task("HOST1", "prefetch"),
            task("HOST1", "mft"),
            task("HOST2", "registry"),
            task("HOST2", "usn-journal"),
            task("HOST2", "mft"),
        ];
        prioritize_long_tasks(&mut tasks);
        let order: Vec<_> = tasks
            .iter()
            .map(|task| (task.computer.as_str(), task.artifact))
            .collect();
        assert_eq!(
            order,
            vec![
                ("HOST1", "mft"),
                ("HOST2", "mft"),
                ("HOST2", "usn-journal"),
                ("HOST1", "prefetch"),
                ("HOST2", "registry"),
            ]
        );
    }

    #[test]
    fn output_folder_names_are_portable_and_unique() {
        assert_eq!(safe_component("host/name"), "host_name");
        assert_eq!(safe_component("CON"), "_CON");
        assert_eq!(safe_component("lpt9"), "_lpt9");

        let host = |computer: &str| Host {
            hostname: computer.to_string(),
            computer: computer.to_string(),
            fqdn: None,
            source: "test",
            root: PathBuf::new(),
            windows: Vec::new(),
            folder: safe_component(computer),
        };
        let mut hosts = vec![host("HOST"), host("host"), host("HOST_2")];
        assign_output_folders(&mut hosts);
        let folders: HashSet<_> = hosts
            .iter()
            .map(|host| host.folder.to_ascii_lowercase())
            .collect();
        assert_eq!(folders.len(), 3);
        let computers: HashSet<_> = hosts
            .iter()
            .map(|host| host.computer.to_ascii_lowercase())
            .collect();
        assert_eq!(computers.len(), 3);
        assert_eq!(hosts[1].computer, "host_2");
        assert_eq!(hosts[1].hostname, "host");
        let manifest = manifest_host(&hosts[1]);
        assert_eq!(manifest.computer, "host_2");
        assert_eq!(manifest.hostname, "host");
    }
}
