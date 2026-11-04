# LNK-Verknüpfungen { #lnk-shortcuts }

## Was LNK-Dateien sind { #what-lnk-files-are }

Windows-Shell-Link-Dateien (`.lnk`) beschreiben ein Ziel sowie Startargumente, Arbeitsverzeichnis, Symbol,
Fensterzustand, Zielmetadaten, Volume-/Netzwerkinformationen, Shell-Namespace-Elemente und optionale
Kennungen der verteilten Linkverfolgung (Distributed Link Tracking). Windows und Anwendungen erstellen sie unter „Zuletzt verwendet“,
auf dem Desktop, im Startmenü und an anderen Orten; Angreifer liefern bösartige Verknüpfungen auch direkt aus.

Eine LNK-Datei im Ordner „Recent“ kann eine Benutzerinteraktion mit einem Ziel belegen, das nicht mehr existiert, etwa auf einem
Wechseldatenträger oder einer Netzwerkfreigabe. Eine beliebige LNK-Datei kann existieren, ohne jemals angeklickt worden zu sein.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Die Verknüpfung wurde mit dem
aufgezeichneten Zielkontext erstellt/aktualisiert. Ihr Speicherort und die umgebenden Artefakte bestimmen, ob sie eine Nutzung belegt.

## Speicherorte und Sicherung { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\*.lnk
%USERPROFILE%\Desktop\*.lnk
%APPDATA%\Microsoft\Windows\Start Menu\Programs\...
%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\...
```

Durchsuchen Sie das gesamte Benutzerprofil sowie gesicherte Archive/E-Mail-Inhalte, nicht nur `Recent`. Bewahren Sie
die eigenen Zeitstempel und den Pfad der Quelldatei, denn der Speicherort verändert die Interpretation.

## Hinweise für die Analyse { #analyst-cautions }

- Zielzeiten, Größe, MFT-Referenz und Volume-Daten sind in der LNK-Datei gespeicherte Momentaufnahmen; sie sind nicht
  die eigenen Metadaten der Verknüpfungsdatei und können veraltet sein.
- `MachineID`, MAC und Erstellungszeit des Trackers beschreiben den Kontext der verteilten Linkverfolgung, in der Regel
  den Rechner, der den Link erstellt/aktualisiert hat – nicht notwendigerweise den Rechner, auf dem Sie ihn gefunden haben.
- Ein MAC-ähnlicher UUID-Knoten kann aus Virtualisierung, Randomisierung oder nicht netzwerkbezogenen Kennungen stammen.
- `Args` kann eine harmlose ausführbare Datei (`powershell.exe`, `rundll32.exe`, `mshta.exe`) gefährlich machen.
- ANSI-Zeichenfolgen erfordern die Codepage des Quellsystems; verwenden Sie `--code-page 932` für viele japanische Links.
- Eine Verknüpfung kann manipuliert oder kopiert sein. Setzen Sie ihre Felder nicht ohne Bestätigung durch Prefetch,
  Jump List, RecentDocs, Ereignisprotokolle oder andere Belege mit einer Ausführung gleich.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.lnk
  -f, --file <FILE>      Parse one shortcut
      --code-page <CP>   ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -d evidence/ --code-page 932 -o lnk-ja.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

## CSV-Ausgabefelder { #csv-output-fields }

| Feldgruppe | Felder und Interpretation |
|---|---|
| Quelle | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: Pfad der Verknüpfung und Zeitstempel des Host-Dateisystems. Diese sind von den Zielzeiten getrennt. |
| Zielzeiten | `TargetCreated`, `TargetModified`, `TargetAccessed`: in der LNK-Datei aufgezeichnete Momentaufnahmen; nützliche historische Hinweise, keine aktuellen Zielmetadaten. |
| Zielgrundlagen | `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`: aufgezeichneter Ziel-/Startkontext. Argumente können die Bedeutung ändern. |
| Volume | `DriveType`, `VolumeSerialNumber`, `VolumeLabel`: identifizieren festen/wechselbaren/Netzwerkkontext und korrelieren andere Verknüpfungen. |
| Pfade | `LocalPath`, `NetworkPath`, `CommonPath`, `TargetIDAbsolutePath`: einander ergänzende Pfaddarstellungen. Prüfen Sie Abweichungen und Shell-Namespace-Pfade. |
| Startverhalten | `Args`, `ShowWindow`, `HotKey`, `ShimLayer`, `DarwinId`: wie die Shell das Ziel startet oder konfiguriert. `Args` ist entscheidend. |
| NTFS-Identität | `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`: Zielreferenz zum Zeitpunkt der Link-Erstellung; gleichen Sie beide mit `$MFT` ab. |
| Verfolgung | `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `VolumeDroid`, `FileDroid`, `VolumeDroidBirth`, `FileDroidBirth`: verteilte Linkverfolgung. Zur Korrelation verwenden, nicht allein zur Zuordnung. |
| Zusätzliche Blöcke | `ExtraBlocksPresent`: Liste der vorhandenen optionalen Strukturen; erklärt, welche anderen Felder unterstützt werden. |
| Darstellung | `Name`, `IconLocation`, `IconIndex`, `IconEnvironment`, `EnvironmentVariables`: Anzeige- und umgebungsvariablen-expandierte Zieldaten. Angreifer können irreführende Namen/Symbole verwenden. |
| Shell-Ordner | `KnownFolder`, `SpecialFolder`: dekodierte Kennungen von Windows-Shell-Ordnern. |
| Netzwerk | `NetworkDevice`, `NetworkProvider`: Kontext zu Netzwerkfreigabe/-gerät und Anbieter. |
| Kodierung | `ConsoleCodePage`: aufgezeichnete Konsolen-Codepage, sofern vorhanden; unabhängig von `--code-page` des Parsers. |
| Nachgestellte Daten | `AppendedBytes`: Bytes nach dem logischen Ende. Unerwartete Daten können auf Schmuggel oder fehlerhafte Dateien hinweisen. |
| Validierung | `Problems`: Beschädigung, Kürzung, inkonsistente Blöcke oder Dekodierungseinschränkungen. In die Prüfung einbeziehen. |

