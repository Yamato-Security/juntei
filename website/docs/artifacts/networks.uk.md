# Мережі { #networks }

## Що це таке { #what-they-are }

Windows запам'ятовує мережі, до яких приєднувалася система, і те, як було налаштовано її мережу.

- **`SOFTWARE`:**
  - **Мережеві профілі** (`NetworkList\Profiles`): по одному на кожну мережу, до якої підключалася система,
    з її назвою (SSID мережі Wi-Fi), категорією (загальнодоступна, приватна, доменна), типом підключення
    (дротове, бездротове, VPN, мобільний широкосмуговий доступ) і часом першого та останнього підключення до неї.
  - **Підписи** (`NetworkList\Signatures`): те, що ідентифікує кожну мережу, як-от
    MAC-адреса шлюзу та DNS-суфікс.
  - **Мережеві карти** (`NetworkCards`): адаптери.
- **`SYSTEM`** (для кожного набору керування):
  - **Інтерфейси TCP/IP:** адреси, DHCP-сервер і оренда, DNS-сервери, домен.
  - **Спільні ресурси**, які надає система (`LanmanServer\Shares`).
  - **Проксі портів** (`PortProxy`): тунелі, які налаштовує `netsh interface portproxy` і які
    зловмисники використовують для півотингу та для доступу до RDP через інший порт.
  - **Постійні маршрути.**
  - **Часовий пояс**, за яким місцевий час профілів перетворюється на UTC.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> система підключалася до мережі з
цією назвою та шлюзом, уперше та востаннє — у цей час. Вона мала цю адресу від цього DHCP-сервера
і надавала ці спільні ресурси та тунелі на момент збереження куща. Сама лише назва мережі не
визначає, де вона була: багато мереж називаються "Network" або мають однаковий SSID.

## Розташування { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\NetworkList\Profiles\{GUID}
      ProfileName, Description, Category, NameType, Managed, DateCreated, DateLastConnected
  Microsoft\Windows NT\CurrentVersion\NetworkList\Signatures\Managed|Unmanaged\<signature>
      ProfileGuid, DefaultGatewayMac, DnsSuffix, FirstNetwork
  Microsoft\Windows NT\CurrentVersion\NetworkCards\<n>          ServiceName, Description
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Services\Tcpip\Parameters\Interfaces\{GUID}                    IP, DHCP, DNS, lease times
  Services\LanmanServer\Shares                                   one value per share
  Services\PortProxy\v4tov4|v4tov6|v6tov4|v6tov6\tcp             listen = connect
  Services\Tcpip\Parameters\PersistentRoutes                     destination,mask,gateway,metric
  Control\TimeZoneInformation
```

Зберіть обидва кущі разом із їхніми журналами транзакцій. Кущ SYSTEM потрібен, щоб перетворити час
профілів на UTC.

## Застереження для аналітика { #analyst-cautions }

- **Час профілів місцевий.** `DateCreated` і `DateLastConnected` записуються в місцевому часі
  системи.
  - `FirstConnectedLocal` і `LastConnectedLocal` зберігають їх у тому вигляді, як їх знайдено.
  - `FirstConnectedUTC` і `LastConnectedUTC` перетворюють їх із часовим поясом куща SYSTEM
    системи (`TimeZone`) і його правилом переходу на літній час.
  - Час, записаний до зміни часового поясу, або з року з іншим правилом переходу на літній
    час (дати в США до 2007 року), може відрізнятися на цю різницю.
  - Без придатного для читання куща SYSTEM і дійсного поточного набору керування наводиться лише місцевий час. Пропуски через літній час і неоднозначні повторювані години не мають єдиної доведеної позначки часу UTC: UTC залишається порожнім, а `Problems` пояснює обмеження. Недійсні зміщення або дати також залишаються неперетвореними.
- **Лише перше та останнє.** Кожен профіль має одне перше та одне останнє підключення, а не кожне
  підключення. Останній запис ключа профілю зазвичай збігається з останнім підключенням.
- **Назви повторюються.** Windows називає некеровані дротові мережі "Network", "Network 2" тощо.
  SSID Wi-Fi не унікальні. Використовуйте `GatewayMac` і `DnsSuffix`, щоб розрізняти мережі, і
  зіставляйте їх із журналами DHCP та інвентаризацією активів. MAC шлюзу не обов'язково є BSSID точки доступу Wi-Fi і сам по собі не встановлює фізичного розташування.
- **Спільні ресурси, проксі та маршрути** — це конфігурація: спільний ресурс або проксі порту можна налаштувати і
  ніколи не використати. Жодне значення не має власного часу; останній запис ключа — це час, коли змінилося
  будь-яке з них.
- **Інтерфейси:** ключ інтерфейсу під GUID без адрес — це адаптер, який не
  налаштовано. З DHCP адреси належать останній оренді. Статична адреса, що залишилася з часів до DHCP,
  не показується як така, що використовується.
- **Набори керування:** за замовчуванням читається набір керування, який Windows використовувала востаннє. `--all-control-sets`
  читає кожен із них, що може показати спільний ресурс або проксі, вилучені з поточного.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry networks <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and SYSTEM hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry networks -d evidence/ -o networks.csv
juntei parse-registry networks -d evidence/ -t jsonl \
  | jq 'select(.Source == "port_proxy" or .Source == "share")'
```

