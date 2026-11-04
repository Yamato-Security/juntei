# LSA-Secrets { #lsa-secrets }

## Was sie sind { #what-they-are }

Die Local Security Authority (LSA) bewahrt Secrets im Hive `SECURITY` auf. Dazu gehören das
Autologon-Kennwort, die Kennwörter der Konten, unter denen Dienste ausgeführt werden, das Domänenkennwort des Computerkontos,
die DPAPI-Systemschlüssel und der Schlüssel, mit dem zwischengespeicherte Domänenanmeldungen verschlüsselt sind. Sie
bewahrt außerdem zwischengespeicherte Domänenanmeldungen auf, damit sich Domänenbenutzer ohne Domänencontroller anmelden können.

`parse-registry lsa-secrets` listet auf, welche Secrets ein System enthält und wann sie gesetzt wurden, sowie wie
viele zwischengespeicherte Anmeldungen es aufbewahrt und verwendet. **Es liest nur Namen und Zeiten: Nichts wird entschlüsselt, und
keine Bytes eines Secrets werden gelesen oder geschrieben.** Juntei liest Beweise; es extrahiert keine Anmeldeinformationen.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Das System enthielt ein Secret mit diesem Namen, gesetzt
zu diesem Zeitpunkt. Das Secret eines Dienstes (`_SC_<service>`) bedeutet, dass die LSA das Kennwort des Kontos speichert,
unter dem dieser Dienst ausgeführt wird.

## Speicherort { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| Name | Art | Was es ist |
|---|---|---|
| `DefaultPassword` | `autologon` | Das Kennwort, mit dem sich Winlogon automatisch anmeldet. |
| `_SC_<service>` | `service_account` | Das Kennwort des Kontos, unter dem `<service>` ausgeführt wird. |
| `$MACHINE.ACC` | `machine_account` | Das Kennwort des Computerkontos in seiner Domäne. |
| `DPAPI_SYSTEM` | `dpapi_system` | Die DPAPI-Schlüssel des Systems. |
| `NL$KM` | `cached_logon_key` | Der Schlüssel, mit dem die zwischengespeicherten Anmeldungen verschlüsselt sind. |
| `L$_SQSA_<SID>` | `security_questions` | Die Sicherheitsfragen eines lokalen Kontos (Windows 10 und neuer). |
| `RasDialParams!<SID>#n`, `L$_Ras…` | `ras_credentials` | Anmeldeinformationen für DFÜ und VPN. |
| `L$…`, `G$…`, `M$…` | `local`, `global`, `machine` | Andere private Daten, nach dem Präfix, das `LsaStorePrivateData` vergibt. |

Sichern Sie `SECURITY` mit den Transaktionsprotokollen sowie den System-Hive `SYSTEM`, der den
Host benennt.

## Hinweise für die Analyse { #analyst-cautions }

- **Namen und Zeiten, keine Werte.** Ob ein Wert Daten enthält, ergibt sich aus seiner Größe. Sein Inhalt
  wird nicht gelesen.
- **`DefaultPassword` bedeutet nicht Autologon.** Das Secret existiert auf den meisten Systemen, auch auf
  solchen, die sich nie automatisch anmelden. `Microsoft\Windows NT\CurrentVersion\Winlogon` in `SOFTWARE`
  (`AutoAdminLogon`, `DefaultUserName`) gibt an,
  ob Autologon konfiguriert ist. Ein kürzlich gesetzter `CurrentSetUTC` ist eine erklärungsbedürftige Änderung.
- **Dienst-Secrets:** `_SC_<service>` erscheint, wenn ein Dienst so eingerichtet ist, dass er unter einem Konto mit
  Kennwort ausgeführt wird. Vergleichen Sie es mit `parse-registry services` (`ObjectName`) und suchen Sie nach Diensten,
  die kürzlich installiert wurden oder unbekannt sind. Das Secret kann bestehen bleiben, nachdem der Dienst entfernt wurde.
