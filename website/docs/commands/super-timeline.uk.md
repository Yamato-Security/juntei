# Супер Хронологія { #super-timeline }

`super-timeline` перетворює проаналізовані артефакти на події із загальною схемою та сортує їх за часом. Запис Prefetch може містити кілька разів виконання; запис MFT може містити вісім або більше позначок часу файлу. Кожна з них стає окремою подією. Це спрощує порівняння доказів із різних джерел, не втрачаючи значення кожної позначки часу.

Почніть із результатів [`auto-parse`](auto-parse.md), а не з оригінального каталогу доказів:

```bash
juntei auto-parse -d evidence/ -o parsed/
juntei super-timeline -d parsed/ -o SuperTimeline.csv
```

Зберігайте проаналізовані файли, маніфести та оригінальні докази. Хронологія – це прогноз для аналізу; він не замінює ці записи та не створює власний файл зберігання Plaso.

## Використання { #usage }

```bash
# Native CSV (default)
juntei super-timeline -d parsed/ -o SuperTimeline.csv

# Native JSON Lines
juntei super-timeline -d parsed/ -o SuperTimeline.jsonl -t jsonl

# Combine native records and an existing Hayabusa JSONL timeline
juntei super-timeline -f NativeRecords.jsonl --hayabusa Hayabusa.jsonl \
  -o SuperTimeline.jsonl -t jsonl

# Timesketch import profile; also available with CSV
juntei super-timeline -d parsed/ -o Timesketch.jsonl -t jsonl --profile timesketch

# One standalone CSV needs an artifact identifier and, if unavailable, a computer name
juntei super-timeline -f MFT.csv --input-artifact mft --computer HOST01 \
  -o Host01Timeline.csv

# Reduce the time range, memory used by sorting, or number of workers
juntei super-timeline -d parsed/ -o IncidentTimeline.jsonl -t jsonl \
  --timeline-start "2026-10-20T06:00:00Z" --timeline-end "2026-10-20T14:00:00Z" \
  --memory-limit 64 -T 4 --temp-dir scratch/
```

`--hayabusa FILE` можна повторити або використати самостійно. Він імпортує наявні мітки часу події/сповіщення та контекст правила без запуску Hayabusa. Підтримуються інформаційні події, `Details` яких є рядком. `--artifact mft,prefetch` обмежує проекцію вибраними ідентифікаторами артефактів. Часові обмеження включають і вимагають явного зміщення UTC або `Z`.

### Вибір входу { #choosing-an-input }

| Введення | Як його ідентифікують |
|---|---|
| `-d DIR` | `manifest.json` надає успішні основні завдання, ідентифікатори артефактів, комп’ютери та точні вихідні файли. Вторинні завдання `auto-parse -j` виключаються, щоб запобігти імпорту обох копій. |
| `-f FILE` | Нативний JSON/JSONL використовує вбудовані поля `Artifact` і `Comp`. Для експорту без тегів потрібен `--input-artifact`; автономний CSV завжди вимагає його. |
| `--computer NAME` | Резервний варіант, коли вхідний запис не має імені комп’ютера; в іншому випадку невідомий комп’ютер буде явно `Unknown`. |
| `--hayabusa FILE` | JSONL із мітками часу Hayabusa, метаданими правила, комп’ютером і, якщо є, `EvtxFile`. |

Поточні маніфести підтримують переміщення проаналізованого каталогу: записані абсолютні вихідні шляхи вирішуються в новому каталозі. Вхідні дані мають залишатися всередині нього, включаючи цілі символічного посилання. Підтримуються застарілі каталоги Amcache, SRUM і Jump Lists для кожного артефакту; каталоги категорій потребують явного `OutputFiles`.

Підтримуються поточні маніфести з відносними шляхами виведення та зведені багатотабличні експорти. Команда розпізнає кожен артефакт, запланований автоматичним аналізом, включно з новішим браузером, таблицями WebCache, Outlook і UAL.

Коли маніфест визначає зображення VHDX або E01, посилання на джерело містять зображення та внутрішній шлях, а ім’я зображення зберігається в контексті події. Тимчасова проміжна робоча область не потрібна для читання проаналізованих результатів.

