# 剖析器驗證 { #parser-validation }

Juntei 所維護的剖析器 crate 均以公開證據與獨立實作進行測試。這些比較有助於
偵測迴歸問題與判讀上的差異；但並不會使任何剖析器變得萬無一失。請保留來源證據、檢查每筆記錄的問題或
損毀欄位，並以其他跡證或實作驗證具有重大影響的發現。

以下說明保留了先前記載於 Juntei 主要 README 中的驗證資訊。目前的測試資料與實作細節，
請參閱所連結的剖析器儲存庫。

## Amcache { #amcache }

Juntei 使用 [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)，這是
Security Ronin 的
[amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) 之 Apache-2.0 分支。其
[輸出文件](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
定義了每一個執行檔、裝置、發現與摘要欄位。

## 主檔案表 { #master-file-table }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) 的 mft crate 是 Omer
Ben-Amram 的 [mft](https://github.com/omerbenamram/mft) crate 之維護分支。在八個 `$MFT` 檔案中——包括該 crate 的
範例、NIST 駭客入侵案例，以及六個 DFIR Artifact Museum 檔案——共 1,048,733 列 CSV 的資料列與欄位
依序與參考剖析器一致。比較涵蓋預設輸出、`--all-times`、
`--short-names` 與 `--cutoff`；時間戳記的呈現方式則是刻意保留的差異。

## NTFS 交易記錄 ($LogFile) { #ntfs-transaction-log-logfile }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) 的 logfile crate 以
[Sootmark ntfslog](https://github.com/Sootmark/ntfslog) 為基礎進行維護。其驗證涵蓋六份完整的 DFIR
Artifact Museum 記錄、公開的 1.1 版與 2.0 版測試資料、解碼後的 redo/undo 中繼資料，以及與獨立剖析之 NTFS 資料
相互比較的日誌寫入。詳細的涵蓋範圍與已知差異，請參閱該剖析器儲存庫。

Juntei 另外加入了 CLI 測試：在公開的 go-ntfs 測試磁碟區（2,129 筆記錄）上比較每個 CSV 欄位與 JSON 及 JSONL，
驗證分開的 redo/undo 資料與原始酬載長度、測試損毀的記錄、
拒絕覆寫證據，並防止目前的 MFT 路徑追蹤到被重複使用的記錄編號。
迴歸測試案例會區分受影響的 MFT 記錄與其寫入時所經由的開啟中屬性、
檢查扁平的目錄項目／資料屬性中繼資料，並防止跨磁碟區的 MFT 配對。
多磁碟區的 `auto-parse` 測試則涵蓋分開的 CSV 檔案，以及帶有出處資訊的單一 JSONL 檔案。

此命令也在六份私人收集的 Windows 記錄上進行過檢查：所匯出的全部 708,465 筆 CSV
記錄均與 JSONL 逐欄一致，且各主機的 `auto-parse` 匯出也保留了相同的記錄
筆數。這些來源證據並未隨儲存庫散布。這些檢查驗證的是輸出的
一致性，而非每一筆記錄下來的交易都代表一個已完成的使用者動作。

## Prefetch { #prefetch }

[ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) 是
ForensicRS 的 [frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) 之維護分支。其解碼輸出曾與
libyal 獨立的 Prefetch 實作 [libscca](https://github.com/libyal/libscca)
進行比較。

## USN 日誌 { #usn-journal }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) 的 usn crate 是
[Sootmark usn](https://github.com/Sootmark/usn) 的維護分支。在六份真實的日誌上，只要獨立剖析器能解析出路徑，
Juntei 考量時間因素的父路徑都與其一致。Juntei
也解析出了比較對象未能解析的 4,321 筆記錄。其擷取器在 880 MB 的 `$MFT` 資料上測試時，
沒有產生任何錯誤的記錄。

## 目錄索引 ($I30) { #directory-indexes-i30 }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) mft crate 中的 `i30` 模組會讀取 INDX
記錄及其 slack。在以 ntfs-3g 建立之磁碟區的兩個 `$I30` 資料流（Juntei 的
`test_files/i30/`）以及三個由 Windows 寫入的公開 `$I30` 範例上，其使用中與位於
slack 中的項目，均與 Maxim Suhanov 的 [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) 一致，名稱與時間
也相同。當 dfir_ntfs 回報超過 32 位元的父參照（父參照被覆寫的位元組）時，
Juntei 會將父項目保留為空白。

## 開機磁區 ($Boot) { #boot-sector-boot }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) 的 boot crate 是 Yamato
Security 自行開發的。在 DFIR Artifact Museum 的五個公開 `$Boot` 檔案（Windows 10、11 與
Server 2012 R2）上，每個欄位都與參考剖析器一致。在以 `mkntfs` 建立、磁區大小為 512、
1,024 與 4,096 位元組，叢集大小為 4 KiB 至 64 KiB 的磁碟區上，大小、`$MFT` 與 `$MFTMirr`
叢集以及序號均與 The Sleuth Kit 的 `fsstat` 一致。Museum 的 Windows 11 `$Boot`
具有與同一主機之 LNK 檔案所記錄相同的磁碟區序號，且這五個檔案具有
相同的開機程式碼。

## 安全性描述元 ($Secure:$SDS) { #security-descriptors-securesds }

[ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) 的 secure crate 是 Yamato
Security 自行開發的。在 Museum 的五個 `$SDS` 資料流（7,808 個描述元、60,781 個 ACE）中，
安全性 ID、雜湊、位移、擁有者、群組、控制旗標與 ACE 數量均與參考
剖析器一致，且每個 ACE 的類型、旗標、遮罩、SID 與物件類型均與 `dissect.ntfs` 一致。每個
記錄的雜湊都屬於其描述元，每份鏡像副本都與其項目一致，且每個描述元都有
SDDL，包括其 3,384 個回呼 ACE 的條件。

## SRUM { #srum }

[ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) 以 Sootmark 的
[ese](https://github.com/Sootmark/ese) 與 [srum](https://github.com/Sootmark/srum) crate 為基礎。在
11 個真實資料庫中，所輸出之 45,255 筆項目的每個欄位，都與對 `dissect.esedb` 所提供之值
進行的獨立解碼結果一致。

## 使用者存取記錄 { #user-access-logging }

`parse-ese ual` 使用相同的 ESE 讀取器讀取 UAL 資料庫。在 16 個真實資料庫（308 筆資料列，
其中 145 筆為用戶端資料列）中，每個值都與對 `dissect.esedb` 所提供之值進行的獨立解碼結果一致。
在每一筆用戶端資料列中，有計數的日子都從 `InsertDate` 的 UTC 日期開始、到 `LastAccess` 的
UTC 日期為止，且其總和等於 `TotalAccesses`。

## 網頁瀏覽器 { #web-browsers }

[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) 是 Sootmark 的 browser、
sqlite 與 plist crate 之維護分支，是在相同的資料庫上與另一個 Rust 函式庫以及 Plaso、
Velociraptor（SQLiteHunter）與 Hindsight 比較後選定的。`parse-sqlite browser` 曾在 38 個公開資料庫上，
與以 `sqlite3` 函式庫讀取各資料庫連同其 `-wal` 檔案之複本的結果進行比較：這些資料庫包括 Plaso 的
測試檔案（Chrome 8 至 68、Firefox 2 至 118）、由 Chromium 150 與 Firefox 140 建立的設定檔，以及
ys-browser-parser 的表單歷程測試資料庫。全部 293 筆造訪、21 筆下載、692 個 Cookie 與 41 筆表單項目
皆一致。已刪除的項目只在確實有項目被刪除之處才被復原：即從 Firefox 140 歷程中刪除的兩個網頁與兩筆
造訪（連同其時間、URL 與標題），以及測試資料庫中已刪除的表單項目。

## Internet Explorer 與舊版 Edge（WebCache） { #internet-explorer-and-legacy-edge-webcache }

`parse-ese webcache` 使用 [ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser)
讀取 `WebCacheV01.dat`。它曾在 Plaso 的兩個公開 WebCache 資料庫上與 `dissect.database` 的 ESE 讀取器
進行比較：一個是 Internet Explorer 11 的資料庫，另一個是處於 dirty 狀態、含有舊版 Edge 瀏覽歷程記錄、
下載與 Cookie 的 Windows 10 資料庫。全部 4,156 筆資料列逐值一致：44 個容器、182 筆歷程記錄項目、
6 筆下載、170 個 Cookie 與 3,754 筆其他項目。比較時另行獨立解碼了下載記錄。資料庫標記為已刪除的
記錄，在兩邊皆予以排除。

## Outlook { #outlook }

`parse-file outlook` 使用 [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
讀取 PST、OST、`.msg` 與 `.oft` 檔案；其 PST 與 OST 讀取器是 Microsoft 的
[outlook-pst-rs](https://github.com/microsoft/outlook-pst-rs) 之分支，而其 `.msg` 讀取器則是依據
Microsoft 的公開規格撰寫而成。

- **PST 與 OST**：66 個公開的 PST 與 OST 檔案（Apache Tika、artemis、Aspose.Email 範例、DFIR
  Artifact Museum、Enron、GroupDocs 範例、java-libpst、libyal 的測試資料、outlook-pst-rs 與
  pst-extractor）曾與 libpff 逐一屬性進行比較：涵蓋每個存放區、資料夾、郵件、收件者與附件的每一個
  屬性，以及每個附件的資料。324,175 個值相同，且沒有任何無法解釋的差異。其餘部分為：只有
  ys-outlook-parser 讀得到的內容（libpff 無法讀取之郵件、收件者與附件，以及一個它無法開啟之存放區的
  15,846 個值；隱藏項目的 8,223 個值）、其資料列標示為不存在、libpff 卻以空白或零呈現的收件者儲存格
  （49,077 個）、依 MAPI 方式計算的資料夾 entry ID 與類型（2,004 個），以及 libpff 以原始資料呈現的
  物件與內嵌郵件（480 個）。其中有兩個檔案，兩邊的讀取器都完全無法讀取。
- **`.msg` 與 `.oft`**：214 個公開檔案，含 2,404 個郵件、收件者與附件以及 41,002 個屬性，曾與
  extract-msg 進行比較：110,114 個值相同，而不同的 791 個值都各有解釋：其中 601 個是 extract-msg 在
  時間上的錯誤，35 個來自它以錯誤字碼頁解碼的 ANSI 檔案，其餘則是它遺漏或額外加入的內容。
- **損毀的輸入**：超過 260,000 個經變異的 `.msg` 檔案與 160,000 個經變異的 PST，以及特製的檔案，
  皆在未發生當機、停滯或記憶體無限制增長的情況下讀取完成。

## LNK 捷徑 { #lnk-shortcuts }

[ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) 是
Sootmark 的 [shell](https://github.com/Sootmark/shell) crate 之維護分支。它曾在 1,118 個公開
捷徑上進行比較。在與參考剖析器共有的 30 個 CSV 欄位中，有 16 個個別值不同：
13 個目標路徑涉及損毀、截斷、預留位置、MTP 或控制台項目；兩份
額外資料區塊清單涉及未知或截斷的區塊；以及一個參考剖析器從不同位元組讀取的
MFT 參照。Juntei 會回報損毀的結構，而不是默默地
將其正規化。

## Jump Lists { #jump-lists }

[ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) 使用 Sootmark 的
複合檔案與 Jump List 相關成果，內嵌的連結則由 `ys-lnk-parser` 解碼。在 77 個公開的
Jump Lists 中，全部 13,057 個共同比較的儲存格——包括精確到 100 ns 的時間戳記——都與
參考剖析器一致。參考剖析器拒絕處理的六個檔案（空白的自訂清單、缺少頁尾，或
並非 Jump List 的檔案），Juntei 則加以解碼或明確地回報。

## 資源回收筒 { #recycle-bin }

`parse-file recycle-bin` 是 Juntei 自有的剖析器。在 [rifiuti2](https://github.com/abelcheung/rifiuti2)
的公開樣本上，rifiuti2 能讀取的每一筆記錄，其索引、刪除時間（精確到秒，與 rifiuti2 的顯示方式相同）、
移除狀態、大小與路徑都一致：共 92 筆記錄，其中 67 筆來自 Windows 95 至 Server 2003 的十一個 INFO2
檔案，25 筆來自三個存放 Windows Vista 與 Windows 10 `$I` 檔案的資料夾。INFO2 的 ANSI 路徑在 rifiuti2
自身測試所使用的字碼頁下皆一致。Juntei 也能讀取 rifiuti2 拒絕處理的內容：混用兩種 `$I` 版本的資料夾、
沒有路徑的 `$I` 標頭，以及遭截斷的記錄，並逐一回報其損毀情形。Plaso 測試資料中的 `$I` 與 INFO2 檔案，
得出的時間、大小、路徑、索引與磁碟機都與 Plaso 測試的預期值相同。在 Digital Corpora 公開的 M57-Jean
磁碟映像檔（Windows XP）上以 `auto-parse --e01` 讀取時，INFO2 記錄與 rifiuti2 一致，而位於其旁的
回收檔案 `Dc1.jpg`，也正是 Juntei 所給出的名稱。

## RDP 點陣圖快取 { #rdp-bitmap-cache }

`parse-file rdp-cache` 是 Juntei 自有的剖析器。在 CODE BLUE 2026 CTF 收集資料的六台主機之一 kofu 上，其三個
`Cache????.bin` 檔案中的 1,911 個圖塊，每一個都與 [bmc-tools](https://github.com/ANSSI-FR/bmc-tools) 產生的
影像逐像素相同。ApoorvCTF 2025「Phantom Connection」題目公開的 `Cache0000.bin` 中的 1,980 個圖塊也是如此，
其中 105 個為 64 × 48 像素。bmc-tools 將每個圖塊都當成 64 像素寬來讀取，因此位於畫面右緣、較窄的圖塊
輸出後會變形；Juntei 則依各圖塊本身的寬度讀取。

目前找不到含有圖塊的公開 `.bmc` 快取：上述樣本與其他 Windows 10 及更新版本的用戶端一樣，只有一個空的
`bcache24.bmc`。因此，`.bmc` 的讀取是以依照該格式建構的檔案來測試：四種色彩深度的未壓縮圖塊、後面接著
較早圖塊尾端的較小圖塊、損毀的槽位，以及 3,000 個隨機產生、以 RLE 壓縮的 16 位元圖塊，涵蓋 MS-RDPBCGR 3.1.9
的每一種指令（order），且沒有任何指令延續超過圖塊第一列的結尾。Juntei 與 bmc-tools 將這 3,000 個圖塊全部
解碼為相同的像素。當某個指令從第一列開始並延續到該列之後時，兩者的結果就會不同：Juntei 遵循規格與
FreeRDP 的解碼器，在整個指令中沿用第一列的規則，而 bmc-tools 則會在指令中途改變規則。

## ShimCache { #shimcache }

[ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) 是
Security Ronin 的
[shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) 之維護分支，並加入了登錄區與
交易記錄的支援。在 20 個公開的 `SYSTEM` 登錄區（24 個控制集與 7,290 個項目）以及 17 個
獨立的 AppCompatCache 值上，每個項目的路徑、時間、執行旗標、旗標與
位置都與參考剖析器的輸出一致。

## Microsoft Defender 隔離資料 { #microsoft-defender-quarantine }

[ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) 是
Sootmark 的
[defender](https://github.com/Sootmark/defender) crate 之維護分支。由剖析器的測試工具以 Defender
格式寫入的隔離資料夾，可由獨立的
`dissect.target` 隔離資料讀取器一致地解碼。

## 從登錄檔衍生的跡證 { #registry-derived-artifacts }

[ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) 會與
[yarp](https://github.com/msuhanov/yarp) 比較已配置的機碼、值、dirty 登錄區的交易重播、
可用 cell 復原、cell slack、登錄區 bin 之外的資料，以及因重播而被移除或取代的記錄。
各跡證特定的比較包括：

- 來自 Windows 7 與 dirty Windows 11 `SYSTEM` 登錄區的服務、值與失敗動作；
- 在九個公開 `NTUSER.DAT` 登錄區上，以 yarp 與 libfwsi 解碼的 RecentDocs；
- 在涵蓋 Windows 7–11、Server 2012 R2 與 Server 2022 的七個公開 `SOFTWARE` 登錄區上，
  以 yarp 獨立解碼的排程工作；以及
- 在七個公開 `SAM` 登錄區上，與 `dissect.target` 比較的 SAM 帳戶結構。

完整的參考工具、格式研究、規格與公開證據來源清單，請參閱[致謝](../acknowledgements.md)。
