# Master File Table (`$MFT`)

## What the MFT is

NTFS represents every file and directory with a record in the Master File Table. Records contain
attributes such as names, parent references, timestamps, sizes, flags, security identifiers,
reparse information, and data-stream metadata. Small file content and alternate data streams can
be **resident**, meaning their bytes are stored inside the MFT record itself.

The MFT is foundational because it covers allocated and deleted records across the entire volume.
It can show that a file existed, where it was linked, whether it was deleted, and how its metadata
changed—even when ordinary directory browsing no longer shows it.

<span class="evidence-strength">Best conclusion:</span> an NTFS file record with the displayed
metadata existed at collection time or survived in an unallocated record. The MFT alone does not
prove execution or user intent.

## Location and collection

```text
C:\$MFT
```

`$MFT` is protected NTFS metadata. Collect it with a forensic acquisition or triage tool capable
of raw NTFS access; an ordinary user-mode copy normally cannot read it. Pair it with `$J`, `$LogFile`,
and volume metadata when possible.

## Analyst cautions

- A deleted record can be partially overwritten or its record number can be reused.
- `SN` distinguishes generations of the same entry number. Match entry **and** sequence.
- `$STANDARD_INFORMATION` (`0x10`) and `$FILE_NAME` (`0x30`) timestamps have different update
  behavior. Neither is immune to manipulation.
- `SI<FN`, `uSecZeros`, and `Copied` are leads, not automatic timestomping verdicts.
- Hard links produce multiple `$FILE_NAME` attributes and paths for one file record.
- NTFS can store several names (including DOS 8.3), alternate data streams, and extension records.
- Access-time updates may be disabled, delayed, or coalesced.
- A full path is reconstructed from parent records. `[Orphaned]`, `[Unknown]`, or `PathUnknown`
  signals missing/reused/damaged ancestry—not a literal directory name.
- A FILE record's **slack** (its bytes past the size in use) can keep names it held before: the
  index entries of a small directory whose index moved to an index allocation (most of them are
  still in use there), an earlier name, or a reused record's previous occupant. Recovered names
  have no `$STANDARD_INFORMATION`, and a name's entry header may have been written over. When a
  record's size in use or attributes don't say where its slack starts, nothing is recovered from it
  and its row's `ParseErrors` says `slack not read` and why.

## Parse with Juntei

