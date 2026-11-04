# ShellBags { #shellbags }

## Що таке ShellBags { #what-shellbags-are }

Explorer запам'ятовує, як відображається кожна папка, яку відкриває користувач: її вигляд, розмір значків, порядок сортування та
положення вікна. Щоб знову знайти параметри папки, він зберігає ідентичність папки в реєстрі користувача
у вигляді дерева елементів оболонки — тих самих елементів, що містить список ідентифікаторів цілі посилання: диск, а потім кожна
папка аж до відкритої. Ці дерева і є ShellBags.

Вони охоплюють папки, відкриті в Explorer і в діалогах відкриття та збереження, зокрема папки на знімних
дисках, мережевих спільних папках, портативних пристроях (телефонах, камерах), в архівах ZIP і на серверах FTP, а також
сторінки панелі керування. Вони залишаються після того, як папка, диск або мережева спільна папка зникли.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> Explorer цього користувача показував цю папку
(або папку всередині неї). ShellBag не показує, що робилося в папці, і не датує
кожне відвідування: див. часові позначки нижче.

## Розташування { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

Ключ `BagMRU` містить пронумеровані значення, кожне з яких — це один елемент оболонки, і значення `MRUListEx`, що задає їхній
порядок, від останнього відкритого. Кожне значення має однойменний підключ для елементів
усередині нього, тож шлях — це елементи від `BagMRU` до значення. `NodeSlot` підключа — це
номер його ключа `Bags\<N>`, у якому зберігаються параметри вигляду папки.

Збирайте `UsrClass.dat` і `NTUSER.DAT` разом із журналами транзакцій (`.LOG1`, `.LOG2`) для кожного
профілю, а також системний кущ `SOFTWARE`, який дає SID користувача `NTUSER.DAT`.

## Що означає кожна часова позначка { #which-times-say-what }

- **Часові позначки елемента оболонки** (`ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC`) — це
  власні часові позначки файлової системи папки (точність FAT: 2 секунди) на момент, коли Explorer записав елемент. Вони
  датують папку, а не відвідування, і не оновлюються під час наступних відвідувань.
  Годинники FAT не мають збереженого зміщення UTC. Застарілі заголовки `UTC` і відображений суфікс `Z` — це
  угоди відображення, а не доказ часового поясу джерела. `auto-parse --timezone` зберігає
  ці поля без змін; інтерпретуйте їх з урахуванням контексту вихідної системи.
- **Час останнього запису ключа** змінюється, коли перезаписується його `MRUListEx`, тобто коли один із його
  елементів відкривають і переміщують на початок. Тож час ключа, у якому міститься значення елемента
  (`ParentKeyLastWrittenUTC`), є часом останнього відкриття елемента, **лише якщо це найновіший
  елемент у цьому ключі** (`IsMostRecentInParent` має значення true, `MruPosition` 0). Для інших елементів це пізніший
  час, коли було відкрито сусідній елемент.
- Власний ключ елемента (`KeyLastWrittenUTC`) так само датує відкриття його найновішого дочірнього елемента.

## Застереження для аналітика { #analyst-cautions }

- **Відсутність доводить мало.** Explorer записує ShellBags, коли зберігається вигляд папки; папка,
  яку бачили лише коротко або відкрили іншим способом (командним рядком, сценарієм, програмою), може їх не мати.
  Користувач може видалити ключі.
- **Посилання NTFS належить папці** (`MftEntry`, `MftSequence`) на момент запису елемента.
  Знайдіть його в `$MFT`, щоб перевірити, чи існує досі та сама папка, або чи інший файл повторно використав
  цей запис.
- **Знімні диски** названо за літерою диска. Це літера, яку диск мав на той момент;
  зіставте її з історією USB і змонтованих пристроїв, щоб визначити пристрій.
- **Імена походять з елемента.** Папка, пізніше перейменована, зберігає тут своє старе ім'я. Кореневі та відомі
  папки названо так, як їх зазвичай називають криміналістичні інструменти (`This PC`, `Quick Access`, `Downloads`);
  `Guid` дає їхній ідентифікатор.
- **Результати пошуку, бібліотеки та сторінки панелі керування** — це подання, а не папки на диску
  (`property_view`, `control_panel_item`, `control_panel_category`).
