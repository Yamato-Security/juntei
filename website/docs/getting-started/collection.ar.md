# جمع الأدلة { #collecting-evidence }

يحلل Juntei الملفات التي جُمعت مسبقًا من Windows. استخدم أداة اقتناء موثوقة وحافظ على البيانات الوصفية الأصلية كلما أمكن.

[auto-parse --vhdx](../commands/auto-parse.md#vhdx-disk-images): Linux / macOS / Windows.

[auto-parse --e01](../commands/auto-parse.md#e01-and-ex01-disk-images): Linux / macOS / Windows.

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
| Internet Explorer وEdge القديم (WebCache) | `%LOCALAPPDATA%\Microsoft\Windows\WebCache\` |
| Outlook | `%LOCALAPPDATA%\Microsoft\Outlook\*.ost`، `%USERPROFILE%\Documents\Outlook Files\*.pst` ‏(`%LOCALAPPDATA%\Microsoft\Outlook\*.pst` قبل Outlook 2010)، `%APPDATA%\Microsoft\Templates\*.oft`، وملفات `.msg` أينما حُفظت |
| متصفحات الويب | `%LOCALAPPDATA%\Google\Chrome\User Data\`، و`%LOCALAPPDATA%\Microsoft\Edge\User Data\`، و`%APPDATA%\Mozilla\Firefox\Profiles\`، ومجلدات ملفات التعريف الخاصة بالمتصفحات الأخرى |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` ومواقع أخرى للمستخدم/النظام |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` و`CustomDestinations` |
| ذاكرة التخزين المؤقت لصور RDP النقطية | `%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\` (`%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\` في Windows XP والإصدارات الأقدم) |
| سلة المحذوفات | `$Recycle.Bin\<SID>\$I*` و`$R*` على كل وحدة تخزين (`RECYCLER\<SID>\INFO2` و`D*` في Windows XP والإصدارات الأقدم) |
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
- اجمع مجلد `WebCache` بأكمله: فـ Windows يُبقي `WebCacheV01.dat` مفتوحًا ما دام المستخدم مسجلًا دخوله،
  ولا توجد أحدث تغييراته إلا في سجلات المعاملات `V01*.log`، التي لا يعيد Juntei تشغيلها.
- اجمع ملفات OST وPST الخاصة بـ Outlook وOutlook مغلق، أو من صورة قرص أو نسخة ظل لوحدة التخزين
  (volume shadow copy): فـ Outlook يُبقيها مفتوحة ما دام يعمل. وابحث أيضًا عن ملفات PST خارج المجلدات
  الافتراضية: فالمستخدمون ينقلون الأرشيفات إلى مجلدات ومحركات أقراص ومشاركات ملفات أخرى.
- اجمع شجرة `Quarantine` الخاصة بـ Defender بأكملها حتى يمكن مطابقة Entries وResourceData وResources.

## تحذيرات للأنظمة الحية { #live-system-cautions }

قد تتغير الملفات المفتوحة أثناء النسخ. وقد تكون خلايا سجل Windows وقواعد بيانات ESE غير متسقة داخليًا.
وقد لا يفرّغ ShimCache أحدث إدخالاته الموجودة في الذاكرة حتى إيقاف التشغيل. ويُفضَّل استخدام أداة جمع جنائية
تستخدم لقطات وحدات التخزين أو الوصول الخام على النسخ العادي للملفات.

## السلامة { #safety }

قد تحتوي الأدلة على برمجيات خبيثة، وعينات استغلال، وملفات LNK ضارة، ونصوص صيغ جداول بيانات.
حلّل على نظام معزول، ولا تنفّذ الدفقات المستعادة، ولا تستخدم `--extract` إلا عندما
تعتزم التعامل مع محتوى خطير. واحتفظ بالمصدر الأصلي دون مساس بشكل منفصل.
