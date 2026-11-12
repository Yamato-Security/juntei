# PowerShell History

## What the history records

PSReadLine saves command lines entered in an interactive PowerShell host. The file contains
text in saved order, with no per-command timestamps. History can be edited, cleared, disabled
or filtered, and does not establish that a command completed successfully. Corroborate execution
and timing with PowerShell event logs, process telemetry and other execution artifacts.

## Location and collection

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

## Parse with Juntei

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

CSV is the default file format. With `-t csv`, `-t json` or `-t jsonl` and no `-o`, records go to
stdout. Without either option, only a summary is displayed. JSON consists of separate
pretty-printed objects; JSONL has one object per line. Add `-C` to overwrite an existing output;
source-file aliases are protected from replacement.

`auto-parse` saves `ShellHistory/PowerShellHistory.csv` (or the selected JSON format), also for
VHDX/E01 images. History does not contribute events to `auto-parse -j` or `super-timeline`.

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
- PSReadLine stores multiline entries with a trailing backtick on each continued physical line.
  Lines stay separate, with backticks and indentation preserved. `EntryStartLine` points to the
  first line of a consecutive chain whose preceding lines end in a backtick. No shell syntax is
  interpreted and no commands are reconstructed. Truncation can affect continuation attribution;
  inspect `Problems` before grouping such rows.

## Output fields

| Field | Interpretation |
|---|---|
| `SourceFile` | Path relative to directory input, using `/`; for `-f`, only the filename. No absolute source path is exported. |
| `User` | Profile name inferred from the retained path. Directory input also recognizes `Users`/`Documents and Settings` layouts below the input root; `-f` requires the usual PSReadLine profile suffix. Unknown values are empty in CSV and `null` in JSON. |
| `ShellHost` | Filename prefix before `_history.txt`, such as `ConsoleHost`; unknown for renamed files without that suffix. |
| `LineNumber` | Physical line number starting at 1, including skipped empty lines. |
| `EntryStartLine` | Starting physical line of the continuation chain, or this line's own number. |
| `Command` | Decoded physical line without its line ending, subject to the documented limit and CSV guard. |
| `FileLastWriteTime` | Source file modification time, repeated on each row. **File metadata, never command execution time.** Image parsing uses original source metadata, not staging time. |
| `FileSize` | Source file size in bytes. |
| `Encoding` | `UTF-8`, `UTF-8-BOM`, `UTF-16LE` or `UTF-16BE`. |
| `Problems` | Line damage and truncation diagnostics; semicolon-separated in CSV, an array in JSON/JSONL. File-level diagnostics also appear in the log. |

`FileLastWriteTime` defaults to UTC in ISO 8601 form, for example `2026-10-20T08:46:38.5Z`.
Only `auto-parse` accepts `-Z, --timezone` to change the displayed offset. The standalone
`parse-file powershell-history` command has no timezone option.

Every CSV text cell is checked for spreadsheet formula prefixes (`=`, `+`, `-`, `@`, including
full-width forms and prefixes after whitespace or certain invisible/control characters).
Affected cells, and cells starting with control characters, receive a leading apostrophe.
This can change command and path text in CSV. JSON and JSONL retain the decoded text unchanged
without the CSV apostrophe; decoding replacement and truncation still apply. Use JSONL when
comparing saved command text, and always preserve the source history files.
