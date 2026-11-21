# PowerShell 歷程記錄 { #powershell-history }

## 歷程記錄的內容 { #what-the-history-records }

PSReadLine 會儲存在互動式 PowerShell 主機中輸入的命令列。檔案依儲存順序包含文字，沒有逐一命令的時間戳記。

## 位置與收集 { #locations-and-collection }

Windows PowerShell 5.1 與 PowerShell 7+ 在 Windows 上的預設位置如下：

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

其他主機會在該目錄使用 `Visual Studio Code Host_history.txt` 等名稱。Linux 與 macOS 上的 PowerShell 通常使用 `~/.local/share/powershell/PSReadLine/<Host>_history.txt`。PSReadLine 的 `HistorySavePath` 可以變更儲存位置；這些預設路徑是搜尋時的假設，並未涵蓋所有位置。

目錄輸入與 `auto-parse` 會遞迴選取直接位於 `PSReadLine` 資料夾內的 `<Host>_history.txt`，比對時不區分 ASCII 大小寫。它們不會選取其他位置的任意 `*_history.txt` 檔案。對於重新命名的檔案或 `PSReadLine` 目錄外的副本，請明確使用 `-f` 輸入。請盡可能保留原始設定檔結構，以便判定使用者。

## 分析注意事項 { #analyst-cautions }

- 每一行都沒有時間。`FileLastWriteTime` 是檔案的修改時間。PSReadLine 預設會在每個命令之後儲存歷程記錄（`HistorySaveStyle` 為 `SaveIncrementally`），因此這個時間通常接近最後一行加入的時間，但無法說明較早的各行。
- 已儲存的行無法證明命令曾經執行或已成功完成。請以 PowerShell 事件記錄、處理程序遙測及其他執行跡證交叉驗證執行與時間。
- 歷程記錄可以被編輯、刪除或關閉（`HistorySaveStyle` 為 `SaveNothing`）。PSReadLine 也會略過看似含有機密的行，例如含有 `password`、`token` 或 `apikey` 的行，而 Windows PowerShell ISE 不使用 PSReadLine。指令碼，以及以 `-Command` 或 `-EncodedCommand` 執行的命令，都不是手動輸入的，因此不會被儲存。
- 多行命令儲存時，除了最後一行以外，每一行末尾都會加上反引號。Juntei 會將這些行保留為各自獨立的資料列，並以 `EntryStartLine` 連結。以使用者輸入的反引號結尾的行看起來完全相同，因此請將同一項目的各行一起閱讀。

## 使用 Juntei 解析 { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

CSV 是預設檔案格式。指定 `-t csv`、`-t json` 或 `-t jsonl` 而不指定 `-o` 時，記錄會寫至標準輸出。兩個選項都不指定時，只顯示摘要。JSON 由分別排版的獨立物件組成；JSONL 每行一個物件。加入 `-C` 可覆寫現有輸出；指向來源檔案的別名會受到保護，不會被取代。

## 編碼與行處理 { #encoding-and-line-handling }

- UTF-8、UTF-16LE 或 UTF-16BE 的位元組順序記號會決定編碼，並從檔案開頭移除。沒有 BOM 時，一律視為 UTF-8。不會猜測無 BOM 的 UTF-16；NUL 字元會保留，並回報可能的編碼損壞。
- 無效的 UTF-8 序列及未配對的 UTF-16 代理碼會轉為 U+FFFD，並在 `Problems` 回報損壞。UTF-16 結尾多出的單一位元組及讀取錯誤也會回報；可讀的行仍會保留。判定資料完整性前，請同時檢查 `Problems` 與解析器日誌。
- 檔案採串流處理。每個實體行最多保留 1,048,576 個輸入位元組，不含 BOM 與 LF 分隔符號（LF 前的 CR 計入上限）。較長的行會在完整字元的邊界截斷，其餘位元組仍會讀取，並由 `Problems` 回報省略的位元組數。上限適用於編碼後的輸入位元組，不是字元數。
- LF 與 CRLF 行尾會移除；未以換行結束的行若有末尾 CR，也會移除。其他空格與定位字元均保留。空行不產生記錄，但計入行號並結束接續鏈；僅含空白字元的行仍會產生記錄。讀取錯誤可能產生空的診斷行，以標示讀取停止的位置。
- 各行仍分開，保留反引號與縮排。`EntryStartLine` 指向連續接續鏈的第一行，鏈中前面的行均以反引號結尾。不會解譯 Shell 語法或重建命令。截斷可能影響接續關係的判定；將這些行分組前，請檢查 `Problems`。

