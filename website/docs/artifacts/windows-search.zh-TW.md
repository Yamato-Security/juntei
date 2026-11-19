# Windows Search 索引 { #windows-search-index }

## 索引的內容 { #what-the-index-records }

Windows Search（`WSearch` 服務）會對使用者設定檔、開始功能表、Outlook 的信箱、Internet Explorer 的歷程記錄、離線檔案，以及其範圍內的其他所有內容進行編目，並將讀取到的內容保存在 `Windows.edb` 這個 ESE 資料庫中。其中有兩個部分與調查有關：

- **屬性存放區**（`SystemIndex_PropertyStore`；Windows 7 上為 `SystemIndex_0A`），每個已建立索引的項目各占一列：其路徑、名稱、類型與大小、其擁有者、索引子從檔案複製而來的建立、修改與存取時間、索引子讀取該項目的時間、文件的標題與作者、其文字的開頭、電腦名稱，以及其他數百個屬性；
- **收集器**（gatherer）資料表（`SystemIndex_Gthr`、`SystemIndex_GthrPth`），是編目器自己對每個項目的記錄：其 URL 路徑、排入佇列時的修改時間、編目編號與旗標，以及刪除計數器。

在檔案、郵件或網頁消失之後，項目仍會留在索引中，直到編目器注意到該刪除為止。因此索引會列出已不存在的檔案，連同其建立索引當時的中繼資料與文字摘錄。

<span class="evidence-strength">最佳結論：</span>索引子曾在 `GatherTime`、於所指名的電腦上，以所記錄的中繼資料讀取位於所記錄路徑的該項目。索引顯示該檔案曾經存在及其內容。它無法顯示使用者曾開啟該檔案。

## 位置與收集 { #location-and-collection }

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

Windows 7 至 Windows 11 21H2，以及同時期的 Windows Server 版本，將索引保存在 `Windows.edb` 中。Windows 11 22H2 及之後的版本則將其保存在同一資料夾內的 SQLite 資料庫中（`Windows.db`、`Windows-gather.db`、`Windows-usn.db`）；Juntei 會找出並指名這些檔案，並將其回報為不支援的較新格式。

請從磁碟映像、磁碟區陰影複製，或在停止 `WSearch` 服務之後，收集整個 `Applications\Windows` 資料夾：服務執行期間資料庫處於開啟狀態，而其最新的變更只存在於 `MSS*.log` 交易記錄檔中，Juntei 不會重播這些記錄檔。從執行中的系統複製而來的索引會處於 dirty 狀態；輸出會加以說明，並讀取磁碟上的內容。目錄輸入與 `auto-parse` 會讀取任何深度中的每一個 `Windows.edb`；單一檔案請使用 `-f`。

## 分析注意事項 { #analyst-cautions }

- 請解析不會變動的證據副本。Juntei 使用記憶體映射來有效存取檔案；讀取期間請勿修改或截短資料庫。

- `GatherTime` 是索引子讀取項目的時間，而不是使用者建立、變更或開啟它的時間。建立索引在背景執行，通常在變更後數分鐘到數小時才進行，並會在電腦忙碌或使用電池時暫停。
- `DateCreated`、`DateModified` 與 `DateAccessed` 是索引子從檔案複製而來的副本。項目再次建立索引時會重新整理這些值，因此它們描述的是 `GatherTime` 當時的檔案。
- `ItemPath` 是顯示路徑。`ItemURL` 與 `GatherPath` 是編目器的 URL：檔案為 `file:`（Windows 8 及之後的版本以 `[GUID]` 表示磁碟區）、Internet Explorer 的歷程記錄為 `iehistory://{SID}/`、Outlook 項目為 `mapi://`、離線檔案為 `csc://{SID}/`，Windows 應用程式的內容則為 `winrt://{SID}/`。
- `User` 是項目路徑中所指名的設定檔（`C:\Users\<name>\…`）。它是項目所在的位置，而不是建立者；`FileOwner` 則是索引子所看到的檔案擁有者。
- `AutoSummary` 是項目文字的開頭。它可能引用一份其後已被刪除或加密的文件，而且它是經截斷的摘錄，並非文件本身。
- dirty 的資料庫可能含有其分支頁面尚未指向的頁面。Juntei 會沿著葉頁面自己的鏈結讀取，並在摘要中說明以此方式讀取了多少頁面。其最新的變更仍然留在交易記錄檔中。
- 位於收集器資料表中、但沒有屬性存放區資料列的項目（`InTables` 為 `Gthr`），表示它曾排入佇列或已編目，而其屬性尚未寫入或已被移除。其路徑、名稱與收集器時間仍然存在。
- Windows 7 以其專屬的壓縮且經混淆的編碼儲存字串。Juntei 會將其解碼；無法解碼的值會維持十六進位，並在 `Problems` 中加註。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

