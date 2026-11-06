# Automatic Collection Parsing

`auto-parse` is the quickest way to turn a multi-computer Windows triage collection into a
structured analysis directory. Give it the evidence root and a new output directory. It discovers
the collected systems, identifies their computer names, and runs every compatible Juntei parser in
parallel.

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

The default output is CSV. Add a JSONL timeline for `jq` or SIEM import while keeping those CSV
results:

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "Keep the output and evidence trees separate"
    Juntei refuses an output directory that is inside `<EVIDENCE>` or contains `<EVIDENCE>`. This
    prevents generated data from being rediscovered as evidence and prevents `--clobber` from
    touching files in the source collection.

## Output time zone

Parsed timestamps default to UTC, independent of the analyst computer's local time zone. UTC
uses the Windows event-log form `2026-10-20T08:46:38.2001668Z`: an ISO 8601 date/time, `T`
between the date and time, and `Z` for UTC. The available fractional digits are preserved.
`manifest.json`'s `GeneratedUTC` uses this form too. The console's `Start time` and `End time`
are the analyst computer's local time, such as `2026-11-04 09:15:02`, whatever the output time
zone.

Choose a fixed display offset with `-Z, --timezone`:

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

An offset is a signed, two-digit `HH:MM` value (hours `00–23`, minutes `00–59`). Names such as
`JST`, shortened values such as `+9`, and invalid ranges produce an error with example syntax.
Omit the option or use `--timezone "+00:00"` for UTC. A fixed offset does not apply daylight
saving rules.

The option applies to parsed timestamps in the per-artifact CSV, JSON and JSONL results and
the additional `-j` JSONL export. For example, `2026-10-20T08:46:38.2001668Z` becomes
`2026-10-20T17:46:38.2001668+09:00`; both describe the same instant.

Field names stay stable, including existing names ending in `UTC`. Converted values explicitly
carry the selected offset. `OutputTimeZone` in both manifests records the requested display
offset, while `GeneratedUTC` remains UTC run metadata. Per-parser logs retain the parsers' native UTC summaries.

Raw FILETIME/epoch numbers, registry-value dictionaries, source time-zone configuration, text
metadata, date-only values and local clock values without a known offset are preserved. Juntei
does not add `Z` to an unknown local time. FAT-derived shell-item clocks (`Item*UTC` in
ShellBags/OpenSaveMRU and LNK/Jump List `TargetItems`) have no recorded offset; their legacy
UTC labels or rendered `Z` suffix do not prove UTC, so the offset option preserves them.
CSV columns containing free-text summaries retain
their recorded text; JSON/JSONL preserves nested parsed timestamps as structured fields.

UTC uses the existing parser output directly. An explicit nonzero offset adds a streaming
formatting pass within each parser worker; it changes display values without changing evidence.

If an offset would move a timestamp outside years `0000–9999`, that value keeps its original
explicit offset. Values with more than nine fractional digits also stay unchanged rather than
being rounded. Juntei records a warning in the parser's `*-logs.txt` file and converts the other
timestamps. Malformed CSV/JSON or an output error still fails the task and leaves the affected
original result file unchanged.

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

1. `hostname` in the nearest ancestor `client_info.json` (with its value retained as `FQDN` in the manifest).
2. `ComputerName` in a collected `SYSTEM` registry hive.
3. The nearest useful collection-directory name, with drive and Velociraptor `uploads/auto`
   scaffolding skipped and a suffix such as `_F.…` removed.

Names are made safe for use as directory names. Duplicate names receive a numeric suffix in both
their output directory and `Comp` value so separate systems remain distinguishable in JSON.
The manifest keeps the hostname exactly as discovered in a separate `Hostname` field.

