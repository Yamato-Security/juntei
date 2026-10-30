# User-Activity Commands

## `parse-srum`

```text
Usage:
  juntei parse-srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SRUM databases (SRUDB.dat)
  -f, --file <FILE>      File path to one SRUM database (SRUDB.dat)
  -r, --software <FILE>  The SOFTWARE hive, for account and Wi-Fi names

Output:
  -o, --output <PATH>        Save every entry: a directory of CSV tables, or a file (json, jsonl; with -d a directory with one file per database)
  -t, --output-type <TYPE>   Output type: csv (default), json, jsonl. Without -o, json and jsonl entries are written to stdout
  -C, --clobber              Overwrite files when saving
      --include-value-state  With -t json or jsonl: how each database value was read and its bytes in hex

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-srum -f SRUDB.dat -t jsonl | jq 'select(.Provider == "NetworkUsages")'
```

See [SRUM](../artifacts/srum.md).

## `parse-lnk`

```text
Usage:
  juntei parse-lnk <INPUT> [OPTIONS]

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
juntei parse-lnk -d evidence/ -o lnk.csv
juntei parse-lnk -f suspicious.lnk -t jsonl | jq .
```

See [LNK Shortcuts](../artifacts/lnk.md).

## `parse-jump-lists`

```text
Usage:
  juntei parse-jump-lists <INPUT> [OPTIONS]

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
juntei parse-jump-lists -d evidence/ -o jump-lists
juntei parse-jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

See [Jump Lists](../artifacts/jump-lists.md).
