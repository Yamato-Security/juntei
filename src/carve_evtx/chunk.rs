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

//! EVTX chunks found outside a log file: checking that a 64 KiB block starting with the chunk
//! signature has a plausible header, undoing NTFS compression around it, and walking its records.
//! Field offsets follow the libevtx documentation of the format.

use sha2::{Digest, Sha256};

/// Every chunk is 64 KiB.
pub const CHUNK_SIZE: usize = 0x10000;
/// The chunk header, with its string and template tables.
pub const HEADER_SIZE: usize = 0x200;
/// The chunk signature.
pub const MAGIC: &[u8; 8] = b"ElfChnk\0";
/// The record signature.
const RECORD_MAGIC: [u8; 4] = [0x2a, 0x2a, 0x00, 0x00];
/// A record's header (signature, size, ID and time) plus the copy of its size at its end.
const MIN_RECORD: usize = 28;
/// More records than fit in a chunk can't be right.
const MAX_RECORDS: u64 = ((CHUNK_SIZE - HEADER_SIZE) / MIN_RECORD) as u64;
/// LZNT1 compresses each 4 KiB of a compression unit separately.
const LZNT1_BLOCK: usize = 0x1000;
/// Where a chunk starts in its compression unit: after the 4 KiB file header, and chunks are
/// 64 KiB, so always 4 KiB into a unit of 8 KiB or more.
const CHUNK_IN_UNIT: usize = 0x1000;

fn u16_at(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn u32_at(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or_default())
}

fn u64_at(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or_default())
}

/// The fields of a chunk header that carving uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub first_number: u64,
    pub last_number: u64,
    pub first_id: u64,
    pub last_id: u64,
    pub last_record_offset: u32,
    pub free_space_offset: u32,
    /// Written as an empty page (no allocated records); old records may remain in it.
    pub empty: bool,
    pub header_checksum_valid: bool,
    pub data_checksum_valid: bool,
}

/// The header checksum: CRC-32 of the first 120 bytes and of the string and template tables.
pub fn header_checksum(chunk: &[u8]) -> u32 {
    let mut hasher = crc32(&chunk[..0x78]);
    hasher.update(&chunk[0x80..HEADER_SIZE]);
    hasher.sum()
}

/// The records checksum: CRC-32 of the bytes from the end of the header to the free space.
pub fn data_checksum(chunk: &[u8], free_space_offset: u32) -> u32 {
    let end = (free_space_offset as usize).clamp(HEADER_SIZE, CHUNK_SIZE);
    crc32(&chunk[HEADER_SIZE..end]).sum()
}

fn crc32(data: &[u8]) -> flate2::Crc {
    let mut crc = flate2::Crc::new();
    crc.update(data);
    crc
}

impl Header {
    /// The header of a 64 KiB block, if it is a plausible chunk. Checksums are reported, not
    /// required: a partly overwritten chunk still holds readable records.
    pub fn parse(chunk: &[u8]) -> Option<Header> {
        if chunk.len() < CHUNK_SIZE || &chunk[..8] != MAGIC || u32_at(chunk, 0x28) != 0x80 {
            return None;
        }
        let first_number = u64_at(chunk, 0x08);
        let last_number = u64_at(chunk, 0x10);
        let first_id = u64_at(chunk, 0x18);
        let last_id = u64_at(chunk, 0x20);
        let last_record_offset = u32_at(chunk, 0x2c);
        let free_space_offset = u32_at(chunk, 0x30);
        let empty = first_number == 1
            && first_id == u64::MAX
            && last_number == u64::MAX
            && last_id == u64::MAX;
        let free_space = free_space_offset as usize;
        if !(HEADER_SIZE..=CHUNK_SIZE).contains(&free_space) {
            return None;
        }
        if !empty {
            let last_record = last_record_offset as usize;
            if first_number > last_number
                || last_number - first_number >= MAX_RECORDS
                || first_id > last_id
                || last_id - first_id >= MAX_RECORDS
                || !(HEADER_SIZE..free_space).contains(&last_record)
            {
                return None;
            }
        }
        Some(Header {
            first_number,
            last_number,
            first_id,
            last_id,
            last_record_offset,
            free_space_offset,
            empty,
            header_checksum_valid: header_checksum(chunk) == u32_at(chunk, 0x7c),
            data_checksum_valid: data_checksum(chunk, free_space_offset) == u32_at(chunk, 0x34),
        })
    }
}

