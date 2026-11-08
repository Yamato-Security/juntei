# Internet Explorer і застарілий Edge { #internet-explorer-and-legacy-edge }

## Що таке база даних WebCache { #what-the-webcache-database-is }

`WebCacheV01.dat` — це база даних інтернет-функцій Windows (WinINet) у профілі кожного користувача,
база даних ESE. Internet Explorer 10 і 11 зберігають у ній свою історію, завантаження, кеш і файли
cookie, як і застарілий Edge — Edge з Windows 10 до того, як Edge став браузером на основі Chromium. Кожен вид
записів — це контейнер, окрема таблиця:

- **Історія** (`History`, а також `MSHist01…` для дня чи тижня): відвідані URL, кожен із заголовком
  сторінки, обліковим записом, для якого його записано, часом останнього відвідування та кількістю відвідувань. Провідник
  файлів теж записує туди локальні файли, які він відкриває, як URL `file:///`.
- **Завантаження** (`iedownload`): URL, з якого надійшов файл, куди його збережено і куди записувано під
  час завантаження, його розмір, коли почалося завантаження, сторінка-реферер і тип MIME.
- **Файли cookie**: у Windows 10 і новіших — самі файли cookie (таблиці `CookieEntryEx`) з їхнім хостом,
  ім'ям, значенням і часом; раніше — файли, у яких вони зберігалися.
- **Кеш** (`Content`): файли, завантажені для показу сторінок, кожен зі своїм URL, розташуванням копії на
  диску та заголовками відповіді HTTP. Інші програми, що використовують WinINet, теж кешують тут свої
  завантаження.
- Вебсховище (`DOMStore`), списки сумісності (`iecompat`) та інше.

База даних переживає браузери: Internet Explorer виведено з експлуатації, а Edge на основі Chromium веде
власні бази даних SQLite (див. [Веб-браузери](browser.md)), але ця база даних і далі є в кожному профілі —
з тим, що було записано раніше, і з тим, що досі записують Провідник файлів та інші програми.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> профіль користувача зафіксував URL,
завантаження або кешований файл у записаний час; база даних не каже, хто сидів за клавіатурою, і не
свідчить, що кешований файл відкривали.

## Розташування та збирання { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

Файли, на які вказують контейнери (кешовані файли, файли cookie), лежать у власних папках:
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\`, `INetCookies\`, `History\`, а для застосунків — у
`%LOCALAPPDATA%\Packages\`.

Збирайте всю папку `WebCache`. Windows тримає базу даних відкритою, поки користувач у системі, тож копія
з працюючої системи зазвичай «брудна»: її найновіші зміни є лише в журналах транзакцій `V01*.log`.

!!! warning "Журнали транзакцій"
    Juntei не відтворює журнали транзакцій ESE. База даних, яку не було коректно закрито, у підсумку
    рахується як залишена «брудною», і про неї повідомляється як про проблему: зміни, що досі є лише в її
    журналах, відсутні. Щоб їх урахувати, спершу відтворіть журнали в копії папки за допомогою команди Windows
    `esentutl /r V01 /d`, запущеної в цій копії.

## Застереження для аналітика { #analyst-cautions }

- Час подано в UTC, з точністю до 100 наносекунд, крім `ModifiedTime` контейнера історії дня чи тижня:
  це місцевий час системи, записаний без зміщення, і `-Z` залишає його без змін. Перетворюйте його за
  часовим поясом системи-джерела на той момент. Його різниця з `AccessedTime` часто дорівнює зміщенню цього
  часового поясу, але не завжди: це можуть бути різні моменти.
- Запис історії — це URL, а не відвідування: `AccessedTime` — його останнє відвідування, а `AccessCount` —
  скільки їх було. Контейнер `History` зберігає один запис на URL; контейнер дня чи тижня зберігає записи
  цього періоду, а його назва містить перший день періоду й день після останнього
  (`MSHist012026110120261102`: 1 листопада 2026 року).
- URL `file:///` в історії — це локальні файли, відкриті з Провідника файлів та інших програм, а не
  вебсторінки.
- `Account` — обліковий запис, для якого зроблено запис (`Visited: alice@…`); `User` — папка профілю, у
  якій лежить шлях бази даних. Вони можуть відрізнятися: папку профілю не завжди названо за її обліковим
  записом, а в шляху бази даних, скопійованої в інше місце, папки профілю немає.
