# Master File Table (`$MFT`) { #master-file-table-mft }

## Was die MFT ist { #what-the-mft-is }

NTFS stellt jede Datei und jedes Verzeichnis durch einen Datensatz in der Master File Table dar. Datensätze enthalten
Attribute wie Namen, Elternreferenzen, Zeitstempel, Größen, Flags, Sicherheitskennungen,
Reparse-Informationen und Metadaten der Datenstreams. Kleine Dateiinhalte und alternative Datenströme können
**resident** sein, d. h., ihre Bytes werden innerhalb des MFT-Datensatzes selbst gespeichert.

Die MFT ist grundlegend, weil sie zugeordnete und gelöschte Datensätze des gesamten Volumes abdeckt.
Sie kann zeigen, dass eine Datei existierte, wo sie verlinkt war, ob sie gelöscht wurde und wie sich ihre Metadaten
geändert haben – selbst wenn das gewöhnliche Durchsuchen von Verzeichnissen sie nicht mehr anzeigt.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Ein NTFS-Dateidatensatz mit den angezeigten
Metadaten existierte zum Zeitpunkt der Sicherung oder hat in einem nicht zugeordneten Datensatz überdauert. Die MFT allein
beweist weder eine Ausführung noch eine Absicht des Benutzers.

## Speicherort und Sicherung { #location-and-collection }

```text
C:\$MFT
```

`$MFT` sind geschützte NTFS-Metadaten. Sichern Sie sie mit einem forensischen Akquisitions- oder Triage-Tool, das
rohen NTFS-Zugriff beherrscht; eine gewöhnliche Kopie im Benutzermodus kann sie normalerweise nicht lesen. Sichern Sie nach Möglichkeit
auch `$J`, `$LogFile` und Volume-Metadaten.

## Hinweise für die Analyse { #analyst-cautions }

- Ein gelöschter Datensatz kann teilweise überschrieben sein, oder seine Datensatznummer kann wiederverwendet werden.
- `SequenceNumber` unterscheidet Generationen derselben Eintragsnummer. Gleichen Sie Eintrag **und** Sequenz ab.
- Zeitstempel von `$STANDARD_INFORMATION` (`0x10`) und `$FILE_NAME` (`0x30`) werden unterschiedlich
  aktualisiert. Keiner ist gegen Manipulation gefeit.
- `SI<FN`, `ZeroU-Secs` und `Copied` sind Ansatzpunkte, keine automatischen Urteile über Timestomping.
- Hardlinks erzeugen mehrere `$FILE_NAME`-Attribute und Pfade für einen einzigen Dateidatensatz.
- NTFS kann mehrere Namen (einschließlich DOS 8.3), alternative Datenströme und Erweiterungsdatensätze speichern.
- Aktualisierungen der Zugriffszeit können deaktiviert, verzögert oder zusammengefasst sein.
- Ein vollständiger Pfad wird aus Elterndatensätzen rekonstruiert. `[Orphaned]`, `[Unknown]` oder `PathUnknown`
  signalisieren eine fehlende/wiederverwendete/beschädigte Abstammung – keinen wörtlichen Verzeichnisnamen.
- Der **Slack** eines FILE-Datensatzes (seine Bytes jenseits der genutzten Größe) kann Namen bewahren, die er zuvor enthielt: die
  Indexeinträge eines kleinen Verzeichnisses, dessen Index in eine Indexzuordnung verschoben wurde (die meisten davon sind dort
  noch in Verwendung), einen früheren Namen oder den vorherigen Inhaber eines wiederverwendeten Datensatzes. Wiederhergestellte Namen
  haben kein `$STANDARD_INFORMATION`, und der Eintragsheader eines Namens kann überschrieben worden sein. Wenn die genutzte Größe
  oder die Attribute eines Datensatzes nicht angeben, wo sein Slack beginnt, wird daraus nichts wiederhergestellt,
  und `ParseErrors` seiner Zeile meldet `slack not read` samt Grund.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $MFT files
  -f, --file <FILE>      Parse one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse selected entry numbers (example: 0-15,30)

