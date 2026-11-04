# اختصارات LNK { #lnk-shortcuts }

## ما هي ملفات LNK { #what-lnk-files-are }

تصف ملفات Windows Shell Link (`.lnk`) هدفًا مع وسيطات التشغيل، ومجلد العمل، والأيقونة، وحالة
النافذة، والبيانات الوصفية للهدف، ومعلومات وحدة التخزين/الشبكة، وعناصر مساحة أسماء الواجهة (shell)،
ومعرّفات اختيارية لتتبع الارتباطات الموزعة (distributed link tracking). ينشئها Windows والتطبيقات في
العناصر الأخيرة (Recent Items) وسطح المكتب وقائمة "ابدأ" ومواقع أخرى؛ كما يسلّم المهاجمون اختصارات
ضارة مباشرةً.

يمكن لملف LNK في مجلد Recent أن يدعم استنتاج تفاعل المستخدم مع هدف لم يعد موجودًا، بما في ذلك محرك
أقراص قابل للإزالة أو مشاركة شبكية. أما ملف LNK عشوائي فقد يوجد دون أن يُنقر عليه قط.

<span class="evidence-strength">أفضل استنتاج:</span> أُنشئ الاختصار/حُدِّث بسياق الهدف المسجل.
ويحدد موقعه والآثار الرقمية المحيطة به ما إذا كان يدعم استنتاج الاستخدام.

