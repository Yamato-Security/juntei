# 超級時間軸 { #super-timeline }

`super-timeline` 會將剖析後的跡證轉換為採用共通結構描述的事件。輸出**預設不排序**：它依照所找到的輸入檔案順序，以及各剖析器將時間戳記投影為事件的順序輸出，與各工作執行緒完成的先後無關。加上 `--sort` 即可依時間排序並合併重複項。一筆 Prefetch 記錄可能包含多個執行時間；一筆 MFT 記錄可能包含八個以上的檔案時間戳記。每個時間戳記都會成為一筆個別加上標籤的事件。如此一來，就能更容易比較來自不同來源的證據，同時不會失去每個時間戳記實際代表的意義。

請從 [`auto-parse`](auto-parse.md) 的結果著手，而不是原始的證據目錄：

```bash
juntei auto-parse -d evidence/ -o parsed/
juntei super-timeline -d parsed/ -o SuperTimeline.csv
```

請保留剖析後的檔案、清單檔與原始證據。時間軸是供分析用的投影；它不會取代那些記錄，也不會產生 Plaso 原生格式的儲存檔。

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

`--hayabusa FILE` 可以重複指定，也可以單獨使用。它會匯入既有的事件／警示時間戳記與規則脈絡，而不需要執行 Hayabusa。也支援 `Details` 為字串的資訊性事件。`--artifact mft,prefetch` 會將投影限制在所選的跡證識別碼。時間範圍的起訖點皆包含在內，且必須明確指定 UTC 時差或 `Z`。

### 選擇輸入來源 { #choosing-an-input }

| 輸入 | 識別方式 |
|---|---|
| `-d DIR` | `manifest.json` 提供成功完成的主要工作、跡證 ID、電腦與確切的輸出檔案。次要的 `auto-parse -j` 工作會被排除，以免同一份資料的兩份副本都被匯入。 |
| `-f FILE` | 原生 JSON／JSONL 使用內嵌的 `Artifact` 與 `Comp` 欄位。未帶跡證標記的匯出需要 `--input-artifact`；獨立的 CSV 則一律需要此選項。 |
| `--computer NAME` | 輸入記錄沒有電腦名稱時所採用的備用值；未指定此選項時，未知的電腦會明確標示為 `Unknown`。 |
| `--hayabusa FILE` | 含有 Hayabusa 時間戳記、規則中繼資料、電腦，以及（若有的話）`EvtxFile` 的 JSONL。 |

目前的清單檔支援搬移剖析結果目錄：所記錄的絕對輸出路徑會改在新目錄之下解析。輸入（包括符號連結的目標）都必須位於該目錄之內。也支援舊版依跡證分開的 Amcache、SRUM 與 Jump Lists 目錄；類別目錄則需要明確的 `OutputFiles`。

支援使用相對於輸出的路徑，以及含有扁平化多資料表匯出的目前清單檔。此命令能辨識自動解析所排定的每一種跡證，包括較新的瀏覽器、WebCache、Outlook 與 UAL 資料表。

當清單檔指出來源為 VHDX 或 E01 映像檔時，來源參照會包含映像檔及其內部路徑，且映像檔名稱會保留在事件脈絡中。讀取剖析結果時，不需要當時的暫存工作區。

原始來源取自記錄或明確的清單指令。對於 `-d`，原始來源引用是相對於證據目錄的，而 `Details.ParsedFile` 是相對於解析目錄的。只有當主機有一個唯一可識別的收集的 SRUDB.dat 時，SRUM CSV 表才會分配給資料庫。否則 `SourceFile` 指向解析的文件，`Details.RawSourceAvailable` 是 `false`。

獨立輸入以其檔名作為參照。檔名相同的輸入會使用各自不同的 `./Inputs/<ID>/<filename>` 參照。位於已知證據根目錄之外的絕對原始來源路徑，則使用 `./Sources/<ID>/<filename>` 識別碼。這些不透明的參照能保留來源之間的區別，又不會匯出分析人員的絕對目錄名稱；若要查看原本的來源，請透過 `ParsedFile` 與 `InputRecord` 查閱原生記錄。證據中所記錄的路徑（例如 Windows 執行檔路徑）則維持不變。

## 共通事件欄位 { #common-event-fields }

