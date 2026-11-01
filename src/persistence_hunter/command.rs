use super::{
    Coverage, Finding, Scan, checks, context::Context, file_checks, profile::Profile, resolver,
    timeline,
};
use crate::cli::{PersistenceHunterOptions, RegistryOutputType};
use crate::{progress, registry, util};
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn run(opts: &PersistenceHunterOptions) -> i32 {
    let result = if opts.list_checks {
        execute(opts)
    } else {
        let threads = opts
            .threads
            .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, usize::from));
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .thread_name(|index| format!("persistence-{index}"))
            .build()
            .map_err(|e| format!("Could not start persistence worker threads: {e}"))
            .and_then(|pool| pool.install(|| execute(opts)))
    };
    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

fn execute(opts: &PersistenceHunterOptions) -> Result<i32, String> {
    let definitions = checks::catalogue();
    if opts.list_checks {
        let mut stdout = io::BufWriter::new(io::stdout().lock());
        for d in definitions {
            if let Err(error) = writeln!(stdout, "{}\t{}\t{}", d.id, d.mitre_id, d.technique) {
                return if error.kind() == io::ErrorKind::BrokenPipe {
                    Ok(0)
                } else {
                    Err(error.to_string())
                };
            }
        }
        if let Err(error) = stdout.flush() {
            return if error.kind() == io::ErrorKind::BrokenPipe {
                Ok(0)
            } else {
                Err(error.to_string())
            };
        }
        return Ok(0);
    }
    let selected: BTreeSet<_> = opts.checks.iter().map(String::as_str).collect();
    for check in &selected {
        if !definitions.iter().any(|d| selected_by(d, check)) {
            return Err(format!(
                "Unknown persistence check {check:?}; use --list-checks"
            ));
        }
    }
    let profile = Profile::load(opts.profile.as_deref(), definitions)?;
    let selected_definitions: Vec<_> = definitions
        .iter()
        .filter(|d| selected.is_empty() || selected.iter().any(|s| selected_by(d, s)))
        .cloned()
        .collect();
    let enabled_definitions: Vec<_> = selected_definitions
        .iter()
        .filter(|d| profile.enabled(&d.id))
        .cloned()
        .collect();
    let input = opts
        .directory
        .as_ref()
        .or(opts.filepath.as_ref())
        .ok_or("Use -d <EVIDENCE> or -f <HIVE>")?;
    if !input.exists()
        || (opts.directory.is_some() && !input.is_dir())
        || (opts.filepath.is_some() && !input.is_file())
    {
        return Err(format!("Invalid evidence input: {}", input.display()));
    }
    let input = std::fs::canonicalize(input).map_err(|e| e.to_string())?;
    if opts.filepath.is_some() && !registry::is_hive_file(&input) {
        return Err(format!(
            "The input is not a primary registry hive: {}",
            input.display()
        ));
    }
    let mut roots = Vec::new();
    if input.is_dir() {
        for root in crate::auto_parse::evidence_roots(&input, 10).map_err(|e| e.to_string())? {
            roots.push((root, None::<String>));
        }
    }
    if roots.is_empty() {
        roots.push((input.clone(), None));
    }
    if opts.mft.is_some() && roots.len() != 1 {
        return Err(
            "--mft requires evidence from one computer; a single MFT cannot date several hosts"
                .into(),
        );
    }
    if let Some(mft) = &opts.mft
        && !mft.is_file()
    {
        return Err(format!("MFT does not exist: {}", mft.display()));
    }
    let outputs: Vec<_> = opts.output.iter().chain(opts.coverage.iter()).collect();
    for path in &outputs {
        util::check_output(path, false, opts.clobber)?;
        preflight_parent(path)?;
        if input.is_dir()
            && folded_path(&crate::auto_parse::absolute_candidate(path).map_err(|e| e.to_string())?)
                .starts_with(folded_path(&input))
        {
            return Err(
                "Keep persistence findings and coverage outside the evidence directory".into(),
            );
        }
    }
    if let (Some(findings), Some(coverage)) = (&opts.output, &opts.coverage)
        && paths_conflict(findings, coverage)?
    {
        return Err("Findings and coverage must be different output files".into());
    }
    if let (Some(findings), Some(coverage)) = (&opts.output, &opts.coverage) {
        let a = crate::auto_parse::absolute_candidate(findings).map_err(|e| e.to_string())?;
        let b = crate::auto_parse::absolute_candidate(coverage).map_err(|e| e.to_string())?;
        let a = folded_path(&a);
        let b = folded_path(&b);
        if a.starts_with(&b) || b.starts_with(&a) {
            return Err("Findings and coverage cannot contain one another's output paths".into());
        }
    }

    let bar =
        (!opts.data_on_stdout()).then(|| progress::progress_bar(roots.len(), opts.common.no_color));
    let mut findings = Vec::new();
    let mut coverage = Vec::new();
    let mut inputs: Vec<PathBuf> = opts
        .profile
        .iter()
        .chain(opts.mft.iter())
        .cloned()
        .collect();
    let mut errors = 0;
    for (root, computer) in roots {
        if let Some(bar) = &bar {
            bar.set_message(format!("Hunting persistence: {}", root.display()));
        }
        let mut context = match Context::open(&root, opts.no_logs) {
            Ok(context) => context,
            Err(e) => {
                errors += 1;
                let mut row = Coverage::new("context", "failed", e);
                row.computer = computer.unwrap_or_default();
                coverage.push(row);
                if let Some(bar) = &bar {
                    bar.inc(1);
                }
                continue;
            }
        };
        if let Some(computer) = computer {
            context.computer = computer;
        }
        if opts.filepath.is_some() && context.hives.is_empty() {
            return Err(format!(
                "The registry hive could not be parsed: {}",
                context.issues.join("; ")
            ));
        }
        inputs.extend(context.files.iter().map(|f| f.physical.clone()));
        // A standalone hive's replay reads adjacent transaction logs even
        // though those files are not in its single-file inventory.
        let hive_paths: Vec<_> = context.hives.iter().map(|h| h.path.clone()).collect();
        inputs.extend(registry::files_read(&hive_paths, &[]));
        let Scan {
            findings: mut registry_findings,
            coverage: mut host_coverage,
        } = run_checks(&context, &enabled_definitions);
        for definition in &selected_definitions {
            if !profile.enabled(&definition.id) {
                host_coverage.push(Coverage::new(
                    &definition.id,
                    "disabled",
                    "Disabled by the selected profile",
                ));
            }
        }
        registry_findings.par_iter_mut().for_each(|finding| {
            finding.computer.clone_from(&context.computer);
            resolver::enrich(&context, finding);
            profile.classify(finding);
        });
        registry_findings.retain(|f| severity_rank(&f.severity) >= opts.min_severity.rank());
        if !opts.no_timeline {
            for detail in timeline::enrich(&context, &mut registry_findings, opts.mft.as_deref()) {
                let mut row = Coverage::new("timestamp_correlation", "partial", detail);
                row.computer.clone_from(&context.computer);
                coverage.push(row);
            }
        }
        for row in &mut host_coverage {
            row.computer.clone_from(&context.computer);
        }
        for row in &mut host_coverage {
            if !profile.enabled(&row.check_id) {
                row.status = "disabled".into();
                row.detail = "Disabled by the selected profile".into();
            }
            if row.status == "failed" {
                errors += 1;
            }
        }
        for detail in &context.issues {
            let mut row = Coverage::new("context", "partial", detail);
            row.computer.clone_from(&context.computer);
            host_coverage.push(row);
        }
        findings.extend(registry_findings);
        coverage.extend(host_coverage);
        if let Some(bar) = &bar {
            bar.inc(1);
        }
    }
    if let Some(bar) = bar {
        bar.finish_and_clear();
    }
    findings.sort_by(|a, b| {
        a.computer
            .cmp(&b.computer)
            .then_with(|| a.check_id.cmp(&b.check_id))
            .then_with(|| a.path.cmp(&b.path))
            .then_with(|| a.value.cmp(&b.value))
    });
    // A source can be seen through both HKCR views; preserve distinct provenance,
    // but suppress identical rows emitted by the same check and same source.
    findings.dedup_by(|a, b| {
        a.computer == b.computer
            && a.check_id == b.check_id
            && a.path == b.path
            && a.value == b.value
            && a.source_file == b.source_file
    });
    inputs.sort();
    inputs.dedup();
    // Check all targets before opening any output so --coverage cannot clobber
    // findings or evidence through an alias/hard link.
    for output in &outputs {
        for source in &inputs {
            if paths_conflict(output, source)? {
                return Err(format!(
                    "Refusing to overwrite evidence {}",
                    source.display()
                ));
            }
        }
    }
    if let Some(out) = registry::sink(
        opts.output.as_ref(),
        opts.clobber,
        opts.data_on_stdout(),
        || inputs.clone(),
    )? {
        write_findings(
            out,
            opts.output_type.unwrap_or(RegistryOutputType::Csv),
            &findings,
        )
        .map_err(|e| format!("Could not write findings: {e}"))?;
    }
    if let Some(path) = &opts.coverage {
        let out = registry::sink(Some(path), opts.clobber, false, || inputs.clone())?
            .ok_or("Missing coverage output")?;
        let mut writer = csv::Writer::from_writer(out);
        writer
            .write_record(["Computer", "CheckId", "Status", "Detail"])
            .map_err(|e| e.to_string())?;
        for row in &coverage {
            writer
                .write_record([&row.computer, &row.check_id, &row.status, &row.detail])
                .map_err(|e| e.to_string())?;
        }
        writer.flush().map_err(|e| e.to_string())?;
    }
    if !opts.data_on_stdout() {
        println!("Persistence candidates: {}", util::fmt_num(findings.len()));
        println!(
            "Checks: {} complete / {} partial / {} not collected / {} failed / {} disabled",
            count_status(&coverage, "complete"),
            count_status(&coverage, "partial"),
            count_status(&coverage, "not_collected"),
            count_status(&coverage, "failed"),
            count_status(&coverage, "disabled")
        );
        for row in coverage
            .iter()
            .filter(|c| c.status != "complete" && c.status != "disabled")
        {
            println!(
                "{} / {} / {}: {}",
                row.computer, row.check_id, row.status, row.detail
            );
        }
        if let Some(path) = &opts.output {
            println!("Results saved: {}", path.display());
        }
        if let Some(path) = &opts.coverage {
            println!("Coverage saved: {}", path.display());
        }
        if opts.output.is_none() {
            for finding in findings.iter().take(30) {
                println!(
                    "{} | {} | {} | {} | {}",
                    finding.computer,
                    finding.severity,
                    finding.check_id,
                    finding.path,
                    finding.value
                );
            }
            if findings.len() > 30 {
                println!("Use -o <FILE> to save all {} candidates.", findings.len());
            }
        }
    } else {
        for row in coverage
            .iter()
            .filter(|c| c.status != "complete" && c.status != "disabled")
        {
            eprintln!(
                "{} / {} / {}: {}",
                row.computer, row.check_id, row.status, row.detail
            );
        }
    }
    Ok(i32::from(errors > 0))
}

