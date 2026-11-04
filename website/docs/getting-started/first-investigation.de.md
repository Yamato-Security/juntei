# Erste Untersuchung { #first-investigation }

Dieser Ablauf geht davon aus, dass Sie eine Offline-Triage-Sammlung in `evidence/` erhalten haben.

## 1. Die Quelle bewahren { #1-preserve-the-source }

Arbeiten Sie mit einer verifizierten Kopie. Dokumentieren Sie Hashes, Sicherungszeitpunkt, Zeitzone, Host, Volume und Sicherungs-
tool. Juntei liest Beweise, aber Optionen wie das Extrahieren residenter Streams oder von Quarantänedateien erzeugen
neue Ausgabedateien; leiten Sie diese in ein separates Analyseverzeichnis.

## 2. Bestand aufnehmen { #2-inventory-what-you-have }

Suchen Sie nach den Pfaden aus [Beweise sichern](collection.md). Fehlende Artefakte sind normal. Gehen Sie nicht
davon aus, dass eine Sammlung Transaktionsprotokolle enthält, nur weil sie einen Registry-Hive enthält.

## 3. Zuerst Zusammenfassungen ausführen { #3-run-summaries-first }

Viele Befehle geben eine knappe Zusammenfassung aus, wenn keine Ausgabedatei angefordert wird:

```bash
juntei parse-file prefetch -d evidence/
juntei parse-registry shimcache -d evidence/
juntei parse-file defender-quarantine -d evidence/
```

Lesen Sie Warnungen zu Beschädigungen, „dirty“ Datenbanken und nicht angewendeten Protokollen, bevor Sie Datensätzen vertrauen.

Um jedes unterstützte Artefakt einer Sammlung mehrerer Computer in einem Durchgang zu parsen, verwenden Sie:

```bash
juntei auto-parse -d evidence/ -o analysis/
```

Siehe [Sammlungen automatisch analysieren](../commands/auto-parse.md) zu Host-Erkennung, Parallelisierung,
Ausgabestruktur und zusammengeführtem JSONL für die Aufnahme in ein SIEM.

## 4. Strukturierte Ausgabe speichern { #4-save-structured-output }

```bash
mkdir -p analysis
juntei parse-file prefetch -d evidence/ -T -o analysis/prefetch.csv
juntei parse-ntfs mft -d evidence/ -o analysis/mft
juntei parse-ntfs usn-journal -d evidence/ -o analysis/usn
juntei parse-registry services -d evidence/ -o analysis/services.jsonl -t jsonl
```

Bevorzugen Sie JSONL für Skripte und verschachtelte Werte und CSV für eine Tabellenkalkulation oder einen Zeitleisten-Viewer.

## 5. Von einer verlässlichen Kennung ausgehen { #5-pivot-from-a-reliable-identifier }

Beginnen Sie mit einem Dateinamen, einem vollständigen Pfad, SHA-1/SHA-256, einer Benutzer-SID, MFT-Eintrag/-Sequenz, einer Aufgaben-GUID,
einem Dienstnamen oder einem Zeitfenster. Normalisieren Sie die Groß-/Kleinschreibung nur bei Feldern, die Windows ohne Beachtung der Groß-/Kleinschreibung behandelt.

```bash
juntei parse-file prefetch -d evidence/ -t jsonl \
  | jq 'select(.ExecutableName? | ascii_downcase == "suspicious.exe")'
```

## 6. Korrelieren { #6-correlate }

Verwenden Sie die [artefaktübergreifenden Untersuchungen](../resources/workflows.md). Eine praxistaugliche Reihenfolge ist:

1. Prefetch für Ausführung und jüngste Ausführungszeiten.
2. `$MFT` und USN-Journal für Erstellung, Pfade, Umbenennungen und Löschung.
3. Amcache und ShimCache für historisches Vorhandensein und Metadaten.
4. LNK, Jump Lists, RecentDocs und SRUM für Benutzer-/Anwendungskontext.
5. Registry-Dienste und TaskCache für Persistenz.
6. Defender-Quarantäne für Erkennung, Entfernung, Herkunft und Hashes.

## 7. Reproduzierbarkeit dokumentieren { #7-document-reproducibility }

Dokumentieren Sie Quellpfad, Quell-Hash, Juntei-Version, exakten Befehl, Ausgabe-Hash, Systemzeitzone
und jede spätere Filterung. Bearbeiten Sie erzeugte Beweisausgaben niemals stillschweigend.
