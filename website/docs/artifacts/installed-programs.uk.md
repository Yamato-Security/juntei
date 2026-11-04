# Встановлені програми { #installed-programs }

## Що зберігає Windows { #what-windows-keeps }

Windows та інсталятори записують встановлене програмне забезпечення в кількох списках реєстру. Кожен список має один ключ
на програму, продукт, дистрибутив або пакет; Juntei формує один запис на ключ.

| Джерело | Кущ і ключ | Що містить |
|---|---|---|
| `uninstall` | `SOFTWARE` і `NTUSER.DAT` `…\CurrentVersion\Uninstall\*`, а також `SOFTWARE\Wow6432Node\…` | Те, що перелічують «Параметри» та *Програми та засоби*: назва, версія, видавець, дата встановлення, папки, команди видалення, розмір, чи приховано запис |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`, `NTUSER.DAT` `Software\Microsoft\Installer\Products\*` | Продукти Windows Installer: назва та версія продукту, ім'я файлу пакета та папка, з якої його встановлено |
| `app_path` | `SOFTWARE` і `NTUSER.DAT` `…\CurrentVersion\App Paths\*` | Програми, які Windows запускає за назвою (`Win+R`, `start tool`): шлях до програми та її папка |
| `wsl` | `NTUSER.DAT` `…\CurrentVersion\Lxss\{GUID}` | Дистрибутиви підсистеми Windows для Linux (WSL): назва, де розташовані її файли або диск, WSL 1 чи 2 |
| `package` | `SOFTWARE` `…\Appx\AppxAllUserStore` | Пакетні програми (Store): підготовлені для всіх користувачів, власні програми Windows, проміжні (staged), скасовані (deprovisioned) та програми кожного користувача |

<span class="evidence-strength">Найобґрунтованіший висновок:</span> цю програму, продукт або пакет було
встановлено (або зареєстровано) в системі для всіх користувачів або для цього користувача. Дата встановлення — це
те, що записав інсталятор; останній запис ключа — момент останньої зміни запису.

## Розташування { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

Збирайте `SOFTWARE`, кожен `NTUSER.DAT` та їхні журнали транзакцій. `SYSTEM` містить назву хоста, а
`ProfileList` у `SOFTWARE` дає SID кожного користувача.

## Застереження для аналітика { #analyst-cautions }

- **Дата встановлення та час ключа.** `InstallDate` — це дата (без часу), записана інсталятором, у
  місцевому часі; деякі інсталятори її не записують, а деякі записують в іншому форматі (зберігається в записаному вигляді в
  `InstallDateRaw`). Оновлення може перезаписати запис, не змінивши дати, або змінити дату,
  майже не змінивши програми: порівнюйте `InstallDate` з `KeyLastWrittenUTC`.
- **Видалене програмне забезпечення.** Програма, яку коректно видалено, втрачає свої ключі. Використовуйте `parse-registry read
  --recover-deleted` для видалених ключів `Uninstall`, а також Amcache, Prefetch і `$MFT` для слідів,
  що залишилися.
- **Реєстрації для окремих користувачів.** Записи `NTUSER.DAT` визначають програмне забезпечення, зареєстроване для профілю.
  Ця область не встановлює, хто запускав інсталятор і чи використовувалися права адміністратора.
- **Приховані записи.** `SystemComponent` приховує запис у *Програми та засоби*; записи
  без `DisplayName` (оновлення, компоненти) також не відображаються. Усі вони наводяться у виводі.
- **Коди продуктів.** Ключ `Uninstall` у формі GUID можна зіставити з упакованим ключем
  `msi_product` через `ProductCode`. Сама лише назва у формі GUID не доводить, що
  запис створив Windows Installer.
- **App Paths.** Це реєстрації, які використовуються для пошуку програм, а не записи
  про виконання. Значення `Path` може містити кілька каталогів пошуку і не обов'язково є
  каталогом встановлення.
- **Видавець.** Це метадані реєстру, надані інсталятором, а не перевірені відомості
  про підпис.
- **Архітектура.** `Wow6432Node` містить 32-бітні програми в 64-бітній Windows (`Architecture` `x86`,
  `Wow64` `True`); інші записи мають власну архітектуру системи (порожнє значення). Архітектура пакета
  є його власною (`x64`, `neutral`, …).
- **WSL.** `Version` — це версія WSL з `Flags` дистрибутива (`0x8`: він працює у віртуальній машині
  WSL 2). Власне значення `Version` дистрибутива (у `Details`) — це версія його файлової системи,
  яка дорівнює 2 і в дистрибутивах WSL 1. `BasePath` містить його `ext4.vhdx` (WSL 2) або `rootfs`
  (WSL 1): зберіть його, щоб дослідити вміст.
- **Пакети.** Списки `<SID>` — це пакети, зареєстровані для кожного користувача; більшість із них — власні пакети
  Windows. Пакети `Deprovisioned` було видалено для нових користувачів, пакети `EndOfLife\<SID>` — для цього користувача.
- **Пошкоджені значення.** Некоректні числа, неправильні типи значень і некоректний UTF-16 вказуються в
  `Problems` і зберігаються в `DataRaw`, зокрема порожній шістнадцятковий рядок для наявного значення з
  нульовою кількістю байтів. Чистий порожній рядок у необов'язковому числовому полі вважається відсутнім значенням. Версія
  пакета має містити чотири десяткові компоненти, кожен у межах діапазону 16-бітного числа без знака.
  Попередження на рівні куща виводяться в stderr, коли `-t` передає записи потоком у stdout.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

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
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.Scope == "user" and .Source != "package")'
```

