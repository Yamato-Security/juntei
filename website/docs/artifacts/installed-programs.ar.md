# البرامج المثبتة { #installed-programs }

## ما الذي يحتفظ به Windows { #what-windows-keeps }

يسجّل Windows والمثبّتات البرمجيات المثبتة في عدة قوائم في سجل Windows. لكل قائمة مفتاح واحد لكل برنامج
أو منتج أو توزيعة أو حزمة؛ ويعطي Juntei سجلًا واحدًا لكل مفتاح.

| المصدر | الخلية والمفتاح | ما الذي تحتويه |
|---|---|---|
| `uninstall` | `SOFTWARE` و`NTUSER.DAT` `…\CurrentVersion\Uninstall\*`، و`SOFTWARE\Wow6432Node\…` | ما تسرده Settings و*Programs and Features*: الاسم، والإصدار، والناشر، وتاريخ التثبيت، والمجلدات، وأوامر إلغاء التثبيت، والحجم، وما إذا كان مخفيًا |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`، `NTUSER.DAT` `Software\Microsoft\Installer\Products\*` | منتجات Windows Installer: اسم المنتج وإصداره، واسم ملف الحزمة، والمجلد الذي ثُبّت منه |
| `app_path` | `SOFTWARE` و`NTUSER.DAT` `…\CurrentVersion\App Paths\*` | البرامج التي يبدؤها Windows بالاسم (`Win+R`، `start tool`): مسار البرنامج ومجلده |
| `wsl` | `NTUSER.DAT` `…\CurrentVersion\Lxss\{GUID}` | توزيعات Windows Subsystem for Linux: الاسم، ومكان ملفاتها أو قرصها، وWSL 1 أو 2 |
| `package` | `SOFTWARE` `…\Appx\AppxAllUserStore` | التطبيقات المحزّمة (Store): المجهّزة مسبقًا (provisioned) لكل المستخدمين، وتطبيقات Windows نفسه، والمرحلية (staged)، والملغى تجهيزها (deprovisioned)، وتطبيقات كل مستخدم |

<span class="evidence-strength">أفضل استنتاج:</span> ثُبّت هذا البرنامج أو المنتج أو الحزمة (أو سُجّل)
على النظام، لكل المستخدمين أو لهذا المستخدم. وتاريخ التثبيت هو ما كتبه المثبّت؛ أما آخر كتابة للمفتاح
فهي آخر وقت تغيّر فيه الإدخال.

## الموقع { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

اجمع `SOFTWARE`، وكل ملفات `NTUSER.DAT`، وسجلات المعاملات الخاصة بها. تسمّي `SYSTEM` المضيف، ويعطي
`ProfileList` في `SOFTWARE` معرّف SID لكل مستخدم.

## تحذيرات للمحلل { #analyst-cautions }

- **تاريخ التثبيت ووقت المفتاح.** ‏`InstallDate` تاريخ (دون وقت) يكتبه المثبّت، بالتوقيت المحلي؛ وبعض
  المثبّتات لا تكتبه، وبعضها يكتبه بتنسيق آخر (يُحتفظ به كما هو مكتوب في `InstallDateRaw`). ويمكن
  لتحديثٍ أن يعيد كتابة الإدخال دون تغيير التاريخ، أو أن يغيّر التاريخ دون أن يتغير البرنامج كثيرًا:
  قارن `InstallDate` بـ `KeyLastWrittenUTC`.
- **البرمجيات التي أُلغي تثبيتها.** يفقد البرنامج الذي أُزيل بشكل نظيف مفاتيحه. استخدم `parse-registry read
  --recover-deleted` لاسترداد مفاتيح `Uninstall` المحذوفة، وAmcache وPrefetch و`$MFT` للآثار المتبقية.
- **التسجيلات الخاصة بكل مستخدم.** تحدد إدخالات `NTUSER.DAT` البرمجيات المسجلة لملف تعريف ما. ولا يثبت
  هذا النطاق من شغّل المثبّت أو ما إذا كانت صلاحيات المسؤول قد استُخدمت.
- **الإدخالات المخفية.** يخفي `SystemComponent` الإدخال من *Programs and Features*؛ كما لا تُعرض
  الإدخالات التي لا تحتوي على `DisplayName` (التحديثات، والمكونات). ويُبلَّغ عنها جميعًا.
- **رموز المنتجات.** يمكن مطابقة مفتاح `Uninstall` على شكل GUID بمفتاح `msi_product` المضغوط (packed)
  عبر `ProductCode`. ولا يثبت الاسم الذي على شكل GUID وحده أن Windows Installer أنشأ الإدخال.
- **App Paths.** هذه تسجيلات تُستخدم لتحديد مواقع البرامج، وليست سجلات تنفيذ. ويمكن أن تحمل القيمة `Path`
  عدة مجلدات بحث، وليست بالضرورة مجلد التثبيت.
- **الناشر.** هذه بيانات وصفية في سجل Windows يقدّمها المثبّت، وليست معلومات توقيع جرى التحقق منها.
- **البنية المعمارية.** يحتوي `Wow6432Node` على برامج 32 بت على Windows بنظام 64 بت (`Architecture` `x86`،
  `Wow64` `True`)؛ أما الإدخالات الأخرى فهي بالبنية المعمارية للنظام نفسه (فارغة). وبنية الحزمة المعمارية
  هي بنيتها الخاصة (`x64`، `neutral`، …).
- **WSL.** ‏`Version` هو إصدار WSL المأخوذ من `Flags` الخاصة بالتوزيعة (`0x8`: تعمل في الجهاز الافتراضي
  لـ WSL 2). أما القيمة `Version` الخاصة بالتوزيعة نفسها (في `Details`) فهي إصدار نظام ملفاتها، وتكون 2
  في توزيعات WSL 1 أيضًا. ويحتوي `BasePath` على `ext4.vhdx` الخاص بها (WSL 2) أو `rootfs`
  (WSL 1): اجمعه للاطلاع على محتواه.
- **الحزم.** قوائم `<SID>` هي الحزم المسجلة لكل مستخدم؛ ومعظمها خاص بـ Windows نفسه. وحزم
  `Deprovisioned` أُزيلت للمستخدمين الجدد، وحزم `EndOfLife\<SID>` أُزيلت لذلك المستخدم.
- **القيم التالفة.** يُبلَّغ عن الأرقام المشوّهة، وأنواع القيم الخاطئة، وUTF-16 غير الصالح في
  `Problems` ويُحتفظ بها في `DataRaw`، بما في ذلك سلسلة ست عشرية فارغة لقيمة موجودة بطول
  صفر بايت. وتُعامل السلسلة الفارغة النظيفة في حقل رقمي اختياري على أنها غائبة. ويجب أن يحتوي إصدار
  الحزمة على أربعة مكونات عشرية، كل منها ضمن نطاق الأعداد الصحيحة غير الموقعة بطول 16 بت.
  وتبقى التحذيرات على مستوى الخلية في stderr عندما يبث `-t` السجلات إلى stdout.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.Scope == "user" and .Source != "package")'
```

