# SRUM { #srum }

## Was SRUM ist { #what-srum-is }

Der System Resource Usage Monitor (SRUM) speichert eine ESE-Datenbank zur Ressourcennutzung von Anwendungen und Diensten.
Je nach Windows-Version zeichnet er Netzwerk-Bytes, Schnittstellen-/Profilbezeichner,
Verbindungen, CPU-Zeit, Vordergrund-/Hintergrundaktivität, Datenträgeraktivität, Energieverbrauch, Push-Benachrichtigungen
und Daten der Anwendungszeitleiste auf. Einträge werden über `SruDbIdMapTable` mit Anwendungs-IDs und Benutzer-SIDs
verknüpft.

SRUM ist besonders nützlich, wenn eine ausführbare Datei gelöscht wurde: Ihre Anwendungs-ID, ihr Benutzer und ihre
Zeilen zur Ressourcennutzung können die Datei überdauern. Netzwerksummen können eine Exfiltrationshypothese stützen, aber
SRUM ist kein Paketmitschnitt und identifiziert in der Regel nicht jeden entfernten Endpunkt oder jede übertragene Datei.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Windows hat die aufgezeichnete Ressourcennutzung
während des Messzeitraums der Tabelle der Anwendung/dem Konto zugeordnet.

## Speicherort und Sicherung { #location-and-collection }

```text
C:\Windows\System32\sru\SRUDB.dat
C:\Windows\System32\config\SOFTWARE   (context for account and Wi-Fi names)
```

SRUM gibt es ab Windows 8. Kopieren Sie `SRUDB.dat` konsistent, idealerweise aus einem Snapshot oder mit
einem forensischen Sicherungswerkzeug. Sichern Sie außerdem `SOFTWARE` sowie die ESE-Transaktions-/Protokolldateien im Verzeichnis `sru`.

!!! warning "„Dirty“-Datenbanken"
    Juntei liest eine „dirty“ `SRUDB.dat` so, wie sie gesichert wurde, und meldet dies. Derzeit spielt es
    `SRU*.log` nicht ein, sodass Änderungen, die nur in diesen Protokollen existieren, fehlen. Beschreiben Sie eine
    „dirty“ Datenbank nicht stillschweigend als vollständig.

## Hinweise für die Analyse { #analyst-cautions }

- Viele Zeilen stellen eine periodische Aggregation dar, häufig an Stundengrenzen – nicht den exakten Zeitpunkt einer Aktion.
- Bytesummen identifizieren für sich genommen weder Inhalt, Absicht, Protokoll noch Ziel.
- Eine Anwendungsidentität kann ein Pfad, ein Dienst, eine paketierte App oder ein Objekt sein; prüfen Sie `AppType`.
- Benutzer- und WLAN-Namen sind Anreicherungen aus dem passenden `SOFTWARE`-Hive. Fehlende Namen bedeuten nicht, dass
  die zugrunde liegende ID fehlt.
- `null`, Null, fehlende Spalten, nicht unterstützte Typen und beschädigte Werte sind unterschiedliche Zustände.
- Die Schemata der Provider unterscheiden sich je nach Windows-Version. Bewahren Sie unbekannte Tabellen und ursprüngliche Spaltennamen auf.
- Windows-Hintergrunddienste erzeugen legitimerweise hohe Ressourcensummen.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-ese srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SRUDB.dat
  -f, --file <FILE>      Parse one SRUDB.dat
  -r, --software <FILE>  Matching SOFTWARE hive for names

Output:
  -o, --output <PATH>        Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>   csv, json, or jsonl
      --include-value-state  Add per-value read state and raw bytes to JSON/JSONL
```

```bash
juntei parse-ese srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-ese srum -f SRUDB.dat -t jsonl \
  | jq 'select(.Provider == "NetworkUsages" and (.BytesSent // 0) > 100000000)'
juntei parse-ese srum -f SRUDB.dat -t jsonl --include-value-state \
  | jq '{Provider, Timestamp, ValueState, Raw}'