/// A record found in a chunk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub offset: usize,
    pub size: usize,
    pub id: u64,
    /// FILETIME of the record header.
    pub filetime: u64,
    /// SHA-256 (first 16 bytes) of the whole record: identical records are the same event.
    pub hash: [u8; 16],
    /// Found past the free space offset (old data the chunk no longer lists).
    pub slack: bool,
}

/// The record at `offset`, if its signature, size and the copy of its size at its end agree.
fn record_at(chunk: &[u8], offset: usize, slack: bool) -> Option<Record> {
    if offset + MIN_RECORD > chunk.len() || chunk[offset..offset + 4] != RECORD_MAGIC {
        return None;
    }
    let size = u32_at(chunk, offset + 4) as usize;
    if size < MIN_RECORD || !size.is_multiple_of(8) || offset + size > chunk.len() {
        return None;
    }
    if u32_at(chunk, offset + size - 4) as usize != size {
        return None;
    }
    let digest = Sha256::digest(&chunk[offset..offset + size]);
    let mut hash = [0_u8; 16];
    hash.copy_from_slice(&digest[..16]);
    Some(Record {
        offset,
        size,
        id: u64_at(chunk, offset + 8),
        filetime: u64_at(chunk, offset + 16),
        hash,
        slack,
    })
}

/// What a chunk holds.
#[derive(Clone, Debug, Default)]
pub struct Records {
    /// The allocated records, then the old ones found in the slack.
    pub records: Vec<Record>,
    /// Allocated records that could not be read (the walk went on at the next valid record).
    pub damaged: usize,
    /// Where the first damaged allocated record starts, if any.
    pub first_damage: Option<usize>,
}

impl Records {
    pub fn allocated(&self) -> impl Iterator<Item = &Record> {
        self.records.iter().filter(|r| !r.slack)
    }
}

/// The records of a chunk: the allocated ones, walked by size from the header up to the header's
/// last record offset (records are never identified by their ID, which can repeat), going on at
/// the next valid record after a damaged one; then every valid record in the slack.
pub fn records(chunk: &[u8], header: &Header) -> Records {
    let mut out = Records::default();
    let free_space = (header.free_space_offset as usize).min(CHUNK_SIZE);
    let mut offset = HEADER_SIZE;
    if !header.empty {
        let last = header.last_record_offset as usize;
        while offset <= last && offset + MIN_RECORD <= free_space {
            match record_at(&chunk[..free_space], offset, false) {
                Some(record) => {
                    offset += record.size;
                    out.records.push(record);
                }
                None => {
                    out.damaged += 1;
                    out.first_damage.get_or_insert(offset);
                    match next_record(&chunk[..free_space], offset + 8, false) {
                        Some(record) if record.offset <= last => {
                            offset = record.offset + record.size;
                            out.records.push(record);
                        }
                        _ => break,
                    }
                }
            }
        }
    }
    // Old records: in an empty page from its header on, otherwise after the free space.
    let mut offset = if header.empty {
        HEADER_SIZE
    } else {
        free_space
    };
    while let Some(record) = next_record(chunk, offset, true) {
        offset = record.offset + record.size;
        out.records.push(record);
    }
    out
}

