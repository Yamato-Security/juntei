# Bootsektor (`$Boot`) { #boot-sector-boot }

## Was `$Boot` ist { #what-boot-is }

`$Boot` ist MFT-Eintrag 7 jedes NTFS-Volumes: die ersten 8 KiB des Volumes. Sein erster Sektor, der
**Volume Boot Record**, enthält den BIOS-Parameterblock, der das Dateisystem beschreibt:

- die Sektor- und Clustergrößen, die Anzahl der Sektoren im Volume und die **verborgenen Sektoren**
  (die Sektoren auf dem Datenträger vor dem Volume, also die Stelle, an der seine Partition beginnt);
- wo `$MFT` und `$MFTMirr` beginnen, sowie die Größen der FILE- und INDX-Datensätze;
- die 64-Bit-**Volume-Seriennummer**, die beim Formatieren des Volumes festgelegt wird.

Der Rest des ersten Sektors und die 15 darauf folgenden Sektoren enthalten den **Bootcode**, der
Windows lädt.

Die unteren 32 Bit der Seriennummer sind das, was Windows als `XXXX-XXXX` anzeigt (`vol`, `dir`), und das, was
[LNK-Verknüpfungen](lnk.md), [Jump Lists](jump-lists.md) und [Prefetch](prefetch.md) über das
Volume aufzeichnen, auf dem sich eine Datei befand. `$Boot` verknüpft diese Datensätze mit einem Volume eines gesicherten Hosts.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Eine Volume-Seriennummer aus LNK, Jump List oder Prefetch,
die dem `VolumeSerialNumber` eines `$Boot` entspricht, verweist auf dieses Volume oder auf ein anderes Volume, das
dieselbe Seriennummer erhalten hat oder mit ihr kopiert wurde. Seriennummern lassen sich ändern und sind über Hosts hinweg nicht
eindeutig.

## Speicherort und Sicherung { #location-and-collection }

```text
C:\$Boot          (MFT entry 7; the first 8 KiB of the volume)
```

Sichern Sie die Datei mit einem forensischen Tool mit Rohzugriff oder lesen Sie den ersten Sektor eines Volume-Images: Beides sind
dieselben Bytes. Juntei liest jede Datei, die mit einem NTFS-Bootsektor beginnt; ein Volume-Image funktioniert daher
als Eingabe. Ein Datenträger-Image beginnt stattdessen mit der Partitionstabelle; übergeben Sie Juntei das Volume der Partition.

## Hinweise für die Analyse { #analyst-cautions }

- Die Seriennummer ändert sich, wenn das Volume formatiert wird. Eine LNK-Datei, deren Seriennummer zu
  keinem aktuellen Volume passt, kann auf Wechselmedien, den Server einer Netzwerkfreigabe oder ein inzwischen
  formatiertes Volume verweisen.
- Der Bootcode ist auf Volumes, die von derselben Windows-Version formatiert wurden, gleich: Er ist identisch in den
  `$Boot`-Dateien von Windows 10, 11 und Server 2012 R2 im DFIR Artifact Museum. Ein Volume-Boot-Record-Bootkit
  verändert ihn oder das Feld der verborgenen Sektoren, mit dem der Bootcode den Rest von
  `$Boot` findet. Vergleichen Sie `BootCodeSHA256`, `JumpInstruction` und `HiddenSectors` mit anderen Hosts und mit der
  Partitionstabelle, bevor Sie Schlüsse ziehen: Auch ein Bootmanager oder Verschlüsselungssoftware kann den Bootcode
  verändern.
- Ein Image eines mit BitLocker verschlüsselten Volumes beginnt mit dem eigenen Bootsektor von BitLocker (OEM-ID
  `-FVE-FS-`). Mit `-f` übergeben, meldet Juntei ihn, und `Problems` gibt an, dass es kein NTFS ist; der NTFS-Bootsektor
  befindet sich in den verschlüsselten Daten. Ein vom entsperrten Volume gesicherter `$Boot` ist der NTFS-Bootsektor.
- `Problems` listet erkannte strukturelle Inkonsistenzen und Kürzungen auf, statt einen vollständigen Test zu liefern, ob Windows das Volume einbinden könnte. Verwenden Sie es, um durch Carving gewonnene Sektoren und Kopien zu beurteilen, nicht als eigenständigen Angriffsbefund. Der Hash des Bootcodes schließt den anfänglichen Sprung, den BIOS-Parameterblock und die Endsignatur aus; prüfen Sie auch diese Felder.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      Parse one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

