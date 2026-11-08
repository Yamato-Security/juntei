# Outlook { #outlook }

## ما هي ملفات Outlook { #what-outlooks-files-are }

يحتفظ Outlook لنظام Windows بالبريد وجهات الاتصال والتقويمات والمهام في ملفات خاصة به:

- **OST** (`.ost`): النسخة غير المتصلة (offline) من صندوق بريد، التي يحتفظ بها Outlook لحسابات
  Exchange وMicrosoft 365، ولحسابات IMAP أيضًا منذ Outlook 2013. وتتكون ملفات OST في Outlook 2013
  والإصدارات الأحدث من صفحات بحجم 4 KiB وكتل مضغوطة.
- **PST** (`.pst`): مخزن شخصي: حسابات POP3، والأرشيفات (`archive.pst`)، وملفات التصدير، والمخازن التي
  ينشئها المستخدمون. وملفات PST بترميز ANSI هي ملفات Outlook 97 إلى 2002؛ وملفات PST بترميز Unicode
  هي ملفات Outlook 2003 والإصدارات الأحدث.
- **MSG** (`.msg`): عنصر واحد محفوظ كملف، عندما يُسحب خارج Outlook، أو يُحفظ بالأمر **حفظ باسم**، أو
  يصدّره برنامج آخر. والرسائل المرفقة ملفات MSG أيضًا.
- **OFT** (`.oft`): قالب Outlook، أي ملف MSG تُنشأ منه العناصر الجديدة.

يضم كل مخزن مجلدات، ويضم كل مجلد عناصره: رسائل البريد الإلكتروني، وطلبات الاجتماعات والردود عليها،
وجهات الاتصال وقوائم التوزيع، وعناصر التقويم، والمهام، والملاحظات، وإدخالات دفتر اليومية، والتقارير.
ويحتفظ كل مجلد أيضًا بعناصر مخفية، هي معلومات المجلد المقترنة (FAI): طرق العرض، والقواعد، والنماذج،
وإعدادات Outlook (عناصر `IPM.Configuration.*`، مثل قائمة الإكمال التلقائي لصندوق بريد Exchange).
ويمكن أن تكون مرفقات العنصر ملفات، أو مراجع إلى ملفات، أو كائنات OLE، أو عناصر أخرى (رسالة أُعيد
توجيهها كمرفق، واستثناءات موعد متكرر)، ولهذه العناصر مرفقاتها الخاصة.

<span class="evidence-strength">أفضل استنتاج:</span> كان المخزن أو الملف يحتوي على العنصر، بهؤلاء
المرسلين والمستلمين وبهذه الأوقات والمرفقات، وقت جمعه. ولا يقول إن المستخدم قرأه أو كتبه أو أرسله،
ولا إن مرسله هو من يقول العنصر إنه هو.

## الموقع والجمع { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

