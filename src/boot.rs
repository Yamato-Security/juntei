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

//! `parse-ntfs boot`: the NTFS boot sector (`$Boot`, or the first sector of a volume image), read
//! by the boot crate of ys-ntfs-parser. One record per boot sector: the volume serial number (its
//! low 32 bits are what LNK files, Jump Lists and Prefetch files record of a file's volume), the
//! volume's geometry, where its `$MFT` is, a hash of its boot code to compare across hosts, and
//! what about the sector Windows would not mount.

use crate::cli::BootOptions;
use crate::color::JunteiColor;
use crate::registry::{RecordWriter, sink};
use crate::util::{self, error_msg, fmt_num, p, p_label};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use ys_ntfs_parser::boot::{self as ntfs_boot, BOOT_FILE_SIZE, BootSector};

/// Errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Boot sectors listed in the summary.
const MAX_PRINTED: usize = 20;

/// Run the command; returns the exit status.
pub fn run(opts: &BootOptions) -> i32 {
    let no_color = opts.common.no_color;
    let data_on_stdout = opts.data_on_stdout();
    let report_error = |msg: &str| {
        if data_on_stdout {
            eprintln!("{msg}");
        } else {
            error_msg(no_color, msg);
        }
    };

    let files = match (&opts.filepath, &opts.directory) {
        (Some(f), _) if f.is_file() => vec![f.clone()],
        (Some(f), _) => {
            report_error(&format!("The file does not exist: {}", f.display()));
            return 1;
        }
        (None, Some(d)) if !d.is_dir() => {
            report_error(&format!("The directory does not exist: {}", d.display()));
            return 1;
        }
        (None, Some(d)) => {
            let found = find_boot_files(d);
            if found.is_empty() {
                report_error(&format!("No $Boot files were found in {}", d.display()));
                return 1;
            }
            found
        }
        (None, None) => unreachable!("clap requires -f or -d"),
    };
    let sink = match sink(opts.output.as_ref(), opts.clobber, data_on_stdout, || {
        files.clone()
    }) {
        Ok(sink) => sink,
        Err(msg) => {
            report_error(&msg);
            return 1;
        }
    };
    let mut writer = match sink.map(|w| RecordWriter::new(w, opts.output_type(), &CSV_HEADER)) {
        Some(Ok(w)) => Some(w),
        Some(Err(e)) => {
            report_error(&format!("Could not write the boot sectors: {e}"));
            return 1;
        }
        None => None,
    };
    if !data_on_stdout {
        p_label(no_color, "$Boot files: ", &fmt_num(files.len()));
        p(None, "", true);
    }

    let mut stats = Stats::default();
    for path in &files {
        let source = path.to_string_lossy().to_string();
        let record = match read_boot(path) {
            Ok(data) => match BootSector::parse(&data) {
                Ok(boot) => Record::new(&source, &boot, &data),
                Err(e) => {
                    stats.fail(&report_error, &format!("Could not parse {source}: {e}"));
                    continue;
                }
            },
            Err(e) => {
                stats.fail(&report_error, &format!("Could not read {source}: {e}"));
                continue;
            }
        };
        let written = match writer.as_mut() {
            Some(w) if w.is_csv() => w.csv(&record.csv()),
            Some(w) => w.json(&record),
            None => Ok(()),
        };
        if let Err(e) = written {
            report_error(&format!("Could not write the boot sectors: {e}"));
            return 1;
        }
        stats.add(record);
    }
    if stats.failed > MAX_PRINTED_ERRORS {
        report_error(&format!(
            "{} more files could not be parsed",
            fmt_num(stats.failed - MAX_PRINTED_ERRORS)
        ));
    }
    if let Some(w) = writer
        && let Err(e) = w.finish()
    {
        report_error(&format!("Could not write the boot sectors: {e}"));
        return 1;
    }

    if !data_on_stdout {
        stats.print(no_color);
        if let Some(out) = &opts.output {
            p_label(
                no_color,
                "Results saved: ",
                &format!(
                    "{} ({} boot sectors, {})",
                    out.display(),
                    fmt_num(stats.records.len()),
                    util::file_size_display(out)
                ),
            );
        } else {
            p(
                None,
                "To save every field, add -o <FILE> (-t csv, json or jsonl).",
                true,
            );
        }
    }
    i32::from(stats.records.is_empty())
}

/// Whether a file name is `$Boot`'s (`$Boot`, `C_$Boot`, …).
pub(crate) fn is_boot_name(name: &str) -> bool {
    name.to_uppercase().ends_with("$BOOT")
}

/// Whether the file starts with an NTFS boot sector: the OEM ID and the `55 AA` signature.
fn starts_with_boot_sector(path: &Path) -> bool {
    let mut head = [0u8; 512];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut head))
        .is_ok()
        && &head[3..11] == ntfs_boot::NTFS_OEM_ID
        && head[510..] == [0x55, 0xaa]
}

