# 主檔案表 (`$MFT`) { #master-file-table-mft }

## 什麼是 MFT { #what-the-mft-is }

NTFS 以主檔案表（Master File Table）中的一筆記錄來表示每個檔案與目錄。記錄包含各種屬性，
例如名稱、父項參照、時間戳記、大小、旗標、安全性識別碼、重新剖析資訊，以及資料流中繼資料。
較小的檔案內容與替代資料流可以是**常駐**的，也就是其位元組直接儲存在 MFT 記錄本身之中。

MFT 之所以是基礎，是因為它涵蓋整個磁碟區中已配置與已刪除的記錄。它可以顯示檔案曾經存在、
它被連結在何處、它是否已被刪除，以及其中繼資料如何變更——即使一般的目錄瀏覽已無法看到它。

<span class="evidence-strength">最佳結論：</span>具有所顯示中繼資料的 NTFS 檔案記錄在收集時
存在，或殘存於未配置的記錄中。單憑 MFT 並不能證明執行或使用者意圖。

## 位置與收集 { #location-and-collection }

```text
C:\$MFT
```

`$MFT` 是受保護的 NTFS 中繼資料。請使用能以原始方式存取 NTFS 的鑑識擷取或分類（triage）
工具收集它；一般的使用者模式複製通常無法讀取它。可能的話，請將它與 `$J`、`$LogFile`
及磁碟區中繼資料一併收集。

## 分析注意事項 { #analyst-cautions }

- 已刪除的記錄可能已被部分覆寫，或其記錄編號可能已被重複使用。
- `SN` 用於區分相同項目編號的不同世代。請同時比對項目編號**與**序號。
- `$STANDARD_INFORMATION`（`0x10`）與 `$FILE_NAME`（`0x30`）的時間戳記具有不同的更新行為。
  兩者都無法免於竄改。
- `SI<FN`、`uSecZeros` 與 `Copied` 只是線索，並非自動的時間戳記竄改判定。
- 硬連結會讓一筆檔案記錄產生多個 `$FILE_NAME` 屬性與路徑。
- NTFS 可以儲存多個名稱（包括 DOS 8.3）、替代資料流與擴充記錄。
- 存取時間的更新可能被停用、延遲或合併。
- 完整路徑是從父項記錄重建而來。`[Orphaned]`、`[Unknown]` 或 `PathUnknown`
  代表祖系資訊缺漏／被重複使用／已損毀——而非字面上的目錄名稱。
- FILE 記錄的 **slack**（超出使用中大小的位元組）可能保留它先前持有的名稱：索引已移至
  索引配置區的小型目錄的索引項目（其中大多數在那裡仍在使用中）、較早的名稱，或被重複使用之記錄的
  前一個佔用者。復原的名稱沒有 `$STANDARD_INFORMATION`，且名稱的項目標頭可能已被覆寫。當記錄的
  使用中大小或屬性無法指出其 slack 從何處開始時，就不會從中復原任何資料，該列的 `ParseErrors`
  會註明 `slack not read` 及其原因。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $MFT files
  -f, --file <FILE>      Parse one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse selected entry numbers (example: 0-15,30)

Output:
  -o, --output <PATH>                   Save records
  -t, --output-type <TYPE>              csv, ys-ntfs-parser-csv, json, jsonl
  -e, --extract-resident-streams <DIR>  Extract resident $DATA streams

CSV Output:
  -a, --all-times      Include every $FILE_NAME time
  -s, --short-names    Include DOS 8.3-name rows
      --cutoff <DATE>  Keep recent SI-created or SI-modified records

Records:
  -x, --recover-slack  Recover file names from FILE record slack

Resident Data:
  -i, --include-resident-data      Put resident $DATA in the CSV
  -E, --resident-extensions <EXT>  Only these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  Only data up to this size

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail
  -L, --list-directory <ENTRY|PATH>  List a directory's contents
```

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff '2026-10-20T00:00:00Z' -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 32000-32100 | jq .
juntei parse-ntfs mft -f '$MFT' -x -o mft-with-slack.csv
juntei parse-ntfs mft -f '$MFT' -i -E ps1,bat,vbs,js -o scripts.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Users\alice\Downloads'
```

絕對不要執行以 `-e` 擷取出的常駐資料流；Juntei 將它們命名為 `.dontrun` 是有原因的。
`-i` 的常駐資料欄位是相同的位元組：請將其視為不受信任的內容。

