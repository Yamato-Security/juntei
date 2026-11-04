# 跨跡證調查流程 { #cross-artifact-workflows }

任何跡證都不應單獨支撐重要的結論。這些調查流程說明如何將一條線索轉化為站得住腳的證據鏈。

## 可疑執行檔的執行 { #suspicious-executable-execution }

1. **Prefetch：**確立所保留的執行時間、路徑／雜湊脈絡、執行次數與已載入的檔案。
2. **`$MFT`：**找出路徑、大小、時間戳記、ADS、刪除狀態與 MFT 參照。
3. **USN Journal：**重建建立、寫入、重新命名與刪除的過程。
4. **Amcache：**補充 SHA-1、PE 中繼資料、版本與歷史路徑。
5. **ShimCache：**補充相容性快取的路徑／順序，以及支援版本中的執行旗標。
6. **SRUM：**尋找同一執行檔／使用者的資源／網路使用情形。
7. **Defender：**檢查隔離時間、原始路徑、雜湊值與下載來源。
8. **事件記錄／EDR：**識別父處理程序、命令列、帳戶、網路與安全性脈絡。

結論應區分檔案存在、執行，以及歸屬於特定人員或入侵事件。

## 勒索軟體的檔案活動 { #ransomware-file-activity }

1. 在 `$MFT` 中識別加密後的副檔名與受害路徑。
2. 使用 USN 原因與順序重建大量的寫入／重新命名／刪除。
3. 使用 Prefetch 識別可能的加密程式及其執行時間。
4. 使用 SRUM 找出候選處理程序的 CPU／磁碟／網路使用量高峰。
5. 檢查服務與 TaskCache，確認部署／持續存取方式。
6. 檢查 Defender 隔離資料與事件記錄，確認偵測／修復情形。

即使沒有任何執行跡證殘存，MFT/USN 仍可證明對檔案系統造成的影響。若沒有處理程序遙測資料或關聯分析，
它們通常無法識別負責的處理程序。

## 惡意文件／捷徑的初始存取 { #malicious-document-shortcut-initial-access }

1. RecentDocs 與 Jump Lists 可識別使用者／應用程式的互動與 MRU 脈絡。
2. 解析對應的「最近使用的項目」資料夾 LNK，取得目標、磁碟區／共用資料夾、MFT 參照與參數。
3. 使用 `$MFT` 常駐 ADS 或 Defender 內容中的 Zone.Identifier，取得來源 URL／區域。
4. 使用 Prefetch／事件記錄確認文件應用程式及其子處理程序。
5. 使用 USN/MFT 追蹤解出之承載資料的建立與刪除。

## 服務持續存取 { #service-persistence }

1. 解碼服務名稱、`ImagePath`、`ServiceDll`、帳戶、啟動類型與失敗動作。
2. 解析環境變數，以及加引號／未加引號路徑的行為。
3. 在 `$MFT` 與 USN 歷程中找出該執行檔／DLL。
4. 檢查 Prefetch 與事件記錄，確認服務／處理程序是否實際啟動。
5. 比較非目前的控制集與復原的登錄記錄，以了解較早的設定。

## 排程工作持續存取 { #scheduled-task-persistence }

1. 解碼 TaskCache 的動作、主體／執行脈絡、註冊／執行期中繼資料，以及隱藏狀態。
2. 與 `C:\Windows\System32\Tasks` 中的 XML 進行比較。
3. 將工作名稱／GUID 與 Security 及 Task Scheduler Operational 事件進行關聯。
4. 找出執行檔／指令碼並計算雜湊值；檢視 Prefetch 與檔案系統歷程。
5. 若工作已被移除，請復原已刪除的 TaskCache 記錄。

## 疑似資料外洩 { #suspected-exfiltration }

1. 使用 SRUM 識別依應用程式／使用者／設定檔區分的異常傳送位元組數，以及量測期間。
2. 以 Prefetch、Amcache 與 `$MFT` 確認執行檔的身分。
3. 透過 Jump Lists、LNK、RecentDocs 與事件／EDR 資料，識別被存取／開啟的檔案。
4. 使用 Proxy、防火牆、DNS、封包或 EDR 遙測資料確認目的地／通訊協定。

SRUM 的資料量可以支持資料外洩的假設，但其本身無法識別所傳輸的內容，也無法證明惡意意圖。

## 值得保留的關聯鍵 { #correlation-keys-worth-preserving }

| 鍵 | 跡證 |
|---|---|
| 完整路徑／檔案名稱 | 幾乎所有跡證 |
| SHA-1 / SHA-256 | Amcache、Defender、復原的檔案 |
| MFT 項目 + 序號 | `$MFT`、Prefetch、LNK、Jump Lists、USN |
| USN | Amcache、`$MFT`、USN Journal |
| 磁碟區序號／droid | Prefetch、LNK、Jump Lists、MFT 物件 ID |
| SID／使用者名稱 | SRUM、SAM、登錄、事件記錄 |
| 工作 GUID／工作名稱 | TaskCache、工作 XML、事件記錄 |
| 服務名稱 | 服務、事件記錄、處理程序／驅動程式資料 |
| Defender EntryId / ScanId | 隔離資料、Defender 歷程、事件記錄 |
