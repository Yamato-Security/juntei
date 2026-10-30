# Windows Registry Hives

## What registry hives are

The Windows Registry stores system and user configuration in binary hive files. Keys form a tree;
values under a key contain typed data. Registry artifacts describe persistence, installed software,
services, accounts, network configuration, user activity, security settings, and almost every part
of Windows state.

Juntei's generic reader decodes any primary hive, replays transaction logs, reports damage, and can
recover deleted/free-cell, cell-slack, remnant, pre-replay, and transaction-log-only records.

<span class="evidence-strength">Best conclusion:</span> the hive contained or recoverably preserved
the key/value state. A key last-write time shows the key changed, not which value changed or why.

## Common hives and locations

| Hive | Typical path | Examples of evidence |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | Services, drivers, control sets, host name, mounted devices, ShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | Installed software, TaskCache, profiles, Windows configuration |
| `SAM` | `C:\Windows\System32\config\SAM` | Local accounts and account metadata |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | Local security policy/secrets metadata (access-dependent) |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | Default system profile configuration |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | User preferences, Run keys, RecentDocs, application activity |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | Shell classes, COM/user shell activity |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | Application inventory (use `parse-amcache` for semantic output) |

Collect adjacent `.LOG`, `.LOG1`, and `.LOG2` files. A hive copied while Windows is running may be
dirty; transaction logs contain committed state that the primary file alone does not yet reflect.

## Analyst cautions

- A key last-write timestamp applies to the **key**, not an individual value.
- Registry values usually have no independent timestamp.
- `CurrentControlSet` is an analyst/Windows alias resolved through `SYSTEM\Select`; it is not a
  literal stored key.
- Deleted records can be partial, stale, duplicated across recovery sources, or overwritten.
- Transaction-log-only records show historical/intermediate state. Explain `recovered_from`.
- An `absent_value` result means the queried key exists but lacks that value in the parsed state;
  it does not prove the value never existed.
- Strings, paths, and command lines are configuration, not proof the configured action ran.
- A dirty hive without applicable logs can omit or expose stale state.

## Parse with Juntei

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## Output fields

| Field | Interpretation |
|---|---|
| `Host` | Host name derived from available system context; empty when Juntei cannot safely identify it. |
| `HiveType` | Detected hive category (`SYSTEM`, `SOFTWARE`, `NTUSER.DAT`, etc.). Use to interpret paths. |
| `SourcePath` | Exact input hive. Essential when directory mode combines hosts/users. |
| `RecordType` | `key`, `value`, `default_value`, or `absent_value`. Absence applies only to the query/state. |
| `KeyPath` | Path beneath the hive root. `[unknown]` marks unresolved deleted ancestry. |
| `KeyLastWrittenUTC` | Last modification of the key as a whole. It cannot identify which value changed. |
| `ValueName` | Named value; empty for a default/unnamed value. |
| `ValueType` | Registry type (`REG_SZ`, `REG_DWORD`, `REG_BINARY`, etc.). Type affects interpretation. |
| `ValueData` | Type-aware rendered value. Multi-strings remain lists in JSON. |
| `DataSize` | Size recorded by the value. Compare with raw bytes and `Problems`. |
| `ValueDataRaw` | Exact bytes when decoding fails or `--raw-data` is used. Preserve for validation. |
| `ValueU64` | Whole numeric representation for DWORD/QWORD without JavaScript precision loss. |
| `ValueFiletimeUTC` | Optional QWORD-as-FILETIME rendering. Only meaningful when the key's schema defines FILETIME. |
| `AllocationStatus` | `allocated`, `deleted_complete`, `deleted_partial`, `residual_value_only`, or `transaction_log_recovered`. Controls confidence. |
| `RecoveredFrom` | `free_cell`, `cell_slack`, `remnant_data`, `before_replay`, or log entry such as `SYSTEM.LOG1#1234`. |
| `TransactionLogStatus` | `clean`, `replayed`, `unchanged`, or `not_applied` for this record/hive context. |
| `LogsApplied` | Transaction-log file names Juntei replayed. Record these in the report. |
| `Problems` | Damage, missing cells, type mismatch, unresolved lists/parents, or other limitations. |

## Recovery-status interpretation

| Status | Meaning |
|---|---|
| `deleted_complete` | Deleted key/value was recovered with complete known path/data. Still historical, not active state. |
| `deleted_partial` | Some path, value list, value, or data was overwritten/unreadable. Do not fill gaps from reused cells. |
| `residual_value_only` | Value survived without enough key context; `KeyPath` may be absent. |
| `transaction_log_recovered` | State exists only in pre-replay/log versions and not in current allocated/free records. |

## Investigation strategy

1. Start with an exact semantic key when known; use regex/data searching for discovery.
2. Check `TransactionLogStatus`, `AllocationStatus`, `RecoveredFrom`, and `Problems` before content.
3. Interpret data according to the key's documented schema; `REG_BINARY` is not self-describing.
4. Correlate configuration with runtime evidence (event logs, Prefetch, SRUM, services/tasks).
5. Preserve raw data for novel or disputed fields.

## Further Details

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Maxim Suhanov's registry format specification](https://github.com/msuhanov/regf)
- [yarp registry parser](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [Eric Zimmerman's Registry](https://github.com/EricZimmerman/Registry)
- [Microsoft: Registry value types](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
