# YARA-Scan { #yara-scan }

Gesammelte Dateien mit einer portablen Rust-Engine untersuchen. Windows, macOS und Linux werden unterstützt. Dateien werden nur als Daten gelesen, niemals ausgeführt. Der Befehl ist unabhängig von der automatischen Artefaktanalyse.

## Regeln auswählen { #choose-rules }

Core ist die Voreinstellung und priorisiert Genauigkeit und Leistung. Extended erweitert die Suche. Full bietet die größte Abdeckung, kann aber mehr Fehlalarme erzeugen.

```bash
juntei yara-scan -d evidence/ -o yara-core/
juntei yara-scan -d evidence/ --ruleset extended -o yara-extended/
juntei yara-scan -d evidence/ --ruleset full -o yara-full/ -t jsonl
juntei yara-scan -d evidence/ --rules local-rules/ -o yara-offline/
```

`--rules <FILE_OR_DIRECTORY>`: Lokale UTF-8-Regeldatei oder ein Verzeichnis verwenden, ohne Download. Verzeichnisse werden rekursiv durchsucht. Includes bleiben im gewählten Regelverzeichnis und werden einmal in Abhängigkeitsreihenfolge aufgelöst. Ungültige oder nicht unterstützte Regeln beenden die Kompilierung, statt stillschweigend verworfen zu werden.

## Grenzen und Umfang { #limits-and-scope }

Alle regulären Dateien sind geeignet, auch wiederhergestellte Quarantäneinhalte. Symbolische Links und Spezialdateien werden übersprungen. Archive und Datenträgerabbilder werden nicht entpackt; laufender Arbeitsspeicher wird nicht untersucht. Stabile extrahierte Dateien und ein neues Ausgabeverzeichnis außerhalb der Beweise verwenden.

- `-T, --threads <N>`: Parallele Scans; standardmäßig CPU-Anzahl, höchstens 8.
- `--timeout <SECONDS>`: Zeitlimit für die Auswertung jeder Datei in Sekunden; Standard: 60.
- `--max-file-mib <MIB>`: Maximale Dateigröße in MiB; Standard: 64. Größere Dateien werden als übersprungen erfasst. Mehr Worker und größere Puffer erhöhen den Speicherbedarf.
- `--rules-release YYYYMMDD`: Forge-Veröffentlichungsdatum oder standardmäßig latest. Heruntergeladene Quellen und Hashwerte werden zur Reproduzierbarkeit gespeichert.

## Ergebnisse { #results }

- `YaraDetections.csv` / `YaraDetections.jsonl`: Treffer je Regel und Datei, mit relativen Pfaden, Beweisrechnernamen, Hashwerten, Metadaten und begrenzten Trefferpositionen. Getroffene Bytes werden nicht exportiert.
- `YaraFiles.csv` / `YaraFiles.jsonl`: Abdeckung je Datei, einschließlich übersprungener Dateien, Lesefehler, Zeitüberschreitungen und Teilergebnisse.
- `Summary.json`: Abschlussstatus, Einstellungen, Anzahlen, Engine-Version und Quellhashwerte.
- `Rules.yar`: Exakte heruntergeladene Regeln oder ein aufgelöster Schnappschuss lokaler Regeln.
- `CompilationWarnings.txt` / `CompilationError.txt`: Kompilierungswarnungen oder Fehler bei fehlgeschlagener Kompilierung.

Exit-Codes: 0 bedeutet vollständig, 3 unvollständige Abdeckung, 2 Einrichtungs-/Ausgabefehler und 130 Abbruch. Ein Treffer ist ein Ermittlungsansatz, kein Kompromittierungsnachweis. Keine Treffer beweisen keine Sicherheit. Regeltitel und Artefaktkontext prüfen. Quarantäneinhalte müssen vor ihrer Identifizierung entschlüsselt werden.

## MFT-Treffer und Wiederherstellung { #mft-recovery }

MFTs werden einzeln als Rohbytes mit eigener Größenbegrenzung untersucht (Standard: 512 MiB). Datensätze an Trefferpositionen werden geparst und ihre residenten DATA-Streams separat geprüft. Nur eigenständig passende Streams werden wiederhergestellt. Metadaten-, datensatzübergreifende und nichtresidente Treffer werden ohne Dateiinhalte gemeldet.

`--max-mft-mib <MIB>` · `--max-mft-recoveries <N>`

Standardmäßig werden höchstens 1000 Streams pro Lauf wiederhergestellt. Grenzen, beschädigte Datensätze und Zeitüberschreitungen bleiben sichtbar. Gespeicherte Namen erhalten die Quarantäne-Endung; Beweisverzeichnisse werden nicht nachgebildet.

`YaraMFTResults.csv` / `YaraMFTResults.jsonl` · `RecoveredMFT/` · `.quarantined`

## Weitere Informationen { #further-details }

- [Engine: Boreal](https://github.com/vthib/boreal)
- [Regelpakete: YARA Forge](https://github.com/YARAHQ/yara-forge)
- [Ausführliche Nutzung und Feldreferenz (Englisch)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.md)
- [Ausführliche Nutzung und Feldreferenz (Japanisch)](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.ja.md)
