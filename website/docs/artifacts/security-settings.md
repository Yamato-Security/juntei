# Security Settings

## What they are

Attackers change Windows' security settings to stay hidden and to reach credentials. They turn
off Microsoft Defender or exclude their tools from it, make Windows keep cleartext passwords,
open Remote Desktop, turn off the firewall, and silence PowerShell logging. The registry keeps
these settings:

- **`SOFTWARE`:** Defender, its exclusions and policies; UAC; Remote Desktop's and the firewall's
  policies; PowerShell's execution policy, logging policies and AMSI providers; System Restore;
  the event log channels.
- **`SYSTEM`** (per control set): `WDigest`, LSA protection, Credential Guard, Restricted Admin,
  NTLM; Remote Desktop; the firewall profiles; SMBv1; last access times; each event log's size.
- **`NTUSER.DAT`:** the user's PowerShell execution policy and logging policies, and AMSI for
  Windows Script Host.

`parse-registry security-settings` reports each setting it finds as a record. Each record has the
setting's value, what the value means, and whether it **weakens the default**: whether it turns a
protection or an evidence source off, or lowers it below what current Windows (10 and 11) does
without the value.

<span class="evidence-strength">Best conclusion:</span> this setting had this value when the hive
was saved, and the key was last written at this time. A weakened setting is a lead, not proof of
compromise. Administrators, Group Policy, other security products and older Windows versions set
some of these values too, and the meaning says so where it is common.

## Location

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Defender[\Real-Time Protection|\Features|\Spynet]
  Microsoft\Windows Defender\Exclusions\Paths|Extensions|Processes|IpAddresses|TemporaryPaths
  Policies\Microsoft\Windows Defender\…
  Microsoft\Windows\CurrentVersion\Policies\System                 UAC
  Policies\Microsoft\Windows NT\Terminal Services                  RDP policy
  Policies\Microsoft\WindowsFirewall\<profile>                      firewall policy
  Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell             ExecutionPolicy
  Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging|ModuleLogging|Transcription
  Microsoft\AMSI\Providers\{CLSID}
  Microsoft\Windows NT\CurrentVersion\SystemRestore                DisableSR
  Microsoft\Windows\CurrentVersion\WINEVT\Channels\<channel>       Enabled, MaxSize
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Control\SecurityProviders\WDigest                                UseLogonCredential
  Control\Lsa                                                      RunAsPPL, LsaCfgFlags, DisableRestrictedAdmin, LmCompatibilityLevel
  Control\Terminal Server[\WinStations\RDP-Tcp]                    fDenyTSConnections, PortNumber, UserAuthentication
  Services\SharedAccess\Parameters\FirewallPolicy\<profile>        EnableFirewall
  Services\LanmanServer\Parameters                                 SMB1
  Control\FileSystem                                               NtfsDisableLastAccessUpdate
  Services\EventLog\<log>                                          MaxSize
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell    ExecutionPolicy
  Software\Policies\Microsoft\Windows\PowerShell\…
  Software\Microsoft\Windows Script\Settings                       AmsiEnable
