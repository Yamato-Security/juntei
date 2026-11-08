# SQLite ညွှန်ကြားချက်များ { #sqlite-commands }

`parse-sqlite` သည် SQLite database များကို ကိုင်တွယ်သည်။ ဝဘ် ဘရောက်ဇာများသည် ၎င်းတို့၏ history၊ download များ၊ cookie များနှင့်
form entry များကို SQLite database များတွင် သိမ်းဆည်းသည်။ ဥပမာ Chromium-based ဘရောက်ဇာများ၏ `History`၊ `Cookies` နှင့်
`Web Data`၊ Firefox ၏ `places.sqlite`၊ `cookies.sqlite` နှင့် `formhistory.sqlite` တို့ ဖြစ်သည်။ Juntei သည် ၎င်းတို့ကို SQLite
မလိုဘဲ ၎င်းတို့၏ `-wal` ဖိုင်များရှိ နောက်ဆုံး ပြောင်းလဲမှုများနှင့်အတူ ဖတ်ပြီး ဖျက်ထားသော entry များကို ပြန်လည်ရယူသည်။

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

[ဝဘ် ဘရောက်ဇာများ](../artifacts/browser.md) ကို ကြည့်ပါ။