/// The first valid record at or after `from` (records are 8-byte aligned).
fn next_record(chunk: &[u8], from: usize, slack: bool) -> Option<Record> {
    let mut at = from.next_multiple_of(8);
    while at + MIN_RECORD <= chunk.len() {
        let found = memchr::memmem::find(&chunk[at..], &RECORD_MAGIC)?;
        let candidate = at + found;
        if candidate.is_multiple_of(8)
            && let Some(record) = record_at(chunk, candidate, slack)
        {
            return Some(record);
        }
        at = candidate + 1;
    }
    None
}

/// The 64 KiB chunk whose compressed form starts 3 bytes before `signature` in `window` (which
/// begins at that LZNT1 block header), and how many of its bytes were present. NTFS compresses a
/// file in units of 16 clusters, each 4 KiB of a unit separately, so a chunk that starts a 4 KiB
/// block keeps its signature as 8 literal bytes after the block header and a zero flag byte. A unit
/// that did not compress is stored as it is. The next unit starts on the next cluster.
///
/// A unit of zeros has no clusters at all (it is sparse), so whatever follows on the disk may or
/// may not be the chunk's next unit. Both readings are tried: the decoded next unit and zeros. The
/// one the records checksum confirms is kept, otherwise the one whose records read better, and
/// zeros when they read the same, rather than invent slack from unrelated data.
///
/// `window_cluster_offset` is where `window` starts relative to the cluster grid of its volume.
pub fn decompress_chunk(
    window: &[u8],
    cluster_size: usize,
    window_cluster_offset: usize,
) -> Option<(Vec<u8>, usize)> {
    let (decoded, present, tail) = decode_units(window, cluster_size, window_cluster_offset)?;
    let Some(tail) = tail else {
        return Some((decoded, present));
    };
    let mut zeros = decoded.clone();
    zeros[tail..].fill(0);
    Some(if reads_better(&decoded, &zeros) {
        (decoded, present)
    } else {
        (zeros, CHUNK_SIZE)
    })
}

/// Whether the first chunk is the better reading of the two.
fn reads_better(a: &[u8], b: &[u8]) -> bool {
    let (Some(ha), Some(hb)) = (Header::parse(a), Header::parse(b)) else {
        return Header::parse(a).is_some();
    };
    if ha.data_checksum_valid != hb.data_checksum_valid {
        return ha.data_checksum_valid;
    }
    // Old records in a chunk's slack are older than its own records. Records with later IDs come
    // from somewhere else, such as the log's next chunk on the following clusters.
    let score = |chunk: &[u8], header: &Header| {
        let found = records(chunk, header);
        let allocated = found.allocated().count() as i64;
        let slack = found
            .records
            .iter()
            .filter(|r| r.slack && (header.empty || r.id < header.first_id))
            .count() as i64;
        (allocated - 2 * found.damaged as i64, slack)
    };
    score(a, &ha) > score(b, &hb)
}

