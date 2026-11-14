# جدول الملفات الرئيسي (`$MFT`) { #master-file-table-mft }

## ما هو MFT { #what-the-mft-is }

يمثّل NTFS كل ملف ومجلد بسجل في جدول الملفات الرئيسي (Master File Table). وتحتوي السجلات على سمات
مثل الأسماء، ومراجع المجلد الأب، والطوابع الزمنية، والأحجام، والعلامات، ومعرّفات الأمان، ومعلومات
نقاط إعادة التوجيه (reparse)، والبيانات الوصفية لدفقات البيانات. ويمكن أن يكون محتوى الملفات الصغيرة
ودفقات البيانات البديلة **مقيمًا** (resident)، أي أن بايتاته مخزنة داخل سجل MFT نفسه.

يُعد MFT أساسيًا لأنه يغطي السجلات المخصصة والمحذوفة عبر وحدة التخزين بأكملها. ويمكنه أن يُظهر أن
ملفًا ما كان موجودًا، وأين كان مرتبطًا، وما إذا كان قد حُذف، وكيف تغيرت بياناته الوصفية—حتى عندما
لا يعود التصفح العادي للمجلدات يُظهره.

<span class="evidence-strength">أفضل استنتاج:</span> كان سجل ملف NTFS بالبيانات الوصفية المعروضة
موجودًا وقت الجمع أو بقي في سجل غير مخصص. ولا يثبت MFT وحده التنفيذ أو نية المستخدم.

## الموقع والجمع { #location-and-collection }

```text
C:\$MFT
```

`$MFT` بيانات وصفية محمية في NTFS. اجمعه بأداة اكتساب جنائي أو فرز (triage) قادرة على الوصول الخام
إلى NTFS؛ إذ لا يمكن عادةً لعملية نسخ عادية في وضع المستخدم قراءته. واقرنه بـ `$J` و`$LogFile`
والبيانات الوصفية لوحدة التخزين متى أمكن.

## تحذيرات للمحلل { #analyst-cautions }

- قد يُكتب فوق السجل المحذوف جزئيًا أو قد يُعاد استخدام رقم سجله.
- يميّز `SN` بين أجيال رقم الإدخال نفسه. طابق رقم الإدخال **و**رقم التسلسل.
- للطوابع الزمنية في `$STANDARD_INFORMATION` ‏(`0x10`) و`$FILE_NAME` ‏(`0x30`) سلوكيات تحديث مختلفة.
  ولا أحد منهما محصّن ضد التلاعب.
- تمثّل `SI<FN` و`uSecZeros` و`Copied` خيوطًا للتحقيق، وليست أحكامًا تلقائية بالتلاعب بالطوابع
  الزمنية (timestomping).
- تُنتج الارتباطات الصلبة (hard links) عدة سمات `$FILE_NAME` وعدة مسارات لسجل ملف واحد.
- يمكن لـ NTFS تخزين عدة أسماء (بما فيها أسماء DOS 8.3)، ودفقات بيانات بديلة، وسجلات امتداد.
- قد تكون تحديثات وقت الوصول معطلة أو مؤجلة أو مدمجة.
- يُعاد بناء المسار الكامل من سجلات المجلدات الأب. وتشير `[Orphaned]` أو `[Unknown]` أو `PathUnknown`
  إلى تسلسل أبوي مفقود/مُعاد استخدامه/تالف—وليست اسم مجلد حرفيًا.
- يمكن لـ **المساحة الراكدة** (slack) في سجل FILE (بايتاته الواقعة بعد الحجم المستخدم) أن تحتفظ بأسماء
  كان يحملها من قبل: إدخالات الفهرس لمجلد صغير انتقل فهرسه إلى تخصيص فهرس (index allocation) (ومعظمها
  لا يزال مستخدمًا هناك)، أو اسم سابق، أو الشاغل السابق لسجل مُعاد استخدامه. ولا تملك الأسماء المستردة
  `$STANDARD_INFORMATION`، وقد يكون رأس إدخال الاسم قد كُتب فوقه. وعندما لا يبيّن الحجم المستخدم للسجل أو
  سماته أين تبدأ مساحته الراكدة، لا يُسترد منها شيء، ويذكر `ParseErrors` في صفه `slack not read` والسبب.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $MFT files
  -f, --file <FILE>      Parse one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse selected entry numbers (example: 0-15,30)

