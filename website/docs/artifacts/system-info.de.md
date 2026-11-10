# Systeminformationen { #system-information }

## Was es ist { #what-it-is }

Die vier Maschinen-Hives eines Systems enthalten das, was Analysten wissen müssen, bevor sie seine Zeitleiste lesen:

- seine Namen (NetBIOS, DNS-Hostname, DNS-Domäne) und die Domäne oder Arbeitsgruppe, der es angehört;
- seine Windows-Version, den Build und die Update-Revision sowie den Zeitpunkt, zu dem Windows installiert oder zuletzt per Upgrade aktualisiert wurde;
- seine Zeitzone, aus der hervorgeht, welche lokalen Zeiten das System geführt hat;
- wann es zuletzt ordnungsgemäß heruntergefahren wurde;
- seine Prozessorarchitektur und Codepages;
- den Benutzer, der sich zuletzt angemeldet hat, und die Profile aller Benutzer, die sich angemeldet haben;
- seine SIDs: die des Computers (lokale Konten bestehen aus dieser plus einer RID) und die seiner Domäne;
- die Windows-Versionen, von denen aus ein Upgrade durchgeführt wurde.

`parse-registry system-info` schreibt pro System einen Datensatz mit all diesen Angaben.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Die Konfiguration, wie sie zum Zeitpunkt der Sicherung der Hives
(oder ihres letzten Schreibvorgangs) bestand. Die Schlüsselzeitstempel geben an, wann sich Teile davon zuletzt geändert haben.

## Speicherort { #location }

```text
C:\Windows\System32\config\SYSTEM     (the control set Windows used last: Select\Current)
  ControlSet00N\Control\ComputerName\ComputerName     ComputerName
  ControlSet00N\Services\Tcpip\Parameters             Hostname, Domain
  ControlSet00N\Control\TimeZoneInformation           TimeZoneKeyName, Bias, ActiveTimeBias, …
  ControlSet00N\Control\Windows                       ShutdownTime
  ControlSet00N\Control\Session Manager\Environment   PROCESSOR_ARCHITECTURE
  ControlSet00N\Control\Nls\CodePage                  ACP, OEMCP
  Setup\Source OS (Updated on M/D/YYYY HH:MM:SS)      the version before each upgrade

C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion                 ProductName, EditionID, DisplayVersion,
                                                      CurrentBuild, UBR, InstallDate, InstallTime, …
  Microsoft\Windows NT\CurrentVersion\ProfileList     one key per profile (SID)
  Microsoft\Windows\CurrentVersion\Authentication\LogonUI   LastLoggedOnUser, …

C:\Windows\System32\config\SAM
  SAM\Domains\Account                                 V: the computer's SID at its end

C:\Windows\System32\config\SECURITY
  Policy\PolAcDmN, PolAcDmS                           the computer's name and SID
  Policy\PolPrDmN, PolPrDmS                           its domain (or workgroup) and the domain's SID
  Policy\PolDnDDN, PolDnTrN, PolDnDmG                 the domain's and forest's DNS names, GUID
  Policy\PolMachineAccountS                           the computer account's SID in the domain
```

Sichern Sie die vier Hives mit ihren Transaktionsprotokollen. Jeder von ihnen liefert auch allein die Angaben, die er enthält.

## Hinweise für die Analyse { #analyst-cautions }

- **Registry-Zeitstempel sind UTC; die Zeitzone gilt für alles andere.** Schlüsselzeitstempel, FILETIMEs und die meisten
  Ereigniszeitstempel sind UTC. Textprotokolle, manche Anwendungsprotokolle sowie Datumsangaben in Namen oder Zeichenfolgen
  (etwa `Source OS (Updated on …)`) können lokal sein. `Bias` ist die Anzahl Minuten von der lokalen Standardzeit
  zu UTC (`UTC = local + Bias`). `UtcOffset` und `DaylightUtcOffset` geben die beiden Offsets an,
  und `TimeZoneDescription` gibt an, wann die Sommerzeit beginnt und endet.
- **Die Zeitzone kann sich ändern.** `TimeZoneLastWrittenUTC` ist der letzte Schreibzugriff auf den Schlüssel. Windows
  schreibt den Schlüssel auch selbstständig neu, wenn es `ActiveTimeBias` (den zu diesem Zeitpunkt geltenden Offset) aktualisiert; ein
  jüngerer Zeitpunkt beweist daher nicht, dass jemand die Zeitzone geändert hat. Vergleichen Sie mit den Ereignisprotokollen: Security 4616
  (Systemzeit geändert) und den `Microsoft-Windows-Kernel-General`-Ereignissen im System-Protokoll.
- **`StandardStart` und `DaylightStart` sind `TIME_FIELDS` des Kernels, keine `SYSTEMTIME`s.** Der
  Wochentag ist ihr letztes Feld, nicht ihr drittes (wie im `TZI`-Wert der Zone in `SOFTWARE`). In
  US-Zeitzonen ergeben beide Layouts dasselbe, weil der Wochentag Sonntag (0) ist. Juntei dekodiert sie
  als `TIME_FIELDS`: Ein öffentlich verfügbares System von der Osterinsel (erster Samstag um 22:00 Uhr) zeigt den
  Unterschied. Ein Monat 0 bedeutet keine Sommerzeit.
