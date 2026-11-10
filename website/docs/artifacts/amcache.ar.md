# Amcache { #amcache }

## ما هو Amcache { #what-amcache-is }

Amcache هو جرد لتوافق التطبيقات في Windows مخزّن في خلية من خلايا سجل Windows. تسجّل الأنظمة الحديثة
الملفات التنفيذية تحت `InventoryApplicationFile` وأجهزة التوصيل والتشغيل (Plug-and-Play) تحت
`InventoryDevicePnp`؛ بينما تستخدم إصدارات Windows الأقدم تخطيط `Root\File` القديم.

في التحقيق في الملفات التنفيذية، يُعد Amcache قيّمًا لأنه قد يحتفظ بالمسار الكامل، وتجزئة SHA-1، وحجم
الملف، وبيانات PE الوصفية، ومعلومات الإصدار، ونص الناشر، وتاريخ الربط (link date)، والطوابع الزمنية في
سجل Windows بعد اختفاء الملف الأصلي. ويمكن لسجلات الأجهزة أن تحدد الأجهزة الموصولة، وبرامج التشغيل،
وتواريخ التثبيت، والمعرّفات.

<span class="evidence-strength">أفضل استنتاج:</span> أدرج Windows الملف أو الجهاز المسجل في الجرد.
وسجل الملف التنفيذي **دليل على الوجود، لا إثبات للتنفيذ**.

## الموقع والجمع { #location-and-collection }

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

اجمع الخلية من نظام التشغيل المتأثر. يحلل `parse-registry amcache` حاليًا الخلية الأساسية لكنه **لا**
يعيد تطبيق سجلات معاملات Amcache المجاورة، لذا قد تكون السجلات الموجودة في تلك السجلات فقط مفقودة من
مجموعة حية أو غير متسقة. دوّن هذا القيد في التقرير.

## تحذيرات للمحلل { #analyst-cautions }

- لا يثبت إدخال الملف التنفيذي في Amcache أن مستخدمًا شغّل الملف.
- `Publisher` بيانات وصفية من PE/موارد الإصدار سجّلها Windows. ولا يثبت أن الملف كان يحمل توقيع
  Authenticode صالحًا وقت الجمع.
- قد تكون التجزئات غائبة، ولا تعرض المخططات القديمة/الحديثة قيمًا متطابقة.
- يأتي `LinkDate` من ترويسة PE، ويمكن تزويره أو نسخه أو إنتاجه عبر عمليات بناء قابلة لإعادة الإنتاج
  (reproducible builds).
- يصف وقت آخر كتابة للمفتاح تحديث سجل سجل Windows، وليس بالضرورة التنزيل أو الإنشاء أو التنفيذ الأول
  أو التنفيذ الأخير.
