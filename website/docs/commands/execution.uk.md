# Команди для файлів { #file-commands }

`parse-file` обробляє файли окремих артефактів: Prefetch, ярлики, Jump Lists і карантин
Defender. Автоматичні Jump Lists використовують контейнер складеного файлу (compound file); користувацькі Jump Lists мають інший
двійковий формат. Обидва типи належать до тієї самої команди артефакту.

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file <COMMAND> [OPTIONS]

Commands:
  defender-quarantine  Parse Defender quarantine
  help                 Print this message or the help of the given subcommand(s)
  jump-lists           Parse Jump Lists
  lnk                  Parse shortcuts
  prefetch             Parse Prefetch files

Options:
  -h, --help  Print help
```

## `parse-file defender-quarantine` { #parse-file-defender-quarantine }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Defender quarantine folders (a volume, a Quarantine folder, a collection of hosts)
  -f, --file <FILE>      File path to one quarantine entry (Quarantine\Entries\{GUID})

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. Without -o, the records are written to stdout
      --extract <DIR>       Write quarantined files (likely malware, never run) with a .quarantined extension
  -C, --clobber             Overwrite files when saving or extracting

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract isolated-output/
```

Видобування зберігає двійкові потоки безпеки, а також створює зручні для читання звіти `.security_descriptor.txt` із
відомостями про власника, дозволи та SDDL. Див. [Карантин Defender](../artifacts/defender-quarantine.md).

## `parse-file jump-lists` { #parse-file-jump-lists }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file jump-lists <INPUT> [OPTIONS]

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
juntei parse-file jump-lists -d evidence/ -o jump-lists
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

Див. [Jump Lists](../artifacts/jump-lists.md).

## `parse-file lnk` { #parse-file-lnk }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file lnk <INPUT> [OPTIONS]

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
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

Див. [Ярлики LNK](../artifacts/lnk.md).

## `parse-file prefetch` { #parse-file-prefetch }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for prefetch files (*.pf)
  -f, --file <FILE>      File path to one prefetch file

Records:
  -T, --timeline  One record per execution (the last 8 run times of every file), sorted by time

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
juntei parse-file prefetch -d evidence/ -T -o prefetch-timeline.csv
```

Див. [Prefetch](../artifacts/prefetch.md).