## المواقع والجمع { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\*.lnk
%USERPROFILE%\Desktop\*.lnk
%APPDATA%\Microsoft\Windows\Start Menu\Programs\...
%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\...
```

ابحث في ملف تعريف المستخدم بأكمله وفي الأرشيفات/محتوى البريد الإلكتروني المجمّع، لا في `Recent`
وحده. احتفظ بأوقات ملف المصدر نفسه ومساره، لأن الموقع يغيّر التفسير.

## تحذيرات للمحلل { #analyst-cautions }

- أوقات الهدف وحجمه ومرجع MFT وبيانات وحدة التخزين هي لقطات مخزنة في ملف LNK؛ وليست البيانات
  الوصفية لملف الاختصار نفسه، وقد تكون قديمة.
- تصف `MachineID` وعنوان MAC ووقت إنشاء المتتبع سياق تتبع الارتباطات الموزعة، وهو عادةً الجهاز
  الذي أنشأ الارتباط/حدّثه—وليس بالضرورة الجهاز الذي عثرت عليه فيه.
- قد تأتي عقدة UUID الشبيهة بعنوان MAC من المحاكاة الافتراضية أو التوليد العشوائي أو معرّفات غير شبكية.
- يمكن لـ `Args` أن تجعل ملفًا تنفيذيًا حميدًا (`powershell.exe`، `rundll32.exe`، `mshta.exe`) خطيرًا.
- تتطلب سلاسل ANSI صفحة الترميز الخاصة بالنظام المصدر؛ استخدم `--code-page 932` لكثير من الارتباطات اليابانية.
- يمكن صياغة الاختصار أو نسخه. لا تساوِ حقوله بالتنفيذ دون دعم من Prefetch أو Jump List أو
  RecentDocs أو سجلات الأحداث أو غيرها.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.lnk
  -f, --file <FILE>      Parse one shortcut
      --code-page <CP>   ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -d evidence/ --code-page 932 -o lnk-ja.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

## حقول إخراج CSV { #csv-output-fields }

| مجموعة الحقول | الحقول وتفسيرها |
|---|---|
| المصدر | `SourceFile`، `SourceCreated`، `SourceModified`، `SourceAccessed`: مسار الاختصار وأوقاته في نظام ملفات المضيف. وهي منفصلة عن أوقات الهدف. |
| أوقات الهدف | `TargetCreated`، `TargetModified`، `TargetAccessed`: لقطات مسجلة في ملف LNK؛ وهي دلائل تاريخية مفيدة، وليست البيانات الوصفية الحالية للهدف. |
| أساسيات الهدف | `FileSize`، `RelativePath`، `WorkingDirectory`، `FileAttributes`، `HeaderFlags`: سياق الهدف/التشغيل المسجل. ويمكن للوسيطات أن تغيّر المعنى. |
| وحدة التخزين | `DriveType`، `VolumeSerialNumber`، `VolumeLabel`: تحدد سياق التخزين الثابت/القابل للإزالة/الشبكي وتربط الاختصارات الأخرى. |
| المسارات | `LocalPath`، `NetworkPath`، `CommonPath`، `TargetIDAbsolutePath`: تمثيلات متكاملة للمسار. راجع حالات التعارض ومسارات مساحة أسماء الواجهة. |
| سلوك التشغيل | `Args`، `ShowWindow`، `HotKey`، `ShimLayer`، `DarwinId`: كيفية تشغيل الواجهة للهدف أو تهيئته. و`Args` حاسمة الأهمية. |
| هوية NTFS | `TargetMFTEntryNumber`، `TargetMFTSequenceNumber`: مرجع الهدف عند إنشاء الارتباط؛ طابق كليهما مع `$MFT`. |
| التتبع | `MachineID`، `MachineMACAddress`، `TrackerCreatedOn`، `VolumeDroid`، `FileDroid`، `VolumeDroidBirth`، `FileDroidBirth`: تتبع الارتباطات الموزعة. استخدمها للربط، لا للإسناد وحده. |
| الكتل الإضافية | `ExtraBlocksPresent`: قائمة بالبنى الاختيارية المتاحة؛ وتوضح الحقول الأخرى المدعومة. |
| العرض | `Name`، `IconLocation`، `IconIndex`، `IconEnvironment`، `EnvironmentVariables`: بيانات العرض وبيانات الهدف بعد توسيع متغيرات البيئة. ويمكن للمهاجمين استخدام أسماء/أيقونات مضللة. |
| مجلدات الواجهة | `KnownFolder`، `SpecialFolder`: معرّفات مجلدات واجهة Windows بعد فك ترميزها. |
| الشبكة | `NetworkDevice`، `NetworkProvider`: سياق المشاركة/الجهاز الشبكي والمزوّد. |
| الترميز | `ConsoleCodePage`: صفحة ترميز وحدة التحكم المسجلة إن وُجدت؛ وهي منفصلة عن `--code-page` الخاص بالمحلل. |
| البيانات اللاحقة | `AppendedBytes`: البايتات الواقعة بعد النهاية المنطقية. وقد تشير البيانات غير المتوقعة إلى تهريب بيانات أو ملفات تالفة البنية. |
| التحقق | `Problems`: التلف أو الاقتطاع أو عدم اتساق الكتل أو قيود فك الترميز. ضمّنها في المراجعة. |

تحتفظ JSON/JSONL كذلك بكل عنصر من عناصر معرّف الهدف (النوع، والاسم، وGUID، وأوقات FAT، ومرجع
MFT)، وبجميع كتل البيانات الإضافية، وخصائص مخزن الخصائص، وإعدادات وحدة التحكم، والعلاقات البنيوية
الخام. استخدمها للاختصار المريب بدلًا من الاعتماد على صف CSV المسطح وحده.
تأتي الأوقات المتداخلة في `TargetItems` من ساعات عناصر الواجهة بتنسيق FAT دون إزاحة مخزنة.
ولا يثبت عرضها القديم بلاحقة `Z` أنها بتوقيت UTC؛ ويحافظ عليها `auto-parse --timezone` كما هي.
أما قيم FILETIME الخاصة بالهدف في ترويسة LNK وأوقات المتتبع فتظل طوابع زمنية مطلقة قابلة للتحويل.

## استراتيجية التحقيق { #investigation-strategy }

1. افحص موقع المصدر وأوقاته قبل تفسير حقول الهدف.
2. أعد بناء التشغيل الفعلي من مسار الهدف و`Args` و`WorkingDirectory` وكتل البيئة.
3. استخدم مرجع MFT للهدف، والرقم التسلسلي لوحدة التخزين، والمسار الشبكي، وقيم droid، وMachineID، وعنوان MAC نقاطًا محورية عبر الأدلة.
4. تحقق من Prefetch وسجلات الأحداث لإثبات التنفيذ؛ وتحقق من Jump Lists/RecentDocs لإثبات تفاعل المستخدم.
5. بالنسبة إلى المرفقات الضارة، احتفظ بالبيانات الملحقة واحسب تجزئة ملف LNK نفسه وحلّله.

## مزيد من التفاصيل { #further-details }

- [المواصفات المفتوحة من Microsoft: MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [Sootmark shell](https://github.com/Sootmark/shell)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [أداة LECmd من Eric Zimmerman](https://github.com/EricZimmerman/LECmd)
