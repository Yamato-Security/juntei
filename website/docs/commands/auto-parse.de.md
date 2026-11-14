# Sammlungen automatisch analysieren { #automatic-collection-parsing }

`auto-parse` ist der schnellste Weg, eine Windows-Triage-Sammlung mehrerer Computer in ein
strukturiertes Analyseverzeichnis zu überführen. Übergeben Sie das Stammverzeichnis der Beweise und ein neues Ausgabeverzeichnis. Der Befehl erkennt
die gesicherten Systeme, ermittelt ihre Computernamen und führt jeden passenden Juntei-Parser
parallel aus.

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

Die Standardausgabe ist CSV. Fügen Sie für `jq` oder den SIEM-Import eine JSONL-Zeitleiste hinzu und behalten Sie dabei die CSV-Ergebnisse
bei:

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "Ausgabe- und Beweisverzeichnisse getrennt halten"
    Juntei lehnt ein Ausgabeverzeichnis ab, das innerhalb von `<EVIDENCE>` liegt oder `<EVIDENCE>` enthält. Dies
    verhindert, dass erzeugte Daten erneut als Beweise erkannt werden, und verhindert, dass `--clobber`
    Dateien in der Quellsammlung verändert.

## VHDX-Disk-Images { #vhdx-disk-images }

Mit `--vhdx` liest `auto-parse` VHDX nativ unter **Linux, macOS und Windows**, über [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser). OS-Mount, FUSE und Administratorrechte sind nicht erforderlich.

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Die Parser benötigen Dateipfade. Daher kopiert Juntei die Dateien, die die Parser lesen, mit begrenzten Puffern in ein privates temporäres Verzeichnis: Registry-Hives und ihre Transaktionsprotokolle, NTFS-Metadatendateien, Datenbanken, Prefetch-Dateien, Verknüpfungen, Sprunglisten, Outlook-Dateien, Indexdateien des Papierkorbs und Defender-Quarantäneordner. Jede andere Datei wird ein leerer Platzhalter mit ihrer ursprünglichen Änderungszeit, sodass Verzeichnisse und Pfade wie auf dem Datenträger aussehen. persistence-hunter liest den Inhalt eines Platzhalters nur bei Bedarf aus dem Abbild, etwa eine ausführbare Datei, auf die ein Persistenzeintrag verweist. **Es muss Platz für die kopierten Dateien vorhanden sein**, zusätzlich zu den Ergebnissen. Nullbereiche werden, soweit unterstützt, als Löcher geschrieben. Namen, Struktur, Hardlinks und Änderungszeiten bleiben erhalten. LNK- und Sprunglisten-Quellzeiten stammen aus einem Snapshot der ursprünglichen NTFS-Metadaten, niemals aus den temporären Dateien. Regulärer Abschluss, auch mit Parserfehlern, entfernt die Kopien; erzwungene Beendigung kann sie zurücklassen. Die Quelle bleibt schreibgeschützt; ausstehende VHDX-Logs werden nur im Speicher angewendet.

Hosterkennung, CSV/JSON/JSONL und `-j` nutzen dieselbe Verarbeitung wie normale Beweisverzeichnisse. `Image` in `manifest.json` bezeichnet das Originalimage, `EvidenceRoot` den internen relativen Pfad. Lesefehler und unsichere Namen erscheinen in `ImageErrors` sowie Fehlerzeilen von `manifest.csv`. Lesbare Ergebnisse bleiben erhalten; unvollständige Eingaben ergeben Exitcode 1. Unsichere oder mehrdeutige Namen werden nicht umbenannt. Benannte NTFS-Streams werden nicht als zusätzliche Dateien extrahiert; als normale Dateien gesammelte Artefakte bleiben verfügbar.

Der separate [`mount`-Befehl](mount.md) stellt anderen Programmen ein Dateisystem bereit und benötigt weiterhin Linux FUSE. `auto-parse --vhdx` verwendet ihn nicht.

Das Staging verwendet standardmäßig ein privates Verzeichnis im Dateisystem der Ausgabe. Mit `--temp-dir` wählen Sie ein vorhandenes Verzeichnis auf einem anderen Datenträger. Quellmetadaten bleiben separat im Ausgabedateisystem, damit bereits lesbare Dateien auch bei vollem Staging-Datenträger analysiert werden können. Platz für Kopien und Ergebnisse einplanen; ein volles Ausgabedateisystem verhindert das Speichern. Ctrl+C stoppt Parserprozesse, entfernt den privaten Arbeitsbereich und beendet mit Code 130. SIGKILL oder Stromausfall können Kopien hinterlassen. Nicht lesbare Unterverzeichnisse werden bei der Imagesuche übersprungen; ein nicht lesbares Eingabewurzelverzeichnis bleibt ein Fehler.

Das Staging-Verzeichnis muss außerhalb des Ausgabeverzeichnisses liegen. Unter Linux und macOS stoppen auch SIGTERM und SIGHUP die Parserprozesse und bereinigen den Arbeitsbereich.

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## E01- und Ex01-Datenträgerabbilder { #e01-and-ex01-disk-images }

`--e01` liest physische EWF1- (`.E01`) und EWF2-Abbilder (`.Ex01`) nativ unter **Linux, macOS und Windows**, ohne FUSE, Betriebssystem-Mount oder Administratorrechte. Es verwendet [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) und dieselbe oben beschriebene NTFS-Verarbeitung.

Bei geteilten Abbildern das erste Segment angeben. Alle zugehörigen Dateien einschließlich `.E02` müssen mit ihren ursprünglichen Namen daneben liegen. Ganze Datenträger (MBR/GPT) und einzelne NTFS-Volumes werden unterstützt. Für gemischte Verzeichnisse `--vhdx --e01` verwenden.

