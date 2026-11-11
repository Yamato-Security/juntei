# SRUM { #srum }

## 什麼是 SRUM { #what-srum-is }

System Resource Usage Monitor（SRUM）以 ESE 資料庫儲存應用程式與服務的資源
使用情形。視 Windows 版本而定，它會記錄網路位元組數、介面／設定檔識別碼、
連線、CPU 時間、前景／背景活動、磁碟活動、能源使用、推播
通知，以及應用程式時間軸資料。各項目透過 `SruDbIdMapTable` 與應用程式 ID 及使用者
SID 建立關聯。

當執行檔已被刪除時，SRUM 特別有用：其應用程式 ID、使用者與
資源使用的資料列可能比檔案本身存在得更久。網路總量可以支持資料外洩的假設，但
SRUM 並非封包擷取，通常無法識別每一個遠端端點或傳輸的檔案。

<span class="evidence-strength">最佳結論：</span>在資料表的量測期間內，Windows 將所記錄的資源
使用量歸屬於該應用程式／帳戶。

## 位置與收集 { #location-and-collection }

```text
C:\Windows\System32\sru\SRUDB.dat
C:\Windows\System32\config\SOFTWARE   (context for account and Wi-Fi names)
```

SRUM 存在於 Windows 8 及更新版本。請以一致的方式複製 `SRUDB.dat`，最好從快照複製，或使用
鑑識收集工具。也請收集 `SOFTWARE` 以及 `sru` 目錄中的 ESE 交易／記錄檔。

對應的 SOFTWARE 登錄區與 ESE 資料庫不同：若登錄區處於 dirty 狀態，Juntei 會在記憶體中自動重播相鄰的登錄交易記錄 `SOFTWARE.LOG`、`.LOG1` 和 `.LOG2`，因此也請收集這些檔案。工具會顯示復原詳細資訊與警告；缺少記錄時，帳號或 Wi-Fi 名稱可能不完整或過時。這項處理不會重播 SRUM 資料庫的記錄。

!!! warning "Dirty 資料庫"
    Juntei 會依收集時的狀態讀取 dirty 的 `SRUDB.dat` 並加以回報。它目前不會重播
    `SRU*.log`，因此僅存在於這些記錄檔中的變更將會缺漏。請勿在未說明的情況下將
    dirty 資料庫描述為完整的資料。

## 分析注意事項 { #analyst-cautions }

- 許多資料列代表定期彙總，通常以整點為界——而非確切的動作時間。
- 位元組總量本身無法識別內容、意圖、通訊協定或目的地。
- 應用程式身分可能是路徑、服務、封裝應用程式或物件；請檢查 `AppType`。
- 使用者與 Wi-Fi 名稱是從對應的 `SOFTWARE` 登錄區補充而來。缺少名稱並不代表
  其底層的 ID 不存在。
- `null`、零、不存在的欄位、不支援的類型以及損毀的值，都是不同的狀態。
- 提供者的結構描述因 Windows 版本而異。請保留未知的資料表與原始欄位名稱。
- 背景執行的 Windows 服務會正當地產生大量的資源使用總量。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-ese srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SRUDB.dat
  -f, --file <FILE>      Parse one SRUDB.dat
  -r, --software <FILE>  Matching SOFTWARE hive for names

Output:
  -o, --output <PATH>        Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>   csv, json, or jsonl
      --include-value-state  Add per-value read state and raw bytes to JSON/JSONL
```

```bash
juntei parse-ese srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-ese srum -f SRUDB.dat -t jsonl \
  | jq 'select(.Provider == "NetworkUsages" and (.BytesSent // 0) > 100000000)'
juntei parse-ese srum -f SRUDB.dat -t jsonl --include-value-state \
  | jq '{Provider, Timestamp, ValueState, Raw}'
