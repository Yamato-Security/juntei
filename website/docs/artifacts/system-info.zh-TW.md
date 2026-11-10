# 系統資訊 { #system-information }

## 這是什麼 { #what-it-is }

系統的四個電腦登錄區，提供了分析人員在閱讀其時間軸之前所需的資訊：

- 其名稱（NetBIOS、DNS 主機名稱、DNS 網域），以及其所屬的網域或工作群組；
- 其 Windows 版本、組建與更新修訂版本，以及 Windows 安裝或最後一次升級的時間；
- 其時區，可據以得知系統所保存的當地時間；
- 最後一次正常關機的時間；
- 其處理器架構與字碼頁；
- 最後登入的使用者，以及每位曾登入之使用者的設定檔；
- 其 SID：電腦的 SID（本機帳戶即為它加上 RID）以及其網域的 SID；
- 系統升級之前的 Windows 版本。

`parse-registry system-info` 會為每個系統寫入一筆包含上述所有資訊的記錄。

<span class="evidence-strength">最佳結論：</span>收集登錄區（或其最後一次寫入）當時的設定狀態。
機碼時間則說明其中各部分最後一次變更的時間。

## 位置 { #location }

```text
C:\Windows\System32\config\SYSTEM     (the control set Windows used last: Select\Current)
  ControlSet00N\Control\ComputerName\ComputerName     ComputerName
  ControlSet00N\Services\Tcpip\Parameters             Hostname, Domain
  ControlSet00N\Control\TimeZoneInformation           TimeZoneKeyName, Bias, ActiveTimeBias, …
  ControlSet00N\Control\Windows                       ShutdownTime
  ControlSet00N\Control\Session Manager\Environment   PROCESSOR_ARCHITECTURE
  ControlSet00N\Control\Nls\CodePage                  ACP, OEMCP
  Setup\Source OS (Updated on M/D/YYYY HH:MM:SS)      the version before each upgrade

C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion                 ProductName, EditionID, DisplayVersion,
                                                      CurrentBuild, UBR, InstallDate, InstallTime, …
  Microsoft\Windows NT\CurrentVersion\ProfileList     one key per profile (SID)
  Microsoft\Windows\CurrentVersion\Authentication\LogonUI   LastLoggedOnUser, …

C:\Windows\System32\config\SAM
  SAM\Domains\Account                                 V: the computer's SID at its end

C:\Windows\System32\config\SECURITY
  Policy\PolAcDmN, PolAcDmS                           the computer's name and SID
  Policy\PolPrDmN, PolPrDmS                           its domain (or workgroup) and the domain's SID
  Policy\PolDnDDN, PolDnTrN, PolDnDmG                 the domain's and forest's DNS names, GUID
  Policy\PolMachineAccountS                           the computer account's SID in the domain
```

請將四個登錄區連同其交易記錄一併收集。任何一個登錄區單獨使用時，也能提供其所保存的資訊。

## 分析注意事項 { #analyst-cautions }

- **登錄檔時間為 UTC；時區則用於其他所有時間。** 機碼時間、FILETIME 以及大多數
  事件時間均為 UTC。文字記錄、部分應用程式記錄，以及寫入名稱或字串中的日期
  （例如 `Source OS (Updated on …)`）可能是當地時間。`Bias` 是從當地標準
  時間到 UTC 的分鐘數（`UTC = local + Bias`）。`UtcOffset` 與 `DaylightUtcOffset` 提供兩種時差，
  而 `TimeZoneDescription` 則說明日光節約時間的開始與結束時間。
- **時區可能被變更。** `TimeZoneLastWrittenUTC` 是該機碼的最後寫入時間。Windows 在更新
  `ActiveTimeBias`（當時生效的時差）時也會自行重寫此機碼，因此
  較近的時間並不能證明有人變更了時區。請與事件記錄比較：Security 4616
  （系統時間已變更）以及 System 記錄中的 `Microsoft-Windows-Kernel-General` 事件。
- **`StandardStart` 與 `DaylightStart` 是核心的 `TIME_FIELDS`，而非 `SYSTEMTIME`。**
  星期是其最後一個欄位，而非第三個欄位（後者為 `SOFTWARE` 中時區 `TZI` 值的格式）。在
  美國時區中，兩種配置讀起來相同，因為星期為星期日（0）。Juntei 將其解碼
  為 `TIME_FIELDS`：公開的復活節島系統（第一個星期六 22:00）即可看出
  差異。月份為 0 表示沒有日光節約時間。
