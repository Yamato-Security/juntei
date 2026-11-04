---
hide:
  - navigation
  - toc
---

<div class="jt-hero" markdown>

![Juntei, Göttin der digitalen Forensik](assets/juntei-goddess.png){ .jt-hero-art width="1448" height="1086" }

<p class="tagline">
<strong>Juntei</strong> ist der plattformübergreifende Parser von Yamato Security für Windows-Forensikartefakte,
die <em>keine</em> Ereignisprotokolle sind. Er wandelt gesicherte Beweise in einheitliche CSV-, JSON- und JSONL-Datensätze
um und bewahrt dabei den Kontext, den Ermittelnde benötigen, um Fehlschlüsse zu vermeiden.
</p>

<div class="jt-cta" markdown>
[Hier beginnen :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Die Artefakte kennenlernen :material-archive-search:](artifacts/index.md){ .md-button }
[Befehlsreferenz :material-console:](commands/index.md){ .md-button }
[GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/juntei){ .md-button }
</div>

</div>

## Vom Rohartefakt zur belastbaren Schlussfolgerung { #from-raw-artifact-to-defensible-conclusion }

Juntei kann Ihnen sagen, was ein Artefakt aufzeichnet; diese Website erklärt, was dieser Datensatz **beweist und was
nicht**. Beginnen Sie mit einer Artefaktseite, auch wenn Sie noch nie Windows-DFIR durchgeführt haben.
Jeder Leitfaden behandelt Zweck, Speicherort auf dem Datenträger, Hinweise zu Aufbewahrung und Sicherung, Parsing-Befehl,
Ausgabefelder, Interpretation, Korrelationen und weiterführende Literatur.

<div class="grid cards" markdown>

-   :material-file-tree:{ .lg .middle } __Dateisystemverlauf__

    ---

    Nutzen Sie `$MFT` und das USN-Journal, um Existenz, Namen, Verschiebung und Löschung von Dateien,
    Metadatenänderungen sowie alternative Datenströme zu rekonstruieren.

-   :material-play-circle-outline:{ .lg .middle } __Programmnachweise__

    ---

    Prefetch kann eine Ausführung stark belegen. Amcache und ShimCache liefern zusätzlich Inventar-, Pfad-, Hash-,
    Kompatibilitäts- und historischen Kontext – erfordern aber eine sorgfältige Interpretation.

-   :material-account-clock:{ .lg .middle } __Benutzeraktivität__

    ---

    LNK-Dateien, Jump Lists, RecentDocs und SRUM zeigen geöffnete Elemente, Anwendungsaktivität,
    Wechseldatenträger- oder Netzwerkspeicherorte, Ressourcennutzung und historischen Benutzerkontext.

-   :material-shield-bug:{ .lg .middle } __Sicherheitsnachweise__

    ---

    Die Defender-Quarantäne kann Bedrohungsbezeichnungen, ursprüngliche Pfade, Download-Quellen, Hashes
    und Besitzer bewahren und sogar die Bytes unter Quarantäne gestellter Dateien sicher für eine isolierte Analyse wiederherstellen.

-   :material-database-search:{ .lg .middle } __Registry-Zustand__

    ---

    Lesen Sie zugeordnete und wiederherstellbare Registry-Daten, spielen Sie Transaktionsprotokolle ein und untersuchen Sie Dienste,
    geplante Aufgaben, lokale Konten und den Zustand zuletzt verwendeter Dokumente.

-   :material-link-variant:{ .lg .middle } __Korrelieren statt raten__

    ---

    Kein einzelnes Artefakt erzählt die ganze Geschichte. Die Leitfäden zeigen, wie Sie unabhängige Quellen
    zu einer beweisgestützten Zeitleiste kombinieren.

</div>

!!! warning "Forensische Ausgabe ist ein Beweismittel, kein Urteil"
    Ein verdächtig wirkender Pfad ist ein Hinweis. Ein Zeitstempel kann die Metadaten einer Datei beschreiben statt
    einer Ausführung. Ein fehlender Datensatz beweist nicht, dass eine Aktion nicht stattgefunden hat. Bewahren Sie die Quellbeweise,
    dokumentieren Sie Tool-Versionen und Befehle und untermauern Sie wichtige Schlussfolgerungen.
