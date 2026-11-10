# التحقيق الأول { #first-investigation }

يفترض سير العمل هذا أنك تلقيت مجموعة فرز (triage) غير متصلة في `evidence/`.

## 1. الحفاظ على المصدر { #1-preserve-the-source }

اعمل على نسخة جرى التحقق منها. سجّل التجزئات، ووقت الجمع، والمنطقة الزمنية، والمضيف، ووحدة التخزين،
وأداة الجمع. يقرأ Juntei الأدلة، لكن خيارات مثل استخراج الدفقات المقيمة أو الحجر تُنشئ ملفات إخراج
جديدة؛ فوجّهها إلى مجلد تحليل منفصل.

## 2. جرد ما لديك { #2-inventory-what-you-have }

ابحث عن المسارات الواردة في [جمع الأدلة](collection.md). من الطبيعي أن تكون بعض الآثار الرقمية مفقودة.
ولا تفترض أن المجموعة تحتوي على سجلات المعاملات لمجرد أنها تحتوي على خلية من سجل Windows.

## 3. شغّل الملخصات أولًا { #3-run-summaries-first }

تطبع أوامر كثيرة ملخصًا موجزًا عندما لا يُطلب ملف إخراج:

```bash
juntei parse-file prefetch -d evidence/
juntei parse-registry shimcache -d evidence/
juntei parse-file defender-quarantine -d evidence/
```

اقرأ تحذيرات التلف، وقواعد البيانات غير النظيفة (dirty)، والسجلات غير المطبّقة قبل الوثوق بالسجلات.

لتحليل كل أثر رقمي مدعوم في مجموعة تضم عدة حواسيب بعملية واحدة، استخدم:

```bash
juntei auto-parse -d evidence/ -o analysis/
```

راجع [تحليل البيانات المجمعة تلقائيًا](../commands/auto-parse.md) لمعرفة اكتشاف المضيفين، والتوازي،
وتخطيط الإخراج، وملف JSONL الموحّد للإدخال في SIEM.

## 4. احفظ إخراجًا منظمًا { #4-save-structured-output }

```bash
mkdir -p analysis
juntei parse-file prefetch -d evidence/ -T -o analysis/prefetch.csv
juntei parse-ntfs mft -d evidence/ -o analysis/mft
juntei parse-ntfs usn-journal -d evidence/ -o analysis/usn
juntei parse-registry services -d evidence/ -o analysis/services.jsonl -t jsonl
```

فضّل JSONL للبرمجة النصية والقيم المتداخلة، وCSV لجداول البيانات أو عارض الجداول الزمنية.

## 5. انطلق من معرّف موثوق { #5-pivot-from-a-reliable-identifier }

ابدأ باسم ملف، أو مسار كامل، أو SHA-1/SHA-256، أو معرّف SID لمستخدم، أو رقم إدخال/تسلسل في MFT، أو
معرّف GUID لمهمة، أو اسم خدمة، أو نافذة زمنية. ولا توحّد حالة الأحرف إلا في الحقول التي يتعامل معها
Windows دون تمييز لحالة الأحرف.

```bash
juntei parse-file prefetch -d evidence/ -t jsonl \
  | jq 'select(.ExecutableName? | ascii_downcase == "suspicious.exe")'
```

## 6. اربط الأدلة { #6-correlate }

استخدم [التحقيق عبر عدة آثار رقمية](../resources/workflows.md). ومن التسلسلات العملية:

1. Prefetch للتنفيذ وأوقات التشغيل الأخيرة.
2. `$MFT` ودفتر يومية USN للإنشاء، والمسارات، وإعادة التسمية، والحذف.
3. Amcache وShimCache للوجود التاريخي والبيانات الوصفية.
4. LNK وJump Lists وRecentDocs وSRUM لسياق المستخدم/التطبيق.
5. خدمات سجل Windows وTaskCache للاستمرارية.
6. حجر Defender للاكتشاف، والإزالة، والمصدر، والتجزئات.

## 7. وثّق قابلية إعادة الإنتاج { #7-document-reproducibility }

سجّل مسار المصدر، وتجزئة المصدر، وإصدار Juntei، والأمر الدقيق، وتجزئة الإخراج، والمنطقة الزمنية
للنظام، وأي تصفية لاحقة. ولا تحرّر أبدًا إخراج الأدلة المُولَّد بصمت.
