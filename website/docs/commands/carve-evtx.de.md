# Ereignisprotokoll-Carving { #carve-evtx }

Ereignisprotokoll-Chunks wiederherstellen, die zurückbleiben, nachdem ein Protokoll geleert oder seine Datei gelöscht wurde: aus einem Disk-Image, nicht zugeordnetem Speicherplatz, einer Auslagerungsdatei oder einem Arbeitsspeicherabbild. Jeder 64 KiB große EVTX-Chunk enthält eigene Zeichenfolgen- und Vorlagentabellen, sodass sich seine Datensätze ohne den Rest des Protokolls lesen lassen. Juntei schreibt die wiederhergestellten Chunks als gewöhnliche `.evtx`-Dateien, eine pro Kanal, zur Auswertung mit Hayabusa oder der Ereignisanzeige.

```bash
juntei carve-evtx -f host.E01 -o carved/
juntei carve-evtx -f host.vhdx --free-space-only -o carved-free/
juntei carve-evtx -f pagefile.sys --live-logs evidence/C/Windows/System32/winevt/Logs/ -o carved-pagefile/
juntei carve-evtx -f memory.raw -o carved-memory/ -t jsonl
```

Ohne `-o` wird nur die Zusammenfassung angezeigt.

## Chunks finden { #finding-chunks }

- Die Chunk-Signatur wird an jedem Byte-Offset gesucht, und jeder Header muss plausibel sein (Headergröße, Datensatznummern, Offsets). Prüfsummen werden gemeldet, aber nicht vorausgesetzt: Ein teilweise überschriebener Chunk enthält noch lesbare Datensätze.
- Chunks NTFS-komprimierter Protokolle werden dekomprimiert (LZNT1, Kompressionseinheiten von 16 Clustern). Bei einem Disk-Image wird die Clustergröße des Volumes verwendet, bei anderen Dateien `--cluster-size <BYTES>` (Standard: 4096).
- Die Datensätze werden anhand ihrer Größe bis zum Offset des letzten Datensatzes laut Header durchlaufen; eine wiederholte Datensatz-ID beendet den Durchlauf nie. Nach einem beschädigten Datensatz wird beim nächsten gültigen Datensatz fortgefahren. Alte Datensätze im Slack des Chunks werden ebenfalls aufgeführt.

## Disk-Images und NTFS-Volumes { #disk-images }

Bei einem E01- oder VHDX-Image oder einem Raw-Image eines Datenträgers oder Volumes mit NTFS wird jeder Chunk anhand des Clusters eingeordnet, in dem er liegt:

| Ort | Bedeutung |
|---|---|
| `FreeSpace` | Ein Cluster, den keine Datei verwendet. |
| `DeletedLog` | Ein freier Cluster, den der `$MFT`-Datensatz eines gelöschten Protokolls noch aufführt, oder ein aus solchen Clustern neu aufgebautes Protokoll. |
| `LiveLog` | Ein Cluster eines Ereignisprotokolls, das in Verwendung ist. Ohne `--keep-live` ausgelassen. |
| `AllocatedFile` | Ein Cluster einer anderen Datei, etwa `pagefile.sys` oder der Speicherdatei einer Schattenkopie. |
| `AllocatedUnknown` | Ein zugeordneter Cluster, den kein `$MFT`-Datensatz aufführt. |
| `OutsideVolumes` | Außerhalb aller NTFS-Volumes des Datenträgers. |
| `Unclassified` | Die Eingabe enthält kein NTFS-Volume (eine Auslagerungsdatei oder ein Arbeitsspeicherabbild). |

Gelöschte Protokolle, deren `$MFT`-Datensatz nur freie Cluster aufführt, werden vollständig neu aufgebaut, auch komprimierte, und unter `Recovered/` gespeichert. `--free-space-only` durchsucht nur freie Cluster: Das ist schneller, überspringt aber die Auslagerungsdatei, Schattenkopien und andere Dateien.

## Deduplizierung { #deduplication }

Jeder Chunk wird nur einmal geschrieben. Ein Chunk wird ausgelassen, wenn er zu einem aktiven Protokoll gehört, wenn er mit einem anderen Chunk identisch ist oder wenn alle seine Datensätze bereits in den aktiven Protokollen oder in einem schon geschriebenen Chunk stehen. Chunks mit mehr Datensätzen werden zuerst berücksichtigt, sodass eine ältere, kürzere Kopie eines Chunks nichts hinzufügt. Bei einer Eingabe ohne NTFS geben Sie die aktiven Protokolle mit `--live-logs <DIR>` an.

## Ergebnisse { #results }

- `Carved/<CHANNEL>.evtx`: Ein Protokoll pro Kanal, benannt wie die Protokolldateien von Windows (`Microsoft-Windows-Sysmon%4Operational.evtx`). Die Datensatznummern in den Chunk-Headern werden neu vergeben und die Prüfsummen neu berechnet. Ein beschädigter Datensatz und alles danach werden zu Slack. Ansonsten bleibt jeder Chunk einschließlich seines Slacks vollständig erhalten. Hayabusas Option `-x` durchsucht als leer markierte Chunks. Datensätze im Slack teilweise belegter Chunks finden Sie in Junteis `records.csv`.
- `Recovered/Volume<N>/...`: Gelöschte Protokolle, die aus ihren `$MFT`-Datensätzen neu aufgebaut wurden.
- `chunks.csv` / `chunks.jsonl`: Jeder gefundene Chunk: wo er lag, was mit ihm geschah (`Status`), sein Kanal, Datensatz-IDs, Zeiten und Prüfsummenstatus.
- `records.csv` / `records.jsonl`: Jeder Datensatz der geschriebenen Chunks. `OnlyInCarvedData` kennzeichnet Datensätze, die in den aktiven Protokollen fehlen.

```bash
hayabusa csv-timeline -d carved/Carved -a -o carved-timeline.csv
```

Carving findet nur zusammenhängende Chunks. Ein Chunk, der über nicht zusammenhängende Cluster oder im Arbeitsspeicher über nicht zusammenhängende physische Seiten verteilt ist, ist unvollständig; seine späteren Datensätze können beschädigt sein oder fehlen. Dekomprimieren Sie `hiberfil.sys` vorher. Der Exit-Code ist 0, wenn die Suche abgeschlossen wurde, und 1 bei einem Fehler.

## Weitere Informationen { #further-details }

- [Ausführliche Nutzung und Feldreferenz (Englisch)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.md)
- [Ausführliche Nutzung und Feldreferenz (Japanisch)](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.ja.md)
- [EVTX-Format (libevtx-Dokumentation)](https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc)
