# ذاكرة التخزين المؤقت لصور RDP النقطية { #rdp-bitmap-cache }

## ما الذي تسجله ذاكرة التخزين المؤقت لصور RDP النقطية { #what-the-rdp-bitmap-cache-records }

عندما يتصل عميل سطح المكتب البعيد (`mstsc.exe`) والتخزين المؤقت الدائم للصور النقطية (persistent bitmap
caching) مفعّل، وهو الإعداد الافتراضي، فإنه يحفظ أجزاءً من الشاشة البعيدة على القرص حتى لا تضطر الجلسات
اللاحقة إلى استلامها مرة أخرى. وكل جزء منها مربع (tile) لا يتجاوز 64 × 64 بكسل. وتشكّل هذه المربعات مجتمعةً
أجزاءً صغيرة مما رآه المستخدم على أجهزة الكمبيوتر البعيدة: النوافذ، والنصوص، وأسماء الملفات، والأوامر،
وأسطح المكتب.

| العميل | ملفات ذاكرة التخزين المؤقت | المربعات |
|---|---|---|
| Windows 7 والإصدارات الأحدث | `Cache0000.bin`، `Cache0001.bin`، …، بحد أقصى 100 MB لكل ملف | 32 بت، مربع تلو الآخر |
| Windows XP وServer 2003 والإصدارات الأقدم | `bcache2.bmc` ‏(8 بتات لكل بكسل)، و`bcache22.bmc` ‏(16)، و`bcache23.bmc` ‏(24)، و`bcache24.bmc` ‏(32)، بحد أقصى 10 MB لكل ملف | مربع واحد لكل خانة بحجم 64 × 64؛ وبعضها مضغوط |

ويترك Windows 10 والإصدارات الأحدث أيضًا ملف `bcache24.bmc` فارغًا بجانب ملفات `.bin`.

<span class="evidence-strength">أفضل استنتاج:</span> اتصل المستخدم الذي يحتوي ملف تعريفه على ذاكرة التخزين
المؤقت، باستخدام عميل سطح المكتب البعيد، بجهاز كمبيوتر عرض المحتوى المخزن مؤقتًا. ولا تحدد ذاكرة التخزين
المؤقت أي جهاز كان، ولا متى.

## المواقع والجمع { #locations-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```

يمكن للقيمة `BitmapPersistCacheLocation` في `HKCU\Software\Microsoft\Terminal Server Client` أن تنقل ذاكرة
التخزين المؤقت إلى موقع آخر. اجمع مجلد `Cache` بأكمله من كل ملف تعريف، ومن نسخ الظل لوحدة التخزين (volume
shadow copies) أيضًا: فقد تحتوي نسخة أقدم من ملف ذاكرة التخزين المؤقت على مربعات لم تعد موجودة في النسخة
الحالية.

## تحذيرات للمحلل { #analyst-cautions }

- ليست للمربعات أوقات، ولا تسمّي ذاكرة التخزين المؤقت أجهزة الكمبيوتر التي جاءت منها. فاربطها باتصالات RDP
  من أدلة أخرى: مفاتيح سجل Windows ‏`Terminal Server Client` لدى العميل (`parse-registry remote-access`)،
  و`Default.rdp` وملفات `.rdp` الأخرى، وJump Lists الخاصة بـ `mstsc.exe`، وأوقات ملفات ذاكرة التخزين المؤقت
  في `$MFT`، وأحداث تسجيل الدخول على أجهزة الكمبيوتر البعيدة.
- يمزج ملف ذاكرة التخزين المؤقت الواحد مربعات من جلسات كثيرة ومن عدة أجهزة كمبيوتر بعيدة. ويستبدل العميل
  المربعات القديمة كلما امتلأ الملف، لذا قد يكون عمر المربع أيامًا أو أشهرًا.
- المربعات أجزاء من الشاشات، لا شاشات كاملة. وغالبًا ما تكون المربعات المتجاورة على الشاشة متجاورة في الملف
  أيضًا، ولكن ليس دائمًا. أعد بناء الشاشات باستخدام [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)،
  أو بالعين من الصور المجمّعة (collages).
- معظم المربعات بحجم 64 × 64 بكسل. أما المربعات عند الحافتين اليمنى والسفلى للشاشة فتكون أضيق أو أقصر، بحسب
  دقة الشاشة: بعرض 58 أو 62 بكسل، أو بارتفاع 28 أو 35 بكسل، على سبيل المثال. ويقرأ Juntei كل مربع بحجمه الخاص.
  أما الأدوات التي تقرأ كل مربع على أنه بعرض 64 بكسل فتعرض المربعات الضيقة مائلة.
