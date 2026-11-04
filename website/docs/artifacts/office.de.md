# Microsoft Office (zuletzt verwendete Dateien, vertrauenswürdige Dokumente, Makroeinstellungen) { #microsoft-office-recent-files-trusted-documents-macro-settings }

## Was Office aufzeichnet { #what-office-records }

Microsoft Office speichert pro Benutzer in `NTUSER.DAT` sowie pro Version und Anwendung:

- **Zuletzt verwendete Dateien und Orte:** `File MRU`, die Dokumente, die eine Anwendung geöffnet hat, und `Place MRU`,
  die Ordner, aus denen sie geöffnet wurden, jeweils mit dem Zeitpunkt der letzten Öffnung. Microsoft 365
  führt zusätzlich eine Liste pro angemeldetem Konto (`User MRU\<account>`).
- **Vertrauenswürdige Dokumente:** Klickt ein Benutzer bei einem Dokument auf *Bearbeitung aktivieren* oder *Inhalt aktivieren*,
  merkt sich Office dies in `TrustRecords`: den Pfad des Dokuments, den Zeitpunkt, zu dem ihm vertraut wurde, und ob
  seine Makros aktiviert wurden.
- **Makroeinstellungen:** `VBAWarnings` (welche Makros ausgeführt werden), `BlockContentExecutionFromInternet`
  (ob Makros in Dateien aus dem Internet blockiert werden), `AccessVBOM` (ob Programme
  VBA-Projekte ändern dürfen) sowie die **vertrauenswürdigen Speicherorte**, Ordner, deren Dokumentmakros ohne
  Rückfrage ausgeführt werden. Gruppenrichtlinien können dasselbe unter `Software\Policies` festlegen.

Ein bösartiges Dokument erfordert in der Regel, dass der Benutzer seine Makros aktiviert: Ein Vertrauensdatensatz mit aktivierten Makros
verknüpft das Dokument, den Benutzer und einen Zeitpunkt miteinander.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Der Benutzer hat dieses Dokument in dieser
Anwendung geöffnet (zuletzt zu diesem Zeitpunkt); der Benutzer hat die Bearbeitung oder die Makros dieses Dokuments zu
diesem Zeitpunkt aktiviert; Office war so konfiguriert, dass Makros auf diese Weise ausgeführt werden.

## Speicherort { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

Versionen: `16.0` ist Office 2016 und neuer (2019, 2021, 2024, Microsoft 365), `15.0` Office 2013,
`14.0` Office 2010, `12.0` Office 2007.

Ein Eintrag sieht so aus: `[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx`; `T`
ist der Zeitpunkt der letzten Öffnung (ein FILETIME in hexadezimaler Form), der Pfad folgt auf `*`. Ein Vertrauensdatensatz ist 24
Byte groß: der FILETIME, zu dem dem Dokument vertraut wurde, gefolgt von seinen Flags in den letzten 4 Byte, `0x7FFFFFFF`,
wenn seine Makros aktiviert wurden, und `0x00000001`, wenn nur seine Bearbeitung aktiviert wurde.

## Hinweise für die Analyse { #analyst-cautions }

- **Zeitstempel.** `LastOpenedUTC` ist der eigene Zeitstempel jedes Eintrags. `TrustedAtUTC` gibt an, wann der Benutzer
  dem Dokument vertraut hat, also den Moment, in dem auf *Bearbeitung aktivieren* oder *Inhalt aktivieren* geklickt wurde; dieser kann vor
  der letzten Öffnung liegen.
- **Makros aktiviert, nicht ausgeführt.** `MacrosEnabled` besagt, dass der Benutzer die Makros des Dokuments aktiviert hat; es
  beweist nicht, dass ein Makro ausgeführt wurde oder was es getan hat. Korrelieren Sie mit Prozesserstellung, Prefetch, PowerShell-Protokollen
  und dem Dokument selbst.
- **Pfade so, wie Office sie speichert.** Vertrauensdatensätze benennen das Dokument über einen Pfad, der
  `%USERPROFILE%`, Schrägstriche, eine `file:///`-URL und URL-Kodierung (`%20`) enthalten kann. MRU-Einträge von
  Clouddokumenten sind URLs.
- **Standardwerte und wirksame Richtlinie.** Office legt standardmäßige vertrauenswürdige Speicherorte an (`Templates`,
  `XLSTART`, `STARTUP`, …). Das `Date` eines Speicherorts ist ein nützlicher Hinweis, aber kein Beweis dafür, wer ihn angelegt hat:
  Auch Administratoren, Skripte, Richtlinien und kopierte Registry-Einstellungen können ihn befüllen. Ein fehlendes
  `VBAWarnings` allein belegt nicht die wirksame Makrorichtlinie. Office-Version, die Kennzeichnung von Internetdateien
  und weitere Verwaltungseinstellungen spielen ebenfalls eine Rolle.
  [Microsoft erläutert die Blockierung von Internetmakros](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked).
