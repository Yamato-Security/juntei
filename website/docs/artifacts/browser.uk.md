# Веб-браузери { #web-browsers }

## Що таке бази даних браузерів { #what-the-browser-databases-are }

Браузери зберігають історію, завантаження, файли cookie та записи форм у базах даних SQLite у профілі
кожного користувача. Браузери на основі Chromium (Chrome, Edge, Brave, Opera, Vivaldi) і браузери на
основі Firefox використовують кожен свої файли й таблиці, але фіксують однакові види активності:

- **Історія**: відвідані сторінки; кожне відвідування — з часом, зі способом, у який браузер потрапив на
  сторінку (введення адреси, посилання, переспрямування, закладка), а в Chromium — і з тим, як довго
  сторінка була на передньому плані.
- **Завантаження**: URL, з якого надійшов файл (і переспрямування на шляху), куди його збережено, його
  розмір, коли завантаження почалося й закінчилося і чи було його завершено.
- **Файли cookie**: сайти, що їх установили, коли їх установлено, востаннє надіслано і коли спливає їхній
  термін дії.
- **Історія форм**: значення, введені у вебформи та поля пошуку, скільки разів, уперше й востаннє.

Активність у браузері часто є найпрямішим записом того, що робив користувач: фішингова сторінка, за якою
він перейшов, інструмент, який він завантажив, вебпошта, якою він користувався, пошукові запити, які він
вводив.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> профіль браузера зафіксував
відвідування URL у записаний час або зберіг завантаження за записаним шляхом; база даних не каже, хто
сидів за клавіатурою.

## Розташування та збирання { #location-and-collection }

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

`<Profile>` — це `Default`, `Profile 1`, `Profile 2` і так далі. У версії 96 Chromium переніс `Cookies`
до `Network`; старіші версії тримають цей файл поруч з `History`. Firefox до версії 25 зберігав
завантаження в `downloads.sqlite`; пізніші версії зберігають їх у `places.sqlite`.

Збирайте всі файли профілю, а кожну базу даних — разом із файлами поруч, що мають ту саму назву:
`places.sqlite-wal`, `cookies.sqlite-wal`, `formhistory.sqlite-wal`, `History-journal`.

!!! warning "Журнали випереджального запису"
    Firefox тримає свої бази даних у режимі журналу випереджального запису (write-ahead log): найновіші
    зміни є лише у файлі `-wal`, доки браузер не перенесе їх до бази даних (контрольна точка, checkpoint).
    Juntei читає кожну базу даних разом з її файлом `-wal`, як це зробив би браузер. Без нього бракує
    недавніх відвідувань, файлів cookie та записів форм, а записи, видалені в журналі, і далі читаються
    як наявні.

## Видалені записи { #deleted-entries }

Коли браузер видаляє історію, рядки зникають із таблиць, але їхні записи можуть залишатися у вільному
просторі бази даних, у її списку вільних сторінок (freelist) або в старіших версіях сторінок бази даних:
у файлі `-wal` або в самому файлі бази даних, якщо новіші версії цих сторінок містить `-wal`. Juntei
відновлює з них видалені відвідування, сторінки, завантаження та записи форм як рядки, у яких `Status`
має значення `Deleted`:

- `Confidence` показує, наскільки впевнено запис зіставлено з його таблицею: `High` для цілого запису,
  кожне значення якого має тип свого стовпця, `Medium` або `Low` для пошкоджених записів чи записів, що
  підходять до кількох таблиць.
- `RecoveredFrom` показує, де був запис: область сторінки (комірка, вільний блок (freeblock),
  нерозподілений простір, вільна сторінка), яка сторінка і яка її версія, а також зміщення запису на
  сторінці. Кадри WAL нумеруються від 0.
- `LostColumns` перелічує стовпці, яких пошкоджений запис уже не містить: їхні поля порожні.

Браузери видаляють дані з параметром SQLite `secure_delete`: звільнений простір у базі даних
заповнюється нулями. Тому те, що можна відновити, здебільшого міститься в старіших версіях сторінок: у
тих, які ще зберігає файл `-wal`, і у власних копіях файлу бази даних для сторінок, замінених через
`-wal`, — до наступної контрольної точки. Копіюйте файл `-wal` разом із його базою даних і робіть це до
наступного запуску браузера.

## Застереження для аналітика { #analyst-cautions }

- Час подано в UTC. Chromium і Firefox записують його з точністю до мікросекунди; історію форм Chromium —
  з точністю до секунди.
- Відвідування не означає, що користувач щось увів або клацнув: переспрямування, фрейми та сторінки,
  відкриті іншими програмами, — це теж відвідування. Перевіряйте `Transition`, `Typed` і `Hidden`.
