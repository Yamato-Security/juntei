# قطاع الإقلاع (`$Boot`) { #boot-sector-boot }

## ما هو `$Boot` { #what-boot-is }

`$Boot` هو إدخال MFT رقم 7 في كل وحدة تخزين NTFS: أول 8 KiB من وحدة التخزين. ويحتوي قطاعه الأول،
وهو **سجل إقلاع وحدة التخزين** (volume boot record)، على كتلة معلمات BIOS ‏(BIOS parameter block) التي
تصف نظام الملفات:

- أحجام القطاعات والعناقيد (clusters)، وعدد القطاعات في وحدة التخزين، و**القطاعات المخفية** (hidden
  sectors: القطاعات الموجودة على القرص قبل وحدة التخزين، أي حيث يبدأ قسمها)؛
- مواضع بداية `$MFT` و`$MFTMirr`، وأحجام سجلات FILE وINDX؛
- **الرقم التسلسلي لوحدة التخزين** بطول 64 بت، الذي يُضبط عند تهيئة وحدة التخزين.

ويحتوي باقي القطاع الأول والقطاعات الخمسة عشر التي تليه على **شيفرة الإقلاع** (boot code) التي تحمّل
Windows.

البتات الـ 32 الدنيا من الرقم التسلسلي هي ما يعرضه Windows بصيغة `XXXX-XXXX` (`vol`، `dir`)، وهي ما
تسجله [اختصارات LNK](lnk.md) و[Jump Lists](jump-lists.md) و[Prefetch](prefetch.md) عن وحدة التخزين
التي كان عليها الملف. ويربط `$Boot` تلك السجلات بوحدة تخزين من مضيف جرى جمعه.

<span class="evidence-strength">أفضل استنتاج:</span> الرقم التسلسلي لوحدة التخزين في LNK أو Jump List
أو Prefetch المساوي لقيمة `VolumeSerialNumber` في `$Boot` يشير إلى وحدة التخزين تلك، أو إلى وحدة
تخزين أخرى أُعطيت الرقم التسلسلي نفسه أو نُسخت معه. ويمكن تغيير الأرقام التسلسلية، وهي ليست فريدة عبر
المضيفين.

## الموقع والجمع { #location-and-collection }

```text
C:\$Boot          (MFT entry 7; the first 8 KiB of the volume)
```

اجمعه بأداة جنائية قادرة على القراءة الخام، أو اقرأ القطاع الأول من صورة وحدة التخزين: فكلاهما البايتات
نفسها. يقرأ Juntei أي ملف يبدأ بقطاع إقلاع NTFS، لذا تصلح صورة وحدة التخزين مُدخلًا. أما صورة القرص
فتبدأ بجدول الأقسام بدلًا من ذلك؛ فأعطِ Juntei وحدة التخزين الخاصة بالقسم.

## تحذيرات للمحلل { #analyst-cautions }

- يتغير الرقم التسلسلي عند تهيئة وحدة التخزين. وقد يشير ملف LNK لا يطابق رقمه التسلسلي أي وحدة تخزين
  حالية إلى وسائط قابلة للإزالة، أو إلى خادم مشاركة شبكية، أو إلى وحدة تخزين جرت تهيئتها منذ ذلك الحين.
- تكون شيفرة الإقلاع متطابقة على وحدات التخزين التي هيّأها إصدار Windows نفسه: فهي متطابقة في ملفات
  `$Boot` الخاصة بـ Windows 10 و11 وServer 2012 R2 في DFIR Artifact Museum. ويغيّرها bootkit يستهدف سجل
  إقلاع وحدة التخزين، أو يغيّر حقل القطاعات المخفية الذي تستخدمه شيفرة الإقلاع للعثور على بقية
  `$Boot`. قارن `BootCodeSHA256` و`JumpInstruction` و`HiddenSectors` مع المضيفين الآخرين ومع جدول الأقسام
  قبل استنتاج أي شيء: فقد يغيّر مدير إقلاع أو برنامج تشفير شيفرة الإقلاع أيضًا.
- تبدأ صورة وحدة تخزين مشفرة بـ BitLocker بقطاع الإقلاع الخاص بـ BitLocker (معرّف OEM هو
  `-FVE-FS-`). وعند تمريرها باستخدام `-f`، يُبلغ Juntei عنها، ويذكر `Problems` أنها ليست NTFS؛ إذ يوجد
  قطاع إقلاع NTFS ضمن البيانات المشفرة. أما `$Boot` المجموع من وحدة التخزين بعد إلغاء قفلها فهو قطاع
  NTFS.
- يسرد `Problems` حالات عدم الاتساق البنيوي والاقتطاع المعروفة، ولا يوفر اختبارًا كاملًا لقابلية
  التركيب (mount) في Windows. استخدمه لتقييم القطاعات المستخرجة بالاقتطاع (carving) والنسخ، لا بوصفه
  دليلًا على هجوم في حد ذاته. تستثني تجزئة شيفرة الإقلاع تعليمة القفز الأولى، وكتلة معلمات BIOS،
  وتوقيع النهاية؛ فافحص تلك الحقول أيضًا.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      Parse one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

