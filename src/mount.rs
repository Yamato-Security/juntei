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

//! `mount`: a disk image's NTFS volumes (a VHDX or E01 image), read-only at a directory through
//! FUSE, for as long as the command runs, so other tools (Hayabusa, Juntei's own commands) can
//! read the files without the operating system supporting the image or the file system, and
//! without root.

use crate::cli::MountOptions;
use crate::util::error_msg;

/// What to use where FUSE isn't available.
#[cfg(not(target_os = "linux"))]
pub const NO_FUSE: &str = "OS-visible mounting needs Linux FUSE. For native parsing on Linux, macOS or Windows without administrator privileges, use: juntei auto-parse --vhdx --e01 -d <IMAGE_OR_DIR> -o <OUTPUT>";

/// Run the command; returns the exit status.
#[cfg(target_os = "linux")]
pub fn run(opts: &MountOptions) -> i32 {
    use crate::color::JunteiColor;
    use crate::disk_image;
    use crate::util::{self, fmt_num, p, p_label};
    use std::sync::{Arc, Mutex};
    use vhdx_mount::image_mount;

    let no_color = opts.common.no_color;
    if !opts.image.is_file() {
        error_msg(
            no_color,
            &format!("The image does not exist: {}", opts.image.display()),
        );
        return 1;
    }
    if !opts.mount_point.is_dir() {
        error_msg(
            no_color,
            &format!(
                "The mount point is not a directory: {} (create an empty directory first)",
                opts.mount_point.display()
            ),
        );
        return 1;
    }
    let opened = match disk_image::open(&opts.image) {
        Ok(opened) => opened,
        Err(e) => {
            error_msg(
                no_color,
                &format!("Could not open {}: {e}", opts.image.display()),
            );
            return 1;
        }
    };
    let image = opened.image;
    // an unreadable volume, or damage the image was read past: what is mounted may lack files
    let initial = opened.status.warnings();
    let partial = !image.problems.is_empty() || !initial.is_empty();
    for problem in &image.problems {
        error_msg(no_color, &format!("Could not read {problem}"));
    }
    for warning in &initial {
        error_msg(no_color, &format!("Warning: {warning}"));
    }
    p_label(no_color, "Image: ", &opts.image.display().to_string());
    let format = if opened.segments > 1 {
        format!(
            "{} ({} segments)",
            opened.kind.name(),
            fmt_num(opened.segments)
        )
    } else {
        opened.kind.name().to_string()
    };
    p_label(no_color, "Format: ", &format);
    p_label(no_color, "Disk size: ", &util::size_display(image.size));
    let volumes: Vec<String> = image
        .volumes
        .iter()
        .map(|v| {
            format!(
                "{} at byte {}",
                util::size_display(v.len),
                fmt_num(v.start as usize)
            )
        })
        .collect();
    p_label(
        no_color,
        "NTFS volumes: ",
        &format!("{} ({})", fmt_num(volumes.len()), volumes.join(", ")),
    );
    let several = image.volumes.len() > 1;
    let mut session = match image_mount::new_session(image, &opts.image, &opts.mount_point) {
        Ok(session) => session,
        Err(e) => {
            error_msg(
                no_color,
                &format!("Could not mount at {}: {e}", opts.mount_point.display()),
            );
            return 1;
        }
    };
    let unmounters = Arc::new(Mutex::new(vec![session.unmount_callable()]));
    if let Err(e) = image_mount::unmount_on_interrupt(unmounters, None, None) {
        error_msg(no_color, &format!("Could not handle Ctrl+C: {e}"));
        return 1;
    }
    p(None, "", true);
    p(
        JunteiColor::Green.rdg(no_color),
        &format!("Mounted read-only at {}", opts.mount_point.display()),
        true,
    );
    if several {
        p(
            None,
            "Each volume is a directory: Volume1, Volume2, ...",
            true,
        );
    }
    p(
        None,
        &format!(
            "Press Ctrl+C to unmount (or run: fusermount3 -u {})",
            opts.mount_point.display()
        ),
        true,
    );
    p(None, "", true);
    if let Err(e) = session.run() {
        error_msg(no_color, &format!("The mount stopped: {e}"));
        return 1;
    }
    p_label(
        no_color,
        "Unmounted: ",
        &opts.mount_point.display().to_string(),
    );
    let final_warnings = opened.status.warnings();
    for warning in &final_warnings {
        if !initial.contains(warning) {
            error_msg(no_color, &format!("Warning: {warning}"));
        }
    }
    i32::from(partial || !final_warnings.is_empty())
}

/// Run the command; returns the exit status.
#[cfg(not(target_os = "linux"))]
pub fn run(opts: &MountOptions) -> i32 {
    error_msg(opts.common.no_color, NO_FUSE);
    1
}
