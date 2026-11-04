# NTFS 交易記錄 (`$LogFile`) { #ntfs-transaction-log-logfile }

## 這是什麼，以及為何重要 { #what-it-is-and-why-it-matters }

NTFS 是 Windows 系統磁碟區通常使用的檔案系統。為了能從當機中復原，它會將
打算對檔案系統中繼資料進行的變更，記錄在名為 `$LogFile` 的隱藏磁碟區檔案中。
每筆更新可以包含 **redo** 位元組（新狀態）與 **undo** 位元組（回復變更所需的先前
狀態）。這與 Windows 事件記錄、登錄的 `.LOG1`
檔案以及 [USN 日誌](usn-journal.md) 是分開的。

對調查人員而言，先前的狀態特別有用。undo 記錄可能保留重新命名前的檔名、
已刪除的目錄項目，或中繼資料更新之前的時間戳記。
有些記錄也包含 NTFS 寫入 USN 日誌的位元組，包括已不在所收集之
`$J` 中的記錄。不過，許多交易涉及的是作業系統的例行活動，並不
描述攻擊行為。

**範例：**某個被移除的索引項目提到 `report.txt`；之後新增的項目則提到
`report.txt.encrypted`。比對完整的檔案參照、上層目錄、交易鏈與其他
跡證，可以佐證一次重新命名。單憑這兩個名稱並不能識別勒索軟體、其操作者，
或加密的確切時間。

## 位置與收集 { #location-and-collection }

```text
C:\$LogFile
D:\$LogFile
```

每個 NTFS 磁碟區都有自己的記錄。請使用能讀取 NTFS 中繼資料的工具取得它，或從
鑑識映像檔或快照中匯出；一般的 Windows 檔案複製可能無法存取它。請在時間上盡可能接近地收集
同一磁碟區的 `$MFT` 與 `$Extend\$UsnJrnl:$J`。保留原始
檔案與雜湊值。之後再次擷取的資料可能包含不同時段的活動。

`$LogFile` 是**循環記錄**：新記錄會覆寫舊記錄。保留時間取決於記錄大小
與工作負載，可能只有數分鐘或數小時。找不到某個檔名或操作，並不能
證明它從未發生。

## 記錄能證明什麼，以及注意事項 { #what-a-record-proves-and-cautions }

- 更新記錄的是 NTFS 為了復原而記錄的中繼資料。單一 redo 項目並不能證明
  整個較高層級的動作已完成，也不能證明是由特定人員發起。請將相關記錄、
  交易完成狀態、最終的檔案系統以及獨立證據交叉比對。
- **沒有通用的交易時間戳記。**內嵌的時間戳記描述的是檔案中繼資料，
  有時已過時、是複製而來，或遭攻擊者修改。請勿將 `CreatedOn` 放在事件時間軸上
  作為重新命名或時間戳記變更的時間。USN 記錄可能提供另一個時間戳記。
- LSN（記錄序號）用於排序**單一磁碟區記錄內**的記錄。它們不是時鐘時間，
  也不得跨主機或跨磁碟區比較。來自較早一輪循環緩衝區的記錄
  可能與較新的活動並存。
- 交易表的槽位會被重複使用。請以 `TransactionFirstLsn`、`TransactionId`、用戶端
  身分與來源進行分組，而不是只用 `TransactionId`。交易鏈可能指向已保留資料之外。
- 檔案參照包含項目編號**與序號**。否則 MFT 項目被重複使用時，
  可能讓不相關的檔案看起來像是同一個檔案。
- 由 `$MFT` 提供的路徑描述的是**所收集之 MFT 的狀態**，而非重建出的歷史
  路徑。Juntei 會比對序號，並明確標示未知的路徑部分。上層目錄可能在交易之後
  被重新命名，而硬式連結可能提供不只一個目前名稱。
- `TargetInferred` 表示某個開啟屬性的對應來自記錄的另一個時段。
  請更加謹慎地看待該對應。未解析的索引項目可能僅依其
  配置進行解碼；請以原始位元組與其他來源驗證其名稱。
