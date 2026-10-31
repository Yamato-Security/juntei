# NTFS Transaction Log (`$LogFile`)

## What it is and why it matters

NTFS is the file system Windows usually uses for its system volume. To recover from a crash, it
records changes it intends to make to file-system metadata in a hidden volume file named
`$LogFile`. Each update can hold **redo** bytes (the new state) and **undo** bytes (the previous
state needed to roll a change back). This is separate from Windows Event Logs, registry `.LOG1`
files, and the [USN Journal](usn-journal.md).

For an investigator, the previous state is particularly useful. An undo record can preserve a
filename before a rename, a deleted directory entry, or timestamps from before a metadata update.
Some records also contain bytes NTFS wrote to the USN Journal, including records no longer in the
collected `$J`. Many transactions concern routine operating-system activity, however, and do not
describe an attack.

**Example:** a removed index entry mentions `report.txt`; a later added entry mentions
`report.txt.encrypted`. Matching the complete file reference, parent, transaction chain and other
artifacts can support a rename. Those two names alone do not identify ransomware, its operator,
or the exact time of encryption.

## Location and collection

```text
C:\$LogFile
D:\$LogFile
```

Every NTFS volume has its own log. Acquire it with a tool that reads NTFS metadata or exports it
from a forensic image or snapshot; an ordinary Windows file copy may not access it. Collect the
same volume's `$MFT` and `$Extend\$UsnJrnl:$J` as close in time as possible. Preserve the original
files and hashes. A later acquisition may contain a different stretch of activity.

`$LogFile` is a **circular log**: new records overwrite old ones. Retention depends on log size
and workload and can be only minutes or hours. The absence of a filename or operation is not
evidence that it never happened.

## What a record proves, and cautions

- An update records metadata NTFS logged for recovery. A single redo entry does not prove that
  an entire higher-level action completed or that a particular person initiated it. Correlate
  related records, transaction completion, the resulting file system, and independent evidence.
- **There is no general transaction timestamp.** Embedded timestamps describe file metadata,
  sometimes stale, copied, or attacker-modified. Do not put `CreatedOn` on an incident timeline
  as the time of a rename or timestamp change. USN records may supply a separate timestamp.
- LSNs (log sequence numbers) order records **within one volume's log**. They are not clock times
  and must not be compared across hosts or volumes. Records from older circular-buffer passes
  can survive alongside newer activity.
- A transaction-table slot is reused. Group with `TransactionFirstLsn`, `TransactionId`, client
  identity, and source rather than `TransactionId` alone. Chains can point outside retained data.
- File references contain an entry number **and a sequence number**. Reuse of an MFT entry can
  otherwise make unrelated files appear to be the same file.
- Paths supplied by `$MFT` describe the **collected MFT state**, not a reconstructed historical
  path. Juntei matches sequence numbers and leaves unknown components explicit. A parent could
  have been renamed after the transaction, and hard links can offer more than one current name.
- `TargetInferred` means an open-attribute mapping came from a different stretch of the log.
  Treat that mapping with additional care. Unresolved index entries may be decoded by their
  layout alone; validate their names with raw bytes and other sources.
- A torn page is skipped; readable surrounding records remain available. `Truncated` marks a
  record whose continuation is missing. Review parser diagnostics before treating output as
  complete. A zero-length input is an error; an unwritten all-`FF` log legitimately has no rows.
- The parser does **not replay** transactions to reconstruct a historical MFT. It does not
  recover arbitrary non-resident file contents or execute any collected data. DAX CRC-protected
  pages are not supported. Real-evidence validation primarily covers 4 KiB log pages.

## Parse with Juntei

```bash
# Single log, current paths from an adjacent $MFT when present
juntei parse-ntfs logfile -f '$LogFile' -o logfile.csv

# Explicit MFT from the SAME volume
juntei parse-ntfs logfile -f '$LogFile' -m '$MFT' -t jsonl -o logfile.jsonl

# Multiple collected logs; SourceFile distinguishes every input
juntei parse-ntfs logfile -d evidence -o collected-logs.csv

# Preserve the underlying payloads for low-level validation
juntei parse-ntfs logfile -f '$LogFile' --raw-data -t jsonl -o logfile-raw.jsonl

# Automatic per-computer output
juntei auto-parse -d evidence -o parsed
```

