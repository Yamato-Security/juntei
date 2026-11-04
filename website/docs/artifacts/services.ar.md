# خدمات Windows { #windows-services }

## ما الذي يسجله تكوين الخدمات { #what-service-configuration-records }

تُكوَّن خدمات Windows وبرامج تشغيل النواة (kernel drivers) ضمن كل مجموعة تحكم (control set) في
`SYSTEM`. وتسمّي السجلات الملف التنفيذي/برنامج التشغيل أو ملف DLL الخاص بالخدمة، ووضع البدء، ونوع
الخدمة، والحساب، ومجموعة ترتيب التحميل، والبدء المؤجل، وسلوك معالجة الأخطاء، والإجراءات التي يتخذها
Windows بعد الفشل.

ينشئ المهاجمون الخدمات أو يعدّلونها لتحقيق الاستمرارية، وتصعيد الامتيازات، وتحميل برامج التشغيل،
والحركة الجانبية. وتستخدم البرامج المشروعة الآلية نفسها تمامًا، لذا فإن المسارات والحسابات والتوقيع
وسياق الإنشاء وأدلة وقت التشغيل كلها مهمة.

<span class="evidence-strength">أفضل استنتاج:</span> كوّنت مجموعة التحكم التي جرى تحليلها
الخدمة/برنامج التشغيل كما هو موضح. ولا يثبت التكوين وحده أن الخدمة بدأت بنجاح.

## الموقع { #location }

```text
SYSTEM\ControlSet00N\Services\<ServiceName>
SYSTEM\Select\Current   (which control set Windows used)
```

اجمع `SYSTEM` مع سجلات المعاملات الخاصة به. يمكن لمعرّفات أحداث مثل System 7045 وSecurity 4697 أن
تسجل التثبيت؛ ويمكن لأحداث التحكم في الخدمات والعمليات أن تدعم استنتاج البدء/التنفيذ الفعلي.

## تحذيرات للمحلل { #analyst-cautions }

- قد يحتوي `ImagePath` على متغيرات بيئة، أو علامات اقتباس، أو وسيطات، أو مسارات أجهزة، أو مسار غير
  محاط بعلامات اقتباس يتضمن مسافات. احتفظ به كما هو تمامًا قبل توحيد صيغته.
- يُعد `ServiceDll` حاسم الأهمية للخدمات المستضافة في `svchost.exe`؛ إذ قد لا يسمّي `ImagePath` وحده سوى المضيف.
- لا يثبت `StartType=automatic` أن الخدمة بدأت أو ظلت مفعّلة.
- `ObjectName` هو الحساب المكوَّن، وليس بالضرورة المستخدم الذي ثبّت الخدمة.
- تستخدم برامج التشغيل وخدمات وضع المستخدم علامات `ServiceType` واصطلاحات مسارات مختلفة.
- يمكن لإجراءات الفشل أن تنفذ أمرًا بحساب الخدمة بعد حدوث فشل؛ وهذا سطح استمرارية يسهل إغفاله.
- قد تحتفظ مجموعات التحكم الأخرى بتكوين سابق؛ استخدم `IsCurrentControlSet`.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --service <NAME>    Only this service name
      --all-control-sets  Parse every ControlSet00N

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --all-control-sets -t jsonl \
  | jq 'select(.FailureCommand != null or (.ImagePath? | test("temp|public"; "i")))'
