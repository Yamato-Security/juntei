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

//! Portable read-only VHDX/E01 input. Stage NTFS files in a private workspace because
//! the artifact parsers accept paths, then reuse the ordinary parsing pipeline.
//! No kernel mount, external command, or administrator permission is required.

use super::{ImageProblem, Images, run_evidence, shown};
use crate::cli::AutoParseOptions;
use crate::disk_image::{self, Kind};
use crate::util::{self, error_msg, fmt_num, p, p_label};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vhdx_mount::image::{Entry, Image, ROOT_RECORD, Stat};

fn kinds(opts: &AutoParseOptions) -> Vec<Kind> {
    let mut selected = Vec::new();
    if opts.vhdx {
        selected.push(Kind::Vhdx);
    }
    if opts.e01 {
        selected.push(Kind::E01);
    }
    selected
}
fn kinds_name(kinds: &[Kind]) -> String {
    kinds
        .iter()
        .map(|k| k.name())
        .collect::<Vec<_>>()
        .join(" or ")
}
fn selected_name(name: &str, kinds: &[Kind]) -> bool {
    kinds.iter().any(|kind| kind.matches_name(name))
}

fn discover(given: &Path, kinds: &[Kind]) -> io::Result<(Vec<PathBuf>, PathBuf)> {
    if given.is_file() {
        if !selected_name(
            &given.file_name().unwrap_or_default().to_string_lossy(),
            kinds,
        ) {
            return Err(io::Error::other(format!(
                "The input must be a {} image; provide the first segment of a split E01",
                kinds_name(kinds)
            )));
        }
        return Ok((
            vec![given.to_path_buf()],
            given
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."))
                .to_path_buf(),
        ));
    }
    let mut pending = vec![given.to_path_buf()];
    let mut images = Vec::new();
    let mut seen = HashSet::new();
    while let Some(directory) = pending.pop() {
        crate::interrupt::check()?;
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if directory == given => return Err(error),
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if kind.is_dir() {
                pending.push(path);
            } else if (kind.is_file() || (kind.is_symlink() && path.is_file()))
                && selected_name(&entry.file_name().to_string_lossy(), kinds)
                && let Ok(canonical) = path.canonicalize()
                && seen.insert(canonical)
            {
                images.push(path);
            }
        }
    }
    images.sort();
    Ok((images, given.to_path_buf()))
}

