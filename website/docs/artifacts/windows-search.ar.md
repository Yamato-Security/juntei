# فهرس Windows Search { #windows-search-index }

## ما الذي يسجله الفهرس { #what-the-index-records }

يزحف Windows Search (خدمة `WSearch`) إلى ملفات تعريف المستخدمين، وقائمة ابدأ، وصناديق بريد Outlook،
وسجل تصفح Internet Explorer، والملفات دون اتصال، وكل ما يقع ضمن نطاقه، ويحتفظ بما قرأه في
`Windows.edb`، وهي قاعدة بيانات ESE. ويهم التحقيقَ جزآن منها:

- **مخزن الخصائص** (`SystemIndex_PropertyStore`؛ و`SystemIndex_0A` في Windows 7) بصف واحد لكل عنصر
  مفهرس: مساره واسمه ونوعه وحجمه، ومالكه، وأوقات الإنشاء والتعديل والوصول التي نسخها المفهرس من الملف،
  ووقت قراءة المفهرس له، وعنوان المستند ومؤلفه، وبداية نصه، واسم الكمبيوتر، ومئات الخصائص الأخرى؛
- جداول **المجمّع** (`SystemIndex_Gthr`، `SystemIndex_GthrPth`) بسجل الزاحف (crawler) الخاص عن كل
  عنصر: مسار URL الخاص به، ووقت تعديله عند وضعه في قائمة الانتظار، ورقم الزحف وعلاماته، وعدّاد للحذف.

يبقى العنصر في الفهرس بعد زوال الملف أو الرسالة أو الصفحة، إلى أن يلاحظ الزاحف الحذف. ولذلك يسمّي
الفهرس ملفات لم تعد موجودة، مع بياناتها الوصفية ومقتطف من نصها كما كانت عند فهرستها.

<span class="evidence-strength">أفضل استنتاج:</span> قرأ المفهرس العنصر في المسار المسجل، بالبيانات
الوصفية المسجلة، في وقت `GatherTime`، على الكمبيوتر المسمّى. ويُظهر الفهرس أن الملف كان موجودًا وما الذي
احتواه. ولا يُظهر أن مستخدمًا فتحه.

## الموقع والجمع { #location-and-collection }

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

تحتفظ إصدارات Windows 7 حتى Windows 11 21H2، وإصدارات Windows Server من السنوات نفسها، بالفهرس في
`Windows.edb`. أما Windows 11 22H2 والإصدارات الأحدث فتحتفظ به في قواعد بيانات SQLite في المجلد نفسه
(`Windows.db` و`Windows-gather.db` و`Windows-usn.db`)؛ ويعثر Juntei عليها ويذكر أسماءها، ويبلّغ عنها
بوصفها التنسيق الأحدث غير المدعوم.

اجمع مجلد `Applications\Windows` بأكمله من صورة قرص، أو من نسخة ظل لوحدة التخزين، أو بعد إيقاف خدمة
`WSearch`: فقاعدة البيانات مفتوحة ما دامت الخدمة تعمل، ولا توجد أحدث تغييراتها إلا في سجلات المعاملات
`MSS*.log`، التي لا يعيد Juntei تشغيلها. ويبقى الفهرس المنسوخ من نظام قيد التشغيل غير متسق (dirty)؛
ويذكر الإخراج ذلك، ويقرأ ما هو موجود على القرص. ويقرأ إدخال المجلد و`auto-parse` كل ملف `Windows.edb`
على أي عمق؛ استخدم `-f` لملف واحد.

## تحذيرات للمحلل { #analyst-cautions }

- حلّل نسخة أدلة ثابتة. يستخدم Juntei ربط الملف بالذاكرة للوصول بكفاءة؛ لا تعدّل قاعدة البيانات أو تقتطعها أثناء قراءتها.
- `GatherTime` هو وقت قراءة المفهرس للعنصر، لا وقت إنشاء مستخدم له أو تغييره أو فتحه. فالفهرسة تعمل في
  الخلفية، وغالبًا بعد التغيير بدقائق إلى ساعات، وتتوقف مؤقتًا ما دام الكمبيوتر مشغولًا أو يعمل على
  البطارية.
