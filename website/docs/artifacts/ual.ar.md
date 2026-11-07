# تسجيل وصول المستخدمين (UAL) { #user-access-logging-ual }

## ما هو UAL { #what-ual-is }

تسجيل وصول المستخدمين (User Access Logging أو UAL) ميزة في Windows Server، مثبّتة ومفعّلة افتراضيًا منذ
Windows Server 2012، تجمّع في قواعد بيانات ESE الطلبات التي يرسلها العملاء إلى أدوار الخادم.
ولكل دور وعنوان عميل وحساب، تحتفظ بوقتَي أول ظهور وآخر ظهور للعميل خلال السنة، والعدد الإجمالي
للطلبات، وعددًا لكل يوم من أيام السنة. ومن هذه الأدوار File Server ‏(SMB)، وActive Directory Domain Services،
وActive Directory Certificate Services، وDHCP Server، وPrint and Document Services، وRemote Access،
وWeb Server ‏(IIS)، وWindows Server Update Services.

يُعد UAL قيّمًا في تحقيقات الحركة الجانبية: إذ تُظهر صفوف العملاء فيه العناوين والحسابات التي استخدمت
خادمًا ما، وفي أي أيام، وغالبًا بعد وقت طويل من التفاف سجلات الأحداث والكتابة فوقها.

<span class="evidence-strength">أفضل استنتاج:</span> استخدم عميلٌ على العنوان المسجل، بعد مصادقته
بالحساب المسجل، الدورَ في كل يوم له عدد.

## الموقع والجمع { #location-and-collection }

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

يحتوي `Current.mdb` على أحدث نشاط. ويصف `SystemIdentity.mdb` الخادم، ويسمّي الأدوار، ويسرد
ملف `{GUID}.mdb` الذي يحتفظ بكل سنة (`CHAINED_DATABASES`). وتُبقي خدمة UAL قواعد البيانات مفتوحة:
لذا اجمع مجلد `Sum` بأكمله باستخدام أداة جمع جنائية أو من لقطة لوحدة التخزين.

!!! warning "قواعد البيانات غير المتسقة"
    غالبًا ما يكون `Current.mdb` غير متسق (dirty) عند جمعه من خادم قيد التشغيل. ويقرأه Juntei كما جُمع
    ويُبلغ عن ذلك، لكنه لا يعيد تشغيل `Svc*.log`: لذا يكون النشاط الموجود في سجلات المعاملات فقط
    مفقودًا.

## تحذيرات للمحلل { #analyst-cautions }

- يعدّ UAL الطلبات يوميًا؛ ولا يسجل كل طلب على حدة. والوقتان الوحيدان في صف العميل هما
  `InsertDate` و`LastAccess`.
- أيام `DailyAccesses` أيام بتوقيت UTC.
- `Address` هو العنوان الذي رآه الخادم: فقد تقف بوابة NAT أو وكيل (proxy) أو VPN بينه وبين
  العميل الحقيقي. ويمثل `::1` و`127.0.0.1` الخادم نفسه.
- حلّل كل قاعدة بيانات في مجلد `Sum`: فقد تكون للعميل نفسه صفوف في `Current.mdb` وفي
  `{GUID}.mdb` الخاص بسنته.
- لا يثبت غياب الصفوف أن أحدًا لم يتصل: إذ يمكن إيقاف UAL (عبر `Disable-Ual`، أو بإيقاف
  خدمة User Access Logging Service)، كما تفتقد قاعدة البيانات غير المتسقة أحدث تغييراتها.
- من دون `SystemIdentity.mdb`، يأتي `RoleName` من الأسماء التي يعطيها Windows Server لأدواره.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese ual <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Current.mdb, {GUID}.mdb and SystemIdentity.mdb
  -f, --file <FILE>      Parse one UAL database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese ual -d Sum -o ual-csv
juntei parse-ese ual -d Sum -t jsonl \
  | jq 'select(.Table == "CLIENTS") | {Address, AuthenticatedUserName, RoleName, LastAccess}'
