# Check Persistence

Persistence lets a program run again after a reboot, a sign-in, a scheduled trigger, or another
event. An attacker can use normal Windows configuration for this purpose. Many legitimate programs
use exactly the same mechanisms, so finding a persistence entry is a starting point for analysis.

`juntei check-persistence` is a native Rust port of
[Hexastrike's PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper), using the
130-check catalogue from upstream commit `b99c117a3cd16c33992a56dd698aedaec024460f`.
It reads evidence offline. It does not start services, register tasks, load DLLs, or execute
the programs it finds. It needs no Python interpreter or Windows runtime.

## Usage

```text
Version: 1.0.0
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei check-persistence -d <EVIDENCE> [OPTIONS]

Input:
  -d, --directory <DIR>  Evidence directory (a mounted Windows volume or a collection of hosts)
  -f, --file <FILE>      One registry hive (filesystem checks need a directory)
      --no-logs          Read dirty hives without replaying transaction logs

Filtering:
      --profile <FILE>               Replace the bundled allow/block profile with a YAML profile
      --min-severity <MIN_SEVERITY>  Minimum classification to include: info, low, medium, high [default: medium] [possible values: info, low, medium, high]
      --check <ID>                   Run only these check or ATT&CK IDs (comma separated; repeatable) [alias: --technique]

General Options:
  -h, --help         Show the help menu
      --list-checks  Show every available check and its ATT&CK technique

Analysis:
      --no-timeline  Skip MFT and event-log timestamp correlation
      --mft <FILE>   MFT to use for timestamp correlation (one host only)

Performance:
  -T, --threads <N>  Worker threads (default: logical CPU count)

Output:
  -o, --output <FILE>              Save findings to a file
  -t, --output-type <OUTPUT_TYPE>  Output type: csv (default), json, jsonl; without -o, write data to stdout [possible values: csv, json, jsonl]
      --coverage <FILE>            Save per-check coverage and errors to CSV
  -C, --clobber                    Overwrite output files

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## First scan

```bash
juntei check-persistence -d evidence/ -o PersistenceSniper.csv --coverage Coverage.csv
```

The evidence can be a mounted Windows volume, a Velociraptor collection, or a directory containing
several collected computers. Computers are kept separate during analysis, and every result includes
`Computer`. Collected filename escaping is decoded when resolving Windows paths.

For one hive:

```bash
juntei check-persistence -f SOFTWARE -o software-persistence.csv --no-timeline
```

A single hive cannot supply checks that need another hive or filesystem evidence. Read the coverage
report before treating an empty result as meaningful.

List the check IDs and their ATT&CK mappings:

```bash
juntei check-persistence --list-checks
juntei check-persistence -d evidence/ --check run_keys,windows_service_image_path -o focused.csv
```

Use the check IDs printed by `--list-checks`, or an ATT&CK ID such as `T1547.001`.
A parent ID such as `T1547` selects its sub-techniques. `--technique` is an alias for `--check`.
Each check includes its technique, description, and reference links in the findings.

## Parallel scans

The command uses a shared worker pool, defaulting to the number of logical CPUs. Set
`-T, --threads <N>` to choose the worker count, or `-T 1` for a single-thread scan:

```bash
juntei check-persistence -d evidence/ --min-severity info -T 4 -o PersistenceSniper.csv
```

Evidence discovery, independent checks, payload metadata, and MFT/EVTX timeline sources run in parallel. A host's
loaded registry hives are shared read-only, and cached metadata for the same payload is initialized once.
Hosts are loaded one at a time to avoid multiplying hive memory across the collection. Findings,
coverage, timestamp selection, and warning order remain the same at different thread counts.

`auto-parse` already runs parser processes in parallel, so it gives each `check-persistence` child
one worker. Its own `-T` option controls the number of concurrent parser processes.

## Evidence to collect

| Source | Examples of checks |
|---|---|
| `SYSTEM` and adjacent transaction logs | Services, service recovery commands, LSA packages, boot settings, authentication providers |
| `SOFTWARE` and adjacent transaction logs | Run keys, Winlogon, IFEO, COM registrations, TaskCache, Office settings |
| Each user's `NTUSER.DAT` and `UsrClass.dat` | User autostarts, environment variables, COM overrides, Office settings |
| `SAM` and `SOFTWARE` | RID anomalies, hidden accounts, unusual local Administrators membership |
| `Windows/System32/Tasks` | Scheduled-task commands and COM handlers; comparison with TaskCache |
| Startup folders, PowerShell profiles, Office templates, Terminal settings | Files and commands run by user or system triggers |
| WMI `OBJECTS.DATA` | Event consumer instances and filter-to-consumer bindings |
| BITS `qmgr.db` or `qmgr*.dat` | Recovered notification commands and owner SIDs |
| `$MFT` and relevant EVTX files | Independent timestamp correlation |
| Referenced executables, DLLs, and catalog files | SHA256 hashes and descriptive PE/certificate metadata |

Registry transaction logs are replayed when available. `--no-logs` reads the hives as collected;
an unreplayed dirty hive may omit recent changes and is reported in coverage. The current control
set is used for active-system checks. Old control sets are available through the individual registry
parser commands but are not silently mixed into the active persistence hunt.

`auto-parse` also runs the hunter and saves `persistence/PersistenceSniper.csv` (or `.json`/`.jsonl`)
inside each computer directory, with `PersistenceSniper-logs.txt` beside it. Its findings also enter
the optional `-j <FILE>` SIEM timeline. Automatic parsing uses `--min-severity info` to retain the
complete configuration inventory, including entries matched by allow rules.

## Classification and profiles

The default minimum classification is `medium`. To inspect the full inventory, including
entries matched by the bundled allowlist:

```bash
juntei check-persistence -d evidence/ --min-severity info -o all-persistence.csv
```

!!! warning "Use all severities when reviewing incomplete collections"
    A broad allow rule can partially match an attacker's command even when its signer is missing
    or different. Upstream classifies partial matches as `LOW`, which the manual command's default
    `medium` threshold hides. This can affect malicious scheduled tasks under Windows directories.
    Use `--min-severity info` for a complete investigation and review `PolicyReason`, `SignerStatus`,
    the command, and source evidence together. `auto-parse` already retains these entries.

| Classification | Meaning |
|---|---|
| `HIGH` | A configured block rule fully matched |
| `MEDIUM` | No allow or block rule matched |
| `LOW` | An allow rule's hard conditions matched, but signer metadata did not match or was unavailable |
| `INFO` | An allow rule fully matched |

These are policy classifications, not measured probabilities of compromise. A default `MEDIUM`
entry is not proof of malware. `INFO` does not establish that a file has a valid signature or that
an attacker did not abuse it. A partial allow match is below the default threshold, so include
`--min-severity low` or `info` when reviewing sparse collections or tuning the allowlist.

Supply a YAML profile with `--profile profile.yaml`. It **replaces** the bundled profile. Global
and per-check rules within the selected file are combined; any fully matched block rule wins.
For example:

```yaml
allow: []
block:
  - reason: Example known-bad command fragment
    value_matches: 'example-malicious-loader\.exe'
checks:
  windows_service_image_path:
    enabled: true
    allow:
      - reason: Organization's expected service path
        value_matches: '^C:\\Program Files\\ExampleCorp\\Agent\\agent\.exe$'
        signer: ExampleCorp
```

Supported rule conditions are `path_matches`, `value_matches`, `hash` (64 hexadecimal SHA256
digits), `not_lolbin`, and `signer` (case-insensitive substring). All stated hard conditions must
match. Signer metadata is the soft condition described above. Regexes use Rust's regular-expression
syntax: lookarounds and backreferences are not supported. Unknown check IDs, invalid expressions,
and unknown profile fields produce an error rather than silently weakening the policy.

## Output fields

CSV is the default file format. Use `-t json` for readable JSON objects or `-t jsonl` for compact,
one-object-per-line output. Without `-o`, an explicit `-t` writes only data to stdout; coverage
warnings go to stderr. Without either option, Juntei prints a summary and up to 30 candidates.

| Fields | How to interpret them |
|---|---|
| `Computer`, `CheckId`, `Technique`, `MitreId` | Which computer and persistence mechanism the row describes |
| `Path`, `Value` | The registry location, file, task, or recovered record and its configured content |
| `Description`, `References` | Why the mechanism matters and where to read more |
| `AccessGained` | The mechanism's usual scope (`USER` or `SYSTEM`; BITS can also identify `LOCAL_SERVICE`/`NETWORK_SERVICE`); not proof that a specific command achieved that privilege |
| `Severity`, `PolicyReason` | Profile classification, matching policy explanation, and any partial-match or unverified-signer qualification |
| `SourceFile`, `RecordStatus` | Exact collected source and allocation/replay/recovery context; follow this back to original evidence |
| `KeyLastWrite` | The registry **key's** timestamp. It does not date an individual value |
| `LastChange`, `ChangeSource`, `ChangeEvidence` | Independently correlated change time, source proof, and whether correlation succeeded |
| `ResolvedPath`, `ResolutionStatus`, `Exists` | Payload path and offline resolution result; uncollected evidence is unknown, not `false` |
| `Sha256` | SHA256 of the collected payload bytes, when readable |
| `Signer`, `SignerStatus` | Descriptive embedded certificate/catalog metadata and its availability; certificate trust and signature validity are not verified |
| `IsLolbin`, `Launcher` | Whether the configured launcher or payload is in the bundled LOLBAS list, and the launcher used for a proxied payload |
| `IsBuiltin`, `IsInOsDirectory` | Context for common Windows commands and system directories; neither field establishes legitimacy |

A `rundll32.exe` command can name a DLL payload. Juntei preserves the launcher while resolving and
hashing the payload. A reference outside the collected computer, a missing drive, an ambiguous
case-insensitive match, or an unresolved environment variable is not substituted with a file from
the analyst's machine.

## Timestamps and limitations

The hunter does not use the analyst VM's copied-file modification times as attack timestamps.
Where available, it correlates registry writes with Sysmon event 13, task installation with
Security 4698 or Task Scheduler 106, service installation with System 7045, and file-backed
configuration with original `$MFT` timestamps. The provenance appears in `ChangeSource`.
An event can date a configuration change without proving that the configured payload ran.
Local System event 6011 can prove an earlier computer name. The hunter follows verified rename
chains anchored to the collected computer, preserving its current `Computer` value while recording
the original event computer and rename-record proof in `ChangeSource`. Unproven foreign names,
archived logs, forwarded logs, and another Windows installation are not accepted through this rule.

Use `--mft <FILE>` to select an externally collected MFT for a single computer. Use `--no-timeline`
to skip timestamp correlation. `NOT_RUN`, `NOT_APPLICABLE`, `NO_ARTIFACT`, `NO_MATCH`, and `REJECTED`
explain an empty `LastChange`; an empty field is not evidence that the entry is harmless.

WMI and BITS recovery is bounded carving rather than a complete database transaction parser.
Recovered strings or bindings can include old/deleted data; these records must be corroborated.
WMI class schemas alone are not counted as subscriptions. A TaskCache entry without a collected
task XML file is a discrepancy for review; a triage collection may simply have omitted that file.
Catalog and PE metadata are unverified and may be absent from a collection. Accessibility-file
replacement checks depend on suitable reference signature metadata; missing catalog evidence
must not cause every accessibility executable to be flagged.

Coverage is recorded for every check, including checks with zero findings. `complete` means the
applicable collected source was read; it does not prove that the collection is complete or that
the computer has no other persistence. `partial`, `not_collected`, `failed`, and `disabled` explain
the limits. Keep `--coverage Coverage.csv` and the parser log with your findings.

The port uses Juntei's CSV/JSON/JSONL formats. It does not call the upstream Python program, query
online enrichment services, or reproduce the upstream HTML/XLSX UI.

## Further details

- [PyrsistenceSniper implementation and profiles](https://github.com/hexastrike/PyrsistenceSniper)
- [MITRE ATT&CK persistence techniques](https://attack.mitre.org/tactics/TA0003/)
- [Microsoft: Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
- [Registry autostart inventory](../artifacts/persistence.md)
- [Services](../artifacts/services.md) and [TaskCache](../artifacts/taskcache.md)
