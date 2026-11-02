# System Information

## What it is

A system's four machine hives say what an analyst needs before reading its timeline:

- its names (NetBIOS, DNS host name, DNS domain) and the domain or workgroup it is in;
- its Windows version, build and update revision, and when Windows was installed or last upgraded;
- its time zone, which tells you the local times the system kept;
- when it last shut down cleanly;
- its processor architecture and code pages;
- the user who last signed in, and the profiles of every user who has signed in;
- its SIDs: the computer's (local accounts are it plus a RID) and its domain's;
- the Windows versions it was upgraded from.

`parse-registry system-info` writes one record per system with all of these.

<span class="evidence-strength">Best conclusion:</span> the configuration as it stood when the hives
were collected (or last written). The key times say when parts of it last changed.

## Location

```text
C:\Windows\System32\config\SYSTEM     (the control set Windows used last: Select\Current)
  ControlSet00N\Control\ComputerName\ComputerName     ComputerName
  ControlSet00N\Services\Tcpip\Parameters             Hostname, Domain
  ControlSet00N\Control\TimeZoneInformation           TimeZoneKeyName, Bias, ActiveTimeBias, …
  ControlSet00N\Control\Windows                       ShutdownTime
  ControlSet00N\Control\Session Manager\Environment   PROCESSOR_ARCHITECTURE
  ControlSet00N\Control\Nls\CodePage                  ACP, OEMCP
  Setup\Source OS (Updated on M/D/YYYY HH:MM:SS)      the version before each upgrade

C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion                 ProductName, EditionID, DisplayVersion,
                                                      CurrentBuild, UBR, InstallDate, InstallTime, …
  Microsoft\Windows NT\CurrentVersion\ProfileList     one key per profile (SID)
  Microsoft\Windows\CurrentVersion\Authentication\LogonUI   LastLoggedOnUser, …

C:\Windows\System32\config\SAM
  SAM\Domains\Account                                 V: the computer's SID at its end

C:\Windows\System32\config\SECURITY
  Policy\PolAcDmN, PolAcDmS                           the computer's name and SID
  Policy\PolPrDmN, PolPrDmS                           its domain (or workgroup) and the domain's SID
  Policy\PolDnDDN, PolDnTrN, PolDnDmG                 the domain's and forest's DNS names, GUID
  Policy\PolMachineAccountS                           the computer account's SID in the domain
```

Collect the four hives with their transaction logs. Any of them alone gives what it holds.

## Analyst cautions

- **Registry times are UTC; the time zone is for everything else.** Key times, FILETIMEs and most
  event times are UTC. Text logs, some application logs, and dates written into names or strings
  (such as `Source OS (Updated on …)`) may be local. `Bias` is the minutes from local standard
  time to UTC (`UTC = local + Bias`). `UtcOffset` and `DaylightUtcOffset` give the two offsets,
  and `TimeZoneDescription` says when daylight time begins and ends.
- **The time zone can change.** `TimeZoneLastWrittenUTC` is the key's last write. Windows also
  rewrites the key on its own when it updates `ActiveTimeBias` (the offset in force then), so a
  recent time doesn't prove someone changed the zone. Compare with event logs: Security 4616
  (the system time changed) and the System log's `Microsoft-Windows-Kernel-General` events.
