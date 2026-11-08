# متصفحات الويب { #web-browsers }

## ما هي قواعد بيانات المتصفحات { #what-the-browser-databases-are }

تحتفظ المتصفحات بسجل التصفح والتنزيلات وملفات تعريف الارتباط (cookies) وإدخالات النماذج في قواعد بيانات
SQLite داخل ملف التعريف (profile) الخاص بكل مستخدم. وتستخدم المتصفحات المبنية على Chromium ‏(Chrome وEdge
وBrave وOpera وVivaldi) والمتصفحات المبنية على Firefox ملفاتها وجداولها الخاصة، لكنها تسجل الأنواع نفسها
من النشاط:

- **سجل التصفح**: الصفحات التي زيرت، وكل زيارة مع وقتها، وكيفية وصول المتصفح إلى الصفحة (بكتابة العنوان،
  أو عبر رابط، أو إعادة توجيه، أو إشارة مرجعية)، ومدة بقاء الصفحة في الواجهة في Chromium.
- **التنزيلات**: عنوان URL الذي جاء منه الملف (وعمليات إعادة التوجيه في الطريق إليه)، ومكان حفظه، وحجمه،
  ووقتا بدء التنزيل وانتهائه، وما إذا كان قد اكتمل.
- **ملفات تعريف الارتباط**: المواقع التي عيّنتها، ووقت تعيينها، وآخر إرسال لها، ووقت انتهاء صلاحيتها.
- **سجل النماذج**: القيم المكتوبة في نماذج الويب ومربعات البحث، وعدد مرات كتابتها، وأول مرة وآخر مرة.

وغالبًا ما يكون نشاط المتصفح أكثر السجلات مباشرةً لما فعله المستخدم: صفحة التصيد التي تبعها، والأداة
التي نزّلها، وبريد الويب الذي استخدمه، وعبارات البحث التي كتبها.

<span class="evidence-strength">أفضل استنتاج:</span> سجّل ملف تعريف المتصفح زيارة لعنوان URL في الوقت
المسجل، أو حفظ التنزيل في المسار المسجل؛ ولا تقول قاعدة البيانات من كان يجلس أمام لوحة المفاتيح.

## الموقع والجمع { #location-and-collection }

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

تكون قيمة `<Profile>` هي `Default` أو `Profile 1` أو `Profile 2` وهكذا. ونقل Chromium الملف `Cookies`
إلى `Network` في الإصدار 96؛ وتحتفظ الإصدارات الأقدم به بجوار `History`. واحتفظ Firefox حتى الإصدار 25
بتنزيلاته في `downloads.sqlite`؛ وتحتفظ بها الإصدارات اللاحقة في `places.sqlite`.

اجمع كل ملفات ملف التعريف، واجمع كل قاعدة بيانات مع الملفات المجاورة لها التي تشاركها اسمها:
`places.sqlite-wal` و`cookies.sqlite-wal` و`formhistory.sqlite-wal` و`History-journal`.

!!! warning "سجلات الكتابة المسبقة"
    يحتفظ Firefox بقواعد بياناته في وضع سجل الكتابة المسبقة (write-ahead log): فلا توجد أحدث التغييرات
    إلا في ملف `-wal` إلى أن ينسخها المتصفح إلى قاعدة البيانات (نقطة تحقق، checkpoint). ويقرأ Juntei كل
    قاعدة بيانات مع ملف `-wal` الخاص بها، كما يفعل المتصفح. ومن دونه، تكون الزيارات وملفات تعريف الارتباط
    وإدخالات النماذج الحديثة مفقودة، وتظل الإدخالات المحذوفة في السجل تُقرأ على أنها نشطة.

## الإدخالات المحذوفة { #deleted-entries }

عندما يحذف المتصفح السجل، تخرج الصفوف من الجداول، لكن سجلاتها قد تبقى في المساحة الحرة لقاعدة البيانات،
أو في قائمة صفحاتها الحرة (freelist)، أو في إصدارات أقدم من صفحات قاعدة البيانات: في ملف `-wal`، أو في
ملف قاعدة البيانات نفسه عندما يحمل ملف `-wal` إصدارات أحدث من تلك الصفحات. ويستعيد Juntei منها الزيارات
والصفحات والتنزيلات وإدخالات النماذج المحذوفة، على شكل صفوف قيمة `Status` فيها هي `Deleted`:

