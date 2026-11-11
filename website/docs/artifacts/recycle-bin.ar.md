# سلة المحذوفات { #recycle-bin }

## ما الذي تسجله سلة المحذوفات { #what-the-recycle-bin-records }

عندما يحذف المستخدم ملفًا أو مجلدًا في مستكشف الملفات، ينقله Windows إلى سلة المحذوفات في وحدة
التخزين التي يوجد عليها، ويكتب بجانبه سجل فهرسة صغيرًا: المسار الذي كان فيه، وحجمه، ووقت حذفه.
ولكل مستخدم مجلد سلة على كل وحدة تخزين، يحمل اسم معرّف SID الخاص به، لذا يدل المجلد أيضًا على
الحساب الذي حذف العنصر.

| إصدار Windows | الفهرس | المحتوى المحذوف |
|---|---|---|
| Vista والإصدارات الأحدث | ملف `$I…` واحد لكل عنصر (إصدار التنسيق 1 من Vista إلى 8.1، والإصدار 2 في Windows 10 والإصدارات الأحدث) | `$R…`، بالاسم نفسه بعد `$R`: الملف نفسه، أو المجلد مع محتوياته |
| NT 4.0، 2000، XP، Server 2003 | ملف `INFO2` واحد لكل سلة: سجلات بحجم 800 بايت تتضمن مسارًا بترميز ANSI ومسارًا بترميز Unicode | `D<drive letter><index><extension>`، مثل `Dc1.txt` |
| 95، 98، ME | ملف `INFO2` واحد (`INFO` في Windows 95): سجلات بحجم 280 بايت لا تتضمن إلا مسارًا بترميز ANSI | أسماء `D…` نفسها |

<span class="evidence-strength">أفضل استنتاج:</span> نقل الحساب الذي تحتوي سلته على السجل العنصرَ
الموجود في المسار الأصلي إلى سلة المحذوفات في الوقت المسجل. أما الحذف باستخدام Shift+Delete، أو من
سطر الأوامر، أو من معظم البرامج، فيتجاوز السلة ولا يترك أي سجل.

## المواقع والجمع { #locations-and-collection }

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

اجمع ملفات `$I` أو INFO2 من سلة كل وحدة تخزين. ويُظهر جمع ملفات `$R` و`D…` أيضًا العناصر المحذوفة
التي لا تزال موجودة؛ ومن دونها تكون قيمة `RecycledFileFound` هي `false` لكل عنصر. والمحتوى المحذوف
دليل قائم بذاته وقد يكون برمجية ضارة: فتعامل معه على هذا الأساس.

## تحذيرات للمحلل { #analyst-cautions }

- يمثل `DeletedTime` وقت نقل العنصر إلى السلة وفق ساعة النظام، بصيغة FILETIME بتوقيت UTC. وهو ليس
  وقت إفراغ السلة. أما أوقات إنشاء العنصر نفسه وتعديله فتوجد في `$MFT` وفي ملف `$R`.
- يمثل معرّف SID الحساب الذي حذف العنصر، وليس بالضرورة مالك الملف. وإذا لم يكن لمعرّف SID ملف تعريف
  في `ProfileList` ضمن خلية SOFTWARE (كحساب محذوف، أو حساب مجال لم يسجّل الدخول قط)، فلا تكون له
  قيمة في `User`.
- عند إفراغ السلة، أو استعادة عنصر منها أو حذفه منها، يحذف Windows Vista والإصدارات الأحدث ملفَي
  `$I` و`$R` الخاصين به. وقد تبقى أسماؤهما في دفتر يومية USN و`$MFT` والمساحة الراكدة (slack) في
  `$I30`؛ ولا يستعيد Juntei منها سجلات `$I` المحذوفة. أما INFO2 فيحتفظ بهذه السجلات ويمحو البايت
  الأول من مسار ANSI الخاص بها: فتكون قيمة `RemovedFromBin` هي `true`، ويعيد Juntei ذلك الحرف
  استنادًا إلى رقم محرك الأقراص.
- يمثل `FileSize` في ملف `$I` حجم العنصر، وفي حالة المجلد الحجم الإجمالي لمحتوياته. أما في INFO2
  فهو المساحة التي شغلها العنصر على القرص، بعناقيد كاملة (4,096 بايت للملف الصغير).
- غالبًا ما يكون مسار ANSI في INFO2 بصيغة 8.3، ويتطلب صفحة الترميز الخاصة بالنظام: استخدم
  `--code-page 932` لنظام Windows الياباني، و936 أو 950 للصيني. أما مسار Unicode، إن وُجد، فهو
  `OriginalPath`.
- كتبت بعض أنظمة Windows Vista ملفات `$I` بحجم 543 بايت يقل فيها حقل الحجم بايتًا واحدًا. ويقرأ
  Juntei وقتها ومسارها ويترك `FileSize` فارغًا، مع الإبلاغ عن مشكلة.
- يصف `RecycledFileFound` البيانات المجمّعة، لا نقطة النهاية: فقد تعني `false` أن عملية الجمع لم
  تأخذ ملفات `$R`، أو أن العنصر لم يعد في السلة.
- تستخدم سلال وحدات التخزين غير النظامية ملفات تعريف الكمبيوتر نفسه فقط، وفق بيانات الجمع أو بنية مجلدات الأقراص ووحدات التخزين المعروفة. إذا كانت النسبة غير واضحة، يبقى المستخدم غير معروف، ولا تُستخدم ملفات تعريف مضيف آخر.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