| 欄位 | 意義 |
|---|---|
| `Timestamp` | UTC 的 RFC 3339 格式，一律寫出九位小數，以維持一致的排序。補零並不會提高精度。 |
| `Comp` | 與輸入記錄相關聯的電腦。 |
| `Artifact` | 穩定的跡證識別碼，例如 `mft`、`prefetch`、`srum` 或 `hayabusa`。 |
| `TimestampType` | 說明該時間戳記代表的意義，例如 `MFT SI Data modified`。 |
| `User` | 轉換器能取得時所記錄的帳戶或 SID；可能為空白。 |
| `Path` | 受影響的檔案、登錄機碼，或其他跡證特有的物件。 |
| `Message` | 供時間軸檢視器顯示的簡短說明。 |
| `SourceFile` | 相對於證據的原始來源參照；若其根目錄不明，則為相對的來源識別碼；兩者皆無時，則為前述以剖析後檔案替代的參照。 |
| `SourceField` | 該時間戳記所取自的確切剖析欄位或巢狀位置。 |
| `RecordID` | 跡證特有的記錄識別值。不一定是 EVTX 的 Record ID。 |
| `Details` | 結構化的脈絡，包括原始時間戳記、精度、時間基準、時間角色、剖析後的檔案、輸入記錄／位置，以及可取得的復原／規則中繼資料。 |

