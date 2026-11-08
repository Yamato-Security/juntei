# Internet Explorer وEdge القديم { #internet-explorer-and-legacy-edge }

## ما هي قاعدة بيانات WebCache { #what-the-webcache-database-is }

الملف `WebCacheV01.dat` هو قاعدة بيانات وظائف الإنترنت في Windows ‏(WinINet) داخل ملف التعريف الخاص
بكل مستخدم، وهي قاعدة بيانات ESE. ويحتفظ فيها Internet Explorer 10 و11 بسجل التصفح والتنزيلات وذاكرة
التخزين المؤقت وملفات تعريف الارتباط، وكذلك يفعل Edge القديم، أي Edge في Windows 10 قبل أن يصبح Edge
مبنيًا على Chromium. وكل نوع من السجلات حاوية (container)، أي جدول قائم بذاته:

- **سجل التصفح** (`History`، و`MSHist01…` ليوم أو لأسبوع): عناوين URL التي زيرت، وكل منها مع عنوان
  الصفحة، والحساب الذي سُجّل له، ووقت آخر زيارة له، وعدد مرات زيارته. ويسجل مستكشف الملفات هناك أيضًا الملفات المحلية
  التي يفتحها، على شكل عناوين URL من النوع `file:///`.
- **التنزيلات** (`iedownload`): عنوان URL الذي جاء منه الملف، والمكان الذي حُفظ فيه والمكان الذي كُتب
  فيه أثناء التنزيل، وحجمه، ووقت بدء التنزيل، والصفحة المُحيلة، ونوع MIME.
- **ملفات تعريف الارتباط**: في Windows 10 والإصدارات الأحدث، ملفات تعريف الارتباط نفسها (جداول
  `CookieEntryEx`) مع مضيفها واسمها وقيمتها وأوقاتها؛ وقبل ذلك، الملفات التي كانت تحفظها.
- **ذاكرة التخزين المؤقت** (`Content`): الملفات التي نُزّلت لعرض الصفحات، وكل منها مع عنوان URL الخاص
  به، ومكان النسخة على القرص، ورؤوس استجابة HTTP. وتخزّن البرامج الأخرى التي تستخدم WinINet تنزيلاتها
  هنا مؤقتًا أيضًا.
- تخزين الويب (`DOMStore`)، وقوائم التوافق (`iecompat`)، وغيرها.

تعيش قاعدة البيانات أطول من المتصفحات: فقد أُوقف Internet Explorer، ويحتفظ Edge المبني على Chromium بقواعد
بيانات SQLite الخاصة به (راجع [متصفحات الويب](browser.md))، لكن قاعدة البيانات ما زالت موجودة في كل ملف
تعريف، بما سُجّل فيها من قبل، وبما لا يزال مستكشف الملفات والبرامج الأخرى تسجله فيها.

<span class="evidence-strength">أفضل استنتاج:</span> سجّل ملف تعريف المستخدم عنوان URL أو التنزيل أو
الملف المخزن مؤقتًا في الوقت المسجل؛ ولا تقول قاعدة البيانات من كان يجلس أمام لوحة المفاتيح، ولا إن الملف
المخزن مؤقتًا قد فُتح.

## الموقع والجمع { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

