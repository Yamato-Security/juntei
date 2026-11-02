# Application Compatibility

## What Windows keeps

Besides the ShimCache (`parse-registry shimcache`) and Amcache (`parse-registry amcache`),
Windows' application compatibility features keep these under
`Microsoft\Windows NT\CurrentVersion\AppCompatFlags`:

| Source | Hive and key | What it shows |
|---|---|---|
| `layers` | `SOFTWARE` (and `Wow6432Node`) and `NTUSER.DAT` `Layers` | Programs set to run with compatibility layers: `RUNASADMIN` (always as administrator), `WIN7RTM`, `HIGHDPIAWARE`, … |
| `pca_store` | `NTUSER.DAT` `Compatibility Assistant\Store` (Windows 8 and later) | Programs the Program Compatibility Assistant saw the user run |
| `pca_persisted` | `NTUSER.DAT` `Compatibility Assistant\Persisted` (Windows 7) | The same, on Windows 7 |
| `cit_module` | `SOFTWARE` `CIT\Module\<module>` | Programs that loaded a tracked module, and when they last did |
| `cit_system` | `SOFTWARE` `CIT\System` (Windows 7 to early Windows 10) | Program paths and command lines in the compressed CIT databases |
| `installed_sdb` | `SOFTWARE` `InstalledSDB\{GUID}` | Custom shim databases installed with `sdbinst`: file, description, install time |
| `custom_sdb` | `SOFTWARE` `Custom\<program>` | The custom shim databases applied to each program, with their install times |

<span class="evidence-strength">Best conclusion:</span> layers and shim databases are
configuration that changes how a program runs; Program Compatibility Assistant and CIT entries
show that a program ran (or loaded a module) as this user or on this system.

Custom shim databases are a persistence and injection technique (MITRE ATT&CK T1546.011): a shim
can load a DLL into a program or start another one whenever the program runs.

## Location

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

Collect `SOFTWARE`, every `NTUSER.DAT` and their transaction logs, and the `.sdb` files a shim
database's path names (usually `C:\Windows\AppPatch\Custom\`).

## Analyst cautions

- **No run time for Program Compatibility Assistant entries.** The `Store` data (`SACP`, kept in
  `DataRaw`) has one time, but on Windows 8.1, 10 and 11 it is the same for every program of a
  hive, and for other systems: it is not when the program ran. Only the `Store` key's last write
  dates the most recent change.
- **Programs on removable media** are named `SIGN.MEDIA=<id> <path>`: `Path` is the path on the
  media, and `Details` has the media id.
- **CIT databases** are LZNT1-compressed and their structure isn't publicly documented: Juntei
  lists the program paths and command lines found in each, once per database, without counts or
  times. A command line is its own record, not tied to its program. Windows 10 and later keep
  little here.
- **Shim databases.** Check every `custom_sdb` and `installed_sdb` record: legitimate software
  rarely installs one. The `.sdb` file itself says what the shim does.
- **Layers** set by a user apply only to that user; `SOFTWARE`'s to everyone. `RUNASADMIN` makes a
  program ask for elevation every time.

## Parse with Juntei

```text
Usage: juntei parse-registry appcompat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry appcompat -d evidence/ -o appcompat.csv
juntei parse-registry appcompat -d evidence/ -t jsonl \
  | jq 'select(.source == "installed_sdb" or .source == "custom_sdb")'
```

The summary lists the custom shim databases, the programs with layers, and the CIT modules' last
uses.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, hive file, and `SOFTWARE` or `NTUSER.DAT`. |
| `Username`, `UserSid` | A user hive's profile folder name, and its SID from the system's `ProfileList`. |
| `Source`, `Scope` | `layers`, `pca_store`, `pca_persisted`, `cit_module`, `cit_system`, `installed_sdb`, `custom_sdb`; `machine` or `user`. |
| `Path` | The program (the value's name), a path or command line in a CIT database, or the program a shim database applies to. |
| `Layers` | The compatibility layers, as written (`~ RUNASADMIN`). |
| `LastSeenUTC` | When a program last loaded a CIT module. |
| `SdbGuid`, `SdbPath`, `SdbInstalledUTC` | A shim database's GUID, file and install time. |
| `Details` | A CIT module's name, a CIT database's value name, a program's removable media id (`SignMedia`), a shim database's `DatabaseDescription` and `DatabaseType`. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName`, `Wow64` | Where the record comes from, when that key last changed, and whether it is under `Wow6432Node`. |
| `DataRaw` | The `Store` value's `SACP` data, or data that isn't what it should be (hex). |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Data that couldn't be decoded or read. |

## Investigation strategy

1. Review every custom shim database: its `.sdb` file, install time, and the programs it applies
   to; correlate with `sdbinst.exe` in Prefetch, Amcache and process creation events.
2. Look for programs in user-writable folders among Program Compatibility Assistant entries, and
   for programs from removable media.
3. Check `RUNASADMIN` and other layers on unusual programs.
4. Correlate with ShimCache, Amcache, BAM, Prefetch and UserAssist for execution times.

## Further Details

- [Microsoft: Application compatibility database](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011: Application Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft: MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
