# Anwendungskompatibilität { #application-compatibility }

## Was Windows aufbewahrt { #what-windows-keeps }

Neben dem ShimCache (`parse-registry shimcache`) und Amcache (`parse-registry amcache`)
bewahren die Funktionen zur Anwendungskompatibilität von Windows Folgendes unter
`Microsoft\Windows NT\CurrentVersion\AppCompatFlags` auf:

| Quelle | Hive und Schlüssel | Was sie zeigt |
|---|---|---|
| `layers` | `SOFTWARE` (und `Wow6432Node`) sowie `NTUSER.DAT` `Layers` | Programme, die mit Kompatibilitätsebenen ausgeführt werden sollen: `RUNASADMIN` (immer als Administrator), `WIN7RTM`, `HIGHDPIAWARE`, … |
| `pca_store` | `NTUSER.DAT` `Compatibility Assistant\Store` (Windows 8 und neuer) | Programme, deren Ausführung durch den Benutzer der Programmkompatibilitäts-Assistent beobachtet hat |
| `pca_persisted` | `NTUSER.DAT` `Compatibility Assistant\Persisted` (Windows 7) | Dasselbe unter Windows 7 |
| `cit_module` | `SOFTWARE` `CIT\Module\<module>` | Programme, die ein verfolgtes Modul geladen haben, und wann dies zuletzt geschah |
| `cit_system` | `SOFTWARE` `CIT\System` (Windows 7 bis frühe Windows-10-Versionen) | Programmpfade und Befehlszeilen in den komprimierten CIT-Datenbanken |
| `installed_sdb` | `SOFTWARE` `InstalledSDB\{GUID}` | Mit `sdbinst` installierte benutzerdefinierte Shim-Datenbanken: Datei, Beschreibung, Installationszeit |
| `custom_sdb` | `SOFTWARE` `Custom\<program>` | Die auf jedes Programm angewendeten benutzerdefinierten Shim-Datenbanken mit ihren Installationszeiten |

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Ebenen und Shim-Datenbanken sind
Konfiguration, die ändert, wie ein Programm ausgeführt wird; Einträge des Programmkompatibilitäts-Assistenten und CIT-Einträge
zeigen, dass ein Programm als dieser Benutzer bzw. auf diesem System ausgeführt wurde (oder ein Modul geladen hat).

Benutzerdefinierte Shim-Datenbanken sind eine Persistenz- und Injektionstechnik (MITRE ATT&CK T1546.011): Ein Shim
kann bei jeder Ausführung des Programms eine DLL in das Programm laden oder ein anderes Programm starten.

## Speicherort { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