- **Zeiten:** `CurrentSetUTC` und `OldSetUTC` geben an, wann Windows den aktuellen und den vorherigen
  Wert gesetzt hat. Das Kennwort von `$MACHINE.ACC` ändert sich standardmäßig alle 30 Tage, daher verschieben sich seine Zeiten.
  `KeyLastWrittenUTC` und die Schlüsselzeiten von `CurrVal`/`OldVal` geben an, wann sich das Secret zuletzt
  geändert hat.
- **Zwischengespeicherte Anmeldungen:** Ein Slot ist belegt, wenn sein Datensatz einen Benutzer benennt. Der Header des Datensatzes ist nicht
  verschlüsselt; nur die Länge seines Benutzernamens wird gelesen. `CachedLogonsKept` ist der Zähler aus `NL$Control`,
  der `CachedLogonsCount` folgt (standardmäßig 10). Belegte Slots bedeuten, dass sich Domänenbenutzer interaktiv
  angemeldet haben, und ihre Anmeldeinformationen können offline aus dem Hive geknackt werden. Die Namen der Benutzer
  sind verschlüsselt und werden nicht dekodiert. Nur validierte binäre Datensätze mit vollständigem Header und konsistenter Benutzernamenlänge zählen als belegt. Meldet `Problems` beschädigte Slots, umfasst die Anzahl belegter Slots nur validierte Slots; interpretieren Sie die beschädigten Slots nicht als leer.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

Für jeden Hive listet die Zusammenfassung jedes Secret mit seiner Art und dem Zeitpunkt auf, zu dem es gesetzt wurde. Anschließend nennt sie
die Dienste, deren Kontokennwörter die LSA speichert, und zählt die belegten Slots für zwischengespeicherte Anmeldungen.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `SourcePath` | Host (aus dem System-Hive `SYSTEM`) und der Hive `SECURITY`. |
| `Secret`, `Kind` | Der Name des Secrets und was es ist (siehe Tabelle oben); `Cache` und `cached_logons` für den Datensatz der zwischengespeicherten Anmeldungen. |
| `ServiceName`, `UserSID` | Der Dienst, für den ein `_SC_`-Secret bestimmt ist, und die SID des Benutzers, die ein Name angibt (`L$_SQSA_`, `RasDialParams`). |
| `CurrentSetUTC`, `OldSetUTC` | `CupdTime` und `OupdTime`: wann der aktuelle und der vorherige Wert gesetzt wurden. |
| `HasCurrentValue`, `HasOldValue` | Ob `CurrVal` und `OldVal` Daten enthalten. |
| `CurrentValueLastWrittenUTC`, `OldValueLastWrittenUTC`, `KeyLastWrittenUTC` | Die letzten Schreibzugriffe auf die Schlüssel `CurrVal`, `OldVal` und den Secret-Schlüssel. |
| `CachedLogons`, `CachedLogonsInUse`, `CachedLogonsKept` | Der Datensatz der zwischengespeicherten Anmeldungen: Slots, belegte Slots und der Zähler aus `NL$Control`. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Fehlerhafte oder außerhalb des gültigen Bereichs liegende Zeiten (nur Zeitmetadaten werden angezeigt), beschädigte Cache-Header, falsche Typen von Cache-Werten und Schlüssel, die nicht gelesen werden können. |

## Untersuchungsstrategie { #investigation-strategy }

1. Suchen Sie nach `_SC_`-Secrets unbekannter oder kürzlich installierter Dienste und gleichen Sie sie mit
   `parse-registry services` und Ereignis 7045 (ein Dienst wurde installiert) ab.
2. Prüfen Sie die Zeiten von `DefaultPassword` anhand der Autologon-Einstellungen von `Winlogon`.
3. Notieren Sie auf Domänenmitgliedern die belegten Slots für zwischengespeicherte Anmeldungen und wann `Cache` zuletzt geschrieben wurde.
4. Vergleichen Sie die Zeiten der Secrets mit der Zeitleiste: Ein während eines Angriffszeitraums gesetztes Secret kann
   auf ein neues Dienstkonto, Autologon oder eine Änderung des Computerkontos hinweisen.

## Weitere Informationen { #further-details }

- [Microsoft: Private data object](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft: Informationen zu zwischengespeicherten Domänenanmeldungen](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [libyal winreg-kb: Cached credentials](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