Dieselben Anforderungen an temporären Speicher und Quellmetadaten gelten. E01-Abbilder enthalten meist ganze Datenträger, und alle Abbilder werden vor Beginn der Analyse bereitgestellt; kopiert werden aber nur die Dateien, die die Parser lesen, meist ein kleiner Teil des belegten Speichers. Abbilder werden nur gelesen. Erfassungslücken und beim Öffnen oder Lesen erkannte Schäden erscheinen in `ImageErrors` und `manifest.csv`; lesbare Ergebnisse bleiben erhalten, der Exit-Code ist 1. Geprüft werden gelesene Daten, nicht alle Chunks oder der Hash des gesamten Mediums. Logische Dateien `.L01` und `.Lx01` werden abgewiesen; eine Passwortoption gibt es nicht.

Der separate Befehl `mount` benötigt weiterhin Linux FUSE, die native automatische Verarbeitung nicht.

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## Ausgabezeitzone { #output-time-zone }

Geparste Zeitstempel werden standardmäßig in UTC ausgegeben, unabhängig von der lokalen Zeitzone des Analysecomputers. UTC
verwendet die Form der Windows-Ereignisprotokolle `2026-10-20T08:46:38.2001668Z`: ein ISO-8601-Datum mit Uhrzeit, `T`
zwischen Datum und Uhrzeit und `Z` für UTC. Die verfügbaren Nachkommastellen bleiben erhalten.
`GeneratedUTC` in `manifest.json` verwendet ebenfalls diese Form. `Start time` und `End time` in der Konsole
sind die lokale Zeit des Analysecomputers, etwa `2026-11-04 09:15:02`, unabhängig von der Ausgabezeitzone.

Wählen Sie mit `-Z, --timezone` einen festen Anzeige-Offset:

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

Ein Offset ist ein vorzeichenbehafteter, zweistelliger Wert `HH:MM` (Stunden `00–23`, Minuten `00–59`). Namen wie
`JST`, verkürzte Werte wie `+9` und ungültige Bereiche erzeugen einen Fehler mit Beispielsyntax.
Lassen Sie die Option weg oder verwenden Sie `--timezone "+00:00"` für UTC. Ein fester Offset wendet keine Sommerzeitregeln
an.

Die Option gilt für geparste Zeitstempel in den CSV-, JSON- und JSONL-Ergebnissen der einzelnen Artefakte sowie
im zusätzlichen JSONL-Export `-j`. So wird beispielsweise aus `2026-10-20T08:46:38.2001668Z`
`2026-10-20T17:46:38.2001668+09:00`; beide beschreiben denselben Zeitpunkt.

Feldnamen bleiben unverändert, auch vorhandene Namen, die auf `UTC` enden. Umgerechnete Werte tragen
explizit den gewählten Offset. `OutputTimeZone` in beiden Manifesten hält den angeforderten Anzeige-Offset fest,
während `GeneratedUTC` als Laufmetadatum in UTC bleibt. Die Protokolle der einzelnen Parser behalten deren native UTC-Zusammenfassungen bei.

Rohe FILETIME-/Epoch-Zahlen, Wörterbücher von Registry-Werten, die Zeitzonenkonfiguration der Quelle, Text-Metadaten,
reine Datumswerte und lokale Uhrzeitwerte ohne bekannten Offset bleiben unverändert. Juntei
fügt einer unbekannten lokalen Zeit kein `Z` hinzu. Aus FAT abgeleitete Shell-Item-Uhrzeiten (`Item*UTC` in
ShellBags/OpenSaveMRU und `TargetItems` von LNK/Jump Lists) haben keinen aufgezeichneten Offset; ihre historischen
UTC-Bezeichnungen oder das ausgegebene Suffix `Z` belegen nicht, dass es sich um UTC handelt, daher lässt die Offset-Option sie unverändert.
CSV-Spalten mit Freitextzusammenfassungen behalten
ihren aufgezeichneten Text; JSON/JSONL bewahrt verschachtelte geparste Zeitstempel als strukturierte Felder.

Für UTC wird die vorhandene Parserausgabe direkt verwendet. Ein expliziter Offset ungleich null fügt in jedem Parser-Worker
einen streamenden Formatierungsdurchlauf hinzu; er ändert die Anzeigewerte, ohne die Beweise zu verändern.

Würde ein Offset einen Zeitstempel außerhalb der Jahre `0000–9999` verschieben, behält dieser Wert seinen ursprünglichen
expliziten Offset. Werte mit mehr als neun Nachkommastellen bleiben ebenfalls unverändert, statt
gerundet zu werden. Juntei protokolliert eine Warnung in der Datei `*-logs.txt` des Parsers und rechnet die übrigen
Zeitstempel um. Fehlerhaftes CSV/JSON oder ein Ausgabefehler lassen die Aufgabe weiterhin fehlschlagen, und die betroffene
ursprüngliche Ergebnisdatei bleibt unverändert.

## Was erkannt wird { #what-it-discovers }

Juntei sucht rekursiv nach Verzeichnissen namens `Windows`, die ein Verzeichnis `System32`, `Prefetch`
oder `AppCompat` enthalten, ohne Berücksichtigung der Groß-/Kleinschreibung. Die Erkennung der beiden letzteren unterstützt auch gezielte
Sammlungen, die nur Prefetch oder Amcache enthalten. Dadurch werden Benutzerprofilverzeichnisse wie
`AppData\Roaming\Microsoft\Windows` ausgeschlossen. Verschachtelte Altinstallationen wie
`C:\Windows.old\Windows` werden dem äußeren gesicherten Volume zugeordnet, statt als doppelte
Computer zu erscheinen. Die standardmäßige maximale Tiefe beträgt 10. Damit werden gängige Velociraptor-Pfade abgedeckt, etwa:

