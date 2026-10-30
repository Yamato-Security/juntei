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
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -t jsonl --single-file
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

## Parsers it runs

One parser failing does not stop the others. `auto-parse` runs these jobs for every discovered
computer:

- Amcache
- `$MFT`
- Prefetch records and a separate Prefetch execution timeline
- USN Journal
- `$I30` directory indexes
- SRUM
- LNK shortcuts
- Jump Lists
- ShimCache
- Defender quarantine metadata
- all registry keys and values, including recoverable deleted data
- services, RecentDocs, TaskCache, and SAM accounts

CSV `$MFT` output includes all timestamps and short names. Registry output includes raw data and a
FILETIME interpretation of `REG_QWORD` values. JSON and JSONL SRUM output includes value-state
metadata. Defender files and resident `$MFT` streams are **not extracted** automatically.

An artifact that was not collected is a normal `skipped` task, not a failure.

## Output layout

CSV keeps separate tables because each artifact has different columns:

```text
<OUTPUT>/
├── manifest.json
├── HOST1/
│   ├── logs/
│   ├── amcache/
│   ├── defender-quarantined/  # only with -x
│   ├── srum/
│   ├── mft.csv
│   ├── usn-journal.csv
│   ├── prefetch.csv
│   ├── prefetch-timeline.csv
│   ├── registry.csv
│   └── shimcache.csv
└── HOST2/
    └── …
```

Some parsers naturally create several tables, so their output is a directory. `logs` contains the
exact command and messages from every parser. `manifest.json` records hostname provenance,
evidence paths, command lines, duration, exit code, result path, whether Defender extraction was
requested, and whether each task succeeded, was skipped, or failed.

When a host has one Amcache hive or SRUM database, its CSV tables are written directly under
`amcache/` or `srum/`. If a collection contains several databases of either kind, each source
receives a collision-safe subdirectory so that tables with the same names cannot overwrite one
another. A normal single-volume collection writes `$MFT` and USN Journal records directly to
`mft.csv` and `usn-journal.csv`. Additional collected volumes use numbered sibling files such as
`mft-2.csv` rather than another directory level.

## JSON and SIEM ingestion

Every JSON or JSONL record receives two top-level fields:

- `Computer`: the source system.
- `Artifact`: the Juntei parser that produced the record, such as `mft` or `prefetch-timeline`.

Without `--single-file`, data remains organized by computer and artifact. With `--single-file`,
Juntei streams all successful records into either `all-artifacts.json` or
`all-artifacts.jsonl`. The per-parser data files are temporary; per-host logs and the manifest are
retained.

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
      --single-file         For JSON/JSONL, create one all-artifacts file instead of per-artifact data files
  -C, --clobber             Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined
                            Extract Defender-quarantined files into each computer's defender-quarantined directory

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK and Jump List files [default: 1252]
```

Use a new output directory when practical. `-C` removes or overwrites only the exact generated
targets for this run, so a now-missing artifact cannot leave a stale result. It does not delete
unrelated files or the complete output tree. It also removes every per-host
`defender-quarantined/` directory from an earlier run before parsing, whether or not the new run
uses `-x`, so extracted malware cannot remain as an unrecorded stale result.

!!! danger "Treat extracted Defender content as malware"
    `-x, --extract-defender-quarantined` writes quarantined content to disk under each computer's
    `defender-quarantined/` directory. Every extracted filename ends in `.quarantined`, and Juntei
    removes execute permission. Never execute the files. Use an isolated analysis system and expect
    antivirus or EDR software to alert on or remove the recovered content.
