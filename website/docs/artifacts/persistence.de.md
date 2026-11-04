# Registry-Autostarts { #registry-autostarts }

## Was sie sind { #what-they-are }

Windows startet Programme und lädt DLLs von vielen Registry-Orten aus: beim Booten, bei der Anmeldung, wenn ein
Programm startet oder beendet wird, wenn `cmd.exe` geöffnet wird, wenn eine COM-Klasse erstellt wird oder wenn Spooler,
LSA, netsh oder der Zeitdienst ihre Plug-ins laden. Angreifer nutzen diese Orte für Persistenz,
Rechteausweitung und DLL-Injektion, und auch legitime Software verwendet sie. Juntei liest sie
aus `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` und `UsrClass.dat` in eine einzige Tabelle. Es schreibt einen Datensatz pro
Programm, DLL oder Paket und versieht jeden mit seiner MITRE-ATT&CK-Technik.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Der Hive hat dieses Programm bzw. diese DLL
zum Starten oder Laden konfiguriert. Das beweist nicht, dass das Programm lief. Damit es ausgeführt wird, müssen die Bedingungen
erfüllt sein, die der Mechanismus nennt (eine Anmeldung, der Start des debuggten Programms, das Erstellen der Klasse), und es braucht andere
Ausführungsbelege.

## Speicherorte { #locations }

| Hive | Was gelesen wird |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run`, `RunOnce` (+`Setup`), `RunOnceEx`, `RunServices(Once)` und `Policies\Explorer\Run`, mit ihren Kopien unter `Wow6432Node` und im Terminal-Server-Installationsmodus; `Explorer\StartupApproved\Run`, `Run32` und `StartupFolder`; `Winlogon` (`Shell`, `Userinit`, `Taskman`, `AppSetup`, `GinaDLL`, … und `Notify`); `Windows` `AppInit_DLLs` mit `LoadAppInit_DLLs`; `Image File Execution Options\*` `Debugger` (mit `UseFilter`) und `VerifierDlls`; `SilentProcessExit\*` `MonitorProcess`; `Active Setup\Installed Components\*` `StubPath`; `NetSh`; `Command Processor` `AutoRun` |
| `SYSTEM` | `Control\Session Manager` (`BootExecute`, `SetupExecute`, `Execute`, `PendingFileRenameOperations`, `KnownDLLs`, `AppCertDlls`, `Environment` .NET-Profiler-Variablen); `Control\Lsa` `Authentication`, `Security` und `Notification Packages` (+`OSConfig`); `SecurityProviders`; `Control\Print` Monitore, Anbieter und Druckprozessoren; `Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | die `Run`-Schlüssel und `StartupApproved` des Benutzers; `Windows NT\CurrentVersion\Windows` `Load` und `Run`; `Winlogon` `Shell`; `Environment` (`UserInitMprLogonScript`, .NET-Profiler); `Control Panel\Desktop` `SCRNSAVE.EXE`; `PackagedAppXDebug`; `Command Processor` `AutoRun`; `Software\Classes` (als `UsrClass.dat`) |
| `UsrClass.dat` | `CLSID\*\InprocServer32` und `LocalServer32` (sowie die unter `Wow6432Node`); `shell\open\command` von Programmtypen (`exefile`, `batfile`, `scrfile`, …) und von dem, was automatisch erhöhte Programme starten (`ms-settings`, `mscfile`, `Folder`, …); `ActivatableClasses\…\DebugInformation` |

Sichern Sie jeden Hive mit seinen Transaktionsprotokollen. Benutzer-Hives liegen in `Users\<name>\NTUSER.DAT` und
`Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat`.

## Hinweise für die Analyse { #analyst-cautions }

- **Auch Standardwerte werden gemeldet.** Die meisten Datensätze auf einem sauberen System sind Windows-Standardwerte:
  - `Userinit` ist `userinit.exe`;
  - `Shell` ist `explorer.exe`;
  - `KnownDLLs`, LSA-Pakete, Druckmonitore, netsh-Hilfsprogramme und Active-Setup-Komponenten haben alle Standardeinträge.

  Suchen Sie nach Abweichungen von einem sauberen Build derselben Version.
- **`StartupApproved` deckt nur die Autostartliste des Task-Managers ab.** Es gibt an, ob ein `Run`-
  Eintrag oder eine Datei im Autostartordner dort deaktiviert ist, und seit wann. Ein Eintrag ohne passenden `Run`-
  Wert wird mit dem Detail „only in StartupApproved“ gemeldet. Er kann einen entfernten Autostart
  sichtbar machen.
- **`OverridesMachineRegistration` benötigt den `SOFTWARE`-Hive des Systems.** Eine COM-Klasse oder ein
  Befehl eines Benutzers wird damit abgeglichen, und der Hive wird auf dieselbe Weise gefunden wie der `SYSTEM`-Hive des Hosts. Eine
  Überschreibung ist das Muster von COM-Hijacking (T1546.015). OneDrive und andere benutzerbezogene Installationen
  registrieren außerdem eigene Klassen, die nichts überschreiben.
- **`ImagePath` und `Args` werden nur dort von der Befehlszeile getrennt, wo sich dies erkennen lässt.**
  - Ein in Anführungszeichen gesetztes Programm wird am schließenden Anführungszeichen getrennt.
  - Ein erstes Wort ohne Ordner gilt als Programm (`cmd /c …`).
  - Andernfalls wird am kürzesten Anfang getrennt, der mit der Erweiterung eines Programms endet.

  Ein Pfad mit Leerzeichen ohne Anführungszeichen kann ein anderes Programm ausführen (`C:\Program.exe`), sofern eines existiert.
