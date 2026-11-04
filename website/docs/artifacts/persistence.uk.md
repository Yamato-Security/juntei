# Автозапуск через реєстр { #registry-autostarts }

## Що це таке { #what-they-are }

Windows запускає програми та завантажує DLL з багатьох розташувань реєстру: під час завантаження системи, під час входу, коли
програма запускається або завершує роботу, коли відкривається `cmd.exe`, коли створюється клас COM або коли спулер,
LSA, netsh чи служба часу завантажує свої плагіни. Зловмисники використовують ці розташування для закріплення,
підвищення привілеїв і впровадження DLL, і легітимне програмне забезпечення також їх використовує. Juntei зчитує їх
із `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` і `UsrClass.dat` в одну таблицю. Він записує по одному запису на
програму, DLL або пакет і позначає кожен відповідною технікою MITRE ATT&CK.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> кущ налаштовував запуск або завантаження цієї програми чи DLL.
Це не доводить, що програма виконувалася. Для її запуску потрібні умови, які визначає
механізм (вхід користувача, запуск програми, що налагоджується, створення класу), а також інші
докази виконання.

## Розташування { #locations }

| Кущ | Що зчитується |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run`, `RunOnce` (+`Setup`), `RunOnceEx`, `RunServices(Once)` і `Policies\Explorer\Run` разом з їхніми копіями `Wow6432Node` і копіями режиму встановлення Terminal Server; `Explorer\StartupApproved\Run`, `Run32` і `StartupFolder`; `Winlogon` (`Shell`, `Userinit`, `Taskman`, `AppSetup`, `GinaDLL`, … і `Notify`); `Windows` `AppInit_DLLs` разом із `LoadAppInit_DLLs`; `Image File Execution Options\*` `Debugger` (разом з `UseFilter`) і `VerifierDlls`; `SilentProcessExit\*` `MonitorProcess`; `Active Setup\Installed Components\*` `StubPath`; `NetSh`; `Command Processor` `AutoRun` |
| `SYSTEM` | `Control\Session Manager` (`BootExecute`, `SetupExecute`, `Execute`, `PendingFileRenameOperations`, `KnownDLLs`, `AppCertDlls`, змінні профілювальника .NET у `Environment`); `Control\Lsa` `Authentication`, `Security` і `Notification Packages` (+`OSConfig`); `SecurityProviders`; монітори, постачальники та обробники друку в `Control\Print`; `Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | ключі `Run` користувача та `StartupApproved`; `Windows NT\CurrentVersion\Windows` `Load` і `Run`; `Winlogon` `Shell`; `Environment` (`UserInitMprLogonScript`, профілювальник .NET); `Control Panel\Desktop` `SCRNSAVE.EXE`; `PackagedAppXDebug`; `Command Processor` `AutoRun`; `Software\Classes` (як `UsrClass.dat`) |
| `UsrClass.dat` | `CLSID\*\InprocServer32` і `LocalServer32` (а також у `Wow6432Node`); `shell\open\command` типів програм (`exefile`, `batfile`, `scrfile`, …) і того, що запускають програми з автоматичним підвищенням привілеїв (`ms-settings`, `mscfile`, `Folder`, …); `ActivatableClasses\…\DebugInformation` |

Збирайте кожен кущ разом із журналами транзакцій. Кущі користувачів розташовані в `Users\<name>\NTUSER.DAT` і
`Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat`.

## Застереження для аналітика { #analyst-cautions }

- **Значення за замовчуванням теж потрапляють у звіт.** Більшість записів у чистій системі — це стандартні значення Windows:
  - `Userinit` має значення `userinit.exe`;
  - `Shell` має значення `explorer.exe`;
  - `KnownDLLs`, пакети LSA, монітори друку, допоміжні модулі netsh і компоненти Active Setup мають стандартні записи.

  Шукайте те, що відрізняється від чистої збірки тієї самої версії.
- **`StartupApproved` охоплює лише список автозавантаження диспетчера завдань.** Він вказує, чи вимкнено там запис `Run`
  або файл папки автозавантаження і коли. Запис без відповідного значення `Run`
  включається до звіту з поясненням «only in StartupApproved». Він може показати автозапуск, який
  було видалено.
- **`OverridesMachineRegistration` потребує системного куща `SOFTWARE`.** Клас COM або
  команда користувача перевіряються щодо нього, і він знаходиться так само, як кущ `SYSTEM` хоста. Перевизначення —
  це шаблон перехоплення COM (COM hijacking, T1546.015). OneDrive та інші встановлення для окремого користувача також
  реєструють власні класи, які нічого не перевизначають.
