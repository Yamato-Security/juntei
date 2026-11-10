# Persistence Hunter { #persistence-hunter }

持續存取是讓程式碼再次執行的方法：在 Windows 啟動、使用者登入、應用程式
開啟、排定時間到達或其他事件發生時執行。攻擊者可以濫用合法軟體用於更新、
裝置支援、安全性與管理的相同設定。

`juntei persistence-hunter` 會離線檢查所擷取的 Windows 證據，提供 **131 項檢查**，涵蓋
這些機制以及相關的帳戶／保護異常。它會讀取登錄區、工作檔案、
指令碼、應用程式設定，以及選定的資料庫記錄。

本頁說明每一項已實作的檢查。完整的選項、輸出欄位、設定檔與
時間戳記處理方式，請參閱[命令指南](../commands/persistence-hunter.md)。
[致謝](acknowledgements.md)頁面連結了 Python 的 PyrsistenceSniper 專案以及原始的
PowerShell PersistenceSniper 工具。

## 開始調查 { #start-an-investigation }

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o PersistenceHunter.csv --coverage Coverage.csv
```

使用 `--min-severity info` 可取得完整的候選項目清單。手動執行命令時預設為
`medium`；其設定檔可能會將部分符合允許清單的項目歸類為 `LOW`，即使因缺少酬載或
簽署者證據而無法進行完整比對。符合允許清單或是陌生的項目，都無法斷定
某個項目是否為惡意。`auto-parse` 已經會保留所有嚴重性等級，並將發現項目寫入
每部電腦的 `persistence/` 類別下。

若要聚焦於某個機制，請從下方複製其**檢查 ID**：

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,scheduled_task_files --min-severity info -o focused.csv
```

此命令預設使用共用的工作執行緒集區。加上 `-T 4` 可選擇四個工作執行緒，或使用 `-T 1`
進行單執行緒掃描。輸入可以包含多部電腦；各電腦的證據會分開處理。

## 將發現項目視為證據來解讀 { #read-findings-as-evidence }

| 問題 | 要檢查的項目 |
|---|---|
| 設定了什麼？設定在哪裡？ | `CheckId`、`Path`，以及包含參數在內的完整 `Value`。循著 `SourceFile` 找到所擷取的登錄區、檔案或資料庫。 |
| 目標是否存在於收集的資料中？ | `ResolvedPath`、`ResolutionStatus` 與 `Exists`。未收集、不明確或不支援的參照屬於未知；Juntei 不會以分析人員機器上的檔案替代。 |
| 該設定是否實際生效？ | 實際的觸發條件、啟用旗標、選定的控制集、Windows／應用程式版本、使用者與架構。部分檢查會刻意保留未生效的設定。 |
| 這是攻擊嗎？ | `Severity` 與 `PolicyReason` 說明的是設定檔比對結果，而非判定結論。請與預期的軟體、完整命令、檔案雜湊值、獨立的執行證據以及周邊時間軸進行比較。 |
| 誰可以執行它？ | `AccessGained` 是範圍／脈絡的提示。全機器範圍的設定通常顯示 `SYSTEM`，但這並不能證明所啟動的處理程序是以 SYSTEM 身分執行。工作主體與 BITS 擁有者 SID 可在有資料時提供更具體的脈絡。 |
| 何時變更的？ | `KeyLastWrite` 是整個登錄機碼的時間。`LastChange`、`ChangeSource` 與 `ChangeEvidence` 則在可用時描述另外進行的 MFT/EVTX 交叉比對。這兩類欄位都不會自動代表酬載的執行時間。 |
| 復原的狀態有多可靠？ | 請閱讀 `RecordStatus`、`Coverage.csv`，以及關於重播、還原（carving）、損毀、遺失或受限證據的警告。歷史上被復原的記錄可能已不再生效。 |
| 具名的發行者是否可信？ | `Signer` 與 `SignerStatus` 是描述性的 PE／憑證／目錄（catalog）中繼資料。Juntei 不會驗證 Authenticode 簽章或憑證信任關係。 |

沒有發現項目並不能證明不存在持續存取機制。`complete` 的涵蓋範圍列表示
已讀取適用的已收集來源，而不是表示擷取的資料包含所有相關檔案、
所有使用者，或所有可能的 Windows 持續存取機制。其他涵蓋範圍狀態則說明遺失、
部分、停用或失敗的檢查。此獵捕工具並不會掃描即時記憶體、網路或雲端身分。

## 檢查目錄與位置表示法 { #catalogue-and-location-notation }

此檢查目錄以 PyrsistenceSniper 為基礎，並加上原生的 `start_menu_com` 檢查；
下方說明記錄了
Juntei 的原生行為，包括其證據上的限制。共有 **116 項以登錄為基礎的
檢查，以及 15 項檔案／存放庫檢查**。部分檔案檢查也會結合登錄資料，因此這些是
實作上的計數，而非互斥的擷取需求。下方依調查目的將各機制
分組，並在每個群組內依檢查 ID 的字母順序排列。

- `HKLM\SYSTEM`、`HKLM\SOFTWARE` 與 `HKLM\SAM` 指的是所擷取的機器登錄區，通常
  從 `Windows\System32\config` 收集，並在可取得時一併收集交易記錄檔。
- `HKU\<user>` 表示相關的已擷取使用者登錄區。大多數使用者設定來自 NTUSER.DAT；
  類別註冊則可能如註明所示來自 `AppData\Local\Microsoft\Windows\UsrClass.dat`。
- `<ControlSet>` 表示 SYSTEM 中所選定的作用中控制集。DSRM 檢查額外的
  歷史位置會在下方明確說明。檢視這些位置時，請查看 `SYSTEM\Select`。
- `<name>` 是可變的路徑元件。在登錄目標表格中，`<child>` 表示直接的
  子項註冊；它並不表示無限制的遞迴走訪。明確進行遞迴的檢查
  會另外說明其走訪方式。Office 路徑中的方括號表示可選的版本。
- 表格列出檢查所選取的實際機器與使用者檢視。已實作 32 位元檢視的檢查
  會將其列出；請勿假設每項檢查都會檢視所有可能的架構／檢視。
- 一般的值檢查會輸出非空白的已設定項目，包括多字串元素以及適用時的
  數值零。這些檢查本身並不要求存在惡意酬載、已啟用的觸發條件，
  或成功的執行。更具選擇性的判斷條件會在各檢查中說明。

ATT&CK ID 是沿用自檢查目錄的對應關係，也是延伸學習的連結。它們描述的是潛在的
濫用方式，而不是認定攻擊者已執行該技術；保護設定與診斷類
檢查可能描述的是促成條件，而非獨立的持續存取機制。

