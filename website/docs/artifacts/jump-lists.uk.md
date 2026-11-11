# Jump Lists { #jump-lists }

## Що таке Jump Lists { #what-jump-lists-are }

Jump Lists лежать в основі меню, які відображаються для записів програм на панелі завдань і в меню «Пуск». Автоматичні Jump
Lists — це керовані Windows історії елементів, які використовувала програма. Користувацькі Jump Lists — це
визначені програмою категорії та завдання. Обидва типи можуть вбудовувати структури LNK зі шляхами до цілей, часом,
контекстом тому/мережі, посиланнями MFT і аргументами запуску.

Автоматичні записи можуть виявити файли або папки, відкриті програмою, порядок/час останнього використання,
кількість взаємодій і стан закріплення. Користувацькі списки можуть розкривати «нещодавно закриті» сторінки або завдання
програми, зокрема шкідливі аргументи команд.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> програма-власник записала або відобразила
елемент/завдання у своєму Jump List. Поведінка відрізняється залежно від програми; підтверджуйте фактичне виконання/доступ.

## Розташування та збирання { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

Шістнадцятковий префікс імені файлу — це AppID. Збирайте обидва каталоги та зберігайте шлях
профілю користувача. Автоматичні списки — це контейнери OLE Compound File з потоком `DestList` і пронумерованими потоками
LNK; користувацькі списки використовують власний формат категорій/посилань.

## Застереження для аналітика { #analyst-cautions }

- Описи AppID походять із відомого зіставлення; невідомий AppID сам по собі не є підозрілим.
- Поля `LastModified` / останнього використання належать запису DestList; час вбудованої цілі — інший.
- Лічильники взаємодій існують лише в підтримуваних версіях і не є повною кількістю відкриттів.
- Закріплення зберігає елементи довше, ніж звичайна давність; очищення або параметри програми можуть видаляти інші.
- Список може містити застарілі/неприкріплені потоки або байти з попереднього, довшого користувацького списку.
- Вбудовані аргументи та шляхи LNK контролюються зловмисником. Прив'язка до програми — це контекст, а не підстава для довіри.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-list-csv
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

З `auto-parse` таблиці CSV — це `FileAndFolderOpening/JumpLists-AutomaticDestinations.csv`
і `FileAndFolderOpening/JumpLists-CustomDestinations.csv` безпосередньо в каталозі кожного комп'ютера.
Спільний журнал — `JumpLists-logs.txt` у цій категорії. Маніфести перелічують точні шляхи
в `OutputFiles`. Окремі експорти CSV зберігають `AutomaticDestinations.csv` і
`CustomDestinations.csv` у вибраному вами каталозі. Автоматичні експорти JSON і JSONL — це
окремі файли `JumpLists.json` або `JumpLists.jsonl`; структура записів не змінюється.

## Поля CSV AutomaticDestinations { #automaticdestinations-csv-fields }

| Група полів | Поля та їх інтерпретація |
|---|---|
| Джерело | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: контейнер Jump List і часові позначки його файлової системи. |
| Програма | `AppId`, `AppIdDescription`: AppID з імені файлу та відоме зіставлення з програмою. Не робіть висновків про ідентичність на основі невідомого хешу. |
| Контейнер | `HasSps`, `DestListVersion`, `LastUsedEntryNumber`: структура/версія DestList і контекст заголовка. |
| Впорядкування | `MRU`, `EntryNumber`: позиція за останнім використанням і шістнадцятковий ідентифікатор запису/потоку. |
| Метадані використання | `CreationTime`, `LastModified`, `InteractionCount`, `PinStatus`, `PinPosition`: життєвий цикл/використання/стан закріплення в DestList. Інтерпретуйте з урахуванням версії. |
| Хост/шлях | `Hostname`, `MacAddress`, `Path`, `DisplayPath`: розташування цілі та контекст відстеження. Хост/MAC не є єдиною підставою для атрибуції. |
| Ідентифікатори відстеження | `FileBirthDroid`, `FileDroid`, `VolumeBirthDroid`, `VolumeDroid`: зіставляйте переміщені цілі та відстеження у вбудованих LNK. |
| Примітки | `Notes`: повідомляє про неприкріплені потоки, відновлені залишки або інший контекст парсера. |
| Вбудований LNK | `TargetCreated`, `TargetModified`, `TargetAccessed`, `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`, `DriveType`, `VolumeSerialNumber`, `VolumeLabel`, `LocalPath`, `CommonPath`, `TargetIDAbsolutePath`, `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`, `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `ExtraBlocksPresent`, `Args`: інтерпретуйте, як описано на сторінці [Ярлики LNK](lnk.md). |
| Додаткові дані LNK | `NetworkPath`, `Name`, `IconLocation`: мережеві поля та поля відображення з вбудованого ярлика. |
| Перевірка | `Problems`: відсутні потоки, усічення, розбіжність лічильників або пошкодження посилань. |

## Поля CSV CustomDestinations { #customdestinations-csv-fields }

Вивід користувацьких списків починається з `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`, `AppId`,
`AppIdDescription` і `EntryName` (назва категорії або `Tasks`), за якими йдуть усі вбудовані поля LNK,
перелічені вище, а потім `Notes`, `NetworkPath`, `Name`, `IconLocation` і `Problems`.

JSON/JSONL додатково зберігає заголовок DestList, оцінку/контрольну суму/сховище властивостей кожного запису,
`DestListPropertyStore`, потоки складеного файлу, користувацькі категорії, байти після останньої категорії
та вкладені поля кожного вбудованого LNK.

## Стратегія розслідування { #investigation-strategy }

1. Визначте користувача та вихідний профіль, а потім обережно зіставте AppID із програмою.
2. Відсортуйте автоматичні записи за `LastModified` і `MRU`; відокремте закріплені елементи.
3. Перегляньте шляхи, мережеві спільні папки, серійні номери знімних томів, посилання MFT і аргументи.
4. Перевірте `Notes` і неприкріплені/залишкові посилання — там можуть зберігатися видалені або попередні елементи.
5. Зіставляйте з RecentDocs, окремими файлами LNK у Recent, Prefetch, SRUM, `$MFT` і журналами подій.

## Додаткові відомості { #further-details }

- [Microsoft: формат Compound Binary File (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft: формат Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [Тестовий/довідковий проєкт парсера JumpList](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
