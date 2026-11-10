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

//! The disk images that `mount` and `auto-parse --vhdx` / `--e01` read: VHDX files, and E01
//! images (EnCase's Expert Witness Format, also EWF2's Ex01), told apart by their signatures and
//! opened read-only as one disk for native parsing or Linux FUSE mounting. An E01 image split
//! into segments (`.E01`, `.E02`, …) is opened from its first segment; the others are found next
//! to it.

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
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
    /// Shared EWF reader state, including defects discovered during later reads.
    pub status: ReadStatus,
}

pub struct ReadStatus(Option<ewf_image::Image>);

impl ReadStatus {
    pub fn warnings(&self) -> Vec<String> {
        let Some(ewf) = &self.0 else {
            return Vec::new();
        };
        let info = ewf.info();
        let mut warnings = Vec::new();
        if !info.acquisition_complete {
            warnings.push("the image is incomplete: its acquisition did not finish".into());
        }
        let unread = unread_sector_count(&info.acquisition_errors);
        if unread > 0 {
            warnings.push(format!(
                "the acquisition could not read {} sector{}, which read as zeros",
                crate::util::fmt_num(usize::try_from(unread).unwrap_or(usize::MAX)),
                if unread == 1 { "" } else { "s" }
            ));
        }
        warnings.extend(
            ewf.lenient_findings()
                .into_iter()
                .map(|finding| finding.message),
        );
        warnings
    }
}

fn unread_sector_count(ranges: &[ewf_image::AcquisitionError]) -> u64 {
    ranges
        .iter()
        .fold(0, |count, range| count.saturating_add(range.sector_count))
}

/// Include even damaged secondary segments before opening any user outputs.
/// The EWF reader can reject a damaged segment, but it remains source evidence.
pub fn source_files(path: &Path) -> io::Result<Vec<PathBuf>> {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if !extension.eq_ignore_ascii_case("e01") && !extension.eq_ignore_ascii_case("ex01") {
        return Ok(vec![path.to_path_buf()]);
    }
    let path = actual_segment_name(path)?;
    let path = path.as_path();
    let mut files = vec![path.to_path_buf()];
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let v2 = extension.eq_ignore_ascii_case("ex01");
    if !v2 && !extension.eq_ignore_ascii_case("e01") {
        return Ok(files);
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    for entry in std::fs::read_dir(parent)? {
        let entry = entry?;
        let candidate = entry.path();
        if candidate.file_stem() != path.file_stem() || !candidate.is_file() {
            continue;
        }
        if candidate
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|ext| segment_extension(ext, v2))
        {
            files.push(candidate);
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

/// Preserve the selected segment family while resolving case aliases on a
/// case-insensitive filesystem. Canonicalizing a symlink would change families.
fn actual_segment_name(path: &Path) -> io::Result<PathBuf> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| io::Error::other("Image filename is not valid UTF-8"))?;
    let entries = std::fs::read_dir(parent)?.collect::<io::Result<Vec<_>>>()?;
    if let Some(entry) = entries.iter().find(|entry| entry.file_name() == name) {
        return Ok(entry.path());
    }
    for entry in &entries {
        if entry
            .file_name()
            .to_str()
            .is_some_and(|s| s.eq_ignore_ascii_case(name))
            && same_file::is_same_file(path, entry.path())?
        {
            return Ok(entry.path());
        }
    }
    // Filesystem identity also resolves Unicode case aliases. If several
    // differently named hard links match, require an exact spelling rather
    // than silently switch to another segment family.
    let mut matched = None;
    for entry in entries {
        if entry
            .path()
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case(path.extension().unwrap_or_default()))
            && entry.path().is_file()
            && same_file::is_same_file(path, entry.path())?
        {
            if matched.is_some() {
                return Err(io::Error::other(
                    "Ambiguous first-segment filename; use its exact name from the source directory",
                ));
            }
            matched = Some(entry.path());
        }
    }
    Ok(matched.unwrap_or_else(|| path.to_path_buf()))
}

fn segment_extension(extension: &str, v2: bool) -> bool {
    let bytes = extension.as_bytes();
    let suffix =
        |s: &[u8]| s.iter().all(u8::is_ascii_digit) || s.iter().all(u8::is_ascii_alphabetic);
    if v2 {
        bytes.len() == 4
            && bytes[0].eq_ignore_ascii_case(&b'E')
            && match bytes[1].to_ascii_lowercase() {
                b'x' => suffix(&bytes[2..]),
                b'y' | b'z' => bytes[2..].iter().all(u8::is_ascii_alphabetic),
                _ => false,
            }
    } else {
        bytes.len() == 3
            && match bytes[0].to_ascii_uppercase() {
                b'E' => suffix(&bytes[1..]),
                b'F'..=b'Z' => bytes[1..].iter().all(u8::is_ascii_alphabetic),
                _ => false,
            }
    }
}

