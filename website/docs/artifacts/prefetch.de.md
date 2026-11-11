# Prefetch { #prefetch }

## Was Prefetch ist { #what-prefetch-is }

Windows-Prefetch-Dateien zeichnen Informationen auf, die zur Optimierung des Anwendungsstarts dienen. Eine programmspezifische
`.pf`-Datei speichert typischerweise den Namen und Hash der ausführbaren Datei, den Ausführungszähler, die letzten Ausführungszeiten,
Volume-Informationen sowie Verzeichnisse und Dateien, auf die beim Start verwiesen wurde.

Bei unterstützten Windows-Versionen ist ein gültiger Prefetch-Datensatz einer Anwendung in der Regel ein starker Beweis dafür,
dass die ausführbare Datei ausgeführt wurde. Er ist eines der nützlichsten Ausführungsartefakte außerhalb der Ereignisprotokolle.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Die genannte ausführbare Datei wurde zu den
aufgezeichneten letzten Ausführungszeitpunkten gestartet, vorbehaltlich des Betriebssystemverhaltens und der Gültigkeit des Parsings.

## Speicherort und Sicherung { #location-and-collection }

```text
C:\Windows\Prefetch\*.pf
```

Juntei unterstützt die Formatversionen 17 bis 31 (Windows XP bis Windows 11), einschließlich komprimierter
Dateien ab Windows 8. Sichern Sie das gesamte Verzeichnis, damit Hashes der Namen ausführbarer Dateien und benachbarte Datensätze
verglichen werden können.

## Hinweise für die Analyse { #analyst-cautions }

- Ältere Windows-Versionen speichern nur einen historischen Ausführungszeitpunkt, neuere bis zu acht.
  `RunCount` kann die Anzahl der verfügbaren Zeitstempel übersteigen.
- Prefetch kann auf manchen Systemen bzw. in manchen Rollen deaktiviert, bereinigt, ausgeschlossen oder nicht verfügbar sein. Das Fehlen ist kein
  Beweis dafür, dass ein Programm nicht ausgeführt wurde.
- Listen geladener Dateien beschreiben Startabhängigkeiten und Zugriffe; sie beweisen nicht, dass jede aufgeführte Datei
  bösartig war, absichtlich geöffnet oder vollständig gelesen wurde.
- Der Name der ausführbaren Datei im Prefetch-Header ist in der Länge begrenzt; Juntei ermittelt einen Pfad aus den geladenen
  Dateien, sofern dies eindeutig möglich ist.
- Die Dateisystem-Zeitstempel einer `.pf`-Quelldatei sind unabhängig von den eingebetteten Ausführungszeiten.
- `Findings` sind Beobachtungen für das Threat Hunting. Installationsprogramme und legitime Administration nutzen häufig Staging-Pfade.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pf files
  -f, --file <FILE>      Parse one Prefetch file

Records:
  -T, --timeline  Write one row per retained execution time, sorted by time

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file prefetch -d Prefetch/ -o prefetch.csv
juntei parse-file prefetch -d Prefetch/ -T -o prefetch-timeline.csv
juntei parse-file prefetch -d Prefetch/ -t jsonl \
  | jq 'select(.ExecutableName? | test("powershell|rundll32"; "i"))'
