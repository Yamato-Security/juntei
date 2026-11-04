# إعدادات الأمان { #security-settings }

## ما هي { #what-they-are }

يغيّر المهاجمون إعدادات الأمان في Windows للبقاء مخفيين وللوصول إلى بيانات الاعتماد. فهم يوقفون
Microsoft Defender أو يستثنون أدواتهم منه، ويجعلون Windows يحتفظ بكلمات المرور بنص واضح، ويفتحون
Remote Desktop، ويوقفون جدار الحماية، ويُسكتون تسجيل PowerShell. ويحتفظ سجل Windows بهذه الإعدادات:

- **`SOFTWARE`:** ‏Defender واستثناءاته ونُهُجه؛ وUAC؛ ونُهُج Remote Desktop وجدار الحماية؛ ونهج تنفيذ
  PowerShell ونُهُج التسجيل وموفّرو AMSI؛ وSystem Restore؛ وقنوات سجل الأحداث.
- **`SYSTEM`** (لكل مجموعة تحكم): ‏`WDigest`، وحماية LSA، وCredential Guard، وRestricted Admin،
  وNTLM؛ وRemote Desktop؛ وملفات تعريف جدار الحماية؛ وSMBv1؛ وأوقات آخر وصول؛ وحجم كل سجل أحداث.
- **`NTUSER.DAT`:** نهج تنفيذ PowerShell للمستخدم ونُهُج التسجيل، وAMSI لـ
  Windows Script Host.

يُبلغ `parse-registry security-settings` عن كل إعداد يعثر عليه بوصفه سجلًا. ولكل سجل قيمة الإعداد، وما
تعنيه القيمة، وما إذا كانت **تُضعف الوضع الافتراضي**: أي ما إذا كانت توقف حمايةً أو مصدرًا للأدلة، أو
تخفّضه إلى ما دون ما يفعله Windows الحالي (10 و11) في غياب هذه القيمة.

<span class="evidence-strength">أفضل استنتاج:</span> كانت لهذا الإعداد هذه القيمة عند حفظ الخلية،
وكُتب المفتاح آخر مرة في هذا الوقت. والإعداد المُضعَف خيط للتحقيق، لا دليل على الاختراق. إذ يضبط
المسؤولون وGroup Policy ومنتجات الأمان الأخرى وإصدارات Windows الأقدم بعض هذه القيم أيضًا، ويذكر
المعنى ذلك حيث يكون شائعًا.

## الموقع { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Defender[\Real-Time Protection|\Features|\Spynet]
  Microsoft\Windows Defender\Exclusions\Paths|Extensions|Processes|IpAddresses|TemporaryPaths
  Policies\Microsoft\Windows Defender\…
  Microsoft\Windows\CurrentVersion\Policies\System                 UAC
  Policies\Microsoft\Windows NT\Terminal Services                  RDP policy
  Policies\Microsoft\WindowsFirewall\<profile>                      firewall policy
  Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell             ExecutionPolicy
  Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging|ModuleLogging|Transcription
  Microsoft\AMSI\Providers\{CLSID}
  Microsoft\Windows NT\CurrentVersion\SystemRestore                DisableSR
  Microsoft\Windows\CurrentVersion\WINEVT\Channels\<channel>       Enabled, MaxSize
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Control\SecurityProviders\WDigest                                UseLogonCredential
  Control\Lsa                                                      RunAsPPL, LsaCfgFlags, DisableRestrictedAdmin, LmCompatibilityLevel
  Control\Terminal Server[\WinStations\RDP-Tcp]                    fDenyTSConnections, PortNumber, UserAuthentication
  Services\SharedAccess\Parameters\FirewallPolicy\<profile>        EnableFirewall
  Services\LanmanServer\Parameters                                 SMB1
  Control\FileSystem                                               NtfsDisableLastAccessUpdate
  Services\EventLog\<log>                                          MaxSize
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell    ExecutionPolicy
  Software\Policies\Microsoft\Windows\PowerShell\…
  Software\Microsoft\Windows Script\Settings                       AmsiEnable
