# PowerShell-Verlauf { #powershell-history }

## Was der Verlauf aufzeichnet { #what-the-history-records }

PSReadLine speichert Befehlszeilen, die in einem interaktiven PowerShell-Host eingegeben wurden. Die Datei enthält den Text in Speicherreihenfolge, ohne Zeitstempel für einzelne Befehle. Der Verlauf kann bearbeitet, gelöscht, deaktiviert oder gefiltert werden und belegt keinen erfolgreichen Befehlsabschluss. Gleichen Sie Ausführung und Zeitpunkt mit PowerShell-Ereignisprotokollen, Prozesstelemetrie und anderen Ausführungsartefakten ab.

## Speicherort und Sicherung { #location-and-collection }

Der Windows-Standardspeicherort für Windows PowerShell 5.1 und PowerShell 7+ ist:

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

Andere Hosts verwenden dort Namen wie `Visual Studio Code Host_history.txt`. PowerShell unter Linux und macOS verwendet normalerweise `~/.local/share/powershell/PSReadLine/<Host>_history.txt`. Mit `HistorySavePath` kann PSReadLine die Datei umleiten; die Standardpfade sind Annahmen für die Suche und keine vollständige Liste möglicher Speicherorte.

Bei Verzeichniseingabe und mit `auto-parse` werden rekursiv Dateien namens `<Host>_history.txt` direkt in einem `PSReadLine`-Ordner ausgewählt, ohne ASCII-Groß-/Kleinschreibung zu beachten. Beliebige `*_history.txt`-Dateien an anderen Orten werden nicht ausgewählt. Verwenden Sie ausdrücklich `-f` für umbenannte Dateien oder Kopien außerhalb eines `PSReadLine`-Verzeichnisses. Bewahren Sie nach Möglichkeit die ursprüngliche Profilstruktur für die Benutzerzuordnung.

## Mit Juntei parsen { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

CSV ist das Standarddateiformat. Mit `-t csv`, `-t json` oder `-t jsonl` ohne `-o` werden Datensätze auf stdout ausgegeben. Ohne beide Optionen erscheint nur eine Zusammenfassung. JSON besteht aus einzelnen formatierten Objekten, JSONL aus einem Objekt je Zeile. Mit `-C` überschreiben Sie vorhandene Ausgaben; Aliase der Quelldatei sind vor dem Ersetzen geschützt.

`auto-parse` speichert `ShellHistory/PowerShellHistory.csv` (oder das gewählte JSON-Format), auch bei VHDX/E01-Abbildern. Der Verlauf liefert keine Ereignisse für `auto-parse -j` oder `super-timeline`.

## Kodierung und Zeilenverarbeitung { #encoding-and-line-handling }