```

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `SourcePath` | هوية النظام والخلية المُدخلة. احتفظ بهما عبر عمليات الجمع. |
| `ControlSet`، `IsCurrentControlSet` | مجموعة التحكم المصدر، وما إذا كان `Select\Current` يسمّيها. فضّل المجموعة الحالية عند استنتاج الحالة النشطة. |
| `ServiceName` | اسم مفتاح سجل Windows/الاسم الداخلي للخدمة. نقطة محورية قوية للانتقال إلى الأحداث وأوامر مدير التحكم في الخدمات (Service Control Manager). |
| `DisplayName`، `Description` | نص موجّه للمستخدم؛ وقد يكون مراجع إلى موارد أو نصًا يتحكم فيه المهاجم. |
| `ImagePath` | أمر الملف التنفيذي/برنامج التشغيل كما هو مكوَّن. حلّل علامات الاقتباس والوسيطات ومتغيرات البيئة ومسارات الأجهزة. |
| `ServiceDLL` | ملف DLL الذي تحمّله خدمة مضيفة. وكثيرًا ما يكون الحمولة الفعلية للخدمات ذات العمليات المشتركة. |
| `StartType`، `StartValue` | وضع البدء بصيغته المقروءة/الخام: boot (0)، وsystem (1)، وautomatic (2)، وmanual (3)، وdisabled (4). |
| `DelayedAutostart` | تبدأ الخدمة التلقائية بعد مرحلة الإقلاع الأولى؛ ولا تزال قادرة على تحقيق الاستمرارية. |
| `ServiceType`، `TypeValue` | العلامات بصيغتها المفكوكة/الخام: برنامج تشغيل نواة، أو برنامج تشغيل نظام ملفات، أو عملية خاصة/مشتركة، أو خدمة مستخدم، وغير ذلك. |
| `ErrorControl` | مستوى معالجة أخطاء الإقلاع/البدء. سياق تشغيلي في المقام الأول. |
| `ObjectName` | حساب الخدمة المكوَّن أو اسم كائن برنامج التشغيل. وليس هوية من قام بالتثبيت. |
| `Group` | مجموعة ترتيب التحميل، وهي مهمة خاصةً لبرامج تشغيل الإقلاع/النظام. |
| `FailureResetPeriod` | عدد الثواني قبل إعادة تعيين عدّاد مرات الفشل؛ وتعني `4294967295` عدم إعادة التعيين أبدًا. |
| `FailureCommand` | الأمر المكوَّن لإجراء الفشل `run_command`. حقل استمرارية عالي القيمة. |
| `FailureActions` | قائمة الإجراءات الكاملة بعد فك ترميزها (`restart`، `reboot`، `run_command`، `none`) مع فترات التأخير. |
| `FailureActionType`، `FailureActionDelayMs` | الإجراء الأول مسطحًا لأغراض التصفية؛ افحص القائمة الكاملة أيضًا. |
| `FailureActionsRaw` | البايتات الأصلية عندما يكون فك الترميز غير مكتمل أو تالفًا. |
| `KeyLastWrittenUTC` | آخر تغيير على مفتاح الخدمة، وليس بالضرورة وقت التثبيت/البدء. |
| `TransactionLogStatus`، `LogsApplied` | سياق إعادة تطبيق سجلات سجل Windows. |
| `Problems` | عدم تطابق النوع، أو بنية فشل تالفة، أو حقول مفقودة/غير قابلة للقراءة. |

## استراتيجية التحقيق { #investigation-strategy }

1. راجع المسارات غير القياسية، والمجلدات القابلة للكتابة، والحسابات غير المعتادة، وأوقات المفاتيح الجديدة، وأوامر الفشل.
2. وسّع متغيرات البيئة باستخدام تكوين المضيف المتأثر، لا جهاز المحلل.
3. احسب التجزئة وتحقق من التوقيع للملف التنفيذي أو ملف DLL الفعلي إن جُمع.
4. اربط اسم الخدمة/مسارها مع الأحداث 7045/4697، وإنشاء العمليات، وPrefetch، و`$MFT`، وUSN.
5. قارن مجموعات التحكم الحالية وغير الحالية بحثًا عن آليات استمرارية مُغيَّرة/محذوفة.

## مزيد من التفاصيل { #further-details }

- [Microsoft: شجرة سجل Windows لخدمات الأجهزة وبرامج التشغيل](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: SERVICE_STATUS / أنواع الخدمات وحالاتها](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_status)
- [Microsoft: إجراءات الفشل في ChangeServiceConfig2](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
