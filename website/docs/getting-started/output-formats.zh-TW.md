# 輸出格式 { #output-formats }

## 終端機摘要 { #terminal-summaries }

未指定 `-o` 或結構化的 `-t` 時，命令通常會顯示筆數、時間範圍、損毀情形以及高價值發現。
摘要可協助您在產生大量輸出之前先掌握範圍。

## CSV { #csv }

CSV 是預設的儲存格式，適合用於試算表工具、SQL 匯入與時間軸檢視器。有些剖析器會產生多個資料表，
因為其跡證包含不同的記錄類型（Amcache、Jump Lists 與 SRUM）。

!!! warning "請將證據欄位視為文字"
    路徑或命令列可能以 `=`、`+`、`-` 或 `@` 開頭，試算表軟體可能會將其解譯為公式。
    請將所有欄位以文字匯入，或使用鑑識安全的 CSV 檢視器。Juntei 會保留證據字串的原貌，不會加以修改。

## 欄位名稱 { #field-names }

CSV 欄位標題與 JSON/JSONL 欄位名稱均使用 PascalCase，包括 Amcache 與
`auto-parse` 清單檔（manifest）。縮寫維持大寫：`ProcessID`、`TimestampUTC`、`UserSID`、
`SHA256`。縮寫後接的單字以連字號分隔：`IP-Address`、`MFT-Entry`、
`USN-Journal`。使用 `jq` 時，含連字號的欄位請加上引號：`.["IP-Address"]`。

預設的 MFT CSV 使用較短且易讀的標題，例如 `EntryNum`、`Ext`、`Dir`、`HasADS`、
`Size`、`Created (SI)` 與 `Created (FN)`。完整的 CSV 結構描述請參閱 [MFT 欄位參考](../artifacts/mft.md)。
時間戳記成對出現時依 MACB 順序排列：DataModified、Accessed、
MetadataChanged、Created，且 SI 在 FN 之前。在 CSV 工具中，含有空白或標點符號的標題請加上引號。

證據值保留其原始拼寫。來源提供的字典鍵，例如 `Details`/`DataRaw` 之下的登錄值名稱，以及
`ZoneIdentifier` 之下的 `Zone.Identifier` 名稱，也維持不變。SRUM 的 `ValueState` 與 `Raw` 字典
使用與其資料欄位相同的標籤。若正規化後發生名稱衝突，會加以回報，而不會默默捨棄欄位。

這會變更先前的匯出結構描述。請重新產生既有的結果並更新已儲存的查詢
（例如 `.record_type` 變為 `.RecordType`，Amcache 的 `full_path` 變為 `FullPath`）。
供人閱讀的文字報告以及輸入／設定格式則維持既有的慣例。

## 縮寫的欄位名稱 { #abbreviated-field-names }

下列標籤適用於 CSV、JSON 與 JSONL 匯出，包括自動解析與清單檔。重新產生較舊的結果時，
請更新已儲存的 `jq`、`xsv` 與試算表篩選條件。
僅輸出標籤有所變更；證據值與原始登錄值名稱保留其原始拼寫。

| 先前的標籤 | 目前的標籤 |
|---|---|
| `ADS` | `HasADS` |
| `Application` | `App` |
| `ApplicationCompany` | `AppCompany` |
| `ApplicationCompanyRaw` | `AppCompanyRaw` |
| `ApplicationPath` | `AppPath` |
| `Arguments` | `Args` |
| `Attributes` | `Attr` |
| `AttributeType` | `AttrType` |
| `BytesRecvd` | `BytesReceived` |
| `Computer` | `Comp` |
| `ComputerName` | `CompName` |
| `ComputerNameLastWrittenUTC` | `CompNameLastWrittenUTC` |
| `Directories` | `Dir` |
| `FileName` | `Filename` |
| `MicrosecondZeros` | `ZeroU-Secs` |
| `SI-BeforeFN` | `SI<FN` |
| `UserName` | `User` |
| `Username` | `User` |
| `UserNameSource` | `UserSource` |
| `Users` | `User` |

