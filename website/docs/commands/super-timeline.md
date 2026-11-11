# Super Timeline

`super-timeline` turns parsed artifacts into events with a common schema. Output is **unsorted by
default**: it follows the discovered input-file order and each parser's projection order, independently
of worker completion order. Add `--sort` for chronological ordering and duplicate consolidation.
A Prefetch record can contain several execution times; an MFT record can contain eight or more
file timestamps. Each becomes a separately labeled event. This makes it easier to compare evidence
from different sources without losing what each timestamp actually means.

Start with the results of [`auto-parse`](auto-parse.md), rather than the original evidence directory:

```bash
juntei auto-parse -d evidence/ -o parsed/
juntei super-timeline -d parsed/ -o SuperTimeline.csv
```

Keep the parsed files, manifests and original evidence. The timeline is a projection for analysis;
it does not replace those records or create a native Plaso storage file.

## Usage

```bash
# Native CSV (default)
juntei super-timeline -d parsed/ -o SuperTimeline.csv

# Chronological CSV with duplicate consolidation
juntei super-timeline -d parsed/ -o SortedTimeline.csv --sort

# Native JSON Lines
juntei super-timeline -d parsed/ -o SuperTimeline.jsonl -t jsonl

# Combine native records and an existing Hayabusa JSONL timeline
juntei super-timeline -f NativeRecords.jsonl --hayabusa Hayabusa.jsonl \
  -o SuperTimeline.jsonl -t jsonl

# Timesketch import profile; also available with CSV
juntei super-timeline -d parsed/ -o Timesketch.jsonl -t jsonl --profile timesketch

# One standalone CSV needs an artifact identifier and, if unavailable, a computer name
juntei super-timeline -f MFT.csv --input-artifact mft --computer HOST01 \
  -o Host01Timeline.csv

# Reduce the time range, memory used by sorting, or number of workers
juntei super-timeline -d parsed/ -o IncidentTimeline.jsonl -t jsonl \
  --timeline-start "2026-10-20T06:00:00Z" --timeline-end "2026-10-20T14:00:00Z" \
  --sort --memory-limit 64 -T 4 --temp-dir scratch/
```

`--hayabusa FILE` can be repeated or used on its own. It imports existing event/alert timestamps
and rule context without running Hayabusa. Informational events whose `Details` is a string are
supported. `--artifact mft,prefetch` restricts the projection to selected artifact identifiers.
Time limits are inclusive and require an explicit UTC offset or `Z`.

### Choosing an input

| Input | How it is identified |
|---|---|
| `-d DIR` | `manifest.json` supplies succeeded primary tasks, artifact IDs, computers and exact output files. Secondary `auto-parse -j` tasks are excluded to prevent importing both copies. |
| `-f FILE` | Native JSON/JSONL uses embedded `Artifact` and `Comp` fields. Untagged exports need `--input-artifact`; standalone CSV always requires it. |
| `--computer NAME` | Fallback when the input record has no computer name; otherwise an unknown computer is explicitly `Unknown`. |
| `--hayabusa FILE` | JSONL with Hayabusa timestamps, rule metadata, computer and, when present, `EvtxFile`. |

Current manifests support relocation of the parsed directory: recorded absolute output paths are
resolved under the new directory. Inputs must remain inside it, including symbolic-link targets.
Legacy per-artifact Amcache, SRUM and Jump Lists directories are supported; category directories
need explicit `OutputFiles`.

Current manifests with output-relative paths and flattened multi-table exports are supported. The command recognizes every artifact scheduled by automatic parsing, including the newer browser, WebCache, Outlook and UAL tables.

When a manifest identifies a VHDX or E01 image, source references include the image and internal path, and the image name is retained in event context. The temporary staging workspace is not needed to read the parsed results.

A raw source is taken from the record or an unambiguous manifest command. With `-d`, raw-source
references are relative to the evidence directory, and `Details.ParsedFile` is relative to the parsed
directory. SRUM CSV tables are assigned to a database only when the host has one uniquely
identifiable collected SRUDB.dat. Otherwise `SourceFile` points to the parsed file and
`Details.RawSourceAvailable` is `false`.

A standalone input is referenced by its filename. Inputs with the same filename use distinct
`./Inputs/<ID>/<filename>` references. Absolute raw-source paths outside known evidence roots use
`./Sources/<ID>/<filename>` identifiers. These opaque references retain source distinctions without
exporting the analyst's absolute directory names; consult the native record using `ParsedFile` and
`InputRecord` for the original source. Paths recorded in the evidence, such as Windows executable
paths, remain unchanged.

