# Sicherheitseinstellungen { #security-settings }

## Was sie sind { #what-they-are }

Angreifer ändern die Sicherheitseinstellungen von Windows, um verborgen zu bleiben und an Anmeldeinformationen zu gelangen. Sie schalten
Microsoft Defender ab oder schließen ihre Werkzeuge davon aus, lassen Windows Kennwörter im Klartext aufbewahren,
öffnen Remotedesktop, schalten die Firewall ab und bringen die PowerShell-Protokollierung zum Schweigen. Die Registry bewahrt
diese Einstellungen auf:

- **`SOFTWARE`:** Defender, seine Ausschlüsse und Richtlinien; UAC; die Richtlinien für Remotedesktop und die
  Firewall; die Ausführungsrichtlinie, Protokollierungsrichtlinien und AMSI-Anbieter von PowerShell; die Systemwiederherstellung;
  die Ereignisprotokollkanäle.
- **`SYSTEM`** (pro Control Set): `WDigest`, LSA-Schutz, Credential Guard, Restricted Admin,
  NTLM; Remotedesktop; die Firewallprofile; SMBv1; letzte Zugriffszeiten; die Größe jedes Ereignisprotokolls.
- **`NTUSER.DAT`:** die PowerShell-Ausführungsrichtlinie und Protokollierungsrichtlinien des Benutzers sowie AMSI für
  Windows Script Host.

`parse-registry security-settings` meldet jede gefundene Einstellung als Datensatz. Jeder Datensatz enthält den
Wert der Einstellung, was der Wert bedeutet und ob er den **Standard abschwächt**: ob er einen
Schutz oder eine Beweisquelle abschaltet oder unter das Niveau senkt, das aktuelles Windows (10 und 11)
ohne diesen Wert bietet.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Diese Einstellung hatte diesen Wert, als der Hive
gespeichert wurde, und der Schlüssel wurde zuletzt zu diesem Zeitpunkt geschrieben. Eine abgeschwächte Einstellung ist ein Ansatzpunkt, kein Beweis für eine
Kompromittierung. Administratoren, Gruppenrichtlinien, andere Sicherheitsprodukte und ältere Windows-Versionen setzen
einige dieser Werte ebenfalls, und die Bedeutung weist darauf hin, wo dies häufig vorkommt.

## Speicherort { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Defender[\Real-Time Protection|\Features|\Spynet]
  Microsoft\Windows Defender\Exclusions\Paths|Extensions|Processes|IpAddresses|TemporaryPaths
  Policies\Microsoft\Windows Defender\…
  Microsoft\Windows\CurrentVersion\Policies\System                 UAC
  Policies\Microsoft\Windows NT\Terminal Services                  RDP policy
  Policies\Microsoft\WindowsFirewall\<profile>                      firewall policy
  Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell             ExecutionPolicy
  Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging|ModuleLogging|Transcription
  Microsoft\AMSI\Providers\{CLSID}
  Microsoft\Windows NT\CurrentVersion\SystemRestore                DisableSR
  Microsoft\Windows\CurrentVersion\WINEVT\Channels\<channel>       Enabled, MaxSize
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Control\SecurityProviders\WDigest                                UseLogonCredential
  Control\Lsa                                                      RunAsPPL, LsaCfgFlags, DisableRestrictedAdmin, LmCompatibilityLevel
  Control\Terminal Server[\WinStations\RDP-Tcp]                    fDenyTSConnections, PortNumber, UserAuthentication
  Services\SharedAccess\Parameters\FirewallPolicy\<profile>        EnableFirewall
  Services\LanmanServer\Parameters                                 SMB1
  Control\FileSystem                                               NtfsDisableLastAccessUpdate
  Services\EventLog\<log>                                          MaxSize
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell    ExecutionPolicy
  Software\Policies\Microsoft\Windows\PowerShell\…
  Software\Microsoft\Windows Script\Settings                       AmsiEnable
