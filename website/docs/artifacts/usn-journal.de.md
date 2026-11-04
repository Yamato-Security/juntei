# USN-Journal (`$UsnJrnl:$J`) { #usn-journal-usnjrnlj }

## Was das USN-Journal ist { #what-the-usn-journal-is }

NTFS zeichnet Änderungen an Dateien und Verzeichnissen im Update Sequence Number (USN) Change Journal auf. Ein
Datensatz kann Erstellung, Erweiterung/Überschreiben von Daten, Attribut- oder Sicherheitsänderungen, Umbenennung,
Schließen und Löschung beschreiben. Er speichert eine Dateireferenz, eine Referenz auf das übergeordnete Verzeichnis, einen Namen, Gründe, Attribute,
einen Zeitstempel und eine monoton steigende USN.

Das Journal eignet sich hervorragend, um eine Abfolge von Dateioperationen eines Angreifers zu rekonstruieren, insbesondere wenn
die Dateien gelöscht wurden. Es zeichnet auf, **dass sich Metadaten/Inhalte geändert haben**, nicht die geänderten Bytes.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> NTFS hat die angegebenen Änderungsgründe für
die referenzierte Datei zum angegebenen Zeitstempel aufgezeichnet. Gründe sammeln sich häufig bis zu einem `CLOSE`-Datensatz an.

## Speicherort und Sicherung { #location-and-collection }

```text
C:\$Extend\$UsnJrnl:$J
```

`$J` ist ein benannter Stream der NTFS-Metadaten. Sichern Sie ihn mit einem forensischen Werkzeug mit Rohzugriff. Sichern Sie außerdem die
`$MFT` desselben Volumes, da USN-Datensätze Dateireferenzen auf das übergeordnete Verzeichnis statt vollständiger Pfade speichern.

## Hinweise für die Analyse { #analyst-cautions }

- Das Journal ist zirkulär und größenbeschränkt. Alte Datensätze werden überschrieben.
- `FILE_DELETE` besagt, dass NTFS die Löschung verarbeitet hat; der Inhalt kann an anderer Stelle wiederherstellbar sein.
- Ein Grund wie `DATA_OVERWRITE` verrät weder die Bytes noch den verantwortlichen Prozess oder Benutzer.
- Mehrere Gründe können in einem Datensatz kombiniert sein; `CLOSE` markiert häufig das Ende angesammelter Änderungen.
- Pfade werden rekonstruiert. Übergeordnete Verzeichnisse können umbenannt, gelöscht oder wiederverwendet worden sein.
- Die Sequenznummern der Dateireferenzen sind wichtig. Die Eintragsnummer allein kann zur falschen Generation führen.
- Carving kann gültige Datensätze ohne ihren ursprünglichen Journalkontext wiederherstellen; weisen Sie diese Unterscheidung im Bericht aus.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-ntfs usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $J files
  -f, --file <FILE>      Parse one $J, or any file with --carve
  -m, --mft <FILE>       Use the matching volume's $MFT for paths
      --carve            Search a disk image/unallocated file for records

Output:
  -o, --output <PATH>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-ntfs usn-journal -d evidence/ -o usn-jsonl -t jsonl
juntei parse-ntfs usn-journal -f unallocated.bin --carve -o carved-usn.csv
```

## CSV-Ausgabefelder { #csv-output-fields }

| Feld | Interpretation |
|---|---|
| `Name` | In diesem USN-Datensatz gespeicherter Datei-/Verzeichnisname. Er ist kein vollständiger Pfad. |
| `Extension` | Aus `Name` abgeleitete Erweiterung zur Vereinfachung; nicht als vertrauenswürdiger Inhaltstyp verwenden. |
| `EntryNumber`, `SequenceNumber` | NTFS-Referenz der Datei. Verwenden Sie beide, um die Generation des Datensatzes zu identifizieren. |
| `ParentEntryNumber`, `ParentSequenceNumber` | NTFS-Referenz des übergeordneten Verzeichnisses zum Zeitpunkt der Änderung. |
| `ParentPath` | Von Juntei zeitbezogen rekonstruierter Pfad des übergeordneten Verzeichnisses. Unbekannt-Markierungen bedeuten, dass die Herkunft nicht aufgelöst werden konnte. |
| `UpdateSequenceNumber` | USN-Ordnungswert. Nützlich für stabile Sortierung und als Ankerpunkt zu Amcache/MFT. |
| `UpdateTimestamp` | In V2-/V3-Datensätzen gespeicherter Zeitstempel. Behandeln Sie Zeitstempel aus gecarvtem Kontext mit derselben Validierung wie jeden anderen Datensatz. |
| `UpdateReasons` | Ein oder mehrere benannte Änderungs-Flags, verbunden durch `|`. Interpretieren Sie Kombinationen und `CLOSE` gemeinsam. |
| `FileAttributes` | Bei der Änderung aufgezeichnete NTFS-Attribute; unbekannte Bits bleiben hexadezimal. |
| `OffsetToData` | Byte-Offset in der Eingabe. Unverzichtbar für die Validierung und für gecarvte Datensätze. |
| `SourceFile` | Eingabejournal/-abbild; unterscheidet Hosts und Volumes. |
| `SourceInfo` | Quell-Flags, die beschreiben, wie die Änderung in das Journal gelangt ist. In der Regel Kontext, keine Zuordnung. |
| `SecurityID` | NTFS-Index der Sicherheitskennung für den Datensatz, keine Benutzer-SID. |
| `Version` | Version des USN-Datensatzes. Juntei schreibt V2/V3 in die CSV; JSON behält die Bereichsverfolgungs-Extents von V4 bei. |

JSON/JSONL stellt 128-Bit-Referenzen von V3 korrekt dar und bewahrt V4-Datensätze zur Bereichsverfolgung und
ihre Extents. Verwenden Sie es, wenn versionsspezifische Felder auf niedriger Ebene von Bedeutung sind.

## Untersuchungsstrategie { #investigation-strategy }

1. Filtern Sie nach Zeit, Name, Erweiterung, übergeordnetem Pfad oder Dateireferenz.
2. Sortieren Sie zusätzlich zur Zeit nach `UpdateSequenceNumber`, um die exakte Reihenfolge im Journal zu sehen.
3. Ordnen Sie `RENAME_OLD_NAME` und `RENAME_NEW_NAME` einander zu, um Verschiebungen/Umbenennungen zu erkennen.
4. Validieren Sie Zustand, Pfade, Größen und Zeitstempel der Datensätze mit der `$MFT`.
5. Korrelieren Sie die Zuordnung zu Prozessen aus Ereignisprotokollen, EDR, Prefetch, Diensten/Aufgaben oder Defender.

## Weitere Informationen { #further-details }

- [Microsoft: Änderungsjournale](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: Grund-Flags des Änderungsjournals](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2#members)
- [Crate usn von ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/usn)
- [Sootmark usn](https://github.com/Sootmark/usn)
