# Microsoft Defender 偵測歷程記錄 { #microsoft-defender-detection-history }

## 歷程記錄的內容 { #what-the-history-records }

Microsoft Defender 會為每一次偵測寫入一個偵測歷程記錄檔案。每個檔案記錄了：

- 威脅的識別碼、名稱、嚴重性與類別；
- Defender 採取了什麼動作，以及是否成功；
- 威脅當時是否正在執行；
- 相關的使用者與處理程序，以及是什麼偵測到它；
- 隔離項目（如果有的話）；
- 發現該威脅的每一個資源：檔案、封存檔成員、下載、處理程序、登錄值、服務、命令列與行為。

Defender 也會為從未隔離的威脅寫入這些檔案：僅偵測到的威脅、被允許的威脅，以及移除失敗的威脅。Windows 安全性的保護歷程記錄與 WMI 的 `MSFT_MpThreatDetection` 類別（`Get-MpThreatDetection`）會顯示這些記錄。當 Orange Cyberdefense 變更檔案中的值時，WMI 回報的是變更後的值。

<span class="evidence-strength">最佳結論：</span>Defender 曾在所列的資源中記錄到所指名威脅的偵測，並附有所記錄的狀態與時間。威脅名稱是產品的判定。單憑偵測記錄無法證明該威脅曾經執行，也無法證明它已被完全移除。

## 位置與收集 { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

每個檔案都以其 `DetectionID` 命名。請收集整個 `DetectionHistory` 資料夾。目錄輸入與 `auto-parse` 會讀取 `DetectionHistory` 資料夾內任何深度中、開頭與偵測歷程記錄檔案相符的每一個檔案。對於保存在此類資料夾以外的副本，請使用 `-f`。Defender 會在可設定的天數之後從掃描歷程記錄中移除項目（`Set-MpPreference` 的 `ScanPurgeItemsAfterDelay`），因此較舊的偵測可能已不存在。

隔離區是另一個獨立的跡證：請參閱 [Defender 隔離資料](defender-quarantine.md)。偵測的 `QuarantineID` 與隔離輸出的 `EntryID` 採用相同的寫法（大寫、不含大括號），因此兩者可以互相關聯。

## 分析注意事項 { #analyst-cautions }

- 偵測、修復嘗試與成功移除是不同的狀態。請將 `ThreatStatus`（`Detected`、`Quarantined`、`Removed`、`Allowed`、`RemoveFailed`、…）與 `ThreatStatusErrorCode` 一併判讀；最後一次狀態變更成功時，後者為 0。
- `ExecutionStatus` 是 Defender 在偵測當下的判斷（`Blocked`、`Allowed`、`Executing`、`NotExecuting`、`Unknown`）。它既不能證明威脅曾經執行，也不能證明威脅從未執行。請以 Prefetch、Amcache、事件記錄與處理程序遙測交叉驗證。
- `ThreatName` 是當時的特徵碼名稱。可能有誤判，之後也可能被重新分類。EICAR 等測試檔案會被當成真正的威脅回報。
- `User` 是 Defender 所記錄的 `DomainUser` 值（`UserSource` 為 `DomainUser`）。它可能為空、是服務帳戶，或是與建立該檔案者不同的使用者。
- Defender 可能為同一個威脅寫入多個檔案，而 Juntei 會保留其中每一個檔案。兩個檔案甚至可能具有相同的 `DetectionID`，複製或還原的檔案即是如此。請同時依 `DetectionID` 與 `SourceFile` 分組。
- 意義不明的值會予以保留而不加以解讀：其區段與位置記錄在 `UnknownValues` 中。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

CSV 是預設檔案格式。指定 `-t csv`、`-t json` 或 `-t jsonl` 而不指定 `-o` 時，記錄會寫至標準輸出。兩個選項都不指定時，只顯示摘要。加入 `-C` 可覆寫現有輸出。即使使用 `-C`，輸出也絕不會寫入並覆蓋任何一個輸入檔案。

