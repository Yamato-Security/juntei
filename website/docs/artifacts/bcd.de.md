# Startkonfigurationsdaten (BCD) { #boot-configuration-data-bcd }

## Was der BCD-Speicher ist { #what-the-bcd-store-is }

Ab Windows Vista wird die Startkonfiguration in einem Registry-Hive gespeichert, dem BCD-Speicher:
`\Boot\BCD` auf BIOS-Systemen, `\EFI\Microsoft\Boot\BCD` auf der EFI-Systempartition von UEFI-Systemen.
`bcdedit` und das Windows-Installationsprogramm bearbeiten ihn; der Start-Manager liest ihn bei jedem Start.

Der Speicher enthält Objekte, die jeweils nach einer GUID benannt sind:

- den **Windows-Start-Manager** (`{bootmgr}`) mit dem Standardeintrag, der Anzeigereihenfolge und dem
  Menü-Timeout; auf UEFI-Systemen zusätzlich den **Firmware-Start-Manager** (`{fwbootmgr}`) mit der
  Startreihenfolge der Firmware;
- einen **Windows-Startlader** pro Windows-Installation (einschließlich der Wiederherstellungsumgebung), eine
  **Fortsetzungsanwendung** (Ruhezustand) und den **Speichertester**;
- Objekte mit **geerbten Einstellungen** (`{globalsettings}`, `{bootloadersettings}`, …), deren Einstellungen die
  anderen Objekte erben;
- **Geräteobjekte** (RAM-Disk-Optionen der Wiederherstellungsumgebung).

Jedes Objekt besitzt Elemente: eine Beschreibung, ein Gerät und einen Pfad sowie Einstellungen wie
`recoveryenabled`, `bootstatuspolicy`, `testsigning`, `nointegritychecks` und `safeboot`.

Zwei Arten von Angriffen verändern sie:

- **Ransomware** deaktiviert die Wiederherstellung, damit das System nicht repariert werden kann:
  `bcdedit /set {default} recoveryenabled No` und
  `bcdedit /set {default} bootstatuspolicy ignoreallfailures`. Manche Familien setzen `safeboot`, damit
  Windows im abgesicherten Modus neu startet, in dem Sicherheitsprodukte nicht ausgeführt werden.
- **Rootkits und unsignierte Treiber** können `testsigning on` oder `nointegritychecks on` nutzen, um Prüfungen beim Start abzuschwächen. Diese Einstellungen allein beweisen nicht, dass ein Rootkit ausgeführt wurde.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Zum Zeitpunkt der Sicherung hatte dieser Starteintrag
diese Einstellung. Der Zeitpunkt des letzten Schreibzugriffs auf den Schlüssel des Elements gibt an, wann die Einstellung zuletzt geschrieben wurde.

## Speicherort { #location }

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

Der Typ eines Elements besteht aus 8 Hexadezimalziffern: seiner Klasse (`1` jede Anwendung, `2` eine bestimmte Art von
Anwendung, `3` Geräte), seinem Format (`1` Gerät, `2` Zeichenfolge, `3` Objekt, `4` Objektliste, `5`
Ganzzahl, `6` Boolescher Wert, `7` Ganzzahlliste) und seiner Nummer. Der Befehl benennt Elemente so, wie
Microsoft sie dokumentiert (`BcdLibraryBoolean_AutoRecoveryEnabled`).

Sicherungswerkzeuge kopieren den Speicher häufig nicht: Die EFI-Systempartition ist nicht eingebunden, und `\Boot`
ist versteckt. Sichern Sie ihn explizit oder binden Sie die Partition ein. `-d` liest auch `BCD-Template`
(`HiveType` gibt an, welche Datei vorliegt); die Vorlage ist der Ausgangspunkt von Windows, nicht der Speicher, mit dem es startet.

## Hinweise für die Analyse { #analyst-cautions }

- **Einstellungen werden geerbt.** Ein Startlader erbt von den Objekten in seiner
  `BcdLibraryObjectList_InheritedObjects` (meist `{bootloadersettings}`, das wiederum
  `{globalsettings}` erbt). `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks`, `SafeBoot` und
  `BootStatusPolicy` sind aufgelöste gespeicherte Werte (eigene, andernfalls geerbte), und `Inherited` gibt an, welche
  aus einem anderen Objekt stammen. Die Vererbung wird über höchstens acht Ebenen verfolgt, Zyklen werden begrenzt. Ein referenziertes Objekt kann in einem unvollständigen Speicher fehlen. Leere Felder belegen keinen wirksamen Windows-Standardwert. Firmware und Secure Boot können außerdem verhindern, dass eine gespeicherte Option wirksam wird.
