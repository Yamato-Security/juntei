# Індекс Windows Search { #windows-search-index }

## Що записує індекс { #what-the-index-records }

Windows Search (служба `WSearch`) обходить профілі користувачів, меню «Пуск», поштові скриньки Outlook,
історію Internet Explorer, автономні файли та все інше, що входить до його області, і зберігає прочитане
у `Windows.edb` — базі даних ESE. Для розслідування мають значення дві її частини:

- **сховище властивостей** (`SystemIndex_PropertyStore`; `SystemIndex_0A` у Windows 7) з одним рядком
  на кожен проіндексований елемент: його шлях, назва, тип і розмір, власник, часи створення, зміни та
  доступу, які індексатор скопіював із файлу, час, коли індексатор його прочитав, назва й автор
  документа, початок його тексту, ім'я комп'ютера та сотні інших властивостей;
- таблиці **збирача** (`SystemIndex_Gthr`, `SystemIndex_GthrPth`) із власним записом обхідника
  (crawler) про кожен елемент: його URL-шлях, час зміни на момент постановки в чергу, номер обходу та
  прапорці, а також лічильник видалень.

Елемент залишається в індексі після зникнення файлу, повідомлення чи сторінки, доки обхідник не помітить
видалення. Тому індекс називає файли, яких уже немає, з їхніми метаданими та витягом тексту такими,
якими вони були на момент індексування.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> індексатор прочитав елемент за
записаним шляхом, із записаними метаданими, о `GatherTime`, на названому комп'ютері. Індекс показує, що
файл існував і що він містив. Він не показує, що користувач його відкривав.

## Розташування та збирання { #location-and-collection }

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

Від Windows 7 до Windows 11 21H2, а також випуски Windows Server тих самих років, зберігають індекс у
`Windows.edb`. Windows 11 22H2 і новіші зберігають його в базах даних SQLite у тій самій папці
(`Windows.db`, `Windows-gather.db`, `Windows-usn.db`); Juntei знаходить і називає їх та повідомляє про
них як про непідтримуваний новіший формат.

Збирайте всю папку `Applications\Windows` з образу диска, тіньової копії тому або після зупинки служби
`WSearch`: поки служба працює, база даних відкрита, а її найновіші зміни є лише в журналах транзакцій
`MSS*.log`, які Juntei не відтворює. Індекс, скопійований із працюючої системи, залишається «брудним»;
вивід повідомляє про це й читає те, що є на диску. Вхідний каталог і `auto-parse` читають кожен
`Windows.edb` на будь-якій глибині; для одного файлу використовуйте `-f`.

## Застереження для аналітика { #analyst-cautions }

- Аналізуйте незмінну копію доказів. Juntei відображає файл у пам’ять для ефективного доступу; не змінюйте й не обрізайте базу даних під час читання.
- `GatherTime` — це час, коли індексатор прочитав елемент, а не коли користувач його створив, змінив
  або відкрив. Індексування виконується у фоновому режимі, часто через хвилини чи години після зміни, і
  призупиняється, поки комп'ютер зайнятий або працює від акумулятора.
- `DateCreated`, `DateModified` і `DateAccessed` — це копії, які індексатор узяв із файлу. Вони
  оновлюються, коли елемент індексується знову, тож описують файл станом на `GatherTime`.
- `ItemPath` — це шлях для відображення. `ItemURL` і `GatherPath` — URL-адреси обхідника: `file:` для
  файлів (із томом у вигляді `[GUID]` у Windows 8 і новіших), `iehistory://{SID}/` для історії Internet
  Explorer, `mapi://` для елементів Outlook, `csc://{SID}/` для автономних файлів і `winrt://{SID}/`
  для вмісту програм Windows.
- `User` — це профіль, названий у шляху елемента (`C:\Users\<name>\…`). Це те, де був елемент, а не
  хто його створив; `FileOwner` — власник файлу таким, яким його бачив індексатор.
- `AutoSummary` — це початок тексту елемента. Він може цитувати документ, який відтоді було видалено
  або зашифровано, і це обрізаний витяг, а не сам документ.
- «Брудна» база даних може містити сторінки, до яких її сторінки гілок ще не ведуть. Juntei іде за
  власним ланцюжком листових сторінок і зазначає в підсумку, скільки їх прочитано в такий спосіб. Її
  найновіші зміни все одно лишаються в журналах транзакцій.
