# MuiCache { #muicache }

## ما هو MuiCache { #what-muicache-is }

عندما يبدأ المستخدم برنامجًا، يقرأ Windows موارد الإصدار الخاصة بالبرنامج للحصول على الاسم الذي يجب
أن تعرضه الواجهة (shell) (الحقل `FileDescription`، على سبيل المثال "Windows Command Processor" للملف
`cmd.exe`) ويخزّنه مؤقتًا لكل مستخدم في `MuiCache`. ويخزّن Windows 8 والإصدارات الأحدث مؤقتًا الشركةَ
أيضًا (`CompanyName`). ومفتاح ذاكرة التخزين المؤقت هو المسار الكامل للبرنامج:

- **Windows 8 والإصدارات الأحدث:** قيمتان لكل برنامج، `<path>.FriendlyAppName` و
  `<path>.AppCompany`.
- **Windows Vista و7:** قيمة واحدة لكل برنامج، `<path>`، تحمل الاسم المألوف (friendly name).
- **Windows XP:** الشكل نفسه، في `NTUSER.DAT`.

يمكن لـ MuiCache أن يقدّم خيوطًا لتنفيذ البرامج في جلسة الواجهة الخاصة بالمستخدم، بما في ذلك البرامج
الموجودة على محركات أقراص قابلة للإزالة، ومشاركات شبكية، ومجلدات لم تعد موجودة، مع الاسم والشركة
اللذين أعطاهما الملف نفسه.

<span class="evidence-strength">أفضل استنتاج:</span> خزّن ملف تعريف الواجهة هذا مسار ملف ووصفه وشركته
مؤقتًا. اربط ذلك بآثار التنفيذ الأخرى. ولا يحدد ملف التعريف الحساب الذي عملت به العملية. ولا يحتوي
MuiCache على وقت لكل برنامج ولا على عدد مرات التشغيل.

## الموقع { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat     Windows Vista and later
Local Settings\Software\Microsoft\Windows\Shell\MuiCache