- **`GlobalFlag` bestimmt `Enabled` für `VerifierDlls` und `MonitorProcess`.** `VerifierDlls` wird
  nur mit `0x100` geladen. `MonitorProcess` läuft nur mit `0x200` und `0x1` in `ReportingMode` von `SilentProcessExit\<program>`.
  Ein Modus von `2` (ein Dump) oder `4` (eine Benachrichtigung) führt nichts aus, und
  `Details` nennt die Flags des Modus.
- **`KeyLastWrittenUTC` ist der letzte Schreibzugriff auf den Schlüssel**, nicht auf den Wert. Mehrere Werte teilen ihn sich.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.Mechanism != "com_server" or .OverridesMachineRegistration == true)'
```

Die Zusammenfassung zählt die Datensätze nach Mechanismus. Sie listet außerdem die Einträge unter `StartupApproved\Run` und `Run32`
auf, denen ihr `Run`-Wert fehlt (Dateien im Autostartordner haben keinen, der verloren gehen könnte), sowie die COM-Klassen
und Befehle des Benutzers, die die des Computers überschreiben.

Wie jeder `parse-registry`-Befehl lehnt er ein `-o` ab, das einer der gelesenen Dateien entspricht, auch
mit `-C`. Das umfasst den Hive, seine Transaktionsprotokolle sowie die SYSTEM- und SOFTWARE-Hives des Systems,
ob über Pfad, symbolischen Link oder Hardlink. Wenn er es nicht feststellen kann, schreibt er nicht.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host-Kontext, Quell-Hive und dessen Typ. |
| `User`, `UserSID` | Bei Benutzer-Hives: der Profilordner und die SID. Die SID stammt aus dem Namen des Stammschlüssels von `UsrClass.dat`, andernfalls aus der `ProfileList` des Systems. |
| `Mechanism` | `run`, `run_once`, `winlogon`, `ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `boot_execute`, `lsa_security_package`, `com_server`, `uac_bypass_command`, … |
| `AttackTechnique` | MITRE-ATT&CK-Technik, sofern eine passt (`T1547.001`, `T1546.012`, …). |
| `KeyPath`, `ValueName` | Der Schlüssel unterhalb des Hive-Stamms (Namen wie geschrieben) und der Wert (leer: der Standardwert). |
| `Data` | Der Text des Werts oder ein Element einer Liste. Leer, wenn er nicht gelesen werden kann oder wenn nur `StartupApproved` den Eintrag nennt. |
| `ImagePath`, `Args` | Das Programm bzw. die DLL und die Argumente, sofern sie sich unterscheiden lassen (siehe oben). |
| `Enabled`, `DisabledAtUTC` | Was `StartupApproved`, `LoadAppInit_DLLs`, `GlobalFlag` (mit `ReportingMode`), `UseFilter`, `IsInstalled`, das `Enabled` des Zeitanbieters oder `ScreenSaveActive` aussagt. Leer, wenn nichts davon etwas aussagt. |
| `Details` | Weiteres: was eine ausstehende Umbenennung bewirkt, der Name der COM-Klasse, `ReportingMode`, `FilterFullPath`, eine Active-Setup-`ShellComponent`, … |
| `OverridesMachineRegistration` | Bei einer COM-Klasse oder einem Befehl eines Benutzers: ob der Computer dieselbe/denselben registriert. Leer ohne den `SOFTWARE`-Hive des Systems. |
| `WOW64` | Unter `Wow6432Node`: für 32-Bit-Programme. |
| `KeyLastWrittenUTC` | Letzter Schreibzugriff auf den Schlüssel. |
| `DataRaw` | Die Bytes des Werts (hexadezimal), wenn sie kein Text oder kein gültiges UTF-16 sind. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Falsche Werttypen, nicht lesbare Daten, `StartupApproved`-Einträge mit falscher Größe. |

## Untersuchungsstrategie { #investigation-strategy }

1. Beginnen Sie mit der Zusammenfassung:
   - COM-Klassen und Befehle, die die des Computers überschreiben;
   - `StartupApproved`-Einträge ohne ihren `Run`-Wert;
   - die Mechanismen, die selten Einträge haben (`ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `uac_bypass_command`, `file_association`, `logon_script`, `dotnet_profiler`).
2. Vergleichen Sie LSA-Pakete, `BootExecute`, `KnownDLLs`, Druckmonitore und netsh-Hilfsprogramme mit einem sauberen
   Build derselben Windows-Version.
3. Lösen Sie `ImagePath` im Dateisystem auf ($MFT, Hashes und Signaturen) und prüfen Sie Pfade in Benutzer-
   profilen, `ProgramData` und `Temp`.
4. Korrelieren Sie `KeyLastWrittenUTC` mit Registry-Ereignissen aus Security und Sysmon, Prozesserstellung,
   Prefetch und Amcache.
5. Verwenden Sie `parse-registry read --recover-deleted`, um Autostarts zu finden, die aus diesen Schlüsseln entfernt wurden.

## Weitere Informationen { #further-details }

- [MITRE ATT&CK T1547 Boot or Logon Autostart Execution](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 Event Triggered Execution](https://attack.mitre.org/techniques/T1546/)
- [Microsoft: Registry-Schlüssel Run und RunOnce](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
