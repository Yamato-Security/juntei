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

//! `parse-file rdp-cache`: the Remote Desktop client's persistent bitmap cache, the screen tiles
//! of earlier sessions that the client kept to avoid receiving them again. One row per tile, and
//! with `--extract` each tile as a BMP image and a PNG collage of each cache file.
//!
//! - `Cache????.bin` (Windows 7 and later): `RDP8bmp\0` and a 32-bit version, then one tile after
//!   another: a 64-bit key, a 16-bit width and height, and 32-bit BGRA pixels, top row first.
//! - `bcache2.bmc`, `bcache22.bmc`, `bcache23.bmc`, `bcache24.bmc` (Windows XP, Server 2003 and
//!   earlier clients; 8, 16, 24 and 32 bits per pixel): fixed slots, each a 20-byte header (key,
//!   width, height, data size, flags) and room for a 64 × 64 tile, bottom row first. Flag 0x08
//!   marks data compressed with RDP's interleaved RLE (MS-RDPBCGR 3.1.9). A tile smaller than its
//!   slot leaves the end of an earlier tile in the rest of it.
//!
//! Tiles have no times. The images are written as RdpCacheStitcher reads them: one folder per
//! cache file, with the tiles named `<cache file>_<index>.bmp`.

use crate::cli::{RdpCacheOptions, RdpCacheOutputType};
use crate::color::JunteiColor;
use crate::progress;
use crate::util::{self, check_output, error_msg, fmt_num, p, p_label};
use flate2::write::ZlibEncoder;
use flate2::{Compression, Crc};
use indicatif::ProgressBar;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};

/// Parse errors printed; the rest are only counted.
const MAX_PRINTED_ERRORS: usize = 20;
/// Larger files are not bitmap caches: Windows keeps each `Cache????.bin` within 100 MB and each
/// `.bmc` file within 10 MB.
const MAX_CACHE_SIZE: u64 = 1024 * 1024 * 1024;
/// `Cache????.bin`: the signature, then a 32-bit version.
const BIN_SIGNATURE: &[u8; 8] = b"RDP8bmp\0";
const BIN_HEADER: usize = 12;
/// `Cache????.bin` tile header: key, width, height.
const BIN_TILE_HEADER: usize = 12;
/// `.bmc` tile header: key, width, height, data size, flags.
const BMC_TILE_HEADER: usize = 20;
/// The `.bmc` flag of RLE-compressed tile data.
const BMC_COMPRESSED: u32 = 0x08;
/// Tiles are at most 64 × 64 pixels, and a `.bmc` slot holds one that size.
const TILE: usize = 64;
/// Collage pixels no tile fills: magenta, which marks them apart from any screen content.
const PAD: [u8; 3] = [255, 0, 255];

/// `Cache0000.bin`, … (`Cache`, digits, `.bin`) and `bcache2.bmc`, … (`bcache`, digits, `.bmc`):
/// the names `-d`, `auto-parse` and disk-image staging select.
pub fn is_cache_name(name: &str) -> bool {
    is_bin_name(name) || is_bmc_name(name)
}

fn is_bin_name(name: &str) -> bool {
    named(name, "cache", ".bin", false)
}

fn is_bmc_name(name: &str) -> bool {
    named(name, "bcache", ".bmc", true)
}

fn named(name: &str, prefix: &str, suffix: &str, digits_optional: bool) -> bool {
    let lower = name.to_ascii_lowercase();
    lower
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(suffix))
        .is_some_and(|digits| {
            (digits_optional || !digits.is_empty()) && digits.bytes().all(|b| b.is_ascii_digit())
        })
}

/// The bytes per pixel a `.bmc` file's name gives.
fn bmc_name_depth(name: &str) -> Option<usize> {
    match name.to_ascii_lowercase().as_str() {
        "bcache2.bmc" => Some(1),
        "bcache22.bmc" => Some(2),
        "bcache23.bmc" => Some(3),
        "bcache24.bmc" => Some(4),
        _ => None,
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Run the command; returns the exit status.
pub fn run(opts: &RdpCacheOptions) -> i32 {
    let no_color = opts.common.no_color;
    let data_on_stdout = opts.data_on_stdout();
    let report_error = |bar: Option<&ProgressBar>, msg: &str| {
        if data_on_stdout {
            eprintln!("{msg}");
        } else {
            progress::suspended(bar, || error_msg(no_color, msg));
        }
    };

    // --- inputs ---------------------------------------------------------------------------
    let given_file = opts.filepath.is_some();
    let files = match (&opts.filepath, &opts.directory) {
        (Some(f), _) => {
            if !f.is_file() {
                report_error(None, &format!("The file does not exist: {}", f.display()));
                return 1;
            }
            vec![f.clone()]
        }
        (None, Some(d)) => {
            if !d.is_dir() {
                report_error(
                    None,
                    &format!("The directory does not exist: {}", d.display()),
                );
                return 1;
            }
            let found = util::find_files(d, is_cache_name);
            if found.is_empty() {
                report_error(
                    None,
                    &format!(
                        "No RDP bitmap caches (Cache????.bin, bcache*.bmc) were found in {}",
                        d.display()
                    ),
                );
                return 1;
            }
            found
        }
        (None, None) => unreachable!("clap requires -f or -d"),
    };
    if let Some(out) = &opts.output {
        if let Err(msg) = check_output(out, false, opts.clobber) {
            report_error(None, &msg);
            return 1;
        }
        if let Err(msg) = crate::registry::check_not_read(out, || files.clone()) {
            report_error(None, &msg);
            return 1;
        }
    }
    let mut extractor = match &opts.extract {
        Some(dir) => {
            if dir.is_file() {
                report_error(
                    None,
                    &format!("{} is a file, not a directory.", dir.display()),
                );
                return 1;
            }
            if let Err(e) = fs::create_dir_all(dir) {
                report_error(None, &format!("Could not create {}: {e}", dir.display()));
                return 1;
            }
            Some(Extractor {
                dir: dir.clone(),
                shown: opts.extract_shown_as.clone().unwrap_or_else(|| dir.clone()),
                clobber: opts.clobber,
                tiles: !opts.collage_only,
                columns: usize::from(opts.collage_width),
                inputs: files.clone(),
                folders: HashSet::new(),
                written: Written::default(),
            })
        }
        None => None,
    };
    if !data_on_stdout {
        let total: u64 = files
            .iter()
            .map(|p| p.metadata().map_or(0, |m| m.len()))
            .sum();
        p_label(no_color, "RDP bitmap cache files: ", &fmt_num(files.len()));
        p_label(no_color, "Total file size: ", &util::size_display(total));
        p(None, "", true);
    }
    let bar = (!data_on_stdout).then(|| {
        p(
            JunteiColor::Orange.rdg(no_color),
            "Parsing in progress. Please wait.",
            true,
        );
        p(None, "", true);
        progress::progress_bar(files.len(), no_color)
    });

    // --- parse -----------------------------------------------------------------------------
    let mut parsed: Vec<Parsed> = Vec::new();
    let mut failed = 0usize;
    let mut other = 0usize;
    for path in &files {
        if let Some(b) = &bar {
            b.set_message(progress::message(path));
        }
        match parse_file(path, extractor.as_mut()) {
            Ok(file) => {
                for problem in &file.problems {
                    report_error(bar.as_ref(), &format!("{}: {problem}", path.display()));
                }
                parsed.push(file);
            }
            Err(Unparsed::NotACache(_)) if !given_file => {
                // Another program's file with a cache's name, found with -d.
                other += 1;
            }
            Err(Unparsed::NotACache(e) | Unparsed::Failed(e)) => {
                failed += 1;
                if failed <= MAX_PRINTED_ERRORS {
                    report_error(
                        bar.as_ref(),
                        &format!("Could not parse {}: {e}", path.display()),
                    );
                }
            }
        }
        if let Some(b) = &bar {
            b.inc(1);
        }
    }
    if failed > MAX_PRINTED_ERRORS {
        report_error(
            bar.as_ref(),
            &format!(
                "{} more files could not be parsed",
                fmt_num(failed - MAX_PRINTED_ERRORS)
            ),
        );
    }
    if let Some(b) = &bar {
        b.finish_with_message(progress::finished_message(no_color));
    }
    let summary = |parsed: &[Parsed], extractor: Option<&Extractor>| {
        if !data_on_stdout {
            print_summary(parsed, failed, other, extractor, no_color);
        }
    };
    if parsed.is_empty() {
        summary(&parsed, extractor.as_ref());
        if failed == 0 && !given_file {
            report_error(
                None,
                "None of the files with cache names are RDP bitmap caches.",
            );
        }
        return 1;
    }

    // --- write -----------------------------------------------------------------------------
    let sink =
        match crate::registry::sink(opts.output.as_ref(), opts.clobber, data_on_stdout, || {
            files.clone()
        }) {
            Ok(sink) => sink,
            Err(msg) => {
                report_error(None, &msg);
                return 1;
            }
        };
    if let Some(sink) = sink
        && let Err(e) = write_records(sink, opts.output_type(), &parsed)
    {
        report_error(None, &format!("Could not write the records: {e}"));
        return 1;
    }

    // --- summary ---------------------------------------------------------------------------
    summary(&parsed, extractor.as_ref());
    if !data_on_stdout {
        match &opts.output {
            Some(out) => p_label(
                no_color,
                "Results saved: ",
                &format!("{} ({})", out.display(), util::file_size_display(out)),
            ),
            None if opts.extract.is_none() => p(
                None,
                "To save every tile's record, add -o <FILE> (-t csv, json or jsonl); to save the images, add --extract <DIR>.",
                true,
            ),
            None => {}
        }
    }
    0
}

// --- cache files ----------------------------------------------------------------------------

/// The two cache file formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Format {
    /// `Cache????.bin`.
    Bin,
    /// `bcache*.bmc`.
    Bmc,
}

impl Format {
    fn name(self) -> &'static str {
        match self {
            Format::Bin => "bin",
            Format::Bmc => "bmc",
        }
    }
}

