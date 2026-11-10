# Gesamttimeline { #super-timeline }

`super-timeline` überführt geparste Artefakte in ein gemeinsames Ereignisschema und sortiert sie zeitlich. Mehrere Ausführungszeiten eines Prefetch-Datensatzes oder acht und mehr Dateizeiten eines MFT-Datensatzes werden einzeln beschriftet. So bleiben beim Quellenvergleich die unterschiedlichen Zeitbedeutungen erhalten.

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
  --memory-limit 64 -T 4 --temp-dir scratch/
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

Rohquellen stammen aus dem Datensatz oder einem eindeutigen Manifestbefehl. SRUM-CSV-Tabellen werden nur bei genau einer eindeutig erkennbaren gesammelten SRUDB.dat zugeordnet. Andernfalls verweist `SourceFile` auf die geparste Datei und `Details.RawSourceAvailable` ist `false`. Dieser Ersatzpfad ist kein Rohbeweispfad.

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
| `SourceFile` | Auflösbarer Rohbeweispfad; sonst der oben beschriebene Ersatzpfad zur geparsten Datei. |
| `SourceField` | Tatsächlich verwendetes Parser-Feld oder verschachtelte Position des Zeitstempels. |
| `RecordID` | Artefaktspezifische Datensatzkennung; nicht zwingend eine EVTX-Datensatz-ID. |
| `Details` | Strukturierter Kontext: Originalzeit, Genauigkeit, Zeitbasis/-rolle, geparste Datei, Eingabedatensatz/-position und verfügbare Wiederherstellungs-/Regelmetadaten. |

