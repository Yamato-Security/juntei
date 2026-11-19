# Microsoft Defender-Erkennungsverlauf { #microsoft-defender-detection-history }

## Was der Verlauf aufzeichnet { #what-the-history-records }

Microsoft Defender schreibt für jede Erkennung eine Erkennungsverlaufsdatei. Jede Datei hält Folgendes fest:

- Kennung, Name, Schweregrad und Kategorie der Bedrohung;
- was Defender unternommen hat und ob es erfolgreich war;
- ob die Bedrohung gerade ausgeführt wurde;
- den beteiligten Benutzer und Prozess und wodurch die Bedrohung erkannt wurde;
- den Quarantäneeintrag, sofern vorhanden;
- jede Ressource, in der die Bedrohung gefunden wurde: Dateien, Archivmitglieder, Downloads, Prozesse, Registry-Werte,
  Dienste, Befehlszeilen und Verhaltensweisen.

Defender schreibt diese Dateien auch für Bedrohungen, die es nie unter Quarantäne gestellt hat: solche, die es nur erkannt hat,
solche, die zugelassen wurden, und solche, deren Entfernung fehlschlug. Der Schutzverlauf der Windows-Sicherheit und die
WMI-Klasse `MSFT_MpThreatDetection` (`Get-MpThreatDetection`) zeigen diese Datensätze an. Als Orange
Cyberdefense Werte in einer Datei änderte, meldete WMI die geänderten Werte.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Defender hat eine Erkennung der benannten
Bedrohung in den aufgeführten Ressourcen aufgezeichnet, mit dem aufgezeichneten Status und den aufgezeichneten Zeiten. Der Bedrohungsname ist das
Urteil des Produkts. Ein Erkennungsdatensatz allein zeigt weder, dass die Bedrohung ausgeführt wurde, noch, dass sie
vollständig entfernt wurde.

## Speicherort und Sicherung { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

Jede Datei ist nach ihrer `DetectionID` benannt. Sichern Sie den gesamten Ordner `DetectionHistory`. Bei Verzeichniseingabe
und mit `auto-parse` wird jede Datei gelesen, die sich in beliebiger Tiefe in einem `DetectionHistory`-Ordner befindet und wie
eine Erkennungsverlaufsdatei beginnt. Verwenden Sie `-f` für eine Kopie, die außerhalb eines solchen Ordners aufbewahrt wird. Defender entfernt
Elemente nach einer konfigurierbaren Anzahl von Tagen aus seinem Scanverlauf (`ScanPurgeItemsAfterDelay` in
`Set-MpPreference`); ältere Erkennungen können daher fehlen.

Die Quarantäne ist ein separates Artefakt: siehe [Defender-Quarantäne](defender-quarantine.md). Die
`QuarantineID` einer Erkennung verwendet dieselbe Schreibweise wie die `EntryID` der Quarantäneausgabe (Großbuchstaben,
ohne geschweifte Klammern), sodass sich beide verknüpfen lassen.

## Hinweise für die Analyse { #analyst-cautions }

- Erkennung, Bereinigungsversuch und erfolgreiche Entfernung sind getrennte Zustände. Lesen Sie
  `ThreatStatus` (`Detected`, `Quarantined`, `Removed`, `Allowed`, `RemoveFailed`, …) zusammen mit
  `ThreatStatusErrorCode`, das 0 ist, wenn die letzte Statusänderung erfolgreich war.
- `ExecutionStatus` ist die Sicht von Defender zum Erkennungszeitpunkt (`Blocked`, `Allowed`, `Executing`,
  `NotExecuting`, `Unknown`). Es beweist weder, dass die Bedrohung ausgeführt wurde, noch, dass sie nie ausgeführt wurde.
  Gleichen Sie dies mit Prefetch, Amcache, Ereignisprotokollen und Prozesstelemetrie ab.
- `ThreatName` ist der Signaturname zum damaligen Zeitpunkt. Fehlalarme und spätere Neuklassifizierungen sind
  möglich. Testdateien wie EICAR werden wie echte Bedrohungen gemeldet.
- `User` ist der von Defender aufgezeichnete Wert `DomainUser` (`UserSource` ist `DomainUser`). Er kann
  leer sein, ein Dienstkonto sein oder ein anderer Benutzer als derjenige, der die Datei erstellt hat.
- Defender kann für eine Bedrohung mehrere Dateien schreiben, und Juntei behält jede davon bei. Zwei Dateien können
  sogar dieselbe `DetectionID` enthalten, wie es bei einer kopierten oder wiederhergestellten Datei der Fall ist. Gruppieren Sie nach `DetectionID` und
  `SourceFile` gemeinsam.
- Werte mit unbekannter Bedeutung werden beibehalten, nicht interpretiert: Ihr Abschnitt und ihre Position stehen in
  `UnknownValues`.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

