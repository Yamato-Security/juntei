# Beweise und Interpretation { #evidence-and-interpretation }

Digitale Forensik bedeutet nicht nur, Felder zu lesen. Es geht darum zu entscheiden, welche Beobachtung ein Feld belegt,
wie stark dieser Beleg ist und was dieselbe Beobachtung auf harmlose Weise hervorrufen könnte.

## Vier nützliche Beweiskategorien { #four-useful-evidence-categories }

| Kategorie | Bedeutung | Beispiele |
|---|---|---|
| Präsenz | Ein Pfad oder eine Datei war Windows bekannt | Amcache-Dateidatensatz, ShimCache-Eintrag, `$MFT`-Datensatz |
| Ausführung | Windows hat einen Programmstart oder einen Ausführungszustand aufgezeichnet | Prefetch-Ausführungszeit; unterstütztes ShimCache-Ausführungsflag |
| Benutzerinteraktion | Ein Benutzer oder eine Anwendung hat auf ein Element verwiesen/es geöffnet | RecentDocs, Jump List, LNK-Datei im Ordner „Recent“ |
| Systemaktivität | Windows hat Ressourcennutzung oder Konfiguration aufgezeichnet | SRUM-Zähler, Dienstkonfiguration, TaskCache-Aktion |

Ein Artefakt kann mehr als eine Kategorie belegen, aber stufen Sie es niemals automatisch höher ein. Zum Beispiel
belegt ein Amcache-Eintrag mit einem verdächtigen Pfad **Präsenz**. Eine passende Prefetch-Ausführungszeit liefert
stärkere Belege für eine **Ausführung**. Ein USN-Erstellungsdatensatz und ein `$MFT`-Pfad können die Datei auf dem Datenträger verorten, und
die Defender-Quarantäne kann erklären, warum sie verschwunden ist.

## Zeitstempel sind Aussagen, keine universelle Wahrheit { #timestamps-are-statements-not-universal-truth }

Fragen Sie, was jeden Zeitstempel verursacht hat:

- Die Zeiten in `$STANDARD_INFORMATION` und `$FILE_NAME` beschreiben NTFS-Metadaten und können voneinander abweichen.
- Eine Prefetch-Ausführungszeit beschreibt einen aufgezeichneten Start, aber nur die jüngsten Ausführungszeiten bleiben erhalten.
- Eine ShimCache-Zeit ist in der Regel die letzte Änderungszeit der zwischengespeicherten Datei – nicht die Ausführungszeit.
- Die letzte Schreibzeit eines Registry-Schlüssels gilt für den Schlüssel als Ganzes, nicht für einen einzelnen Wert.
- RecentDocs ordnet die Zeit des Schlüssels dem jüngsten Element zu; ältere Elemente haben eine Reihenfolge, aber in der Regel keine Öffnungszeit.
- SRUM-Zeitstempel kennzeichnen oft ein Aggregationsintervall statt des genauen Netzwerkvorgangs.

Verwenden Sie während der Analyse UTC, bewahren Sie den ursprünglichen Wert, sofern verfügbar, und dokumentieren Sie jede Umrechnung.

## Positive und negative Beweise { #positive-and-negative-evidence }

Das Vorhandensein eines gut verstandenen Datensatzes kann ein starker positiver Beweis sein. Das Fehlen ist in der Regel
schwächer, denn Artefakte werden überschrieben, Caches sind begrenzt, die Sicherung kann unvollständig sein, Einstellungen variieren
und es gibt Bereinigungswerkzeuge. Sagen Sie „im gesicherten Artefakt nicht beobachtet“ statt „ist nicht geschehen“.

## Bestätigungsleiter { #corroboration-ladder }

1. Validieren Sie die Quelldatei und den Sicherungspfad.
2. Bestätigen Sie, dass der Parser keine relevanten Beschädigungen oder nicht angewendeten Transaktionsprotokolle gemeldet hat.
3. Interpretieren Sie das Feld gemäß der Artefaktseite.
4. Suchen Sie ein unabhängiges Artefakt, das dieselbe Aktion belegt.
5. Prüfen Sie Ereignisprotokolle, Netzwerktelemetrie, Arbeitsspeicher oder Dateiinhalte, sofern verfügbar.
6. Dokumentieren Sie die genaue Juntei-Version, den Befehl, die Filter und den Hash der Quelle.

!!! tip "Formulieren Sie Schlussfolgerungen entsprechend der Stärke der Beweise"
    Bevorzugen Sie „Prefetch zeichnet die Ausführung von `tool.exe` um 10:02:11Z auf“ gegenüber „der Angreifer hat die
    Malware genau um 10:02:11Z ausgeführt“. Die zweite Aussage fügt Identität und Absicht hinzu, die Prefetch
    allein nicht belegen kann.
