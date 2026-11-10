# Знімки екрана { #screenshots }

Ознайомтеся з командним рядком Juntei, автоматичним аналізом і впорядкованими результатами розслідування.
Клацніть знімок екрана, щоб переглянути його в повному розмірі.

## 1. Запуск і команди { #1-startup-and-commands }

<figure markdown>

[![Стартовий екран Juntei з випуском CODE BLUE і доступними командами](assets/screenshots/01-Startup.png)](assets/screenshots/01-Startup.png)

<figcaption>Стартовий екран Juntei перелічує автоматичний аналіз, парсери для окремих джерел і Persistence Hunter.</figcaption>

</figure>

## 2. Автоматичний аналіз менш ніж за 20 секунд { #2-automatic-parsing-in-under-20-seconds }

На **MacBook Pro M4** показаний нижче запуск проаналізував **8,7 GiB зібраних даних Windows**
із **шести систем** і зберіг **4,4 GiB виводу менш ніж за 20 секунд**.
Команда [auto-parse](commands/auto-parse.md) виконала 240 завдань парсерів у 16 паралельних
процесах і створила 482 файли результатів. Було **210 успішних завдань, 30 пропущених,
оскільки артефакти не було зібрано, і жодного збою**.

<figure markdown>

[![Juntei auto-parse завершує обробку шести систем: 8,7 GiB вхідних даних і 4,4 GiB виводу менш ніж за 20 секунд](assets/screenshots/02-AutoParse.png)](assets/screenshots/02-AutoParse.png)

<figcaption>Автоматичне виявлення хостів, паралельний аналіз і підсумок завершення з розміром вхідних даних, розміром виводу, результатами завдань і витраченим часом.</figcaption>

</figure>

## 3. Результати, впорядковані для розслідування { #3-results-organized-for-investigation }

<figure markdown>

[![Папки виводу однієї системи, згруповані за категоріями DFIR, з CSV-файлами облікових записів і журналами їх парсерів](assets/screenshots/03-OutputFolders.png)](assets/screenshots/03-OutputFolders.png)

<figcaption>Результати кожного комп'ютера згруповано за категоріями DFIR. CSV-файли артефактів і відповідні їм файли <code>-logs.txt</code> зберігаються разом, тож легко знайти і записи, і подробиці аналізу.</figcaption>

</figure>