Необроблене джерело береться із запису або однозначної команди маніфесту. З `-d` посилання на необроблені джерела відносяться до каталогу доказів, а `Details.ParsedFile` і резюме `InputFiles` відносяться до каталогу проаналізованого аналізу. Назви підсумкових вихідних даних/звітів відносяться до каталогу вихідних даних часової шкали. Таблиці SRUM CSV призначаються базі даних лише тоді, коли на хості є один зібраний SRUDB.dat, який можна унікально ідентифікувати. В іншому випадку `SourceFile` вказує на проаналізований файл, а `Details.RawSourceAvailable` — це `false`.

На окремий вхід посилається його ім’я файлу. Вхідні дані з однаковою назвою файлу використовують різні посилання `./Inputs/<ID>/<filename>`. Абсолютні вихідні шляхи за межами відомих коренів доказів використовують ідентифікатори `./Sources/<ID>/<filename>`. Ці непрозорі посилання зберігають розрізнення джерел без експорту абсолютних імен каталогів аналітика; зверніться до рідного запису, використовуючи `ParsedFile` і `InputRecord` як першоджерело. Шляхи, записані в свідоцтві, наприклад шляхи до виконуваних файлів Windows, залишаються незмінними.

## Загальні поля подій { #common-event-fields }

| Поле | Значення |
|---|---|
| `Timestamp` | UTC RFC 3339, написаний дев’ятьма дробовими цифрами для узгодженого впорядкування. Набивка не додає точності. |
| `Comp` | Комп'ютер, пов'язаний із вхідним записом. |
| `Artifact` | Стабільний ідентифікатор артефакту, наприклад `mft`, `prefetch`, `srum` або `hayabusa`. |
| `TimestampType` | Опис значення позначки часу, наприклад `MFT SI Data modified`. |
| `User` | Записаний обліковий запис або SID, якщо його має адаптер; може бути порожнім. |
| `Path` | Уражений файл, розділ реєстру або інший об’єкт, пов’язаний із артефактом. |
| `Message` | Короткий опис для перегляду хронології. |
| `SourceFile` | Відносне до доказів посилання на необроблене джерело або відносний ідентифікатор джерела, якщо його корінь невідомий; інакше резервний варіант аналізованого файлу, описаний вище. |
| `SourceField` | Точне проаналізоване поле або вкладене місце, що використовується для мітки часу. |
| `RecordID` | Специфічна ідентифікація запису артефакту. Це не обов’язково запис EVTX ID. |
| `Details` | Структурований контекст, включаючи оригінальну позначку часу, точність, часову основу, часову роль, проаналізований файл, вхідний запис/розташування та доступні метадані відновлення/правила. |

