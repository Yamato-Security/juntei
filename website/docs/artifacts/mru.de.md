# MRU-Listen (RunMRU, TypedPaths, WordWheelQuery, TypedURLs) { #most-recently-used-lists-runmru-typedpaths-wordwheelquery-typedurls }

## Was die Listen sind { #what-the-lists-are }

Windows und seine Programme führen in der `NTUSER.DAT` des Benutzers kurze Listen darüber, was ein Benutzer zuletzt eingegeben oder
geöffnet hat:

- **RunMRU:** Befehle, die in das Dialogfeld „Ausführen“ (Win+R) eingegeben wurden: `cmd`, `powershell -ep bypass`,
  `\\server\share`, `regedit`.
- **TypedPaths:** in die Adressleiste des Explorers eingegebene Pfade.
- **WordWheelQuery:** im Suchfeld des Explorers gesuchte Begriffe.
- **TypedURLs:** in die Adressleiste des Internet Explorer eingegebene Adressen und seit Windows 8
  `TypedURLsTime`, wann jede davon eingegeben wurde.
- **Paint und WordPad:** die zuletzt von jedem Programm geöffneten Dateien (`Recent File List`).
- **ACMru (Windows XP):** die Begriffe des Such-Assistenten, je Suchart.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Der Benutzer hat diesen Befehl, Pfad,
Suchbegriff oder diese Adresse eingegeben, oder das Programm hat diese Datei geöffnet. Nur der jüngste Eintrag jeder Liste
ist datiert, und zwar durch die letzte Schreibzeit seines Schlüssels; ausgenommen sind eingegebene Adressen, die `TypedURLsTime` datiert.

## Speicherort { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

Die Unterschlüssel von `ACMru` sind die Sucharten: `5603` Datei- und Ordnernamen, `5604` Wörter oder Ausdrücke in
Dateien, `5647` Computer und Personen, `5001` das Internet.

Sichern Sie `NTUSER.DAT` mit seinen Transaktionsprotokollen für jedes Profil.

## Hinweise für die Analyse { #analyst-cautions }

- **Eine Zeit pro Liste.** Die letzte Schreibzeit eines Schlüssels ändert sich, wenn sich seine Liste ändert; sie datiert also nur den
  jüngsten Eintrag (`IsMostRecent`, `LastUsedUTC`). Die übrigen sind geordnet, aber undatiert.
  `TypedURLsTime` datiert jede eingegebene Adresse (`TypedTimeUTC`).
- **Eingegeben, nicht ausgeführt.** RunMRU zeichnet auf, was in das Dialogfeld „Ausführen“ eingegeben wurde, unabhängig davon, ob es
  erfolgreich ausgeführt wurde. Windows speichert jeden Befehl mit einem nachgestellten `\1`; Juntei entfernt es. Korrelieren Sie
  mit Prefetch, BAM, UserAssist und Ereignissen zur Prozesserstellung.
- **Die Listen sind kurz.** Jede behält nur ihre jüngsten Einträge, und Datenschutzeinstellungen sowie Bereinigungs-
  werkzeuge leeren sie. Verwenden Sie `parse-registry read --recover-deleted` für gelöschte Werte.
- **Begriffe sind das Eingegebene.** WordWheelQuery und ACMru speichern den eingegebenen Text, nicht das Gefundene.
- **Die Reihenfolge stammt aus der Liste.** `MRUList` von RunMRU und `MRUListEx` von WordWheelQuery geben die
  Reihenfolge an; bei den übrigen tut dies die Nummer des Werts (`url1`, `File1` und `000` sind die jüngsten).

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

Die Zusammenfassung führt die Einträge je Liste sowie die zuletzt datierten Einträge (eingegebene Adressen und den jeweils
jüngsten Eintrag jeder Liste) auf, die neuesten zuerst.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, der Profilbesitzer (aus dem Pfad oder den Shell-Ordnern), die SID aus der `ProfileList` des Systems und der Hive. |
| `List` | `RunMRU`, `TypedPaths`, `WordWheelQuery`, `TypedURLs`, `Paint`, `Wordpad` oder `ACMru`. |
| `Subkey` | Unterschlüssel von `ACMru`: die Suchart (`5603`, `5604`, `5647`, `5001`). |
| `MRU-Position`, `IsMostRecent` | Die Position in der Liste (0 ist die jüngste); leer, wenn die Liste den Wert nicht benennt. |
| `ValueName` | `a`, `0`, `url1`, `File1`, `000`, … |
| `Value` | Der Befehl (ohne sein `\1`), Pfad, Suchbegriff, Adresse oder Datei. |
| `TypedTimeUTC` | Wann die Adresse eingegeben wurde, aus `TypedURLsTime`. |
| `LastUsedUTC` | Die letzte Schreibzeit des Schlüssels, nur für den jüngsten Eintrag. |
| `KeyPath`, `KeyLastWrittenUTC` | Der Schlüssel und sein letzter Schreibvorgang: die Zeit seines jüngsten Eintrags, nicht jeder Zeile. |
| `DataRaw` | Die Bytes des Werts (hex), wenn sie sich nicht dekodieren lassen. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Werte, die die Liste benennt, die der Schlüssel aber nicht enthält, Werte, die sie nicht benennt, falsch benannte Werte, falsche Typen, abgeschnittene Begriffe, Zeiten, die keine FILETIMEs sind. |

## Untersuchungsstrategie { #investigation-strategy }

1. Lesen Sie RunMRU auf Befehle, Skripte, Netzwerkfreigaben und Administrationswerkzeuge, die der Benutzer eingegeben hat.
2. Lesen Sie TypedPaths und WordWheelQuery auf Ordner und Dateien, nach denen der Benutzer gesucht hat:
   Anmeldedaten, Exporte, Archive, Wechseldatenträger und Netzwerkspeicherorte.
3. Nehmen Sie eingegebene Adressen mit `TypedTimeUTC` in die Zeitleiste auf.
4. Korrelieren Sie mit Prefetch, BAM, UserAssist, ShellBags, LNK-Dateien, Jump Lists und dem Browserverlauf.

## Weitere Informationen { #further-details }

- [libyal winreg-kb: Most recently used (MRU) keys](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
