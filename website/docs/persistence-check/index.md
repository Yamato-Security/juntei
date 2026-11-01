# Persistence Check

Persistence is a way to make code run again: when Windows starts, a user signs in, an application
opens, a scheduled time arrives, or another event occurs. Attackers can abuse the same settings
that legitimate software uses for updates, device support, security, and administration.

`juntei check-persistence` examines captured Windows evidence offline for **131 checks** covering
these mechanisms and related account/protection anomalies. It reads registry hives, task files,
scripts, application settings, and selected database records. It does not execute the programs it
finds or change the source computer. No Python interpreter or Windows runtime is required.

This page explains every implemented check. For complete options, output fields, profiles, and
timestamp handling, open the [command guide](../commands/check-persistence.md). The
[acknowledgements](acknowledgements.md) link the Python PyrsistenceSniper project and the original
PowerShell PersistenceSniper tool.

## Start an investigation

```bash
juntei check-persistence -d evidence/ --min-severity info -o PersistenceSniper.csv --coverage Coverage.csv
```

Use `--min-severity info` for the complete candidate inventory. The manual command defaults to
`medium`; its profile can classify partial allow matches as `LOW` even when missing payload or
signer evidence prevents a full comparison. Neither an allow match nor an unfamiliar entry settles
whether something is malicious. `auto-parse` already retains all severities and writes the findings
under each computer's `persistence/` category.

To focus on a mechanism, copy its **Check ID** from below:

```bash
juntei check-persistence --list-checks
juntei check-persistence -d evidence/ --check run_keys,scheduled_task_files --min-severity info -o focused.csv
```

The command uses a shared worker pool by default. Add `-T 4` to select four workers, or `-T 1`
for a single-thread scan. Input may contain several computers; their evidence stays separate.

## Read findings as evidence

| Question | What to inspect |
|---|---|
| What was configured, and where? | `CheckId`, `Path`, and the complete `Value`, including arguments. Follow `SourceFile` to the captured hive, file, or database. |
| Does the target exist in the collection? | `ResolvedPath`, `ResolutionStatus`, and `Exists`. Uncollected, ambiguous, or unsupported references are unknown; Juntei does not substitute a file from the analyst's machine. |
| Is the configuration effective? | The actual trigger, enable flags, selected control set, Windows/application version, user, and architecture. Some checks deliberately retain inactive configuration. |
| Is this an attack? | `Severity` and `PolicyReason` explain profile matching, not a verdict. Compare with expected software, the full command, file hashes, independent execution evidence, and the surrounding timeline. |
| Who could run it? | `AccessGained` is a scope/context hint. Machine-wide configuration often says `SYSTEM`, but that does not prove the launched process ran as SYSTEM. Task principals and BITS owner SIDs provide more specific context where available. |
| When did it change? | `KeyLastWrite` dates the whole registry key. `LastChange`, `ChangeSource`, and `ChangeEvidence` describe separate MFT/EVTX correlation, when available. Neither field automatically dates payload execution. |
| How reliable is the recovered state? | Read `RecordStatus`, `Coverage.csv`, and warnings for replayed, carved, damaged, missing, or limited evidence. A historical recovered record may no longer be active. |
| Is a named publisher trustworthy? | `Signer` and `SignerStatus` are descriptive PE/certificate/catalog metadata. Juntei does not validate Authenticode signatures or certificate trust. |

An absence of findings does not prove an absence of persistence. A `complete` coverage row means
the applicable collected source was read, not that the acquisition contains every relevant file,
every user, or every possible Windows persistence mechanism. Other coverage states explain missing,
partial, disabled, or failed checks. The hunter is not a live-memory, network, or cloud-identity scan.

## Catalogue and location notation

The catalogue includes the 130 checks from PyrsistenceSniper commit `b99c117` plus the native
`start_menu_com` check; the descriptions below document
Juntei's native behavior, including its evidence limitations. There are **116 registry-backed
checks and 15 file/repository checks**. Some file checks also join registry data, so these are
implementation counts rather than mutually exclusive acquisition requirements. The mechanisms
are grouped by investigative purpose below, and alphabetized by Check ID within each group.

- `HKLM\SYSTEM`, `HKLM\SOFTWARE`, and `HKLM\SAM` refer to captured machine hives, normally
  collected from `Windows\System32\config`, with transaction logs when available.
- `HKU\<user>` means the relevant captured user hive. Most user settings come from NTUSER.DAT;
  class registrations may come from `AppData\Local\Microsoft\Windows\UsrClass.dat` as noted.
- `<ControlSet>` means the selected active control set in SYSTEM. The DSRM check's additional
  historical locations are stated explicitly below. Check `SYSTEM\Select` when reviewing them.
- `<name>` is a variable path component. In registry target tables, `<child>` means immediate
  child registrations; it does not mean an unlimited recursive walk. Explicit recursive checks
  describe their traversal separately. Brackets in Office paths indicate an optional version.
- Tables list the actual machine and user views selected by the check. A 32-bit view is listed
  where implemented; do not assume every check examines every possible architecture/view.
- Generic value checks emit nonempty configured entries, including multi-string elements and
  numeric zero where applicable. They do not, by themselves, require a malicious payload, an
  enabled trigger, or a successful execution. More selective predicates are stated per check.

ATT&CK IDs are inherited catalogue mappings and links for further study. They describe potential
abuse, not a finding that an attacker performed that technique; protection-setting and diagnostic
checks may describe enabling conditions rather than a standalone persistence mechanism.