```

Collect all three hives with their transaction logs. A setting changed shortly before collection
may only be in the logs.

## Analyst cautions

- **Stored configuration is not the effective protection state.** Collected registry values and diagnostic rows for damaged value lists
  are reported. Missing data can reflect defaults, uncollected sources or another management
  mechanism; it is not proof that protection is enabled or disabled. `WeakensDefault` flags
  settings that can reduce the stated baseline when they are honored.
- **Legacy Defender flags may be ignored.** `DisableAntiSpyware` and `DisableAntivirus` do not
  prove Defender stopped. Applicability depends on platform version, onboarding and tamper
  protection. [Microsoft explains these legacy settings](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware).
- **Modern managed exclusions can be unavailable in registry captures.** Starting with platform
  4.18.25110.6, devices using Defender for Endpoint configuration management require supported
  Defender PowerShell cmdlets to retrieve exclusion configuration. Offline registry absence
  therefore does not establish an absence of exclusions.
  [Microsoft configuration guidance](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings).
- **Weakened isn't malicious.** Examples:
  - `DisableAntiSpyware` = 1 is also set by Windows while another antivirus is registered.
  - Remote Desktop is opened deliberately on servers.
  - Windows 7 kept smaller event logs and didn't update last access times.
- **Defaults depend on the version.** Some values were the default before:
  - `NtfsDisableLastAccessUpdate` = 1 was the default before Windows 10 1803. It is reported
    without a verdict, but `0x80000001`, set by hand on a later version, is weakened.
  - `RunAsPPL`, `LsaCfgFlags` and `UseLogonCredential` are compared with current Windows.
- **Defender exclusions:** each path, extension, process or address is a record. Every exclusion
  weakens Defender. `TemporaryPaths`, which Defender adds itself, has no verdict. Look for user-writable folders,
  `.ps1`/`.exe` extensions and LOLBins (`rundll32.exe`, `powershell.exe`).
- **Policy keys** (`Policy` = True) are written by Group Policy, by tools, and by attackers who
  know policies win over the local settings. Correlate them with the domain's GPOs.
- **Event log channels:** only 25 security-relevant channels are reported, with whether Windows
  turns each on by default. `TaskScheduler/Operational` and `DNS-Client/Operational`, for
  example, are off by default. Use `parse-registry read` for the others.
- **Key times:** `KeyLastWrittenUTC` is the key's, so it is not necessarily when this value
  changed. A whole `Policies` key written at once is common.
- **Control sets:** by default the control set Windows used last is read. `--all-control-sets`
  reads every one.

## Parse with Juntei

```text
Usage: juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use
      --weakened-only     Only settings that weaken the default

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry security-settings -d evidence/ -o settings.csv
juntei parse-registry security-settings --weakened-only -d evidence/ -t jsonl \
  | jq -r '[.Host, .Area, .KeyPath, .ValueName, .Value, .Meaning] | @tsv'
```

The summary counts the settings by area. It lists the settings that weaken the default and every
Defender exclusion.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, the hive, and its type (`SOFTWARE`, `SYSTEM`, `NTUSER.DAT`). |
| `ControlSet`, `IsCurrentControlSet` | For `SYSTEM`: the control set, and whether it is the one Windows used last; unknown when `Select\Current` is missing or invalid. |
| `Username`, `UserSID` | For `NTUSER.DAT`: the profile folder's name, and the user's SID from the system's `ProfileList`. |
| `Area` | `defender`, `uac`, `credentials`, `remote_access`, `network`, `powershell`, `evidence`. |
| `Setting` | What it is: `defender_disable_realtime_monitoring`, `defender_exclusion_path`, `uac_local_account_token_filter_policy`, `wdigest_use_logon_credential`, `rdp_deny_connections`, `powershell_script_block_logging`, `event_channel_enabled`, … |
| `Value` | The value: a number in decimal, a string, or the exclusion itself. Empty when the value isn't of the setting's type. |
| `Meaning` | What the value means. |
| `WeakensDefault` | `True` when the value weakens what current Windows does without it, `False` when it doesn't, empty when that doesn't apply (a size, a port, an AMSI provider) or can't be told. |
| `Policy` | Whether the key is under a `Policies` key. |
| `KeyPath`, `ValueName` | The key and value. An exclusion's value name is the excluded item; an AMSI provider has none. |
| `KeyLastWrittenUTC` | The key's last write. |
| `DataRaw` | The value's bytes (hex), when they aren't of the setting's type. |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context. |
| `Problems` | Values of the wrong type and keys that can't be read. |

## Investigation strategy

1. Run with `--weakened-only` on every host, and compare the hosts with each other: a setting
   only one host has stands out.
2. Check each Defender exclusion against the tools found on the host and in Defender's logs
   (Event 5007 records configuration changes).
3. Correlate key times with logons, service installs and process creation. Look for
   `reg.exe`, `Set-MpPreference`, `netsh advfirewall` and `powershell` command lines (event
   4688, Sysmon 1 and 13, PowerShell 4104).
4. For credential settings (`WDigest`, `RunAsPPL`, `LocalAccountTokenFilterPolicy`,
   `DisableRestrictedAdmin`), look for credential dumping and pass-the-hash afterwards.
5. Use `parse-registry read --recover-deleted` for values deleted to restore a default, and
   `--all-control-sets` for older control sets.

## Further Details

- [Microsoft: Configure and validate exclusions for Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus)
- [Microsoft: User Account Control settings and configuration](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft: Configure added LSA protection](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft: about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001: Impair Defenses: Disable or Modify Tools](https://attack.mitre.org/techniques/T1562/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
