# Event Log Carving

`carve-evtx` recovers Windows event-log (EVTX) chunks that remain after a log was cleared or
deleted, or that Windows left in the pagefile or in memory, and writes them as ordinary `.evtx`
files that Hayabusa and Event Viewer open.

An EVTX file is a 4 KiB file header followed by 64 KiB chunks. Each chunk carries its own string
and template tables, so its records can be read without the rest of the log. When a log is
cleared or deleted, its chunks usually stay on disk until the clusters are reused. Windows also
keeps chunks of open logs in memory, and memory pages end up in `pagefile.sys`.

## Usage

```bash
juntei carve-evtx -f host.E01 -o carved/
juntei carve-evtx -f host.vhdx --free-space-only -o carved-free/
juntei carve-evtx -f pagefile.sys --live-logs evidence/C/Windows/System32/winevt/Logs/ -o carved-pagefile/
juntei carve-evtx -f memory.raw -o carved-memory/ -t jsonl
juntei carve-evtx -f host.E01
```

| Option | Meaning |
|---|---|
| `-f, --file <FILE>` | The input: an E01 or VHDX disk image, a raw disk or volume, `pagefile.sys`, a raw memory image, or any other file. |
| `-l, --live-logs <DIR>` | Live `.evtx` logs (searched recursively) whose records are not written again. Use it for an input without NTFS, such as a pagefile next to a collection's `winevt\Logs`. On a disk image the live logs are found on its volumes. |
| `--no-file-system` | Read the input as plain bytes, even if it holds NTFS volumes. |
| `--cluster-size <BYTES>` | Cluster size for NTFS-compressed chunks in an input without an NTFS volume: 512, 1024, 2048 or 4096 (default). |
| `--free-space-only` | Search only the free clusters of NTFS volumes. Faster, but skips the pagefile, shadow copy store files and every other file. |
| `--keep-live` | Also write the chunks of live logs, and chunks whose records the live logs still hold. |
| `-o, --output <DIR>` | A new or empty directory for the rebuilt logs, recovered files and reports. Without it, only the summary is shown. |
| `-t, --output-type <TYPE>` | `csv` (default) or `jsonl`, for the chunk and record reports. |

The exit code is 0 when the search completed (also when no chunk was found) and 1 on an error.

## How chunks are found

1. **Signature search.** The input is read in 4 MiB blocks and searched for `ElfChnk\0` at
   every byte offset, because chunks in free space, in the pagefile and in memory are not
   aligned.
2. **Header check.** A chunk is accepted when its header is plausible: header size 128, record
   numbers and record IDs in order and fewer than fit in a chunk, and the last record and free
   space offsets inside the chunk. An empty page (a chunk written with no records) is accepted
   too, because old records may remain in it.
3. **Checksums.** The header and records checksums are computed and reported in the chunk report,
   but not required: a partly overwritten chunk still holds readable records.
4. **NTFS compression.** When the bytes at a signature are not a plausible chunk, Juntei tries
   NTFS compression. NTFS compresses a file in units of 16 clusters, each 4 KiB of a unit
   separately (LZNT1), so a chunk that starts a 4 KiB block keeps its signature as literal bytes.
   The unit is decompressed, and the next unit is read from the next cluster. A unit that did not
   compress is stored as it is, and a unit of zeros has no clusters at all: for the chunk's tail,
   both the decoded data and zeros are tried, and the reading that the records checksum confirms,
   or whose records read better, is kept. On a disk image, the cluster size and cluster grid of
   the volume are used. For other files, `--cluster-size` gives the cluster size and the grid
   starts at byte 0.
5. **Records.** Records are walked by size from the end of the header up to the header's last
   record offset. A record is valid when its signature, size and the copy of its size at its end
   agree. The walk is never stopped by a record ID, because real chunks can hold an earlier record
   with the same ID as the last one. After a damaged record, the walk continues at the next
   valid record. Old records in the slack after the free space offset (and every record of an
   empty page) are listed as slack records.
6. **Details.** The channel, provider, computer and event ID of each allocated record are read
   with the evtx crate. A chunk's channel is the most common one among its records. For an empty
   page, which has no allocated records, the channel comes from the name of the log it is in, when
   that is known.

## Disk images and NTFS volumes

For an E01 or VHDX image, or a raw disk or volume with an NTFS file system (an MBR or GPT disk,
or a bare volume), Juntei reads each NTFS volume's `$Boot`, `$Bitmap` and `$MFT`, and places
every chunk by its cluster:

| Location | Meaning |
|---|---|
| `FreeSpace` | A cluster that no file uses. |
| `DeletedLog` | A free cluster that a deleted log's `$MFT` record still lists, or a log rebuilt from such clusters. |
| `LiveLog` | A cluster of an event log that is in use. Left out unless `--keep-live`. |
| `AllocatedFile` | A cluster of another file, such as `pagefile.sys`, a shadow copy's store file in `System Volume Information`, or a copied log. `Owner` gives its path (and stream). |
| `AllocatedUnknown` | An allocated cluster that no `$MFT` record lists (file system metadata or damage). |
| `OutsideVolumes` | Outside every NTFS volume of the disk. |
| `Unclassified` | The input has no NTFS volume, such as a pagefile or a memory image. |