## Common event fields

| Field | Meaning |
|---|---|
| `Timestamp` | UTC RFC 3339, written with nine fractional digits for consistent ordering. Padding does not add precision. |
| `Comp` | Computer associated with the input record. |
| `Artifact` | Stable artifact identifier such as `mft`, `prefetch`, `srum` or `hayabusa`. |
| `TimestampType` | Description of the timestamp's meaning, such as `MFT SI Data modified`. |
| `User` | Recorded account or SID when the adapter has one; may be empty. |
| `Path` | Affected file, registry key or other artifact-specific object. |
| `Message` | Short description for a timeline viewer. |
| `SourceFile` | Evidence-relative raw-source reference, or a relative source identifier when its root is unknown; otherwise the parsed-file fallback described above. |
| `SourceField` | Exact parsed field or nested location used for the timestamp. |
| `RecordID` | Artifact-specific record identity. It is not necessarily an EVTX Record ID. |
| `Details` | Structured context including the original timestamp, precision, time basis, time role, parsed file, input record/location and available recovery/rule metadata. |

In CSV, `Details` is a quoted JSON cell. In JSONL it is an object. The Timesketch profile changes
`Timestamp`, `TimestampType` and `Message` to `datetime`, `timestamp_desc` and `message`, respectively,
while keeping the other provenance fields. Those are the
[three required Timesketch import fields](https://timesketch.org/guides/user/import-from-json-csv/).

`Details` deliberately contains selected context, rather than full binary data or every native
record field. Long strings, arrays and objects are bounded and marked `ProjectionOmitted`.
Look up `Details.ParsedFile`, `InputRecord` and `Location` to inspect the complete native record.
For CSV, the record number counts CSV records after the header, not physical lines inside quoted
cells. For JSONL it is the line number; for JSON it is the top-level document number plus a nested
location.

## What the times mean

A timestamp is evidence about a recorded object. It is not, by itself, proof of execution or an
attack. The adapters intentionally keep these distinctions:

| Artifacts | Timeline interpretation |
|---|---|
| MFT | Separate SI and FN modification, access, metadata-change and creation times; named FN attributes remain distinct. With `--sort`, repeated SI fields on filename rows are consolidated. ADS rows' shared base-file times are excluded in both modes. Recovered FILE-record slack names keep separate recovered-FN events. |
| I30 | Stored FN/index metadata, with slack/recovery context when available. These are not timestamps of the directory-index write. |
| USN Journal | Journal record timestamp and update reasons. A record can describe several changes; it does not necessarily record their precise start time. |
| LogFile | Stored redo/undo timestamp snapshots and filename metadata. Redo and undo remain separate. LSNs and file offsets are identifiers, never converted to dates. |
| Prefetch | Recorded execution times. With `--sort`, these are consolidated with the matching Prefetch timeline export. Referenced volume-creation metadata is labeled separately. |
| Browser SQLite databases | Recorded visits, recovered page last-visit metadata, download start/end, cookie creation/access/expiry and form-history first/last use or deletion tombstones. A download end does not by itself prove success or execution; a cookie expiry is a configured deadline. Recovery status, confidence and origin remain in context. |
| Internet Explorer / legacy Edge WebCache | Stored entry access, creation/modification, expiry and synchronization, download starts, cookie metadata and container access/scavenging. Cache bookkeeping is not automatically evidence of a user opening a page. |
| Outlook PST, OST, MSG and OFT | Message submission/delivery; folder, item and attachment creation/modification; calendar start/end/reminders; task start/due/completion/reminders; and explicitly typed time properties. Scheduled times do not prove a meeting occurred, a reminder fired or a program executed. A stored delivery time does not prove the message was read. |
| User Access Logging (UAL) | First/last access metadata, database insertion/creation, first/last seen and stored operating-system boot time. Role, account, source address and daily counters remain context. Daily dates/counts are not invented midnight logons; untimed identity and DNS rows remain in the original parsed results. |
| Amcache | Inventory key last writes, parent volume key context, recorded install dates and PE link metadata. Inventory presence or LinkDate alone does not prove execution. |
| ShimCache | Stored file modification, cache-update metadata and cache key last write. None is automatically an execution timestamp. |
| SRUM | Provider timestamp/bucket, connection start, explicit start/end interval fields and application-identifier metadata. Bytes, duration and other counters are not dates. |
| LNK | Target MAC timestamps, tracking-identifier creation and shell-item times. They describe stored target metadata, not necessarily when the shortcut was opened. |
| Jump Lists | Entry last-use metadata, tracking-identifier creation and embedded LNK target times. The default omits collection-side container file times. |
| Recycle Bin | When each item was moved to the bin, with the bin's user, the original path and whether the deleted content was collected. It is not when the bin was emptied, and not the item's own file times. |
| Defender quarantine | Entry timestamp and stored resource creation/access/write times. Extraction times are not evidence of endpoint activity. |
| BAM, UserAssist, FeatureUsage | Recorded activity fields where present, with key last-write context kept distinct. A counter without a time does not create an event. |
| RecentDocs, MRU, Open/Save MRU, ShellBags, Office | Recorded use/trust/shell-item times and key context. A list's key time is not the individual last-use time of every item in the list. |
| SAM and LSA secrets | Recorded account logon/password/expiration or secret-set metadata and related key times. These do not replace authentication event logs. |
| Services, TaskCache, registry persistence, BCD | Key/configuration metadata and explicitly recorded task times. Service-key writes alone do not prove service execution. |
| Networks and USB | Network profile/lease and device-install/arrival/removal fields, with registry key context. Native USB user MountPoints2 times retain the corresponding user-hive source. |
| System Info, installed programs, AppCompat, application history | Explicit install, shutdown, inventory/activity and key metadata fields. Native System Info profile/version arrays retain their nested times. |
| Audit policy, security settings, MuiCache, generic registry | Key last-write times when supplied; untimed values/counters remain available in native input. |
| Persistence Hunter | Supporting key last write and correlated change time. A finding is not itself a timestamped installation or a verdict of compromise. |
| Hayabusa | Event/alert timestamp with rule, level, channel, event ID and available detail context. Multiple rules and distinct correlation rows remain separate. |

The default export retains all timestamp projections, including repeated metadata. It performs no
global sorting or duplicate consolidation. With `--sort`, duplicate projections are consolidated;
this can produce fewer rows than an unsorted export of the same inputs. Multiple equal timestamps
with different meanings or evidence sources are retained in both modes. Duplicate identity
includes the computer, source, artifact, record, time meaning and recovery state. Consolidated rows
record `DuplicateCount` and bounded alternative paths/source fields. Distinct sources are never
merged merely because their paths and timestamps look alike. When the raw source cannot be resolved,
input-record identity is retained to avoid collapsing different captures.

### Precision, local times and assumptions

Known UTC fields and timestamps with explicit offsets are normalized to UTC. Original values and
their recorded precision are retained, including 100 ns FILETIMEs. A source parser or downstream
viewer may already have rounded a time; this command cannot restore lost precision.

Some shell-item/FAT fields have local-time semantics even when an upstream parser serializes them
with `Z`. The timeline preserves the parsed value and field origin; it cannot recover the missing
endpoint time zone or validate an upstream conversion. Verify the raw artifact and endpoint settings
before using such a time for precise cross-host correlation.

A naive timestamp with no artifact-defined time zone is counted as unresolved, rather than silently
treated as UTC. If the evidence justifies it, supply a signed fixed offset:

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

Date-only values require that option too. Their midnight anchor is labeled as an assumption with
**day precision**; it is not an observed midnight event. Time-zone names and daylight-saving rules
are not accepted. Missing, zero, invalid and out-of-range values do not produce synthetic epoch
events. An explicitly recorded valid 1970 date is retained.

Two optional categories are excluded by default:

- `--include-collection-times`: LNK/Jump List source-file metadata on the collection/analysis
  filesystem. These events are labeled `collection_filesystem`, separate from endpoint activity.
- `--include-interpreted-times`: generic registry QWORD values rendered as FILETIME by the registry
  reader. They are labeled `interpreted_value` because many such values are actually counters.

## Output and skipped records { #output-and-skipped-records }

The command writes **only the timeline** selected with `-o`, in the format chosen with `-t`
(`csv` by default, or `jsonl`). It does not create untimed, diagnostic or summary companion files,
or per-record diagnostic spools. Existing companion files from earlier runs are left untouched.

The terminal summary reports the number of timeline events and untimed records. Nonzero invalid
timestamp and unresolved-time-zone counts are shown separately. With `--sort`, it also reports the
number of consolidated duplicates. Keep the original parsed results for records that do not produce
a timeline event and for the full artifact context.

Valid dates awaiting a justified time-zone assumption count as unresolved time zones, rather than
invalid timestamps. No timestamp is invented, and records without a usable mapped time are not
inserted into the timeline as synthetic epoch events.

Records with valid times outside the requested range are counted as filtered, not untimed.
A successful export means the selected parsed inputs were processed successfully; it does not
mean every artifact was collected, every field has a timestamp or every attack is detectable.

CSV exports can flatten nested metadata: System Info profiles, USB user mount points, shell items
and other structured context can be richer in native JSON/JSONL. Use native input when those
fields matter. The timeline cannot reconstruct data omitted by a CSV export. Unsupported artifact
IDs and malformed input files cause an error instead of a success containing partial results.

## Performance and output safety

Inputs are streamed and processed in parallel. By default, each worker writes a formatted event
fragment; fragments are concatenated in input-file order without sorting or re-reading their event
structures. CSV has one header even when multiple files are processed. Output order is deterministic
across worker counts; it does not imply chronology.

`Input files` shows the combined size of the selected parsed inputs and Hayabusa timelines.
`Results saved` shows the size of the saved timeline file. Both use the same human-readable
`KiB`/`MiB`/`GiB` units as `auto-parse`.

With `--sort`, buffers spill to a private temporary directory, then bounded merge passes produce
chronological output. Sorting uses full integer seconds and nanoseconds, including dates before 1970.
Equal-time ordering is deterministic across worker counts and sort budgets. A progress bar advances
for each event processed during intermediate and final merges, including repeated passes. It appears
on a terminal and is hidden when output is redirected.

`--memory-limit` applies only with `--sort`: it is the aggregate **sort-buffer budget**, not a
process RSS limit. Input records, JSON object/string overhead, CSV buffers and merge records require
additional memory. Worker count is reduced for small budgets. `--max-record-size` limits a parsed input record (default 16 MiB);
a pretty JSON array/report counts as one top-level input document. Larger documents may need a
higher limit or, preferably, JSONL. CSV parser read-ahead also contributes to this limit.

Temporary disk space must accommodate the formatted event fragments (default) or expanded sort
records and merge passes (`--sort`), in addition to final results. Both modes retain temporary output
staging and protection against publishing failed inputs. A record can yield many events, so the
timeline can be much larger than the input. Filter the time range/artifacts when appropriate and
select a temporary volume with adequate free space.

Ctrl+C cancels cooperatively, removes the private sort/staging directories and exits with code
`130`. On Linux and macOS this also applies to SIGTERM and SIGHUP. Cancellation is checked while
reading, normalizing, merging and writing the timeline, before publication begins. Uncatchable termination
or power loss can still leave temporary files.

Existing outputs require `-C`. The timeline must not overwrite inputs, manifests,
referenced local evidence or their protected evidence directories. Input errors or sort failures
leave the existing timeline untouched. The completed timeline is staged beside its destination
and replaces it atomically. Use a new output filename for each run when retaining an earlier
complete export matters.

## Further details

- [Timesketch CSV/JSONL import](https://timesketch.org/guides/user/import-from-json-csv/)
- [Plaso timestamp expansion and legacy CSV](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Plaso output and formatting](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [Evidence and interpretation](../overview/evidence-model.md)
- [Output formats](../getting-started/output-formats.md)

Current Juntei inputs use abbreviated labels such as `Comp`, `Filename`, `Args` and `Attr`. MFT CSV also accepts the current `File`, `SN`, `ParentEntryNum`, `ParentSN`, `Type`, `USN`, `LogfileSN` and `HasADS` headers, keeping sequence-number generations and DOS/ADS distinctions intact. Older exported labels remain accepted, and `SourceField` retains the spelling in the actual input. Hayabusa input retains its own `Computer` field. Boot-sector and security-descriptor metadata without event timestamps remains in the original parsed results; it does not create timeline events.

## Newer multi-table parsers

Native JSON/JSONL carries each table name. For a standalone CSV, the browser, WebCache and Outlook adapters identify its table from the column schema, so renaming the CSV or using automatic parsing's source suffixes does not change timestamp interpretation. UAL keeps its explicit known timestamp fields; access counters and unrelated date-looking text are not guessed as dates.

Outlook property rows are projected only when their declared type is a time or multiple-time type. A string containing a date remains a string. Embedded messages keep their item IDs and parent references, and attachment extraction times are not substituted for stored attachment timestamps. Untimed file, recipient, contact or property rows remain traceable in the native output and untimed report.

The same CSV and JSONL output schemas apply to these parsers; adding a new artifact does not add hundreds of CSV columns. Artifact-specific context stays in `Details`.
