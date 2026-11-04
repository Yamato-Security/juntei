# Пристрої USB { #usb-devices }

## Що таке історія USB { #what-the-usb-history-is }

Windows фіксує кожен пристрій, який вона встановлює. Для накопичувачів USB, інших пристроїв USB і портативних
пристроїв (телефонів, камер, медіапрогравачів) кущ `SYSTEM` зберігає по ключу на пристрій із його назвою,
серійним номером і часом, коли Windows його встановила, востаннє зафіксувала його підключення та востаннє зафіксувала від'єднання.
`MountedDevices` фіксує GUID томів і літери дисків, призначені їхнім томам. Кущ
`SOFTWARE` зберігає назви, які Explorer дав портативним пристроям і томам, а також записи ReadyBoost
про серійні номери томів. `NTUSER.DAT` кожного користувача фіксує томи, які цей користувач
монтував.

`parse-registry usb` об'єднує їх в один запис на пристрій і том: який це був пристрій, коли
його було встановлено та підключено, яку літеру диска і який том він мав і які користувачі монтували цей
том.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> пристрій із цим постачальником, продуктом і
серійним номером підключався до цієї системи, уперше й востаннє — у цей час; його том мав цю
літеру диска; ці користувачі його монтували. Записи реєстру не показують, які файли копіювалися.

## Розташування { #location }

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| Властивість пристрою | Поле | Значення (`devpkey.h`) |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate`: коли востаннє встановлювався його драйвер |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate`: коли пристрій було встановлено вперше |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate`: коли його востаннє підключали (Windows 8 і новіші) |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate`: коли його востаннє від'єднували (Windows 8 і новіші) |

Windows 8 і новіші зберігають кожен час як значення за замовчуванням ключа `0064` … `0067`. Windows 7
зберігає його в `00000064\00000000\Data`. Juntei зчитує обидва варіанти.

Багато довідників DFIR позначають `0064` як перше встановлення, а `0065` — як встановлення. Juntei дотримується
наведених вище визначень із `devpkey.h` від Microsoft і називає стовпці відповідно до того, що фіксує кожен час.

Передавайте в `-f` або `-d` кущ `SYSTEM`. Системний кущ `SOFTWARE` і кущі `NTUSER.DAT` його користувачів
знаходяться поруч із ним у тому вигляді, як їх зберігає збирання (`Windows\System32\config\SOFTWARE`,
`Users\<user>\NTUSER.DAT`), або в тому самому каталозі та на один рівень нижче, якщо кущі
експортували разом. Збирайте їх усі разом із журналами транзакцій.

## Як об'єднуються записи { #how-the-records-are-joined }

- **Пристрої:** один запис на ключ пристрою: `usbstor`, `usb` і `wpdbusenum` (`DeviceClass`).
  Накопичувач USB бере ідентифікатори постачальника та продукту з пристрою USB з тим самим
  ідентифікатором контейнера (а за його відсутності — з тим самим серійним номером).
- **Томи:** томи накопичувача USB — це значення `\??\Volume{GUID}` у
  `MountedDevices`, дані яких є шляхом, що називає пристрій. Пристрій із двома томами має два
  записи. Його літери дисків — це значення `\DosDevices\X:` з тими самими даними.
- **Назви та серійні номери:** назви томів беруться з `Windows Portable Devices` і з
  портативних пристроїв, що називають той самий пристрій. Мітки томів і серійні номери беруться з
  ключів `EMDMgmt`, які його називають.
- **Користувачі:** користувачі, чий `MountPoints2` містить GUID тому, разом із часом останнього запису цього ключа.
- **Томи без ключа пристрою** (`volume`): том `MountedDevices`, якого не називає жоден ключ пристрою (том
  внутрішнього диска або видаленого пристрою, ключ `Enum` якого зник), літера диска, з якою жодне значення GUID тому
  не має спільних даних, і том, який є лише в `MountPoints2` користувачів.
- **Пристрої, які називає лише `SOFTWARE`** (`portable_device`, `readyboost`).

`SourceKeys` перелічує кожен ключ і значення, з яких було об'єднано запис, разом із кущем.

## Застереження для аналітика { #analyst-cautions }

- **Літери дисків використовуються повторно.** `MountedDevices` зберігає останній пристрій, якому призначалася кожна літера.
  Літера показується лише для пристрою, дані якого досі відповідають цій літері.
- **Серійні номери можуть бути згенеровані.** Якщо другий символ екземпляра — `&`, Windows
  згенерувала його, оскільки пристрій не повідомляє серійного номера; тоді `Serial` порожнє. Такий
  екземпляр може відрізнятися на іншому комп'ютері.
- **Екземпляри накопичувачів USB закінчуються логічним пристроєм** (`&0`). `Serial` не містить його; `Instance`
  зберігає.
- **Часові позначки залежать від версії Windows.** Windows 7 має лише час встановлення. `LastArrivalUTC` і
  `LastRemovalUTC` — це лише останнє підключення, а не кожне підключення. Журнали подій
  (`Microsoft-Windows-Partition/Diagnostic` 1006, Kernel-PnP, `DriverFrameworks-UserMode`) і
  `setupapi.dev.log` дають більше відомостей.
- **Набори керування:** за замовчуванням зчитується набір керування, який Windows використовувала останнім. `--all-control-sets`
  зчитує всі набори керування; старіший набір усе ще може перелічувати пристрій, якого в поточному вже немає.
