# Jump Lists { #jump-lists }

## ما هي Jump Lists { #what-jump-lists-are }

تقف Jump Lists وراء القوائم التي تظهر من إدخالات التطبيقات في شريط المهام وقائمة "ابدأ". فـ Jump
Lists التلقائية هي سجلات يديرها Windows للعناصر التي استخدمها تطبيق ما. أما Jump Lists المخصصة فهي
فئات ومهام يحددها التطبيق. ويمكن لكلا النوعين تضمين بُنى LNK تحتوي على مسارات الأهداف، والأوقات،
وسياق وحدة التخزين/الشبكة، ومراجع MFT، ووسيطات التشغيل.

يمكن للسجلات التلقائية أن تكشف الملفات أو المجلدات التي فتحها تطبيق ما، وترتيب/وقت آخر استخدام، وعدد
مرات التفاعل، وحالة التثبيت (pinned). ويمكن للقوائم المخصصة أن تكشف صفحات "أُغلقت مؤخرًا" أو مهام
التطبيق—بما في ذلك وسيطات أوامر ضارة.

<span class="evidence-strength">أفضل استنتاج:</span> سجّل التطبيق المالك العنصر/المهمة في Jump List
الخاصة به أو عرضها فيها. ويختلف السلوك من تطبيق إلى آخر؛ فأكّد التنفيذ/الوصول الفعلي.

## المواقع والجمع { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

البادئة الست عشرية في اسم الملف هي AppID. اجمع كلا المجلدين واحتفظ بمسار ملف تعريف المستخدم.
القوائم التلقائية حاويات OLE Compound File تضم دفق `DestList` ودفقات LNK مرقّمة؛ وتستخدم القوائم
المخصصة تنسيقها الخاص للفئات/الارتباطات.

## تحذيرات للمحلل { #analyst-cautions }

- تأتي أوصاف AppID من جدول تعيين معروف؛ ومعرّف AppID غير المعروف ليس مريبًا في حد ذاته.
- تنتمي حقول `LastModified` / آخر استخدام إلى إدخال DestList؛ أما أوقات الهدف المضمّنة فمختلفة.
- لا توجد أعداد التفاعل إلا في الإصدارات المدعومة، وهي ليست عددًا كاملًا لمرات الفتح.
- يحافظ التثبيت على العناصر بما يتجاوز الحداثة المعتادة؛ وقد يزيل التنظيف أو إعدادات التطبيق عناصر أخرى.
- قد تحتوي القائمة على دفقات قديمة/غير مرتبطة أو على بايتات من قائمة مخصصة سابقة أطول.
- وسيطات ومسارات LNK المضمّنة يتحكم فيها المهاجم. والارتباط بتطبيق ما سياقٌ، لا ثقة.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-list-csv
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

مع `auto-parse`، تكون جداول CSV هي `FileAndFolderOpening/JumpLists-AutomaticDestinations.csv`
و`FileAndFolderOpening/JumpLists-CustomDestinations.csv`، مباشرةً تحت كل حاسوب.
والسجل المشترك هو `JumpLists-logs.txt` في تلك الفئة. وتسرد ملفات البيان (manifests) المسارات الدقيقة
في `OutputFiles`. أما عمليات تصدير CSV المستقلة فتحتفظ بـ `AutomaticDestinations.csv`
و`CustomDestinations.csv` في المجلد الذي تختاره. وعمليات التصدير التلقائية بصيغتي JSON وJSONL هي
ملفات مفردة `JumpLists.json` أو `JumpLists.jsonl`؛ وتبقى بنية السجلات دون تغيير.

## حقول CSV لـ AutomaticDestinations { #automaticdestinations-csv-fields }