- **Windows 11 meldet sich als „Windows 10“.** `ProductName` enthält unter Windows 11 weiterhin `Windows 10 …`. Builds ab 22000
  sind Windows 11, und die Zusammenfassung weist dies aus. `CurrentVersion` ist unter Windows 8.1, 10
  und 11 `6.3`; `Version` verwendet `CurrentMajorVersionNumber` (10), sofern vorhanden.
- **`InstallDate` verschiebt sich mit Funktionsupgrades.** Ein direktes Upgrade auf ein neues Release (z. B. von 1809 auf
  21H2) setzt es zurück. Das Release, das das jeweilige Upgrade ersetzt hat, steht mit eigenem `InstallDate`
  in `PreviousVersions` (`Setup\Source OS`). Ihr frühestes `InstallDate` kommt der
  ursprünglichen Installation am nächsten. Das Datum im Namen eines `Source OS`-Schlüssels gibt keine Zeitzone an.
- **`ShutdownTime` ist das letzte ordnungsgemäße Herunterfahren.** Ein Absturz oder Stromausfall aktualisiert es nicht.
  Vergleichen Sie es mit den Ereignisprotokollen (6006, 6008, 1074) und den Zeitpunkten der letzten Schreibzugriffe auf die Hives.
- **`LastLoggedOnUser` ist der letzte Benutzer des Anmeldebildschirms,** nicht jede Anmeldung. Remotedesktop,
  `runas`, Dienste und geplante Aufgaben setzen ihn nicht alle. `LogonUiLastWrittenUTC` gibt ungefähr an, wann
  er zuletzt gesetzt wurde.
- **Profile:** `ProfileList` führt jeden Benutzer (und Dienst) auf, der auf dem System ein Profil hatte,
  einschließlich gelöschter Konten, bis ihr Profil entfernt wird. Ab Windows 10 schreibt Windows
  `LocalProfileLoadTime` und `LocalProfileUnloadTime`: ungefähr die letzte Anmeldung und Abmeldung.
- **SIDs:** Lokale Konten sind `MachineSid-<RID>`, Domänenkonten `DomainSid-<RID>`.
  `MachineSid` stammt aus dem `SAM` (andernfalls aus `PolAcDmS` in `SECURITY`). Die beiden sollten übereinstimmen;
  ist dies nicht der Fall, meldet `Problems` dies, wie es bei Hives unterschiedlicher Systeme zu erwarten wäre. `DomainSid` ist
  in einer Arbeitsgruppe leer.
- **Ein Verzeichnis, ein System.** Mit `-d` gehören die Hives in einem Verzeichnis zu einem System.
  `RegBack`-Kopien und ein zweiter Hive derselben Art (`SYSTEM.old`) bilden eigene Datensätze.
  Mit `-f` gehören die angegebenen Hives zu einem System, und die übrigen werden daneben gesucht.
- **Steuerungssätze:** Nur der Steuerungssatz, den Windows zuletzt verwendet hat, wird gelesen. `parse-registry read` zeigt
  die übrigen.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM, SOFTWARE, SAM and SECURITY hives
  -f, --file <FILE>      A hive of one system; repeat for each
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry system-info -d collection/ -o system-info.csv
juntei parse-registry system-info -f SYSTEM -f SOFTWARE -t jsonl \
  | jq '{Host, TimeZoneDescription, ProductName, Build, LastShutdown}'
