# أساليب الإخراج الشائعة { #common-output-patterns }

## أثر رقمي واحد أو مجموعة مجمّعة { #one-artifact-or-a-collection }

```bash
juntei parse-file prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-file prefetch -d evidence/
```

يبحث `-d` بشكل تكراري عن الملفات المناسبة لذلك الأمر. وتميّز أعمدة مسار المصدر بين المضيفين والآثار
الرقمية في الإخراج المدمج.

## حفظ CSV { #save-csv }

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

تحتاج بعض المحللات إلى مجلد لأنها تُخرج عدة جداول CSV:

```bash
juntei parse-registry amcache -f Amcache.hve -o amcache-tables
juntei parse-ese srum -f SRUDB.dat -o srum-tables
juntei parse-file jump-lists -d evidence/ -o jump-list-tables
```

## بث JSONL { #stream-jsonl }

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.ImagePath? | test("temp|public"; "i"))'
```

تُرسل الرسائل ومؤشرات التقدم إلى الخطأ القياسي (standard error)؛ ويبقى الإخراج القياسي قابلًا للتحليل.

## الكتابة فوق الملفات عن قصد { #overwrite-intentionally }

يرفض Juntei افتراضيًا الكتابة فوق إخراج موجود. تأكد من الهدف، واحتفظ بأي نتيجة سابقة مهمة، ثم أضف
`-C` / `--clobber`:

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv --clobber
```

## الوضع الصامت ووضع إلغاء الألوان { #quiet-and-no-color-modes }

استخدم `-q` للأتمتة و`-K` عندما تكون رموز ألوان الطرفية غير مناسبة:

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv -q -K
```

## حالة الأحرف والمسارات { #case-and-paths }

ضع مسارات Windows التي تحتوي على مسافات، أو `$`، أو أحرف الصدفة الخاصة (shell metacharacters)، أو
الشرطات المائلة العكسية بين علامات اقتباس حسب ما تتطلبه الصدفة لديك. وفي الصدفات المشابهة لـ Bash،
تمنع علامات الاقتباس المفردة تحوّل `$MFT` و`$J` إلى متغيرات:

```bash
juntei parse-ntfs mft -f '/evidence/C/$MFT' -o mft.csv
```
