# بيانات تكوين الإقلاع (BCD) { #boot-configuration-data-bcd }

## ما هو مخزن BCD { #what-the-bcd-store-is }

يحتفظ Windows Vista والإصدارات الأحدث بتكوين الإقلاع في خلية من خلايا سجل Windows هي مخزن BCD:
`\Boot\BCD` على أنظمة BIOS، و`\EFI\Microsoft\Boot\BCD` على قسم نظام EFI في أنظمة
UEFI. ويعدّله `bcdedit` ومثبّت Windows؛ ويقرؤه مدير الإقلاع عند كل إقلاع.

يحتوي المخزن على كائنات، يُسمى كل منها بمعرّف GUID:

- **مدير إقلاع Windows** ‏(`{bootmgr}`)، مع الإدخال الافتراضي وترتيب العرض ومهلة
  القائمة؛ وعلى أنظمة UEFI أيضًا **مدير إقلاع البرنامج الثابت** ‏(`{fwbootmgr}`) مع ترتيب الإقلاع
  الخاص بالبرنامج الثابت؛
- **مُحمِّل إقلاع Windows** واحد لكل تثبيت لـ Windows (بما في ذلك بيئة الاسترداد)، و**تطبيق
  الاستئناف** (السبات) و**أداة اختبار الذاكرة**؛
- كائنات **الإعدادات الموروثة** (`{globalsettings}`، `{bootloadersettings}`، …) التي ترث
  الكائنات الأخرى إعداداتها؛
- كائنات **الأجهزة** (خيارات قرص RAM الخاصة ببيئة الاسترداد).

لكل كائن عناصر: وصف، وجهاز ومسار، وإعدادات مثل
`recoveryenabled` و`bootstatuspolicy` و`testsigning` و`nointegritychecks` و`safeboot`.

يغيّرها نوعان من الهجمات:

- **برامج الفدية** تعطّل الاسترداد كي لا يمكن إصلاح النظام:
  `bcdedit /set {default} recoveryenabled No` و
  `bcdedit /set {default} bootstatuspolicy ignoreallfailures`. وتضبط بعض العائلات `safeboot` بحيث
  يُعاد تشغيل Windows في الوضع الآمن، حيث لا تعمل منتجات الأمان.
- **أدوات الجذر (Rootkits) وبرامج التشغيل غير الموقعة** قد تستخدم `testsigning on` أو `nointegritychecks on` لإضعاف فحوصات وقت الإقلاع. ولا تثبت هذه الإعدادات وحدها أن أداة جذر قد عملت.

<span class="evidence-strength">أفضل استنتاج:</span> في وقت الجمع، كان لإدخال الإقلاع هذا
هذا الإعداد. ووقت آخر كتابة لمفتاح العنصر هو آخر وقت كُتب فيه الإعداد.

## الموقع { #location }

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

نوع العنصر 8 أرقام سداسية عشرية: فئته (`1` كل التطبيقات، `2` نوع واحد من
التطبيقات، `3` الأجهزة)، وتنسيقه (`1` جهاز، `2` سلسلة نصية، `3` كائن، `4` قائمة كائنات، `5`
عدد صحيح، `6` قيمة منطقية، `7` قائمة أعداد صحيحة) ورقمه. ويسمّي الأمر العناصر كما
توثقها Microsoft ‏(`BcdLibraryBoolean_AutoRecoveryEnabled`).

كثيرًا ما لا تنسخ أدوات الجمع المخزن: فقسم نظام EFI غير مُركَّب، و`\Boot`
مخفي. اجمعه صراحةً، أو قم بتركيب القسم. ويقرأ `-d` أيضًا `BCD-Template`
(يوضح `HiveType` أيهما)؛ والقالب هو ما يبدأ منه Windows، وليس المخزن الذي يُقلع به.

## تحذيرات للمحلل { #analyst-cautions }

- **الإعدادات موروثة.** يرث مُحمِّل الإقلاع من الكائنات الموجودة في
  `BcdLibraryObjectList_InheritedObjects` الخاص به (عادةً `{bootloadersettings}`، الذي يرث
  `{globalsettings}`). وتمثل `RecoveryEnabled` و`TestSigning` و`NoIntegrityChecks` و`SafeBoot`
  و`BootStatusPolicy` قيمًا مخزنة بعد حلّها (القيمة الخاصة بالكائن، وإلا فالموروثة)، ويوضح `Inherited` أيها
  يأتي من كائن آخر. ويُتتبع التوريث لثمانية مستويات كحد أقصى، مع تقييد الحلقات. وقد يكون كائن مُشار إليه غائبًا عن مخزن جزئي. ولا تُثبت الحقول الفارغة قيمة Windows الافتراضية الفعلية. وقد يمنع البرنامج الثابت وSecure Boot أيضًا خيارًا مخزنًا من أن يسري.
