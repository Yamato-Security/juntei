# Installierte Programme { #installed-programs }

## Was Windows aufbewahrt { #what-windows-keeps }

Windows und Installationsprogramme zeichnen installierte Software in mehreren Registry-Listen auf. Jede Liste hat einen Schlüssel
pro Programm, Produkt, Distribution oder Paket; Juntei gibt einen Datensatz pro Schlüssel aus.

| Quelle | Hive und Schlüssel | Was sie enthält |
|---|---|---|
| `uninstall` | `SOFTWARE` und `NTUSER.DAT` `…\CurrentVersion\Uninstall\*` sowie `SOFTWARE\Wow6432Node\…` | Was die Einstellungen und *Programme und Features* auflisten: Name, Version, Herausgeber, Installationsdatum, Ordner, Deinstallationsbefehle, Größe, ob der Eintrag ausgeblendet ist |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`, `NTUSER.DAT` `Software\Microsoft\Installer\Products\*` | Windows Installer-Produkte: Produktname und Version, Dateiname des Pakets und der Ordner, aus dem es installiert wurde |
| `app_path` | `SOFTWARE` und `NTUSER.DAT` `…\CurrentVersion\App Paths\*` | Programme, die Windows über ihren Namen startet (`Win+R`, `start tool`): Pfad des Programms und sein Ordner |
| `wsl` | `NTUSER.DAT` `…\CurrentVersion\Lxss\{GUID}` | Distributionen des Windows-Subsystems für Linux: Name, Speicherort ihrer Dateien oder ihres Datenträgers, WSL 1 oder 2 |
| `package` | `SOFTWARE` `…\Appx\AppxAllUserStore` | Paketierte (Store-)Apps: für alle Benutzer bereitgestellt, Windows-eigene, bereitgestellte (staged), deprovisionierte und die jedes einzelnen Benutzers |

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Dieses Programm, Produkt oder Paket wurde
auf dem System für alle Benutzer oder für diesen Benutzer installiert (oder registriert). Das Installationsdatum ist
das, was das Installationsprogramm geschrieben hat; der letzte Schreibzugriff auf den Schlüssel ist der Zeitpunkt der letzten Änderung des Eintrags.

## Speicherort { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

Sichern Sie `SOFTWARE`, jede `NTUSER.DAT` und deren Transaktionsprotokolle. `SYSTEM` benennt den Host, und
die `ProfileList` in `SOFTWARE` liefert die SID jedes Benutzers.

## Hinweise für die Analyse { #analyst-cautions }

- **Installationsdatum und Schlüsselzeit.** `InstallDate` ist ein vom Installationsprogramm geschriebenes Datum (ohne Uhrzeit) in
  lokaler Zeit; manche Installationsprogramme schreiben es nicht, andere in einem anderen Format (wie geschrieben in
  `InstallDateRaw` beibehalten). Ein Update kann den Eintrag neu schreiben, ohne das Datum zu ändern, oder das Datum ändern,
  ohne dass sich das Programm wesentlich ändert: Vergleichen Sie `InstallDate` mit `KeyLastWrittenUTC`.
- **Deinstallierte Software.** Ein sauber entferntes Programm verliert seine Schlüssel. Verwenden Sie `parse-registry read
  --recover-deleted` für gelöschte `Uninstall`-Schlüssel sowie Amcache, Prefetch und die `$MFT` für zurückgebliebene
  Spuren.
- **Benutzerbezogene Registrierungen.** Einträge in `NTUSER.DAT` kennzeichnen Software, die für ein Profil registriert ist.
  Dieser Geltungsbereich belegt nicht, wer das Installationsprogramm ausgeführt hat oder ob Administratorrechte verwendet wurden.
- **Ausgeblendete Einträge.** `SystemComponent` blendet einen Eintrag in *Programme und Features* aus; Einträge
  ohne `DisplayName` (Updates, Komponenten) werden ebenfalls nicht angezeigt. Sie alle werden gemeldet.
- **Produktcodes.** Ein `Uninstall`-Schlüssel in GUID-Form kann über `ProductCode` einem gepackten
  `msi_product`-Schlüssel zugeordnet werden. Der Name in GUID-Form allein beweist nicht, dass
  Windows Installer den Eintrag erstellt hat.
- **App Paths.** Dies sind Registrierungen zum Auffinden von Programmen, keine Ausführungsdatensätze.
  Der Wert `Path` kann mehrere Suchverzeichnisse enthalten und ist nicht notwendigerweise das
  Installationsverzeichnis.
- **Herausgeber.** Dies sind vom Installationsprogramm gelieferte Registry-Metadaten, keine geprüften Signaturinformationen.
- **Architektur.** `Wow6432Node` enthält 32-Bit-Programme unter 64-Bit-Windows (`Architecture` `x86`,
  `Wow64` `True`); andere Einträge haben die eigene Architektur des Systems (leer). Die Architektur eines Pakets
  ist seine eigene (`x64`, `neutral`, …).
- **WSL.** `Version` ist die WSL-Version aus den `Flags` der Distribution (`0x8`: Sie läuft in der virtuellen Maschine von WSL 2).
  Der eigene `Version`-Wert der Distribution (in `Details`) ist die Version ihres Dateisystems,
  auch bei WSL-1-Distributionen 2. `BasePath` enthält ihre `ext4.vhdx` (WSL 2) bzw. `rootfs`
  (WSL 1): Sichern Sie diese, um hineinzusehen.
- **Pakete.** Die `<SID>`-Listen sind die für jeden Benutzer registrierten Pakete; die meisten gehören zu Windows
  selbst. `Deprovisioned`-Pakete wurden für neue Benutzer entfernt, `EndOfLife\<SID>`-Pakete für den jeweiligen Benutzer.
- **Beschädigte Werte.** Fehlerhafte Zahlen, falsche Werttypen und ungültiges UTF-16 werden in
  `Problems` gemeldet und in `DataRaw` beibehalten, einschließlich einer leeren Hex-Zeichenfolge für einen vorhandenen Wert mit
  null Bytes. Eine saubere leere Zeichenfolge in einem optionalen numerischen Feld wird als fehlend behandelt. Eine Paketversion
  muss aus vier Dezimalkomponenten bestehen, jede im Bereich vorzeichenloser 16-Bit-Werte.
  Warnungen auf Hive-Ebene bleiben auf stderr, wenn `-t` Datensätze nach stdout streamt.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.Scope == "user" and .Source != "package")'
```

