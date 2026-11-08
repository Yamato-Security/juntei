# Webbrowser { #web-browsers }

## Was die Browserdatenbanken sind { #what-the-browser-databases-are }

Browser speichern ihren Verlauf, ihre Downloads, Cookies und Formulareinträge in SQLite-Datenbanken im Profil jedes
Benutzers. Chromium-basierte Browser (Chrome, Edge, Brave, Opera, Vivaldi) und Firefox-basierte Browser verwenden
jeweils eigene Dateien und Tabellen, zeichnen aber dieselben Arten von Aktivität auf:

- **Verlauf**: die besuchten Seiten, jeder Besuch mit seinem Zeitpunkt, der Art, wie der Browser zur Seite gelangt ist
  (eingetippt, über einen Link, eine Weiterleitung, ein Lesezeichen), und bei Chromium, wie lange die Seite im
  Vordergrund war.
- **Downloads**: die URL, von der eine Datei stammt (und die Weiterleitungen auf dem Weg dorthin), wo sie gespeichert
  wurde, ihre Größe, wann der Download begann und endete und ob er abgeschlossen wurde.
- **Cookies**: die Websites, die sie gesetzt haben, wann sie gesetzt und zuletzt gesendet wurden und wann sie ablaufen.
- **Formularverlauf**: in Webformulare und Suchfelder eingegebene Werte, wie oft sowie zum ersten und zum letzten Mal.

Browseraktivität ist oft die direkteste Aufzeichnung dessen, was ein Benutzer getan hat: die Phishing-Seite, der er
gefolgt ist, das Werkzeug, das er heruntergeladen hat, der Webmail-Dienst, den er genutzt hat, die Suchbegriffe, die er
eingegeben hat.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Das Browserprofil hat zum erfassten Zeitpunkt
einen Besuch der URL verzeichnet oder den Download unter dem erfassten Pfad gespeichert; die Datenbank sagt nicht, wer
an der Tastatur saß.

## Speicherort und Sicherung { #location-and-collection }

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

`<Profile>` ist `Default`, `Profile 1`, `Profile 2` usw. Chromium hat `Cookies` in Version 96 nach `Network`
verschoben; ältere Versionen legen die Datei neben `History` ab. Firefox speicherte seine Downloads bis Version 25 in
`downloads.sqlite`; spätere Versionen speichern sie in `places.sqlite`.

Sichern Sie jede Datei des Profils und jede Datenbank zusammen mit den benachbarten Dateien, die ihren Namen teilen:
`places.sqlite-wal`, `cookies.sqlite-wal`, `formhistory.sqlite-wal`, `History-journal`.

!!! warning "Write-Ahead-Logs"
    Firefox betreibt seine Datenbanken im Write-Ahead-Log-Modus: Die neuesten Änderungen stehen nur in der
    `-wal`-Datei, bis der Browser sie in die Datenbank übernimmt (ein Checkpoint). Juntei liest jede Datenbank zusammen
    mit ihrer `-wal`-Datei, so wie es der Browser tun würde. Ohne sie fehlen jüngste Besuche, Cookies und
    Formulareinträge, und im Log gelöschte Einträge erscheinen weiterhin als aktiv.

## Gelöschte Einträge { #deleted-entries }

Wenn ein Browser Verlauf löscht, verschwinden die Zeilen aus den Tabellen, ihre Datensätze können aber im freien
Speicherplatz der Datenbank, auf ihrer Freelist oder in älteren Versionen der Datenbankseiten erhalten bleiben: in der
`-wal`-Datei oder in der Datenbankdatei, deren Seiten eine `-wal`-Datei ersetzt hat. Juntei stellt daraus gelöschte
Besuche, Seiten, Downloads und Formulareinträge wieder her, als Zeilen, deren `Status` `Deleted` ist:

- `Confidence` gibt an, wie sicher die Zuordnung des Datensatzes zu seiner Tabelle ist: `High` für einen vollständigen
  Datensatz, dessen Werte alle den Typ ihrer Spalte haben, `Medium` oder `Low` für beschädigte Datensätze oder
  Datensätze, die zu mehreren Tabellen passen.