/// The `$Boot` files under `dir`, recursively, in path order: named so, or starting with an NTFS
/// boot sector (a volume image).
fn find_boot_files(dir: &Path) -> Vec<PathBuf> {
    util::find_files(dir, |_| true)
        .into_iter()
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| is_boot_name(&n.to_string_lossy()))
                || starts_with_boot_sector(p)
        })
        .collect()
}

/// The first 8 KiB of the file: `$Boot` whole, or the start of a volume.
fn read_boot(path: &Path) -> io::Result<Vec<u8>> {
    let mut data = Vec::with_capacity(BOOT_FILE_SIZE);
    File::open(path)?
        .take(BOOT_FILE_SIZE as u64)
        .read_to_end(&mut data)?;
    Ok(data)
}

/// A boot sector's record, its fields in this order.
#[derive(Serialize)]
struct Record {
    #[serde(rename = "SourcePath")]
    source_path: String,
    #[serde(rename = "VolumeSerialNumber")]
    volume_serial_number: String,
    #[serde(rename = "VolumeSerialNumber64")]
    volume_serial_number_64: String,
    #[serde(rename = "OemId")]
    oem_id: String,
    #[serde(rename = "BytesPerSector")]
    bytes_per_sector: u16,
    #[serde(rename = "SectorsPerCluster")]
    sectors_per_cluster: Option<u64>,
    #[serde(rename = "ClusterSize")]
    cluster_size: Option<u64>,
    #[serde(rename = "TotalSectors")]
    total_sectors: u64,
    #[serde(rename = "VolumeSize")]
    volume_size: Option<u64>,
    #[serde(rename = "HiddenSectors")]
    hidden_sectors: u32,
    #[serde(rename = "MftCluster")]
    mft_cluster: u64,
    #[serde(rename = "MftOffset")]
    mft_offset: Option<u64>,
    #[serde(rename = "MftMirrorCluster")]
    mft_mirror_cluster: u64,
    #[serde(rename = "MftMirrorOffset")]
    mft_mirror_offset: Option<u64>,
    #[serde(rename = "FileRecordSize")]
    file_record_size: Option<u64>,
    #[serde(rename = "IndexRecordSize")]
    index_record_size: Option<u64>,
    #[serde(rename = "MediaDescriptor")]
    media_descriptor: String,
    #[serde(rename = "JumpInstruction")]
    jump_instruction: String,
    #[serde(rename = "EndSignature")]
    end_signature: String,
    #[serde(rename = "BootCodeSHA256")]
    boot_code_sha256: String,
    #[serde(rename = "Problems")]
    problems: Vec<String>,
}

const CSV_HEADER: [&str; 21] = [
    "SourcePath",
    "VolumeSerialNumber",
    "VolumeSerialNumber64",
    "OemId",
    "BytesPerSector",
    "SectorsPerCluster",
    "ClusterSize",
    "TotalSectors",
    "VolumeSize",
    "HiddenSectors",
    "MftCluster",
    "MftOffset",
    "MftMirrorCluster",
    "MftMirrorOffset",
    "FileRecordSize",
    "IndexRecordSize",
    "MediaDescriptor",
    "JumpInstruction",
    "EndSignature",
    "BootCodeSHA256",
    "Problems",
];

/// Bytes as hexadecimal pairs, separated by spaces (`EB 52 90`).
fn spaced_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

impl Record {
    fn new(source: &str, boot: &BootSector, data: &[u8]) -> Self {
        let mut hash = Sha256::new();
        for part in ntfs_boot::boot_code(data) {
            hash.update(part);
        }
        let mut problems: Vec<String> = boot.problems().iter().map(ToString::to_string).collect();
        if data.len() < BOOT_FILE_SIZE {
            problems.push(format!(
                "only {} of $Boot's {} bytes: BootCodeSHA256 covers those",
                fmt_num(data.len()),
                fmt_num(BOOT_FILE_SIZE)
            ));
        }
        Record {
            source_path: source.to_string(),
            volume_serial_number: format!("{:08X}", boot.volume_serial_number()),
            volume_serial_number_64: format!("{:016X}", boot.serial_number),
            oem_id: boot.oem_id_text().trim_end().to_string(),
            bytes_per_sector: boot.bytes_per_sector,
            sectors_per_cluster: boot.sectors_per_cluster(),
            cluster_size: boot.cluster_size(),
            total_sectors: boot.total_sectors,
            volume_size: boot.volume_size(),
            hidden_sectors: boot.hidden_sectors,
            mft_cluster: boot.mft_cluster,
            mft_offset: boot.mft_offset(),
            mft_mirror_cluster: boot.mft_mirror_cluster,
            mft_mirror_offset: boot.mft_mirror_offset(),
            file_record_size: boot.file_record_size(),
            index_record_size: boot.index_record_size(),
            media_descriptor: format!("{:02X}", boot.media_descriptor),
            jump_instruction: spaced_hex(&boot.jump),
            end_signature: spaced_hex(&boot.end_signature.to_le_bytes()),
            boot_code_sha256: hash.finalize().iter().map(|b| format!("{b:02x}")).collect(),
            problems,
        }
    }

