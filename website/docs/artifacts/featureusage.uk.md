# FeatureUsage, RecentApps і закріплені на панелі завдань елементи { #featureusage-recentapps-and-taskbar-pins }

## Що це таке { #what-they-are }

Провідник зберігає в `NTUSER.DAT` користувача кілька записів про те, як користувач використовує програми:

- **FeatureUsage** (Windows 10 і новіші): лічильники для кожної програми за видами використання.
  - `AppSwitched`: скільки разів користувач перемикався на вікно програми (клацання на панелі завдань,
    Alt+Tab).
  - `AppLaunch`: скільки разів програму було запущено з панелі завдань (закріплений елемент).
  - `ShowJumpView`: скільки разів відкривався її список переходів (клацання правою кнопкою на панелі завдань).
  - `AppBadgeUpdated`: скільки разів змінювався її значок-індикатор на панелі завдань (кількість непрочитаних).
  - `TrayButtonClicked`: скільки разів клацали кожну кнопку панелі завдань (`StartButton`, `SearchButton`,
    `TaskViewButton`, `ClockButton`, …).

  Програмою є ідентифікатор моделі користувача програми (application user model ID) (`Microsoft.Windows.Explorer`,
  `Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`), шлях, перша частина якого може бути GUID відомої
  папки (`{1AC14E77-…}\cmd.exe` — це `%windir%\system32\cmd.exe`), або ідентифікатор процесу
  (`*PID00001d74`) для вікон, які Провідник не зміг зіставити з програмою.
- **RecentApps** (Windows 10 до версії 1709): програми, які користувач нещодавно запускав, із лічильником
  запусків і часом останнього запуску; під кожною — файли, які вона відкривала (`RecentItems`).
- **Закріплені на панелі завдань елементи** (`Taskband`): програми, закріплені на панелі завдань, у вигляді елементів оболонки
  (`Favorites`) і у вигляді посилання кожного закріпленого ярлика (`FavoritesResolve`). У Windows 10 і
  новіших версіях для кожного закріпленого елемента вказано, чи закріпила його Windows (`SystemPinned`), чи користувач (`UserPinned`).
- **ProgramsCache** (`StartPage`, `StartPage2`; від Windows XP до 7): кешовані програми меню «Пуск»
  (`ProgramsCache`), а у Windows 7 — закріплені програми меню «Пуск» і панелі завдань
  (`ProgramsCacheSMP`, `ProgramsCacheTBP`).

<span class="evidence-strength">Найобґрунтованіший висновок:</span> цей користувач використовував цю програму
стільки разів у такий спосіб (FeatureUsage), запускав її в цей час (RecentApps) або закріпив її.
Лічильники не мають власного часу; датувати щось можуть лише останній запис ключа та час RecentApps.

## Розташування { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

`KeyCreationTime` ключа `FeatureUsage` — це момент, коли почався підрахунок; його показано в підсумку. Збирайте
`NTUSER.DAT` разом із журналами транзакцій для кожного профілю, а також кущі `SYSTEM` і
`SOFTWARE` системи для імені хоста та SID користувача.

## Застереження для аналітика { #analyst-cautions }

- **Лічильники без часу.** Лічильник FeatureUsage показує, як часто, а не коли. Час останнього запису
  ключа виду (`KeyLastWrittenUTC`) — це найновіша зміна будь-якого з його лічильників.
- **Не кожен запуск.** `AppSwitched` рахує перемикання вікон, а не запуски; програма, що працювала
  без вікна або на яку користувач жодного разу не перемикався, не враховується. `AppLaunch` рахує лише
  запуски з панелі завдань.
- **Пакетні програми та шляхи.** Програму названо так, як Windows згрупувала її вікна: ідентифікатор
  пакетної програми, ідентифікатор, який програма встановила для себе сама (`Chrome`, `VMware.Workstation.vmui`), або
  шлях. `AppPath` подає шлях, у якому GUID відомої папки замінено на шлях папки
  за замовчуванням.
- **Закріплення за замовчуванням.** Нові профілі отримують закріплені за замовчуванням елементи (Провідник, Edge, Store); у Windows 10
  і новіших версіях для них указано `SystemPinned`. Закріплення `UserPinned` — це вибір користувача.
