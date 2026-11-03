# ファイルコマンド { #file-commands }

`parse-file` は、アーティファクト固有のファイル (Prefetch、ショートカット、Jump Lists、および Defender 隔離) を処理します。自動 Jump Lists は複合ファイル コンテナーを使用します。カスタム Jump Lists は異なるバイナリ レイアウトを使用します。両方とも同じアーティファクト コマンドに属します。

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

抽出ではバイナリ セキュリティ ストリームが保存され、所有権、権限、および SDDL を含む読み取り可能な `.security_descriptor.txt` レポートも作成されます。 「[Defender 隔離」](../artifacts/defender-quarantine.md) を参照してください。

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

[Jump Lists](../artifacts/jump-lists.md)を参照してください。

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

「[LNK ショートカット](../artifacts/lnk.md)」を参照してください。

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

[Prefetch](../artifacts/prefetch.md)を参照してください。
