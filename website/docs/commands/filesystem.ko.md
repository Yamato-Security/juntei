# NTFS 명령어 { #ntfs-commands }

`parse-ntfs`는 NTFS 메타데이터를 처리합니다. 파일 레코드, 파일 시스템 트랜잭션, 변경 저널, 디렉터리 인덱스,
부트 섹터, 보안 설명자입니다. 옵션과 출력 필드는 아티팩트마다 고유합니다.

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-ntfs <COMMAND> [OPTIONS]

Commands:
  boot         Parse the NTFS boot sector
  help         Print this message or the help of the given subcommand(s)
  i30          Parse directory indexes
  logfile      Parse the NTFS transaction log
  mft          Parse the Master File Table
  secure-sds   Parse the security descriptors ($Secure:$SDS)
  usn-journal  Parse the USN journal

Options:
  -h, --help  Print help
```

## `parse-ntfs boot` { #parse-ntfs-boot }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      File path to one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the boot sectors are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

[부트 섹터 ($Boot)](../artifacts/boot.md)를 참조하십시오.

## `parse-ntfs i30` { #parse-ntfs-i30 }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-ntfs i30 <INPUT> [OPTIONS]

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
juntei parse-ntfs i30 -f 'Users_$I30'
juntei parse-ntfs i30 -d evidence/ -o i30.csv
juntei parse-ntfs i30 -f 'Users_$I30' --slack-only --no-copies -t jsonl
juntei parse-ntfs i30 -f disk.raw -o carved-i30.csv
```

[디렉터리 인덱스 ($I30)](../artifacts/i30.md)를 참조하십시오.

## `parse-ntfs logfile` { #parse-ntfs-logfile }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-ntfs logfile <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for $LogFile files (also LogFile, $LogFile.bin and $LogFile exports prefixed with a volume name)
  -f, --file <FILE>      File path to one NTFS $LogFile
  -m, --mft <FILE>       The same volume's $MFT for current paths (default: matching adjacent export, respecting its volume prefix); requires -f

Parsing:
      --cluster-size <BYTES>  Volume cluster size in bytes; otherwise read from checkpoints or inferred
      --record-size <BYTES>   MFT record size in bytes (default: 1024); supply the volume's actual size if different

Output:
      --raw-data            Include redo, undo and unknown client data as hex (can substantially increase output size)
  -o, --output <FILE>       Save every record to a file; directory inputs are combined with SourceFile on every row
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-ntfs logfile -f '$LogFile' -o logfile.csv
juntei parse-ntfs logfile -f '$LogFile' -m '$MFT' -o logfile.jsonl -t jsonl
juntei parse-ntfs logfile -d evidence -o all-logs.csv
juntei parse-ntfs logfile -f '$LogFile' --raw-data -t jsonl
```

디렉터리 모드는 모든 레코드에 `SourceFile`을 붙여 입력을 결합합니다. LSN 순서는 볼륨 전체가 아니라 입력별
순서입니다. 이 명령어는 한 번에 하나의 로그 전체를 메모리로 읽습니다. 반면 `auto-parse`는
`<COMPUTER>/FileSystem/LogFile.csv`(또는 JSON/JSONL)를 기록하며, 컴퓨터에 여러 볼륨이 있으면 번호가 붙은 파일을
사용합니다.

필드의 의미와 포렌식상의 한계는 [NTFS 트랜잭션 로그](../artifacts/logfile.md)를 참조하십시오.

## `parse-ntfs mft` { #parse-ntfs-mft }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for $MFT files
  -f, --file <FILE>      File path to one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse only these entry numbers (ex: 0-15,30)

Output:
  -o, --output <PATH>                   Save every entry to a file (with -d: a directory with one file per $MFT)
  -t, --output-type <TYPE>              Output type: csv (default), ys-ntfs-parser-csv, json, jsonl. Without -o, the entries are written to stdout
  -e, --extract-resident-streams <DIR>  Write the resident $DATA streams (the content of small files and alternate data streams) to files in DIR
  -C, --clobber                         Overwrite files when saving

CSV Output:
  -a, --all-times      Write every $FILE_NAME time, not only those that differ from $STANDARD_INFORMATION
  -s, --short-names    Write rows for DOS (8.3) names too
      --cutoff <DATE>  Only records whose $STANDARD_INFORMATION created or modified time is on or after DATE, in UTC unless an offset is given (ex: 2025-10-19, 2025-10-19T13:45:30Z)

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
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff 2026-10-20 -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 0-15,30
juntei parse-ntfs mft -f '$MFT' -x -i -E ps1,bat,vbs,js -o mft.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Windows\System32'
```

[마스터 파일 테이블](../artifacts/mft.md)을 참조하십시오.

## `parse-ntfs secure-sds` { #parse-ntfs-secure-sds }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      File path to one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the security descriptors are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

[보안 설명자 ($Secure:$SDS)](../artifacts/secure-sds.md)를 참조하십시오.

## `parse-ntfs usn-journal` { #parse-ntfs-usn-journal }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-ntfs usn-journal <INPUT> [OPTIONS]

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
juntei parse-ntfs usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-ntfs usn-journal -f unallocated.bin --carve -o carved.csv
```

[USN 저널](../artifacts/usn-journal.md)을 참조하십시오.
