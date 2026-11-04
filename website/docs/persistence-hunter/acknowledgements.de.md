# Danksagungen zu Persistence Hunter { #persistence-hunter-acknowledgements }

Der Befehl `persistence-hunter` von Juntei baut auf zwei Generationen der Persistenzsuche aus der Community auf.
Wir danken den Autorinnen und Autoren sowie allen Mitwirkenden, die ihre Techniken, Implementierungen und Forschungsergebnisse
für Untersuchende verfügbar gemacht haben.

## PyrsistenceSniper: das Python-Projekt { #pyrsistencesniper-the-python-project }

[**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper) von
[Hexastrike](https://github.com/Hexastrike) ist die direkte Referenz für diese Funktion. Juntei portiert
dessen Katalog mit 130 Prüfungen nach nativem Rust, zusammen mit der Standard-Allowlist, dem LOLBAS-Snapshot und dem
Klassifizierungsverhalten. Die Referenzversion ist auf
[Commit `b99c117`](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f) festgelegt.

Das Python-Projekt steht unter der MIT-Lizenz. Juntei bewahrt dessen Copyright- und Lizenzhinweis in
[`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt).
Junteis Implementierung läuft offline ohne Python-Interpreter. Ihre Behandlung von Sammlungen,
die Berichterstattung über die Abdeckung und die zurückhaltende Interpretation unvollständiger Beweise sind im
[Befehlsleitfaden](../commands/persistence-hunter.md) dokumentiert.

## PersistenceSniper: das ursprüngliche PowerShell-Werkzeug { #persistencesniper-the-original-powershell-tool }

[**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper) ist das ursprüngliche PowerShell-Projekt
zur Persistenzsuche, das PyrsistenceSniper inspiriert hat. PyrsistenceSniper nennt
Federico Lagrasta; das ursprüngliche Repository nennt außerdem Riccardo Ancarani und weitere
Mitwirkende. Wir danken dem ursprünglichen Projekt und seinen Mitwirkenden dafür, dass sie die Untersuchung von Windows-Persistenzmechanismen
erleichtert haben.

Junteis direkte Portierungsreferenz ist das Python-Projekt. Das ursprüngliche PowerShell-Werkzeug wird
für diese Herkunft genannt. Juntei implementiert außerdem unabhängig die Mechanismen, die
[hongson11698](https://github.com/hongson11698) im ursprünglichen Projekt in
[PR #31 zur COM-Auflösung geplanter Aufgaben](https://github.com/last-byte/PersistenceSniper/pull/31)
und [PR #32 zu Startmenü-Bibliotheken/CLSID-Ordnern](https://github.com/last-byte/PersistenceSniper/pull/32) vorgeschlagen hat.
Diese ergänzen den Referenzkatalog des Python-Projekts; daraus folgt nicht, dass jede Live-System-Funktion eines der beiden Upstream-Projekte
vom Offline-Befehl von Juntei implementiert wird.

## Unterstützende Forschung und Werkzeuge { #supporting-research-and-tools }

- [MITRE ATT&CK](https://attack.mitre.org/) liefert die im Katalog geführten Technikbezeichner.
  Eine Technikzuordnung beschreibt eine mögliche Nutzung eines Mechanismus; ein passender Registry-Wert
  oder eine passende Datei ist für sich genommen kein durch ATT&CK bestätigter Angriff.
- [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
  wird von PyrsistenceSniper ebenfalls als Inspiration für die Untersuchung von Windows-Autostarts genannt.
- [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS) dokumentiert legitime Windows-Binärdateien, die
  missbraucht werden können. Das Erkennen einer dieser Binärdateien hilft bei der Interpretation eines Befehls, macht diesen
  Befehl aber nicht automatisch bösartig.

Zu den Grundlagen für Registry, NTFS, Shell Link, Ereignisprotokolle und weitere Parser, die in ganz Juntei verwendet werden,
siehe die [projektweiten Danksagungen](../acknowledgements.md).

Zurück zum [Leitfaden zu Persistenzmechanismen](index.md).
