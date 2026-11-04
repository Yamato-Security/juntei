# خلايا سجل Windows { #windows-registry-hives }

## ما هي خلايا سجل Windows { #what-registry-hives-are }

يخزن سجل Windows تكوين النظام والمستخدم في ملفات خلايا ثنائية. وتشكّل المفاتيح شجرة؛
وتحتوي القيم الموجودة تحت مفتاح ما على بيانات ذات أنواع محددة. وتصف آثار سجل Windows الاستمرارية، والبرامج المثبتة،
والخدمات، والحسابات، وتكوين الشبكة، ونشاط المستخدم، وإعدادات الأمان، وكل جزء تقريبًا
من حالة Windows.

يفك القارئ العام في Juntei ترميز أي خلية أساسية، ويعيد تشغيل سجلات المعاملات، ويُبلغ عن التلف، ويمكنه
استعادة السجلات المحذوفة/الموجودة في الخلايا الحرة (free-cell)، وفي المساحة الراكدة للخلايا (cell-slack)، والبقايا (remnant)، وما قبل إعادة التشغيل (pre-replay)، والسجلات الموجودة في سجلات المعاملات فقط.

<span class="evidence-strength">أفضل استنتاج:</span> احتوت الخلية على حالة المفتاح/القيمة أو حفظتها بشكل قابل
للاستعادة. ويُظهر وقت آخر كتابة للمفتاح أن المفتاح تغيّر، لا أي قيمة تغيرت ولا لماذا.

## الخلايا الشائعة ومواقعها { #common-hives-and-locations }

| الخلية | المسار المعتاد | أمثلة على الأدلة |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | الخدمات، وبرامج التشغيل، ومجموعات التحكم، واسم المضيف، والأجهزة المُركَّبة، وShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | البرامج المثبتة، وTaskCache، وملفات التعريف، وتكوين Windows |
| `SAM` | `C:\Windows\System32\config\SAM` | الحسابات المحلية والبيانات الوصفية للحسابات |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | البيانات الوصفية لنهج الأمان المحلي/الأسرار (حسب صلاحيات الوصول) |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | تكوين ملف تعريف النظام الافتراضي |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | تفضيلات المستخدم، ومفاتيح Run، وRecentDocs، ونشاط التطبيقات |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | فئات shell، ونشاط COM/shell الخاص بالمستخدم |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | جرد التطبيقات (استخدم `parse-registry amcache` للحصول على مخرجات دلالية) |

اجمع ملفات `.LOG` و`.LOG1` و`.LOG2` المجاورة. فالخلية المنسوخة أثناء تشغيل Windows قد تكون
غير متسقة (dirty)؛ وتحتوي سجلات المعاملات على حالة مُلتزَم بها لا يعكسها الملف الأساسي وحده بعد.

## تحذيرات للمحلل { #analyst-cautions }

- ينطبق الطابع الزمني لآخر كتابة للمفتاح على **المفتاح**، لا على قيمة بعينها.
- لا يكون لقيم سجل Windows عادةً طابع زمني مستقل.
- `CurrentControlSet` اسم مستعار يستخدمه المحلل/Windows ويُحل عبر `SYSTEM\Select`؛ وليس
  مفتاحًا مخزنًا حرفيًا.
- قد تكون السجلات المحذوفة جزئية، أو قديمة، أو مكررة عبر مصادر الاستعادة، أو مكتوبًا فوقها.
- تُظهر السجلات الموجودة في سجلات المعاملات فقط حالة تاريخية/وسيطة. اشرح `recovered_from`.
- تعني النتيجة `absent_value` أن المفتاح المُستعلَم عنه موجود لكنه يفتقر إلى تلك القيمة في الحالة المحللة؛
  ولا تثبت أن القيمة لم توجد قط.
