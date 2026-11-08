<p align="center">
  <img src="./art/Juntei_Goddess%20of%20Digital%20Forensics_small.png" alt="Juntei, Goddess of Digital Forensics" width="65%">
</p>

<h1 align="center">Juntei</h1>

<p align="center">
  <strong>English</strong> | <a href="README-Japanese.md">日本語</a>
</p>

Juntei is a Windows forensics artifact parser by [Yamato Security](https://github.com/Yamato-Security),
a sibling of [Hayabusa](https://github.com/Yamato-Security/hayabusa) for the artifacts that are
not event logs.

Juntei is short for Juntei Kannon (准胝観音), or "Pure Avalokitesvara" in English, a bodhisattva
associated with great compassion. She brings that compassion to your investigations, using her
many arms to parse all the Windows DFIR artifacts you need.

## [The Juntei documentation →](https://yamato-security.github.io/juntei/)

**The Juntei documentation explains how to install and use Juntei, collect evidence, parse every
supported artifact, interpret each output field, and avoid common forensic-analysis mistakes.**

## Supported artifacts

Juntei currently supports 41 of the most popular Windows forensic artifacts and we continue to add more.

Use `juntei auto-parse -d <EVIDENCE> -o <OUTPUT>` to discover multiple collected Windows systems
and parse every supported artifact in parallel. It groups results by computer and artifact category,
with `<artifact>-logs.txt` files beside the results. Add a computer-labelled SIEM JSONL file with `-j <FILE>`
while keeping the CSV or JSON results.

For individual artifacts, choose `parse-ese`, `parse-file`, `parse-ntfs`, `parse-registry`, or
`parse-sqlite`.
For example: `juntei parse-file prefetch -d <EVIDENCE> -o prefetch.csv`.

Use `juntei mount <IMAGE> <MOUNTPOINT>` to mount a VHDX disk image
read-only for other tools such as Hayabusa (FUSE, Linux only), or add `--vhdx` to `auto-parse` to
parse the systems inside every VHDX image of a directory. The VHDX reader and mounting library live in
[Yamato Security/ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

Use [`juntei persistence-hunter`](https://yamato-security.github.io/juntei/commands/persistence-hunter/)
to hunt for persistence across registry hives, scheduled tasks, startup files, WMI, and BITS.
This native Rust port of [PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper)
provides 131 checks, configurable allow/block profiles, source evidence, and CSV, JSON, or JSONL
results. Candidates describe persistence configuration; they do not by themselves prove compromise
or execution. Per-check coverage identifies evidence that was missing or only partly readable.

### Process Execution

| Artifact | Type |
| --- | --- |
| **BAM and DAM** (`SYSTEM`) — The last time each user ran each program, with users named from the SAM and ProfileList. | Registry |
| **Application compatibility** (`SOFTWARE`, `NTUSER.DAT`) — Compatibility layers, programs the Program Compatibility Assistant saw run, CIT program and module use, and custom shim databases. | Registry |
| **Prefetch** (`*.pf`) — Program run counts, recent execution times, and referenced files. | File |
| **SRUM** (`SRUDB.dat`) — Application and user resource usage, including network bytes sent and received. | ESE |
| **UserAssist** (`NTUSER.DAT`) — Programs and shortcuts a user started from Explorer, with run counts, focus time, and last run times. | Registry |
| **FeatureUsage and pins** (`NTUSER.DAT`) — How often a user switched to, launched, or clicked each application; recent apps and their files; taskbar pins and the Start menu's programs. | Registry |
| **MuiCache** (`UsrClass.dat`) — Program paths, descriptions and companies cached in a user's shell profile. | Registry |

### File System and File Existence

| Artifact | Type |
| --- | --- |
| **Master File Table** (`$MFT`) — File and directory records, paths, timestamps, alternate data streams, and deleted entries. | NTFS |
| **Transaction log** (`$LogFile`) — Redo and undo metadata that can preserve earlier filenames, timestamps, and directory entries. | NTFS |
| **USN journal** (`$UsnJrnl:$J`) — Records of file creation, modification, renaming, and deletion. | NTFS |
| **Directory indexes** (`$I30`) — Directory entries and possible deleted or renamed filenames left in index slack. | NTFS |
| **Security descriptors** (`$Secure:$SDS`) — Owners, DACLs and SACLs of files by the `$MFT`'s security ID, readable and as SDDL. | NTFS |
| **Boot sector** (`$Boot`) — Volume serial number (the one LNK and Prefetch files record), geometry, `$MFT` location, and a boot code hash to compare across hosts. | NTFS |
| **Amcache** (`Amcache.hve`) — Inventory of executable paths, hashes, publisher metadata, and devices; an entry alone does not prove execution. | Registry |
| **ShimCache / AppCompatCache** (`SYSTEM` hive) — Cached application paths, timestamps, and compatibility flags; an entry alone does not prove execution. | Registry |

### File and Folder Opening

| Artifact | Type |
| --- | --- |
| **Shortcuts** (`*.lnk`) — Target paths, arguments, timestamps, volumes, network shares, and source-computer information. | File |
| **Microsoft Office** (`NTUSER.DAT` `Software\Microsoft\Office`) — Documents and folders each application opened with their times, documents whose macros the user enabled (Trusted Documents), and macro settings and trusted locations. | Registry |
| **Jump lists** (`*.automaticDestinations-ms`, `*.customDestinations-ms`) — Application-specific recent and pinned items, usage metadata, and embedded shortcuts. | File |
| **MRU lists** (`NTUSER.DAT` `RunMRU`, `TypedPaths`, `WordWheelQuery`, `TypedURLs`, …) — Commands typed into the Run dialog, paths typed into Explorer, Explorer searches, typed Internet Explorer addresses with their times, and Paint's and WordPad's recent files. | Registry |
| **Recent documents** (`NTUSER.DAT` `RecentDocs`) — Recently accessed files and folders in most-recently-used order. | Registry |
| **Open and Save As dialogs** (`NTUSER.DAT` `ComDlg32`) — Files opened or saved through the common dialogs, by extension, and the programs and folders that used them. | Registry |
| **ShellBags** (`UsrClass.dat`, `NTUSER.DAT` `BagMRU`) — Folders a user opened in Explorer, including removable drives, network shares, portable devices, ZIP archives and FTP sites, as full paths. | Registry |

### External Devices

| Artifact | Type |
| --- | --- |
| **USB devices** (`SYSTEM`, `SOFTWARE`, `NTUSER.DAT`) — USB storage, USB and portable devices with their serial numbers and install, arrival and removal times, their volumes' drive letters, names and serial numbers, and the users who had them mounted. | Registry |

### Registry

| Artifact | Type |
| --- | --- |
| **Registry hives** (`SYSTEM`, `SOFTWARE`, `NTUSER.DAT`, …) — Keys, values, configuration, transaction-log replay, and recoverable deleted records. | Registry |

### Startup and Persistence

| Artifact | Type |
| --- | --- |
| **Services** (`SYSTEM` hive) — Service and driver paths, startup settings, accounts, and failure actions. | Registry |
| **Scheduled tasks** (`SOFTWARE` `TaskCache`) — Cached task names, actions, execution metadata, and indicators of hidden tasks. | Registry |
| **Registry autostarts** (`SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat`) — Run keys, Winlogon, Image File Execution Options, AppInit/AppCert DLLs, LSA packages, COM hijacks and other autostart locations, with their ATT&CK techniques. | Registry |
| **Boot configuration** (`BCD`) — Boot entries and their settings: recovery turned off, failures ignored, test signing, integrity checks off and safe mode. | Registry |

### Account Usage

| Artifact | Type |
| --- | --- |
| **Local accounts** (`SAM`) — Local usernames, RIDs, account flags, and recorded logon and password metadata. | Registry |
| **LSA secrets** (`SECURITY`) — Which secrets LSA holds (autologon, service account and machine account passwords) and when they were set, and cached domain logons in use; names and times only, nothing decrypted. | Registry |

### Applications

| Artifact | Type |
| --- | --- |
| **Installed programs** (`SOFTWARE`, `NTUSER.DAT`) — Programs installed machine-wide and per user, Windows Installer products, App Paths, WSL distributions, and packaged apps, with install dates and key times. | Registry |

### Lateral Movement

| Artifact | Type |
| --- | --- |
| **Remote access, user side** (`NTUSER.DAT`) — Computers a user reached with Remote Desktop, PuTTY and WinSCP, the accounts used, accepted SSH host keys with their fingerprints, mapped drives and shares. | Registry |
| **User Access Logging** (`Current.mdb`, `{GUID}.mdb`, `SystemIdentity.mdb`) — The client addresses and accounts that used a Windows Server's roles (file shares, Active Directory, DHCP, IIS and more), with daily request counts. | ESE |

### System Information

| Artifact | Type |
| --- | --- |
| **System information** (`SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY`) — Computer and domain names, Windows version and install date, time zone and the local times it kept, last shutdown, last user, profiles, and SIDs. | Registry |

### Applications

| Artifact | Type |
| --- | --- |
| **Application history** (`NTUSER.DAT`) — Sysinternals tools a user ran, WinRAR, 7-Zip and WinZip archive and folder history, OneDrive accounts, Adobe recent files, and TeamViewer and CCleaner values. | Registry |

### System Information

| Artifact | Type |
| --- | --- |
| **Audit policy** (`SECURITY` `PolAdtEv`) — Which security events the system was set to log, per subcategory, and so which missing events say nothing. | Registry |

### Browser Activity

| Artifact | Type |
| --- | --- |
| **Web browsers** (`History`, `Cookies`, `Web Data`, `places.sqlite`, `cookies.sqlite`, `formhistory.sqlite`) — The visits, downloads, cookies and form entries of Chrome, Edge, Brave, Opera, Vivaldi, Firefox and other Chromium- and Firefox-based browsers, read with their write-ahead logs, with deleted visits, pages, downloads and form entries recovered. | SQLite |
| **Internet Explorer and legacy Edge** (`WebCacheV01.dat`) — The history (with the local files File Explorer opened), downloads, cookies and cached files of Internet Explorer 10 and 11, legacy Edge and other programs that use the Windows Internet functions, with each cached file's path and HTTP headers. | ESE |

### Network Activity

| Artifact | Type |
| --- | --- |
| **Networks** (`SOFTWARE`, `SYSTEM`) — Networks joined with first and last connection times in UTC, gateway MAC addresses and DNS suffixes, interfaces and DHCP leases, shares, port proxies and persistent routes. | Registry |

### Security Products

| Artifact | Type |
| --- | --- |
| **Microsoft Defender quarantine** (`Quarantine\Entries`, `ResourceData`, `Resources`) — Threat records, original paths, hashes, download origins, and recoverable quarantined content. | File |
| **Security settings** (`SOFTWARE`, `SYSTEM`, `NTUSER.DAT`) — Defender and its exclusions, UAC, WDigest and LSA protection, RDP, the firewall, SMBv1, PowerShell logging and AMSI, and event log settings, flagged where they weaken the default. | Registry |

## License

Juntei is released under the [Reciprocal Public License 1.5](LICENSE) (RPL-1.5) and copyright
Shirofune Security Co., Ltd.
The licenses of the crates and other third-party code and data that Juntei is built with are in
[`THIRD_PARTY_NOTICES`](THIRD_PARTY_NOTICES/), and the licenses of the test files are next to them
in [`test_files`](test_files/).
