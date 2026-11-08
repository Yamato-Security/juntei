# 網頁瀏覽器 { #web-browsers }

## 什麼是瀏覽器資料庫 { #what-the-browser-databases-are }

瀏覽器會將其瀏覽歷程、下載、Cookie 與表單項目保存在各使用者設定檔內的 SQLite 資料庫中。
Chromium 系列瀏覽器（Chrome、Edge、Brave、Opera、Vivaldi）與 Firefox 系列瀏覽器各自使用不同的
檔案與資料表，但記錄的活動種類相同：

- **瀏覽歷程**：造訪過的網頁；每次造訪都附有其時間、瀏覽器如何來到該網頁（手動輸入、連結、
  重新導向、書籤），以及（在 Chromium 中）該網頁位於前景的時間長度。
- **下載**：檔案的來源 URL（以及途中經過的重新導向）、儲存位置、大小、下載開始與結束的時間，
  以及是否已完成。
- **Cookie**：設定 Cookie 的網站，以及 Cookie 的設定時間、最後一次傳送時間與到期時間。
- **表單歷程**：在網頁表單與搜尋框中輸入的值、輸入的次數，以及首次與最後一次輸入的時間。

瀏覽器活動往往是使用者行為最直接的記錄：使用者點進的釣魚網頁、下載的工具、使用的網頁郵件，
以及輸入的搜尋字詞。

<span class="evidence-strength">最佳結論：</span>瀏覽器設定檔記錄了在所記錄的時間造訪該 URL，
或將下載的檔案儲存到所記錄的路徑；資料庫並未說明當時是誰在操作鍵盤。

## 位置與收集 { #location-and-collection }

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

`<Profile>` 為 `Default`、`Profile 1`、`Profile 2` 等。Chromium 自 96 版起將 `Cookies` 移至
`Network`；較舊的版本則將其放在 `History` 旁。Firefox 直到 25 版都將下載保存在
`downloads.sqlite` 中；之後的版本則保存在 `places.sqlite` 中。

請收集設定檔中的每一個檔案，並將每個資料庫連同其旁與其同名的檔案一併收集：
`places.sqlite-wal`、`cookies.sqlite-wal`、`formhistory.sqlite-wal`、`History-journal`。

!!! warning "預寫式記錄檔"
    Firefox 以預寫式記錄（write-ahead log）模式保存其資料庫：最新的變更只存在於 `-wal` 檔案中，
    直到瀏覽器將其複製到資料庫為止（即檢查點）。Juntei 會如同瀏覽器一樣，將每個資料庫連同其
    `-wal` 檔案一起讀取。若缺少該檔案，最近的造訪、Cookie 與表單項目將會缺漏，而在記錄檔中已被
    刪除的項目仍會被讀取為現存項目。

## 已刪除的項目 { #deleted-entries }

當瀏覽器刪除瀏覽歷程時，資料列會從資料表中移除，但其記錄可能仍殘留在資料庫的可用空間中、
空閒頁清單（freelist）上，或資料庫頁面的舊版本中：這些舊版本可能位於 `-wal` 檔案內，也可能位於
資料庫檔案內、已被 `-wal` 中的新版本所取代。Juntei 會從這些位置復原已刪除的造訪、網頁、下載與
表單項目，並將其輸出為 `Status` 為 `Deleted` 的資料列：

- `Confidence` 表示記錄與其資料表比對的確定程度：每個值都符合其欄位類型的完整記錄為 `High`；
  損毀的記錄或可符合多個資料表的記錄則為 `Medium` 或 `Low`。
- `RecoveredFrom` 表示記錄所在的位置：頁面中的區域（儲存格、空閒區塊、未配置空間、空閒頁面）、
  哪一個頁面及其哪一個版本，以及記錄在頁面中的位移。WAL 訊框從 0 開始編號。
- `LostColumns` 列出損毀記錄已不再保有的欄位：這些欄位的值為空白。

瀏覽器以 SQLite 的 `secure_delete` 進行刪除：資料庫中被釋放的空間會被清零。因此，可復原的內容
大多位於頁面的舊版本中：即 `-wal` 檔案仍保有的版本，以及資料庫檔案本身保有、已被 `-wal` 取代之
頁面的副本，這些都只會保留到下一次檢查點為止。請將 `-wal` 檔案與其資料庫一併複製，並在瀏覽器
再次執行之前完成。

## 分析注意事項 { #analyst-cautions }

- 時間皆為 UTC。Chromium 與 Firefox 記錄到微秒；Chromium 的表單歷程則只記錄到秒。
- 有造訪並不代表使用者曾輸入網址或點擊連結：重新導向、框架以及由其他程式開啟的網頁也都算是
  造訪。請判讀 `Transition`、`Typed` 與 `Hidden`。