`-D` 會印出一個項目：其標頭、每個屬性（含資料執行區段），以及其 slack 中的名稱（以
`-t json -x` 寫出的形式），接著是記錄的位元組。`-L` 會列出目錄中的每個名稱（使用中與已刪除），
並附上其項目與序號、類型、大小與修改時間。目錄可以依項目指定（`30`、`0x1E`、`30-1`），
也可以依路徑指定（`\Windows\System32`、`C:/Windows`；不區分大小寫）。

## CSV 輸出欄位 { #csv-output-fields }

| 欄位 | 判讀 |
|---|---|
| `EntryNum` (Entry Number) | MFT 記錄編號。請與 `SN` 合併使用；記錄編號會被重複使用。 |
| `SN` (Sequence Number) | NTFS 參照所使用的世代計數器。不相符可能揭露過時／已刪除的關係。 |
| `InUse` | 收集時的配置旗標。`false` 代表記錄為可用／已刪除，並不代表所有位元組都完整無缺。 |
| `ParentEntryNum` (ParentEntryNumber)、`ParentSN` (Parent Sequence Number) | 取自 `$FILE_NAME` 的父目錄參照。請驗證這兩個部分。 |
| `ParentPath` | 重建的父項路徑。未知／孤立標記會揭露無法解析的祖系資訊。 |
| `File` (Filename) | 取自目前這一列之 `$FILE_NAME` 或資料流的名稱。一筆記錄可以有多列。 |
| `Ext` (Extension) | 為方便起見從名稱剖析出的副檔名；並非可信任的檔案類型判定。 |
| `Size` (File Size) | 與該列／資料流相關的邏輯大小。疏鬆／壓縮的配置可能不同。 |
| `RefCnt` (Reference Count) | NTFS 硬連結參照計數；請審慎地與殘存名稱的數量進行比較。 |
| `ReparseTarget` | 對支援之重新剖析點（例如連接點／符號連結）解碼後的目標。路徑可能被重新導向。 |
| `Dir` (Directory) | 記錄具有目錄旗標。 |
| `HasADS` (Has Alternate Data Streams) | 記錄包含具名的替代資料流。這並不代表它是惡意的。 |
| `IsADS` (Is Alternate Data Stream) | 這一列代表具名的替代資料流，而非未命名的檔案資料。 |
| `SI<FN` (Standard Information Before File Name) | SI 建立時間早於不同的 FN 建立時間。請調查複製／時間戳記竄改以及合法的原因。 |
| `uSecZeros` (Microsecond Zeros) | SI 建立／修改時間剛好落在毫秒邊界上。僅為薄弱的時間戳記竄改啟發式判斷。 |
| `Copied` | 修改時間早於建立時間。通常與檔案複製一致；並非攻擊者行為的證明。 |
| `SI-Flags` (Standard Information Flags) | `$STANDARD_INFORMATION` 的檔案屬性／旗標。隱藏／系統旗標可作為有用的樞紐。 |
| `Type` (Name Type) | `$FILE_NAME` 命名空間（Win32、DOS、POSIX 或合併）。用於避免重複計算名稱。 |
| `DataModified (SI)`、`Accessed (SI)`、`MetadataChanged (SI)`、`Created (SI)` | SI 時間。常用的時間軸欄位，但使用者模式工具可以變更其中數個。 |
| `DataModified (FN)`、`Accessed (FN)`、`MetadataChanged (FN)`、`Created (FN)` | FN 時間。更新方式不同，一般 API 通常較難變更；但仍非不可變更。 |
| `USN` (Update Sequence Number) | 與該記錄之 SI 屬性相關的最後一個 USN。若該 USN 仍殘存，可用來樞紐至 `$J`。 |
| `LogfileSN` (Logfile Sequence Number) | 取自 FILE 記錄標頭的 NTFS `$LogFile` 序號；可用於底層的交易分析。 |
| `SID` (Security ID) | NTFS `$Secure` 的索引；本身並非 Windows SID。請將它與 [`parse-ntfs secure-sds`](secure-sds.md) 的 `SecurityID` 結合，以取得檔案的擁有者與 ACL。 |
| `ObjectID-FileDroid` | 若存在，為物件 ID／分散式連結追蹤識別碼；可與 LNK／Jump List 的 droid 進行關聯。 |
| `LoggedUtilStream` | 在支援的情況下，解碼後的 `$LOGGED_UTILITY_STREAM` 脈絡。請檢視不熟悉的值。 |
| `ZoneID-Contents` | 常駐的 `Zone.Identifier` 內容，例如 ZoneId 與下載 URL。它可能不存在、已被移除或為非常駐。 |
| `SourceFile` | 輸入的 `$MFT`，在合併多台主機或多個磁碟區時不可或缺。 |
| `ResidentDataBase64`、`ResidentDataHex`、`ResidentDataASCII` | 使用 `-i` 時：該列常駐 `$DATA`（檔案本身的，或替代資料流的）的內容，分別以 Base64、位元組（`41-42`）表示，以及在其為可列印 ASCII 時以文字表示。資料為非常駐或被 `-E`／`-M` 篩除時為空。 |
| `ParseErrors` | 影響這一列的損毀或不支援的關係。篩選時絕對不要捨棄此欄位。 |
| `FromSlack` | 使用 `-x` 時：對於從 FILE 記錄 slack 中復原的名稱為 `True`。其 `EntryNum` 與 `SN` 是該名稱所指向的檔案（其項目標頭已被覆寫時為空），其復原的時間戳記位於 `(FN)` 欄位中，且 `InUse` 為 `False`。 |
| `SlackRecord`、`SlackOffset` | 使用 `-x` 時：其 slack 保存該名稱的 FILE 記錄，以其在 `$MFT` 中的位置表示，而非寫在記錄內部的記錄編號（擴充記錄則為其自身的位置，即使其名稱是與其基底記錄一起列出），以及該名稱在 `$MFT` 中的位元組位移。slack 列的 `ParseErrors` 是該記錄的問題，例如 `fixup values do not match`：復原的位元組可能受到影響。 |
| `CopyOfEntryInUse` | 使用 `-x` 時：某個使用中的檔案在同一目錄中仍具有此名稱（且在參照殘存時為同一個檔案）。沒有任何使用中檔案具有的名稱才是值得關注的：已刪除、已重新命名，或被重複使用之記錄的前一個佔用者。 |

