# أجهزة USB { #usb-devices }

## ما هو سجل USB { #what-the-usb-history-is }

يسجل Windows كل جهاز يثبّته. فبالنسبة لأجهزة تخزين USB، وأجهزة USB الأخرى، والأجهزة المحمولة
(الهواتف، والكاميرات، ومشغلات الوسائط)، تحتفظ خلية `SYSTEM` بمفتاح لكل جهاز يتضمن اسمه، ورقمه
التسلسلي، والأوقات التي ثبّته فيها Windows، وآخر مرة رصد فيها وصوله، وآخر مرة رصد فيها إزالته.
ويسجل `MountedDevices` معرّفات GUID لوحدات التخزين وأحرف محركات الأقراص المعطاة لوحدات تخزينها. وتحتفظ
خلية `SOFTWARE` بالأسماء التي أعطاها Explorer للأجهزة المحمولة ووحدات التخزين، وبسجل ReadyBoost للأرقام
التسلسلية لوحدات التخزين. ويسجل ملف `NTUSER.DAT` الخاص بكل مستخدم وحدات التخزين التي كانت مركّبة لدى
ذلك المستخدم.

يدمج `parse-registry usb` هذه المصادر في سجل واحد لكل جهاز ووحدة تخزين: ما الجهاز، ومتى ثُبّت ووُصل،
وأي حرف محرك أقراص ووحدة تخزين كانا له، وأي المستخدمين كانت وحدة التخزين تلك مركّبة لديهم.

<span class="evidence-strength">أفضل استنتاج:</span> وُصل جهاز بهذا المورّد والمنتج والرقم التسلسلي
بهذا النظام، أول مرة وآخر مرة في هذه الأوقات؛ وكان لوحدة تخزينه هذا الحرف؛ وكانت مركّبة لدى هؤلاء
المستخدمين. ولا تُظهر سجلات سجل Windows الملفات التي نُسخت.

## الموقع { #location }

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| خاصية الجهاز | الحقل | المعنى (`devpkey.h`) |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate`: وقت آخر تثبيت لبرنامج تشغيله |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate`: وقت أول تثبيت للجهاز |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate`: وقت آخر توصيل له (Windows 8 والإصدارات الأحدث) |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate`: وقت آخر إزالة له (Windows 8 والإصدارات الأحدث) |

يحتفظ Windows 8 والإصدارات الأحدث بكل وقت بوصفه القيمة الافتراضية للمفتاح `0064` … `0067`. ويحتفظ
Windows 7 به في `00000064\00000000\Data`. ويقرأ Juntei كلتا الصيغتين.

تسمّي كثير من مراجع DFIR الخاصية `0064` بأنها التثبيت الأول و`0065` بأنها التثبيت. أما Juntei فيتبع
تعريفات `devpkey.h` من Microsoft الواردة أعلاه، ويسمّي الأعمدة بحسب ما يسجله كل وقت.

مرّر خلية `SYSTEM` إلى `-f` أو `-d`. ويُعثر على خلية `SOFTWARE` الخاصة بالنظام وخلايا `NTUSER.DAT`
الخاصة بمستخدميه بجوارها، كما يحفظها الجمع (`Windows\System32\config\SOFTWARE`،
`Users\<user>\NTUSER.DAT`)، أو في المجلد نفسه وعلى مستوى واحد أدناه عندما تكون الخلايا قد صُدّرت معًا.
اجمعها كلها مع سجلات المعاملات الخاصة بها.

## كيف تُدمج السجلات { #how-the-records-are-joined }

- **الأجهزة:** سجل واحد لكل مفتاح جهاز: `usbstor` و`usb` و`wpdbusenum` (`DeviceClass`). ويأخذ جهاز
  تخزين USB معرّفي المورّد والمنتج من جهاز USB الذي يحمل معرّف الحاوية (container ID) نفسه (وإلا
  فالرقم التسلسلي نفسه).
- **وحدات التخزين:** وحدات تخزين جهاز تخزين USB هي قيم `\??\Volume{GUID}` في `MountedDevices` التي
  تكون بياناتها مسارًا يسمّي الجهاز. والجهاز الذي له وحدتا تخزين له سجلان. وأحرف محركات أقراصه هي قيم
  `\DosDevices\X:` التي تحمل البيانات نفسها.
- **الأسماء والأرقام التسلسلية:** تأتي أسماء وحدات التخزين من `Windows Portable Devices` ومن الأجهزة
  المحمولة التي تسمّي الجهاز نفسه. وتأتي تسميات وحدات التخزين وأرقامها التسلسلية من مفاتيح `EMDMgmt`
  التي تسمّيه.