- `DateCreated` و`DateModified` و`DateAccessed` نسخ أخذها المفهرس من الملف. وتُحدَّث عند إعادة فهرسة
  العنصر، ولذلك فهي تصف الملف كما كان في وقت `GatherTime`.
- `ItemPath` هو مسار العرض. أما `ItemURL` و`GatherPath` فهما عنوانا URL لدى الزاحف: `file:` للملفات (مع
  وحدة التخزين بصيغة `[GUID]` في Windows 8 والإصدارات الأحدث)، و`iehistory://{SID}/` لسجل تصفح
  Internet Explorer، و`mapi://` لعناصر Outlook، و`csc://{SID}/` للملفات دون اتصال، و`winrt://{SID}/`
  لمحتوى تطبيقات Windows.
- `User` هو ملف التعريف المسمّى في مسار العنصر (`C:\Users\<name>\…`). وهو يدل على مكان وجود العنصر، لا
  على من أنشأه؛ أما `FileOwner` فهو مالك الملف كما رآه المفهرس.
- `AutoSummary` هو بداية نص العنصر. وقد يقتبس من مستند حُذف أو شُفّر منذ ذلك الحين، وهو مقتطف مبتور،
  وليس المستند نفسه.
- قد تحتوي قاعدة البيانات غير المتسقة على صفحات لا تصل إليها صفحات الفروع فيها بعد. ويتتبع Juntei
  السلسلة الخاصة بالصفحات الورقية نفسها ويذكر في الملخص عدد ما قرأه منها بهذه الطريقة. ولا تزال أحدث
  تغييراتها في سجلات المعاملات.
- العنصر الموجود في جداول المجمّع دون صف في مخزن الخصائص (قيمة `InTables` هي `Gthr`) وُضع في قائمة
  الانتظار أو زُحف إليه، ولم تُكتب خصائصه بعد أو أُزيلت. ولا يزال مساره واسمه وأوقات المجمّع الخاصة به
  موجودة.
- يخزّن Windows 7 السلاسل النصية بترميز خاص به مضغوط ومموَّه. ويفك Juntei ترميزه؛ والقيمة التي لا يُفك
  ترميزها تبقى ست عشرية، مع ملاحظة في `Problems`.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

CSV هو تنسيق الملف الافتراضي. باستخدام `-t csv` أو `-t json` أو `-t jsonl` دون `-o`، تُكتب السجلات
إلى stdout. عند غياب الخيارين، يُعرض ملخص فقط. أضف `-C` لاستبدال ناتج موجود. ولا يُكتب الإخراج أبدًا
فوق أحد ملفات الإدخال، حتى مع `-C`.

يكتب Juntei سجلًا واحدًا لكل عنصر مفهرس، رابطًا مخزن الخصائص بجداول المجمّع عبر معرّف العنصر. ويعدّ
الملخص قواعد البيانات حسب المخطط، وغير المتسقة منها، والعناصر الموجودة في الجدولين معًا، وفي المجمّع
فقط، وفي مخزن الخصائص فقط، والمجلدات، والعناصر المعلَّمة على أنها محذوفة، والعناصر لكل مخزن، والعناصر
التي لا وقت مخزنًا لها، والمشكلات.

