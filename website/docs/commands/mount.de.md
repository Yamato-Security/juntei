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

## Native Analyse von Images { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` liest VHDX-Images nativ unter **Linux, macOS und Windows**. Ein Betriebssystem-Mount, FUSE oder Administratorrechte sind nicht erforderlich.

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Die Parser benötigen Dateipfade. Juntei kopiert deshalb die vom NTFS-Leser bereitgestellten Dateien in ein privates temporäres Verzeichnis. **Dafür muss genügend temporärer Speicherplatz vorhanden sein.** Das Quellimage bleibt schreibgeschützt; bei regulärem Abschluss werden die Kopien entfernt.

Lesefehler und unsichere Namen erscheinen in `ImageErrors` und als fehlgeschlagene Zeilen in `manifest.csv`. Lesbare Ergebnisse bleiben erhalten; unvollständige Eingaben führen zum Exitcode 1.

Der separate Befehl `mount` stellt Dateien anderen Programmen bereit und benötigt weiterhin Linux, FUSE 3 und Zugriff auf `/dev/fuse`.

[English: implementation details and limitations](https://yamato-security.github.io/juntei/commands/auto-parse/#vhdx-disk-images).
