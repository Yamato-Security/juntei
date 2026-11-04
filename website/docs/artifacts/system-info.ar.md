# معلومات النظام { #system-information }

## ما هو { #what-it-is }

تخبر خلايا الجهاز الأربع للنظام بما يحتاجه المحلل قبل قراءة جدوله الزمني:

- أسماؤه (NetBIOS، واسم مضيف DNS، ومجال DNS) والمجال أو مجموعة العمل التي ينتمي إليها؛
- إصدار Windows، والبنية (build) ومراجعة التحديث، ووقت تثبيت Windows أو آخر ترقية له؛
- منطقته الزمنية، التي تخبرك بالأوقات المحلية التي احتفظ بها النظام؛
- وقت آخر إيقاف تشغيل سليم له؛
- بنية المعالج وصفحات الترميز؛
- آخر مستخدم سجّل الدخول، وملفات تعريف كل مستخدم سجّل الدخول؛
- معرّفات SID الخاصة به: معرّف الحاسوب (الحسابات المحلية هي هذا المعرّف مضافًا إليه RID) ومعرّف مجاله؛
- إصدارات Windows التي رُقّي منها.

يكتب `parse-registry system-info` سجلًا واحدًا لكل نظام يتضمن كل ذلك.

<span class="evidence-strength">أفضل استنتاج:</span> التكوين كما كان عند جمع الخلايا
(أو عند آخر كتابة لها). وتوضح أوقات المفاتيح متى تغيرت أجزاء منه آخر مرة.

## الموقع { #location }

```text
C:\Windows\System32\config\SYSTEM     (the control set Windows used last: Select\Current)
  ControlSet00N\Control\ComputerName\ComputerName     ComputerName
  ControlSet00N\Services\Tcpip\Parameters             Hostname, Domain
  ControlSet00N\Control\TimeZoneInformation           TimeZoneKeyName, Bias, ActiveTimeBias, …
  ControlSet00N\Control\Windows                       ShutdownTime
  ControlSet00N\Control\Session Manager\Environment   PROCESSOR_ARCHITECTURE
  ControlSet00N\Control\Nls\CodePage                  ACP, OEMCP
  Setup\Source OS (Updated on M/D/YYYY HH:MM:SS)      the version before each upgrade

C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion                 ProductName, EditionID, DisplayVersion,
                                                      CurrentBuild, UBR, InstallDate, InstallTime, …
  Microsoft\Windows NT\CurrentVersion\ProfileList     one key per profile (SID)
  Microsoft\Windows\CurrentVersion\Authentication\LogonUI   LastLoggedOnUser, …

C:\Windows\System32\config\SAM
  SAM\Domains\Account                                 V: the computer's SID at its end

C:\Windows\System32\config\SECURITY
  Policy\PolAcDmN, PolAcDmS                           the computer's name and SID
  Policy\PolPrDmN, PolPrDmS                           its domain (or workgroup) and the domain's SID
  Policy\PolDnDDN, PolDnTrN, PolDnDmG                 the domain's and forest's DNS names, GUID
  Policy\PolMachineAccountS                           the computer account's SID in the domain
```

اجمع الخلايا الأربع مع سجلات المعاملات الخاصة بها. وتعطي أي خلية منها وحدها ما تحتويه.

## تحذيرات للمحلل { #analyst-cautions }

- **أوقات سجل Windows بتوقيت UTC؛ والمنطقة الزمنية لكل ما عدا ذلك.** أوقات المفاتيح وقيم FILETIME ومعظم
  أوقات الأحداث بتوقيت UTC. أما السجلات النصية، وبعض سجلات التطبيقات، والتواريخ المكتوبة في الأسماء أو السلاسل النصية
  (مثل `Source OS (Updated on …)`) فقد تكون محلية. و`Bias` هو عدد الدقائق من التوقيت القياسي المحلي
  إلى UTC ‏(`UTC = local + Bias`). ويعطي `UtcOffset` و`DaylightUtcOffset` الإزاحتين،
  ويوضح `TimeZoneDescription` متى يبدأ التوقيت الصيفي وينتهي.