C:\Documents and Settings\<user>\NTUSER.DAT                         Windows XP
Software\Microsoft\Windows\ShellNoRoam\MUICache
```

`LangID` (لغة السلاسل النصية المخزنة مؤقتًا) ليس برنامجًا ويُستبعد. والقيم المسماة
`@<file>,-<number>` (`@shell32.dll,-22019`) هي سلاسل موارد خزّنتها الواجهة مؤقتًا لقوائمها وأنواع
ملفاتها، وليست برامج؛ فتُستبعد وتُعدّ في الملخص. وفي Windows Vista والإصدارات الأحدث، لا يحتوي
`NTUSER.DAT` على MuiCache.

اجمع `UsrClass.dat` (و`NTUSER.DAT`) مع سجلات المعاملات الخاصة بهما لكل ملف تعريف، وخليتي `SYSTEM`
و`SOFTWARE` الخاصتين بالنظام لأسماء المضيف والمستخدمين.

## تحذيرات للمحلل { #analyst-cautions }

- **مالك ملف التعريف ليس بالضرورة حساب العملية.** يمكن لبرنامج بُدئ بحساب آخر أن يعرض نافذة في جلسة
  هذا المستخدم ويترك بيانات وصفية في ذاكرة التخزين المؤقت لهذا المستخدم.
  ويحدد `User` و`UserSid` مالك الخلية، لا سياق الأمان الخاص بالعملية.
- **لا يوجد وقت لكل برنامج.** ‏`KeyLastWrittenUTC` هو آخر كتابة للمفتاح: أحدث تغيير لأي من برامجه، لا
  الوقت الذي عمل فيه برنامج بعينه. وتعطي Prefetch وBAM وUserAssist وسجلات الأحداث الأوقات.
- **ليست البرامج التي بدأها المستخدم يدويًا فقط.** يمكن أن يظهر ملف DLL أو برنامج حمّلته الواجهة نفسها
  (من أجل `Open with`، أو امتداد للواجهة، أو عملية فرعية لمثبّت)، مثل `explorerframe.dll` أو
  `shell32.dll`.
- **الاسم يأتي من الملف.** ‏`FriendlyAppName` و`AppCompany` هما ما ذكرته موارد إصدار الملف عند تشغيله.
  ويمكن للبرمجيات الخبيثة أن تدّعي أي وصف وأي شركة؛ أما الأداة المعاد تسميتها فتحتفظ بوصفها الأصلي،
  وهو ما قد يكشفها (ملف مسمّى `svchost.exe` اسمه المألوف هو اسم أداة وصول عن بُعد).
- **إدخال واحد لكل مسار.** لا يضيف تشغيل البرنامج نفسه مجددًا إدخالًا؛ أما نسخة منه في مسار آخر فتضيف.
  ويمكن أن يبقى إدخال البرنامج بعد حذف الملف.
- **الشكلان لمسار واحد.** عندما يكون لمسار اسم مألوف بكلا الشكلين (ملف تعريف تمت ترقيته)، يحصل كل منهما
  على سجله الخاص، فلا تضيع أي قيمة.
- **القيم التالفة.** القيمة التي ليست سلسلة نصية، أو ليست UTF-16 صالحًا، أو لا يمكن قراءتها كاملة، تحتفظ
  ببايتاتها (`FriendlyAppNameRaw`، `AppCompanyRaw`) ويُبلَّغ عنها في `Problems`.
  كما يمكن أن ينتج UTF-16 غير الصالح أحرفَ استبدال في النص المعروض؛ استخدم العمود الخام للبايتات
  الأصلية.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry muicache -d evidence/ -o muicache.csv
juntei parse-registry muicache -f UsrClass.dat -t jsonl \
  | jq 'select(.Path | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

يسرد الملخص البرامج الموجودة خارج مجلدي Windows وProgram Files (مجلدات المستخدمين، ومحركات الأقراص
الأخرى، والمشاركات الشبكية) مع مستخدميها، وعدد سلاسل الموارد المستبعدة.
كما تُدرج مجلدات Program Files ذات الأسماء الأخرى (في إصدارات Windows بلغات أخرى).
وتبقى المسارات النسبية والمسارات التي تخرج من مجلد متغير بيئة مدرجة. وتُفحص مسارات محركات الأقراص
العادية بعد تحليل المكونين `.` و`..` دون الوصول إلى نظام ملفات الفاحص.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `User`، `UserSID`، `SourcePath` | المضيف، واسم مجلد ملف التعريف، ومعرّف SID للمستخدم (يسمّيه المفتاح الجذري لـ `UsrClass.dat`؛ وإلا فمن `ProfileList` في النظام)، والخلية. |
| `HiveType` | `UsrClass.dat` أو `NTUSER.DAT`. |
| `Path` | مسار البرنامج: اسم القيمة دون `.FriendlyAppName` أو `.AppCompany`. |
| `FriendlyAppName` | وصف البرنامج، من `<path>.FriendlyAppName` أو `<path>`. |
| `AppCompany` | شركة البرنامج، من `<path>.AppCompany` (Windows 8 والإصدارات الأحدث). |
| `KeyPath`، `KeyLastWrittenUTC` | مفتاح MuiCache، وآخر كتابة له: أحدث تغيير لأي من برامجه. |
| `FriendlyAppNameRaw`، `AppCompanyRaw` | بايتات القيمة (بالنظام الست عشري)، عندما لا تكون نصًا يُفك ترميزه بسلامة. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows: ‏`replayed` عندما تكون السجلات قد كتبت إحدى قيم البرنامج. |
| `Problems` | القيم التي ليست سلاسل نصية، أو ليست UTF-16 صالحًا، أو لا يمكن قراءتها. |

## استراتيجية التحقيق { #investigation-strategy }

1. انظر في البرامج الموجودة في أماكن قابلة للكتابة من قِبل المستخدم (`\Users\`، `Downloads`، `Temp`،
   `ProgramData`)، وعلى محركات أقراص أخرى وعلى مشاركات شبكية، وفي أدوات الإدارة والوصول عن بُعد والأرشفة.
2. قارن اسم الملف باسمه المألوف وشركته: قد يكشف عدم التطابق عن أداة معاد تسميتها أو ملف منتحل.
3. اربط النتائج بـ Prefetch وBAM وUserAssist (وقت التشغيل)، وAmcache وShimCache (تجزئة الملف ووجوده)،
   و`$MFT` (ما إذا كان الملف لا يزال موجودًا).
4. استخدم `parse-registry read --recover-deleted` على `UsrClass.dat` لاسترداد قيم MuiCache المحذوفة.

## مزيد من التفاصيل { #further-details }

- [Microsoft: مورد VERSIONINFO](https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource)
- [Cyber Triage: تجارب MuiCache، وجلسات الواجهة الرسومية، وقيود حساب العملية](https://www.cybertriage.com/blog/muicache-2026-guide/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