يسرد الملخص آخر البرامج المثبتة (بحسب `InstallDate`)، وما ثبّته المستخدمون لأنفسهم، وتوزيعات WSL.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `SourcePath`، `HiveType` | المضيف، وملف الخلية، و`SOFTWARE` أو `NTUSER.DAT`. |
| `User`، `UserSID` | مستخدم خلية المستخدم (مجلد ملف تعريفه ومعرّف SID من `ProfileList`)، أو المستخدم الذي أُدرجت له الحزمة. |
| `Source`، `Scope` | القائمة (`uninstall`، `msi_product`، `app_path`، `wsl`، `package`)، و`machine` أو `user`. |
| `Name`، `Version`، `Publisher` | `DisplayName`، `DisplayVersion`، `Publisher`؛ و`ProductName` و`Version` للمنتج؛ واسم ملف البرنامج لـ App Path؛ واسم التوزيعة وإصدار WSL؛ واسم الحزمة وإصدارها. |
| `InstallDate`، `InstallDateRaw` | `InstallDate` بصيغة تاريخ عندما يُكتب `YYYYMMDD`؛ وإلا فكما هو مكتوب. |
| `KeyLastWrittenUTC` | آخر وقت تغيّر فيه مفتاح الإدخال. |
| `InstallLocation`، `InstallSource` | `InstallLocation` (‏`Path` لـ App Path، و`BasePath` للتوزيعة، و`Path` للحزمة)؛ و`InstallSource` (آخر مجلد مصدر استُخدم للمنتج). |
| `Program` | برنامج App Path. |
| `UninstallCommand`، `QuietUninstallCommand` | `UninstallString`، `QuietUninstallString`. |
| `SizeKB`، `SystemComponent`، `WindowsInstaller` | `EstimatedSize` بوحدة KiB؛ وما إذا كان مخفيًا؛ وما إذا كان Windows Installer قد ثبّته. |
| `ProductCode` | رمز منتج Windows Installer (`{GUID}`). |
| `Architecture`، `WOW64` | `x86` تحت `Wow6432Node`، أو البنية المعمارية للحزمة؛ وما إذا كان المفتاح تحت `Wow6432Node`. |
| `Details` | المزيد: `DisplayIcon`، `ModifyPath`، `ParentKeyName`، `ReleaseType`، `Inno Setup: User` (الحساب الذي شغّل مثبّت Inno Setup)؛ و`PackageCode` و`PackageName` للمنتج؛ و`Flags` و`State` و`DefaultUid` و`Version` و`PackageFamilyName` و`Default` للتوزيعة؛ و`List` و`ResourceId` و`PublisherId` للحزمة. |
| `KeyPath` | مفتاح الإدخال. |
| `DataRaw` | البايتات (بالنظام الست عشري) للقيم التي ليست كما تدل عليه أسماؤها. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows. |
| `Problems` | القيم التي تعذّرت قراءتها أو التي ليست كما تدل عليه أسماؤها. |

## استراتيجية التحقيق { #investigation-strategy }

1. رتّب بحسب `InstallDate` و`KeyLastWrittenUTC` حول وقت الحادث؛ وانظر في أدوات الوصول عن بُعد، وعملاء
   الأنفاق ونقل الملفات، وأدوات الأرشفة، وبيئات تشغيل البرامج النصية، وأدوات الأمان.
2. راجع سجلات `Scope` `user`: البرمجيات المثبتة دون صلاحيات المسؤول.
3. افحص `InstallLocation` و`InstallSource` و`UninstallCommand` بحثًا عن مجلدات قابلة للكتابة من قِبل
   المستخدم (`Downloads`، `AppData`، `Temp`، `ProgramData`).
4. ابحث عن توزيعات WSL، واجمع `BasePath` الخاص بها.
5. اربط النتائج بـ Amcache وPrefetch (المثبّت والبرنامج قيد التشغيل)، و`$MFT` (إنشاء مجلد التثبيت)،
   وسجل أحداث Application (أحداث MsiInstaller).

## مزيد من التفاصيل { #further-details }

- [Microsoft: مفتاح سجل Windows الخاص بـ Uninstall](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key)
- [Microsoft: تسجيل التطبيقات (App Paths)](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft: نظرة عامة على هوية الحزم](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
