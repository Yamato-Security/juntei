# Jump Lists { #jump-lists }

## Was Jump Lists sind { #what-jump-lists-are }

Jump Lists liefern die Menüs, die an Anwendungseinträgen in der Taskleiste und im Startmenü angezeigt werden. Automatische Jump
Lists sind von Windows verwaltete Verläufe der von einer Anwendung verwendeten Elemente. Benutzerdefinierte Jump Lists sind
von der Anwendung festgelegte Kategorien und Aufgaben. Beide können LNK-Strukturen mit Zielpfaden, Zeiten,
Volume-/Netzwerkkontext, MFT-Referenzen und Startargumenten einbetten.

Automatische Datensätze können von einer Anwendung geöffnete Dateien oder Ordner, Reihenfolge/Zeitpunkt der letzten Nutzung,
Interaktionsanzahl und Anheftstatus offenlegen. Benutzerdefinierte Listen können „zuletzt geschlossene“ Seiten oder Anwendungs-
aufgaben zeigen – einschließlich bösartiger Befehlsargumente.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Die zugehörige Anwendung hat das Element bzw. die Aufgabe
in ihrer Jump List aufgezeichnet oder angeboten. Das Verhalten unterscheidet sich je nach Anwendung; untermauern Sie tatsächliche Ausführung bzw. tatsächlichen Zugriff.

## Speicherorte und Sicherung { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

Das hexadezimale Präfix des Dateinamens ist eine AppID. Sichern Sie beide Verzeichnisse und bewahren Sie den Pfad des Benutzerprofils.
Automatische Listen sind OLE-Compound-File-Container mit einem `DestList`-Stream und nummerierten LNK-
Streams; benutzerdefinierte Listen verwenden ein eigenes Kategorie-/Link-Format.

## Hinweise für die Analyse { #analyst-cautions }

- AppID-Beschreibungen stammen aus einer bekannten Zuordnung; eine unbekannte AppID ist für sich genommen nicht verdächtig.
- `LastModified` / Felder der letzten Nutzung gehören zu einem DestList-Eintrag; eingebettete Zielzeiten sind etwas anderes.
- Interaktionsanzahlen gibt es nur in unterstützten Versionen, und sie sind keine vollständige Anzahl der Öffnungen.
- Angeheftete Elemente bleiben über die normale Aktualität hinaus erhalten; Bereinigung oder Anwendungseinstellungen können andere entfernen.
- Eine Liste kann veraltete/nicht zugeordnete Streams oder Bytes aus einer früheren, längeren benutzerdefinierten Liste enthalten.
- Eingebettete LNK-Argumente und -Pfade sind vom Angreifer kontrollierbar. Die Zuordnung zu einer Anwendung ist Kontext, kein Vertrauensnachweis.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-list-csv
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

Mit `auto-parse` heißen die CSV-Tabellen `FileAndFolderOpening/JumpLists-AutomaticDestinations.csv`
und `FileAndFolderOpening/JumpLists-CustomDestinations.csv`, direkt unter jedem Computer.
Das gemeinsame Protokoll ist `JumpLists-logs.txt` in dieser Kategorie. Die Manifeste listen die exakten Pfade
in `OutputFiles` auf. Eigenständige CSV-Exporte behalten `AutomaticDestinations.csv` und
`CustomDestinations.csv` in dem von Ihnen gewählten Verzeichnis bei. Automatische JSON- und JSONL-Exporte sind
einzelne Dateien `JumpLists.json` oder `JumpLists.jsonl`; die Datensatzstruktur ist unverändert.

## AutomaticDestinations-CSV-Felder { #automaticdestinations-csv-fields }

| Feldgruppe | Felder und Interpretation |
|---|---|
| Quelle | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: der Jump-List-Container und seine Dateisystemzeiten. |
| Anwendung | `AppId`, `AppIdDescription`: AppID aus dem Dateinamen und bekannte Anwendungszuordnung. Leiten Sie aus einem unbekannten Hash keine Identität ab. |
| Container | `HasSps`, `DestListVersion`, `LastUsedEntryNumber`: DestList-Struktur/-Version und Header-Kontext. |
| Reihenfolge | `MRU`, `EntryNumber`: Position der jüngsten Nutzung und hexadezimale Eintrags-/Stream-Kennung. |
| Nutzungsmetadaten | `CreationTime`, `LastModified`, `InteractionCount`, `PinStatus`, `PinPosition`: Lebenszyklus-/Nutzungs-/Anheftstatus aus der DestList. Je nach Version interpretieren. |
| Host/Pfad | `Hostname`, `MacAddress`, `Path`, `DisplayPath`: Zielort und Verfolgungskontext. Host/MAC ist keine alleinige Zuordnung. |
| Verfolgungs-IDs | `FileBirthDroid`, `FileDroid`, `VolumeBirthDroid`, `VolumeDroid`: korrelieren verschobene Ziele und die Verfolgung eingebetteter LNKs. |
| Hinweise | `Notes`: informiert über nicht zugeordnete Streams, wiederhergestellte Überreste oder anderen Parserkontext. |
| Eingebettete LNK | `TargetCreated`, `TargetModified`, `TargetAccessed`, `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`, `DriveType`, `VolumeSerialNumber`, `VolumeLabel`, `LocalPath`, `CommonPath`, `TargetIDAbsolutePath`, `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`, `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `ExtraBlocksPresent`, `Args`: wie unter [LNK-Verknüpfungen](lnk.md) beschrieben interpretieren. |
| Zusätzliche LNK-Felder | `NetworkPath`, `Name`, `IconLocation`: Netzwerk- und Darstellungsfelder aus der eingebetteten Verknüpfung. |
| Validierung | `Problems`: fehlende Streams, Kürzung, abweichende Anzahlen oder beschädigte Links. |

## CustomDestinations-CSV-Felder { #customdestinations-csv-fields }

Die benutzerdefinierte Ausgabe beginnt mit `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`, `AppId`,
`AppIdDescription` und `EntryName` (Kategoriename oder `Tasks`), gefolgt von allen oben aufgeführten eingebetteten LNK-Feldern,
dann `Notes`, `NetworkPath`, `Name`, `IconLocation` und `Problems`.

JSON/JSONL bewahrt zusätzlich den DestList-Header, Score/Prüfsumme/Property Store pro Eintrag,
`DestListPropertyStore`, Compound-File-Streams, benutzerdefinierte Kategorien, Bytes nach der letzten Kategorie
und die verschachtelten Felder jeder eingebetteten LNK.

## Untersuchungsstrategie { #investigation-strategy }

1. Ermitteln Sie Benutzer und Quellprofil und ordnen Sie die AppID dann vorsichtig einer Anwendung zu.
2. Sortieren Sie automatische Einträge nach `LastModified` und `MRU`; trennen Sie angeheftete Elemente ab.
3. Prüfen Sie Pfade, Netzwerkfreigaben, Seriennummern von Wechselvolumes, MFT-Referenzen und Argumente.
4. Untersuchen Sie `Notes` sowie nicht zugeordnete/übrig gebliebene Links – gelöschte oder frühere Elemente können dort überdauern.
5. Korrelieren Sie mit RecentDocs, eigenständigen LNKs in „Recent“, Prefetch, SRUM, `$MFT` und Ereignisprotokollen.

## Weitere Informationen { #further-details }

- [Microsoft: Compound Binary File Format (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft: Shell-Link-Format (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [JLECmd von Eric Zimmerman](https://github.com/EricZimmerman/JLECmd)
- [JumpList-Parser als Test-/Referenzprojekt](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
