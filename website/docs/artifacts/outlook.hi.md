# Outlook { #outlook }

## Outlook की files क्या हैं { #what-outlooks-files-are }

Windows के लिए Outlook mail, contacts, calendars और tasks को अपनी files में रखता है:

- **OST** (`.ost`): किसी mailbox की offline copy, जिसे Outlook Exchange और Microsoft 365 accounts के लिए
  रखता है, और Outlook 2013 से IMAP accounts के लिए भी। Outlook 2013 और बाद के versions की OSTs में 4 KiB
  pages और compressed blocks होते हैं।
- **PST** (`.pst`): एक personal store: POP3 accounts, archives (`archive.pst`), exports और उपयोगकर्ताओं द्वारा
  बनाए गए stores। ANSI PSTs Outlook 97 से 2002 तक की होती हैं; Unicode PSTs Outlook 2003 और बाद के versions की।
- **MSG** (`.msg`): file के रूप में save किया गया एक item, जब उसे Outlook से बाहर drag किया जाता है, **Save As**
  से save किया जाता है, या कोई अन्य program उसे export करता है। Attach किए गए messages भी MSG files होते हैं।
- **OFT** (`.oft`): एक Outlook template: एक MSG file जिससे नए items बनाए जाते हैं।

हर store में folders होते हैं, और हर folder में उसके items: e-mail, meeting requests और responses, contacts
और distribution lists, calendar items, tasks, notes, journal entries और reports। हर folder छिपे हुए items भी
रखता है, यानी उसकी folder associated information (FAI): views, rules, forms और Outlook की settings
(`IPM.Configuration.*` items, जैसे किसी Exchange mailbox की autocomplete list)। किसी item के attachments files,
files के references, OLE objects, या अन्य items हो सकते हैं (attachment के रूप में forward किया गया message,
किसी recurring appointment के exceptions), जिनके अपने attachments होते हैं।

<span class="evidence-strength">सर्वोत्तम निष्कर्ष:</span> संग्रह के समय store या file में यह item था, इन
senders, recipients, समयों और attachments के साथ। यह नहीं बताता कि उपयोगकर्ता ने उसे पढ़ा, लिखा या भेजा, और न
ही यह कि उसका sender वही है जो item बताता है।

## स्थान और संग्रह { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

