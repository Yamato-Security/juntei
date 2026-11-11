# Prefetch { #prefetch }

## 什麼是 Prefetch { #what-prefetch-is }

Windows Prefetch 檔案記錄用來最佳化應用程式啟動的資訊。程式專屬的 `.pf` 檔案通常會儲存
執行檔名稱與雜湊、執行次數、最近的執行時間、磁碟區資訊、目錄，以及啟動期間所參照的檔案。

在支援的 Windows 版本上，有效的應用程式 Prefetch 記錄通常是該執行檔曾經執行的有力證據。
它是事件記錄以外最有用的執行跡證之一。

<span class="evidence-strength">最佳結論：</span>具名的執行檔曾在所記錄的最近執行時間啟動，
但須視作業系統行為與剖析結果的有效性而定。

## 位置與收集 { #location-and-collection }

```text
C:\Windows\Prefetch\*.pf
```

Juntei 支援格式版本 17 至 31（Windows XP 至 Windows 11），包括 Windows 8 以上的壓縮檔案。
請收集整個目錄，以便比較執行檔名稱雜湊與相鄰的記錄。

## 分析注意事項 { #analyst-cautions }

- 較舊的版本中，Windows 只儲存一個歷史執行時間；較新的版本最多儲存八個。
  `RunCount` 可能超過可用時間戳記的數量。
- 在某些系統／角色上，Prefetch 可能被停用、清除、排除或無法使用。不存在並不能證明程式未曾執行。
- 已載入檔案清單描述的是啟動時的相依性與存取；並不能證明所列出的每個檔案都是惡意的、
  被刻意開啟的，或被完整讀取的。
- Prefetch 標頭中的執行檔名稱有長度限制。Juntei 會使用 SCCA 路徑雜湊來區分同名的已載入路徑。若仍有多個候選路徑無法區分，則保留標頭名稱並設定 PathAmbiguous；已載入檔案清單仍可供查閱。若只有一個候選路徑但雜湊不符，仍會保留該路徑，並將 PathValidated 設為 false。代管處理程序可能會將命令列資料納入雜湊計算，因此未經驗證的路徑不一定是錯的。
- `.pf` 來源檔案的檔案系統時間戳記與內嵌的執行時間是分開的。
- `Findings` 是威脅獵捕的觀察結果。安裝程式與合法的系統管理作業也經常使用資料整備路徑。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pf files
  -f, --file <FILE>      Parse one Prefetch file

Records:
  -T, --timeline  Write one row per retained execution time, sorted by time

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file prefetch -d Prefetch/ -o prefetch.csv
juntei parse-file prefetch -d Prefetch/ -T -o prefetch-timeline.csv
juntei parse-file prefetch -d Prefetch/ -t jsonl \
  | jq 'select(.ExecutableName? | test("powershell|rundll32"; "i"))'
```

## 標準 CSV 欄位 { #standard-csv-fields }

| 欄位 | 判讀 |
|---|---|
| `SourceFile` | 輸入的 `.pf`；請保留它以區分主機與重複的執行檔名稱。 |
| `ExecutableName` | 標頭中的執行檔名稱。大多數獵捕作業可不區分大小寫；但請保留原始拼寫。 |
| `ExecutablePath` | 在可行時，Juntei 解析出的載入路徑。請以 `Files` 與其他跡證加以確認。 |
| `PathValidated` | 所選路徑符合 SCCA 路徑雜湊；這僅確認路徑歸屬，不代表檔案內容或可信度。 |
| `PathAmbiguous` | 多個載入候選路徑無法區分；`ExecutablePath` 保留標頭名稱。請檢查 `Files` 並以其他證據佐證。 |
| `PrefetchHash` | Windows 用於 `.pf` 檔案名稱的雜湊；並非以密碼學方式計算的檔案內容雜湊。 |
| `FormatVersion` | Prefetch 結構版本；決定可用的欄位與時間戳記數量。 |
| `RunCount` | Windows 記錄的累計計數器。可用於判斷頻率，而非確切的歷程。 |
| `LastRun` | 所保留之最近一次的執行時間。 |
| `PreviousRun1` … `PreviousRun7` | 所保留之較早的執行時間，由新到舊。格式／歷程中沒有這些資料時為空。 |
| `VolumeCount` | 所記錄的來源磁碟區數量。 |
| `VolumeDevicePaths` | 啟動期間所使用磁碟區的 NT 裝置路徑。請與序號進行關聯。 |
| `VolumeSerialNumbers` | 磁碟區識別碼；可用於連結卸除式或重新格式化磁碟機的證據。 |
| `VolumeCreated` | 所記錄的磁碟區建立時間，而非執行檔的建立時間。 |
| `DirectoryCount` | 所記錄的目錄字串數量。 |
| `FileCount` | 所記錄的已載入檔案參照數量。 |
| `Findings` | Juntei 的觀察結果，例如被移置的系統二進位檔或可疑的執行路徑。請以人工檢視。 |
| `Files` | 已載入／被參照的檔案，以及可用的 NTFS 參照。是極佳的 DLL／路徑樞紐，但並非檔案意圖的證明。 |
| `Dir` | 與啟動相關的目錄字串。當個別已載入檔案的路徑不完整時很有用。 |

## 時間軸 CSV 欄位 (`-T`) { #timeline-csv-fields-t }

| 欄位 | 判讀 |
|---|---|
| `RunTime` | 一個所保留的 Prefetch 執行時間。 |
| `ExecutableName`、`ExecutablePath` | 程式身分／路徑脈絡。 |
| `PathValidated` | 所選路徑符合 SCCA 路徑雜湊；這僅確認路徑歸屬，不代表檔案內容或可信度。 |
| `PathAmbiguous` | 多個載入候選路徑無法區分；`ExecutablePath` 保留標頭名稱。請檢查 `Files` 並以其他證據佐證。 |
| `PrefetchHash` | 區分共用相同執行檔名稱的變體／路徑。 |
| `RunCount` | 來自 Prefetch 檔案的計數器；會在每一列時間軸中重複出現。 |
| `SourceFile` | 用於驗證的來源 `.pf`。 |

JSON/JSONL 會以結構化陣列保留磁碟區、目錄、已載入檔案與 NTFS 檔案參照，而非以逗號串接的 CSV 儲存格表示。

## 調查策略 { #investigation-strategy }

1. 使用 `-T` 將所保留的執行紀錄加入時間軸。
2. 依執行檔名稱加上 Prefetch 雜湊分組；相同的名稱可能有不同的路徑／雜湊。
3. 檢視 `ExecutablePath`、資料整備路徑、被移置之系統二進位檔的發現項目，以及已載入的 DLL。
4. 將目標 MFT 參照與檔案路徑與 `$MFT`／USN 記錄進行比對。
5. 將執行時間與事件記錄、SRUM、網路遙測資料、服務、工作排程及使用者跡證進行關聯分析。

## 延伸資訊 { #further-details }

- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch)
- [libyal libscca](https://github.com/libyal/libscca)
- [Security Ronin prefetch-forensic](https://github.com/SecurityRonin/prefetch-forensic)
