# Open and Save As Dialogs (OpenSaveMRU)

## What the dialog history is

When a program shows the common Open or Save As dialog, Windows records in the user's
`NTUSER.DAT`:

- **which file was opened or saved:** `OpenSavePidlMRU` has one subkey per extension (`docx`,
  `ps1`, …) and `*` for every extension, each a most-recently-used list;
- **which program used the dialog, and the folder it was in:** `LastVisitedPidlMRU` (and
  `LastVisitedPidlMRULegacy` for programs that use the older dialogs);
- **programs that showed the dialog:** `CIDSizeMRU` (the dialog's size per program), and
  `FirstFolder` (a program's path and a folder);
- on Windows 2000 and XP, `OpenSaveMRU` (paths per extension) and `LastVisitedMRU` (programs and
  their folders as paths).

Since Windows Vista the files and folders are shell item lists, the same structures as a
shortcut's target: each item is a part of the path (This PC, a drive, a folder, the file), and
a file or folder item can carry its own times and NTFS file reference.

<span class="evidence-strength">Best conclusion:</span> a program used the Open or Save As dialog
to open or save this file, or in this folder, in this user's session. Only the most recent
entry of each list is dated: its key's last-write time.

## Location

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

Collect `NTUSER.DAT` with its transaction logs for every profile.

## Analyst cautions

- **One time per list.** A key's last-write time changes when one of its entries moves to the
  front of its list, so it dates the entry at position 0 only (`IsMostRecent`, `LastUsedUTC`).
  The other entries are in order, newest to oldest down the list, but undated.
- **The same file in two lists.** A file is in its extension's list and in `*`; do not count it
  as two uses. `*` gives the order across extensions.
- **Opened or saved.** The lists don't say which. Correlate with the file's `$MFT` and USN
  Journal records, LNK files and jump lists.
- **Item times are the file's, not the dialog's.** `ItemModifiedUTC`, `ItemCreatedUTC` and
  `ItemAccessedUTC` are the file's times when the item was written, to 2 seconds (FAT). Items of
  known folders (`CLSID_ThisPCDocumentsRegFolder`, `Downloads`, …) often have none. These FAT
  timestamps have no timezone offset. The `UTC` column suffix is a display convention, not
  evidence that the original clock was UTC; correlate with the source system's timezone and
  other timestamps before comparing them with registry FILETIMEs. See Microsoft's
  [file-time guidance](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times).
- **Paths are shell paths.** They start at the desktop: `This PC\C:\Users\…`, a known folder, a
  library (`UsersLibraries\Documents\…`) or a network location. A path that starts with a folder
  or a file is on the desktop. An empty path is the desktop itself.
- **Items not decoded.** An item of a type the shell item parser doesn't know is shown by its
  class (`[class 0x00]`), its type is `unknown`, and the list's bytes are in `DataRaw`.
- **Programs are names.** `LastVisitedPidlMRU` and `CIDSizeMRU` keep a file name
  (`WINWORD.EXE`), not a path. `FirstFolder` keeps a path.
- **Lists are short and cleaned.** Each list keeps a limited number of entries, and cleanup tools
  delete the keys. Use `parse-registry read --recover-deleted` for deleted values.

## Parse with Juntei

```text
Usage: juntei parse-registry opensavemru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry opensavemru -d evidence/ -o opensavemru.csv
juntei parse-registry opensavemru -f NTUSER.DAT -t jsonl \
  | jq 'select(.list == "OpenSavePidlMRU" and .extension != "*")'
```

The summary lists the entries per list, the programs that used the dialogs, and each hive's most
recent file (the first entry of `*`).

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `Username`, `UserSid`, `SourcePath` | Host, the profile owner (from the path or the shell folders), the SID from the system's `ProfileList`, and the hive. |
| `List` | `OpenSavePidlMRU`, `LastVisitedPidlMRU`, `LastVisitedPidlMRULegacy`, `CIDSizeMRU`, `FirstFolder`, `OpenSaveMRU` or `LastVisitedMRU`. |
| `Extension` | The extension's subkey (`docx`, `*`); empty for the other lists. |
| `MruPosition`, `IsMostRecent` | The place in the key's list (0 is the most recent); empty when the list doesn't name the value. |
| `ValueName` | The value's number (`MRUListEx`) or letter (`MRUList`). |
| `Program` | The program that showed the dialog. |
| `Path` | The file or folder: the shell items' names joined, or the path stored as text. |
| `ItemType` | The last shell item's type: `file`, `directory`, `root_folder`, `volume`, `network_location`, `property_view`, `unknown`, … |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | The file's or folder's times, as its shell item recorded them. |
| `MftEntry`, `MftSequence` | The NTFS file reference the shell item recorded. |
| `LastUsedUTC` | The key's last-write time, for the most recent entry only. |
| `KeyPath`, `KeyLastWrittenUTC` | The key and its last write: the time of its most recent entry, not of every row. |
| `DataRaw` | The value's bytes (hex) when they don't decode, or the shell item list when an item's type is unknown. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Values the list names but the key hasn't, values it doesn't name, wrong types, names or lists cut short. |

## Investigation strategy

1. Read `*` in order per user, and `LastUsedUTC` for the most recent file.
2. Look for scripts, archives, executables and documents in user-writable, removable or network
   places, and for files named like credentials or exports.
3. Use `LastVisitedPidlMRU` to tie a program to the folder it last used, and `CIDSizeMRU` and
   `FirstFolder` for programs that used the dialogs at all.
4. Correlate the paths and `MftEntry` with the `$MFT`, the USN Journal, LNK files, jump lists,
   ShellBags and RecentDocs.

## Further Details

- [libyal winreg-kb: Most recently used (MRU) keys](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi: Windows Shell Item format](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
