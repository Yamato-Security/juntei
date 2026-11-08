# Internet Explorer 與舊版 Edge { #internet-explorer-and-legacy-edge }

## 什麼是 WebCache 資料庫 { #what-the-webcache-database-is }

`WebCacheV01.dat` 是每位使用者設定檔中 Windows 網際網路功能（WinINet）的資料庫，屬於 ESE 資料庫。
Internet Explorer 10 與 11 將其瀏覽歷程記錄、下載、快取與 Cookie 保存在其中，舊版 Edge 亦然；
舊版 Edge 即 Edge 改以 Chromium 為基礎之前、Windows 10 所附的 Edge。每一種記錄各為一個容器，
也就是一個獨立的資料表：

- **歷程記錄**（`History`，以及以一天或一週為單位的 `MSHist01…`）：造訪過的 URL；每筆都附有網頁
  標題、其記錄所屬的帳戶、最後一次造訪的時間以及造訪次數。檔案總管開啟的本機檔案也會以 `file:///` URL
  的形式記錄在此。
- **下載**（`iedownload`）：檔案的來源 URL、儲存位置與下載期間寫入的位置、大小、下載開始的時間、
  參照網頁以及 MIME 類型。
- **Cookie**：在 Windows 10 及更新版本中為 Cookie 本身（`CookieEntryEx` 資料表），包括其主機、
  名稱、值與時間；在此之前則為保存這些 Cookie 的 Cookie 檔案。
- **快取**（`Content`）：為顯示網頁而下載的檔案；每筆都附有其 URL、副本在磁碟上的位置，以及 HTTP
  回應標頭。使用 WinINet 的其他程式也會將其下載的內容快取在此。
- 網頁儲存空間（`DOMStore`）、相容性清單（`iecompat`）等。

這個資料庫比瀏覽器存在得更久：Internet Explorer 已停用，而以 Chromium 為基礎的 Edge 使用自己的
SQLite 資料庫（請參閱[網頁瀏覽器](browser.md)），但這個資料庫仍存在於每個設定檔中，保有先前記錄的
內容，以及檔案總管與其他程式至今仍在記錄的內容。

<span class="evidence-strength">最佳結論：</span>使用者的設定檔在所記錄的時間記錄了該 URL、該下載
或該快取檔案；資料庫並未說明當時是誰在操作鍵盤，也未說明快取的檔案是否曾被開啟。

## 位置與收集 { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

容器所指名的檔案（快取的檔案、Cookie 檔案）位於各自的資料夾中：
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\`、`INetCookies\`、`History\`，應用程式的檔案則位於
`%LOCALAPPDATA%\Packages\` 之下。

請收集整個 `WebCache` 資料夾。使用者登入期間，Windows 會持續開啟這個資料庫，因此從執行中系統
取得的複本通常處於 dirty 狀態：其最新的變更只存在於 `V01*.log` 交易記錄檔中。

!!! warning "交易記錄檔"
    Juntei 不會重播 ESE 交易記錄檔。未正常關閉的資料庫會在摘要中計為仍處於 dirty 狀態，並回報為
    問題：仍只存在於其記錄檔中的變更將會缺漏。若要納入這些變更，請先複製整個資料夾，並在該複本中
    執行 Windows 的 `esentutl /r V01 /d`，將記錄檔重播至資料庫。

## 分析注意事項 { #analyst-cautions }

- 時間皆為 UTC，精確到 100 奈秒；但以一天或一週為單位的歷程記錄容器的 `ModifiedTime` 除外：
  它是系統的本機時間，寫入時未附時差，`-Z` 也會保持原樣。請以來源系統當時的時區加以換算。它與
  `AccessedTime` 的差距通常即為該時區的時差，但並非總是如此：兩者可能是不同的時刻。
- 歷程記錄項目代表一個 URL，而非一次造訪：`AccessedTime` 是其最後一次造訪，`AccessCount` 是
  造訪的次數。`History` 容器中每個 URL 只有一筆項目；以一天或一週為單位的容器則保存該期間的
  項目，其名稱標示期間的第一天與最後一天的隔天（`MSHist012026110120261102`：2026 年 11 月 1 日）。
- 歷程記錄中的 `file:///` URL 是從檔案總管與其他程式開啟的本機檔案，而非網頁。
- `Account` 是項目記錄所屬的帳戶（`Visited: alice@…`）；`User` 是資料庫路徑所在的設定檔資料夾。
  兩者可能不同：設定檔資料夾不一定以其帳戶命名，而複製到他處的資料庫，其路徑中並沒有設定檔資料夾。
