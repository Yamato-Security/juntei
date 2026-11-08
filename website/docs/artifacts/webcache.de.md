# Internet Explorer und Edge Legacy { #internet-explorer-and-legacy-edge }

## Was die WebCache-Datenbank ist { #what-the-webcache-database-is }

`WebCacheV01.dat` ist die Datenbank der Windows-Internetfunktionen (WinINet) im Profil jedes Benutzers, eine
ESE-Datenbank. Internet Explorer 10 und 11 speichern darin ihren Verlauf, ihre Downloads, ihren Cache und ihre Cookies,
ebenso der alte Edge (Edge Legacy), der Edge von Windows 10, bevor Edge auf Chromium umgestellt wurde. Jede Art von Datensatz ist ein
Container, eine eigene Tabelle:

- **Verlauf** (`History` sowie `MSHist01…` für einen Tag oder eine Woche): die besuchten URLs, jede mit dem Konto, für
  das sie aufgezeichnet wurde, dem Zeitpunkt des letzten Besuchs und der Anzahl der Besuche. Auch der Datei-Explorer
  zeichnet dort die lokalen Dateien auf, die er öffnet, als `file:///`-URLs.
- **Downloads** (`iedownload`): die URL, von der eine Datei stammt, wo sie gespeichert und wohin sie während des
  Downloads geschrieben wurde, ihre Größe, wann der Download begann, die verweisende Seite und der MIME-Typ.
- **Cookies**: ab Windows 10 die Cookies selbst (`CookieEntryEx`-Tabellen), mit Host, Name, Wert und Zeiten; davor die
  Cookie-Dateien, die sie enthielten.
- **Cache** (`Content`): die zur Anzeige von Seiten heruntergeladenen Dateien, jede mit ihrer URL, dem Speicherort der
  Kopie auf dem Datenträger und den HTTP-Antwortheadern. Andere Programme, die WinINet verwenden, legen ihre Downloads
  ebenfalls hier im Cache ab.
- Webspeicher (`DOMStore`), Kompatibilitätslisten (`iecompat`) und weitere.

Die Datenbank überdauert die Browser: Internet Explorer wurde eingestellt, und der Chromium-basierte Edge führt eigene
SQLite-Datenbanken (siehe [Webbrowser](browser.md)), doch die Datenbank liegt weiterhin in jedem Profil, mit dem, was
zuvor aufgezeichnet wurde, und dem, was der Datei-Explorer und andere Programme weiterhin aufzeichnen.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Das Benutzerprofil hat die URL, den Download oder
die zwischengespeicherte Datei zum erfassten Zeitpunkt verzeichnet; die Datenbank sagt nicht, wer an der Tastatur saß,
und auch nicht, dass eine zwischengespeicherte Datei geöffnet wurde.

## Speicherort und Sicherung { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

Die Dateien, die die Container nennen (zwischengespeicherte Dateien, Cookie-Dateien), liegen in eigenen Ordnern:
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\`, `INetCookies\`, `History\` und für Apps unter
`%LOCALAPPDATA%\Packages\`.

Sichern Sie den gesamten Ordner `WebCache`. Windows hält die Datenbank geöffnet, solange der Benutzer angemeldet ist;
eine Kopie von einem laufenden System ist daher meist „dirty“: Ihre neuesten Änderungen stehen nur in den
Transaktionsprotokollen `V01*.log`.

!!! warning "Transaktionsprotokolle"
    Juntei spielt keine ESE-Transaktionsprotokolle ein. Eine Datenbank, die nicht sauber heruntergefahren wurde, wird
    in der Zusammenfassung als „dirty“ gezählt und als Problem gemeldet: Änderungen, die noch nur in ihren Protokollen
    stehen, fehlen. Um sie einzubeziehen, spielen Sie die Protokolle zuerst in eine Kopie des Ordners ein, mit dem
    Windows-Befehl `esentutl /r V01 /d`, ausgeführt in dieser Kopie.

## Hinweise für die Analyse { #analyst-cautions }

- Zeiten sind in UTC angegeben, auf 100 Nanosekunden genau, mit Ausnahme von `ModifiedTime` im Verlaufscontainer
  eines Tages oder einer Woche: Dieser Wert ist die lokale Zeit des Systems, ohne UTC-Offset geschrieben, und `-Z`
  lässt ihn unverändert. Rechnen Sie ihn mit der Zeitzone um, die das Quellsystem zu diesem Zeitpunkt hatte. Seine
  Differenz zu `AccessedTime` ist oft der UTC-Offset dieser Zeitzone, aber nicht immer: Beide Werte können
  unterschiedliche Zeitpunkte sein.
- Ein Verlaufseintrag ist eine URL, kein Besuch: `AccessedTime` ist ihr letzter Besuch und `AccessCount` die Anzahl der
  Besuche. Der Container `History` enthält einen Eintrag pro URL; der Container eines Tages oder einer Woche enthält die
  Einträge dieses Zeitraums, wobei sein Name den ersten Tag und den Tag nach dem letzten angibt
  (`MSHist012026110120261102`: 1. November 2026).
- `file:///`-URLs im Verlauf sind lokale Dateien, die aus dem Datei-Explorer und anderen Programmen geöffnet wurden,
  keine Webseiten.
