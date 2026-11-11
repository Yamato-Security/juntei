# SRUM { #srum }

## ما هو SRUM { #what-srum-is }

يخزن مراقب استخدام موارد النظام (System Resource Usage Monitor أو SRUM) قاعدة بيانات ESE لاستخدام الموارد من قِبل التطبيقات
والخدمات. وبحسب إصدار Windows، يسجل بايتات الشبكة، ومعرّفات الواجهات/ملفات التعريف،
والاتصالات، ووقت CPU، والنشاط في المقدمة/الخلفية، ونشاط القرص، واستهلاك الطاقة، والإشعارات
الفورية، وبيانات الجدول الزمني للتطبيقات. وتُربط الإدخالات بمعرّفات التطبيقات ومعرّفات SID للمستخدمين
عبر `SruDbIdMapTable`.

يُعد SRUM مفيدًا بشكل خاص عندما يكون ملف تنفيذي قد حُذف: إذ قد يبقى معرّف تطبيقه ومستخدمه وصفوف
استخدام الموارد بعد زوال الملف. ويمكن لإجماليات الشبكة أن تدعم فرضية تسريب البيانات، لكن
SRUM ليس التقاطًا للحزم ولا يحدد عمومًا كل نقطة نهاية بعيدة أو كل ملف منقول.

<span class="evidence-strength">أفضل استنتاج:</span> نسب Windows استخدام الموارد المسجل
إلى التطبيق/الحساب خلال فترة القياس الخاصة بالجدول.

## الموقع والجمع { #location-and-collection }

```text
C:\Windows\System32\sru\SRUDB.dat
C:\Windows\System32\config\SOFTWARE   (context for account and Wi-Fi names)
```

يوجد SRUM على Windows 8 والإصدارات الأحدث. انسخ `SRUDB.dat` بشكل متسق، ويفضّل من لقطة أو باستخدام
أداة جمع جنائية. واجمع أيضًا `SOFTWARE` وملفات معاملات/سجلات ESE في مجلد `sru`.

خلية SOFTWARE المطابقة منفصلة عن قاعدة ESE: إذا كانت غير متزامنة، يعيد Juntei تلقائيًا تشغيل سجلات معاملات السجل المجاورة `SOFTWARE.LOG` و`.LOG1` و`.LOG2` في الذاكرة. اجمع هذه الملفات أيضًا. تُعرض تفاصيل الاستعادة والتحذيرات؛ وبدون السجلات قد تكون أسماء الحسابات أو Wi-Fi ناقصة أو قديمة. هذا لا يعيد تشغيل سجلات قاعدة SRUM.

!!! warning "قواعد البيانات غير المتسقة"
    يقرأ Juntei ملف `SRUDB.dat` غير المتسق (dirty) كما جُمع ويُبلغ عن ذلك. ولا يعيد حاليًا تشغيل
    `SRU*.log`، لذا تكون التغييرات الموجودة في تلك السجلات فقط مفقودة. لا تصف قاعدة بيانات
    غير متسقة بأنها كاملة دون الإشارة إلى ذلك.

## تحذيرات للمحلل { #analyst-cautions }

- تمثل صفوف كثيرة تجميعًا دوريًا، عادةً عند حدود الساعات تقريبًا، لا وقت الإجراء الدقيق.
- لا تحدد إجماليات البايتات بحد ذاتها المحتوى أو النية أو البروتوكول أو الوجهة.
- قد تكون هوية التطبيق مسارًا أو خدمة أو تطبيقًا محزّمًا أو كائنًا؛ افحص `AppType`.
- أسماء المستخدمين وشبكات Wi-Fi إثراء من خلية `SOFTWARE` المطابقة. ولا يعني غياب الأسماء
  أن المعرّف الأساسي غائب.
- `null`، والصفر، والأعمدة الغائبة، والأنواع غير المدعومة، والقيم التالفة حالات مختلفة.
- تختلف مخططات المزوّدين باختلاف إصدار Windows. احتفظ بالجداول غير المعروفة وأسماء الأعمدة الأصلية.
- تولّد خدمات Windows العاملة في الخلفية بشكل مشروع إجماليات موارد كبيرة.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SRUDB.dat
  -f, --file <FILE>      Parse one SRUDB.dat
  -r, --software <FILE>  Matching SOFTWARE hive for names

Output:
  -o, --output <PATH>        Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>   csv, json, or jsonl
      --include-value-state  Add per-value read state and raw bytes to JSON/JSONL
```

```bash
juntei parse-ese srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-ese srum -f SRUDB.dat -t jsonl \
  | jq 'select(.Provider == "NetworkUsages" and (.BytesSent // 0) > 100000000)'
juntei parse-ese srum -f SRUDB.dat -t jsonl --include-value-state \
  | jq '{Provider, Timestamp, ValueState, Raw}'
