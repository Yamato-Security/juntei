# ShimCache (AppCompatCache) { #shimcache-appcompatcache }

## Що таке ShimCache { #what-shimcache-is }

Підсистема сумісності програм Windows (Application Compatibility) кешує шляхи до виконуваних файлів і метадані під час оцінювання
сумісності/поведінки шимів. Кеш зберігається, навіть якщо файли перейменовано, видалено або від'єднано разом
зі знімним томом. Його структура та доступні поля відрізняються у версіях від Windows XP до Windows 11.

ShimCache цінний для виявлення історичних шляхів і, якщо формат має надійний прапорець виконання,
як додаткове підтвердження виконання. Кешований час зазвичай є **часом останньої зміни файлу**,
а не часом, коли Windows побачила або запустила його.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> Windows зафіксувала шлях під час обробки
сумісності. Сам лише запис є доказом наявності/спостереження; використовуйте `Executed` лише там, де це підтримується.

## Розташування та збирання { #location-and-collection }

Байти зберігаються в кущі реєстру `SYSTEM`, зазвичай у:

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

Збирайте `SYSTEM`, `SYSTEM.LOG1` і `SYSTEM.LOG2`. Juntei читає всі набори керування та визначає
`Select\Current`, якщо це можливо.

!!! warning "Найновіших активних записів може не бути в кущі"
    Windows може утримувати оновлення ShimCache в пам'яті та записувати їх під час вимкнення/перезавантаження. Кущ, скопійований
    з працюючої системи, може не містити найновіших спостережень. Час останнього запису кешу в реєстрі
    допомагає описати, коли його було збережено, але не є часом виконання окремого запису.

## Застереження для аналітика { #analyst-cautions }

- Запис кешу сам по собі **не доводить виконання**.
- `Executed` може мати значення `Yes`, `No` або `NA`; доступна семантика залежить від формату/версії.
- `LastModifiedTimeUTC` належить файлу, на який посилається запис, а не спостереженню чи виконанню.
- Порядок корисний, але не є універсальною точною хронологією; дублікати трапляються в різних наборах керування/slack.
- Записи `FromSlack` відновлено поза активним кешем, і вони потребують додаткової обережності.
- «Брудний» `SYSTEM` без придатних журналів може не містити поточних записів. Читайте підсумок відновлення.
- Знахідки вказують на підозрілі шаблони назв/шляхів, а не є вердиктом про шкідливе ПЗ.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shimcache -f SYSTEM -o shimcache.csv
juntei parse-registry shimcache -d evidence/ -t jsonl \
  | jq 'select(.Executed == true or (.Findings | length > 0))'
```

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `ControlSet` | Номер набору керування, що містить кеш. Перш ніж вибрати один, порівняйте `IsCurrent`. |
| `CacheEntryPosition` | Позиція/порядок у декодованому кеші. Корисна для відносного впорядкування, але не є часом. |
| `Path` | Кешований шлях до виконуваного файлу/пакета. Основна точка переходу для пошуку загроз. |
| `LastModifiedTimeUTC` | Час зміни файлу, збережений у записі, — не час виконання/спостереження. |
| `Executed` | `Yes`, `No` або `NA` відповідно до прапорців підтримуваних форматів. Використовуйте лише з урахуванням версії. |
| `Duplicate` | Той самий запис уже траплявся в попередньому наборі керування або slack; допомагає уникнути подвійного підрахунку. |
| `SourceFile` | Вхідний кущ/необроблене значення. Необхідне для роботи з кількома хостами. |
| `FromSlack` | Запис відновлено зі slack значення кешу, а не з активного списку. Нижча достовірність, можливо, старіший запис. |
| `CacheFormat` | Декодована структура Windows/кешу. Визначає доступність і семантику полів. |
| `CacheLastWrittenUTC` | Час збереження ключа реєстру/кешу. Це спільний контекст, а не час запуску кожного запису. |
| `InsertFlags`, `ShimFlags` | Необроблені прапорці сумісності; `Executed` — інтерпретований Juntei результат там, де це підтримується. |
| `DataSize` | Розмір даних цього запису, корисний для структурної перевірки. |
| `Machine` | Архітектура, записана новішими форматами (`x64`, `x86`, `ARM64`). |
| `FileSize`, `LastUpdateTimeUTC` | Кешовані метадані, специфічні для Windows XP, якщо вони наявні. |
| `PackageName`, `PackageVersion`, `PackagePublisherID` | Ідентичність пакетної програми в записах, де це підтримується. Publisher ID є контекстом, а не доказом підпису. |
| `Findings` | Коди аномалій Juntei, як-от переміщений системний двійковий файл або підозрілий шлях підготовки (staging). Лише зачіпки. |
| `RegistryPath`, `RegistryValueName` | Точне джерело — ключ/значення куща. Зберігайте для перевірки. |
| `CacheValueSize` | Розмір усього значення реєстру, а не `DataSize` окремого запису. |
| `IsCurrent` | Чи збігається набір керування цього запису з `SYSTEM\Select\Current`; порожнє, якщо це не відомо напевно. |
| `SelectCurrent`, `SelectDefault`, `SelectLastKnownGood`, `SelectFailed` | Необроблені номери наборів керування з `SYSTEM\Select` для контексту та аналізу відновлення. |

JSON/JSONL додатково зберігають дані записів, теги, підзаписи пакетних програм, структуровані знахідки,
проблеми кешу та подробиці відновлення.

## Стратегія розслідування { #investigation-strategy }

1. Надавайте перевагу поточному набору керування, але зберігайте інші набори та slack як історичні докази.
2. Шукайте за шляхами та знахідками; не використовуйте `LastModifiedTimeUTC` як час виконання.
3. Якщо `Executed == Yes`, підтверджуйте це за допомогою Prefetch, SRUM, журналів подій або інших слідів виконання.
4. Зіставляйте шляхи з `$MFT`/USN, щоб визначити історію створення, видалення та метаданих.
5. У звіті вказуйте, чи було відтворено журнали транзакцій і чи міг активний кеш не бути записаний на диск.

## Додаткові відомості { #further-details }

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [AppCompatCacheParser від Eric Zimmerman](https://github.com/EricZimmerman/AppCompatCacheParser)
- [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Специфікація формату реєстру Windows від Maxim Suhanov](https://github.com/msuhanov/regf)
