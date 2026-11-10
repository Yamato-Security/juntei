# SQLite Komutları { #sqlite-commands }

`parse-sqlite`, SQLite veritabanlarını işler. Web tarayıcıları geçmişlerini, indirmelerini, çerezlerini ve
form girdilerini SQLite veritabanlarında saklar: Chromium tabanlı tarayıcıların `History`, `Cookies` ve
`Web Data` dosyaları ile Firefox'un `places.sqlite`, `cookies.sqlite` ve `formhistory.sqlite` dosyaları.
Juntei bunları SQLite kullanmadan, `-wal` dosyalarındaki en yeni değişikliklerle birlikte okur ve silinmiş
girdileri kurtarır.

```text
Version: 1.1.0 (CODE BLUE Release)
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
Version: 1.1.0 (CODE BLUE Release)
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

Bkz. [Web Tarayıcıları](../artifacts/browser.md).
