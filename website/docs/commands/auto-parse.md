# Automatic Collection Parsing

`auto-parse` is the quickest way to turn a multi-computer Windows triage collection into a
structured analysis directory. Give it the evidence root and a new output directory. It discovers
the collected systems, identifies their computer names, and runs every compatible Juntei parser in
parallel.

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

The default output is CSV. Use JSONL when the result will be queried with `jq` or ingested into a
SIEM:

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -t jsonl -s
```

!!! warning "Keep the output and evidence trees separate"
    Juntei refuses an output directory that is inside `<EVIDENCE>` or contains `<EVIDENCE>`. This
    prevents generated data from being rediscovered as evidence and prevents `--clobber` from
    touching files in the source collection.

## What it discovers

Juntei searches recursively for directories named `Windows` that contain a `System32`, `Prefetch`,
or `AppCompat` directory, case-insensitively. Recognizing the latter two also supports targeted
collections that contain only Prefetch or Amcache. This excludes user-profile directories such as
`AppData\Roaming\Microsoft\Windows`. Nested legacy installations such as
`C:\Windows.old\Windows` are assigned to the outer collected volume instead of becoming duplicate
computers. The default maximum depth is 10. This covers common Velociraptor paths such as:

```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

Change the limit only when the collection is nested more deeply:

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

For each discovered system, the computer name is chosen in this order:

1. `hostname` in the nearest ancestor `client_info.json` (with `fqdn` retained in the manifest).
2. `ComputerName` in a collected `SYSTEM` registry hive.
3. The nearest useful collection-directory name, with drive and Velociraptor `uploads/auto`
   scaffolding skipped and a suffix such as `_F.…` removed.

Names are made safe for use as directory names. Duplicate names receive a numeric suffix in both
their output directory and `Computer` value so separate systems remain distinguishable in JSON.
The manifest keeps the hostname exactly as discovered in a separate `hostname` field.

## Parsers it runs

One parser failing does not stop the others. `auto-parse` runs these jobs for every discovered
computer. Jobs use the same source-grouped commands as manual parsing, such as
`parse-ntfs mft`, `parse-registry amcache`, `parse-ese srum`, and `parse-file prefetch`:

- Amcache
- `$MFT`
- `$LogFile` NTFS redo/undo records
- Prefetch records and a separate Prefetch execution timeline
- USN Journal
- `$I30` directory indexes
- SRUM
- LNK shortcuts
- Jump Lists
- ShimCache
- Defender quarantine metadata
- all registry keys and values, including recoverable deleted data
- services, RecentDocs, TaskCache, SAM accounts, registry autostart locations, and UserAssist

CSV `$MFT` output includes all timestamps and short names. Registry output includes raw data and a
FILETIME interpretation of `REG_QWORD` values. JSON and JSONL SRUM output includes value-state
metadata. Defender files and resident `$MFT` streams are **not extracted** automatically.

An artifact that was not collected is a normal `skipped` task, not a failure.

## Output layout

Results are organized by computer, investigative category, and artifact. Every category has its
own `logs/` directory, including when its artifacts were not collected. CSV keeps separate tables
because each artifact has different columns:

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── accounts/
│   │   ├── logs/
│   │   └── sam.csv
│   ├── file_and_folder_opening/
│   │   ├── logs/
│   │   ├── jump-lists/
│   │   ├── lnk.csv
│   │   └── recentdocs.csv
│   ├── file_existence/
│   │   ├── logs/
│   │   ├── amcache/
│   │   └── shimcache.csv
│   ├── file_system/
│   │   ├── logs/
│   │   ├── i30.csv
│   │   ├── logfile.csv
│   │   ├── mft.csv
│   │   └── usn-journal.csv
│   ├── persistence/
│   │   ├── logs/
│   │   ├── persistence.csv
│   │   ├── services.csv
│   │   └── taskcache.csv
│   ├── process_execution/
│   │   ├── logs/
│   │   ├── prefetch.csv
│   │   ├── prefetch-timeline.csv
│   │   ├── srum/
│   │   └── userassist.csv
│   ├── registry/
│   │   ├── logs/
│   │   └── registry.csv
│   └── security_products/
│       ├── logs/
│       ├── defender-quarantine.csv
│       └── defender-quarantined/  # only with -x
└── HOST2/
    └── …
