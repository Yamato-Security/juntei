# Microsoft Defender-Quarantäne { #microsoft-defender-quarantine }

## Was die Quarantäne enthält { #what-the-quarantine-contains }

Die Quarantäne von Microsoft Defender bewahrt Metadaten zu einer Erkennung sowie verschlüsselte Inhalte der
Ressourcen auf, die sie unter Quarantäne gestellt hat. Juntei verknüpft Datensätze aus Entries, ResourceData und Resources, um
die Bedrohungsbezeichnung, die Quarantänezeit, ursprüngliche logische und physische Pfade, Dateimetadaten, den Besitzer,
den Download-Ursprung aus `Zone.Identifier`, Hashes und Integritätsprobleme zu melden.

Dieses Artefakt kann die Ereignisprotokolle überdauern, die Erkennung/Bereinigung ursprünglich aufgezeichnet haben. Es kann
außerdem die exakten verdächtigen Bytes für ein isoliertes Reverse Engineering bewahren.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Defender hat den Quarantänedatensatz für
die benannte Bedrohung/Ressource zum aufgezeichneten Zeitpunkt erstellt. Die Produktbezeichnung ist kein unabhängiger Beweis für Malware,
und die Quarantäne allein beweist keine vorherige Ausführung.

## Speicherort und Sicherung { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Quarantine\
  Entries\{GUID}
  ResourceData\<prefix>\<SHA-1>
  Resources\<prefix>\<SHA-1>
```

Sichern Sie das gesamte Verzeichnis `Quarantine`. Ein Eintrag ohne passende Inhalts-/Datensatzdateien liefert weiterhin
nützliche Metadaten, kann aber weder die Dateibytes noch berechnete Hashes liefern.

## Sicherheit und Hinweise für die Analyse { #safety-and-analyst-cautions }

- Extrahierte Inhalte sind wahrscheinlich bösartig. Verwenden Sie ein isoliertes Analysesystem und führen Sie sie niemals aus.
- `ThreatName` ist die Signatur/Klassifizierung von Defender zum damaligen Zeitpunkt. Fehlalarme und spätere
  Neuklassifizierungen sind möglich.
- Die Quarantänezeit ist die Zeit der Bereinigung/Quarantäne – nicht notwendigerweise die des Downloads, der Erstellung oder der Ausführung.
- Eine unter Quarantäne gestellte Datei kann vor der Ausführung oder nach ihrer Ausführung erkannt worden sein.
- Ursprünglicher Pfad und Zone.Identifier sind wertvoll, können aber fehlen oder vom Angreifer kontrolliert sein.
- Verwaiste Datensätze können durch Aufbewahrungsregeln/Bereinigung, unvollständige Sicherung oder Beschädigung entstehen.
- Juntei prüft Größen, CRCs, IDs und Hashes; prüfen Sie `Problems`, bevor Sie wiederhergestellten Bytes vertrauen.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find quarantine folders
  -f, --file <FILE>      Parse one Entries\{GUID} file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
      --extract <DIR>       Extract dangerous content with a .quarantined extension
  -C, --clobber             Overwrite confirmed output targets
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -t jsonl \
  | jq '{QuarantineTime, ThreatName, Path, SHA256: .Content.SHA256, Problems}'

# Only in an isolated malware-analysis environment
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract extracted/
```

