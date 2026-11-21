# سجل PowerShell { #powershell-history }

## ما الذي يسجله السجل { #what-the-history-records }

يحفظ PSReadLine أسطر الأوامر المُدخلة في مضيف PowerShell تفاعلي. يحتوي الملف على النص بترتيب حفظه، دون طوابع زمنية لكل أمر.

## المواقع والجمع { #locations-and-collection }

الموقع الافتراضي في Windows لكل من Windows PowerShell 5.1 وPowerShell 7+ هو:

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

تستخدم المضيفات الأخرى أسماء مثل `Visual Studio Code Host_history.txt` في ذلك المجلد. يستخدم PowerShell على Linux وmacOS عادةً `~/.local/share/powershell/PSReadLine/<Host>_history.txt`. يمكن لإعداد `HistorySavePath` في PSReadLine تغيير موقع الملف؛ هذه المسارات الافتراضية افتراضات للاكتشاف وليست قائمة شاملة بالمواقع.

يختار إدخال المجلد و`auto-parse` ملفات `<Host>_history.txt` الموجودة مباشرةً داخل مجلد `PSReadLine` بالبحث المتكرر، دون تمييز حالة أحرف ASCII. ولا يختاران ملفات `*_history.txt` عشوائية في مواقع أخرى. استخدم الإدخال الصريح `-f` لملف أُعيدت تسميته أو نسخة خارج مجلد `PSReadLine`. حافظ على بنية ملف التعريف الأصلية متى أمكن لإسناد البيانات إلى المستخدم.

## تحذيرات للمحلل { #analyst-cautions }

- لا يحمل أي سطر وقتاً. `FileLastWriteTime` هو وقت تعديل الملف. يحفظ PSReadLine السجل بعد كل أمر افتراضياً (`HistorySaveStyle` `SaveIncrementally`)، لذا يكون هذا الوقت عادةً قريباً من وقت إضافة السطر الأخير، لكنه لا يدل على شيء بشأن الأسطر السابقة.
- لا يثبت السطر المحفوظ أن الأمر نُفّذ أو اكتمل بنجاح. تحقّق من التنفيذ وتوقيته بمقارنتهما بسجلات أحداث PowerShell وبيانات مراقبة العمليات وآثار التنفيذ الأخرى.
- يمكن تعديل السجل أو حذفه أو إيقافه (`HistorySaveStyle` `SaveNothing`). ويستبعد PSReadLine أيضاً الأسطر التي يبدو أنها تحتوي على أسرار، مثل تلك التي تتضمن `password` أو `token` أو `apikey`، ولا تستخدم Windows PowerShell ISE مكوّن PSReadLine. أما البرامج النصية والأوامر المشغّلة باستخدام `-Command` أو `-EncodedCommand` فلا تُكتب يدوياً، لذا لا تُحفظ.
- يُحفظ الأمر المكوّن من عدة أسطر مع علامة اقتباس خلفية في نهاية كل سطر منه عدا الأخير. ويُبقي Juntei هذه الأسطر صفوفاً منفصلة يربط بينها `EntryStartLine`. ويبدو السطر الذي ينتهي بعلامة اقتباس خلفية كتبها المستخدم بالشكل نفسه، لذا اقرأ أسطر الإدخال الواحد معاً.

## التحليل باستخدام Juntei { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

CSV هو تنسيق الملف الافتراضي. باستخدام `-t csv` أو `-t json` أو `-t jsonl` دون `-o`، تُكتب السجلات إلى stdout. عند غياب الخيارين، يُعرض ملخص فقط. يتكون JSON من كائنات منفصلة منسقة للقراءة؛ ويحتوي JSONL على كائن واحد في كل سطر. أضف `-C` لاستبدال ناتج موجود؛ تُحمى المسارات البديلة التي تشير إلى الملف المصدر من الاستبدال.

## الترميز ومعالجة الأسطر { #encoding-and-line-handling }