- معظم المربعات أحادية اللون (`SingleColor`) خلفيات. فاستبعدها بالتصفية لتعثر على المحتوى.
- في ملفات `.bmc`، تكون ألوان المربعات ذات 8 بتات مأخوذة من لوحة ألوان الجلسة، وهي لوحة لا تحتفظ بها ذاكرة
  التخزين المؤقت. ويستخدم Juntei لوحة الألوان الافتراضية في Windows، لذا قد تكون ألوانها خاطئة. وتستخدم
  المربعات المضغوطة ترميز interleaved RLE الخاص ببروتوكول سطح المكتب البعيد (MS-RDPBCGR 3.1.9)، الذي يفك
  Juntei ضغطه كما تعرّفه المواصفة.
- يترك مربع `.bmc` الأصغر من خانته البالغة 64 × 64 بعده ذيلَ مربع سابق. ويحفظ Juntei تلك الصفوف المتبقية صورةً
  منفصلة (`RemnantImage`). وهي تعود إلى مربع أقدم، لا إلى المربع الموجود في الخانة.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file rdp-cache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Cache????.bin and bcache*.bmc files
  -f, --file <FILE>      Parse one cache file, whatever its name

Output:
  -o, --output <FILE>       Save one record per tile
  -t, --output-type <TYPE>  csv, json, or jsonl
  -x, --extract <DIR>       Save each tile as a BMP and a PNG collage of each cache file
      --collage-only        With --extract, save only the collages
      --collage-width <N>   Tiles per collage row (default 64)
