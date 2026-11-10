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

//! Files of a staged disk image that were left as empty placeholders. Image staging copies only
//! the files the parsers select. persistence-hunter can refer to any other file (a payload, a
//! signature catalog, an event log), so it reads a placeholder's content from the image just
//! before using it.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, FileTimes};
use std::io::{self, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use vhdx_mount::image::{Image, ROOT_RECORD, Stat};

/// Where each staged volume came from. Written by image staging; read by persistence-hunter.
#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Sources {
    /// The private workspace: temporary copies are written here, outside the parsed trees,
    /// then renamed over their placeholders.
    #[serde(rename = "Workspace")]
    pub(crate) workspace: PathBuf,
    #[serde(rename = "Volumes")]
    pub(crate) volumes: Vec<VolumeSource>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct VolumeSource {
    /// The staged volume's directory (canonical).
    #[serde(rename = "Root")]
    pub(crate) root: PathBuf,
    /// The image (canonical; an E01 image's first segment).
    #[serde(rename = "Image")]
    pub(crate) image: PathBuf,
    /// The index of the NTFS volume in the image.
    #[serde(rename = "Volume")]
    pub(crate) volume: usize,
}

/// Copy a file's unnamed data stream from an image into a new temporary file in `scratch`.
/// Returns the file and, if its timestamps could not be preserved, why.
pub(crate) fn copy_record(
    image: &Arc<Image>,
    volume: usize,
    record: u64,
    stat: &Stat,
    scratch: &Path,
) -> io::Result<(tempfile::NamedTempFile, Option<String>)> {
    let mut reader = image.file_reader(volume, record)?;
    let mut temporary = tempfile::NamedTempFile::new_in(scratch)?;
    let file = temporary.as_file_mut();
    copy_sparse(&mut reader, file, stat.size)?;
    let time_error = set_times(file, stat).err().map(|error| {
        format!("File content was retained, but its filesystem timestamps could not be preserved: {error}")
    });
    Ok((temporary, time_error))
}

/// Preserve the filesystem mtime used by persistence-hunter; parsed artifact timestamps still
/// come from the evidence bytes, never these staging files.
pub(crate) fn set_times(file: &File, stat: &Stat) -> io::Result<()> {
    let mut times = FileTimes::new();
    if let Some(t) = crate::image_metadata::system_time(stat.modified.0) {
        times = times.set_modified(t);
    }
    if let Some(t) = crate::image_metadata::system_time(stat.accessed.0) {
        times = times.set_accessed(t);
    }
    file.set_times(times)
}

pub(crate) fn copy_sparse(
    reader: &mut impl Read,
    output: &mut File,
    expected: u64,
) -> io::Result<()> {
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

struct OpenedImage {
    image: Arc<Image>,
    status: crate::disk_image::ReadStatus,
    /// Reader findings already reported (staging reported those found before).
    reported: Mutex<usize>,
}

struct State {
    sources: Sources,
    images: Mutex<HashMap<PathBuf, Arc<OpenedImage>>>,
    /// One copy at a time: concurrent readers of the same placeholder wait for it.
    copying: Mutex<()>,
}

static STATE: OnceLock<State> = OnceLock::new();

/// Read placeholders from the images described in `path` (written by image staging).
pub(crate) fn install(path: &Path) -> Result<(), String> {
    let sources: Sources = File::open(path)
        .map_err(|e| e.to_string())
        .and_then(|file| serde_json::from_reader(BufReader::new(file)).map_err(|e| e.to_string()))
        .map_err(|e| format!("Could not read the image file sources: {e}"))?;
    // A second installation in the same process (tests) keeps the first.
    let _ = STATE.set(State {
        sources,
        images: Mutex::new(HashMap::new()),
        copying: Mutex::new(()),
    });
    Ok(())
}

impl State {
    fn image(&self, path: &Path) -> io::Result<Arc<OpenedImage>> {
        let mut images = self.images.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(opened) = images.get(path) {
            return Ok(opened.clone());
        }
        let opened = crate::disk_image::open(path)?;
        let reported = Mutex::new(opened.status.warnings().len());
        let opened = Arc::new(OpenedImage {
            image: Arc::new(opened.image),
            status: opened.status,
            reported,
        });
        images.insert(path.to_path_buf(), opened.clone());
        Ok(opened)
    }

    /// The image, volume and record of a staged file; `None` if it is not in a staged volume.
    fn locate(&self, path: &Path) -> io::Result<Option<(Arc<OpenedImage>, usize, u64)>> {
        let canonical = fs::canonicalize(path)?;
        let Some(source) = self
            .sources
            .volumes
            .iter()
            .find(|source| canonical.starts_with(&source.root))
        else {
            return Ok(None);
        };
        let Ok(relative) = canonical.strip_prefix(&source.root) else {
            return Ok(None);
        };
        let opened = self.image(&source.image)?;
        let Some(volume) = opened.image.volumes.get(source.volume) else {
            return Ok(None);
        };
        let mut record = ROOT_RECORD;
        for component in relative.components() {
            let Component::Normal(name) = component else {
                return Ok(None);
            };
            let Some(name) = name.to_str() else {
                return Ok(None);
            };
            match volume.lookup(record, name)? {
                Some(entry) => record = entry.record,
                None => return Ok(None),
            }
        }
        Ok(Some((opened, source.volume, record)))
    }
}

/// The size of a file of the evidence: for a placeholder, its file's size in the image.
pub(crate) fn len(path: &Path, on_disk: u64) -> u64 {
    if on_disk != 0 {
        return on_disk;
    }
    let Some(state) = STATE.get() else {
        return on_disk;
    };
    // The size recorded at discovery may predate reading the placeholder from the image.
    if let Ok(metadata) = fs::metadata(path)
        && metadata.len() != 0
    {
        return metadata.len();
    }
    match state.locate(path) {
        Ok(Some((opened, volume, record))) => {
            opened.image.volumes[volume]
                .stat(record)
                .map_or(
                    on_disk,
                    |stat| if stat.directory { on_disk } else { stat.size },
                )
        }
        _ => on_disk,
    }
}

/// Replace a placeholder with its content from the image. Files that already have content,
/// and paths outside the staged volumes, are left as they are.
pub(crate) fn materialize(path: &Path) -> io::Result<()> {
    let Some(state) = STATE.get() else {
        return Ok(());
    };
    let empty = |path: &Path| fs::metadata(path).is_ok_and(|m| m.is_file() && m.len() == 0);
    if !empty(path) {
        return Ok(());
    }
    let _copying = state.copying.lock().unwrap_or_else(PoisonError::into_inner);
    if !empty(path) {
        return Ok(());
    }
    let Some((opened, volume, record)) = state.locate(path)? else {
        return Ok(());
    };
    let stat = opened.image.volumes[volume].stat(record)?;
    if stat.directory || stat.size == 0 {
        return Ok(());
    }
    let (temporary, _) = copy_record(
        &opened.image,
        volume,
        record,
        &stat,
        &state.sources.workspace,
    )?;
    temporary.persist(path).map_err(|e| e.error)?;
    // As when staging copies a file: keep what was read and report acquisition damage.
    let warnings = opened.status.warnings();
    let mut reported = opened
        .reported
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    for warning in warnings.iter().skip(*reported) {
        eprintln!(
            "Image read problem while reading {}: {warning}",
            path.display()
        );
    }
    *reported = (*reported).max(warnings.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
