# توافق التطبيقات { #application-compatibility }

## ما الذي يحتفظ به Windows { #what-windows-keeps }

إلى جانب ShimCache (`parse-registry shimcache`) وAmcache (`parse-registry amcache`)، تحتفظ ميزات
توافق التطبيقات في Windows بما يلي تحت
`Microsoft\Windows NT\CurrentVersion\AppCompatFlags`:

| المصدر | الخلية والمفتاح | ما الذي يُظهره |
|---|---|---|
| `layers` | `SOFTWARE` (و`Wow6432Node`) و`NTUSER.DAT` `Layers` | البرامج المضبوطة للعمل بطبقات توافق: `RUNASADMIN` (دائمًا بصفة مسؤول)، `WIN7RTM`، `HIGHDPIAWARE`، … |
| `pca_store` | `NTUSER.DAT` `Compatibility Assistant\Store` (Windows 8 والإصدارات الأحدث) | البرامج التي رأى Program Compatibility Assistant المستخدمَ يشغّلها |
| `pca_persisted` | `NTUSER.DAT` `Compatibility Assistant\Persisted` (Windows 7) | الأمر نفسه، على Windows 7 |
| `cit_module` | `SOFTWARE` `CIT\Module\<module>` | البرامج التي حمّلت وحدة (module) متتبَّعة، ومتى فعلت ذلك آخر مرة |
| `cit_system` | `SOFTWARE` `CIT\System` (من Windows 7 حتى الإصدارات الأولى من Windows 10) | مسارات البرامج وأسطر الأوامر في قواعد بيانات CIT المضغوطة |
| `installed_sdb` | `SOFTWARE` `InstalledSDB\{GUID}` | قواعد بيانات shim المخصصة المثبتة باستخدام `sdbinst`: الملف، والوصف، ووقت التثبيت |
| `custom_sdb` | `SOFTWARE` `Custom\<program>` | قواعد بيانات shim المخصصة المطبقة على كل برنامج، مع أوقات تثبيتها |

<span class="evidence-strength">أفضل استنتاج:</span> الطبقات وقواعد بيانات shim تكوينٌ يغيّر طريقة
تشغيل البرنامج؛ أما إدخالات Program Compatibility Assistant وCIT فتُظهر أن برنامجًا قد عمل (أو حمّل وحدة)
بصفة هذا المستخدم أو على هذا النظام.

تُعد قواعد بيانات shim المخصصة تقنيةً للاستمرارية والحقن (MITRE ATT&CK T1546.011): إذ يمكن لـ shim أن
يحمّل ملف DLL داخل برنامج أو أن يبدأ برنامجًا آخر كلما عمل البرنامج.

## الموقع { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

