# USB-Geräte { #usb-devices }

## Was der USB-Verlauf ist { #what-the-usb-history-is }

Windows zeichnet jedes Gerät auf, das es installiert. Für USB-Speicher, andere USB-Geräte und tragbare
Geräte (Telefone, Kameras, Mediaplayer) speichert der `SYSTEM`-Hive einen Schlüssel pro Gerät mit Name,
Seriennummer und den Zeitpunkten, zu denen Windows es installiert, zuletzt dessen Anschluss und zuletzt dessen Entfernung registriert hat.
`MountedDevices` zeichnet die Volume-GUIDs und Laufwerksbuchstaben auf, die ihren Volumes zugewiesen wurden. Der
`SOFTWARE`-Hive speichert die Namen, die der Explorer tragbaren Geräten und Volumes gegeben hat, sowie die Aufzeichnung von
Volume-Seriennummern durch ReadyBoost. Die `NTUSER.DAT` jedes Benutzers zeichnet die Volumes auf, die dieser Benutzer
eingebunden hatte.

`parse-registry usb` führt sie zu einem Datensatz pro Gerät und Volume zusammen: welches Gerät es war, wann
es installiert und angeschlossen wurde, welchen Laufwerksbuchstaben und welches Volume es hatte und welche Benutzer dieses
Volume eingebunden hatten.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Ein Gerät mit diesem Hersteller, Produkt und dieser
Seriennummer war an dieses System angeschlossen, erstmals und zuletzt zu diesen Zeitpunkten; sein Volume hatte diesen
Laufwerksbuchstaben; diese Benutzer hatten es eingebunden. Registry-Datensätze zeigen nicht, welche Dateien kopiert wurden.

## Speicherort { #location }

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| Geräteeigenschaft | Feld | Bedeutung (`devpkey.h`) |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate`: wann sein Treiber zuletzt installiert wurde |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate`: wann das Gerät erstmals installiert wurde |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate`: wann es zuletzt angeschlossen wurde (Windows 8 und später) |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate`: wann es zuletzt entfernt wurde (Windows 8 und später) |

Windows 8 und später speichern jede Zeit als Standardwert des Schlüssels `0064` … `0067`. Windows 7
speichert sie in `00000064\00000000\Data`. Juntei liest beides.

Viele DFIR-Referenzen bezeichnen `0064` als Erstinstallation und `0065` als Installation. Juntei folgt
den oben genannten Definitionen aus Microsofts `devpkey.h` und benennt die Spalten nach dem, was jede Zeit aufzeichnet.

Übergeben Sie `-f` oder `-d` den `SYSTEM`-Hive. Der `SOFTWARE`-Hive des Systems und die `NTUSER.DAT`-
Hives seiner Benutzer werden daneben gefunden, so wie eine Sicherung sie ablegt (`Windows\System32\config\SOFTWARE`,
`Users\<user>\NTUSER.DAT`), oder im selben Verzeichnis und eine Ebene darunter, wenn die Hives gemeinsam
exportiert wurden. Sichern Sie sie alle mit ihren Transaktionsprotokollen.

## Wie die Datensätze zusammengeführt werden { #how-the-records-are-joined }

- **Geräte:** ein Datensatz pro Geräteschlüssel: `usbstor`, `usb` und `wpdbusenum` (`DeviceClass`). Ein
  USB-Speichergerät übernimmt Hersteller- und Produkt-ID von dem USB-Gerät mit derselben
  Container-ID (andernfalls derselben Seriennummer).
- **Volumes:** Die Volumes eines USB-Speichergeräts sind die `\??\Volume{GUID}`-Werte von
  `MountedDevices`, deren Daten ein Pfad sind, der das Gerät benennt. Ein Gerät mit zwei Volumes hat zwei
  Datensätze. Seine Laufwerksbuchstaben sind die `\DosDevices\X:`-Werte mit denselben Daten.
- **Namen und Seriennummern:** Volume-Namen stammen aus `Windows Portable Devices` und von den
  tragbaren Geräten, die dasselbe Gerät benennen. Volume-Bezeichnungen und Seriennummern stammen aus
  `EMDMgmt`-Schlüsseln, die es benennen.
