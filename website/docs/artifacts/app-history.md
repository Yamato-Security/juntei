# Application History

## What tools leave behind

Many third-party tools keep settings and histories in the user's `NTUSER.DAT`. Several are among
the tools attackers use, or use to stage data before taking it: Sysinternals tools, archivers,
cloud sync clients and remote access tools. Juntei reads these:

| Application | Key under `HKCU\Software` | What it shows |
|---|---|---|
| Sysinternals | `Sysinternals\<tool>` | Each tool whose licence the user accepted (`EulaAccepted`): PsExec, ProcDump, Sysmon, … |
| WinRAR | `WinRAR\ArcHistory`, `DialogEditHistory\ArcName`, `DialogEditHistory\ExtrPath` | Archives opened; archive names and extraction folders typed in its dialogs |
| 7-Zip | `7-Zip\Compression\ArcHistory`, `Extraction\PathHistory`, `FM\FolderHistory`, `FM\CopyHistory`, `FM\PanelPath0/1` | Archives made; extraction folders; folders browsed and copied to in its file manager |
| WinZip | `Nico Mak Computing\WinZip\filemenu`, `extract`, `mru\archives` | Archives on its File menu, extraction folders, recent archives |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | Each account's e-mail address, local folder and names; synced libraries (`ScopeIdToMountPointPathCache`) and folders (`Tenants\<tenant>`) |
| Adobe Acrobat and Reader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`, `cRecentFolders` | PDF files and folders opened last, with file names, sizes and dates |
| TeamViewer | `TeamViewer` (and an older version's `Version<N>`) | Every value: account and user names, file transfer start folders (`<partner ID>?<local>|<remote>`), … |
| CCleaner | `Piriform\CCleaner` | Every value: what it is set to clean, `UpdateKey`, … |

<span class="evidence-strength">Best conclusion:</span> this user ran the tool (or accepted its
licence), or the tool opened, made or extracted these archives and folders, or this cloud account
was set up for the user. Most histories have no time per entry: only the most recent entry gets
its list key's last-write time.

## Location

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

Collect every user's `NTUSER.DAT` with its transaction logs, and the system's `SOFTWARE` for
users' SIDs.

## Analyst cautions

- **Sysinternals EULA acceptance.** `FirstUsedUTC` is a first-use estimate from the key's
  last-write time when its only value is a readable `EulaAccepted` set to `1`. A declined or
  damaged EULA does not receive this estimate. Registry settings can also be preconfigured,
  copied or edited: corroborate execution and its time with other artifacts.
  Tools that keep settings there (Process Explorer, Autoruns, …) rewrite the key later:
  `FirstUsedUTC` is then empty and `KeyLastWrittenUTC` is only the last settings change. A tool
  run with `-accepteula`, or as another user (PsExec's remote side runs as SYSTEM), writes the
  key of that user.
- **Positions, not times.** List entries are ordered (`MruPosition` 0 is the most recent), but
  only the list key's last write dates the most recent one. Older entries were used earlier, at an
  unknown time.
- **Typed names.** WinRAR's `ArcName` is the archive name typed when creating one: often a file
  name without a folder.
- **Adobe paths** are kept as Adobe writes them (`/C/Users/alice/a.pdf` is
  `C:\Users\alice\a.pdf`). Its `sDate` (in `Details`) is a PDF date string, kept as written.
- **CCleaner's `UpdateKey`** is local time without a zone; it is kept as written.
- **TeamViewer.** Connection logs are files (`Connections_incoming.txt`, `TeamViewer*_Logfile.log`),
  not registry values: collect them too.
- **Absence.** Users and tools can clear these histories (CCleaner itself can).

## Parse with Juntei

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.application == "Sysinternals" and .first_used != null)'
```

The summary lists the newest Sysinternals first-use estimates, the archives and folders each list used
last, and the OneDrive accounts.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `SourcePath` | Host and `NTUSER.DAT`. |
| `Username`, `UserSid` | The profile folder's name, and its SID from the system's `ProfileList`. |
| `Application` | `Sysinternals`, `WinRAR`, `7-Zip`, `WinZip`, `OneDrive`, `Adobe Acrobat Reader`, `Adobe Acrobat`, `TeamViewer`, `CCleaner`. |
| `Item` | The Sysinternals tool, the list (`ArcHistory`, `ExtrPath`, `FolderHistory`, `filemenu`, `cRecentFiles`, …), the OneDrive account or its synced folders' key, or the value's name. |
| `MruPosition` | The entry's position in its list, 0 the most recent. |
| `Path` | The archive, folder or document. |
| `Value` | `EulaAccepted`, a OneDrive account's e-mail address or a synced library's id, a TeamViewer or CCleaner value. |
| `FirstUsedUTC` | First-use estimate from EULA acceptance (see cautions above). |
| `Details` | A OneDrive account's `DisplayName`, `UserCID`, `ConfiguredTenantId`, …; an Adobe document's `Version`, `tFileName`, `uFileSize`, `uPageCount`, `sDate`. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName` | Where the record comes from, and when that key last changed. |
| `DataRaw` | The value's bytes (hex), when they don't decode. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Data that couldn't be decoded or read. |

## Investigation strategy

1. Look for Sysinternals tools an ordinary user doesn't run (PsExec, ProcDump, PsLoggedOn,
   SDelete) and their EULA timestamps.
2. Review archive names and extraction folders around the incident: archives made in
   `ProgramData`, `Temp` or a user's folder before data left the system.
3. Check OneDrive accounts that aren't the organization's, and TeamViewer account names and file
   transfer folders.
4. Correlate with Prefetch, Amcache, BAM and UserAssist (the tool running), the `$MFT` and USN
   Journal (the archive's file), LNK files and jump lists (opened documents), and event logs.

## Further Details

- [Microsoft: Sysinternals](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
