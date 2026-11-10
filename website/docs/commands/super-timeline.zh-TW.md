# 超級時間軸 { #super-timeline }

`super-timeline` 將解析的工件轉換為具有公共模式的事件。輸出**預設未排序**：它遵循發現的輸入檔案順序和每個解析器的投影順序，獨立於工作器完成順序。新增 `--sort` 以按時間順序排序和重複合併。一筆Prefetch記錄可以包含多個執行時間； MFT 記錄可以包含八個或更多檔案時間戳記。每個事件都成為單獨標記的事件。這樣可以更輕鬆地比較來自不同來源的證據，而不會失去每個時間戳的實際意義。

從[`auto-parse`](auto-parse.md)的結果開始，而不是原始證據目錄：

```bash
juntei auto-parse -d evidence/ -o parsed/
juntei super-timeline -d parsed/ -o SuperTimeline.csv
```

保留解析後的文件、清單和原始證據。時間軸是用於分析的預測；它不會取代這些記錄或建立本機 Plaso 儲存檔案。

## 用法 { #usage }

```bash
# Native CSV (default)
juntei super-timeline -d parsed/ -o SuperTimeline.csv

# Chronological CSV with duplicate consolidation
juntei super-timeline -d parsed/ -o SortedTimeline.csv --sort

# Native JSON Lines
juntei super-timeline -d parsed/ -o SuperTimeline.jsonl -t jsonl

# Combine native records and an existing Hayabusa JSONL timeline
juntei super-timeline -f NativeRecords.jsonl --hayabusa Hayabusa.jsonl \
  -o SuperTimeline.jsonl -t jsonl

# Timesketch import profile; also available with CSV
juntei super-timeline -d parsed/ -o Timesketch.jsonl -t jsonl --profile timesketch

# One standalone CSV needs an artifact identifier and, if unavailable, a computer name
juntei super-timeline -f MFT.csv --input-artifact mft --computer HOST01 \
  -o Host01Timeline.csv

# Reduce the time range, memory used by sorting, or number of workers
juntei super-timeline -d parsed/ -o IncidentTimeline.jsonl -t jsonl \
  --timeline-start "2026-10-20T06:00:00Z" --timeline-end "2026-10-20T14:00:00Z" \
  --sort --memory-limit 64 -T 4 --temp-dir scratch/
```

`--hayabusa FILE` 可以重複或單獨使用。它會匯入現有事件/警報時間戳記和規則上下文，而無需執行 Hayabusa。支援 `Details` 為字串的資訊事件。 `--artifact mft,prefetch` 將投影限制為選定的工件標識符。時間限制包括在內，並且需要明確的 UTC 偏移量或 `Z`。

### 選擇輸入 { #choosing-an-input }

| 輸入 | 如何識別 |
|---|---|
| `-d DIR` | `manifest.json` 提供成功的主要任務、工件 ID、電腦和準確的輸出檔案。輔助 `auto-parse -j` 任務被排除以防止匯入兩個副本。 |
| `-f FILE` | 本機 JSON/JSONL 使用嵌入式 `Artifact` 和 `Comp` 欄位。無標籤導出需要`--input-artifact`；獨立的 CSV 始終需要它。 |
| `--computer NAME` | 當輸入記錄沒有電腦名稱時回退；否則，未知電腦明確為 `Unknown`。 |
| `--hayabusa FILE` | JSONL 與 Hayabusa 時間戳記、規則元資料、計算機以及 `EvtxFile`（如果存在）。 |

目前清單支援解析目錄的重定位：記錄的絕對輸出路徑在新目錄下解析。輸入必須保留在其中，包括符號連結目標。支援舊版每個工件 Amcache、SRUM 和 Jump Lists 目錄；類別目錄需要顯式 `OutputFiles`。

支援具有輸出相對路徑和扁平多表導出的目前清單。此指令可辨識自動解析安排的每個工件，包括較新的瀏覽器、WebCache、Outlook 和 UAL 表。

當清單標識 VHDX 或 E01 映像時，來源引用包括影像和內部路徑，且影像名稱保留在事件上下文中。不需要暫時暫存工作區來讀取解析的結果。

