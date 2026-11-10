# حجر Microsoft Defender { #microsoft-defender-quarantine }

## ما الذي يحتويه الحجر { #what-the-quarantine-contains }

يحتفظ حجر Microsoft Defender ببيانات وصفية عن عملية الاكتشاف وبالمحتوى المشفر للموارد التي حجرها.
يربط Juntei سجلات Entries وResourceData وResources ليُبلغ عن تسمية التهديد، ووقت الحجر، والمسارات
المنطقية والفعلية الأصلية، والبيانات الوصفية للملف، والمالك، و`Zone.Identifier` الخاص بمصدر التنزيل،
والتجزئات، ومشكلات السلامة.

يمكن لهذا الأثر الرقمي أن يبقى بعد زوال سجلات الأحداث التي سجّلت الاكتشاف/المعالجة في الأصل. كما يمكنه
الاحتفاظ بالبايتات المريبة نفسها تمامًا لأغراض الهندسة العكسية في بيئة معزولة.

<span class="evidence-strength">أفضل استنتاج:</span> أنشأ Defender سجل الحجر للتهديد/المورد المسمّى في
الوقت المسجل. وتسمية المنتج ليست دليلًا مستقلًا على وجود برمجية خبيثة، ولا يثبت الحجر وحده حدوث تنفيذ
سابق.

## الموقع والجمع { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Quarantine\
  Entries\{GUID}
  ResourceData\<prefix>\<SHA-1>
  Resources\<prefix>\<SHA-1>
```

اجمع مجلد `Quarantine` بأكمله. فالإدخال الذي لا تقابله ملفات محتوى/سجلات يظل بيانات وصفية مفيدة، لكنه
لا يمكن أن يعطي بايتات الملف أو التجزئات المحسوبة.

## السلامة وتحذيرات للمحلل { #safety-and-analyst-cautions }

- من المرجح أن يكون المحتوى المستخرج خبيثًا. استخدم نظام تحليل معزولًا ولا تنفّذه أبدًا.
- `ThreatName` هو توقيع/تصنيف Defender في ذلك الوقت. والنتائج الإيجابية الكاذبة وإعادة التصنيف لاحقًا
  ممكنة.
- وقت الحجر هو وقت المعالجة/الحجر—وليس بالضرورة وقت التنزيل أو الإنشاء أو التنفيذ.
- قد يكون الملف المحجور قد اكتُشف قبل التنفيذ أو بعد تشغيله.
- المسار الأصلي وZone.Identifier قيّمان، لكنهما قد يكونان غائبين أو خاضعين لتحكم المهاجم.
- قد تنتج السجلات اليتيمة (orphan) عن الاحتفاظ/التنظيف، أو عن جمع غير مكتمل، أو عن التلف.
- يتحقق Juntei من الأحجام وقيم CRC والمعرّفات والتجزئات؛ راجع `Problems` قبل الوثوق بالبايتات المستردة.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find quarantine folders
  -f, --file <FILE>      Parse one Entries\{GUID} file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
      --extract <DIR>       Extract dangerous content with a .quarantined extension
  -C, --clobber             Overwrite confirmed output targets
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -t jsonl \
  | jq '{QuarantineTime, ThreatName, Path, SHA256: .Content.SHA256, Problems}'

# Only in an isolated malware-analysis environment
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract extracted/
```