- 撕裂頁面（torn page）會被略過；周圍可讀取的記錄仍然可用。`Truncated` 標示
  缺少後續內容的記錄。在將輸出視為完整之前，請先檢視剖析器的診斷訊息。
  長度為零的輸入是錯誤；從未寫入、全為 `FF` 的記錄則理所當然沒有任何資料列。
- 剖析器**不會重播**交易來重建歷史 MFT。它不會
  復原任意的非常駐檔案內容，也不會執行任何收集到的資料。不支援受 DAX CRC 保護的
  頁面。以真實證據進行的驗證主要涵蓋 4 KiB 的記錄頁面。

## 使用 Juntei 剖析 { #parse-with-juntei }

```bash
# Single log, current paths from an adjacent $MFT when present
juntei parse-ntfs logfile -f '$LogFile' -o logfile.csv

# Explicit MFT from the SAME volume
juntei parse-ntfs logfile -f '$LogFile' -m '$MFT' -t jsonl -o logfile.jsonl

# Multiple collected logs; SourceFile distinguishes every input
juntei parse-ntfs logfile -d evidence -o collected-logs.csv

# Preserve the underlying payloads for low-level validation
juntei parse-ntfs logfile -f '$LogFile' --raw-data -t jsonl -o logfile-raw.jsonl

# Automatic per-computer output
juntei auto-parse -d evidence -o parsed
```

