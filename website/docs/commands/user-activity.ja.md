# ESE コマンド { #ese-commands }

`parse-ese` は、Extensible Storage Engine データベースを処理します。 SRUM の `SRUDB.dat` は ESE データベースです。オプションの SOFTWARE ハイブにより、アカウント名と Wi-Fi 名が強化されます。Windows Server のユーザーアクセスログのデータベース (`Current.mdb`、`{GUID}.mdb`、`SystemIdentity.mdb`) も ESE データベースで、Internet Explorer と従来の Edge が閲覧履歴、ダウンロード、Cookie、キャッシュを保存する `WebCacheV01.dat` も同様です。

```text
Version: 1.1.0 (CODE BLUE Release)
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
Version: 1.1.0 (CODE BLUE Release)
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

[SRUM](../artifacts/srum.md)を参照してください。

## `parse-ese ual` { #parse-ese-ual }

```text
Version: 1.1.0 (CODE BLUE Release)
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

[ユーザーアクセスログ (UAL)](../artifacts/ual.md)を参照してください。

## `parse-ese webcache` { #parse-ese-webcache }

```text
Version: 1.1.0 (CODE BLUE Release)
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

[Internet Explorer と従来の Edge](../artifacts/webcache.md)を参照してください。