على سبيل المثال، يصبح `tool.exe` هو `tool.exe.quarantined`. وتتضمن الدفقات المساعدة نوعها في الاسم، مثل
`tool.exe.Zone.Identifier.quarantined`. كما يزيل Juntei إذن التنفيذ.
ويقلل الامتداد والأذونات من التنفيذ العرضي، لكن يجب مع ذلك التعامل مع البايتات المستردة على أنها
برمجيات خبيثة.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Record` | `resource`، أو إدخال بلا موارد، أو `orphan-content`، أو `orphan-record`. وتكشف السجلات اليتيمة عن ربط/جمع غير مكتمل. |
| `QuarantineTime` | وقت الحجر/المعالجة في Defender بتوقيت UTC، وليس وقت التنفيذ. |
| `ThreatName`، `ThreatID` | تصنيف Defender ومعرّفه الرقمي. تعامل معهما على أنهما حكم/سياق من المورّد. |
| `ResourceType` | ملف، أو مفتاح سجل Windows، أو خدمة، أو عنصر بدء تشغيل، أو مهمة مجدولة، أو فئة موارد أخرى. |
| `Path`، `PhysicalPath` | الموقع المنطقي/الفعلي الأصلي. النقطة المحورية الرئيسية للربط؛ احتفظ بالكتابة الدقيقة. |
| `FileSize` | الحجم الذي سجّلته البيانات الوصفية للمورد. قارنه بـ `ContentSize` وMFT. |
| `CreationTime`، `LastAccessTime`، `LastWriteTime` | البيانات الوصفية الملتقطة للمورد المحجور؛ منفصلة عن وقت الحجر وعرضة للتلاعب بالطوابع الزمنية (timestomping). |
| `Attr` | سمات الملف المسجلة. قد تكون حالة الإخفاء/النظام سياقًا مفيدًا. |
| `Owner` | معرّف SID من واصف الأمان المسترد؛ والملكية لا تعني بالضرورة المستخدم المنفِّذ. |
| `ZoneID`، `ReferrerURL`، `HostURL` | قيم منطقة التنزيل وعناوين URL من `Zone.Identifier`. خيوط قوية إلى المصدر عند وجودها؛ وقد تكون ADS غائبة/مُزالة. |
| `ContentSize` | الحجم الفعلي للمحتوى الأساسي المسترد. قارنه بـ `FileSize` المسجل. |
| `MD5`، `SHA1`، `SHA256` | محسوبة من البايتات المستردة. استخدم SHA-256 لتعريف المحتوى الحديث؛ واحتفظ بها كلها لقابلية التشغيل البيني. |
| `Streams` | أنواع الدفقات المستردة/أسماؤها/أحجامها؛ وقد تشمل ADS، أو بيانات أمان، أو خلية سجل Windows. |
| `ResourceID` | معرّف يربط سجلات البيانات الوصفية/المحتوى للمورد. |
| `EntryID` | معرّف GUID لإدخال الحجر؛ ويرتبط أيضًا بأدلة سجل/أحداث Defender. |
| `ScanID` | معرّف فحص Defender لتجميع الموارد/الإجراءات. |
| `QuarantineFolder` | المجلد المصدر الجذري/التمييز بين المضيفات. |
| `EntryFile`، `ContentFile`، `ResourceRecordFile` | ملفات المصدر الدقيقة المستخدمة لبناء الصف. ضرورية للتحقق. |
| `ExtractedFile` | الملف المحجور الأساسي الذي كتبه `--extract`؛ ويكون فارغًا عندما لا يحدث استخراج أساسي. وتُكتب الدفقات المساعدة بجانبه. ويعطيه `auto-parse -x` انطلاقًا من مجلد الإخراج الخاص به (`./HOST1/SecurityProducts/DefenderQuarantined/…`). |
| `Problems` | عدم تطابق السلامة، أو محتوى مفقود، أو تعارض في المعرّفات، أو تلف، أو سياق السجلات اليتيمة. لا تُخفِها. |

تحتفظ JSON/JSONL بالمورد المنظّم، وقائمة الدفقات، وإدخالات السجلات، وكل تفاصيل العلاقات بدلًا من النص
المدمج في CSV.

## سلوك الاستخراج { #extraction-behavior }

يستخدم Juntei اسم الملف الأصلي مع امتداد `.quarantined` في النهاية عندما يسجّل إدخال الحجر اسمًا. ويستخدم
معرّف المحتوى للموارد اليتيمة أو الموارد التي ليست ملفات، ويضيف المعرّف لحل تعارضات أسماء الملفات.
وتُستبدل الأحرف غير الصالحة في أسماء الملفات على Windows. وتُكتب دفقات الأمان والدفقات البديلة بجانب
الملف الأساسي. كما يحصل كل دفق أمان على تقرير نصي مقروء. ويزيل Juntei إذن التنفيذ؛
ولا يُكتب فوق محتوى مختلف موجود دون `-C`. تقلل هذه الضوابط من الحوادث، لكنها لا تجعل البايتات آمنة.

### قراءة واصف الأمان المسترد { #reading-the-recovered-security-descriptor }

يحتوي ملف `.security.quarantined` على بيانات أمان ثنائية خاصة بـ Windows، لذا لا يمكن قراءته في محرر
نصوص. وهو دفق نسخ احتياطي منفصل، وليس نسخة أخرى من الملف التنفيذي. يفك Juntei ترميزه تلقائيًا عند
الاستخراج، سواء باستخدام `parse-file defender-quarantine --extract` أو `auto-parse -x`:

```text
tool.exe.quarantined              Original file content; treat as malware
tool.exe.security.quarantined     Original binary security descriptor, preserved unchanged
tool.exe.security_descriptor.txt  Readable ownership and permission report
```

يتبع اسم الملف النصي الاسم الفعلي للملف الثنائي، بما في ذلك أي معرّف لحل التعارض. والنص المُنشأ بترميز
UTF-8 عادي ويستخدم حمايات الكتابة فوق الملفات نفسها المستخدمة مع الملفات المستخرجة الأخرى. ويتضمن اسم
الملف الثنائي وعدد البايتات وتجزئة SHA-256 حتى يمكن مطابقة التقرير بمصدره.

| حقل التقرير | كيفية تفسيره |
|---|---|
| المالك والمجموعة الأساسية | معرّفات SID المسجلة في الواصف. تحصل معرّفات SID المعروفة على أسماء مثل `BUILTIN\Administrators` أو `NT AUTHORITY\SYSTEM`. وتبقى معرّفات SID الأخرى دون حل؛ فمعرّف RID المألوف وحده لا يكفي لتسمية حساب محلي/حساب مجال. ولا تثبت الملكية من أنشأ الملف أو شغّله. |
| علامات التحكم | العلامات الرقمية الأصلية وأسماؤها، بما في ذلك وجود ACL والوراثة والحماية. |
| DACL | إدخالات التحكم في الوصول (ACE) مرتبةً: نوع السماح/الرفض، ومعرّف SID للحساب، وقناع الأذونات، والحقوق، وعلامات الوراثة. وهذه إدخالات مسجلة، وليست حسابًا للوصول الفعلي لرمز (token) مستخدم ما. |
| SACL | إدخالات التدقيق أو السلامة الإلزامية المسجلة. تصف علامات نجاح/فشل التدقيق التكوين، لا سجلًا للوصول. ولا يثبت غياب SACL أن كل التدقيق كان معطّلًا. |
| معرّفات GUID للكائنات | معرّفات GUID اختيارية لإدخالات ACE الخاصة بالكائنات، مع الاحتفاظ بالحقوق الخاصة بالكائنات رقميًا بدلًا من تسميتها خطأً حقوقًا للملفات. |
| SDDL | تمثيل المالك والمجموعة وإدخالات ACL المدعومة بلغة تعريف واصف الأمان (Security Descriptor Definition Language). راجع علامات التحكم بشكل منفصل. ولا يُصدر Juntei سلسلة SDDL جزئية إذا كان أي مكون تالفًا أو غير مدعوم. |
| حالة فك الترميز والمشكلات | `complete` أو `incomplete`، مع الأسباب. وتحتفظ إدخالات ACE غير المدعومة بصيغتها الست عشرية الخام في التقرير. ويُحتفظ دائمًا بالملف الثنائي الأصلي عند نجاح الاستخراج. وتظهر مشكلات فك الترميز أيضًا في الحقل `Problems` لسجل الحجر. |

يُبلَّغ عن DACL الغائب وDACL الفارغ (NULL DACL) وDACL الخالي من الإدخالات بشكل منفصل. فـ DACL الغائب أو
NULL DACL لا يوفر أي تقييد تقديري للوصول؛ أما DACL الخالي من الإدخالات فلا يحتوي على أي ACE يمنح الوصول.
وتظل فحوص الوصول والامتيازات الأخرى في Windows مهمة. وتبقى إدخالات ACE الموروثة وذات الوراثة فقط
(inherit-only) معلَّمة صراحةً.

على سبيل المثال، يمكن أن يظهر إدخال يمنح Administrators تحكمًا كاملًا موروثًا على النحو التالي:

```text
Entry 1: ALLOW (type 0x00)
  Flags: 0x10 (INHERITED_ACE)
  Account: S-1-5-32-544 (BUILTIN\Administrators)
  Access mask: 0x001F01FF
  Rights: Full control (FILE_ALL_ACCESS)