- السلاسل النصية والمسارات وأسطر الأوامر تكوين، لا دليل على أن الإجراء المُكوَّن قد نُفذ.
- قد تُغفل الخلية غير المتسقة من دون سجلات قابلة للتطبيق حالة ما أو تكشف حالة قديمة.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host` | اسم المضيف المستمد من سياق النظام المتاح؛ فارغ عندما لا يستطيع Juntei تحديده بأمان. |
| `HiveType` | فئة الخلية المكتشفة (`SYSTEM`، `SOFTWARE`، `NTUSER.DAT`، إلخ). استخدمها لتفسير المسارات. |
| `SourcePath` | خلية الإدخال بالضبط. ضروري عندما يجمع وضع المجلد بين عدة مضيفين/مستخدمين. |
| `RecordType` | `key` أو `value` أو `default_value` أو `absent_value`. ولا ينطبق الغياب إلا على الاستعلام/الحالة. |
| `KeyPath` | المسار تحت جذر الخلية. ويشير `[unknown]` إلى أسلاف محذوفة لم تُحل. |
| `KeyLastWrittenUTC` | آخر تعديل للمفتاح ككل. ولا يمكنه تحديد أي قيمة تغيرت. |
| `ValueName` | القيمة المسماة؛ فارغ للقيمة الافتراضية/غير المسماة. |
| `ValueType` | نوع سجل Windows (`REG_SZ`، `REG_DWORD`، `REG_BINARY`، إلخ). ويؤثر النوع في التفسير. |
| `ValueData` | القيمة المعروضة بحسب نوعها. وتبقى السلاسل المتعددة قوائم في JSON. |
| `DataSize` | الحجم المسجل في القيمة. قارنه بالبايتات الخام و`Problems`. |
| `ValueDataRaw` | البايتات الدقيقة عند فشل فك الترميز أو عند استخدام `--raw-data`. احتفظ بها للتحقق. |
| `ValueU64` | التمثيل العددي الكامل لـ DWORD/QWORD دون فقدان الدقة في JavaScript. |
| `ValueFiletimeUTC` | عرض اختياري لـ QWORD على هيئة FILETIME. لا يكون ذا معنى إلا عندما يعرّف مخطط المفتاح FILETIME. |
| `AllocationStatus` | `allocated` أو `deleted_complete` أو `deleted_partial` أو `residual_value_only` أو `transaction_log_recovered`. ويحدد مستوى الثقة. |
| `RecoveredFrom` | `free_cell` أو `cell_slack` أو `remnant_data` أو `before_replay`، أو إدخال سجل مثل `SYSTEM.LOG1#1234`. |
| `TransactionLogStatus` | `clean` أو `replayed` أو `unchanged` أو `not_applied` لسياق هذا السجل/الخلية. |
| `LogsApplied` | أسماء ملفات سجلات المعاملات التي أعاد Juntei تشغيلها. سجّلها في التقرير. |
| `Problems` | التلف، أو الخلايا المفقودة، أو عدم تطابق النوع، أو القوائم/الآباء غير المحلولة، أو قيود أخرى. |

## تفسير حالة الاستعادة { #recovery-status-interpretation }

| الحالة | المعنى |
|---|---|
| `deleted_complete` | استُعيد مفتاح/قيمة محذوفة بمسار/بيانات معروفة كاملة. ومع ذلك فهي تاريخية، وليست حالة نشطة. |
| `deleted_partial` | كُتب فوق جزء من المسار أو قائمة القيم أو القيمة أو البيانات أو أصبح غير قابل للقراءة. لا تملأ الفجوات من خلايا أُعيد استخدامها. |
| `residual_value_only` | بقيت القيمة دون سياق كافٍ للمفتاح؛ وقد يكون `KeyPath` غائبًا. |
| `transaction_log_recovered` | الحالة موجودة فقط في إصدارات ما قبل إعادة التشغيل/السجل، وليست في السجلات المخصصة/الحرة الحالية. |

## استراتيجية التحقيق { #investigation-strategy }

1. ابدأ بمفتاح دلالي محدد عند معرفته؛ واستخدم البحث بالتعبيرات النمطية/البيانات للاستكشاف.
2. افحص `TransactionLogStatus` و`AllocationStatus` و`RecoveredFrom` و`Problems` قبل المحتوى.
3. فسّر البيانات وفق المخطط الموثق للمفتاح؛ فـ `REG_BINARY` لا يصف نفسه.
4. اربط التكوين بأدلة وقت التشغيل (سجلات الأحداث، وPrefetch، وSRUM، والخدمات/المهام).
5. احتفظ بالبيانات الخام للحقول الجديدة أو المتنازع عليها.

## مزيد من التفاصيل { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [مواصفة تنسيق سجل Windows من Maxim Suhanov](https://github.com/msuhanov/regf)
- [محلل سجل Windows ‏yarp](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [مكتبة Registry من Eric Zimmerman](https://github.com/EricZimmerman/Registry)
- [Microsoft: أنواع قيم سجل Windows](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
