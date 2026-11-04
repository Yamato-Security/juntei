# 跡證指南 { #artifact-guide }

Windows 會在檔案系統、登錄、相容性資料庫、使用者介面歷程、資源用量統計與資安產品中
保留彼此重疊的痕跡。Juntei 會解析這些來源，但每個來源代表什麼意義，仍須由調查人員判讀。

## 依問題選擇跡證 { #choose-an-artifact-by-question }

| 調查問題 | 從這裡開始 | 佐證來源 |
|---|---|---|
| 這個執行檔是否曾經執行？ | [Prefetch](prefetch.md) | [ShimCache](shimcache.md)、[SRUM](srum.md)、事件記錄 |
| 這個檔案是否曾經存在？ | [Amcache](amcache.md)、[$MFT](mft.md) | [USN Journal](usn-journal.md)、[$I30](i30.md)、ShimCache |
| 檔案是何時被建立、重新命名或刪除的？ | [USN Journal](usn-journal.md) | [$MFT](mft.md)、[$I30](i30.md)、Defender 隔離資料 |
| 中繼資料變更之前存在的是什麼名稱或時間戳記？ | [$LogFile](logfile.md) | [$MFT](mft.md)、[USN Journal](usn-journal.md)、[$I30](i30.md) |
| 檔案的擁有者是誰，誰可以變更它？ | [$Secure:$SDS](secure-sds.md) 搭配 [$MFT](mft.md) | [SAM](sam.md)、Security 事件記錄 |
| 捷徑或 Prefetch 檔案的目標位於哪個磁碟區？ | [$Boot](boot.md) | [LNK](lnk.md)、[Prefetch](prefetch.md)、[USB 裝置](usb.md) |
| 使用者開啟了什麼？ | [Jump Lists](jump-lists.md)、[LNK](lnk.md)、[RecentDocs](recentdocs.md) | SRUM、Prefetch |
| 是否有程式傳輸了大量資料？ | [SRUM](srum.md) | 防火牆、Proxy、DNS、封包或 EDR 遙測資料 |
| 是否設定了持續存取機制？ | [服務](services.md)、[TaskCache](taskcache.md)、[登錄](registry.md) | 事件記錄、Prefetch、`$MFT` |
| Defender 移除了什麼？ | [Defender 隔離資料](defender-quarantine.md) | 事件記錄、`$MFT`、USN Journal |
| 是否新增或變更了本機帳戶？ | [SAM](sam.md) | Security 事件記錄與登錄交易歷程 |

## 先讀警告，再看欄位 { #read-warnings-before-fields }

每個跡證頁面都依循相同的順序：

1. **這是什麼**，以及它能回答哪些問題。
2. **在哪裡找到它**，以及收集方式對它的影響。
3. **它能證明什麼，以及不能證明什麼。**
4. **如何使用 Juntei 解析它。**
5. **如何判讀每個輸出欄位。**
6. **如何將它與獨立證據進行關聯分析。**
7. **進一步資訊**，供深入的底層研究之用。

!!! danger "不要將剖析器的發現等同於惡意行為"
    Juntei 的發現項目會指出值得檢視的模式，例如位於 `System32` 之外的系統二進位檔名稱。
    安裝程式、系統管理員、相容性層與軟體更新都可能產生不尋常的記錄。請以脈絡與獨立證據加以驗證。