Output:
  -o, --output <PATH>                   Save records
  -t, --output-type <TYPE>              csv, ys-ntfs-parser-csv, json, jsonl
  -e, --extract-resident-streams <DIR>  Extract resident $DATA streams
  -R, --reconstruct-filesystem <DIR>  Recreate a tree; recovered content gets .dontrun
      --reconstruction-i30 <FILE>      Supplement the tree with an I30 file (repeatable)

CSV Output:
  -a, --all-times      Include every $FILE_NAME time
  -s, --short-names    Include DOS 8.3-name rows
      --cutoff <DATE>  Keep recent SI-created or SI-modified records

Records:
  -x, --recover-slack  Recover file names from FILE record slack

Resident Data:
  -i, --include-resident-data      Put resident $DATA in the CSV
  -E, --resident-extensions <EXT>  Only these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  Only data up to this size

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail
  -L, --list-directory <ENTRY|PATH>  List a directory's contents
```

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff '2026-10-20T00:00:00Z' -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 32000-32100 | jq .
juntei parse-ntfs mft -f '$MFT' -x -o mft-with-slack.csv
juntei parse-ntfs mft -f '$MFT' -i -E ps1,bat,vbs,js -o scripts.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Users\alice\Downloads'
```

لا تنفّذ أبدًا الدفقات المقيمة المستخرجة باستخدام `-e`؛ فـ Juntei يمنحها أسماء `.dontrun` لسبب وجيه.
وأعمدة البيانات المقيمة في `-i` هي البايتات نفسها: تعامل معها بوصفها محتوى غير موثوق.

يطبع `-D` إدخالًا واحدًا: رأسه، وكل سمة (مع مسارات البيانات data runs) والأسماء الموجودة في مساحته
الراكدة كما يكتبها `-t json -x`، ثم بايتات السجل. ويسرد `-L` كل اسم في مجلد ما، المستخدم منها
والمحذوف، مع رقم الإدخال ورقم التسلسل، والنوع، والحجم، ووقت التعديل. ويُحدَّد المجلد برقم الإدخال
(`30`، `0x1E`، `30-1`) أو بالمسار (`\Windows\System32`، `C:/Windows`؛ دون تمييز لحالة الأحرف).

