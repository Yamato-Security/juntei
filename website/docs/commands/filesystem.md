# File-System Commands

## `parse-mft`

```text
Usage:
  juntei parse-mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for $MFT files
  -f, --file <FILE>      File path to one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse only these entry numbers (ex: 0-15,30)

Output:
  -o, --output <PATH>                   Save every entry to a file (with -d: a directory with one file per $MFT)
  -t, --output-type <TYPE>              Output type: csv (default), ys-mft-parser-csv, json, jsonl. Without -o, the entries are written to stdout
  -e, --extract-resident-streams <DIR>  Write the resident $DATA streams (the content of small files and alternate data streams) to files in DIR
  -C, --clobber                         Overwrite files when saving

CSV Output:
  -a, --all-times      Write every $FILE_NAME time, not only those that differ from $STANDARD_INFORMATION
  -s, --short-names    Write rows for DOS (8.3) names too
      --cutoff <DATE>  Only records whose $STANDARD_INFORMATION created or modified time is on or after DATE, in UTC unless an offset is given

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-mft -f '$MFT' -o mft.csv
juntei parse-mft -f '$MFT' --cutoff 2026-10-20 -a -o recent.csv
juntei parse-mft -f '$MFT' -t jsonl -r 0-15,30
```

See [Master File Table](../artifacts/mft.md).

## `parse-usn-journal`

```text
Usage:
  juntei parse-usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for USN journals ($J)
  -f, --file <FILE>      File path to one USN journal ($J), or with --carve any file
  -m, --mft <FILE>       The volume's $MFT, for paths (default: the $MFT next to the journal's $Extend directory)
      --carve            Carve records out of a disk image, unallocated space or any other file that is not a journal

Output:
  -o, --output <PATH>       Save every record to a file (with -d: a directory with one file per journal)
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. Without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-usn-journal -f unallocated.bin --carve -o carved.csv
```

See [USN Journal](../artifacts/usn-journal.md).
