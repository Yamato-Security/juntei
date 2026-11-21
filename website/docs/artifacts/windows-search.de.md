# Windows-Search-Index { #windows-search-index }

## Was der Index aufzeichnet { #what-the-index-records }

Windows Search (der Dienst `WSearch`) durchsucht die Benutzerprofile, das Startmenü, die Postfächer von Outlook,
den Verlauf des Internet Explorers, Offlinedateien und alles Weitere in seinem Suchbereich und speichert das
Gelesene in `Windows.edb`, einer ESE-Datenbank. Zwei Teile davon sind für eine Untersuchung von Bedeutung:

- der **Property Store** (`SystemIndex_PropertyStore`; `SystemIndex_0A` unter Windows 7) mit einer Zeile je
  indiziertem Element: Pfad, Name, Typ und Größe, Besitzer, die vom Indexer aus der Datei übernommenen
  Erstellungs-, Änderungs- und Zugriffszeiten, der Zeitpunkt, zu dem der Indexer es gelesen hat, Titel und Autor
  eines Dokuments, der Anfang seines Textes, der Computername und Hunderte weiterer Eigenschaften;
- die **Gatherer**-Tabellen (`SystemIndex_Gthr`, `SystemIndex_GthrPth`) mit den eigenen Aufzeichnungen des
  Crawlers zu jedem Element: sein URL-Pfad, seine Änderungszeit zum Zeitpunkt der Einreihung, die Crawl-Nummer
  und -Flags sowie ein Löschzähler.

Ein Element bleibt im Index, nachdem die Datei, die Nachricht oder die Seite verschwunden ist, bis der Crawler die
Löschung bemerkt. Der Index nennt daher Dateien, die nicht mehr existieren, mit ihren Metadaten und einem Auszug
ihres Textes, so wie sie bei der Indizierung waren.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Der Indexer hat das Element unter dem
aufgezeichneten Pfad, mit den aufgezeichneten Metadaten, zum Zeitpunkt `GatherTime` auf dem genannten Computer
gelesen. Der Index zeigt, dass die Datei existierte und was sie enthielt. Er zeigt nicht, dass ein Benutzer sie
geöffnet hat.

## Speicherort und Sicherung { #location-and-collection }

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

Windows 7 bis Windows 11 21H2 sowie die Windows-Server-Versionen derselben Jahre halten den Index in
`Windows.edb`. Windows 11 22H2 und neuer halten ihn in SQLite-Datenbanken im selben Ordner
(`Windows.db`, `Windows-gather.db`, `Windows-usn.db`); Juntei findet und benennt sie und meldet sie als das
nicht unterstützte neuere Format.

Sichern Sie den gesamten Ordner `Applications\Windows` aus einem Datenträgerabbild, einer Volumeschattenkopie
oder nach dem Beenden des Dienstes `WSearch`: Die Datenbank ist geöffnet, solange der Dienst läuft, und ihre
neuesten Änderungen stehen nur in den Transaktionsprotokollen `MSS*.log`, die Juntei nicht einspielt. Ein von
einem laufenden System kopierter Index bleibt „dirty“; die Ausgabe weist darauf hin und liest, was auf dem
Datenträger steht. Bei Verzeichniseingabe und mit `auto-parse` wird jede `Windows.edb` in beliebiger Tiefe
gelesen; verwenden Sie `-f` für eine einzelne Datei.

## Hinweise für die Analyse { #analyst-cautions }

- Eine unveränderliche Beweiskopie analysieren. Juntei verwendet eine Speicherabbildung der Datei; die Datenbank darf während des Lesens weder geändert noch gekürzt werden.
- `GatherTime` ist der Zeitpunkt, zu dem der Indexer das Element gelesen hat, nicht der Zeitpunkt, zu dem ein
  Benutzer es erstellt, geändert oder geöffnet hat. Die Indizierung läuft im Hintergrund, oft Minuten bis Stunden
  nach einer Änderung, und pausiert, während der Computer ausgelastet ist oder im Akkubetrieb läuft.