- **عدم الضبط لا يعني التعطيل.** تعني `RecoveryEnabled` الفارغة أن العنصر غير موجود؛ وتنطبق
  القيمة الافتراضية لـ Windows.
- **الأوقات لكل عنصر على حدة.** وقت مفتاح كل عنصر (`key_last_written` في `Elements`) هو آخر وقت
  كُتب فيه. ويعيد Windows كتابة بعض العناصر عند التحديثات والترقيات؛ فقارن بأوقات العناصر
  الأخرى.
- **الأجهزة مفكوكة الترميز جزئيًا.** يُعطى قسم GPT على هيئة معرّفي GUID للقسم والقرص؛
  وقرص RAM على هيئة كائن خياراته. وتحتفظ الأجهزة الأخرى ببايتاتها فقط (`data_raw`).
- **العناصر غير الموثقة** (`0x15000065`، الفئة `0x4…`) ليس لها اسم وتحتفظ ببايتاتها.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.TestSigning == true or .NoIntegrityChecks == true or .RecoveryEnabled == false)'
```

يسرد الملخص الإدخال الافتراضي لكل مخزن وكل إعداد لتوقيع الاختبار، وفحص السلامة، والاسترداد،
ونهج حالة الإقلاع (أي `Ignore…`) والوضع الآمن، مع الوقت الذي ضُبط فيه أو الكائن
الذي ورثه منه.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `SourcePath`، `HiveType` | المضيف، والخلية، و`BCD` أو `BCD-Template`. |
| `ObjectGUID`، `ObjectType`، `ObjectTypeName` | معرّف GUID للكائن، ونوعه (`0x10200003`) وماهيته (`Windows boot loader`، `inherited settings for Windows boot loader`، …). |
| `Description`، `Path`، `Device` | الوصف، ومسار التطبيق، وجهازه (قسم GPT وقرص، أو كائن خيارات قرص RAM). |
| `IsDefault`، `DisplayOrder` | ما إذا كان مدير إقلاع Windows يُقلعه افتراضيًا، وموضعه في قائمة الإقلاع. |
| `RecoveryEnabled`، `TestSigning`، `NoIntegrityChecks` | `recoveryenabled`، `testsigning`، `nointegritychecks`، بعد حلّها من المخزن. |
| `SafeBoot`، `BootStatusPolicy` | `safeboot` (`Minimal`، `Network`، `DsRepair`) و`bootstatuspolicy` (`IgnoreAllFailures`، …)، مع الرقم. |
| `Inherited` | أيٌّ من هذه يأتي من كائن آخر، ومن أي كائن. |
| `Elements` | كل عنصر: نوعه، واسمه، وقيمته، وبايتاته (للأجهزة والأنواع غير الموثقة والبيانات التي يتعذر فك ترميزها) ووقت مفتاحه. |
| `KeyLastWrittenUTC` | آخر كتابة لمفتاح الكائن. |
| `TransactionLogStatus`، `LogsApplied` | سياق استعادة سجل Windows. |
| `Problems` | كائنات بلا نوع، وعناصر لا تتوافق بياناتها مع تنسيقها. |

## استراتيجية التحقيق { #investigation-strategy }

1. افحص الإدخال الافتراضي وكل مُحمِّل إقلاع بحثًا عن `TestSigning` و`NoIntegrityChecks`
   و`RecoveryEnabled = false` و`BootStatusPolicy` من نوع `Ignore…` و`SafeBoot`.
2. استخدم أوقات العناصر لتأريخ التغيير، واربطها بـ `bcdedit.exe` في Prefetch
   وAmcache وأحداث إنشاء العمليات (4688، Sysmon 1) وسجل أحداث System.
3. ابحث عن إدخالات إقلاع أو أجهزة لا ينبغي وجودها (مسار مُحمِّل آخر، قرص RAM)، و
   عن تغيير في الإدخال الافتراضي أو ترتيب العرض.
4. قارن المخزن بـ `BCD-Template` وبنظام آخر من الإصدار نفسه.

## مزيد من التفاصيل { #further-details }

- [Microsoft: مرجع خيارات إقلاع BCD](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft: BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft: BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft: BcdObject (أنواع الكائنات)](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
