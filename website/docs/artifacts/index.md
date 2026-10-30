# Artifact Guide

Windows preserves overlapping traces in the file system, registry, compatibility databases,
user-interface history, resource accounting, and security products. Juntei parses these sources,
but the investigator must still interpret what each source means.

## Choose an artifact by question

| Investigation question | Start here | Corroborate with |
|---|---|---|
| Did this executable run? | [Prefetch](prefetch.md) | [ShimCache](shimcache.md), [SRUM](srum.md), event logs |
| Was this file ever present? | [Amcache](amcache.md), [$MFT](mft.md) | [USN Journal](usn-journal.md), ShimCache |
| When was a file created, renamed, or deleted? | [USN Journal](usn-journal.md) | [$MFT](mft.md), Defender quarantine |
| What did a user open? | [Jump Lists](jump-lists.md), [LNK](lnk.md), [RecentDocs](recentdocs.md) | SRUM, Prefetch |
| Did a program transfer a large amount of data? | [SRUM](srum.md) | Firewall, proxy, DNS, packet or EDR telemetry |
| Was persistence configured? | [Services](services.md), [TaskCache](taskcache.md), [Registry](registry.md) | Event logs, Prefetch, `$MFT` |
| What did Defender remove? | [Defender Quarantine](defender-quarantine.md) | Event logs, `$MFT`, USN Journal |
| Were local accounts added or changed? | [SAM](sam.md) | Security event logs and registry transaction history |

## Read warnings before fields

Every artifact page follows the same sequence:

1. **What it is** and what questions it can answer.
2. **Where to find it** and how collection affects it.
3. **What it proves—and does not prove.**
4. **How to parse it** with Juntei.
5. **How to read every output field.**
6. **How to correlate it** with independent evidence.
7. **Further Details** for low-level research.

!!! danger "Do not equate a parser finding with maliciousness"
    Juntei findings identify patterns worth review, such as a system-binary name outside
    `System32`. Installers, administrators, compatibility layers, and software updates can produce
    unusual records. Validate with context and independent evidence.