fn run_checks(ctx: &Context, definitions: &[checks::CheckDefinition]) -> Scan {
    // Keep the serial registry-then-file order in the collected results. Each
    // worker owns its findings/coverage and only borrows the loaded evidence.
    let is_file = |d: &&checks::CheckDefinition| checks::FILE_CHECKS.contains(&d.id.as_str());
    let ordered: Vec<_> = definitions
        .iter()
        .filter(|d| !is_file(d))
        .chain(definitions.iter().filter(is_file))
        .collect();
    let scans: Vec<_> = ordered
        .par_iter()
        .map(|definition| {
            let definitions = std::slice::from_ref(*definition);
            if checks::FILE_CHECKS.contains(&definition.id.as_str()) {
                file_checks::run(ctx, definitions)
            } else {
                checks::run_definitions(ctx, definitions)
            }
        })
        .collect();
    let mut result = Scan::default();
    for mut scan in scans {
        result.findings.append(&mut scan.findings);
        result.coverage.append(&mut scan.coverage);
    }
    result
}

fn count_status(coverage: &[Coverage], status: &str) -> usize {
    coverage.iter().filter(|c| c.status == status).count()
}

fn selected_by(definition: &checks::CheckDefinition, token: &str) -> bool {
    definition.id == token
        || definition.mitre_id.eq_ignore_ascii_case(token)
        || definition
            .mitre_id
            .split('.')
            .next()
            .is_some_and(|id| id.eq_ignore_ascii_case(token))
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "INFO" => 0,
        "LOW" => 1,
        "MEDIUM" => 2,
        "HIGH" => 3,
        _ => 2,
    }
}

