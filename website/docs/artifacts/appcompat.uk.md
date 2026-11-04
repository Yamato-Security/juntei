# Сумісність програм { #application-compatibility }

## Що зберігає Windows { #what-windows-keeps }

Крім ShimCache (`parse-registry shimcache`) і Amcache (`parse-registry amcache`),
функції сумісності програм Windows зберігають такі дані в
`Microsoft\Windows NT\CurrentVersion\AppCompatFlags`:

| Джерело | Кущ і ключ | Що показує |
|---|---|---|
| `layers` | `SOFTWARE` (і `Wow6432Node`) та `NTUSER.DAT` `Layers` | Програми, налаштовані на роботу з шарами сумісності: `RUNASADMIN` (завжди від імені адміністратора), `WIN7RTM`, `HIGHDPIAWARE`, … |
| `pca_store` | `NTUSER.DAT` `Compatibility Assistant\Store` (Windows 8 і новіші) | Програми, запуск яких користувачем побачив помічник із сумісності програм (Program Compatibility Assistant) |
| `pca_persisted` | `NTUSER.DAT` `Compatibility Assistant\Persisted` (Windows 7) | Те саме у Windows 7 |
| `cit_module` | `SOFTWARE` `CIT\Module\<module>` | Програми, які завантажували відстежуваний модуль, і коли вони востаннє це робили |
| `cit_system` | `SOFTWARE` `CIT\System` (від Windows 7 до ранніх версій Windows 10) | Шляхи до програм і командні рядки в стиснених базах даних CIT |
| `installed_sdb` | `SOFTWARE` `InstalledSDB\{GUID}` | Власні бази даних шимів, установлені за допомогою `sdbinst`: файл, опис, час встановлення |
| `custom_sdb` | `SOFTWARE` `Custom\<program>` | Власні бази даних шимів, застосовані до кожної програми, з часом їх встановлення |

<span class="evidence-strength">Найобґрунтованіший висновок:</span> шари та бази даних шимів — це
конфігурація, яка змінює спосіб роботи програми; записи помічника із сумісності програм і CIT
показують, що програма запускалася (або завантажувала модуль) від імені цього користувача або в цій системі.

Власні бази даних шимів — це техніка закріплення та впровадження коду (MITRE ATT&CK T1546.011): шим
може завантажувати DLL у програму або запускати іншу програму щоразу, коли запускається ця програма.

## Розташування { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

Збирайте `SOFTWARE`, кожен `NTUSER.DAT` разом із їхніми журналами транзакцій, а також файли `.sdb`, на які вказує шлях
бази даних шимів (зазвичай `C:\Windows\AppPatch\Custom\`).

## Застереження для аналітика { #analyst-cautions }

- **Розпакування CIT обмежене.** Значення, які заявляють понад 64 MiB у розпакованому вигляді, зберігаються
  як необроблені байти з діагностичним повідомленням. Повідомляється про некоректне стиснення та невідповідність розмірів.


- **Немає часу запуску для записів помічника із сумісності програм.** Дані `Store` (`SACP`, зберігаються в
  `DataRaw`) мають один час, але у Windows 8.1, 10 і 11 він однаковий для всіх програм куща,
  а також для інших систем: це не час запуску програми. Лише час останнього запису ключа `Store`
  датує найновішу зміну.
- **Програми на знімних носіях** мають назви у форматі `SIGN.MEDIA=<id> <path>`: `Path` — це шлях на
  носії, а `Details` містить ідентифікатор носія.
- **Бази даних CIT** стиснено за допомогою LZNT1, а їхня структура не задокументована публічно: Juntei
  перелічує шляхи до програм і командні рядки, знайдені в кожній із них, по одному разу на базу даних, без лічильників і
  часу. Командний рядок є окремим записом і не прив'язаний до своєї програми. Windows 10 і новіші версії зберігають
  тут небагато даних.
- **Бази даних шимів.** Перевіряйте кожен запис `custom_sdb` і `installed_sdb`: легітимне програмне забезпечення
  рідко їх встановлює. Що саме робить шим, показує сам файл `.sdb`.
- **Шари**, установлені користувачем, застосовуються лише до цього користувача; шари в `SOFTWARE` — до всіх. `RUNASADMIN` змушує
  програму щоразу запитувати підвищення привілеїв.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry app-compat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-compat -d evidence/ -o appcompat.csv
juntei parse-registry app-compat -d evidence/ -t jsonl \
  | jq 'select(.Source == "installed_sdb" or .Source == "custom_sdb")'
```

Підсумок перелічує власні бази даних шимів, програми із шарами та останні використання
модулів CIT.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Хост, файл куща та `SOFTWARE` або `NTUSER.DAT`. |
| `User`, `UserSID` | Назва папки профілю куща користувача та його SID із `ProfileList` системи. |
| `Source`, `Scope` | `layers`, `pca_store`, `pca_persisted`, `cit_module`, `cit_system`, `installed_sdb`, `custom_sdb`; `machine` або `user`. |
| `Path` | Програма (назва значення), шлях або командний рядок у базі даних CIT або програма, до якої застосовується база даних шимів. |
| `Layers` | Шари сумісності в записаному вигляді (`~ RUNASADMIN`). |
| `LastSeenUTC` | Коли програма востаннє завантажувала модуль CIT. |
| `SdbGUID`, `SdbPath`, `SdbInstalledUTC` | GUID, файл і час встановлення бази даних шимів. |
| `Details` | Назва модуля CIT, назва значення бази даних CIT, ідентифікатор знімного носія програми (`SignMedia`), `DatabaseDescription` і `DatabaseType` бази даних шимів. |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName`, `WOW64` | Звідки походить запис, коли ключ востаннє змінювався і чи розташований він у `Wow6432Node`. |
| `DataRaw` | Дані `SACP` значення `Store` або дані, що не відповідають очікуваному (у шістнадцятковому вигляді). |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Дані, які не вдалося декодувати або прочитати. |

## Стратегія розслідування { #investigation-strategy }

1. Перевірте кожну власну базу даних шимів: її файл `.sdb`, час встановлення та програми, до яких вона
   застосовується; зіставляйте з `sdbinst.exe` у Prefetch, Amcache і подіях створення процесів.
2. Шукайте серед записів помічника із сумісності програм програми в папках, доступних користувачу для запису, а також
   програми зі знімних носіїв.
3. Перевіряйте `RUNASADMIN` та інші шари в незвичних програмах.
4. Для визначення часу виконання зіставляйте з ShimCache, Amcache, BAM, Prefetch і UserAssist.

## Додаткові відомості { #further-details }

- [Microsoft: база даних сумісності програм](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011: Application Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft: MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