- **`ImagePath` і `Args` відокремлюються від командного рядка лише там, де це можна визначити.**
  - Програма в лапках відокремлюється за закривальною лапкою.
  - Перше слово без папки вважається програмою (`cmd /c …`).
  - В інших випадках поділ відбувається за найкоротшим початком, що закінчується розширенням програми.

  Шлях без лапок із пробілами може запустити іншу програму (`C:\Program.exe`), якщо така існує.
- **`GlobalFlag` визначає `Enabled` для `VerifierDlls` і `MonitorProcess`.** `VerifierDlls` завантажується
  лише з `0x100`. `MonitorProcess` запускається лише з `0x200` і `0x1` у `ReportingMode`
  ключа `SilentProcessExit\<program>`. Режим `2` (дамп) або `4` (сповіщення) нічого не запускає, а
  `Details` називає прапорці режиму.
- **`KeyLastWrittenUTC` — це останній запис ключа**, а не значення. Кілька значень мають його спільно.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.Mechanism != "com_server" or .OverridesMachineRegistration == true)'
```

Підсумок підраховує записи за механізмами. Він також перелічує записи `StartupApproved\Run` і `Run32`,
що залишилися без свого значення `Run` (файлам папки автозавантаження нічого втрачати), а також класи COM
і команди користувача, які перевизначають машинні.

Як і кожна команда `parse-registry`, вона відхиляє `-o`, що вказує на один із файлів, які вона зчитує, навіть
з `-C`. Це стосується куща, його журналів транзакцій, а також системних кущів SYSTEM і SOFTWARE,
незалежно від того, чи вказано їх шляхом, символьним або жорстким посиланням. Якщо визначити це неможливо, вона нічого не записує.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Контекст хоста, вихідний кущ і його тип. |
| `User`, `UserSID` | Для кущів користувачів: папка профілю та SID. SID береться з назви кореневого ключа `UsrClass.dat`, інакше — із системного `ProfileList`. |
| `Mechanism` | `run`, `run_once`, `winlogon`, `ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `boot_execute`, `lsa_security_package`, `com_server`, `uac_bypass_command`, … |
| `AttackTechnique` | Техніка MITRE ATT&CK, якщо така підходить (`T1547.001`, `T1546.012`, …). |
| `KeyPath`, `ValueName` | Ключ під коренем куща (назви в записаному вигляді) і значення (порожнє: значення за замовчуванням). |
| `Data` | Текст значення або один елемент списку. Порожнє, якщо його не вдається прочитати або якщо запис називає лише `StartupApproved`. |
| `ImagePath`, `Args` | Програма або DLL та аргументи, якщо їх можна розрізнити (див. вище). |
| `Enabled`, `DisabledAtUTC` | Що вказують `StartupApproved`, `LoadAppInit_DLLs`, `GlobalFlag` (разом з `ReportingMode`), `UseFilter`, `IsInstalled`, `Enabled` постачальника часу або `ScreenSaveActive`. Порожнє, якщо нічого не вказано. |
| `Details` | Додатково: що робить відкладене перейменування, назва класу COM, `ReportingMode`, `FilterFullPath`, `ShellComponent` компонента Active Setup, … |
| `OverridesMachineRegistration` | Для класу COM або команди користувача: чи реєструє машина такий самий. Порожнє без системного куща `SOFTWARE`. |
| `WOW64` | У `Wow6432Node`: для 32-бітних програм. |
| `KeyLastWrittenUTC` | Останній запис ключа. |
| `DataRaw` | Байти значення (hex), якщо вони не є текстом або не є дійсним UTF-16. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Неправильні типи значень, дані, які не вдається прочитати, записи `StartupApproved` неправильного розміру. |

## Стратегія розслідування { #investigation-strategy }

1. Почніть із підсумку:
   - класи COM і команди, які перевизначають машинні;
   - записи `StartupApproved` без їхнього значення `Run`;
   - механізми, які рідко мають записи (`ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `uac_bypass_command`, `file_association`, `logon_script`, `dotnet_profiler`).
2. Порівняйте пакети LSA, `BootExecute`, `KnownDLLs`, монітори друку та допоміжні модулі netsh із чистою
   збіркою тієї самої версії Windows.
3. Зіставте `ImagePath` із файловою системою ($MFT, хеші та підписи) і перевірте шляхи профілю
   користувача, `ProgramData` і `Temp`.
4. Зіставляйте `KeyLastWrittenUTC` з подіями реєстру Security і Sysmon, створенням процесів,
   Prefetch і Amcache.
5. Використовуйте `parse-registry read --recover-deleted`, щоб знайти автозапуски, видалені з цих ключів.

## Додаткові відомості { #further-details }

- [MITRE ATT&CK T1547 Boot or Logon Autostart Execution](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 Event Triggered Execution](https://attack.mitre.org/techniques/T1546/)
- [Microsoft: ключі реєстру Run і RunOnce](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