مع `-d`، يقرأ Juntei الملفات التي يتكون اسمها من `$I` متبوعًا بستة أحرف أو أرقام (بأي امتداد)،
وملفات `INFO2`، وملف `INFO` داخل مجلد `RECYCLED`. ومع `-f`، يقرأ الملف أيًّا كان اسمه.

## حقول إخراج CSV { #csv-output-fields }

| الحقل | المعنى |
|---|---|
| `SourceFile` | ملف `$I` أو INFO2. |
| `DeletedTime` | وقت نقل العنصر إلى السلة (UTC). |
| `OriginalPath` | المكان الذي كان فيه العنصر. وفي INFO2، مسار Unicode إن وُجد في السجل، وإلا فمسار ANSI. |
| `FileSize` | `$I`: حجم العنصر بالبايت (الحجم الإجمالي للمجلد). INFO2: المساحة التي يشغلها على القرص، بعناقيد كاملة. يكون فارغًا إذا تعذّرت قراءته. |
| `User`، `SID` | معرّف SID لمجلد السلة، والمستخدم الذي يحمل ملف تعريفه في `ProfileList` اسمه. |
| `RecycledFile`، `RecycledFileFound` | مسار المحتوى المحذوف بجانب الفهرس (`$R…`، `D<drive><index><extension>`)، وما إذا كان موجودًا في البيانات المجمّعة. |
| `Format`، `FormatVersion` | `$I` ‏1 (من Vista إلى 8.1) أو 2 (Windows 10 والإصدارات الأحدث)؛ `INFO2` ‏0 (95)، أو 2 (NT 4.0)، أو 4 (98)، أو 5 (ME، ومن 2000 إلى Server 2003). |
| `RecordIndex`، `Drive`، `RecordOffset` | INFO2: رقم السجل (الرقم الوارد في اسم ملف `D…` الخاص به)، ومحرك الأقراص، وموضع بداية السجل في INFO2. ولا يكون لمسار UNC محرك أقراص. |
| `RemovedFromBin` | INFO2: ‏`true` عندما يكون العنصر قد استُعيد أو حُذف من السلة. |
| `ShortPath` | INFO2: مسار ANSI، عندما يختلف عن `OriginalPath` (وغالبًا ما يتكون من أسماء 8.3). |
| `Problems` | التلف: ملف مبتور أو أطول من اللازم، أو رقم محرك أقراص غير صالح، أو مسار ليس UTF-16 صالحًا، أو وقت حذف ليس وقتًا صالحًا. |

تتضمن JSON وJSONL الحقول نفسها، مع أنواع فعلية للأرقام والقيم المنطقية، و`null` للقيم غير الموجودة
في السجل، و`Problems` على هيئة قائمة.

إذا تعذر سرد دليل السلة أو كان اسم المحتوى ملتبسًا، تكون `RecycledFileFound` غير معروفة (خلية CSV فارغة أو JSON `null`)، ويُسجّل السبب في `Problems`. لا يمكن للمخرجات استبدال المحتوى المحذوف، حتى عبر الروابط الصلبة، أو الكتابة داخل مجلد محذوف. تُحفظ النتائج الموجودة إذا تعذرت قراءة كل الفهارس.

## التحليل التلقائي { #automatic-parsing }

يكتب `auto-parse` الملف `FileExistence/RecycleBin.csv` (أو `RecycleBin.json` أو `RecycleBin.jsonl`
مع `-t json` أو `-t jsonl`)، مع `FileExistence/RecycleBin-logs.txt`. ويُطبَّق خياره `--code-page`
على INFO2. ومن صور الأقراص تُنسخ ملفات `$I` وINFO2؛ أما ملفات `$R` و`D…` فتكون عناصر نائبة فارغة،
لذا يظل الحقل `RecycledFileFound` معبّأً. ويتضمن الجدول الزمني المدمج `-j` كل صف مع معرّف `Artifact`
ذي القيمة `recycle-bin`، ويحوّل `-Z` الحقل `DeletedTime`، وينشئ
[`super-timeline`](../commands/super-timeline.md) حدثًا واحدًا `Recycle Bin deletion (moved to the bin)`
لكل عنصر.

## استراتيجية التحقيق { #investigation-strategy }

1. رتّب السجلات حسب `DeletedTime` لكل مستخدم، وقارنها بأوقات النشاط الذي تحقق فيه.
2. ابحث عن `OriginalPath` في LNK وJump Lists وShellBags وRecentDocs لمعرفة ما إذا كان العنصر قد
   استُخدم قبل حذفه.
3. في دفتر يومية USN، يظهر الحذف على شكل إعادة تسمية إلى `$Recycle.Bin` في وقت قريب من
   `DeletedTime`؛ ويوفر `$MFT` أوقات العنصر نفسه.
4. عندما تكون قيمة `RecycledFileFound` هي `true`، احسب تجزئة محتوى `$R` أو `D…` وافحصه.
5. إذا أُفرغت السلة، فابحث في دفتر يومية USN و`$MFT` عن أسماء `$I` و`$R`.

## مزيد من التفاصيل { #further-details }

- [libyal dtformats — Windows Recycle.Bin file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — Recycle Bin and INFO2 parsers](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
