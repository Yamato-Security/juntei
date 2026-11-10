# التشغيل التلقائي عبر سجل Windows { #registry-autostarts }

## ما هي { #what-they-are }

يبدأ Windows البرامج ويحمّل ملفات DLL من مواقع كثيرة في سجل Windows: عند الإقلاع، وعند تسجيل الدخول،
وعند بدء برنامج ما أو خروجه، وعند فتح `cmd.exe`، وعند إنشاء فئة COM، أو عندما يحمّل المخزن المؤقت
للطباعة (spooler) أو LSA أو netsh أو خدمة الوقت المكونات الإضافية الخاصة به. يستخدم المهاجمون هذه
المواقع لتحقيق الاستمرارية، وتصعيد الامتيازات، وحقن ملفات DLL، كما تستخدمها البرامج المشروعة أيضًا.
يقرؤها Juntei من `SOFTWARE` و`SYSTEM` و`NTUSER.DAT` و`UsrClass.dat` في جدول واحد. ويكتب سجلًا واحدًا
لكل برنامج أو ملف DLL أو حزمة، ويوسم كلًا منها بتقنية MITRE ATT&CK الخاصة به.

<span class="evidence-strength">أفضل استنتاج:</span> كوّنت الخلية هذا البرنامج أو ملف DLL ليبدأ أو
يُحمَّل. ولا يثبت ذلك أن البرنامج قد شُغّل. فتشغيله يتطلب الشروط التي تحددها الآلية (تسجيل دخول، أو بدء
البرنامج الخاضع للتصحيح، أو إنشاء الفئة)، وأدلة أخرى على التنفيذ.

## المواقع { #locations }

| الخلية | ما الذي يُقرأ |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run` و`RunOnce` (+`Setup`) و`RunOnceEx` و`RunServices(Once)` و`Policies\Explorer\Run`، مع نسخها في `Wow6432Node` ونسخ وضع التثبيت في Terminal Server؛ و`Explorer\StartupApproved\Run` و`Run32` و`StartupFolder`؛ و`Winlogon` ‏(`Shell`، `Userinit`، `Taskman`، `AppSetup`، `GinaDLL`، … و`Notify`)؛ و`AppInit_DLLs` في `Windows` مع `LoadAppInit_DLLs`؛ و`Debugger` في `Image File Execution Options\*` (مع `UseFilter`) و`VerifierDlls`؛ و`MonitorProcess` في `SilentProcessExit\*`؛ و`StubPath` في `Active Setup\Installed Components\*`؛ و`NetSh`؛ و`AutoRun` في `Command Processor` |
| `SYSTEM` | `Control\Session Manager` ‏(`BootExecute`، `SetupExecute`، `Execute`، `PendingFileRenameOperations`، `KnownDLLs`، `AppCertDlls`، ومتغيرات محلل أداء .NET ‏(profiler) في `Environment`)؛ و`Authentication` و`Security` و`Notification Packages` (+`OSConfig`) في `Control\Lsa`؛ و`SecurityProviders`؛ وشاشات المراقبة والموفرون ومعالجات الطباعة في `Control\Print`؛ و`Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | مفاتيح `Run` الخاصة بالمستخدم و`StartupApproved`؛ و`Load` و`Run` في `Windows NT\CurrentVersion\Windows`؛ و`Shell` في `Winlogon`؛ و`Environment` ‏(`UserInitMprLogonScript`، ومحلل أداء .NET)؛ و`SCRNSAVE.EXE` في `Control Panel\Desktop`؛ و`PackagedAppXDebug`؛ و`AutoRun` في `Command Processor`؛ و`Software\Classes` (بوصفه `UsrClass.dat`) |
| `UsrClass.dat` | `CLSID\*\InprocServer32` و`LocalServer32` (ونظائرهما في `Wow6432Node`)؛ و`shell\open\command` لأنواع البرامج (`exefile`، `batfile`، `scrfile`، …) ولما تبدؤه البرامج ذاتية رفع الامتيازات (`ms-settings`، `mscfile`، `Folder`، …)؛ و`ActivatableClasses\…\DebugInformation` |