Output:
  -o, --output <PATH>                   Save records
  -t, --output-type <TYPE>              csv, ys-ntfs-parser-csv, json, jsonl
  -e, --extract-resident-streams <DIR>  Extract resident $DATA streams

CSV Output:
  -a, --all-times      Include every $FILE_NAME time
  -s, --short-names    Include DOS 8.3-name rows
      --cutoff <DATE>  Keep recent SI-created or SI-modified records

Records:
  -x, --recover-slack  Recover file names from FILE record slack

Resident Data:
  -i, --include-resident-data      Put resident $DATA in the CSV
  -E, --resident-extensions <EXT>  Only these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  Only data up to this size

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail
  -L, --list-directory <ENTRY|PATH>  List a directory's contents
```

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff '2026-10-20T00:00:00Z' -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 32000-32100 | jq .
juntei parse-ntfs mft -f '$MFT' -x -o mft-with-slack.csv
juntei parse-ntfs mft -f '$MFT' -i -E ps1,bat,vbs,js -o scripts.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Users\alice\Downloads'
```

Führen Sie mit `-e` extrahierte residente Streams niemals aus; Juntei gibt ihnen nicht ohne Grund `.dontrun`-Namen.
Die Spalten für residente Daten von `-i` enthalten dieselben Bytes: Behandeln Sie sie als nicht vertrauenswürdige Inhalte.

`-D` gibt einen Eintrag aus: seinen Header, jedes Attribut (mit Data Runs) und die Namen in seinem Slack, so wie
`-t json -x` sie schreibt, danach die Bytes des Datensatzes. `-L` listet jeden Namen in einem Verzeichnis auf, verwendete und
gelöschte, mit Eintrags- und Sequenznummer, Typ, Größe und Änderungszeit. Ein Verzeichnis wird über den
Eintrag (`30`, `0x1E`, `30-1`) oder über den Pfad (`\Windows\System32`, `C:/Windows`; ohne Beachtung der Groß-/Kleinschreibung) angegeben.

## CSV-Ausgabefelder { #csv-output-fields }