- Eine UTF-8-, UTF-16LE- oder UTF-16BE-Byte-Reihenfolgemarkierung bestimmt die Kodierung und wird am Dateianfang entfernt. Ohne BOM wird die Eingabe immer als UTF-8 behandelt. BOM-loses UTF-16 wird nicht erraten; NUL-Zeichen bleiben erhalten und werden als mögliche Kodierungsschäden gemeldet.
- Ungültige UTF-8-Sequenzen und ungepaarte UTF-16-Surrogate werden zu U+FFFD; `Problems` meldet den Schaden. Auch ein einzelnes abschließendes UTF-16-Byte und Lesefehler werden gemeldet; lesbare Zeilen bleiben erhalten. Prüfen Sie `Problems` und das Parserprotokoll, bevor Sie von Vollständigkeit ausgehen.
- Dateien werden als Datenstrom verarbeitet. Je physischer Zeile bleiben höchstens 1,048,576 Eingabebytes erhalten, ohne BOM und LF-Trennzeichen (ein CR vor LF zählt zum Limit). Längere Zeilen werden an einer vollständigen Zeichengrenze gekürzt; die übrigen Bytes werden eingelesen und `Problems` meldet die Anzahl ausgelassener Bytes. Das Limit gilt für kodierte Eingabebytes, nicht für die Zeichenzahl.
- LF- und CRLF-Zeilenenden werden entfernt, ebenso ein abschließendes CR einer nicht abgeschlossenen Zeile. Andere Leerzeichen und Tabulatoren bleiben erhalten. Leere Zeilen erzeugen keinen Datensatz, zählen aber bei den Zeilennummern mit und beenden eine Fortsetzungskette. Zeilen nur aus Leerraum bleiben Datensätze. Bei Lesefehlern kann eine leere Diagnosezeile markieren, wo das Lesen endete.
- PSReadLine speichert mehrzeilige Einträge mit einem abschließenden Backtick an jeder fortgesetzten physischen Zeile. Die Zeilen bleiben getrennt; Backticks und Einrückung bleiben erhalten. `EntryStartLine` verweist auf die erste Zeile einer zusammenhängenden Kette, deren vorherige Zeilen jeweils mit Backtick enden. Shell-Syntax wird nicht interpretiert, Befehle werden nicht rekonstruiert. Kürzungen können die Fortsetzungszuordnung beeinflussen; prüfen Sie `Problems` vor dem Gruppieren solcher Zeilen.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `SourceFile` | Pfad relativ zur Verzeichniseingabe, mit `/`; bei `-f` nur der Dateiname. Es wird kein absoluter Quellpfad exportiert. |
| `User` | Aus dem erhaltenen Pfad abgeleiteter Profilname. Verzeichniseingabe erkennt auch Strukturen mit `Users`/`Documents and Settings` unterhalb des Eingabestamms; `-f` benötigt das übliche PSReadLine-Profilpfad-Suffix. Unbekannte Werte sind in CSV leer und in JSON `null`. |
| `ShellHost` | Dateinamenspräfix vor `_history.txt`, etwa `ConsoleHost`; bei umbenannten Dateien ohne dieses Suffix unbekannt. |
| `LineNumber` | Physische Zeilennummer ab 1, einschließlich übersprungener Leerzeilen. |
| `EntryStartLine` | Erste physische Zeile der Fortsetzungskette oder die Nummer dieser Zeile selbst. |
| `Command` | Dekodierte physische Zeile ohne Zeilenende, unter Beachtung des dokumentierten Limits und CSV-Schutzes. |
| `FileLastWriteTime` | Änderungszeit der Quelldatei, in jeder Zeile wiederholt. **Dateimetadaten, niemals Befehlsausführungszeit.** Bei Abbildern werden ursprüngliche Quellmetadaten statt der Staging-Zeit verwendet. |
| `FileSize` | Größe der Quelldatei in Bytes. |
| `Encoding` | `UTF-8`, `UTF-8-BOM`, `UTF-16LE` oder `UTF-16BE`. |
| `Problems` | Diagnosen zu Zeilenschäden und Kürzungen; in CSV durch Semikolon getrennt, in JSON/JSONL als Array. Dateibezogene Diagnosen stehen auch im Protokoll. |

`FileLastWriteTime` wird standardmäßig in UTC im ISO-8601-Format dargestellt, etwa `2026-10-20T08:46:38.5Z`. Nur `auto-parse` akzeptiert `-Z, --timezone`, um den angezeigten Offset zu ändern. Der eigenständige Befehl `parse-file powershell-history` hat keine Zeitzonenoption.

Jede CSV-Textzelle wird auf Tabellenkalkulations-Formelpräfixe geprüft (`=`, `+`, `-`, `@`, einschließlich vollbreiter Formen und Präfixe nach Leerraum oder bestimmten unsichtbaren/Steuerzeichen). Betroffene Zellen und Zellen, die mit Steuerzeichen beginnen, erhalten ein führendes Apostroph. Dadurch können Befehls- und Pfadtexte in CSV verändert werden. JSON und JSONL bewahren den dekodierten Text ohne CSV-Apostroph; Dekodierungsersetzungen und Kürzungen gelten weiterhin. Verwenden Sie JSONL zum Vergleich gespeicherter Befehlstexte und bewahren Sie immer die ursprünglichen Verlaufsdateien auf.
