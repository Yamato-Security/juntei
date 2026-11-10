# Persistence Hunter { #persistence-hunter }

الاستمرارية (persistence) طريقة لجعل الشيفرة تعمل مجددًا: عند بدء تشغيل Windows، أو تسجيل دخول
مستخدم، أو فتح تطبيق، أو حلول موعد مجدول، أو وقوع حدث آخر. ويمكن للمهاجمين إساءة استخدام الإعدادات
نفسها التي تستخدمها البرامج المشروعة للتحديثات، ودعم الأجهزة، والأمان، والإدارة.

يفحص `juntei persistence-hunter` أدلة Windows الملتقطة دون اتصال من خلال **131 فحصًا** تغطي هذه
الآليات وما يرتبط بها من حالات شاذة في الحسابات/الحماية. وهو يقرأ خلايا سجل Windows، وملفات المهام،
والبرامج النصية، وإعدادات التطبيقات، وسجلات مختارة من قواعد البيانات.

تشرح هذه الصفحة كل فحص منفَّذ. وللاطلاع على الخيارات الكاملة وحقول الإخراج والملفات الشخصية
(profiles) ومعالجة الطوابع الزمنية، افتح [دليل الأمر](../commands/persistence-hunter.md). وتتضمن صفحة
[شكر وتقدير](acknowledgements.md) روابط إلى مشروع PyrsistenceSniper المكتوب بلغة Python وأداة
PersistenceSniper الأصلية المكتوبة بلغة PowerShell.

## بدء التحقيق { #start-an-investigation }

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o PersistenceHunter.csv --coverage Coverage.csv
```

استخدم `--min-severity info` للحصول على الجرد الكامل للمرشحين. القيمة الافتراضية للأمر اليدوي هي
`medium`؛ ويمكن لملفه الشخصي أن يصنّف التطابقات الجزئية مع قائمة السماح على أنها `LOW` حتى عندما يحول
غياب أدلة الحمولة أو الموقِّع دون إجراء مقارنة كاملة. ولا يحسم تطابقٌ مع قائمة السماح ولا إدخالٌ غير مألوف
ما إذا كان الشيء ضارًا. ويحتفظ `auto-parse` أصلًا بجميع مستويات الخطورة، ويكتب النتائج ضمن الفئة
`persistence/` لكل حاسوب.

للتركيز على آلية بعينها، انسخ **معرّف الفحص** الخاص بها مما يلي:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,scheduled_task_files --min-severity info -o focused.csv
```

يستخدم الأمر مجمّع عمّال مشتركًا افتراضيًا. أضف `-T 4` لاختيار أربعة عمّال، أو `-T 1` لفحص بخيط
تنفيذ واحد. ويمكن أن يحتوي الإدخال على عدة حواسيب؛ وتظل أدلتها منفصلة.

## قراءة النتائج بوصفها أدلة { #read-findings-as-evidence }

| السؤال | ما الذي يجب فحصه |
|---|---|
| ما الذي كُوِّن، وأين؟ | `CheckId` و`Path` و`Value` الكاملة، بما فيها الوسيطات. تتبّع `SourceFile` وصولًا إلى الخلية أو الملف أو قاعدة البيانات الملتقطة. |
| هل الهدف موجود في المجموعة المجمّعة؟ | `ResolvedPath` و`ResolutionStatus` و`Exists`. المراجع غير المجمّعة أو الملتبسة أو غير المدعومة مجهولة؛ ولا يستعيض Juntei عنها بملف من جهاز المحلل. |
| هل التكوين فعّال؟ | المشغّل الفعلي، وعلامات التفعيل، ومجموعة التحكم المختارة، وإصدار Windows/التطبيق، والمستخدم، والبنية المعمارية. تحتفظ بعض الفحوص عمدًا بالتكوين غير النشط. |
| هل هذا هجوم؟ | يشرح `Severity` و`PolicyReason` مطابقة الملف الشخصي، لا الحكم النهائي. قارن مع البرامج المتوقعة، والأمر الكامل، وتجزئات الملفات، وأدلة التنفيذ المستقلة، والجدول الزمني المحيط. |
| من يمكنه تشغيله؟ | `AccessGained` تلميح إلى النطاق/السياق. كثيرًا ما يشير التكوين على مستوى الجهاز إلى `SYSTEM`، لكن ذلك لا يثبت أن العملية التي جرى تشغيلها عملت بصلاحيات SYSTEM. وتوفر الجهات الأساسية للمهام (task principals) ومعرّفات SID لمالكي مهام BITS سياقًا أكثر تحديدًا حيثما توفرت. |
| متى تغيّر؟ | يؤرخ `KeyLastWrite` مفتاح سجل Windows بأكمله. وتصف `LastChange` و`ChangeSource` و`ChangeEvidence` ربطًا منفصلًا مع MFT/EVTX عند توفره. ولا يؤرخ أيٌّ من الحقلين تنفيذ الحمولة تلقائيًا. |
| ما مدى موثوقية الحالة المستردة؟ | اقرأ `RecordStatus` و`Coverage.csv` والتحذيرات المتعلقة بالأدلة المعاد تطبيقها أو المستخلصة أو التالفة أو المفقودة أو المحدودة. وقد لا يكون السجل التاريخي المسترد نشطًا بعد الآن. |
| هل الناشر المسمى جدير بالثقة؟ | `Signer` و`SignerStatus` بيانات وصفية توصيفية لملف PE/الشهادة/الكتالوج. ولا يتحقق Juntei من صحة توقيعات Authenticode أو من الثقة في الشهادات. |

لا يثبت غياب النتائج غياب الاستمرارية. فصف التغطية `complete` يعني أن المصدر المجمّع المعني قد قُرئ،
لا أن عملية الاقتناء تحتوي على كل ملف ذي صلة، أو كل مستخدم، أو كل آلية استمرارية ممكنة في Windows.
وتشرح حالات التغطية الأخرى الفحوص المفقودة أو الجزئية أو المعطلة أو الفاشلة. ولا يُعد هذا الأمر فحصًا
للذاكرة الحية أو الشبكة أو الهوية السحابية.

## الفهرس وتدوين المواقع { #catalogue-and-location-notation }

يستند الفهرس إلى PyrsistenceSniper، إضافةً إلى الفحص الأصلي `start_menu_com`؛
وتوثّق الأوصاف أدناه
سلوك Juntei الأصلي، بما في ذلك قيود أدلته. وهناك **116 فحصًا تعتمد على سجل Windows و15 فحصًا
للملفات/المستودعات**. وتربط بعض فحوص الملفات بيانات سجل Windows أيضًا، لذا فهذه أعداد تنفيذية وليست
متطلبات اقتناء متنافية. وقد جُمِّعت الآليات أدناه حسب الغرض التحقيقي، ورُتِّبت أبجديًا حسب معرّف الفحص
داخل كل مجموعة.

- تشير `HKLM\SYSTEM` و`HKLM\SOFTWARE` و`HKLM\SAM` إلى خلايا الجهاز الملتقطة، التي تُجمع عادةً
  من `Windows\System32\config`، مع سجلات المعاملات عند توفرها.
- يعني `HKU\<user>` خلية المستخدم الملتقطة المعنية. تأتي معظم إعدادات المستخدم من NTUSER.DAT؛
  وقد تأتي تسجيلات الفئات (class registrations) من `AppData\Local\Microsoft\Windows\UsrClass.dat` كما هو مذكور.
- يعني `<ControlSet>` مجموعة التحكم النشطة المختارة في SYSTEM. وتُذكر المواقع التاريخية الإضافية لفحص
  DSRM صراحةً أدناه. تحقق من `SYSTEM\Select` عند مراجعتها.
- `<name>` مكوّن متغير في المسار. وفي جداول أهداف سجل Windows، يعني `<child>` التسجيلات الفرعية
  المباشرة؛ ولا يعني استعراضًا تكراريًا غير محدود. وتصف الفحوص التكرارية الصريحة طريقة استعراضها بشكل
  منفصل. وتشير الأقواس المعقوفة في مسارات Office إلى إصدار اختياري.
- تسرد الجداول طرق عرض الجهاز والمستخدم الفعلية التي يختارها الفحص. ويُذكر عرض 32 بت حيثما نُفِّذ؛
  فلا تفترض أن كل فحص يتناول كل بنية معمارية/طريقة عرض ممكنة.
- تُخرج فحوص القيم العامة الإدخالات المكوَّنة غير الفارغة، بما فيها عناصر السلاسل المتعددة والقيمة
  الرقمية صفر حيثما انطبق ذلك. ولا تشترط بحد ذاتها وجود حمولة ضارة، أو مشغّل مفعّل، أو تنفيذ ناجح.
  وتُذكر الشروط الأكثر انتقائية لكل فحص على حدة.

معرّفات ATT&CK تعيينات موروثة من الفهرس وروابط لمزيد من الدراسة. وهي تصف إساءة استخدام محتملة، لا
نتيجةً بأن مهاجمًا نفّذ تلك التقنية؛ وقد تصف فحوص إعدادات الحماية والفحوص التشخيصية شروطًا ممكِّنة بدلًا
من آلية استمرارية قائمة بذاتها.

