# 檔案命令 { #file-commands }

`parse-file` 處理特定跡證的檔案：Prefetch、捷徑、Jump Lists、Defender 隔離資料，以及
Outlook 的 PST、OST、MSG 與 OFT 檔案。自動 Jump Lists 使用複合檔案容器；自訂 Jump Lists 則使用不同的
二進位配置。兩者都屬於同一個跡證命令。

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file <COMMAND> [OPTIONS]

Commands:
  defender-quarantine  Parse Defender quarantine
  help                 Print this message or the help of the given subcommand(s)
  jump-lists           Parse Jump Lists
  lnk                  Parse shortcuts
  outlook              Parse Outlook PST, OST, MSG and OFT files
  prefetch             Parse Prefetch files

Options:
  -h, --help  Print help
```

## `parse-file defender-quarantine` { #parse-file-defender-quarantine }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Defender quarantine folders (a volume, a Quarantine folder, a collection of hosts)
  -f, --file <FILE>      File path to one quarantine entry (Quarantine\Entries\{GUID})

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. Without -o, the records are written to stdout
      --extract <DIR>       Write quarantined files (likely malware, never run) with a .quarantined extension
  -C, --clobber             Overwrite files when saving or extracting

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract isolated-output/
```

擷取時會保留二進位的安全性資料流，並另外建立可讀的 `.security_descriptor.txt` 報告，內含
擁有者、權限與 SDDL。請參閱 [Defender 隔離資料](../artifacts/defender-quarantine.md)。

## `parse-file jump-lists` { #parse-file-jump-lists }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for jump lists (*.automaticDestinations-ms, *.customDestinations-ms)
  -f, --file <FILE>      File path to one jump list
      --code-page <CP>   Code page of text in links that isn't Unicode: the system's ANSI code page (932 for Japanese) [default: 1252]

Output:
  -o, --output <PATH>       Save every entry: a directory of CSV tables, or a file (json, jsonl)
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. Without -o, json and jsonl are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-lists
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

請參閱 [Jump Lists](../artifacts/jump-lists.md)。

## `parse-file lnk` { #parse-file-lnk }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for shortcuts (*.lnk)
  -f, --file <FILE>      File path to one shortcut
      --code-page <CP>   Code page of text that isn't Unicode: the system's ANSI code page (932 for Japanese) [default: 1252]

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. Without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

請參閱 [LNK 捷徑](../artifacts/lnk.md)。

## `parse-file outlook` { #parse-file-outlook }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file outlook <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Outlook files (*.pst, *.ost, *.msg, *.oft)
  -f, --file <FILE>      File path to one PST, OST, MSG or OFT file (told apart by its contents, whatever its name)
      --code-page <CP>   Code page of text that isn't Unicode, for items that don't give theirs: the system's ANSI code page (932 for Japanese) [default: 1252]

Output:
  -o, --output <PATH>              Save every row: a directory of CSV tables, or a file (json, jsonl)
  -t, --output-type <TYPE>         Output type: csv (default), json, jsonl. Without -o, json and jsonl rows are written to stdout
      --properties                 Also write every property of the stores, folders, items, recipients and attachments (the Properties table)
      --extract-attachments <DIR>  Write each attachment's data once to DIR, named by its SHA-256 (attachments can be malware: never run them)
  -C, --clobber                    Overwrite files when saving or extracting

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-file outlook -d Users -o outlook-csv
juntei parse-file outlook -f mailbox.ost -t jsonl | jq 'select(.Table == "Items")'
juntei parse-file outlook -f invoice.msg -o msg-csv --properties --extract-attachments isolated-output/
```

這些資料表包括讀取的檔案、其資料夾、每一個項目（含資料夾的隱藏項目與內嵌於附件中的項目）、
收件者、附件、連絡人、行事曆項目與工作；使用 `--properties` 時，另含依原樣儲存的每一個屬性。
請參閱 [Outlook](../artifacts/outlook.md)。

## `parse-file prefetch` { #parse-file-prefetch }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for prefetch files (*.pf)
  -f, --file <FILE>      File path to one prefetch file

Records:
  -T, --timeline  One record per execution (the last 8 run times of every file), sorted by time

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
juntei parse-file prefetch -d evidence/ -T -o prefetch-timeline.csv
```

請參閱 [Prefetch](../artifacts/prefetch.md)。