    fn csv(&self) -> Vec<String> {
        let num = |v: Option<u64>| v.map(|n| n.to_string()).unwrap_or_default();
        vec![
            self.source_path.clone(),
            self.volume_serial_number.clone(),
            self.volume_serial_number_64.clone(),
            self.oem_id.clone(),
            self.bytes_per_sector.to_string(),
            num(self.sectors_per_cluster),
            num(self.cluster_size),
            self.total_sectors.to_string(),
            num(self.volume_size),
            self.hidden_sectors.to_string(),
            self.mft_cluster.to_string(),
            num(self.mft_offset),
            self.mft_mirror_cluster.to_string(),
            num(self.mft_mirror_offset),
            num(self.file_record_size),
            num(self.index_record_size),
            self.media_descriptor.clone(),
            self.jump_instruction.clone(),
            self.end_signature.clone(),
            self.boot_code_sha256.clone(),
            self.problems.join("; "),
        ]
    }

    /// The serial number as Windows shows it: `180B-1E67`.
    fn serial_display(&self) -> String {
        let s = &self.volume_serial_number;
        format!("{}-{}", &s[..4], &s[4..])
    }
}

/// What the summary shows.
#[derive(Default)]
struct Stats {
    records: Vec<Record>,
    failed: usize,
}

impl Stats {
    fn fail(&mut self, report: &impl Fn(&str), msg: &str) {
        self.failed += 1;
        if self.failed <= MAX_PRINTED_ERRORS {
            report(msg);
        }
    }

    fn add(&mut self, record: Record) {
        self.records.push(record);
    }

    fn print(&self, no_color: bool) {
        p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
        p(None, "", true);
        p_label(
            no_color,
            "Boot sectors parsed: ",
            &fmt_num(self.records.len()),
        );
        if self.failed > 0 {
            p_label(no_color, "Could not parse: ", &fmt_num(self.failed));
        }
        for r in self.records.iter().take(MAX_PRINTED) {
            let size = r
                .volume_size
                .map_or_else(|| "unknown size".to_string(), util::size_display);
            let cluster = r
                .cluster_size
                .map_or_else(|| "unknown".to_string(), |c| fmt_num(c as usize));
            p(
                None,
                &format!(
                    "  {}: volume serial number {} ({}), {size}, clusters of {cluster} bytes, $MFT at cluster {}",
                    r.source_path,
                    r.serial_display(),
                    r.volume_serial_number,
                    fmt_num(r.mft_cluster as usize),
                ),
                true,
            );
        }
        if self.records.len() > MAX_PRINTED {
            p(
                None,
                &format!("  … {} more", fmt_num(self.records.len() - MAX_PRINTED)),
                true,
            );
        }
        // the boot code is the same on volumes formatted by the same Windows: one that differs
        // calls for a look
        let mut by_code: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for r in &self.records {
            by_code
                .entry(&r.boot_code_sha256)
                .or_default()
                .push(&r.source_path);
        }
        if by_code.len() > 1 {
            p_label(
                no_color,
                "Boot code (SHA-256), by number of boot sectors: ",
                &fmt_num(by_code.len()),
            );
            let mut codes: Vec<_> = by_code.into_iter().collect();
            codes.sort_by_key(|(code, sources)| (std::cmp::Reverse(sources.len()), *code));
            for (code, sources) in codes {
                let list = if sources.len() <= 3 {
                    sources.join(", ")
                } else {
                    format!("{} boot sectors", fmt_num(sources.len()))
                };
                p(None, &format!("  {code}: {list}"), true);
            }
        } else if let Some(code) = by_code.keys().next() {
            p_label(no_color, "Boot code (SHA-256): ", code);
        }
        let problems: Vec<_> = self
            .records
            .iter()
            .filter(|r| !r.problems.is_empty())
            .collect();
        if !problems.is_empty() {
            p_label(
                no_color,
                "Boot sectors with problems: ",
                &fmt_num(problems.len()),
            );
            for r in problems.iter().take(MAX_PRINTED) {
                p(
                    None,
                    &format!("  {}: {}", r.source_path, r.problems.join("; ")),
                    true,
                );
            }
        }
        p(None, "", true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_names() {
        assert!(is_boot_name("$Boot"));
        assert!(is_boot_name("C_$BOOT"));
        assert!(!is_boot_name("$Boot.txt"));
        assert!(!is_boot_name("boot.ini"));
    }

    #[test]
    fn boot_files_by_name_and_content() {
        let dir = tempfile::tempdir().unwrap();
        let mut sector = vec![0u8; 512];
        sector[3..11].copy_from_slice(b"NTFS    ");
        sector[510..].copy_from_slice(&[0x55, 0xaa]);
        std::fs::write(dir.path().join("$Boot"), b"short").unwrap();
        std::fs::write(dir.path().join("volume.dd"), &sector).unwrap();
        sector[510] = 0;
        std::fs::write(dir.path().join("not-a-volume.bin"), &sector).unwrap();
        let mut found: Vec<String> = find_boot_files(dir.path())
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        found.sort();
        assert_eq!(found, ["$Boot", "volume.dd"]);
    }
}
