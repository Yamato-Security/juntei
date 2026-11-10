# Параметри безпеки { #security-settings }

## Що це таке { #what-they-are }

Зловмисники змінюють параметри безпеки Windows, щоб залишатися непоміченими та отримати доступ до облікових даних. Вони
вимикають Microsoft Defender або виключають із нього свої інструменти, змушують Windows зберігати паролі у відкритому вигляді,
відкривають Remote Desktop, вимикають брандмауер і глушать журналювання PowerShell. Реєстр зберігає
ці параметри:

- **`SOFTWARE`:** Defender, його виключення та політики; UAC; політики Remote Desktop і
  брандмауера; політика виконання PowerShell, політики журналювання та постачальники AMSI; відновлення системи (System Restore);
  канали журналів подій.
- **`SYSTEM`** (для кожного набору керування): `WDigest`, захист LSA, Credential Guard, Restricted Admin,
  NTLM; Remote Desktop; профілі брандмауера; SMBv1; час останнього доступу; розмір кожного журналу подій.
- **`NTUSER.DAT`:** політика виконання та політики журналювання PowerShell користувача, а також AMSI для
  Windows Script Host.

`parse-registry security-settings` повідомляє кожен знайдений параметр як запис. Кожен запис містить
значення параметра, що це значення означає і чи **послаблює воно стан за замовчуванням**: чи вимикає воно
захист або джерело доказів, чи знижує його нижче рівня, який поточна Windows (10 і 11) забезпечує
без цього значення.

<span class="evidence-strength">Найобґрунтованіший висновок:</span> цей параметр мав це значення на момент
збереження куща, а ключ востаннє записувався в цей час. Послаблений параметр — це зачіпка, а не доказ
компрометації. Деякі з цих значень також встановлюють адміністратори, групова політика, інші засоби захисту та старіші версії Windows;
там, де це поширено, про це сказано в описі значення.

## Розташування { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Defender[\Real-Time Protection|\Features|\Spynet]
  Microsoft\Windows Defender\Exclusions\Paths|Extensions|Processes|IpAddresses|TemporaryPaths
  Policies\Microsoft\Windows Defender\…
  Microsoft\Windows\CurrentVersion\Policies\System                 UAC
  Policies\Microsoft\Windows NT\Terminal Services                  RDP policy
  Policies\Microsoft\WindowsFirewall\<profile>                      firewall policy
  Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell             ExecutionPolicy
  Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging|ModuleLogging|Transcription
  Microsoft\AMSI\Providers\{CLSID}
  Microsoft\Windows NT\CurrentVersion\SystemRestore                DisableSR
  Microsoft\Windows\CurrentVersion\WINEVT\Channels\<channel>       Enabled, MaxSize
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Control\SecurityProviders\WDigest                                UseLogonCredential
  Control\Lsa                                                      RunAsPPL, LsaCfgFlags, DisableRestrictedAdmin, LmCompatibilityLevel
  Control\Terminal Server[\WinStations\RDP-Tcp]                    fDenyTSConnections, PortNumber, UserAuthentication
  Services\SharedAccess\Parameters\FirewallPolicy\<profile>        EnableFirewall
  Services\LanmanServer\Parameters                                 SMB1
  Control\FileSystem                                               NtfsDisableLastAccessUpdate
  Services\EventLog\<log>                                          MaxSize
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell    ExecutionPolicy
  Software\Policies\Microsoft\Windows\PowerShell\…
  Software\Microsoft\Windows Script\Settings                       AmsiEnable
```

Збирайте всі три кущі разом із журналами транзакцій. Параметр, змінений незадовго до збирання,
може бути лише в журналах.

## Застереження для аналітика { #analyst-cautions }

- **Збережена конфігурація — це не фактичний стан захисту.** Повідомляються зібрані значення реєстру та діагностичні рядки для пошкоджених списків значень.
  Відсутність даних може відображати значення за замовчуванням, незібрані джерела або інший механізм
  керування; це не доводить, що захист увімкнено чи вимкнено. `WeakensDefault` позначає
  параметри, які можуть знизити вказаний базовий рівень, якщо вони враховуються.
- **Застарілі прапорці Defender можуть ігноруватися.** `DisableAntiSpyware` і `DisableAntivirus` не
  доводять, що Defender зупинено. Їхня дія залежить від версії платформи, підключення (onboarding) і захисту від
  втручання (tamper protection). [Microsoft пояснює ці застарілі параметри](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware).
- **Сучасні керовані виключення можуть бути недоступні в знімках реєстру.** Починаючи з версії платформи
  4.18.25110.6, пристрої, що використовують керування конфігурацією Defender for Endpoint, потребують підтримуваних
  командлетів PowerShell для Defender, щоб отримати конфігурацію виключень. Тому відсутність виключень в офлайн-реєстрі
  не означає, що виключень немає.
  [Рекомендації Microsoft щодо конфігурації](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings).
- **Послаблений не означає шкідливий.** Приклади:
  - `DisableAntiSpyware` = 1 також встановлює сама Windows, коли зареєстровано інший антивірус.
  - Remote Desktop навмисно відкривають на серверах.
  - Windows 7 зберігала менші журнали подій і не оновлювала час останнього доступу.
- **Значення за замовчуванням залежать від версії.** Деякі значення раніше були стандартними:
  - `NtfsDisableLastAccessUpdate` = 1 було значенням за замовчуванням до Windows 10 1803. Воно повідомляється
    без вердикту, але `0x80000001`, установлене вручну в пізнішій версії, вважається послабленням.
  - `RunAsPPL`, `LsaCfgFlags` і `UseLogonCredential` порівнюються з поточною Windows.
- **Виключення Defender:** кожен шлях, розширення, процес або адреса є окремим записом. Кожне виключення
  послаблює Defender. `TemporaryPaths`, які Defender додає сам, не мають вердикту. Шукайте папки, доступні користувачу для запису,
  розширення `.ps1`/`.exe` і LOLBins (`rundll32.exe`, `powershell.exe`).
- **Ключі політик** (`Policy` = True) записують групова політика, інструменти та зловмисники, які
  знають, що політики мають пріоритет над локальними параметрами. Зіставляйте їх з об'єктами групової політики (GPO) домену.
- **Канали журналів подій:** повідомляється лише 25 каналів, важливих для безпеки, із зазначенням того, чи вмикає Windows
  кожен із них за замовчуванням. Наприклад, `TaskScheduler/Operational` і `DNS-Client/Operational`
  за замовчуванням вимкнено. Для інших використовуйте `parse-registry read`.
- **Час ключів:** `KeyLastWrittenUTC` належить ключу, тож це не обов'язково час зміни саме цього
  значення. Запис усього ключа `Policies` за один раз — звичне явище.
- **Набори керування:** за замовчуванням читається набір керування, який Windows використовувала останнім. `--all-control-sets`
  читає всі.

## Аналіз за допомогою Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use
      --weakened-only     Only settings that weaken the default

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry security-settings -d evidence/ -o settings.csv
juntei parse-registry security-settings --weakened-only -d evidence/ -t jsonl \
  | jq -r '[.Host, .Area, .KeyPath, .ValueName, .Value, .Meaning] | @tsv'
```

