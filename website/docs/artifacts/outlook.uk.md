# Outlook { #outlook }

## Що таке файли Outlook { #what-outlooks-files-are }

Outlook для Windows зберігає пошту, контакти, календарі й завдання у власних файлах:

- **OST** (`.ost`): офлайн-копія поштової скриньки, яку Outlook веде для облікових записів Exchange і
  Microsoft 365, а починаючи з Outlook 2013 — і для облікових записів IMAP. Файли OST Outlook 2013 і
  новіших мають сторінки по 4 KiB і стиснені блоки.
- **PST** (`.pst`): особисте сховище: облікові записи POP3, архіви (`archive.pst`), експорти та
  сховища, які створюють користувачі. ANSI PST — це файли Outlook від 97 до 2002; Unicode PST — файли
  Outlook 2003 і новіших.
- **MSG** (`.msg`): один елемент, збережений як файл, коли його перетягують з Outlook, зберігають
  командою **Зберегти як** або експортує інша програма. Вкладені повідомлення — теж файли MSG.
- **OFT** (`.oft`): шаблон Outlook — файл MSG, з якого створюються нові елементи.

Кожне сховище містить папки, а кожна папка — свої елементи: електронні листи, запрошення на наради та
відповіді на них, контакти й списки розсилки, елементи календаря, завдання, нотатки, записи щоденника
та звіти. Кожна папка також зберігає приховані елементи — пов'язану з папкою інформацію (FAI):
подання, правила, форми та налаштування Outlook (елементи `IPM.Configuration.*`, як-от список
автозаповнення поштової скриньки Exchange). Вкладеннями елемента можуть бути файли, посилання на
файли, об'єкти OLE або інші елементи (повідомлення, переслане як вкладення, винятки повторюваної
зустрічі), які мають власні вкладення.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> на момент збирання сховище або
файл містили елемент із цими відправниками, одержувачами, часом і вкладеннями. Це не свідчить, що
користувач його прочитав, написав чи надіслав, і не свідчить, що відправник — саме той, ким його
називає елемент.

## Розташування та збирання { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