У CSV `Details` є клітинкою JSON у лапках. У JSONL це об’єкт. Профіль Timesketch змінює `Timestamp`, `TimestampType` і `Message` на `datetime`, `timestamp_desc` і `message` відповідно, зберігаючи інші поля походження. Це [ три обов’язкові поля імпорту Timesketch ](https://timesketch.org/guides/user/import-from-json-csv/).

`Details` навмисно містить вибраний контекст, а не повні двійкові дані чи кожне поле рідного запису. Довгі рядки, масиви та об’єкти обмежені та позначені `ProjectionOmitted`. Знайдіть `Details.ParsedFile`, `InputRecord` і `Location`, щоб перевірити повний рідний запис. Для CSV номер запису враховує записи CSV після заголовка, а не фізичні рядки в клітинках у лапках. Для JSONL це номер рядка; для JSON це номер документа верхнього рівня плюс вкладене розташування.

## Що означають часи { #what-the-times-mean }

Мітка часу є свідченням записаного об’єкта. Сам по собі він не є доказом страти чи нападу. Адаптери навмисно зберігають ці відмінності:

| Артефакти | Інтерпретація шкали часу |
|---|---|
| MFT | Окремий час модифікації, доступу, зміни метаданих і створення SI і FN; названі атрибути FN залишаються різними. Повторювані поля SI у рядках імен файлів консолідуються. ADS час спільного базового файлу рядків виключено. Імена відновлених записів FILE зберігають окремі події відновлення FN. |
| I30 | Збережені метадані FN/індекс із контекстом затримки/відновлення, якщо доступний. Це не мітки часу запису індексу каталогу. |
| USN Журнал | Мітка часу запису журналу та причини оновлення. Запис може описувати декілька змін; він не обов’язково записує їхній точний час початку. |
| LogFile | Збережені знімки часових позначок повторення/скасування та метадані імені файлу. Повторити та скасувати залишаються окремо. LSN і зміщення файлів є ідентифікаторами, які ніколи не перетворюються на дати. |
| Prefetch | Записаний час виконання, консолідований із відповідним експортом часової шкали Prefetch. Метадані створення тому, на які посилаються, позначаються окремо. |
| Браузерні бази даних SQLite | Записані відвідування, відновлені метадані останнього відвідування сторінки, початок/кінець завантаження, створення/доступ/термін дії файлів cookie та надгробки історії першого/останнього використання чи видалення. Завершення завантаження саме по собі не є доказом успіху чи виконання; закінчення терміну дії файлів cookie – це налаштований термін. Статус відновлення, впевненість і походження залишаються в контексті. |
| Internet Explorer / застарілий Edge WebCache | Доступ до збережених записів, створення/модифікація, закінчення терміну дії та синхронізація, початок завантаження, метадані файлів cookie та доступ до контейнера/переробка. Кеш-бухгалтерія не є автоматичним доказом того, що користувач відкрив сторінку. |
| Outlook PST, OST, MSG і OFT | Надсилання/доставка повідомлення; створення/змінення папок, елементів і вкладень; початок/кінець/нагадування календаря; початок/термін виконання/завершення завдання/нагадування; і явно введені властивості часу. Запланований час не доводить, що зустріч відбулася, нагадування запущено чи програма виконана. Збережений час доставки не підтверджує, що повідомлення було прочитано. |
| Журнал доступу користувачів (UAL) | Метадані першого/останнього доступу, вставлення/створення бази даних, перше/останнє відвідування та збережений час завантаження операційної системи. Роль, обліковий запис, адреса джерела та щоденні лічильники залишаються в контексті. Щоденні дати/підрахунки не є придуманими опівнічними входами; рядки untimed identity та DNS залишаються доступними у супровідному звіті. |
| Amcache | Останні записи ключа інвентаризації, контекст батьківського ключа тому, записані дати встановлення та метадані посилання PE. Наявність інвентаризації або LinkDate сама по собі не є доказом виконання. |
| ShimCache | Збережений час зміни файлу, метадані оновлення кешу та останнього запису ключа кешу. Жодне з цих значень саме по собі не є часом виконання. |
| SRUM | Позначка часу/сегмент постачальника, початок з’єднання, явні поля інтервалу початку/завершення та метадані ідентифікатора програми. Байти, тривалість та інші лічильники не є датами. |
| LNK | Цільові позначки часу MAC, час створення ідентифікатора відстеження та час елемента оболонки. Вони описують збережені цільові метадані, не обов’язково, коли було відкрито ярлик. |
| Jump Lists | Entry last-use metadata, tracking-identifier creation and embedded LNK target times. За замовчуванням час файлу контейнера на стороні збору не враховано. |
| Defender карантин | Мітка часу введення та час створення/доступу/запису збережених ресурсів. Час вилучення не є доказом активності кінцевої точки. |
| BAM, UserAssist, FeatureUsage | Записані поля активності, де вони присутні, із ключовим контекстом останнього запису, який зберігається чітким. Лічильник без часу не створює події. |
| RecentDocs, MRU, відкрити/зберегти MRU, ShellBags, Office | Recorded use/trust/shell-item times and key context. Ключовий час списку – це не окремий час останнього використання кожного елемента в списку. |
| Секрети SAM і LSA | Записані метадані входу до облікового запису/пароль/закінчення терміну дії або секретні налаштування та відповідний час ключа. Вони не замінюють журнали подій автентифікації. |
| Служби, TaskCache, збереження реєстру, BCD | Метадані ключа/конфігурації та явно записаний час виконання завдань. Service-key writes alone do not prove service execution. |
| Мережі та USB | Поля мережевого профілю/оренди та встановлення пристрою/прибуття/видалення з контекстом ключа реєстру. Native USB user MountPoints2 times retain the corresponding user-hive source. |
| Інформація про систему, встановлені програми, AppCompat, історія програм | Explicit install, shutdown, inventory/activity and key metadata fields. Власні масиви профілів/версій системної інформації зберігають вкладений час. |
| Audit policy, security settings, MuiCache, generic registry | Час останнього запису ключа, якщо він надається; untimed values/counters remain available in native input. |
| Persistence Hunter | Підтримка часу останнього запису ключа та корельованого часу зміни. Знахідка сама по собі не є інсталяцією з міткою часу чи вироком компромісу. |
| Hayabusa | Event/alert timestamp with rule, level, channel, event ID and available detail context. Multiple rules and distinct correlation rows remain separate. |

Multiple equal timestamps with different meanings or evidence sources are retained. Duplicate identity includes the computer, source, artifact, record, time meaning and recovery state. Консолідовані рядки записують `DuplicateCount` і обмежені альтернативні шляхи/вихідні поля. Distinct sources are never merged merely because their paths and timestamps look alike. When the raw source cannot be resolved, input-record identity is retained to avoid collapsing different captures.

### Точність, місцевий час і припущення { #precision-local-times-and-assumptions }

Known UTC fields and timestamps with explicit offsets are normalized to UTC. Original values and their recorded precision are retained, including 100 ns FILETIMEs. A source parser or downstream viewer may already have rounded a time; ця команда не може відновити втрачену точність.

Деякі поля shell-item/FAT мають семантику локального часу, навіть якщо попередній аналізатор серіалізує їх за допомогою `Z`. Шкала часу зберігає проаналізоване значення та походження поля; він не може відновити відсутній часовий пояс кінцевої точки або підтвердити конверсію вгору. Перевірте необроблені параметри артефакту та кінцевої точки, перш ніж використовувати такий час для точної міжхостової кореляції.

A naive timestamp with no artifact-defined time zone is diagnosed, rather than silently treated as UTC. Якщо докази виправдовують це, надайте підписане фіксоване зміщення:

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

Для значень лише дати також потрібна ця опція. Їх опівнічний якір позначено як припущення з **денною точністю**; це не спостережувана подія опівночі. Назви часових поясів і правила переходу на літній час не приймаються. Відсутні, нульові, недійсні значення та значення, що виходять за межі діапазону, не створюють синтетичних подій епохи. Явно записана дійсна дата 1970 зберігається.

За умовчанням виключено дві додаткові категорії:

- `--include-collection-times`: метадані вихідного файлу LNK/Jump List у файловій системі збору/аналізу. Ці події позначаються `collection_filesystem` окремо від активності кінцевої точки.
- `--include-interpreted-times`: generic registry QWORD values rendered as FILETIME by the registry reader. They are labeled `interpreted_value` because many such values are actually counters.

## Companion reports { #companion-reports }

Для `SuperTimeline.csv` команда також пише:

- `SuperTimeline.csv-Untimed.jsonl`: місцезнаходження власних записів без придатної позначки часу події.
- `SuperTimeline.csv-Diagnostics.jsonl`: invalid/ambiguous times, intentionally skipped shared metadata and fidelity limitations that require the original parsed record.
- `SuperTimeline.csv-Summary.json`: вхідні файли, кількість артефактів, відфільтровані події/кандидати/консолідовані події, кількість вихідних даних, працівники, сортування бюджету/фрагментів/пропусків і час, що минув.

Супутні імена включають повну назву вихідного файлу. Наприклад, `SuperTimeline.jsonl` має власний `SuperTimeline.jsonl-Summary.json` і відповідні незаплановані/діагностичні звіти. Експорт CSV і JSONL з однаковою основою можуть співіснувати; `-C` потрібен лише для заміни існуючого набору результатів.

Дійсні дати, що очікують обґрунтованого збільшення припущення часового поясу `UnresolvedTimeZones`, а не `InvalidTimestamps`. Обидва залишаються простеженими в діагностиці; жодної позначки часу не винайдено.

Записи з дійсним часом поза запитуваним діапазоном вважаються відфільтрованими, а не невизначеними. Candidate event counts precede duplicate consolidation. `Complete: true` означає, що всі вибрані проаналізовані вхідні дані оброблено успішно; it does not mean every artifact was collected, every field is timestamped or every attack is detectable.

Експорт CSV може згладити вкладені метадані: профілі системної інформації, точки монтування користувача USB, елементи оболонки та інший структурований контекст можуть бути багатшими рідними JSON/JSONL. Використовуйте власне введення, коли ці поля важливі. The timeline cannot reconstruct data omitted by a CSV export. Непідтримувані ідентифікатори артефактів і неправильно сформовані вхідні файли викликають помилку замість успіху, що містить часткові результати.

## Продуктивність і безпечність виходу { #performance-and-output-safety }

Вхідні дані передаються та обробляються паралельно. Sort buffers spill to a private temporary directory, then bounded merge passes produce deterministic chronological output. Для сортування використовуються цілі секунди та наносекунди, включаючи дати до 1970. Equal-time ordering is deterministic across worker counts and sort budgets.

`--memory-limit` is the aggregate **sort-buffer budget**, not a process RSS limit. Input records, JSON object/string overhead, CSV buffers and merge records require additional memory. Для невеликих бюджетів кількість працівників зменшується. `--max-record-size` обмежує аналізований вхідний запис (за замовчуванням 16 MiB); a pretty JSON array/report counts as one top-level input document. Для великих документів може знадобитися більший ліміт або, бажано, JSONL. Випереджальне читання аналізатора CSV також сприяє цьому обмеженню.

Temporary disk space must accommodate the expanded events and merge passes, in addition to final results. Запис може давати багато подій, тому шкала часу може бути набагато більшою за вхідні дані. Filter the time range/artifacts when appropriate and select a temporary volume with adequate free space.

Ctrl+C скасовує кооперацію, видаляє приватні каталоги сортування/постановки та виходить із кодом `130`. Для Linux і macOS це також стосується SIGTERM і SIGHUP. Скасування перевіряється під час читання, нормалізації, об’єднання та написання звітів перед початком публікації. Невловиме припинення або втрата живлення все ще може залишати тимчасові файли.

Існуючі виходи потребують `-C`. Хронологія та супутні елементи не повинні перезаписувати вхідні дані, маніфести, локальні докази, на які посилаються, або їх захищені каталоги доказів. Помилки введення або помилки сортування залишають існуючі виходи недоторканими. Completed files are staged beside the destination and replaced individually, publishing the timeline last. This is **not** a transaction across all four files: an I/O failure during final publication can leave some companion reports replaced. Keep a new output basename for each run when retaining an earlier complete export matters.

## Further details { #further-details }

- [Timesketch CSV/JSONL import](https://timesketch.org/guides/user/import-from-json-csv/)
- [Plaso timestamp expansion and legacy CSV](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Вивід і форматування Plaso](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [Evidence and interpretation](../overview/evidence-model.md)
- [Вихідні формати](../getting-started/output-formats.md)

Поточні вхідні дані Juntei використовують скорочені мітки, такі як `Comp`, `Filename`, `Args` і `Attr`. MFT CSV також приймає поточні `File`, `SN`, `ParentEntryNum`, `ParentSN`, `Type`, `USN`, `LogfileSN` і Заголовки `HasADS`, зберігаючи генерації порядкових номерів і розрізнення DOS/ADS недоторканими. Старіші експортовані мітки залишаються прийнятними, а `SourceField` зберігає правопис у фактичному введенні. Hayabusa input retains its own `Computer` field. Метадані завантажувального сектора та дескриптора безпеки зберігаються у звіті без часу, оскільки він не надає мітки часу події.

## Новіші багатотабличні аналізатори { #newer-multi-table-parsers }

Рідна JSON/JSONL містить кожну назву таблиці. For a standalone CSV, the browser, WebCache and Outlook adapters identify its table from the column schema, so renaming the CSV or using automatic parsing's source suffixes does not change timestamp interpretation. UAL зберігає свої явні поля позначки часу; Лічильники доступу та непов'язаний текст, що виглядає як дата, не вгадуються як дати.

Рядки властивостей Outlook проектуються лише тоді, коли їх оголошений тип є часовим або багаторазовим. Рядок, що містить дату, залишається рядком. Вбудовані повідомлення зберігають свої ідентифікатори елементів і батьківські посилання, а час вилучення вкладень не замінює збережені позначки часу вкладень. Рядки файлів, одержувачів, контактів або властивостей, які не мають часу, залишаються відстежуваними у вихідних даних і звіті без часу.

До цих аналізаторів застосовуються однакові вихідні схеми CSV і JSONL; додавання нового артефакту не додає сотні стовпців CSV. Специфічний контекст артефакту залишається в `Details`.
