# مربعات حوار الفتح والحفظ باسم (OpenSaveMRU) { #open-and-save-as-dialogs-opensavemru }

## ما هو سجل مربعات الحوار { #what-the-dialog-history-is }

عندما يعرض برنامج مربع الحوار الشائع "فتح" أو "حفظ باسم"، يسجّل Windows في
`NTUSER.DAT` الخاص بالمستخدم:

- **الملف الذي فُتح أو حُفظ:** يحتوي `OpenSavePidlMRU` على مفتاح فرعي لكل امتداد (`docx`،
  `ps1`، …) و`*` لكل الامتدادات، وكل منها قائمة للعناصر المستخدمة مؤخرًا (MRU)؛
- **البرنامج الذي استخدم مربع الحوار، والمجلد الذي كان فيه:** ‏`LastVisitedPidlMRU` (و
  `LastVisitedPidlMRULegacy` للبرامج التي تستخدم مربعات الحوار الأقدم)؛
- **البرامج التي عرضت مربع الحوار:** ‏`CIDSizeMRU` (حجم مربع الحوار لكل برنامج)، و`FirstFolder`
  (مسار البرنامج ومجلد)؛
- على Windows 2000 وXP، ‏`OpenSaveMRU` (مسارات لكل امتداد) و`LastVisitedMRU` (البرامج و
  مجلداتها على هيئة مسارات).

منذ Windows Vista، تكون الملفات والمجلدات قوائم عناصر shell، وهي البنى نفسها المستخدمة في هدف
الاختصار: كل عنصر جزء من المسار (This PC، أو محرك أقراص، أو مجلد، أو الملف)، و
يمكن لعنصر الملف أو المجلد أن يحمل أوقاته الخاصة ومرجع ملف NTFS.

<span class="evidence-strength">أفضل استنتاج:</span> استخدم برنامجٌ مربع حوار "فتح" أو "حفظ باسم"
لفتح هذا الملف أو حفظه، أو في هذا المجلد، ضمن جلسة هذا المستخدم. ولا يُؤرَّخ إلا الإدخال
الأحدث في كل قائمة: وقت آخر كتابة لمفتاحه.

## الموقع { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

اجمع `NTUSER.DAT` مع سجلات المعاملات الخاصة به لكل ملف تعريف.

## تحذيرات للمحلل { #analyst-cautions }

- **وقت واحد لكل قائمة.** يتغير وقت آخر كتابة للمفتاح عندما ينتقل أحد إدخالاته إلى
  مقدمة قائمته، لذا فهو يؤرّخ الإدخال في الموضع 0 فقط (`IsMostRecent`، `LastUsedUTC`).
  أما الإدخالات الأخرى فمرتبة، من الأحدث إلى الأقدم نزولًا في القائمة، لكنها غير مؤرّخة.
- **الملف نفسه في قائمتين.** يوجد الملف في قائمة امتداده وفي `*`؛ فلا تحسبه
  استخدامين. ويعطي `*` الترتيب عبر الامتدادات.
- **فُتح أم حُفظ.** لا توضح القوائم أيهما. اربط ذلك بسجلات `$MFT` ودفتر يومية USN
  الخاصة بالملف، وملفات LNK وJump Lists.
- **أوقات العناصر هي أوقات الملف، لا مربع الحوار.** ‏`ItemModifiedUTC` و`ItemCreatedUTC` و
  `ItemAccessedUTC` هي أوقات الملف عند كتابة العنصر، بدقة ثانيتين (FAT). وكثيرًا ما لا تحتوي
  عناصر المجلدات المعروفة (`CLSID_ThisPCDocumentsRegFolder`، `Downloads`، …) على أي منها. ولا تتضمن هذه الطوابع
  الزمنية بصيغة FAT إزاحة للمنطقة الزمنية. واللاحقة `UTC` في اسم العمود اصطلاح عرض، لا
  دليل على أن الساعة الأصلية كانت بتوقيت UTC؛ فاربطها بالمنطقة الزمنية للنظام المصدر و
  الطوابع الزمنية الأخرى قبل مقارنتها بقيم FILETIME في سجل Windows. راجع
  [إرشادات Microsoft حول أوقات الملفات](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times).
- **المسارات مسارات shell.** تبدأ من سطح المكتب: `This PC\C:\Users\…`، أو مجلد معروف، أو
  مكتبة (`UsersLibraries\Documents\…`) أو موقع شبكة. والمسار الذي يبدأ بمجلد
  أو ملف موجود على سطح المكتب. والمسار الفارغ هو سطح المكتب نفسه.
