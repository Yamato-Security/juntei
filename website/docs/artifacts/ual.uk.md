# Журналювання доступу користувачів (UAL) { #user-access-logging-ual }

## Що таке UAL { #what-ual-is }

Журналювання доступу користувачів (User Access Logging, UAL) — це функція Windows Server, яка встановлена й
увімкнена за замовчуванням починаючи з Windows Server 2012 і агрегує в базах даних ESE запити клієнтів до ролей
сервера. Для кожної ролі, адреси клієнта й облікового запису UAL зберігає, коли клієнта вперше та востаннє було
помічено протягом року, загальну кількість запитів і лічильник для кожного дня року. Серед ролей — File Server
(SMB), Active Directory Domain Services, Active Directory Certificate Services, DHCP Server, Print and Document
Services, Remote Access, Web Server (IIS) і Windows Server Update Services.

UAL цінний для розслідувань латерального переміщення: його рядки клієнтів показують, які адреси й облікові
записи використовували сервер і в які дні, — часто ще довго після того, як журнали подій було перезаписано.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> клієнт із записаною адресою, автентифікований
під записаним обліковим записом, використовував роль у кожен день, для якого є лічильник.

## Розташування та збирання { #location-and-collection }

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

`Current.mdb` містить найновішу активність. `SystemIdentity.mdb` описує сервер, містить назви ролей і вказує
файл `{GUID}.mdb`, у якому зберігається кожен рік (`CHAINED_DATABASES`). Служба UAL тримає бази даних
відкритими: збирайте весь каталог `Sum` за допомогою криміналістичного збирача або зі знімка тому.

!!! warning "«Брудні» бази даних"
    `Current.mdb`, зібраний із працюючого сервера, часто «брудний». Juntei читає його в зібраному вигляді та
    повідомляє про це, але не відтворює `Svc*.log`: активність, що існує лише в журналах транзакцій,
    відсутня.

## Застереження для аналітика { #analyst-cautions }

- UAL підраховує запити за день і не журналює кожен запит окремо. Єдині часові позначки рядка клієнта —
  `InsertDate` і `LastAccess`.
- Дні в `DailyAccesses` — це дні за UTC.
- `Address` — це адреса, яку бачив сервер: між ним і справжнім клієнтом може стояти шлюз NAT, проксі або VPN.
  `::1` і `127.0.0.1` — це сам сервер.
- Аналізуйте всі бази даних каталогу `Sum`: той самий клієнт може мати рядки і в `Current.mdb`, і в
  `{GUID}.mdb` свого року.
- Відсутність рядків не доводить, що ніхто не підключався: UAL можна вимкнути (`Disable-Ual` або зупинивши
  службу User Access Logging), а в «брудній» базі даних бракує її найновіших змін.
- Без `SystemIdentity.mdb` значення `RoleName` береться з назв, які Windows Server дає своїм ролям.
- Нульовий або некоректний `LastAccess` не визначає рік. Juntei тоді використовує коректний `InsertDate`, а далі метадані річної бази; без коректного року зберігається `DayOfYear`.
- Підсумки додають усі вихідні рядки. `Current.mdb`, річні бази та знімки можуть містити ту саму активність; це не кількість з’єднань чи входів після усунення дублікатів.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese ual <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Current.mdb, {GUID}.mdb and SystemIdentity.mdb
  -f, --file <FILE>      Parse one UAL database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese ual -d Sum -o ual-csv
juntei parse-ese ual -d Sum -t jsonl \
  | jq 'select(.Table == "CLIENTS") | {Address, AuthenticatedUserName, RoleName, LastAccess}'
