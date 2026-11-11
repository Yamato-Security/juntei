# تحليل البيانات المجمعة تلقائيًا { #automatic-collection-parsing }

يُعد `auto-parse` أسرع طريقة لتحويل مجموعة فرز أولي (triage) لـ Windows من عدة حواسيب إلى
مجلد تحليل منظم. أعطه جذر الأدلة ومجلد إخراج جديدًا. فيكتشف
الأنظمة المجمعة، ويحدد أسماء حواسيبها، ويشغّل كل محلل متوافق من محللات Juntei
بالتوازي.

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

الإخراج الافتراضي هو CSV. أضف جدولًا زمنيًا بصيغة JSONL لاستخدامه مع `jq` أو للاستيراد إلى SIEM مع الإبقاء على نتائج
CSV تلك:

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "أبقِ شجرتي الإخراج والأدلة منفصلتين"
    يرفض Juntei مجلد إخراج يقع داخل `<EVIDENCE>` أو يحتوي على `<EVIDENCE>`. وهذا
    يمنع إعادة اكتشاف البيانات المولَّدة بوصفها أدلة ويمنع `--clobber` من
    المساس بالملفات في المجموعة المصدر.

## صور أقراص VHDX { #vhdx-disk-images }

مع `--vhdx` يقرأ `auto-parse` صور VHDX مباشرة على **Linux وmacOS وWindows** عبر [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser)، دون تركيب نظام أو FUSE أو صلاحيات مسؤول.

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

تحتاج المحللات إلى مسارات، لذلك ينسخ Juntei الملفات التي تقرؤها المحللات إلى مساحة مؤقتة خاصة بمخازن قراءة محدودة: خلايا السجل وسجلات معاملاتها، وملفات بيانات NTFS الوصفية، وقواعد البيانات، وملفات Prefetch، والاختصارات، وJump Lists، وملفات Outlook، وملفات الفهرس في سلة المحذوفات، ومجلدات العزل في Defender. يصبح كل ملف آخر عنصراً نائباً فارغاً يحتفظ بوقت تعديله الأصلي، فتبدو المجلدات والمسارات كما هي على القرص. لا يقرأ persistence-hunter محتوى العنصر النائب من الصورة إلا عند الحاجة، مثل ملف تنفيذي يشير إليه إدخال استمرارية. **وفّر مساحة للملفات المنسوخة** إضافة إلى النتائج. تُكتب مناطق الأصفار كفجوات حيثما أمكن. تُحفظ الأسماء والبنية والروابط الصلبة وأوقات التعديل. تأتي أوقات مصادر LNK وJump Lists من لقطة بيانات NTFS الأصلية لا من النسخ المؤقتة. يحذف الانتهاء الطبيعي، بما فيه أخطاء المحللات، النسخ؛ وقد يتركها الإنهاء القسري. المصدر للقراءة فقط وتُطبّق سجلات VHDX المعلقة في الذاكرة وحدها.

يستخدم اكتشاف الأجهزة وCSV/JSON/JSONL و`-j` مسار المعالجة المعتاد. يحدد `Image` في `manifest.json` الصورة الأصلية، ويحدد `EvidenceRoot` المسار النسبي داخلها. تُسجّل أخطاء القراءة والأسماء غير الآمنة في `ImageErrors` وصفوف الفشل في `manifest.csv`. تُحفظ النتائج المقروءة ويُرجع الإدخال غير المكتمل الرمز 1. لا تُعاد تسمية الأسماء الخطرة أو الملتبسة. لا تُستخرج تدفقات NTFS المسماة كملفات إضافية؛ وتبقى الآثار المجموعة كملفات عادية متاحة.

يعرض أمر [`mount` المنفصل](mount.md) نظام الملفات للبرامج الأخرى ويحتاج إلى Linux FUSE. لا يستخدمه `auto-parse --vhdx`.

تستخدم مساحة التجهيز افتراضياً مجلداً خاصاً على نظام ملفات الإخراج. استخدم `--temp-dir` لاختيار مجلد موجود على قرص آخر. تُحفظ بيانات المصدر الوصفية منفصلةً على نظام ملفات الإخراج، فلا يمنع امتلاء قرص التجهيز تحليل الملفات المقروءة مسبقاً. خصص مساحة للنسخ والنتائج معاً؛ إذا امتلأت وحدة الإخراج نفسها فلن يمكن حفظ النتائج. يوقف Ctrl+C عمليات التحليل وينظف المساحة الخاصة ويخرج بالرمز 130. قد يترك SIGKILL أو انقطاع الطاقة نسخاً. يتم تجاوز المجلدات الفرعية غير المقروءة أثناء البحث عن الصور، لكن جذر الإدخال غير المقروء يبقى خطأ.

يجب أن يكون مجلد التجهيز خارج مجلد الإخراج. في Linux وmacOS، يوقف SIGTERM وSIGHUP أيضاً عمليات التحليل وينظفان مساحة العمل.

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## صور الأقراص E01 وEx01 { #e01-and-ex01-disk-images }

يقرأ `--e01` صور EWF1 الفعلية (`.E01`) وصور EWF2 (`.Ex01`) مباشرةً على **Linux وmacOS وWindows**، دون FUSE أو تركيب عبر نظام التشغيل أو صلاحيات مسؤول. يستخدم [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) ومسار تحليل NTFS نفسه الموضح أعلاه.

