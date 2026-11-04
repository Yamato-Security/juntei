# حسابات SAM المحلية { #sam-local-accounts }

## ما يحتويه SAM { #what-the-sam-contains }

تخزن خلية مدير حسابات الأمان (`SAM`) سجلات الحسابات المحلية. ويفك Juntei ترميز أسماء
الحسابات، ومعرّفات RID، وحقول ملف التعريف/التعليقات، وأعلام التحكم بالحساب، والطوابع الزمنية لآخر تسجيل دخول/كلمة مرور، والعدادات،
وتلميحات كلمات المرور، والارتباط بحساب Microsoft، ومسارات ملفات التعريف، وما إذا كان Windows يخفي الحساب
من واجهة تسجيل الدخول.

وهذا مفيد للعثور على مسؤولين محليين أُنشئوا حديثًا، وحسابات خاملة جرى تمكينها، وتغييرات كلمات
المرور، والحسابات المخفية، وحسابات التجهيز للحركة الجانبية.

<span class="evidence-strength">أفضل استنتاج:</span> مثّل SAM المحلي الحساب بالحالة
المسجلة. وهو ليس سجلًا كاملًا للمصادقة ولا يصف حسابات المجال.

## الموقع والجمع { #location-and-collection }

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

اجمع الخلية والسجلات بصلاحيات كافية/وصول خام. ويمكن إثراء هوية المضيف من
خلية `SYSTEM` المقابلة في المجموعة المجمّعة.

## تحذيرات للمحلل { #analyst-cautions }

- يحلل هذا الأمر الحسابات **المحلية**، لا مستخدمي مجال Active Directory.
- لا يستخرج Juntei تجزئات كلمات المرور عن قصد.
- RID 500 هو حساب Administrator المدمج حتى لو أُعيدت تسميته؛ فالاسم وحده غير كافٍ.
- `LastLogon` والعدادات حالة في سجل Windows، لا مسار تدقيق كامل لكل جلسة.
- يخفي `HiddenFromLogonUI` الحساب من شاشة تسجيل الدخول؛ لكنه لا يعطّل الحساب.
- يمكن أن يكون الحسابان المدمجان `DefaultAccount` و`WDAGUtilityAccount` مخفيين بشكل مشروع.
- يجب عدم تفسير أعلام الحساب الفارغة الناتجة عن قيمة `F` تالفة/مفقودة على أنها تعني أن الحساب مُمكَّن.
- يقارب `NamesKeyLastWrittenUTC` سياق تسجيل اسم الحساب لكنه ليس وقت إنشاء مضمونًا.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.HiddenFromLogonUi == true or (.AccountFlags | index("disabled") | not))'
```

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Host`، `SourcePath` | سياق المضيف وخلية SAM بالضبط. |
| `RID` | المعرّف النسبي (Relative Identifier). نقطة ارتكاز قوية لهوية الحساب؛ و500 هو حساب Administrator المدمج. |
| `User` | الاسم الحالي للحساب المحلي. قد تُعاد تسميته؛ اربط عبر RID. |
| `FullName`، `Comment`، `UserComment` | نص وصفي؛ يتحكم فيه المهاجم ومفيد لمراجعة انتحال الهوية. |
| `AccountFlags` | حالات بعد فك ترميزها مثل: معطّل، مقفل، لا يتطلب كلمة مرور، حساب عادي، كلمة مرور لا تنتهي صلاحيتها أبدًا. |
| `AccountFlagsValue` | الأعلام الرقمية الخام للتحقق وللبتات غير المدعومة. |
| `CountryCode`، `CodePage` | بيانات وصفية للإعدادات المحلية/صفحة الترميز للحساب؛ سياق عادةً، لا إسناد. |
| `LastLogonUTC`، `LastLogonWholeSecondsUTC` | حالة آخر تسجيل دخول نفسها بالدقة الكاملة وبثوانٍ كاملة للتوافق. ليست سجلًا كاملًا. |
| `PasswordLastSetUTC` | وقت تغيير كلمة المرور المسجل. حقّق في القيم الحديثة/القديمة غير المتوقعة. |
| `AccountExpirationUTC` | تاريخ انتهاء الصلاحية المُكوَّن عندما يكون محددًا. وقد تعني القيمة الفارغة "أبدًا"/"غير مضبوط". |
| `LastFailedLogonUTC` | آخر تسجيل دخول محلي فاشل مسجل في حالة SAM؛ أكّده بأحداث Security. |
| `FailedLogonCount`، `LogonCount` | عدادات تراكمية/حالة بسلوك تحديث خاص بـ Windows؛ استخدمها كسياق، لا كعدد أحداث دقيق. |
| `PasswordHint` | تلميح حساس يقدمه المستخدم؛ قد يكشف أنماط كلمات المرور. تعامل معه بالشكل المناسب. |
| `InternetUserName` | هوية حساب Microsoft المرتبط عند وجوده. |
| `HiddenFromLogonUI` | حالة الإخفاء من الواجهة. راجع معرّفات RID المرتفعة/الحسابات المخصصة غير المتوقعة؛ ولا يعني أن الحساب معطّل. |
| `HomeDirectory`، `HomeDrive`، `ProfilePath`، `LogonScript` | تكوين بيئة/تسجيل دخول الحساب المحلي؛ وقد تشير البرامج النصية أو مسارات UNC غير المعتادة إلى الاستمرارية. |
| `KeyLastWrittenUTC` | وقت تغيير مفتاح الحساب الخاص بـ RID. على مستوى المفتاح، لا خاص بحقل معين. |
| `NamesKeyLastWrittenUTC` | وقت تغيير مفتاح ربط اسم المستخدم؛ تقريب مفيد لدورة حياة الحساب. |
| `LastLogonFiletime`، `PasswordLastSetFiletime`، `AccountExpirationFiletime`، `LastFailedLogonFiletime` | قيم FILETIME الخام بطول 64 بت، يُحتفظ بها للتحقق وللتعامل مع القيم الحارسة الخاصة. |
| `TransactionLogStatus`، `LogsApplied` | سياق إعادة تشغيل سجل Windows. |
| `Problems` | عدم الاتفاق بين F/V/Names، أو إدخالات مفقودة، أو إزاحات مشوّهة، أو تلف في فك الترميز. |

## استراتيجية التحقيق { #investigation-strategy }

1. قارن RID والاسم والأعلام ومسار ملف التعريف و`NamesKeyLastWrittenUTC` بخط الأساس للمضيف.
2. راجع الحسابات المُمكَّنة ذات RID ‏1000 فما فوق، والحسابات التي لا تتطلب كلمة مرور، أو لا تنتهي صلاحيتها، أو المخفية، أو التي تغيرت مؤخرًا.
3. اربط ذلك بأحداث Security ‏4720/4722/4724/4732، وأحداث تسجيل الدخول، وإنشاء ملفات التعريف، والنشاط عن بُعد.
4. ابحث في سجل استعادة سجل Windows إذا كان الحساب قد أُزيل قبل الجمع.
5. ميّز الحسابات المُجهّزة/الافتراضية المشروعة من الحسابات التفاعلية التي أنشأها المهاجم.

## مزيد من التفاصيل { #further-details }

- [Microsoft Open Specifications: بروتوكول SAM البعيد](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [مكوّن SAM الإضافي في dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
- [أداة Registry Explorer من Eric Zimmerman](https://ericzimmerman.github.io/#!index.md)
