# التحقق من المحللات { #parser-validation }

تُختبر حزم المحللات التي يصونها Juntei مقابل أدلة عامة وتطبيقات
مستقلة. وتساعد هذه المقارنات على اكتشاف التراجعات واختلافات التفسير؛ لكنها
لا تجعل أي محلل معصومًا من الخطأ. احتفظ بالأدلة المصدرية، وراجع حقول المشكلات أو
التلف في كل سجل، وتحقق من النتائج ذات العواقب باستخدام أثر رقمي أو تطبيق آخر.

تحفظ البيانات أدناه معلومات التحقق التي كانت موجودة سابقًا في ملف README الرئيسي
لـ Juntei. واتبع مستودعات المحللات المرتبطة للاطلاع على بيانات الاختبار الحالية وتفاصيل التنفيذ.

## Amcache { #amcache }

يستخدم Juntei المحلل [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)، وهو
تفرّع مرخص بموجب Apache-2.0 من
[amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) الخاص بـ Security Ronin. ويعرّف
[توثيق الإخراج](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md) الخاص به
كل حقل من حقول الملفات التنفيذية والأجهزة والنتائج والملخص.

## جدول الملفات الرئيسي { #master-file-table }

حزمة mft في [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) تفرّع مُصان من حزمة
[mft](https://github.com/omerbenamram/mft) الخاصة بـ Omer Ben-Amram. وعبر ثمانية ملفات `$MFT`، هي عينة
الحزمة، وقضية الاختراق من NIST، وستة ملفات من DFIR Artifact Museum، تطابقت 1,048,733 صفًا وعمودًا في CSV
بالترتيب مع محلل مرجعي. وشملت المقارنات الإخراج الافتراضي، و`--all-times`،
و`--short-names`، و`--cutoff`؛ وكان عرض الطوابع الزمنية هو الاختلاف المقصود.

## سجل معاملات NTFS ‏($LogFile) { #ntfs-transaction-log-logfile }

تُصان حزمة logfile في [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) انطلاقًا من
[Sootmark ntfslog](https://github.com/Sootmark/ntfslog). ويشمل التحقق منها ستة سجلات كاملة من DFIR
Artifact Museum، وبيانات اختبار عامة للإصدارين 1.1 و2.0، والبيانات الوصفية للإعادة/التراجع بعد فك ترميزها، وعمليات
كتابة دفتر اليومية مقارنةً ببيانات NTFS محللة بشكل مستقل. راجع مستودع المحلل للاطلاع على التغطية
التفصيلية والاختلافات المعروفة.

يضيف Juntei اختبارات لسطر الأوامر تقارن كل حقل CSV بـ JSON وJSONL على وحدة تخزين الاختبار العامة
go-ntfs ‏(2,129 سجلًا)، وتتحقق من بيانات الإعادة/التراجع المنفصلة وأطوال الحمولات الخام، وتختبر السجلات التالفة،
وترفض الكتابة فوق الأدلة، وتمنع مسارات MFT الحالية من تتبع أرقام سجلات أُعيد استخدامها.
وتميّز حالات اختبار الانحدار سجلات MFT المتأثرة عن السمة المفتوحة التي تكتب من خلالها،
وتفحص البيانات الوصفية المسطحة لإدخالات المجلدات/سمات البيانات، وتمنع إقران MFT عبر وحدات تخزين مختلفة.
وتغطي اختبارات `auto-parse` متعددة وحدات التخزين ملفات CSV منفصلة وملف JSONL واحدًا يحمل معلومات المصدر.

فُحص الأمر أيضًا على ستة سجلات Windows جُمعت بشكل خاص: تطابقت كل سجلات CSV المُصدَّرة البالغة 708,465
سجلًا مع JSONL حقلًا بحقل، واحتفظت صادرات `auto-parse` لكل مضيف بأعداد السجلات
نفسها. ولا تُوزَّع الأدلة المصدرية مع المستودع. وتتحقق هذه الفحوصات من اتساق
الإخراج، لا من أن كل معاملة مسجلة تمثل إجراءً مكتملًا من المستخدم.

## Prefetch { #prefetch }

يُعد [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) تفرّعًا مُصانًا من
[frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) الخاص بـ ForensicRS. وقد قورن إخراجه بعد فك الترميز
مع [libscca](https://github.com/libyal/libscca)، وهو تطبيق libyal المستقل لـ
Prefetch.

## دفتر يومية USN { #usn-journal }

حزمة usn في [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) تفرّع
مُصان من [Sootmark usn](https://github.com/Sootmark/usn). وعلى ستة دفاتر يومية حقيقية، تطابقت مسارات
الأب المراعية للزمن في Juntei مع محلل مستقل في كل مكان حلّ فيه ذلك المحلل مسارًا. كما
حلّ Juntei 4,321 سجلًا تركتها المقارنة دون حل. ولم تُنتج أداة الاستخراج الخاصة به أي سجلات
زائفة عند اختبارها على 880 MB من بيانات `$MFT`.

## فهارس المجلدات ($I30) { #directory-indexes-i30 }

تقرأ الوحدة `i30` في حزمة mft الخاصة بـ [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) سجلات INDX
ومساحتها الراكدة (slack). وعلى دفقين `$I30` من وحدة تخزين أُنشئت باستخدام ntfs-3g (‏`test_files/i30/` في
Juntei) وعلى ثلاث عينات `$I30` عامة كتبها Windows، تطابقت إدخالاتها المستخدمة والموجودة في
المساحة الراكدة مع إدخالات [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) من Maxim Suhanov، بالأسماء
والأوقات نفسها. وحيث يُبلغ dfir_ntfs عن مرجع أب يتجاوز 32 بت (بايتات كُتبت فوق
الأب)، يترك Juntei الأب فارغًا.

## قطاع الإقلاع ($Boot) { #boot-sector-boot }

حزمة boot في [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) من تطوير Yamato
Security نفسها. وعلى ملفات `$Boot` العامة الخمسة من DFIR Artifact Museum ‏(Windows 10 و11 و
Server 2012 R2)، تطابق كل حقل مع محلل مرجعي. وعلى وحدات تخزين أُنشئت باستخدام `mkntfs` بقطاعات 512
و1,024 و4,096 بايت ومجموعات (clusters) من 4 KiB إلى 64 KiB، تطابقت الأحجام، ومجموعات `$MFT` و`$MFTMirr`
والأرقام التسلسلية مع `fsstat` من The Sleuth Kit. ويحمل ملف `$Boot` الخاص بـ Windows 11 في المتحف
الرقم التسلسلي لوحدة التخزين نفسه الذي يسجله ملف LNK من المضيف نفسه، وللملفات الخمسة
شيفرة الإقلاع نفسها.

## واصفات الأمان ($Secure:$SDS) { #security-descriptors-securesds }

حزمة secure في [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) من تطوير Yamato
Security نفسها. وعبر دفقات `$SDS` الخمسة في المتحف (7,808 واصفات، و60,781 إدخال ACE)، تطابقت
معرّفات الأمان، والتجزئات، والإزاحات، والمالكون، والمجموعات، وأعلام التحكم، وأعداد إدخالات ACE مع محلل
مرجعي، وتطابق نوع كل ACE وأعلامه وقناعه وSID وأنواع كائناته مع `dissect.ntfs`. وكل
تجزئة مسجلة هي تجزئة واصفها، وكل نسخة مرآة تطابق إدخالها، ولكل واصف
تمثيل SDDL، بما في ذلك شروط إدخالات ACE من نوع callback البالغ عددها 3,384.

## SRUM { #srum }

يعتمد [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) على حزمتي
[ese](https://github.com/Sootmark/ese) و[srum](https://github.com/Sootmark/srum) من Sootmark. وعبر
11 قاعدة بيانات حقيقية، تطابق كل عمود في الإدخالات المُخرجة البالغة 45,255 إدخالًا مع فك ترميز مستقل
للقيم التي يكشفها `dissect.esedb`.

## اختصارات LNK { #lnk-shortcuts }

يُعد [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) تفرّعًا مُصانًا من حزمة
[shell](https://github.com/Sootmark/shell) الخاصة بـ Sootmark. وقد قورن عبر 1,118 اختصارًا
عامًا. ومن بين أعمدة CSV الثلاثين المشتركة مع المحلل المرجعي، اختلفت 16 قيمة فردية:
13 مسار هدف تتضمن عناصر تالفة، أو مبتورة، أو عناصر نائبة، أو MTP، أو لوحة التحكم؛ وقائمتان
لكتل البيانات الإضافية تتضمنان كتلًا غير معروفة أو مبتورة؛ ومرجع MFT واحد قرأه
المحلل المرجعي من بايتات مختلفة. ويُبلغ Juntei عن البنى التالفة بدلًا من تطبيعها
بصمت.

## Jump Lists { #jump-lists }

يستخدم [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) أعمال Sootmark
حول الملفات المركبة وJump Lists، مع فك ترميز الروابط المضمّنة بواسطة `ys-lnk-parser`. وعبر 77 ملف
Jump Lists عامًا، تطابقت كل خلايا المقارنة المشتركة البالغة 13,057 خلية، بما في ذلك الطوابع الزمنية حتى 100 نانوثانية، مع
المحلل المرجعي. أما الملفات الستة التي رفضها المرجع (قوائم مخصصة فارغة، أو تذييل مفقود، أو ملف
ليس من Jump Lists) فقد فك Juntei ترميزها أو أبلغ عنها صراحةً.

## ShimCache { #shimcache }

يُعد [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) تفرّعًا مُصانًا
من [shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) الخاص بـ Security Ronin، مع دعم لخلايا سجل Windows
وسجلات المعاملات. وعلى 20 خلية `SYSTEM` عامة (24 مجموعة تحكم و7,290 إدخالًا) إضافةً إلى 17
قيمة AppCompatCache مستقلة، تطابق كل إدخال في المسار، والوقت، وعلم التنفيذ، والأعلام، و
الموضع مع ما أخرجه المحلل المرجعي.

## حجر Microsoft Defender { #microsoft-defender-quarantine }

يُعد [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
تفرّعًا مُصانًا من حزمة
[defender](https://github.com/Sootmark/defender) الخاصة بـ Sootmark. ويفك قارئ الحجر المستقل في
`dissect.target` ترميز مجلد حجر كتبته أدوات اختبار المحلل بتنسيق Defender
بشكل متسق.

## الآثار الرقمية المستمدة من سجل Windows { #registry-derived-artifacts }

يُقارن [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) مع
[yarp](https://github.com/msuhanov/yarp) فيما يخص المفاتيح المخصصة، والقيم، وإعادة تشغيل معاملات الخلايا غير المتسقة،
واستعادة الخلايا الحرة، والمساحة الراكدة للخلايا، والبيانات الموجودة خارج حاويات الخلية (hive bins)، والسجلات التي أزالتها إعادة التشغيل أو استبدلتها.
وتشمل المقارنات الخاصة بكل أثر رقمي:

- الخدمات، والقيم، وإجراءات الفشل من خلايا `SYSTEM` لـ Windows 7 وWindows 11 غير المتسقة؛
- RecentDocs بعد فك ترميزها باستخدام yarp وlibfwsi عبر تسع خلايا `NTUSER.DAT` عامة؛
- المهام المجدولة بعد فك ترميزها بشكل مستقل باستخدام yarp عبر سبع خلايا `SOFTWARE` عامة تغطي
  Windows 7–11، وServer 2012 R2، وServer 2022؛ و
- بنى حسابات SAM مقارنةً بـ `dissect.target` عبر سبع خلايا `SAM` عامة.

راجع [شكر وتقدير](../acknowledgements.md) للاطلاع على القائمة الكاملة للأدوات المرجعية، وأبحاث
التنسيقات، والمواصفات، ومصادر الأدلة العامة.