- **Windows 11 顯示為「Windows 10」。** 在 Windows 11 上，`ProductName` 仍維持 `Windows 10 …`。組建 22000
  及之後為 Windows 11，摘要中也會如此標示。在 Windows 8.1、10
  與 11 上，`CurrentVersion` 均為 `6.3`；若存在 `CurrentMajorVersionNumber`（10），`Version` 會使用它。
- **`InstallDate` 會隨功能升級而變動。** 就地升級至新版本（例如 1809 升級至
  21H2）會重設此值。每次升級所取代的版本及其各自的 `InstallDate`，記錄於
  `PreviousVersions`（`Setup\Source OS`）中。其中最早的 `InstallDate` 最接近
  原始安裝時間。`Source OS` 機碼名稱中的日期並未指出其時區。
- **`ShutdownTime` 是最後一次正常關機。** 當機或斷電不會更新此值。
  請與事件記錄（6006、6008、1074）以及登錄區最後寫入的時間比較。
- **`LastLoggedOnUser` 是登入畫面的最後一位使用者，** 而非每一次登入。遠端桌面、
  `runas`、服務與排程工作並不一定會設定此值。`LogonUiLastWrittenUTC` 大約是其
  最後一次被設定的時間。
- **設定檔：** `ProfileList` 會列出曾在系統上擁有設定檔的每位使用者（及服務），
  已刪除的帳戶在其設定檔被移除之前也包含在內。Windows 10 及更新版本會寫入
  `LocalProfileLoadTime` 與 `LocalProfileUnloadTime`：大約是最後一次登入與登出的時間。
- **SID：** 本機帳戶為 `MachineSid-<RID>`，網域帳戶為 `DomainSid-<RID>`。
  `MachineSid` 取自 `SAM`（否則取自 `SECURITY` 的 `PolAcDmS`）。兩者應該相符；
  若不相符，`Problems` 會加以說明，就如同來自不同系統之登錄區的情況。在工作群組中，
  `DomainSid` 為空白。
- **一個目錄，一個系統。** 使用 `-d` 時，同一目錄中的登錄區視為同一個系統。
  `RegBack` 副本與同類型的第二個登錄區（`SYSTEM.old`）則各自成為獨立的記錄。
  使用 `-f` 時，所指定的登錄區視為同一個系統，其他登錄區則會在其旁邊尋找。
- **控制集：** 只會讀取 Windows 最後使用的控制集。`parse-registry read` 可顯示
  其他控制集。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM, SOFTWARE, SAM and SECURITY hives
  -f, --file <FILE>      A hive of one system; repeat for each
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry system-info -d collection/ -o system-info.csv
juntei parse-registry system-info -f SYSTEM -f SOFTWARE -t jsonl \
  | jq '{Host, TimeZoneDescription, ProductName, Build, LastShutdown}'
