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

The compiled rules are shared between workers; each worker reads one complete file.
There is **no default file-size limit** for ordinary files or MFTs. Memory use depends on
file sizes, parallel scans, compiled rules and engine match data. Set optional caps with
`--max-file-mib` and/or `--max-mft-mib`, or reduce `-T` when working with large files.
These are file-size caps, not a process-wide memory cap. Files are hashed and scanned from
the same in-memory bytes; detected size/modification-time changes during reading cause a
read error. Prefer stable evidence copies. An allocation or I/O failure is reported as
incomplete coverage, never as a clean scan.

Ctrl+C requests cancellation between files. An in-progress evaluation can take up to its
configured timeout before it returns. Interrupted files are recorded as cancelled.

## Results

| File | Contents |
|---|---|
| `YaraDetections.csv` or `.jsonl` | One row per rule/file match, including partial matches from incomplete scans. |
| `YaraFiles.csv` or `.jsonl` | Coverage for each discovered file, skipped link/special entry, directory discovery error and candidate MFT resident stream. |
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

## Raw MFT scanning and matched resident-file recovery

Collected MFTs are recognized by common names (`$MFT`, names ending in `$MFT`, percent-encoded
`%24MFT`, `MFT`, `.mft`) or a plausible leading FILE record header. Their raw bytes are scanned
without a size cap by default. An optional `--max-file-mib` cap applies to all files;
`--max-mft-mib` overrides it for MFTs. Both accept positive whole MiB values within the
64-bit byte-count range. MFTs run **one at a time after ordinary files**, sharing the compiled
rules. This avoids several large MFT buffers being held by parallel workers.

When a raw rule matches, Juntei uses its pattern offsets to locate candidate FILE records.
It validates the records, applies NTFS update-sequence fixups, and re-evaluates each resident
`$DATA` stream in those records as an independent file. This makes `filesize`, offsets and
compound conditions refer to the recovered content rather than the entire MFT.
**Only independently matched resident streams are saved**, under `RecoveredMFT/`, with a
`.quarantined` suffix. The flat, collision-resistant names retain the original basename plus
record/generation/attribute identifiers; evidence directory paths are never recreated.
Source evidence remains unchanged. Named alternate streams and deleted records are included.
Extension-record owners are asserted only when both sequence numbers and the base record's
attribute list confirm that relationship.

This is **raw-hit-driven recovery**, not an exhaustive scan of every resident stream. A rule
that cannot match the raw MFT at all (for example, one restricted to very small files) cannot
by itself nominate a record for recovery. Separately extract and scan resident streams for
that additional coverage. Metadata-only matches, patterns spanning FILE records, unconfirmed
compound matches, and rules without usable pattern offsets do not export a supposed payload.
Nonresident contents require the original volume's data clusters and cannot be recovered from
a standalone MFT; no empty placeholder is written for them. Damaged records are reported.

`--max-mft-recoveries <N>` limits actual recovered streams across the run (default **1000**,
range 1–100000). Reaching this limit or the raw-offset storage ceiling makes recovery coverage
incomplete and visible in the results. The display limit of 16 match-pattern samples does
not limit recovery: it uses all retained raw occurrence offsets, up to 100,000 per MFT and the
engine's 1000-occurrences-per-pattern ceiling. Reaching either ceiling is reported.

```bash
juntei yara-scan -d evidence/ --ruleset full -o yara-mft/
```

MFT processing adds `YaraMFTResults.csv` or `YaraMFTResults.jsonl` when MFT inputs are found:

| Field | Meaning |
|---|---|
| `Computer`, `MFTPath` | Evidence host and relative source MFT path. |
| `EntryNum`, `SN` | Physical FILE record slot and sequence/generation. |
| `BaseEntryNum`, `BaseSN` | Verified base owner of an extension record, otherwise empty. |
| `OriginalPath`, `Stream` | Resolved Windows file path and named stream; unresolved paths are labelled. |
| `AttributeID`, `AttributeOffset` | Attribute instance and its byte offset within the record. |
| `Deleted` | Whether the physical FILE record is unallocated. |
| `RawRules`, `StreamRules` | Raw-container rule names and independently matching stream rule names. |
| `Status`, `Note` | Recovery result and explanation. |
| `Size`, `SHA256` | Exact fixup-restored resident content size and hash; nonresident rows have no content hash. |
| `RecoveredFile` | Recovered file path relative to the results directory, empty when no file was saved. |

Statuses distinguish `Recovered`, `ResidentNotConfirmed`, `NonResidentUnavailable`,
`MetadataOnly`, `NoMatchOffsets`, `CrossRecordSpan`, `InvalidMFT`, `InvalidRecord`,
`AttributeError`, `TruncatedRecord`, `StreamScanError`, `RawLocationLimit`,
`RecoveryLimitReached` and `Cancelled`. Recovery errors and limits make coverage incomplete;
expected absence of recoverable nonresident bytes is explicitly reported.

The ordinary detections and coverage tables now include `SourceType`: `File`, `MFTRaw`, or
`MFTResident`. Resident `FilePath` values are virtual references to a record and stream,
not additional source files. Detection rows also include `RecoveredFile`; raw MFT offsets
are relative to the MFT, while resident-stream offsets are relative to the recovered content.
A raw and a resident match can represent the same evidence, so counts are not unique malware
counts. `Summary.json` includes the effective MFT limit, explicit MFT override and recovery statistics. A null file-size limit means unlimited.

A saved file is **YARA-matched evidence**, not a verdict of malware or proof of execution.
