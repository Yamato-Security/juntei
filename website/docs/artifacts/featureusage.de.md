# FeatureUsage, RecentApps und an die Taskleiste angeheftete Elemente { #featureusage-recentapps-and-taskbar-pins }

## Was sie sind { #what-they-are }

Der Explorer führt in der `NTUSER.DAT` des Benutzers mehrere Aufzeichnungen darüber, wie ein Benutzer Anwendungen verwendet:

- **FeatureUsage** (Windows 10 und neuer): Zähler pro Anwendung, nach Art der Verwendung.
  - `AppSwitched`: wie oft der Benutzer zum Fenster der Anwendung gewechselt ist (Klick auf die Taskleiste,
    Alt+Tab).
  - `AppLaunch`: wie oft sie über die Taskleiste gestartet wurde (ein angeheftetes Element).
  - `ShowJumpView`: wie oft ihre Jump List geöffnet wurde (Rechtsklick auf die Taskleiste).
  - `AppBadgeUpdated`: wie oft sich ihr Taskleisten-Badge (eine Anzahl ungelesener Elemente) geändert hat.
  - `TrayButtonClicked`: wie oft jede Taskleistenschaltfläche (`StartButton`, `SearchButton`,
    `TaskViewButton`, `ClockButton`, …) angeklickt wurde.

  Eine Anwendung ist eine Application User Model ID (`Microsoft.Windows.Explorer`,
  `Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`), ein Pfad, dessen erster Teil die GUID eines bekannten
  Ordners sein kann (`{1AC14E77-…}\cmd.exe` ist `%windir%\system32\cmd.exe`), oder eine Prozess-ID
  (`*PID00001d74`) für Fenster, die der Explorer keiner Anwendung zuordnen konnte.
- **RecentApps** (Windows 10 bis 1709): die Anwendungen, die ein Benutzer kürzlich ausgeführt hat, mit einem
  Startzähler und dem letzten Zeitpunkt; unter jeder die Dateien, die sie geöffnet hat (`RecentItems`).
- **An die Taskleiste angeheftete Elemente** (`Taskband`): die an die Taskleiste angehefteten Programme als Shell-Elemente
  (`Favorites`) und als Link jeder angehefteten Verknüpfung (`FavoritesResolve`). Ab Windows 10
  gibt jedes angeheftete Element an, ob Windows es angeheftet hat (`SystemPinned`) oder der Benutzer (`UserPinned`).
- **ProgramsCache** (`StartPage`, `StartPage2`; Windows XP bis 7): die zwischengespeicherten Programme des Startmenüs
  (`ProgramsCache`) und unter Windows 7 die an das Startmenü und die Taskleiste angehefteten Programme
  (`ProgramsCacheSMP`, `ProgramsCacheTBP`).

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Dieser Benutzer hat diese Anwendung so
oft auf diese Weise verwendet (FeatureUsage), sie zu diesem Zeitpunkt ausgeführt (RecentApps) oder sie angeheftet. Die
Zähler haben keine eigene Zeit; nur der letzte Schreibzugriff auf den Schlüssel und die Zeiten von RecentApps datieren
etwas.

## Speicherort { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

Die `KeyCreationTime` von `FeatureUsage` ist der Zeitpunkt, ab dem gezählt wurde; die Zusammenfassung zeigt ihn an. Sichern Sie
`NTUSER.DAT` mit den Transaktionsprotokollen für jedes Profil sowie die System-Hives `SYSTEM` und
`SOFTWARE` für den Host und die SID des Benutzers.

## Hinweise für die Analyse { #analyst-cautions }

- **Zähler ohne Zeiten.** Ein FeatureUsage-Zähler sagt, wie oft, nicht wann. Der Zeitpunkt des letzten Schreibzugriffs
  auf den Schlüssel der jeweiligen Art (`KeyLastWrittenUTC`) ist die jüngste Änderung an einem ihrer Zähler.
- **Nicht jede Ausführung.** `AppSwitched` zählt Fensterwechsel, keine Starts; eine Anwendung, die
  ohne Fenster lief oder zu der der Benutzer nie gewechselt ist, wird nicht gezählt. `AppLaunch` zählt nur
  Starts über die Taskleiste.
- **Paketierte Apps und Pfade.** Eine Anwendung wird so benannt, wie Windows ihre Fenster gruppiert hat: die ID einer
  paketierten App, eine ID, die ein Programm für sich selbst festgelegt hat (`Chrome`, `VMware.Workstation.vmui`), oder ein
  Pfad. `AppPath` gibt den Pfad an, wobei die GUID eines bekannten Ordners durch den Standardpfad des Ordners
  ersetzt ist.