**Deleted logs.** A deleted `.evtx` whose `$MFT` record is still readable and lists only free
clusters is rebuilt whole before the search, NTFS-compressed logs included, and saved under
`Recovered/Volume<N>/<path>`. Its chunks are then handled like the others, and the same chunks
found by the search are duplicates. A deleted log is not rebuilt when some of its clusters were
reused, when its data runs continue in other `$MFT` records, or when it is larger than 4 GiB; the
search still finds its remaining chunks.

**Live logs.** The live logs are read through the file system (decompressed when they are
compressed). Their chunks and allocated records are the reference for deduplication. Old records
in a live chunk's slack are not part of it, because the log no longer shows them.

## Deduplication

Each chunk is written once, and only when it adds a record:

| Status | Meaning |
|---|---|
| `Written` | Written to a rebuilt log. |
| `SkippedLiveLog` | In a live log's clusters. |
| `DuplicateOfLiveLog` | Identical to a chunk of a live log. |
| `DuplicateChunk` | Identical to another chunk found. |
| `RecordsInLiveLogs` | Every record (allocated and slack) is in the live logs. |
| `RecordsInOtherChunks` | Every record is in the live logs or in a chunk already written. |
| `NoRecords` | No valid record at all. |

Records are compared by the SHA-256 of their bytes. Chunks with more records are considered
first, so an older, shorter copy of a chunk adds nothing. A chunk with at least one new record is
written whole: its other records may also appear in the live logs, and the record report marks
them.

## Results

```text
carved/
├── Carved/
│   ├── Security.evtx
│   ├── Microsoft-Windows-Sysmon%4Operational.evtx
│   └── ...
├── Recovered/
│   └── Volume1/C/Windows/System32/winevt/Logs/...
├── chunks.csv
└── records.csv
```

**Rebuilt logs.** `Carved/` holds one log per channel, named as Windows names logs (`/` becomes
`%4`), in time order. Chunks without a known channel go to `Unknown.evtx`. A log holds at most
65,535 chunks; more go to `<name>-2.evtx` and so on. Each log gets a new file header. In each
chunk, the record numbers in the header are renumbered to continue from the previous chunk, and
both checksums are recomputed so that Event Viewer opens the file. When an allocated record is
damaged, it and every record after it become slack, so that readers which stop at a bad record
still read the good ones. Each chunk is otherwise kept byte for byte, including its slack, so
record recovery such as Hayabusa's `-x` still works on it.

```bash
hayabusa csv-timeline -d carved/Carved -a -o carved-timeline.csv
```

**Chunk report** (`chunks.csv` / `chunks.jsonl`), one row per chunk found:

| Field | Meaning |
|---|---|
| `Source` | The input file name, or the rebuilt log under `Recovered/` the chunk is in. |
| `Offset` | Where the chunk starts in the source (for a compressed chunk, its LZNT1 block). |
| `Location`, `Volume`, `Owner` | Where it was (see above), the volume number, and the file it belongs to. |
| `Status`, `OutputFile` | What became of it (see above), and the rebuilt log it was written to. |
| `Channel`, `Computer` | The most common channel and computer of its records. |
| `FirstRecordId`, `LastRecordId`, `FirstTime`, `LastTime` | The range of its allocated records (UTC). |
| `Records`, `SlackRecords`, `DamagedRecords` | Allocated records, old records in the slack, and allocated records that could not be read. |
| `NewRecords` | Records that the live logs do not hold. |
| `EmptyPage` | Written as an empty page. |
| `Compressed`, `Truncated` | Decompressed from NTFS compression; cut short by the end of the input (the rest is zeros). |
| `HeaderChecksumValid`, `DataChecksumValid` | Whether the stored checksums matched. |
| `Repaired` | A damaged record and the records after it were made slack in the rebuilt log. |

**Record report** (`records.csv` / `records.jsonl`), one row per record of the chunks written:

| Field | Meaning |
|---|---|
| `OutputFile`, `Source`, `ChunkOffset`, `Location` | The rebuilt log and where the chunk was found. |
| `RecordOffset`, `Allocation` | The record's offset in the chunk; `Allocated` or `Slack`. |
| `EventRecordId`, `Timestamp` | From the record header (UTC). |
| `Channel`, `EventId`, `Provider`, `Computer` | From the record (empty for slack records). |
| `OnlyInCarvedData` | The live logs do not hold this record. |

## Limitations

- Carving finds contiguous chunks only. A chunk split across non-contiguous clusters in free
  space, or across non-contiguous physical pages in memory, is incomplete: its later records may
  be damaged or missing, and many such chunks have checksum mismatches.
- `hiberfil.sys` and crash dumps must be decompressed first.
- Slack records are listed in the record report but have no channel or event details. Event log
  readers show them only if they recover records from chunk slack.
