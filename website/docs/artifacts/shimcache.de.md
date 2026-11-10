# ShimCache (AppCompatCache) { #shimcache-appcompatcache }

## Was ShimCache ist { #what-shimcache-is }

Das Anwendungskompatibilitäts-Subsystem von Windows speichert Pfade und Metadaten ausführbarer Dateien zwischen, während es
das Kompatibilitäts-/Shim-Verhalten auswertet. Der Cache überdauert das Umbenennen und Entfernen von Dateien sowie das Trennen
eines Wechseldatenträgers. Sein Layout und die verfügbaren Felder unterscheiden sich von Windows XP bis Windows 11.

ShimCache ist wertvoll, um historische Pfade zu entdecken, und – wo das Format ein zuverlässiges Ausführungsflag
besitzt – als zusätzliche Stütze für eine Ausführung. Die zwischengespeicherte Zeit ist normalerweise der **Zeitpunkt der letzten Änderung der Datei**,
nicht der Zeitpunkt, zu dem Windows sie gesehen oder ausgeführt hat.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Windows hat den Pfad im Rahmen der Kompatibilitätsverarbeitung
beobachtet. Ein Eintrag allein ist ein Beleg für Vorhandensein/Beobachtung; verwenden Sie `Executed` nur dort, wo es unterstützt wird.

## Speicherort und Sicherung { #location-and-collection }

Die Bytes befinden sich im Registry-Hive `SYSTEM`, üblicherweise unter:

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

Sichern Sie `SYSTEM`, `SYSTEM.LOG1` und `SYSTEM.LOG2`. Juntei liest alle Control Sets und ermittelt
nach Möglichkeit `Select\Current`.

!!! warning "Die neuesten Live-Einträge befinden sich möglicherweise nicht im Hive"
    Windows kann ShimCache-Aktualisierungen im Arbeitsspeicher halten und sie beim Herunterfahren/Neustart schreiben. Ein von einem
    laufenden System kopierter Hive enthält möglicherweise nicht die neuesten Beobachtungen. Der Zeitpunkt des letzten Schreibzugriffs auf den Cache
    in der Registry hilft zu beschreiben, wann er gespeichert wurde, ist aber keine Ausführungszeit pro Eintrag.

## Hinweise für die Analyse { #analyst-cautions }

- Ein Cache-Eintrag allein ist **kein Beweis für eine Ausführung**.
- `Executed` kann `Yes`, `No` oder `NA` sein; die verfügbare Semantik hängt von Format/Version ab.
- `LastModifiedTimeUTC` gehört zur referenzierten Datei, nicht zur Beobachtung oder Ausführung.
- Die Reihenfolge ist nützlich, aber keine allgemeingültige präzise Zeitleiste; Duplikate treten über Control Sets/Slack hinweg auf.
- `FromSlack`-Einträge wurden außerhalb des aktiven Caches wiederhergestellt und erfordern zusätzliche Vorsicht.
- Ein „dirty“ `SYSTEM` ohne verwendbare Protokolle kann aktuelle Datensätze auslassen. Lesen Sie die Wiederherstellungszusammenfassung.
- Befunde weisen auf verdächtige Namens-/Pfadmuster hin, sind aber keine Malware-Urteile.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shimcache -f SYSTEM -o shimcache.csv
juntei parse-registry shimcache -d evidence/ -t jsonl \
  | jq 'select(.Executed == true or (.Findings | length > 0))'