Juntei 會為每次偵測的每個資源寫入一筆記錄。沒有資源的偵測會產生一筆資源欄位為空的記錄。同一偵測的各筆記錄共用其 `DetectionID`，並依儲存順序以 `ResourceIndex` 編號。

損毀不會中斷執行。被截斷的檔案會保留損毀處之前讀取到的值，並由 `Problems` 說明缺少了哪些內容。位於 `DetectionHistory` 資料夾中、但不是偵測歷程記錄檔案的檔案會被回報並略過。其他檔案仍會照常讀取。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Record` | `resource`；沒有資源的偵測則為 `detection`。 |
| `InitialDetectionTime` | Defender 首次偵測到該威脅的時間（UTC）。檔案中的值為 0 時為空；不是時間的值會保留在 `UnknownValues` 中。 |
| `LastThreatStatusChangeTime` | 威脅狀態最後一次變更的時間（UTC）。 |
| `RemediationTime` | Defender 修復該威脅的時間（UTC）。這是嘗試的時間，並不能證明已經移除；請檢查 `ThreatStatus`。 |
| `ThreatName`、`ThreatID` | Defender 的特徵碼名稱與數值識別碼（`Get-MpThreatCatalog`）。 |
| `Severity`、`SeverityID` | `Low`、`Moderate`、`High`、`Severe` 或 `Unknown`，以及所儲存的數值。 |
| `Category`、`CategoryID` | 威脅類別，例如 `TROJAN`、`TOOL` 或 `BEHAVIOR`，以及所儲存的數值。 |
| `ThreatStatus`、`ThreatStatusID` | Defender 對該威脅判定的狀態。沒有文件記載名稱的數值會予以保留，而 `ThreatStatus` 為空。 |
| `ThreatStatusErrorCode` | 最後一次狀態變更的錯誤碼；0 表示成功。 |
| `ExecutionStatus`、`ExecutionStatusID` | 偵測當時 Defender 是否看到該威脅正在執行。 |
| `DetectionSource`、`DetectionSourceTypeID` | 是什麼偵測到它：`Real-time`、`IOAV`（下載與附件）、`User`、`System`、`AMSI` 等。 |
| `User`、`UserSource` | Defender 所記錄的 `DomainUser` 值及其依據（`DomainUser`）。檔案中未指名使用者時為空。 |
| `Process` | 相關的處理程序（`ProcessName`）；可能為 `Unknown`。 |
| `ResourceIndex`、`ResourceCount` | 此資源的位置，以及該偵測中的資源數量。 |
| `ResourceType` | `file`、`containerfile`、`webfile`、`process`、`regkeyvalue`、`service`、`CmdLine`、`behavior`、`amsi` 等，沿用 Defender 的命名。 |
| `Path` | 資源本身：路徑、封存檔成員（`archive.zip->member`）、登錄值、命令列或處理程序。`webfile` 也會指明其 URL 與進行下載的處理程序（`pid:…,ProcessStart:…`）。請保留確切的拼寫。 |
| `ResourceFlags` | 以十六進位表示的資源旗標；其意義沒有文件記載。 |
| `TrackingStartTime` | 資源的 `ThreatTrackingStartTime`，是一個被解讀為 FILETIME（UTC）的整數，與 plaso 為偵測標定日期的方式相同。 |
| `SHA256`、`SHA1`、`MD5`、`TrackingSize` | 來自資源追蹤資料的雜湊值與大小。它們描述的是 Defender 掃描該檔案時的檔案。 |
| `TrackingID` | 資源的 `ThreatTrackingId`。 |
| `OtherTracking` | 其他所有追蹤值，依儲存順序以 `Key=Value` 表示，例如行為偵測的 `ThreatTrackingScanSource` 或 `ThreatTrackingBMContextRich`。 |
| `QuarantineID` | 隔離項目（`Quarantine\Entries\{GUID}`）；檔案中為全零 GUID 時為空。 |
| `DetectionID` | 偵測的 GUID；也是檔案的名稱。兩者不一致時會在 `Problems` 中回報。 |
| `MagicVersion` | 標頭的 `Magic.Version:1.2`；在寫入時未包含此值的檔案中為空，例如部分來自 Windows Server 2016 的檔案。 |
| `UnknownValues` | 意義不明的值，以 `position=value` 表示（例如 `6.13=NT AUTHORITY\SYSTEM`），狀態之後的計數清單則以 `2.list` 表示。 |
| `SourceFile` | 所讀取的檔案，為相對於輸入目錄的路徑；使用 `-f` 時則為其檔名。 |
| `Problems` | 損毀、缺少的值、非預期的型別，或不相符的檔名。請勿隱藏。 |

JSON 與 JSONL 保留相同的欄位，但值帶有型別：數字維持為數字，缺少的值為 `null`。它們以 `Tracking` 取代 `OtherTracking`，其中包含每一個追蹤值，以及 Defender 的鍵名、型別與值。`UnknownValues` 會變成附有型別的位置清單。`CountedList` 保存計數清單。所儲存的 FILETIME 會在其解讀出的時間旁保留原始的 tick 數。

## 時間軸行為 { #timeline-behavior }

`auto-parse` 會將記錄儲存為 `SecurityProducts/DefenderHistory.csv`（或 `.json`／`.jsonl`），並加入合併的 `-j` 時間軸。`super-timeline` 會從每筆資源記錄上的 `InitialDetectionTime`、`LastThreatStatusChangeTime` 與 `RemediationTime` 產生事件，並在事件詳細資料中附上狀態、錯誤碼、執行狀態與偵測來源。`TrackingStartTime` 是以整數儲存的，因此只有在使用 `--include-interpreted-times` 時才會成為事件。絕不會使用檔案本身的時間。沒有可用時間的記錄會保留在剖析輸出中，並計為無時間的記錄。

## 調查策略 { #investigation-strategy }

1. 從 `ThreatName`、`Path`、`SHA256`、`InitialDetectionTime` 與 `ThreatStatus` 著手。
2. 在斷定威脅已被移除之前，先檢查 `ThreatStatusErrorCode` 與 `Problems`。
3. 將 `QuarantineID` 與隔離資料的 `EntryID` 互相關聯，以復原被隔離的位元組。
4. 以雜湊值與路徑為樞紐，查詢 `$MFT`、USN 日誌、Prefetch、Amcache、瀏覽器歷程記錄與事件記錄（Defender 的操作記錄：事件 ID 1116、1117、1118 與 1119）。
5. 對於 `webfile` 資源，使用 URL 與處理程序啟動時間調查初始存取，並以瀏覽器、Proxy 與 DNS 證據加以驗證。

## 測試過的版本 { #tested-versions }

測試使用的是公開檔案，其結果已由其他工具獨立讀取：

- Orange Cyberdefense 的 Windows 10 範例（Defender 4.18.24090.11），已與 WMI 回報的內容比對。
- plaso 的兩個 Windows 測試檔案，已與 plaso 的 `windefender_history` 剖析器比對。
- 來自 DFIRArtifactMuseum 之 APT 模擬器 VM 的 14 個檔案。
- 一份寫入時未包含標頭版本的副本，與部分來自 Windows Server 2016 的檔案相同。
- 被截斷、空白或其他種類的檔案。

## 延伸資訊 { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark 的 defender](https://github.com/Sootmark/defender)
- [Orange Cyberdefense 的 wddh-parser](https://github.com/cert-orangecyberdefense/wddh-parser)
- [plaso 的 Defender 歷程記錄剖析器](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Velociraptor Windows.Applications.DefenderHistory](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft：MSFT_MpThreatDetection 類別](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft：Microsoft Defender 防毒軟體事件 ID](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