- Елемент у таблицях збирача без рядка у сховищі властивостей (`InTables` дорівнює `Gthr`) було
  поставлено в чергу або пройдено обходом, а його властивості ще не записано або вже вилучено. Його
  шлях, назва та часи збирача все ще на місці.
- Windows 7 зберігає рядки у власному стисненому, обфускованому кодуванні. Juntei декодує його; значення,
  яке не декодується, залишається шістнадцятковим, із приміткою в `Problems`.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

Типовий формат файлу — CSV. З `-t csv`, `-t json` або `-t jsonl` без `-o` записи надходять у stdout.
Без обох параметрів відображається лише підсумок. Додайте `-C`, щоб перезаписати наявний результат.
Вивід ніколи не записується поверх одного з вхідних файлів, навіть із `-C`.

Juntei створює один запис на кожен проіндексований елемент, об'єднуючи сховище властивостей і таблиці
збирача за ідентифікатором елемента. Підсумок підраховує бази даних за схемою, «брудні» бази, елементи в
обох таблицях, лише у збирача та лише у сховищі властивостей, папки, елементи, позначені як видалені,
елементи за кожним сховищем, елементи без збереженого часу, а також проблеми.

Пошкодження не зупиняє роботу. Обрізана база даних зберігає елементи, які містять її вцілілі сторінки,
а підсумок повідомляє, чого не вдалося прочитати. Про файл із назвою `Windows.edb`, який не є індексом
Windows Search, повідомляється, і його пропущено. Інші бази даних все одно читаються.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `DocumentID` | Ідентифікатор елемента в індексі: `WorkID` сховища властивостей (`DocID` у Windows 7) і `DocumentID` рядка збирача. |
| `InTables` | `Gthr+PropertyStore`, `Gthr` (елемент у черзі або пройдений обходом, без властивостей) або `PropertyStore`. |
| `ItemPath` | `System_ItemPathDisplay`: шлях так, як його показує Windows. Порожнє, якщо у сховищі властивостей немає рядка. |
| `ItemURL` | `System_ItemUrl`: URL-адреса елемента в обхідника (`file:`, `iehistory://`, `mapi://`, `csc://`, `winrt://`). |
| `GatherPath` | Шлях збирача, складений із дерева областей і назви файлу з рядка. |
| `Name` | `System_ItemName` або назва файлу з рядка збирача. |
| `ItemType`, `ItemTypeText` | `System_ItemType` (розширення або `Directory`) та його відображувана назва. |
| `Kind` | `System_Kind`: `document`, `folder`, `link`, `program`, `email`, `picture`, …; кілька значень об'єднано через `\|`. |
| `MimeType` | `System_MIMEType`. |
| `Store` | `System_Search_Store`: `file`, `mapi` (Outlook), `iehistory`, `csc` (автономні файли) або `winrt` (програми Windows). |
| `IsFolder`, `IsDeleted` | `System_IsFolder` і `System_IsDeleted`, як збережено. |
| `DeletedCount` | Лічильник видалень із рядка збирача. |
| `Size` | `System_Size` у байтах. Порожнє для папок, які містять заповнювач. |
| `FileAttributes` | `System_FileAttributes` у вигляді числа (16 — каталог, 32 — архівний, 2 — прихований, …). |
| `FileOwner` | `System_FileOwner` (`DOMAIN\user`) таким, яким його прочитав індексатор. |
| `DateCreated`, `DateModified`, `DateAccessed` | Часи файлу так, як їх скопіював індексатор (UTC). |
| `ItemDate` | `System_ItemDate`: власна дата елемента, як-от час надсилання повідомлення або дата зйомки фотографії; для файлу зазвичай його час зміни з точністю до секунди. |
| `GatherTime` | `System_Search_GatherTime`: коли індексатор прочитав елемент (UTC). |
| `GatherLastModified` | `LastModified` рядка збирача: час зміни елемента на момент, коли обхідник поставив його в чергу (UTC). |
| `Title`, `Author` | `System_Title` і `System_Author`. |
| `AutoSummary` | `System_Search_AutoSummary`: початок тексту елемента. |
| `CompName` | `System_ComputerName`: комп'ютер, якому належить індекс. |
| `TargetURL`, `DateVisited` | URL-адреса та час відвідування елемента історії Internet Explorer (`System_Link_TargetUrl`, `System_Link_DateVisited`). |
| `ActivityApp`, `ActivityText`, `ActivityContentURI`, `ActivityStartTime`, `ActivityEndTime` | Елемент історії дій Windows 10 (`System_Activity_*`, `System_ActivityHistory_*`). |
| `User`, `UserSource` | Профіль, названий у шляху елемента, і те, з якого шляху його прочитано (`ItemPath`, `ItemURL` або `GatherPath`). Порожнє поза профілем. |
| `ScopeID`, `SDID` | Ідентифікатори області та дескриптора безпеки з рядка збирача. |
| `CrawlNumber` | `CrawlNumberCrawled` рядка збирача. |
| `GatherFlags` | `TransactionFlags` рядка збирача. |
| `Priority` | Пріоритет обходу для елемента, який ще в черзі. ESE позначає його як NULL, щойно елемент пройдено обходом, тому для більшості елементів воно порожнє. |
| `PropertyCount` | Скільки властивостей сховище властивостей містить для елемента. |
| `OtherProperties` | Усі інші властивості у вигляді `Name=Value`, розділені `; `. |
| `SourceFile` | Прочитана база даних: шлях відносно вхідного каталогу або її назва з `-f`. |
| `Problems` | Значення, які не декодувалися, часи, що не є часами, дубльовані рядки та пошкодження. Не приховуйте. |