Sichern Sie `SOFTWARE`, jede `NTUSER.DAT` samt Transaktionsprotokollen sowie die `.sdb`-Dateien, die der Pfad
einer Shim-Datenbank benennt (in der Regel `C:\Windows\AppPatch\Custom\`).

## Hinweise für die Analyse { #analyst-cautions }

- **Die CIT-Dekomprimierung ist begrenzt.** Werte, die unkomprimiert mehr als 64 MiB angeben, werden
  als Rohbytes mit einer Diagnosemeldung beibehalten. Fehlerhafte Komprimierung und Größenabweichungen werden gemeldet.


- **Keine Ausführungszeit für Einträge des Programmkompatibilitäts-Assistenten.** Die `Store`-Daten (`SACP`, in
  `DataRaw` abgelegt) enthalten eine Zeit, doch unter Windows 8.1, 10 und 11 ist sie für jedes Programm eines
  Hives gleich, und auch auf anderen Systemen ist sie nicht der Zeitpunkt der Programmausführung. Nur der letzte Schreibzugriff
  auf den `Store`-Schlüssel datiert die jüngste Änderung.
- **Programme auf Wechselmedien** werden als `SIGN.MEDIA=<id> <path>` benannt: `Path` ist der Pfad auf dem
  Medium, und `Details` enthält die Medien-ID.
- **CIT-Datenbanken** sind LZNT1-komprimiert, und ihre Struktur ist nicht öffentlich dokumentiert: Juntei
  listet die in jeder Datenbank gefundenen Programmpfade und Befehlszeilen einmal pro Datenbank auf, ohne Zähler oder
  Zeiten. Eine Befehlszeile ist ein eigener Datensatz und nicht an ihr Programm gebunden. Ab Windows 10 wird hier
  wenig aufbewahrt.
- **Shim-Datenbanken.** Prüfen Sie jeden `custom_sdb`- und `installed_sdb`-Datensatz: Legitime Software
  installiert selten eine solche Datenbank. Die `.sdb`-Datei selbst gibt an, was der Shim bewirkt.
- **Ebenen**, die ein Benutzer festlegt, gelten nur für diesen Benutzer; die in `SOFTWARE` gelten für alle. `RUNASADMIN` bewirkt, dass ein
  Programm jedes Mal eine Rechteerhöhung anfordert.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry app-compat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-compat -d evidence/ -o appcompat.csv
juntei parse-registry app-compat -d evidence/ -t jsonl \
  | jq 'select(.Source == "installed_sdb" or .Source == "custom_sdb")'
```

Die Zusammenfassung listet die benutzerdefinierten Shim-Datenbanken, die Programme mit Ebenen und die letzten
Verwendungen der CIT-Module auf.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, Hive-Datei und `SOFTWARE` oder `NTUSER.DAT`. |
| `User`, `UserSID` | Name des Profilordners eines Benutzer-Hives und dessen SID aus der `ProfileList` des Systems. |
| `Source`, `Scope` | `layers`, `pca_store`, `pca_persisted`, `cit_module`, `cit_system`, `installed_sdb`, `custom_sdb`; `machine` oder `user`. |
| `Path` | Das Programm (der Name des Werts), ein Pfad oder eine Befehlszeile in einer CIT-Datenbank oder das Programm, auf das eine Shim-Datenbank angewendet wird. |
| `Layers` | Die Kompatibilitätsebenen wie geschrieben (`~ RUNASADMIN`). |
| `LastSeenUTC` | Wann ein Programm zuletzt ein CIT-Modul geladen hat. |
| `SdbGUID`, `SdbPath`, `SdbInstalledUTC` | GUID, Datei und Installationszeit einer Shim-Datenbank. |
| `Details` | Name eines CIT-Moduls, Wertname einer CIT-Datenbank, Wechselmedien-ID eines Programms (`SignMedia`), `DatabaseDescription` und `DatabaseType` einer Shim-Datenbank. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName`, `WOW64` | Woher der Datensatz stammt, wann dieser Schlüssel zuletzt geändert wurde und ob er unter `Wow6432Node` liegt. |
| `DataRaw` | Die `SACP`-Daten des `Store`-Werts oder Daten, die nicht der Erwartung entsprechen (hex). |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Daten, die nicht dekodiert oder gelesen werden konnten. |

## Untersuchungsstrategie { #investigation-strategy }

1. Prüfen Sie jede benutzerdefinierte Shim-Datenbank: ihre `.sdb`-Datei, die Installationszeit und die Programme, auf die sie
   angewendet wird; korrelieren Sie mit `sdbinst.exe` in Prefetch, Amcache und Prozesserstellungsereignissen.
2. Suchen Sie unter den Einträgen des Programmkompatibilitäts-Assistenten nach Programmen in für Benutzer beschreibbaren Ordnern
   sowie nach Programmen von Wechselmedien.
3. Prüfen Sie `RUNASADMIN` und andere Ebenen bei ungewöhnlichen Programmen.
4. Korrelieren Sie für Ausführungszeiten mit ShimCache, Amcache, BAM, Prefetch und UserAssist.

## Weitere Informationen { #further-details }

- [Microsoft: Anwendungskompatibilitätsdatenbank](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011: Application Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft: MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
