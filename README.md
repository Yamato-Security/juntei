<p align="center">
  <img src="./art/Juntei_Goddess%20of%20Digital%20Forensics_small.png" alt="Juntei, Goddess of Digital Forensics" width="50%">
</p>

# Juntei

Juntei is a Windows forensics artifact parser by [Yamato Security](https://github.com/Yamato-Security),
a sibling of [Hayabusa](https://github.com/Yamato-Security/hayabusa) for the artifacts that are
not event logs.

## [The Juntei documentation →](https://yamato-security.github.io/juntei/)

**The Juntei documentation explains how to install and use Juntei, collect evidence, parse every
supported artifact, interpret each output field, and avoid common forensic-analysis mistakes.**

## Supported artifacts

Use `juntei auto-parse -d <EVIDENCE> -o <OUTPUT>` to discover multiple collected Windows systems
and parse every supported artifact in parallel. It creates per-computer output directories and can
produce one computer-labelled JSONL file for SIEM ingestion.

### Process Execution

| Artifact | Type |
| --- | --- |
| **Prefetch** (`*.pf`) — Program run counts, recent execution times, and referenced files. | File |
| **Amcache** (`Amcache.hve`) — Inventory of executable paths, hashes, publisher metadata, and devices. | Registry |
| **ShimCache / AppCompatCache** (`SYSTEM` hive) — Cached application paths, timestamps, and compatibility flags. | Registry |
| **SRUM** (`SRUDB.dat`) — Application and user resource usage, including network bytes sent and received. | File |

### File System and File Existence

| Artifact | Type |
| --- | --- |
| **Master File Table** (`$MFT`) — File and directory records, paths, timestamps, alternate data streams, and deleted entries. | NTFS |
| **Transaction log** (`$LogFile`) — Redo and undo metadata that can preserve earlier filenames, timestamps, and directory entries. | NTFS |
| **USN journal** (`$UsnJrnl:$J`) — Records of file creation, modification, renaming, and deletion. | NTFS |
| **Directory indexes** (`$I30`) — Directory entries and possible deleted or renamed filenames left in index slack. | NTFS |

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

### Account Usage

| Artifact | Type |
| --- | --- |
| **Local accounts** (`SAM`) — Local usernames, RIDs, account flags, and recorded logon and password metadata. | Registry |

### Security Products

| Artifact | Type |
| --- | --- |
| **Microsoft Defender quarantine** (`Quarantine\Entries`, `ResourceData`, `Resources`) — Threat records, original paths, hashes, download origins, and recoverable quarantined content. | File |
