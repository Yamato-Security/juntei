# DFIR 詞彙表 { #dfir-glossary }

| 術語 | 白話說明 |
|---|---|
| Allocated（已配置） | 依據該跡證，記錄／物件目前正在使用中。 |
| Alternate Data Stream (ADS)（替代資料流） | 除了檔案一般的未命名內容之外，附加在檔案上的具名 NTFS 資料，例如 `Zone.Identifier`。 |
| Artifact（跡證） | 由作業系統／應用程式產生、可用來回答鑑識問題的持久性資料。 |
| Carving（擷取） | 不依賴記錄原本所屬的容器／索引，而是依其結構／特徵碼來尋找記錄。 |
| Control set（控制集） | SYSTEM 設定的一個版本（`ControlSet001` 等）；`Select\Current` 指出目前作用中的版本。 |
| Corroboration（佐證） | 支持同一結論的獨立證據。 |
| Dirty database/hive（dirty 資料庫／登錄區） | 收集時主要檔案尚未完全同步；所需的狀態可能存在於交易記錄中。 |
| Droid | 類似 GUID 的分散式連結追蹤識別碼，用來在檔案／磁碟區移動後持續追蹤。 |
| ESE / JET Blue | Microsoft 的 Extensible Storage Engine 資料庫格式，SRUM 與其他 Windows 資料皆使用此格式。 |
| FILETIME | 自 1601-01-01 UTC 起以 100 奈秒為單位的間隔數；只有在其結構描述如此定義時，數值才具有時間意義。 |
| File reference（檔案參照） | NTFS 項目編號加上序號，用以識別 MFT 記錄的某一個世代。 |
| Hive（登錄區） | 包含 Windows 登錄樹狀結構的二進位檔案。 |
| INDX record（INDX 記錄） | NTFS 目錄索引（`$I30`）中列出該目錄所含檔案的區塊（通常為 4 KiB）；其 slack 可能保留已刪除或已重新命名之檔案名稱的可能證據。 |
| LNK | Windows Shell Link／捷徑，包含目標與大量中繼資料。 |
| MRU | Most Recently Used（最近使用）順序；通常是相對順序，而非每個項目都有時間。 |
| MFT | NTFS 主檔案表（Master File Table），包含檔案／目錄記錄與屬性。 |
| Prefetch hash（Prefetch 雜湊） | Windows 用於 `.pf` 命名的識別碼；並非以密碼學方式計算的內容雜湊。 |
| Registry key last-write（登錄機碼最後寫入時間） | 機碼變更的時間；Windows 通常不會指出是哪個值發生變更。 |
| Resident data（常駐資料） | 直接儲存在 MFT 屬性中的檔案／資料流位元組，而非儲存在外部資料執行區段（data run）中。 |
| RID | 相對識別碼（Relative Identifier），即 Windows SID 授權單位／網域之下最後一段的帳戶／群組成分。 |
| SDDL | 安全性描述元定義語言（Security Descriptor Definition Language）：以文字表示的安全性描述元，例如 `O:S-1-5-18D:(A;;0x001f01ff;;;S-1-5-32-544)`。 |
| Security descriptor（安全性描述元） | 檔案的擁有者、主要群組、DACL（誰可以做什麼）與 SACL（稽核與完整性標籤）。NTFS 會將每個不同的描述元以安全性 ID 為鍵，僅在 `$Secure:$SDS` 中保存一次。 |
| SID | Windows 安全性識別碼（Security Identifier），用於帳戶、服務、群組或安全性主體。 |
| Slack | 已配置結構內部／之後未使用的位元組，可能保留較舊的資料。 |
| SRUM | System Resource Usage Monitor，記錄應用程式／使用者資源使用情形的資料庫。 |
| Timestomping（時間戳記竄改） | 為誤導調查而刻意修改時間戳記；啟發式判斷並非證明。 |
| Transaction log（交易記錄） | 用來將登錄檔或資料庫復原至一致狀態的待處理／已認可變更。 |
| USN | 更新序號（Update Sequence Number），用於 NTFS 變更日誌記錄與相關中繼資料。 |
| UTC | 世界協調時間（Coordinated Universal Time）；跨系統時間軸建議採用的共通時間基準。 |
| Volume serial number（磁碟區序號） | NTFS 在格式化磁碟區時賦予的編號（`$Boot`）；LNK、Jump List 與 Prefetch 檔案所記錄的檔案所在磁碟區，即為其低 32 位元。 |