Die Zusammenfassung listet für jeden Bootsektor die Volume-Seriennummer (wie Windows sie anzeigt und wie LNK- und
Prefetch-Felder sie anzeigen), die Größe, die Clustergröße und den `$MFT`-Cluster auf. Werden mehrere Hosts geparst,
gruppiert sie diese nach dem Hash des Bootcodes, sodass ein Volume, dessen Bootcode von den anderen abweicht, auffällt.

## CSV-Ausgabefelder { #csv-output-fields }

| Feld | Interpretation |
|---|---|
| `SourcePath` | Eingabedatei. |
| `VolumeSerialNumber` | Die 32-Bit-Seriennummer, wie die Ausgabe von LNK, Jump List und Prefetch sie zeigt (`180B1E67`). Windows zeigt sie als `180B-1E67`. |
| `VolumeSerialNumber64` | Die vollständige 64-Bit-Seriennummer. |
| `OEM-ID` | `NTFS` auf einem NTFS-Volume. |
| `BytesPerSector`, `SectorsPerCluster`, `ClusterSize` | Die Geometrie des Volumes. Mehr als 128 Sektoren pro Cluster werden als Zweierpotenz gespeichert und hier dekodiert. Leer, wenn ungültig. |
| `TotalSectors`, `VolumeSize` | Sektoren im Volume und ihre Größe in Bytes, ohne die Kopie des Bootsektors an seinem Ende. |
| `HiddenSectors` | Sektoren auf dem Datenträger vor dem Volume: wo seine Partition beginnt. |
| `MFT-Cluster`, `MFT-Offset` | Wo `$MFT` beginnt, in Clustern und in Bytes ab dem Anfang des Volumes. |
| `MFT-MirrorCluster`, `MFT-MirrorOffset` | Wo `$MFTMirr` beginnt. |
| `FileRecordSize`, `IndexRecordSize` | Bytes pro FILE-Datensatz (meist 1.024) und pro INDX-Datensatz (meist 4.096). |
| `MediaDescriptor` | `F8` für eine Festplatte. |
| `JumpInstruction` | Die ersten drei Bytes (`EB 52 90`): ein Sprung über den BIOS-Parameterblock zum Bootcode. |
| `EndSignature` | `55 AA` am Ende eines gültigen Bootsektors. |
| `BootCodeSHA256` | SHA-256 des Bootcodes: die Bytes 0x54 bis 0x1FD des ersten Sektors und der Rest der 8 KiB. |
| `Problems` | Was Windows nicht einbinden würde: eine andere OEM-ID als `NTFS`, kein `55 AA`, ungültige Größen, gesetzte FAT-Felder, die NTFS auf null hält, `$MFT` hinter dem Ende des Volumes; sowie eine Datei, die kürzer als 8 KiB ist. |

JSON/JSONL haben dieselben Felder und Namen, mit Zahlen als Zahlen, `null` für unbekannte Werte und
`Problems` als Liste.

## Untersuchungsstrategie { #investigation-strategy }

1. Stellen Sie das `VolumeSerialNumber` jedes Hosts den Volume-Seriennummern seiner LNK-Dateien, Jump
   Lists und Prefetch-Dateien gegenüber: Diejenigen, die zu keinem gesicherten Volume passen, gehören zu anderen Volumes (USB-
   Laufwerke, Freigaben, inzwischen formatierte Volumes).
2. Suchen Sie die übrigen Seriennummern in der Ausgabe zu [USB-Geräten](usb.md), die die Volume-
   Seriennummern eingebundener Wechselvolumes auflistet.
3. Vergleichen Sie `BootCodeSHA256` und `HiddenSectors` über Hosts hinweg. Untersuchen Sie abweichenden Bootcode
   mit einem Disassembler und der Partitionstabelle des Datenträgers, und suchen Sie nach bootbezogenen Einträgen in Ereignisprotokollen.
4. Verwenden Sie `MFT-Offset` und `ClusterSize`, um `$MFT` in einem Volume-Image zu finden.

## Weitere Informationen { #further-details }

- [Microsoft: NTFS-Übersicht](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
- [libfsntfs: NTFS-Formatdokumentation](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (der Volume-Header)
- [The Sleuth Kit: fsstat](https://www.sleuthkit.org/sleuthkit/man/fsstat.html)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (dessen `boot`-Crate)