```

## جداول الإخراج { #output-tables }

ينشئ CSV ملفًا واحدًا لكل جدول، لا يحتوي إلا على صف الترويسة عندما يكون الجدول فارغًا. وتحمل صفوف JSON وJSONL
الحقل `SourceFile` (قاعدة البيانات) والحقل `Table` (اسم الجدول في قاعدة البيانات).

| ملف CSV | الجدول | المحتوى |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | عملاء كل دور: العنوان، والحساب، وأول وآخر وصول خلال السنة، وأعداد الطلبات الإجمالية واليومية. |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | وقت أول استخدام وآخر استخدام لكل دور. |
| `UAL-DNS.csv` | `DNS` | أزواج العناوين وأسماء المضيفين، مع وقت آخر ظهور لكل منها. |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | الأجهزة الافتراضية في Hyper-V: المعرّفات، والرقم التسلسلي، ووقتا الإنشاء وآخر نشاط. |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | الخادم: الاسم، والمجال، وإصدار Windows، ونوع المنتج، والعتاد، والمنطقة الزمنية، وآخر إقلاع؛ وقد يكشف صف جديد عن إعادة تسمية، أو انضمام إلى مجال، أو ترقية إلى وحدة تحكم بالمجال. |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | ملف `{GUID}.mdb` لكل سنة. |
| `UAL-Roles.csv` | `ROLE_IDS` | معرّفات GUID للأدوار مع أسمائها. |
| `UAL-<table>.csv` | جدول آخر | كل أعمدة جدول لا يعرفه Juntei. |

## الحقول { #fields }

| الحقل | التفسير |
|---|---|
| `SourceFile` | قاعدة البيانات التي قُرئ منها الصف. |
| `RoleGUID`، `RoleName` | الدور، واسمه من `SystemIdentity.mdb` المجاور لقاعدة البيانات (والأسماء المضمّنة عند غيابه). |
| `TenantID` | معرّف GUID للمستأجر (tenant) الخاص بالعميل، عندما يُبلغ الدور عنه. |
| `TotalAccesses` | طلبات العميل في سنة قاعدة البيانات؛ وهو مجموع `DailyAccesses`. |
| `InsertDate` | وقت إنشاء الصف: أول طلب للعميل خلال السنة في قاعدة البيانات هذه (UTC). |
| `LastAccess` | أحدث طلب للعميل (UTC). |
| `Address` | عنوان IPv4 أو IPv6 الخاص بالعميل. |
| `AuthenticatedUserName` | الحساب الذي صادق به العميل (`DOMAIN\user`؛ وتنتهي حسابات الحواسيب بـ `$`). |
| `ClientName` | اسم العميل، عندما يُبلغ الدور عنه. |
| `DailyAccesses` | الأيام التي شهدت طلبات وأعدادها، من الأعمدة `Day1` … `Day366`، مؤرّخة في سنة `LastAccess`: ‏`2023-01-12: 5, 2023-01-13: 2` في CSV، وكائنات تحتوي على `Date` و`Accesses` في JSON. ويحمل اليوم الذي سنته غير معروفة `DayOfYear` بدلًا من `Date`. |
| `FirstSeen`، `LastSeen` | أول وآخر استخدام لدور (`ROLE_ACCESS`)، وآخر ظهور لزوج DNS ‏(`DNS`)، بتوقيت UTC. |
| `CreationTime`، `LastSeenActive` | وقت إنشاء جهاز افتراضي أو صف `SYSTEM_IDENTITY`، ووقت آخر نشاط لجهاز افتراضي، بتوقيت UTC. |
| `OS-ProductType`، `OS-ProductTypeName` | 1 `Workstation`، 2 `Domain Controller`، 3 `Server`. |
| `OS-CurrentTimeZone` | فرق توقيت الخادم عن UTC بالدقائق (`-420` يعني UTC-07:00). |
| `OS-LastBootUpTime` | آخر إقلاع للخادم بتوقيت UTC (أو نص WMI كما خُزّن عندما يتعذر قراءته). |
| أعمدة أخرى | كل عمود آخر في الجدول تحت تسميات الحقول بصيغة PascalCase الخاصة بـ Juntei، مثل `SystemDNS-HostName` و`SystemDomainName` و`OS-BuildNumber` و`SystemSerialNumber`. |

## التحليل التلقائي { #automatic-parsing }

يكتب `auto-parse` الجداول في `LateralMovement` بأسماء مثل `UAL-Clients.csv` و`UAL-RoleAccess.csv`
وغيرها (أو ملفًا واحدًا هو `UAL.json` أو `UAL.jsonl` مع `-t json` أو `-t jsonl`)، مع
`LateralMovement/UAL-logs.txt`. ويتضمن الجدول الزمني المدمج `-j` كل صف، مع `Comp` ومعرّف
`Artifact` ذي القيمة `ual`. ويحوّل `-Z` الحقول `InsertDate` و`LastAccess` و`FirstSeen` و`LastSeen`
و`CreationTime` و`LastSeenActive` و`OS-LastBootUpTime`.
