# ShellBags { #shellbags }

## Was ShellBags sind { #what-shellbags-are }

Der Explorer merkt sich, wie jeder von einem Benutzer geöffnete Ordner angezeigt wird: Ansicht, Symbolgröße, Sortierreihenfolge und
Fensterposition. Um die Einstellungen eines Ordners wiederzufinden, speichert er die Identität des Ordners in der Registry des Benutzers
als Baum von Shell-Items – dieselben Items, die die Ziel-ID-Liste einer Verknüpfung enthält: ein Laufwerk, dann jeder
Ordner bis hinunter zum geöffneten. Diese Bäume sind die ShellBags.

Sie umfassen Ordner, die im Explorer sowie in Dialogen zum Öffnen und Speichern geöffnet wurden, einschließlich Ordnern auf Wechsel-
laufwerken, Netzwerkfreigaben, tragbaren Geräten (Telefone, Kameras), ZIP-Archiven und FTP-Servern sowie
Seiten der Systemsteuerung. Sie bleiben erhalten, nachdem der Ordner, das Laufwerk oder die Netzwerkfreigabe verschwunden ist.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Der Explorer dieses Benutzers hat diesen Ordner angezeigt
(oder einen Ordner darunter). Ein ShellBag zeigt nicht, was in dem Ordner getan wurde, und es datiert nicht
jeden Besuch: siehe die Zeiten unten.

## Speicherort { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

Ein `BagMRU`-Schlüssel enthält nummerierte Werte, jeder ein Shell-Item, und einen `MRUListEx`-Wert, der ihre
Reihenfolge angibt, das zuletzt geöffnete zuerst. Jeder Wert hat einen gleichnamigen Unterschlüssel für die Items
darunter, sodass ein Pfad aus den Items von `BagMRU` bis hinunter zu einem Wert besteht. Das `NodeSlot` eines Unterschlüssels ist die
Nummer seines `Bags\<N>`-Schlüssels, in dem die Ansichtseinstellungen des Ordners liegen.

Sichern Sie `UsrClass.dat` und `NTUSER.DAT` mit ihren Transaktionsprotokollen (`.LOG1`, `.LOG2`) für jedes
Profil sowie den `SOFTWARE`-Hive des Systems, der die SID des Benutzers einer `NTUSER.DAT` liefert.

## Welche Zeiten was aussagen { #which-times-say-what }

- **Die Zeiten des Shell-Items** (`ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC`) sind die
  eigenen Dateisystemzeiten des Ordners (FAT-Genauigkeit: 2 Sekunden) zum Zeitpunkt, als der Explorer das Item schrieb. Sie
  datieren den Ordner, nicht den Besuch, und werden bei späteren Besuchen nicht aktualisiert.
  FAT-Uhren haben keinen gespeicherten UTC-Offset. Die althergebrachten `UTC`-Überschriften und das angezeigte Suffix `Z` sind
  Darstellungskonventionen, kein Beweis für die Zeitzone der Quelle. `auto-parse --timezone` bewahrt
  diese Felder unverändert; interpretieren Sie sie mit dem Kontext des Quellsystems.
- **Die Zeit des letzten Schreibzugriffs auf einen Schlüssel** ändert sich, wenn sein `MRUListEx` neu geschrieben wird, also wenn eines seiner
  Items geöffnet und nach vorn verschoben wird. Daher ist die Zeit des Schlüssels, in dem der Wert eines Items steht
  (`ParentKeyLastWrittenUTC`), der Zeitpunkt der letzten Öffnung des Items **nur dann, wenn es das jüngste
  Item in diesem Schlüssel ist** (`IsMostRecentInParent` true, `MruPosition` 0). Für die anderen Items ist es ein
  späterer Zeitpunkt, zu dem ein Geschwister-Item geöffnet wurde.
- Der eigene Schlüssel eines Items (`KeyLastWrittenUTC`) datiert auf dieselbe Weise die Öffnung seines jüngsten untergeordneten Items.

## Hinweise für die Analyse { #analyst-cautions }

- **Fehlen beweist wenig.** Der Explorer schreibt ShellBags, wenn die Ansicht eines Ordners gespeichert wird; ein Ordner,
  der nur kurz angezeigt oder auf andere Weise geöffnet wurde (Befehlszeile, Skript, Programm), hat möglicherweise keine.
  Ein Benutzer kann die Schlüssel löschen.
- **Eine NTFS-Referenz ist die des Ordners** (`MftEntry`, `MftSequence`) zum Zeitpunkt, als das Item geschrieben wurde.
  Schlagen Sie sie in der `$MFT` nach, um zu sehen, ob derselbe Ordner noch existiert oder eine andere Datei den
  Eintrag wiederverwendet hat.
- **Wechsellaufwerke** werden nach ihrem Laufwerksbuchstaben benannt. Es ist der Buchstabe, den das Laufwerk damals hatte;
  ordnen Sie ihn mit dem Verlauf von USB- und eingebundenen Geräten dem Gerät zu.
- **Namen stammen aus dem Item.** Ein später umbenannter Ordner behält hier seinen alten Namen. Stamm- und bekannte
  Ordner werden so benannt, wie forensische Tools sie üblicherweise benennen (`This PC`, `Quick Access`, `Downloads`);
  `Guid` liefert ihre Kennung.
- **Suchergebnisse, Bibliotheken und Seiten der Systemsteuerung** sind Ansichten, keine Ordner auf dem Datenträger
  (`property_view`, `control_panel_item`, `control_panel_category`).