| Feld | Interpretation |
|---|---|
| `EntryNum` | MFT-Datensatznummer. Mit `SequenceNumber` kombinieren; Datensatznummern werden wiederverwendet. |
| `SequenceNumber` | Generationszähler, der von NTFS-Referenzen verwendet wird. Eine Abweichung kann veraltete/gelöschte Beziehungen aufdecken. |
| `InUse` | Zuordnungsflag zum Zeitpunkt der Sicherung. `false` bedeutet, dass der Datensatz frei/gelöscht ist, nicht dass alle Bytes intakt sind. |
| `ParentEntryNumber`, `ParentSequenceNumber` | Referenz auf das übergeordnete Verzeichnis aus `$FILE_NAME`. Beide Teile validieren. |
| `ParentPath` | Rekonstruierter übergeordneter Pfad. Markierungen für unbekannt/verwaist zeigen ungeklärte Abstammung. |
| `Filename` | Name aus `$FILE_NAME` oder dem Stream der aktuellen Zeile. Ein Datensatz kann mehrere Zeilen haben. |
| `Ext` | Aus dem Namen abgeleitete Erweiterung zur Vereinfachung; keine verlässliche Bestimmung des Dateityps. |
| `Size` | Der Zeile/dem Stream zugeordnete logische Größe. Die Zuordnung bei Sparse-/komprimierten Dateien kann abweichen. |
| `ReferenceCount` | NTFS-Referenzzähler der Hardlinks; vorsichtig mit der Anzahl der erhaltenen Namen vergleichen. |
| `ReparseTarget` | Dekodiertes Ziel unterstützter Reparse-Punkte wie Junctions/Symlinks. Pfade können umleiten. |
| `Dir` | Datensatz trägt das Verzeichnisflag. |
| `HasADS` | Datensatz enthält benannte alternative Datenströme. Das macht ihn nicht bösartig. |
| `IsADS` | Diese Zeile stellt einen benannten alternativen Datenstrom statt der unbenannten Dateidaten dar. |
| `SI<FN` | Die SI-Erstellungszeit liegt vor einer abweichenden FN-Erstellungszeit. Kopieren/Timestomping und legitime Ursachen untersuchen. |
| `ZeroU-Secs` | SI-Erstellungs-/Änderungszeit liegt exakt auf einer Millisekundengrenze. Nur eine schwache Timestomping-Heuristik. |
| `Copied` | Änderungszeit liegt vor der Erstellungszeit. Oft mit einem Kopiervorgang vereinbar; kein Beweis für eine Handlung des Angreifers. |
| `SI-Flags` | Dateiattribute/Flags von `$STANDARD_INFORMATION`. Hidden-/System-Flags können nützliche Ansatzpunkte sein. |
| `NameType` | Namensraum von `$FILE_NAME` (Win32, DOS, POSIX oder kombiniert). Verwenden, um Namen nicht doppelt zu zählen. |
| `DataModified (SI)`, `Accessed (SI)`, `MetadataChanged (SI)`, `Created (SI)` | SI-Zeiten. Übliche Zeitleistenfelder, aber Tools im Benutzermodus können mehrere davon ändern. |
| `DataModified (FN)`, `Accessed (FN)`, `MetadataChanged (FN)`, `Created (FN)` | FN-Zeiten. Werden anders aktualisiert und sind mit gewöhnlichen APIs im Allgemeinen schwerer zu ändern; dennoch nicht unveränderlich. |
| `UpdateSequenceNumber` | Letzte USN, die dem SI-Attribut des Datensatzes zugeordnet ist. Ansatzpunkt in `$J`, falls diese USN erhalten ist. |
| `LogfileSequenceNumber` | NTFS-`$LogFile`-Sequenznummer aus dem Header des FILE-Datensatzes; nützlich für die Transaktionsanalyse auf niedriger Ebene. |
| `SecurityID` | Index in NTFS-`$Secure`; für sich genommen keine Windows-SID. Mit der `SecurityID` von [`parse-ntfs secure-sds`](secure-sds.md) verknüpfen, um Besitzer und ACLs der Datei zu erhalten. |
| `ObjectID-FileDroid` | Objekt-ID / Kennung der verteilten Linkverfolgung, sofern vorhanden; mit Droids aus LNK/Jump List korrelieren. |
| `LoggedUtilStream` | Dekodierter Kontext von `$LOGGED_UTILITY_STREAM`, sofern unterstützt. Unbekannte Werte prüfen. |
| `ZoneID-Contents` | Residenter Inhalt von `Zone.Identifier`, etwa ZoneId und Download-URLs. Er kann fehlen, entfernt oder nicht resident sein. |
| `SourceFile` | Die Eingabe-`$MFT`; unerlässlich beim Zusammenführen von Hosts oder Volumes. |
| `ResidentDataBase64`, `ResidentDataHex`, `ResidentDataASCII` | Mit `-i`: der Inhalt des residenten `$DATA` der Zeile (der Datei oder des alternativen Datenstroms) als Base64, als Bytes (`41-42`) und als Text, wenn es druckbares ASCII ist. Leer, wenn die Daten nicht resident sind oder durch `-E`/`-M` herausgefiltert wurden. |
| `ParseErrors` | Beschädigungen oder nicht unterstützte Beziehungen, die diese Zeile betreffen. Diese Spalte beim Filtern niemals verwerfen. |
| `FromSlack` | Mit `-x`: `True` für einen aus dem Slack eines FILE-Datensatzes wiederhergestellten Namen. `EntryNum` und `SequenceNumber` bezeichnen die Datei, auf die der Name verwies (leer, wenn sein Eintragsheader überschrieben wurde), die wiederhergestellten Zeitstempel stehen in den `(FN)`-Spalten, und `InUse` ist `False`. |
| `SlackRecord`, `SlackOffset` | Mit `-x`: der FILE-Datensatz, dessen Slack den Namen enthielt, bezeichnet nach seiner Position in der `$MFT` statt nach der darin geschriebenen Datensatznummer (bei einem Erweiterungsdatensatz dessen eigene, obwohl seine Namen beim Basisdatensatz aufgeführt werden), sowie der Byte-Offset des Namens in der `$MFT`. `ParseErrors` einer Slack-Zeile sind die Probleme dieses Datensatzes, etwa `fixup values do not match`: Die wiederhergestellten Bytes können betroffen sein. |
| `CopyOfEntryInUse` | Mit `-x`: Eine verwendete Datei hat diesen Namen noch im selben Verzeichnis (und ist dieselbe Datei, sofern die Referenz erhalten ist). Interessant sind die Namen, die keine verwendete Datei trägt: gelöscht, umbenannt oder der vorherige Inhaber eines wiederverwendeten Datensatzes. |

