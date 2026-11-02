# Persistence Hunter

Persistence lets a program run again after a reboot, a sign-in, a scheduled trigger, or another
event. An attacker can use normal Windows configuration for this purpose. Many legitimate programs
use exactly the same mechanisms, so finding a persistence entry is a starting point for analysis.

`juntei persistence-hunter` is a native Rust port of
[Hexastrike's PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper), with 132 checks: the
130-check PyrsistenceSniper catalogue plus native Start Menu COM and historical-service checks.

For an explanation of every mechanism, its evidence locations, and how to investigate a finding,
see the [132-check persistence guide](../persistence-hunter/index.md). The
[persistence acknowledgements](../persistence-hunter/acknowledgements.md) describe the Python and
PowerShell projects behind the catalogue.

## Usage

```text
Version: 1.0.0
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei persistence-hunter -d <EVIDENCE> [OPTIONS]

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
juntei persistence-hunter -d evidence/ -o PersistenceHunter.csv --coverage Coverage.csv
```

The evidence can be a mounted Windows volume, a Velociraptor collection, or a directory containing
several collected computers. Computers are kept separate during analysis, and every result includes
`Computer`. Collected filename escaping is decoded when resolving Windows paths.

For one hive:

```bash
juntei persistence-hunter -f SOFTWARE -o software-persistence.csv --no-timeline
```

A single hive cannot supply checks that need another hive or filesystem evidence. Read the coverage
report before treating an empty result as meaningful.

List the check IDs and their ATT&CK mappings:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,windows_service_image_path -o focused.csv
```

Use the check IDs printed by `--list-checks`, or an ATT&CK ID such as `T1547.001`.
A parent ID such as `T1547` selects its sub-techniques. `--technique` is an alias for `--check`.
Each check includes its technique, description, and reference links in the findings.

## Parallel scans

The command uses a shared worker pool, defaulting to the number of logical CPUs. Set
`-T, --threads <N>` to choose the worker count, or `-T 1` for a single-thread scan:

```bash
juntei persistence-hunter -d evidence/ --min-severity info -T 4 -o PersistenceHunter.csv
```

Evidence discovery, independent checks, payload metadata, and MFT/EVTX timeline sources run in parallel. A host's
loaded registry hives are shared read-only, and cached metadata for the same payload is initialized once.
Hosts are loaded one at a time to avoid multiplying hive memory across the collection. Findings,
coverage, timestamp selection, and warning order remain the same at different thread counts.

`auto-parse` already runs parser processes in parallel, so it gives each `persistence-hunter` child
one worker. Its own `-T` option controls the number of concurrent parser processes.

## Evidence to collect

| Source | Examples of checks |
|---|---|
| `SYSTEM` and adjacent transaction logs | Services, service recovery commands, LSA packages, boot settings, authentication providers |
| `SOFTWARE` and adjacent transaction logs | Run keys, Winlogon, IFEO, COM registrations, TaskCache, Office settings |
| Each user's `NTUSER.DAT` and `UsrClass.dat` | User autostarts, environment variables, COM overrides, Office settings |
| `SAM` and `SOFTWARE` | RID anomalies, hidden accounts, unusual local Administrators membership |
| `Windows/System32/Tasks` | Scheduled-task commands and COM handlers; comparison with TaskCache |
| Per-user Start Menu `.library-ms` files and CLSID-named folders | COM activation-trigger candidates, joined to matching user/machine class registrations |
| Startup folders, PowerShell profiles, Office templates, Terminal settings | Files and commands run by user or system triggers |
| WMI `OBJECTS.DATA` | Event consumer instances and filter-to-consumer bindings |
| BITS `qmgr.db` or `qmgr*.dat` | Recovered notification commands and owner SIDs |
| `$MFT` and relevant EVTX files | Independent timestamp correlation |
| Referenced executables, DLLs, and catalog files | SHA256 hashes and descriptive PE/certificate metadata |

Registry transaction logs are replayed when available. `--no-logs` reads the hives as collected;
an unreplayed dirty hive may omit recent changes and is reported in coverage. The current control
set is used for active-system checks. The DSRM backdoor check additionally examines
`ControlSet001`, `ControlSet002`, and `CurrentControlSet` when present; check its reported path
against `SYSTEM\\Select` before describing that setting as active. Other historical control sets
can be examined with the individual registry parser commands.

Hive reads share a 1 GiB per-computer input budget. Machine hives are attempted before user hives;
inputs or transaction logs that do not fit are named in partial coverage. Other readable hives and
file-backed checks continue. Only canonical hive locations or explicitly supplied exports establish
hive identity; a copy inside a user's temporary directory cannot replace that user's real hive.
The original system drive recorded in SOFTWARE is mapped to the captured system volume, so mounting
an original `C:` volume under a different evidence drive label does not discard payload metadata.

`auto-parse` also runs the hunter and saves `persistence/PersistenceHunter.csv` (or `.json`/`.jsonl`)
inside each computer directory, with `PersistenceHunter-logs.txt` beside it. Its findings also enter
the optional `-j <FILE>` SIEM timeline. Automatic parsing uses `--min-severity info` to retain the
complete configuration inventory, including entries matched by allow rules.

## COM registrations and activation triggers

The `scheduled_task_files` check selects each action's principal using the task XML's
`Actions Context` and matching `Principal id`. A group principal is retained as a group;
it cannot establish which individual user's class registrations would be used.

COM-handler candidates are resolved separately in native and 32-bit class views. Only the
matching user's captured `UsrClass.dat`/`NTUSER.DAT` and machine `SOFTWARE` are considered.
SID principals are joined through captured `ProfileList` paths. Account names, including
`HOST\user` and `DOMAIN\user`, prefer a matching `user.DOMAIN` profile over a plain
`user` profile. Local host aliases and `.\user` use captured host names for suffix matching;
an exact suffix takes precedence over other host aliases. The plain profile is considered
only when no qualified profile matches. Every such match is explicitly labeled a
**folder-name heuristic** in `Description`; it does not prove account ownership or domain membership. Ambiguous matches
remain unresolved and are reported in both the finding description and coverage; ambiguity
among qualified profiles does not trigger a fallback to the plain profile. This does not depend
on `Volatile Environment`, which is normally memory-only and is not saved with registry hives. A duplicate user CLSID
shadows the machine CLSID's children. Group principals use machine COM candidates and retain
that limitation in the finding description; a normal group principal does not itself make
coverage partial or consume the error-summary entries.

The resolver follows `InprocServer32` and `LocalServer32`, uses managed `CodeBase` paths
(including version subkeys), and joins AppID `LocalService` to the collected SYSTEM service
`ImagePath`. AppID executable-name associations are retained, but a basename does not establish
an executable's full path. Local `file:` CodeBase URLs are decoded; remote URLs are recorded
without accessing them. Missing registrations retain the CLSID with
`ResolutionStatus: unresolved_com_class`, empty path/hash fields, and unknown `Exists`.

The `start_menu_com` check reads per-user Start Menu `.library-ms` XML URLs and collects
CLSID-named folders, including empty directories. It reports the trigger path, CLSID, candidate
server, and registry provenance. A file-only acquisition cannot preserve an empty directory,
and missing user hives prevent a complete per-user registration lookup. Symbolic links are
not followed. These are configuration candidates: architecture, elevation, UAC, policy and
activation context can affect whether a registration is actually used. Corroborate execution
with process, module-load or other independent evidence.

## Classification and profiles

The default minimum classification is `medium`. To inspect the full inventory, including
entries matched by the bundled allowlist:

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o all-persistence.csv
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

Captured ComputerName/ActiveComputerName, TCP/IP Hostname/NV Hostname, and client-info hostname/FQDN
supply host aliases, covering DNS/NetBIOS differences and pending renames. With no captured identity,
a single name in the primary logs can support dating with explicit unverified-identity provenance
and a coverage warning. Multiple unverified names are not attributed to the host. Rejected Computer
names are reported as a correlation coverage limitation; their channels are still marked collected.

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

## Recover historical services

```bash
juntei persistence-hunter -d evidence/ --check deleted_services --min-severity info -o HistoricalServices.csv --coverage Coverage.csv
```

The check preserves separate recovered service states and their provenance, including deleted
registry cells and transaction-log versions, including independent `Parameters\ServiceDll` payload
rows. Missing fields remain unknown. Key-only origins add provenance without duplicate empty states. Historical records
default to informational severity; they do not establish active persistence or execution. Current
files at old paths are not treated as historical payload bytes. Matching local System 7045 events
can provide independent installation references using the key name or same-state display name; `--no-timeline` disables this optional correlation.
See the [historical-service guide](../persistence-hunter/index.md#deleted_services) for limits and interpretation.

## Further details

- [PyrsistenceSniper implementation and profiles](https://github.com/hexastrike/PyrsistenceSniper)
- [MITRE ATT&CK persistence techniques](https://attack.mitre.org/tactics/TA0003/)
- [Microsoft: Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
- [Registry autostart inventory](../artifacts/persistence.md)
- [Services](../artifacts/services.md) and [TaskCache](../artifacts/taskcache.md)
