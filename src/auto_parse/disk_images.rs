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
use std::fs::{self, File, FileTimes};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vhdx_mount::image::{Image, ROOT_RECORD};

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
    let mut inputs = Images {
        directory_name: root
            .canonicalize()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| ".".into()),
        mounts: Vec::new(),
        problems: Vec::new(),
        metadata: crate::image_metadata::Snapshot::default(),
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

fn stage_volume(
    image: &Arc<Image>,
    volume: usize,
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
    let mut pending = vec![(ROOT_RECORD, root.to_path_buf())];
    let mut visited = HashSet::new();
    let mut files = HashMap::new();
    while let Some((record, directory)) = pending.pop() {
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
        for entry in entries.iter() {
            crate::interrupt::check()?;
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
            let output = directory.join(&entry.name);
            if entry.directory {
                match fs::create_dir(&output) {
                    Ok(()) => pending.push((entry.record, output)),
                    Err(e) => {
                        problem(inputs, source, &relative, e.to_string());
                        if e.kind() == io::ErrorKind::StorageFull {
                            return Err(e);
                        }
                    }
                }
            } else {
                // Preserve NTFS file identity so hive deduplication behaves as
                // it does when the image is exposed through FUSE.
                let result = if let Some(first) = files.get(&entry.record) {
                    fs::hard_link(first, &output).map(|()| None)
                } else {
                    stage_file(image, volume, entry.record, &output, scratch)
                };
                match result {
                    Ok(time_error) => {
                        if let Some(error) = time_error {
                            problem(inputs, source, &relative, error);
                        }
                        if crate::lnk::is_lnk_name(&entry.name)
                            || crate::jump_lists::is_jump_list_name(&entry.name)
                        {
                            let result =
                                image.volumes[volume].stat(entry.record).and_then(|stat| {
                                    inputs.metadata.insert(
                                        &output,
                                        [stat.created.0, stat.modified.0, stat.accessed.0],
                                    )
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
                        files.insert(entry.record, output);
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
    let mut reader = image.file_reader(volume, record)?;
    let mut temporary = tempfile::NamedTempFile::new_in(scratch)?;
    let file = temporary.as_file_mut();
    copy_sparse(&mut reader, file, stat.size)?;
    // Preserve the filesystem mtime used by persistence-hunter; parsed artifact
    // timestamps still come from the evidence bytes, never these staging files.
    let mut times = FileTimes::new();
    if let Some(t) = crate::image_metadata::system_time(stat.modified.0) {
        times = times.set_modified(t);
    }
    if let Some(t) = crate::image_metadata::system_time(stat.accessed.0) {
        times = times.set_accessed(t);
    }
    let time_error = file.set_times(times).err().map(|error|
        format!("File content was retained, but its filesystem timestamps could not be preserved: {error}"));
    temporary.persist_noclobber(output).map_err(|e| e.error)?;
    Ok(time_error)
}

fn copy_sparse(reader: &mut impl Read, output: &mut File, expected: u64) -> io::Result<()> {
    let mut buffer = vec![0; 256 * 1024];
    let mut copied = 0u64;
    loop {
        crate::interrupt::check()?;
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        copied = copied
            .checked_add(count as u64)
            .ok_or_else(|| io::Error::other("File length overflow"))?;
        if copied > expected {
            return Err(io::Error::other("NTFS stream exceeds its declared length"));
        }
        if buffer[..count].iter().all(|b| *b == 0) {
            output.seek(SeekFrom::Current(count as i64))?;
        } else {
            output.write_all(&buffer[..count])?;
        }
    }
    if copied != expected {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "NTFS stream ended before its declared length",
        ));
    }
    output.set_len(expected)
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
    fn sparse_copy_preserves_bytes_and_rejects_short_or_long_reads() {
        let mut bytes = vec![0; 600_001];
        bytes[42] = 1;
        bytes[599_999] = 2;
        let mut file = tempfile::tempfile().unwrap();
        copy_sparse(&mut bytes.as_slice(), &mut file, bytes.len() as u64).unwrap();
        file.rewind().unwrap();
        let mut actual = Vec::new();
        file.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, bytes);
        assert!(copy_sparse(&mut b"x".as_slice(), &mut file, 2).is_err());
        assert!(copy_sparse(&mut b"xx".as_slice(), &mut file, 1).is_err());
    }
}