- يبيّن `Confidence` مدى اليقين من مطابقة السجل لجدوله: `High` لسجل كامل كل قيمة فيه من نوع عمودها،
  و`Medium` أو `Low` للسجلات التالفة أو للسجلات التي تناسب عدة جداول.
- يبيّن `RecoveredFrom` مكان السجل: منطقة الصفحة (خلية، أو كتلة حرة (freeblock)، أو مساحة غير مخصصة، أو
  صفحة حرة)، وأي صفحة وأي إصدار منها، وإزاحة السجل داخل الصفحة. وتُعدّ إطارات WAL بدءًا من 0.
- يسرد `LostColumns` الأعمدة التي لم يعد السجل التالف يحتويها: وتكون حقولها فارغة.

تحذف المتصفحات باستخدام الخيار `secure_delete` في SQLite: إذ تُصفَّر المساحة المحرَّرة في قاعدة البيانات.
ولذلك يكون ما يمكن استعادته غالبًا في إصدارات الصفحات الأقدم: تلك التي ما زال ملف `-wal` يحتفظ بها، والنسخ
الموجودة في ملف قاعدة البيانات نفسه من الصفحات التي استبدلها ملف `-wal`، حتى نقطة التحقق التالية. انسخ
ملف `-wal` مع قاعدة بياناته، وافعل ذلك قبل أن يعمل المتصفح مجددًا.

## تحذيرات للمحلل { #analyst-cautions }

- الأوقات بتوقيت UTC. ويسجلها Chromium وFirefox بدقة الميكروثانية؛ أما سجل النماذج في Chromium فبدقة
  الثانية.
- لا تعني الزيارة أن المستخدم كتب أو نقر: فعمليات إعادة التوجيه والإطارات والصفحات التي فتحتها برامج أخرى
  زيارات أيضًا. اقرأ `Transition` و`Typed` و`Hidden`.
- يشفّر Chromium قيم ملفات تعريف الارتباط: فيكون `Value` فارغًا لها. ولا يفك Juntei هذا التشفير.
- يأتي صف سجل النماذج الذي حالته `Removed` من الجدول `moz_deleted_formhistory` في Firefox، الذي يملؤه
  Firefox لنظام Android لأغراض المزامنة (Sync). ويتركه Firefox لسطح المكتب فارغًا عادةً.
- قد يكون الإدخال المحذوف ذو الثقة `Low` جزءًا من سجل آخر: فقارنه بالصفوف النشطة وبالأدلة الأخرى.
- لا يعني غياب الصفوف المحذوفة أنه لم يُحذف شيء: فالمساحة المصفَّرة ونقطة التحقق لا تتركان شيئًا
  للاستعادة. ولا تُستعاد ملفات تعريف الارتباط المحذوفة بعد.
- تأتي قيم `Browser` و`User` و`Profile` من مسار قاعدة البيانات: فملف التعريف المنسوخ إلى مكان آخر لا يحتفظ
  إلا بما يقوله مساره. ومن دون مجلد متصفح معروف، تكون قيمة `Browser` هي `Chromium-based` أو
  `Firefox-based`.
- لا تُقرأ سجلات التراجع (`-journal`)، ولا الملف `WebCacheV01.dat` الخاص بـ Internet Explorer وEdge القديم.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

يبحث `-d` عن الملفات بأسمائها ويتعرّف على كل منها من جداوله: فيُتخطّى ملف `History` الخاص ببرنامج آخر.
ويقرأ `-f` قاعدة البيانات أيًّا كان اسمها.

## جداول الإخراج { #output-tables }

ينشئ CSV أربعة ملفات، لا يحتوي كل منها إلا على صف الترويسة عندما لا تكون فيه صفوف. وتحمل صفوف JSON وJSONL
الحقل `Table` ‏(`History` أو `Downloads` أو `Cookies` أو `FormHistory`) بعد `SourceFile`.

