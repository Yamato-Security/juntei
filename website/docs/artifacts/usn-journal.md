# USN Journal (`$UsnJrnl:$J`)

## What the USN Journal is

NTFS records file and directory changes in the Update Sequence Number (USN) Change Journal. A
record can describe creation, data extension/overwrite, attribute or security changes, rename,
close, and deletion. It stores a file reference, parent reference, name, reasons, attributes,
timestamp, and monotonically increasing USN.

The journal is excellent for reconstructing a burst of attacker file operations, especially when
the files were deleted. It records **that metadata/content changed**, not the changed bytes.

<span class="evidence-strength">Best conclusion:</span> NTFS recorded the stated change reasons for
the referenced file at the timestamp. Reasons often accumulate until a `CLOSE` record.

## Location and collection

```text
C:\$Extend\$UsnJrnl:$J
```

`$J` is a named stream of NTFS metadata. Collect it with a raw-aware forensic tool. Also collect the
same volume's `$MFT`, because USN records store parent file references rather than complete paths.

## Analyst cautions

- The journal is circular and size-limited. Old records are overwritten.
- `FILE_DELETE` says NTFS processed deletion; content may remain recoverable elsewhere.
- A reason such as `DATA_OVERWRITE` does not reveal the bytes, process, or user responsible.
- Several reasons can be combined in one record; `CLOSE` often marks the end of accumulated changes.
- Paths are reconstructed. Parent directories may have been renamed, deleted, or reused.
- File-reference sequence numbers matter. Entry number alone can connect to the wrong generation.
- Carving can recover valid records without their original journal context; report that distinction.

## Parse with Juntei

```text
Usage: juntei parse-usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $J files
  -f, --file <FILE>      Parse one $J, or any file with --carve
  -m, --mft <FILE>       Use the matching volume's $MFT for paths
      --carve            Search a disk image/unallocated file for records

Output:
  -o, --output <PATH>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-usn-journal -d evidence/ -o usn-jsonl -t jsonl
juntei parse-usn-journal -f unallocated.bin --carve -o carved-usn.csv
```

## CSV output fields

| Field | Interpretation |
|---|---|
| `Name` | File/directory name stored in this USN record. It is not a complete path. |
| `Extension` | Convenience extension parsed from `Name`; do not use it as a trusted content type. |
| `EntryNumber`, `SequenceNumber` | File's NTFS reference. Use both to identify the record generation. |
| `ParentEntryNumber`, `ParentSequenceNumber` | Parent directory's NTFS reference at the change. |
| `ParentPath` | Juntei's time-aware reconstructed parent path. Unknown markers mean ancestry could not be resolved. |
| `UpdateSequenceNumber` | USN ordering value. Useful for stable sorting and Amcache/MFT pivots. |
| `UpdateTimestamp` | Timestamp stored in V2/V3 records. Treat carved-context timestamps with the same validation as any record. |
| `UpdateReasons` | One or more named change flags, joined by `|`. Interpret combinations and `CLOSE` together. |
| `FileAttributes` | NTFS attributes recorded at the change; unknown bits remain hexadecimal. |
| `OffsetToData` | Byte offset in the input. Essential for validation and carved records. |
| `SourceFile` | Input journal/image; distinguishes hosts and volumes. |
| `SourceInfo` | Source flags describing how the change entered the journal. Usually contextual, not attribution. |
| `SecurityId` | NTFS security identifier index for the record, not a user SID. |
| `Version` | USN record version. Juntei writes V2/V3 to CSV; JSON retains V4 range-tracking extents. |

JSON/JSONL represents 128-bit V3 references correctly and preserves V4 range-tracking records and
their extents. Use it when low-level version-specific fields matter.

## Investigation strategy

1. Filter on time, name, extension, parent path, or file reference.
2. Sort by `UpdateSequenceNumber` as well as time to see exact journal order.
3. Pair `RENAME_OLD_NAME` with `RENAME_NEW_NAME` for moves/renames.
4. Use `$MFT` to validate record state, paths, sizes, and timestamps.
5. Correlate process attribution from event logs, EDR, Prefetch, services/tasks, or Defender.

## Further Details

- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: Change Journal reason flags](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2#members)
- [ys-usn-journal-parser](https://github.com/Yamato-Security/ys-usn-journal-parser)
- [Sootmark usn](https://github.com/Sootmark/usn)
