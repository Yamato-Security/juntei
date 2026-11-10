# واصفات الأمان (`$Secure:$SDS`) { #security-descriptors-securesds }

## ما هو `$SDS` { #what-sds-is }

لكل ملف ومجلد على NTFS **واصف أمان** (security descriptor): مالكه، ومجموعته الأساسية، و**DACL** (من
يحق له فعل ماذا به) و**SACL** (أي وصول يُدقَّق، وتسمية السلامة الخاصة به). يحتفظ NTFS بكل واصف مميز
مرة واحدة، في الدفق `$SDS` الخاص بالملف `$Secure` (إدخال MFT رقم 9)، ويعطيه **معرّف أمان** (security
ID). ولا يسجّل `$STANDARD_INFORMATION` الخاص بالملف إلا معرّف الأمان ذاك: العمود `SID` في
[`parse-ntfs mft`](mft.md). ويحوّله `$SDS` إلى مالك وقوائم ACL.

يتكون `$SDS` من كتل بحجم 256 KiB، تتبع كلًّا منها نسخةٌ منها (مرآتها). ولكل إدخال تجزئة لواصفه، يستخدمها
NTFS للعثور على واصف موجود مسبقًا (`$SDH`)، ومعرّف الأمان (الذي يفهرسه `$SII`).

<span class="evidence-strength">أفضل استنتاج:</span> كان للملفات التي يسمّي `SID` الخاص بها
واصفًا ما، عند آخر كتابة لسجل `$MFT` الخاص بها، ذلك المالك وتلك القوائم ACL. ومالك الملف هو عادةً
الحساب الذي أنشأه، لكن يمكن لمسؤول أو مثبّت أو برنامج يستولي على الملكية أن يغيّره.

## الموقع والجمع { #location-and-collection }

```text
C:\$Secure:$SDS   (MFT entry 9, the $SDS data stream)
```

اجمعه بأداة جنائية تدعم القراءة الخام؛ وعادةً ما تسمّيه المجموعات `$Secure_$SDS`، لأن `:`
لا يمكن أن يكون في اسم ملف. واجمع معه `$MFT` الخاص بوحدة التخزين نفسها.

## تحذيرات للمحلل { #analyst-cautions }

- المالك معرّف SID. ولا تُسمّى إلا معرّفات SID المعروفة (`NT AUTHORITY\SYSTEM`، `BUILTIN\Administrators`،
  `NT SERVICE\TrustedInstaller`، …) والحسابات المضمّنة في جهاز أو مجال (`-500`
  `Administrator`، …). حلّل المعرّفات الأخرى باستخدام `SAM` الخاص بالمضيف
  ([حسابات SAM](sam.md))، أو قائمة ملفات التعريف الخاصة به ([معلومات النظام](system-info.md))، أو المجال.
- يتشارك الواصفَ كلُّ ملف له المالك نفسه والقوائم ACL نفسها، ويبقى عادةً في `$SDS`
  بعد حذف تلك الملفات. والواصف الذي لا ملفات له في `$MFT` ليس في حد ذاته ملفًا محذوفًا.
- إدخال ACE ليس سجلًا للوصول. يبيّن DACL من *يحق له* الوصول إلى ملف؛ أما سجلات الأحداث والآثار الرقمية
  الأخرى فتبيّن من وصل إليه فعلًا. وتصف إدخالات التدقيق في SACL التكوين، لا ما سُجّل.
- تُدرج إدخالات ACE بترتيبها في ACL. ولا يحسب Juntei الوصول الفعلي: إذ تُحتسب كلها، إدخالات الرفض،
  والوراثة، وعضوية المجموعات، والامتيازات، والحقوق الضمنية للمالك.
- يكتب `DACL` و`SACL` كل إدخال ACE كما يفعل `icacls`. ويمثّل `SDDL` الأذونات المدعومة، مع الأقنعة
  بالنظام الست عشري والحسابات بصيغة معرّفات SID. وهو ليس نسخة مطابقة بايتًا ببايت للواصف الثنائي؛ فالنص
  الشرطي غير القابل للتمثيل يتركه فارغًا ويُشخَّص.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      Parse one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

يعدّ الملخص الواصفات ويسرد مالكيها، مع عدد الواصفات التي يملكها كل منهم، والواصفات ذات DACL الفارغ
(NULL DACL) (أي بلا أي تقييد على الإطلاق)، وأي تلف.

## حقول إخراج CSV { #csv-output-fields }

| الحقل | التفسير |
|---|---|
| `SourcePath` | الملف المُدخل. |
| `SecurityID` | معرّف الأمان: اربطه بـ `SID` في [`parse-ntfs mft`](mft.md) لوحدة التخزين نفسها. |
| `Hash` | التجزئة المسجلة مع الواصف. |
| `EntryOffset` | موضع الإدخال في الدفق. |
| `FromMirror` | `True` لنسخة مقروءة من كتلة المرآة: نسخة تختلف عن إدخالها (مدرجة بعده)، أو نسخة إدخال تالف. |
| `Size` | حجم الإدخال، بما في ذلك ترويسته البالغة 20 بايت. |
| `Owner`، `OwnerName` | معرّف SID للمالك، واسمه إذا كان معروفًا. |
| `Group`، `GroupName` | معرّف SID للمجموعة الأساسية، واسمها إذا كانت معروفة. ولا يستخدمها Windows في فحوص الوصول. |
| `Control` | علامات التحكم الخاصة بالواصف: `SE_DACL_PRESENT`، `SE_DACL_PROTECTED` (الوراثة محظورة)، `SE_DACL_AUTO_INHERITED`، … مفصولة بـ `|`. |
| `DACL-ACE-Count`، `DACL` | إدخالات DACL، مفصولة بـ `; `. يكون فارغًا إذا لم يوجد DACL؛ و`NULL DACL: no restriction` إذا كان الواصف يمنح الجميع كل شيء. |
| `SACL-ACE-Count`، `SACL` | إدخالات SACL: إدخالات التدقيق، وتسمية السلامة الإلزامية، وتسميات ثقة العمليات، وسمات الموارد. |
| `SDDL` | الواصف بأكمله بصيغة SDDL. ولا يكون فارغًا إلا إذا كان فيه شيء لم يُفك ترميزه. |
| `Problems` | تجزئة لا تخص الواصف، أو أجزاء يتعذر فك ترميزها، أو إدخالات ACE نوعها غير معروف. |

