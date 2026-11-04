# شكر وتقدير Persistence Hunter { #persistence-hunter-acknowledgements }

يبني أمر `persistence-hunter` في Juntei على جيلين من جهود المجتمع في البحث عن آليات الاستمرارية.
شكرًا للمؤلفين والمساهمين الذين أتاحوا تقنياتهم وتطبيقاتهم وأبحاثهم
للمحققين.

## PyrsistenceSniper: مشروع Python { #pyrsistencesniper-the-python-project }

يُعد [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper) من
[Hexastrike](https://github.com/Hexastrike) المرجع المباشر لهذه الميزة. وينقل Juntei
فهرس فحوصاته البالغ 130 فحصًا إلى Rust أصلي، مع قائمة السماح الافتراضية، ولقطة LOLBAS، وسلوك
التصنيف. والإصدار المرجعي مثبّت على
[الإيداع `b99c117`](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f).

مشروع Python مرخص بموجب MIT. ويحتفظ Juntei بإشعار حقوق النشر والترخيص الخاص به في
[`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt).
ويعمل تطبيق Juntei دون اتصال ودون مفسّر Python. ويوثّق
[دليل الأمر](../commands/persistence-hunter.md) طريقة تعامله مع المجموعات المجمعة،
والإبلاغ عن التغطية، وتفسيره المتحفظ للأدلة غير المكتملة.

## PersistenceSniper: أداة PowerShell الأصلية { #persistencesniper-the-original-powershell-tool }

يُعد [**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper) مشروع PowerShell الأصلي
للبحث عن آليات الاستمرارية الذي ألهم PyrsistenceSniper. وينسب PyrsistenceSniper الفضل إلى
Federico Lagrasta؛ كما ينسب المستودع الأصلي الفضل إلى Riccardo Ancarani ومساهمين
آخرين. ونشكر المشروع الأصلي ومساهميه على تسهيل التحقيق في آليات الاستمرارية
في Windows.

المرجع المباشر لنقل Juntei هو مشروع Python. ويُنسب الفضل إلى أداة PowerShell الأصلية
في سلسلة النسب تلك. كما يطبّق Juntei بشكل مستقل الآليات التي اقترحها
[hongson11698](https://github.com/hongson11698) في
[طلب السحب PR #31 لحل COM في المهام المجدولة](https://github.com/last-byte/PersistenceSniper/pull/31)
و[طلب السحب PR #32 لمكتبات قائمة Start/مجلدات CLSID](https://github.com/last-byte/PersistenceSniper/pull/32) في المشروع الأصلي.
وتُضاف هذه إلى الفهرس المرجعي لمشروع Python؛ ولا يعني ذلك أن كل ميزة خاصة بالأنظمة الحية في أي من المشروعين الأصليين
مطبّقة في أمر Juntei الذي يعمل دون اتصال.

## الأبحاث والأدوات الداعمة { #supporting-research-and-tools }

- يوفر [MITRE ATT&CK](https://attack.mitre.org/) معرّفات التقنيات الواردة في
  الفهرس. ويصف ربط التقنية استخدامًا محتملًا لآلية ما؛ أما قيمة سجل Windows أو الملف المطابق
  فليس، بحد ذاته، هجومًا مؤكدًا وفق ATT&CK.
- كما يذكر PyrsistenceSniper أداة [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
  بوصفها مصدر إلهام لفحص التشغيل التلقائي في Windows.
- يوثّق [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS) الملفات الثنائية المشروعة في Windows التي يمكن
  إساءة استخدامها. ويساعد التعرف على أحد هذه الملفات الثنائية في تفسير الأمر، لكنه لا يجعل ذلك
  الأمر ضارًا تلقائيًا.

بالنسبة إلى أسس محللات سجل Windows وNTFS وShell Link وسجلات الأحداث وغيرها المستخدمة في كل أجزاء Juntei،
راجع [شكر وتقدير المشروع ككل](../acknowledgements.md).

ارجع إلى [دليل آليات الاستمرارية](index.md).
