# Автоматичний аналіз зібраних даних { #automatic-collection-parsing }

`auto-parse` — найшвидший спосіб перетворити зібрані для первинного аналізу дані Windows з кількох комп'ютерів на
структурований каталог для аналізу. Укажіть корінь доказів і новий каталог виводу. Команда виявляє
зібрані системи, визначає імена їхніх комп'ютерів і паралельно запускає кожен сумісний парсер
Juntei.

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

Типовий формат виводу — CSV. Додайте хронологію JSONL для `jq` або імпорту в SIEM, зберігши при цьому
результати CSV:

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "Тримайте дерева виводу та доказів окремо"
    Juntei відхиляє каталог виводу, який розташований усередині `<EVIDENCE>` або містить `<EVIDENCE>`. Це
    запобігає повторному виявленню згенерованих даних як доказів і не дає `--clobber`
    торкатися файлів у вихідній колекції.

## Образи дисків VHDX { #vhdx-disk-images }

З параметром `--vhdx` команда `auto-parse` аналізує системи в кожному образі диска VHDX у каталозі доказів. Під час аналізу кожен образ монтується через FUSE лише для читання; файли не розпаковуються. Див. [Монтування образів дисків](mount.md).

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

Якщо том NTFS не відкривається, Juntei повідомляє про проблему та зберігає доступ до читабельних томів. Для частково читабельного образу `auto-parse --vhdx` повертає код завершення 1; `mount` також повертає 1 після відмонтування.

## Часовий пояс виводу { #output-time-zone }

Позначки часу після аналізу за замовчуванням подаються в UTC, незалежно від місцевого часового поясу комп'ютера аналітика. UTC
використовує форму журналу подій Windows `2026-10-20T08:46:38.2001668Z`: дата/час ISO 8601, `T`
між датою та часом і `Z` для UTC. Наявні дробові розряди зберігаються.
`GeneratedUTC` у `manifest.json` також використовує цю форму. `Start time` і `End time` у консолі
подаються в місцевому часі комп'ютера аналітика, наприклад `2026-11-04 09:15:02`, незалежно від часового
поясу виводу.

Виберіть фіксоване зміщення відображення за допомогою `-Z, --timezone`:

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

Зміщення — це значення `HH:MM` зі знаком і двома цифрами (години `00–23`, хвилини `00–59`). Назви на кшталт
`JST`, скорочені значення на кшталт `+9` і недійсні діапазони спричиняють помилку з прикладом синтаксису.
Пропустіть параметр або використайте `--timezone "+00:00"` для UTC. Фіксоване зміщення не застосовує правил
переходу на літній час.

Параметр застосовується до позначок часу після аналізу в результатах CSV, JSON і JSONL для кожного артефакту та
до додаткового експорту JSONL `-j`. Наприклад, `2026-10-20T08:46:38.2001668Z` стає
`2026-10-20T17:46:38.2001668+09:00`; обидва значення описують той самий момент.

Назви полів залишаються незмінними, включно з наявними назвами, що закінчуються на `UTC`. Перетворені значення явно
містять вибране зміщення. `OutputTimeZone` в обох маніфестах фіксує запитане зміщення
відображення, тоді як `GeneratedUTC` залишається метаданими запуску в UTC. Журнали окремих парсерів зберігають власні зведення парсерів в UTC.

Сирі числа FILETIME/epoch, словники значень реєстру, конфігурація часового поясу джерела, текстові
метадані, значення, що містять лише дату, і значення місцевого годинника без відомого зміщення зберігаються без змін. Juntei
не додає `Z` до невідомого місцевого часу. Значення годинника елементів оболонки, отримані з FAT (`Item*UTC` у
ShellBags/OpenSaveMRU і `TargetItems` у LNK/Jump List), не мають записаного зміщення; їхні успадковані
мітки UTC або відображений суфікс `Z` не доводять UTC, тому параметр зміщення зберігає їх без змін.
Стовпці CSV, що містять довільні текстові зведення, зберігають
записаний текст; JSON/JSONL зберігає вкладені позначки часу після аналізу як структуровані поля.

Для UTC безпосередньо використовується наявний вивід парсера. Явне ненульове зміщення додає потоковий
прохід форматування в кожному робочому процесі парсера; він змінює значення відображення, не змінюючи доказів.