اجمع كل خلية مع سجلات المعاملات الخاصة بها. توجد خلايا المستخدمين في `Users\<name>\NTUSER.DAT`
و`Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat`.

## تحذيرات للمحلل { #analyst-cautions }

- **يُبلَغ عن الإعدادات الافتراضية أيضًا.** معظم السجلات على نظام نظيف هي إعدادات Windows الافتراضية:
  - `Userinit` هو `userinit.exe`؛
  - `Shell` هو `explorer.exe`؛
  - لكل من `KnownDLLs` وحزم LSA وشاشات مراقبة الطباعة ومساعدات netsh ومكونات Active Setup إدخالات قياسية.

  ابحث عمّا يختلف عن بنية نظيفة من الإصدار نفسه.
- **لا يغطي `StartupApproved` سوى قائمة Startup في Task Manager.** فهو يبيّن ما إذا كان إدخال `Run`
  أو ملف في مجلد Startup معطّلًا هناك، ومتى. ويُبلَغ عن الإدخال الذي لا تقابله قيمة `Run`، مع التفاصيل
  "only in StartupApproved". ويمكن أن يكشف ذلك عن تشغيل تلقائي أُزيل.
- **يحتاج `OverridesMachineRegistration` إلى خلية `SOFTWARE` الخاصة بالنظام.** تُفحص فئة COM أو الأمر
  الخاص بالمستخدم مقابلها، ويُعثر عليها بالطريقة نفسها التي يُعثر بها على خلية `SYSTEM` الخاصة
  بالمضيف. والتجاوز هو نمط اختطاف COM ‏(T1546.015). كما تسجّل OneDrive وغيرها من عمليات التثبيت لكل
  مستخدم فئات خاصة بها لا تتجاوز شيئًا.
- **لا يُفصل `ImagePath` و`Args` عن سطر الأوامر إلا حيث يمكن التمييز بينهما.**
  - يُفصل البرنامج المحاط بعلامات اقتباس عند علامة الاقتباس الختامية.
  - تُعدّ الكلمة الأولى التي لا تتضمن مجلدًا هي البرنامج (`cmd /c …`).
  - وفي غير ذلك يكون الفصل عند أقصر بداية تنتهي بامتداد برنامج.

  قد يشغّل المسار غير المحاط بعلامات اقتباس والمتضمن مسافات برنامجًا مختلفًا (`C:\Program.exe`) إن وُجد.
- **يحدد `GlobalFlag` قيمة `Enabled` لكل من `VerifierDlls` و`MonitorProcess`.** لا يُحمَّل `VerifierDlls`
  إلا مع `0x100`. ولا يعمل `MonitorProcess` إلا مع `0x200` ومع `0x1` في `ReportingMode` ضمن
  `SilentProcessExit\<program>`. والوضع `2` (تفريغ dump) أو `4` (إشعار) لا يشغّل شيئًا، ويسمّي `Details`
  علامات الوضع.
