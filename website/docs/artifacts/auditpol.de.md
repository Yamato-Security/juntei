# Überwachungsrichtlinie { #audit-policy }

## Was es ist { #what-it-is }

Die Überwachungsrichtlinie legt fest, welche Sicherheitsereignisse Windows im Security-Ereignisprotokoll aufzeichnet. Sie wird
unter Windows Vista und später pro Unterkategorie festgelegt (59 unter Windows 10 und 11), unter
Windows XP und Server 2003 pro Kategorie. Jede Einstellung protokolliert Erfolge, Fehler, beides oder nichts.

LSA speichert die geltende Richtlinie im `SECURITY`-Hive, unabhängig davon, ob sie aus `auditpol`, der lokalen
Sicherheitsrichtlinie oder einer Gruppenrichtlinie stammt.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Das System war so eingestellt, dass es diese Ereignisse protokolliert (oder
nicht), als der Hive zuletzt geschrieben wurde. Wird eine Unterkategorie nicht überwacht, sagt ein fehlendes Ereignis
nichts darüber aus, ob die Aktivität stattgefunden hat.

## Speicherort { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

Windows Vista und später speichern einen 12-Byte-Header, dann eine Einstellung pro Unterkategorie, Kategorie für
Kategorie, und anschließend, wie viele Unterkategorien jede Kategorie hat. Die Versionen unterscheiden sich nur in diesen Anzahlen:
Jede fügt Unterkategorien am Ende einer Kategorie hinzu (56 bei den gelesenen Hives von Windows 8.x und Server 2012 R2,
59 unter Windows 10, 11 und Server 2022). Windows XP und Server 2003 speichern, ob die Überwachung
aktiviert ist (ein Byte, gefolgt von Füllbytes, die Windows nicht löscht), und neun Kategorieeinstellungen. Dieses Layout wurde
am öffentlichen Windows-XP-Image des Hacking Case aus NISTs CFReDS geprüft, das nichts überwacht; die
Werte seiner Kategorieeinstellungen folgen der Dokumentation von Microsoft.

Sichern Sie `SECURITY` mit seinen Transaktionsprotokollen sowie den `SYSTEM`-Hive des Systems, der den
Host benennt.

## Hinweise für die Analyse { #analyst-cautions }

- **Unbekannte Daten bleiben unbekannt.** Nicht unterstützte Revisionen werden mit Diagnosen beibehalten. Unbekannte
  Einstellungsflags lassen `Success` und `Failure` leer und werden aus den Zählungen überwachter Einstellungen ausgeschlossen.
  Eine beschädigte Zähltabelle kann das reservierte Wort nicht zu einer weiteren Überwachungseinstellung machen.
  Sicherungen können zusätzliche Slots jenseits des Namens-/GUID-Katalogs enthalten. Deren Einstellungen und
  numerische `SubcategoryId` werden mit leeren Namen/GUIDs beibehalten; ein nicht zugeordneter Slot ist für sich genommen
  kein Hinweis auf Beschädigung.


- **Die Reihenfolge ist die von Windows.** Innerhalb einer Kategorie stehen die Einstellungen in der Reihenfolge der
  Meldungs-IDs der Unterkategorien, nicht ihrer GUIDs. Special Logon (`%%12548`, Index 4 von Logon/Logoff)
  steht vor IPsec Quick Mode. Tools, die sie in GUID-Reihenfolge lesen, melden für die Standardeinstellung von Windows 10
  Special Logon als „No Auditing“ und IPsec Quick Mode als „Success“. `SubcategoryId` ist
  dieselbe ID, die Ereignis 4719 (System audit policy was changed) angibt, sodass sich beide abgleichen lassen.
- **Die geltende Richtlinie, nicht ihre Quelle.** Gruppenrichtlinie und lokale Richtlinie schreiben denselben Wert.
  Die `audit.csv` der Gruppenrichtlinie (in SYSVOL und `C:\Windows\System32\GroupPolicy`) zeigt, was
  angewendet werden sollte (`Machine\Microsoft\Windows NT\Audit\audit.csv` einer Richtlinie).
- **Änderungen im Zeitverlauf:** `KeyLastWrittenUTC` gibt ungefähr an, wann sich die Richtlinie zuletzt geändert hat. Ereignis 4719
  protokolliert jede Änderung (solange das Security-Protokoll sie aufbewahrt). Ein Angreifer kann die Überwachung deaktivieren
  und anschließend wieder aktivieren.
