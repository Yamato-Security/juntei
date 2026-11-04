# FeatureUsage وRecentApps والعناصر المثبتة في شريط المهام { #featureusage-recentapps-and-taskbar-pins }

## ما هي { #what-they-are }

يحتفظ Explorer بعدة سجلات لكيفية استخدام المستخدم للتطبيقات في ملف `NTUSER.DAT` الخاص بالمستخدم:

- **FeatureUsage** (Windows 10 والإصدارات الأحدث): عدادات لكل تطبيق، بحسب نوع الاستخدام.
  - `AppSwitched`: عدد مرات انتقال المستخدم إلى نافذة التطبيق (نقرة على شريط المهام،
    Alt+Tab).
  - `AppLaunch`: عدد مرات بدئه من شريط المهام (عنصر مثبّت).
  - `ShowJumpView`: عدد مرات فتح قائمة الانتقال (jump list) الخاصة به (النقر بزر الماوس الأيمن على شريط المهام).
  - `AppBadgeUpdated`: عدد مرات تغيّر شارته على شريط المهام (عدد العناصر غير المقروءة).
  - `TrayButtonClicked`: عدد مرات النقر على كل زر في شريط المهام (`StartButton`، `SearchButton`،
    `TaskViewButton`، `ClockButton`، …).

  التطبيق هو معرّف نموذج مستخدم التطبيق (application user model ID) (`Microsoft.Windows.Explorer`،
  `Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`)، أو مسار قد يكون جزؤه الأول GUID لمجلد معروف
  (`{1AC14E77-…}\cmd.exe` هو `%windir%\system32\cmd.exe`)، أو معرّف عملية
  (`*PID00001d74`) للنوافذ التي لم يتمكن Explorer من مطابقتها مع تطبيق.
- **RecentApps** (من Windows 10 حتى 1709): التطبيقات التي شغّلها المستخدم مؤخرًا، مع عدد مرات التشغيل
  وآخر وقت؛ وتحت كل منها، الملفات التي فتحها (`RecentItems`).
- **العناصر المثبتة في شريط المهام** (`Taskband`): البرامج المثبتة على شريط المهام، بوصفها عناصر واجهة
  (`Favorites`) وبوصفها ارتباط كل اختصار مثبّت (`FavoritesResolve`). وفي Windows 10 والإصدارات الأحدث،
  يبيّن كل عنصر مثبّت ما إذا كان Windows قد ثبّته (`SystemPinned`) أم المستخدم (`UserPinned`).
- **ProgramsCache** (`StartPage`، `StartPage2`؛ من Windows XP حتى 7): البرامج المخزنة مؤقتًا في قائمة
  "ابدأ" (`ProgramsCache`)، وفي Windows 7 البرامج المثبتة في قائمة "ابدأ" وفي شريط المهام
  (`ProgramsCacheSMP`، `ProgramsCacheTBP`).

<span class="evidence-strength">أفضل استنتاج:</span> استخدم هذا المستخدم هذا التطبيق هذا العدد من
المرات بهذه الطريقة (FeatureUsage)، أو شغّله في هذا الوقت (RecentApps)، أو ثبّته. وليس للعدادات وقت
خاص بها؛ ولا يؤرّخ أي شيء إلا آخر كتابة للمفتاح وأوقات RecentApps.

## الموقع { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

القيمة `KeyCreationTime` في `FeatureUsage` هي وقت بدء العدّ؛ ويعرضها الملخص. اجمع
`NTUSER.DAT` مع سجلات المعاملات الخاصة به لكل ملف تعريف، وخليتي `SYSTEM` و
`SOFTWARE` الخاصتين بالنظام للمضيف ومعرّف SID للمستخدم.

## تحذيرات للمحلل { #analyst-cautions }

- **أعداد دون أوقات.** يبيّن عدد FeatureUsage عدد المرات، لا وقتها. ووقت آخر كتابة لمفتاح النوع
  (`KeyLastWrittenUTC`) هو أحدث تغيير لأي من أعداده.
- **ليس كل تشغيل.** يعدّ `AppSwitched` عمليات التبديل بين النوافذ، لا عمليات البدء؛ فالتطبيق الذي عمل
  دون نافذة، أو الذي لم ينتقل إليه المستخدم قط، لا يُعدّ. ولا يعدّ `AppLaunch` إلا عمليات البدء من شريط
  المهام.
- **التطبيقات المحزّمة والمسارات.** يُسمّى التطبيق بالطريقة التي جمّع بها Windows نوافذه: معرّف تطبيق
  محزّم، أو معرّف حدده برنامج لنفسه (`Chrome`، `VMware.Workstation.vmui`)، أو مسار. ويعطي `AppPath`
  المسار بعد استبدال GUID المجلد المعروف بالمسار الافتراضي للمجلد.
