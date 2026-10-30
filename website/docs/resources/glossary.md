# DFIR Glossary

| Term | Plain-language meaning |
|---|---|
| Allocated | Record/object is currently in use according to the artifact. |
| Alternate Data Stream (ADS) | Named NTFS data attached to a file besides its normal unnamed content, such as `Zone.Identifier`. |
| Artifact | Persistent data produced by an operating system/application that can answer forensic questions. |
| Carving | Finding records by structure/signature without relying on their normal container/index. |
| Control set | A version of SYSTEM configuration (`ControlSet001`, etc.); `Select\Current` names the active one. |
| Corroboration | Independent evidence supporting the same conclusion. |
| Dirty database/hive | Primary file was not fully synchronized when collected; transaction logs may hold required state. |
| Droid | GUID-like distributed-link-tracking identifier used to follow files/volumes across moves. |
| ESE / JET Blue | Microsoft's Extensible Storage Engine database format, used by SRUM and other Windows data. |
| FILETIME | Count of 100-nanosecond intervals since 1601-01-01 UTC; a number is meaningful as time only when its schema says so. |
| File reference | NTFS entry number plus sequence number identifying a generation of an MFT record. |
| Hive | Binary file containing a Windows Registry tree. |
| INDX record | Block (usually 4 KiB) of an NTFS directory index (`$I30`) listing the directory's files; its slack can keep possible evidence of deleted or renamed file names. |
| LNK | Windows Shell Link/shortcut containing a target and extensive metadata. |
| MRU | Most Recently Used ordering; usually relative order rather than a time for every item. |
| MFT | NTFS Master File Table containing file/directory records and attributes. |
| Prefetch hash | Windows identifier used in `.pf` naming; not a cryptographic content hash. |
| Registry key last-write | Time the key changed; Windows usually does not say which value changed. |
| Resident data | File/stream bytes stored directly in an MFT attribute rather than external data runs. |
| RID | Relative Identifier, the final account/group component under a Windows SID authority/domain. |
| SID | Windows Security Identifier for an account, service, group, or security principal. |
| Slack | Unused bytes inside/after an allocated structure that can retain older data. |
| SRUM | System Resource Usage Monitor database of application/user resource use. |
| Timestomping | Intentional timestamp modification to mislead investigation; heuristics are not proof. |
| Transaction log | Pending/committed changes used to recover a registry or database to a consistent state. |
| USN | Update Sequence Number used by NTFS change-journal records and related metadata. |
| UTC | Coordinated Universal Time; preferred common time basis for cross-system timelines. |
