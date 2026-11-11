# Кущі реєстру Windows { #windows-registry-hives }

## Що таке кущі реєстру { #what-registry-hives-are }

Реєстр Windows зберігає конфігурацію системи та користувачів у двійкових файлах кущів. Ключі утворюють дерево;
значення під ключем містять типізовані дані. Артефакти реєстру описують закріплення, встановлене програмне забезпечення,
служби, облікові записи, мережеву конфігурацію, дії користувачів, параметри безпеки і майже кожну частину
стану Windows.

Універсальний засіб читання Juntei декодує будь-який основний кущ, відтворює журнали транзакцій, повідомляє про пошкодження і може
відновлювати видалені записи та записи з вільних комірок, cell slack, залишкових даних, стану до відтворення і лише з журналів транзакцій.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> кущ містив або зберіг у придатному для відновлення вигляді
стан ключа/значення. Час останнього запису ключа показує, що ключ змінився, але не те, яке значення змінилося або чому.

## Поширені кущі та їх розташування { #common-hives-and-locations }

| Кущ | Типовий шлях | Приклади доказів |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | Служби, драйвери, набори керування, ім'я хоста, змонтовані пристрої, ShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | Встановлене програмне забезпечення, TaskCache, профілі, конфігурація Windows |
| `SAM` | `C:\Windows\System32\config\SAM` | Локальні облікові записи та їхні метадані |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | Метадані локальної політики безпеки/секретів (залежно від доступу) |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | Конфігурація типового системного профілю |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | Уподобання користувача, ключі Run, RecentDocs, активність програм |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | Класи оболонки, COM/активність оболонки користувача |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | Інвентаризація програм (для семантичного виводу використовуйте `parse-registry amcache`) |

Збирайте розташовані поруч файли `.LOG`, `.LOG1` і `.LOG2`. Кущ, скопійований під час роботи Windows, може бути
«брудним»; журнали транзакцій містять зафіксований стан, якого сам основний файл ще не відображає.

## Застереження для аналітика { #analyst-cautions }

- Позначка часу останнього запису ключа стосується **ключа**, а не окремого значення.
- Значення реєстру зазвичай не мають власної позначки часу.
- `CurrentControlSet` — це псевдонім аналітика/Windows, що розв'язується через `SYSTEM\Select`; це не
  ключ, збережений буквально.
- Видалені записи можуть бути неповними, застарілими, дубльованими в різних джерелах відновлення або перезаписаними.
- Записи, наявні лише в журналі транзакцій, показують історичний/проміжний стан. Пояснюйте `recovered_from`.
- Результат `absent_value` означає, що запитаний ключ існує, але в проаналізованому стані не має цього значення;
  це не доводить, що значення ніколи не існувало.
- Рядки, шляхи та командні рядки — це конфігурація, а не доказ того, що налаштовану дію було виконано.
- «Брудний» кущ без застосовних журналів може пропускати стан або показувати застарілий.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host` | Ім'я хоста, визначене з доступного системного контексту; порожнє, якщо Juntei не може безпечно його ідентифікувати. |
| `HiveType` | Виявлена категорія куща (`SYSTEM`, `SOFTWARE`, `NTUSER.DAT` тощо). Використовуйте для інтерпретації шляхів. |
| `SourcePath` | Точний вхідний кущ. Необхідний, коли режим каталогу поєднує хости/користувачів. |
| `RecordType` | `key`, `value`, `default_value` або `absent_value`. Відсутність стосується лише запиту/стану. |
| `KeyPath` | Шлях під коренем куща. `[unknown]` позначає нерозв'язаних предків видаленого запису. |
| `KeyLastWrittenUTC` | Остання зміна ключа загалом. Не дає змоги визначити, яке значення змінилося. |
| `ValueName` | Іменоване значення; порожнє для типового/безіменного значення. |
| `ValueType` | Тип реєстру (`REG_SZ`, `REG_DWORD`, `REG_BINARY` тощо). Тип впливає на інтерпретацію. |
| `ValueData` | Значення, відображене з урахуванням типу. Багаторядкові значення залишаються списками в JSON. |
| `DataSize` | Розмір, записаний у значенні. Порівнюйте із сирими байтами та `Problems`. |
| `ValueDataRaw` | Точні байти, якщо декодування не вдалося або використано `--raw-data`. Зберігайте для перевірки. |
| `ValueU64` | Повне числове представлення для DWORD/QWORD без втрати точності в JavaScript. |
| `ValueFiletimeUTC` | Необов'язкове відображення QWORD як FILETIME. Має сенс лише тоді, коли схема ключа визначає FILETIME. |
| `AllocationStatus` | `allocated`, `deleted_complete`, `deleted_partial`, `residual_value_only` або `transaction_log_recovered`. Визначає рівень довіри. |
| `RecoveredFrom` | `free_cell`, `cell_slack`, `remnant_data`, `before_replay` або запис журналу, як-от `SYSTEM.LOG1#1234`. |
| `TransactionLogStatus` | `clean`, `replayed`, `unchanged` або `not_applied` для контексту цього запису/куща. |
| `LogsApplied` | Імена файлів журналів транзакцій, які відтворив Juntei. Фіксуйте їх у звіті. |
| `Problems` | Пошкодження, відсутні комірки, невідповідність типів, нерозв'язані списки/батьківські елементи або інші обмеження. |

## Інтерпретація стану відновлення { #recovery-status-interpretation }

| Стан | Значення |
|---|---|
| `deleted_complete` | Видалений ключ/значення відновлено з повним відомим шляхом/даними. Усе одно це історичний, а не активний стан. |
| `deleted_partial` | Частину шляху, списку значень, значення або даних перезаписано або вона непридатна для читання. Не заповнюйте прогалини з повторно використаних комірок. |
| `residual_value_only` | Значення збереглося без достатнього контексту ключа; `KeyPath` може бути відсутнім. |
| `transaction_log_recovered` | Стан існує лише у версіях до відтворення/версіях журналу і відсутній у поточних розподілених/вільних записах. |

## Стратегія розслідування { #investigation-strategy }

1. Почніть із точного семантичного ключа, якщо він відомий; для виявлення використовуйте пошук за regex/даними.
2. Перевірте `TransactionLogStatus`, `AllocationStatus`, `RecoveredFrom` і `Problems`, перш ніж аналізувати вміст.
3. Інтерпретуйте дані відповідно до задокументованої схеми ключа; `REG_BINARY` не є самоописовим.
4. Зіставляйте конфігурацію з доказами виконання (журнали подій, Prefetch, SRUM, служби/завдання).
5. Зберігайте сирі дані для нових або спірних полів.

## Додаткові відомості { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Специфікація формату реєстру Максима Суханова](https://github.com/msuhanov/regf)
- [Парсер реєстру yarp](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [Microsoft: типи значень реєстру](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