Якщо зміщення перемістило б позначку часу за межі років `0000–9999`, це значення зберігає своє початкове
явне зміщення. Значення з понад дев'ятьма дробовими розрядами також залишаються без змін, а не
округлюються. Juntei записує попередження у файл `*-logs.txt` парсера і перетворює інші
позначки часу. Некоректний CSV/JSON або помилка виводу все одно призводять до збою завдання і залишають відповідний
початковий файл результатів без змін.

## Що вона виявляє { #what-it-discovers }

Juntei рекурсивно шукає каталоги з назвою `Windows`, які містять каталог `System32`, `Prefetch`
або `AppCompat`, без урахування регістру. Розпізнавання двох останніх також підтримує цільові
колекції, що містять лише Prefetch або Amcache. Це виключає каталоги профілів користувачів, як-от
`AppData\Roaming\Microsoft\Windows`. Вкладені застарілі інсталяції, як-от
`C:\Windows.old\Windows`, призначаються зовнішньому зібраному тому, а не стають дублікатами
комп'ютерів. Типова максимальна глибина — 10. Це охоплює поширені шляхи Velociraptor, як-от:

```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

Змінюйте обмеження лише тоді, коли колекція вкладена глибше:

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

Для кожної виявленої системи ім'я комп'ютера вибирається в такому порядку:

1. `hostname` у найближчому файлі-предку `client_info.json` (його значення зберігається як `FQDN` у маніфесті).
2. `ComputerName` у зібраному кущі реєстру `SYSTEM`.
3. Найближча придатна назва каталогу колекції, з пропуском диска та службової структури Velociraptor `uploads/auto`
   і вилученням суфікса на кшталт `_F.…`.

Імена перетворюються на безпечні для використання як назви каталогів. Повторювані імена отримують числовий суфікс як у
каталозі виводу, так і в значенні `Comp`, щоб окремі системи залишалися розрізнюваними в JSON.
Маніфест зберігає ім'я хоста точно в тому вигляді, як його виявлено, в окремому полі `Hostname`.

Після переліку систем `auto-parse` виводить один рядок `Timezone:` безпосередньо над
`Parser tasks`. Він показує вибраний часовий пояс виводу: `UTC` за замовчуванням або значення на кшталт
`UTC+09:00`, коли використано `--timezone "+09:00"`. Мітка зелена, а значення має
типовий колір тексту термінала, як і інші значення зведення. `-K, --no-color` залишає рядок без кольору.
Конфігурація часового поясу зібраних систем доступна в результатах
[Відомості про систему](../artifacts/system-info.md).

## Парсери, які вона запускає { #parsers-it-runs }

Збій одного парсера не зупиняє інші. `auto-parse` запускає ці завдання для кожного виявленого
комп'ютера. Завдання використовують ті самі згруповані за джерелом команди, що й ручний аналіз, як-от
`parse-ntfs mft`, `parse-registry amcache`, `parse-ese srum` і `parse-file prefetch`:

- Amcache
- `$MFT`
- записи redo/undo NTFS з `$LogFile`
- записи Prefetch і окрема хронологія виконання Prefetch
- журнал USN
- індекси каталогів `$I30`
- завантажувальні сектори `$Boot` і дескриптори безпеки `$Secure:$SDS`
- SRUM
- журналювання доступу користувачів (UAL)
- історія, завантаження, файли cookie та історія форм веб-браузерів (браузери на основі Chromium і Firefox)
- WebCache Internet Explorer і застарілого Edge: історія, завантаження, файли cookie та записи кешу
- ярлики LNK
- Jump Lists
- ShimCache
- Persistence Hunter (131 перевірка закріплення за кількома джерелами з профілями дозволу/блокування)
- метадані карантину Defender
- історія діалогів відкриття та збереження файлів
- усі ключі та значення реєстру, включно з придатними для відновлення видаленими даними
- служби, RecentDocs, TaskCache, облікові записи SAM, розташування автозапуску в реєстрі, UserAssist, BAM,
  встановлені програми, MuiCache, ShellBags і пристрої USB
- відомості про систему: імена, версія Windows, часовий пояс, останнє завершення роботи, профілі та SID
- політика аудиту: які події безпеки система мала реєструвати
- конфігурація завантаження (BCD і BCD-Template)
- мережеві профілі, інтерфейси, спільні ресурси, проксі портів і маршрути
- імена та час секретів LSA, а також кешовані доменні входи, що використовуються (нічого не розшифровується)

Persistence Hunter зберігає всі рівні серйозності (`--min-severity info`), включно з частковими збігами з дозволами.
`LOW` — це класифікація для перевірки; відсутні метадані підпису можуть спричинити її і для шкідливих
команд. Автоматична інвентаризація не відкидає ці записи за типовим середнім порогом
ручної команди.

Hunter використовує один робочий потік у кожному процесі автоматичного парсера. Це не дає `auto-parse -T <N>`
помножити свій бюджет процесів на ще один пул розміром із кількість CPU у кожному дочірньому процесі. Окрема команда
`persistence-hunter -T <N>` керує власними робочими потоками hunter.

Вивід CSV `$MFT` включає всі позначки часу та короткі імена. Вивід реєстру включає сирі дані та
інтерпретацію значень `REG_QWORD` як FILETIME. Вивід SRUM у JSON і JSONL включає метадані стану
значень. Файли Defender і резидентні потоки `$MFT` автоматично **не вилучаються**.

З `--mft-slack` вивід `$MFT` також містить імена файлів, що залишилися в slack записів FILE, у тому ж вигляді, що й у [`parse-ntfs mft -x`](../artifacts/mft.md): рядки з позначкою `FromSlack` у CSV і `SlackFileNames` у JSON та JSONL. `--code-page` застосовується до файлів LNK, Jump List, ShellBags і OpenSaveMRU.

Артефакт, який не було зібрано, — це звичайне завдання зі станом `skipped`, а не збій.

## Структура виводу { #output-layout }

Результати впорядковано за комп'ютерами та категоріями розслідування. Каталоги категорій використовують
PascalCase, як-от `Accounts` і `FileAndFolderOpening`. Таблиці Amcache і Jump Lists розміщуються
безпосередньо у своїх категоріях із префіксами імен файлів. Назви каталогів комп'ютерів зберігають виявлене написання. Наявні каталоги виводу
не переносяться; вибирайте новий каталог виводу під час переходу зі старішої схеми найменування. Кожен парсер записує
файл `<artifact>-logs.txt` поруч зі своїми результатами, зокрема й тоді, коли артефакт не було зібрано.
CSV зберігає окремі таблиці, оскільки кожен артефакт має інші стовпці:

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── BrowserActivity/
│   │   ├── Browser-Cookies.csv
│   │   ├── Browser-Downloads.csv
│   │   ├── Browser-FormHistory.csv
│   │   ├── Browser-History.csv
│   │   ├── Browser-logs.txt
│   │   ├── WebCache-Containers.csv
│   │   ├── WebCache-Cookies.csv
│   │   ├── WebCache-Downloads.csv
│   │   ├── WebCache-Entries.csv
│   │   ├── WebCache-History.csv
│   │   └── WebCache-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   ├── Registry-RemoteAccess-logs.txt
│   │   ├── UAL-Clients.csv
│   │   ├── UAL-RoleAccess.csv
│   │   ├── UAL-SystemIdentity.csv
│   │   └── UAL-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```

