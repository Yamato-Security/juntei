# Event Log Carving { #carve-evtx }

Recover event-log chunks that remain after a log was cleared or deleted: from a disk image, unallocated space, a pagefile or a memory image. Each 64 KiB EVTX chunk carries its own string and template tables, so its records can be read without the rest of the log. Juntei writes the recovered chunks as ordinary `.evtx` files, one per channel, for Hayabusa or Event Viewer.

```bash
juntei carve-evtx -f host.E01 -o carved/
juntei carve-evtx -f host.vhdx --free-space-only -o carved-free/
juntei carve-evtx -f pagefile.sys --live-logs evidence/C/Windows/System32/winevt/Logs/ -o carved-pagefile/
juntei carve-evtx -f memory.raw -o carved-memory/ -t jsonl
```

Without `-o`, only the summary is shown.

## Finding chunks { #finding-chunks }

- The chunk signature is searched at every byte offset, and each header must be plausible (header size, record numbers, offsets). Checksums are reported, not required: a partly overwritten chunk still holds readable records.
- Chunks of NTFS-compressed logs are decompressed (LZNT1, 16-cluster units). On a disk image the volume's cluster size is used; for other files, `--cluster-size <BYTES>` (default 4096).
- Records are walked by size up to the header's last record offset, never stopped by a repeated record ID. After a damaged record the walk continues at the next valid record. Old records in the chunk's slack are listed as well.

## Disk images and NTFS volumes { #disk-images }

With an E01 or VHDX image, or a raw disk or volume with NTFS, every chunk is placed by its cluster:

| Location | Meaning |
|---|---|
| `FreeSpace` | A cluster that no file uses. |
| `DeletedLog` | A free cluster that a deleted log's `$MFT` record still lists, or a log rebuilt from them. |
| `LiveLog` | A cluster of an event log that is in use. Left out unless `--keep-live`. |
| `AllocatedFile` | A cluster of another file, such as `pagefile.sys` or a shadow copy's store file. |
| `AllocatedUnknown` | An allocated cluster that no `$MFT` record lists. |
| `OutsideVolumes` | Outside every NTFS volume of the disk. |
| `Unclassified` | The input has no NTFS volume (a pagefile or a memory image). |

Deleted logs whose `$MFT` record lists only free clusters are rebuilt whole, compressed ones included, and saved under `Recovered/`. `--free-space-only` searches only free clusters: it is faster, but skips the pagefile, shadow copies and other files.

## Deduplication { #deduplication }

Each chunk is written once. A chunk is left out when it belongs to a live log, when it is identical to another chunk, or when all its records are in the live logs or in a chunk already written. Chunks with more records are considered first, so an older, shorter copy of a chunk adds nothing. For an input without NTFS, give the live logs with `--live-logs <DIR>`.

## Results { #results }

- `Carved/<CHANNEL>.evtx`: one log per channel, named as Windows names logs (`Microsoft-Windows-Sysmon%4Operational.evtx`). Chunk header record numbers are renumbered and checksums recomputed. A damaged record and everything after it become slack. Each chunk is otherwise kept whole, including its slack, so record recovery such as Hayabusa's `-x` still works on it.
- `Recovered/Volume<N>/...`: deleted logs rebuilt from their `$MFT` records.
- `chunks.csv` / `chunks.jsonl`: every chunk found: where it was, what became of it (`Status`), its channel, record IDs, times and checksum state.
- `records.csv` / `records.jsonl`: every record of the chunks written. `OnlyInCarvedData` marks records that the live logs do not hold.

```bash
hayabusa csv-timeline -d carved/Carved -a -o carved-timeline.csv
```

Carving finds contiguous chunks only. A chunk split across non-contiguous clusters, or across non-contiguous physical pages in memory, is incomplete, and its later records may be damaged or missing. Decompress `hiberfil.sys` first. The exit code is 0 when the search completed and 1 on an error.

## Further Details { #further-details }

- [Detailed usage and field reference (English)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.md)
- [Detailed usage and field reference (Japanese)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.ja.md)
- [EVTX format (libevtx documentation)](https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc)
