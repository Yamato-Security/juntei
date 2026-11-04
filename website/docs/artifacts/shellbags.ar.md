# ShellBags { #shellbags }

## ما هي ShellBags { #what-shellbags-are }

يتذكر Explorer كيفية عرض كل مجلد يفتحه المستخدم: طريقة العرض، وحجم الأيقونات، وترتيب الفرز، وموضع
النافذة. وللعثور على إعدادات المجلد مجددًا، يحتفظ بهوية المجلد في سجل Windows الخاص بالمستخدم على هيئة
شجرة من عناصر الواجهة (shell items)، وهي العناصر نفسها التي تحتويها قائمة معرّفات الهدف في الارتباط:
محرك أقراص، ثم كل مجلد نزولًا حتى المجلد الذي فُتح. وهذه الأشجار هي ShellBags.

وهي تغطي المجلدات التي فُتحت في Explorer وفي مربعات حوار الفتح والحفظ، بما في ذلك المجلدات الموجودة على
محركات الأقراص القابلة للإزالة، ومشاركات الشبكة، والأجهزة المحمولة (الهواتف، والكاميرات)، وأرشيفات
ZIP، وخوادم FTP، وصفحات Control Panel. وتبقى بعد زوال المجلد أو محرك الأقراص أو مشاركة الشبكة.

<span class="evidence-strength">أفضل استنتاج:</span> عرض Explorer الخاص بهذا المستخدم هذا المجلد
(أو مجلدًا تحته). ولا تُظهر ShellBag ما جرى في المجلد، ولا تؤرّخ كل زيارة: انظر الأوقات أدناه.

## الموقع { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

يحتوي مفتاح `BagMRU` على قيم مرقّمة، كل منها عنصر واجهة واحد، وعلى قيمة `MRUListEx` تحدد ترتيبها،
والأحدث فتحًا أولًا. ولكل قيمة مفتاح فرعي يحمل الاسم نفسه للعناصر الواقعة تحتها، لذا فالمسار هو العناصر
من `BagMRU` نزولًا حتى قيمة ما. ويمثّل `NodeSlot` في المفتاح الفرعي رقم مفتاح `Bags\<N>` الخاص به، حيث
توجد إعدادات عرض المجلد.

اجمع `UsrClass.dat` و`NTUSER.DAT` مع سجلات المعاملات الخاصة بهما (`.LOG1`، `.LOG2`) لكل ملف تعريف،
وخلية `SOFTWARE` الخاصة بالنظام، التي توفر معرّف SID لمستخدم `NTUSER.DAT`.

## أي الأوقات يدل على ماذا { #which-times-say-what }

- **أوقات عنصر الواجهة** (`ItemModifiedUTC`، `ItemCreatedUTC`، `ItemAccessedUTC`) هي أوقات نظام
  الملفات الخاصة بالمجلد نفسه (بدقة FAT: ثانيتان) حين كتب Explorer العنصر. وهي تؤرّخ المجلد، لا
  الزيارة، ولا تُحدَّث في الزيارات اللاحقة.
  لا تخزن ساعات FAT إزاحة UTC. وعناوين `UTC` القديمة واللاحقة `Z` المعروضة اصطلاحات عرض، وليست دليلًا
  على المنطقة الزمنية للمصدر. يحافظ `auto-parse --timezone` على هذه الحقول؛ ففسّرها في ضوء سياق النظام
  المصدر.
- **وقت آخر كتابة للمفتاح** يتغير عند إعادة كتابة `MRUListEx` الخاص به، أي عند فتح أحد عناصره ونقله
  إلى المقدمة. لذا فإن وقت المفتاح الذي توجد فيه قيمة العنصر (`ParentKeyLastWrittenUTC`) يمثّل آخر
  مرة فُتح فيها العنصر **فقط عندما يكون العنصر الأحدث في ذلك المفتاح** (`IsMostRecentInParent` بقيمة
  true، و`MruPosition` بقيمة 0). أما بالنسبة للعناصر الأخرى، فهو وقت لاحق، حين فُتح عنصر شقيق.
