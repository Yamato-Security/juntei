# Кеш планувальника завдань { #task-scheduler-cache }

## Що таке TaskCache { #what-taskcache-is }

Планувальник завдань Windows (Task Scheduler) зберігає кеш у реєстрі в `SOFTWARE`, який пов'язує шляхи завдань із GUID,
тригерами/списками, суб'єктами безпеки (principals), діями, метаданими виконання та дескрипторами безпеки. Juntei декодує
дії виконання та обробника COM, а також застарілі дії надсилання електронної пошти/повідомлень, часові позначки завдань і ознаки
завдань, прихованих від звичайного інтерфейсу Task Scheduler.

Завдання — поширений механізм закріплення та віддаленого виконання. Кеш у реєстрі також цінний,
коли зловмисник видаляє або змінює XML-файл завдання.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> TaskCache налаштував або зберіг
завдання/дію. Щоб довести конкретне виконання, потрібні поля часу виконання та незалежні журнали.

## Розташування { #locations }

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

Збирайте `SOFTWARE` разом із журналами транзакцій і повне дерево `System32\Tasks`. Журнали подій Task Scheduler
Operational і Security додають докази створення/оновлення/виконання.

## Застереження для аналітика { #analyst-cautions }

- Кеш у реєстрі та XML завдання можуть не збігатися після підробки, видалення, неповної реєстрації або пошкодження.
- `Hidden` у Juntei визначає структурне приховування (відсутній зв'язок із Tree/дескриптор безпеки за
  відповідних умов), а не просто завдання, налаштоване зі звичайним прапорцем інтерфейсу «Hidden».
- Windows 7 зберігає дії завдань у XML, а не в значенні `Actions` кешу; порожня дія там
  не є автоматично підозрілою.
- `LastRun` і `LastResult` корисні, але не є повною, захищеною від підробки історією виконання.
- `Context` називає налаштований суб'єкт безпеки/контекст, а не користувача, який зареєстрував завдання.
- CLSID обробників COM потребують окремого визначення через реєстр/файли.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.Hidden == true or (.Executable? | test("powershell|cmd|mshta"; "i")))'
```

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `SourcePath` | Контекст хоста та вихідний кущ `SOFTWARE`. |
| `TaskName` | Повний шлях у планувальнику. Основна точка переходу до XML і журналів подій. |
| `TaskGUID` | GUID, що поєднує записи кешу Tree і Tasks. Надійний ключ зіставлення. |
| `TaskIndex` | Метадані Tree/індексу, які використовує планувальник; контекст, а не кількість виконань. |
| `TaskLists` | Списки кешу (`Boot`, `Logon`, `Plain`, `Maintenance`), що класифікують запуск/обробку. |
| `Hidden` | Структурний індикатор прихованого завдання. Перш ніж робити висновок про підробку, перевірте `InTree`, `InTasks` і стан дескриптора. |
| `InTree`, `InTasks` | Чи існують обидві частини кешу. Відсутній зв'язок може вказувати на видалення/підробку/неповну реєстрацію. |
| `HasSecurityDescriptor` | Чи має запис Tree значення `SD`; його відсутність може сприяти приховуванню завдання у відповідних версіях. |
| `TreeKeyLastWrittenUTC`, `TaskKeyLastWrittenUTC` | Зміни на рівні ключа для кожної частини кешу; не точний час створення/запуску. |
| `ActionIndex` | Номер дії в завданні, починаючи з нуля. Одне завдання може дати кілька рядків. |
| `ActionType` | `exec`, `com_handler`, `send_email` або `show_message`. |
| `Executable`, `Args`, `WorkingDirectory` | Фактична дія виконання. Інтерпретуйте лапки, змінні, відносні шляхи та LOLBins. |
| `COM-ClassID`, `ActionData` | CLSID і дані обробника COM або застаріле корисне навантаження електронного листа/повідомлення. CLSID визначайте окремо. |
| `Context` | Контекст суб'єкта безпеки, закодований разом із діями (`LocalSystem`, `Author` тощо); налаштована ідентичність, а не той, хто реєстрував завдання. |
| `Author`, `URI`, `Description` | Метадані завдання. Зловмисник легко може їх контролювати; вони не є доказом довіри. |
| `LastRegisteredUTC` | Час реєстрації/оновлення з DynamicInfo, якщо він наявний. |
| `LastRunUTC`, `LastSuccessfulRunUTC`, `LastResult` | Кешовані метадані виконання/результату. Зіставляйте з операційними журналами та доказами процесів. |
| `ActionsRaw` | Вихідні байти дій, якщо декодування неповне або потрібна перевірка. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Невідповідність шляху/GUID, неправильний тип значення, усічена структура дії або відсутні частини. |

## Стратегія розслідування { #investigation-strategy }

1. Спершу перевірте приховані/неузгоджені завдання та шляхи, що не належать Microsoft.
2. Відтворіть фактичні команди, зокрема аргументи, робочий каталог, розгортання змінних середовища та визначення COM.
3. Порівняйте запис реєстру з XML завдання та обчисліть хеш/перевірте часові позначки виконуваного файлу, на який він посилається.
4. Зіставляйте `TaskName`/GUID/час із подіями Task Scheduler, Security 4698/4702, журналами процесів, Prefetch і MFT/USN.
5. Якщо зловмисник видалив завдання, шукайте у видалених/відновлених даних реєстру.

## Додаткові відомості { #further-details }

- [Microsoft: схема Task Scheduler](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft: журнал подій Task Scheduler](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 Scheduled Task](https://attack.mitre.org/techniques/T1053/005/)