Zum Beispiel wird `tool.exe` zu `tool.exe.quarantined`. Zusätzliche Streams enthalten ihren Typ im
Namen, etwa `tool.exe.Zone.Identifier.quarantined`. Juntei entfernt außerdem die Ausführungsberechtigung.
Die Erweiterung und die Berechtigungen verringern das Risiko einer versehentlichen Ausführung, doch die wiederhergestellten Bytes müssen dennoch
als Malware behandelt werden.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Record` | `resource`, Eintrag ohne Ressourcen, `orphan-content` oder `orphan-record`. Verwaiste Datensätze zeigen unvollständige Verknüpfung/Sicherung. |
| `QuarantineTime` | Quarantäne-/Bereinigungszeit von Defender in UTC, keine Ausführungszeit. |
| `ThreatName`, `ThreatID` | Klassifizierung und numerische ID von Defender. Als Herstellerurteil/Kontext behandeln. |
| `ResourceType` | Datei, Registry-Schlüssel, Dienst, Autostart-Element, geplante Aufgabe oder eine andere Ressourcenkategorie. |
| `Path`, `PhysicalPath` | Ursprünglicher logischer/physischer Speicherort. Wichtigster Ansatzpunkt für die Korrelation; exakte Schreibweise beibehalten. |
| `FileSize` | In den Ressourcenmetadaten aufgezeichnete Größe. Mit `ContentSize` und der MFT vergleichen. |
| `CreationTime`, `LastAccessTime`, `LastWriteTime` | Für die unter Quarantäne gestellte Ressource erfasste Metadaten; getrennt von der Quarantänezeit und anfällig für Timestomping. |
| `Attr` | Aufgezeichnete Dateiattribute. Der Status „versteckt“/„System“ kann nützlicher Kontext sein. |
| `Owner` | SID aus der wiederhergestellten Sicherheitsbeschreibung; der Besitzer ist nicht notwendigerweise der ausführende Benutzer. |
| `ZoneID`, `ReferrerURL`, `HostURL` | Download-Zone und URL-Werte aus `Zone.Identifier`. Starke Hinweise auf die Herkunft, sofern vorhanden; der ADS kann fehlen/entfernt worden sein. |
| `ContentSize` | Tatsächliche Größe des wiederhergestellten primären Inhalts. Mit dem aufgezeichneten `FileSize` vergleichen. |
| `MD5`, `SHA1`, `SHA256` | Aus den wiederhergestellten Bytes berechnet. SHA-256 zur modernen Identifizierung von Inhalten verwenden; alle für die Interoperabilität beibehalten. |
| `Streams` | Arten/Namen/Größen der wiederhergestellten Streams; können ADS, Sicherheitsdaten oder einen Registry-Hive umfassen. |
| `ResourceID` | Kennung, die Ressourcenmetadaten und Inhaltsdatensätze verknüpft. |
| `EntryID` | GUID des Quarantäneeintrags; verknüpft auch mit Defender-Verlauf und Ereignisbelegen. |
| `ScanID` | Defender-Scankennung zur Gruppierung von Ressourcen/Aktionen. |
| `QuarantineFolder` | Quell-Stammordner/Unterscheidung des Hosts. |
| `EntryFile`, `ContentFile`, `ResourceRecordFile` | Exakte Quelldateien, aus denen die Zeile erstellt wurde. Unverzichtbar für die Validierung. |
| `ExtractedFile` | Durch `--extract` geschriebene primäre Quarantänedatei; leer, wenn keine primäre Extraktion erfolgte. Zusätzliche Streams werden daneben geschrieben. `auto-parse -x` gibt sie relativ zu seinem Ausgabeverzeichnis an (`./HOST1/SecurityProducts/DefenderQuarantined/…`). |
| `Problems` | Integritätsabweichung, fehlender Inhalt, ID-Widerspruch, Beschädigung oder Kontext zu verwaisten Datensätzen. Nicht unterdrücken. |

JSON/JSONL bewahrt die strukturierte Ressource, die Streamliste, die Datensatzeinträge und alle Beziehungsdetails,
statt des zusammengefügten Texts der CSV.

## Extraktionsverhalten { #extraction-behavior }

Juntei verwendet den ursprünglichen Dateinamen mit einer abschließenden Erweiterung `.quarantined`, wenn der Quarantäneeintrag
einen aufzeichnet. Für verwaiste oder Nicht-Datei-Ressourcen verwendet es die Inhaltskennung und fügt die
Kennung hinzu, um Namenskollisionen aufzulösen. Unter Windows ungültige Zeichen in Dateinamen werden ersetzt. Sicherheits-
und alternative Streams werden neben die primäre Datei geschrieben. Für jeden Sicherheitsstream wird zusätzlich ein lesbarer
Textbericht erstellt. Juntei entfernt die Ausführungsberechtigung;
vorhandene abweichende Inhalte werden ohne `-C` nicht überschrieben. Diese Schutzmaßnahmen verringern Unfälle, machen
die Bytes aber nicht sicher.

### Die wiederhergestellte Sicherheitsbeschreibung lesen { #reading-the-recovered-security-descriptor }

Eine `.security.quarantined`-Datei enthält binäre Windows-Sicherheitsmetadaten und ist daher in einem
Texteditor nicht lesbar. Sie ist ein separater Sicherungsstream, keine weitere Kopie der ausführbaren Datei. Juntei
dekodiert sie bei der Extraktion automatisch, sowohl mit `parse-file defender-quarantine --extract` als auch mit `auto-parse -x`:

```text
tool.exe.quarantined              Original file content; treat as malware
tool.exe.security.quarantined     Original binary security descriptor, preserved unchanged
tool.exe.security_descriptor.txt  Readable ownership and permission report
```

Der Name der Textdatei folgt dem tatsächlichen Namen der Binärdatei, einschließlich einer etwaigen Kollisionskennung. Der erzeugte
Text ist gewöhnliches UTF-8 und unterliegt denselben Überschreibschutzmaßnahmen wie andere extrahierte Dateien. Er enthält
den Namen der Binärdatei, die Byteanzahl und SHA-256, damit der Bericht seiner Quelle zugeordnet werden kann.

| Berichtsfeld | Interpretation |
|---|---|
| Besitzer und primäre Gruppe | In der Sicherheitsbeschreibung aufgezeichnete SIDs. Bekannte SIDs erhalten Namen wie `BUILTIN\Administrators` oder `NT AUTHORITY\SYSTEM`. Andere SIDs bleiben unaufgelöst; eine vertraute RID allein reicht nicht aus, um ein lokales Konto oder Domänenkonto zu benennen. Der Besitz beweist nicht, wer eine Datei erstellt oder ausgeführt hat. |
| Steuerflags | Ursprüngliche numerische Flags und deren Namen, einschließlich ACL-Vorhandensein, Vererbung und Schutz. |
| DACL | Geordnete Zugriffssteuerungseinträge (ACEs): Typ Zulassen/Verweigern, Konto-SID, Berechtigungsmaske, Rechte und Vererbungsflags. Dies sind aufgezeichnete Einträge, keine Berechnung des effektiven Zugriffs für das Token eines Benutzers. |
| SACL | Aufgezeichnete Überwachungs- oder Mandatory-Integrity-Einträge. Flags für erfolgreiche/fehlgeschlagene Überwachung beschreiben die Konfiguration, keinen Zugriffsverlauf. Eine fehlende SACL belegt nicht, dass jegliche Überwachung deaktiviert war. |
| Objekt-GUIDs | Optionale GUIDs für objektspezifische ACEs; objektspezifische Rechte werden numerisch beibehalten, statt fälschlich als Dateirechte bezeichnet zu werden. |
| SDDL | Darstellung von Besitzer, Gruppe und unterstützten ACL-Einträgen in der Security Descriptor Definition Language. Die Steuerflags gesondert berücksichtigen. Juntei gibt keine unvollständige SDDL-Zeichenfolge aus, wenn eine Komponente beschädigt oder nicht unterstützt ist. |
| Dekodierstatus und Probleme | `complete` oder `incomplete`, mit Begründungen. Nicht unterstützte ACEs behalten ihr rohes Hex im Bericht. Die ursprüngliche Binärdatei wird bei erfolgreicher Extraktion immer beibehalten. Dekodierprobleme erscheinen auch im Feld `Problems` des Quarantänedatensatzes. |

Eine fehlende DACL, eine NULL-DACL und eine leere DACL werden getrennt gemeldet. Eine fehlende oder NULL-DACL bietet
keine diskretionäre Zugriffsbeschränkung; eine leere DACL enthält keine ACEs, die Zugriff gewähren. Andere Windows-Zugriffsprüfungen
und Privilegien sind dennoch relevant. Geerbte und reine Vererbungs-ACEs (inherit-only) bleiben ausdrücklich gekennzeichnet.

Ein Eintrag, der Administratoren geerbten Vollzugriff gewährt, kann beispielsweise so erscheinen:

```text
Entry 1: ALLOW (type 0x00)
  Flags: 0x10 (INHERITED_ACE)
  Account: S-1-5-32-544 (BUILTIN\Administrators)
  Access mask: 0x001F01FF
  Rights: Full control (FILE_ALL_ACCESS)
