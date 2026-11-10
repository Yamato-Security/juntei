# نهج التدقيق { #audit-policy }

## ما هو { #what-it-is }

يحدد نهج التدقيق أحداث الأمان التي يسجلها Windows في سجل أحداث Security. ويُضبط لكل فئة فرعية على
Windows Vista والإصدارات الأحدث (59 فئة فرعية على Windows 10 و11)، ولكل فئة على Windows XP وServer
2003. ويسجل كل إعداد حالات النجاح، أو حالات الفشل، أو كلتيهما، أو لا شيء.

تحتفظ LSA بالنهج الساري في خلية `SECURITY`، سواء جاء من `auditpol` أو من نهج الأمان المحلي أو من
نهج المجموعة (Group Policy).

<span class="evidence-strength">أفضل استنتاج:</span> كان النظام مضبوطًا على تسجيل هذه الأحداث (أو عدم
تسجيلها) عند آخر كتابة للخلية. وعندما لا تخضع فئة فرعية للتدقيق، فإن غياب الحدث لا يدل على شيء بشأن
ما إذا كان النشاط قد حدث.

## الموقع { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

يحتفظ Windows Vista والإصدارات الأحدث برأس من 12 بايت، ثم بإعداد واحد لكل فئة فرعية، فئةً تلو
الأخرى، ثم بعدد الفئات الفرعية في كل فئة. ولا تختلف الإصدارات إلا في هذه الأعداد: إذ يضيف كل إصدار
فئات فرعية في نهاية فئة ما (56 في خلايا Windows 8.x وServer 2012 R2 التي قُرئت، و59 في Windows 10 و11
وServer 2022). أما Windows XP وServer 2003 فيحتفظان بما إذا كان التدقيق مفعّلًا (بايت واحد، يليه حشو
لا يمسحه Windows) وبإعدادات الفئات التسع. وقد جرى التحقق من هذا التخطيط على صورة Windows XP العامة
الخاصة بحالة Hacking Case في CFReDS من NIST، وهي لا تدقق أي شيء؛ أما قيم إعدادات الفئات فيها فتتبع
وثائق Microsoft.

اجمع `SECURITY` مع سجلات المعاملات الخاصة به، وخلية `SYSTEM` الخاصة بالنظام، التي تحدد اسم المضيف.

## تحذيرات للمحلل { #analyst-cautions }

- **البيانات غير المعروفة تبقى غير معروفة.** تُحفظ المراجعات غير المدعومة مع معلومات تشخيصية. وتترك
  علامات الإعداد غير المعروفة `Success` و`Failure` فارغتين وتُستبعد من أعداد الإعدادات الخاضعة
  للتدقيق. ولا يمكن لجدول أعداد تالف أن يرقّي الكلمة المحجوزة إلى إعداد تدقيق آخر.
  قد تحتوي البيانات الملتقطة على خانات إضافية تتجاوز فهرس الأسماء/معرّفات GUID. وتُحفظ إعداداتها
  و`SubcategoryId` الرقمي الخاص بها مع أسماء/معرّفات GUID فارغة؛ والخانة غير المعيّنة ليست في حد ذاتها
  دليلًا على التلف.


- **الترتيب هو ترتيب Windows نفسه.** داخل الفئة الواحدة، تأتي الإعدادات بترتيب معرّفات الرسائل الخاصة
  بالفئات الفرعية، لا بترتيب معرّفات GUID الخاصة بها. فـ Special Logon (`%%12548`، الفهرس 4 في
  Logon/Logoff) يأتي قبل IPsec Quick Mode. والأدوات التي تقرؤها بترتيب GUID تُبلغ عن الإعداد
  الافتراضي في Windows 10 بأن Special Logon هو "No Auditing" وأن IPsec Quick Mode هو "Success". ويمثّل
  `SubcategoryId` المعرّف نفسه الذي يعطيه الحدث 4719 (System audit policy was changed)، لذا يمكن
  المطابقة بينهما.
- **النهج الساري، لا مصدره.** يكتب نهج المجموعة والنهج المحلي القيمة نفسها. ويُظهر ملف `audit.csv`
  الخاص بنهج المجموعة (في SYSVOL و`C:\Windows\System32\GroupPolicy`) ما كان يُراد تطبيقه
  (`Machine\Microsoft\Windows NT\Audit\audit.csv` الخاص بنهج ما).
