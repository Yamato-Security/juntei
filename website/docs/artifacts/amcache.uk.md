# Amcache { #amcache }

## Що таке Amcache { #what-amcache-is }

Amcache — це інвентарний реєстр сумісності програм Windows, що зберігається в кущі реєстру. Сучасні системи
записують виконувані файли в `InventoryApplicationFile`, а пристрої Plug-and-Play — у
`InventoryDevicePnp`; старіші версії Windows використовують застарілу структуру `Root\File`.

Для дослідження виконуваних файлів Amcache цінний тим, що може зберігати повний шлях, SHA-1,
розмір файлу, метадані PE, відомості про версію, текст видавця, дату компонування (link date) і часові позначки реєстру
після того, як вихідний файл зник. Записи пристроїв можуть ідентифікувати під'єднане обладнання, драйвери,
дати встановлення та ідентифікатори.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> Windows занесла записаний файл або
пристрій до інвентарю. Запис виконуваного файлу є **доказом наявності, а не доказом виконання**.

## Розташування та збирання { #location-and-collection }

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

Збирайте кущ разом із сусідніми файлами `Amcache.hve.LOG`, `.LOG1` і `.LOG2`. Якщо кущ має стан dirty, `parse-registry amcache` автоматично застосовує придатні журнали транзакцій у пам’яті перед аналізом, не змінюючи докази. Відомості про відновлення записуються до журналу парсера або термінала; під час виведення записів у stdout діагностика надходить у stderr. Відсутні або непридатні журнали можуть призвести до неповних записів або неможливості прочитати неузгоджений кущ.

## Застереження для аналітика { #analyst-cautions }

- Запис виконуваного файлу в Amcache не доводить, що користувач запустив файл.
- `Publisher` — це метадані PE/ресурсу версії, записані Windows. Вони не доводять, що файл мав
  дійсний підпис Authenticode на момент збирання.
- Хеші можуть бути відсутні, а застарілі та сучасні схеми не надають ідентичних значень.
- `LinkDate` береться із заголовка PE і може бути підробленим, скопійованим або створеним відтворюваними збираннями.
- Час останнього запису ключа описує оновлення запису реєстру, а не обов'язково завантаження, створення,
  перше або останнє виконання.
