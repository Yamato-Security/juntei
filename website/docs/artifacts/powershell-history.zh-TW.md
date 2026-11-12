# PowerShell 歷程記錄 { #powershell-history }

## 歷程記錄的內容 { #what-the-history-records }

PSReadLine 會儲存在互動式 PowerShell 主機中輸入的命令列。檔案依儲存順序包含文字，沒有逐一命令的時間戳記。歷程記錄可以編輯、清除、停用或篩選，無法證明命令已成功完成。請以 PowerShell 事件記錄、程序遙測及其他執行跡證交叉驗證執行與時間。

## 位置與蒐集 { #location-and-collection }

Windows PowerShell 5.1 與 PowerShell 7+ 在 Windows 上的預設位置如下：

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

其他主機會在該目錄使用 `Visual Studio Code Host_history.txt` 等名稱。Linux 與 macOS 上的 PowerShell 通常使用 `~/.local/share/powershell/PSReadLine/<Host>_history.txt`。PSReadLine 的 `HistorySavePath` 可以變更儲存位置；這些預設路徑是搜尋時的假設，並未涵蓋所有位置。

目錄輸入與 `auto-parse` 會遞迴選取直接位於 `PSReadLine` 資料夾內的 `<Host>_history.txt`，比對時不區分 ASCII 大小寫。它們不會選取其他位置的任意 `*_history.txt` 檔案。對於重新命名的檔案或 `PSReadLine` 目錄外的副本，請明確使用 `-f` 輸入。請盡可能保留原始設定檔結構，以便判定使用者。

## 使用 Juntei 解析 { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

CSV 是預設檔案格式。指定 `-t csv`、`-t json` 或 `-t jsonl` 而不指定 `-o` 時，記錄會寫至標準輸出。兩個選項都不指定時，只顯示摘要。JSON 由分別排版的獨立物件組成；JSONL 每行一個物件。加入 `-C` 可覆寫現有輸出；指向來源檔案的別名會受到保護，不會被取代。

`auto-parse` 會儲存為 `ShellHistory/PowerShellHistory.csv`（或所選的 JSON 格式），VHDX/E01 映像檔亦同。歷程記錄不會為 `auto-parse -j` 或 `super-timeline` 提供事件。

## 編碼與行處理 { #encoding-and-line-handling }

- UTF-8、UTF-16LE 或 UTF-16BE 的位元組順序記號會決定編碼，並從檔案開頭移除。沒有 BOM 時，一律視為 UTF-8。不會猜測無 BOM 的 UTF-16；NUL 字元會保留，並回報可能的編碼損壞。
- 無效的 UTF-8 序列及未配對的 UTF-16 代理碼會轉為 U+FFFD，並在 `Problems` 回報損壞。UTF-16 結尾多出的單一位元組及讀取錯誤也會回報；可讀的行仍會保留。判定資料完整性前，請同時檢查 `Problems` 與解析器日誌。
- 檔案採串流處理。每個實體行最多保留 1,048,576 個輸入位元組，不含 BOM 與 LF 分隔符號（LF 前的 CR 計入上限）。較長的行會在完整字元的邊界截斷，其餘位元組仍會讀取，並由 `Problems` 回報省略的位元組數。上限適用於編碼後的輸入位元組，不是字元數。
- LF 與 CRLF 行尾會移除；未以換行結束的行若有末尾 CR，也會移除。其他空格與定位字元均保留。空行不產生記錄，但計入行號並結束接續鏈；僅含空白字元的行仍會產生記錄。讀取錯誤可能產生空的診斷行，以標示讀取停止的位置。
- PSReadLine 儲存多行項目時，會在每個接續的實體行末尾加上反引號。各行仍分開，保留反引號與縮排。`EntryStartLine` 指向連續接續鏈的第一行，鏈中前面的行均以反引號結尾。不會解譯 Shell 語法或重建命令。截斷可能影響接續關係的判定；將這些行分組前，請檢查 `Problems`。

## 輸出欄位 { #output-fields }

| 欄位 | 解讀 |
|---|---|
| `SourceFile` | 相對於輸入目錄的路徑，使用 `/`；`-f` 僅含檔名。不會匯出絕對來源路徑。 |
| `User` | 由保留的路徑推定的設定檔名稱。目錄輸入也會辨識輸入根目錄下的 `Users`/`Documents and Settings` 結構；`-f` 需要一般的 PSReadLine 設定檔路徑尾段。未知值在 CSV 為空，在 JSON 為 `null`。 |
| `ShellHost` | `_history.txt` 前的檔名前綴，例如 `ConsoleHost`；重新命名後無此後綴的檔案則為未知。 |
| `LineNumber` | 從 1 開始的實體行號，包括略過的空行。 |
| `EntryStartLine` | 接續鏈起始的實體行號，或此行本身的行號。 |
| `Command` | 移除行尾的解碼後實體行，受文件所述上限及 CSV 防護限制。 |
| `FileLastWriteTime` | 來源檔案的修改時間，每行重複記錄。**這是檔案中繼資料，絕非命令執行時間。** 解析映像檔使用原始來源中繼資料，不使用暫存時間。 |
| `FileSize` | 來源檔案大小，單位為位元組。 |
| `Encoding` | `UTF-8`、`UTF-8-BOM`、`UTF-16LE` 或 `UTF-16BE`。 |
| `Problems` | 行損壞與截斷診斷；CSV 以分號分隔，JSON/JSONL 為陣列。檔案層級診斷也會出現在日誌中。 |

`FileLastWriteTime` 預設使用 UTC 的 ISO 8601 格式，例如 `2026-10-20T08:46:38.5Z`。只有 `auto-parse` 接受 `-Z, --timezone` 來變更顯示的時差。獨立的 `parse-file powershell-history` 命令沒有時區選項。

每個 CSV 文字儲存格都會檢查試算表公式前綴（`=`、`+`、`-`、`@`，包括全形形式，以及空白或特定不可見／控制字元後的前綴）。符合的儲存格，以及以控制字元開頭的儲存格，會加上前導單引號。這可能改變 CSV 中的命令與路徑文字。JSON 與 JSONL 保留解碼後的文字，不加 CSV 單引號；解碼替換與截斷仍適用。比較儲存的命令文字時請使用 JSONL，並務必保留來源歷程記錄檔案。
