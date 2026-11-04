# UserAssist { #userassist }

## Was UserAssist ist { #what-userassist-is }

Der Windows-Explorer zählt die Programme und Verknüpfungen, die ein Benutzer über die Explorer-Shell startet: das
Startmenü, die Taskleiste, den Desktop und Doppelklicks in Ordnern. Er speichert die Zählerstände in der
`NTUSER.DAT` des Benutzers. Jedes Element hat:

- eine Ausführungsanzahl;
- unter Windows 7 und später, wie oft und wie lange (in Millisekunden) es den Fokus hatte;
- den Zeitpunkt der letzten Ausführung.

Die Wertnamen sind ROT13-kodiert. Der Name eines Programms beginnt meist mit der GUID eines bekannten Ordners
(`{1AC14E77-…}` ist `System32`).

UserAssist zeigt, was ein bestimmter Benutzer interaktiv gestartet hat – etwas, das die meisten anderen Ausführungsbelege
keinem Benutzer zuordnen.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Der Benutzer hat dieses Programm bzw. diese Verknüpfung
über den Explorer gestartet, so oft, zuletzt zu diesem Zeitpunkt. Auf andere Weise gestartete Programme (Befehls-
zeile, Dienst, geplante Aufgabe, anderes Programm) werden nicht gezählt.

## Speicherort { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | Enthält |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | Programme (`executable`), Windows 7 und später |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | Verknüpfungen (`shortcut`), Windows 7 und später |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | Programme und Verknüpfungen (`active_desktop`), Windows 2000 bis Vista |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | Internet-Symbolleiste (`internet_toolbar`), Windows 2000 bis Vista |

Andere GUIDs werden als `other` gemeldet. Sichern Sie `NTUSER.DAT` mit seinen Transaktionsprotokollen für jedes
Profil sowie den `SOFTWARE`-Hive des Systems, der die SID des Benutzers liefert.

## Hinweise für die Analyse { #analyst-cautions }

- **Neue Profile werden vorbelegt.** Unter Windows 7 und später beginnt ein neues Profil mit Standardelementen
  (Erste Schritte, Rechner, Paint, Kurznotizen, …). Ihre Ausführungsanzahlen zählen von 14 abwärts,
  und sie teilen sich alle denselben Zeitpunkt der letzten Ausführung, ungefähr zum Zeitpunkt der Profilerstellung. Sie sind keine Benutzer-
  aktivität.
- **Version 3 zählt ab 5** (Windows 2000 bis Vista). Der gespeicherte Zähler beginnt bei 5, daher wird 5
  davon abgezogen. Ein gespeicherter Zähler unter 5 wird in `Problems` gemeldet.
- **Pfade verwenden den Standardpfad des bekannten Ordners** (`%windir%\system32`, `%ProgramFiles%`,
  `%APPDATA%\…`). Ein an einen anderen Ort umgeleiteter Ordner zeigt weiterhin seinen Standardpfad. App-IDs
  (`Microsoft.Windows.Explorer`, `Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`) und Systemsteuerungs-
  elemente (`::{…}`) haben keinen Pfad.
- **Die Aufzeichnung kann deaktiviert werden** mit `Explorer\Advanced` `Start_TrackProgs` = 0,
  `Policies\Explorer` `NoInstrumentation` = 1 oder `UserAssist\Settings` `NoLog` = 1. Die Zusammenfassung
  listet die Hives auf, in denen sie deaktiviert ist. Ein leeres oder fehlendes UserAssist bei einem aktiven Profil kann auf
  Anti-Forensik hindeuten.
- **`NoEncrypt` = 1** veranlasst Windows, Namen ohne ROT13 zu schreiben. Solche Namen werden falsch dekodiert, daher
  vermerkt jedes Element dieses Hives dies in `Problems` (und auf stderr, wenn die Datensätze nach stdout gehen).
- **Zeitpunkt der letzten Ausführung und Zähler werden pro Element gespeichert,** aber die Zeit des letzten Schreibzugriffs auf den Schlüssel
  (`KeyLastWrittenUTC`) ist nur der Zeitpunkt der jüngsten Aktualisierung irgendeines Elements.
- **Ausführungsanzahlen können 0 sein** bei Elementen, die nur den Fokus hatten. Fokusanzahlen und -zeiten gibt es nur in
  Version 5 (Windows 7 und später).

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry user-assist <INPUT> [OPTIONS]

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
juntei parse-registry user-assist -d evidence/ -o userassist.csv
juntei parse-registry user-assist -f NTUSER.DAT -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Die Zusammenfassung listet die zuletzt ausgeführten Elemente (neueste zuerst) mit ihren Benutzern auf sowie die Hives, in denen
UserAssist deaktiviert ist.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, Name des Profilordners, SID des Benutzers aus der `ProfileList` des Systems (mit dem `SOFTWARE`-Hive des Systems) und der Hive. |
| `GUID`, `Kind` | Der `{GUID}`-Schlüssel und was er enthält (`executable`, `shortcut`, `active_desktop`, `internet_toolbar`, `other`). |
| `NameRot13`, `Name` | Der Name des Werts wie gespeichert und dekodiert. |
| `Path` | Der dekodierte Name als Pfad, wobei die GUID eines bekannten Ordners durch dessen Standardpfad ersetzt ist; leer bei App-IDs und Shell-Items. |
| `RunCount` | Wie oft es über den Explorer gestartet wurde (Version 3: der gespeicherte Zähler minus 5). |
| `FocusCount`, `FocusTimeMs` | Wie oft es den Fokus hatte und wie lange in Millisekunden (Version 5). |
| `LastRunUTC` | Wann es zuletzt gestartet wurde. Leer, wenn Windows 0 gespeichert hat oder wenn die gespeicherte Zeit nach 9999 liegt (dann gibt `Problems` sie an und `DataRaw` bewahrt die Bytes). |
| `Format`, `SessionID` | Das Layout der Daten (5: 72 Bytes, 3: 16 Bytes) und die Nummer der Sitzung, in der sie zuletzt geschrieben wurden. |
| `KeyLastWrittenUTC` | Letzter Schreibzugriff auf den `Count`-Schlüssel: die jüngste Aktualisierung eines seiner Elemente. |
| `DataRaw` | Die Bytes des Werts (hexadezimal), wenn sie sich nicht dekodieren lassen. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Daten unbekannter Größe, eine Größe, die nicht zur `Version` des Schlüssels passt, ein Zähler unter 5 in Version 3, eine letzte Ausführung nach 9999, `NoEncrypt`. |

## Untersuchungsstrategie { #investigation-strategy }

1. Sortieren Sie nach `LastRunUTC` und klammern Sie die vorbelegten Elemente aus: Sie teilen sich einen Zeitpunkt, ungefähr zur
   Erstellung des Profils.
2. Suchen Sie nach Programmen an für Benutzer beschreibbaren Orten (`%USERPROFILE%`, `Downloads`, `Temp`,
   `ProgramData`), auf Wechsellaufwerken und Netzwerkpfaden sowie nach Administrations- und Fernzugriffs-
   tools.
3. Korrelieren Sie mit Prefetch (Ausführung mit Ausführungszeiten), Amcache und ShimCache (Vorhandensein),
   Jump Lists und LNK-Dateien (was damit geöffnet wurde) sowie der `$MFT` für die Programmdateien.
4. Prüfen Sie die Hives, in denen die Aufzeichnung deaktiviert ist, und Profile mit leerem UserAssist auf Bereinigung.
5. Verwenden Sie `parse-registry read --recover-deleted` für gelöschte UserAssist-Werte.

## Weitere Informationen { #further-details }

- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: User Assist keys](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
