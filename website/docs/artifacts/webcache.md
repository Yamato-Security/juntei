# Internet Explorer and Legacy Edge

## What the WebCache database is

`WebCacheV01.dat` is the database of the Windows Internet functions (WinINet) in each user's
profile, an ESE database. Internet Explorer 10 and 11 keep their history, downloads, cache and
cookies in it, and so does legacy Edge, the Edge of Windows 10 before Edge became Chromium-based.
Each kind of record is a container, a table of its own:

- **History** (`History`, and `MSHist01…` for a day or a week): the URLs visited, each with the
  account it was recorded for, when it was last visited and how often. File Explorer records the
  local files it opens there too, as `file:///` URLs.
- **Downloads** (`iedownload`): the URL a file came from, where it was saved and written while
  downloading, its size, when the download started, the referring page and the MIME type.
- **Cookies**: in Windows 10 and later, the cookies themselves (`CookieEntryEx` tables), with their
  host, name, value and times; before, the cookie files that held them.
- **Cache** (`Content`): the files downloaded to show pages, each with its URL, where the copy is
  on disk, and the HTTP response headers. Other programs that use WinINet cache their downloads
  here too.
- Web storage (`DOMStore`), compatibility lists (`iecompat`) and others.

The database outlives the browsers: Internet Explorer is retired and Chromium-based Edge keeps its
own SQLite databases (see [Web Browsers](browser.md)), but the database is still in every profile,
with what was recorded before, and what File Explorer and other programs still record.

<span class="evidence-strength">Best conclusion:</span> the user's profile recorded the URL, the
download or the cached file at the recorded time; the database does not say who sat at the
keyboard, or that a cached file was opened.

## Location and collection

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

The files the containers name (cached files, cookie files) are in their own folders:
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\`, `INetCookies\`, `History\`, and under
`%LOCALAPPDATA%\Packages\` for apps.

Collect the whole `WebCache` folder. Windows keeps the database open while the user is logged on,
so a copy from a running system is usually dirty: its newest changes are only in the `V01*.log`
transaction logs.

!!! warning "Transaction logs"
    Juntei does not replay ESE transaction logs. A database that wasn't shut down cleanly is
    counted in the summary as left dirty and reported as a problem: changes still only in its logs
    are missing. To include them, replay the logs into a copy of the folder first, with Windows'
    `esentutl /r V01 /d` run in that copy.

## Analyst cautions

- Times are in UTC, to the 100 nanoseconds.
- A history entry is a URL, not a visit: `AccessedTime` is its last visit and `AccessCount` how
  many there were. The `History` container keeps one entry per URL; a day's or a week's
  container keeps those of that period, its name giving its first day and the day after its last
  (`MSHist012026110120261102`: November 1, 2026).
- `file:///` URLs in the history are local files opened from File Explorer and other programs, not
  web pages.
- `Account` is the account an entry was recorded for (`Visited: alice@…`); `User` is the profile
  folder the database's path is in. They can differ: a profile folder isn't always named after its
  account, and a database copied elsewhere has no profile folder in its path.
- `Package` says whose an entry is: legacy Edge's (`microsoft.microsoftedge_8wekyb3d8bbwe`),
  Internet Explorer's in Enhanced Protected Mode (`windows_ie_ac_001`) or another app's. Empty, it
  is the user's own: Internet Explorer, File Explorer and other programs.
- Microsoft doesn't document the download record. Juntei reads its start time and size only when
  its header fits within the record, and its paths and other strings only when the last two are a URL and a
  Windows path. Otherwise the row has the entry's own URL, and its access time when the start
  can't be read, with a problem. `Publisher` is the name the browser recorded, not a check of the
  file's signature.
- A cache entry says the file was downloaded and cached. The copy on disk can be gone, replaced or
  never opened.
- Deleted entries are not recovered yet: ESE can keep a deleted record in its pages until it
  cleans them up, but Juntei reads only live ones.

## Parse with Juntei

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

`-d` looks for `WebCacheV01.dat` by name; `-f` reads a database whatever its name.

## Output tables

CSV creates five files, each with only its header when it has no rows. JSON and JSONL rows carry
`Table` (`History`, `Downloads`, `Cookies`, `Entries` or `Containers`) after `SourceFile`.

| CSV file | Contents |
|---|---|
| `WebCache-History.csv` | The entries of the history containers. |
| `WebCache-Downloads.csv` | Downloads. |
| `WebCache-Cookies.csv` | Cookies (Windows 10 and later). |
| `WebCache-Entries.csv` | The entries of every other container: cached files, cookie files, web storage, compatibility lists and the rest. |
| `WebCache-Containers.csv` | The containers. |

## Fields

Every table starts with these fields:

