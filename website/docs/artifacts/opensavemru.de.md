# Dialoge zum Öffnen und Speichern unter (OpenSaveMRU) { #open-and-save-as-dialogs-opensavemru }

## Was der Dialogverlauf ist { #what-the-dialog-history-is }

Wenn ein Programm den allgemeinen Dialog „Öffnen“ oder „Speichern unter“ anzeigt, zeichnet Windows in der
`NTUSER.DAT` des Benutzers Folgendes auf:

- **welche Datei geöffnet oder gespeichert wurde:** `OpenSavePidlMRU` hat einen Unterschlüssel pro Dateierweiterung (`docx`,
  `ps1`, …) und `*` für alle Erweiterungen, jeweils als Liste der zuletzt verwendeten Elemente (MRU);
- **welches Programm den Dialog verwendet hat und in welchem Ordner er sich befand:** `LastVisitedPidlMRU` (und
  `LastVisitedPidlMRULegacy` für Programme, die die älteren Dialoge verwenden);
- **Programme, die den Dialog angezeigt haben:** `CIDSizeMRU` (die Größe des Dialogs pro Programm) und
  `FirstFolder` (der Pfad eines Programms und ein Ordner);
- unter Windows 2000 und XP `OpenSaveMRU` (Pfade pro Erweiterung) und `LastVisitedMRU` (Programme und
  ihre Ordner als Pfade).

Seit Windows Vista sind die Dateien und Ordner Shell-Item-Listen, also dieselben Strukturen wie das Ziel einer
Verknüpfung: Jedes Element ist ein Teil des Pfads (Dieser PC, ein Laufwerk, ein Ordner, die Datei), und
ein Datei- oder Ordnerelement kann eigene Zeitstempel und eine NTFS-Dateireferenz enthalten.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Ein Programm hat in der Sitzung dieses Benutzers den Dialog „Öffnen“ oder „Speichern unter“ verwendet,
um diese Datei zu öffnen oder zu speichern bzw. in diesem Ordner. Nur der neueste
Eintrag jeder Liste ist datiert: durch den Zeitpunkt des letzten Schreibzugriffs auf seinen Schlüssel.

## Speicherort { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

Sichern Sie `NTUSER.DAT` mit seinen Transaktionsprotokollen für jedes Profil.

## Hinweise für die Analyse { #analyst-cautions }

- **Ein Zeitstempel pro Liste.** Der Zeitpunkt des letzten Schreibzugriffs auf einen Schlüssel ändert sich, wenn einer seiner Einträge an den
  Anfang seiner Liste rückt; er datiert daher nur den Eintrag an Position 0 (`IsMostRecent`, `LastUsedUTC`).
  Die übrigen Einträge sind geordnet, vom neuesten zum ältesten absteigend, aber undatiert.
- **Dieselbe Datei in zwei Listen.** Eine Datei steht in der Liste ihrer Erweiterung und in `*`; zählen Sie sie nicht
  als zwei Verwendungen. `*` liefert die Reihenfolge über alle Erweiterungen hinweg.
- **Geöffnet oder gespeichert.** Die Listen sagen nicht, was davon zutrifft. Korrelieren Sie mit den `$MFT`- und USN-Journal-Datensätzen
  der Datei, LNK-Dateien und Jump Lists.
- **Elementzeitstempel gehören zur Datei, nicht zum Dialog.** `ItemModifiedUTC`, `ItemCreatedUTC` und
  `ItemAccessedUTC` sind die Zeitstempel der Datei zum Zeitpunkt, als das Element geschrieben wurde, mit einer Genauigkeit von 2 Sekunden (FAT). Elemente
  bekannter Ordner (`CLSID_ThisPCDocumentsRegFolder`, `Downloads`, …) haben oft keine. Diese FAT-Zeitstempel
  besitzen keinen Zeitzonen-Offset. Das Spaltensuffix `UTC` ist eine Darstellungskonvention, kein
  Beleg dafür, dass die ursprüngliche Uhr in UTC lief; korrelieren Sie mit der Zeitzone des Quellsystems und
  anderen Zeitstempeln, bevor Sie sie mit Registry-FILETIMEs vergleichen. Siehe Microsofts
  [Hinweise zu Dateizeiten](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times).
- **Pfade sind Shell-Pfade.** Sie beginnen beim Desktop: `This PC\C:\Users\…`, ein bekannter Ordner, eine
  Bibliothek (`UsersLibraries\Documents\…`) oder ein Netzwerkspeicherort. Ein Pfad, der mit einem Ordner
  oder einer Datei beginnt, liegt auf dem Desktop. Ein leerer Pfad ist der Desktop selbst.