```

Sichern Sie alle drei Hives mit ihren Transaktionsprotokollen. Eine kurz vor der Sicherung geänderte Einstellung
befindet sich möglicherweise nur in den Protokollen.

## Hinweise für die Analyse { #analyst-cautions }

- **Gespeicherte Konfiguration ist nicht der wirksame Schutzzustand.** Gesicherte Registry-Werte und Diagnosezeilen für beschädigte Wertlisten
  werden gemeldet. Fehlende Daten können auf Standardwerte, nicht gesicherte Quellen oder einen anderen Verwaltungsmechanismus
  zurückgehen; sie beweisen nicht, dass ein Schutz aktiviert oder deaktiviert ist. `WeakensDefault` kennzeichnet
  Einstellungen, die die angegebene Basislinie senken können, wenn sie berücksichtigt werden.
- **Veraltete Defender-Flags werden möglicherweise ignoriert.** `DisableAntiSpyware` und `DisableAntivirus` beweisen nicht,
  dass Defender angehalten wurde. Ob sie gelten, hängt von Plattformversion, Onboarding und Manipulationsschutz
  ab. [Microsoft erläutert diese veralteten Einstellungen](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware).
- **Moderne verwaltete Ausschlüsse sind in Registry-Sicherungen möglicherweise nicht verfügbar.** Ab Plattformversion
  4.18.25110.6 erfordern Geräte mit Konfigurationsverwaltung über Defender for Endpoint unterstützte
  Defender-PowerShell-Cmdlets, um die Ausschlusskonfiguration abzurufen. Das Fehlen in der Offline-Registry
  belegt daher nicht, dass keine Ausschlüsse existieren.
  [Konfigurationshinweise von Microsoft](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings).
- **Abgeschwächt heißt nicht bösartig.** Beispiele:
  - `DisableAntiSpyware` = 1 wird auch von Windows gesetzt, während ein anderes Antivirenprogramm registriert ist.
  - Remotedesktop wird auf Servern absichtlich geöffnet.
  - Windows 7 hatte kleinere Ereignisprotokolle und aktualisierte die letzten Zugriffszeiten nicht.
- **Standardwerte hängen von der Version ab.** Einige Werte waren früher der Standard:
  - `NtfsDisableLastAccessUpdate` = 1 war vor Windows 10 1803 der Standard. Es wird
    ohne Bewertung gemeldet, doch `0x80000001`, manuell auf einer späteren Version gesetzt, gilt als abgeschwächt.
  - `RunAsPPL`, `LsaCfgFlags` und `UseLogonCredential` werden mit aktuellem Windows verglichen.
- **Defender-Ausschlüsse:** Jeder Pfad, jede Erweiterung, jeder Prozess und jede Adresse ist ein Datensatz. Jeder Ausschluss
  schwächt Defender ab. `TemporaryPaths`, die Defender selbst hinzufügt, erhalten keine Bewertung. Suchen Sie nach für Benutzer beschreibbaren Ordnern,
  `.ps1`-/`.exe`-Erweiterungen und LOLBins (`rundll32.exe`, `powershell.exe`).
- **Richtlinienschlüssel** (`Policy` = True) werden von Gruppenrichtlinien, von Werkzeugen und von Angreifern geschrieben, die
  wissen, dass Richtlinien Vorrang vor den lokalen Einstellungen haben. Korrelieren Sie sie mit den GPOs der Domäne.
- **Ereignisprotokollkanäle:** Nur 25 sicherheitsrelevante Kanäle werden gemeldet, jeweils mit der Angabe, ob Windows
  sie standardmäßig aktiviert. `TaskScheduler/Operational` und `DNS-Client/Operational` beispielsweise
  sind standardmäßig deaktiviert. Verwenden Sie `parse-registry read` für die übrigen.
- **Schlüsselzeiten:** `KeyLastWrittenUTC` gehört zum Schlüssel und ist daher nicht notwendigerweise der Zeitpunkt, zu dem sich dieser Wert
  geändert hat. Dass ein ganzer `Policies`-Schlüssel auf einmal geschrieben wird, ist üblich.
- **Control Sets:** Standardmäßig wird das zuletzt von Windows verwendete Control Set gelesen. `--all-control-sets`
  liest alle.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use
      --weakened-only     Only settings that weaken the default

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry security-settings -d evidence/ -o settings.csv
juntei parse-registry security-settings --weakened-only -d evidence/ -t jsonl \
  | jq -r '[.Host, .Area, .KeyPath, .ValueName, .Value, .Meaning] | @tsv'
```

