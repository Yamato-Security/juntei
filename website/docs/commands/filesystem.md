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

Records:
  -x, --recover-slack  Recover the file names left in the slack of FILE records (past their attributes): rows marked FromSlack, or slack_file_names in JSON

Resident Data:
  -i, --include-resident-data      Include resident $DATA (the content of small files and of alternate data streams such as Zone.Identifier) in the CSV
  -E, --resident-extensions <EXT>  With -i: only files with these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  With -i: only resident data of at most this many bytes

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail: header, every attribute, data runs, slack and the record's bytes (ex: 624, 0x270, 624-5)
  -L, --list-directory <ENTRY|PATH>  List the contents of a directory, in use and deleted (ex: 5, 624-5, \Windows\System32)

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
juntei parse-mft -f '$MFT' -x -i -E ps1,bat,vbs,js -o mft.csv
juntei parse-mft -f '$MFT' -D 0x270-5
juntei parse-mft -f '$MFT' -L '\Windows\System32'
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
