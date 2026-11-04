# NTFS-Transaktionsprotokoll (`$LogFile`) { #ntfs-transaction-log-logfile }

## Was es ist und warum es wichtig ist { #what-it-is-and-why-it-matters }

NTFS ist das Dateisystem, das Windows in der Regel für sein Systemvolume verwendet. Um sich nach einem Absturz wiederherzustellen,
zeichnet es geplante Änderungen an Dateisystem-Metadaten in einer versteckten Volume-Datei namens
`$LogFile` auf. Jede Aktualisierung kann **Redo**-Bytes (den neuen Zustand) und **Undo**-Bytes (den vorherigen
Zustand, der zum Zurücksetzen einer Änderung benötigt wird) enthalten. Dies ist unabhängig von den Windows-Ereignisprotokollen, den `.LOG1`-Dateien
der Registry und dem [USN-Journal](usn-journal.md).

Für Ermittelnde ist der vorherige Zustand besonders nützlich. Ein Undo-Datensatz kann einen
Dateinamen vor einer Umbenennung, einen gelöschten Verzeichniseintrag oder Zeitstempel vor einer Metadatenaktualisierung bewahren.
Manche Datensätze enthalten außerdem Bytes, die NTFS in das USN-Journal geschrieben hat, einschließlich Datensätzen, die im
gesicherten `$J` nicht mehr vorhanden sind. Viele Transaktionen betreffen jedoch routinemäßige Betriebssystemaktivität und
beschreiben keinen Angriff.

**Beispiel:** Ein entfernter Indexeintrag nennt `report.txt`; ein später hinzugefügter Eintrag nennt
`report.txt.encrypted`. Der Abgleich der vollständigen Dateireferenz, des übergeordneten Verzeichnisses, der Transaktionskette und anderer
Artefakte kann eine Umbenennung stützen. Diese beiden Namen allein identifizieren weder Ransomware noch deren Betreiber
noch den exakten Zeitpunkt der Verschlüsselung.

## Speicherort und Sicherung { #location-and-collection }

```text
C:\$LogFile
D:\$LogFile
```

Jedes NTFS-Volume hat sein eigenes Protokoll. Sichern Sie es mit einem Werkzeug, das NTFS-Metadaten liest, oder exportieren Sie es
aus einem forensischen Image oder Snapshot; eine gewöhnliche Windows-Dateikopie kann möglicherweise nicht darauf zugreifen. Sichern Sie die
`$MFT` und `$Extend\$UsnJrnl:$J` desselben Volumes zeitlich so nah wie möglich. Bewahren Sie die Originaldateien
und Hashes auf. Eine spätere Sicherung kann einen anderen Aktivitätsabschnitt enthalten.

`$LogFile` ist ein **zirkuläres Protokoll**: Neue Datensätze überschreiben alte. Die Aufbewahrungsdauer hängt von der Protokollgröße
und der Arbeitslast ab und kann nur Minuten oder Stunden betragen. Das Fehlen eines Dateinamens oder einer Operation ist kein
Beleg dafür, dass sie nie stattgefunden hat.

## Was ein Datensatz beweist, und Hinweise { #what-a-record-proves-and-cautions }

- Eine Aktualisierung zeichnet Metadaten auf, die NTFS zur Wiederherstellung protokolliert hat. Ein einzelner Redo-Eintrag beweist nicht, dass
  eine gesamte übergeordnete Aktion abgeschlossen wurde oder dass eine bestimmte Person sie ausgelöst hat. Korrelieren Sie
  zusammengehörige Datensätze, den Abschluss der Transaktion, das resultierende Dateisystem und unabhängige Belege.
- **Es gibt keinen allgemeinen Transaktionszeitstempel.** Eingebettete Zeitstempel beschreiben Dateimetadaten,
  die mitunter veraltet, kopiert oder vom Angreifer verändert sind. Setzen Sie `CreatedOn` nicht als Zeitpunkt einer Umbenennung oder Zeitstempeländerung
  in eine Vorfallszeitleiste. USN-Datensätze können einen separaten Zeitstempel liefern.
- LSNs (Log Sequence Numbers) ordnen Datensätze **innerhalb des Protokolls eines Volumes**. Sie sind keine Uhrzeiten
  und dürfen nicht über Hosts oder Volumes hinweg verglichen werden. Datensätze aus älteren Durchläufen des Ringpuffers
  können neben neuerer Aktivität erhalten bleiben.