```

## Standard-CSV-Felder { #standard-csv-fields }

| Feld | Interpretation |
|---|---|
| `SourceFile` | Eingabe-`.pf`; behalten Sie dieses Feld bei, um Hosts und doppelte Namen ausführbarer Dateien zu unterscheiden. |
| `ExecutableName` | Name der ausführbaren Datei aus dem Header. Für die meisten Suchen ohne Beachtung der Groß-/Kleinschreibung verwenden; die ursprüngliche Schreibweise bewahren. |
| `ExecutablePath` | Von Juntei nach Möglichkeit ermittelter Ladepfad. Gleichen Sie ihn mit `Files` und anderen Artefakten ab. |
| `PrefetchHash` | Hash, den Windows im `.pf`-Dateinamen verwendet; kein kryptografischer Hash des Dateiinhalts. |
| `FormatVersion` | Version der Prefetch-Struktur; bestimmt die verfügbaren Felder und die Anzahl der Zeitstempel. |
| `RunCount` | Von Windows aufgezeichneter kumulativer Zähler. Nützlich für die Häufigkeit, nicht für den genauen Verlauf. |
| `LastRun` | Zuletzt gespeicherter Ausführungszeitpunkt. |
| `PreviousRun1` … `PreviousRun7` | Frühere gespeicherte Ausführungszeitpunkte, vom neuesten zum älteren. Leer, wenn Format oder Verlauf sie nicht enthalten. |
| `VolumeCount` | Anzahl der aufgezeichneten Quell-Volumes. |
| `VolumeDevicePaths` | NT-Gerätepfade der beim Start verwendeten Volumes. Mit den Seriennummern korrelieren. |
| `VolumeSerialNumbers` | Volume-Kennungen; nützlich, um Beweise zu Wechseldatenträgern oder neu formatierten Laufwerken zu verknüpfen. |
| `VolumeCreated` | Aufgezeichnete Erstellungszeitpunkte der Volumes, nicht der ausführbaren Datei. |
| `DirectoryCount` | Anzahl der aufgezeichneten Verzeichniszeichenfolgen. |
| `FileCount` | Anzahl der aufgezeichneten Verweise auf geladene Dateien. |
| `Findings` | Beobachtungen von Juntei, etwa verschobene System-Binärdateien oder verdächtige Ausführungspfade. Manuell prüfen. |
| `Files` | Geladene bzw. referenzierte Dateien mit verfügbaren NTFS-Referenzen. Hervorragender Ansatzpunkt für DLLs/Pfade, aber kein Beweis für eine Absicht bezüglich der Datei. |
| `Dir` | Mit dem Start verknüpfte Verzeichniszeichenfolgen. Nützlich, wenn einzelne Pfade geladener Dateien unvollständig sind. |

## Zeitleisten-CSV-Felder (`-T`) { #timeline-csv-fields-t }

| Feld | Interpretation |
|---|---|
| `RunTime` | Ein gespeicherter Prefetch-Ausführungszeitpunkt. |
| `ExecutableName`, `ExecutablePath` | Identität des Programms bzw. Pfadkontext. |
| `PrefetchHash` | Unterscheidet Varianten/Pfade, die denselben Namen der ausführbaren Datei haben. |
| `RunCount` | Zähler aus der Prefetch-Datei; er wird in jeder Zeitleistenzeile wiederholt. |
| `SourceFile` | Quell-`.pf` zur Überprüfung. |

JSON/JSONL bewahrt Volumes, Verzeichnisse, geladene Dateien und NTFS-Dateireferenzen als strukturierte
Arrays statt als kommagetrennte CSV-Zellen.

## Untersuchungsstrategie { #investigation-strategy }

1. Verwenden Sie `-T`, um gespeicherte Ausführungen in eine Zeitleiste aufzunehmen.
2. Gruppieren Sie nach Name der ausführbaren Datei plus Prefetch-Hash; derselbe Name kann unterschiedliche Pfade/Hashes haben.
3. Prüfen Sie `ExecutablePath`, Staging-Pfade, Findings zu verschobenen System-Binärdateien und geladene DLLs.
4. Gleichen Sie MFT-Referenzen und Dateipfade der Zieldateien mit `$MFT`-/USN-Datensätzen ab.
5. Korrelieren Sie Ausführungszeiten mit Ereignisprotokollen, SRUM, Netzwerktelemetrie, Diensten, Aufgaben und Benutzerartefakten.

## Weitere Informationen { #further-details }

- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch)
- [libyal libscca](https://github.com/libyal/libscca)
- [Security Ronin prefetch-forensic](https://github.com/SecurityRonin/prefetch-forensic)