時間戳記欄位依循 MACB 順序（Modified、Accessed、Changed、Birth／建立）：
`DataModified` SI/FN、`Accessed` SI/FN、`MetadataChanged` SI/FN，然後是 `Created` SI/FN。

`(SI)` 代表 `$STANDARD_INFORMATION`；`(FN)` 代表 `$FILE_NAME`。這些名稱描述的是完整記錄的
一般來源。對於沒有 SI 屬性的一般檔案名稱列，既有的備援做法會將 FN 建立時間放在
`Created (FN)` 中，並將 FN 的修改／記錄變更／存取值分別放在 `DataModified (SI)`、
`MetadataChanged (SI)` 與 `Accessed (SI)` 中。復原的 slack 列則將四個時間全部放在
`(FN)` 欄位中，SI 欄位保持空白。一般列的備援值**並非**獨立的 SI 證據。判斷時間戳記來源時，
請使用原始的 JSON 屬性與 slack 來源資訊。

`-t ys-ntfs-parser-csv`（舊名 `ys-mft-parser-csv`，仍可接受）使用所維護剖析器以項目為導向的
結構描述。這些易讀的欄位名稱適用於預設的 `-t csv` 輸出，包括 `auto-parse` 的 `MFT.csv`。
JSON/JSONL 會保留每個屬性（包括以十六進位表示的常駐 `$DATA`）與 `FullPath`；當扁平的調查用
CSV 省略了底層結構時，請使用它。使用 `-x` 時，每個 JSON 項目還會有 `SlackFileNames`。

## 調查策略 { #investigation-strategy }

1. 篩選時間範圍，但保留已刪除的記錄與兩組時間戳記。
2. 搜尋完整路徑、可疑的雙重副檔名、ADS、資料整備目錄，以及已知的雜湊／名稱。
   使用 `-x` 時，請納入沒有任何使用中檔案具有的 slack 名稱（`CopyOfEntryInUse` 為 `False`）；
   使用 `-i -E ps1,bat,vbs,js` 時，可直接從 CSV 讀取小型指令碼。
3. 以項目／序號與 `USN` 樞紐至 USN Journal。
4. 將 Prefetch／LNK／Jump List 的 MFT 參照與目標記錄進行比較。
5. 將刪除行為與 Defender 隔離資料或 USN `FILE_DELETE`／重新命名活動進行關聯分析。

## 延伸資訊 { #further-details }

- [Microsoft：主檔案表](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications：NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [ys-ntfs-parser 的 mft crate](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/mft)
- [Omer Ben-Amram 的 mft 剖析器](https://github.com/omerbenamram/mft)
- [libfsntfs 格式文件](https://github.com/libyal/libfsntfs/tree/main/documentation)