- **Закріплені елементи у двох списках.** `Favorites` і `FavoritesResolve` описують ті самі закріплені елементи в тому самому
  порядку (`Index`), але подаються як окремі записи.
- **Шляхи ProgramsCache** відносні: за папкою (`Programs\Accessories`) ідуть
  ярлики, що в ній містяться (`Notepad.lnk`).
- **Пошкоджені значення.** Лічильник, який не є `REG_DWORD`, час, який не є `REG_QWORD`, закріплений елемент,
  який не вдалося розпізнати, або посилання, яке насправді не є посиланням, зберігає свої байти (`DataRaw`) і вказується в
  `Problems`.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry feature-usage <INPUT> [OPTIONS]

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
juntei parse-registry feature-usage -d evidence/ -o featureusage.csv
juntei parse-registry feature-usage -f NTUSER.DAT -t jsonl \
  | jq 'select(.Source == "AppSwitched") | [.Count, .AppPath // .App]'
```

Підсумок перелічує записи за джерелами, момент початку підрахунку FeatureUsage для кожного куща,
програми, на які найчастіше перемикалися, нещодавні програми й елементи (спершу найновіші) та закріплені на панелі завдань елементи.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | Хост, назва папки профілю, SID користувача з `ProfileList` системи та кущ. |
| `Source` | `AppSwitched`, `AppLaunch`, `ShowJumpView`, `AppBadgeUpdated`, `TrayButtonClicked` (FeatureUsage), `RecentApps`, `RecentItems`, `Taskband` або `ProgramsCache`. |
| `App` | Назва значення FeatureUsage, `AppId` нещодавньої програми або ідентифікатор моделі користувача програми для закріпленого елемента. |
| `AppPath` | `App` у вигляді шляху, у якому GUID відомої папки замінено на її шлях за замовчуванням; порожнє для ідентифікаторів. |
| `Count` | Лічильник FeatureUsage або `LaunchCount` нещодавньої програми. |
| `LastAccessedUTC` | `LastAccessedTime` нещодавньої програми або елемента. |
| `ItemPath` | `Path` нещодавнього елемента (або `AppPath` нещодавньої програми); шлях елемента оболонки закріпленого елемента або кешованої програми (`User Pinned\TaskBar\File Explorer.lnk`, `Applications\<app ID>`); шлях посилання `FavoritesResolve`. |
| `DisplayName` | `DisplayName` нещодавнього елемента. |
| `PinType` | `SystemPinned` або `UserPinned` для закріплення в `Favorites`. |
| `Index` | Місце закріпленого елемента або кешованої програми в її списку. |
| `ValueName`, `KeyPath`, `KeyLastWrittenUTC` | Значення та ключ, з яких походить запис, і останній запис ключа. |
| `DataRaw` | Байти (у шістнадцятковому вигляді), які не вдалося декодувати. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Значення неправильного типу, закріплені елементи або списки, які не вдалося розпізнати, посилання, які насправді не є посиланнями. |

## Стратегія розслідування { #investigation-strategy }

1. Відсортуйте `AppSwitched` за `Count`, щоб побачити програми, у яких користувач працював найбільше, і шукайте
   засоби адміністрування, віддаленого доступу, архівування та передавання файлів, а також шляхи в розташуваннях, доступних користувачу для запису
   (`\Users\`, `Downloads`, `Temp`).
2. Перевірте закріплені елементи: інструмент із `UserPinned` або закріплений елемент, ярлик якого розташовано в незвичному місці.
3. Використовуйте час RecentApps разом з UserAssist, Prefetch, BAM і Jump Lists, щоб датувати використання.
4. Порівняйте `KeyCreationTime` із часом створення профілю: саме тоді починається підрахунок.
5. Використовуйте `parse-registry read --recover-deleted`, щоб знайти видалені лічильники та закріплені елементи.

## Додаткові відомості { #further-details }

- [Microsoft: ідентифікатори моделі користувача програми (Application User Model IDs)](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: кеш програм](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