- **قد تتغير المنطقة الزمنية.** `TimeZoneLastWrittenUTC` هو آخر كتابة للمفتاح. ويعيد Windows أيضًا
  كتابة المفتاح من تلقاء نفسه عندما يحدّث `ActiveTimeBias` (الإزاحة السارية حينها)، لذا فإن
  الوقت الحديث لا يثبت أن أحدًا غيّر المنطقة. قارن بسجلات الأحداث: الحدث Security 4616
  (تغيّر وقت النظام) وأحداث `Microsoft-Windows-Kernel-General` في سجل System.
- **‏`StandardStart` و`DaylightStart` هما بنيتا `TIME_FIELDS` الخاصتان بالنواة، لا `SYSTEMTIME`.** فيوم
  الأسبوع هو حقلهما الأخير، لا الثالث (كما في قيمة `TZI` للمنطقة في `SOFTWARE`). وفي
  مناطق الولايات المتحدة تُقرأ البنيتان بالطريقة نفسها، لأن يوم الأسبوع هو الأحد (0). ويفك Juntei ترميزهما
  بوصفهما `TIME_FIELDS`: ويُظهر نظام عام من جزيرة إيستر (أول سبت عند 22:00)
  الفرق. ويعني الشهر 0 عدم وجود توقيت صيفي.
- **Windows 11 يقول "Windows 10".** يحتفظ `ProductName` بالقيمة `Windows 10 …` على Windows 11. والبنى 22000
  وما بعدها هي Windows 11، ويذكر الملخص ذلك. وتكون `CurrentVersion` هي `6.3` على Windows 8.1 و10
  و11؛ ويستخدم `Version` القيمة `CurrentMajorVersionNumber` ‏(10) عند وجودها.
- **يتحرك `InstallDate` مع ترقيات الميزات.** تعيد الترقية في المكان إلى إصدار جديد (من 1809 إلى
  21H2، مثلًا) ضبطه. ويوجد الإصدار الذي استبدلته كل ترقية، مع `InstallDate` الخاص به،
  في `PreviousVersions` ‏(`Setup\Source OS`). وأقدم `InstallDate` بينها هو الأقرب إلى
  التثبيت الأصلي. ولا يوضح التاريخ الموجود في اسم مفتاح `Source OS` منطقته الزمنية.
- **‏`ShutdownTime` هو آخر إيقاف تشغيل سليم.** لا يحدّثه الانهيار أو انقطاع الطاقة.
  قارنه بسجلات الأحداث (6006، 6008، 1074) وبأوقات آخر كتابات للخلايا.
- **‏`LastLoggedOnUser` هو آخر مستخدم في شاشة تسجيل الدخول،** لا كل عمليات تسجيل الدخول. فسطح المكتب البعيد،
  و`runas`، والخدمات، والمهام المجدولة لا تضبطه كلها. و`LogonUiLastWrittenUTC` يقارب وقت
  آخر ضبط له.
- **ملفات التعريف:** يسرد `ProfileList` كل مستخدم (وخدمة) كان له ملف تعريف على
  النظام، بما في ذلك الحسابات المحذوفة إلى أن يُزال ملف تعريفها. ويكتب Windows 10 والإصدارات الأحدث
  `LocalProfileLoadTime` و`LocalProfileUnloadTime`: وهما يقاربان آخر تسجيل دخول وآخر تسجيل خروج.
- **معرّفات SID:** الحسابات المحلية هي `MachineSid-<RID>`، وحسابات المجال `DomainSid-<RID>`.
  ويأتي `MachineSid` من `SAM` (وإلا فمن `PolAcDmS` في `SECURITY`). وينبغي أن يتطابق الاثنان؛
  وعندما لا يتطابقان، يذكر `Problems` ذلك، كما يحدث مع خلايا من أنظمة مختلفة. ويكون `DomainSid` فارغًا
  في مجموعة العمل.
