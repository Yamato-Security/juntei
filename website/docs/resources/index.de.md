# Artikel und Referenzen { #articles-and-references }

Diese Seite sammelt hochwertige Einstiegspunkte für Leserinnen und Leser, die über die Artefakt-
leitfäden hinausgehen möchten. Die Abschnitte **Weitere Informationen** der einzelnen Artefakte enthalten die gezieltesten Low-Level-Links.

## DFIR-Orientierung für Einsteiger { #beginner-dfir-orientation }

- [SANS-Poster „Windows Forensic Analysis“](https://www.sans.org/posters/windows-forensic-analysis/) — eine visuelle Übersicht gängiger Windows-Artefakte und ihrer Speicherorte.
- [SANS-Poster „Hunt Evil“](https://www.sans.org/posters/hunt-evil/) — praktische Ansatzpunkte für das Hunting und Gedächtnisstützen für Windows-Untersuchungen.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) — öffentliche Artefaktbeispiele zum Lernen und zur Parservalidierung.
- [Velociraptor-Artefaktreferenz](https://docs.velociraptor.app/artifact_references/) — sammlungsorientierte Beschreibungen und VQL-Implementierungen für viele Windows-Artefakte.
- [Plaso-Dokumentation](https://plaso.readthedocs.io/) — Zeitleistenkonzepte und Parser für artefaktübergreifende Analysen.

## Dateisystemartefakte { #file-system-artifacts }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: Sicherheitsbeschreibungen](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors) und [SDDL](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) — Sicherheitsbeschreibungen, ACEs und bedingte Ausdrücke.
- [Formatdokumentation von libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [mft-Parser von Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) — NTFS-Parser mit Carving von INDX-Datensätzen und Index-Slack.
- [INDXRipper](https://github.com/harelsegev/INDXRipper) — extrahiert Dateinamen per Carving aus dem Slack von `$I30`.
- [LogFileParser](https://github.com/jschicht/LogFileParser) — detaillierte Forschung zum NTFS-Transaktionsprotokoll und dessen Parsing.
- [ntfs-linker](https://github.com/strozfriedberg/ntfs-linker) — korreliert `$LogFile`, `$MFT` und den USN-Verlauf.

## Ausführungs- und Inventarartefakte { #execution-and-inventory-artifacts }

- [AmcacheParser von Eric Zimmerman](https://github.com/EricZimmerman/AmcacheParser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [libyal libscca](https://github.com/libyal/libscca) — Arbeiten zu Prefetch-Format und -Bibliothek.
- [PECmd von Eric Zimmerman](https://github.com/EricZimmerman/PECmd)
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [AppCompatCacheParser von Eric Zimmerman](https://github.com/EricZimmerman/AppCompatCacheParser)

## Artefakte der Benutzeraktivität { #user-activity-artifacts }

- [Microsoft Open Specifications: Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [Microsoft Open Specifications: Compound File Binary (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [LECmd von Eric Zimmerman](https://github.com/EricZimmerman/LECmd)
- [JLECmd von Eric Zimmerman](https://github.com/EricZimmerman/JLECmd)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [SrumECmd von Eric Zimmerman](https://github.com/EricZimmerman/Srum)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## Registry und Persistenz { #registry-and-persistence }

- [Microsoft: Registry-Werttypen](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [Spezifikation des Windows-Registry-Formats von Maxim Suhanov](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) und [libregf](https://github.com/libyal/libregf) — Referenzen für Low-Level-Parsing und Wiederherstellung von Hives.
- [Dokumentation zu Registry Explorer / RECmd von Eric Zimmerman](https://ericzimmerman.github.io/)
- [Microsoft: Registry-Struktur der Dienste](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: Schema der Aufgabenplanung](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK: Scheduled Task/Job](https://attack.mitre.org/techniques/T1053/)
- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender und Umgang mit Malware { #defender-and-malware-handling }

- [Microsoft: Wiederherstellen von Dateien aus der Quarantäne in Defender](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [Defender-Plugin von dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump) — eine separate Forschungs-/Extraktionsimplementierung.
- [FIRST Malware Analysis Fundamentals](https://www.first.org/global/sigs/malware/analysis/) — Ressourcen zu Handhabung und Analyse.

## Parser-Projekte von Juntei { #juntei-parser-projects }

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`$MFT`, `$I30`, USN-Journal, `$LogFile`, `$Boot`, `$Secure:$SDS`)
- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "Externe Materialien ändern sich"
    Überprüfen Sie Aussagen anhand der betroffenen Windows-Version und Ihrer gesicherten Daten. Ein nützlicher Artikel kann
    eine bestimmte Betriebssystemgeneration, Parserversion oder ein Experiment beschreiben statt einer
    allgemeingültigen Regel.
