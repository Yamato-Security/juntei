# Windows Search Index

## What the index records

Windows Search (the `WSearch` service) crawls the user profiles, the Start menu, Outlook's
mailboxes, Internet Explorer's history, offline files and whatever else is in its scope, and keeps
what it read in `Windows.edb`, an ESE database. Two parts of it matter to an investigation:

- the **property store** (`SystemIndex_PropertyStore`; `SystemIndex_0A` on Windows 7) with one row
  per indexed item: its path, name, type and size, its owner, the created, modified and accessed
  times the indexer copied from the file, when the indexer read it, a document's title and author,
  the beginning of its text, the computer name, and hundreds of other properties;
- the **gatherer** tables (`SystemIndex_Gthr`, `SystemIndex_GthrPth`) with the crawler's own record
  of each item: its URL path, its modification time when it was queued, the crawl number and
  flags, and a deletion counter.

An item stays in the index after the file, message or page is gone, until the crawler notices the
deletion. The index therefore names files that no longer exist, with their metadata and an extract
of their text as they were when they were indexed.

<span class="evidence-strength">Best conclusion:</span> the indexer read the item at the recorded
path, with the recorded metadata, at `GatherTime`, on the named computer. The index shows that the
file existed and what it contained. It does not show that a user opened it.

## Location and collection

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

Windows 7 to Windows 11 21H2, and the Windows Server releases of the same years, keep the index in
`Windows.edb`. Windows 11 22H2 and later keep it in SQLite databases in the same folder
(`Windows.db`, `Windows-gather.db`, `Windows-usn.db`); Juntei finds and names them, and reports
them as the unsupported newer format.

Collect the whole `Applications\Windows` folder from a disk image, a volume shadow copy, or after
stopping the `WSearch` service: the database is open while the service runs, and its newest
changes are only in the `MSS*.log` transaction logs, which Juntei does not replay. An index copied
from a running system is left dirty; the output says so, and reads what is on disk. Directory
input and `auto-parse` read every `Windows.edb` at any depth; use `-f` for one file.

## Analyst cautions

- Parse an immutable evidence copy. Do not modify or truncate the database while it is being
  read; Juntei memory-maps the file for efficient access.
- `GatherTime` is when the indexer read the item, not when a user created, changed or opened it.
  Indexing runs in the background, often minutes to hours after a change, and pauses while the
  computer is busy or on battery.
- `DateCreated`, `DateModified` and `DateAccessed` are copies the indexer took from the file. They
  are refreshed when the item is indexed again, so they describe the file as of `GatherTime`.
- `ItemPath` is the display path. `ItemURL` and `GatherPath` are the crawler's URLs: `file:` for
  files (with the volume as `[GUID]` on Windows 8 and later), `iehistory://{SID}/` for Internet
  Explorer's history, `mapi://` for Outlook items, `csc://{SID}/` for offline files and
  `winrt://{SID}/` for Windows apps' content.
- `User` is the profile named in the item's path (`C:\Users\<name>\…`). It is where the item was,
  not who created it; `FileOwner` is the file's owner as the indexer saw it.
- `AutoSummary` is the beginning of the item's text. It can quote a document that has since been
  deleted or encrypted, and it is a truncated extract, not the document.
- A dirty database can hold pages its branch pages do not reach yet. Juntei follows the leaf
  pages' own chain and says in the summary how many it read that way. Its newest changes still sit
  in the transaction logs.
- An item in the gatherer tables without a property-store row (`InTables` is `Gthr`) was queued
  or crawled, and its properties were not written yet or have been removed. Its path, name and
  gatherer times are still there.
- Windows 7 stores strings in its own compressed, obfuscated encoding. Juntei decodes it; a value
  that does not decode stays hexadecimal, with a note in `Problems`.

## Parse with Juntei

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

CSV is the default file format. With `-t csv`, `-t json` or `-t jsonl` and no `-o`, records go to
stdout. Without either option, only a summary is displayed. Add `-C` to overwrite an existing
output. The output is never written over one of the input files, even with `-C`.

Juntei writes one record per indexed item, joining the property store and the gatherer tables on
the item's identifier. The summary counts the databases by schema, the dirty ones, the items in
both tables, in the gatherer only and in the property store only, the folders, the items marked
deleted, the items per store, the items without a stored time, and the problems.

Damage does not stop the run. A database cut short keeps the items its remaining pages hold, and
the summary says what could not be read. A file named `Windows.edb` that is not a Windows Search
index is reported and skipped. The other databases are still read.

## Output fields

