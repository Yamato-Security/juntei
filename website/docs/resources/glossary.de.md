# DFIR-Glossar { #dfir-glossary }

| Begriff | Bedeutung in einfachen Worten |
|---|---|
| Allocated (zugeordnet) | Der Datensatz/das Objekt ist laut Artefakt aktuell in Verwendung. |
| Alternate Data Stream (ADS) | Benannte NTFS-Daten, die einer Datei zusätzlich zu ihrem normalen unbenannten Inhalt angehängt sind, etwa `Zone.Identifier`. |
| Artefakt | Persistente Daten, die von einem Betriebssystem/einer Anwendung erzeugt werden und forensische Fragen beantworten können. |
| Carving | Auffinden von Datensätzen anhand ihrer Struktur/Signatur, ohne sich auf ihren normalen Container/Index zu stützen. |
| Steuerungssatz (Control Set) | Eine Version der SYSTEM-Konfiguration (`ControlSet001` usw.); `Select\Current` benennt den aktiven. |
| Untermauerung (Corroboration) | Unabhängige Beweise, die dieselbe Schlussfolgerung stützen. |
| „Dirty“ Datenbank/Hive | Die primäre Datei war bei der Sicherung nicht vollständig synchronisiert; Transaktionsprotokolle können den erforderlichen Zustand enthalten. |
| Droid | GUID-ähnlicher Bezeichner der verteilten Linkverfolgung, mit dem Dateien/Volumes über Verschiebungen hinweg verfolgt werden. |
| ESE / JET Blue | Microsofts Datenbankformat Extensible Storage Engine, das von SRUM und anderen Windows-Daten verwendet wird. |
| FILETIME | Anzahl von 100-Nanosekunden-Intervallen seit dem 1601-01-01 UTC; eine Zahl ist nur dann als Zeit aussagekräftig, wenn ihr Schema dies festlegt. |
| Dateireferenz | NTFS-Eintragsnummer plus Sequenznummer, die eine Generation eines MFT-Datensatzes identifiziert. |
| Hive | Binärdatei, die einen Baum der Windows-Registry enthält. |
| INDX-Datensatz | Block (meist 4 KiB) eines NTFS-Verzeichnisindex (`$I30`), der die Dateien des Verzeichnisses auflistet; sein Slack kann mögliche Beweise für gelöschte oder umbenannte Dateinamen bewahren. |
| LNK | Windows-Shell-Link/Verknüpfung mit einem Ziel und umfangreichen Metadaten. |
| MRU | Reihenfolge „Most Recently Used“ (zuletzt verwendet); in der Regel eine relative Reihenfolge statt eines Zeitstempels für jedes Element. |
| MFT | NTFS Master File Table mit Datei-/Verzeichnisdatensätzen und Attributen. |
| Prefetch-Hash | Windows-Bezeichner, der in der Benennung von `.pf`-Dateien verwendet wird; kein kryptografischer Inhalts-Hash. |
| Letzter Schreibzugriff auf einen Registry-Schlüssel | Zeitpunkt, zu dem sich der Schlüssel geändert hat; Windows gibt in der Regel nicht an, welcher Wert sich geändert hat. |
| Residente Daten | Datei-/Stream-Bytes, die direkt in einem MFT-Attribut statt in externen Data Runs gespeichert sind. |
| RID | Relative Identifier, die abschließende Konto-/Gruppenkomponente unterhalb einer Windows-SID-Autorität/-Domäne. |
| SDDL | Security Descriptor Definition Language: eine als Text geschriebene Sicherheitsbeschreibung, etwa `O:S-1-5-18D:(A;;0x001f01ff;;;S-1-5-32-544)`. |
| Sicherheitsbeschreibung (Security Descriptor) | Besitzer, primäre Gruppe, DACL (wer was tun darf) und SACL (Überwachung und Integritätsbezeichnung) einer Datei. NTFS speichert jede eindeutige Sicherheitsbeschreibung einmal in `$Secure:$SDS`, unter einer Sicherheits-ID. |
| SID | Windows Security Identifier für ein Konto, einen Dienst, eine Gruppe oder einen Sicherheitsprinzipal. |
| Slack | Ungenutzte Bytes innerhalb/hinter einer zugeordneten Struktur, die ältere Daten bewahren können. |
| SRUM | Datenbank des System Resource Usage Monitor zur Ressourcennutzung von Anwendungen/Benutzern. |
| Timestomping | Absichtliche Änderung von Zeitstempeln, um eine Untersuchung in die Irre zu führen; Heuristiken sind kein Beweis. |
| Transaktionsprotokoll | Ausstehende/festgeschriebene Änderungen, mit denen eine Registry oder Datenbank in einen konsistenten Zustand zurückversetzt wird. |
| USN | Update Sequence Number, die von Datensätzen des NTFS-Änderungsjournals und zugehörigen Metadaten verwendet wird. |
| UTC | Koordinierte Weltzeit; bevorzugte gemeinsame Zeitbasis für systemübergreifende Zeitleisten. |
| Volume-Seriennummer | Nummer, die NTFS einem Volume bei der Formatierung zuweist (`$Boot`); ihre unteren 32 Bit sind das, was LNK-, Jump-List- und Prefetch-Dateien über das Volume einer Datei aufzeichnen. |