/// Open a disk image and every NTFS volume in it.
pub fn open(path: &Path) -> io::Result<Opened> {
    match kind(path)? {
        Kind::Vhdx => Ok(Opened {
            image: Image::open(path)?,
            kind: Kind::Vhdx,
            segments: 1,
            status: ReadStatus(None),
        }),
        Kind::E01 => open_e01(path),
    }
}

fn open_e01(path: &Path) -> io::Result<Opened> {
    let invalid =
        |e: ewf_image::EwfError| io::Error::new(io::ErrorKind::InvalidData, e.to_string());
    let actual = actual_segment_name(path)?;
    let ewf = ewf_image::Image::open(&actual).map_err(invalid)?;
    let info = ewf.info();
    if info.single_files.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "it is a logical evidence file, which holds files, not a disk",
        ));
    }
    let segments = info.segment_count;
    let size = ewf.media_size();
    let sector = ewf.bytes_per_sector().unwrap_or(512);
    let image = Image::from_disk(Box::new(ewf.cursor()), size, sector)?;
    Ok(Opened {
        image,
        kind: Kind::E01,
        segments,
        status: ReadStatus(Some(ewf)),
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
    #[test]
    fn acquisition_error_counts_saturate_instead_of_overflowing() {
        let ranges = [
            ewf_image::AcquisitionError {
                first_sector: 0,
                sector_count: u64::MAX,
            },
            ewf_image::AcquisitionError {
                first_sector: 1,
                sector_count: 1,
            },
        ];
        assert_eq!(unread_sector_count(&ranges), u64::MAX);
    }

    #[test]
    fn source_files_include_damaged_segments_without_treating_unrelated_files_as_sources() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            "case.E01",
            "case.E02",
            "case.EAA",
            "case.FAA",
            "case.csv",
            "other.E02",
        ] {
            std::fs::write(dir.path().join(name), b"damaged").unwrap();
        }
        let sources = source_files(&dir.path().join("case.E01")).unwrap();
        assert_eq!(sources.len(), 4);
        assert!(sources.contains(&dir.path().join("case.E02")));
        assert!(!sources.contains(&dir.path().join("case.csv")));
        for ext in ["Ex01", "Ex02", "EyAA", "EzAA"] {
            std::fs::write(dir.path().join(format!("v2.{ext}")), b"damaged").unwrap();
        }
        assert_eq!(source_files(&dir.path().join("v2.Ex01")).unwrap().len(), 4);
    }

    #[test]
    fn source_status_keeps_findings_discovered_after_opening() {
        let dir = tempfile::tempdir().unwrap();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/e01");
        for ext in ["E01", "E02"] {
            std::fs::copy(
                source.join(format!("sample.{ext}")),
                dir.path().join(format!("late.{ext}")),
            )
            .unwrap();
        }
        let path = dir.path().join("late.E01");
        let ewf = ewf_image::Image::open(&path).unwrap();
        let sections = ewf.sections().to_vec();
        drop(ewf);
        let mut patched = 0;
        for section in sections {
            if !matches!(&section.kind, ewf_image::SectionKind::Ewf1(name) if name=="volume" || name=="data")
            {
                continue;
            }
            assert_eq!(section.data_size, 1052);
            let segment = dir.path().join(if section.segment_index == 0 {
                "late.E01"
            } else {
                "late.E02"
            });
            let mut bytes = std::fs::read(&segment).unwrap();
            let start = section.data_offset as usize;
            let data = &mut bytes[start..start + 1052];
            let sectors = u64::from_le_bytes(data[16..24].try_into().unwrap());
            data[16..24].copy_from_slice(&(sectors - 1).to_le_bytes());
            let (mut a, mut b) = (1u32, 0u32);
            for value in &data[..1048] {
                a = (a + u32::from(*value)) % 65521;
                b = (b + a) % 65521;
            }
            data[1048..1052].copy_from_slice(&((b << 16) | a).to_le_bytes());
            std::fs::write(segment, bytes).unwrap();
            patched += 1;
        }
        assert!(patched >= 2);
        let opened = open(&path).unwrap();
        let before = opened.status.warnings();
        assert!(before.is_empty(), "{before:?}");
        let reader = opened.status.0.as_ref().unwrap();
        let mut byte = [0u8];
        reader.read_at(&mut byte, reader.media_size() - 1).unwrap();
        let after = opened.status.warnings();
        assert!(
            after
                .iter()
                .any(|s| s.contains("extra") && s.contains("dropped")),
            "{after:?}"
        );
    }
}
