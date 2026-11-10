# คำสั่ง SQLite { #sqlite-commands }

`parse-sqlite` จัดการฐานข้อมูล SQLite เว็บเบราว์เซอร์เก็บประวัติ การดาวน์โหลด คุกกี้ และรายการฟอร์มไว้ใน
ฐานข้อมูล SQLite ได้แก่ `History`, `Cookies` และ `Web Data` ของเบราว์เซอร์ที่ใช้ Chromium เป็นพื้นฐาน
และ `places.sqlite`, `cookies.sqlite` และ `formhistory.sqlite` ของ Firefox Juntei อ่านฐานข้อมูลเหล่านี้
โดยไม่พึ่งไลบรารี SQLite พร้อมการเปลี่ยนแปลงล่าสุดจากไฟล์ `-wal` ของแต่ละฐานข้อมูล และกู้คืนรายการที่ถูกลบ

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

ดู [เว็บเบราว์เซอร์](../artifacts/browser.md)