CSV 是預設檔案格式。指定 `-t csv`、`-t json` 或 `-t jsonl` 而不指定 `-o` 時，記錄會寫至標準輸出。兩個選項都不指定時，只顯示摘要。加入 `-C` 可覆寫現有輸出。即使使用 `-C`，輸出也絕不會寫入並覆蓋任何一個輸入檔案。

Juntei 會為每個已建立索引的項目寫入一筆記錄，並以項目的識別碼將屬性存放區與收集器資料表結合。摘要會依結構描述統計資料庫數、dirty 的資料庫數、同時位於兩個資料表中的項目數、僅位於收集器中的項目數、僅位於屬性存放區中的項目數、資料夾數、標記為已刪除的項目數、各存放區的項目數、沒有儲存時間的項目數，以及問題數。

損毀不會中斷執行。被截斷的資料庫會保留其剩餘頁面所含的項目，並由摘要說明無法讀取的內容。名為 `Windows.edb` 但並非 Windows Search 索引的檔案會被回報並略過。其他資料庫仍會照常讀取。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `DocumentID` | 項目在索引中的識別碼：屬性存放區的 `WorkID`（Windows 7 上為 `DocID`）與收集器資料列的 `DocumentID`。 |
| `InTables` | `Gthr+PropertyStore`、`Gthr`（已排入佇列或已編目但沒有屬性的項目）或 `PropertyStore`。 |
| `ItemPath` | `System_ItemPathDisplay`：Windows 所顯示的路徑。屬性存放區沒有資料列時為空。 |
| `ItemURL` | `System_ItemUrl`：編目器為該項目所用的 URL（`file:`、`iehistory://`、`mapi://`、`csc://`、`winrt://`）。 |
| `GatherPath` | 收集器的路徑，由範圍樹狀結構與資料列的檔名組合而成。 |
| `Name` | `System_ItemName`，或收集器資料列的檔名。 |
| `ItemType`、`ItemTypeText` | `System_ItemType`（副檔名，或 `Directory`）及其顯示名稱。 |
| `Kind` | `System_Kind`：`document`、`folder`、`link`、`program`、`email`、`picture`、…；多個值以 `\|` 連接。 |
| `MimeType` | `System_MIMEType`。 |
| `Store` | `System_Search_Store`：`file`、`mapi`（Outlook）、`iehistory`、`csc`（離線檔案）或 `winrt`（Windows 應用程式）。 |
| `IsFolder`、`IsDeleted` | 依原樣儲存的 `System_IsFolder` 與 `System_IsDeleted`。 |
| `DeletedCount` | 收集器資料列的刪除計數器。 |
| `Size` | 以位元組計的 `System_Size`。資料夾存放的是預留位置，因此為空。 |
| `FileAttributes` | 以數字表示的 `System_FileAttributes`（16 為目錄、32 為封存、2 為隱藏、…）。 |
| `FileOwner` | 索引子讀取時的 `System_FileOwner`（`DOMAIN\user`）。 |
| `DateCreated`、`DateModified`、`DateAccessed` | 索引子複製當時的檔案時間（UTC）。 |
| `ItemDate` | `System_ItemDate`：項目本身的日期，例如郵件的傳送時間或相片的拍攝日期；對檔案而言通常是精確到秒的修改時間。 |
| `GatherTime` | `System_Search_GatherTime`：索引子讀取該項目的時間（UTC）。 |
| `GatherLastModified` | 收集器資料列的 `LastModified`：編目器將該項目排入佇列時的修改時間（UTC）。 |
| `Title`、`Author` | `System_Title` 與 `System_Author`。 |
| `AutoSummary` | `System_Search_AutoSummary`：項目文字的開頭。 |
| `CompName` | `System_ComputerName`：索引所屬的電腦。 |
| `TargetURL`、`DateVisited` | Internet Explorer 歷程記錄項目的 URL 與造訪時間（`System_Link_TargetUrl`、`System_Link_DateVisited`）。 |
| `ActivityApp`、`ActivityText`、`ActivityContentURI`、`ActivityStartTime`、`ActivityEndTime` | Windows 10 活動歷程記錄項目（`System_Activity_*`、`System_ActivityHistory_*`）。 |
| `User`、`UserSource` | 項目路徑中所指名的設定檔，以及它是從哪個路徑讀取的（`ItemPath`、`ItemURL` 或 `GatherPath`）。不在設定檔內時為空。 |
| `ScopeID`、`SDID` | 收集器資料列的範圍識別碼與安全性描述元識別碼。 |
| `CrawlNumber` | 收集器資料列的 `CrawlNumberCrawled`。 |
| `GatherFlags` | 收集器資料列的 `TransactionFlags`。 |
| `Priority` | 仍在佇列中之項目的編目優先順序。項目一經編目，ESE 就會將其標示為 NULL，因此大多數項目的此欄位為空。 |
| `PropertyCount` | 屬性存放區為該項目保存的屬性數量。 |
| `OtherProperties` | 其他所有屬性，以 `Name=Value` 表示並以 `; ` 分隔。 |
| `SourceFile` | 所讀取的資料庫，為相對於輸入目錄的路徑；使用 `-f` 時則為其檔名。 |
| `Problems` | 無法解碼的值、不是時間的時間、重複的資料列，以及損毀。請勿隱藏。 |