/// A cache file's tiles, as found by their headers.
#[derive(Debug)]
struct Cache {
    format: Format,
    /// `Cache????.bin`: the version after the signature.
    version: Option<u32>,
    entries: Vec<Entry>,
    /// Damage outside any one tile.
    problems: Vec<String>,
    /// `.bmc`: slots with an all-zero header, never used.
    empty_slots: usize,
}

/// One tile's header.
#[derive(Debug)]
struct Entry {
    index: usize,
    offset: usize,
    key: u64,
    width: usize,
    height: usize,
    /// Bytes per pixel.
    depth: usize,
    /// `.bmc`: the header's flags.
    flags: Option<u32>,
    compressed: bool,
    data: Range<usize>,
    /// `.bmc`: the rest of the slot after an uncompressed tile smaller than 64 × 64.
    rest: Option<Range<usize>>,
    problems: Vec<String>,
}

/// Why a file gave no tiles.
enum Unparsed {
    /// Not an RDP bitmap cache at all (for `-d`, another program's file with a cache's name).
    NotACache(String),
    /// A cache that could not be read.
    Failed(String),
}

/// A parsed cache file: its records, and what it held.
struct Parsed {
    format: Format,
    records: Vec<Record>,
    problems: Vec<String>,
    empty: bool,
}

/// One tile's row.
struct Record {
    source: String,
    user: Option<String>,
    format: Format,
    version: Option<u32>,
    index: usize,
    offset: usize,
    key: u64,
    width: usize,
    height: usize,
    bits: usize,
    compressed: Option<bool>,
    flags: Option<u32>,
    single_color: Option<bool>,
    tile_image: Option<String>,
    remnant_image: Option<String>,
    collage: Option<String>,
    problems: Vec<String>,
}

fn parse_file(path: &Path, extractor: Option<&mut Extractor>) -> Result<Parsed, Unparsed> {
    let name = file_name(path);
    let size = path
        .metadata()
        .map_err(|e| Unparsed::Failed(e.to_string()))?
        .len();
    if size > MAX_CACHE_SIZE {
        return Err(Unparsed::NotACache(format!(
            "{} is larger than any RDP bitmap cache",
            util::size_display(size)
        )));
    }
    let data = fs::read(path).map_err(|e| Unparsed::Failed(e.to_string()))?;
    let user = crate::browser::path_user(path);
    if data.is_empty() {
        // Windows 10 and later leave an empty bcache24.bmc beside their Cache????.bin files.
        return Ok(Parsed {
            format: if is_bmc_name(&name) {
                Format::Bmc
            } else {
                Format::Bin
            },
            records: Vec::new(),
            problems: Vec::new(),
            empty: true,
        });
    }
    let cache = index(&data, &name)?;
    let source = path.to_string_lossy().into_owned();
    let mut records: Vec<Record> = Vec::with_capacity(cache.entries.len());
    let mut problems = cache.problems.clone();
    let mut collage = None;
    let mut extractor = extractor;
    let folder = extractor.as_mut().map(|x| x.folder(path, user.as_deref()));
    if let (Some(x), Some(folder)) = (extractor.as_mut(), &folder) {
        match x.start_collage(folder, &name, cache.entries.len()) {
            Ok(c) => collage = c,
            Err(e) => problems.push(e),
        }
    }
    for entry in &cache.entries {
        let mut record = Record {
            source: source.clone(),
            user: user.clone(),
            format: cache.format,
            version: cache.version,
            index: entry.index,
            offset: entry.offset,
            key: entry.key,
            width: entry.width,
            height: entry.height,
            bits: entry.depth * 8,
            compressed: (cache.format == Format::Bmc).then_some(entry.compressed),
            flags: entry.flags,
            single_color: None,
            tile_image: None,
            remnant_image: None,
            collage: collage.as_ref().map(|c: &Collage| c.shown.clone()),
            problems: entry.problems.clone(),
        };
        let image = match decode(entry, &data, cache.format) {
            Ok(image) => {
                record.single_color = Some(image.single_color());
                Some(image)
            }
            Err(e) => {
                // A tile cut short already says so.
                if record.problems.is_empty() {
                    record.problems.push(e);
                }
                None
            }
        };
        let remnant = entry
            .rest
            .as_ref()
            .and_then(|rest| remnant(entry, &data[rest.clone()]));
        if let (Some(x), Some(folder)) = (extractor.as_mut(), &folder) {
            if let Some(image) = &image
                && x.tiles
            {
                match x.write(
                    folder,
                    &format!("{name}_{:04}.bmp", entry.index),
                    &bmp(image),
                ) {
                    Ok(shown) => {
                        x.written.tiles += 1;
                        record.tile_image = Some(shown);
                    }
                    Err(e) => record.problems.push(e),
                }
            }
            if let Some(remnant) = &remnant
                && x.tiles
            {
                match x.write(
                    folder,
                    &format!("{name}_old_{:04}.bmp", entry.index),
                    &bmp(remnant),
                ) {
                    Ok(shown) => {
                        x.written.remnants += 1;
                        record.remnant_image = Some(shown);
                    }
                    Err(e) => record.problems.push(e),
                }
            }
        }
        let mut collage_failed = false;
        if let Some(c) = &mut collage
            && let Err(e) = c.add(image)
        {
            problems.push(format!("Could not write the collage {}: {e}", c.shown));
            collage_failed = true;
        }
        if collage_failed {
            collage = None;
            for r in &mut records {
                r.collage = None;
            }
            record.collage = None;
        }
        records.push(record);
    }
    if let Some(c) = collage {
        let shown = c.shown.clone();
        match c.finish() {
            Ok(()) => {
                if let Some(x) = extractor.as_mut() {
                    x.written.collages += 1;
                }
            }
            Err(e) => {
                problems.push(format!("Could not write the collage {shown}: {e}"));
                for r in &mut records {
                    r.collage = None;
                }
            }
        }
    }
    if cache.empty_slots > 0 && cache.entries.is_empty() && problems.is_empty() {
        // A .bmc file whose slots were all allocated but never used.
        return Ok(Parsed {
            format: cache.format,
            records,
            problems,
            empty: true,
        });
    }
    Ok(Parsed {
        format: cache.format,
        records,
        problems,
        empty: false,
    })
}

/// The tiles of a cache file, by its format: `Cache????.bin` by its signature, a `.bmc` file by
/// its name or, under another name, by a plausible first tile header.
fn index(data: &[u8], name: &str) -> Result<Cache, Unparsed> {
    if data.starts_with(BIN_SIGNATURE) || BIN_SIGNATURE.starts_with(data) {
        return Ok(index_bin(data));
    }
    if is_bin_name(name) {
        return Err(Unparsed::NotACache(
            "It does not start with RDP8bmp, so it is not an RDP bitmap cache".into(),
        ));
    }
    if !is_bmc_name(name) && !plausible_bmc_header(data, None) {
        return Err(Unparsed::NotACache(
            "It is neither a Cache????.bin file (RDP8bmp) nor a .bmc file".into(),
        ));
    }
    index_bmc(data, name).map_err(Unparsed::Failed)
}

fn index_bin(data: &[u8]) -> Cache {
    let mut cache = Cache {
        format: Format::Bin,
        version: None,
        entries: Vec::new(),
        problems: Vec::new(),
        empty_slots: 0,
    };
    if data.len() < BIN_HEADER {
        cache.problems.push(format!(
            "The file ends inside its 12-byte header ({} bytes)",
            data.len()
        ));
        return cache;
    }
    cache.version = Some(u32_at(data, 8));
    let mut at = BIN_HEADER;
    while at < data.len() {
        let rest = &data[at..];
        if rest.len() < BIN_TILE_HEADER {
            if !all_zero(rest) {
                cache.problems.push(format!(
                    "The last {} bytes (offset {at}) are too few for a tile",
                    rest.len()
                ));
            }
            break;
        }
        let key = u64_at(data, at);
        let width = usize::from(u16_at(data, at + 8));
        let height = usize::from(u16_at(data, at + 10));
        if !(1..=TILE).contains(&width) || !(1..=TILE).contains(&height) {
            if !all_zero(rest) {
                cache.problems.push(format!(
                    "Offset {at}: a tile header of {width} × {height} pixels, which no tile has; the remaining {} bytes are not read",
                    rest.len()
                ));
            }
            break;
        }
        let start = at + BIN_TILE_HEADER;
        let size = width * height * 4;
        let end = (start + size).min(data.len());
        let mut problems = Vec::new();
        if start + size > data.len() {
            problems.push(format!(
                "The file ends inside the tile's pixels ({} of {size} bytes)",
                end - start
            ));
        }
        cache.entries.push(Entry {
            index: cache.entries.len(),
            offset: at,
            key,
            width,
            height,
            depth: 4,
            flags: None,
            compressed: false,
            data: start..end,
            rest: None,
            problems,
        });
        at = start + size;
    }
    cache
}