- **Nicht dekodierte Elemente.** Ein Element eines Typs, den der Shell-Item-Parser nicht kennt, wird durch seine
  Klasse angezeigt (`[class 0x00]`), sein Typ ist `unknown`, und die Bytes der Liste stehen in `DataRaw`.
- **Programme sind Namen.** `LastVisitedPidlMRU` und `CIDSizeMRU` speichern einen Dateinamen
  (`WINWORD.EXE`), keinen Pfad. `FirstFolder` speichert einen Pfad.
- **Listen sind kurz und werden bereinigt.** Jede Liste enthält nur eine begrenzte Anzahl von Einträgen, und Bereinigungswerkzeuge
  löschen die Schlüssel. Verwenden Sie `parse-registry read --recover-deleted` für gelöschte Werte.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry open-save-mru -d evidence/ -o opensavemru.csv
juntei parse-registry open-save-mru -f NTUSER.DAT -t jsonl \
  | jq 'select(.List == "OpenSavePidlMRU" and .Extension != "*")'
```

Die Zusammenfassung listet die Einträge pro Liste, die Programme, die die Dialoge verwendet haben, und die neueste
Datei jedes Hives (den ersten Eintrag von `*`) auf.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, der Profilinhaber (aus dem Pfad oder den Shell-Ordnern), die SID aus der `ProfileList` des Systems und der Hive. |
| `List` | `OpenSavePidlMRU`, `LastVisitedPidlMRU`, `LastVisitedPidlMRULegacy`, `CIDSizeMRU`, `FirstFolder`, `OpenSaveMRU` oder `LastVisitedMRU`. |
| `Extension` | Der Unterschlüssel der Erweiterung (`docx`, `*`); leer bei den übrigen Listen. |
| `MRU-Position`, `IsMostRecent` | Die Position in der Liste des Schlüssels (0 ist der neueste); leer, wenn die Liste den Wert nicht nennt. |
| `ValueName` | Die Nummer (`MRUListEx`) oder der Buchstabe (`MRUList`) des Werts. |
| `Program` | Das Programm, das den Dialog angezeigt hat. |
| `Path` | Die Datei oder der Ordner: die verbundenen Namen der Shell-Items oder der als Text gespeicherte Pfad. |
| `ItemType` | Der Typ des letzten Shell-Items: `file`, `directory`, `root_folder`, `volume`, `network_location`, `property_view`, `unknown`, … |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | Die Zeitstempel der Datei oder des Ordners, wie ihr Shell-Item sie aufgezeichnet hat. |
| `MFT-Entry`, `MFT-Sequence` | Die NTFS-Dateireferenz, die das Shell-Item aufgezeichnet hat. |
| `LastUsedUTC` | Der Zeitpunkt des letzten Schreibzugriffs auf den Schlüssel, nur für den neuesten Eintrag. |
| `KeyPath`, `KeyLastWrittenUTC` | Der Schlüssel und sein letzter Schreibzugriff: der Zeitpunkt seines neuesten Eintrags, nicht jeder Zeile. |
| `DataRaw` | Die Bytes des Werts (hexadezimal), wenn sie sich nicht dekodieren lassen, oder die Shell-Item-Liste, wenn der Typ eines Elements unbekannt ist. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Werte, die die Liste nennt, die der Schlüssel aber nicht hat, Werte, die sie nicht nennt, falsche Typen, abgeschnittene Namen oder Listen. |

## Untersuchungsstrategie { #investigation-strategy }

1. Lesen Sie `*` pro Benutzer in der Reihenfolge, und `LastUsedUTC` für die neueste Datei.
2. Suchen Sie nach Skripten, Archiven, ausführbaren Dateien und Dokumenten an für Benutzer beschreibbaren Orten, auf Wechselmedien oder im
   Netzwerk sowie nach Dateien, deren Namen auf Anmeldedaten oder Exporte hindeuten.
3. Verwenden Sie `LastVisitedPidlMRU`, um ein Programm mit dem zuletzt verwendeten Ordner zu verknüpfen, sowie `CIDSizeMRU` und
   `FirstFolder` für Programme, die die Dialoge überhaupt verwendet haben.
4. Korrelieren Sie die Pfade und `MftEntry` mit der `$MFT`, dem USN-Journal, LNK-Dateien, Jump Lists,
   ShellBags und RecentDocs.

## Weitere Informationen { #further-details }

- [libyal winreg-kb: Schlüssel der zuletzt verwendeten Elemente (MRU)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi: Format der Windows Shell Items](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