Зведення перелічує мережі, до яких підключалися останніми (спершу найновіші), а також кожен спільний ресурс, проксі порту і
постійний маршрут.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `SourcePath` | Хост і кущ. |
| `ControlSet`, `IsCurrentControlSet` | Для записів SYSTEM: набір керування та чи є він тим, який Windows використовувала востаннє. |
| `Source` | `network_profile`, `network_signature` (підпис, якого немає в жодному профілі), `network_card`, `interface`, `share`, `port_proxy`, `route`. |
| `ProfileName` | Назва профілю (SSID мережі Wi-Fi); опис підпису. |
| `Category`, `ConnectionType` | `public`, `private`, `domain`; `wired`, `wireless`, `vpn`, `mobile_broadband` (інакше — число). |
| `FirstConnectedUTC`, `LastConnectedUTC` | `DateCreated` і `DateLastConnected`, перетворені на UTC з `TimeZone`. |
| `FirstConnectedLocal`, `LastConnectedLocal` | Те саме у знайденому вигляді (місцевий час, з точністю до мілісекунди). |
| `TimeZone` | Часовий пояс, використаний для перетворення. |
| `GatewayMAC`, `DNS-Suffix` | MAC-адреса шлюзу та DNS-суфікс підписів профілю (`|` між кількома значеннями). |
| `InterfaceGUID` | GUID інтерфейсу; `ServiceName` мережевої карти. |
| `IP-Address`, `DhcpServer`, `LeaseObtainedUTC`, `LeaseExpiresUTC` | Адреси інтерфейсу, що використовуються, DHCP-сервер і оренда. |
| `ShareName`, `SharePath` | Спільний ресурс і папка, яку він надає. |
| `ListenAddress`, `ListenPort`, `ConnectAddress`, `ConnectPort` | Проксі порту: що він прослуховує і куди переспрямовує. |
| `Route` | Постійний маршрут у збереженому вигляді: `destination,netmask,gateway,metric`. |
| `Details` | Інші значення у вигляді `Name=Value; …` (CSV) або об'єкта (JSON): опис профілю, `Managed`, перша назва та ключі підписів; адаптер інтерфейсу (з `NetworkCards`), маски, шлюзи, DNS-сервери та домен; тип, примітка та інші поля спільного ресурсу; вид проксі; частини маршруту. |
| `KeyPath`, `ValueName`, `KeyLastWrittenUTC` | Ключ, значення для спільного ресурсу, проксі або маршруту, і останній запис ключа. |
| `DataRaw` | Байти, які не декодуються: час, що не є дійсним `SYSTEMTIME`, спільний ресурс, що не є списком рядків. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Значення неправильного типу або розміру, неможливі дати та назви, що не відповідають своїй формі. |

## Стратегія розслідування { #investigation-strategy }

1. Відсортуйте профілі за `LastConnectedUTC` і перевірте мережі навколо інциденту: невідомі
   мережі Wi-Fi, мобільні точки доступу, VPN.
2. Використовуйте `GatewayMac` і `DnsSuffix`, щоб розрізняти мережі з однаковою назвою і зіставляти
   мережу між хостами.
3. Перевірте кожен проксі порту та спільний ресурс. Проксі порту на 3389 або 445 внутрішнього хоста чи спільний ресурс
   проміжної папки (`C:\ProgramData`, `C:\Users\Public`) — це зачіпка. Зіставляйте з журналами
   подій (створення процесу `netsh`, події спільних ресурсів 5142/5144) і часом ключів.
4. Використовуйте адреси інтерфейсів і час оренди, щоб визначити місце хоста в мережі, і зіставляйте їх
   із журналами брандмауера, DHCP і проксі.
5. Використовуйте `--all-control-sets` і `parse-registry read --recover-deleted` для вилучених спільних ресурсів,
   проксі та профілів.

## Додаткові відомості { #further-details }

- [Microsoft: netsh interface portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001: Proxy: Internal Proxy](https://attack.mitre.org/techniques/T1090/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
