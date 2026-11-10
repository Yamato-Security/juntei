# أسرار LSA { #lsa-secrets }

## ما هي { #what-they-are }

تحتفظ سلطة الأمان المحلية (Local Security Authority، ‏LSA) بأسرار في خلية `SECURITY`. وتشمل هذه
الأسرار كلمة مرور تسجيل الدخول التلقائي (autologon)، وكلمات مرور الحسابات التي تعمل بها الخدمات، وكلمة
مرور حساب الكمبيوتر في المجال، ومفاتيح DPAPI الخاصة بالنظام، والمفتاح الذي تُشفَّر به عمليات تسجيل
الدخول المخزنة مؤقتًا للمجال. كما تحتفظ بعمليات تسجيل الدخول المخزنة مؤقتًا للمجال، بحيث يتمكن مستخدمو
المجال من تسجيل الدخول دون وحدة تحكم بالمجال.

يسرد `parse-registry lsa-secrets` الأسرار التي يحتفظ بها النظام ووقت ضبطها، وعدد عمليات تسجيل الدخول
المخزنة مؤقتًا التي يحتفظ بها ويستخدمها. **إنه لا يقرأ إلا الأسماء والأوقات: لا يُفك تشفير أي شيء، ولا
تُقرأ بايتات أي سر ولا تُكتب.** يقرأ Juntei الأدلة؛ ولا يستخرج بيانات الاعتماد.

<span class="evidence-strength">أفضل استنتاج:</span> احتفظ النظام بسرّ بهذا الاسم، ضُبط في هذا الوقت.
ويعني سرّ الخدمة (`_SC_<service>`) أن LSA تخزّن كلمة مرور الحساب الذي تعمل به تلك الخدمة.

## الموقع { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| الاسم | النوع | ما هو |
|---|---|---|
| `DefaultPassword` | `autologon` | كلمة المرور التي يسجّل بها Winlogon الدخول تلقائيًا. |
| `_SC_<service>` | `service_account` | كلمة مرور الحساب الذي تعمل به `<service>`. |
| `$MACHINE.ACC` | `machine_account` | كلمة مرور حساب الكمبيوتر في مجاله. |
| `DPAPI_SYSTEM` | `dpapi_system` | مفاتيح DPAPI الخاصة بالنظام. |
| `NL$KM` | `cached_logon_key` | المفتاح الذي تُشفَّر به عمليات تسجيل الدخول المخزنة مؤقتًا. |
| `L$_SQSA_<SID>` | `security_questions` | أسئلة الأمان لحساب محلي (Windows 10 والإصدارات الأحدث). |
| `RasDialParams!<SID>#n`، `L$_Ras…` | `ras_credentials` | بيانات اعتماد الاتصال الهاتفي (dial-up) وVPN. |
| `L$…`، `G$…`، `M$…` | `local`، `global`، `machine` | بيانات خاصة أخرى، بحسب البادئة التي يعطيها لها `LsaStorePrivateData`. |

اجمع `SECURITY` مع سجلات المعاملات الخاصة به، وخلية `SYSTEM` الخاصة بالنظام، والتي تسمّي المضيف.

## تحذيرات للمحلل { #analyst-cautions }

- **أسماء وأوقات، لا قيم.** يُستدل على احتواء القيمة على بيانات من حجمها. ولا يُقرأ محتواها.
- **`DefaultPassword` لا يعني تسجيل الدخول التلقائي.** يوجد هذا السر على معظم الأنظمة، بما فيها أنظمة لا
  تسجّل الدخول تلقائيًا أبدًا. ويوضح المفتاح
  `Microsoft\Windows NT\CurrentVersion\Winlogon` في `SOFTWARE` (‏`AutoAdminLogon`، `DefaultUserName`)
  ما إذا كان تسجيل الدخول التلقائي مكوَّنًا. ويُعد `CurrentSetUTC` الحديث تغييرًا يستحق التفسير.
- **أسرار الخدمات:** يظهر `_SC_<service>` عندما تُضبط خدمة للعمل بحساب ذي كلمة مرور. قارنه بـ
  `parse-registry services` (‏`ObjectName`)، وابحث عن الخدمات المثبتة حديثًا أو غير المعروفة. ويمكن أن
  يبقى السر بعد زوال الخدمة.