```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

Ändern Sie das Limit nur, wenn die Sammlung tiefer verschachtelt ist:

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

Für jedes erkannte System wird der Computername in dieser Reihenfolge gewählt:

1. `hostname` in der nächstgelegenen übergeordneten `client_info.json` (der Wert wird im Manifest als `FQDN` beibehalten).
2. `ComputerName` in einem gesicherten `SYSTEM`-Registry-Hive.
3. Der nächstgelegene verwertbare Name eines Sammlungsverzeichnisses, wobei Laufwerks- und Velociraptor-Gerüstverzeichnisse (`uploads/auto`)
   übersprungen und ein Suffix wie `_F.…` entfernt wird.

Namen werden so bereinigt, dass sie als Verzeichnisnamen verwendbar sind. Doppelte Namen erhalten sowohl in ihrem
Ausgabeverzeichnis als auch im Wert `Comp` ein numerisches Suffix, damit unterschiedliche Systeme in JSON unterscheidbar bleiben.
Das Manifest bewahrt den Hostnamen exakt wie erkannt in einem separaten Feld `Hostname`.

Nach der Auflistung der Systeme gibt `auto-parse` direkt über
`Parser tasks` eine Zeile `Timezone:` aus. Sie zeigt die gewählte Ausgabezeitzone: standardmäßig `UTC` oder einen Wert wie
`UTC+09:00`, wenn `--timezone "+09:00"` verwendet wird. Die Bezeichnung ist grün, und der Wert verwendet die
Standardtextfarbe des Terminals, wie die übrigen Zusammenfassungswerte. `-K, --no-color` gibt die Zeile ohne Farbe aus.
Die Zeitzonenkonfiguration der gesicherten Systeme finden Sie in den Ergebnissen der
[Systeminformationen](../artifacts/system-info.md).

## Ausgeführte Parser { #parsers-it-runs }

Fällt ein Parser aus, werden die übrigen nicht angehalten. `auto-parse` führt für jeden erkannten
Computer diese Jobs aus. Die Jobs verwenden dieselben nach Quelle gruppierten Befehle wie das manuelle Parsen, etwa
`parse-ntfs mft`, `parse-registry amcache`, `parse-ese srum` und `parse-file prefetch`:

- Amcache
- `$MFT`
- `$LogFile`-Redo/Undo-Datensätze von NTFS
- Prefetch-Datensätze und eine separate Prefetch-Ausführungszeitleiste
- USN-Journal
- `$I30`-Verzeichnisindizes
- `$Boot`-Bootsektoren und `$Secure:$SDS`-Sicherheitsbeschreibungen
- SRUM
- Benutzerzugriffsprotokollierung (UAL)
- Webbrowser-Verlauf, Downloads, Cookies und Formularverlauf (Chromium- und Firefox-basierte Browser)
- WebCache von Internet Explorer und dem alten Edge: Verlauf, Downloads, Cookies und Cache-Einträge
- PST-, OST-, MSG- und OFT-Dateien von Outlook: Ordner, Elemente, Empfänger, Anhänge, Kontakte, Kalenderelemente und Aufgaben
- LNK-Verknüpfungen
- Jump Lists
- Papierkorb: ursprüngliche Pfade, Größen, Löschzeiten und Benutzer gelöschter Elemente
- ShimCache
- Persistence Hunter (131 quellenübergreifende Persistenzprüfungen mit Allow-/Block-Profilen)
- Metadaten der Defender-Quarantäne
- Verlauf der Dialoge „Öffnen“ und „Speichern unter“
- alle Registry-Schlüssel und -Werte, einschließlich wiederherstellbarer gelöschter Daten
- Dienste, RecentDocs, TaskCache, SAM-Konten, Registry-Autostart-Speicherorte, UserAssist, BAM,
  installierte Programme, MuiCache, ShellBags und USB-Geräte
- Systeminformationen: Namen, Windows-Version, Zeitzone, letztes Herunterfahren, Profile und SIDs
- die Überwachungsrichtlinie: welche Sicherheitsereignisse das System protokollieren sollte
- Startkonfiguration (BCD und BCD-Template)
- Netzwerkprofile, Schnittstellen, Freigaben, Portproxys und Routen
- Namen und Zeitstempel der LSA-Secrets sowie verwendete zwischengespeicherte Domänenanmeldungen (es wird nichts entschlüsselt)

Persistence Hunter behält jeden Schweregrad bei (`--min-severity info`), einschließlich teilweiser Allow-Treffer.
`LOW` ist eine Einstufung, die untersucht werden sollte; fehlende Signaturmetadaten können sie auch für bösartige
Befehle erzeugen. Das automatische Inventar verwirft diese Einträge nicht anhand der standardmäßigen
Schwelle „medium“ des manuellen Befehls.

Der Hunter verwendet innerhalb jedes automatischen Parser-Prozesses einen Worker. So wird verhindert, dass `auto-parse -T <N>`
sein Prozessbudget in jedem Kindprozess um einen weiteren Pool in Größe der CPU-Anzahl vervielfacht. Das eigenständige
`persistence-hunter -T <N>` steuert die eigenen Worker-Threads des Hunters.

Die CSV-Ausgabe von `$MFT` enthält alle Zeitstempel und Kurznamen. Die Registry-Ausgabe enthält Rohdaten und eine
FILETIME-Interpretation von `REG_QWORD`-Werten. Die JSON- und JSONL-Ausgabe von SRUM enthält Metadaten zum
Wertzustand. Die Defender-Quarantäneextraktion bleibt optional.

Mit `--mft-slack` enthält die `$MFT`-Ausgabe auch die Dateinamen, die im Slack von FILE-Datensätzen verblieben sind, so wie [`parse-ntfs mft -x`](../artifacts/mft.md) sie schreibt: in CSV mit `FromSlack` markierte Zeilen, in JSON und JSONL `SlackFileNames`. `--code-page` gilt für LNK-Dateien, Jump Lists, ShellBags und OpenSaveMRU.

Ein nicht gesichertes Artefakt ist eine normale Aufgabe mit dem Status `skipped`, kein Fehler.


## Dateisystemrekonstruktion { #filesystem-reconstruction }

Die Dateisystemrekonstruktion ist **standardmäßig deaktiviert**. Mit `-R, --reconstruct-filesystem` wird für jede gesammelte MFT ein durchsuchbarer Verzeichnisbaum hinzugefügt. Namen und Inhalte der vorhandenen Tabellen bleiben in CSV, JSON und JSONL unverändert.

Bäume entstehen in einem privaten benachbarten Arbeitsverzeichnis und werden erst vollständig veröffentlicht. Behandelte Fehler und Unterbrechungen räumen es auf und erhalten den alten Baum. Erzwungenes Beenden kann `.juntei-reconstruction-*`-Verzeichnisse zurücklassen; diese blockieren keinen neuen Versuch und werden kein aktiver Volume-Baum.

Ein aus dem Sammlungspfad abgeleitetes Laufwerk erhält etwa `FileSystem/C`. Die MFT selbst speichert keinen Laufwerksbuchstaben. Ohne erkennbare Bezeichnung gilt `UnknownVolume`; mehrere MFTs desselben Laufwerks bleiben getrennt, etwa `C-MFT-2`.

Nicht zugewiesene Dateien und Verzeichnisse erhalten `-deleted`, einschließlich gelöschter Elternverzeichnisse. Erhaltene residente unbenannte DATA-Inhalte werden wiederhergestellt. Nicht residente oder unbekannte Inhalte werden leere Platzhalter; logische Größe und wiederhergestellte Bytes sind verschieden.

`-i30` kennzeichnet zusätzliche Indexnamen ohne bestätigte Löschung. Residente MFT-Indizes und separat gesammelte I30-Dateien desselben Volumes werden verwendet. Slack kann Umbenennungen oder Kopien enthalten. Historische Namen übernehmen keine Inhalte anderer Dateigenerationen.

`_Juntei-Unresolved` enthält Einträge mit fehlenden, wiederverwendeten, zyklischen oder zu tiefen Elternpfaden. Unsichere Namen werden maskiert, Kollisionen getrennt gespeichert und Originalnamen protokolliert. DOS-Aliasse entfallen bei erhaltenem Langnamen; Hardlinks werden unabhängige Kopien.

`_Juntei-Reconstruction.csv` dokumentiert die Rekonstruktion: `DataStatus`, `LogicalSize`, `RecoveredBytes`, `OriginalPath`, `OutputPath`. Die Herkunft enthalten `EntryNum`, `SN`, `ParentEntryNum`, `ParentSN`, `Source`, `FromSlack`, `Offset`. Laufwerk, Löschzustand und Verzeichnistyp werden ebenfalls erfasst. Ein unvollständiges Inventar endet auf .partial.csv; der endgültige Name entsteht erst nach Erfolg.

`manifest.json` / `manifest.csv`: `ReconstructionRoot`, `ReconstructionInventory`, `ReconstructionStatus`, `ReconstructionError`.

Unlesbare oder beschädigte optionale I30-Eingaben werden als `i30_input_error` protokolliert; gültige MFT-Ergebnisse bleiben erhalten.

Dateien mit wiederhergestelltem nicht leerem residentem Inhalt erhalten vor dem Schreiben `.dontrun`. Originalnamen bleiben im Inventar; leere Platzhalter behalten ihre Namen. Inhalte niemals ausführen. Reparse Points, ACLs, Originalzeiten und Ausführungsrechte werden nicht wiederhergestellt. Benannte Streams separat extrahieren.

Die Rekonstruktion kann Hunderttausende Dateien erzeugen; auch leere Platzhalter benötigen Inodes und Verzeichnismetadaten. Aktivieren Sie sie nur, wenn Sie den Baum benötigen:

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> --reconstruct-filesystem
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -R
```