| Field | Interpretation |
|---|---|
| `DocumentID` | The item's identifier in the index: the property store's `WorkID` (`DocID` on Windows 7) and the gatherer row's `DocumentID`. |
| `InTables` | `Gthr+PropertyStore`, `Gthr` (a queued or crawled item without properties) or `PropertyStore`. |
| `ItemPath` | `System_ItemPathDisplay`: the path as Windows shows it. Empty when the property store has no row. |
| `ItemURL` | `System_ItemUrl`: the crawler's URL for the item (`file:`, `iehistory://`, `mapi://`, `csc://`, `winrt://`). |
| `GatherPath` | The gatherer's path, joined from the scope tree and the row's file name. |
| `Name` | `System_ItemName`, or the gatherer row's file name. |
| `ItemType`, `ItemTypeText` | `System_ItemType` (the extension, or `Directory`) and its display name. |
| `Kind` | `System_Kind`: `document`, `folder`, `link`, `program`, `email`, `picture`, …; several are joined with `\|`. |
| `MimeType` | `System_MIMEType`. |
| `Store` | `System_Search_Store`: `file`, `mapi` (Outlook), `iehistory`, `csc` (offline files) or `winrt` (Windows apps). |
| `IsFolder`, `IsDeleted` | `System_IsFolder` and `System_IsDeleted`, as stored. |
| `DeletedCount` | The gatherer row's deletion counter. |
| `Size` | `System_Size` in bytes. Empty for folders, which hold a placeholder. |
| `FileAttributes` | `System_FileAttributes` as a number (16 directory, 32 archive, 2 hidden, …). |
| `FileOwner` | `System_FileOwner` (`DOMAIN\user`) as the indexer read it. |
| `DateCreated`, `DateModified`, `DateAccessed` | The file's times as the indexer copied them (UTC). |
| `ItemDate` | `System_ItemDate`: the item's own date, such as a message's sent time or a photo's date taken; for a file usually its modification time to the second. |
| `GatherTime` | `System_Search_GatherTime`: when the indexer read the item (UTC). |
| `GatherLastModified` | The gatherer row's `LastModified`: the item's modification time when the crawler queued it (UTC). |
| `Title`, `Author` | `System_Title` and `System_Author`. |
| `AutoSummary` | `System_Search_AutoSummary`: the beginning of the item's text. |
| `CompName` | `System_ComputerName`: the computer the index belongs to. |
| `TargetURL`, `DateVisited` | An Internet Explorer history item's URL and visit time (`System_Link_TargetUrl`, `System_Link_DateVisited`). |
| `ActivityApp`, `ActivityText`, `ActivityContentURI`, `ActivityStartTime`, `ActivityEndTime` | A Windows 10 activity history item (`System_Activity_*`, `System_ActivityHistory_*`). |
| `User`, `UserSource` | The profile named in the item's path, and which path it was read from (`ItemPath`, `ItemURL` or `GatherPath`). Empty outside a profile. |
| `ScopeID`, `SDID` | The gatherer row's scope and security descriptor identifiers. |
| `CrawlNumber` | The gatherer row's `CrawlNumberCrawled`. |
| `GatherFlags` | The gatherer row's `TransactionFlags`. |
| `Priority` | The crawl priority of an item still queued. ESE marks it NULL once the item is crawled, so it is empty for most items. |
| `PropertyCount` | How many properties the property store holds for the item. |
| `OtherProperties` | Every other property, as `Name=Value` separated by `; `. |
| `SourceFile` | The database read, relative to the input directory, or its name with `-f`. |
| `Problems` | Values that did not decode, times that are no times, duplicate rows, and damage. Do not suppress. |

JSON and JSONL keep the same fields with typed values: numbers stay numbers and missing values are
`null`. Instead of `OtherProperties` they have `Properties`, every property of the item with its
name, numeric identifier, column type and value, and for a time its raw FILETIME tick count beside
the time it reads as. `Gather` holds the gatherer row's columns.

## Timeline behavior

`auto-parse` saves the records as `FileExistence/WindowsSearch.csv` (or `.json`/`.jsonl`) and
adds them to the combined `-j` timeline. `super-timeline` makes events from `DateCreated`,
`DateModified` and `DateAccessed` (the file's times as recorded by Windows Search), from
`GatherTime` (when the item was indexed), from an Internet Explorer history item's `DateVisited`
and from an activity's `ActivityStartTime` and `ActivityEndTime`. `GatherLastModified` becomes an
event only for an item the property store holds no modification time for. An item without any of
these stays in the parsed output and is counted as untimed.

## Investigation strategy

1. Filter `Store` to `file` and search `ItemPath`, `Name` and `AutoSummary` for the files, names
   and words of the case. The index names files that have since been deleted.
2. Read `GatherTime` as the latest time the file existed with the recorded metadata, and
   `DateModified` as its modification time at that moment.
3. Use `FileOwner` and `User` to place a file with an account, then confirm with the `$MFT`, the
   USN Journal and `$Secure:$SDS`.
4. For e-mail (`Store` `mapi`) and Internet Explorer history (`iehistory`), pivot into Outlook's
   files and the WebCache for the full records.
5. Compare the index across hosts: `CompName` names the computer the index belongs to, and an
   index collected from an image names the files on that image.

## Tested versions

The tests use public indexes with results that other tools read independently:

- plaso's Windows 7 index (`SystemIndex_0A`), checked against libyal's `esedbexport`, including
  the Windows 7 string encodings.
- DFIRArtifactMuseum's Windows 10 index, left dirty, checked against Stroz Friedberg's SIDR, and
  its Windows 11 index (ESE format revision 200), checked against `esedbexport`.
- DFIRArtifactMuseum's Windows 11 22H2 SQLite index, reported as the unsupported newer format.
- Databases cut short, empty, or of another kind.

## Further Details

- [Microsoft — Windows Search overview](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE database file format](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb — Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg — SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft — Extensible Storage Engine source](https://github.com/microsoft/Extensible-Storage-Engine)
