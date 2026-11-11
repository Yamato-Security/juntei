# Gesamttimeline { #super-timeline }

`super-timeline` überführt geparste Artefakte in ein gemeinsames Ereignisschema. Die Ausgabe ist **standardmäßig unsortiert**: Sie folgt der Reihenfolge der gefundenen Eingabedateien und der Projektionsreihenfolge des jeweiligen Parsers, unabhängig davon, wann die einzelnen Worker fertig werden. `--sort` sortiert chronologisch und führt Duplikate zusammen. Mehrere Ausführungszeiten eines Prefetch-Datensatzes oder acht und mehr Dateizeiten eines MFT-Datensatzes werden einzeln beschriftet. So bleiben beim Quellenvergleich die unterschiedlichen Zeitbedeutungen erhalten.

Verwenden Sie die Ergebnisse von [`auto-parse`](auto-parse.md) statt des ursprünglichen Beweisverzeichnisses:

```bash
juntei auto-parse -d evidence/ -o parsed/
juntei super-timeline -d parsed/ -o SuperTimeline.csv
```

Bewahren Sie Parser-Ergebnisse, Manifeste und Originalbeweise auf. Die Timeline ist eine Analyseprojektion; sie ersetzt diese Daten nicht und erstellt keinen nativen Plaso-Datenspeicher.

## Verwendung { #usage }

```bash
# Native CSV (default)
juntei super-timeline -d parsed/ -o SuperTimeline.csv

# Chronological CSV with duplicate consolidation
juntei super-timeline -d parsed/ -o SortedTimeline.csv --sort

# Native JSON Lines
juntei super-timeline -d parsed/ -o SuperTimeline.jsonl -t jsonl

# Combine native records and an existing Hayabusa JSONL timeline
juntei super-timeline -f NativeRecords.jsonl --hayabusa Hayabusa.jsonl \
  -o SuperTimeline.jsonl -t jsonl

# Timesketch import profile; also available with CSV
juntei super-timeline -d parsed/ -o Timesketch.jsonl -t jsonl --profile timesketch

# One standalone CSV needs an artifact identifier and, if unavailable, a computer name
juntei super-timeline -f MFT.csv --input-artifact mft --computer HOST01 \
  -o Host01Timeline.csv

# Reduce the time range, memory used by sorting, or number of workers
juntei super-timeline -d parsed/ -o IncidentTimeline.jsonl -t jsonl \
  --timeline-start "2026-10-20T06:00:00Z" --timeline-end "2026-10-20T14:00:00Z" \
  --sort --memory-limit 64 -T 4 --temp-dir scratch/
```

`--hayabusa FILE` ist mehrfach oder allein nutzbar und importiert vorhandene Ereignis-/Alarmzeiten samt Regelkontext, ohne Hayabusa auszuführen. Informationseinträge mit `Details` als Zeichenfolge werden unterstützt. `--artifact mft,prefetch` beschränkt die Artefakte. Zeitgrenzen sind einschließlich und benötigen einen expliziten UTC-Offset oder `Z`.

### Eingabe auswählen { #choosing-an-input }

| Eingabe | Erkennung |
|---|---|
| `-d DIR` | `manifest.json` liefert erfolgreiche primäre Aufgaben, Artefakt-IDs, Rechner und genaue Ausgabedateien. Sekundäre `auto-parse -j`-Aufgaben werden gegen Doppelimporte ausgeschlossen. |
| `-f FILE` | Natives JSON/JSONL nutzt eingebettete Felder `Artifact` und `Comp`. Unmarkierte Exporte benötigen `--input-artifact`; einzelne CSV-Dateien benötigen es immer. |
| `--computer NAME` | Ersatz bei fehlendem Rechnernamen im Datensatz; unbekannte Rechner heißen ausdrücklich `Unknown`. |
| `--hayabusa FILE` | JSONL mit Hayabusa-Zeiten, Regelmetadaten, Rechner und gegebenenfalls `EvtxFile`. |

Aktuelle Manifeste erlauben das Verschieben des Ergebnisverzeichnisses und lösen gespeicherte absolute Ausgabepfade dort neu auf. Eingaben einschließlich Symlink-Zielen müssen darin bleiben. Ältere Artefaktverzeichnisse für Amcache, SRUM und Jump Lists werden unterstützt; Kategorieverzeichnisse benötigen explizite `OutputFiles`.

