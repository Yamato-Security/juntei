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

//! The disk images that `mount` and `auto-parse --vhdx` / `--e01` mount: VHDX files, and E01
//! images (EnCase's Expert Witness Format, also EWF2's Ex01), told apart by their signatures and
//! opened read-only as one disk whose NTFS volumes are served through FUSE. An E01 image split
//! into segments (`.E01`, `.E02`, …) is opened from its first segment; the others are found next
//! to it.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use vhdx_mount::image::Image;

/// The container of a disk image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Vhdx,
    E01,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Vhdx => "VHDX",
            Kind::E01 => "E01",
        }
    }

    /// What finds this kind's files by name (see [`Kind::matches_name`]).
    pub fn name_filter(self) -> fn(&str) -> bool {
        match self {
            Kind::Vhdx => |name| Kind::Vhdx.matches_name(name),
            Kind::E01 => |name| Kind::E01.matches_name(name),
        }
    }

    /// Whether a file name is a disk image's of this kind: for E01, the first segment's only.
    pub fn matches_name(self, name: &str) -> bool {
        let name = name.to_ascii_lowercase();
        match self {
            Kind::Vhdx => name.ends_with(".vhdx"),
            Kind::E01 => name.ends_with(".e01") || name.ends_with(".ex01"),
        }
    }
}

const VHDX: &[u8; 8] = b"vhdxfile";
const EWF1: &[u8; 8] = b"EVF\x09\x0d\x0a\xff\x00";
const EWF2: &[u8; 8] = b"EVF2\x0d\x0a\x81\x00";
const LOGICAL_EWF1: &[u8; 8] = b"LVF\x09\x0d\x0a\xff\x00";
const LOGICAL_EWF2: &[u8; 8] = b"LEF2\x0d\x0a\x81\x00";

/// What a file is, from its first 8 bytes.
pub fn kind(path: &Path) -> io::Result<Kind> {
    let mut head = [0_u8; 8];
    File::open(path)?.read_exact(&mut head).map_err(|e| {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            not_an_image()
        } else {
            e
        }
    })?;
    match &head {
        VHDX => Ok(Kind::Vhdx),
        EWF1 | EWF2 => Ok(Kind::E01),
        LOGICAL_EWF1 | LOGICAL_EWF2 => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "it is a logical evidence file (L01 or Lx01), which holds files, not a disk",
        )),
        _ => Err(not_an_image()),
    }
}

fn not_an_image() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "it is not a VHDX or E01 disk image",
    )
}

/// An opened disk image.
pub struct Opened {
    pub image: Image,
    pub kind: Kind,
    /// The files it is made of (an E01 image's segments).
    pub segments: usize,
    /// What may be missing from what is read: an unfinished acquisition, sectors the acquisition
    /// could not read, damage the reader went past.
    pub warnings: Vec<String>,
}

/// Open a disk image and every NTFS volume in it.
pub fn open(path: &Path) -> io::Result<Opened> {
    match kind(path)? {
        Kind::Vhdx => Ok(Opened {
            image: Image::open(path)?,
            kind: Kind::Vhdx,
            segments: 1,
            warnings: Vec::new(),
        }),
        Kind::E01 => open_e01(path),
    }
}

fn open_e01(path: &Path) -> io::Result<Opened> {
    let invalid =
        |e: ewf_image::EwfError| io::Error::new(io::ErrorKind::InvalidData, e.to_string());
    let ewf = ewf_image::Image::open(path).map_err(invalid)?;
    let info = ewf.info();
    if info.single_files.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "it is a logical evidence file, which holds files, not a disk",
        ));
    }
    let mut warnings = Vec::new();
    if !info.acquisition_complete {
        warnings.push("the image is incomplete: its acquisition did not finish".to_string());
    }
    let unread: u64 = info
        .acquisition_errors
        .iter()
        .map(|range| range.sector_count)
        .sum();
    if unread > 0 {
        warnings.push(format!(
            "the acquisition could not read {} sector{}, which read as zeros",
            crate::util::fmt_num(usize::try_from(unread).unwrap_or(usize::MAX)),
            if unread == 1 { "" } else { "s" }
        ));
    }
    // what lenient opening went past: damaged structures, with the chunks they make unreadable
    warnings.extend(
        ewf.lenient_findings()
            .into_iter()
            .map(|finding| finding.message),
    );
    let segments = info.segment_count;
    let size = ewf.media_size();
    let sector = ewf.bytes_per_sector().unwrap_or(512);
    let image = Image::from_disk(Box::new(ewf.cursor()), size, sector)?;
    Ok(Opened {
        image,
        kind: Kind::E01,
        segments,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert!(Kind::Vhdx.matches_name("host1-triage.vhdx"));
        assert!(Kind::Vhdx.matches_name("HOST.VHDX"));
        assert!(!Kind::Vhdx.matches_name("disk.vhd"));
        assert!(!Kind::Vhdx.matches_name("disk.vhdx.txt"));
        assert!(Kind::E01.matches_name("host1.E01"));
        assert!(Kind::E01.matches_name("host1.e01"));
        assert!(Kind::E01.matches_name("host1.Ex01"));
        // the other segments are found from the first
        assert!(!Kind::E01.matches_name("host1.E02"));
        assert!(!Kind::E01.matches_name("host1.EAA"));
        assert!(!Kind::E01.matches_name("host1.Ex02"));
        assert!(!Kind::E01.matches_name("host1.L01"));
        assert!(!Kind::E01.matches_name("host1.E01.txt"));
    }

    #[test]
    fn signatures() {
        let dir = tempfile::tempdir().unwrap();
        let file = |name: &str, bytes: &[u8]| {
            let path = dir.path().join(name);
            std::fs::write(&path, bytes).unwrap();
            path
        };
        assert_eq!(kind(&file("a", b"vhdxfile....")).unwrap(), Kind::Vhdx);
        assert_eq!(kind(&file("b", EWF1)).unwrap(), Kind::E01);
        assert_eq!(kind(&file("c", EWF2)).unwrap(), Kind::E01);
        let logical = kind(&file("d", LOGICAL_EWF1)).unwrap_err().to_string();
        assert!(logical.contains("logical evidence file"), "{logical}");
        let other = kind(&file("e", b"MZ")).unwrap_err().to_string();
        assert_eq!(other, "it is not a VHDX or E01 disk image");
    }
}