| ملف CSV | المحتوى |
|---|---|
| `Browser-History.csv` | الزيارات، النشطة والمحذوفة؛ والصفحات المحذوفة التي لا يطابق عنوان URL الخاص بها أي صفحة نشطة. |
| `Browser-Downloads.csv` | التنزيلات، النشطة والمحذوفة. |
| `Browser-Cookies.csv` | ملفات تعريف الارتباط. |
| `Browser-FormHistory.csv` | إدخالات النماذج، النشطة والمحذوفة؛ وسجلات الإزالة في Firefox. |

## الحقول { #fields }

يبدأ كل جدول بهذه الحقول:

| الحقل | التفسير |
|---|---|
| `SourceFile` | قاعدة البيانات التي قُرئ منها الصف. |
| `User` | مستخدم Windows الذي يقع المسار ضمن ملف تعريفه (المجلد الذي يلي `Users`). |
| `Browser` | المتصفح الذي يسميه المسار: `Chrome`، `Edge`، `Brave`، `Opera`، `Vivaldi`، `Firefox` … |
| `Profile` | ملف تعريف المتصفح: مجلد قاعدة البيانات. |
| `Status` | `Live`، أو `Deleted` (مستعاد)، أو `Removed` (سجل إزالة في Firefox). |

### سجل التصفح { #history }

