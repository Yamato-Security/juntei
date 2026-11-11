# Perintah berkas { #file-commands }

`parse-file` menangani berkas yang khusus untuk artefak tertentu: Prefetch, pintasan, Jump Lists,
karantina Defender, berkas PST, OST, MSG, dan OFT milik Outlook, serta berkas indeks Recycle Bin. Jump List otomatis
menggunakan kontainer compound-file; Jump List kustom menggunakan tata letak biner yang berbeda.
Keduanya termasuk dalam perintah artefak yang sama.

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file <COMMAND> [OPTIONS]

Commands:
  defender-quarantine  Parse Defender quarantine
  help                 Print this message or the help of the given subcommand(s)
  jump-lists           Parse Jump Lists
  lnk                  Parse shortcuts
  outlook              Parse Outlook PST, OST, MSG and OFT files
  prefetch             Parse Prefetch files
  recycle-bin          Parse Recycle Bin index files ($I files and INFO2)

Options:
  -h, --help  Print help
```

## `parse-file defender-quarantine` { #parse-file-defender-quarantine }

```text
Version: 1.1.0 (CODE BLUE Release)
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

Ekstraksi mempertahankan stream keamanan biner dan juga membuat laporan `.security_descriptor.txt` yang mudah dibaca berisi
kepemilikan, izin, dan SDDL. Lihat [Karantina Defender](../artifacts/defender-quarantine.md).

## `parse-file jump-lists` { #parse-file-jump-lists }

```text
Version: 1.1.0 (CODE BLUE Release)
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

Lihat [Jump Lists](../artifacts/jump-lists.md).

## `parse-file lnk` { #parse-file-lnk }

```text
Version: 1.1.0 (CODE BLUE Release)
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

Lihat [Pintasan LNK](../artifacts/lnk.md).

## `parse-file outlook` { #parse-file-outlook }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file outlook <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Outlook files (*.pst, *.ost, *.msg, *.oft)
  -f, --file <FILE>      File path to one PST, OST, MSG or OFT file (told apart by its contents, whatever its name)
      --code-page <CP>   Code page of text that isn't Unicode, for items that don't give theirs: the system's ANSI code page (932 for Japanese) [default: 1252]

Output:
  -o, --output <PATH>              Save every row: a directory of CSV tables, or a file (json, jsonl)
  -t, --output-type <TYPE>         Output type: csv (default), json, jsonl. Without -o, json and jsonl rows are written to stdout
      --properties                 Also write every property of the stores, folders, items, recipients and attachments (the Properties table)
      --extract-attachments <DIR>  Write each attachment's data once to DIR, named by its SHA-256 (attachments can be malware: never run them)
  -C, --clobber                    Overwrite files when saving or extracting

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-file outlook -d Users -o outlook-csv
juntei parse-file outlook -f mailbox.ost -t jsonl | jq 'select(.Table == "Items")'
juntei parse-file outlook -f invoice.msg -o msg-csv --properties --extract-attachments isolated-output/
```

Tabelnya adalah berkas yang dibaca, folder-foldernya, setiap item (termasuk item tersembunyi di folder
dan item yang tertanam di lampiran), penerima, lampiran, kontak, item kalender, dan tugas, serta, dengan
`--properties`, setiap properti sebagaimana tersimpan. Lihat [Outlook](../artifacts/outlook.md).

## `parse-file prefetch` { #parse-file-prefetch }

```text
Version: 1.1.0 (CODE BLUE Release)
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

Lihat [Prefetch](../artifacts/prefetch.md).

## `parse-file recycle-bin` { #parse-file-recycle-bin }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Recycle Bin index files ($I files, INFO2)
  -f, --file <FILE>      File path to one $I or INFO2 file
      --code-page <CP>   Code page of INFO2's ANSI paths: the system's ANSI code page (932 for Japanese) [default: 1252]

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
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

Lihat [Keranjang Sampah (Recycle Bin)](../artifacts/recycle-bin.md).