After listing the systems, `auto-parse` prints one `Timezone:` line directly above
`Parser tasks`. It shows the selected output timezone: `UTC` by default, or a value such as
`UTC+09:00` when `--timezone "+09:00"` is used. The label is green and the value uses the
terminal's default text color, like the other summary values. `-K, --no-color` keeps the line plain.
Collected systems' time-zone configuration is available in the
[System Information](../artifacts/system-info.md) results.

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
- `$Boot` boot sectors and `$Secure:$SDS` security descriptors
- SRUM
- LNK shortcuts
- Jump Lists
- ShimCache
- Persistence Hunter (131 cross-source persistence checks with allow/block profiles)
- Defender quarantine metadata
- Open and Save As dialog history
- all registry keys and values, including recoverable deleted data
- services, RecentDocs, TaskCache, SAM accounts, registry autostart locations, UserAssist, BAM,
  installed programs, MuiCache, ShellBags, and USB devices
- system information: names, Windows version, time zone, last shutdown, profiles and SIDs
- the audit policy: which security events the system was set to log
- boot configuration (BCD and BCD-Template)
- network profiles, interfaces, shares, port proxies and routes
- LSA secrets' names and times, and cached domain logons in use (nothing is decrypted)

Persistence Hunter retains every severity (`--min-severity info`), including partial allow matches.
`LOW` is a classification to investigate; missing signature metadata can produce it for malicious
commands too. The automatic inventory does not discard these entries using the manual command's
default medium threshold.

The hunter uses one worker inside each automatic parser process. This keeps `auto-parse -T <N>`
from multiplying its process budget by another CPU-sized pool in every child. Standalone
`persistence-hunter -T <N>` controls the hunter's own worker threads.

CSV `$MFT` output includes all timestamps and short names. Registry output includes raw data and a
FILETIME interpretation of `REG_QWORD` values. JSON and JSONL SRUM output includes value-state
metadata. Defender files and resident `$MFT` streams are **not extracted** automatically.

With `--mft-slack`, the `$MFT` output also includes the file names left in the slack of FILE
records, as [`parse-ntfs mft -x`](../artifacts/mft.md) writes them: rows marked `FromSlack` in
CSV, and `SlackFileNames` in JSON and JSONL. `--code-page` applies to LNK files, Jump Lists,
ShellBags and OpenSaveMRU.

An artifact that was not collected is a normal `skipped` task, not a failure.

## Output layout

