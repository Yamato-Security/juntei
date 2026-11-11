# Papierkorb { #recycle-bin }

## Was der Papierkorb aufzeichnet { #what-the-recycle-bin-records }

Wenn ein Benutzer im Datei-Explorer eine Datei oder einen Ordner löscht, verschiebt Windows das Element in den Papierkorb
seines Volumes und legt daneben einen kleinen Indexdatensatz an: den bisherigen Pfad, die Größe und den Zeitpunkt der
Löschung. Jeder Benutzer hat auf jedem Volume einen eigenen Papierkorbordner, der nach seiner SID benannt ist; der Ordner
gibt also auch an, welches Konto das Element gelöscht hat.

| Windows-Version | Index | Gelöschter Inhalt |
|---|---|---|
| Vista und neuer | Eine `$I…`-Datei pro Element (Formatversion 1 unter Vista bis 8.1, Version 2 ab Windows 10) | `$R…` mit demselben Namen nach `$R`: die Datei bzw. der Ordner mit seinem Inhalt |
| NT 4.0, 2000, XP, Server 2003 | Eine `INFO2`-Datei pro Papierkorb: 800-Byte-Datensätze mit einem ANSI- und einem Unicode-Pfad | `D<drive letter><index><extension>`, etwa `Dc1.txt` |
| 95, 98, ME | Eine `INFO2`-Datei (`INFO` unter Windows 95): 280-Byte-Datensätze, nur mit einem ANSI-Pfad | Dieselben `D…`-Namen |

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Das Konto, in dessen Papierkorb der Datensatz liegt,
hat das Element unter dem ursprünglichen Pfad zum aufgezeichneten Zeitpunkt in den Papierkorb verschoben. Wer mit
Umschalt+Entf, über die Befehlszeile oder aus den meisten Programmen heraus löscht, umgeht den Papierkorb, und es entsteht kein
Datensatz.

## Speicherorte und Sicherung { #locations-and-collection }

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

Sichern Sie die `$I`-Dateien bzw. INFO2 aus dem Papierkorb jedes Volumes. Werden auch die `$R`- und `D…`-Dateien
gesichert, ist erkennbar, welche gelöschten Elemente noch vorhanden sind; ohne sie ist `RecycledFileFound` für jedes
Element `false`. Der gelöschte Inhalt ist ein eigenständiges Beweismittel und kann Schadsoftware sein: Behandeln Sie ihn
entsprechend.

## Hinweise für die Analyse { #analyst-cautions }

- `DeletedTime` ist der Zeitpunkt, zu dem das Element in den Papierkorb verschoben wurde, gemessen an der Systemuhr und
  gespeichert als UTC-FILETIME. Es ist nicht der Zeitpunkt, zu dem der Papierkorb geleert wurde. Die eigenen Erstellungs-
  und Änderungszeiten des Elements stehen in `$MFT` und in der `$R`-Datei.
- Die SID bezeichnet das Konto, das das Element gelöscht hat, nicht unbedingt den Besitzer der Datei. Eine SID ohne Profil
  in der `ProfileList` des SOFTWARE-Hives (ein gelöschtes Konto oder ein Domänenkonto, das sich nie angemeldet hat) hat
  keinen `User`.
- Wird der Papierkorb geleert oder ein Element wiederhergestellt oder aus dem Papierkorb gelöscht, löscht Windows ab Vista
  dessen `$I`- und `$R`-Datei. Ihre Namen können im USN-Journal, in `$MFT` und im `$I30`-Slack verbleiben; Juntei stellt
  daraus keine gelöschten `$I`-Datensätze wieder her. INFO2 behält solche Datensätze und setzt das erste Byte ihres
  ANSI-Pfads auf null: `RemovedFromBin` ist `true`, und Juntei stellt dieses Zeichen anhand der Laufwerksnummer wieder her.
- `FileSize` ist in einer `$I`-Datei die Größe des Elements, bei einem Ordner die Gesamtgröße seines Inhalts. In INFO2
  ist es der Platz, den das Element auf dem Datenträger belegte, in ganzen Clustern (4.096 Byte bei einer kleinen Datei).
- Der ANSI-Pfad in INFO2 liegt oft in 8.3-Form vor und erfordert die Codepage des Systems: Verwenden Sie `--code-page 932`
  für japanisches Windows, 936 oder 950 für chinesisches. Der Unicode-Pfad steht, sofern vorhanden, in
  `OriginalPath`.
- Einige Systeme mit Windows Vista schrieben 543 Byte große `$I`-Dateien, deren Größenfeld ein Byte zu kurz ist. Juntei
  liest daraus Zeit und Pfad, lässt `FileSize` leer und meldet ein Problem.
- `RecycledFileFound` beschreibt die Sicherung, nicht den Endpunkt: `false` kann bedeuten, dass das Sicherungswerkzeug
  keine `$R`-Dateien erfasst hat oder dass sich das Element nicht mehr im Papierkorb befindet.
