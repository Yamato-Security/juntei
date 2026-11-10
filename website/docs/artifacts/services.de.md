# Windows-Dienste { #windows-services }

## Was die Dienstkonfiguration aufzeichnet { #what-service-configuration-records }

Windows-Dienste und Kerneltreiber werden unter jedem Control Set von `SYSTEM` konfiguriert. Die Datensätze benennen
die ausführbare Datei/den Treiber oder die Dienst-DLL, Startmodus, Diensttyp, Konto, Ladereihenfolgegruppe, verzögerten
Start, Fehlerverhalten und die Aktionen, die Windows nach einem Fehler ausführt.

Angreifer erstellen oder ändern Dienste für Persistenz, Rechteausweitung, das Laden von Treibern und
laterale Bewegung. Legitime Software verwendet genau denselben Mechanismus; daher kommt es auf Pfade, Konten, Signaturen,
Erstellungskontext und Laufzeitbelege an.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Das geparste Control Set hat den
Dienst/Treiber wie dargestellt konfiguriert. Die Konfiguration allein beweist nicht, dass der Dienst erfolgreich gestartet wurde.

## Speicherort { #location }

```text
SYSTEM\ControlSet00N\Services\<ServiceName>
SYSTEM\Select\Current   (which control set Windows used)
```

Sichern Sie `SYSTEM` mit seinen Transaktionsprotokollen. Ereignis-IDs wie System 7045 und Security 4697 können
die Installation aufzeichnen; Dienststeuerungs- und Prozessereignisse können einen tatsächlichen Start bzw. eine Ausführung belegen.

## Hinweise für die Analyse { #analyst-cautions }

- `ImagePath` kann Umgebungsvariablen, Anführungszeichen, Argumente, Gerätepfade oder einen nicht in Anführungszeichen gesetzten
  Pfad mit Leerzeichen enthalten. Bewahren Sie ihn exakt, bevor Sie ihn normalisieren.
- `ServiceDll` ist für Dienste entscheidend, die in `svchost.exe` gehostet werden; `ImagePath` allein benennt möglicherweise nur den Host.
- `StartType=automatic` beweist nicht, dass der Dienst gestartet wurde oder aktiviert blieb.
- `ObjectName` ist das konfigurierte Konto, nicht notwendigerweise der Benutzer, der den Dienst installiert hat.
- Treiber und Benutzermodusdienste verwenden unterschiedliche `ServiceType`-Flags und Pfadkonventionen.
- Fehleraktionen können nach einem Fehler einen Befehl unter dem Dienstkonto ausführen; dies ist eine Persistenz-
  oberfläche, die leicht übersehen wird.
- Andere Control Sets können eine frühere Konfiguration bewahren; verwenden Sie `IsCurrentControlSet`.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --service <NAME>    Only this service name
      --all-control-sets  Parse every ControlSet00N

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --all-control-sets -t jsonl \
  | jq 'select(.FailureCommand != null or (.ImagePath? | test("temp|public"; "i")))'
```

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `SourcePath` | Systemidentität und Eingabe-Hive. Über Sicherungen hinweg beibehalten. |
| `ControlSet`, `IsCurrentControlSet` | Quell-Control-Set und ob `Select\Current` es benennt. Für Aussagen über den aktiven Zustand das aktuelle bevorzugen. |
| `ServiceName` | Registry-Schlüssel/interner Dienstname. Starker Ansatzpunkt für Ereignisse und Befehle des Dienststeuerungs-Managers. |
| `DisplayName`, `Description` | Für Benutzer sichtbarer Text; kann aus Ressourcenverweisen bestehen oder vom Angreifer kontrolliert sein. |
| `ImagePath` | Befehl der ausführbaren Datei/des Treibers wie konfiguriert. Anführungszeichen, Argumente, Umgebungsvariablen und Gerätepfade auswerten. |
| `ServiceDLL` | Von einem Host-Dienst geladene DLL. Bei Diensten mit gemeinsam genutztem Prozess oft die eigentliche Payload. |
| `StartType`, `StartValue` | Lesbarer/roher Startmodus: Boot (0), System (1), automatisch (2), manuell (3), deaktiviert (4). |
| `DelayedAutostart` | Automatischer Dienst startet nach der anfänglichen Bootphase; dennoch zur Persistenz geeignet. |
| `ServiceType`, `TypeValue` | Dekodierte/rohe Flags: Kerneltreiber, Dateisystemtreiber, eigener/gemeinsamer Prozess, Benutzerdienst usw. |
| `ErrorControl` | Stufe der Fehlerbehandlung bei Boot/Start. Vorwiegend betrieblicher Kontext. |
| `ObjectName` | Konfiguriertes Dienstkonto oder Name des Treiberobjekts. Nicht die Identität des Installierenden. |
| `Group` | Ladereihenfolgegruppe, besonders wichtig für Boot-/Systemtreiber. |
| `FailureResetPeriod` | Sekunden, bis der Fehlerzähler zurückgesetzt wird; `4294967295` bedeutet nie. |
| `FailureCommand` | Für eine Fehleraktion `run_command` konfigurierter Befehl. Hochwertiges Persistenzfeld. |
| `FailureActions` | Vollständige dekodierte Aktionsliste (`restart`, `reboot`, `run_command`, `none`) und Verzögerungen. |
| `FailureActionType`, `FailureActionDelayMs` | Erste Aktion, zum Filtern abgeflacht; prüfen Sie auch die vollständige Liste. |
| `FailureActionsRaw` | Ursprüngliche Bytes, wenn die Dekodierung unvollständig/beschädigt ist. |
| `KeyLastWrittenUTC` | Letzte Änderung am Dienstschlüssel, nicht notwendigerweise Installations-/Startzeit. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiedergabe. |
| `Problems` | Typkonflikt, fehlerhafte Fehleraktionsstruktur oder fehlende/unlesbare Felder. |

## Untersuchungsstrategie { #investigation-strategy }

1. Prüfen Sie nicht standardmäßige Pfade, beschreibbare Verzeichnisse, ungewöhnliche Konten, neue Schlüsselzeiten und Fehlerbefehle.
2. Expandieren Sie Umgebungsvariablen anhand der Konfiguration des betroffenen Hosts, nicht des Analyserechners.
3. Prüfen Sie Hash/Signatur der tatsächlichen ausführbaren Datei oder DLL, sofern gesichert.
4. Korrelieren Sie Dienstname/-pfad mit den Ereignissen 7045/4697, Prozesserstellung, Prefetch, `$MFT` und USN.
5. Vergleichen Sie aktuelle und nicht aktuelle Control Sets auf geänderte/gelöschte Persistenz.

## Weitere Informationen { #further-details }

- [Microsoft: Registry-Struktur für Geräte- und Treiberdienste](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: SERVICE_STATUS / Diensttypen und -zustände](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_status)
- [Microsoft: ChangeServiceConfig2-Fehleraktionen](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
