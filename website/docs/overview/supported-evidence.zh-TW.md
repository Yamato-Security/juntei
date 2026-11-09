# 支援的證據類型 { #supported-evidence }

| 證據來源 | Juntei 命令 | 最適合的用途 | 重要限制 |
|---|---|---|---|
| Amcache | `parse-registry amcache` | 檔案清單、路徑、SHA-1、PE 中繼資料、裝置 | 存在不等於執行；發行者並非簽章驗證 |
| NTFS `$MFT` | `parse-ntfs mft` | 檔案、已刪除的記錄、路徑、時間戳記、ADS | 被重複使用的記錄與時間戳記竄改可能造成誤導 |
| NTFS `$LogFile` | `parse-ntfs logfile` | redo／undo 中繼資料、舊名稱與時間戳記、對 USN 日誌的寫入 | 循環保留時間短；中繼資料時間並非交易時間；不進行重播 |
| Prefetch | `parse-file prefetch` | 程式執行、執行次數、最近的執行時間、載入的檔案 | 只保留有限的執行時間歷程 |
| USN 日誌 `$J` | `parse-ntfs usn-journal` | 建立、刪除、重新命名、寫入、歷史路徑 | 循環日誌；記錄的是變更，而非檔案內容 |
| 目錄索引 `$I30` | `parse-ntfs i30` | 目錄中的檔案；殘留於索引 slack 中、可能已刪除或已重新命名之檔案的名稱、時間與大小 | slack 並不完整，也無法說明項目何時被移除 |
| NTFS 開機磁區 `$Boot` | `parse-ntfs boot` | LNK、Jump Lists 與 Prefetch 所記錄的磁碟區序號；幾何資訊與 `$MFT` 位置；可跨主機比較的開機程式碼雜湊值 | 序號可能被變更或重複；開機程式碼不同只是線索，並非 bootkit 的證明 |
| NTFS 安全性描述元 `$Secure:$SDS` | `parse-ntfs secure-sds` | 依 `$MFT` 的 `SID` 取得檔案的擁有者與 ACL；NULL DACL 與寬鬆的 ACL | 擁有者可被變更；ACL 說明誰可以存取，而非誰實際存取過 |
| VHDX 磁碟映像檔 | `mount`、`auto-parse --vhdx` | 其 NTFS 磁碟區中的檔案，可由 Juntei 或其他工具就地以唯讀方式讀取 | 需要 FUSE（僅限 Linux）；不會顯示替代資料流 |
| E01 磁碟映像檔（`.E01`、`.Ex01`，無論是否分割） | `mount`、`auto-parse --e01` | 其 NTFS 磁碟區中的檔案，可由 Juntei 或其他工具就地以唯讀方式讀取 | 需要 FUSE（僅限 Linux）；不會顯示替代資料流；不會掛載邏輯證據檔（`.L01`） |
| SRUM | `parse-ese srum` | 各應用程式／使用者的網路與資源使用情形 | 通常為彙總資料；無法作為封包或目的地層級的證明 |
| 使用者存取記錄 (UAL) | `parse-ese ual` | 哪些用戶端位址與帳戶在哪幾天使用了 Windows Server 的角色（檔案共用、Active Directory、DHCP、IIS） | 要求以天為單位計數，而非逐筆記錄；位址可能是 NAT 或 Proxy；dirty 的 `Current.mdb` 會缺少最新的變更 |
| Internet Explorer 與舊版 Edge（WebCache） | `parse-ese webcache` | 造訪過的網頁與在檔案總管中開啟的本機檔案、下載的檔案及其來源、Cookie、快取的檔案及其 HTTP 標頭 | 一筆歷程記錄項目是一個 URL 及其最後一次造訪，而非每一次造訪；不會復原已刪除的項目；dirty 的資料庫會缺少最新的變更 |
| 網頁瀏覽器（Chromium 與 Firefox 系列） | `parse-sqlite browser` | 造訪過的網頁及造訪方式、下載的檔案及其來源、Cookie、在網頁表單中輸入的值；復原已刪除的造訪、下載與表單項目 | 一次造訪可能是重新導向、框架或其他程式開啟的網頁；Chromium 的 Cookie 值經過加密；瀏覽器將已刪除的項目清零或執行檢查點後，這些項目即無法復原 |
| LNK 捷徑 | `parse-file lnk` | 開啟過的目標、參數、磁碟區、共用資料夾、來源機器 | 即使目標從未執行，也可以建立捷徑 |
| Jump Lists | `parse-file jump-lists` | 應用程式開啟過的項目、釘選、使用情形與內嵌的 LNK 資料 | 行為與保留方式因應用程式而異 |
| ShimCache | `parse-registry shimcache` | 歷史執行檔路徑與相容性檢查 | 快取可能要到關機時才寫出；單一項目不等於執行 |
| Defender 隔離資料 | `parse-file defender-quarantine` | 偵測、原始路徑、雜湊值、下載來源、復原的位元組 | 偵測名稱是產品的判定；擷取具有危險性 |
| 登錄區 | `parse-registry read` | 設定、持續存取、已刪除／重播的值 | 機碼時間並非值的時間；即時登錄區需要交易記錄 |
| 服務 | `parse-registry services` | 服務／驅動程式的持續存取與失敗動作 | 設定並不能證明服務曾啟動 |
| RecentDocs | `parse-registry recent-docs` | 每位使用者最近開啟的檔案與資料夾 | 只有每份清單中最近的項目能取得站得住腳的開啟時間 |
| Microsoft Office | `parse-registry office` | 各 Office 應用程式開啟過的文件與資料夾及其時間；使用者啟用了巨集的文件；巨集設定與信任的位置 | 啟用巨集並不能證明巨集曾執行；信任記錄的路徑依 Office 寫入的方式儲存 |
| 開啟與另存新檔對話方塊 | `parse-registry open-save-mru` | 透過通用對話方塊開啟或儲存的檔案（依副檔名區分），以及使用這些對話方塊的程式與資料夾 | 只有每份清單中最近的項目附有時間；無法區分開啟與儲存 |
| TaskCache | `parse-registry taskcache` | 排程工作的動作、中繼資料、隱藏的工作 | 若要取得執行期證明，請與工作 XML 及事件記錄交叉比對 |
| 開機設定 (BCD) | `parse-registry bcd` | 開機項目；關閉復原、忽略失敗、測試簽署、關閉完整性檢查、安全模式，以及各項的設定時間 | 存放區常未被收集（EFI 系統磁碟分割、`\Boot`）；設定會被繼承 |
| 登錄自動啟動項目 | `parse-registry persistence` | Run 機碼、Winlogon、IFEO、AppInit/AppCert DLL、LSA 套件、COM 劫持及其他自動啟動位置 | 是設定，而非執行的證明；大多數項目是 Windows 的預設值 |
| 應用程式相容性 | `parse-registry app-compat` | 相容性層、程式相容性助理觀察到執行過的程式、CIT 程式與模組使用情形、自訂 shim 資料庫 | 程式相容性助理項目不帶執行時間；CIT 資料大多來自 Windows 7 與 8 |
| FeatureUsage、RecentApps、釘選項目 | `parse-registry feature-usage` | 使用者切換到、啟動或點擊各應用程式的頻率；最近使用的應用程式及其檔案；工作列釘選項目與「開始」功能表程式 | 計數沒有時間；計算的是視窗切換，而非啟動 |
| ShellBags | `parse-registry shell-bags` | 使用者在檔案總管與對話方塊中開啟過的資料夾（包括卸除式、網路、可攜式裝置、ZIP 及已刪除的資料夾），以完整路徑呈現 | 只有每個機碼中最近的項目能取得站得住腳的開啟時間；項目時間屬於資料夾，而非瀏覽行為 |
| BAM / DAM | `parse-registry bam` | 每位使用者最後一次執行各程式的時間，包括從卸除式與網路裝置執行 | 每個程式與使用者只有一個時間，且僅涵蓋近期活動；呈現的是裝置路徑，而非磁碟機代號 |
| USB 裝置 | `parse-registry usb` | USB 儲存裝置與其他裝置、序號、安裝／連接／移除時間、磁碟機代號、磁碟區名稱與序號、掛載過裝置的使用者 | 只保留最後一次連接與移除；磁碟機代號會被重複使用 |
| MuiCache | `parse-registry muicache` | 與使用者 Shell 設定檔相關聯的快取程式路徑、描述與公司 | 沒有個別程式的時間；設定檔不一定是處理程序的帳戶，且中繼資料可被偽造 |
| UserAssist | `parse-registry user-assist` | 使用者從檔案總管啟動的程式與捷徑，以及執行次數、焦點時間與最後執行時間 | 只計算從檔案總管的啟動；新設定檔會預先植入預設項目 |
| 網路 | `parse-registry networks` | 加入過的網路及其首次／最後連線時間、閘道 MAC、介面與 DHCP 租約、共用資料夾、連接埠 Proxy、路由 | 設定檔時間為當地時間，並以目前的時區轉換；"Network 2" 之類的名稱會重複出現 |
| 安全性設定 | `parse-registry security-settings` | Defender（及其排除項目）、UAC、WDigest／LSA 保護、RDP、防火牆、SMBv1、PowerShell 記錄與 AMSI、事件記錄與證據相關設定，並標示削弱預設值的設定 | 被削弱的設定並非遭入侵的證明；不存在的值（Windows 的預設值）不會被回報 |
| 應用程式歷程 | `parse-registry app-history` | 使用者執行過的 Sysinternals 工具、壓縮檔與資料夾歷程（WinRAR、7-Zip、WinZip）、OneDrive 帳戶、Adobe 最近使用的檔案、TeamViewer 與 CCleaner 的值 | 大多數清單項目沒有時間；只有每份清單中最近的項目附有時間 |
| MRU 清單 | `parse-registry mru` | 「執行」對話方塊的命令、在檔案總管輸入的路徑、檔案總管的搜尋、在 Internet Explorer 輸入的網址（附時間）、小畫家與 WordPad 的檔案、XP 的搜尋字詞 | 除了輸入的網址之外，只有每份清單中最近的項目附有時間；輸入過不等於執行過 |
| 遠端存取（使用者端） | `parse-registry remote-access` | 使用者以遠端桌面、PuTTY 與 WinSCP 連線過的電腦、已接受的 SSH 主機金鑰、對應的磁碟機與共用資料夾 | 儲存或輸入過的目標不等於連線成功；清單機碼的所有項目只共用一個時間 |
| 已安裝程式 | `parse-registry installed-programs` | 已安裝的程式（全機與個別使用者）、Windows Installer 產品、App Paths、WSL 發行版與封裝應用程式 | 安裝日期由安裝程式寫入；乾淨解除安裝的程式不會留下項目 |
| SAM | `parse-registry sam` | 本機帳戶、旗標、RID、登入／密碼中繼資料 | 僅限本機帳戶；最後登入值並非完整的登入歷程 |
| LSA 機密資訊 | `parse-registry lsa-secrets` | LSA 持有哪些機密資訊（自動登入、服務帳戶、電腦帳戶）及其設定時間；使用中的快取網域登入 | 只有名稱與時間，不解密任何內容；即使沒有自動登入，也會存在 `DefaultPassword` |
| 稽核原則 | `parse-registry auditpol` | 系統被設定為記錄哪些安全性事件，從而得知哪些事件的缺席不具意義 | 這是登錄區最後一次寫入時的原則；變更會由事件 4719 記錄 |
| 系統資訊 | `parse-registry system-info` | 電腦與網域名稱、Windows 版本與安裝日期、時區、上次關機、上次登入的使用者、設定檔、SID | 這是收集當時的設定；`InstallDate` 會隨功能更新而變動，而 Windows 11 會自稱為 Windows 10 |

## 收集資料的類型 { #collection-types }

Juntei 可以使用 `-f` 接受單一跡證，或使用 `-d` 以遞迴方式搜尋收集資料。
目錄模式是為常見的分類（triage）配置而設計，其中每台主機都包含
`C/Windows/System32/config/SYSTEM` 或 `C/Users/alice/NTUSER.DAT` 之類的路徑。

在分析複製而來的登錄或 ESE 資料庫之前，請先參閱[收集證據](../getting-started/collection.md)：
相關的交易記錄對於重建 Windows 當時所見的狀態可能至關重要。
