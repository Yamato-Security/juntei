---
hide:
  - navigation
  - toc
---

<div class="jt-hero" markdown>

![Juntei，數位鑑識女神](assets/juntei-goddess.png){ .jt-hero-art width="1448" height="1086" }

<p class="tagline">
<strong>Juntei</strong> 是 Yamato Security 推出的跨平台剖析器，用於處理事件記錄<em>以外</em>的
Windows 鑑識跡證。它會將收集到的證據轉換為格式一致的 CSV、JSON 與 JSONL
記錄，同時保留調查人員避免得出錯誤結論所需的脈絡。
</p>

<div class="jt-cta" markdown>
[從這裡開始 :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[認識鑑識跡證 :material-archive-search:](artifacts/index.md){ .md-button }
[命令參考 :material-console:](commands/index.md){ .md-button }
[GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/juntei){ .md-button }
</div>

</div>

## 從原始跡證到站得住腳的結論 { #from-raw-artifact-to-defensible-conclusion }

Juntei 能告訴您跡證記錄了什麼；本網站則說明該記錄**能證明什麼、不能證明
什麼**。即使您從未進行過 Windows DFIR，也請從跡證頁面開始。
每份指南都涵蓋其用途、在磁碟上的位置、保留與收集上的注意事項、剖析命令、
輸出欄位、判讀方式、交叉比對，以及延伸閱讀。

<div class="grid cards" markdown>

-   :material-file-tree:{ .lg .middle } __檔案系統歷程__

    ---

    使用 `$MFT` 與 USN 日誌重建檔案的存在、名稱、移動、刪除、
    中繼資料變更與替代資料流。

-   :material-play-circle-outline:{ .lg .middle } __程式證據__

    ---

    Prefetch 能有力地佐證執行。Amcache 與 ShimCache 可補充清單、路徑、雜湊值、
    相容性與歷史脈絡，但需要謹慎判讀。

-   :material-account-clock:{ .lg .middle } __使用者活動__

    ---

    LNK 檔案、Jump Lists、RecentDocs 與 SRUM 可揭露開啟過的項目、應用程式活動、
    卸除式或網路位置、資源使用情形，以及使用者的歷史脈絡。

-   :material-shield-bug:{ .lg .middle } __資安證據__

    ---

    Defender 隔離資料可保留威脅標籤、原始路徑、下載來源、雜湊值、
    擁有者，甚至能安全地復原被隔離的位元組，供在隔離環境中分析。

-   :material-database-search:{ .lg .middle } __登錄狀態__

    ---

    讀取已配置與可復原的登錄資料、重播交易記錄，並檢視服務、
    排程工作、本機帳戶與最近使用文件的狀態。

-   :material-link-variant:{ .lg .middle } __交叉比對，而非猜測__

    ---

    沒有任何單一跡證能說明全貌。這些指南說明如何結合獨立的來源，
    建立有證據支持的時間軸。

</div>

!!! warning "鑑識輸出是證據，而非判定"
    看似可疑的路徑只是線索。時間戳記可能描述的是檔案的中繼資料，而非
    執行。缺少記錄並不能證明某個動作沒有發生。請保留來源證據、
    記錄工具版本與命令，並佐證重要的結論。
