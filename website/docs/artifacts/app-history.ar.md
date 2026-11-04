# سجل التطبيقات { #application-history }

## ما الذي تتركه الأدوات وراءها { #what-tools-leave-behind }

تحتفظ كثير من أدوات الجهات الخارجية بإعداداتها وسجلاتها في ملف `NTUSER.DAT` الخاص بالمستخدم. وبعضها
من الأدوات التي يستخدمها المهاجمون، أو يستخدمونها لتجهيز البيانات قبل الاستيلاء عليها: أدوات
Sysinternals، وبرامج الأرشفة، وعملاء المزامنة السحابية، وأدوات الوصول عن بُعد. يقرأ Juntei ما يلي:

| التطبيق | المفتاح تحت `HKCU\Software` | ما الذي يُظهره |
|---|---|---|
| Sysinternals | `Sysinternals\<tool>` | كل أداة قبل المستخدم ترخيصها (`EulaAccepted`): PsExec، وProcDump، وSysmon، … |
| WinRAR | `WinRAR\ArcHistory`، `DialogEditHistory\ArcName`، `DialogEditHistory\ExtrPath` | الأرشيفات التي فُتحت؛ وأسماء الأرشيفات ومجلدات الاستخراج المكتوبة في مربعات الحوار الخاصة به |
| 7-Zip | `7-Zip\Compression\ArcHistory`، `Extraction\PathHistory`، `FM\FolderHistory`، `FM\CopyHistory`، `FM\PanelPath0/1` | الأرشيفات التي أُنشئت؛ ومجلدات الاستخراج؛ والمجلدات التي تُصفِّحت ونُسخ إليها في مدير الملفات الخاص به |
| WinZip | `Nico Mak Computing\WinZip\filemenu`، `extract`، `mru\archives` | الأرشيفات الموجودة في قائمة File الخاصة به، ومجلدات الاستخراج، والأرشيفات الأخيرة |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | عنوان البريد الإلكتروني لكل حساب ومجلده المحلي وأسماؤه؛ والمكتبات المتزامنة (`ScopeIdToMountPointPathCache`) والمجلدات (`Tenants\<tenant>`) |
| Adobe Acrobat وReader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`، `cRecentFolders` | آخر ملفات PDF والمجلدات التي فُتحت، مع أسماء الملفات وأحجامها وتواريخها |
| TeamViewer | `TeamViewer` (و`Version<N>` الخاص بإصدار أقدم) | كل قيمة: أسماء الحسابات والمستخدمين، ومجلدات بدء نقل الملفات (`<partner ID>?<local>|<remote>`)، … |
| CCleaner | `Piriform\CCleaner` | كل قيمة: ما الذي ضُبط لتنظيفه، و`UpdateKey`، … |

<span class="evidence-strength">أفضل استنتاج:</span> شغّل هذا المستخدم الأداة (أو قبل ترخيصها)، أو
فتحت الأداة هذه الأرشيفات والمجلدات أو أنشأتها أو استخرجتها، أو أُعدّ هذا الحساب السحابي للمستخدم.
ولا يحمل معظم هذه السجلات وقتًا لكل إدخال: فالإدخال الأحدث وحده يحصل على وقت آخر كتابة لمفتاح القائمة.

## الموقع { #location }

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

اجمع ملف `NTUSER.DAT` لكل مستخدم مع سجلات المعاملات الخاصة به، وخلية `SOFTWARE` الخاصة بالنظام
للحصول على معرّفات SID للمستخدمين.

## تحذيرات للمحلل { #analyst-cautions }

- **قبول EULA لأدوات Sysinternals.** يمثّل `FirstUsedUTC` تقديرًا لأول استخدام مأخوذًا من وقت آخر
  كتابة للمفتاح عندما تكون قيمته الوحيدة `EulaAccepted` قابلة للقراءة ومضبوطة على `1`. ولا تحصل
  اتفاقية EULA المرفوضة أو التالفة على هذا التقدير. كما يمكن تكوين إعدادات سجل Windows مسبقًا أو
  نسخها أو تحريرها: أكّد التنفيذ ووقته بآثار رقمية أخرى.
  الأدوات التي تحفظ إعداداتها هناك (Process Explorer، وAutoruns، …) تعيد كتابة المفتاح لاحقًا:
  فيكون `FirstUsedUTC` حينئذٍ فارغًا، ولا يمثّل `KeyLastWrittenUTC` سوى آخر تغيير في الإعدادات. وأي
  أداة تُشغَّل باستخدام `-accepteula`، أو بصفة مستخدم آخر (يعمل الجانب البعيد من PsExec بصفة SYSTEM)،
  تكتب مفتاح ذلك المستخدم.
- **مواضع لا أوقات.** إدخالات القوائم مرتبة (`MruPosition` 0 هو الأحدث)، لكن آخر كتابة لمفتاح القائمة
  لا تؤرّخ إلا الإدخال الأحدث. أما الإدخالات الأقدم فقد استُخدمت قبل ذلك، في وقت غير معروف.
- **الأسماء المكتوبة.** `ArcName` في WinRAR هو اسم الأرشيف الذي كُتب عند إنشائه: وكثيرًا ما يكون اسم
  ملف دون مجلد.
- **مسارات Adobe** تُحفظ كما يكتبها Adobe (`/C/Users/alice/a.pdf` تعني
  `C:\Users\alice\a.pdf`). أما `sDate` الخاص به (في `Details`) فهو سلسلة تاريخ بصيغة PDF، تُحفظ كما كُتبت.
- **`UpdateKey` في CCleaner** هو وقت محلي دون منطقة زمنية؛ ويُحفظ كما كُتب.
- **TeamViewer.** سجلات الاتصال ملفات (`Connections_incoming.txt`، `TeamViewer*_Logfile.log`)، وليست
  قيمًا في سجل Windows: اجمعها أيضًا.
- **الغياب.** يمكن للمستخدمين والأدوات مسح هذه السجلات (ويستطيع CCleaner نفسه ذلك).

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.App == "Sysinternals" and .FirstUsed != null)'
```

