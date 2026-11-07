# Über Juntei { #about-juntei }

Juntei ist ein Parser für forensische Windows-Artefakte, geschrieben in speichersicherem Rust. Er ergänzt
[Hayabusa](https://github.com/Yamato-Security/hayabusa): Hayabusa analysiert Windows-Ereignisprotokolle,
während Juntei persistente Artefakte aus Dateisystem, Registry, Ausführung, Benutzeraktivität, Ressourcennutzung
und Sicherheitsprodukten parst.

Dieselbe Binärdatei läuft unter Windows, macOS und Linux. Das ist nützlich, wenn eine Analystin oder ein Analyst eine KAPE-,
Velociraptor-, Disk-Image- oder andere Offline-Sammlung erhält und eine einheitliche Oberfläche statt
eines separaten Programms für jedes Artefakt wünscht.

## Designziele { #design-goals }

- **Belastbares Parsing:** Rohwerte bewahren und Beschädigungen melden, statt stillschweigend Daten zu erfinden.
- **Für alle nützlich:** Die Standardnutzung soll für Einsteiger einfach sein, für erfahrene Fachleute aber ebenso anpassbar und
  voll ausgestattet.
- **Eigenständige Binärdatei:** Zur Ausführung sind keine Laufzeitbibliotheken erforderlich. Sie funktioniert einfach
  sofort.
- **Rasend schnell:** Strebt an, der schnellste Parser zu sein, ohne dabei viel Arbeitsspeicher
  zu benötigen.
- **Plattformübergreifende Nutzung:** Windows-Beweise analysieren, ohne eine Windows-Workstation zu benötigen.
- **Automatisierung:** CSV für Tabellenkalkulationen und Zeitleistenwerkzeuge; JSON/JSONL für `jq`, Skripte und Data Lakes.
- **Sicheres Scheitern:** Fehlerhafte Datensätze werden gemeldet und nach Möglichkeit übersprungen.
- **Transparente Herkunft:** Juntei benennt die Open-Source-Parser und Referenzwerkzeuge hinter jeder Implementierung.

## Was Juntei nicht tut { #what-juntei-does-not-do }

Juntei erfasst keinen Live-Datenträger und ersetzt nicht die Beweissicherung; als Disk-Images bindet es nur
VHDX-Images ein, und zwar schreibgeschützt. Es
fasst nicht automatisch alle Artefakte zu einer einzigen Darstellung des Vorfalls zusammen. Es parst die
von Ihnen bereitgestellten Beweise und erzeugt Datensätze, die sich für Untersuchung und Korrelation eignen.

Juntei übernimmt auch nicht die Analyse für Sie. Für die Analyse der Beweise ist eine DFIR-Fachkraft erforderlich.

## Lizenz { #license }

Juntei wird unter der
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) veröffentlicht; das Urheberrecht liegt bei Shirofune Security Co., Ltd. Der Quellcode wurde vor Version 1.0.0 unter der
AGPLv3 veröffentlicht. Die Lizenzen der Crates und des sonstigen Codes und der Daten Dritter, mit denen Juntei
erstellt wird, finden Sie in
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES).
Die Projekte, auf denen Juntei aufbaut und anhand derer es geprüft wurde, sind unter
[Danksagungen](../acknowledgements.md) aufgeführt.