```

Der Decoder läuft offline unter Windows, macOS und Linux, ohne Windows-APIs, Python oder Kontosuche.
Er dekodiert einfache Allow-/Deny-/Audit-/Alarm-ACEs, Objekt-ACEs und Mandatory-Integrity-Labels. Bedingte,
Callback-, Ressourcenattribut- und andere nicht unterstützte ACEs werden ausdrücklich gekennzeichnet und in
Rohform bewahrt; ihre Zugriffsbedingungen werden nicht ausgewertet. Eine beschädigte Sicherheitsbeschreibung verhindert nicht die Extraktion
der anderen Streams.

## Untersuchungsstrategie { #investigation-strategy }

1. Beginnen Sie mit `ThreatName`, `Path`, `QuarantineTime`, `EntryId` und den Hashes.
2. Prüfen Sie `Problems` und vergleichen Sie die aufgezeichnete Größe mit dem wiederhergestellten Inhalt.
3. Pivotieren Sie mit dem SHA-256 in Daten anderer Hosts, EDR, Sandbox und Threat Intelligence.
4. Korrelieren Sie ursprünglichen Pfad und Zeit mit `$MFT`, USN-Journal, Prefetch, Amcache und Ereignisprotokollen.
5. Nutzen Sie URL-/Zonenfelder, um den Erstzugriff zu untersuchen, validieren Sie dies aber mit Browser-, Proxy- und DNS-Belegen.

## Weitere Informationen { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [Defender-Plugin von dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [Microsoft: Bereinigung und Quarantäne in Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [defender-dump](https://github.com/knez/defender-dump)
- [Microsoft: Sicherheitsbeschreibungen](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft: Zeichenfolgenformat für Sicherheitsbeschreibungen](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: ACE-Zeichenfolgen](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings)
