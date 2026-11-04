# Ausgabeformate { #output-formats }

## Zusammenfassungen im Terminal { #terminal-summaries }

Ohne `-o` oder ein strukturiertes `-t` zeigen Befehle in der Regel Anzahlen, Zeiträume, Schäden und besonders
aussagekräftige Befunde an. Zusammenfassungen helfen Ihnen, den Umfang zu verstehen, bevor Sie umfangreiche Ausgaben erzeugen.

## CSV { #csv }

CSV ist das standardmäßige Speicherformat. Es eignet sich gut für Tabellenkalkulationen, den SQL-Import und
Zeitleistenbetrachter. Manche Parser erzeugen mehrere Tabellen, weil ihre Artefakte unterschiedliche Datensatztypen
enthalten (Amcache, Jump Lists und SRUM).

!!! warning "Beweisspalten als Text behandeln"
    Ein Pfad oder eine Befehlszeile kann mit `=`, `+`, `-` oder `@` beginnen. Tabellenkalkulationssoftware kann
    dies als Formel interpretieren. Importieren Sie alle Felder als Text oder verwenden Sie einen forensisch sicheren CSV-Betrachter. Juntei bewahrt
    Beweiszeichenfolgen, statt sie zu verändern.

## Feldnamen { #field-names }

CSV-Spaltenüberschriften und JSON-/JSONL-Feldnamen verwenden PascalCase, auch bei Amcache und den
Manifesten von `auto-parse`. Akronyme bleiben in Großbuchstaben: `ProcessID`, `TimestampUTC`, `UserSID`,
`SHA256`. Ein auf ein Akronym folgendes Wort wird mit einem Bindestrich abgetrennt: `IP-Address`, `MFT-Entry`,
`USN-Journal`. Setzen Sie in `jq` ein Feld mit Bindestrich in Anführungszeichen: `.["IP-Address"]`.

Die Standard-CSV der MFT verwendet kürzere, gut lesbare Überschriften wie `EntryNum`, `Ext`, `Dir`, `HasADS`,
`Size`, `Created (SI)` und `Created (FN)`. Das vollständige CSV-Schema finden Sie in der [Feldreferenz der MFT](../artifacts/mft.md).
Zeitstempelpaare folgen der MACB-Reihenfolge: DataModified, Accessed,
MetadataChanged, Created, jeweils SI vor FN. Setzen Sie in CSV-Werkzeugen Überschriften mit Leer- oder Satzzeichen
in Anführungszeichen.

Beweiswerte behalten ihre ursprüngliche Schreibweise. Von der Quelle stammende Wörterbuchschlüssel, etwa Namen von Registry-Werten
unter `Details`/`DataRaw` und `Zone.Identifier`-Namen unter `ZoneIdentifier`, bleiben ebenfalls
unverändert. Die Wörterbücher `ValueState` und `Raw` von SRUM verwenden dieselben Bezeichnungen wie ihre Datenspalten.
Eine Kollision bei der Normalisierung wird gemeldet, statt ein Feld stillschweigend zu verwerfen.

Dies ändert frühere Exportschemata. Erzeugen Sie vorhandene Ergebnisse neu und aktualisieren Sie gespeicherte Abfragen
(so wird beispielsweise `.record_type` zu `.RecordType` und `full_path` von Amcache zu `FullPath`).
Menschenlesbare Textberichte sowie Eingabe-/Konfigurationsformate behalten ihre bisherigen Konventionen.

## Abgekürzte Feldnamen { #abbreviated-field-names }

Die folgenden Bezeichnungen gelten für CSV-, JSON- und JSONL-Exporte, einschließlich des automatischen Parsens und der
Manifeste. Aktualisieren Sie gespeicherte Filter für `jq`, `xsv` und Tabellenkalkulationen, wenn Sie ältere Ergebnisse neu erzeugen.
Nur die Ausgabebezeichnungen ändern sich; Beweiswerte und rohe Namen von Registry-Werten behalten ihre ursprüngliche Schreibweise.

