# Локальні облікові записи SAM { #sam-local-accounts }

## Що містить SAM { #what-the-sam-contains }

Кущ диспетчера облікових записів безпеки (Security Account Manager, `SAM`) зберігає записи локальних облікових записів. Juntei декодує
імена облікових записів, RID, поля профілю/коментарів, прапорці керування обліковим записом, позначки часу останнього входу/зміни пароля, лічильники,
підказки до паролів, зв'язок з обліковим записом Microsoft, шляхи профілів і те, чи приховує Windows обліковий запис
в інтерфейсі входу.

Це корисно для виявлення нещодавно створених локальних адміністраторів, увімкнених неактивних облікових записів, змін
паролів, прихованих облікових записів і проміжних облікових записів для латерального переміщення.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> локальний SAM представляв обліковий запис із
записаним станом. Це не повна історія автентифікації, і вона не описує доменні облікові записи.

## Розташування та збирання { #location-and-collection }

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

Збирайте кущ і журнали з достатніми привілеями/сирим доступом. Ідентифікацію хоста можна доповнити з
відповідного куща `SYSTEM` у колекції.

## Застереження для аналітика { #analyst-cautions }

- Ця команда аналізує **локальні** облікові записи, а не користувачів домену Active Directory.
- Juntei навмисно не вилучає хеші паролів.
- RID 500 — це вбудований обліковий запис Administrator, навіть якщо його перейменовано; самого імені недостатньо.
- `LastLogon` і лічильники — це стан реєстру, а не повний журнал аудиту кожного сеансу.
- `HiddenFromLogonUI` приховує обліковий запис з екрана входу; він не вимикає обліковий запис.
- Вбудовані `DefaultAccount` і `WDAGUtilityAccount` можуть бути приховані цілком легітимно.
- Порожні прапорці облікового запису через пошкоджене/відсутнє значення `F` не можна інтерпретувати як увімкнений обліковий запис.
- `NamesKeyLastWrittenUTC` приблизно вказує на контекст реєстрації імені облікового запису, але не є гарантованим часом створення.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.HiddenFromLogonUi == true or (.AccountFlags | index("disabled") | not))'
```

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `SourcePath` | Контекст хоста та точний кущ SAM. |
| `RID` | Відносний ідентифікатор (Relative Identifier). Надійна точка опори для ідентифікації облікового запису; 500 — вбудований Administrator. |
| `User` | Поточне ім'я локального облікового запису. Може бути перейменоване; зіставляйте за RID. |
| `FullName`, `Comment`, `UserComment` | Описовий текст; контролюється зловмисником і корисний для перевірки маскування. |
| `AccountFlags` | Декодовані стани, як-от вимкнений, заблокований, пароль не потрібен, звичайний обліковий запис, термін дії пароля не обмежений. |
| `AccountFlagsValue` | Сирі числові прапорці для перевірки та непідтримуваних бітів. |
| `CountryCode`, `CodePage` | Метадані локалі/кодової сторінки облікового запису; зазвичай це контекст, а не атрибуція. |
| `LastLogonUTC`, `LastLogonWholeSecondsUTC` | Той самий стан останнього входу з повною точністю та з точністю до цілих секунд для сумісності. Не повна історія. |
| `PasswordLastSetUTC` | Записаний час зміни пароля. Досліджуйте неочікувано нові/старі значення. |
| `AccountExpirationUTC` | Налаштований термін дії, якщо він скінченний. Порожнє значення може означати «ніколи»/«не встановлено». |
| `LastFailedLogonUTC` | Остання записана невдала локальна спроба входу у стані SAM; підтверджуйте подіями Security. |
| `FailedLogonCount`, `LogonCount` | Накопичувальні лічильники/лічильники стану з особливостями оновлення в Windows; використовуйте як контекст, а не як точну кількість подій. |
| `PasswordHint` | Чутлива підказка, надана користувачем; може розкривати шаблони паролів. Поводьтеся з нею належним чином. |
| `InternetUserName` | Ідентичність пов'язаного облікового запису Microsoft, якщо є. |
| `HiddenFromLogonUI` | Стан приховування в інтерфейсі. Перевіряйте високі RID/неочікувані власні облікові записи; не означає, що обліковий запис вимкнено. |
| `HomeDirectory`, `HomeDrive`, `ProfilePath`, `LogonScript` | Середовище/конфігурація входу локального облікового запису; незвичні скрипти/шляхи UNC можуть вказувати на закріплення. |
| `KeyLastWrittenUTC` | Час зміни ключа облікового запису RID. На рівні ключа, а не окремого поля. |
| `NamesKeyLastWrittenUTC` | Час зміни ключа зіставлення імені користувача; корисне наближення життєвого циклу облікового запису. |
| `LastLogonFiletime`, `PasswordLastSetFiletime`, `AccountExpirationFiletime`, `LastFailedLogonFiletime` | Сирі 64-бітні значення FILETIME, збережені для перевірки та обробки спеціальних сигнальних значень. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відтворення реєстру. |
| `Problems` | Розбіжності між F/V/Names, відсутні записи, некоректні зміщення або пошкодження під час декодування. |

## Стратегія розслідування { #investigation-strategy }

1. Порівняйте RID, ім'я, прапорці, шлях профілю та `NamesKeyLastWrittenUTC` з базовим станом хоста.
2. Перевірте увімкнені облікові записи з RID 1000+, облікові записи, для яких пароль не потрібен, з необмеженим терміном дії, приховані або нещодавно змінені.
3. Зіставляйте з подіями Security 4720/4722/4724/4732, подіями входу, створенням профілів і віддаленою активністю.
4. Шукайте в історії відновлення реєстру, якщо обліковий запис було вилучено до збирання.
5. Відрізняйте легітимні облікові записи, створені під час розгортання, та типові облікові записи від інтерактивних облікових записів, створених зловмисником.

## Додаткові відомості { #further-details }

- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Плагін SAM у dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
- [Registry Explorer Еріка Ціммермана](https://ericzimmerman.github.io/#!index.md)
