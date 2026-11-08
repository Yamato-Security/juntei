# جمع الأدلة { #collecting-evidence }

يحلل Juntei الملفات التي جُمعت مسبقًا من Windows. استخدم أداة اقتناء موثوقة وحافظ على البيانات الوصفية الأصلية كلما أمكن. يمكن تحليل صور أقراص VHDX دون استخراج ملفاتها: راجع [تحميل صور الأقراص](../commands/mount.md).

## المواقع الشائعة { #common-locations }

| الأثر الرقمي | الموقع المعتاد |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | ملف البيانات الوصفية لوحدة تخزين NTFS ‏`C:\$MFT` |
| `$LogFile` | ملف البيانات الوصفية لوحدة تخزين NTFS ‏`C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| دفتر يومية USN | دفق البيانات الوصفية لـ NTFS ‏`C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| تسجيل وصول المستخدمين (Windows Server) | `C:\Windows\System32\LogFiles\Sum\` |
| متصفحات الويب | `%LOCALAPPDATA%\Google\Chrome\User Data\`، و`%LOCALAPPDATA%\Microsoft\Edge\User Data\`، و`%APPDATA%\Mozilla\Firefox\Profiles\`، ومجلدات ملفات التعريف الخاصة بالمتصفحات الأخرى |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` ومواقع أخرى للمستخدم/النظام |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` و`CustomDestinations` |
| ShimCache | خلية `SYSTEM`: ‏`...\Session Manager\AppCompatCache\AppCompatCache` |
| حجر Defender | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| سجل Windows للجهاز | `C:\Windows\System32\config\SYSTEM`، `SOFTWARE`، `SAM`، `SECURITY` |
| سجل Windows للمستخدم | `%USERPROFILE%\NTUSER.DAT`، `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## الملفات التي يجب جمعها معًا { #files-that-must-travel-together }

- اجمع خلايا سجل Windows مع ملفات `.LOG` و`.LOG1` و`.LOG2` المجاورة. ويمكن لـ Juntei إعادة تشغيل السجلات
  لأوامر سجل Windows ولـ ShimCache عندما تكون الخلية غير متسقة (dirty).
- اجمع `SRUDB.dat` الخاص بـ SRUM بشكل متسق. ويُبلغ Juntei حاليًا عن قاعدة بيانات ESE غير المتسقة لكنه
  لا يعيد تشغيل `SRU*.log`؛ لذا تكون السجلات الموجودة في تلك السجلات فقط غير متاحة.
- اجمع `$MFT` بجانب `$Extend/$J` عندما يكون ذلك ممكنًا. فسجل USN يخزن مراجع ملفات الأب،
  لا مسارًا كاملًا؛ ويحسّن MFT حل المسارات.
- اجمع `$LogFile` مع `$MFT` لوحدة التخزين نفسها في أقرب وقت ممكن. فـ `$LogFile` يلتف
  بسرعة؛ وقد يؤدي الجمع اللاحق إلى فقدان سجلات الإعادة/التراجع (redo/undo) الأقدم بالكامل.
- اجمع SRUM مع خلية `SOFTWARE` المطابقة لحل أسماء الحسابات وملفات تعريف Wi-Fi.
- اجمع مجلد `Sum` الخاص بتسجيل وصول المستخدمين بأكمله: فـ `SystemIdentity.mdb` يسمّي الأدوار
  وقواعد بيانات `{GUID}.mdb` السنوية. ولا يعيد Juntei كذلك تشغيل سجلات المعاملات `Svc*.log` الخاصة به.
- اجمع قواعد بيانات المتصفحات مع ملفات `-wal` المجاورة لها (`places.sqlite-wal`،
  `cookies.sqlite-wal`): فأحدث التغييرات، وإصدارات الصفحات الأقدم التي تُستعاد منها الإدخالات المحذوفة،
  لا توجد هناك إلا إلى أن ينفذ المتصفح نقطة تحقق (checkpoint).
- اجمع شجرة `Quarantine` الخاصة بـ Defender بأكملها حتى يمكن مطابقة Entries وResourceData وResources.

## تحذيرات للأنظمة الحية { #live-system-cautions }

قد تتغير الملفات المفتوحة أثناء النسخ. وقد تكون خلايا سجل Windows وقواعد بيانات ESE غير متسقة داخليًا.
وقد لا يفرّغ ShimCache أحدث إدخالاته الموجودة في الذاكرة حتى إيقاف التشغيل. ويُفضَّل استخدام أداة جمع جنائية
تستخدم لقطات وحدات التخزين أو الوصول الخام على النسخ العادي للملفات.

## السلامة { #safety }

قد تحتوي الأدلة على برمجيات خبيثة، وعينات استغلال، وملفات LNK ضارة، ونصوص صيغ جداول بيانات.
حلّل على نظام معزول، ولا تنفّذ الدفقات المستعادة، ولا تستخدم `--extract` إلا عندما
تعتزم التعامل مع محتوى خطير. واحتفظ بالمصدر الأصلي دون مساس بشكل منفصل.