In CSV ist `Details` eine zitierte JSON-Zelle, in JSONL ein Objekt. Das Timesketch-Profil benennt `Timestamp`, `TimestampType` und `Message` in `datetime`, `timestamp_desc` und `message` um; die übrigen Herkunftsfelder bleiben erhalten. Dies sind die [drei erforderlichen Timesketch-Importfelder](https://timesketch.org/guides/user/import-from-json-csv/).

`Details` enthält ausgewählten Kontext statt vollständiger Binärdaten oder sämtlicher nativer Felder. Lange Zeichenfolgen und Strukturen werden begrenzt und mit `ProjectionOmitted` markiert. `Details.ParsedFile`, `InputRecord` und `Location` führen zum vollständigen Datensatz. Bei CSV zählt die Nummer CSV-Datensätze nach der Kopfzeile, keine physischen Zeilen in Zellen; bei JSONL Zeilen und bei JSON Dokumentnummer plus verschachtelte Position.

## Bedeutung der Zeitangaben { #what-the-times-mean }

Ein Zeitstempel beschreibt ein gespeichertes Objekt und beweist allein weder Ausführung noch Angriff. Die Adapter unterscheiden deshalb:

| Artefakte | Interpretation in der Timeline |
|---|---|
| MFT | Getrennte SI-/FN-Zeiten für Änderung, Zugriff, Metadatenänderung und Erstellung; benannte FN-Attribute bleiben unterscheidbar. Wiederholte SI-Felder werden zusammengeführt; gemeinsame Basisdateizeiten von ADS-Zeilen entfallen. Wiederhergestellte Namen aus FILE-Datensatz-Slack erhalten eigene wiederhergestellte FN-Ereignisse. |
| I30 | Gespeicherte FN-/Indexmetadaten mit verfügbarem Slack-/Wiederherstellungskontext, keine Zeitpunkte des Schreibens des Verzeichnisindexes. |
| USN-Journal | Journalzeit und Änderungsgründe; ein Datensatz kann mehrere Änderungen beschreiben, ohne deren exakten Beginn festzuhalten. |
| LogFile | Gespeicherte Redo-/Undo-Zeitschnappschüsse und Dateinamenmetadaten bleiben getrennt. LSNs und Dateioffsets sind Kennungen, keine Datumswerte. |
| Prefetch | Gespeicherte Ausführungszeiten, mit dem passenden Prefetch-Timeline-Export zusammengeführt. Volume-Erstellungszeiten werden gesondert beschriftet. |
| SQLite-Browserdatenbanken | Besuche, wiederhergestellte letzte Seitenbesuche, Downloadbeginn/-ende, Cookie-Erstellung/-Zugriff/-Ablauf und Formularnutzung/-löschung. Downloadende beweist keinen Erfolg oder Start, Cookie-Ablauf ist eine konfigurierte Frist. Wiederherstellungsstatus, Sicherheit und Herkunft bleiben erhalten. |
| Internet Explorer / älteres Edge WebCache | Gespeicherte Zugriffs-, Erstellungs-, Änderungs-, Ablauf- und Synchronisationszeiten sowie Download-, Cookie- und Containerverwaltung. Cache-Verwaltung beweist nicht automatisch einen Seitenaufruf durch den Benutzer. |
| Outlook PST, OST, MSG und OFT | Übermittlung/Zustellung, Erstellung/Änderung von Ordnern, Elementen und Anlagen; Kalender- und Aufgabenfristen, Erinnerungen, Abschlüsse sowie explizit typisierte Zeitwerte. Geplante Zeiten beweisen weder Besprechung noch Erinnerung oder Programmausführung. Zustellung beweist kein Lesen. |
| Benutzerzugriffsprotokollierung (UAL) | Erster/letzter Zugriff, Datenbankeinfügung/-erstellung, erste/letzte Sichtung und gespeicherter Systemstart. Rolle, Konto, Quelladresse und Tageszähler bleiben Kontext. Tageswerte werden nicht zu erfundenen Mitternachtsanmeldungen; zeitlose Identitäts- und DNS-Zeilen stehen im Begleitbericht. |
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

Gleiche Zeiten mit unterschiedlicher Bedeutung oder Herkunft bleiben erhalten. Identität berücksichtigt Rechner, Quelle, Artefakt, Datensatz, Zeitrolle und Wiederherstellungszustand. Zusammengeführte Zeilen enthalten `DuplicateCount` und begrenzte Alternativpfade/-felder. Ähnlichkeit allein führt nie zum Zusammenführen verschiedener Quellen. Bei unbekannter Rohquelle verhindert die Eingabedatensatzkennung das Vermischen verschiedener Erfassungen.

### Genauigkeit, Ortszeiten und Annahmen { #precision-local-times-and-assumptions }

Bekannte UTC-Felder und explizite Offsets werden auf UTC normalisiert. Originalwerte und Genauigkeit einschließlich 100-ns-FILETIMEs bleiben erhalten; bereits weggerundete Genauigkeit kann nicht wiederhergestellt werden.

Einige Shell-Item-/FAT-Felder bedeuten Ortszeit, selbst wenn der Parser `Z` ausgibt. Wert und Herkunft bleiben erhalten, fehlende Endpunktzeitzone oder fehlerhafte Vorumrechnung lassen sich aber nicht rekonstruieren. Vor genauer rechnerübergreifender Korrelation Original und Systemeinstellungen prüfen.

Zeitwerte ohne artefaktdefinierte Zeitzone werden diagnostiziert, nicht stillschweigend als UTC behandelt. Wenn Beweise es rechtfertigen, einen festen Offset mit Vorzeichen angeben:

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

Reine Datumswerte benötigen diese Option ebenfalls; ihr Mitternachtsanker ist eine Annahme mit **Tagesgenauigkeit**, kein beobachtetes Ereignis. Zeitzonennamen und Sommerzeitregeln sind unzulässig. Fehlende, Null-, ungültige oder außerhalb des Bereichs liegende Werte erzeugen keine künstlichen Epochenereignisse; gültige explizite 1970-Daten bleiben erhalten.

Standardmäßig sind zwei optionale Kategorien ausgeschlossen:

- `--include-collection-times`: LNK-/Jump-List-Quelldateimetadaten des Erfassungs-/Analysesystems. Mit `collection_filesystem` markiert und von Endpunktaktivität getrennt.
- `--include-interpreted-times`: allgemeine Registrierungs-QWORD-Werte, vom Leser als FILETIME interpretiert. Markierung `interpreted_value`, da viele tatsächlich Zähler sind.

## Begleitberichte { #companion-reports }

Für `SuperTimeline.csv` oder `SuperTimeline.jsonl` entstehen außerdem:

- `SuperTimeline-Untimed.jsonl`: Fundstellen nativer Datensätze ohne nutzbare zugeordnete Ereigniszeit.
- `SuperTimeline-Diagnostics.jsonl`: ungültige/mehrdeutige Zeiten, bewusst ausgelassene gemeinsame Metadaten und Einschränkungen, die den Originaldatensatz erfordern.
- `SuperTimeline-Summary.json`: Eingaben, Artefaktzähler, gefilterte/mögliche/konsolidierte Ereignisse, Ausgabeanzahl, Worker, Sortierbudget/-dateien/-Durchläufe und Laufzeit.

Gültige Zeiten außerhalb des Zeitfensters zählen als gefiltert, nicht zeitlos. Kandidaten werden vor Deduplizierung gezählt. `Complete: true` bedeutet erfolgreiche Verarbeitung aller gewählten Eingaben, nicht vollständige Erfassung, Zeitangaben in jedem Feld oder Erkennbarkeit jedes Angriffs.

CSV kann verschachtelte Systemprofile, USB-Benutzer-Mountpoints und Shell-Items verflachen; natives JSON/JSONL ist hierfür oft reicher. Fehlende CSV-Daten lassen sich nicht rekonstruieren. Nicht unterstützte Artefaktkennungen und fehlerhafte Eingaben führen zu einem Fehler statt erfolgreicher Teilergebnisse.

## Leistung und sichere Ausgabe { #performance-and-output-safety }

Eingaben werden parallel gestreamt; Sortierpuffer laufen in ein privates temporäres Verzeichnis über. Begrenzte Zusammenführungsdurchläufe liefern deterministische Zeitreihen. Vollständige Sekunden/Nanosekunden berücksichtigen auch Zeiten vor 1970. Gleiche Zeiten bleiben unabhängig von Workern und Budget stabil sortiert.

`--memory-limit` begrenzt das gesamte **Sortierpufferbudget**, nicht Prozess-RSS. Datensätze, JSON-Objekte/-Strings, CSV-Puffer und Zusammenführung benötigen weiteren Speicher; kleine Budgets reduzieren Worker. `--max-record-size` begrenzt Eingabedatensätze auf standardmäßig 16 MiB. Ein formatiertes JSON-Array zählt als ganzes Dokument; größere Daten benötigen eine höhere Grenze oder besser JSONL. Auch CSV-Lesevorschau zählt dazu.

Temporärspeicher muss erweiterte Ereignisse und Zusammenführungsdurchläufe zusätzlich zu Ergebnissen aufnehmen. Ein Datensatz kann viele Ereignisse liefern. Zeit-/Artefaktfilter verwenden und ein ausreichend freies temporäres Volume wählen.

Bestehende Ausgaben erfordern `-C`. Eingaben, Manifeste und geschützte Beweisquellen dürfen nicht überschrieben werden. Eingabe-/Sortierfehler lassen vorhandene Ergebnisse unverändert. Fertige Dateien werden am Ziel einzeln ersetzt, die Timeline zuletzt. Das ist **keine** Transaktion über vier Dateien: Ein abschließender E/A-Fehler kann einzelne Begleitberichte bereits ersetzt haben. Zum Bewahren früherer Exporte einen neuen Basisnamen verwenden.

## Weitere Informationen { #further-details }

- [Timesketch-Import von CSV/JSONL](https://timesketch.org/guides/user/import-from-json-csv/)
- [Plaso-Zeitstempelerweiterung und älteres CSV](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Plaso-Ausgabe und Formatierung](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [Beweise und Interpretation](../overview/evidence-model.md)
- [Ausgabeformate](../getting-started/output-formats.md)

Aktuelle Juntei-Eingaben verwenden Kürzel wie `Comp`, `Filename`, `Args` und `Attr`. MFT-CSV unterstützt auch `File`, `SN`, `ParentEntryNum`, `ParentSN`, `Type`, `USN`, `LogfileSN` und `HasADS`, unter Erhalt der Sequenzgenerationen und DOS-/ADS-Unterschiede. Ältere Namen bleiben gültig; `SourceField` zeigt die tatsächliche Schreibweise. Hayabusa behält `Computer`. Bootsektor- und Sicherheitsdeskriptormetadaten bleiben mangels Ereigniszeit im zeitlosen Bericht.

## Neuere Mehrtabellen-Parser { #newer-multi-table-parsers }

Natives JSON/JSONL enthält Tabellennamen. Bei einzelnen CSV-Dateien erkennen Browser-, WebCache- und Outlook-Adapter die Tabelle am Spaltenschema; Umbenennung oder Quellsuffixe ändern die Zeitinterpretation nicht. CSV wird ebenso unterstützt. UAL nutzt bekannte Zeitfelder, ohne Zähler oder ähnlich aussehenden Text als Datum zu erraten.

Outlook-Eigenschaften werden nur bei deklariertem Zeit-/Mehrfachzeittyp projiziert. Datumstext bleibt Text. Eingebettete Nachrichten behalten IDs und Elternverweise; Extraktionszeiten ersetzen keine gespeicherten Anlagenzeiten. Zeitlose Datei-, Empfänger-, Kontakt- und Eigenschaftszeilen bleiben im nativen Ergebnis und Begleitbericht auffindbar.

Für diese Parser gelten dieselben CSV-/JSONL-Schemas. Neue Artefakte erzeugen keine Hunderte CSV-Spalten; spezifischer Kontext bleibt in `Details`.