- **Gelöschte Dokumente.** Die Listen überdauern die Dokumente; eine fehlende Datei kann hier dennoch angezeigt
  werden. Die Listen sind kurz und werden durch die Office-Funktion *Nicht angeheftete Elemente löschen* und durch Bereinigungswerkzeuge geleert.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

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
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

Die Zusammenfassung listet die Dokumente auf, deren Makros aktiviert wurden, Einstellungen, die die Ausführung von Makros
erleichtern (`VBAWarnings` 1, `BlockContentExecutionFromInternet` 0, `AccessVBOM` 1,
`AllowNetworkLocations` 1), von Benutzern hinzugefügte oder per Richtlinie gesetzte vertrauenswürdige Speicherorte sowie die
zuletzt geöffneten Dokumente.

## Ausgabefelder { #output-fields }

| Feld | Interpretation |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Host, der Profilinhaber (aus dem Pfad oder den Shell-Ordnern), die SID aus der `ProfileList` des Systems und der Hive. |
| `OfficeVersion`, `App` | Der Schlüssel der Version (`16.0`) und die Anwendung (`Word`, `Excel`, …). |
| `List` | `File MRU`, `Place MRU`, `TrustRecords`, `Security` oder `Trusted Locations`. |
| `Account` | Das Microsoft-365-Konto einer `User MRU`-Liste. |
| `Policy` | Ob die Einstellung oder der Speicherort per Richtlinie gesetzt ist. |
| `MRU-Position`, `IsMostRecent` | Die Position eines Eintrags in seiner Liste (`Item 1` ist 0, der neueste). |
| `Path` | Das Dokument oder der Ordner; der Pfad eines vertrauenswürdigen Dokuments wie gespeichert; der `Path` eines vertrauenswürdigen Speicherorts. |
| `LastOpenedUTC` | Wann der Eintrag zuletzt geöffnet wurde. |
| `TrustedAtUTC`, `MacrosEnabled`, `TrustFlags` | Wann dem Dokument vertraut wurde, ob seine Makros aktiviert wurden, und die Flags des Datensatzes. Unbekannte Flags lassen `MacrosEnabled` leer und erzeugen eine Diagnosemeldung. |
| `Setting`, `Value` | Eine Makroeinstellung und ihr Wert samt Bedeutung (`1 (all macros enabled)`); der Schlüssel eines vertrauenswürdigen Speicherorts (`Location3`) und seine weiteren Werte (`AllowSubFolders=1; Date=…; Description=…`). |
| `KeyPath`, `KeyLastWrittenUTC` | Der Schlüssel und sein letzter Schreibzugriff. |
| `DataRaw` | Die Bytes des Werts (hexadezimal), wenn sie sich nicht dekodieren lassen. |
| `TransactionLogStatus`, `LogsApplied` | Kontext der Registry-Wiederherstellung. |
| `Problems` | Einträge ohne `*` oder mit einem ungültigen Zeitstempel, Vertrauensdatensätze, die nicht 24 Byte groß sind, Speicherorte ohne `Path`, falsche Typen. |

## Untersuchungsstrategie { #investigation-strategy }

1. Beginnen Sie mit `MacrosEnabled` gleich true: die Dokumente, der Zeitpunkt und der Benutzer. Suchen Sie die Dateien (`$MFT`,
   USN-Journal, `Zone.Identifier`), ihre Herkunft (Browser- und E-Mail-Verlauf) und was danach ausgeführt wurde.
2. Prüfen Sie die Makroeinstellungen und vertrauenswürdigen Speicherorte: Alle Makros aktiviert, nicht blockierte Internetmakros,
   Zugriff auf VBA-Projekte oder ein von einem Benutzer hinzugefügter Speicherort (in `Downloads`, `Temp`, einer Netzwerkfreigabe)
   schwächen die Standardeinstellungen.
3. Ordnen Sie `LastOpenedUTC` zusammen mit LNK-Dateien, Jump Lists und RecentDocs für die Pfade der Dokumente in die Zeitleiste ein,
   einschließlich Wechselmedien und Netzwerkspeicherorten.

## Weitere Informationen { #further-details }

- [Microsoft: Aktivieren oder Deaktivieren von Makros in Microsoft 365-Dateien](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6)
- [Microsoft: Vertrauenswürdige Dokumente](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft: Hinzufügen, Entfernen oder Ändern eines vertrauenswürdigen Speicherorts](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