للصورة المقسمة، حدد الجزء الأول. احتفظ بجميع الملفات المصاحبة، ومنها `.E02`، بجواره بأسمائها الأصلية. تُدعم الأقراص الكاملة (MBR/GPT) ووحدات NTFS المنفردة. استخدم `--vhdx --e01` للمجلدات المختلطة.

تنطبق متطلبات المساحة المؤقتة وبيانات المصدر الوصفية نفسها. تحتوي صور E01 عادةً على أقراص كاملة، وتُجهَّز جميع الصور قبل بدء التحليل، لكن لا تُنسخ إلا الملفات التي تقرؤها المحللات، وهي عادةً جزء صغير من المساحة المستخدمة. تُفتح الصور للقراءة فقط. تُسجل فجوات الاكتساب والأضرار المكتشفة عند الفتح أو القراءة في `ImageErrors` و`manifest.csv`؛ تُحفظ النتائج المقروءة مع رمز خروج 1. تُفحص البيانات التي تم الوصول إليها، وليس كل الكتل أو بصمة الوسيط بأكمله. تُرفض الملفات المنطقية `.L01` و`.Lx01`، ولا يوجد خيار كلمة مرور.

لا يزال الأمر المنفصل `mount` يتطلب Linux FUSE؛ أما التحليل التلقائي المباشر فلا يحتاجه.

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## المنطقة الزمنية للإخراج { #output-time-zone }

تكون الطوابع الزمنية المحللة افتراضيًا بتوقيت UTC، بصرف النظر عن المنطقة الزمنية المحلية لحاسوب المحلل. ويستخدم UTC
صيغة سجل أحداث Windows ‏`2026-10-20T08:46:38.2001668Z`: تاريخ/وقت وفق ISO 8601، و`T`
بين التاريخ والوقت، و`Z` للدلالة على UTC. وتُحفظ الأرقام الكسرية المتاحة.
ويستخدم `GeneratedUTC` في `manifest.json` هذه الصيغة أيضًا. أما `Start time` و`End time` في وحدة التحكم
فهما بالتوقيت المحلي لحاسوب المحلل، مثل `2026-11-04 09:15:02`، أيًا كانت المنطقة الزمنية
للإخراج.

اختر إزاحة عرض ثابتة باستخدام `-Z, --timezone`:

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

الإزاحة قيمة `HH:MM` ذات إشارة ومكونة من رقمين (الساعات `00–23`، والدقائق `00–59`). وتُنتج الأسماء مثل
`JST`، والقيم المختصرة مثل `+9`، والنطاقات غير الصالحة خطأً مع مثال على الصيغة.
احذف الخيار أو استخدم `--timezone "+00:00"` للحصول على UTC. ولا تطبق الإزاحة الثابتة قواعد
التوقيت الصيفي.

ينطبق الخيار على الطوابع الزمنية المحللة في نتائج CSV وJSON وJSONL لكل أثر رقمي و
على تصدير JSONL الإضافي ‏`-j`. فمثلًا، يصبح `2026-10-20T08:46:38.2001668Z`
‏`2026-10-20T17:46:38.2001668+09:00`؛ وكلاهما يصف اللحظة نفسها.

تبقى أسماء الحقول ثابتة، بما في ذلك الأسماء الحالية المنتهية بـ `UTC`. وتحمل القيم المحوّلة صراحةً
الإزاحة المختارة. ويسجّل `OutputTimeZone` في كلا البيانين إزاحة العرض المطلوبة،
بينما يبقى `GeneratedUTC` بيانات وصفية للتشغيل بتوقيت UTC. وتحتفظ سجلات كل محلل بملخصات UTC الأصلية للمحللات.

تُحفظ أرقام FILETIME/epoch الخام، وقواميس قيم سجل Windows، وتكوين المنطقة الزمنية للمصدر، والبيانات
الوصفية النصية، والقيم التي تحتوي على التاريخ فقط، وقيم الساعة المحلية التي لا تُعرف إزاحتها كما هي. ولا يضيف Juntei
`Z` إلى وقت محلي غير معروف. ولا تملك ساعات عناصر shell المستمدة من FAT ‏(`Item*UTC` في
ShellBags/OpenSaveMRU و`TargetItems` في LNK/Jump Lists) إزاحة مسجلة؛ ولا تثبت تسمياتها القديمة بـ
UTC أو اللاحقة `Z` المعروضة أنها بتوقيت UTC، لذا يحتفظ بها خيار الإزاحة كما هي.
وتحتفظ أعمدة CSV التي تحتوي على ملخصات نصية حرة
بنصها المسجل؛ وتحتفظ JSON/JSONL بالطوابع الزمنية المحللة المتداخلة بوصفها حقولًا منظمة.

يستخدم UTC إخراج المحلل الموجود مباشرة. أما الإزاحة الصريحة غير الصفرية فتضيف مرحلة تنسيق
متدفقة داخل كل عامل من عمال المحللات؛ وهي تغيّر قيم العرض دون تغيير الأدلة.

إذا كانت الإزاحة ستنقل طابعًا زمنيًا خارج السنوات `0000–9999`، فإن تلك القيمة تحتفظ بإزاحتها
الصريحة الأصلية. كما تبقى القيم التي تحتوي على أكثر من تسعة أرقام كسرية دون تغيير بدلًا من
تقريبها. ويسجل Juntei تحذيرًا في ملف `*-logs.txt` الخاص بالمحلل ويحوّل الطوابع
الزمنية الأخرى. أما ملف CSV/JSON المشوّه أو خطأ الإخراج فيُفشل المهمة ويترك ملف النتيجة
الأصلي المتأثر دون تغيير.