JSON/JSONL bewahrt zusätzlich jedes Target-ID-Element (Art, Name, GUID, FAT-Zeiten und MFT-Referenz),
alle Extra-Data-Blöcke, Property-Store-Eigenschaften, Konsoleneinstellungen und rohe strukturelle
Beziehungen. Verwenden Sie es bei einer verdächtigen Verknüpfung, statt sich nur auf die flache CSV-Zeile zu verlassen.
Die Zeiten der verschachtelten `TargetItems` stammen aus FAT-Shell-Item-Uhren ohne gespeicherten Offset.
Ihre althergebrachte Darstellung mit `Z` beweist kein UTC; `auto-parse --timezone` bewahrt sie unverändert.
Die Ziel-FILETIMEs im LNK-Header und die Tracker-Zeiten bleiben umrechenbare absolute Zeitstempel.

## Untersuchungsstrategie { #investigation-strategy }

1. Untersuchen Sie Speicherort und Zeitstempel der Quelle, bevor Sie Zielfelder interpretieren.
2. Rekonstruieren Sie den tatsächlichen Start aus Zielpfad, `Args`, `WorkingDirectory` und Umgebungsblöcken.
3. Verfolgen Sie Ziel-MFT-Referenz, Volume-Seriennummer, Netzwerkpfad, Droids, MachineID und MAC über alle Beweise hinweg.
4. Prüfen Sie Prefetch und Ereignisprotokolle auf Ausführung; prüfen Sie Jump Lists/RecentDocs auf Benutzerinteraktion.
5. Bewahren Sie bei bösartigen Anhängen angehängte Daten auf und hashen/analysieren Sie die LNK-Datei selbst.

## Weitere Informationen { #further-details }

- [Microsoft Open Specifications: MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [Sootmark shell](https://github.com/Sootmark/shell)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [LECmd von Eric Zimmerman](https://github.com/EricZimmerman/LECmd)
