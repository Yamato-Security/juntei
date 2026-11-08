# Outlook { #outlook }

## Was die Dateien von Outlook sind { #what-outlooks-files-are }

Outlook für Windows speichert E-Mails, Kontakte, Kalender und Aufgaben in eigenen Dateien:

- **OST** (`.ost`): die Offlinekopie eines Postfachs, die Outlook für Exchange- und Microsoft-365-Konten führt, seit
  Outlook 2013 auch für IMAP-Konten. Die OSTs von Outlook 2013 und neuer haben Seiten von 4 KiB und komprimierte
  Blöcke.
- **PST** (`.pst`): ein persönlicher Nachrichtenspeicher: POP3-Konten, Archive (`archive.pst`), Exporte und die
  Nachrichtenspeicher, die Benutzer selbst anlegen. ANSI-PSTs stammen von Outlook 97 bis 2002, Unicode-PSTs von
  Outlook 2003 und neuer.
- **MSG** (`.msg`): ein einzelnes, als Datei gespeichertes Element, wenn es aus Outlook herausgezogen, mit
  **Speichern unter** gespeichert oder von einem anderen Programm exportiert wird. Angehängte Nachrichten sind
  ebenfalls MSG-Dateien.
- **OFT** (`.oft`): eine Outlook-Vorlage: eine MSG-Datei, aus der neue Elemente erstellt werden.

Jeder Nachrichtenspeicher enthält Ordner und jeder Ordner seine Elemente: E-Mails, Besprechungsanfragen und
-antworten, Kontakte und Verteilerlisten, Kalenderelemente, Aufgaben, Notizen, Journaleinträge und Berichte. Jeder
Ordner enthält außerdem versteckte Elemente, seine ordnerzugeordneten Informationen (Folder Associated Information,
FAI): Ansichten, Regeln, Formulare und die Einstellungen von Outlook (`IPM.Configuration.*`-Elemente, etwa die
AutoVervollständigen-Liste eines Exchange-Postfachs). Die Anhänge eines Elements können Dateien, Verweise auf
Dateien, OLE-Objekte oder andere Elemente sein (eine als Anhang weitergeleitete Nachricht, die Ausnahmen eines
Serientermins), die ihrerseits eigene Anhänge haben.

<span class="evidence-strength">Bestmögliche Schlussfolgerung:</span> Der Nachrichtenspeicher oder die Datei enthielt
das Element mit diesen Absendern, Empfängern, Zeiten und Anhängen zum Zeitpunkt der Sicherung. Das besagt weder, dass
der Benutzer es gelesen, geschrieben oder gesendet hat, noch, dass sein Absender der ist, den das Element angibt.

## Speicherort und Sicherung { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