```

يعمل فك الترميز دون اتصال على Windows وmacOS وLinux دون واجهات Windows البرمجية أو Python أو البحث عن
الحسابات. ويفك ترميز إدخالات ACE الأساسية من أنواع السماح/الرفض/التدقيق/الإنذار، وإدخالات ACE الخاصة
بالكائنات، وتسميات السلامة الإلزامية. أما إدخالات ACE الشرطية، وإدخالات callback، وسمات الموارد، وغيرها
من الإدخالات غير المدعومة فتُحدَّد صراحةً ويُحتفظ بها في صيغتها الخام؛ ولا تُقيَّم شروط الوصول الخاصة
بها. ولا يوقف الواصف التالف استخراج الدفقات الأخرى.

## استراتيجية التحقيق { #investigation-strategy }

1. ابدأ من `ThreatName` و`Path` و`QuarantineTime` و`EntryId` والتجزئات.
2. تحقق من `Problems` وقارن الحجم المسجل بالمحتوى المسترد.
3. انطلق من SHA-256 إلى المضيفات الأخرى، وEDR، وبيئات الاختبار المعزولة (sandbox)، وبيانات استخبارات
   التهديدات.
4. اربط المسار/الوقت الأصليين بـ `$MFT` ودفتر يومية USN وPrefetch وAmcache وسجلات الأحداث.
5. استخدم حقول URL/المنطقة للتحقيق في الوصول الأولي، لكن تحقق منها بأدلة المتصفح/الوكيل (proxy)/DNS.

## مزيد من التفاصيل { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [defender من Sootmark](https://github.com/Sootmark/defender)
- [إضافة Defender في dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [Microsoft: المعالجة والحجر في Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [defender-dump](https://github.com/knez/defender-dump)
- [Microsoft: واصفات الأمان](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft: تنسيق سلسلة واصف الأمان](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: سلاسل ACE](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings)
