# ShellBags

## What ShellBags are

Explorer remembers how each folder a user opens is displayed: its view, icon size, sort order and
window position. To find a folder's settings again, it keeps the folder's identity in the user's
registry as a tree of shell items, the same items a link's target ID list holds: a drive, then each
folder down to the one opened. These trees are ShellBags.

They cover folders opened in Explorer and in open and save dialogs, including folders on removable
drives, network shares, portable devices (phones, cameras), ZIP archives and FTP servers, and
Control Panel pages. They stay after the folder, the drive or the network share is gone.

<span class="evidence-strength">Best conclusion:</span> this user's Explorer showed this folder
(or a folder under it). A ShellBag doesn't show what was done in the folder, and it doesn't date
every visit: see the times below.

## Location

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

A `BagMRU` key holds numbered values, each one shell item, and an `MRUListEx` value giving their
order, the most recently opened first. Each value has a subkey of the same name for the items
under it, so a path is the items from `BagMRU` down to a value. A subkey's `NodeSlot` is the
number of its `Bags\<N>` key, where the folder's view settings are.

Collect `UsrClass.dat` and `NTUSER.DAT` with their transaction logs (`.LOG1`, `.LOG2`) for every
profile, and the system's `SOFTWARE` hive, which gives an `NTUSER.DAT` user's SID.

## Which times say what

- **The shell item's times** (`ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC`) are the
  folder's own file system times (FAT precision: 2 seconds) when Explorer wrote the item. They
  date the folder, not the visit, and are not updated on later visits.
- **A key's last-write time** changes when its `MRUListEx` is rewritten, that is when one of its
  items is opened and moved to the front. So the time of the key an item's value is in
  (`ParentKeyLastWrittenUTC`) is when the item was last opened **only when it is the most recent
  item in that key** (`IsMostRecentInParent` true, `MruPosition` 0). For the other items, it is a
  later time, when a sibling was opened.
- An item's own key (`KeyLastWrittenUTC`) dates the opening of its most recent child the same
  way.

## Analyst cautions

- **Absence proves little.** Explorer writes ShellBags when a folder's view is saved; a folder
  seen only briefly, or opened another way (a command line, a script, a program), may have none.
  A user may delete the keys.
- **An NTFS reference is the folder's** (`MftEntry`, `MftSequence`) when the item was written.
  Look it up in the `$MFT` to see whether the same folder still exists, or another file reused the
  entry.
- **Removable drives** are named by their drive letter. The letter is the one the drive had then;
  match it with the USB and mounted-device history for the device.
- **Names come from the item.** A folder renamed later keeps its old name here. Root and known
  folders are named as forensic tools commonly name them (`This PC`, `Quick Access`, `Downloads`);
  `Guid` gives their identifier.
- **Search results, libraries and Control Panel pages** are views, not folders on disk
  (`property_view`, `control_panel_item`, `control_panel_category`).
- **An item no parser knows** keeps its bytes in `DataRaw`, with `ItemType` `unknown` and its
  class byte in `ItemClass`. Damaged or unusual items are reported in `Problems`, and their bytes
  are kept too.
- **A dirty hive** may lack its latest items without its logs. Juntei replays the logs next to the
  hive unless `--no-logs`; `TransactionLogStatus` says whether a record came from them.

## Parse with Juntei

```text
Usage: juntei parse-registry shellbags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shellbags -d evidence/ -o shellbags.csv
juntei parse-registry shellbags -f UsrClass.dat -t jsonl \
  | jq 'select(.item_type == "volume" and .guid == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

Shell items are decoded with ys-lnk-parser, the shell item parser of `parse-file lnk`. The
summary lists the folders opened last (newest first), from their parent key's last-write time,
with their users.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `Username`, `UserSid`, `SourcePath`, `HiveType` | Host, the profile folder's name, the user's SID (a `UsrClass.dat`'s root key names it; for `NTUSER.DAT`, the system's `ProfileList`), the hive and its type. |
| `BagKey` | The item's key, under the hive's root (`…\BagMRU\4\3\0`): its value is in the key above it. |
| `MruPosition`, `IsMostRecentInParent` | Its place in its parent key's `MRUListEx` (0: the most recently opened); empty when the list doesn't have it. |
| `NodeSlot` | The number of its `Bags\<N>` key (the folder's view settings). |
| `Path` | The items from the top of its tree down to it, joined with `\`. `[unknown]` stands for an item whose value is missing. |
| `ItemType`, `ItemClass` | What the item is (`root_folder`, `volume`, `directory`, `file`, `network_location`, `uri`, `ftp_folder`, `compressed_folder`, `portable_device`, `portable_device_folder`, `users_files_folder`, `property_view`, `control_panel_item`, `control_panel_category`, `cd_burn`, `unknown`), and its class byte. |
| `ItemName`, `Guid` | Its name (a folder's long name), and the class identifier of a root folder, Control Panel item or delegate folder. |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | **From the shell item:** the folder's times when the item was written (FAT, 2-second precision). |
| `MftEntry`, `MftSequence` | **From the shell item:** the folder's NTFS file reference. |
| `ExtensionBlocks` | How many extension blocks the item has. |
| `KeyLastWrittenUTC` | **From the registry:** the last write of the item's own key: when its most recent child was opened. |
| `ParentKeyLastWrittenUTC` | **From the registry:** the last write of the key its value is in: when it was last opened, if `IsMostRecentInParent` is true. |
| `DataRaw` | The value's bytes (hex), for an item that isn't fully understood or has problems. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | A value its key's `MRUListEx` names but the key hasn't, a value the list doesn't name, a subkey without its value, a value that isn't binary, a `NodeSlot` of the wrong type, a damaged shell item list. |

## Investigation strategy

1. List removable drives (`volume` with the Removable Drives `Guid`), network locations, FTP
   folders and portable devices, and the folders under them.
2. Look for folders of interest: user-writable staging places (`Temp`, `ProgramData`, `Public`),
   tool and archive folders, other users' profiles, and folders that no longer exist (check the
   NTFS reference in the `$MFT`).
3. Date visits only with `ParentKeyLastWrittenUTC` of items where `IsMostRecentInParent` is true;
   correlate the rest with LNK files, Jump Lists and RecentDocs, which date opens of files.
4. Use `parse-registry read --recover-deleted` for deleted `BagMRU` keys and values.

## Further Details

- [libyal libfwsi: Windows Shell Item format](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb: Most recently used (BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

SID lookup from `ProfileList` requires one unique, syntactically valid SID. Ambiguous profile-folder mappings and backup keys such as `S-1-...bak` leave `UserSid` unknown.