- **مجلد واحد، نظام واحد.** مع `-d`، تُعد الخلايا الموجودة في مجلد واحد خلايا نظام واحد.
  ونسخ `RegBack` والخلية الثانية من النوع نفسه (`SYSTEM.old`) سجلات مستقلة.
  ومع `-f`، تُعد الخلايا المعطاة خلايا نظام واحد، ويُبحث عن الخلايا الأخرى بجانبها.
- **مجموعات التحكم:** لا تُقرأ إلا مجموعة التحكم التي استخدمها Windows آخر مرة. ويُظهر `parse-registry read`
  المجموعات الأخرى.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM, SOFTWARE, SAM and SECURITY hives
  -f, --file <FILE>      A hive of one system; repeat for each
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry system-info -d collection/ -o system-info.csv
juntei parse-registry system-info -f SYSTEM -f SOFTWARE -t jsonl \
  | jq '{Host, TimeZoneDescription, ProductName, Build, LastShutdown}'
```

يعطي الملخص لكل نظام: إصدار Windows، ووقت تثبيته، ومنطقته الزمنية بالكلمات،
والإزاحة السارية عند آخر كتابة للمفتاح، وآخر إيقاف تشغيل له، ومجاله أو مجموعة
عمله، وآخر مستخدم سجّل الدخول، ومعرّف SID الخاص به، وعدد ملفات التعريف لديه، والإصدارات التي
رُقّي منها. ويطبع `auto-parse` أيضًا المنطقة الزمنية لكل نظام عندما يسرد الأنظمة التي
عثر عليها.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `SourcePath`، `SoftwarePath`، `SamPath`، `SecurityPath` | المضيف، والخلايا المقروءة (`SourcePath` هو خلية `SYSTEM`، وإلا فأول خلية قُرئت). |
| `CompName`، `Hostname`، `Domain` | اسم NetBIOS ‏(`SYSTEM`، وإلا فـ `PolAcDmN` في `SECURITY`)، واسم مضيف DNS ومجال DNS ‏(`Tcpip\Parameters`). |
| `PrimaryDomain`، `DNS-Domain` | المجال الذي انضم إليه الحاسوب، أو مجموعة عمله (`PolPrDmN`)، واسم DNS للمجال (`PolDnDDN`). |
| `ProductName`، `Edition`، `DisplayVersion`، `Version`، `Build`، `Ubr`، `ServicePack`، `InstallationType` | من `CurrentVersion`: اسم المنتج كما كُتب، و`EditionID`، و`DisplayVersion` (وإلا فـ `ReleaseId`)، والإصدار الرئيسي والفرعي (وإلا فـ `CurrentVersion`)، والبنية، ومراجعة بنية التحديث، و`CSDVersion`، و`Client` أو `Server` أو `Server Core`. |
| `InstallDateUTC`، `InstallTimeUTC` | `InstallDate` (حتى الثانية) و`InstallTime` (حتى 100 نانوثانية، Windows 10 والإصدارات الأحدث): التثبيت أو آخر ترقية ميزات. |
| `RegisteredOwner`، `RegisteredOrganization`، `SystemRoot` | كما كُتبت. |
| `TimeZone`، `TimeZoneDescription` | `TimeZoneKeyName` (وإلا فـ `StandardName`)، والمنطقة بالكلمات مع إزاحاتها وتواريخ التوقيت الصيفي. |
| `UTC-Offset`، `DaylightUTC-Offset` | إزاحة التوقيت القياسي المحلي عن UTC، وإزاحة التوقيت الصيفي (فارغة دون توقيت صيفي). |
| `BiasMinutes`، `ActiveBiasMinutes`، `StandardBiasMinutes`، `DaylightBiasMinutes` | `Bias`، و`ActiveTimeBias` (الساري عند كتابة المفتاح)، و`StandardBias`، و`DaylightBias`. |
| `StandardName`، `DaylightName` | كما كُتبا، وغالبًا مرجع لمورد (`@tzres.dll,-112`). |
| `StandardStart`، `DaylightStart` | متى يبدأ التوقيت القياسي والصيفي، بالكلمات (`first Sunday of November, 02:00`). |
| `DynamicDaylightTimeDisabled`، `RealTimeIsUniversal` | تعطيل التوقيت الصيفي الديناميكي؛ وساعة العتاد تحتفظ بتوقيت UTC. |
| `TimeZoneLastWrittenUTC` | آخر كتابة لمفتاح المنطقة الزمنية. |
| `LastShutdownUTC` | `ShutdownTime`: آخر إيقاف تشغيل سليم. |
| `CurrentControlSet`، `ProcessorArchitecture`، `AnsiCodePage`، `OemCodePage` | مجموعة التحكم المقروءة، و`PROCESSOR_ARCHITECTURE`، و`ACP` و`OEMCP`. |
| `LastLoggedOnUser`، `LastLoggedOnSamUser`، `LastLoggedOnUserSID`، `LastLoggedOnDisplayName`، `LogonUiLastWrittenUTC` | آخر مستخدم في شاشة تسجيل الدخول، وآخر كتابة لمفتاحه. |
| `MachineSID`، `DomainSID`، `MachineAccountSID` | معرّف SID للحاسوب، ومعرّف SID لمجاله (`PolPrDmS`) ومعرّف SID لحساب الحاسوب في المجال (`PolMachineAccountS`). |
| `CompNameLastWrittenUTC`، `PrimaryDomainLastWrittenUTC`، `VersionLastWrittenUTC` | تقريب لآخر وقت سُمّي فيه الحاسوب، وآخر انضمام لمجال أو مجموعة عمل، وآخر تحديث (مفتاح `CurrentVersion`). |
| `PreviousVersions` | كل مفتاح `Source OS`: الإصدار، وتاريخ تثبيته، والتاريخ الموجود في اسم المفتاح (JSON: كائن لكل منها). |
| `Profiles` | كل ملف تعريف: SID، والمجلد، ووقت المفتاح، وآخر تحميل وإلغاء تحميل (JSON: كائن لكل منها). |
| `TransactionLogStatus`، `LogsApplied` | سياق استعادة سجل Windows، عبر الخلايا الأربع كلها. |
| `Problems` | قيم من نوع أو حجم خاطئ (مع الاحتفاظ ببايتاتها)، و`SAM` و`SECURITY` يختلف فيهما معرّف SID للحاسوب. |

## استراتيجية التحقيق { #investigation-strategy }

1. اقرأ المنطقة الزمنية أولًا: دوّن `UtcOffset`، وتواريخ التوقيت الصيفي، وما إذا كانت المنطقة قد
   تغيرت مؤخرًا. وحوّل بها السجلات ذات التوقيت المحلي.
2. دوّن الإصدار والبنية وتاريخ التثبيت. وافحص `PreviousVersions` بحثًا عن الترقيات، وقارن
   تاريخ التثبيت بأقدم أوقات نظام الملفات وسجلات الأحداث.
3. دوّن `LastShutdownUTC`: فوجود نشاط بعده دون وقت إيقاف تشغيل جديد يشير إلى انهيار، أو انقطاع
   للطاقة، أو جمع حي.
4. استخدم `MachineSid` و`DomainSid` للتمييز بين الحسابات المحلية وحسابات المجال في معرّفات SID في أماكن أخرى (BAM،
   و`ProfileList`، وسجلات الأحداث)، وأوقات تحميل ملفات التعريف وإلغاء تحميلها لآخر عمليات تسجيل الدخول.
5. اربط أسماء الحاسوب والمجال بسجلات الأحداث، وبـ `CompNameLastWrittenUTC`
   و`PrimaryDomainLastWrittenUTC` لمعرفة عمليات إعادة التسمية والانضمام إلى المجالات.

## مزيد من التفاصيل { #further-details }

- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft: DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft: TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft: معرّفات الأمان](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb: المناطق الزمنية](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb: معلومات منتج Windows](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
