# BAM / DAM { #bam-and-dam }

## ما هما BAM وDAM { #what-bam-and-dam-are }

إن Background Activity Moderator ‏(BAM) وDesktop Activity Moderator ‏(DAM) خدمتان من خدمات Windows
(في Windows 10 1709 والإصدارات الأحدث) تديران ما يُسمح للتطبيقات بفعله في الخلفية. وتحتفظان، لكل
مستخدم، بآخر وقت شُغّل فيه كل برنامج، في خلية `SYSTEM`:

- مفتاح واحد لكل مستخدم، يحمل اسم معرّف SID الخاص بالمستخدم؛
- قيمة واحدة لكل برنامج: اسمها هو مسار البرنامج على جهازه
  (`\Device\HarddiskVolume3\Windows\System32\cmd.exe`) أو معرّف تطبيق محزَّم
  (`Microsoft.WindowsCalculator_8wekyb3d8bbwe`)؛
- أول 8 بايتات من بيانات القيمة هي آخر وقت شُغّل فيه البرنامج (FILETIME).

يربط BAM البرنامج بمستخدم وبوقت دون الحاجة إلى خلية المستخدم نفسه، ويسرد البرامج التي شُغّلت من
الأجهزة القابلة للإزالة والأجهزة الشبكية.

<span class="evidence-strength">أفضل استنتاج:</span> عملت عملية لهذا البرنامج في جلسة هذا المستخدم؛
وكان آخر تشغيل لها في هذا الوقت. لا يحصي BAM عدد مرات التشغيل، وقد يبقى الإدخال بعد حذف البرنامج.

## الموقع { #location }

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

يحتوي مفتاح كل مستخدم أيضًا على القيمتين `Version` و`SequenceNumber`؛ وهما ليستا برنامجين وتُستبعدان.
وقد يحتوي نظام تمت ترقيته من 1709 أو 1803 على كلا التخطيطين.

اجمع `SYSTEM` مع سجلات المعاملات الخاصة به، وخليتي `SAM` و`SOFTWARE` الخاصتين بالنظام: فهما تحددان
أسماء المستخدمين.

## تحذيرات للمحلل { #analyst-cautions }

- **وقت واحد لكل برنامج ومستخدم.** لا يُحتفظ إلا بآخر تشغيل. وتوفر Prefetch وAmcache وسجلات الأحداث
  مرات التشغيل السابقة.
- **النشاط الحديث.** يُذكر أن إدخالات البرامج التي لم تُشغَّل منذ مدة تُزال عند بدء Windows، لذا يُظهر
  BAM في الغالب النشاط الحديث: فالبرنامج الغائب عنه ربما شُغّل قبل وقت طويل.
- **مسارات أجهزة، لا أحرف محركات أقراص.** `\Device\HarddiskVolume3` هو المجلد (volume) الثالث الذي رقّمه
  Windows، وليس `C:`. يفصل `DevicePath` و`Path` الجهاز عن المسار عليه؛ طابق وحدة التخزين مع
  `MountedDevices` أو `$MFT` أو آثار رقمية أخرى قبل كتابة حرف محرك أقراص.
- **ليس كل برنامج.** يتتبع BAM وDAM البرامج في جلسات المستخدمين؛ ولا تظهر الخدمات في الغالب. إدخالات
  `S-1-5-18` هي عادةً عمليات الإعداد وشاشة تسجيل الدخول (`FirstLogonAnim.exe`)، وإدخالات
  `S-1-5-90-0-<n>` هي Desktop Window Manager ‏(`DWM-<n>`).
- **مجموعات التحكم.** افتراضيًا تُقرأ مجموعة التحكم التي استخدمها Windows آخر مرة. ويقرأ
  `--all-control-sets` كل المجموعات: فقد تحتفظ مجموعة تحكم أقدم بإدخال لم يعد موجودًا في المجموعة
  الحالية.
- **أسماء المستخدمين.** يأتي اسم الحساب المحلي من `SAM` (معرّف SID الخاص بالحاسوب ومعرّف RID الخاص
  بالحساب)؛ ويأتي اسم حساب المجال من مجلد ملف تعريفه في `ProfileList`. ويبيّن `UserSource` المصدر
  (`sam`، `profile_list`، `well_known`). ودونهما لا يُعطى سوى معرّف SID. ويبقى معرّف RID الذي لم يعد
  موجودًا في `SAM` (حساب محذوف) دون اسم.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

يسرد الملخص كل مستخدم مع معرّف SID واسمه وعدد إدخالاته، والبرامج التي شُغّلت أخيرًا (الأحدث أولًا).

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `SourcePath` | المضيف، وخلية `SYSTEM`. |
| `ControlSet`، `IsCurrentControlSet` | مجموعة التحكم التي يوجد فيها الإدخال، وما إذا كانت المجموعة التي استخدمها Windows آخر مرة. |
| `Moderator` | `bam` أو `dam`. |
| `UserSID`، `User`، `UserSource` | معرّف SID الذي يحمل المفتاح اسمه، واسم المستخدم، ومصدره (`sam`، `profile_list`، `well_known`). |
| `Program` | اسم القيمة كما كتبه Windows: مسار جهاز أو معرّف تطبيق محزَّم. |
| `DevicePath`، `Path` | بالنسبة لمسار جهاز، الجهاز (`\Device\HarddiskVolume3`) والمسار عليه (`\Windows\System32\cmd.exe`)؛ ويكونان فارغين للتطبيق المحزَّم. |
| `LastRunUTC` | وقت آخر تشغيل للبرنامج (بدقة 100 نانوثانية). |
| `KeyPath`، `KeyLastWrittenUTC` | مفتاح المستخدم ووقت آخر كتابة له: أحدث تحديث لأي من إدخالاته. |
| `DataRaw` | بايتات القيمة (بالنظام الست عشري)، عندما لا تحتوي على وقت: ليست `REG_BINARY`، أو أقل من 8 بايتات. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows. |
| `Problems` | بيانات لا تحتوي على وقت، وقيم تتعذر قراءتها. |

## استراتيجية التحقيق { #investigation-strategy }

1. رتّب حسب `LastRunUTC` وافحص البرامج الموجودة في أماكن قابلة للكتابة من قِبل المستخدم (`\Users\`،
   `Downloads`، `Temp`، `ProgramData`)، وعلى أجهزة غير وحدة تخزين النظام، وأدوات الإدارة والوصول عن بُعد.
2. تحقق من المستخدم الذي شغّلها، ومن الحسابات التي لا تطابق ملف تعريف حاليًا.
3. اربط النتائج مع Prefetch (عدد مرات التشغيل والتشغيلات السابقة)، وUserAssist (التشغيل من Explorer)،
   وAmcache وShimCache (الوجود)، و`$MFT` (ملف البرنامج)، وسجلات الأحداث.
4. استخدم `--all-control-sets`، و`parse-registry read --recover-deleted` لقيم BAM المحذوفة.

## مزيد من التفاصيل { #further-details }

- [libyal winreg-kb: Background activity moderator (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft: معرّفات الأمان (معرّفات SID المعروفة)](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

تترك قيم FILETIME لآخر تشغيل الواقعة خارج النطاق الحقلَ `LastRunUTC` فارغًا وتحفظ القيمة الأصلية في
`DataRaw`، مع شرح في `Problems`. ويحدد ملخص النشاط الحديث كل مضيف ويُبقي ملاحظات الخلايا/المستخدمين
المنفصلة متمايزة.