```

Die Zusammenfassung gibt für jedes System an: seine Windows-Version, den Installationszeitpunkt, seine Zeitzone in
Worten, den beim letzten Schreiben des Schlüssels geltenden Offset, das letzte Herunterfahren, seine Domäne oder
Arbeitsgruppe, den letzten angemeldeten Benutzer, seine SID, die Anzahl seiner Profile und die Versionen, von denen aus ein Upgrade
erfolgte. `auto-parse` gibt außerdem die Zeitzone jedes Systems aus, wenn es die gefundenen Systeme
auflistet.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `SourcePath`, `SoftwarePath`, `SamPath`, `SecurityPath` | Host und die gelesenen Hives (`SourcePath` ist der `SYSTEM`-Hive, andernfalls der erste gelesene). |
| `CompName`, `Hostname`, `Domain` | Der NetBIOS-Name (`SYSTEM`, andernfalls `PolAcDmN` aus `SECURITY`), der DNS-Hostname und die DNS-Domäne (`Tcpip\Parameters`). |
| `PrimaryDomain`, `DNS-Domain` | Die Domäne, der der Computer beigetreten ist, oder seine Arbeitsgruppe (`PolPrDmN`), und der DNS-Name der Domäne (`PolDnDDN`). |
| `ProductName`, `Edition`, `DisplayVersion`, `Version`, `Build`, `Ubr`, `ServicePack`, `InstallationType` | `CurrentVersion`: der Produktname wie geschrieben, `EditionID`, `DisplayVersion` (andernfalls `ReleaseId`), Haupt- und Nebenversion (andernfalls `CurrentVersion`), der Build, die Update Build Revision, `CSDVersion` sowie `Client`, `Server` oder `Server Core`. |
| `InstallDateUTC`, `InstallTimeUTC` | `InstallDate` (auf die Sekunde) und `InstallTime` (auf 100 ns, ab Windows 10): Installation oder letztes Funktionsupgrade. |
| `RegisteredOwner`, `RegisteredOrganization`, `SystemRoot` | Wie geschrieben. |
| `TimeZone`, `TimeZoneDescription` | `TimeZoneKeyName` (andernfalls `StandardName`) sowie die Zeitzone in Worten mit ihren Offsets und Sommerzeitdaten. |
| `UTC-Offset`, `DaylightUTC-Offset` | Der Offset der lokalen Standardzeit gegenüber UTC und der der Sommerzeit (leer ohne Sommerzeit). |
| `BiasMinutes`, `ActiveBiasMinutes`, `StandardBiasMinutes`, `DaylightBiasMinutes` | `Bias`, `ActiveTimeBias` (zum Zeitpunkt des Schreibens des Schlüssels geltend), `StandardBias`, `DaylightBias`. |
| `StandardName`, `DaylightName` | Wie geschrieben, oft ein Ressourcenverweis (`@tzres.dll,-112`). |
| `StandardStart`, `DaylightStart` | Wann Standard- und Sommerzeit beginnen, in Worten (`first Sunday of November, 02:00`). |
| `DynamicDaylightTimeDisabled`, `RealTimeIsUniversal` | Dynamische Sommerzeit deaktiviert; die Hardwareuhr läuft in UTC. |
| `TimeZoneLastWrittenUTC` | Der letzte Schreibzugriff auf den Zeitzonenschlüssel. |
| `LastShutdownUTC` | `ShutdownTime`: das letzte ordnungsgemäße Herunterfahren. |
| `CurrentControlSet`, `ProcessorArchitecture`, `AnsiCodePage`, `OemCodePage` | Der gelesene Steuerungssatz, `PROCESSOR_ARCHITECTURE`, `ACP` und `OEMCP`. |
| `LastLoggedOnUser`, `LastLoggedOnSamUser`, `LastLoggedOnUserSID`, `LastLoggedOnDisplayName`, `LogonUiLastWrittenUTC` | Der letzte Benutzer des Anmeldebildschirms und der letzte Schreibzugriff auf seinen Schlüssel. |
| `MachineSID`, `DomainSID`, `MachineAccountSID` | Die SID des Computers, die SID seiner Domäne (`PolPrDmS`) und die SID des Computerkontos in der Domäne (`PolMachineAccountS`). |
| `CompNameLastWrittenUTC`, `PrimaryDomainLastWrittenUTC`, `VersionLastWrittenUTC` | Ungefähr, wann der Computer zuletzt benannt wurde, zuletzt einer Domäne oder Arbeitsgruppe beigetreten ist und zuletzt aktualisiert wurde (Schlüssel von `CurrentVersion`). |
| `PreviousVersions` | Jeder `Source OS`-Schlüssel: die Version, ihr Installationsdatum und das Datum im Namen des Schlüssels (JSON: jeweils ein Objekt). |
| `Profiles` | Jedes Profil: SID, Ordner, Schlüsselzeitstempel, letztes Laden und Entladen (JSON: jeweils ein Objekt). |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung über alle vier Hives. |
| `Problems` | Werte mit falschem Typ oder falscher Größe (ihre Bytes bleiben erhalten) sowie `SAM` und `SECURITY` mit abweichenden Computer-SIDs. |

## Untersuchungsstrategie { #investigation-strategy }

1. Lesen Sie zuerst die Zeitzone: Notieren Sie `UtcOffset`, die Sommerzeitdaten und ob die Zeitzone
   kürzlich geändert wurde. Rechnen Sie Protokolle in lokaler Zeit damit um.
2. Notieren Sie Version, Build und Installationsdatum. Prüfen Sie `PreviousVersions` auf Upgrades und vergleichen Sie
   das Installationsdatum mit den ältesten Zeitstempeln im Dateisystem und in den Ereignisprotokollen.
3. Notieren Sie `LastShutdownUTC`: Aktivität danach ohne neuen Herunterfahrzeitpunkt deutet auf einen Absturz, einen Stromausfall
   oder eine Live-Sicherung hin.
4. Verwenden Sie `MachineSid` und `DomainSid`, um in SIDs an anderen Stellen (BAM,
   `ProfileList`, Ereignisprotokolle) lokale von Domänenkonten zu unterscheiden, und die Lade- und Entladezeiten der Profile für die letzten Anmeldungen.
5. Korrelieren Sie Computer- und Domänennamen mit den Ereignisprotokollen sowie mit `CompNameLastWrittenUTC`
   und `PrimaryDomainLastWrittenUTC`, um Umbenennungen und Domänenbeitritte zu erkennen.

## Weitere Informationen { #further-details }

- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft: DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft: TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft: Sicherheits-IDs](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb: Zeitzonen](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb: Windows-Produktinformationen](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
