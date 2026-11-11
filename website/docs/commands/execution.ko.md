# 파일 명령어 { #file-commands }

`parse-file`는 아티팩트별 파일을 처리합니다: Prefetch, 바로 가기, Jump Lists, Defender
격리 데이터, 그리고 Outlook의 PST, OST, MSG 및 OFT 파일. 자동(Automatic) Jump Lists는 복합 파일(compound file) 컨테이너를 사용하고, 사용자 지정(Custom)
Jump Lists는 다른 바이너리 레이아웃을 사용합니다. 두 가지 모두 같은 아티팩트 명령어에 속합니다.

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
Version: 1.1.0 (CODE BLUE Release)
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

추출 시 바이너리 보안 스트림을 보존하며, 소유권, 권한 및 SDDL을 담은 읽기 쉬운 `.security_descriptor.txt`
보고서도 생성합니다. [Defender 격리 데이터](../artifacts/defender-quarantine.md)를 참조하십시오.

## `parse-file jump-lists` { #parse-file-jump-lists }

```text
Version: 1.1.0 (CODE BLUE Release)
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

[Jump Lists](../artifacts/jump-lists.md)를 참조하십시오.

## `parse-file lnk` { #parse-file-lnk }

```text
Version: 1.1.0 (CODE BLUE Release)
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

[LNK 바로 가기](../artifacts/lnk.md)를 참조하십시오.

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

테이블은 읽은 파일, 그 폴더, 모든 항목(폴더의 숨겨진 항목과 첨부 파일에 포함된 항목 포함), 받는 사람,
첨부 파일, 연락처, 일정 항목, 작업이며, `--properties`를 사용하면 저장된 그대로의 모든 속성도 포함됩니다.
[Outlook](../artifacts/outlook.md)을 참조하십시오.

## `parse-file prefetch` { #parse-file-prefetch }

```text
Version: 1.1.0 (CODE BLUE Release)
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

[Prefetch](../artifacts/prefetch.md)를 참조하십시오.
