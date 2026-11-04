# FeatureUsage、RecentApps 與工作列釘選項目 { #featureusage-recentapps-and-taskbar-pins }

## 這些是什麼 { #what-they-are }

檔案總管會在使用者的 `NTUSER.DAT` 中保存數種關於使用者如何使用應用程式的記錄：

- **FeatureUsage**（Windows 10 及更新版本）：依使用方式區分的各應用程式計數器。
  - `AppSwitched`：使用者切換到該應用程式視窗的次數（點擊工作列、
    Alt+Tab）。
  - `AppLaunch`：從工作列（釘選的項目）啟動該應用程式的次數。
  - `ShowJumpView`：開啟其跳躍清單的次數（在工作列上按右鍵）。
  - `AppBadgeUpdated`：其工作列徽章（未讀數）變更的次數。
  - `TrayButtonClicked`：各工作列按鈕（`StartButton`、`SearchButton`、
    `TaskViewButton`、`ClockButton`……）被點擊的次數。

  應用程式可以是應用程式使用者模型 ID（`Microsoft.Windows.Explorer`、
  `Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`）、第一段可能是已知資料夾 GUID 的
  路徑（`{1AC14E77-…}\cmd.exe` 即 `%windir%\system32\cmd.exe`），或是處理程序 ID
  （`*PID00001d74`），後者用於檔案總管無法對應到應用程式的視窗。
- **RecentApps**（Windows 10 至 1709 版）：使用者最近執行過的應用程式，含啟動
  次數與最後時間；每個應用程式之下列有其開啟過的檔案（`RecentItems`）。
- **工作列釘選項目**（`Taskband`）：釘選到工作列的程式，以 Shell 項目
  （`Favorites`）及每個釘選捷徑的連結（`FavoritesResolve`）兩種形式呈現。在 Windows 10 及
  更新版本中，每個釘選項目會標示是由 Windows 釘選（`SystemPinned`）還是由使用者釘選（`UserPinned`）。
- **ProgramsCache**（`StartPage`、`StartPage2`；Windows XP 至 7）：「開始」功能表的快取
  程式（`ProgramsCache`），以及在 Windows 7 上「開始」功能表與工作列的釘選程式
  （`ProgramsCacheSMP`、`ProgramsCacheTBP`）。

<span class="evidence-strength">最佳結論：</span>此使用者以此方式使用過此應用程式這麼多次
（FeatureUsage）、在此時間執行過它（RecentApps），或曾釘選它。這些
計數器本身沒有時間；只有機碼的最後寫入時間與 RecentApps 的時間能夠標示
時間。

## 位置 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

`FeatureUsage` 的 `KeyCreationTime` 是開始計數的時間；摘要中會顯示此值。請收集
每個設定檔的 `NTUSER.DAT` 及其交易記錄，以及系統的 `SYSTEM` 與
`SOFTWARE` 登錄區，以取得主機與使用者的 SID。

## 分析注意事項 { #analyst-cautions }

- **有次數、沒有時間。**FeatureUsage 的次數說明的是頻率，而非時間。各類別機碼的
  最後寫入時間（`KeyLastWrittenUTC`）是其中任一計數最近一次變更的時間。
- **並非每次執行都會計入。**`AppSwitched` 計算的是視窗切換次數，而非啟動次數；沒有視窗
  就執行的應用程式，或使用者從未切換過去的應用程式，都不會被計入。`AppLaunch` 只計算
  從工作列啟動的次數。
- **封裝應用程式與路徑。**應用程式的名稱取決於 Windows 如何將其視窗分組：
  封裝應用程式的 ID、程式自行設定的 ID（`Chrome`、`VMware.Workstation.vmui`），或是
  路徑。`AppPath` 會提供將已知資料夾 GUID 替換為該資料夾預設路徑後的
  路徑。
- **預設釘選項目。**新的設定檔會取得預設釘選項目（檔案總管、Edge、Store）；在 Windows 10
  及更新版本中，這些項目會標示為 `SystemPinned`。`UserPinned` 釘選項目則是使用者自己的選擇。
- **兩份清單中的釘選項目。**`Favorites` 與 `FavoritesResolve` 以相同順序（`Index`）
  描述相同的釘選項目，但會以不同的記錄呈現。
- **ProgramsCache 路徑**是相對路徑：資料夾（`Programs\Accessories`）之後會接著
  其中的捷徑（`Notepad.lnk`）。
- **損毀的值。**非 `REG_DWORD` 的計數、非 `REG_QWORD` 的時間、無法理解的釘選
  項目，或不是連結的連結，都會保留其位元組（`DataRaw`），並在
  `Problems` 中回報。

## 使用 Juntei 剖析 { #parse-with-juntei }

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

摘要會依來源列出記錄、各登錄區的 FeatureUsage 開始計數的時間、最常被切換到的
應用程式、最近使用的應用程式與項目（由新到舊），以及工作列的釘選項目。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`User`、`UserSID`、`SourcePath` | 主機、設定檔資料夾名稱、取自系統 `ProfileList` 的使用者 SID，以及登錄區。 |
| `Source` | `AppSwitched`、`AppLaunch`、`ShowJumpView`、`AppBadgeUpdated`、`TrayButtonClicked`（FeatureUsage）、`RecentApps`、`RecentItems`、`Taskband` 或 `ProgramsCache`。 |
| `App` | FeatureUsage 值的名稱、最近使用之應用程式的 `AppId`，或釘選項目的應用程式使用者模型 ID。 |
| `AppPath` | 以路徑表示的 `App`，已知資料夾 GUID 已替換為其預設路徑；若為 ID 則為空。 |
| `Count` | FeatureUsage 的計數，或最近使用之應用程式的 `LaunchCount`。 |
| `LastAccessedUTC` | 最近使用之應用程式或項目的 `LastAccessedTime`。 |
| `ItemPath` | 最近使用之項目的 `Path`（或最近使用之應用程式的 `AppPath`）；釘選項目或快取程式的 Shell 項目路徑（`User Pinned\TaskBar\File Explorer.lnk`、`Applications\<app ID>`）；`FavoritesResolve` 連結的路徑。 |
| `DisplayName` | 最近使用之項目的 `DisplayName`。 |
| `PinType` | `Favorites` 釘選項目的 `SystemPinned` 或 `UserPinned`。 |
| `Index` | 釘選項目或快取程式在其清單中的位置。 |
| `ValueName`、`KeyPath`、`KeyLastWrittenUTC` | 來源的值與機碼，以及該機碼的最後寫入時間。 |
| `DataRaw` | 無法解碼的位元組（十六進位）。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡。 |
| `Problems` | 類型錯誤的值、無法理解的釘選項目或清單、不是連結的連結。 |

## 調查策略 { #investigation-strategy }

1. 依 `Count` 排序 `AppSwitched`，找出使用者最常使用的應用程式，並留意
   系統管理、遠端存取、壓縮封存與傳輸工具，以及位於使用者可寫入位置的路徑
   （`\Users\`、`Downloads`、`Temp`）。
2. 檢查釘選項目：`UserPinned` 的工具，或捷徑位於不尋常位置的釘選項目。
3. 搭配 UserAssist、Prefetch、BAM 與 Jump Lists，使用 RecentApps 的時間來判定使用時間。
4. 將 `KeyCreationTime` 與設定檔的建立時間比較：計數從那時開始。
5. 使用 `parse-registry read --recover-deleted` 找出已刪除的計數器與釘選項目。

## 延伸資訊 { #further-details }

- [Microsoft：應用程式使用者模型 ID](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft：KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb：Programs cache](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
