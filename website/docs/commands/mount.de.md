# Disk-Images einbinden { #mounting-disk-images }

`juntei mount` stellt die Dateien eines VHDX-Disk-Images während seiner Laufzeit schreibgeschützt in einem Verzeichnis bereit. Werkzeuge wie Hayabusa und Juntei können sie ohne Extraktion und ohne root-Rechte untersuchen. Die Funktion verwendet FUSE und läuft unter Linux. Reader und Mount-Bibliothek werden in [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) gepflegt.

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

In einem zweiten Terminal:

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

Drücken Sie im ersten Terminal Strg+C, um die Einbindung aufzuheben, oder führen Sie
`fusermount3 -u /mnt/host1` aus.

## Was die Einbindung zeigt { #what-the-mount-shows }

- Am Einhängepunkt erscheinen die Dateien des NTFS-Volumes. Ein gesichertes Image kann Verzeichnisse wie `C` und `LongFileNames` sowie Volume-Metadaten wie `$MFT` enthalten. Mehrere NTFS-Volumes erscheinen als `Volume1`, `Volume2` usw.
- Jede Datei ist schreibgeschützt und gehört dem Besitzer des Einhängepunkts. Die Dateizeiten sind die
  NTFS-Zeiten aus `$STANDARD_INFORMATION`.
- Namen werden so aufgelistet, wie NTFS sie speichert, und exakt nachgeschlagen. DOS-8.3-Namen und
  alternative Datenströme werden nicht angezeigt.
- Komprimierte Dateien werden dekomprimiert (bis zu 1 GiB pro Datei). EFS-verschlüsselte Dateien können nicht
  gelesen werden.

## Die Beweise werden nicht verändert { #the-evidence-is-not-changed }

Das Image wird schreibgeschützt geöffnet; der Mount verweigert Schreibzugriffe. Bei einem nicht sauber geschlossenen VHDX wird das ausstehende Protokoll nur im Arbeitsspeicher angewendet und nie zurückgeschrieben. Manche DiscUtils-Writer lassen die Sprunganweisung und die Signatur `55 AA` im NTFS-Bootsektor auf null; Juntei liest solche Volumes unverändert.

## Verwendung { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Jedes Image mit auto-parse parsen { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` parst die Windows-Systeme in jedem VHDX-Disk-Image im Beweisverzeichnis (oder in dem
einen mit `-d` angegebenen Image), ohne dass Sie es separat einbinden müssen:

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

- Jedes Image wird schreibgeschützt in einem neuen temporären Verzeichnis eingebunden, in einem nach dem Image
  benannten Ordner (`host1-triage`), und die dort gefundenen Systeme werden wie eine Sammlung auf dem
  Datenträger geparst. Die Einbindungen werden am Ende aufgehoben, ebenso bei Strg+C.
- Das Terminal listet die Images und das Image jedes Systems auf:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`.
- Im Manifest ist `EvidenceDirectory` das angegebene Verzeichnis, und `Image` nennt für jedes System sein
  Image. `EvidenceRoot` und die Parser-Befehle verwenden den Einbindungsordner (`./host1-triage/C`).
- Enthält das Image Dateien aus Volume Shadow Copies in Ordnern wie `VSS32`, wird jeder Ordner als weiteres System (`HOST1_2`, …) erkannt und geparst.
- Ein Image, das sich nicht einbinden lässt, wird gemeldet; die übrigen werden trotzdem geparst, und der Lauf
  endet mit dem Exit-Status 1.

Kann ein NTFS-Volume nicht geöffnet werden, meldet Juntei dies und behält die lesbaren Volumes bei. Bei teilweise lesbaren Images liefert `auto-parse --vhdx` den Exitstatus 1; `mount` liefert nach dem Aushängen ebenfalls 1.

## Voraussetzungen { #requirements }

- Linux: FUSE 3 (`fusermount3`, das Paket `fuse3`) und Zugriff auf `/dev/fuse`, den normale Benutzer in der
  Regel haben. Root-Rechte sind nicht erforderlich.
- Windows und macOS werden nicht unterstützt. Fügen Sie das Image unter Windows stattdessen in einer PowerShell
  mit erhöhten Rechten schreibgeschützt an, und übergeben Sie den Werkzeugen seinen Laufwerksbuchstaben:

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