- تحدد علامة ترتيب البايتات UTF-8 أو UTF-16LE أو UTF-16BE الترميز، وتُزال من بداية الملف. دون BOM، يُعامل الإدخال دائماً بوصفه UTF-8. لا يُخمّن UTF-16 الذي لا يحمل BOM؛ تُحفظ محارف NUL ويُبلّغ عنها كتلف محتمل في الترميز.
- تُستبدل تسلسلات UTF-8 غير الصالحة وبدائل UTF-16 غير المزدوجة بالمحرف U+FFFD، ويُذكر التلف في `Problems`. يُبلّغ أيضاً عن بايت UTF-16 زائد في النهاية وأخطاء القراءة؛ وتُحفظ الأسطر المقروءة. افحص `Problems` وسجل المحلل قبل الاعتماد على اكتمال النتائج.
- تُعالج الملفات تدفقياً. يُحتفظ بحد أقصى قدره 1,048,576 بايت إدخال لكل سطر فعلي، باستثناء BOM وفاصل LF (يُحتسب CR قبل LF ضمن الحد). تُقتطع الأسطر الأطول عند حد محرف كامل، وتُقرأ بقية البايتات، ويذكر `Problems` عدد البايتات المحذوفة. ينطبق الحد على بايتات الإدخال المرمّزة، لا على عدد المحارف.
- تُزال نهايات الأسطر LF وCRLF، وكذلك CR الأخير في سطر غير منتهٍ بفاصل سطر. تُحفظ المسافات وعلامات الجدولة الأخرى. لا تنتج الأسطر الفارغة سجلات، لكنها تُحتسب في أرقام الأسطر وتنهي سلسلة الاستمرار؛ أما الأسطر المؤلفة من محارف بيضاء فقط فتبقى سجلات. قد ينتج خطأ القراءة سطر تشخيص فارغاً يحدد موضع توقف القراءة.
- تبقى الأسطر منفصلة مع حفظ علامات الاقتباس الخلفية والمسافات البادئة. يشير `EntryStartLine` إلى أول سطر في سلسلة متتابعة تنتهي أسطرها السابقة بعلامة اقتباس خلفية. لا تُفسَّر صياغة الصدفة ولا يُعاد بناء الأوامر. قد يؤثر الاقتطاع في إسناد الاستمرار؛ افحص `Problems` قبل تجميع هذه الأسطر.

## حقول إخراج CSV { #csv-output-fields }

| الحقل | التفسير |
|---|---|
| `SourceFile` | مسار نسبي إلى مجلد الإدخال باستخدام `/`؛ ومع `-f` اسم الملف فقط. لا يُصدَّر مسار مصدر مطلق. |
| `User` | اسم ملف التعريف المأخوذ من المسار. يحدد المجلد الذي يعلو `AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine` (أو `.local/share/powershell/PSReadLine`) اسم المستخدم، وكذلك عندما يكون الإدخال هو ملف التعريف ذاك أو مجلد `PSReadLine` نفسه. يتعرف إدخال المجلد أيضاً على بنيتي `Users`/`Documents and Settings` تحت جذر الإدخال. القيم المجهولة فارغة في CSV وتساوي `null` في JSON. |
| `ShellHost` | بادئة اسم الملف قبل `_history.txt`، مثل `ConsoleHost`؛ مجهولة للملفات المعاد تسميتها دون هذه اللاحقة. |
| `LineNumber` | رقم السطر الفعلي بدءاً من 1، بما في ذلك الأسطر الفارغة المتجاوزة. |
| `EntryStartLine` | السطر الفعلي الأول لسلسلة الاستمرار، أو رقم هذا السطر نفسه. |
| `Command` | السطر الفعلي بعد فك الترميز دون نهاية السطر، مع تطبيق الحد الموثق. |
| `FileLastWriteTime` | وقت تعديل الملف المصدر، مكرر في كل صف. **بيانات وصفية للملف، وليس وقت تنفيذ الأمر أبداً.** يستخدم تحليل الصور بيانات المصدر الأصلية لا وقت النسخ المؤقت. |
| `FileSize` | حجم الملف المصدر بالبايت. |
| `Encoding` | `UTF-8` أو `UTF-8-BOM` أو `UTF-16LE` أو `UTF-16BE`. |
| `Problems` | تشخيص تلف الأسطر واقتطاعها؛ مفصول بفواصل منقوطة في CSV، ومصفوفة في JSON/JSONL. تظهر تشخيصات مستوى الملف أيضاً في السجل. |