- `DateCreated`, `DateModified` und `DateAccessed` sind Kopien, die der Indexer aus der Datei übernommen hat.
  Sie werden aktualisiert, wenn das Element erneut indiziert wird, und beschreiben die Datei daher mit Stand
  `GatherTime`.
- `ItemPath` ist der Anzeigepfad. `ItemURL` und `GatherPath` sind die URLs des Crawlers: `file:` für Dateien
  (mit dem Volume als `[GUID]` ab Windows 8), `iehistory://{SID}/` für den Verlauf des Internet Explorers,
  `mapi://` für Outlook-Elemente, `csc://{SID}/` für Offlinedateien und `winrt://{SID}/` für die Inhalte von
  Windows-Apps.
- `User` ist das im Pfad des Elements genannte Profil (`C:\Users\<name>\…`). Es gibt an, wo sich das Element
  befand, nicht, wer es erstellt hat; `FileOwner` ist der Besitzer der Datei, wie der Indexer ihn gesehen hat.
- `AutoSummary` ist der Anfang des Textes des Elements. Er kann ein Dokument zitieren, das inzwischen gelöscht
  oder verschlüsselt wurde, und ist ein abgeschnittener Auszug, nicht das Dokument.
- Eine „dirty“ Datenbank kann Seiten enthalten, die ihre Branch-Seiten noch nicht erreichen. Juntei folgt der
  eigenen Verkettung der Blattseiten und gibt in der Zusammenfassung an, wie viele es auf diesem Weg gelesen hat.
  Ihre neuesten Änderungen stehen weiterhin in den Transaktionsprotokollen.
- Ein Element in den Gatherer-Tabellen ohne Zeile im Property Store (`InTables` ist `Gthr`) wurde eingereiht
  oder gecrawlt, und seine Eigenschaften wurden noch nicht geschrieben oder bereits entfernt. Sein Pfad, sein Name
  und seine Gatherer-Zeiten sind weiterhin vorhanden.
- Windows 7 speichert Zeichenfolgen in einer eigenen komprimierten, verschleierten Kodierung. Juntei dekodiert
  sie; ein Wert, der sich nicht dekodieren lässt, bleibt hexadezimal, mit einem Hinweis in `Problems`.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

CSV ist das Standarddateiformat. Mit `-t csv`, `-t json` oder `-t jsonl` ohne `-o` werden Datensätze auf
stdout ausgegeben. Ohne beide Optionen erscheint nur eine Zusammenfassung. Mit `-C` überschreiben Sie eine vorhandene
Ausgabe. Die Ausgabe wird niemals über eine der Eingabedateien geschrieben, auch nicht mit `-C`.

Juntei schreibt je indiziertem Element einen Datensatz und verknüpft dabei den Property Store und die
Gatherer-Tabellen über die Kennung des Elements. Die Zusammenfassung zählt die Datenbanken nach Schema, die
„dirty“ Datenbanken, die Elemente in beiden Tabellen, nur im Gatherer und nur im Property Store, die Ordner, die
als gelöscht markierten Elemente, die Elemente je Store, die Elemente ohne gespeicherte Zeit und die Probleme.

