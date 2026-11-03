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

//! Chronological, semantically labeled projections of existing parsed evidence.
mod adapters;
mod input;
mod model;
mod output;
mod sort;
#[cfg(test)]
mod tests;

use crate::cli::SuperTimelineOptions;
use crate::color::JunteiColor;
use crate::{progress, util};
use model::{ArtifactCounts, Counts, Diagnostic, SortRow};
use serde_json::{Value, json};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

struct Scratch(PathBuf);
impl Scratch {
    fn new(parent: &Path) -> io::Result<Self> {
        fs::create_dir_all(parent)?;
        let clock = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        for index in 0..1000 {
            let path = parent.join(format!(
                ".juntei-super-timeline-{}-{clock}-{index}",
                std::process::id()
            ));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::other(
            "Could not create private timeline temporary directory",
        ))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Targets {
    timeline: PathBuf,
    untimed: PathBuf,
    diagnostics: PathBuf,
    summary: PathBuf,
}
impl Targets {
    fn new(path: &Path) -> io::Result<Self> {
        if fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(io::Error::other(
                "Timeline output must not be a symbolic link",
            ));
        }
        let path = crate::auto_parse::absolute_candidate(path)?;
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| io::Error::other("Timeline output needs a filename"))?;
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("Timeline output needs a parent directory"))?;
        Ok(Self {
            untimed: parent.join(format!("{stem}-Untimed.jsonl")),
            diagnostics: parent.join(format!("{stem}-Diagnostics.jsonl")),
            summary: parent.join(format!("{stem}-Summary.json")),
            timeline: path,
        })
    }
    fn all(&self) -> [&Path; 4] {
        [
            &self.timeline,
            &self.untimed,
            &self.diagnostics,
            &self.summary,
        ]
    }
    fn check(&self, inventory: &input::Inventory, clobber: bool) -> io::Result<()> {
        for target in self.all() {
            if let Ok(meta) = fs::symlink_metadata(target) {
                if meta.file_type().is_symlink() || !meta.is_file() {
                    return Err(io::Error::other(format!(
                        "Output must be a regular file: {}",
                        target.display()
                    )));
                }
                if !clobber {
                    return Err(io::Error::other(format!(
                        "Output exists: {}. Use -C to replace generated files.",
                        target.display()
                    )));
                }
            }
            crate::registry::check_not_read(target, || inventory.protected.clone())
                .map_err(io::Error::other)?;
            for root in &inventory.evidence_roots {
                let root = crate::auto_parse::absolute_candidate(root)?;
                if target.starts_with(&root) || root.starts_with(target) {
                    return Err(io::Error::other(format!(
                        "Timeline outputs must be outside evidence directories: {}",
                        root.display()
                    )));
                }
            }
        }
        Ok(())
    }
    fn referenced_source(&self, source: &str) -> io::Result<()> {
        let path = Path::new(source);
        if !path.is_absolute() {
            return Ok(());
        }
        for target in self.all() {
            if target == path {
                return Err(io::Error::other(
                    "Timeline output overlaps a referenced raw source",
                ));
            }
            if path.is_file() {
                crate::registry::check_not_read(target, || vec![path.to_path_buf()])
                    .map_err(io::Error::other)?;
                if let Some(parent) = path.parent() {
                    let root = fs::canonicalize(parent)?;
                    if target.starts_with(&root) {
                        return Err(io::Error::other(format!(
                            "Output is inside a referenced raw-source directory: {}",
                            root.display()
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

struct Completed {
    chunks: Vec<PathBuf>,
    diagnostic_file: PathBuf,
    counts: ArtifactCounts,
    largest: usize,
}

fn process(
    input: &input::Input,
    opts: &SuperTimelineOptions,
    policy: &adapters::Policy,
    targets: &Targets,
    folder: &Path,
    budget: usize,
) -> io::Result<Completed> {
    let mut buffer = sort::Buffer::new(folder, input.index, budget);
    let diagnostic_file = folder.join(format!("input-{}-diagnostics.jsonl", input.index));
    let mut diagnostics = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&diagnostic_file)?,
    );
    let mut counts = ArtifactCounts::new();
    let mut previous_source = String::new();
    input::records(input, opts.max_record_size * 1024 * 1024, |value, ctx| {
        if !opts.artifacts.is_empty()
            && !opts.artifacts.iter().any(|a| {
                a.eq_ignore_ascii_case(&ctx.artifact)
                    || (a == "prefetch" && ctx.artifact == "prefetch-timeline")
            })
        {
            return Ok(());
        }
        if ctx.artifact.is_empty() {
            return Err(io::Error::other(
                "Untagged records need --input-artifact; use native auto-parse JSONL with Artifact/Computer metadata",
            ));
        }
        if !adapters::ARTIFACTS.contains(&ctx.artifact.as_str()) {
            return Err(io::Error::other(format!(
                "No timestamp adapter for artifact '{}'",
                ctx.artifact
            )));
        }
        let (source, raw) = adapters::source(value, ctx);
        if raw && source != previous_source {
            targets.referenced_source(&source)?;
            previous_source = source;
        }
        let projected = adapters::project_into(value, ctx, policy, |item| {
            match item {
                adapters::Item::Event(event) => {
                    if event
                        .event
                        .details
                        .get("RawSourceAvailable")
                        .and_then(Value::as_bool)
                        == Some(true)
                        && event.event.source_file != previous_source
                    {
                        targets.referenced_source(&event.event.source_file)?;
                        previous_source = event.event.source_file.clone();
                    }
                    buffer.push(event)?;
                }
                adapters::Item::Diagnostic(diagnostic) => {
                    serde_json::to_writer(&mut diagnostics, &diagnostic)?;
                    diagnostics.write_all(b"\n")?;
                }
            }
            Ok(())
        })?;
        let name = if ctx.artifact == "prefetch-timeline" {
            "prefetch"
        } else {
            &ctx.artifact
        };
        counts.entry(name.into()).or_default().add(&projected);
        Ok(())
    })?;
    diagnostics.flush()?;
    buffer.flush()?;
    Ok(Completed {
        chunks: buffer.chunks,
        diagnostic_file,
        counts,
        largest: buffer.largest_row,
    })
}

fn execute(opts: &SuperTimelineOptions) -> io::Result<Value> {
    let started = Instant::now();
    let inventory = input::discover(opts)?;
    for artifact in opts.artifacts.iter().chain(opts.input_artifact.iter()) {
        if !adapters::ARTIFACTS.contains(&artifact.as_str()) {
            return Err(io::Error::other(format!(
                "Unknown --artifact/--input-artifact '{artifact}'"
            )));
        }
    }
    let policy = adapters::Policy::new(opts).map_err(io::Error::other)?;
    let targets = Targets::new(&opts.output)?;
    targets.check(&inventory, opts.clobber)?;
    let temp_base = crate::auto_parse::absolute_candidate(
        &opts.temp_dir.clone().unwrap_or_else(std::env::temp_dir),
    )?;
    for evidence in &inventory.evidence_roots {
        let root = crate::auto_parse::absolute_candidate(evidence)?;
        if temp_base.starts_with(&root) {
            return Err(io::Error::other(
                "Temporary files must be outside evidence directories",
            ));
        }
    }
    let scratch = Scratch::new(&temp_base)?;
    let memory = opts.memory_limit * 1024 * 1024;
    let available = std::thread::available_parallelism().map_or(1, usize::from);
    let workers = opts
        .threads
        .unwrap_or(available)
        .min(inventory.inputs.len())
        .min((memory / (4 * 1024 * 1024)).max(1))
        .max(1);
    let per_worker = memory / workers;
    util::p_label(
        opts.common.no_color,
        "Input files: ",
        &util::fmt_num(inventory.inputs.len()),
    );
    util::p_label(
        opts.common.no_color,
        "Parallel workers: ",
        &util::fmt_num(workers),
    );
    util::p(None, "", true);
    util::p(
        JunteiColor::Orange.rdg(opts.common.no_color),
        "Normalizing timestamps.",
        true,
    );
    util::p(None, "", true);
    let bar = progress::progress_bar(inventory.inputs.len(), opts.common.no_color);
    let next = AtomicUsize::new(0);
    let completed = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(input) = inventory.inputs.get(index) else {
                        break;
                    };
                    bar.set_message(progress::message(&input.path));
                    let result = process(input, opts, &policy, &targets, &scratch.0, per_worker)
                        .map_err(|e| io::Error::other(format!("{}: {e}", input.path.display())));
                    completed.lock().unwrap().push((index, result));
                    bar.inc(1);
                }
            });
        }
    });
    let mut completed = completed.into_inner().unwrap();
    completed.sort_by_key(|(i, _)| *i);
    if completed.iter().any(|(_, r)| r.is_err()) {
        bar.abandon_with_message("Input failed");
        let errors = completed
            .into_iter()
            .filter_map(|(_, r)| r.err())
            .map(|e| e.to_string())
            .collect::<Vec<_>>();
        return Err(io::Error::other(format!(
            "{}; no timeline or companions were replaced",
            errors.join("\n")
        )));
    }
    bar.finish_with_message(progress::finished_message(opts.common.no_color));
    let completed: Vec<_> = completed.into_iter().map(|(_, r)| r.unwrap()).collect();
    let mut counts = ArtifactCounts::new();
    let mut chunks = Vec::new();
    let mut largest = 0;
    for result in &completed {
        for (name, n) in &result.counts {
            counts.entry(name.clone()).or_default().add(n);
        }
        chunks.extend(result.chunks.iter().cloned());
        largest = largest.max(result.largest);
    }
    let original_chunks = chunks.len();
    util::p_label(
        opts.common.no_color,
        "Sorting chunks: ",
        &util::fmt_num(original_chunks),
    );
    let (chunks, passes) = sort::reduce(chunks, &scratch.0, memory, largest)?;
    // Outputs are staged only after every input has been read and checked. A private sibling
    // directory keeps final replacements on the same filesystem as their destination.
    let parent = targets.timeline.parent().unwrap();
    let staged = Scratch::new(parent)?;
    let timeline = staged.0.join("timeline");
    let mut writer = output::Writer::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&timeline)?,
        opts.output_type,
        opts.profile,
    )?;
    let mut pending: Option<SortRow> = None;
    let mut written = 0u64;
    sort::merge(&chunks, |row| {
        if let Some(previous) = &mut pending {
            if previous.time == row.time && previous.identity == row.identity {
                output::duplicate(previous, &row);
                counts
                    .entry(row.event.artifact.clone())
                    .or_default()
                    .duplicate_events += 1;
                return Ok(());
            }
            writer.event(&previous.event)?;
            written += 1;
        }
        pending = Some(row);
        Ok(())
    })?;
    if let Some(row) = pending {
        writer.event(&row.event)?;
        written += 1;
    }
    writer.finish()?;
    let untimed = staged.0.join("untimed");
    let diagnostics = staged.0.join("diagnostics");
    let mut untimed_writer = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&untimed)?,
    );
    let mut diagnostic_writer = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&diagnostics)?,
    );
    for result in &completed {
        for diagnostic in serde_json::Deserializer::from_reader(BufReader::new(File::open(
            &result.diagnostic_file,
        )?))
        .into_iter::<Diagnostic>()
        {
            let diagnostic = diagnostic?;
            let writer = if diagnostic.reason.starts_with("No usable mapped") {
                &mut untimed_writer
            } else {
                &mut diagnostic_writer
            };
            serde_json::to_writer(&mut *writer, &diagnostic)?;
            writer.write_all(b"\n")?;
        }
    }
    untimed_writer.flush()?;
    diagnostic_writer.flush()?;
    drop(untimed_writer);
    drop(diagnostic_writer);
    let mut total = Counts::default();
    for value in counts.values() {
        total.add(value);
    }
    let summary = json!({
        "SchemaVersion":1,"Complete":true,"JunteiVersion":env!("CARGO_PKG_VERSION"),
        "Output":targets.timeline,"Untimed":targets.untimed,"Diagnostics":targets.diagnostics,
        "Format":match opts.output_type { crate::cli::TimelineOutputType::Csv=>"csv",crate::cli::TimelineOutputType::Jsonl=>"jsonl" },
        "Profile":match opts.profile { crate::cli::TimelineProfile::Native=>"native",crate::cli::TimelineProfile::Timesketch=>"timesketch" },
        "Workers":workers,"SortMemoryBudgetMiB":opts.memory_limit,"LargestSerializedSortRowBytes":largest,
        "InitialSortChunks":original_chunks,"MergePasses":passes,"OutputEvents":written,
        "Counts":total,"Artifacts":counts,"ElapsedSeconds":started.elapsed().as_secs_f64(),
        "SourceRecordsRetainedInInput":true,"InputFiles":inventory.inputs.iter().map(|i|i.path.clone()).collect::<Vec<_>>(),
        "TimestampSemantics":"Typed timestamps are recorded evidence, not a verdict that an action or attack occurred"
    });
    let summary_path = staged.0.join("summary");
    let mut summary_writer = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&summary_path)?,
    );
    serde_json::to_writer_pretty(&mut summary_writer, &summary)?;
    summary_writer.write_all(b"\n")?;
    summary_writer.flush()?;
    drop(summary_writer);
    targets.check(&inventory, opts.clobber)?;
    // Each file is atomically replaced; the timeline is published last. A malformed input or
    // failed sort never replaces any previous output. This is not a multi-file transaction.
    for (source, target) in [
        (&untimed, &targets.untimed),
        (&diagnostics, &targets.diagnostics),
        (&summary_path, &targets.summary),
        (&timeline, &targets.timeline),
    ] {
        crate::registry::check_not_read(target, || inventory.protected.clone())
            .map_err(io::Error::other)?;
        fs::rename(source, target)?;
    }
    Ok(summary)
}

pub fn run(opts: &SuperTimelineOptions) -> i32 {
    match execute(opts) {
        Ok(summary) => {
            util::p(None, "", true);
            util::p_label(
                opts.common.no_color,
                "Timeline events: ",
                &summary["OutputEvents"].to_string(),
            );
            util::p_label(
                opts.common.no_color,
                "Duplicate events consolidated: ",
                &summary["Counts"]["DuplicateEvents"].to_string(),
            );
            util::p_label(
                opts.common.no_color,
                "Untimed records: ",
                &summary["Counts"]["UntimedRecords"].to_string(),
            );
            util::p_label(
                opts.common.no_color,
                "Results saved: ",
                &opts.output.to_string_lossy(),
            );
            0
        }
        Err(error) => {
            util::error_msg(opts.common.no_color, &error.to_string());
            1
        }
    }
}
