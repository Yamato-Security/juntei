<p align="center">
  <img src="./art/Juntei_Goddess%20of%20Digital%20Forensics_small.png" alt="Juntei, Goddess of Digital Forensics" width="65%">
</p>

# Juntei

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

Use `juntei auto-parse -d <EVIDENCE> -o <OUTPUT>` to discover multiple collected Windows systems
and parse every supported artifact in parallel. It groups results by computer and artifact category,
with `<artifact>-logs.txt` files beside the results. Add a computer-labelled SIEM JSONL file with `-j <FILE>`
while keeping the CSV or JSON results.

For individual artifacts, choose `parse-ese`, `parse-file`, `parse-ntfs`, or `parse-registry`.
For example: `juntei parse-file prefetch -d <EVIDENCE> -o prefetch.csv`.

Use [`juntei check-persistence`](https://yamato-security.github.io/juntei/commands/check-persistence/)
to hunt for persistence across registry hives, scheduled tasks, startup files, WMI, and BITS.
This native Rust port of [PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper)
provides 130 checks, configurable allow/block profiles, source evidence, and CSV, JSON, or JSONL
results. Candidates describe persistence configuration; they do not by themselves prove compromise
or execution. Per-check coverage identifies evidence that was missing or only partly readable.

### Process Execution

| Artifact | Type |
| --- | --- |
| **Prefetch** (`*.pf`) — Program run counts, recent execution times, and referenced files. | File |
| **SRUM** (`SRUDB.dat`) — Application and user resource usage, including network bytes sent and received. | ESE |
| **UserAssist** (`NTUSER.DAT`) — Programs and shortcuts a user started from Explorer, with run counts, focus time, and last run times. | Registry |

### File System and File Existence

| Artifact | Type |
| --- | --- |
| **Master File Table** (`$MFT`) — File and directory records, paths, timestamps, alternate data streams, and deleted entries. | NTFS |
| **Transaction log** (`$LogFile`) — Redo and undo metadata that can preserve earlier filenames, timestamps, and directory entries. | NTFS |
| **USN journal** (`$UsnJrnl:$J`) — Records of file creation, modification, renaming, and deletion. | NTFS |
| **Directory indexes** (`$I30`) — Directory entries and possible deleted or renamed filenames left in index slack. | NTFS |
| **Amcache** (`Amcache.hve`) — Inventory of executable paths, hashes, publisher metadata, and devices; an entry alone does not prove execution. | Registry |
| **ShimCache / AppCompatCache** (`SYSTEM` hive) — Cached application paths, timestamps, and compatibility flags; an entry alone does not prove execution. | Registry |

### File and Folder Opening

| Artifact | Type |
| --- | --- |
| **Shortcuts** (`*.lnk`) — Target paths, arguments, timestamps, volumes, network shares, and source-computer information. | File |
| **Jump lists** (`*.automaticDestinations-ms`, `*.customDestinations-ms`) — Application-specific recent and pinned items, usage metadata, and embedded shortcuts. | File |
| **Recent documents** (`NTUSER.DAT` `RecentDocs`) — Recently accessed files and folders in most-recently-used order. | Registry |

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

### Account Usage

| Artifact | Type |
| --- | --- |
| **Local accounts** (`SAM`) — Local usernames, RIDs, account flags, and recorded logon and password metadata. | Registry |

### Security Products

| Artifact | Type |
| --- | --- |
| **Microsoft Defender quarantine** (`Quarantine\Entries`, `ResourceData`, `Resources`) — Threat records, original paths, hashes, download origins, and recoverable quarantined content. | File |
