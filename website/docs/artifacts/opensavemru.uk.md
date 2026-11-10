# Діалоги відкриття та збереження файлів (OpenSaveMRU) { #open-and-save-as-dialogs-opensavemru }

## Що таке історія діалогів { #what-the-dialog-history-is }

Коли програма показує стандартний діалог «Відкрити» або «Зберегти як», Windows записує в
`NTUSER.DAT` користувача:

- **який файл було відкрито або збережено:** `OpenSavePidlMRU` має по одному підключу на розширення (`docx`,
  `ps1`, …) і `*` для всіх розширень, кожен з яких є списком нещодавно використаних елементів;
- **яка програма використовувала діалог і в якій папці він був:** `LastVisitedPidlMRU` (і
  `LastVisitedPidlMRULegacy` для програм, які використовують старіші діалоги);
- **програми, які показували діалог:** `CIDSizeMRU` (розмір діалогу для кожної програми) і
  `FirstFolder` (шлях програми та папка);
- у Windows 2000 і XP — `OpenSaveMRU` (шляхи за розширеннями) і `LastVisitedMRU` (програми та
  їхні папки у вигляді шляхів).

Починаючи з Windows Vista, файли та папки зберігаються як списки елементів оболонки (shell items) — ті самі структури, що й ціль
ярлика: кожен елемент — це частина шляху (This PC, диск, папка, файл), а
елемент файлу чи папки може містити власні значення часу та посилання на файл NTFS.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> програма використала діалог «Відкрити» або «Зберегти як»,
щоб відкрити чи зберегти цей файл або в цій папці, у сеансі цього користувача. Датовано лише найновіший
запис кожного списку: часом останнього запису його ключа.

## Розташування { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

Зберіть `NTUSER.DAT` разом із журналами транзакцій для кожного профілю.

## Застереження для аналітика { #analyst-cautions }

- **Один час на список.** Час останнього запису ключа змінюється, коли один із його записів переміщується на
  початок списку, тому він датує лише запис у позиції 0 (`IsMostRecent`, `LastUsedUTC`).
  Інші записи впорядковано від найновішого до найстарішого вниз за списком, але вони не датовані.
- **Той самий файл у двох списках.** Файл є в списку свого розширення та в `*`; не рахуйте це
  як два використання. `*` дає порядок у межах усіх розширень.
- **Відкрито чи збережено.** Списки цього не вказують. Зіставляйте із записами `$MFT` і журналу USN
  для файлу, файлами LNK і Jump Lists.
- **Час елементів — це час файлу, а не діалогу.** `ItemModifiedUTC`, `ItemCreatedUTC` і
  `ItemAccessedUTC` — це час файлу на момент запису елемента, з точністю до 2 секунд (FAT). Елементи
  відомих папок (`CLSID_ThisPCDocumentsRegFolder`, `Downloads`, …) часто їх не мають. Ці часові мітки FAT
  не мають зміщення часового поясу. Суфікс стовпця `UTC` — це лише умовність відображення, а не
  доказ того, що вихідний годинник працював в UTC; перш ніж порівнювати їх із FILETIME реєстру, зіставте їх з часовим поясом вихідної системи
  та іншими часовими мітками. Див.
  [рекомендації Microsoft щодо часу файлів](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times).
- **Шляхи — це шляхи оболонки.** Вони починаються від робочого столу: `This PC\C:\Users\…`, відома папка,
  бібліотека (`UsersLibraries\Documents\…`) або мережеве розташування. Шлях, що починається з папки
  або файлу, вказує на робочий стіл. Порожній шлях — це сам робочий стіл.
- **Недекодовані елементи.** Елемент типу, невідомого парсеру елементів оболонки, показується за його
  класом (`[class 0x00]`), його тип — `unknown`, а байти списку містяться в `DataRaw`.
- **Програми — це імена.** `LastVisitedPidlMRU` і `CIDSizeMRU` зберігають ім'я файлу
  (`WINWORD.EXE`), а не шлях. `FirstFolder` зберігає шлях.
- **Списки короткі й очищаються.** Кожен список зберігає обмежену кількість записів, а засоби очищення
  видаляють ключі. Для видалених значень використовуйте `parse-registry read --recover-deleted`.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry open-save-mru -d evidence/ -o opensavemru.csv
juntei parse-registry open-save-mru -f NTUSER.DAT -t jsonl \
  | jq 'select(.List == "OpenSavePidlMRU" and .Extension != "*")'
```

Зведення перелічує записи кожного списку, програми, які використовували діалоги, і найновіший файл кожного куща
(перший запис `*`).

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Хост, власник профілю (зі шляху або папок оболонки), SID із `ProfileList` системи та кущ. |
| `List` | `OpenSavePidlMRU`, `LastVisitedPidlMRU`, `LastVisitedPidlMRULegacy`, `CIDSizeMRU`, `FirstFolder`, `OpenSaveMRU` або `LastVisitedMRU`. |
| `Extension` | Підключ розширення (`docx`, `*`); порожнє для інших списків. |
| `MRU-Position`, `IsMostRecent` | Місце у списку ключа (0 — найновіший); порожнє, якщо список не називає значення. |
| `ValueName` | Номер (`MRUListEx`) або літера (`MRUList`) значення. |
| `Program` | Програма, яка показала діалог. |
| `Path` | Файл або папка: об'єднані імена елементів оболонки або шлях, збережений як текст. |
| `ItemType` | Тип останнього елемента оболонки: `file`, `directory`, `root_folder`, `volume`, `network_location`, `property_view`, `unknown`, … |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | Час файлу або папки в тому вигляді, як його записав елемент оболонки. |
| `MFT-Entry`, `MFT-Sequence` | Посилання на файл NTFS, записане елементом оболонки. |
| `LastUsedUTC` | Час останнього запису ключа, лише для найновішого запису. |
| `KeyPath`, `KeyLastWrittenUTC` | Ключ і його останній запис: час його найновішого запису, а не кожного рядка. |
| `DataRaw` | Байти значення (hex), якщо вони не декодуються, або список елементів оболонки, якщо тип елемента невідомий. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Значення, які список називає, але яких немає в ключі, значення, яких він не називає, неправильні типи, обрізані імена або списки. |

## Стратегія розслідування { #investigation-strategy }

1. Прочитайте `*` по порядку для кожного користувача та `LastUsedUTC` для найновішого файлу.
2. Шукайте скрипти, архіви, виконувані файли та документи в розташуваннях, доступних користувачу для запису, знімних або мережевих
   розташуваннях, а також файли з назвами, що нагадують облікові дані або експорти.
3. Використовуйте `LastVisitedPidlMRU`, щоб пов'язати програму з папкою, яку вона використовувала востаннє, а `CIDSizeMRU` і
   `FirstFolder` — для програм, які взагалі використовували діалоги.
4. Зіставляйте шляхи та `MftEntry` з `$MFT`, журналом USN, файлами LNK, Jump Lists,
   ShellBags і RecentDocs.

## Додаткові відомості { #further-details }

- [libyal winreg-kb: ключі нещодавно використаних елементів (MRU)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi: формат Windows Shell Item](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