- **Unterkategorien gegenüber Kategorien.** Unter Windows Vista und später verwendet Windows die Unterkategorien
  (es sei denn, die Sicherheitsoption `SCENoApplyLegacyAuditPolicy` ist deaktiviert und eine ältere
  Kategorierichtlinie setzt sie außer Kraft).
- **Windows XP und 2003:** Bei deaktivierter Überwachung (`AuditingEnabled` False) wird keine Kategorie überwacht,
  unabhängig von ihrer Einstellung.
- **Ereignisse:** `Events` listet die wichtigsten Ereignisse jeder Unterkategorie gemäß der Microsoft-Dokumentation
  der erweiterten Überwachungsrichtlinieneinstellungen auf. Das sind die Ereignisse, nach denen eine Untersuchung üblicherweise sucht, nicht
  jedes Ereignis, das die Unterkategorie protokolliert.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

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
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.Setting == "none") | .Subcategory'
```

Für jeden Hive gibt die Zusammenfassung das Layout an, wie viele Einstellungen überwacht werden und wann die Richtlinie
zuletzt geschrieben wurde. Sie listet auf, was bei Erfolg und Fehler, nur bei Erfolg und nur bei Fehler
überwacht wird, danach die nicht überwachten Unterkategorien mit ihren wichtigsten Ereignissen: Diese Ereignisse sind
in der gesicherten Richtlinie deaktiviert; daraus lässt sich nicht ableiten, welche Richtlinie während früherer Aktivitäten galt.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `SourcePath` | Host (aus dem `SYSTEM`-Hive des Systems) und der `SECURITY`-Hive. |
| `Layout` | `subcategories` (Windows Vista und später) oder `categories` (XP und 2003); leer, wenn sich der Wert nicht dekodieren lässt. |
| `Category`, `Subcategory` | So, wie `auditpol` sie benennt. |
| `SubcategoryGUID`, `SubcategoryID` | Die GUID der Unterkategorie und ihre Meldungs-ID, wie Ereignis 4719 sie angibt (`%%12548`). |
| `Setting` | `none`, `success`, `failure` oder `success_and_failure`. |
| `Success`, `Failure` | Ob Erfolge und Fehler protokolliert werden. |
| `AuditingEnabled` | Windows XP und 2003: ob die Überwachung überhaupt aktiviert ist. |
| `Events` | Die wichtigsten Ereignis-IDs der Unterkategorie. |
| `KeyLastWrittenUTC` | Letzter Schreibzugriff auf `PolAdtEv`: ungefähr der Zeitpunkt der letzten Richtlinienänderung. |
| `DataRaw` | Die Bytes des Werts (hexadezimal), wenn er sich nicht vollständig dekodieren lässt. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Ein unbekanntes Layout oder eine unbekannte Größe, nicht aufgehende Anzahlen und Werte über 3. |

## Untersuchungsstrategie { #investigation-strategy }

1. Bevor Sie schließen, dass etwas nicht geschehen ist, prüfen Sie, ob seine Unterkategorie überwacht wurde:
   Process Creation (4688), Logon (4624, 4625), Special Logon (4672), Other Object Access Events
   (geplante Aufgaben, 4698), Security System Extension (Dienstinstallationen, 4697), User Account und
   Security Group Management (4720, 4732), File Share (5140), Kerberos (4768, 4769).
2. Vergleichen Sie die Richtlinie mit dem Security-Protokoll: Ereignisse von Unterkategorien, die jetzt nicht überwacht werden,
   bedeuten, dass sich die Richtlinie geändert hat. Suchen Sie nach Ereignis 4719 und nach einem aktuellen `KeyLastWrittenUTC`.
3. Vergleichen Sie mit der `audit.csv` der Gruppenrichtlinie, um eine domänenweite Richtlinie von einer lokalen Änderung zu unterscheiden.
4. Verwenden Sie `parse-registry read --recover-deleted` und die Transaktionsprotokolle für frühere Versionen von
   `PolAdtEv`.

## Weitere Informationen { #further-details }

- [Microsoft: Erweiterte Einstellungen der Sicherheitsüberwachungsrichtlinie](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft: Überwachungskonstanten (GUIDs der Unterkategorien)](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft: 4719(S) System audit policy was changed](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