PSTs कहीं भी हो सकती हैं: उपयोगकर्ता archives को अन्य folders, drives और file shares पर ले जाते हैं। Outlook
हर profile के stores के paths उपयोगकर्ता की registry में रखता है। उपयोगकर्ता ने Outlook से जो attachments खोले,
वे `%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\` के एक folder में copy किए जाते हैं (Windows
8.1 से पहले `Temporary Internet Files\Content.Outlook`), जहाँ वे बने रह सकते हैं।

Outlook चलते समय अपने stores को खुला रखता है, और एक OST दसियों gigabytes की हो सकती है। उन्हें Outlook बंद
करके, या disk image या volume shadow copy से संग्रहित करें। Juntei हर store को stream के रूप में पढ़ता है: वह कभी
पूरे store को memory में नहीं रखता।

## विश्लेषक के लिए सावधानियाँ { #analyst-cautions }

- Juntei उदारता से (leniently) पढ़ता है: जो क्षतिग्रस्त है उसे पार करके आगे बढ़ता है, जो पढ़ा जा सकता है उसे
  पढ़ता है, और बाकी को problems के रूप में रिपोर्ट करता है। वे print की जाती हैं (हर file की पहली 10), summary
  और Files table में गिनी जाती हैं, और जिन items और attachments से वे संबंधित हैं उनके `Problems` field में
  सूचीबद्ध होती हैं। जो item बिल्कुल नहीं पढ़ा जा सकता, उसकी भी Items table में एक row होती है, उसके `ItemID`
  और कारण के साथ।
- समय UTC में हैं, 100 nanoseconds तक। `SubmitTime` वह समय है जब sender के client ने message submit किया,
  उस computer की घड़ी के अनुसार; `DeliveryTime` वह समय जब वह इस mailbox में पहुँचा। `CreationTime` वह समय है
  जब item की यह copy बनी: mailbox में deliver हुई या बनाई गई, OST में synchronize हुई, PST में copy या import
  हुई, या MSG file के रूप में save हुई। यह message लिखे जाने का समय नहीं है। `LastModifiedTime` वह समय है जब
  store में item आखिरी बार बदला गया।
- किसी task के `StartTime` और `DueTime` उपयोगकर्ता के time zone में dates हैं, जिन्हें Outlook मध्यरात्रि के रूप
  में store करता है। Juntei उन्हें बिना offset के लिखता है (`2017-02-19T00:00:00`), और `-Z` उन्हें जैसा है वैसा
  ही छोड़ देता है।
- कुछ dates में, जैसे किसी task की due date या किसी reminder का समय, Outlook **None** के लिए 1 जनवरी 4501 की
  मध्यरात्रि लिखता है। Juntei उसे जैसा stored है वैसा ही लिखता है: `4501-01-01T00:00:00Z`, और किसी task के
  `StartTime` और `DueTime` में `Z` के बिना।
- `Read` वह read flag है जो store के संग्रह के समय था: किसी व्यक्ति ने, या किसी rule ने, item को read के रूप में
  चिह्नित किया। यह न तो बताता है कि किसने, न ही कब।
- Sender और recipient के नाम और addresses वही हैं जो भेजने वाले program और mail servers ने दर्ज किए, और वे
  जाली (forged) हो सकते हैं। `EX` addresses Exchange distinguished names हैं
  (`/o=…/ou=…/cn=Recipients/cn=…`); जब item SMTP address भी store करता है, तो वह `SenderSMTP-Address` और
  `SMTP-Address` में होता है। Message कहाँ से आया, यह जाँचने के लिए उसके Internet headers पढ़ें: Properties
  table का `0x007D001F` या `0x007D001E` (`PidTagTransportMessageHeaders`)।
- Outlook में delete किए गए items **Deleted Items** में तब तक रहते हैं जब तक उसे खाली नहीं किया जाता, और किसी भी
  अन्य item की तरह पढ़े जाते हैं। जो items अब किसी भी folder की table में नहीं हैं, जिन्हें store उनकी जगह दोबारा
  उपयोग होने तक रख सकता है, वे अभी नहीं पढ़े जाते।
- `ItemID` उसकी PST या OST में message का node id है: store की किसी अन्य copy में उसी item का id अलग हो सकता
  है। किसी embedded item का id उसके parent का id और attachment का index है (`2097348/0`, `0/0/1`); किसी `.msg`
  या `.oft` file के item का id `0` है।
- `User` केवल path से आता है (`Users` के बाद वाला folder), और file कहीं और copy की जाने पर खो जाता है। एक PST
  किसी भी उपयोगकर्ता की हो सकती है।
- जो text Unicode नहीं है, उसे item के code page में पढ़ा जाता है (उसका अपना, उसके locale का या उसके Internet code
  page का)। `--code-page` उन items का code page देता है जो अपना code page नहीं बताते: उन्हें लिखने वाले system का
  ANSI code page, Japanese system के लिए `932`।
- PST का password उसे encrypt नहीं करता: Juntei password-protected PSTs को किसी भी अन्य PST की तरह पढ़ता है।
  किसी store का `Encryption` (`Permute`, Outlook का "compressible encryption", या `Cyclic`) केवल उसके blocks को
  अस्पष्ट (obscure) करता है। S/MIME से encrypt किए गए messages अपना body और attachments एक encrypted attachment
  (`smime.p7m`) में रखते हैं, जिसे decrypt नहीं किया जाता।
- Attachments अक्सर malware होते हैं। `--extract-attachments` उनका data उनके SHA-256 के नाम वाली files में लिखता
  है, Unix पर नई files के लिए mode `0600` के साथ; उन्हें कभी न चलाएँ, और उन्हें एक अलग-थलग (isolated) analysis system पर ही रखें।

### अभी नहीं पढ़ा जाता { #not-read-yet }

Offline address books (OAB), autocomplete stream (`Stream_Autocomplete_*.dat`) और `.nk2` files, send/receive
settings (`.srs`), Outlook के settings caches (RoamCache, Autodiscover), Outlook के SQLite databases, और वे
messages जो अब किसी भी folder की table में नहीं हैं।

## Juntei से parse करें { #parse-with-juntei }

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

`-d` बिना case का भेद किए `.pst`, `.ost`, `.msg` और `.oft` नाम खोजता है, फिर header जाँचता है: PST या OST के लिए `!BDN`, MSG या OFT के लिए compound-file signature। समान extension वाली असंबंधित files, जैसे Tcl message catalogs, छोड़ दी जाती हैं। न पढ़ी जा सकने वाली files और पहचान योग्य signature वाले क्षतिग्रस्त containers अब भी errors हैं। `-f` नाम की परवाह किए बिना दी गई file को सत्यापित करता है और अमान्य input को error बताता है। कोई file न पढ़ी जा सके तो exit status 1 है।

`--extract-attachments` हर attachment का data एक बार लिखता है, `<SHA-256>` नाम से, और जब attachment का extension
1 से 10 ASCII अक्षरों और अंकों का हो तो उस extension के साथ (`0c7d…1311.jpg`); किसी item से आया कोई भी नाम path
में उपयोग नहीं होता। जब तक `-C` न दिया जाए, directory खाली होनी चाहिए; `-C` के साथ, उसी नाम की ऐसी file जिसमें
कुछ और है, बदल दी जाती है।

## आउटपुट tables { #output-tables }

CSV आठ files बनाता है, `--properties` के साथ नौ; rows न होने पर हर file में केवल उसका header होता है। JSON और
JSONL rows में `SourceFile` के बाद `Table` (`Files`, `Folders`, `Items`, `Recipients`, `Attachments`,
`Contacts`, `Calendar`, `Tasks` या `Properties`) होता है। किसी item की rows उसके बाद आती हैं, और उसके embedded
items की rows उनके बाद; किसी file की Files row उसकी बाकी सभी rows के बाद आती है।

| CSV file | सामग्री |
|---|---|
| `Outlook-Files.csv` | पढ़ी गई हर file के लिए एक row: store या file, और उसमें क्या है। |
| `Outlook-Folders.csv` | PSTs और OSTs के folders। |
| `Outlook-Items.csv` | हर item: folders के messages, उनके छिपे हुए items, `.msg` और `.oft` files के items, और उनके attachments में embedded items। |
| `Outlook-Recipients.csv` | Items के recipients। |
| `Outlook-Attachments.csv` | Items के attachments। |
| `Outlook-Contacts.csv` | Contacts और distribution lists के fields। |
| `Outlook-Calendar.csv` | Calendar items और meeting messages के fields। |
| `Outlook-Tasks.csv` | Tasks और task requests के fields। |
| `Outlook-Properties.csv` | `--properties` के साथ: stores, folders, items, recipients और attachments की हर property, जैसी stored है। |

## Fields { #fields }

हर table इन fields से शुरू होती है:

| Field | व्याख्या |
|---|---|
| `SourceFile` | वह file जिससे row पढ़ी गई। |
| `User` | वह Windows उपयोगकर्ता जिसकी profile में path है (`Users` के बाद वाला folder)। |

### Files { #files }

| Field | व्याख्या |
|---|---|
| `FileType` | `PST`, `OST`, `MSG`, `OFT`, या item के रूप में पढ़ी गई ऐसी compound file के लिए `CFB` जिसका class ID न message का है, न template का (Outlook के अलावा कई programs उसे खाली छोड़ देते हैं)। |
| `Format` | किसी store का layout: `ANSI` (Outlook 97 से 2002), `Unicode` (Outlook 2003 और बाद के versions) या `Unicode 4K` (Outlook 2013 और बाद के versions की OSTs); किसी MSG या OFT file का compound file version (`CFB v3`, `CFB v4`)। |
| `Version`, `ClientVersion` | Store के header का `wVer` और `wVerClient`। |
| `Encryption` | Store के blocks कैसे encode किए गए हैं: `None`, `Permute` या `Cyclic`। |
| `DisplayName` | Store का नाम (`Personal Folders`, कोई e-mail address)। |
| `RecordKey` | Store की पहचान (hex), जो उसके items के entry ids में होती है। |
| `Folders` | Store के folders। |
| `Items` | उसके folders के messages, या MSG या OFT file के लिए 1। |
| `HiddenItems` | उसके folders के छिपे हुए items (folder associated information)। |
| `EmbeddedItems` | Attachments में embedded items, हर स्तर पर। |
| `Attachments` | उन सभी items के attachments। |
| `Problems` | File में पाई गई problems। |

### Folders { #folders }

| Field | व्याख्या |
|---|---|
| `FolderID`, `ParentID` | Folder का node id, और उसके parent folder का (root folder के लिए खाली)। |
| `FolderPath` | Root से नीचे तक folders के नाम: `/Top of Personal Folders/Inbox`, `/Root - Mailbox/IPM_SUBTREE/Sent Items`। |
| `Name` | Folder का नाम। |
| `ContainerClass` | Folder में क्या है: `IPF.Note` (mail), `IPF.Appointment`, `IPF.Contact`, `IPF.Task`, `IPF.Imap`, …। |
| `ContentCount`, `UnreadCount` | Folder के अनुसार उसमें कितने messages हैं, और कितने unread messages। |
| `Messages`, `HiddenItems`, `Subfolders` | उसकी contents table, उसकी associated contents table और उसकी hierarchy table की rows: वह जो Juntei पढ़ता है। `ContentCount` से कोई अंतर देखने लायक है। |
| `CreationTime`, `LastModifiedTime` | Folder कब बना और आखिरी बार कब बदला (UTC), जब store उन्हें रखता है। |
| `Problem` | Folder क्यों नहीं खोला जा सका; तब उसके subfolders नहीं पढ़े जाते। |

### Items { #items }

| Field | व्याख्या |
|---|---|
| `FolderPath` | Item का folder; MSG या OFT file में खाली। |
| `ItemID` | Item का id: PST या OST में उसका node id, MSG या OFT file में `0`, embedded item के लिए उसके parent का id और attachment का index (`2097348/0`)। |
| `Hidden` | क्या item folder associated information है: एक छिपा हुआ item। |
| `EmbeddedIn` | उस item का id जिससे यह attach है; बाकी items के लिए खाली। |
| `ItemType` | Message class के अनुसार item क्या है: `Mail`, `MeetingMessage`, `Appointment`, `Contact`, `DistributionList`, `Task`, `TaskRequest`, `StickyNote`, `Journal`, `Post`, `Report`, `Configuration` या `Other`। |
| `MessageClass` | Message class, जैसी stored है (`IPM.Note`, `IPM.Note.SMIME`, `IPM.Schedule.Meeting.Request`, `REPORT.IPM.Note.NDR`, …)। |
| `SubmitTime` | Sender के client ने message कब submit किया (UTC, sender की घड़ी)। |
| `DeliveryTime` | वह इस mailbox में कब deliver हुआ (UTC)। |
| `CreationTime`, `LastModifiedTime` | यह copy store या file में कब बनी, और आखिरी बार कब बदली (UTC)। |
| `Subject` | Subject, उसके prefix (`RE: `) के साथ। |
| `SenderName`, `SenderAddressType`, `SenderAddress` | किसने भेजा: नाम, address type (`SMTP`, `EX`) और उस type का address। |
| `SenderSMTP-Address` | Sender का SMTP address, जब वह किसी अन्य type के address के साथ-साथ stored हो। |
| `SentRepresentingName`, `SentRepresentingAddress` | किसकी ओर से भेजा गया (कोई delegate किसी manager की ओर से भेजता है; कोई shared mailbox), stored होने पर उनके SMTP address के साथ, अन्यथा उसके type का address। |
| `To`, `Cc`, `Bcc` | Recipients के नाम, जैसे item उन्हें दिखाता है (उनके addresses के लिए Recipients देखें)। `Bcc` केवल sender की copy में होता है। |
| `InternetMessageID`, `InReplyTo` | Message का `Message-ID`, और उस message का जिसका यह reply है। |
| `ConversationTopic` | Prefix के बिना subject, जो किसी conversation के सभी messages में समान होता है। |
| `Importance`, `Sensitivity` | `Low`, `Normal` या `High`; `Normal`, `Personal`, `Private` या `Confidential`। |
| `Read` | क्या item read के रूप में चिह्नित है। |
| `Flags` | Message flags (`PidTagMessageFlags`, hex): `0x1` read, `0x2` unmodified, `0x4` submitted, `0x8` unsent (draft), `0x10` attachments हैं, `0x20` मेरी ओर से, `0x40` hidden, `0x80` resent, `0x400` कभी पढ़ा गया, `0x2000` Internet से। |
| `Size` | Store में item का size, उसके attachments सहित। |
| `Recipients`, `Attachments` | उसके कितने recipients और attachments हैं। |
| `Categories` | उसकी categories, उनके बीच `;` के साथ। |
| `BodyPreview` | उसके plain-text body की शुरुआत, white space को समेटकर, अधिकतम 256 characters; जब उसमें केवल HTML या RTF body हो तो खाली। |
| `Problems` | Item में क्या पढ़ा नहीं जा सका (अधिकतम 10, फिर कितनी और)। |

### Recipients { #recipients }

| Field | व्याख्या |
|---|---|
| `ItemID` | Item का id। |
| `Index` | Item की recipient table में recipient की row, 0 से। |
| `RecipientType` | `To`, `Cc`, `Bcc`, `Originator`, या value जैसी stored है (hex)। |
| `Name`, `AddressType`, `Address` | Recipient का नाम, address type और address। |
| `SMTP-Address` | SMTP address, जब वह किसी अन्य type के address के साथ-साथ stored हो। |

### Attachments { #attachments }

| Field | व्याख्या |
|---|---|
| `ItemID`, `Index` | Item का id, और उसकी attachment table में attachment की row, 0 से। |
| `Method` | `ByValue` (एक file), `ByReference`, `ByReferenceResolve`, `ByReferenceOnly` (किसी file का path), `EmbeddedMessage` (एक item), `OLE` (एक OLE object), `ByWebReference`, या संख्या। |
| `Filename` | Attachment का नाम: उसका long filename, उसका short filename, या उसका display name। |
| `Extension` | उसका extension जैसा stored है (`.pdf`), अन्यथा उसके नाम का। |
| `MimeType`, `ContentID` | उसका MIME type, और वह content id जिससे HTML body उसका संदर्भ देता है। |
| `Inline` | क्या body उसे सूची में रखने के बजाय दिखाता है (HTML body की कोई image)। |
| `Hidden` | क्या Outlook उसे attachment list से छिपाता है। |
| `Size` | उसका size जैसा store देता है, उसकी properties सहित। |
| `DataSize`, `SHA256` | उसके data का size और SHA-256: file, या PST या OST में किसी OLE object की compound file। Embedded item और reference वाले attachment के लिए खाली। |
| `Pathname` | Reference वाले attachment का path। |
| `CreationTime`, `LastModifiedTime` | Attachment के समय (UTC): अक्सर उस file के समय जिससे वह बना। |
| `EmbeddedSubject` | Embedded item का subject, जिसकी अपनी rows का `ItemID` `<ItemID>/<Index>` है। |
| `ExtractedFile` | `--extract-attachments` के साथ, उस directory में वह file जिसमें उसका data लिखा गया। |
| `Problems` | Attachment में क्या पढ़ा नहीं जा सका, या लिखा नहीं जा सका। |

### Contacts { #contacts }

| Field | व्याख्या |
|---|---|
| `FolderPath`, `ItemID`, `ItemType` | Items table की तरह: `Contact` या `DistributionList`। |
| `DisplayName`, `GivenName`, `Surname`, `Nickname` | Contact के नाम। |
| `FileUnder` | वह नाम जिसके अंतर्गत contact file किया गया है। |
| `Company`, `Title`, `Department` | Contact कहाँ काम करता है। |
| `Email1Name`, `Email1Address`, `Email2Name`, `Email2Address`, `Email3Name`, `Email3Address` | Contact के तीन e-mail addresses, हर एक अपने display name के साथ। |
| `BusinessPhone`, `HomePhone`, `MobilePhone` | Contact के phone numbers। |
| `InstantMessaging` | Contact का instant messaging address। |
| `CreationTime`, `LastModifiedTime` | Items table की तरह। |

### Calendar { #calendar }

| Field | व्याख्या |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Items table की तरह: `Appointment` या `MeetingMessage`। |
| `StartTime`, `EndTime` | Appointment कब शुरू होता है और कब समाप्त होता है (UTC); recurring appointment के लिए पहली occurrence के समय। |
| `DurationMinutes` | उसकी अवधि, minutes में। |
| `Location` | वह कहाँ है। |
| `AllDay`, `Recurring` | क्या वह पूरे दिन का event है, और क्या वह दोहराया जाता है (उसके exceptions embedded items हैं)। |
| `BusyStatus` | `Free`, `Tentative`, `Busy`, `OutOfOffice` या `WorkingElsewhere`। |
| `ResponseStatus` | उपयोगकर्ता का response: `None`, `Organized`, `Tentative`, `Accepted`, `Declined` या `NotResponded`। |
| `Organizer` | Item के sender का नाम। |
| `AllAttendees`, `ToAttendees`, `CcAttendees` | Attendees, जैसे item उन्हें सूचीबद्ध करता है। |
| `GlobalObjectID` | सभी mailboxes में meeting की पहचान (hex): organizer और attendees के calendars में वही meeting। |
| `ReminderSet`, `ReminderTime` | क्या reminder set है, और वह किस समय के लिए है (UTC)। |
| `CreationTime`, `LastModifiedTime` | Items table की तरह। |

### Tasks { #tasks }

| Field | व्याख्या |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Items table की तरह: `Task` या `TaskRequest`। |
| `Status` | `NotStarted`, `InProgress`, `Complete`, `Waiting` या `Deferred`। |
| `PercentComplete` | उसका कितना हिस्सा पूरा हुआ, 0 से 100 तक। |
| `StartTime`, `DueTime` | वह कब शुरू होता है और कब due है: उपयोगकर्ता के time zone में dates, बिना offset के। |
| `CompletedTime` | वह कब पूरा हुआ (UTC)। |
| `Complete` | क्या वह पूरा हो चुका है। |
| `Owner`, `Assigner` | उसका owner कौन है, और उसे किसने assign किया। |
| `ReminderSet`, `ReminderTime` | Calendar table की तरह। |
| `CreationTime`, `LastModifiedTime` | Items table की तरह। |

### Properties { #properties }

| Field | व्याख्या |
|---|---|
| `ItemID` | Item का id; store की अपनी properties के लिए `store`; किसी folder की properties के लिए `folder:<FolderID>`। |
| `Scope` | Property किसकी है: `Store`, `Folder`, `Item`, `Recipient` या `Attachment`। |
| `Index` | Recipient या attachment का index; बाकी के लिए खाली। |
| `PropertyTag` | Property का id और type, `0x0037001F` के रूप में (`PidTagSubject`, एक string)। |
| `PropertyName` | किसी named property के लिए (`0x8000` से आगे के ids), उसका property set और नाम: `PSETID_Address:0x8083`, `PS_PUBLIC_STRINGS:Keywords`, या set का GUID जब वह कोई well-known set न हो। |
| `Type` | Property type: `PtypString`, `PtypInteger32`, `PtypTime`, `PtypBinary`, `PtypMultipleString`, …। |
| `Value` | Value: strings के लिए text, समयों के लिए ISO 8601 UTC (जिसे `-Z` convert नहीं करता), binary values के लिए hex (पहले 4,096 bytes, फिर `…(N bytes)`), multi-valued property की values एक सूची के रूप में (CSV में उनके बीच `, `), embedded item या attachment के data के लिए `(object)`, ऐसे type की value के लिए जिसे decode नहीं किया जाता, bytes hex में एक note के साथ, और क्षतिग्रस्त value के लिए उसमें से जितना recover हो सका, उसके बाद एक note जिसमें उसके bytes hex में हों (कुछ भी recover न हो पाने पर केवल bytes और note)। |

## स्वचालित विश्लेषण { #automatic-parsing }

`auto-parse` tables को नई `Communication` (संचार) category में `Outlook-Files.csv`, `Outlook-Folders.csv`,
`Outlook-Items.csv`, `Outlook-Recipients.csv`, `Outlook-Attachments.csv`, `Outlook-Contacts.csv`,
`Outlook-Calendar.csv` और `Outlook-Tasks.csv` के रूप में लिखता है (`-t json` या `-t jsonl` के साथ एक ही
`Outlook.json` या `Outlook.jsonl` file), साथ में `Communication/Outlook-logs.txt` भी। यह parser को अपना
`--code-page` देता है, और न तो Properties table जोड़ता है और न ही attachments extract करता है: उनके लिए
`parse-file outlook` चलाएँ। संयुक्त `-j` timeline में हर row शामिल होती है, `Comp` और `Artifact` identifier
`outlook` के साथ। `-Z` `Time` पर समाप्त होने वाले हर field को convert करता है, सिवाय किसी task के `StartTime` और
`DueTime` के।

## अधिक जानकारी { #further-details }

- [Microsoft — \[MS-PST\]: Outlook Personal Folders (.pst) File Format](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]: Outlook Item (.msg) File Format](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]: Message and Attachment Object Protocol](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]: Exchange Server Protocols Master Property List](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — Personal Folder File (PFF) format](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