- **Час останнього запису ключа не є часом підключення.** Windows перезаписує ключі пристроїв з багатьох
  причин.
- **Видалені пристрої залишають сліди.** Запис тому з постачальником і продуктом, але без ключа пристрою —
  це пристрій, ключ `Enum` якого було видалено (під час видалення програми або засобами очищення), але том якого
  досі зберігається в `MountedDevices`.
- **Серійні номери ReadyBoost** — це серійний номер тому (`VolumeSerials`, так, як його показує `dir`),
  який також фіксують файли LNK і Jump Lists.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.DeviceClass == "usbstor") | {Vendor, Product, Serial, LastArrival, DriveLetters, Users}'
```

Підсумок перелічує накопичувачі USB, які підключалися останніми (від найновіших), з їхніми літерами
дисків, а також кількість томів, що належать накопичувачам USB без ключа пристрою.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host` | Ім'я комп'ютера із `SYSTEM`. |
| `DeviceClass` | `usbstor`, `usb`, `wpdbusenum` або, без ключа пристрою, `volume`, `portable_device` чи `readyboost`. |
| `ControlSet` | Набір керування ключа пристрою. |
| `Vendor`, `Product`, `Revision` | Накопичувач USB: із назви пристрою (`Disk&Ven_…&Prod_…&Rev_…`), а також для пристрою, який називає том або ключ `SOFTWARE`. |
| `Vid`, `PID` | Ідентифікатори постачальника та продукту USB. |
| `Serial`, `Instance` | Серійний номер і назва ключа екземпляра в записаному вигляді. |
| `FriendlyName`, `DeviceDesc`, `Service`, `ContainerID` | Значення ключа пристрою в записаному вигляді (`DeviceDesc` часто має вигляд `@<inf>,%…%;<text>`). `FriendlyName` портативного пристрою зазвичай є міткою або диском його тому. |
| `DeviceFirstInstallUTC`, `DriverLastInstalledUTC`, `LastArrivalUTC`, `LastRemovalUTC` | Часові позначки властивостей пристрою (див. вище). |
| `DriveLetters` | Літери дисків, дані `MountedDevices` яких відповідають тому. |
| `VolumeGUID`, `VolumeData` | GUID тому і те, на чому він розташований згідно з даними `MountedDevices`: шлях до пристрою, підпис диска MBR і зміщення розділу або розділ GPT. |
| `VolumeNames`, `VolumeSerials` | Назви, які Explorer дав пристрою або його тому, мітки ReadyBoost і серійні номери томів (`1234-ABCD`). |
| `User` | Користувачі, чий `MountPoints2` містить том, разом з останнім записом цього ключа (JSON: `User`, `MountPoints2LastWritten`, `SourcePath`). |
| `FoundIn`, `SourceKeys` | Де його знайдено (`Enum`, `MountedDevices`, `Windows Portable Devices`, `EMDMgmt`, `MountPoints2`) і кожен ключ і значення, з яких його об'єднано. |
| `KeyPath`, `KeyLastWrittenUTC` | Ключ екземпляра пристрою та його останній запис. |
| `SourcePath`, `SoftwarePath` | Кущ `SYSTEM`, а також кущ `SOFTWARE`, якщо він щось надав. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру для кущів, з яких об'єднано запис. |
| `Problems` | Ключі, які не вдалося прочитати, і властивості пристрою, що не є значеннями FILETIME (з їхніми байтами). |

## Стратегія розслідування { #investigation-strategy }

1. Відсортуйте накопичувачі USB за `LastArrivalUTC` (за його відсутності — за `DriverLastInstalledUTC`) і відкладіть
   власні пристрої читання та концентратори комп'ютера.
2. Запишіть для кожного пристрою постачальника, продукт, серійний номер, GUID тому, літеру диска та серійний номер
   тому і шукайте їх у файлах LNK, Jump Lists, ShellBags і `$MFT` (шляхи на цій
   літері диска або томі).
3. Перевірте `User`, щоб дізнатися, хто монтував том і коли (останній запис ключа `MountPoints2`).
4. Шукайте записи `volume`, які називають накопичувач USB: пристрої, ключі яких було видалено.
5. Зіставляйте з журналами подій для кожного підключення, а не лише останнього.
6. Використовуйте `parse-registry read --recover-deleted` для видалених ключів пристроїв.

## Додаткові відомості { #further-details }

- [Microsoft: властивості пристроїв (devpkey.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft: підтримка запитів диспетчера монтування](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h` (у заголовках Wine)](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h): номери властивостей пристроїв
- [libyal winreg-kb: накопичувачі USB](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md),
  [змонтовані пристрої](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md) і
  [точки монтування](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

Об'єднання з користувачами використовує ідентичність джерела кожного куща, тому експортовані кущі з однаковим ім'ям профілю залишаються окремими. `SourceKeys` зберігає шляхи кущів користувачів, які надали дані. Пошук супутніх кущів користувачів пропускає символьні посилання, як і рекурсивний пошук кущів. Зіставлення VID/PID вимагає відповідного придатного ідентифікатора контейнера; без нього зіставлення за серійним номером має давати однозначні ідентифікатори. Конфліктні кандидати залишаються невідомими з позначкою проблеми.