- **Benutzer:** die Benutzer, deren `MountPoints2` die GUID des Volumes enthält, mit der Zeit des letzten Schreibzugriffs auf diesen Schlüssel.
- **Volumes ohne Geräteschlüssel** (`volume`): ein `MountedDevices`-Volume, das kein Geräteschlüssel benennt (das einer
  internen Festplatte oder eines entfernten Geräts, dessen `Enum`-Schlüssel verschwunden ist), ein Laufwerksbuchstabe, dessen Daten mit keinem Volume-GUID-
  Wert übereinstimmen, und ein Volume, das nur in `MountPoints2` von Benutzern vorkommt.
- **Geräte, die nur `SOFTWARE` benennt** (`portable_device`, `readyboost`).

`SourceKeys` listet jeden Schlüssel und Wert auf, aus dem ein Datensatz zusammengeführt wurde, mit seinem Hive.

## Hinweise für die Analyse { #analyst-cautions }

- **Laufwerksbuchstaben werden wiederverwendet.** `MountedDevices` speichert das letzte Gerät, das jeden Buchstaben erhalten hat. Ein
  Buchstabe wird nur bei dem Gerät angezeigt, dessen Daten noch die dieses Buchstabens sind.
- **Seriennummern können erfunden sein.** Ist das zweite Zeichen einer Instanz `&`, hat Windows
  sie erzeugt, weil das Gerät keine Seriennummer meldet; `Serial` ist dann leer. Eine solche
  Instanz kann auf einem anderen Computer abweichen.
- **Instanzen von USB-Speichergeräten enden mit der logischen Einheit** (`&0`). `Serial` lässt sie weg; `Instance`
  behält sie bei.
- **Die Zeiten unterscheiden sich je nach Windows-Version.** Windows 7 hat nur die Installationszeiten. `LastArrivalUTC` und
  `LastRemovalUTC` sind nur der letzte Anschluss, nicht jeder Anschluss. Ereignisprotokolle
  (`Microsoft-Windows-Partition/Diagnostic` 1006, Kernel-PnP, `DriverFrameworks-UserMode`) und
  `setupapi.dev.log` liefern mehr.
- **Control Sets:** Standardmäßig wird das zuletzt von Windows verwendete Control Set gelesen. `--all-control-sets`
  liest jedes Control Set; ein älteres kann noch ein Gerät auflisten, das das aktuelle nicht mehr hat.
- **Die Zeit des letzten Schreibzugriffs auf einen Schlüssel ist keine Anschlusszeit.** Windows schreibt Geräteschlüssel aus vielen
  Gründen neu.
- **Entfernte Geräte hinterlassen Spuren.** Ein Volume-Datensatz mit Hersteller und Produkt, aber ohne Geräteschlüssel,
  ist ein Gerät, dessen `Enum`-Schlüssel entfernt wurde (durch eine Deinstallation oder durch Bereinigungstools) und dessen Volume
  `MountedDevices` noch enthält.
- **ReadyBoost-Seriennummern** sind die Seriennummer des Volumes (`VolumeSerials`, wie `dir` sie
  anzeigt), die auch LNK-Dateien und Jump Lists aufzeichnen.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.DeviceClass == "usbstor") | {Vendor, Product, Serial, LastArrival, DriveLetters, Users}'
