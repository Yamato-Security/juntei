# 收集證據 { #collecting-evidence }

Juntei 剖析的是已從 Windows 收集而來的檔案。請使用可信任的取證工具，並盡可能保留原始中繼資料。

## 常見位置 { #common-locations }

| 跡證 | 一般位置 |
|---|---|
| Amcache | `C:\Windows\AppCompat\Programs\Amcache.hve` |
| `$MFT` | NTFS 磁碟區中繼資料檔案 `C:\$MFT` |
| `$LogFile` | NTFS 磁碟區中繼資料檔案 `C:\$LogFile` |
| Prefetch | `C:\Windows\Prefetch\*.pf` |
| USN Journal | NTFS 中繼資料資料流 `C:\$Extend\$UsnJrnl:$J` |
| SRUM | `C:\Windows\System32\sru\SRUDB.dat` |
| LNK / Recent | `%APPDATA%\Microsoft\Windows\Recent\*.lnk` 以及其他使用者／系統位置 |
| Jump Lists | `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` 與 `CustomDestinations` |
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
- 請收集完整的 Defender `Quarantine` 樹狀結構，以便比對 Entries、ResourceData 與 Resources。

## 執行中系統的注意事項 { #live-system-cautions }

開啟中的檔案可能在複製期間發生變化。登錄區與 ESE 資料庫在內部可能處於 dirty 狀態。
ShimCache 可能要到關機時才會將記憶體中最新的項目寫入。相較於一般的檔案複製，使用磁碟區快照或原始存取的
鑑識收集工具較為理想。

## 安全性 { #safety }

證據中可能含有惡意程式、漏洞利用樣本、惡意 LNK 檔案以及試算表公式文字。
請在隔離的系統上進行分析，不要執行復原出來的資料流，且僅在確定要處理危險內容時才使用 `--extract`。
請另行保存未經改動的原始來源。