fn index_bmc(data: &[u8], name: &str) -> Result<Cache, String> {
    let depth = bmc_depth(data, name).ok_or(
        "Could not tell its tiles' color depth: the name is not bcache2.bmc, bcache22.bmc, bcache23.bmc or bcache24.bmc, and its tiles do not show it",
    )?;
    let capacity = TILE * TILE * depth;
    let slot = BMC_TILE_HEADER + capacity;
    let mut cache = Cache {
        format: Format::Bmc,
        version: None,
        entries: Vec::new(),
        problems: Vec::new(),
        empty_slots: 0,
    };
    let mut at = 0;
    while at < data.len() {
        let rest = &data[at..];
        if rest.len() < BMC_TILE_HEADER {
            if !all_zero(rest) {
                cache.problems.push(format!(
                    "The last {} bytes (offset {at}) are too few for a tile",
                    rest.len()
                ));
            }
            break;
        }
        if all_zero(&rest[..BMC_TILE_HEADER]) {
            cache.empty_slots += 1;
            at += slot;
            continue;
        }
        let key = u64_at(data, at);
        let width = usize::from(u16_at(data, at + 8));
        let height = usize::from(u16_at(data, at + 10));
        let length = u32_at(data, at + 12) as usize;
        let flags = u32_at(data, at + 16);
        if !(1..=TILE).contains(&width)
            || !(1..=TILE).contains(&height)
            || length == 0
            || length > capacity
        {
            cache.problems.push(format!(
                "Offset {at}: an unreadable tile header ({width} × {height} pixels, {length} bytes of data)"
            ));
            at += slot;
            continue;
        }
        let compressed = flags & BMC_COMPRESSED != 0;
        let start = at + BMC_TILE_HEADER;
        let end = (start + length).min(data.len());
        let mut problems = Vec::new();
        if start + length > data.len() {
            problems.push(format!(
                "The file ends inside the tile's data ({} of {length} bytes)",
                end - start
            ));
        }
        let mut rest_range = None;
        if !compressed {
            if stride(width, height, depth, length).is_none() {
                problems.push(format!(
                    "Its {length} bytes of data do not fit {width} × {height} pixels at {} bits",
                    depth * 8
                ));
            } else if width * height < TILE * TILE {
                let rest_end = (at + slot).min(data.len());
                if start + length < rest_end {
                    rest_range = Some(start + length..rest_end);
                }
            }
        }
        cache.entries.push(Entry {
            index: cache.entries.len(),
            offset: at,
            key,
            width,
            height,
            depth,
            flags: Some(flags),
            compressed,
            data: start..end,
            rest: rest_range,
            problems,
        });
        at += slot;
    }
    Ok(cache)
}

/// A `.bmc` tile header that a tile could have: 1 to 64 pixels each way, and data that fits the
/// slot (exactly the pixels, when uncompressed). With no `depth`, any of the four depths.
fn plausible_bmc_header(header: &[u8], depth: Option<usize>) -> bool {
    if header.len() < BMC_TILE_HEADER {
        return false;
    }
    let width = usize::from(u16_at(header, 8));
    let height = usize::from(u16_at(header, 10));
    let length = u32_at(header, 12) as usize;
    let flags = u32_at(header, 16);
    if !(1..=TILE).contains(&width) || !(1..=TILE).contains(&height) || length == 0 {
        return false;
    }
    let depths = depth.map_or(1..=4, |d| d..=d);
    depths.into_iter().any(|d| {
        length <= TILE * TILE * d
            && (flags & BMC_COMPRESSED != 0 || stride(width, height, d, length).is_some())
    })
}

/// A `.bmc` file's bytes per pixel: by its name; else by its first uncompressed tile, whose data
/// is its pixels; else the depth whose slot size puts a plausible header after the first slot.
fn bmc_depth(data: &[u8], name: &str) -> Option<usize> {
    if let Some(depth) = bmc_name_depth(name) {
        return Some(depth);
    }
    if data.len() >= BMC_TILE_HEADER && u32_at(data, 16) & BMC_COMPRESSED == 0 {
        let width = usize::from(u16_at(data, 8));
        let height = usize::from(u16_at(data, 10));
        let length = u32_at(data, 12) as usize;
        if let Some(depth) = (1..=4).find(|&d| {
            (1..=TILE).contains(&width)
                && (1..=TILE).contains(&height)
                && stride(width, height, d, length).is_some()
        }) {
            return Some(depth);
        }
    }
    (1..=4).find(|&d| {
        let next = BMC_TILE_HEADER + TILE * TILE * d;
        data.get(next..next + BMC_TILE_HEADER)
            .is_some_and(|h| plausible_bmc_header(h, Some(d)))
    })
}

/// The row length of `length` bytes of uncompressed `.bmc` pixels: `width × depth`, or that
/// rounded up to 4 bytes, as RDP pads bitmap rows.
fn stride(width: usize, height: usize, depth: usize, length: usize) -> Option<usize> {
    let tight = width * depth;
    let padded = tight.div_ceil(4) * 4;
    [tight, padded]
        .into_iter()
        .find(|&row| row * height == length)
}

fn all_zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|&b| b == 0)
}

fn u16_at(data: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([data[at], data[at + 1]])
}

fn u32_at(data: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(data[at..at + 4].try_into().expect("4 bytes"))
}

fn u64_at(data: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(data[at..at + 8].try_into().expect("8 bytes"))
}

// --- pixels ---------------------------------------------------------------------------------

/// A tile's pixels, RGB, top row first.
#[derive(Debug, PartialEq, Eq)]
struct Image {
    width: usize,
    height: usize,
    rgb: Vec<u8>,
}

impl Image {
    fn single_color(&self) -> bool {
        let pixels = self.rgb.as_chunks::<3>().0;
        pixels.iter().all(|pixel| pixel == &pixels[0])
    }

    fn row(&self, y: usize) -> &[u8] {
        &self.rgb[y * self.width * 3..(y + 1) * self.width * 3]
    }
}

fn decode(entry: &Entry, data: &[u8], format: Format) -> Result<Image, String> {
    let src = &data[entry.data.clone()];
    let (width, height, depth) = (entry.width, entry.height, entry.depth);
    let (pixels, row, bottom_up) = match format {
        Format::Bin => {
            if src.len() != width * height * 4 {
                return Err("Its pixels are cut short".into());
            }
            (src.to_vec(), width * 4, false)
        }
        Format::Bmc if entry.compressed => {
            let (pixels, row) = rle::decompress(src, width, height, depth)
                .map_err(|e| format!("Could not decompress it: {e}"))?;
            (pixels, row, true)
        }
        Format::Bmc => {
            let row = stride(width, height, depth, src.len())
                .ok_or_else(|| "Its data does not fit its size".to_string())?;
            (src.to_vec(), row, true)
        }
    };
    Ok(Image {
        width,
        height,
        rgb: to_rgb(&pixels, width, height, depth, row, bottom_up),
    })
}

/// `height` rows of `width` pixels of `depth` bytes, `row` bytes apart, as RGB top row first:
/// 8-bit palette indexes in the Windows default palette, 16-bit RGB565, 24-bit BGR, 32-bit
/// BGRX/BGRA (the alpha byte is not used).
fn to_rgb(
    pixels: &[u8],
    width: usize,
    height: usize,
    depth: usize,
    row: usize,
    bottom_up: bool,
) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(width * height * 3);
    for y in 0..height {
        let source = if bottom_up { height - 1 - y } else { y };
        let line = &pixels[source * row..source * row + width * depth];
        for pixel in line.chunks_exact(depth) {
            rgb.extend_from_slice(&match depth {
                1 => palette(pixel[0]),
                2 => rgb565(u16::from_le_bytes([pixel[0], pixel[1]])),
                _ => [pixel[2], pixel[1], pixel[0]],
            });
        }
    }
    rgb
}

/// The Windows default 256-color palette, which 8-bit tiles index: the 20 static colors at both
/// ends and, between them, red and green in 8 steps and blue in 4. The session's own palette is
/// not kept in the cache, so 8-bit colors can differ from what the screen showed.
fn palette(index: u8) -> [u8; 3] {
    const LOW: [[u8; 3]; 10] = [
        [0, 0, 0],
        [128, 0, 0],
        [0, 128, 0],
        [128, 128, 0],
        [0, 0, 128],
        [128, 0, 128],
        [0, 128, 128],
        [192, 192, 192],
        [192, 220, 192],
        [166, 202, 240],
    ];
    const HIGH: [[u8; 3]; 10] = [
        [255, 251, 240],
        [160, 160, 164],
        [128, 128, 128],
        [255, 0, 0],
        [0, 255, 0],
        [255, 255, 0],
        [0, 0, 255],
        [255, 0, 255],
        [0, 255, 255],
        [255, 255, 255],
    ];
    match index {
        0..=9 => LOW[usize::from(index)],
        246..=255 => HIGH[usize::from(index - 246)],
        i => [(i & 7) * 32, ((i >> 3) & 7) * 32, (i >> 6) * 64],
    }
}

/// A 16-bit RGB565 pixel as 8-bit RGB, the high bits repeated in the low ones.
fn rgb565(v: u16) -> [u8; 3] {
    let r = ((v >> 11) & 0x1F) as u8;
    let g = ((v >> 5) & 0x3F) as u8;
    let b = (v & 0x1F) as u8;
    [
        (r << 3) | (r >> 2),
        (g << 2) | (g >> 4),
        (b << 3) | (b >> 2),
    ]
}

/// The end of an earlier tile left in a `.bmc` slot after an uncompressed tile smaller than
/// 64 × 64: the slot's last whole 64-pixel rows, as the earlier, full-width tile had them. None
/// when that is less than a row or all zero.
fn remnant(entry: &Entry, rest: &[u8]) -> Option<Image> {
    let row = TILE * entry.depth;
    let rows = rest.len() / row;
    if rows == 0 || all_zero(rest) {
        return None;
    }
    let pixels = &rest[rest.len() - rows * row..];
    Some(Image {
        width: TILE,
        height: rows,
        rgb: to_rgb(pixels, TILE, rows, entry.depth, row, true),
    })
}

/// RDP's interleaved RLE bitmap compression, as MS-RDPBCGR 3.1.9 specifies its decompression.
mod rle {
    const BG_RUN: u8 = 0x0;
    const FG_RUN: u8 = 0x1;
    const FGBG_IMAGE: u8 = 0x2;
    const COLOR_RUN: u8 = 0x3;
    const COLOR_IMAGE: u8 = 0x4;
    const SET_FG_FG_RUN: u8 = 0xC;
    const SET_FG_FGBG_IMAGE: u8 = 0xD;
    const DITHERED_RUN: u8 = 0xE;
    const MEGA_BG_RUN: u8 = 0xF0;
    const MEGA_FG_RUN: u8 = 0xF1;
    const MEGA_FGBG_IMAGE: u8 = 0xF2;
    const MEGA_COLOR_RUN: u8 = 0xF3;
    const MEGA_COLOR_IMAGE: u8 = 0xF4;
    const MEGA_SET_FG_RUN: u8 = 0xF6;
    const MEGA_SET_FGBG_IMAGE: u8 = 0xF7;
    const MEGA_DITHERED_RUN: u8 = 0xF8;
    const SPECIAL_FGBG_1: u8 = 0xF9;
    const SPECIAL_FGBG_2: u8 = 0xFA;
    const WHITE: u8 = 0xFD;
    const BLACK: u8 = 0xFE;

