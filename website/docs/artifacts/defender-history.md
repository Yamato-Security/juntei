# Microsoft Defender Detection History

## What the history records

Microsoft Defender writes one detection history file for each detection. Each file records:

- the threat's identifier, name, severity and category;
- what Defender did and whether it succeeded;
- whether the threat was running;
- the user and process involved, and what detected it;
- the quarantine entry, if there is one;
- every resource it was found in: files, archive members, downloads, processes, registry values,
  services, command lines and behaviors.

Defender also writes these files for threats it never quarantined: those it only detected, those
that were allowed, and those it failed to remove. Windows Security's protection history and WMI's
`MSFT_MpThreatDetection` class (`Get-MpThreatDetection`) show these records. When Orange
Cyberdefense changed values in a file, WMI reported the changed values.

<span class="evidence-strength">Best conclusion:</span> Defender recorded a detection of the named
threat in the listed resources, with the recorded status and times. The threat name is the
product's verdict. A detection record alone does not show that the threat ran, or that it was
removed completely.

## Location and collection

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

Each file is named after its `DetectionID`. Collect the whole `DetectionHistory` folder. Directory
input and `auto-parse` read every file inside a `DetectionHistory` folder, at any depth, that starts
like a detection history file. Use `-f` for a copy kept outside such a folder. Defender removes
items from its scan history after a configurable number of days (`ScanPurgeItemsAfterDelay` in
`Set-MpPreference`), so older detections may be gone.

The quarantine is a separate artifact: see [Defender Quarantine](defender-quarantine.md). A
detection's `QuarantineID` uses the same spelling as the quarantine output's `EntryID` (upper case,
without braces), so the two can be joined.

## Analyst cautions

- Detection, a remediation attempt and successful removal are separate states. Read
  `ThreatStatus` (`Detected`, `Quarantined`, `Removed`, `Allowed`, `RemoveFailed`, …) together with
  `ThreatStatusErrorCode`, which is 0 when the last status change succeeded.
- `ExecutionStatus` is Defender's view at detection time (`Blocked`, `Allowed`, `Executing`,
  `NotExecuting`, `Unknown`). It is not proof that the threat ran, nor that it never ran.
  Corroborate with Prefetch, Amcache, event logs and process telemetry.
- `ThreatName` is the signature name at that time. False positives and later reclassification are
  possible. Test files such as EICAR are reported like real threats.
- `User` is the `DomainUser` value Defender recorded (`UserSource` is `DomainUser`). It can be
  empty, a service account, or a different user from the one who created the file.
- Defender may write several files for one threat, and Juntei keeps each of them. Two files can
  even hold the same `DetectionID`, as a copied or restored file does. Group by `DetectionID` and
  `SourceFile` together.
- Values whose meaning is not known are kept, not interpreted: their section and position are in
  `UnknownValues`.

## Parse with Juntei

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

CSV is the default file format. With `-t csv`, `-t json` or `-t jsonl` and no `-o`, records go to
stdout. Without either option, only a summary is displayed. Add `-C` to overwrite an existing
output. The output is never written over one of the input files, even with `-C`.

Juntei writes one record per resource of each detection. A detection without resources gets one
record with empty resource fields. The records of one detection share its `DetectionID` and are
numbered by `ResourceIndex` in the order stored.

Damage does not stop the run. A file cut short keeps the values read before the damage, and
`Problems` says what is missing. A file in a `DetectionHistory` folder that is not a detection
history file is reported and skipped. The other files are still read.

## Output fields