يمكن أن توجد ملفات PST في أي مكان: فالمستخدمون ينقلون الأرشيفات إلى مجلدات ومحركات أقراص ومشاركات
ملفات أخرى. ويحتفظ Outlook بمسارات مخازن كل ملف تعريف (profile) في سجل Windows الخاص بالمستخدم.
وتُنسخ المرفقات التي فتحها المستخدم من Outlook إلى مجلد داخل
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\` ‏(`Temporary Internet Files\Content.Outlook`
قبل Windows 8.1)، وقد تبقى هناك.

يُبقي Outlook مخازنه مفتوحة ما دام يعمل، وقد يبلغ حجم ملف OST عشرات الجيجابايتات. فاجمعها وOutlook
مغلق، أو من صورة قرص أو نسخة ظل لوحدة التخزين (volume shadow copy). ويقرأ Juntei كل مخزن على شكل
دفق: فلا يحتفظ أبدًا بمخزن كامل في الذاكرة.

## تحذيرات للمحلل { #analyst-cautions }

- يقرأ Juntei بتساهل: فهو يتجاوز ما هو تالف، ويقرأ ما يمكن قراءته، ويبلّغ عن الباقي بوصفه مشكلات.
  وتُطبع هذه المشكلات (أول 10 منها لكل ملف)، وتُحتسب في الملخص وفي جدول الملفات، وتُدرج في الحقل
  `Problems` للعناصر والمرفقات التي تخصها. والعنصر الذي تتعذر قراءته كليًا يبقى له صف في جدول
  العناصر، مع `ItemID` الخاص به وسبب التعذر.
- الأوقات بتوقيت UTC، بدقة 100 نانوثانية. `SubmitTime` هو الوقت الذي أرسل فيه عميل المرسل الرسالة،
  بحسب ساعة ذلك الحاسوب؛ و`DeliveryTime` هو وقت وصولها إلى صندوق البريد هذا. و`CreationTime` هو وقت
  إنشاء هذه النسخة من العنصر: عند تسليمه إلى صندوق البريد أو إنشائه فيه، أو مزامنته إلى OST، أو نسخه
  أو استيراده إلى PST، أو حفظه كملف MSG. وهو ليس وقت كتابة الرسالة. و`LastModifiedTime` هو آخر وقت
  تغيّر فيه العنصر في المخزن.
- `StartTime` و`DueTime` الخاصان بالمهمة تاريخان بالمنطقة الزمنية للمستخدم، يخزنهما Outlook على أنهما
  منتصف الليل. ويكتبهما Juntei من دون إزاحة (`2017-02-19T00:00:00`)، ويتركهما `-Z` كما هما.
- يكتب Outlook منتصف ليل 1 يناير 4501 للقيمة **بلا** (None) في بعض التواريخ، مثل تاريخ استحقاق مهمة أو
  وقت تذكير. ويكتبه Juntei كما خُزّن: `4501-01-01T00:00:00Z`، ومن دون `Z` في `StartTime` و`DueTime`
  الخاصين بالمهمة.
- `Read` هو علامة القراءة وقت جمع المخزن: أي أن شخصًا ما، أو قاعدة ما، علّم العنصر كمقروء. ولا يقول
  من فعل ذلك ولا متى.
- أسماء المرسل والمستلم وعناوينهما هي ما سجّله البرنامج المرسِل وخوادم البريد، ويمكن تزويرها. وعناوين
  `EX` هي أسماء مميّزة (distinguished names) في Exchange ‏(`/o=…/ou=…/cn=Recipients/cn=…`)؛ ويوجد
  عنوان SMTP في `SenderSMTP-Address` و`SMTP-Address` عندما يخزنه العنصر إلى جانب ذلك. وللتحقق من مصدر
  رسالة، اقرأ ترويسات الإنترنت الخاصة بها: `0x007D001F` أو `0x007D001E`
  ‏(`PidTagTransportMessageHeaders`) في جدول الخصائص.
- تبقى العناصر المحذوفة في Outlook في **العناصر المحذوفة** (Deleted Items) إلى أن يُفرَّغ هذا المجلد،
  وتُقرأ كأي عنصر آخر. أما العناصر التي لم تعد في جدول أي مجلد، والتي قد يحتفظ بها المخزن إلى أن يعيد
  استخدام مساحتها، فلا تُقرأ بعد.
- `ItemID` هو معرّف العقدة (node id) للرسالة في ملف PST أو OST الخاص بها: وقد يكون للعنصر نفسه في نسخة
  أخرى من المخزن معرّف آخر. ومعرّف العنصر المضمّن هو معرّف العنصر الأب وفهرس المرفق (`2097348/0`،
  `0/0/1`)؛ ومعرّف عنصر ملف `.msg` أو `.oft` هو `0`.
- يأتي `User` من المسار فقط (المجلد الذي يلي `Users`)، ويضيع عند نسخ الملف إلى مكان آخر. وقد يعود ملف
  PST إلى أي مستخدم.
- يُقرأ النص غير المرمّز بـ Unicode بصفحة الترميز الخاصة بالعنصر (صفحته هو، أو صفحة إعداداته
  المحلية، أو صفحة ترميز الإنترنت الخاصة به). ويحدد `--code-page` صفحة الترميز للعناصر التي لا تحدد
  صفحتها: صفحة ترميز ANSI الخاصة بالنظام الذي كتبها، و`932` لنظام ياباني.
- كلمة مرور ملف PST لا تشفّره: فـ Juntei يقرأ ملفات PST المحمية بكلمة مرور كغيرها. ولا يفعل
  `Encryption` الخاص بالمخزن (`Permute`، أي «التشفير القابل للضغط» (compressible encryption) في
  Outlook، أو `Cyclic`) سوى تمويه كتله. والرسائل المشفرة بـ S/MIME تحتفظ بنصها ومرفقاتها في مرفق مشفّر
  (`smime.p7m`)، لا يُفك تشفيره.
- كثيرًا ما تكون المرفقات برمجيات خبيثة. ويكتب `--extract-attachments` بياناتها في ملفات مسماة بقيمة
  SHA-256 الخاصة بها، من دون إذن تنفيذ؛ فلا تشغّلها أبدًا، واحتفظ بها على نظام تحليل معزول.

### ما لا يُقرأ بعد { #not-read-yet }

دفاتر العناوين غير المتصلة (OAB)، ودفق الإكمال التلقائي (`Stream_Autocomplete_*.dat`) وملفات `.nk2`،
وإعدادات الإرسال/الاستلام (`.srs`)، وذاكرات التخزين المؤقت لإعدادات Outlook ‏(RoamCache، Autodiscover)،
وقواعد بيانات SQLite الخاصة بـ Outlook، والرسائل التي لم تعد في جدول أي مجلد.

## التحليل باستخدام Juntei { #parse-with-juntei }

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

يبحث `-d` عن الملفات المنتهية بـ `.pst` و`.ost` و`.msg` و`.oft`، من دون تمييز بين الأحرف الكبيرة
والصغيرة. وأيًّا كانت طريقة تحديد الملف، فإن بايتاته الأولى هي التي تحدد نوعه: `!BDN` لملف PST أو OST،
وتوقيع الملف المركّب (compound file) لملف MSG أو OFT. وأي شيء آخر يُبلَّغ عنه بوصفه ملفًا تعذر تحليله.
وتكون حالة الخروج 1 عندما لا يمكن قراءة أي ملف.

يكتب `--extract-attachments` بيانات كل مرفق مرة واحدة، باسم `<SHA-256>` مع امتداد المرفق إذا كان
مكونًا من 1 إلى 10 أحرف وأرقام ASCII ‏(`0c7d…1311.jpg`)؛ ولا يُستخدم في المسار أي اسم مأخوذ من
عنصر. ويجب أن يكون المجلد فارغًا ما لم يُعطَ `-C`؛ ومع `-C` يُستبدل الملف الذي يحمل الاسم نفسه ويحتوي
على شيء آخر.

## جداول الإخراج { #output-tables }

ينشئ CSV ثمانية ملفات، وتسعة مع `--properties`، لا يحتوي كل منها إلا على صف الترويسة عندما لا تكون
فيه صفوف. وتحمل صفوف JSON وJSONL الحقل `Table` ‏(`Files` أو `Folders` أو `Items` أو `Recipients` أو
`Attachments` أو `Contacts` أو `Calendar` أو `Tasks` أو `Properties`) بعد `SourceFile`. وتأتي صفوف
العنصر بعده، وصفوف عناصره المضمّنة بعدها؛ ويأتي صف الملف في جدول الملفات بعد سائر صفوفه.

| ملف CSV | المحتوى |
|---|---|
| `Outlook-Files.csv` | صف لكل ملف مقروء: المخزن أو الملف وما يحتويه. |
| `Outlook-Folders.csv` | مجلدات ملفات PST وOST. |
| `Outlook-Items.csv` | كل العناصر: رسائل المجلدات، وعناصرها المخفية، وعناصر ملفات `.msg` و`.oft`، والعناصر المضمّنة في مرفقاتها. |
| `Outlook-Recipients.csv` | مستلمو العناصر. |
| `Outlook-Attachments.csv` | مرفقات العناصر. |
| `Outlook-Contacts.csv` | حقول جهات الاتصال وقوائم التوزيع. |
| `Outlook-Calendar.csv` | حقول عناصر التقويم ورسائل الاجتماعات. |
| `Outlook-Tasks.csv` | حقول المهام وطلبات المهام. |
| `Outlook-Properties.csv` | مع `--properties`: كل خصائص المخازن والمجلدات والعناصر والمستلمين والمرفقات، كما خُزّنت. |

## الحقول { #fields }

يبدأ كل جدول بهذه الحقول:

| الحقل | التفسير |
|---|---|
| `SourceFile` | الملف الذي قُرئ منه الصف. |
| `User` | مستخدم Windows الذي يقع المسار ضمن ملف تعريفه (المجلد الذي يلي `Users`). |

### الملفات (Files) { #files }

| الحقل | التفسير |
|---|---|
| `FileType` | `PST` أو `OST` أو `MSG` أو `OFT`، أو `CFB` لملف مركّب قُرئ كعنصر ولا يكون معرّف الفئة (class ID) فيه معرّف رسالة ولا معرّف قالب (تتركه برامج كثيرة غير Outlook فارغًا). |
| `Format` | بنية المخزن: `ANSI` ‏(Outlook 97 إلى 2002)، أو `Unicode` ‏(Outlook 2003 والإصدارات الأحدث)، أو `Unicode 4K` (ملفات OST في Outlook 2013 والإصدارات الأحدث)؛ ولملف MSG أو OFT، إصدار الملف المركّب (`CFB v3`، `CFB v4`). |
| `Version`، `ClientVersion` | `wVer` و`wVerClient` في ترويسة المخزن. |
| `Encryption` | كيفية ترميز كتل المخزن: `None` أو `Permute` أو `Cyclic`. |
| `DisplayName` | اسم المخزن (`Personal Folders`، أو عنوان بريد إلكتروني). |
| `RecordKey` | هوية المخزن (بالنظام الست عشري)، التي تتضمنها معرّفات الإدخال (entry ids) لعناصره. |
| `Folders` | مجلدات المخزن. |
| `Items` | رسائل مجلداته، أو 1 لملف MSG أو OFT. |
| `HiddenItems` | العناصر المخفية (معلومات المجلد المقترنة) في مجلداته. |
| `EmbeddedItems` | العناصر المضمّنة في المرفقات، على جميع المستويات. |
| `Attachments` | مرفقات كل تلك العناصر. |
| `Problems` | المشكلات التي عُثر عليها في الملف. |

### المجلدات (Folders) { #folders }

| الحقل | التفسير |
|---|---|
| `FolderID`، `ParentID` | معرّف العقدة للمجلد، ومعرّف مجلده الأب (فارغ للمجلد الجذر). |
| `FolderPath` | أسماء المجلدات من الجذر نزولًا: `/Top of Personal Folders/Inbox`، `/Root - Mailbox/IPM_SUBTREE/Sent Items`. |
| `Name` | اسم المجلد. |
| `ContainerClass` | ما يحتويه المجلد: `IPF.Note` (البريد)، `IPF.Appointment`، `IPF.Contact`، `IPF.Task`، `IPF.Imap`، …. |
| `ContentCount`، `UnreadCount` | الرسائل، والرسائل غير المقروءة، التي يقول المجلد إنها فيه. |
| `Messages`، `HiddenItems`، `Subfolders` | صفوف جدول محتوياته، وجدول محتوياته المقترنة، وجدول تسلسله الهرمي: أي ما يقرؤه Juntei. والفرق عن `ContentCount` يستحق الفحص. |
| `CreationTime`، `LastModifiedTime` | وقت إنشاء المجلد وآخر تغيير له (UTC)، عندما يحتفظ المخزن بهما. |
| `Problem` | سبب تعذر فتح المجلد؛ وعندئذ لا تُقرأ مجلداته الفرعية. |

### العناصر (Items) { #items }

| الحقل | التفسير |
|---|---|
| `FolderPath` | مجلد العنصر؛ فارغ في ملف MSG أو OFT. |
| `ItemID` | معرّف العنصر: معرّف العقدة الخاص به في PST أو OST، و`0` في ملف MSG أو OFT، ومعرّف العنصر الأب وفهرس المرفق للعنصر المضمّن (`2097348/0`). |
| `Hidden` | ما إذا كان العنصر من معلومات المجلد المقترنة، أي عنصرًا مخفيًا. |
| `EmbeddedIn` | معرّف العنصر الذي أُرفق به هذا العنصر؛ فارغ للعناصر الأخرى. |
| `ItemType` | نوع العنصر بحسب فئة الرسالة: `Mail`، `MeetingMessage`، `Appointment`، `Contact`، `DistributionList`، `Task`، `TaskRequest`، `StickyNote`، `Journal`، `Post`، `Report`، `Configuration` أو `Other`. |
| `MessageClass` | فئة الرسالة كما خُزّنت (`IPM.Note`، `IPM.Note.SMIME`، `IPM.Schedule.Meeting.Request`، `REPORT.IPM.Note.NDR`، …). |
| `SubmitTime` | الوقت الذي أرسل فيه عميل المرسل الرسالة (UTC، بحسب ساعة المرسل). |
| `DeliveryTime` | وقت تسليمها إلى صندوق البريد هذا (UTC). |
| `CreationTime`، `LastModifiedTime` | وقت إنشاء هذه النسخة في المخزن أو الملف، وآخر تغيير لها (UTC). |
| `Subject` | الموضوع، مع بادئته (`RE: `). |
| `SenderName`، `SenderAddressType`، `SenderAddress` | من أرسلها: الاسم، ونوع العنوان (`SMTP`، `EX`)، والعنوان من ذلك النوع. |
| `SenderSMTP-Address` | عنوان SMTP للمرسل، عندما يُخزَّن إلى جانب عنوان من نوع آخر. |
| `SentRepresentingName`، `SentRepresentingAddress` | من أُرسلت نيابةً عنه (مفوَّض يرسل نيابةً عن مدير؛ صندوق بريد مشترك)، مع عنوان SMTP الخاص به عند تخزينه، وإلا فالعنوان من نوعه. |
| `To`، `Cc`، `Bcc` | أسماء المستلمين كما يعرضها العنصر (راجع جدول المستلمين لمعرفة عناوينهم). ولا يوجد `Bcc` إلا في نسخة المرسل. |
| `InternetMessageID`، `InReplyTo` | قيمة `Message-ID` للرسالة، وقيمته للرسالة التي ترد عليها. |
| `ConversationTopic` | الموضوع من دون بادئته، وتشترك فيه رسائل المحادثة الواحدة. |
| `Importance`، `Sensitivity` | `Low` أو `Normal` أو `High`؛ و`Normal` أو `Personal` أو `Private` أو `Confidential`. |
| `Read` | ما إذا كان العنصر معلَّمًا كمقروء. |
| `Flags` | علامات الرسالة (`PidTagMessageFlags`، بالنظام الست عشري): `0x1` مقروءة، `0x2` غير معدّلة، `0x4` مُقدَّمة للإرسال، `0x8` غير مُرسلة (مسودة)، `0x10` لها مرفقات، `0x20` مني، `0x40` مخفية، `0x80` أُعيد إرسالها، `0x400` قُرئت من قبل، `0x2000` من الإنترنت. |
| `Size` | حجم العنصر في المخزن، مع مرفقاته. |
| `Recipients`، `Attachments` | عدد مستلميه ومرفقاته. |
| `Categories` | تصنيفاته، تفصل بينها `;`. |
| `BodyPreview` | بداية نصه العادي، مع دمج المسافات البيضاء، بحد أقصى 256 حرفًا؛ وهو فارغ عندما لا يكون له إلا نص HTML أو RTF. |
| `Problems` | ما تعذرت قراءته في العنصر (بحد أقصى 10، ثم عدد ما تبقى). |

### المستلمون (Recipients) { #recipients }

| الحقل | التفسير |
|---|---|
| `ItemID` | معرّف العنصر. |
| `Index` | صف المستلم في جدول مستلمي العنصر، بدءًا من 0. |
| `RecipientType` | `To` أو `Cc` أو `Bcc` أو `Originator`، أو القيمة كما خُزّنت (بالنظام الست عشري). |
| `Name`، `AddressType`، `Address` | اسم المستلم، ونوع عنوانه، وعنوانه. |
| `SMTP-Address` | عنوان SMTP، عندما يُخزَّن إلى جانب عنوان من نوع آخر. |

### المرفقات (Attachments) { #attachments }

| الحقل | التفسير |
|---|---|
| `ItemID`، `Index` | معرّف العنصر، وصف المرفق في جدول مرفقاته، بدءًا من 0. |
| `Method` | `ByValue` (ملف)، `ByReference`، `ByReferenceResolve`، `ByReferenceOnly` (مسار إلى ملف)، `EmbeddedMessage` (عنصر)، `OLE` (كائن OLE)، `ByWebReference`، أو الرقم. |
| `Filename` | اسم المرفق: اسم ملفه الطويل، أو اسمه القصير، أو اسم العرض الخاص به. |
| `Extension` | امتداده كما خُزّن (`.pdf`)، وإلا فامتداد اسمه. |
| `MimeType`، `ContentID` | نوع MIME الخاص به، ومعرّف المحتوى الذي يشير إليه به نص HTML. |
| `Inline` | ما إذا كان نص الرسالة يعرضه (صورة في نص HTML) بدلًا من إدراجه في قائمة المرفقات. |
| `Hidden` | ما إذا كان Outlook يخفيه من قائمة المرفقات. |
| `Size` | حجمه كما يعطيه المخزن، مع خصائصه. |
| `DataSize`، `SHA256` | حجم بياناته وقيمة SHA-256 لها: الملف، أو الملف المركّب لكائن OLE في PST أو OST. فارغان للعنصر المضمّن وللمرفق بالمرجع. |
| `Pathname` | مسار المرفق بالمرجع. |
| `CreationTime`، `LastModifiedTime` | أوقات المرفق (UTC): وهي غالبًا أوقات الملف الذي أُنشئ منه. |
| `EmbeddedSubject` | موضوع العنصر المضمّن، الذي تحمل صفوفه الخاصة `ItemID` بالصيغة `<ItemID>/<Index>`. |
| `ExtractedFile` | مع `--extract-attachments`، الملف الذي كُتبت فيه بياناته، داخل ذلك المجلد. |
| `Problems` | ما تعذرت قراءته في المرفق، أو كتابته. |

### جهات الاتصال (Contacts) { #contacts }

| الحقل | التفسير |
|---|---|
| `FolderPath`، `ItemID`، `ItemType` | كما في جدول العناصر: `Contact` أو `DistributionList`. |
| `DisplayName`، `GivenName`، `Surname`، `Nickname` | أسماء جهة الاتصال. |
| `FileUnder` | الاسم الذي تُحفظ جهة الاتصال تحته. |
| `Company`، `Title`، `Department` | مكان عمل جهة الاتصال. |
| `Email1Name`، `Email1Address`، `Email2Name`، `Email2Address`، `Email3Name`، `Email3Address` | عناوين البريد الإلكتروني الثلاثة لجهة الاتصال، كل منها مع اسم العرض الخاص به. |
| `BusinessPhone`، `HomePhone`، `MobilePhone` | أرقام هواتف جهة الاتصال. |
| `InstantMessaging` | عنوان المراسلة الفورية لجهة الاتصال. |
| `CreationTime`، `LastModifiedTime` | كما في جدول العناصر. |

### التقويم (Calendar) { #calendar }

| الحقل | التفسير |
|---|---|
| `FolderPath`، `ItemID`، `ItemType`، `Subject` | كما في جدول العناصر: `Appointment` أو `MeetingMessage`. |
| `StartTime`، `EndTime` | وقت بدء الموعد وانتهائه (UTC)؛ وللموعد المتكرر، وقت أول تكرار. |
| `DurationMinutes` | مدته بالدقائق. |
| `Location` | مكانه. |
| `AllDay`، `Recurring` | ما إذا كان حدثًا ليوم كامل، وما إذا كان متكررًا (استثناءاته عناصر مضمّنة). |
| `BusyStatus` | `Free` أو `Tentative` أو `Busy` أو `OutOfOffice` أو `WorkingElsewhere`. |
| `ResponseStatus` | رد المستخدم: `None` أو `Organized` أو `Tentative` أو `Accepted` أو `Declined` أو `NotResponded`. |
| `Organizer` | اسم مرسل العنصر. |
| `AllAttendees`، `ToAttendees`، `CcAttendees` | الحضور، كما يدرجهم العنصر. |
| `GlobalObjectID` | هوية الاجتماع عبر صناديق البريد (بالنظام الست عشري): الاجتماع نفسه في تقويمات المنظِّم والحضور. |
| `ReminderSet`، `ReminderTime` | ما إذا كان التذكير مضبوطًا، والوقت المحدد له (UTC). |
| `CreationTime`، `LastModifiedTime` | كما في جدول العناصر. |

### المهام (Tasks) { #tasks }

| الحقل | التفسير |
|---|---|
| `FolderPath`، `ItemID`، `ItemType`، `Subject` | كما في جدول العناصر: `Task` أو `TaskRequest`. |
| `Status` | `NotStarted` أو `InProgress` أو `Complete` أو `Waiting` أو `Deferred`. |
| `PercentComplete` | مقدار ما أُنجز منها، من 0 إلى 100. |
| `StartTime`، `DueTime` | موعد بدئها وموعد استحقاقها: تاريخان بالمنطقة الزمنية للمستخدم، من دون إزاحة. |
| `CompletedTime` | وقت إكمالها (UTC). |
| `Complete` | ما إذا كانت مكتملة. |
| `Owner`، `Assigner` | مالكها، ومن أسندها. |
| `ReminderSet`، `ReminderTime` | كما في جدول التقويم. |
| `CreationTime`، `LastModifiedTime` | كما في جدول العناصر. |

### الخصائص (Properties) { #properties }

| الحقل | التفسير |
|---|---|
| `ItemID` | معرّف العنصر؛ و`store` لخصائص المخزن نفسه؛ و`folder:<FolderID>` لخصائص مجلد. |
| `Scope` | لمن تعود الخاصية: `Store` أو `Folder` أو `Item` أو `Recipient` أو `Attachment`. |
| `Index` | فهرس المستلم أو المرفق؛ فارغ لغيرهما. |
| `PropertyTag` | معرّف الخاصية ونوعها، بالصيغة `0x0037001F` ‏(`PidTagSubject`، سلسلة نصية). |
| `PropertyName` | للخاصية المسماة (المعرّفات من `0x8000` فصاعدًا)، مجموعة خصائصها واسمها: `PSETID_Address:0x8083`، `PS_PUBLIC_STRINGS:Keywords`، أو معرّف GUID للمجموعة عندما لا تكون من المجموعات المعروفة. |
| `Type` | نوع الخاصية: `PtypString`، `PtypInteger32`، `PtypTime`، `PtypBinary`، `PtypMultipleString`، …. |
| `Value` | القيمة: نص للسلاسل النصية، وISO 8601 بتوقيت UTC للأوقات (لا يحوّلها `-Z`)، وبالنظام الست عشري للقيم الثنائية (أول 4,096 بايت، ثم `…(N bytes)`)، وقيم الخاصية متعددة القيم على شكل قائمة (تفصل بينها `, ` في CSV)، و`(object)` للعنصر المضمّن أو لبيانات المرفق، والبايتات بالنظام الست عشري مع ملاحظة للقيمة التالفة أو التي لا يُفك ترميز نوعها. |

## التحليل التلقائي { #automatic-parsing }

يكتب `auto-parse` الجداول في الفئة الجديدة `Communication` بالأسماء `Outlook-Files.csv`
و`Outlook-Folders.csv` و`Outlook-Items.csv` و`Outlook-Recipients.csv` و`Outlook-Attachments.csv`
و`Outlook-Contacts.csv` و`Outlook-Calendar.csv` و`Outlook-Tasks.csv` (أو ملفًا واحدًا هو
`Outlook.json` أو `Outlook.jsonl` مع `-t json` أو `-t jsonl`)، مع `Communication/Outlook-logs.txt`.
ويمرّر إلى المحلل قيمة `--code-page` الخاصة به، ولا يضيف جدول الخصائص ولا يستخرج المرفقات: شغّل
`parse-file outlook` لذلك. ويتضمن الجدول الزمني المدمج `-j` كل صف، مع `Comp` ومعرّف `Artifact` ذي
القيمة `outlook`. ويحوّل `-Z` كل حقل ينتهي اسمه بـ `Time`، باستثناء `StartTime` و`DueTime` الخاصين
بالمهمة.

## مزيد من التفاصيل { #further-details }

- [Microsoft — \[MS-PST\]: Outlook Personal Folders (.pst) File Format](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]: Outlook Item (.msg) File Format](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]: Message and Attachment Object Protocol](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]: Exchange Server Protocols Master Property List](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — Personal Folder File (PFF) format](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