| مجموعة الآليات | الفحوص |
|---|---:|
| [الإقلاع وتسجيل الدخول وبدء التشغيل](#boot-sign-in-and-startup) | 32 |
| [الخدمات وامتدادات النظام](#services-and-system-extensions) | 13 |
| [المهام المجدولة](#scheduled-tasks) | 3 |
| [WMI وBITS ومشغّلات الأحداث الأخرى](#wmi-bits-and-other-event-triggers) | 11 |
| [المستكشف وCOM ومعالجات الملفات أو عناوين URL](#explorer-com-and-file-or-url-handlers) | 15 |
| [تطبيقات Office والوظائف الإضافية](#office-applications-and-add-ins) | 7 |
| [مصححات الأخطاء ومعالجات الأعطال ورقع التوافق](#debuggers-crash-handlers-and-compatibility-shims) | 13 |
| [إعدادات وقت تشغيل .NET وأداة التوصيف](#net-runtime-and-profiler-settings) | 7 |
| [تجاوزات البحث عن ملفات DLL والملفات التنفيذية](#dll-and-executable-lookup-overrides) | 18 |
| [الحسابات وظهورها عند تسجيل الدخول](#accounts-and-sign-in-visibility) | 4 |
| [مكونات المصادقة وحماية بيانات الاعتماد](#authentication-components-and-credential-protection) | 8 |
| **الإجمالي** | **131** |

## الإقلاع وتسجيل الدخول وبدء التشغيل { #boot-sign-in-and-startup }

تُعِدّ هذه الإعدادات أمرًا أو مكتبة للتشغيل أثناء الإقلاع أو تسجيل الدخول أو تهيئة سطح المكتب. راجع إصدار Windows والمستخدم المعني: فقد يكون الإعداد متقادمًا أو معطلًا أو مخصصًا لجهاز كشك (kiosk) لا لمهاجم.

### Active Setup {#active_setup}

**معرّف الفحص:** `active_setup` · **ATT&CK في الفهرس:** [T1547.014](https://attack.mitre.org/techniques/T1547/014/)

**ما يفحصه Juntei:** يقرأ `StubPath` من تسجيلات مكونات Active Setup على مستوى الجهاز في العرض الأصلي وعرض 32 بت. ويمكن لـ Windows استخدام هذه الأوامر عند إعداد ملف تعريف المستخدم؛ ويتخطى الفحص قيم العناصر النائبة الحرفية `U` و`/UserInstall`.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Active Setup\Installed Components\<component>\StubPath` والشجرة المقابلة في `Wow6432Node`.

**كيفية التحقيق:** قارن كل مكوّن بالمنتج المثبت المقابل له، وافحص وسيطات الأمر. يمكن أن يؤدي التسجيل على مستوى الجهاز إلى التنفيذ في جلسة المستخدم الذي يسجل الدخول؛ وتصف التسمية `SYSTEM` في الإخراج نطاق التكوين، ولا تُعد دليلًا على رمز الوصول (token) للعملية الناتجة.

### تنفيذ الإقلاع (Boot Execute) {#boot_execute}

**معرّف الفحص:** `boot_execute` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يقرأ السلسلة المتعددة `BootExecute` في Session Manager، مع الحفاظ على كل إدخال غير فارغ على حدة. ويسجل الأوامر المكوَّنة لمرحلة إقلاع مبكرة.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `BootExecute` |

**كيفية التحقيق:** `autocheck autochk *` قيمة افتراضية مألوفة، ويمكن لأدوات الأقراص إضافة إدخالات مشروعة. تحقق من الأوامر المضافة وملفاتها دون افتراض أن كل إدخال غير افتراضي قد نُفِّذ أو أنه ضار.

### برنامج التحقق من الإقلاع (Boot Verification Program) {#boot_verification_program}

**معرّف الفحص:** `boot_verification_program` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يقرأ `ImagePath` الخاصة بـ `BootVerificationProgram`، وهي موقع ملف تنفيذي مرتبط بالتحقق من الإقلاع.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\BootVerificationProgram` | `ImagePath` |

**كيفية التحقيق:** تحقق مما إذا كانت هذه الآلية القديمة تنطبق على إصدار Windows الملتقط وتكوين الإقلاع. حدّد الملف الهدف وقارن غرضه ببرامج إدارة الإقلاع المتوقعة.

### قيمة Load في المستكشف {#explorer_load}

**معرّف الفحص:** `explorer_load` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يقرأ القيمة `Load` المرتبطة بتكوين تهيئة تسجيل الدخول إلى Windows من خلايا الجهاز والمستخدم.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |

**كيفية التحقيق:** تحقق من الأوامر غير المألوفة، والوجهات القابلة للكتابة من قِبل المستخدم، والوسيطات التي تجلب محتوى أو تفك ترميزه. تأكد من قابلية التطبيق ومن التنفيذ كلٌّ على حدة؛ ولا تغطي النتيجة الفارغة آليات تسجيل الدخول الأخرى.

### برامج تشغيل الخطوط (Font Drivers) {#font_drivers}

**معرّف الفحص:** `font_drivers` · **ATT&CK في الفهرس:** [T1547](https://attack.mitre.org/techniques/T1547/)

**ما يفحصه Juntei:** يعدّد القيم غير الفارغة في مفتاح سجل Windows القديم `Font Drivers` بوصفها تسجيلات محتملة لمكتبات.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Font Drivers` | جميع القيم غير الفارغة |

**كيفية التحقيق:** تعامل مع هذا بوصفه سطح تكوين قديمًا وغير معتاد. تحقق من دعم إصدار Windows ومن المكتبة المشار إليها قبل استنتاج أنها قد تُحمَّل، وعزّز الاستخدام الفعلي بأدلة العمليات أو تحميل الصور (image-load).

### ملفات DLL لامتدادات نهج المجموعة {#gp_extension_dlls}

**معرّف الفحص:** `gp_extension_dlls` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ `DllName` من كل تسجيل مباشر لامتداد نهج المجموعة (Group Policy) ضمن Winlogon. وتوسّع ملفات DLL هذه معالجة النهج.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\GPExtensions\<child>` | `DllName` |

**كيفية التحقيق:** يمكن لمنتجات الإدارة والأمان تثبيت امتدادات. طابق GUID الامتداد وملف DLL مع المنتج المتوقع، وافحص أدلة معالجة النهج وتحميل الصور قبل الجزم بحدوث التنفيذ.

### البرامج النصية لنهج المجموعة {#gp_scripts}

**معرّف الفحص:** `gp_scripts` · **ATT&CK في الفهرس:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**ما يفحصه Juntei:** يحلل أزواج الأمر/المعامل من ملفات تسجيل نهج المجموعة المحلي الملتقطة `scripts.ini` و`psscripts.ini`، بما في ذلك مخازن النهج المحلي لكل مستخدم. وتحتفظ أقسام بدء التشغيل/إيقاف التشغيل وتسجيل الدخول/تسجيل الخروج بأوامرها ووسيطاتها المكوَّنة، مع تمييز أقسام الجهاز عن أقسام المستخدم.

**الأدلة المفحوصة:**

- `Windows\System32\GroupPolicy\Machine\Scripts\scripts.ini` / `psscripts.ini` والملفات المكافئة في `GroupPolicy\User\Scripts`.
- `Windows\System32\GroupPolicyUsers\<SID>\User\Scripts\scripts.ini` / `psscripts.ini`.

**كيفية التحقيق:** لا تُفحص إلا مواقع تسجيل النهج المحلي المنفَّذة؛ فهذا ليس تدقيقًا كاملًا لـ SYSVOL/GPO. ولا يكفي وجود ملف برنامج نصي في مكان آخر ضمن مجلد نهج. اربط تسجيله بأدلة معالجة النهج والتنفيذ.

### البرامج النصية لتسجيل الدخول (UserInitMprLogonScript) {#logon_scripts}

**معرّف الفحص:** `logon_scripts` · **ATT&CK في الفهرس:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**ما يفحصه Juntei:** يقرأ قيمة البيئة `UserInitMprLogonScript` الخاصة بالمستخدم، والتي يمكنها تحديد برنامج نصي لتسجيل دخول ذلك المستخدم.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKU\<user>\Environment` | `UserInitMprLogonScript` |

**كيفية التحقيق:** قارن مع البرامج النصية المعتمدة لتسجيل الدخول وافحص البرنامج النصي الفعلي. لا تثبت القيمة حدوث تسجيل دخول لاحق أو تنفيذ، وتتطلب البرامج النصية التي يوفرها النطاق أدلة نهج إضافية.

### تنفيذ المنصة (Platform Execute) {#platform_execute}

**معرّف الفحص:** `platform_execute` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يقرأ كل إدخال غير فارغ في `PlatformExecute` ضمن Session Manager. ويحدد البرامج المختارة لتهيئة الإقلاع المتعلقة بالمنصة.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `PlatformExecute` |

**كيفية التحقيق:** قد تفسر مسارات عمل العتاد أو التجهيز (provisioning) وجود إدخال ما. حدّد ما إذا كان مسار الإقلاع في إصدار Windows هذا يستخدمه، وتحقق من الصورة المشار إليها بدلًا من استنتاج هجوم من اسم القيمة.

### ملف DLL لامتداد عميل RDP {#rdp_clx_dll}

**معرّف الفحص:** `rdp_clx_dll` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يقرأ `ClxDllPath` في تكوين المستخدم الافتراضي لـ Terminal Server بوصفه موقعًا محتملًا لامتداد متعلق بـ RDP.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\DefaultUserConfiguration` | `ClxDllPath` |

**كيفية التحقيق:** تحقق من إصدار Windows/RDP والامتدادات المثبتة. يُعد ملف DLL المكوَّن خيطًا لتحليل تحميل الصور؛ ولا تُظهر قيمة سجل Windows وحدها حدوث اتصال RDP أو تحميل ملف DLL.

### ملف DLL للقناة الافتراضية في RDP {#rdp_virtual_channel}

**معرّف الفحص:** `rdp_virtual_channel` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يعدّد القيم ضمن تسجيلات `Default\AddIns` الخاصة بـ Terminal Server Client على مستوى الجهاز. وتُفك ترميز القيم الثنائية بصيغة UTF-16LE حيثما أمكن، مع الحفاظ على مسارات الامتدادات القابلة للاستخدام.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Terminal Server Client\Default\AddIns\<add-in>`: جميع القيم.

**كيفية التحقيق:** تسجّل عملاء RDP ومنتجات إعادة توجيه الأجهزة وظائف إضافية بصورة مشروعة. افحص ملف DLL أو الأمر الفعلي وجلسة العميل ذات الصلة؛ فليست كل قيمة ضمن وظيفة إضافية مسارًا لملف تنفيذي.

### برامج بدء التشغيل في RDP WDS {#rdp_wds_startup}

**معرّف الفحص:** `rdp_wds_startup` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يقرأ `StartupPrograms` من تكوين برنامج تشغيل Windows Station الخاص بـ RDP. ويحدد البرامج المكوَّنة لبدء تشغيل الجلسات البعيدة.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\Wds\rdpwd` | `StartupPrograms` |

**كيفية التحقيق:** `rdpclip` إدخال مألوف. تحقق من الإضافات ومن تكوين RDP على المضيف، واستخدم سجلات الجلسات/العمليات لتحديد ما إذا كان البرنامج المكوَّن قد بدأ فعلًا.

### مفاتيح Run في سجل Windows {#run_keys}

**معرّف الفحص:** `run_keys` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يعدّد قيم Run وRunOnce وRunEx وRunOnceEx وقيم Run الخاصة بنهج Explorer من خلايا الجهاز/المستخدم، إضافةً إلى مفاتيح الجهاز المدرجة في عرض 32 بت. وتُستعرض أقسام RunEx/RunOnceEx مع فحوص للحدود وللحلقات الدائرية.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | جميع القيم غير الفارغة |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | جميع القيم غير الفارغة |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | جميع القيم غير الفارغة |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | جميع القيم غير الفارغة |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | جميع القيم غير الفارغة |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | جميع القيم غير الفارغة |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | جميع القيم غير الفارغة |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | جميع القيم غير الفارغة |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | جميع القيم غير الفارغة |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | جميع القيم غير الفارغة |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Run` | جميع القيم غير الفارغة |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnce` | جميع القيم غير الفارغة |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnceEx` | جميع القيم غير الفارغة |

**كيفية التحقيق:** تظهر هنا عادةً برامج تحديث البرامج وتطبيقات منطقة الإعلام (tray). قارن الملف التنفيذي ووسيطات الأمر وسياق المستخدم. قد يختفي إدخال RunOnce بعد استخدامه؛ ولا تُعد الحالة الملتقطة سجلًا كاملًا لعمليات تسجيل الدخول السابقة.

### RunServices (قديم) {#run_services}

**معرّف الفحص:** `run_services` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يعدّد القيم في المفتاح القديم `RunServices` على مستوى الجهاز.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServices` | جميع القيم غير الفارغة |

**كيفية التحقيق:** هذا سطح تكوين تاريخي، وليس ضمانًا بأن إصدارًا حديثًا من Windows ينفذه. تحقق من الهدف ومن انطباقه على نظام التشغيل، ثم ابحث عن أدلة تنفيذ مستقلة.

### RunServicesOnce (قديم) {#run_services_once}

**معرّف الفحص:** `run_services_once` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يعدّد القيم في المفتاح القديم `RunServicesOnce` على مستوى الجهاز.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServicesOnce` | جميع القيم غير الفارغة |

**كيفية التحقيق:** تعامل مع القيم غير الفارغة بوصفها خيوطًا تكوينية. تحقق مما إذا كان إصدار Windows المصدر يدعم الآلية، وما إذا كانت برامج النشر أو التوافق تفسر وجود الإدخال.

### الأمر الأولي للجلسة S0 {#s0_initial_command}

**معرّف الفحص:** `s0_initial_command` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يقرأ `S0InitialCommand` من Session Manager بوصفه أمرًا محتملًا لتهيئة الجلسة 0 (Session 0).

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `S0InitialCommand` |

**كيفية التحقيق:** تأكد من انطباقه على إصدار Windows الملتقط وحالة الإقلاع. فالأمر المكوَّن ليس سجلًا مؤرخًا لتنفيذ عند الإقلاع؛ افحص ملفه واربطه ببيانات القياس عن بُعد الخاصة بالإقلاع/العمليات.

### تنفيذ Session Manager {#session_manager_execute}

**معرّف الفحص:** `session_manager_execute` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يقرأ إدخالات `Execute` غير الفارغة من Session Manager، كاشفًا البرامج المختارة لتهيئة الجلسة.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `Execute` |

**كيفية التحقيق:** قارن مع خط الأساس المتوقع للصور وافحص الملفات التنفيذية الأصلية (native) أو المسارات غير المعتادة. تأكد مما إذا كان إقلاع/جلسة ذات صلة قد حدث بعد التغيير قبل الجزم بالتنفيذ.

### الأنظمة الفرعية في Session Manager {#session_manager_subsystems}

**معرّف الفحص:** `session_manager_subsystems` · **ATT&CK في الفهرس:** [T1547](https://attack.mitre.org/techniques/T1547/)

**ما يفحصه Juntei:** يقرأ القيمة `Windows` في المفتاح `SubSystems` الخاص بـ Session Manager، والتي تكوّن أمر النظام الفرعي لـ Windows.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\SubSystems` | `Windows` |

**كيفية التحقيق:** الأمر العادي طويل ويعتمد على الإصدار. قارن القيمة الكاملة ووسيطاتها بخط أساس موثوق من الإصدار نفسه؛ كما أن تغيير إعداد نظام فرعي حرج قد يعطّل بدء التشغيل.

### تنفيذ الإعداد (Setup Execute) {#setup_execute}

**معرّف الفحص:** `setup_execute` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يقرأ إدخالات `SetupExecute` في Session Manager، والتي تغطي الأوامر المتعلقة بالإعداد المكوَّنة لبدء التشغيل.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `SetupExecute` |

**كيفية التحقيق:** قد تنشئ أنشطة التثبيت والترقية والاسترداد إدخالات مشروعة. قارن مع سجلات الإعداد والصورة المشار إليها قبل نسب التكوين إلى الاستمرارية.

### إعادة توجيه مجلد Startup عبر Shell Folders {#shell_folders_startup}

**معرّف الفحص:** `shell_folders_startup` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يقارن `Startup` و`Common Startup` في كل من Shell Folders وUser Shell Folders بمواقع Startup الافتراضية. ويُبلَّغ عن عمليات إعادة التوجيه غير الافتراضية، ويُفحص محتوى مجلداتها المجمّع.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders` و`User Shell Folders`: `Common Startup`.
- إعدادات `Software\Microsoft\Windows\CurrentVersion\Explorer` المكافئة في NTUSER.DAT: `Startup`، إضافةً إلى الملفات الملتقطة في المواقع المعاد توجيهها.

**كيفية التحقيق:** يمكن لمجلد Startup معاد توجيهه أن يخفي عناصر التشغيل التلقائي، لكن إعادة التوجيه قد تكون مشروعة. يجب أن يُحَل المسار ضمن الأدلة الملتقطة؛ فالوجهة غير المجمّعة مجهولة، وليست دليلًا على أن المجلد كان فارغًا.

### تجاوز مشغّل الواجهة (Shell Launcher) {#shell_launcher}

**معرّف الفحص:** `shell_launcher` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يقرأ قيمة النهج `Shell` وتعيين Shell للإقلاع في `system.ini` من SOFTWARE. وهذان سطحان بديلان لاختيار الواجهة.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `Shell` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\IniFileMapping\system.ini\boot` | `Shell` |

**كيفية التحقيق:** يمكن لأجهزة الكشك والأجهزة المخصصة لتطبيقات بعينها أن تستبدل Explorer عمدًا. قارن الواجهة المختارة بدور الجهاز وتأكد من أن الإعداد كان فعّالًا للجلسة المعنية.

### مجلد Startup {#startup_folder}

**معرّف الفحص:** `startup_folder` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يفحص الملفات المباشرة في مجلدات Startup الافتراضية لكل مستخدم والمشتركة، متبعًا أول إعادة توجيه متاحة لمجلد Startup. وتُسترد أهداف الاختصارات حين يمكن تحليل ملف `.lnk` مجمّع.

**الأدلة المفحوصة:**

- `ProgramData\Microsoft\Windows\Start Menu\Programs\Startup` و`Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`.
- عمليات إعادة توجيه `Common Startup` / `Startup` المنطبقة في إعدادات Shell Folders وUser Shell Folders للجهاز/المستخدم.

**كيفية التحقيق:** يمكن لارتباطات Startup وملفاته أن تشغّل تطبيقات مشروعة عند تسجيل الدخول. افحص وسيطات الهدف إلى جانب أسماء الملفات. المجلد المشترك موقع تكوين مشترك؛ ولا يثبت أن البرنامج الذي جرى تشغيله استخدم رمز وصول SYSTEM.

### البرنامج الأولي لخدمات Terminal Services {#ts_initial_program}

**معرّف الفحص:** `ts_initial_program` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يقرأ قيمة نهج Terminal Services `InitialProgram`، وتكوين Run/RunOnce/RunOnceEx لتثبيت Terminal Server على الجهاز، بما في ذلك أقسام RunOnceEx المتداخلة ضمن حدود.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` | `InitialProgram` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Run` | جميع القيم غير الفارغة |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Runonce` | جميع القيم غير الفارغة |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx` | جميع القيم غير الفارغة |

**كيفية التحقيق:** قد تستخدم عمليات نشر التطبيقات البعيدة هذه الإعدادات عمدًا. حدّد الجلسات المتأثرة وتتبّع الأوامر المختارة في أدلة العمليات؛ فالتسجيل وحده لا يُظهر حدوث تسجيل دخول عن بُعد.

### Winlogon AppSetup {#winlogon_appsetup}

**معرّف الفحص:** `winlogon_appsetup` · **ATT&CK في الفهرس:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**ما يفحصه Juntei:** يقرأ `AppSetup` من Winlogon، مقسّمًا الإدخالات المفصولة بفواصل إلى مرشحين منفصلين.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `AppSetup` |

**كيفية التحقيق:** قارن كل مكوّن ببرامج الإعداد/تسجيل الدخول المتوقعة في Windows. فإدخال القائمة ليس تنفيذًا منفصلًا مُلاحَظًا، وقد تعتمد أهميته على إصدار Windows وحالة النشر.

### Winlogon GinaDLL {#winlogon_gina_dll}

**معرّف الفحص:** `winlogon_gina_dll` · **ATT&CK في الفهرس:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**ما يفحصه Juntei:** يقرأ إعداد الاستبدال القديم `GinaDLL` في Winlogon. وكانت GINA نموذجًا أقدم لامتدادات تسجيل الدخول إلى Windows.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `GinaDLL` |

**كيفية التحقيق:** يتعلق هذا أساسًا بالأنظمة السابقة لـ Vista. ولا يُعد مفتاح متبقٍّ على نظام أحدث دليلًا على امتداد فعّال لتسجيل الدخول؛ افحص إصدار المصدر وأي ملف DLL مشار إليه.

**مزيد من التفاصيل:** [وثائق Microsoft](https://learn.microsoft.com/en-us/windows/win32/secauthn/gina).

### Winlogon MPNotify {#winlogon_mpnotify}

**معرّف الفحص:** `winlogon_mpnotify` · **ATT&CK في الفهرس:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**ما يفحصه Juntei:** يقرأ إعداد الأمر/المكتبة `mpnotify` في Winlogon، وهو سطح امتداد متعلق بتسجيل الدخول.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `mpnotify` |

**كيفية التحقيق:** قارن الهدف والوسيطات بخط أساس مطابق من Windows. تحقق من السلوك الخاص بالإصدار واربط ذلك بتسجيل دخول لاحق قبل الجزم بأن الشيفرة قد عملت.

### حزم الإشعارات في Winlogon {#winlogon_notify_packages}

**معرّف الفحص:** `winlogon_notify_packages` · **ATT&CK في الفهرس:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**ما يفحصه Juntei:** يقرأ `DllName` من كل تسجيل مباشر لحزمة `Notify` في Winlogon.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Notify\<child>` | `DllName` |

**كيفية التحقيق:** حزم الإشعارات آلية امتداد قديمة. حدّد ما إذا كان إصدار Windows الملتقط يحمّلها، وميّز برامج المصادقة المعتمدة عن المكتبات غير المبررة.

### Winlogon Shell {#winlogon_shell}

**معرّف الفحص:** `winlogon_shell` · **ATT&CK في الفهرس:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**ما يفحصه Juntei:** يقرأ قيم `Shell` في Winlogon على مستوى الجهاز ولكل مستخدم للكشف عن اختيار واجهة سطح المكتب.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |

**كيفية التحقيق:** Explorer هو الشائع، لكن أجهزة الكشك قد تستخدم واجهة أخرى. تحقق من الأوامر المضافة أو المسارات غير المعتادة، وحدّد التكوين الفعّال لأي مستخدم طُبِّق عند تسجيل الدخول.

### Winlogon System {#winlogon_system}

**معرّف الفحص:** `winlogon_system` · **ATT&CK في الفهرس:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**ما يفحصه Juntei:** يقرأ القيمة `System` في Winlogon ويفصل الإدخالات المحددة بفواصل إلى مرشحين منفردين.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `System` |

**كيفية التحقيق:** تعامل مع هذا بوصفه خيطًا تكوينيًا لتسجيل الدخول حساسًا للإصدار. افحص كل مكوّن مشار إليه وعزّز ذلك بأدلة التنفيذ بدلًا من التعامل مع القائمة كسجل تشغيل.

### Winlogon Taskman {#winlogon_taskman}

**معرّف الفحص:** `winlogon_taskman` · **ATT&CK في الفهرس:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**ما يفحصه Juntei:** يقرأ قيم `Taskman` في Winlogon للجهاز/المستخدم بحثًا عن أمر بديل لإدارة المهام.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |

**كيفية التحقيق:** حدّد سلوك Windows الذي سيستخدم هذا الإعداد، وما إذا كان الهدف متوقعًا. فالبديل المكوَّن ليس دليلًا على أن مستخدمًا استدعى تلك الميزة.

### Winlogon Userinit {#winlogon_userinit}

**معرّف الفحص:** `winlogon_userinit` · **ATT&CK في الفهرس:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**ما يفحصه Juntei:** يقرأ قيم `Userinit` في Winlogon للجهاز/المستخدم، مقسّمًا البرامج المفصولة بفواصل مع تجاهل العناصر الفارغة في النهاية.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |

**كيفية التحقيق:** كثيرًا ما يشير الإدخال العادي إلى `userinit.exe`. افحص الإدخالات الإضافية والمسارات المعدّلة. تأكد من المستخدم المعني ومن حدوث تسجيل دخول لاحق؛ إذ يخص `KeyLastWrite` المفتاح بأكمله، لا هذه القيمة تحديدًا.

### Winlogon VMApplet {#winlogon_vmapplet}

**معرّف الفحص:** `winlogon_vmapplet` · **ATT&CK في الفهرس:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**ما يفحصه Juntei:** يقرأ إعداد الأمر `VMApplet` في Winlogon، والذي يغطي سطحًا أقدم لتشغيل بريمجات النظام (applets).

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `VMApplet` |

**كيفية التحقيق:** تحقق من انطباقه على نظام التشغيل ومن البريمج المتوقع. يستحق الأمر غير المتوقع التحقيق، لكنه لا يثبت أن الواجهة المشغِّلة قد فُتحت.

## الخدمات وامتدادات النظام { #services-and-system-extensions }

توفر خدمات Windows ومكونات النظام القابلة للتوسيع طرقًا مشروعة كثيرة لتحميل الملفات التنفيذية أو ملفات DLL. قارن الهدف المكوَّن بالمنتج المتوقع، وحساب الخدمة، وإعدادات البدء، وسجل التثبيت المدعوم بالمصادر.

### ملف DLL لـ DiagTrack {#diagtrack_dll}

**معرّف الفحص:** `diagtrack_dll` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ `ImagePath` الخاصة بخدمة DiagTrack. وعلى الرغم من أن اسم الفحص في الفهرس يوحي بملفات DLL، فإنه يفحص أمر صورة الخدمة المكوَّن.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DiagTrack` | `ImagePath` |

**كيفية التحقيق:** قارن مع خط الأساس للخدمة ومع فحوص الخدمات العامة. وقد ينتج التكوين نفسه أكثر من نتيجة فحص واحدة؛ فأعداد المرشحين ليست أعداد هجمات متمايزة.

### ملف DLL لمستمع DiagTrack {#diagtrack_listener_dll}

**معرّف الفحص:** `diagtrack_listener_dll` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ `FileName` من تكوين المسجّل التلقائي DiagTrack-Listener. وهذا إعداد لمسجّل ETW تلقائي، وليس دليلًا على تحميل ملف DLL.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\WMI\Autologger\DiagTrack-Listener` | `FileName` |

**كيفية التحقيق:** افحص القيمة الخام وتكوين التسجيل في Windows. يصنّف الفهرس هذا بوصفه تقنية DLL، لكن البحث لا يتحقق من وجود ملف DLL قابل للتحميل ولا يثبت التنفيذ عبر هذه القيمة.

### ملف DLL مساعد لـ Netsh {#netsh_helper}

**معرّف الفحص:** `netsh_helper` · **ATT&CK في الفهرس:** [T1546.007](https://attack.mitre.org/techniques/T1546/007/)

**ما يفحصه Juntei:** يعدّد القيم غير الفارغة ضمن مفتاح تسجيل المساعدات `NetSh` على الجهاز. ويمكن للمساعدات أن توسّع `netsh.exe` بأوامر قائمة على ملفات DLL.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\NetSh` | جميع القيم غير الفارغة |

**كيفية التحقيق:** قد تثبّت منتجات الشبكات مساعدات. طابق كل قيمة مع منتج متوقع وافحص المكتبة المحددة، ثم ابحث عن تنفيذ netsh أو عمليات تحميل صور ذات صلة.

### مراقبات الطباعة (Print Monitors) {#print_monitors}

**معرّف الفحص:** `print_monitors` · **ATT&CK في الفهرس:** [T1547.010](https://attack.mitre.org/techniques/T1547/010/)

**ما يفحصه Juntei:** يقرأ `Driver` ضمن كل تسجيل مباشر لمراقب طباعة في مجموعة التحكم النشطة.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Monitors\<child>` | `Driver` |

**كيفية التحقيق:** توسّع برامج الطباعة ومراقبات المنافذ خدمة التخزين المؤقت للطباعة (spooler) بصورة مشروعة. قارن ملف DLL والبيانات الوصفية للمنتج بجرد الطابعات، وتأكد من التحميل بأدلة الـ spooler/تحميل الصور.

### معالجات الطباعة (Print Processors) {#print_processors}

**معرّف الفحص:** `print_processors` · **ATT&CK في الفهرس:** [T1547.012](https://attack.mitre.org/techniques/T1547/012/)

**ما يفحصه Juntei:** يقرأ `Driver` ضمن شجرتي تسجيل معالجات الطباعة لبنيتي x64 وx86.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows x64\Print Processors\<child>` | `Driver` |
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows NT x86\Print Processors\<child>` | `Driver` |

**كيفية التحقيق:** يمكن لحزم الطابعات من جهات خارجية إضافة معالجات تتجاوز الإعدادات الافتراضية المألوفة في Windows. راجع كلتا البنيتين، وملف DLL الفعلي، وبرامج الطابعات المثبتة قبل نسب أي نية ضارة.

### امتداد Service Control Manager {#scm_extension}

**معرّف الفحص:** `scm_extension` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يعدّد القيم غير الفارغة في `ServiceControlManagerExtension`، وهو سطح امتداد لمدير الخدمات.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\ServiceControlManagerExtension` | جميع القيم غير الفارغة |

**كيفية التحقيق:** افحص المكتبات المسماة وما إذا كان إصدار Windows هذا يستخدم الإعداد. اربط ذلك بنشاط الإقلاع/الخدمات؛ فالفحص يجرد التسجيلات ولا يثبت تحميل امتداد.

### ملف DLL لمكوّن إضافي على مستوى خادم DNS {#server_level_plugin_dll}

**معرّف الفحص:** `server_level_plugin_dll` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ إعداد `ServerLevelPluginDll` لخدمة DNS في مجموعة التحكم النشطة في SYSTEM.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DNS\Parameters` | `ServerLevelPluginDll` |

**كيفية التحقيق:** يتعلق هذا بالأنظمة التي تشغّل دور DNS Server، بما فيها بعض وحدات التحكم بالمجال. تحقق من ملف DLL وحساب الخدمة وأدلة إعادة تشغيل الخدمة/التحميل؛ فالدور غير النشط أو الغائب يغيّر الاستنتاج.

### أمر فشل الخدمة {#service_failure_command}

**معرّف الفحص:** `service_failure_command` · **ATT&CK في الفهرس:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**ما يفحصه Juntei:** يقرأ `FailureCommand` لكل خدمة في مجموعة التحكم النشطة، كاشفًا أمر الاسترداد المكوَّن.

**الأدلة المفحوصة:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\FailureCommand`.

**كيفية التحقيق:** لا يعمل الأمر إلا عندما تختاره إجراءات الاسترداد للخدمة ويقع الفشل المعني. افحص تكوين الاسترداد وسجلات الفشل/العمليات؛ فالقيمة وحدها لا تثبت وجود إجراء استرداد نشط.

### ملف DLL لوكيل امتداد SNMP {#snmp_extension_agent}

**معرّف الفحص:** `snmp_extension_agent` · **ATT&CK في الفهرس:** [T1574](https://attack.mitre.org/techniques/T1574/)

**ما يفحصه Juntei:** يعدّد القيم ضمن مفتاح التسجيل `ExtensionAgents` لخدمة SNMP. وقد تحدد القيم موقعًا آخر في سجل Windows بدلًا من مسار مباشر لملف DLL.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\SNMP\Parameters\ExtensionAgents` | جميع القيم غير الفارغة |

**كيفية التحقيق:** تتبّع التسجيل غير المباشر يدويًا وصولًا إلى مكتبته، وتحقق من أن SNMP مثبت ومستخدم. ولا تُعد السلسلة غير المحلولة في هذا الإخراج دليلًا على أن ملف DLL مفقود أو ضار.

### موفرو الوقت (Time Providers) {#time_providers}

**معرّف الفحص:** `time_providers` · **ATT&CK في الفهرس:** [T1547.003](https://attack.mitre.org/techniques/T1547/003/)

**ما يفحصه Juntei:** يقرأ `DllName` من التسجيلات المباشرة لـ `TimeProviders` في خدمة Windows Time.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\W32Time\TimeProviders\<child>` | `DllName` |

**كيفية التحقيق:** راجع إعدادات تفعيل الموفر وحالة خدمة Windows Time كلًّا على حدة؛ إذ لا يشترط فحص مسار DLL أن يكون الموفر مفعّلًا. وقد توفر منتجات مزامنة الوقت بدائل مشروعة.

### خدمة Windows (ServiceDll) {#windows_service_dll}

**معرّف الفحص:** `windows_service_dll` · **ATT&CK في الفهرس:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**ما يفحصه Juntei:** يقرأ `Parameters\ServiceDll` ضمن الخدمات في مجموعة التحكم النشطة في SYSTEM، ويُرفق أدلة الطوابع الزمنية الخاصة بالخدمة عند توفرها.

**الأدلة المفحوصة:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Parameters\ServiceDll`.

**كيفية التحقيق:** المضيف المشترك مثل `svchost.exe` ليس ملف DLL الحمولة. افحص المكتبة، وحساب الخدمة، وحالة البدء، وأدلة تثبيت الخدمة/تحميل الصور المطابقة؛ فالتسجيل لا يثبت أن الخدمة قد عملت.

**مزيد من التفاصيل:** [وثائق Microsoft](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### خدمة Windows (ImagePath) {#windows_service_image_path}

**معرّف الفحص:** `windows_service_image_path` · **ATT&CK في الفهرس:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**ما يفحصه Juntei:** يقرأ أوامر `ImagePath` للخدمات في مجموعة التحكم النشطة. وتحتفظ أنواع برامج التشغيل 1 أو 2 أو 8 التي لا تحتوي على ImagePath بهدف ضمني هو `System32\drivers\<service>.sys`.

**الأدلة المفحوصة:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>`: `ImagePath` و`Type` والبيانات الوصفية الداعمة للخدمة.

**كيفية التحقيق:** راجع حساب الخدمة وإعدادات البدء بمعزل عن تسمية نطاق الجهاز في الإخراج. وتظل التسجيلات المعطلة أو التي تبدأ عند الطلب جزءًا من الجرد. وتُفضَّل الأدلة المستندة إلى اسم الخدمة على مسارات الملفات التنفيذية المشتركة الملتبسة عند تحديد الطوابع الزمنية.

**مزيد من التفاصيل:** [وثائق Microsoft](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### ملف DLL لبدء تشغيل خدمة Windows Update {#wu_service_startup_dll}

**معرّف الفحص:** `wu_service_startup_dll` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ `Parameters\ServiceDll` الخاصة بخدمة Windows Update من مجموعة التحكم النشطة.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\wuauserv\Parameters` | `ServiceDll` |

**كيفية التحقيق:** قارن المكتبة بإصدار Windows نفسه وحالة الصيانة (servicing) المثبتة. يتداخل هذا مع الجرد العام لـ ServiceDll ولا يمثل اختراقًا ثانيًا متمايزًا.

## المهام المجدولة { #scheduled-tasks }

يخزن Task Scheduler التكوين في ملفات XML وفي خلية SOFTWARE معًا. اقرأ هذين المصدرين معًا. يمكن تسجيل مهمة دون أن تكون قد عملت، ويمكن لمجموعة أدلة غير مكتملة أن تشبه مهمة مخفية أو شبحية.

### مهمة مجدولة شبحية {#ghost_task}

**معرّف الفحص:** `ghost_task` · **ATT&CK في الفهرس:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**ما يفحصه Juntei:** يقارن تسجيلات `Tree` و`Tasks` في TaskCache ضمن SOFTWARE، وعندما توجد ملفات مهام في المجموعة المجمّعة، يبحث عن ملفات XML المقابلة لها.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree` و`TaskCache\Tasks`.
- ملفات `Windows\System32\Tasks\<task path>` الملتقطة للمقارنة مع نظام الملفات.

**كيفية التحقيق:** قد يعكس غياب النظائر مهامًا محذوفة/قديمة، أو تسجيلات مدمجة، أو مجموعة أدلة غير مكتملة. ويعني عدم جمع ملفات XML أن المقارنة مع نظام الملفات غير متاحة. افحص التغطية والتعارض الدقيق قبل وصف مهمة بأنها مخفية أو ضارة.

### مهمة مجدولة مخفية {#hidden_scheduled_task}

**معرّف الفحص:** `hidden_scheduled_task` · **ATT&CK في الفهرس:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**ما يفحصه Juntei:** يفحص تسجيلات Tree في TaskCache بحثًا عن غياب القيمة `SD` أو عن واصف أمان ثنائي فارغ، وهي حالات مرتبطة بإخفاء مهمة عن التعداد العادي.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\<task>`: وجود `SD`/بياناتها، مع البيانات الوصفية للنظير في TaskCache.

**كيفية التحقيق:** غياب SD أو فراغه حالة شاذة محددة في التكوين، وليس دليلًا على أن المهمة نُفِّذت أو أنها ضارة. إعداد `Hidden` في XML ومشكلات قوائم ACL العشوائية حالات مختلفة، وليست الشرط المستخدم هنا.

**مزيد من التفاصيل:** [وثائق Microsoft](https://www.microsoft.com/en-us/security/blog/2022/04/12/tarrask-malware-uses-scheduled-task-defense-evasion/).

### مهمة مجدولة (ملفات XML) {#scheduled_task_files}

**معرّف الفحص:** `scheduled_task_files` · **ATT&CK في الفهرس:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**ما يفحصه Juntei:** يحلل ملفات XML الملتقطة للمهام بحثًا عن أوامر Exec ووسيطاتها، ومعرّفات فئات معالجات COM وبياناتها. ويختار الجهة الأساسية (principal) بحسب `Actions Context`/`Principal id`، ويحتفظ بهوية المستخدم والمجموعة كلٌّ على حدة، ويُبلغ عن SYSTEM أو USER أو UNKNOWN. وتظل إجراءات COM غير المحلولة ظاهرة.

**الأدلة المفحوصة:**

- ملفات XML في `Windows\System32\Tasks\<task path>`، بما في ذلك Exec وComHandler وحقول الجهة الأساسية المتاحة.
- ملف `UsrClass.dat` الملتقط للمستخدم المطابق و`Software\Classes` في NTUSER، إضافةً إلى `Classes\CLSID` في SOFTWARE على مستوى الجهاز، في العرض الأصلي وعرض 32 بت.
- `ProfileList` الملتقط من SOFTWARE للجهات الأساسية المعرّفة بـ SID. وتُفضّل الجهات الأساسية المسماة مجلد `user.DOMAIN` مطابقًا وفريدًا، ثم لواحق المضيف المحلي الملتقطة، وأخيرًا المجلد العادي `user`. وتُوسم التطابقات صراحةً بأنها استدلالية؛ ويظل الالتباس الحقيقي دون حل ويُذكر في النتيجة والتغطية. ولا تلزم مفاتيح سجل Windows المتطايرة (volatile).
- `CodeBase` المُدار (بما فيه المفاتيح الفرعية للإصدارات)، وارتباطات AppID، و`ImagePath` لخدمة في SYSTEM عند وجود `LocalService` في AppID.

يتضمن مرشحو خوادم COM مصدرهم في سجل Windows ضمن `Description`. وتظل ارتباطات AppID
التي تقتصر على اسم الملف التنفيذي دون حل، بدلًا من إسناد مجلد مُخمَّن إليها. وتُفك ترميز عناوين
`file:` المحلية؛ بينما يُحتفظ بالعناوين البعيدة دون الوصول إلى الشبكة. ويقتصر نطاق تجاوزات فئات المستخدم
على ملف التعريف ذاك، ولا يرتد معرّف CLSID غير المكتمل للمستخدم إلى تسجيل الجهاز الذي يحجبه. وتحتفظ
المهام المقتصرة على مجموعة بمرشحي الجهاز مع ذكر قيد المستخدم الفردي في وصفها، دون إضافة إشعارات روتينية
إلى ملخص أخطاء التغطية.

**كيفية التحقيق:** افحص المشغّلات، وحالة التفعيل، والجهة الأساسية، وسجل التشغيل في ملف XML المصدر وسجلات الأحداث. قد لا تعمل مهمة مُنشأة أبدًا. ويظل معالج COM غير المحلول مرشحًا، ولا يجوز افتراض أن بيانات الجهة الأساسية المفقودة أو المقتصرة على مجموعة تعني SYSTEM.

## WMI وBITS ومشغّلات الأحداث الأخرى { #wmi-bits-and-other-event-triggers }

تدعم Windows Management Instrumentation (WMI) إدارة النظام. وتدعم Background Intelligent Transfer Service (BITS) عمليات النقل في الخلفية. ويمكن لكلتيهما الاحتفاظ بأوامر تُشغَّل بالأحداث. وتنتظر بعض الآليات حدثًا إداريًا، أو إشعار نقل، أو تشغيل تطبيق، أو إجراءً من المستخدم. والمشغّل المخزَّن دليل تكويني مفيد، لكن وجوده لا يثبت أن المشغّل قد انطلق.

### باب خلفي عبر ميزات إمكانية الوصول {#accessibility_tools}

**معرّف الفحص:** `accessibility_tools` · **ATT&CK في الفهرس:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**ما يفحصه Juntei:** يقارن سبعة ملفات ثنائية مجمّعة لإمكانية الوصول في System32 باستخدام OriginalFilename في PE والبيانات الوصفية للشهادة/الكتالوج، بعد اشتراط توفر بيانات وصفية صالحة للاستخدام من كتالوج Microsoft لملف ثنائي مرجعي مجمّع.

**الأدلة المفحوصة:**

- `Windows\System32\sethc.exe` و`osk.exe` و`narrator.exe` و`magnify.exe` و`utilman.exe` و`atbroker.exe` و`DisplaySwitch.exe`.
- البيانات الوصفية المرجعية المجمّعة لـ `Windows\System32\winlogon.exe` أو `cmd.exe` أو `notepad.exe`، والكتالوجات المجمّعة المنطبقة.

**كيفية التحقيق:** قد توحي البيانات الوصفية ببديل أُعيدت تسميته، لكن Juntei لا يتحقق من صحة التوقيعات أو الثقة في الشهادات. وتُنتج مجموعة أدلة ناقصة تفتقر إلى الكتالوجات اللازمة تغطية مقارنة غير مكتملة، بدلًا من معاملة كل ملف يبدو غير موقَّع على أنه باب خلفي.

### ميزات إمكانية الوصول {#assistive_technology}

**معرّف الفحص:** `assistive_technology` · **ATT&CK في الفهرس:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**ما يفحصه Juntei:** يقرأ أوامر `StartExe` مع `StartParams` المسجلة لإمكانية الوصول في عرضي SOFTWARE الأصلي و32 بت، وإدخالات `Configuration` المفعّلة لإمكانية الوصول من خلايا المستخدم وخلية DEFAULT. وتُتخطى العناصر النائبة الرقمية/غير النصية في StartExe.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\<feature>`: `StartExe` و`StartParams`؛ وكذلك عرض `Wow6432Node` على مستوى الجهاز.
- `HKU\<user>\Software\Microsoft\Windows NT\CurrentVersion\Accessibility\Configuration` في NTUSER.DAT والإعداد المكافئ في خلية DEFAULT.

**كيفية التحقيق:** برامج إمكانية الوصول مشروعة. ميّز بين اسم الميزة المفعّلة وتسجيل ملفها التنفيذي الفعلي، ثم افحص الهدف وأدلة التنفيذ عند تسجيل الدخول/على سطح المكتب. وهذا منفصل عن استبدال الملف التنفيذي لإمكانية الوصول نفسه.

**مزيد من التفاصيل:** [مرجع التقنية (hexacorn.com)](https://www.hexacorn.com/blog/2016/07/22/beyond-good-ol-run-key-part-42/).

### أمر إشعار مهمة BITS {#bits_notify_command}

**معرّف الفحص:** `bits_notify_command` · **ATT&CK في الفهرس:** [T1197](https://attack.mitre.org/techniques/T1197/)

**ما يفحصه Juntei:** يسترد سجلات مهام BITS ذات السلاسل المعدودة (counted-string) من `qmgr.db` أو `qmgr0.dat` أو `qmgr1.dat` الملتقطة، محتفظًا ببرامج الإشعار غير الفارغة ومعاملاتها ومعرّفات المهام ومعرّفات SID للمالكين.

**الأدلة المفحوصة:**

- `ProgramData\Microsoft\Network\Downloader\qmgr.db` و`qmgr0.dat` و`qmgr1.dat`.

**كيفية التحقيق:** هذه سجلات مستخلصة (carved): لا يجري التحقق من حالة المهمة المخصصة/الحالية، وقد تظهر مهام قديمة/محذوفة. ووقت الكتابة في MFT لقاعدة البيانات دليل ضعيف على مستوى الحاوية، وليس وقت تنفيذ الإشعار. وتميّز معرّفات SID للمالكين بين SYSTEM والخدمة المحلية وخدمة الشبكة والمستخدمين.

### AutoRun في معالج الأوامر {#cmd_autorun}

**معرّف الفحص:** `cmd_autorun` · **ATT&CK في الفهرس:** [T1546](https://attack.mitre.org/techniques/T1546/)

**ما يفحصه Juntei:** يقرأ أوامر `AutoRun` في Command Processor للجهاز/المستخدم، وهو إعداد قد يؤثر في بدء تشغيل موجه الأوامر.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |
| `HKU\<user>\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |

**كيفية التحقيق:** قد تفسر أدوات التطوير وتخصيص الواجهة وجود قيمة ما. يتجاوز `cmd.exe /d` إعداد AutoRun، لذا لا يثبت التكوين وحده أنه عمل كلما ظهر cmd.exe. افحص محتوى الأمر ووسيطات العملية المشغِّلة.

**مزيد من التفاصيل:** [وثائق Microsoft](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd).

### الاستمرارية عبر ErrorHandler.cmd {#error_handler_cmd}

**معرّف الفحص:** `error_handler_cmd` · **ATT&CK في الفهرس:** [T1546](https://attack.mitre.org/techniques/T1546/)

**ما يفحصه Juntei:** يُبلغ عن وجود البرنامج النصي المجمّع `Windows\System32\ErrorHandler.cmd` بوصفه مرشحًا للاستمرارية عبر معالج الأخطاء.

**الأدلة المفحوصة:**

- `Windows\System32\ErrorHandler.cmd`.

**كيفية التحقيق:** لا يثبت الوجود وحده أن مسار الاسترداد/الأخطاء المعني في Windows قد استدعاه. اقرأ البرنامج النصي، وافحص مصدره، واربطه بأدلة الإعداد/الأخطاء/العمليات.

### تدفقات Power Automate Desktop {#power_automate}

**معرّف الفحص:** `power_automate` · **ATT&CK في الفهرس:** [T1546](https://attack.mitre.org/techniques/T1546/)

**ما يفحصه Juntei:** يعثر على مجلدات التدفقات/البرامج النصية الملتقطة ضمن مخزني `flows` و`scripts` في Power Automate Desktop لكل مستخدم، ويُبلغ عن كل تدفق مكتشف مرة واحدة.

**الأدلة المفحوصة:**

- `Users\<user>\AppData\Local\Microsoft\Power Automate Desktop\flows\<flow>\...` و`scripts\<flow>\...`.

**كيفية التحقيق:** يحدد هذا الأتمتة المخزنة، لا كل مشغّل مجدول أو تدفق نُفِّذ. افحص محتويات التدفق وتكوين التشغيل الخارجي قبل اعتباره استمرارية تلقائية أو ضارة.

### ملف تعريف PowerShell {#powershell_profiles}

**معرّف الفحص:** `powershell_profiles` · **ATT&CK في الفهرس:** [T1546.013](https://attack.mitre.org/techniques/T1546/013/)

**ما يفحصه Juntei:** يعثر على ملفات من نمط profile.ps1 في مجلدات التثبيت/ملفات التعريف المعروفة لـ Windows PowerShell وPowerShell، بما في ذلك مجلد Documents للمستخدم، ومجلدات OneDrive Documents المكتشفة، وعمليات إعادة توجيه Documents المجمّعة.

**الأدلة المفحوصة:**

- الملفات المنتهية بـ `profile.ps1` مباشرةً في `Windows\System32\WindowsPowerShell\v1.0` و`Windows\SysWOW64\WindowsPowerShell\v1.0`.
- `Documents\WindowsPowerShell` و`Documents\PowerShell` لكل مستخدم، بما في ذلك عمليات إعادة توجيه Documents الملتقطة ومجلدات OneDrive Documents المكتشفة.
- الملفات المنتهية بـ `profile.ps1` مباشرةً في مجلدات الإصدارات ضمن `Program Files\PowerShell` أو `Program Files (x86)\PowerShell`.

**كيفية التحقيق:** قد تحتوي ملفات التعريف على تخصيصات غير ضارة. اقرأ البرنامج النصي وتتبّع الحمولات التي يستدعيها. ويتخطاها تشغيل PowerShell مع `-NoProfile`؛ ولا يثبت وجود الملف وحده تنفيذ ملف التعريف أو وجود محتوى ضار.

### اختطاف شاشة التوقف {#screensaver}

**معرّف الفحص:** `screensaver` · **ATT&CK في الفهرس:** [T1546.002](https://attack.mitre.org/techniques/T1546/002/)

**ما يفحصه Juntei:** يقرأ القيمة `SCRNSAVE.EXE` في Desktop لكل مستخدم، محددًا صورة شاشة التوقف المكوَّنة.

**الأدلة المفحوصة:**

- `HKU\<user>\Control Panel\Desktop\SCRNSAVE.EXE` في NTUSER.DAT.

**كيفية التحقيق:** قارن مع شاشات التوقف المعتمدة وافحص الهدف. تحدد إعدادات سطح المكتب الأخرى وحالة الجلسة ومهلة الانتظار ما إذا كان يمكن تشغيلها؛ ولا يثبت المسار المحفوظ أن شاشة التوقف قد بدأت.

### أمر وحدة التحكم في القياس عن بُعد (TelemetryController) {#telemetry_controller}

**معرّف الفحص:** `telemetry_controller` · **ATT&CK في الفهرس:** [T1546](https://attack.mitre.org/techniques/T1546/)

**ما يفحصه Juntei:** يقرأ `Binary` و`Command` و`MaintenanceCommand` من TelemetryController وتسجيلاته الفرعية المباشرة.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController` والمفاتيح الفرعية المباشرة: `Binary` و`Command` و`MaintenanceCommand`.

**كيفية التحقيق:** قارن الملف التنفيذي المختار ووسيطاته بإصدار Windows نفسه وبرامج الصيانة. فأمر سجل Windows لا يبيّن أن مشغّل الصيانة قد انطلق.

### ملفات التعريف المخصصة في Windows Terminal {#windows_terminal}

**معرّف الفحص:** `windows_terminal` · **ATT&CK في الفهرس:** [T1546](https://attack.mitre.org/techniques/T1546/)

**ما يفحصه Juntei:** يحلل إعدادات JSON/JSONC لـ Windows Terminal بإصداراته المحزّمة والتجريبية (preview) وغير المحزّمة، ويُبلغ عن إعدادات `commandline` غير الفارغة في ملفات التعريف/الإعدادات الافتراضية.

**الأدلة المفحوصة:**

- `Users\<user>\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json` والمكافئ في WindowsTerminalPreview.
- `Users\<user>\AppData\Local\Microsoft\Windows Terminal\settings.json`: مصفوفات ملفات التعريف القديمة، وحقول commandline في `profiles.list` و`profiles.defaults`.

**كيفية التحقيق:** تظهر هنا الواجهات العادية بطبيعة الحال. حدّد ملف التعريف المختار/الافتراضي وافحص الأوامر أو وسيطات التشغيل غير المعتادة. ولا يؤدي مجرد حفظ ملف تعريف إلى تشغيله.

### اشتراك أحداث WMI {#wmi_event_subscription}

**معرّف الفحص:** `wmi_event_subscription` · **ATT&CK في الفهرس:** [T1546.003](https://attack.mitre.org/techniques/T1546/003/)

**ما يفحصه Juntei:** يستخلص مثيلات مستهلكي الأحداث المعروفة ومراجع الربط بين المرشحات والمستهلكين من مستودع WMI الملتقط OBJECTS.DATA. وتشمل الفئات المعروفة مستهلكي سطر الأوامر، والبرامج النصية النشطة، وملفات السجل، وسجل الأحداث، وSMTP.

**الأدلة المفحوصة:**

- `Windows\System32\wbem\Repository\OBJECTS.DATA` و`Repository\FS\OBJECTS.DATA`.

**كيفية التحقيق:** قد تكون الإصابة الخام في المستودع قديمة أو غير مربوطة أو تاريخية؛ ويحافظ الإخراج على هذا التحفظ. ولا يعيد الاستخلاص بناء مخطط اشتراك نشط كامل ولا يثبت أن المشغّل قد انطلق. ووقت كتابة ملف المستودع ليس وقت إنشاء الاشتراك/تنفيذه.

## المستكشف وCOM ومعالجات الملفات أو عناوين URL { #explorer-com-and-file-or-url-handlers }

يتيح Component Object Model (COM) للتطبيقات تفعيل مكونات برمجية مسجلة. وتستخدم واجهة Windows معالجات مسجلة لفتح الملفات وعناوين URL والمجلدات والكائنات الخاصة. وتحدد معرّفات COM (CLSIDs) المكونات؛ بينما تمنح ProgIDs بعض المكونات أسماء مقروءة. تتبّع كل تسجيل وصولًا إلى الأمر أو ملف DLL الذي يختاره.

### اختطاف App Paths {#app_paths}

**معرّف الفحص:** `app_paths` · **ATT&CK في الفهرس:** [T1546](https://attack.mitre.org/techniques/T1546/)

**ما يفحصه Juntei:** يقرأ القيم الافتراضية ضمن تسجيلات `App Paths` على مستوى الجهاز، والتي يمكنها إعادة توجيه البحث عن التطبيقات في عمليات التشغيل عبر الواجهة.

**الأدلة المفحوصة:**

- القيمة الافتراضية لـ `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\<application>`.

**كيفية التحقيق:** تسجيل App Paths شائع للتطبيقات المثبتة، وليس إعادة كتابة عامة لكل عملية تشغيل. قارن الاسم المسجل والصورة بالبرامج المتوقعة، ثم افحص أدلة الواجهة/العمليات ذات الصلة.

### اختطاف كائن COM {#com_treat_as}

**معرّف الفحص:** `com_treat_as` · **ATT&CK في الفهرس:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**ما يفحصه Juntei:** يعثر على عمليات إعادة التوجيه `TreatAs` في COM وتسجيلات InprocServer32/LocalServer32 الخاصة بكل مستخدم التي تحجب معرّف CLSID على مستوى الجهاز، عبر مخازن الفئات المدعومة في العرض الأصلي وعرض 32 بت. وتُحَل أهداف TreatAs باستخدام فئات المستخدم نفسه وفئات الجهاز.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Classes\CLSID\<CLSID>` وعرض `Classes\Wow6432Node` على مستوى الجهاز.
- `Software\Classes\CLSID` لكل مستخدم في NTUSER.DAT و`CLSID` في UsrClass.dat، بما في ذلك عروض `Wow6432Node` المنفَّذة لهما: القيم الافتراضية لـ `TreatAs` و`InprocServer32` و`LocalServer32`.

**كيفية التحقيق:** قد لا تتطلب التجاوزات الخاصة بكل مستخدم صلاحيات المسؤول، لكن البرامج المشروعة قد تسجلها أيضًا. تتبّع الفئة المختارة وملف DLL/الملف التنفيذي، وقيّم البنية المعمارية وسياق التفعيل، وعزّز حدوث تفعيل COM الفعلي بشكل مستقل.

### اختطاف معالج Disk Cleanup {#disk_cleanup_handler}

**معرّف الفحص:** `disk_cleanup_handler` · **ATT&CK في الفهرس:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**ما يفحصه Juntei:** يقرأ معرّفات CLSID لمعالجات VolumeCaches في Disk Cleanup، ويتتبع تسجيلات InprocServer32 القابلة للحل في COM على مستوى الجهاز وصولًا إلى مكتباتها.

**الأدلة المفحوصة:**

- معرّف CLSID الافتراضي في `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\<handler>` و`Classes\CLSID\<CLSID>\InprocServer32`.

**كيفية التحقيق:** يجب أن يستدعي مسار عمل التنظيف المعني المعالجَ كي يعمل. ولا يُخرج هذا الفحص تحديدًا المعالجات غير المحلولة، لذا استخدم محلل سجل Windows لفحص تسجيلاتها الخام إذا كان الالتقاط غير مكتمل.

### تجاوز AppKey في المستكشف {#explorer_app_key}

**معرّف الفحص:** `explorer_app_key` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يقرأ قيمتي `ShellExecute` و`Association` ضمن المفاتيح الفرعية المباشرة لـ `AppKey` في المستكشف، والتي تغطي الأوامر/الإجراءات المختارة بمفاتيح التطبيقات.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\AppKey\<key>`: `ShellExecute` و`Association`.

**كيفية التحقيق:** يمكن لأدوات لوحة المفاتيح وأدوات الشركات المصنّعة (OEM) تعريف هذه القيم. وقد يتطلب اسم الارتباط محورًا آخر في سجل Windows بدلًا من أن يكون مسار ملف. تحقق من إجراء المستخدم الذي من شأنه تشغيله.

### كائنات مساعدة المتصفح (Browser Helper Objects) {#explorer_bho}

**معرّف الفحص:** `explorer_bho` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يقرأ تسجيلات CLSID لكائنات Browser Helper Object ويحل مكتبات InprocServer32 على مستوى الجهاز عند توفرها؛ وتظل معرّفات الفئات غير المحلولة ظاهرة.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects\<CLSID>` والقيمة الافتراضية لـ `Classes\CLSID\<CLSID>\InprocServer32` على مستوى الجهاز.

**كيفية التحقيق:** يرتبط هذا بسلوك المتصفح/المستكشف المتوافق، ويتعلق خصوصًا بالبيئات القديمة. وقد تظهر امتدادات معتمدة للمتصفح/الأمان. ولا يُعد BHO المسجل دليلًا على أن متصفحًا حاليًا قد حمّله.

### اختطاف CLSID لسطح المكتب في المستكشف {#explorer_clsid_hijack}

**معرّف الفحص:** `explorer_clsid_hijack` · **ATT&CK في الفهرس:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**ما يفحصه Juntei:** يقرأ أوامر الفتح المكوَّنة في الواجهة لمعرّفات CLSID الخاصة الثلاثة المدرجة في مساحة أسماء المستكشف، من مخازن فئات الجهاز/المستخدم.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |

**كيفية التحقيق:** قارن معالجات الأوامر لـ Desktop وMy Computer وMy Documents بخط أساس مطابق. يفحص هذا معرّفات CLSID ومسارات مختارة، لا كل اختطاف ممكن لمساحة أسماء المستكشف.

### معالجات قائمة السياق في المستكشف {#explorer_context_menu}

**معرّف الفحص:** `explorer_context_menu` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يعدّد معالجات قائمة السياق لـ Directory وDirectory Background وFolder وDrive وAllFilesystemObjects و`*`، متتبعًا معرّفات الفئات وصولًا إلى تسجيلات DLL المجمّعة حيثما أمكن.

**الأدلة المفحوصة:**

- مخزن فئات الجهاز `HKLM\SOFTWARE\Classes` ومخزن فئات المستخدم UsrClass.dat: القيمة الافتراضية لـ `<subject>\shellex\ContextMenuHandlers\<handler>` وتسجيلات مكتبات COM المقابلة.

**كيفية التحقيق:** امتدادات النقر بزر الماوس الأيمن من أدوات الأرشفة ومنتجات الأمان وعملاء التخزين السحابي شائعة. افحص المعالج الفعلي والتفاعل المشغِّل. وتظل معرّفات GUID غير المحلولة خيوطًا مفيدة، لا دليلًا على أن ملف DLL لم يكن موجودًا.

### اختطاف اقترانات الملفات {#file_association_hijack}

**معرّف الفحص:** `file_association_hijack` · **ATT&CK في الفهرس:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**ما يفحصه Juntei:** يفحص تسجيلات `shell\open\command` للامتدادات المباشرة ولمعرّفات ProgID المحلولة لكل من `.txt` و`.pdf` و`.doc` و`.docx` و`.html` و`.htm` و`.js` و`.vbs` و`.hta` و`.exe` و`.bat` و`.cmd` و`.ps1`.

**الأدلة المفحوصة:**

- مخزن فئات الجهاز `HKLM\SOFTWARE\Classes` ومخزن فئات المستخدم UsrClass.dat: القيم الافتراضية لـ `<extension>` المختارة والقيم الافتراضية لـ `<extension-or-ProgID>\shell\open\command`.

**كيفية التحقيق:** قارن مع التطبيق المتوقع وافحص وسيطات الأمر. هذه مجموعة امتدادات محددة، لا جميع أنواع الملفات ولا الحل الفعلي الكامل لـ UserChoice. ولا يثبت الأمر المحفوظ أن مستندًا قد فُتح.

### اختطاف معالجات البروتوكولات {#protocol_handler_hijack}

**معرّف الفحص:** `protocol_handler_hijack` · **ATT&CK في الفهرس:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**ما يفحصه Juntei:** يقرأ أوامر فتح عناوين URL لـ http وhttps وmailto وms-msdt وms-officecmd، إضافةً إلى تسجيلات `URL Protocol` المكتشفة وتجاوزات المستخدم المنطبقة.

**الأدلة المفحوصة:**

- مخزن فئات الجهاز `HKLM\SOFTWARE\Classes` ومخزن فئات المستخدم UsrClass.dat: علامات `URL Protocol` والقيم الافتراضية لـ `<scheme>\shell\open\command`.

**كيفية التحقيق:** تسجّل المتصفحات وبرامج تثبيت التطبيقات المخططات (schemes) بصورة مشروعة. تتبّع الأمر ووسيطاته، وقيّم الغرض المتوقع من المخطط، واربط ذلك بأدلة فتح عناوين URL/العمليات.

### معالج امتداد COM لسلة المحذوفات {#recycle_bin_com_extension}

**معرّف الفحص:** `recycle_bin_com_extension` · **ATT&CK في الفهرس:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**ما يفحصه Juntei:** يقرأ تجاوزات أوامر open/empty/explore لمعرّف CLSID المدرج لسلة المحذوفات (Recycle Bin)، وتسجيلات معالجات قائمة السياق/السحب والإفلات المباشرة.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\empty\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\explore\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\ContextMenuHandlers\<child>` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\DragDropHandlers\<child>` | `(Default)` |

**كيفية التحقيق:** قارن مع امتدادات الواجهة المعتمدة. بعض القيم المُبلَّغ عنها معرّفات COM لا مكتبات محلولة مباشرةً؛ تحقق من الفئة المشار إليها وإجراء المستخدم قبل الجزم بتنفيذ الشيفرة.

### اختطاف معالج بروتوكول البحث {#search_protocol_handler}

**معرّف الفحص:** `search_protocol_handler` · **ATT&CK في الفهرس:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**ما يفحصه Juntei:** يقرأ المعالج الافتراضي `search-ms\shell\open\command` للجهاز/المستخدم.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Classes\search-ms\shell\open\command` | `(Default)` |
| `HKU\<user>\Software\Classes\search-ms\shell\open\command` | `(Default)` |

**كيفية التحقيق:** يمكن لتغيير المعالج أن يغيّر ما يحدث عند فتح عنوان URL من نوع search-ms، لكن وجوده ليس دليلًا على إيصال استغلال. قارن مع الإعدادات الافتراضية لـ Windows، وتحقق من نشاط عناوين URL/العمليات الذي من شأنه استخدامه.

### SharedTaskScheduler {#shared_task_scheduler}

**معرّف الفحص:** `shared_task_scheduler` · **ATT&CK في الفهرس:** [T1546](https://attack.mitre.org/techniques/T1546/)

**ما يفحصه Juntei:** يعدّد قيم `SharedTaskScheduler` غير الفارغة في المستكشف بوصفها مكونات واجهة مسجلة.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SharedTaskScheduler` | جميع القيم غير الفارغة |

**كيفية التحقيق:** قد تشير القيم إلى فئات COM بدلًا من مسار ملف تنفيذي. تتبّع تسجيلات الخوادم الخاصة بها، وتحقق من سلوك نظام التشغيل، ومصدر الامتداد، وأدلة التحميل في الواجهة ذات الصلة.

### ShellExecuteHooks {#shell_execute_hooks}

**معرّف الفحص:** `shell_execute_hooks` · **ATT&CK في الفهرس:** [T1546](https://attack.mitre.org/techniques/T1546/)

**ما يفحصه Juntei:** يعدّد قيم `ShellExecuteHooks` غير الفارغة في المستكشف، والتي تغطي تسجيلات خطافات تنفيذ الواجهة القديمة.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ShellExecuteHooks` | جميع القيم غير الفارغة |

**كيفية التحقيق:** احلل أي CLSID مشار إليه وتحقق مما إذا كان إصدار Windows والنهج يسمحان بالخطاف. قد يكون التسجيل القديم غير مستخدم؛ والقيمة بحد ذاتها ليست تنفيذًا مُلاحَظًا عبر الواجهة.

### الاستمرارية عبر COM في مكتبات قائمة "ابدأ" ومجلدات CLSID {#start_menu_com}

**معرّف الفحص:** `start_menu_com` · **ATT&CK في الفهرس:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**ما يفحصه Juntei:** يعثر على ملفات XML للمكتبات `.library-ms` التي يشير عنوان URL في `simpleLocation` الخاص بها
إلى معرّف CLSID في COM، وعلى المجلدات المسماة `<name>.{CLSID}`، ضمن
`Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu`. ويمكن لتفاعل المستكشف مع هذه
الكائنات أن يفعّل شيفرة COM. ويشمل الفحص المجلدات الملتقطة الفارغة؛ ولا يتبع الارتباطات
الرمزية. ولا يُبلغ هذا الفحص عن ملفات المكتبات العادية خارج قائمة "ابدأ" ولا عن معرّفات GUID الواردة في
تعليقات XML.

**الأدلة المفحوصة:**

- ملفات `.library-ms` وإدخالات المجلدات المسماة بمعرّفات CLSID في قائمة "ابدأ" لكل مستخدم.
- تسجيلات الفئات الملتقطة للمستخدم المطابق وخلية SOFTWARE للجهاز، كلٌّ حسب البنية المعمارية على حدة.
- تكوين CodeBase/AppID/الخدمات المُدار عبر أداة الحل نفسها المستخدمة للمهام المجدولة.

**كيفية التحقيق:** يحدد `Path` و`SourceFile` الأثر الرقمي المشغِّل للتفعيل. ويحتوي `Value`
على أمر الخادم المرشح، أو على CLSID إذا تعذر الحل. ويربط `Description`
المشغّل بمعرّف CLSID ومصدره الملتقط في سجل Windows. ولا يصف `ResolvedPath`/`Sha256` إلا
حمولة محلولة ومجمّعة. ويظل ارتباط AppID المقتصر على اسم الملف التنفيذي دون حل.
وقد يكون المشغّل مشروعًا؛ فوجوده لا يثبت حدوث تسجيل دخول أو تفعيل أو تنفيذ أو هجوم.
افحص عنوان URL للمكتبة، وتسجيل الفئة، والحمولة، وسياق المستخدم، وسجلات التنفيذ المستقلة.

**تحذيرات الجمع:** قد تُسقط مجموعات الأدلة المقتصرة على الملفات المجلدات الفارغة. تُفحص مسارات قائمة "ابدأ"
القياسية للمستخدم؛ ولا يغطي هذا الفحص قوائم "ابدأ" المعاد توجيهها ولا مواقع مساحة أسماء المستكشف
الأخرى. وتُنتج الخلايا المفقودة أو التسجيلات غير المحلولة تغطية جزئية مع الاحتفاظ بالمرشح.
ويمكن لبنية المهمة المعمارية ورفع الصلاحيات والنهج أن تغيّر سلوك البحث الفعلي في COM.

**مزيد من التفاصيل:** [الاقتراح الأصلي ونسبة الفضل إلى الباحث](https://github.com/last-byte/PersistenceSniper/pull/32)،
[طرق العرض المدمجة لـ COM لكل مستخدم/جهاز](https://learn.microsoft.com/en-us/windows/win32/sysinfo/merged-view-of-hkey-classes-root)،
[اختطاف COM في MITRE](https://attack.mitre.org/techniques/T1546/015/).

### اختطاف COM عبر TypeLib {#typelib_hijack}

**معرّف الفحص:** `typelib_hijack` · **ATT&CK في الفهرس:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**ما يفحصه Juntei:** يقرأ تسجيلات TypeLib لكل مستخدم في UsrClass.dat حسب GUID/الإصدار/الإعدادات المحلية لـ win32/win64، محتفظًا بالمسارات وقيم مُعرِّفات البرامج النصية (script monikers). ولا تُزال فهارس الموارد الرقمية اللاحقة إلا لغرض حل الملف الهدف.

**الأدلة المفحوصة:**

- القيم الافتراضية لـ `TypeLib\<GUID>\<version>\<locale>\win32` و`win64` في كل ملف UsrClass.dat ملتقط.

**كيفية التحقيق:** قد تشير مكتبة الأنواع (type library) إلى ملف DLL مشروع، أو ملف بديل، أو مُعرِّف برنامج نصي. يجرد الفحص هذه التسجيلات؛ فقيّم التطبيق المتأثر وعزّز تحميل مكتبة الأنواع بدلًا من افتراض أن كل مسار غير افتراضي هجوم.

## تطبيقات Office والوظائف الإضافية { #office-applications-and-add-ins }

يدعم Office الوظائف الإضافية والقوالب والامتدادات الخاصة بالتطبيقات المشروعة. وقد يعتمد التسجيل المريب على إصدار معين من Office، أو تطبيق، أو إعداد تحميل، أو تفاعل من المستخدم. ولا يثبت التكوين وحده تنفيذ وحدات الماكرو.

### تسجيل وظيفة إضافية في Office {#office_addins}

**معرّف الفحص:** `office_addins` · **ATT&CK في الفهرس:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**ما يفحصه Juntei:** يعدّد الوظائف الإضافية لـ Word وExcel وPowerPoint وOutlook وAccess في جذور Office غير المرتبطة بإصدار وفي جذور 14.0/15.0/16.0، بما في ذلك مخزن Click-to-Run المنفَّذ على مستوى الجهاز وجذور المستخدم. ويحتفظ بقيم Manifest/FileName/Path الصريحة أو يحل الإدخالات المؤهلة المدعومة بـ COM، مع سياق LoadBehavior.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` والمكافئ المنفَّذ في Click-to-Run على مستوى الجهاز.
- `HKU\<user>\Software\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` في NTUSER.DAT؛ وفئات COM المطابقة للجهاز/المستخدم من SOFTWARE وNTUSER.DAT وUsrClass.dat.

**كيفية التحقيق:** الوظائف الإضافية برامج أعمال شائعة. راجع LoadBehavior، والتطبيق/الإصدار، وملفات البيان (manifests)، والحمولات الفعلية، والمورّد المتوقع. ولا يضمن الفحص أن كل تنسيق وظيفة إضافية مثبتة أو كل طريقة عرض للتسجيل قد جُمعت أو حُمِّلت.

### اختطاف وظيفة إضافية للذكاء الاصطناعي في Office {#office_ai_hijack}

**معرّف الفحص:** `office_ai_hijack` · **ATT&CK في الفهرس:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**ما يفحصه Juntei:** يعدّد القيم في موقع تسجيل Common AI الخاص بـ Office 16.0 Click-to-Run.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI`: جميع القيم.

**كيفية التحقيق:** قارن القيم والمسارات غير المعتادة بإصدار Office المثبت. ولا يُظهر وجود قيمة ميزة Office التي استخدمتها، ولا يثبت أنها اختارت شيفرة قابلة للتنفيذ.

### تجاوز DLL في Office {#office_dll_override}

**معرّف الفحص:** `office_dll_override` · **ATT&CK في الفهرس:** [T1137](https://attack.mitre.org/techniques/T1137/)

**ما يفحصه Juntei:** يقرأ `WwlibtDll` و`PPCoreTDLL` ضمن جذور Word/PowerPoint للإعدادات غير المرتبطة بإصدار وإعدادات 14.0/15.0/16.0، بما في ذلك جذور Click-to-Run والمستخدم المنفَّذة.

**الأدلة المفحوصة:**

- إعدادات `Microsoft\Office\[<version>\]Word` و`PowerPoint` للجهاز/المستخدم، إضافةً إلى شجرة Office المنفَّذة في Click-to-Run على مستوى الجهاز: `WwlibtDll` و`PPCoreTDLL`.

**كيفية التحقيق:** حدّد التطبيق/الإصدار الذي يتعرف على التجاوز، وافحص المكتبة ومصدرها، واربط ذلك بتشغيل Office أو تحميل صورة. ولا يُعد التجاوز المكوَّن وحده دليلًا على التنفيذ.

### القوالب الافتراضية في Office {#office_templates}

**معرّف الفحص:** `office_templates` · **ATT&CK في الفهرس:** [T1137.001](https://attack.mitre.org/techniques/T1137/001/)

**ما يفحصه Juntei:** يجرد ملفات Normal.dotm/NormalEmail.dotm المجمّعة والملفات الموجودة مباشرةً في مجلدي Word Startup وExcel XLSTART المعروفين، بما في ذلك مجلدات بدء التشغيل في تثبيت Office.

**الأدلة المفحوصة:**

- `Users\<user>\AppData\Roaming\Microsoft\Templates\Normal.dotm` و`NormalEmail.dotm`.
- الملفات المباشرة في `Microsoft\Word\STARTUP` و`Microsoft\Excel\XLSTART` للمستخدم ضمن Roaming AppData.
- الملفات المباشرة في مجلدات Office المعروفة `<version>\STARTUP` / `XLSTART` و`root\<version>\STARTUP` / `XLSTART` ضمن Program Files وProgram Files (x86).

**كيفية التحقيق:** قد يكون القالب مشروعًا أو لا يحتوي على أي محتوى قابل للتنفيذ. ولا يستخرج Juntei وحدات الماكرو ولا ينفذها في هذا الفحص. افحص البنية الداخلية للمستند، ونهج وحدات الماكرو، ونشاط التطبيق؛ ولا يعني مسار التثبيت على مستوى الجهاز وجود رمز وصول SYSTEM للعملية.

### ملف DLL لـ Office Test {#office_test_dll}

**معرّف الفحص:** `office_test_dll` · **ATT&CK في الفهرس:** [T1137.002](https://attack.mitre.org/techniques/T1137/002/)

**ما يفحصه Juntei:** يعدّد القيم في المفتاح Office Test Special Perf من طرق العرض المدرجة للجهاز/المستخدم وعرض 32 بت للجهاز.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Office Test\Special\Perf` | جميع القيم غير الفارغة |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Office Test\Special\Perf` | جميع القيم غير الفارغة |
| `HKU\<user>\SOFTWARE\Microsoft\Office Test\Special\Perf` | جميع القيم غير الفارغة |

**كيفية التحقيق:** تعامل مع التجاوزات غير المتوقعة بوصفها خيطًا يتطلب التحقق من إصدار Office والحمولة. يُبلغ الفحص عن التكوين بدلًا من اختبار ما إذا كان تثبيت Office بعينه يحمّل القيمة.

### هجوم الصفحة الرئيسية في Outlook {#outlook_home_page}

**معرّف الفحص:** `outlook_home_page` · **ATT&CK في الفهرس:** [T1137.004](https://attack.mitre.org/techniques/T1137/004/)

**ما يفحصه Juntei:** يقرأ `URL` ضمن المفاتيح الفرعية لـ WebView في Outlook للمستخدم لإصدارات Office 14.0 و15.0 و16.0.

**الأدلة المفحوصة:**

- `HKU\<user>\Software\Microsoft\Office\<14.0|15.0|16.0>\Outlook\WebView\<folder>\URL` في NTUSER.DAT.

**كيفية التحقيق:** قد تكون الصفحة الرئيسية للمجلد مشروعة، ويختلف السلوك باختلاف إصدار Outlook وتحديثات الأمان. افحص عنوان URL/المحتوى والنهج الفعّال، ثم اربط ذلك بأدلة Outlook/الويب/العمليات.

### اختطاف ملف DLL لمراقب VBA {#vba_monitors}

**معرّف الفحص:** `vba_monitors` · **ATT&CK في الفهرس:** [T1137](https://attack.mitre.org/techniques/T1137/)

**ما يفحصه Juntei:** يقرأ قيم InprocServer32 لمعرّفي CLSID المنفَّذين المتعلقين بمراقب VBA في مخازن فئات الجهاز وكل مستخدم.

**الأدلة المفحوصة:**

- `Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32` و`{0002E157-0000-0000-C000-000000000046}` في SOFTWARE للجهاز، إضافةً إلى التسجيلات المكافئة في UsrClass.dat للمستخدم.

**كيفية التحقيق:** قارن المكتبات المختارة ببيئة Office/VBA المثبتة. هذا فحص موجّه لمعرّفات CLSID محددة، وليس جردًا لكل امتداد VBA ولا دليلًا على تشغيل ماكرو.

## مصححات الأخطاء ومعالجات الأعطال ورقع التوافق { #debuggers-crash-handlers-and-compatibility-shims }

يمكن لمصحح أخطاء أو معالج أخطاء أن يعمل عندما يبدأ تطبيق ما، أو يتعطل، أو يتوقف عن الاستجابة، أو ينتهي. ويمكن لقواعد بيانات توافق التطبيقات أيضًا أن تؤثر في بدء التشغيل. تحقق من الإعدادات المسبقة اللازمة والبرنامج المشغِّل إلى جانب الأمر المُبلَّغ عنه.

### اختطاف مصحح أخطاء AeDebug {#ae_debug}

**معرّف الفحص:** `ae_debug` · **ATT&CK في الفهرس:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**ما يفحصه Juntei:** يقرأ أوامر `Debugger` في AeDebug بالعرض الأصلي وعرض 32 بت، والمستخدمة في تكوين تصحيح الأعطال الفوري (just-in-time).

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |

**كيفية التحقيق:** قد تضبط برامج التطوير/التصحيح هذا الإعداد بصورة مشروعة. افحص أمر مصحح الأخطاء الكامل والشروط المسبقة لتشغيله عند العطل، ثم اربط ذلك بأحداث أخطاء التطبيقات والعمليات.

### مصحح أخطاء العمليات المحمية في AeDebug {#ae_debug_protected}

**معرّف الفحص:** `ae_debug_protected` · **ATT&CK في الفهرس:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**ما يفحصه Juntei:** يقرأ أوامر `Debugger` في AeDebugProtected بالعرض الأصلي وعرض 32 بت، الخاصة بسطح تصحيح أخطاء العمليات المحمية.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |

**كيفية التحقيق:** حدّد ما إذا كان إصدار نظام التشغيل ونهج الحماية يسمحان باستخدام الأمر المكوَّن. ولا يُعد مصحح الأخطاء المحفوظ دليلًا على تعطل عملية محمية أو على تشغيل الأمر.

### مصحح أخطاء Image File Execution Options {#ifeo_debugger}

**معرّف الفحص:** `ifeo_debugger` · **ATT&CK في الفهرس:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**ما يفحصه Juntei:** يبحث في تسجيلات Image File Execution Options بالعرض الأصلي وعرض 32 بت، بما فيها مفاتيح المرشحات المتداخلة، عن قيم `Debugger`.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\Debugger`، بما في ذلك مفاتيح المرشحات المتداخلة وعرض `Wow6432Node` على مستوى الجهاز.

**كيفية التحقيق:** يمكن اختيار مصحح أخطاء عند تشغيل الصورة المسماة. افحص نطاق الصورة/المرشح ووسيطات مصحح الأخطاء؛ إذ يمكن لأدوات المطورين أيضًا استخدام IFEO. تتبّع تشغيل الهدف لإثبات التنفيذ.

### NTDLL المفوَّض عبر IFEO {#ifeo_delegated_ntdll}

**معرّف الفحص:** `ifeo_delegated_ntdll` · **ATT&CK في الفهرس:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**ما يفحصه Juntei:** يبحث في تسجيلات Image File Execution Options بالعرض الأصلي وعرض 32 بت، بما فيها المفاتيح المتداخلة، عن تجاوزات `VerifierDlls`.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\VerifierDlls`، بما في ذلك المفاتيح المتداخلة وعرض `Wow6432Node` على مستوى الجهاز.

**كيفية التحقيق:** شرط الفحص هو قيمة VerifierDlls المكوَّنة؛ ولا يثبت أن جميع الشروط المسبقة للتحقق/التفويض مفعّلة. راجع العلامات المنطبقة وإصدار Windows والحمولة قبل استنتاج وجود مسار حقن فعّال.

### مراقب الخروج الصامت للعمليات {#ifeo_silent_process_exit}

**معرّف الفحص:** `ifeo_silent_process_exit` · **ATT&CK في الفهرس:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**ما يفحصه Juntei:** يبحث في تسجيلات SilentProcessExit بالعرض الأصلي وعرض 32 بت عن `MonitorProcess`، بما في ذلك المفاتيح المتداخلة.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\<image>\...\MonitorProcess`، بما في ذلك المفاتيح المتداخلة وعرض `Wow6432Node` على مستوى الجهاز.

**كيفية التحقيق:** تتطلب مراقبة الخروج الصامت أيضًا تكوين الإبلاغ/IFEO المعني. افحص تلك الإعدادات وأحداث خروج الصورة المراقَبة؛ وقد تفسر أدوات التشخيص المشروعة وجود المراقب.

### رقع توافق التطبيقات (Application Shimming) {#installed_sdb}

**معرّف الفحص:** `installed_sdb` · **ATT&CK في الفهرس:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**ما يفحصه Juntei:** يقرأ `DatabasePath` من تسجيلات InstalledSDB المباشرة لقواعد بيانات توافق التطبيقات المثبتة.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\<child>` | `DatabasePath` |

**كيفية التحقيق:** قد تكون إصلاحات التوافق مشروعة. افحص محتويات ملف SDB والملفات التنفيذية المستهدفة وسجل التثبيت. ولا يحدد تسجيل قاعدة البيانات رقعها (shims) ولا يثبت أن تطبيقًا مطابقًا قد عمل.

### اختطاف مصحح أخطاء LSM {#lsm_debugger}

**معرّف الفحص:** `lsm_debugger` · **ATT&CK في الفهرس:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**ما يفحصه Juntei:** يقرأ `MonitorProcess` تحديدًا ضمن SilentProcessExit الخاص بـ `lsm.exe` في عرضي الجهاز الأصلي و32 بت.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |

**كيفية التحقيق:** يتداخل هذا مع الجرد الأشمل لـ SilentProcessExit. تأكد من العلامات المسبقة اللازمة ومن حدوث خروج ذي صلة لـ LSM قبل معاملة التكوين على أنه استمرارية نُفِّذت؛ واحسب الإدخال نفسه مرة واحدة عند تقييم النتائج المتمايزة.

### ملفات DLL المساعدة لـ MiniDump {#minidump_auxiliary_dlls}

**معرّف الفحص:** `minidump_auxiliary_dlls` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يعدّد القيم المسماة غير الفارغة في MiniDumpAuxiliaryDlls، محتفظًا باسم القيمة بوصفه سياق الوحدة المشغِّلة وببيانات القيمة بوصفها الهدف المختار.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls`: أسماء قيم الوحدات المشغِّلة وبيانات الأهداف.

**كيفية التحقيق:** افحص كلًّا من الوحدة المشغِّلة وملف DLL المساعد. قد تفسر أدوات الأعطال/التفريغ المصغّر (minidump) وجود إدخال ما، ولا يُظهر التسجيل أن عملية تفريغ قد حمّلت المكتبة.

### رقع توافق التطبيقات (Application Shimming) {#shim_custom}

**معرّف الفحص:** `shim_custom` · **ATT&CK في الفهرس:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**ما يفحصه Juntei:** يقرأ القيم المسماة بلاحقة `.sdb` ضمن مفاتيح التطبيقات المباشرة في AppCompatFlags Custom، محتفظًا بالارتباط بين التطبيق وقاعدة البيانات.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<application>`: أسماء القيم المنتهية بـ `.sdb`.

**كيفية التحقيق:** اقرن هذه الارتباطات بـ InstalledSDB وبقاعدة بيانات التوافق الفعلية. ولا يكشف ارتباط اسم الملف وحده سلوك قاعدة البيانات ولا يثبت تشغيل تطبيق مطابق.

### اختطاف مصحح أخطاء WER {#wer_debugger}

**معرّف الفحص:** `wer_debugger` · **ATT&CK في الفهرس:** [T1546](https://attack.mitre.org/techniques/T1546/)

**ما يفحصه Juntei:** يقرأ قيم `Debugger` في Windows Error Reporting بالعرض الأصلي وعرض 32 بت.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `Debugger` |

**كيفية التحقيق:** قد تكوّن برامج التشخيص المشروعة مصحح أخطاء. تحقق من الأمر الكامل وأحداث الأعطال ذات الصلة؛ فالتسجيل ليس دليلًا على أن Windows Error Reporting قد شغّله.

### اختطاف مصحح أخطاء Hangs في WER {#wer_hangs}

**معرّف الفحص:** `wer_hangs` · **ATT&CK في الفهرس:** [T1546](https://attack.mitre.org/techniques/T1546/)

**ما يفحصه Juntei:** يقرأ قيم `Debugger` في Windows Error Reporting Hangs بالعرض الأصلي وعرض 32 بت.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |

**كيفية التحقيق:** تحقق مما إذا كان مسار عمل معالجة حالات التوقف عن الاستجابة سيستدعي هذا الإعداد، وافحص البرنامج الهدف. اربط ذلك بسجلات توقف التطبيقات عن الاستجابة والعمليات بدلًا من معاملة الكتابة في سجل Windows على أنها تنفيذ لمصحح أخطاء.

### اختطاف ReflectDebugger في WER {#wer_reflect_debugger}

**معرّف الفحص:** `wer_reflect_debugger` · **ATT&CK في الفهرس:** [T1546](https://attack.mitre.org/techniques/T1546/)

**ما يفحصه Juntei:** يقرأ قيم `ReflectDebugger` في Windows Error Reporting بالعرض الأصلي وعرض 32 بت.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |

**كيفية التحقيق:** قارن مع تكوين التشخيص المعتمد وتأكد من انطباقه على إصدار Windows/المشغّل. افحص وسيطات الأمر وابحث عن تعزيز من أدلة الأعطال/العمليات قبل نسب التنفيذ.

### وحدات استثناءات وقت التشغيل في WER {#wer_runtime_exception}

**معرّف الفحص:** `wer_runtime_exception` · **ATT&CK في الفهرس:** [T1546](https://attack.mitre.org/techniques/T1546/)

**ما يفحصه Juntei:** يعدّد أسماء القيم ضمن RuntimeExceptionHelperModules بالعرض الأصلي وعرض 32 بت؛ وهنا يُحمل مسار المكتبة في اسم القيمة، لا بالضرورة في بياناتها.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules` وعرض `Wow6432Node` على مستوى الجهاز: أسماء القيم.

**كيفية التحقيق:** افحص الوحدة المسماة وسياق التطبيق/وقت التشغيل. فالقيمة الرقمية الصغيرة في البيانات ليست مسار الحمولة، ولا يثبت التسجيل وحده أن الوحدة المساعدة قد حُمِّلت.

## إعدادات وقت تشغيل .NET وأداة التوصيف { #net-runtime-and-profiler-settings }

يمكن لأوقات التشغيل المُدارة أن تحمّل أدوات التوصيف (profilers) أو تجميعات بدء التشغيل أو مكتبات التصحيح قبل شيفرة التطبيق. ويستخدم المطورون ومنتجات المراقبة هذه الإعدادات أيضًا. تحقق من وقت التشغيل، وبنية العملية المعمارية، وعلامات التفعيل، وما إذا كان يمكن لعملية بدأت حديثًا أن ترث الإعداد.

### حقن AppDomainManager {#appdomain_manager}

**معرّف الفحص:** `appdomain_manager` · **ATT&CK في الفهرس:** [T1574.014](https://attack.mitre.org/techniques/T1574/014/)

**ما يفحصه Juntei:** يقرأ متغيري البيئة `APPDOMAIN_MANAGER_ASM` و`APPDOMAIN_MANAGER_TYPE` للجهاز/المستخدم، اللذين يغطيان اختيار التجميع (assembly) والنوع لمدير AppDomain مخصص في .NET.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_TYPE` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_TYPE` |

**كيفية التحقيق:** قد تستخدم التطبيقات مديرين مخصصين عمدًا. احلل التجميع والنوع معًا وتحقق من توافق وقت التشغيل ووراثة العمليات؛ فلا يثبت متغير منفرد ولا الطابع الزمني لمفتاحه أن تطبيقًا مُدارًا قد حمّله.

### اختطاف أداة توصيف .NET CLR {#cor_profiler}

**معرّف الفحص:** `cor_profiler` · **ATT&CK في الفهرس:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**ما يفحصه Juntei:** يقرأ إعدادات بيئة التوصيف COR للجهاز/المستخدم، والإدخالات المسماة المطابقة في السلاسل المتعددة Environment الخاصة بكل خدمة. ويحتفظ بعلامات التفعيل، ومعرّفات أداة التوصيف، والمسارات الخاصة بكل بنية معمارية.

**الأدلة المفحوصة:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` و`Environment` في كل ملف NTUSER.DAT ملتقط للمستخدمين.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: إعدادات COR المسماة في السلسلة المتعددة.

**كيفية التحقيق:** تدعم أدوات التوصيف المراقبة المشروعة. علامة التفعيل وCLSID ومسار DLL أجزاء مختلفة من التكوين؛ تأكد من التفعيل الفعلي ومن عملية .NET المعنية بدلًا من معاملة كل صف على أنه أداة توصيف نُفِّذت.

### اختطاف أداة توصيف .NET Core CLR {#coreclr_profiler}

**معرّف الفحص:** `coreclr_profiler` · **ATT&CK في الفهرس:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**ما يفحصه Juntei:** يقرأ إعدادات بيئة التوصيف CORECLR المقابلة للجهاز/المستخدم، وإدخالات Environment المطابقة للخدمات، بما في ذلك المسارات الخاصة بكل بنية معمارية.

**الأدلة المفحوصة:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` و`Environment` في كل ملف NTUSER.DAT ملتقط للمستخدمين.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: إعدادات CORECLR المسماة في السلسلة المتعددة.

**كيفية التحقيق:** تحقق من وقت تشغيل .NET، والبنية المعمارية، وعلامة التفعيل، والمكتبة. قد تكوّن منتجات قابلية المراقبة (observability) المعتمدة هذه الإعدادات، وقد لا ترث عملية قائمة بيئةً جرى تغييرها حديثًا.

### اختطاف DbgManagedDebugger في .NET {#dotnet_dbg_managed_debugger}

**معرّف الفحص:** `dotnet_dbg_managed_debugger` · **ATT&CK في الفهرس:** [T1546](https://attack.mitre.org/techniques/T1546/)

**ما يفحصه Juntei:** يقرأ إعدادات `DbgManagedDebugger` المدرجة في `.NETFramework` للجهاز/المستخدم، بما في ذلك مسار 32 بت الصريح للجهاز.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `DbgManagedDebugger` |

**كيفية التحقيق:** حدّد ما إذا كان نهج وقت تشغيل .NET/التصحيح المنطبق يستخدم هذا الأمر. قد تفسره أدوات المطورين؛ اربطه بأعطال التطبيقات المُدارة وإنشاء العمليات لإثبات التنفيذ.

### مفتاح سجل Windows لـ COR_PROFILER في .NET Framework {#dotnet_framework_profiler}

**معرّف الفحص:** `dotnet_framework_profiler` · **ATT&CK في الفهرس:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**ما يفحصه Juntei:** يقرأ `COR_PROFILER` و`COR_PROFILER_PATH` و`COR_ENABLE_PROFILING` من طرق عرض سجل Windows المدرجة لـ `.NETFramework`. ويكمّل هذا فحوص أدوات التوصيف القائمة على البيئة.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |

**كيفية التحقيق:** قيّم القيم معًا وحدّد ما إذا كان وقت التشغيل يعتدّ بذلك الموقع في سجل Windows. قد تكون أداة التوصيف المكوَّنة مشروعة؛ فلا تفسّر علامة تفعيل منفردة على أنها تحميل لملف DLL محدد.

### DOTNET_STARTUP_HOOKS {#dotnet_startup_hooks}

**معرّف الفحص:** `dotnet_startup_hooks` · **ATT&CK في الفهرس:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**ما يفحصه Juntei:** يقرأ قيم البيئة `DOTNET_STARTUP_HOOKS` للجهاز/المستخدم، والتي يمكنها اختيار تجميعات خطافات بدء التشغيل لأوقات تشغيل .NET المدعومة.

**الأدلة المفحوصة:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment\DOTNET_STARTUP_HOOKS` و`Environment\DOTNET_STARTUP_HOOKS` في NTUSER.DAT لكل مستخدم.

**كيفية التحقيق:** افحص القيمة بأكملها بحثًا عن تجميعات متعددة وتحقق من دعم وقت التشغيل. لا يعدّد هذا الفحص بيئات خطافات بدء التشغيل الخاصة بالخدمات، ولا يثبت أن أي عملية ورثت القيمة واستخدمتها.

### ملفات DLL المعروفة لتصحيح الشيفرة المُدارة {#known_managed_debugging_dlls}

**معرّف الفحص:** `known_managed_debugging_dlls` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ تسجيلات `KnownManagedDebuggingDlls` في `.NETFramework` بوصفها تجاوزات محتملة لمكتبات مصحح أخطاء الشيفرة المُدارة.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `KnownManagedDebuggingDlls` |

**كيفية التحقيق:** قارن مع أدوات التطوير/وقت التشغيل المثبتة وحدّد مصحح الأخطاء المشغِّل. ولا يثبت إدخال سجل Windows حدوث جلسة تصحيح أو تحميل مكتبة.

## تجاوزات البحث عن ملفات DLL والملفات التنفيذية { #dll-and-executable-lookup-overrides }

تفحص هذه الفحوص مواقع المكتبات الصريحة أو الإعدادات التي تؤثر في كيفية عثور مكوّن ما على الشيفرة. ويُعد تجاوز المسار أو إضعاف نهج البحث خيطًا تحقيقيًا؛ ولا تزال هناك حاجة إلى حمولة ومكوّن مشغِّل متوافق لحدوث التنفيذ.

### AppCert DLLs {#appcert_dlls}

**معرّف الفحص:** `appcert_dlls` · **ATT&CK في الفهرس:** [T1546.009](https://attack.mitre.org/techniques/T1546/009/)

**ما يفحصه Juntei:** يعدّد تسجيلات AppCertDlls غير الفارغة في Session Manager، وهي سطح امتداد للمكتبات مرتبط بواجهات برمجة التطبيقات الخاصة بإنشاء العمليات.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\AppCertDlls` | جميع القيم غير الفارغة |

**كيفية التحقيق:** افحص كل مكتبة ومصدر تثبيتها. حدّد قابلية التطبيق والتحميل وقت التشغيل كلًّا على حدة؛ فإنشاء العمليات العادي في جدول زمني ليس بحد ذاته دليلًا على أن ملف DLL المكوَّن قد حُمِّل.

### AppInit DLLs {#appinit_dlls}

**معرّف الفحص:** `appinit_dlls` · **ATT&CK في الفهرس:** [T1546.010](https://attack.mitre.org/techniques/T1546/010/)

**ما يفحصه Juntei:** يقرأ `AppInit_DLLs` بالعرض الأصلي وعرض 32 بت، ويقسّم قائمة مكتباتها، ويضيف سياق `LoadAppInit_DLLs` و`RequireSignedAppInit_DLLs` المتاح إلى كل مرشح.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows`: `AppInit_DLLs` و`LoadAppInit_DLLs` و`RequireSignedAppInit_DLLs`؛ وكذلك عرض `Wow6432Node` على مستوى الجهاز.

**كيفية التحقيق:** تُوسم علامة LoadAppInit_DLLs ذات القيمة صفر صراحةً بأنها غير نشطة. كما يؤثر إصدار Windows والإقلاع الآمن (Secure Boot) ومتطلبات التوقيع في التحميل. راجع الإعدادات الفعّالة قبل الجزم بأن ملف AppInit DLL قد نُفِّذ.

**مزيد من التفاصيل:** [وثائق Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/secure-boot-and-appinit-dlls).

### تجاوز AutodialDLL {#autodial_dll}

**معرّف الفحص:** `autodial_dll` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ تجاوز `AutodialDLL` في Parameters الخاصة بـ WinSock2.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters` | `AutodialDLL` |

**كيفية التحقيق:** تحقق من ميزة الشبكات وإصدار Windows المصدر اللذين يستخدمان هذا الإعداد. راجع ملف DLL الفعلي ونشاط الشبكة/العمليات؛ فالتجاوز وحده لا يثبت حدوث تحميل نشط أو ضار.

### ملف DLL مساعد لـ CHM {#chm_helper_dll}

**معرّف الفحص:** `chm_helper_dll` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ `Location` ضمن HtmlHelp Author، وهو سطح لتجاوز المكتبة/الموقع متعلق بالتعليمات.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\HtmlHelp Author` | `Location` |

**كيفية التحقيق:** حدّد ما إذا كان مكوّن التعليمات المعني يستخدم القيمة على هذا النظام، وافحص الملف المختار. ولا يُعد تجاوز سجل Windows دليلًا على فتح مستند CHM.

### تجاوز ملف DLL لـ NL6 ContentIndex {#content_index_dll}

**معرّف الفحص:** `content_index_dll` · **ATT&CK في الفهرس:** [T1574](https://attack.mitre.org/techniques/T1574/)

**ما يفحصه Juntei:** يقرأ `DLLOverridePath` من تسجيلات ContentIndex Language المباشرة في مجموعة التحكم النشطة.

**الأدلة المفحوصة:**

- `HKLM\SYSTEM\<ControlSet>\Control\ContentIndex\Language\<language>\DLLOverridePath`.

**كيفية التحقيق:** قد تختلف مكونات اللغة/البحث باختلاف إصدار Windows. افحص المكتبة والمكوّن المثبت المطابق؛ وتتبّع عملية الفهرسة الفعلية لتعزيز حدوث التحميل.

### ملف DLL لـ Crypto ExpoOffload {#crypto_expo_offload}

**معرّف الفحص:** `crypto_expo_offload` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ القيمة `ExpoOffload` في Cryptography Offload بوصفها مرشحًا لتجاوز مكتبة تشفير.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Cryptography\Offload` | `ExpoOffload` |

**كيفية التحقيق:** تأكد من انطباقه على الميزة/الإصدار وتحقق من أصل المكتبة المختارة. لا تعامل وجود قيمة غير شائعة على أنه دليل على سرقة بيانات الاعتماد أو على تنفيذ شيفرة.

### ملف DLL للمحوّل النقطي البرمجي في Direct3D {#direct3d_dll}

**معرّف الفحص:** `direct3d_dll` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ تجاوز المكتبة `SoftwareRasterizer` في Direct3D Drivers.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Direct3D\Drivers` | `SoftwareRasterizer` |

**كيفية التحقيق:** يمكن لبرامج الرسوميات والتوافق أن تؤثر في تكوين المحوّل النقطي (rasterizer). افحص ملف DLL ونشاط تطبيق Direct3D متوافق؛ فحفظ المسار لا يثبت أنه حُمِّل.

### إضعاف ترتيب البحث عن ملفات DLL {#dll_search_mode}

**معرّف الفحص:** `dll_search_mode` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يُبلغ عن `SafeDllSearchMode` أو `CWDIllegalInDllSearch` فقط عندما تُحلَّل قيمة Session Manager النشطة إلى صفر. وهذه شروط لنهج البحث، وليست حمولات مختارة.

**الأدلة المفحوصة:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager`: `SafeDllSearchMode` و`CWDIllegalInDllSearch`.

**كيفية التحقيق:** قد يسهّل نهج البحث المُضعَف هجومًا منفصلًا يستغل ترتيب البحث. لكنه لا يحدد ملف DLL مزروعًا ولا يثبت التنفيذ. ولا يُخرج هذا الفحص القيم غير الصفرية أو الغائبة؛ قيّم الإعدادات الافتراضية الفعّالة والنهج كلًّا على حدة.

**مزيد من التفاصيل:** [وثائق Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security).

### الاستثناء من KnownDLLs {#exclude_from_known_dlls}

**معرّف الفحص:** `exclude_from_known_dlls` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ إدخالات `ExcludeFromKnownDlls` غير الفارغة من Session Manager بوصفها تغييرات في سلوك البحث عن ملفات DLL المحمية/المعروفة.

**الأدلة المفحوصة:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\ExcludeFromKnownDlls`.

**كيفية التحقيق:** يحدد الاستثناء شرطًا للبحث، لا المكتبة البديلة نفسها. ابحث عن التطبيق المتأثر ومواقع البحث المحتملة عن ملفات DLL، ثم عزّز حدوث تحميل صورة غير قياسي.

**مزيد من التفاصيل:** [وثائق Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order).

### تجاوز ملف DLL لـ hhctrl.ocx {#hhctrl_ocx_dll}

**معرّف الفحص:** `hhctrl_ocx_dll` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ تجاوزات InprocServer32 في مخازن فئات الجهاز بالعرض الأصلي وعرض 32 بت ومخازن فئات المستخدم المدعومة، لمعرّف CLSID المنفَّذ لعنصر تحكم HTML Help.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKLM\SOFTWARE\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\Software\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |

**كيفية التحقيق:** حدّد التطبيق والبنية المعمارية اللذين يفعّلان عنصر التحكم. افحص ملف DLL المسجل والنشاط المتعلق بالتعليمات؛ فتسجيل COM لا يثبت أن HTML Help عرض محتوى.

### ملفات DLL المعروفة (Known DLLs) {#known_dlls}

**معرّف الفحص:** `known_dlls` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يعدّد القيم المسماة في المفتاح KnownDLLs ضمن Session Manager النشط.

**الأدلة المفحوصة:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\KnownDLLs`: القيم المسماة.

**كيفية التحقيق:** كثير من الإدخالات مكتبات Windows متوقعة، وبعض القيم تكوين داعم لا مسار حمولة. قارن المفتاح الكامل بإصدار موثوق مطابق؛ ولا يفحص هذا الفحص كل شجرة KnownDLLs بديلة خاصة بالبنى المعمارية.

### ملف DLL لامتدادات LSA {#lsa_extensions}

**معرّف الفحص:** `lsa_extensions` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ `Extensions` من تكوين LsaSrv في LsaExtensionConfig.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\LsaExtensionConfig\LsaSrv` | `Extensions` |

**كيفية التحقيق:** يمكن لبرامج الأمان أن توسّع مكونات المصادقة. افحص المكتبات الفعلية وتوافقها مع إصدار Windows المصدر، ثم ابحث عن أدلة تحميل الصور في LSASS أو غيرها من أدلة الاستخدام المستقلة.

### تجاوز مسار ملف DLL لـ MAPI32 {#mapi32_dll_path}

**معرّف الفحص:** `mapi32_dll_path` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ `DLLPathEx` و`DLLPath` ضمن تسجيلات عملاء البريد (Mail) المباشرة في عرضي الجهاز الأصلي و32 بت.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPath` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPath` |

**كيفية التحقيق:** تسجّل عملاء البريد موفري MAPI مشروعين. حدّد العميل/الموفر الذي اختير وافحص المكتبة الفعلية؛ فالموفر المثبت لا يثبت أن تطبيقًا استخدم MAPI أثناء الحادثة.

### ملف DLL لـ MSDTC XA {#msdtc_xa_dll}

**معرّف الفحص:** `msdtc_xa_dll` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ `OracleXaLib` و`OracleOciLib` من تكوين MTxOCI في MSDTC.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleXaLib` |
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleOciLib` |

**كيفية التحقيق:** قد يتطلب التكامل مع قواعد البيانات هذه المكتبات بصورة مشروعة. تحقق من تكوين منسّق المعاملات وبرامج قواعد البيانات المثبتة، واربط ذلك باستخدام الخدمة/الوحدة بدلًا من افتراض أن مسارًا مكوَّنًا قد عمل.

### تجاوز ملف DLL لـ NLDP {#nldp_dll}

**معرّف الفحص:** `nldp_dll` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ قيمة التكوين `DllOverridePath` في NlsData.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NlsData` | `DllOverridePath` |

**كيفية التحقيق:** راجع المكتبة الهدف وإصدار Windows ومكوّن معالجة اللغة الطبيعية الذي قد يستخدم التجاوز. ولا تُظهر القيمة أن الفهرسة أو معالجة اللغة قد حدثت.

### ملف DLL لـ TestDVCPlugin في RDP {#rdp_test_dvc_plugin}

**معرّف الفحص:** `rdp_test_dvc_plugin` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ إعداد المكتبة `TestDVCPlugin` في Terminal Server Client على مستوى الجهاز.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Terminal Server Client` | `TestDVCPlugin` |

**كيفية التحقيق:** تحقق من أي مكوّن إضافي تجريبي غير مألوف ومن السلوك الخاص بالإصدار. يلزم وجود جلسة عميل متوافقة وتحميل للمكتبة لإثبات التنفيذ؛ ولا تُظهر قيمة سجل Windows وحدها أيًّا منهما.

### تجاوز ملف DLL لمفهرس البحث {#search_indexer_dll}

**معرّف الفحص:** `search_indexer_dll` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ تجاوز `DllPath` الخاص بـ Windows Search من SOFTWARE.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Search` | `DllPath` |

**كيفية التحقيق:** حدّد مكوّن Search/الإصدار المطابق وافحص الملف الهدف. فخدمة Windows Search غير النشطة أو الإعداد غير المستخدم أمر مختلف عن تحميل مكتبة ضارة مُلاحَظ.

### ملف DLL لـ Winsock AutoProxy {#winsock_auto_proxy}

**معرّف الفحص:** `winsock_auto_proxy` · **ATT&CK في الفهرس:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**ما يفحصه Juntei:** يقرأ `LibraryPath` ضمن التسجيلات المباشرة لـ Catalog_Entries وCatalog_Entries64 في NameSpace_Catalog5.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries\<child>` | `LibraryPath` |
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries64\<child>` | `LibraryPath` |

**كيفية التحقيق:** قد يكون موفرو الشبكات مشروعين. راجع كلتا البنيتين المعماريتين، ومصدر الموفر، وأدلة تطبيقات الشبكة/تحميل الصور ذات الصلة. ولا يشمل هذا البحث الموجّه عن موفري مساحة الأسماء كل امتداد ممكن لـ Winsock.

## الحسابات وظهورها عند تسجيل الدخول { #accounts-and-sign-in-visibility }

يمكن لسجلات الحسابات المحلية وإعدادات الظهور عند تسجيل الدخول أن تكشف عن استمرارية قائمة على الهوية. تفحص هذه الفحوص خليتي SAM وSOFTWARE الملتقطتين؛ وهي ليست جردًا لحسابات Active Directory، أو عضوية مجموعات النطاق، أو عمليات تسجيل الدخول السابقة.

### العضوية في مجموعة Administrators {#admin_group_membership}

**معرّف الفحص:** `admin_group_membership` · **ATT&CK في الفهرس:** [T1098](https://attack.mitre.org/techniques/T1098/)

**ما يفحصه Juntei:** يفك ترميز قائمة أعضاء الاسم المستعار (alias) لمجموعة Administrators المحلية من SAM. ويُبلغ عن معرّفات SID الخاصة بالجهاز المحلي التي تسمّي حسابًا مخفيًا أو لا تسمّي أي حساب/مجموعة في SAM الملتقط.

**الأدلة المفحوصة:**

- `SAM\Domains\Builtin\Aliases\00000220\C` وسجلات SID الجهاز/المستخدمين/المجموعات ضمن `SAM\Domains\Account` في خلية SAM.
- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList` للربط بالأسماء المخفية.

**كيفية التحقيق:** هذا فحص موجّه للحالات الشاذة، وليس قائمة بجميع المسؤولين. ولا تُقيَّم معرّفات SID الخارجية/الخاصة بالنطاق على أنها مستخدمون محليون مفقودون. وتتطلب الحسابات المحذوفة، أو توقيت الالتقاط، أو الخلايا غير المكتملة تحقيقًا قبل استنتاج وجود عضوية ضارة.

### حساب محلي مخفي {#hidden_account}

**معرّف الفحص:** `hidden_account` · **ATT&CK في الفهرس:** [T1564.002](https://attack.mitre.org/techniques/T1564/002/)

**ما يفحصه Juntei:** يُبلغ عن القيم الرقمية صفر في UserList ضمن SpecialAccounts في Winlogon، محددًا الأسماء المخفية عن شاشة تسجيل الدخول العادية.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`: قيم أسماء الحسابات.

**كيفية التحقيق:** قد تُخفى حسابات الخدمات أو الصيانة عمدًا، ولا تثبت القيمة أن الحساب المسمى موجود أو أنه مسؤول. اربطها بسجلات الحسابات في SAM ونشاط تسجيل الدخول.

### اختطاف RID {#rid_hijacking}

**معرّف الفحص:** `rid_hijacking` · **ATT&CK في الفهرس:** [T1098](https://attack.mitre.org/techniques/T1098/)

**ما يفحصه Juntei:** يقارن قيمة RID الست عشرية للمفتاح الفرعي لحساب في SAM بقيمة RID المخزنة عند الإزاحة 0x30 في القيمة F الخاصة به، ويُبلغ عن حالات عدم التطابق.

**الأدلة المفحوصة:**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` في خلية SAM.

**كيفية التحقيق:** عدم التطابق حالة شاذة خطيرة في سجل الحساب، لكنه قد يتطلب أيضًا تحليلًا للتلف/الالتقاط. افحص بايتات المصدر، وحالة سجل المعاملات، وسجلات الحسابات/تسجيل الدخول؛ ولا يثبت ذلك أي جهة عدّلت السجل.

### RID Suborner (حساب مسؤول مخفي) {#rid_suborner}

**معرّف الفحص:** `rid_suborner` · **ATT&CK في الفهرس:** [T1098](https://attack.mitre.org/techniques/T1098/)

**ما يفحصه Juntei:** يُبلغ عن حسابات SAM التي لا تكون قيمة RID لمفتاحها الفرعي 500 لكن قيمة RID في F تساوي 500، وهو نمط أكثر تحديدًا لاستبدال هوية المسؤول.

**الأدلة المفحوصة:**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` في خلية SAM.

**كيفية التحقيق:** يتداخل هذا مع الفحص العام لعدم تطابق RID. ولا يشترط بشكل مستقل وجود إعداد إخفاء عند تسجيل الدخول رغم عنوانه في الفهرس؛ تأكد من ظهور الحساب وامتيازاته واستخدامه الفعلي كلًّا على حدة.

## مكونات المصادقة وحماية بيانات الاعتماد { #authentication-components-and-credential-protection }

تتولى Local Security Authority (LSA) وظائف الأمان من خلال LSASS والمكونات المرتبطة به. ويمكن لحزم المصادقة ومرشحات كلمات المرور وموفري الشبكة أن تكون برامج أمان مشروعة أو طريقًا للوصول المستمر إلى بيانات الاعتماد. ويجرد فحصان إعدادات الحماية بدلًا من عناصر تشغيل تلقائي قابلة للتنفيذ. فسّرهما باستخدام دور المضيف وإصدار Windows والنهج الفعّال.

### ملف DLL لموفر AMSI {#amsi_providers}

**معرّف الفحص:** `amsi_providers` · **ATT&CK في الفهرس:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**ما يفحصه Juntei:** يعدّد معرّفات CLSID لموفري AMSI على مستوى الجهاز ويحل مكتبات InprocServer32 الخاصة بها بالعرض الأصلي وعرض 32 بت عند توفرها. وتحتفظ التسجيلات غير المحلولة بمعرّف الفئة.

**الأدلة المفحوصة:**

- `HKLM\SOFTWARE\Microsoft\AMSI\Providers\<CLSID>` وتسجيلات InprocServer32 القابلة للحل في `Classes\CLSID` / `Classes\Wow6432Node\CLSID`.

**كيفية التحقيق:** تسجّل برامج مكافحة الفيروسات ومنتجات الأمان موفري AMSI بصورة مشروعة. راجع مصدر الموفر/المكتبة والتحميل الفعلي؛ فالتسجيل لا يثبت تجاوز AMSI أو أن الموفر ضار.

### حزم المصادقة {#authentication_packages}

**معرّف الفحص:** `authentication_packages` · **ATT&CK في الفهرس:** [T1547.002](https://attack.mitre.org/techniques/T1547/002/)

**ما يفحصه Juntei:** يقرأ إدخالات `Authentication Packages` غير الفارغة في LSA كلًّا على حدة من مجموعة التحكم النشطة.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Authentication Packages` |

**كيفية التحقيق:** قارن مع إصدار Windows المصدر وبرامج المصادقة المثبتة. قد تكون الحزمة غير الافتراضية مشروعة. احلل ملف DLL وعزّز أدلة التحميل/نشاط بيانات الاعتماد قبل استنتاج حدوث اعتراض.

### سلوك تسجيل دخول مسؤول DSRM {#dsrm_backdoor}

**معرّف الفحص:** `dsrm_backdoor` · **ATT&CK في الفهرس:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**ما يفحصه Juntei:** يُبلغ عن القيمة الرقمية `DsrmAdminLogonBehavior=2` في المواقع المنفَّذة ControlSet001 أو ControlSet002 أو CurrentControlSet. ويمكن لهذا الإعداد أن يسمح باستخدام حساب DSRM خارج سياق الاسترداد المعتاد له.

**الأدلة المفحوصة:**

- `HKLM\SYSTEM\ControlSet001\Control\Lsa\DsrmAdminLogonBehavior`، إضافةً إلى ControlSet002 وCurrentControlSet عند وجودهما.

**كيفية التحقيق:** يتعلق أساسًا بوحدات التحكم بالمجال. حدّد مجموعة التحكم النشطة باستخدام Select في SYSTEM؛ فالتطابق في مجموعة تحكم تاريخية ليس فعّالًا بالضرورة. ولا تثبت القيمة أن مهاجمًا عرف كلمة مرور DSRM أو سجّل الدخول.

### تكوين Credential Guard {#lsa_cfg_flags}

**معرّف الفحص:** `lsa_cfg_flags` · **ATT&CK في الفهرس:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**ما يفحصه Juntei:** يقرأ قيمة LSA المكوَّنة `LsaCfgFlags`، بما في ذلك قيم التفعيل العادية وكذلك الصفر. ويجرد هذا تكوين Credential Guard بدلًا من عنصر تشغيل تلقائي قابل للتنفيذ.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `LsaCfgFlags` |

**كيفية التحقيق:** فسّر القيمة في ضوء إصدار Windows، والمتطلبات المسبقة للبرامج الثابتة (firmware)/الأمان، والنهج الفعّال. ولا تثبت القيمة صفر المخزنة أو الغائبة وحدها حدوث تغيير من قِبل مهاجم، ولا حالة الحماية الفعلية، ولا سرقة بيانات الاعتماد.

### مرشح كلمات المرور في LSA {#lsa_password_filter}

**معرّف الفحص:** `lsa_password_filter` · **ATT&CK في الفهرس:** [T1556.002](https://attack.mitre.org/techniques/T1556/002/)

**ما يفحصه Juntei:** يقرأ `Notification Packages` في LSA. وتُحَل أسماء المكتبات المجردة مقابل مجلد Windows System32 الملتقط، مع إضافة `.dll` حيثما كان ذلك مناسبًا.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Notification Packages` |

**كيفية التحقيق:** تثبّت منتجات نهج كلمات المرور عادةً مرشحات. افحص المكتبة وأدلة التحميل، وميّز بين المرشحات المكوَّنة والدليل على حدوث تغيير لكلمة المرور أو اعتراض لبيانات الاعتماد.

### حالة حماية PPL لـ LSASS {#lsa_run_as_ppl}

**معرّف الفحص:** `lsa_run_as_ppl` · **ATT&CK في الفهرس:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**ما يفحصه Juntei:** يقرأ قيمة LSA المكوَّنة `RunAsPPL`، بما في ذلك القيم غير الصفرية. وهو جرد لإعداد الحماية، وليس اختبارًا لما إذا كان LSASS قد عمل دون حماية.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `RunAsPPL` |

**كيفية التحقيق:** تعتمد حماية LSA الفعلية على إصدار Windows، والنهج، وقفل البرامج الثابتة، وحالة إعادة التشغيل. وعدم وجود صف لقيمة غائبة ليس نتيجة حماية مُقاسة. استخدم أدلة مستقلة من النظام/وقت التشغيل لتقييم الحماية الفعلية.

### ملف DLL لموفر الشبكة {#network_provider_dll}

**معرّف الفحص:** `network_provider_dll` · **ATT&CK في الفهرس:** [T1556.008](https://attack.mitre.org/techniques/T1556/008/)

**ما يفحصه Juntei:** يقرأ `NetworkProvider\ProviderPath` ضمن الخدمات في مجموعة التحكم النشطة في SYSTEM.

**الأدلة المفحوصة:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\NetworkProvider\ProviderPath`.

**كيفية التحقيق:** قد يكون موفرو تسجيل الدخول عبر الشبكة مشروعين. تحقق من ترتيب الموفرين/قابلية تطبيقهم، وبرامج الشبكات/الأمان المثبتة، والتحميل الفعلي للمكتبة؛ ولا تثبت قيمة ProviderPath وحدها اعتراض بيانات الاعتماد.

### حزم الأمان {#security_packages}

**معرّف الفحص:** `security_packages` · **ATT&CK في الفهرس:** [T1547.005](https://attack.mitre.org/techniques/T1547/005/)

**ما يفحصه Juntei:** يقرأ إدخالات `Security Packages` غير الفارغة في LSA كلًّا على حدة، والتي تغطي موفري دعم الأمان (security support providers) المكوَّنين.

**الأدلة المفحوصة:**

| مفتاح سجل Windows | القيمة المفحوصة |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Security Packages` |

**كيفية التحقيق:** قارن مع خط الأساس المتوقع لـ Windows/منتجات الأمان وافحص كل مكتبة. ولا يثبت التكوين أن موفرًا قد حُمِّل، أو التقط بيانات اعتماد، أو أُضيف أثناء الحادثة.

## من المرشح إلى الاستنتاج { #work-from-a-candidate-to-a-conclusion }

1. افتح المصدر الأصلي الذي يحدده `SourceFile`. تأكد من القيمة الكاملة، أو إجراء XML، أو
   البرنامج النصي، أو سجل قاعدة البيانات المسترد، ومن المستخدم/المضيف الدقيق الذي ينتمي إليه.
2. حدّد ما إذا كان يمكن للآلية أن تعمل: تحقق من علامات التفعيل، وإصدار وقت التشغيل أو Windows،
   وحالة الخدمة/المهمة، والمشغّل المطلوب، والمستخدم، والبنية المعمارية، ومجموعة التحكم النشطة.
3. افحص البرنامج أو ملف DLL أو البرنامج النصي أو التجميع المختار. احسب تجزئة البايتات المجمّعة وراجع الوسيطات
   والمحتوى. ولا يحسم وجود مشغّل Windows مألوف أو سلسلة ناشر معروفة مسألة المشروعية.
4. حدّد ما حدث فعلًا باستخدام أدلة مستقلة. ومن الأمثلة على ذلك إنشاء العمليات،
   وعمليات تحميل الصور في Sysmon، وسجلات وقت التشغيل في Task Scheduler، ونشاط الخدمات، وPrefetch، والملفات ذات الصلة.
   ويمكن لحدث تغيير التكوين أن يثبت أن الإعداد قد تغيّر دون أن يثبت أن حمولته قد عملت.
5. ابنِ جدولًا زمنيًا باستخدام الطوابع الزمنية من المصادر الأصلية ومصدر الربط المسجل.
   واحتفظ بعمليات الكتابة في المفاتيح، وتغييرات التثبيت، وعمليات الكتابة في الملفات/الحاويات، وعمليات التنفيذ بوصفها حقائق منفصلة.
6. سجّل القيود إلى جانب النتائج: الخلايا المفقودة، والملفات الهدف المفقودة، والجهات الأساسية للمهام المجهولة،
   والبيانات الوصفية للموقِّع غير المتحقق منها، والسجلات التاريخية المستخلصة، أو المشغّل الذي لا يمكن تعزيزه.

للاطلاع على صيغة الأوامر، والملفات الشخصية، وكل حقل من حقول الإخراج، وحالات التغطية، عُد إلى
[دليل أمر Persistence Hunter](../commands/persistence-hunter.md). وتشرح
[أدلة الآثار الرقمية](../artifacts/index.md) المصادر الأساسية، وتحدد صفحة
[شكر وتقدير](acknowledgements.md) أدوات الاستمرارية الأصلية.