توجد الملفات التي تسمّيها الحاويات (الملفات المخزنة مؤقتًا، وملفات تعريف الارتباط) في مجلداتها الخاصة:
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\` و`INetCookies\` و`History\`، وتحت
`%LOCALAPPDATA%\Packages\` للتطبيقات.

اجمع مجلد `WebCache` بأكمله. يُبقي Windows قاعدة البيانات مفتوحة ما دام المستخدم مسجلًا دخوله، لذا تكون
النسخة المأخوذة من نظام قيد التشغيل غير متسقة (dirty) عادةً: فلا توجد أحدث تغييراتها إلا في سجلات
المعاملات `V01*.log`.

!!! warning "سجلات المعاملات"
    لا يعيد Juntei تشغيل سجلات معاملات ESE. وتُحتسب قاعدة البيانات التي لم تُغلق بشكل سليم في الملخص على
    أنها تُركت غير متسقة، ويُبلَّغ عنها بوصفها مشكلة: فالتغييرات التي لا توجد إلا في سجلاتها تكون مفقودة.
    ولتضمينها، أعد تشغيل السجلات أولًا في نسخة من المجلد، بتشغيل الأمر `esentutl /r V01 /d` الخاص
    بـ Windows داخل تلك النسخة.

## تحذيرات للمحلل { #analyst-cautions }

- الأوقات بتوقيت UTC، بدقة 100 نانوثانية، باستثناء `ModifiedTime` في حاوية سجل التصفح الخاصة بيوم أو
  أسبوع: فهو بالتوقيت المحلي للنظام، ويُكتب من دون إزاحة، ويتركه `-Z` كما هو. فحوّله باستخدام المنطقة
  الزمنية للنظام المصدر في ذلك الوقت. والفرق بينه وبين `AccessedTime` هو غالبًا إزاحة تلك المنطقة الزمنية،
  لكن ليس دائمًا: فقد يكون الوقتان لحظتين مختلفتين.
- إدخال سجل التصفح عنوانُ URL وليس زيارة: فـ `AccessedTime` هو آخر زيارة له، و`AccessCount` عدد
  الزيارات. وتحتفظ الحاوية `History` بإدخال واحد لكل عنوان URL؛ وتحتفظ حاوية اليوم أو الأسبوع بإدخالات
  تلك الفترة، ويعطي اسمها أول يوم فيها واليوم الذي يلي آخر يوم فيها
  (`MSHist012026110120261102`: 1 نوفمبر 2026).
- عناوين URL من النوع `file:///` في سجل التصفح ملفات محلية فُتحت من مستكشف الملفات ومن برامج أخرى، وليست
  صفحات ويب.
- `Account` هو الحساب الذي سُجّل له الإدخال (`Visited: alice@…`)؛ و`User` هو مجلد ملف التعريف الذي يقع
  فيه مسار قاعدة البيانات. وقد يختلفان: فلا يحمل مجلد ملف التعريف اسم حسابه دائمًا، وقاعدة البيانات
  المنسوخة إلى مكان آخر لا يتضمن مسارها مجلد ملف تعريف.
- يبيّن `Package` لمن يعود الإدخال: لـ Edge القديم (`microsoft.microsoftedge_8wekyb3d8bbwe`)، أو
  لـ Internet Explorer في الوضع المحمي المحسّن (`windows_ie_ac_001`)، أو لتطبيق آخر. وعندما يكون فارغًا،
  يكون الإدخال للمستخدم نفسه: Internet Explorer ومستكشف الملفات والبرامج الأخرى.
- لا توثّق Microsoft سجل التنزيل. ويقرأ Juntei وقت بدئه وحجمه فقط عندما تقع ترويسته بكاملها داخل السجل، ومساراته
  وسلاسله النصية الأخرى فقط عندما تكون السلسلتان الأخيرتان عنوان URL ومسار Windows. وإلا فيحمل الصف عنوان
  URL الخاص بالإدخال نفسه، ووقت الوصول الخاص به عندما تتعذر قراءة وقت البدء، مع مشكلة. و`Publisher` هو
  الاسم الذي سجله المتصفح، وليس تحققًا من توقيع الملف.
- يدل إدخال ذاكرة التخزين المؤقت على أن الملف نُزّل وخُزّن مؤقتًا. وقد تكون النسخة على القرص قد اختفت،
  أو استُبدلت، أو لم تُفتح قط.
- لا تُستعاد الإدخالات المحذوفة بعد: فقد يحتفظ ESE بسجل محذوف في صفحاته إلى أن ينظفها، لكن Juntei لا
  يقرأ إلا السجلات النشطة.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

يبحث `-d` عن `WebCacheV01.dat` باسمه؛ ويقرأ `-f` قاعدة البيانات أيًّا كان اسمها.

## جداول الإخراج { #output-tables }

ينشئ CSV خمسة ملفات، لا يحتوي كل منها إلا على صف الترويسة عندما لا تكون فيه صفوف. وتحمل صفوف JSON
وJSONL الحقل `Table` ‏(`History` أو `Downloads` أو `Cookies` أو `Entries` أو `Containers`) بعد
`SourceFile`.

