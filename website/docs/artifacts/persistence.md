# Registry Autostarts

## What they are

Windows starts programs and loads DLLs from many registry locations: at boot, at logon, when a
program starts or exits, when `cmd.exe` opens, when a COM class is created, or when the spooler,
LSA, netsh or the time service loads its plug-ins. Attackers use these locations for persistence,
privilege escalation and DLL injection, and legitimate software uses them too. Juntei reads them
from `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` and `UsrClass.dat` into one table. It writes one record per
program, DLL or package, and tags each with its MITRE ATT&CK technique.

<span class="evidence-strength">Best conclusion:</span> the hive configured this program or DLL
to start or load. It does not prove the program ran. Running it needs the conditions the
mechanism names (a logon, the debugged program starting, the class being created), and other
evidence of execution.

## Locations

| Hive | What is read |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run`, `RunOnce` (+`Setup`), `RunOnceEx`, `RunServices(Once)` and `Policies\Explorer\Run`, with their `Wow6432Node` and Terminal Server install-mode copies; `Explorer\StartupApproved\Run`, `Run32` and `StartupFolder`; `Winlogon` (`Shell`, `Userinit`, `Taskman`, `AppSetup`, `GinaDLL`, … and `Notify`); `Windows` `AppInit_DLLs` with `LoadAppInit_DLLs`; `Image File Execution Options\*` `Debugger` (with `UseFilter`) and `VerifierDlls`; `SilentProcessExit\*` `MonitorProcess`; `Active Setup\Installed Components\*` `StubPath`; `NetSh`; `Command Processor` `AutoRun` |
| `SYSTEM` | `Control\Session Manager` (`BootExecute`, `SetupExecute`, `Execute`, `PendingFileRenameOperations`, `KnownDLLs`, `AppCertDlls`, `Environment` .NET profiler variables); `Control\Lsa` `Authentication`, `Security` and `Notification Packages` (+`OSConfig`); `SecurityProviders`; `Control\Print` monitors, providers and print processors; `Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | the user's `Run` keys and `StartupApproved`; `Windows NT\CurrentVersion\Windows` `Load` and `Run`; `Winlogon` `Shell`; `Environment` (`UserInitMprLogonScript`, .NET profiler); `Control Panel\Desktop` `SCRNSAVE.EXE`; `PackagedAppXDebug`; `Command Processor` `AutoRun`; `Software\Classes` (as `UsrClass.dat`) |
| `UsrClass.dat` | `CLSID\*\InprocServer32` and `LocalServer32` (and `Wow6432Node`'s); `shell\open\command` of program types (`exefile`, `batfile`, `scrfile`, …) and of what auto-elevating programs start (`ms-settings`, `mscfile`, `Folder`, …); `ActivatableClasses\…\DebugInformation` |

Collect each hive with its transaction logs. User hives are in `Users\<name>\NTUSER.DAT` and
`Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat`.

## Analyst cautions

- **Defaults are reported too.** Most records on a clean system are Windows defaults:
  - `Userinit` is `userinit.exe`;
  - `Shell` is `explorer.exe`;
  - `KnownDLLs`, LSA packages, print monitors, netsh helpers and Active Setup components all have stock entries.

  Look for what differs from a clean build of the same version.
- **`StartupApproved` only covers the Task Manager's Startup list.** It says whether a `Run`
  entry or a Startup folder file is disabled there, and when. An entry with no matching `Run`
  value is reported, with the details "only in StartupApproved". It can show an autostart that
  was removed.
- **`OverridesMachineRegistration` needs the system's `SOFTWARE` hive.** A user's COM class or
  command is checked against it, and it is found the same way the host's `SYSTEM` hive is. An
  override is the COM hijacking pattern (T1546.015). OneDrive and other per-user installs also
  register classes of their own that override nothing.
- **`ImagePath` and `Arguments` are split from the command line only where that can be told.**
  - A quoted program is split at its closing quote.
  - A first word without a folder is taken as the program (`cmd /c …`).
  - Otherwise the split is at the shortest start that ends with a program's extension.

  An unquoted path with spaces may run a different program (`C:\Program.exe`) if one exists.
- **`GlobalFlag` decides `Enabled` for `VerifierDlls` and `MonitorProcess`.** `VerifierDlls` loads
  only with `0x100`, and `MonitorProcess` runs only with `0x200`.
- **`KeyLastWrittenUTC` is the key's last write**, not the value's. Several values share it.

## Parse with Juntei

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.mechanism != "com_server" or .overrides_machine_registration == true)'
```

The summary counts records by mechanism. It also lists the `StartupApproved` entries left
without their `Run` value, and the user COM classes and commands that override the machine's.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host context, source hive and its type. |
| `User`, `UserSid` | For user hives: the profile folder, and the SID. The SID comes from `UsrClass.dat`'s root key name, else from the system's `ProfileList`. |
| `Mechanism` | `run`, `run_once`, `winlogon`, `ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `boot_execute`, `lsa_security_package`, `com_server`, `uac_bypass_command`, … |
| `AttackTechnique` | MITRE ATT&CK technique, where one fits (`T1547.001`, `T1546.012`, …). |
| `KeyPath`, `ValueName` | The key under the hive's root (names as written) and the value (empty: the default value). |
| `Data` | The value's text, or one item of a list. Empty when it can't be read, or when only `StartupApproved` names the entry. |
| `ImagePath`, `Arguments` | The program or DLL, and the arguments, where they can be told apart (see above). |
| `Enabled`, `DisabledAtUTC` | What `StartupApproved`, `LoadAppInit_DLLs`, `GlobalFlag`, `UseFilter`, `IsInstalled`, the time provider's `Enabled` or `ScreenSaveActive` says. Empty when nothing says. |
| `Details` | More: what a pending rename does, the COM class's name, `ReportingMode`, `FilterFullPath`, an Active Setup `ShellComponent`, … |
| `OverridesMachineRegistration` | For a user's COM class or command: whether the machine registers the same one. Empty without the system's `SOFTWARE` hive. |
| `Wow64` | Under `Wow6432Node`: for 32-bit programs. |
| `KeyLastWrittenUTC` | The key's last write. |
| `DataRaw` | The value's bytes (hex), when they are not text or not valid UTF-16. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Wrong value types, data that can't be read, `StartupApproved` entries of the wrong size. |

## Investigation strategy

1. Start with the summary:
   - COM classes and commands that override the machine's;
   - `StartupApproved` entries without their `Run` value;
   - the mechanisms that seldom have entries (`ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `uac_bypass_command`, `file_association`, `logon_script`, `dotnet_profiler`).
2. Compare LSA packages, `BootExecute`, `KnownDLLs`, print monitors and netsh helpers with a clean
   build of the same Windows version.
3. Resolve `ImagePath` against the file system ($MFT, hashes and signatures), and check user
   profile, `ProgramData` and `Temp` paths.
4. Correlate `KeyLastWrittenUTC` with Security and Sysmon registry events, process creation,
   Prefetch and Amcache.
5. Use `parse-registry read --recover-deleted` to find autostarts removed from these keys.

## Further Details

- [MITRE ATT&CK T1547 Boot or Logon Autostart Execution](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 Event Triggered Execution](https://attack.mitre.org/techniques/T1546/)
- [Microsoft: Run and RunOnce registry keys](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