```

اجمع الخلايا الثلاث كلها مع سجلات المعاملات الخاصة بها. فالإعداد الذي تغيّر قبل الجمع بوقت قصير قد لا
يوجد إلا في السجلات.

## تحذيرات للمحلل { #analyst-cautions }

- **التكوين المخزن ليس حالة الحماية الفعلية.** يُبلَّغ عن قيم سجل Windows المجمّعة وعن صفوف التشخيص
  لقوائم القيم التالفة. وقد تعكس البيانات المفقودة القيمَ الافتراضية، أو مصادر لم تُجمع، أو آلية إدارة
  أخرى؛ وهي ليست دليلًا على أن الحماية مفعّلة أو معطّلة. ويضع `WeakensDefault` علامة على الإعدادات التي
  يمكن أن تخفّض خط الأساس المذكور عند الالتزام بها.
- **قد تُتجاهل علامات Defender القديمة.** لا يثبت `DisableAntiSpyware` و`DisableAntivirus` أن Defender
  توقف. إذ تعتمد قابلية التطبيق على إصدار المنصة، والإلحاق (onboarding)، والحماية من العبث (tamper
  protection). [توضح Microsoft هذه الإعدادات القديمة](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware).
- **قد تكون الاستثناءات المُدارة الحديثة غير متاحة في لقطات سجل Windows.** بدءًا من المنصة
  4.18.25110.6، تتطلب الأجهزة التي تستخدم إدارة التكوين في Defender for Endpoint أوامر PowerShell
  (cmdlets) مدعومة خاصة بـ Defender لاسترجاع تكوين الاستثناءات. لذا فإن غيابها من سجل Windows دون اتصال
  لا يثبت غياب الاستثناءات.
  [إرشادات التكوين من Microsoft](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings).
- **المُضعَف ليس خبيثًا بالضرورة.** أمثلة:
  - يضبط Windows أيضًا `DisableAntiSpyware` = 1 ما دام برنامج مكافحة فيروسات آخر مسجلًا.
  - يُفتح Remote Desktop عمدًا على الخوادم.
  - كان Windows 7 يحتفظ بسجلات أحداث أصغر ولم يكن يحدّث أوقات آخر وصول.
- **تعتمد القيم الافتراضية على الإصدار.** كانت بعض القيم هي الافتراضية سابقًا:
  - كانت `NtfsDisableLastAccessUpdate` = 1 هي القيمة الافتراضية قبل Windows 10 1803. ويُبلَّغ عنها
    دون حكم، لكن `0x80000001`، إذا ضُبطت يدويًا على إصدار لاحق، تُعد مُضعِفة.
  - تُقارن `RunAsPPL` و`LsaCfgFlags` و`UseLogonCredential` بإصدار Windows الحالي.
- **استثناءات Defender:** كل مسار أو امتداد أو عملية أو عنوان يمثّل سجلًا. وكل استثناء يُضعف Defender.
  وليس لـ `TemporaryPaths`، التي يضيفها Defender بنفسه، حكم. ابحث عن المجلدات القابلة للكتابة من قِبل
  المستخدم، والامتدادات `.ps1`/`.exe`، وLOLBins (`rundll32.exe`، `powershell.exe`).
- **مفاتيح النُهُج** (`Policy` = True) تكتبها Group Policy، والأدوات، والمهاجمون الذين يعرفون أن النُهُج
  تتغلب على الإعدادات المحلية. اربطها بكائنات GPO الخاصة بالمجال.
- **قنوات سجل الأحداث:** لا يُبلَّغ إلا عن 25 قناة ذات صلة بالأمان، مع بيان ما إذا كان Windows يفعّل كلًا
  منها افتراضيًا. فـ `TaskScheduler/Operational` و`DNS-Client/Operational`، على سبيل المثال، معطّلتان
  افتراضيًا. استخدم `parse-registry read` للقنوات الأخرى.
- **أوقات المفاتيح:** ‏`KeyLastWrittenUTC` خاص بالمفتاح، لذا فهو ليس بالضرورة وقت تغيّر هذه القيمة. ومن
  الشائع أن يُكتب مفتاح `Policies` بأكمله دفعة واحدة.
- **مجموعات التحكم:** تُقرأ افتراضيًا مجموعة التحكم التي استخدمها Windows آخر مرة. ويقرأ
  `--all-control-sets` كل المجموعات.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use
      --weakened-only     Only settings that weaken the default

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry security-settings -d evidence/ -o settings.csv
juntei parse-registry security-settings --weakened-only -d evidence/ -t jsonl \
  | jq -r '[.Host, .Area, .KeyPath, .ValueName, .Value, .Meaning] | @tsv'
```