لا يوقف التلف التشغيل. فقاعدة البيانات المقتطعة تحتفظ بالعناصر التي تحملها صفحاتها المتبقية، ويذكر
الملخص ما تعذّرت قراءته. ويُبلَّغ عن أي ملف يحمل الاسم `Windows.edb` وليس فهرس Windows Search ويُتخطّى.
وتُقرأ قواعد البيانات الأخرى مع ذلك.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `DocumentID` | معرّف العنصر في الفهرس: `WorkID` في مخزن الخصائص (`DocID` في Windows 7) و`DocumentID` في صف المجمّع. |
| `InTables` | `Gthr+PropertyStore`، أو `Gthr` (عنصر في قائمة الانتظار أو زُحف إليه دون خصائص)، أو `PropertyStore`. |
| `ItemPath` | `System_ItemPathDisplay`: المسار كما يعرضه Windows. يكون فارغًا عندما لا يوجد صف في مخزن الخصائص. |
| `ItemURL` | `System_ItemUrl`: عنوان URL الذي يستخدمه الزاحف للعنصر (`file:`، `iehistory://`، `mapi://`، `csc://`، `winrt://`). |
| `GatherPath` | مسار المجمّع، مركّبًا من شجرة النطاقات واسم الملف في الصف. |
| `Name` | `System_ItemName`، أو اسم الملف في صف المجمّع. |
| `ItemType`، `ItemTypeText` | `System_ItemType` (الامتداد، أو `Directory`) واسمه المعروض. |
| `Kind` | `System_Kind`: `document`، `folder`، `link`، `program`، `email`، `picture`، …؛ وتُربط القيم المتعددة بـ `\|`. |
| `MimeType` | `System_MIMEType`. |
| `Store` | `System_Search_Store`: `file`، أو `mapi` (Outlook)، أو `iehistory`، أو `csc` (الملفات دون اتصال)، أو `winrt` (تطبيقات Windows). |
| `IsFolder`، `IsDeleted` | `System_IsFolder` و`System_IsDeleted`، كما هما مخزنان. |
| `DeletedCount` | عدّاد الحذف في صف المجمّع. |
| `Size` | `System_Size` بالبايت. يكون فارغًا للمجلدات، التي تحمل قيمة بديلة. |
| `FileAttributes` | `System_FileAttributes` بوصفه رقمًا (16 مجلد، 32 أرشيف، 2 مخفي، …). |
| `FileOwner` | `System_FileOwner` ‏(`DOMAIN\user`) كما قرأه المفهرس. |
| `DateCreated`، `DateModified`، `DateAccessed` | أوقات الملف كما نسخها المفهرس (UTC). |
| `ItemDate` | `System_ItemDate`: تاريخ العنصر نفسه، مثل وقت إرسال رسالة أو تاريخ التقاط صورة؛ وبالنسبة إلى الملف يكون عادةً وقت تعديله حتى الثانية. |
| `GatherTime` | `System_Search_GatherTime`: وقت قراءة المفهرس للعنصر (UTC). |
| `GatherLastModified` | قيمة `LastModified` في صف المجمّع: وقت تعديل العنصر عندما وضعه الزاحف في قائمة الانتظار (UTC). |
| `Title`، `Author` | `System_Title` و`System_Author`. |
| `AutoSummary` | `System_Search_AutoSummary`: بداية نص العنصر. |
| `CompName` | `System_ComputerName`: الكمبيوتر الذي يعود إليه الفهرس. |
| `TargetURL`، `DateVisited` | عنوان URL ووقت الزيارة لعنصر من سجل تصفح Internet Explorer ‏(`System_Link_TargetUrl`، `System_Link_DateVisited`). |
| `ActivityApp`، `ActivityText`، `ActivityContentURI`، `ActivityStartTime`، `ActivityEndTime` | عنصر من سجل الأنشطة في Windows 10 ‏(`System_Activity_*`، `System_ActivityHistory_*`). |
| `User`، `UserSource` | ملف التعريف المسمّى في مسار العنصر، والمسار الذي قُرئ منه (`ItemPath` أو `ItemURL` أو `GatherPath`). يكون فارغًا خارج ملف التعريف. |
| `ScopeID`، `SDID` | معرّفا النطاق وواصف الأمان في صف المجمّع. |
| `CrawlNumber` | قيمة `CrawlNumberCrawled` في صف المجمّع. |
| `GatherFlags` | قيمة `TransactionFlags` في صف المجمّع. |
| `Priority` | أولوية الزحف لعنصر لا يزال في قائمة الانتظار. ويعلّمه ESE بالقيمة NULL بمجرد الزحف إلى العنصر، لذا يكون فارغًا لمعظم العناصر. |
| `PropertyCount` | عدد الخصائص التي يحتفظ بها مخزن الخصائص للعنصر. |
| `OtherProperties` | كل الخصائص الأخرى، بصيغة `Name=Value` مفصولة بـ `; `. |
| `SourceFile` | قاعدة البيانات المقروءة، نسبةً إلى مجلد الإدخال، أو اسمها مع `-f`. |
| `Problems` | القيم التي لم يُفك ترميزها، والأوقات التي ليست أوقاتًا، والصفوف المكررة، والتلف. لا تُخفِها. |