Die Zeitstempelspalten folgen der MACB-Reihenfolge (Modified, Accessed, Changed, Birth/Erstellung):
`DataModified` SI/FN, `Accessed` SI/FN, `MetadataChanged` SI/FN, dann `Created` SI/FN.

`(SI)` bedeutet `$STANDARD_INFORMATION`; `(FN)` bedeutet `$FILE_NAME`. Diese Namen beschreiben die
übliche Quelle bei einem vollständigen Datensatz. Bei einer normalen Dateinamenzeile ohne SI-Attribut legt der
bestehende Fallback die FN-Erstellungszeit in `Created (FN)` und die FN-Werte für Änderung/Datensatzänderung/Zugriff
jeweils in `DataModified (SI)`, `MetadataChanged (SI)` und `Accessed (SI)` ab.
Wiederhergestellte Slack-Zeilen legen alle vier Zeiten in die `(FN)`-Spalten und lassen die SI-Spalten leer.
Die Fallback-Werte normaler Zeilen sind **keine** unabhängigen SI-Beweise. Verwenden Sie die ursprünglichen JSON-Attribute
und die Slack-Herkunft, um die Quelle eines Zeitstempels zu bestimmen.

`-t ys-ntfs-parser-csv` (früher `ys-mft-parser-csv`, weiterhin akzeptiert) verwendet das eintragsorientierte Schema
des gepflegten Parsers. Diese lesbaren Spaltennamen gelten für die Standardausgabe `-t csv`,
einschließlich `MFT.csv` von `auto-parse`. JSON/JSONL bewahrt jedes
Attribut (einschließlich residentem `$DATA`, hexadezimal) sowie `FullPath`; verwenden Sie es, wenn die flache
Untersuchungs-CSV Strukturen auf niedriger Ebene auslässt. Mit `-x` hat jeder JSON-Eintrag zusätzlich
`SlackFileNames`.

## Untersuchungsstrategie { #investigation-strategy }

1. Filtern Sie ein Zeitfenster, behalten Sie aber gelöschte Datensätze und beide Zeitstempelsätze bei.
2. Durchsuchen Sie vollständige Pfade, verdächtige doppelte Erweiterungen, ADS, Staging-Verzeichnisse und bekannte Hashes/Namen.
   Beziehen Sie mit `-x` die Slack-Namen ein, die keine verwendete Datei trägt (`CopyOfEntryInUse` `False`); mit
   `-i -E ps1,bat,vbs,js` lesen Sie kleine Skripte direkt aus der CSV.
3. Verwenden Sie Eintrag/Sequenz und `UpdateSequenceNumber` als Ansatzpunkt im USN-Journal.
4. Vergleichen Sie MFT-Referenzen aus Prefetch/LNK/Jump List mit dem Zieldatensatz.
5. Korrelieren Sie Löschungen mit der Defender-Quarantäne oder USN-Aktivität zu `FILE_DELETE`/Umbenennung.

## Weitere Informationen { #further-details }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [mft-Crate von ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/mft)
- [mft-Parser von Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [Formatdokumentation von libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)
