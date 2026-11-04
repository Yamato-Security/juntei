# Screenshots { #screenshots }

Entdecken Sie die Befehlszeile von Juntei, das automatische Parsing und die übersichtlich organisierten Untersuchungsergebnisse.
Klicken Sie auf einen Screenshot, um ihn in voller Größe anzuzeigen.

## 1. Start und Befehle { #1-startup-and-commands }

<figure markdown>

[![Startbildschirm von Juntei mit dem CODE BLUE Release und den verfügbaren Befehlen](assets/screenshots/01-Startup.png)](assets/screenshots/01-Startup.png)

<figcaption>Der Startbildschirm von Juntei listet das automatische Parsing, quellenspezifische Parser und Persistence Hunter auf.</figcaption>

</figure>

## 2. Automatisches Parsing in weniger als 20 Sekunden { #2-automatic-parsing-in-under-20-seconds }

Auf einem **MacBook Pro M4** hat der unten gezeigte Lauf **8,7 GiB gesicherter Windows-Daten**
von **sechs Systemen** verarbeitet und **4,4 GiB Ausgabe in weniger als 20 Sekunden** gespeichert.
Der Befehl [auto-parse](commands/auto-parse.md) führte 240 Parser-Aufgaben mit 16 parallelen
Prozessen aus und erzeugte 482 Ergebnisdateien. Es gab **210 erfolgreiche Aufgaben, 30 übersprungene,
weil die Artefakte nicht gesichert worden waren, und null Fehlschläge**.

<figure markdown>

[![Juntei auto-parse schließt sechs Systeme mit 8,7 GiB Eingabe und 4,4 GiB Ausgabe in weniger als 20 Sekunden ab](assets/screenshots/02-AutoParse.png)](assets/screenshots/02-AutoParse.png)

<figcaption>Automatische Host-Erkennung, paralleles Parsing und eine Abschlusszusammenfassung mit Eingabegröße, Ausgabegröße, Aufgabenergebnissen und verstrichener Zeit.</figcaption>

</figure>

## 3. Für die Untersuchung organisierte Ergebnisse { #3-results-organized-for-investigation }

<figure markdown>

[![Ausgabeordner eines Systems, gruppiert nach DFIR-Kategorien, mit Konten-CSVs und den zugehörigen Parser-Protokollen](assets/screenshots/03-OutputFolders.png)](assets/screenshots/03-OutputFolders.png)

<figcaption>Die Ergebnisse jedes Computers sind nach DFIR-Kategorie gruppiert. Artefakt-CSVs und die passenden <code>-logs.txt</code>-Dateien bleiben zusammen, sodass sich sowohl die Datensätze als auch die Parsing-Details leicht finden lassen.</figcaption>

</figure>
