# Prefetch

## What Prefetch is

Windows Prefetch files record information used to optimize application startup. A program-specific
`.pf` file typically stores the executable name and hash, run count, recent run times, volume
information, directories, and files referenced during startup.

For supported Windows versions, a valid application Prefetch record is usually strong evidence
that the executable ran. It is one of the most useful non-event-log execution artifacts.

<span class="evidence-strength">Best conclusion:</span> the named executable was launched at the
recorded recent run time(s), subject to operating-system behavior and parser validity.

## Location and collection

```text
C:\Windows\Prefetch\*.pf
```

Juntei supports format versions 17 through 31 (Windows XP through Windows 11), including compressed
Windows 8+ files. Collect the whole directory so executable-name hashes and neighboring records can
be compared.

## Analyst cautions

- Windows stores only one historical run time on older versions and up to eight on newer versions.
  `RunCount` can exceed the number of available timestamps.
- Prefetch can be disabled, cleaned, excluded, or unavailable on some systems/roles. Absence is not
  proof a program did not run.
- Loaded-file lists describe startup dependencies and accesses; they do not prove every listed file
  was malicious, deliberately opened, or fully read.
- The executable name in a Prefetch header is length-limited; Juntei resolves a path from loaded
  files when uniquely possible.
- A `.pf` source file's file-system timestamps are separate from the embedded run times.
- `Findings` are hunting observations. Installers and legitimate administration often use staging paths.

## Parse with Juntei

```text
Usage: juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pf files
  -f, --file <FILE>      Parse one Prefetch file

Records:
  -T, --timeline  Write one row per retained execution time, sorted by time

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file prefetch -d Prefetch/ -o prefetch.csv
juntei parse-file prefetch -d Prefetch/ -T -o prefetch-timeline.csv
juntei parse-file prefetch -d Prefetch/ -t jsonl \
  | jq 'select(.executable_name? | test("powershell|rundll32"; "i"))'
```

## Standard CSV fields

| Field | Interpretation |
|---|---|
| `SourceFile` | Input `.pf`; retain it to distinguish hosts and duplicate executable names. |
| `ExecutableName` | Header executable name. Case-insensitive for most hunting; preserve original spelling. |
| `ExecutablePath` | Juntei's resolved loaded path when possible. Confirm it against `Files` and other artifacts. |
| `PrefetchHash` | Hash Windows uses in the `.pf` file name; it is not a cryptographic file-content hash. |
| `FormatVersion` | Prefetch structure version; determines available fields and timestamp count. |
| `RunCount` | Cumulative counter recorded by Windows. Useful for frequency, not exact history. |
| `LastRun` | Most recently retained execution time. |
| `PreviousRun1` … `PreviousRun7` | Earlier retained execution times, newest to older. Empty where the format/history lacks them. |
| `VolumeCount` | Number of recorded source volumes. |
| `VolumeDevicePaths` | NT device paths for volumes used during startup. Correlate with serials. |
| `VolumeSerialNumbers` | Volume identifiers; useful for linking removable or reformatted drive evidence. |
| `VolumeCreated` | Recorded volume creation times, not executable creation times. |
| `DirectoryCount` | Number of directory strings recorded. |
| `FileCount` | Number of loaded-file references recorded. |
| `Findings` | Juntei observations such as relocated system binaries or suspicious execution paths. Review manually. |
| `Files` | Loaded/referenced files with available NTFS references. Excellent DLL/path pivot, not proof of file intent. |
| `Directories` | Directory strings associated with startup. Useful when individual loaded-file paths are incomplete. |

## Timeline CSV fields (`-T`)

| Field | Interpretation |
|---|---|
| `RunTime` | One retained Prefetch execution time. |
| `ExecutableName`, `ExecutablePath` | Program identity/path context. |
| `PrefetchHash` | Separates variants/paths that share an executable name. |
| `RunCount` | Counter from the Prefetch file; it is repeated on each timeline row. |
| `SourceFile` | Source `.pf` for validation. |

JSON/JSONL preserves volumes, directories, loaded files, and NTFS file references as structured
arrays instead of comma-joined CSV cells.

## Investigation strategy

1. Use `-T` to add retained runs to a timeline.
2. Group on executable name plus Prefetch hash; the same name can have different paths/hashes.
3. Review `ExecutablePath`, staging paths, relocated system-binary findings, and loaded DLLs.
4. Match target MFT references and file paths to `$MFT`/USN records.
5. Correlate run times with event logs, SRUM, network telemetry, services, tasks, and user artifacts.

## Further Details

- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch)
- [libyal libscca](https://github.com/libyal/libscca)
- [Security Ronin prefetch-forensic](https://github.com/SecurityRonin/prefetch-forensic)
- [Eric Zimmerman's PECmd](https://github.com/EricZimmerman/PECmd)