## ما الذي يكتشفه { #what-it-discovers }

يبحث Juntei بشكل متكرر عن مجلدات تُسمى `Windows` تحتوي على مجلد `System32` أو `Prefetch`
أو `AppCompat`، دون تمييز لحالة الأحرف. ويدعم التعرف على المجلدين الأخيرين أيضًا المجموعات الموجهة
التي لا تحتوي إلا على Prefetch أو Amcache. ويستبعد ذلك مجلدات ملفات تعريف المستخدمين مثل
`AppData\Roaming\Microsoft\Windows`. وتُنسب التثبيتات القديمة المتداخلة مثل
`C:\Windows.old\Windows` إلى وحدة التخزين الخارجية المجمعة بدلًا من أن تصبح حواسيب
مكررة. والعمق الأقصى الافتراضي هو 10. وهذا يغطي مسارات Velociraptor الشائعة مثل:

```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

لا تغيّر الحد إلا عندما تكون المجموعة متداخلة بعمق أكبر:

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

لكل نظام مكتشف، يُختار اسم الحاسوب بهذا الترتيب:

1. `hostname` في أقرب `client_info.json` من الأسلاف (مع الاحتفاظ بقيمته بوصفها `FQDN` في البيان).
2. `ComputerName` في خلية سجل Windows ‏`SYSTEM` مجمعة.
3. أقرب اسم مفيد لمجلد المجموعة، مع تخطي محرك الأقراص وهيكل `uploads/auto` الخاص بـ Velociraptor
   وإزالة لاحقة مثل `_F.…`.

تُجعل الأسماء آمنة للاستخدام كأسماء مجلدات. وتتلقى الأسماء المكررة لاحقة رقمية في كل من
مجلد إخراجها وقيمة `Comp` حتى تبقى الأنظمة المنفصلة قابلة للتمييز في JSON.
ويحتفظ البيان باسم المضيف تمامًا كما اكتُشف في حقل منفصل هو `Hostname`.

بعد سرد الأنظمة، يطبع `auto-parse` سطرًا واحدًا `Timezone:` مباشرة فوق
`Parser tasks`. ويعرض المنطقة الزمنية المختارة للإخراج: `UTC` افتراضيًا، أو قيمة مثل
`UTC+09:00` عند استخدام `--timezone "+09:00"`. والتسمية خضراء والقيمة تستخدم
لون النص الافتراضي في الطرفية، مثل قيم الملخص الأخرى. ويُبقي `-K, --no-color` السطر عاديًا.
ويتوفر تكوين المنطقة الزمنية للأنظمة المجمعة في نتائج
[معلومات النظام](../artifacts/system-info.md).

## المحللات التي يشغّلها { #parsers-it-runs }

لا يوقف فشل محلل واحد المحللات الأخرى. ويشغّل `auto-parse` هذه المهام لكل حاسوب
مكتشف. وتستخدم المهام الأوامر نفسها المجمعة حسب المصدر المستخدمة في التحليل اليدوي، مثل
`parse-ntfs mft` و`parse-registry amcache` و`parse-ese srum` و`parse-file prefetch`:

- Amcache
- `$MFT`
- سجلات الإعادة/التراجع (redo/undo) في NTFS ‏`$LogFile`
- سجلات Prefetch وجدول زمني منفصل لتنفيذ Prefetch
- دفتر يومية USN
- فهارس المجلدات `$I30`
- قطاعات الإقلاع `$Boot` وواصفات الأمان `$Secure:$SDS`
- SRUM
- تسجيل وصول المستخدمين (UAL)
- سجل التصفح والتنزيلات وملفات تعريف الارتباط وسجل النماذج في متصفحات الويب (المتصفحات المبنية على Chromium وFirefox)
- قاعدة WebCache الخاصة بـ Internet Explorer وEdge القديم: سجل التصفح والتنزيلات وملفات تعريف الارتباط وإدخالات ذاكرة التخزين المؤقت
- ملفات Outlook من نوع PST وOST وMSG وOFT: المجلدات والعناصر والمستلمون والمرفقات وجهات الاتصال وعناصر التقويم والمهام
- اختصارات LNK
- Jump Lists
- سلة المحذوفات: المسارات الأصلية للعناصر المحذوفة وأحجامها وأوقات حذفها ومستخدموها
- ShimCache
- Persistence Hunter ‏(131 فحصًا للاستمرارية عبر المصادر مع ملفات تعريف السماح/الحظر)
- البيانات الوصفية لحجر Defender
- سجل مربعات حوار الفتح والحفظ باسم
- كل مفاتيح سجل Windows وقيمه، بما في ذلك البيانات المحذوفة القابلة للاستعادة
- الخدمات، وRecentDocs، وTaskCache، وحسابات SAM، ومواقع التشغيل التلقائي في سجل Windows، وUserAssist، وBAM،
  والبرامج المثبتة، وMuiCache، وShellBags، وأجهزة USB
- معلومات النظام: الأسماء، وإصدار Windows، والمنطقة الزمنية، وآخر إيقاف تشغيل، وملفات التعريف، ومعرّفات SID
- نهج التدقيق: أحداث الأمان التي ضُبط النظام لتسجيلها
- تكوين الإقلاع (BCD وBCD-Template)
- ملفات تعريف الشبكات، والواجهات، والمشاركات، ووكلاء المنافذ، والمسارات
- أسماء أسرار LSA وأوقاتها، وعمليات تسجيل الدخول المخزنة مؤقتًا للمجال قيد الاستخدام (لا يُفك تشفير أي شيء)

يحتفظ Persistence Hunter بكل مستويات الخطورة (`--min-severity info`)، بما في ذلك مطابقات السماح الجزئية.
و`LOW` تصنيف يستدعي التحقيق؛ وقد يُنتجه غياب البيانات الوصفية للتوقيع لأوامر ضارة
أيضًا. ولا يتجاهل الجرد التلقائي هذه الإدخالات باستخدام الحد الافتراضي المتوسط للأمر
اليدوي.
يستخدم الباحث عاملًا واحدًا داخل كل عملية محلل تلقائية. وهذا يمنع `auto-parse -T <N>`
من مضاعفة ميزانية العمليات بمجموعة أخرى بحجم عدد وحدات CPU في كل عملية فرعية. أما الأمر المستقل
`persistence-hunter -T <N>` فيتحكم في خيوط العمل الخاصة بالباحث.

يتضمن إخراج `$MFT` بصيغة CSV كل الطوابع الزمنية والأسماء المختصرة. ويتضمن إخراج سجل Windows البيانات الخام و
تفسيرًا بصيغة FILETIME لقيم `REG_QWORD`. ويتضمن إخراج SRUM بصيغتي JSON وJSONL البيانات الوصفية
لحالة القيم. ولا **تُستخرج** ملفات Defender ودفقات `$MFT` المقيمة تلقائيًا.

مع `--mft-slack`، يتضمن إخراج `$MFT` أيضًا أسماء الملفات المتبقية في المساحة الراكدة (slack) لسجلات FILE، كما يكتبها [`parse-ntfs mft -x`](../artifacts/mft.md): صفوف معلَّمة بـ `FromSlack` في CSV، و`SlackFileNames` في JSON وJSONL. ويُطبَّق `--code-page` على ملفات LNK وJump Lists وShellBags وOpenSaveMRU.

الأثر الرقمي الذي لم يُجمع هو مهمة `skipped` عادية، وليس فشلًا.

## تخطيط الإخراج { #output-layout }

تُنظَّم النتائج حسب الحاسوب وفئة التحقيق. وتستخدم مجلدات الفئات صيغة
PascalCase، مثل `Accounts` و`FileAndFolderOpening`. وتوجد جداول Amcache وJump Lists
مباشرة في فئاتها مع بادئات لأسماء الملفات. وتحتفظ أسماء مجلدات الحواسيب بتهجئتها المكتشفة. ولا تُرحَّل مجلدات
الإخراج الموجودة؛ لذا اختر مجلد إخراج جديدًا عند الانتقال من التسمية الأقدم. ويكتب كل محلل ملف
`<artifact>-logs.txt` بجانب نتائجه، حتى عندما لا يكون الأثر الرقمي قد جُمع.
ويحتفظ CSV بجداول منفصلة لأن لكل أثر رقمي أعمدة مختلفة:

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── BrowserActivity/
│   │   ├── Browser-Cookies.csv
│   │   ├── Browser-Downloads.csv
│   │   ├── Browser-FormHistory.csv
│   │   ├── Browser-History.csv
│   │   ├── Browser-logs.txt
│   │   ├── WebCache-Containers.csv
│   │   ├── WebCache-Cookies.csv
│   │   ├── WebCache-Downloads.csv
│   │   ├── WebCache-Entries.csv
│   │   ├── WebCache-History.csv
│   │   └── WebCache-logs.txt
│   ├── Communication/
│   │   ├── Outlook-Attachments.csv
│   │   ├── Outlook-Calendar.csv
│   │   ├── Outlook-Contacts.csv
│   │   ├── Outlook-Files.csv
│   │   ├── Outlook-Folders.csv
│   │   ├── Outlook-Items.csv
│   │   ├── Outlook-Recipients.csv
│   │   ├── Outlook-Tasks.csv
│   │   └── Outlook-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── RecycleBin.csv
│   │   ├── RecycleBin-logs.txt
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   ├── Registry-RemoteAccess-logs.txt
│   │   ├── UAL-Clients.csv
│   │   ├── UAL-RoleAccess.csv
│   │   ├── UAL-SystemIdentity.csv
│   │   └── UAL-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```

