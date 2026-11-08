# Parservalidierung { #parser-validation }

Die von Juntei gepflegten Parser-Crates werden mit öffentlichem Beweismaterial und unabhängigen
Implementierungen getestet. Diese Vergleiche helfen, Regressionen und Interpretationsunterschiede zu erkennen; sie
machen keinen Parser unfehlbar. Bewahren Sie die Quellbeweise auf, prüfen Sie die Problem- bzw.
Schadensfelder jedes Datensatzes und validieren Sie folgenreiche Befunde mit einem anderen Artefakt oder einer anderen Implementierung.

Die folgenden Aussagen bewahren die Validierungsinformationen, die früher in der Haupt-README von Juntei
standen. Aktuelle Testdaten und Implementierungsdetails finden Sie in den verlinkten Parser-Repositorys.

## Amcache { #amcache }

Juntei verwendet [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser), einen
Apache-2.0-Fork von
[amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) von Security Ronin. Seine
[Ausgabedokumentation](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
definiert jedes Feld zu ausführbaren Dateien, Geräten, Befunden und Zusammenfassungen.

## Master File Table { #master-file-table }

Das Crate mft von [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) ist ein gepflegter Fork des Crates
[mft](https://github.com/omerbenamram/mft) von Omer Ben-Amram. Über acht `$MFT`-Dateien hinweg – das Beispiel des
Crates, den NIST Hacking Case und sechs Dateien aus dem DFIR Artifact Museum – stimmten 1.048.733 CSV-Zeilen und -Spalten
in der Reihenfolge mit einem Referenzparser überein. Die Vergleiche umfassten die Standardausgabe, `--all-times`,
`--short-names` und `--cutoff`; die Darstellung der Zeitstempel war der beabsichtigte Unterschied.

## NTFS-Transaktionsprotokoll ($LogFile) { #ntfs-transaction-log-logfile }

Das Crate logfile von [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) wird auf Basis von
[Sootmark ntfslog](https://github.com/Sootmark/ntfslog) gepflegt. Seine Validierung umfasst sechs vollständige Protokolle aus dem DFIR
Artifact Museum, öffentliche Testdaten der Versionen 1.1 und 2.0, dekodierte Redo/Undo-Metadaten sowie Journal-Schreibvorgänge,
die mit unabhängig geparsten NTFS-Daten verglichen wurden. Detaillierte Angaben zur Abdeckung und bekannte Unterschiede finden Sie im
Parser-Repository.

Juntei ergänzt CLI-Tests, die auf dem öffentlichen go-ntfs-Testvolume (2.129 Datensätze) jedes CSV-Feld mit JSON und JSONL
vergleichen, separate Redo/Undo-Daten und Längen der Roh-Payloads prüfen, beschädigte Protokolle testen,
das Überschreiben von Beweisen verweigern und verhindern, dass aktuelle MFT-Pfade wiederverwendeten Datensatznummern folgen.
Regressionsfälle unterscheiden betroffene MFT-Datensätze von dem geöffneten Attribut, über das sie schreiben,
prüfen flache Metadaten von Verzeichniseinträgen/Datenattributen und verhindern die volumeübergreifende Zuordnung von MFTs.
Tests von `auto-parse` mit mehreren Volumes decken separate CSV-Dateien und eine einzelne JSONL-Datei mit Herkunftsangaben ab.

Der Befehl wurde außerdem an sechs privat gesicherten Windows-Protokollen geprüft: Alle 708.465 exportierten CSV-Datensätze
stimmten Feld für Feld mit JSONL überein, und die Exporte von `auto-parse` pro Host behielten dieselbe Anzahl an Datensätzen.
Die Quellbeweise werden nicht mit dem Repository verteilt. Diese Prüfungen bestätigen die Konsistenz der Ausgabe,
nicht, dass jede protokollierte Transaktion eine abgeschlossene Benutzeraktion darstellt.

## Prefetch { #prefetch }

[ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) ist ein gepflegter Fork von
[frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) von ForensicRS. Seine dekodierte Ausgabe wurde
mit [libscca](https://github.com/libyal/libscca) verglichen, der unabhängigen Prefetch-Implementierung von libyal.

## USN-Journal { #usn-journal }

Das Crate usn von [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) ist ein gepflegter
Fork von [Sootmark usn](https://github.com/Sootmark/usn). Bei sechs echten Journalen stimmten Junteis
zeitbezogene Pfade übergeordneter Verzeichnisse überall dort mit einem unabhängigen Parser überein, wo dieser Parser einen Pfad auflöste. Juntei
löste zudem 4.321 Datensätze auf, die im Vergleich ungelöst blieben. Sein Carver erzeugte keine falschen
Datensätze, als er mit 880 MB `$MFT`-Daten getestet wurde.

## Verzeichnisindizes ($I30) { #directory-indexes-i30 }

Das Modul `i30` des Crates mft von [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) liest INDX-Datensätze
und deren Slack. Bei zwei `$I30`-Streams eines mit ntfs-3g erstellten Volumes (Junteis
`test_files/i30/`) und bei drei öffentlichen, von Windows geschriebenen `$I30`-Beispielen stimmten seine verwendeten Einträge und die Einträge im
Slack mit denen von [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) von Maxim Suhanov überein, mit
denselben Namen und Zeitstempeln. Wo dfir_ntfs eine Referenz auf das übergeordnete Verzeichnis jenseits von 32 Bit meldet (Bytes, die das
übergeordnete Verzeichnis überschrieben haben), lässt Juntei das übergeordnete Verzeichnis leer.

## Bootsektor ($Boot) { #boot-sector-boot }

Das Crate boot von [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) ist eine Eigenentwicklung von Yamato
Security. Bei den fünf öffentlichen `$Boot`-Dateien des DFIR Artifact Museum (Windows 10, 11 und
Server 2012 R2) stimmte jedes Feld mit einem Referenzparser überein. Bei mit `mkntfs` erstellten Volumes mit Sektoren von 512,
1.024 und 4.096 Byte und Clustern von 4 KiB bis 64 KiB stimmten die Größen, die Cluster von `$MFT` und `$MFTMirr`
sowie die Seriennummern mit `fsstat` aus The Sleuth Kit überein. Der Windows-11-`$Boot` des Museums
hat die Volume-Seriennummer, die die LNK-Datei desselben Hosts aufzeichnet, und die fünf Dateien haben
denselben Bootcode.

## Sicherheitsbeschreibungen ($Secure:$SDS) { #security-descriptors-securesds }

Das Crate secure von [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) ist eine Eigenentwicklung von Yamato
Security. Über die fünf `$SDS`-Streams des Museums hinweg (7.808 Sicherheitsbeschreibungen, 60.781 ACEs) stimmten die
Sicherheits-IDs, Hashes, Offsets, Besitzer, Gruppen, Steuerungs-Flags und ACE-Anzahlen mit einem Referenzparser
überein, und Typ, Flags, Maske, SID und Objekttypen jedes ACE stimmten mit `dissect.ntfs` überein. Jeder
aufgezeichnete Hash gehört zu seiner Sicherheitsbeschreibung, jede Spiegelkopie stimmt mit ihrem Eintrag überein, und jede Sicherheitsbeschreibung hat
SDDL, einschließlich der Bedingungen ihrer 3.384 Callback-ACEs.

## SRUM { #srum }

[ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) basiert auf den Crates
[ese](https://github.com/Sootmark/ese) und [srum](https://github.com/Sootmark/srum) von Sootmark. Über
11 echte Datenbanken hinweg stimmte jede Spalte der 45.255 ausgegebenen Einträge mit einer unabhängigen Dekodierung
der von `dissect.esedb` bereitgestellten Werte überein.

## Benutzerzugriffsprotokollierung { #user-access-logging }

`parse-ese ual` liest UAL-Datenbanken mit demselben ESE-Leser. Über 16 echte Datenbanken hinweg (308 Zeilen, davon
145 Client-Zeilen) stimmte jeder Wert mit einer unabhängigen Dekodierung der von `dissect.esedb` bereitgestellten
Werte überein. In jeder Client-Zeile reichten die Tage mit Anzahlen vom UTC-Tag von `InsertDate` bis zum UTC-Tag von
`LastAccess` und ergaben in der Summe `TotalAccesses`.

## Webbrowser { #web-browsers }

[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) ist ein gepflegter Fork der Crates
browser, sqlite und plist von Sootmark, ausgewählt nach einem Vergleich mit einer anderen Rust-Bibliothek sowie mit
Plaso, Velociraptor (SQLiteHunter) und Hindsight anhand derselben Datenbanken. `parse-sqlite browser` wurde anhand
von 38 öffentlichen Datenbanken mit der Bibliothek `sqlite3` verglichen, die jeweils eine Kopie jeder Datenbank mit
ihrer `-wal`-Datei las: die Testdateien von Plaso (Chrome 8 bis 68, Firefox 2 bis 118), mit Chromium 150 und
Firefox 140 erstellte Profile sowie die Testdatenbanken von ys-browser-parser für den Formularverlauf. Alle 293
Besuche, 21 Downloads, 692 Cookies und 41 Formulareinträge stimmten überein. Gelöschte Einträge wurden nur dort
wiederhergestellt, wo Einträge gelöscht worden waren: die zwei Seiten und zwei Besuche, die aus dem Verlauf von
Firefox 140 gelöscht worden waren, mit ihren Zeiten, URLs und Titeln, sowie die gelöschten Formulareinträge der
Testdatenbanken.

## Internet Explorer und Edge Legacy (WebCache) { #internet-explorer-and-legacy-edge-webcache }

`parse-ese webcache` liest `WebCacheV01.dat` mit
[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser). Es wurde anhand der zwei öffentlichen
WebCache-Datenbanken von Plaso mit dem ESE-Leser von `dissect.database` verglichen: einer von Internet Explorer 11
und einer „dirty“ Datenbank von Windows 10 mit Verlauf, Downloads und Cookies des alten Edge. Alle 4.156 Zeilen
stimmten Wert für Wert überein: 44 Container, 182 Verlaufseinträge, 6 Downloads, 170 Cookies und 3.754 weitere
Einträge. Der Vergleich dekodierte die Download-Datensätze eigenständig. Datensätze, die die Datenbanken als gelöscht
kennzeichnen, wurden bei beiden ausgelassen.

## LNK-Verknüpfungen { #lnk-shortcuts }

[ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) ist ein gepflegter Fork des Crates
[shell](https://github.com/Sootmark/shell) von Sootmark. Er wurde anhand von 1.118 öffentlichen
Verknüpfungen verglichen. Von den 30 mit dem Referenzparser gemeinsamen CSV-Spalten wichen 16 einzelne Werte ab:
13 Zielpfade mit beschädigten, abgeschnittenen, Platzhalter-, MTP- oder Systemsteuerungselementen; zwei
Listen zusätzlicher Datenblöcke mit unbekannten oder abgeschnittenen Blöcken; und eine MFT-Referenz, die der
Referenzparser aus anderen Bytes las. Juntei meldet beschädigte Strukturen, statt sie stillschweigend
zu normalisieren.

## Jump Lists { #jump-lists }

[ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) nutzt die Arbeiten von Sootmark zu
Compound Files und Jump Lists, wobei eingebettete Links von `ys-lnk-parser` dekodiert werden. Über 77 öffentliche
Jump Lists hinweg stimmten alle 13.057 gemeinsamen Vergleichszellen – einschließlich Zeitstempeln auf 100 ns genau – mit dem
Referenzparser überein. Sechs Dateien, die die Referenz ablehnte (leere benutzerdefinierte Listen, eine fehlende Fußzeile oder eine Datei,
die keine Jump List war), wurden von Juntei entweder dekodiert oder ausdrücklich gemeldet.

## ShimCache { #shimcache }

[ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) ist ein gepflegter Fork
von
[shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) von Security Ronin, mit Unterstützung für Registry-Hives und
Transaktionsprotokolle. Bei 20 öffentlichen `SYSTEM`-Hives (24 Steuerungssätze und 7.290 Einträge) sowie 17
eigenständigen AppCompatCache-Werten stimmte jeder Eintrag in Pfad, Zeitstempel, Ausführungs-Flag, Flags und
Position mit der Ausgabe des Referenzparsers überein.

## Microsoft-Defender-Quarantäne { #microsoft-defender-quarantine }

[ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) ist ein
gepflegter Fork des Crates
[defender](https://github.com/Sootmark/defender) von Sootmark. Ein Quarantäneordner, den die Testwerkzeuge des Parsers im Defender-Format
geschrieben haben, wird vom unabhängigen Quarantäne-Reader von
`dissect.target` übereinstimmend dekodiert.

## Aus der Registry abgeleitete Artefakte { #registry-derived-artifacts }

[ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) wird mit
[yarp](https://github.com/msuhanov/yarp) verglichen, und zwar bei zugeordneten Schlüsseln, Werten, dem Einspielen von Transaktionen bei „dirty“ Hives,
der Wiederherstellung freier Zellen, Zell-Slack, Daten jenseits der Hive-Bins sowie Datensätzen, die durch das Einspielen entfernt oder ersetzt wurden.
Artefaktspezifische Vergleiche umfassen:

- Dienste, Werte und Fehlerbehandlungsaktionen aus `SYSTEM`-Hives von Windows 7 und aus „dirty“ Windows-11-Hives;
- mit yarp und libfwsi dekodierte RecentDocs über neun öffentliche `NTUSER.DAT`-Hives hinweg;
- unabhängig mit yarp dekodierte geplante Aufgaben über sieben öffentliche `SOFTWARE`-Hives hinweg, die
  Windows 7–11, Server 2012 R2 und Server 2022 abdecken; sowie
- mit `dissect.target` verglichene SAM-Kontostrukturen über sieben öffentliche `SAM`-Hives hinweg.

Die vollständige Liste der Referenzwerkzeuge, Formatforschung, Spezifikationen und öffentlichen Beweisquellen finden Sie unter
[Danksagungen](../acknowledgements.md).
