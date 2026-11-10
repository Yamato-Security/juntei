# ESE ညွှန်ကြားချက်များ { #ese-commands }

`parse-ese` သည် Extensible Storage Engine ဒေတာဘေ့စ်များကို ကိုင်တွယ်သည်။ SRUM ၏ `SRUDB.dat` သည် ESE ဒေတာဘေ့စ် ဖြစ်သည်။
ရွေးချယ်နိုင်သော SOFTWARE hive က အကောင့်နှင့် Wi-Fi အမည်များကို ဖြည့်စွက်ပေးသည်။ Windows Server ၏ အသုံးပြုသူ ဝင်ရောက်မှု မှတ်တမ်း
(User Access Logging) ဒေတာဘေ့စ်များ (`Current.mdb`၊ `{GUID}.mdb` နှင့် `SystemIdentity.mdb`) သည်လည်း ESE ဒေတာဘေ့စ်များ ဖြစ်သည်။
Internet Explorer နှင့် legacy Edge တို့က ၎င်းတို့၏ history၊ download များ၊ cookie များနှင့် cache ကို သိမ်းဆည်းသော
`WebCacheV01.dat` သည်လည်း ESE ဒေတာဘေ့စ် ဖြစ်သည်။

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-ese <COMMAND> [OPTIONS]

Commands:
  help      Print this message or the help of the given subcommand(s)
  srum      Parse SRUM databases
  ual       Parse User Access Logging (UAL) databases
  webcache  Parse Internet Explorer and legacy Edge's WebCache databases

Options:
  -h, --help  Print help
```

## `parse-ese srum` { #parse-ese-srum }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-ese srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SRUM databases (SRUDB.dat)
  -f, --file <FILE>      File path to one SRUM database (SRUDB.dat)
  -r, --software <FILE>  The SOFTWARE hive, for account and Wi-Fi names (default: config/SOFTWARE of the same Windows, or a SOFTWARE next to the database)

Output:
  -o, --output <PATH>        Save every entry: a directory of CSV tables, or a file (json, jsonl; with -d a directory with one file per database)
  -t, --output-type <TYPE>   Output type: csv (default), json, jsonl. Without -o, json and jsonl entries are written to stdout
  -C, --clobber              Overwrite files when saving
      --include-value-state  With -t json or jsonl: how each database value was read (_value_state: parsed, null, absent, unsupported, damaged) and its bytes in hex (_raw)

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-ese srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-ese srum -f SRUDB.dat -t jsonl | jq 'select(.Provider == "NetworkUsages")'
```

[SRUM](../artifacts/srum.md) ကို ကြည့်ပါ။

## `parse-ese ual` { #parse-ese-ual }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-ese ual <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for UAL databases (Current.mdb, {GUID}.mdb, SystemIdentity.mdb)
  -f, --file <FILE>      File path to one UAL database (role names come from the SystemIdentity.mdb next to it)

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
juntei parse-ese ual -d Sum -o ual-csv
juntei parse-ese ual -d Sum -t jsonl | jq 'select(.Table == "CLIENTS")'
```

[အသုံးပြုသူ ဝင်ရောက်မှု မှတ်တမ်း (UAL)](../artifacts/ual.md) ကို ကြည့်ပါ။

## `parse-ese webcache` { #parse-ese-webcache }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for WebCache databases (WebCacheV01.dat)
  -f, --file <FILE>      File path to one WebCache database (its transaction logs are not read)

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
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -f WebCacheV01.dat -t jsonl | jq 'select(.Table == "History")'
```

[Internet Explorer နှင့် legacy Edge](../artifacts/webcache.md) ကို ကြည့်ပါ။