```

## جداول الإخراج { #output-tables }

ينشئ CSV الملف `SRUM-IdentityMap.csv` لـ `SruDbIdMapTable` وجدولًا واحدًا لكل مزوّد. ومن أسماء الإخراج المعروفة:

| المزوّد/الجدول | الغرض في التحقيق |
|---|---|
| `NetworkUsages` | البايتات المرسلة/المستلمة لكل تطبيق ومستخدم وواجهة وملف تعريف. |
| `NetworkConnections` | نشاط من نوع عدد الاتصالات/مدتها وأوقات بدء الاتصالات. |
| `AppResourceUseInfo` | استخدام CPU والقرص والشبكة في المقدمة/الخلفية وما يتصل به من استخدام التطبيقات. |
| `AppTimelineProvider` | نشاط الجدول الزمني للتطبيقات مع حقلي وقت الانتهاء وحالة الاستخدام. |
| `EnergyUsage` | قياسات الطاقة/المزوّد قصيرة المدى وطويلة المدى؛ ويميّز `IsLt` بينهما. |
| `PushNotifications` | نشاط الإشعارات الفورية. |
| `VfuProvider` | المزوّد `vfuprov`؛ وغرضه التفصيلي غير موثق جيدًا، لذا فسّر حقوله بحذر. |
| `EnergyEstimation`، `TaggedEnergy` | مزوّدات للطاقة والاستخدام خاصة بإصدارات معينة. |
| `ProcessorTime`، `SizeInBytes`، `TotalUsed`، `NetworkBytes` | جداول مزوّدات Windows Server. |
| اسم جدول غير معروف | يحتفظ Juntei بالمزوّدات غير المعروفة تحت اسم GUID/الجدول الخاص بها ويُخرج كل أعمدة الفهرس. |

لأن SRUM يعتمد على الفهرس (catalog) ولأن Windows يضيف أعمدة للمزوّدات، لا توجد عن قصد قائمة ثابتة
محدودة بالحقول الخاصة بكل مزوّد. ويُخرج Juntei **كل عمود موجود في قاعدة البيانات مع
تسميات الحقول بصيغة PascalCase الخاصة بـ Juntei**. والحقول التالية هي عقد الإخراج الثابت/المشترك.

## إخراج auto-parse المصنّف { #categorized-auto-parse-output }

يضع `auto-parse` كل جدول من جداول SRUM مباشرة في فئة التحقيق الخاصة به. وتستخدم CSV وJSON وJSONL
البادئة نفسها لاسم الملف `SRUM-`؛ ولا يوجد مجلد `SRUM/` حاوٍ.

| الفئة | ملفات CSV |
|---|---|
| `NetworkActivity` | `SRUM-NetworkConnections.csv`، `SRUM-NetworkUsages.csv`، `SRUM-NetworkBytes.csv`، `SRUM-PushNotifications.csv` |
| `ProcessExecution` | `SRUM-AppResourceUseInfo.csv`، `SRUM-AppTimelineProvider.csv`، `SRUM-ProcessorTime.csv` |
| `SystemInformation` | `SRUM-EnergyUsage.csv`، `SRUM-EnergyEstimation.csv`، `SRUM-TaggedEnergy.csv`، `SRUM-SizeInBytes.csv`، `SRUM-TotalUsed.csv`، `SRUM-IdentityMap.csv`، `SRUM-VfuProvider.csv`، وجداول المزوّدات غير المعروفة |

لا تُصدَّر إلا الجداول الموجودة في البيانات المجمعة. وتوضع عدادات الشبكة وسجلات الإشعارات
بحسب استخدامها الرئيسي في التحقيق؛ وهذا لا يعني أن كل صف يصف اتصالًا
بعيدًا. وتدعم خريطة الهوية عدة فئات. ويبقى VFU والمزوّدات غير المعروفة في
`SystemInformation` لأن وجودها وحده لا يثبت التنفيذ.

يحتوي `SystemInformation/SRUM-logs.txt` على تشخيصات المحلل وعلى الربط بين الإخراج الأصلي
والنهائي. ويسرد كلا البيانين (manifests) مسارات الجداول الفعلية في `OutputFiles`؛ ويكون `Output` لمهمة SRUM
هو مجلد الحاسوب، ويحدد `Category` موقع السجل/البيانات الوصفية المشترك الخاص بها.

مع تعدد قواعد البيانات، تكتسب أسماء الملفات `-Source1` و`-Source2` وهكذا، قبل الامتداد.
ويحتفظ السجل بالأسماء المستعارة الأصلية للبيانات المجمعة، وتحتفظ صفوف JSON/JSONL بمعلومات المصدر `SourceFile`
و`Table` و`Provider` و`Comp` و`Artifact`. ولا يُنسخ أي جدول إلى عدة
فئات. ويتضمن الجدول الزمني المدمج لـ SIEM ‏`-j` كل سجل SRUM محفوظ مرة واحدة.

يحتفظ الأمر المستقل `parse-ese srum` بموقع الإخراج المطلوب. وتحمل ملفات جداول CSV الخاصة به أيضًا
البادئة `SRUM-`؛ أما أسماء ملفات إخراج JSON/JSONL الصريحة فيبقى اختيارها للمحلل.

## حقول الإخراج المشتركة { #common-output-fields }

| الحقل | التفسير |
|---|---|
| `SourceFile` | قاعدة بيانات الإدخال؛ موجود في JSON/JSONL ومفيد عبر المضيفين. |
| `Table` | اسم/GUID جدول ESE الأصلي. احتفظ به عند مقارنة الإصدارات أو المزوّدات غير المعروفة. |
| `Provider` | الاسم المألوف للمزوّد/الإخراج، أو اسم الجدول الأصلي إذا كان غير معروف. |
| `ID` | قيمة `AutoIncId` للصف؛ معرّف صف في قاعدة البيانات، لا معرّف عملية. |
| `Timestamp` | وقت تسجيل/تجميع SRUM للأرقام، مقرّبًا إلى الملّي ثانية. وعادةً ليس وقت الحدث الدقيق. |
| `EXE-Info` | هوية التطبيق/الخدمة/الحزمة بعد حلّها من `AppID`. نقطة الارتكاز الرئيسية للبرنامج. |
| `EXE-InfoDescription` | الوصف المستخرج من أسماء تطبيقات SRUM المركبة. للسياق فقط. |
| `EXE-Timestamp` | الوقت المضمّن في هوية تطبيق مركبة عند وجوده؛ وليس وقت استخدام الصف. |
| `AppType` | `NormalApp` أو `Service` أو `ModernApp` أو `Object`؛ ويحدد كيفية تفسير `EXE-Info`. |
| `SID` | معرّف SID للحساب بعد حلّه من `UserID`. نقطة ارتكاز قوية لهوية المستخدم/الخدمة. |
| `User` | اسم ملف التعريف المألوف المُثرى من `SOFTWARE`؛ وقد يكون فارغًا بينما يظل `SID` صالحًا. |
| `UserID`، `AppID` | مراجع رقمية خام إلى `SruDbIdMapTable`؛ احتفظ بها للأسماء غير المحلولة أو المكررة. |
| `IsLt` | `true` لجدول EnergyUsage طويل المدى، و`false` لقصير المدى. |
| `InterfaceLUID` | معرّف LUID الخام لواجهة الشبكة من صفوف المزوّد. |
| `InterfaceType` | فئة الواجهة بعد فك ترميزها (مثل `IF_TYPE_IEEE80211`)؛ فئة، لا محوّل فريد. |
| `L2ProfileID` | مرجع رقمي لملف تعريف الشبكة. |
| `ProfileName` | اسم Wi-Fi/ملف التعريف المُثرى من `SOFTWARE`؛ قد يكون حساسًا وليس دليلًا على اتصال حالي. |
| `ConnectStartTime`، `EventTimestamp`، `EndTime`، `StartTime` | أعمدة FILETIME معروفة للمزوّدات بعد فك ترميزها إلى UTC. ومعناها الدقيق يخص المزوّد، وليس وقت تنفيذ عامًا. |
| أعمدة خاصة بالمزوّد | أعمدة المزوّد المُصدَّرة مثل `BytesSent` و`BytesReceived` (عمود قاعدة البيانات `BytesRecvd`)، وعدادات CPU/القرص، والمدد، والأعداد، والأعلام، أو التقديرات. فسّر الوحدات بحسب المزوّد وقارن الصفوف ضمن المخطط نفسه. |

## الحقول التشخيصية (`--include-value-state`) { #diagnostic-fields-include-value-state }

| الحقل | التفسير |
|---|---|
| `ValueState` | خريطة من حقل قاعدة البيانات إلى `parsed` أو `null` أو `absent` أو `unsupported` أو `damaged`. وهذا يفصل غياب الدليل عن فشل المحلل. |
| `Raw` | البايتات الأصلية بالنظام السداسي عشري للقيم التي لها بايتات. استخدمها للتحقق ولفك الترميز مستقبلًا. |

يحتوي `SRUM-IdentityMap.csv` (من `SruDbIdMapTable`) على `IdIndex` و`IdType` و`IdBlob`: الفهرس الخام، وفئة الهوية،
ونص البرنامج/SID/الكائن بعد فك ترميزه الذي تستخدمه صفوف المزوّدات.

## استراتيجية التحقيق { #investigation-strategy }

1. حدّد البرنامج باستخدام `EXE-Info` و`AppType` و`AppID`؛ ولا تخلط بين `Id` وPID.
2. جمّع صفوف الشبكة حسب البرنامج والمستخدم وملف التعريف والواجهة والفترة الزمنية.
3. قارن البايتات المرسلة/المستلمة بخطوط الأساس المعتادة وبالمضيفين الآخرين.
4. اربط الفترات المشبوهة بـ Prefetch وسجلات الأحداث وبيانات DNS/الوكيل/جدار الحماية والوصول إلى الملفات.
5. أبلغ عن حالة قاعدة البيانات غير المتسقة وعدم إعادة تشغيل سجلات المعاملات بوصفها قيدًا في الجمع.

## مزيد من التفاصيل { #further-details }

- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [dissect.esedb](https://github.com/fox-it/dissect.esedb)
- [libyal libesedb](https://github.com/libyal/libesedb)
- [Microsoft: محرك التخزين القابل للتوسيع (Extensible Storage Engine)](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