- **Nicht gesetzt heißt nicht deaktiviert.** Ein leeres `RecoveryEnabled` bedeutet, dass das Element nicht vorhanden ist; es gilt der
  Windows-Standardwert.
- **Zeitstempel gelten pro Element.** Der Schlüsselzeitstempel jedes Elements (`key_last_written` in `Elements`) gibt an, wann es
  zuletzt geschrieben wurde. Windows schreibt manche Elemente bei Updates und Upgrades neu; vergleichen Sie mit den Zeitstempeln der anderen
  Elemente.
- **Geräte werden nur teilweise dekodiert.** Eine GPT-Partition wird durch die GUIDs ihrer Partition und ihres Datenträgers angegeben,
  eine RAM-Disk durch ihr Optionsobjekt. Andere Geräte behalten nur ihre Bytes (`data_raw`).
- **Undokumentierte Elemente** (`0x15000065`, die Klasse `0x4…`) haben keinen Namen und behalten ihre Bytes.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.TestSigning == true or .NoIntegrityChecks == true or .RecoveryEnabled == false)'
```

Die Zusammenfassung listet den Standardeintrag jedes Speichers sowie jede Einstellung zu Testsignierung, Integritätsprüfung, Wiederherstellung,
Startstatusrichtlinie (jedes `Ignore…`) und abgesichertem Modus auf, jeweils mit dem Zeitpunkt, zu dem sie gesetzt wurde, oder dem Objekt,
von dem sie geerbt wird.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, der Hive sowie `BCD` oder `BCD-Template`. |
| `ObjectGUID`, `ObjectType`, `ObjectTypeName` | Die GUID des Objekts, sein Typ (`0x10200003`) und was es ist (`Windows boot loader`, `inherited settings for Windows boot loader`, …). |
| `Description`, `Path`, `Device` | Die Beschreibung, der Pfad der Anwendung und ihr Gerät (eine GPT-Partition und ein Datenträger oder das Optionsobjekt einer RAM-Disk). |
| `IsDefault`, `DisplayOrder` | Ob der Windows-Start-Manager es standardmäßig startet, und seine Position im Startmenü. |
| `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks` | `recoveryenabled`, `testsigning`, `nointegritychecks`, wie aus dem Speicher aufgelöst. |
| `SafeBoot`, `BootStatusPolicy` | `safeboot` (`Minimal`, `Network`, `DsRepair`) und `bootstatuspolicy` (`IgnoreAllFailures`, …), jeweils mit der Nummer. |
| `Inherited` | Welche dieser Werte aus einem anderen Objekt stammen, und aus welchem. |
| `Elements` | Jedes Element: sein Typ, Name, Wert, seine Bytes (für Geräte, undokumentierte Typen und nicht dekodierbare Daten) und sein Schlüsselzeitstempel. |
| `KeyLastWrittenUTC` | Der letzte Schreibzugriff auf den Schlüssel des Objekts. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Objekte ohne Typ, Elemente, deren Daten nicht zu ihrem Format passen. |

## Untersuchungsstrategie { #investigation-strategy }

1. Prüfen Sie den Standardeintrag und jeden Startlader auf `TestSigning`, `NoIntegrityChecks`,
   `RecoveryEnabled = false`, eine `Ignore…`-`BootStatusPolicy` und `SafeBoot`.
2. Datieren Sie die Änderung anhand der Elementzeitstempel und korrelieren Sie sie mit `bcdedit.exe` in Prefetch,
   Amcache, Prozesserstellungsereignissen (4688, Sysmon 1) und dem System-Ereignisprotokoll.
3. Suchen Sie nach Starteinträgen oder Geräten, die nicht vorhanden sein sollten (ein anderer Loader-Pfad, eine RAM-Disk), sowie
   nach einem geänderten Standardeintrag oder einer geänderten Anzeigereihenfolge.
4. Vergleichen Sie den Speicher mit `BCD-Template` und mit einem anderen System desselben Builds.

## Weitere Informationen { #further-details }

- [Microsoft: Referenz der BCD-Startoptionen](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft: BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft: BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft: BcdObject (Objekttypen)](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
