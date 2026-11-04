# Artefaktübergreifende Untersuchungen { #cross-artifact-workflows }

Kein Artefakt sollte eine wichtige Schlussfolgerung allein tragen. Diese Abläufe zeigen, wie aus einem Ansatzpunkt
eine belastbare Beweiskette wird.

## Ausführung einer verdächtigen ausführbaren Datei { #suspicious-executable-execution }

1. **Prefetch:** gespeicherte Ausführungszeit, Pfad-/Hash-Kontext, Ausführungsanzahl und geladene Dateien feststellen.
2. **`$MFT`:** Pfad, Größe, Zeitstempel, ADS, Löschstatus und MFT-Referenz ermitteln.
3. **USN-Journal:** Erstellung, Schreibvorgänge, Umbenennung und Löschung rekonstruieren.
4. **Amcache:** SHA-1, PE-Metadaten, Version und historischen Pfad ergänzen.
5. **ShimCache:** Pfad/Reihenfolge im Kompatibilitätscache und unterstütztes Ausführungsflag ergänzen.
6. **SRUM:** nach Ressourcen-/Netzwerknutzung durch dieselbe ausführbare Datei bzw. denselben Benutzer suchen.
7. **Defender:** Quarantänezeitpunkt, ursprünglichen Pfad, Hash und Download-Herkunft prüfen.
8. **Ereignisprotokolle/EDR:** übergeordneten Prozess, Befehlszeile, Konto, Netzwerk und Sicherheitskontext ermitteln.

Die Schlussfolgerung sollte zwischen Vorhandensein der Datei, Ausführung und Zuordnung zu einer Person oder einem Angriff unterscheiden.

## Dateiaktivität von Ransomware { #ransomware-file-activity }

1. Ermitteln Sie verschlüsselte Erweiterungen und betroffene Pfade in `$MFT`.
2. Verwenden Sie USN-Gründe und -Reihenfolge, um massenhafte Schreibvorgänge/Umbenennungen/Löschungen zu rekonstruieren.
3. Verwenden Sie Prefetch, um mögliche Verschlüsselungsprogramme und Ausführungszeiten zu ermitteln.
4. Verwenden Sie SRUM für CPU-/Datenträger-/Netzwerkspitzen des verdächtigen Prozesses.
5. Prüfen Sie Dienste und TaskCache auf Verteilung/Persistenz.
6. Prüfen Sie Defender-Quarantäne und Ereignisprotokolle auf Erkennung/Bereinigung.

MFT/USN können Auswirkungen auf das Dateisystem belegen, selbst wenn kein Ausführungsartefakt erhalten ist. Sie identifizieren
den verantwortlichen Prozess im Allgemeinen nicht ohne Prozesstelemetrie oder Korrelation.

## Erstzugriff über bösartige Dokumente / Verknüpfungen { #malicious-document-shortcut-initial-access }

1. RecentDocs und Jump Lists zeigen Benutzer-/Anwendungsinteraktion und MRU-Kontext.
2. Parsen Sie die passende LNK im Ordner „Recent“ auf Ziel, Volume/Freigabe, MFT-Referenz und Argumente.
3. Verwenden Sie Zone.Identifier aus residenten ADS in `$MFT` oder Defender-Inhalten für Herkunfts-URLs/-Zonen.
4. Verwenden Sie Prefetch/Ereignisprotokolle für die Dokumentanwendung und untergeordnete Prozesse.
5. Verwenden Sie USN/MFT, um Erstellung und Löschung extrahierter Payloads zu verfolgen.

## Persistenz über Dienste { #service-persistence }

1. Dekodieren Sie Dienstname, `ImagePath`, `ServiceDll`, Konto, Starttyp und Fehleraktionen.
2. Lösen Sie Umgebungsvariablen und das Verhalten von Pfaden mit/ohne Anführungszeichen auf.
3. Lokalisieren Sie die ausführbare Datei/DLL in `$MFT` und im USN-Verlauf.
4. Prüfen Sie Prefetch und Ereignisprotokolle auf tatsächlichen Dienst-/Prozessstart.
5. Vergleichen Sie nicht aktuelle Control Sets und wiederhergestellte Registry-Datensätze auf frühere Konfigurationen.

## Persistenz über geplante Aufgaben { #scheduled-task-persistence }

1. Dekodieren Sie TaskCache-Aktionen, Prinzipal/Kontext, Registrierungs-/Laufzeitmetadaten und verborgenen Status.
2. Vergleichen Sie mit dem XML in `C:\Windows\System32\Tasks`.
3. Korrelieren Sie Aufgabenname/-GUID mit Security-Ereignissen und Ereignissen aus dem Operational-Protokoll des Aufgabenplaners.
4. Lokalisieren und hashen Sie die ausführbare Datei bzw. das Skript; prüfen Sie Prefetch und den Dateisystemverlauf.
5. Stellen Sie gelöschte TaskCache-Datensätze wieder her, falls die Aufgabe entfernt wurde.

## Verdacht auf Exfiltration { #suspected-exfiltration }

1. Verwenden Sie SRUM, um ungewöhnlich viele gesendete Bytes nach App/Benutzer/Profil und den Messzeitraum zu ermitteln.
2. Bestätigen Sie die Identität der ausführbaren Datei mit Prefetch, Amcache und `$MFT`.
3. Ermitteln Sie zugegriffene/geöffnete Dateien über Jump Lists, LNK, RecentDocs und Ereignis-/EDR-Daten.
4. Verwenden Sie Telemetrie von Proxy, Firewall, DNS, Paketmitschnitten oder EDR für Ziel/Protokoll.

Das Datenvolumen in SRUM stützt eine Exfiltrationshypothese, identifiziert aber weder übertragene Inhalte noch beweist es
für sich genommen eine böswillige Absicht.

## Korrelationsschlüssel, die es zu bewahren lohnt { #correlation-keys-worth-preserving }

| Schlüssel | Artefakte |
|---|---|
| Vollständiger Pfad / Dateiname | Fast jedes Artefakt |
| SHA-1 / SHA-256 | Amcache, Defender, wiederhergestellte Dateien |
| MFT-Eintrag + Sequenz | `$MFT`, Prefetch, LNK, Jump Lists, USN |
| USN | Amcache, `$MFT`, USN-Journal |
| Volume-Seriennummer / Droids | Prefetch, LNK, Jump Lists, MFT-Objekt-IDs |
| SID / Benutzername | SRUM, SAM, Registry, Ereignisprotokolle |
| Aufgaben-GUID / Aufgabenname | TaskCache, Aufgaben-XML, Ereignisprotokolle |
| Dienstname | Dienste, Ereignisprotokolle, Prozess-/Treiberdaten |
| Defender EntryId / ScanId | Quarantäne, Defender-Verlauf, Ereignisprotokolle |