原始來源取自記錄或明確的清單指令。對於 `-d`，原始來源引用與證據目錄相關，`Details.ParsedFile` 和摘要 `InputFiles` 與解析目錄相關。摘要輸出/報告名稱與時間軸的輸出目錄相關。只有當主機具有一個唯一可識別的收集的 SRUDB.dat 時，SRUM CSV 表才會分配給資料庫。否則 `SourceFile` 指向解析的文件，`Details.RawSourceAvailable` 是 `false`。

獨立輸入由其檔案名稱引用。具有相同檔案名稱的輸入使用不同的 `./Inputs/<ID>/<filename>` 引用。已知證據根以外的絕對原始來源路徑使用 `./Sources/<ID>/<filename>` 標識符。這些不透明的引用保留了來源區別，而不導出分析人員的絕對目錄名稱；使用 `ParsedFile` 和 `InputRecord` 查閱原始記錄以獲取原始來源。證據中記錄的路徑（例如 Windows 可執行路徑）保持不變。

## 公共事件字段 { #common-event-fields }

| 場地 | 意義 |
|---|---|
| `Timestamp` | UTC RFC 3339，用九個小數位書寫，以保持順序一致。填充不會增加精度。 |
| `Comp` | 與輸入記錄關聯的計算機。 |
| `Artifact` | 穩定的工件標識符，例如 `mft`、`prefetch`、`srum` 或 `hayabusa`。 |
| `TimestampType` | 時間戳含義的描述，例如`MFT SI Data modified`。 |
| `User` | 當適配器有一個時，記錄的帳戶或SID；可能為空。 |
| `Path` | 受影響的文件、登錄項目或其他特定於工件的物件。 |
| `Message` | 時間軸檢視器的簡短描述。 |
| `SourceFile` | 證據相關的原始來源引用，或當其根未知時的相對來源標識符；否則，上述解析文件後備。 |
| `SourceField` | 用於時間戳記的精確解析欄位或嵌套位置。 |
| `RecordID` | 特定於工件的記錄識別。它不一定是 EVTX 記錄 ID。 |
| `Details` | 結構化上下文包括原始時間戳記、精度、時間基礎、時間角色、解析文件、輸入記錄/位置和可用恢復/規則元資料。 |