    /// `height` rows of `width` pixels of `depth` bytes, bottom row first as RDP sends bitmaps,
    /// and the row length: `width × depth`, or that rounded up to 4 bytes when that is what the
    /// data fills.
    pub(super) fn decompress(
        src: &[u8],
        width: usize,
        height: usize,
        depth: usize,
    ) -> Result<(Vec<u8>, usize), String> {
        let tight = width * depth;
        let padded = tight.div_ceil(4) * 4;
        match decode(src, tight, height, depth) {
            Ok(pixels) => Ok((pixels, tight)),
            Err(e) if padded != tight => decode(src, padded, height, depth)
                .map(|pixels| (pixels, padded))
                .map_err(|_| e),
            Err(e) => Err(e),
        }
    }

    /// The order code of a header byte: the 8-bit codes 0xF0–0xFE, the 4-bit lite codes
    /// 0xC–0xE, the 3-bit regular codes 0x0–0x4. None for the codes no order has.
    fn code(header: u8) -> Option<u8> {
        match header {
            0xF5 | 0xFB | 0xFC | 0xFF | 0xA0..=0xBF => None,
            0xF0..=0xFE => Some(header),
            0xC0..=0xEF => Some(header >> 4),
            _ => Some(header >> 5),
        }
    }

    /// An order's run length and header size.
    fn run_length(code: u8, order: &[u8]) -> Option<(usize, usize)> {
        let header = order[0];
        let next = || order.get(1).map(|&b| usize::from(b));
        Some(match code {
            BG_RUN | FG_RUN | COLOR_RUN | COLOR_IMAGE => match usize::from(header & 0x1F) {
                0 => (next()? + 32, 2),
                n => (n, 1),
            },
            FGBG_IMAGE => match usize::from(header & 0x1F) {
                0 => (next()? + 1, 2),
                n => (n * 8, 1),
            },
            SET_FG_FG_RUN | DITHERED_RUN => match usize::from(header & 0x0F) {
                0 => (next()? + 16, 2),
                n => (n, 1),
            },
            SET_FG_FGBG_IMAGE => match usize::from(header & 0x0F) {
                0 => (next()? + 1, 2),
                n => (n * 8, 1),
            },
            MEGA_BG_RUN | MEGA_FG_RUN | MEGA_FGBG_IMAGE | MEGA_COLOR_RUN | MEGA_COLOR_IMAGE
            | MEGA_SET_FG_RUN | MEGA_SET_FGBG_IMAGE | MEGA_DITHERED_RUN => (
                usize::from(u16::from_le_bytes([*order.get(1)?, *order.get(2)?])),
                3,
            ),
            _ => (0, 1),
        })
    }

    struct Output {
        bytes: Vec<u8>,
        size: usize,
        row: usize,
        depth: usize,
    }

    impl Output {
        fn push(&mut self, pixel: u32) -> Result<(), String> {
            if self.bytes.len() + self.depth > self.size {
                return Err("its data decodes to more pixels than the tile has".into());
            }
            self.bytes
                .extend_from_slice(&pixel.to_le_bytes()[..self.depth]);
            Ok(())
        }

        /// The pixel one row above the next one.
        fn above(&self) -> u32 {
            let at = self.bytes.len() - self.row;
            let mut bytes = [0u8; 4];
            bytes[..self.depth].copy_from_slice(&self.bytes[at..at + self.depth]);
            u32::from_le_bytes(bytes)
        }

        /// `bits` pixels of a foreground/background image, from the mask's low bit up: a set
        /// bit is the foreground (XORed with the pixel above after the first row), a clear one
        /// black on the first row and the pixel above after it.
        fn fgbg(&mut self, mask: u8, bits: usize, fg: u32, first_line: bool) -> Result<(), String> {
            for bit in 0..bits {
                let set = mask & (1 << bit) != 0;
                let pixel = match (first_line, set) {
                    (true, true) => fg,
                    (true, false) => 0,
                    (false, true) => self.above() ^ fg,
                    (false, false) => self.above(),
                };
                self.push(pixel)?;
            }
            Ok(())
        }
    }

    fn pixel(src: &[u8], at: &mut usize, depth: usize) -> Result<u32, String> {
        let bytes = src
            .get(*at..*at + depth)
            .ok_or("the data ends inside an order")?;
        *at += depth;
        let mut value = [0u8; 4];
        value[..depth].copy_from_slice(bytes);
        Ok(u32::from_le_bytes(value))
    }

    fn decode(src: &[u8], row: usize, height: usize, depth: usize) -> Result<Vec<u8>, String> {
        let white = match depth {
            1 => 0xFF,
            2 => 0xFFFF,
            3 => 0xFF_FFFF,
            _ => 0xFFFF_FFFF,
        };
        let mut out = Output {
            bytes: Vec::with_capacity(row * height),
            size: row * height,
            row,
            depth,
        };
        let mut fg = white;
        let mut insert_fg = false;
        let mut first_line = true;
        let mut at = 0;
        while at < src.len() {
            if first_line && out.bytes.len() >= row {
                first_line = false;
                insert_fg = false;
            }
            let header = src[at];
            let code = code(header)
                .ok_or_else(|| format!("no order has the code 0x{header:02X} (byte {at})"))?;
            let (mut run, header_size) =
                run_length(code, &src[at..]).ok_or("the data ends inside an order")?;
            at += header_size;
            if code == BG_RUN || code == MEGA_BG_RUN {
                // Back-to-back background runs: the second starts with a foreground pixel.
                if insert_fg && run > 0 {
                    out.push(if first_line { fg } else { out.above() ^ fg })?;
                    run -= 1;
                }
                for _ in 0..run {
                    out.push(if first_line { 0 } else { out.above() })?;
                }
                insert_fg = true;
                continue;
            }
            insert_fg = false;
            match code {
                FG_RUN | MEGA_FG_RUN | SET_FG_FG_RUN | MEGA_SET_FG_RUN => {
                    if code == SET_FG_FG_RUN || code == MEGA_SET_FG_RUN {
                        fg = pixel(src, &mut at, depth)?;
                    }
                    for _ in 0..run {
                        out.push(if first_line { fg } else { out.above() ^ fg })?;
                    }
                }
                DITHERED_RUN | MEGA_DITHERED_RUN => {
                    let a = pixel(src, &mut at, depth)?;
                    let b = pixel(src, &mut at, depth)?;
                    for _ in 0..run {
                        out.push(a)?;
                        out.push(b)?;
                    }
                }
                COLOR_RUN | MEGA_COLOR_RUN => {
                    let a = pixel(src, &mut at, depth)?;
                    for _ in 0..run {
                        out.push(a)?;
                    }
                }
                FGBG_IMAGE | MEGA_FGBG_IMAGE | SET_FG_FGBG_IMAGE | MEGA_SET_FGBG_IMAGE => {
                    if code == SET_FG_FGBG_IMAGE || code == MEGA_SET_FGBG_IMAGE {
                        fg = pixel(src, &mut at, depth)?;
                    }
                    while run > 0 {
                        let mask = *src.get(at).ok_or("the data ends inside an order")?;
                        at += 1;
                        let bits = run.min(8);
                        out.fgbg(mask, bits, fg, first_line)?;
                        run -= bits;
                    }
                }
                COLOR_IMAGE | MEGA_COLOR_IMAGE => {
                    for _ in 0..run {
                        let a = pixel(src, &mut at, depth)?;
                        out.push(a)?;
                    }
                }
                SPECIAL_FGBG_1 => out.fgbg(0x03, 8, fg, first_line)?,
                SPECIAL_FGBG_2 => out.fgbg(0x05, 8, fg, first_line)?,
                WHITE => out.push(white)?,
                BLACK => out.push(0)?,
                _ => unreachable!("code() gives only these codes"),
            }
        }
        if out.bytes.len() != out.size {
            return Err(format!(
                "its data fills {} of the tile's {} bytes",
                out.bytes.len(),
                out.size
            ));
        }
        Ok(out.bytes)
    }

    #[cfg(test)]
    pub(super) fn decode_for_test(
        src: &[u8],
        row: usize,
        height: usize,
        depth: usize,
    ) -> Result<Vec<u8>, String> {
        decode(src, row, height, depth)
    }
}

// --- images ---------------------------------------------------------------------------------

