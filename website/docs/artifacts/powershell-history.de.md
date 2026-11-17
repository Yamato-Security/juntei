# PowerShell-Verlauf { #powershell-history }

## Was der Verlauf aufzeichnet { #what-the-history-records }

PSReadLine speichert Befehlszeilen, die in einem interaktiven PowerShell-Host eingegeben wurden. Die Datei enthält den Text in Speicherreihenfolge, ohne Zeitstempel für einzelne Befehle.

## Speicherorte und Sicherung { #locations-and-collection }

Der Windows-Standardspeicherort für Windows PowerShell 5.1 und PowerShell 7+ ist:

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

Andere Hosts verwenden dort Namen wie `Visual Studio Code Host_history.txt`. PowerShell unter Linux und macOS verwendet normalerweise `~/.local/share/powershell/PSReadLine/<Host>_history.txt`. Mit `HistorySavePath` kann PSReadLine die Datei umleiten; die Standardpfade sind Annahmen für die Suche und keine vollständige Liste möglicher Speicherorte.

Bei Verzeichniseingabe und mit `auto-parse` werden rekursiv Dateien namens `<Host>_history.txt` direkt in einem `PSReadLine`-Ordner ausgewählt, ohne ASCII-Groß-/Kleinschreibung zu beachten. Beliebige `*_history.txt`-Dateien an anderen Orten werden nicht ausgewählt. Verwenden Sie ausdrücklich `-f` für umbenannte Dateien oder Kopien außerhalb eines `PSReadLine`-Verzeichnisses. Bewahren Sie nach Möglichkeit die ursprüngliche Profilstruktur für die Benutzerzuordnung.

## Hinweise für die Analyse { #analyst-cautions }

- Keine Zeile hat eine Zeitangabe. `FileLastWriteTime` ist die Änderungszeit der Datei. PSReadLine speichert den Verlauf standardmäßig nach jedem Befehl (`HistorySaveStyle` `SaveIncrementally`); diese Zeit liegt daher meist nahe an dem Zeitpunkt, zu dem die letzte Zeile hinzukam, sagt aber nichts über die früheren Zeilen aus.
- Eine gespeicherte Zeile belegt nicht, dass ein Befehl ausgeführt oder erfolgreich abgeschlossen wurde. Gleichen Sie Ausführung und Zeitpunkt mit PowerShell-Ereignisprotokollen, Prozesstelemetrie und anderen Ausführungsartefakten ab.
- Der Verlauf kann bearbeitet, gelöscht oder abgeschaltet werden (`HistorySaveStyle` `SaveNothing`). PSReadLine lässt außerdem Zeilen aus, die anscheinend Geheimnisse enthalten, etwa solche mit `password`, `token` oder `apikey`, und die Windows PowerShell ISE verwendet PSReadLine nicht. Skripte und mit `-Command` oder `-EncodedCommand` ausgeführte Befehle werden nicht eingetippt und daher nicht gespeichert.
- Ein mehrzeiliger Befehl wird mit einem Backtick am Ende jeder Zeile außer der letzten gespeichert. Juntei behält diese Zeilen als separate Datensätze bei, die über `EntryStartLine` verknüpft sind. Eine Zeile, die mit einem eingetippten Backtick endet, sieht genauso aus; lesen Sie die Zeilen eines Eintrags daher zusammen.

## Mit Juntei parsen { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

CSV ist das Standarddateiformat. Mit `-t csv`, `-t json` oder `-t jsonl` ohne `-o` werden Datensätze auf stdout ausgegeben. Ohne beide Optionen erscheint nur eine Zusammenfassung. JSON besteht aus einzelnen formatierten Objekten, JSONL aus einem Objekt je Zeile. Mit `-C` überschreiben Sie vorhandene Ausgaben; Aliase der Quelldatei sind vor dem Ersetzen geschützt.

## Kodierung und Zeilenverarbeitung { #encoding-and-line-handling }

- Eine UTF-8-, UTF-16LE- oder UTF-16BE-Byte-Reihenfolgemarkierung bestimmt die Kodierung und wird am Dateianfang entfernt. Ohne BOM wird die Eingabe immer als UTF-8 behandelt. BOM-loses UTF-16 wird nicht erraten; NUL-Zeichen bleiben erhalten und werden als mögliche Kodierungsschäden gemeldet.
- Ungültige UTF-8-Sequenzen und ungepaarte UTF-16-Surrogate werden zu U+FFFD; `Problems` meldet den Schaden. Auch ein einzelnes abschließendes UTF-16-Byte und Lesefehler werden gemeldet; lesbare Zeilen bleiben erhalten. Prüfen Sie `Problems` und das Parserprotokoll, bevor Sie von Vollständigkeit ausgehen.
- Dateien werden als Datenstrom verarbeitet. Je physischer Zeile bleiben höchstens 1,048,576 Eingabebytes erhalten, ohne BOM und LF-Trennzeichen (ein CR vor LF zählt zum Limit). Längere Zeilen werden an einer vollständigen Zeichengrenze gekürzt; die übrigen Bytes werden eingelesen und `Problems` meldet die Anzahl ausgelassener Bytes. Das Limit gilt für kodierte Eingabebytes, nicht für die Zeichenzahl.
- LF- und CRLF-Zeilenenden werden entfernt, ebenso ein abschließendes CR einer nicht abgeschlossenen Zeile. Andere Leerzeichen und Tabulatoren bleiben erhalten. Leere Zeilen erzeugen keinen Datensatz, zählen aber bei den Zeilennummern mit und beenden eine Fortsetzungskette. Zeilen nur aus Leerraum bleiben Datensätze. Bei Lesefehlern kann eine leere Diagnosezeile markieren, wo das Lesen endete.
- Die Zeilen bleiben getrennt; Backticks und Einrückung bleiben erhalten. `EntryStartLine` verweist auf die erste Zeile einer zusammenhängenden Kette, deren vorherige Zeilen jeweils mit Backtick enden. Shell-Syntax wird nicht interpretiert, Befehle werden nicht rekonstruiert. Kürzungen können die Fortsetzungszuordnung beeinflussen; prüfen Sie `Problems` vor dem Gruppieren solcher Zeilen.

