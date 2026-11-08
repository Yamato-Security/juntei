# Збирання доказів { #collecting-evidence }

Juntei аналізує файли, уже зібрані з Windows. Використовуйте надійний засіб збору та за можливості зберігайте початкові метадані. Образи дисків VHDX можна аналізувати без розпакування: див. [Монтування образів дисків](../commands/mount.md).

## Поширені розташування { #common-locations }

| Артефакт | Типове розташування |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | Файл метаданих тому NTFS `C:\$MFT` |
| `$LogFile` | Файл метаданих тому NTFS `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| Журнал USN | Потік метаданих NTFS `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| Журналювання доступу користувачів (Windows Server) | `C:\Windows\System32\LogFiles\Sum\` |
| Internet Explorer і застарілий Edge (WebCache) | `%LOCALAPPDATA%\Microsoft\Windows\WebCache\` |
| Веб-браузери | `%LOCALAPPDATA%\Google\Chrome\User Data\`, `%LOCALAPPDATA%\Microsoft\Edge\User Data\`, `%APPDATA%\Mozilla\Firefox\Profiles\` та папки профілів інших браузерів |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` та інші розташування користувача/системи |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` і `CustomDestinations` |
| ShimCache | Кущ `SYSTEM`: `...\Session Manager\AppCompatCache\AppCompatCache` |
| Карантин Defender | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| Реєстр машини | `C:\Windows\System32\config\SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` |
| Реєстр користувача | `%USERPROFILE%\NTUSER.DAT`, `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## Файли, які потрібно збирати разом { #files-that-must-travel-together }

- Збирайте кущі реєстру разом із розташованими поруч файлами `.LOG`, `.LOG1` і `.LOG2`. Juntei може відтворювати журнали
  для команд реєстру та ShimCache, коли кущ «брудний».
- Збирайте `SRUDB.dat` для SRUM в узгодженому стані. Наразі Juntei повідомляє про «брудну» базу даних ESE, але
  не відтворює `SRU*.log`; записи, що існують лише в цих журналах, недоступні.
- За можливості збирайте `$MFT` разом із `$Extend/$J`. Запис USN зберігає посилання на батьківські файли,
  а не повний шлях; MFT покращує розв'язання шляхів.
- Збирайте `$LogFile` разом із `$MFT` того самого тому якомога ближче в часі. `$LogFile` швидко
  перезаписується по колу; при пізнішому збиранні попередні записи redo/undo можуть бути повністю втрачені.
- Збирайте SRUM разом із відповідним кущем `SOFTWARE`, щоб розв'язувати імена облікових записів і профілів Wi-Fi.
- Збирайте весь каталог `Sum` журналювання доступу користувачів: `SystemIdentity.mdb` містить назви ролей
  і перелічує щорічні бази даних `{GUID}.mdb`. Як і у випадку SRUM, Juntei не відтворює журнали транзакцій
  UAL `Svc*.log`.
- Збирайте бази даних браузерів разом із файлами `-wal` поруч із ними (`places.sqlite-wal`,
  `cookies.sqlite-wal`): найновіші зміни, а також старіші версії сторінок, з яких відновлюються видалені
  записи, зберігаються там лише до контрольної точки браузера.
- Збирайте всю папку `WebCache`: Windows тримає `WebCacheV01.dat` відкритим, поки користувач у системі,
  а його найновіші зміни є лише в журналах транзакцій `V01*.log`, які Juntei не відтворює.
- Збирайте все дерево `Quarantine` Defender, щоб можна було зіставити Entries, ResourceData і Resources.

## Застереження щодо працюючих систем { #live-system-cautions }

Відкриті файли можуть змінюватися під час копіювання. Кущі реєстру та бази даних ESE можуть бути внутрішньо «брудними».
ShimCache може не скидати свої найновіші записи з пам'яті до завершення роботи. Криміналістичний інструмент збирання,
який використовує знімки томів або сирий доступ, кращий за звичайне копіювання файлів.

## Безпека { #safety }

Докази можуть містити шкідливе програмне забезпечення, зразки експлойтів, шкідливі файли LNK і текст із формулами електронних таблиць.
Аналізуйте на ізольованій системі, не запускайте відновлені потоки і використовуйте `--extract` лише тоді, коли
маєте намір працювати з небезпечним вмістом. Зберігайте незмінене джерело окремо.
