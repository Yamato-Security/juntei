# 安全性設定 { #security-settings }

## 這些是什麼 { #what-they-are }

攻擊者會變更 Windows 的安全性設定，以隱藏自身並取得認證資訊。他們會
關閉 Microsoft Defender 或將自己的工具排除在掃描之外、讓 Windows 保存明文密碼、
開放遠端桌面、關閉防火牆，以及讓 PowerShell 記錄失效。登錄保存了
這些設定：

- **`SOFTWARE`：**Defender 及其排除項目與原則；UAC；遠端桌面與防火牆的
  原則；PowerShell 的執行原則、記錄原則與 AMSI 提供者；系統還原；
  事件記錄通道。
- **`SYSTEM`**（依控制集）：`WDigest`、LSA 保護、Credential Guard、Restricted Admin、
  NTLM；遠端桌面；防火牆設定檔；SMBv1；上次存取時間；各事件記錄的大小。
- **`NTUSER.DAT`：**使用者的 PowerShell 執行原則與記錄原則，以及 Windows Script Host 的
  AMSI。

`parse-registry security-settings` 會將找到的每個設定回報為一筆記錄。每筆記錄包含
設定的值、該值的意義，以及它是否**削弱了預設值**：也就是它是否關閉了某項
保護或證據來源，或將其降低到低於目前的 Windows（10 與 11）在沒有此值時的
行為。

<span class="evidence-strength">最佳結論：</span>在儲存登錄區時，此設定具有此值，
且該機碼最後一次寫入是在此時間。被削弱的設定是線索，而非遭入侵的
證明。系統管理員、群組原則、其他資安產品以及較舊的 Windows 版本也會設定
其中某些值；在這種情況常見時，意義欄位會加以說明。

## 位置 { #location }

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

請收集這三個登錄區及其交易記錄。在收集前不久才變更的設定
可能只存在於交易記錄中。

## 分析注意事項 { #analyst-cautions }

- **儲存的設定並非實際生效的保護狀態。**會回報所收集的登錄值，以及損毀之值清單的診斷資料列。
  缺少的資料可能反映的是預設值、未收集的來源或其他管理
  機制；它並不能證明保護已啟用或已停用。`WeakensDefault` 標示的是
  在被遵循時可能使保護低於所述基準的設定。
- **舊式的 Defender 旗標可能被忽略。**`DisableAntiSpyware` 與 `DisableAntivirus` 並不能
  證明 Defender 已停止。是否適用取決於平台版本、上線（onboarding）狀態與竄改
  防護。[Microsoft 說明了這些舊式設定](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware)。
- **新式受管理的排除項目可能無法從登錄擷取中取得。**從平台
  4.18.25110.6 開始，使用 Defender for Endpoint 設定管理的裝置，必須透過受支援的
  Defender PowerShell Cmdlet 才能取得排除項目設定。因此，離線登錄中
  不存在排除項目，並不能確定實際上沒有排除項目。
  [Microsoft 設定指引](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings)。
- **被削弱不等於惡意。**例如：
  - 當另一套防毒軟體已註冊時，Windows 也會設定 `DisableAntiSpyware` = 1。
  - 伺服器上會刻意開放遠端桌面。
  - Windows 7 保存的事件記錄較小，且不會更新上次存取時間。
- **預設值取決於版本。**有些值在過去是預設值：
  - `NtfsDisableLastAccessUpdate` = 1 在 Windows 10 1803 之前是預設值。它會被回報
    但不附判定；然而在較新版本上手動設定的 `0x80000001` 會被判定為削弱。
  - `RunAsPPL`、`LsaCfgFlags` 與 `UseLogonCredential` 是與目前的 Windows 比較。
- **Defender 排除項目：**每個路徑、副檔名、處理程序或位址都是一筆記錄。每個排除項目
  都會削弱 Defender。由 Defender 自行加入的 `TemporaryPaths` 不附判定。請留意使用者可寫入的資料夾、
  `.ps1`／`.exe` 副檔名以及 LOLBin（`rundll32.exe`、`powershell.exe`）。