JSON 與 JSONL 保留相同的欄位，但值帶有型別：數字維持為數字，缺少的值為 `null`。它們以 `Properties` 取代 `OtherProperties`，其中包含該項目的每一個屬性及其名稱、數值識別碼、資料行型別與值；對時間而言，則會在其解讀出的時間旁保留原始的 FILETIME tick 數。`Gather` 保存收集器資料列的各資料行。

## 時間軸行為 { #timeline-behavior }

`auto-parse` 會將記錄儲存為 `FileExistence/WindowsSearch.csv`（或 `.json`／`.jsonl`），並加入合併的 `-j` 時間軸。`super-timeline` 會從 `DateCreated`、`DateModified` 與 `DateAccessed`（Windows Search 所記錄的檔案時間）、從 `GatherTime`（項目建立索引的時間）、從 Internet Explorer 歷程記錄項目的 `DateVisited`，以及從活動的 `ActivityStartTime` 與 `ActivityEndTime` 產生事件。`GatherLastModified` 只有在屬性存放區沒有該項目修改時間時才會成為事件。沒有上述任何時間的項目會保留在剖析輸出中，並計為無時間的記錄。

## 調查策略 { #investigation-strategy }

1. 將 `Store` 篩選為 `file`，並在 `ItemPath`、`Name` 與 `AutoSummary` 中搜尋案件相關的檔案、名稱與字詞。索引會列出其後已被刪除的檔案。
2. 將 `GatherTime` 解讀為該檔案以所記錄的中繼資料存在的最後時間，並將 `DateModified` 解讀為它在當時的修改時間。
3. 使用 `FileOwner` 與 `User` 將檔案與帳戶對應起來，再以 `$MFT`、USN 日誌與 `$Secure:$SDS` 確認。
4. 對於電子郵件（`Store` 為 `mapi`）與 Internet Explorer 歷程記錄（`iehistory`），請轉向 Outlook 的檔案與 WebCache 取得完整記錄。
5. 跨主機比較索引：`CompName` 指名索引所屬的電腦，而從映像收集的索引列出的是該映像上的檔案。

## 測試過的版本 { #tested-versions }

測試使用的是公開索引，其結果已由其他工具獨立讀取：

- plaso 的 Windows 7 索引（`SystemIndex_0A`），已與 libyal 的 `esedbexport` 比對，包括 Windows 7 的字串編碼。
- DFIRArtifactMuseum 的 Windows 10 索引（維持 dirty 狀態），已與 Stroz Friedberg 的 SIDR 比對；及其 Windows 11 索引（ESE 格式修訂版 200），已與 `esedbexport` 比對。
- DFIRArtifactMuseum 的 Windows 11 22H2 SQLite 索引，回報為不支援的較新格式。
- 被截斷、空白或其他種類的資料庫。

## 延伸資訊 { #further-details }

- [Microsoft — Windows Search 概觀](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE 資料庫檔案格式](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb — Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg — SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft — Extensible Storage Engine 原始碼](https://github.com/microsoft/Extensible-Storage-Engine)
