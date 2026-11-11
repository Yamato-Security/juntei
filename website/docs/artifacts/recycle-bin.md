# Recycle Bin

## What the Recycle Bin records

When a user deletes a file or folder in File Explorer, Windows moves it to the Recycle Bin of its
volume and writes a small index record beside it: the path it had, its size, and when it was
deleted. Each user has a bin folder on each volume, named by the user's SID, so the folder also
says which account deleted the item.

| Windows version | Index | Deleted content |
|---|---|---|
| Vista and later | One `$I…` file per item (format version 1 on Vista to 8.1, version 2 on Windows 10 and later) | `$R…`, with the same name after `$R`: the file, or the folder with its contents |
| NT 4.0, 2000, XP, Server 2003 | One `INFO2` file per bin: 800-byte records with an ANSI and a Unicode path | `D<drive letter><index><extension>`, such as `Dc1.txt` |
| 95, 98, ME | One `INFO2` file (`INFO` on Windows 95): 280-byte records with only an ANSI path | The same `D…` names |

<span class="evidence-strength">Best conclusion:</span> the account whose bin holds the record moved
the item at the original path to the Recycle Bin at the recorded time. Deleting with Shift+Delete,
from the command line or from most programs bypasses the bin and leaves no record.

## Locations and collection

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

Collect the `$I` files or INFO2 of every volume's bin. Collecting the `$R` and `D…` files too
shows which deleted items are still there; without them, `RecycledFileFound` is `false` for every
item. The deleted content is evidence in its own right and can be malware: handle it as such.

## Analyst cautions

- `DeletedTime` is when the item was moved to the bin, by the system clock, as a UTC FILETIME. It
  is not when the bin was emptied. The item's own creation and modification times are in `$MFT`
  and the `$R` file.
- The SID is the account that deleted the item, not necessarily the file's owner. A SID without a
  profile in the SOFTWARE hive's `ProfileList` (a deleted account, or a domain account that never
  logged on) has no `User`.
- When the bin is emptied, or an item is restored or deleted from the bin, Windows Vista and later
  delete its `$I` and `$R` files. Their names can remain in the USN Journal, `$MFT` and `$I30`
  slack; Juntei does not recover deleted `$I` records from them. INFO2 keeps such records and
  clears the first byte of their ANSI path: `RemovedFromBin` is `true`, and Juntei puts that
  character back from the drive number.
- `FileSize` in a `$I` file is the item's size; for a folder, the total size of its contents. In
  INFO2 it is the space the item took on disk, in whole clusters (4,096 bytes for a small file).
- INFO2's ANSI path is often in 8.3 form and needs the system's code page: use `--code-page 932`
  for Japanese Windows, 936 or 950 for Chinese. The Unicode path, when there is one, is
  `OriginalPath`.
- Some Windows Vista systems wrote 543-byte `$I` files whose size field is one byte short. Juntei
  reads their time and path and leaves `FileSize` empty, with a problem.
- `RecycledFileFound` describes the collection, not the endpoint: `false` can mean the collector
  did not take `$R` files, or that the item is no longer in the bin.
- Bins on non-system volumes use profiles only from the same collected computer, identified by
  collection metadata or recognized drive/volume folders. If attribution is ambiguous, the user
  remains unknown; profiles are never borrowed from another host.

## Parse with Juntei

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

With `-d`, Juntei reads files named `$I` followed by six letters or digits (and any extension),
`INFO2`, and `INFO` in a `RECYCLED` folder. With `-f`, it reads the file whatever its name.

## CSV output fields

| Field | Meaning |
|---|---|
| `SourceFile` | The `$I` or INFO2 file. |
| `DeletedTime` | When the item was moved to the bin (UTC). |
| `OriginalPath` | Where the item was. For INFO2, the Unicode path when the record has one, else the ANSI path. |
| `FileSize` | `$I`: the item's size in bytes (a folder's total). INFO2: its space on disk, in whole clusters. Empty when it can't be read. |
| `User`, `SID` | The bin folder's SID, and the user its `ProfileList` profile is named after. |
| `RecycledFile`, `RecycledFileFound` | The deleted content's path beside the index (`$R…`, `D<drive><index><extension>`), and whether it is in the collection. |
| `Format`, `FormatVersion` | `$I` 1 (Vista to 8.1) or 2 (Windows 10 and later); `INFO2` 0 (95), 2 (NT 4.0), 4 (98) or 5 (ME, 2000 to Server 2003). |
| `RecordIndex`, `Drive`, `RecordOffset` | INFO2: the record's number (the number in its `D…` file name), the drive, and where the record starts in INFO2. A UNC path has no drive. |
| `RemovedFromBin` | INFO2: `true` when the item was restored or deleted from the bin. |
| `ShortPath` | INFO2: the ANSI path, when it differs from `OriginalPath` (often 8.3 names). |
| `Problems` | Damage: a truncated or overlong file, an invalid drive number, a path that is not valid UTF-16, a deletion time that is not a time. |

JSON and JSONL have the same fields, with numbers and booleans typed, `null` for values a record
does not have, and `Problems` as a list.

If a bin directory cannot be listed or a payload name is ambiguous, `RecycledFileFound` is unknown
(an empty CSV cell or JSON `null`), with the cause recorded in `Problems`. Output cannot replace
recycled content, including through hard links, or be written inside a recycled folder. Existing
results are preserved when no index can be read.

## Automatic parsing

`auto-parse` writes `FileExistence/RecycleBin.csv` (`RecycleBin.json` or `RecycleBin.jsonl` with
`-t json` or `-t jsonl`), with `FileExistence/RecycleBin-logs.txt`. Its `--code-page` applies to
INFO2. From disk images, `$I` files and INFO2 are copied; `$R` and `D…` files are empty
placeholders, so `RecycledFileFound` is still filled. The combined `-j` timeline includes every
row with the `Artifact` identifier `recycle-bin`, `-Z` converts `DeletedTime`, and
[`super-timeline`](../commands/super-timeline.md) makes one `Recycle Bin deletion (moved to the bin)`
event per item.

## Investigation strategy

1. Sort by `DeletedTime` per user, and compare with the times of the activity you are investigating.
2. Look up `OriginalPath` in LNK, Jump Lists, ShellBags and RecentDocs to see whether the item was
   used before it was deleted.
3. In the USN Journal, the deletion appears as a rename into `$Recycle.Bin` at about `DeletedTime`;
   `$MFT` gives the item's own times.
4. When `RecycledFileFound` is `true`, hash and examine the `$R` or `D…` content.
5. If the bin was emptied, search the USN Journal and `$MFT` for `$I` and `$R` names.

## Further Details

- [libyal dtformats — Windows Recycle.Bin file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — Recycle Bin and INFO2 parsers](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