fn paths_conflict(a: &Path, b: &Path) -> Result<bool, String> {
    let normalized_a = crate::auto_parse::absolute_candidate(a).map_err(|e| e.to_string())?;
    let normalized_b = crate::auto_parse::absolute_candidate(b).map_err(|e| e.to_string())?;
    // Some case-insensitive shared filesystems expose different virtual inode
    // numbers for differently cased spellings of the same file. Require names
    // that remain distinct on these filesystems, including before creation.
    if folded_path(&normalized_a) == folded_path(&normalized_b) {
        return Ok(true);
    }
    if a.exists() && b.exists() {
        return same_file::is_same_file(a, b).map_err(|e| {
            format!(
                "Cannot distinguish {} from {}: {e}",
                a.display(),
                b.display()
            )
        });
    }
    Ok(false)
}

fn folded_path(path: &Path) -> PathBuf {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .collect()
}

fn preflight_parent(path: &Path) -> Result<(), String> {
    if std::fs::symlink_metadata(path).is_ok() && path.canonicalize().is_err() {
        return Err(format!(
            "Output is a dangling or inaccessible link: {}",
            path.display()
        ));
    }
    let mut parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    loop {
        match parent.metadata() {
            Ok(meta) if meta.is_dir() => return Ok(()),
            Ok(_) => {
                return Err(format!(
                    "Output parent is not a directory: {}",
                    parent.display()
                ));
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                parent = parent
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or(Path::new("."));
            }
            Err(e) => {
                return Err(format!(
                    "Cannot inspect output parent {}: {e}",
                    parent.display()
                ));
            }
        }
    }
}