تحتفظ JSON وJSONL بالحقول نفسها بقيم ذات أنواع: تبقى الأرقام أرقامًا، وتكون القيم المفقودة `null`.
وبدلًا من `OtherProperties` تتضمنان `Properties`، أي كل خاصية من خصائص العنصر مع اسمها ومعرّفها الرقمي
ونوع عمودها وقيمتها، وللوقت عدد نبضات FILETIME الخام بجانب الوقت الذي يُقرأ به. ويحمل `Gather` أعمدة
صف المجمّع.

## سلوك الجدول الزمني { #timeline-behavior }

يحفظ `auto-parse` السجلات بوصفها `FileExistence/WindowsSearch.csv` (أو `.json`/`.jsonl`) ويضيفها إلى
الجدول الزمني المدمج `-j`. وينشئ `super-timeline` أحداثًا من `DateCreated` و`DateModified`
و`DateAccessed` (أوقات الملف كما سجّلها Windows Search)، ومن `GatherTime` (وقت فهرسة العنصر)، ومن
`DateVisited` لعنصر من سجل تصفح Internet Explorer، ومن `ActivityStartTime` و`ActivityEndTime` للنشاط.
ولا يصبح `GatherLastModified` حدثًا إلا لعنصر لا يحتفظ مخزن الخصائص بوقت تعديل له. ويبقى العنصر الذي
لا يحمل أيًا منها في الإخراج المحلل ويُحتسب ضمن العناصر غير المحددة بوقت.

## استراتيجية التحقيق { #investigation-strategy }

1. رشّح `Store` إلى `file` وابحث في `ItemPath` و`Name` و`AutoSummary` عن ملفات القضية وأسمائها
   وكلماتها. فالفهرس يسمّي ملفات حُذفت منذ ذلك الحين.
2. اقرأ `GatherTime` بوصفه آخر وقت كان فيه الملف موجودًا بالبيانات الوصفية المسجلة، و`DateModified`
   بوصفه وقت تعديله في تلك اللحظة.
3. استخدم `FileOwner` و`User` لربط ملف بحساب، ثم تحقق من ذلك باستخدام `$MFT` ودفتر يومية USN
   و`$Secure:$SDS`.
4. بالنسبة إلى البريد الإلكتروني (`Store` بقيمة `mapi`) وسجل تصفح Internet Explorer ‏(`iehistory`)،
   انطلق إلى ملفات Outlook وقاعدة WebCache للحصول على السجلات الكاملة.
5. قارن الفهرس بين الأجهزة المضيفة: يسمّي `CompName` الكمبيوتر الذي يعود إليه الفهرس، ويسمّي الفهرس
   المجموع من صورة الملفات الموجودة على تلك الصورة.

## الإصدارات المختبرة { #tested-versions }

تستخدم الاختبارات فهارس عامة قرأت أدوات أخرى نتائجها بشكل مستقل:

- فهرس Windows 7 من plaso ‏(`SystemIndex_0A`)، مع مقارنته بأداة `esedbexport` من libyal، بما في ذلك
  ترميزات السلاسل النصية في Windows 7.
- فهرس Windows 10 من DFIRArtifactMuseum، وقد تُرك غير متسق، مع مقارنته بأداة SIDR من Stroz Friedberg،
  وفهرس Windows 11 منه (مراجعة تنسيق ESE رقم 200)، مع مقارنته بـ `esedbexport`.
- فهرس SQLite لنظام Windows 11 22H2 من DFIRArtifactMuseum، وقد أُبلغ عنه بوصفه التنسيق الأحدث غير
  المدعوم.
- قواعد بيانات مقتطعة، أو فارغة، أو من نوع آخر.

## مزيد من التفاصيل { #further-details }

- [Microsoft — نظرة عامة على Windows Search](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — تنسيق ملف قاعدة بيانات ESE](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb — Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg — SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft — الشفرة المصدرية لـ Extensible Storage Engine](https://github.com/microsoft/Extensible-Storage-Engine)
