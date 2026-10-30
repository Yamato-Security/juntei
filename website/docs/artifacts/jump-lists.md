# Jump Lists

## What Jump Lists are

Jump Lists back the menus shown from taskbar and Start-menu application entries. Automatic Jump
Lists are Windows-managed histories of items used by an application. Custom Jump Lists are
application-defined categories and tasks. Both can embed LNK structures with target paths, times,
volume/network context, MFT references, and launch arguments.

Automatic records can reveal files or folders opened by an application, last-use order/time,
interaction count, and pinned state. Custom lists can expose “recently closed” pages or application
tasks—including malicious command arguments.

<span class="evidence-strength">Best conclusion:</span> the owning application recorded or exposed
the item/task in its Jump List. Behavior differs by application; corroborate actual execution/access.

## Locations and collection

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

The hexadecimal file-name prefix is an AppID. Collect both directories and retain the user profile
path. Automatic lists are OLE Compound File containers with a `DestList` stream and numbered LNK
streams; custom lists use their own category/link format.

## Analyst cautions

- AppID descriptions come from a known mapping; an unknown AppID is not suspicious by itself.
- `LastModified` / last-used fields belong to a DestList entry; embedded target times are different.
- Interaction counts exist only in supported versions and are not a full open count.
- Pinning preserves items beyond normal recency; cleanup or application settings can remove others.
- A list can contain stale/unattached streams or bytes from an earlier, longer custom list.
- Embedded LNK arguments and paths are attacker-controlled. Application association is context, not trust.

## Parse with Juntei

```text
Usage: juntei parse-jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-jump-lists -d evidence/ -o jump-list-csv
juntei parse-jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

## AutomaticDestinations CSV fields

| Field group | Fields and interpretation |
|---|---|
| Source | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: the Jump List container and its file-system times. |
| Application | `AppId`, `AppIdDescription`: file-name AppID and known application mapping. Do not infer identity from an unknown hash. |
| Container | `HasSps`, `DestListVersion`, `LastUsedEntryNumber`: DestList structure/version and header context. |
| Ordering | `MRU`, `EntryNumber`: most-recent-use position and hexadecimal entry/stream identifier. |
| Use metadata | `CreationTime`, `LastModified`, `InteractionCount`, `PinStatus`, `PinPosition`: DestList lifecycle/use/pin state. Interpret by version. |
| Host/path | `Hostname`, `MacAddress`, `Path`, `DisplayPath`: target location and tracking context. Host/MAC is not sole attribution. |
| Tracking IDs | `FileBirthDroid`, `FileDroid`, `VolumeBirthDroid`, `VolumeDroid`: correlate moved targets and embedded LNK tracking. |
| Notes | `Notes`: tells you about unattached streams, recovered leftovers, or other parser context. |
| Embedded LNK | `TargetCreated`, `TargetModified`, `TargetAccessed`, `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`, `DriveType`, `VolumeSerialNumber`, `VolumeLabel`, `LocalPath`, `CommonPath`, `TargetIDAbsolutePath`, `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`, `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `ExtraBlocksPresent`, `Arguments`: interpret as described on [LNK Shortcuts](lnk.md). |
| Additional LNK | `NetworkPath`, `Name`, `IconLocation`: network and presentation fields from the embedded shortcut. |
| Validation | `Problems`: missing streams, truncation, count disagreement, or link damage. |

## CustomDestinations CSV fields

Custom output starts with `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`, `AppId`,
`AppIdDescription`, and `EntryName` (category name or `Tasks`), followed by all embedded LNK fields
listed above, then `Notes`, `NetworkPath`, `Name`, `IconLocation`, and `Problems`.

JSON/JSONL additionally preserves the DestList header, per-entry score/checksum/property store,
`DestListPropertyStore`, compound-file streams, custom categories, bytes after the last category,
and every embedded LNK's nested fields.

## Investigation strategy

1. Identify user and source profile, then map AppID to application cautiously.
2. Sort automatic entries by `LastModified` and `MRU`; separate pinned items.
3. Review paths, network shares, removable volume serials, MFT references, and arguments.
4. Inspect `Notes` and unattached/leftover links—deleted or earlier items can survive there.
5. Correlate with RecentDocs, standalone Recent LNKs, Prefetch, SRUM, `$MFT`, and event logs.

## Further Details

- [Microsoft: Compound Binary File format (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft: Shell Link format (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [Eric Zimmerman's JLECmd](https://github.com/EricZimmerman/JLECmd)
- [JumpList parser test/reference project](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
