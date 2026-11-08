# Web Browsers

## What the browser databases are

Browsers keep their history, downloads, cookies and form entries in SQLite databases in each
user's profile. Chromium-based browsers (Chrome, Edge, Brave, Opera, Vivaldi) and Firefox-based
browsers each use their own files and tables, but record the same kinds of activity:

- **History**: the pages visited, each visit with its time, how the browser came to the page
  (typed, a link, a redirect, a bookmark) and, in Chromium, how long the page was in front.
- **Downloads**: the URL a file came from (and the redirects on the way), where it was saved, its
  size, when the download started and ended, and whether it completed.
- **Cookies**: the sites that set them, when they were set, last sent and expire.
- **Form history**: values typed in web forms and search boxes, how often, first and last.

Browser activity is often the most direct record of what a user did: the phishing page they
followed, the tool they downloaded, the webmail they used, the search terms they typed.

<span class="evidence-strength">Best conclusion:</span> the browser profile recorded a visit to the
URL at the recorded time, or saved the download to the recorded path; the database does not say
who sat at the keyboard.

## Location and collection

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

`<Profile>` is `Default`, `Profile 1`, `Profile 2` and so on. Chromium moved `Cookies` into
`Network` in version 96; older versions keep it beside `History`. Firefox up to version 25 kept its
downloads in `downloads.sqlite`; later versions keep them in `places.sqlite`.

Collect every file of the profile, and each database with the files beside it that share its
name: `places.sqlite-wal`, `cookies.sqlite-wal`, `formhistory.sqlite-wal`, `History-journal`.

!!! warning "Write-ahead logs"
    Firefox keeps its databases in write-ahead log mode: the newest changes are only in the
    `-wal` file until the browser copies them into the database (a checkpoint). Juntei reads each
    database with its `-wal` file, as the browser would. Without it, recent visits, cookies and
    form entries are missing, and entries deleted in the log still read as live.

## Deleted entries

When a browser deletes history, the rows leave the tables but their records can stay in the
database's free space, on its freelist, or in older versions of the database's pages: in the
`-wal` file, or in the database file under a `-wal` that replaced them. Juntei recovers deleted
visits, pages, downloads and form entries from them, as rows whose `Status` is `Deleted`:

- `Confidence` says how sure the match of the record with its table is: `High` for a whole record
  whose every value has its column's type, `Medium` or `Low` for damaged records or records that
  fit several tables.
- `RecoveredFrom` says where the record was: the area of the page (a cell, a freeblock, unallocated
  space, a free page), which page and which version of it, and the record's offset in the page.
  WAL frames count from 0.
- `LostColumns` lists the columns a damaged record no longer holds: their fields are empty.

Browsers delete with SQLite's `secure_delete`: freed space in the database is zeroed. What can be
recovered is then mostly in older page versions: those the `-wal` file still holds, and the
database file's own copies of the pages a `-wal` replaced, until the next checkpoint. Copy the
`-wal` file with its database, and do it before the browser runs again.

## Analyst cautions

- Times are in UTC. Chromium and Firefox record them to the microsecond; Chromium's form history
  to the second.
- A visit does not mean the user typed or clicked: redirects, frames and pages opened by other
  programs are visits too. Read `Transition`, `Typed` and `Hidden`.
- Chromium encrypts cookie values: `Value` is empty for them. Juntei does not decrypt.
- A `Removed` form-history row comes from Firefox's `moz_deleted_formhistory`, which Firefox for
  Android fills for Sync. Desktop Firefox normally leaves it empty.
- A deleted entry with `Low` confidence may be a fragment of another record: check it against the
  live rows and the other evidence.
- No deleted row does not mean nothing was deleted: zeroed space and a checkpoint leave nothing
  to recover. Deleted cookies are not recovered yet.
- `Browser`, `User` and `Profile` come from the database's path: a profile copied elsewhere keeps
  only what its path says. Without a known browser folder, `Browser` is `Chromium-based` or
  `Firefox-based`.
- Rollback journals (`-journal`) are not read, nor Internet Explorer and legacy Edge's
  `WebCacheV01.dat`.

## Parse with Juntei

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

`-d` looks for the files by name and tells each by its tables: a `History` file of another program
is skipped. `-f` reads a database whatever its name.

## Output tables

CSV creates four files, each with only its header when it has no rows. JSON and JSONL rows carry
`Table` (`History`, `Downloads`, `Cookies` or `FormHistory`) after `SourceFile`.

| CSV file | Contents |
|---|---|
| `Browser-History.csv` | Visits, live and deleted; deleted pages whose URL no live page has. |
| `Browser-Downloads.csv` | Downloads, live and deleted. |
| `Browser-Cookies.csv` | Cookies. |
| `Browser-FormHistory.csv` | Form entries, live and deleted; Firefox's removal records. |

## Fields

Every table starts with these fields:

| Field | Interpretation |
|---|---|
| `SourceFile` | The database the row was read from. |
| `User` | The Windows user whose profile the path is in (the folder after `Users`). |
| `Browser` | The browser the path names: `Chrome`, `Edge`, `Brave`, `Opera`, `Vivaldi`, `Firefox` … |
| `Profile` | The browser profile: the database's folder. |
| `Status` | `Live`, `Deleted` (recovered), or `Removed` (a Firefox removal record). |