Підсумок підраховує параметри за областями. Він перелічує параметри, які послаблюють стан за замовчуванням, і кожне
виключення Defender.

## Поля виводу { #output-fields }

| Поле | Інтерпретація |
|---|---|
| `Host`, `SourcePath`, `HiveType` | Хост, кущ і його тип (`SOFTWARE`, `SYSTEM`, `NTUSER.DAT`). |
| `ControlSet`, `IsCurrentControlSet` | Для `SYSTEM`: набір керування і те, чи використовувала його Windows останнім; невідомо, якщо `Select\Current` відсутній або некоректний. |
| `User`, `UserSID` | Для `NTUSER.DAT`: назва папки профілю та SID користувача з `ProfileList` системи. |
| `Area` | `defender`, `uac`, `credentials`, `remote_access`, `network`, `powershell`, `evidence`. |
| `Setting` | Що це за параметр: `defender_disable_realtime_monitoring`, `defender_exclusion_path`, `uac_local_account_token_filter_policy`, `wdigest_use_logon_credential`, `rdp_deny_connections`, `powershell_script_block_logging`, `event_channel_enabled`, … |
| `Value` | Значення: десяткове число, рядок або саме виключення. Порожнє, якщо значення не відповідає типу параметра. |
| `Meaning` | Що означає значення. |
| `WeakensDefault` | `True`, якщо значення послаблює те, що поточна Windows робить без нього, `False`, якщо ні, порожнє, якщо це не застосовується (розмір, порт, постачальник AMSI) або визначити неможливо. |
| `Policy` | Чи розташований ключ у ключі `Policies`. |
| `KeyPath`, `ValueName` | Ключ і значення. Назва значення виключення — це виключений елемент; постачальник AMSI не має назви значення. |
| `KeyLastWrittenUTC` | Останній запис ключа. |
| `DataRaw` | Байти значення (у шістнадцятковому вигляді), якщо вони не відповідають типу параметра. |
| `TransactionLogStatus`, `LogsApplied` | Контекст відновлення реєстру. |
| `Problems` | Значення неправильного типу та ключі, які неможливо прочитати. |

## Стратегія розслідування { #investigation-strategy }

1. Запустіть із `--weakened-only` на кожному хості й порівняйте хости між собою: параметр,
   який має лише один хост, привертає увагу.
2. Перевірте кожне виключення Defender щодо інструментів, знайдених на хості та в журналах Defender
   (подія 5007 фіксує зміни конфігурації).
3. Зіставляйте час ключів із входами в систему, встановленням служб і створенням процесів. Шукайте
   командні рядки `reg.exe`, `Set-MpPreference`, `netsh advfirewall` і `powershell` (подія
   4688, Sysmon 1 і 13, PowerShell 4104).
4. Для параметрів облікових даних (`WDigest`, `RunAsPPL`, `LocalAccountTokenFilterPolicy`,
   `DisableRestrictedAdmin`) шукайте подальше вивантаження облікових даних (credential dumping) і pass-the-hash.
5. Використовуйте `parse-registry read --recover-deleted` для значень, видалених, щоб відновити стан за замовчуванням, і
   `--all-control-sets` для старіших наборів керування.

## Додаткові відомості { #further-details }

- [Microsoft: налаштування та перевірка виключень для Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus)
- [Microsoft: параметри та конфігурація контролю облікових записів користувачів (UAC)](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft: налаштування додаткового захисту LSA](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft: about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001: Impair Defenses: Disable or Modify Tools](https://attack.mitre.org/techniques/T1562/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