/// The JSONL destination must never replace an input, including through hard links.
fn protect_images(opts: &AutoParseOptions, images: &[PathBuf]) -> io::Result<()> {
    let output_root = super::absolute_candidate(&opts.output)?;
    for image in images {
        let image = image.canonicalize()?;
        if image != output_root && image.starts_with(&output_root) {
            return Err(io::Error::other(
                "The output directory contains an input image or segment",
            ));
        }
    }
    for output in std::iter::once(&opts.output).chain(opts.json_timeline.iter()) {
        match fs::metadata(output) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e),
            Ok(_) => {}
        }
        for image in images {
            if same_file::is_same_file(output, image)? {
                return Err(io::Error::other(
                    "An output overlaps an input image or segment and cannot be overwritten",
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn run(opts: &AutoParseOptions) -> i32 {
    if let Err(error) = crate::interrupt::install() {
        error_msg(
            opts.common.no_color,
            &format!("Could not install Ctrl+C handler: {error}"),
        );
        return 1;
    }
    let code = run_inner(opts);
    if crate::interrupt::requested() {
        error_msg(opts.common.no_color, "Cancelled.");
        130
    } else {
        code
    }
}

fn run_inner(opts: &AutoParseOptions) -> i32 {
    let no_color = opts.common.no_color;
    let kinds = kinds(opts);
    let label = kinds_name(&kinds);
    let (paths, root) = match discover(&opts.directory, &kinds) {
        Ok(found) => found,
        Err(e) => {
            if crate::interrupt::requested() {
                return 130;
            }
            error_msg(
                no_color,
                &format!("Could not search {}: {e}", shown(&opts.directory)),
            );
            return 1;
        }
    };
    if paths.is_empty() {
        error_msg(
            no_color,
            &format!(
                "No {} disk images ({}) were found in {}",
                label,
                kinds
                    .iter()
                    .map(|k| match k {
                        Kind::Vhdx => "*.vhdx",
                        Kind::E01 => "*.E01, *.Ex01",
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                shown(&opts.directory)
            ),
        );
        return 1;
    }
    if let Err(error) = super::validate_image_staging(opts) {
        error_msg(no_color, &error.to_string());
        return 1;
    }
    let sources = match paths
        .iter()
        .map(|path| disk_image::source_files(path))
        .collect::<io::Result<Vec<_>>>()
    {
        Ok(groups) => groups.into_iter().flatten().collect::<Vec<_>>(),
        Err(error) => {
            error_msg(no_color, &error.to_string());
            return 1;
        }
    };
    if let Err(e) = protect_images(opts, &sources) {
        error_msg(no_color, &e.to_string());
        return 1;
    }
    if let Err(e) = util::check_output(&opts.output, true, opts.clobber) {
        error_msg(no_color, &e);
        return 1;
    }
    // Prefer the output filesystem to a small RAM-backed system temp volume.
    // Keep the metadata snapshot there even if an explicitly selected staging
    // filesystem fills, so readable inputs can still reach the parsers.
    let output = match super::absolute_candidate(&opts.output) {
        Ok(path) => path,
        Err(error) => {
            error_msg(
                no_color,
                &format!("Could not resolve output directory: {error}"),
            );
            return 1;
        }
    };
    let parent = output.parent().unwrap_or(Path::new("."));
    if let Err(error) = fs::create_dir_all(parent) {
        error_msg(
            no_color,
            &format!("Could not prepare output parent: {error}"),
        );
        return 1;
    }
    let control = match tempfile::Builder::new()
        .prefix(".juntei-image-control-")
        .tempdir_in(parent)
    {
        Ok(control) => control,
        Err(error) => {
            error_msg(
                no_color,
                &format!("Could not create image control workspace: {error}"),
            );
            return 1;
        }
    };
    let staging_parent = opts.temp_dir.as_deref().unwrap_or(parent);
    let base = match tempfile::Builder::new()
        .prefix("juntei-images-")
        .tempdir_in(staging_parent)
    {
        Ok(base) => base,
        Err(e) => {
            error_msg(
                no_color,
                &format!("Could not create a temporary workspace: {e}"),
            );
            return 1;
        }
    };
    // Files are verified in this workspace before their shared placeholders are filled.
    let workspace = match base.path().canonicalize() {
        Ok(path) => path,
        Err(e) => {
            error_msg(
                no_color,
                &format!("Could not create a temporary workspace: {e}"),
            );
            return 1;
        }
    };
    let mut inputs = Images {
        directory_name: root
            .canonicalize()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| ".".into()),
        mounts: Vec::new(),
        problems: Vec::new(),
        metadata: crate::image_metadata::Snapshot::default(),
        placeholders: HashMap::new(),
        sources: crate::image_files::Sources {
            workspace,
            volumes: Vec::new(),
        },
    };
    p_label(
        no_color,
        &format!("{label} images: "),
        &fmt_num(paths.len()),
    );
    let mut used = HashSet::new();
    let mut storage_full = false;
    for path in paths {
        if crate::interrupt::requested() {
            return 130;
        }
        let source = Path::new(".")
            .join(path.strip_prefix(&root).unwrap_or(&path))
            .display()
            .to_string();
        #[cfg(windows)]
        let source = source.replace('\\', "/");
        if storage_full {
            problem(
                &mut inputs,
                &source,
                "",
                "Not staged because the staging filesystem ran out of space".into(),
            );
            continue;
        }
        let folder = util::unique_name(&root, &path, &mut used);
        let point = base.path().join(folder);
        match disk_image::kind(&path) {
            Ok(kind) if kinds.contains(&kind) => {}
            Ok(kind) => {
                problem(
                    &mut inputs,
                    &source,
                    "",
                    format!("Expected {label}, found {}", kind.name()),
                );
                continue;
            }
            Err(error) => {
                problem(&mut inputs, &source, "", error.to_string());
                continue;
            }
        }
        let opened = match disk_image::open(&path) {
            Ok(opened) if kinds.contains(&opened.kind) => opened,
            Ok(opened) => {
                problem(
                    &mut inputs,
                    &source,
                    "",
                    format!("Expected {label}, found {}", opened.kind.name()),
                );
                continue;
            }
            Err(e) => {
                problem(&mut inputs, &source, "", e.to_string());
                continue;
            }
        };
        let initial_warnings = opened.status.warnings();
        for warning in &initial_warnings {
            problem(&mut inputs, &source, "", warning.clone());
        }
        let image_path = match path.canonicalize() {
            Ok(image_path) => image_path,
            Err(e) => {
                problem(&mut inputs, &source, "", e.to_string());
                continue;
            }
        };
        let image = Arc::new(opened.image);
        for e in &image.problems {
            problem(&mut inputs, &source, "", e.clone());
        }
        p(
            None,
            &format!(
                "  {}: reading natively ({} NTFS volume{})",
                source.escape_debug(),
                fmt_num(image.volumes.len()),
                if image.volumes.len() == 1 { "" } else { "s" }
            ),
            true,
        );
        if opened.segments > 1 {
            p(
                None,
                &format!(
                    "    {} segments: {}",
                    opened.kind.name(),
                    fmt_num(opened.segments)
                ),
                true,
            );
        }
        if let Err(e) = fs::create_dir(&point) {
            problem(&mut inputs, &source, "", e.to_string());
            storage_full = e.kind() == io::ErrorKind::StorageFull;
            continue;
        }
        for index in 0..image.volumes.len() {
            let volume_root = if image.volumes.len() == 1 {
                point.clone()
            } else {
                point.join(format!("Volume{}", index + 1))
            };
            if let Err(e) = stage_volume(
                &image,
                index,
                &image_path,
                &volume_root,
                base.path(),
                &source,
                &point,
                &mut inputs,
            ) {
                if e.kind() == io::ErrorKind::Interrupted {
                    return 130;
                }
                if e.kind() == io::ErrorKind::StorageFull {
                    storage_full = true;
                    break;
                }
                problem(&mut inputs, &source, "", e.to_string());
            }
        }
        // Reader findings append as chunks are read. Keep distinct findings
        // with identical text, without re-reporting the opening snapshot.
        for warning in opened
            .status
            .warnings()
            .into_iter()
            .skip(initial_warnings.len())
        {
            problem(&mut inputs, &source, "", warning);
        }
        // Discovery canonicalizes its root. macOS resolves /var to /private/var
        // and Windows adds a verbatim prefix; provenance must use that same
        // spelling rather than the temporary directory's original alias.
        match point.canonicalize() {
            Ok(point) => inputs.mounts.push((point, source)),
            Err(error) => problem(&mut inputs, &source, "", error.to_string()),
        }
    }
    for entry in &inputs.problems {
        let location = if entry.path.is_empty() {
            entry.image.clone()
        } else {
            format!("{}!/{}", entry.image, entry.path)
        };
        error_msg(
            no_color,
            &format!(
                "Could not read {}: {}",
                location.escape_debug(),
                entry.error.escape_debug()
            ),
        );
    }
    if inputs.mounts.is_empty() {
        return 1;
    }
    let incomplete = !inputs.problems.is_empty();
    let mut inner = opts.clone();
    inner.directory = base.path().to_path_buf();
    inner.vhdx = false;
    inner.e01 = false;
    if crate::interrupt::requested() {
        return 130;
    }
    let metadata_path = control.path().join("SourceMetadata.json");
    if let Err(error) = super::write_pretty_json(&metadata_path, &inputs.metadata) {
        error_msg(
            no_color,
            &format!("Could not preserve original image metadata: {error}"),
        );
        return 1;
    }
    inner.image_source_metadata = Some(metadata_path);
    let sources_path = control.path().join("ImageFiles.json");
    if let Err(error) = super::write_pretty_json(&sources_path, &inputs.sources) {
        error_msg(
            no_color,
            &format!("Could not record the image file sources: {error}"),
        );
        return 1;
    }
    inner.image_files = Some(sources_path);
    let code = run_evidence(&inner, Some(&inputs)).max(i32::from(incomplete));
    if crate::interrupt::requested() {
        return 130;
    }
    if incomplete {
        error_msg(
            no_color,
            &format!(
                "Incomplete {} input: {} read problem(s). Readable results were retained.",
                label,
                inputs.problems.len()
            ),
        );
    }
    // TempDir owns only this run's private copies, never the source evidence.
    if let Err(e) = base.close() {
        error_msg(
            no_color,
            &format!("Could not remove the temporary workspace: {e}"),
        );
        return 1;
    }
    code
}

fn problem(inputs: &mut Images, image: &str, path: &str, error: String) {
    inputs.problems.push(ImageProblem {
        image: image.into(),
        path: path.into(),
        error,
    });
}

/// Refuse names that escape the private tree or alias a Windows device/path.
/// Use the same rules on every OS rather than silently rename evidence.
fn safe_name(name: &str) -> bool {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.ends_with(['.', ' '])
        || name.chars().any(|c| {
            c.is_control() || matches!(c, '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*')
        })
    {
        return false;
    }
    let stem = name
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end()
        .to_uppercase();
    if matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$" | "CONIN$" | "CONOUT$"
    ) {
        return false;
    }
    if let Some(number) = stem
        .strip_prefix("COM")
        .or_else(|| stem.strip_prefix("LPT"))
        && matches!(
            number,
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
        )
    {
        return false;
    }
    true
}

/// Files the parsers choose by name.
fn named_input(name: &str) -> bool {
    super::NAMED_INPUTS.iter().any(|(_, matches)| matches(name))
        || crate::i30::is_i30_name(name)
        || crate::mft::is_mft_name(name)
        || crate::usn_journal::is_journal_name(name)
        || crate::logfile::is_logfile_name(name)
        // Host discovery and persistence-hunter read a collection's client information.
        || name.eq_ignore_ascii_case("client_info.json")
}

/// The longest prefix [`content_input`] inspects.
const HEAD_BYTES: u64 = 4096;

/// Files the parsers recognize by content wherever they are: registry hives, `$I30` files,
/// `$Boot` copies and `$SDS` streams.
fn content_input(name: &str, head: &[u8]) -> bool {
    crate::registry::is_hive_head(name, head)
        || crate::i30::is_i30_head(head)
        || crate::boot::is_boot_head(head)
        || crate::secure_sds::is_sds_head(head)
}

/// Files read together with a selected file beside them: registry transaction logs, and
/// SQLite write-ahead logs and journals.
const COMPANION_SUFFIXES: [&str; 6] = [".log", ".log1", ".log2", "-wal", "-journal", "-shm"];

/// Whether a file is read together with a copied file beside it (`copied`: lower-case names).
fn is_companion(name: &str, copied: &HashSet<String>) -> bool {
    let lower = name.to_lowercase();
    COMPANION_SUFFIXES.iter().any(|suffix| {
        lower
            .strip_suffix(suffix)
            .is_some_and(|base| copied.contains(base))
    })
}

/// A Defender quarantine folder holds `Entries`, `ResourceData` and `Resources`. The parser
/// recognizes it by its entries' content, so copy all three wherever an `Entries` folder is.
const QUARANTINE_FOLDERS: [&str; 3] = ["Entries", "ResourceData", "Resources"];

/// What to do with a file of the image.
enum Plan {
    Copy,
    /// An empty file with the original's timestamps.
    Placeholder(Stat),
}

fn plan(image: &Arc<Image>, volume: usize, entry: &Entry, copy_all: bool, parent: &Path) -> Plan {
    if copy_all
        || named_input(&entry.name)
        || (entry.name.eq_ignore_ascii_case("INFO")
            && crate::recycle_bin::is_legacy_info_path(&parent.join(&entry.name)))
    {
        return Plan::Copy;
    }
    // Copying reports a file that can't be read, as it always has.
    let Ok(stat) = image.volumes[volume].stat(entry.record) else {
        return Plan::Copy;
    };
    if stat.size > 0 {
        let mut head = Vec::new();
        match image
            .file_reader(volume, entry.record)
            .and_then(|reader| reader.take(HEAD_BYTES).read_to_end(&mut head))
        {
            Ok(_) if !content_input(&entry.name, &head) => {}
            _ => return Plan::Copy,
        }
    }
    Plan::Placeholder(stat)
}

/// Stage one NTFS volume. Copy the files the parsers can read, and make every other file an
/// empty placeholder with the original's timestamps: host discovery, paths and existence checks
/// still see the whole filesystem, without the disk space of programs, documents and caches.
/// persistence-hunter reads a placeholder's content from the image when it needs it.
#[allow(clippy::too_many_arguments)]
fn stage_volume(
    image: &Arc<Image>,
    volume: usize,
    image_path: &Path,
    root: &Path,
    scratch: &Path,
    source: &str,
    point: &Path,
    inputs: &mut Images,
) -> io::Result<()> {
    if let Err(error) = fs::create_dir_all(root) {
        problem(inputs, source, "", error.to_string());
        if error.kind() == io::ErrorKind::StorageFull {
            return Err(error);
        }
        return Ok(());
    }
    let canonical_root = match root.canonicalize() {
        Ok(path) => path,
        Err(error) => {
            problem(inputs, source, "", error.to_string());
            return Ok(());
        }
    };
    inputs
        .sources
        .volumes
        .push(crate::image_files::VolumeSource {
            root: canonical_root.clone(),
            image: image_path.to_path_buf(),
            source: source.to_string(),
            path_prefix: root
                .strip_prefix(point)
                .unwrap_or(Path::new(""))
                .to_path_buf(),
            volume,
        });
    // A directory record, its staged directory, and whether to copy every file under it.
    let mut pending = vec![(ROOT_RECORD, root.to_path_buf(), false)];
    let mut visited = HashSet::new();
    // Each record's first staged path, and whether that path holds its content.
    let mut files: HashMap<u64, (PathBuf, bool)> = HashMap::new();
    while let Some((record, directory, copy_all)) = pending.pop() {
        crate::interrupt::check()?;
        let display = directory
            .strip_prefix(point)
            .unwrap_or(&directory)
            .to_string_lossy()
            .replace('\\', "/");
        if !visited.insert(record) {
            problem(
                inputs,
                source,
                &display,
                "Directory record repeats; traversal stopped".into(),
            );
            continue;
        }
        let entries = match image.volumes[volume].list(record) {
            Ok(entries) => entries,
            Err(e) => {
                problem(inputs, source, &display, e.to_string());
                continue;
            }
        };
        let mut names = HashSet::new();
        let mut accepted = Vec::new();
        for entry in entries.iter() {
            let relative = format!("{display}/{}", entry.name)
                .trim_start_matches('/')
                .to_string();
            if !safe_name(&entry.name) || !names.insert(entry.name.to_lowercase()) {
                problem(
                    inputs,
                    source,
                    &relative,
                    "Unsafe or ambiguous filename; it was not copied".into(),
                );
                continue;
            }
            accepted.push((entry, relative));
        }
        // Decide every file first: a companion depends on the files beside it.
        let mut plans = Vec::with_capacity(accepted.len());
        for (entry, _) in &accepted {
            crate::interrupt::check()?;
            plans
                .push((!entry.directory).then(|| plan(image, volume, entry, copy_all, &directory)));
        }
        let copied: HashSet<String> = accepted
            .iter()
            .zip(&plans)
            .filter(|(_, plan)| matches!(plan, Some(Plan::Copy)))
            .map(|((entry, _), _)| entry.name.to_lowercase())
            .collect();
        for ((entry, _), plan) in accepted.iter().zip(plans.iter_mut()) {
            if matches!(plan, Some(Plan::Placeholder(_))) && is_companion(&entry.name, &copied) {
                *plan = Some(Plan::Copy);
            }
        }
        let quarantine = entries
            .iter()
            .any(|e| e.directory && e.name.eq_ignore_ascii_case("Entries"));
        for ((entry, relative), plan) in accepted.into_iter().zip(plans) {
            crate::interrupt::check()?;
            let output = directory.join(&entry.name);
            let Some(plan) = plan else {
                match fs::create_dir(&output) {
                    Ok(()) => {
                        let all = copy_all
                            || (quarantine
                                && QUARANTINE_FOLDERS
                                    .iter()
                                    .any(|name| entry.name.eq_ignore_ascii_case(name)));
                        pending.push((entry.record, output, all));
                    }
                    Err(e) => {
                        problem(inputs, source, &relative, e.to_string());
                        if e.kind() == io::ErrorKind::StorageFull {
                            return Err(e);
                        }
                    }
                }
                continue;
            };
            // Preserve NTFS file identity so hive deduplication behaves as it does when the
            // image is exposed through FUSE.
            let first = files.get(&entry.record).cloned();
            let result = match (&plan, &first) {
                (Plan::Copy, Some((first, true))) | (Plan::Placeholder(_), Some((first, _))) => {
                    fs::hard_link(first, &output).map(|()| None)
                }
                (Plan::Copy, Some((first, false))) => {
                    // A different name may have been staged as a placeholder first. Fill that
                    // inode before linking the selected name; replacing it detaches older names.
                    stage_linked_file(image, volume, entry.record, first, &output, scratch)
                }
                (Plan::Copy, None) => stage_file(image, volume, entry.record, &output, scratch),
                (Plan::Placeholder(stat), None) => placeholder(&output, stat),
            };
            match result {
                Ok(time_error) => {
                    if let Some(error) = time_error {
                        problem(inputs, source, &relative, error);
                    }
                    let has_content =
                        matches!(plan, Plan::Copy) || first.as_ref().is_some_and(|(_, c)| *c);
                    if let Plan::Placeholder(stat) = &plan
                        && !has_content
                        && stat.size > 0
                        && let Ok(within) = output.strip_prefix(root)
                    {
                        inputs
                            .placeholders
                            .insert(canonical_root.join(within), stat.size);
                    }
                    if has_content
                        && (crate::lnk::is_lnk_name(&entry.name)
                            || crate::jump_lists::is_jump_list_name(&entry.name))
                    {
                        let result = image.volumes[volume].stat(entry.record).and_then(|stat| {
                            inputs
                                .metadata
                                .insert(&output, [stat.created.0, stat.modified.0, stat.accessed.0])
                        });
                        if let Err(error) = result {
                            problem(
                                inputs,
                                source,
                                &relative,
                                format!("Original image metadata is unavailable: {error}"),
                            );
                        }
                    }
                    if first.is_none() || (has_content && !first.as_ref().is_some_and(|f| f.1)) {
                        files.insert(entry.record, (output, has_content));
                    }
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::Interrupted {
                        return Err(e);
                    }
                    problem(inputs, source, &relative, e.to_string());
                    if e.kind() == io::ErrorKind::StorageFull {
                        return Err(e);
                    }
                }
            }
        }
    }
    Ok(())
}

fn stage_file(
    image: &Arc<Image>,
    volume: usize,
    record: u64,
    output: &Path,
    scratch: &Path,
) -> io::Result<Option<String>> {
    let stat = image.volumes[volume].stat(record)?;
    let (temporary, time_error) =
        crate::image_files::copy_record(image, volume, record, &stat, scratch)?;
    temporary.persist_noclobber(output).map_err(|e| e.error)?;
    Ok(time_error)
}

fn stage_linked_file(
    image: &Arc<Image>,
    volume: usize,
    record: u64,
    first: &Path,
    output: &Path,
    scratch: &Path,
) -> io::Result<Option<String>> {
    let stat = image.volumes[volume].stat(record)?;
    let (mut temporary, time_error) =
        crate::image_files::copy_record(image, volume, record, &stat, scratch)?;
    let mut file = fs::OpenOptions::new().write(true).open(first)?;
    let filled_time_error = crate::image_files::fill_placeholder(&mut temporary, &mut file, &stat)?;
    fs::hard_link(first, output)?;
    Ok(time_error.or(filled_time_error))
}

/// An empty file standing in for one that was not copied, with its timestamps.
fn placeholder(output: &Path, stat: &Stat) -> io::Result<Option<String>> {
    let file = File::create_new(output)?;
    Ok(crate::image_files::set_times(&file, stat)
        .err()
        .map(|error| format!("Its filesystem timestamps could not be preserved: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_names_do_not_escape_or_alias_devices() {
        for name in [
            "",
            ".",
            "..",
            "../outside",
            "a/b",
            "a\\b",
            "C:foo",
            "CON",
            "nul.txt",
            "COM1.log",
            "LPT²",
            "x.",
            "x ",
            "a:stream",
            "a?",
            "a\0",
        ] {
            assert!(!safe_name(name), "{name:?}");
        }
        for name in [
            "SYSTEM",
            "$MFT",
            "日本語.pf",
            "x%3Ay",
            "CONTEXT.txt",
            "COM10",
        ] {
            assert!(safe_name(name), "{name:?}");
        }
    }

    #[test]
    fn parser_inputs_are_chosen_by_name_content_and_neighbor() {
        for name in [
            "SYSTEM",
            "Amcache.hve",
            "CMD.EXE-6D6290C5.pf",
            "SRUDB.dat",
            "Current.mdb",
            "History",
            "WebCacheV01.dat",
            "Notes.lnk",
            "5f7b5f1e01b83767.automaticDestinations-ms",
            "mail.pst",
            "$MFT",
            "$J",
            "$LogFile",
            "$Boot",
            "$Secure_$SDS",
            "Users_$I30",
            "client_info.json",
        ] {
            assert!(named_input(name), "{name}");
        }
        for name in [
            "notepad.exe",
            "SYSTEM.LOG1",
            "report.docx",
            "pagefile.sys",
            "System.evtx",
        ] {
            assert!(!named_input(name), "{name}");
        }

        let mut hive = vec![0u8; 4096];
        hive[..4].copy_from_slice(b"regf");
        assert!(content_input("NTUSER.DAT", &hive));
        assert!(content_input("renamed-export.bin", &hive));
        // A transaction log starts like its hive; it is copied beside the hive instead.
        assert!(!content_input("NTUSER.DAT.LOG1", &hive));
        let mut indx = vec![0u8; 4096];
        indx[..4].copy_from_slice(b"INDX");
        assert!(content_input("Users_index", &indx));
        let mut boot = vec![0u8; 512];
        boot[3..11].copy_from_slice(b"NTFS    ");
        boot[510..].copy_from_slice(&[0x55, 0xaa]);
        assert!(content_input("volume.img", &boot));
        assert!(!content_input("volume.img", &boot[..511]));
        for head in [&[][..], b"MZ\x90\x00", &[0u8; 4096]] {
            assert!(!content_input("file.bin", head));
        }

        let copied: HashSet<String> = ["ntuser.dat", "history"].map(String::from).into();
        for name in [
            "NTUSER.DAT.LOG1",
            "ntuser.dat.LOG2",
            "NTUSER.DAT.LOG",
            "History-wal",
            "History-journal",
        ] {
            assert!(is_companion(name, &copied), "{name}");
        }
        for name in ["other.LOG1", "setupapi.log", "History", "Historywal"] {
            assert!(!is_companion(name, &copied), "{name}");
        }
    }

    #[test]
    fn recycle_bin_review_stages_windows95_info_indexes() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = tempfile::tempdir().unwrap();
        let image_path = root.path().join("sample.vhdx");
        let gz = File::open(repo.join("test_files/vhdx/sample.vhdx.gz")).unwrap();
        io::copy(
            &mut flate2::read::GzDecoder::new(gz),
            &mut File::create(&image_path).unwrap(),
        )
        .unwrap();
        let image = Arc::new(Image::open(&image_path).unwrap());
        let mut record = ROOT_RECORD;
        let mut entry = None;
        for name in ["C", "Users", "sample", "Data", "small.txt"] {
            let next = image.volumes[0].lookup(record, name).unwrap().unwrap();
            record = next.record;
            entry = Some(next);
        }
        let mut entry = entry.unwrap();
        entry.name = "INFO".into();
        let header = fs::read(repo.join("test_files/recycle_bin/rifiuti2/INFO-95-ja-1")).unwrap();
        assert!(!content_input("INFO", &header));
        assert!(matches!(
            plan(&image, 0, &entry, false, Path::new("C/RECYCLED")),
            Plan::Copy
        ));
        assert!(matches!(
            plan(&image, 0, &entry, false, Path::new("C/Documents")),
            Plan::Placeholder(_)
        ));
    }

    #[test]
    fn staging_copies_parser_inputs_and_reads_placeholders_from_the_image() {
        use sha2::{Digest, Sha256};
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
        let dir = tempfile::tempdir().unwrap();
        let image_path = dir.path().join("sample.vhdx");
        let gz = File::open(repo.join("test_files/vhdx/sample.vhdx.gz")).unwrap();
        io::copy(
            &mut flate2::read::GzDecoder::new(gz),
            &mut File::create(&image_path).unwrap(),
        )
        .unwrap();
        let image_path = image_path.canonicalize().unwrap();
        let image = Arc::new(Image::open(&image_path).unwrap());
        let base = dir.path().join("work");
        fs::create_dir(&base).unwrap();
        let point = base.join("host1");
        let mut inputs = Images {
            directory_name: ".".into(),
            mounts: Vec::new(),
            problems: Vec::new(),
            metadata: crate::image_metadata::Snapshot::default(),
            placeholders: HashMap::new(),
            sources: crate::image_files::Sources {
                workspace: base.canonicalize().unwrap(),
                volumes: Vec::new(),
            },
        };
        stage_volume(
            &image,
            0,
            &image_path,
            &point,
            &base,
            "./sample.vhdx",
            &point,
            &mut inputs,
        )
        .unwrap();
        let problems: Vec<_> = inputs.problems.iter().map(|p| &p.error).collect();
        assert!(problems.is_empty(), "{problems:?}");
        let c = point.join("C");
        let len = |path: &Path| fs::metadata(path).unwrap().len();
        let original = |path: &Path| {
            inputs
                .placeholders
                .get(&path.canonicalize().unwrap())
                .copied()
        };

        // Parser inputs are copied whole, wherever they are.
        for (staged, source) in [
            (
                "Windows/Prefetch/CMD.EXE-6D6290C5.pf",
                "test_files/prefetch/CMD.EXE-6D6290C5.pf",
            ),
            (
                "Windows/System32/LogFiles/Sum/Current.mdb",
                "test_files/ual/server2019/Current.mdb",
            ),
        ] {
            assert_eq!(
                fs::read(c.join(staged)).unwrap(),
                fs::read(repo.join(source)).unwrap()
            );
        }
        assert!(len(&point.join("$MFT")) > 0);
        assert!(len(&point.join("$LogFile")) > 0);

        // A selected name can appear after another name of the same record was left empty.
        // Promoting it must populate the earlier name without changing its file identity.
        let small = c.join("Users/sample/Data/small.txt");
        let selected = c.join("Users/sample/Data/selected.lnk");
        assert_eq!(len(&small), 0);
        let mut small_record = ROOT_RECORD;
        for name in ["C", "Users", "sample", "Data", "small.txt"] {
            small_record = image.volumes[0]
                .lookup(small_record, name)
                .unwrap()
                .unwrap()
                .record;
        }
        stage_linked_file(&image, 0, small_record, &small, &selected, &base).unwrap();
        assert_eq!(fs::read(&small).unwrap(), b"resident content\n");
        assert_eq!(fs::read(&selected).unwrap(), b"resident content\n");
        assert!(same_file::is_same_file(&small, &selected).unwrap());

        // Other files are empty placeholders that keep their size and modification time.
        let evtx = c
            .join("Windows/System32/winevt/Logs/Microsoft-Windows-TaskScheduler%4Operational.evtx");
        let fragmented = c.join("Users/sample/Data/fragmented.bin");
        let compressed = c.join("Users/sample/Data/compressed/text.log");
        let source_evtx = repo.join("test_files/persistence/task-creation.evtx");
        assert_eq!(len(&evtx), 0);
        assert_eq!(original(&evtx), Some(len(&source_evtx)));
        assert_eq!(len(&fragmented), 0);
        assert_eq!(original(&fragmented), Some(8 * (16 * 1024 + 123)));
        let volume = &image.volumes[0];
        let mut record = ROOT_RECORD;
        for name in ["C", "Users", "sample", "Data", "fragmented.bin"] {
            record = volume.lookup(record, name).unwrap().unwrap().record;
        }
        let modified = crate::image_metadata::system_time(volume.stat(record).unwrap().modified.0);
        assert_eq!(fs::metadata(&fragmented).unwrap().modified().ok(), modified);
        // An empty file needs no size.
        let empty = c.join("Users/sample/Data/empty.bin");
        assert_eq!((len(&empty), original(&empty)), (0, None));

        // persistence-hunter reads placeholders from the image.
        let sources = dir.path().join("ImageFiles.json");
        super::super::write_pretty_json(&sources, &inputs.sources).unwrap();
        crate::image_files::install(&sources).unwrap();
        assert_eq!(
            crate::image_files::len(&fragmented, 0),
            8 * (16 * 1024 + 123)
        );
        let sha256 = |path: &Path| crate::util::hex_lower(&Sha256::digest(fs::read(path).unwrap()));
        // Hydration must not detach other names of the same NTFS file. Consumers deduplicate
        // hives and payloads by identity, not just by their bytes.
        let alias = c.join("Users/sample/Data/fragmented-alias.bin");
        fs::hard_link(&fragmented, &alias).unwrap();
        crate::image_files::materialize(&fragmented).unwrap();
        assert_eq!(
            sha256(&fragmented),
            "dad53fd22d62e1acbf48ababc172c8eb3ffb45486b261fb960a2990199a8e459"
        );
        assert_eq!(sha256(&alias), sha256(&fragmented));
        assert!(same_file::is_same_file(&alias, &fragmented).unwrap());
        assert_eq!(fs::metadata(&fragmented).unwrap().modified().ok(), modified);
        crate::image_files::materialize(&compressed).unwrap();
        assert_eq!(
            sha256(&compressed),
            "cc3c52b090233f44ecfc2dbd90adc6c787e071872ca488fa96eae24af8162f98"
        );
        crate::image_files::materialize(&evtx).unwrap();
        assert_eq!(fs::read(&evtx).unwrap(), fs::read(&source_evtx).unwrap());
        // Files with content, and files outside the staged volumes, are left alone.
        let prefetch = c.join("Windows/Prefetch/CMD.EXE-6D6290C5.pf");
        let before = fs::metadata(&prefetch).unwrap().modified().unwrap();
        crate::image_files::materialize(&prefetch).unwrap();
        assert_eq!(fs::metadata(&prefetch).unwrap().modified().unwrap(), before);
        let outside = dir.path().join("outside.txt");
        File::create(&outside).unwrap();
        crate::image_files::materialize(&outside).unwrap();
        assert_eq!(len(&outside), 0);
        crate::image_files::materialize(&empty).unwrap();
        assert_eq!(len(&empty), 0);
        // A second hunter must wait for a writer even after its first nonzero bytes exist.
        // Each opens its own handle, as the CSV and JSON-timeline child processes do.
        let mut writing = fs::OpenOptions::new().write(true).open(&small).unwrap();
        writing.lock().unwrap();
        writing.set_len(0).unwrap();
        io::Write::write_all(&mut writing, b"resident ").unwrap();
        std::thread::scope(|scope| {
            let (started, begin) = std::sync::mpsc::channel();
            let (finished, done) = std::sync::mpsc::channel();
            let small = &small;
            let reader = scope.spawn(move || {
                started.send(()).unwrap();
                crate::image_files::materialize(small).unwrap();
                finished.send(fs::read(small).unwrap()).unwrap();
            });
            begin.recv().unwrap();
            assert!(
                done.recv_timeout(std::time::Duration::from_millis(50))
                    .is_err()
            );
            io::Write::write_all(&mut writing, b"content\n").unwrap();
            writing.unlock().unwrap();
            assert_eq!(
                done.recv_timeout(std::time::Duration::from_secs(10))
                    .unwrap(),
                b"resident content\n"
            );
            reader.join().unwrap();
        });
        // No temporary copy is left in the workspace.
        let leftovers: Vec<_> = fs::read_dir(&base)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .filter(|name| name != "host1")
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
    }
}