在 CSV 中，`Details` 是引用的 JSON 單元。在 JSONL 中它是一個物件。 Timesketch 設定檔分別將 `Timestamp`、`TimestampType` 和 `Message` 變更為 `datetime`、`timestamp_desc` 和 `message`，同時保留其他來源欄位。這些是 [ 三個必要的 Timesketch 導入欄位](https://timesketch.org/guides/user/import-from-json-csv/)。

`Details` 故意包含選定的上下文，而不是完整的二進位資料或每個本機記錄欄位。長字串、陣列和物件有界並標記為 `ProjectionOmitted`。尋找 `Details.ParsedFile`、`InputRecord` 和 `Location` 以檢查完整的本機記錄。對於 CSV，記錄編號對標頭後面的 CSV 記錄進行計數，而不是帶引號的單元格內的實體行。對於 JSONL，它是行號；對於 JSON，它是頂級文件編號加上巢狀位置。

## 時代意味著什麼 { #what-the-times-mean }

時間戳是有關記錄對象的證據。它本身並不是執行或攻擊的證據。適配器有意保留這些差異：

| 文物 | 時間軸解讀 |
|---|---|
| MFT | 單獨的 SI 和 FN 修改、存取、元資料變更和建立時間；名為 FN 的屬性仍然不同。使用 `--sort`，檔案名稱行上重複的 SI 欄位將會合併。 ADS 行的共享基本檔案時間在兩種模式中均被排除。恢復的 FILE 記錄鬆弛名稱保留單獨的恢復的 FN 事件。 |
| I30 | 儲存的 FN/索引元數據，以及可用的鬆弛/恢復上下文。這些不是目錄索引寫入的時間戳記。 |
| USN 期刊 | 日誌記錄時間戳記和更新原因。一筆記錄可以描述多個變化；它不一定會記錄他們的精確開始時間。 |
| LogFile | 儲存的重做/撤銷時間戳快照和檔案名稱元資料。重做和撤消保持分開。 LSN 和檔案偏移量是標識符，永遠不會轉換為日期。 |
| Prefetch | 記錄執行時間。透過 `--sort`，這些內容與匹配的 Prefetch 時間軸導出進行合併。引用的磁碟區建立元資料是單獨標記的。 |
| 瀏覽器 SQLite 資料庫 | 記錄的存取、恢復的頁面上次存取元資料、下載開始/結束、cookie 建立/存取/過期以及表單歷史記錄首次/上次使用或刪除墓碑。下載結束本身並不證明成功或執行； cookie 過期是一個配置的截止日期。恢復狀態、信心和起源仍然在上下文中。 |
| Internet Explorer / 舊版 Edge WebCache | 儲存條目存取、建立/修改、到期和同步、下載開始、cookie 元資料和容器存取/清理。快取簿記不會自動證明使用者開啟頁面。 |
| 展望 PST、OST、MSG 和 OFT | 訊息提交/傳遞；資料夾、專案和附件建立/修改；日曆開始/結束/提醒；任務開始/到期/完成/提醒；以及明確鍵入的時間屬性。預定時間並不能證明會議發生、提醒已觸發或程序已執行。儲存的傳遞時間並不能證明該訊息已被閱讀。 |
| 使用者存取日誌記錄 (UAL) | 第一次/最後一次存取元資料、資料庫插入/建立、第一次/最後一次看到和儲存的作業系統啟動時間。角色、帳戶、來源位址和每日計數器仍然是上下文。每日日期/計數不是午夜登入發明的；無定時身分和 DNS 行在伴隨報告中仍然可用。 |
| Amcache | 庫存金鑰上次寫入、父卷金鑰上下文、記錄的安裝日期和 PE 連結元資料。庫存存在或 LinkDate 本身並不能證明執行。 |
| ShimCache | 儲存的檔案修改時間、快取更新中繼資料及快取鍵最後寫入時間。這些都不能自動視為執行時間戳記。 |
| SRUM | 提供者時間戳/儲存桶、連線開始、明確開始/結束間隔欄位和應用程式識別碼元資料。位元組、持續時間和其他計數器不是日期。 |
| LNK | 目標 MAC 時間戳記、追蹤標識符建立和 shell 項時間。它們描述儲存的目標元數據，不一定是在開啟捷徑時。 |
| Jump Lists | 輸入上次使用元資料、追蹤標識符建立和嵌入的 LNK 目標時間。預設忽略收集端容器檔案時間。 |
| Defender 檢疫 | 條目時間戳記和儲存的資源建立/存取/寫入時間。提取時間並不是終點活動的證據。 |
| BAM, UserAssist, FeatureUsage | 記錄的活動欄位（如果存在），關鍵的最後寫入上下文保持不同。沒有時間的計數器不會建立事件。 |
| RecentDocs、MRU、開啟/儲存 MRU、ShellBags、Office | 記錄使用/信任/shell 項時間和關鍵上下文。清單的關鍵時間不是清單中每個項目的單獨最後使用時間。 |
| SAM 和 LSA 秘密 | 記錄帳戶登入/密碼/過期或秘密設定元資料以及相關關​​鍵時間。這些不會取代身份驗證事件日誌。 |
| 服務，TaskCache，登錄持久性，BCD | 密鑰/配置元資料和明確記錄的任務時間。服務金鑰寫入本身並不能證明服務執行。 |
| 網路和 USB | 網路設定檔/租賃和設備安裝/到達/刪除字段，以及註冊表項上下文。原生USB用戶MountPoints2次保留對應的用戶hive來源。 |
| 系統資訊、已安裝程式、AppCompat、應用程式歷史記錄 | 明確安裝、關閉、庫存/活動和關鍵元資料欄位。本機系統資訊設定檔/版本陣列保留其嵌套時間。 |
| 審核原則、安全設定、MuiCache、通用登錄 | 提供關鍵的最後寫入時間；不定時的值/計數器在本機輸入中仍然可用。 |
| Persistence Hunter | 支援關鍵最後寫入和相關變更時間。發現本身並不是帶時間戳記的安裝或妥協的判決。 |
| Hayabusa | 事件/警報時間戳，包含規則、等級、通道、事件 ID 和可用的詳細上下文。多個規則和不同的相關行保持獨立。 |

預設匯出保留所有時間戳投影，包括重複的元資料。它不執行全域排序或重複合併。使用 `--sort`，可以合併重複的投影；與相同輸入的未排序導出相比，這可以產生更少的行。兩種模式都會保留多個具有不同意義或證據來源的相等時間戳。重複身分包括電腦、來源、工件、記錄、時間意義和恢復狀態。合併行記錄 `DuplicateCount` 和有界替代路徑/來源欄位。不同的來源永遠不會因為它們的路徑和時間戳看起來相似而被合併。當無法解析原始來源時，將保留輸入記錄標識以避免折疊不同的擷取。

### 精度、當地時間和假設 { #precision-local-times-and-assumptions }

已知的 UTC 欄位和具有明確偏移量的時間戳記被標準化為 UTC。保留原始值及其記錄的精確度，包括 100 ns FILETIME。來源解析器或下游檢視器可能已經捨去了時間；該指令無法恢復遺失的精確度。

即使上游解析器使用 `Z` 序列化某些 shell-item/FAT 字段，它們也具有本地時間語義。時間軸保留解析值和欄位原點；它無法恢復遺失的端點時區或驗證上游轉換。在使用這樣的時間進行精確的跨主機關聯之前，請先驗證原始工件和端點設定。

診斷出沒有工件定義時區的原始時間戳，而不是默默地將其視為 UTC。如果證據證明這一點是合理的，請提供一個帶符號的固定偏移量：

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

僅日期值也需要該選項。他們的午夜錨點被標記為具有**日精度**的假設；這不是觀察到的午夜事件。不接受時區名稱和夏令時規則。缺失值、零值、無效值和超出範圍的值不會產生合成紀元事件。保留明確記錄的有效 1970 日期。

預設排除兩個可選類別：

- `--include-collection-times`：收集/分析檔案系統上的 LNK/跳轉清單來源檔案元資料。這些事件標記為 `collection_filesystem`，與端點活動分開。
- `--include-interpreted-times`：由登錄讀取器呈現為 FILETIME 的通用登錄 QWORD 值。它們被標記為 `interpreted_value`，因為許多此類值實際上是計數器。

## 同儕報告 { #companion-reports }

對於 `SuperTimeline.csv`，該指令也寫入：

- `SuperTimeline.csv-Untimed.jsonl`：沒有可用映射事件時間戳記的本機記錄位置。
- `SuperTimeline.csv-Diagnostics.jsonl`：無效/不明確的時間，故意跳過需要原始解析記錄的共享元資料和保真度限制。
- `SuperTimeline.csv-Summary.json`：輸入檔案、每個工件計數、過濾/候選/合併事件、輸出計數、工作人員、排序預算/區塊/通道和經過的時間。

伴隨名稱包括完整的輸出檔名。例如，`SuperTimeline.jsonl`有自己的`SuperTimeline.jsonl-Summary.json`和相應的不定時/診斷報告。 CSV和JSONL出口具有相同的莖可以共存；僅在替換現有結果集時才需要 `-C`。

有效日期等待合理的時區假設增量 `UnresolvedTimeZones`，而不是 `InvalidTimestamps`。兩者在診斷中都保持可追溯性；沒有發明時間戳記。

有效時間超出請求範圍的記錄將被視為已過濾，而不是未計時。候選事件計數先於重複合併。 `Sorted` 和 `DuplicateConsolidation` 記錄 `--sort` 是否啟用。如果沒有它，`DuplicateEvents`、`InitialSortChunks` 和 `MergePasses` 為零，`SortMemoryBudgetMiB` 為 `null`，因為沒有分配排序緩衝區。 `Complete: true` 表示所有選定的解析輸入均已成功處理；這並不表示每個工件都被收集，每個欄位都帶有時間戳記或每個攻擊都是可偵測的。

CSV 匯出可以展平巢狀元資料：系統資訊設定檔、USB 使用者掛載點、shell 專案和其他結構化上下文可以在本機 JSON/JSONL 中更加豐富。當這些欄位很重要時，請使用本機輸入。時間軸無法重建 CSV 匯出忽略的資料。不受支援的工件 ID 和格式錯誤的輸入檔案會導致錯誤，而不是包含部分結果的成功。

## 性能和輸出安全 { #performance-and-output-safety }

輸入被並行傳輸和處理。預設情況下，每個worker都會寫入一個格式化的事件片段；片段按輸入檔案順序連接，無需排序或重新讀取其事件結構。即使處理多個文件，CSV 也只有一個標頭。輸出順序在不同工人數量之間是確定的；它並不意味著時間順序。

使用 `--sort`，緩衝區溢位到私有暫存目錄，然後有界合併過程產生依時間順序排列的輸出。排序使用完整整數秒和奈秒，包括 1970 之前的日期。等時排序在工人數量和排序預算方面具有確定性。進度條針對中間和最終合併期間處理的每個事件前進，包括重複通過。它出現在終端機上，並在輸出重定向時隱藏。

`--memory-limit` 僅適用於 `--sort`：它是總計 **排序緩衝區預算**，而不是進程 RSS 限制。輸入記錄、JSON 物件/字串開銷、CSV 緩衝區和合併記錄需要額外的記憶體。對於小預算，工人數量會減少。 `--max-record-size` 限制已解析的輸入記錄（預設 16 MiB）；一個漂亮的 JSON 陣列/報告算是頂級輸入文件。較大的文件可能需要更高的限制，或者最好是 JSONL。 CSV 解析器預讀也會導致此限制。

除了最終結果之外，臨時磁碟空間還必須容納格式化的事件片段（預設）或擴充的排序記錄和合併通道 (`--sort`)。兩種模式都保留臨時輸出分段並防止發布失敗的輸入。一筆記錄可以產生許多事件，因此時間軸可以比輸入大得多。在適當的時候過濾時間範圍/工件，並選擇具有足夠可用空間的臨時磁碟區。

Ctrl+C 協同取消，刪除私人排序/暫存目錄並退出，代碼為 `130`。在 Linux 和 macOS 上，這也適用於 SIGTERM 和 SIGHUP。在發布開始之前，在閱讀、標準化、合併和編寫報告時會檢查取消情況。無法捕獲的終止或斷電仍然可能留下臨時檔案。

現有輸出需要 `-C`。時間軸與輔助報告不得覆寫輸入、資訊清單、引用的原始證據或受保護的證據目錄。輸入或排序失敗時，既有輸出保持不變。完成的檔案先暫存於目的地旁，再逐一替換，最後發布時間軸。這**不是**涵蓋四個檔案的整體交易：最終寫入發生 I/O 錯誤時，部分輔助報告可能已被替換。如需保留先前的完整匯出，每次執行應使用新的基本檔名。

## 更多詳情 { #further-details }

- [Timesketch CSV/JSONL 導入](https://timesketch.org/guides/user/import-from-json-csv/)
- [Plaso 時間戳擴充與舊版 CSV](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Plaso 輸出與格式化](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [證據與解釋](../overview/evidence-model.md)
- [輸出格式](../getting-started/output-formats.md)

目前的 Juntei 輸入使用縮寫標籤，例如 `Comp`, `Filename`, `Args` 和 `Attr`. MFT CSV 也接受目前的 `File`, `SN`, `ParentEntryNum`, `ParentSN`, `Type`, `USN`, `LogfileSN` 和 `HasADS` 標頭，保留序號產生和 DOS/ADS 差別完好無損。舊的出口標籤仍然被接受，並且 `SourceField` 保留實際輸入中的拼字。 Hayabusa 輸入保留自己的 `Computer` 場地。引導磁區和安全描述符元資料保留在不定時報告中，因為它不提供事件時間戳記。

## 較新的多表解析器 { #newer-multi-table-parsers }

本機 JSON/JSONL 攜帶每個表名稱。對於獨立的 CSV，瀏覽器、WebCache 和 Outlook 適配器從列架構中識別其表，因此重命名 CSV 或使用自動解析的來源後綴不會更改時間戳解釋。 UAL 保留其明確已知時間戳欄位；存取計數器和不相關的日期文字不會被猜測為日期。

只有當 Outlook 屬性行的聲明類型為時間或多次類型時，才會投影它們。包含日期的字串仍然是字串。嵌入的訊息保留其項目 ID 和父引用，且附件提取時間不會取代儲存的附件時間戳記。不定時的文件、收件者、聯絡人或屬性行在本機輸出和不定時的報告中保持可追蹤。

相同的 CSV 和 JSONL 輸出模式適用於這些解析器；新增工件不會新增數百個 CSV 欄位。特定於工件的上下文保留在 `Details` 中。
