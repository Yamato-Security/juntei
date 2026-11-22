# YARA Scan

`yara-scan` searches collected file contents for YARA patterns using the portable Rust
[Boreal](https://github.com/vthib/boreal) engine. Files are read as data and never executed.
The command works on Windows, macOS and Linux, including files recovered from Defender
quarantine with a `.quarantined` extension. It is independent of `auto-parse` and does not
run automatically during artifact parsing.

## Choose your rules

Without `--rules`, Juntei downloads the selected [YARA Forge](https://github.com/YARAHQ/yara-forge)
package over HTTPS from its GitHub releases, compiles it, then scans:

| Package | Selection | Intended use |
|---|---|---|
| Core | Default, or `--ruleset core` | Start here: the curated, narrower package prioritizes accuracy and performance. |
| Extended | `--ruleset extended` | Broader hunting coverage; review additional matches in context. |
| Full | `--ruleset full` | Broadest coverage, with more resource use and potential false positives. |

```bash
juntei yara-scan -d evidence/ -o yara-core/
juntei yara-scan -d evidence/ --ruleset extended -o yara-extended/
juntei yara-scan -d evidence/ --ruleset full -o yara-full/ -t jsonl
```

By default, each online invocation downloads the latest package. To request a specific
release, add `--rules-release YYYYMMDD`, for example `--rules-release 20261115` if that release
is available. Keep `Rules.yar` and `Summary.json` with the results: even when `latest` changes,
the saved source and its SHA256 identify the exact rules used.

For offline scanning, provide a rule file or a directory of `.yar`/`.yara` files:

```bash
juntei yara-scan -d evidence/ --rules local-rules/ -o yara-offline/
juntei yara-scan -f recovered/agent.exe.quarantined --rules local.yar -o yara-file/
```

`--rules` makes no network requests and cannot be combined with an explicitly supplied
`--ruleset` or `--rules-release`. Local directories are searched recursively. Rules share
one namespace and are read in sorted path order; use `include` for dependencies.
Includes resolve relative to the containing file, stay inside the selected directory
(or the parent of a selected rule file), and are expanded once. Circular includes and
includes outside that boundary fail. Duplicate rule names or unsupported modules also
fail compilation; Juntei does not silently discard rules. Local rules must be UTF-8.

Compilation warnings are saved to `CompilationWarnings.txt`. A failed compilation writes
`CompilationError.txt` when compilation was reached and returns a setup error.
`Rules.yar` is a flattened local-rule snapshot or the original downloaded source.
Rule authors retain their respective licenses; downloading a package does not relicense it.

## Scope and limits

All regular files are eligible, irrespective of their extension. Directory scans are
recursive. Symbolic links and special files are reported as skipped, rather than followed.
Local rule sources inside the evidence directory are excluded to avoid self-matches.
Results must use a **new directory outside the input evidence** and must not overwrite
local rules. A single-file scan may save results beside that file in a new subdirectory.
The evidence is not modified.

This command scans complete file bytes using **file semantics**, not process-memory
semantics. It does not unpack ZIP archives or disk images, recover nonresident file data,
or scan live process memory. Point it at extracted files or Defender quarantine payloads,
not at raw VHDX/E01 images expecting filesystem-level coverage. An empty placeholder
created from MFT metadata contains no recoverable malware bytes to inspect.

| Option | Default | Effect |
|---|---|---|
| `-d, --directory <DIR>` | Required unless `-f` is used | Scan a directory recursively. |
| `-f, --file <FILE>` | Required unless `-d` is used | Scan one file. |
| `-o, --output <DIR>` | Required | New results directory. |
| `-r, --rules <PATH>` | None | Offline rule file/directory. |
| `--ruleset core\|extended\|full` | `core` | Package to download. |
| `--rules-release <RELEASE>` | `latest` | `latest` or a release date in `YYYYMMDD` form. |
| `-t, --output-type csv\|jsonl` | `csv` | Format of detections and per-file coverage. |
| `-T, --threads <N>` | CPU count, capped at 8 | Parallel file scans; 1–256. |
| `--timeout <SECONDS>` | 60 | Per-file YARA evaluation timeout, excluding file reading. |
| `--max-file-mib <MIB>` | Unlimited | Optional positive MiB cap for all files, including MFTs. |
| `--max-mft-mib <MIB>` | Inherit global cap | Optional positive MiB override for MFT files. |
| `--computer <NAME>` | Nearby `client_info.json`, otherwise empty | Override the evidence computer name. |
| `-q, --quiet` | Off | Suppress the banner/progress and routine preparation messages. |
| `-K, --no-color` | Off | Disable terminal colors. |

The compiled rules are shared between workers. Each worker maps its file read-only and hashes
and scans the mapping, so nothing is copied into memory and memory use does not grow with file
size or with the number of parallel scans; the operating system drops the mapped pages again as
the scan moves on. There is **no default file-size limit** for ordinary files or MFTs. The
optional `--max-file-mib` and `--max-mft-mib` caps bound scan time, not memory. Files are hashed
and scanned from the same mapping; a size or modification-time change detected during the scan
is reported as a read error, so prefer stable evidence copies, and never let another process
truncate a file while it is being scanned. An I/O failure is reported as incomplete coverage,
never as a clean scan.

Ctrl+C requests cancellation between files. An in-progress evaluation can take up to its
configured timeout before it returns. Interrupted files are recorded as cancelled.

## Results

| File | Contents |
|---|---|
| `YaraDetections.csv` or `.jsonl` | One row per rule/file match, including partial matches from incomplete scans. |
| `YaraFiles.csv` or `.jsonl` | Coverage for each discovered file, skipped link/special entry, directory discovery error and every MFT resident stream that matched or could not be scanned. |
| `Summary.json` | Completion status, status counts, settings, rule counts, rules/package hashes, source manifests and engine version. |
| `Rules.yar` | Exact downloaded rules or flattened local rules, for reproducibility. |
| `CompilationWarnings.txt` | Engine compilation warnings; may be empty. |
| `RunStarted.json` | Initial run marker. If no `Summary.json` exists, the run did not finish successfully. |

Paths in exported evidence rows are relative to the selected directory, or the containing
directory of a single file. `Computer` comes from a nearby `client_info.json` or `--computer`;
unknown hosts stay empty instead of being assigned the analyst's computer name.

### Detection fields

| Field | Interpretation |
|---|---|
| `Computer` | Evidence host, when known. |
| `FilePath` | Relative source path; `.quarantined` is retained. |
| `Size` | Number of bytes scanned. |
| `SHA256` | Hash of those same bytes. |
| `ScanStatus` | `Scanned` for completed evaluation, or `ScanError` when results are partial. |
| `Rule`, `Namespace`, `Tags` | Rule identity and tags from the engine. |
| `Metadata` | JSON-encoded rule metadata, such as author, description, reference or severity if supplied by its author. |
| `MatchLocations` | JSON-encoded first occurrence per matched pattern, up to 16 patterns, with byte `Offset`, `Length` and `XorKey`. Matched content is not exported. |

### Coverage fields

`Computer`, `FilePath`, `Size` and `SHA256` identify each file. Unread or skipped files have
an empty hash. `Matches` counts matched rules, `ElapsedMs` measures file read/hash/evaluation
work, and `Note` supplies an error explanation when available.

`Status` distinguishes `Scanned`, `SkippedTooLarge`, `SkippedSymlink`, `SkippedSpecialFile`,
`SkippedRuleSource`, `DiscoveryError`, `ReadError`, `ScanError` and `Cancelled`.
Skipped files and timeouts are **not clean scans**. A file can have partial rule matches
and still have `ScanError`. Detection count is the number of rule/file pairs, not a count
of unique malware families or compromised computers.

Exit status is 0 for complete coverage of the selected regular files (matches are not an
error), 3 for incomplete coverage, 2 for a setup/output failure, and 130 for cancellation.
`SkippedRuleSource` alone does not make coverage incomplete. Check `Summary.json` first,
then review matches alongside process execution, file-system and event-log evidence.
A YARA match is a hunting lead, and absence of matches is not proof that a file is safe.

## Further Details

- [YARA Forge packages and releases](https://github.com/YARAHQ/yara-forge)
- [Boreal engine and compatibility](https://github.com/vthib/boreal)
- [YARA rule-writing reference](https://yara.readthedocs.io/en/stable/writingrules.html)

## MFT scanning and resident-file recovery

Collected MFTs are recognized by common names (`$MFT`, names ending in `$MFT`, percent-encoded
`%24MFT`, `MFT`, `.mft`) or a plausible leading FILE record header. They are processed **one at
a time after the ordinary files**, with the same compiled rules, in two passes:

1. The raw bytes are mapped and scanned like any other file. Matches on file names, index
   entries or slack appear as detections with `SourceType` `MFTRaw` and offsets relative to the
   MFT.
2. Every FILE record is read from the file one record at a time, never from a copy of the whole
   MFT. For each record with valid NTFS update-sequence fixups, every resident `$DATA` stream is
   scanned as an independent file, so `filesize`, header checks and compound conditions refer to
   that file rather than to the MFT. This pass does not depend on the first one: a rule that can
   never match the MFT as a whole, for example one restricted to small PE files, still finds the
   resident files it was written for.

**Only resident streams that match on their own are saved**, under `RecoveredMFT/`, with a
`.quarantined` suffix. The flat, collision-resistant names retain the original basename plus
record/generation/attribute identifiers; evidence directory paths are never recreated. Source
evidence remains unchanged. Named alternate streams and deleted records are included.
Extension-record owners are asserted only when both sequence numbers and the base record's
attribute list confirm that relationship. Nonresident contents live in the volume's data
clusters and cannot be recovered from a standalone MFT; they are counted in `Summary.json`, and
no empty placeholder is written for them. Unused (zeroed) record slots are counted. A record
whose signature, size or fixups are invalid is reported as `InvalidRecord`, and because its
content could not be checked, it makes coverage incomplete.

An optional `--max-file-mib` cap applies to all files; `--max-mft-mib` overrides it for MFTs.
Both accept positive whole MiB values within the 64-bit byte-count range, and a skipped MFT gets
neither pass. `--max-mft-recoveries <N>` limits the streams actually saved across the run
(default **1000**, range 1–100000); matching streams beyond the limit are still reported as
detections, with the status `RecoveryLimitReached`, and the run is incomplete. The display limit
of 16 match-pattern samples does not affect recovery.

```bash
juntei yara-scan -d evidence/ --ruleset full -o yara-mft/
```

MFT processing adds `YaraMFTResults.csv` or `YaraMFTResults.jsonl` when MFT inputs are found. It
has one row per recovered stream, per matching stream that was not saved, per stream whose
evaluation failed, and per damaged record (the first 100; the rest are counted):

| Field | Meaning |
|---|---|
| `Computer`, `MFTPath` | Evidence host and relative source MFT path. |
| `EntryNum`, `SN` | Physical FILE record slot and sequence/generation. |
| `BaseEntryNum`, `BaseSN` | Verified base owner of an extension record, otherwise empty. |
| `OriginalPath`, `Stream` | Resolved Windows file path and named stream; unresolved paths are labelled. |
| `AttributeID`, `AttributeOffset` | Attribute instance and its byte offset within the record. |
| `Deleted` | Whether the physical FILE record is unallocated. |
| `Rules` | Rule names that matched the stream on its own. |
| `Status`, `Note` | Recovery result and explanation. |
| `Size`, `SHA256` | Exact fixup-restored resident content size and hash. |
| `RecoveredFile` | Recovered file path relative to the results directory, empty when no file was saved. |

Statuses distinguish `Recovered`, `RecoveryLimitReached`, `StreamScanError`, `InvalidMFT`,
`InvalidRecord`, `AttributeError`, `TruncatedTail` and `Cancelled`. Scan errors, damaged records
and the recovery limit make coverage incomplete. `TruncatedTail` only notes bytes after the last
complete record, which the raw pass has already covered.

The ordinary detections and coverage tables include `SourceType`: `File`, `MFTRaw`, or
`MFTResident`. Resident `FilePath` values are virtual references to a record and stream,
not additional source files. Detection rows also include `RecoveredFile`; raw MFT offsets
are relative to the MFT, while resident-stream offsets are relative to the recovered content.
A raw and a resident match can represent the same evidence, so counts are not unique malware
counts. `Summary.json` includes the effective MFT limit, explicit MFT override and the record
statistics: records read, unused and damaged records, resident and nonresident streams, and
recovered files. A null file-size limit means unlimited.

A saved file is **YARA-matched evidence**, not a verdict of malware or proof of execution.