- Ein Slot der Transaktionstabelle wird wiederverwendet. Gruppieren Sie mit `TransactionFirstLsn`, `TransactionId`, Client-Identität
  und Quelle statt nur mit `TransactionId`. Ketten können auf Bereiche außerhalb der erhaltenen Daten verweisen.
- Dateireferenzen enthalten eine Eintragsnummer **und eine Sequenznummer**. Andernfalls kann die Wiederverwendung eines MFT-Eintrags
  dazu führen, dass nicht zusammenhängende Dateien als dieselbe Datei erscheinen.
- Von der `$MFT` gelieferte Pfade beschreiben den **gesicherten MFT-Zustand**, keinen rekonstruierten historischen
  Pfad. Juntei gleicht Sequenznummern ab und weist unbekannte Komponenten ausdrücklich aus. Ein übergeordnetes Verzeichnis kann
  nach der Transaktion umbenannt worden sein, und Hardlinks können mehr als einen aktuellen Namen bieten.
- `TargetInferred` bedeutet, dass eine Zuordnung eines geöffneten Attributs aus einem anderen Abschnitt des Protokolls stammt.
  Behandeln Sie diese Zuordnung mit besonderer Vorsicht. Nicht aufgelöste Indexeinträge werden möglicherweise allein anhand ihres
  Layouts dekodiert; validieren Sie ihre Namen mit Rohbytes und anderen Quellen.
- Eine zerrissene Seite (Torn Page) wird übersprungen; lesbare umgebende Datensätze bleiben verfügbar. `Truncated` kennzeichnet einen
  Datensatz, dessen Fortsetzung fehlt. Prüfen Sie die Diagnosemeldungen des Parsers, bevor Sie die Ausgabe als
  vollständig behandeln. Eine Eingabe der Länge null ist ein Fehler; ein ungeschriebenes, vollständig aus `FF` bestehendes Protokoll hat berechtigterweise keine Zeilen.
- Der Parser spielt Transaktionen **nicht ein**, um eine historische MFT zu rekonstruieren. Er stellt keine
  beliebigen nicht residenten Dateiinhalte wieder her und führt keine gesicherten Daten aus. Durch DAX-CRC geschützte
  Seiten werden nicht unterstützt. Die Validierung mit realen Beweisdaten deckt in erster Linie Protokollseiten mit 4 KiB ab.

## Mit Juntei parsen { #parse-with-juntei }

```bash
# Single log, current paths from an adjacent $MFT when present
juntei parse-ntfs logfile -f '$LogFile' -o logfile.csv

# Explicit MFT from the SAME volume
juntei parse-ntfs logfile -f '$LogFile' -m '$MFT' -t jsonl -o logfile.jsonl

# Multiple collected logs; SourceFile distinguishes every input
juntei parse-ntfs logfile -d evidence -o collected-logs.csv

# Preserve the underlying payloads for low-level validation
juntei parse-ntfs logfile -f '$LogFile' --raw-data -t jsonl -o logfile-raw.jsonl

# Automatic per-computer output
juntei auto-parse -d evidence -o parsed
```