```

## 輸出資料表 { #output-tables }

CSV 會為 `SruDbIdMapTable` 建立 `SRUM-IdentityMap.csv`，並為每個提供者各建立一個資料表。已知的輸出名稱包括：

| 提供者／資料表 | 調查用途 |
|---|---|
| `NetworkUsages` | 每個應用程式、使用者、介面與設定檔所傳送／接收的位元組數。 |
| `NetworkConnections` | 連線次數／持續時間類型的活動，以及連線開始時間。 |
| `AppResourceUseInfo` | 前景／背景的 CPU、磁碟、網路及相關的應用程式使用情形。 |
| `AppTimelineProvider` | 應用程式時間軸活動，含結束時間與使用狀態欄位。 |
| `EnergyUsage` | 短期與長期的能源／提供者量測值；以 `IsLt` 區分。 |
| `PushNotifications` | 推播通知活動。 |
| `VfuProvider` | `vfuprov` 提供者；其詳細用途並無完善的文件記載，因此請審慎判讀其欄位。 |
| `EnergyEstimation`, `TaggedEnergy` | 依版本而異的能源與使用量提供者。 |
| `ProcessorTime`, `SizeInBytes`, `TotalUsed`, `NetworkBytes` | Windows Server 的提供者資料表。 |
| 未知的資料表名稱 | Juntei 會以其 GUID／資料表名稱保留未知的提供者，並輸出所有目錄欄位。 |

由於 SRUM 是由目錄（catalog）驅動，且 Windows 會增加提供者欄位，因此刻意不設定固定、
有限的提供者特定欄位清單。Juntei 會輸出**資料庫中存在的每一個欄位，並使用
Juntei 的 PascalCase 欄位標籤**。下列欄位是穩定／共通的輸出約定。

## 分類後的 auto-parse 輸出 { #categorized-auto-parse-output }

`auto-parse` 會將每個 SRUM 資料表直接放入其對應的調查類別。CSV、JSON 與 JSONL
使用相同的 `SRUM-` 檔名前綴；不會另外建立外層的 `SRUM/` 目錄。

| 類別 | CSV 檔案 |
|---|---|
| `NetworkActivity` | `SRUM-NetworkConnections.csv`, `SRUM-NetworkUsages.csv`, `SRUM-NetworkBytes.csv`, `SRUM-PushNotifications.csv` |
| `ProcessExecution` | `SRUM-AppResourceUseInfo.csv`, `SRUM-AppTimelineProvider.csv`, `SRUM-ProcessorTime.csv` |
| `SystemInformation` | `SRUM-EnergyUsage.csv`、`SRUM-EnergyEstimation.csv`、`SRUM-TaggedEnergy.csv`、`SRUM-SizeInBytes.csv`、`SRUM-TotalUsed.csv`、`SRUM-IdentityMap.csv`、`SRUM-VfuProvider.csv` 以及未知提供者的資料表 |

只會匯出擷取資料中存在的資料表。網路計數器與通知記錄依其主要調查用途
分類；這並不表示每一列都描述了一次遠端連線。身分對應表可支援多個類別。VFU 與未知提供者則保留在
`SystemInformation` 中，因為單憑其存在並不能確立有程式執行。

`SystemInformation/SRUM-logs.txt` 包含剖析器的診斷訊息，以及原始名稱與最終輸出名稱的
對應。兩份清單檔都會在 `OutputFiles` 中列出實際的資料表路徑；SRUM 工作的 `Output`
是電腦目錄，而 `Category` 則指出其共用記錄／中繼資料的位置。

有多個資料庫時，檔案名稱會在副檔名之前加上 `-Source1`、`-Source2` 等。
記錄檔會保留原始的擷取別名，而 JSON/JSONL 資料列則保留其 `SourceFile`、
`Table`、`Provider`、`Comp` 與 `Artifact` 出處資訊。不會將任何資料表複製到多個
類別中。合併後的 `-j` SIEM 時間軸會將每筆保留的 SRUM 記錄各包含一次。

獨立執行的 `parse-ese srum` 則維持所指定的輸出位置。其 CSV 資料表檔案同樣帶有
`SRUM-` 前綴；明確指定的 JSON/JSONL 輸出檔名則由分析人員自行決定。

## 共通輸出欄位 { #common-output-fields }

| 欄位 | 判讀 |
|---|---|
| `SourceFile` | 輸入資料庫；存在於 JSON/JSONL 中，在跨主機分析時很有用。 |
| `Table` | 原始的 ESE 資料表名稱／GUID。比較不同版本或未知提供者時請保留。 |
| `Provider` | 易讀的提供者／輸出名稱，若未知則為原始資料表名稱。 |
| `ID` | 資料列的 `AutoIncId`；這是資料庫的資料列識別碼，而非程序 ID。 |
| `Timestamp` | SRUM 記錄／彙總這些數值的時間，四捨五入至毫秒。通常不是確切的事件時間。 |
| `EXE-Info` | 從 `AppID` 解析出的應用程式／服務／套件身分。主要的程式交叉比對依據。 |
| `EXE-InfoDescription` | 從複合式 SRUM 應用程式名稱中擷取出的描述。僅供參考脈絡。 |
| `EXE-Timestamp` | 若存在，為內嵌於複合式應用程式身分中的時間；並非該資料列的使用時間。 |
| `AppType` | `NormalApp`、`Service`、`ModernApp` 或 `Object`；決定 `EXE-Info` 應如何判讀。 |
| `SID` | 從 `UserID` 解析出的帳戶 SID。強而有力的使用者／服務身分交叉比對依據。 |
| `User` | 從 `SOFTWARE` 補充而來的易讀設定檔名稱；即使 `SID` 有效，此欄也可能為 null。 |
| `UserID`, `AppID` | 指向 `SruDbIdMapTable` 的原始數值參照；在名稱無法解析或重複時請保留。 |
| `IsLt` | 長期 EnergyUsage 資料表為 `true`，短期則為 `false`。 |
| `InterfaceLUID` | 來自提供者資料列的原始網路介面 LUID。 |
| `InterfaceType` | 解碼後的介面類別（例如 `IF_TYPE_IEEE80211`）；這是類別，而非唯一的介面卡。 |
| `L2ProfileID` | 網路設定檔的數值參照。 |
| `ProfileName` | 從 `SOFTWARE` 補充而來的 Wi-Fi／設定檔名稱；可能屬於敏感資訊，且不能證明目前仍在連線。 |
| `ConnectStartTime`, `EventTimestamp`, `EndTime`, `StartTime` | 已解碼為 UTC 的已知提供者 FILETIME 欄位。其確切意義取決於提供者，並非通用的執行時間。 |
| 提供者特定欄位 | 匯出的提供者欄位，例如 `BytesSent`、`BytesReceived`（資料庫欄位 `BytesRecvd`）、CPU／磁碟計數器、持續時間、次數、旗標或估計值。請依提供者判讀單位，並在相同結構描述內比較資料列。 |

## 診斷欄位 (`--include-value-state`) { #diagnostic-fields-include-value-state }

| 欄位 | 判讀 |
|---|---|
| `ValueState` | 資料庫欄位對應至 `parsed`、`null`、`absent`、`unsupported` 或 `damaged` 的對照表。用以區分證據不存在與剖析器失敗。 |
| `Raw` | 具有位元組之值的原始位元組（十六進位）。用於驗證與日後的解碼。 |

`SRUM-IdentityMap.csv`（來自 `SruDbIdMapTable`）包含 `IdIndex`、`IdType` 與 `IdBlob`：即原始索引、身分類別，
以及提供者資料列所使用之已解碼的程式／SID／物件文字。

## 調查策略 { #investigation-strategy }

1. 以 `EXE-Info`、`AppType` 與 `AppID` 識別程式；請勿將 `Id` 與 PID 混淆。
2. 依程式、使用者、設定檔、介面與時間區間將網路資料列分組。
3. 將傳送／接收的位元組數與正常基準線及其他主機進行比較。
4. 將可疑的時間區間與 Prefetch、事件記錄、DNS／Proxy／防火牆資料以及檔案存取交叉比對。
5. 將 dirty 資料庫的狀態以及未重播交易記錄一事，作為收集限制記載於報告中。

## 延伸資訊 { #further-details }

- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [dissect.esedb](https://github.com/fox-it/dissect.esedb)
- [libyal libesedb](https://github.com/libyal/libesedb)
- [Microsoft：Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