```

```bash
juntei parse-file rdp-cache -d evidence/ -o rdp-cache.csv --extract rdp-cache/
juntei parse-file rdp-cache -f Cache0000.bin --extract rdp-cache/ --collage-only
juntei parse-file rdp-cache -d evidence/ -t jsonl | jq 'select(.SingleColor == false)'
```

مع `-d`، يقرأ Juntei الملفات التي يتكون اسمها من `Cache` متبوعًا بأرقام ثم `.bin`، ومن `bcache` متبوعًا بأرقام
ثم `.bmc`. أما ملف `Cache????.bin` الذي لا يبدأ بالتوقيع `RDP8bmp` فيعود إلى برنامج آخر: إذ يُحتسب في الملخص
ويُتخطى. ومع `-f`، يقرأ Juntei الملف أيًّا كان اسمه.

### الصور { #images }

ينشئ `--extract` مجلدًا لكل ملف من ملفات ذاكرة التخزين المؤقت، `<DIR>/<user>/<cache file>/`. وعندما يكون من
شأن ملفين أن يتشاركا المجلد نفسه، مثل ملف ذاكرة تخزين مؤقت ونسخته من نسخة ظل لوحدة التخزين، يُرقَّم المجلد
الثاني (`Cache0001.bin_2`). ويحتوي المجلد على:

- `<cache file>_<index>.bmp`: كل مربع، بحجمه الخاص. والأسماء هي نفسها التي تستخدمها bmc-tools، لذا يستطيع
  RdpCacheStitcher فتح المجلد.
- `<cache file>_old_<index>.bmp`: لملفات `.bmc` فقط، ذيل مربع سابق متروك بعد مربع أصغر.
- `<cache file>_collage.png`: كل المربعات بترتيبها في الملف، 64 مربعًا في كل صف (`--collage-width`)، كلٌّ منها
  في خلية بحجم 64 × 64 بدءًا من زاويتها العلوية اليسرى. ويقع المربع ذو الفهرس *i* في الصف *i* / 64 والعمود
  *i* % 64، بالعد من 0. ويملأ اللون الأرجواني (magenta) الأجزاء الفارغة من الخلايا: بجانب المربع الضيق،
  وأسفل المربع القصير، وحيث تعذّرت قراءة مربع.

يحفظ `--collage-only` الصور المجمّعة فقط. ولا تُستبدل صورة موجودة من قبل دون `-C`، ويذكر ذلك حقل `Problems`
الخاص بمربعها. أما ملفات ذاكرة التخزين المؤقت نفسها فلا تُستبدل أبدًا، حتى مع `-C`.

## حقول إخراج CSV { #csv-output-fields }

| الحقل | المعنى |
|---|---|
| `SourceFile` | ملف ذاكرة التخزين المؤقت. |
| `User` | المستخدم الذي يقع ملف ذاكرة التخزين المؤقت في مجلد ملف تعريفه، استنادًا إلى مساره. |
| `TileIndex` | رقم المربع في الملف، بالعد من 0، كما في اسم صورته. |
| `Width`، `Height` | حجم المربع بالبكسل. |
| `SingleColor` | ‏`true` عندما يكون لكل بكسلات المربع اللون نفسه. |
| `TileImage` | مع `--extract`: ملف BMP الخاص بالمربع. |
| `Collage` | مع `--extract`: الصورة المجمّعة لملف ذاكرة التخزين المؤقت. |
| `RemnantImage` | ‏`.bmc` مع `--extract`: ذيل مربع سابق متروك في الخانة. |
| `Key` | مفتاح المربع ذو 64 بت في بروتوكول سطح المكتب البعيد، بالنظام الست عشري. |
| `Format`، `FormatVersion` | ‏`bin` مع الإصدار الذي يلي توقيعه (6 في Windows 10 والإصدارات الأحدث)، أو `bmc`. |
| `BitsPerPixel` | 32 لـ `.bin`؛ و8 أو 16 أو 24 أو 32 لـ `.bmc`. |
| `Compressed`، `Flags` | ‏`.bmc`: ما إذا كان المربع مضغوطًا، وأعلام ترويسته. |
| `Offset` | موضع بداية ترويسة المربع في الملف. |
| `Problems` | التلف: ملف ينتهي في منتصف مربع، أو ترويسة مربع لا يمكن أن تكون لأي مربع، أو بيانات لا تتوافق مع حجم المربع، أو بيانات مضغوطة يتعذّر فك ضغطها، أو صورة تعذّر حفظها. |

تتضمن JSON وJSONL الحقول نفسها، مع أنواع فعلية للأرقام والقيم المنطقية، و`null` للقيم غير الموجودة في المربع،
و`Problems` على هيئة قائمة. ويُبلَّغ عن التلف الواقع خارج أي مربع بعينه، مثل البيانات التي تلي آخر مربع، في
رسائل التشغيل.

## التحليل التلقائي { #automatic-parsing }

يكتب `auto-parse` الملف `LateralMovement/RdpCache.csv` (أو `RdpCache.json` أو `RdpCache.jsonl` مع
`-t json` أو `-t jsonl`)، مع `LateralMovement/RdpCache-logs.txt`، ويحفظ الصورة المجمّعة لكل ملف من ملفات ذاكرة
التخزين المؤقت في مجلد `LateralMovement/RdpCache/`. أضف `--extract-rdp-tiles` لحفظ كل مربع أيضًا، وقد يبلغ
ذلك عشرات الآلاف من الملفات لمستخدم واحد. ومن صور الأقراص تُنسخ ملفات ذاكرة التخزين المؤقت. ويتضمن الجدول
الزمني المدمج `-j` كل صف مع معرّف `Artifact` ذي القيمة `rdp-cache`. وليست للمربعات أوقات، لذا لا ينشئ
[`super-timeline`](../commands/super-timeline.md) منها أي أحداث.

## استراتيجية التحقيق { #investigation-strategy }

1. افتح الصور المجمّعة وابحث عن النصوص: أسماء أجهزة الكمبيوتر والمستخدمين، ومسارات الملفات، والأوامر، ونوافذ
   وحدة التحكم، وأشرطة عناوين المتصفحات، ونوافذ الحوار.
2. أعد بناء الشاشات باستخدام RdpCacheStitcher من مجلد مربعات محفوظة باستخدام `--extract`.
3. حدّد أجهزة الكمبيوتر التي اتصل بها المستخدم، من سجل Windows الخاص بالعميل (`parse-registry remote-access`)،
   وملفات `.rdp`، وJump Lists الخاصة بـ `mstsc.exe`. وعلى تلك الأجهزة، ابحث عن عمليات تسجيل الدخول عبر RDP
   المطابقة وعن سجلات تسجيل وصول المستخدمين (UAL).
4. استخدم أوقات `$MFT` لملفات ذاكرة التخزين المؤقت، وأوقات نسخها في نسخ الظل لوحدة التخزين، لتضييق النطاق
   الزمني الذي كُتبت فيه المربعات.

## مزيد من التفاصيل { #further-details }

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017: ذاكرة التخزين المؤقت لصور RDP النقطية (بالفرنسية)](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — المكوّن الإضافي لذاكرة التخزين المؤقت لـ RDP](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9: Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — ذاكرة التخزين المؤقت الدائمة للصور النقطية](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
