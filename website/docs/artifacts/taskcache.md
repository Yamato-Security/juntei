# Task Scheduler Cache

## What TaskCache is

Windows Task Scheduler keeps a registry cache under `SOFTWARE` linking task paths to GUIDs,
triggers/lists, principals, actions, runtime metadata, and security descriptors. Juntei decodes
execution and COM-handler actions plus legacy email/message actions, task timestamps, and signs of
tasks hidden from the normal Task Scheduler interface.

Tasks are a common persistence and remote-execution mechanism. The registry cache is also valuable
when an attacker deletes or tampers with the task's XML file.

<span class="evidence-strength">Best conclusion:</span> the TaskCache configured or preserved the
task/action. Runtime fields and independent logs are needed to prove a particular execution.

## Locations

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

Collect `SOFTWARE` with transaction logs and the complete `System32\Tasks` tree. Task Scheduler
Operational and Security event logs add creation/update/runtime evidence.

## Analyst cautions

- Registry cache and task XML can disagree after tampering, deletion, incomplete registration, or damage.
- Juntei's `Hidden` identifies structural hiding (missing Tree linkage/security descriptor under
  relevant conditions), not simply a task configured with a normal “Hidden” UI flag.
- Windows 7 stores task actions in XML rather than the cache's `Actions` value; an empty action is
  not automatically suspicious there.
- `LastRun` and `LastResult` are useful but not a complete, tamper-proof execution history.
- `Context` names the configured principal/context, not the user who registered the task.
- COM handler CLSIDs require separate registry/file resolution.

## Parse with Juntei

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.hidden == true or (.executable? | test("powershell|cmd|mshta"; "i")))'
```

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `SourcePath` | Host context and source `SOFTWARE` hive. |
| `TaskName` | Full scheduler path. Primary pivot into XML and event logs. |
| `TaskGuid` | GUID joining Tree and Tasks cache entries. Strong correlation key. |
| `TaskIndex` | Tree/index metadata used by the scheduler; contextual, not an execution count. |
| `TaskLists` | Cache lists (`Boot`, `Logon`, `Plain`, `Maintenance`) that categorize triggering/handling. |
| `Hidden` | Structural hidden-task indicator. Review `InTree`, `InTasks`, and descriptor state before concluding tampering. |
| `InTree`, `InTasks` | Whether both cache sides exist. Missing linkage can indicate deletion/tampering/incomplete registration. |
| `HasSecurityDescriptor` | Whether Tree entry has `SD`; absence can contribute to hidden-task behavior on relevant versions. |
| `TreeKeyLastWrittenUTC`, `TaskKeyLastWrittenUTC` | Key-level changes for each cache side; not exact creation/run times. |
| `ActionIndex` | Zero-based action within the task. One task can produce multiple rows. |
| `ActionType` | `exec`, `com_handler`, `send_email`, or `show_message`. |
| `Executable`, `Arguments`, `WorkingDirectory` | Effective execution action. Interpret quoting, variables, relative paths, and LOLBins. |
| `ComClassId`, `ActionData` | COM handler CLSID and data, or legacy email/message payload. Resolve CLSID separately. |
| `Context` | Principal context encoded with actions (`LocalSystem`, `Author`, etc.); configured identity, not registrar. |
| `Author`, `Uri`, `Description` | Task metadata. Easily attacker-controlled and not trust evidence. |
| `LastRegisteredUTC` | DynamicInfo registration/update time where present. |
| `LastRunUTC`, `LastSuccessfulRunUTC`, `LastResult` | Cached runtime/result metadata. Correlate with operational logs and process evidence. |
| `ActionsRaw` | Original action bytes if decoding is incomplete or validation is required. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Path/GUID mismatch, wrong value type, truncated action structure, or missing pieces. |

## Investigation strategy

1. Review hidden/inconsistent tasks and non-Microsoft paths first.
2. Reconstruct effective commands, including arguments, working directory, environment expansion, and COM resolution.
3. Compare registry record with task XML and hash/timestamp the referenced executable.
4. Correlate `TaskName`/GUID/time with Task Scheduler events, Security 4698/4702, process logs, Prefetch, and MFT/USN.
5. Search deleted/recovered registry data when an attacker removed the task.

## Further Details

- [Microsoft: Task Scheduler schema](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft: Task Scheduler event log](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 Scheduled Task](https://attack.mitre.org/techniques/T1053/005/)