اجمع `SOFTWARE`، وكل ملفات `NTUSER.DAT` وسجلات المعاملات الخاصة بها، وملفات `.sdb` التي يسمّيها مسار
قاعدة بيانات shim (عادةً `C:\Windows\AppPatch\Custom\`).

## تحذيرات للمحلل { #analyst-cautions }

- **توسيع CIT محدود.** القيم التي تعلن حجمًا غير مضغوط يتجاوز 64 MiB يُحتفظ بها
  بايتاتٍ خامًا مع رسالة تشخيصية. ويُبلَّغ عن الضغط المشوّه وحالات عدم تطابق الحجم.


- **لا يوجد وقت تشغيل لإدخالات Program Compatibility Assistant.** تحتوي بيانات `Store` (‏`SACP`،
  المحفوظة في `DataRaw`) على وقت واحد، لكنه في Windows 8.1 و10 و11 هو نفسه لكل برامج الخلية، وكذلك
  في الأنظمة الأخرى: إنه ليس وقت تشغيل البرنامج. ولا يؤرّخ أحدث تغيير إلا وقت آخر كتابة لمفتاح `Store`.
- **البرامج الموجودة على وسائط قابلة للإزالة** تُسمّى `SIGN.MEDIA=<id> <path>`: ‏`Path` هو المسار على
  الوسيط، و`Details` يحتوي على معرّف الوسيط.
- **قواعد بيانات CIT** مضغوطة بخوارزمية LZNT1 وبنيتها غير موثقة علنًا: يسرد Juntei مسارات البرامج
  وأسطر الأوامر الموجودة في كل منها، مرة واحدة لكل قاعدة بيانات، دون أعداد أو أوقات. وسطر الأوامر سجلٌ
  مستقل بذاته، غير مرتبط ببرنامجه. ولا يحتفظ Windows 10 والإصدارات الأحدث إلا بالقليل هنا.
- **قواعد بيانات shim.** افحص كل سجل `custom_sdb` و`installed_sdb`: نادرًا ما تثبّت البرمجيات المشروعة
  واحدة منها. ويوضح ملف `.sdb` نفسه ما يفعله shim.
- **الطبقات** التي يضبطها مستخدم لا تنطبق إلا على ذلك المستخدم؛ أما طبقات `SOFTWARE` فتنطبق على الجميع.
  ويجعل `RUNASADMIN` البرنامج يطلب رفع الامتيازات في كل مرة.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry app-compat <INPUT> [OPTIONS]

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
juntei parse-registry app-compat -d evidence/ -o appcompat.csv
juntei parse-registry app-compat -d evidence/ -t jsonl \
  | jq 'select(.Source == "installed_sdb" or .Source == "custom_sdb")'
```

يسرد الملخص قواعد بيانات shim المخصصة، والبرامج ذات الطبقات، وآخر استخدامات وحدات CIT.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `SourcePath`، `HiveType` | المضيف، وملف الخلية، و`SOFTWARE` أو `NTUSER.DAT`. |
| `User`، `UserSID` | اسم مجلد ملف التعريف لخلية المستخدم، ومعرّف SID الخاص به من `ProfileList` في النظام. |
| `Source`، `Scope` | `layers`، `pca_store`، `pca_persisted`، `cit_module`، `cit_system`، `installed_sdb`، `custom_sdb`؛ `machine` أو `user`. |
| `Path` | البرنامج (اسم القيمة)، أو مسار أو سطر أوامر في قاعدة بيانات CIT، أو البرنامج الذي تنطبق عليه قاعدة بيانات shim. |
| `Layers` | طبقات التوافق كما هي مكتوبة (`~ RUNASADMIN`). |
| `LastSeenUTC` | آخر مرة حمّل فيها برنامج وحدة CIT. |
| `SdbGUID`، `SdbPath`، `SdbInstalledUTC` | معرّف GUID لقاعدة بيانات shim، وملفها، ووقت تثبيتها. |
| `Details` | اسم وحدة CIT، أو اسم قيمة قاعدة بيانات CIT، أو معرّف الوسيط القابل للإزالة لبرنامج (`SignMedia`)، أو `DatabaseDescription` و`DatabaseType` لقاعدة بيانات shim. |
| `KeyPath`، `KeyLastWrittenUTC`، `ValueName`، `WOW64` | مصدر السجل، ووقت آخر تغيير لذلك المفتاح، وما إذا كان تحت `Wow6432Node`. |
| `DataRaw` | بيانات `SACP` لقيمة `Store`، أو البيانات التي ليست على النحو المتوقع (بالنظام الست عشري). |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows. |
| `Problems` | البيانات التي تعذّر فك ترميزها أو قراءتها. |

## استراتيجية التحقيق { #investigation-strategy }

1. راجع كل قاعدة بيانات shim مخصصة: ملف `.sdb` الخاص بها، ووقت تثبيتها، والبرامج التي تنطبق عليها؛
   واربطها بـ `sdbinst.exe` في Prefetch وAmcache وأحداث إنشاء العمليات.
2. ابحث عن البرامج الموجودة في مجلدات قابلة للكتابة من قِبل المستخدم ضمن إدخالات Program Compatibility
   Assistant، وعن البرامج الواردة من وسائط قابلة للإزالة.
3. افحص `RUNASADMIN` والطبقات الأخرى على البرامج غير المعتادة.
4. اربط النتائج بـ ShimCache وAmcache وBAM وPrefetch وUserAssist لتحديد أوقات التنفيذ.

## مزيد من التفاصيل { #further-details }

- [Microsoft: قاعدة بيانات توافق التطبيقات](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011: Application Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft: MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
