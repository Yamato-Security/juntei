# Outlook

## What Outlook's files are

Outlook for Windows keeps mail, contacts, calendars and tasks in files of its own:

- **OST** (`.ost`): the offline copy of a mailbox, which Outlook keeps for Exchange and Microsoft
  365 accounts, and since Outlook 2013 for IMAP accounts too. Those of Outlook 2013 and later have
  4 KiB pages and compressed blocks.
- **PST** (`.pst`): a personal store: POP3 accounts, archives (`archive.pst`), exports and the
  stores users create. ANSI PSTs are those of Outlook 97 to 2002; Unicode PSTs those of Outlook
  2003 and later.
- **MSG** (`.msg`): one item saved as a file, when it is dragged out of Outlook, saved with
  **Save As**, or exported by another program. Attached messages are MSG files too.
- **OFT** (`.oft`): an Outlook template: an MSG file that new items are made from.

Each store holds folders, and each folder its items: e-mail, meeting requests and responses,
contacts and distribution lists, calendar items, tasks, notes, journal entries and reports. Each
folder also keeps hidden items, its folder associated information (FAI): views, rules, forms and
Outlook's settings (`IPM.Configuration.*` items, such as the autocomplete list of an Exchange
mailbox). An item's attachments can be files, references to files, OLE objects, or other items
(a message forwarded as an attachment, the exceptions of a recurring appointment), which hold
attachments of their own.

<span class="evidence-strength">Best conclusion:</span> the store or file held the item, with
these senders, recipients, times and attachments, when it was collected. It does not say that the
user read, wrote or sent it, nor that its sender is who the item says.

## Location and collection

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