- Шляхи можуть вказувати на знімні томи або томи, яких уже немає.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.RecordType == "file" and .SHA1 == "0123456789abcdef0123456789abcdef01234567")'
```

Вивід CSV містить таблиці підсумку, знахідок, файлів і пристроїв. З `auto-parse` це
`Registry-Amcache-Summary.csv`, `Registry-Amcache-Findings.csv`, `Registry-Amcache-Files.csv` і
`Registry-Amcache-Devices.csv` безпосередньо в каталозі `FileExistence/` кожного комп'ютера, а
`Registry-Amcache-logs.txt` розміщується поруч із таблицями. Для кількох кущів перед розширенням додається `-Source1`, `-Source2` тощо;
`OutputFiles` у маніфестах перелічує точні шляхи. Окремі експорти CSV
зберігають власні назви таблиць і вказане розташування виводу. Записи JSONL використовують
значення `RecordType` `summary`, `finding`, `file` і `device`.

## Поля виводу для файлів { #file-output-fields }

| Поле | Як аналітику його використовувати |
|---|---|
| `KeyName`, `SourceKeyPath` | Визначають точний вихідний запис реєстру та гілку схеми. Зберігайте для відтворюваності. |
| `KeyLastWrittenUTC`, `KeyLastWrittenFiletime` | Час оновлення запису реєстру в зрозумілому та необробленому вигляді. Не позначайте його як час виконання. |
| `SourceVolumeKeyLastWrittenUTC`, `SourceVolumeKeyLastWrittenFiletime` | Час батьківського ключа тому в застарілих схемах; контекст, а не час дії з файлом. |
| `FileID` | Записаний Amcache ідентифікатор файлу. Зберігається без змін; не вважайте, що це завжди SHA-1. |
| `SHA1`, `SHA256` | Надійні точки переходу до Defender, розвідданих про загрози, EDR і сховищ файлів. Null означає, що значення не записано. |
| `Name`, `OriginalFileName`, `Description` | Записані імена PE/версії. Порівнюйте з фактичним шляхом, щоб виявити маскування; метадані контролює зловмисник. |
| `FullPath` | Найкорисніша точка переходу за розташуванням. Обережно нормалізуйте регістр, але зберігайте вихідний рядок доказу. |
| `LongPathHash` | Ідентифікатор Windows, пов'язаний зі шляхом; корисний для зіставлення записів, але не є хешем вмісту. |
| `ProgramID` | Пов'язує файл із записом інвентарю програм, якщо такий є. |
| `Publisher` | Записаний текст видавця. Розглядайте як метадані, а не як перевірку підпису. |
| `Version`, `BinFileVersion`, `ProductName`, `ProductVersion`, `BinProductVersion` | Контекст ресурсу версії. Розбіжності або неправдоподібні значення можуть підтримати аналіз маскування. |
| `BinaryType` | Записаний контекст архітектури/типу; порівнюйте з аналізом PE, якщо файл відновлено. |
| `LinkDate` | Часова позначка компіляції/компонування PE в записаному вигляді. Корисна лише за скептичного ставлення та підтвердження. |
| `Size` | Записаний розмір; порівнюйте з `$MFT`, вмістом Defender або відновленим файлом. |
| `Language` | Ідентифікатор мови PE; слабка контекстна підказка, а не атрибуція. |
| `IsPE-File`, `IsOS-Component` | Записані Windows прапорці класифікації. Хибне значення не доводить нешкідливості. |
| `USN` | Значення послідовності оновлень NTFS, записане Amcache; може допомогти перейти до збережених записів USN. |
| `APPX-PackageFullName`, `APPX-PackageRelativeID` | Пов'язують пакетні програми/компоненти з файлом. |
| `RegistryValues` | Кожне вихідне значення реєстру з типом, декодованим значенням і необробленими шістнадцятковими даними. Використовуйте, якщо типізоване поле відсутнє, або для майбутніх/застарілих полів. |

## Поля виводу для пристроїв { #device-output-fields }

| Група полів | Поля та їх інтерпретація |
|---|---|
| Джерело | `KeyName`, `SourceKeyPath`, `KeyLastWrittenUTC`, `KeyLastWrittenFiletime`: точний запис пристрою та час оновлення реєстру. |
| Ідентичність | `Description`, `BusDescription`, `Manufacturer`, `Model`: зрозумілі людині описи пристрою; рядки постачальника можуть бути загальними. |
| Апаратні ідентифікатори | `HWID`, `CompID`, `Class`, `ClassGUID`, `ContainerID`, `DeviceInterfaceClasses`: найнадійніші точки переходу за сімейством і екземпляром пристрою. |
| Стан | `DeviceState`, `ProblemCode`, `ParentID`, `MatchingID`, `Enumerator`: контекст переліку пристроїв Windows і зв'язків між ними. |
| Драйвер | `DriverID`, `DriverName`, `DriverPackageStrongName`, `DriverVerDate`, `DriverVerVersion`, `Provider`, `Service`, `INF`: зв'язки з пакетом драйвера та службою; зіставляйте зі встановленими файлами та службами. |
| Встановлення | `InstallDate`, `FirstInstallDate`, `InstallState`, `ExtendedInfs`: записаний життєвий цикл встановлення. Відсутність дати не доводить, що пристрій ніколи не під'єднували. |
| Фільтри та стек | `LowerClassFilters`, `UpperClassFilters`, `LowerFilters`, `UpperFilters`, `StackID`, `LocationPaths`: розширений контекст стека драйверів; неочікувані драйвери-фільтри заслуговують на перевірку. |
| Збереження необроблених даних | `RegistryValues`: повні типізовані/необроблені вихідні значення для перевірки та непідтримуваних полів. |

Стовпці масивів і необроблених значень у CSV мають суфікс `JSON` (наприклад, `HWID-JSON` і
`RegistryValuesJSON`); їхні комірки містять JSON, тоді як JSON/JSONL зберігають власні масиви.

## Стратегія розслідування { #investigation-strategy }

1. Використовуйте як точки переходу `FullPath`, `SHA1`, `Name` і `USN`.
2. Шукайте відповідні записи Prefetch, щоб підтвердити виконання.
3. Використовуйте `$MFT` і журнал USN, щоб встановити створення, перейменування, видалення та розмір файлу.
4. Порівнюйте метадані видавця/версії з незалежно відновленим PE і результатом перевірки підпису.
5. Перевіряйте карантин Defender і журнали подій на наявність виявлень або видалення.

## Додаткові відомості { #further-details }

- [Схема виводу ys-amcache-parser для Juntei](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [AmcacheParser від Eric Zimmerman](https://github.com/EricZimmerman/AmcacheParser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [Плагін Amcache для Regipy](https://github.com/mkorman90/regipy)