```

Die Zusammenfassung listet die zuletzt angeschlossenen USB-Speichergeräte (neueste zuerst) mit ihren Laufwerks-
buchstaben auf sowie die Anzahl der Volumes, die zu USB-Speichergeräten ohne Geräteschlüssel gehören.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host` | Der Name des Computers, aus `SYSTEM`. |
| `DeviceClass` | `usbstor`, `usb`, `wpdbusenum` oder, ohne Geräteschlüssel, `volume`, `portable_device` oder `readyboost`. |
| `ControlSet` | Das Control Set des Geräteschlüssels. |
| `Vendor`, `Product`, `Revision` | USB-Speicher: aus dem Namen des Geräts (`Disk&Ven_…&Prod_…&Rev_…`), auch für ein Gerät, das ein Volume- oder `SOFTWARE`-Schlüssel benennt. |
| `Vid`, `PID` | Die USB-Hersteller- und -Produkt-IDs. |
| `Serial`, `Instance` | Die Seriennummer und der Name des Instanzschlüssels wie geschrieben. |
| `FriendlyName`, `DeviceDesc`, `Service`, `ContainerID` | Die Werte des Geräteschlüssels wie geschrieben (`DeviceDesc` ist oft `@<inf>,%…%;<text>`). Der `FriendlyName` eines tragbaren Geräts ist meist die Bezeichnung oder das Laufwerk seines Volumes. |
| `DeviceFirstInstallUTC`, `DriverLastInstalledUTC`, `LastArrivalUTC`, `LastRemovalUTC` | Die Zeiten aus den Geräteeigenschaften (siehe oben). |
| `DriveLetters` | Die Laufwerksbuchstaben, deren `MountedDevices`-Daten denen des Volumes entsprechen. |
| `VolumeGUID`, `VolumeData` | Die GUID des Volumes und worauf es laut seinen `MountedDevices`-Daten liegt: ein Gerätepfad, eine MBR-Datenträgersignatur mit Partitionsoffset oder eine GPT-Partition. |
| `VolumeNames`, `VolumeSerials` | Namen, die der Explorer dem Gerät oder seinem Volume gegeben hat, ReadyBoost-Bezeichnungen und Volume-Seriennummern (`1234-ABCD`). |
| `User` | Die Benutzer, deren `MountPoints2` das Volume enthält, mit dem letzten Schreibzugriff auf diesen Schlüssel (JSON: `User`, `MountPoints2LastWritten`, `SourcePath`). |
| `FoundIn`, `SourceKeys` | Wo es gefunden wurde (`Enum`, `MountedDevices`, `Windows Portable Devices`, `EMDMgmt`, `MountPoints2`), und jeder Schlüssel und Wert, aus dem es zusammengeführt wurde. |
| `KeyPath`, `KeyLastWrittenUTC` | Der Instanzschlüssel des Geräts und sein letzter Schreibzugriff. |
| `SourcePath`, `SoftwarePath` | Der `SYSTEM`-Hive und der `SOFTWARE`-Hive, sofern er etwas beigetragen hat. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung der Hives, aus denen der Datensatz zusammengeführt wurde. |
| `Problems` | Schlüssel, die nicht gelesen werden konnten, und Geräteeigenschaften, die keine FILETIMEs sind (mit ihren Bytes). |

## Untersuchungsstrategie { #investigation-strategy }

1. Sortieren Sie USB-Speichergeräte nach `LastArrivalUTC` (andernfalls `DriverLastInstalledUTC`) und klammern Sie
   die eigenen Lesegeräte und Hubs des Computers aus.
2. Notieren Sie Hersteller, Produkt, Seriennummer, Volume-GUID, Laufwerksbuchstabe und Volume-Seriennummer jedes Geräts
   und suchen Sie danach in LNK-Dateien, Jump Lists, ShellBags und der `$MFT` (Pfade auf diesem
   Laufwerksbuchstaben oder Volume).
3. Prüfen Sie `User`, um festzustellen, wer das Volume eingebunden hatte und wann (letzter Schreibzugriff auf den `MountPoints2`-Schlüssel).
4. Suchen Sie nach `volume`-Datensätzen, die ein USB-Speichergerät benennen: Geräte, deren Schlüssel entfernt wurden.
5. Korrelieren Sie mit den Ereignisprotokollen für jeden Anschluss, nicht nur den letzten.
6. Verwenden Sie `parse-registry read --recover-deleted` für gelöschte Geräteschlüssel.

## Weitere Informationen { #further-details }

- [Microsoft: Geräteeigenschaften (devpkey.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft: Unterstützung von Mount-Manager-Anforderungen](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h` (wie in den Headern von Wine)](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h): die Nummern der Geräteeigenschaften
- [libyal winreg-kb: USB storage](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md),
  [Mounted devices](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md) und
  [Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

Benutzerzuordnungen verwenden die Quellidentität jedes Hives, sodass exportierte Hives mit demselben Profilnamen getrennt bleiben. `SourceKeys` bewahrt die Pfade der beitragenden Benutzer-Hives. Die Erkennung zugehöriger Benutzer-Hives überspringt symbolische Links, im Einklang mit der rekursiven Hive-Erkennung. Die VID/PID-Korrelation erfordert eine übereinstimmende, verwendbare Container-ID; ohne eine solche muss der Abgleich über Seriennummern eindeutige IDs ergeben. Widersprüchliche Kandidaten bleiben unbekannt und werden mit einem Problem gemeldet.
