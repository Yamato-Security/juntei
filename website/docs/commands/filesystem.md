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

## `parse-logfile`

```text
Usage:
  juntei parse-logfile <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $LogFile, LogFile, $LogFile.bin or volume_$LogFile exports
  -f, --file <FILE>      One NTFS $LogFile (any filename)
  -m, --mft <FILE>       Same volume's $MFT for current paths; requires -f
                        Default: $MFT next to each input

Parsing:
      --cluster-size <BYTES>  Volume cluster size; otherwise from checkpoints or inferred
      --record-size <BYTES>   MFT record size (default: 1024)

Output:
      --raw-data              Include redo, undo and unknown client data as hex
  -o, --output <FILE>          Save every log record to one file
  -t, --output-type <TYPE>     csv (default), json, jsonl; with -t and no -o, write data to stdout
  -C, --clobber                Overwrite existing output files

General Options:
  -h, --help      Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Do not display the launch banner
```

```bash
juntei parse-logfile -f '$LogFile' -o logfile.csv
juntei parse-logfile -f '$LogFile' -m '$MFT' -o logfile.jsonl -t jsonl
juntei parse-logfile -d evidence -o all-logs.csv
juntei parse-logfile -f '$LogFile' --raw-data -t jsonl
```

Directory mode combines inputs with `SourceFile` on every record. LSN order is per input, not
across volumes. The command reads one whole log into memory at a time. `auto-parse` instead writes
`<COMPUTER>/logfile.csv` (or JSON/JSONL), with numbered files when a computer has multiple volumes.

See [NTFS Transaction Log](../artifacts/logfile.md) for field meanings and forensic limitations.

## `parse-i30`

```text
Usage:
  juntei parse-i30 <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for $I30 files (with $I30 in their name, or starting with an INDX record)
  -f, --file <FILE>      File path to one $I30 file (a directory's $INDEX_ALLOCATION:$I30 stream), or any file holding INDX records (a disk image, unallocated space)

Filtering:
      --slack-only  Only the entries found in slack (possible deleted or renamed file names, and copies of entries in use)
      --no-copies   Leave out the slack entries that copy an entry in use (same name, file, directory and times)

Output:
  -o, --output <FILE>       Save every entry to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the entries are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-i30 -f 'Users_$I30'
juntei parse-i30 -d evidence/ -o i30.csv
juntei parse-i30 -f 'Users_$I30' --slack-only --no-copies -t jsonl
juntei parse-i30 -f disk.raw -o carved-i30.csv
```

See [Directory Indexes ($I30)](../artifacts/i30.md).