Results are organized by computer and investigative category. Category directories use
PascalCase, such as `Accounts` and `FileAndFolderOpening`. Amcache and Jump Lists tables sit
directly in their categories with filename prefixes. Computer directory names retain their discovered spelling. Existing output
directories are not migrated; choose a new output directory when switching from the older naming. Each parser writes an
`<artifact>-logs.txt` file beside its results, including when the artifact was not collected.
CSV keeps separate tables because each artifact has different columns:

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   └── Registry-RemoteAccess-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```

| Category | Artifacts |
|---|---|
| `Accounts` | SAM local accounts, LSA secrets' names and times |
| `Applications` | Application history (third-party tools' traces) |
| `Applications` | Installed programs |
| `ExternalDevices` | USB devices and volumes |
| `FileAndFolderOpening` | Jump Lists, LNK shortcuts, Microsoft Office, MRU lists, Open and Save As dialogs, RecentDocs, ShellBags |
| `FileExistence` | Amcache, ShimCache |
| `FileSystem` | `$Boot`, `$I30`, `$LogFile`, `$MFT`, `$Secure:$SDS`, USN Journal |
| `NetworkActivity` | Network profiles, interfaces, shares, port proxies and routes; SRUM connections, traffic and push notifications |
| `LateralMovement` | Users' outbound Remote Desktop, PuTTY and WinSCP connections, mapped drives and shares |
| `Persistence` | Boot configuration (BCD), registry autostarts, services, TaskCache |
| `ProcessExecution` | Application compatibility, BAM and DAM, FeatureUsage and pins, MuiCache, Prefetch, Prefetch timeline, SRUM application/CPU activity, UserAssist |
| `Registry` | General hive keys and values, including recovered records |
| `SecurityProducts` | Defender quarantine metadata and optional extracted content; security settings (Defender, UAC, credential protection, RDP, firewall, logging) |
| `SystemInformation` | Names, Windows version, time zone, last shutdown, profiles, SIDs and audit policy; SRUM energy/capacity, identity maps and unknown providers |

Categories help organize investigations; they are not proof of a particular activity. For example,
Amcache and ShimCache entries alone do not establish execution. SRUM records resource usage;
consult the artifact guide before drawing conclusions about a program running.

CSV, JSON, and JSONL use the same categories. One `<artifact>-logs.txt` file per parser job contains its
exact command and messages; for example, `HOST1/FileSystem/MFT-logs.txt` and
`HOST1/ProcessExecution/Prefetch-logs.txt`. JSON and JSONL use the same log filenames.
Amcache and Jump Lists each have one shared log beside their prefixed tables. SRUM provider tables are placed directly
in `NetworkActivity`, `ProcessExecution` and `SystemInformation`, with a shared
`SystemInformation/SRUM-logs.txt` file. No `logs/` subdirectories are created.

Default CSV filenames are `Registry-SAM.csv`, `LNK.csv`, `Registry-RecentDocs.csv`, `Registry-ShimCache.csv`, `LogFile.csv`,
`I30.csv`, `MFT.csv`, `UsnJournal.csv`, `Boot.csv`, `SecureSDS.csv`, `Registry-Networks.csv`, `Registry-Persistence.csv`, `Registry-Services.csv`, `Registry-TaskCache.csv`, `Registry-BCD.csv`,
`Registry-SecuritySettings.csv`,
`PrefetchTimeline.csv`, `Prefetch.csv`, `Registry-UserAssist.csv`, `Registry-AppCompat.csv`, `Registry-Office.csv`, `Registry-FeatureUsage.csv`, `Registry-AppHistory.csv`, `Registry-MRU.csv`, `Registry-SystemInfo.csv`, `Registry-RemoteAccess.csv`, `Registry-OpenSaveMRU.csv`, `Registry-InstalledPrograms.csv`, `Registry-BAM.csv`, `Registry-MuiCache.csv`, `Registry-ShellBags.csv`, `Registry-USB.csv`, `Registry-SecuritySettings.csv`, `Registry-AuditPolicy.csv`, `Registry-Dump.csv`, and
`DefenderQuarantine.csv`. Registry-derived tables use the `Registry-` prefix, including
`Registry-ShimCache.csv`, `Registry-UserAssist.csv`, `Registry-BAM.csv` and `Registry-SystemInfo.csv`.
General hive keys and values are in `Registry-Dump.csv`. Amcache tables are
`FileExistence/Registry-Amcache-Devices.csv`, `Registry-Amcache-Files.csv`,
`Registry-Amcache-Findings.csv` and `Registry-Amcache-Summary.csv`. Jump Lists use
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` and
`JumpLists-CustomDestinations.csv`. There are no Amcache or Jump Lists subdirectories.
SRUM tables use `SRUM-` filenames directly under their category, for example
`NetworkActivity/SRUM-NetworkConnections.csv`.
See the [complete SRUM placement table](../artifacts/srum.md#categorized-auto-parse-output).

Logs match the capitalized artifact name, such as `Registry-SAM-logs.txt`, `PrefetchTimeline-logs.txt`,
and `Registry-UserAssist-logs.txt`. JSON/JSONL files use the same stems, for example `MFT.jsonl`; the
record's `Artifact` identifier is `mft` for queries. Multi-table outputs have
`Registry-Amcache-logs.txt` or `JumpLists-logs.txt` beside their tables. The shared SRUM log is
in `SystemInformation`.


`manifest.json` records the original discovered `Hostname`, the unique output/record `Comp`
identity, hostname provenance, evidence paths, artifact `Category`, command lines, duration, exit
code, result path, log path, the requested JSONL timeline and any export error, whether Defender extraction was requested,
and whether each task
succeeded, was skipped, or failed.

Auto-parse writes no absolute path, so results can be shared without showing where the evidence,
the results or Juntei are on the analysis system:

- Evidence paths are relative to the evidence directory. The terminal lists each computer with its
  directory in the evidence (`HOST1: ./HOST1_F`), and the parsers run in the evidence directory and
  are given such paths. The results' source path columns (`SourceFile`, `SourcePath`, …), the logs
  and the manifests' `EvidenceRoot` and `WindowsDirectories` show where each file is in the
  evidence (`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`).
- The manifests' and logs' result paths (`Output`, `OutputFiles`, `Log`, `JSON-Timeline`, the
  output paths in `Command`) and Defender's `ExtractedFile` are relative to the output directory
  (`./HOST1/ProcessExecution/Prefetch.csv`, `../siem.jsonl` for a timeline beside it). `Command`
  starts with the program's name (`juntei`); its evidence paths are relative to the evidence
  directory and its output paths to the output directory.
- `EvidenceDirectory` and `OutputDirectory` are the two directories' names. `ManifestVersion` 2
  marks manifests that record result paths from the output directory; earlier manifests have no
  `ManifestVersion` and record them as given to `-o`. A `-C` rerun reads both.
- The terminal shows an output or timeline path as given when it is relative, and from the
  current directory when it is absolute (`../results/`).

After the number of parallel processes, the terminal shows the total size of the files of the
Windows systems found (`Total filesize: 26.1 MiB`).

The progress message shows the active computer and artifact followed by an approximate input
size, for example `HOST1 / prefetch (~2.0 MiB)`. A group of files uses their combined size; each
MFT, LogFile or USN Journal task shows its individual input size. Estimates exclude auxiliary transaction
logs and enrichment files and do not predict the size of the parsed output.

`manifest.csv` provides the same task index as a spreadsheet-friendly table, with one row per
parser task. It includes `Comp`, original `Hostname`, `FQDN`, `HostnameSource`, `EvidenceRoot`,
`Artifact`, `Category`, `OutputType`, `Purpose`, `Status`, `DurationMs`, `ExitCode`, `Output`, `OutputFiles`, `Log`,
`Error`, and `Command`. `Purpose` is `artifact` for the categorized results or `json_timeline` for
the additional native JSONL pass used with CSV. `OutputType` is the actual parser output format.
`Command` is a JSON array of the executable and its arguments, preserving spaces and quoting.
For SRUM, `Output` is the computer directory, `OutputFiles` is a JSON array of its exact
provider output paths, and `Category` names the shared log location (`SystemInformation`).
Files are categorized individually; `OutputFiles` also appears in the JSON manifest.

Filter `Status` to `skipped` or `failed` to find tasks requiring review; a skipped task's `Error`
may be empty, with the explanation in its `Log` file.

The terminal summary includes the total number and size of regular files currently beneath the
output directory, including both manifests, parser logs, and extracted files. Symbolic links are
not followed. Sizes scale automatically using the same formatter as `Total filesize`, with
units such as `KiB`, `MiB`, and `GiB`:

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

Amcache and SRUM tables use `-Source1`, `-Source2`, etc. when several source databases
were collected. The shared parser log records the original-to-final paths, and JSON/JSONL
records retain source provenance. CSV Amcache/Jump Lists tasks list their exact table paths in
manifest `OutputFiles`; their `Output` is the investigative category directory. JSON and JSONL
Amcache and Jump Lists exports are already single files named `Registry-Amcache.json[l]` and
`JumpLists.json[l]`. The combined `-j` timeline keeps the same record contents and `Artifact` values.
`--clobber` uses the preceding manifest to clear recorded outputs, including earlier names,
while preserving unrelated category files. A normal single-volume collection writes `$MFT` and USN Journal records directly to
`FileSystem/MFT.csv` and `FileSystem/UsnJournal.csv`. Additional collected volumes use numbered
sibling files such as `FileSystem/MFT-2.csv` and `FileSystem/MFT-2-logs.txt`.

## JSON and SIEM ingestion

Every JSON or JSONL record receives two top-level fields:

- `Comp`: the source system.
- `Artifact`: the Juntei parser that produced the record, such as `mft` or `prefetch-timeline`.

`-j, --json-timeline <FILE>` creates one additional JSONL file, regardless of the individual
result format. All categorized outputs and their logs are retained:

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

The filename is used as supplied; relative paths are relative to the current working directory.
It may be in the output root, another non-reserved output subdirectory, or outside the output
directory. Evidence, manifests, per-computer results/logs, and temporary parser directories cannot
be used as the destination. An existing file requires `-C`; a failed export leaves it intact.

In **CSV mode**, Juntei performs an additional native JSONL pass so the SIEM file preserves
numeric and boolean types, nested data, raw fields, and metadata that CSV tables can omit. The
extra jobs share the same `-T` concurrency limit, do not extract quarantine contents again, and
have separate `*-json-timeline-logs.txt` sidecar logs. Both manifests distinguish their purpose and
output format. Temporary JSONL files are removed after a successful run; they are retained if a
parser or export fails. Plan for the extra processing time and temporary disk space.

In **JSON mode**, Juntei streams the existing results into compact JSONL without reparsing the
evidence. Amcache reports are expanded into the same individual summary, finding, file, and device
records as native JSONL. In **JSONL mode**, the existing annotated records are combined directly.
All modes use the same SIEM record structure. The dataset keeps native timestamps and artifact
fields; it does not invent a common event timestamp or globally sort heterogeneous artifacts.

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

`manifest.json` records the requested `JSON-Timeline` path and any `JSON-TimelineError`. Its task
records, and `manifest.csv`, still point to retained artifact outputs. Successful CSV timeline-only
jobs reference the final JSONL file instead of deleted temporary files. If any parser fails, the
run exits unsuccessfully and the timeline contains only successful results; inspect the manifests
and category logs before treating it as complete. The file-count/size summary covers files inside
the output directory; an external timeline is reported separately as `JSONL timeline: <FILE>`.

## Performance

Parser jobs are separate processes, so CPU-heavy artifacts can run at the same time and damage in
one parser cannot terminate the complete run. By default, Juntei uses the logical CPU count. Limit
concurrency when evidence is on a slow disk or the system has little memory:

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## Usage

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -Z, --timezone <OFFSET>             Display parsed timestamps using a fixed UTC offset (UTC by default) [default: +00:00]
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK files, Jump Lists, ShellBags and OpenSaveMRU [default: 1252]
      --mft-slack           Recover the file names left in the slack of $MFT records

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Use a new output directory when practical. `-C` removes or overwrites only the exact generated
targets for this run, so a now-missing artifact cannot leave a stale result. It does not delete
unrelated files or the complete output tree. It also removes each computer's
`SecurityProducts/DefenderQuarantined/` directory, and the reserved `defender-quarantined/`
directory from the previous flat layout, before parsing, whether or not the new run uses `-x`.

Use a fresh output directory when switching from an older flat layout. Earlier CSV/JSON files
are not relocated or deleted automatically.

!!! danger "Treat extracted Defender content as malware"
    `-x, --extract-defender-quarantined` writes quarantined content to disk under each computer's
    `SecurityProducts/DefenderQuarantined/` directory. Recovered binary streams end in `.quarantined`, and Juntei
    removes execute permission. Each binary security descriptor also gets a readable `.security_descriptor.txt`
    report with ownership, permissions and SDDL. Never execute recovered payloads. Use an isolated
    analysis system and expect antivirus or EDR software to alert on or remove the recovered content.

See [reading the recovered security descriptor](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)
for the report fields and their forensic limitations.

Auto-parse also reads users' remote connections (RDP, PuTTY, WinSCP and mapped drives).