Alle Optionen finden Sie in der [Befehlsreferenz](../commands/filesystem.md#parse-ntfs-logfile). Die Verzeichnissuche
akzeptiert `$LogFile`, `LogFile`, deren `.bin`-Formen sowie Namen wie `C_$LogFile` oder
`C_$LogFile.bin`, ohne Berücksichtigung der Groß-/Kleinschreibung. Eine einzelne mit `-f` übergebene Datei kann einen beliebigen Namen haben.

Die automatische MFT-Zuordnung berücksichtigt das Volume-Präfix eines Exports: `C_$LogFile` verwendet `C_$MFT`, und
`D_$LogFile.bin` versucht zuerst `D_$MFT.bin`, dann `D_$MFT`. Protokolle mit Präfix verwenden niemals eine einfache `$MFT`
als Ausweichlösung. Ein `LogFile` ohne Präfix akzeptiert auch `MFT` oder `$MFT`. Verwenden Sie `-m` zusammen mit `-f`, um explizit eine
passende MFT auszuwählen, wenn Ihr Export ein anderes Benennungsschema hat.

Die Ausgabe enthält einen Datensatz pro LSN, einschließlich Aktualisierungs-, Checkpoint- und unbekannter Client-Datensätze. JSON ist ein
Strom formatiert ausgegebener Objekte; JSONL enthält ein Objekt pro Zeile. CSV hat dieselben Feldnamen;
Arrays und strukturierte Felder stehen als kompaktes JSON in einer CSV-Zelle. Unbekannte Skalare sind in
CSV leer und in JSON `null`. Leere Arrays bedeuten, dass keine Elemente dekodiert wurden, nicht, dass keine existierten.

Es wird jeweils ein vollständiges Protokoll samt seinen geparsten Datensätzen im Speicher gehalten. Das Auflösen von MFT-Pfaden erhöht
Speicherbedarf und Aufwand. `--raw-data` kann die Ausgabegröße stark erhöhen. Verwenden Sie in `auto-parse` `-T`, um
auf einem Rechner mit begrenztem Speicher die Anzahl gleichzeitiger Parserprozesse zu beschränken.

Wenn eine benachbarte MFT nicht gelesen werden kann, gibt Juntei eine Warnung aus und exportiert das Protokoll ohne Pfade; `MftSource`
bleibt dann null. Dieser Fehler bei der Anreicherung verwirft keine lesbaren primären Beweisdaten.
Fehlende oder mehrdeutige MFT-Zuordnungen erzeugen ebenfalls eine Warnung und einen Zähler `Logs without a matching MFT`
in der Zusammenfassung; sie zählen nicht als Parse-Fehler.

## Ausgabefelder { #output-fields }

### Identität, Integrität und Herkunft des Datensatzes { #record-identity-integrity-and-provenance }

| Feld | Bedeutung und Interpretation |
|---|---|
| `SourceFile`, `FileOffset` | Eingabepfad und Byte-Offset, an dem der Header dieses Datensatzes gelesen wurde; verwenden Sie diese zur Validierung der Bytes. |
| `LSN` | Die Log Sequence Number des Datensatzes. Große Werte sind Ganzzahlen, keine Zeitstempel. |
| `PreviousLSN`, `UndoNextLSN` | Verweise auf vorherige und Rollback-Datensätze. Null bedeutet kein Verweis; Verweise ungleich null können auf Bereiche außerhalb der erhaltenen Daten zeigen. |
| `TransactionID`, `TransactionFirstLSN` | Slot der Transaktionstabelle und die früheste mit dieser Transaktion verknüpfte LSN; der Slot allein ist nicht eindeutig. |
| `RecordType` | `ClientRecord` (Aktualisierung), `ClientRestart` (Checkpoint) oder `Other`. |
| `ClientIndex`, `ClientSequence` | Identität des Protokoll-Clients; normalerweise NTFS. Dies sind weder ein Benutzerkonto noch eine Prozess-ID. |
| `Flags`, `SpansPages` | Rohe Datensatzflags und ob sich die Client-Daten auf einer weiteren Seite fortsetzen. Seitenübergreifende Datensätze sind normal. |
| `Truncated` | True, wenn der Parser für diese LSN eine fehlende Fortsetzung gefunden hat. Es fasst nicht alle Schäden in der gesamten Datei zusammen. |
| `ClusterSize`, `ClusterSizeSource` | Bytes pro Cluster für Ziel-Offsets; die Quelle ist `Caller`, `Checkpoint` oder `Inferred`. Geben Sie den tatsächlichen Wert mit `--cluster-size` an, falls bekannt. |
| `LogVersion` | Version aus der neuesten lesbaren Restart-Seite, typischerweise `1.1` oder `2.0`; null, wenn keine lesbare Restart-Seite erhalten ist. |
| `MFT-Source` | Die für die Anreicherung mit aktuellen Pfaden verwendete MFT, falls vorhanden. Sie muss zum selben Volume gehören. |

In der JSON-/JSONL-Ausgabe von `auto-parse` werden `Comp` und `Artifact` wie bei anderen Parsern hinzugefügt. Das
`.txt`-Protokoll der Aufgabe enthält Diagnosemeldungen; die Manifeste geben an, ob die Aufgabe erfolgreich war, fehlgeschlagen ist oder übersprungen wurde.
Behebbare Schäden können zu einer erfolgreichen Aufgabe mit einem Zähler `Parse problems` ungleich null in ihrem Protokoll führen.

### Operationen und Ziel { #operations-and-target }

| Feld | Bedeutung und Interpretation |
|---|---|
| `RedoOperation`, `UndoOperation` | Namen der NTFS-Operationen oder hexadezimal bei einem unbekannten Code. Redo beschreibt die beabsichtigte Aktualisierung; Undo beschreibt deren Umkehrung. |
| `TargetEntryNumber`, `TargetSequenceNumber` | Bei einer Operation auf einem Dateidatensatz: die Nummer des betroffenen MFT-Datensatzes, ohne abgeleitete Sequenznummer. Andernfalls: die Dateireferenz, der das aufgelöste geöffnete Attribut gehört. Eine Aktualisierung eines Inline-Verzeichnisindex zielt auf das Verzeichnis, nicht auf das durch ihren Eintrag beschriebene untergeordnete Element. |
| `TargetAttributeType`, `TargetAttributeName` | Numerischer Typ und Name des von der Operation verwendeten geöffneten Attributs. Bei Operationen auf Dateidatensätzen ist dies normalerweise `$DATA` der `$MFT`. Häufige Typen: 16 (`$STANDARD_INFORMATION`), 48 (`$FILE_NAME`), 128 (`$DATA`), 144 (`$INDEX_ROOT`), 160 (`$INDEX_ALLOCATION`), 176 (`$BITMAP`). |
| `TargetPath` | Aktueller MFT-Pfad, wenn die vollständige Referenz des Ziels bekannt ist, oder unbekannte Komponenten. Leer bei Operationen auf Dateidatensätzen, deren historische Generation unbekannt ist; eine Datensatznummer allein darf nicht auf ihren aktuellen Belegungsinhaber aufgelöst werden. |
| `OpenAttributeEntryNumber`, `OpenAttributeSequenceNumber`, `OpenAttributePath` | Datei, der das geöffnete Attribut gehört, über das die Operation schreibt, mit ihrem aktuellen MFT-Pfad. Bei Operationen auf Dateidatensätzen bewahren diese den `$MFT`-Kontext getrennt vom betroffenen Datensatz. |
| `TargetAttribute` | Offset in der Tabelle der geöffneten Attribute des Protokolls; dies unterscheidet sich vom Typ oder der MFT-Kennung des Attributs. |
| `TargetInferred` | True, wenn die Zielzuordnung aus einem anderen Abschnitt des Protokolls stammt. Ein Ziel mit Wert null bleibt unaufgelöst, auch wenn dieses Flag false ist. |
| `TargetVCN`, `Lcns` | Virtueller Cluster im Ziel und physische Clusternummern; eher für die Validierung auf niedriger Ebene als für die zeitliche Einordnung nützlich. |
| `RecordOffset`, `AttributeOffset` | Offsets des Attributs/Eintrags und der Aktualisierung darin. |
| `ClusterBlockOffset`, `TargetBlockSize` | Offset und Zielgröße in Einheiten von 512 Bytes. |
| `OffsetInTarget` | Berechneter Byte-Offset im betroffenen Dateidatensatz oder nicht residenten Stream. Hängt von Cluster- und MFT-Datensatzgröße ab. |
| `RedoLength`, `UndoLength` | Angegebene Datenlängen. Eine Nullungsoperation kann eine Länge angeben, ohne entsprechend viele Nutzdatenbytes zu enthalten. |
| `RedoBytesAvailable`, `UndoBytesAvailable` | Tatsächlich zur Dekodierung verfügbare Nutzdatenbytes. Mit den angegebenen Längen und der Semantik der Operation vergleichen. |

### Praktische Spalten für Dateimetadaten { #convenient-file-metadata-columns }

Diese Spalten zeigen einen bevorzugten Nicht-DOS-Dateinamen von der Redo-Seite, ersatzweise von der Undo-Seite.
Enthält keine der beiden Seiten einen Dateinamen, bevorzugen Felder für Zeitstempelaktualisierungen die Redo-Zeitstempel, dann die Undo-Zeitstempel.
Aktualisierungen von Verzeichniseinträgen geben ihre Zeiten, Größe und Flags ebenfalls direkt aus, wobei Redo Vorrang vor
Undo hat. Dekodierte `$DATA`-Attribute geben ihre Größe auch dann aus, wenn kein Dateiname vorhanden ist. Die Größe des unbenannten
`$DATA` eines dekodierten Dateidatensatzes hat Vorrang vor der möglicherweise veralteten Größe aus dem Dateinamen.
Sie dienen als Hilfe beim Durchsehen; verwenden Sie für einen vollständigen Vorher-Nachher-Vergleich die separaten Felder der beiden Seiten.

| Feld | Bedeutung und Interpretation |
|---|---|
| `MetadataSide` | `Redo` oder `Undo`; kennzeichnet die Seite, die die praktischen Dateinamen-/Zeitstempelspalten liefert. |
| `EntryNumber`, `SequenceNumber` | Datei, die durch dekodierte Metadaten aus Dateiname/Dateidatensatz/Attribut oder eine partielle Zeitstempelaktualisierung von `$STANDARD_INFORMATION` beschrieben wird, mit der Sequenznummer, sofern bekannt. Leer bei einer Aktualisierung eines Verzeichniseintrags, die das untergeordnete Element nicht identifiziert, sowie bei rohen Größen-/Run-List-Aktualisierungen. Verwenden Sie `TargetEntryNumber` für den betroffenen Datensatz. |
| `Filename` | Bevorzugter dekodierter Name; alle Namen bleiben in den Arrays der beiden Seiten erhalten, einschließlich DOS-Aliasen und Hardlinks. |
| `ParentEntryNumber`, `ParentSequenceNumber`, `ParentPath` | Referenz auf das übergeordnete Verzeichnis und aktueller MFT-Pfad; keine Rekonstruktion des historischen Pfads. |
| `CreatedOn`, `ContentModifiedOn`, `RecordModifiedOn`, `LastAccessedOn` | Metadatenzeiten in UTC mit einer Genauigkeit von 100 ns. Dies sind keine Transaktionszeitstempel. Fehlende/ungültige Zeiten bleiben leer/null. |
| `FileSize`, `FileAttributes` | Größe und numerische Windows-Flags aus den ausgewählten Metadaten; Werte aus Verzeichniseinträgen können veraltet sein. `$DATA`-Attribute und dekodierte Dateidatensätze können eine direktere Größe liefern. |
| `AttrType` | Numerischer Typ eines dekodierten Attributdatensatzes; 48 bei einer dekodierten Metadatenaktualisierung eines Verzeichniseintrags. Unabhängig vom geöffneten Attribut, auf das die Operation zielt. |

### Vollständige Redo-/Undo-Metadaten und Nutzdaten { #complete-redoundo-metadata-and-payloads }

| Feld | Bedeutung und Interpretation |
|---|---|
| `RedoFileNames`, `UndoFileNames` | Arrays aller dekodierten `$FILE_NAME` auf dieser Seite; hinzugefügte/neue und entfernte/alte Namen bleiben unterscheidbar. |
| `RedoTimes`, `UndoTimes` | Von der jeweiligen Seite geschriebene Zeitstempel aus `$STANDARD_INFORMATION`; bei partiellen Aktualisierungen fehlen unveränderte Zeitstempel, statt erraten zu werden. |
| `RedoDecoded`, `UndoDecoded` | Strukturierte Informationen zu Dateidatensatz, Attribut, Indexeintrag oder Verzeichniseintrag, gekennzeichnet durch `Kind`. Null kann eine nicht unterstützte Operation oder zu wenige Bytes bedeuten. |
| `JournalWriteOffset`, `JournalWriteLength` | Offset und Byteanzahl von Redo-Daten, die `$DATA:$J` schreiben. Dies kennzeichnet eine mögliche Quelle ansonsten verlorener USN-Datensätze; es wird kein rekonstruiertes Journal exportiert. |
| `Checkpoint` | Checkpoint-Version, Start-LSN, Clustergröße sowie LSN/Länge der Dumps der Tabelle geöffneter Attribute, der Attributnamen, der Dirty Pages und der Transaktionstabelle. |
| `OtherRecordType` | Roher Client-Datensatztyp für einen nicht erkannten oder nicht dekodierbaren Inhalt. |
| `RedoData`, `UndoData`, `OtherData` | Hex-Nutzdaten nur mit `--raw-data`; andernfalls null/leer. Bewahren Sie diese zur Validierung nicht unterstützter oder beschädigter Strukturen auf. |

Jedes Dateinamenobjekt enthält die Referenzen auf Datei und übergeordnetes Verzeichnis (`Raw`, `EntryNumber`,
`SequenceNumber`), `Name`, `NameEscaped`, `Namespace`, den aktuellen `ParentPath`, vier Zeitstempel,
`Size`, `AllocatedSize`, `Attr` und `ReparseValue`. `NameEscaped` kennzeichnet ungültiges UTF-16,
das mit Escape-Sequenzen dargestellt wird. Ein Reparse-Wert kann ein Reparse-Tag oder die Größe erweiterter Attribute sein.

Zeitstempelobjekte bewahren sowohl `Utc` als auch die ganzzahlige `Filetime`; ein Sentinel- oder ungültiger Wert kann
eine UTC-Darstellung von null haben und dennoch seine ursprüngliche Ganzzahl behalten. `RedoTimes` und `UndoTimes`
enthalten `Created`, `Modified`, `Changed` und `Accessed`; die Werte fehlender Elemente sind null.

Dekodierte `FileRecord`-Objekte enthalten Nummer, Sequenz, Anzahl der Hardlinks, Flags, Basisreferenz,
ob nur der Header erhalten ist, und die vorhandenen Attribute. Jedes dekodierte Attribut bewahrt
Typ, Name, Residenzstatus, Flags, ID, Größe und zugewiesene Größe. `$STANDARD_INFORMATION` ergänzt die
vier Zeiten, Attribute, Besitzer-/Sicherheits-IDs, Kontingent und USN; diese IDs müssen gesondert aufgelöst werden und
sind kein Beweis für den Benutzer, der die Datei geändert hat. Indexeinträge enthalten Flags, Schlüssellänge und einen
optionalen Dateinamen. Aktualisierungen von Verzeichniseinträgen bewahren die vier Zeiten, Größen, Attribute und den Reparse-Wert
auch dann, wenn der Name selbst nicht protokolliert wurde.

## Untersuchungsablauf { #investigation-workflow }

1. Prüfen Sie die Diagnosemeldungen auf fehlende Restart-Seiten, zerrissene Seiten und abgeschnittene Datensätze.
2. Durchsuchen Sie sowohl `RedoFileNames` als auch `UndoFileNames` nach einem verdächtigen Dateinamen oder einer verdächtigen Erweiterung.
3. Gleichen Sie die Eintragsnummer **und die Sequenznummer** ab und untersuchen Sie dann die LSN-Kette und die Operationen der Transaktion.
4. Vergleichen Sie Redo- und Undo-Zeitstempel auf Änderungen. Routinemäßiges Kopieren, Entpacken und Wiederherstellen kann
   Zeitstempel ebenfalls ändern; ein Unterschied ist ein Ansatzpunkt, kein automatischer Befund für Timestomping.
5. Korrelieren Sie mit dem [USN-Journal](usn-journal.md), der [$MFT](mft.md) und [$I30](i30.md). Verwenden Sie Ereignisprotokolle,
   [Prefetch](prefetch.md) und Artefakte der Benutzeraktivität, um Ausführung oder Zuordnung festzustellen.

Finden Sie beispielsweise alle Datensätze, die auf einer der beiden Seiten einen verschlüsselten Dateinamen tragen:

```bash
jq 'select(any(.RedoFileNames[]?, .UndoFileNames[]?;
  .Name | endswith(".encrypted"))) |
  {Lsn, TransactionId, TransactionFirstLsn, RedoOperation, UndoOperation,
   RedoFileNames, UndoFileNames}' logfile.jsonl
```

Dies ist eine Suche nach Dateinamen, keine vollständige Ransomware-Erkennung. Um Journal-Nutzdaten zu untersuchen,
exportieren Sie mit `--raw-data` und wählen Datensätze mit einem `JournalWriteOffset` ungleich null aus; das Hex in `RedoData`
stellt die an diesem Offset geschriebenen Bytes dar. Das Parsen und Deduplizieren dieser eingebetteten USN-Datensätze
ist ein separater Schritt. Juntei führt sie noch nicht mit der Ausgabe von `parse-ntfs usn-journal` zusammen.

## Weitere Informationen { #further-details }

- [logfile-Crate von ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/logfile): unterstützte Strukturen, Validierungskorpus und bekannte Einschränkungen.
- [Sootmark ntfslog](https://github.com/Sootmark/ntfslog): Rust-Implementierung des Upstream-Projekts.
- [NTFS-Formatdokumentation von libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation): Protokollseiten, Fixups und Metadatenstrukturen.
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs): unabhängige Forschung und Implementierung zum Parsen von NTFS.
- [LogFileParser von Joakim Schicht](https://github.com/jschicht/LogFileParser): detaillierte Analyse des Transaktionsprotokolls.
- [ntfs-linker von Stroz Friedberg](https://github.com/strozfriedberg/ntfs-linker): Korrelation von NTFS-Quellen zu einer Dateihistorie.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum): reale Windows-NTFS-Beispiele.
- [Velocidex go-ntfs](https://github.com/Velocidex/go-ntfs): NTFS-Implementierung und öffentliches Testvolume.
