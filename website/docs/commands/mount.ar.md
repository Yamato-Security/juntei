# تحميل صور الأقراص { #mounting-disk-images }

يتيح `juntei mount` قراءة ملفات صورة قرص VHDX عبر مجلد للقراءة فقط طوال تشغيله. تستطيع أدوات مثل Hayabusa وJuntei فحصها دون استخراج الملفات أو طلب صلاحيات root. تستخدم الميزة FUSE وتعمل على Linux. تُصان مكتبة القراءة ومكتبة التركيب في [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

في طرفية أخرى:

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

اضغط Ctrl+C في الطرفية الأولى لإلغاء التحميل، أو شغّل `fusermount3 -u /mnt/host1`.

## ما يعرضه التحميل { #what-the-mount-shows }

- تظهر ملفات وحدة NTFS عند نقطة التركيب. قد تحتوي الصورة المجمعة على مجلدات مثل `C` و`LongFileNames`، وبيانات وصفية للوحدة مثل `$MFT`. تظهر وحدات NTFS المتعددة باسم `Volume1` و`Volume2` وهكذا.
- كل ملف للقراءة فقط ويعود إلى مالك نقطة التحميل. وأوقات الملفات هي أوقات `$STANDARD_INFORMATION`
  في NTFS.
- تُسرد الأسماء كما يخزنها NTFS، ويُبحث عنها بمطابقة تامة. ولا تُعرض أسماء DOS 8.3 ولا دفقات البيانات
  البديلة.
- يُفك ضغط الملفات المضغوطة (حتى 1 GiB لكل ملف). ولا يمكن قراءة الملفات المشفرة بـ EFS.

## لا تتغير الأدلة { #the-evidence-is-not-changed }

تُفتح الصورة للقراءة فقط ويرفض التركيب الكتابة. إذا لم تُغلق صورة VHDX بصورة سليمة، يُطبق سجلها المعلق في الذاكرة ولا يُكتب إلى الصورة. تترك بعض مكونات DiscUtils تعليمة القفز والتوقيع `55 AA` في قطاع إقلاع NTFS بالقيمة صفر؛ يقرأ Juntei هذه الوحدات كما هي.

## الاستخدام { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## تحليل الصور محليًا { #parsing-every-image-with-auto-parse }

يقرأ `auto-parse --vhdx` صور VHDX مباشرة على **Linux وmacOS وWindows**، دون تركيب عبر نظام التشغيل أو FUSE أو صلاحيات مسؤول.

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

تحتاج المحللات إلى مسارات ملفات، لذلك ينسخ Juntei الملفات التي يتيحها قارئ NTFS إلى مجلد مؤقت خاص. **يجب توفير مساحة مؤقتة كافية لهذه الملفات.** تبقى الصورة الأصلية للقراءة فقط وتُحذف النسخ عند انتهاء التشغيل بشكل طبيعي.

تُسجّل أخطاء القراءة والأسماء غير الآمنة في `ImageErrors` والصفوف الفاشلة في `manifest.csv`. تُحفظ النتائج القابلة للقراءة، وتكون حالة الخروج 1 عند وجود مدخلات غير مكتملة.

لا يزال أمر `mount` المنفصل، الذي يتيح الملفات للبرامج الأخرى، يتطلب Linux وFUSE 3 والوصول إلى `/dev/fuse`.

[English: implementation details and limitations](https://yamato-security.github.io/juntei/commands/auto-parse/#vhdx-disk-images).
