# Aufgabenplaner-Cache { #task-scheduler-cache }

## Was TaskCache ist { #what-taskcache-is }

Die Windows-Aufgabenplanung führt unter `SOFTWARE` einen Registry-Cache, der Aufgabenpfade mit GUIDs,
Triggern/Listen, Prinzipalen, Aktionen, Laufzeitmetadaten und Sicherheitsbeschreibungen verknüpft. Juntei dekodiert
Ausführungs- und COM-Handler-Aktionen sowie veraltete E-Mail-/Nachrichtenaktionen, Zeitstempel der Aufgaben und Anzeichen für
Aufgaben, die vor der normalen Oberfläche der Aufgabenplanung verborgen sind.

Aufgaben sind ein verbreiteter Mechanismus für Persistenz und Remoteausführung. Der Registry-Cache ist außerdem wertvoll,
wenn ein Angreifer die XML-Datei der Aufgabe löscht oder manipuliert.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Der TaskCache hat die
Aufgabe/Aktion konfiguriert oder bewahrt. Um eine bestimmte Ausführung zu beweisen, sind Laufzeitfelder und unabhängige Protokolle erforderlich.

## Speicherorte { #locations }

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

Sichern Sie `SOFTWARE` mit den Transaktionsprotokollen und den vollständigen Verzeichnisbaum `System32\Tasks`. Die Ereignisprotokolle
Task Scheduler Operational und Security liefern zusätzliche Belege zu Erstellung, Aktualisierung und Ausführung.

## Hinweise für die Analyse { #analyst-cautions }

- Registry-Cache und Aufgaben-XML können nach Manipulation, Löschung, unvollständiger Registrierung oder Beschädigung voneinander abweichen.
- `Hidden` von Juntei kennzeichnet ein strukturelles Verbergen (fehlende Tree-Verknüpfung/Sicherheitsbeschreibung unter
  den relevanten Bedingungen), nicht einfach eine Aufgabe, die mit dem normalen UI-Flag „Hidden“ konfiguriert ist.
- Windows 7 speichert Aufgabenaktionen in XML statt im Wert `Actions` des Caches; eine leere Aktion ist
  dort nicht automatisch verdächtig.
- `LastRun` und `LastResult` sind nützlich, aber kein vollständiger, manipulationssicherer Ausführungsverlauf.
- `Context` benennt den konfigurierten Prinzipal/Kontext, nicht den Benutzer, der die Aufgabe registriert hat.
- CLSIDs von COM-Handlern erfordern eine separate Auflösung über Registry/Dateien.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.Hidden == true or (.Executable? | test("powershell|cmd|mshta"; "i")))'
```

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `SourcePath` | Host-Kontext und Quell-Hive `SOFTWARE`. |
| `TaskName` | Vollständiger Pfad in der Aufgabenplanung. Wichtigster Ansatzpunkt für XML und Ereignisprotokolle. |
| `TaskGUID` | GUID, die Einträge der Caches Tree und Tasks verknüpft. Starker Korrelationsschlüssel. |
| `TaskIndex` | Von der Aufgabenplanung verwendete Tree-/Indexmetadaten; Kontext, kein Ausführungszähler. |
| `TaskLists` | Cache-Listen (`Boot`, `Logon`, `Plain`, `Maintenance`), die Auslösung/Verarbeitung kategorisieren. |
| `Hidden` | Struktureller Indikator für eine verborgene Aufgabe. Vor dem Schluss auf Manipulation `InTree`, `InTasks` und den Zustand der Sicherheitsbeschreibung prüfen. |
| `InTree`, `InTasks` | Ob beide Seiten des Caches existieren. Eine fehlende Verknüpfung kann auf Löschung/Manipulation/unvollständige Registrierung hinweisen. |
| `HasSecurityDescriptor` | Ob der Tree-Eintrag einen `SD` hat; sein Fehlen kann bei relevanten Versionen zum Verhalten verborgener Aufgaben beitragen. |
| `TreeKeyLastWrittenUTC`, `TaskKeyLastWrittenUTC` | Änderungen auf Schlüsselebene für jede Seite des Caches; keine exakten Erstellungs-/Ausführungszeiten. |
| `ActionIndex` | Nullbasierte Aktion innerhalb der Aufgabe. Eine Aufgabe kann mehrere Zeilen erzeugen. |
| `ActionType` | `exec`, `com_handler`, `send_email` oder `show_message`. |
| `Executable`, `Args`, `WorkingDirectory` | Effektive Ausführungsaktion. Anführungszeichen, Variablen, relative Pfade und LOLBins auswerten. |
| `COM-ClassID`, `ActionData` | CLSID und Daten des COM-Handlers oder veraltete E-Mail-/Nachrichten-Nutzdaten. CLSID gesondert auflösen. |
| `Context` | Mit den Aktionen kodierter Prinzipalkontext (`LocalSystem`, `Author` usw.); konfigurierte Identität, nicht die registrierende Person. |
| `Author`, `URI`, `Description` | Aufgabenmetadaten. Leicht vom Angreifer kontrollierbar und kein Vertrauensnachweis. |
| `LastRegisteredUTC` | Registrierungs-/Aktualisierungszeit aus DynamicInfo, sofern vorhanden. |
| `LastRunUTC`, `LastSuccessfulRunUTC`, `LastResult` | Zwischengespeicherte Laufzeit-/Ergebnismetadaten. Mit Betriebsprotokollen und Prozessbelegen korrelieren. |
| `ActionsRaw` | Ursprüngliche Aktionsbytes, wenn die Dekodierung unvollständig ist oder eine Validierung erforderlich ist. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Abweichung von Pfad/GUID, falscher Werttyp, abgeschnittene Aktionsstruktur oder fehlende Teile. |

## Untersuchungsstrategie { #investigation-strategy }

1. Prüfen Sie zuerst verborgene/inkonsistente Aufgaben und Pfade, die nicht von Microsoft stammen.
2. Rekonstruieren Sie die effektiven Befehle, einschließlich Argumenten, Arbeitsverzeichnis, Erweiterung von Umgebungsvariablen und COM-Auflösung.
3. Vergleichen Sie den Registry-Datensatz mit der Aufgaben-XML und ermitteln Sie Hash/Zeitstempel der referenzierten ausführbaren Datei.
4. Korrelieren Sie `TaskName`/GUID/Zeit mit Ereignissen der Aufgabenplanung, Security 4698/4702, Prozessprotokollen, Prefetch und MFT/USN.
5. Durchsuchen Sie gelöschte/wiederhergestellte Registry-Daten, wenn ein Angreifer die Aufgabe entfernt hat.

## Weitere Informationen { #further-details }

- [Microsoft: Schema der Aufgabenplanung](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft: Ereignisprotokoll der Aufgabenplanung](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 Scheduled Task](https://attack.mitre.org/techniques/T1053/005/)
