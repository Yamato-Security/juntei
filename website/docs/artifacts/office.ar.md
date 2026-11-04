# Microsoft Office (الملفات الأخيرة، والمستندات الموثوقة، وإعدادات وحدات الماكرو) { #microsoft-office-recent-files-trusted-documents-macro-settings }

## ما يسجله Office { #what-office-records }

يحتفظ Microsoft Office، لكل مستخدم في `NTUSER.DAT` ولكل إصدار وتطبيق، بما يلي:

- **الملفات والأماكن الأخيرة:** ‏`File MRU`، وهي المستندات التي فتحها التطبيق، و`Place MRU`،
  وهي المجلدات التي فتحها منها، ولكل عنصر وقت آخر فتح له. ويحتفظ Microsoft 365
  أيضًا بقائمة لكل حساب مسجّل الدخول (`User MRU\<account>`).
- **المستندات الموثوقة (Trusted Documents):** عندما ينقر المستخدم على *Enable Editing* أو *Enable Content* في مستند،
  يتذكره Office في `TrustRecords`: مسار المستند، ووقت الوثوق به، وما إذا
  كانت وحدات الماكرو الخاصة به قد مُكّنت.
- **إعدادات وحدات الماكرو:** ‏`VBAWarnings` (أي وحدات الماكرو تعمل)، و`BlockContentExecutionFromInternet`
  (ما إذا كانت وحدات الماكرو في الملفات القادمة من الإنترنت محظورة)، و`AccessVBOM` (ما إذا كان يُسمح للبرامج
  بتغيير مشاريع VBA) و**المواقع الموثوقة (Trusted Locations)**، وهي مجلدات تعمل وحدات الماكرو في مستنداتها دون
  سؤال. ويمكن لنهج المجموعة (Group Policy) ضبط الإعدادات نفسها تحت `Software\Policies`.

عادةً ما يحتاج المستند الضار إلى أن يمكّن المستخدم وحدات الماكرو الخاصة به: فسجل الثقة الذي مُكّنت فيه
وحدات الماكرو يربط المستند والمستخدم ووقتًا معًا.

<span class="evidence-strength">أفضل استنتاج:</span> فتح المستخدم هذا المستند في هذا
التطبيق (وآخر مرة في هذا الوقت)؛ ومكّن المستخدم التحرير أو وحدات الماكرو لهذا المستند في
هذا الوقت؛ وكان Office مُكوَّنًا لتشغيل وحدات الماكرو بهذه الطريقة.

## الموقع { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

الإصدارات: `16.0` هو Office 2016 وما بعده (2019، 2021، 2024، Microsoft 365)، و`15.0` هو Office 2013،
و`14.0` هو Office 2010، و`12.0` هو Office 2007.

يبدو العنصر هكذا `[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx`: ‏`T`
هو وقت آخر فتح (FILETIME بالنظام السداسي عشري)، ويأتي المسار بعد `*`. وسجل الثقة 24
بايتًا: قيمة FILETIME لوقت الوثوق بالمستند، ثم أعلامه في آخر 4 بايتات، `0x7FFFFFFF`
عندما مُكّنت وحدات الماكرو الخاصة به و`0x00000001` عندما مُكّن التحرير فقط.

## تحذيرات للمحلل { #analyst-cautions }

- **الأوقات.** `LastOpenedUTC` هو الوقت الخاص بكل عنصر. و`TrustedAtUTC` هو وقت وثوق المستخدم
  بالمستند، أي لحظة النقر على *Enable Editing* أو *Enable Content*، والتي قد تسبق
  آخر فتح.
- **تمكين وحدات الماكرو لا يعني تشغيلها.** يوضح `MacrosEnabled` أن المستخدم مكّن وحدات الماكرو في المستند؛ لكنه
  لا يثبت أن وحدة ماكرو قد عملت أو ماذا فعلت. اربط ذلك بإنشاء العمليات، وPrefetch، وسجلات
  PowerShell والمستند نفسه.
- **المسارات كما يخزنها Office.** تسمّي سجلات الثقة المستند بمسار قد يستخدم
  `%USERPROFILE%`، أو شرطات مائلة للأمام، أو عنوان URL من نوع `file:///` وترميز URL ‏(`%20`). وعناصر MRU
  للمستندات السحابية هي عناوين URL.