```text
Usage: juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $MFT files
  -f, --file <FILE>      Parse one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse selected entry numbers (example: 0-15,30)

Output:
  -o, --output <PATH>                   Save records
  -t, --output-type <TYPE>              csv, ys-ntfs-parser-csv, json, jsonl
  -e, --extract-resident-streams <DIR>  Extract resident $DATA streams
      --reconstruct-filesystem <DIR>  Recreate a tree; recovered content gets .dontrun
      --reconstruction-i30 <FILE>      Supplement the tree with an I30 file (repeatable)

CSV Output:
  -a, --all-times      Include every $FILE_NAME time
  -s, --short-names    Include DOS 8.3-name rows
      --cutoff <DATE>  Keep recent SI-created or SI-modified records

Records:
  -x, --recover-slack  Recover file names from FILE record slack

Resident Data:
  -i, --include-resident-data      Put resident $DATA in the CSV
  -E, --resident-extensions <EXT>  Only these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  Only data up to this size

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail
  -L, --list-directory <ENTRY|PATH>  List a directory's contents
```

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff '2026-10-20T00:00:00Z' -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 32000-32100 | jq .
juntei parse-ntfs mft -f '$MFT' -x -o mft-with-slack.csv
juntei parse-ntfs mft -f '$MFT' -i -E ps1,bat,vbs,js -o scripts.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Users\alice\Downloads'
```

Never execute resident streams extracted with `-e`; Juntei gives them `.dontrun` names for a reason.
The resident data columns of `-i` are the same bytes: treat them as untrusted content.

`-D` prints one entry: its header, every attribute (with data runs) and the names in its slack as
`-t json -x` writes them, then the record's bytes. `-L` lists every name in a directory, in use and
deleted, with its entry and sequence number, type, size and modified time. A directory is given by
entry (`30`, `0x1E`, `30-1`) or by path (`\Windows\System32`, `C:/Windows`; case-insensitive).

To reconstruct browsable file paths, use the example below. Non-resident content becomes empty placeholders. See [Filesystem reconstruction](../commands/auto-parse.md#filesystem-reconstruction).

```bash
juntei parse-ntfs mft -f '$MFT' --reconstruct-filesystem reconstructed
```

## CSV output fields

| Field | Interpretation |
|---|---|
| `EntryNum` (Entry Number) | MFT record number. Combine with `SN`; record numbers are reused. |
| `SN` (Sequence Number) | Generation counter used by NTFS references. A mismatch can reveal stale/deleted relationships. |
| `InUse` | Allocation flag at collection. `false` means the record is free/deleted, not that all bytes are intact. |
| `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number) | Parent directory reference from `$FILE_NAME`. Validate both parts. |
| `ParentPath` | Reconstructed parent path. Unknown/orphan markers expose unresolved ancestry. |
| `File` (Filename) | Name from the current row's `$FILE_NAME` or stream. One record can have several rows. |
| `Ext` (Extension) | Convenience extension parsed from the name; not a trusted file-type determination. |
| `Size` (File Size) | Logical size associated with the row/stream. Sparse/compressed allocation can differ. |
| `RefCnt` (Reference Count) | NTFS hard-link reference count; compare with the number of surviving names cautiously. |
| `ReparseTarget` | Decoded target for supported reparse points such as junctions/symlinks. Paths may redirect. |
| `Dir` (Directory) | Record has the directory flag. |
| `HasADS` (Has Alternate Data Streams) | Record contains named alternate data streams. This does not make it malicious. |
| `IsADS` (Is Alternate Data Stream) | This row represents a named alternate data stream rather than the unnamed file data. |
| `SI<FN` (Standard Information Before File Name) | SI creation precedes a different FN creation time. Investigate copying/timestomping and legitimate causes. |
| `uSecZeros` (Microsecond Zeros) | SI created/modified time lands exactly on a millisecond boundary. Weak timestomping heuristic only. |
| `Copied` | Modified time precedes created time. Often consistent with file copy; not proof of attacker action. |
| `SI-Flags` (Standard Information Flags) | `$STANDARD_INFORMATION` file attributes/flags. Hidden/system flags can be useful pivots. |
| `Type` (Name Type) | `$FILE_NAME` namespace (Win32, DOS, POSIX, or combined). Use to avoid double-counting names. |
| `DataModified (SI)`, `Accessed (SI)`, `MetadataChanged (SI)`, `Created (SI)` | SI times. Common timeline fields, but user-mode tools can change several of them. |
| `DataModified (FN)`, `Accessed (FN)`, `MetadataChanged (FN)`, `Created (FN)` | FN times. Updated differently and generally harder for ordinary APIs to alter; still not immutable. |
| `USN` (Update Sequence Number) | Last USN associated with the record's SI attribute. Pivot into `$J` if that USN survives. |
| `LogfileSN` (Logfile Sequence Number) | NTFS `$LogFile` sequence number from the FILE record header; useful in low-level transaction analysis. |
| `SID` (Security ID) | Index into NTFS `$Secure`; not a Windows SID by itself. Join it with the `SecurityID` of [`parse-ntfs secure-sds`](secure-sds.md) for the file's owner and ACLs. |
| `ObjectID-FileDroid` | Object ID / distributed-link-tracking identifier where present; correlate with LNK/Jump List droids. |
| `LoggedUtilStream` | Decoded `$LOGGED_UTILITY_STREAM` context where supported. Review unfamiliar values. |
| `ZoneID-Contents` | Resident `Zone.Identifier` content, such as ZoneId and download URLs. It may be absent, removed, or non-resident. |
| `SourceFile` | The input `$MFT`, essential when combining hosts or volumes. |
| `ResidentDataBase64`, `ResidentDataHex`, `ResidentDataASCII` | With `-i`: the content of the row's resident `$DATA` (the file's, or the alternate data stream's), as Base64, as bytes (`41-42`), and as text when it is printable ASCII. Empty when the data is non-resident or filtered out by `-E`/`-M`. |
| `ParseErrors` | Damage or unsupported relationships affecting this row. Never discard this column during filtering. |
| `FromSlack` | With `-x`: `True` for a name recovered from the slack of a FILE record. Its `EntryNum` and `SN` are the file the name pointed to (empty when its entry header was written over), its recovered timestamps are in the `(FN)` columns, and `InUse` is `False`. |
| `SlackRecord`, `SlackOffset` | With `-x`: the FILE record whose slack held the name, by its place in the `$MFT` rather than the record number written inside it (an extension record's own, even though its names are listed with its base record), and the name's byte offset in the `$MFT`. A slack row's `ParseErrors` are that record's problems, such as `fixup values do not match`: the recovered bytes may be affected. |
| `CopyOfEntryInUse` | With `-x`: a file in use still has this name in the same directory (and is the same file, when the reference survived). Names that no file in use has are the interesting ones: deleted, renamed, or a reused record's previous occupant. |

Timestamp columns follow MACB order (Modified, Accessed, Changed, Birth/creation):
`DataModified` SI/FN, `Accessed` SI/FN, `MetadataChanged` SI/FN, then `Created` SI/FN.

`(SI)` means `$STANDARD_INFORMATION`; `(FN)` means `$FILE_NAME`. These names describe the
usual source for a complete record. For a normal filename row with no SI attribute, the
existing fallback places FN creation in `Created (FN)` and FN modified/record-change/access
values in `DataModified (SI)`, `MetadataChanged (SI)` and `Accessed (SI)` respectively.
Recovered slack rows place all four times in the `(FN)` columns and leave the SI columns empty.
The normal-row fallback values are **not** independent SI evidence. Use the original JSON attributes
and slack provenance when determining the timestamp source.

`-t ys-ntfs-parser-csv` (formerly `ys-mft-parser-csv`, still accepted) uses the maintained parser's
entry-oriented schema. These readable column names apply to the default `-t csv` output,
including `auto-parse`'s `MFT.csv`. JSON/JSONL retains every
attribute (resident `$DATA` included, in hexadecimal) and `FullPath`; use it when the flat
investigative CSV omits low-level structure. With `-x`, each JSON entry also has
`SlackFileNames`.

## Investigation strategy

1. Filter a time window, but keep deleted records and both timestamp sets.
2. Search full paths, suspicious double extensions, ADS, staging directories, and known hashes/names.
   With `-x`, include the slack names that no file in use has (`CopyOfEntryInUse` `False`); with
   `-i -E ps1,bat,vbs,js`, read small scripts straight from the CSV.
3. Pivot entry/sequence and `USN` into the USN Journal.
4. Compare Prefetch/LNK/Jump List MFT references with the target record.
5. Correlate deletion with Defender quarantine or USN `FILE_DELETE`/rename activity.

## Further Details

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [ys-ntfs-parser's mft crate](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/mft)
- [Omer Ben-Amram's mft parser](https://github.com/omerbenamram/mft)
- [libfsntfs format documentation](https://github.com/libyal/libfsntfs/tree/main/documentation)