### History

| Field | Interpretation |
|---|---|
| `RecordType` | `Visit`, or `Page` for a deleted page whose visits may be gone too. |
| `VisitTime` | When the visit began (UTC). |
| `URL`, `Title` | The page and its title, as last seen. |
| `Transition` | How the browser came to the page. Chromium: the core type then its qualifiers (`LINK`, `TYPED`, `AUTO_BOOKMARK`, `FORM_SUBMIT`, `RELOAD`, … with `CHAIN_START`, `SERVER_REDIRECT`, `FROM_ADDRESS_BAR`, …); Firefox: `LINK`, `TYPED`, `BOOKMARK`, `EMBED`, `REDIRECT_PERMANENT`, `REDIRECT_TEMPORARY`, `DOWNLOAD`, `FRAMED_LINK`, `RELOAD`. |
| `VisitDuration` | Seconds the page was in front (Chromium; 0 when not measured). |
| `VisitCount` | Visits to the page in all. |
| `Typed`, `TypedCount` | Whether the URL was ever typed, and how many times (Chromium). |
| `Hidden` | Hidden from the history list: subframes, redirect sources, errors. |
| `Frecency` | Firefox's ranking of the page for address bar suggestions. |
| `VisitID`, `FromVisitID` | The visit's row id, and that of the visit it came from (the referring page, the redirect's source). |
| `LastVisitTime` | A deleted page's last visit (UTC). |
| `PageSource` | Where a deleted visit's URL and title come from: `Live` (a live page), `Recovered` (a deleted page) or `NotFound`. |
| `Confidence`, `RecoveredFrom`, `LostColumns` | How a deleted entry was recovered; see [Deleted entries](#deleted-entries). |

### Downloads

| Field | Interpretation |
|---|---|
| `StartTime`, `EndTime` | When the download began and ended (UTC). |
| `URL`, `RedirectChain` | Where the file came from, and every URL on the way (Chromium; ` -> ` between them in CSV). |
| `TargetPath`, `CurrentPath` | Where the file was saved (a `file://` URI in Firefox), and where it was written while downloading. |
| `ReceivedBytes`, `TotalBytes` | Bytes received, and the file's size. |
| `State` | `Complete`, `InProgress`, `Cancelled`, `Interrupted`, `Paused`, `Blocked` or `Dirty` (blocked as malware). |
| `DangerType` | Chromium's verdict on the file: 0 not dangerous, 1 dangerous file type, 2 dangerous URL, 3 dangerous content, 4 maybe dangerous, 5 uncommon, 6 kept by the user, 7 dangerous host, 8 potentially unwanted, … |
| `InterruptReason` | Why an interrupted download stopped (Chromium; 0 for none). |
| `Referrer`, `TabURL` | The referring page, and the page in the tab that started the download. |
| `MimeType`, `Opened` | The file's type as served, and whether it was opened from the browser (Chromium). |
| `FileDeletedByUser` | Whether the user deleted the downloaded file from disk through the browser's download list (Firefox). |
| `DownloadID` | The download's row id. |

### Cookies

| Field | Interpretation |
|---|---|
| `Created`, `LastAccessed`, `Expires` | When the cookie was set, last sent, and expires (UTC; empty `Expires` for a session cookie). |
| `Host`, `Name`, `Path` | The site the cookie is for, its name and path. |
| `Value` | The value as stored: empty when Chromium encrypted it. |
| `Secure`, `HttpOnly`, `Persistent` | Sent over HTTPS only, hidden from scripts, kept across sessions (Chromium). |
| `GoogleAnalytics` | A Google Analytics cookie's decoded fields (`__utma`, `__utmb`, `__utmt`, `__utmz`): visit times, sessions, and the campaign source and search terms. |
| `RowID` | The cookie's row id. |

### Form history

| Field | Interpretation |
|---|---|
| `FirstUsed`, `LastUsed` | When the value was first and last entered (UTC). |
| `RemovedTime` | When a Firefox removal record says the entry was removed (UTC). |
| `FieldName`, `Value` | The form field (`email`, `q`, Firefox's search bar: `searchbar-history`) and what was typed. |
| `Count` | How many times the value was entered. |
| `GUID` | Firefox's identifier of the entry, which removal records name. |
| `Sources` | Where Firefox saved the entry from, in the versions that keep it (separated by a vertical bar in CSV). |
| `RowID` | The entry's row id. |

## Automatic parsing

`auto-parse` writes the tables to `BrowserActivity` as `Browser-History.csv`,
`Browser-Downloads.csv`, `Browser-Cookies.csv` and `Browser-FormHistory.csv` (one `Browser.json` or
`Browser.jsonl` file with `-t json` or `-t jsonl`), with `BrowserActivity/Browser-logs.txt`. The
combined `-j` timeline includes every row, with `Comp` and the `Artifact` identifier `browser`. `-Z`
converts `VisitTime`, `LastVisitTime`, `StartTime`, `EndTime`, `Created`, `LastAccessed`, `Expires`,
`FirstUsed`, `LastUsed` and `RemovedTime`.

## Further Details

- [Chromium — page transition types](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — download danger types](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — Places database](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — write-ahead logging](https://www.sqlite.org/wal.html)
- [Plaso — browser SQLite parsers](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