| ملف CSV | المحتوى |
|---|---|
| `WebCache-History.csv` | إدخالات حاويات سجل التصفح. |
| `WebCache-Downloads.csv` | التنزيلات. |
| `WebCache-Cookies.csv` | ملفات تعريف الارتباط (Windows 10 والإصدارات الأحدث). |
| `WebCache-Entries.csv` | إدخالات كل الحاويات الأخرى: الملفات المخزنة مؤقتًا، وملفات تعريف الارتباط، وتخزين الويب، وقوائم التوافق، وما سواها. |
| `WebCache-Containers.csv` | الحاويات. |

## الحقول { #fields }

يبدأ كل جدول بهذه الحقول:

| الحقل | التفسير |
|---|---|
| `SourceFile` | قاعدة البيانات التي قُرئ منها الصف. |
| `User` | مستخدم Windows الذي يقع المسار ضمن ملف تعريفه (المجلد الذي يلي `Users`). |
| `Package` | التطبيق الذي تعود إليه بيانات الصف: المجلد الموجود تحت `AppData\Local\Packages` في مجلد حاويته (أو مجلد جدول ملفات تعريف الارتباط الخاص به). فارغ لبيانات المستخدم نفسه. |

### سجل التصفح { #history }

| الحقل | التفسير |
|---|---|
| `Container` | الحاوية: `History`، أو `MSHist01` متبوعًا بأول يوم في فترتها واليوم الذي يلي آخر يوم فيها. |
| `AccessedTime` | آخر زيارة لعنوان URL ‏(UTC)؛ وفي حاوية يوم أو أسبوع، آخر زيارة في تلك الفترة. |
| `ModifiedTime` | آخر وقت تغيّر فيه الإدخال (UTC)؛ وفي حاوية يوم أو أسبوع، التوقيت المحلي للنظام، من دون إزاحة. |
| `CreationTime` | وقت إنشاء الإدخال (UTC). |
| `ExpiryTime`، `SyncTime` | الوقت المقرر لخروج الإدخال من سجل التصفح، وآخر وقت جرت فيه مزامنته (UTC). |
| `Account` | الحساب الذي سُجّل له الإدخال. |
| `URL` | الصفحة أو الملف، من دون البادئة `Visited: account@`. |
| `Title` | عنوان الصفحة كما سجّله الإدخال؛ وهو فارغ في حاوية يوم أو أسبوع. |
| `AccessCount`، `SyncCount` | عدد مرات زيارة عنوان URL، وعدد مرات مزامنة الإدخال. |
| `EntryType`، `Flags` | نوع الإدخال وعلاماته كما خُزّنت. |
| `EntryID`، `ContainerID` | معرّف الإدخال في حاويته، ومعرّف الحاوية. |

### التنزيلات { #downloads }

| الحقل | التفسير |
|---|---|
| `StartTime` | وقت بدء التنزيل (UTC). |
| `URL` | مصدر الملف. |
| `TargetPath`، `CurrentPath` | مكان حفظ الملف، والمكان الذي كُتب فيه أثناء التنزيل: ملف `.partial` في Edge القديم، ونسخة ذاكرة التخزين المؤقت في Internet Explorer. |
| `TotalBytes` | حجم الملف. |
| `Referrer` | الصفحة المُحيلة. |
| `MimeType` | نوع الملف كما قدّمه الخادم (Internet Explorer). |
| `Publisher` | الاسم الذي سجله Edge القديم لناشر الملف. |
| `AccessedTime` | وقت الوصول الخاص بالإدخال نفسه (UTC)، وهو وقت البدء في أغلب الأحيان. |
| `DownloadID`، `ContainerID` | معرّف الإدخال في حاويته، ومعرّف الحاوية. |

### ملفات تعريف الارتباط { #cookies }