Підсумок перелічує програми, встановлені останніми (за `InstallDate`), те, що користувачі встановили
для себе, і дистрибутиви WSL.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Хост, файл куща та `SOFTWARE` або `NTUSER.DAT`. |
| `User`, `UserSID` | Користувач куща користувача (його папка профілю та SID з `ProfileList`) або користувач, для якого вказано пакет. |
| `Source`, `Scope` | Список (`uninstall`, `msi_product`, `app_path`, `wsl`, `package`) і `machine` або `user`. |
| `Name`, `Version`, `Publisher` | `DisplayName`, `DisplayVersion`, `Publisher`; `ProductName` і `Version` продукту; ім'я файлу програми з App Path; назва дистрибутива та версія WSL; назва та версія пакета. |
| `InstallDate`, `InstallDateRaw` | `InstallDate` як дата, якщо записано у форматі `YYYYMMDD`; інакше — у записаному вигляді. |
| `KeyLastWrittenUTC` | Коли ключ запису востаннє змінювався. |
| `InstallLocation`, `InstallSource` | `InstallLocation` (`Path` з App Path, `BasePath` дистрибутива, `Path` пакета); `InstallSource` (остання використана папка-джерело продукту). |
| `Program` | Програма з App Path. |
| `UninstallCommand`, `QuietUninstallCommand` | `UninstallString`, `QuietUninstallString`. |
| `SizeKB`, `SystemComponent`, `WindowsInstaller` | `EstimatedSize` у KiB; чи приховано запис; чи встановив його Windows Installer. |
| `ProductCode` | Код продукту Windows Installer (`{GUID}`). |
| `Architecture`, `WOW64` | `x86` у `Wow6432Node` або архітектура пакета; чи розташований ключ у `Wow6432Node`. |
| `Details` | Додатково: `DisplayIcon`, `ModifyPath`, `ParentKeyName`, `ReleaseType`, `Inno Setup: User` (обліковий запис, який запускав інсталятор Inno Setup); `PackageCode`, `PackageName` продукту; `Flags`, `State`, `DefaultUid`, `Version`, `PackageFamilyName`, `Default` дистрибутива; `List`, `ResourceId`, `PublisherId` пакета. |
| `KeyPath` | Ключ запису. |
| `DataRaw` | Байти (у шістнадцятковому вигляді) значень, які не відповідають тому, що вказують їхні назви. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Значення, які не вдалося прочитати або які не відповідають тому, що вказують їхні назви. |

## Стратегія розслідування { #investigation-strategy }

1. Відсортуйте за `InstallDate` і `KeyLastWrittenUTC` навколо часу інциденту; перевірте засоби віддаленого доступу,
   тунелювання та клієнти передавання файлів, архіватори, середовища виконання скриптів і засоби безпеки.
2. Перегляньте записи зі `Scope` `user`: програмне забезпечення, встановлене без прав адміністратора.
3. Перевірте `InstallLocation`, `InstallSource` і `UninstallCommand` на наявність папок, доступних користувачу для запису
   (`Downloads`, `AppData`, `Temp`, `ProgramData`).
4. Шукайте дистрибутиви WSL і збирайте їхні `BasePath`.
5. Зіставляйте з Amcache і Prefetch (запуск інсталятора та програми), `$MFT` (створення
   папки встановлення) і журналом подій Application (події MsiInstaller).

## Додаткові відомості { #further-details }

- [Microsoft: ключ реєстру Uninstall](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key)
- [Microsoft: реєстрація програм (App Paths)](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft: огляд ідентичності пакетів](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
