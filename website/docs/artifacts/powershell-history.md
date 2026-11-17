# PowerShell History

## What the history records

PSReadLine saves command lines entered in an interactive PowerShell host. The file contains
text in saved order, with no per-command timestamps.

## Locations and collection

The default Windows location for Windows PowerShell 5.1 and PowerShell 7+ is:

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

Other hosts use names such as `Visual Studio Code Host_history.txt` in that directory.
PowerShell on Linux and macOS normally uses
`~/.local/share/powershell/PSReadLine/<Host>_history.txt`. PSReadLine's `HistorySavePath`
can redirect the file; these default paths are discovery assumptions, not exhaustive locations.

Directory input and `auto-parse` recursively select `<Host>_history.txt` directly inside a
`PSReadLine` folder, matching names without regard to ASCII case. They do not select arbitrary
`*_history.txt` files elsewhere. Use explicit `-f` input for a renamed file or a copy outside a
`PSReadLine` directory. Preserve the original profile layout when possible for user attribution.

## Analyst cautions

- No line has a time. `FileLastWriteTime` is the file's modification time. PSReadLine saves
  history after each command by default (`HistorySaveStyle` `SaveIncrementally`), so it is
  usually close to when the last line was added, but it says nothing about the earlier lines.
- A saved line does not establish that a command ran or completed successfully. Corroborate
  execution and timing with PowerShell event logs, process telemetry and other execution
  artifacts.
- History can be edited, deleted or turned off (`HistorySaveStyle` `SaveNothing`). PSReadLine
  also leaves out lines that look like they contain secrets, such as those with `password`,
  `token` or `apikey`, and the Windows PowerShell ISE doesn't use PSReadLine. Scripts and
  commands run with `-Command` or `-EncodedCommand` are not typed, so they are not saved.
- A command of several lines is saved with a backtick at the end of every line but its last.
  Juntei keeps those lines as separate rows linked by `EntryStartLine`. A line that ends with a
  typed backtick looks the same, so read the lines of an entry together.

## Parse with Juntei

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

CSV is the default file format. With `-t csv`, `-t json` or `-t jsonl` and no `-o`, records go to
stdout. Without either option, only a summary is displayed. JSON consists of separate
pretty-printed objects; JSONL has one object per line. Add `-C` to overwrite an existing output;
source-file aliases are protected from replacement.

## Encoding and line handling

- A UTF-8, UTF-16LE or UTF-16BE byte order mark selects the encoding and is removed at the
  beginning of the file. Without a BOM, input is always treated as UTF-8. BOM-less UTF-16 is
  not guessed; NUL characters are retained and reported as possible encoding damage.
- Invalid UTF-8 sequences and unpaired UTF-16 surrogates become U+FFFD, with damage reported
  in `Problems`. An odd trailing UTF-16 byte and read errors are also reported; readable rows
  are retained. Inspect both `Problems` and the parser log before relying on completeness.
- Files are streamed. At most 1,048,576 input bytes per physical line are retained, excluding
  the BOM and LF delimiter (a CR before LF counts toward the limit). Longer lines are cut at
  a whole-character boundary, the remaining bytes are consumed, and `Problems` reports how
  many bytes were omitted. The limit applies to encoded input bytes, not character count.
- LF and CRLF line endings are removed. A final CR is also removed from an unterminated line.
  Other spaces and tabs are retained. Empty lines produce no row but count toward line numbers
  and end a continuation chain; whitespace-only lines remain rows. Read errors may produce an
  empty diagnostic row to mark where reading stopped.
- Lines stay separate, with backticks and indentation preserved. `EntryStartLine` points to the
  first line of a consecutive chain whose preceding lines end in a backtick. No shell syntax is
  interpreted and no commands are reconstructed. Truncation can affect continuation attribution;
  inspect `Problems` before grouping such rows.

## CSV output fields

| Field | Interpretation |
|---|---|
| `SourceFile` | Path relative to directory input, using `/`; for `-f`, only the filename. No absolute source path is exported. |
| `User` | Profile name from the path. The folder above `AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine` (or `.local/share/powershell/PSReadLine`) names the user, also when the input is that profile or the `PSReadLine` folder itself. Directory input also recognizes `Users`/`Documents and Settings` layouts below the input root. Unknown values are empty in CSV and `null` in JSON. |
| `ShellHost` | Filename prefix before `_history.txt`, such as `ConsoleHost`; unknown for renamed files without that suffix. |
| `LineNumber` | Physical line number starting at 1, including skipped empty lines. |
| `EntryStartLine` | Starting physical line of the continuation chain, or this line's own number. |
| `Command` | Decoded physical line without its line ending, subject to the documented limit. |
| `FileLastWriteTime` | Source file modification time, repeated on each row. **File metadata, never command execution time.** Image parsing uses original source metadata, not staging time. |
| `FileSize` | Source file size in bytes. |
| `Encoding` | `UTF-8`, `UTF-8-BOM`, `UTF-16LE` or `UTF-16BE`. |
| `Problems` | Line damage and truncation diagnostics; semicolon-separated in CSV, an array in JSON/JSONL. File-level diagnostics also appear in the log. |

JSON and JSONL have the same fields, with numbers typed, `null` for unknown values and
`Problems` as a list. Every format keeps the command text as saved.

`FileLastWriteTime` defaults to UTC in ISO 8601 form, for example `2026-10-20T08:46:38.5Z`.
Only `auto-parse` accepts `-Z, --timezone` to change the displayed offset. The standalone
`parse-file powershell-history` command has no timezone option.

## Automatic parsing

`auto-parse` writes `ShellHistory/PowerShellHistory.csv` (`PowerShellHistory.json` or
`PowerShellHistory.jsonl` with `-t json` or `-t jsonl`), with
`ShellHistory/PowerShellHistory-logs.txt`. From VHDX and E01 images, the history files are
copied, and `FileLastWriteTime` is the time recorded in the image. The combined `-j` timeline
includes every row with the `Artifact` identifier `powershell-history`. The lines have no times,
so [`super-timeline`](../commands/super-timeline.md) makes no events from them and counts them as
untimed records.

## Investigation strategy

1. Read every profile's history, including `Windows\System32\config\systemprofile` and the
   profiles under `Windows\ServiceProfiles`, which hold commands typed in shells running as those
   accounts.
2. Read multiline entries together, and look for downloads, encoded commands, changes to security
   settings, account and credential commands, and remote connections.
3. Corroborate with the PowerShell event logs (script block logging, event 4104), process
   creation events, and the Prefetch files of `powershell.exe` and `pwsh.exe`.
4. Compare `FileLastWriteTime` with the history file's `$MFT` and USN Journal times, and with
   logons, to bound when the last lines were added.

## Further Details

- [Microsoft — about_PSReadLine: command history](https://learn.microsoft.com/en-us/powershell/module/psreadline/about/about_psreadline)
- [Microsoft — Set-PSReadLineOption: HistorySavePath and HistorySaveStyle](https://learn.microsoft.com/en-us/powershell/module/psreadline/set-psreadlineoption)
- [Microsoft — about_Logging_Windows: script block logging](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [PSReadLine — how the history file is saved and read](https://github.com/PowerShell/PSReadLine/blob/master/PSReadLine/History.cs)