## CSV 輸出欄位 { #csv-output-fields }

| 欄位 | 解讀 |
|---|---|
| `SourceFile` | 相對於輸入目錄的路徑，使用 `/`；`-f` 僅含檔名。不會匯出絕對來源路徑。 |
| `User` | 由路徑判斷的設定檔名稱。`AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine`（或 `.local/share/powershell/PSReadLine`）的上一層資料夾即為使用者名稱；輸入為該設定檔或 `PSReadLine` 資料夾本身時亦同。目錄輸入也會辨識輸入根目錄下的 `Users`/`Documents and Settings` 結構。未知值在 CSV 為空，在 JSON 為 `null`。 |
| `ShellHost` | `_history.txt` 前的檔名前綴，例如 `ConsoleHost`；重新命名後無此後綴的檔案則為未知。 |
| `LineNumber` | 從 1 開始的實體行號，包括略過的空行。 |
| `EntryStartLine` | 接續鏈起始的實體行號，或此行本身的行號。 |
| `Command` | 移除行尾的解碼後實體行，受前述上限限制。 |
| `FileLastWriteTime` | 來源檔案的修改時間，每行重複記錄。**這是檔案中繼資料，絕非命令執行時間。** 解析映像檔使用原始來源中繼資料，不使用暫存時間。 |
| `FileSize` | 來源檔案大小，單位為位元組。 |
| `Encoding` | `UTF-8`、`UTF-8-BOM`、`UTF-16LE` 或 `UTF-16BE`。 |
| `Problems` | 行損壞與截斷診斷；CSV 以分號分隔，JSON/JSONL 為陣列。檔案層級診斷也會出現在日誌中。 |

JSON 與 JSONL 具有相同的欄位，但數字帶有型別，未知值為 `null`，`Problems` 則為清單。所有格式都會保留儲存時的命令文字。

`FileLastWriteTime` 預設使用 UTC 的 ISO 8601 格式，例如 `2026-10-20T08:46:38.5Z`。只有 `auto-parse` 接受 `-Z, --timezone` 來變更顯示的時差。獨立的 `parse-file powershell-history` 命令沒有時區選項。

## 自動剖析 { #automatic-parsing }

`auto-parse` 會寫入 `ShellHistory/PowerShellHistory.csv`（使用 `-t json` 或 `-t jsonl` 時則為 `PowerShellHistory.json` 或 `PowerShellHistory.jsonl`），並附上 `ShellHistory/PowerShellHistory-logs.txt`。從 VHDX 與 E01 映像檔剖析時，會複製歷程記錄檔案，而 `FileLastWriteTime` 是映像檔中記錄的時間。合併的 `-j` 時間軸包含每一筆資料列，並帶有 `Artifact` 識別碼 `powershell-history`。由於這些行沒有時間，[`super-timeline`](../commands/super-timeline.md) 不會從中產生任何事件，而是將其計為無時間的記錄。

## 調查策略 { #investigation-strategy }

1. 閱讀每個設定檔的歷程記錄，包括 `Windows\System32\config\systemprofile` 及 `Windows\ServiceProfiles` 下的設定檔，其中保存了在以這些帳戶身分執行的 Shell 中輸入的命令。
2. 將多行項目一起閱讀，並尋找下載、編碼過的命令、安全性設定的變更、帳戶與認證資訊相關命令，以及遠端連線。
3. 與 PowerShell 事件記錄（指令碼區塊記錄，事件 4104）、處理程序建立事件，以及 `powershell.exe` 與 `pwsh.exe` 的 Prefetch 檔案交叉比對。
4. 將 `FileLastWriteTime` 與歷程記錄檔案的 `$MFT` 及 USN 日誌時間，以及登入事件比較，以界定最後幾行加入的時間範圍。

## 延伸資訊 { #further-details }

- [Microsoft — about_PSReadLine：命令歷程記錄](https://learn.microsoft.com/en-us/powershell/module/psreadline/about/about_psreadline)
- [Microsoft — Set-PSReadLineOption：HistorySavePath 與 HistorySaveStyle](https://learn.microsoft.com/en-us/powershell/module/psreadline/set-psreadlineoption)
- [Microsoft — about_Logging_Windows：指令碼區塊記錄](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [PSReadLine — 歷程記錄檔案的儲存與讀取方式](https://github.com/PowerShell/PSReadLine/blob/master/PSReadLine/History.cs)
