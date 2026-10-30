# Articles and References

This page collects high-quality starting points for readers who want to go beyond the artifact
guides. The per-artifact **Further Details** sections contain the most targeted low-level links.

## Beginner DFIR orientation

- [SANS Windows Forensic Analysis poster](https://www.sans.org/posters/windows-forensic-analysis/) — a visual map of common Windows artifacts and locations.
- [SANS Hunt Evil poster](https://www.sans.org/posters/hunt-evil/) — practical hunting pivots and Windows investigation reminders.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) — public artifact samples for learning and parser validation.
- [Velociraptor artifact reference](https://docs.velociraptor.app/artifact_references/) — collection-oriented descriptions and VQL implementations for many Windows artifacts.
- [Plaso documentation](https://plaso.readthedocs.io/) — timeline concepts and parsers for cross-artifact analysis.

## File-system artifacts

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [libfsntfs format documentation](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [Omer Ben-Amram's mft parser](https://github.com/omerbenamram/mft)

## Execution and inventory artifacts

- [Eric Zimmerman's AmcacheParser](https://github.com/EricZimmerman/AmcacheParser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [libyal libscca](https://github.com/libyal/libscca) — Prefetch format/library work.
- [Eric Zimmerman's PECmd](https://github.com/EricZimmerman/PECmd)
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Eric Zimmerman's AppCompatCacheParser](https://github.com/EricZimmerman/AppCompatCacheParser)

## User-activity artifacts

- [Microsoft Open Specifications: Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [Microsoft Open Specifications: Compound File Binary (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Eric Zimmerman's LECmd](https://github.com/EricZimmerman/LECmd)
- [Eric Zimmerman's JLECmd](https://github.com/EricZimmerman/JLECmd)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [Eric Zimmerman's SrumECmd](https://github.com/EricZimmerman/Srum)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## Registry and persistence

- [Microsoft: Registry value types](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [Maxim Suhanov's Windows registry format specification](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) and [libregf](https://github.com/libyal/libregf) — low-level hive parsing and recovery references.
- [Eric Zimmerman's Registry Explorer / RECmd documentation](https://ericzimmerman.github.io/)
- [Microsoft: Services registry tree](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: Task Scheduler schema](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK: Scheduled Task/Job](https://attack.mitre.org/techniques/T1053/)
- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender and malware handling

- [Microsoft: Restore quarantined files in Defender](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [dissect.target Defender plugin](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump) — a separate research/extraction implementation.
- [FIRST Malware Analysis Fundamentals](https://www.first.org/global/sigs/malware/analysis/) — handling and analysis resources.

## Juntei parser projects

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-mft-parser](https://github.com/Yamato-Security/ys-mft-parser)
- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-usn-journal-parser](https://github.com/Yamato-Security/ys-usn-journal-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "External material changes"
    Verify claims against the affected Windows version and your collected data. A useful article can
    describe a specific operating-system generation, parser version, or experiment rather than a
    universal rule.
