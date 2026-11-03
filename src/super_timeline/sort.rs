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

use super::model::SortRow;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

pub struct Buffer {
    rows: Vec<SortRow>,
    bytes: usize,
    budget: usize,
    folder: PathBuf,
    input: usize,
    pub chunks: Vec<PathBuf>,
    pub largest_row: usize,
}

impl Buffer {
    pub fn new(folder: &Path, input: usize, budget: usize) -> Self {
        Self {
            rows: Vec::new(),
            bytes: 0,
            budget,
            folder: folder.to_path_buf(),
            input,
            chunks: Vec::new(),
            largest_row: 0,
        }
    }

    pub fn push(&mut self, row: SortRow) -> io::Result<()> {
        let size = serde_json::to_vec(&row)?.len();
        self.largest_row = self.largest_row.max(size);
        // Account for tree/string/vector overhead as well as serialized bytes. One record and
        // parser state remain additional to the configured aggregate sort-buffer budget.
        let estimate = size
            .saturating_mul(3)
            .saturating_add(std::mem::size_of::<SortRow>());
        if !self.rows.is_empty() && self.bytes.saturating_add(estimate) > self.budget {
            self.flush()?;
        }
        self.rows.push(row);
        self.bytes = self.bytes.saturating_add(estimate);
        if self.bytes >= self.budget {
            self.flush()?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if self.rows.is_empty() {
            return Ok(());
        }
        self.rows.sort_unstable_by(|a, b| a.key().cmp(&b.key()));
        let path = self.folder.join(format!(
            "input-{}-chunk-{}.jsonl",
            self.input,
            self.chunks.len()
        ));
        let mut out = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?,
        );
        for row in self.rows.drain(..) {
            serde_json::to_writer(&mut out, &row)?;
            out.write_all(b"\n")?;
        }
        out.flush()?;
        self.chunks.push(path);
        self.bytes = 0;
        Ok(())
    }
}

struct Head {
    row: SortRow,
    reader: usize,
}
impl PartialEq for Head {
    fn eq(&self, other: &Self) -> bool {
        self.row.key() == other.row.key() && self.reader == other.reader
    }
}
impl Eq for Head {}
impl PartialOrd for Head {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Head {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .row
            .key()
            .cmp(&self.row.key())
            .then_with(|| other.reader.cmp(&self.reader))
    }
}

fn next(reader: &mut BufReader<File>) -> io::Result<Option<SortRow>> {
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    serde_json::from_str(&line)
        .map(Some)
        .map_err(io::Error::other)
}

pub fn merge(paths: &[PathBuf], mut sink: impl FnMut(SortRow) -> io::Result<()>) -> io::Result<()> {
    let mut readers: Vec<_> = paths
        .iter()
        .map(|p| File::open(p).map(BufReader::new))
        .collect::<Result<_, _>>()?;
    let mut queue = BinaryHeap::new();
    for (index, reader) in readers.iter_mut().enumerate() {
        if let Some(row) = next(reader)? {
            queue.push(Head { row, reader: index });
        }
    }
    while let Some(head) = queue.pop() {
        let index = head.reader;
        sink(head.row)?;
        if let Some(row) = next(&mut readers[index])? {
            queue.push(Head { row, reader: index });
        }
    }
    Ok(())
}

/// Bound both open descriptors and queued event records during every merge pass.
pub fn reduce(
    mut paths: Vec<PathBuf>,
    folder: &Path,
    budget: usize,
    largest: usize,
) -> io::Result<(Vec<PathBuf>, usize)> {
    let fan_in = (budget / largest.saturating_mul(3).saturating_add(8192).max(1)).clamp(2, 32);
    let mut pass = 0;
    while paths.len() > fan_in {
        let mut reduced = Vec::new();
        for (index, group) in paths.chunks(fan_in).enumerate() {
            let target = folder.join(format!("merge-{pass}-{index}.jsonl"));
            let mut out = BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&target)?,
            );
            merge(group, |row| {
                serde_json::to_writer(&mut out, &row)?;
                out.write_all(b"\n")
            })?;
            out.flush()?;
            reduced.push(target);
        }
        // These are exclusively the chunks created in our private scratch directory.
        for path in paths {
            std::fs::remove_file(path)?;
        }
        paths = reduced;
        pass += 1;
    }
    Ok((paths, pass))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::super_timeline::model::{Event, Point};

    fn row(second: i64, identity: &str, origin: &str) -> SortRow {
        SortRow {
            time: Point {
                seconds: second,
                nanos: 0,
            },
            identity: identity.into(),
            origin: origin.into(),
            event: Event {
                timestamp: second.to_string(),
                computer: "host".into(),
                artifact: "test".into(),
                timestamp_type: "test".into(),
                user: String::new(),
                path: String::new(),
                message: String::new(),
                source_file: String::new(),
                source_field: String::new(),
                record_id: identity.into(),
                details: Default::default(),
            },
        }
    }

    #[test]
    fn multiple_merge_passes_preserve_precise_stable_order_and_duplicates() {
        let tmp = tempfile::tempdir().unwrap();
        let mut buffer = Buffer::new(tmp.path(), 0, 1);
        for i in (0..100).rev() {
            buffer
                .push(row(
                    i % 7 - 3,
                    &format!("id-{:03}", i % 13),
                    &format!("{i:03}"),
                ))
                .unwrap();
        }
        buffer.flush().unwrap();
        let (files, passes) = reduce(buffer.chunks, tmp.path(), 1, buffer.largest_row).unwrap();
        assert!(passes > 1);
        let mut rows = Vec::new();
        merge(&files, |row| {
            rows.push(row);
            Ok(())
        })
        .unwrap();
        assert_eq!(rows.len(), 100);
        assert!(rows.windows(2).all(|pair| pair[0].key() <= pair[1].key()));
        assert!(rows.first().unwrap().time.seconds < 0);
    }
}