Überschreiben ersetzt nur vollständige inventarisierte Bäume ohne fehlende, unbekannte Objekte oder Links. Teilbäume und Zusatzdateien wie `.DS_Store` bleiben erhalten. Auto-parse überspringt nur die Rekonstruktion und protokolliert Verzeichnis und Grund im Log und beiden Manifesten; MFT und andere Parser laufen weiter. Schreibfehler werden separat gemeldet, führen insgesamt zu Fehlerstatus und erhalten geparste Ergebnisse. Der Einzelbefehl lehnt blockierte Ziele weiterhin ab.


## Ausgabestruktur { #output-layout }

Die Ergebnisse sind nach Computer und Untersuchungskategorie geordnet. Kategorieverzeichnisse verwenden
PascalCase, etwa `Accounts` und `FileAndFolderOpening`. Tabellen von Amcache und Jump Lists liegen
mit Dateinamenpräfixen direkt in ihren Kategorien. Computerverzeichnisnamen behalten ihre erkannte Schreibweise bei. Bestehende Ausgabeverzeichnisse
werden nicht migriert; wählen Sie beim Wechsel von der älteren Benennung ein neues Ausgabeverzeichnis. Jeder Parser schreibt eine
Datei `<artifact>-logs.txt` neben seine Ergebnisse, auch wenn das Artefakt nicht gesichert wurde.
CSV behält getrennte Tabellen bei, da jedes Artefakt andere Spalten hat:

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── BrowserActivity/
│   │   ├── Browser-Cookies.csv
│   │   ├── Browser-Downloads.csv
│   │   ├── Browser-FormHistory.csv
│   │   ├── Browser-History.csv
│   │   ├── Browser-logs.txt
│   │   ├── WebCache-Containers.csv
│   │   ├── WebCache-Cookies.csv
│   │   ├── WebCache-Downloads.csv
│   │   ├── WebCache-Entries.csv
│   │   ├── WebCache-History.csv
│   │   └── WebCache-logs.txt
│   ├── Communication/
│   │   ├── Outlook-Attachments.csv
│   │   ├── Outlook-Calendar.csv
│   │   ├── Outlook-Contacts.csv
│   │   ├── Outlook-Files.csv
│   │   ├── Outlook-Folders.csv
│   │   ├── Outlook-Items.csv
│   │   ├── Outlook-Recipients.csv
│   │   ├── Outlook-Tasks.csv
│   │   └── Outlook-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── RecycleBin.csv
│   │   ├── RecycleBin-logs.txt
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   ├── Registry-RemoteAccess-logs.txt
│   │   ├── UAL-Clients.csv
│   │   ├── UAL-RoleAccess.csv
│   │   ├── UAL-SystemIdentity.csv
│   │   └── UAL-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```

| Kategorie | Artefakte |
|---|---|
| `Accounts` | Lokale SAM-Konten, Namen und Zeitstempel der LSA-Secrets |
| `Applications` | Anwendungsverlauf (Spuren von Drittanbieterwerkzeugen) |
| `Applications` | Installierte Programme |
| `BrowserActivity` | Webbrowser-Verlauf, Downloads, Cookies und Formularverlauf, mit wiederhergestellten gelöschten Einträgen; WebCache von Internet Explorer und dem alten Edge |
| `Communication` | PST-, OST-, MSG- und OFT-Dateien von Outlook: Ordner, Elemente, Empfänger, Anhänge, Kontakte, Kalenderelemente und Aufgaben |
| `ExternalDevices` | USB-Geräte und -Volumes |
| `FileAndFolderOpening` | Jump Lists, LNK-Verknüpfungen, Microsoft Office, MRU-Listen, Dialoge zum Öffnen und Speichern unter, RecentDocs, ShellBags |
| `FileExistence` | Amcache, Papierkorb, ShimCache |
| `FileSystem` | `$Boot`, `$I30`, `$LogFile`, `$MFT`, `$Secure:$SDS`, USN-Journal |
| `NetworkActivity` | Netzwerkprofile, Schnittstellen, Freigaben, Portproxys und Routen; SRUM-Verbindungen, -Datenverkehr und -Push-Benachrichtigungen |
| `LateralMovement` | Ausgehende Remotedesktop-, PuTTY- und WinSCP-Verbindungen der Benutzer, zugeordnete Laufwerke und Freigaben; Benutzerzugriffsprotokollierung (die Clients der Rollen eines Servers) |
| `Persistence` | Startkonfiguration (BCD), Registry-Autostarts, Dienste, TaskCache |
| `ProcessExecution` | Anwendungskompatibilität, BAM und DAM, FeatureUsage und angeheftete Elemente, MuiCache, Prefetch, Prefetch-Zeitleiste, SRUM-Anwendungs-/CPU-Aktivität, UserAssist |
| `Registry` | Allgemeine Hive-Schlüssel und -Werte, einschließlich wiederhergestellter Datensätze |
| `SecurityProducts` | Metadaten der Defender-Quarantäne und optional extrahierte Inhalte; Sicherheitseinstellungen (Defender, UAC, Schutz von Anmeldeinformationen, RDP, Firewall, Protokollierung) |
| `SystemInformation` | Namen, Windows-Version, Zeitzone, letztes Herunterfahren, Profile, SIDs und Überwachungsrichtlinie; SRUM-Energie/-Kapazität, Identitätszuordnungen und unbekannte Provider |

Kategorien helfen bei der Organisation von Untersuchungen; sie beweisen keine bestimmte Aktivität. So
belegen beispielsweise Einträge in Amcache und ShimCache allein keine Ausführung. SRUM zeichnet Ressourcennutzung auf;
ziehen Sie den Artefaktleitfaden heran, bevor Sie Schlussfolgerungen über die Ausführung eines Programms ziehen.

CSV, JSON und JSONL verwenden dieselben Kategorien. Eine Datei `<artifact>-logs.txt` pro Parser-Job enthält dessen
exakten Befehl und seine Meldungen, zum Beispiel `HOST1/FileSystem/MFT-logs.txt` und
`HOST1/ProcessExecution/Prefetch-logs.txt`. JSON und JSONL verwenden dieselben Protokolldateinamen.
Amcache und Jump Lists haben jeweils ein gemeinsames Protokoll neben ihren Tabellen mit Präfix. SRUM-Provider-Tabellen liegen direkt
in `NetworkActivity`, `ProcessExecution` und `SystemInformation`, mit einer gemeinsamen
Datei `SystemInformation/SRUM-logs.txt`. Es werden keine Unterverzeichnisse `logs/` angelegt.

Die standardmäßigen CSV-Dateinamen sind `Registry-SAM.csv`, `LNK.csv`, `RecycleBin.csv`, `Registry-RecentDocs.csv`, `Registry-ShimCache.csv`, `LogFile.csv`,
`I30.csv`, `MFT.csv`, `UsnJournal.csv`, `Boot.csv`, `SecureSDS.csv`, `Registry-Networks.csv`, `Registry-Persistence.csv`, `Registry-Services.csv`, `Registry-TaskCache.csv`, `Registry-BCD.csv`,
`Registry-SecuritySettings.csv`,
`PrefetchTimeline.csv`, `Prefetch.csv`, `Registry-UserAssist.csv`, `Registry-AppCompat.csv`, `Registry-Office.csv`, `Registry-FeatureUsage.csv`, `Registry-AppHistory.csv`, `Registry-MRU.csv`, `Registry-SystemInfo.csv`, `Registry-RemoteAccess.csv`, `Registry-OpenSaveMRU.csv`, `Registry-InstalledPrograms.csv`, `Registry-BAM.csv`, `Registry-MuiCache.csv`, `Registry-ShellBags.csv`, `Registry-USB.csv`, `Registry-SecuritySettings.csv`, `Registry-AuditPolicy.csv`, `Registry-Dump.csv` und
`DefenderQuarantine.csv`. Aus der Registry abgeleitete Tabellen verwenden das Präfix `Registry-`, darunter
`Registry-ShimCache.csv`, `Registry-UserAssist.csv`, `Registry-BAM.csv` und `Registry-SystemInfo.csv`.
Allgemeine Hive-Schlüssel und -Werte stehen in `Registry-Dump.csv`. Die Amcache-Tabellen sind
`FileExistence/Registry-Amcache-Devices.csv`, `Registry-Amcache-Files.csv`,
`Registry-Amcache-Findings.csv` und `Registry-Amcache-Summary.csv`. Jump Lists verwenden
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` und
`JumpLists-CustomDestinations.csv`. Es gibt keine Unterverzeichnisse für Amcache oder Jump Lists.
SRUM-Tabellen verwenden `SRUM-`-Dateinamen direkt unter ihrer Kategorie, zum Beispiel
`NetworkActivity/SRUM-NetworkConnections.csv`.
Die Tabellen der Benutzerzugriffsprotokollierung liegen in `LateralMovement`, eine Datei pro Tabelle, etwa
`UAL-Clients.csv` und `UAL-SystemIdentity.csv`, zusammen mit `UAL-logs.txt`; siehe
[Benutzerzugriffsprotokollierung (UAL)](../artifacts/ual.md#automatic-parsing).
Die Tabellen der Webbrowser liegen in `BrowserActivity`: `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv` und `Browser-FormHistory.csv`, zusammen mit `Browser-logs.txt`; siehe
[Webbrowser](../artifacts/browser.md#automatic-parsing). Die WebCache-Tabellen von Internet Explorer und dem alten
Edge liegen ebenfalls dort: `WebCache-History.csv`, `WebCache-Downloads.csv`,
`WebCache-Cookies.csv`, `WebCache-Entries.csv` und `WebCache-Containers.csv`, zusammen mit
`WebCache-logs.txt`; siehe [Internet Explorer und Edge Legacy](../artifacts/webcache.md#automatic-parsing).
Die Outlook-Tabellen liegen in `Communication`: `Outlook-Files.csv`, `Outlook-Folders.csv`,
`Outlook-Items.csv`, `Outlook-Recipients.csv`, `Outlook-Attachments.csv`, `Outlook-Contacts.csv`,
`Outlook-Calendar.csv` und `Outlook-Tasks.csv`, zusammen mit `Outlook-logs.txt`; siehe
[Outlook](../artifacts/outlook.md#automatic-parsing).
Siehe die [vollständige Tabelle zur Ablage der SRUM-Tabellen](../artifacts/srum.md#categorized-auto-parse-output).

Protokolle entsprechen dem großgeschriebenen Artefaktnamen, etwa `Registry-SAM-logs.txt`, `PrefetchTimeline-logs.txt`
und `Registry-UserAssist-logs.txt`. JSON-/JSONL-Dateien verwenden dieselben Namensstämme, zum Beispiel `MFT.jsonl`; der
`Artifact`-Bezeichner des Datensatzes lautet für Abfragen `mft`. Ausgaben mit mehreren Tabellen haben
`Registry-Amcache-logs.txt` bzw. `JumpLists-logs.txt` neben ihren Tabellen. Das gemeinsame SRUM-Protokoll liegt
in `SystemInformation`.


`manifest.json` hält den ursprünglich erkannten `Hostname` fest, die eindeutige Ausgabe-/Datensatzidentität `Comp`,
die Herkunft des Hostnamens, Beweispfade, die Artefaktkategorie `Category`, Befehlszeilen, Dauer, Exit-Code,
Ergebnispfad, Protokollpfad, die angeforderte JSONL-Zeitleiste und etwaige Exportfehler, ob eine Defender-Extraktion angefordert wurde,
und ob jede Aufgabe
erfolgreich war, übersprungen wurde oder fehlgeschlagen ist.

Auto-parse schreibt keine absoluten Pfade, sodass Ergebnisse weitergegeben werden können, ohne preiszugeben, wo die Beweise,
die Ergebnisse oder Juntei auf dem Analysesystem liegen:

- Beweispfade sind relativ zum Beweisverzeichnis. Das Terminal listet jeden Computer mit seinem
  Verzeichnis in den Beweisen auf (`HOST1: ./HOST1_F`), und die Parser laufen im Beweisverzeichnis und
  erhalten solche Pfade. Die Quellpfadspalten der Ergebnisse (`SourceFile`, `SourcePath`, …), die Protokolle
  und `EvidenceRoot` sowie `WindowsDirectories` in den Manifesten zeigen, wo sich jede Datei in den
  Beweisen befindet (`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`).
- Die Ergebnispfade der Manifeste und Protokolle (`Output`, `OutputFiles`, `Log`, `JSON-Timeline`, die
  Ausgabepfade in `Command`) sowie `ExtractedFile` von Defender sind relativ zum Ausgabeverzeichnis
  (`./HOST1/ProcessExecution/Prefetch.csv`, `../siem.jsonl` für eine daneben liegende Zeitleiste). `Command`
  beginnt mit dem Namen des Programms (`juntei`); seine Beweispfade sind relativ zum Beweisverzeichnis
  und seine Ausgabepfade relativ zum Ausgabeverzeichnis.
- `EvidenceDirectory` und `OutputDirectory` sind die Namen der beiden Verzeichnisse. `ManifestVersion` 2
  kennzeichnet Manifeste, die Ergebnispfade ausgehend vom Ausgabeverzeichnis festhalten; frühere Manifeste haben kein
  `ManifestVersion` und halten sie so fest, wie sie an `-o` übergeben wurden. Ein erneuter Lauf mit `-C` liest beide.
- Das Terminal zeigt einen Ausgabe- oder Zeitleistenpfad so an, wie er angegeben wurde, wenn er relativ ist, und ausgehend vom
  aktuellen Verzeichnis, wenn er absolut ist (`../results/`).

Nach der Anzahl paralleler Prozesse zeigt das Terminal die Gesamtgröße der Dateien der
gefundenen Windows-Systeme an (`Total filesize: 26.1 MiB`).

Die Fortschrittsmeldung zeigt den aktiven Computer und das aktive Artefakt, gefolgt von einer ungefähren Eingabegröße,
zum Beispiel `HOST1 / prefetch (~2.0 MiB)`. Eine Gruppe von Dateien verwendet ihre Gesamtgröße; jede
Aufgabe für MFT, LogFile oder USN-Journal zeigt ihre individuelle Eingabegröße. Die Schätzungen schließen zusätzliche Transaktionsprotokolle
und Anreicherungsdateien aus und sagen nicht die Größe der geparsten Ausgabe voraus.

`manifest.csv` stellt denselben Aufgabenindex als tabellenkalkulationsfreundliche Tabelle bereit, mit einer Zeile pro
Parser-Aufgabe. Sie enthält `Comp`, den ursprünglichen `Hostname`, `FQDN`, `HostnameSource`, `EvidenceRoot`,
`Artifact`, `Category`, `OutputType`, `Purpose`, `Status`, `DurationMs`, `ExitCode`, `Output`, `OutputFiles`, `Log`,
`Error` und `Command`. `Purpose` ist `artifact` für die kategorisierten Ergebnisse oder `json_timeline` für
den zusätzlichen nativen JSONL-Durchlauf bei CSV-Ausgabe. `OutputType` ist das tatsächliche Ausgabeformat des Parsers.
`Command` ist ein JSON-Array aus der ausführbaren Datei und ihren Argumenten, wobei Leerzeichen und Anführungszeichen erhalten bleiben.
Für SRUM ist `Output` das Computerverzeichnis, `OutputFiles` ist ein JSON-Array der exakten
Ausgabepfade der Provider, und `Category` nennt den gemeinsamen Protokollspeicherort (`SystemInformation`).
Dateien werden einzeln kategorisiert; `OutputFiles` erscheint auch im JSON-Manifest.

Filtern Sie `Status` nach `skipped` oder `failed`, um Aufgaben zu finden, die geprüft werden müssen; der `Error` einer übersprungenen Aufgabe
kann leer sein, wobei die Erklärung in ihrer `Log`-Datei steht.

Die Zusammenfassung im Terminal enthält die Gesamtzahl und -größe der regulären Dateien, die sich aktuell unterhalb des
Ausgabeverzeichnisses befinden, einschließlich beider Manifeste, der Parser-Protokolle und extrahierter Dateien. Symbolischen Links wird
nicht gefolgt. Die Größen werden automatisch mit demselben Formatierer wie `Total filesize` skaliert, mit
Einheiten wie `KiB`, `MiB` und `GiB`:

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

Amcache- und SRUM-Tabellen verwenden `-Source1`, `-Source2` usw., wenn mehrere Quelldatenbanken
gesichert wurden. Das gemeinsame Parser-Protokoll hält die Zuordnung von ursprünglichen zu endgültigen Pfaden fest, und JSON-/JSONL-Datensätze
behalten ihre Herkunftsangaben. CSV-Aufgaben für Amcache/Jump Lists listen ihre exakten Tabellenpfade in
`OutputFiles` des Manifests auf; ihr `Output` ist das Verzeichnis der Untersuchungskategorie. JSON- und JSONL-Exporte
von Amcache und Jump Lists sind bereits Einzeldateien namens `Registry-Amcache.json[l]` und
`JumpLists.json[l]`. Die kombinierte Zeitleiste `-j` behält dieselben Datensatzinhalte und `Artifact`-Werte bei.
`--clobber` verwendet das vorherige Manifest, um aufgezeichnete Ausgaben einschließlich früherer Namen zu entfernen,
und bewahrt dabei nicht zugehörige Kategoriedateien. Eine normale Sammlung mit einem Volume schreibt `$MFT`- und USN-Journal-Datensätze direkt nach
`FileSystem/MFT.csv` und `FileSystem/UsnJournal.csv`. Weitere gesicherte Volumes verwenden nummerierte
Geschwisterdateien wie `FileSystem/MFT-2.csv` und `FileSystem/MFT-2-logs.txt`.

## JSON- und SIEM-Import { #json-and-siem-ingestion }

Jeder JSON- oder JSONL-Datensatz erhält zwei Felder auf oberster Ebene:

- `Comp`: das Quellsystem.
- `Artifact`: der Juntei-Parser, der den Datensatz erzeugt hat, etwa `mft` oder `prefetch-timeline`.

`-j, --json-timeline <FILE>` erzeugt eine zusätzliche JSONL-Datei, unabhängig vom Format der einzelnen
Ergebnisse. Alle kategorisierten Ausgaben und ihre Protokolle bleiben erhalten:

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

Der Dateiname wird so verwendet, wie er angegeben wurde; relative Pfade beziehen sich auf das aktuelle Arbeitsverzeichnis.
Die Datei kann im Stammverzeichnis der Ausgabe, in einem anderen nicht reservierten Unterverzeichnis der Ausgabe oder außerhalb des Ausgabeverzeichnisses
liegen. Beweise, Manifeste, Ergebnisse/Protokolle pro Computer und temporäre Parser-Verzeichnisse können nicht
als Ziel verwendet werden. Eine vorhandene Datei erfordert `-C`; ein fehlgeschlagener Export lässt sie unverändert.

Im **CSV-Modus** führt Juntei einen zusätzlichen nativen JSONL-Durchlauf aus, damit die SIEM-Datei
numerische und boolesche Typen, verschachtelte Daten, Rohfelder und Metadaten bewahrt, die CSV-Tabellen auslassen können. Die
zusätzlichen Jobs teilen sich dasselbe Parallelitätslimit `-T`, extrahieren Quarantäneinhalte nicht erneut und
haben separate begleitende Protokolle `*-json-timeline-logs.txt`. Beide Manifeste unterscheiden ihren Zweck und ihr
Ausgabeformat. Temporäre JSONL-Dateien werden nach einem erfolgreichen Lauf entfernt; sie bleiben erhalten, wenn ein
Parser oder der Export fehlschlägt. Planen Sie die zusätzliche Verarbeitungszeit und den temporären Speicherplatz ein.

Im **JSON-Modus** streamt Juntei die vorhandenen Ergebnisse in kompaktes JSONL, ohne die Beweise
erneut zu parsen. Amcache-Berichte werden in dieselben einzelnen Zusammenfassungs-, Befund-, Datei- und Gerätedatensätze
wie bei nativem JSONL aufgeteilt. Im **JSONL-Modus** werden die vorhandenen annotierten Datensätze direkt zusammengeführt.
Alle Modi verwenden dieselbe SIEM-Datensatzstruktur. Der Datensatz behält native Zeitstempel und Artefaktfelder bei;
er erfindet keinen gemeinsamen Ereigniszeitstempel und sortiert heterogene Artefakte nicht global.

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

`manifest.json` hält den angeforderten Pfad `JSON-Timeline` und etwaige `JSON-TimelineError` fest. Seine Aufgabendatensätze
und `manifest.csv` verweisen weiterhin auf die beibehaltenen Artefaktausgaben. Erfolgreiche CSV-Jobs, die nur der Zeitleiste dienen,
verweisen auf die endgültige JSONL-Datei statt auf gelöschte temporäre Dateien. Schlägt ein Parser fehl, endet der
Lauf erfolglos, und die Zeitleiste enthält nur erfolgreiche Ergebnisse; prüfen Sie die Manifeste
und Kategorieprotokolle, bevor Sie sie als vollständig betrachten. Die Zusammenfassung von Dateianzahl/-größe umfasst Dateien innerhalb
des Ausgabeverzeichnisses; eine externe Zeitleiste wird separat als `JSONL timeline: <FILE>` gemeldet.

## Leistung { #performance }

Parser-Jobs sind separate Prozesse, sodass CPU-intensive Artefakte gleichzeitig laufen können und ein Schaden in
einem Parser nicht den gesamten Lauf beenden kann. Standardmäßig verwendet Juntei die Anzahl der logischen CPUs. Begrenzen Sie
die Parallelität, wenn die Beweise auf einem langsamen Datenträger liegen oder das System wenig Arbeitsspeicher hat:

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## Verwendung { #usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]
      --vhdx                  Parse VHDX images natively on Linux, macOS and Windows (read-only; temporary disk space required)
      --e01                   Parse E01/Ex01 images natively (split images from their first segment; temporary disk space required)
      --temp-dir <DIR>        Directory for private image staging (default: output filesystem)

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -Z, --timezone <OFFSET>             Display parsed timestamps using a fixed UTC offset (UTC by default) [default: +00:00]
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files
  -R, --reconstruct-filesystem        Reconstruct filesystem trees from MFT and I30

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK files, Jump Lists, ShellBags and OpenSaveMRU [default: 1252]
      --mft-slack           Recover the file names left in the slack of $MFT records

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Verwenden Sie nach Möglichkeit ein neues Ausgabeverzeichnis. `-C` entfernt oder überschreibt nur die exakten erzeugten
Ziele dieses Laufs, sodass ein inzwischen fehlendes Artefakt kein veraltetes Ergebnis hinterlassen kann. Es löscht keine
nicht zugehörigen Dateien und nicht den gesamten Ausgabebaum. Außerdem entfernt es vor dem Parsen das Verzeichnis
`SecurityProducts/DefenderQuarantined/` jedes Computers sowie das reservierte Verzeichnis `defender-quarantined/`
aus der früheren flachen Struktur, unabhängig davon, ob der neue Lauf `-x` verwendet.

Verwenden Sie beim Wechsel von einer älteren flachen Struktur ein neues Ausgabeverzeichnis. Frühere CSV-/JSON-Dateien
werden nicht automatisch verschoben oder gelöscht.

!!! danger "Extrahierte Defender-Inhalte als Malware behandeln"
    `-x, --extract-defender-quarantined` schreibt Quarantäneinhalte unterhalb des Verzeichnisses
    `SecurityProducts/DefenderQuarantined/` jedes Computers auf den Datenträger. Wiederhergestellte Binär-Streams enden auf `.quarantined`, und Juntei
    entfernt die Ausführungsberechtigung. Jede binäre Sicherheitsbeschreibung erhält zudem einen lesbaren Bericht `.security_descriptor.txt`
    mit Besitzer, Berechtigungen und SDDL. Führen Sie wiederhergestellte Payloads niemals aus. Verwenden Sie ein isoliertes
    Analysesystem und rechnen Sie damit, dass Antiviren- oder EDR-Software bei den wiederhergestellten Inhalten Alarm schlägt oder sie entfernt.

Unter [Lesen der wiederhergestellten Sicherheitsbeschreibung](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)
finden Sie die Berichtsfelder und ihre forensischen Grenzen.

Auto-parse liest außerdem die Remoteverbindungen der Benutzer (RDP, PuTTY, WinSCP und zugeordnete Laufwerke).

## Chronologische Gesamttimeline erstellen { #build-a-chronological-super-timeline }

Nach dem Parsen erweitert [`super-timeline`](super-timeline.md) die gespeicherten Zeitstempel zu beschrifteten Ereignissen. Mit `--sort` werden sie über Rechner und Artefakte hinweg sortiert:

```bash
juntei super-timeline -d output/ -o SuperTimeline.csv --sort
```

Der bestehende Export `-j, --json-timeline` bleibt eine Datei mit nativen Datensätzen im Format JSONL. Er erhält die Parser-Schemas, erweitert oder sortiert aber keine Zeitstempel. Für die Beweisprüfung sollten auch diese Datei und die einzelnen Parser-Ergebnisse aufbewahrt werden.
