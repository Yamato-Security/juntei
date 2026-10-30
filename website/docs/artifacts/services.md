# Windows Services

## What service configuration records

Windows services and kernel drivers are configured under each `SYSTEM` control set. Records name
the executable/driver or service DLL, start mode, service type, account, load-order group, delayed
start, error behavior, and actions Windows takes after failure.

Attackers create or modify services for persistence, privilege escalation, driver loading, and
lateral movement. Legitimate software uses exactly the same mechanism, so paths, accounts, signing,
creation context, and runtime evidence matter.

<span class="evidence-strength">Best conclusion:</span> the parsed control set configured the
service/driver as shown. Configuration alone does not prove the service successfully started.

## Location

```text
SYSTEM\ControlSet00N\Services\<ServiceName>
SYSTEM\Select\Current   (which control set Windows used)
```

Collect `SYSTEM` with its transaction logs. Event IDs such as System 7045 and Security 4697 can
record installation; service-control and process events can support actual start/execution.

## Analyst cautions

- `ImagePath` can contain environment variables, quotes, arguments, device paths, or an unquoted
  path with spaces. Preserve it exactly before normalizing.
- `ServiceDll` is critical for `svchost.exe`-hosted services; `ImagePath` alone may name only the host.
- `StartType=automatic` does not prove the service started or remained enabled.
- `ObjectName` is the configured account, not necessarily the user who installed the service.
- Drivers and user-mode services use different `ServiceType` flags and path conventions.
- Failure actions can execute a command as the service account after a failure; this is a persistence
  surface that is easy to miss.
- Other control sets may preserve previous configuration; use `IsCurrentControlSet`.

## Parse with Juntei

```text
Usage: juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --service <NAME>    Only this service name
      --all-control-sets  Parse every ControlSet00N

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --all-control-sets -t jsonl \
  | jq 'select(.failure_command != null or (.image_path? | test("temp|public"; "i")))'
```

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `SourcePath` | System identity and input hive. Preserve across collections. |
| `ControlSet`, `IsCurrentControlSet` | Source control set and whether `Select\Current` names it. Prefer current for active-state claims. |
| `ServiceName` | Registry key/internal service name. Strong pivot into events and Service Control Manager commands. |
| `DisplayName`, `Description` | User-facing text; can be resource references or attacker-controlled. |
| `ImagePath` | Executable/driver command as configured. Parse quoting, arguments, environment variables, and device paths. |
| `ServiceDll` | DLL loaded by a host service. Often the actual payload for shared-process services. |
| `StartType`, `StartValue` | Friendly/raw start mode: boot (0), system (1), automatic (2), manual (3), disabled (4). |
| `DelayedAutostart` | Automatic service starts after initial boot phase; still persistence-capable. |
| `ServiceType`, `TypeValue` | Decoded/raw flags: kernel driver, file-system driver, own/share process, user service, etc. |
| `ErrorControl` | Boot/start error handling level. Primarily operational context. |
| `ObjectName` | Configured service account or driver object name. Not installer identity. |
| `Group` | Load-order group, especially important for boot/system drivers. |
| `FailureResetPeriod` | Seconds before failure count resets; `4294967295` means never. |
| `FailureCommand` | Command configured for a `run_command` failure action. High-value persistence field. |
| `FailureActions` | Full decoded action list (`restart`, `reboot`, `run_command`, `none`) and delays. |
| `FailureActionType`, `FailureActionDelayMs` | First action flattened for filtering; inspect full list too. |
| `FailureActionsRaw` | Original bytes when decoding is incomplete/damaged. |
| `KeyLastWrittenUTC` | Last change to the service key, not necessarily installation/start time. |
| `TransactionLogStatus`, `LogsApplied` | Registry-replay context. |
| `Problems` | Type mismatch, malformed failure structure, or missing/unreadable fields. |

## Investigation strategy

1. Review non-standard paths, writable directories, unusual accounts, new key times, and failure commands.
2. Expand environment variables using the affected host's configuration, not the analyst machine.
3. Hash/signature-check the actual executable or DLL if collected.
4. Correlate service name/path with Events 7045/4697, process creation, Prefetch, `$MFT`, and USN.
5. Compare current and non-current control sets for changed/deleted persistence.

## Further Details

- [Microsoft: Registry tree for device and driver services](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: SERVICE_STATUS / service types and states](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_status)
- [Microsoft: ChangeServiceConfig2 failure actions](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