- **Ein Item, das kein Parser kennt**, behält seine Bytes in `DataRaw`, mit `ItemType` `unknown` und seinem
  Klassenbyte in `ItemClass`. Beschädigte oder ungewöhnliche Items werden in `Problems` gemeldet, und auch ihre Bytes
  bleiben erhalten.
- **Einem „dirty“ Hive** können ohne seine Protokolle die neuesten Items fehlen. Juntei spielt die Protokolle neben dem
  Hive ein, sofern nicht `--no-logs` angegeben ist; `TransactionLogStatus` gibt an, ob ein Datensatz aus ihnen stammt.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shell-bags -d evidence/ -o shellbags.csv
juntei parse-registry shell-bags -f UsrClass.dat -t jsonl \
  | jq 'select(.ItemType == "volume" and .GUID == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

Shell-Items werden mit ys-lnk-parser dekodiert, dem Shell-Item-Parser von `parse-file lnk`. Die
Zusammenfassung listet die zuletzt geöffneten Ordner (neueste zuerst) anhand der Zeit des letzten Schreibzugriffs auf ihren Elternschlüssel auf,
zusammen mit ihren Benutzern.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath`, `HiveType` | Host, Name des Profilordners, SID des Benutzers (der Stammschlüssel einer `UsrClass.dat` benennt sie; bei `NTUSER.DAT` die `ProfileList` des Systems), der Hive und sein Typ. |
| `BagKey` | Der Schlüssel des Items unterhalb des Hive-Stamms (`…\BagMRU\4\3\0`): Sein Wert steht im darüberliegenden Schlüssel. |
| `MRU-Position`, `IsMostRecentInParent` | Seine Position im `MRUListEx` des Elternschlüssels (0: das zuletzt geöffnete); leer, wenn die Liste es nicht enthält. |
| `NodeSlot` | Die Nummer seines `Bags\<N>`-Schlüssels (die Ansichtseinstellungen des Ordners). |
| `Path` | Die Items von der Spitze seines Baums bis zu ihm, verbunden mit `\`. `[unknown]` steht für ein Item, dessen Wert fehlt. |
| `ItemType`, `ItemClass` | Was das Item ist (`root_folder`, `volume`, `directory`, `file`, `network_location`, `uri`, `ftp_folder`, `compressed_folder`, `portable_device`, `portable_device_folder`, `users_files_folder`, `property_view`, `control_panel_item`, `control_panel_category`, `cd_burn`, `unknown`), und sein Klassenbyte. |
| `ItemName`, `GUID` | Sein Name (der lange Name eines Ordners) und die Klassenkennung eines Stammordners, Systemsteuerungselements oder Delegatordners. |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | **Aus dem Shell-Item:** die Zeiten des Ordners zum Zeitpunkt, als das Item geschrieben wurde (FAT, 2 Sekunden Genauigkeit). |
| `MFT-Entry`, `MFT-Sequence` | **Aus dem Shell-Item:** die NTFS-Dateireferenz des Ordners. |
| `ExtensionBlocks` | Wie viele Erweiterungsblöcke das Item hat. |
| `KeyLastWrittenUTC` | **Aus der Registry:** der letzte Schreibzugriff auf den eigenen Schlüssel des Items: wann sein jüngstes untergeordnetes Item geöffnet wurde. |
| `ParentKeyLastWrittenUTC` | **Aus der Registry:** der letzte Schreibzugriff auf den Schlüssel, in dem sein Wert steht: wann es zuletzt geöffnet wurde, sofern `IsMostRecentInParent` true ist. |
| `DataRaw` | Die Bytes des Werts (hexadezimal) bei einem Item, das nicht vollständig verstanden wird oder Probleme aufweist. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Ein Wert, den das `MRUListEx` seines Schlüssels nennt, den der Schlüssel aber nicht hat, ein Wert, den die Liste nicht nennt, ein Unterschlüssel ohne seinen Wert, ein Wert, der nicht binär ist, ein `NodeSlot` des falschen Typs, eine beschädigte Shell-Item-Liste. |

## Untersuchungsstrategie { #investigation-strategy }

1. Listen Sie Wechsellaufwerke (`volume` mit der `Guid` für Removable Drives), Netzwerkorte, FTP-
   Ordner und tragbare Geräte sowie die Ordner darunter auf.
2. Suchen Sie nach relevanten Ordnern: für Benutzer beschreibbare Staging-Orte (`Temp`, `ProgramData`, `Public`),
   Tool- und Archivordner, Profile anderer Benutzer und nicht mehr existierende Ordner (prüfen Sie die
   NTFS-Referenz in der `$MFT`).
3. Datieren Sie Besuche nur mit `ParentKeyLastWrittenUTC` von Items, bei denen `IsMostRecentInParent` true ist;
   korrelieren Sie den Rest mit LNK-Dateien, Jump Lists und RecentDocs, die das Öffnen von Dateien datieren.
4. Verwenden Sie `parse-registry read --recover-deleted` für gelöschte `BagMRU`-Schlüssel und -Werte.

## Weitere Informationen { #further-details }

- [libyal libfwsi: Windows-Shell-Item-Format](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb: Most recently used (BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

Die SID-Ermittlung aus `ProfileList` erfordert genau eine eindeutige, syntaktisch gültige SID. Mehrdeutige Zuordnungen von Profilordnern und Sicherungsschlüssel wie `S-1-...bak` lassen `UserSid` unbekannt.