| Field | Interpretation |
|---|---|
| `SourceFile` | The database the row was read from. |
| `User` | The Windows user whose profile the path is in (the folder after `Users`). |
| `Package` | The app whose data the row is: the folder under `Packages` of its container's directory (or its cookie table's). Empty for the user's own. |

### History

| Field | Interpretation |
|---|---|
| `Container` | The container: `History`, or `MSHist01` followed by the first day of its period and the day after its last. |
| `AccessedTime` | The URL's last visit (UTC); in a day's or a week's container, the last of that period. |
| `Account` | The account the entry was recorded for. |
| `URL` | The page or file, without the `Visited: account@` prefix. |
| `AccessCount` | How many times the URL was visited. |
| `EntryID`, `ContainerID` | The entry's id in its container, and the container's id. |

### Downloads

| Field | Interpretation |
|---|---|
| `StartTime` | When the download started (UTC). |
| `URL` | Where the file came from. |
| `TargetPath`, `CurrentPath` | Where the file was saved, and where it was written while downloading: a `.partial` file for legacy Edge, the cache's copy for Internet Explorer. |
| `TotalBytes` | The file's size. |
| `Referrer` | The referring page. |
| `MimeType` | The file's type as served (Internet Explorer). |
| `Publisher` | The name legacy Edge recorded for the file's publisher. |
| `AccessedTime` | The entry's own access time (UTC), most often the start. |
| `DownloadID`, `ContainerID` | The entry's id in its container, and the container's id. |

### Cookies

| Field | Interpretation |
|---|---|
| `LastModified`, `Expires` | When the cookie was set, and when it expires (UTC). |
| `Host`, `Name`, `Path` | The site the cookie is for, its name and path. |
| `Value` | The value as stored. |
| `Flags` | The cookie's flags, as stored. |
| `Directory` | The directory the database gives the cookie table: whose cookies they are (`…\INetCookies\ESE\`, legacy Edge's `…\MicrosoftEdge\Cookies\ESE\`, an app's). |
| `CookieTable`, `EntryID` | The table (`CookieEntryEx_<n>`) and the cookie's id in it. |

### Entries

| Field | Interpretation |
|---|---|
| `Container` | The container: `Content` (cached files), `Cookies` (cookie files), `DOMStore` (web storage), `iecompat`, …. |
| `AccessedTime` | When the entry was last used (UTC). |
| `ModifiedTime` | The resource's last modification, as the server gave it (UTC). |
| `CreationTime` | When the entry was made (UTC). |
| `ExpiryTime`, `SyncTime` | When it expires, and when it was last checked with the server (UTC). |
| `URL` | The URL as stored: a URL for a cached file, `Cookie:alice@example.com/` for a cookie file, `DOMStore:…`, `iecompat:…`. |
| `Account` | The account of a cookie file. |
| `Filename`, `FilePath` | The file of the entry, and where it is: the container's directory and the subfolder the entry names. |
| `FileSize` | The file's size. |
| `AccessCount`, `SyncCount` | How many times the entry was used, and checked with the server. |
| `EntryType`, `Flags` | The entry's type and flags, as stored. |
| `RedirectURL` | Where the URL redirected to. |
| `ResponseHeaders`, `RequestHeaders` | The HTTP response headers (`HTTP/1.1 200 OK`, `Content-Type`, `Server`, …) and request headers, as stored, their lines and all. |
| `EntryID`, `ContainerID` | The entry's id in its container, and the container's id. |

### Containers

| Field | Interpretation |
|---|---|
| `ContainerID`, `Name` | The container's id and name. |
| `Directory` | Where its files are on disk. |
| `SecureDirectories` | The subfolders of `Directory` its files are in (a comma between them in CSV). |
| `PartitionID` | `M` for the user's own containers, `L` for those of Internet Explorer's Protected Mode (the `Low` folders), an app container's SID for an app's. |
| `LastAccessTime`, `LastScavengeTime` | When the container was last used, and last cleaned up (UTC). |

## Automatic parsing

`auto-parse` writes the tables to `BrowserActivity` as `WebCache-History.csv`,
`WebCache-Downloads.csv`, `WebCache-Cookies.csv`, `WebCache-Entries.csv` and
`WebCache-Containers.csv` (one `WebCache.json` or `WebCache.jsonl` file with `-t json` or
`-t jsonl`), with `BrowserActivity/WebCache-logs.txt`. The combined `-j` timeline includes every
row, with `Comp` and the `Artifact` identifier `webcache`. `-Z` converts every time: the fields
ending in `Time`, `LastModified` and `Expires`.

## Further Details

- [Microsoft — About WinINet](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE database file format](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — MSIE WebCache parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