| Категорія | Артефакти |
|---|---|
| `Accounts` | Локальні облікові записи SAM, імена та час секретів LSA |
| `Applications` | Історія програм (сліди сторонніх інструментів) |
| `Applications` | Встановлені програми |
| `BrowserActivity` | Історія, завантаження, файли cookie та історія форм веб-браузерів разом із відновленими видаленими записами; WebCache Internet Explorer і застарілого Edge |
| `ExternalDevices` | Пристрої та томи USB |
| `FileAndFolderOpening` | Jump Lists, ярлики LNK, Microsoft Office, списки MRU, діалоги відкриття та збереження файлів, RecentDocs, ShellBags |
| `FileExistence` | Amcache, ShimCache |
| `FileSystem` | `$Boot`, `$I30`, `$LogFile`, `$MFT`, `$Secure:$SDS`, журнал USN |
| `NetworkActivity` | Мережеві профілі, інтерфейси, спільні ресурси, проксі портів і маршрути; підключення, трафік і push-сповіщення SRUM |
| `LateralMovement` | Вихідні підключення користувачів через віддалений робочий стіл, PuTTY і WinSCP, підключені диски та спільні ресурси; журналювання доступу користувачів (клієнти ролей сервера) |
| `Persistence` | Конфігурація завантаження (BCD), автозапуск через реєстр, служби, TaskCache |
| `ProcessExecution` | Сумісність програм, BAM і DAM, FeatureUsage і закріплені елементи, MuiCache, Prefetch, хронологія Prefetch, активність програм/CPU у SRUM, UserAssist |
| `Registry` | Загальні ключі та значення кущів, включно з відновленими записами |
| `SecurityProducts` | Метадані карантину Defender і необов'язково вилучений вміст; параметри безпеки (Defender, UAC, захист облікових даних, RDP, брандмауер, журналювання) |
| `SystemInformation` | Імена, версія Windows, часовий пояс, останнє завершення роботи, профілі, SID і політика аудиту; енергоспоживання/ємність SRUM, карти ідентичностей і невідомі провайдери |

