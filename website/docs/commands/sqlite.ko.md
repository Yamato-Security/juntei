# SQLite 명령어 { #sqlite-commands }

`parse-sqlite`는 SQLite 데이터베이스를 처리합니다. 웹 브라우저는 방문 기록, 다운로드, 쿠키, 양식 입력 항목을
SQLite 데이터베이스에 저장합니다. Chromium 기반 브라우저의 `History`, `Cookies`, `Web Data`와 Firefox의
`places.sqlite`, `cookies.sqlite`, `formhistory.sqlite`가 이에 해당합니다. Juntei는 SQLite 없이 이들을 읽으며,
`-wal` 파일의 최신 변경 사항까지 반영하고 삭제된 항목을 복구합니다.

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-sqlite <COMMAND> [OPTIONS]

Commands:
  browser  Parse browser history, downloads, cookies and form history
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## `parse-sqlite browser` { #parse-sqlite-browser }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for browser databases (History, Cookies, Web Data, places.sqlite, cookies.sqlite, formhistory.sqlite, ...)
  -f, --file <FILE>      File path to one browser database (its -wal file next to it is read too)

Output:
  -o, --output <PATH>       Save every row: a directory of CSV tables, or a file (json, jsonl)
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. Without -o, json and jsonl rows are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -f places.sqlite -t jsonl | jq 'select(.Status == "Deleted")'
```

[웹 브라우저](../artifacts/browser.md)를 참조하십시오.