- Chromium шифрує значення файлів cookie: для таких файлів `Value` порожнє. Juntei не розшифровує їх.
- Рядок історії форм зі статусом `Removed` походить із таблиці Firefox `moz_deleted_formhistory`, яку
  Firefox для Android заповнює для Sync. Настільний Firefox зазвичай залишає її порожньою.
- Видалений запис із довірою `Low` може бути фрагментом іншого запису: звіряйте його з наявними рядками
  та іншими доказами.
- Відсутність видалених рядків не означає, що нічого не видаляли: обнулений простір і контрольна точка не
  залишають нічого для відновлення. Видалені файли cookie поки що не відновлюються.
- Вміст бази визначає сімейство браузера. Сумісний шлях профілю уточнює `Browser` до Chrome чи Firefox; суперечливий шлях не змінює сімейство. Інакше це `Chromium-based`, `Firefox-based` або `Safari` (`Unknown`, якщо не визначено). `User` і `Profile` беруться зі шляху та можуть бути втрачені під час копіювання.
- Журнали відкату (`-journal`) не читаються, як і `WebCacheV01.dat` Internet Explorer і застарілого Edge.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

`-d` шукає файли за назвою і розпізнає кожен за його таблицями: файл `History` іншої програми
пропускається. `-f` читає базу даних незалежно від її назви.

## Таблиці виводу { #output-tables }

CSV створює чотири файли; файл без рядків містить лише заголовок. Рядки JSON і JSONL містять `Table`
(`History`, `Downloads`, `Cookies` або `FormHistory`) після `SourceFile`.

| Файл CSV | Вміст |
|---|---|
| `Browser-History.csv` | Відвідування, наявні та видалені; видалені сторінки, URL яких немає в жодної наявної сторінки. |
| `Browser-Downloads.csv` | Завантаження, наявні та видалені. |
| `Browser-Cookies.csv` | Файли cookie. |
| `Browser-FormHistory.csv` | Записи форм, наявні та видалені; записи Firefox про вилучення. |

## Поля { #fields }

Кожна таблиця починається з таких полів:

| Поле | Інтерпретація |
|---|---|
| `SourceFile` | База даних, з якої прочитано рядок. |
| `User` | Користувач Windows, у профілі якого лежить шлях (папка після `Users`). |
| `Browser` | Сімейство за вмістом, уточнене сумісним шляхом профілю. |
| `Profile` | Профіль браузера: папка бази даних. |
| `Status` | `Live`, `Deleted` (відновлений) або `Removed` (запис Firefox про вилучення). |

### Історія { #history }

