# قوائم العناصر المستخدمة مؤخرًا (RunMRU، TypedPaths، WordWheelQuery، TypedURLs) { #most-recently-used-lists-runmru-typedpaths-wordwheelquery-typedurls }

## ما هي هذه القوائم { #what-the-lists-are }

يحتفظ Windows وبرامجه بقوائم قصيرة لآخر ما كتبه المستخدم أو فتحه، في الملف `NTUSER.DAT` الخاص
بالمستخدم:

- **RunMRU:** الأوامر المكتوبة في مربع حوار "تشغيل" (Win+R): `cmd`، `powershell -ep bypass`،
  `\\server\share`، `regedit`.
- **TypedPaths:** المسارات المكتوبة في شريط عنوان المستكشف (Explorer).
- **WordWheelQuery:** مصطلحات البحث المكتوبة في مربع البحث في المستكشف.
- **TypedURLs:** العناوين المكتوبة في شريط عنوان Internet Explorer، ومنذ Windows 8 يسجل
  `TypedURLsTime` وقت كتابة كل منها.
- **Paint وWordPad:** آخر الملفات التي فتحها كل منهما (`Recent File List`).
- **ACMru (Windows XP):** مصطلحات مساعد البحث (Search Assistant)، لكل نوع من أنواع البحث.

<span class="evidence-strength">أفضل استنتاج:</span> كتب المستخدم هذا الأمر أو المسار أو مصطلح
البحث أو العنوان، أو فتح البرنامج هذا الملف. ولا يُؤرَّخ إلا أحدث إدخال في كل قائمة، بوقت آخر كتابة
لمفتاحها، باستثناء العناوين المكتوبة التي يؤرخها `TypedURLsTime`.

## الموقع { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

المفاتيح الفرعية لـ `ACMru` هي أنواع البحث: `5603` أسماء الملفات والمجلدات، و`5604` الكلمات أو
العبارات داخل الملفات، و`5647` أجهزة الكمبيوتر والأشخاص، و`5001` الإنترنت.

اجمع `NTUSER.DAT` مع سجلات المعاملات الخاصة به لكل ملف تعريف.

## تحذيرات للمحلل { #analyst-cautions }

- **وقت واحد لكل قائمة.** يتغير وقت آخر كتابة للمفتاح عند تغيّر قائمته، لذا فهو يؤرخ أحدث إدخال
  فقط (`IsMostRecent`، `LastUsedUTC`). أما الإدخالات الأخرى فمرتبة لكنها غير مؤرخة.
  ويؤرخ `TypedURLsTime` كل عنوان مكتوب (`TypedTimeUTC`).
- **مكتوب، لا مُشغَّل.** يسجل RunMRU ما كُتب في مربع حوار "تشغيل"، سواء نجح تشغيله أم لا. ويخزن
  Windows كل أمر مع لاحقة `\1`؛ ويزيلها Juntei. اربط ذلك مع Prefetch وBAM وUserAssist وأحداث
  إنشاء العمليات.
- **القوائم قصيرة.** تحتفظ كل قائمة بأحدث إدخالاتها فقط، وتمسحها إعدادات الخصوصية وأدوات
  التنظيف. استخدم `parse-registry read --recover-deleted` لاسترداد القيم المحذوفة.
- **المصطلحات هي ما كُتب.** يحتفظ WordWheelQuery وACMru بالنص المكتوب، لا بما عُثر عليه.
- **الترتيب يأتي من القائمة.** يحدد `MRUList` في RunMRU و`MRUListEx` في WordWheelQuery الترتيب؛
  أما في القوائم الأخرى فيحدده رقم القيمة (`url1` و`File1` و`000` هي الأحدث).

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

يسرد الملخص الإدخالات لكل قائمة، والإدخالات المؤرخة الأخيرة (العناوين المكتوبة، وأحدث إدخال في كل
قائمة)، من الأحدث إلى الأقدم.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `User`، `UserSID`، `SourcePath` | المضيف، ومالك ملف التعريف (من المسار أو مجلدات الواجهة)، ومعرّف SID من `ProfileList` الخاص بالنظام، والخلية. |
| `List` | `RunMRU` أو `TypedPaths` أو `WordWheelQuery` أو `TypedURLs` أو `Paint` أو `Wordpad` أو `ACMru`. |
| `Subkey` | المفتاح الفرعي لـ `ACMru`: نوع البحث (`5603`، `5604`، `5647`، `5001`). |
| `MRU-Position`، `IsMostRecent` | الموضع في القائمة (0 هو الأحدث)؛ ويكون فارغًا حين لا تسمّي القائمة القيمة. |
| `ValueName` | `a`، `0`، `url1`، `File1`، `000`، … |
| `Value` | الأمر (دون `\1`)، أو المسار، أو مصطلح البحث، أو العنوان، أو الملف. |
| `TypedTimeUTC` | وقت كتابة العنوان، من `TypedURLsTime`. |
| `LastUsedUTC` | وقت آخر كتابة للمفتاح، لأحدث إدخال فقط. |
| `KeyPath`، `KeyLastWrittenUTC` | المفتاح ووقت آخر كتابة له: وهو وقت أحدث إدخال فيه، لا وقت كل صف. |
| `DataRaw` | بايتات القيمة (بالنظام الست عشري) حين يتعذر فك ترميزها. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows. |
| `Problems` | قيم تسمّيها القائمة ولا يحتويها المفتاح، وقيم لا تسمّيها القائمة، وقيم بأسماء خاطئة، وأنواع خاطئة، ومصطلحات مقتطعة، وأوقات ليست بتنسيق FILETIME. |

## استراتيجية التحقيق { #investigation-strategy }

1. اقرأ RunMRU بحثًا عن الأوامر والبرامج النصية والمشاركات الشبكية وأدوات الإدارة التي كتبها المستخدم.
2. اقرأ TypedPaths وWordWheelQuery بحثًا عن المجلدات والملفات التي بحث عنها المستخدم:
   بيانات الاعتماد، والملفات المُصدَّرة، والأرشيفات، والمواقع القابلة للإزالة والشبكية.
3. ضع العناوين المكتوبة على الجدول الزمني باستخدام `TypedTimeUTC`.
4. اربط ذلك مع Prefetch وBAM وUserAssist وShellBags وملفات LNK وJump Lists وسجل المتصفح.

## مزيد من التفاصيل { #further-details }

- [libyal winreg-kb: مفاتيح العناصر المستخدمة مؤخرًا (MRU)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
