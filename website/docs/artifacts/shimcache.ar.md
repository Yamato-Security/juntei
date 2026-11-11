# ShimCache (AppCompatCache) { #shimcache-appcompatcache }

## ما هو ShimCache { #what-shimcache-is }

يخزّن النظام الفرعي لتوافق التطبيقات في Windows مسارات الملفات التنفيذية وبياناتها الوصفية مؤقتًا
أثناء تقييم سلوك التوافق/التعديل (shim). وتبقى ذاكرة التخزين المؤقت قائمة حتى بعد إعادة تسمية الملفات
أو إزالتها أو فصلها مع وحدة تخزين قابلة للإزالة. ويختلف تخطيطها والحقول المتاحة فيها من Windows XP
حتى Windows 11.

يُعد ShimCache قيّمًا لاكتشاف المسارات التاريخية، كما يقدّم دعمًا إضافيًا لاستنتاج التنفيذ حيث يحتوي
التنسيق على علامة تنفيذ موثوقة. والوقت المخزّن مؤقتًا هو عادةً **وقت آخر تعديل للملف**، لا الوقت الذي
رآه فيه Windows أو شغّله.

<span class="evidence-strength">أفضل استنتاج:</span> رصد Windows المسار لأغراض معالجة التوافق.
والإدخال وحده دليل على الوجود/الرصد؛ لا تستخدم `Executed` إلا حيث يكون مدعومًا.

## الموقع والجمع { #location-and-collection }

توجد البايتات في خلية سجل Windows `SYSTEM`، وعادةً تحت:

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

اجمع `SYSTEM` و`SYSTEM.LOG1` و`SYSTEM.LOG2`. يقرأ Juntei جميع مجموعات التحكم ويحدد
`Select\Current` حيثما أمكن.

!!! warning "قد لا تكون أحدث الإدخالات الحية في الخلية"
    يمكن لـ Windows أن يحتفظ بتحديثات ShimCache في الذاكرة ويكتبها إلى القرص أثناء إيقاف التشغيل/إعادة
    التشغيل. وقد لا تحتوي خلية منسوخة من نظام قيد التشغيل على أحدث عمليات الرصد. ويساعد وقت آخر كتابة
    لذاكرة التخزين المؤقت في سجل Windows على وصف وقت حفظها، لكنه ليس وقت تنفيذ لكل إدخال.

## تحذيرات للمحلل { #analyst-cautions }

- إدخال ذاكرة التخزين المؤقت وحده **ليس دليلًا على التنفيذ**.
- يمكن أن تكون قيمة `Executed` هي `Yes` أو `No` أو `NA`؛ وتعتمد الدلالات المتاحة على التنسيق/الإصدار.
- ينتمي `LastModifiedTimeUTC` إلى الملف المشار إليه، لا إلى الرصد أو التنفيذ.
- الترتيب مفيد، لكنه ليس جدولًا زمنيًا دقيقًا عامًا؛ وتحدث تكرارات عبر مجموعات التحكم/المساحة الراكدة.
- استُردت إدخالات `FromSlack` من خارج ذاكرة التخزين المؤقت النشطة وتتطلب حذرًا إضافيًا.
- قد تُغفل خلية `SYSTEM` غير النظيفة (dirty) دون سجلات قابلة للاستخدام السجلاتِ الحالية. اقرأ ملخص الاسترداد.
- تشير النتائج (Findings) إلى أنماط تسمية/مسارات مريبة، لا إلى أحكام بوجود برمجيات خبيثة.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shimcache -f SYSTEM -o shimcache.csv
juntei parse-registry shimcache -d evidence/ -t jsonl \
  | jq 'select(.Executed == true or (.Findings | length > 0))'