- `RecoveredFrom` gibt an, wo sich der Datensatz befand: der Bereich der Seite (eine Zelle, ein Freeblock, nicht
  zugeordneter Speicherplatz, eine freie Seite), welche Seite und welche Version davon sowie der Offset des Datensatzes
  in der Seite. WAL-Frames werden ab 0 gezählt.
- `LostColumns` listet die Spalten auf, die ein beschädigter Datensatz nicht mehr enthält: Ihre Felder sind leer.

Browser löschen mit `secure_delete` von SQLite: Freigegebener Speicherplatz in der Datenbank wird mit Nullen
überschrieben. Was sich wiederherstellen lässt, liegt dann überwiegend in älteren Seitenversionen: in denen, die die
`-wal`-Datei noch enthält, und in den eigenen Kopien der Datenbankdatei von den Seiten, die eine `-wal`-Datei ersetzt
hat, bis zum nächsten Checkpoint. Kopieren Sie die `-wal`-Datei zusammen mit ihrer Datenbank, und zwar bevor der Browser
erneut läuft.

## Hinweise für die Analyse { #analyst-cautions }

- Zeiten sind in UTC angegeben. Chromium und Firefox zeichnen sie auf die Mikrosekunde genau auf, den Formularverlauf
  von Chromium auf die Sekunde genau.
- Ein Besuch bedeutet nicht, dass der Benutzer etwas eingetippt oder geklickt hat: Weiterleitungen, Frames und von
  anderen Programmen geöffnete Seiten sind ebenfalls Besuche. Lesen Sie `Transition`, `Typed` und `Hidden`.
- Chromium verschlüsselt Cookie-Werte: `Value` ist für sie leer. Juntei entschlüsselt sie nicht.
- Eine `Removed`-Zeile des Formularverlaufs stammt aus der Tabelle `moz_deleted_formhistory` von Firefox, die Firefox
  für Android für Sync befüllt. Firefox für den Desktop lässt sie normalerweise leer.
- Ein gelöschter Eintrag mit der Konfidenz `Low` kann ein Fragment eines anderen Datensatzes sein: Gleichen Sie ihn mit
  den aktiven Zeilen und den übrigen Beweisen ab.
- Keine gelöschte Zeile bedeutet nicht, dass nichts gelöscht wurde: Mit Nullen überschriebener Speicherplatz und ein
  Checkpoint lassen nichts Wiederherstellbares zurück. Gelöschte Cookies werden noch nicht wiederhergestellt.
- `Browser`, `User` und `Profile` stammen aus dem Pfad der Datenbank: Ein an einen anderen Ort kopiertes Profil behält
  nur, was sein Pfad aussagt. Ohne bekannten Browserordner ist `Browser` `Chromium-based` oder `Firefox-based`.
- Rollback-Journale (`-journal`) werden nicht gelesen, ebenso wenig `WebCacheV01.dat` von Internet Explorer und dem
  alten Edge.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

`-d` sucht die Dateien anhand ihres Namens und erkennt jede an ihren Tabellen: Eine `History`-Datei eines anderen
Programms wird übersprungen. `-f` liest eine Datenbank unabhängig von ihrem Namen.

## Ausgabetabellen { #output-tables }

Die CSV-Ausgabe erzeugt vier Dateien; hat eine davon keine Zeilen, enthält sie nur die Kopfzeile. JSON- und
JSONL-Zeilen enthalten `Table` (`History`, `Downloads`, `Cookies` oder `FormHistory`) nach `SourceFile`.

| CSV-Datei | Inhalt |
|---|---|
| `Browser-History.csv` | Besuche, aktive und gelöschte; gelöschte Seiten, deren URL keine aktive Seite hat. |
| `Browser-Downloads.csv` | Downloads, aktive und gelöschte. |
| `Browser-Cookies.csv` | Cookies. |
| `Browser-FormHistory.csv` | Formulareinträge, aktive und gelöschte; Entfernungsdatensätze von Firefox. |

