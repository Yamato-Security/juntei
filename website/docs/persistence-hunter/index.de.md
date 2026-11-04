# Persistence Hunter { #persistence-hunter }

Persistenz ist ein Weg, Code erneut auszuführen: wenn Windows startet, sich ein Benutzer anmeldet, eine Anwendung
geöffnet wird, ein geplanter Zeitpunkt erreicht ist oder ein anderes Ereignis eintritt. Angreifer können dieselben Einstellungen missbrauchen,
die legitime Software für Updates, Geräteunterstützung, Sicherheit und Administration verwendet.

`juntei persistence-hunter` untersucht gesicherte Windows-Beweise offline mit **131 Prüfungen**, die
diese Mechanismen sowie damit zusammenhängende Anomalien bei Konten und Schutzmaßnahmen abdecken. Es liest Registry-Hives, Aufgabendateien,
Skripte, Anwendungseinstellungen und ausgewählte Datenbankeinträge.

Diese Seite erläutert jede implementierte Prüfung. Vollständige Optionen, Ausgabefelder, Profile und die
Behandlung von Zeitstempeln finden Sie im [Befehlsleitfaden](../commands/persistence-hunter.md). Die
[Danksagungen](acknowledgements.md) verweisen auf das Python-Projekt PyrsistenceSniper und das ursprüngliche
PowerShell-Werkzeug PersistenceSniper.

## Eine Untersuchung starten { #start-an-investigation }

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o PersistenceHunter.csv --coverage Coverage.csv
```

Verwenden Sie `--min-severity info` für das vollständige Kandidateninventar. Der manuelle Befehl verwendet standardmäßig
`medium`; sein Profil kann teilweise Übereinstimmungen mit der Zulassungsliste als `LOW` einstufen, selbst wenn fehlende Belege zu Payload oder
Signierer einen vollständigen Vergleich verhindern. Weder eine Übereinstimmung mit der Zulassungsliste noch ein unbekannter Eintrag entscheidet,
ob etwas bösartig ist. `auto-parse` behält bereits alle Schweregrade bei und schreibt die Ergebnisse
in die Kategorie `persistence/` jedes Computers.

Um sich auf einen Mechanismus zu konzentrieren, kopieren Sie dessen **Check-ID** aus der folgenden Liste:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,scheduled_task_files --min-severity info -o focused.csv
```

Der Befehl verwendet standardmäßig einen gemeinsamen Worker-Pool. Fügen Sie `-T 4` hinzu, um vier Worker zu wählen, oder `-T 1`
für einen Single-Thread-Scan. Die Eingabe kann mehrere Computer enthalten; deren Beweise bleiben getrennt.

## Ergebnisse als Beweise lesen { #read-findings-as-evidence }

| Frage | Was zu prüfen ist |
|---|---|
| Was wurde konfiguriert und wo? | `CheckId`, `Path` und der vollständige `Value` einschließlich Argumenten. Folgen Sie `SourceFile` zum gesicherten Hive, zur Datei oder Datenbank. |
| Existiert das Ziel in der Sammlung? | `ResolvedPath`, `ResolutionStatus` und `Exists`. Nicht gesicherte, mehrdeutige oder nicht unterstützte Verweise sind unbekannt; Juntei ersetzt sie nicht durch eine Datei vom Rechner der Analystin oder des Analysten. |
| Ist die Konfiguration wirksam? | Der tatsächliche Auslöser, Aktivierungsflags, das ausgewählte Control Set, die Windows-/Anwendungsversion, der Benutzer und die Architektur. Einige Prüfungen behalten absichtlich inaktive Konfigurationen bei. |
| Ist dies ein Angriff? | `Severity` und `PolicyReason` erklären den Profilabgleich, kein Urteil. Vergleichen Sie mit erwarteter Software, dem vollständigen Befehl, Datei-Hashes, unabhängigen Ausführungsbelegen und der umgebenden Zeitleiste. |
| Wer könnte es ausführen? | `AccessGained` ist ein Hinweis auf Geltungsbereich/Kontext. Maschinenweite Konfiguration lautet oft `SYSTEM`, das beweist aber nicht, dass der gestartete Prozess als SYSTEM lief. Aufgabenprinzipale und BITS-Besitzer-SIDs liefern, sofern verfügbar, spezifischeren Kontext. |
| Wann wurde es geändert? | `KeyLastWrite` datiert den gesamten Registry-Schlüssel. `LastChange`, `ChangeSource` und `ChangeEvidence` beschreiben, sofern verfügbar, eine separate MFT-/EVTX-Korrelation. Keines dieser Felder datiert automatisch die Ausführung der Payload. |
| Wie zuverlässig ist der wiederhergestellte Zustand? | Lesen Sie `RecordStatus`, `Coverage.csv` und Warnungen zu wiedergegebenen, per Carving gewonnenen, beschädigten, fehlenden oder eingeschränkten Beweisen. Ein historisch wiederhergestellter Datensatz ist möglicherweise nicht mehr aktiv. |
| Ist ein genannter Herausgeber vertrauenswürdig? | `Signer` und `SignerStatus` sind beschreibende PE-/Zertifikats-/Katalogmetadaten. Juntei validiert keine Authenticode-Signaturen und kein Zertifikatsvertrauen. |

Das Fehlen von Ergebnissen beweist nicht das Fehlen von Persistenz. Eine Abdeckungszeile `complete` bedeutet,
dass die anwendbare gesicherte Quelle gelesen wurde, nicht dass die Sicherung jede relevante Datei,
jeden Benutzer oder jeden möglichen Windows-Persistenzmechanismus enthält. Andere Abdeckungszustände erklären fehlende,
teilweise, deaktivierte oder fehlgeschlagene Prüfungen. Der Hunter ist kein Scan von Live-Speicher, Netzwerk oder Cloud-Identitäten.

## Katalog und Notation der Speicherorte { #catalogue-and-location-notation }

Der Katalog basiert auf PyrsistenceSniper sowie der nativen Prüfung `start_menu_com`;
die folgenden Beschreibungen dokumentieren
das native Verhalten von Juntei einschließlich der Einschränkungen seiner Beweise. Es gibt **116 registrygestützte
Prüfungen und 15 Datei-/Repository-Prüfungen**. Einige Dateiprüfungen ziehen auch Registry-Daten hinzu; es handelt sich also um
Implementierungszahlen, nicht um einander ausschließende Anforderungen an die Sicherung. Die Mechanismen
sind unten nach Untersuchungszweck gruppiert und innerhalb jeder Gruppe alphabetisch nach Check-ID sortiert.

- `HKLM\SYSTEM`, `HKLM\SOFTWARE` und `HKLM\SAM` bezeichnen gesicherte Maschinen-Hives, die normalerweise
  aus `Windows\System32\config` gesichert werden, mit Transaktionsprotokollen, sofern verfügbar.
- `HKU\<user>` bezeichnet den jeweils relevanten gesicherten Benutzer-Hive. Die meisten Benutzereinstellungen stammen aus NTUSER.DAT;
  Klassenregistrierungen können, wie angegeben, aus `AppData\Local\Microsoft\Windows\UsrClass.dat` stammen.
- `<ControlSet>` bezeichnet das ausgewählte aktive Control Set in SYSTEM. Die zusätzlichen historischen
  Speicherorte der DSRM-Prüfung sind unten ausdrücklich angegeben. Prüfen Sie `SYSTEM\Select`, wenn Sie diese untersuchen.
- `<name>` ist eine variable Pfadkomponente. In Tabellen mit Registry-Zielen bezeichnet `<child>` unmittelbare
  untergeordnete Registrierungen; es bedeutet keinen unbegrenzten rekursiven Durchlauf. Ausdrücklich rekursive Prüfungen
  beschreiben ihren Durchlauf gesondert. Eckige Klammern in Office-Pfaden kennzeichnen eine optionale Version.
- Tabellen führen die tatsächlichen Maschinen- und Benutzeransichten auf, die die Prüfung auswählt. Eine 32-Bit-Ansicht ist aufgeführt,
  wo sie implementiert ist; nehmen Sie nicht an, dass jede Prüfung jede mögliche Architektur/Ansicht untersucht.
- Generische Wertprüfungen geben nicht leere konfigurierte Einträge aus, einschließlich Elementen von Mehrfachzeichenfolgen und
  gegebenenfalls des numerischen Werts Null. Sie setzen für sich genommen weder eine bösartige Payload noch einen
  aktivierten Auslöser oder eine erfolgreiche Ausführung voraus. Selektivere Bedingungen sind bei der jeweiligen Prüfung angegeben.

ATT&CK-IDs sind übernommene Katalogzuordnungen und Links zur weiteren Vertiefung. Sie beschreiben möglichen
Missbrauch, nicht die Feststellung, dass ein Angreifer diese Technik angewendet hat; Prüfungen von Schutzeinstellungen und Diagnose-
funktionen können eher begünstigende Bedingungen als einen eigenständigen Persistenzmechanismus beschreiben.