يُكتب كل إدخال في `DACL` و`SACL` كما يكتبه `icacls`: الحساب، والنوع إن لم يكن إدخال سماح، وعلامات
الوراثة، والحقوق.

| الجزء | المعنى |
|---|---|
| الحساب | الاسم المعروف، أو معرّف SID. |
| `(DENY)`، `(AUDIT)`، `(ACCESS_ALLOWED_CALLBACK)`، … | نوع ACE، عندما لا يكون إدخال سماح. ولا يُعرض نوع لتسمية إلزامية. |
| `(I)` | موروث من المجلد الأم. |
| `(OI)`، `(CI)`، `(IO)`، `(NP)` | ترثه الملفات (object inherit)، وترثه المجلدات (container inherit)، وترثه هي فقط دون هذا الكائن (inherit only)، ولمستوى واحد فقط إلى الأسفل (no propagate). |
| `(SA)`، `(FA)` | إدخال تدقيق للوصول الناجح أو الفاشل. |
| `(F)`، `(M)`، `(RX)`، `(R)`، `(W)` | تحكم كامل، وتعديل، وقراءة وتنفيذ، وقراءة، وكتابة. |
| `(RD,WD,AD,…)` | حقوق محددة فيما عدا ذلك: قراءة البيانات، وكتابة البيانات، وإلحاق البيانات، وقراءة السمات الموسعة وكتابتها (`REA`، `WEA`)، والتنفيذ (`X`)، وحذف العنصر الفرعي (`DC`)، وقراءة السمات وكتابتها (`RA`، `WA`)، والحذف (`DE`)، وقراءة التحكم (`RC`)، وكتابة DAC (`WDAC`)، وكتابة المالك (`WO`)، والمزامنة (`S`)، والحقوق العامة `GA`، `GR`، `GW`، `GE`. و`N` تعني عدم الوصول. |
| `(NW)`، `(NR)`، `(NX)` | نهج التسمية الإلزامية: no write up، وno read up، وno execute up. |
| `if (…)` | شرط إدخال ACE من نوع callback، مثل `(WIN://SYSAPPID Contains "…")`: لا ينطبق الإدخال إلا على العمليات التي يكون الشرط صحيحًا لها. ويستخدمها Windows لمنح حزم app container حق الوصول. |

تُظهر تسميات ثقة العمليات وسمات الموارد قناعها بالنظام الست عشري، وتُظهر سمة المورد مطالبتها (claim)
(`("IMAGELOAD",TU,0x0,1)`).

تحتوي JSON/JSONL على الحقول والأسماء نفسها، مع الأرقام بوصفها أرقامًا، و`null` للقيم الفارغة، و
`Problems` بوصفها قائمة.

## استراتيجية التحقيق { #investigation-strategy }

1. حلّل `$MFT` الخاص بوحدة التخزين نفسها واربط `SID` الخاص به بالحقل المقابل في هذا الإخراج:
   فيحصل كل ملف على مالكه وقوائم ACL الخاصة به.
2. انظر في المالكين الذين هم حسابات مستخدمين (`S-1-5-21-…`): ما الملفات التي يملكها حساب مريب، وما
   الحسابات التي تملك ملفات في مجلدات النظام.
3. ابحث عن قوائم ACL متساهلة على المجلدات التي تعمل منها البرامج أو تُحمَّل منها: `Everyone` أو
   `Authenticated Users` أو `User` مع `(F)` أو `(M)` أو `(W)`، وعن DACL الفارغة (NULL DACL).
4. الواصف الذي لا يستخدمه إلا ملف واحد، أو مجموعة غير معتادة من الملفات، قد يكشف ملفات أُنشئت أو
   غُيّرت خارج التثبيت العادي.
5. عزّز تغييرات الملكية والأذونات بإدخالات سجل أحداث Security (4670، و4663 مع
   SACL) وبسجلات `SECURITY_CHANGE` في [دفتر يومية USN](usn-journal.md).

## مزيد من التفاصيل { #further-details }

- [Microsoft: واصفات الأمان](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [المواصفات المفتوحة من Microsoft: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) (واصفات الأمان، وإدخالات ACE، والتعبيرات الشرطية)
- [Microsoft: تنسيق سلسلة واصف الأمان (SDDL)](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: icacls](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls)
- [libfsntfs: توثيق تنسيق NTFS](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (`$Secure`)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (حزمة `secure` الخاصة به)

مع الإخراج المنظّم على stdout (`-t` دون `-o`)، تذهب تحذيرات الاسترداد على مستوى الدفق إلى stderr، بحيث
تحتفظ الأنابيب (pipes) بإخراج CSV/JSON/JSONL مقروء دون إخفاء المناطق التالفة أو المستردة.