/// A 24-bit BMP of `image`.
fn bmp(image: &Image) -> Vec<u8> {
    let row = (image.width * 3).div_ceil(4) * 4;
    let pixels = row * image.height;
    let mut out = Vec::with_capacity(54 + pixels);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&u32::try_from(54 + pixels).unwrap_or(u32::MAX).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(image.width as i32).to_le_bytes());
    // Positive: the rows are stored bottom row first.
    out.extend_from_slice(&(image.height as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&u32::try_from(pixels).unwrap_or(u32::MAX).to_le_bytes());
    // 72 dpi.
    out.extend_from_slice(&2835u32.to_le_bytes());
    out.extend_from_slice(&2835u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    for y in (0..image.height).rev() {
        let start = out.len();
        for pixel in image.row(y).as_chunks::<3>().0 {
            out.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
        }
        out.resize(start + row, 0);
    }
    out
}

/// A PNG written a row at a time: 8-bit RGB, each row filtered against the one above.
struct Png {
    zlib: ZlibEncoder<Idat>,
    previous: Vec<u8>,
    filtered: Vec<u8>,
}

/// The compressed rows, written to the file in IDAT chunks of about 1 MiB.
struct Idat {
    file: BufWriter<File>,
    buffer: Vec<u8>,
}

impl Write for Idat {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(data);
        if self.buffer.len() >= 1 << 20 {
            chunk(&mut self.file, b"IDAT", &self.buffer)?;
            self.buffer.clear();
        }
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Png {
    fn create(file: File, width: usize, height: usize) -> io::Result<Self> {
        let mut file = BufWriter::new(file);
        file.write_all(b"\x89PNG\r\n\x1a\n")?;
        let mut header = Vec::with_capacity(13);
        header.extend_from_slice(
            &u32::try_from(width)
                .map_err(io::Error::other)?
                .to_be_bytes(),
        );
        header.extend_from_slice(
            &u32::try_from(height)
                .map_err(io::Error::other)?
                .to_be_bytes(),
        );
        // 8 bits per sample, RGB, deflate, adaptive filtering, no interlace.
        header.extend_from_slice(&[8, 2, 0, 0, 0]);
        chunk(&mut file, b"IHDR", &header)?;
        let idat = Idat {
            file,
            buffer: Vec::new(),
        };
        Ok(Self {
            zlib: ZlibEncoder::new(idat, Compression::default()),
            previous: vec![0; width * 3],
            filtered: Vec::with_capacity(width * 3 + 1),
        })
    }

    fn row(&mut self, rgb: &[u8]) -> io::Result<()> {
        // Filter type 2 (Up): screen content repeats from row to row.
        self.filtered.clear();
        self.filtered.push(2);
        self.filtered.extend(
            rgb.iter()
                .zip(&self.previous)
                .map(|(byte, above)| byte.wrapping_sub(*above)),
        );
        self.zlib.write_all(&self.filtered)?;
        self.previous.copy_from_slice(rgb);
        Ok(())
    }

    fn finish(self) -> io::Result<()> {
        // The end of the zlib stream (at least its checksum) makes the last IDAT chunk.
        let mut idat = self.zlib.finish()?;
        chunk(&mut idat.file, b"IDAT", &idat.buffer)?;
        chunk(&mut idat.file, b"IEND", &[])?;
        idat.file.flush()
    }
}

fn chunk(out: &mut impl Write, kind: &[u8; 4], data: &[u8]) -> io::Result<()> {
    out.write_all(
        &u32::try_from(data.len())
            .map_err(io::Error::other)?
            .to_be_bytes(),
    )?;
    out.write_all(kind)?;
    out.write_all(data)?;
    let mut crc = Crc::new();
    crc.update(kind);
    crc.update(data);
    out.write_all(&crc.sum().to_be_bytes())
}

/// A collage of a cache file's tiles in file order, `columns` to a row, each in a 64 × 64 cell
/// from its top left; magenta where no tile is.
struct Collage {
    png: Png,
    shown: String,
    columns: usize,
    cells: Vec<Option<Image>>,
    line: Vec<u8>,
}

impl Collage {
    fn add(&mut self, image: Option<Image>) -> io::Result<()> {
        self.cells.push(image);
        if self.cells.len() == self.columns {
            self.flush()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        for y in 0..TILE {
            self.line.clear();
            for column in 0..self.columns {
                let filled = match self.cells.get(column) {
                    Some(Some(image)) if y < image.height => {
                        self.line.extend_from_slice(image.row(y));
                        image.width
                    }
                    _ => 0,
                };
                for _ in filled..TILE {
                    self.line.extend_from_slice(&PAD);
                }
            }
            self.png.row(&self.line)?;
        }
        self.cells.clear();
        Ok(())
    }

    fn finish(mut self) -> io::Result<()> {
        if !self.cells.is_empty() {
            self.flush()?;
        }
        self.png.finish()
    }
}

/// What `--extract` wrote.
#[derive(Default)]
struct Written {
    tiles: usize,
    remnants: usize,
    collages: usize,
}

/// `--extract`: a folder per cache file (`<user>/<cache file>`), with its tiles and collage.
struct Extractor {
    dir: PathBuf,
    /// The directory as the records show it: `dir` unless `--extract-shown-as` is given.
    shown: PathBuf,
    clobber: bool,
    /// Each tile too, not only the collages (no `--collage-only`).
    tiles: bool,
    columns: usize,
    /// The cache files, which are never overwritten.
    inputs: Vec<PathBuf>,
    folders: HashSet<String>,
    written: Written,
}

impl Extractor {
    /// The folder of a cache file, relative to the extraction directory: the user's and the
    /// file's names, numbered when two cache files would share them.
    fn folder(&mut self, cache: &Path, user: Option<&str>) -> PathBuf {
        let name = safe_name(&file_name(cache));
        let base = match user {
            Some(user) => Path::new(&safe_name(user)).join(&name),
            None => PathBuf::from(&name),
        };
        let mut folder = base.clone();
        let mut n = 1;
        while !self.folders.insert(folder.to_string_lossy().to_lowercase()) {
            n += 1;
            folder = base.with_file_name(format!("{name}_{n}"));
        }
        folder
    }

    /// Create `folder/name` with `bytes`; its path as the records show it.
    fn write(&self, folder: &Path, name: &str, bytes: &[u8]) -> Result<String, String> {
        let (path, mut file) = self.create(folder, name)?;
        file.write_all(bytes)
            .map_err(|e| format!("Could not write {}: {e}", path.display()))?;
        Ok(self
            .shown
            .join(folder)
            .join(name)
            .to_string_lossy()
            .into_owned())
    }

    fn create(&self, folder: &Path, name: &str) -> Result<(PathBuf, File), String> {
        let directory = self.dir.join(folder);
        fs::create_dir_all(&directory)
            .map_err(|e| format!("Could not create {}: {e}", directory.display()))?;
        let path = directory.join(name);
        if path.exists() {
            if !self.clobber {
                return Err(format!(
                    "{} already exists; add -C to overwrite it",
                    path.display()
                ));
            }
            crate::registry::check_not_read(&path, || self.inputs.clone())?;
        }
        let file =
            File::create(&path).map_err(|e| format!("Could not create {}: {e}", path.display()))?;
        Ok((path, file))
    }

    /// The collage of a cache file of `tiles` tiles; none without tiles.
    fn start_collage(
        &self,
        folder: &Path,
        name: &str,
        tiles: usize,
    ) -> Result<Option<Collage>, String> {
        if tiles == 0 {
            return Ok(None);
        }
        let columns = self.columns.min(tiles);
        let rows = tiles.div_ceil(columns);
        let collage = format!("{name}_collage.png");
        let (path, file) = self.create(folder, &collage)?;
        let png = Png::create(file, columns * TILE, rows * TILE)
            .map_err(|e| format!("Could not write {}: {e}", path.display()))?;
        Ok(Some(Collage {
            png,
            shown: self
                .shown
                .join(folder)
                .join(&collage)
                .to_string_lossy()
                .into_owned(),
            columns,
            cells: Vec::with_capacity(columns),
            line: Vec::with_capacity(columns * TILE * 3),
        }))
    }
}

/// A name for a folder: Windows' reserved characters and control characters as `_`.
fn safe_name(name: &str) -> String {
    let name: String = name
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') {
                '_'
            } else {
                c
            }
        })
        .collect();
    if name.is_empty() || name == "." || name == ".." {
        "_".into()
    } else {
        name
    }
}

// --- output ---------------------------------------------------------------------------------

const CSV_HEADER: &[&str] = &[
    "SourceFile",
    "User",
    "TileIndex",
    "Width",
    "Height",
    "SingleColor",
    "TileImage",
    "Collage",
    "RemnantImage",
    "Key",
    "Format",
    "FormatVersion",
    "BitsPerPixel",
    "Compressed",
    "Flags",
    "Offset",
    "Problems",
];

fn text<T: ToString>(value: Option<T>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}

fn csv_row(r: &Record) -> Vec<String> {
    vec![
        r.source.clone(),
        text(r.user.as_ref()),
        r.index.to_string(),
        r.width.to_string(),
        r.height.to_string(),
        text(r.single_color),
        text(r.tile_image.as_ref()),
        text(r.collage.as_ref()),
        text(r.remnant_image.as_ref()),
        format!("{:016X}", r.key),
        r.format.name().to_owned(),
        text(r.version),
        r.bits.to_string(),
        text(r.compressed),
        text(r.flags.map(|f| format!("0x{f:08X}"))),
        r.offset.to_string(),
        r.problems.join("; "),
    ]
}

fn json_record(r: &Record) -> Value {
    json!({
        "SourceFile": r.source,
        "User": r.user,
        "TileIndex": r.index,
        "Width": r.width,
        "Height": r.height,
        "SingleColor": r.single_color,
        "TileImage": r.tile_image,
        "Collage": r.collage,
        "RemnantImage": r.remnant_image,
        "Key": format!("{:016X}", r.key),
        "Format": r.format.name(),
        "FormatVersion": r.version,
        "BitsPerPixel": r.bits,
        "Compressed": r.compressed,
        "Flags": r.flags.map(|f| format!("0x{f:08X}")),
        "Offset": r.offset,
        "Problems": r.problems,
    })
}

fn write_records(
    mut w: Box<dyn Write>,
    output_type: RdpCacheOutputType,
    parsed: &[Parsed],
) -> io::Result<()> {
    let records = parsed.iter().flat_map(|file| &file.records);
    if output_type == RdpCacheOutputType::Csv {
        let mut csv = csv::Writer::from_writer(w);
        csv.write_record(crate::output_schema::labels(CSV_HEADER.iter().copied())?)?;
        for record in records {
            csv.write_record(csv_row(record))?;
        }
        return csv.flush();
    }
    for record in records {
        let v = json_record(record);
        crate::output_schema::write_json(&mut w, &v, output_type == RdpCacheOutputType::Json)?;
        w.write_all(b"\n")?;
    }
    w.flush()
}