يسرد الملخص لكل قطاع إقلاع الرقم التسلسلي لوحدة التخزين (كما يعرضه Windows، وكما تعرضه حقول LNK
وPrefetch)، والحجم، وحجم العنقود، وعنقود `$MFT`. وعند تحليل عدة مضيفين، يجمّعهم حسب تجزئة شيفرة
الإقلاع، بحيث تبرز وحدة التخزين التي تختلف شيفرة إقلاعها عن البقية.

## حقول إخراج CSV { #csv-output-fields }

| الحقل | التفسير |
|---|---|
| `SourcePath` | ملف الإدخال. |
| `VolumeSerialNumber` | الرقم التسلسلي بطول 32 بت، كما يعرضه إخراج LNK وJump List وPrefetch ‏(`180B1E67`). ويعرضه Windows بصيغة `180B-1E67`. |
| `VolumeSerialNumber64` | الرقم التسلسلي الكامل بطول 64 بت. |
| `OEM-ID` | `NTFS` على وحدة تخزين NTFS. |
| `BytesPerSector`، `SectorsPerCluster`، `ClusterSize` | الهندسة الخاصة بوحدة التخزين. يُسجَّل عدد القطاعات لكل عنقود الذي يتجاوز 128 بوصفه أُسًّا للعدد 2، ويُفك ترميزه هنا. ويكون فارغًا عندما لا يكون صالحًا. |
| `TotalSectors`، `VolumeSize` | عدد القطاعات في وحدة التخزين وحجمها بالبايت، دون نسخة قطاع الإقلاع الموجودة في نهايتها. |
| `HiddenSectors` | القطاعات الموجودة على القرص قبل وحدة التخزين: حيث يبدأ قسمها. |
| `MFT-Cluster`، `MFT-Offset` | موضع بداية `$MFT`، بالعناقيد وبالبايتات من بداية وحدة التخزين. |
| `MFT-MirrorCluster`، `MFT-MirrorOffset` | موضع بداية `$MFTMirr`. |
| `FileRecordSize`، `IndexRecordSize` | عدد البايتات لكل سجل FILE (عادةً 1,024) ولكل سجل INDX (عادةً 4,096). |
| `MediaDescriptor` | `F8` للقرص الثابت. |
| `JumpInstruction` | البايتات الثلاثة الأولى (`EB 52 90`): قفزة فوق كتلة معلمات BIOS إلى شيفرة الإقلاع. |
| `EndSignature` | `55 AA` في نهاية قطاع إقلاع صالح. |
| `BootCodeSHA256` | تجزئة SHA-256 لشيفرة الإقلاع: البايتات من 0x54 إلى 0x1FD في القطاع الأول وبقية الـ 8 KiB. |
| `Problems` | ما لن يركّبه Windows: معرّف OEM غير `NTFS`، أو غياب `55 AA`، أو أحجام غير صالحة، أو حقول FAT التي يبقيها NTFS صفرية وقد ضُبطت، أو `$MFT` بعد نهاية وحدة التخزين؛ وكذلك ملف أقصر من 8 KiB. |

يحتوي JSON/JSONL على الحقول والأسماء نفسها، مع الأرقام بوصفها أرقامًا، و`null` للقيم غير المعروفة،
و`Problems` بوصفه قائمة.

## استراتيجية التحقيق { #investigation-strategy }

1. ضع `VolumeSerialNumber` لكل مضيف بجوار الأرقام التسلسلية لوحدات التخزين في ملفات LNK وJump Lists
   وملفات Prefetch الخاصة به: فالأرقام التي لا تطابق أي وحدة تخزين مجموعة تخص وحدات تخزين أخرى
   (محركات USB، أو مشاركات، أو وحدات تخزين جرت تهيئتها منذ ذلك الحين).
2. ابحث عن الأرقام التسلسلية الأخرى في إخراج [أجهزة USB](usb.md)، الذي يسرد الأرقام التسلسلية لوحدات
   التخزين القابلة للإزالة التي جرى تركيبها.
3. قارن `BootCodeSHA256` و`HiddenSectors` عبر المضيفين. وافحص شيفرة الإقلاع المختلفة باستخدام أداة
   تفكيك (disassembler) وجدول أقسام القرص، وابحث عن إدخالات سجلات الأحداث المتعلقة بالإقلاع.
4. استخدم `MFT-Offset` و`ClusterSize` للعثور على `$MFT` في صورة وحدة تخزين.

## مزيد من التفاصيل { #further-details }

- [Microsoft: نظرة عامة على NTFS](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
- [libfsntfs: وثائق تنسيق NTFS](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (رأس وحدة التخزين)
- [The Sleuth Kit: fsstat](https://www.sleuthkit.org/sleuthkit/man/fsstat.html)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (حزمة `boot` الخاصة به)