CSV ist das Standarddateiformat. Mit `-t csv`, `-t json` oder `-t jsonl` ohne `-o` werden Datensätze auf
stdout ausgegeben. Ohne beide Optionen erscheint nur eine Zusammenfassung. Mit `-C` überschreiben Sie eine vorhandene
Ausgabe. Die Ausgabe wird niemals über eine der Eingabedateien geschrieben, auch nicht mit `-C`.

Juntei schreibt je Ressource einer Erkennung einen Datensatz. Eine Erkennung ohne Ressourcen erhält einen
Datensatz mit leeren Ressourcenfeldern. Die Datensätze einer Erkennung teilen deren `DetectionID` und sind über
`ResourceIndex` in der gespeicherten Reihenfolge nummeriert.

Beschädigungen brechen den Lauf nicht ab. Bei einer abgeschnittenen Datei bleiben die vor der Beschädigung gelesenen Werte erhalten, und
`Problems` gibt an, was fehlt. Eine Datei in einem `DetectionHistory`-Ordner, die keine
Erkennungsverlaufsdatei ist, wird gemeldet und übersprungen. Die übrigen Dateien werden weiterhin gelesen.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Record` | `resource` oder `detection` für eine Erkennung ohne Ressourcen. |
| `InitialDetectionTime` | Zeitpunkt, zu dem Defender die Bedrohung zuerst erkannt hat (UTC). Leer, wenn die Datei 0 enthält; ein Wert, der keine Zeitangabe ist, wird in `UnknownValues` beibehalten. |
| `LastThreatStatusChangeTime` | Zeitpunkt der letzten Statusänderung der Bedrohung (UTC). |
| `RemediationTime` | Zeitpunkt, zu dem Defender die Bedrohung bereinigt hat (UTC). Dies ist die Zeit eines Versuchs, kein Beweis für die Entfernung; prüfen Sie `ThreatStatus`. |
| `ThreatName`, `ThreatID` | Signaturname und numerische Kennung von Defender (`Get-MpThreatCatalog`). |
| `Severity`, `SeverityID` | `Low`, `Moderate`, `High`, `Severe` oder `Unknown`, mit der gespeicherten Zahl. |
| `Category`, `CategoryID` | Bedrohungskategorie wie `TROJAN`, `TOOL` oder `BEHAVIOR`, mit der gespeicherten Zahl. |
| `ThreatStatus`, `ThreatStatusID` | Status der Bedrohung laut Defender. Eine Zahl ohne dokumentierten Namen wird mit leerem `ThreatStatus` beibehalten. |
| `ThreatStatusErrorCode` | Fehlercode der letzten Statusänderung; 0 bedeutet, dass sie erfolgreich war. |
| `ExecutionStatus`, `ExecutionStatusID` | Ob Defender die Bedrohung bei ihrer Erkennung als laufend gesehen hat. |
| `DetectionSource`, `DetectionSourceTypeID` | Wodurch sie erkannt wurde: `Real-time`, `IOAV` (Downloads und Anhänge), `User`, `System`, `AMSI` und weitere. |
| `User`, `UserSource` | Der von Defender aufgezeichnete Wert `DomainUser` und seine Grundlage (`DomainUser`). Leer, wenn die Datei keinen Benutzer nennt. |
| `Process` | Der beteiligte Prozess (`ProcessName`); kann `Unknown` sein. |
| `ResourceIndex`, `ResourceCount` | Position dieser Ressource und Anzahl der Ressourcen in der Erkennung. |
| `ResourceType` | `file`, `containerfile`, `webfile`, `process`, `regkeyvalue`, `service`, `CmdLine`, `behavior`, `amsi` und weitere, wie Defender sie benennt. |
| `Path` | Die Ressource: ein Pfad, ein Archivmitglied (`archive.zip->member`), ein Registry-Wert, eine Befehlszeile oder ein Prozess. Ein `webfile` nennt außerdem seine URL und den herunterladenden Prozess (`pid:…,ProcessStart:…`). Exakte Schreibweise beibehalten. |
| `ResourceFlags` | Ressourcenflags in Hexadezimalschreibweise; ihre Bedeutung ist nicht dokumentiert. |
| `TrackingStartTime` | `ThreatTrackingStartTime` der Ressource, eine Ganzzahl, die als FILETIME (UTC) gelesen wird, so wie plaso eine Erkennung datiert. |
| `SHA256`, `SHA1`, `MD5`, `TrackingSize` | Hashes und Größe aus den Tracking-Daten der Ressource. Sie beschreiben die Datei zum Zeitpunkt des Defender-Scans. |
| `TrackingID` | `ThreatTrackingId` der Ressource. |
| `OtherTracking` | Alle übrigen Tracking-Werte als `Key=Value` in gespeicherter Reihenfolge, etwa `ThreatTrackingScanSource` oder `ThreatTrackingBMContextRich` bei Verhaltenserkennungen. |
| `QuarantineID` | Der Quarantäneeintrag (`Quarantine\Entries\{GUID}`); leer, wenn die Datei eine Null-GUID enthält. |
| `DetectionID` | GUID der Erkennung; zugleich der Name der Datei. Eine Abweichung wird in `Problems` gemeldet. |
| `MagicVersion` | `Magic.Version:1.2` aus dem Header; leer bei Dateien, die ohne diesen Wert geschrieben wurden, etwa einigen aus Windows Server 2016. |
| `UnknownValues` | Werte mit unbekannter Bedeutung als `position=value` (zum Beispiel `6.13=NT AUTHORITY\SYSTEM`); die gezählte Liste nach dem Status erscheint als `2.list`. |
| `SourceFile` | Die gelesene Datei, relativ zum Eingabeverzeichnis, oder bei `-f` ihr Name. |
| `Problems` | Beschädigungen, fehlende Werte, unerwartete Typen oder ein nicht passender Dateiname. Nicht unterdrücken. |

JSON und JSONL haben dieselben Felder mit typisierten Werten: Zahlen bleiben Zahlen, fehlende Werte sind
`null`. Statt `OtherTracking` enthalten sie `Tracking`, jeden Tracking-Wert mit dem Schlüssel von Defender,
seinem Typ und seinem Wert. `UnknownValues` wird zu einer Liste von Positionen mit Typen. `CountedList`
enthält die gezählte Liste. Eine gespeicherte FILETIME behält ihre rohe Tick-Anzahl neben der Zeit, als die sie gelesen wird.

## Verhalten in der Zeitleiste { #timeline-behavior }

`auto-parse` speichert die Datensätze als `SecurityProducts/DefenderHistory.csv` (bzw. `.json`/`.jsonl`) und
fügt sie der kombinierten Zeitleiste `-j` hinzu. `super-timeline` erzeugt Ereignisse aus
`InitialDetectionTime`, `LastThreatStatusChangeTime` und `RemediationTime` im Datensatz jeder Ressource,
mit Status, Fehlercode, Ausführungsstatus und Erkennungsquelle in den Details des Ereignisses.
`TrackingStartTime` ist als Ganzzahl gespeichert und wird daher nur mit
`--include-interpreted-times` zu einem Ereignis. Die eigenen Zeiten der Datei werden nie verwendet. Ein Datensatz ohne verwendbare Zeit
bleibt in der geparsten Ausgabe und wird als Datensatz ohne Zeitangabe gezählt.

## Untersuchungsstrategie { #investigation-strategy }

1. Beginnen Sie mit `ThreatName`, `Path`, `SHA256`, `InitialDetectionTime` und `ThreatStatus`.
2. Prüfen Sie `ThreatStatusErrorCode` und `Problems`, bevor Sie schließen, dass eine Bedrohung entfernt wurde.
3. Verknüpfen Sie `QuarantineID` mit der `EntryID` der Quarantäne, um die unter Quarantäne gestellten Bytes wiederherzustellen.
4. Pivotieren Sie mit Hashes und Pfaden in `$MFT`, USN-Journal, Prefetch, Amcache, Browserverlauf und Ereignisprotokolle
   (das Betriebsprotokoll von Defender: Ereignis-IDs 1116, 1117, 1118 und 1119).
5. Nutzen Sie bei `webfile`-Ressourcen die URL und den Prozessstart, um den Erstzugriff zu untersuchen, und
   validieren Sie diese mit Browser-, Proxy- und DNS-Belegen.

## Getestete Versionen { #tested-versions }

Die Tests verwenden öffentliche Dateien, deren Ergebnisse andere Werkzeuge unabhängig gelesen haben:

- Das Windows-10-Beispiel von Orange Cyberdefense (Defender 4.18.24090.11), abgeglichen mit dem, was WMI
  gemeldet hat.
- Die zwei Windows-Testdateien von plaso, abgeglichen mit dem Parser `windefender_history` von plaso.
- 14 Dateien aus der APT-Simulator-VM des DFIRArtifactMuseum.
- Eine Kopie, die ohne die Version im Header geschrieben wurde, wie manche Dateien aus Windows Server 2016.
- Abgeschnittene, leere oder andersartige Dateien.

## Weitere Informationen { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [wddh-parser von Orange Cyberdefense](https://github.com/cert-orangecyberdefense/wddh-parser)
- [Defender-Verlaufsparser von plaso](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Velociraptor Windows.Applications.DefenderHistory](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft: Klasse MSFT_MpThreatDetection](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft: Ereignis-IDs von Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
