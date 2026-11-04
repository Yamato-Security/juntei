# RecentDocs { #recentdocs }

## Was RecentDocs ist { #what-recentdocs-is }

Der Windows-Explorer speichert in der `NTUSER.DAT` jedes Benutzers Listen der zuletzt verwendeten Dateien und Ordner (MRU).
Die Stammliste umfasst zuletzt verwendete Elemente aller Typen; Unterschlüssel gruppieren Erweiterungen wie `.docx` sowie eine
`Folder`-Liste. `MRUListEx` gibt die Reihenfolge der nummerierten Werte an.

RecentDocs ist nützlich, um Dokumente zu identifizieren, mit denen ein Benutzer interagiert hat, einschließlich Dateien, die
später gelöscht oder verschoben wurden. Werte können außerdem den Verknüpfungsnamen enthalten, den Windows im Recent-Ordner des Benutzers
verwendet hat, was sich mit LNK-Belegen korrelieren lässt.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Der Explorer hat das Element in der
RecentDocs-Liste dieses Benutzers aufgezeichnet. Die Schlüsselzeit datiert nur das jüngste Element in genau dieser Liste.

## Speicherort { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

Sichern Sie `NTUSER.DAT` und die zugehörigen Transaktionsprotokolle für jedes relevante Profil sowie das Recent-LNK-Verzeichnis
des Benutzers.

## Hinweise für die Analyse { #analyst-cautions }

- `MRUListEx` gibt die Reihenfolge an, nicht für jedes Element einen Zeitstempel.
- Der Zeitpunkt des letzten Schreibzugriffs auf den Schlüssel entspricht der jüngsten Aktualisierung; nur MRU-Position 0 erhält einen
  daraus abgeleiteten, belastbaren `LastOpenedUTC`.
- Ein Element kann in der Stammliste und in einer Erweiterungsliste erscheinen. Zählen Sie es nicht doppelt als zwei Öffnungsvorgänge.
- Das Verhalten von Anwendungen und Explorer kann variieren; manche Zugriffsmethoden aktualisieren RecentDocs möglicherweise nicht.
- Bereinigungs-/Datenschutzwerkzeuge und Größenbeschränkungen der Listen entfernen ältere Belege.
- Ein Dateiname kann vom Angreifer kontrolliert sein und ist kein Beweis für den Inhaltstyp.
- Die Benutzerzuordnung stammt aus Profilpfaden oder Shell-Ordner-Daten; prüfen Sie leere/mehrdeutige Benutzernamen.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry recent-docs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recent-docs -d evidence/ -t jsonl \
  | jq 'select(.Extension == ".docx" or .Extension == ".pdf")'
```

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host` | Host, sofern verfügbar aus dem Sicherungs-/Systemkontext abgeleitet. |
| `User` | Aus dem Pfad oder aus Shell-Ordner-Pfaden abgeleiteter Profilinhaber; Unsicherheit beibehalten, wenn leer. |
| `SourcePath` | Eingabe-`NTUSER.DAT`. Unverzichtbar beim Zusammenführen mehrerer Benutzer. |
| `Extension` | Unterliste (`.docx`, `Folder` usw.); null für die RecentDocs-Stammliste. |
| `MRU-Position` | Null ist das jüngste Element innerhalb dieser Liste. Dies ist eine relative Reihenfolge, keine verstrichene Zeit. |
| `ValueIndex` | Von `MRUListEx` referenzierter nummerierter Registry-Wert. Nur struktureller Ansatzpunkt. |
| `Filename` | Dekodierter Elementname. Den vollständigen Pfad über LNK, Jump List, MFT oder Anwendungsdaten korrelieren. |
| `LinkName` | Im Shell-Element kodierter Verknüpfungsname, häufig im Recent-Verzeichnis des Benutzers zu finden. |
| `KeyLastWrittenUTC` | Zeitpunkt der letzten Änderung des Listenschlüssels. Gilt für die Liste, nicht für jede Zeile. |
| `LastOpenedUTC` | Nur für MRU-Position 0 befüllt, wo die Schlüsselzeit diese Interpretation stützt. |
| `ValueDataRaw` | Rohbytes, wenn die Dekodierung fehlschlägt oder Belege validiert werden müssen. |
| `TransactionLogStatus`, `LogsApplied` | Ob Registry-Protokolle eingespielt wurden und welche. |
| `Problems` | Fehlende MRU-Werte, nicht aufgeführte Werte, falsche Typen oder Dekodierungsschäden. |

## Untersuchungsstrategie { #investigation-strategy }

1. Trennen Sie die Stamm- und Erweiterungslisten jedes Benutzers; deduplizieren Sie dasselbe Element sorgfältig.
2. Verwenden Sie die MRU-Position für die Reihenfolge und nur `LastOpenedUTC` für eine Aussage mit Zeitstempel.
3. Gleichen Sie `LinkName`/`Filename` mit Recent-LNK-, Jump-List-, `$MFT`- und USN-Daten ab.
4. Prüfen Sie Täuschungsnamen, doppelte Erweiterungen, Netzwerk-/Wechseldatenträgerziele und Dokumente im zeitlichen Umfeld des Erstzugriffs.
5. Korrelieren Sie mit Prefetch/SRUM der Anwendung und Ereignisprotokollen, wenn die Ausführung relevant ist.

## Weitere Informationen { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [libyal libfwsi](https://github.com/libyal/libfwsi)
- [RECmd und Registry Explorer von Eric Zimmerman](https://github.com/EricZimmerman/RECmd)
- [SANS-Poster „Windows Forensic Analysis“](https://www.sans.org/posters/windows-forensic-analysis/)