- Chromium 會加密 Cookie 值：這類 Cookie 的 `Value` 為空白。Juntei 不會進行解密。
- `Removed` 的表單歷程資料列來自 Firefox 的 `moz_deleted_formhistory`，此資料表由 Android 版
  Firefox 為了 Sync 而填入。桌面版 Firefox 通常會讓它保持空白。
- 信賴度為 `Low` 的已刪除項目可能是其他記錄的片段：請將其與現存的資料列及其他證據相互比對。
- 沒有已刪除的資料列，並不代表沒有任何內容被刪除：清零的空間與檢查點不會留下任何可復原的
  內容。目前尚不會復原已刪除的 Cookie。
- `Browser`、`User` 與 `Profile` 取自資料庫的路徑：被複製到其他位置的設定檔，只會保有其路徑
  所呈現的資訊。若路徑中沒有已知的瀏覽器資料夾，`Browser` 會是 `Chromium-based` 或
  `Firefox-based`。
- 不會讀取回復日誌（`-journal`），也不會讀取 Internet Explorer 與舊版 Edge 的
  `WebCacheV01.dat`。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

`-d` 會依名稱尋找這些檔案，並依其資料表辨識每個檔案：其他程式的 `History` 檔案會被略過。
`-f` 則不論資料庫的名稱為何都會加以讀取。

## 輸出資料表 { #output-tables }

CSV 會建立四個檔案；沒有資料列的檔案只含標題列。JSON 與 JSONL 資料列會在 `SourceFile` 之後
帶有 `Table`（`History`、`Downloads`、`Cookies` 或 `FormHistory`）。

| CSV 檔案 | 內容 |
|---|---|
| `Browser-History.csv` | 造訪，包括現存與已刪除的造訪；以及 URL 不屬於任何現存網頁的已刪除網頁。 |
| `Browser-Downloads.csv` | 下載，包括現存與已刪除的下載。 |
| `Browser-Cookies.csv` | Cookie。 |
| `Browser-FormHistory.csv` | 表單項目，包括現存與已刪除的項目；以及 Firefox 的移除記錄。 |

## 欄位 { #fields }

每個資料表都以下列欄位開頭：

| 欄位 | 判讀 |
|---|---|
| `SourceFile` | 讀取該資料列的資料庫。 |
| `User` | 路徑所在設定檔的 Windows 使用者（`Users` 之後的資料夾）。 |
| `Browser` | 路徑所指出的瀏覽器：`Chrome`、`Edge`、`Brave`、`Opera`、`Vivaldi`、`Firefox` … |
| `Profile` | 瀏覽器設定檔：即資料庫所在的資料夾。 |
| `Status` | `Live`、`Deleted`（復原所得）或 `Removed`（Firefox 的移除記錄）。 |

### 瀏覽歷程 { #history }

