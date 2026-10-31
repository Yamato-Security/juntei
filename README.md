<p align="center">
  <img src="./art/Juntei_Goddess%20of%20Digital%20Forensics_small.png" alt="Juntei, Goddess of Digital Forensics" width="100%">
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

| Artifact | Command | Status |
| --- | --- | --- |
| Amcache (`Amcache.hve`) | `parse-amcache` | Supported |
| `$MFT` | `parse-mft` | Supported |
| Prefetch (`*.pf`) | `parse-prefetch` | Supported |
| USN journal (`$UsnJrnl:$J`) | `parse-usn-journal` | Supported |
| Directory indexes (`$I30`), with possible deleted or renamed file names left in their slack | `parse-i30` | Supported |
| SRUM (`SRUDB.dat`) | `parse-srum` | Supported |
| Shortcuts (`*.lnk`) | `parse-lnk` | Supported |
| Jump lists (`*.automaticDestinations-ms`, `*.customDestinations-ms`) | `parse-jump-lists` | Supported |
| ShimCache / AppCompatCache (`SYSTEM` hive) | `parse-shimcache` | Supported |
| Microsoft Defender quarantine (`Quarantine\Entries`, `ResourceData`, `Resources`) | `parse-defender-quarantine` | Supported |
| Registry hives (`SYSTEM`, `SOFTWARE`, `NTUSER.DAT`, …), with their transaction logs and deleted keys | `parse-registry read` | Supported |
| Services (`SYSTEM` hive), with their failure actions | `parse-registry services` | Supported |
| Recent documents (`NTUSER.DAT` `RecentDocs`), in their MRU order | `parse-registry recentdocs` | Supported |
| Scheduled tasks (`SOFTWARE` `TaskCache`), with their actions and hidden tasks | `parse-registry taskcache` | Supported |
| Local accounts (`SAM`) | `parse-registry sam` | Supported |
| More Windows artifacts | | Planned |
