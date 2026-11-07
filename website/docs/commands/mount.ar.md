# تحميل صور الأقراص { #mounting-disk-images }

يتيح `juntei mount` قراءة الملفات الموجودة داخل صورة قرص VHDX من مجلد، للقراءة فقط، طوال مدة
تشغيله. ويمكن عندئذ فحص مجموعات KAPE المأخوذة بالخيار `--vhdx` وأقراص Hyper-V بأي أداة، مثل Hayabusa
أو أوامر Juntei نفسها، دون استخراج أي شيء، ودون صلاحيات root، ودون الحاجة إلى أن يدعم نظام التشغيل VHDX
أو NTFS. ويستخدم الأمر FUSE ويعمل على Linux. تُصان مكتبة القراءة ومكتبة التركيب في [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser).

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

- ملفات وحدة تخزين NTFS عند نقطة التحميل (mount point): ففي مجموعة KAPE، يظهر المجلد `C` الذي يضم
  الملفات المجمعة، ومجلد `LongFileNames` الخاص بـ KAPE، وملفات البيانات الوصفية الخاصة بوحدة التخزين نفسها
  مثل `$MFT`. وتعرض الصورة التي تحتوي على عدة وحدات تخزين NTFS هذه الوحدات باسم `Volume1` و`Volume2`
  وهكذا.
- كل ملف للقراءة فقط ويعود إلى مالك نقطة التحميل. وأوقات الملفات هي أوقات `$STANDARD_INFORMATION`
  في NTFS.
- تُسرد الأسماء كما يخزنها NTFS، ويُبحث عنها بمطابقة تامة. ولا تُعرض أسماء DOS 8.3 ولا دفقات البيانات
  البديلة.
- يُفك ضغط الملفات المضغوطة (حتى 1 GiB لكل ملف). ولا يمكن قراءة الملفات المشفرة بـ EFS.

## لا تتغير الأدلة { #the-evidence-is-not-changed }

تُفتح الصورة للقراءة فقط، ويرفض التحميل أي كتابة. ويُقرأ ملف VHDX الذي لم يُغلق بشكل سليم مع تطبيق سجله
المعلّق في الذاكرة، ولا تُكتب النتيجة إلى الملف أبدًا. ويكتب KAPE ملفات VHDX الخاصة به باستخدام DiscUtils، الذي
يترك قطاع الإقلاع في NTFS دون تعليمة القفز ودون التوقيع `55 AA`؛ ويقرأ Juntei وحدات التخزين هذه كما هي.

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
- يخزن KAPE الملفات التي ينسخها من Volume Shadow Copies في مجلدات مثل `VSS32`. ويُعثر على كل منها
  ويُحلَّل بوصفه نظامًا آخر (`HOST1_2`، …).
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