| Поле | Інтерпретація |
|---|---|
| `RecordType` | `Visit` або `Page` для видаленої сторінки, відвідування якої теж могли зникнути. |
| `VisitTime` | Коли почалося відвідування (UTC). |
| `URL`, `Title` | Сторінка та її заголовок у тому вигляді, у якому їх бачили востаннє. |
| `Transition` | Як браузер потрапив на сторінку. Chromium: основний тип, а за ним його кваліфікатори (`LINK`, `TYPED`, `AUTO_BOOKMARK`, `FORM_SUBMIT`, `RELOAD`, … з `CHAIN_START`, `SERVER_REDIRECT`, `FROM_ADDRESS_BAR`, …); Firefox: `LINK`, `TYPED`, `BOOKMARK`, `EMBED`, `REDIRECT_PERMANENT`, `REDIRECT_TEMPORARY`, `DOWNLOAD`, `FRAMED_LINK`, `RELOAD`. |
| `VisitDuration` | Скільки секунд сторінка була на передньому плані (Chromium; 0, якщо не вимірювалося). |
| `VisitCount` | Загальна кількість відвідувань сторінки. |
| `Typed`, `TypedCount` | Чи URL коли-небудь вводили вручну і скільки разів (Chromium). |
| `Hidden` | Приховано зі списку історії: вкладені фрейми, джерела переспрямувань, помилки. |
| `Frecency` | Рейтинг сторінки у Firefox для підказок в адресному рядку. |
| `VisitID`, `FromVisitID` | Ідентифікатор рядка відвідування та ідентифікатор відвідування, з якого на нього перейшли (сторінка-реферер, джерело переспрямування). Ідентифікатор відвідування порожній, якщо відновлений запис його втратив. |
| `LastVisitTime` | Останнє відвідування видаленої сторінки (UTC). |
| `PageSource` | Звідки взято URL і заголовок видаленого відвідування: `Live` (наявна сторінка), `Recovered` (видалена сторінка) або `NotFound`. |
| `Confidence`, `RecoveredFrom`, `LostColumns` | Як відновлено видалений запис; див. [Видалені записи](#deleted-entries). |

### Завантаження { #downloads }

| Поле | Інтерпретація |
|---|---|
| `StartTime`, `EndTime` | Коли завантаження почалося й закінчилося (UTC). |
| `URL`, `RedirectChain` | Звідки надійшов файл і кожен URL на шляху (Chromium; у CSV між ними ` -> `). |
| `TargetPath`, `CurrentPath` | Куди збережено файл (у Firefox — URI `file://`) і куди його записували під час завантаження. |
| `ReceivedBytes`, `TotalBytes` | Отримані байти та розмір файлу. |
| `State` | `Complete`, `InProgress`, `Cancelled`, `Interrupted`, `Paused`, `Blocked` або `Dirty` (заблоковано як зловмисне ПЗ). |
| `DangerType` | Вердикт Chromium щодо файлу: 0 не небезпечний, 1 небезпечний тип файлу, 2 небезпечний URL, 3 небезпечний вміст, 4 можливо небезпечний, 5 рідкісний, 6 залишений користувачем, 7 небезпечний хост, 8 потенційно небажаний, … |
| `InterruptReason` | Чому перерване завантаження зупинилося (Chromium; 0, якщо переривання не було). |
| `Referrer`, `TabURL` | Сторінка-реферер і сторінка у вкладці, з якої почалося завантаження. |
| `MimeType`, `Opened` | Тип файлу, як його віддав сервер, і чи відкривали файл із браузера (Chromium). |
| `FileDeletedByUser` | Чи видалив користувач завантажений файл із диска через список завантажень браузера (Firefox). |
| `DownloadID` | Ідентифікатор рядка завантаження. Порожній, якщо відновлений запис його втратив. |

### Файли cookie { #cookies }

| Поле | Інтерпретація |
|---|---|
| `Created`, `LastAccessed`, `Expires` | Коли файл cookie було встановлено, востаннє надіслано і коли спливає його термін дії (UTC; для сеансового файлу cookie `Expires` порожнє). |
| `Host`, `Name`, `Path` | Сайт, для якого призначено файл cookie, його ім'я та шлях. |
| `Value` | Значення в збереженому вигляді: порожнє, якщо Chromium його зашифрував. |
| `Secure`, `HttpOnly`, `Persistent` | Надсилається лише через HTTPS, прихований від скриптів, зберігається між сеансами (Chromium). |
| `GoogleAnalytics` | Декодовані поля файлу cookie Google Analytics (`__utma`, `__utmb`, `__utmt`, `__utmz`): час відвідувань, сеанси, джерело кампанії та пошукові запити. |
| `RowID` | Ідентифікатор рядка файлу cookie. |

### Історія форм { #form-history }

| Поле | Інтерпретація |
|---|---|
| `FirstUsed`, `LastUsed` | Коли значення ввели вперше та востаннє (UTC). |
| `RemovedTime` | Коли, згідно із записом Firefox про вилучення, запис було вилучено (UTC). |
| `FieldName`, `Value` | Поле форми (`email`, `q`, рядок пошуку Firefox: `searchbar-history`) і введене значення. |
| `Count` | Скільки разів значення вводили. |
| `GUID` | Ідентифікатор запису у Firefox, на який посилаються записи про вилучення. |
| `Sources` | Звідки Firefox зберіг запис, у версіях, які це зберігають (у CSV значення розділено вертикальною рискою). |
| `RowID` | Ідентифікатор рядка; порожньо/null, якщо його втрачено під час відновлення. Фактично записаний нуль залишається `0`. |

## Автоматичний аналіз { #automatic-parsing }

`auto-parse` записує таблиці до `BrowserActivity` як `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv` і `Browser-FormHistory.csv` (один файл `Browser.json` або `Browser.jsonl` з
`-t json` чи `-t jsonl`) разом із `BrowserActivity/Browser-logs.txt`. Об'єднана хронологія `-j` містить
кожен рядок, із `Comp` та ідентифікатором `Artifact` `browser`. `-Z` перетворює `VisitTime`,
`LastVisitTime`, `StartTime`, `EndTime`, `Created`, `LastAccessed`, `Expires`, `FirstUsed`, `LastUsed` і
`RemovedTime`.

## Додаткові відомості { #further-details }

- [Chromium — page transition types](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — download danger types](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — Places database](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — write-ahead logging](https://www.sqlite.org/wal.html)
- [Plaso — browser SQLite parsers](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
