# SRUM

## What SRUM is

The System Resource Usage Monitor (SRUM) stores an ESE database of application and service resource
usage. Depending on the Windows version, it records network bytes, interface/profile identifiers,
connections, CPU time, foreground/background activity, disk activity, energy use, push
notifications, and application timeline data. Entries are associated with application IDs and user
SIDs through `SruDbIdMapTable`.

SRUM is especially useful when an executable has been deleted: its application ID, user, and
resource-use rows can outlive the file. Network totals can support an exfiltration hypothesis, but
SRUM is not packet capture and generally does not identify every remote endpoint or transferred file.

<span class="evidence-strength">Best conclusion:</span> Windows attributed the recorded resource
usage to the application/account during the table's measurement period.

## Location and collection

```text
C:\Windows\System32\sru\SRUDB.dat
C:\Windows\System32\config\SOFTWARE   (context for account and Wi-Fi names)
```

SRUM exists on Windows 8 and later. Copy `SRUDB.dat` consistently, ideally from a snapshot or with
a forensic collector. Also collect `SOFTWARE` and the ESE transaction/log files in the `sru`
directory.

!!! warning "Dirty databases"
    Juntei reads a dirty `SRUDB.dat` as collected and reports it. It does not currently replay
    `SRU*.log`, so changes existing only in those logs are missing. Do not silently describe a
    dirty database as complete.

## Analyst cautions

- Many rows represent periodic aggregation, commonly around hourly boundaries—not exact action time.
- Byte totals do not identify content, intent, protocol, or destination by themselves.
- An application identity can be a path, service, packaged app, or object; examine `AppType`.
- User and Wi-Fi names are enrichment from the matching `SOFTWARE` hive. Missing names do not mean
  the underlying ID is absent.
- `null`, zero, absent columns, unsupported types, and damaged values are different states.
- Provider schemas vary by Windows version. Preserve unknown tables and original column names.
- Background Windows services legitimately generate large resource totals.

## Parse with Juntei

```text
Usage: juntei parse-srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SRUDB.dat
  -f, --file <FILE>      Parse one SRUDB.dat
  -r, --software <FILE>  Matching SOFTWARE hive for names

Output:
  -o, --output <PATH>        Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>   csv, json, or jsonl
      --include-value-state  Add per-value read state and raw bytes to JSON/JSONL
```

```bash
juntei parse-srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-srum -f SRUDB.dat -t jsonl \
  | jq 'select(.Provider == "NetworkUsages" and (.BytesSent // 0) > 100000000)'
juntei parse-srum -f SRUDB.dat -t jsonl --include-value-state \
  | jq '{Provider, Timestamp, _value_state, _raw}'
```

## Output tables

CSV creates `IdentityMap.csv` for `SruDbIdMapTable` and one table per provider. Known output names include:

| Provider/table | Investigative purpose |
|---|---|
| `NetworkUsages` | Bytes sent/received per application, user, interface, and profile. |
| `NetworkConnections` | Connection-count/duration style activity and connection start times. |
| `AppResourceUseInfo` | Foreground/background CPU, disk, network, and related application usage. |
| `AppTimelineProvider` | Application timeline activity with end-time and usage-state fields. |
| `EnergyUsage` | Short- and long-term energy/provider measurements; `IsLt` distinguishes them. |
| `PushNotifications` | Push-notification activity. |
| `VfuProvider` | The `vfuprov` provider; its detailed purpose is not well documented, so interpret its fields cautiously. |
| `EnergyEstimation`, `TaggedEnergy` | Version-specific energy and usage providers. |
| `ProcessorTime`, `SizeInBytes`, `TotalUsed`, `NetworkBytes` | Windows Server provider tables. |
| Unknown table name | Juntei preserves unknown providers under their GUID/table name and emits every catalog column. |

Because SRUM is catalog-driven and Windows adds provider columns, there is intentionally no fixed
finite list of provider-specific fields. Juntei emits **every column present in the database using
its original name**. The following fields are the stable/common output contract.

## Common output fields

| Field | Interpretation |
|---|---|
| `SourceFile` | Input database; present in JSON/JSONL and useful across hosts. |
| `Table` | Original ESE table name/GUID. Preserve when comparing versions or unknown providers. |
| `Provider` | Friendly provider/output name, or the original table name if unknown. |
| `Id` | Row's `AutoIncId`; a database row identifier, not a process ID. |
| `Timestamp` | When SRUM recorded/aggregated the figures, rounded to milliseconds. Usually not exact event time. |
| `ExeInfo` | Resolved application/service/package identity from `AppId`. Main program pivot. |
| `ExeInfoDescription` | Description extracted from composite SRUM application names. Context only. |
| `ExeTimestamp` | Time embedded in a composite application identity where present; not the row's usage time. |
| `AppType` | `NormalApp`, `Service`, `ModernApp`, or `Object`; controls how `ExeInfo` should be interpreted. |
| `Sid` | Resolved account SID from `UserId`. Strong user/service identity pivot. |
| `UserName` | Friendly profile name enriched from `SOFTWARE`; may be null while `Sid` remains valid. |
| `UserId`, `AppId` | Raw numeric references into `SruDbIdMapTable`; preserve for unresolved or duplicate names. |
| `IsLt` | `true` for the long-term EnergyUsage table, `false` for short-term. |
| `InterfaceLuid` | Raw network interface LUID from provider rows. |
| `InterfaceType` | Decoded interface class (for example `IF_TYPE_IEEE80211`); class, not a unique adapter. |
| `L2ProfileId` | Numeric network profile reference. |
| `ProfileName` | Wi-Fi/profile name enriched from `SOFTWARE`; potentially sensitive and not proof of current connection. |
| `ConnectStartTime`, `EventTimestamp`, `EndTime`, `StartTime` | Known provider FILETIME columns decoded to UTC. Their exact meaning belongs to the provider, not a universal execution time. |
| Provider-specific columns | Original catalog names such as `BytesSent`, `BytesRecvd`, CPU/disk counters, durations, counts, flags, or estimates. Interpret units according to the provider and compare rows within the same schema. |

## Diagnostic fields (`--include-value-state`)

| Field | Interpretation |
|---|---|
| `_value_state` | Map of database field to `parsed`, `null`, `absent`, `unsupported`, or `damaged`. This separates evidence absence from parser failure. |
| `_raw` | Original bytes in hexadecimal for values that have bytes. Use for validation and future decoding. |

`IdentityMap.csv` (from `SruDbIdMapTable`) contains `IdIndex`, `IdType`, and `IdBlob`: the raw index, identity category,
and decoded program/SID/object text used by provider rows.

## Investigation strategy

1. Identify the program with `ExeInfo`, `AppType`, and `AppId`; do not confuse `Id` with PID.
2. Group network rows by program, user, profile, interface, and time interval.
3. Compare bytes sent/received with normal baselines and other hosts.
4. Correlate suspicious intervals with Prefetch, event logs, DNS/proxy/firewall data, and file access.
5. Report dirty-database status and missing transaction-log replay as a collection limitation.

## Further Details

- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [Eric Zimmerman's SrumECmd](https://github.com/EricZimmerman/Srum)
- [dissect.esedb](https://github.com/fox-it/dissect.esedb)
- [libyal libesedb](https://github.com/libyal/libesedb)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