Unterstützt werden aktuelle Manifeste mit ausgaberelativen Pfaden und flach abgelegten Mehrtabellen-Exporten sowie sämtliche von der automatischen Analyse eingeplanten Artefakte, einschließlich Browser, WebCache, Outlook und UAL.

Bei VHDX-/E01-Abbildern enthalten Quellenangaben das Abbild und den internen Pfad; der Abbildname bleibt im Ereigniskontext. Zum Lesen der Ergebnisse ist der temporäre Bereitstellungsbereich nicht erforderlich.

Eine Rohquelle wird aus dem Datensatz oder einem eindeutigen Manifestbefehl entnommen. Bei `-d` sind Rohquellenverweise relativ zum Beweisverzeichnis und `Details.ParsedFile` relativ zum analysierten Verzeichnis. SRUM CSV-Tabellen werden einer Datenbank nur dann zugewiesen, wenn der Host über eine eindeutig identifizierbare gesammelte SRUDB.dat verfügt. Andernfalls verweist `SourceFile` auf die analysierte Datei und `Details.RawSourceAvailable` ist `false`.

Auf eine eigenständige Eingabe wird durch ihren Dateinamen verwiesen. Eingaben mit demselben Dateinamen verwenden unterschiedliche `./Inputs/<ID>/<filename>`-Referenzen. Absolute Rohquellenpfade außerhalb bekannter Beweiswurzeln verwenden `./Sources/<ID>/<filename>`-Bezeichner. Diese undurchsichtigen Referenzen behalten die Quellenunterscheidung bei, ohne die absoluten Verzeichnisnamen des Analysten zu exportieren. Konsultieren Sie den nativen Datensatz mit `ParsedFile` und `InputRecord` für die Originalquelle. In den Beweisen aufgezeichnete Pfade, wie z. B. die ausführbaren Pfade Windows, bleiben unverändert.

## Gemeinsame Ereignisfelder { #common-event-fields }

| Feld | Bedeutung |
|---|---|
| `Timestamp` | UTC RFC 3339 mit neun Nachkommastellen für stabile Sortierung. Aufgefüllte Nullen erhöhen die Genauigkeit nicht. |
| `Comp` | Rechner des Eingabedatensatzes. |
| `Artifact` | Stabile Artefaktkennung wie `mft`, `prefetch`, `srum` oder `hayabusa`. |
| `TimestampType` | Bedeutung des Zeitstempels, etwa `MFT SI Data modified`. |
| `User` | Gespeichertes Konto oder SID, sofern verfügbar; kann leer sein. |
| `Path` | Betroffene Datei, Registrierungsschlüssel oder anderes artefaktspezifisches Objekt. |
| `Message` | Kurzbeschreibung für Timeline-Betrachter. |
| `SourceFile` | Beweisrelativer Rohquellenverweis oder ein relativer Quellenbezeichner, wenn dessen Wurzel unbekannt ist; andernfalls der oben beschriebene Fallback für geparste Dateien. |
| `SourceField` | Tatsächlich verwendetes Parser-Feld oder verschachtelte Position des Zeitstempels. |
| `RecordID` | Artefaktspezifische Datensatzkennung; nicht zwingend eine EVTX-Datensatz-ID. |
| `Details` | Strukturierter Kontext: Originalzeit, Genauigkeit, Zeitbasis/-rolle, geparste Datei, Eingabedatensatz/-position und verfügbare Wiederherstellungs-/Regelmetadaten. |

