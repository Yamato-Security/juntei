# تحميل صور الأقراص { #mounting-disk-images }

يتيح `juntei mount` قراءة ملفات صورة قرص VHDX / E01 / Ex01 عبر مجلد للقراءة فقط طوال تشغيله. تستطيع أدوات مثل Hayabusa وJuntei فحصها دون استخراج الملفات أو طلب صلاحيات root. تستخدم الميزة FUSE وتعمل على Linux. تُصان مكتبة القراءة ومكتبة التركيب في [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

في طرفية أخرى:

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

اضغط Ctrl+C في الطرفية الأولى لإلغاء التحميل، أو شغّل `fusermount3 -u /mnt/host1`.

## ما يعرضه التحميل { #what-the-mount-shows }

- تظهر ملفات وحدة NTFS عند نقطة التركيب. قد تحتوي الصورة المجمعة على مجلدات مثل `C` و`LongFileNames`، وبيانات وصفية للوحدة مثل `$MFT`. تظهر وحدات NTFS المتعددة باسم `Volume1` و`Volume2` وهكذا.
- كل ملف للقراءة فقط ويعود إلى مالك نقطة التحميل. وأوقات الملفات هي أوقات `$STANDARD_INFORMATION`
  في NTFS.
- تُسرد الأسماء كما يخزنها NTFS، ويُبحث عنها بمطابقة تامة. ولا تُعرض أسماء DOS 8.3 ولا دفقات البيانات
  البديلة.
- يُفك ضغط الملفات المضغوطة (حتى 1 GiB لكل ملف). ولا يمكن قراءة الملفات المشفرة بـ EFS.

## لا تتغير الأدلة { #the-evidence-is-not-changed }

تُفتح الصورة للقراءة فقط ويرفض التركيب الكتابة. إذا لم تُغلق صورة VHDX بصورة سليمة، يُطبق سجلها المعلق في الذاكرة ولا يُكتب إلى الصورة. تترك بعض مكونات DiscUtils تعليمة القفز والتوقيع `55 AA` في قطاع إقلاع NTFS بالقيمة صفر؛ يقرأ Juntei هذه الوحدات كما هي.

## الاستخدام { #usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX or E01; a split E01 image from its first segment)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## تحليل الصور محليًا { #parsing-every-image-with-auto-parse }

مع `--vhdx` يقرأ `auto-parse` صور VHDX مباشرة على **Linux وmacOS وWindows** عبر [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser)، دون تركيب نظام أو FUSE أو صلاحيات مسؤول.

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

تحتاج المحللات إلى مسارات، لذلك ينسخ Juntei الملفات التي تقرؤها المحللات إلى مساحة مؤقتة خاصة بمخازن قراءة محدودة: خلايا السجل وسجلات معاملاتها، وملفات بيانات NTFS الوصفية، وقواعد البيانات، وملفات Prefetch، والاختصارات، وJump Lists، وملفات Outlook، ومجلدات العزل في Defender. يصبح كل ملف آخر عنصراً نائباً فارغاً يحتفظ بوقت تعديله الأصلي، فتبدو المجلدات والمسارات كما هي على القرص. لا يقرأ persistence-hunter محتوى العنصر النائب من الصورة إلا عند الحاجة، مثل ملف تنفيذي يشير إليه إدخال استمرارية. **وفّر مساحة للملفات المنسوخة** إضافة إلى النتائج. تُكتب مناطق الأصفار كفجوات حيثما أمكن. تُحفظ الأسماء والبنية والروابط الصلبة وأوقات التعديل. تأتي أوقات مصادر LNK وJump Lists من لقطة بيانات NTFS الأصلية لا من النسخ المؤقتة. يحذف الانتهاء الطبيعي، بما فيه أخطاء المحللات، النسخ؛ وقد يتركها الإنهاء القسري. المصدر للقراءة فقط وتُطبّق سجلات VHDX المعلقة في الذاكرة وحدها.

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

## متطلبات التركيب الظاهر لنظام التشغيل { #os-visible-mount-requirements }

يتطلب `juntei mount` نظام Linux وFUSE 3 (`fusermount3`) والوصول إلى `/dev/fuse`. استخدم التحليل المباشر أعلاه على Windows وmacOS.