- **المستخدمون:** المستخدمون الذين يحتوي `MountPoints2` لديهم على معرّف GUID لوحدة التخزين، مع وقت آخر
  كتابة لذلك المفتاح.
- **وحدات التخزين التي ليس لها مفتاح جهاز** (`volume`): وحدة تخزين في `MountedDevices` لا يسمّيها أي
  مفتاح جهاز (لقرص داخلي، أو لجهاز أُزيل وزال مفتاح `Enum` الخاص به)، أو حرف محرك أقراص لا تشاركه
  البيانات أي قيمة GUID لوحدة تخزين، أو وحدة تخزين لا توجد إلا في `MountPoints2` لدى المستخدمين.
- **الأجهزة التي لا يسمّيها إلا `SOFTWARE`** (`portable_device`، `readyboost`).

يسرد `SourceKeys` كل مفتاح وقيمة دُمج منها السجل، مع الخلية الخاصة بكل منها.

## تحذيرات للمحلل { #analyst-cautions }

- **يُعاد استخدام أحرف محركات الأقراص.** يحتفظ `MountedDevices` بآخر جهاز أُعطي كل حرف. ولا يُعرض الحرف
  إلا على الجهاز الذي لا تزال بياناته هي بيانات ذلك الحرف.
- **يمكن اختلاق الأرقام التسلسلية.** عندما يكون الحرف الثاني من المثيل (instance) هو `&`، فإن Windows
  اختلقه لأن الجهاز لا يُبلغ عن رقم تسلسلي؛ ويكون `Serial` فارغًا حينئذٍ. وقد يختلف مثل هذا المثيل على
  حاسوب آخر.
- **تنتهي مثيلات تخزين USB بالوحدة المنطقية** (`&0`). ويُسقطها `Serial`؛ بينما يحتفظ بها `Instance`.
- **تختلف الأوقات باختلاف إصدار Windows.** لا يملك Windows 7 سوى أوقات التثبيت. ويمثّل `LastArrivalUTC`
  و`LastRemovalUTC` آخر اتصال فقط، لا كل اتصال. وتوفر سجلات الأحداث
  (`Microsoft-Windows-Partition/Diagnostic` 1006، وKernel-PnP، و`DriverFrameworks-UserMode`)
  و`setupapi.dev.log` المزيد.
- **مجموعات التحكم:** افتراضيًا تُقرأ مجموعة التحكم التي استخدمها Windows آخر مرة. ويقرأ
  `--all-control-sets` كل مجموعة تحكم؛ فقد تظل مجموعة أقدم تسرد جهازًا لم يعد في المجموعة الحالية.
- **وقت آخر كتابة للمفتاح ليس وقت اتصال.** يعيد Windows كتابة مفاتيح الأجهزة لأسباب كثيرة.
- **الأجهزة المُزالة تترك آثارًا.** سجل وحدة التخزين الذي يحمل مورّدًا ومنتجًا دون مفتاح جهاز هو جهاز
  أُزيل مفتاح `Enum` الخاص به (بإلغاء التثبيت أو بأدوات التنظيف) بينما لا يزال `MountedDevices` يحتفظ
  بوحدة تخزينه.
- **الأرقام التسلسلية في ReadyBoost** هي الرقم التسلسلي لوحدة التخزين (`VolumeSerials`، كما يعرضه
  `dir`)، الذي تسجله ملفات LNK وJump Lists أيضًا.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.DeviceClass == "usbstor") | {Vendor, Product, Serial, LastArrival, DriveLetters, Users}'