- Papierkörbe auf Volumes ohne Windows-Installation übernehmen die Benutzernamen von der einzigen Windows-Installation
  der Sicherung, sofern es genau eine gibt.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

Mit `-d` liest Juntei Dateien, deren Name aus `$I` und sechs Buchstaben oder Ziffern besteht (mit beliebiger
Erweiterung), außerdem `INFO2` sowie `INFO` in einem `RECYCLED`-Ordner. Mit `-f` liest es die Datei unabhängig von
ihrem Namen.

## CSV-Ausgabefelder { #csv-output-fields }

| Feld | Bedeutung |
|---|---|
| `SourceFile` | Die `$I`- oder INFO2-Datei. |
| `DeletedTime` | Zeitpunkt, zu dem das Element in den Papierkorb verschoben wurde (UTC). |
| `OriginalPath` | Wo sich das Element befand. Bei INFO2 der Unicode-Pfad, sofern der Datensatz einen hat, sonst der ANSI-Pfad. |
| `FileSize` | `$I`: die Größe des Elements in Byte (bei einem Ordner die Gesamtgröße). INFO2: sein Platz auf dem Datenträger, in ganzen Clustern. Leer, wenn sie nicht lesbar ist. |
| `User`, `SID` | Die SID des Papierkorbordners und der Benutzer, nach dem das zugehörige Profil in `ProfileList` benannt ist. |
| `RecycledFile`, `RecycledFileFound` | Der Pfad des gelöschten Inhalts neben dem Index (`$R…`, `D<drive><index><extension>`) und ob er in der Sicherung enthalten ist. |
| `Format`, `FormatVersion` | `$I` 1 (Vista bis 8.1) oder 2 (ab Windows 10); `INFO2` 0 (95), 2 (NT 4.0), 4 (98) oder 5 (ME, 2000 bis Server 2003). |
| `RecordIndex`, `Drive`, `RecordOffset` | INFO2: die Nummer des Datensatzes (die Zahl in seinem `D…`-Dateinamen), das Laufwerk und die Stelle, an der der Datensatz in INFO2 beginnt. Ein UNC-Pfad hat kein Laufwerk. |
| `RemovedFromBin` | INFO2: `true`, wenn das Element wiederhergestellt oder aus dem Papierkorb gelöscht wurde. |
| `ShortPath` | INFO2: der ANSI-Pfad, wenn er von `OriginalPath` abweicht (oft 8.3-Namen). |
| `Problems` | Beschädigungen: eine abgeschnittene oder überlange Datei, eine ungültige Laufwerksnummer, ein Pfad, der kein gültiges UTF-16 ist, eine Löschzeit, die keine gültige Zeitangabe ist. |

JSON und JSONL haben dieselben Felder, mit typisierten Zahlen und booleschen Werten, `null` für Werte, die ein Datensatz
nicht hat, und `Problems` als Liste.

## Automatisches Parsen { #automatic-parsing }

`auto-parse` schreibt `FileExistence/RecycleBin.csv` (`RecycleBin.json` bzw. `RecycleBin.jsonl` mit
`-t json` bzw. `-t jsonl`), zusammen mit `FileExistence/RecycleBin-logs.txt`. Seine Option `--code-page` gilt für
INFO2. Aus Datenträgerabbildern werden `$I`-Dateien und INFO2 kopiert; `$R`- und `D…`-Dateien sind leere
Platzhalter, sodass `RecycledFileFound` trotzdem gefüllt wird. Die kombinierte Zeitleiste `-j` enthält jede Zeile mit
dem `Artifact`-Bezeichner `recycle-bin`, `-Z` konvertiert `DeletedTime`, und
[`super-timeline`](../commands/super-timeline.md) erzeugt pro Element ein Ereignis `Recycle Bin deletion (moved to the bin)`.

## Untersuchungsstrategie { #investigation-strategy }

1. Sortieren Sie pro Benutzer nach `DeletedTime`, und vergleichen Sie mit den Zeiten der untersuchten Aktivität.
2. Suchen Sie `OriginalPath` in LNK, Jump Lists, ShellBags und RecentDocs, um festzustellen, ob das Element vor dem
   Löschen verwendet wurde.
3. Im USN-Journal erscheint die Löschung als Umbenennung nach `$Recycle.Bin` ungefähr zum Zeitpunkt `DeletedTime`;
   `$MFT` liefert die eigenen Zeiten des Elements.
4. Wenn `RecycledFileFound` `true` ist, hashen und untersuchen Sie den Inhalt von `$R` bzw. `D…`.
5. Wurde der Papierkorb geleert, durchsuchen Sie das USN-Journal und `$MFT` nach `$I`- und `$R`-Namen.

## Weitere Informationen { #further-details }

- [libyal dtformats — Windows Recycle.Bin file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — Recycle Bin and INFO2 parsers](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