/// Decode the chunk's units, and where the first unit after the chunk's own first unit starts in
/// the chunk (from there on the data may belong to another file, if that unit was sparse).
fn decode_units(
    window: &[u8],
    cluster_size: usize,
    window_cluster_offset: usize,
) -> Option<(Vec<u8>, usize, Option<usize>)> {
    if window.len() < 3 + MAGIC.len() || window[2] != 0 || !compressed_block_header(window) {
        return None;
    }
    let unit = cluster_size.checked_mul(16)?;
    if !(0x200..=0x1000).contains(&cluster_size)
        || !cluster_size.is_power_of_two()
        || unit < 2 * LZNT1_BLOCK
    {
        return None;
    }
    let mut data = Vec::with_capacity(CHUNK_SIZE);
    // Uncompressed bytes of the current unit read so far (the chunk starts 4 KiB into its unit).
    let mut in_unit = CHUNK_IN_UNIT;
    let mut at = 0;
    let mut tail = None;
    while data.len() < CHUNK_SIZE {
        if in_unit == unit {
            // The unit is complete: the next one starts on the next cluster.
            tail.get_or_insert(data.len());
            let absolute = window_cluster_offset + at;
            at = absolute.next_multiple_of(cluster_size) - window_cluster_offset;
            in_unit = 0;
            if at + 2 > window.len() {
                break;
            }
            // An LZNT1 unit starts with a compressed block, or a raw block of a full 4 KiB.
            let first = u16_at(window, at);
            if first & 0xf000 != 0xb000 && first != 0x3fff {
                // Stored as it is: the rest of the chunk follows.
                let take = (CHUNK_SIZE - data.len()).min(window.len() - at);
                data.extend_from_slice(&window[at..at + take]);
                break;
            }
        }
        if at + 2 > window.len() {
            break;
        }
        let header = u16_at(window, at);
        if header == 0 {
            // The unit's compressed data ended early: the rest of the unit was zeros.
            let zeros = (unit - in_unit).min(CHUNK_SIZE - data.len());
            data.resize(data.len() + zeros, 0);
            in_unit = unit;
            continue;
        }
        let end = at + usize::from(header & 0x0fff) + 3;
        if header & 0x7000 == 0x3000 && end > window.len() {
            break; // the input ended inside this block
        }
        let block = if header & 0x7000 != 0x3000 {
            None
        } else if header & 0x8000 != 0 {
            let mut out = Vec::with_capacity(LZNT1_BLOCK);
            lznt1::decompress(&window[at..end], &mut out)
                .ok()
                .map(|()| out)
        } else {
            Some(window[at + 2..end].to_vec())
        }
        .filter(|b| !b.is_empty() && b.len() <= LZNT1_BLOCK);
        let Some(block) = block else {
            // In the chunk's first unit this is not a compressed chunk; past it, the rest is
            // left to the zeros reading.
            tail?;
            break;
        };
        // A short block ends the unit's data; the rest of the unit reads as zeros.
        let short = block.len() < LZNT1_BLOCK;
        in_unit += block.len();
        data.extend_from_slice(&block);
        at = end;
        if short && in_unit < unit {
            let zeros = (unit - in_unit).min(CHUNK_SIZE.saturating_sub(data.len()));
            data.resize(data.len() + zeros, 0);
            in_unit = unit;
        }
    }
    data.truncate(CHUNK_SIZE);
    let present = data.len();
    data.resize(CHUNK_SIZE, 0);
    Some((data, present, tail))
}

/// Whether `data` starts with the header of a compressed LZNT1 block.
fn compressed_block_header(data: &[u8]) -> bool {
    data.len() >= 2 && u16_at(data, 0) & 0xf000 == 0xb000
}

/// Rewrite a chunk for a rebuilt log: its record numbers continue from `next_number`, a damaged
/// allocated record and everything after it become slack (so that viewers which stop at a bad
/// record still read the good ones), and both checksums are recomputed. An empty page is left as
/// it is, apart from its checksums. Returns the number after the chunk's last record and whether
/// the chunk was truncated at a damaged record.
pub fn renumber(chunk: &mut [u8], records: &Records, next_number: u64) -> (u64, bool) {
    let Some(header) = Header::parse(chunk) else {
        return (next_number, false);
    };
    let mut truncated = false;
    if !header.empty {
        let mut allocated: Vec<&Record> = records.allocated().collect();
        if let Some(damage) = records.first_damage {
            allocated.retain(|r| r.offset < damage);
            truncated = true;
        }
        if let Some(last) = allocated.last() {
            let count = allocated.len() as u64;
            chunk[0x08..0x10].copy_from_slice(&next_number.to_le_bytes());
            chunk[0x10..0x18].copy_from_slice(&(next_number + count - 1).to_le_bytes());
            chunk[0x18..0x20].copy_from_slice(&allocated[0].id.to_le_bytes());
            chunk[0x20..0x28].copy_from_slice(&last.id.to_le_bytes());
            chunk[0x2c..0x30].copy_from_slice(&(last.offset as u32).to_le_bytes());
            chunk[0x30..0x34].copy_from_slice(&((last.offset + last.size) as u32).to_le_bytes());
            let free_space = u32_at(chunk, 0x30);
            let data = data_checksum(chunk, free_space);
            chunk[0x34..0x38].copy_from_slice(&data.to_le_bytes());
            let checksum = header_checksum(chunk);
            chunk[0x7c..0x80].copy_from_slice(&checksum.to_le_bytes());
            return (next_number + count, truncated);
        }
    }
    let data = data_checksum(chunk, header.free_space_offset);
    chunk[0x34..0x38].copy_from_slice(&data.to_le_bytes());
    let checksum = header_checksum(chunk);
    chunk[0x7c..0x80].copy_from_slice(&checksum.to_le_bytes());
    (next_number, truncated)
}