| 機制群組 | 檢查數 |
|---|---:|
| [開機、登入與啟動](#boot-sign-in-and-startup) | 32 |
| [服務與系統擴充功能](#services-and-system-extensions) | 13 |
| [排定的工作](#scheduled-tasks) | 3 |
| [WMI、BITS 與其他事件觸發器](#wmi-bits-and-other-event-triggers) | 11 |
| [Explorer、COM 與檔案或 URL 處理常式](#explorer-com-and-file-or-url-handlers) | 15 |
| [Office 應用程式與增益集](#office-applications-and-add-ins) | 7 |
| [偵錯工具、當機處理常式與相容性填充碼](#debuggers-crash-handlers-and-compatibility-shims) | 13 |
| [.NET 執行階段與分析工具設定](#net-runtime-and-profiler-settings) | 7 |
| [DLL 與執行檔搜尋覆寫](#dll-and-executable-lookup-overrides) | 18 |
| [帳戶與登入可見性](#accounts-and-sign-in-visibility) | 4 |
| [驗證元件與認證保護](#authentication-components-and-credential-protection) | 8 |
| **總計** | **131** |

## 開機、登入與啟動 { #boot-sign-in-and-startup }

這些設定會安排命令或函式庫在開機、登入或桌面初始化期間執行。請檢視 Windows 版本與適用的使用者：某項設定可能已過時、已停用，或是為 Kiosk 用途而設，而非攻擊者所為。

### Active Setup {#active_setup}

**檢查 ID：** `active_setup` · **檢查目錄 ATT&CK：** [T1547.014](https://attack.mitre.org/techniques/T1547/014/)

**Juntei 檢查的內容：**讀取原生與 32 位元檢視中，全機器範圍 Active Setup 元件註冊的 `StubPath`。Windows 可能在準備使用者設定檔時使用這些命令；此檢查會略過字面上的預留位置值 `U` 與 `/UserInstall`。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Active Setup\Installed Components\<component>\StubPath` 及其對應的 `Wow6432Node` 樹狀結構。

**調查方式：**將每個元件與其已安裝的產品進行比較，並檢查命令參數。全機器範圍的註冊可能導致程式在登入使用者的工作階段中執行；輸出中的 `SYSTEM` 標籤描述的是設定範圍，並不能證明所產生處理程序的權杖。

### Boot Execute {#boot_execute}

**檢查 ID：** `boot_execute` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**讀取 Session Manager 的 `BootExecute` 多字串值，並保留每個非空白的項目。它記錄了設定在開機早期階段執行的命令。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `BootExecute` |

**調查方式：**`autocheck autochk *` 是常見的預設值，磁碟工具也可能新增合法的項目。請調查新增的命令及其檔案，但不要假設每個非預設項目都曾執行或都是惡意的。

### Boot Verification Program {#boot_verification_program}

**檢查 ID：** `boot_verification_program` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**讀取 `BootVerificationProgram` 的 `ImagePath`，這是與開機驗證相關聯的執行檔位置。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\BootVerificationProgram` | `ImagePath` |

**調查方式：**確認這個較舊的機制是否適用於所擷取的 Windows 版本與開機設定。解析目標檔案，並將其用途與預期的開機管理軟體進行比較。

### Explorer Load 值 {#explorer_load}

**檢查 ID：** `explorer_load` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**從機器與使用者登錄區讀取與 Windows 登入初始化設定相關聯的 `Load` 值。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |

**調查方式：**調查陌生的命令、使用者可寫入的目的地，以及會擷取或解碼內容的參數。請另外確認適用性與執行情形；空白結果並不涵蓋其他登入機制。

### 字型驅動程式 {#font_drivers}

**檢查 ID：** `font_drivers` · **檢查目錄 ATT&CK：** [T1547](https://attack.mitre.org/techniques/T1547/)

**Juntei 檢查的內容：**列舉舊版 `Font Drivers` 登錄機碼中的非空白值，視為潛在的函式庫註冊。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Font Drivers` | 所有非空白的值 |

**調查方式：**請將此視為不常見的舊版設定攻擊面。在斷定它可能被載入之前，請先確認 Windows 版本的支援情形與所參照的函式庫，並以處理程序或映像載入證據佐證實際使用情形。

### 群組原則擴充 DLL {#gp_extension_dlls}

**檢查 ID：** `gp_extension_dlls` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取 Winlogon 下每個直接子層群組原則擴充註冊的 `DllName`。這些 DLL 用於擴充原則處理。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\GPExtensions\<child>` | `DllName` |

**調查方式：**管理與資安產品可能會安裝擴充功能。請將擴充 GUID 與 DLL 對應到預期的產品，並在主張已執行之前，先檢查原則處理與映像載入的證據。

### 群組原則指令碼 {#gp_scripts}

**檢查 ID：** `gp_scripts` · **檢查目錄 ATT&CK：** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Juntei 檢查的內容：**從所擷取的本機群組原則 `scripts.ini` 與 `psscripts.ini` 註冊檔案（包括每位使用者的本機原則存放區）中剖析命令／參數配對。啟動／關機與登入／登出區段會保留其所設定的命令與參數，並將機器區段與使用者區段區分開來。

**檢查的證據：**

- `Windows\System32\GroupPolicy\Machine\Scripts\scripts.ini` / `psscripts.ini` 以及對應的 `GroupPolicy\User\Scripts` 檔案。
- `Windows\System32\GroupPolicyUsers\<SID>\User\Scripts\scripts.ini` / `psscripts.ini`。

**調查方式：**只會檢查已實作的本機原則註冊位置；這並非完整的 SYSVOL/GPO 稽核。單純位於原則目錄下其他位置的指令碼檔案並不足以構成證據。請將其註冊與原則處理及執行證據交叉比對。

### 登入指令碼（UserInitMprLogonScript） {#logon_scripts}

**檢查 ID：** `logon_scripts` · **檢查目錄 ATT&CK：** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Juntei 檢查的內容：**讀取使用者的 `UserInitMprLogonScript` 環境值，此值可為該使用者的登入指定一個指令碼。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKU\<user>\Environment` | `UserInitMprLogonScript` |

**調查方式：**與核准的登入指令碼進行比較，並檢查實際的指令碼內容。此值並不能證明之後曾有登入或執行，而由網域提供的指令碼則需要額外的原則證據。

### Platform Execute {#platform_execute}

**檢查 ID：** `platform_execute` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**讀取 Session Manager 中每個非空白的 `PlatformExecute` 項目。它可識別被選定用於平台相關開機初始化的程式。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `PlatformExecute` |

**調查方式：**硬體或佈建流程可能可以解釋某個項目。請判斷此 Windows 組建的開機路徑是否會使用它，並調查所參照的映像，而不是僅憑值名稱推斷為攻擊。

### RDP 用戶端擴充 DLL {#rdp_clx_dll}

**檢查 ID：** `rdp_clx_dll` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**讀取 Terminal Server 預設使用者設定中的 `ClxDllPath`，視為可能的 RDP 相關擴充位置。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\DefaultUserConfiguration` | `ClxDllPath` |

**調查方式：**檢查 Windows／RDP 版本與已安裝的擴充功能。已設定的 DLL 是映像載入分析的線索；單憑登錄值並不能顯示曾發生 RDP 連線或 DLL 載入。

### RDP 虛擬通道 DLL {#rdp_virtual_channel}

**檢查 ID：** `rdp_virtual_channel` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**列舉全機器範圍 Terminal Server Client `Default\AddIns` 註冊下的值。二進位值會在可能時以 UTF-16LE 解碼，以保留可用的擴充路徑。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Terminal Server Client\Default\AddIns\<add-in>`：所有值。

**調查方式：**RDP 用戶端與裝置重新導向產品會合法地註冊增益集。請檢查實際的 DLL 或命令以及相關的用戶端工作階段；增益集下的每個值並不都是執行檔路徑。

### RDP WDS 啟動程式 {#rdp_wds_startup}

**檢查 ID：** `rdp_wds_startup` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**讀取 RDP Windows Station 驅動程式設定中的 `StartupPrograms`。它可識別設定在遠端工作階段啟動時執行的程式。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\Wds\rdpwd` | `StartupPrograms` |

**調查方式：**`rdpclip` 是常見的項目。請調查新增的項目與主機的 RDP 設定，並使用工作階段／處理程序記錄判斷所設定的程式是否確實曾啟動。

### 登錄 Run 機碼 {#run_keys}

**檢查 ID：** `run_keys` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**從機器／使用者登錄區列舉 Run、RunOnce、RunEx、RunOnceEx 以及原則 Explorer Run 的值，另加上所列的機器 32 位元機碼。走訪 RunEx/RunOnceEx 區段時會進行邊界與循環檢查。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | 所有非空白的值 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | 所有非空白的值 |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | 所有非空白的值 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | 所有非空白的值 |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | 所有非空白的值 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | 所有非空白的值 |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | 所有非空白的值 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | 所有非空白的值 |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | 所有非空白的值 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | 所有非空白的值 |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Run` | 所有非空白的值 |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnce` | 所有非空白的值 |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnceEx` | 所有非空白的值 |

**調查方式：**軟體更新程式與系統匣應用程式經常出現在這裡。請比較執行檔、命令參數與使用者脈絡。RunOnce 項目在使用後可能會消失；所擷取的狀態並非先前所有登入的完整歷程。

### RunServices（舊版） {#run_services}

**檢查 ID：** `run_services` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**列舉舊版全機器範圍 `RunServices` 機碼中的值。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServices` | 所有非空白的值 |

**調查方式：**這是歷史上的設定攻擊面，並不保證新版 Windows 組建會執行它。請調查目標與作業系統的適用性，然後尋找獨立的執行證據。

### RunServicesOnce（舊版） {#run_services_once}

**檢查 ID：** `run_services_once` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**列舉舊版全機器範圍 `RunServicesOnce` 機碼中的值。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServicesOnce` | 所有非空白的值 |

**調查方式：**請將非空白的值視為設定上的線索。確認來源 Windows 版本是否支援此機制，以及部署或相容性軟體是否可以解釋該項目。

### S0 初始命令 {#s0_initial_command}

**檢查 ID：** `s0_initial_command` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**讀取 Session Manager 中的 `S0InitialCommand`，視為潛在的 Session 0 初始化命令。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `S0InitialCommand` |

**調查方式：**確立其對所擷取 Windows 組建與開機狀態的適用性。已設定的命令並不是帶有時間戳記的開機執行記錄；請檢查其檔案，並與開機／處理程序遙測交叉比對。

### Session Manager Execute {#session_manager_execute}

**檢查 ID：** `session_manager_execute` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**讀取 Session Manager 中非空白的 `Execute` 項目，揭露被選定用於工作階段初始化的程式。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `Execute` |

**調查方式：**與預期的映像基準進行比較，並檢查不尋常的原生執行檔或路徑。在主張已執行之前，請先確立變更之後是否曾發生相關的開機／工作階段。

### Session Manager SubSystems {#session_manager_subsystems}

**檢查 ID：** `session_manager_subsystems` · **檢查目錄 ATT&CK：** [T1547](https://attack.mitre.org/techniques/T1547/)

**Juntei 檢查的內容：**讀取 Session Manager 的 `SubSystems` 機碼中的 `Windows` 值，此值設定 Windows 子系統的命令。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\SubSystems` | `Windows` |

**調查方式：**正常的命令很長，而且因版本而異。請將完整的值與參數，與相同組建的可信基準進行比較；變更關鍵的子系統設定也可能導致無法啟動。

### Setup Execute {#setup_execute}

**檢查 ID：** `setup_execute` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**讀取 Session Manager 中的 `SetupExecute` 項目，涵蓋設定在啟動時執行的安裝相關命令。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `SetupExecute` |

**調查方式：**安裝、升級與復原活動可能會建立合法的項目。在將此設定歸因於持續存取之前，請先與安裝記錄及所參照的映像進行比較。

### Shell Folders 啟動資料夾重新導向 {#shell_folders_startup}

**檢查 ID：** `shell_folders_startup` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**將 Shell Folders 與 User Shell Folders 中的 `Startup` 與 `Common Startup`，與預設的啟動資料夾位置進行比較。非預設的重新導向會被回報，並檢查其已收集的資料夾內容。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders` 與 `User Shell Folders`：`Common Startup`。
- NTUSER.DAT 中對應的 `Software\Microsoft\Windows\CurrentVersion\Explorer` 設定：`Startup`，以及重新導向位置中已擷取的檔案。

**調查方式：**重新導向的啟動資料夾可以隱藏自動啟動項目，但重新導向也可能是合法的。路徑必須能在所擷取的證據中解析；未收集的目的地屬於未知，並不能證明該資料夾是空的。

### Shell Launcher 覆寫 {#shell_launcher}

**檢查 ID：** `shell_launcher` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**從 SOFTWARE 讀取原則的 `Shell` 值與 `system.ini` boot Shell 對應。這些是替代的 Shell 選擇攻擊面。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `Shell` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\IniFileMapping\system.ini\boot` | `Shell` |

**調查方式：**Kiosk 與應用程式專用裝置可能會刻意取代 Explorer。請將所選的 Shell 與該機器的角色進行比較，並確認此設定在相關工作階段中確實生效。

### 啟動資料夾 {#startup_folder}

**檢查 ID：** `startup_folder` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**檢查預設的每位使用者與共用啟動資料夾中的直接檔案，並遵循第一個可用的啟動資料夾重新導向。當已收集的 `.lnk` 可以被剖析時，會還原捷徑的目標。

**檢查的證據：**

- `ProgramData\Microsoft\Windows\Start Menu\Programs\Startup` 與 `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`。
- 機器／使用者 Shell Folders 與 User Shell Folders 設定中適用的 `Common Startup` / `Startup` 重新導向。

**調查方式：**啟動資料夾中的連結與檔案可以在登入時啟動合法的應用程式。除了檔案名稱之外，也請檢查目標參數。共用資料夾是共享的設定位置；它並不能證明所啟動的程式使用了 SYSTEM 權杖。

### Terminal Services 初始程式 {#ts_initial_program}

**檢查 ID：** `ts_initial_program` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**讀取 Terminal Services 原則的 `InitialProgram`，以及機器 Terminal Server 安裝模式下的 Run/RunOnce/RunOnceEx 設定，包括有走訪邊界限制的巢狀 RunOnceEx 區段。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` | `InitialProgram` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Run` | 所有非空白的值 |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Runonce` | 所有非空白的值 |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx` | 所有非空白的值 |

**調查方式：**遠端應用程式部署可能會刻意使用這些設定。請確立受影響的工作階段，並循著所選的命令追查處理程序證據；單憑註冊並不能顯示曾有遠端登入。

### Winlogon AppSetup {#winlogon_appsetup}

**檢查 ID：** `winlogon_appsetup` · **檢查目錄 ATT&CK：** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei 檢查的內容：**讀取 Winlogon 的 `AppSetup`，並將以逗號分隔的項目拆分為個別的候選項目。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `AppSetup` |

**調查方式：**將每個元件與預期的 Windows 安裝／登入軟體進行比較。清單中的項目並不代表一次個別觀察到的執行，而其相關性可能取決於 Windows 版本與部署狀態。

### Winlogon GinaDLL {#winlogon_gina_dll}

**檢查 ID：** `winlogon_gina_dll` · **檢查目錄 ATT&CK：** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei 檢查的內容：**讀取舊版 Winlogon 的 `GinaDLL` 替換設定。GINA 是較舊的 Windows 登入擴充模型。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `GinaDLL` |

**調查方式：**這主要與 Vista 之前的系統相關。較新系統上殘留的機碼並不能證明登入擴充實際生效；請檢查來源版本以及任何被參照的 DLL。

**進一步資訊：**[Microsoft 說明文件](https://learn.microsoft.com/en-us/windows/win32/secauthn/gina)。

### Winlogon MPNotify {#winlogon_mpnotify}

**檢查 ID：** `winlogon_mpnotify` · **檢查目錄 ATT&CK：** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei 檢查的內容：**讀取 Winlogon 的 `mpnotify` 命令／函式庫設定，這是與登入相關的擴充攻擊面。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `mpnotify` |

**調查方式：**將目標與參數與相符的 Windows 基準進行比較。在主張程式碼已執行之前，請先確認版本特定的行為，並與之後的登入交叉比對。

### Winlogon Notify 套件 {#winlogon_notify_packages}

**檢查 ID：** `winlogon_notify_packages` · **檢查目錄 ATT&CK：** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei 檢查的內容：**讀取每個直接子層 Winlogon `Notify` 套件註冊中的 `DllName`。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Notify\<child>` | `DllName` |

**調查方式：**通知套件是舊版的擴充機制。請判斷所擷取的 Windows 版本是否會載入它們，並區分核准的驗證軟體與來路不明的函式庫。

### Winlogon Shell {#winlogon_shell}

**檢查 ID：** `winlogon_shell` · **檢查目錄 ATT&CK：** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei 檢查的內容：**讀取全機器範圍與每位使用者的 Winlogon `Shell` 值，以揭露桌面 Shell 的選擇。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |

**調查方式：**Explorer 是常見的值，但 Kiosk 可能使用其他 Shell。請調查新增的命令或不尋常的路徑，並判斷登入時套用的是哪位使用者的實際設定。

### Winlogon System {#winlogon_system}

**檢查 ID：** `winlogon_system` · **檢查目錄 ATT&CK：** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei 檢查的內容：**讀取 Winlogon 的 `System` 值，並將以逗號分隔的項目拆分為個別的候選項目。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `System` |

**調查方式：**請將此視為與版本相關的登入設定線索。檢查每個被參照的元件，並以執行證據佐證，而不要將此清單視為執行歷程。

### Winlogon Taskman {#winlogon_taskman}

**檢查 ID：** `winlogon_taskman` · **檢查目錄 ATT&CK：** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei 檢查的內容：**讀取機器／使用者 Winlogon 的 `Taskman` 值，以找出替代的工作管理命令。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |

**調查方式：**請判斷哪一種 Windows 行為會使用此設定，以及目標是否符合預期。已設定的替換並不能證明使用者曾叫用該功能。

### Winlogon Userinit {#winlogon_userinit}

**檢查 ID：** `winlogon_userinit` · **檢查目錄 ATT&CK：** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei 檢查的內容：**讀取機器／使用者 Winlogon 的 `Userinit` 值，拆分以逗號分隔的程式，並忽略尾端的空白項目。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |

**調查方式：**正常的項目通常指向 `userinit.exe`。請檢查額外的項目與被變更的路徑。確認適用的使用者以及之後的登入；`KeyLastWrite` 屬於整個機碼，而非特定於此值。

### Winlogon VMApplet {#winlogon_vmapplet}

**檢查 ID：** `winlogon_vmapplet` · **檢查目錄 ATT&CK：** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei 檢查的內容：**讀取 Winlogon 的 `VMApplet` 命令設定，涵蓋較舊的系統小程式啟動攻擊面。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `VMApplet` |

**調查方式：**確認作業系統的適用性以及預期的小程式。非預期的命令值得調查，但並不能確立觸發該命令的介面曾被開啟。

## 服務與系統擴充功能 { #services-and-system-extensions }

Windows 服務與可擴充的系統元件提供許多載入執行檔或 DLL 的合法方式。請將所設定的目標與預期的產品、服務帳戶、啟動設定，以及有來源佐證的安裝歷程進行比較。

### DiagTrack DLL {#diagtrack_dll}

**檢查 ID：** `diagtrack_dll` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取 DiagTrack 服務的 `ImagePath`。儘管檢查目錄中的名稱以 DLL 為主，此檢查檢視的是所設定的服務映像命令。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DiagTrack` | `ImagePath` |

**調查方式：**與服務基準以及一般服務檢查進行比較。同一個設定可能產生不只一筆檢查結果；候選項目的數量並不等於不同攻擊的數量。

### DiagTrack Listener DLL {#diagtrack_listener_dll}

**檢查 ID：** `diagtrack_listener_dll` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取 DiagTrack-Listener 自動記錄器設定中的 `FileName`。這是 ETW 自動記錄器設定，而非 DLL 被載入的證據。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\WMI\Autologger\DiagTrack-Listener` | `FileName` |

**調查方式：**檢查原始值與 Windows 記錄設定。檢查目錄將此標示為 DLL 技術，但此查詢並不會驗證可載入的 DLL，也無法證明曾透過此值執行。

### Netsh 協助程式 DLL {#netsh_helper}

**檢查 ID：** `netsh_helper` · **檢查目錄 ATT&CK：** [T1546.007](https://attack.mitre.org/techniques/T1546/007/)

**Juntei 檢查的內容：**列舉機器 `NetSh` 協助程式註冊機碼下的非空白值。協助程式可以用以 DLL 為基礎的命令擴充 `netsh.exe`。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\NetSh` | 所有非空白的值 |

**調查方式：**網路產品可能會安裝協助程式。請將每個值對應到預期的產品並檢查解析後的函式庫，然後尋找相關的 netsh 執行或映像載入記錄。

### 列印監視器 {#print_monitors}

**檢查 ID：** `print_monitors` · **檢查目錄 ATT&CK：** [T1547.010](https://attack.mitre.org/techniques/T1547/010/)

**Juntei 檢查的內容：**讀取作用中控制集內每個直接子層 Print Monitor 註冊下的 `Driver`。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Monitors\<child>` | `Driver` |

**調查方式：**列印軟體與連接埠監視器會合法地擴充多工緩衝處理器。請將 DLL 與產品中繼資料與印表機清冊進行比較，並以多工緩衝處理器／映像載入證據確認是否曾被載入。

### 列印處理器 {#print_processors}

**檢查 ID：** `print_processors` · **檢查目錄 ATT&CK：** [T1547.012](https://attack.mitre.org/techniques/T1547/012/)

**Juntei 檢查的內容：**讀取 x64 與 x86 列印處理器註冊樹狀結構下的 `Driver`。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows x64\Print Processors\<child>` | `Driver` |
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows NT x86\Print Processors\<child>` | `Driver` |

**調查方式：**第三方印表機套件可能會在常見的 Windows 預設值之外新增處理器。在認定其具有惡意意圖之前，請檢視兩種架構、實際的 DLL 以及已安裝的印表機軟體。

### 服務控制管理員擴充 {#scm_extension}

**檢查 ID：** `scm_extension` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**列舉 `ServiceControlManagerExtension` 中的非空白值，這是服務管理員的擴充攻擊面。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\ServiceControlManagerExtension` | 所有非空白的值 |

**調查方式：**檢查所列出的函式庫，以及此 Windows 組建是否會使用該設定。請與開機／服務活動交叉比對；此檢查清查的是註冊項目，並不能證明擴充已被載入。

### DNS 伺服器層級外掛程式 DLL {#server_level_plugin_dll}

**檢查 ID：** `server_level_plugin_dll` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取作用中 SYSTEM 控制集內 DNS 服務的 `ServerLevelPluginDll` 設定。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DNS\Parameters` | `ServerLevelPluginDll` |

**調查方式：**這與執行 DNS 伺服器角色的系統相關，包括部分網域控制站。請調查該 DLL、服務帳戶以及服務重新啟動／載入的證據；若角色未作用或不存在，結論就會不同。

### 服務失敗命令 {#service_failure_command}

**檢查 ID：** `service_failure_command` · **檢查目錄 ATT&CK：** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei 檢查的內容：**讀取作用中控制集內每個服務的 `FailureCommand`，揭露所設定的復原命令。

**檢查的證據：**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\FailureCommand`。

**調查方式：**只有在服務的復原動作選用此命令，且發生適用的失敗時，此命令才會執行。請檢查復原設定以及失敗／處理程序記錄；單憑此值並不能確立復原動作處於作用中。

### SNMP 擴充代理程式 DLL {#snmp_extension_agent}

**檢查 ID：** `snmp_extension_agent` · **檢查目錄 ATT&CK：** [T1574](https://attack.mitre.org/techniques/T1574/)

**Juntei 檢查的內容：**列舉 SNMP 服務 `ExtensionAgents` 註冊機碼下的值。這些值可能指向另一個登錄位置，而不是直接的 DLL 路徑。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\SNMP\Parameters\ExtensionAgents` | 所有非空白的值 |

**調查方式：**請手動循著間接註冊找到其函式庫，並確認 SNMP 已安裝且有在使用。此輸出中未解析的字串並不能證明 DLL 遺失或具有惡意。

### 時間提供者 {#time_providers}

**檢查 ID：** `time_providers` · **檢查目錄 ATT&CK：** [T1547.003](https://attack.mitre.org/techniques/T1547/003/)

**Juntei 檢查的內容：**讀取直接子層 Windows Time `TimeProviders` 註冊中的 `DllName`。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\W32Time\TimeProviders\<child>` | `DllName` |

**調查方式：**請另外檢視提供者的啟用設定與 Windows Time 服務的狀態；DLL 路徑檢查並不要求提供者已啟用。時間同步產品可能會提供合法的替代品。

### Windows 服務（ServiceDll） {#windows_service_dll}

**檢查 ID：** `windows_service_dll` · **檢查目錄 ATT&CK：** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei 檢查的內容：**讀取作用中 SYSTEM 控制集內各服務下的 `Parameters\ServiceDll`，並在可用時附加服務特定的時間戳記證據。

**檢查的證據：**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Parameters\ServiceDll`。

**調查方式：**像 `svchost.exe` 這樣的共用承載程序並不是酬載 DLL。請檢查該函式庫、服務帳戶、啟動狀態，以及相符的服務安裝／映像載入證據；註冊並不能證明該服務曾執行。

**進一步資訊：**[Microsoft 說明文件](https://docs.microsoft.com/en-us/windows/win32/services/service-programs)。

### Windows 服務（ImagePath） {#windows_service_image_path}

**檢查 ID：** `windows_service_image_path` · **檢查目錄 ATT&CK：** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei 檢查的內容：**讀取作用中控制集內服務的 `ImagePath` 命令。類型為 1、2 或 8 且沒有 ImagePath 的驅動程式，會保留隱含的 `System32\drivers\<service>.sys` 目標。

**檢查的證據：**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>`：`ImagePath`、`Type` 以及輔助的服務中繼資料。

**調查方式：**請將服務帳戶與啟動設定，與輸出中的機器範圍標籤分開檢視。已停用或依需求啟動的註冊仍屬於清冊的一部分。就時間戳記而言，以服務名稱為依據的證據優先於不明確的共用執行檔路徑。

**進一步資訊：**[Microsoft 說明文件](https://docs.microsoft.com/en-us/windows/win32/services/service-programs)。

### Windows Update 服務啟動 DLL {#wu_service_startup_dll}

**檢查 ID：** `wu_service_startup_dll` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**從作用中控制集讀取 Windows Update 服務的 `Parameters\ServiceDll`。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\wuauserv\Parameters` | `ServiceDll` |

**調查方式：**將該函式庫與相同 Windows 組建及已安裝的服務維護狀態進行比較。此項目與一般的 ServiceDll 清冊重疊，並不代表第二起不同的入侵。

## 排定的工作 { #scheduled-tasks }

工作排程器會將設定同時儲存在 XML 檔案與 SOFTWARE 登錄區中。請將這兩個來源一併檢視。工作可能已註冊但從未執行過，而收集不完整的資料也可能看起來像是隱藏或幽靈工作。

### 幽靈排定工作 {#ghost_task}

**檢查 ID：** `ghost_task` · **檢查目錄 ATT&CK：** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei 檢查的內容：**比較 SOFTWARE TaskCache 的 `Tree` 與 `Tasks` 註冊；當收集的資料中有工作檔案時，也會尋找其對應的工作 XML 檔案。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree` 與 `TaskCache\Tasks`。
- 用於檔案系統比較的已擷取 `Windows\System32\Tasks\<task path>` 檔案。

**調查方式：**缺少對應項目可能反映出已刪除／過時的工作、內建的註冊，或收集不完整。若沒有收集到 XML，就無法進行檔案系統比較。在將工作稱為隱藏或惡意之前，請先檢查涵蓋範圍與確切的不一致之處。

### 隱藏的排定工作 {#hidden_scheduled_task}

**檢查 ID：** `hidden_scheduled_task` · **檢查目錄 ATT&CK：** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei 檢查的內容：**檢查 TaskCache Tree 註冊是否缺少 `SD` 值或具有空白的二進位安全性描述元，這些情況與將工作從一般列舉中隱藏有關。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\<task>`：`SD` 是否存在及其資料，以及 TaskCache 對應項目的中繼資料。

**調查方式：**SD 遺失或空白是特定的設定異常，並不能證明該工作曾執行或具有惡意。XML 的 `Hidden` 設定與任意的 ACL 問題屬於不同的情況，並非此處使用的判斷條件。

**進一步資訊：**[Microsoft 說明文件](https://www.microsoft.com/en-us/security/blog/2022/04/12/tarrask-malware-uses-scheduled-task-defense-evasion/)。

### 排定的工作（XML 檔案） {#scheduled_task_files}

**檢查 ID：** `scheduled_task_files` · **檢查目錄 ATT&CK：** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei 檢查的內容：**剖析所擷取的工作 XML，找出 Exec 命令／參數以及 COM 處理常式的類別 ID／資料。它會依 `Actions Context`/`Principal id` 選取主體，分別保留使用者與群組身分，並回報 SYSTEM、USER 或 UNKNOWN。未解析的 COM 動作仍會保持可見。

**檢查的證據：**

- `Windows\System32\Tasks\<task path>` XML 檔案，包括 Exec、ComHandler 以及可用的主體欄位。
- 相符使用者所擷取的 `UsrClass.dat` 與 NTUSER 的 `Software\Classes`，以及機器 SOFTWARE 的 `Classes\CLSID`，涵蓋原生與 32 位元檢視。
- 用於 SID 主體的已擷取 SOFTWARE `ProfileList`。具名主體會優先採用唯一相符的 `user.DOMAIN` 資料夾，其次是已擷取的本機主機尾碼，最後才是單純的 `user` 資料夾。相符結果會明確標示為啟發式判斷；真正不明確的情況會保持未解析，並在發現項目與涵蓋範圍中註明。不需要揮發性登錄機碼。
- 受控碼的 `CodeBase`（包括版本子機碼）、AppID 關聯，以及 AppID `LocalService` 所對應的 SYSTEM 服務 `ImagePath`。

COM 伺服器候選項目會在 `Description` 中包含其登錄來源。只有執行檔名稱的
AppID 關聯會保持未解析，而不會被指派一個猜測的目錄。本機
`file:` URL 會被解碼；遠端 URL 則原樣保留，不進行網路存取。使用者類別覆寫
的範圍僅限於該設定檔，而不完整的使用者 CLSID 不會退回使用被其遮蔽的
機器註冊。僅有群組的工作會保留機器候選項目，並在其說明中註明無法對應到個別使用者的
限制，而不會在涵蓋範圍錯誤摘要中加入例行通知。

**調查方式：**在來源 XML 與事件記錄中檢查觸發條件、啟用狀態、主體與執行歷程。已建立的工作可能從未執行。未解析的 COM 處理常式仍是候選項目，而遺失或僅有群組的主體資料不可被假設為代表 SYSTEM。

## WMI、BITS 與其他事件觸發器 { #wmi-bits-and-other-event-triggers }

Windows Management Instrumentation（WMI）支援系統管理。Background Intelligent Transfer Service（BITS）支援背景傳輸。兩者都可以保留由事件觸發的命令。有些機制會等待管理事件、傳輸通知、應用程式啟動或使用者動作。儲存的觸發器是有用的設定證據，但它的存在並不能確立該觸發器曾被觸發。

### 協助工具功能後門 {#accessibility_tools}

**檢查 ID：** `accessibility_tools` · **檢查目錄 ATT&CK：** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Juntei 檢查的內容：**先要求從已收集的參考二進位檔取得可用的 Microsoft 目錄中繼資料，然後使用 PE OriginalFilename 與憑證／目錄中繼資料，比較七個已收集的 System32 協助工具二進位檔。

**檢查的證據：**

- `Windows\System32\sethc.exe`、`osk.exe`、`narrator.exe`、`magnify.exe`、`utilman.exe`、`atbroker.exe` 與 `DisplaySwitch.exe`。
- 已收集的 `Windows\System32\winlogon.exe`、`cmd.exe` 或 `notepad.exe` 參考中繼資料，以及適用的已收集目錄檔。

**調查方式：**中繼資料可能暗示有經過重新命名的替換檔案，但 Juntei 不會驗證簽章或憑證信任關係。缺少所需目錄檔的稀疏收集，只會產生不完整的比較涵蓋範圍，而不會將每個看似未簽署的檔案都視為後門。

### 協助工具功能 {#assistive_technology}

**檢查 ID：** `assistive_technology` · **檢查目錄 ATT&CK：** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Juntei 檢查的內容：**讀取原生／32 位元 SOFTWARE 檢視中已註冊協助工具的 `StartExe` 與 `StartParams` 命令，以及使用者與 DEFAULT 登錄區中已啟用的協助工具 `Configuration` 項目。數值／非字串的 StartExe 預留位置會被略過。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\<feature>`：`StartExe` 與 `StartParams`；以及機器的 `Wow6432Node` 檢視。
- NTUSER.DAT 中的 `HKU\<user>\Software\Microsoft\Windows NT\CurrentVersion\Accessibility\Configuration`，以及 DEFAULT 登錄區中對應的設定。

**調查方式：**協助工具軟體是合法的。請區分已啟用的功能名稱與其實際的執行檔註冊，然後檢查目標以及登入／桌面的執行證據。這與替換協助工具執行檔本身是不同的情況。

**進一步資訊：**[技術參考資料（hexacorn.com）](https://www.hexacorn.com/blog/2016/07/22/beyond-good-ol-run-key-part-42/)。

### BITS 工作通知命令 {#bits_notify_command}

**檢查 ID：** `bits_notify_command` · **檢查目錄 ATT&CK：** [T1197](https://attack.mitre.org/techniques/T1197/)

**Juntei 檢查的內容：**從已擷取的 `qmgr.db`、`qmgr0.dat` 或 `qmgr1.dat` 中還原以計數字串表示的 BITS 工作記錄，保留非空白的通知程式、參數、工作 ID 與擁有者 SID。

**檢查的證據：**

- `ProgramData\Microsoft\Network\Downloader\qmgr.db`、`qmgr0.dat` 與 `qmgr1.dat`。

**調查方式：**這些是還原（carving）而來的記錄：已配置／目前的工作狀態並未經過驗證，過時／已刪除的工作也可能出現。資料庫的 MFT 寫入時間只是薄弱的容器證據，而非通知的執行時間。擁有者 SID 可區分 SYSTEM、本機服務、網路服務與使用者。

### 命令處理器 AutoRun {#cmd_autorun}

**檢查 ID：** `cmd_autorun` · **檢查目錄 ATT&CK：** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei 檢查的內容：**讀取機器／使用者 Command Processor 的 `AutoRun` 命令，此設定可能影響命令提示字元的啟動。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |
| `HKU\<user>\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |

**調查方式：**開發工具與 Shell 自訂可能可以解釋某個值。`cmd.exe /d` 會略過 AutoRun，因此單憑設定並不能證明每次出現 cmd.exe 時它都有執行。請檢查命令內容與觸發處理程序的參數。

**進一步資訊：**[Microsoft 說明文件](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd)。

### ErrorHandler.cmd 持續存取 {#error_handler_cmd}

**檢查 ID：** `error_handler_cmd` · **檢查目錄 ATT&CK：** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei 檢查的內容：**將已收集的 `Windows\System32\ErrorHandler.cmd` 指令碼的存在回報為錯誤處理常式持續存取的候選項目。

**檢查的證據：**

- `Windows\System32\ErrorHandler.cmd`。

**調查方式：**單憑存在並不能證明適用的 Windows 復原／錯誤路徑曾叫用它。請閱讀該指令碼、檢查其來源，並與安裝／錯誤／處理程序證據交叉比對。

### Power Automate Desktop 流程 {#power_automate}

**檢查 ID：** `power_automate` · **檢查目錄 ATT&CK：** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei 檢查的內容：**在每位使用者的 Power Automate Desktop `flows` 與 `scripts` 存放區下尋找已擷取的流程／指令碼目錄，每個找到的流程只回報一次。

**檢查的證據：**

- `Users\<user>\AppData\Local\Microsoft\Power Automate Desktop\flows\<flow>\...` 與 `scripts\<flow>\...`。

**調查方式：**這只能識別已儲存的自動化流程，而非每個排定的觸發條件或已執行的流程。在將其視為自動或惡意的持續存取機制之前，請先檢查流程內容與外部的啟動設定。

### PowerShell 設定檔 {#powershell_profiles}

**檢查 ID：** `powershell_profiles` · **檢查目錄 ATT&CK：** [T1546.013](https://attack.mitre.org/techniques/T1546/013/)

**Juntei 檢查的內容：**在可辨識的 Windows PowerShell 與 PowerShell 安裝／設定檔目錄中尋找 profile.ps1 類型的檔案，包括使用者的 Documents、找到的 OneDrive Documents，以及已收集的 Documents 重新導向。

**檢查的證據：**

- 直接位於 `Windows\System32\WindowsPowerShell\v1.0` 與 `Windows\SysWOW64\WindowsPowerShell\v1.0` 中、檔名以 `profile.ps1` 結尾的檔案。
- 每位使用者的 `Documents\WindowsPowerShell` 與 `Documents\PowerShell`，包括已擷取的 Documents 重新導向與找到的 OneDrive Documents 資料夾。
- 直接位於 `Program Files\PowerShell` 或 `Program Files (x86)\PowerShell` 下版本目錄中、檔名以 `profile.ps1` 結尾的檔案。

**調查方式：**設定檔可能包含無害的自訂內容。請閱讀指令碼並追查其所叫用的酬載。以 `-NoProfile` 啟動 PowerShell 會略過設定檔；單憑檔案存在並不能證明設定檔曾被執行或含有惡意內容。

### 螢幕保護程式劫持 {#screensaver}

**檢查 ID：** `screensaver` · **檢查目錄 ATT&CK：** [T1546.002](https://attack.mitre.org/techniques/T1546/002/)

**Juntei 檢查的內容：**讀取每位使用者 Desktop 的 `SCRNSAVE.EXE` 值，以識別所設定的螢幕保護程式映像。

**檢查的證據：**

- NTUSER.DAT 中的 `HKU\<user>\Control Panel\Desktop\SCRNSAVE.EXE`。

**調查方式：**與核准的螢幕保護程式進行比較，並檢查其目標。其他桌面設定、工作階段狀態與逾時時間決定了它是否能被觸發；已儲存的路徑並不能證明螢幕保護程式曾啟動。

### 遙測控制器命令 {#telemetry_controller}

**檢查 ID：** `telemetry_controller` · **檢查目錄 ATT&CK：** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei 檢查的內容：**讀取 TelemetryController 及其直接子層註冊中的 `Binary`、`Command` 與 `MaintenanceCommand`。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController` 及其直接子項：`Binary`、`Command`、`MaintenanceCommand`。

**調查方式：**將所選的執行檔與參數，與相同的 Windows 組建及維護軟體進行比較。登錄中的命令並不能證明維護觸發器曾被觸發。

### Windows Terminal 自訂設定檔 {#windows_terminal}

**檢查 ID：** `windows_terminal` · **檢查目錄 ATT&CK：** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei 檢查的內容：**剖析封裝版、預覽版與未封裝版的 Windows Terminal JSON/JSONC 設定，並回報非空白的設定檔／預設 `commandline` 設定。

**檢查的證據：**

- `Users\<user>\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json` 以及 WindowsTerminalPreview 的對應檔案。
- `Users\<user>\AppData\Local\Microsoft\Windows Terminal\settings.json`：舊版 profiles 陣列、`profiles.list` 與 `profiles.defaults` 的 commandline 欄位。

**調查方式：**一般的 Shell 自然會出現在這裡。請判斷所選／預設的設定檔，並檢查不尋常的命令或啟動參數。僅儲存設定檔並不會啟動它。

### WMI 事件訂閱 {#wmi_event_subscription}

**檢查 ID：** `wmi_event_subscription` · **檢查目錄 ATT&CK：** [T1546.003](https://attack.mitre.org/techniques/T1546/003/)

**Juntei 檢查的內容：**從已擷取的 WMI OBJECTS.DATA 存放庫中還原可辨識的事件取用者執行個體，以及篩選器與取用者之間的繫結參照。可辨識的類別包括命令列、作用中指令碼、記錄檔、事件記錄與 SMTP 取用者。

**檢查的證據：**

- `Windows\System32\wbem\Repository\OBJECTS.DATA` 與 `Repository\FS\OBJECTS.DATA`。

**調查方式：**原始存放庫中的命中結果可能已過時、未繫結或屬於歷史資料；輸出會保留這些限定說明。還原並不能重建完整的作用中訂閱關係圖，也無法證明觸發器曾被觸發。存放庫的檔案寫入時間並非訂閱的建立／執行時間。

## Explorer、COM 與檔案或 URL 處理常式 { #explorer-com-and-file-or-url-handlers }

元件物件模型（COM）讓應用程式能夠啟用已註冊的軟體元件。Windows Shell 使用已註冊的處理常式來開啟檔案、URL、資料夾與特殊物件。COM 識別碼（CLSID）用於識別元件；ProgID 則為部分元件提供易讀的名稱。請循著每個註冊追查其所選用的命令或 DLL。

### App Paths 劫持 {#app_paths}

**檢查 ID：** `app_paths` · **檢查目錄 ATT&CK：** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei 檢查的內容：**讀取全機器範圍 `App Paths` 註冊下的預設值，這些值可以在透過 Shell 啟動時重新導向應用程式的搜尋。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\<application>` 的預設值。

**調查方式：**App Paths 註冊對已安裝的應用程式而言很常見，而且並不會全面改寫每一次處理程序啟動。請將註冊的名稱與映像與預期的軟體進行比較，然後檢查相關的 Shell／處理程序證據。

### COM 物件劫持 {#com_treat_as}

**檢查 ID：** `com_treat_as` · **檢查目錄 ATT&CK：** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei 檢查的內容：**在支援的原生／32 位元類別存放區中，尋找 COM `TreatAs` 重新導向，以及會遮蔽機器 CLSID 的每位使用者 InprocServer32/LocalServer32 註冊。TreatAs 目標會使用同一位使用者的類別與機器類別來解析。

**檢查的證據：**

- `HKLM\SOFTWARE\Classes\CLSID\<CLSID>` 以及機器的 `Classes\Wow6432Node` 檢視。
- NTUSER.DAT 中每位使用者的 `Software\Classes\CLSID` 與 UsrClass.dat 中的 `CLSID`，包括其已實作的 `Wow6432Node` 檢視：`TreatAs`、`InprocServer32` 與 `LocalServer32` 的預設值。

**調查方式：**每位使用者的覆寫可能不需要系統管理員權限，但合法軟體也可能註冊它們。請追查所選的類別與 DLL／執行檔，評估架構與啟用脈絡，並以獨立方式佐證實際的 COM 啟用。

### 磁碟清理處理常式劫持 {#disk_cleanup_handler}

**檢查 ID：** `disk_cleanup_handler` · **檢查目錄 ATT&CK：** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei 檢查的內容：**讀取磁碟清理 VolumeCaches 處理常式的 CLSID，並循著可解析的機器 COM InprocServer32 註冊找到其函式庫。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\<handler>` 的預設 CLSID 與 `Classes\CLSID\<CLSID>\InprocServer32`。

**調查方式：**處理常式必須由相關的清理流程叫用才會執行。此檢查不會輸出未解析的處理常式，因此若擷取的資料不完整，請使用登錄剖析器檢查其原始註冊。

### Explorer AppKey 覆寫 {#explorer_app_key}

**檢查 ID：** `explorer_app_key` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**讀取 Explorer `AppKey` 直接子項下的 `ShellExecute` 與 `Association` 值，涵蓋由應用程式按鍵所選用的命令／動作。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\AppKey\<key>`：`ShellExecute` 與 `Association`。

**調查方式：**鍵盤公用程式與 OEM 工具可能會定義這些值。關聯名稱可能需要再透過另一個登錄位置追查，而不是直接的檔案路徑。請確認會觸發它的使用者動作。

### 瀏覽器協助程式物件 {#explorer_bho}

**檢查 ID：** `explorer_bho` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**讀取 Browser Helper Object 的 CLSID 註冊，並在可用時解析機器的 InprocServer32 函式庫；未解析的類別 ID 仍會保持可見。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects\<CLSID>` 以及機器 `Classes\CLSID\<CLSID>\InprocServer32` 的預設值。

**調查方式：**這與相容的瀏覽器／Explorer 行為相關，在舊版環境中特別重要。核准的瀏覽器／資安擴充功能可能會出現在這裡。已註冊的 BHO 並不能證明目前的瀏覽器曾載入它。

### Explorer 桌面 CLSID 劫持 {#explorer_clsid_hijack}

**檢查 ID：** `explorer_clsid_hijack` · **檢查目錄 ATT&CK：** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei 檢查的內容：**從機器／使用者類別存放區讀取所列出的三個特殊 Explorer 命名空間 CLSID 上所設定的 Shell 開啟命令。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |

**調查方式：**將桌面、我的電腦與我的文件的命令處理常式與相符的基準進行比較。此檢查只檢視選定的 CLSID 與路徑，而非所有可能的 Explorer 命名空間劫持。

### Explorer 內容功能表處理常式 {#explorer_context_menu}

**檢查 ID：** `explorer_context_menu` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**列舉 Directory、Directory Background、Folder、Drive、AllFilesystemObjects 與 `*` 的內容功能表處理常式，並在可能時循著類別 ID 找到已收集的 DLL 註冊。

**檢查的證據：**

- 機器 `HKLM\SOFTWARE\Classes` 與使用者 UsrClass.dat 類別存放區：`<subject>\shellex\ContextMenuHandlers\<handler>` 的預設值及對應的 COM 函式庫註冊。

**調查方式：**來自壓縮工具、資安產品與雲端用戶端的右鍵擴充功能很常見。請檢查實際的處理常式與觸發它的互動。未解析的 GUID 仍是有用的線索，而不能證明該 DLL 不存在。

### 檔案關聯劫持 {#file_association_hijack}

**檢查 ID：** `file_association_hijack` · **檢查目錄 ATT&CK：** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei 檢查的內容：**檢查 `.txt`、`.pdf`、`.doc`、`.docx`、`.html`、`.htm`、`.js`、`.vbs`、`.hta`、`.exe`、`.bat`、`.cmd` 與 `.ps1` 的直接副檔名註冊以及解析後的 ProgID `shell\open\command` 註冊。

**檢查的證據：**

- 機器 `HKLM\SOFTWARE\Classes` 與使用者 UsrClass.dat 類別存放區：選定的 `<extension>` 預設值與 `<extension-or-ProgID>\shell\open\command` 預設值。

**調查方式：**與預期的應用程式進行比較，並檢查命令參數。這是一組既定的副檔名，而非所有檔案類型，也不是完整的實際 UserChoice 解析結果。已儲存的命令並不能證明曾開啟某份文件。

### 通訊協定處理常式劫持 {#protocol_handler_hijack}

**檢查 ID：** `protocol_handler_hijack` · **檢查目錄 ATT&CK：** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei 檢查的內容：**讀取 http、https、mailto、ms-msdt 與 ms-officecmd 的 URL 開啟命令，以及找到的 `URL Protocol` 註冊與適用的使用者覆寫。

**檢查的證據：**

- 機器 `HKLM\SOFTWARE\Classes` 與使用者 UsrClass.dat 類別存放區：`URL Protocol` 標記與 `<scheme>\shell\open\command` 預設值。

**調查方式：**瀏覽器與應用程式安裝程式會合法地註冊 URL 配置。請追查命令與參數、評估該配置預期的用途，並與 URL 開啟／處理程序證據交叉比對。

### 資源回收筒 COM 擴充處理常式 {#recycle_bin_com_extension}

**檢查 ID：** `recycle_bin_com_extension` · **檢查目錄 ATT&CK：** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei 檢查的內容：**讀取所列出的資源回收筒 CLSID 的 open/empty/explore 命令覆寫，以及直接子層的內容功能表／拖放處理常式註冊。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\empty\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\explore\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\ContextMenuHandlers\<child>` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\DragDropHandlers\<child>` | `(Default)` |

**調查方式：**與核准的 Shell 擴充功能進行比較。部分回報的值是 COM 識別碼，而非直接解析出的函式庫；在主張程式碼已執行之前，請先調查所參照的類別與使用者動作。

### 搜尋通訊協定處理常式劫持 {#search_protocol_handler}

**檢查 ID：** `search_protocol_handler` · **檢查目錄 ATT&CK：** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei 檢查的內容：**讀取機器／使用者 `search-ms\shell\open\command` 的預設處理常式。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Classes\search-ms\shell\open\command` | `(Default)` |
| `HKU\<user>\Software\Classes\search-ms\shell\open\command` | `(Default)` |

**調查方式：**變更處理常式可能改變開啟 search-ms URL 時的行為，但它的存在並不能證明曾投遞漏洞利用。請與 Windows 預設值進行比較，並調查會使用它的 URL／處理程序活動。

### SharedTaskScheduler {#shared_task_scheduler}

**檢查 ID：** `shared_task_scheduler` · **檢查目錄 ATT&CK：** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei 檢查的內容：**列舉 Explorer `SharedTaskScheduler` 中的非空白值，視為已註冊的 Shell 元件。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SharedTaskScheduler` | 所有非空白的值 |

**調查方式：**這些值可能參照 COM 類別，而非執行檔路徑。請追查其伺服器註冊，並確認作業系統行為、擴充功能來源以及相關的 Shell 載入證據。

### ShellExecuteHooks {#shell_execute_hooks}

**檢查 ID：** `shell_execute_hooks` · **檢查目錄 ATT&CK：** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei 檢查的內容：**列舉 Explorer `ShellExecuteHooks` 中的非空白值，涵蓋舊版的 Shell 執行掛鉤註冊。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ShellExecuteHooks` | 所有非空白的值 |

**調查方式：**解析任何被參照的 CLSID，並確認該 Windows 組建與原則是否允許此掛鉤。舊版註冊可能未被使用；此值本身並非觀察到的 Shell 執行。

### 「開始」功能表媒體櫃與 CLSID 資料夾 COM 持續存取 {#start_menu_com}

**檢查 ID：** `start_menu_com` · **檢查目錄 ATT&CK：** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei 檢查的內容：**在
`Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu` 下，尋找其 `simpleLocation` URL 參照
COM CLSID 的 `.library-ms` 媒體櫃 XML 檔案，以及名為 `<name>.{CLSID}` 的目錄。Explorer 與這些
物件的互動可能會啟用 COM 程式碼。此檢查會納入已擷取的空目錄；它不會追蹤
符號連結。位於「開始」功能表之外的一般媒體櫃檔案，以及 XML 註解中的 GUID，不會由此檢查
回報。

**檢查的證據：**

- 每位使用者「開始」功能表中的 `.library-ms` 檔案與以 CLSID 命名的目錄項目。
- 相符使用者已擷取的類別註冊與機器 SOFTWARE，依架構分別處理。
- 受控碼的 CodeBase／AppID／服務設定，透過與排定工作相同的解析器處理。

**調查方式：**`Path` 與 `SourceFile` 可識別觸發啟用的跡證。`Value`
包含候選的伺服器命令；若無法解析，則為 CLSID。`Description`
將觸發項目連結到 CLSID 與已擷取的登錄來源。`ResolvedPath`/`Sha256` 只描述
已解析且已收集的酬載。只有執行檔名稱的 AppID 關聯會保持未解析。
觸發項目可能是合法的；它的存在並不能證明曾有登入、啟用、執行或攻擊。
請檢查媒體櫃 URL、類別註冊、酬載、使用者脈絡以及獨立的執行記錄。

**收集注意事項：**僅收集檔案的方式可能會遺漏空目錄。此檢查只檢視標準的使用者「開始」功能表
路徑；重新導向的「開始」功能表與其他 Explorer 命名空間位置不在此檢查的
涵蓋範圍內。缺少登錄區或未解析的註冊會產生部分涵蓋範圍，但仍會保留
候選項目。工作的架構、權限提升與原則可能會改變實際的 COM 搜尋行為。

**進一步資訊：**[原始提案與研究者致謝](https://github.com/last-byte/PersistenceSniper/pull/32)、
[COM 每位使用者／機器的合併檢視](https://learn.microsoft.com/en-us/windows/win32/sysinfo/merged-view-of-hkey-classes-root)、
[MITRE COM 劫持](https://attack.mitre.org/techniques/T1546/015/)。

### TypeLib COM 劫持 {#typelib_hijack}

**檢查 ID：** `typelib_hijack` · **檢查目錄 ATT&CK：** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei 檢查的內容：**讀取每位使用者 UsrClass.dat 中 TypeLib 的 GUID／版本／地區設定 win32/win64 註冊，保留路徑與指令碼 Moniker 值。尾端的數值資源索引只會在解析目標檔案時移除。

**檢查的證據：**

- 每個已擷取 UsrClass.dat 中 `TypeLib\<GUID>\<version>\<locale>\win32` 與 `win64` 的預設值。

**調查方式：**型別程式庫可能指向合法的 DLL、替代檔案或指令碼 Moniker。此檢查清查這些註冊；請評估受影響的應用程式並佐證型別程式庫的載入，而不要假設每個非預設路徑都是攻擊。

## Office 應用程式與增益集 { #office-applications-and-add-ins }

Office 支援合法的增益集、範本與應用程式特定的擴充功能。可疑的註冊可能取決於特定的 Office 版本、應用程式、載入設定或使用者互動。單憑設定並不能確立巨集曾執行。

### Office 增益集註冊 {#office_addins}

**檢查 ID：** `office_addins` · **檢查目錄 ATT&CK：** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Juntei 檢查的內容：**列舉未標示版本以及 14.0/15.0/16.0 Office 根目錄中的 Word、Excel、PowerPoint、Outlook 與 Access 增益集，包括已實作的機器隨選即用（Click-to-Run）存放區與使用者根目錄。它會保留明確的 Manifest/FileName/Path 值，或解析符合條件、以 COM 為基礎的項目，並附上 LoadBehavior 脈絡。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` 以及已實作的機器隨選即用對應位置。
- NTUSER.DAT 中的 `HKU\<user>\Software\Microsoft\Office\[<version>\]<application>\Addins\<add-in>`；以及來自 SOFTWARE、NTUSER.DAT 與 UsrClass.dat 的相符機器／使用者 COM 類別。

**調查方式：**增益集是常見的商業軟體。請檢視 LoadBehavior、應用程式／版本、資訊清單、實際的酬載以及預期的廠商。此檢查並不保證每種已安裝的增益集格式或註冊檢視都已被收集或載入。

### Office AI 增益集劫持 {#office_ai_hijack}

**檢查 ID：** `office_ai_hijack` · **檢查目錄 ATT&CK：** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Juntei 檢查的內容：**列舉 Office 16.0 隨選即用 Common AI 註冊位置中的值。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI`：所有值。

**調查方式：**將不尋常的值與路徑，與已安裝的 Office 組建進行比較。值的存在並不能顯示是哪個 Office 功能使用了它，也無法證明它選用了可執行的程式碼。

### Office DLL 覆寫 {#office_dll_override}

**檢查 ID：** `office_dll_override` · **檢查目錄 ATT&CK：** [T1137](https://attack.mitre.org/techniques/T1137/)

**Juntei 檢查的內容：**讀取 Word／PowerPoint 根目錄下，未標示版本以及 14.0/15.0/16.0 設定中的 `WwlibtDll` 與 `PPCoreTDLL`，包括已實作的隨選即用與使用者根目錄。

**檢查的證據：**

- 機器／使用者的 `Microsoft\Office\[<version>\]Word` 與 `PowerPoint` 設定，以及已實作的機器隨選即用 Office 樹狀結構：`WwlibtDll` 與 `PPCoreTDLL`。

**調查方式：**請判斷哪個應用程式／組建會辨識此覆寫、檢查函式庫及其來源，並與 Office 啟動或映像載入交叉比對。單憑已設定的覆寫並不足以證明執行。

### Office 預設範本 {#office_templates}

**檢查 ID：** `office_templates` · **檢查目錄 ATT&CK：** [T1137.001](https://attack.mitre.org/techniques/T1137/001/)

**Juntei 檢查的內容：**清查已收集的 Normal.dotm/NormalEmail.dotm 檔案，以及直接位於可辨識的 Word Startup 與 Excel XLSTART 資料夾（包括 Office 安裝目錄中的啟動資料夾）中的檔案。

**檢查的證據：**

- `Users\<user>\AppData\Roaming\Microsoft\Templates\Normal.dotm` 與 `NormalEmail.dotm`。
- 直接位於 Roaming AppData 下使用者 `Microsoft\Word\STARTUP` 與 `Microsoft\Excel\XLSTART` 中的檔案。
- 直接位於 Program Files 與 Program Files (x86) 下可辨識的 Office `<version>\STARTUP` / `XLSTART` 與 `root\<version>\STARTUP` / `XLSTART` 資料夾中的檔案。

**調查方式：**範本可能是合法的，也可能不含任何可執行的內容。Juntei 在此檢查中不會擷取或執行巨集。請檢查文件內部結構、巨集原則與應用程式活動；機器的安裝路徑並不代表 SYSTEM 處理程序權杖。

### Office Test DLL {#office_test_dll}

**檢查 ID：** `office_test_dll` · **檢查目錄 ATT&CK：** [T1137.002](https://attack.mitre.org/techniques/T1137/002/)

**Juntei 檢查的內容：**從所列出的機器／使用者以及 32 位元機器檢視中，列舉 Office Test Special Perf 機碼中的值。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Office Test\Special\Perf` | 所有非空白的值 |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Office Test\Special\Perf` | 所有非空白的值 |
| `HKU\<user>\SOFTWARE\Microsoft\Office Test\Special\Perf` | 所有非空白的值 |

**調查方式：**請將非預期的覆寫視為需要驗證 Office 版本與酬載的線索。此檢查回報的是設定，而不是測試特定的 Office 安裝是否會載入該值。

### Outlook 首頁攻擊 {#outlook_home_page}

**檢查 ID：** `outlook_home_page` · **檢查目錄 ATT&CK：** [T1137.004](https://attack.mitre.org/techniques/T1137/004/)

**Juntei 檢查的內容：**讀取 Office 14.0、15.0 與 16.0 版使用者 Outlook WebView 子項下的 `URL`。

**檢查的證據：**

- NTUSER.DAT 中的 `HKU\<user>\Software\Microsoft\Office\<14.0|15.0|16.0>\Outlook\WebView\<folder>\URL`。

**調查方式：**資料夾首頁可能是合法的，而其行為會因 Outlook 版本與安全性更新而異。請檢查 URL／內容與實際生效的原則，然後與 Outlook／網頁／處理程序證據交叉比對。

### VBA Monitor DLL 劫持 {#vba_monitors}

**檢查 ID：** `vba_monitors` · **檢查目錄 ATT&CK：** [T1137](https://attack.mitre.org/techniques/T1137/)

**Juntei 檢查的內容：**讀取機器與每位使用者類別存放區中，兩個已實作的 VBA Monitor 相關 CLSID 的 InprocServer32 值。

**檢查的證據：**

- 機器 SOFTWARE 中的 `Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32` 與 `{0002E157-0000-0000-C000-000000000046}`，以及使用者 UsrClass.dat 中對應的註冊。

**調查方式：**將所選的函式庫與已安裝的 Office/VBA 環境進行比較。這是針對特定 CLSID 的檢查，並非所有 VBA 擴充功能的清冊，也無法證明巨集曾執行。

## 偵錯工具、當機處理常式與相容性填充碼 { #debuggers-crash-handlers-and-compatibility-shims }

偵錯工具或錯誤處理常式可能會在應用程式啟動、當機、停止回應或結束時執行。應用程式相容性資料庫也可能影響啟動。請連同所回報的命令一併調查先決設定與觸發的程式。

### AeDebug 偵錯工具劫持 {#ae_debug}

**檢查 ID：** `ae_debug` · **檢查目錄 ATT&CK：** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei 檢查的內容：**讀取原生與 32 位元 AeDebug 中，Just-In-Time 當機偵錯設定所使用的 `Debugger` 命令。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |

**調查方式：**開發／偵錯軟體可能會合法地設定此值。請檢查完整的偵錯工具命令與當機觸發的先決條件，然後與應用程式錯誤及處理程序事件交叉比對。

### AeDebug 受保護處理程序偵錯工具 {#ae_debug_protected}

**檢查 ID：** `ae_debug_protected` · **檢查目錄 ATT&CK：** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei 檢查的內容：**讀取原生與 32 位元 AeDebugProtected 中，用於受保護處理程序偵錯攻擊面的 `Debugger` 命令。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |

**調查方式：**請判斷作業系統組建與保護原則是否能使用所設定的命令。已儲存的偵錯工具並不能證明受保護的處理程序曾當機，或該命令曾執行。

### Image File Execution Options 偵錯工具 {#ifeo_debugger}

**檢查 ID：** `ifeo_debugger` · **檢查目錄 ATT&CK：** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei 檢查的內容：**在原生／32 位元 Image File Execution Options 註冊（包括巢狀篩選機碼）中搜尋 `Debugger` 值。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\Debugger`，包括巢狀篩選機碼以及機器的 `Wow6432Node` 檢視。

**調查方式：**啟動指定的映像時，可能會選用偵錯工具。請檢查映像／篩選範圍與偵錯工具參數；開發人員工具也可能使用 IFEO。請追查目標的啟動以確立是否執行。

### IFEO 委派 NTDLL {#ifeo_delegated_ntdll}

**檢查 ID：** `ifeo_delegated_ntdll` · **檢查目錄 ATT&CK：** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei 檢查的內容：**在原生／32 位元 Image File Execution Options 註冊（包括巢狀機碼）中搜尋 `VerifierDlls` 覆寫。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\VerifierDlls`，包括巢狀機碼以及機器的 `Wow6432Node` 檢視。

**調查方式：**此檢查的判斷條件是已設定的 VerifierDlls 值；它無法證明所有驗證器／委派的先決條件都已啟用。在推斷存在可行的注入路徑之前，請先檢視適用的旗標、Windows 版本與酬載。

### 靜默處理程序結束監視器 {#ifeo_silent_process_exit}

**檢查 ID：** `ifeo_silent_process_exit` · **檢查目錄 ATT&CK：** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei 檢查的內容：**在原生／32 位元 SilentProcessExit 註冊（包括巢狀機碼）中搜尋 `MonitorProcess`。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\<image>\...\MonitorProcess`，包括巢狀機碼以及機器的 `Wow6432Node` 檢視。

**調查方式：**靜默結束監視也需要相關的報告／IFEO 設定。請檢查這些設定以及受監視映像的結束事件；合法的診斷工具可能可以解釋此監視器。

### 應用程式填充碼 {#installed_sdb}

**檢查 ID：** `installed_sdb` · **檢查目錄 ATT&CK：** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Juntei 檢查的內容：**讀取已安裝應用程式相容性資料庫的直接子層 InstalledSDB 註冊中的 `DatabasePath`。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\<child>` | `DatabasePath` |

**調查方式：**相容性修正可能是合法的。請檢查 SDB 內容、目標執行檔與安裝歷程。資料庫註冊並不會指出其中的填充碼，也無法證明曾有相符的應用程式執行。

### LSM 偵錯工具劫持 {#lsm_debugger}

**檢查 ID：** `lsm_debugger` · **檢查目錄 ATT&CK：** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei 檢查的內容：**在原生與 32 位元機器檢視中，專門讀取 `lsm.exe` 的 SilentProcessExit 下的 `MonitorProcess`。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |

**調查方式：**此項目與範圍更廣的 SilentProcessExit 清冊重疊。在將此設定視為已執行的持續存取機制之前，請先確認先決旗標以及相關的 LSM 結束事件；評估不同的發現項目時，同一個項目只計算一次。

### MiniDump 輔助 DLL {#minidump_auxiliary_dlls}

**檢查 ID：** `minidump_auxiliary_dlls` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**列舉 MiniDumpAuxiliaryDlls 中具名且非空白的值，將值名稱保留為觸發模組的脈絡，並將值資料保留為所選的目標。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls`：觸發模組的值名稱與目標資料。

**調查方式：**請同時檢查觸發模組與輔助 DLL。當機／小型傾印工具可能可以解釋某個項目，而註冊並不能顯示傾印作業曾載入該函式庫。

### 應用程式填充碼 {#shim_custom}

**檢查 ID：** `shim_custom` · **檢查目錄 ATT&CK：** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Juntei 檢查的內容：**讀取 AppCompatFlags Custom 直接子層應用程式機碼下以 `.sdb` 命名的值，保留應用程式與資料庫之間的關聯。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<application>`：名稱以 `.sdb` 結尾的值。

**調查方式：**請將這些關聯與 InstalledSDB 以及實際的相容性資料庫搭配檢視。單憑檔案名稱關聯並無法揭示資料庫的行為，也無法證明曾有相符的應用程式啟動。

### WER 偵錯工具劫持 {#wer_debugger}

**檢查 ID：** `wer_debugger` · **檢查目錄 ATT&CK：** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei 檢查的內容：**讀取原生／32 位元 Windows Error Reporting 的 `Debugger` 值。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `Debugger` |

**調查方式：**合法的診斷軟體可能會設定偵錯工具。請調查完整的命令與相關的當機事件；註冊並不能證明 Windows 錯誤報告曾啟動它。

### WER Hangs 偵錯工具劫持 {#wer_hangs}

**檢查 ID：** `wer_hangs` · **檢查目錄 ATT&CK：** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei 檢查的內容：**讀取原生／32 位元 Windows Error Reporting Hangs 的 `Debugger` 值。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |

**調查方式：**確認停止回應的處理流程是否會叫用此設定，並檢查目標程式。請與應用程式停止回應及處理程序記錄交叉比對，而不要將登錄寫入視為偵錯工具已執行。

### WER ReflectDebugger 劫持 {#wer_reflect_debugger}

**檢查 ID：** `wer_reflect_debugger` · **檢查目錄 ATT&CK：** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei 檢查的內容：**讀取原生／32 位元 Windows Error Reporting 的 `ReflectDebugger` 值。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |

**調查方式：**與核准的診斷設定進行比較，並確立 Windows 版本／觸發條件的適用性。在歸因為已執行之前，請檢查命令參數並尋求當機／處理程序方面的佐證。

### WER 執行階段例外狀況模組 {#wer_runtime_exception}

**檢查 ID：** `wer_runtime_exception` · **檢查目錄 ATT&CK：** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei 檢查的內容：**列舉原生／32 位元 RuntimeExceptionHelperModules 下的值名稱；在此，函式庫路徑存放在值名稱中，而不一定在其資料中。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules` 以及機器的 `Wow6432Node` 檢視：值名稱。

**調查方式：**請檢查所指名的模組以及應用程式／執行階段脈絡。資料中的小數值並不是酬載路徑，而單憑註冊也無法證明協助程式曾被載入。

## .NET 執行階段與分析工具設定 { #net-runtime-and-profiler-settings }

受控執行階段可以在應用程式程式碼之前載入分析工具（profiler）、啟動組件或偵錯函式庫。開發人員與監控產品也會使用這些設定。請檢查執行階段、處理程序架構、啟用旗標，以及新啟動的處理程序是否可能繼承此設定。

### AppDomainManager 注入 {#appdomain_manager}

**檢查 ID：** `appdomain_manager` · **檢查目錄 ATT&CK：** [T1574.014](https://attack.mitre.org/techniques/T1574/014/)

**Juntei 檢查的內容：**讀取機器／使用者的 `APPDOMAIN_MANAGER_ASM` 與 `APPDOMAIN_MANAGER_TYPE` 環境變數，涵蓋自訂 .NET AppDomain 管理員的組件與型別選擇。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_TYPE` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_TYPE` |

**調查方式：**應用程式可能會刻意使用自訂管理員。請一併解析組件與型別，並確認執行階段相容性與處理程序繼承；單一變數或其機碼時間戳記都無法證明受控應用程式曾載入它。

### .NET CLR 分析工具劫持 {#cor_profiler}

**檢查 ID：** `cor_profiler` · **檢查目錄 ATT&CK：** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei 檢查的內容：**讀取機器／使用者的 COR 分析環境設定，以及個別服務 Environment 多字串中相符的具名項目。它會保留啟用旗標、分析工具識別碼與特定架構的路徑。

**檢查的證據：**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` 以及每個已擷取使用者 NTUSER.DAT 的 `Environment`。
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`：多字串中的具名 COR 設定。

**調查方式：**分析工具支援合法的監控。啟用旗標、CLSID 與 DLL 路徑是不同的設定元素；請確認實際的啟用情形與相關的 .NET 處理程序，而不要將每一列都視為已執行的分析工具。

### .NET Core CLR 分析工具劫持 {#coreclr_profiler}

**檢查 ID：** `coreclr_profiler` · **檢查目錄 ATT&CK：** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei 檢查的內容：**讀取對應的機器／使用者 CORECLR 分析環境設定以及相符的服務 Environment 項目，包括特定架構的路徑。

**檢查的證據：**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` 以及每個已擷取使用者 NTUSER.DAT 的 `Environment`。
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`：多字串中的具名 CORECLR 設定。

**調查方式：**請驗證 .NET 執行階段、架構、啟用旗標與函式庫。核准的可觀測性產品可能會設定這些值，而既有的處理程序可能不會繼承新變更的環境。

### .NET DbgManagedDebugger 劫持 {#dotnet_dbg_managed_debugger}

**檢查 ID：** `dotnet_dbg_managed_debugger` · **檢查目錄 ATT&CK：** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei 檢查的內容：**讀取所列出的機器／使用者 `.NETFramework` `DbgManagedDebugger` 設定，包括明確的機器 32 位元路徑。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `DbgManagedDebugger` |

**調查方式：**請判斷適用的 .NET 執行階段／偵錯原則是否會使用此命令。開發人員工具可能可以解釋它；請與受控程式碼當機及處理程序建立交叉比對以確立是否執行。

### .NET Framework COR_PROFILER 登錄機碼 {#dotnet_framework_profiler}

**檢查 ID：** `dotnet_framework_profiler` · **檢查目錄 ATT&CK：** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei 檢查的內容：**從所列出的 `.NETFramework` 登錄檢視讀取 `COR_PROFILER`、`COR_PROFILER_PATH` 與 `COR_ENABLE_PROFILING`。這可補足以環境變數為基礎的分析工具檢查。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |

**調查方式：**請一併評估這些值，並判斷執行階段是否會採用該登錄位置。已設定的分析工具可能是合法的；不要將單獨的啟用旗標解讀為已確認的 DLL 載入。

### DOTNET_STARTUP_HOOKS {#dotnet_startup_hooks}

**檢查 ID：** `dotnet_startup_hooks` · **檢查目錄 ATT&CK：** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei 檢查的內容：**讀取機器／使用者的 `DOTNET_STARTUP_HOOKS` 環境值，此值可為支援的 .NET 執行階段選擇啟動掛鉤組件。

**檢查的證據：**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment\DOTNET_STARTUP_HOOKS` 以及每位使用者 NTUSER.DAT 的 `Environment\DOTNET_STARTUP_HOOKS`。

**調查方式：**請檢查整個值是否包含多個組件，並確認執行階段的支援情形。此檢查不會列舉服務特定的啟動掛鉤環境，也無法證明任何處理程序曾繼承並使用該值。

### 已知的受控偵錯 DLL {#known_managed_debugging_dlls}

**檢查 ID：** `known_managed_debugging_dlls` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取 `.NETFramework` 的 `KnownManagedDebuggingDlls` 註冊，視為可能的受控偵錯工具函式庫覆寫。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `KnownManagedDebuggingDlls` |

**調查方式：**與已安裝的開發／執行階段工具進行比較，並判斷觸發的偵錯工具。登錄項目並不能證明曾有偵錯工作階段或函式庫載入。

## DLL 與執行檔搜尋覆寫 { #dll-and-executable-lookup-overrides }

這些檢查會檢視明確的函式庫位置，或影響元件尋找程式碼方式的設定。路徑覆寫或被削弱的搜尋原則是調查線索；要達成執行，仍需要酬載以及相容的觸發元件。

### AppCert DLL {#appcert_dlls}

**檢查 ID：** `appcert_dlls` · **檢查目錄 ATT&CK：** [T1546.009](https://attack.mitre.org/techniques/T1546/009/)

**Juntei 檢查的內容：**列舉 Session Manager 中非空白的 AppCertDlls 註冊，這是與處理程序建立 API 相關聯的函式庫擴充攻擊面。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\AppCertDlls` | 所有非空白的值 |

**調查方式：**檢查每個函式庫及其安裝來源。請另外判斷適用性與執行期載入情形；時間軸中正常的處理程序建立本身並不能證明所設定的 DLL 曾被載入。

### AppInit DLL {#appinit_dlls}

**檢查 ID：** `appinit_dlls` · **檢查目錄 ATT&CK：** [T1546.010](https://attack.mitre.org/techniques/T1546/010/)

**Juntei 檢查的內容：**讀取原生／32 位元的 `AppInit_DLLs`，拆分其函式庫清單，並為每個候選項目加上可取得的 `LoadAppInit_DLLs` 與 `RequireSignedAppInit_DLLs` 脈絡。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows`：`AppInit_DLLs`、`LoadAppInit_DLLs` 與 `RequireSignedAppInit_DLLs`；以及機器的 `Wow6432Node` 檢視。

**調查方式：**LoadAppInit_DLLs 旗標為零時會被明確標示為未生效。Windows 版本、安全開機與簽署要求也會影響載入。在主張 AppInit DLL 已執行之前，請先檢視實際生效的設定。

**進一步資訊：**[Microsoft 說明文件](https://learn.microsoft.com/en-us/windows/win32/dlls/secure-boot-and-appinit-dlls)。

### AutodialDLL 覆寫 {#autodial_dll}

**檢查 ID：** `autodial_dll` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取 WinSock2 Parameters 的 `AutodialDLL` 覆寫。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters` | `AutodialDLL` |

**調查方式：**確認使用此設定的網路功能與來源 Windows 版本。檢視實際的 DLL 以及網路／處理程序活動；單憑覆寫並不能確立作用中或惡意的載入。

### CHM 協助程式 DLL {#chm_helper_dll}

**檢查 ID：** `chm_helper_dll` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取 HtmlHelp Author 下的 `Location`，這是與說明相關的函式庫／位置覆寫攻擊面。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\HtmlHelp Author` | `Location` |

**調查方式：**請判斷此系統上相關的說明元件是否會使用此值，並檢查所選的檔案。登錄覆寫並不能證明曾開啟 CHM 文件。

### NL6 ContentIndex DLL 覆寫 {#content_index_dll}

**檢查 ID：** `content_index_dll` · **檢查目錄 ATT&CK：** [T1574](https://attack.mitre.org/techniques/T1574/)

**Juntei 檢查的內容：**讀取作用中控制集內直接子層 ContentIndex Language 註冊中的 `DLLOverridePath`。

**檢查的證據：**

- `HKLM\SYSTEM\<ControlSet>\Control\ContentIndex\Language\<language>\DLLOverridePath`。

**調查方式：**語言／搜尋元件可能因 Windows 組建而異。請檢查函式庫與相符的已安裝元件；追查實際的索引處理程序以佐證載入。

### Crypto ExpoOffload DLL {#crypto_expo_offload}

**檢查 ID：** `crypto_expo_offload` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取 Cryptography Offload 的 `ExpoOffload` 值，視為密碼編譯函式庫覆寫的候選項目。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Cryptography\Offload` | `ExpoOffload` |

**調查方式：**確認功能／組建的適用性，並調查所選函式庫的來源。不要將不常見值的存在視為認證竊取或程式碼已執行的證據。

### Direct3D 軟體點陣化器 DLL {#direct3d_dll}

**檢查 ID：** `direct3d_dll` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取 Direct3D Drivers 的 `SoftwareRasterizer` 函式庫覆寫。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Direct3D\Drivers` | `SoftwareRasterizer` |

**調查方式：**圖形與相容性軟體可能會影響點陣化器設定。請檢查該 DLL 以及相容 Direct3D 應用程式的活動；儲存路徑並不能證明它曾被載入。

### DLL 搜尋順序弱化 {#dll_search_mode}

**檢查 ID：** `dll_search_mode` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**只有在作用中的 Session Manager 值剖析為零時，才會回報 `SafeDllSearchMode` 或 `CWDIllegalInDllSearch`。這些是搜尋原則的狀況，而非選定的酬載。

**檢查的證據：**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager`：`SafeDllSearchMode` 與 `CWDIllegalInDllSearch`。

**調查方式：**被削弱的搜尋原則可能使另一個搜尋順序攻擊更容易得逞。它並不會指出被植入的 DLL，也無法證明執行。此檢查不會輸出非零或不存在的值；請另外評估實際的預設值與原則。

**進一步資訊：**[Microsoft 說明文件](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security)。

### KnownDLLs 排除 {#exclude_from_known_dlls}

**檢查 ID：** `exclude_from_known_dlls` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取 Session Manager 中非空白的 `ExcludeFromKnownDlls` 項目，視為對受保護／已知 DLL 搜尋行為的變更。

**檢查的證據：**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\ExcludeFromKnownDlls`。

**調查方式：**排除項目指出的是搜尋狀況，而非替換的函式庫本身。請找出受影響的應用程式與可能的 DLL 搜尋位置，然後佐證非標準的映像載入。

**進一步資訊：**[Microsoft 說明文件](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order)。

### hhctrl.ocx DLL 覆寫 {#hhctrl_ocx_dll}

**檢查 ID：** `hhctrl_ocx_dll` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**針對已實作的 HTML Help 控制項 CLSID，讀取原生／32 位元機器以及支援的使用者類別存放區中的 InprocServer32 覆寫。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKLM\SOFTWARE\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\Software\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |

**調查方式：**請判斷是哪個應用程式與架構會啟用此控制項。檢查已註冊的 DLL 與說明相關的活動；COM 註冊並不能證明 HTML Help 曾轉譯內容。

### Known DLLs {#known_dlls}

**檢查 ID：** `known_dlls` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**列舉作用中 Session Manager KnownDLLs 機碼中的具名值。

**檢查的證據：**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\KnownDLLs`：具名值。

**調查方式：**許多項目都是預期中的 Windows 函式庫，而部分值是輔助設定而非酬載路徑。請將整個機碼與可信且相符的組建進行比較；此檢查不會掃描每個替代的特定架構 KnownDLLs 樹狀結構。

### LSA 擴充 DLL {#lsa_extensions}

**檢查 ID：** `lsa_extensions` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取 LsaExtensionConfig LsaSrv 設定中的 `Extensions`。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\LsaExtensionConfig\LsaSrv` | `Extensions` |

**調查方式：**資安軟體可能會擴充驗證元件。請檢查實際的函式庫及其與來源 Windows 版本的相容性，然後尋找 LSASS 映像載入或其他獨立的使用證據。

### MAPI32 DLL 路徑覆寫 {#mapi32_dll_path}

**檢查 ID：** `mapi32_dll_path` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取原生與 32 位元機器檢視中，直接子層郵件用戶端註冊下的 `DLLPathEx` 與 `DLLPath`。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPath` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPath` |

**調查方式：**郵件用戶端會註冊合法的 MAPI 提供者。請判斷所選的用戶端／提供者並檢查實際的函式庫；已安裝的提供者並不能證明事件期間曾有應用程式使用 MAPI。

### MSDTC XA DLL {#msdtc_xa_dll}

**檢查 ID：** `msdtc_xa_dll` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取 MSDTC MTxOCI 設定中的 `OracleXaLib` 與 `OracleOciLib`。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleXaLib` |
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleOciLib` |

**調查方式：**資料庫整合可能合法地需要這些函式庫。請確認交易協調器設定與已安裝的資料庫軟體，並與服務／模組的使用交叉比對，而不要假設所設定的路徑曾執行。

### NLDP DLL 覆寫 {#nldp_dll}

**檢查 ID：** `nldp_dll` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取 NlsData 的 `DllOverridePath` 設定值。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NlsData` | `DllOverridePath` |

**調查方式：**請檢視目標函式庫、Windows 組建，以及可能使用此覆寫的自然語言元件。此值並不能顯示曾發生索引或語言處理。

### RDP TestDVCPlugin DLL {#rdp_test_dvc_plugin}

**檢查 ID：** `rdp_test_dvc_plugin` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取機器 Terminal Server Client 的 `TestDVCPlugin` 函式庫設定。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Terminal Server Client` | `TestDVCPlugin` |

**調查方式：**請調查陌生的測試外掛程式以及版本特定的行為。要確立執行，需要相容的用戶端工作階段與函式庫載入；單憑登錄值無法顯示其中任何一項。

### Search Indexer DLL 覆寫 {#search_indexer_dll}

**檢查 ID：** `search_indexer_dll` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**從 SOFTWARE 讀取 Windows Search 的 `DllPath` 覆寫。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Search` | `DllPath` |

**調查方式：**請判斷相符的 Search 元件／版本並檢查目標檔案。未作用的 Windows Search 服務或未被使用的設定，與觀察到的惡意函式庫載入是不同的情況。

### Winsock AutoProxy DLL {#winsock_auto_proxy}

**檢查 ID：** `winsock_auto_proxy` · **檢查目錄 ATT&CK：** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei 檢查的內容：**讀取直接子層 NameSpace_Catalog5 Catalog_Entries 與 Catalog_Entries64 註冊下的 `LibraryPath`。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries\<child>` | `LibraryPath` |
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries64\<child>` | `LibraryPath` |

**調查方式：**網路提供者可能是合法的。請檢視兩種架構、提供者來源，以及相關的網路應用程式／映像載入證據。此針對命名空間提供者的查詢，並未涵蓋所有可能的 Winsock 擴充。

## 帳戶與登入可見性 { #accounts-and-sign-in-visibility }

本機帳戶記錄與登入可見性設定可以揭露以身分為基礎的持續存取機制。這些檢查會檢視已擷取的 SAM 與 SOFTWARE 登錄區；它們並非 Active Directory 帳戶、網域群組成員資格或過去登入的清冊。

### Administrators 群組成員資格 {#admin_group_membership}

**檢查 ID：** `admin_group_membership` · **檢查目錄 ATT&CK：** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei 檢查的內容：**從 SAM 解碼本機 Administrators 別名的成員清單。它會回報指向隱藏帳戶，或在已擷取的 SAM 中找不到對應帳戶／群組的本機 SID。

**檢查的證據：**

- SAM 登錄區中的 `SAM\Domains\Builtin\Aliases\00000220\C`，以及 `SAM\Domains\Account` 下的機器 SID／使用者／群組記錄。
- 用於隱藏名稱交叉比對的 `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`。

**調查方式：**這是針對特定異常的檢查，並非所有系統管理員的清單。外部／網域 SID 不會被評估為遺失的本機使用者。已刪除的帳戶、擷取時間點或不完整的登錄區都需要先調查，才能斷定成員資格具有惡意。

### 隱藏的本機帳戶 {#hidden_account}

**檢查 ID：** `hidden_account` · **檢查目錄 ATT&CK：** [T1564.002](https://attack.mitre.org/techniques/T1564/002/)

**Juntei 檢查的內容：**回報 Winlogon SpecialAccounts UserList 中數值為零的值，以識別從一般登入畫面中隱藏的名稱。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`：帳戶名稱值。

**調查方式：**服務或維護帳戶可能會被刻意隱藏，而此值並不能確立所指名的帳戶存在或具有系統管理員身分。請與 SAM 帳戶記錄及登入活動結合檢視。

### RID 劫持 {#rid_hijacking}

**檢查 ID：** `rid_hijacking` · **檢查目錄 ATT&CK：** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei 檢查的內容：**將 SAM 帳戶子機碼的十六進位 RID，與其 F 值中位移 0x30 處所儲存的 RID 進行比較，並回報不相符的情況。

**檢查的證據：**

- SAM 登錄區中的 `SAM\Domains\Account\Users\<hexadecimal RID>\F`。

**調查方式：**不相符是嚴重的帳戶記錄異常，但也可能需要進行損毀／擷取方面的分析。請檢查來源位元組、交易記錄狀態以及帳戶／登入記錄；它無法證明是哪個行為者修改了該記錄。

### RID Suborner（隱藏的系統管理員帳戶） {#rid_suborner}

**檢查 ID：** `rid_suborner` · **檢查目錄 ATT&CK：** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei 檢查的內容：**回報子機碼 RID 不是 500、但 F 值 RID 為 500 的 SAM 帳戶，這是更具體的系統管理員身分替換模式。

**檢查的證據：**

- SAM 登錄區中的 `SAM\Domains\Account\Users\<hexadecimal RID>\F`。

**調查方式：**此項目與一般的 RID 不相符檢查重疊。儘管檢查目錄的標題如此，它並不另外要求存在隱藏登入的設定；請另外確認帳戶可見性、權限與實際的帳戶使用情形。

## 驗證元件與認證保護 { #authentication-components-and-credential-protection }

本機安全性授權（LSA）透過 LSASS 及相關元件處理安全性功能。驗證套件、密碼篩選器與網路提供者可能是合法的資安軟體，也可能是持續存取認證資訊的途徑。其中兩項檢查清查的是保護設定，而非可執行的自動啟動項目。請依主機角色、Windows 版本與實際生效的原則來解讀它們。

### AMSI 提供者 DLL {#amsi_providers}

**檢查 ID：** `amsi_providers` · **檢查目錄 ATT&CK：** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei 檢查的內容：**列舉機器的 AMSI 提供者 CLSID，並在可用時解析其原生／32 位元 InprocServer32 函式庫。未解析的註冊會保留類別 ID。

**檢查的證據：**

- `HKLM\SOFTWARE\Microsoft\AMSI\Providers\<CLSID>` 以及可解析的 `Classes\CLSID` / `Classes\Wow6432Node\CLSID` InprocServer32 註冊。

**調查方式：**防毒與資安產品會合法地註冊 AMSI 提供者。請檢視提供者／函式庫的來源與實際載入情形；註冊並不能證明 AMSI 曾被繞過，或該提供者具有惡意。

### 驗證套件 {#authentication_packages}

**檢查 ID：** `authentication_packages` · **檢查目錄 ATT&CK：** [T1547.002](https://attack.mitre.org/techniques/T1547/002/)

**Juntei 檢查的內容：**從作用中控制集讀取 LSA `Authentication Packages` 中每個非空白的項目。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Authentication Packages` |

**調查方式：**與來源 Windows 組建及已安裝的驗證軟體進行比較。非預設的套件可能是合法的。在斷定有攔截行為之前，請先解析 DLL 並佐證載入／認證活動。

### DSRM 系統管理員登入行為 {#dsrm_backdoor}

**檢查 ID：** `dsrm_backdoor` · **檢查目錄 ATT&CK：** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei 檢查的內容：**回報已實作的 ControlSet001、ControlSet002 或 CurrentControlSet 位置中數值為 `DsrmAdminLogonBehavior=2` 的情況。此設定可能允許在一般復原情境之外使用 DSRM 帳戶。

**檢查的證據：**

- `HKLM\SYSTEM\ControlSet001\Control\Lsa\DsrmAdminLogonBehavior`，以及存在時的 ControlSet002 與 CurrentControlSet。

**調查方式：**主要與網域控制站相關。請使用 SYSTEM Select 判斷哪個控制集處於作用中；歷史控制集中的相符項目不一定實際生效。此值並不能證明攻擊者知道 DSRM 密碼或曾經登入。

### Credential Guard 設定 {#lsa_cfg_flags}

**檢查 ID：** `lsa_cfg_flags` · **檢查目錄 ATT&CK：** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Juntei 檢查的內容：**讀取所設定的 LSA `LsaCfgFlags` 值，包括一般的啟用值以及零。這是在清查 Credential Guard 設定，而非可執行的自動啟動項目。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `LsaCfgFlags` |

**調查方式：**請依 Windows 版本、韌體／安全性先決條件與實際生效的原則來解讀此值。單憑儲存的零值或值不存在，並不能確立攻擊者曾變更設定、實際的保護狀態或認證竊取。

### LSA 密碼篩選器 {#lsa_password_filter}

**檢查 ID：** `lsa_password_filter` · **檢查目錄 ATT&CK：** [T1556.002](https://attack.mitre.org/techniques/T1556/002/)

**Juntei 檢查的內容：**讀取 LSA 的 `Notification Packages`。單純的函式庫名稱會對照已擷取的 Windows System32 目錄進行解析，並在適當時加上 `.dll`。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Notification Packages` |

**調查方式：**密碼原則產品經常安裝篩選器。請檢查函式庫與載入證據，並區分已設定的篩選器與曾發生密碼變更或認證攔截的證據。

### LSASS PPL 保護狀態 {#lsa_run_as_ppl}

**檢查 ID：** `lsa_run_as_ppl` · **檢查目錄 ATT&CK：** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Juntei 檢查的內容：**讀取所設定的 LSA `RunAsPPL` 值，包括非零值。這是保護設定的清查，而非測試 LSASS 是否曾在未受保護的狀態下執行。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `RunAsPPL` |

**調查方式：**實際的 LSA 保護取決於 Windows 版本、原則、韌體鎖定與重新啟動狀態。值不存在時沒有輸出列，並不代表經過量測的保護結果。請使用獨立的系統／執行期證據評估實際的保護狀態。

### 網路提供者 DLL {#network_provider_dll}

**檢查 ID：** `network_provider_dll` · **檢查目錄 ATT&CK：** [T1556.008](https://attack.mitre.org/techniques/T1556/008/)

**Juntei 檢查的內容：**讀取作用中 SYSTEM 控制集內各服務下的 `NetworkProvider\ProviderPath`。

**檢查的證據：**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\NetworkProvider\ProviderPath`。

**調查方式：**網路登入提供者可能是合法的。請確認提供者順序／適用性、已安裝的網路／資安軟體以及實際的函式庫載入；單憑 ProviderPath 值並不能證明有認證攔截。

### 安全性套件 {#security_packages}

**檢查 ID：** `security_packages` · **檢查目錄 ATT&CK：** [T1547.005](https://attack.mitre.org/techniques/T1547/005/)

**Juntei 檢查的內容：**讀取 LSA `Security Packages` 中每個非空白的項目，涵蓋所設定的安全性支援提供者。

**檢查的證據：**

| 登錄機碼 | 檢查的值 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Security Packages` |

**調查方式：**與預期的 Windows／資安產品基準進行比較，並檢查每個函式庫。此設定並不能證明提供者曾被載入、曾擷取認證，或是在事件期間被新增。

## 從候選項目推導出結論 { #work-from-a-candidate-to-a-conclusion }

1. 開啟 `SourceFile` 所指出的原始來源。確認完整的值、XML 動作、
   指令碼或還原的資料庫記錄，以及它所屬的確切使用者／主機。
2. 判斷該機制是否能夠運作：檢查啟用旗標、執行階段或 Windows 版本、
   服務／工作狀態、所需的觸發條件、使用者、架構以及作用中的控制集。
3. 檢查所選的程式、DLL、指令碼或組件。對已收集的位元組計算雜湊值，並檢視參數
   與內容。可辨識的 Windows 啟動器或發行者字串並不能斷定其合法性。
4. 使用獨立證據確立實際發生的事情。例如處理程序建立、
   Sysmon 映像載入、工作排程器執行期記錄、服務活動、Prefetch 以及相關檔案。
   設定變更事件可以證明設定曾變更，但無法證明其酬載曾執行。
5. 使用原始來源的時間戳記以及所記錄的交叉比對來源脈絡建立時間軸。
   請將機碼寫入、安裝變更、檔案／容器寫入與執行視為各自獨立的事實。
6. 除了發現項目之外，也請記錄限制：缺少的登錄區、缺少的目標檔案、不明的工作主體、
   未經驗證的簽署者中繼資料、歷史上還原的記錄，或無法佐證的觸發條件。

關於命令語法、設定檔、每個輸出欄位與涵蓋範圍狀態，請回到
[Persistence Hunter 命令指南](../commands/persistence-hunter.md)。
[跡證指南](../artifacts/index.md)說明了底層的來源，而
[致謝](acknowledgements.md)則列出上游的持續存取偵測工具。
