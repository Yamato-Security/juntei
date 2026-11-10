# RecentDocs { #recentdocs }

## ما هو RecentDocs { #what-recentdocs-is }

يخزّن Windows Explorer قوائم بالملفات والمجلدات المستخدمة مؤخرًا (MRU) في ملف `NTUSER.DAT` لكل مستخدم.
تغطي القائمة الجذرية العناصر الأخيرة من جميع الأنواع؛ وتجمع المفاتيح الفرعية الامتدادات مثل `.docx`،
إضافةً إلى قائمة `Folder`. ويحدد `MRUListEx` ترتيب القيم المرقمة.

يفيد RecentDocs في تحديد المستندات التي تفاعل معها المستخدم، بما في ذلك الملفات التي حُذفت أو نُقلت
لاحقًا. ويمكن أن تحتوي القيم أيضًا على اسم الاختصار الذي استخدمه Windows في مجلد Recent الخاص
بالمستخدم، والذي يمكن ربطه بأدلة LNK.

<span class="evidence-strength">أفضل استنتاج:</span> سجّل Explorer العنصر في قائمة RecentDocs الخاصة
بذلك المستخدم. ولا يؤرّخ وقت المفتاح إلا أحدث عنصر في تلك القائمة بعينها.

## الموقع { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

اجمع `NTUSER.DAT` وسجلات المعاملات الخاصة به لكل ملف تعريف ذي صلة، إضافةً إلى مجلد LNK الخاص بـ Recent
لدى المستخدم.

## تحذيرات للمحلل { #analyst-cautions }

- يعطي `MRUListEx` الترتيب، لا طابعًا زمنيًا لكل عنصر.
- يقابل وقت آخر كتابة للمفتاح أحدث تحديث؛ ولا يحصل على `LastOpenedUTC` مشتق منه وقابل للدفاع عنه
  إلا موضع MRU رقم 0.
- يمكن أن يظهر العنصر في القائمة الجذرية وفي قائمة امتداد. لا تحسبه مرتين على أنه عمليتا فتح.
- قد يختلف سلوك التطبيقات وExplorer؛ وقد لا تحدّث بعض طرق الوصول RecentDocs.
- تزيل أدوات التنظيف/الخصوصية وحدود حجم القوائم الأدلة الأقدم.
- يمكن أن يتحكم المهاجم في اسم الملف، وهو ليس دليلًا على نوع المحتوى.
- يُستدل على المستخدم من مسارات ملف التعريف أو بيانات مجلدات الواجهة (shell folders)؛ راجع أسماء
  المستخدمين الفارغة/الملتبسة.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry recent-docs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recent-docs -d evidence/ -t jsonl \
  | jq 'select(.Extension == ".docx" or .Extension == ".pdf")'
```

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host` | المضيف المشتق من سياق المجموعة/النظام حيثما كان متاحًا. |
| `User` | مالك ملف التعريف المستدل عليه من المسار أو من مسارات مجلدات الواجهة؛ احتفظ بعدم اليقين عندما يكون فارغًا. |
| `SourcePath` | ملف `NTUSER.DAT` المُدخل. ضروري عند الجمع بين عدة مستخدمين. |
| `Extension` | القائمة الفرعية (`.docx`، `Folder`، إلخ)؛ وتكون null لقائمة RecentDocs الجذرية. |
| `MRU-Position` | الصفر هو الأحدث داخل هذه القائمة. وهذا ترتيب نسبي، لا زمن منقضٍ. |
| `ValueIndex` | قيمة سجل Windows المرقمة التي يشير إليها `MRUListEx`. وهي نقطة محورية بنيوية فقط. |
| `Filename` | اسم العنصر بعد فك ترميزه. اربط المسار الكامل عبر LNK أو Jump List أو MFT أو بيانات التطبيقات. |
| `LinkName` | اسم الاختصار المرمّز في عنصر الواجهة (shell item)، ويوجد غالبًا في مجلد Recent الخاص بالمستخدم. |
| `KeyLastWrittenUTC` | وقت آخر تغيير لمفتاح القائمة. ينطبق على القائمة، لا على كل صف. |
| `LastOpenedUTC` | لا يُملأ إلا لموضع MRU رقم 0، حيث يدعم وقت المفتاح هذا التفسير. |
| `ValueDataRaw` | البايتات الخام عند فشل فك الترميز أو عند حاجة الدليل إلى التحقق. |
| `TransactionLogStatus`، `LogsApplied` | ما إذا كانت سجلات سجل Windows قد أُعيد تطبيقها وأيّها. |
| `Problems` | قيم MRU مفقودة، أو قيم غير مدرجة، أو أنواع خاطئة، أو تلف في فك الترميز. |

## استراتيجية التحقيق { #investigation-strategy }

1. افصل القائمة الجذرية وقوائم الامتدادات لكل مستخدم؛ وأزل تكرار العنصر نفسه بعناية.
2. استخدم موضع MRU للترتيب، ولا تستخدم إلا `LastOpenedUTC` لأي ادعاء مؤرّخ بطابع زمني.
3. طابق `LinkName`/`Filename` مع بيانات LNK في Recent وJump List و`$MFT` وUSN.
4. راجع الأسماء الخادعة، والامتدادات المزدوجة، والأهداف الشبكية/القابلة للإزالة، والمستندات القريبة
   من الوصول الأولي.
5. اربط النتائج ببيانات Prefetch/SRUM للتطبيقات وبسجلات الأحداث عندما يكون التنفيذ مهمًا.

## مزيد من التفاصيل { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [libfwsi من libyal](https://github.com/libyal/libfwsi)
- [أداتا RECmd وRegistry Explorer من Eric Zimmerman](https://github.com/EricZimmerman/RECmd)
- [ملصق SANS لتحليل Windows الجنائي](https://www.sans.org/posters/windows-forensic-analysis/)