- **العناصر المثبتة افتراضيًا.** تحصل ملفات التعريف الجديدة على عناصر مثبتة افتراضية (File Explorer،
  وEdge، وStore)؛ وفي Windows 10 والإصدارات الأحدث تحمل `SystemPinned`. أما العنصر المثبت بـ
  `UserPinned` فهو اختيار المستخدم.
- **العناصر المثبتة في قائمتين.** يصف `Favorites` و`FavoritesResolve` العناصر المثبتة نفسها بالترتيب
  نفسه (`Index`)، لكنهما يُعطيان سجلين منفصلين.
- **مسارات ProgramsCache** نسبية: يتبع المجلدَ (`Programs\Accessories`) الاختصاراتُ الموجودة فيه
  (`Notepad.lnk`).
- **القيم التالفة.** العدد الذي ليس `REG_DWORD`، أو الوقت الذي ليس `REG_QWORD`، أو العنصر المثبت غير
  المفهوم، أو الارتباط الذي ليس ارتباطًا، يحتفظ ببايتاته (`DataRaw`) ويُبلَّغ عنه في
  `Problems`.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry feature-usage <INPUT> [OPTIONS]

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
juntei parse-registry feature-usage -d evidence/ -o featureusage.csv
juntei parse-registry feature-usage -f NTUSER.DAT -t jsonl \
  | jq 'select(.Source == "AppSwitched") | [.Count, .AppPath // .App]'
```

يسرد الملخص السجلات بحسب المصدر، ووقت بدء FeatureUsage في العدّ لكل خلية، والتطبيقات الأكثر انتقالًا
إليها، والتطبيقات والعناصر الأخيرة (الأحدث أولًا)، والعناصر المثبتة في شريط المهام.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `User`، `UserSID`، `SourcePath` | المضيف، واسم مجلد ملف التعريف، ومعرّف SID للمستخدم من `ProfileList` في النظام، والخلية. |
| `Source` | `AppSwitched`، `AppLaunch`، `ShowJumpView`، `AppBadgeUpdated`، `TrayButtonClicked` (FeatureUsage)، `RecentApps`، `RecentItems`، `Taskband` أو `ProgramsCache`. |
| `App` | اسم قيمة FeatureUsage، أو `AppId` لتطبيق حديث، أو معرّف نموذج مستخدم التطبيق لعنصر مثبّت. |
| `AppPath` | `App` بصيغة مسار، بعد استبدال GUID المجلد المعروف بمساره الافتراضي؛ ويكون فارغًا للمعرّفات. |
| `Count` | عدد FeatureUsage، أو `LaunchCount` لتطبيق حديث. |
| `LastAccessedUTC` | `LastAccessedTime` لتطبيق أو عنصر حديث. |
| `ItemPath` | `Path` لعنصر حديث (أو `AppPath` لتطبيق حديث)؛ أو مسار عنصر الواجهة لعنصر مثبّت أو برنامج مخزن مؤقتًا (`User Pinned\TaskBar\File Explorer.lnk`، `Applications\<app ID>`)؛ أو مسار ارتباط `FavoritesResolve`. |
| `DisplayName` | `DisplayName` لعنصر حديث. |
| `PinType` | `SystemPinned` أو `UserPinned`، لعنصر مثبّت في `Favorites`. |
| `Index` | موضع العنصر المثبّت أو البرنامج المخزن مؤقتًا في قائمته. |
| `ValueName`، `KeyPath`، `KeyLastWrittenUTC` | القيمة والمفتاح اللذان يأتي منهما، وآخر كتابة للمفتاح. |
| `DataRaw` | البايتات (بالنظام الست عشري) التي لا يُفك ترميزها. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows. |
| `Problems` | قيم من النوع الخاطئ، أو عناصر مثبتة أو قوائم غير مفهومة، أو ارتباطات ليست ارتباطات. |

## استراتيجية التحقيق { #investigation-strategy }

1. رتّب `AppSwitched` بحسب `Count` لمعرفة التطبيقات التي عمل فيها المستخدم أكثر من غيرها، وابحث عن
   أدوات الإدارة والوصول عن بُعد والأرشفة والنقل، وعن المسارات في أماكن قابلة للكتابة من قِبل المستخدم
   (`\Users\`، `Downloads`، `Temp`).
2. افحص العناصر المثبتة: أداة بعلامة `UserPinned`، أو عنصر مثبّت يقع اختصاره في مكان غير معتاد.
3. استخدم أوقات RecentApps، مع UserAssist وPrefetch وBAM وJump Lists، لتأريخ الاستخدام.
4. قارن `KeyCreationTime` بوقت إنشاء ملف التعريف: يبدأ العدّ حينها.
5. استخدم `parse-registry read --recover-deleted` لاسترداد العدادات والعناصر المثبتة المحذوفة.

## مزيد من التفاصيل { #further-details }

- [Microsoft: معرّفات نموذج مستخدم التطبيق (Application User Model IDs)](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [winreg-kb من libyal: ذاكرة التخزين المؤقت للبرامج (Programs cache)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
