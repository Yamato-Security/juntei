# Секрети LSA { #lsa-secrets }

## Що це таке { #what-they-are }

Локальний центр безпеки (Local Security Authority, LSA) зберігає секрети в кущі `SECURITY`. До них належать
пароль автоматичного входу, паролі облікових записів, від імені яких працюють служби, доменний пароль облікового запису
комп'ютера, системні ключі DPAPI та ключ, яким зашифровано кешовані доменні входи. LSA
також зберігає кешовані доменні входи, щоб користувачі домену могли входити без контролера домену.

`parse-registry lsa-secrets` перелічує, які секрети зберігає система і коли їх було встановлено, а також скільки
кешованих входів вона зберігає та використовує. **Команда читає лише назви та час: нічого не розшифровується, і
жодні байти секретів не читаються й не записуються.** Juntei читає докази; він не видобуває облікові дані.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> система зберігала секрет із цією назвою,
встановлений у цей час. Секрет служби (`_SC_<service>`) означає, що LSA зберігає пароль облікового запису,
від імені якого працює ця служба.

## Розташування { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| Назва | Вид | Що це |
|---|---|---|
| `DefaultPassword` | `autologon` | Пароль, з яким Winlogon автоматично виконує вхід. |
| `_SC_<service>` | `service_account` | Пароль облікового запису, від імені якого працює `<service>`. |
| `$MACHINE.ACC` | `machine_account` | Пароль облікового запису комп'ютера в його домені. |
| `DPAPI_SYSTEM` | `dpapi_system` | Ключі DPAPI системи. |
| `NL$KM` | `cached_logon_key` | Ключ, яким зашифровано кешовані входи. |
| `L$_SQSA_<SID>` | `security_questions` | Контрольні запитання локального облікового запису (Windows 10 і новіші). |
| `RasDialParams!<SID>#n`, `L$_Ras…` | `ras_credentials` | Облікові дані комутованого доступу та VPN. |
| `L$…`, `G$…`, `M$…` | `local`, `global`, `machine` | Інші приватні дані — за префіксом, який їм дає `LsaStorePrivateData`. |

Збирайте `SECURITY` разом із журналами транзакцій, а також кущ `SYSTEM` системи, який містить назву
хоста.

## Застереження для аналітика { #analyst-cautions }

- **Назви та час, а не значення.** Те, чи містить значення дані, визначається за його розміром. Його вміст
  не читається.
- **`DefaultPassword` не означає автоматичного входу.** Цей секрет існує в більшості систем, зокрема
  в тих, які ніколи не входять автоматично. Ключ `Microsoft\Windows NT\CurrentVersion\Winlogon` куща `SOFTWARE`
  (`AutoAdminLogon`, `DefaultUserName`) показує,
  чи налаштовано автоматичний вхід. Нещодавнє значення `CurrentSetUTC` — це зміна, яку варто пояснити.
- **Секрети служб:** `_SC_<service>` з'являється, коли службу налаштовано на роботу від імені облікового запису з
  паролем. Порівнюйте його з `parse-registry services` (`ObjectName`) і шукайте служби,
  які було встановлено нещодавно або які невідомі. Секрет може залишатися після видалення служби.
- **Час:** `CurrentSetUTC` і `OldSetUTC` — це моменти, коли Windows встановила поточне та попереднє
  значення. Пароль `$MACHINE.ACC` за замовчуванням змінюється кожні 30 днів, тож його час змінюється.
  `KeyLastWrittenUTC` і час ключів `CurrVal`/`OldVal` стосуються того, коли секрет востаннє
  змінювався.
- **Кешовані входи:** комірка використовується, якщо її запис називає користувача. Заголовок запису не
  зашифровано; з нього читається лише довжина імені користувача. `CachedLogonsKept` — це лічильник `NL$Control`,
  який відповідає `CachedLogonsCount` (10 за замовчуванням). Використовувані комірки означають, що користувачі домену входили
  інтерактивно, і їхні облікові дані можна зламати офлайн за допомогою куща. Імена користувачів
  зашифровано, і вони не декодуються. Використовуваними вважаються лише перевірені двійкові записи з повним заголовком і узгодженою довжиною імені користувача. Якщо `Problems` повідомляє про пошкоджені комірки, кількість використовуваних охоплює лише перевірені комірки; не інтерпретуйте пошкоджені комірки як порожні.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

Для кожного куща підсумок перелічує кожен секрет із його видом і часом встановлення. Потім він називає
служби, паролі облікових записів яких зберігає LSA, і підраховує використовувані комірки кешованих входів.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `SourcePath` | Хост (з куща `SYSTEM` системи) і кущ `SECURITY`. |
| `Secret`, `Kind` | Назва секрету та що це таке (див. таблицю вище); `Cache` і `cached_logons` — для запису кешованих входів. |
| `ServiceName`, `UserSID` | Служба, для якої призначено секрет `_SC_`, і SID користувача, який містить назва (`L$_SQSA_`, `RasDialParams`). |
| `CurrentSetUTC`, `OldSetUTC` | `CupdTime` і `OupdTime`: коли було встановлено поточне та попереднє значення. |
| `HasCurrentValue`, `HasOldValue` | Чи містять `CurrVal` і `OldVal` дані. |
| `CurrentValueLastWrittenUTC`, `OldValueLastWrittenUTC`, `KeyLastWrittenUTC` | Останні записи ключів `CurrVal`, `OldVal` і ключа секрету. |
| `CachedLogons`, `CachedLogonsInUse`, `CachedLogonsKept` | Запис кешованих входів: комірки, використовувані комірки та лічильник `NL$Control`. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Некоректний або позадіапазонний час (показуються лише метадані часу), пошкоджені заголовки кешу, неправильні типи значень кешу та ключі, які неможливо прочитати. |

## Стратегія розслідування { #investigation-strategy }

1. Шукайте секрети `_SC_` невідомих або нещодавно встановлених служб і зіставляйте їх із
   `parse-registry services` та подією 7045 (встановлено службу).
2. Перевіряйте час `DefaultPassword` відносно параметрів автоматичного входу в `Winlogon`.
3. На членах домену зверніть увагу на використовувані комірки кешованих входів і на час останнього запису `Cache`.
4. Порівнюйте час секретів із хронологією: секрет, установлений у вікні вторгнення, може
   вказувати на новий обліковий запис служби, автоматичний вхід або зміну облікового запису комп'ютера.

## Додаткові відомості { #further-details }

- [Microsoft: об'єкт приватних даних](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft: відомості про кешовані доменні входи](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [libyal winreg-kb: кешовані облікові дані](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