## CSV-Ausgabefelder { #csv-output-fields }

| Feld | Interpretation |
|---|---|
| `SourceFile` | Pfad relativ zur Verzeichniseingabe, mit `/`; bei `-f` nur der Dateiname. Es wird kein absoluter Quellpfad exportiert. |
| `User` | Profilname aus dem Pfad. Der Ordner oberhalb von `AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine` (oder `.local/share/powershell/PSReadLine`) benennt den Benutzer, auch wenn die Eingabe dieses Profil oder der `PSReadLine`-Ordner selbst ist. Verzeichniseingabe erkennt auch Strukturen mit `Users`/`Documents and Settings` unterhalb des Eingabestamms. Unbekannte Werte sind in CSV leer und in JSON `null`. |
| `ShellHost` | Dateinamenspräfix vor `_history.txt`, etwa `ConsoleHost`; bei umbenannten Dateien ohne dieses Suffix unbekannt. |
| `LineNumber` | Physische Zeilennummer ab 1, einschließlich übersprungener Leerzeilen. |
| `EntryStartLine` | Erste physische Zeile der Fortsetzungskette oder die Nummer dieser Zeile selbst. |
| `Command` | Dekodierte physische Zeile ohne Zeilenende, unter Beachtung des dokumentierten Limits. |
| `FileLastWriteTime` | Änderungszeit der Quelldatei, in jeder Zeile wiederholt. **Dateimetadaten, niemals Befehlsausführungszeit.** Bei Abbildern werden ursprüngliche Quellmetadaten statt der Staging-Zeit verwendet. |
| `FileSize` | Größe der Quelldatei in Bytes. |
| `Encoding` | `UTF-8`, `UTF-8-BOM`, `UTF-16LE` oder `UTF-16BE`. |
| `Problems` | Diagnosen zu Zeilenschäden und Kürzungen; in CSV durch Semikolon getrennt, in JSON/JSONL als Array. Dateibezogene Diagnosen stehen auch im Protokoll. |

JSON und JSONL haben dieselben Felder, mit typisierten Zahlen, `null` für unbekannte Werte und `Problems` als Liste. Jedes Format bewahrt den Befehlstext so, wie er gespeichert wurde.

`FileLastWriteTime` wird standardmäßig in UTC im ISO-8601-Format dargestellt, etwa `2026-10-20T08:46:38.5Z`. Nur `auto-parse` akzeptiert `-Z, --timezone`, um den angezeigten Offset zu ändern. Der eigenständige Befehl `parse-file powershell-history` hat keine Zeitzonenoption.

## Automatisches Parsen { #automatic-parsing }

`auto-parse` schreibt `ShellHistory/PowerShellHistory.csv` (`PowerShellHistory.json` bzw. `PowerShellHistory.jsonl` mit `-t json` bzw. `-t jsonl`), zusammen mit `ShellHistory/PowerShellHistory-logs.txt`. Aus VHDX- und E01-Abbildern werden die Verlaufsdateien kopiert, und `FileLastWriteTime` ist die im Abbild aufgezeichnete Zeit. Die kombinierte Zeitleiste `-j` enthält jede Zeile mit dem `Artifact`-Bezeichner `powershell-history`. Die Verlaufszeilen haben keine Zeitangaben, daher erzeugt [`super-timeline`](../commands/super-timeline.md) aus ihnen keine Ereignisse und zählt sie als Datensätze ohne Zeitangabe.

## Untersuchungsstrategie { #investigation-strategy }

1. Lesen Sie den Verlauf jedes Profils, einschließlich `Windows\System32\config\systemprofile` und der Profile unter `Windows\ServiceProfiles`, die Befehle enthalten, die in Shells unter diesen Konten eingegeben wurden.
2. Lesen Sie mehrzeilige Einträge zusammen, und suchen Sie nach Downloads, kodierten Befehlen, Änderungen an Sicherheitseinstellungen, Konto- und Anmeldedatenbefehlen sowie Remoteverbindungen.
3. Gleichen Sie die Ergebnisse mit den PowerShell-Ereignisprotokollen (Skriptblockprotokollierung, Ereignis 4104), Prozesserstellungsereignissen und den Prefetch-Dateien von `powershell.exe` und `pwsh.exe` ab.
4. Vergleichen Sie `FileLastWriteTime` mit den `$MFT`- und USN-Journal-Zeiten der Verlaufsdatei sowie mit Anmeldungen, um einzugrenzen, wann die letzten Zeilen hinzugefügt wurden.

## Weitere Informationen { #further-details }

- [Microsoft — about_PSReadLine: Befehlsverlauf](https://learn.microsoft.com/en-us/powershell/module/psreadline/about/about_psreadline)
- [Microsoft — Set-PSReadLineOption: HistorySavePath und HistorySaveStyle](https://learn.microsoft.com/en-us/powershell/module/psreadline/set-psreadlineoption)
- [Microsoft — about_Logging_Windows: Skriptblockprotokollierung](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [PSReadLine — wie die Verlaufsdatei gespeichert und gelesen wird](https://github.com/PowerShell/PSReadLine/blob/master/PSReadLine/History.cs)