## Felder { #fields }

Jede Tabelle beginnt mit diesen Feldern:

| Feld | Interpretation |
|---|---|
| `SourceFile` | Die Datenbank, aus der die Zeile gelesen wurde. |
| `User` | Der Windows-Benutzer, in dessen Profil der Pfad liegt (der Ordner nach `Users`). |
| `Browser` | Der Browser, den der Pfad nennt: `Chrome`, `Edge`, `Brave`, `Opera`, `Vivaldi`, `Firefox` … |
| `Profile` | Das Browserprofil: der Ordner der Datenbank. |
| `Status` | `Live`, `Deleted` (wiederhergestellt) oder `Removed` (ein Entfernungsdatensatz von Firefox). |

### Verlauf { #history }

| Feld | Interpretation |
|---|---|
| `RecordType` | `Visit` oder `Page` für eine gelöschte Seite, deren Besuche ebenfalls verschwunden sein können. |
| `VisitTime` | Wann der Besuch begann (UTC). |
| `URL`, `Title` | Die Seite und ihr Titel, wie zuletzt gesehen. |
| `Transition` | Wie der Browser zur Seite gelangt ist. Chromium: der Kerntyp, dann seine Qualifizierer (`LINK`, `TYPED`, `AUTO_BOOKMARK`, `FORM_SUBMIT`, `RELOAD`, … mit `CHAIN_START`, `SERVER_REDIRECT`, `FROM_ADDRESS_BAR`, …); Firefox: `LINK`, `TYPED`, `BOOKMARK`, `EMBED`, `REDIRECT_PERMANENT`, `REDIRECT_TEMPORARY`, `DOWNLOAD`, `FRAMED_LINK`, `RELOAD`. |
| `VisitDuration` | Sekunden, die die Seite im Vordergrund war (Chromium; 0, wenn nicht gemessen). |
| `VisitCount` | Besuche der Seite insgesamt. |
| `Typed`, `TypedCount` | Ob die URL jemals eingetippt wurde, und wie oft (Chromium). |
| `Hidden` | In der Verlaufsliste ausgeblendet: Subframes, Quellen von Weiterleitungen, Fehler. |
| `Frecency` | Die Rangfolge der Seite bei Firefox für Vorschläge in der Adressleiste. |
| `VisitID`, `FromVisitID` | Die Zeilen-ID des Besuchs und die des Besuchs, von dem er ausging (die verweisende Seite, die Quelle der Weiterleitung). |
| `LastVisitTime` | Der letzte Besuch einer gelöschten Seite (UTC). |
| `PageSource` | Woher URL und Titel eines gelöschten Besuchs stammen: `Live` (eine aktive Seite), `Recovered` (eine gelöschte Seite) oder `NotFound`. |
| `Confidence`, `RecoveredFrom`, `LostColumns` | Wie ein gelöschter Eintrag wiederhergestellt wurde; siehe [Gelöschte Einträge](#deleted-entries). |

### Downloads { #downloads }

| Feld | Interpretation |
|---|---|
| `StartTime`, `EndTime` | Wann der Download begann und endete (UTC). |
| `URL`, `RedirectChain` | Woher die Datei stammt, und jede URL auf dem Weg dorthin (Chromium; in CSV durch ` -> ` getrennt). |
| `TargetPath`, `CurrentPath` | Wo die Datei gespeichert wurde (bei Firefox ein `file://`-URI), und wohin sie während des Downloads geschrieben wurde. |
| `ReceivedBytes`, `TotalBytes` | Empfangene Bytes und die Größe der Datei. |
| `State` | `Complete`, `InProgress`, `Cancelled`, `Interrupted`, `Paused`, `Blocked` oder `Dirty` (als Malware blockiert). |
| `DangerType` | Die Einstufung der Datei durch Chromium: 0 nicht gefährlich, 1 gefährlicher Dateityp, 2 gefährliche URL, 3 gefährlicher Inhalt, 4 möglicherweise gefährlich, 5 ungewöhnlich, 6 vom Benutzer behalten, 7 gefährlicher Host, 8 potenziell unerwünscht, … |
| `InterruptReason` | Warum ein unterbrochener Download angehalten wurde (Chromium; 0, wenn es keinen Grund gibt). |
| `Referrer`, `TabURL` | Die verweisende Seite und die Seite im Tab, der den Download gestartet hat. |
| `MimeType`, `Opened` | Der Typ der Datei, wie vom Server geliefert, und ob sie aus dem Browser heraus geöffnet wurde (Chromium). |
| `FileDeletedByUser` | Ob der Benutzer die heruntergeladene Datei über die Downloadliste des Browsers vom Datenträger gelöscht hat (Firefox). |
| `DownloadID` | Die Zeilen-ID des Downloads. |

### Cookies { #cookies }

| Feld | Interpretation |
|---|---|
| `Created`, `LastAccessed`, `Expires` | Wann das Cookie gesetzt und zuletzt gesendet wurde und wann es abläuft (UTC; `Expires` ist bei einem Sitzungscookie leer). |
| `Host`, `Name`, `Path` | Die Website, für die das Cookie gilt, sein Name und sein Pfad. |
| `Value` | Der Wert, wie er gespeichert ist: leer, wenn Chromium ihn verschlüsselt hat. |
| `Secure`, `HttpOnly`, `Persistent` | Nur über HTTPS gesendet, vor Skripten verborgen, über Sitzungen hinweg beibehalten (Chromium). |
| `GoogleAnalytics` | Die dekodierten Felder eines Google-Analytics-Cookies (`__utma`, `__utmb`, `__utmt`, `__utmz`): Besuchszeiten, Sitzungen sowie Kampagnenquelle und Suchbegriffe. |
| `RowID` | Die Zeilen-ID des Cookies. |

### Formularverlauf { #form-history }

| Feld | Interpretation |
|---|---|
| `FirstUsed`, `LastUsed` | Wann der Wert zum ersten und zum letzten Mal eingegeben wurde (UTC). |
| `RemovedTime` | Wann der Eintrag laut einem Entfernungsdatensatz von Firefox entfernt wurde (UTC). |
| `FieldName`, `Value` | Das Formularfeld (`email`, `q`, die Suchleiste von Firefox: `searchbar-history`) und was eingegeben wurde. |
| `Count` | Wie oft der Wert eingegeben wurde. |
| `GUID` | Der Bezeichner des Eintrags bei Firefox, den Entfernungsdatensätze nennen. |
| `Sources` | Woher Firefox den Eintrag gespeichert hat, in den Versionen, die dies festhalten (in CSV durch einen senkrechten Strich getrennt). |
| `RowID` | Die Zeilen-ID des Eintrags. |

## Automatisches Parsen { #automatic-parsing }

`auto-parse` schreibt die Tabellen nach `BrowserActivity` als `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv` und `Browser-FormHistory.csv` (eine einzige Datei `Browser.json` bzw. `Browser.jsonl` mit
`-t json` bzw. `-t jsonl`), zusammen mit `BrowserActivity/Browser-logs.txt`. Die kombinierte Zeitleiste `-j` enthält
jede Zeile, mit `Comp` und dem `Artifact`-Bezeichner `browser`. `-Z` konvertiert `VisitTime`, `LastVisitTime`,
`StartTime`, `EndTime`, `Created`, `LastAccessed`, `Expires`, `FirstUsed`, `LastUsed` und `RemovedTime`.

## Weitere Informationen { #further-details }

- [Chromium — Seitenübergangstypen](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — Gefahrentypen von Downloads](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — Places-Datenbank](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — Write-Ahead-Logging](https://www.sqlite.org/wal.html)
- [Plaso — SQLite-Parser für Browser](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
