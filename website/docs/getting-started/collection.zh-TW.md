# 收集證據 { #collecting-evidence }

Juntei 剖析已從 Windows 收集的檔案。請使用可信任的擷取工具，並盡可能保留原始中繼資料。

[auto-parse --vhdx](../commands/auto-parse.md#vhdx-disk-images): Linux / macOS / Windows.

[auto-parse --e01](../commands/auto-parse.md#e01-and-ex01-disk-images): Linux / macOS / Windows.

## 常見位置 { #common-locations }

| 跡證 | 一般位置 |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | NTFS 磁碟區中繼資料檔案 `C:\$MFT` |
| `$LogFile` | NTFS 磁碟區中繼資料檔案 `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| USN Journal | NTFS 中繼資料資料流 `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| 使用者存取記錄（Windows Server） | `C:\Windows\System32\LogFiles\Sum\` |
| Internet Explorer 與舊版 Edge（WebCache） | `%LOCALAPPDATA%\Microsoft\Windows\WebCache\` |
| Outlook | `%LOCALAPPDATA%\Microsoft\Outlook\*.ost`、`%USERPROFILE%\Documents\Outlook Files\*.pst`（Outlook 2010 之前為 `%LOCALAPPDATA%\Microsoft\Outlook\*.pst`）、`%APPDATA%\Microsoft\Templates\*.oft`，以及儲存在任何位置的 `.msg` 檔案 |
| 網頁瀏覽器 | `%LOCALAPPDATA%\Google\Chrome\User Data\`、`%LOCALAPPDATA%\Microsoft\Edge\User Data\`、`%APPDATA%\Mozilla\Firefox\Profiles\` 以及其他瀏覽器的設定檔資料夾 |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` 以及其他使用者／系統位置 |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` 與 `CustomDestinations` |
| RDP 點陣圖快取 | `%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\`（Windows XP 及更早版本為 `%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\`） |
| 資源回收筒 | 每個磁碟區上的 `$Recycle.Bin\<SID>\$I*` 與 `$R*`（Windows XP 及更早版本為 `RECYCLER\<SID>\INFO2` 與 `D*`） |
| ShimCache | `SYSTEM` 登錄區：`...\Session Manager\AppCompatCache\AppCompatCache` |
| Defender 隔離資料 | `C:\ProgramData\Microsoft\Windows Defender\Quarantine` |
| 電腦登錄檔 | `C:\Windows\System32\config\SYSTEM`、`SOFTWARE`、`SAM`、`SECURITY` |
| 使用者登錄檔 | `%USERPROFILE%\NTUSER.DAT`、`%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` |

## 必須一併收集的檔案 { #files-that-must-travel-together }

- 收集登錄區時，請一併收集相鄰的 `.LOG`、`.LOG1` 與 `.LOG2` 檔案。當登錄區處於 dirty 狀態時，
  Juntei 可在登錄檔命令與 ShimCache 中重播這些記錄檔。
- 請以一致的方式收集 SRUM 的 `SRUDB.dat`。Juntei 目前會回報 dirty 的 ESE 資料庫，但
  不會重播 `SRU*.log`；僅存在於這些記錄檔中的記錄將無法取得。
- 可能的話，請將 `$MFT` 與 `$Extend/$J` 一併收集。USN 記錄儲存的是父檔案參照，
  而非完整路徑；MFT 可改善路徑解析。
- 請在盡可能接近的時間點，將 `$LogFile` 與同一磁碟區的 `$MFT` 一併收集。`$LogFile` 循環覆寫的速度很快；
  較晚收集可能會完全失去先前的 redo/undo 記錄。
- 收集 SRUM 時請一併收集對應的 `SOFTWARE` 登錄區，以解析帳戶與 Wi-Fi 設定檔名稱。
- 請收集完整的使用者存取記錄 `Sum` 目錄：`SystemIdentity.mdb` 提供角色名稱，並列出各年度的
  `{GUID}.mdb` 資料庫。Juntei 同樣不會重播其 `Svc*.log` 交易記錄檔。
- 收集瀏覽器資料庫時，請一併收集其旁的 `-wal` 檔案（`places.sqlite-wal`、
  `cookies.sqlite-wal`）：最新的變更，以及用來復原已刪除項目的舊版頁面，只會保留到瀏覽器執行
  檢查點為止。
- 請收集整個 `WebCache` 資料夾：使用者登入期間，Windows 會持續開啟 `WebCacheV01.dat`，而其最新的
  變更只存在於 `V01*.log` 交易記錄檔中，Juntei 不會重播這些記錄檔。
- 請在 Outlook 關閉時收集其 OST 與 PST 檔案，或從磁碟映像或磁碟區陰影複製中收集：Outlook 執行
  期間會持續開啟這些檔案。也請在預設資料夾之外尋找 PST：使用者會將封存檔移到其他資料夾、磁碟機與
  檔案共用。
- 請收集完整的 Defender `Quarantine` 樹狀結構，以便比對 Entries、ResourceData 與 Resources。

## 執行中系統的注意事項 { #live-system-cautions }

開啟中的檔案可能在複製期間發生變化。登錄區與 ESE 資料庫在內部可能處於 dirty 狀態。
ShimCache 可能要到關機時才會將記憶體中最新的項目寫入。相較於一般的檔案複製，使用磁碟區快照或原始存取的
鑑識收集工具較為理想。

## 安全性 { #safety }

證據中可能含有惡意程式、漏洞利用樣本、惡意 LNK 檔案以及試算表公式文字。
請在隔離的系統上進行分析，不要執行復原出來的資料流，且僅在確定要處理危險內容時才使用 `--extract`。
請另行保存未經改動的原始來源。
