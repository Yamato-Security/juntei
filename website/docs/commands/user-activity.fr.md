# Commandes ESE { #ese-commands }

`parse-ese` traite les bases de données Extensible Storage Engine. Le fichier `SRUDB.dat` de SRUM est une base de données ESE ;
la ruche SOFTWARE, facultative, enrichit les noms de comptes et de réseaux Wi-Fi. Les bases de données de la
journalisation des accès utilisateur de Windows Server (`Current.mdb`, `{GUID}.mdb` et `SystemIdentity.mdb`)
sont, elles aussi, des bases de données ESE, tout comme `WebCacheV01.dat`, où Internet Explorer et l'ancien Edge
conservent leur historique, leurs téléchargements, leurs cookies et leur cache.

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

Voir [SRUM](../artifacts/srum.md).

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

Voir [Journalisation des accès utilisateur (UAL)](../artifacts/ual.md).

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

Voir [Internet Explorer et l'ancien Edge](../artifacts/webcache.md).
