# سجل اكتشافات Microsoft Defender { #microsoft-defender-detection-history }

## ما الذي يسجله السجل { #what-the-history-records }

يكتب Microsoft Defender ملفًا واحدًا لسجل الاكتشاف عن كل اكتشاف. ويسجّل كل ملف:

- معرّف التهديد واسمه وخطورته وفئته؛
- ما فعله Defender وما إذا كان قد نجح؛
- ما إذا كان التهديد قيد التشغيل؛
- المستخدم والعملية المعنيّين، وما الذي اكتشف التهديد؛
- إدخال الحجر، إن وُجد؛
- كل مورد عُثر على التهديد فيه: الملفات، وعناصر الأرشيفات، والتنزيلات، والعمليات، وقيم سجل Windows،
  والخدمات، وأسطر الأوامر، والسلوكيات.

ويكتب Defender هذه الملفات أيضًا للتهديدات التي لم يحجرها قط: تلك التي اكتشفها فقط، وتلك التي سُمح
بها، وتلك التي فشل في إزالتها. ويعرض سجل الحماية في أمن Windows وفئة WMI
`MSFT_MpThreatDetection` (`Get-MpThreatDetection`) هذه السجلات. وعندما غيّرت Orange Cyberdefense
قيمًا في أحد الملفات، أبلغت WMI عن القيم المعدّلة.

<span class="evidence-strength">أفضل استنتاج:</span> سجّل Defender اكتشافًا للتهديد المسمّى في
الموارد المدرجة، بالحالة والأوقات المسجلة. واسم التهديد حكم من المنتج. ولا يُظهر سجل الاكتشاف وحده
أن التهديد قد شُغّل، أو أنه أُزيل بالكامل.

## الموقع والجمع { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

يحمل كل ملف اسم `DetectionID` الخاص به. اجمع مجلد `DetectionHistory` بأكمله. ويقرأ إدخال المجلد
و`auto-parse` كل ملف داخل مجلد `DetectionHistory`، على أي عمق، إذا كانت بدايته كبداية ملف سجل
الاكتشاف. استخدم `-f` لنسخة محفوظة خارج مثل هذا المجلد. ويزيل Defender العناصر من سجل الفحص بعد عدد
قابل للتكوين من الأيام (`ScanPurgeItemsAfterDelay` في `Set-MpPreference`)، لذا قد تكون الاكتشافات
الأقدم قد زالت.

الحجر أثر رقمي منفصل: راجع [حجر Defender](defender-quarantine.md). ويستخدم `QuarantineID` الخاص
بالاكتشاف الكتابة نفسها المستخدمة في `EntryID` في إخراج الحجر (أحرف كبيرة، دون أقواس معقوفة)، لذا
يمكن ربط الاثنين.

## تحذيرات للمحلل { #analyst-cautions }

- الاكتشاف، ومحاولة المعالجة، والإزالة الناجحة حالات منفصلة. اقرأ `ThreatStatus` (`Detected`،
  `Quarantined`، `Removed`، `Allowed`، `RemoveFailed`، …) مع `ThreatStatusErrorCode`، الذي تكون
  قيمته 0 عندما ينجح آخر تغيير في الحالة.
- `ExecutionStatus` هو رؤية Defender وقت الاكتشاف (`Blocked`، `Allowed`، `Executing`،
  `NotExecuting`، `Unknown`). وهو ليس دليلًا على أن التهديد قد شُغّل، ولا على أنه لم يُشغَّل قط.
  تحقّق من ذلك بمقارنته بـ Prefetch وAmcache وسجلات الأحداث وبيانات مراقبة العمليات.
- `ThreatName` هو اسم التوقيع في ذلك الوقت. والنتائج الإيجابية الكاذبة وإعادة التصنيف لاحقًا ممكنة.
  ويُبلَّغ عن ملفات الاختبار مثل EICAR كما لو كانت تهديدات حقيقية.
- `User` هو قيمة `DomainUser` التي سجّلها Defender (قيمة `UserSource` هي `DomainUser`). وقد يكون
  فارغًا، أو حساب خدمة، أو مستخدمًا غير الذي أنشأ الملف.
- قد يكتب Defender عدة ملفات لتهديد واحد، ويحتفظ Juntei بكل منها. بل قد يحمل ملفان `DetectionID`
  نفسه، كما يحدث مع ملف منسوخ أو مستعاد. جمّع حسب `DetectionID` و`SourceFile` معًا.
- يُحتفظ بالقيم غير المعروفة المعنى دون تفسير: يوجد قسمها وموضعها في `UnknownValues`.

## التحليل باستخدام Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

