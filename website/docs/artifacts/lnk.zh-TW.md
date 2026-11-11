# LNK 捷徑 { #lnk-shortcuts }

## 什麼是 LNK 檔案 { #what-lnk-files-are }

Windows Shell Link（`.lnk`）檔案描述一個目標，以及啟動參數、工作目錄、圖示、
視窗狀態、目標中繼資料、磁碟區／網路資訊、Shell 命名空間項目，以及選用的
分散式連結追蹤識別碼。Windows 與應用程式會在「最近使用的項目」、
桌面、「開始」功能表及其他位置建立這些檔案；攻擊者也會直接投遞惡意捷徑。

「最近使用的項目」資料夾中的 LNK 可以佐證使用者曾與某個目標互動，即使該目標已不存在，包括
卸除式磁碟機或網路共用資料夾上的目標。任意位置的 LNK 則可能從未被點擊過。

<span class="evidence-strength">最佳結論：</span>此捷徑曾以所記錄的目標脈絡被建立／更新。
其所在位置與周邊跡證決定它是否能佐證使用行為。

## 位置與收集 { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\*.lnk
%USERPROFILE%\Desktop\*.lnk
%APPDATA%\Microsoft\Windows\Start Menu\Programs\...
%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\...
```

請搜尋整個使用者設定檔以及收集到的封存檔／電子郵件內容，而不只是 `Recent`。請保留
來源檔案本身的時間與路徑，因為所在位置會改變判讀結果。

## 分析注意事項 { #analyst-cautions }

- 目標時間、大小、MFT 參照與磁碟區資料是儲存在 LNK 中的快照；它們並非
  捷徑檔案本身的中繼資料，而且可能已經過時。
- `MachineID`、MAC 與追蹤器建立時間描述的是分散式連結追蹤的脈絡，通常是
  建立／更新該連結的機器——不一定是你發現它的那台機器。
- 類似 MAC 的 UUID 節點值可能來自虛擬化、隨機化或非網路識別碼。
- `Args` 可能讓良性的執行檔（`powershell.exe`、`rundll32.exe`、`mshta.exe`）變得危險。
- ANSI 字串需要來源系統的字碼頁；許多日文連結請使用 `--code-page 932`。
- 捷徑可能是刻意偽造或複製而來。若沒有 Prefetch、
  Jump List、RecentDocs、事件記錄或其他佐證，不要將其欄位等同於執行證據。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.lnk
  -f, --file <FILE>      Parse one shortcut
      --code-page <CP>   ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -d evidence/ --code-page 932 -o lnk-ja.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

## CSV 輸出欄位 { #csv-output-fields }

| 欄位群組 | 欄位與判讀 |
|---|---|
| 來源 | `SourceFile`、`SourceCreated`、`SourceModified`、`SourceAccessed`：捷徑路徑與主機檔案系統時間。這些與目標時間是分開的。 |
| 目標時間 | `TargetCreated`、`TargetModified`、`TargetAccessed`：記錄在 LNK 中的快照；是有用的歷史線索，而非目標目前的中繼資料。 |
| 目標基本資訊 | `FileSize`、`RelativePath`、`WorkingDirectory`、`FileAttributes`、`HeaderFlags`：所記錄的目標／啟動脈絡。參數可能改變其意義。 |
| 磁碟區 | `DriveType`、`VolumeSerialNumber`、`VolumeLabel`：識別固定／卸除式／網路脈絡，並與其他捷徑交叉比對。 |
| 路徑 | `LocalPath`、`NetworkPath`、`CommonPath`、`TargetIDAbsolutePath`：互補的路徑表示方式。請檢視彼此不一致之處與 Shell 命名空間路徑。 |
| 啟動行為 | `Args`、`ShowWindow`、`HotKey`、`ShimLayer`、`DarwinId`：Shell 如何啟動或設定目標。`Args` 至關重要。 |
| NTFS 識別 | `TargetMFTEntryNumber`、`TargetMFTSequenceNumber`：建立連結時的目標參照；請同時比對兩者與 `$MFT`。 |
| 追蹤 | `MachineID`、`MachineMACAddress`、`TrackerCreatedOn`、`VolumeDroid`、`FileDroid`、`VolumeDroidBirth`、`FileDroidBirth`：分散式連結追蹤。請用於交叉比對，不可單獨作為歸因依據。 |
| 額外區塊 | `ExtraBlocksPresent`：可用的選用結構清單；說明其他哪些欄位有資料支撐。 |
| 顯示 | `Name`、`IconLocation`、`IconIndex`、`IconEnvironment`、`EnvironmentVariables`：顯示資料與展開環境變數後的目標資料。攻擊者可能使用誤導性的名稱／圖示。 |
| Shell 資料夾 | `KnownFolder`、`SpecialFolder`：解碼後的 Windows Shell 資料夾識別碼。 |
| 網路 | `NetworkDevice`、`NetworkProvider`：網路共用資料夾／裝置與提供者脈絡。 |
| 編碼 | `ConsoleCodePage`：若存在，為所記錄的主控台字碼頁；與剖析器的 `--code-page` 是分開的。 |
| 尾端資料 | `AppendedBytes`：邏輯結尾之後的位元組。非預期的資料可能代表夾帶走私或檔案格式錯誤。 |
| 驗證 | `Problems`：損毀、截斷、區塊不一致或解碼限制。請納入檢視。 |

JSON/JSONL 另外保留每個目標 ID 項目（種類、名稱、GUID、FAT 時間與 MFT
參照）、所有額外資料區塊、屬性存放區屬性、主控台設定，以及原始結構
關係。面對可疑捷徑時請使用它，而不要只依賴扁平的 CSV 列。
巢狀 `TargetItems` 的時間來自 FAT Shell 項目時鐘，並未儲存時差。
其沿用的 `Z` 呈現方式並不能證明是 UTC；`auto-parse --timezone` 會保留它們原樣。
LNK 標頭中的目標 FILETIME 與追蹤器時間仍是可轉換的絕對時間戳記。

## 調查策略 { #investigation-strategy }

1. 在判讀目標欄位之前，先檢查來源位置與來源時間。
2. 從目標路徑、`Args`、`WorkingDirectory` 與環境區塊重建實際的啟動方式。
3. 以目標 MFT 參照、磁碟區序號、網路路徑、droid、MachineID 與 MAC 在各證據間進行樞紐比對。
4. 檢查 Prefetch 與事件記錄以確認執行；檢查 Jump Lists／RecentDocs 以確認使用者互動。
5. 對於惡意附件，請保留附加資料，並對 LNK 本身計算雜湊值／進行分析。

## 延伸資訊 { #further-details }

- [Microsoft Open Specifications：MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [Sootmark shell](https://github.com/Sootmark/shell)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