| Mechanism group | Checks |
|---|---:|
| [Boot, sign-in, and startup](#boot-sign-in-and-startup) | 32 |
| [Services and system extensions](#services-and-system-extensions) | 13 |
| [Scheduled tasks](#scheduled-tasks) | 3 |
| [WMI, BITS, and other event triggers](#wmi-bits-and-other-event-triggers) | 11 |
| [Explorer, COM, and file or URL handlers](#explorer-com-and-file-or-url-handlers) | 15 |
| [Office applications and add-ins](#office-applications-and-add-ins) | 7 |
| [Debuggers, crash handlers, and compatibility shims](#debuggers-crash-handlers-and-compatibility-shims) | 13 |
| [.NET runtime and profiler settings](#net-runtime-and-profiler-settings) | 7 |
| [DLL and executable lookup overrides](#dll-and-executable-lookup-overrides) | 18 |
| [Accounts and sign-in visibility](#accounts-and-sign-in-visibility) | 4 |
| [Authentication components and credential protection](#authentication-components-and-credential-protection) | 8 |
| **Total** | **131** |

## Boot, sign-in, and startup

These settings arrange for a command or library to run during boot, sign-in, or desktop initialization. Review the Windows version and the applicable user: a setting can be obsolete, disabled, or intended for a kiosk rather than an attacker.

### Active Setup {#active_setup}

**Check ID:** `active_setup` · **Catalogue ATT&CK:** [T1547.014](https://attack.mitre.org/techniques/T1547/014/)

**What Juntei checks:** Reads `StubPath` from machine-wide Active Setup component registrations in the native and 32-bit views. Windows can use these commands when preparing a user's profile; the check skips the literal placeholder values `U` and `/UserInstall`.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Active Setup\Installed Components\<component>\StubPath` and the corresponding `Wow6432Node` tree.

**How to investigate:** Compare each component with its installed product and inspect the command arguments. A machine-wide registration can lead to execution in a signing-in user's session; the output's `SYSTEM` label describes the configuration scope, not proof of the resulting process token.

### Boot Execute {#boot_execute}

**Check ID:** `boot_execute` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Reads the Session Manager `BootExecute` multi-string, preserving individual nonempty entries. It records commands configured for an early boot phase.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `BootExecute` |

**How to investigate:** `autocheck autochk *` is a familiar default, and disk tools can add legitimate entries. Investigate added commands and their files without assuming that every nondefault entry ran or is malicious.

### Boot Verification Program {#boot_verification_program}

**Check ID:** `boot_verification_program` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Reads `BootVerificationProgram`'s `ImagePath`, an executable location associated with boot verification.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\BootVerificationProgram` | `ImagePath` |

**How to investigate:** Check whether this older mechanism is applicable to the captured Windows version and boot configuration. Resolve the target file and compare its purpose with expected boot-management software.

### Explorer Load Value {#explorer_load}

**Check ID:** `explorer_load` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Reads the `Load` value associated with the Windows sign-in initialization configuration from machine and user hives.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |

**How to investigate:** Investigate unfamiliar commands, user-writable destinations, and arguments that fetch or decode content. Confirm applicability and execution separately; an empty result does not cover other sign-in mechanisms.

### Font Drivers {#font_drivers}

**Check ID:** `font_drivers` · **Catalogue ATT&CK:** [T1547](https://attack.mitre.org/techniques/T1547/)

**What Juntei checks:** Enumerates nonempty values in the legacy `Font Drivers` registry key as potential library registrations.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Font Drivers` | All nonempty values |

**How to investigate:** Treat this as an unusual legacy configuration surface. Check Windows-version support and the referenced library before concluding that it could load, and corroborate actual use with process or image-load evidence.

### Group Policy Extension DLLs {#gp_extension_dlls}

**Check ID:** `gp_extension_dlls` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads `DllName` from each immediate Group Policy extension registration under Winlogon. These DLLs extend policy processing.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\GPExtensions\<child>` | `DllName` |

**How to investigate:** Management and security products can install extensions. Match the extension GUID and DLL to the expected product, and examine policy-processing and image-load evidence before asserting execution.

### Group Policy Scripts {#gp_scripts}

**Check ID:** `gp_scripts` · **Catalogue ATT&CK:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**What Juntei checks:** Parses command/parameter pairs from the captured local Group Policy `scripts.ini` and `psscripts.ini` registration files, including per-user local policy stores. Startup/shutdown and logon/logoff sections retain their configured commands and arguments, with machine sections distinguished from user sections.

**Evidence examined:**

- `Windows\System32\GroupPolicy\Machine\Scripts\scripts.ini` / `psscripts.ini` and the equivalent `GroupPolicy\User\Scripts` files.
- `Windows\System32\GroupPolicyUsers\<SID>\User\Scripts\scripts.ini` / `psscripts.ini`.

**How to investigate:** Only the implemented local policy registration locations are examined; this is not a complete SYSVOL/GPO audit. A script file lying elsewhere under a policy directory is not sufficient. Correlate its registration with policy processing and execution evidence.

### Logon Scripts (UserInitMprLogonScript) {#logon_scripts}

**Check ID:** `logon_scripts` · **Catalogue ATT&CK:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**What Juntei checks:** Reads the user's `UserInitMprLogonScript` environment value, which can select a script for that user's sign-in.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKU\<user>\Environment` | `UserInitMprLogonScript` |

**How to investigate:** Compare with approved logon scripts and inspect the actual script. The value does not prove a subsequent sign-in or execution, and domain-supplied scripts require additional policy evidence.

### Platform Execute {#platform_execute}

**Check ID:** `platform_execute` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Reads each nonempty `PlatformExecute` entry in Session Manager. It identifies programs selected for platform-related boot initialization.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `PlatformExecute` |

**How to investigate:** Hardware or provisioning workflows may explain an entry. Determine whether the boot path on this Windows build uses it and investigate the referenced image rather than inferring an attack from the value name.

### RDP Client Extension DLL {#rdp_clx_dll}

**Check ID:** `rdp_clx_dll` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Reads `ClxDllPath` in Terminal Server's default user configuration as a possible RDP-related extension location.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\DefaultUserConfiguration` | `ClxDllPath` |

**How to investigate:** Check the Windows/RDP version and installed extensions. A configured DLL is a lead for image-load analysis; the registry value alone does not show that an RDP connection or DLL load occurred.

### RDP Virtual Channel DLL {#rdp_virtual_channel}

**Check ID:** `rdp_virtual_channel` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Enumerates values beneath machine-wide Terminal Server Client `Default\AddIns` registrations. Binary values are decoded as UTF-16LE when possible, preserving usable extension paths.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Terminal Server Client\Default\AddIns\<add-in>`: all values.

**How to investigate:** RDP clients and device-redirection products legitimately register add-ins. Inspect the actual DLL or command and the relevant client session; not every value under an add-in is an executable path.

### RDP WDS Startup Programs {#rdp_wds_startup}

**Check ID:** `rdp_wds_startup` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Reads `StartupPrograms` from the RDP Windows Station driver configuration. It identifies programs configured for remote-session startup.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\Wds\rdpwd` | `StartupPrograms` |

**How to investigate:** `rdpclip` is a familiar entry. Investigate additions and the host's RDP configuration, and use session/process records to determine whether the configured program actually started.

### Registry Run Keys {#run_keys}

**Check ID:** `run_keys` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Enumerates Run, RunOnce, RunEx, RunOnceEx, and policy Explorer Run values from machine/user hives, plus the listed machine 32-bit keys. RunEx/RunOnceEx sections are traversed with bounds and cycle checks.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | All nonempty values |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | All nonempty values |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | All nonempty values |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | All nonempty values |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | All nonempty values |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | All nonempty values |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | All nonempty values |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | All nonempty values |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | All nonempty values |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | All nonempty values |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Run` | All nonempty values |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnce` | All nonempty values |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnceEx` | All nonempty values |

**How to investigate:** Software updaters and tray applications commonly appear here. Compare the executable, command arguments, and user context. A RunOnce entry can disappear after use; the captured state is not a complete history of prior sign-ins.

### RunServices (Legacy) {#run_services}

**Check ID:** `run_services` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Enumerates values in the legacy machine-wide `RunServices` key.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServices` | All nonempty values |

**How to investigate:** This is a historical configuration surface, not a guarantee that a modern Windows build executes it. Investigate the target and operating-system applicability, then seek independent execution evidence.

### RunServicesOnce (Legacy) {#run_services_once}

**Check ID:** `run_services_once` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Enumerates values in the legacy machine-wide `RunServicesOnce` key.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServicesOnce` | All nonempty values |

**How to investigate:** Treat nonempty values as configuration leads. Check whether the source Windows version supports the mechanism and whether deployment or compatibility software explains the entry.

### S0 Initial Command {#s0_initial_command}

**Check ID:** `s0_initial_command` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Reads `S0InitialCommand` from Session Manager as a potential Session 0 initialization command.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `S0InitialCommand` |

**How to investigate:** Establish applicability to the captured Windows build and boot state. A configured command is not a timestamped record of boot execution; inspect its file and correlate with boot/process telemetry.

### Session Manager Execute {#session_manager_execute}

**Check ID:** `session_manager_execute` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Reads nonempty `Execute` entries from Session Manager, exposing programs selected for session initialization.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `Execute` |

**How to investigate:** Compare against the expected image baseline and inspect unusual native executables or paths. Establish whether a relevant boot/session occurred after the change before claiming execution.

### Session Manager SubSystems {#session_manager_subsystems}

**Check ID:** `session_manager_subsystems` · **Catalogue ATT&CK:** [T1547](https://attack.mitre.org/techniques/T1547/)

**What Juntei checks:** Reads the `Windows` value in Session Manager's `SubSystems` key, which configures the Windows subsystem command.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\SubSystems` | `Windows` |

**How to investigate:** The normal command is long and version dependent. Compare the full value and arguments with a trusted baseline of the same build; changing a critical subsystem setting can also break startup.

### Setup Execute {#setup_execute}

**Check ID:** `setup_execute` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Reads `SetupExecute` entries in Session Manager, covering setup-related commands configured for startup.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `SetupExecute` |

**How to investigate:** Installation, upgrade, and recovery activity may create legitimate entries. Compare with setup logs and the referenced image before attributing the configuration to persistence.

### Shell Folders Startup Redirect {#shell_folders_startup}

**Check ID:** `shell_folders_startup` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Compares `Startup` and `Common Startup` in both Shell Folders and User Shell Folders against default Startup locations. Nondefault redirections are reported, and their collected folder contents are inspected.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders` and `User Shell Folders`: `Common Startup`.
- Equivalent `Software\Microsoft\Windows\CurrentVersion\Explorer` settings in NTUSER.DAT: `Startup`, plus captured files in the redirected locations.

**How to investigate:** A redirected Startup folder can conceal autostarts, but redirection can be legitimate. The path must resolve within the captured evidence; an uncollected destination is unknown, not proof that the folder was empty.

### Shell Launcher Override {#shell_launcher}

**Check ID:** `shell_launcher` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Reads the policy `Shell` value and the `system.ini` boot Shell mapping from SOFTWARE. These are alternative shell-selection surfaces.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `Shell` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\IniFileMapping\system.ini\boot` | `Shell` |

**How to investigate:** Kiosks and application appliances can intentionally replace Explorer. Compare the selected shell with the machine's role and confirm the setting was effective for the relevant session.

### Startup Folder {#startup_folder}

**Check ID:** `startup_folder` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Inspects direct files in default per-user and common Startup folders, following the first available Startup-folder redirection. Shortcut targets are recovered when a collected `.lnk` can be parsed.

**Evidence examined:**

- `ProgramData\Microsoft\Windows\Start Menu\Programs\Startup` and `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`.
- Applicable `Common Startup` / `Startup` redirections in machine/user Shell Folders and User Shell Folders settings.

**How to investigate:** Startup links and files can launch legitimate applications at sign-in. Inspect target arguments as well as filenames. The common folder is a shared configuration location; it does not prove that the launched program used a SYSTEM token.

### Terminal Services Initial Program {#ts_initial_program}

**Check ID:** `ts_initial_program` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Reads the Terminal Services policy `InitialProgram` and the machine Terminal Server installation Run/RunOnce/RunOnceEx configuration, including bounded nested RunOnceEx sections.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` | `InitialProgram` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Run` | All nonempty values |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Runonce` | All nonempty values |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx` | All nonempty values |

**How to investigate:** Remote application deployments may use these settings intentionally. Establish the affected sessions and follow the selected commands into process evidence; the registration alone does not show a remote logon.

### Winlogon AppSetup {#winlogon_appsetup}

**Check ID:** `winlogon_appsetup` · **Catalogue ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**What Juntei checks:** Reads `AppSetup` from Winlogon, splitting comma-separated entries into separate candidates.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `AppSetup` |

**How to investigate:** Compare each component with expected Windows setup/sign-in software. A list entry is not a separate observed execution, and its relevance can depend on Windows version and deployment state.

### Winlogon GinaDLL {#winlogon_gina_dll}

**Check ID:** `winlogon_gina_dll` · **Catalogue ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**What Juntei checks:** Reads the legacy Winlogon `GinaDLL` replacement setting. GINA was an older Windows sign-in extension model.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `GinaDLL` |

**How to investigate:** This is primarily relevant to pre-Vista systems. A leftover key on a newer system is not proof of an effective logon extension; inspect the source version and any referenced DLL.

**Further details:** [Microsoft documentation](https://learn.microsoft.com/en-us/windows/win32/secauthn/gina).

### Winlogon MPNotify {#winlogon_mpnotify}

**Check ID:** `winlogon_mpnotify` · **Catalogue ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**What Juntei checks:** Reads Winlogon's `mpnotify` command/library setting, a sign-in-related extension surface.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `mpnotify` |

**How to investigate:** Compare the target and arguments with a matching Windows baseline. Check version-specific behavior and correlate with a subsequent sign-in before asserting that code ran.

### Winlogon Notify Packages {#winlogon_notify_packages}

**Check ID:** `winlogon_notify_packages` · **Catalogue ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**What Juntei checks:** Reads `DllName` from each immediate Winlogon `Notify` package registration.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Notify\<child>` | `DllName` |

**How to investigate:** Notification packages are a legacy extension mechanism. Determine whether the captured Windows version loads them and distinguish approved authentication software from unexplained libraries.

### Winlogon Shell {#winlogon_shell}

**Check ID:** `winlogon_shell` · **Catalogue ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**What Juntei checks:** Reads machine-wide and per-user Winlogon `Shell` values to expose desktop shell selection.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |

**How to investigate:** Explorer is common, but kiosks can use another shell. Investigate added commands or unusual paths and determine which user's effective configuration applied at sign-in.

### Winlogon System {#winlogon_system}

**Check ID:** `winlogon_system` · **Catalogue ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**What Juntei checks:** Reads Winlogon's `System` value and separates comma-delimited entries into individual candidates.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `System` |

**How to investigate:** Treat this as a version-sensitive sign-in configuration lead. Inspect every referenced component and corroborate with execution evidence rather than treating the list as a run history.

### Winlogon Taskman {#winlogon_taskman}

**Check ID:** `winlogon_taskman` · **Catalogue ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**What Juntei checks:** Reads machine/user Winlogon `Taskman` values for an alternate task-management command.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |

**How to investigate:** Determine which Windows behavior would consume the setting and whether the target is expected. A configured replacement is not proof that a user invoked that feature.

### Winlogon Userinit {#winlogon_userinit}

**Check ID:** `winlogon_userinit` · **Catalogue ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**What Juntei checks:** Reads machine/user Winlogon `Userinit` values, splitting comma-separated programs while ignoring empty trailing items.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |

**How to investigate:** A normal entry often points to `userinit.exe`. Inspect additional entries and altered paths. Confirm the applicable user and a later sign-in; `KeyLastWrite` belongs to the whole key, not specifically this value.

### Winlogon VMApplet {#winlogon_vmapplet}

**Check ID:** `winlogon_vmapplet` · **Catalogue ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**What Juntei checks:** Reads Winlogon's `VMApplet` command setting, covering an older system applet launch surface.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `VMApplet` |

**How to investigate:** Check operating-system applicability and the expected applet. An unexpected command merits investigation but does not establish that the triggering interface was opened.

## Services and system extensions

Windows services and extensible system components provide many legitimate ways to load executables or DLLs. Compare the configured target with the expected product, service account, start settings, and source-backed installation history.

### DiagTrack DLL {#diagtrack_dll}

**Check ID:** `diagtrack_dll` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads the DiagTrack service's `ImagePath`. Despite the catalogue's DLL-oriented name, this check examines the configured service image command.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DiagTrack` | `ImagePath` |

**How to investigate:** Compare with the service baseline and the general service checks. The same configuration may produce more than one check result; candidate counts are not counts of distinct attacks.

### DiagTrack Listener DLL {#diagtrack_listener_dll}

**Check ID:** `diagtrack_listener_dll` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads `FileName` from the DiagTrack-Listener autologger configuration. This is an ETW autologger setting, not evidence of a DLL being loaded.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\WMI\Autologger\DiagTrack-Listener` | `FileName` |

**How to investigate:** Inspect the raw value and Windows logging configuration. The catalogue labels this as a DLL technique, but the lookup does not validate a loadable DLL or prove execution through this value.

### Netsh Helper DLL {#netsh_helper}

**Check ID:** `netsh_helper` · **Catalogue ATT&CK:** [T1546.007](https://attack.mitre.org/techniques/T1546/007/)

**What Juntei checks:** Enumerates nonempty values under the machine `NetSh` helper registration key. Helpers can extend `netsh.exe` with DLL-based commands.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\NetSh` | All nonempty values |

**How to investigate:** Network products may install helpers. Match each value to an expected product and inspect the resolved library, then look for relevant netsh execution or image loads.

### Print Monitors {#print_monitors}

**Check ID:** `print_monitors` · **Catalogue ATT&CK:** [T1547.010](https://attack.mitre.org/techniques/T1547/010/)

**What Juntei checks:** Reads `Driver` beneath each immediate Print Monitor registration in the active control set.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Monitors\<child>` | `Driver` |

**How to investigate:** Printing software and port monitors legitimately extend the spooler. Compare the DLL and product metadata with the printer inventory, and confirm loading with spooler/image-load evidence.

### Print Processors {#print_processors}

**Check ID:** `print_processors` · **Catalogue ATT&CK:** [T1547.012](https://attack.mitre.org/techniques/T1547/012/)

**What Juntei checks:** Reads `Driver` under the x64 and x86 print-processor registration trees.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows x64\Print Processors\<child>` | `Driver` |
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows NT x86\Print Processors\<child>` | `Driver` |

**How to investigate:** Third-party printer suites can add processors beyond the familiar Windows defaults. Review both architectures, the actual DLL, and installed printer software before assigning malicious intent.

### Service Control Manager Extension {#scm_extension}

**Check ID:** `scm_extension` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Enumerates nonempty values in `ServiceControlManagerExtension`, a service-manager extension surface.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\ServiceControlManagerExtension` | All nonempty values |

**How to investigate:** Examine the named libraries and whether this Windows build consumes the setting. Correlate with boot/service activity; the check inventories registrations rather than proving a loaded extension.

### DNS Server Level Plugin DLL {#server_level_plugin_dll}

**Check ID:** `server_level_plugin_dll` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads the DNS service's `ServerLevelPluginDll` setting in the active SYSTEM control set.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DNS\Parameters` | `ServerLevelPluginDll` |

**How to investigate:** This is relevant to systems running the DNS Server role, including some domain controllers. Investigate the DLL, service account, and service restart/load evidence; an inactive or absent role changes the conclusion.

### Service Failure Command {#service_failure_command}

**Check ID:** `service_failure_command` · **Catalogue ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**What Juntei checks:** Reads each service's `FailureCommand` in the active control set, exposing a configured recovery command.

**Evidence examined:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\FailureCommand`.

**How to investigate:** The command only runs when the service's recovery actions select it and the applicable failure occurs. Inspect recovery configuration and failure/process records; the value alone does not establish an active recovery action.

### SNMP Extension Agent DLL {#snmp_extension_agent}

**Check ID:** `snmp_extension_agent` · **Catalogue ATT&CK:** [T1574](https://attack.mitre.org/techniques/T1574/)

**What Juntei checks:** Enumerates values under the SNMP service's `ExtensionAgents` registration key. Values may identify another registry location instead of a direct DLL path.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\SNMP\Parameters\ExtensionAgents` | All nonempty values |

**How to investigate:** Follow an indirect registration manually to its library and verify that SNMP is installed and used. An unresolved string in this output is not proof that the DLL is missing or malicious.

### Time Providers {#time_providers}

**Check ID:** `time_providers` · **Catalogue ATT&CK:** [T1547.003](https://attack.mitre.org/techniques/T1547/003/)

**What Juntei checks:** Reads `DllName` from immediate Windows Time `TimeProviders` registrations.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\W32Time\TimeProviders\<child>` | `DllName` |

**How to investigate:** Review provider enable settings and the Windows Time service state separately; the DLL path check does not require an enabled provider. Time synchronization products may supply legitimate replacements.

### Windows Service (ServiceDll) {#windows_service_dll}

**Check ID:** `windows_service_dll` · **Catalogue ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**What Juntei checks:** Reads `Parameters\ServiceDll` beneath services in the active SYSTEM control set and attaches service-specific timestamp evidence when available.

**Evidence examined:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Parameters\ServiceDll`.

**How to investigate:** A shared host such as `svchost.exe` is not the payload DLL. Inspect the library, service account, start state, and matching service installation/image-load evidence; a registration does not prove that the service ran.

**Further details:** [Microsoft documentation](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### Windows Service (ImagePath) {#windows_service_image_path}

**Check ID:** `windows_service_image_path` · **Catalogue ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**What Juntei checks:** Reads service `ImagePath` commands in the active control set. Driver types 1, 2, or 8 without ImagePath retain an implied `System32\drivers\<service>.sys` target.

**Evidence examined:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>`: `ImagePath`, `Type`, and supporting service metadata.

**How to investigate:** Review the service account and start settings separately from the output's machine scope label. Disabled or demand-start registrations are still inventory. Service-name evidence is preferred over ambiguous shared executable paths for timestamps.

**Further details:** [Microsoft documentation](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### Windows Update Service Startup DLL {#wu_service_startup_dll}

**Check ID:** `wu_service_startup_dll` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads the Windows Update service's `Parameters\ServiceDll` from the active control set.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\wuauserv\Parameters` | `ServiceDll` |

**How to investigate:** Compare the library with the same Windows build and installed servicing state. This overlaps the generic ServiceDll inventory and does not represent a second distinct compromise.

## Scheduled tasks

Task Scheduler stores configuration in both XML files and the SOFTWARE hive. Read those sources together. A task can be registered without having run, and an incomplete collection can resemble a hidden or ghost task.

### Ghost Scheduled Task {#ghost_task}

**Check ID:** `ghost_task` · **Catalogue ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**What Juntei checks:** Compares SOFTWARE TaskCache `Tree` and `Tasks` registrations and, when task files exist in the collection, looks for their corresponding task XML files.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree` and `TaskCache\Tasks`.
- Captured `Windows\System32\Tasks\<task path>` files for the filesystem comparison.

**How to investigate:** Missing counterparts can reflect deleted/stale tasks, built-in registrations, or an incomplete collection. No collected XML means the filesystem comparison is unavailable. Inspect coverage and the exact discrepancy before calling a task hidden or malicious.

### Hidden Scheduled Task {#hidden_scheduled_task}

**Check ID:** `hidden_scheduled_task` · **Catalogue ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**What Juntei checks:** Checks TaskCache Tree registrations for a missing `SD` value or an empty binary security descriptor, conditions associated with hiding a task from ordinary enumeration.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\<task>`: `SD` presence/data, with TaskCache counterpart metadata.

**How to investigate:** Missing/empty SD is a specific configuration anomaly, not proof that the task executed or that it is malicious. The XML `Hidden` setting and arbitrary ACL problems are different conditions and are not the predicate used here.

**Further details:** [Microsoft documentation](https://www.microsoft.com/en-us/security/blog/2022/04/12/tarrask-malware-uses-scheduled-task-defense-evasion/).

### Scheduled Task (XML Files) {#scheduled_task_files}

**Check ID:** `scheduled_task_files` · **Catalogue ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**What Juntei checks:** Parses captured task XML for Exec commands/arguments and COM-handler class IDs/data. It selects the principal by `Actions Context`/`Principal id`, retains user and group identity separately, and reports SYSTEM, USER, or UNKNOWN. Unresolved COM actions remain visible.

**Evidence examined:**

- `Windows\System32\Tasks\<task path>` XML files, including Exec, ComHandler, and available principal fields.
- Matching user's captured `UsrClass.dat` and NTUSER `Software\Classes`, plus machine SOFTWARE `Classes\CLSID`, in native and 32-bit views.
- Captured SOFTWARE `ProfileList` for SID principals. Unique collected `user` or `user.DOMAIN` folder names provide explicitly labeled heuristic matches for named principals; ambiguous matches remain unresolved. Volatile registry keys are not required.
- Managed `CodeBase` (including version subkeys), AppID associations, and SYSTEM service `ImagePath` for an AppID `LocalService`.

COM server candidates include their registry source in `Description`. Executable-name-only
AppID associations remain unresolved rather than being assigned a guessed directory. Local
`file:` URLs are decoded; remote URLs are retained without network access. User class overrides
are scoped to that profile, and an incomplete user CLSID does not fall through to its shadowed
machine registration. Group-only tasks retain machine candidates with the individual-user
limitation in their description, without adding routine notices to the coverage error summary.

**How to investigate:** Inspect triggers, enabled state, principal, and run history in the source XML and event logs. A created task may never run. An unresolved COM handler is still a candidate, and missing/group-only principal data must not be assumed to mean SYSTEM.

## WMI, BITS, and other event triggers

Windows Management Instrumentation (WMI) supports system management. Background Intelligent Transfer Service (BITS) supports background transfers. Both can retain event-triggered commands. Some mechanisms wait for a management event, transfer notification, application launch, or user action. A stored trigger is useful configuration evidence, but its presence does not establish that the trigger fired.

### Accessibility Features Backdoor {#accessibility_tools}

**Check ID:** `accessibility_tools` · **Catalogue ATT&CK:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**What Juntei checks:** Compares seven collected System32 accessibility binaries using PE OriginalFilename and certificate/catalog metadata, after requiring usable Microsoft catalog metadata from a collected reference binary.

**Evidence examined:**

- `Windows\System32\sethc.exe`, `osk.exe`, `narrator.exe`, `magnify.exe`, `utilman.exe`, `atbroker.exe`, and `DisplaySwitch.exe`.
- Collected `Windows\System32\winlogon.exe`, `cmd.exe`, or `notepad.exe` reference metadata and applicable collected catalogs.

**How to investigate:** Metadata can suggest a renamed replacement, but Juntei does not validate signatures or certificate trust. A sparse collection without the needed catalogs produces incomplete comparison coverage rather than treating every unsigned-looking file as a backdoor.

### Accessibility Features {#assistive_technology}

**Check ID:** `assistive_technology` · **Catalogue ATT&CK:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**What Juntei checks:** Reads registered accessibility `StartExe` plus `StartParams` commands in native/32-bit SOFTWARE views, and enabled accessibility `Configuration` entries from user and DEFAULT hives. Numeric/nonstring StartExe placeholders are skipped.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\<feature>`: `StartExe` and `StartParams`; also the machine `Wow6432Node` view.
- `HKU\<user>\Software\Microsoft\Windows NT\CurrentVersion\Accessibility\Configuration` in NTUSER.DAT and the equivalent setting in the DEFAULT hive.

**How to investigate:** Accessibility software is legitimate. Distinguish an enabled feature name from its actual executable registration, then inspect the target and sign-in/desktop execution evidence. This is separate from replacing the accessibility executable itself.

**Further details:** [Technique reference (hexacorn.com)](https://www.hexacorn.com/blog/2016/07/22/beyond-good-ol-run-key-part-42/).

### BITS Job Notification Command {#bits_notify_command}

**Check ID:** `bits_notify_command` · **Catalogue ATT&CK:** [T1197](https://attack.mitre.org/techniques/T1197/)

**What Juntei checks:** Recovers counted-string BITS job records from captured `qmgr.db`, `qmgr0.dat`, or `qmgr1.dat`, retaining nonempty notification programs, parameters, job IDs, and owner SIDs.

**Evidence examined:**

- `ProgramData\Microsoft\Network\Downloader\qmgr.db`, `qmgr0.dat`, and `qmgr1.dat`.

**How to investigate:** These are carved records: allocated/current job state is not validated and stale/deleted jobs may appear. The database's MFT write time is weak container evidence, not the notification's execution time. Owner SIDs distinguish SYSTEM, local service, network service, and users.

### Command Processor AutoRun {#cmd_autorun}

**Check ID:** `cmd_autorun` · **Catalogue ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**What Juntei checks:** Reads machine/user Command Processor `AutoRun` commands, a setting that can affect command-prompt startup.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |
| `HKU\<user>\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |

**How to investigate:** Development tools and shell customization may explain a value. `cmd.exe /d` bypasses AutoRun, so configuration alone does not prove it ran whenever cmd.exe appeared. Inspect the command content and triggering process arguments.

**Further details:** [Microsoft documentation](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd).

### ErrorHandler.cmd Persistence {#error_handler_cmd}

**Check ID:** `error_handler_cmd` · **Catalogue ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**What Juntei checks:** Reports the presence of the collected `Windows\System32\ErrorHandler.cmd` script as an error-handler persistence candidate.

**Evidence examined:**

- `Windows\System32\ErrorHandler.cmd`.

**How to investigate:** Presence alone does not prove the applicable Windows recovery/error path invoked it. Read the script, inspect its provenance, and correlate with setup/error/process evidence.

### Power Automate Desktop Flows {#power_automate}

**Check ID:** `power_automate` · **Catalogue ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**What Juntei checks:** Finds captured flow/script directories beneath each user's Power Automate Desktop `flows` and `scripts` stores, reporting each discovered flow once.

**Evidence examined:**

- `Users\<user>\AppData\Local\Microsoft\Power Automate Desktop\flows\<flow>\...` and `scripts\<flow>\...`.

**How to investigate:** This identifies stored automation, not every scheduled trigger or an executed flow. Inspect the flow contents and external launch configuration before treating it as automatic or malicious persistence.

### PowerShell Profile {#powershell_profiles}

**Check ID:** `powershell_profiles` · **Catalogue ATT&CK:** [T1546.013](https://attack.mitre.org/techniques/T1546/013/)

**What Juntei checks:** Finds profile.ps1-style files in recognized Windows PowerShell and PowerShell installation/profile directories, including user Documents, discovered OneDrive Documents, and collected Documents redirections.

**Evidence examined:**

- Files ending in `profile.ps1` directly in `Windows\System32\WindowsPowerShell\v1.0` and `Windows\SysWOW64\WindowsPowerShell\v1.0`.
- Per-user `Documents\WindowsPowerShell` and `Documents\PowerShell`, including captured Documents redirections and discovered OneDrive Documents folders.
- Files ending in `profile.ps1` directly in version directories under `Program Files\PowerShell` or `Program Files (x86)\PowerShell`.

**How to investigate:** Profiles can contain harmless customization. Read the script and follow invoked payloads. Launching PowerShell with `-NoProfile` skips them; file presence alone does not prove profile execution or malicious content.

### Screensaver Hijack {#screensaver}

**Check ID:** `screensaver` · **Catalogue ATT&CK:** [T1546.002](https://attack.mitre.org/techniques/T1546/002/)

**What Juntei checks:** Reads each user's Desktop `SCRNSAVE.EXE` value, identifying the configured screensaver image.

**Evidence examined:**

- `HKU\<user>\Control Panel\Desktop\SCRNSAVE.EXE` in NTUSER.DAT.

**How to investigate:** Compare with approved screensavers and inspect the target. Other desktop settings, session state, and timeout determine whether it can trigger; a saved path does not prove a screensaver started.

### Telemetry Controller Command {#telemetry_controller}

**Check ID:** `telemetry_controller` · **Catalogue ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**What Juntei checks:** Reads `Binary`, `Command`, and `MaintenanceCommand` from TelemetryController and its immediate child registrations.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController` and immediate children: `Binary`, `Command`, `MaintenanceCommand`.

**How to investigate:** Compare the selected executable and arguments with the same Windows build and maintenance software. A registry command does not demonstrate that a maintenance trigger fired.

### Windows Terminal Custom Profiles {#windows_terminal}

**Check ID:** `windows_terminal` · **Catalogue ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**What Juntei checks:** Parses packaged, preview, and unpackaged Windows Terminal JSON/JSONC settings and reports nonempty profile/default `commandline` settings.

**Evidence examined:**

- `Users\<user>\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json` and the WindowsTerminalPreview equivalent.
- `Users\<user>\AppData\Local\Microsoft\Windows Terminal\settings.json`: legacy profiles arrays, `profiles.list`, and `profiles.defaults` commandline fields.

**How to investigate:** Ordinary shells naturally appear here. Determine the selected/default profile and inspect unusual commands or launch arguments. Merely saving a profile does not launch it.

### WMI Event Subscription {#wmi_event_subscription}

**Check ID:** `wmi_event_subscription` · **Catalogue ATT&CK:** [T1546.003](https://attack.mitre.org/techniques/T1546/003/)

**What Juntei checks:** Carves recognized event-consumer instances and filter-to-consumer binding references from the captured WMI OBJECTS.DATA repository. Recognized classes include command-line, active-script, log-file, event-log, and SMTP consumers.

**Evidence examined:**

- `Windows\System32\wbem\Repository\OBJECTS.DATA` and `Repository\FS\OBJECTS.DATA`.

**How to investigate:** A raw repository hit may be stale, unbound, or historical; the output preserves that qualification. Carving does not reconstruct a complete active subscription graph or prove the trigger fired. The repository's file write time is not a subscription creation/execution time.

## Explorer, COM, and file or URL handlers

Component Object Model (COM) lets applications activate registered software components. The Windows shell uses registered handlers to open files, URLs, folders, and special objects. COM identifiers (CLSIDs) identify components; ProgIDs give some components readable names. Follow each registration to the command or DLL it selects.

### App Paths Hijack {#app_paths}

**Check ID:** `app_paths` · **Catalogue ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**What Juntei checks:** Reads default values beneath machine-wide `App Paths` registrations, which can redirect application lookup for shell-based launches.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\<application>` default value.

**How to investigate:** An App Paths registration is common for installed applications and is not a general rewrite of every process launch. Compare the registered name and image with expected software, then examine relevant shell/process evidence.

### COM Object Hijack {#com_treat_as}

**Check ID:** `com_treat_as` · **Catalogue ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**What Juntei checks:** Finds COM `TreatAs` redirections and per-user InprocServer32/LocalServer32 registrations that shadow a machine CLSID, across supported native/32-bit class stores. TreatAs targets are resolved using the same user's classes and machine classes.

**Evidence examined:**

- `HKLM\SOFTWARE\Classes\CLSID\<CLSID>` and the machine `Classes\Wow6432Node` view.
- Per-user `Software\Classes\CLSID` in NTUSER.DAT and `CLSID` in UsrClass.dat, including their implemented `Wow6432Node` views: `TreatAs`, `InprocServer32`, and `LocalServer32` default values.

**How to investigate:** Per-user overrides can require no administrator rights, but legitimate software can register them too. Follow the selected class and DLL/executable, assess architecture and activation context, and corroborate actual COM activation independently.

### Disk Cleanup Handler Hijack {#disk_cleanup_handler}

**Check ID:** `disk_cleanup_handler` · **Catalogue ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**What Juntei checks:** Reads Disk Cleanup VolumeCaches handler CLSIDs and follows resolvable machine COM InprocServer32 registrations to their libraries.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\<handler>` default CLSID and `Classes\CLSID\<CLSID>\InprocServer32`.

**How to investigate:** The handler must be invoked by the relevant cleanup workflow to run. Unresolved handlers are not emitted by this particular check, so use the registry parser to inspect their raw registrations if the capture is incomplete.

### Explorer AppKey Override {#explorer_app_key}

**Check ID:** `explorer_app_key` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Reads `ShellExecute` and `Association` values beneath immediate Explorer `AppKey` children, covering commands/actions selected by application keys.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\AppKey\<key>`: `ShellExecute` and `Association`.

**How to investigate:** Keyboard utilities and OEM tools can define these values. An association name may require another registry pivot rather than being a file path. Check the user action that would trigger it.

### Browser Helper Objects {#explorer_bho}

**Check ID:** `explorer_bho` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Reads Browser Helper Object CLSID registrations and resolves machine InprocServer32 libraries when available; unresolved class IDs remain visible.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects\<CLSID>` and the machine `Classes\CLSID\<CLSID>\InprocServer32` default value.

**How to investigate:** This is tied to compatible browser/Explorer behavior and is especially relevant to legacy environments. Approved browser/security extensions can appear. A registered BHO is not proof that a current browser loaded it.

### Explorer Desktop CLSID Hijack {#explorer_clsid_hijack}

**Check ID:** `explorer_clsid_hijack` · **Catalogue ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**What Juntei checks:** Reads configured shell-open commands on the three listed special Explorer namespace CLSIDs from machine/user class stores.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |

**How to investigate:** Compare command handlers for Desktop, My Computer, and My Documents with a matching baseline. This checks selected CLSIDs and paths, not every possible Explorer namespace hijack.

### Explorer Context Menu Handlers {#explorer_context_menu}

**Check ID:** `explorer_context_menu` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Enumerates context-menu handlers for Directory, Directory Background, Folder, Drive, AllFilesystemObjects, and `*`, following class IDs to collected DLL registrations when possible.

**Evidence examined:**

- Machine `HKLM\SOFTWARE\Classes` and user UsrClass.dat class stores: `<subject>\shellex\ContextMenuHandlers\<handler>` default value and corresponding COM library registrations.

**How to investigate:** Right-click extensions from archive tools, security products, and cloud clients are common. Inspect the actual handler and triggering interaction. Unresolved GUIDs remain useful leads rather than proof that the DLL did not exist.

### File Association Hijacking {#file_association_hijack}

**Check ID:** `file_association_hijack` · **Catalogue ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**What Juntei checks:** Examines direct extension and resolved ProgID `shell\open\command` registrations for `.txt`, `.pdf`, `.doc`, `.docx`, `.html`, `.htm`, `.js`, `.vbs`, `.hta`, `.exe`, `.bat`, `.cmd`, and `.ps1`.

**Evidence examined:**

- Machine `HKLM\SOFTWARE\Classes` and user UsrClass.dat class stores: selected `<extension>` defaults and `<extension-or-ProgID>\shell\open\command` defaults.

**How to investigate:** Compare with the expected application and inspect command arguments. This is a defined extension set, not all file types or the full effective UserChoice resolution. A saved command does not prove a document was opened.

### Protocol Handler Hijacking {#protocol_handler_hijack}

**Check ID:** `protocol_handler_hijack` · **Catalogue ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**What Juntei checks:** Reads URL open commands for http, https, mailto, ms-msdt, and ms-officecmd, plus discovered `URL Protocol` registrations and applicable user overrides.

**Evidence examined:**

- Machine `HKLM\SOFTWARE\Classes` and user UsrClass.dat class stores: `URL Protocol` markers and `<scheme>\shell\open\command` default values.

**How to investigate:** Browsers and application installers legitimately register schemes. Follow the command and arguments, assess the scheme's expected purpose, and correlate with URL opening/process evidence.

### Recycle Bin COM Extension Handler {#recycle_bin_com_extension}

**Check ID:** `recycle_bin_com_extension` · **Catalogue ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**What Juntei checks:** Reads the listed Recycle Bin CLSID open/empty/explore command overrides and immediate context-menu/drag-drop handler registrations.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\empty\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\explore\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\ContextMenuHandlers\<child>` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\DragDropHandlers\<child>` | `(Default)` |

**How to investigate:** Compare with approved shell extensions. Some reported values are COM identifiers rather than a directly resolved library; investigate the referenced class and user action before asserting code execution.

### Search Protocol Handler Hijack {#search_protocol_handler}

**Check ID:** `search_protocol_handler` · **Catalogue ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**What Juntei checks:** Reads the machine/user `search-ms\shell\open\command` default handler.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Classes\search-ms\shell\open\command` | `(Default)` |
| `HKU\<user>\Software\Classes\search-ms\shell\open\command` | `(Default)` |

**How to investigate:** A handler change can alter what happens when a search-ms URL opens, but its presence is not proof of exploit delivery. Compare with Windows defaults and investigate the URL/process activity that would consume it.

### SharedTaskScheduler {#shared_task_scheduler}

**Check ID:** `shared_task_scheduler` · **Catalogue ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**What Juntei checks:** Enumerates nonempty Explorer `SharedTaskScheduler` values as registered shell components.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SharedTaskScheduler` | All nonempty values |

**How to investigate:** Values may reference COM classes rather than an executable path. Follow their server registrations and check operating-system behavior, extension provenance, and relevant shell loading evidence.

### ShellExecuteHooks {#shell_execute_hooks}

**Check ID:** `shell_execute_hooks` · **Catalogue ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**What Juntei checks:** Enumerates nonempty Explorer `ShellExecuteHooks` values, covering legacy shell-execution hook registrations.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ShellExecuteHooks` | All nonempty values |

**How to investigate:** Resolve any referenced CLSID and verify whether the Windows build and policy permit the hook. A legacy registration can be unused; the value itself is not an observed shell execution.

### Start Menu Library and CLSID Folder COM Persistence {#start_menu_com}

**Check ID:** `start_menu_com` · **Catalogue ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**What Juntei checks:** Finds `.library-ms` library XML files whose `simpleLocation` URL references
a COM CLSID, and directories named `<name>.{CLSID}`, beneath
`Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu`. Explorer interaction with these
objects can activate COM code. The check includes empty captured directories; it does not follow
symbolic links. Ordinary library files outside the Start Menu and GUIDs in XML comments are not
reported by this check.

**Evidence examined:**

- Per-user Start Menu `.library-ms` files and CLSID-named directory entries.
- Matching user's captured class registrations and machine SOFTWARE, separately by architecture.
- Managed CodeBase/AppID/service configuration through the same resolver as scheduled tasks.

**How to investigate:** `Path` and `SourceFile` identify the activation-trigger artifact. `Value`
contains the candidate server command, or the CLSID if resolution is unavailable. `Description`
links the trigger to the CLSID and captured registry source. `ResolvedPath`/`Sha256` describe only
a resolved, collected payload. An executable-name-only AppID association remains unresolved.
A trigger can be legitimate; existence does not prove a sign-in, activation, execution, or attack.
Inspect the library URL, class registration, payload, user context and independent execution logs.

**Collection cautions:** File-only collections can omit empty directories. Standard user Start Menu
paths are examined; redirected Start Menus and other Explorer namespace locations are not covered
by this check. Missing hives or unresolved registrations produce partial coverage while retaining
the candidate. Task architecture, elevation and policy can change effective COM lookup behavior.

**Further details:** [Original proposal and researcher credit](https://github.com/last-byte/PersistenceSniper/pull/32),
[COM per-user/machine merged views](https://learn.microsoft.com/en-us/windows/win32/sysinfo/merged-view-of-hkey-classes-root),
[MITRE COM hijacking](https://attack.mitre.org/techniques/T1546/015/).

### TypeLib COM Hijacking {#typelib_hijack}

**Check ID:** `typelib_hijack` · **Catalogue ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**What Juntei checks:** Reads per-user UsrClass.dat TypeLib GUID/version/locale win32/win64 registrations, retaining paths and script-moniker values. Numeric trailing resource indices are removed only for target-file resolution.

**Evidence examined:**

- `TypeLib\<GUID>\<version>\<locale>\win32` and `win64` default values in each captured UsrClass.dat.

**How to investigate:** A type library can point into a legitimate DLL, an alternate file, or a script moniker. The check inventories these registrations; assess the affected application and corroborate type-library loading rather than assuming every nondefault path is an attack.

## Office applications and add-ins

Office supports legitimate add-ins, templates, and application-specific extensions. A suspicious registration may depend on a particular Office version, application, loading setting, or user interaction. Configuration alone does not establish macro execution.

### Office Add-in Registration {#office_addins}

**Check ID:** `office_addins` · **Catalogue ATT&CK:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**What Juntei checks:** Enumerates Word, Excel, PowerPoint, Outlook, and Access add-ins in unversioned and 14.0/15.0/16.0 Office roots, including the implemented machine Click-to-Run store and user roots. It retains explicit Manifest/FileName/Path values or resolves eligible COM-backed entries, with LoadBehavior context.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` and the implemented machine Click-to-Run equivalent.
- `HKU\<user>\Software\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` in NTUSER.DAT; matching machine/user COM classes from SOFTWARE, NTUSER.DAT, and UsrClass.dat.

**How to investigate:** Add-ins are common business software. Review LoadBehavior, application/version, manifests, actual payloads, and expected vendor. The check does not guarantee that every installed add-in format or registration view has been collected or loaded.

### Office AI Add-in Hijack {#office_ai_hijack}

**Check ID:** `office_ai_hijack` · **Catalogue ATT&CK:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**What Juntei checks:** Enumerates values in the Office 16.0 Click-to-Run Common AI registration location.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI`: all values.

**How to investigate:** Compare unusual values and paths with the installed Office build. The presence of a value does not show which Office feature consumed it or prove it selected executable code.

### Office DLL Override {#office_dll_override}

**Check ID:** `office_dll_override` · **Catalogue ATT&CK:** [T1137](https://attack.mitre.org/techniques/T1137/)

**What Juntei checks:** Reads `WwlibtDll` and `PPCoreTDLL` under Word/PowerPoint roots for unversioned and 14.0/15.0/16.0 settings, including the implemented Click-to-Run and user roots.

**Evidence examined:**

- Machine/user `Microsoft\Office\[<version>\]Word` and `PowerPoint` settings, plus the implemented machine Click-to-Run Office tree: `WwlibtDll` and `PPCoreTDLL`.

**How to investigate:** Determine which application/build recognizes the override, inspect the library and provenance, and correlate with an Office launch or image load. A configured override alone is not execution proof.

### Office Default Templates {#office_templates}

**Check ID:** `office_templates` · **Catalogue ATT&CK:** [T1137.001](https://attack.mitre.org/techniques/T1137/001/)

**What Juntei checks:** Inventories collected Normal.dotm/NormalEmail.dotm files and files directly in recognized Word Startup and Excel XLSTART folders, including Office installation startup folders.

**Evidence examined:**

- `Users\<user>\AppData\Roaming\Microsoft\Templates\Normal.dotm` and `NormalEmail.dotm`.
- Direct files in user `Microsoft\Word\STARTUP` and `Microsoft\Excel\XLSTART` under Roaming AppData.
- Direct files in recognized Office `<version>\STARTUP` / `XLSTART` and `root\<version>\STARTUP` / `XLSTART` folders under Program Files and Program Files (x86).

**How to investigate:** A template can be legitimate or contain no executable content. Juntei does not extract or execute macros in this check. Inspect the document internals, macro policy, and application activity; a machine installation path does not imply a SYSTEM process token.

### Office Test DLL {#office_test_dll}

**Check ID:** `office_test_dll` · **Catalogue ATT&CK:** [T1137.002](https://attack.mitre.org/techniques/T1137/002/)

**What Juntei checks:** Enumerates values in the Office Test Special Perf key from the listed machine/user and 32-bit machine views.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Office Test\Special\Perf` | All nonempty values |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Office Test\Special\Perf` | All nonempty values |
| `HKU\<user>\SOFTWARE\Microsoft\Office Test\Special\Perf` | All nonempty values |

**How to investigate:** Treat unexpected overrides as a lead requiring Office-version and payload validation. The check reports configuration rather than testing whether a particular Office installation loads the value.

### Outlook Home Page Attack {#outlook_home_page}

**Check ID:** `outlook_home_page` · **Catalogue ATT&CK:** [T1137.004](https://attack.mitre.org/techniques/T1137/004/)

**What Juntei checks:** Reads `URL` beneath user Outlook WebView children for Office versions 14.0, 15.0, and 16.0.

**Evidence examined:**

- `HKU\<user>\Software\Microsoft\Office\<14.0|15.0|16.0>\Outlook\WebView\<folder>\URL` in NTUSER.DAT.

**How to investigate:** A folder home page may be legitimate and behavior varies by Outlook version and security updates. Inspect the URL/content and effective policy, then correlate with Outlook/web/process evidence.

### VBA Monitor DLL Hijack {#vba_monitors}

**Check ID:** `vba_monitors` · **Catalogue ATT&CK:** [T1137](https://attack.mitre.org/techniques/T1137/)

**What Juntei checks:** Reads InprocServer32 values for the two implemented VBA-monitor-related CLSIDs in machine and per-user class stores.

**Evidence examined:**

- `Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32` and `{0002E157-0000-0000-C000-000000000046}` in machine SOFTWARE, plus equivalent user UsrClass.dat registrations.

**How to investigate:** Compare the selected libraries with the installed Office/VBA environment. This is a targeted CLSID check, not an inventory of every VBA extension or proof that a macro ran.

## Debuggers, crash handlers, and compatibility shims

A debugger or error handler can run when an application starts, crashes, hangs, or exits. Application compatibility databases can also affect startup. Investigate the prerequisite settings and triggering program alongside the reported command.

### AeDebug Debugger Hijack {#ae_debug}

**Check ID:** `ae_debug` · **Catalogue ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**What Juntei checks:** Reads native and 32-bit AeDebug `Debugger` commands used by just-in-time crash debugging configuration.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |

**How to investigate:** Development/debugging software may set this legitimately. Examine the full debugger command and crash-trigger prerequisites, then correlate with application-error and process events.

### AeDebug Protected Process Debugger {#ae_debug_protected}

**Check ID:** `ae_debug_protected` · **Catalogue ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**What Juntei checks:** Reads native and 32-bit AeDebugProtected `Debugger` commands for the protected-process debugging surface.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |

**How to investigate:** Determine whether the operating-system build and protection policy can use the configured command. A saved debugger is not proof that a protected process crashed or that the command ran.

### Image File Execution Options Debugger {#ifeo_debugger}

**Check ID:** `ifeo_debugger` · **Catalogue ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**What Juntei checks:** Searches native/32-bit Image File Execution Options registrations, including nested filter keys, for `Debugger` values.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\Debugger`, including nested filter keys and the machine `Wow6432Node` view.

**How to investigate:** A debugger can be selected when the named image is launched. Inspect the image/filter scope and debugger arguments; developer tooling can also use IFEO. Trace the target launch to establish execution.

### IFEO Delegated NTDLL {#ifeo_delegated_ntdll}

**Check ID:** `ifeo_delegated_ntdll` · **Catalogue ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**What Juntei checks:** Searches native/32-bit Image File Execution Options registrations, including nested keys, for `VerifierDlls` overrides.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\VerifierDlls`, including nested keys and the machine `Wow6432Node` view.

**How to investigate:** The check's predicate is the configured VerifierDlls value; it does not prove all verifier/delegation prerequisites are enabled. Review applicable flags, Windows version, and the payload before inferring a working injection path.

### Silent Process Exit Monitor {#ifeo_silent_process_exit}

**Check ID:** `ifeo_silent_process_exit` · **Catalogue ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**What Juntei checks:** Searches native/32-bit SilentProcessExit registrations for `MonitorProcess`, including nested keys.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\<image>\...\MonitorProcess`, including nested keys and the machine `Wow6432Node` view.

**How to investigate:** Silent-exit monitoring also requires the relevant reporting/IFEO configuration. Inspect those settings and the monitored image's exit events; legitimate diagnostic tooling may explain the monitor.

### Application Shimming {#installed_sdb}

**Check ID:** `installed_sdb` · **Catalogue ATT&CK:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**What Juntei checks:** Reads `DatabasePath` from immediate InstalledSDB registrations for installed application compatibility databases.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\<child>` | `DatabasePath` |

**How to investigate:** Compatibility fixes can be legitimate. Inspect the SDB contents, targeted executables, and installation history. A database registration does not identify its shims or prove that a matching application ran.

### LSM Debugger Hijack {#lsm_debugger}

**Check ID:** `lsm_debugger` · **Catalogue ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**What Juntei checks:** Reads `MonitorProcess` specifically under SilentProcessExit for `lsm.exe` in native and 32-bit machine views.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |

**How to investigate:** This overlaps the broader SilentProcessExit inventory. Confirm prerequisite flags and a relevant LSM exit before treating the configuration as executed persistence; count the same entry once when assessing distinct findings.

### MiniDump Auxiliary DLLs {#minidump_auxiliary_dlls}

**Check ID:** `minidump_auxiliary_dlls` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Enumerates named, nonempty values in MiniDumpAuxiliaryDlls, preserving the value name as the trigger-module context and the value data as the selected target.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls`: trigger-module value names and target data.

**How to investigate:** Inspect both the trigger module and auxiliary DLL. Crash/minidump tooling may explain an entry, and a registration does not show that a dump operation loaded the library.

### Application Shimming {#shim_custom}

**Check ID:** `shim_custom` · **Catalogue ATT&CK:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**What Juntei checks:** Reads `.sdb`-named values beneath immediate AppCompatFlags Custom application keys, preserving the application-to-database association.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<application>`: value names ending in `.sdb`.

**How to investigate:** Pair these associations with InstalledSDB and the actual compatibility database. A filename association alone does not reveal the database's behavior or demonstrate a matching application launch.

### WER Debugger Hijack {#wer_debugger}

**Check ID:** `wer_debugger` · **Catalogue ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**What Juntei checks:** Reads native/32-bit Windows Error Reporting `Debugger` values.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `Debugger` |

**How to investigate:** Legitimate diagnostic software may configure a debugger. Investigate the complete command and relevant crash events; registration is not proof that Windows Error Reporting started it.

### WER Hangs Debugger Hijack {#wer_hangs}

**Check ID:** `wer_hangs` · **Catalogue ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**What Juntei checks:** Reads native/32-bit Windows Error Reporting Hangs `Debugger` values.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |

**How to investigate:** Check whether a hang-handling workflow would invoke this setting and inspect the target program. Correlate with application hang and process records rather than treating a registry write as a debugger execution.

### WER ReflectDebugger Hijack {#wer_reflect_debugger}

**Check ID:** `wer_reflect_debugger` · **Catalogue ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**What Juntei checks:** Reads native/32-bit Windows Error Reporting `ReflectDebugger` values.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |

**How to investigate:** Compare with approved diagnostic configuration and establish Windows-version/trigger applicability. Inspect command arguments and seek crash/process corroboration before attributing execution.

### WER Runtime Exception Modules {#wer_runtime_exception}

**Check ID:** `wer_runtime_exception` · **Catalogue ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**What Juntei checks:** Enumerates value names under native/32-bit RuntimeExceptionHelperModules; here the library path is carried in the value name, not necessarily its data.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules` and the machine `Wow6432Node` view: value names.

**How to investigate:** Inspect the named module and application/runtime context. A small numeric value in the data is not the payload path, and registration alone does not prove the helper was loaded.

## .NET runtime and profiler settings

Managed runtimes can load profilers, startup assemblies, or debugging libraries before application code. These settings are also used by developers and monitoring products. Check runtime, process architecture, enable flags, and whether a newly started process could inherit the setting.

### AppDomainManager Injection {#appdomain_manager}

**Check ID:** `appdomain_manager` · **Catalogue ATT&CK:** [T1574.014](https://attack.mitre.org/techniques/T1574/014/)

**What Juntei checks:** Reads machine/user `APPDOMAIN_MANAGER_ASM` and `APPDOMAIN_MANAGER_TYPE` environment variables, covering assembly and type selection for a custom .NET AppDomain manager.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_TYPE` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_TYPE` |

**How to investigate:** Applications can intentionally use custom managers. Resolve the assembly and type together and check runtime compatibility and process inheritance; neither a single variable nor its key timestamp proves a managed application loaded it.

### .NET CLR Profiler Hijack {#cor_profiler}

**Check ID:** `cor_profiler` · **Catalogue ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**What Juntei checks:** Reads machine/user COR profiling environment settings and matching named entries in individual services' Environment multi-strings. It retains enable flags, profiler identifiers, and architecture-specific paths.

**Evidence examined:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` and each captured user NTUSER.DAT `Environment`.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: named COR settings in the multi-string.

**How to investigate:** Profilers support legitimate monitoring. An enable flag, CLSID, and DLL path are different pieces of configuration; confirm effective enablement and the relevant .NET process rather than treating each row as an executed profiler.

### .NET Core CLR Profiler Hijack {#coreclr_profiler}

**Check ID:** `coreclr_profiler` · **Catalogue ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**What Juntei checks:** Reads corresponding machine/user CORECLR profiling environment settings and matching service Environment entries, including architecture-specific paths.

**Evidence examined:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` and each captured user NTUSER.DAT `Environment`.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: named CORECLR settings in the multi-string.

**How to investigate:** Validate the .NET runtime, architecture, enable flag, and library. Approved observability products may configure these settings, and an existing process may not inherit a newly changed environment.

### .NET DbgManagedDebugger Hijack {#dotnet_dbg_managed_debugger}

**Check ID:** `dotnet_dbg_managed_debugger` · **Catalogue ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**What Juntei checks:** Reads the listed machine/user `.NETFramework` `DbgManagedDebugger` settings, including the explicit machine 32-bit path.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `DbgManagedDebugger` |

**How to investigate:** Determine whether the applicable .NET runtime/debugging policy uses this command. Developer tooling may explain it; correlate with managed crashes and process creation to establish execution.

### .NET Framework COR_PROFILER Registry Key {#dotnet_framework_profiler}

**Check ID:** `dotnet_framework_profiler` · **Catalogue ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**What Juntei checks:** Reads `COR_PROFILER`, `COR_PROFILER_PATH`, and `COR_ENABLE_PROFILING` from the listed `.NETFramework` registry views. This complements the environment-based profiler checks.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |

**How to investigate:** Evaluate the values together and determine whether the runtime honors that registry location. A configured profiling tool may be legitimate; do not interpret a standalone enable flag as an identified DLL load.

### DOTNET_STARTUP_HOOKS {#dotnet_startup_hooks}

**Check ID:** `dotnet_startup_hooks` · **Catalogue ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**What Juntei checks:** Reads machine/user `DOTNET_STARTUP_HOOKS` environment values, which can select startup-hook assemblies for supported .NET runtimes.

**Evidence examined:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment\DOTNET_STARTUP_HOOKS` and each user NTUSER.DAT `Environment\DOTNET_STARTUP_HOOKS`.

**How to investigate:** Inspect the entire value for multiple assemblies and verify runtime support. This check does not enumerate service-specific startup-hook environments or prove that any process inherited and used the value.

### Known Managed Debugging DLLs {#known_managed_debugging_dlls}

**Check ID:** `known_managed_debugging_dlls` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads `.NETFramework` `KnownManagedDebuggingDlls` registrations as possible managed-debugger library overrides.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `KnownManagedDebuggingDlls` |

**How to investigate:** Compare with installed development/runtime tooling and determine the triggering debugger. The registry entry does not prove a debugger session or library load.

## DLL and executable lookup overrides

These checks examine explicit library locations or settings that influence how a component finds code. A path override or weakened search policy is an investigative lead; a payload and a compatible triggering component are still needed for execution.

### AppCert DLLs {#appcert_dlls}

**Check ID:** `appcert_dlls` · **Catalogue ATT&CK:** [T1546.009](https://attack.mitre.org/techniques/T1546/009/)

**What Juntei checks:** Enumerates nonempty Session Manager AppCertDlls registrations, a library extension surface associated with process-creation APIs.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\AppCertDlls` | All nonempty values |

**How to investigate:** Inspect each library and its installation provenance. Determine applicability and runtime loading separately; normal process creation in a timeline is not by itself proof that the configured DLL loaded.

### AppInit DLLs {#appinit_dlls}

**Check ID:** `appinit_dlls` · **Catalogue ATT&CK:** [T1546.010](https://attack.mitre.org/techniques/T1546/010/)

**What Juntei checks:** Reads native/32-bit `AppInit_DLLs`, splits its library list, and adds available `LoadAppInit_DLLs` and `RequireSignedAppInit_DLLs` context to each candidate.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows`: `AppInit_DLLs`, `LoadAppInit_DLLs`, and `RequireSignedAppInit_DLLs`; also the machine `Wow6432Node` view.

**How to investigate:** A zero LoadAppInit_DLLs flag is explicitly marked inactive. Windows version, Secure Boot, and signing requirements also affect loading. Review the effective settings before claiming an AppInit DLL executed.

**Further details:** [Microsoft documentation](https://learn.microsoft.com/en-us/windows/win32/dlls/secure-boot-and-appinit-dlls).

### AutodialDLL Override {#autodial_dll}

**Check ID:** `autodial_dll` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads the WinSock2 Parameters `AutodialDLL` override.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters` | `AutodialDLL` |

**How to investigate:** Check the networking feature and source Windows version that consume this setting. Review the actual DLL and network/process activity; an override alone does not establish an active or malicious load.

### CHM Helper DLL {#chm_helper_dll}

**Check ID:** `chm_helper_dll` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads `Location` under HtmlHelp Author, a help-related library/location override surface.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\HtmlHelp Author` | `Location` |

**How to investigate:** Determine whether the relevant help component uses the value on this system and inspect the selected file. A registry override is not proof that a CHM document was opened.

### NL6 ContentIndex DLL Override {#content_index_dll}

**Check ID:** `content_index_dll` · **Catalogue ATT&CK:** [T1574](https://attack.mitre.org/techniques/T1574/)

**What Juntei checks:** Reads `DLLOverridePath` from immediate active-control-set ContentIndex Language registrations.

**Evidence examined:**

- `HKLM\SYSTEM\<ControlSet>\Control\ContentIndex\Language\<language>\DLLOverridePath`.

**How to investigate:** Language/search components may differ by Windows build. Inspect the library and matching installed component; follow the actual indexing process to corroborate loading.

### Crypto ExpoOffload DLL {#crypto_expo_offload}

**Check ID:** `crypto_expo_offload` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads the Cryptography Offload `ExpoOffload` value as a cryptographic library override candidate.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Cryptography\Offload` | `ExpoOffload` |

**How to investigate:** Confirm feature/build applicability and investigate the selected library's origin. Do not treat the presence of an uncommon value as proof of credential theft or executed code.

### Direct3D Software Rasterizer DLL {#direct3d_dll}

**Check ID:** `direct3d_dll` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads the Direct3D Drivers `SoftwareRasterizer` library override.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Direct3D\Drivers` | `SoftwareRasterizer` |

**How to investigate:** Graphics and compatibility software can influence rasterizer configuration. Inspect the DLL and a compatible Direct3D application's activity; saving the path does not prove it loaded.

### DLL Search Order Weakening {#dll_search_mode}

**Check ID:** `dll_search_mode` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reports `SafeDllSearchMode` or `CWDIllegalInDllSearch` only when the active Session Manager value parses to zero. These are search-policy conditions, not selected payloads.

**Evidence examined:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager`: `SafeDllSearchMode` and `CWDIllegalInDllSearch`.

**How to investigate:** A weakened lookup policy may make a separate search-order attack easier. It does not identify a planted DLL or prove execution. Nonzero or absent values are not emitted by this check; evaluate effective defaults and policy separately.

**Further details:** [Microsoft documentation](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security).

### KnownDLLs Exclusion {#exclude_from_known_dlls}

**Check ID:** `exclude_from_known_dlls` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads nonempty `ExcludeFromKnownDlls` entries from Session Manager as changes to protected/known DLL lookup behavior.

**Evidence examined:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\ExcludeFromKnownDlls`.

**How to investigate:** An exclusion identifies a lookup condition, not the replacement library itself. Find the affected application and potential DLL search locations, then corroborate a nonstandard image load.

**Further details:** [Microsoft documentation](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order).

### hhctrl.ocx DLL Override {#hhctrl_ocx_dll}

**Check ID:** `hhctrl_ocx_dll` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads native/32-bit machine and supported user class-store InprocServer32 overrides for the implemented HTML Help control CLSID.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKLM\SOFTWARE\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\Software\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |

**How to investigate:** Determine which application and architecture activate the control. Inspect the registered DLL and help-related activity; a COM registration does not prove that HTML Help rendered content.

### Known DLLs {#known_dlls}

**Check ID:** `known_dlls` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Enumerates named values in the active Session Manager KnownDLLs key.

**Evidence examined:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\KnownDLLs`: named values.

**How to investigate:** Many entries are expected Windows libraries, and some values are supporting configuration rather than a payload path. Compare the full key with a trusted matching build; this check does not scan every alternate architecture-specific KnownDLLs tree.

### LSA Extensions DLL {#lsa_extensions}

**Check ID:** `lsa_extensions` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads `Extensions` from the LsaExtensionConfig LsaSrv configuration.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\LsaExtensionConfig\LsaSrv` | `Extensions` |

**How to investigate:** Security software can extend authentication components. Inspect the actual libraries and compatibility with the source Windows version, then seek LSASS image-load or other independent use evidence.

### MAPI32 DLL Path Override {#mapi32_dll_path}

**Check ID:** `mapi32_dll_path` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads `DLLPathEx` and `DLLPath` beneath immediate Mail client registrations in native and 32-bit machine views.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPath` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPath` |

**How to investigate:** Mail clients register legitimate MAPI providers. Determine which client/provider was selected and inspect the actual library; an installed provider does not prove an application used MAPI during the incident.

### MSDTC XA DLL {#msdtc_xa_dll}

**Check ID:** `msdtc_xa_dll` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads `OracleXaLib` and `OracleOciLib` from MSDTC MTxOCI configuration.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleXaLib` |
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleOciLib` |

**How to investigate:** Database integration can legitimately require these libraries. Verify the transaction-coordinator configuration and installed database software, and correlate with service/module use rather than assuming a configured path ran.

### NLDP DLL Override {#nldp_dll}

**Check ID:** `nldp_dll` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads the NlsData `DllOverridePath` configuration value.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NlsData` | `DllOverridePath` |

**How to investigate:** Review the target library, Windows build, and natural-language component that could consume the override. The value does not show that indexing or language processing occurred.

### RDP TestDVCPlugin DLL {#rdp_test_dvc_plugin}

**Check ID:** `rdp_test_dvc_plugin` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads the machine Terminal Server Client `TestDVCPlugin` library setting.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Terminal Server Client` | `TestDVCPlugin` |

**How to investigate:** Investigate an unfamiliar test plugin and version-specific behavior. A compatible client session and library load are needed to establish execution; the registry value alone does not show either.

### Search Indexer DLL Override {#search_indexer_dll}

**Check ID:** `search_indexer_dll` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads the Windows Search `DllPath` override from SOFTWARE.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Search` | `DllPath` |

**How to investigate:** Determine the matching Search component/version and inspect the target file. An inactive Windows Search service or unconsumed setting is different from an observed malicious library load.

### Winsock AutoProxy DLL {#winsock_auto_proxy}

**Check ID:** `winsock_auto_proxy` · **Catalogue ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**What Juntei checks:** Reads `LibraryPath` beneath immediate NameSpace_Catalog5 Catalog_Entries and Catalog_Entries64 registrations.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries\<child>` | `LibraryPath` |
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries64\<child>` | `LibraryPath` |

**How to investigate:** Networking providers can be legitimate. Review both architectures, provider provenance, and relevant network application/image-load evidence. This targeted namespace-provider lookup is not every possible Winsock extension.

## Accounts and sign-in visibility

Local account records and sign-in visibility settings can reveal identity-based persistence. These checks inspect the captured SAM and SOFTWARE hives; they are not an inventory of Active Directory accounts, domain group membership, or past logons.

### Administrators Group Membership {#admin_group_membership}

**Check ID:** `admin_group_membership` · **Catalogue ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**What Juntei checks:** Decodes the local Administrators alias member list from SAM. It reports local-machine SIDs that name a hidden account or no account/group in the captured SAM.

**Evidence examined:**

- `SAM\Domains\Builtin\Aliases\00000220\C` and machine SID/user/group records beneath `SAM\Domains\Account` in the SAM hive.
- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList` for hidden-name correlation.

**How to investigate:** This is a targeted anomaly check, not a list of all administrators. External/domain SIDs are not evaluated as missing local users. Deleted accounts, capture timing, or incomplete hives require investigation before concluding malicious membership.

### Hidden Local Account {#hidden_account}

**Check ID:** `hidden_account` · **Catalogue ATT&CK:** [T1564.002](https://attack.mitre.org/techniques/T1564/002/)

**What Juntei checks:** Reports numeric zero values in Winlogon SpecialAccounts UserList, identifying names hidden from the ordinary sign-in screen.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`: account-name values.

**How to investigate:** Service or maintenance accounts can be hidden intentionally, and the value does not establish that the named account exists or is an administrator. Join with SAM account records and logon activity.

### RID Hijacking {#rid_hijacking}

**Check ID:** `rid_hijacking` · **Catalogue ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**What Juntei checks:** Compares a SAM account subkey's hexadecimal RID with the RID stored at offset 0x30 in its F value and reports mismatches.

**Evidence examined:**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` in the SAM hive.

**How to investigate:** A mismatch is a serious account-record anomaly but can also require corruption/capture analysis. Inspect source bytes, transaction-log status, and account/logon records; it does not prove which actor modified the record.

### RID Suborner (Hidden Admin Account) {#rid_suborner}

**Check ID:** `rid_suborner` · **Catalogue ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**What Juntei checks:** Reports SAM accounts whose subkey RID is not 500 but whose F-value RID is 500, a more specific administrator-identity substitution pattern.

**Evidence examined:**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` in the SAM hive.

**How to investigate:** This overlaps the general RID mismatch check. It does not independently require a hidden sign-in setting despite the catalogue title; confirm account visibility, privileges, and actual account use separately.

## Authentication components and credential protection

The Local Security Authority (LSA) handles security functions through LSASS and related components. Authentication packages, password filters, and network providers can be legitimate security software or a route to persistent credential access. Two checks inventory protection settings rather than executable autostarts. Interpret them using host role, Windows version, and effective policy.

### AMSI Provider DLL {#amsi_providers}

**Check ID:** `amsi_providers` · **Catalogue ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**What Juntei checks:** Enumerates machine AMSI provider CLSIDs and resolves their native/32-bit InprocServer32 libraries when available. Unresolved registrations retain the class ID.

**Evidence examined:**

- `HKLM\SOFTWARE\Microsoft\AMSI\Providers\<CLSID>` and resolvable `Classes\CLSID` / `Classes\Wow6432Node\CLSID` InprocServer32 registrations.

**How to investigate:** Antivirus and security products legitimately register AMSI providers. Review the provider/library provenance and actual loading; a registration does not prove AMSI was bypassed or that the provider was malicious.

### Authentication Packages {#authentication_packages}

**Check ID:** `authentication_packages` · **Catalogue ATT&CK:** [T1547.002](https://attack.mitre.org/techniques/T1547/002/)

**What Juntei checks:** Reads individual nonempty LSA `Authentication Packages` entries from the active control set.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Authentication Packages` |

**How to investigate:** Compare with the source Windows build and installed authentication software. A nondefault package can be legitimate. Resolve the DLL and corroborate loading/credential activity before concluding interception.

### DSRM Admin Logon Behavior {#dsrm_backdoor}

**Check ID:** `dsrm_backdoor` · **Catalogue ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**What Juntei checks:** Reports numeric `DsrmAdminLogonBehavior=2` in the implemented ControlSet001, ControlSet002, or CurrentControlSet locations. This setting can permit DSRM-account use outside its usual recovery context.

**Evidence examined:**

- `HKLM\SYSTEM\ControlSet001\Control\Lsa\DsrmAdminLogonBehavior`, plus ControlSet002 and CurrentControlSet when present.

**How to investigate:** Relevant primarily to domain controllers. Determine which control set is active using SYSTEM Select; a historical control-set match is not necessarily effective. The value does not prove an attacker knew the DSRM password or logged on.

### Credential Guard Configuration {#lsa_cfg_flags}

**Check ID:** `lsa_cfg_flags` · **Catalogue ATT&CK:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**What Juntei checks:** Reads the configured LSA `LsaCfgFlags` value, including ordinary enabled values as well as zero. This inventories Credential Guard configuration rather than an executable autostart.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `LsaCfgFlags` |

**How to investigate:** Interpret the value with Windows version, firmware/security prerequisites, and effective policy. A stored zero or absent value alone does not establish a change by an attacker, effective protection state, or credential theft.

### LSA Password Filter {#lsa_password_filter}

**Check ID:** `lsa_password_filter` · **Catalogue ATT&CK:** [T1556.002](https://attack.mitre.org/techniques/T1556/002/)

**What Juntei checks:** Reads LSA `Notification Packages`. Bare library names are resolved against the captured Windows System32 directory, adding `.dll` where appropriate.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Notification Packages` |

**How to investigate:** Password policy products commonly install filters. Inspect the library and loading evidence, and distinguish configured filters from proof that a password change or credential interception occurred.

### LSASS PPL Protection Status {#lsa_run_as_ppl}

**Check ID:** `lsa_run_as_ppl` · **Catalogue ATT&CK:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**What Juntei checks:** Reads the configured LSA `RunAsPPL` value, including nonzero values. It is a protection-setting inventory, not a test that LSASS ran unprotected.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `RunAsPPL` |

**How to investigate:** Effective LSA protection depends on Windows version, policy, firmware locking, and restart state. No row for an absent value is not a measured protection result. Use independent system/runtime evidence to assess effective protection.

### Network Provider DLL {#network_provider_dll}

**Check ID:** `network_provider_dll` · **Catalogue ATT&CK:** [T1556.008](https://attack.mitre.org/techniques/T1556/008/)

**What Juntei checks:** Reads `NetworkProvider\ProviderPath` beneath services in the active SYSTEM control set.

**Evidence examined:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\NetworkProvider\ProviderPath`.

**How to investigate:** Network logon providers can be legitimate. Check provider order/applicability, installed network/security software, and actual library loading; a ProviderPath value alone does not prove credential interception.

### Security Packages {#security_packages}

**Check ID:** `security_packages` · **Catalogue ATT&CK:** [T1547.005](https://attack.mitre.org/techniques/T1547/005/)

**What Juntei checks:** Reads individual nonempty LSA `Security Packages` entries, covering configured security support providers.

**Evidence examined:**

| Registry key | Value examined |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Security Packages` |

**How to investigate:** Compare with the expected Windows/security-product baseline and inspect each library. The configuration does not prove a provider loaded, captured credentials, or was added during the incident.

## Work from a candidate to a conclusion

1. Open the original source identified by `SourceFile`. Confirm the complete value, XML action,
   script, or recovered database record and the exact user/host it belongs to.
2. Determine whether the mechanism could operate: check enable flags, runtime or Windows version,
   service/task state, required trigger, user, architecture, and the active control set.
3. Inspect the selected program, DLL, script, or assembly. Hash collected bytes and review arguments
   and content. A recognizable Windows launcher or publisher string does not settle legitimacy.
4. Establish what actually happened using independent evidence. Examples include process creation,
   Sysmon image loads, Task Scheduler runtime records, service activity, Prefetch, and related files.
   A configuration-change event can prove the setting changed without proving its payload ran.
5. Build a timeline using original-source timestamps and the recorded correlation provenance.
   Keep key writes, installation changes, file/container writes, and executions as separate facts.
6. Record limits as well as findings: missing hives, missing target files, unknown task principals,
   unverified signer metadata, historical carved records, or a trigger that cannot be corroborated.

For command syntax, profiles, every output field, and coverage states, return to the
[Check Persistence command guide](../commands/check-persistence.md). The
[artifact guides](../artifacts/index.md) explain the underlying sources, and the
[acknowledgements](acknowledgements.md) identify the upstream persistence tools.