CSV هو تنسيق الملف الافتراضي. باستخدام `-t csv` أو `-t json` أو `-t jsonl` دون `-o`، تُكتب السجلات
إلى stdout. عند غياب الخيارين، يُعرض ملخص فقط. أضف `-C` لاستبدال ناتج موجود. ولا يُكتب الإخراج أبدًا
فوق أحد ملفات الإدخال، حتى مع `-C`.

يكتب Juntei سجلًا واحدًا لكل مورد من موارد كل اكتشاف. ويحصل الاكتشاف الذي لا موارد له على سجل واحد
بحقول موارد فارغة. وتشترك سجلات الاكتشاف الواحد في `DetectionID` الخاص به، وتُرقَّم بـ
`ResourceIndex` بترتيب تخزينها.

لا يوقف التلف التشغيل. فالملف المقتطع يحتفظ بالقيم المقروءة قبل موضع التلف، ويذكر `Problems` ما هو
مفقود. ويُبلَّغ عن أي ملف في مجلد `DetectionHistory` ليس ملف سجل اكتشاف ويُتخطّى. وتُقرأ الملفات
الأخرى مع ذلك.

## حقول الإخراج { #output-fields }

| الحقل | التفسير |
|---|---|
| `Record` | `resource`، أو `detection` للاكتشاف الذي لا موارد له. |
| `InitialDetectionTime` | وقت اكتشاف Defender للتهديد أول مرة (UTC). يكون فارغًا عندما يحتوي الملف على 0؛ ويُحتفظ بالقيمة التي ليست وقتًا في `UnknownValues`. |
| `LastThreatStatusChangeTime` | وقت آخر تغيير في حالة التهديد (UTC). |
| `RemediationTime` | وقت معالجة Defender للتهديد (UTC). وهو وقت محاولة، وليس دليلًا على الإزالة؛ تحقق من `ThreatStatus`. |
| `ThreatName`، `ThreatID` | اسم توقيع Defender ومعرّفه الرقمي (`Get-MpThreatCatalog`). |
| `Severity`، `SeverityID` | `Low` أو `Moderate` أو `High` أو `Severe` أو `Unknown`، مع الرقم المخزن. |
| `Category`، `CategoryID` | فئة التهديد مثل `TROJAN` أو `TOOL` أو `BEHAVIOR`، مع الرقم المخزن. |
| `ThreatStatus`، `ThreatStatusID` | حالة التهديد لدى Defender. ويُحتفظ بالرقم الذي لا اسم موثقًا له مع `ThreatStatus` فارغ. |
| `ThreatStatusErrorCode` | رمز خطأ آخر تغيير في الحالة؛ وتعني القيمة 0 أنه نجح. |
| `ExecutionStatus`، `ExecutionStatusID` | ما إذا كان Defender قد رأى التهديد قيد التشغيل عند اكتشافه. |
| `DetectionSource`، `DetectionSourceTypeID` | ما الذي اكتشف التهديد: `Real-time`، و`IOAV` (التنزيلات والمرفقات)، و`User`، و`System`، و`AMSI`، وغيرها. |
| `User`، `UserSource` | قيمة `DomainUser` التي سجّلها Defender، وأساسها (`DomainUser`). يكون فارغًا عندما لا يذكر الملف أي مستخدم. |
| `Process` | العملية المعنية (`ProcessName`)؛ وقد تكون `Unknown`. |
| `ResourceIndex`، `ResourceCount` | موضع هذا المورد، وعدد الموارد في الاكتشاف. |
| `ResourceType` | `file`، و`containerfile`، و`webfile`، و`process`، و`regkeyvalue`، و`service`، و`CmdLine`، و`behavior`، و`amsi`، وغيرها، كما يسمّيها Defender. |
| `Path` | المورد: مسار، أو عنصر في أرشيف (`archive.zip->member`)، أو قيمة في سجل Windows، أو سطر أوامر، أو عملية. ويذكر `webfile` أيضًا عنوان URL الخاص به والعملية التي نزّلته (`pid:…,ProcessStart:…`). احتفظ بالكتابة الدقيقة. |
| `ResourceFlags` | علامات المورد بالصيغة الست عشرية؛ ومعناها غير موثق. |
| `TrackingStartTime` | قيمة `ThreatTrackingStartTime` للمورد، وهي عدد صحيح يُقرأ بوصفه FILETIME ‏(UTC)، كما يؤرّخ plaso الاكتشاف. |
| `SHA256`، `SHA1`، `MD5`، `TrackingSize` | التجزئات والحجم من بيانات تتبع المورد. وهي تصف الملف كما كان عندما فحصه Defender. |
| `TrackingID` | قيمة `ThreatTrackingId` للمورد. |
| `OtherTracking` | كل قيم التتبع الأخرى، بصيغة `Key=Value` بترتيب تخزينها، مثل `ThreatTrackingScanSource` أو `ThreatTrackingBMContextRich` لاكتشافات السلوك. |
| `QuarantineID` | إدخال الحجر (`Quarantine\Entries\{GUID}`)؛ ويكون فارغًا عندما يحتوي الملف على GUID صفري. |
| `DetectionID` | معرّف GUID للاكتشاف؛ وهو أيضًا اسم الملف. ويُبلَّغ عن عدم التطابق في `Problems`. |
| `MagicVersion` | قيمة `Magic.Version:1.2` في الترويسة؛ وتكون فارغة في الملفات المكتوبة دونها، مثل بعض ملفات Windows Server 2016. |
| `UnknownValues` | القيم غير المعروفة المعنى، بصيغة `position=value` (مثل `6.13=NT AUTHORITY\SYSTEM`)، مع القائمة المعدودة التي تلي الحالة بوصفها `2.list`. |
| `SourceFile` | الملف المقروء، نسبةً إلى مجلد الإدخال، أو اسمه مع `-f`. |
| `Problems` | التلف، أو القيم المفقودة، أو الأنواع غير المتوقعة، أو اسم ملف غير مطابق. لا تُخفِها. |