| الفئة | الآثار الرقمية |
|---|---|
| `Accounts` | حسابات SAM المحلية، وأسماء أسرار LSA وأوقاتها |
| `Applications` | سجل التطبيقات (آثار أدوات الجهات الخارجية) |
| `Applications` | البرامج المثبتة |
| `BrowserActivity` | سجل التصفح والتنزيلات وملفات تعريف الارتباط وسجل النماذج في متصفحات الويب، مع الإدخالات المحذوفة المستعادة؛ وقاعدة WebCache الخاصة بـ Internet Explorer وEdge القديم |
| `Communication` | ملفات Outlook من نوع PST وOST وMSG وOFT: المجلدات والعناصر والمستلمون والمرفقات وجهات الاتصال وعناصر التقويم والمهام |
| `ExternalDevices` | أجهزة USB ووحدات التخزين |
| `FileAndFolderOpening` | Jump Lists، واختصارات LNK، وMicrosoft Office، وقوائم MRU، ومربعات حوار الفتح والحفظ باسم، وRecentDocs، وShellBags |
| `FileExistence` | Amcache، سلة المحذوفات، ShimCache |
| `FileSystem` | `$Boot`، `$I30`، `$LogFile`، `$MFT`، `$Secure:$SDS`، دفتر يومية USN |
| `NetworkActivity` | ملفات تعريف الشبكات، والواجهات، والمشاركات، ووكلاء المنافذ، والمسارات؛ واتصالات SRUM وحركة المرور والإشعارات الفورية |
| `LateralMovement` | اتصالات المستخدمين الصادرة عبر سطح المكتب البعيد وPuTTY وWinSCP، ومحركات الأقراص المعيّنة والمشاركات؛ وتسجيل وصول المستخدمين (عملاء أدوار الخادم) |
| `Persistence` | تكوين الإقلاع (BCD)، والتشغيل التلقائي عبر سجل Windows، والخدمات، وTaskCache |
| `ProcessExecution` | توافق التطبيقات، وBAM وDAM، وFeatureUsage والعناصر المثبتة، وMuiCache، وPrefetch، والجدول الزمني لـ Prefetch، ونشاط التطبيقات/CPU في SRUM، وUserAssist |
| `Registry` | مفاتيح الخلايا وقيمها العامة، بما في ذلك السجلات المستعادة |
| `SecurityProducts` | البيانات الوصفية لحجر Defender والمحتوى المستخرج الاختياري؛ وإعدادات الأمان (Defender، وUAC، وحماية بيانات الاعتماد، وRDP، وجدار الحماية، والتسجيل) |
| `SystemInformation` | الأسماء، وإصدار Windows، والمنطقة الزمنية، وآخر إيقاف تشغيل، وملفات التعريف، ومعرّفات SID، ونهج التدقيق؛ وطاقة/سعة SRUM، وخرائط الهوية، والمزوّدات غير المعروفة |