Beschädigungen brechen den Lauf nicht ab. Bei einer abgeschnittenen Datenbank bleiben die Elemente erhalten, die
ihre verbliebenen Seiten enthalten, und die Zusammenfassung gibt an, was nicht gelesen werden konnte. Eine Datei
namens `Windows.edb`, die kein Windows-Search-Index ist, wird gemeldet und übersprungen. Die übrigen Datenbanken
werden weiterhin gelesen.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `DocumentID` | Die Kennung des Elements im Index: die `WorkID` des Property Store (`DocID` unter Windows 7) und die `DocumentID` der Gatherer-Zeile. |
| `InTables` | `Gthr+PropertyStore`, `Gthr` (ein eingereihtes oder gecrawltes Element ohne Eigenschaften) oder `PropertyStore`. |
| `ItemPath` | `System_ItemPathDisplay`: der Pfad, wie Windows ihn anzeigt. Leer, wenn der Property Store keine Zeile enthält. |
| `ItemURL` | `System_ItemUrl`: die URL des Crawlers für das Element (`file:`, `iehistory://`, `mapi://`, `csc://`, `winrt://`). |
| `GatherPath` | Der Pfad des Gatherers, zusammengesetzt aus dem Suchbereichsbaum und dem Dateinamen der Zeile. |
| `Name` | `System_ItemName` oder der Dateiname der Gatherer-Zeile. |
| `ItemType`, `ItemTypeText` | `System_ItemType` (die Erweiterung oder `Directory`) und sein Anzeigename. |
| `Kind` | `System_Kind`: `document`, `folder`, `link`, `program`, `email`, `picture`, …; mehrere werden mit `\|` verbunden. |
| `MimeType` | `System_MIMEType`. |
| `Store` | `System_Search_Store`: `file`, `mapi` (Outlook), `iehistory`, `csc` (Offlinedateien) oder `winrt` (Windows-Apps). |
| `IsFolder`, `IsDeleted` | `System_IsFolder` und `System_IsDeleted`, wie gespeichert. |
| `DeletedCount` | Der Löschzähler der Gatherer-Zeile. |
| `Size` | `System_Size` in Bytes. Leer bei Ordnern, die einen Platzhalter enthalten. |
| `FileAttributes` | `System_FileAttributes` als Zahl (16 Verzeichnis, 32 Archiv, 2 versteckt, …). |
| `FileOwner` | `System_FileOwner` (`DOMAIN\user`), wie der Indexer ihn gelesen hat. |
| `DateCreated`, `DateModified`, `DateAccessed` | Die Zeiten der Datei, wie der Indexer sie übernommen hat (UTC). |
| `ItemDate` | `System_ItemDate`: das eigene Datum des Elements, etwa die Sendezeit einer Nachricht oder das Aufnahmedatum eines Fotos; bei einer Datei meist ihre Änderungszeit auf die Sekunde genau. |
| `GatherTime` | `System_Search_GatherTime`: Zeitpunkt, zu dem der Indexer das Element gelesen hat (UTC). |
| `GatherLastModified` | `LastModified` der Gatherer-Zeile: die Änderungszeit des Elements, als der Crawler es eingereiht hat (UTC). |
| `Title`, `Author` | `System_Title` und `System_Author`. |
| `AutoSummary` | `System_Search_AutoSummary`: der Anfang des Textes des Elements. |
| `CompName` | `System_ComputerName`: der Computer, zu dem der Index gehört. |
| `TargetURL`, `DateVisited` | URL und Besuchszeit eines Verlaufselements des Internet Explorers (`System_Link_TargetUrl`, `System_Link_DateVisited`). |
| `ActivityApp`, `ActivityText`, `ActivityContentURI`, `ActivityStartTime`, `ActivityEndTime` | Ein Element des Aktivitätsverlaufs von Windows 10 (`System_Activity_*`, `System_ActivityHistory_*`). |
| `User`, `UserSource` | Das im Pfad des Elements genannte Profil und der Pfad, aus dem es gelesen wurde (`ItemPath`, `ItemURL` oder `GatherPath`). Leer außerhalb eines Profils. |
| `ScopeID`, `SDID` | Die Kennungen des Suchbereichs und der Sicherheitsbeschreibung der Gatherer-Zeile. |
| `CrawlNumber` | `CrawlNumberCrawled` der Gatherer-Zeile. |
| `GatherFlags` | `TransactionFlags` der Gatherer-Zeile. |
| `Priority` | Die Crawl-Priorität eines noch eingereihten Elements. ESE setzt sie auf NULL, sobald das Element gecrawlt wurde, daher ist sie bei den meisten Elementen leer. |
| `PropertyCount` | Anzahl der Eigenschaften, die der Property Store für das Element enthält. |
| `OtherProperties` | Alle übrigen Eigenschaften als `Name=Value`, getrennt durch `; `. |
| `SourceFile` | Die gelesene Datenbank, relativ zum Eingabeverzeichnis, oder bei `-f` ihr Name. |
| `Problems` | Werte, die sich nicht dekodieren ließen, Zeiten, die keine Zeiten sind, doppelte Zeilen und Beschädigungen. Nicht unterdrücken. |