在 CSV 中，`Details` 是一個加上引號的 JSON 儲存格；在 JSONL 中則是物件。Timesketch 設定檔會將 `Timestamp`、`TimestampType` 與 `Message` 分別改為 `datetime`、`timestamp_desc` 與 `message`，同時保留其他出處欄位。這三個欄位就是 [Timesketch 匯入時必要的三個欄位](https://timesketch.org/guides/user/import-from-json-csv/)。

`Details` 刻意只包含經過挑選的脈絡，而不是完整的二進位資料或原生記錄的每個欄位。過長的字串、陣列與物件會受到大小限制，並標記為 `ProjectionOmitted`。若要檢視完整的原生記錄，請查閱 `Details.ParsedFile`、`InputRecord` 與 `Location`。對 CSV 而言，記錄編號計算的是標題列之後的 CSV 記錄，而不是加上引號的儲存格內的實體行。對 JSONL 而言，它是行號；對 JSON 而言，它是頂層文件的編號再加上巢狀位置。

## 各時間代表的意義 { #what-the-times-mean }

時間戳記是關於某個被記錄物件的證據。單憑它本身，並不能證明有執行或攻擊發生。各轉換器刻意保留以下區別：

| 跡證 | 在時間軸上的判讀 |
|---|---|
| MFT | SI 與 FN 的修改、存取、中繼資料變更與建立時間各自分開；各個名稱的 FN 屬性也維持各自獨立。使用 `--sort` 時，檔名資料列上重複的 SI 欄位會被合併。兩種模式都會排除 ADS 資料列中與基底檔案共用的時間。從 FILE 記錄 slack 復原的名稱，會保留各自獨立的已復原 FN 事件。 |
| I30 | 所儲存的 FN／索引中繼資料，並在可取得時附上 slack／復原脈絡。這些並不是目錄索引寫入當下的時間戳記。 |
| USN Journal | 日誌記錄的時間戳記與更新原因。一筆記錄可能描述多項變更；它不一定記錄了這些變更確切的開始時間。 |
| LogFile | 所儲存的 redo／undo 時間戳記快照與檔名中繼資料。redo 與 undo 維持分開。LSN 與檔案位移是識別碼，絕不會被轉換為日期。 |
| Prefetch | 所記錄的執行時間。使用 `--sort` 時，這些時間會與相對應的 Prefetch 時間軸匯出合併。所參照的磁碟區建立中繼資料會另外加上標籤。 |
| 瀏覽器 SQLite 資料庫 | 所記錄的造訪、復原的網頁最後造訪中繼資料、下載開始／結束、Cookie 建立／存取／到期，以及表單歷程的首次／最後使用或刪除後留下的墓碑記錄。下載結束時間本身並不能證明下載成功或檔案已執行；Cookie 到期時間只是設定的期限。復原狀態、可信度與來源位置都會保留在脈絡中。 |
| Internet Explorer／舊版 Edge 的 WebCache | 所儲存的項目存取、建立／修改、到期與同步時間、下載開始、Cookie 中繼資料，以及容器的存取／清理。快取的內部管理記錄並不會自動成為使用者開啟網頁的證據。 |
| Outlook PST、OST、MSG 與 OFT | 郵件的送出／傳遞；資料夾、項目與附件的建立／修改；行事曆的開始／結束／提醒；工作的開始／到期／完成／提醒；以及明確宣告為時間型別的屬性。排定的時間並不能證明會議確實舉行、提醒確實觸發或程式確實執行。所儲存的傳遞時間並不能證明郵件已被閱讀。 |
| 使用者存取日誌記錄 (UAL) | 第一次/最後一次存取元資料、資料庫插入/建立、第一次/最後一次看到和儲存的作業系統啟動時間。角色、帳戶、來源位址和每日計數器仍然是上下文。每日日期/計數不是午夜登入發明的；未計時的標識和 DNS 行保留在原始解析結果中。 |
| Amcache | Inventory 機碼的最後寫入時間、上層磁碟區機碼的脈絡、所記錄的安裝日期，以及 PE 連結中繼資料。僅憑出現在 Inventory 中或 LinkDate，並不能證明曾經執行。 |
| ShimCache | 所儲存的檔案修改時間、快取更新中繼資料，以及快取機碼的最後寫入時間。這些都不會自動被視為執行時間戳記。 |
| SRUM | 提供者時間戳記／彙總區間、連線開始、明確的開始／結束區間欄位，以及應用程式識別碼中繼資料。位元組數、持續時間與其他計數器都不是日期。 |
| LNK | 目標的 MAC 時間戳記、追蹤識別碼的建立時間，以及殼層項目時間。這些描述的是所儲存的目標中繼資料，不一定代表捷徑被開啟的時間。 |
| Jump Lists | 項目的最後使用中繼資料、追蹤識別碼的建立時間，以及內嵌 LNK 的目標時間。預設會省略收集端容器檔案本身的時間。 |
| 資源回收筒 | 每個項目被移至回收筒的時間，並附上回收筒的使用者、原始路徑，以及已刪除的內容是否已被收集。這不是清空回收筒的時間，也不是項目本身的檔案時間。 |
| Defender 隔離資料 | 項目時間戳記，以及所儲存的資源建立／存取／寫入時間。擷取時間並不是端點活動的證據。 |
| BAM、UserAssist、FeatureUsage | 存在時所記錄的活動欄位，並另外保留機碼最後寫入時間的脈絡。沒有時間的計數器不會產生事件。 |
| RecentDocs、MRU、開啟／另存新檔 MRU、ShellBags、Office | 所記錄的使用／信任／殼層項目時間及機碼脈絡。清單的機碼時間並不是清單中每個項目各自的最後使用時間。 |
| SAM 與 LSA 機密資訊 | 所記錄的帳戶登入／密碼／到期或機密資訊設定中繼資料，以及相關的機碼時間。這些並不能取代驗證事件記錄。 |
| 服務、TaskCache、登錄自動啟動項目、BCD | 機碼／設定中繼資料，以及明確記錄的排程工作時間。單憑服務機碼的寫入，並不能證明服務曾經執行。 |
| 網路與 USB | 網路設定檔／租約，以及裝置安裝／連接／移除欄位，並附上登錄機碼脈絡。原生 USB 輸出中，使用者 MountPoints2 的時間會保留其對應的使用者登錄區來源。 |
| 系統資訊、已安裝程式、AppCompat、應用程式歷程 | 明確的安裝、關機、清冊／活動以及機碼中繼資料欄位。原生系統資訊的設定檔／版本陣列會保留其巢狀時間。 |
| 稽核原則、安全性設定、MuiCache、一般登錄區 | 有提供時採用機碼最後寫入時間；無時間戳記的值／計數器仍保留在原生輸入中。 |
| Persistence Hunter | 作為佐證的機碼最後寫入時間，以及相關聯的變更時間。發現項目本身既不是帶有時間戳記的安裝事件，也不是已遭入侵的判定。 |
| Hayabusa | 事件／警示時間戳記，並附上規則、等級、通道、事件 ID 與可取得的詳細脈絡。多條規則以及不同的關聯規則資料列都會分開保留。 |

預設匯出會保留所有時間戳記投影，包括重複的中繼資料。它不會進行全域排序，也不會合併重複項。使用 `--sort` 時，重複的投影會被合併；因此資料列數可能少於以相同輸入產生的未排序匯出。時間相同、但意義或證據來源不同的多個時間戳記，在兩種模式中都會保留。判定是否重複的識別依據包括電腦、來源、跡證、記錄、時間意義與復原狀態。合併後的資料列會記錄 `DuplicateCount`，以及數量有上限的替代路徑／來源欄位。不同的來源絕不會只因路徑與時間戳記看起來相似就被合併。無法解析原始來源時，會保留輸入記錄的識別值，以免將不同次收集的資料合而為一。

### 精度、當地時間與假設 { #precision-local-times-and-assumptions }

已知為 UTC 的欄位，以及帶有明確時差的時間戳記，都會正規化為 UTC。原始值及其所記錄的精度都會保留，包括 100 奈秒精度的 FILETIME。來源剖析器或下游的檢視器可能已將時間四捨五入；此命令無法還原已遺失的精度。

有些殼層項目／FAT 欄位即使被上游剖析器加上 `Z` 序列化，實際上仍具有當地時間的語意。時間軸會保留剖析出的值與欄位來源；它無法找回缺少的端點時區，也無法驗證上游所做的轉換。在將這類時間用於精確的跨主機關聯之前，請先查證原始跡證與端點的設定。

沒有工件定義時區的原始時間戳記被視為未解析，而不是默默地視為 UTC。如果證據證明這一點是合理的，請提供一個帶符號的固定偏移量：

```bash
juntei super-timeline -f Amcache.jsonl --input-artifact amcache \
  --assume-offset "+09:00" -o Timeline.csv
```

僅有日期的值也需要此選項。其午夜基準點會被標示為具有**日精度**的假設；它並不是觀察到的午夜事件。不接受時區名稱與日光節約時間規則。缺漏、為零、無效與超出範圍的值，不會產生虛構的 epoch 事件。明確記錄的有效 1970 年日期則會保留。

預設會排除兩種選用類別：

- `--include-collection-times`：收集／分析環境檔案系統上的 LNK／Jump List 來源檔案中繼資料。這些事件會標示為 `collection_filesystem`，與端點活動區隔開來。
- `--include-interpreted-times`：登錄讀取器將一般登錄 QWORD 值呈現為 FILETIME 的結果。這些事件會標示為 `interpreted_value`，因為許多這類值實際上是計數器。

## 輸出和跳過的記錄 { #output-and-skipped-records }

此指令僅以 `-t` 選擇的格式寫入 **僅使用 `-o` 選擇的時間軸**（預設為 `csv`，或 `jsonl`）。它不會創建不定時的診斷或摘要伴隨文件或每筆記錄的診斷假脫機。早期運行的現有伴隨文件保持不變。

終端摘要報告時間軸事件和未計時記錄的數量。非零無效時間戳記和未解析時區計數單獨顯示。對於 `--sort`，它還報告合併的重複項的數量。保留不產生時間軸事件的記錄和完整工件上下文的原始解析結果。

等待合理時區假設的有效日期算是未解析的時區，而不是無效時間戳記。沒有發明時間戳，也沒有可用映射時間的記錄不會作為合成紀元事件插入時間軸。

有效時間超出請求範圍的記錄將被視為已過濾，而不是未計時。成功匯出意味著所選解析輸入已成功處理；這並不意味著每個工件都被收集，每個欄位都有時間戳或每個攻擊都是可偵測的。

CSV 匯出可能會將巢狀中繼資料扁平化：系統資訊設定檔、USB 使用者掛載點、殼層項目及其他結構化脈絡，在原生 JSON／JSONL 中可能更為完整。這些欄位很重要時，請使用原生輸入。時間軸無法重建 CSV 匯出時遺漏的資料。不支援的跡證 ID 與格式錯誤的輸入檔案會產生錯誤，而不會以含有部分結果的成功狀態結束。

## 效能與輸出安全性 { #performance-and-output-safety }

輸入會以串流方式平行處理。預設情況下，每個工作執行緒會寫出一段已格式化的事件片段；這些片段會依輸入檔案順序串接，不需要排序，也不需要重新讀取其事件結構。即使處理多個檔案，CSV 也只會有一個標題列。不論工作執行緒數量為何，輸出順序都固定不變；但這並不代表依時間先後排列。

`Input files` 顯示所選解析輸入和 Hayabusa 時間軸的組合大小。 `Results saved` 顯示已儲存的時間軸檔案的大小。兩者都使用與 `auto-parse` 相同的人類可讀 `KiB`/`MiB`/`GiB` 單位。

使用 `--sort` 時，緩衝區內容會溢寫到專用的暫存目錄，再經由有界的合併輪次產生依時間排序的輸出。排序使用完整的整數秒與奈秒，包括 1970 年之前的日期。時間相同的事件，其排列順序不論工作執行緒數量或排序預算為何都固定不變。在中間合併與最終合併期間（包括重複的合併輪次），每處理一個事件，進度列就會前進一格。進度列會在終端機上顯示，輸出被重新導向時則會隱藏。

`--memory-limit` 僅在使用 `--sort` 時適用：它是合計的**排序緩衝區預算**，而不是處理程序的 RSS 上限。輸入記錄、JSON 物件／字串的額外負擔、CSV 緩衝區與合併記錄都需要額外的記憶體。預算較小時，工作執行緒數量會減少。`--max-record-size` 限制單筆剖析輸入記錄的大小（預設 16 MiB）；多行縮排格式的 JSON 陣列／報告會算作一份頂層輸入文件。較大的文件可能需要提高上限，或者最好改用 JSONL。CSV 剖析器的預先讀取也會計入此上限。

除了最終結果之外，暫存磁碟空間還必須容納已格式化的事件片段（預設），或是展開後的排序記錄與合併輪次（`--sort`）。兩種模式都保留輸出先暫存再發布的機制，並會防止發布處理失敗之輸入的結果。一筆記錄可能產生許多事件，因此時間軸可能遠大於輸入。適當時請篩選時間範圍／跡證，並選擇有足夠可用空間的暫存磁碟區。

Ctrl+C 協同取消，刪除私人排序/暫存目錄並退出，代碼為 `130`。在 Linux 和 macOS 上，這也適用於 SIGTERM 和 SIGHUP。在發布開始之前，在讀取、規範化、合併和寫入時間軸時會檢查取消情況。無法捕獲的終止或斷電仍然可能留下臨時檔案。

現有輸出需要 `-C`。時間軸不得涵蓋輸入、清單、引用的本地證據或其受保護的證據目錄。輸入錯誤或排序失敗不會影響現有時間軸。完成的時間線會在其目的地旁邊上演並自動替換它。當保留較早的完整匯出很重要時，請為每次運行使用新的輸出檔名。

## 更多細節 { #further-details }

- [Timesketch CSV／JSONL 匯入](https://timesketch.org/guides/user/import-from-json-csv/)
- [Plaso 時間戳記展開與舊版 CSV](https://plaso.readthedocs.io/en/latest/sources/user/Output-format-l2tcsv.html)
- [Plaso 輸出與格式化](https://plaso.readthedocs.io/en/latest/sources/user/Output-and-formatting.html)
- [證據與判讀](../overview/evidence-model.md)
- [輸出格式](../getting-started/output-formats.md)

目前的 Juntei 輸入使用縮寫標籤，例如 `Comp`, `Filename`, `Args` 和 `Attr`. MFT CSV 也接受目前的 `File`, `SN`, `ParentEntryNum`, `ParentSN`, `Type`, `USN`, `LogfileSN` 和 `HasADS` 標頭，保留序號產生和 DOS/ADS 差別完好無損。舊的出口標籤仍然被接受，並且 `SourceField` 保留實際輸入中的拼字。 Hayabusa 輸入保留自己的 `Computer` 場地。沒有事件時間戳記的引導磁區和安全描述符元資料保留在原始解析結果中；它不會建立時間軸事件。

## 較新的多資料表剖析器 { #newer-multi-table-parsers }

原生 JSON／JSONL 帶有各資料表的名稱。對於獨立的 CSV，瀏覽器、WebCache 與 Outlook 轉換器會依欄位結構描述辨識其資料表，因此即使重新命名 CSV，或使用自動解析的來源後綴，也不會改變時間戳記的判讀。UAL 維持只採用其明確已知的時間戳記欄位；存取計數器與看似日期但無關的文字，不會被猜測為日期。

Outlook 屬性資料列只有在其宣告的型別為時間或多值時間型別時，才會被投影。包含日期的字串仍然是字串。內嵌郵件會保留其項目 ID 與父項參照，且附件的擷取時間不會取代所儲存的附件時間戳記。無時間戳記的檔案、收件者、連絡人或屬性資料列，仍可在原生輸出與無時間戳記報告中追溯。

相同的 CSV 與 JSONL 輸出結構描述也適用於這些剖析器；新增跡證並不會增加數百個 CSV 欄位。跡證特有的脈絡會保留在 `Details` 中。
