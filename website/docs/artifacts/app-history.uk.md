# Історія програм { #application-history }

## Що залишають по собі інструменти { #what-tools-leave-behind }

Багато сторонніх інструментів зберігають параметри та історії в `NTUSER.DAT` користувача. Кілька з них належать до
інструментів, якими користуються зловмисники або за допомогою яких вони готують дані перед викраденням: інструменти Sysinternals, архіватори,
клієнти хмарної синхронізації та засоби віддаленого доступу. Juntei зчитує такі дані:

| Програма | Ключ у `HKCU\Software` | Що показує |
|---|---|---|
| Sysinternals | `Sysinternals\<tool>` | Кожен інструмент, ліцензію якого прийняв користувач (`EulaAccepted`): PsExec, ProcDump, Sysmon, … |
| WinRAR | `WinRAR\ArcHistory`, `DialogEditHistory\ArcName`, `DialogEditHistory\ExtrPath` | Відкриті архіви; назви архівів і папки розпакування, введені в його діалогах |
| 7-Zip | `7-Zip\Compression\ArcHistory`, `Extraction\PathHistory`, `FM\FolderHistory`, `FM\CopyHistory`, `FM\PanelPath0/1` | Створені архіви; папки розпакування; папки, які переглядали та в які копіювали у його файловому менеджері |
| WinZip | `Nico Mak Computing\WinZip\filemenu`, `extract`, `mru\archives` | Архіви в меню File, папки розпакування, нещодавні архіви |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | Адреса електронної пошти, локальна папка та імена кожного облікового запису; синхронізовані бібліотеки (`ScopeIdToMountPointPathCache`) і папки (`Tenants\<tenant>`) |
| Adobe Acrobat і Reader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`, `cRecentFolders` | Останні відкриті файли PDF і папки з іменами файлів, розмірами та датами |
| TeamViewer | `TeamViewer` (і `Version<N>` старішої версії) | Усі значення: імена облікових записів і користувачів, початкові папки передавання файлів (`<partner ID>?<local>|<remote>`), … |
| CCleaner | `Piriform\CCleaner` | Усі значення: що налаштовано очищати, `UpdateKey`, … |

<span class="evidence-strength">Найобґрунтованіший висновок:</span> цей користувач запускав інструмент (або прийняв його
ліцензію), або інструмент відкривав, створював чи розпаковував ці архіви та папки, або цей хмарний обліковий запис
було налаштовано для користувача. Більшість історій не мають часу для кожного запису: лише найновіший запис отримує
час останнього запису ключа свого списку.

## Розташування { #location }

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

Збирайте `NTUSER.DAT` кожного користувача разом із журналами транзакцій, а також системний `SOFTWARE` для
SID користувачів.

## Застереження для аналітика { #analyst-cautions }

- **Прийняття EULA Sysinternals.** `FirstUsedUTC` — це оцінка першого використання на основі часу останнього запису ключа,
  якщо його єдиним значенням є читабельне `EulaAccepted`, встановлене в `1`. Відхилена або
  пошкоджена EULA такої оцінки не отримує. Параметри реєстру також можуть бути заздалегідь налаштовані,
  скопійовані або відредаговані: підтверджуйте виконання та його час іншими артефактами.
  Інструменти, які зберігають там свої параметри (Process Explorer, Autoruns, …), пізніше перезаписують ключ:
  тоді `FirstUsedUTC` порожнє, а `KeyLastWrittenUTC` відображає лише останню зміну параметрів. Інструмент,
  запущений з `-accepteula` або від імені іншого користувача (віддалена сторона PsExec працює як SYSTEM), записує
  ключ саме цього користувача.
- **Позиції, а не час.** Записи списку впорядковані (`MruPosition` 0 — найновіший), але
  лише останній запис ключа списку датує найновіший запис. Старіші записи використовувалися раніше, у
  невідомий час.
- **Введені імена.** `ArcName` у WinRAR — це назва архіву, введена під час його створення: часто це ім'я файлу
  без папки.
- **Шляхи Adobe** зберігаються так, як їх записує Adobe (`/C/Users/alice/a.pdf` означає
  `C:\Users\alice\a.pdf`). Його `sDate` (у `Details`) — це рядок дати PDF, збережений у записаному вигляді.
- **`UpdateKey` у CCleaner** — це локальний час без часового поясу; він зберігається у записаному вигляді.
- **TeamViewer.** Журнали підключень — це файли (`Connections_incoming.txt`, `TeamViewer*_Logfile.log`),
  а не значення реєстру: збирайте і їх.
- **Відсутність.** Користувачі та інструменти можуть очищати ці історії (зокрема й сам CCleaner).

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.App == "Sysinternals" and .FirstUsed != null)'
```

Підсумок перелічує найновіші оцінки першого використання Sysinternals, архіви та папки, які кожен список використовував
останніми, а також облікові записи OneDrive.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `SourcePath` | Хост і `NTUSER.DAT`. |
| `User`, `UserSID` | Ім'я папки профілю та її SID із системного `ProfileList`. |
| `App` | `Sysinternals`, `WinRAR`, `7-Zip`, `WinZip`, `OneDrive`, `Adobe Acrobat Reader`, `Adobe Acrobat`, `TeamViewer`, `CCleaner`. |
| `Item` | Інструмент Sysinternals, список (`ArcHistory`, `ExtrPath`, `FolderHistory`, `filemenu`, `cRecentFiles`, …), обліковий запис OneDrive або ключ його синхронізованих папок, або назва значення. |
| `MRU-Position` | Позиція запису в його списку, 0 — найновіший. |
| `Path` | Архів, папка або документ. |
| `Value` | `EulaAccepted`, адреса електронної пошти облікового запису OneDrive або ідентифікатор синхронізованої бібліотеки, значення TeamViewer або CCleaner. |
| `FirstUsedUTC` | Оцінка першого використання на основі прийняття EULA (див. застереження вище). |
| `Details` | `DisplayName`, `UserCID`, `ConfiguredTenantId`, … облікового запису OneDrive; `Version`, `tFileName`, `uFileSize`, `uPageCount`, `sDate` документа Adobe. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName` | Звідки походить запис і коли цей ключ востаннє змінювався. |
| `DataRaw` | Байти значення (hex), якщо їх не вдається декодувати. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Дані, які не вдалося декодувати або прочитати. |

## Стратегія розслідування { #investigation-strategy }

1. Шукайте інструменти Sysinternals, які звичайний користувач не запускає (PsExec, ProcDump, PsLoggedOn,
   SDelete), та їхні часові позначки EULA.
2. Перегляньте назви архівів і папки розпакування в період інциденту: архіви, створені в
   `ProgramData`, `Temp` або папці користувача до того, як дані залишили систему.
3. Перевірте облікові записи OneDrive, які не належать організації, а також імена облікових записів TeamViewer і папки
   передавання файлів.
4. Зіставляйте з Prefetch, Amcache, BAM і UserAssist (запуск інструмента), `$MFT` і журналом
   USN (файл архіву), файлами LNK і Jump Lists (відкриті документи), а також журналами подій.

## Додаткові відомості { #further-details }

- [Microsoft: Sysinternals](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
