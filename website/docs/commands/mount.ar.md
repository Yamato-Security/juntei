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

## تحليل كل صورة باستخدام auto-parse { #parsing-every-image-with-auto-parse }

يحلل `auto-parse --vhdx` أنظمة Windows الموجودة داخل كل صورة قرص VHDX في مجلد الأدلة (أو الصورة الواحدة
المعطاة عبر `-d`)، دون تحميل منفصل:

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

- تُحمَّل كل صورة للقراءة فقط في مجلد مؤقت جديد، داخل مجلد يحمل اسم الصورة (`host1-triage`)، وتُحلَّل
  الأنظمة التي يُعثر عليها هناك كما في مجموعة موجودة على القرص. ويُلغى تحميل الصور في النهاية، وعند الضغط
  على Ctrl+C.
- تسرد الطرفية الصور وصورة كل نظام:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`.
- في البيان، يمثل `EvidenceDirectory` المجلد المعطى، ويسمّي `Image` الخاص بكل نظام صورته. ويستخدم
  `EvidenceRoot` وأوامر المحللات مجلد التحميل (`./host1-triage/C`).
- إذا احتوت الصورة على ملفات من Volume Shadow Copies ضمن مجلدات مثل `VSS32`، يُكتشف كل مجلد ويُحلل بوصفه نظامًا آخر (`HOST1_2`، …).
- يُبلَّغ عن الصورة التي يتعذر تحميلها؛ وتظل الصور الأخرى تُحلَّل، وينتهي التشغيل بحالة الخروج 1.

إذا تعذر فتح وحدة NTFS، يبلغ Juntei عن المشكلة ويحتفظ بالوحدات القابلة للقراءة. تعيد الصور القابلة للقراءة جزئيًا رمز الخروج 1 مع `auto-parse --vhdx`، ويعيد `mount` أيضًا الرمز 1 بعد إلغاء التركيب.

## المتطلبات { #requirements }

- Linux: يلزم FUSE 3 ‏(`fusermount3`، والحزمة `fuse3`) وإمكانية الوصول إلى `/dev/fuse`، وهي متاحة
  عادةً للمستخدمين العاديين. ولا حاجة إلى صلاحيات root.
- لا يُدعم Windows ولا macOS. وعلى Windows، أرفق الصورة للقراءة فقط بدلًا من ذلك في جلسة PowerShell بصلاحيات
  المسؤول (elevated)، ثم أعطِ الأدوات حرف محرك الأقراص الخاص بها:

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
