# Amcache { #amcache }

## Was Amcache ist { #what-amcache-is }

Amcache ist ein Inventar der Windows-Anwendungskompatibilität, das in einem Registry-Hive gespeichert wird. Moderne Systeme
zeichnen ausführbare Dateien unter `InventoryApplicationFile` und Plug-and-Play-Geräte unter
`InventoryDevicePnp` auf; ältere Windows-Versionen verwenden ein Legacy-Layout unter `Root\File`.

Für die Untersuchung ausführbarer Dateien ist Amcache wertvoll, weil es einen vollständigen Pfad, SHA-1,
Dateigröße, PE-Metadaten, Versionsinformationen, Herausgebertext, Link-Datum und Registry-Zeitstempel
bewahren kann, nachdem die ursprüngliche Datei verschwunden ist. Gerätedatensätze können angeschlossene Hardware, Treiber,
Installationsdaten und Kennungen identifizieren.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Windows hat die aufgezeichnete Datei bzw. das
Gerät inventarisiert. Ein Datensatz zu einer ausführbaren Datei ist ein **Beleg für das Vorhandensein, kein Beweis für die Ausführung**.

## Speicherort und Sicherung { #location-and-collection }

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

Sichern Sie den Hive und die benachbarten Dateien `Amcache.hve.LOG`, `.LOG1` und `.LOG2`. Bei einem nicht konsistent gespeicherten Hive spielt `parse-registry amcache` verwendbare Transaktionsprotokolle vor dem Parsen automatisch im Arbeitsspeicher ein, ohne die Beweise zu ändern. Wiederherstellungsdetails stehen im Parserprotokoll oder Terminal; bei Datenausgabe auf stdout gehen Diagnosen an stderr. Fehlende oder unbrauchbare Protokolle können unvollständige Datensätze oder einen nicht lesbaren Hive verursachen.

## Hinweise für die Analyse { #analyst-cautions }

- Ein Amcache-Eintrag zu einer ausführbaren Datei beweist nicht, dass ein Benutzer die Datei gestartet hat.
- `Publisher` sind von Windows aufgezeichnete Metadaten aus der PE-/Versionsressource. Sie beweisen nicht, dass die Datei
  zum Zeitpunkt der Sicherung eine gültige Authenticode-Signatur hatte.
- Hashes können fehlen, und Legacy- und moderne Schemata stellen nicht dieselben Werte bereit.
- `LinkDate` stammt aus dem PE-Header und kann gefälscht, kopiert oder durch reproduzierbare Builds erzeugt sein.
- Der Zeitpunkt des letzten Schreibzugriffs auf einen Schlüssel beschreibt die Aktualisierung des Registry-Datensatzes, nicht notwendigerweise Download, Erstellung,
  erste oder letzte Ausführung.