PSTs können überall liegen: Benutzer verschieben Archive in andere Ordner, auf andere Laufwerke und auf
Dateifreigaben. Outlook speichert die Pfade der Nachrichtenspeicher jedes Profils in der Registry des Benutzers.
Anhänge, die ein Benutzer aus Outlook heraus geöffnet hat, werden in einen Ordner unter
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\` kopiert (vor Windows 8.1
`Temporary Internet Files\Content.Outlook`), wo sie verbleiben können.

Outlook hält seine Nachrichtenspeicher geöffnet, solange es läuft, und eine OST kann Dutzende Gigabyte groß sein.
Sichern Sie sie bei geschlossenem Outlook oder aus einem Datenträgerabbild bzw. einer Volumeschattenkopie. Juntei
liest jeden Nachrichtenspeicher als Datenstrom: Es hält nie einen ganzen Nachrichtenspeicher im Arbeitsspeicher.

## Hinweise für die Analyse { #analyst-cautions }

- Juntei liest fehlertolerant: Es überspringt Beschädigtes, liest, was lesbar ist, und meldet den Rest als Probleme.
  Diese werden ausgegeben (die ersten 10 jeder Datei), in der Zusammenfassung und in der Dateitabelle gezählt und im
  Feld `Problems` der betroffenen Elemente und Anhänge aufgeführt. Ein Element, das sich überhaupt nicht lesen lässt,
  hat trotzdem eine Zeile in der Elementtabelle, mit seiner `ItemID` und dem Grund.
- Zeiten sind in UTC angegeben, auf 100 Nanosekunden genau. `SubmitTime` ist der Zeitpunkt, zu dem der Client des
  Absenders die Nachricht übermittelt hat, nach der Uhr dieses Computers; `DeliveryTime` der Zeitpunkt, zu dem sie
  dieses Postfach erreicht hat. `CreationTime` ist der Zeitpunkt, zu dem diese Kopie des Elements entstanden ist:
  zugestellt an das Postfach oder darin erstellt, mit der OST synchronisiert, in die PST kopiert oder importiert oder
  als MSG-Datei gespeichert. Es ist nicht der Zeitpunkt, zu dem die Nachricht verfasst wurde. `LastModifiedTime` ist
  der Zeitpunkt der letzten Änderung des Elements im Nachrichtenspeicher.
- `StartTime` und `DueTime` einer Aufgabe sind Datumsangaben in der Zeitzone des Benutzers, die Outlook als
  Mitternacht speichert. Juntei schreibt sie ohne Offset (`2017-02-19T00:00:00`), und `-Z` lässt sie unverändert.
- Für **Ohne** schreibt Outlook in manche Datumsfelder, etwa das Fälligkeitsdatum einer Aufgabe oder die Zeit einer
  Erinnerung, Mitternacht des 1. Januar 4501. Juntei gibt diesen Wert wie gespeichert aus: `4501-01-01T00:00:00Z`,
  ohne das `Z` in `StartTime` und `DueTime` einer Aufgabe.
- `Read` ist das Gelesen-Flag zum Zeitpunkt der Sicherung des Nachrichtenspeichers: Jemand oder eine Regel hat das
  Element als gelesen markiert. Es sagt weder, wer, noch wann.
- Namen und Adressen von Absendern und Empfängern sind das, was das sendende Programm und die Mailserver
  aufgezeichnet haben, und können gefälscht sein. `EX`-Adressen sind Exchange-Distinguished-Names
  (`/o=…/ou=…/cn=Recipients/cn=…`); die SMTP-Adresse steht in `SenderSMTP-Address` und `SMTP-Address`, wenn das
  Element sie zusätzlich speichert. Um zu prüfen, woher eine Nachricht stammt, lesen Sie ihre Internetkopfzeilen:
  `0x007D001F` oder `0x007D001E` (`PidTagTransportMessageHeaders`) in der Eigenschaftentabelle.
- In Outlook gelöschte Elemente liegen in **Gelöschte Elemente**, bis dieser Ordner geleert wird, und werden wie alle
  anderen gelesen. Elemente, die in der Tabelle keines Ordners mehr stehen und die ein Nachrichtenspeicher behalten
  kann, bis er ihren Platz wiederverwendet, werden noch nicht gelesen.
- `ItemID` ist die Knoten-ID der Nachricht in ihrer PST oder OST: Dasselbe Element kann in einer anderen Kopie des
  Nachrichtenspeichers eine andere haben. Bei einem eingebetteten Element besteht sie aus der ID des übergeordneten
  Elements und dem Index des Anhangs (`2097348/0`, `0/0/1`); das Element einer `.msg`- oder `.oft`-Datei hat `0`.
- `User` stammt nur aus dem Pfad (dem Ordner nach `Users`) und geht verloren, wenn eine Datei an einen anderen Ort
  kopiert wird. Eine PST kann jedem beliebigen Benutzer gehören.
- Text, der nicht in Unicode vorliegt, wird in der Codepage des Elements gelesen (seiner eigenen, der seines
  Gebietsschemas oder seiner Internet-Codepage). `--code-page` gibt die Codepage der Elemente an, die keine angeben:
  die ANSI-Codepage des Systems, das sie geschrieben hat, `932` für ein japanisches System.
- Das Kennwort einer PST verschlüsselt sie nicht: Juntei liest kennwortgeschützte PSTs wie alle anderen. Die
  `Encryption` eines Nachrichtenspeichers (`Permute`, die „komprimierbare Verschlüsselung“ von Outlook, oder
  `Cyclic`) verschleiert nur seine Blöcke. Mit S/MIME verschlüsselte Nachrichten speichern ihren Textkörper und ihre
  Anhänge in einem verschlüsselten Anhang (`smime.p7m`), der nicht entschlüsselt wird.
- Anhänge sind oft Malware. `--extract-attachments` schreibt ihre Daten in Dateien, die nach ihrem SHA-256 benannt
  sind, ohne Ausführungsberechtigung; führen Sie sie niemals aus und bewahren Sie sie auf einem isolierten
  Analysesystem auf.

### Noch nicht gelesen { #not-read-yet }

Offlineadressbücher (OAB), der AutoVervollständigen-Datenstrom (`Stream_Autocomplete_*.dat`) und `.nk2`-Dateien,
Senden/Empfangen-Einstellungen (`.srs`), die Einstellungscaches von Outlook (RoamCache, Autodiscover), die
SQLite-Datenbanken von Outlook sowie Nachrichten, die in der Tabelle keines Ordners mehr stehen.

## Mit Juntei parsen { #parse-with-juntei }

```text
Usage: juntei parse-file outlook <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pst, *.ost, *.msg and *.oft
  -f, --file <FILE>      Parse one PST, OST, MSG or OFT file, whatever its name
      --code-page <CP>   Code page of items that don't give theirs (default 1252)

