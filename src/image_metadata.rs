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

//! Original filesystem times for staged image inputs. A missing entry is an
//! error: never fall back to a temporary file's creation/access timestamps.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, Metadata};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Snapshot {
    #[serde(rename = "Files")]
    files: HashMap<PathBuf, [u64; 3]>,
}

impl Snapshot {
    pub(crate) fn insert(&mut self, path: &Path, times: [u64; 3]) -> io::Result<()> {
        self.files.insert(path.canonicalize()?, times);
        Ok(())
    }
}

pub(crate) struct SourceTimes(Option<Snapshot>);

impl SourceTimes {
    pub(crate) fn load(path: Option<&Path>) -> Result<Self, String> {
        let snapshot = path
            .map(|path| -> io::Result<Snapshot> {
                serde_json::from_reader(BufReader::new(File::open(path)?)).map_err(io::Error::other)
            })
            .transpose()
            .map_err(|e| format!("Could not read original image metadata: {e}"))?;
        Ok(Self(snapshot))
    }

    pub(crate) fn for_file(
        &self,
        path: &Path,
        metadata: &Metadata,
    ) -> Result<[Option<SystemTime>; 3], String> {
        match &self.0 {
            None => Ok([
                metadata.created().ok(),
                metadata.modified().ok(),
                metadata.accessed().ok(),
            ]),
            Some(snapshot) => {
                let path = path.canonicalize().map_err(|e| e.to_string())?;
                let times = snapshot
                    .files
                    .get(&path)
                    .ok_or("Original image metadata is missing for this file")?;
                Ok(times.map(system_time))
            }
        }
    }
}

pub(crate) fn system_time(value: u64) -> Option<SystemTime> {
    if value == 0 || value > 2_650_467_743_999_999_999 {
        return None;
    }
    let ticks = i128::from(value) - 116_444_736_000_000_000i128;
    let magnitude = ticks.unsigned_abs();
    let duration = Duration::new(
        (magnitude / 10_000_000) as u64,
        ((magnitude % 10_000_000) * 100) as u32,
    );
    if ticks >= 0 {
        UNIX_EPOCH.checked_add(duration)
    } else {
        UNIX_EPOCH.checked_sub(duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_or_invalid_native_times_never_become_staging_times() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("input");
        std::fs::write(&path, b"data").unwrap();
        let metadata = path.metadata().unwrap();
        let mut snapshot = Snapshot::default();
        snapshot
            .insert(&path, [116_444_736_000_000_000, 0, u64::MAX])
            .unwrap();
        let source = SourceTimes(Some(snapshot));
        assert_eq!(
            source.for_file(&path, &metadata).unwrap(),
            [Some(UNIX_EPOCH), None, None]
        );
        assert!(
            SourceTimes(Some(Snapshot::default()))
                .for_file(&path, &metadata)
                .is_err()
        );
    }
}