- **Елемент, невідомий жодному парсеру,** зберігає свої байти в `DataRaw` з `ItemType` `unknown`, а його
  байт класу — в `ItemClass`. Про пошкоджені або незвичні елементи повідомляється в `Problems`, і їхні байти
  також зберігаються.
- **«Брудний» кущ** без своїх журналів може не містити найновіших елементів. Juntei відтворює журнали, розташовані поруч із
  кущем, якщо не вказано `--no-logs`; `TransactionLogStatus` вказує, чи походить запис із них.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shell-bags -d evidence/ -o shellbags.csv
juntei parse-registry shell-bags -f UsrClass.dat -t jsonl \
  | jq 'select(.ItemType == "volume" and .GUID == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

Елементи оболонки декодуються за допомогою ys-lnk-parser — парсера елементів оболонки, який використовує `parse-file lnk`.
Підсумок перелічує останні відкриті папки (від найновіших) за часом останнього запису їхнього батьківського ключа
разом із їхніми користувачами.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath`, `HiveType` | Хост, ім'я папки профілю, SID користувача (його називає кореневий ключ `UsrClass.dat`; для `NTUSER.DAT` — системний `ProfileList`), кущ і його тип. |
| `BagKey` | Ключ елемента під коренем куща (`…\BagMRU\4\3\0`): його значення міститься в ключі рівнем вище. |
| `MRU-Position`, `IsMostRecentInParent` | Його місце в `MRUListEx` батьківського ключа (0: останній відкритий); порожнє, якщо список його не містить. |
| `NodeSlot` | Номер його ключа `Bags\<N>` (параметри вигляду папки). |
| `Path` | Елементи від вершини його дерева до нього, об'єднані через `\`. `[unknown]` позначає елемент, значення якого відсутнє. |
| `ItemType`, `ItemClass` | Що являє собою елемент (`root_folder`, `volume`, `directory`, `file`, `network_location`, `uri`, `ftp_folder`, `compressed_folder`, `portable_device`, `portable_device_folder`, `users_files_folder`, `property_view`, `control_panel_item`, `control_panel_category`, `cd_burn`, `unknown`) і його байт класу. |
| `ItemName`, `GUID` | Його ім'я (довге ім'я папки) та ідентифікатор класу кореневої папки, елемента панелі керування або делегованої папки. |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | **З елемента оболонки:** часові позначки папки на момент запису елемента (FAT, точність 2 секунди). |
| `MFT-Entry`, `MFT-Sequence` | **З елемента оболонки:** посилання на файл NTFS папки. |
| `ExtensionBlocks` | Скільки блоків розширення має елемент. |
| `KeyLastWrittenUTC` | **З реєстру:** останній запис власного ключа елемента: коли було відкрито його найновіший дочірній елемент. |
| `ParentKeyLastWrittenUTC` | **З реєстру:** останній запис ключа, у якому міститься його значення: коли його було востаннє відкрито, якщо `IsMostRecentInParent` має значення true. |
| `DataRaw` | Байти значення (hex) для елемента, який не вдалося повністю розпізнати або який має проблеми. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Значення, яке називає `MRUListEx` ключа, але якого немає в ключі, значення, якого не називає список, підключ без свого значення, значення, яке не є двійковим, `NodeSlot` неправильного типу, пошкоджений список елементів оболонки. |

## Стратегія розслідування { #investigation-strategy }

1. Складіть перелік знімних дисків (`volume` з `Guid` Removable Drives), мережевих розташувань, папок FTP
   і портативних пристроїв, а також папок у них.
2. Шукайте цікаві папки: доступні користувачу для запису місця підготовки даних (`Temp`, `ProgramData`, `Public`),
   папки інструментів і архівів, профілі інших користувачів і папки, яких уже не існує (перевірте
   посилання NTFS у `$MFT`).
3. Датуйте відвідування лише за `ParentKeyLastWrittenUTC` тих елементів, де `IsMostRecentInParent` має значення true;
   решту зіставляйте з файлами LNK, Jump Lists і RecentDocs, які датують відкриття файлів.
4. Використовуйте `parse-registry read --recover-deleted` для видалених ключів і значень `BagMRU`.

## Додаткові відомості { #further-details }

- [libyal libfwsi: формат Windows Shell Item](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb: Most recently used (BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

Пошук SID у `ProfileList` вимагає одного унікального, синтаксично коректного SID. Неоднозначні зіставлення папок профілів і резервні ключі, як-от `S-1-...bak`, залишають `UserSid` невідомим.