See the [command reference](../commands/filesystem.md#parse-ntfs-logfile) for all options. Directory
discovery accepts `$LogFile`, `LogFile`, their `.bin` forms, and names such as `C_$LogFile` or
`C_$LogFile.bin`, ignoring case. A single file passed with `-f` can have any name.

Automatic MFT pairing respects an export's volume prefix: `C_$LogFile` uses `C_$MFT`, and
`D_$LogFile.bin` first tries `D_$MFT.bin`, then `D_$MFT`. Prefixed logs never use a plain `$MFT`
as a fallback. Unprefixed `LogFile` also accepts `MFT` or `$MFT`. Use `-m` with `-f` to select a
matching MFT explicitly when your export has another naming scheme.

Output is one record per LSN, including update, checkpoint and unknown client records. JSON is a
stream of pretty-printed objects; JSONL has one object per line. CSV has the same field names;
arrays and structured fields are compact JSON inside a CSV cell. Unknown scalars are empty in
CSV and `null` in JSON. Empty arrays mean no items were decoded, not proof none existed.

One whole log and its parsed records are held in memory at a time. Resolving MFT paths adds
memory and work. `--raw-data` can greatly increase output size. In `auto-parse`, use `-T` to
limit concurrent parser processes on a memory-constrained machine.

If an adjacent MFT cannot be read, Juntei warns and exports the log without paths; `MftSource`
then remains null. This enrichment failure does not discard readable primary evidence.
Missing or ambiguous MFT matches also produce a warning and a `Logs without a matching MFT`
count in the summary; they do not count as parse failures.

## Output fields

### Record identity, integrity and provenance

| Field | Meaning and interpretation |
|---|---|
| `SourceFile`, `FileOffset` | Input path and the byte offset where this record's header was read; use them to validate the bytes. |
| `Lsn` | The record's log sequence number. Large values are integers, not timestamps. |
| `PreviousLsn`, `UndoNextLsn` | Links to previous and rollback records. Zero means no link; nonzero links may point outside retained data. |
| `TransactionId`, `TransactionFirstLsn` | Transaction-table slot and the earliest LSN linked to this transaction; the slot alone is not unique. |
| `RecordType` | `ClientRecord` (update), `ClientRestart` (checkpoint), or `Other`. |
| `ClientIndex`, `ClientSequence` | Identity of the log client; normally NTFS. These are not a user account or process ID. |
| `Flags`, `SpansPages` | Raw record flags and whether client data continues onto another page. Spanning pages is normal. |
| `Truncated` | True if the parser found a missing continuation for this LSN. It does not summarize all damage in the entire file. |
| `ClusterSize`, `ClusterSizeSource` | Bytes per cluster used for target offsets; source is `Caller`, `Checkpoint`, or `Inferred`. Supply the real value with `--cluster-size` if known. |
| `LogVersion` | Version from the latest readable restart page, typically `1.1` or `2.0`; null if no readable restart page survives. |
| `MftSource` | The MFT used for current path enrichment, if present. It must belong to the same volume. |

In `auto-parse` JSON/JSONL, `Computer` and `Artifact` are added as for other parsers. The task's
`.txt` log holds diagnostics; the manifests say whether the task succeeded, failed, or was skipped.
Salvageable damage can produce a successful task with a nonzero `Parse problems` count in its log.

### Operations and target

| Field | Meaning and interpretation |
|---|---|
| `RedoOperation`, `UndoOperation` | NTFS operation names, or hexadecimal for an unknown code. Redo describes the intended update; undo describes its inverse. |
| `TargetEntryNumber`, `TargetSequenceNumber` | For an operation on a file record: the affected MFT record number, with no sequence number inferred. Otherwise: the file reference owning the resolved open attribute. An inline directory-index update targets the directory, not the child described by its entry. |
| `TargetAttributeType`, `TargetAttributeName` | Numeric type and name of the open attribute used by the operation. For file-record operations this is normally `$MFT`'s `$DATA`. Common types: 16 (`$STANDARD_INFORMATION`), 48 (`$FILE_NAME`), 128 (`$DATA`), 144 (`$INDEX_ROOT`), 160 (`$INDEX_ALLOCATION`), 176 (`$BITMAP`). |
| `TargetPath` | Current MFT path when the target's full reference is known, or unknown components. Empty for file-record operations whose historical generation is unknown; a record number alone must not resolve to its current occupant. |
| `OpenAttributeEntryNumber`, `OpenAttributeSequenceNumber`, `OpenAttributePath` | File owning the open attribute through which the operation writes, with its current MFT path. For file-record operations these preserve the `$MFT` context separately from the affected record. |
| `TargetAttribute` | Offset in the log's open-attribute table; this is different from the attribute's type or MFT identifier. |
| `TargetInferred` | True if the target mapping comes from another stretch of the log. A null target remains unresolved even if this flag is false. |
| `TargetVcn`, `Lcns` | Virtual cluster in the target and physical cluster numbers; useful for low-level validation rather than timing. |
| `RecordOffset`, `AttributeOffset` | Offsets of the attribute/entry and the update within it. |
| `ClusterBlockOffset`, `TargetBlockSize` | Offset and target size in units of 512 bytes. |
| `OffsetInTarget` | Computed byte offset in the affected file record or non-resident stream. Depends on cluster and MFT record sizes. |
| `RedoLength`, `UndoLength` | Declared data lengths. A zeroing operation can declare a length without carrying that many payload bytes. |
| `RedoBytesAvailable`, `UndoBytesAvailable` | Payload bytes actually available for decoding. Compare with declared lengths and operation semantics. |

### Convenient file metadata columns

These columns show one preferred non-DOS filename from the redo side, falling back to undo.
If neither side carries a filename, timestamp-update fields prefer redo timestamps, then undo.
Directory-entry updates also expose their times, size and flags directly, preferring redo over
undo. Decoded `$DATA` attributes expose their size even when no filename is present. A decoded
file record's unnamed `$DATA` size takes priority over its possibly stale filename size.
They are a browsing aid; use the separate side fields for a complete before/after comparison.

| Field | Meaning and interpretation |
|---|---|
| `MetadataSide` | `Redo` or `Undo`, identifying the side supplying the convenient filename/timestamp columns. |
| `EntryNumber`, `SequenceNumber` | File described by decoded filename/file-record/attribute metadata or a partial `$STANDARD_INFORMATION` timestamp update, with the sequence when known. Empty for a directory-entry update that does not identify the child, and raw size/run-list updates. Use `TargetEntryNumber` for the affected record. |
| `FileName` | Preferred decoded name; all names remain in the side arrays, including DOS aliases and hard links. |
| `ParentEntryNumber`, `ParentSequenceNumber`, `ParentPath` | Parent reference and current MFT path; not a historical-path reconstruction. |
| `CreatedOn`, `ContentModifiedOn`, `RecordModifiedOn`, `LastAccessedOn` | UTC metadata times, to 100 ns. These are not transaction timestamps. Missing/invalid times remain empty/null. |
| `FileSize`, `FileAttributes` | Size and numeric Windows flags from the selected metadata; directory-entry values can be stale. `$DATA` attributes and decoded file records can supply a more direct size. |
| `AttributeType` | Numeric type of a decoded attribute record; 48 for a decoded directory-entry metadata update. Separate from the open attribute targeted by the operation. |

### Complete redo/undo metadata and payloads

| Field | Meaning and interpretation |
|---|---|
| `RedoFileNames`, `UndoFileNames` | Arrays of every decoded `$FILE_NAME` on that side, preserving added/new versus removed/old names. |
| `RedoTimes`, `UndoTimes` | `$STANDARD_INFORMATION` timestamps written by each side; partial updates leave unchanged timestamps absent rather than guessing them. |
| `RedoDecoded`, `UndoDecoded` | Structured file record, attribute, index entry or directory-entry information, identified by `Kind`. Null can mean an unsupported operation or insufficient bytes. |
| `JournalWriteOffset`, `JournalWriteLength` | Offset and byte count of redo data writing `$DATA:$J`. This identifies a possible source of otherwise lost USN records; it does not export a reconstructed journal. |
| `Checkpoint` | Checkpoint version, start LSN, cluster size, and the LSN/length of open-attribute, attribute-name, dirty-page and transaction-table dumps. |
| `OtherRecordType` | Raw client record type for an unrecognized or undecodable body. |
| `RedoData`, `UndoData`, `OtherData` | Hex payloads only with `--raw-data`; otherwise null/empty. Retain these for validation of unsupported or damaged structures. |

Each filename object includes the file and parent references (`Raw`, `EntryNumber`,
`SequenceNumber`), `Name`, `NameEscaped`, `Namespace`, current `ParentPath`, four timestamps,
`Size`, `AllocatedSize`, `Attributes` and `ReparseValue`. `NameEscaped` identifies invalid UTF-16
represented with escapes. A reparse value can be a reparse tag or extended-attribute size.

Timestamp objects preserve both `Utc` and the integer `Filetime`; a sentinel or invalid value can
have a null UTC representation while retaining its original integer. `RedoTimes` and `UndoTimes`
contain `Created`, `Modified`, `Changed`, and `Accessed`; missing members' values are null.

Decoded `FileRecord` objects include number, sequence, hard-link count, flags, base reference,
whether only the header survives, and the attributes present. Each decoded attribute preserves
type, name, resident state, flags, ID, size and allocated size. `$STANDARD_INFORMATION` adds the
four times, attributes, owner/security IDs, quota and USN; these IDs need separate resolution and
are not proof of the user who changed the file. Index entries include flags, key length and an
optional filename. Directory-entry updates preserve the four times, sizes, attributes and reparse
value even when the name itself was not logged.

## Investigation workflow

1. Review diagnostics for missing restart pages, torn pages and truncated records.
2. Search both `RedoFileNames` and `UndoFileNames` for a suspicious filename or extension.
3. Match entry **and sequence** numbers, then examine the transaction's LSN chain and operations.
4. Compare redo and undo timestamps for changes. Routine copying, extraction and restoration can
   also change timestamps; a difference is a lead, not an automatic timestomping verdict.
5. Correlate with the [USN Journal](usn-journal.md), [$MFT](mft.md) and [$I30](i30.md). Use event
   logs, [Prefetch](prefetch.md), and user-activity artifacts to establish execution or attribution.

For example, find all records carrying an encrypted filename on either side:

```bash
jq 'select(any(.RedoFileNames[]?, .UndoFileNames[]?;
  .Name | endswith(".encrypted"))) |
  {Lsn, TransactionId, TransactionFirstLsn, RedoOperation, UndoOperation,
   RedoFileNames, UndoFileNames}' logfile.jsonl
```

This is a filename search, not a complete ransomware detector. To inspect journal payloads,
export with `--raw-data` and select records with a non-null `JournalWriteOffset`; the `RedoData`
hex represents bytes written at that offset. Parsing and deduplicating those embedded USN records
is a separate step. Juntei does not yet merge them with `parse-ntfs usn-journal` output.

## Further Details

- [ys-logfile-parser](https://github.com/Yamato-Security/ys-logfile-parser): supported structures, validation corpus, and known limitations.
- [Sootmark ntfslog](https://github.com/Sootmark/ntfslog): upstream Rust implementation.
- [libfsntfs NTFS format documentation](https://github.com/libyal/libfsntfs/tree/main/documentation): log pages, fixups, and metadata structures.
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs): independent NTFS parsing research and implementation.
- [Joakim Schicht's LogFileParser](https://github.com/jschicht/LogFileParser): detailed transaction-log analysis.
- [Stroz Friedberg's ntfs-linker](https://github.com/strozfriedberg/ntfs-linker): correlating NTFS sources into file history.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum): real Windows NTFS samples.
- [Velocidex go-ntfs](https://github.com/Velocidex/go-ntfs): NTFS implementation and public test volume.