- Pfade können auf Wechseldatenträger oder nicht mehr vorhandene Volumes verweisen.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.RecordType == "file" and .SHA1 == "0123456789abcdef0123456789abcdef01234567")'
```

Die CSV-Ausgabe enthält Tabellen für Zusammenfassung, Befunde, Dateien und Geräte. Mit `auto-parse` sind dies
`Registry-Amcache-Summary.csv`, `Registry-Amcache-Findings.csv`, `Registry-Amcache-Files.csv` und
`Registry-Amcache-Devices.csv` direkt im Verzeichnis `FileExistence/` des jeweiligen Computers, mit
`Registry-Amcache-logs.txt` neben den Tabellen. Bei mehreren Hives wird vor der Dateierweiterung `-Source1`, `-Source2` usw.
angefügt; `OutputFiles` in den Manifesten listet die exakten Pfade auf. Eigenständige CSV-Exporte
behalten ihre nativen Tabellennamen und den angeforderten Ausgabeort. JSONL-Datensätze verwenden
die `RecordType`-Werte `summary`, `finding`, `file` und `device`.

## Ausgabefelder für Dateien { #file-output-fields }

| Feld | Verwendung in der Analyse |
|---|---|
| `KeyName`, `SourceKeyPath` | Identifizieren den exakten Registry-Quelldatensatz und den Schemazweig. Für die Reproduzierbarkeit beibehalten. |
| `KeyLastWrittenUTC`, `KeyLastWrittenFiletime` | Aktualisierungszeit des Registry-Datensatzes, lesbar und roh. Nicht als Ausführungszeit bezeichnen. |
| `SourceVolumeKeyLastWrittenUTC`, `SourceVolumeKeyLastWrittenFiletime` | Zeit des übergeordneten Volume-Schlüssels in Legacy-Schemata; Kontext, keine Zeit einer Dateiaktion. |
| `FileID` | Von Amcache aufgezeichnete Dateikennung. Exakt beibehalten; nicht annehmen, dass es immer ein SHA-1 ist. |
| `SHA1`, `SHA256` | Starke Ansatzpunkte für Defender, Threat Intelligence, EDR und Dateirepositorys. Null bedeutet nicht aufgezeichnet. |
| `Name`, `OriginalFileName`, `Description` | Aufgezeichnete PE-/Versionsnamen. Mit dem tatsächlichen Pfad vergleichen, um Tarnung (Masquerading) zu erkennen; die Metadaten sind vom Angreifer kontrollierbar. |
| `FullPath` | Nützlichster Ansatzpunkt für den Speicherort. Groß-/Kleinschreibung sorgfältig normalisieren, aber die ursprüngliche Beweiszeichenfolge beibehalten. |
| `LongPathHash` | Pfadbezogene Kennung von Windows; nützlich zum Abgleich von Datensätzen, kein Inhaltshash. |
| `ProgramID` | Verknüpft die Datei, sofern verfügbar, mit einem Datensatz im Anwendungsinventar. |
| `Publisher` | Aufgezeichneter Herausgebertext. Als Metadaten behandeln, nicht als Signaturprüfung. |
| `Version`, `BinFileVersion`, `ProductName`, `ProductVersion`, `BinProductVersion` | Kontext aus der Versionsressource. Abweichungen oder unplausible Werte können eine Masquerading-Analyse stützen. |
| `BinaryType` | Aufgezeichneter Architektur-/Typkontext; mit einer PE-Analyse vergleichen, wenn die Datei wiederhergestellt wird. |
| `LinkDate` | PE-Kompilierungs-/Link-Zeitstempel wie aufgezeichnet. Nur mit Skepsis und Bestätigung durch weitere Quellen nützlich. |
| `Size` | Aufgezeichnete Größe; mit `$MFT`, Defender-Inhalten oder einer wiederhergestellten Datei vergleichen. |
| `Language` | PE-Sprachkennung; ein schwacher Kontexthinweis, keine Attribution. |
| `IsPE-File`, `IsOS-Component` | Von Windows aufgezeichnete Klassifizierungsflags. Ein falscher Wert beweist keine Harmlosigkeit. |
| `USN` | Von Amcache aufgezeichneter NTFS-Update-Sequence-Wert; kann helfen, zu erhaltenen USN-Datensätzen zu pivotieren. |
| `APPX-PackageFullName`, `APPX-PackageRelativeID` | Ordnet paketierte Anwendungen/Komponenten der Datei zu. |
| `RegistryValues` | Jeder ursprüngliche Registry-Wert mit Typ, dekodiertem Wert und rohem Hex. Verwenden, wenn ein typisiertes Feld fehlt, oder für zukünftige/Legacy-Felder. |

## Ausgabefelder für Geräte { #device-output-fields }

| Feldgruppe | Felder und Interpretation |
|---|---|
| Quelle | `KeyName`, `SourceKeyPath`, `KeyLastWrittenUTC`, `KeyLastWrittenFiletime`: exakter Gerätedatensatz und Aktualisierungszeit in der Registry. |
| Identität | `Description`, `BusDescription`, `Manufacturer`, `Model`: für Menschen lesbare Gerätebeschreibungen; Herstellerzeichenfolgen können generisch sein. |
| Hardwarekennungen | `HWID`, `CompID`, `Class`, `ClassGUID`, `ContainerID`, `DeviceInterfaceClasses`: stärkste Ansatzpunkte für Gerätefamilie und Geräteinstanz. |
| Zustand | `DeviceState`, `ProblemCode`, `ParentID`, `MatchingID`, `Enumerator`: Kontext zur Windows-Enumeration und zu Beziehungen. |
| Treiber | `DriverID`, `DriverName`, `DriverPackageStrongName`, `DriverVerDate`, `DriverVerVersion`, `Provider`, `Service`, `INF`: Zuordnungen von Treiberpaket und Dienst; mit installierten Dateien und Diensten korrelieren. |
| Installation | `InstallDate`, `FirstInstallDate`, `InstallState`, `ExtendedInfs`: aufgezeichneter Installationslebenszyklus. Ein fehlendes Datum beweist nicht, dass das Gerät nie angeschlossen war. |
| Filter und Stack | `LowerClassFilters`, `UpperClassFilters`, `LowerFilters`, `UpperFilters`, `StackID`, `LocationPaths`: erweiterter Kontext zum Treiberstack; unerwartete Filtertreiber verdienen eine Prüfung. |
| Rohdatensicherung | `RegistryValues`: vollständige typisierte/rohe Quellwerte zur Validierung und für nicht unterstützte Felder. |

Array- und Rohwertspalten in CSV verwenden ein `JSON`-Suffix (zum Beispiel `HWID-JSON` und
`RegistryValuesJSON`); ihre Zellen enthalten JSON, während JSON/JSONL native Arrays beibehält.

## Untersuchungsstrategie { #investigation-strategy }

1. Pivotieren Sie über `FullPath`, `SHA1`, `Name` und `USN`.
2. Suchen Sie nach passenden Prefetch-Dateien, um eine Ausführung zu stützen.
3. Verwenden Sie `$MFT` und das USN-Journal, um Erstellung, Umbenennung, Löschung und Größe der Datei festzustellen.
4. Vergleichen Sie Herausgeber-/Versionsmetadaten mit einer unabhängig wiederhergestellten PE-Datei und einer Signaturprüfung.
5. Prüfen Sie Defender-Quarantäne und Ereignisprotokolle auf Erkennung oder Entfernung.

## Weitere Informationen { #further-details }

- [Ausgabeschema von Junteis ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [Regipy-Amcache-Plugin](https://github.com/mkorman90/regipy)
