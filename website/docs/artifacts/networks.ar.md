# الشبكات { #networks }

## ما هي { #what-they-are }

يتذكر Windows الشبكات التي انضم إليها النظام وكيفية إعداد شبكته.

- **`SOFTWARE`:**
  - **ملفات تعريف الشبكات** (`NetworkList\Profiles`): ملف لكل شبكة اتصل بها النظام،
    مع اسمها (SSID لشبكة Wi-Fi)، وفئتها (عامة، خاصة، مجال)، ونوع الاتصال
    (سلكي، لاسلكي، VPN، نطاق عريض للجوال)، ووقت أول وآخر اتصال للنظام بها.
  - **التواقيع** (`NetworkList\Signatures`): ما يعرّف كل شبكة، مثل
    عنوان MAC الخاص بالبوابة ولاحقة DNS.
  - **بطاقات الشبكة** (`NetworkCards`): المحوّلات.
- **`SYSTEM`** (لكل مجموعة تحكم):
  - **واجهات TCP/IP:** العناوين، وخادم DHCP وعقد الإيجار، وخوادم DNS، والمجال.
  - **المشاركات** التي يتيحها النظام (`LanmanServer\Shares`).
  - **وكلاء المنافذ** (`PortProxy`): الأنفاق التي يُعدّها `netsh interface portproxy`، والتي
    يستخدمها المهاجمون للتنقل المحوري (pivot) وللوصول إلى RDP عبر منفذ آخر.
  - **المسارات الدائمة.**
  - **المنطقة الزمنية**، التي تحوّل الأوقات المحلية لملفات التعريف إلى UTC.

<span class="evidence-strength">أفضل استنتاج:</span> اتصل النظام بشبكة تحمل
هذا الاسم وهذه البوابة، أول مرة وآخر مرة في هذين الوقتين. وكان لديه هذا العنوان من خادم DHCP هذا،
وكان يتيح هذه المشاركات والأنفاق عند حفظ الخلية. ولا يحدد اسم الشبكة وحده
مكانها: فكثير من الشبكات تُسمى "Network" أو تتشارك SSID.

## الموقع { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\NetworkList\Profiles\{GUID}
      ProfileName, Description, Category, NameType, Managed, DateCreated, DateLastConnected
  Microsoft\Windows NT\CurrentVersion\NetworkList\Signatures\Managed|Unmanaged\<signature>
      ProfileGuid, DefaultGatewayMac, DnsSuffix, FirstNetwork
  Microsoft\Windows NT\CurrentVersion\NetworkCards\<n>          ServiceName, Description
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Services\Tcpip\Parameters\Interfaces\{GUID}                    IP, DHCP, DNS, lease times
  Services\LanmanServer\Shares                                   one value per share
  Services\PortProxy\v4tov4|v4tov6|v6tov4|v6tov6\tcp             listen = connect
  Services\Tcpip\Parameters\PersistentRoutes                     destination,mask,gateway,metric
  Control\TimeZoneInformation
```

اجمع الخليتين مع سجلات المعاملات الخاصة بهما. وخلية SYSTEM مطلوبة لتحويل أوقات ملفات
التعريف إلى UTC.

## تحذيرات للمحلل { #analyst-cautions }

- **أوقات ملفات التعريف محلية.** يُكتب `DateCreated` و`DateLastConnected` بالتوقيت
  المحلي للنظام.
  - يحتفظ `FirstConnectedLocal` و`LastConnectedLocal` بهما كما وُجدا.
  - يحوّلهما `FirstConnectedUTC` و`LastConnectedUTC` باستخدام المنطقة الزمنية الموجودة في خلية
    SYSTEM للنظام (`TimeZone`) وقاعدة التوقيت الصيفي الخاصة بها.
  - الوقت السابق لتغيير المنطقة الزمنية، أو من سنة ذات قاعدة توقيت صيفي
    أخرى (التواريخ الأمريكية قبل 2007)، قد ينحرف بمقدار الفرق.
  - من دون خلية SYSTEM قابلة للقراءة ومجموعة تحكم حالية صالحة، لا تُعطى إلا الأوقات المحلية. ولا توجد لفجوات التوقيت الصيفي والساعات المكررة الملتبسة طوابع زمنية UTC وحيدة مثبتة: يُترك UTC فارغًا ويشرح `Problems` هذا القيد. كما تبقى الإزاحات أو التواريخ غير الصالحة دون تحويل.
- **الأول والأخير فقط.** لكل ملف تعريف اتصال أول واحد واتصال أخير واحد، لا كل
  اتصال. وعادةً ما يطابق آخر كتابة لمفتاح ملف التعريف آخر اتصال.
- **الأسماء تتكرر.** يسمّي Windows الشبكات السلكية غير المُدارة "Network" و"Network 2" وهكذا.
  ومعرّفات SSID لشبكات Wi-Fi ليست فريدة. استخدم `GatewayMac` و`DnsSuffix` للتمييز بين الشبكات، و
  اربطها بسجلات DHCP وجرد الأصول. وعنوان MAC للبوابة ليس بالضرورة BSSID لنقطة وصول Wi-Fi، ولا يثبت وحده الموقع المادي.
- **المشاركات والوكلاء والمسارات** إعدادات: فقد تُعدّ مشاركة أو وكيل منفذ دون أن
  يُستخدم أبدًا. وليس لكل قيمة وقت خاص بها؛ وآخر كتابة للمفتاح هي وقت تغيّر أي منها.
- **الواجهات:** مفتاح الواجهة تحت GUID بلا عناوين هو محوّل غير
  مُعدّ. ومع DHCP، تكون العناوين هي عناوين آخر عقد إيجار. ولا يُعرض العنوان الثابت المتبقي من قبل DHCP
  على أنه قيد الاستخدام.
- **مجموعات التحكم:** افتراضيًا تُقرأ مجموعة التحكم التي استخدمها Windows آخر مرة. ويقرأ `--all-control-sets`
  كل مجموعة، مما قد يُظهر مشاركة أو وكيلًا أُزيل من المجموعة الحالية.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry networks <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and SYSTEM hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry networks -d evidence/ -o networks.csv
juntei parse-registry networks -d evidence/ -t jsonl \
  | jq 'select(.Source == "port_proxy" or .Source == "share")'
```