| Field | Interpretation |
|---|---|
| `Record` | `resource`, or `detection` for a detection without resources. |
| `InitialDetectionTime` | When Defender first detected the threat (UTC). Empty when the file holds 0; a value that is not a time is kept in `UnknownValues`. |
| `LastThreatStatusChangeTime` | When the threat's status last changed (UTC). |
| `RemediationTime` | When Defender remediated the threat (UTC). It is a time of an attempt, not proof of removal; check `ThreatStatus`. |
| `ThreatName`, `ThreatID` | Defender's signature name and numeric identifier (`Get-MpThreatCatalog`). |
| `Severity`, `SeverityID` | `Low`, `Moderate`, `High`, `Severe`, or `Unknown`, with the stored number. |
| `Category`, `CategoryID` | Threat category such as `TROJAN`, `TOOL` or `BEHAVIOR`, with the stored number. |
| `ThreatStatus`, `ThreatStatusID` | Defender's status for the threat. A number without a documented name is kept with an empty `ThreatStatus`. |
| `ThreatStatusErrorCode` | Error code of the last status change; 0 means it succeeded. |
| `ExecutionStatus`, `ExecutionStatusID` | Whether Defender saw the threat running when it was detected. |
| `DetectionSource`, `DetectionSourceTypeID` | What detected it: `Real-time`, `IOAV` (downloads and attachments), `User`, `System`, `AMSI`, and others. |
| `User`, `UserSource` | The `DomainUser` value Defender recorded, and its basis (`DomainUser`). Empty when the file names no user. |
| `Process` | The process involved (`ProcessName`); can be `Unknown`. |
| `ResourceIndex`, `ResourceCount` | This resource's position, and the number of resources in the detection. |
| `ResourceType` | `file`, `containerfile`, `webfile`, `process`, `regkeyvalue`, `service`, `CmdLine`, `behavior`, `amsi`, and others, as Defender names them. |
| `Path` | The resource: a path, an archive member (`archive.zip->member`), a registry value, a command line or a process. A `webfile` also names its URL and the downloading process (`pid:…,ProcessStart:…`). Preserve the exact spelling. |
| `ResourceFlags` | Resource flags in hexadecimal; their meaning is not documented. |
| `TrackingStartTime` | The resource's `ThreatTrackingStartTime`, an integer read as a FILETIME (UTC), as plaso dates a detection. |
| `SHA256`, `SHA1`, `MD5`, `TrackingSize` | Hashes and size from the resource's tracking data. They describe the file when Defender scanned it. |
| `TrackingID` | The resource's `ThreatTrackingId`. |
| `OtherTracking` | Every other tracking value, as `Key=Value` in stored order, such as `ThreatTrackingScanSource` or `ThreatTrackingBMContextRich` for behavior detections. |
| `QuarantineID` | The quarantine entry (`Quarantine\Entries\{GUID}`); empty when the file holds a zero GUID. |
| `DetectionID` | The detection's GUID; also the file's name. A mismatch is reported in `Problems`. |
| `MagicVersion` | The header's `Magic.Version:1.2`; empty in files written without it, such as some from Windows Server 2016. |
| `UnknownValues` | Values whose meaning is not known, as `position=value` (for example `6.13=NT AUTHORITY\SYSTEM`), with the counted list after the status as `2.list`. |
| `SourceFile` | The file read, relative to the input directory, or its name with `-f`. |
| `Problems` | Damage, missing values, unexpected types, or a file name that does not match. Do not suppress. |

JSON and JSONL keep the same fields with typed values: numbers stay numbers and missing values are
`null`. Instead of `OtherTracking` they have `Tracking`, every tracking value with Defender's key,
its type and its value. `UnknownValues` becomes a list of positions with types. `CountedList`
holds the counted list. A stored FILETIME keeps its raw tick count beside the time it reads as.

## Timeline behavior

`auto-parse` saves the records as `SecurityProducts/DefenderHistory.csv` (or `.json`/`.jsonl`) and
adds them to the combined `-j` timeline. `super-timeline` makes events from
`InitialDetectionTime`, `LastThreatStatusChangeTime` and `RemediationTime` on each resource's
record, with the status, error code, execution status and detection source in the event's
details. `TrackingStartTime` is stored as an integer, so it becomes an event only with
`--include-interpreted-times`. The file's own times are never used. A record with no usable time
stays in the parsed output and is counted as untimed.

## Investigation strategy

1. Start from `ThreatName`, `Path`, `SHA256`, `InitialDetectionTime` and `ThreatStatus`.
2. Check `ThreatStatusErrorCode` and `Problems` before concluding that a threat was removed.
3. Join `QuarantineID` with the quarantine's `EntryID` to recover the quarantined bytes.
4. Pivot hashes and paths into `$MFT`, USN Journal, Prefetch, Amcache, browser history and event
   logs (Defender's operational log: event IDs 1116, 1117, 1118 and 1119).
5. For `webfile` resources, use the URL and process start to investigate initial access, and
   validate them with browser, proxy and DNS evidence.

## Tested versions

The tests use public files with results that other tools read independently:

- Orange Cyberdefense's Windows 10 sample (Defender 4.18.24090.11), checked against what WMI
  reported.
- plaso's two Windows test files, checked against plaso's `windefender_history` parser.
- 14 files from DFIRArtifactMuseum's APT simulator VM.
- A copy written without the header's version, as some files from Windows Server 2016 are.
- Files cut short, empty, or of another kind.

## Further Details

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [Orange Cyberdefense wddh-parser](https://github.com/cert-orangecyberdefense/wddh-parser)
- [plaso Defender history parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Velociraptor Windows.Applications.DefenderHistory](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft: MSFT_MpThreatDetection class](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft: Microsoft Defender Antivirus event IDs](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
