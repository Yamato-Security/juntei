# Windows-Registry-Hives { #windows-registry-hives }

## Was Registry-Hives sind { #what-registry-hives-are }

Die Windows-Registry speichert System- und Benutzerkonfiguration in binären Hive-Dateien. Schlüssel bilden einen Baum;
Werte unter einem Schlüssel enthalten typisierte Daten. Registry-Artefakte beschreiben Persistenz, installierte Software,
Dienste, Konten, Netzwerkkonfiguration, Benutzeraktivität, Sicherheitseinstellungen und nahezu jeden Teil
des Windows-Zustands.

Junteis generischer Reader dekodiert jeden primären Hive, spielt Transaktionsprotokolle ein, meldet Schäden und kann
Datensätze aus gelöschten/freien Zellen, Zell-Slack, Restdaten, dem Zustand vor dem Einspielen und ausschließlich aus Transaktionsprotokollen wiederherstellen.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Der Hive enthielt den Schlüssel-/Wertzustand oder bewahrte ihn
wiederherstellbar auf. Der Zeitpunkt des letzten Schreibzugriffs auf einen Schlüssel zeigt, dass sich der Schlüssel geändert hat, nicht welcher Wert sich geändert hat oder warum.

## Gängige Hives und Speicherorte { #common-hives-and-locations }

| Hive | Typischer Pfad | Beispiele für Beweise |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | Dienste, Treiber, Steuerungssätze, Hostname, eingebundene Geräte, ShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | Installierte Software, TaskCache, Profile, Windows-Konfiguration |
| `SAM` | `C:\Windows\System32\config\SAM` | Lokale Konten und Kontometadaten |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | Metadaten zu lokalen Sicherheitsrichtlinien/Geheimnissen (abhängig von den Zugriffsrechten) |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | Konfiguration des Standard-Systemprofils |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | Benutzereinstellungen, Run-Schlüssel, RecentDocs, Anwendungsaktivität |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | Shell-Klassen, COM-/Shell-Aktivität des Benutzers |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | Anwendungsinventar (verwenden Sie `parse-registry amcache` für eine semantische Ausgabe) |

Sichern Sie die benachbarten Dateien `.LOG`, `.LOG1` und `.LOG2`. Ein bei laufendem Windows kopierter Hive kann
„dirty“ sein; Transaktionsprotokolle enthalten festgeschriebenen Zustand, den die primäre Datei allein noch nicht widerspiegelt.

## Hinweise für die Analyse { #analyst-cautions }

- Ein Zeitstempel des letzten Schreibzugriffs gilt für den **Schlüssel**, nicht für einen einzelnen Wert.
- Registry-Werte haben in der Regel keinen eigenen Zeitstempel.
- `CurrentControlSet` ist ein Alias von Analysten/Windows, der über `SYSTEM\Select` aufgelöst wird; es ist kein
  tatsächlich gespeicherter Schlüssel.
- Gelöschte Datensätze können unvollständig, veraltet, über mehrere Wiederherstellungsquellen hinweg dupliziert oder überschrieben sein.
- Datensätze, die nur in Transaktionsprotokollen vorkommen, zeigen einen historischen/zwischenzeitlichen Zustand. Erläutern Sie `recovered_from`.
- Ein Ergebnis `absent_value` bedeutet, dass der abgefragte Schlüssel existiert, ihm im geparsten Zustand aber dieser Wert fehlt;
  es beweist nicht, dass der Wert nie existiert hat.
