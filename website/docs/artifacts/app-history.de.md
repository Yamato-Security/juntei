# Anwendungsverlauf { #application-history }

## Was Tools hinterlassen { #what-tools-leave-behind }

Viele Tools von Drittanbietern speichern Einstellungen und Verläufe in der `NTUSER.DAT` des Benutzers. Mehrere davon gehören zu
den Tools, die Angreifer verwenden oder mit denen sie Daten vor der Exfiltration bereitstellen: Sysinternals-Tools, Archivprogramme,
Cloud-Synchronisierungsclients und Fernzugriffstools. Juntei liest folgende aus:

| Anwendung | Schlüssel unter `HKCU\Software` | Was er zeigt |
|---|---|---|
| Sysinternals | `Sysinternals\<tool>` | Jedes Tool, dessen Lizenz der Benutzer akzeptiert hat (`EulaAccepted`): PsExec, ProcDump, Sysmon, … |
| WinRAR | `WinRAR\ArcHistory`, `DialogEditHistory\ArcName`, `DialogEditHistory\ExtrPath` | Geöffnete Archive; in den Dialogen eingegebene Archivnamen und Extraktionsordner |
| 7-Zip | `7-Zip\Compression\ArcHistory`, `Extraction\PathHistory`, `FM\FolderHistory`, `FM\CopyHistory`, `FM\PanelPath0/1` | Erstellte Archive; Extraktionsordner; im Dateimanager durchsuchte Ordner und Kopierziele |
| WinZip | `Nico Mak Computing\WinZip\filemenu`, `extract`, `mru\archives` | Archive im Menü „Datei“, Extraktionsordner, zuletzt verwendete Archive |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | E-Mail-Adresse, lokaler Ordner und Namen jedes Kontos; synchronisierte Bibliotheken (`ScopeIdToMountPointPathCache`) und Ordner (`Tenants\<tenant>`) |
| Adobe Acrobat und Reader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`, `cRecentFolders` | Zuletzt geöffnete PDF-Dateien und Ordner mit Dateinamen, Größen und Datumsangaben |
| TeamViewer | `TeamViewer` (und `Version<N>` einer älteren Version) | Jeder Wert: Konto- und Benutzernamen, Startordner von Dateiübertragungen (`<partner ID>?<local>|<remote>`), … |
| CCleaner | `Piriform\CCleaner` | Jeder Wert: was bereinigt werden soll, `UpdateKey`, … |

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Dieser Benutzer hat das Tool ausgeführt (oder seine
Lizenz akzeptiert), oder das Tool hat diese Archive und Ordner geöffnet, erstellt oder extrahiert, oder dieses Cloud-Konto
wurde für den Benutzer eingerichtet. Die meisten Verläufe haben keine Zeit pro Eintrag: Nur der jüngste Eintrag erhält
die Zeit des letzten Schreibzugriffs auf den Schlüssel seiner Liste.

## Speicherort { #location }

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

Sichern Sie die `NTUSER.DAT` jedes Benutzers mit ihren Transaktionsprotokollen sowie den `SOFTWARE`-Hive des Systems für
die SIDs der Benutzer.

## Hinweise für die Analyse { #analyst-cautions }

- **Annahme der Sysinternals-EULA.** `FirstUsedUTC` ist eine Schätzung der ersten Nutzung anhand der Zeit des letzten
  Schreibzugriffs auf den Schlüssel, wenn dessen einziger Wert ein lesbares, auf `1` gesetztes `EulaAccepted` ist. Eine abgelehnte oder
  beschädigte EULA erhält diese Schätzung nicht. Registry-Einstellungen können zudem vorkonfiguriert,
  kopiert oder bearbeitet sein: Untermauern Sie die Ausführung und ihren Zeitpunkt mit anderen Artefakten.
  Tools, die dort Einstellungen speichern (Process Explorer, Autoruns, …), schreiben den Schlüssel später neu:
  `FirstUsedUTC` ist dann leer, und `KeyLastWrittenUTC` ist nur die letzte Änderung der Einstellungen. Ein Tool,
  das mit `-accepteula` oder als anderer Benutzer ausgeführt wird (die Remote-Seite von PsExec läuft als SYSTEM), schreibt den
  Schlüssel dieses Benutzers.
- **Positionen, keine Zeiten.** Listeneinträge sind geordnet (`MruPosition` 0 ist der jüngste), aber
  nur der letzte Schreibzugriff auf den Listenschlüssel datiert den jüngsten Eintrag. Ältere Einträge wurden früher verwendet, zu einem
  unbekannten Zeitpunkt.
- **Eingegebene Namen.** `ArcName` von WinRAR ist der beim Erstellen eines Archivs eingegebene Archivname: oft ein Dateiname
  ohne Ordner.
- **Adobe-Pfade** werden so beibehalten, wie Adobe sie schreibt (`/C/Users/alice/a.pdf` entspricht
  `C:\Users\alice\a.pdf`). Sein `sDate` (in `Details`) ist eine PDF-Datumszeichenfolge und wird unverändert beibehalten.
- **`UpdateKey` von CCleaner** ist Ortszeit ohne Zeitzone; er wird unverändert beibehalten.
- **TeamViewer.** Verbindungsprotokolle sind Dateien (`Connections_incoming.txt`, `TeamViewer*_Logfile.log`),
  keine Registry-Werte: Sichern Sie diese ebenfalls.
- **Fehlen von Einträgen.** Benutzer und Tools können diese Verläufe löschen (CCleaner selbst kann das).

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.App == "Sysinternals" and .FirstUsed != null)'
```

