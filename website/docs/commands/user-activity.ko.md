# ESE 명령어 { #ese-commands }

`parse-ese`는 ESE(Extensible Storage Engine) 데이터베이스를 처리합니다. SRUM의 `SRUDB.dat`는 ESE 데이터베이스이며,
선택적으로 지정하는 SOFTWARE 하이브를 통해 계정 이름과 Wi-Fi 이름을 보강합니다. Windows Server의 사용자 액세스 로깅
데이터베이스(`Current.mdb`, `{GUID}.mdb`, `SystemIdentity.mdb`)도 ESE 데이터베이스이며, Internet Explorer와
레거시 Edge가 방문 기록, 다운로드, 쿠키, 캐시를 저장하는 `WebCacheV01.dat`도 마찬가지입니다.

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

[SRUM](../artifacts/srum.md)을 참조하십시오.

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

[사용자 액세스 로깅 (UAL)](../artifacts/ual.md)을 참조하십시오.

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

[Internet Explorer와 레거시 Edge](../artifacts/webcache.md)를 참조하십시오.
