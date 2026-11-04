# MuiCache { #muicache }

## Was MuiCache ist { #what-muicache-is }

Wenn ein Benutzer ein Programm startet, liest Windows aus den Versionsressourcen des Programms den Namen, den die
Shell anzeigen soll (die `FileDescription`, zum Beispiel „Windows Command Processor“ für
`cmd.exe`), und speichert ihn pro Benutzer in `MuiCache` zwischen. Ab Windows 8 wird zusätzlich das Unternehmen
(`CompanyName`) zwischengespeichert. Der Cache ist nach dem vollständigen Pfad des Programms indiziert:

- **Windows 8 und neuer:** zwei Werte pro Programm, `<path>.FriendlyAppName` und
  `<path>.AppCompany`.
- **Windows Vista und 7:** ein Wert pro Programm, `<path>`, der den Anzeigenamen enthält.
- **Windows XP:** dieselbe Form, in `NTUSER.DAT`.

MuiCache kann Hinweise auf Programmausführungen in der Shell-Sitzung eines Benutzers liefern, einschließlich Programmen
auf Wechseldatenträgern, Netzwerkfreigaben und in nicht mehr vorhandenen Ordnern, mit dem Namen und dem Unternehmen,
die die Datei selbst angegeben hat.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Dieses Shell-Profil hat Pfad,
Beschreibung und Unternehmen einer Datei zwischengespeichert. Korrelieren Sie dies mit anderen Ausführungsartefakten. Das Profil identifiziert nicht
das Konto, unter dem der Prozess ausgeführt wurde. MuiCache hat keine Zeit pro Programm und keinen Ausführungszähler.

## Speicherort { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat     Windows Vista and later
Local Settings\Software\Microsoft\Windows\Shell\MuiCache