```

摘要會為每個系統列出：其 Windows 版本、安裝時間、以文字描述的時區、
機碼最後寫入時生效的時差、最後一次關機、其網域或
工作群組、最後登入的使用者、其 SID、設定檔數量，以及升級之前的
版本。`auto-parse` 在列出所找到的系統時，也會顯示每個系統的時區。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`, `SourcePath`, `SoftwarePath`, `SamPath`, `SecurityPath` | 主機，以及所讀取的登錄區（`SourcePath` 為 `SYSTEM` 登錄區，否則為第一個讀取的登錄區）。 |
| `CompName`, `Hostname`, `Domain` | NetBIOS 名稱（取自 `SYSTEM`，否則取自 `SECURITY` 的 `PolAcDmN`）、DNS 主機名稱與 DNS 網域（`Tcpip\Parameters`）。 |
| `PrimaryDomain`, `DNS-Domain` | 電腦所加入的網域，或其工作群組（`PolPrDmN`），以及網域的 DNS 名稱（`PolDnDDN`）。 |
| `ProductName`, `Edition`, `DisplayVersion`, `Version`, `Build`, `Ubr`, `ServicePack`, `InstallationType` | `CurrentVersion`：依原樣寫入的產品名稱、`EditionID`、`DisplayVersion`（否則為 `ReleaseId`）、主要與次要版本（否則為 `CurrentVersion`）、組建、更新組建修訂版本、`CSDVersion`，以及 `Client`、`Server` 或 `Server Core`。 |
| `InstallDateUTC`, `InstallTimeUTC` | `InstallDate`（精確到秒）與 `InstallTime`（精確到 100 ns，Windows 10 及更新版本）：安裝或最後一次功能升級的時間。 |
| `RegisteredOwner`, `RegisteredOrganization`, `SystemRoot` | 依原樣寫入。 |
| `TimeZone`, `TimeZoneDescription` | `TimeZoneKeyName`（否則為 `StandardName`），以及以文字描述的時區，含其時差與日光節約時間日期。 |
| `UTC-Offset`, `DaylightUTC-Offset` | 當地標準時間與 UTC 的時差，以及日光節約時間的時差（沒有日光節約時間時為空白）。 |
| `BiasMinutes`, `ActiveBiasMinutes`, `StandardBiasMinutes`, `DaylightBiasMinutes` | `Bias`、`ActiveTimeBias`（寫入機碼時生效的值）、`StandardBias`、`DaylightBias`。 |
| `StandardName`, `DaylightName` | 依原樣寫入，通常是資源參照（`@tzres.dll,-112`）。 |
| `StandardStart`, `DaylightStart` | 標準時間與日光節約時間的開始時間，以文字描述（`first Sunday of November, 02:00`）。 |
| `DynamicDaylightTimeDisabled`, `RealTimeIsUniversal` | 已關閉動態日光節約時間；硬體時鐘保存 UTC。 |
| `TimeZoneLastWrittenUTC` | 時區機碼的最後寫入時間。 |
| `LastShutdownUTC` | `ShutdownTime`：最後一次正常關機。 |
| `CurrentControlSet`, `ProcessorArchitecture`, `AnsiCodePage`, `OemCodePage` | 所讀取的控制集、`PROCESSOR_ARCHITECTURE`、`ACP` 與 `OEMCP`。 |
| `LastLoggedOnUser`, `LastLoggedOnSamUser`, `LastLoggedOnUserSID`, `LastLoggedOnDisplayName`, `LogonUiLastWrittenUTC` | 登入畫面的最後一位使用者，以及其機碼的最後寫入時間。 |
| `MachineSID`, `DomainSID`, `MachineAccountSID` | 電腦的 SID、其網域的 SID（`PolPrDmS`）以及電腦帳戶在網域中的 SID（`PolMachineAccountS`）。 |
| `CompNameLastWrittenUTC`, `PrimaryDomainLastWrittenUTC`, `VersionLastWrittenUTC` | 電腦最後一次命名、最後一次加入網域或工作群組，以及最後一次更新（`CurrentVersion` 的機碼）的大約時間。 |
| `PreviousVersions` | 每個 `Source OS` 機碼：版本、其安裝日期，以及機碼名稱中的日期（JSON：每個一個物件）。 |
| `Profiles` | 每個設定檔：SID、資料夾、機碼時間、最後載入與卸載時間（JSON：每個一個物件）。 |
| `TransactionLogStatus`, `LogsApplied` | 涵蓋全部四個登錄區的登錄檔復原脈絡。 |
| `Problems` | 類型或大小錯誤的值（保留其位元組），以及電腦 SID 不一致的 `SAM` 與 `SECURITY`。 |

## 調查策略 { #investigation-strategy }

1. 先讀取時區：記下 `UtcOffset`、日光節約時間日期，以及時區是否
   在近期被變更過。並據以轉換當地時間的記錄。
2. 記下版本、組建與安裝日期。檢查 `PreviousVersions` 中的升級紀錄，並將
   安裝日期與最舊的檔案系統及事件記錄時間比較。
3. 記下 `LastShutdownUTC`：若在此之後仍有活動，卻沒有新的關機時間，則暗示發生過當機、斷電
   或屬於執行中系統的收集。
4. 使用 `MachineSid` 與 `DomainSid`，在其他地方（BAM、
   `ProfileList`、事件記錄）的 SID 中區分本機帳戶與網域帳戶，並以設定檔的載入與卸載時間掌握最後幾次登入。
5. 將電腦與網域名稱與事件記錄交叉比對，並以 `CompNameLastWrittenUTC`
   與 `PrimaryDomainLastWrittenUTC` 找出重新命名與加入網域的情形。

## 延伸資訊 { #further-details }

- [Microsoft：TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft：DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft：TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft：安全性識別碼](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb：Time zones](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb：Windows product information](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