- `Account` ist das Konto, für das ein Eintrag aufgezeichnet wurde (`Visited: alice@…`); `User` ist der Profilordner,
  in dem der Pfad der Datenbank liegt. Beide können sich unterscheiden: Ein Profilordner ist nicht immer nach seinem
  Konto benannt, und der Pfad einer an einen anderen Ort kopierten Datenbank enthält keinen Profilordner.
- `Package` gibt an, wem ein Eintrag gehört: dem alten Edge (`microsoft.microsoftedge_8wekyb3d8bbwe`), Internet
  Explorer im erweiterten geschützten Modus (`windows_ie_ac_001`) oder einer anderen App. Ist es leer, gehört der
  Eintrag dem Benutzer selbst: Internet Explorer, dem Datei-Explorer und anderen Programmen.
- Microsoft dokumentiert den Download-Datensatz nicht. Juntei liest seine Startzeit und seine Größe nur, wenn sein
  Header innerhalb des Datensatzes liegt, und seine Pfade und übrigen Zeichenfolgen nur, wenn die letzten beiden eine URL und ein
  Windows-Pfad sind. Andernfalls enthält die Zeile die eigene URL des Eintrags und, wenn der Start nicht gelesen werden
  kann, seine Zugriffszeit, zusammen mit einem Problem. `Publisher` ist der Name, den der Browser aufgezeichnet hat,
  keine Prüfung der Signatur der Datei.
- Ein Cache-Eintrag besagt, dass die Datei heruntergeladen und zwischengespeichert wurde. Die Kopie auf dem
  Datenträger kann verschwunden, ersetzt oder nie geöffnet worden sein.
- Gelöschte Einträge werden noch nicht wiederhergestellt: ESE kann einen gelöschten Datensatz in seinen Seiten
  behalten, bis es sie bereinigt, Juntei liest aber nur aktive Datensätze.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

`-d` sucht `WebCacheV01.dat` anhand des Namens; `-f` liest eine Datenbank unabhängig von ihrem Namen.

## Ausgabetabellen { #output-tables }

Die CSV-Ausgabe erzeugt fünf Dateien; hat eine davon keine Zeilen, enthält sie nur die Kopfzeile. JSON- und
JSONL-Zeilen enthalten `Table` (`History`, `Downloads`, `Cookies`, `Entries` oder `Containers`) nach `SourceFile`.

| CSV-Datei | Inhalt |
|---|---|
| `WebCache-History.csv` | Die Einträge der Verlaufscontainer. |
| `WebCache-Downloads.csv` | Downloads. |
| `WebCache-Cookies.csv` | Cookies (ab Windows 10). |
| `WebCache-Entries.csv` | Die Einträge aller anderen Container: zwischengespeicherte Dateien, Cookie-Dateien, Webspeicher, Kompatibilitätslisten und der Rest. |
| `WebCache-Containers.csv` | Die Container. |

## Felder { #fields }

Jede Tabelle beginnt mit diesen Feldern:

| Feld | Interpretation |
|---|---|
| `SourceFile` | Die Datenbank, aus der die Zeile gelesen wurde. |
| `User` | Der Windows-Benutzer, in dessen Profil der Pfad liegt (der Ordner nach `Users`). |
| `Package` | Die App, deren Daten die Zeile enthält: der Ordner unter `Packages` im Verzeichnis ihres Containers (bzw. ihrer Cookie-Tabelle). Leer für die eigenen Daten des Benutzers. |

### Verlauf { #history }

| Feld | Interpretation |
|---|---|
| `Container` | Der Container: `History` oder `MSHist01`, gefolgt vom ersten Tag seines Zeitraums und dem Tag nach dem letzten. |
| `AccessedTime` | Der letzte Besuch der URL (UTC); im Container eines Tages oder einer Woche der letzte in diesem Zeitraum. |
| `ModifiedTime` | Wann der Eintrag zuletzt geändert wurde (UTC); im Container eines Tages oder einer Woche die lokale Zeit des Systems, ohne UTC-Offset. |
| `CreationTime` | Wann der Eintrag angelegt wurde (UTC). |
| `ExpiryTime`, `SyncTime` | Wann der Eintrag aus dem Verlauf entfernt werden soll und wann er zuletzt synchronisiert wurde (UTC). |
| `Account` | Das Konto, für das der Eintrag aufgezeichnet wurde. |
| `URL` | Die Seite oder Datei, ohne das Präfix `Visited: account@`. |
| `AccessCount`, `SyncCount` | Wie oft die URL besucht und wie oft der Eintrag synchronisiert wurde. |
| `EntryType`, `Flags` | Typ und Flags des Eintrags, wie gespeichert. |
| `EntryID`, `ContainerID` | Die ID des Eintrags in seinem Container und die ID des Containers. |

### Downloads { #downloads }