```

يسرد الملخص أجهزة تخزين USB التي وصلت أخيرًا (الأحدث أولًا)، مع أحرف محركات أقراصها، وعدد وحدات التخزين
التابعة لأجهزة تخزين USB التي ليس لها مفتاح جهاز.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host` | اسم الحاسوب، من `SYSTEM`. |
| `DeviceClass` | `usbstor` أو `usb` أو `wpdbusenum`، أو، دون مفتاح جهاز، `volume` أو `portable_device` أو `readyboost`. |
| `ControlSet` | مجموعة التحكم الخاصة بمفتاح الجهاز. |
| `Vendor`، `Product`، `Revision` | تخزين USB: من اسم الجهاز (`Disk&Ven_…&Prod_…&Rev_…`)، وكذلك لجهاز يسمّيه مفتاح وحدة تخزين أو مفتاح في `SOFTWARE`. |
| `Vid`، `PID` | معرّفا المورّد والمنتج في USB. |
| `Serial`، `Instance` | الرقم التسلسلي، واسم مفتاح المثيل كما كُتب. |
| `FriendlyName`، `DeviceDesc`، `Service`، `ContainerID` | قيم مفتاح الجهاز، كما كُتبت (غالبًا ما يكون `DeviceDesc` بالصيغة `@<inf>,%…%;<text>`). ويكون `FriendlyName` للجهاز المحمول عادةً تسمية وحدة تخزينه أو محرك الأقراص. |
| `DeviceFirstInstallUTC`، `DriverLastInstalledUTC`، `LastArrivalUTC`، `LastRemovalUTC` | أوقات خصائص الجهاز (انظر أعلاه). |
| `DriveLetters` | أحرف محركات الأقراص التي تكون بياناتها في `MountedDevices` هي بيانات وحدة التخزين. |
| `VolumeGUID`، `VolumeData` | معرّف GUID لوحدة التخزين، وما تقوله بياناتها في `MountedDevices` عن موضعها: مسار جهاز، أو توقيع قرص MBR وإزاحة القسم، أو قسم GPT. |
| `VolumeNames`، `VolumeSerials` | الأسماء التي أعطاها Explorer للجهاز أو لوحدة تخزينه، وتسميات ReadyBoost، والأرقام التسلسلية لوحدات التخزين (`1234-ABCD`). |
| `User` | المستخدمون الذين يحتوي `MountPoints2` لديهم على وحدة التخزين، مع وقت آخر كتابة لذلك المفتاح (JSON: ‏`User`، `MountPoints2LastWritten`، `SourcePath`). |
| `FoundIn`، `SourceKeys` | مكان العثور عليه (`Enum`، `MountedDevices`، `Windows Portable Devices`، `EMDMgmt`، `MountPoints2`)، وكل مفتاح وقيمة دُمج منها. |
| `KeyPath`، `KeyLastWrittenUTC` | مفتاح مثيل الجهاز ووقت آخر كتابة له. |
| `SourcePath`، `SoftwarePath` | خلية `SYSTEM`، وخلية `SOFTWARE` عندما تكون قد أسهمت بشيء. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows للخلايا التي دُمج منها السجل. |
| `Problems` | مفاتيح تعذرت قراءتها، وخصائص أجهزة ليست من نوع FILETIME (مع بايتاتها). |

## استراتيجية التحقيق { #investigation-strategy }

1. رتّب أجهزة تخزين USB حسب `LastArrivalUTC` (وإلا فحسب `DriverLastInstalledUTC`)، واستبعد قارئات
   الحاسوب نفسه وموزعاته (hubs).
2. دوّن المورّد والمنتج والرقم التسلسلي ومعرّف GUID لوحدة التخزين وحرف محرك الأقراص والرقم التسلسلي
   لوحدة التخزين لكل جهاز، وابحث عنها في ملفات LNK وJump Lists وShellBags و`$MFT` (المسارات على ذلك
   الحرف أو وحدة التخزين).
3. افحص `User` لمعرفة من كانت وحدة التخزين مركّبة لديه، ومتى (آخر كتابة لمفتاح `MountPoints2`).
4. ابحث عن سجلات `volume` التي تسمّي جهاز تخزين USB: أجهزة أُزيلت مفاتيحها.
5. اربط النتائج مع سجلات الأحداث لكل اتصال، لا لآخر اتصال فقط.
6. استخدم `parse-registry read --recover-deleted` لمفاتيح الأجهزة المحذوفة.

## مزيد من التفاصيل { #further-details }

- [Microsoft: خصائص الأجهزة (devpkey.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft: دعم طلبات مدير التركيب](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h` (كما في ترويسات Wine)](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h): أرقام خصائص الأجهزة
- [libyal winreg-kb: USB storage](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md)،
  و[Mounted devices](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md)
  و[Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

تستخدم عمليات دمج المستخدمين هوية المصدر لكل خلية، لذا تبقى الخلايا المصدّرة التي تحمل اسم ملف التعريف
نفسه منفصلة. ويحتفظ `SourceKeys` بمسارات خلايا المستخدمين المساهمة. ويتخطى اكتشاف المستخدمين المرافقين
الارتباطات الرمزية، اتساقًا مع الاكتشاف التكراري للخلايا. ويتطلب ربط VID/PID معرّف حاوية مطابقًا وقابلًا
للاستخدام؛ ودونه، يجب أن تُنتج مطابقة الرقم التسلسلي معرّفات غير ملتبسة. وتبقى المرشحات المتعارضة غير
معروفة مع تسجيل مشكلة.
