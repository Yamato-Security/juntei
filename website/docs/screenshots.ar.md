# لقطات الشاشة { #screenshots }

استكشف سطر أوامر Juntei، والتحليل التلقائي، ونتائج التحقيق المنظمة.
انقر على أي لقطة شاشة لعرضها بالحجم الكامل.

## 1. بدء التشغيل والأوامر { #1-startup-and-commands }

<figure markdown>

[![شاشة بدء تشغيل Juntei تعرض إصدار CODE BLUE والأوامر المتاحة](assets/screenshots/01-Startup.png)](assets/screenshots/01-Startup.png)

<figcaption>تعرض شاشة بدء تشغيل Juntei التحليل التلقائي، والمحللات الخاصة بكل مصدر، وPersistence Hunter.</figcaption>

</figure>

## 2. تحليل تلقائي في أقل من 20 ثانية { #2-automatic-parsing-in-under-20-seconds }

على جهاز **MacBook Pro M4**، حلّل التشغيل الموضح أدناه **8.7 GiB من بيانات Windows المجمعة**
من **ستة أنظمة** وحفظ **4.4 GiB من المخرجات في أقل من 20 ثانية**.
نفّذ الأمر [auto-parse](commands/auto-parse.md) ما مجموعه 240 مهمة تحليل باستخدام 16 عملية
متوازية، وأنتج 482 ملف نتائج. وكانت النتيجة **210 مهمة ناجحة، و30 مهمة تم تخطيها
لأن الآثار الرقمية لم تُجمع، دون أي حالة فشل**.

<figure markdown>

[![الأمر auto-parse في Juntei وهو يُكمل تحليل ستة أنظمة، بحجم إدخال 8.7 GiB وإخراج 4.4 GiB في أقل من 20 ثانية](assets/screenshots/02-AutoParse.png)](assets/screenshots/02-AutoParse.png)

<figcaption>اكتشاف تلقائي للمضيفين، وتحليل متوازٍ، وملخص للإكمال يعرض حجم الإدخال وحجم الإخراج ونتائج المهام والوقت المنقضي.</figcaption>

</figure>

## 3. نتائج منظمة لأغراض التحقيق { #3-results-organized-for-investigation }

<figure markdown>

[![مجلدات الإخراج لنظام واحد مجمّعة حسب فئات DFIR، مع ملفات CSV الخاصة بالحسابات وسجلات المحلل المرتبطة بها](assets/screenshots/03-OutputFolders.png)](assets/screenshots/03-OutputFolders.png)

<figcaption>تُجمَّع نتائج كل حاسوب حسب فئة DFIR. وتبقى ملفات CSV الخاصة بالآثار الرقمية وملفات <code>-logs.txt</code> المطابقة لها معًا، مما يسهّل العثور على السجلات وتفاصيل التحليل على حد سواء.</figcaption>

</figure>