- **الأوقات:** ‏`CurrentSetUTC` و`OldSetUTC` هما الوقتان اللذان ضبط فيهما Windows القيمتين الحالية
  والسابقة. تتغير كلمة مرور `$MACHINE.ACC` كل 30 يومًا افتراضيًا، لذا تتحرك أوقاتها.
  ويتعلق `KeyLastWrittenUTC` وأوقات المفتاحين `CurrVal`/`OldVal` بآخر وقت تغيّر فيه السر.
- **عمليات تسجيل الدخول المخزنة مؤقتًا:** تكون الخانة (slot) قيد الاستخدام عندما يسمّي سجلها مستخدمًا.
  ترويسة السجل غير مشفرة؛ ولا يُقرأ منها إلا طول اسم المستخدم. ‏`CachedLogonsKept` هو العدد الوارد في
  `NL$Control`، والذي يتبع `CachedLogonsCount` (‏10 افتراضيًا). وتعني الخانات المستخدمة أن مستخدمي
  المجال سجّلوا الدخول تفاعليًا، ويمكن كسر بيانات اعتمادهم دون اتصال (offline) انطلاقًا من الخلية. أسماء
  المستخدمين مشفرة ولا يُفك ترميزها. ولا تُعد قيد الاستخدام إلا السجلات الثنائية التي جرى التحقق منها
  ذات الترويسة الكاملة وطول اسم المستخدم المتسق. وإذا أبلغ `Problems` عن خانات تالفة، فإن عدد الخانات
  المستخدمة لا يشمل إلا الخانات التي جرى التحقق منها؛ فلا تفسّر الخانات التالفة على أنها فارغة.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

لكل خلية، يسرد الملخص كل سر مع نوعه ووقت ضبطه. ثم يسمّي الخدمات التي تخزّن LSA كلمات مرور حساباتها،
ويعدّ خانات تسجيل الدخول المخزنة مؤقتًا المستخدمة.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `SourcePath` | المضيف (من خلية `SYSTEM` الخاصة بالنظام)، وخلية `SECURITY`. |
| `Secret`، `Kind` | اسم السر، وماهيته (انظر الجدول أعلاه)؛ و`Cache` و`cached_logons` لسجل عمليات تسجيل الدخول المخزنة مؤقتًا. |
| `ServiceName`، `UserSID` | الخدمة التي يخصها سر `_SC_`، ومعرّف SID للمستخدم الذي يعطيه الاسم (`L$_SQSA_`، `RasDialParams`). |
| `CurrentSetUTC`، `OldSetUTC` | `CupdTime` و`OupdTime`: وقتا ضبط القيمتين الحالية والسابقة. |
| `HasCurrentValue`، `HasOldValue` | ما إذا كان `CurrVal` و`OldVal` يحتويان على بيانات. |
| `CurrentValueLastWrittenUTC`، `OldValueLastWrittenUTC`، `KeyLastWrittenUTC` | آخر كتابة لمفاتيح `CurrVal` و`OldVal` والسر. |
| `CachedLogons`، `CachedLogonsInUse`، `CachedLogonsKept` | سجل عمليات تسجيل الدخول المخزنة مؤقتًا: الخانات، والخانات المستخدمة، والعدد الوارد في `NL$Control`. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows. |
| `Problems` | أوقات مشوّهة أو خارج النطاق (لا تُعرض إلا البيانات الوصفية للوقت)، وترويسات ذاكرة تخزين مؤقت تالفة، وأنواع قيم خاطئة لذاكرة التخزين المؤقت، ومفاتيح تتعذر قراءتها. |

## استراتيجية التحقيق { #investigation-strategy }

1. ابحث عن أسرار `_SC_` لخدمات غير معروفة أو مثبتة حديثًا، وطابقها مع
   `parse-registry services` والحدث 7045 (تثبيت خدمة).
2. افحص أوقات `DefaultPassword` مقابل إعدادات تسجيل الدخول التلقائي في `Winlogon`.
3. على الأجهزة الأعضاء في مجال، دوّن خانات تسجيل الدخول المخزنة مؤقتًا المستخدمة، ووقت آخر كتابة لـ `Cache`.
4. قارن أوقات الأسرار بالجدول الزمني: فالسر الذي ضُبط خلال نافذة الاختراق قد يدل على حساب خدمة جديد،
   أو على تغيير في تسجيل الدخول التلقائي أو في حساب الجهاز.

## مزيد من التفاصيل { #further-details }

- [Microsoft: كائن البيانات الخاصة (Private data object)](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft: معلومات تسجيل الدخول المخزنة مؤقتًا للمجال](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [winreg-kb من libyal: بيانات الاعتماد المخزنة مؤقتًا](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
