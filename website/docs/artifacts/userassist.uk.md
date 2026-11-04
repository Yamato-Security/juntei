# UserAssist { #userassist }

## Що таке UserAssist { #what-userassist-is }

Windows Explorer підраховує програми та ярлики, які користувач запускає з оболонки Explorer: з
меню «Пуск», панелі завдань, робочого стола та подвійним клацанням у папках. Він зберігає ці лічильники в
`NTUSER.DAT` користувача. Кожен елемент має:

- кількість запусків;
- у Windows 7 і новіших — скільки разів і як довго (у мілісекундах) він перебував у фокусі;
- час останнього запуску.

Назви значень закодовано за допомогою ROT13. Ім'я програми зазвичай починається з GUID відомої папки
(`{1AC14E77-…}` — це `System32`).

UserAssist показує, що конкретний користувач запускав інтерактивно, тоді як більшість інших доказів виконання
не пов'язує виконання з користувачем.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> користувач запускав цю програму або ярлик
через Explorer, стільки разів, востаннє — у цей час. Програми, запущені іншим способом (з командного
рядка, як служба, заплановане завдання, з іншої програми), не враховуються.

## Розташування { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | Містить |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | Програми (`executable`), Windows 7 і новіші |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | Ярлики (`shortcut`), Windows 7 і новіші |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | Програми та ярлики (`active_desktop`), від Windows 2000 до Vista |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | Панель інструментів Інтернету (`internet_toolbar`), від Windows 2000 до Vista |

Інші GUID позначаються як `other`. Збирайте `NTUSER.DAT` разом із журналами транзакцій для кожного
профілю, а також системний кущ `SOFTWARE`, який дає SID користувача.

## Застереження для аналітика { #analyst-cautions }

- **Нові профілі заповнюються початковими елементами.** У Windows 7 і новіших новий профіль починається зі стандартних елементів
  (Getting Started, Calculator, Paint, Sticky Notes, …). Їхні лічильники запусків відраховуються від 14,
  і всі вони мають спільний час останнього запуску, приблизно в момент створення профілю. Це не
  активність користувача.
- **Версія 3 рахує від 5** (від Windows 2000 до Vista). Збережений лічильник починається з 5, тому від нього
  віднімається 5. Про збережений лічильник, менший за 5, повідомляється в `Problems`.
- **Шляхи використовують стандартний шлях відомої папки** (`%windir%\system32`, `%ProgramFiles%`,
  `%APPDATA%\…`). Папка, перенаправлена в інше місце, однаково показує свій стандартний шлях. Ідентифікатори програм
  (`Microsoft.Windows.Explorer`, `Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`) та елементи панелі
  керування (`::{…}`) не мають шляху.
- **Відстеження можна вимкнути** за допомогою `Explorer\Advanced` `Start_TrackProgs` = 0,
  `Policies\Explorer` `NoInstrumentation` = 1 або `UserAssist\Settings` `NoLog` = 1. Підсумок
  перелічує кущі, у яких його вимкнено. Порожній або відсутній UserAssist в активному профілі може свідчити
  про протидію криміналістичному аналізу (anti-forensics).
- **`NoEncrypt` = 1** змушує Windows записувати імена без ROT13. Такі імена декодуються неправильно, тому
  кожен елемент цього куща вказує на це в `Problems` (і в stderr, якщо записи виводяться в stdout).
- **Час останнього запуску та лічильники зберігаються для кожного елемента,** але час останнього запису ключа
  (`KeyLastWrittenUTC`) — це лише час найновішого оновлення будь-якого елемента.
- **Лічильники запусків можуть дорівнювати 0** для елементів, які лише перебували у фокусі. Лічильники та час фокусу існують лише
  у версії 5 (Windows 7 і новіші).

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry user-assist <INPUT> [OPTIONS]

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
juntei parse-registry user-assist -d evidence/ -o userassist.csv
juntei parse-registry user-assist -f NTUSER.DAT -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

Підсумок перелічує останні запущені елементи (від найновіших) з їхніми користувачами, а також кущі, у яких
UserAssist вимкнено.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Хост, ім'я папки профілю, SID користувача із системного `ProfileList` (за наявності системного куща `SOFTWARE`) і кущ. |
| `GUID`, `Kind` | Ключ `{GUID}` і те, що він містить (`executable`, `shortcut`, `active_desktop`, `internet_toolbar`, `other`). |
| `NameRot13`, `Name` | Назва значення в збереженому та декодованому вигляді. |
| `Path` | Декодована назва у вигляді шляху, у якому GUID відомої папки замінено її стандартним шляхом; порожнє для ідентифікаторів програм і елементів оболонки. |
| `RunCount` | Скільки разів елемент запускали з Explorer (версія 3: збережений лічильник мінус 5). |
| `FocusCount`, `FocusTimeMs` | Скільки разів елемент перебував у фокусі і як довго в мілісекундах (версія 5). |
| `LastRunUTC` | Час останнього запуску. Порожнє, якщо Windows зберегла 0 або якщо збережений час виходить за межі 9999 року (тоді `Problems` наводить його, а `DataRaw` зберігає байти). |
| `Format`, `SessionID` | Структура даних (5: 72 байти, 3: 16 байтів) і номер сеансу, у якому їх востаннє записано. |
| `KeyLastWrittenUTC` | Останній запис ключа `Count`: найновіше оновлення будь-якого з його елементів. |
| `DataRaw` | Байти значення (hex), якщо їх не вдається декодувати. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Дані невідомого розміру, розмір, що не відповідає `Version` ключа, лічильник версії 3, менший за 5, останній запуск після 9999 року, `NoEncrypt`. |

## Стратегія розслідування { #investigation-strategy }

1. Відсортуйте за `LastRunUTC` і відкладіть початкові елементи: вони мають спільний час, приблизно в момент
   створення профілю.
2. Шукайте програми в розташуваннях, доступних користувачу для запису (`%USERPROFILE%`, `Downloads`, `Temp`,
   `ProgramData`), на знімних дисках і мережевих шляхах, а також засоби адміністрування та віддаленого
   доступу.
3. Зіставляйте з Prefetch (виконання з часом запусків), Amcache і ShimCache (наявність),
   Jump Lists і файлами LNK (що з їх допомогою відкривали), а також `$MFT` для файлів програм.
4. Перевірте кущі, у яких відстеження вимкнено, і профілі з порожнім UserAssist на ознаки очищення.
5. Використовуйте `parse-registry read --recover-deleted` для видалених значень UserAssist.

## Додаткові відомості { #further-details }

- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: ключі User Assist](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
