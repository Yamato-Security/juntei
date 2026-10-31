# LNK Shortcuts

## What LNK files are

Windows Shell Link (`.lnk`) files describe a target plus launch arguments, working directory, icon,
window state, target metadata, volume/network information, shell namespace items, and optional
distributed-link-tracking identifiers. Windows and applications create them in Recent Items,
Desktop, Start Menu, and other locations; attackers also deliver malicious shortcuts directly.

A Recent-folder LNK can support user interaction with a target that no longer exists, including a
removable drive or network share. An arbitrary LNK can exist without ever being clicked.

<span class="evidence-strength">Best conclusion:</span> the shortcut was created/updated with the
recorded target context. Its location and surrounding artifacts determine whether it supports use.

## Locations and collection

```text
%APPDATA%\Microsoft\Windows\Recent\*.lnk
%USERPROFILE%\Desktop\*.lnk
%APPDATA%\Microsoft\Windows\Start Menu\Programs\...
%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\...
```

Search the whole user profile and collected archives/email content, not only `Recent`. Preserve
the source file's own times and path because location changes interpretation.

## Analyst cautions

- Target times, size, MFT reference, and volume data are snapshots stored in the LNK; they are not
  the shortcut file's own metadata and may be stale.
- `MachineID`, MAC, and tracker creation time describe distributed-link-tracking context, commonly
  the machine that created/updated the link—not necessarily the machine where you found it.
- A MAC-like UUID node can come from virtualization, randomization, or non-network identifiers.
- `Arguments` can make a benign executable (`powershell.exe`, `rundll32.exe`, `mshta.exe`) dangerous.
- ANSI strings require the source system's code page; use `--code-page 932` for many Japanese links.
- A shortcut can be crafted or copied. Do not equate its fields with execution without Prefetch,
  Jump List, RecentDocs, event-log, or other support.

## Parse with Juntei

```text
Usage: juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.lnk
  -f, --file <FILE>      Parse one shortcut
      --code-page <CP>   ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -d evidence/ --code-page 932 -o lnk-ja.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

## CSV output fields

| Field group | Fields and interpretation |
|---|---|
| Source | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: shortcut path and host-file-system times. These are separate from target times. |
| Target times | `TargetCreated`, `TargetModified`, `TargetAccessed`: snapshots recorded in the LNK; useful historical clues, not current target metadata. |
| Target basics | `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`: recorded target/launch context. Arguments can change meaning. |
| Volume | `DriveType`, `VolumeSerialNumber`, `VolumeLabel`: identify fixed/removable/network context and correlate other shortcuts. |
| Paths | `LocalPath`, `NetworkPath`, `CommonPath`, `TargetIDAbsolutePath`: complementary path representations. Review disagreement and shell namespace paths. |
| Launch behavior | `Arguments`, `ShowWindow`, `HotKey`, `ShimLayer`, `DarwinId`: how the shell launches or configures the target. `Arguments` is critical. |
| NTFS identity | `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`: target reference at link creation; match both to `$MFT`. |
| Tracking | `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `VolumeDroid`, `FileDroid`, `VolumeDroidBirth`, `FileDroidBirth`: distributed-link tracking. Use as correlation, not attribution alone. |
| Extra blocks | `ExtraBlocksPresent`: list of optional structures available; explains which other fields are supported. |
| Presentation | `Name`, `IconLocation`, `IconIndex`, `IconEnvironment`, `EnvironmentVariables`: display and environment-expanded target data. Attackers can use misleading names/icons. |
| Shell folders | `KnownFolder`, `SpecialFolder`: decoded Windows shell folder identifiers. |
| Network | `NetworkDevice`, `NetworkProvider`: network-share/device and provider context. |
| Encoding | `ConsoleCodePage`: recorded console code page where present; separate from parser `--code-page`. |
| Trailing data | `AppendedBytes`: bytes after the logical end. Unexpected data can indicate smuggling or malformed files. |
| Validation | `Problems`: damage, truncation, inconsistent blocks, or decoding limitations. Include in review. |

JSON/JSONL additionally preserves every target-ID item (kind, name, GUID, FAT times and MFT
reference), all extra-data blocks, property-store properties, console settings, and raw structural
relationships. Use it for a suspicious shortcut rather than relying only on the flat CSV row.

## Investigation strategy

1. Examine source location and source times before interpreting target fields.
2. Reconstruct the effective launch from target path, `Arguments`, `WorkingDirectory`, and environment blocks.
3. Pivot target MFT reference, volume serial, network path, droids, MachineID, and MAC across evidence.
4. Check Prefetch and event logs for execution; check Jump Lists/RecentDocs for user interaction.
5. For malicious attachments, preserve appended data and hash/analyze the LNK itself.

## Further Details

- [Microsoft Open Specifications: MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [Sootmark shell](https://github.com/Sootmark/shell)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [Eric Zimmerman's LECmd](https://github.com/EricZimmerman/LECmd)