| الحقل | التفسير |
|---|---|
| `LastModified`، `Expires` | وقت تعيين ملف تعريف الارتباط، ووقت انتهاء صلاحيته (UTC). |
| `Host`، `Name`، `Path` | الموقع الذي يخصه ملف تعريف الارتباط، واسمه، ومساره. |
| `Value` | القيمة كما خُزّنت. |
| `Flags` | علامات ملف تعريف الارتباط كما خُزّنت. |
| `Directory` | المجلد الذي تعطيه قاعدة البيانات لجدول ملفات تعريف الارتباط: لمن تعود ملفات تعريف الارتباط هذه (`…\INetCookies\ESE\`، أو `…\MicrosoftEdge\Cookies\ESE\` الخاص بـ Edge القديم، أو مجلد تطبيق). |
| `CookieTable`، `EntryID` | الجدول (`CookieEntryEx_<n>`) ومعرّف ملف تعريف الارتباط فيه. |

### الإدخالات { #entries }

| الحقل | التفسير |
|---|---|
| `Container` | الحاوية: `Content` (الملفات المخزنة مؤقتًا)، `Cookies` (ملفات تعريف الارتباط)، `DOMStore` (تخزين الويب)، `iecompat`، …. |
| `AccessedTime` | آخر وقت استُخدم فيه الإدخال (UTC). |
| `ModifiedTime` | آخر تعديل للمورد، كما أعطاه الخادم (UTC). |
| `CreationTime` | وقت إنشاء الإدخال (UTC). |
| `ExpiryTime`، `SyncTime` | وقت انتهاء صلاحيته، وآخر وقت جرى فيه التحقق منه مع الخادم (UTC). |
| `URL` | عنوان URL كما خُزّن: عنوان URL لملف مخزن مؤقتًا، و`Cookie:alice@example.com/` لملف تعريف ارتباط، و`DOMStore:…`، و`iecompat:…`. |
| `Account` | حساب ملف تعريف الارتباط. |
| `Filename`، `FilePath` | ملف الإدخال ومكانه: مجلد الحاوية والمجلد الفرعي الذي يسمّيه الإدخال. |
| `FileSize` | حجم الملف. |
| `AccessCount`، `SyncCount` | عدد مرات استخدام الإدخال، وعدد مرات التحقق منه مع الخادم. |
| `EntryType`، `Flags` | نوع الإدخال وعلاماته كما خُزّنت. |
| `RedirectURL` | الوجهة التي أعاد عنوان URL التوجيه إليها. |
| `ResponseHeaders`، `RequestHeaders` | رؤوس استجابة HTTP ‏(`HTTP/1.1 200 OK`، `Content-Type`، `Server`، …) ورؤوس الطلب، كما خُزّنت، بأسطرها كاملة. |
| `EntryID`، `ContainerID` | معرّف الإدخال في حاويته، ومعرّف الحاوية. |

### الحاويات { #containers }

| الحقل | التفسير |
|---|---|
| `ContainerID`، `Name` | معرّف الحاوية واسمها. |
| `Directory` | مكان ملفاتها على القرص. |
| `SecureDirectories` | المجلدات الفرعية من `Directory` التي توجد فيها ملفاتها (تفصل بينها فاصلة في CSV). |
| `PartitionID` | `M` لحاويات المستخدم نفسه، و`L` لحاويات الوضع المحمي في Internet Explorer (مجلدات `Low`)، ومعرّف SID لحاوية التطبيق لحاويات أي تطبيق. |
| `LastAccessTime`، `LastScavengeTime` | آخر وقت استُخدمت فيه الحاوية، وآخر وقت نُظّفت فيه (UTC). |

## التحليل التلقائي { #automatic-parsing }

يكتب `auto-parse` الجداول في `BrowserActivity` بالأسماء `WebCache-History.csv` و`WebCache-Downloads.csv`
و`WebCache-Cookies.csv` و`WebCache-Entries.csv` و`WebCache-Containers.csv` (أو ملفًا واحدًا هو
`WebCache.json` أو `WebCache.jsonl` مع `-t json` أو `-t jsonl`)، مع `BrowserActivity/WebCache-logs.txt`.
ويتضمن الجدول الزمني المدمج `-j` كل صف، مع `Comp` ومعرّف `Artifact` ذي القيمة `webcache`. ويحوّل `-Z`
كل الأوقات، أي الحقول التي تنتهي أسماؤها بـ `Time`، و`LastModified` و`Expires`، باستثناء `ModifiedTime`
المحلي في حاوية سجل التصفح الخاصة بيوم أو أسبوع.

## مزيد من التفاصيل { #further-details }

- [Microsoft — About WinINet](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE database file format](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — MSIE WebCache parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