In CSV ist `Details` eine zitierte JSON-Zelle, in JSONL ein Objekt. Das Timesketch-Profil benennt `Timestamp`, `TimestampType` und `Message` in `datetime`, `timestamp_desc` und `message` um; die übrigen Herkunftsfelder bleiben erhalten. Dies sind die [drei erforderlichen Timesketch-Importfelder](https://timesketch.org/guides/user/import-from-json-csv/).

`Details` enthält ausgewählten Kontext statt vollständiger Binärdaten oder sämtlicher nativer Felder. Lange Zeichenfolgen und Strukturen werden begrenzt und mit `ProjectionOmitted` markiert. `Details.ParsedFile`, `InputRecord` und `Location` führen zum vollständigen Datensatz. Bei CSV zählt die Nummer CSV-Datensätze nach der Kopfzeile, keine physischen Zeilen in Zellen; bei JSONL Zeilen und bei JSON Dokumentnummer plus verschachtelte Position.

## Bedeutung der Zeitangaben { #what-the-times-mean }

Ein Zeitstempel beschreibt ein gespeichertes Objekt und beweist allein weder Ausführung noch Angriff. Die Adapter unterscheiden deshalb:

| Artefakte | Interpretation in der Timeline |
|---|---|
| MFT | Getrennte SI-/FN-Zeiten für Änderung, Zugriff, Metadatenänderung und Erstellung; benannte FN-Attribute bleiben unterscheidbar. Mit `--sort` werden wiederholte SI-Felder zusammengeführt; gemeinsame Basisdateizeiten von ADS-Zeilen entfallen in beiden Modi. Wiederhergestellte Namen aus FILE-Datensatz-Slack erhalten eigene wiederhergestellte FN-Ereignisse. |
| I30 | Gespeicherte FN-/Indexmetadaten mit verfügbarem Slack-/Wiederherstellungskontext, keine Zeitpunkte des Schreibens des Verzeichnisindexes. |
| USN-Journal | Journalzeit und Änderungsgründe; ein Datensatz kann mehrere Änderungen beschreiben, ohne deren exakten Beginn festzuhalten. |
| LogFile | Gespeicherte Redo-/Undo-Zeitschnappschüsse und Dateinamenmetadaten bleiben getrennt. LSNs und Dateioffsets sind Kennungen, keine Datumswerte. |
| Prefetch | Gespeicherte Ausführungszeiten; mit `--sort` werden sie mit dem passenden Prefetch-Timeline-Export zusammengeführt. Volume-Erstellungszeiten werden gesondert beschriftet. |
| SQLite-Browserdatenbanken | Besuche, wiederhergestellte letzte Seitenbesuche, Downloadbeginn/-ende, Cookie-Erstellung/-Zugriff/-Ablauf und Formularnutzung/-löschung. Downloadende beweist keinen Erfolg oder Start, Cookie-Ablauf ist eine konfigurierte Frist. Wiederherstellungsstatus, Sicherheit und Herkunft bleiben erhalten. |
| Internet Explorer / älteres Edge WebCache | Gespeicherte Zugriffs-, Erstellungs-, Änderungs-, Ablauf- und Synchronisationszeiten sowie Download-, Cookie- und Containerverwaltung. Cache-Verwaltung beweist nicht automatisch einen Seitenaufruf durch den Benutzer. |
| Outlook PST, OST, MSG und OFT | Übermittlung/Zustellung, Erstellung/Änderung von Ordnern, Elementen und Anlagen; Kalender- und Aufgabenfristen, Erinnerungen, Abschlüsse sowie explizit typisierte Zeitwerte. Geplante Zeiten beweisen weder Besprechung noch Erinnerung oder Programmausführung. Zustellung beweist kein Lesen. |
| Benutzerzugriffsprotokollierung (UAL) | Metadaten des ersten/letzten Zugriffs, Einfügung/Erstellung der Datenbank, erste/letzte gesehene und gespeicherte Startzeit des Betriebssystems. Rolle, Konto, Quelladresse und Tageszähler bleiben im Kontext. Tägliche Termine/Zählungen sind keine erfundenen Mitternachtsanmeldungen; Die Zeilen „Identität ohne Zeitangabe“ und „DNS“ verbleiben in den ursprünglich analysierten Ergebnissen. |
| Amcache | Inventarschlüssel-Schreibzeiten, übergeordneter Volume-Schlüssel, Installationsdaten und PE-Linkmetadaten. Inventareintrag oder LinkDate allein beweist keine Ausführung. |
| ShimCache | Gespeicherte Dateiänderung, Cache-Aktualisierung und Schlüssel-Schreibzeit sind nicht automatisch Ausführungszeiten. |
| SRUM | Anbieterzeit/-intervall, Verbindungsbeginn, explizite Beginn-/Endfelder und Anwendungskennungen. Bytes, Dauer und Zähler sind keine Datumswerte. |
| LNK | MAC-Zeiten des Ziels, Tracking-Kennungserstellung und Shell-Item-Zeiten beschreiben Zielmetadaten, nicht zwingend das Öffnen der Verknüpfung. |
| Jump Lists | Letzte Eintragsnutzung, Tracking-Kennung und eingebettete LNK-Zielzeiten. Containerdateizeiten auf der Sammelseite sind standardmäßig ausgenommen. |
| Defender-Quarantäne | Eintragszeit und gespeicherte Ressourcen-Erstellungs-/Zugriffs-/Schreibzeiten. Extraktionszeiten beweisen keine Aktivität am Endpunkt. |
| BAM, UserAssist, FeatureUsage | Gespeicherte Aktivität mit getrenntem Schlüssel-Schreibkontext. Zähler ohne Zeit erzeugen kein Ereignis. |
| RecentDocs, MRU, Öffnen/Speichern-MRU, ShellBags, Office | Gespeicherte Nutzungs-/Vertrauens-/Shell-Item-Zeiten und Schlüsselkontext. Die Schlüsselzeit einer Liste ist nicht die letzte Nutzung jedes einzelnen Elements. |
| SAM und LSA-Geheimnisse | Kontoanmeldung, Kennwort-/Ablauf-/Geheimnisänderungsmetadaten und Schlüsselzeiten ersetzen keine Authentifizierungsereignisprotokolle. |
| Dienste, TaskCache, Registrierungspersistenz, BCD | Schlüssel-/Konfigurationsmetadaten und ausdrücklich gespeicherte Aufgabenzeiten. Dienstschlüssel-Schreibvorgänge beweisen keine Dienstausführung. |
| Netzwerke und USB | Netzwerkprofile/-Leases, Geräteinstallation/-ankunft/-entfernung und Schlüsselkontext. Native USB-Benutzer-MountPoints2-Zeiten behalten ihre Benutzer-Hive-Quelle. |
| Systeminformationen, installierte Programme, AppCompat, Anwendungsverlauf | Explizite Installations-, Herunterfahr-, Inventar-/Aktivitäts- und Schlüsselmetadaten. Native Systeminformations-Arrays behalten verschachtelte Profil-/Versionszeiten. |
| Überwachungsrichtlinien, Sicherheitseinstellungen, MuiCache, allgemeine Registrierung | Schlüssel-Schreibzeiten, sofern vorhanden; zeitlose Werte/Zähler bleiben im nativen Eingang. |
| Persistence Hunter | Schlüssel-Schreib- und korrelierte Änderungszeiten. Ein Fund beweist allein weder zeitlich bestimmte Installation noch Kompromittierung. |
| Hayabusa | Ereignis-/Alarmzeit mit Regel, Schweregrad, Kanal, Ereignis-ID und Kontext. Verschiedene Regeln und Korrelationszeilen bleiben getrennt. |

Der Standardexport enthält alle Zeitstempelprojektionen, auch wiederholte Metadaten. Er sortiert nicht global und führt keine Duplikate zusammen. Mit `--sort` werden doppelte Projektionen zusammengeführt; die Ausgabe kann dadurch weniger Zeilen haben als ein unsortierter Export derselben Eingaben. Gleiche Zeiten mit unterschiedlicher Bedeutung oder Herkunft bleiben in beiden Modi erhalten. Die Duplikatidentität berücksichtigt Rechner, Quelle, Artefakt, Datensatz, Zeitrolle und Wiederherstellungszustand. Zusammengeführte Zeilen enthalten `DuplicateCount` und begrenzte Alternativpfade/-felder. Ähnlichkeit allein führt nie zum Zusammenführen verschiedener Quellen. Bei unbekannter Rohquelle verhindert die Eingabedatensatzkennung das Vermischen verschiedener Erfassungen.

### Genauigkeit, Ortszeiten und Annahmen { #precision-local-times-and-assumptions }

Bekannte UTC-Felder und explizite Offsets werden auf UTC normalisiert. Originalwerte und Genauigkeit einschließlich 100-ns-FILETIMEs bleiben erhalten; bereits weggerundete Genauigkeit kann nicht wiederhergestellt werden.

Einige Shell-Item-/FAT-Felder bedeuten Ortszeit, selbst wenn der Parser `Z` ausgibt. Wert und Herkunft bleiben erhalten, fehlende Endpunktzeitzone oder fehlerhafte Vorumrechnung lassen sich aber nicht rekonstruieren. Vor genauer rechnerübergreifender Korrelation Original und Systemeinstellungen prüfen.

Ein naiver Zeitstempel ohne artefaktdefinierte Zeitzone wird als ungelöst gezählt und nicht stillschweigend als UTC behandelt. Wenn die Beweise dies rechtfertigen, geben Sie einen unterschriebenen festen Offset an:

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

Reine Datumswerte benötigen diese Option ebenfalls; ihr Mitternachtsanker ist eine Annahme mit **Tagesgenauigkeit**, kein beobachtetes Ereignis. Zeitzonennamen und Sommerzeitregeln sind unzulässig. Fehlende, Null-, ungültige oder außerhalb des Bereichs liegende Werte erzeugen keine künstlichen Epochenereignisse; gültige explizite 1970-Daten bleiben erhalten.

Standardmäßig sind zwei optionale Kategorien ausgeschlossen:

- `--include-collection-times`: LNK-/Jump-List-Quelldateimetadaten des Erfassungs-/Analysesystems. Mit `collection_filesystem` markiert und von Endpunktaktivität getrennt.
- `--include-interpreted-times`: allgemeine Registrierungs-QWORD-Werte, vom Leser als FILETIME interpretiert. Markierung `interpreted_value`, da viele tatsächlich Zähler sind.

## Ausgegebene und übersprungene Datensätze { #output-and-skipped-records }

Der Befehl schreibt **nur die mit `-o` ausgewählte Zeitleiste** in dem mit `-t` gewählten Format (standardmäßig `csv` oder `jsonl`). Es werden keine zeitunabhängigen Diagnose- oder Zusammenfassungs-Begleitdateien oder datensatzbezogenen Diagnose-Spools erstellt. Vorhandene Begleitdateien aus früheren Läufen bleiben unberührt.

Die Terminalzusammenfassung meldet die Anzahl der Zeitleistenereignisse und nicht zeitgesteuerten Datensätze. Ungültige Zeitstempel ungleich Null und die Anzahl der nicht aufgelösten Zeitzonen werden separat angezeigt. Mit `--sort` wird auch die Anzahl der konsolidierten Duplikate gemeldet. Behalten Sie die ursprünglichen Analyseergebnisse für Datensätze bei, die kein Zeitachsenereignis erzeugen, und für den gesamten Artefaktkontext.

Gültige Daten, die auf eine begründete Zeitzonenannahme warten, gelten als unaufgelöste Zeitzonen und nicht als ungültige Zeitstempel. Es wird kein Zeitstempel erfunden und Datensätze ohne nutzbare zugeordnete Zeit werden nicht als synthetische Epochenereignisse in die Zeitleiste eingefügt.

Datensätze mit gültigen Zeiten außerhalb des angeforderten Bereichs werden als gefiltert und nicht ohne Zeitangabe gezählt. Ein erfolgreicher Export bedeutet, dass die ausgewählten analysierten Eingaben erfolgreich verarbeitet wurden; Dies bedeutet nicht, dass jedes Artefakt eingesammelt wurde, jedes Feld einen Zeitstempel hat oder jeder Angriff erkennbar ist.

CSV kann verschachtelte Systemprofile, USB-Benutzer-Mountpoints und Shell-Items verflachen; natives JSON/JSONL ist hierfür oft reicher. Fehlende CSV-Daten lassen sich nicht rekonstruieren. Nicht unterstützte Artefaktkennungen und fehlerhafte Eingaben führen zu einem Fehler statt erfolgreicher Teilergebnisse.

## Leistung und sichere Ausgabe { #performance-and-output-safety }

Eingaben werden parallel gestreamt und verarbeitet. Standardmäßig schreibt jeder Worker ein formatiertes Ereignisfragment; die Fragmente werden in der Reihenfolge der Eingabedateien aneinandergehängt, ohne Sortierung und ohne erneutes Einlesen ihrer Ereignisstrukturen. CSV erhält auch bei mehreren Eingabedateien nur eine Kopfzeile. Die Ausgabereihenfolge ist unabhängig von der Worker-Anzahl deterministisch, bedeutet aber keine chronologische Ordnung.

Mit `--sort` laufen Sortierpuffer in ein privates temporäres Verzeichnis über; begrenzte Zusammenführungsdurchläufe liefern dann eine chronologische Ausgabe. Sortiert wird nach vollständigen ganzzahligen Sekunden und Nanosekunden, auch für Zeiten vor 1970. Gleiche Zeiten bleiben unabhängig von Worker-Anzahl und Sortierbudget deterministisch geordnet. Ein Fortschrittsbalken rückt mit jedem Ereignis vor, das in Zwischen- und Endzusammenführungen verarbeitet wird, auch bei wiederholten Durchläufen. Er erscheint im Terminal und wird bei umgeleiteter Ausgabe ausgeblendet.

`--memory-limit` gilt nur mit `--sort` und begrenzt das gesamte **Sortierpufferbudget**, nicht Prozess-RSS. Datensätze, JSON-Objekte/-Strings, CSV-Puffer und Zusammenführung benötigen weiteren Speicher; kleine Budgets reduzieren Worker. `--max-record-size` begrenzt Eingabedatensätze auf standardmäßig 16 MiB. Ein formatiertes JSON-Array zählt als ganzes Dokument; größere Daten benötigen eine höhere Grenze oder besser JSONL. Auch CSV-Lesevorschau zählt dazu.

Temporärspeicher muss zusätzlich zu den Ergebnissen die formatierten Ereignisfragmente (Standard) oder erweiterte Sortierdatensätze und Zusammenführungsdurchläufe (`--sort`) aufnehmen. In beiden Modi bleiben das temporäre Staging der Ausgabe und der Schutz vor dem Veröffentlichen fehlgeschlagener Eingaben erhalten. Ein Datensatz kann viele Ereignisse liefern, sodass die Timeline viel größer als die Eingabe werden kann. Bei Bedarf Zeit-/Artefaktfilter verwenden und ein ausreichend freies temporäres Volume wählen.

Strg+C bricht den kooperativen Vorgang ab, entfernt die privaten Sortier-/Staging-Verzeichnisse und wird mit dem Code `130` beendet. Bei Linux und macOS gilt dies auch für SIGTERM und SIGHUP. Die Stornierung wird beim Lesen, Normalisieren, Zusammenführen und Schreiben der Zeitleiste überprüft, bevor die Veröffentlichung beginnt. Bei einer nicht aufhebbaren Beendigung oder einem Stromausfall können weiterhin temporäre Dateien zurückbleiben.

Vorhandene Ausgänge erfordern `-C`. Die Zeitleiste darf Eingaben, Manifeste, referenzierte lokale Beweise oder deren geschützte Beweisverzeichnisse nicht überschreiben. Eingabefehler oder Sortierfehler lassen die bestehende Zeitleiste unberührt. Die fertige Zeitleiste wird neben ihrem Ziel bereitgestellt und ersetzt es atomar. Verwenden Sie für jeden Lauf einen neuen Ausgabedateinamen, wenn es wichtig ist, einen früheren vollständigen Export beizubehalten.

## Weitere Informationen { #further-details }

- [Timesketch-Import von CSV/JSONL](https://timesketch.org/guides/user/import-from-json-csv/)
- [Plaso-Zeitstempelerweiterung und älteres CSV](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Plaso-Ausgabe und Formatierung](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [Beweise und Interpretation](../overview/evidence-model.md)
- [Ausgabeformate](../getting-started/output-formats.md)

Aktuell Juntei Eingaben verwenden abgekürzte Bezeichnungen wie z `Comp`, `Filename`, `Args` Und `Attr`. MFT CSV akzeptiert auch den Strom `File`, `SN`, `ParentEntryNum`, `ParentSN`, `Type`, `USN`, `LogfileSN` Und `HasADS` Header, Beibehaltung der Sequenznummerngenerationen und DOS/ADS Unterscheidungen intakt. Ältere exportierte Etiketten werden weiterhin akzeptiert und `SourceField` Behält die Schreibweise in der tatsächlichen Eingabe bei. Hayabusa Die Eingabe bleibt erhalten `Computer` Feld. Bootsektor- und Sicherheitsbeschreibungsmetadaten ohne Ereigniszeitstempel bleiben in den ursprünglich analysierten Ergebnissen; Es werden keine Timeline-Ereignisse erstellt.

## Neuere Mehrtabellen-Parser { #newer-multi-table-parsers }

Natives JSON/JSONL enthält Tabellennamen. Bei einzelnen CSV-Dateien erkennen Browser-, WebCache- und Outlook-Adapter die Tabelle am Spaltenschema; Umbenennung oder Quellsuffixe ändern die Zeitinterpretation nicht. CSV wird ebenso unterstützt. UAL nutzt bekannte Zeitfelder, ohne Zähler oder ähnlich aussehenden Text als Datum zu erraten.

Outlook-Eigenschaften werden nur bei deklariertem Zeit-/Mehrfachzeittyp projiziert. Datumstext bleibt Text. Eingebettete Nachrichten behalten IDs und Elternverweise; Extraktionszeiten ersetzen keine gespeicherten Anlagenzeiten. Zeitlose Datei-, Empfänger-, Kontakt- und Eigenschaftszeilen bleiben im nativen Ergebnis und Begleitbericht auffindbar.

Für diese Parser gelten dieselben CSV-/JSONL-Schemas. Neue Artefakte erzeugen keine Hunderte CSV-Spalten; spezifischer Kontext bleibt in `Details`.