```

| Category | Artifacts |
|---|---|
| `accounts` | SAM local accounts |
| `file_and_folder_opening` | Jump Lists, LNK shortcuts, RecentDocs |
| `file_existence` | Amcache, ShimCache |
| `file_system` | `$I30`, `$LogFile`, `$MFT`, USN Journal |
| `persistence` | Registry autostarts, services, TaskCache |
| `process_execution` | Prefetch, Prefetch timeline, SRUM, UserAssist |
| `registry` | General hive keys and values, including recovered records |
| `security_products` | Defender quarantine metadata and optional extracted content |

Categories help organize investigations; they are not proof of a particular activity. For example,
Amcache and ShimCache entries alone do not establish execution. SRUM records resource usage;
consult the artifact guide before drawing conclusions about a program running.

CSV, JSON, and JSONL use the same categories. Parsers that produce several tables retain their
artifact subdirectory where needed. Each category's `logs/` contains one `.txt` file per parser
job with its exact command and messages; for example,
`HOST1/file_system/logs/mft.txt` and `HOST1/process_execution/logs/prefetch.txt`.

`manifest.json` records the original discovered `hostname`, the unique output/record `Computer`
identity, hostname provenance, evidence paths, artifact `category`, command lines, duration, exit
code, result path, log path, whether Defender extraction was requested, and whether each task
succeeded, was skipped, or failed.

The progress message shows the active computer and artifact followed by an approximate input
size, for example `HOST1 / prefetch (~2.0 MiB)`. A group of files uses their combined size; each
MFT, LogFile or USN Journal task shows its individual input size. Estimates exclude auxiliary transaction
logs and enrichment files and do not predict the size of the parsed output.

`manifest.csv` provides the same task index as a spreadsheet-friendly table, with one row per
parser task. It includes `Computer`, original `Hostname`, `Fqdn`, `HostnameSource`, `EvidenceRoot`,
`Artifact`, `Category`, `Status`, `DurationMs`, `ExitCode`, `Output`, `Log`, `Error`, and `Command`.
`Command` is a JSON array of the executable and its arguments, preserving spaces and quoting.
Filter `Status` to `skipped` or `failed` to find tasks requiring review; a skipped task's `Error`
may be empty, with the explanation in its `Log` file.

The terminal summary includes the total number and size of regular files currently beneath the
output directory, including both manifests, parser logs, and extracted files. Symbolic links are
not followed. Sizes use decimal MB (1 MB = 1,000,000 bytes):

```text
Results saved: output/ ( 125 files / 42.50 MB )
```

When a host has one Amcache hive or SRUM database, its CSV tables are written directly under
`file_existence/amcache/` or `process_execution/srum/`. If a collection contains several databases
of either kind, each source receives a collision-safe subdirectory so that tables with the same names cannot overwrite one
another. A normal single-volume collection writes `$MFT` and USN Journal records directly to
`file_system/mft.csv` and `file_system/usn-journal.csv`. Additional collected volumes use numbered
sibling files such as `file_system/mft-2.csv` and `file_system/logs/mft-2.txt`.

## JSON and SIEM ingestion

Every JSON or JSONL record receives two top-level fields:

- `Computer`: the source system.
- `Artifact`: the Juntei parser that produced the record, such as `mft` or `prefetch-timeline`.

Without `--single-file`, data remains organized by computer, category, and artifact. For example,
`HOST1/process_execution/prefetch.jsonl` contains Prefetch records. With `--single-file`,
Juntei streams all successful records into either `all-artifacts.json` or
`all-artifacts.jsonl`. The per-parser data files are temporary; each computer's category directories
and their `logs/` remain, along with both manifests. The combined dataset stays at the output root.

JSONL is recommended for a large collection because each line can be processed independently:

```bash
jq -c 'select(.Computer == "HOST1" and .Artifact == "prefetch")' \
  parsed/all-artifacts.jsonl
```

```bash
jq -r 'select(.Artifact == "mft") | .full_path? // empty' \
  parsed/all-artifacts.jsonl
```

Large `$MFT`, registry, and USN Journal collections can produce many gigabytes of output. Ensure
there is enough free space for the final consolidated file and temporary per-parser files. If any
parser fails, Juntei preserves the temporary directory for diagnosis instead of deleting it.

## Performance

Parser jobs are separate processes, so CPU-heavy artifacts can run at the same time and damage in
one parser cannot terminate the complete run. By default, Juntei uses the logical CPU count. Limit
concurrency when evidence is on a slow disk or the system has little memory:

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## Usage

```text
Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]

Output:
  -o, --output <DIR>        New directory in which to create per-computer results
  -t, --output-type <TYPE>  Output type: csv (default), json, or jsonl [default: csv]
  -s, --single-file         For JSON/JSONL, create one all-artifacts file instead of per-artifact data files
  -C, --clobber             Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined
                            Extract Defender-quarantined files

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK and Jump List files [default: 1252]
```

Use a new output directory when practical. `-C` removes or overwrites only the exact generated
targets for this run, so a now-missing artifact cannot leave a stale result. It does not delete
unrelated files or the complete output tree. It also removes each computer's
`security_products/defender-quarantined/` directory, and the reserved `defender-quarantined/`
directory from the previous flat layout, before parsing, whether or not the new run uses `-x`.

Use a fresh output directory when switching from an older flat layout. Earlier CSV/JSON files
are not relocated or deleted automatically.

!!! danger "Treat extracted Defender content as malware"
    `-x, --extract-defender-quarantined` writes quarantined content to disk under each computer's
    `security_products/defender-quarantined/` directory. Recovered binary streams end in `.quarantined`, and Juntei
    removes execute permission. Each binary security descriptor also gets a readable `.security_descriptor.txt`
    report with ownership, permissions and SDDL. Never execute recovered payloads. Use an isolated
    analysis system and expect antivirus or EDR software to alert on or remove the recovered content.

See [reading the recovered security descriptor](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)
for the report fields and their forensic limitations.
