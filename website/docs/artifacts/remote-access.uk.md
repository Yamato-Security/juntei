# Віддалений доступ (з боку користувача) { #remote-access-user-side }

## Що це таке { #what-it-is }

`NTUSER.DAT` користувача зберігає сліди віддалених підключень, які користувач здійснював із комп'ютера.
Це вихідна сторона латерального переміщення:

- **Remote Desktop (`mstsc`):**
  - комп'ютери, введені в клієнті, від найновішого;
  - по одному ключу на кожен комп'ютер, на який входив користувач, з використаним ім'ям облікового запису та відбитком
    сертифіката сервера, який прийняв користувач;
  - комп'ютери, яким користувач дозволив використовувати локальні пристрої.
- **PuTTY:**
  - збережені сеанси: хост, порт, обліковий запис, протокол, файл ключа, проксі та переспрямування портів;
  - останні запущені сеанси;
  - кожен ключ сервера SSH, прийнятий користувачем.
- **WinSCP:** збережені сеанси та прийняті ключі серверів SSH у тому самому вигляді, що й у PuTTY.
- **Підключені диски:** кожен диск, підключений для відновлення під час входу, і шляхи, введені в діалозі
  *Map Network Drive* (підключення мережевого диска).
- **Спільні папки:** спільні папки, змонтовані Explorer (`MountPoints2`).

Вони показують, куди далі переходив обліковий запис і з якими обліковими даними, навіть після того, як журнали
віддаленого комп'ютера зникли.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> профіль цього користувача підключався до цієї цілі або зберіг
підключення до неї. Сам лише збережений сеанс або введене ім'я не доводить, що
підключення було успішним. Прийнятий ключ хоста або ключ `Servers` з `UsernameHint` є вагомішим доказом:
клієнт досяг сервера.

## Розташування { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Terminal Server Client\Default            MRU0 … MRU9
Software\Microsoft\Terminal Server Client\Servers\<host>     UsernameHint, CertHash
Software\Microsoft\Terminal Server Client\LocalDevices       <host> = devices
Software\SimonTatham\PuTTY\Sessions\<session>
Software\SimonTatham\PuTTY\Jumplist                          Recent sessions
Software\SimonTatham\PuTTY\SshHostKeys                       <type>@<port>:<host> = key
Software\Martin Prikryl\WinSCP 2\Sessions\<session>
Software\Martin Prikryl\WinSCP 2\SshHostKeys
Network\<letter>                                             RemotePath, UserName, ProviderName
Software\Microsoft\Windows\CurrentVersion\Explorer\Map Network Drive MRU
Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2\##server#share
```

Збирайте `NTUSER.DAT` разом із журналами транзакцій для кожного профілю. Збирайте також системний `SOFTWARE`:
він дає SID кожного користувача.

## Застереження для аналітика { #analyst-cautions }

- **Час ключа — це час списку.** `Default`, `SshHostKeys` і `Map Network Drive MRU` містять списки,
  а останній запис їхнього ключа — це час останньої зміни списку. Цей час належить першому елементу
  (`MruPosition` 0). Для інших він є лише верхньою межею. Ключ `Servers\<host>`, ключ сеансу,
  `Network\<letter>` або ключ спільної папки має власний час.
- **Збережений не означає використаний.** Сеанс PuTTY або WinSCP можна зберегти, жодного разу не підключившись. PuTTY
  перезаписує `Recent sessions`, коли запускається збережений сеанс, і додає ключ хоста, коли до сервера
  вперше вдається підключитися.
- **Паролі:**
  - WinSCP може зберігати паролі сеансів, тунелів і проксі.
  - PuTTY зберігає лише пароль проксі, у відкритому вигляді.
  - `PasswordStored` і `StoredPasswords` у `Details` вказують, які значення задано. Вони ніколи не
    декодуються і не виводяться.
- **WinSCP не записує значення за замовчуванням.** Сеанс без `FSProtocol` — це SFTP, а сеанс без
  `PortNumber` використовує стандартний порт протоколу. У такому разі `Protocol` і `Port` залишаються порожніми,
  а не заповнюються здогадками.
- **Відбитки ключів хостів** подаються у форматі OpenSSH `SHA256:…`, так, як їх показують `ssh-keygen -l`, `ssh` і PuTTY 0.75
  і новіші. Juntei обчислює їх із ключа, який зберігають PuTTY і WinSCP. Порівнюйте їх із
  власним ключем сервера, файлами `known_hosts` і журналами, щоб ідентифікувати сервер за іменем або IP-адресою.
  Ключ SSH-1 (`rsa`) відбитка не має.
- **Підключені диски:** `RemoteUsername` задається, лише якщо диск було підключено з іншим
  обліковим записом; Windows записує `REG_DWORD` 0, якщо використано власний обліковий запис користувача.
- **Спільні папки:** `MountPoints2` також має ключі для томів і літер дисків. Тут перелічуються лише мережеві спільні папки
  (`##server#share`); `\\server@SSL@443\DavWWWRoot` — це спільна папка WebDAV.