PSTs can be anywhere: users move archives to other folders, drives and file shares. Outlook keeps
the paths of the stores of each profile in the user's registry. Attachments a user opened from
Outlook are copied to a folder of `%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\`
(`Temporary Internet Files\Content.Outlook` before Windows 8.1), where they can remain.

Outlook keeps its stores open while it runs, and an OST can be tens of gigabytes. Collect them with
Outlook closed, or from a disk image or a volume shadow copy. Juntei reads each store as a stream:
it never holds a whole store in memory.

## Analyst cautions

- Juntei reads leniently: it goes past what is damaged, reads what can be read, and reports the
  rest as problems. They are printed (the first 10 of each file), counted in the summary and the
  Files table, and listed in the `Problems` field of the items and attachments they concern. An
  item that cannot be read at all still has a row in the Items table, with its `ItemID` and why.
- Times are in UTC, to the 100 nanoseconds. `SubmitTime` is when the sender's client submitted the
  message, by that computer's clock; `DeliveryTime` when it reached this mailbox. `CreationTime` is
  when this copy of the item was made: delivered to or created in the mailbox, synchronized to the
  OST, copied or imported into the PST, or saved as the MSG file. It is not when the message was
  written. `LastModifiedTime` is when the item was last changed in the store.
- A task's `StartTime` and `DueTime` are dates in the user's time zone, which Outlook stores as
  midnight. Juntei writes them without an offset (`2017-02-19T00:00:00`), and `-Z` leaves them as
  they are.
- Outlook writes midnight of January 1, 4501 for **None** in some dates, such as a task's due date
  or a reminder's time. Juntei writes it as stored: `4501-01-01T00:00:00Z`, without the `Z` in a
  task's `StartTime` and `DueTime`.
- `Read` is the read flag when the store was collected: someone, or a rule, marked the item read.
  It says neither who nor when.
- A sender's and a recipient's names and addresses are what the sending program and the mail
  servers recorded, and can be forged. `EX` addresses are Exchange distinguished names
  (`/o=…/ou=…/cn=Recipients/cn=…`); the SMTP address is in `SenderSMTP-Address` and
  `SMTP-Address` when the item stores it besides. To check where a message came from, read its
  Internet headers: the Properties table's `0x007D001F` or `0x007D001E`
  (`PidTagTransportMessageHeaders`).
- Items deleted in Outlook are in **Deleted Items** until it is emptied, and are read like any
  other. Items no longer in any folder's table, which a store can keep until it reuses their
  space, are not read yet.
- `ItemID` is the message's node id in its PST or OST: the same item in another copy of the store
  can have another. An embedded item's is its parent's and the attachment's index (`2097348/0`,
  `0/0/1`); a `.msg` or `.oft` file's item is `0`.
- `User` comes from the path only (the folder after `Users`), and is lost when a file is copied
  elsewhere. A PST can be any user's.
- Text that isn't Unicode is read in the item's code page (its own, its locale's or its Internet
  code page's). `--code-page` gives the code page of the items that give none: the ANSI code page
  of the system that wrote them, `932` for a Japanese system.
- A PST's password does not encrypt it: Juntei reads password-protected PSTs as any other. A
  store's `Encryption` (`Permute`, Outlook's "compressible encryption", or `Cyclic`) only obscures
  its blocks. Messages encrypted with S/MIME keep their body and attachments in an encrypted
  attachment (`smime.p7m`), which is not decrypted.
- Attachments are often malware. `--extract-attachments` writes their data to files named by their
  SHA-256 (new files have mode `0600` on Unix); never run them, and keep them on an isolated analysis
  system.

### Not read yet

Offline address books (OAB), the autocomplete stream (`Stream_Autocomplete_*.dat`) and `.nk2`
files, send/receive settings (`.srs`), Outlook's settings caches (RoamCache, Autodiscover),
Outlook's SQLite databases, and messages that are no longer in any folder's table.

## Parse with Juntei

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

`-d` searches for `.pst`, `.ost`, `.msg` and `.oft` names, case ignored, then checks their headers:
`!BDN` for a PST or OST, or the compound file signature for an MSG or OFT. Unrelated files that
share these extensions, such as Tcl message catalogs, are skipped. Unreadable files and damaged
containers with a recognizable signature remain errors. `-f` always validates the supplied file,
regardless of its name, and reports invalid input as an error. The exit status is 1 when no file
could be read.

`--extract-attachments` writes each attachment's data once, as `<SHA-256>` with the attachment's
extension when it is 1 to 10 ASCII letters and digits (`0c7d…1311.jpg`); no name from an item is
used in a path. The directory must be empty unless `-C` is given; with `-C`, a file of the same
name that holds something else is replaced.

## Output tables

CSV creates eight files, nine with `--properties`, each with only its header when it has no rows.
JSON and JSONL rows carry `Table` (`Files`, `Folders`, `Items`, `Recipients`, `Attachments`,
`Contacts`, `Calendar`, `Tasks` or `Properties`) after `SourceFile`. The rows of an item follow
it, and those of its embedded items follow them; a file's Files row comes after the rest of its
rows.

| CSV file | Contents |
|---|---|
| `Outlook-Files.csv` | One row per file read: the store or file and what it holds. |
| `Outlook-Folders.csv` | The folders of the PSTs and OSTs. |
| `Outlook-Items.csv` | Every item: the messages of the folders, their hidden items, the items of `.msg` and `.oft` files, and the items embedded in their attachments. |
| `Outlook-Recipients.csv` | The items' recipients. |
| `Outlook-Attachments.csv` | The items' attachments. |
| `Outlook-Contacts.csv` | The fields of contacts and distribution lists. |
| `Outlook-Calendar.csv` | The fields of calendar items and meeting messages. |
| `Outlook-Tasks.csv` | The fields of tasks and task requests. |
| `Outlook-Properties.csv` | With `--properties`: every property of the stores, folders, items, recipients and attachments, as stored. |

## Fields

Every table starts with these fields:

| Field | Interpretation |
|---|---|
| `SourceFile` | The file the row was read from. |
| `User` | The Windows user whose profile the path is in (the folder after `Users`). |

### Files

| Field | Interpretation |
|---|---|
| `FileType` | `PST`, `OST`, `MSG`, `OFT`, or `CFB` for a compound file read as an item whose class ID is neither a message's nor a template's (many programs other than Outlook leave it empty). |
| `Format` | A store's layout: `ANSI` (Outlook 97 to 2002), `Unicode` (Outlook 2003 and later) or `Unicode 4K` (the OSTs of Outlook 2013 and later); an MSG or OFT file's compound file version (`CFB v3`, `CFB v4`). |
| `Version`, `ClientVersion` | A store's header's `wVer` and `wVerClient`. |
| `Encryption` | How a store's blocks are encoded: `None`, `Permute` or `Cyclic`. |
| `DisplayName` | The store's name (`Personal Folders`, an e-mail address). |
| `RecordKey` | The store's identity (hex), which the entry ids of its items hold. |
| `Folders` | The store's folders. |
| `Items` | The messages of its folders, or 1 for an MSG or OFT file. |
| `HiddenItems` | The hidden items (folder associated information) of its folders. |
| `EmbeddedItems` | The items embedded in attachments, at every level. |
| `Attachments` | The attachments of all those items. |
| `Problems` | The problems found in the file. |

### Folders

| Field | Interpretation |
|---|---|
| `FolderID`, `ParentID` | The folder's node id, and its parent folder's (empty for the root folder). |
| `FolderPath` | The names of the folders from the root down: `/Top of Personal Folders/Inbox`, `/Root - Mailbox/IPM_SUBTREE/Sent Items`. |
| `Name` | The folder's name. |
| `ContainerClass` | What the folder holds: `IPF.Note` (mail), `IPF.Appointment`, `IPF.Contact`, `IPF.Task`, `IPF.Imap`, …. |
| `ContentCount`, `UnreadCount` | The messages, and unread messages, the folder says it has. |
| `Messages`, `HiddenItems`, `Subfolders` | The rows of its contents table, its associated contents table and its hierarchy table: what Juntei reads. A difference from `ContentCount` is worth a look. |
| `CreationTime`, `LastModifiedTime` | When the folder was made and last changed (UTC), when the store keeps them. |
| `Problem` | Why the folder could not be opened; its subfolders are then not read. |

### Items

| Field | Interpretation |
|---|---|
| `FolderPath` | The item's folder; empty in an MSG or OFT file. |
| `ItemID` | The item's id: its node id in a PST or OST, `0` in an MSG or OFT file, its parent's id and the attachment's index for an embedded item (`2097348/0`). |
| `Hidden` | Whether the item is folder associated information: a hidden item. |
| `EmbeddedIn` | The id of the item it is attached to; empty for the others. |
| `ItemType` | What the message class says the item is: `Mail`, `MeetingMessage`, `Appointment`, `Contact`, `DistributionList`, `Task`, `TaskRequest`, `StickyNote`, `Journal`, `Post`, `Report`, `Configuration` or `Other`. |
| `MessageClass` | The message class as stored (`IPM.Note`, `IPM.Note.SMIME`, `IPM.Schedule.Meeting.Request`, `REPORT.IPM.Note.NDR`, …). |
| `SubmitTime` | When the sender's client submitted the message (UTC, the sender's clock). |
| `DeliveryTime` | When it was delivered to this mailbox (UTC). |
| `CreationTime`, `LastModifiedTime` | When this copy was made in the store or file, and last changed (UTC). |
| `Subject` | The subject, with its prefix (`RE: `). |
| `SenderName`, `SenderAddressType`, `SenderAddress` | Who sent it: the name, the address type (`SMTP`, `EX`) and the address of that type. |
| `SenderSMTP-Address` | The sender's SMTP address, when it is stored besides an address of another type. |
| `SentRepresentingName`, `SentRepresentingAddress` | Whom it was sent for (a delegate sends for a manager; a shared mailbox), with their SMTP address when stored, else the address of its type. |
| `To`, `Cc`, `Bcc` | The recipients' names as the item displays them (see Recipients for their addresses). `Bcc` is only in the sender's copy. |
| `InternetMessageID`, `InReplyTo` | The message's `Message-ID`, and that of the message it replies to. |
| `ConversationTopic` | The subject without its prefix, which the messages of a conversation share. |
| `Importance`, `Sensitivity` | `Low`, `Normal` or `High`; `Normal`, `Personal`, `Private` or `Confidential`. |
| `Read` | Whether the item is marked read. |
| `Flags` | The message flags (`PidTagMessageFlags`, hex): `0x1` read, `0x2` unmodified, `0x4` submitted, `0x8` unsent (a draft), `0x10` has attachments, `0x20` from me, `0x40` hidden, `0x80` resent, `0x400` ever read, `0x2000` from the Internet. |
| `Size` | The item's size in the store, with its attachments. |
| `Recipients`, `Attachments` | How many recipients and attachments it has. |
| `Categories` | Its categories, a `;` between them. |
| `BodyPreview` | The beginning of its plain-text body, its white space collapsed, at most 256 characters; empty when it has only an HTML or RTF body. |
| `Problems` | What could not be read in the item (at most 10, then how many more). |

### Recipients

| Field | Interpretation |
|---|---|
| `ItemID` | The item's id. |
| `Index` | The recipient's row in the item's recipient table, from 0. |
| `RecipientType` | `To`, `Cc`, `Bcc`, `Originator`, or the value as stored (hex). |
| `Name`, `AddressType`, `Address` | The recipient's name, address type and address. |
| `SMTP-Address` | The SMTP address, when it is stored besides an address of another type. |

### Attachments

| Field | Interpretation |
|---|---|
| `ItemID`, `Index` | The item's id, and the attachment's row in its attachment table, from 0. |
| `Method` | `ByValue` (a file), `ByReference`, `ByReferenceResolve`, `ByReferenceOnly` (a path to a file), `EmbeddedMessage` (an item), `OLE` (an OLE object), `ByWebReference`, or the number. |
| `Filename` | The attachment's name: its long filename, its short one, or its display name. |
| `Extension` | Its extension as stored (`.pdf`), else its name's. |
| `MimeType`, `ContentID` | Its MIME type, and the content id an HTML body refers to it by. |
| `Inline` | Whether the body shows it (an image of the HTML body) rather than listing it. |
| `Hidden` | Whether Outlook hides it from the attachment list. |
| `Size` | Its size as the store gives it, with its properties. |
| `DataSize`, `SHA256` | The size and SHA-256 of its data: the file, or an OLE object's compound file in a PST or OST. Empty for an embedded item and an attachment by reference. |
| `Pathname` | The path of an attachment by reference. |
| `CreationTime`, `LastModifiedTime` | The attachment's times (UTC): often those of the file it was made from. |
| `EmbeddedSubject` | The subject of an embedded item, whose own rows have the `ItemID` `<ItemID>/<Index>`. |
| `ExtractedFile` | With `--extract-attachments`, the file its data was written to, in that directory. |
| `Problems` | What could not be read in the attachment, or written. |

### Contacts

| Field | Interpretation |
|---|---|
| `FolderPath`, `ItemID`, `ItemType` | As in the Items table: `Contact` or `DistributionList`. |
| `DisplayName`, `GivenName`, `Surname`, `Nickname` | The contact's names. |
| `FileUnder` | The name the contact is filed under. |
| `Company`, `Title`, `Department` | Where the contact works. |
| `Email1Name`, `Email1Address`, `Email2Name`, `Email2Address`, `Email3Name`, `Email3Address` | The contact's three e-mail addresses, each with its display name. |
| `BusinessPhone`, `HomePhone`, `MobilePhone` | The contact's phone numbers. |
| `InstantMessaging` | The contact's instant messaging address. |
| `CreationTime`, `LastModifiedTime` | As in the Items table. |

### Calendar

| Field | Interpretation |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | As in the Items table: `Appointment` or `MeetingMessage`. |
| `StartTime`, `EndTime` | When the appointment starts and ends (UTC); the first occurrence's for a recurring one. |
| `DurationMinutes` | Its length in minutes. |
| `Location` | Where it is. |
| `AllDay`, `Recurring` | Whether it is an all-day event, and whether it recurs (its exceptions are embedded items). |
| `BusyStatus` | `Free`, `Tentative`, `Busy`, `OutOfOffice` or `WorkingElsewhere`. |
| `ResponseStatus` | The user's response: `None`, `Organized`, `Tentative`, `Accepted`, `Declined` or `NotResponded`. |
| `Organizer` | The item's sender's name. |
| `AllAttendees`, `ToAttendees`, `CcAttendees` | The attendees, as the item lists them. |
| `GlobalObjectID` | The meeting's identity across mailboxes (hex): the same meeting in the organizer's and the attendees' calendars. |
| `ReminderSet`, `ReminderTime` | Whether a reminder is set, and the time it is for (UTC). |
| `CreationTime`, `LastModifiedTime` | As in the Items table. |

### Tasks

| Field | Interpretation |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | As in the Items table: `Task` or `TaskRequest`. |
| `Status` | `NotStarted`, `InProgress`, `Complete`, `Waiting` or `Deferred`. |
| `PercentComplete` | How much of it is done, from 0 to 100. |
| `StartTime`, `DueTime` | When it starts and is due: dates in the user's time zone, without an offset. |
| `CompletedTime` | When it was completed (UTC). |
| `Complete` | Whether it is complete. |
| `Owner`, `Assigner` | Who owns it, and who assigned it. |
| `ReminderSet`, `ReminderTime` | As in the Calendar table. |
| `CreationTime`, `LastModifiedTime` | As in the Items table. |

### Properties

| Field | Interpretation |
|---|---|
| `ItemID` | The item's id; `store` for the store's own properties; `folder:<FolderID>` for a folder's. |
| `Scope` | Whose property it is: `Store`, `Folder`, `Item`, `Recipient` or `Attachment`. |
| `Index` | The recipient's or attachment's index; empty for the others. |
| `PropertyTag` | The property's id and type, as `0x0037001F` (`PidTagSubject`, a string). |
| `PropertyName` | For a named property (ids from `0x8000`), its property set and name: `PSETID_Address:0x8083`, `PS_PUBLIC_STRINGS:Keywords`, or the set's GUID when it isn't a well-known one. |
| `Type` | The property type: `PtypString`, `PtypInteger32`, `PtypTime`, `PtypBinary`, `PtypMultipleString`, …. |
| `Value` | The value: text for strings, ISO 8601 UTC for times (which `-Z` does not convert), hex for binary values (the first 4,096 bytes, then `…(N bytes)`), the values of a multi-valued property as a list (a `, ` between them in CSV), `(object)` for an embedded item or an attachment's data, the bytes in hex with a note for a value of a type that isn't decoded, and, for a damaged value, what could be recovered of it followed by a note with its bytes in hex (only the bytes and the note when nothing could be recovered). |

## Automatic parsing

`auto-parse` writes the tables to the new `Communication` category as `Outlook-Files.csv`,
`Outlook-Folders.csv`, `Outlook-Items.csv`, `Outlook-Recipients.csv`, `Outlook-Attachments.csv`,
`Outlook-Contacts.csv`, `Outlook-Calendar.csv` and `Outlook-Tasks.csv` (one `Outlook.json` or
`Outlook.jsonl` file with `-t json` or `-t jsonl`), with `Communication/Outlook-logs.txt`. It
gives the parser its `--code-page`, and neither adds the Properties table nor extracts
attachments: run `parse-file outlook` for those. The combined `-j` timeline includes every row,
with `Comp` and the `Artifact` identifier `outlook`. `-Z` converts every field ending in `Time`,
except a task's `StartTime` and `DueTime`.

## Further Details

- [Microsoft — \[MS-PST\]: Outlook Personal Folders (.pst) File Format](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]: Outlook Item (.msg) File Format](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]: Message and Attachment Object Protocol](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]: Exchange Server Protocols Master Property List](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — Personal Folder File (PFF) format](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
