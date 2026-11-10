# ذاكرة التخزين المؤقت لجدولة المهام { #task-scheduler-cache }

## ما هو TaskCache { #what-taskcache-is }

يحتفظ Task Scheduler في Windows بذاكرة تخزين مؤقت في سجل Windows ضمن `SOFTWARE` تربط مسارات المهام
بمعرّفات GUID، والمشغّلات/القوائم، والكيانات الأمنية (principals)، والإجراءات، والبيانات الوصفية لوقت
التشغيل، وواصفات الأمان. يفك Juntei ترميز إجراءات التنفيذ وإجراءات معالجات COM، إضافةً إلى إجراءات
البريد الإلكتروني/الرسائل القديمة، والطوابع الزمنية للمهام، وعلامات المهام المخفية عن واجهة Task
Scheduler العادية.

المهام آلية شائعة للاستمرارية والتنفيذ عن بُعد. كما أن ذاكرة التخزين المؤقت في سجل Windows قيّمة عندما
يحذف المهاجم ملف XML الخاص بالمهمة أو يعبث به.

<span class="evidence-strength">أفضل استنتاج:</span> كوّن TaskCache المهمة/الإجراء أو احتفظ بهما.
وتلزم حقول وقت التشغيل وسجلات مستقلة لإثبات تنفيذ بعينه.

## المواقع { #locations }

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

اجمع `SOFTWARE` مع سجلات المعاملات وشجرة `System32\Tasks` كاملة. وتضيف سجلات أحداث Task Scheduler
Operational وSecurity أدلة على الإنشاء/التحديث/وقت التشغيل.

## تحذيرات للمحلل { #analyst-cautions }

- قد تتعارض ذاكرة التخزين المؤقت في سجل Windows مع ملف XML الخاص بالمهمة بعد العبث، أو الحذف، أو
  التسجيل غير المكتمل، أو التلف.
- يحدد `Hidden` في Juntei الإخفاء البنيوي (غياب الربط بـ Tree/واصف الأمان في ظل الظروف ذات الصلة)، لا
  مجرد مهمة مكوّنة بعلامة "Hidden" العادية في الواجهة.
- يخزّن Windows 7 إجراءات المهام في XML بدلًا من القيمة `Actions` في ذاكرة التخزين المؤقت؛ لذا فإن
  الإجراء الفارغ هناك ليس مريبًا تلقائيًا.
- `LastRun` و`LastResult` مفيدان، لكنهما ليسا سجل تنفيذ كاملًا ومحصّنًا ضد العبث.
- يسمّي `Context` الكيان الأمني/السياق المكوَّن، لا المستخدم الذي سجّل المهمة.
- تتطلب معرّفات CLSID لمعالجات COM تحليلًا منفصلًا في سجل Windows/الملفات.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.Hidden == true or (.Executable? | test("powershell|cmd|mshta"; "i")))'
```

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `SourcePath` | سياق المضيف وخلية `SOFTWARE` المصدر. |
| `TaskName` | المسار الكامل في المجدول. النقطة المحورية الرئيسية إلى XML وسجلات الأحداث. |
| `TaskGUID` | معرّف GUID الذي يربط إدخالات ذاكرة التخزين المؤقت في Tree وTasks. مفتاح ربط قوي. |
| `TaskIndex` | بيانات Tree/الفهرس الوصفية التي يستخدمها المجدول؛ سياقية، وليست عدد مرات تنفيذ. |
| `TaskLists` | قوائم ذاكرة التخزين المؤقت (`Boot`، `Logon`، `Plain`، `Maintenance`) التي تصنّف التشغيل/المعالجة. |
| `Hidden` | مؤشر بنيوي على المهمة المخفية. راجع `InTree` و`InTasks` وحالة الواصف قبل استنتاج حدوث عبث. |
| `InTree`، `InTasks` | ما إذا كان جانبا ذاكرة التخزين المؤقت موجودين. وقد يشير غياب الربط إلى حذف/عبث/تسجيل غير مكتمل. |
| `HasSecurityDescriptor` | ما إذا كان لإدخال Tree قيمة `SD`؛ وقد يسهم غيابها في سلوك المهمة المخفية في الإصدارات ذات الصلة. |
| `TreeKeyLastWrittenUTC`، `TaskKeyLastWrittenUTC` | تغييرات على مستوى المفتاح لكل جانب من جانبي ذاكرة التخزين المؤقت؛ وليست أوقات إنشاء/تشغيل دقيقة. |
| `ActionIndex` | رقم الإجراء داخل المهمة بدءًا من الصفر. يمكن لمهمة واحدة أن تنتج عدة صفوف. |
| `ActionType` | `exec` أو `com_handler` أو `send_email` أو `show_message`. |
| `Executable`، `Args`، `WorkingDirectory` | إجراء التنفيذ الفعلي. فسّر علامات الاقتباس، والمتغيرات، والمسارات النسبية، وLOLBins. |
| `COM-ClassID`، `ActionData` | معرّف CLSID لمعالج COM وبياناته، أو حمولة البريد الإلكتروني/الرسالة القديمة. حلّل CLSID بشكل منفصل. |
| `Context` | سياق الكيان الأمني المرمّز مع الإجراءات (`LocalSystem`، `Author`، إلخ)؛ هوية مكوَّنة، لا الجهة التي سجّلت المهمة. |
| `Author`، `URI`، `Description` | البيانات الوصفية للمهمة. يسهل على المهاجم التحكم فيها، وليست دليلًا على الثقة. |
| `LastRegisteredUTC` | وقت التسجيل/التحديث في DynamicInfo حيثما وُجد. |
| `LastRunUTC`، `LastSuccessfulRunUTC`، `LastResult` | بيانات وصفية مخزنة مؤقتًا لوقت التشغيل/النتيجة. اربطها بالسجلات التشغيلية وأدلة العمليات. |
| `ActionsRaw` | بايتات الإجراء الأصلية إذا كان فك الترميز غير مكتمل أو كان التحقق مطلوبًا. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows. |
| `Problems` | عدم تطابق المسار/GUID، أو نوع قيمة خاطئ، أو بنية إجراء مقتطعة، أو أجزاء مفقودة. |

## استراتيجية التحقيق { #investigation-strategy }

1. راجع المهام المخفية/غير المتسقة والمسارات غير التابعة لـ Microsoft أولًا.
2. أعد بناء الأوامر الفعلية، بما في ذلك الوسيطات، ومجلد العمل، وتوسيع متغيرات البيئة، وتحليل COM.
3. قارن سجل سجل Windows بملف XML الخاص بالمهمة، واحسب تجزئة الملف التنفيذي المشار إليه وتحقق من طوابعه الزمنية.
4. اربط `TaskName`/GUID/الوقت بأحداث Task Scheduler، وSecurity 4698/4702، وسجلات العمليات، وPrefetch، وMFT/USN.
5. ابحث في بيانات سجل Windows المحذوفة/المستردة عندما يكون المهاجم قد أزال المهمة.

## مزيد من التفاصيل { #further-details }

- [Microsoft: مخطط Task Scheduler](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft: سجل أحداث Task Scheduler](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 Scheduled Task](https://attack.mitre.org/techniques/T1053/005/)