يسرد الملخص أحدث تقديرات أول استخدام لأدوات Sysinternals، والأرشيفات والمجلدات التي استُخدمت أخيرًا
في كل قائمة، وحسابات OneDrive.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `SourcePath` | المضيف و`NTUSER.DAT`. |
| `User`، `UserSID` | اسم مجلد ملف التعريف، ومعرّف SID الخاص به من `ProfileList` في النظام. |
| `App` | `Sysinternals`، `WinRAR`، `7-Zip`، `WinZip`، `OneDrive`، `Adobe Acrobat Reader`، `Adobe Acrobat`، `TeamViewer`، `CCleaner`. |
| `Item` | أداة Sysinternals، أو القائمة (`ArcHistory`، `ExtrPath`، `FolderHistory`، `filemenu`، `cRecentFiles`، …)، أو حساب OneDrive أو مفتاح مجلداته المتزامنة، أو اسم القيمة. |
| `MRU-Position` | موضع الإدخال في قائمته، و0 هو الأحدث. |
| `Path` | الأرشيف أو المجلد أو المستند. |
| `Value` | `EulaAccepted`، أو عنوان البريد الإلكتروني لحساب OneDrive أو معرّف مكتبة متزامنة، أو قيمة من TeamViewer أو CCleaner. |
| `FirstUsedUTC` | تقدير أول استخدام من قبول EULA (انظر التحذيرات أعلاه). |
| `Details` | `DisplayName` و`UserCID` و`ConfiguredTenantId` … لحساب OneDrive؛ و`Version` و`tFileName` و`uFileSize` و`uPageCount` و`sDate` لمستند Adobe. |
| `KeyPath`، `KeyLastWrittenUTC`، `ValueName` | مصدر السجل، ووقت آخر تغيير لذلك المفتاح. |
| `DataRaw` | بايتات القيمة (بالنظام الست عشري)، عندما يتعذر فك ترميزها. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows. |
| `Problems` | بيانات تعذّر فك ترميزها أو قراءتها. |

## استراتيجية التحقيق { #investigation-strategy }

1. ابحث عن أدوات Sysinternals التي لا يشغّلها المستخدم العادي (PsExec، وProcDump، وPsLoggedOn،
   وSDelete) وعن الطوابع الزمنية لقبول EULA الخاصة بها.
2. راجع أسماء الأرشيفات ومجلدات الاستخراج في الفترة المحيطة بالحادث: الأرشيفات التي أُنشئت في
   `ProgramData` أو `Temp` أو مجلد أحد المستخدمين قبل خروج البيانات من النظام.
3. افحص حسابات OneDrive التي لا تخص المؤسسة، وأسماء حسابات TeamViewer ومجلدات نقل الملفات.
4. اربط النتائج مع Prefetch وAmcache وBAM وUserAssist (تشغيل الأداة)، و`$MFT` ودفتر يومية USN (ملف
   الأرشيف)، وملفات LNK وJump Lists (المستندات المفتوحة)، وسجلات الأحداث.

## مزيد من التفاصيل { #further-details }

- [Microsoft: Sysinternals](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