fn print_summary(
    parsed: &[Parsed],
    failed: usize,
    other: usize,
    extractor: Option<&Extractor>,
    no_color: bool,
) {
    p(JunteiColor::Green.rdg(no_color), "Results Summary:", true);
    let count = |format: Format| parsed.iter().filter(|f| f.format == format).count();
    let mut files = format!(
        "{} (Cache????.bin: {}, bcache*.bmc: {})",
        fmt_num(parsed.len()),
        fmt_num(count(Format::Bin)),
        fmt_num(count(Format::Bmc))
    );
    let empty = parsed.iter().filter(|f| f.empty).count();
    if empty > 0 {
        files.push_str(&format!(" (empty: {})", fmt_num(empty)));
    }
    if failed > 0 {
        files.push_str(&format!(" (could not parse: {})", fmt_num(failed)));
    }
    p_label(no_color, "Cache files parsed: ", &files);
    if other > 0 {
        p_label(
            no_color,
            "Other files with cache names (not RDP bitmap caches): ",
            &fmt_num(other),
        );
    }
    let records: Vec<&Record> = parsed.iter().flat_map(|f| &f.records).collect();
    let single = records
        .iter()
        .filter(|r| r.single_color == Some(true))
        .count();
    p_label(
        no_color,
        "Tiles: ",
        &format!(
            "{} (single-color: {})",
            fmt_num(records.len()),
            fmt_num(single)
        ),
    );
    let mut users: BTreeMap<&str, usize> = BTreeMap::new();
    for record in &records {
        *users
            .entry(record.user.as_deref().unwrap_or("(unknown)"))
            .or_default() += 1;
    }
    let mut shown: Vec<String> = users
        .iter()
        .take(5)
        .map(|(user, n)| format!("{user}: {}", fmt_num(*n)))
        .collect();
    if users.len() > shown.len() {
        shown.push("…".into());
    }
    if !users.is_empty() {
        p_label(no_color, "Tiles by user: ", &shown.join(", "));
    }
    let damaged = records.iter().filter(|r| !r.problems.is_empty()).count()
        + parsed.iter().filter(|f| !f.problems.is_empty()).count();
    p_label(no_color, "With problems: ", &fmt_num(damaged));
    if let Some(x) = extractor {
        let mut images = format!(
            "{} (collages: {}",
            x.shown.display(),
            fmt_num(x.written.collages)
        );
        if x.tiles {
            images.push_str(&format!(", tiles: {}", fmt_num(x.written.tiles)));
            if x.written.remnants > 0 {
                images.push_str(&format!(
                    ", ends of earlier tiles: {}",
                    fmt_num(x.written.remnants)
                ));
            }
        }
        images.push(')');
        p_label(no_color, "Images saved: ", &images);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `Cache????.bin` file of `tiles` (key, width, height, BGRA pixels top row first).
    pub(super) fn bin_file(version: u32, tiles: &[(u64, usize, usize, Vec<u8>)]) -> Vec<u8> {
        let mut out = BIN_SIGNATURE.to_vec();
        out.extend_from_slice(&version.to_le_bytes());
        for (key, width, height, pixels) in tiles {
            out.extend_from_slice(&key.to_le_bytes());
            out.extend_from_slice(&(*width as u16).to_le_bytes());
            out.extend_from_slice(&(*height as u16).to_le_bytes());
            out.extend_from_slice(pixels);
        }
        out
    }

    /// A `.bmc` slot of `depth` bytes per pixel: the header, the data, and `fill` for the rest.
    pub(super) fn bmc_slot(
        depth: usize,
        key: u64,
        width: usize,
        height: usize,
        flags: u32,
        data: &[u8],
        fill: u8,
    ) -> Vec<u8> {
        let mut out = key.to_le_bytes().to_vec();
        out.extend_from_slice(&(width as u16).to_le_bytes());
        out.extend_from_slice(&(height as u16).to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&flags.to_le_bytes());
        out.extend_from_slice(data);
        out.resize(BMC_TILE_HEADER + TILE * TILE * depth, fill);
        out
    }

    fn gradient(width: usize, height: usize) -> Vec<u8> {
        // BGRA: blue = x, green = y, red = x + y, alpha left at 0 as Windows often does.
        let mut out = Vec::new();
        for y in 0..height {
            for x in 0..width {
                out.extend_from_slice(&[x as u8 * 3, y as u8 * 3, (x + y) as u8, 0]);
            }
        }
        out
    }

    #[test]
    fn names_select_bin_and_bmc_caches() {
        for name in [
            "Cache0000.bin",
            "cache0003.BIN",
            "Cache12.bin",
            "bcache24.bmc",
            "BCACHE2.BMC",
            "bcache.bmc",
        ] {
            assert!(is_cache_name(name), "{name}");
        }
        for name in [
            "Cache.bin",
            "Cache00a0.bin",
            "xCache0000.bin",
            "bcache24.bin",
            "Cache0000.bin.bak",
            "bcache2x.bmc",
        ] {
            assert!(!is_cache_name(name), "{name}");
        }
        assert_eq!(bmc_name_depth("bcache2.bmc"), Some(1));
        assert_eq!(bmc_name_depth("BCACHE22.BMC"), Some(2));
        assert_eq!(bmc_name_depth("bcache23.bmc"), Some(3));
        assert_eq!(bmc_name_depth("bcache24.bmc"), Some(4));
        assert_eq!(bmc_name_depth("bcache25.bmc"), None);
    }

    #[test]
    fn bin_tiles_of_any_size_read_top_row_first_at_their_own_width() {
        let tiles = vec![
            (0x1122_3344_5566_7788u64, 64, 64, gradient(64, 64)),
            (2, 58, 28, gradient(58, 28)),
            (3, 62, 35, gradient(62, 35)),
        ];
        let data = bin_file(6, &tiles);
        let cache = index(&data, "Cache0000.bin").ok().unwrap();
        assert_eq!(cache.format, Format::Bin);
        assert_eq!(cache.version, Some(6));
        assert!(cache.problems.is_empty(), "{:?}", cache.problems);
        assert_eq!(cache.entries.len(), 3);
        assert_eq!(cache.entries[0].key, 0x1122_3344_5566_7788);
        assert_eq!(cache.entries[1].offset, 12 + 12 + 64 * 64 * 4);
        for (entry, (_, width, height, _)) in cache.entries.iter().zip(&tiles) {
            let image = decode(entry, &data, Format::Bin).unwrap();
            assert_eq!((image.width, image.height), (*width, *height));
            // Top left: (0, 0, 0); the pixel at (x, y) has blue 3x, green 3y, red x + y.
            for (x, y) in [
                (0, 0),
                (width - 1, 0),
                (0, height - 1),
                (width - 1, height - 1),
            ] {
                let at = (y * width + x) * 3;
                assert_eq!(
                    &image.rgb[at..at + 3],
                    &[(x + y) as u8, y as u8 * 3, x as u8 * 3],
                    "{width}x{height} at {x},{y}"
                );
            }
            assert!(!image.single_color());
        }
    }

    #[test]
    fn bin_damage_keeps_the_tiles_before_it() {
        let mut data = bin_file(
            6,
            &[(1, 64, 64, gradient(64, 64)), (2, 64, 64, gradient(64, 64))],
        );
        data.truncate(data.len() - 100);
        let cache = index_bin(&data);
        assert_eq!(cache.entries.len(), 2);
        assert!(cache.entries[0].problems.is_empty());
        assert!(cache.entries[1].problems[0].contains("16284 of 16384"));
        assert!(decode(&cache.entries[1], &data, Format::Bin).is_err());

        let mut data = bin_file(3, &[(1, 8, 8, gradient(8, 8))]);
        data.extend_from_slice(&[0xAB; 12]);
        let cache = index_bin(&data);
        assert_eq!(cache.entries.len(), 1);
        assert!(
            cache.problems[0].contains("which no tile has"),
            "{:?}",
            cache.problems
        );

        let mut data = bin_file(3, &[(1, 8, 8, gradient(8, 8))]);
        data.extend_from_slice(&[0; 40]);
        let cache = index_bin(&data);
        assert!(cache.problems.is_empty(), "trailing zeros are not damage");

        let cache = index_bin(&BIN_SIGNATURE[..]);
        assert!(cache.problems[0].contains("12-byte header"));
        assert!(matches!(
            index(b"something else entirely", "Cache0001.bin"),
            Err(Unparsed::NotACache(_))
        ));
    }

    #[test]
    fn bmc_uncompressed_tiles_at_every_depth_read_bottom_row_first() {
        // 8 × 8 tiles whose bottom row (stored first) is one color and the rest another.
        // Bytes per pixel, file name, bottom row's pixel, other rows' pixel, and their RGB.
        type Case = (usize, &'static str, Vec<u8>, Vec<u8>, [u8; 3], [u8; 3]);
        let cases: [Case; 4] = [
            (
                1,
                "bcache2.bmc",
                vec![249],
                vec![252],
                [255, 0, 0],
                [0, 0, 255],
            ),
            (
                2,
                "bcache22.bmc",
                0xF800u16.to_le_bytes().to_vec(),
                0x07E0u16.to_le_bytes().to_vec(),
                [255, 0, 0],
                [0, 255, 0],
            ),
            (
                3,
                "bcache23.bmc",
                vec![0, 0, 255],
                vec![255, 0, 0],
                [255, 0, 0],
                [0, 0, 255],
            ),
            (
                4,
                "bcache24.bmc",
                vec![0, 0, 255, 0],
                vec![0, 255, 0, 0],
                [255, 0, 0],
                [0, 255, 0],
            ),
        ];
        for (depth, name, bottom, other, bottom_rgb, other_rgb) in cases {
            let mut data = Vec::new();
            for y in 0..8 {
                for _ in 0..8 {
                    data.extend_from_slice(if y == 0 { &bottom } else { &other });
                }
            }
            let file = [
                bmc_slot(depth, 7, 8, 8, 0x11, &data, 0),
                bmc_slot(depth, 8, 8, 8, 0x11, &data, 0),
            ]
            .concat();
            let cache = index(&file, name).ok().unwrap();
            assert_eq!(cache.format, Format::Bmc);
            assert_eq!(cache.entries.len(), 2, "{name}");
            assert_eq!(cache.entries[1].offset, BMC_TILE_HEADER + 64 * 64 * depth);
            let image = decode(&cache.entries[0], &file, Format::Bmc).unwrap();
            assert_eq!(&image.rgb[..3], &other_rgb, "{name} top");
            assert_eq!(
                &image.rgb[7 * 8 * 3..7 * 8 * 3 + 3],
                &bottom_rgb,
                "{name} bottom"
            );
            // The slot's zeros after an 8 × 8 tile leave nothing of an earlier tile.
            assert!(
                remnant(
                    &cache.entries[0],
                    &file[cache.entries[0].rest.clone().unwrap()]
                )
                .is_none()
            );
        }
    }

    #[test]
    fn bmc_depth_from_tiles_when_renamed_and_empty_slots_skipped() {
        let data = vec![0x55u8; 16 * 16 * 2];
        let slot = bmc_slot(2, 9, 16, 16, 0x11, &data, 0);
        let file = [vec![0u8; BMC_TILE_HEADER + 64 * 64 * 2], slot.clone(), slot].concat();
        // The first slot was never used; the name gives no depth; the second slot's header does.
        assert_eq!(bmc_depth(&file, "renamed.dat"), Some(2));
        let cache = index_bmc(&file, "renamed.dat").unwrap();
        assert_eq!(cache.empty_slots, 1);
        assert_eq!(cache.entries.len(), 2);
        assert_eq!(cache.entries[0].index, 0);
        assert_eq!(cache.entries[0].offset, BMC_TILE_HEADER + 64 * 64 * 2);
    }

    #[test]
    fn bmc_short_tiles_keep_the_end_of_the_tile_before() {
        // An earlier 64 × 64 tile of gray rows 0..63 (bottom row first), then a 64 × 16 tile
        // written over its first 16 rows.
        let mut old = Vec::new();
        for y in 0..64u8 {
            old.extend(std::iter::repeat_n([y, y, y, 0], 64).flatten());
        }
        let new = vec![0xFFu8; 64 * 16 * 4];
        let mut slot = bmc_slot(4, 1, 64, 16, 0x11, &new, 0);
        slot[BMC_TILE_HEADER + new.len()..].copy_from_slice(&old[new.len()..]);
        let cache = index_bmc(&slot, "bcache24.bmc").unwrap();
        let entry = &cache.entries[0];
        let rest = remnant(entry, &slot[entry.rest.clone().unwrap()]).unwrap();
        assert_eq!((rest.width, rest.height), (64, 48));
        // Top row of the remnant is the old tile's top row (63); its bottom row is row 16.
        assert_eq!(&rest.rgb[..3], &[63, 63, 63]);
        assert_eq!(&rest.rgb[rest.rgb.len() - 3..], &[16, 16, 16]);
    }

    #[test]
    fn bmc_damage_is_reported_and_the_next_slots_still_read() {
        let good = bmc_slot(4, 1, 8, 8, 0x11, &[1u8; 8 * 8 * 4], 0);
        let mut bad = good.clone();
        bad[8..10].copy_from_slice(&500u16.to_le_bytes());
        let mut wrong_size = good.clone();
        wrong_size[12..16].copy_from_slice(&(8u32 * 8 * 4 - 4).to_le_bytes());
        let file = [good.clone(), bad, wrong_size, good].concat();
        let cache = index_bmc(&file, "bcache24.bmc").unwrap();
        // The 500-pixel header is not a tile; the others are, at their slots.
        assert_eq!(cache.problems.len(), 1);
        assert!(cache.problems[0].contains("unreadable tile header"));
        assert_eq!(cache.entries.len(), 3);
        assert_eq!(cache.entries[2].offset, 3 * (BMC_TILE_HEADER + 64 * 64 * 4));
        assert!(cache.entries[1].problems[0].contains("do not fit 8 × 8 pixels at 32 bits"));
        assert!(decode(&cache.entries[1], &file, Format::Bmc).is_err());
        assert!(cache.entries[2].problems.is_empty());
        let mut truncated = bmc_slot(4, 1, 8, 8, 0x11, &[1u8; 8 * 8 * 4], 0);
        truncated.truncate(BMC_TILE_HEADER + 10);
        let cache = index_bmc(&truncated, "bcache24.bmc").unwrap();
        assert!(cache.entries[0].problems[0].contains("10 of 256"));
        assert!(decode(&cache.entries[0], &truncated, Format::Bmc).is_err());
    }

    #[test]
    fn palette_and_rgb565_colors() {
        assert_eq!(palette(0), [0, 0, 0]);
        assert_eq!(palette(9), [166, 202, 240]);
        assert_eq!(palette(10), [64, 32, 0]);
        assert_eq!(palette(245), [160, 192, 192]);
        assert_eq!(palette(249), [255, 0, 0]);
        assert_eq!(palette(255), [255, 255, 255]);
        assert_eq!(rgb565(0xFFFF), [255, 255, 255]);
        assert_eq!(rgb565(0xF800), [255, 0, 0]);
        assert_eq!(rgb565(0x07E0), [0, 255, 0]);
        assert_eq!(rgb565(0x001F), [0, 0, 255]);
        assert_eq!(rgb565(0x8410), [132, 130, 132]);
    }

    /// Decompress 16-bit `src` into `width × height`, as u16 pixels in decoded order.
    fn rle16(src: &[u8], width: usize, height: usize) -> Result<Vec<u16>, String> {
        rle::decode_for_test(src, width * 2, height, 2).map(|bytes| {
            bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|p| u16::from_le_bytes(*p))
                .collect()
        })
    }

    #[test]
    fn rle_runs_on_the_first_line_and_after_it() {
        // 4 × 3. Row 1: color run of 0x1234 ×2, then a foreground run (white) ×2.
        // Row 2: background run ×4 (copies row 1). Row 3: foreground run ×4 (row 2 XOR white).
        let src = [0x62, 0x34, 0x12, 0x22, 0x04, 0x24];
        let px = rle16(&src, 4, 3).unwrap();
        assert_eq!(px[..4], [0x1234, 0x1234, 0xFFFF, 0xFFFF]);
        assert_eq!(px[4..8], px[..4]);
        assert_eq!(px[8..], [!0x1234u16, !0x1234u16, 0, 0]);
    }

    #[test]
    fn rle_back_to_back_background_runs_insert_a_foreground_pixel() {
        // First line of 8: two background runs of 4: the second starts with the foreground
        // (white), as both are on the first line.
        let px = rle16(&[0x04, 0x04], 8, 1).unwrap();
        assert_eq!(px, [0, 0, 0, 0, 0xFFFF, 0, 0, 0]);
        // A run that ends the first line, then one on the second: no pixel is inserted.
        let px = rle16(&[0x04, 0x04], 4, 2).unwrap();
        assert_eq!(px, [0; 8]);
        // A background run that starts on the first line and ends on the second: the next one
        // gets none either (bmc-tools inserts one here).
        let px = rle16(&[0x06, 0x02], 4, 2).unwrap();
        assert_eq!(px, [0; 8]);
        // Later lines: back-to-back runs insert the pixel above XOR the foreground.
        let px = rle16(&[0x64, 0x07, 0x00, 0x02, 0x02, 0x64, 0x01, 0x00], 4, 3).unwrap();
        assert_eq!(px[4..8], [7, 7, 0xFFFF ^ 7, 7]);
        assert_eq!(px[8..], [1; 4]);
        // A color run crossing into row 2, then background runs of 2 and 4 after the first line:
        // the second inserts a foreground pixel.
        let px = rle16(&[0x66, 0x07, 0x00, 0x02, 0x04], 4, 3).unwrap();
        assert_eq!(px[..8], [7; 8]);
        assert_eq!(px[8..], [0xFFFF ^ 7, 7, 7, 7]);
    }

    #[test]
    fn rle_orders_that_start_on_the_first_line_keep_its_rules_past_it() {
        // MS-RDPBCGR and FreeRDP decide the first line once per order: a foreground run of 6
        // on a 4-pixel line writes the foreground 6 times (bmc-tools XORs the last 2 with the
        // line above, giving black).
        let px = rle16(&[0x26, 0x62, 0x01, 0x00], 4, 2).unwrap();
        assert_eq!(px, [0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 1, 1]);
        // An FGBG image of 8 from the first line: set bits stay the foreground, clear bits
        // black, also on the second line.
        let px = rle16(&[0x41, 0xFF], 4, 2).unwrap();
        assert_eq!(px, [0xFFFF; 8]);
    }

    #[test]
    fn rle_foreground_background_images_and_set_foreground() {
        // First line: lite set-fg FGBG of 8 with fg 0x00AA and mask 0b1010_0101.
        let px = rle16(&[0xD1, 0xAA, 0x00, 0xA5], 8, 1).unwrap();
        assert_eq!(px, [0xAA, 0, 0xAA, 0, 0, 0xAA, 0, 0xAA]);
        // Regular FGBG with an extended length of 3 (0x40, then 3 - 1), on a second line of
        // color 5: set bits are 5 ^ white, clear bits 5.
        let px = rle16(&[0x63, 0x05, 0x00, 0x40, 0x02, 0x05], 3, 2).unwrap();
        assert_eq!(px[3..], [0xFFFF ^ 5, 5, 0xFFFF ^ 5]);
        // Special FGBG orders: masks 0x03 and 0x05 of 8 pixels with the default foreground.
        let px = rle16(&[0xF9, 0xFA], 16, 1).unwrap();
        assert_eq!(px[..8], [0xFFFF, 0xFFFF, 0, 0, 0, 0, 0, 0]);
        assert_eq!(px[8..], [0xFFFF, 0, 0xFFFF, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn rle_color_dithered_white_black_and_mega_orders() {
        // Color image of 2, dithered run of 2 pairs, white, black.
        let src = [
            0x82, 0x01, 0x00, 0x02, 0x00, 0xE2, 0x0A, 0x00, 0x0B, 0x00, 0xFD, 0xFE,
        ];
        let px = rle16(&src, 8, 1).unwrap();
        assert_eq!(px, [1, 2, 0x0A, 0x0B, 0x0A, 0x0B, 0xFFFF, 0]);
        // Mega-mega color run of 300 and an extended regular color run (0x60, 300 - 32).
        let px = rle16(&[0xF3, 0x2C, 0x01, 0x09, 0x00], 300, 1).unwrap();
        assert!(px.iter().all(|&p| p == 9));
        let px = rle16(&[0x60, 0x0C, 0x09, 0x00], 44, 1).unwrap();
        assert_eq!(px.len(), 44);
        // Mega-mega set-foreground run, and a lite set-foreground run with extended length.
        let px = rle16(
            &[0xF6, 0x03, 0x00, 0x34, 0x12, 0xC0, 0x00, 0x56, 0x00],
            19,
            1,
        )
        .unwrap();
        assert_eq!(px[..3], [0x1234; 3]);
        assert_eq!(px[3..], [0x0056; 16]);
    }

    #[test]
    fn rle_errors_are_reported_not_guessed() {
        assert!(rle16(&[0xA0], 4, 1).unwrap_err().contains("0xA0"));
        assert!(rle16(&[0xF5], 4, 1).is_err());
        assert!(
            rle16(&[0x63, 0x01], 4, 1)
                .unwrap_err()
                .contains("ends inside")
        );
        assert!(
            rle16(&[0x65, 0x01, 0x00], 4, 1)
                .unwrap_err()
                .contains("more pixels")
        );
        assert!(
            rle16(&[0x62, 0x01, 0x00], 4, 1)
                .unwrap_err()
                .contains("fills 4 of")
        );
    }

    #[test]
    fn rle_eight_and_twentyfour_bit_pixels() {
        let px = rle::decode_for_test(&[0x63, 0x7F, 0xFD], 4, 1, 1).unwrap();
        assert_eq!(px, [0x7F, 0x7F, 0x7F, 0xFF]);
        let px = rle::decode_for_test(&[0x62, 0x01, 0x02, 0x03, 0xFE], 9, 1, 3).unwrap();
        assert_eq!(px, [1, 2, 3, 1, 2, 3, 0, 0, 0]);
    }

    #[test]
    fn compressed_bmc_tiles_decode_bottom_row_first() {
        // 16-bit 4 × 2: a color run of red fills the first row decoded, the bottom one; a color
        // run of blue the second, the top one.
        let src = [0x64, 0x00, 0xF8, 0x64, 0x1F, 0x00];
        let file = bmc_slot(2, 1, 4, 2, 0x08 | 0x11, &src, 0);
        let cache = index_bmc(&file, "bcache22.bmc").unwrap();
        let entry = &cache.entries[0];
        assert!(entry.compressed);
        assert!(entry.rest.is_none(), "compressed slots keep no remnant");
        let image = decode(entry, &file, Format::Bmc).unwrap();
        assert_eq!(
            &image.rgb[..3],
            &[0, 0, 255],
            "top row: blue (decoded second)"
        );
        assert_eq!(
            &image.rgb[12..15],
            &[255, 0, 0],
            "bottom row: red (decoded first)"
        );
    }

    #[test]
    fn bmp_and_png_are_well_formed() {
        let image = Image {
            width: 3,
            height: 2,
            rgb: vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
            ],
        };
        let b = bmp(&image);
        assert_eq!(&b[..2], b"BM");
        assert_eq!(u32_at(&b, 2) as usize, b.len());
        assert_eq!(u32_at(&b, 18), 3);
        assert_eq!(u32_at(&b, 22), 2);
        // Rows of 9 bytes padded to 12, bottom row first, BGR.
        assert_eq!(b.len(), 54 + 24);
        assert_eq!(&b[54..57], &[12, 11, 10]);
        assert_eq!(&b[66..69], &[3, 2, 1]);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.png");
        let mut png = Png::create(File::create(&path).unwrap(), 3, 2).unwrap();
        png.row(image.row(0)).unwrap();
        png.row(image.row(1)).unwrap();
        png.finish().unwrap();
        let bytes = fs::read(&path).unwrap();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        // Chunks: IHDR, IDAT, IEND, each with a valid CRC.
        let mut at = 8;
        let mut kinds = Vec::new();
        let mut idat = Vec::new();
        while at < bytes.len() {
            let len = u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
            let kind = &bytes[at + 4..at + 8];
            let data = &bytes[at + 8..at + 8 + len];
            let mut crc = Crc::new();
            crc.update(kind);
            crc.update(data);
            assert_eq!(
                crc.sum(),
                u32::from_be_bytes(bytes[at + 8 + len..at + 12 + len].try_into().unwrap())
            );
            if kind == b"IDAT" {
                idat.extend_from_slice(data);
            }
            kinds.push(String::from_utf8_lossy(kind).into_owned());
            at += 12 + len;
        }
        assert_eq!(kinds, ["IHDR", "IDAT", "IEND"]);
        let mut raw = Vec::new();
        io::Read::read_to_end(&mut flate2::read::ZlibDecoder::new(&idat[..]), &mut raw).unwrap();
        // Up filter: row 1 as is, row 2 as the difference from row 1 (9 each byte).
        assert_eq!(raw[0], 2);
        assert_eq!(&raw[1..10], &image.rgb[..9]);
        assert_eq!(raw[10], 2);
        assert_eq!(&raw[11..20], &[9; 9]);
    }

    #[test]
    fn extracted_folders_are_per_user_and_cache_file_and_numbered() {
        let dir = tempfile::tempdir().unwrap();
        let mut x = Extractor {
            dir: dir.path().to_path_buf(),
            shown: PathBuf::from("./images"),
            clobber: false,
            tiles: true,
            columns: 64,
            inputs: Vec::new(),
            folders: HashSet::new(),
            written: Written::default(),
        };
        let a = x.folder(
            Path::new("C/Users/alice/Cache/Cache0000.bin"),
            Some("alice"),
        );
        let b = x.folder(
            Path::new("C/Windows.old/Users/alice/Cache/Cache0000.bin"),
            Some("alice"),
        );
        let c = x.folder(Path::new("Cache0000.bin"), None);
        assert_eq!(a, Path::new("alice").join("Cache0000.bin"));
        assert_eq!(b, Path::new("alice").join("Cache0000.bin_2"));
        assert_eq!(c, Path::new("Cache0000.bin"));
        assert_eq!(safe_name("a:b*c"), "a_b_c");
        let shown = x.write(&a, "Cache0000.bin_0000.bmp", b"x").unwrap();
        assert_eq!(
            shown,
            Path::new("./images")
                .join(&a)
                .join("Cache0000.bin_0000.bmp")
                .to_string_lossy()
        );
        assert!(
            x.write(&a, "Cache0000.bin_0000.bmp", b"y")
                .unwrap_err()
                .contains("-C")
        );
        x.clobber = true;
        x.write(&a, "Cache0000.bin_0000.bmp", b"y").unwrap();
        // An input is never overwritten, even with -C.
        let input = dir.path().join(&a).join("Cache0000.bin_0001.bmp");
        fs::write(&input, b"evidence").unwrap();
        x.inputs = vec![input.clone()];
        assert!(x.write(&a, "Cache0000.bin_0001.bmp", b"z").is_err());
        assert_eq!(fs::read(&input).unwrap(), b"evidence");
    }

    #[test]
    fn collages_place_tiles_in_file_order_and_pad_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let x = Extractor {
            dir: dir.path().to_path_buf(),
            shown: dir.path().to_path_buf(),
            clobber: false,
            tiles: false,
            columns: 2,
            inputs: Vec::new(),
            folders: HashSet::new(),
            written: Written::default(),
        };
        let mut collage = x
            .start_collage(Path::new("f"), "Cache0000.bin", 3)
            .unwrap()
            .unwrap();
        let solid = |c: u8, w: usize, h: usize| Image {
            width: w,
            height: h,
            rgb: vec![c; w * h * 3],
        };
        collage.add(Some(solid(10, 64, 64))).unwrap();
        collage.add(Some(solid(20, 32, 16))).unwrap();
        collage.add(None).unwrap();
        collage.finish().unwrap();
        let bytes = fs::read(dir.path().join("f").join("Cache0000.bin_collage.png")).unwrap();
        // 2 columns × 2 rows of 64-pixel cells.
        assert_eq!(u32::from_be_bytes(bytes[16..20].try_into().unwrap()), 128);
        assert_eq!(u32::from_be_bytes(bytes[20..24].try_into().unwrap()), 128);
        let len = u32::from_be_bytes(bytes[33..37].try_into().unwrap()) as usize;
        let mut raw = Vec::new();
        io::Read::read_to_end(
            &mut flate2::read::ZlibDecoder::new(&bytes[41..41 + len]),
            &mut raw,
        )
        .unwrap();
        // Undo the Up filter.
        let stride = 128 * 3;
        let mut rows: Vec<Vec<u8>> = Vec::new();
        for line in raw.chunks_exact(stride + 1) {
            let previous = rows.last().cloned().unwrap_or_else(|| vec![0; stride]);
            rows.push(
                line[1..]
                    .iter()
                    .zip(&previous)
                    .map(|(a, b)| a.wrapping_add(*b))
                    .collect(),
            );
        }
        let at = |x: usize, y: usize| rows[y][x * 3..x * 3 + 3].to_vec();
        assert_eq!(at(0, 0), [10, 10, 10]);
        assert_eq!(at(64, 0), [20, 20, 20]);
        assert_eq!(at(64 + 32, 0), PAD, "beyond the narrow tile");
        assert_eq!(at(64, 16), PAD, "below the short tile");
        assert_eq!(at(0, 64), PAD, "no image for the third tile");
        assert_eq!(at(127, 127), PAD, "no fourth tile");
    }
}