يعيد المثال بناء مسارات قابلة للتصفح. تصبح المحتويات غير المقيمة ملفات فارغة. التفاصيل: [إعادة بناء نظام الملفات](../commands/auto-parse.md#filesystem-reconstruction).

```bash
juntei parse-ntfs mft -f '$MFT' -R reconstructed
```

## حقول إخراج CSV { #csv-output-fields }

| الحقل | التفسير |
|---|---|
| `EntryNum` (Entry Number) | رقم سجل MFT. اجمعه مع `SN`؛ فأرقام السجلات يُعاد استخدامها. |
| `SN` (Sequence Number) | عدّاد الأجيال الذي تستخدمه مراجع NTFS. وقد يكشف عدم التطابق علاقات قديمة/محذوفة. |
| `InUse` | علامة التخصيص وقت الجمع. وتعني `false` أن السجل حر/محذوف، لا أن جميع البايتات سليمة. |
| `ParentEntryNum` (ParentEntryNumber)، `ParentSN` (Parent Sequence Number) | مرجع المجلد الأب من `$FILE_NAME`. تحقق من كلا الجزأين. |
| `ParentPath` | مسار المجلد الأب المُعاد بناؤه. وتكشف علامات المجهول/اليتيم تسلسلًا أبويًا لم يُحَل. |
| `File` (Filename) | الاسم من `$FILE_NAME` أو الدفق الخاص بالصف الحالي. وقد يكون للسجل الواحد عدة صفوف. |
| `Ext` (Extension) | امتداد مستخرج من الاسم للتيسير؛ وليس تحديدًا موثوقًا لنوع الملف. |
| `Size` (File Size) | الحجم المنطقي المرتبط بالصف/الدفق. وقد يختلف التخصيص المتناثر (sparse)/المضغوط. |
| `RefCnt` (Reference Count) | عدد مراجع الارتباطات الصلبة في NTFS؛ قارنه بحذر مع عدد الأسماء الباقية. |
| `ReparseTarget` | الهدف بعد فك ترميزه لنقاط إعادة التوجيه المدعومة مثل junctions/symlinks. وقد تعيد المسارات التوجيه. |
| `Dir` (Directory) | يحمل السجل علامة المجلد. |
| `HasADS` (Has Alternate Data Streams) | يحتوي السجل على دفقات بيانات بديلة مسماة. وهذا لا يجعله ضارًا. |
| `IsADS` (Is Alternate Data Stream) | يمثّل هذا الصف دفق بيانات بديلًا مسمى بدلًا من بيانات الملف غير المسماة. |
| `SI<FN` (Standard Information Before File Name) | يسبق وقتُ إنشاء SI وقتَ إنشاء FN مختلفًا. افحص احتمالات النسخ/التلاعب بالطوابع الزمنية والأسباب المشروعة. |
| `uSecZeros` (Microsecond Zeros) | يقع وقت إنشاء/تعديل SI تمامًا على حد ملّي ثانية. مجرد مؤشر استدلالي ضعيف على التلاعب بالطوابع الزمنية. |
| `Copied` | يسبق وقت التعديل وقت الإنشاء. وكثيرًا ما يتسق ذلك مع نسخ الملف؛ وليس دليلًا على فعل المهاجم. |
| `SI-Flags` (Standard Information Flags) | سمات/علامات الملف في `$STANDARD_INFORMATION`. ويمكن أن تكون علامات المخفي/النظام نقاط محورية مفيدة. |
| `Type` (Name Type) | مساحة أسماء `$FILE_NAME` ‏(Win32، أو DOS، أو POSIX، أو مجمّعة). استخدمها لتجنب احتساب الأسماء مرتين. |
| `DataModified (SI)`، `Accessed (SI)`، `MetadataChanged (SI)`، `Created (SI)` | أوقات SI. حقول شائعة في الجداول الزمنية، لكن أدوات وضع المستخدم قادرة على تغيير عدد منها. |
| `DataModified (FN)`، `Accessed (FN)`، `MetadataChanged (FN)`، `Created (FN)` | أوقات FN. تُحدَّث بطريقة مختلفة ويصعب عمومًا على واجهات API العادية تغييرها؛ لكنها مع ذلك ليست ثابتة. |
| `USN` (Update Sequence Number) | آخر USN مرتبط بسمة SI للسجل. انتقل منه إلى `$J` إن بقي ذلك USN. |
| `LogfileSN` (Logfile Sequence Number) | رقم تسلسل `$LogFile` في NTFS من رأس سجل FILE؛ مفيد في تحليل المعاملات على المستوى المنخفض. |
| `SID` (Security ID) | فهرس داخل `$Secure` في NTFS؛ وليس معرّف SID في Windows بحد ذاته. اربطه بـ `SecurityID` من [`parse-ntfs secure-sds`](secure-sds.md) لمعرفة مالك الملف وقوائم ACL الخاصة به. |
| `ObjectID-FileDroid` | معرّف الكائن / معرّف تتبع الارتباطات الموزعة حيثما وُجد؛ اربطه بمعرّفات droid في LNK/Jump List. |
| `LoggedUtilStream` | سياق `$LOGGED_UTILITY_STREAM` بعد فك ترميزه حيثما كان مدعومًا. راجع القيم غير المألوفة. |
| `ZoneID-Contents` | محتوى `Zone.Identifier` المقيم، مثل ZoneId وعناوين URL للتنزيل. وقد يكون غائبًا أو مُزالًا أو غير مقيم. |
| `SourceFile` | ملف `$MFT` المُدخل، وهو ضروري عند الجمع بين مضيفين أو وحدات تخزين. |
| `ResidentDataBase64`، `ResidentDataHex`، `ResidentDataASCII` | مع `-i`: محتوى `$DATA` المقيم في الصف (الخاص بالملف، أو بدفق البيانات البديل)، بصيغة Base64، وبصيغة بايتات (`41-42`)، وبصيغة نصية حين يكون ASCII قابلًا للطباعة. ويكون فارغًا عندما تكون البيانات غير مقيمة أو مستبعدة بواسطة `-E`/`-M`. |
| `ParseErrors` | تلف أو علاقات غير مدعومة تؤثر في هذا الصف. لا تتجاهل هذا العمود أبدًا أثناء التصفية. |
| `FromSlack` | مع `-x`: ‏`True` لاسم مسترد من المساحة الراكدة لسجل FILE. ويمثّل `EntryNum` و`SN` الخاصان به الملف الذي كان الاسم يشير إليه (ويكونان فارغين عندما يكون رأس إدخاله قد كُتب فوقه)، وتوجد طوابعه الزمنية المستردة في أعمدة `(FN)`، وتكون قيمة `InUse` هي `False`. |
| `SlackRecord`، `SlackOffset` | مع `-x`: سجل FILE الذي احتوت مساحته الراكدة على الاسم، بحسب موضعه في `$MFT` لا رقم السجل المكتوب داخله (سجل الامتداد نفسه، رغم أن أسماءه تُسرد مع سجله الأساسي)، وإزاحة الاسم بالبايت داخل `$MFT`. وتمثّل `ParseErrors` لصف المساحة الراكدة مشكلات ذلك السجل، مثل `fixup values do not match`: فقد تتأثر البايتات المستردة. |
| `CopyOfEntryInUse` | مع `-x`: لا يزال ملف مستخدم يحمل هذا الاسم في المجلد نفسه (وهو الملف نفسه، إن بقي المرجع). والأسماء التي لا يحملها أي ملف مستخدم هي المثيرة للاهتمام: محذوفة، أو أُعيدت تسميتها، أو الشاغل السابق لسجل مُعاد استخدامه. |

تتبع أعمدة الطوابع الزمنية ترتيب MACB ‏(Modified، Accessed، Changed، Birth/creation):
`DataModified` ‏SI/FN، ثم `Accessed` ‏SI/FN، ثم `MetadataChanged` ‏SI/FN، ثم `Created` ‏SI/FN.

تعني `(SI)` ‏`$STANDARD_INFORMATION`؛ وتعني `(FN)` ‏`$FILE_NAME`. وتصف هذه الأسماء المصدر المعتاد
لسجل كامل. أما بالنسبة لصف اسم ملف عادي لا يحتوي على سمة SI، فإن آلية الاحتياط القائمة تضع وقت إنشاء
FN في `Created (FN)`، وتضع قيم FN للتعديل/تغيير السجل/الوصول في `DataModified (SI)`
و`MetadataChanged (SI)` و`Accessed (SI)` على الترتيب.
وتضع صفوف المساحة الراكدة المستردة الأوقات الأربعة جميعها في أعمدة `(FN)` وتترك أعمدة SI فارغة.
وقيم الاحتياط في الصفوف العادية **ليست** دليلًا مستقلًا من SI. استخدم سمات JSON الأصلية ومصدر
المساحة الراكدة عند تحديد مصدر الطابع الزمني.

يستخدم `-t ys-ntfs-parser-csv` (المعروف سابقًا باسم `ys-mft-parser-csv`، ولا يزال مقبولًا) المخطط
الموجّه نحو الإدخالات الخاص بالمحلل الذي تجري صيانته. وتنطبق أسماء الأعمدة المقروءة هذه على إخراج
`-t csv` الافتراضي، بما في ذلك `MFT.csv` الخاص بـ `auto-parse`. ويحتفظ JSON/JSONL بكل
سمة (بما فيها `$DATA` المقيم، بالنظام الست عشري) وبـ `FullPath`؛ استخدمه عندما يُسقط ملف CSV التحقيقي
المسطح البنية منخفضة المستوى. ومع `-x`، يحتوي كل إدخال JSON أيضًا على
`SlackFileNames`.

## استراتيجية التحقيق { #investigation-strategy }

1. صفِّ نافذة زمنية، لكن احتفظ بالسجلات المحذوفة وبكلتا مجموعتي الطوابع الزمنية.
2. ابحث في المسارات الكاملة، والامتدادات المزدوجة المريبة، وADS، ومجلدات التجهيز، والتجزئات/الأسماء المعروفة.
   ومع `-x`، أدرج أسماء المساحة الراكدة التي لا يحملها أي ملف مستخدم (`CopyOfEntryInUse` ‏`False`)؛ ومع
   `-i -E ps1,bat,vbs,js`، اقرأ النصوص البرمجية الصغيرة مباشرةً من ملف CSV.
3. انتقل من رقم الإدخال/التسلسل و`USN` إلى دفتر يومية USN.
4. قارن مراجع MFT في Prefetch/LNK/Jump List بالسجل الهدف.
5. اربط الحذف بحجر Defender أو بنشاط `FILE_DELETE`/إعادة التسمية في USN.

## مزيد من التفاصيل { #further-details }

- [Microsoft: جدول الملفات الرئيسي](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [حزمة mft في ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/mft)
- [محلل mft من Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [وثائق تنسيق libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)
