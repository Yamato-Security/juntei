# Disk-Images einbinden { #mounting-disk-images }

`juntei mount` stellt die Dateien eines VHDX / E01 / Ex01-Disk-Images während seiner Laufzeit schreibgeschützt in einem Verzeichnis bereit. Werkzeuge wie Hayabusa und Juntei können sie ohne Extraktion und ohne root-Rechte untersuchen. Die Funktion verwendet FUSE und läuft unter Linux. Reader und Mount-Bibliothek werden in [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) gepflegt.

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
  <IMAGE>       The disk image (VHDX or E01; a split E01 image from its first segment)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## Native Analyse von Images { #parsing-every-image-with-auto-parse }

Mit `--vhdx` liest `auto-parse` VHDX nativ unter **Linux, macOS und Windows**, über [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser). OS-Mount, FUSE und Administratorrechte sind nicht erforderlich.

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

Die Parser benötigen Dateipfade. Daher kopiert Juntei die bereitgestellten NTFS-Dateien mit begrenzten Puffern in ein privates temporäres Verzeichnis. **Es muss Platz für alle bereitgestellten Dateien vorhanden sein**, nicht nur für Artefakte mit Ergebnissen. Nullbereiche werden, soweit unterstützt, als Löcher geschrieben. Namen, Struktur, Hardlinks und Änderungszeiten bleiben erhalten. LNK- und Sprunglisten-Quellzeiten stammen aus einem Snapshot der ursprünglichen NTFS-Metadaten, niemals aus den temporären Dateien. Regulärer Abschluss, auch mit Parserfehlern, entfernt die Kopien; erzwungene Beendigung kann sie zurücklassen. Die Quelle bleibt schreibgeschützt; ausstehende VHDX-Logs werden nur im Speicher angewendet.

Hosterkennung, CSV/JSON/JSONL und `-j` nutzen dieselbe Verarbeitung wie normale Beweisverzeichnisse. `Image` in `manifest.json` bezeichnet das Originalimage, `EvidenceRoot` den internen relativen Pfad. Lesefehler und unsichere Namen erscheinen in `ImageErrors` sowie Fehlerzeilen von `manifest.csv`. Lesbare Ergebnisse bleiben erhalten; unvollständige Eingaben ergeben Exitcode 1. Unsichere oder mehrdeutige Namen werden nicht umbenannt. Benannte NTFS-Streams werden nicht als zusätzliche Dateien extrahiert; als normale Dateien gesammelte Artefakte bleiben verfügbar.

Der separate [`mount`-Befehl](mount.md) stellt anderen Programmen ein Dateisystem bereit und benötigt weiterhin Linux FUSE. `auto-parse --vhdx` verwendet ihn nicht.

Das Staging nutzt standardmäßig das Dateisystem der Ausgabe statt des möglicherweise kleinen RAM-basierten System-Temp-Verzeichnisses. Mit `--temp-dir` wählen Sie ein vorhandenes Verzeichnis auf einem anderen Datenträger. Quellmetadaten bleiben separat im Ausgabedateisystem, damit bereits lesbare Dateien auch bei vollem Staging-Datenträger analysiert werden können. Platz für Kopien und Ergebnisse einplanen; ein volles Ausgabedateisystem verhindert das Speichern. Ctrl+C stoppt Parserprozesse, entfernt den privaten Arbeitsbereich und beendet mit Code 130. SIGKILL oder Stromausfall können Kopien hinterlassen. Nicht lesbare Unterverzeichnisse werden bei der Imagesuche übersprungen; ein nicht lesbares Eingabewurzelverzeichnis bleibt ein Fehler.

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## E01- und Ex01-Datenträgerabbilder { #e01-and-ex01-disk-images }

`--e01` liest physische EWF1- (`.E01`) und EWF2-Abbilder (`.Ex01`) nativ unter **Linux, macOS und Windows**, ohne FUSE, Betriebssystem-Mount oder Administratorrechte. Es verwendet [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) und dieselbe oben beschriebene NTFS-Verarbeitung.

Bei geteilten Abbildern das erste Segment angeben. Alle zugehörigen Dateien einschließlich `.E02` müssen mit ihren ursprünglichen Namen daneben liegen. Ganze Datenträger (MBR/GPT) und einzelne NTFS-Volumes werden unterstützt. Für gemischte Verzeichnisse `--vhdx --e01` verwenden.

Dieselben Anforderungen an temporären Speicher und Quellmetadaten gelten. Abbilder werden nur gelesen. Erfassungslücken und beim Öffnen oder Lesen erkannte Schäden erscheinen in `ImageErrors` und `manifest.csv`; lesbare Ergebnisse bleiben erhalten, der Exit-Code ist 1. Geprüft werden gelesene Daten, nicht alle Chunks oder der Hash des gesamten Mediums. Logische Dateien `.L01` und `.Lx01` werden abgewiesen; eine Passwortoption gibt es nicht.

Der separate Befehl `mount` benötigt weiterhin Linux FUSE, die native automatische Verarbeitung nicht.

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## Anforderungen für OS-sichtbare Mounts { #os-visible-mount-requirements }

`juntei mount` benötigt Linux FUSE 3 (`fusermount3`) und Zugriff auf `/dev/fuse`. Unter Windows und macOS steht die oben beschriebene native Analyse bereit.