Категорії допомагають упорядкувати розслідування; вони не є доказом певної активності. Наприклад,
записи Amcache і ShimCache самі по собі не встановлюють факту виконання. SRUM записує використання ресурсів;
перш ніж робити висновки про запуск програми, зверніться до посібника з артефактів.

CSV, JSON і JSONL використовують ті самі категорії. Один файл `<artifact>-logs.txt` на кожне завдання парсера містить його
точну команду та повідомлення; наприклад, `HOST1/FileSystem/MFT-logs.txt` і
`HOST1/ProcessExecution/Prefetch-logs.txt`. JSON і JSONL використовують ті самі імена файлів журналів.
Amcache і Jump Lists мають по одному спільному журналу поруч зі своїми таблицями з префіксами. Таблиці провайдерів SRUM розміщуються безпосередньо
в `NetworkActivity`, `ProcessExecution` і `SystemInformation` зі спільним
файлом `SystemInformation/SRUM-logs.txt`. Підкаталоги `logs/` не створюються.

Типові імена файлів CSV: `Registry-SAM.csv`, `LNK.csv`, `Registry-RecentDocs.csv`, `Registry-ShimCache.csv`, `LogFile.csv`,
`I30.csv`, `MFT.csv`, `UsnJournal.csv`, `Boot.csv`, `SecureSDS.csv`, `Registry-Networks.csv`, `Registry-Persistence.csv`, `Registry-Services.csv`, `Registry-TaskCache.csv`, `Registry-BCD.csv`,
`Registry-SecuritySettings.csv`,
`PrefetchTimeline.csv`, `Prefetch.csv`, `Registry-UserAssist.csv`, `Registry-AppCompat.csv`, `Registry-Office.csv`, `Registry-FeatureUsage.csv`, `Registry-AppHistory.csv`, `Registry-MRU.csv`, `Registry-SystemInfo.csv`, `Registry-RemoteAccess.csv`, `Registry-OpenSaveMRU.csv`, `Registry-InstalledPrograms.csv`, `Registry-BAM.csv`, `Registry-MuiCache.csv`, `Registry-ShellBags.csv`, `Registry-USB.csv`, `Registry-SecuritySettings.csv`, `Registry-AuditPolicy.csv`, `Registry-Dump.csv` і
`DefenderQuarantine.csv`. Таблиці, отримані з реєстру, мають префікс `Registry-`, зокрема
`Registry-ShimCache.csv`, `Registry-UserAssist.csv`, `Registry-BAM.csv` і `Registry-SystemInfo.csv`.
Загальні ключі та значення кущів містяться в `Registry-Dump.csv`. Таблиці Amcache:
`FileExistence/Registry-Amcache-Devices.csv`, `Registry-Amcache-Files.csv`,
`Registry-Amcache-Findings.csv` і `Registry-Amcache-Summary.csv`. Jump Lists використовують
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` і
`JumpLists-CustomDestinations.csv`. Підкаталогів Amcache або Jump Lists немає.
Таблиці SRUM використовують імена файлів `SRUM-` безпосередньо у своїй категорії, наприклад
`NetworkActivity/SRUM-NetworkConnections.csv`.
Таблиці журналювання доступу користувачів розміщуються в `LateralMovement`, по одному файлу на таблицю, наприклад
`UAL-Clients.csv` і `UAL-SystemIdentity.csv`, разом із `UAL-logs.txt`; див.
[Журналювання доступу користувачів (UAL)](../artifacts/ual.md#automatic-parsing).
Таблиці веб-браузерів розміщуються в `BrowserActivity`: `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv` і `Browser-FormHistory.csv`, разом із `Browser-logs.txt`; див.
[Веб-браузери](../artifacts/browser.md#automatic-parsing). Там само розміщуються й таблиці WebCache
Internet Explorer і застарілого Edge: `WebCache-History.csv`, `WebCache-Downloads.csv`,
`WebCache-Cookies.csv`, `WebCache-Entries.csv` і `WebCache-Containers.csv`, разом із
`WebCache-logs.txt`; див. [Internet Explorer і застарілий Edge](../artifacts/webcache.md#automatic-parsing).
Див. [повну таблицю розміщення SRUM](../artifacts/srum.md#categorized-auto-parse-output).

Імена журналів відповідають назві артефакту з великої літери, як-от `Registry-SAM-logs.txt`, `PrefetchTimeline-logs.txt`
і `Registry-UserAssist-logs.txt`. Файли JSON/JSONL використовують ті самі основи імен, наприклад `MFT.jsonl`;
ідентифікатор `Artifact` запису для запитів — `mft`. Багатотабличні виводи мають
`Registry-Amcache-logs.txt` або `JumpLists-logs.txt` поруч зі своїми таблицями. Спільний журнал SRUM
міститься в `SystemInformation`.


`manifest.json` фіксує початково виявлене `Hostname`, унікальну ідентичність виводу/запису `Comp`,
походження імені хоста, шляхи до доказів, `Category` артефакту, командні рядки, тривалість, код
виходу, шлях до результату, шлях до журналу, запитану хронологію JSONL і будь-яку помилку експорту, чи було запитано вилучення з Defender,
а також чи кожне завдання
виконалося успішно, було пропущене або завершилося збоєм.

Auto-parse не записує жодного абсолютного шляху, тому результатами можна ділитися, не показуючи, де в аналітичній системі розташовані докази,
результати або Juntei:

- Шляхи до доказів задаються відносно каталогу доказів. Термінал перелічує кожен комп'ютер із його
  каталогом у доказах (`HOST1: ./HOST1_F`), а парсери запускаються в каталозі доказів і
  отримують такі шляхи. Стовпці шляху до джерела в результатах (`SourceFile`, `SourcePath`, …), журнали
  та `EvidenceRoot` і `WindowsDirectories` у маніфестах показують, де розташований кожен файл у
  доказах (`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`).
- Шляхи до результатів у маніфестах і журналах (`Output`, `OutputFiles`, `Log`, `JSON-Timeline`, шляхи
  виводу в `Command`) і `ExtractedFile` для Defender задаються відносно каталогу виводу
  (`./HOST1/ProcessExecution/Prefetch.csv`, `../siem.jsonl` для хронології поруч із ним). `Command`
  починається з імені програми (`juntei`); її шляхи до доказів задаються відносно каталогу
  доказів, а шляхи виводу — відносно каталогу виводу.
- `EvidenceDirectory` і `OutputDirectory` — це назви двох каталогів. `ManifestVersion` 2
  позначає маніфести, які фіксують шляхи до результатів відносно каталогу виводу; попередні маніфести не мають
  `ManifestVersion` і фіксують їх так, як їх передано в `-o`. Повторний запуск із `-C` читає обидва варіанти.
- Термінал показує шлях виводу або хронології в заданому вигляді, якщо він відносний, і відносно
  поточного каталогу, якщо він абсолютний (`../results/`).

Після кількості паралельних процесів термінал показує загальний розмір файлів
знайдених систем Windows (`Total filesize: 26.1 MiB`).

Повідомлення про хід виконання показує активний комп'ютер і артефакт, а потім приблизний розмір
вхідних даних, наприклад `HOST1 / prefetch (~2.0 MiB)`. Для групи файлів використовується їхній сукупний розмір; кожне
завдання MFT, LogFile або журналу USN показує розмір свого окремого вхідного файлу. Оцінки не враховують допоміжні журнали
транзакцій і файли доповнення та не прогнозують розмір виводу після аналізу.

`manifest.csv` надає той самий індекс завдань у вигляді таблиці, зручної для електронних таблиць, з одним рядком на
завдання парсера. Він містить `Comp`, початкове `Hostname`, `FQDN`, `HostnameSource`, `EvidenceRoot`,
`Artifact`, `Category`, `OutputType`, `Purpose`, `Status`, `DurationMs`, `ExitCode`, `Output`, `OutputFiles`, `Log`,
`Error` і `Command`. `Purpose` має значення `artifact` для категоризованих результатів або `json_timeline` для
додаткового нативного проходу JSONL, що використовується з CSV. `OutputType` — фактичний формат виводу парсера.
`Command` — це масив JSON із виконуваного файлу та його аргументів, що зберігає пробіли та лапки.
Для SRUM `Output` — це каталог комп'ютера, `OutputFiles` — масив JSON його точних
шляхів виводу провайдерів, а `Category` називає розташування спільного журналу (`SystemInformation`).
Файли категоризуються окремо; `OutputFiles` також присутнє в маніфесті JSON.

Відфільтруйте `Status` за `skipped` або `failed`, щоб знайти завдання, які потребують перегляду; `Error` пропущеного завдання
може бути порожнім, а пояснення міститься в його файлі `Log`.

Зведення в терміналі включає загальну кількість і розмір звичайних файлів, що наразі містяться в
каталозі виводу, включно з обома маніфестами, журналами парсерів і вилученими файлами. Символьні посилання
не відстежуються. Розміри автоматично масштабуються за допомогою того самого засобу форматування, що й `Total filesize`, з
одиницями на кшталт `KiB`, `MiB` і `GiB`:

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

Таблиці Amcache і SRUM використовують `-Source1`, `-Source2` тощо, якщо було зібрано кілька вихідних баз
даних. Спільний журнал парсера фіксує відповідність між оригінальними та кінцевими шляхами, а записи JSON/JSONL
зберігають відомості про походження джерела. Завдання CSV Amcache/Jump Lists перелічують точні шляхи своїх таблиць в
`OutputFiles` маніфесту; їхній `Output` — це каталог категорії розслідування. Експорти JSON і JSONL
Amcache і Jump Lists уже є окремими файлами з іменами `Registry-Amcache.json[l]` і
`JumpLists.json[l]`. Об'єднана хронологія `-j` зберігає той самий вміст записів і значення `Artifact`.
`--clobber` використовує попередній маніфест, щоб очистити зафіксовані виводи, включно з попередніми іменами,
зберігаючи при цьому не пов'язані з ними файли категорій. Звичайна колекція з одного тому записує записи `$MFT` і журналу USN безпосередньо в
`FileSystem/MFT.csv` і `FileSystem/UsnJournal.csv`. Додаткові зібрані томи використовують нумеровані
сусідні файли, як-от `FileSystem/MFT-2.csv` і `FileSystem/MFT-2-logs.txt`.

## Імпорт JSON і SIEM { #json-and-siem-ingestion }

Кожен запис JSON або JSONL отримує два поля верхнього рівня:

- `Comp`: вихідна система.
- `Artifact`: парсер Juntei, який створив запис, як-от `mft` або `prefetch-timeline`.

`-j, --json-timeline <FILE>` створює один додатковий файл JSONL незалежно від формату окремих
результатів. Усі категоризовані виводи та їхні журнали зберігаються:

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

Ім'я файлу використовується в наданому вигляді; відносні шляхи задаються відносно поточного робочого каталогу.
Файл може бути в корені виводу, в іншому незарезервованому підкаталозі виводу або поза каталогом
виводу. Докази, маніфести, результати/журнали окремих комп'ютерів і тимчасові каталоги парсерів не можна
використовувати як місце призначення. Наявний файл вимагає `-C`; невдалий експорт залишає його незмінним.

У **режимі CSV** Juntei виконує додатковий нативний прохід JSONL, щоб файл SIEM зберігав
числові та логічні типи, вкладені дані, сирі поля та метадані, які таблиці CSV можуть пропускати.
Додаткові завдання підпадають під те саме обмеження паралельності `-T`, не вилучають вміст карантину повторно і
мають окремі супровідні журнали `*-json-timeline-logs.txt`. Обидва маніфести розрізняють їхнє призначення та
формат виводу. Тимчасові файли JSONL видаляються після успішного запуску; вони зберігаються, якщо
парсер або експорт завершився збоєм. Заплануйте додатковий час обробки та тимчасовий дисковий простір.

У **режимі JSON** Juntei потоково перетворює наявні результати на компактний JSONL без повторного аналізу
доказів. Звіти Amcache розгортаються в ті самі окремі записи зведення, знахідок, файлів і пристроїв,
що й у нативному JSONL. У **режимі JSONL** наявні анотовані записи об'єднуються безпосередньо.
Усі режими використовують ту саму структуру записів SIEM. Набір даних зберігає нативні позначки часу та поля
артефактів; він не вигадує спільної позначки часу події і не сортує глобально різнорідні артефакти.

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

`manifest.json` фіксує запитаний шлях `JSON-Timeline` і будь-яку `JSON-TimelineError`. Його записи
завдань і `manifest.csv` і надалі вказують на збережені виводи артефактів. Успішні завдання CSV, призначені лише для хронології,
посилаються на кінцевий файл JSONL замість видалених тимчасових файлів. Якщо будь-який парсер завершується збоєм,
запуск завершується неуспішно, а хронологія містить лише успішні результати; перегляньте маніфести
та журнали категорій, перш ніж вважати її повною. Зведення кількості/розміру файлів охоплює файли всередині
каталогу виводу; про зовнішню хронологію повідомляється окремо як `JSONL timeline: <FILE>`.

## Продуктивність { #performance }

Завдання парсерів — це окремі процеси, тому артефакти, що інтенсивно навантажують CPU, можуть виконуватися одночасно, а пошкодження в
одному парсері не може перервати весь запуск. За замовчуванням Juntei використовує кількість логічних CPU. Обмежте
паралельність, якщо докази розташовано на повільному диску або система має мало пам'яті:

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## Використання { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]
      --vhdx                  Parse the files inside every VHDX disk image in the evidence directory, each mounted read-only with FUSE while it is parsed

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -Z, --timezone <OFFSET>             Display parsed timestamps using a fixed UTC offset (UTC by default) [default: +00:00]
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK files, Jump Lists, ShellBags and OpenSaveMRU [default: 1252]
      --mft-slack           Recover the file names left in the slack of $MFT records

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

За можливості використовуйте новий каталог виводу. `-C` видаляє або перезаписує лише точні згенеровані
цілі цього запуску, тому артефакт, якого тепер немає, не може залишити застарілий результат. Він не видаляє
не пов'язані файли чи все дерево виводу. Перед аналізом він також видаляє каталог
`SecurityProducts/DefenderQuarantined/` кожного комп'ютера та зарезервований каталог `defender-quarantined/`
з попередньої пласкої структури, незалежно від того, чи використовує новий запуск `-x`.

Під час переходу зі старішої пласкої структури використовуйте новий каталог виводу. Попередні файли CSV/JSON
автоматично не переміщуються і не видаляються.

!!! danger "Вважайте вилучений вміст Defender шкідливим програмним забезпеченням"
    `-x, --extract-defender-quarantined` записує вміст із карантину на диск у каталог
    `SecurityProducts/DefenderQuarantined/` кожного комп'ютера. Відновлені двійкові потоки мають закінчення `.quarantined`, а Juntei
    знімає дозвіл на виконання. Для кожного двійкового дескриптора безпеки також створюється зручний для читання звіт `.security_descriptor.txt`
    із даними про власника, дозволи та SDDL. Ніколи не запускайте відновлене корисне навантаження. Використовуйте ізольовану
    систему для аналізу і будьте готові до того, що антивірус або EDR сповіщатимуть про відновлений вміст або видалятимуть його.

Див. [читання відновленого дескриптора безпеки](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor),
щоб дізнатися про поля звіту та їхні криміналістичні обмеження.

Auto-parse також читає віддалені підключення користувачів (RDP, PuTTY, WinSCP і підключені диски).