- `Package` показує, чий це запис: застарілого Edge (`microsoft.microsoftedge_8wekyb3d8bbwe`), Internet
  Explorer у розширеному захищеному режимі (`windows_ie_ac_001`) чи іншого застосунку. Якщо поле порожнє,
  запис належить самому користувачеві: Internet Explorer, Провіднику файлів та іншим програмам.
- Microsoft не документує запис завантаження. Juntei читає його час початку й розмір лише тоді, коли його
  заголовок уміщується в межах запису, а шляхи та інші рядки — лише тоді, коли два останні з них — це URL і шлях Windows.
  Інакше рядок містить власний URL запису, а коли час початку прочитати не вдається, — і його час доступу,
  разом із проблемою. `Publisher` — це ім'я, яке записав браузер, а не перевірка підпису файлу.
- Запис кешу свідчить, що файл було завантажено й кешовано. Копії на диску вже може не бути, її могли
  замінити або ніколи не відкривати.
- Видалені записи поки що не відновлюються: ESE може зберігати видалений запис на своїх сторінках, доки
  не очистить їх, але Juntei читає лише наявні записи.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

`-d` шукає `WebCacheV01.dat` за назвою; `-f` читає базу даних незалежно від її назви.

## Таблиці виводу { #output-tables }

CSV створює п'ять файлів; файл без рядків містить лише заголовок. Рядки JSON і JSONL містять `Table`
(`History`, `Downloads`, `Cookies`, `Entries` або `Containers`) після `SourceFile`.

| Файл CSV | Вміст |
|---|---|
| `WebCache-History.csv` | Записи контейнерів історії. |
| `WebCache-Downloads.csv` | Завантаження. |
| `WebCache-Cookies.csv` | Файли cookie (Windows 10 і новіші). |
| `WebCache-Entries.csv` | Записи всіх інших контейнерів: кешовані файли, файли cookie, вебсховище, списки сумісності та решта. |
| `WebCache-Containers.csv` | Контейнери. |

## Поля { #fields }

Кожна таблиця починається з таких полів:

| Поле | Інтерпретація |
|---|---|
| `SourceFile` | База даних, з якої прочитано рядок. |
| `User` | Користувач Windows, у профілі якого лежить шлях (папка після `Users`). |
| `Package` | Застосунок, чиї дані містить рядок: папка в `Packages` у каталозі його контейнера (або його таблиці файлів cookie). Порожнє для власних даних користувача. |

### Історія { #history }

| Поле | Інтерпретація |
|---|---|
| `Container` | Контейнер: `History` або `MSHist01`, після якого йдуть перший день його періоду й день після останнього. |
| `AccessedTime` | Останнє відвідування URL (UTC); у контейнері дня чи тижня — останнє в цьому періоді. |
| `ModifiedTime` | Коли запис востаннє змінено (UTC); у контейнері дня чи тижня — місцевий час системи, без зміщення. |
| `CreationTime` | Коли створено запис (UTC). |
| `ExpiryTime`, `SyncTime` | Коли запис має бути вилучено з історії і коли його востаннє синхронізовано (UTC). |
| `Account` | Обліковий запис, для якого зроблено запис. |
| `URL` | Сторінка або файл без префікса `Visited: account@`. |
| `Title` | Заголовок сторінки в тому вигляді, у якому його зберіг запис; порожній у контейнері дня чи тижня. |
| `AccessCount`, `SyncCount` | Скільки разів відвідували URL і скільки разів синхронізували запис. |
| `EntryType`, `Flags` | Тип і прапорці запису в збереженому вигляді. |
| `EntryID`, `ContainerID` | Ідентифікатор запису в його контейнері та ідентифікатор контейнера. |

### Завантаження { #downloads }

| Поле | Інтерпретація |
|---|---|
| `StartTime` | Коли почалося завантаження (UTC). |
| `URL` | Звідки надійшов файл. |
| `TargetPath`, `CurrentPath` | Куди збережено файл і куди його записували під час завантаження: файл `.partial` для застарілого Edge, копія в кеші для Internet Explorer. |
| `TotalBytes` | Розмір файлу. |
| `Referrer` | Сторінка-реферер. |
| `MimeType` | Тип файлу, як його віддав сервер (Internet Explorer). |
| `Publisher` | Ім'я видавця файлу, яке записав застарілий Edge. |
| `AccessedTime` | Власний час доступу запису (UTC), найчастіше — час початку. |
| `DownloadID`, `ContainerID` | Ідентифікатор запису в його контейнері та ідентифікатор контейнера. |