- **`KeyLastWrittenUTC` هو آخر كتابة للمفتاح**، لا للقيمة. وتشترك فيه عدة قيم.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.Mechanism != "com_server" or .OverridesMachineRegistration == true)'
```

يحصي الملخص السجلات حسب الآلية. كما يسرد إدخالات `StartupApproved\Run` و`Run32` التي بقيت دون قيمة
`Run` الخاصة بها (ليس لملفات مجلد Startup ما تفقده)، وفئات COM والأوامر الخاصة بالمستخدم التي تتجاوز
نظائرها على مستوى الجهاز.

وكسائر أوامر `parse-registry`، يرفض هذا الأمر قيمة `-o` تكون أحد الملفات التي يقرؤها، حتى مع `-C`.
ويشمل ذلك الخلية، وسجلات المعاملات الخاصة بها، وخليتي SYSTEM وSOFTWARE الخاصتين بالنظام، سواء بالمسار
أو بالارتباط الرمزي أو بالارتباط الصلب. وعندما يتعذر عليه التحديد، لا يكتب.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `SourcePath`، `HiveType` | سياق المضيف، والخلية المصدر ونوعها. |
| `User`، `UserSID` | لخلايا المستخدمين: مجلد ملف التعريف، ومعرّف SID. ويأتي معرّف SID من اسم المفتاح الجذر في `UsrClass.dat`، وإلا فمن `ProfileList` في النظام. |
| `Mechanism` | `run`، `run_once`، `winlogon`، `ifeo_debugger`، `silent_process_exit`، `appinit_dlls`، `appcert_dlls`، `boot_execute`، `lsa_security_package`، `com_server`، `uac_bypass_command`، … |
| `AttackTechnique` | تقنية MITRE ATT&CK، حيثما انطبقت إحداها (`T1547.001`، `T1546.012`، …). |
| `KeyPath`، `ValueName` | المفتاح تحت جذر الخلية (الأسماء كما كُتبت) والقيمة (فارغة: القيمة الافتراضية). |
| `Data` | نص القيمة، أو عنصر واحد من قائمة. ويكون فارغًا عندما تتعذر قراءته، أو عندما لا يسمّي الإدخالَ سوى `StartupApproved`. |
| `ImagePath`، `Args` | البرنامج أو ملف DLL، والوسيطات، حيثما أمكن التمييز بينهما (انظر أعلاه). |
| `Enabled`، `DisabledAtUTC` | ما يقوله `StartupApproved` أو `LoadAppInit_DLLs` أو `GlobalFlag` (مع `ReportingMode`) أو `UseFilter` أو `IsInstalled` أو `Enabled` الخاص بموفر الوقت أو `ScreenSaveActive`. ويكون فارغًا عندما لا يقول أي منها شيئًا. |
| `Details` | مزيد من المعلومات: ما تفعله عملية إعادة تسمية معلّقة، واسم فئة COM، و`ReportingMode`، و`FilterFullPath`، و`ShellComponent` في Active Setup، … |
| `OverridesMachineRegistration` | لفئة COM أو أمر خاص بالمستخدم: ما إذا كان الجهاز يسجّل الفئة/الأمر نفسه. ويكون فارغًا دون خلية `SOFTWARE` الخاصة بالنظام. |
| `WOW64` | تحت `Wow6432Node`: للبرامج ذات 32 بت. |
| `KeyLastWrittenUTC` | آخر كتابة للمفتاح. |
| `DataRaw` | بايتات القيمة (بالنظام الست عشري)، عندما لا تكون نصًا أو لا تكون UTF-16 صالحًا. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows. |
| `Problems` | أنواع قيم خاطئة، أو بيانات تتعذر قراءتها، أو إدخالات `StartupApproved` ذات حجم خاطئ. |

## استراتيجية التحقيق { #investigation-strategy }

1. ابدأ بالملخص:
   - فئات COM والأوامر التي تتجاوز نظائرها على مستوى الجهاز؛
   - إدخالات `StartupApproved` التي لا تقابلها قيمة `Run`؛
   - الآليات التي نادرًا ما تحتوي على إدخالات (`ifeo_debugger`، `silent_process_exit`، `appinit_dlls`، `appcert_dlls`، `uac_bypass_command`، `file_association`، `logon_script`، `dotnet_profiler`).
2. قارن حزم LSA و`BootExecute` و`KnownDLLs` وشاشات مراقبة الطباعة ومساعدات netsh مع بنية نظيفة من
   إصدار Windows نفسه.
3. حلّ `ImagePath` مقابل نظام الملفات ($MFT، والتجزئات، والتوقيعات)، وافحص مسارات ملف تعريف المستخدم
   و`ProgramData` و`Temp`.
4. اربط `KeyLastWrittenUTC` مع أحداث سجل Windows في Security وSysmon، وإنشاء العمليات، وPrefetch،
   وAmcache.
5. استخدم `parse-registry read --recover-deleted` للعثور على عمليات التشغيل التلقائي التي أُزيلت من هذه
   المفاتيح.

## مزيد من التفاصيل { #further-details }

- [MITRE ATT&CK T1547 Boot or Logon Autostart Execution](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 Event Triggered Execution](https://attack.mitre.org/techniques/T1546/)
- [Microsoft: مفاتيح سجل Windows ‏Run وRunOnce](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
