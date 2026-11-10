# UserAssist { #userassist }

## ما هو UserAssist { #what-userassist-is }

يحصي Windows Explorer البرامج والاختصارات التي يبدؤها المستخدم من واجهة Explorer: قائمة "ابدأ"، وشريط
المهام، وسطح المكتب، والنقر المزدوج في المجلدات. ويحتفظ بهذه الأعداد في ملف `NTUSER.DAT` الخاص
بالمستخدم. ولكل عنصر:

- عدد مرات التشغيل؛
- على Windows 7 والإصدارات الأحدث، عدد المرات ومدة (بالملّي ثانية) حصوله على التركيز (focus)؛
- وقت آخر تشغيل له.

أسماء القيم مرمّزة بـ ROT13. ويبدأ اسم البرنامج عادةً بمعرّف GUID لمجلد معروف (`{1AC14E77-…}` هو
`System32`).

يُظهر UserAssist ما بدأه مستخدم بعينه تفاعليًا، وهو ما لا تنسبه معظم أدلة التنفيذ الأخرى إلى مستخدم.

<span class="evidence-strength">أفضل استنتاج:</span> بدأ المستخدم هذا البرنامج أو الاختصار عبر
Explorer، بهذا العدد من المرات، وكانت آخر مرة في هذا الوقت. ولا تُحتسب البرامج التي بدأت بطريقة أخرى
(سطر أوامر، أو خدمة، أو مهمة مجدولة، أو برنامج آخر).

## الموقع { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | يحتوي على |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | البرامج (`executable`)، في Windows 7 والإصدارات الأحدث |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | الاختصارات (`shortcut`)، في Windows 7 والإصدارات الأحدث |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | البرامج والاختصارات (`active_desktop`)، من Windows 2000 إلى Vista |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | شريط أدوات الإنترنت (`internet_toolbar`)، من Windows 2000 إلى Vista |

يُبلَغ عن معرّفات GUID الأخرى بوصفها `other`. اجمع `NTUSER.DAT` مع سجلات المعاملات الخاصة به لكل ملف
تعريف، وخلية `SOFTWARE` الخاصة بالنظام، التي توفر معرّف SID الخاص بالمستخدم.

## تحذيرات للمحلل { #analyst-cautions }

- **تُزوَّد ملفات التعريف الجديدة بعناصر أولية.** على Windows 7 والإصدارات الأحدث، يبدأ ملف التعريف الجديد
  بعناصر افتراضية (Getting Started، وCalculator، وPaint، وSticky Notes، …). وتتناقص أعداد تشغيلها
  من 14، وتشترك جميعها في وقت آخر تشغيل واحد، قريب من وقت إنشاء ملف التعريف. وهي ليست نشاطًا للمستخدم.
- **الإصدار 3 يبدأ العدّ من 5** (من Windows 2000 إلى Vista). يبدأ العدد المخزن من 5، لذا يُطرح منه 5.
  ويُبلَغ عن العدد المخزن الأقل من 5 في `Problems`.
- **تستخدم المسارات المسار الافتراضي للمجلد المعروف** (`%windir%\system32`، `%ProgramFiles%`،
  `%APPDATA%\…`). والمجلد المعاد توجيهه إلى مكان آخر لا يزال يُظهر مساره الافتراضي. ولا تملك معرّفات
  التطبيقات (`Microsoft.Windows.Explorer`، `Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`) وعناصر Control
  Panel ‏(`::{…}`) مسارًا.
- **يمكن إيقاف التتبع** عبر `Start_TrackProgs` = 0 في `Explorer\Advanced`، أو `NoInstrumentation` = 1
  في `Policies\Explorer`، أو `NoLog` = 1 في `UserAssist\Settings`. ويسرد الملخص الخلايا التي يكون فيها
  التتبع موقوفًا. وقد يعني UserAssist الفارغ أو المفقود في ملف تعريف نشط وجود مكافحة للتحليل الجنائي
  (anti-forensics).
- **`NoEncrypt` = 1** يجعل Windows يكتب الأسماء دون ROT13. ويُفك ترميز مثل هذه الأسماء بشكل خاطئ، لذا
  يذكر كل عنصر في تلك الخلية ذلك في `Problems` (وعلى stderr عندما تُرسل السجلات إلى stdout).
