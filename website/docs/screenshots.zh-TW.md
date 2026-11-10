# 螢幕擷圖 { #screenshots }

一覽 Juntei 的命令列、自動解析，以及經過整理的調查結果。
點擊螢幕擷圖即可檢視完整大小。

## 1. 啟動畫面與命令 { #1-startup-and-commands }

<figure markdown>

[![Juntei 啟動畫面，顯示 CODE BLUE 版本與可用的命令](assets/screenshots/01-Startup.png)](assets/screenshots/01-Startup.png)

<figcaption>Juntei 的啟動畫面列出了自動解析、各來源專用的剖析器，以及 Persistence Hunter。</figcaption>

</figure>

## 2. 不到 20 秒完成自動解析 { #2-automatic-parsing-in-under-20-seconds }

在 **MacBook Pro M4** 上，下圖所示的執行剖析了來自**六個系統**、
**8.7 GiB 的 Windows 收集資料**，並**在不到 20 秒內儲存了 4.4 GiB 的輸出**。
[auto-parse](commands/auto-parse.md) 命令使用 16 個平行處理程序執行了 240 個剖析器工作，
產生了 482 個結果檔案。其中 **210 個工作成功、30 個因未收集到
相關跡證而略過，失敗為零**。

<figure markdown>

[![Juntei auto-parse 在不到 20 秒內處理完六個系統、8.7 GiB 的輸入並產生 4.4 GiB 的輸出](assets/screenshots/02-AutoParse.png)](assets/screenshots/02-AutoParse.png)

<figcaption>自動探索主機、平行剖析，以及顯示輸入大小、輸出大小、工作結果與經過時間的完成摘要。</figcaption>

</figure>

## 3. 為調查而整理的結果 { #3-results-organized-for-investigation }

<figure markdown>

[![單一系統的輸出資料夾依 DFIR 類別分組，含帳戶 CSV 及其剖析器記錄檔](assets/screenshots/03-OutputFolders.png)](assets/screenshots/03-OutputFolders.png)

<figcaption>每台電腦的結果依 DFIR 類別分組。跡證 CSV 與其對應的 <code>-logs.txt</code> 檔案放在一起，讓您能輕鬆找到記錄與剖析細節。</figcaption>

</figure>