- **Видалення:** очищення списків або використання портативного клієнта не залишає тут слідів. Порівнюйте з
  Jump Lists, Prefetch, файлами `.rdp`, кешем растрових зображень RDP і журналами цілі.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry remote-access <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry remote-access -d evidence/ -o remote.csv
juntei parse-registry remote-access -f NTUSER.DAT -t jsonl \
  | jq 'select(.Client == "rdp" or .HostKeyFingerprint != null)'
```

Підсумок підраховує підключення за клієнтами. Він перелічує підключення, ключі яких було записано останніми,
від найновіших, а також сеанси, що зберігають пароль.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Хост, ім'я папки профілю, SID користувача із системного `ProfileList` (за наявності системного куща `SOFTWARE`) і кущ. |
| `Client` | `rdp`, `putty`, `winscp`, `mapped_drive` або `network_share`. |
| `Kind` | Звідки походить запис: `rdp_mru`, `rdp_server`, `rdp_local_devices`, `putty_session`, `putty_host_key`, `winscp_session`, `winscp_host_key`, `network_drive`, `map_network_drive_mru`, `mount_point`. |
| `Target`, `Port` | Комп'ютер (ім'я хоста або IP-адреса) і порт або спільна папка (`\\server\share`). Порт наводиться лише там, де його записала Windows або клієнт. |
| `RemoteUsername` | Обліковий запис, використаний на цілі: `UsernameHint`, `UserName` сеансу (або `user@` з `HostName` у PuTTY), `UserName` підключеного диска. |
| `Protocol` | `Protocol` у PuTTY (`ssh`, `telnet`, `raw`, …), `FSProtocol` у WinSCP (`sftp`, `scp`, `ftp`, `webdav`, `s3`), постачальник підключеного диска (`Microsoft Windows Network`). |
| `MRU-Position` | Місце у списку (0 — найновіший): `MRU<n>`, `MRUList`, `Recent sessions` у PuTTY. |
| `SessionName` | Назва збереженого сеансу з декодованими `%20` та іншими екранованими послідовностями. |
| `HostKeyType`, `HostKeyFingerprint`, `HostKey` | Тип прийнятого ключа хоста SSH (`rsa2`, `ssh-ed25519`, `ecdsa-sha2-nistp256`, …), його відбиток OpenSSH і його числа в збереженому вигляді. |
| `DriveLetter` | Літера підключеного диска. |
| `PasswordStored` | Чи зберігає сеанс пароль; ніколи не декодується. |
| `Details` | Інші параметри у вигляді `Name=Value; …` (CSV) або об'єкта (JSON): `CertHash` сертифіката RDP, `LocalDevices`, файл ключа сеансу, проксі, тунель SSH, переспрямування портів, віддалена команда та каталоги, `StoredPasswords`, `Label` спільної папки, `ConnectionType` диска. |
| `KeyPath`, `ValueName` | Ключ і значення для запису, який є одним значенням. |
| `KeyLastWrittenUTC` | Останній запис ключа (див. застереження щодо списків). |
| `DataRaw` | Байти значення (hex), якщо воно не є рядком або не декодується. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Значення неправильного типу (з їхніми байтами), ключі хостів, які не вдається прочитати, літери `MRUList`, яких немає, і значення, яких він не перелічує. |

## Стратегія розслідування { #investigation-strategy }

1. Складіть перелік цілей кожного користувача за клієнтами та перевірте їх щодо систем, до яких цей користувач мав би
   підключатися. Шукайте адміністративні спільні ресурси (`C$`, `ADMIN$`) та IP-адреси.
2. Зверніть увагу на використані облікові записи (`RemoteUsername`): адміністратори домену та локальні облікові записи на інших
   комп'ютерах.
3. Зіставте `HostKeyFingerprint` із ключами серверів, а `CertHash` — із сертифікатами
   серверів RDP.
4. Зіставляйте з подіями входу на цілях (4624 типу 10 і 3, 1149), Jump Lists і файлами LNK
   (`mstsc.exe`, `putty.exe`, файли, відкриті зі спільних папок), Prefetch і кешем растрових зображень RDP.
5. Використовуйте `parse-registry read --recover-deleted` для видалених записів, а старіші копії
   `NTUSER.DAT` профілю (Volume Shadow Copies, резервні копії) — для очищених списків.

## Додаткові відомості { #further-details }

- [Microsoft: видалення записів із поля Computer у Remote Desktop Connection](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [PuTTY FAQ: де PuTTY зберігає свої дані?](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP: сховище конфігурації](https://winscp.net/eng/docs/config)
- [libyal winreg-kb: Terminal Server Client](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb: точки монтування](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