```

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `ControlSet` | Nummer des Control Sets, das den Cache enthält. Vor der Auswahl `IsCurrent` vergleichen. |
| `CacheEntryPosition` | Position/Reihenfolge im dekodierten Cache. Nützlich für die relative Reihenfolge, keine Zeit. |
| `Path` | Zwischengespeicherter Pfad der ausführbaren Datei/des Pakets. Wichtigster Ansatzpunkt für die Suche. |
| `LastModifiedTimeUTC` | Im Eintrag gespeicherte Änderungszeit der Datei – keine Ausführungs-/Beobachtungszeit. |
| `Executed` | `Yes`, `No` oder `NA` gemäß den unterstützten Formatflags. Nur mit Versionskontext verwenden. |
| `Duplicate` | Derselbe Eintrag wurde bereits in einem früheren Control Set oder im Slack gesehen; hilft, Doppelzählungen zu vermeiden. |
| `SourceFile` | Eingabe-Hive/Rohwert. Unverzichtbar über mehrere Hosts hinweg. |
| `FromSlack` | Aus dem Slack des Cache-Werts statt aus der aktiven Liste wiederhergestellter Eintrag. Geringere Zuverlässigkeit und möglicherweise älter. |
| `CacheFormat` | Dekodiertes Windows-/Cache-Layout. Bestimmt Verfügbarkeit und Semantik der Felder. |
| `CacheLastWrittenUTC` | Zeitpunkt der Speicherung des Registry-Schlüssels/Caches. Gemeinsamer Kontext, nicht die Ausführungszeit jedes Eintrags. |
| `InsertFlags`, `ShimFlags` | Rohe Kompatibilitätsflags; `Executed` ist das von Juntei interpretierte Ergebnis, sofern unterstützt. |
| `DataSize` | Größe der Daten dieses Eintrags, nützlich für die strukturelle Validierung. |
| `Machine` | Von neueren Formaten aufgezeichnete Architektur (`x64`, `x86`, `ARM64`). |
| `FileSize`, `LastUpdateTimeUTC` | Windows-XP-spezifische zwischengespeicherte Metadaten, sofern vorhanden. |
| `PackageName`, `PackageVersion`, `PackagePublisherID` | Identität paketierter Anwendungen in unterstützten Einträgen. Die Herausgeber-ID ist Kontext, kein Signaturnachweis. |
| `Findings` | Anomaliecodes von Juntei, etwa eine verschobene Systembinärdatei oder ein verdächtiger Staging-Pfad. Nur Ansatzpunkte. |
| `RegistryPath`, `RegistryValueName` | Exakte Quelle aus Hive-Schlüssel/-Wert. Für die Validierung beibehalten. |
| `CacheValueSize` | Größe des gesamten Registry-Werts, nicht die `DataSize` des einzelnen Eintrags. |
| `IsCurrent` | Ob das Control Set dieses Eintrags `SYSTEM\Select\Current` entspricht; leer, wenn dies nicht sicher bekannt ist. |
| `SelectCurrent`, `SelectDefault`, `SelectLastKnownGood`, `SelectFailed` | Rohe Control-Set-Nummern aus `SYSTEM\Select` als Kontext und für die Wiederherstellungsanalyse. |

JSON/JSONL bewahrt zusätzlich Eintragsdaten, Tags, Unterdatensätze paketierter Apps, strukturierte Befunde,
Cache-Probleme und Wiederherstellungsdetails.

## Untersuchungsstrategie { #investigation-strategy }

1. Bevorzugen Sie das aktuelle Control Set, bewahren Sie aber andere Sets und Slack als historische Belege auf.
2. Durchsuchen Sie Pfade und Befunde; verwenden Sie `LastModifiedTimeUTC` nicht als Ausführungszeit.
3. Wo `Executed == Yes` gilt, bestätigen Sie dies mit Prefetch, SRUM, Ereignisprotokollen oder anderen Laufzeitspuren.
4. Gleichen Sie Pfade mit `$MFT`/USN ab, um Erstellung, Löschung und Metadatenverlauf zu bestimmen.
5. Geben Sie im Bericht an, ob Transaktionsprotokolle eingespielt wurden und ob der Live-Cache möglicherweise nicht geschrieben wurde.

## Weitere Informationen { #further-details }

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [AppCompatCacheParser von Eric Zimmerman](https://github.com/EricZimmerman/AppCompatCacheParser)
- [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Spezifikation des Windows-Registry-Formats von Maxim Suhanov](https://github.com/msuhanov/regf)
