# MuiCache { #muicache }

## 什麼是 MuiCache { #what-muicache-is }

當使用者啟動程式時，Windows 會讀取該程式的版本資源，以取得 Shell 應顯示的名稱
（`FileDescription`，例如 `cmd.exe` 的 "Windows Command Processor"），並依使用者
快取在 `MuiCache` 中。Windows 8 及更新版本也會快取公司名稱
（`CompanyName`）。此快取以程式的完整路徑為鍵：

- **Windows 8 及更新版本：**每個程式兩個值，`<path>.FriendlyAppName` 與
  `<path>.AppCompany`。
- **Windows Vista 與 7：**每個程式一個值 `<path>`，保存易記名稱。
- **Windows XP：**相同的形式，位於 `NTUSER.DAT` 中。

MuiCache 可為使用者的 Shell 工作階段提供程式執行的線索，包括位於
卸除式磁碟機、網路共用資料夾及已不存在之資料夾中的程式，並附上檔案本身所提供的名稱與公司。

<span class="evidence-strength">最佳結論：</span>此 Shell 設定檔曾快取某個檔案的路徑、
描述與公司。請與其他執行跡證交叉比對。設定檔並不能識別
處理程序執行時所用的帳戶。MuiCache 沒有個別程式的時間，也沒有執行次數。

## 位置 { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat     Windows Vista and later
Local Settings\Software\Microsoft\Windows\Shell\MuiCache

C:\Documents and Settings\<user>\NTUSER.DAT                         Windows XP
Software\Microsoft\Windows\ShellNoRoam\MUICache
```

`LangID`（快取字串的語言）不是程式，因此會被排除。名稱為
`@<file>,-<number>`（`@shell32.dll,-22019`）的值是 Shell 為自身功能表與檔案類型
快取的資源字串，而非程式；這些值會被排除，並在摘要中計數。在 Windows Vista
及更新版本中，`NTUSER.DAT` 沒有 MuiCache。

請收集每個設定檔的 `UsrClass.dat`（及 `NTUSER.DAT`）與其交易記錄，以及
系統的 `SYSTEM` 與 `SOFTWARE` 登錄區，以取得主機與使用者名稱。

## 分析注意事項 { #analyst-cautions }

- **設定檔擁有者不一定是處理程序的帳戶。**以其他帳戶啟動的程式，
  可以在此使用者的工作階段中顯示視窗，並在此使用者的快取中留下中繼資料。
  `User` 與 `UserSid` 識別的是登錄區的擁有者，而非處理程序的安全性內容。
- **沒有個別程式的時間。**`KeyLastWrittenUTC` 是機碼的最後寫入時間：即其中任何程式
  最近一次變更的時間，而非特定程式執行的時間。Prefetch、BAM、UserAssist 與事件記錄才能提供
  時間。
- **不只是使用者手動啟動的程式。**Shell 本身載入的 DLL 或程式（用於
  `Open with`、Shell 擴充功能、安裝程式的子處理程序）也可能出現，例如 `explorerframe.dll` 或
  `shell32.dll`。
- **名稱來自檔案。**`FriendlyAppName` 與 `AppCompany` 是檔案執行時其
  版本資源所宣稱的內容。惡意軟體可以宣稱任何描述與公司；被重新命名的
  工具會保留其原始描述，這可能使其露出破綻（例如名為 `svchost.exe`、但易記名稱
  卻是遠端存取工具的檔案）。
- **每個路徑一個項目。**再次執行同一個程式不會新增項目；位於其他
  路徑的複本則會。檔案刪除後，程式的項目仍可能留存。
- **同一路徑的兩種形式。**當某個路徑同時以兩種形式具有易記名稱時（升級過的
  設定檔），每種形式各有一筆記錄，因此兩個值都不會遺失。
- **損毀的值。**不是字串、不是有效 UTF-16 或無法完整讀取的值，會保留
  其位元組（`FriendlyAppNameRaw`、`AppCompanyRaw`），並在 `Problems` 中回報。
  無效的 UTF-16 也可能在顯示的文字中產生替代字元；請使用原始資料
  欄位取得原始位元組。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry muicache -d evidence/ -o muicache.csv
juntei parse-registry muicache -f UsrClass.dat -t jsonl \
  | jq 'select(.Path | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

摘要會列出位於 Windows 與 Program Files 資料夾以外（使用者資料夾、
其他磁碟機、網路共用資料夾）的程式及其使用者，以及被排除的資源字串數量。
名稱不同的 Program Files 資料夾（其他語言版本的 Windows）也會列出。
相對路徑以及跳脫環境變數資料夾的路徑仍會列出。一般的
磁碟機路徑會在解析 `.` 與 `..` 部分後進行檢查，而不會存取檢驗人員的
檔案系統。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`User`、`UserSID`、`SourcePath` | 主機、設定檔資料夾名稱、使用者的 SID（取自 `UsrClass.dat` 根機碼的名稱；否則取自系統的 `ProfileList`），以及登錄區。 |
| `HiveType` | `UsrClass.dat` 或 `NTUSER.DAT`。 |
| `Path` | 程式的路徑：去除 `.FriendlyAppName` 或 `.AppCompany` 後的值名稱。 |
| `FriendlyAppName` | 程式的描述，取自 `<path>.FriendlyAppName` 或 `<path>`。 |
| `AppCompany` | 程式的公司，取自 `<path>.AppCompany`（Windows 8 及更新版本）。 |
| `KeyPath`、`KeyLastWrittenUTC` | MuiCache 機碼及其最後寫入時間：其中任何程式最近一次變更的時間。 |
| `FriendlyAppNameRaw`、`AppCompanyRaw` | 當值不是能乾淨解碼的文字時，其位元組（十六進位）。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡：當交易記錄寫入了該程式的某個值時為 `replayed`。 |
| `Problems` | 不是字串、不是有效 UTF-16 或無法讀取的值。 |

## 調查策略 { #investigation-strategy }

1. 檢視位於使用者可寫入位置（`\Users\`、`Downloads`、`Temp`、`ProgramData`）、
   其他磁碟機與網路共用資料夾上的程式，以及系統管理、遠端存取與壓縮封存工具。
2. 比較檔名與其易記名稱及公司：不一致可能揭露被重新命名的
   工具或偽裝的檔案。
3. 與 Prefetch、BAM 和 UserAssist（執行時間）、Amcache 和 ShimCache（檔案的
   雜湊值與存在）以及 `$MFT`（檔案是否仍存在）交叉比對。
4. 對 `UsrClass.dat` 使用 `parse-registry read --recover-deleted`，以找出已刪除的 MuiCache 值。

## 延伸資訊 { #further-details }

- [Microsoft：VERSIONINFO 資源](https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource)
- [Cyber Triage：MuiCache 實驗、GUI 工作階段與處理程序帳戶的限制](https://www.cybertriage.com/blog/muicache-2026-guide/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
