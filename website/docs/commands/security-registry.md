# Security and Registry Commands

## `parse-defender-quarantine`

```text
Usage:
  juntei parse-defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Defender quarantine folders
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
juntei parse-defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-defender-quarantine -d evidence/ -o quarantine.csv --extract isolated-output/
```

Extraction preserves binary security streams and also creates readable `.security_descriptor.txt` reports with
ownership, permissions and SDDL. See [Defender Quarantine](../artifacts/defender-quarantine.md).

## Registry subcommands

```text
Usage:
  juntei parse-registry <COMMAND> [OPTIONS]

Commands:
  read        Read keys and values, types, data, times, allocation and recovery status
  services    Decode services and failure actions from SYSTEM
  recentdocs  Decode Explorer RecentDocs from NTUSER.DAT
  taskcache   Decode scheduled tasks and actions from SOFTWARE TaskCache
  sam         Decode local accounts from SAM
  help        Print this message or the help of the given subcommand(s)
```

All registry subcommands accept `-f <HIVE>` or `-d <COLLECTION>`, structured output options, and
automatic adjacent transaction-log replay. Use `--log1` / `--log2` to name logs explicitly or
`--no-logs` to intentionally inspect the primary file without replay.

### `parse-registry read`

```text
Usage:
  juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for registry hives
  -f, --file <FILE>      File path to one registry hive
      --log1 <FILE>      Explicit first transaction log
      --log2 <FILE>      Explicit second transaction log
      --no-logs          Read the primary hive without transaction-log replay

Filtering:
  -k, --key <PATH>             Only the key at this path
      --key-regex <REGEX>      Only keys whose path matches REGEX (ignoring case)
      --value <NAME>           Only the value named NAME
      --value-regex <REGEX>    Only values whose name matches REGEX (ignoring case)
      --data-regex <REGEX>     Only values whose rendered data matches REGEX (ignoring case)
      --timeline-start <DATE>  Only keys last written on or after DATE
      --timeline-end <DATE>    Only keys last written on or before DATE
      --recover-deleted        Recover deleted, slack, remnant and transaction-log-only records

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Write every value's exact bytes in hexadecimal
      --qword-as-filetime   Also interpret REG_QWORD values as FILETIMEs
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-registry read -f SYSTEM --key 'CurrentControlSet\Services' -o services-raw.csv
juntei parse-registry read -f NTUSER.DAT --recover-deleted -t jsonl | jq .
```

### `parse-registry services`

```text
Usage:
  juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM hives
  -f, --file <FILE>      File path to one SYSTEM hive
      --log1 <FILE>      Explicit first transaction log
      --log2 <FILE>      Explicit second transaction log
      --no-logs          Read the primary hive without transaction-log replay

Filtering:
      --service <NAME>    Only the service of this name
      --all-control-sets  Parse every ControlSet00N, not only Select\Current

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

### `parse-registry recentdocs`

```text
Usage:
  juntei parse-registry recentdocs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      Explicit first transaction log
      --log2 <FILE>      Explicit second transaction log
      --no-logs          Read the primary hive without transaction-log replay

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

### `parse-registry taskcache`

```text
Usage:
  juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE hives
  -f, --file <FILE>      File path to one SOFTWARE hive
      --log1 <FILE>      Explicit first transaction log
      --log2 <FILE>      Explicit second transaction log
      --no-logs          Read the primary hive without transaction-log replay

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

### `parse-registry sam`

```text
Usage:
  juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SAM hives
  -f, --file <FILE>      File path to one SAM hive
      --log1 <FILE>      Explicit first transaction log
      --log2 <FILE>      Explicit second transaction log
      --no-logs          Read the primary hive without transaction-log replay

Filtering:
      --rid <RID>  Only one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

### Examples

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --service ExampleSvc -t jsonl
juntei parse-registry recentdocs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry taskcache -f SOFTWARE --task ScheduledDefrag -t jsonl
juntei parse-registry sam -f SAM --rid 500 -t jsonl
```

The complete options and field semantics are on [Registry Hives](../artifacts/registry.md),
[Windows Services](../artifacts/services.md), [RecentDocs](../artifacts/recentdocs.md),
[Task Scheduler Cache](../artifacts/taskcache.md), and [SAM Local Accounts](../artifacts/sam.md).