### Файли cookie { #cookies }

| Поле | Інтерпретація |
|---|---|
| `LastModified`, `Expires` | Коли файл cookie було встановлено і коли спливає його термін дії (UTC). |
| `Host`, `Name`, `Path` | Сайт, для якого призначено файл cookie, його ім'я та шлях. |
| `Value` | Значення в збереженому вигляді. |
| `Flags` | Прапорці файлу cookie в збереженому вигляді. |
| `Directory` | Каталог, який база даних вказує для таблиці файлів cookie: чиї це файли cookie (`…\INetCookies\ESE\`, `…\MicrosoftEdge\Cookies\ESE\` застарілого Edge, каталог застосунку). |
| `CookieTable`, `EntryID` | Таблиця (`CookieEntryEx_<n>`) та ідентифікатор файлу cookie в ній. |

### Записи { #entries }

| Поле | Інтерпретація |
|---|---|
| `Container` | Контейнер: `Content` (кешовані файли), `Cookies` (файли cookie), `DOMStore` (вебсховище), `iecompat`, …. |
| `AccessedTime` | Коли запис востаннє використовувався (UTC). |
| `ModifiedTime` | Остання зміна ресурсу, як її повідомив сервер (UTC). |
| `CreationTime` | Коли створено запис (UTC). |
| `ExpiryTime`, `SyncTime` | Коли спливає термін дії запису і коли його востаннє звіряли із сервером (UTC). |
| `URL` | URL у збереженому вигляді: URL для кешованого файлу, `Cookie:alice@example.com/` для файлу cookie, `DOMStore:…`, `iecompat:…`. |
| `Account` | Обліковий запис файлу cookie. |
| `Filename`, `FilePath` | Файл запису і його розташування: каталог контейнера та вказана записом підпапка. |
| `FileSize` | Розмір файлу. |
| `AccessCount`, `SyncCount` | Скільки разів запис використовувався і скільки разів його звіряли із сервером. |
| `EntryType`, `Flags` | Тип і прапорці запису в збереженому вигляді. |
| `RedirectURL` | Куди переспрямовував URL. |
| `ResponseHeaders`, `RequestHeaders` | Заголовки відповіді HTTP (`HTTP/1.1 200 OK`, `Content-Type`, `Server`, …) і заголовки запиту в збереженому вигляді, з усіма їхніми рядками. |
| `EntryID`, `ContainerID` | Ідентифікатор запису в його контейнері та ідентифікатор контейнера. |

### Контейнери { #containers }

| Поле | Інтерпретація |
|---|---|
| `ContainerID`, `Name` | Ідентифікатор і назва контейнера. |
| `Directory` | Де на диску лежать його файли. |
| `SecureDirectories` | Підпапки `Directory`, у яких лежать його файли (у CSV розділені комою). |
| `PartitionID` | `M` для власних контейнерів користувача, `L` для контейнерів захищеного режиму Internet Explorer (папки `Low`), SID контейнера застосунку для контейнерів застосунку. |
| `LastAccessTime`, `LastScavengeTime` | Коли контейнер востаннє використовувався і коли його востаннє очищали (UTC). |

## Автоматичний аналіз { #automatic-parsing }

`auto-parse` записує таблиці до `BrowserActivity` як `WebCache-History.csv`,
`WebCache-Downloads.csv`, `WebCache-Cookies.csv`, `WebCache-Entries.csv` і
`WebCache-Containers.csv` (один файл `WebCache.json` або `WebCache.jsonl` з `-t json` чи
`-t jsonl`) разом із `BrowserActivity/WebCache-logs.txt`. Об'єднана хронологія `-j` містить кожен
рядок, із `Comp` та ідентифікатором `Artifact` `webcache`. `-Z` перетворює кожен час (поля, назви яких
закінчуються на `Time`, а також `LastModified` і `Expires`), крім місцевого `ModifiedTime` контейнера
історії дня чи тижня.

## Додаткові відомості { #further-details }

- [Microsoft — About WinINet](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE database file format](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — MSIE WebCache parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