Die Zusammenfassung zählt die Einstellungen nach Bereich. Sie listet die Einstellungen auf, die den Standard abschwächen, sowie jeden
Defender-Ausschluss.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, der Hive und sein Typ (`SOFTWARE`, `SYSTEM`, `NTUSER.DAT`). |
| `ControlSet`, `IsCurrentControlSet` | Für `SYSTEM`: das Control Set und ob es das zuletzt von Windows verwendete ist; unbekannt, wenn `Select\Current` fehlt oder ungültig ist. |
| `User`, `UserSID` | Für `NTUSER.DAT`: der Name des Profilordners und die SID des Benutzers aus der `ProfileList` des Systems. |
| `Area` | `defender`, `uac`, `credentials`, `remote_access`, `network`, `powershell`, `evidence`. |
| `Setting` | Um welche Einstellung es sich handelt: `defender_disable_realtime_monitoring`, `defender_exclusion_path`, `uac_local_account_token_filter_policy`, `wdigest_use_logon_credential`, `rdp_deny_connections`, `powershell_script_block_logging`, `event_channel_enabled`, … |
| `Value` | Der Wert: eine Dezimalzahl, eine Zeichenfolge oder der Ausschluss selbst. Leer, wenn der Wert nicht den Typ der Einstellung hat. |
| `Meaning` | Was der Wert bedeutet. |
| `WeakensDefault` | `True`, wenn der Wert abschwächt, was aktuelles Windows ohne ihn tut, `False`, wenn nicht, leer, wenn dies nicht zutrifft (eine Größe, ein Port, ein AMSI-Anbieter) oder nicht feststellbar ist. |
| `Policy` | Ob der Schlüssel unter einem `Policies`-Schlüssel liegt. |
| `KeyPath`, `ValueName` | Schlüssel und Wert. Der Wertname eines Ausschlusses ist das ausgeschlossene Element; ein AMSI-Anbieter hat keinen. |
| `KeyLastWrittenUTC` | Der letzte Schreibzugriff auf den Schlüssel. |
| `DataRaw` | Die Bytes des Werts (hex), wenn sie nicht den Typ der Einstellung haben. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Werte mit falschem Typ und Schlüssel, die nicht gelesen werden können. |

## Untersuchungsstrategie { #investigation-strategy }

1. Führen Sie das Kommando mit `--weakened-only` auf jedem Host aus und vergleichen Sie die Hosts miteinander: Eine Einstellung,
   die nur ein Host hat, fällt auf.
2. Prüfen Sie jeden Defender-Ausschluss anhand der auf dem Host und in den Defender-Protokollen gefundenen Werkzeuge
   (Ereignis 5007 zeichnet Konfigurationsänderungen auf).
3. Korrelieren Sie Schlüsselzeiten mit Anmeldungen, Dienstinstallationen und Prozesserstellung. Suchen Sie nach
   Befehlszeilen mit `reg.exe`, `Set-MpPreference`, `netsh advfirewall` und `powershell` (Ereignis
   4688, Sysmon 1 und 13, PowerShell 4104).
4. Suchen Sie bei Einstellungen für Anmeldeinformationen (`WDigest`, `RunAsPPL`, `LocalAccountTokenFilterPolicy`,
   `DisableRestrictedAdmin`) anschließend nach Credential Dumping und Pass-the-Hash.
5. Verwenden Sie `parse-registry read --recover-deleted` für Werte, die gelöscht wurden, um einen Standard wiederherzustellen, und
   `--all-control-sets` für ältere Control Sets.

## Weitere Informationen { #further-details }

- [Microsoft: Konfigurieren und Überprüfen von Ausschlüssen für Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus)
- [Microsoft: Einstellungen und Konfiguration der Benutzerkontensteuerung](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft: Konfigurieren des zusätzlichen LSA-Schutzes](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft: about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001: Impair Defenses: Disable or Modify Tools](https://attack.mitre.org/techniques/T1562/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