- **يُحتفظ بوقت آخر تشغيل وبالأعداد لكل عنصر،** لكن وقت آخر كتابة للمفتاح (`KeyLastWrittenUTC`) لا يمثّل
  سوى وقت أحدث تحديث لأي عنصر.
- **قد تكون أعداد التشغيل 0** للعناصر التي حصلت على التركيز فقط. ولا توجد أعداد التركيز وأوقاته إلا في
  الإصدار 5 (Windows 7 والإصدارات الأحدث).

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry user-assist <INPUT> [OPTIONS]

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
juntei parse-registry user-assist -d evidence/ -o userassist.csv
juntei parse-registry user-assist -f NTUSER.DAT -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

يسرد الملخص العناصر التي شُغّلت أخيرًا (الأحدث أولًا) مع مستخدميها، والخلايا التي يكون فيها UserAssist
موقوفًا.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `User`، `UserSID`، `SourcePath` | المضيف، واسم مجلد ملف التعريف، ومعرّف SID الخاص بالمستخدم من `ProfileList` في النظام (مع خلية `SOFTWARE` الخاصة بالنظام)، والخلية. |
| `GUID`، `Kind` | مفتاح `{GUID}` وما يحتويه (`executable`، `shortcut`، `active_desktop`، `internet_toolbar`، `other`). |
| `NameRot13`، `Name` | اسم القيمة كما خُزّن، وبعد فك ترميزه. |
| `Path` | الاسم بعد فك ترميزه بوصفه مسارًا، مع استبدال معرّف GUID للمجلد المعروف بمساره الافتراضي؛ ويكون فارغًا لمعرّفات التطبيقات وعناصر الواجهة. |
| `RunCount` | عدد مرات بدئه من Explorer (الإصدار 3: العدد المخزن ناقص 5). |
| `FocusCount`، `FocusTimeMs` | عدد مرات حصوله على التركيز، ومدته بالملّي ثانية (الإصدار 5). |
| `LastRunUTC` | وقت آخر بدء له. ويكون فارغًا عندما يخزن Windows القيمة 0، أو عندما يتجاوز الوقت المخزن العام 9999 (وحينئذٍ يذكره `Problems` ويحتفظ `DataRaw` بالبايتات). |
| `Format`، `SessionID` | تخطيط البيانات (5: ‏72 بايتًا، 3: ‏16 بايتًا) ورقم الجلسة التي كُتب فيها آخر مرة. |
| `KeyLastWrittenUTC` | آخر كتابة لمفتاح `Count`: أحدث تحديث لأي من عناصره. |
| `DataRaw` | بايتات القيمة (بالنظام الست عشري)، عندما يتعذر فك ترميزها. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows. |
| `Problems` | بيانات بحجم غير معروف، أو حجم لا يتفق مع `Version` الخاص بالمفتاح، أو عدد في الإصدار 3 أقل من 5، أو آخر تشغيل يتجاوز العام 9999، أو `NoEncrypt`. |

## استراتيجية التحقيق { #investigation-strategy }

1. رتّب حسب `LastRunUTC` واستبعد العناصر الأولية: فهي تشترك في وقت واحد، قريب من وقت إنشاء ملف التعريف.
2. ابحث عن البرامج الموجودة في أماكن قابلة للكتابة من قِبل المستخدم (`%USERPROFILE%`، `Downloads`،
   `Temp`، `ProgramData`)، وعلى محركات الأقراص القابلة للإزالة ومسارات الشبكة، وعن أدوات الإدارة
   والوصول عن بُعد.
3. اربط النتائج مع Prefetch (التنفيذ مع أوقات التشغيل)، وAmcache وShimCache (الوجود)، وJump Lists
   وملفات LNK (ما فُتح بها)، و`$MFT` لملفات البرامج.
4. افحص الخلايا التي يكون فيها التتبع موقوفًا، وملفات التعريف التي يكون UserAssist فيها فارغًا، بحثًا عن
   عمليات المسح.
5. استخدم `parse-registry read --recover-deleted` لقيم UserAssist المحذوفة.

## مزيد من التفاصيل { #further-details }

- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: مفاتيح User Assist](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