| مجموعة الحقول | الحقول وتفسيرها |
|---|---|
| المصدر | `SourceFile`، `SourceCreated`، `SourceModified`، `SourceAccessed`: حاوية Jump List وأوقاتها في نظام الملفات. |
| التطبيق | `AppId`، `AppIdDescription`: معرّف AppID من اسم الملف وتعيين التطبيق المعروف. لا تستنتج الهوية من تجزئة غير معروفة. |
| الحاوية | `HasSps`، `DestListVersion`، `LastUsedEntryNumber`: بنية/إصدار DestList وسياق الرأس. |
| الترتيب | `MRU`، `EntryNumber`: موضع آخر استخدام ومعرّف الإدخال/الدفق بالنظام الست عشري. |
| بيانات الاستخدام الوصفية | `CreationTime`، `LastModified`، `InteractionCount`، `PinStatus`، `PinPosition`: دورة حياة إدخال DestList واستخدامه وحالة تثبيته. فسّرها حسب الإصدار. |
| المضيف/المسار | `Hostname`، `MacAddress`، `Path`، `DisplayPath`: موقع الهدف وسياق التتبع. ولا يكفي المضيف/عنوان MAC وحده للإسناد. |
| معرّفات التتبع | `FileBirthDroid`، `FileDroid`، `VolumeBirthDroid`، `VolumeDroid`: اربط الأهداف المنقولة وتتبع LNK المضمّن. |
| الملاحظات | `Notes`: تخبرك بالدفقات غير المرتبطة، أو البقايا المستردة، أو سياق آخر من المحلل. |
| LNK المضمّن | `TargetCreated`، `TargetModified`، `TargetAccessed`، `FileSize`، `RelativePath`، `WorkingDirectory`، `FileAttributes`، `HeaderFlags`، `DriveType`، `VolumeSerialNumber`، `VolumeLabel`، `LocalPath`، `CommonPath`، `TargetIDAbsolutePath`، `TargetMFTEntryNumber`، `TargetMFTSequenceNumber`، `MachineID`، `MachineMACAddress`، `TrackerCreatedOn`، `ExtraBlocksPresent`، `Args`: فسّرها كما هو موضح في [اختصارات LNK](lnk.md). |
| LNK إضافي | `NetworkPath`، `Name`، `IconLocation`: حقول الشبكة والعرض من الاختصار المضمّن. |
| التحقق | `Problems`: دفقات مفقودة، أو اقتطاع، أو عدم تطابق في الأعداد، أو تلف في الارتباط. |

## حقول CSV لـ CustomDestinations { #customdestinations-csv-fields }

يبدأ الإخراج المخصص بـ `SourceFile` و`SourceCreated` و`SourceModified` و`SourceAccessed` و`AppId`
و`AppIdDescription` و`EntryName` (اسم الفئة أو `Tasks`)، تليها جميع حقول LNK المضمّنة المذكورة
أعلاه، ثم `Notes` و`NetworkPath` و`Name` و`IconLocation` و`Problems`.

ويحفظ JSON/JSONL أيضًا رأس DestList، والنتيجة/المجموع الاختباري/مخزن الخصائص لكل إدخال،
و`DestListPropertyStore`، ودفقات الملف المركّب، والفئات المخصصة، والبايتات الواقعة بعد آخر فئة،
والحقول المتداخلة لكل LNK مضمّن.

## استراتيجية التحقيق { #investigation-strategy }

1. حدد المستخدم وملف التعريف المصدر، ثم عيّن AppID إلى التطبيق بحذر.
2. رتّب الإدخالات التلقائية حسب `LastModified` و`MRU`؛ وافصل العناصر المثبتة.
3. راجع المسارات، والمشاركات الشبكية، والأرقام التسلسلية لوحدات التخزين القابلة للإزالة، ومراجع MFT، والوسيطات.
4. افحص `Notes` والارتباطات غير المرتبطة/المتبقية—فقد تبقى فيها عناصر محذوفة أو سابقة.
5. اربط النتائج مع RecentDocs، وملفات LNK المستقلة في Recent، وPrefetch، وSRUM، و`$MFT`، وسجلات الأحداث.

## مزيد من التفاصيل { #further-details }

- [Microsoft: تنسيق Compound Binary File ‏(MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft: تنسيق Shell Link ‏(MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [أداة JLECmd من Eric Zimmerman](https://github.com/EricZimmerman/JLECmd)
- [مشروع اختبار/مرجع لمحلل JumpList](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