| 欄位 | 判讀 |
|---|---|
| `RecordType` | `Visit`；或 `Page`，代表其造訪可能也已消失的已刪除網頁。 |
| `VisitTime` | 造訪開始的時間（UTC）。 |
| `URL`、`Title` | 網頁及其標題，為最後一次所見的內容。 |
| `Transition` | 瀏覽器如何來到該網頁。Chromium：先列出核心類型，再列出其修飾詞（`LINK`、`TYPED`、`AUTO_BOOKMARK`、`FORM_SUBMIT`、`RELOAD` …，搭配 `CHAIN_START`、`SERVER_REDIRECT`、`FROM_ADDRESS_BAR` …）；Firefox：`LINK`、`TYPED`、`BOOKMARK`、`EMBED`、`REDIRECT_PERMANENT`、`REDIRECT_TEMPORARY`、`DOWNLOAD`、`FRAMED_LINK`、`RELOAD`。 |
| `VisitDuration` | 網頁位於前景的秒數（Chromium；未量測時為 0）。 |
| `VisitCount` | 該網頁的造訪總次數。 |
| `Typed`、`TypedCount` | 該 URL 是否曾被手動輸入，以及輸入的次數（Chromium）。 |
| `Hidden` | 不顯示於歷程清單中：子框架、重新導向的來源、錯誤。 |
| `Frecency` | Firefox 為網址列建議所計算的網頁排名。 |
| `VisitID`、`FromVisitID` | 該造訪的資料列 ID，以及其來源造訪的資料列 ID（參照網頁、重新導向的來源）。 |
| `LastVisitTime` | 已刪除網頁的最後一次造訪（UTC）。 |
| `PageSource` | 已刪除造訪的 URL 與標題來自何處：`Live`（現存網頁）、`Recovered`（已刪除網頁）或 `NotFound`。 |
| `Confidence`、`RecoveredFrom`、`LostColumns` | 已刪除項目的復原方式；請參閱[已刪除的項目](#deleted-entries)。 |

### 下載 { #downloads }

| 欄位 | 判讀 |
|---|---|
| `StartTime`、`EndTime` | 下載開始與結束的時間（UTC）。 |
| `URL`、`RedirectChain` | 檔案的來源，以及途中經過的每個 URL（Chromium；在 CSV 中以 ` -> ` 分隔）。 |
| `TargetPath`、`CurrentPath` | 檔案的儲存位置（Firefox 中為 `file://` URI），以及下載期間寫入的位置。 |
| `ReceivedBytes`、`TotalBytes` | 已接收的位元組數，以及檔案大小。 |
| `State` | `Complete`、`InProgress`、`Cancelled`、`Interrupted`、`Paused`、`Blocked` 或 `Dirty`（因判定為惡意軟體而遭封鎖）。 |
| `DangerType` | Chromium 對該檔案的判定：0 不危險、1 危險的檔案類型、2 危險的 URL、3 危險的內容、4 可能有危險、5 不常見、6 由使用者保留、7 危險的主機、8 可能是不需要的程式 … |
| `InterruptReason` | 中斷的下載停止的原因（Chromium；無中斷時為 0）。 |
| `Referrer`、`TabURL` | 參照網頁，以及啟動下載之分頁中的網頁。 |
| `MimeType`、`Opened` | 伺服器提供的檔案類型，以及是否曾從瀏覽器開啟該檔案（Chromium）。 |
| `FileDeletedByUser` | 使用者是否透過瀏覽器的下載清單，從磁碟刪除了下載的檔案（Firefox）。 |
| `DownloadID` | 該下載的資料列 ID。 |

### Cookie { #cookies }

| 欄位 | 判讀 |
|---|---|
| `Created`、`LastAccessed`、`Expires` | Cookie 的設定時間、最後一次傳送時間與到期時間（UTC；工作階段 Cookie 的 `Expires` 為空白）。 |
| `Host`、`Name`、`Path` | Cookie 所屬的網站，以及其名稱與路徑。 |
| `Value` | 所儲存的值：經 Chromium 加密時為空白。 |
| `Secure`、`HttpOnly`、`Persistent` | 僅透過 HTTPS 傳送、對指令碼隱藏、跨工作階段保留（Chromium）。 |
| `GoogleAnalytics` | Google Analytics Cookie 解碼後的欄位（`__utma`、`__utmb`、`__utmt`、`__utmz`）：造訪時間、工作階段，以及行銷活動來源與搜尋字詞。 |
| `RowID` | 該 Cookie 的資料列 ID。 |

### 表單歷程 { #form-history }

| 欄位 | 判讀 |
|---|---|
| `FirstUsed`、`LastUsed` | 首次與最後一次輸入該值的時間（UTC）。 |
| `RemovedTime` | Firefox 移除記錄所指出的項目移除時間（UTC）。 |
| `FieldName`、`Value` | 表單欄位（`email`、`q`；Firefox 搜尋列為 `searchbar-history`）以及所輸入的內容。 |
| `Count` | 該值被輸入的次數。 |
| `GUID` | Firefox 對該項目的識別碼，移除記錄即以此識別碼指名項目。 |
| `Sources` | Firefox 儲存該項目時的來源，僅見於保存此資訊的版本（在 CSV 中以垂直線分隔）。 |
| `RowID` | 該項目的資料列 ID。 |

## 自動剖析 { #automatic-parsing }

`auto-parse` 會將這些資料表以 `Browser-History.csv`、`Browser-Downloads.csv`、
`Browser-Cookies.csv` 與 `Browser-FormHistory.csv` 寫入 `BrowserActivity`（使用 `-t json` 或
`-t jsonl` 時則為單一的 `Browser.json` 或 `Browser.jsonl` 檔案），並附上
`BrowserActivity/Browser-logs.txt`。合併的 `-j` 時間軸包含每一筆資料列，並帶有 `Comp` 以及
`Artifact` 識別碼 `browser`。`-Z` 會轉換 `VisitTime`、`LastVisitTime`、`StartTime`、`EndTime`、
`Created`、`LastAccessed`、`Expires`、`FirstUsed`、`LastUsed` 與 `RemovedTime`。

## 延伸資訊 { #further-details }

- [Chromium — 網頁轉換類型](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — 下載危險類型](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — Places 資料庫](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — 預寫式記錄](https://www.sqlite.org/wal.html)
- [Plaso — 瀏覽器 SQLite 剖析器](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