```

## Таблиці виводу { #output-tables }

CSV створює по одному файлу на таблицю; якщо таблиця порожня, файл містить лише заголовок. Рядки JSON і JSONL
містять `SourceFile` (базу даних) і `Table` (назву таблиці в базі даних).

| Файл CSV | Таблиця | Вміст |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | Клієнти кожної ролі: адреса, обліковий запис, перший і останній доступ за рік, загальна кількість запитів і кількість за кожен день. |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | Коли кожну роль було використано вперше та востаннє. |
| `UAL-DNS.csv` | `DNS` | Пари адреси та імені хоста з часом, коли кожну з них було помічено востаннє. |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | Віртуальні машини Hyper-V: ідентифікатори, серійний номер, час створення та останньої активності. |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | Сервер: ім'я, домен, версія Windows, тип продукту, обладнання, часовий пояс і останнє завантаження; новий рядок може свідчити про перейменування, приєднання до домену або підвищення до контролера домену. |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | Файл `{GUID}.mdb` кожного року. |
| `UAL-Roles.csv` | `ROLE_IDS` | GUID ролей з їхніми назвами. |
| `UAL-<table>.csv` | Інша таблиця | Усі стовпці таблиці, якої Juntei не знає. |

## Поля { #fields }

| Поле | Інтерпретація |
|---|---|
| `SourceFile` | База даних, з якої прочитано рядок. |
| `RoleGUID`, `RoleName` | Роль і її назва з `SystemIdentity.mdb`, що лежить поруч із базою даних (за його відсутності — вбудовані назви). |
| `TenantID` | GUID орендаря (tenant) клієнта, якщо роль його повідомляє. |
| `TotalAccesses` | Запити клієнта за рік бази даних; сума `DailyAccesses`. |
| `InsertDate` | Коли створено рядок: перший запит клієнта за рік у цій базі даних (UTC). |
| `LastAccess` | Останній запит клієнта (UTC). |
| `Address` | IPv4- або IPv6-адреса клієнта. |
| `AuthenticatedUserName` | Обліковий запис, під яким автентифікувався клієнт (`DOMAIN\user`; імена облікових записів комп'ютерів закінчуються на `$`). |
| `ClientName` | Ім'я клієнта, якщо роль його повідомляє. |
| `DailyAccesses` | Дні із запитами та їхня кількість зі стовпців `Day1` … `Day366`, датовані роком `LastAccess`: `2023-01-12: 5, 2023-01-13: 2` у CSV, об'єкти з `Date` і `Accesses` у JSON. День, рік якого невідомий, має `DayOfYear` замість `Date`. |
| `FirstSeen`, `LastSeen` | Перше й останнє використання ролі (`ROLE_ACCESS`), остання поява пари DNS (`DNS`), за UTC. |
| `CreationTime`, `LastSeenActive` | Коли було створено віртуальну машину або рядок `SYSTEM_IDENTITY` і коли віртуальна машина була активна востаннє, за UTC. |
| `OS-ProductType`, `OS-ProductTypeName` | 1 `Workstation`, 2 `Domain Controller`, 3 `Server`. |
| `OS-CurrentTimeZone` | Зміщення сервера відносно UTC у хвилинах (`-420` — це UTC-07:00). |
| `OS-LastBootUpTime` | Останнє завантаження сервера за UTC (якщо його не вдається прочитати — текст WMI у збереженому вигляді). |
| Інші стовпці | Усі інші стовпці таблиці під мітками полів Juntei у PascalCase, наприклад `SystemDNS-HostName`, `SystemDomainName`, `OS-BuildNumber` і `SystemSerialNumber`. |

## Автоматичний аналіз { #automatic-parsing }

`auto-parse` записує таблиці до `LateralMovement` як `UAL-Clients.csv`, `UAL-RoleAccess.csv` тощо (один файл
`UAL.json` або `UAL.jsonl` з `-t json` чи `-t jsonl`) разом із `LateralMovement/UAL-logs.txt`. Об'єднана
хронологія `-j` містить кожен рядок, із `Comp` та ідентифікатором `Artifact` `ual`. `-Z` перетворює
`InsertDate`, `LastAccess`, `FirstSeen`, `LastSeen`, `CreationTime`, `LastSeenActive` і
`OS-LastBootUpTime`.

## Додаткові відомості { #further-details }

- [Microsoft — User Access Logging](https://learn.microsoft.com/en-us/windows-server/administration/user-access-logging/get-started-with-user-access-logging)
- [Plaso — User Access Logging parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/user_access_logging.py)
- [Velociraptor — Windows.Forensics.UserAccessLogs](https://docs.velociraptor.app/artifact_references/pages/windows.forensics.useraccesslogs/)
- [CrowdStrike — How to Leverage User Access Logging for Forensic Investigations](https://www.crowdstrike.com/en-us/blog/user-access-logging-ual-overview/)
