# Befehlsliste { #command-list }

Die einzelnen Parser sind nach ihrem Quelltyp gruppiert. Führen Sie `juntei --help`,
`juntei <SOURCE> --help` oder `juntei <SOURCE> <ARTIFACT> --help` aus, um sie zu erkunden:

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Für eine Windows-Beweissammlung von mehreren Computern verwenden Sie
[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md). Der Befehl ruft dieselben gruppierten Befehle auf
und organisiert Ergebnisse und Protokolle nach Computer und Untersuchungskategorie.

## [Disk-Images](mount.md) { #disk-images }

| Befehl | Eingabe | Ausführlicher Leitfaden |
|---|---|---|
| `mount` | VHDX- oder E01-Disk-Image | [Disk-Images einbinden](mount.md) |

## [ESE-Befehle](user-activity.md) { #ese-commands }

| Befehl | Eingabeartefakt | Ausführlicher Leitfaden |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ESE `Current.mdb`, `{GUID}.mdb`, `SystemIdentity.mdb` | [Benutzerzugriffsprotokollierung (UAL)](../artifacts/ual.md) |
| `parse-ese webcache` | ESE `WebCacheV01.dat` | [Internet Explorer und Edge Legacy](../artifacts/webcache.md) |

## [Dateibefehle](execution.md) { #file-commands }

| Befehl | Eingabeartefakt | Ausführlicher Leitfaden |
|---|---|---|
| `parse-file defender-quarantine` | Defender-Quarantäneverzeichnis oder -eintrag | [Defender-Quarantäne](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [LNK-Verknüpfungen](../artifacts/lnk.md) |
| `parse-file outlook` | `*.pst`, `*.ost`, `*.msg`, `*.oft` | [Outlook](../artifacts/outlook.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |

## [NTFS-Befehle](filesystem.md) { #ntfs-commands }

| Befehl | Eingabeartefakt | Ausführlicher Leitfaden |
|---|---|---|
| `parse-ntfs boot` | `$Boot` oder ein NTFS-Volume-Image | [Bootsektor ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30` (`$INDEX_ALLOCATION`) oder ein Datenträger-Image | [Verzeichnisindizes ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | NTFS `$LogFile` | [NTFS-Transaktionsprotokoll](../artifacts/logfile.md) |
| `parse-ntfs mft` | NTFS `$MFT` | [Master File Table](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS` (`$Secure_$SDS`) | [Sicherheitsbeschreibungen ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [USN-Journal](../artifacts/usn-journal.md) |

## [Registry-Befehle](security-registry.md) { #registry-commands }

| Befehl | Eingabeartefakt | Ausführlicher Leitfaden |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`, `SYSTEM` | [Netzwerke](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [LSA-Secrets](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD` (und `BCD-Template`) | [Startkonfigurationsdaten (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`, `NTUSER.DAT` | [Anwendungskompatibilität](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [Überwachungsrichtlinie](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft Office](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage, RecentApps und an die Taskleiste angeheftete Elemente](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [Anwendungsverlauf](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [Listen zuletzt verwendeter Elemente (MRU)](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM` (mit `SAM`, `SOFTWARE` für Benutzernamen) | [BAM / DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat` (`NTUSER.DAT` unter Windows XP) | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`, `NTUSER.DAT` | [Installierte Programme](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [Dialoge zum Öffnen und Speichern unter](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat` | [Registry-Autostarts](../artifacts/persistence.md) |
| `parse-registry read` | Beliebiger primärer Registry-Hive | [Registry-Hives](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT` (mit `SOFTWARE` für Benutzer-SIDs) | [Fernzugriff (Benutzerseite)](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [Lokale SAM-Konten](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` | [Sicherheitseinstellungen](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [Windows-Dienste](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` oder ein exportierter AppCompatCache-Wert | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | `SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` eines Systems | [Systeminformationen](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [Aufgabenplaner-Cache](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM`, mit `SOFTWARE` und `NTUSER.DAT` | [USB-Geräte](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`, `NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## [SQLite-Befehle](sqlite.md) { #sqlite-commands }

| Befehl | Eingabeartefakt | Ausführlicher Leitfaden |
|---|---|---|
| `parse-sqlite browser` | `History`, `Cookies`, `Web Data`, `places.sqlite`, `cookies.sqlite`, `formhistory.sqlite` | [Webbrowser](../artifacts/browser.md) |

## Quelltypen und Formatänderungen { #source-types-and-format-changes }

Die Gruppen beschreiben die Formate, die Juntei derzeit unterstützt. `parse-registry amcache` liest
`Amcache.hve`; dies impliziert keine Unterstützung für SQLite-basierte Amcache-Inventardatenbanken. Die Parser-Crates
für Artefakte bleiben getrennt, sodass die Unterstützung für ein weiteres Speicherformat unabhängig hinzugefügt werden kann.

Die früheren flachen Parser-Namen wurden durch die oben genannten Pfade aus Quelle und Artefakt ersetzt. Behalten Sie
bestehende Parser-Optionen nach dem Artefaktnamen bei. `auto-parse` und die bestehenden Registry-Unterbefehle
behalten ihre Namen.

## [Persistence Hunter](persistence-hunter.md) { #persistence-hunter }

`juntei persistence-hunter` sucht über mehrere Quellformate hinweg und ist daher ein Analysebefehl
der obersten Ebene. Er ergänzt das vollständige Konfigurationsinventar aus `parse-registry persistence` um
131 Prüfungen, Allow-/Block-Klassifizierung, Payload-Metadaten, Abdeckung und Zeitstempelkorrelation.

## Verwendung auf oberster Ebene { #top-level-usage }

```text
Usage: juntei <COMMAND>

Commands:
  auto-parse          Discover and parse Windows evidence
  help                Print this message or the help of the given subcommand(s)
  mount               Mount a VHDX or E01 disk image read-only (FUSE)
  parse-ese           Parse ESE databases
  parse-file          Parse file artifacts
  parse-ntfs          Parse NTFS artifacts
  parse-registry      Parse registry hives
  parse-sqlite        Parse SQLite databases
  persistence-hunter  Hunt offline Windows persistence

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Die Quellseiten geben die Befehlsverwendung wieder und enthalten kurze Beispiele. Die Artefaktleitfäden erläutern
die Beweise, Ausgabefelder, Einschränkungen und die Untersuchungsstrategie.