Die Zusammenfassung listet die neuesten Schätzungen der ersten Nutzung von Sysinternals-Tools, die von jeder Liste zuletzt
verwendeten Archive und Ordner sowie die OneDrive-Konten auf.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `SourcePath` | Host und `NTUSER.DAT`. |
| `User`, `UserSID` | Name des Profilordners und seine SID aus der `ProfileList` des Systems. |
| `App` | `Sysinternals`, `WinRAR`, `7-Zip`, `WinZip`, `OneDrive`, `Adobe Acrobat Reader`, `Adobe Acrobat`, `TeamViewer`, `CCleaner`. |
| `Item` | Das Sysinternals-Tool, die Liste (`ArcHistory`, `ExtrPath`, `FolderHistory`, `filemenu`, `cRecentFiles`, …), das OneDrive-Konto oder der Schlüssel seiner synchronisierten Ordner oder der Name des Werts. |
| `MRU-Position` | Position des Eintrags in seiner Liste, 0 ist der jüngste. |
| `Path` | Das Archiv, der Ordner oder das Dokument. |
| `Value` | `EulaAccepted`, die E-Mail-Adresse eines OneDrive-Kontos oder die ID einer synchronisierten Bibliothek, ein TeamViewer- oder CCleaner-Wert. |
| `FirstUsedUTC` | Schätzung der ersten Nutzung anhand der EULA-Annahme (siehe Hinweise oben). |
| `Details` | `DisplayName`, `UserCID`, `ConfiguredTenantId`, … eines OneDrive-Kontos; `Version`, `tFileName`, `uFileSize`, `uPageCount`, `sDate` eines Adobe-Dokuments. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName` | Herkunft des Datensatzes und Zeitpunkt der letzten Änderung dieses Schlüssels. |
| `DataRaw` | Die Bytes des Werts (hexadezimal), wenn sie sich nicht dekodieren lassen. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Daten, die nicht dekodiert oder gelesen werden konnten. |

## Untersuchungsstrategie { #investigation-strategy }

1. Suchen Sie nach Sysinternals-Tools, die ein gewöhnlicher Benutzer nicht ausführt (PsExec, ProcDump, PsLoggedOn,
   SDelete), und nach deren EULA-Zeitstempeln.
2. Prüfen Sie Archivnamen und Extraktionsordner rund um den Vorfall: Archive, die in
   `ProgramData`, `Temp` oder einem Benutzerordner erstellt wurden, bevor Daten das System verließen.
3. Prüfen Sie OneDrive-Konten, die nicht der Organisation gehören, sowie TeamViewer-Kontonamen und
   Ordner von Dateiübertragungen.
4. Korrelieren Sie mit Prefetch, Amcache, BAM und UserAssist (Ausführung des Tools), `$MFT` und USN-Journal
   (die Archivdatei), LNK-Dateien und Jump Lists (geöffnete Dokumente) sowie Ereignisprotokollen.

## Weitere Informationen { #further-details }

- [Microsoft: Sysinternals](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
