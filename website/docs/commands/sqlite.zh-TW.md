# SQLite 命令 { #sqlite-commands }

`parse-sqlite` 處理 SQLite 資料庫。網頁瀏覽器會將其瀏覽歷程、下載、Cookie 與表單項目保存在 SQLite
資料庫中：Chromium 系列瀏覽器的 `History`、`Cookies` 與 `Web Data`，以及 Firefox 的 `places.sqlite`、
`cookies.sqlite` 與 `formhistory.sqlite`。Juntei 不依賴 SQLite 即可讀取這些資料庫，會納入其 `-wal`
檔案中的最新變更，並復原已刪除的項目。

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

請參閱 [網頁瀏覽器](../artifacts/browser.md)。