- **Standardmäßig angeheftete Elemente.** Neue Profile erhalten standardmäßig angeheftete Elemente (Datei-Explorer, Edge, Store); ab Windows 10
  sind sie als `SystemPinned` gekennzeichnet. Ein als `UserPinned` angeheftetes Element ist die Wahl des Benutzers.
- **Angeheftete Elemente in zwei Listen.** `Favorites` und `FavoritesResolve` beschreiben dieselben angehefteten Elemente in derselben
  Reihenfolge (`Index`), werden aber als separate Datensätze ausgegeben.
- **ProgramsCache-Pfade** sind relativ: Auf einen Ordner (`Programs\Accessories`) folgen die
  darin enthaltenen Verknüpfungen (`Notepad.lnk`).
- **Beschädigte Werte.** Ein Zähler, der kein `REG_DWORD` ist, eine Zeit, die kein `REG_QWORD` ist, ein angeheftetes Element,
  das nicht verstanden wird, oder ein Link, der keiner ist, behält seine Bytes (`DataRaw`) und wird in
  `Problems` gemeldet.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry feature-usage <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry feature-usage -d evidence/ -o featureusage.csv
juntei parse-registry feature-usage -f NTUSER.DAT -t jsonl \
  | jq 'select(.Source == "AppSwitched") | [.Count, .AppPath // .App]'
```

Die Zusammenfassung listet Datensätze nach Quelle auf, wann die FeatureUsage-Zählung jedes Hives begonnen hat, die
Anwendungen, zu denen am häufigsten gewechselt wurde, die zuletzt verwendeten Apps und Elemente (neueste zuerst) sowie die an die Taskleiste angehefteten Elemente.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, Name des Profilordners, SID des Benutzers aus der `ProfileList` des Systems und der Hive. |
| `Source` | `AppSwitched`, `AppLaunch`, `ShowJumpView`, `AppBadgeUpdated`, `TrayButtonClicked` (FeatureUsage), `RecentApps`, `RecentItems`, `Taskband` oder `ProgramsCache`. |
| `App` | Name eines FeatureUsage-Werts, `AppId` einer zuletzt verwendeten App oder Application User Model ID eines angehefteten Elements. |
| `AppPath` | `App` als Pfad, wobei die GUID eines bekannten Ordners durch dessen Standardpfad ersetzt ist; leer bei IDs. |
| `Count` | Ein FeatureUsage-Zähler oder der `LaunchCount` einer zuletzt verwendeten App. |
| `LastAccessedUTC` | `LastAccessedTime` einer zuletzt verwendeten App oder eines zuletzt verwendeten Elements. |
| `ItemPath` | `Path` eines zuletzt verwendeten Elements (oder `AppPath` einer zuletzt verwendeten App); Shell-Element-Pfad eines angehefteten Elements oder zwischengespeicherten Programms (`User Pinned\TaskBar\File Explorer.lnk`, `Applications\<app ID>`); Pfad eines `FavoritesResolve`-Links. |
| `DisplayName` | `DisplayName` eines zuletzt verwendeten Elements. |
| `PinType` | `SystemPinned` oder `UserPinned` für ein angeheftetes Element aus `Favorites`. |
| `Index` | Position eines angehefteten Elements oder zwischengespeicherten Programms in seiner Liste. |
| `ValueName`, `KeyPath`, `KeyLastWrittenUTC` | Der Wert und der Schlüssel, aus denen der Datensatz stammt, und der letzte Schreibzugriff auf den Schlüssel. |
| `DataRaw` | Bytes (hex), die sich nicht dekodieren lassen. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Werte mit falschem Typ, nicht verstandene angeheftete Elemente oder Listen, Links, die keine Links sind. |

## Untersuchungsstrategie { #investigation-strategy }

1. Sortieren Sie `AppSwitched` nach `Count`, um die Anwendungen zu finden, in denen ein Benutzer am meisten gearbeitet hat, und suchen Sie nach
   Administrations-, Fernzugriffs-, Archivierungs- und Übertragungswerkzeugen sowie nach Pfaden an für Benutzer beschreibbaren Orten
   (`\Users\`, `Downloads`, `Temp`).
2. Prüfen Sie die angehefteten Elemente: ein als `UserPinned` angeheftetes Werkzeug oder ein angeheftetes Element, dessen Verknüpfung an einem ungewöhnlichen Ort liegt.
3. Verwenden Sie die Zeiten von RecentApps zusammen mit UserAssist, Prefetch, BAM und Jump Lists, um die Verwendung zu datieren.
4. Vergleichen Sie `KeyCreationTime` mit der Erstellung des Profils: Ab dann beginnt die Zählung.
5. Verwenden Sie `parse-registry read --recover-deleted` für gelöschte Zähler und angeheftete Elemente.

## Weitere Informationen { #further-details }

- [Microsoft: Application User Model IDs](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: Programs cache](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