MFT CSV 輸出使用專用欄位名稱：`File` (Filename)、`SN` (Sequence Number)、`ParentEntryNum` (ParentEntryNumber)、`ParentSN` (Parent Sequence Number)、`USN` (Update Sequence Number)、`LogfileSN` (Logfile Sequence Number)、`SID` (Security ID)、`Type` (Name Type) 和 `RefCnt` (Reference Count)。`auto-parse` 的 MFT CSV 也使用相同名稱；請參閱 [MFT 輸出欄位](../artifacts/mft.md#csv-output-fields)。

原生系統資訊 JSON 的時間欄位為 `CompNameLastWritten`（CSV：
`CompNameLastWrittenUTC`）。`HasADS` 是布林指標；Prefetch 中的 `Dir` 包含所記錄的目錄清單。
`User` 保留各跡證特有的型別：帳戶名稱時為字串，USB 輸出中則為相關使用者的清單。
它並不代表執行時的身分。在查詢中請為標點符號加上引號：`."SI<FN"` 與 `.["ZeroU-Secs"]`。

## JSON { #json }

JSON 將完整的結果儲存為單一文件。它會保留陣列、巢狀結構、null、
大整數，以及扁平的 CSV 不一定能表達的區別。

```bash
juntei parse-file lnk -f sample.lnk -o lnk.json -t json
```

## JSON Lines { #json-lines }

JSONL 每行寫入一個獨立的 JSON 物件。它適合串流處理，也能自然地搭配 `jq` 使用：

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.StartType == "automatic") | {Host, ServiceName, ImagePath}'
```

當結構化輸出寫往標準輸出時，Juntei 會將進度與訊息送往標準錯誤，讓管線只會收到資料。

`auto-parse -j timeline.jsonl` 會為完整的多主機收集資料額外建立一份 JSONL 資料集，
同時保留分類後的 CSV 結果。加上 `-t json` 或 `-t jsonl` 可變更個別結果的格式。每筆記錄都有
`Comp` 與 `Artifact` 欄位，因此匯入 SIEM 後仍可清楚辨識其來源。

## UTC 與精度 { #utc-and-precision }

在格式允許的情況下，Juntei 會以 RFC 3339 的 UTC 格式呈現剖析出的時間。有些跡證具有
100 奈秒的精度；有些則儲存秒、FAT 時間、彙總區間，或根本不儲存時間。
位數多不代表準確：請務必了解各跡證頁面所說明的來源欄位。

UTC 採用 Windows 事件記錄的格式 `2026-10-20T08:46:38.2001668Z`，並保留可用的
小數精度。`auto-parse -Z "+09:00"` 會在其所有結果格式以及選用的 `-j` 匯出中，
以明確的固定時差顯示剖析出的時間點。無論執行 Juntei 的電腦設定為何，預設皆為 UTC。
原始值與未帶時區的當地時間則保持記錄時的原樣；請參閱
[輸出時區](../commands/auto-parse.md#output-time-zone)。

## 損毀與缺漏值 { #damage-and-missing-values }

空值、`null`、`NA`、`Problems` 以及剖析器特有的狀態欄位都具有意義。它們可能代表
跡證沒有儲存該值、記錄已損毀、格式不支援該值，或是剖析器無法加以解析。請勿將所有空值都替換為零。

## 標準化時間順序 { #normalized-chronological-timeline }

[`super-timeline`](../commands/super-timeline.md) 建立一個通用的 CSV/JSONL 事件模式，其中包含標記的時間戳意義和來源引用。預設輸出是未排序的； `--sort` 支援按時間順序排序和重複合併。它還支援 Timesketch 導入設定檔。本機解析器匯出和 `auto-parse -j` 保留其現有架構。
