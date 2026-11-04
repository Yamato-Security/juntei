# Відомості про систему { #system-information }

## Що це таке { #what-it-is }

Чотири машинні кущі системи повідомляють те, що аналітику потрібно знати, перш ніж читати її хронологію:

- її імена (NetBIOS, ім'я хоста DNS, домен DNS) і домен або робоча група, до якої вона належить;
- її версію Windows, збірку та ревізію оновлення, а також коли Windows було встановлено або востаннє оновлено до нового випуску;
- її часовий пояс, який указує на місцевий час, що його вела система;
- коли вона востаннє коректно завершила роботу;
- її архітектуру процесора та кодові сторінки;
- користувача, який входив останнім, і профілі всіх користувачів, які коли-небудь входили;
- її SID: SID комп'ютера (локальні облікові записи — це він плюс RID) і SID її домену;
- версії Windows, з яких її було оновлено.

`parse-registry system-info` записує один запис на систему з усіма цими відомостями.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> конфігурація у тому стані, у якому вона була на момент збирання
кущів (або їх останнього запису). Час ключів показує, коли окремі її частини востаннє змінювалися.

## Розташування { #location }

```text
C:\Windows\System32\config\SYSTEM     (the control set Windows used last: Select\Current)
  ControlSet00N\Control\ComputerName\ComputerName     ComputerName
  ControlSet00N\Services\Tcpip\Parameters             Hostname, Domain
  ControlSet00N\Control\TimeZoneInformation           TimeZoneKeyName, Bias, ActiveTimeBias, …
  ControlSet00N\Control\Windows                       ShutdownTime
  ControlSet00N\Control\Session Manager\Environment   PROCESSOR_ARCHITECTURE
  ControlSet00N\Control\Nls\CodePage                  ACP, OEMCP
  Setup\Source OS (Updated on M/D/YYYY HH:MM:SS)      the version before each upgrade

C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion                 ProductName, EditionID, DisplayVersion,
                                                      CurrentBuild, UBR, InstallDate, InstallTime, …
  Microsoft\Windows NT\CurrentVersion\ProfileList     one key per profile (SID)
  Microsoft\Windows\CurrentVersion\Authentication\LogonUI   LastLoggedOnUser, …

C:\Windows\System32\config\SAM
  SAM\Domains\Account                                 V: the computer's SID at its end

C:\Windows\System32\config\SECURITY
  Policy\PolAcDmN, PolAcDmS                           the computer's name and SID
  Policy\PolPrDmN, PolPrDmS                           its domain (or workgroup) and the domain's SID
  Policy\PolDnDDN, PolDnTrN, PolDnDmG                 the domain's and forest's DNS names, GUID
  Policy\PolMachineAccountS                           the computer account's SID in the domain
```

Зберіть чотири кущі разом із їхніми журналами транзакцій. Кожен із них окремо дає те, що в ньому міститься.

## Застереження для аналітика { #analyst-cautions }

- **Час реєстру — в UTC; часовий пояс потрібен для всього іншого.** Час ключів, FILETIME і більшість
  часу подій — в UTC. Текстові журнали, деякі журнали програм і дати, записані в імена або рядки
  (як-от `Source OS (Updated on …)`), можуть бути в місцевому часі. `Bias` — це кількість хвилин від місцевого стандартного
  часу до UTC (`UTC = local + Bias`). `UtcOffset` і `DaylightUtcOffset` дають два зміщення,
  а `TimeZoneDescription` вказує, коли починається та закінчується літній час.
- **Часовий пояс може змінюватися.** `TimeZoneLastWrittenUTC` — це останній запис ключа. Windows також
  самостійно перезаписує ключ, коли оновлює `ActiveTimeBias` (зміщення, чинне на той момент), тому
  нещодавній час не доводить, що хтось змінив пояс. Порівнюйте з журналами подій: Security 4616
  (змінено системний час) і події `Microsoft-Windows-Kernel-General` журналу System.
- **`StandardStart` і `DaylightStart` — це `TIME_FIELDS` ядра, а не `SYSTEMTIME`.** День
  тижня — їхнє останнє поле, а не третє (як у значенні `TZI` пояса в `SOFTWARE`). У
  поясах США обидва формати читаються однаково, бо днем тижня є неділя (0). Juntei декодує їх
  як `TIME_FIELDS`: публічна система з острова Пасхи (перша субота о 22:00) показує
  різницю. Місяць 0 означає відсутність переходу на літній час.
- **Windows 11 називає себе "Windows 10".** `ProductName` зберігає `Windows 10 …` у Windows 11. Збірки 22000
  і новіші — це Windows 11, і зведення про це повідомляє. `CurrentVersion` має значення `6.3` у Windows 8.1, 10
  і 11; `Version` використовує `CurrentMajorVersionNumber` (10), якщо воно є.
- **`InstallDate` змінюється з оновленнями функцій.** Оновлення на місці до нового випуску (наприклад, з 1809 до
  21H2) скидає його. Випуск, який замінило кожне оновлення, з власним `InstallDate`, міститься
  в `PreviousVersions` (`Setup\Source OS`). Найраніший із цих `InstallDate` найближчий до
  початкової інсталяції. Дата в імені ключа `Source OS` не вказує свого часового поясу.
- **`ShutdownTime` — це останнє коректне завершення роботи.** Збій або втрата живлення його не оновлюють.
  Порівнюйте його з журналами подій (6006, 6008, 1074) і з часом останніх записів у кущі.
- **`LastLoggedOnUser` — це останній користувач екрана входу,** а не кожен вхід. Віддалений робочий стіл,
  `runas`, служби та заплановані завдання не завжди його встановлюють. `LogonUiLastWrittenUTC` приблизно вказує, коли
  його було встановлено востаннє.
- **Профілі:** `ProfileList` перелічує кожного користувача (і службу), який мав профіль у
  системі, включно з видаленими обліковими записами, доки їхній профіль не вилучено. Windows 10 і новіші записують
  `LocalProfileLoadTime` і `LocalProfileUnloadTime`: приблизний час останнього входу та виходу.
- **SID:** локальні облікові записи мають вигляд `MachineSid-<RID>`, доменні облікові записи — `DomainSid-<RID>`.
  `MachineSid` береться з `SAM` (інакше з `PolAcDmS` куща `SECURITY`). Ці два значення мають збігатися;
  якщо ні, про це повідомляє `Problems`, як це було б із кущами різних систем. `DomainSid` порожнє
  в робочій групі.
- **Один каталог — одна система.** З `-d` кущі в одному каталозі вважаються кущами однієї системи.
  Копії `RegBack` і другий кущ того самого виду (`SYSTEM.old`) утворюють окремі записи.
  З `-f` задані кущі вважаються кущами однієї системи, а інші шукаються поруч із ними.
- **Набори керування:** читається лише набір керування, який Windows використовувала востаннє. `parse-registry read` показує
  інші.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM, SOFTWARE, SAM and SECURITY hives
  -f, --file <FILE>      A hive of one system; repeat for each
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry system-info -d collection/ -o system-info.csv
juntei parse-registry system-info -f SYSTEM -f SOFTWARE -t jsonl \
  | jq '{Host, TimeZoneDescription, ProductName, Build, LastShutdown}'
```

Для кожної системи зведення наводить: її версію Windows, час інсталяції, її часовий пояс
словами, зміщення, чинне на момент останнього запису ключа, останнє завершення роботи, її домен або
робочу групу, останнього користувача, який входив, її SID, кількість профілів і версії, з яких її
було оновлено. `auto-parse` також виводить часовий пояс кожної системи, коли перелічує знайдені
системи.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `SourcePath`, `SoftwarePath`, `SamPath`, `SecurityPath` | Хост і прочитані кущі (`SourcePath` — кущ `SYSTEM`, інакше перший прочитаний). |
| `CompName`, `Hostname`, `Domain` | Ім'я NetBIOS (`SYSTEM`, інакше `PolAcDmN` з `SECURITY`), ім'я хоста DNS і домен DNS (`Tcpip\Parameters`). |
| `PrimaryDomain`, `DNS-Domain` | Домен, до якого приєднано комп'ютер, або його робоча група (`PolPrDmN`), та DNS-ім'я домену (`PolDnDDN`). |
| `ProductName`, `Edition`, `DisplayVersion`, `Version`, `Build`, `Ubr`, `ServicePack`, `InstallationType` | `CurrentVersion`: назва продукту в записаному вигляді, `EditionID`, `DisplayVersion` (інакше `ReleaseId`), основна та додаткова версія (інакше `CurrentVersion`), збірка, ревізія збірки оновлення, `CSDVersion` і `Client`, `Server` або `Server Core`. |
| `InstallDateUTC`, `InstallTimeUTC` | `InstallDate` (з точністю до секунди) і `InstallTime` (з точністю до 100 нс, Windows 10 і новіші): інсталяція або останнє оновлення функцій. |
| `RegisteredOwner`, `RegisteredOrganization`, `SystemRoot` | У записаному вигляді. |
| `TimeZone`, `TimeZoneDescription` | `TimeZoneKeyName` (інакше `StandardName`) і пояс словами з його зміщеннями та датами переходу на літній час. |
| `UTC-Offset`, `DaylightUTC-Offset` | Зміщення місцевого стандартного часу від UTC і зміщення літнього часу (порожнє, якщо переходу на літній час немає). |
| `BiasMinutes`, `ActiveBiasMinutes`, `StandardBiasMinutes`, `DaylightBiasMinutes` | `Bias`, `ActiveTimeBias` (чинне на момент запису ключа), `StandardBias`, `DaylightBias`. |
| `StandardName`, `DaylightName` | У записаному вигляді, часто посилання на ресурс (`@tzres.dll,-112`). |
| `StandardStart`, `DaylightStart` | Коли починається стандартний і літній час, словами (`first Sunday of November, 02:00`). |
| `DynamicDaylightTimeDisabled`, `RealTimeIsUniversal` | Динамічний перехід на літній час вимкнено; апаратний годинник працює в UTC. |
| `TimeZoneLastWrittenUTC` | Останній запис ключа часового поясу. |
| `LastShutdownUTC` | `ShutdownTime`: останнє коректне завершення роботи. |
| `CurrentControlSet`, `ProcessorArchitecture`, `AnsiCodePage`, `OemCodePage` | Прочитаний набір керування, `PROCESSOR_ARCHITECTURE`, `ACP` і `OEMCP`. |
| `LastLoggedOnUser`, `LastLoggedOnSamUser`, `LastLoggedOnUserSID`, `LastLoggedOnDisplayName`, `LogonUiLastWrittenUTC` | Останній користувач екрана входу та останній запис його ключа. |
| `MachineSID`, `DomainSID`, `MachineAccountSID` | SID комп'ютера, SID його домену (`PolPrDmS`) і SID облікового запису комп'ютера в домені (`PolMachineAccountS`). |
| `CompNameLastWrittenUTC`, `PrimaryDomainLastWrittenUTC`, `VersionLastWrittenUTC` | Приблизно коли комп'ютер востаннє було названо, востаннє приєднано до домену або робочої групи та востаннє оновлено (ключ `CurrentVersion`). |
| `PreviousVersions` | Кожен ключ `Source OS`: версія, її дата інсталяції та дата в імені ключа (JSON: по одному об'єкту на кожен). |
| `Profiles` | Кожен профіль: SID, папка, час ключа, останнє завантаження та вивантаження (JSON: по одному об'єкту на кожен). |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру для всіх чотирьох кущів. |
| `Problems` | Значення неправильного типу або розміру (їхні байти збережено), а також `SAM` і `SECURITY` з різними SID комп'ютера. |

## Стратегія розслідування { #investigation-strategy }

1. Спершу прочитайте часовий пояс: зверніть увагу на `UtcOffset`, дати переходу на літній час і те, чи змінювався пояс
   нещодавно. Перетворюйте за ним журнали в місцевому часі.
2. Зверніть увагу на версію, збірку та дату інсталяції. Перевірте `PreviousVersions` на наявність оновлень і порівняйте
   дату інсталяції з найстарішим часом файлової системи та журналів подій.
3. Зверніть увагу на `LastShutdownUTC`: активність після нього без нового часу завершення роботи вказує на збій, втрату
   живлення або збирання з працюючої системи.
4. Використовуйте `MachineSid` і `DomainSid`, щоб відрізняти локальні облікові записи від доменних у SID в інших джерелах (BAM,
   `ProfileList`, журнали подій), а час завантаження та вивантаження профілів — для останніх входів.
5. Зіставляйте імена комп'ютера та домену з журналами подій, а також з `CompNameLastWrittenUTC`
   і `PrimaryDomainLastWrittenUTC` для перейменувань і приєднань до домену.

## Додаткові відомості { #further-details }

- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft: DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft: TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft: ідентифікатори безпеки](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb: часові пояси](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb: відомості про продукт Windows](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