- `Package` 表示項目屬於誰：舊版 Edge（`microsoft.microsoftedge_8wekyb3d8bbwe`）、增強受保護模式下的
  Internet Explorer（`windows_ie_ac_001`）或其他應用程式。若為空白，則屬於使用者本身：Internet
  Explorer、檔案總管與其他程式。
- Microsoft 並未公開說明下載記錄的格式。Juntei 只在標頭完整位於記錄之內時才讀取開始時間及大小，
  並只在最後兩個字串為 URL 與 Windows 路徑時才讀取其路徑與其他字串。否則，該資料列會帶有項目本身的
  URL，並在無法讀取開始時間時帶有項目的存取時間，同時回報問題。`Publisher` 是瀏覽器所記錄的名稱，
  而非對檔案簽章的檢查結果。
- 快取項目表示該檔案曾被下載並快取。磁碟上的副本可能已不存在、已被取代，或從未被開啟。
- 目前尚不會復原已刪除的項目：ESE 可能會將已刪除的記錄保留在其頁面中，直到清理這些頁面為止，
  但 Juntei 只讀取現存的記錄。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

`-d` 會依名稱尋找 `WebCacheV01.dat`；`-f` 則不論資料庫的名稱為何都會加以讀取。

## 輸出資料表 { #output-tables }

CSV 會建立五個檔案；沒有資料列的檔案只含標題列。JSON 與 JSONL 資料列會在 `SourceFile` 之後
帶有 `Table`（`History`、`Downloads`、`Cookies`、`Entries` 或 `Containers`）。

| CSV 檔案 | 內容 |
|---|---|
| `WebCache-History.csv` | 歷程記錄容器的項目。 |
| `WebCache-Downloads.csv` | 下載。 |
| `WebCache-Cookies.csv` | Cookie（Windows 10 及更新版本）。 |
| `WebCache-Entries.csv` | 其他所有容器的項目：快取的檔案、Cookie 檔案、網頁儲存空間、相容性清單等。 |
| `WebCache-Containers.csv` | 容器。 |

## 欄位 { #fields }

每個資料表都以下列欄位開頭：

| 欄位 | 判讀 |
|---|---|
| `SourceFile` | 讀取該資料列的資料庫。 |
| `User` | 路徑所在設定檔的 Windows 使用者（`Users` 之後的資料夾）。 |
| `Package` | 該資料列所屬的應用程式：其容器的目錄（或其 Cookie 資料表的目錄）中 `AppData\Local\Packages` 之下的資料夾。屬於使用者本身的資料列為空白。 |

### 歷程記錄 { #history }

| 欄位 | 判讀 |
|---|---|
| `Container` | 容器：`History`，或 `MSHist01` 後接其期間的第一天與最後一天的隔天。 |
| `AccessedTime` | 該 URL 的最後一次造訪（UTC）；在以一天或一週為單位的容器中，為該期間內的最後一次造訪。 |
| `ModifiedTime` | 項目最後一次變更的時間（UTC）；在以一天或一週為單位的容器中，為系統的本機時間，未附時差。 |
| `CreationTime` | 項目建立的時間（UTC）。 |
| `ExpiryTime`、`SyncTime` | 項目預定離開歷程記錄的時間，以及最後一次同步的時間（UTC）。 |
| `Account` | 項目記錄所屬的帳戶。 |
| `URL` | 網頁或檔案，不含 `Visited: account@` 前置詞。 |
| `Title` | 項目所記錄的網頁標題；在以一天或一週為單位的容器中為空白。 |
| `AccessCount`、`SyncCount` | 該 URL 被造訪的次數，以及項目同步的次數。 |
| `EntryType`、`Flags` | 項目的類型與旗標，依原樣儲存。 |
| `EntryID`、`ContainerID` | 項目在其容器中的 ID，以及容器的 ID。 |

### 下載 { #downloads }

| 欄位 | 判讀 |
|---|---|
| `StartTime` | 下載開始的時間（UTC）。 |
| `URL` | 檔案的來源。 |
| `TargetPath`、`CurrentPath` | 檔案的儲存位置，以及下載期間寫入的位置：舊版 Edge 為 `.partial` 檔案，Internet Explorer 為快取中的副本。 |
| `TotalBytes` | 檔案大小。 |
| `Referrer` | 參照網頁。 |
| `MimeType` | 伺服器提供的檔案類型（Internet Explorer）。 |
| `Publisher` | 舊版 Edge 為檔案發行者所記錄的名稱。 |
| `AccessedTime` | 項目本身的存取時間（UTC），通常即為下載開始的時間。 |
| `DownloadID`、`ContainerID` | 項目在其容器中的 ID，以及容器的 ID。 |

