# Benutzerzugriffsprotokollierung (UAL) { #user-access-logging-ual }

## Was UAL ist { #what-ual-is }

Die Benutzerzugriffsprotokollierung (User Access Logging, UAL) ist eine Funktion von Windows Server, die seit
Windows Server 2012 standardmäßig installiert und aktiviert ist. Sie fasst die Anforderungen, die Clients an die
Rollen des Servers richten, in ESE-Datenbanken zusammen. Für jede Kombination aus Rolle, Clientadresse und Konto
speichert sie, wann der Client im Jahr zum ersten und zum letzten Mal gesehen wurde, die Gesamtzahl der
Anforderungen und eine Anzahl für jeden Tag des Jahres. Zu den Rollen gehören File Server (SMB), Active Directory
Domain Services, Active Directory Certificate Services, DHCP Server, Print and Document Services, Remote Access,
Web Server (IIS) und Windows Server Update Services.

UAL ist wertvoll für Untersuchungen lateraler Bewegung: Die Client-Zeilen zeigen, welche Adressen und Konten einen
Server an welchen Tagen genutzt haben, oft noch lange nachdem die Ereignisprotokolle überschrieben wurden.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Ein Client mit der aufgezeichneten Adresse,
authentifiziert als das aufgezeichnete Konto, hat die Rolle an jedem Tag genutzt, für den eine Anzahl vorliegt.

## Speicherort und Sicherung { #location-and-collection }

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

`Current.mdb` enthält die jüngste Aktivität. `SystemIdentity.mdb` beschreibt den Server, benennt die Rollen und
listet für jedes Jahr die `{GUID}.mdb`-Datei auf, in der es gespeichert ist (`CHAINED_DATABASES`). Der UAL-Dienst hält die
Datenbanken geöffnet: Sichern Sie das gesamte Verzeichnis `Sum` mit einem forensischen Sicherungswerkzeug oder aus
einem Volume-Snapshot.

!!! warning "„Dirty“-Datenbanken"
    `Current.mdb` ist oft „dirty“, wenn sie von einem laufenden Server gesichert wird. Juntei liest sie so, wie sie
    gesichert wurde, und meldet dies, spielt aber `Svc*.log` nicht ein: Aktivität, die nur in den
    Transaktionsprotokollen existiert, fehlt.

## Hinweise für die Analyse { #analyst-cautions }

- UAL zählt Anforderungen pro Tag; es protokolliert nicht jede einzelne Anforderung. Die einzigen Zeitpunkte einer
  Client-Zeile sind `InsertDate` und `LastAccess`.
- Die Tage von `DailyAccesses` sind UTC-Tage.
- `Address` ist die Adresse, die der Server gesehen hat: Ein NAT-Gateway, ein Proxy oder ein VPN kann zwischen ihm
  und dem tatsächlichen Client stehen. `::1` und `127.0.0.1` sind der Server selbst.
- Parsen Sie jede Datenbank des Verzeichnisses `Sum`: Derselbe Client kann Zeilen in `Current.mdb` und in der
  `{GUID}.mdb` seines Jahres haben.
- Fehlende Zeilen beweisen nicht, dass sich nichts verbunden hat: UAL kann abgeschaltet werden (`Disable-Ual` oder
  durch Beenden des User Access Logging Service), und einer „dirty“ Datenbank fehlen ihre neuesten Änderungen.
- Ohne `SystemIdentity.mdb` stammt `RoleName` aus den Namen, die Windows Server seinen Rollen gibt.
- Ein `LastAccess` mit dem Wert null oder einem ungültigen Zeitstempel bestimmt das Jahr nicht. Juntei verwendet dann einen gültigen `InsertDate`, danach die Metadaten der Jahresdatenbank; ohne gültiges Jahr bleibt `DayOfYear` erhalten.
- Die Summen der Zusammenfassung addieren alle Quellzeilen. `Current.mdb`, Jahresdatenbanken und Snapshots können sich überschneiden; dies sind keine deduplizierten Verbindungs- oder Anmeldezahlen.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-ese ual <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Current.mdb, {GUID}.mdb and SystemIdentity.mdb
  -f, --file <FILE>      Parse one UAL database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese ual -d Sum -o ual-csv
juntei parse-ese ual -d Sum -t jsonl \
  | jq 'select(.Table == "CLIENTS") | {Address, AuthenticatedUserName, RoleName, LastAccess}'
