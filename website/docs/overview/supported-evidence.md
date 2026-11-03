# Supported Evidence

| Evidence source | Juntei command | Most useful for | Important limitation |
|---|---|---|---|
| Amcache | `parse-registry amcache` | File inventory, paths, SHA-1, PE metadata, devices | Presence is not execution; publisher is not signature validation |
| NTFS `$MFT` | `parse-ntfs mft` | Files, deleted records, paths, timestamps, ADS | Reused records and timestomping can mislead |
| NTFS `$LogFile` | `parse-ntfs logfile` | Redo/undo metadata, old names and timestamps, writes to the USN Journal | Short circular retention; metadata times are not transaction times; no replay |
| Prefetch | `parse-file prefetch` | Program execution, run count, recent run times, loaded files | Only a limited run-time history is retained |
| USN Journal `$J` | `parse-ntfs usn-journal` | Creates, deletes, renames, writes, historical paths | Circular journal; records changes, not file contents |
| Directory indexes `$I30` | `parse-ntfs i30` | Files in a directory; names, times, and sizes of possible deleted or renamed files left in index slack | Slack is partial and says nothing about when an entry was removed |
| NTFS boot sector `$Boot` | `parse-ntfs boot` | The volume serial number LNK, Jump Lists and Prefetch record; geometry and `$MFT` location; boot code hash to compare across hosts | Serial numbers can be changed or repeated; a differing boot code is a lead, not proof of a bootkit |
| NTFS security descriptors `$Secure:$SDS` | `parse-ntfs secure-sds` | Owners and ACLs of files, by the `$MFT`'s `SecurityID`; NULL DACLs and permissive ACLs | Owners can be changed; ACLs say who may access, not who did |
| SRUM | `parse-ese srum` | Per-app/user network and resource use | Usually aggregated; not packet or destination-level proof |
| LNK shortcuts | `parse-file lnk` | Opened targets, arguments, volumes, shares, source machine | A shortcut can be created without the target running |
| Jump Lists | `parse-file jump-lists` | Application-opened items, pinning, usage and embedded LNK data | Application-specific behavior and retention |
| ShimCache | `parse-registry shimcache` | Historical executable paths and compatibility checks | Cache may not be flushed until shutdown; entry alone is not execution |
| Defender quarantine | `parse-file defender-quarantine` | Detection, original path, hashes, download origin, recovered bytes | Detection name is a product verdict; extraction is hazardous |
| Registry hives | `parse-registry read` | Configuration, persistence, deleted/replayed values | Key time is not a value time; live hives need transaction logs |
| Services | `parse-registry services` | Service/driver persistence and failure actions | Configuration does not prove the service started |
| RecentDocs | `parse-registry recentdocs` | Recently opened files and folders per user | Only the most recent item in each list gets a defensible open time |
| Microsoft Office | `parse-registry office` | Documents and folders each Office application opened, with times; documents whose macros the user enabled; macro settings and trusted locations | Enabling macros is not proof a macro ran; trust-record paths are stored as Office writes them |
| Open and Save As dialogs | `parse-registry opensavemru` | Files opened or saved through the common dialogs, by extension, and the programs and folders that used them | Only the most recent entry of each list is dated; opened and saved are not told apart |
| TaskCache | `parse-registry taskcache` | Scheduled-task actions, metadata, hidden tasks | Correlate with task XML and event logs for runtime proof |
| Boot configuration (BCD) | `parse-registry bcd` | Boot entries; recovery turned off, failures ignored, test signing, integrity checks off, safe mode, with when each was set | The store is often not collected (EFI system partition, `\Boot`); settings are inherited |
| Registry autostarts | `parse-registry persistence` | Run keys, Winlogon, IFEO, AppInit/AppCert DLLs, LSA packages, COM hijacks and other autostart locations | Configuration, not proof of execution; most entries are Windows defaults |
| Application compatibility | `parse-registry appcompat` | Compatibility layers, programs the Program Compatibility Assistant saw run, CIT program and module use, custom shim databases | Program Compatibility Assistant entries carry no run time; CIT data is mostly from Windows 7 and 8 |
| FeatureUsage, RecentApps, pins | `parse-registry featureusage` | How often a user switched to, launched or clicked each application; recent apps and their files; taskbar pins and Start menu programs | Counts have no times; window switches, not starts |
| ShellBags | `parse-registry shellbags` | Folders a user opened in Explorer and dialogs, including removable, network, portable-device, ZIP and deleted ones, as full paths | Only the most recent item in each key gets a defensible open time; item times are the folder's, not the visit's |
| BAM and DAM | `parse-registry bam` | The last time each user ran each program, including from removable and network devices | One time per program and user, recent activity only; device paths, not drive letters |
| USB devices | `parse-registry usb` | USB storage and other devices, serial numbers, install/arrival/removal times, drive letters, volume names and serials, users who mounted them | Only the last arrival and removal are kept; drive letters are reused |
| MuiCache | `parse-registry muicache` | Cached program paths, descriptions and companies associated with a user's shell profile | No time per program; the profile is not necessarily the process account, and metadata can be forged |
| UserAssist | `parse-registry userassist` | Programs and shortcuts a user started from Explorer, with run counts, focus time and last run | Only Explorer starts are counted; new profiles are seeded with default items |
| Networks | `parse-registry networks` | Networks joined with first/last connection times, gateway MACs, interfaces and DHCP leases, shares, port proxies, routes | Profile times are local, converted with the current time zone; names like "Network 2" repeat |
| Security settings | `parse-registry security-settings` | Defender (and its exclusions), UAC, WDigest/LSA protection, RDP, firewall, SMBv1, PowerShell logging and AMSI, event log and evidence settings, flagged where they weaken the default | A weakened setting isn't proof of compromise; missing values (Windows' defaults) aren't reported |
| Application history | `parse-registry app-history` | Sysinternals tools a user ran, archive and folder history (WinRAR, 7-Zip, WinZip), OneDrive accounts, Adobe recent files, TeamViewer and CCleaner values | Most list entries have no time; only the most recent entry of each list is dated |
| MRU lists | `parse-registry mru` | Run dialog commands, typed Explorer paths, Explorer searches, typed Internet Explorer addresses (with their times), Paint and WordPad files, XP search terms | Only each list's most recent entry is dated, typed addresses aside; typed is not executed |
| Remote access (user side) | `parse-registry remote-access` | Computers a user reached with Remote Desktop, PuTTY and WinSCP, accepted SSH host keys, mapped drives and shares | A saved or typed target isn't a successful connection; list keys have one time for all their items |
| Installed programs | `parse-registry installed-programs` | Installed programs (machine-wide and per user), Windows Installer products, App Paths, WSL distributions and packaged apps | The install date is the installer's; cleanly uninstalled programs leave no entry |
| SAM | `parse-registry sam` | Local accounts, flags, RIDs, logon/password metadata | Local accounts only; last-logon values are not a complete logon history |
| LSA secrets | `parse-registry lsa-secrets` | Which secrets LSA holds (autologon, service accounts, machine account) and when they were set; cached domain logons in use | Names and times only, nothing decrypted; `DefaultPassword` exists without autologon |
| Audit policy | `parse-registry auditpol` | Which security events the system was set to log, and so which missing events say nothing | The policy when the hive was last written; changes are logged by event 4719 |
| System information | `parse-registry system-info` | Computer and domain names, Windows version and install date, time zone, last shutdown, last user, profiles, SIDs | The configuration when collected; `InstallDate` moves with feature upgrades, and Windows 11 calls itself Windows 10 |

## Collection types

Juntei accepts a single artifact with `-f` or searches a collection recursively with `-d`.
Directory mode is designed for common triage layouts where each host contains paths such as
`C/Windows/System32/config/SYSTEM` or `C/Users/alice/NTUSER.DAT`.

See [Collecting Evidence](../getting-started/collection.md) before analyzing copied registry or ESE
databases: related transaction logs can be essential to reconstructing the state Windows saw.