| الحقل | التفسير |
|---|---|
| `RecordType` | `Visit`، أو `Page` لصفحة محذوفة ربما اختفت زياراتها أيضًا. |
| `VisitTime` | وقت بدء الزيارة (UTC). |
| `URL`، `Title` | الصفحة وعنوانها، كما شوهدا آخر مرة. |
| `Transition` | كيفية وصول المتصفح إلى الصفحة. في Chromium: النوع الأساسي ثم مؤهلاته (`LINK`، `TYPED`، `AUTO_BOOKMARK`، `FORM_SUBMIT`، `RELOAD`، … مع `CHAIN_START`، `SERVER_REDIRECT`، `FROM_ADDRESS_BAR`، …)؛ وفي Firefox: `LINK`، `TYPED`، `BOOKMARK`، `EMBED`، `REDIRECT_PERMANENT`، `REDIRECT_TEMPORARY`، `DOWNLOAD`، `FRAMED_LINK`، `RELOAD`. |
| `VisitDuration` | عدد الثواني التي بقيت فيها الصفحة في الواجهة (Chromium؛ 0 عندما لا تُقاس). |
| `VisitCount` | إجمالي الزيارات إلى الصفحة. |
| `Typed`، `TypedCount` | ما إذا كان عنوان URL قد كُتب يومًا، وكم مرة (Chromium). |
| `Hidden` | مخفي من قائمة السجل: الإطارات الفرعية، ومصادر إعادة التوجيه، والأخطاء. |
| `Frecency` | ترتيب Firefox للصفحة في اقتراحات شريط العناوين. |
| `VisitID`، `FromVisitID` | معرّف صف الزيارة، ومعرّف صف الزيارة التي جاءت منها (الصفحة المُحيلة، أو مصدر إعادة التوجيه). |
| `LastVisitTime` | آخر زيارة لصفحة محذوفة (UTC). |
| `PageSource` | مصدر عنوان URL والعنوان الخاصين بزيارة محذوفة: `Live` (صفحة نشطة)، أو `Recovered` (صفحة محذوفة)، أو `NotFound`. |
| `Confidence`، `RecoveredFrom`، `LostColumns` | كيفية استعادة الإدخال المحذوف؛ راجع [الإدخالات المحذوفة](#deleted-entries). |

### التنزيلات { #downloads }

| الحقل | التفسير |
|---|---|
| `StartTime`، `EndTime` | وقتا بدء التنزيل وانتهائه (UTC). |
| `URL`، `RedirectChain` | مصدر الملف، وكل عنوان URL في الطريق إليه (Chromium؛ ويفصل بينها ` -> ` في CSV). |
| `TargetPath`، `CurrentPath` | مكان حفظ الملف (معرّف URI بصيغة `file://` في Firefox)، والمكان الذي كُتب فيه أثناء التنزيل. |
| `ReceivedBytes`، `TotalBytes` | البايتات المستلمة، وحجم الملف. |
| `State` | `Complete` أو `InProgress` أو `Cancelled` أو `Interrupted` أو `Paused` أو `Blocked` أو `Dirty` (محظور بوصفه برمجية خبيثة). |
| `DangerType` | حكم Chromium على الملف: 0 غير خطير، 1 نوع ملف خطير، 2 عنوان URL خطير، 3 محتوى خطير، 4 قد يكون خطيرًا، 5 غير شائع، 6 احتفظ به المستخدم، 7 مضيف خطير، 8 قد يكون غير مرغوب فيه، … |
| `InterruptReason` | سبب توقف تنزيل مُقاطَع (Chromium؛ 0 عند عدم وجود سبب). |
| `Referrer`، `TabURL` | الصفحة المُحيلة، والصفحة الموجودة في علامة التبويب التي بدأت التنزيل. |
| `MimeType`، `Opened` | نوع الملف كما قدّمه الخادم، وما إذا فُتح من المتصفح (Chromium). |
| `FileDeletedByUser` | ما إذا حذف المستخدم الملف المنزَّل من القرص عبر قائمة تنزيلات المتصفح (Firefox). |
| `DownloadID` | معرّف صف التنزيل. |

### ملفات تعريف الارتباط { #cookies }

| الحقل | التفسير |
|---|---|
| `Created`، `LastAccessed`، `Expires` | وقت تعيين ملف تعريف الارتباط، وآخر إرسال له، وانتهاء صلاحيته (UTC؛ ويكون `Expires` فارغًا لملف تعريف ارتباط الجلسة). |
| `Host`، `Name`، `Path` | الموقع الذي يخصه ملف تعريف الارتباط، واسمه، ومساره. |
| `Value` | القيمة كما خُزّنت: فارغة عندما يشفّرها Chromium. |
| `Secure`، `HttpOnly`، `Persistent` | يُرسل عبر HTTPS فقط، ومخفي عن البرامج النصية، ويُحتفظ به عبر الجلسات (Chromium). |
| `GoogleAnalytics` | الحقول المفكوك ترميزها لملف تعريف ارتباط Google Analytics ‏(`__utma`، `__utmb`، `__utmt`، `__utmz`): أوقات الزيارات، والجلسات، ومصدر الحملة وعبارات البحث. |
| `RowID` | معرّف صف ملف تعريف الارتباط. |

### سجل النماذج { #form-history }

| الحقل | التفسير |
|---|---|
| `FirstUsed`، `LastUsed` | وقتا إدخال القيمة أول مرة وآخر مرة (UTC). |
| `RemovedTime` | الوقت الذي يذكر سجل إزالة في Firefox أن الإدخال أُزيل فيه (UTC). |
| `FieldName`، `Value` | حقل النموذج (`email`، `q`، شريط البحث في Firefox: `searchbar-history`) وما كُتب فيه. |
| `Count` | عدد مرات إدخال القيمة. |
| `GUID` | معرّف Firefox للإدخال، الذي تشير إليه سجلات الإزالة. |
| `Sources` | المكان الذي حفظ Firefox الإدخال منه، في الإصدارات التي تحتفظ بذلك (مفصولة بشريط عمودي في CSV). |
| `RowID` | معرّف صف الإدخال. |

## التحليل التلقائي { #automatic-parsing }

يكتب `auto-parse` الجداول في `BrowserActivity` بالأسماء `Browser-History.csv` و`Browser-Downloads.csv`
و`Browser-Cookies.csv` و`Browser-FormHistory.csv` (أو ملفًا واحدًا هو `Browser.json` أو `Browser.jsonl` مع
`-t json` أو `-t jsonl`)، مع `BrowserActivity/Browser-logs.txt`. ويتضمن الجدول الزمني المدمج `-j` كل صف،
مع `Comp` ومعرّف `Artifact` ذي القيمة `browser`. ويحوّل `-Z` الحقول `VisitTime` و`LastVisitTime`
و`StartTime` و`EndTime` و`Created` و`LastAccessed` و`Expires` و`FirstUsed` و`LastUsed` و`RemovedTime`.

## مزيد من التفاصيل { #further-details }

- [Chromium — page transition types](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — download danger types](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — Places database](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — write-ahead logging](https://www.sqlite.org/wal.html)
- [Plaso — browser SQLite parsers](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