- **原則機碼**（`Policy` = True）會由群組原則、工具，以及知道
  原則優先於本機設定的攻擊者寫入。請與網域的 GPO 交叉比對。
- **事件記錄通道：**只會回報 25 個與資安相關的通道，並標示 Windows
  是否預設啟用各通道。例如 `TaskScheduler/Operational` 與 `DNS-Client/Operational`
  預設為關閉。其他通道請使用 `parse-registry read`。
- **機碼時間：**`KeyLastWrittenUTC` 是機碼的時間，因此不一定是此值
  變更的時間。整個 `Policies` 機碼一次寫入的情況很常見。
- **控制集：**預設會讀取 Windows 最後使用的控制集。`--all-control-sets`
  會讀取所有控制集。

## 使用 Juntei 剖析 { #parse-with-juntei }

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

摘要會依領域計算設定數量。它會列出削弱預設值的設定，以及每個
Defender 排除項目。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`SourcePath`、`HiveType` | 主機、登錄區及其類型（`SOFTWARE`、`SYSTEM`、`NTUSER.DAT`）。 |
| `ControlSet`、`IsCurrentControlSet` | 適用於 `SYSTEM`：控制集，以及它是否為 Windows 最後使用的控制集；當 `Select\Current` 遺失或無效時為未知。 |
| `User`、`UserSID` | 適用於 `NTUSER.DAT`：設定檔資料夾名稱，以及取自系統 `ProfileList` 的使用者 SID。 |
| `Area` | `defender`、`uac`、`credentials`、`remote_access`、`network`、`powershell`、`evidence`。 |
| `Setting` | 設定的種類：`defender_disable_realtime_monitoring`、`defender_exclusion_path`、`uac_local_account_token_filter_policy`、`wdigest_use_logon_credential`、`rdp_deny_connections`、`powershell_script_block_logging`、`event_channel_enabled`…… |
| `Value` | 值：十進位數字、字串或排除項目本身。當值不屬於該設定的類型時為空。 |
| `Meaning` | 該值的意義。 |
| `WeakensDefault` | 當該值削弱了目前的 Windows 在沒有此值時的行為時為 `True`，未削弱時為 `False`；不適用（大小、連接埠、AMSI 提供者）或無法判斷時為空。 |
| `Policy` | 該機碼是否位於 `Policies` 機碼之下。 |
| `KeyPath`、`ValueName` | 機碼與值。排除項目的值名稱即為被排除的項目；AMSI 提供者則沒有值名稱。 |
| `KeyLastWrittenUTC` | 機碼的最後寫入時間。 |
| `DataRaw` | 當值不屬於該設定的類型時，其位元組（十六進位）。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡。 |
| `Problems` | 類型錯誤的值，以及無法讀取的機碼。 |

## 調查策略 { #investigation-strategy }

1. 在每台主機上以 `--weakened-only` 執行，並將主機彼此比較：只有一台主機
   具有的設定會格外醒目。
2. 將每個 Defender 排除項目與在主機上及 Defender 記錄中找到的工具互相核對
   （事件 5007 會記錄設定變更）。
3. 將機碼時間與登入、服務安裝及處理程序建立交叉比對。尋找
   `reg.exe`、`Set-MpPreference`、`netsh advfirewall` 與 `powershell` 的命令列（事件
   4688、Sysmon 1 與 13、PowerShell 4104）。
4. 對於認證相關設定（`WDigest`、`RunAsPPL`、`LocalAccountTokenFilterPolicy`、
   `DisableRestrictedAdmin`），請尋找之後的認證資訊傾印與 pass-the-hash 行為。
5. 使用 `parse-registry read --recover-deleted` 找出為了還原預設值而被刪除的值，並使用
   `--all-control-sets` 檢視較舊的控制集。

## 延伸資訊 { #further-details }

- [Microsoft：設定並驗證 Microsoft Defender 防毒軟體的排除項目](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus)
- [Microsoft：使用者帳戶控制的設定與組態](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft：設定額外的 LSA 保護](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft：about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001：Impair Defenses: Disable or Modify Tools](https://attack.mitre.org/techniques/T1562/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