```

## Ausgabetabellen { #output-tables }

Die CSV-Ausgabe erzeugt für jede Tabelle eine Datei; ist die Tabelle leer, enthält die Datei nur die Kopfzeile. JSON-
und JSONL-Zeilen enthalten `SourceFile` (die Datenbank) und `Table` (den Namen der Tabelle in der Datenbank).

| CSV-Datei | Tabelle | Inhalt |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | Die Clients jeder Rolle: Adresse, Konto, erster und letzter Zugriff des Jahres, Gesamtzahl und tägliche Anzahl der Anforderungen. |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | Wann jede Rolle zum ersten und zum letzten Mal genutzt wurde. |
| `UAL-DNS.csv` | `DNS` | Paare aus Adresse und Hostname, jeweils mit dem Zeitpunkt, zu dem sie zuletzt gesehen wurden. |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | Virtuelle Hyper-V-Maschinen: Bezeichner, Seriennummer, Erstellungszeit und Zeitpunkt der letzten Aktivität. |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | Der Server: Name, Domäne, Windows-Version, Produkttyp, Hardware, Zeitzone und letzter Start; eine neue Zeile kann eine Umbenennung, einen Domänenbeitritt oder eine Heraufstufung zum Domänencontroller zeigen. |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | Die `{GUID}.mdb`-Datei jedes Jahres. |
| `UAL-Roles.csv` | `ROLE_IDS` | Rollen-GUIDs mit ihren Namen. |
| `UAL-<table>.csv` | Eine andere Tabelle | Jede Spalte einer Tabelle, die Juntei nicht kennt. |

## Felder { #fields }

| Feld | Interpretation |
|---|---|
| `SourceFile` | Die Datenbank, aus der die Zeile gelesen wurde. |
| `RoleGUID`, `RoleName` | Die Rolle und ihr Name aus der `SystemIdentity.mdb` neben der Datenbank (ohne diese die integrierten Namen). |
| `TenantID` | Die GUID des Mandanten des Clients, sofern die Rolle eine meldet. |
| `TotalAccesses` | Anforderungen des Clients im Jahr der Datenbank; die Summe von `DailyAccesses`. |
| `InsertDate` | Wann die Zeile erstellt wurde: die erste Anforderung des Clients im Jahr in dieser Datenbank (UTC). |
| `LastAccess` | Die letzte Anforderung des Clients (UTC). |
| `Address` | Die IPv4- oder IPv6-Adresse des Clients. |
| `AuthenticatedUserName` | Das Konto, als das sich der Client authentifiziert hat (`DOMAIN\user`; Computerkonten enden auf `$`). |
| `ClientName` | Der Name des Clients, sofern die Rolle einen meldet. |
| `DailyAccesses` | Die Tage mit Anforderungen und ihre Anzahl aus den Spalten `Day1` … `Day366`, datiert im Jahr von `LastAccess`: `2023-01-12: 5, 2023-01-13: 2` in CSV, Objekte mit `Date` und `Accesses` in JSON. Ein Tag, dessen Jahr unbekannt ist, hat `DayOfYear` statt `Date`. |
| `FirstSeen`, `LastSeen` | Erste und letzte Nutzung einer Rolle (`ROLE_ACCESS`), letzte Sichtung eines DNS-Paares (`DNS`), in UTC. |
| `CreationTime`, `LastSeenActive` | Wann eine virtuelle Maschine oder eine `SYSTEM_IDENTITY`-Zeile erstellt wurde und wann eine virtuelle Maschine zuletzt aktiv war, in UTC. |
| `OS-ProductType`, `OS-ProductTypeName` | 1 `Workstation`, 2 `Domain Controller`, 3 `Server`. |
| `OS-CurrentTimeZone` | Die Abweichung des Servers von UTC in Minuten (`-420` ist UTC-07:00). |
| `OS-LastBootUpTime` | Der letzte Start des Servers in UTC (der WMI-Text so, wie er gespeichert ist, wenn er nicht gelesen werden kann). |
| Weitere Spalten | Jede weitere Spalte der Tabelle unter den PascalCase-Feldbezeichnungen von Juntei, zum Beispiel `SystemDNS-HostName`, `SystemDomainName`, `OS-BuildNumber` und `SystemSerialNumber`. |

## Automatisches Parsen { #automatic-parsing }

`auto-parse` schreibt die Tabellen nach `LateralMovement` als `UAL-Clients.csv`, `UAL-RoleAccess.csv` usw. (eine
einzige Datei `UAL.json` bzw. `UAL.jsonl` mit `-t json` bzw. `-t jsonl`), zusammen mit
`LateralMovement/UAL-logs.txt`. Die kombinierte Zeitleiste `-j` enthält jede Zeile, mit `Comp` und dem
`Artifact`-Bezeichner `ual`. `-Z` konvertiert `InsertDate`, `LastAccess`, `FirstSeen`, `LastSeen`,
`CreationTime`, `LastSeenActive` und `OS-LastBootUpTime`.

## Weitere Informationen { #further-details }

- [Microsoft — User Access Logging](https://learn.microsoft.com/en-us/windows-server/administration/user-access-logging/get-started-with-user-access-logging)
- [Plaso — User Access Logging parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/user_access_logging.py)
- [Velociraptor — Windows.Forensics.UserAccessLogs](https://docs.velociraptor.app/artifact_references/pages/windows.forensics.useraccesslogs/)
- [CrowdStrike — How to Leverage User Access Logging for Forensic Investigations](https://www.crowdstrike.com/en-us/blog/user-access-logging-ual-overview/)