- **عناصر لم يُفك ترميزها.** يُعرض العنصر من نوع لا يعرفه محلل عناصر shell بفئته
  (`[class 0x00]`)، ويكون نوعه `unknown`، وتوجد بايتات القائمة في `DataRaw`.
- **البرامج أسماء.** يحتفظ `LastVisitedPidlMRU` و`CIDSizeMRU` باسم ملف
  (`WINWORD.EXE`)، لا بمسار. ويحتفظ `FirstFolder` بمسار.
- **القوائم قصيرة وتُنظَّف.** تحتفظ كل قائمة بعدد محدود من الإدخالات، وتحذف أدوات التنظيف
  المفاتيح. استخدم `parse-registry read --recover-deleted` للقيم المحذوفة.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry open-save-mru -d evidence/ -o opensavemru.csv
juntei parse-registry open-save-mru -f NTUSER.DAT -t jsonl \
  | jq 'select(.List == "OpenSavePidlMRU" and .Extension != "*")'
```

يسرد الملخص الإدخالات لكل قائمة، والبرامج التي استخدمت مربعات الحوار، والملف الأحدث
في كل خلية (الإدخال الأول في `*`).

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `User`، `UserSID`، `SourcePath` | المضيف، ومالك ملف التعريف (من المسار أو مجلدات shell)، ومعرّف SID من `ProfileList` الخاص بالنظام، والخلية. |
| `List` | `OpenSavePidlMRU` أو `LastVisitedPidlMRU` أو `LastVisitedPidlMRULegacy` أو `CIDSizeMRU` أو `FirstFolder` أو `OpenSaveMRU` أو `LastVisitedMRU`. |
| `Extension` | المفتاح الفرعي للامتداد (`docx`، `*`)؛ فارغ للقوائم الأخرى. |
| `MRU-Position`، `IsMostRecent` | الموضع في قائمة المفتاح (0 هو الأحدث)؛ فارغ عندما لا تسمّي القائمة القيمة. |
| `ValueName` | رقم القيمة (`MRUListEx`) أو حرفها (`MRUList`). |
| `Program` | البرنامج الذي عرض مربع الحوار. |
| `Path` | الملف أو المجلد: أسماء عناصر shell مدمجة، أو المسار المخزن كنص. |
| `ItemType` | نوع آخر عنصر shell: ‏`file`، `directory`، `root_folder`، `volume`، `network_location`، `property_view`، `unknown`، … |
| `ItemModifiedUTC`، `ItemCreatedUTC`، `ItemAccessedUTC` | أوقات الملف أو المجلد، كما سجّلها عنصر shell الخاص به. |
| `MFT-Entry`، `MFT-Sequence` | مرجع ملف NTFS الذي سجّله عنصر shell. |
| `LastUsedUTC` | وقت آخر كتابة للمفتاح، للإدخال الأحدث فقط. |
| `KeyPath`، `KeyLastWrittenUTC` | المفتاح وآخر كتابة له: وقت إدخاله الأحدث، لا وقت كل صف. |
| `DataRaw` | بايتات القيمة (بالنظام السداسي عشري) عندما يتعذر فك ترميزها، أو قائمة عناصر shell عندما يكون نوع العنصر غير معروف. |
| `TransactionLogStatus`، `LogsApplied` | سياق استعادة سجل Windows. |
| `Problems` | قيم تسميها القائمة لكن المفتاح لا يحتويها، وقيم لا تسميها، وأنواع خاطئة، وأسماء أو قوائم مبتورة. |

## استراتيجية التحقيق { #investigation-strategy }

1. اقرأ `*` بالترتيب لكل مستخدم، و`LastUsedUTC` للملف الأحدث.
2. ابحث عن برامج نصية وأرشيفات وملفات تنفيذية ومستندات في أماكن قابلة للكتابة من المستخدم أو قابلة للإزالة أو
   على الشبكة، وعن ملفات تحمل أسماء تشبه بيانات الاعتماد أو عمليات التصدير.
3. استخدم `LastVisitedPidlMRU` لربط برنامج بالمجلد الذي استخدمه آخر مرة، و`CIDSizeMRU` و
   `FirstFolder` للبرامج التي استخدمت مربعات الحوار أصلًا.
4. اربط المسارات و`MftEntry` بـ `$MFT` ودفتر يومية USN وملفات LNK وJump Lists
   وShellBags وRecentDocs.

## مزيد من التفاصيل { #further-details }

- [libyal winreg-kb: مفاتيح العناصر المستخدمة مؤخرًا (MRU)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi: تنسيق Windows Shell Item](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