- يؤرّخ المفتاح الخاص بالعنصر نفسه (`KeyLastWrittenUTC`) فتح أحدث عنصر فرعي له بالطريقة نفسها.

## تحذيرات للمحلل { #analyst-cautions }

- **الغياب لا يثبت الكثير.** يكتب Explorer ShellBags عند حفظ طريقة عرض المجلد؛ فالمجلد الذي شوهد
  لوقت قصير فقط، أو فُتح بطريقة أخرى (سطر أوامر، أو نص برمجي، أو برنامج)، قد لا يكون له أي منها. وقد
  يحذف المستخدم المفاتيح.
- **مرجع NTFS هو مرجع المجلد** (`MftEntry`، `MftSequence`) حين كُتب العنصر. ابحث عنه في `$MFT` لمعرفة
  ما إذا كان المجلد نفسه لا يزال موجودًا، أو أن ملفًا آخر أعاد استخدام الإدخال.
- **محركات الأقراص القابلة للإزالة** تُسمّى بحرف محرك الأقراص. والحرف هو الذي كان لمحرك الأقراص حينها؛
  فطابقه مع سجل USB والأجهزة المركّبة لمعرفة الجهاز.
- **الأسماء تأتي من العنصر.** يحتفظ المجلد الذي أُعيدت تسميته لاحقًا باسمه القديم هنا. وتُسمّى المجلدات
  الجذرية والمجلدات المعروفة كما تسمّيها الأدوات الجنائية عادةً (`This PC`، `Quick Access`،
  `Downloads`)؛ ويعطي `Guid` معرّفها.
- **نتائج البحث والمكتبات وصفحات Control Panel** طرق عرض، وليست مجلدات على القرص
  (`property_view`، `control_panel_item`، `control_panel_category`).
- **العنصر الذي لا يعرفه أي محلل** يحتفظ ببايتاته في `DataRaw`، مع `ItemType` بقيمة `unknown` وبايت
  الفئة الخاص به في `ItemClass`. ويُبلَغ عن العناصر التالفة أو غير المعتادة في `Problems`، وتُحفظ
  بايتاتها أيضًا.
- **الخلية غير النظيفة (dirty)** قد تفتقد أحدث عناصرها دون سجلاتها. يعيد Juntei تطبيق السجلات الموجودة
  بجوار الخلية ما لم يُستخدم `--no-logs`؛ ويبيّن `TransactionLogStatus` ما إذا كان السجل قد جاء منها.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shell-bags -d evidence/ -o shellbags.csv
juntei parse-registry shell-bags -f UsrClass.dat -t jsonl \
  | jq 'select(.ItemType == "volume" and .GUID == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