### Cookie { #cookies }

| 欄位 | 判讀 |
|---|---|
| `LastModified`、`Expires` | Cookie 的設定時間與到期時間（UTC）。 |
| `Host`、`Name`、`Path` | Cookie 所屬的網站，以及其名稱與路徑。 |
| `Value` | 所儲存的值。 |
| `Flags` | Cookie 的旗標，依原樣儲存。 |
| `Directory` | 資料庫為該 Cookie 資料表指定的目錄，表示這些 Cookie 屬於誰（`…\INetCookies\ESE\`、舊版 Edge 的 `…\MicrosoftEdge\Cookies\ESE\`、某個應用程式的目錄）。 |
| `CookieTable`、`EntryID` | 資料表（`CookieEntryEx_<n>`），以及該 Cookie 在其中的 ID。 |

### 項目 { #entries }

| 欄位 | 判讀 |
|---|---|
| `Container` | 容器：`Content`（快取的檔案）、`Cookies`（Cookie 檔案）、`DOMStore`（網頁儲存空間）、`iecompat` 等。 |
| `AccessedTime` | 項目最後一次被使用的時間（UTC）。 |
| `ModifiedTime` | 伺服器所提供的資源最後修改時間（UTC）。 |
| `CreationTime` | 項目建立的時間（UTC）。 |
| `ExpiryTime`、`SyncTime` | 項目的到期時間，以及最後一次向伺服器確認的時間（UTC）。 |
| `URL` | 所儲存的 URL：快取檔案為其 URL，Cookie 檔案為 `Cookie:alice@example.com/`，另有 `DOMStore:…`、`iecompat:…`。 |
| `Account` | Cookie 檔案的帳戶。 |
| `Filename`、`FilePath` | 項目的檔案及其位置：容器的目錄加上項目所指名的子資料夾。 |
| `FileSize` | 檔案大小。 |
| `AccessCount`、`SyncCount` | 項目被使用的次數，以及向伺服器確認的次數。 |
| `EntryType`、`Flags` | 項目的類型與旗標，依原樣儲存。 |
| `RedirectURL` | 該 URL 重新導向的目標。 |
| `ResponseHeaders`、`RequestHeaders` | HTTP 回應標頭（`HTTP/1.1 200 OK`、`Content-Type`、`Server` 等）與要求標頭，依原樣儲存，包括其中的換行。 |
| `EntryID`、`ContainerID` | 項目在其容器中的 ID，以及容器的 ID。 |

### 容器 { #containers }

| 欄位 | 判讀 |
|---|---|
| `ContainerID`、`Name` | 容器的 ID 與名稱。 |
| `Directory` | 其檔案在磁碟上的位置。 |
| `SecureDirectories` | 其檔案所在的 `Directory` 子資料夾（在 CSV 中以逗號分隔）。 |
| `PartitionID` | 使用者本身的容器為 `M`；Internet Explorer 受保護模式的容器（`Low` 資料夾）為 `L`；應用程式的容器則為其應用程式容器的 SID。 |
| `LastAccessTime`、`LastScavengeTime` | 容器最後一次被使用的時間，以及最後一次被清理的時間（UTC）。 |

## 自動剖析 { #automatic-parsing }

`auto-parse` 會將這些資料表以 `WebCache-History.csv`、`WebCache-Downloads.csv`、
`WebCache-Cookies.csv`、`WebCache-Entries.csv` 與 `WebCache-Containers.csv` 寫入 `BrowserActivity`
（使用 `-t json` 或 `-t jsonl` 時則為單一的 `WebCache.json` 或 `WebCache.jsonl` 檔案），並附上
`BrowserActivity/WebCache-logs.txt`。合併的 `-j` 時間軸包含每一筆資料列，並帶有 `Comp` 以及
`Artifact` 識別碼 `webcache`。`-Z` 會轉換所有時間：名稱以 `Time` 結尾的欄位、`LastModified` 與
`Expires`，但以一天或一週為單位的歷程記錄容器中屬於本機時間的 `ModifiedTime` 除外。

## 延伸資訊 { #further-details }

- [Microsoft — 關於 WinINet](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE 資料庫檔案格式](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — MSIE WebCache 剖析器](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