fn write_findings(
    mut out: Box<dyn Write>,
    kind: RegistryOutputType,
    findings: &[Finding],
) -> io::Result<()> {
    match kind {
        RegistryOutputType::Csv => {
            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(out);
            let template = serde_json::to_value(Finding::default())?;
            let fields = template
                .as_object()
                .ok_or_else(|| io::Error::other("Finding is not an object"))?;
            writer.write_record(fields.keys())?;
            for finding in findings {
                writer.serialize(finding)?;
            }
            writer.flush()
        }
        RegistryOutputType::Json | RegistryOutputType::Jsonl => {
            for finding in findings {
                if kind == RegistryOutputType::Json {
                    serde_json::to_writer_pretty(&mut out, finding)?;
                } else {
                    serde_json::to_writer(&mut out, finding)?;
                }
                out.write_all(b"\n")?;
            }
            out.flush()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_csv_has_headers_and_no_fake_finding() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.csv");
        write_findings(
            Box::new(std::fs::File::create(&path).unwrap()),
            RegistryOutputType::Csv,
            &[],
        )
        .unwrap();
        let mut reader = csv::Reader::from_path(path).unwrap();
        assert!(reader.headers().unwrap().iter().any(|h| h == "Computer"));
        assert_eq!(reader.records().count(), 0);
    }
    #[test]
    fn csv_json_have_identical_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("result.csv");
        let f = Finding {
            computer: "HOST".into(),
            value: "comma, quote\" and\nline".into(),
            ..Default::default()
        };
        write_findings(
            Box::new(std::fs::File::create(&path).unwrap()),
            RegistryOutputType::Csv,
            std::slice::from_ref(&f),
        )
        .unwrap();
        let mut reader = csv::Reader::from_path(path).unwrap();
        let json = serde_json::to_value(&f).unwrap();
        assert_eq!(
            reader.headers().unwrap().len(),
            json.as_object().unwrap().len()
        );
        let row: Finding = reader.deserialize().next().unwrap().unwrap();
        assert_eq!(row.value, f.value);
    }
}