تحتفظ JSON وJSONL بالحقول نفسها بقيم ذات أنواع: تبقى الأرقام أرقامًا، وتكون القيم المفقودة `null`.
وبدلًا من `OtherTracking` تتضمنان `Tracking`، أي كل قيمة تتبع مع مفتاح Defender ونوعها وقيمتها.
ويصبح `UnknownValues` قائمة بالمواضع مع أنواعها. ويحمل `CountedList` القائمة المعدودة. ويحتفظ
FILETIME المخزن بعدد نبضاته الخام بجانب الوقت الذي يُقرأ به.

## سلوك الجدول الزمني { #timeline-behavior }

يحفظ `auto-parse` السجلات بوصفها `SecurityProducts/DefenderHistory.csv` (أو `.json`/`.jsonl`)
ويضيفها إلى الجدول الزمني المدمج `-j`. وينشئ `super-timeline` أحداثًا من `InitialDetectionTime`
و`LastThreatStatusChangeTime` و`RemediationTime` في سجل كل مورد، مع الحالة ورمز الخطأ وحالة التنفيذ
ومصدر الاكتشاف في تفاصيل الحدث. ويُخزَّن `TrackingStartTime` بوصفه عددًا صحيحًا، لذا لا يصبح حدثًا
إلا مع `--include-interpreted-times`. ولا تُستخدم أوقات الملف نفسه أبدًا. ويبقى السجل الذي لا يحمل
وقتًا صالحًا للاستخدام في الإخراج المحلل ويُحتسب ضمن السجلات غير المحددة بوقت.

## استراتيجية التحقيق { #investigation-strategy }

1. ابدأ من `ThreatName` و`Path` و`SHA256` و`InitialDetectionTime` و`ThreatStatus`.
2. تحقق من `ThreatStatusErrorCode` و`Problems` قبل أن تستنتج أن التهديد قد أُزيل.
3. اربط `QuarantineID` بـ `EntryID` في الحجر لاستعادة البايتات المحجورة.
4. انطلق من التجزئات والمسارات إلى `$MFT` ودفتر يومية USN وPrefetch وAmcache وسجل المتصفح وسجلات
   الأحداث (السجل التشغيلي لـ Defender: معرّفات الأحداث 1116 و1117 و1118 و1119).
5. بالنسبة إلى موارد `webfile`، استخدم عنوان URL ووقت بدء العملية للتحقيق في الوصول الأولي، وتحقق
   منهما بأدلة المتصفح والوكيل (proxy) وDNS.

## الإصدارات المختبرة { #tested-versions }

تستخدم الاختبارات ملفات عامة قرأت أدوات أخرى نتائجها بشكل مستقل:

- عيّنة Windows 10 من Orange Cyberdefense ‏(Defender 4.18.24090.11)، مع مقارنتها بما أبلغت عنه WMI.
- ملفا اختبار Windows من plaso، مع مقارنتهما بمحلل `windefender_history` في plaso.
- 14 ملفًا من الآلة الافتراضية لمحاكي APT في DFIRArtifactMuseum.
- نسخة مكتوبة دون الإصدار في الترويسة، كما هي حال بعض الملفات من Windows Server 2016.
- ملفات مقتطعة، أو فارغة، أو من نوع آخر.

## مزيد من التفاصيل { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [defender من Sootmark](https://github.com/Sootmark/defender)
- [wddh-parser من Orange Cyberdefense](https://github.com/cert-orangecyberdefense/wddh-parser)
- [محلل سجل Defender في plaso](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Windows.Applications.DefenderHistory في Velociraptor](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft: فئة MSFT_MpThreatDetection](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft: معرّفات أحداث Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