Die Zusammenfassung listet die zuletzt installierten Programme (nach `InstallDate`), die Software, die Benutzer für
sich selbst installiert haben, und die WSL-Distributionen auf.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Host, Hive-Datei und `SOFTWARE` oder `NTUSER.DAT`. |
| `User`, `UserSID` | Benutzer eines Benutzer-Hives (sein Profilordner und die SID aus `ProfileList`) oder der Benutzer, für den ein Paket aufgeführt ist. |
| `Source`, `Scope` | Die Liste (`uninstall`, `msi_product`, `app_path`, `wsl`, `package`) und `machine` oder `user`. |
| `Name`, `Version`, `Publisher` | `DisplayName`, `DisplayVersion`, `Publisher`; `ProductName` und `Version` eines Produkts; Programmdateiname eines App-Path-Eintrags; Name und WSL-Version einer Distribution; Name und Version eines Pakets. |
| `InstallDate`, `InstallDateRaw` | `InstallDate` als Datum, wenn es als `YYYYMMDD` geschrieben ist; andernfalls wie geschrieben. |
| `KeyLastWrittenUTC` | Wann der Schlüssel des Eintrags zuletzt geändert wurde. |
| `InstallLocation`, `InstallSource` | `InstallLocation` (`Path` eines App-Path-Eintrags, `BasePath` einer Distribution, `Path` eines Pakets); `InstallSource` (zuletzt verwendeter Quellordner eines Produkts). |
| `Program` | Das Programm eines App-Path-Eintrags. |
| `UninstallCommand`, `QuietUninstallCommand` | `UninstallString`, `QuietUninstallString`. |
| `SizeKB`, `SystemComponent`, `WindowsInstaller` | `EstimatedSize` in KiB; ob der Eintrag ausgeblendet ist; ob Windows Installer die Software installiert hat. |
| `ProductCode` | Der Windows Installer-Produktcode (`{GUID}`). |
| `Architecture`, `WOW64` | `x86` unter `Wow6432Node` oder die Architektur eines Pakets; ob der Schlüssel unter `Wow6432Node` liegt. |
| `Details` | Weiteres: `DisplayIcon`, `ModifyPath`, `ParentKeyName`, `ReleaseType`, `Inno Setup: User` (das Konto, das ein Inno-Setup-Installationsprogramm ausgeführt hat); `PackageCode`, `PackageName` eines Produkts; `Flags`, `State`, `DefaultUid`, `Version`, `PackageFamilyName`, `Default` einer Distribution; `List`, `ResourceId`, `PublisherId` eines Pakets. |
| `KeyPath` | Der Schlüssel des Eintrags. |
| `DataRaw` | Die Bytes (hex) von Werten, die nicht dem entsprechen, was ihre Namen besagen. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Werte, die nicht gelesen werden konnten oder nicht dem entsprechen, was ihre Namen besagen. |

## Untersuchungsstrategie { #investigation-strategy }

1. Sortieren Sie rund um den Vorfall nach `InstallDate` und `KeyLastWrittenUTC`; achten Sie auf Fernzugriffswerkzeuge,
   Tunneling- und Dateiübertragungsclients, Archivierungsprogramme, Skript-Laufzeitumgebungen und Sicherheitswerkzeuge.
2. Prüfen Sie Datensätze mit `Scope` `user`: Software, die ohne Administratorrechte installiert wurde.
3. Prüfen Sie `InstallLocation`, `InstallSource` und `UninstallCommand` auf für Benutzer beschreibbare Ordner
   (`Downloads`, `AppData`, `Temp`, `ProgramData`).
4. Suchen Sie nach WSL-Distributionen und sichern Sie deren `BasePath`.
5. Korrelieren Sie mit Amcache und Prefetch (Ausführung des Installationsprogramms und des Programms), der `$MFT` (Erstellung des
   Installationsordners) und dem Anwendungsereignisprotokoll (MsiInstaller-Ereignisse).

## Weitere Informationen { #further-details }

- [Microsoft: Registrierungsschlüssel „Uninstall“](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key)
- [Microsoft: Anwendungsregistrierung (App Paths)](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft: Übersicht über die Paketidentität](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