تتضمن JSON وJSONL الحقول نفسها، مع أنواع فعلية للأرقام، و`null` للقيم المجهولة، و`Problems` على هيئة قائمة. وتحتفظ جميع التنسيقات بنص الأمر كما حُفظ.

يستخدم `FileLastWriteTime` افتراضياً UTC بتنسيق ISO 8601، مثل `2026-10-20T08:46:38.5Z`. يقبل `auto-parse` وحده الخيار `-Z, --timezone` لتغيير الإزاحة المعروضة. لا يملك الأمر المستقل `parse-file powershell-history` خياراً للمنطقة الزمنية.

## التحليل التلقائي { #automatic-parsing }

يكتب `auto-parse` الملف `ShellHistory/PowerShellHistory.csv` (أو `PowerShellHistory.json` أو `PowerShellHistory.jsonl` مع `-t json` أو `-t jsonl`)، مع `ShellHistory/PowerShellHistory-logs.txt`. ومن صور VHDX وE01 تُنسخ ملفات السجل، ويكون `FileLastWriteTime` هو الوقت المسجل في الصورة. ويتضمن الجدول الزمني المدمج `-j` كل صف مع معرّف `Artifact` ذي القيمة `powershell-history`. وليست للأسطر أوقات، لذا لا ينشئ [`super-timeline`](../commands/super-timeline.md) منها أي أحداث ويحتسبها ضمن السجلات غير المحددة بوقت.

## استراتيجية التحقيق { #investigation-strategy }

1. اقرأ سجل كل ملف تعريف، بما في ذلك `Windows\System32\config\systemprofile` وملفات التعريف الموجودة تحت `Windows\ServiceProfiles`، فهي تحتوي على الأوامر المكتوبة في صدفات تعمل بتلك الحسابات.
2. اقرأ الإدخالات متعددة الأسطر معاً، وابحث عن عمليات التنزيل، والأوامر المرمّزة، وتغييرات إعدادات الأمان، وأوامر الحسابات وبيانات الاعتماد، والاتصالات البعيدة.
3. تحقّق بالمقارنة مع سجلات أحداث PowerShell (تسجيل كتل البرامج النصية، الحدث 4104)، وأحداث إنشاء العمليات، وملفات Prefetch الخاصة بـ `powershell.exe` و`pwsh.exe`.
4. قارن `FileLastWriteTime` بأوقات ملف السجل في `$MFT` وUSN Journal، وبعمليات تسجيل الدخول، لتحديد النطاق الزمني الذي أُضيفت فيه الأسطر الأخيرة.

## مزيد من التفاصيل { #further-details }

- [Microsoft — about_PSReadLine: سجل الأوامر](https://learn.microsoft.com/en-us/powershell/module/psreadline/about/about_psreadline)
- [Microsoft — Set-PSReadLineOption: ‏HistorySavePath وHistorySaveStyle](https://learn.microsoft.com/en-us/powershell/module/psreadline/set-psreadlineoption)
- [Microsoft — about_Logging_Windows: تسجيل كتل البرامج النصية](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [PSReadLine — كيفية حفظ ملف السجل وقراءته](https://github.com/PowerShell/PSReadLine/blob/master/PSReadLine/History.cs)