يعدّ الملخص الإعدادات بحسب المجال. ويسرد الإعدادات التي تُضعف الوضع الافتراضي وكل استثناء من استثناءات
Defender.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `SourcePath`، `HiveType` | المضيف، والخلية، ونوعها (`SOFTWARE`، `SYSTEM`، `NTUSER.DAT`). |
| `ControlSet`، `IsCurrentControlSet` | لـ `SYSTEM`: مجموعة التحكم، وما إذا كانت هي التي استخدمها Windows آخر مرة؛ وتكون غير معروفة عندما يكون `Select\Current` مفقودًا أو غير صالح. |
| `User`، `UserSID` | لـ `NTUSER.DAT`: اسم مجلد ملف التعريف، ومعرّف SID للمستخدم من `ProfileList` في النظام. |
| `Area` | `defender`، `uac`، `credentials`، `remote_access`، `network`، `powershell`، `evidence`. |
| `Setting` | ماهية الإعداد: `defender_disable_realtime_monitoring`، `defender_exclusion_path`، `uac_local_account_token_filter_policy`، `wdigest_use_logon_credential`، `rdp_deny_connections`، `powershell_script_block_logging`، `event_channel_enabled`، … |
| `Value` | القيمة: رقم بالنظام العشري، أو سلسلة نصية، أو الاستثناء نفسه. وتكون فارغة عندما لا تكون القيمة من نوع الإعداد. |
| `Meaning` | ما تعنيه القيمة. |
| `WeakensDefault` | `True` عندما تُضعف القيمة ما يفعله Windows الحالي في غيابها، و`False` عندما لا تُضعفه، وفارغة عندما لا ينطبق ذلك (حجم، أو منفذ، أو موفّر AMSI) أو تعذّر تحديده. |
| `Policy` | ما إذا كان المفتاح تحت مفتاح `Policies`. |
| `KeyPath`، `ValueName` | المفتاح والقيمة. واسم قيمة الاستثناء هو العنصر المستثنى؛ وليس لموفّر AMSI اسم قيمة. |
| `KeyLastWrittenUTC` | آخر كتابة للمفتاح. |
| `DataRaw` | بايتات القيمة (بالنظام الست عشري)، عندما لا تكون من نوع الإعداد. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows. |
| `Problems` | القيم من النوع الخاطئ والمفاتيح التي تتعذر قراءتها. |

## استراتيجية التحقيق { #investigation-strategy }

1. شغّله مع `--weakened-only` على كل مضيف، وقارن المضيفات بعضها ببعض: فالإعداد الذي لا يوجد إلا على
   مضيف واحد يبرز.
2. افحص كل استثناء من استثناءات Defender مقابل الأدوات الموجودة على المضيف وفي سجلات Defender
   (يسجّل الحدث 5007 تغييرات التكوين).
3. اربط أوقات المفاتيح بعمليات تسجيل الدخول وتثبيت الخدمات وإنشاء العمليات. وابحث عن أسطر أوامر
   `reg.exe` و`Set-MpPreference` و`netsh advfirewall` و`powershell` (الحدث
   4688، وSysmon 1 و13، وPowerShell 4104).
4. بالنسبة إلى إعدادات بيانات الاعتماد (`WDigest`، `RunAsPPL`، `LocalAccountTokenFilterPolicy`،
   `DisableRestrictedAdmin`)، ابحث بعدها عن تفريغ بيانات الاعتماد (credential dumping) وعن pass-the-hash.
5. استخدم `parse-registry read --recover-deleted` للقيم المحذوفة بهدف استعادة وضع افتراضي، و
   `--all-control-sets` لمجموعات التحكم الأقدم.

## مزيد من التفاصيل { #further-details }

- [Microsoft: تكوين الاستثناءات في Microsoft Defender Antivirus والتحقق منها](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus)
- [Microsoft: إعدادات User Account Control وتكوينه](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft: تكوين حماية LSA الإضافية](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft: about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001: Impair Defenses: Disable or Modify Tools](https://attack.mitre.org/techniques/T1562/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