Output:
  -o, --output <PATH>              Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>         csv, json, or jsonl
      --properties                 Add the Properties table: every property as stored
      --extract-attachments <DIR>  Write each attachment's data once, named by its SHA-256
  -C, --clobber                    Overwrite files when saving or extracting
```

```bash
juntei parse-file outlook -d Users -o outlook-csv
juntei parse-file outlook -f mailbox.ost -t jsonl \
  | jq 'select(.Table == "Items" and .Hidden == false) | {DeliveryTime, SenderAddress, Subject}'
juntei parse-file outlook -f suspicious.msg -o msg-csv --properties --extract-attachments isolated/
```

`-d` sucht Dateien, die auf `.pst`, `.ost`, `.msg` und `.oft` enden, ohne Beachtung der Groß-/Kleinschreibung. Wie
auch immer eine Datei angegeben wird, ihre ersten Bytes bestimmen, was sie ist: `!BDN` für eine PST oder OST, die
Compound-File-Signatur für eine MSG- oder OFT-Datei. Alles andere wird als Datei gemeldet, die nicht geparst werden
konnte. Der Exit-Status ist 1, wenn keine Datei gelesen werden konnte.

`--extract-attachments` schreibt die Daten jedes Anhangs genau einmal, als `<SHA-256>` mit der Erweiterung des
Anhangs, wenn diese aus 1 bis 10 ASCII-Buchstaben und -Ziffern besteht (`0c7d…1311.jpg`); kein Name aus einem Element
wird in einem Pfad verwendet. Das Verzeichnis muss leer sein, sofern nicht `-C` angegeben ist; mit `-C` wird eine
gleichnamige Datei mit anderem Inhalt ersetzt.

## Ausgabetabellen { #output-tables }

Die CSV-Ausgabe erzeugt acht Dateien, mit `--properties` neun; hat eine davon keine Zeilen, enthält sie nur die
Kopfzeile. JSON- und JSONL-Zeilen enthalten `Table` (`Files`, `Folders`, `Items`, `Recipients`, `Attachments`,
`Contacts`, `Calendar`, `Tasks` oder `Properties`) nach `SourceFile`. Auf ein Element folgen seine Zeilen und auf
diese die seiner eingebetteten Elemente; die Zeile einer Datei in der Dateitabelle folgt auf alle übrigen Zeilen
dieser Datei.

| CSV-Datei | Inhalt |
|---|---|
| `Outlook-Files.csv` | Eine Zeile pro gelesener Datei: der Nachrichtenspeicher oder die Datei und was sie enthält. |
| `Outlook-Folders.csv` | Die Ordner der PSTs und OSTs. |
| `Outlook-Items.csv` | Jedes Element: die Nachrichten der Ordner, ihre versteckten Elemente, die Elemente von `.msg`- und `.oft`-Dateien und die in deren Anhängen eingebetteten Elemente. |
| `Outlook-Recipients.csv` | Die Empfänger der Elemente. |
| `Outlook-Attachments.csv` | Die Anhänge der Elemente. |
| `Outlook-Contacts.csv` | Die Felder von Kontakten und Verteilerlisten. |
| `Outlook-Calendar.csv` | Die Felder von Kalenderelementen und Besprechungsnachrichten. |
| `Outlook-Tasks.csv` | Die Felder von Aufgaben und Aufgabenanfragen. |
| `Outlook-Properties.csv` | Mit `--properties`: jede Eigenschaft der Nachrichtenspeicher, Ordner, Elemente, Empfänger und Anhänge, wie gespeichert. |

## Felder { #fields }

Jede Tabelle beginnt mit diesen Feldern:

| Feld | Interpretation |
|---|---|
| `SourceFile` | Die Datei, aus der die Zeile gelesen wurde. |
| `User` | Der Windows-Benutzer, in dessen Profil der Pfad liegt (der Ordner nach `Users`). |

### Dateien { #files }

| Feld | Interpretation |
|---|---|
| `FileType` | `PST`, `OST`, `MSG`, `OFT` oder `CFB` für eine als Element gelesene Compound File, deren Klassen-ID weder die einer Nachricht noch die einer Vorlage ist (viele Programme außer Outlook lassen sie leer). |
| `Format` | Das Layout eines Nachrichtenspeichers: `ANSI` (Outlook 97 bis 2002), `Unicode` (Outlook 2003 und neuer) oder `Unicode 4K` (die OSTs von Outlook 2013 und neuer); bei einer MSG- oder OFT-Datei die Version der Compound File (`CFB v3`, `CFB v4`). |
| `Version`, `ClientVersion` | `wVer` und `wVerClient` aus dem Header eines Nachrichtenspeichers. |
| `Encryption` | Wie die Blöcke eines Nachrichtenspeichers kodiert sind: `None`, `Permute` oder `Cyclic`. |
| `DisplayName` | Der Name des Nachrichtenspeichers (`Personal Folders`, eine E-Mail-Adresse). |
| `RecordKey` | Die Identität des Nachrichtenspeichers (hex), die in den Eintrags-IDs seiner Elemente enthalten ist. |
| `Folders` | Die Anzahl der Ordner des Nachrichtenspeichers. |
| `Items` | Die Anzahl der Nachrichten seiner Ordner, bzw. 1 bei einer MSG- oder OFT-Datei. |
| `HiddenItems` | Die Anzahl der versteckten Elemente (ordnerzugeordnete Informationen) seiner Ordner. |
| `EmbeddedItems` | Die Anzahl der in Anhängen eingebetteten Elemente, auf allen Ebenen. |
| `Attachments` | Die Anzahl der Anhänge all dieser Elemente. |
| `Problems` | Die Anzahl der in der Datei gefundenen Probleme. |

### Ordner { #folders }

| Feld | Interpretation |
|---|---|
| `FolderID`, `ParentID` | Die Knoten-ID des Ordners und die seines übergeordneten Ordners (leer beim Stammordner). |
| `FolderPath` | Die Namen der Ordner vom Stamm abwärts: `/Top of Personal Folders/Inbox`, `/Root - Mailbox/IPM_SUBTREE/Sent Items`. |
| `Name` | Der Name des Ordners. |
| `ContainerClass` | Was der Ordner enthält: `IPF.Note` (E-Mail), `IPF.Appointment`, `IPF.Contact`, `IPF.Task`, `IPF.Imap`, …. |
| `ContentCount`, `UnreadCount` | Die Anzahl der Nachrichten bzw. der ungelesenen Nachrichten, die der Ordner nach eigener Angabe enthält. |
| `Messages`, `HiddenItems`, `Subfolders` | Die Zeilen seiner Inhaltstabelle, seiner Tabelle der zugeordneten Inhalte und seiner Hierarchietabelle: das, was Juntei liest. Eine Abweichung von `ContentCount` verdient einen genaueren Blick. |
| `CreationTime`, `LastModifiedTime` | Wann der Ordner angelegt und zuletzt geändert wurde (UTC), sofern der Nachrichtenspeicher dies speichert. |
| `Problem` | Warum der Ordner nicht geöffnet werden konnte; seine Unterordner werden dann nicht gelesen. |

### Elemente { #items }

| Feld | Interpretation |
|---|---|
| `FolderPath` | Der Ordner des Elements; leer in einer MSG- oder OFT-Datei. |
| `ItemID` | Die ID des Elements: seine Knoten-ID in einer PST oder OST, `0` in einer MSG- oder OFT-Datei, bei einem eingebetteten Element die ID des übergeordneten Elements und der Index des Anhangs (`2097348/0`). |
| `Hidden` | Ob das Element zu den ordnerzugeordneten Informationen gehört: ein verstecktes Element. |
| `EmbeddedIn` | Die ID des Elements, an das es angehängt ist; bei allen anderen leer. |
| `ItemType` | Was das Element laut seiner Nachrichtenklasse ist: `Mail`, `MeetingMessage`, `Appointment`, `Contact`, `DistributionList`, `Task`, `TaskRequest`, `StickyNote`, `Journal`, `Post`, `Report`, `Configuration` oder `Other`. |
| `MessageClass` | Die Nachrichtenklasse, wie gespeichert (`IPM.Note`, `IPM.Note.SMIME`, `IPM.Schedule.Meeting.Request`, `REPORT.IPM.Note.NDR`, …). |
| `SubmitTime` | Wann der Client des Absenders die Nachricht übermittelt hat (UTC, nach der Uhr des Absenders). |
| `DeliveryTime` | Wann sie an dieses Postfach zugestellt wurde (UTC). |
| `CreationTime`, `LastModifiedTime` | Wann diese Kopie im Nachrichtenspeicher oder in der Datei entstanden ist und wann sie zuletzt geändert wurde (UTC). |
| `Subject` | Der Betreff, mit seinem Präfix (`RE: `). |
| `SenderName`, `SenderAddressType`, `SenderAddress` | Wer sie gesendet hat: der Name, der Adresstyp (`SMTP`, `EX`) und die Adresse dieses Typs. |
| `SenderSMTP-Address` | Die SMTP-Adresse des Absenders, wenn sie zusätzlich zu einer Adresse eines anderen Typs gespeichert ist. |
| `SentRepresentingName`, `SentRepresentingAddress` | Für wen sie gesendet wurde (ein Stellvertreter sendet im Namen eines Vorgesetzten; ein freigegebenes Postfach), mit dessen SMTP-Adresse, falls gespeichert, sonst der Adresse seines Typs. |
| `To`, `Cc`, `Bcc` | Die Namen der Empfänger, wie das Element sie anzeigt (ihre Adressen stehen in der Empfängertabelle). `Bcc` steht nur in der Kopie des Absenders. |
| `InternetMessageID`, `InReplyTo` | Die `Message-ID` der Nachricht und die der Nachricht, auf die sie antwortet. |
| `ConversationTopic` | Der Betreff ohne Präfix, den die Nachrichten einer Unterhaltung gemeinsam haben. |
| `Importance`, `Sensitivity` | `Low`, `Normal` oder `High`; `Normal`, `Personal`, `Private` oder `Confidential`. |
| `Read` | Ob das Element als gelesen markiert ist. |
| `Flags` | Die Nachrichtenflags (`PidTagMessageFlags`, hex): `0x1` gelesen, `0x2` unverändert, `0x4` übermittelt, `0x8` nicht gesendet (ein Entwurf), `0x10` hat Anhänge, `0x20` von mir, `0x40` versteckt, `0x80` erneut gesendet, `0x400` jemals gelesen, `0x2000` aus dem Internet. |
| `Size` | Die Größe des Elements im Nachrichtenspeicher, einschließlich seiner Anhänge. |
| `Recipients`, `Attachments` | Wie viele Empfänger und Anhänge es hat. |
| `Categories` | Seine Kategorien, durch `;` getrennt. |
| `BodyPreview` | Der Anfang seines Textkörpers im Nur-Text-Format, mit zusammengefassten Leerräumen, höchstens 256 Zeichen; leer, wenn es nur einen HTML- oder RTF-Textkörper hat. |
| `Problems` | Was im Element nicht gelesen werden konnte (höchstens 10, dann wie viele weitere). |

### Empfänger { #recipients }

| Feld | Interpretation |
|---|---|
| `ItemID` | Die ID des Elements. |
| `Index` | Die Zeile des Empfängers in der Empfängertabelle des Elements, ab 0. |
| `RecipientType` | `To`, `Cc`, `Bcc`, `Originator` oder der gespeicherte Wert (hex). |
| `Name`, `AddressType`, `Address` | Name, Adresstyp und Adresse des Empfängers. |
| `SMTP-Address` | Die SMTP-Adresse, wenn sie zusätzlich zu einer Adresse eines anderen Typs gespeichert ist. |

### Anhänge { #attachments }

| Feld | Interpretation |
|---|---|
| `ItemID`, `Index` | Die ID des Elements und die Zeile des Anhangs in dessen Anhangtabelle, ab 0. |
| `Method` | `ByValue` (eine Datei), `ByReference`, `ByReferenceResolve`, `ByReferenceOnly` (ein Pfad zu einer Datei), `EmbeddedMessage` (ein Element), `OLE` (ein OLE-Objekt), `ByWebReference` oder die Zahl. |
| `Filename` | Der Name des Anhangs: sein langer Dateiname, sein kurzer oder sein Anzeigename. |
| `Extension` | Seine Erweiterung, wie gespeichert (`.pdf`), sonst die seines Namens. |
| `MimeType`, `ContentID` | Sein MIME-Typ und die Content-ID, über die ein HTML-Textkörper auf ihn verweist. |
| `Inline` | Ob der Textkörper ihn anzeigt (ein Bild des HTML-Textkörpers), statt ihn aufzulisten. |
| `Hidden` | Ob Outlook ihn in der Anhangliste ausblendet. |
| `Size` | Seine Größe laut Nachrichtenspeicher, einschließlich seiner Eigenschaften. |
| `DataSize`, `SHA256` | Größe und SHA-256 seiner Daten: der Datei oder, in einer PST oder OST, der Compound File eines OLE-Objekts. Leer bei einem eingebetteten Element und bei einem Anhang per Verweis. |
| `Pathname` | Der Pfad eines Anhangs per Verweis. |
| `CreationTime`, `LastModifiedTime` | Die Zeiten des Anhangs (UTC): oft die der Datei, aus der er erstellt wurde. |
| `EmbeddedSubject` | Der Betreff eines eingebetteten Elements, dessen eigene Zeilen die `ItemID` `<ItemID>/<Index>` haben. |
| `ExtractedFile` | Mit `--extract-attachments` die Datei in diesem Verzeichnis, in die seine Daten geschrieben wurden. |
| `Problems` | Was im Anhang nicht gelesen oder nicht geschrieben werden konnte. |

### Kontakte { #contacts }

| Feld | Interpretation |
|---|---|
| `FolderPath`, `ItemID`, `ItemType` | Wie in der Elementtabelle: `Contact` oder `DistributionList`. |
| `DisplayName`, `GivenName`, `Surname`, `Nickname` | Die Namen des Kontakts. |
| `FileUnder` | Der Name, unter dem der Kontakt abgelegt ist. |
| `Company`, `Title`, `Department` | Wo der Kontakt arbeitet. |
| `Email1Name`, `Email1Address`, `Email2Name`, `Email2Address`, `Email3Name`, `Email3Address` | Die drei E-Mail-Adressen des Kontakts, jede mit ihrem Anzeigenamen. |
| `BusinessPhone`, `HomePhone`, `MobilePhone` | Die Telefonnummern des Kontakts. |
| `InstantMessaging` | Die Chatadresse (Instant Messaging) des Kontakts. |
| `CreationTime`, `LastModifiedTime` | Wie in der Elementtabelle. |

### Kalender { #calendar }

| Feld | Interpretation |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Wie in der Elementtabelle: `Appointment` oder `MeetingMessage`. |
| `StartTime`, `EndTime` | Wann der Termin beginnt und endet (UTC); bei einem Serientermin die des ersten Vorkommens. |
| `DurationMinutes` | Seine Dauer in Minuten. |
| `Location` | Wo er stattfindet. |
| `AllDay`, `Recurring` | Ob es ein ganztägiges Ereignis ist und ob er sich wiederholt (seine Ausnahmen sind eingebettete Elemente). |
| `BusyStatus` | `Free`, `Tentative`, `Busy`, `OutOfOffice` oder `WorkingElsewhere`. |
| `ResponseStatus` | Die Antwort des Benutzers: `None`, `Organized`, `Tentative`, `Accepted`, `Declined` oder `NotResponded`. |
| `Organizer` | Der Name des Absenders des Elements. |
| `AllAttendees`, `ToAttendees`, `CcAttendees` | Die Teilnehmer, wie das Element sie aufführt. |
| `GlobalObjectID` | Die postfachübergreifende Identität der Besprechung (hex): dieselbe Besprechung in den Kalendern des Organisators und der Teilnehmer. |
| `ReminderSet`, `ReminderTime` | Ob eine Erinnerung gesetzt ist, und der Zeitpunkt, für den sie gilt (UTC). |
| `CreationTime`, `LastModifiedTime` | Wie in der Elementtabelle. |

### Aufgaben { #tasks }

| Feld | Interpretation |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Wie in der Elementtabelle: `Task` oder `TaskRequest`. |
| `Status` | `NotStarted`, `InProgress`, `Complete`, `Waiting` oder `Deferred`. |
| `PercentComplete` | Wie viel davon erledigt ist, von 0 bis 100. |
| `StartTime`, `DueTime` | Wann sie beginnt und wann sie fällig ist: Datumsangaben in der Zeitzone des Benutzers, ohne Offset. |
| `CompletedTime` | Wann sie erledigt wurde (UTC). |
| `Complete` | Ob sie erledigt ist. |
| `Owner`, `Assigner` | Wem sie gehört und wer sie zugewiesen hat. |
| `ReminderSet`, `ReminderTime` | Wie in der Kalendertabelle. |
| `CreationTime`, `LastModifiedTime` | Wie in der Elementtabelle. |

### Eigenschaften { #properties }

| Feld | Interpretation |
|---|---|
| `ItemID` | Die ID des Elements; `store` für die eigenen Eigenschaften des Nachrichtenspeichers; `folder:<FolderID>` für die eines Ordners. |
| `Scope` | Wessen Eigenschaft es ist: `Store`, `Folder`, `Item`, `Recipient` oder `Attachment`. |
| `Index` | Der Index des Empfängers oder Anhangs; bei allen anderen leer. |
| `PropertyTag` | ID und Typ der Eigenschaft, in der Form `0x0037001F` (`PidTagSubject`, eine Zeichenfolge). |
| `PropertyName` | Bei einer benannten Eigenschaft (IDs ab `0x8000`) ihr Eigenschaftensatz und ihr Name: `PSETID_Address:0x8083`, `PS_PUBLIC_STRINGS:Keywords` oder die GUID des Eigenschaftensatzes, wenn es kein bekannter ist. |
| `Type` | Der Eigenschaftstyp: `PtypString`, `PtypInteger32`, `PtypTime`, `PtypBinary`, `PtypMultipleString`, …. |
| `Value` | Der Wert: Text bei Zeichenfolgen, ISO 8601 in UTC bei Zeiten (die `-Z` nicht konvertiert), hex bei Binärwerten (die ersten 4.096 Bytes, dann `…(N bytes)`), die Werte einer mehrwertigen Eigenschaft als Liste (in CSV durch `, ` getrennt), `(object)` bei einem eingebetteten Element oder den Daten eines Anhangs, und die Bytes in hex mit einem Hinweis bei einem beschädigten Wert oder einem Typ, der nicht dekodiert wird. |

## Automatisches Parsen { #automatic-parsing }

`auto-parse` schreibt die Tabellen in die neue Kategorie `Communication` als `Outlook-Files.csv`,
`Outlook-Folders.csv`, `Outlook-Items.csv`, `Outlook-Recipients.csv`, `Outlook-Attachments.csv`,
`Outlook-Contacts.csv`, `Outlook-Calendar.csv` und `Outlook-Tasks.csv` (eine einzige Datei `Outlook.json` bzw.
`Outlook.jsonl` mit `-t json` bzw. `-t jsonl`), zusammen mit `Communication/Outlook-logs.txt`. Es übergibt dem Parser
seine `--code-page`, fügt aber weder die Eigenschaftentabelle hinzu noch extrahiert es Anhänge: Verwenden Sie dafür
`parse-file outlook`. Die kombinierte Zeitleiste `-j` enthält jede Zeile, mit `Comp` und dem `Artifact`-Bezeichner
`outlook`. `-Z` konvertiert jedes Feld, das auf `Time` endet, mit Ausnahme von `StartTime` und `DueTime` einer
Aufgabe.

## Weitere Informationen { #further-details }

- [Microsoft — \[MS-PST\]: Outlook Personal Folders (.pst) File Format](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]: Outlook Item (.msg) File Format](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]: Message and Attachment Object Protocol](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]: Exchange Server Protocols Master Property List](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — Dateiformat Personal Folder File (PFF)](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
