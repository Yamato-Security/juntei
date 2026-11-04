# Persistence Hunter { #persistence-hunter }

Persistenz ermöglicht es einem Programm, nach einem Neustart, einer Anmeldung, einem geplanten Auslöser oder einem anderen
Ereignis erneut ausgeführt zu werden. Ein Angreifer kann dafür die normale Windows-Konfiguration nutzen. Viele legitime Programme
verwenden genau dieselben Mechanismen; ein gefundener Persistenzeintrag ist daher ein Ausgangspunkt für die Analyse.

`juntei persistence-hunter` ist eine native Rust-Portierung von
[PyrsistenceSniper von Hexastrike](https://github.com/hexastrike/PyrsistenceSniper) mit 131 Prüfungen: dem
Katalog von PyrsistenceSniper mit 130 Prüfungen plus einer nativen Prüfung für Startmenü-COM.

Eine Erläuterung jedes Mechanismus, seiner Beweisorte und der Untersuchung eines Befunds finden Sie im
[Persistenzleitfaden mit 131 Prüfungen](../persistence-hunter/index.md). Die
[Danksagungen zur Persistenz](../persistence-hunter/acknowledgements.md) beschreiben die Python- und
PowerShell-Projekte hinter dem Katalog.

## Verwendung { #usage }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei persistence-hunter -d <EVIDENCE> [OPTIONS]

Input:
  -d, --directory <DIR>  Evidence directory (a mounted Windows volume or a collection of hosts)
  -f, --file <FILE>      One registry hive (filesystem checks need a directory)
      --no-logs          Read dirty hives without replaying transaction logs

Filtering:
      --profile <FILE>               Replace the bundled allow/block profile with a YAML profile
      --min-severity <MIN_SEVERITY>  Minimum classification to include: info, low, medium, high [default: medium] [possible values: info, low, medium, high]
      --check <ID>                   Run only these check or ATT&CK IDs (comma separated; repeatable) [alias: --technique]

General Options:
  -h, --help         Show the help menu
      --list-checks  Show every available check and its ATT&CK technique

Analysis:
      --no-timeline  Skip MFT and event-log timestamp correlation
      --mft <FILE>   MFT to use for timestamp correlation (one host only)

Performance:
  -T, --threads <N>  Worker threads (default: logical CPU count)

Output:
  -o, --output <FILE>              Save findings to a file
  -t, --output-type <OUTPUT_TYPE>  Output type: csv (default), json, jsonl; without -o, write data to stdout [possible values: csv, json, jsonl]
      --coverage <FILE>            Save per-check coverage and errors to CSV
  -C, --clobber                    Overwrite output files

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Erster Scan { #first-scan }

```bash
juntei persistence-hunter -d evidence/ -o PersistenceHunter.csv --coverage Coverage.csv
```

Die Beweise können ein eingebundenes Windows-Volume, eine Velociraptor-Sammlung oder ein Verzeichnis mit
mehreren gesicherten Computern sein. Computer werden während der Analyse getrennt gehalten, und jedes Ergebnis enthält
`Comp`. Die Maskierung von Dateinamen in Sammlungen wird beim Auflösen von Windows-Pfaden dekodiert.

Für einen einzelnen Hive:

```bash
juntei persistence-hunter -f SOFTWARE -o software-persistence.csv --no-timeline
```

Ein einzelner Hive kann keine Prüfungen bedienen, die einen anderen Hive oder Dateisystembelege benötigen. Lesen Sie den Abdeckungs-
bericht, bevor Sie ein leeres Ergebnis als aussagekräftig behandeln.

Die Check-IDs und ihre ATT&CK-Zuordnungen auflisten:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,windows_service_image_path -o focused.csv
```

Verwenden Sie die von `--list-checks` ausgegebenen Check-IDs oder eine ATT&CK-ID wie `T1547.001`.
Eine übergeordnete ID wie `T1547` wählt ihre Untertechniken aus. `--technique` ist ein Alias für `--check`.
Jede Prüfung enthält in den Befunden ihre Technik, Beschreibung und Referenzlinks.

## Parallele Scans { #parallel-scans }

Der Befehl verwendet einen gemeinsamen Worker-Pool, standardmäßig mit der Anzahl der logischen CPUs. Setzen Sie
`-T, --threads <N>`, um die Anzahl der Worker zu wählen, oder `-T 1` für einen Scan mit einem Thread:

```bash
juntei persistence-hunter -d evidence/ --min-severity info -T 4 -o PersistenceHunter.csv
```

Beweiserkennung, unabhängige Prüfungen, Payload-Metadaten sowie MFT-/EVTX-Zeitleistenquellen laufen parallel. Die
geladenen Registry-Hives eines Hosts werden schreibgeschützt geteilt, und zwischengespeicherte Metadaten derselben Payload werden einmal initialisiert.
Hosts werden nacheinander geladen, um den Hive-Speicher nicht über die gesamte Sammlung zu vervielfachen. Befunde,
Abdeckung, Zeitstempelauswahl und Reihenfolge der Warnungen bleiben bei unterschiedlichen Thread-Anzahlen gleich.

`auto-parse` führt Parser-Prozesse bereits parallel aus und gibt daher jedem `persistence-hunter`-Kindprozess
einen Worker. Seine eigene Option `-T` steuert die Anzahl gleichzeitiger Parser-Prozesse.

## Zu sichernde Beweise { #evidence-to-collect }

| Quelle | Beispiele für Prüfungen |
|---|---|
| `SYSTEM` und zugehörige Transaktionsprotokolle | Dienste, Wiederherstellungsbefehle von Diensten, LSA-Pakete, Booteinstellungen, Authentifizierungsanbieter |
| `SOFTWARE` und zugehörige Transaktionsprotokolle | Run-Schlüssel, Winlogon, IFEO, COM-Registrierungen, TaskCache, Office-Einstellungen |
| `NTUSER.DAT` und `UsrClass.dat` jedes Benutzers | Benutzer-Autostarts, Umgebungsvariablen, COM-Überschreibungen, Office-Einstellungen |
| `SAM` und `SOFTWARE` | RID-Anomalien, verborgene Konten, ungewöhnliche Mitgliedschaft in der lokalen Gruppe Administratoren |
| `Windows/System32/Tasks` | Befehle und COM-Handler geplanter Aufgaben; Vergleich mit TaskCache |
| `.library-ms`-Dateien und nach CLSIDs benannte Ordner im Startmenü jedes Benutzers | Kandidaten für COM-Aktivierungsauslöser, verknüpft mit passenden Klassenregistrierungen von Benutzer/Computer |
| Autostartordner, PowerShell-Profile, Office-Vorlagen, Terminal-Einstellungen | Dateien und Befehle, die durch Benutzer- oder Systemauslöser ausgeführt werden |
| WMI `OBJECTS.DATA` | Instanzen von Ereigniskonsumenten und Bindungen von Filtern an Konsumenten |
| BITS `qmgr.db` oder `qmgr*.dat` | Wiederhergestellte Benachrichtigungsbefehle und Besitzer-SIDs |
| `$MFT` und relevante EVTX-Dateien | Unabhängige Zeitstempelkorrelation |
| Referenzierte ausführbare Dateien, DLLs und Katalogdateien | SHA256-Hashes und beschreibende PE-/Zertifikatsmetadaten |

Registry-Transaktionsprotokolle werden eingespielt, sofern vorhanden. `--no-logs` liest die Hives so, wie sie gesichert wurden;
ein nicht wiedergegebener „dirty“ Hive kann jüngste Änderungen auslassen und wird in der Abdeckung gemeldet. Für Prüfungen des aktiven Systems
wird das aktuelle Control Set verwendet. Die Prüfung auf eine DSRM-Hintertür untersucht zusätzlich
`ControlSet001`, `ControlSet002` und `CurrentControlSet`, sofern vorhanden; gleichen Sie den gemeldeten Pfad
mit `SYSTEM\\Select` ab, bevor Sie diese Einstellung als aktiv beschreiben. Andere historische Control Sets
lassen sich mit den einzelnen Registry-Parserbefehlen untersuchen.

Hive-Lesevorgänge teilen sich ein Eingabebudget von 1 GiB pro Computer. Computer-Hives werden vor Benutzer-Hives versucht;
Eingaben oder Transaktionsprotokolle, die nicht hineinpassen, werden in der teilweisen Abdeckung genannt. Andere lesbare Hives und
dateibasierte Prüfungen laufen weiter. Nur kanonische Hive-Speicherorte oder explizit übergebene Exporte begründen die
Identität eines Hives; eine Kopie im temporären Verzeichnis eines Benutzers kann den echten Hive dieses Benutzers nicht ersetzen.
Das in SOFTWARE aufgezeichnete ursprüngliche Systemlaufwerk wird dem gesicherten Systemvolume zugeordnet; wird ein ursprüngliches
`C:`-Volume unter einem anderen Laufwerksbuchstaben der Beweise eingebunden, gehen daher keine Payload-Metadaten verloren.

`auto-parse` führt den Hunter ebenfalls aus und speichert `persistence/PersistenceHunter.csv` (oder `.json`/`.jsonl`)
im Verzeichnis jedes Computers, mit `PersistenceHunter-logs.txt` daneben. Seine Befunde fließen zudem in
die optionale SIEM-Zeitleiste `-j <FILE>` ein. Die automatische Analyse verwendet `--min-severity info`, um das
vollständige Konfigurationsinventar zu bewahren, einschließlich der Einträge, auf die Zulassungsregeln zutreffen.

## COM-Registrierungen und Aktivierungsauslöser { #com-registrations-and-activation-triggers }

Die Prüfung `scheduled_task_files` wählt den Prinzipal jeder Aktion anhand von
`Actions Context` und passender `Principal id` im Aufgaben-XML aus. Ein Gruppenprinzipal wird als Gruppe beibehalten;
er kann nicht festlegen, welche Klassenregistrierungen welches einzelnen Benutzers verwendet würden.

Kandidaten für COM-Handler werden getrennt in der nativen und der 32-Bit-Klassenansicht aufgelöst. Berücksichtigt werden nur die
gesicherte `UsrClass.dat`/`NTUSER.DAT` des passenden Benutzers und das `SOFTWARE` des Computers.
SID-Prinzipale werden über gesicherte `ProfileList`-Pfade verknüpft. Kontonamen, einschließlich
`HOST\user` und `DOMAIN\user`, bevorzugen ein passendes `user.DOMAIN`-Profil gegenüber einem einfachen
`user`-Profil. Lokale Host-Aliase und `.\user` verwenden gesicherte Hostnamen für den Abgleich von Suffixen;
ein exaktes Suffix hat Vorrang vor anderen Host-Aliasen. Das einfache Profil wird nur berücksichtigt,
wenn kein qualifiziertes Profil passt. Jede solche Übereinstimmung wird in `Description` ausdrücklich als
**Heuristik anhand des Ordnernamens** gekennzeichnet; sie beweist weder den Besitz des Kontos noch eine Domänenmitgliedschaft. Mehrdeutige Übereinstimmungen
bleiben ungelöst und werden sowohl in der Beschreibung des Befunds als auch in der Abdeckung gemeldet; Mehrdeutigkeit
zwischen qualifizierten Profilen löst keinen Rückgriff auf das einfache Profil aus. Dies hängt nicht
von `Volatile Environment` ab, das normalerweise nur im Arbeitsspeicher existiert und nicht mit Registry-Hives gespeichert wird. Eine doppelte Benutzer-CLSID
überdeckt die untergeordneten Elemente der Computer-CLSID. Gruppenprinzipale verwenden COM-Kandidaten des Computers und behalten
diese Einschränkung in der Beschreibung des Befunds bei; ein normaler Gruppenprinzipal macht die
Abdeckung für sich genommen weder unvollständig noch verbraucht er die Einträge der Fehlerzusammenfassung.

Der Resolver folgt `InprocServer32` und `LocalServer32`, verwendet verwaltete `CodeBase`-Pfade
(einschließlich Versionsunterschlüsseln) und verknüpft AppID-`LocalService` mit dem `ImagePath` des gesicherten SYSTEM-
Dienstes. AppID-Zuordnungen zu Namen ausführbarer Dateien bleiben erhalten, doch ein Basisname legt nicht den
vollständigen Pfad einer ausführbaren Datei fest. Lokale `file:`-CodeBase-URLs werden dekodiert; entfernte URLs werden aufgezeichnet,
ohne sie aufzurufen. Fehlende Registrierungen behalten die CLSID mit
`ResolutionStatus: unresolved_com_class`, leeren Pfad-/Hash-Feldern und unbekanntem `Exists`.

Die Prüfung `start_menu_com` liest XML-URLs aus `.library-ms`-Dateien im Startmenü jedes Benutzers und erfasst
nach CLSIDs benannte Ordner, einschließlich leerer Verzeichnisse. Sie meldet Auslöserpfad, CLSID, Kandidaten-
server und Registry-Herkunft. Eine reine Dateisicherung kann kein leeres Verzeichnis bewahren,
und fehlende Benutzer-Hives verhindern eine vollständige Suche nach Registrierungen pro Benutzer. Symbolischen Links wird
nicht gefolgt. Es handelt sich um Konfigurationskandidaten: Architektur, Rechteerhöhung, UAC, Richtlinien und
Aktivierungskontext können beeinflussen, ob eine Registrierung tatsächlich verwendet wird. Untermauern Sie eine Ausführung
mit Prozess-, Modullade- oder anderen unabhängigen Belegen.

## Klassifizierung und Profile { #classification-and-profiles }

Die standardmäßige Mindestklassifizierung ist `medium`. Um das vollständige Inventar zu prüfen, einschließlich
der Einträge, auf die die mitgelieferte Zulassungsliste zutrifft:

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o all-persistence.csv
```

!!! warning "Verwenden Sie alle Schweregrade bei der Prüfung unvollständiger Sammlungen"
    Eine weit gefasste Zulassungsregel kann teilweise auf den Befehl eines Angreifers zutreffen, selbst wenn dessen Signierer fehlt
    oder abweicht. Upstream klassifiziert teilweise Übereinstimmungen als `LOW`, was der Standardschwellenwert
    `medium` des manuellen Befehls ausblendet. Dies kann bösartige geplante Aufgaben unter Windows-Verzeichnissen betreffen.
    Verwenden Sie `--min-severity info` für eine vollständige Untersuchung und prüfen Sie `PolicyReason`, `SignerStatus`,
    den Befehl und die Quellbelege gemeinsam. `auto-parse` behält diese Einträge bereits bei.

| Klassifizierung | Bedeutung |
|---|---|
| `HIGH` | Eine konfigurierte Sperrregel traf vollständig zu |
| `MEDIUM` | Keine Zulassungs- oder Sperrregel traf zu |
| `LOW` | Die harten Bedingungen einer Zulassungsregel trafen zu, aber die Signierer-Metadaten passten nicht oder waren nicht verfügbar |
| `INFO` | Eine Zulassungsregel traf vollständig zu |

Dies sind Richtlinienklassifizierungen, keine gemessenen Wahrscheinlichkeiten einer Kompromittierung. Ein standardmäßiger `MEDIUM`-
Eintrag ist kein Beweis für Malware. `INFO` belegt weder, dass eine Datei eine gültige Signatur hat, noch dass
ein Angreifer sie nicht missbraucht hat. Eine teilweise Übereinstimmung mit einer Zulassungsregel liegt unter dem Standardschwellenwert; verwenden Sie daher
`--min-severity low` oder `info`, wenn Sie spärliche Sammlungen prüfen oder die Zulassungsliste abstimmen.

Übergeben Sie ein YAML-Profil mit `--profile profile.yaml`. Es **ersetzt** das mitgelieferte Profil. Globale
und prüfungsspezifische Regeln innerhalb der ausgewählten Datei werden kombiniert; jede vollständig zutreffende Sperrregel hat Vorrang.
Zum Beispiel:

```yaml
allow: []
block:
  - reason: Example known-bad command fragment
    value_matches: 'example-malicious-loader\.exe'
checks:
  windows_service_image_path:
    enabled: true
    allow:
      - reason: Organization's expected service path
        value_matches: '^C:\\Program Files\\ExampleCorp\\Agent\\agent\.exe$'
        signer: ExampleCorp
```

Unterstützte Regelbedingungen sind `path_matches`, `value_matches`, `hash` (64 hexadezimale SHA256-
Ziffern), `not_lolbin` und `signer` (Teilzeichenfolge ohne Beachtung der Groß-/Kleinschreibung). Alle angegebenen harten Bedingungen müssen
zutreffen. Signierer-Metadaten sind die oben beschriebene weiche Bedingung. Reguläre Ausdrücke verwenden die Syntax regulärer Ausdrücke von Rust:
Lookarounds und Rückverweise werden nicht unterstützt. Unbekannte Check-IDs, ungültige Ausdrücke
und unbekannte Profilfelder führen zu einem Fehler, statt die Richtlinie stillschweigend abzuschwächen.

## Ausgabefelder { #output-fields }

CSV ist das Standarddateiformat. Verwenden Sie `-t json` für lesbare JSON-Objekte oder `-t jsonl` für kompakte
Ausgabe mit einem Objekt pro Zeile. Ohne `-o` schreibt ein explizites `-t` nur Daten nach stdout; Abdeckungs-
warnungen gehen nach stderr. Ohne beide Optionen gibt Juntei eine Zusammenfassung und bis zu 30 Kandidaten aus.

| Felder | Wie sie zu interpretieren sind |
|---|---|
| `Comp`, `CheckID`, `Technique`, `MitreID` | Welchen Computer und welchen Persistenzmechanismus die Zeile beschreibt |
| `Path`, `Value` | Der Registry-Ort, die Datei, die Aufgabe oder der wiederhergestellte Datensatz und dessen konfigurierter Inhalt |
| `Description`, `References` | Warum der Mechanismus wichtig ist und wo Sie mehr dazu lesen können |
| `AccessGained` | Der übliche Geltungsbereich des Mechanismus (`USER` oder `SYSTEM`; BITS kann auch `LOCAL_SERVICE`/`NETWORK_SERVICE` angeben); kein Beweis, dass ein bestimmter Befehl diese Berechtigung erlangt hat |
| `Severity`, `PolicyReason` | Klassifizierung durch das Profil, Erläuterung der zutreffenden Richtlinie und etwaige Einschränkung wegen teilweiser Übereinstimmung oder nicht verifiziertem Signierer |
| `SourceFile`, `RecordStatus` | Exakte gesicherte Quelle, deren Pfad mit der Eingabe wie übergeben beginnt (`./HOST/...` bei auto-parse), sowie Kontext zu Zuordnung/Wiedergabe/Wiederherstellung; verfolgen Sie dies bis zum ursprünglichen Beweis zurück |
| `KeyLastWrite` | Der Zeitstempel des Registry-**Schlüssels**. Er datiert keinen einzelnen Wert |
| `LastChange`, `ChangeSource`, `ChangeEvidence` | Unabhängig korrelierter Änderungszeitpunkt, Quellnachweis und ob die Korrelation gelungen ist |
| `ResolvedPath`, `ResolutionStatus`, `Exists` | Payload-Pfad und Ergebnis der Offline-Auflösung; nicht gesicherte Beweise sind unbekannt, nicht `false` |
| `SHA256` | SHA256 der gesicherten Payload-Bytes, sofern lesbar |
| `Signer`, `SignerStatus` | Beschreibende eingebettete Zertifikats-/Katalogmetadaten und deren Verfügbarkeit; Vertrauenswürdigkeit des Zertifikats und Gültigkeit der Signatur werden nicht geprüft |
| `IsLolbin`, `Launcher` | Ob das konfigurierte Startprogramm oder die Payload in der mitgelieferten LOLBAS-Liste steht, und das für eine vermittelte Payload verwendete Startprogramm |
| `IsBuiltin`, `IsInOS-Directory` | Kontext für gängige Windows-Befehle und Systemverzeichnisse; keines der Felder belegt Legitimität |

Ein `rundll32.exe`-Befehl kann eine DLL-Payload benennen. Juntei bewahrt das Startprogramm, während es die Payload auflöst und
hasht. Ein Verweis außerhalb des gesicherten Computers, ein fehlendes Laufwerk, eine mehrdeutige
Übereinstimmung ohne Beachtung der Groß-/Kleinschreibung oder eine nicht aufgelöste Umgebungsvariable wird nicht durch eine Datei
vom Rechner der Analystin oder des Analysten ersetzt.

## Zeitstempel und Einschränkungen { #timestamps-and-limitations }

Der Hunter verwendet die Änderungszeiten kopierter Dateien auf der Analyse-VM nicht als Angriffszeitstempel.
Sofern verfügbar, korreliert er Registry-Schreibvorgänge mit Sysmon-Ereignis 13, Aufgabeninstallationen mit
Security 4698 oder Task Scheduler 106, Dienstinstallationen mit System 7045 und dateibasierte
Konfigurationen mit den ursprünglichen `$MFT`-Zeitstempeln. Die Herkunft erscheint in `ChangeSource`.
Ein Ereignis kann eine Konfigurationsänderung datieren, ohne zu beweisen, dass die konfigurierte Payload ausgeführt wurde.
Ereignis 6011 im lokalen System-Protokoll kann einen früheren Computernamen belegen. Der Hunter folgt verifizierten Umbenennungs-
ketten, die am gesicherten Computer verankert sind, behält dessen aktuellen `Comp`-Wert bei und zeichnet den
ursprünglichen Ereigniscomputer und den Nachweis des Umbenennungsdatensatzes in `ChangeSource` auf. Nicht belegte fremde Namen,
archivierte Protokolle, weitergeleitete Protokolle und eine andere Windows-Installation werden über diese Regel nicht akzeptiert.

Gesicherte Werte für ComputerName/ActiveComputerName, TCP/IP Hostname/NV Hostname sowie Hostname/FQDN aus Client-Informationen
liefern Host-Aliase und decken DNS-/NetBIOS-Unterschiede sowie ausstehende Umbenennungen ab. Ohne gesicherte Identität
kann ein einzelner Name in den primären Protokollen eine Datierung stützen, mit ausdrücklich als nicht verifiziert gekennzeichneter Identitätsherkunft
und einer Abdeckungswarnung. Mehrere nicht verifizierte Namen werden dem Host nicht zugeordnet. Abgelehnte Computer-
namen werden als Einschränkung der Korrelationsabdeckung gemeldet; ihre Kanäle werden dennoch als gesichert markiert.

Verwenden Sie `--mft <FILE>`, um für einen einzelnen Computer eine extern gesicherte MFT auszuwählen. Verwenden Sie `--no-timeline`,
um die Zeitstempelkorrelation zu überspringen. `NOT_RUN`, `NOT_APPLICABLE`, `NO_ARTIFACT`, `NO_MATCH` und `REJECTED`
erklären ein leeres `LastChange`; ein leeres Feld ist kein Beleg dafür, dass der Eintrag harmlos ist.

Die Wiederherstellung von WMI und BITS ist begrenztes Carving und kein vollständiger Parser für Datenbanktransaktionen.
Wiederhergestellte Zeichenfolgen oder Bindungen können alte/gelöschte Daten enthalten; diese Datensätze müssen untermauert werden.
WMI-Klassenschemata allein werden nicht als Abonnements gezählt. Ein TaskCache-Eintrag ohne gesicherte
Aufgaben-XML-Datei ist eine zu prüfende Unstimmigkeit; eine Triage-Sammlung hat diese Datei möglicherweise einfach ausgelassen.
Katalog- und PE-Metadaten sind nicht verifiziert und können in einer Sammlung fehlen. Prüfungen auf das Ersetzen von Dateien der Bedienungshilfen
hängen von geeigneten Referenz-Signaturmetadaten ab; fehlende Katalogbelege dürfen nicht dazu führen,
dass jede ausführbare Datei der Bedienungshilfen markiert wird.

Die Abdeckung wird für jede Prüfung aufgezeichnet, auch für Prüfungen ohne Befunde. `complete` bedeutet, dass die
zutreffende gesicherte Quelle gelesen wurde; es beweist nicht, dass die Sammlung vollständig ist oder dass
der Computer keine andere Persistenz aufweist. `partial`, `not_collected`, `failed` und `disabled` erklären
die Grenzen. Bewahren Sie `--coverage Coverage.csv` und das Parserprotokoll zusammen mit Ihren Befunden auf.

Die Portierung verwendet die CSV-/JSON-/JSONL-Formate von Juntei. Sie ruft weder das Upstream-Python-Programm auf, noch fragt sie
Online-Anreicherungsdienste ab, noch bildet sie die HTML-/XLSX-Oberfläche von Upstream nach.

## Weitere Informationen { #further-details }

- [Implementierung und Profile von PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper)
- [MITRE-ATT&CK-Persistenztechniken](https://attack.mitre.org/tactics/TA0003/)
- [Microsoft: Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
- [Inventar der Registry-Autostarts](../artifacts/persistence.md)
- [Dienste](../artifacts/services.md) und [TaskCache](../artifacts/taskcache.md)
