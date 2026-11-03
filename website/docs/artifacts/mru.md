# Most-Recently-Used Lists (RunMRU, TypedPaths, WordWheelQuery, TypedURLs)

## What the lists are

Windows and its programs keep short lists of what a user typed or opened last, in the user's
`NTUSER.DAT`:

- **RunMRU:** commands typed into the Run dialog (Win+R): `cmd`, `powershell -ep bypass`,
  `\\server\share`, `regedit`.
- **TypedPaths:** paths typed into Explorer's address bar.
- **WordWheelQuery:** terms searched for in Explorer's search box.
- **TypedURLs:** addresses typed into Internet Explorer's address bar, and since Windows 8
  `TypedURLsTime`, when each was typed.
- **Paint and WordPad:** the files each opened last (`Recent File List`).
- **ACMru (Windows XP):** the Search Assistant's terms, per kind of search.

<span class="evidence-strength">Best conclusion:</span> the user typed this command, path,
search term or address, or the program opened this file. Only the most recent entry of each list
is dated, by its key's last-write time, apart from typed addresses, which `TypedURLsTime` dates.

## Location

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

`ACMru`'s subkeys are the kinds of search: `5603` file and folder names, `5604` words or phrases in
files, `5647` computers and people, `5001` the Internet.

Collect `NTUSER.DAT` with its transaction logs for every profile.

## Analyst cautions

- **One time per list.** A key's last-write time changes when its list changes, so it dates the
  most recent entry only (`IsMostRecent`, `LastUsedUTC`). The others are in order, undated.
  `TypedURLsTime` dates each typed address (`TypedTimeUTC`).
- **Typed, not run.** RunMRU records what was typed into the Run dialog, whether or not it ran
  successfully. Windows stores each command with a trailing `\1`; Juntei removes it. Correlate
  with Prefetch, BAM, UserAssist and process creation events.
- **Lists are short.** Each keeps only its most recent entries, and privacy settings and cleanup
  tools clear them. Use `parse-registry read --recover-deleted` for deleted values.
- **Terms are what was typed.** WordWheelQuery and ACMru keep the text typed, not what was found.
- **Order comes from the list.** RunMRU's `MRUList` and WordWheelQuery's `MRUListEx` give the
  order; for the others, the value's number does (`url1`, `File1` and `000` are the most recent).

## Parse with Juntei

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

The summary lists the entries per list and the entries dated last (typed addresses, and each
list's most recent entry), newest first.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, the profile owner (from the path or the shell folders), the SID from the system's `ProfileList`, and the hive. |
| `List` | `RunMRU`, `TypedPaths`, `WordWheelQuery`, `TypedURLs`, `Paint`, `Wordpad` or `ACMru`. |
| `Subkey` | `ACMru`'s subkey: the kind of search (`5603`, `5604`, `5647`, `5001`). |
| `MRU-Position`, `IsMostRecent` | The place in the list (0 is the most recent); empty when the list doesn't name the value. |
| `ValueName` | `a`, `0`, `url1`, `File1`, `000`, … |
| `Value` | The command (without its `\1`), path, search term, address or file. |
| `TypedTimeUTC` | When the address was typed, from `TypedURLsTime`. |
| `LastUsedUTC` | The key's last-write time, for the most recent entry only. |
| `KeyPath`, `KeyLastWrittenUTC` | The key and its last write: the time of its most recent entry, not of every row. |
| `DataRaw` | The value's bytes (hex) when they don't decode. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Values the list names but the key hasn't, values it doesn't name, misnamed values, wrong types, terms cut short, times that aren't FILETIMEs. |

## Investigation strategy

1. Read RunMRU for commands, scripts, network shares and administration tools typed by the user.
2. Read TypedPaths and WordWheelQuery for folders and files the user went looking for:
   credentials, exports, archives, removable and network locations.
3. Put typed addresses on the timeline with `TypedTimeUTC`.
4. Correlate with Prefetch, BAM, UserAssist, ShellBags, LNK files, jump lists and browser history.

## Further Details

- [libyal winreg-kb: Most recently used (MRU) keys](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