所有選項請參閱[命令參考](../commands/filesystem.md#parse-ntfs-logfile)。目錄
探索會接受 `$LogFile`、`LogFile`、其 `.bin` 形式，以及 `C_$LogFile` 或
`C_$LogFile.bin` 之類的名稱，且不區分大小寫。以 `-f` 傳入的單一檔案可以使用任何名稱。

自動 MFT 配對會遵循匯出資料的磁碟區前綴：`C_$LogFile` 使用 `C_$MFT`，而
`D_$LogFile.bin` 會先嘗試 `D_$MFT.bin`，再嘗試 `D_$MFT`。帶有前綴的記錄絕不會退而使用無前綴的 `$MFT`。
無前綴的 `LogFile` 也接受 `MFT` 或 `$MFT`。若您的匯出資料採用其他命名方式，
請搭配 `-f` 使用 `-m` 明確指定相符的 MFT。

輸出為每個 LSN 一筆記錄，包括更新、檢查點與未知的用戶端記錄。JSON 是
經過美化排版的物件串流；JSONL 則是每行一個物件。CSV 具有相同的欄位名稱；
陣列與結構化欄位會以精簡的 JSON 形式放在 CSV 儲存格中。未知的純量值在
CSV 中為空，在 JSON 中為 `null`。空陣列表示沒有解碼出任何項目，並不能證明不存在任何項目。

每次只會在記憶體中保存一整份記錄及其剖析後的記錄。解析 MFT 路徑會增加
記憶體用量與處理量。`--raw-data` 可能大幅增加輸出大小。在 `auto-parse` 中，於記憶體有限的
機器上請使用 `-T` 限制同時執行的剖析器處理程序數量。

若無法讀取相鄰的 MFT，Juntei 會發出警告，並在不含路徑的情況下匯出記錄；此時 `MftSource`
會維持 null。此豐富化失敗不會捨棄可讀取的主要證據。
找不到相符的 MFT 或比對結果不明確時，也會產生警告，並在摘要中計入 `Logs without a matching MFT`；
這些不計為剖析失敗。

## 輸出欄位 { #output-fields }

### 記錄的識別、完整性與出處 { #record-identity-integrity-and-provenance }

| 欄位 | 意義與判讀 |
|---|---|
| `SourceFile`、`FileOffset` | 輸入路徑，以及讀取此記錄標頭的位元組偏移；可用來驗證位元組。 |
| `LSN` | 記錄的記錄序號。較大的值是整數，並非時間戳記。 |
| `PreviousLSN`、`UndoNextLSN` | 指向前一筆記錄與回復記錄的連結。零表示沒有連結；非零連結可能指向已保留資料之外。 |
| `TransactionID`、`TransactionFirstLSN` | 交易表槽位，以及與此交易連結的最早 LSN；單憑槽位並不唯一。 |
| `RecordType` | `ClientRecord`（更新）、`ClientRestart`（檢查點）或 `Other`。 |
| `ClientIndex`、`ClientSequence` | 記錄用戶端的身分；通常為 NTFS。這些不是使用者帳戶或處理程序 ID。 |
| `Flags`、`SpansPages` | 原始記錄旗標，以及用戶端資料是否延續到另一個頁面。跨頁是正常的。 |
| `Truncated` | 若剖析器發現此 LSN 缺少後續內容則為 true。它並不彙總整個檔案中的所有損毀。 |
| `ClusterSize`、`ClusterSizeSource` | 用於計算目標偏移的每叢集位元組數；來源為 `Caller`、`Checkpoint` 或 `Inferred`。若已知實際值，請以 `--cluster-size` 提供。 |
| `LogVersion` | 取自最新可讀取之重新啟動頁面的版本，通常為 `1.1` 或 `2.0`；若沒有留存可讀取的重新啟動頁面則為 null。 |
| `MFT-Source` | 用於豐富目前路徑的 MFT（若有）。它必須屬於同一個磁碟區。 |

在 `auto-parse` 的 JSON／JSONL 中，會如同其他剖析器一樣加上 `Comp` 與 `Artifact`。該工作的
`.txt` 記錄檔保存診斷訊息；清單檔則說明該工作是成功、失敗或被略過。
可挽救的損毀可能產生一個成功的工作，但其記錄檔中的 `Parse problems` 計數不為零。

### 操作與目標 { #operations-and-target }

| 欄位 | 意義與判讀 |
|---|---|
| `RedoOperation`、`UndoOperation` | NTFS 操作名稱，未知代碼則以十六進位表示。redo 描述預定的更新；undo 描述其反向操作。 |
| `TargetEntryNumber`、`TargetSequenceNumber` | 對檔案記錄的操作：受影響的 MFT 記錄編號，不推斷序號。其他情況：擁有已解析之開啟屬性的檔案參照。內嵌的目錄索引更新以目錄為目標，而非其項目所描述的子項目。 |
| `TargetAttributeType`、`TargetAttributeName` | 操作所使用之開啟屬性的數值類型與名稱。對檔案記錄的操作而言，通常是 `$MFT` 的 `$DATA`。常見類型：16（`$STANDARD_INFORMATION`）、48（`$FILE_NAME`）、128（`$DATA`）、144（`$INDEX_ROOT`）、160（`$INDEX_ALLOCATION`）、176（`$BITMAP`）。 |
| `TargetPath` | 當目標的完整參照已知時為目前的 MFT 路徑，否則為未知的路徑部分。對於歷史世代未知的檔案記錄操作則為空；不得僅憑記錄編號解析為其目前的佔用者。 |
| `OpenAttributeEntryNumber`、`OpenAttributeSequenceNumber`、`OpenAttributePath` | 擁有該操作所寫入之開啟屬性的檔案，及其目前的 MFT 路徑。對檔案記錄的操作而言，這些欄位會將 `$MFT` 脈絡與受影響的記錄分開保存。 |
| `TargetAttribute` | 在記錄的開啟屬性表中的偏移；這與屬性的類型或 MFT 識別碼不同。 |
| `TargetInferred` | 若目標對應來自記錄的另一個時段則為 true。即使此旗標為 false，null 目標仍維持未解析。 |
| `TargetVCN`、`Lcns` | 目標中的虛擬叢集與實體叢集編號；適用於低階驗證，而非時序判斷。 |
| `RecordOffset`、`AttributeOffset` | 屬性／項目的偏移，以及其中更新的偏移。 |
| `ClusterBlockOffset`、`TargetBlockSize` | 以 512 位元組為單位的偏移與目標大小。 |
| `OffsetInTarget` | 在受影響的檔案記錄或非常駐資料流中計算出的位元組偏移。取決於叢集大小與 MFT 記錄大小。 |
| `RedoLength`、`UndoLength` | 宣告的資料長度。清零操作可以宣告某個長度，卻不攜帶那麼多的酬載位元組。 |
| `RedoBytesAvailable`、`UndoBytesAvailable` | 實際可供解碼的酬載位元組。請與宣告的長度及操作語意比較。 |

### 便利的檔案中繼資料欄位 { #convenient-file-metadata-columns }

這些欄位顯示一個取自 redo 端、優先選用的非 DOS 檔名，若無則退而使用 undo 端。
若兩端都沒有檔名，時間戳記更新欄位會優先採用 redo 的時間戳記，其次為 undo。
目錄項目更新也會直接呈現其時間、大小與旗標，並優先採用 redo 而非
undo。已解碼的 `$DATA` 屬性即使沒有檔名也會呈現其大小。已解碼之
檔案記錄的未命名 `$DATA` 大小，優先於其可能已過時的檔名大小。
這些欄位是瀏覽輔助；若要完整比較前後狀態，請使用各端獨立的欄位。

| 欄位 | 意義與判讀 |
|---|---|
| `MetadataSide` | `Redo` 或 `Undo`，識別提供便利檔名／時間戳記欄位的一端。 |
| `EntryNumber`、`SequenceNumber` | 由已解碼的檔名／檔案記錄／屬性中繼資料，或部分 `$STANDARD_INFORMATION` 時間戳記更新所描述的檔案，若已知則附上序號。對於未識別子項目的目錄項目更新，以及原始大小／執行清單更新，則為空。受影響的記錄請使用 `TargetEntryNumber`。 |
| `Filename` | 優先選用的已解碼名稱；所有名稱（包括 DOS 別名與硬式連結）都保留在各端的陣列中。 |
| `ParentEntryNumber`、`ParentSequenceNumber`、`ParentPath` | 上層參照與目前的 MFT 路徑；並非歷史路徑的重建。 |
| `CreatedOn`、`ContentModifiedOn`、`RecordModifiedOn`、`LastAccessedOn` | UTC 中繼資料時間，精確至 100 ns。這些不是交易時間戳記。遺失／無效的時間維持為空／null。 |
| `FileSize`、`FileAttributes` | 取自所選中繼資料的大小與 Windows 數值旗標；目錄項目中的值可能已過時。`$DATA` 屬性與已解碼的檔案記錄可提供更直接的大小。 |
| `AttrType` | 已解碼之屬性記錄的數值類型；已解碼的目錄項目中繼資料更新則為 48。與操作所針對的開啟屬性是分開的。 |

### 完整的 redo／undo 中繼資料與酬載 { #complete-redoundo-metadata-and-payloads }

| 欄位 | 意義與判讀 |
|---|---|
| `RedoFileNames`、`UndoFileNames` | 該端每個已解碼 `$FILE_NAME` 的陣列，分別保留新增／新的名稱與移除／舊的名稱。 |
| `RedoTimes`、`UndoTimes` | 各端寫入的 `$STANDARD_INFORMATION` 時間戳記；部分更新會讓未變更的時間戳記保持缺席，而不加以猜測。 |
| `RedoDecoded`、`UndoDecoded` | 結構化的檔案記錄、屬性、索引項目或目錄項目資訊，以 `Kind` 識別。null 可能代表不支援的操作或位元組不足。 |
| `JournalWriteOffset`、`JournalWriteLength` | 寫入 `$DATA:$J` 之 redo 資料的偏移與位元組數。這可識別原本已遺失之 USN 記錄的可能來源；它不會匯出重建的日誌。 |
| `Checkpoint` | 檢查點版本、起始 LSN、叢集大小，以及開啟屬性、屬性名稱、已變更頁面與交易表傾印的 LSN／長度。 |
| `OtherRecordType` | 無法辨識或無法解碼之內容的原始用戶端記錄類型。 |
| `RedoData`、`UndoData`、`OtherData` | 僅在使用 `--raw-data` 時提供的十六進位酬載；否則為 null／空。請保留這些資料以驗證不支援或損毀的結構。 |

每個檔名物件都包含檔案與上層參照（`Raw`、`EntryNumber`、
`SequenceNumber`）、`Name`、`NameEscaped`、`Namespace`、目前的 `ParentPath`、四個時間戳記、
`Size`、`AllocatedSize`、`Attr` 與 `ReparseValue`。`NameEscaped` 用於識別以逸出序列
表示的無效 UTF-16。重新剖析值可能是重新剖析標籤或延伸屬性大小。

時間戳記物件會同時保留 `Utc` 與整數 `Filetime`；哨兵值或無效值的
UTC 表示可能為 null，但仍保留其原始整數。`RedoTimes` 與 `UndoTimes`
包含 `Created`、`Modified`、`Changed` 與 `Accessed`；缺少的成員其值為 null。

已解碼的 `FileRecord` 物件包含編號、序號、硬式連結數、旗標、基底參照、
是否僅留存標頭，以及存在的屬性。每個已解碼的屬性都保留
類型、名稱、常駐狀態、旗標、ID、大小與配置大小。`$STANDARD_INFORMATION` 另外加上
四個時間、屬性、擁有者／安全性 ID、配額與 USN；這些 ID 需要另外解析，
且不能證明是哪位使用者變更了檔案。索引項目包含旗標、鍵長度與
選用的檔名。目錄項目更新即使沒有記錄名稱本身，也會保留四個時間、大小、屬性與重新剖析
值。

## 調查工作流程 { #investigation-workflow }

1. 檢視診斷訊息中缺少的重新啟動頁面、撕裂頁面與遭截斷的記錄。
2. 在 `RedoFileNames` 與 `UndoFileNames` 中搜尋可疑的檔名或副檔名。
3. 比對項目編號**與序號**，再檢查該交易的 LSN 鏈與操作。
4. 比較 redo 與 undo 的時間戳記以找出變更。例行的複製、解壓縮與還原
   也可能變更時間戳記；差異只是線索，並不自動等於時間戳記竄改（timestomping）的判定。
5. 與 [USN 日誌](usn-journal.md)、[$MFT](mft.md) 及 [$I30](i30.md) 交叉比對。使用事件
   記錄、[Prefetch](prefetch.md) 與使用者活動跡證來確立執行或歸因。

例如，找出任一端帶有加密檔名的所有記錄：

```bash
jq 'select(any(.RedoFileNames[]?, .UndoFileNames[]?;
  .Name | endswith(".encrypted"))) |
  {Lsn, TransactionId, TransactionFirstLsn, RedoOperation, UndoOperation,
   RedoFileNames, UndoFileNames}' logfile.jsonl
```

這是檔名搜尋，並非完整的勒索軟體偵測器。若要檢視日誌酬載，
請以 `--raw-data` 匯出，並選取 `JournalWriteOffset` 不為 null 的記錄；`RedoData`
的十六進位資料代表寫入該偏移的位元組。剖析這些內嵌的 USN 記錄並去除重複
是另一個步驟。Juntei 目前尚未將其與 `parse-ntfs usn-journal` 的輸出合併。

## 延伸資訊 { #further-details }

- [ys-ntfs-parser 的 logfile crate](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/logfile)：支援的結構、驗證語料庫與已知限制。
- [Sootmark 的 ntfslog](https://github.com/Sootmark/ntfslog)：上游的 Rust 實作。
- [libfsntfs NTFS 格式文件](https://github.com/libyal/libfsntfs/tree/main/documentation)：記錄頁面、修正（fixup）與中繼資料結構。
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs)：獨立的 NTFS 剖析研究與實作。
- [Joakim Schicht 的 LogFileParser](https://github.com/jschicht/LogFileParser)：詳盡的交易記錄分析。
- [Stroz Friedberg 的 ntfs-linker](https://github.com/strozfriedberg/ntfs-linker)：將 NTFS 來源交叉關聯為檔案歷程。
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum)：真實的 Windows NTFS 樣本。
- [Velocidex go-ntfs](https://github.com/Velocidex/go-ntfs)：NTFS 實作與公開的測試磁碟區。