- **`StandardStart` and `DaylightStart` are the kernel's `TIME_FIELDS`, not `SYSTEMTIME`s.** The
  weekday is their last field, not their third (as in the zone's `TZI` value in `SOFTWARE`). In
  US zones both layouts read the same, because the weekday is Sunday (0). Juntei decodes them
  as `TIME_FIELDS`: a public Easter Island system (first Saturday at 22:00) shows the
  difference. A month of 0 means no daylight saving time.
- **Windows 11 says "Windows 10".** `ProductName` keeps `Windows 10 …` on Windows 11. Builds 22000
  and later are Windows 11, and the summary says so. `CurrentVersion` is `6.3` on Windows 8.1, 10
  and 11; `Version` uses `CurrentMajorVersionNumber` (10) when it is there.
- **`InstallDate` moves with feature upgrades.** An in-place upgrade to a new release (1809 to
  21H2, for example) resets it. The release each upgrade replaced, with its own `InstallDate`, is
  in `PreviousVersions` (`Setup\Source OS`). Their earliest `InstallDate` is the closest to the
  original installation. The date in a `Source OS` key's name doesn't say its time zone.
- **`ShutdownTime` is the last clean shutdown.** A crash or power loss doesn't update it.
  Compare it with event logs (6006, 6008, 1074) and with the times of the last writes to the hives.
- **`LastLoggedOnUser` is the sign-in screen's last user,** not every logon. Remote Desktop,
  `runas`, services and scheduled tasks don't all set it. `LogonUiLastWrittenUTC` is about when
  it was last set.
- **Profiles:** `ProfileList` lists every user (and service) that has had a profile on the
  system, deleted accounts included until their profile is removed. Windows 10 and later write
  `LocalProfileLoadTime` and `LocalProfileUnloadTime`: about the last sign-in and sign-out.
- **SIDs:** local accounts are `MachineSid-<RID>`, domain accounts `DomainSid-<RID>`.
  `MachineSid` comes from the `SAM` (else from `SECURITY`'s `PolAcDmS`). The two should match;
  when they don't, `Problems` says so, as hives of different systems would. `DomainSid` is empty
  in a workgroup.
- **One directory, one system.** With `-d`, the hives in one directory are one system's.
  `RegBack` copies and a second hive of the same kind (`SYSTEM.old`) are records of their own.
  With `-f`, the hives given are one system's, and the others are looked for next to them.
- **Control sets:** only the control set Windows used last is read. `parse-registry read` shows
  the others.

## Parse with Juntei

```text
Usage: juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM, SOFTWARE, SAM and SECURITY hives
  -f, --file <FILE>      A hive of one system; repeat for each
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry system-info -d collection/ -o system-info.csv
juntei parse-registry system-info -f SYSTEM -f SOFTWARE -t jsonl \
  | jq '{Host, TimeZoneDescription, ProductName, Build, LastShutdown}'
```

The summary gives, for each system: its Windows version, when it was installed, its time zone in
words, the offset in force when the key was last written, its last shutdown, its domain or
workgroup, the last user who signed in, its SID, how many profiles it has, and the versions it
was upgraded from. `auto-parse` also prints each system's time zone when it lists the systems it
found.

## Output fields

| Field | Interpretation |
|---|---|
| `Host`, `SourcePath`, `SoftwarePath`, `SamPath`, `SecurityPath` | Host, and the hives read (`SourcePath` is the `SYSTEM` hive, else the first one read). |
| `ComputerName`, `Hostname`, `Domain` | The NetBIOS name (`SYSTEM`, else `SECURITY`'s `PolAcDmN`), the DNS host name and DNS domain (`Tcpip\Parameters`). |
| `PrimaryDomain`, `DNS-Domain` | The domain the computer is joined to, or its workgroup (`PolPrDmN`), and the domain's DNS name (`PolDnDDN`). |
| `ProductName`, `Edition`, `DisplayVersion`, `Version`, `Build`, `Ubr`, `ServicePack`, `InstallationType` | `CurrentVersion`: the product name as written, `EditionID`, `DisplayVersion` (else `ReleaseId`), the major and minor version (else `CurrentVersion`), the build, the update build revision, `CSDVersion`, and `Client`, `Server` or `Server Core`. |
| `InstallDateUTC`, `InstallTimeUTC` | `InstallDate` (to the second) and `InstallTime` (to the 100 ns, Windows 10 and later): installation or the last feature upgrade. |
| `RegisteredOwner`, `RegisteredOrganization`, `SystemRoot` | As written. |
| `TimeZone`, `TimeZoneDescription` | `TimeZoneKeyName` (else `StandardName`), and the zone in words with its offsets and daylight saving dates. |
| `UTC-Offset`, `DaylightUTC-Offset` | Local standard time's offset from UTC, and daylight time's (empty without daylight saving time). |
| `BiasMinutes`, `ActiveBiasMinutes`, `StandardBiasMinutes`, `DaylightBiasMinutes` | `Bias`, `ActiveTimeBias` (in force when the key was written), `StandardBias`, `DaylightBias`. |
| `StandardName`, `DaylightName` | As written, often a resource reference (`@tzres.dll,-112`). |
| `StandardStart`, `DaylightStart` | When standard and daylight time begin, in words (`first Sunday of November, 02:00`). |
| `DynamicDaylightTimeDisabled`, `RealTimeIsUniversal` | Dynamic daylight saving time turned off; the hardware clock keeps UTC. |
| `TimeZoneLastWrittenUTC` | The time zone key's last write. |
| `LastShutdownUTC` | `ShutdownTime`: the last clean shutdown. |
| `CurrentControlSet`, `ProcessorArchitecture`, `AnsiCodePage`, `OemCodePage` | The control set read, `PROCESSOR_ARCHITECTURE`, `ACP` and `OEMCP`. |
| `LastLoggedOnUser`, `LastLoggedOnSamUser`, `LastLoggedOnUserSID`, `LastLoggedOnDisplayName`, `LogonUiLastWrittenUTC` | The sign-in screen's last user, and its key's last write. |
| `MachineSID`, `DomainSID`, `MachineAccountSID` | The computer's SID, its domain's SID (`PolPrDmS`) and the computer account's SID in the domain (`PolMachineAccountS`). |
| `ComputerNameLastWrittenUTC`, `PrimaryDomainLastWrittenUTC`, `VersionLastWrittenUTC` | About when the computer was last named, last joined a domain or workgroup, and last updated (`CurrentVersion`'s key). |
| `PreviousVersions` | Each `Source OS` key: the version, its install date, and the date in the key's name (JSON: one object each). |
| `Profiles` | Each profile: SID, folder, key time, last load and unload (JSON: one object each). |
| `TransactionLogStatus`, `LogsApplied` | Registry recovery context, over all four hives. |
| `Problems` | Values of the wrong type or size (their bytes kept), and a `SAM` and `SECURITY` whose computer SIDs differ. |

## Investigation strategy

1. Read the time zone first: note `UtcOffset`, the daylight saving dates, and whether the zone was
   changed recently. Convert local-time logs with it.
2. Note the version, build and install date. Check `PreviousVersions` for upgrades, and compare
   the install date with the oldest file system and event log times.
3. Note `LastShutdownUTC`: activity after it with no new shutdown time suggests a crash, a power
   loss or a live collection.
4. Use `MachineSid` and `DomainSid` to tell local from domain accounts in SIDs elsewhere (BAM,
   `ProfileList`, event logs), and the profiles' load and unload times for the last sign-ins.
5. Correlate the computer and domain names with event logs, and with `ComputerNameLastWrittenUTC`
   and `PrimaryDomainLastWrittenUTC` for renames and domain joins.

## Further Details

- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft: DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft: TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft: Security identifiers](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb: Time zones](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb: Windows product information](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