- **التغييرات عبر الزمن:** يشير `KeyLastWrittenUTC` تقريبًا إلى وقت آخر تغيير للنهج. ويسجل الحدث 4719
  كل تغيير (ما دام سجل Security يحتفظ به). ويمكن للمهاجم إيقاف التدقيق، ثم إعادة تشغيله.
- **الفئات الفرعية مقابل الفئات.** على Windows Vista والإصدارات الأحدث، الفئات الفرعية هي ما يستخدمه
  Windows (ما لم يكن خيار الأمان `SCENoApplyLegacyAuditPolicy` معطّلًا ويتجاوزها نهج فئات قديم).
- **Windows XP و2003:** عندما يكون التدقيق معطّلًا (`AuditingEnabled` بقيمة False)، لا تخضع أي فئة
  للتدقيق، أيًا كان إعدادها.
- **الأحداث:** يسرد `Events` الأحداث الرئيسية لكل فئة فرعية، استنادًا إلى وثائق Microsoft الخاصة
  بإعدادات نهج التدقيق المتقدم. وهي ما يبحث عنه التحقيق عادةً، لا كل حدث تسجله الفئة الفرعية.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.Setting == "none") | .Subcategory'
```

لكل خلية، يعرض الملخص تخطيطها، وعدد الإعدادات الخاضعة للتدقيق، ووقت آخر كتابة للنهج. ويسرد ما يخضع
للتدقيق عند النجاح والفشل، وعند النجاح فقط، وعند الفشل فقط، ثم الفئات الفرعية التي لا تخضع للتدقيق،
مع أحداثها الرئيسية: هذه الأحداث معطّلة في النهج الملتقط؛ ولا يثبت ذلك ما كان عليه النهج أثناء
النشاط السابق.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `SourcePath` | المضيف (من خلية `SYSTEM` الخاصة بالنظام)، وخلية `SECURITY`. |
| `Layout` | `subcategories` (Windows Vista والإصدارات الأحدث) أو `categories` (XP و2003)؛ ويكون فارغًا عندما يتعذر فك ترميز القيمة. |
| `Category`، `Subcategory` | كما يسمّيها `auditpol`. |
| `SubcategoryGUID`، `SubcategoryID` | معرّف GUID الخاص بالفئة الفرعية، ومعرّف الرسالة الخاص بها كما يعطيه الحدث 4719 (`%%12548`). |
| `Setting` | `none` أو `success` أو `failure` أو `success_and_failure`. |
| `Success`، `Failure` | ما إذا كانت حالات النجاح والفشل تُسجَّل. |
| `AuditingEnabled` | Windows XP و2003: ما إذا كان التدقيق مفعّلًا أصلًا. |
| `Events` | معرّفات الأحداث الرئيسية للفئة الفرعية. |
| `KeyLastWrittenUTC` | آخر كتابة لـ `PolAdtEv`: تقريبًا وقت آخر تغيير للنهج. |
| `DataRaw` | بايتات القيمة (بالنظام الست عشري)، عندما يتعذر فك ترميزها بالكامل. |
| `TransactionLogStatus`، `LogsApplied` | سياق استرداد سجل Windows. |
| `Problems` | تخطيط أو حجم غير معروف، أو أعداد غير متسقة، أو قيم تتجاوز 3. |

## استراتيجية التحقيق { #investigation-strategy }

1. قبل استنتاج أن أمرًا ما لم يحدث، تحقق مما إذا كانت فئته الفرعية خاضعة للتدقيق:
   Process Creation (4688)، وLogon (4624، 4625)، وSpecial Logon (4672)، وOther Object Access Events
   (المهام المجدولة، 4698)، وSecurity System Extension (تثبيت الخدمات، 4697)، وUser Account
   Management وSecurity Group Management (4720، 4732)، وFile Share (5140)، وKerberos (4768، 4769).
2. قارن النهج بسجل Security: فوجود أحداث لفئات فرعية لا تخضع للتدقيق حاليًا يعني أن النهج قد تغيّر.
   ابحث عن الحدث 4719 وعن `KeyLastWrittenUTC` حديث.
3. قارن مع ملف `audit.csv` الخاص بنهج المجموعة للتمييز بين نهج على مستوى المجال وتغيير محلي.
4. استخدم `parse-registry read --recover-deleted` وسجلات المعاملات للحصول على إصدارات سابقة من
   `PolAdtEv`.

## مزيد من التفاصيل { #further-details }

- [Microsoft: إعدادات نهج تدقيق الأمان المتقدم](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft: ثوابت التدقيق (معرّفات GUID للفئات الفرعية)](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft: 4719(S) System audit policy was changed](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
