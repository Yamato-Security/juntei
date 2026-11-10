# BAM / DAM { #bam-and-dam }

## Що таке BAM і DAM { #what-bam-and-dam-are }

Background Activity Moderator (BAM) і Desktop Activity Moderator (DAM) — це служби Windows
(Windows 10 1709 і новіші), які керують тим, що програмам дозволено робити у фоновому режимі.
Вони зберігають для кожного користувача час останнього запуску кожної програми в кущі `SYSTEM`:

- по одному ключу на користувача, названому за SID користувача;
- по одному значенню на програму: його назва — це шлях до програми на її пристрої
  (`\Device\HarddiskVolume3\Windows\System32\cmd.exe`) або ідентифікатор пакетної програми
  (`Microsoft.WindowsCalculator_8wekyb3d8bbwe`);
- перші 8 байтів даних значення — це час останнього запуску (FILETIME).

BAM пов'язує програму з користувачем і часом без власного куща користувача, а також перелічує програми, запущені
зі знімних і мережевих пристроїв.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> процес цієї програми виконувався в сеансі цього
користувача; востаннє вона запускалася в цей час. BAM не підраховує запуски, і запис може залишатися після
видалення програми.

## Розташування { #location }

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

Ключ кожного користувача також має значення `Version` і `SequenceNumber`; вони не є програмами і
не включаються. Система, оновлена з 1709 або 1803, може мати обидві структури.

Збирайте `SYSTEM` разом із журналами транзакцій, а також системні `SAM` і `SOFTWARE`: вони містять імена
користувачів.

## Застереження для аналітика { #analyst-cautions }

- **Один час на програму та користувача.** Зберігається лише найновіший запуск. Prefetch, Amcache і
  журнали подій дають відомості про попередні запуски.
- **Нещодавня активність.** Повідомляється, що записи програм, які деякий час не запускалися,
  видаляються під час запуску Windows, тому BAM переважно показує нещодавню активність: програма, якої в ньому немає,
  все одно могла запускатися давно.
- **Шляхи до пристроїв, а не літери дисків.** `\Device\HarddiskVolume3` — це третій том у нумерації Windows,
  а не `C:`. `DevicePath` і `Path` відокремлюють пристрій від шляху на ньому; зіставте
  том із `MountedDevices`, `$MFT` або іншими артефактами, перш ніж записувати літеру диска.
- **Не кожна програма.** BAM і DAM відстежують програми в сеансах користувачів; служби здебільшого не
  з'являються. Записи `S-1-5-18` зазвичай є процесами встановлення та екрана входу
  (`FirstLogonAnim.exe`), а записи `S-1-5-90-0-<n>` — Desktop Window Manager (`DWM-<n>`).
- **Набори керування.** За замовчуванням зчитується набір керування, який Windows використовувала останнім. `--all-control-sets`
  зчитує всі: старіший набір керування може зберігати запис, якого в поточному вже немає.
- **Імена користувачів.** Ім'я локального облікового запису береться із `SAM` (SID комп'ютера та RID облікового
  запису); ім'я доменного облікового запису — з папки його профілю в `ProfileList`. `UserSource`
  вказує джерело (`sam`, `profile_list`, `well_known`). Без них наводиться лише SID. RID,
  якого вже немає в `SAM` (видалений обліковий запис), залишається без імені.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Підсумок перелічує кожного користувача з його SID, іменем і кількістю записів, а також останні запущені програми
(від найновіших).

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `SourcePath` | Хост і кущ `SYSTEM`. |
| `ControlSet`, `IsCurrentControlSet` | Набір керування, у якому міститься запис, і чи саме його Windows використовувала останнім. |
| `Moderator` | `bam` або `dam`. |
| `UserSID`, `User`, `UserSource` | SID, за яким названо ключ, ім'я користувача та його джерело (`sam`, `profile_list`, `well_known`). |
| `Program` | Назва значення в тому вигляді, як її записала Windows: шлях до пристрою або ідентифікатор пакетної програми. |
| `DevicePath`, `Path` | Для шляху до пристрою — пристрій (`\Device\HarddiskVolume3`) і шлях на ньому (`\Windows\System32\cmd.exe`); порожнє для пакетної програми. |
| `LastRunUTC` | Час останнього запуску програми (з точністю до 100 нс). |
| `KeyPath`, `KeyLastWrittenUTC` | Ключ користувача та його останній запис: найновіше оновлення будь-якого з його записів. |
| `DataRaw` | Байти значення (hex), якщо вони не містять часу: не `REG_BINARY` або менше ніж 8 байтів. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Дані, які не містять часу, і значення, які не вдається прочитати. |

## Стратегія розслідування { #investigation-strategy }

1. Відсортуйте за `LastRunUTC` і перегляньте програми в розташуваннях, доступних користувачу для запису (`\Users\`, `Downloads`,
   `Temp`, `ProgramData`), на пристроях, відмінних від системного тому, а також засоби адміністрування та
   віддаленого доступу.
2. Перевірте, який користувач їх запускав, і облікові записи, що не відповідають жодному поточному профілю.
3. Зіставляйте з Prefetch (кількість запусків і попередні запуски), UserAssist (запуски з Explorer), Amcache
   і ShimCache (наявність), `$MFT` (файл програми) і журналами подій.
4. Використовуйте `--all-control-sets`, а також `parse-registry read --recover-deleted` для видалених значень BAM.

## Додаткові відомості { #further-details }

- [libyal winreg-kb: Background activity moderator (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft: ідентифікатори безпеки (загальновідомі SID)](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

FILETIME останнього запуску поза допустимим діапазоном залишають `LastRunUTC` порожнім і зберігають вихідне значення в `DataRaw` з поясненням у `Problems`. Підсумок нещодавньої активності ідентифікує кожен хост і розмежовує окремі спостереження за кущами/користувачами.