يُفك ترميز عناصر الواجهة باستخدام ys-lnk-parser، وهو محلل عناصر الواجهة المستخدم في `parse-file lnk`.
ويسرد الملخص المجلدات التي فُتحت أخيرًا (الأحدث أولًا)، استنادًا إلى وقت آخر كتابة لمفتاحها الأب، مع
مستخدميها.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `User`، `UserSID`، `SourcePath`، `HiveType` | المضيف، واسم مجلد ملف التعريف، ومعرّف SID الخاص بالمستخدم (يسمّيه المفتاح الجذر في `UsrClass.dat`؛ وبالنسبة لـ `NTUSER.DAT`، ‏`ProfileList` في النظام)، والخلية ونوعها. |
| `BagKey` | مفتاح العنصر، تحت جذر الخلية (`…\BagMRU\4\3\0`): وتوجد قيمته في المفتاح الذي يعلوه. |
| `MRU-Position`، `IsMostRecentInParent` | موضعه في `MRUListEx` الخاص بمفتاحه الأب (0: الأحدث فتحًا)؛ ويكون فارغًا عندما لا تتضمنه القائمة. |
| `NodeSlot` | رقم مفتاح `Bags\<N>` الخاص به (إعدادات عرض المجلد). |
| `Path` | العناصر من قمة شجرته نزولًا إليه، موصولة بـ `\`. ويرمز `[unknown]` إلى عنصر قيمته مفقودة. |
| `ItemType`، `ItemClass` | ماهية العنصر (`root_folder`، `volume`، `directory`، `file`، `network_location`، `uri`، `ftp_folder`، `compressed_folder`، `portable_device`، `portable_device_folder`، `users_files_folder`، `property_view`، `control_panel_item`، `control_panel_category`، `cd_burn`، `unknown`)، وبايت الفئة الخاص به. |
| `ItemName`، `GUID` | اسمه (الاسم الطويل للمجلد)، ومعرّف الفئة لمجلد جذري أو عنصر Control Panel أو مجلد مفوَّض (delegate folder). |
| `ItemModifiedUTC`، `ItemCreatedUTC`، `ItemAccessedUTC` | **من عنصر الواجهة:** أوقات المجلد حين كُتب العنصر (FAT، بدقة ثانيتين). |
| `MFT-Entry`، `MFT-Sequence` | **من عنصر الواجهة:** مرجع ملف NTFS الخاص بالمجلد. |
| `ExtensionBlocks` | عدد كتل الامتداد في العنصر. |
| `KeyLastWrittenUTC` | **من سجل Windows:** آخر كتابة للمفتاح الخاص بالعنصر نفسه: حين فُتح أحدث عنصر فرعي له. |
| `ParentKeyLastWrittenUTC` | **من سجل Windows:** آخر كتابة للمفتاح الذي توجد فيه قيمته: حين فُتح آخر مرة، إذا كان `IsMostRecentInParent` بقيمة true. |
| `DataRaw` | بايتات القيمة (بالنظام الست عشري)، لعنصر غير مفهوم بالكامل أو فيه مشكلات. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows. |
| `Problems` | قيمة يسمّيها `MRUListEx` الخاص بمفتاحها لكن المفتاح لا يحتويها، أو قيمة لا تسمّيها القائمة، أو مفتاح فرعي دون قيمته، أو قيمة ليست ثنائية، أو `NodeSlot` من النوع الخاطئ، أو قائمة عناصر واجهة تالفة. |

## استراتيجية التحقيق { #investigation-strategy }

1. اسرد محركات الأقراص القابلة للإزالة (`volume` مع `Guid` الخاص بـ Removable Drives)، ومواقع الشبكة،
   ومجلدات FTP، والأجهزة المحمولة، والمجلدات الواقعة تحتها.
2. ابحث عن المجلدات المثيرة للاهتمام: أماكن التجهيز القابلة للكتابة من قِبل المستخدم (`Temp`،
   `ProgramData`، `Public`)، ومجلدات الأدوات والأرشيفات، وملفات تعريف المستخدمين الآخرين، والمجلدات
   التي لم تعد موجودة (افحص مرجع NTFS في `$MFT`).
3. لا تؤرّخ الزيارات إلا باستخدام `ParentKeyLastWrittenUTC` للعناصر التي يكون فيها `IsMostRecentInParent`
   بقيمة true؛ واربط البقية مع ملفات LNK وJump Lists وRecentDocs، التي تؤرّخ فتح الملفات.
4. استخدم `parse-registry read --recover-deleted` لمفاتيح وقيم `BagMRU` المحذوفة.

## مزيد من التفاصيل { #further-details }

- [libyal libfwsi: تنسيق Windows Shell Item](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb: Most recently used ‏(BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

يتطلب البحث عن معرّف SID من `ProfileList` معرّف SID واحدًا فريدًا وصالحًا من حيث الصياغة. وتترك
التعيينات الملتبسة لمجلدات ملفات التعريف ومفاتيح النسخ الاحتياطي مثل `S-1-...bak` قيمة `UserSid` غير
معروفة.