C:\Documents and Settings\<user>\NTUSER.DAT                         Windows XP
Software\Microsoft\Windows\ShellNoRoam\MUICache
```

`LangID` (die Sprache der zwischengespeicherten Zeichenfolgen) ist kein Programm und wird ausgelassen. Werte mit Namen der Form
`@<file>,-<number>` (`@shell32.dll,-22019`) sind Ressourcenzeichenfolgen, die die Shell für ihre eigenen
Menüs und Dateitypen zwischengespeichert hat, keine Programme; sie werden ausgelassen und in der Zusammenfassung gezählt. Ab Windows Vista
enthält `NTUSER.DAT` keinen MuiCache.

Sichern Sie `UsrClass.dat` (und `NTUSER.DAT`) mit den Transaktionsprotokollen für jedes Profil sowie die
System-Hives `SYSTEM` und `SOFTWARE` für Host- und Benutzernamen.

## Hinweise für die Analyse { #analyst-cautions }

- **Der Profilinhaber ist nicht notwendigerweise das Prozesskonto.** Ein unter einem anderen
  Konto gestartetes Programm kann ein Fenster in der Sitzung dieses Benutzers anzeigen und Metadaten im Cache dieses Benutzers hinterlassen.
  `User` und `UserSid` identifizieren den Inhaber des Hives, nicht den Sicherheitskontext des Prozesses.
- **Keine Zeit pro Programm.** `KeyLastWrittenUTC` ist der letzte Schreibzugriff auf den Schlüssel: die jüngste Änderung an
  einem seiner Programme, nicht der Zeitpunkt, zu dem ein bestimmtes Programm ausgeführt wurde. Prefetch, BAM, UserAssist und Ereignisprotokolle liefern
  Zeiten.
- **Nicht nur Programme, die der Benutzer von Hand gestartet hat.** Eine DLL oder ein Programm, das die Shell selbst geladen hat (für
  `Open with`, eine Shell-Erweiterung, einen untergeordneten Prozess eines Installationsprogramms), kann erscheinen, etwa `explorerframe.dll` oder
  `shell32.dll`.
- **Der Name stammt aus der Datei.** `FriendlyAppName` und `AppCompany` sind das, was die
  Versionsressourcen der Datei zum Zeitpunkt der Ausführung angaben. Malware kann jede beliebige Beschreibung und jedes Unternehmen angeben; ein umbenanntes
  Werkzeug behält seine ursprüngliche Beschreibung, was es verraten kann (eine Datei namens `svchost.exe`, deren
  Anzeigename ein Fernzugriffswerkzeug ist).
- **Ein Eintrag pro Pfad.** Eine erneute Ausführung desselben Programms fügt keinen Eintrag hinzu; eine Kopie unter einem anderen
  Pfad hingegen schon. Der Eintrag eines Programms kann bestehen bleiben, nachdem die Datei gelöscht wurde.
- **Beide Formen für einen Pfad.** Hat ein Pfad einen Anzeigenamen in beiden Formen (ein aktualisiertes
  Profil), erhält jede Form einen eigenen Datensatz, sodass keiner der Werte verloren geht.
- **Beschädigte Werte.** Ein Wert, der keine Zeichenfolge ist, kein gültiges UTF-16 ist oder nicht vollständig gelesen werden kann, behält
  seine Bytes (`FriendlyAppNameRaw`, `AppCompanyRaw`) und wird in `Problems` gemeldet.
  Ungültiges UTF-16 kann im angezeigten Text auch Ersetzungszeichen erzeugen; verwenden Sie die Rohspalte
  für die ursprünglichen Bytes.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry muicache -d evidence/ -o muicache.csv
juntei parse-registry muicache -f UsrClass.dat -t jsonl \
  | jq 'select(.Path | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Die Zusammenfassung listet die Programme außerhalb der Ordner Windows und Program Files (Benutzerordner,
andere Laufwerke, Netzwerkfreigaben) mit ihren Benutzern sowie die Anzahl der ausgelassenen Ressourcenzeichenfolgen auf.
Program-Files-Ordner mit anderen Namen (unter Windows in anderen Sprachen) werden ebenfalls aufgelistet.
Relative Pfade und Pfade, die aus einem Umgebungsvariablen-Ordner herausführen, bleiben aufgelistet. Gewöhnliche
Laufwerkspfade werden nach dem Auflösen von `.`- und `..`-Komponenten geprüft, ohne auf das Dateisystem des Untersuchungsrechners
zuzugreifen.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, Name des Profilordners, SID des Benutzers (vom Stammschlüssel einer `UsrClass.dat` benannt; sonst aus der `ProfileList` des Systems) und der Hive. |
| `HiveType` | `UsrClass.dat` oder `NTUSER.DAT`. |
| `Path` | Der Pfad des Programms: der Name des Werts ohne `.FriendlyAppName` bzw. `.AppCompany`. |
| `FriendlyAppName` | Die Beschreibung des Programms aus `<path>.FriendlyAppName` oder `<path>`. |
| `AppCompany` | Das Unternehmen des Programms aus `<path>.AppCompany` (Windows 8 und neuer). |
| `KeyPath`, `KeyLastWrittenUTC` | Der MuiCache-Schlüssel und sein letzter Schreibzugriff: die jüngste Änderung an einem seiner Programme. |
| `FriendlyAppNameRaw`, `AppCompanyRaw` | Die Bytes des Werts (hex), wenn sie kein sauber dekodierbarer Text sind. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung: `replayed`, wenn die Protokolle einen der Werte des Programms geschrieben haben. |
| `Problems` | Werte, die keine Zeichenfolgen sind, kein gültiges UTF-16 sind oder nicht gelesen werden können. |

## Untersuchungsstrategie { #investigation-strategy }

1. Betrachten Sie Programme an für Benutzer beschreibbaren Orten (`\Users\`, `Downloads`, `Temp`, `ProgramData`), auf
   anderen Laufwerken und auf Netzwerkfreigaben sowie Administrations-, Fernzugriffs- und Archivierungswerkzeuge.
2. Vergleichen Sie den Dateinamen mit seinem Anzeigenamen und Unternehmen: Eine Abweichung kann ein umbenanntes
   Werkzeug oder eine getarnte Datei aufdecken.
3. Korrelieren Sie mit Prefetch, BAM und UserAssist (wann es ausgeführt wurde), Amcache und ShimCache (Hash und
   Vorhandensein der Datei) sowie der `$MFT` (ob die Datei noch vorhanden ist).
4. Verwenden Sie `parse-registry read --recover-deleted` auf `UsrClass.dat` für gelöschte MuiCache-Werte.

## Weitere Informationen { #further-details }

- [Microsoft: VERSIONINFO-Ressource](https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource)
- [Cyber Triage: MuiCache-Experimente, GUI-Sitzungen und Einschränkungen beim Prozesskonto](https://www.cybertriage.com/blog/muicache-2026-guide/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