```

## Ausgabetabellen { #output-tables }

Die CSV-Ausgabe erzeugt `SRUM-IdentityMap.csv` für `SruDbIdMapTable` und eine Tabelle pro Provider. Zu den bekannten Ausgabenamen gehören:

| Provider/Tabelle | Zweck für die Untersuchung |
|---|---|
| `NetworkUsages` | Gesendete/empfangene Bytes pro Anwendung, Benutzer, Schnittstelle und Profil. |
| `NetworkConnections` | Aktivität in Form von Verbindungsanzahl/-dauer sowie Startzeiten von Verbindungen. |
| `AppResourceUseInfo` | CPU-, Datenträger- und Netzwerknutzung im Vorder- und Hintergrund sowie zugehörige Anwendungsnutzung. |
| `AppTimelineProvider` | Aktivität der Anwendungszeitleiste mit Feldern für Endzeit und Nutzungsstatus. |
| `EnergyUsage` | Kurz- und langfristige Energie-/Provider-Messungen; `IsLt` unterscheidet sie. |
| `PushNotifications` | Aktivität von Push-Benachrichtigungen. |
| `VfuProvider` | Der Provider `vfuprov`; sein genauer Zweck ist nicht gut dokumentiert, interpretieren Sie seine Felder daher vorsichtig. |
| `EnergyEstimation`, `TaggedEnergy` | Versionsspezifische Energie- und Nutzungsprovider. |
| `ProcessorTime`, `SizeInBytes`, `TotalUsed`, `NetworkBytes` | Provider-Tabellen von Windows Server. |
| Unbekannter Tabellenname | Juntei bewahrt unbekannte Provider unter ihrem GUID-/Tabellennamen auf und gibt jede Katalogspalte aus. |

Da SRUM katalogbasiert ist und Windows Provider-Spalten hinzufügt, gibt es bewusst keine feste,
abschließende Liste providerspezifischer Felder. Juntei gibt **jede in der Datenbank vorhandene Spalte mit
Junteis Feldbezeichnungen in PascalCase** aus. Die folgenden Felder bilden den stabilen/gemeinsamen Ausgabevertrag.

## Kategorisierte Ausgabe von auto-parse { #categorized-auto-parse-output }

`auto-parse` legt jede SRUM-Tabelle direkt in ihrer Untersuchungskategorie ab. CSV, JSON und JSONL
verwenden dasselbe Dateinamenpräfix `SRUM-`; es gibt kein umschließendes Verzeichnis `SRUM/`.

| Kategorie | CSV-Dateien |
|---|---|
| `NetworkActivity` | `SRUM-NetworkConnections.csv`, `SRUM-NetworkUsages.csv`, `SRUM-NetworkBytes.csv`, `SRUM-PushNotifications.csv` |
| `ProcessExecution` | `SRUM-AppResourceUseInfo.csv`, `SRUM-AppTimelineProvider.csv`, `SRUM-ProcessorTime.csv` |
| `SystemInformation` | `SRUM-EnergyUsage.csv`, `SRUM-EnergyEstimation.csv`, `SRUM-TaggedEnergy.csv`, `SRUM-SizeInBytes.csv`, `SRUM-TotalUsed.csv`, `SRUM-IdentityMap.csv`, `SRUM-VfuProvider.csv` sowie Tabellen unbekannter Provider |

Es werden nur Tabellen exportiert, die in der Sicherung vorhanden sind. Netzwerkzähler und Benachrichtigungsdatensätze werden
nach ihrem hauptsächlichen Untersuchungszweck eingeordnet; damit wird nicht behauptet, dass jede Zeile eine entfernte
Verbindung beschreibt. Die Identitätszuordnung unterstützt mehrere Kategorien. VFU und unbekannte Provider verbleiben in
`SystemInformation`, da ihr Vorhandensein allein keine Ausführung belegt.

`SystemInformation/SRUM-logs.txt` enthält die Parser-Diagnosen und die Zuordnung von ursprünglicher zu endgültiger Ausgabe.
Beide Manifeste listen die tatsächlichen Tabellenpfade in `OutputFiles` auf; `Output` der SRUM-Aufgabe
ist das Computerverzeichnis, und `Category` bezeichnet den gemeinsamen Speicherort für Protokolle/Metadaten.

Bei mehreren Datenbanken erhalten die Dateinamen vor der Erweiterung `-Source1`, `-Source2` usw.
Das Protokoll bewahrt die ursprünglichen Aliase der Sicherung, und JSON/JSONL-Zeilen behalten ihre Herkunftsangaben `SourceFile`,
`Table`, `Provider`, `Comp` und `Artifact`. Keine Tabelle wird in mehrere
Kategorien kopiert. Die kombinierte SIEM-Zeitleiste `-j` enthält jeden beibehaltenen SRUM-Datensatz genau einmal.

Das eigenständige `parse-ese srum` behält den angeforderten Ausgabeort bei. Seine CSV-Tabellendateien tragen ebenfalls
das Präfix `SRUM-`; explizit angegebene JSON/JSONL-Ausgabedateinamen bleiben die Wahl der Analysten.

## Gemeinsame Ausgabefelder { #common-output-fields }

| Feld | Interpretation |
|---|---|
| `SourceFile` | Eingabedatenbank; in JSON/JSONL vorhanden und hostübergreifend nützlich. |
| `Table` | Ursprünglicher ESE-Tabellenname/GUID. Beim Vergleich von Versionen oder unbekannten Providern aufbewahren. |
| `Provider` | Lesbarer Provider-/Ausgabename oder bei unbekannten Providern der ursprüngliche Tabellenname. |
| `ID` | `AutoIncId` der Zeile; ein Zeilenbezeichner der Datenbank, keine Prozess-ID. |
| `Timestamp` | Wann SRUM die Werte aufgezeichnet/aggregiert hat, auf Millisekunden gerundet. In der Regel nicht der exakte Ereigniszeitpunkt. |
| `EXE-Info` | Aus `AppID` aufgelöste Identität der Anwendung/des Dienstes/des Pakets. Wichtigster Ankerpunkt für Programme. |
| `EXE-InfoDescription` | Aus zusammengesetzten SRUM-Anwendungsnamen extrahierte Beschreibung. Nur Kontext. |
| `EXE-Timestamp` | In einer zusammengesetzten Anwendungsidentität eingebetteter Zeitpunkt, sofern vorhanden; nicht der Nutzungszeitpunkt der Zeile. |
| `AppType` | `NormalApp`, `Service`, `ModernApp` oder `Object`; bestimmt, wie `EXE-Info` zu interpretieren ist. |
| `SID` | Aus `UserID` aufgelöste Konto-SID. Starker Ankerpunkt für Benutzer-/Dienstidentität. |
| `User` | Aus `SOFTWARE` angereicherter lesbarer Profilname; kann null sein, während `SID` gültig bleibt. |
| `UserID`, `AppID` | Numerische Rohverweise in `SruDbIdMapTable`; für nicht aufgelöste oder doppelte Namen aufbewahren. |
| `IsLt` | `true` für die langfristige EnergyUsage-Tabelle, `false` für die kurzfristige. |
| `InterfaceLUID` | Rohe LUID der Netzwerkschnittstelle aus Provider-Zeilen. |
| `InterfaceType` | Dekodierte Schnittstellenklasse (zum Beispiel `IF_TYPE_IEEE80211`); eine Klasse, kein eindeutiger Adapter. |
| `L2ProfileID` | Numerischer Verweis auf ein Netzwerkprofil. |
| `ProfileName` | Aus `SOFTWARE` angereicherter WLAN-/Profilname; potenziell sensibel und kein Beweis für eine aktuelle Verbindung. |
| `ConnectStartTime`, `EventTimestamp`, `EndTime`, `StartTime` | Bekannte FILETIME-Spalten von Providern, nach UTC dekodiert. Ihre genaue Bedeutung hängt vom Provider ab und ist kein universeller Ausführungszeitpunkt. |
| Providerspezifische Spalten | Exportierte Provider-Spalten wie `BytesSent`, `BytesReceived` (die Datenbankspalte `BytesRecvd`), CPU-/Datenträgerzähler, Dauern, Anzahlen, Flags oder Schätzungen. Interpretieren Sie Einheiten gemäß dem Provider und vergleichen Sie Zeilen innerhalb desselben Schemas. |

## Diagnosefelder (`--include-value-state`) { #diagnostic-fields-include-value-state }

| Feld | Interpretation |
|---|---|
| `ValueState` | Zuordnung jedes Datenbankfelds zu `parsed`, `null`, `absent`, `unsupported` oder `damaged`. Damit wird das Fehlen von Beweisen von einem Parserfehler unterschieden. |
| `Raw` | Ursprüngliche Bytes in hexadezimaler Form für Werte, die Bytes besitzen. Zur Validierung und für künftige Dekodierung verwenden. |

`SRUM-IdentityMap.csv` (aus `SruDbIdMapTable`) enthält `IdIndex`, `IdType` und `IdBlob`: den Rohindex, die Identitätskategorie
und den dekodierten Programm-/SID-/Objekttext, auf den die Provider-Zeilen verweisen.

## Untersuchungsstrategie { #investigation-strategy }

1. Identifizieren Sie das Programm mit `EXE-Info`, `AppType` und `AppID`; verwechseln Sie `Id` nicht mit der PID.
2. Gruppieren Sie Netzwerkzeilen nach Programm, Benutzer, Profil, Schnittstelle und Zeitintervall.
3. Vergleichen Sie gesendete/empfangene Bytes mit normalen Baselines und anderen Hosts.
4. Korrelieren Sie verdächtige Intervalle mit Prefetch, Ereignisprotokollen, DNS-/Proxy-/Firewall-Daten und Dateizugriffen.
5. Melden Sie den „dirty“-Status der Datenbank und das fehlende Einspielen der Transaktionsprotokolle als Einschränkung der Sicherung.

## Weitere Informationen { #further-details }

- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [SrumECmd von Eric Zimmerman](https://github.com/EricZimmerman/Srum)
- [dissect.esedb](https://github.com/fox-it/dissect.esedb)
- [libyal libesedb](https://github.com/libyal/libesedb)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