JSON und JSONL haben dieselben Felder mit typisierten Werten: Zahlen bleiben Zahlen, fehlende Werte sind
`null`. Statt `OtherProperties` enthalten sie `Properties`, jede Eigenschaft des Elements mit ihrem Namen, ihrer
numerischen Kennung, ihrem Spaltentyp und ihrem Wert, und bei einer Zeit die rohe FILETIME-Tick-Anzahl neben der
Zeit, als die sie gelesen wird. `Gather` enthält die Spalten der Gatherer-Zeile.

## Verhalten in der Zeitleiste { #timeline-behavior }

`auto-parse` speichert die Datensätze als `FileExistence/WindowsSearch.csv` (bzw. `.json`/`.jsonl`) und
fügt sie der kombinierten Zeitleiste `-j` hinzu. `super-timeline` erzeugt Ereignisse aus `DateCreated`,
`DateModified` und `DateAccessed` (den Zeiten der Datei, wie Windows Search sie aufgezeichnet hat), aus
`GatherTime` (dem Zeitpunkt der Indizierung des Elements), aus `DateVisited` eines Verlaufselements des Internet
Explorers und aus `ActivityStartTime` und `ActivityEndTime` einer Aktivität. `GatherLastModified` wird nur bei
einem Element zu einem Ereignis, für das der Property Store keine Änderungszeit enthält. Ein Element ohne eine
dieser Zeiten bleibt in der geparsten Ausgabe und wird als Element ohne Zeitangabe gezählt.

## Untersuchungsstrategie { #investigation-strategy }

1. Filtern Sie `Store` auf `file` und durchsuchen Sie `ItemPath`, `Name` und `AutoSummary` nach den Dateien,
   Namen und Begriffen des Falls. Der Index nennt Dateien, die inzwischen gelöscht wurden.
2. Lesen Sie `GatherTime` als den spätesten Zeitpunkt, zu dem die Datei mit den aufgezeichneten Metadaten
   existierte, und `DateModified` als ihre Änderungszeit zu diesem Zeitpunkt.
3. Ordnen Sie eine Datei mit `FileOwner` und `User` einem Konto zu, und bestätigen Sie dies dann mit `$MFT`,
   USN-Journal und `$Secure:$SDS`.
4. Pivotieren Sie bei E-Mails (`Store` `mapi`) und dem Verlauf des Internet Explorers (`iehistory`) in die
   Outlook-Dateien und den WebCache, um die vollständigen Datensätze zu erhalten.
5. Vergleichen Sie den Index über mehrere Hosts hinweg: `CompName` nennt den Computer, zu dem der Index gehört,
   und ein aus einem Abbild gesicherter Index nennt die Dateien auf diesem Abbild.

## Getestete Versionen { #tested-versions }

Die Tests verwenden öffentliche Indizes, deren Ergebnisse andere Werkzeuge unabhängig gelesen haben:

- Der Windows-7-Index von plaso (`SystemIndex_0A`), abgeglichen mit `esedbexport` von libyal, einschließlich
  der Zeichenfolgenkodierungen von Windows 7.
- Der Windows-10-Index des DFIRArtifactMuseum, im Zustand „dirty“ belassen, abgeglichen mit SIDR von Stroz
  Friedberg, und sein Windows-11-Index (ESE-Formatrevision 200), abgeglichen mit `esedbexport`.
- Der SQLite-Index von Windows 11 22H2 des DFIRArtifactMuseum, gemeldet als das nicht unterstützte neuere Format.
- Abgeschnittene, leere oder andersartige Datenbanken.

## Weitere Informationen { #further-details }

- [Microsoft: Übersicht über Windows Search](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb: Dateiformat der ESE-Datenbank](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb: Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg: SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft: Quellcode der Extensible Storage Engine](https://github.com/microsoft/Extensible-Storage-Engine)
