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

//! `auto-parse --vhdx` and `--e01`: every VHDX or E01 disk image in the evidence directory (or
//! the one image given) is mounted read-only with FUSE in a folder of a new temporary directory,
//! named after the image (`host1-triage`), and that directory is parsed as the evidence: each
//! image's Windows systems are found and parsed as in a collection on disk. An E01 image split
//! into segments is one image, found from its first segment (`.E01`, `.Ex01`). The images are
//! unmounted at the end, and on Ctrl+C or SIGTERM.

use super::{Images, run_evidence, shown};
use crate::cli::AutoParseOptions;
use crate::disk_image::{self, Kind};
use crate::util::{self, error_msg, fmt_num, p, p_label};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use vhdx_mount::image_mount::{self, Mounted};

/// The kinds of image the options ask for.
fn kinds(opts: &AutoParseOptions) -> Vec<Kind> {
    let mut kinds = Vec::new();
    if opts.vhdx {
        kinds.push(Kind::Vhdx);
    }
    if opts.e01 {
        kinds.push(Kind::E01);
    }
    kinds
}

/// What the images asked for are, for messages: `VHDX`, `E01`, `VHDX or E01`.
fn kinds_name(kinds: &[Kind]) -> String {
    kinds
        .iter()
        .map(|k| k.name())
        .collect::<Vec<_>>()
        .join(" or ")
}

/// Their file names, for messages: `*.vhdx`, `*.E01, *.Ex01`.
fn kinds_patterns(kinds: &[Kind]) -> String {
    kinds
        .iter()
        .map(|k| match k {
            Kind::Vhdx => "*.vhdx",
            Kind::E01 => "*.E01, *.Ex01",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn run(opts: &AutoParseOptions) -> i32 {
    let no_color = opts.common.no_color;
    let given = &opts.directory;
    let kinds = kinds(opts);
    let (images, root) = if given.is_file() {
        let root = given
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."))
            .to_path_buf();
        (vec![given.clone()], root)
    } else if given.is_dir() {
        let mut images: Vec<PathBuf> = kinds
            .iter()
            .flat_map(|kind| util::find_files(given, kind.name_filter()))
            .collect();
        images.sort();
        (images, given.clone())
    } else {
        error_msg(
            no_color,
            &format!("The evidence directory does not exist: {}", shown(given)),
        );
        return 1;
    };
    if images.is_empty() {
        error_msg(
            no_color,
            &format!(
                "No {} disk images ({}) were found in {}",
                kinds_name(&kinds),
                kinds_patterns(&kinds),
                shown(given)
            ),
        );
        return 1;
    }

    // a new directory for the mount points, removed at the end
    let base = match new_mount_directory() {
        Ok(base) => base,
        Err(e) => {
            error_msg(
                no_color,
                &format!("Could not create a directory for the mount points: {e}"),
            );
            return 1;
        }
    };
    let unmounters = Arc::new(Mutex::new(Vec::new()));
    if let Err(e) =
        image_mount::unmount_on_interrupt(unmounters.clone(), Some(base.clone()), Some(130))
    {
        error_msg(no_color, &format!("Could not handle Ctrl+C: {e}"));
        image_mount::remove_mount_points(&base);
        return 1;
    }

    p_label(
        no_color,
        &format!("{} images: ", kinds_name(&kinds)),
        &fmt_num(images.len()),
    );
    let mut used = HashSet::new();
    let mut mounted: Vec<Mounted> = Vec::new();
    let mut mounts = Vec::new();
    let mut failed = 0usize;
    for image in &images {
        let shown_image = Path::new(".")
            .join(image.strip_prefix(&root).unwrap_or(image))
            .display()
            .to_string();
        let folder = util::unique_name(&root, image, &mut used);
        let point = base.join(&folder);
        let result = std::fs::create_dir(&point).and_then(|()| {
            let opened = disk_image::open(image)?;
            for warning in &opened.warnings {
                error_msg(no_color, &format!("Warning: {shown_image}: {warning}"));
            }
            image_mount::mount_image(opened.image, image, &point, &unmounters)
        });
        match result {
            Ok(mount) => {
                if !mount.problems.is_empty() {
                    failed += 1;
                    for problem in &mount.problems {
                        error_msg(
                            no_color,
                            &format!("Could not read {shown_image}: {problem}"),
                        );
                    }
                }
                p(
                    None,
                    &format!(
                        "  {shown_image}: mounted read-only ({} NTFS volume{})",
                        fmt_num(mount.volumes.len()),
                        if mount.volumes.len() == 1 { "" } else { "s" }
                    ),
                    true,
                );
                mounts.push((point, shown_image));
                mounted.push(mount);
            }
            Err(e) => {
                failed += 1;
                let _ = std::fs::remove_dir(&point);
                error_msg(no_color, &format!("Could not mount {shown_image}: {e}"));
            }
        }
    }
    if mounted.is_empty() {
        image_mount::remove_mount_points(&base);
        return 1;
    }

    let mut inner = opts.clone();
    inner.directory = base.clone();
    inner.vhdx = false;
    inner.e01 = false;
    let images = Images {
        directory_name: root
            .canonicalize()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| ".".into()),
        mounts,
    };
    // an image that couldn't be mounted is a failure, as a parser's is
    let mut code = run_evidence(&inner, Some(&images)).max(i32::from(failed > 0));
    for mount in mounted {
        let point = mount.mount_point.clone();
        if let Err(e) = mount.unmount() {
            error_msg(
                no_color,
                &format!("Could not unmount {}: {e}", point.display()),
            );
            code = 1;
        }
    }
    image_mount::remove_mount_points(&base);
    code
}

/// A new, empty directory for the mount points, in the system's temporary directory.
fn new_mount_directory() -> std::io::Result<PathBuf> {
    let temp = std::env::temp_dir();
    let pid = std::process::id();
    for n in 0..1000u32 {
        let name = if n == 0 {
            format!("juntei-images-{pid}")
        } else {
            format!("juntei-images-{pid}-{n}")
        };
        let path = temp.join(name);
        match std::fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::other("no free directory name"))
}