| Mechanismusgruppe | Prüfungen |
|---|---:|
| [Boot, Anmeldung und Start](#boot-sign-in-and-startup) | 32 |
| [Dienste und Systemerweiterungen](#services-and-system-extensions) | 13 |
| [Geplante Aufgaben](#scheduled-tasks) | 3 |
| [WMI, BITS und andere Ereignisauslöser](#wmi-bits-and-other-event-triggers) | 11 |
| [Explorer, COM und Datei- oder URL-Handler](#explorer-com-and-file-or-url-handlers) | 15 |
| [Office-Anwendungen und Add-ins](#office-applications-and-add-ins) | 7 |
| [Debugger, Absturzhandler und Kompatibilitäts-Shims](#debuggers-crash-handlers-and-compatibility-shims) | 13 |
| [.NET-Laufzeit und Profiler-Einstellungen](#net-runtime-and-profiler-settings) | 7 |
| [Überschreibungen der DLL- und Programmsuche](#dll-and-executable-lookup-overrides) | 18 |
| [Konten und Sichtbarkeit bei der Anmeldung](#accounts-and-sign-in-visibility) | 4 |
| [Authentifizierungskomponenten und Schutz von Anmeldeinformationen](#authentication-components-and-credential-protection) | 8 |
| **Gesamt** | **131** |

## Boot, Anmeldung und Start { #boot-sign-in-and-startup }

Diese Einstellungen sorgen dafür, dass ein Befehl oder eine Bibliothek während des Boot-Vorgangs, der Anmeldung oder der Desktop-Initialisierung ausgeführt wird. Prüfen Sie die Windows-Version und den betroffenen Benutzer: Eine Einstellung kann veraltet, deaktiviert oder für einen Kiosk statt für einen Angreifer gedacht sein.

### Active Setup {#active_setup}

**Check-ID:** `active_setup` · **ATT&CK laut Katalog:** [T1547.014](https://attack.mitre.org/techniques/T1547/014/)

**Was Juntei prüft:** Liest `StubPath` aus maschinenweiten Active-Setup-Komponentenregistrierungen in der nativen und der 32-Bit-Ansicht. Windows kann diese Befehle bei der Vorbereitung eines Benutzerprofils verwenden; die Prüfung überspringt die wörtlichen Platzhalterwerte `U` und `/UserInstall`.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Active Setup\Installed Components\<component>\StubPath` und der entsprechende `Wow6432Node`-Baum.

**Untersuchungshinweise:** Vergleichen Sie jede Komponente mit ihrem installierten Produkt und prüfen Sie die Befehlsargumente. Eine maschinenweite Registrierung kann zur Ausführung in der Sitzung eines sich anmeldenden Benutzers führen; die Bezeichnung `SYSTEM` in der Ausgabe beschreibt den Geltungsbereich der Konfiguration, nicht einen Beweis für das Token des daraus resultierenden Prozesses.

### Boot Execute {#boot_execute}

**Check-ID:** `boot_execute` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Liest die Mehrfachzeichenfolge `BootExecute` des Session Managers und behält einzelne nicht leere Einträge bei. Sie zeichnet Befehle auf, die für eine frühe Bootphase konfiguriert sind.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `BootExecute` |

**Untersuchungshinweise:** `autocheck autochk *` ist ein bekannter Standardwert, und Datenträgerwerkzeuge können legitime Einträge hinzufügen. Untersuchen Sie hinzugefügte Befehle und deren Dateien, ohne anzunehmen, dass jeder vom Standard abweichende Eintrag ausgeführt wurde oder bösartig ist.

### Boot Verification Program {#boot_verification_program}

**Check-ID:** `boot_verification_program` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Liest `ImagePath` von `BootVerificationProgram`, einen Speicherort einer ausführbaren Datei, der mit der Bootverifizierung verbunden ist.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\BootVerificationProgram` | `ImagePath` |

**Untersuchungshinweise:** Prüfen Sie, ob dieser ältere Mechanismus auf die gesicherte Windows-Version und Bootkonfiguration anwendbar ist. Lösen Sie die Zieldatei auf und vergleichen Sie ihren Zweck mit der erwarteten Bootverwaltungssoftware.

### Explorer-Wert Load {#explorer_load}

**Check-ID:** `explorer_load` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Liest den Wert `Load`, der mit der Konfiguration der Windows-Anmeldeinitialisierung verbunden ist, aus Maschinen- und Benutzer-Hives.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |

**Untersuchungshinweise:** Untersuchen Sie unbekannte Befehle, vom Benutzer beschreibbare Ziele und Argumente, die Inhalte abrufen oder dekodieren. Bestätigen Sie Anwendbarkeit und Ausführung gesondert; ein leeres Ergebnis deckt andere Anmeldemechanismen nicht ab.

### Schriftarttreiber {#font_drivers}

**Check-ID:** `font_drivers` · **ATT&CK laut Katalog:** [T1547](https://attack.mitre.org/techniques/T1547/)

**Was Juntei prüft:** Zählt nicht leere Werte im veralteten Registry-Schlüssel `Font Drivers` als mögliche Bibliotheksregistrierungen auf.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Font Drivers` | Alle nicht leeren Werte |

**Untersuchungshinweise:** Behandeln Sie dies als ungewöhnliche, veraltete Konfigurationsoberfläche. Prüfen Sie die Unterstützung durch die Windows-Version und die referenzierte Bibliothek, bevor Sie schließen, dass sie geladen werden könnte, und bestätigen Sie die tatsächliche Nutzung mit Belegen zu Prozessen oder dem Laden von Images.

### Gruppenrichtlinien-Erweiterungs-DLLs {#gp_extension_dlls}

**Check-ID:** `gp_extension_dlls` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest `DllName` aus jeder unmittelbaren Registrierung einer Gruppenrichtlinienerweiterung unter Winlogon. Diese DLLs erweitern die Richtlinienverarbeitung.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\GPExtensions\<child>` | `DllName` |

**Untersuchungshinweise:** Verwaltungs- und Sicherheitsprodukte können Erweiterungen installieren. Ordnen Sie GUID und DLL der Erweiterung dem erwarteten Produkt zu und untersuchen Sie Belege zur Richtlinienverarbeitung und zum Laden von Images, bevor Sie eine Ausführung behaupten.

### Gruppenrichtlinienskripte {#gp_scripts}

**Check-ID:** `gp_scripts` · **ATT&CK laut Katalog:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Was Juntei prüft:** Wertet Befehl/Parameter-Paare aus den gesicherten Registrierungsdateien `scripts.ini` und `psscripts.ini` der lokalen Gruppenrichtlinie aus, einschließlich benutzerspezifischer lokaler Richtlinienspeicher. Die Abschnitte für Start/Herunterfahren und Anmeldung/Abmeldung behalten ihre konfigurierten Befehle und Argumente bei, wobei Maschinenabschnitte von Benutzerabschnitten unterschieden werden.

**Untersuchte Beweise:**

- `Windows\System32\GroupPolicy\Machine\Scripts\scripts.ini` / `psscripts.ini` und die entsprechenden Dateien unter `GroupPolicy\User\Scripts`.
- `Windows\System32\GroupPolicyUsers\<SID>\User\Scripts\scripts.ini` / `psscripts.ini`.

**Untersuchungshinweise:** Nur die implementierten lokalen Registrierungsorte für Richtlinien werden untersucht; dies ist keine vollständige SYSVOL-/GPO-Prüfung. Eine Skriptdatei, die an anderer Stelle unter einem Richtlinienverzeichnis liegt, reicht nicht aus. Korrelieren Sie ihre Registrierung mit Belegen zur Richtlinienverarbeitung und Ausführung.

### Anmeldeskripte (UserInitMprLogonScript) {#logon_scripts}

**Check-ID:** `logon_scripts` · **ATT&CK laut Katalog:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Was Juntei prüft:** Liest den Umgebungswert `UserInitMprLogonScript` des Benutzers, der ein Skript für die Anmeldung dieses Benutzers festlegen kann.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKU\<user>\Environment` | `UserInitMprLogonScript` |

**Untersuchungshinweise:** Vergleichen Sie mit genehmigten Anmeldeskripten und prüfen Sie das tatsächliche Skript. Der Wert beweist weder eine nachfolgende Anmeldung noch eine Ausführung, und von der Domäne bereitgestellte Skripte erfordern zusätzliche Richtlinienbelege.

### Platform Execute {#platform_execute}

**Check-ID:** `platform_execute` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Liest jeden nicht leeren Eintrag `PlatformExecute` im Session Manager. Er identifiziert Programme, die für die plattformbezogene Bootinitialisierung ausgewählt sind.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `PlatformExecute` |

**Untersuchungshinweise:** Hardware- oder Bereitstellungsabläufe können einen Eintrag erklären. Ermitteln Sie, ob der Bootpfad dieses Windows-Builds ihn verwendet, und untersuchen Sie das referenzierte Image, statt aus dem Wertnamen auf einen Angriff zu schließen.

### RDP-Client-Erweiterungs-DLL {#rdp_clx_dll}

**Check-ID:** `rdp_clx_dll` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Liest `ClxDllPath` in der Standardbenutzerkonfiguration des Terminal Servers als möglichen Speicherort einer RDP-bezogenen Erweiterung.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\DefaultUserConfiguration` | `ClxDllPath` |

**Untersuchungshinweise:** Prüfen Sie die Windows-/RDP-Version und die installierten Erweiterungen. Eine konfigurierte DLL ist ein Ansatzpunkt für die Analyse geladener Images; der Registry-Wert allein zeigt nicht, dass eine RDP-Verbindung oder ein DLL-Ladevorgang stattgefunden hat.

### RDP-Virtual-Channel-DLL {#rdp_virtual_channel}

**Check-ID:** `rdp_virtual_channel` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Zählt Werte unterhalb maschinenweiter Registrierungen `Default\AddIns` des Terminal Server Client auf. Binärwerte werden nach Möglichkeit als UTF-16LE dekodiert, sodass verwertbare Erweiterungspfade erhalten bleiben.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Terminal Server Client\Default\AddIns\<add-in>`: alle Werte.

**Untersuchungshinweise:** RDP-Clients und Produkte zur Geräteumleitung registrieren legitimerweise Add-ins. Prüfen Sie die tatsächliche DLL oder den Befehl und die betreffende Client-Sitzung; nicht jeder Wert unter einem Add-in ist ein Pfad zu einer ausführbaren Datei.

### RDP-WDS-Startprogramme {#rdp_wds_startup}

**Check-ID:** `rdp_wds_startup` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Liest `StartupPrograms` aus der Konfiguration des RDP-Windows-Station-Treibers. Es identifiziert Programme, die für den Start von Remotesitzungen konfiguriert sind.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\Wds\rdpwd` | `StartupPrograms` |

**Untersuchungshinweise:** `rdpclip` ist ein bekannter Eintrag. Untersuchen Sie Ergänzungen und die RDP-Konfiguration des Hosts und ermitteln Sie anhand von Sitzungs-/Prozessdatensätzen, ob das konfigurierte Programm tatsächlich gestartet wurde.

### Registry-Run-Schlüssel {#run_keys}

**Check-ID:** `run_keys` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Zählt Run-, RunOnce-, RunEx-, RunOnceEx- und richtlinienbasierte Explorer-Run-Werte aus Maschinen-/Benutzer-Hives sowie die aufgeführten 32-Bit-Schlüssel der Maschine auf. RunEx-/RunOnceEx-Abschnitte werden mit Begrenzungen und Zyklusprüfungen durchlaufen.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | Alle nicht leeren Werte |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | Alle nicht leeren Werte |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | Alle nicht leeren Werte |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | Alle nicht leeren Werte |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | Alle nicht leeren Werte |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | Alle nicht leeren Werte |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | Alle nicht leeren Werte |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | Alle nicht leeren Werte |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | Alle nicht leeren Werte |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | Alle nicht leeren Werte |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Run` | Alle nicht leeren Werte |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnce` | Alle nicht leeren Werte |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnceEx` | Alle nicht leeren Werte |

**Untersuchungshinweise:** Software-Updater und Infobereichsanwendungen erscheinen hier häufig. Vergleichen Sie die ausführbare Datei, die Befehlsargumente und den Benutzerkontext. Ein RunOnce-Eintrag kann nach der Verwendung verschwinden; der gesicherte Zustand ist kein vollständiger Verlauf früherer Anmeldungen.

### RunServices (veraltet) {#run_services}

**Check-ID:** `run_services` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Zählt Werte im veralteten maschinenweiten Schlüssel `RunServices` auf.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServices` | Alle nicht leeren Werte |

**Untersuchungshinweise:** Dies ist eine historische Konfigurationsoberfläche, keine Garantie, dass ein moderner Windows-Build sie ausführt. Untersuchen Sie das Ziel und die Anwendbarkeit auf das Betriebssystem und suchen Sie dann nach unabhängigen Ausführungsbelegen.

### RunServicesOnce (veraltet) {#run_services_once}

**Check-ID:** `run_services_once` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Zählt Werte im veralteten maschinenweiten Schlüssel `RunServicesOnce` auf.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServicesOnce` | Alle nicht leeren Werte |

**Untersuchungshinweise:** Behandeln Sie nicht leere Werte als Konfigurationshinweise. Prüfen Sie, ob die Windows-Version der Quelle den Mechanismus unterstützt und ob Bereitstellungs- oder Kompatibilitätssoftware den Eintrag erklärt.

### S0-Initialbefehl {#s0_initial_command}

**Check-ID:** `s0_initial_command` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Liest `S0InitialCommand` aus dem Session Manager als möglichen Initialisierungsbefehl für Session 0.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `S0InitialCommand` |

**Untersuchungshinweise:** Klären Sie die Anwendbarkeit auf den gesicherten Windows-Build und den Bootzustand. Ein konfigurierter Befehl ist kein mit Zeitstempel versehener Nachweis einer Ausführung beim Booten; prüfen Sie seine Datei und korrelieren Sie mit Boot-/Prozesstelemetrie.

### Session Manager Execute {#session_manager_execute}

**Check-ID:** `session_manager_execute` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Liest nicht leere `Execute`-Einträge aus dem Session Manager und legt so Programme offen, die für die Sitzungsinitialisierung ausgewählt sind.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `Execute` |

**Untersuchungshinweise:** Vergleichen Sie mit der erwarteten Image-Baseline und prüfen Sie ungewöhnliche native ausführbare Dateien oder Pfade. Klären Sie, ob nach der Änderung ein relevanter Boot-Vorgang bzw. eine relevante Sitzung stattfand, bevor Sie eine Ausführung behaupten.

### Session Manager SubSystems {#session_manager_subsystems}

**Check-ID:** `session_manager_subsystems` · **ATT&CK laut Katalog:** [T1547](https://attack.mitre.org/techniques/T1547/)

**Was Juntei prüft:** Liest den Wert `Windows` im Schlüssel `SubSystems` des Session Managers, der den Befehl des Windows-Subsystems konfiguriert.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\SubSystems` | `Windows` |

**Untersuchungshinweise:** Der normale Befehl ist lang und versionsabhängig. Vergleichen Sie den vollständigen Wert und die Argumente mit einer vertrauenswürdigen Baseline desselben Builds; eine Änderung einer kritischen Subsystemeinstellung kann zudem den Start verhindern.

### Setup Execute {#setup_execute}

**Check-ID:** `setup_execute` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Liest `SetupExecute`-Einträge im Session Manager und erfasst so setupbezogene Befehle, die für den Start konfiguriert sind.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `SetupExecute` |

**Untersuchungshinweise:** Installations-, Upgrade- und Wiederherstellungsaktivitäten können legitime Einträge erzeugen. Vergleichen Sie mit Setup-Protokollen und dem referenzierten Image, bevor Sie die Konfiguration einer Persistenz zuschreiben.

### Umleitung des Autostartordners über Shell Folders {#shell_folders_startup}

**Check-ID:** `shell_folders_startup` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Vergleicht `Startup` und `Common Startup` sowohl in Shell Folders als auch in User Shell Folders mit den Standardspeicherorten des Autostartordners. Vom Standard abweichende Umleitungen werden gemeldet, und deren gesicherte Ordnerinhalte werden untersucht.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders` und `User Shell Folders`: `Common Startup`.
- Entsprechende Einstellungen unter `Software\Microsoft\Windows\CurrentVersion\Explorer` in NTUSER.DAT: `Startup` sowie gesicherte Dateien an den umgeleiteten Speicherorten.

**Untersuchungshinweise:** Ein umgeleiteter Autostartordner kann Autostarts verbergen, doch eine Umleitung kann legitim sein. Der Pfad muss innerhalb der gesicherten Beweise auflösbar sein; ein nicht gesichertes Ziel ist unbekannt und kein Beweis dafür, dass der Ordner leer war.

### Überschreibung des Shell Launchers {#shell_launcher}

**Check-ID:** `shell_launcher` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Liest den Richtlinienwert `Shell` und die Shell-Zuordnung für den Boot-Abschnitt von `system.ini` aus SOFTWARE. Dies sind alternative Oberflächen zur Auswahl der Shell.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `Shell` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\IniFileMapping\system.ini\boot` | `Shell` |

**Untersuchungshinweise:** Kioske und Anwendungs-Appliances können den Explorer absichtlich ersetzen. Vergleichen Sie die ausgewählte Shell mit der Rolle des Rechners und bestätigen Sie, dass die Einstellung für die betreffende Sitzung wirksam war.

### Autostartordner {#startup_folder}

**Check-ID:** `startup_folder` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Untersucht direkt enthaltene Dateien in den benutzerspezifischen und gemeinsamen Standard-Autostartordnern und folgt dabei der ersten verfügbaren Umleitung des Autostartordners. Verknüpfungsziele werden wiederhergestellt, wenn eine gesicherte `.lnk`-Datei geparst werden kann.

**Untersuchte Beweise:**

- `ProgramData\Microsoft\Windows\Start Menu\Programs\Startup` und `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`.
- Zutreffende Umleitungen `Common Startup` / `Startup` in den Einstellungen Shell Folders und User Shell Folders von Maschine/Benutzer.

**Untersuchungshinweise:** Links und Dateien im Autostartordner können bei der Anmeldung legitime Anwendungen starten. Prüfen Sie neben den Dateinamen auch die Zielargumente. Der gemeinsame Ordner ist ein gemeinsam genutzter Konfigurationsort; er beweist nicht, dass das gestartete Programm ein SYSTEM-Token verwendet hat.

### Initialprogramm der Terminaldienste {#ts_initial_program}

**Check-ID:** `ts_initial_program` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Liest den Terminaldienste-Richtlinienwert `InitialProgram` und die Run-/RunOnce-/RunOnceEx-Konfiguration der Terminal-Server-Installation der Maschine, einschließlich begrenzt verschachtelter RunOnceEx-Abschnitte.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` | `InitialProgram` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Run` | Alle nicht leeren Werte |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Runonce` | Alle nicht leeren Werte |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx` | Alle nicht leeren Werte |

**Untersuchungshinweise:** Bereitstellungen von Remoteanwendungen können diese Einstellungen absichtlich verwenden. Ermitteln Sie die betroffenen Sitzungen und verfolgen Sie die ausgewählten Befehle bis in die Prozessbelege; die Registrierung allein zeigt keine Remoteanmeldung.

### Winlogon AppSetup {#winlogon_appsetup}

**Check-ID:** `winlogon_appsetup` · **ATT&CK laut Katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Was Juntei prüft:** Liest `AppSetup` aus Winlogon und teilt kommagetrennte Einträge in separate Kandidaten auf.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `AppSetup` |

**Untersuchungshinweise:** Vergleichen Sie jede Komponente mit der erwarteten Windows-Setup-/Anmeldesoftware. Ein Listeneintrag ist keine separat beobachtete Ausführung, und seine Relevanz kann von der Windows-Version und dem Bereitstellungszustand abhängen.

### Winlogon GinaDLL {#winlogon_gina_dll}

**Check-ID:** `winlogon_gina_dll` · **ATT&CK laut Katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Was Juntei prüft:** Liest die veraltete Ersetzungseinstellung `GinaDLL` von Winlogon. GINA war ein älteres Erweiterungsmodell für die Windows-Anmeldung.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `GinaDLL` |

**Untersuchungshinweise:** Dies ist vor allem für Systeme vor Vista relevant. Ein übrig gebliebener Schlüssel auf einem neueren System ist kein Beweis für eine wirksame Anmeldeerweiterung; prüfen Sie die Version der Quelle und jede referenzierte DLL.

**Weitere Informationen:** [Microsoft-Dokumentation](https://learn.microsoft.com/en-us/windows/win32/secauthn/gina).

### Winlogon MPNotify {#winlogon_mpnotify}

**Check-ID:** `winlogon_mpnotify` · **ATT&CK laut Katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Was Juntei prüft:** Liest die Befehls-/Bibliothekseinstellung `mpnotify` von Winlogon, eine anmeldebezogene Erweiterungsoberfläche.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `mpnotify` |

**Untersuchungshinweise:** Vergleichen Sie Ziel und Argumente mit einer passenden Windows-Baseline. Prüfen Sie versionsspezifisches Verhalten und korrelieren Sie mit einer nachfolgenden Anmeldung, bevor Sie behaupten, dass Code ausgeführt wurde.

### Winlogon-Benachrichtigungspakete {#winlogon_notify_packages}

**Check-ID:** `winlogon_notify_packages` · **ATT&CK laut Katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Was Juntei prüft:** Liest `DllName` aus jeder unmittelbaren Paketregistrierung unter `Notify` von Winlogon.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Notify\<child>` | `DllName` |

**Untersuchungshinweise:** Benachrichtigungspakete sind ein veralteter Erweiterungsmechanismus. Ermitteln Sie, ob die gesicherte Windows-Version sie lädt, und unterscheiden Sie genehmigte Authentifizierungssoftware von unerklärten Bibliotheken.

### Winlogon Shell {#winlogon_shell}

**Check-ID:** `winlogon_shell` · **ATT&CK laut Katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Was Juntei prüft:** Liest maschinenweite und benutzerspezifische `Shell`-Werte von Winlogon, um die Auswahl der Desktop-Shell offenzulegen.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |

**Untersuchungshinweise:** Explorer ist üblich, Kioske können jedoch eine andere Shell verwenden. Untersuchen Sie hinzugefügte Befehle oder ungewöhnliche Pfade und ermitteln Sie, welche wirksame Benutzerkonfiguration bei der Anmeldung galt.

### Winlogon System {#winlogon_system}

**Check-ID:** `winlogon_system` · **ATT&CK laut Katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Was Juntei prüft:** Liest den Wert `System` von Winlogon und trennt kommagetrennte Einträge in einzelne Kandidaten auf.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `System` |

**Untersuchungshinweise:** Behandeln Sie dies als versionsabhängigen Hinweis auf die Anmeldekonfiguration. Prüfen Sie jede referenzierte Komponente und bestätigen Sie mit Ausführungsbelegen, statt die Liste als Ausführungsverlauf zu behandeln.

### Winlogon Taskman {#winlogon_taskman}

**Check-ID:** `winlogon_taskman` · **ATT&CK laut Katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Was Juntei prüft:** Liest `Taskman`-Werte von Winlogon für Maschine/Benutzer auf einen alternativen Befehl zur Taskverwaltung.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |

**Untersuchungshinweise:** Ermitteln Sie, welches Windows-Verhalten die Einstellung verwenden würde und ob das Ziel erwartet ist. Ein konfigurierter Ersatz ist kein Beweis dafür, dass ein Benutzer diese Funktion aufgerufen hat.

### Winlogon Userinit {#winlogon_userinit}

**Check-ID:** `winlogon_userinit` · **ATT&CK laut Katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Was Juntei prüft:** Liest `Userinit`-Werte von Winlogon für Maschine/Benutzer, teilt kommagetrennte Programme auf und ignoriert leere nachgestellte Elemente.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |

**Untersuchungshinweise:** Ein normaler Eintrag verweist häufig auf `userinit.exe`. Prüfen Sie zusätzliche Einträge und veränderte Pfade. Bestätigen Sie den betroffenen Benutzer und eine spätere Anmeldung; `KeyLastWrite` gehört zum gesamten Schlüssel, nicht speziell zu diesem Wert.

### Winlogon VMApplet {#winlogon_vmapplet}

**Check-ID:** `winlogon_vmapplet` · **ATT&CK laut Katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Was Juntei prüft:** Liest die Befehlseinstellung `VMApplet` von Winlogon und erfasst damit eine ältere Startoberfläche für ein System-Applet.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `VMApplet` |

**Untersuchungshinweise:** Prüfen Sie die Anwendbarkeit auf das Betriebssystem und das erwartete Applet. Ein unerwarteter Befehl verdient eine Untersuchung, belegt aber nicht, dass die auslösende Oberfläche geöffnet wurde.

## Dienste und Systemerweiterungen { #services-and-system-extensions }

Windows-Dienste und erweiterbare Systemkomponenten bieten viele legitime Möglichkeiten, ausführbare Dateien oder DLLs zu laden. Vergleichen Sie das konfigurierte Ziel mit dem erwarteten Produkt, dem Dienstkonto, den Starteinstellungen und dem durch Quellen belegten Installationsverlauf.

### DiagTrack-DLL {#diagtrack_dll}

**Check-ID:** `diagtrack_dll` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest `ImagePath` des Dienstes DiagTrack. Trotz des DLL-bezogenen Namens im Katalog untersucht diese Prüfung den konfigurierten Image-Befehl des Dienstes.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DiagTrack` | `ImagePath` |

**Untersuchungshinweise:** Vergleichen Sie mit der Dienst-Baseline und den allgemeinen Dienstprüfungen. Dieselbe Konfiguration kann mehr als ein Prüfergebnis erzeugen; Kandidatenzahlen sind keine Anzahl unterschiedlicher Angriffe.

### DiagTrack-Listener-DLL {#diagtrack_listener_dll}

**Check-ID:** `diagtrack_listener_dll` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest `FileName` aus der Autologger-Konfiguration DiagTrack-Listener. Dies ist eine ETW-Autologger-Einstellung, kein Beleg dafür, dass eine DLL geladen wurde.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\WMI\Autologger\DiagTrack-Listener` | `FileName` |

**Untersuchungshinweise:** Prüfen Sie den Rohwert und die Windows-Protokollierungskonfiguration. Der Katalog führt dies als DLL-Technik, aber die Abfrage validiert weder eine ladbare DLL noch beweist sie eine Ausführung über diesen Wert.

### Netsh-Hilfs-DLL {#netsh_helper}

**Check-ID:** `netsh_helper` · **ATT&CK laut Katalog:** [T1546.007](https://attack.mitre.org/techniques/T1546/007/)

**Was Juntei prüft:** Zählt nicht leere Werte unter dem Registrierungsschlüssel für `NetSh`-Hilfsprogramme der Maschine auf. Hilfsprogramme können `netsh.exe` um DLL-basierte Befehle erweitern.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\NetSh` | Alle nicht leeren Werte |

**Untersuchungshinweise:** Netzwerkprodukte können Hilfsprogramme installieren. Ordnen Sie jeden Wert einem erwarteten Produkt zu, prüfen Sie die aufgelöste Bibliothek und suchen Sie dann nach relevanten netsh-Ausführungen oder Image-Ladevorgängen.

### Druckmonitore {#print_monitors}

**Check-ID:** `print_monitors` · **ATT&CK laut Katalog:** [T1547.010](https://attack.mitre.org/techniques/T1547/010/)

**Was Juntei prüft:** Liest `Driver` unterhalb jeder unmittelbaren Druckmonitor-Registrierung im aktiven Control Set.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Monitors\<child>` | `Driver` |

**Untersuchungshinweise:** Drucksoftware und Portmonitore erweitern den Spooler legitimerweise. Vergleichen Sie DLL und Produktmetadaten mit dem Druckerinventar und bestätigen Sie das Laden mit Spooler-/Image-Ladebelegen.

### Druckprozessoren {#print_processors}

**Check-ID:** `print_processors` · **ATT&CK laut Katalog:** [T1547.012](https://attack.mitre.org/techniques/T1547/012/)

**Was Juntei prüft:** Liest `Driver` unter den Registrierungsbäumen der Druckprozessoren für x64 und x86.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows x64\Print Processors\<child>` | `Driver` |
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows NT x86\Print Processors\<child>` | `Driver` |

**Untersuchungshinweise:** Drucker-Suiten von Drittanbietern können über die bekannten Windows-Standards hinaus Prozessoren hinzufügen. Prüfen Sie beide Architekturen, die tatsächliche DLL und die installierte Druckersoftware, bevor Sie eine böswillige Absicht unterstellen.

### Erweiterung des Dienststeuerungs-Managers {#scm_extension}

**Check-ID:** `scm_extension` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Zählt nicht leere Werte in `ServiceControlManagerExtension` auf, einer Erweiterungsoberfläche des Dienst-Managers.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\ServiceControlManagerExtension` | Alle nicht leeren Werte |

**Untersuchungshinweise:** Untersuchen Sie die genannten Bibliotheken und ob dieser Windows-Build die Einstellung verwendet. Korrelieren Sie mit Boot-/Dienstaktivität; die Prüfung inventarisiert Registrierungen, statt eine geladene Erweiterung zu beweisen.

### Plugin-DLL auf DNS-Serverebene {#server_level_plugin_dll}

**Check-ID:** `server_level_plugin_dll` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest die Einstellung `ServerLevelPluginDll` des DNS-Dienstes im aktiven SYSTEM-Control-Set.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DNS\Parameters` | `ServerLevelPluginDll` |

**Untersuchungshinweise:** Dies ist für Systeme mit der Rolle DNS-Server relevant, darunter einige Domänencontroller. Untersuchen Sie die DLL, das Dienstkonto und Belege für Neustart/Laden des Dienstes; eine inaktive oder fehlende Rolle ändert die Schlussfolgerung.

### Befehl bei Dienstfehler {#service_failure_command}

**Check-ID:** `service_failure_command` · **ATT&CK laut Katalog:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Was Juntei prüft:** Liest `FailureCommand` jedes Dienstes im aktiven Control Set und legt so einen konfigurierten Wiederherstellungsbefehl offen.

**Untersuchte Beweise:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\FailureCommand`.

**Untersuchungshinweise:** Der Befehl wird nur ausgeführt, wenn die Wiederherstellungsaktionen des Dienstes ihn auswählen und der entsprechende Fehler auftritt. Prüfen Sie die Wiederherstellungskonfiguration sowie Fehler-/Prozessdatensätze; der Wert allein belegt keine aktive Wiederherstellungsaktion.

### SNMP-Erweiterungsagent-DLL {#snmp_extension_agent}

**Check-ID:** `snmp_extension_agent` · **ATT&CK laut Katalog:** [T1574](https://attack.mitre.org/techniques/T1574/)

**Was Juntei prüft:** Zählt Werte unter dem Registrierungsschlüssel `ExtensionAgents` des SNMP-Dienstes auf. Werte können statt eines direkten DLL-Pfads einen anderen Registry-Speicherort bezeichnen.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\SNMP\Parameters\ExtensionAgents` | Alle nicht leeren Werte |

**Untersuchungshinweise:** Verfolgen Sie eine indirekte Registrierung manuell bis zu ihrer Bibliothek und überprüfen Sie, ob SNMP installiert ist und verwendet wird. Eine nicht aufgelöste Zeichenfolge in dieser Ausgabe ist kein Beweis, dass die DLL fehlt oder bösartig ist.

### Zeitanbieter {#time_providers}

**Check-ID:** `time_providers` · **ATT&CK laut Katalog:** [T1547.003](https://attack.mitre.org/techniques/T1547/003/)

**Was Juntei prüft:** Liest `DllName` aus unmittelbaren `TimeProviders`-Registrierungen des Windows-Zeitdienstes.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\W32Time\TimeProviders\<child>` | `DllName` |

**Untersuchungshinweise:** Prüfen Sie die Aktivierungseinstellungen der Anbieter und den Zustand des Windows-Zeitdienstes gesondert; die Prüfung des DLL-Pfads setzt keinen aktivierten Anbieter voraus. Produkte zur Zeitsynchronisation können legitime Ersatzkomponenten liefern.

### Windows-Dienst (ServiceDll) {#windows_service_dll}

**Check-ID:** `windows_service_dll` · **ATT&CK laut Katalog:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Was Juntei prüft:** Liest `Parameters\ServiceDll` unterhalb der Dienste im aktiven SYSTEM-Control-Set und fügt, sofern verfügbar, dienstspezifische Zeitstempelbelege hinzu.

**Untersuchte Beweise:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Parameters\ServiceDll`.

**Untersuchungshinweise:** Ein gemeinsam genutzter Host wie `svchost.exe` ist nicht die Payload-DLL. Prüfen Sie die Bibliothek, das Dienstkonto, den Startzustand und passende Belege zur Dienstinstallation bzw. zum Laden des Images; eine Registrierung beweist nicht, dass der Dienst ausgeführt wurde.

**Weitere Informationen:** [Microsoft-Dokumentation](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### Windows-Dienst (ImagePath) {#windows_service_image_path}

**Check-ID:** `windows_service_image_path` · **ATT&CK laut Katalog:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Was Juntei prüft:** Liest `ImagePath`-Befehle der Dienste im aktiven Control Set. Treibertypen 1, 2 oder 8 ohne ImagePath behalten ein implizites Ziel `System32\drivers\<service>.sys`.

**Untersuchte Beweise:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>`: `ImagePath`, `Type` und ergänzende Dienstmetadaten.

**Untersuchungshinweise:** Prüfen Sie Dienstkonto und Starteinstellungen getrennt von der Bezeichnung des Maschinen-Geltungsbereichs in der Ausgabe. Deaktivierte Registrierungen oder Registrierungen mit Start bei Bedarf gehören dennoch zum Inventar. Für Zeitstempel werden Belege anhand des Dienstnamens gegenüber mehrdeutigen, gemeinsam genutzten Pfaden ausführbarer Dateien bevorzugt.

**Weitere Informationen:** [Microsoft-Dokumentation](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### Start-DLL des Windows Update-Dienstes {#wu_service_startup_dll}

**Check-ID:** `wu_service_startup_dll` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest `Parameters\ServiceDll` des Windows Update-Dienstes aus dem aktiven Control Set.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\wuauserv\Parameters` | `ServiceDll` |

**Untersuchungshinweise:** Vergleichen Sie die Bibliothek mit demselben Windows-Build und dem installierten Wartungsstand. Dies überschneidet sich mit dem generischen ServiceDll-Inventar und stellt keine zweite, eigenständige Kompromittierung dar.

## Geplante Aufgaben { #scheduled-tasks }

Die Aufgabenplanung speichert die Konfiguration sowohl in XML-Dateien als auch im SOFTWARE-Hive. Lesen Sie diese Quellen gemeinsam. Eine Aufgabe kann registriert sein, ohne ausgeführt worden zu sein, und eine unvollständige Sammlung kann einer versteckten oder Geisteraufgabe ähneln.

### Geisteraufgabe {#ghost_task}

**Check-ID:** `ghost_task` · **ATT&CK laut Katalog:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Was Juntei prüft:** Vergleicht die Registrierungen `Tree` und `Tasks` im TaskCache von SOFTWARE und sucht, wenn Aufgabendateien in der Sammlung vorhanden sind, nach den zugehörigen Aufgaben-XML-Dateien.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree` und `TaskCache\Tasks`.
- Gesicherte Dateien `Windows\System32\Tasks\<task path>` für den Dateisystemvergleich.

**Untersuchungshinweise:** Fehlende Gegenstücke können auf gelöschte/veraltete Aufgaben, integrierte Registrierungen oder eine unvollständige Sammlung zurückgehen. Ohne gesicherte XML-Dateien ist der Dateisystemvergleich nicht verfügbar. Prüfen Sie die Abdeckung und die genaue Abweichung, bevor Sie eine Aufgabe als versteckt oder bösartig bezeichnen.

### Versteckte geplante Aufgabe {#hidden_scheduled_task}

**Check-ID:** `hidden_scheduled_task` · **ATT&CK laut Katalog:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Was Juntei prüft:** Prüft TaskCache-Tree-Registrierungen auf einen fehlenden `SD`-Wert oder eine leere binäre Sicherheitsbeschreibung – Bedingungen, die damit verbunden sind, eine Aufgabe vor der gewöhnlichen Aufzählung zu verbergen.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\<task>`: Vorhandensein/Daten von `SD`, mit Metadaten des TaskCache-Gegenstücks.

**Untersuchungshinweise:** Ein fehlender/leerer SD ist eine spezifische Konfigurationsanomalie, kein Beweis, dass die Aufgabe ausgeführt wurde oder bösartig ist. Die XML-Einstellung `Hidden` und beliebige ACL-Probleme sind andere Bedingungen und nicht das hier verwendete Kriterium.

**Weitere Informationen:** [Microsoft-Dokumentation](https://www.microsoft.com/en-us/security/blog/2022/04/12/tarrask-malware-uses-scheduled-task-defense-evasion/).

### Geplante Aufgabe (XML-Dateien) {#scheduled_task_files}

**Check-ID:** `scheduled_task_files` · **ATT&CK laut Katalog:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Was Juntei prüft:** Wertet gesicherte Aufgaben-XML auf Exec-Befehle/-Argumente sowie Klassen-IDs/Daten von COM-Handlern aus. Es wählt den Prinzipal anhand von `Actions Context`/`Principal id`, behält Benutzer- und Gruppenidentität getrennt bei und meldet SYSTEM, USER oder UNKNOWN. Nicht aufgelöste COM-Aktionen bleiben sichtbar.

**Untersuchte Beweise:**

- XML-Dateien `Windows\System32\Tasks\<task path>`, einschließlich Exec, ComHandler und verfügbarer Prinzipalfelder.
- Gesicherte `UsrClass.dat` und NTUSER-`Software\Classes` des passenden Benutzers sowie `Classes\CLSID` im SOFTWARE-Hive der Maschine, in der nativen und der 32-Bit-Ansicht.
- Gesicherte `ProfileList` aus SOFTWARE für SID-Prinzipale. Benannte Prinzipale bevorzugen einen eindeutig passenden Ordner `user.DOMAIN`, dann gesicherte Suffixe des lokalen Hosts und schließlich den einfachen Ordner `user`. Übereinstimmungen werden ausdrücklich als Heuristik gekennzeichnet; echte Mehrdeutigkeit bleibt unaufgelöst und wird im Ergebnis und in der Abdeckung vermerkt. Flüchtige Registry-Schlüssel sind nicht erforderlich.
- Verwaltete `CodeBase` (einschließlich Versionsunterschlüsseln), AppID-Zuordnungen und `ImagePath` des SYSTEM-Dienstes für einen AppID-`LocalService`.

COM-Server-Kandidaten enthalten ihre Registry-Quelle in `Description`. AppID-Zuordnungen, die nur
aus dem Namen einer ausführbaren Datei bestehen, bleiben unaufgelöst, statt ihnen ein geratenes Verzeichnis zuzuweisen. Lokale
`file:`-URLs werden dekodiert; Remote-URLs werden ohne Netzwerkzugriff beibehalten. Benutzerspezifische Klassenüberschreibungen
gelten nur für das jeweilige Profil, und eine unvollständige Benutzer-CLSID fällt nicht auf die von ihr verdeckte
Maschinenregistrierung zurück. Aufgaben, die nur für Gruppen gelten, behalten Maschinenkandidaten mit dem Hinweis auf die Einschränkung
bezüglich einzelner Benutzer in ihrer Beschreibung, ohne der Fehlerzusammenfassung der Abdeckung Routinehinweise hinzuzufügen.

**Untersuchungshinweise:** Prüfen Sie Auslöser, Aktivierungszustand, Prinzipal und Ausführungsverlauf in der Quell-XML und den Ereignisprotokollen. Eine erstellte Aufgabe wird möglicherweise nie ausgeführt. Ein nicht aufgelöster COM-Handler ist dennoch ein Kandidat, und fehlende oder nur gruppenbezogene Prinzipaldaten dürfen nicht als SYSTEM gedeutet werden.

## WMI, BITS und andere Ereignisauslöser { #wmi-bits-and-other-event-triggers }

Windows Management Instrumentation (WMI) unterstützt die Systemverwaltung. Der Background Intelligent Transfer Service (BITS) unterstützt Hintergrundübertragungen. Beide können ereignisgesteuerte Befehle speichern. Einige Mechanismen warten auf ein Verwaltungsereignis, eine Übertragungsbenachrichtigung, einen Anwendungsstart oder eine Benutzeraktion. Ein gespeicherter Auslöser ist ein nützlicher Konfigurationsbeleg, sein Vorhandensein belegt jedoch nicht, dass der Auslöser ausgelöst wurde.

### Hintertür über Bedienungshilfen {#accessibility_tools}

**Check-ID:** `accessibility_tools` · **ATT&CK laut Katalog:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Was Juntei prüft:** Vergleicht sieben gesicherte Binärdateien der Bedienungshilfen in System32 anhand von PE-OriginalFilename und Zertifikats-/Katalogmetadaten, nachdem verwertbare Microsoft-Katalogmetadaten aus einer gesicherten Referenzbinärdatei vorausgesetzt wurden.

**Untersuchte Beweise:**

- `Windows\System32\sethc.exe`, `osk.exe`, `narrator.exe`, `magnify.exe`, `utilman.exe`, `atbroker.exe` und `DisplaySwitch.exe`.
- Gesicherte Referenzmetadaten von `Windows\System32\winlogon.exe`, `cmd.exe` oder `notepad.exe` und zutreffende gesicherte Kataloge.

**Untersuchungshinweise:** Metadaten können auf einen umbenannten Ersatz hindeuten, aber Juntei validiert weder Signaturen noch Zertifikatsvertrauen. Eine lückenhafte Sammlung ohne die benötigten Kataloge führt zu einer unvollständigen Vergleichsabdeckung, statt jede scheinbar unsignierte Datei als Hintertür zu behandeln.

### Bedienungshilfen {#assistive_technology}

**Check-ID:** `assistive_technology` · **ATT&CK laut Katalog:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Was Juntei prüft:** Liest registrierte `StartExe`- und `StartParams`-Befehle der Bedienungshilfen in nativen/32-Bit-Ansichten von SOFTWARE sowie aktivierte `Configuration`-Einträge der Bedienungshilfen aus Benutzer- und DEFAULT-Hives. Numerische bzw. nicht als Zeichenfolge vorliegende StartExe-Platzhalter werden übersprungen.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\<feature>`: `StartExe` und `StartParams`; außerdem die `Wow6432Node`-Ansicht der Maschine.
- `HKU\<user>\Software\Microsoft\Windows NT\CurrentVersion\Accessibility\Configuration` in NTUSER.DAT und die entsprechende Einstellung im DEFAULT-Hive.

**Untersuchungshinweise:** Software für Bedienungshilfen ist legitim. Unterscheiden Sie den Namen einer aktivierten Funktion von ihrer tatsächlichen Registrierung der ausführbaren Datei und prüfen Sie dann das Ziel sowie Belege für eine Ausführung bei Anmeldung/auf dem Desktop. Dies ist vom Ersetzen der ausführbaren Datei der Bedienungshilfe selbst zu unterscheiden.

**Weitere Informationen:** [Technikreferenz (hexacorn.com)](https://www.hexacorn.com/blog/2016/07/22/beyond-good-ol-run-key-part-42/).

### Benachrichtigungsbefehl eines BITS-Jobs {#bits_notify_command}

**Check-ID:** `bits_notify_command` · **ATT&CK laut Katalog:** [T1197](https://attack.mitre.org/techniques/T1197/)

**Was Juntei prüft:** Stellt BITS-Jobdatensätze mit längenpräfixierten Zeichenfolgen aus gesicherten Dateien `qmgr.db`, `qmgr0.dat` oder `qmgr1.dat` wieder her und behält nicht leere Benachrichtigungsprogramme, Parameter, Job-IDs und Besitzer-SIDs bei.

**Untersuchte Beweise:**

- `ProgramData\Microsoft\Network\Downloader\qmgr.db`, `qmgr0.dat` und `qmgr1.dat`.

**Untersuchungshinweise:** Dies sind per Carving gewonnene Datensätze: Der zugewiesene/aktuelle Jobzustand wird nicht validiert, und veraltete/gelöschte Jobs können erscheinen. Die MFT-Schreibzeit der Datenbank ist ein schwacher Beleg für den Container, nicht die Ausführungszeit der Benachrichtigung. Besitzer-SIDs unterscheiden SYSTEM, lokalen Dienst, Netzwerkdienst und Benutzer.

### AutoRun des Befehlsprozessors {#cmd_autorun}

**Check-ID:** `cmd_autorun` · **ATT&CK laut Katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Was Juntei prüft:** Liest `AutoRun`-Befehle des Command Processor für Maschine/Benutzer, eine Einstellung, die den Start der Eingabeaufforderung beeinflussen kann.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |
| `HKU\<user>\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |

**Untersuchungshinweise:** Entwicklungswerkzeuge und Shell-Anpassungen können einen Wert erklären. `cmd.exe /d` umgeht AutoRun; die Konfiguration allein beweist daher nicht, dass es bei jedem Auftreten von cmd.exe ausgeführt wurde. Prüfen Sie den Befehlsinhalt und die Argumente des auslösenden Prozesses.

**Weitere Informationen:** [Microsoft-Dokumentation](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd).

### Persistenz über ErrorHandler.cmd {#error_handler_cmd}

**Check-ID:** `error_handler_cmd` · **ATT&CK laut Katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Was Juntei prüft:** Meldet das Vorhandensein des gesicherten Skripts `Windows\System32\ErrorHandler.cmd` als Kandidaten für Persistenz über einen Fehlerhandler.

**Untersuchte Beweise:**

- `Windows\System32\ErrorHandler.cmd`.

**Untersuchungshinweise:** Das Vorhandensein allein beweist nicht, dass der betreffende Windows-Wiederherstellungs-/Fehlerpfad es aufgerufen hat. Lesen Sie das Skript, prüfen Sie seine Herkunft und korrelieren Sie mit Setup-/Fehler-/Prozessbelegen.

### Power Automate Desktop-Flows {#power_automate}

**Check-ID:** `power_automate` · **ATT&CK laut Katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Was Juntei prüft:** Findet gesicherte Flow-/Skriptverzeichnisse unterhalb der Speicher `flows` und `scripts` von Power Automate Desktop jedes Benutzers und meldet jeden gefundenen Flow einmal.

**Untersuchte Beweise:**

- `Users\<user>\AppData\Local\Microsoft\Power Automate Desktop\flows\<flow>\...` und `scripts\<flow>\...`.

**Untersuchungshinweise:** Dies identifiziert gespeicherte Automatisierung, nicht jeden geplanten Auslöser oder einen ausgeführten Flow. Prüfen Sie die Flow-Inhalte und die externe Startkonfiguration, bevor Sie dies als automatische oder bösartige Persistenz behandeln.

### PowerShell-Profil {#powershell_profiles}

**Check-ID:** `powershell_profiles` · **ATT&CK laut Katalog:** [T1546.013](https://attack.mitre.org/techniques/T1546/013/)

**Was Juntei prüft:** Findet Dateien nach dem Muster profile.ps1 in bekannten Installations-/Profilverzeichnissen von Windows PowerShell und PowerShell, einschließlich der Dokumente des Benutzers, gefundener OneDrive-Dokumentordner und gesicherter Umleitungen des Dokumentordners.

**Untersuchte Beweise:**

- Dateien, die auf `profile.ps1` enden, direkt in `Windows\System32\WindowsPowerShell\v1.0` und `Windows\SysWOW64\WindowsPowerShell\v1.0`.
- Benutzerspezifische `Documents\WindowsPowerShell` und `Documents\PowerShell`, einschließlich gesicherter Umleitungen des Dokumentordners und gefundener OneDrive-Dokumentordner.
- Dateien, die auf `profile.ps1` enden, direkt in Versionsverzeichnissen unter `Program Files\PowerShell` oder `Program Files (x86)\PowerShell`.

**Untersuchungshinweise:** Profile können harmlose Anpassungen enthalten. Lesen Sie das Skript und verfolgen Sie aufgerufene Payloads. Ein Start von PowerShell mit `-NoProfile` überspringt sie; das Vorhandensein der Datei allein beweist weder eine Ausführung des Profils noch bösartigen Inhalt.

### Bildschirmschoner-Hijacking {#screensaver}

**Check-ID:** `screensaver` · **ATT&CK laut Katalog:** [T1546.002](https://attack.mitre.org/techniques/T1546/002/)

**Was Juntei prüft:** Liest den Desktop-Wert `SCRNSAVE.EXE` jedes Benutzers und identifiziert so das konfigurierte Bildschirmschoner-Image.

**Untersuchte Beweise:**

- `HKU\<user>\Control Panel\Desktop\SCRNSAVE.EXE` in NTUSER.DAT.

**Untersuchungshinweise:** Vergleichen Sie mit genehmigten Bildschirmschonern und prüfen Sie das Ziel. Weitere Desktop-Einstellungen, der Sitzungszustand und das Zeitlimit bestimmen, ob er ausgelöst werden kann; ein gespeicherter Pfad beweist nicht, dass ein Bildschirmschoner gestartet wurde.

### Befehl des Telemetry Controllers {#telemetry_controller}

**Check-ID:** `telemetry_controller` · **ATT&CK laut Katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Was Juntei prüft:** Liest `Binary`, `Command` und `MaintenanceCommand` aus TelemetryController und seinen unmittelbaren untergeordneten Registrierungen.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController` und unmittelbar untergeordnete Elemente: `Binary`, `Command`, `MaintenanceCommand`.

**Untersuchungshinweise:** Vergleichen Sie die ausgewählte ausführbare Datei und die Argumente mit demselben Windows-Build und der Wartungssoftware. Ein Registry-Befehl zeigt nicht, dass ein Wartungsauslöser ausgelöst wurde.

### Benutzerdefinierte Profile von Windows Terminal {#windows_terminal}

**Check-ID:** `windows_terminal` · **ATT&CK laut Katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Was Juntei prüft:** Wertet JSON-/JSONC-Einstellungen von gepackten, Vorschau- und nicht gepackten Windows Terminal-Installationen aus und meldet nicht leere `commandline`-Einstellungen von Profilen/Standardwerten.

**Untersuchte Beweise:**

- `Users\<user>\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json` und das Gegenstück von WindowsTerminalPreview.
- `Users\<user>\AppData\Local\Microsoft\Windows Terminal\settings.json`: commandline-Felder in älteren Profil-Arrays, `profiles.list` und `profiles.defaults`.

**Untersuchungshinweise:** Gewöhnliche Shells erscheinen hier naturgemäß. Ermitteln Sie das ausgewählte/Standardprofil und prüfen Sie ungewöhnliche Befehle oder Startargumente. Das bloße Speichern eines Profils startet es nicht.

### WMI-Ereignisabonnement {#wmi_event_subscription}

**Check-ID:** `wmi_event_subscription` · **ATT&CK laut Katalog:** [T1546.003](https://attack.mitre.org/techniques/T1546/003/)

**Was Juntei prüft:** Gewinnt per Carving erkannte Ereignis-Consumer-Instanzen und Verweise auf Filter-zu-Consumer-Bindungen aus dem gesicherten WMI-Repository OBJECTS.DATA. Zu den erkannten Klassen gehören Befehlszeilen-, Active-Script-, Logdatei-, Ereignisprotokoll- und SMTP-Consumer.

**Untersuchte Beweise:**

- `Windows\System32\wbem\Repository\OBJECTS.DATA` und `Repository\FS\OBJECTS.DATA`.

**Untersuchungshinweise:** Ein roher Treffer im Repository kann veraltet, ungebunden oder historisch sein; die Ausgabe bewahrt diese Einschränkung. Carving rekonstruiert weder einen vollständigen aktiven Abonnementgraphen noch beweist es, dass der Auslöser ausgelöst wurde. Die Schreibzeit der Repository-Datei ist kein Erstellungs- oder Ausführungszeitpunkt eines Abonnements.

## Explorer, COM und Datei- oder URL-Handler { #explorer-com-and-file-or-url-handlers }

Das Component Object Model (COM) ermöglicht es Anwendungen, registrierte Softwarekomponenten zu aktivieren. Die Windows-Shell verwendet registrierte Handler, um Dateien, URLs, Ordner und spezielle Objekte zu öffnen. COM-Kennungen (CLSIDs) identifizieren Komponenten; ProgIDs geben einigen Komponenten lesbare Namen. Verfolgen Sie jede Registrierung bis zu dem Befehl oder der DLL, die sie auswählt.

### App-Paths-Hijacking {#app_paths}

**Check-ID:** `app_paths` · **ATT&CK laut Katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Was Juntei prüft:** Liest Standardwerte unterhalb maschinenweiter `App Paths`-Registrierungen, die die Anwendungssuche bei Starts über die Shell umleiten können.

**Untersuchte Beweise:**

- Standardwert von `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\<application>`.

**Untersuchungshinweise:** Eine App-Paths-Registrierung ist bei installierten Anwendungen üblich und keine allgemeine Umschreibung jedes Prozessstarts. Vergleichen Sie den registrierten Namen und das Image mit der erwarteten Software und untersuchen Sie dann relevante Shell-/Prozessbelege.

### COM-Objekt-Hijacking {#com_treat_as}

**Check-ID:** `com_treat_as` · **ATT&CK laut Katalog:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Was Juntei prüft:** Findet COM-`TreatAs`-Umleitungen und benutzerspezifische InprocServer32-/LocalServer32-Registrierungen, die eine Maschinen-CLSID verdecken, über die unterstützten nativen/32-Bit-Klassenspeicher hinweg. TreatAs-Ziele werden mit den Klassen desselben Benutzers und den Maschinenklassen aufgelöst.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Classes\CLSID\<CLSID>` und die Ansicht `Classes\Wow6432Node` der Maschine.
- Benutzerspezifische `Software\Classes\CLSID` in NTUSER.DAT und `CLSID` in UsrClass.dat, einschließlich ihrer implementierten `Wow6432Node`-Ansichten: Standardwerte von `TreatAs`, `InprocServer32` und `LocalServer32`.

**Untersuchungshinweise:** Benutzerspezifische Überschreibungen erfordern unter Umständen keine Administratorrechte, aber auch legitime Software kann sie registrieren. Verfolgen Sie die ausgewählte Klasse und DLL/ausführbare Datei, bewerten Sie Architektur und Aktivierungskontext und bestätigen Sie die tatsächliche COM-Aktivierung unabhängig.

### Hijacking von Datenträgerbereinigungs-Handlern {#disk_cleanup_handler}

**Check-ID:** `disk_cleanup_handler` · **ATT&CK laut Katalog:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Was Juntei prüft:** Liest Handler-CLSIDs der Datenträgerbereinigung unter VolumeCaches und folgt auflösbaren COM-InprocServer32-Registrierungen der Maschine bis zu ihren Bibliotheken.

**Untersuchte Beweise:**

- Standard-CLSID von `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\<handler>` und `Classes\CLSID\<CLSID>\InprocServer32`.

**Untersuchungshinweise:** Damit der Handler ausgeführt wird, muss er vom entsprechenden Bereinigungsablauf aufgerufen werden. Nicht aufgelöste Handler werden von dieser Prüfung nicht ausgegeben; verwenden Sie daher den Registry-Parser, um ihre Rohregistrierungen zu prüfen, wenn die Sicherung unvollständig ist.

### Überschreibung von Explorer-AppKeys {#explorer_app_key}

**Check-ID:** `explorer_app_key` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Liest die Werte `ShellExecute` und `Association` unterhalb unmittelbarer untergeordneter Elemente von `AppKey` des Explorers und erfasst so durch Anwendungstasten ausgewählte Befehle/Aktionen.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\AppKey\<key>`: `ShellExecute` und `Association`.

**Untersuchungshinweise:** Tastatur-Dienstprogramme und OEM-Werkzeuge können diese Werte definieren. Ein Zuordnungsname kann einen weiteren Schritt in der Registry erfordern, statt ein Dateipfad zu sein. Prüfen Sie die Benutzeraktion, die ihn auslösen würde.

### Browserhilfsobjekte {#explorer_bho}

**Check-ID:** `explorer_bho` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Liest CLSID-Registrierungen von Browserhilfsobjekten und löst, sofern verfügbar, die InprocServer32-Bibliotheken der Maschine auf; nicht aufgelöste Klassen-IDs bleiben sichtbar.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects\<CLSID>` und der Standardwert von `Classes\CLSID\<CLSID>\InprocServer32` der Maschine.

**Untersuchungshinweise:** Dies ist an das Verhalten kompatibler Browser/des Explorers gebunden und besonders für ältere Umgebungen relevant. Genehmigte Browser-/Sicherheitserweiterungen können erscheinen. Ein registriertes BHO ist kein Beweis, dass ein aktueller Browser es geladen hat.

### Hijacking von Explorer-Desktop-CLSIDs {#explorer_clsid_hijack}

**Check-ID:** `explorer_clsid_hijack` · **ATT&CK laut Katalog:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Was Juntei prüft:** Liest konfigurierte Shell-Öffnen-Befehle für die drei aufgeführten speziellen Explorer-Namespace-CLSIDs aus den Klassenspeichern von Maschine/Benutzer.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |

**Untersuchungshinweise:** Vergleichen Sie die Befehlshandler für Desktop, Arbeitsplatz und Eigene Dateien mit einer passenden Baseline. Dies prüft ausgewählte CLSIDs und Pfade, nicht jedes mögliche Hijacking eines Explorer-Namespaces.

### Kontextmenü-Handler des Explorers {#explorer_context_menu}

**Check-ID:** `explorer_context_menu` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Zählt Kontextmenü-Handler für Directory, Directory Background, Folder, Drive, AllFilesystemObjects und `*` auf und folgt Klassen-IDs nach Möglichkeit zu gesicherten DLL-Registrierungen.

**Untersuchte Beweise:**

- Klassenspeicher `HKLM\SOFTWARE\Classes` der Maschine und UsrClass.dat des Benutzers: Standardwert von `<subject>\shellex\ContextMenuHandlers\<handler>` und die zugehörigen COM-Bibliotheksregistrierungen.

**Untersuchungshinweise:** Rechtsklick-Erweiterungen von Archivwerkzeugen, Sicherheitsprodukten und Cloud-Clients sind häufig. Prüfen Sie den tatsächlichen Handler und die auslösende Interaktion. Nicht aufgelöste GUIDs bleiben nützliche Ansatzpunkte, statt zu beweisen, dass die DLL nicht existierte.

### Hijacking von Dateizuordnungen {#file_association_hijack}

**Check-ID:** `file_association_hijack` · **ATT&CK laut Katalog:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Was Juntei prüft:** Untersucht `shell\open\command`-Registrierungen direkter Erweiterungen und aufgelöster ProgIDs für `.txt`, `.pdf`, `.doc`, `.docx`, `.html`, `.htm`, `.js`, `.vbs`, `.hta`, `.exe`, `.bat`, `.cmd` und `.ps1`.

**Untersuchte Beweise:**

- Klassenspeicher `HKLM\SOFTWARE\Classes` der Maschine und UsrClass.dat des Benutzers: Standardwerte der ausgewählten `<extension>` und Standardwerte von `<extension-or-ProgID>\shell\open\command`.

**Untersuchungshinweise:** Vergleichen Sie mit der erwarteten Anwendung und prüfen Sie die Befehlsargumente. Dies ist eine festgelegte Menge von Erweiterungen, nicht alle Dateitypen und nicht die vollständige wirksame UserChoice-Auflösung. Ein gespeicherter Befehl beweist nicht, dass ein Dokument geöffnet wurde.

### Hijacking von Protokollhandlern {#protocol_handler_hijack}

**Check-ID:** `protocol_handler_hijack` · **ATT&CK laut Katalog:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Was Juntei prüft:** Liest URL-Öffnen-Befehle für http, https, mailto, ms-msdt und ms-officecmd sowie gefundene `URL Protocol`-Registrierungen und zutreffende Benutzerüberschreibungen.

**Untersuchte Beweise:**

- Klassenspeicher `HKLM\SOFTWARE\Classes` der Maschine und UsrClass.dat des Benutzers: `URL Protocol`-Markierungen und Standardwerte von `<scheme>\shell\open\command`.

**Untersuchungshinweise:** Browser und Anwendungsinstallationsprogramme registrieren Schemata legitimerweise. Verfolgen Sie den Befehl und die Argumente, bewerten Sie den erwarteten Zweck des Schemas und korrelieren Sie mit Belegen zum Öffnen von URLs bzw. zu Prozessen.

### COM-Erweiterungshandler des Papierkorbs {#recycle_bin_com_extension}

**Check-ID:** `recycle_bin_com_extension` · **ATT&CK laut Katalog:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Was Juntei prüft:** Liest die aufgeführten Befehlsüberschreibungen für Öffnen/Leeren/Durchsuchen der Papierkorb-CLSID sowie unmittelbare Registrierungen von Kontextmenü- und Drag-and-Drop-Handlern.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\empty\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\explore\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\ContextMenuHandlers\<child>` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\DragDropHandlers\<child>` | `(Default)` |

**Untersuchungshinweise:** Vergleichen Sie mit genehmigten Shell-Erweiterungen. Einige gemeldete Werte sind COM-Kennungen statt einer direkt aufgelösten Bibliothek; untersuchen Sie die referenzierte Klasse und die Benutzeraktion, bevor Sie eine Codeausführung behaupten.

### Hijacking des Suchprotokollhandlers {#search_protocol_handler}

**Check-ID:** `search_protocol_handler` · **ATT&CK laut Katalog:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Was Juntei prüft:** Liest den Standardhandler `search-ms\shell\open\command` von Maschine/Benutzer.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Classes\search-ms\shell\open\command` | `(Default)` |
| `HKU\<user>\Software\Classes\search-ms\shell\open\command` | `(Default)` |

**Untersuchungshinweise:** Eine Änderung des Handlers kann verändern, was beim Öffnen einer search-ms-URL geschieht, aber ihr Vorhandensein ist kein Beweis für die Auslieferung eines Exploits. Vergleichen Sie mit den Windows-Standardwerten und untersuchen Sie die URL-/Prozessaktivität, die ihn verwenden würde.

### SharedTaskScheduler {#shared_task_scheduler}

**Check-ID:** `shared_task_scheduler` · **ATT&CK laut Katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Was Juntei prüft:** Zählt nicht leere `SharedTaskScheduler`-Werte des Explorers als registrierte Shell-Komponenten auf.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SharedTaskScheduler` | Alle nicht leeren Werte |

**Untersuchungshinweise:** Werte können auf COM-Klassen statt auf einen Pfad einer ausführbaren Datei verweisen. Verfolgen Sie ihre Serverregistrierungen und prüfen Sie das Verhalten des Betriebssystems, die Herkunft der Erweiterung und relevante Belege für das Laden durch die Shell.

### ShellExecuteHooks {#shell_execute_hooks}

**Check-ID:** `shell_execute_hooks` · **ATT&CK laut Katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Was Juntei prüft:** Zählt nicht leere `ShellExecuteHooks`-Werte des Explorers auf und erfasst damit veraltete Hook-Registrierungen für die Shell-Ausführung.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ShellExecuteHooks` | Alle nicht leeren Werte |

**Untersuchungshinweise:** Lösen Sie jede referenzierte CLSID auf und überprüfen Sie, ob Windows-Build und Richtlinie den Hook zulassen. Eine veraltete Registrierung kann ungenutzt sein; der Wert selbst ist keine beobachtete Shell-Ausführung.

### COM-Persistenz über Startmenü-Bibliotheken und CLSID-Ordner {#start_menu_com}

**Check-ID:** `start_menu_com` · **ATT&CK laut Katalog:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Was Juntei prüft:** Findet `.library-ms`-Bibliotheks-XML-Dateien, deren `simpleLocation`-URL auf
eine COM-CLSID verweist, sowie Verzeichnisse mit dem Namen `<name>.{CLSID}` unterhalb von
`Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu`. Eine Interaktion des Explorers mit diesen
Objekten kann COM-Code aktivieren. Die Prüfung schließt leere gesicherte Verzeichnisse ein; sie folgt keinen
symbolischen Links. Gewöhnliche Bibliotheksdateien außerhalb des Startmenüs und GUIDs in XML-Kommentaren werden
von dieser Prüfung nicht gemeldet.

**Untersuchte Beweise:**

- Benutzerspezifische `.library-ms`-Dateien im Startmenü und nach CLSIDs benannte Verzeichniseinträge.
- Gesicherte Klassenregistrierungen des passenden Benutzers und SOFTWARE der Maschine, getrennt nach Architektur.
- Verwaltete CodeBase-/AppID-/Dienstkonfiguration über denselben Resolver wie bei geplanten Aufgaben.

**Untersuchungshinweise:** `Path` und `SourceFile` identifizieren das Artefakt, das die Aktivierung auslöst. `Value`
enthält den Kandidaten für den Serverbefehl oder die CLSID, wenn keine Auflösung möglich ist. `Description`
verknüpft den Auslöser mit der CLSID und der gesicherten Registry-Quelle. `ResolvedPath`/`Sha256` beschreiben nur
eine aufgelöste, gesicherte Payload. Eine AppID-Zuordnung, die nur aus dem Namen einer ausführbaren Datei besteht, bleibt unaufgelöst.
Ein Auslöser kann legitim sein; seine Existenz beweist weder eine Anmeldung noch eine Aktivierung, Ausführung oder einen Angriff.
Prüfen Sie die Bibliotheks-URL, die Klassenregistrierung, die Payload, den Benutzerkontext und unabhängige Ausführungsprotokolle.

**Hinweise zur Sicherung:** Reine Dateisammlungen können leere Verzeichnisse auslassen. Die Standardpfade des Startmenüs
des Benutzers werden untersucht; umgeleitete Startmenüs und andere Explorer-Namespace-Speicherorte werden
von dieser Prüfung nicht abgedeckt. Fehlende Hives oder nicht aufgelöste Registrierungen führen zu teilweiser Abdeckung, wobei
der Kandidat erhalten bleibt. Aufgabenarchitektur, Rechteerhöhung und Richtlinien können das wirksame COM-Suchverhalten ändern.

**Weitere Informationen:** [Ursprünglicher Vorschlag und Nennung des Forschers](https://github.com/last-byte/PersistenceSniper/pull/32),
[Zusammengeführte COM-Ansichten von Benutzer/Maschine](https://learn.microsoft.com/en-us/windows/win32/sysinfo/merged-view-of-hkey-classes-root),
[MITRE: COM-Hijacking](https://attack.mitre.org/techniques/T1546/015/).

### TypeLib-COM-Hijacking {#typelib_hijack}

**Check-ID:** `typelib_hijack` · **ATT&CK laut Katalog:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Was Juntei prüft:** Liest benutzerspezifische TypeLib-Registrierungen (GUID/Version/Gebietsschema, win32/win64) aus UsrClass.dat und behält Pfade und Skript-Moniker-Werte bei. Numerische nachgestellte Ressourcenindizes werden nur für die Auflösung der Zieldatei entfernt.

**Untersuchte Beweise:**

- Standardwerte von `TypeLib\<GUID>\<version>\<locale>\win32` und `win64` in jeder gesicherten UsrClass.dat.

**Untersuchungshinweise:** Eine Typbibliothek kann auf eine legitime DLL, eine alternative Datei oder einen Skript-Moniker verweisen. Die Prüfung inventarisiert diese Registrierungen; bewerten Sie die betroffene Anwendung und bestätigen Sie das Laden der Typbibliothek, statt anzunehmen, dass jeder vom Standard abweichende Pfad ein Angriff ist.

## Office-Anwendungen und Add-ins { #office-applications-and-add-ins }

Office unterstützt legitime Add-ins, Vorlagen und anwendungsspezifische Erweiterungen. Eine verdächtige Registrierung kann von einer bestimmten Office-Version, Anwendung, Ladeeinstellung oder Benutzerinteraktion abhängen. Die Konfiguration allein belegt keine Makroausführung.

### Registrierung von Office-Add-ins {#office_addins}

**Check-ID:** `office_addins` · **ATT&CK laut Katalog:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Was Juntei prüft:** Zählt Add-ins für Word, Excel, PowerPoint, Outlook und Access in versionslosen Office-Wurzeln sowie in 14.0/15.0/16.0 auf, einschließlich des implementierten Click-to-Run-Speichers der Maschine und der Benutzerwurzeln. Es behält explizite Manifest-/FileName-/Path-Werte bei oder löst geeignete COM-gestützte Einträge auf, jeweils mit LoadBehavior-Kontext.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` und das implementierte Click-to-Run-Gegenstück der Maschine.
- `HKU\<user>\Software\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` in NTUSER.DAT; passende COM-Klassen von Maschine/Benutzer aus SOFTWARE, NTUSER.DAT und UsrClass.dat.

**Untersuchungshinweise:** Add-ins sind gängige Unternehmenssoftware. Prüfen Sie LoadBehavior, Anwendung/Version, Manifeste, tatsächliche Payloads und den erwarteten Hersteller. Die Prüfung garantiert nicht, dass jedes installierte Add-in-Format oder jede Registrierungsansicht gesichert oder geladen wurde.

### Hijacking von Office-AI-Add-ins {#office_ai_hijack}

**Check-ID:** `office_ai_hijack` · **ATT&CK laut Katalog:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Was Juntei prüft:** Zählt Werte am Registrierungsort Common AI von Office 16.0 Click-to-Run auf.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI`: alle Werte.

**Untersuchungshinweise:** Vergleichen Sie ungewöhnliche Werte und Pfade mit dem installierten Office-Build. Das Vorhandensein eines Werts zeigt nicht, welche Office-Funktion ihn verwendet hat, und beweist nicht, dass er ausführbaren Code ausgewählt hat.

### Office-DLL-Überschreibung {#office_dll_override}

**Check-ID:** `office_dll_override` · **ATT&CK laut Katalog:** [T1137](https://attack.mitre.org/techniques/T1137/)

**Was Juntei prüft:** Liest `WwlibtDll` und `PPCoreTDLL` unter den Word-/PowerPoint-Wurzeln für versionslose Einstellungen sowie 14.0/15.0/16.0, einschließlich der implementierten Click-to-Run- und Benutzerwurzeln.

**Untersuchte Beweise:**

- Einstellungen `Microsoft\Office\[<version>\]Word` und `PowerPoint` von Maschine/Benutzer sowie der implementierte Click-to-Run-Office-Baum der Maschine: `WwlibtDll` und `PPCoreTDLL`.

**Untersuchungshinweise:** Ermitteln Sie, welche Anwendung/welcher Build die Überschreibung erkennt, prüfen Sie die Bibliothek und ihre Herkunft und korrelieren Sie mit einem Office-Start oder einem Image-Ladevorgang. Eine konfigurierte Überschreibung allein ist kein Ausführungsbeweis.

### Office-Standardvorlagen {#office_templates}

**Check-ID:** `office_templates` · **ATT&CK laut Katalog:** [T1137.001](https://attack.mitre.org/techniques/T1137/001/)

**Was Juntei prüft:** Inventarisiert gesicherte Normal.dotm-/NormalEmail.dotm-Dateien und Dateien, die direkt in bekannten Word-Startup- und Excel-XLSTART-Ordnern liegen, einschließlich der Startup-Ordner der Office-Installation.

**Untersuchte Beweise:**

- `Users\<user>\AppData\Roaming\Microsoft\Templates\Normal.dotm` und `NormalEmail.dotm`.
- Direkt enthaltene Dateien in den Benutzerordnern `Microsoft\Word\STARTUP` und `Microsoft\Excel\XLSTART` unter Roaming AppData.
- Direkt enthaltene Dateien in bekannten Office-Ordnern `<version>\STARTUP` / `XLSTART` und `root\<version>\STARTUP` / `XLSTART` unter Program Files und Program Files (x86).

**Untersuchungshinweise:** Eine Vorlage kann legitim sein oder keinen ausführbaren Inhalt enthalten. Juntei extrahiert oder führt in dieser Prüfung keine Makros aus. Prüfen Sie das Innere des Dokuments, die Makrorichtlinie und die Anwendungsaktivität; ein Installationspfad der Maschine impliziert kein SYSTEM-Prozesstoken.

### Office-Test-DLL {#office_test_dll}

**Check-ID:** `office_test_dll` · **ATT&CK laut Katalog:** [T1137.002](https://attack.mitre.org/techniques/T1137/002/)

**Was Juntei prüft:** Zählt Werte im Schlüssel Office Test Special Perf aus den aufgeführten Ansichten von Maschine/Benutzer sowie der 32-Bit-Ansicht der Maschine auf.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Office Test\Special\Perf` | Alle nicht leeren Werte |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Office Test\Special\Perf` | Alle nicht leeren Werte |
| `HKU\<user>\SOFTWARE\Microsoft\Office Test\Special\Perf` | Alle nicht leeren Werte |

**Untersuchungshinweise:** Behandeln Sie unerwartete Überschreibungen als Ansatzpunkt, der eine Validierung von Office-Version und Payload erfordert. Die Prüfung meldet die Konfiguration, statt zu testen, ob eine bestimmte Office-Installation den Wert lädt.

### Outlook-Homepage-Angriff {#outlook_home_page}

**Check-ID:** `outlook_home_page` · **ATT&CK laut Katalog:** [T1137.004](https://attack.mitre.org/techniques/T1137/004/)

**Was Juntei prüft:** Liest `URL` unterhalb untergeordneter Elemente von Outlook WebView des Benutzers für die Office-Versionen 14.0, 15.0 und 16.0.

**Untersuchte Beweise:**

- `HKU\<user>\Software\Microsoft\Office\<14.0|15.0|16.0>\Outlook\WebView\<folder>\URL` in NTUSER.DAT.

**Untersuchungshinweise:** Eine Ordner-Homepage kann legitim sein, und das Verhalten variiert je nach Outlook-Version und Sicherheitsupdates. Prüfen Sie URL/Inhalt und die wirksame Richtlinie und korrelieren Sie dann mit Outlook-, Web- und Prozessbelegen.

### Hijacking der VBA-Monitor-DLL {#vba_monitors}

**Check-ID:** `vba_monitors` · **ATT&CK laut Katalog:** [T1137](https://attack.mitre.org/techniques/T1137/)

**Was Juntei prüft:** Liest InprocServer32-Werte für die beiden implementierten VBA-Monitor-bezogenen CLSIDs in den Klassenspeichern der Maschine und der Benutzer.

**Untersuchte Beweise:**

- `Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32` und `{0002E157-0000-0000-C000-000000000046}` im SOFTWARE-Hive der Maschine sowie entsprechende Registrierungen in UsrClass.dat des Benutzers.

**Untersuchungshinweise:** Vergleichen Sie die ausgewählten Bibliotheken mit der installierten Office-/VBA-Umgebung. Dies ist eine gezielte CLSID-Prüfung, kein Inventar aller VBA-Erweiterungen und kein Beweis, dass ein Makro ausgeführt wurde.

## Debugger, Absturzhandler und Kompatibilitäts-Shims { #debuggers-crash-handlers-and-compatibility-shims }

Ein Debugger oder Fehlerhandler kann ausgeführt werden, wenn eine Anwendung startet, abstürzt, hängt oder beendet wird. Datenbanken zur Anwendungskompatibilität können ebenfalls den Start beeinflussen. Untersuchen Sie die vorausgesetzten Einstellungen und das auslösende Programm zusammen mit dem gemeldeten Befehl.

### AeDebug-Debugger-Hijacking {#ae_debug}

**Check-ID:** `ae_debug` · **ATT&CK laut Katalog:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Was Juntei prüft:** Liest native und 32-Bit-`Debugger`-Befehle von AeDebug, die von der Konfiguration für Just-in-Time-Absturzdebugging verwendet werden.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |

**Untersuchungshinweise:** Entwicklungs-/Debugging-Software kann dies legitim setzen. Untersuchen Sie den vollständigen Debugger-Befehl und die Voraussetzungen für einen auslösenden Absturz und korrelieren Sie dann mit Anwendungsfehler- und Prozessereignissen.

### AeDebug-Debugger für geschützte Prozesse {#ae_debug_protected}

**Check-ID:** `ae_debug_protected` · **ATT&CK laut Katalog:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Was Juntei prüft:** Liest native und 32-Bit-`Debugger`-Befehle von AeDebugProtected für die Debugging-Oberfläche geschützter Prozesse.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |

**Untersuchungshinweise:** Ermitteln Sie, ob der Betriebssystem-Build und die Schutzrichtlinie den konfigurierten Befehl verwenden können. Ein gespeicherter Debugger ist kein Beweis, dass ein geschützter Prozess abgestürzt ist oder dass der Befehl ausgeführt wurde.

### Debugger über Image File Execution Options {#ifeo_debugger}

**Check-ID:** `ifeo_debugger` · **ATT&CK laut Katalog:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Was Juntei prüft:** Durchsucht native/32-Bit-Registrierungen der Image File Execution Options, einschließlich verschachtelter Filterschlüssel, nach `Debugger`-Werten.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\Debugger`, einschließlich verschachtelter Filterschlüssel und der `Wow6432Node`-Ansicht der Maschine.

**Untersuchungshinweise:** Ein Debugger kann ausgewählt werden, wenn das genannte Image gestartet wird. Prüfen Sie den Geltungsbereich von Image/Filter und die Debugger-Argumente; auch Entwicklerwerkzeuge können IFEO verwenden. Verfolgen Sie den Start des Ziels, um eine Ausführung zu belegen.

### IFEO Delegated NTDLL {#ifeo_delegated_ntdll}

**Check-ID:** `ifeo_delegated_ntdll` · **ATT&CK laut Katalog:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Was Juntei prüft:** Durchsucht native/32-Bit-Registrierungen der Image File Execution Options, einschließlich verschachtelter Schlüssel, nach `VerifierDlls`-Überschreibungen.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\VerifierDlls`, einschließlich verschachtelter Schlüssel und der `Wow6432Node`-Ansicht der Maschine.

**Untersuchungshinweise:** Das Kriterium der Prüfung ist der konfigurierte VerifierDlls-Wert; sie beweist nicht, dass alle Voraussetzungen für Verifier/Delegierung aktiviert sind. Prüfen Sie zutreffende Flags, die Windows-Version und die Payload, bevor Sie auf einen funktionierenden Injektionspfad schließen.

### Überwachung stiller Prozessbeendigung {#ifeo_silent_process_exit}

**Check-ID:** `ifeo_silent_process_exit` · **ATT&CK laut Katalog:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Was Juntei prüft:** Durchsucht native/32-Bit-Registrierungen von SilentProcessExit, einschließlich verschachtelter Schlüssel, nach `MonitorProcess`.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\<image>\...\MonitorProcess`, einschließlich verschachtelter Schlüssel und der `Wow6432Node`-Ansicht der Maschine.

**Untersuchungshinweise:** Die Überwachung stiller Beendigungen erfordert außerdem die entsprechende Berichts-/IFEO-Konfiguration. Prüfen Sie diese Einstellungen und die Beendigungsereignisse des überwachten Images; legitime Diagnosewerkzeuge können die Überwachung erklären.

### Application Shimming {#installed_sdb}

**Check-ID:** `installed_sdb` · **ATT&CK laut Katalog:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Was Juntei prüft:** Liest `DatabasePath` aus unmittelbaren InstalledSDB-Registrierungen installierter Datenbanken zur Anwendungskompatibilität.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\<child>` | `DatabasePath` |

**Untersuchungshinweise:** Kompatibilitätskorrekturen können legitim sein. Prüfen Sie den Inhalt der SDB, die betroffenen ausführbaren Dateien und den Installationsverlauf. Eine Datenbankregistrierung identifiziert weder ihre Shims noch beweist sie, dass eine passende Anwendung ausgeführt wurde.

### LSM-Debugger-Hijacking {#lsm_debugger}

**Check-ID:** `lsm_debugger` · **ATT&CK laut Katalog:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Was Juntei prüft:** Liest `MonitorProcess` gezielt unter SilentProcessExit für `lsm.exe` in den nativen und 32-Bit-Ansichten der Maschine.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |

**Untersuchungshinweise:** Dies überschneidet sich mit dem umfassenderen SilentProcessExit-Inventar. Bestätigen Sie vorausgesetzte Flags und eine relevante Beendigung von LSM, bevor Sie die Konfiguration als ausgeführte Persistenz behandeln; zählen Sie denselben Eintrag bei der Bewertung unterschiedlicher Ergebnisse nur einmal.

### MiniDump-Hilfs-DLLs {#minidump_auxiliary_dlls}

**Check-ID:** `minidump_auxiliary_dlls` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Zählt benannte, nicht leere Werte in MiniDumpAuxiliaryDlls auf und behält den Wertnamen als Kontext des auslösenden Moduls und die Wertdaten als ausgewähltes Ziel bei.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls`: Wertnamen der auslösenden Module und Zieldaten.

**Untersuchungshinweise:** Prüfen Sie sowohl das auslösende Modul als auch die Hilfs-DLL. Absturz-/Minidump-Werkzeuge können einen Eintrag erklären, und eine Registrierung zeigt nicht, dass ein Dump-Vorgang die Bibliothek geladen hat.

### Application Shimming {#shim_custom}

**Check-ID:** `shim_custom` · **ATT&CK laut Katalog:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Was Juntei prüft:** Liest auf `.sdb` endende Werte unterhalb unmittelbarer Anwendungsschlüssel von AppCompatFlags Custom und behält die Zuordnung von Anwendung zu Datenbank bei.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<application>`: Wertnamen, die auf `.sdb` enden.

**Untersuchungshinweise:** Kombinieren Sie diese Zuordnungen mit InstalledSDB und der tatsächlichen Kompatibilitätsdatenbank. Eine Zuordnung über den Dateinamen allein offenbart weder das Verhalten der Datenbank noch belegt sie einen passenden Anwendungsstart.

### WER-Debugger-Hijacking {#wer_debugger}

**Check-ID:** `wer_debugger` · **ATT&CK laut Katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Was Juntei prüft:** Liest native/32-Bit-`Debugger`-Werte der Windows-Fehlerberichterstattung.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `Debugger` |

**Untersuchungshinweise:** Legitime Diagnosesoftware kann einen Debugger konfigurieren. Untersuchen Sie den vollständigen Befehl und relevante Absturzereignisse; die Registrierung ist kein Beweis, dass die Windows-Fehlerberichterstattung ihn gestartet hat.

### WER-Hangs-Debugger-Hijacking {#wer_hangs}

**Check-ID:** `wer_hangs` · **ATT&CK laut Katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Was Juntei prüft:** Liest native/32-Bit-`Debugger`-Werte unter Hangs der Windows-Fehlerberichterstattung.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |

**Untersuchungshinweise:** Prüfen Sie, ob ein Ablauf zur Behandlung hängender Anwendungen diese Einstellung aufrufen würde, und untersuchen Sie das Zielprogramm. Korrelieren Sie mit Datensätzen zu hängenden Anwendungen und Prozessen, statt einen Registry-Schreibvorgang als Debugger-Ausführung zu werten.

### WER-ReflectDebugger-Hijacking {#wer_reflect_debugger}

**Check-ID:** `wer_reflect_debugger` · **ATT&CK laut Katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Was Juntei prüft:** Liest native/32-Bit-`ReflectDebugger`-Werte der Windows-Fehlerberichterstattung.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |

**Untersuchungshinweise:** Vergleichen Sie mit genehmigter Diagnosekonfiguration und klären Sie die Anwendbarkeit hinsichtlich Windows-Version/Auslöser. Prüfen Sie die Befehlsargumente und suchen Sie nach bestätigenden Absturz-/Prozessbelegen, bevor Sie eine Ausführung zuschreiben.

### WER-Laufzeitausnahmemodule {#wer_runtime_exception}

**Check-ID:** `wer_runtime_exception` · **ATT&CK laut Katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Was Juntei prüft:** Zählt Wertnamen unter nativen/32-Bit-RuntimeExceptionHelperModules auf; hier steckt der Bibliothekspfad im Wertnamen, nicht notwendigerweise in den Daten.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules` und die `Wow6432Node`-Ansicht der Maschine: Wertnamen.

**Untersuchungshinweise:** Prüfen Sie das genannte Modul und den Anwendungs-/Laufzeitkontext. Ein kleiner numerischer Wert in den Daten ist nicht der Payload-Pfad, und die Registrierung allein beweist nicht, dass das Hilfsmodul geladen wurde.

## .NET-Laufzeit und Profiler-Einstellungen { #net-runtime-and-profiler-settings }

Verwaltete Laufzeitumgebungen können Profiler, Start-Assemblies oder Debugging-Bibliotheken vor dem Anwendungscode laden. Diese Einstellungen werden auch von Entwicklern und Überwachungsprodukten verwendet. Prüfen Sie Laufzeitumgebung, Prozessarchitektur, Aktivierungsflags und ob ein neu gestarteter Prozess die Einstellung erben könnte.

### AppDomainManager-Injektion {#appdomain_manager}

**Check-ID:** `appdomain_manager` · **ATT&CK laut Katalog:** [T1574.014](https://attack.mitre.org/techniques/T1574/014/)

**Was Juntei prüft:** Liest die Umgebungsvariablen `APPDOMAIN_MANAGER_ASM` und `APPDOMAIN_MANAGER_TYPE` von Maschine/Benutzer, die Assembly und Typ für einen benutzerdefinierten .NET-AppDomain-Manager festlegen.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_TYPE` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_TYPE` |

**Untersuchungshinweise:** Anwendungen können benutzerdefinierte Manager absichtlich verwenden. Lösen Sie Assembly und Typ gemeinsam auf und prüfen Sie Laufzeitkompatibilität und Prozessvererbung; weder eine einzelne Variable noch der Zeitstempel ihres Schlüssels beweist, dass eine verwaltete Anwendung sie geladen hat.

### .NET-CLR-Profiler-Hijacking {#cor_profiler}

**Check-ID:** `cor_profiler` · **ATT&CK laut Katalog:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Was Juntei prüft:** Liest COR-Profiling-Umgebungseinstellungen von Maschine/Benutzer sowie passende benannte Einträge in den Environment-Mehrfachzeichenfolgen einzelner Dienste. Es behält Aktivierungsflags, Profiler-Kennungen und architekturspezifische Pfade bei.

**Untersuchte Beweise:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` und `Environment` in jeder gesicherten NTUSER.DAT eines Benutzers.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: benannte COR-Einstellungen in der Mehrfachzeichenfolge.

**Untersuchungshinweise:** Profiler unterstützen legitime Überwachung. Ein Aktivierungsflag, eine CLSID und ein DLL-Pfad sind verschiedene Konfigurationsbestandteile; bestätigen Sie die wirksame Aktivierung und den relevanten .NET-Prozess, statt jede Zeile als ausgeführten Profiler zu behandeln.

### .NET-Core-CLR-Profiler-Hijacking {#coreclr_profiler}

**Check-ID:** `coreclr_profiler` · **ATT&CK laut Katalog:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Was Juntei prüft:** Liest entsprechende CORECLR-Profiling-Umgebungseinstellungen von Maschine/Benutzer und passende Environment-Einträge von Diensten, einschließlich architekturspezifischer Pfade.

**Untersuchte Beweise:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` und `Environment` in jeder gesicherten NTUSER.DAT eines Benutzers.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: benannte CORECLR-Einstellungen in der Mehrfachzeichenfolge.

**Untersuchungshinweise:** Validieren Sie .NET-Laufzeit, Architektur, Aktivierungsflag und Bibliothek. Genehmigte Observability-Produkte können diese Einstellungen konfigurieren, und ein bereits laufender Prozess erbt eine neu geänderte Umgebung möglicherweise nicht.

### .NET-DbgManagedDebugger-Hijacking {#dotnet_dbg_managed_debugger}

**Check-ID:** `dotnet_dbg_managed_debugger` · **ATT&CK laut Katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Was Juntei prüft:** Liest die aufgeführten `.NETFramework`-Einstellungen `DbgManagedDebugger` von Maschine/Benutzer, einschließlich des expliziten 32-Bit-Pfads der Maschine.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `DbgManagedDebugger` |

**Untersuchungshinweise:** Ermitteln Sie, ob die zutreffende .NET-Laufzeit bzw. Debugging-Richtlinie diesen Befehl verwendet. Entwicklerwerkzeuge können ihn erklären; korrelieren Sie mit Abstürzen verwalteter Anwendungen und der Prozesserstellung, um eine Ausführung zu belegen.

### .NET-Framework-Registry-Schlüssel COR_PROFILER {#dotnet_framework_profiler}

**Check-ID:** `dotnet_framework_profiler` · **ATT&CK laut Katalog:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Was Juntei prüft:** Liest `COR_PROFILER`, `COR_PROFILER_PATH` und `COR_ENABLE_PROFILING` aus den aufgeführten `.NETFramework`-Registry-Ansichten. Dies ergänzt die umgebungsbasierten Profiler-Prüfungen.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |

**Untersuchungshinweise:** Bewerten Sie die Werte gemeinsam und ermitteln Sie, ob die Laufzeit diesen Registry-Speicherort berücksichtigt. Ein konfiguriertes Profiling-Werkzeug kann legitim sein; deuten Sie ein alleinstehendes Aktivierungsflag nicht als identifizierten DLL-Ladevorgang.

### DOTNET_STARTUP_HOOKS {#dotnet_startup_hooks}

**Check-ID:** `dotnet_startup_hooks` · **ATT&CK laut Katalog:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Was Juntei prüft:** Liest die Umgebungswerte `DOTNET_STARTUP_HOOKS` von Maschine/Benutzer, die Startup-Hook-Assemblies für unterstützte .NET-Laufzeiten festlegen können.

**Untersuchte Beweise:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment\DOTNET_STARTUP_HOOKS` und `Environment\DOTNET_STARTUP_HOOKS` in der NTUSER.DAT jedes Benutzers.

**Untersuchungshinweise:** Prüfen Sie den gesamten Wert auf mehrere Assemblies und überprüfen Sie die Unterstützung durch die Laufzeit. Diese Prüfung zählt keine dienstspezifischen Startup-Hook-Umgebungen auf und beweist nicht, dass ein Prozess den Wert geerbt und verwendet hat.

### Bekannte Debugging-DLLs für verwalteten Code {#known_managed_debugging_dlls}

**Check-ID:** `known_managed_debugging_dlls` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest `.NETFramework`-Registrierungen `KnownManagedDebuggingDlls` als mögliche Bibliotheksüberschreibungen des Debuggers für verwalteten Code.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `KnownManagedDebuggingDlls` |

**Untersuchungshinweise:** Vergleichen Sie mit installierten Entwicklungs-/Laufzeitwerkzeugen und ermitteln Sie den auslösenden Debugger. Der Registry-Eintrag beweist weder eine Debugger-Sitzung noch das Laden einer Bibliothek.

## Überschreibungen der DLL- und Programmsuche { #dll-and-executable-lookup-overrides }

Diese Prüfungen untersuchen explizite Bibliotheksspeicherorte oder Einstellungen, die beeinflussen, wie eine Komponente Code findet. Eine Pfadüberschreibung oder eine abgeschwächte Suchrichtlinie ist ein Ermittlungsansatz; für eine Ausführung sind weiterhin eine Payload und eine kompatible auslösende Komponente erforderlich.

### AppCert-DLLs {#appcert_dlls}

**Check-ID:** `appcert_dlls` · **ATT&CK laut Katalog:** [T1546.009](https://attack.mitre.org/techniques/T1546/009/)

**Was Juntei prüft:** Zählt nicht leere AppCertDlls-Registrierungen des Session Managers auf, einer Erweiterungsoberfläche für Bibliotheken, die mit APIs zur Prozesserstellung verbunden ist.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\AppCertDlls` | Alle nicht leeren Werte |

**Untersuchungshinweise:** Prüfen Sie jede Bibliothek und ihre Installationsherkunft. Klären Sie Anwendbarkeit und Laden zur Laufzeit gesondert; eine normale Prozesserstellung in einer Zeitleiste ist für sich genommen kein Beweis, dass die konfigurierte DLL geladen wurde.

### AppInit-DLLs {#appinit_dlls}

**Check-ID:** `appinit_dlls` · **ATT&CK laut Katalog:** [T1546.010](https://attack.mitre.org/techniques/T1546/010/)

**Was Juntei prüft:** Liest natives/32-Bit-`AppInit_DLLs`, teilt dessen Bibliotheksliste auf und fügt jedem Kandidaten den verfügbaren Kontext aus `LoadAppInit_DLLs` und `RequireSignedAppInit_DLLs` hinzu.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows`: `AppInit_DLLs`, `LoadAppInit_DLLs` und `RequireSignedAppInit_DLLs`; außerdem die `Wow6432Node`-Ansicht der Maschine.

**Untersuchungshinweise:** Ein LoadAppInit_DLLs-Flag mit dem Wert Null wird ausdrücklich als inaktiv gekennzeichnet. Windows-Version, Secure Boot und Signaturanforderungen beeinflussen das Laden ebenfalls. Prüfen Sie die wirksamen Einstellungen, bevor Sie behaupten, dass eine AppInit-DLL ausgeführt wurde.

**Weitere Informationen:** [Microsoft-Dokumentation](https://learn.microsoft.com/en-us/windows/win32/dlls/secure-boot-and-appinit-dlls).

### AutodialDLL-Überschreibung {#autodial_dll}

**Check-ID:** `autodial_dll` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest die Überschreibung `AutodialDLL` unter den WinSock2-Parametern.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters` | `AutodialDLL` |

**Untersuchungshinweise:** Prüfen Sie die Netzwerkfunktion und die Windows-Version der Quelle, die diese Einstellung verwenden. Prüfen Sie die tatsächliche DLL und die Netzwerk-/Prozessaktivität; eine Überschreibung allein belegt keinen aktiven oder bösartigen Ladevorgang.

### CHM-Hilfs-DLL {#chm_helper_dll}

**Check-ID:** `chm_helper_dll` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest `Location` unter HtmlHelp Author, einer hilfebezogenen Überschreibungsoberfläche für Bibliotheken/Speicherorte.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\HtmlHelp Author` | `Location` |

**Untersuchungshinweise:** Ermitteln Sie, ob die betreffende Hilfekomponente den Wert auf diesem System verwendet, und prüfen Sie die ausgewählte Datei. Eine Registry-Überschreibung ist kein Beweis, dass ein CHM-Dokument geöffnet wurde.

### NL6-ContentIndex-DLL-Überschreibung {#content_index_dll}

**Check-ID:** `content_index_dll` · **ATT&CK laut Katalog:** [T1574](https://attack.mitre.org/techniques/T1574/)

**Was Juntei prüft:** Liest `DLLOverridePath` aus unmittelbaren ContentIndex-Language-Registrierungen des aktiven Control Sets.

**Untersuchte Beweise:**

- `HKLM\SYSTEM\<ControlSet>\Control\ContentIndex\Language\<language>\DLLOverridePath`.

**Untersuchungshinweise:** Sprach-/Suchkomponenten können sich je nach Windows-Build unterscheiden. Prüfen Sie die Bibliothek und die passende installierte Komponente; verfolgen Sie den tatsächlichen Indizierungsprozess, um das Laden zu bestätigen.

### Crypto-ExpoOffload-DLL {#crypto_expo_offload}

**Check-ID:** `crypto_expo_offload` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest den Wert `ExpoOffload` unter Cryptography Offload als Kandidaten für die Überschreibung einer kryptografischen Bibliothek.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Cryptography\Offload` | `ExpoOffload` |

**Untersuchungshinweise:** Bestätigen Sie die Anwendbarkeit von Funktion/Build und untersuchen Sie die Herkunft der ausgewählten Bibliothek. Behandeln Sie das Vorhandensein eines ungewöhnlichen Werts nicht als Beweis für den Diebstahl von Anmeldeinformationen oder ausgeführten Code.

### Direct3D-Software-Rasterizer-DLL {#direct3d_dll}

**Check-ID:** `direct3d_dll` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest die Bibliotheksüberschreibung `SoftwareRasterizer` unter Direct3D Drivers.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Direct3D\Drivers` | `SoftwareRasterizer` |

**Untersuchungshinweise:** Grafik- und Kompatibilitätssoftware kann die Rasterizer-Konfiguration beeinflussen. Prüfen Sie die DLL und die Aktivität einer kompatiblen Direct3D-Anwendung; das Speichern des Pfads beweist nicht, dass sie geladen wurde.

### Abschwächung der DLL-Suchreihenfolge {#dll_search_mode}

**Check-ID:** `dll_search_mode` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Meldet `SafeDllSearchMode` oder `CWDIllegalInDllSearch` nur, wenn der aktive Session-Manager-Wert als Null ausgewertet wird. Dies sind Bedingungen der Suchrichtlinie, keine ausgewählten Payloads.

**Untersuchte Beweise:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager`: `SafeDllSearchMode` und `CWDIllegalInDllSearch`.

**Untersuchungshinweise:** Eine abgeschwächte Suchrichtlinie kann einen separaten Angriff auf die Suchreihenfolge erleichtern. Sie identifiziert weder eine platzierte DLL noch beweist sie eine Ausführung. Werte ungleich Null oder fehlende Werte werden von dieser Prüfung nicht ausgegeben; bewerten Sie wirksame Standardwerte und Richtlinien gesondert.

**Weitere Informationen:** [Microsoft-Dokumentation](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security).

### Ausschluss aus KnownDLLs {#exclude_from_known_dlls}

**Check-ID:** `exclude_from_known_dlls` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest nicht leere `ExcludeFromKnownDlls`-Einträge aus dem Session Manager als Änderungen am Suchverhalten für geschützte/bekannte DLLs.

**Untersuchte Beweise:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\ExcludeFromKnownDlls`.

**Untersuchungshinweise:** Ein Ausschluss kennzeichnet eine Suchbedingung, nicht die Ersatzbibliothek selbst. Ermitteln Sie die betroffene Anwendung und mögliche DLL-Suchorte und bestätigen Sie dann einen vom Standard abweichenden Image-Ladevorgang.

**Weitere Informationen:** [Microsoft-Dokumentation](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order).

### hhctrl.ocx-DLL-Überschreibung {#hhctrl_ocx_dll}

**Check-ID:** `hhctrl_ocx_dll` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest InprocServer32-Überschreibungen für die implementierte CLSID des HTML-Hilfe-Steuerelements in nativen/32-Bit-Klassenspeichern der Maschine und in unterstützten Klassenspeichern der Benutzer.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKLM\SOFTWARE\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\Software\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |

**Untersuchungshinweise:** Ermitteln Sie, welche Anwendung und Architektur das Steuerelement aktivieren. Prüfen Sie die registrierte DLL und hilfebezogene Aktivität; eine COM-Registrierung beweist nicht, dass die HTML-Hilfe Inhalte dargestellt hat.

### Bekannte DLLs {#known_dlls}

**Check-ID:** `known_dlls` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Zählt benannte Werte im aktiven KnownDLLs-Schlüssel des Session Managers auf.

**Untersuchte Beweise:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\KnownDLLs`: benannte Werte.

**Untersuchungshinweise:** Viele Einträge sind erwartete Windows-Bibliotheken, und einige Werte sind ergänzende Konfiguration statt eines Payload-Pfads. Vergleichen Sie den vollständigen Schlüssel mit einem vertrauenswürdigen passenden Build; diese Prüfung durchsucht nicht jeden alternativen architekturspezifischen KnownDLLs-Baum.

### LSA-Erweiterungs-DLL {#lsa_extensions}

**Check-ID:** `lsa_extensions` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest `Extensions` aus der LsaSrv-Konfiguration unter LsaExtensionConfig.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\LsaExtensionConfig\LsaSrv` | `Extensions` |

**Untersuchungshinweise:** Sicherheitssoftware kann Authentifizierungskomponenten erweitern. Prüfen Sie die tatsächlichen Bibliotheken und deren Kompatibilität mit der Windows-Version der Quelle und suchen Sie dann nach Image-Ladevorgängen in LSASS oder anderen unabhängigen Nutzungsbelegen.

### Überschreibung des MAPI32-DLL-Pfads {#mapi32_dll_path}

**Check-ID:** `mapi32_dll_path` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest `DLLPathEx` und `DLLPath` unterhalb unmittelbarer Registrierungen von E-Mail-Clients in den nativen und 32-Bit-Ansichten der Maschine.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPath` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPath` |

**Untersuchungshinweise:** E-Mail-Clients registrieren legitime MAPI-Anbieter. Ermitteln Sie, welcher Client/Anbieter ausgewählt war, und prüfen Sie die tatsächliche Bibliothek; ein installierter Anbieter beweist nicht, dass eine Anwendung während des Vorfalls MAPI verwendet hat.

### MSDTC-XA-DLL {#msdtc_xa_dll}

**Check-ID:** `msdtc_xa_dll` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest `OracleXaLib` und `OracleOciLib` aus der MTxOCI-Konfiguration von MSDTC.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleXaLib` |
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleOciLib` |

**Untersuchungshinweise:** Eine Datenbankintegration kann diese Bibliotheken legitimerweise erfordern. Überprüfen Sie die Konfiguration des Transaktionskoordinators und die installierte Datenbanksoftware und korrelieren Sie mit der Nutzung von Diensten/Modulen, statt anzunehmen, dass ein konfigurierter Pfad ausgeführt wurde.

### NLDP-DLL-Überschreibung {#nldp_dll}

**Check-ID:** `nldp_dll` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest den Konfigurationswert `DllOverridePath` unter NlsData.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NlsData` | `DllOverridePath` |

**Untersuchungshinweise:** Prüfen Sie die Zielbibliothek, den Windows-Build und die Komponente zur Verarbeitung natürlicher Sprache, die die Überschreibung verwenden könnte. Der Wert zeigt nicht, dass eine Indizierung oder Sprachverarbeitung stattgefunden hat.

### RDP-TestDVCPlugin-DLL {#rdp_test_dvc_plugin}

**Check-ID:** `rdp_test_dvc_plugin` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest die Bibliothekseinstellung `TestDVCPlugin` des Terminal Server Client der Maschine.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Terminal Server Client` | `TestDVCPlugin` |

**Untersuchungshinweise:** Untersuchen Sie ein unbekanntes Test-Plugin und versionsspezifisches Verhalten. Um eine Ausführung zu belegen, sind eine kompatible Client-Sitzung und ein Bibliotheksladevorgang erforderlich; der Registry-Wert allein zeigt keines von beiden.

### DLL-Überschreibung des Suchindexers {#search_indexer_dll}

**Check-ID:** `search_indexer_dll` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest die Überschreibung `DllPath` von Windows Search aus SOFTWARE.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Search` | `DllPath` |

**Untersuchungshinweise:** Ermitteln Sie die passende Search-Komponente/-Version und prüfen Sie die Zieldatei. Ein inaktiver Windows Search-Dienst oder eine nicht verwendete Einstellung ist etwas anderes als ein beobachteter bösartiger Bibliotheksladevorgang.

### Winsock-AutoProxy-DLL {#winsock_auto_proxy}

**Check-ID:** `winsock_auto_proxy` · **ATT&CK laut Katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Was Juntei prüft:** Liest `LibraryPath` unterhalb unmittelbarer Registrierungen unter Catalog_Entries und Catalog_Entries64 von NameSpace_Catalog5.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries\<child>` | `LibraryPath` |
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries64\<child>` | `LibraryPath` |

**Untersuchungshinweise:** Netzwerkanbieter können legitim sein. Prüfen Sie beide Architekturen, die Herkunft des Anbieters und relevante Belege zu Netzwerkanwendungen bzw. Image-Ladevorgängen. Diese gezielte Abfrage von Namespace-Anbietern umfasst nicht jede mögliche Winsock-Erweiterung.

## Konten und Sichtbarkeit bei der Anmeldung { #accounts-and-sign-in-visibility }

Lokale Kontodatensätze und Einstellungen zur Sichtbarkeit bei der Anmeldung können identitätsbasierte Persistenz offenlegen. Diese Prüfungen untersuchen die gesicherten SAM- und SOFTWARE-Hives; sie sind kein Inventar von Active Directory-Konten, Domänengruppenmitgliedschaften oder vergangenen Anmeldungen.

### Mitgliedschaft in der Gruppe Administratoren {#admin_group_membership}

**Check-ID:** `admin_group_membership` · **ATT&CK laut Katalog:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Was Juntei prüft:** Dekodiert die Mitgliederliste des lokalen Alias Administratoren aus SAM. Es meldet SIDs der lokalen Maschine, die ein verstecktes Konto oder im gesicherten SAM kein Konto bzw. keine Gruppe bezeichnen.

**Untersuchte Beweise:**

- `SAM\Domains\Builtin\Aliases\00000220\C` und Datensätze zu Maschinen-SID/Benutzern/Gruppen unterhalb von `SAM\Domains\Account` im SAM-Hive.
- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList` zur Korrelation versteckter Namen.

**Untersuchungshinweise:** Dies ist eine gezielte Anomalieprüfung, keine Liste aller Administratoren. Externe/Domänen-SIDs werden nicht als fehlende lokale Benutzer bewertet. Gelöschte Konten, der Zeitpunkt der Sicherung oder unvollständige Hives erfordern eine Untersuchung, bevor auf eine bösartige Mitgliedschaft geschlossen wird.

### Verstecktes lokales Konto {#hidden_account}

**Check-ID:** `hidden_account` · **ATT&CK laut Katalog:** [T1564.002](https://attack.mitre.org/techniques/T1564/002/)

**Was Juntei prüft:** Meldet numerische Nullwerte in UserList unter Winlogon SpecialAccounts und identifiziert so Namen, die auf dem gewöhnlichen Anmeldebildschirm verborgen sind.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`: Werte mit Kontonamen.

**Untersuchungshinweise:** Dienst- oder Wartungskonten können absichtlich verborgen sein, und der Wert belegt nicht, dass das genannte Konto existiert oder ein Administrator ist. Verknüpfen Sie mit SAM-Kontodatensätzen und Anmeldeaktivität.

### RID-Hijacking {#rid_hijacking}

**Check-ID:** `rid_hijacking` · **ATT&CK laut Katalog:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Was Juntei prüft:** Vergleicht die hexadezimale RID eines SAM-Kontounterschlüssels mit der RID, die an Offset 0x30 in dessen F-Wert gespeichert ist, und meldet Abweichungen.

**Untersuchte Beweise:**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` im SAM-Hive.

**Untersuchungshinweise:** Eine Abweichung ist eine schwerwiegende Anomalie im Kontodatensatz, kann aber auch eine Analyse auf Beschädigung bzw. Sicherungsfehler erfordern. Prüfen Sie die Quellbytes, den Status der Transaktionsprotokolle sowie Konto-/Anmeldedatensätze; sie beweist nicht, welcher Akteur den Datensatz verändert hat.

### RID Suborner (verstecktes Administratorkonto) {#rid_suborner}

**Check-ID:** `rid_suborner` · **ATT&CK laut Katalog:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Was Juntei prüft:** Meldet SAM-Konten, deren Unterschlüssel-RID nicht 500 ist, deren RID im F-Wert aber 500 lautet – ein spezifischeres Muster für die Ersetzung der Administratoridentität.

**Untersuchte Beweise:**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` im SAM-Hive.

**Untersuchungshinweise:** Dies überschneidet sich mit der allgemeinen Prüfung auf RID-Abweichungen. Trotz des Katalogtitels setzt sie nicht eigenständig eine versteckte Anmeldeeinstellung voraus; bestätigen Sie Sichtbarkeit, Berechtigungen und tatsächliche Nutzung des Kontos gesondert.

## Authentifizierungskomponenten und Schutz von Anmeldeinformationen { #authentication-components-and-credential-protection }

Die Local Security Authority (LSA) wickelt Sicherheitsfunktionen über LSASS und verwandte Komponenten ab. Authentifizierungspakete, Kennwortfilter und Netzwerkanbieter können legitime Sicherheitssoftware oder ein Weg zu dauerhaftem Zugriff auf Anmeldeinformationen sein. Zwei Prüfungen inventarisieren Schutzeinstellungen statt ausführbarer Autostarts. Interpretieren Sie sie anhand der Rolle des Hosts, der Windows-Version und der wirksamen Richtlinie.

### AMSI-Anbieter-DLL {#amsi_providers}

**Check-ID:** `amsi_providers` · **ATT&CK laut Katalog:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Was Juntei prüft:** Zählt die AMSI-Anbieter-CLSIDs der Maschine auf und löst, sofern verfügbar, deren native/32-Bit-InprocServer32-Bibliotheken auf. Nicht aufgelöste Registrierungen behalten die Klassen-ID.

**Untersuchte Beweise:**

- `HKLM\SOFTWARE\Microsoft\AMSI\Providers\<CLSID>` und auflösbare InprocServer32-Registrierungen unter `Classes\CLSID` / `Classes\Wow6432Node\CLSID`.

**Untersuchungshinweise:** Antiviren- und Sicherheitsprodukte registrieren legitimerweise AMSI-Anbieter. Prüfen Sie die Herkunft von Anbieter/Bibliothek und das tatsächliche Laden; eine Registrierung beweist nicht, dass AMSI umgangen wurde oder dass der Anbieter bösartig war.

### Authentifizierungspakete {#authentication_packages}

**Check-ID:** `authentication_packages` · **ATT&CK laut Katalog:** [T1547.002](https://attack.mitre.org/techniques/T1547/002/)

**Was Juntei prüft:** Liest einzelne nicht leere LSA-Einträge unter `Authentication Packages` aus dem aktiven Control Set.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Authentication Packages` |

**Untersuchungshinweise:** Vergleichen Sie mit dem Windows-Build der Quelle und der installierten Authentifizierungssoftware. Ein vom Standard abweichendes Paket kann legitim sein. Lösen Sie die DLL auf und bestätigen Sie Lade- bzw. Anmeldeinformationsaktivität, bevor Sie auf ein Abfangen schließen.

### DSRM-Administrator-Anmeldeverhalten {#dsrm_backdoor}

**Check-ID:** `dsrm_backdoor` · **ATT&CK laut Katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Was Juntei prüft:** Meldet den numerischen Wert `DsrmAdminLogonBehavior=2` an den implementierten Speicherorten ControlSet001, ControlSet002 oder CurrentControlSet. Diese Einstellung kann die Nutzung des DSRM-Kontos außerhalb seines üblichen Wiederherstellungskontexts ermöglichen.

**Untersuchte Beweise:**

- `HKLM\SYSTEM\ControlSet001\Control\Lsa\DsrmAdminLogonBehavior` sowie ControlSet002 und CurrentControlSet, sofern vorhanden.

**Untersuchungshinweise:** Vor allem für Domänencontroller relevant. Ermitteln Sie anhand von Select in SYSTEM, welches Control Set aktiv ist; eine Übereinstimmung in einem historischen Control Set ist nicht notwendigerweise wirksam. Der Wert beweist nicht, dass ein Angreifer das DSRM-Kennwort kannte oder sich angemeldet hat.

### Credential Guard-Konfiguration {#lsa_cfg_flags}

**Check-ID:** `lsa_cfg_flags` · **ATT&CK laut Katalog:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Was Juntei prüft:** Liest den konfigurierten LSA-Wert `LsaCfgFlags`, einschließlich gewöhnlicher aktivierter Werte ebenso wie Null. Dies inventarisiert die Credential Guard-Konfiguration statt eines ausführbaren Autostarts.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `LsaCfgFlags` |

**Untersuchungshinweise:** Interpretieren Sie den Wert zusammen mit der Windows-Version, Firmware-/Sicherheitsvoraussetzungen und der wirksamen Richtlinie. Ein gespeicherter Nullwert oder ein fehlender Wert allein belegt weder eine Änderung durch einen Angreifer noch den wirksamen Schutzzustand oder einen Diebstahl von Anmeldeinformationen.

### LSA-Kennwortfilter {#lsa_password_filter}

**Check-ID:** `lsa_password_filter` · **ATT&CK laut Katalog:** [T1556.002](https://attack.mitre.org/techniques/T1556/002/)

**Was Juntei prüft:** Liest LSA-`Notification Packages`. Reine Bibliotheksnamen werden gegen das gesicherte Windows-Verzeichnis System32 aufgelöst, wobei gegebenenfalls `.dll` ergänzt wird.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Notification Packages` |

**Untersuchungshinweise:** Produkte für Kennwortrichtlinien installieren häufig Filter. Prüfen Sie die Bibliothek und Ladebelege und unterscheiden Sie konfigurierte Filter von einem Beweis, dass eine Kennwortänderung oder ein Abfangen von Anmeldeinformationen stattgefunden hat.

### LSASS-PPL-Schutzstatus {#lsa_run_as_ppl}

**Check-ID:** `lsa_run_as_ppl` · **ATT&CK laut Katalog:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Was Juntei prüft:** Liest den konfigurierten LSA-Wert `RunAsPPL`, einschließlich Werten ungleich Null. Es handelt sich um ein Inventar der Schutzeinstellungen, nicht um einen Test, ob LSASS ungeschützt lief.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `RunAsPPL` |

**Untersuchungshinweise:** Der wirksame LSA-Schutz hängt von Windows-Version, Richtlinie, Firmware-Sperre und Neustartzustand ab. Das Fehlen einer Zeile für einen fehlenden Wert ist kein gemessenes Schutzergebnis. Verwenden Sie unabhängige System-/Laufzeitbelege, um den wirksamen Schutz zu bewerten.

### Netzwerkanbieter-DLL {#network_provider_dll}

**Check-ID:** `network_provider_dll` · **ATT&CK laut Katalog:** [T1556.008](https://attack.mitre.org/techniques/T1556/008/)

**Was Juntei prüft:** Liest `NetworkProvider\ProviderPath` unterhalb der Dienste im aktiven SYSTEM-Control-Set.

**Untersuchte Beweise:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\NetworkProvider\ProviderPath`.

**Untersuchungshinweise:** Anbieter für die Netzwerkanmeldung können legitim sein. Prüfen Sie Anbieterreihenfolge/Anwendbarkeit, installierte Netzwerk-/Sicherheitssoftware und das tatsächliche Laden der Bibliothek; ein ProviderPath-Wert allein beweist kein Abfangen von Anmeldeinformationen.

### Sicherheitspakete {#security_packages}

**Check-ID:** `security_packages` · **ATT&CK laut Katalog:** [T1547.005](https://attack.mitre.org/techniques/T1547/005/)

**Was Juntei prüft:** Liest einzelne nicht leere LSA-Einträge unter `Security Packages` und erfasst so konfigurierte Security Support Provider.

**Untersuchte Beweise:**

| Registry-Schlüssel | Geprüfter Wert |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Security Packages` |

**Untersuchungshinweise:** Vergleichen Sie mit der erwarteten Baseline von Windows bzw. Sicherheitsprodukten und prüfen Sie jede Bibliothek. Die Konfiguration beweist nicht, dass ein Anbieter geladen wurde, Anmeldeinformationen erfasst hat oder während des Vorfalls hinzugefügt wurde.

## Vom Kandidaten zur Schlussfolgerung { #work-from-a-candidate-to-a-conclusion }

1. Öffnen Sie die ursprüngliche Quelle, die `SourceFile` angibt. Bestätigen Sie den vollständigen Wert, die XML-Aktion,
   das Skript oder den wiederhergestellten Datenbankeintrag sowie den genauen Benutzer/Host, zu dem er gehört.
2. Ermitteln Sie, ob der Mechanismus funktionieren konnte: Prüfen Sie Aktivierungsflags, Laufzeit- oder Windows-Version,
   Dienst-/Aufgabenzustand, den erforderlichen Auslöser, Benutzer, Architektur und das aktive Control Set.
3. Prüfen Sie das ausgewählte Programm, die DLL, das Skript oder die Assembly. Hashen Sie die gesicherten Bytes und prüfen Sie Argumente
   und Inhalt. Ein bekanntes Windows-Startprogramm oder eine Herausgeberzeichenfolge entscheidet nicht über die Legitimität.
4. Stellen Sie anhand unabhängiger Belege fest, was tatsächlich geschehen ist. Beispiele sind Prozesserstellung,
   Sysmon-Image-Ladevorgänge, Laufzeitdatensätze der Aufgabenplanung, Dienstaktivität, Prefetch und zugehörige Dateien.
   Ein Ereignis zu einer Konfigurationsänderung kann beweisen, dass sich die Einstellung geändert hat, ohne zu beweisen, dass ihre Payload ausgeführt wurde.
5. Erstellen Sie eine Zeitleiste anhand der Zeitstempel der ursprünglichen Quellen und der aufgezeichneten Herkunft der Korrelation.
   Behandeln Sie Schlüsselschreibvorgänge, Installationsänderungen, Datei-/Container-Schreibvorgänge und Ausführungen als getrennte Fakten.
6. Dokumentieren Sie neben den Ergebnissen auch die Grenzen: fehlende Hives, fehlende Zieldateien, unbekannte Aufgabenprinzipale,
   nicht verifizierte Signierer-Metadaten, historische per Carving gewonnene Datensätze oder einen Auslöser, der nicht bestätigt werden kann.

Zu Befehlssyntax, Profilen, allen Ausgabefeldern und Abdeckungszuständen kehren Sie zum
[Befehlsleitfaden von Persistence Hunter](../commands/persistence-hunter.md) zurück. Die
[Artefaktleitfäden](../artifacts/index.md) erläutern die zugrunde liegenden Quellen, und die
[Danksagungen](acknowledgements.md) nennen die vorgelagerten Persistenzwerkzeuge.
