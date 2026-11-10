# دفتر يومية USN ‏(`$UsnJrnl:$J`) { #usn-journal-usnjrnlj }

## ما هو دفتر يومية USN { #what-the-usn-journal-is }

يسجّل NTFS تغييرات الملفات والمجلدات في دفتر يومية التغييرات الخاص برقم تسلسل التحديث (Update Sequence Number أو USN). ويمكن
للسجل أن يصف الإنشاء، وتوسيع البيانات/الكتابة فوقها، وتغييرات السمات أو الأمان، وإعادة التسمية،
والإغلاق، والحذف. ويخزن مرجع الملف، ومرجع الأب، والاسم، والأسباب، والسمات،
والطابع الزمني، ورقم USN متزايدًا باطراد.

يُعد دفتر اليومية ممتازًا لإعادة بناء موجة من عمليات المهاجم على الملفات، خاصةً عندما
تكون الملفات قد حُذفت. وهو يسجل **أن البيانات الوصفية/المحتوى قد تغيّر**، لا البايتات التي تغيرت.

<span class="evidence-strength">أفضل استنتاج:</span> سجّل NTFS أسباب التغيير المذكورة
للملف المُشار إليه في الطابع الزمني. وكثيرًا ما تتراكم الأسباب حتى سجل `CLOSE`.

## الموقع والجمع { #location-and-collection }

```text
C:\$Extend\$UsnJrnl:$J
```

`$J` دفق مسمّى من البيانات الوصفية لـ NTFS. اجمعه بأداة جنائية تدعم الوصول الخام. واجمع أيضًا
`$MFT` من وحدة التخزين نفسها، لأن سجلات USN تخزن مراجع ملفات الأب بدلًا من المسارات الكاملة.

## تحذيرات للمحلل { #analyst-cautions }

- دفتر اليومية دائري ومحدود الحجم. وتُكتب السجلات القديمة فوقها.
- يعني `FILE_DELETE` أن NTFS عالج الحذف؛ وقد يظل المحتوى قابلًا للاستعادة في مكان آخر.
- لا يكشف سبب مثل `DATA_OVERWRITE` البايتات أو العملية أو المستخدم المسؤول.
- يمكن الجمع بين عدة أسباب في سجل واحد؛ وكثيرًا ما يشير `CLOSE` إلى نهاية التغييرات المتراكمة.
- المسارات مُعاد بناؤها. وقد تكون المجلدات الأب قد أُعيدت تسميتها أو حُذفت أو أُعيد استخدامها.
- أرقام تسلسل مراجع الملفات مهمة. فرقم الإدخال وحده قد يربط بجيل خاطئ.
- يمكن لعملية الاستخراج (carving) استعادة سجلات صالحة دون سياق دفتر اليومية الأصلي؛ فأبلغ عن هذا التمييز.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $J files
  -f, --file <FILE>      Parse one $J, or any file with --carve
  -m, --mft <FILE>       Use the matching volume's $MFT for paths
      --carve            Search a disk image/unallocated file for records

Output:
  -o, --output <PATH>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-ntfs usn-journal -d evidence/ -o usn-jsonl -t jsonl
juntei parse-ntfs usn-journal -f unallocated.bin --carve -o carved-usn.csv
```

## حقول إخراج CSV { #csv-output-fields }

| الحقل | التفسير |
|---|---|
| `Name` | اسم الملف/المجلد المخزن في سجل USN هذا. وليس مسارًا كاملًا. |
| `Extension` | امتداد مستخرج من `Name` للتيسير؛ لا تستخدمه كنوع محتوى موثوق. |
| `EntryNumber`، `SequenceNumber` | مرجع NTFS للملف. استخدم كليهما لتحديد جيل السجل. |
| `ParentEntryNumber`، `ParentSequenceNumber` | مرجع NTFS للمجلد الأب وقت التغيير. |
| `ParentPath` | مسار الأب الذي أعاد Juntei بناءه مع مراعاة الزمن. وتعني علامات المجهول أنه تعذر حل الأسلاف. |
| `UpdateSequenceNumber` | قيمة ترتيب USN. مفيدة للفرز الثابت ولنقاط الارتكاز مع Amcache/MFT. |
| `UpdateTimestamp` | الطابع الزمني المخزن في سجلات V2/V3. تعامل مع الطوابع الزمنية في سياق الاستخراج بالتحقق نفسه المطبق على أي سجل. |
| `UpdateReasons` | علم تغيير مسمى واحد أو أكثر، مدمجة بالرمز `|`. فسّر التركيبات و`CLOSE` معًا. |
| `FileAttributes` | سمات NTFS المسجلة وقت التغيير؛ وتبقى البتات غير المعروفة بالنظام السداسي عشري. |
| `OffsetToData` | إزاحة البايت في ملف الإدخال. ضرورية للتحقق وللسجلات المستخرجة. |
| `SourceFile` | دفتر اليومية/الصورة المُدخلة؛ يميّز بين المضيفين ووحدات التخزين. |
| `SourceInfo` | أعلام المصدر التي تصف كيف دخل التغيير إلى دفتر اليومية. سياقية عادةً، لا إسناد. |
| `SecurityID` | فهرس معرّف الأمان في NTFS للسجل، لا معرّف SID للمستخدم. |
| `Version` | إصدار سجل USN. يكتب Juntei V2/V3 إلى CSV؛ وتحتفظ JSON بنطاقات تتبع المدى (range-tracking extents) في V4. |

تمثل JSON/JSONL مراجع V3 ذات 128 بت تمثيلًا صحيحًا وتحتفظ بسجلات تتبع المدى في V4 و
نطاقاتها. استخدمها عندما تكون الحقول منخفضة المستوى الخاصة بكل إصدار مهمة.

## استراتيجية التحقيق { #investigation-strategy }

1. صفِّ حسب الوقت أو الاسم أو الامتداد أو مسار الأب أو مرجع الملف.
2. رتّب حسب `UpdateSequenceNumber` إضافةً إلى الوقت لرؤية الترتيب الدقيق في دفتر اليومية.
3. اقرن `RENAME_OLD_NAME` بـ `RENAME_NEW_NAME` لعمليات النقل/إعادة التسمية.
4. استخدم `$MFT` للتحقق من حالة السجل والمسارات والأحجام والطوابع الزمنية.
5. اربط ذلك بإسناد العمليات من سجلات الأحداث، أو EDR، أو Prefetch، أو الخدمات/المهام، أو Defender.

## مزيد من التفاصيل { #further-details }

- [Microsoft: دفاتر يومية التغييرات](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: أعلام أسباب دفتر يومية التغييرات](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2#members)
- [حزمة usn في ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/usn)
- [Sootmark usn](https://github.com/Sootmark/usn)