JSON і JSONL мають ті самі поля з типізованими значеннями: числа залишаються числами, а відсутні
значення дорівнюють `null`. Замість `OtherProperties` вони мають `Properties` — кожну властивість
елемента з її назвою, числовим ідентифікатором, типом стовпця та значенням, а для часу — сире число
тактів FILETIME поруч із часом, який із нього прочитано. `Gather` містить стовпці рядка збирача.

## Поведінка в хронології { #timeline-behavior }

`auto-parse` зберігає записи як `FileExistence/WindowsSearch.csv` (або `.json`/`.jsonl`) і додає їх до
об'єднаної хронології `-j`. `super-timeline` створює події з `DateCreated`, `DateModified` і
`DateAccessed` (часи файлу, як їх записав Windows Search), з `GatherTime` (коли елемент було
проіндексовано), з `DateVisited` елемента історії Internet Explorer і з `ActivityStartTime` та
`ActivityEndTime` дії. `GatherLastModified` стає подією лише для елемента, для якого сховище
властивостей не містить часу зміни. Елемент без жодного з них залишається в результатах розбору й
зараховується до записів без часу.

## Стратегія розслідування { #investigation-strategy }

1. Відфільтруйте `Store` за значенням `file` і шукайте файли, назви та слова справи в `ItemPath`, `Name`
   і `AutoSummary`. Індекс називає файли, які відтоді було видалено.
2. Читайте `GatherTime` як останній час, коли файл існував із записаними метаданими, а `DateModified`
   — як його час зміни на той момент.
3. Використовуйте `FileOwner` і `User`, щоб пов'язати файл з обліковим записом, а потім підтвердьте це
   за `$MFT`, журналом USN і `$Secure:$SDS`.
4. Для електронної пошти (`Store` `mapi`) та історії Internet Explorer (`iehistory`) переходьте до
   файлів Outlook і WebCache по повні записи.
5. Порівнюйте індекс між хостами: `CompName` називає комп'ютер, якому належить індекс, а індекс,
   зібраний з образу, називає файли на цьому образі.

## Перевірені версії { #tested-versions }

Тести використовують публічні індекси, результати яких незалежно прочитали інші інструменти:

- Індекс Windows 7 із plaso (`SystemIndex_0A`), звірений з `esedbexport` від libyal, зокрема кодування
  рядків Windows 7.
- Індекс Windows 10 із DFIRArtifactMuseum, залишений «брудним», звірений із SIDR від Stroz Friedberg, і
  його індекс Windows 11 (ревізія формату ESE 200), звірений з `esedbexport`.
- Індекс SQLite Windows 11 22H2 із DFIRArtifactMuseum, про який повідомлено як про непідтримуваний
  новіший формат.
- Обрізані й порожні бази даних, а також бази іншого типу.

## Додаткові відомості { #further-details }

- [Microsoft — огляд Windows Search](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — формат файлу бази даних ESE](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb — Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg — SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft — вихідний код Extensible Storage Engine](https://github.com/microsoft/Extensible-Storage-Engine)