| Feld | Interpretation |
|---|---|
| `StartTime` | Wann der Download begann (UTC). |
| `URL` | Woher die Datei stammt. |
| `TargetPath`, `CurrentPath` | Wo die Datei gespeichert wurde und wohin sie während des Downloads geschrieben wurde: eine `.partial`-Datei beim alten Edge, die Kopie im Cache bei Internet Explorer. |
| `TotalBytes` | Die Größe der Datei. |
| `Referrer` | Die verweisende Seite. |
| `MimeType` | Der Typ der Datei, wie vom Server geliefert (Internet Explorer). |
| `Publisher` | Der Name, den der alte Edge für den Herausgeber der Datei aufgezeichnet hat. |
| `AccessedTime` | Die eigene Zugriffszeit des Eintrags (UTC), meist der Start. |
| `DownloadID`, `ContainerID` | Die ID des Eintrags in seinem Container und die ID des Containers. |

### Cookies { #cookies }

| Feld | Interpretation |
|---|---|
| `LastModified`, `Expires` | Wann das Cookie gesetzt wurde und wann es abläuft (UTC). |
| `Host`, `Name`, `Path` | Die Website, für die das Cookie gilt, sein Name und sein Pfad. |
| `Value` | Der Wert, wie er gespeichert ist. |
| `Flags` | Die Flags des Cookies, wie gespeichert. |
| `Directory` | Das Verzeichnis, das die Datenbank der Cookie-Tabelle zuordnet: wessen Cookies es sind (`…\INetCookies\ESE\`, beim alten Edge `…\MicrosoftEdge\Cookies\ESE\`, das Verzeichnis einer App). |
| `CookieTable`, `EntryID` | Die Tabelle (`CookieEntryEx_<n>`) und die ID des Cookies darin. |

### Einträge { #entries }

| Feld | Interpretation |
|---|---|
| `Container` | Der Container: `Content` (zwischengespeicherte Dateien), `Cookies` (Cookie-Dateien), `DOMStore` (Webspeicher), `iecompat`, …. |
| `AccessedTime` | Wann der Eintrag zuletzt verwendet wurde (UTC). |
| `ModifiedTime` | Die letzte Änderung der Ressource, wie vom Server angegeben (UTC). |
| `CreationTime` | Wann der Eintrag angelegt wurde (UTC). |
| `ExpiryTime`, `SyncTime` | Wann er abläuft und wann er zuletzt mit dem Server abgeglichen wurde (UTC). |
| `URL` | Die URL, wie gespeichert: eine URL für eine zwischengespeicherte Datei, `Cookie:alice@example.com/` für eine Cookie-Datei, `DOMStore:…`, `iecompat:…`. |
| `Account` | Das Konto einer Cookie-Datei. |
| `Filename`, `FilePath` | Die Datei des Eintrags und wo sie liegt: das Verzeichnis des Containers und der Unterordner, den der Eintrag nennt. |
| `FileSize` | Die Größe der Datei. |
| `AccessCount`, `SyncCount` | Wie oft der Eintrag verwendet und wie oft er mit dem Server abgeglichen wurde. |
| `EntryType`, `Flags` | Typ und Flags des Eintrags, wie gespeichert. |
| `RedirectURL` | Wohin die URL weitergeleitet hat. |
| `ResponseHeaders`, `RequestHeaders` | Die HTTP-Antwortheader (`HTTP/1.1 200 OK`, `Content-Type`, `Server`, …) und die Anforderungsheader, wie gespeichert, vollständig mit allen Zeilen. |
| `EntryID`, `ContainerID` | Die ID des Eintrags in seinem Container und die ID des Containers. |

### Container { #containers }

| Feld | Interpretation |
|---|---|
| `ContainerID`, `Name` | Die ID und der Name des Containers. |
| `Directory` | Wo seine Dateien auf dem Datenträger liegen. |
| `SecureDirectories` | Die Unterordner von `Directory`, in denen seine Dateien liegen (in CSV durch Kommas getrennt). |
| `PartitionID` | `M` für die eigenen Container des Benutzers, `L` für die des geschützten Modus von Internet Explorer (die `Low`-Ordner), die SID eines App-Containers für die einer App. |
| `LastAccessTime`, `LastScavengeTime` | Wann der Container zuletzt verwendet und wann er zuletzt bereinigt wurde (UTC). |

## Automatisches Parsen { #automatic-parsing }

`auto-parse` schreibt die Tabellen nach `BrowserActivity` als `WebCache-History.csv`, `WebCache-Downloads.csv`,
`WebCache-Cookies.csv`, `WebCache-Entries.csv` und `WebCache-Containers.csv` (eine einzige Datei `WebCache.json` bzw.
`WebCache.jsonl` mit `-t json` bzw. `-t jsonl`), zusammen mit `BrowserActivity/WebCache-logs.txt`. Die kombinierte
Zeitleiste `-j` enthält jede Zeile, mit `Comp` und dem `Artifact`-Bezeichner `webcache`. `-Z` konvertiert jede Zeit, also die
Felder, die auf `Time` enden, `LastModified` und `Expires`, mit Ausnahme der lokalen `ModifiedTime` im
Verlaufscontainer eines Tages oder einer Woche.

## Weitere Informationen { #further-details }

- [Microsoft — Über WinINet](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — Dateiformat von ESE-Datenbanken](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — Parser für MSIE WebCache](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