- **القيم الافتراضية والنهج الفعلي.** ينشئ Office مواقع موثوقة افتراضية (`Templates`،
  `XLSTART`، `STARTUP`، …). و`Date` الخاص بالموقع دليل مفيد، لا إثبات لمن أنشأه:
  فقد يملؤه أيضًا المسؤولون، والبرامج النصية، والنُّهج، وإعدادات سجل Windows المنسوخة. ولا يُثبت غياب
  `VBAWarnings` وحده نهج وحدات الماكرو الفعلي. كما أن إصدار Office، ووسم ملفات
  الإنترنت، وإعدادات الإدارة الأخرى مهمة أيضًا.
  [توضح Microsoft حظر وحدات الماكرو القادمة من الإنترنت](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked).
- **المستندات المحذوفة.** تبقى القوائم بعد زوال المستندات؛ فقد يظل ملف مفقود ظاهرًا
  هنا. والقوائم قصيرة وتُمسح بواسطة *Clear unpinned items* في Office وأدوات التنظيف.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

يسرد الملخص المستندات التي مُكّنت وحدات الماكرو الخاصة بها، والإعدادات التي تسمح بتشغيل وحدات الماكرو بحرية
أكبر (`VBAWarnings` ‏1، `BlockContentExecutionFromInternet` ‏0، `AccessVBOM` ‏1،
`AllowNetworkLocations` ‏1)، والمواقع الموثوقة التي أضافها المستخدمون أو ضبطها نهج، والمستندات
التي فُتحت مؤخرًا.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `User`، `UserSID`، `SourcePath` | المضيف، ومالك ملف التعريف (من المسار أو مجلدات shell)، ومعرّف SID من `ProfileList` الخاص بالنظام، والخلية. |
| `OfficeVersion`، `App` | مفتاح الإصدار (`16.0`) والتطبيق (`Word`، `Excel`، …). |
| `List` | `File MRU` أو `Place MRU` أو `TrustRecords` أو `Security` أو `Trusted Locations`. |
| `Account` | حساب Microsoft 365 لقائمة `User MRU`. |
| `Policy` | ما إذا كان الإعداد أو الموقع مضبوطًا بواسطة نهج. |
| `MRU-Position`، `IsMostRecent` | موضع العنصر في قائمته (`Item 1` هو 0، الأحدث). |
| `Path` | المستند أو المجلد؛ ومسار المستند الموثوق كما هو مخزن؛ و`Path` الخاص بالموقع الموثوق. |
| `LastOpenedUTC` | وقت آخر فتح للعنصر. |
| `TrustedAtUTC`، `MacrosEnabled`، `TrustFlags` | وقت الوثوق بالمستند، وما إذا كانت وحدات الماكرو الخاصة به قد مُكّنت، وأعلام السجل. وتترك الأعلام غير المعروفة `MacrosEnabled` فارغًا وتُنتج رسالة تشخيصية. |
| `Setting`، `Value` | إعداد ماكرو وقيمته مع معناها (`1 (all macros enabled)`)؛ ومفتاح الموقع الموثوق (`Location3`) وقيمه الأخرى (`AllowSubFolders=1; Date=…; Description=…`). |
| `KeyPath`، `KeyLastWrittenUTC` | المفتاح وآخر كتابة له. |
| `DataRaw` | بايتات القيمة (بالنظام السداسي عشري) عندما يتعذر فك ترميزها. |
| `TransactionLogStatus`، `LogsApplied` | سياق استعادة سجل Windows. |
| `Problems` | عناصر بلا `*` أو بوقت غير صالح، وسجلات ثقة ليست 24 بايتًا، ومواقع بلا `Path`، وأنواع خاطئة. |

## استراتيجية التحقيق { #investigation-strategy }

1. ابدأ بقيمة `MacrosEnabled` الصحيحة: المستندات، ومتى، ومن قِبل من. ابحث عن الملفات (`$MFT`،
   ودفتر يومية USN، و`Zone.Identifier`)، ومصدرها (سجل المتصفح والبريد) وما عمل بعدها.
2. افحص إعدادات وحدات الماكرو والمواقع الموثوقة: فتمكين كل وحدات الماكرو، أو عدم حظر وحدات الماكرو القادمة من
   الإنترنت، أو الوصول إلى مشاريع VBA، أو موقع أضافه المستخدم (في `Downloads`، `Temp`، مشاركة شبكة)
   كلها تُضعف الإعدادات الافتراضية.
3. ضع `LastOpenedUTC` على الجدول الزمني مع ملفات LNK وJump Lists وRecentDocs لمسارات
   المستندات، بما في ذلك المواقع القابلة للإزالة ومواقع الشبكة.

## مزيد من التفاصيل { #further-details }

- [Microsoft: تمكين وحدات الماكرو أو تعطيلها في ملفات Microsoft 365](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6)
- [Microsoft: المستندات الموثوقة](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft: إضافة موقع موثوق أو إزالته أو تغييره](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