- Zeichenfolgen, Pfade und Befehlszeilen sind Konfiguration, kein Beweis dafür, dass die konfigurierte Aktion ausgeführt wurde.
- Ein „dirty“ Hive ohne anwendbare Protokolle kann Zustand auslassen oder veralteten Zustand zeigen.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host` | Aus dem verfügbaren Systemkontext abgeleiteter Hostname; leer, wenn Juntei ihn nicht sicher ermitteln kann. |
| `HiveType` | Erkannte Hive-Kategorie (`SYSTEM`, `SOFTWARE`, `NTUSER.DAT` usw.). Zur Interpretation von Pfaden verwenden. |
| `SourcePath` | Exakter Eingabe-Hive. Unverzichtbar, wenn der Verzeichnismodus Hosts/Benutzer zusammenführt. |
| `RecordType` | `key`, `value`, `default_value` oder `absent_value`. Das Fehlen gilt nur für die Abfrage/den Zustand. |
| `KeyPath` | Pfad unterhalb des Hive-Stamms. `[unknown]` kennzeichnet eine nicht aufgelöste Herkunft gelöschter Elemente. |
| `KeyLastWrittenUTC` | Letzte Änderung des Schlüssels als Ganzes. Er kann nicht angeben, welcher Wert sich geändert hat. |
| `ValueName` | Benannter Wert; leer für einen Standardwert/unbenannten Wert. |
| `ValueType` | Registry-Typ (`REG_SZ`, `REG_DWORD`, `REG_BINARY` usw.). Der Typ beeinflusst die Interpretation. |
| `ValueData` | Typgerecht dargestellter Wert. Mehrfachzeichenfolgen bleiben in JSON Listen. |
| `DataSize` | Vom Wert aufgezeichnete Größe. Mit den Rohbytes und `Problems` vergleichen. |
| `ValueDataRaw` | Exakte Bytes, wenn die Dekodierung fehlschlägt oder `--raw-data` verwendet wird. Zur Validierung aufbewahren. |
| `ValueU64` | Vollständige numerische Darstellung für DWORD/QWORD ohne Genauigkeitsverlust durch JavaScript. |
| `ValueFiletimeUTC` | Optionale Darstellung eines QWORD als FILETIME. Nur aussagekräftig, wenn das Schema des Schlüssels FILETIME definiert. |
| `AllocationStatus` | `allocated`, `deleted_complete`, `deleted_partial`, `residual_value_only` oder `transaction_log_recovered`. Bestimmt die Belastbarkeit. |
| `RecoveredFrom` | `free_cell`, `cell_slack`, `remnant_data`, `before_replay` oder ein Protokolleintrag wie `SYSTEM.LOG1#1234`. |
| `TransactionLogStatus` | `clean`, `replayed`, `unchanged` oder `not_applied` für diesen Datensatz-/Hive-Kontext. |
| `LogsApplied` | Namen der Transaktionsprotokolldateien, die Juntei eingespielt hat. Im Bericht festhalten. |
| `Problems` | Schäden, fehlende Zellen, Typkonflikte, nicht aufgelöste Listen/übergeordnete Elemente oder andere Einschränkungen. |

## Interpretation des Wiederherstellungsstatus { #recovery-status-interpretation }

| Status | Bedeutung |
|---|---|
| `deleted_complete` | Gelöschter Schlüssel/Wert wurde mit vollständig bekanntem Pfad/Daten wiederhergestellt. Dennoch historisch, kein aktiver Zustand. |
| `deleted_partial` | Teile des Pfads, der Werteliste, des Werts oder der Daten wurden überschrieben/sind unlesbar. Füllen Sie Lücken nicht aus wiederverwendeten Zellen auf. |
| `residual_value_only` | Der Wert ist ohne ausreichenden Schlüsselkontext erhalten geblieben; `KeyPath` kann fehlen. |
| `transaction_log_recovered` | Der Zustand existiert nur in Versionen vor dem Einspielen/in Protokollversionen und nicht in aktuellen zugeordneten/freien Datensätzen. |

## Untersuchungsstrategie { #investigation-strategy }

1. Beginnen Sie mit einem exakten semantischen Schlüssel, sofern bekannt; nutzen Sie Regex-/Datensuchen zur Erkundung.
2. Prüfen Sie `TransactionLogStatus`, `AllocationStatus`, `RecoveredFrom` und `Problems` vor dem Inhalt.
3. Interpretieren Sie die Daten gemäß dem dokumentierten Schema des Schlüssels; `REG_BINARY` ist nicht selbstbeschreibend.
4. Korrelieren Sie Konfiguration mit Laufzeitbeweisen (Ereignisprotokolle, Prefetch, SRUM, Dienste/Aufgaben).
5. Bewahren Sie Rohdaten für neuartige oder strittige Felder auf.

## Weitere Informationen { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Spezifikation des Registry-Formats von Maxim Suhanov](https://github.com/msuhanov/regf)
- [Registry-Parser yarp](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [Microsoft: Registry-Werttypen](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