Файли PST можуть бути будь-де: користувачі переносять архіви в інші папки, на інші диски та у
спільні файлові ресурси. Outlook зберігає шляхи до сховищ кожного профілю в реєстрі користувача.
Вкладення, які користувач відкривав з Outlook, копіюються в папку всередині
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\` (до Windows 8.1 —
`Temporary Internet Files\Content.Outlook`), де вони можуть залишатися.

Outlook тримає свої сховища відкритими, поки працює, а файл OST може займати десятки гігабайтів.
Збирайте їх із закритим Outlook або з образу диска чи тіньової копії тому. Juntei читає кожне
сховище як потік і ніколи не тримає все сховище в пам'яті.

## Застереження для аналітика { #analyst-cautions }

- Juntei читає поблажливо: оминає пошкоджене, читає те, що можна прочитати, а решту повідомляє як
  проблеми. Їх виводять на екран (перші 10 для кожного файлу), підраховують у підсумку й у таблиці
  Файли та перелічують у полі `Problems` елементів і вкладень, яких вони стосуються. Елемент, який
  не вдається прочитати зовсім, однаково має рядок у таблиці Елементи — зі своїм `ItemID` і
  причиною.
- Час подано в UTC, з точністю до 100 наносекунд. `SubmitTime` — коли клієнт відправника передав
  повідомлення на надсилання, за годинником того комп'ютера; `DeliveryTime` — коли воно надійшло до
  цієї поштової скриньки. `CreationTime` — коли створено цю копію елемента: доставлено до поштової
  скриньки або створено в ній, синхронізовано в OST, скопійовано чи імпортовано в PST або збережено як
  файл MSG. Це не час написання повідомлення. `LastModifiedTime` — коли елемент востаннє змінено в
  сховищі.
- `StartTime` і `DueTime` завдання — це дати в часовому поясі користувача, які Outlook зберігає як
  північ. Juntei записує їх без зміщення (`2017-02-19T00:00:00`), і `-Z` залишає їх без змін.
- Для **Немає** (None) в деяких датах, як-от у терміні виконання завдання чи часі нагадування, Outlook
  записує північ 1 січня 4501 року. Juntei записує це значення так, як його збережено:
  `4501-01-01T00:00:00Z`, а в `StartTime` і `DueTime` завдання — без `Z`.
- `Read` — це прапорець прочитання на момент збирання сховища: хтось або якесь правило позначили
  елемент як прочитаний. Він не каже ні хто, ні коли.
- Імена та адреси відправника й одержувача — це те, що записали програма-відправник і поштові
  сервери, і їх можна підробити. Адреси `EX` — це розрізнювальні імена Exchange (distinguished names)
  (`/o=…/ou=…/cn=Recipients/cn=…`); SMTP-адреса міститься в `SenderSMTP-Address` і `SMTP-Address`,
  коли елемент зберігає її додатково. Щоб перевірити, звідки надійшло повідомлення, читайте його
  інтернет-заголовки: `0x007D001F` або `0x007D001E` (`PidTagTransportMessageHeaders`) у таблиці
  Властивості.
- Елементи, видалені в Outlook, лежать у папці **Видалені** (Deleted Items), доки її не очистять, і
  читаються як будь-які інші. Елементи, яких уже немає в таблиці жодної папки і які сховище може
  зберігати, доки не використає їхнє місце повторно, поки що не читаються.
- `ItemID` — це ідентифікатор вузла (node id) повідомлення в його PST або OST: той самий елемент в
  іншій копії сховища може мати інший. Ідентифікатор вбудованого елемента складається з
  ідентифікатора батьківського елемента та індексу вкладення (`2097348/0`, `0/0/1`); елемент файлу
  `.msg` або `.oft` має ідентифікатор `0`.
- `User` береться лише зі шляху (папка після `Users`) і втрачається, коли файл копіюють в інше місце.
  PST може належати будь-якому користувачеві.
- Текст не в Unicode читається в кодовій сторінці елемента (його власній, кодовій сторінці його
  локалі або його інтернет-кодовій сторінці). `--code-page` задає кодову сторінку для елементів, які
  її не вказують: кодову сторінку ANSI системи, що їх записала, `932` для японської системи.
- Пароль PST не шифрує файл: Juntei читає PST, захищені паролем, як і будь-які інші. `Encryption`
  сховища (`Permute`, «стискуване шифрування» Outlook (compressible encryption), або `Cyclic`) лише
  маскує його блоки. Повідомлення, зашифровані за допомогою S/MIME, зберігають свій текст і вкладення
  в зашифрованому вкладенні (`smime.p7m`), яке не розшифровується.
- Вкладення часто бувають шкідливим програмним забезпеченням. `--extract-attachments` записує їхні
  дані у файли, названі за їхнім SHA-256, без дозволу на виконання; ніколи не запускайте їх і тримайте
  їх в ізольованій системі для аналізу.

### Що поки не читається { #not-read-yet }

Офлайнові адресні книги (OAB), потік автозаповнення (`Stream_Autocomplete_*.dat`) і файли `.nk2`,
налаштування надсилання й отримання (`.srs`), кеші налаштувань Outlook (RoamCache, Autodiscover), бази
даних SQLite Outlook, а також повідомлення, яких уже немає в таблиці жодної папки.

## Аналіз за допомогою Juntei { #parse-with-juntei }

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

`-d` шукає файли, назви яких закінчуються на `.pst`, `.ost`, `.msg` і `.oft`, без урахування регістру.
Хоч би як було задано файл, що це за файл, визначають його перші байти: `!BDN` для PST або OST,
сигнатура складеного файлу для MSG або OFT. Про будь-що інше повідомляється як про файл, який не
вдалося розібрати. Код завершення дорівнює 1, коли не вдалося прочитати жодного файлу.

`--extract-attachments` записує дані кожного вкладення один раз, як `<SHA-256>` з розширенням
вкладення, якщо воно складається з 1–10 літер і цифр ASCII (`0c7d…1311.jpg`); жодне ім'я з
елемента не використовується у шляху. Каталог має бути порожнім, якщо не задано `-C`; з `-C` файл з
тим самим ім'ям, що містить щось інше, замінюється.

## Таблиці виводу { #output-tables }

CSV створює вісім файлів, а з `--properties` — дев'ять; файл без рядків містить лише заголовок. Рядки
JSON і JSONL містять `Table` (`Files`, `Folders`, `Items`, `Recipients`, `Attachments`, `Contacts`,
`Calendar`, `Tasks` або `Properties`) після `SourceFile`. Рядки елемента йдуть після нього, а рядки
його вбудованих елементів — після них; рядок файлу в таблиці Файли йде після решти його рядків.

| Файл CSV | Вміст |
|---|---|
| `Outlook-Files.csv` | Один рядок на кожен прочитаний файл: сховище або файл і те, що він містить. |
| `Outlook-Folders.csv` | Папки файлів PST і OST. |
| `Outlook-Items.csv` | Усі елементи: повідомлення папок, їхні приховані елементи, елементи файлів `.msg` і `.oft` та елементи, вбудовані в їхні вкладення. |
| `Outlook-Recipients.csv` | Одержувачі елементів. |
| `Outlook-Attachments.csv` | Вкладення елементів. |
| `Outlook-Contacts.csv` | Поля контактів і списків розсилки. |
| `Outlook-Calendar.csv` | Поля елементів календаря та повідомлень про наради. |
| `Outlook-Tasks.csv` | Поля завдань і запитів на завдання. |
| `Outlook-Properties.csv` | З `--properties`: усі властивості сховищ, папок, елементів, одержувачів і вкладень у збереженому вигляді. |

## Поля { #fields }

Кожна таблиця починається з таких полів:

| Поле | Інтерпретація |
|---|---|
| `SourceFile` | Файл, з якого прочитано рядок. |
| `User` | Користувач Windows, у профілі якого лежить шлях (папка після `Users`). |

### Файли (Files) { #files }

| Поле | Інтерпретація |
|---|---|
| `FileType` | `PST`, `OST`, `MSG`, `OFT` або `CFB` для складеного файлу, прочитаного як елемент, ідентифікатор класу (class ID) якого не належить ні повідомленню, ні шаблону (багато програм, крім Outlook, залишають його порожнім). |
| `Format` | Структура сховища: `ANSI` (Outlook від 97 до 2002), `Unicode` (Outlook 2003 і новіші) або `Unicode 4K` (файли OST Outlook 2013 і новіших); для файлу MSG або OFT — версія складеного файлу (`CFB v3`, `CFB v4`). |
| `Version`, `ClientVersion` | `wVer` і `wVerClient` із заголовка сховища. |
| `Encryption` | Як закодовано блоки сховища: `None`, `Permute` або `Cyclic`. |
| `DisplayName` | Назва сховища (`Personal Folders`, адреса електронної пошти). |
| `RecordKey` | Ідентичність сховища (у шістнадцятковому вигляді), яку містять ідентифікатори записів (entry ids) його елементів. |
| `Folders` | Папки сховища. |
| `Items` | Повідомлення його папок або 1 для файлу MSG чи OFT. |
| `HiddenItems` | Приховані елементи (пов'язана з папками інформація) його папок. |
| `EmbeddedItems` | Елементи, вбудовані у вкладення, на всіх рівнях. |
| `Attachments` | Вкладення всіх цих елементів. |
| `Problems` | Проблеми, виявлені у файлі. |

### Папки (Folders) { #folders }

| Поле | Інтерпретація |
|---|---|
| `FolderID`, `ParentID` | Ідентифікатор вузла папки та ідентифікатор її батьківської папки (порожній для кореневої папки). |
| `FolderPath` | Назви папок від кореня вниз: `/Top of Personal Folders/Inbox`, `/Root - Mailbox/IPM_SUBTREE/Sent Items`. |
| `Name` | Назва папки. |
| `ContainerClass` | Що містить папка: `IPF.Note` (пошта), `IPF.Appointment`, `IPF.Contact`, `IPF.Task`, `IPF.Imap`, …. |
| `ContentCount`, `UnreadCount` | Повідомлення та непрочитані повідомлення, які, за словами папки, вона має. |
| `Messages`, `HiddenItems`, `Subfolders` | Рядки її таблиці вмісту, таблиці пов'язаного вмісту та таблиці ієрархії — те, що читає Juntei. Розбіжність із `ContentCount` варто перевірити. |
| `CreationTime`, `LastModifiedTime` | Коли папку створено і востаннє змінено (UTC), якщо сховище це зберігає. |
| `Problem` | Чому папку не вдалося відкрити; її підпапки тоді не читаються. |

### Елементи (Items) { #items }

| Поле | Інтерпретація |
|---|---|
| `FolderPath` | Папка елемента; порожнє у файлі MSG або OFT. |
| `ItemID` | Ідентифікатор елемента: його ідентифікатор вузла в PST або OST, `0` у файлі MSG або OFT, ідентифікатор батьківського елемента та індекс вкладення для вбудованого елемента (`2097348/0`). |
| `Hidden` | Чи є елемент пов'язаною з папкою інформацією, тобто прихованим елементом. |
| `EmbeddedIn` | Ідентифікатор елемента, до якого його вкладено; порожнє для інших. |
| `ItemType` | Чим є елемент згідно з класом повідомлення: `Mail`, `MeetingMessage`, `Appointment`, `Contact`, `DistributionList`, `Task`, `TaskRequest`, `StickyNote`, `Journal`, `Post`, `Report`, `Configuration` або `Other`. |
| `MessageClass` | Клас повідомлення в збереженому вигляді (`IPM.Note`, `IPM.Note.SMIME`, `IPM.Schedule.Meeting.Request`, `REPORT.IPM.Note.NDR`, …). |
| `SubmitTime` | Коли клієнт відправника передав повідомлення на надсилання (UTC, за годинником відправника). |
| `DeliveryTime` | Коли його доставлено до цієї поштової скриньки (UTC). |
| `CreationTime`, `LastModifiedTime` | Коли цю копію створено в сховищі або файлі і коли її востаннє змінено (UTC). |
| `Subject` | Тема з її префіксом (`RE: `). |
| `SenderName`, `SenderAddressType`, `SenderAddress` | Хто його надіслав: ім'я, тип адреси (`SMTP`, `EX`) і адреса цього типу. |
| `SenderSMTP-Address` | SMTP-адреса відправника, якщо її збережено додатково до адреси іншого типу. |
| `SentRepresentingName`, `SentRepresentingAddress` | Від чийого імені його надіслано (делегат надсилає від імені керівника; спільна поштова скринька), з SMTP-адресою, якщо її збережено, інакше з адресою відповідного типу. |
| `To`, `Cc`, `Bcc` | Імена одержувачів так, як їх показує елемент (їхні адреси див. у таблиці Одержувачі). `Bcc` є лише в копії відправника. |
| `InternetMessageID`, `InReplyTo` | `Message-ID` повідомлення та ідентифікатор повідомлення, на яке воно відповідає. |
| `ConversationTopic` | Тема без префікса, спільна для повідомлень однієї розмови. |
| `Importance`, `Sensitivity` | `Low`, `Normal` або `High`; `Normal`, `Personal`, `Private` або `Confidential`. |
| `Read` | Чи позначено елемент як прочитаний. |
| `Flags` | Прапорці повідомлення (`PidTagMessageFlags`, у шістнадцятковому вигляді): `0x1` прочитано, `0x2` не змінено, `0x4` передано на надсилання, `0x8` не надіслано (чернетка), `0x10` має вкладення, `0x20` від мене, `0x40` приховано, `0x80` надіслано повторно, `0x400` колись прочитано, `0x2000` з інтернету. |
| `Size` | Розмір елемента в сховищі разом із вкладеннями. |
| `Recipients`, `Attachments` | Скільки в нього одержувачів і вкладень. |
| `Categories` | Його категорії, розділені `;`. |
| `BodyPreview` | Початок його текстового вмісту в простому тексті зі згорнутими пробілами, не більше ніж 256 символів; порожнє, якщо вміст є лише в HTML або RTF. |
| `Problems` | Що не вдалося прочитати в елементі (не більше 10, а далі — скільки ще). |

### Одержувачі (Recipients) { #recipients }

| Поле | Інтерпретація |
|---|---|
| `ItemID` | Ідентифікатор елемента. |
| `Index` | Рядок одержувача в таблиці одержувачів елемента, починаючи з 0. |
| `RecipientType` | `To`, `Cc`, `Bcc`, `Originator` або значення в збереженому вигляді (у шістнадцятковому вигляді). |
| `Name`, `AddressType`, `Address` | Ім'я, тип адреси та адреса одержувача. |
| `SMTP-Address` | SMTP-адреса, якщо її збережено додатково до адреси іншого типу. |

### Вкладення (Attachments) { #attachments }

| Поле | Інтерпретація |
|---|---|
| `ItemID`, `Index` | Ідентифікатор елемента та рядок вкладення в його таблиці вкладень, починаючи з 0. |
| `Method` | `ByValue` (файл), `ByReference`, `ByReferenceResolve`, `ByReferenceOnly` (шлях до файлу), `EmbeddedMessage` (елемент), `OLE` (об'єкт OLE), `ByWebReference` або число. |
| `Filename` | Ім'я вкладення: його довге ім'я файлу, коротке ім'я або відображуване ім'я. |
| `Extension` | Його розширення в збереженому вигляді (`.pdf`), інакше розширення з його імені. |
| `MimeType`, `ContentID` | Його тип MIME та ідентифікатор вмісту, за яким на нього посилається вміст HTML. |
| `Inline` | Чи показує його вміст листа (зображення у вмісті HTML), а не перелічує серед вкладень. |
| `Hidden` | Чи приховує його Outlook зі списку вкладень. |
| `Size` | Його розмір, як його подає сховище, разом із його властивостями. |
| `DataSize`, `SHA256` | Розмір і SHA-256 його даних: файлу або складеного файлу об'єкта OLE в PST чи OST. Порожні для вбудованого елемента та вкладення за посиланням. |
| `Pathname` | Шлях вкладення за посиланням. |
| `CreationTime`, `LastModifiedTime` | Час вкладення (UTC): часто це час файлу, з якого його створено. |
| `EmbeddedSubject` | Тема вбудованого елемента, власні рядки якого мають `ItemID` `<ItemID>/<Index>`. |
| `ExtractedFile` | З `--extract-attachments` — файл у цьому каталозі, до якого записано його дані. |
| `Problems` | Що не вдалося прочитати у вкладенні або записати. |

### Контакти (Contacts) { #contacts }

| Поле | Інтерпретація |
|---|---|
| `FolderPath`, `ItemID`, `ItemType` | Як у таблиці Елементи: `Contact` або `DistributionList`. |
| `DisplayName`, `GivenName`, `Surname`, `Nickname` | Імена контакту. |
| `FileUnder` | Ім'я, під яким зберігається контакт. |
| `Company`, `Title`, `Department` | Де працює контакт. |
| `Email1Name`, `Email1Address`, `Email2Name`, `Email2Address`, `Email3Name`, `Email3Address` | Три адреси електронної пошти контакту, кожна з відображуваним ім'ям. |
| `BusinessPhone`, `HomePhone`, `MobilePhone` | Номери телефонів контакту. |
| `InstantMessaging` | Адреса контакту для обміну миттєвими повідомленнями. |
| `CreationTime`, `LastModifiedTime` | Як у таблиці Елементи. |

### Календар (Calendar) { #calendar }

| Поле | Інтерпретація |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Як у таблиці Елементи: `Appointment` або `MeetingMessage`. |
| `StartTime`, `EndTime` | Коли зустріч починається і закінчується (UTC); для повторюваної — час першого повторення. |
| `DurationMinutes` | Її тривалість у хвилинах. |
| `Location` | Де вона відбувається. |
| `AllDay`, `Recurring` | Чи це подія на весь день і чи вона повторюється (її винятки — вбудовані елементи). |
| `BusyStatus` | `Free`, `Tentative`, `Busy`, `OutOfOffice` або `WorkingElsewhere`. |
| `ResponseStatus` | Відповідь користувача: `None`, `Organized`, `Tentative`, `Accepted`, `Declined` або `NotResponded`. |
| `Organizer` | Ім'я відправника елемента. |
| `AllAttendees`, `ToAttendees`, `CcAttendees` | Учасники так, як їх перелічує елемент. |
| `GlobalObjectID` | Ідентичність наради в різних поштових скриньках (у шістнадцятковому вигляді): та сама нарада в календарях організатора й учасників. |
| `ReminderSet`, `ReminderTime` | Чи встановлено нагадування і на який час (UTC). |
| `CreationTime`, `LastModifiedTime` | Як у таблиці Елементи. |

### Завдання (Tasks) { #tasks }

| Поле | Інтерпретація |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Як у таблиці Елементи: `Task` або `TaskRequest`. |
| `Status` | `NotStarted`, `InProgress`, `Complete`, `Waiting` або `Deferred`. |
| `PercentComplete` | Яку частину виконано, від 0 до 100. |
| `StartTime`, `DueTime` | Коли завдання починається і коли його термін виконання: дати в часовому поясі користувача, без зміщення. |
| `CompletedTime` | Коли його виконано (UTC). |
| `Complete` | Чи його виконано. |
| `Owner`, `Assigner` | Хто є його власником і хто його призначив. |
| `ReminderSet`, `ReminderTime` | Як у таблиці Календар. |
| `CreationTime`, `LastModifiedTime` | Як у таблиці Елементи. |

### Властивості (Properties) { #properties }

| Поле | Інтерпретація |
|---|---|
| `ItemID` | Ідентифікатор елемента; `store` для власних властивостей сховища; `folder:<FolderID>` для властивостей папки. |
| `Scope` | Чия це властивість: `Store`, `Folder`, `Item`, `Recipient` або `Attachment`. |
| `Index` | Індекс одержувача або вкладення; порожнє для інших. |
| `PropertyTag` | Ідентифікатор і тип властивості у вигляді `0x0037001F` (`PidTagSubject`, рядок). |
| `PropertyName` | Для іменованої властивості (ідентифікатори від `0x8000`) — її набір властивостей та ім'я: `PSETID_Address:0x8083`, `PS_PUBLIC_STRINGS:Keywords` або GUID набору, якщо це не загальновідомий набір. |
| `Type` | Тип властивості: `PtypString`, `PtypInteger32`, `PtypTime`, `PtypBinary`, `PtypMultipleString`, …. |
| `Value` | Значення: текст для рядків, ISO 8601 UTC для часу (який `-Z` не перетворює), шістнадцятковий вигляд для двійкових значень (перші 4 096 байтів, далі `…(N bytes)`), значення багатозначної властивості як список (у CSV розділені `, `), `(object)` для вбудованого елемента або даних вкладення, а для пошкодженого значення чи значення типу, який не декодується, — байти в шістнадцятковому вигляді з приміткою. |

## Автоматичний аналіз { #automatic-parsing }

`auto-parse` записує таблиці до нової категорії `Communication` як `Outlook-Files.csv`,
`Outlook-Folders.csv`, `Outlook-Items.csv`, `Outlook-Recipients.csv`, `Outlook-Attachments.csv`,
`Outlook-Contacts.csv`, `Outlook-Calendar.csv` і `Outlook-Tasks.csv` (один файл `Outlook.json` або
`Outlook.jsonl` з `-t json` чи `-t jsonl`) разом із `Communication/Outlook-logs.txt`. Він передає
аналізатору свій `--code-page` і не додає таблицю Властивості та не видобуває вкладення: для цього
запускайте `parse-file outlook`. Об'єднана хронологія `-j` містить кожен рядок, із `Comp` та
ідентифікатором `Artifact` `outlook`. `-Z` перетворює кожне поле, назва якого закінчується на `Time`,
крім `StartTime` і `DueTime` завдання.

## Додаткові відомості { #further-details }

- [Microsoft — \[MS-PST\]: Outlook Personal Folders (.pst) File Format](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]: Outlook Item (.msg) File Format](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]: Message and Attachment Object Protocol](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]: Exchange Server Protocols Master Property List](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — Personal Folder File (PFF) format](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