يسرد الملخص الشبكات التي اتُّصل بها مؤخرًا (الأحدث أولًا)، وكل مشاركة ووكيل منفذ
ومسار دائم.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `SourcePath` | المضيف، والخلية. |
| `ControlSet`، `IsCurrentControlSet` | لسجلات SYSTEM: مجموعة التحكم، وما إذا كانت هي التي استخدمها Windows آخر مرة. |
| `Source` | `network_profile`، `network_signature` (توقيع لا يملكه أي ملف تعريف)، `network_card`، `interface`، `share`، `port_proxy`، `route`. |
| `ProfileName` | اسم ملف التعريف (SSID لشبكة Wi-Fi)؛ ووصف التوقيع. |
| `Category`، `ConnectionType` | `public`، `private`، `domain`؛ `wired`، `wireless`، `vpn`، `mobile_broadband` (وإلا فالرقم). |
| `FirstConnectedUTC`، `LastConnectedUTC` | `DateCreated` و`DateLastConnected` بعد تحويلهما إلى UTC باستخدام `TimeZone`. |
| `FirstConnectedLocal`، `LastConnectedLocal` | القيمتان نفسهما كما وُجدتا (بالتوقيت المحلي، حتى الملّي ثانية). |
| `TimeZone` | المنطقة الزمنية المستخدمة في التحويل. |
| `GatewayMAC`، `DNS-Suffix` | عنوان MAC للبوابة ولاحقة DNS لتواقيع ملف التعريف (`|` بين القيم المتعددة). |
| `InterfaceGUID` | معرّف GUID للواجهة؛ و`ServiceName` لبطاقة الشبكة. |
| `IP-Address`، `DhcpServer`، `LeaseObtainedUTC`، `LeaseExpiresUTC` | عناوين الواجهة المستخدمة، وخادم DHCP وعقد الإيجار. |
| `ShareName`، `SharePath` | المشاركة والمجلد الذي تتيحه. |
| `ListenAddress`، `ListenPort`، `ConnectAddress`، `ConnectPort` | وكيل منفذ: ما يستمع عليه وإلى أين يعيد التوجيه. |
| `Route` | مسار دائم كما هو مخزن: `destination,netmask,gateway,metric`. |
| `Details` | قيم أخرى، بصيغة `Name=Value; …` (CSV) أو كائن (JSON): وصف ملف التعريف، و`Managed`، والاسم الأول ومفاتيح التواقيع؛ ومحوّل الواجهة (من `NetworkCards`)، والأقنعة، والبوابات، وخوادم DNS والمجال؛ ونوع المشاركة وملاحظتها وحقولها الأخرى؛ ونوع الوكيل؛ وأجزاء المسار. |
| `KeyPath`، `ValueName`، `KeyLastWrittenUTC` | المفتاح، والقيمة للمشاركة أو الوكيل أو المسار، وآخر كتابة للمفتاح. |
| `DataRaw` | بايتات يتعذر فك ترميزها: وقت ليس `SYSTEMTIME` صالحًا، أو مشاركة ليست قائمة سلاسل نصية. |
| `TransactionLogStatus`، `LogsApplied` | سياق استعادة سجل Windows. |
| `Problems` | قيم من نوع أو حجم خاطئ، وتواريخ مستحيلة، وأسماء لا تتوافق مع صيغتها. |

## استراتيجية التحقيق { #investigation-strategy }

1. رتّب ملفات التعريف حسب `LastConnectedUTC` وافحص الشبكات المحيطة بالحادثة: شبكات Wi-Fi
   غير المعروفة، ونقاط الاتصال الفعالة للجوال، وشبكات VPN.
2. استخدم `GatewayMac` و`DnsSuffix` للتمييز بين الشبكات التي تحمل الاسم نفسه، ولمطابقة
   شبكة عبر عدة مضيفين.
3. افحص كل وكيل منفذ ومشاركة. فوكيل منفذ إلى المنفذ 3389 أو 445 لمضيف داخلي، أو مشاركة
   لمجلد تجهيز (`C:\ProgramData`، `C:\Users\Public`)، يمثل خيطًا للتحقيق. واربط ذلك بسجلات
   الأحداث (إنشاء عملية `netsh`، وأحداث المشاركة 5142/5144) وأوقات المفاتيح.
4. استخدم عناوين الواجهات وأوقات عقود الإيجار لتحديد موقع المضيف على الشبكة، وطابقها
   مع سجلات جدار الحماية وDHCP والوكيل.
5. استخدم `--all-control-sets` و`parse-registry read --recover-deleted` للمشاركات
   والوكلاء وملفات التعريف المُزالة.

## مزيد من التفاصيل { #further-details }

- [Microsoft: netsh interface portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001: Proxy: Internal Proxy](https://attack.mitre.org/techniques/T1090/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
