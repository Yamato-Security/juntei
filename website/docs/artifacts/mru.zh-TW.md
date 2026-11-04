# 最近使用（MRU）清單（RunMRU、TypedPaths、WordWheelQuery、TypedURLs） { #most-recently-used-lists-runmru-typedpaths-wordwheelquery-typedurls }

## 這些清單是什麼 { #what-the-lists-are }

Windows 及其程式會在使用者的 `NTUSER.DAT` 中，保存使用者最近輸入或開啟內容的簡短清單：

- **RunMRU：**在「執行」對話方塊（Win+R）中輸入的命令：`cmd`、`powershell -ep bypass`、
  `\\server\share`、`regedit`。
- **TypedPaths：**在檔案總管網址列中輸入的路徑。
- **WordWheelQuery：**在檔案總管搜尋方塊中搜尋的字詞。
- **TypedURLs：**在 Internet Explorer 網址列中輸入的位址；自 Windows 8 起，
  `TypedURLsTime` 會記錄每個位址的輸入時間。
- **Paint 與 WordPad：**各自最近開啟的檔案（`Recent File List`）。
- **ACMru（Windows XP）：**搜尋助理的搜尋字詞，依搜尋種類區分。

<span class="evidence-strength">最佳結論：</span>使用者輸入過此命令、路徑、
搜尋字詞或位址，或該程式開啟過此檔案。每個清單只有最新的一筆項目有日期，
即其機碼的最後寫入時間；唯一例外是輸入的位址，由 `TypedURLsTime` 提供日期。

## 位置 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

`ACMru` 的子機碼代表搜尋種類：`5603` 為檔案與資料夾名稱，`5604` 為檔案中的字詞或片語，
`5647` 為電腦與人員，`5001` 為網際網路。

請為每個設定檔收集 `NTUSER.DAT` 及其交易記錄檔。

## 分析注意事項 { #analyst-cautions }

- **每個清單只有一個時間。**機碼的最後寫入時間會在其清單變更時改變，因此它只能為
  最新的一筆項目定時（`IsMostRecent`、`LastUsedUTC`）。其他項目只有順序，沒有日期。
  `TypedURLsTime` 則為每個輸入的位址提供日期（`TypedTimeUTC`）。
- **是輸入，不是執行。**RunMRU 記錄的是在「執行」對話方塊中輸入的內容，不論是否
  成功執行。Windows 會在每個命令後面加上 `\1`；Juntei 會將其移除。請與
  Prefetch、BAM、UserAssist 以及處理程序建立事件交叉比對。
- **清單很短。**每個清單只保留最近的項目，而且隱私權設定與清理工具會將其清除。
  已刪除的值請使用 `parse-registry read --recover-deleted`。
- **字詞是輸入的內容。**WordWheelQuery 與 ACMru 保存的是輸入的文字，而不是找到的結果。
- **順序來自清單本身。**RunMRU 的 `MRUList` 與 WordWheelQuery 的 `MRUListEx` 提供
  順序；其他清單則由值名稱中的編號決定（`url1`、`File1` 與 `000` 是最新的）。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

摘要會列出每個清單的項目數，以及最後有日期的項目（輸入的位址，以及每個
清單最新的一筆項目），由新到舊排列。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`User`、`UserSID`、`SourcePath` | 主機、設定檔擁有者（取自路徑或 Shell 資料夾）、取自系統 `ProfileList` 的 SID，以及登錄區。 |
| `List` | `RunMRU`、`TypedPaths`、`WordWheelQuery`、`TypedURLs`、`Paint`、`Wordpad` 或 `ACMru`。 |
| `Subkey` | `ACMru` 的子機碼：搜尋種類（`5603`、`5604`、`5647`、`5001`）。 |
| `MRU-Position`、`IsMostRecent` | 在清單中的位置（0 為最新）；若清單未列出該值則為空白。 |
| `ValueName` | `a`、`0`、`url1`、`File1`、`000`、… |
| `Value` | 命令（不含其 `\1`）、路徑、搜尋字詞、位址或檔案。 |
| `TypedTimeUTC` | 位址的輸入時間，取自 `TypedURLsTime`。 |
| `LastUsedUTC` | 機碼的最後寫入時間，僅適用於最新的一筆項目。 |
| `KeyPath`、`KeyLastWrittenUTC` | 機碼及其最後寫入時間：這是其最新項目的時間，而非每一列的時間。 |
| `DataRaw` | 無法解碼時，值的位元組（十六進位）。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡。 |
| `Problems` | 清單中列出但機碼中不存在的值、清單未列出的值、名稱錯誤的值、類型錯誤、被截斷的字詞，以及不是 FILETIME 的時間。 |

## 調查策略 { #investigation-strategy }

1. 檢視 RunMRU，找出使用者輸入的命令、指令碼、網路共用資料夾與管理工具。
2. 檢視 TypedPaths 與 WordWheelQuery，找出使用者刻意尋找的資料夾與檔案：
   認證資訊、匯出資料、封存檔、卸除式與網路位置。
3. 使用 `TypedTimeUTC` 將輸入的位址放入時間軸。
4. 與 Prefetch、BAM、UserAssist、ShellBags、LNK 檔案、Jump Lists 與瀏覽器歷程記錄交叉比對。

## 延伸資訊 { #further-details }

- [libyal winreg-kb：最近使用（MRU）機碼](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