تساعد الفئات على تنظيم التحقيقات؛ لكنها ليست دليلًا على نشاط بعينه. فمثلًا،
لا تثبت إدخالات Amcache وShimCache وحدها التنفيذ. ويسجل SRUM استخدام الموارد؛
راجع دليل الآثار الرقمية قبل استخلاص استنتاجات حول تشغيل برنامج ما.

تستخدم CSV وJSON وJSONL الفئات نفسها. ويحتوي ملف `<artifact>-logs.txt` واحد لكل مهمة محلل على
أمرها الدقيق ورسائلها؛ مثل `HOST1/FileSystem/MFT-logs.txt` و
`HOST1/ProcessExecution/Prefetch-logs.txt`. وتستخدم JSON وJSONL أسماء ملفات السجلات نفسها.
ولكل من Amcache وJump Lists سجل مشترك واحد بجانب جداولهما ذات البادئات. وتوضع جداول مزوّدات SRUM مباشرة
في `NetworkActivity` و`ProcessExecution` و`SystemInformation`، مع ملف مشترك هو
`SystemInformation/SRUM-logs.txt`. ولا تُنشأ مجلدات فرعية `logs/`.

أسماء ملفات CSV الافتراضية هي `Registry-SAM.csv`، `LNK.csv`، `RecycleBin.csv`، `Registry-RecentDocs.csv`، `Registry-ShimCache.csv`، `LogFile.csv`،
`I30.csv`، `MFT.csv`، `UsnJournal.csv`، `Boot.csv`، `SecureSDS.csv`، `Registry-Networks.csv`، `Registry-Persistence.csv`، `Registry-Services.csv`، `Registry-TaskCache.csv`، `Registry-BCD.csv`،
`Registry-SecuritySettings.csv`،
`PrefetchTimeline.csv`، `Prefetch.csv`، `Registry-UserAssist.csv`، `Registry-AppCompat.csv`، `Registry-Office.csv`، `Registry-FeatureUsage.csv`، `Registry-AppHistory.csv`، `Registry-MRU.csv`، `Registry-SystemInfo.csv`، `Registry-RemoteAccess.csv`، `Registry-OpenSaveMRU.csv`، `Registry-InstalledPrograms.csv`، `Registry-BAM.csv`، `Registry-MuiCache.csv`، `Registry-ShellBags.csv`، `Registry-USB.csv`، `Registry-SecuritySettings.csv`، `Registry-AuditPolicy.csv`، `Registry-Dump.csv`، و
`DefenderQuarantine.csv`. وتستخدم الجداول المستمدة من سجل Windows البادئة `Registry-`، بما في ذلك
`Registry-ShimCache.csv` و`Registry-UserAssist.csv` و`Registry-BAM.csv` و`Registry-SystemInfo.csv`.
وتوجد مفاتيح الخلايا وقيمها العامة في `Registry-Dump.csv`. وجداول Amcache هي
`FileExistence/Registry-Amcache-Devices.csv` و`Registry-Amcache-Files.csv`
و`Registry-Amcache-Findings.csv` و`Registry-Amcache-Summary.csv`. وتستخدم Jump Lists
‏`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` و
`JumpLists-CustomDestinations.csv`. ولا توجد مجلدات فرعية لـ Amcache أو Jump Lists.
وتستخدم جداول SRUM أسماء ملفات تبدأ بـ `SRUM-` مباشرة تحت فئتها، مثل
`NetworkActivity/SRUM-NetworkConnections.csv`.
وتوجد جداول تسجيل وصول المستخدمين في `LateralMovement`، بملف واحد لكل جدول، مثل
`UAL-Clients.csv` و`UAL-SystemIdentity.csv`، مع `UAL-logs.txt`؛ راجع
[تسجيل وصول المستخدمين (UAL)](../artifacts/ual.md#automatic-parsing).
وتوجد جداول متصفحات الويب في `BrowserActivity`: ‏`Browser-History.csv` و`Browser-Downloads.csv`
و`Browser-Cookies.csv` و`Browser-FormHistory.csv`، مع `Browser-logs.txt`؛ راجع
[متصفحات الويب](../artifacts/browser.md#automatic-parsing). وتوجد هناك أيضًا جداول WebCache الخاصة
بـ Internet Explorer وEdge القديم: ‏`WebCache-History.csv` و`WebCache-Downloads.csv`
و`WebCache-Cookies.csv` و`WebCache-Entries.csv` و`WebCache-Containers.csv`، مع
`WebCache-logs.txt`؛ راجع [Internet Explorer وEdge القديم](../artifacts/webcache.md#automatic-parsing).
وتوجد جداول Outlook في `Communication`: ‏`Outlook-Files.csv` و`Outlook-Folders.csv`
و`Outlook-Items.csv` و`Outlook-Recipients.csv` و`Outlook-Attachments.csv` و`Outlook-Contacts.csv`
و`Outlook-Calendar.csv` و`Outlook-Tasks.csv`، مع `Outlook-logs.txt`؛ راجع
[Outlook](../artifacts/outlook.md#automatic-parsing).
راجع [جدول توزيع SRUM الكامل](../artifacts/srum.md#categorized-auto-parse-output).

تطابق السجلات اسم الأثر الرقمي بأحرفه الكبيرة، مثل `Registry-SAM-logs.txt` و`PrefetchTimeline-logs.txt`
و`Registry-UserAssist-logs.txt`. وتستخدم ملفات JSON/JSONL الجذور نفسها، مثل `MFT.jsonl`؛ ويكون
معرّف `Artifact` للسجل هو `mft` لأغراض الاستعلام. وللمخرجات متعددة الجداول ملف
`Registry-Amcache-logs.txt` أو `JumpLists-logs.txt` بجانب جداولها. ويوجد سجل SRUM المشترك
في `SystemInformation`.


يسجل `manifest.json` قيمة `Hostname` الأصلية المكتشفة، وهوية `Comp` الفريدة للإخراج/السجل،
ومصدر اسم المضيف، ومسارات الأدلة، و`Category` للأثر الرقمي، وأسطر الأوامر، والمدة، ورمز
الخروج، ومسار النتيجة، ومسار السجل، والجدول الزمني JSONL المطلوب وأي خطأ في التصدير، وما إذا كان استخراج Defender مطلوبًا،
وما إذا كانت كل مهمة
قد نجحت، أو تُخطيت، أو فشلت.

لا يكتب auto-parse أي مسار مطلق، لذا يمكن مشاركة النتائج دون إظهار مكان الأدلة أو
النتائج أو Juntei على نظام التحليل:

- مسارات الأدلة نسبية إلى مجلد الأدلة. وتسرد الطرفية كل حاسوب مع
  مجلده في الأدلة (`HOST1: ./HOST1_F`)، وتعمل المحللات في مجلد الأدلة و
  تُعطى مثل هذه المسارات. وتُظهر أعمدة مسار المصدر في النتائج (`SourceFile`، `SourcePath`، …)، والسجلات،
  و`EvidenceRoot` و`WindowsDirectories` في البيانين موقع كل ملف في
  الأدلة (`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`).
- مسارات النتائج في البيانين والسجلات (`Output`، `OutputFiles`، `Log`، `JSON-Timeline`، و
  مسارات الإخراج في `Command`) و`ExtractedFile` الخاص بـ Defender نسبية إلى مجلد الإخراج
  (`./HOST1/ProcessExecution/Prefetch.csv`، و`../siem.jsonl` لجدول زمني بجانبه). ويبدأ `Command`
  باسم البرنامج (`juntei`)؛ ومسارات الأدلة فيه نسبية إلى مجلد الأدلة
  ومسارات الإخراج نسبية إلى مجلد الإخراج.
- `EvidenceDirectory` و`OutputDirectory` هما اسما المجلدين. ويشير `ManifestVersion` ‏2
  إلى البيانات التي تسجل مسارات النتائج انطلاقًا من مجلد الإخراج؛ أما البيانات الأقدم فليس فيها
  `ManifestVersion` وتسجلها كما أُعطيت لـ `-o`. وتقرأ إعادة التشغيل باستخدام `-C` كليهما.
- تعرض الطرفية مسار الإخراج أو الجدول الزمني كما أُعطي عندما يكون نسبيًا، وانطلاقًا من
  المجلد الحالي عندما يكون مطلقًا (`../results/`).

بعد عدد العمليات المتوازية، تعرض الطرفية الحجم الإجمالي لملفات
أنظمة Windows التي عُثر عليها (`Total filesize: 26.1 MiB`).

تعرض رسالة التقدم الحاسوب والأثر الرقمي النشطين متبوعين بحجم إدخال
تقريبي، مثل `HOST1 / prefetch (~2.0 MiB)`. وتستخدم مجموعة الملفات حجمها المجمّع؛ وتعرض كل
مهمة MFT أو LogFile أو دفتر يومية USN حجم إدخالها الفردي. ولا تشمل التقديرات سجلات المعاملات
المساعدة وملفات الإثراء ولا تتنبأ بحجم الإخراج المحلل.

يوفر `manifest.csv` فهرس المهام نفسه على هيئة جدول ملائم لجداول البيانات، مع صف واحد لكل
مهمة محلل. ويتضمن `Comp`، و`Hostname` الأصلي، و`FQDN`، و`HostnameSource`، و`EvidenceRoot`،
و`Artifact`، و`Category`، و`OutputType`، و`Purpose`، و`Status`، و`DurationMs`، و`ExitCode`، و`Output`، و`OutputFiles`، و`Log`،
و`Error`، و`Command`. وتكون قيمة `Purpose` هي `artifact` للنتائج المصنفة أو `json_timeline` لـ
مرحلة JSONL الأصلية الإضافية المستخدمة مع CSV. و`OutputType` هو تنسيق الإخراج الفعلي للمحلل.
و`Command` مصفوفة JSON تضم الملف التنفيذي ووسائطه، مع الحفاظ على المسافات وعلامات الاقتباس.
وبالنسبة إلى SRUM، يكون `Output` هو مجلد الحاسوب، و`OutputFiles` مصفوفة JSON بمسارات
إخراج المزوّدات الدقيقة، ويسمّي `Category` موقع السجل المشترك (`SystemInformation`).
وتُصنَّف الملفات فرديًا؛ ويظهر `OutputFiles` أيضًا في بيان JSON.

صفِّ `Status` على `skipped` أو `failed` للعثور على المهام التي تتطلب المراجعة؛ وقد يكون `Error`
لمهمة متخطاة فارغًا، مع وجود الشرح في ملف `Log` الخاص بها.

يتضمن ملخص الطرفية العدد الإجمالي للملفات العادية الموجودة حاليًا تحت مجلد
الإخراج وحجمها، بما في ذلك البيانان، وسجلات المحللات، والملفات المستخرجة. ولا تُتبع الروابط
الرمزية. وتتدرج الأحجام تلقائيًا باستخدام المُنسِّق نفسه المستخدم في `Total filesize`، مع
وحدات مثل `KiB` و`MiB` و`GiB`:

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

تستخدم جداول Amcache وSRUM ‏`-Source1` و`-Source2` وهكذا عند جمع عدة قواعد بيانات
مصدرية. ويسجل سجل المحلل المشترك الربط بين المسارات الأصلية والنهائية، وتحتفظ سجلات JSON/JSONL
بمعلومات المصدر. وتسرد مهام Amcache/Jump Lists بصيغة CSV مسارات جداولها الدقيقة في
`OutputFiles` في البيان؛ ويكون `Output` الخاص بها هو مجلد فئة التحقيق. أما صادرات JSON وJSONL
لـ Amcache وJump Lists فهي بالفعل ملفات مفردة باسم `Registry-Amcache.json[l]` و
`JumpLists.json[l]`. ويحتفظ الجدول الزمني المدمج ‏`-j` بمحتويات السجلات نفسها وقيم `Artifact`.
ويستخدم `--clobber` البيان السابق لمسح المخرجات المسجلة، بما في ذلك الأسماء الأقدم،
مع الحفاظ على ملفات الفئات غير ذات الصلة. وتكتب المجموعة العادية لوحدة تخزين واحدة سجلات `$MFT` ودفتر يومية USN مباشرة في
`FileSystem/MFT.csv` و`FileSystem/UsnJournal.csv`. وتستخدم وحدات التخزين الإضافية المجمعة ملفات
شقيقة مرقمة مثل `FileSystem/MFT-2.csv` و`FileSystem/MFT-2-logs.txt`.

## الاستيعاب في JSON وSIEM { #json-and-siem-ingestion }

يتلقى كل سجل JSON أو JSONL حقلين في المستوى الأعلى:

- `Comp`: النظام المصدر.
- `Artifact`: محلل Juntei الذي أنتج السجل، مثل `mft` أو `prefetch-timeline`.

يُنشئ `-j, --json-timeline <FILE>` ملف JSONL إضافيًا واحدًا، بصرف النظر عن تنسيق
النتائج الفردية. وتُحفظ كل المخرجات المصنفة وسجلاتها:

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

يُستخدم اسم الملف كما أُعطي؛ والمسارات النسبية نسبية إلى مجلد العمل الحالي.
ويمكن أن يكون في جذر الإخراج، أو في مجلد فرعي آخر غير محجوز في الإخراج، أو خارج مجلد
الإخراج. ولا يمكن استخدام الأدلة، أو البيانات، أو نتائج/سجلات كل حاسوب، أو مجلدات المحللات المؤقتة
وجهةً. ويتطلب الملف الموجود `-C`؛ ويتركه التصدير الفاشل سليمًا.

في **وضع CSV**، يُجري Juntei مرحلة JSONL أصلية إضافية حتى يحتفظ ملف SIEM بالأنواع
الرقمية والمنطقية، والبيانات المتداخلة، والحقول الخام، والبيانات الوصفية التي قد تُغفلها جداول CSV. وتتشارك
المهام الإضافية حد التزامن نفسه `-T`، ولا تستخرج محتويات الحجر مرة أخرى، و
لها سجلات جانبية منفصلة `*-json-timeline-logs.txt`. ويميّز كلا البيانين غرضها و
تنسيق إخراجها. وتُزال ملفات JSONL المؤقتة بعد تشغيل ناجح؛ وتُحفظ إذا فشل
محلل أو تصدير. فخطّط للوقت الإضافي للمعالجة ولمساحة القرص المؤقتة.

في **وضع JSON**، يدفق Juntei النتائج الموجودة إلى JSONL مضغوط دون إعادة تحليل
الأدلة. وتُوسَّع تقارير Amcache إلى سجلات الملخص والنتائج والملفات والأجهزة الفردية نفسها
الموجودة في JSONL الأصلي. وفي **وضع JSONL**، تُدمج السجلات المشروحة الموجودة مباشرة.
وتستخدم كل الأوضاع بنية سجل SIEM نفسها. وتحتفظ مجموعة البيانات بالطوابع الزمنية الأصلية وحقول
الآثار الرقمية؛ ولا تخترع طابعًا زمنيًا مشتركًا للأحداث ولا ترتب الآثار الرقمية غير المتجانسة ترتيبًا شاملًا.

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

يسجل `manifest.json` مسار `JSON-Timeline` المطلوب وأي `JSON-TimelineError`. ولا تزال سجلات
مهامه، و`manifest.csv`، تشير إلى مخرجات الآثار الرقمية المحفوظة. وتشير مهام الجدول الزمني فقط الناجحة في وضع CSV
إلى ملف JSONL النهائي بدلًا من الملفات المؤقتة المحذوفة. وإذا فشل أي محلل، فإن
التشغيل ينتهي دون نجاح ولا يحتوي الجدول الزمني إلا على النتائج الناجحة؛ فافحص البيانين
وسجلات الفئات قبل اعتباره كاملًا. ويغطي ملخص عدد الملفات/حجمها الملفات الموجودة داخل
مجلد الإخراج؛ ويُبلَّغ عن الجدول الزمني الخارجي بشكل منفصل بصيغة `JSONL timeline: <FILE>`.

## الأداء { #performance }

مهام المحللات عمليات منفصلة، لذا يمكن تشغيل الآثار الرقمية كثيفة الاستخدام لـ CPU في الوقت نفسه، ولا يمكن للتلف في
محلل واحد أن ينهي التشغيل بأكمله. ويستخدم Juntei افتراضيًا عدد وحدات CPU المنطقية. قيّد
التزامن عندما تكون الأدلة على قرص بطيء أو تكون ذاكرة النظام قليلة:

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## الاستخدام { #usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]
      --vhdx                  Parse VHDX images natively on Linux, macOS and Windows (read-only; temporary disk space required)
      --e01                   Parse E01/Ex01 images natively (split images from their first segment; temporary disk space required)
      --temp-dir <DIR>        Directory for private image staging (default: output filesystem)

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -Z, --timezone <OFFSET>             Display parsed timestamps using a fixed UTC offset (UTC by default) [default: +00:00]
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK files, Jump Lists, ShellBags and OpenSaveMRU [default: 1252]
      --mft-slack           Recover the file names left in the slack of $MFT records

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

استخدم مجلد إخراج جديدًا متى كان ذلك عمليًا. يزيل `-C` الأهداف المولَّدة الدقيقة لهذا التشغيل فقط أو يكتب فوقها،
لذا لا يمكن لأثر رقمي أصبح مفقودًا الآن أن يترك نتيجة قديمة. ولا يحذف
الملفات غير ذات الصلة ولا شجرة الإخراج بأكملها. كما يزيل مجلد
`SecurityProducts/DefenderQuarantined/` لكل حاسوب، والمجلد المحجوز `defender-quarantined/`
من التخطيط المسطح السابق، قبل التحليل، سواء استخدم التشغيل الجديد `-x` أم لا.

استخدم مجلد إخراج جديدًا عند الانتقال من تخطيط مسطح أقدم. فلا تُنقل ملفات CSV/JSON الأقدم
ولا تُحذف تلقائيًا.

!!! danger "تعامل مع محتوى Defender المستخرج على أنه برمجيات خبيثة"
    يكتب `-x, --extract-defender-quarantined` المحتوى المحجور على القرص تحت مجلد
    `SecurityProducts/DefenderQuarantined/` لكل حاسوب. وتنتهي الدفقات الثنائية المستعادة بـ `.quarantined`، ويزيل Juntei
    إذن التنفيذ. ويحصل كل واصف أمان ثنائي أيضًا على تقرير مقروء `.security_descriptor.txt`
    يتضمن الملكية والأذونات وSDDL. لا تنفّذ أبدًا الحمولات المستعادة. استخدم نظام تحليل
    معزولًا وتوقّع أن تُطلق برامج مكافحة الفيروسات أو EDR تنبيهات بشأن المحتوى المستعاد أو تزيله.

راجع [قراءة واصف الأمان المستعاد](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)
للاطلاع على حقول التقرير وحدودها الجنائية.

يقرأ auto-parse أيضًا الاتصالات البعيدة للمستخدمين (RDP وPuTTY وWinSCP ومحركات الأقراص المعيّنة).

## إنشاء جدول زمني شامل مرتب زمنيًا { #build-a-chronological-super-timeline }

بعد التحليل، استخدم [`super-timeline`](super-timeline.md) لتوسيع الطوابع الزمنية المسجلة
إلى أحداث موسومة. وأضف `--sort` لفرزها عبر الحواسيب والآثار الرقمية:

```bash
juntei super-timeline -d output/ -o SuperTimeline.csv --sort
```

يبقى تصدير `-j, --json-timeline` الحالي ملف JSONL مجمَّعًا للسجلات الأصلية.
وهو يحافظ على مخططات المحللات؛ ولا يوسّع حقول الطوابع الزمنية ولا يفرز الأحداث. احتفظ به
وبنتائج المحللات الفردية للمراجعة على مستوى الأدلة.