/// The 4 KiB file header of a rebuilt log holding `chunks` chunks.
pub fn file_header(chunks: u16, next_record_id: u64) -> Vec<u8> {
    let mut header = vec![0_u8; 0x1000];
    header[..8].copy_from_slice(b"ElfFile\0");
    // first chunk number (0), last chunk number, next record ID
    let last = u64::from(chunks.saturating_sub(1));
    header[0x10..0x18].copy_from_slice(&last.to_le_bytes());
    header[0x18..0x20].copy_from_slice(&next_record_id.to_le_bytes());
    header[0x20..0x24].copy_from_slice(&0x80_u32.to_le_bytes());
    header[0x24..0x26].copy_from_slice(&1_u16.to_le_bytes()); // minor version
    header[0x26..0x28].copy_from_slice(&3_u16.to_le_bytes()); // major version
    header[0x28..0x2a].copy_from_slice(&0x1000_u16.to_le_bytes());
    header[0x2a..0x2c].copy_from_slice(&chunks.to_le_bytes());
    let checksum = crc32(&header[..0x78]).sum();
    header[0x7c..0x80].copy_from_slice(&checksum.to_le_bytes());
    header
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A chunk with `ids.len()` records of 32 bytes each.
    pub(crate) fn chunk_with(ids: &[u64]) -> Vec<u8> {
        let mut chunk = vec![0_u8; CHUNK_SIZE];
        chunk[..8].copy_from_slice(MAGIC);
        chunk[0x08..0x10].copy_from_slice(&1_u64.to_le_bytes());
        chunk[0x10..0x18].copy_from_slice(&(ids.len() as u64).to_le_bytes());
        chunk[0x18..0x20].copy_from_slice(&ids[0].to_le_bytes());
        chunk[0x20..0x28].copy_from_slice(&ids[ids.len() - 1].to_le_bytes());
        chunk[0x28..0x2c].copy_from_slice(&0x80_u32.to_le_bytes());
        let mut offset = HEADER_SIZE;
        for (i, id) in ids.iter().enumerate() {
            chunk[offset..offset + 4].copy_from_slice(&RECORD_MAGIC);
            chunk[offset + 4..offset + 8].copy_from_slice(&32_u32.to_le_bytes());
            chunk[offset + 8..offset + 16].copy_from_slice(&id.to_le_bytes());
            chunk[offset + 16..offset + 24].copy_from_slice(&(1000 + i as u64).to_le_bytes());
            chunk[offset + 28..offset + 32].copy_from_slice(&32_u32.to_le_bytes());
            if i + 1 < ids.len() {
                offset += 32;
            }
        }
        chunk[0x2c..0x30].copy_from_slice(&(offset as u32).to_le_bytes());
        chunk[0x30..0x34].copy_from_slice(&((offset + 32) as u32).to_le_bytes());
        let data = data_checksum(&chunk, (offset + 32) as u32);
        chunk[0x34..0x38].copy_from_slice(&data.to_le_bytes());
        let checksum = header_checksum(&chunk);
        chunk[0x7c..0x80].copy_from_slice(&checksum.to_le_bytes());
        chunk
    }

    #[test]
    fn plausible_headers_and_checksums() {
        let chunk = chunk_with(&[10, 11, 12]);
        let header = Header::parse(&chunk).unwrap();
        assert_eq!((header.first_id, header.last_id), (10, 12));
        assert!(header.header_checksum_valid && header.data_checksum_valid);

        let mut bad = chunk.clone();
        bad[0x28] = 0x81; // header size
        assert!(Header::parse(&bad).is_none());
        let mut bad = chunk.clone();
        bad[0x30..0x34].copy_from_slice(&0x20000_u32.to_le_bytes()); // free space past the chunk
        assert!(Header::parse(&bad).is_none());
        let mut damaged = chunk.clone();
        damaged[HEADER_SIZE + 40] ^= 1;
        let header = Header::parse(&damaged).unwrap();
        assert!(header.header_checksum_valid && !header.data_checksum_valid);
        assert!(Header::parse(&chunk[..CHUNK_SIZE - 1]).is_none());
    }

    #[test]
    fn records_ignore_repeated_ids_resume_after_damage_and_read_slack() {
        // The second record repeats the last record ID: the walk must not stop there.
        let mut chunk = chunk_with(&[7, 9, 8, 9]);
        let header = Header::parse(&chunk).unwrap();
        let found = records(&chunk, &header);
        assert_eq!(
            found.records.iter().map(|r| r.id).collect::<Vec<_>>(),
            [7, 9, 8, 9]
        );
        assert_eq!(found.damaged, 0);

        // Break the second record's signature: the walk goes on at the third.
        chunk[HEADER_SIZE + 32] = b'X';
        let header = Header::parse(&chunk).unwrap();
        let found = records(&chunk, &header);
        assert_eq!(
            found.records.iter().map(|r| r.id).collect::<Vec<_>>(),
            [7, 8, 9]
        );
        assert_eq!(
            (found.damaged, found.first_damage),
            (1, Some(HEADER_SIZE + 32))
        );

        // Shrink the allocated area to the first two records: the other two become slack.
        let mut chunk = chunk_with(&[1, 2, 3, 4]);
        chunk[0x2c..0x30].copy_from_slice(&((HEADER_SIZE + 32) as u32).to_le_bytes());
        chunk[0x30..0x34].copy_from_slice(&((HEADER_SIZE + 64) as u32).to_le_bytes());
        let header = Header::parse(&chunk).unwrap();
        let found = records(&chunk, &header);
        let slack: Vec<_> = found.records.iter().map(|r| (r.id, r.slack)).collect();
        assert_eq!(slack, [(1, false), (2, false), (3, true), (4, true)]);
    }

    #[test]
    fn renumbering_truncates_at_damage_and_fixes_checksums() {
        let mut chunk = chunk_with(&[5, 6, 7]);
        chunk[HEADER_SIZE + 64] = b'X'; // third record
        let header = Header::parse(&chunk).unwrap();
        let found = records(&chunk, &header);
        let (next, truncated) = renumber(&mut chunk, &found, 100);
        assert_eq!((next, truncated), (102, true));
        let header = Header::parse(&chunk).unwrap();
        assert_eq!((header.first_number, header.last_number), (100, 101));
        assert_eq!((header.first_id, header.last_id), (5, 6));
        assert!(header.header_checksum_valid && header.data_checksum_valid);
    }

    #[test]
    fn file_header_is_valid() {
        let header = file_header(3, 42);
        assert_eq!(&header[..8], b"ElfFile\0");
        assert_eq!(u16_at(&header, 0x2a), 3);
        assert_eq!(u64_at(&header, 0x10), 2);
        assert_eq!(crc32(&header[..0x78]).sum(), u32_at(&header, 0x7c));
    }

    /// Bytes no two 4 KiB blocks of which compress alike.
    fn noise(n: usize, seed: u64) -> Vec<u8> {
        let mut x = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
        (0..n)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                (x >> 24) as u8
            })
            .collect()
    }

    /// The compression units of a file, as NTFS stores them on clusters of `cluster` bytes: each
    /// unit of 16 clusters LZNT1-compressed when that saves a cluster, otherwise as it is; and the
    /// offset of each unit on the disk.
    fn ntfs_compressed(file: &[u8], cluster: usize) -> (Vec<u8>, Vec<usize>) {
        let unit = 16 * cluster;
        let mut disk = Vec::new();
        let mut starts = Vec::new();
        for data in file.chunks(unit) {
            starts.push(disk.len());
            if data.iter().all(|b| *b == 0) {
                continue; // a unit of zeros is sparse: it has no clusters
            }
            let mut compressed = Vec::new();
            lznt1::compress(data, &mut compressed);
            if compressed.len().next_multiple_of(cluster) < unit {
                disk.extend_from_slice(&compressed);
                disk.resize(disk.len().next_multiple_of(cluster), 0);
            } else {
                disk.extend_from_slice(data);
            }
        }
        (disk, starts)
    }

    #[test]
    fn decompresses_a_chunk_across_compression_units() {
        let chunk = chunk_with(&[1, 2, 3]);
        for (cluster, incompressible_tail) in [(0x1000, false), (0x1000, true), (0x800, false)] {
            // A log file: its 4 KiB header, the chunk, then a chunk of noise (stored as it is when
            // the unit holding the chunk's tail does not compress).
            let mut file = vec![0xaa_u8; 0x1000];
            file.extend_from_slice(&chunk);
            let tail = if incompressible_tail {
                noise(CHUNK_SIZE, 7)
            } else {
                chunk_with(&[4, 5])
            };
            file.extend_from_slice(&tail);
            let (disk, starts) = ntfs_compressed(&file, cluster);
            let signature = memchr::memmem::find(&disk, MAGIC).unwrap();
            assert!(signature > starts[0] && signature < starts[1]);
            let window = &disk[signature - 3..];
            let (data, present) = decompress_chunk(window, cluster, signature - 3).unwrap();
            assert_eq!(present, CHUNK_SIZE, "cluster {cluster:#x}");
            assert_eq!(data, chunk, "cluster {cluster:#x}");
        }

        // The chunk's last 4 KiB are zeros and end the file, so that unit is sparse; the next
        // clusters on the disk belong to something else and must not become the chunk's tail.
        let mut file = vec![0xaa_u8; 0x1000];
        file.extend_from_slice(&chunk);
        let (mut disk, _) = ntfs_compressed(&file, 0x1000);
        assert!(chunk[CHUNK_SIZE - LZNT1_BLOCK..].iter().all(|b| *b == 0));
        let mut other = Vec::new();
        lznt1::compress(&noise(0x1000, 9), &mut other);
        disk.extend_from_slice(&other);
        disk.extend_from_slice(&noise(0x3000, 8));
        let signature = memchr::memmem::find(&disk, MAGIC).unwrap();
        let (data, _) = decompress_chunk(&disk[signature - 3..], 0x1000, signature - 3).unwrap();
        assert_eq!(data, chunk);

        // A truncated source zero-fills the missing part.
        let mut file = vec![0xaa_u8; 0x1000];
        file.extend_from_slice(&chunk);
        let (disk, _) = ntfs_compressed(&file, 0x1000);
        let signature = memchr::memmem::find(&disk, MAGIC).unwrap();
        let short = &disk[signature - 3..signature + 200];
        let (data, present) = decompress_chunk(short, 0x1000, signature - 3).unwrap();
        assert!(present < CHUNK_SIZE && data.len() == CHUNK_SIZE);
        // Data that is not compressed.
        assert!(decompress_chunk(&chunk, 0x1000, 0).is_none());
    }
}