```

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `ControlSet` | رقم مجموعة التحكم التي تحتوي على ذاكرة التخزين المؤقت. قارن `IsCurrent` قبل اختيار إحداها. |
| `CacheEntryPosition` | الموضع/الترتيب داخل ذاكرة التخزين المؤقت بعد فك ترميزها. مفيد للترتيب النسبي، وليس وقتًا. |
| `Path` | مسار الملف التنفيذي/الحزمة المخزن مؤقتًا. النقطة المحورية الرئيسية للبحث. |
| `LastModifiedTimeUTC` | وقت تعديل الملف المخزن في الإدخال—وليس وقت التنفيذ/الرصد. |
| `Executed` | `Yes` أو `No` أو `NA` وفقًا لعلامات التنسيق المدعومة. لا تستخدمه إلا مع سياق الإصدار. |
| `Duplicate` | سبق رصد الإدخال نفسه في مجموعة تحكم سابقة أو في المساحة الراكدة؛ يساعد على تجنب العد المزدوج. |
| `SourceFile` | الخلية/القيمة الخام المُدخلة. ضروري عبر المضيفات. |
| `FromSlack` | إدخال مسترد من المساحة الراكدة لقيمة ذاكرة التخزين المؤقت بدلًا من القائمة النشطة. ثقة أدنى وربما أقدم. |
| `CacheFormat` | تخطيط Windows/ذاكرة التخزين المؤقت بعد فك ترميزه. يحدد توفر الحقول ودلالاتها. |
| `CacheLastWrittenUTC` | وقت حفظ مفتاح سجل Windows/ذاكرة التخزين المؤقت. وهو سياق مشترك، لا وقت تشغيل كل إدخال. |
| `InsertFlags`، `ShimFlags` | علامات التوافق الخام؛ و`Executed` هو النتيجة التي يفسرها Juntei حيثما كان ذلك مدعومًا. |
| `DataSize` | حجم بيانات هذا الإدخال، مفيد للتحقق البنيوي. |
| `Machine` | البنية المعمارية التي تسجلها التنسيقات الأحدث (`x64`، `x86`، `ARM64`). |
| `FileSize`، `LastUpdateTimeUTC` | بيانات وصفية مخزنة مؤقتًا خاصة بـ Windows XP حيثما وُجدت. |
| `PackageName`، `PackageVersion`، `PackagePublisherID` | هوية التطبيق المحزّم في الإدخالات المدعومة. ومعرّف الناشر سياق، لا دليل على التوقيع. |
| `Findings` | رموز الحالات الشاذة في Juntei، مثل ملف ثنائي للنظام منقول أو مسار تجهيز (staging) مريب. خيوط للتحقيق فقط. |
| `RegistryPath`، `RegistryValueName` | مصدر المفتاح/القيمة الدقيق في الخلية. احتفظ به للتحقق. |
| `CacheValueSize` | حجم قيمة سجل Windows بأكملها، لا `DataSize` الخاص بالإدخال الفردي. |
| `IsCurrent` | ما إذا كانت مجموعة التحكم لهذا الإدخال تساوي `SYSTEM\Select\Current`؛ ويكون فارغًا إذا لم يُعرف ذلك بأمان. |
| `SelectCurrent`، `SelectDefault`، `SelectLastKnownGood`، `SelectFailed` | أرقام مجموعات التحكم الخام في `SYSTEM\Select` للسياق ولتحليل الاسترداد. |

تحتفظ JSON/JSONL كذلك ببيانات الإدخالات، والوسوم، والسجلات الفرعية للتطبيقات المحزّمة، والنتائج
المنظّمة، ومشكلات ذاكرة التخزين المؤقت، وتفاصيل الاسترداد.

## استراتيجية التحقيق { #investigation-strategy }

1. فضّل مجموعة التحكم الحالية، لكن احتفظ بالمجموعات الأخرى وبالمساحة الراكدة دليلًا تاريخيًا.
2. ابحث في المسارات والنتائج؛ ولا تستخدم `LastModifiedTimeUTC` وقتًا للتنفيذ.
3. حيث تكون `Executed == Yes`، عزّز ذلك بـ Prefetch أو SRUM أو سجلات الأحداث أو آثار وقت تشغيل أخرى.
4. طابق المسارات مع `$MFT`/USN لتحديد تاريخ الإنشاء والحذف والبيانات الوصفية.
5. اذكر في التقرير ما إذا كانت سجلات المعاملات قد أُعيد تطبيقها، وما إذا كان من المحتمل أن ذاكرة
   التخزين المؤقت الحية لم تُكتب إلى القرص.

## مزيد من التفاصيل { #further-details }

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [shimcache-forensic من Security Ronin](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant: الاستفادة من ذاكرة التخزين المؤقت لتوافق التطبيقات في التحقيقات الجنائية (Leveraging the Application Compatibility Cache in Forensic Investigations)](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [مواصفات تنسيق سجل Windows من Maxim Suhanov](https://github.com/msuhanov/regf)