| Bisherige Bezeichnung | Aktuelle Bezeichnung |
|---|---|
| `ADS` | `HasADS` |
| `Application` | `App` |
| `ApplicationCompany` | `AppCompany` |
| `ApplicationCompanyRaw` | `AppCompanyRaw` |
| `ApplicationPath` | `AppPath` |
| `Arguments` | `Args` |
| `Attributes` | `Attr` |
| `AttributeType` | `AttrType` |
| `BytesRecvd` | `BytesReceived` |
| `Computer` | `Comp` |
| `ComputerName` | `CompName` |
| `ComputerNameLastWrittenUTC` | `CompNameLastWrittenUTC` |
| `Directories` | `Dir` |
| `FileName` | `Filename` |
| `MicrosecondZeros` | `ZeroU-Secs` |
| `SI-BeforeFN` | `SI<FN` |
| `UserName` | `User` |
| `Username` | `User` |
| `UserNameSource` | `UserSource` |
| `Users` | `User` |

Der native JSON-Zeitwert der Systeminformationen ist `CompNameLastWritten` (CSV:
`CompNameLastWrittenUTC`). `HasADS` ist ein boolescher Indikator; `Dir` in Prefetch enthält die
aufgezeichnete Verzeichnisliste. `User` behält seinen artefaktspezifischen Typ: eine Zeichenfolge für einen Kontonamen
oder die Liste der zugehörigen Benutzer in der USB-Ausgabe. Es impliziert keine ausführende Identität.
Setzen Sie Satzzeichen in Abfragen in Anführungszeichen: `."SI<FN"` und `.["ZeroU-Secs"]`.

## JSON { #json }

JSON speichert das vollständige Ergebnis als ein Dokument. Es bewahrt Arrays, verschachtelte Strukturen, Nullwerte,
große Ganzzahlen und Unterscheidungen, die eine flache CSV nicht immer abbilden kann.

```bash
juntei parse-file lnk -f sample.lnk -o lnk.json -t json
```

## JSON Lines { #json-lines }

JSONL schreibt ein unabhängiges JSON-Objekt pro Zeile. Es lässt sich gut streamen und arbeitet ganz natürlich mit `jq` zusammen:

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.StartType == "automatic") | {Host, ServiceName, ImagePath}'
```

Wenn strukturierte Ausgabe an die Standardausgabe geht, sendet Juntei Fortschritt und Meldungen an die Standardfehlerausgabe,
sodass eine Pipe nur Daten empfängt.

`auto-parse -j timeline.jsonl` erzeugt einen zusätzlichen JSONL-Datensatz für eine vollständige Sammlung mehrerer Hosts
und behält dabei die kategorisierten CSV-Ergebnisse bei. Fügen Sie `-t json` oder `-t jsonl` hinzu, um das Format der
einzelnen Ergebnisse zu ändern. Jeder Datensatz hat die Felder `Comp` und `Artifact`, sodass seine Quelle auch
nach dem Import in ein SIEM erkennbar bleibt.

## UTC und Genauigkeit { #utc-and-precision }

Juntei formatiert geparste Zeiten in UTC gemäß RFC 3339, soweit das Format dies zulässt. Manche Artefakte haben
eine Genauigkeit von 100 Nanosekunden; andere speichern Sekunden, FAT-Zeit, Aggregationsintervalle oder gar keine Zeit.
Stellen sind keine Genauigkeit: Machen Sie sich stets mit dem Quellfeld vertraut, das auf der Seite des jeweiligen Artefakts beschrieben ist.

UTC verwendet die Form der Windows-Ereignisprotokolle `2026-10-20T08:46:38.2001668Z` und behält die verfügbare
Genauigkeit der Nachkommastellen bei. `auto-parse -Z "+09:00"` zeigt geparste Zeitpunkte mit einem expliziten festen
Offset in allen Ergebnisformaten und im optionalen Export `-j` an. Standard ist UTC, unabhängig vom
Computer, auf dem Juntei läuft. Rohwerte und Uhrzeiten ohne Zeitzone bleiben wie aufgezeichnet; siehe
[Ausgabezeitzonen](../commands/auto-parse.md#output-time-zone).

## Schäden und fehlende Werte { #damage-and-missing-values }

Leere Werte, `null`, `NA`, `Problems` und parserspezifische Zustandsfelder sind aussagekräftig. Sie können bedeuten, dass das
Artefakt keinen Wert gespeichert hat, der Datensatz beschädigt war, das Format ihn nicht unterstützt oder der
Parser ihn nicht auflösen konnte. Ersetzen Sie nicht alle leeren Werte durch Null.