- قد تشير المسارات إلى وحدات تخزين قابلة للإزالة أو لم تعد موجودة.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.RecordType == "file" and .SHA1 == "0123456789abcdef0123456789abcdef01234567")'
```

يحتوي إخراج CSV على جداول الملخص والنتائج والملفات والأجهزة. ومع `auto-parse`، تكون هذه الجداول
`Registry-Amcache-Summary.csv` و`Registry-Amcache-Findings.csv` و`Registry-Amcache-Files.csv`
و`Registry-Amcache-Devices.csv` مباشرةً تحت مجلد `FileExistence/` لكل كمبيوتر، مع
`Registry-Amcache-logs.txt` بجانب الجداول. وتضيف الخلايا المتعددة `-Source1` و`-Source2` وما إلى ذلك
قبل الامتداد؛ ويسرد `OutputFiles` في ملفات البيان (manifests) المسارات الدقيقة. أما عمليات تصدير CSV
المستقلة فتحتفظ بأسماء جداولها الأصلية وبموقع الإخراج المطلوب. وتستخدم سجلات JSONL قيم
`RecordType` التالية: `summary` و`finding` و`file` و`device`.

## حقول إخراج الملفات { #file-output-fields }

| الحقل | كيف ينبغي للمحلل استخدامه |
|---|---|
| `KeyName`، `SourceKeyPath` | يحددان سجل سجل Windows المصدر الدقيق وفرع المخطط. احتفظ بهما لقابلية إعادة الإنتاج. |
| `KeyLastWrittenUTC`، `KeyLastWrittenFiletime` | وقت تحديث سجل سجل Windows، بصيغة مقروءة وخام. لا تسمّه وقت تنفيذ. |
| `SourceVolumeKeyLastWrittenUTC`، `SourceVolumeKeyLastWrittenFiletime` | وقت مفتاح وحدة التخزين الأم في المخططات القديمة؛ سياقي، وليس وقت إجراء على الملف. |
| `FileID` | معرّف الملف الذي سجّله Amcache. يُحتفظ به كما هو تمامًا؛ لا تفترض أنه دائمًا SHA-1. |
| `SHA1`، `SHA256` | نقاط محورية قوية إلى Defender واستخبارات التهديدات وEDR ومستودعات الملفات. تعني القيمة null أنها لم تُسجَّل. |
| `Name`، `OriginalFileName`، `Description` | أسماء PE/الإصدار المسجلة. قارنها بالمسار الفعلي لكشف الانتحال؛ فالبيانات الوصفية خاضعة لتحكم المهاجم. |
| `FullPath` | أكثر نقاط الموقع المحورية فائدة. وحّد حالة الأحرف بعناية، لكن احتفظ بالسلسلة الأصلية للدليل. |
| `LongPathHash` | معرّف مرتبط بالمسار في Windows؛ مفيد لمطابقة السجلات، وليس تجزئة للمحتوى. |
| `ProgramID` | يربط الملف بسجل جرد التطبيقات حيثما كان متاحًا. |
| `Publisher` | نص الناشر المسجل. تعامل معه على أنه بيانات وصفية، لا تحقق من التوقيع. |
| `Version`، `BinFileVersion`، `ProductName`، `ProductVersion`، `BinProductVersion` | سياق موارد الإصدار. يمكن للاختلافات أو القيم غير المعقولة أن تدعم تحليل الانتحال. |
| `BinaryType` | سياق البنية المعمارية/النوع المسجل؛ قارنه بتحليل PE عند استرداد الملف. |
| `LinkDate` | الطابع الزمني لترجمة/ربط PE كما هو مسجل. لا يفيد إلا مع التشكك والتعزيز. |
| `Size` | الحجم المسجل؛ قارنه بـ `$MFT` أو بمحتوى Defender أو بملف مسترد. |
| `Language` | معرّف لغة PE؛ دليل سياقي ضعيف، لا إسناد. |
| `IsPE-File`، `IsOS-Component` | علامات التصنيف التي سجّلها Windows. ولا تثبت القيمة false أن الملف غير ضار. |
| `USN` | قيمة تسلسل التحديث في NTFS التي سجّلها Amcache؛ قد تساعد على الانتقال إلى سجلات USN الباقية. |
| `APPX-PackageFullName`، `APPX-PackageRelativeID` | يربطان التطبيقات/المكونات المحزّمة بالملف. |
| `RegistryValues` | كل قيمة أصلية في سجل Windows مع نوعها، وقيمتها بعد فك الترميز، وصيغتها الست عشرية الخام. استخدمها عندما يكون حقل محدد النوع غائبًا أو للحقول المستقبلية/القديمة. |

## حقول إخراج الأجهزة { #device-output-fields }

| مجموعة الحقول | الحقول وتفسيرها |
|---|---|
| المصدر | `KeyName`، `SourceKeyPath`، `KeyLastWrittenUTC`، `KeyLastWrittenFiletime`: سجل الجهاز الدقيق ووقت تحديث سجل Windows. |
| الهوية | `Description`، `BusDescription`، `Manufacturer`، `Model`: أوصاف الجهاز الموجهة للبشر؛ وقد تكون سلاسل المورّد عامة. |
| معرّفات العتاد | `HWID`، `CompID`، `Class`، `ClassGUID`، `ContainerID`، `DeviceInterfaceClasses`: أقوى النقاط المحورية لعائلة الجهاز ونسخته. |
| الحالة | `DeviceState`، `ProblemCode`، `ParentID`، `MatchingID`، `Enumerator`: سياق التعداد في Windows والعلاقات. |
| برنامج التشغيل | `DriverID`، `DriverName`، `DriverPackageStrongName`، `DriverVerDate`، `DriverVerVersion`، `Provider`، `Service`، `INF`: ارتباطات حزمة برنامج التشغيل والخدمة؛ اربطها بالملفات والخدمات المثبتة. |
| التثبيت | `InstallDate`، `FirstInstallDate`، `InstallState`، `ExtendedInfs`: دورة حياة التثبيت المسجلة. ولا يثبت غياب التاريخ أن الجهاز لم يُوصل قط. |
| المرشحات والمكدس | `LowerClassFilters`، `UpperClassFilters`، `LowerFilters`، `UpperFilters`، `StackID`، `LocationPaths`: سياق متقدم لمكدس برامج التشغيل؛ وتستحق برامج تشغيل المرشحات غير المتوقعة المراجعة. |
| الحفظ الخام | `RegistryValues`: القيم المصدرية الكاملة محددة النوع/الخام للتحقق وللحقول غير المدعومة. |

تستخدم أعمدة المصفوفات والقيم الخام في CSV اللاحقة `JSON` (على سبيل المثال، `HWID-JSON` و
`RegistryValuesJSON`)؛ وتحتوي خلاياها على JSON، بينما تحتفظ JSON/JSONL بالمصفوفات الأصلية.

## استراتيجية التحقيق { #investigation-strategy }

1. انطلق من `FullPath` و`SHA1` و`Name` و`USN` نقاطًا محورية.
2. ابحث عن Prefetch مطابق لدعم استنتاج التنفيذ.
3. استخدم `$MFT` ودفتر يومية USN لإثبات إنشاء الملف وإعادة تسميته وحذفه وحجمه.
4. قارن البيانات الوصفية للناشر/الإصدار بملف PE مسترد بشكل مستقل وبالتحقق من التوقيع.
5. افحص حجر Defender وسجلات الأحداث بحثًا عن عمليات الاكتشاف أو الإزالة.

## مزيد من التفاصيل { #further-details }

- [مخطط إخراج ys-amcache-parser الخاص بـ Juntei](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [أداة AmcacheParser من Eric Zimmerman](https://github.com/EricZimmerman/AmcacheParser)
- [amcache-forensic من Security Ronin](https://github.com/SecurityRonin/amcache-forensic)
- [إضافة Amcache في Regipy](https://github.com/mkorman90/regipy)
