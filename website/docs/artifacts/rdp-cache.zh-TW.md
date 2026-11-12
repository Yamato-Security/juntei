# RDP 點陣圖快取 { #rdp-bitmap-cache }

## RDP 點陣圖快取記錄了什麼 { #what-the-rdp-bitmap-cache-records }

遠端桌面用戶端（`mstsc.exe`）在啟用持續性點陣圖快取（預設即為啟用）的情況下連線時，會將遠端畫面的
片段儲存到磁碟上，讓之後的工作階段不必再次接收這些片段。每個片段都是最大 64 × 64 像素的圖塊。這些圖塊
合起來，就是使用者在遠端電腦上所見內容的細小片段：視窗、文字、檔名、命令與桌面。

| 用戶端 | 快取檔案 | 圖塊 |
|---|---|---|
| Windows 7 及更新版本 | `Cache0000.bin`、`Cache0001.bin`、…，每個最大 100 MB | 32 位元，一個接著一個存放 |
| Windows XP、Server 2003 及更早版本 | `bcache2.bmc`（每像素 8 位元）、`bcache22.bmc`（16）、`bcache23.bmc`（24）與 `bcache24.bmc`（32），每個最大 10 MB | 每個 64 × 64 槽位一個；部分經過壓縮 |

Windows 10 及更新版本還會在 `.bin` 檔案旁留下一個空的 `bcache24.bmc`。

<span class="evidence-strength">最佳結論：</span>快取所在設定檔的使用者，曾以遠端桌面用戶端連線到一台
顯示過所快取內容的電腦。快取並不會說明那是哪一台電腦，也不會說明是何時連線。

## 位置與收集 { #locations-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```
`HKCU\Software\Microsoft\Terminal Server Client` 的 `BitmapPersistCacheLocation` 值可以變更快取的
位置。請收集每個設定檔的整個 `Cache` 資料夾，也要從磁碟區陰影複製中收集：快取檔案的較舊副本可能保有
目前檔案已不再保留的圖塊。

## 分析注意事項 { #analyst-cautions }

- 圖塊沒有時間，快取也不會記錄圖塊來自哪些電腦。請透過其他證據將圖塊與 RDP 連線建立關聯：用戶端的
  `Terminal Server Client` 登錄機碼（`parse-registry remote-access`）、`Default.rdp` 與其他 `.rdp`
  檔案、`mstsc.exe` 的 Jump Lists、`$MFT` 中快取檔案的時間，以及遠端電腦的登入事件。
- 一個快取檔案會混雜來自許多工作階段與多台遠端電腦的圖塊。檔案填滿時，用戶端會取代舊的圖塊，因此
  某個圖塊可能是數天甚至數個月前留下的。
- 圖塊只是畫面的片段，而非完整畫面。在畫面上相鄰的圖塊，在檔案中通常也相鄰，但並非總是如此。請使用
  [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher) 重建畫面，或從拼貼圖以肉眼拼湊。
- 大多數圖塊為 64 × 64 像素。位於畫面右緣與下緣的圖塊，會依解析度而較窄或較矮，例如寬 58 或 62 像素，
  或高 28 或 35 像素。Juntei 會依每個圖塊本身的大小讀取。將每個圖塊都當成 64 像素寬來讀取的工具，會讓
  較窄的圖塊顯示為變形。
- 大多數單色圖塊（`SingleColor`）都是背景。請將其篩除，以找出有內容的圖塊。
- 在 `.bmc` 檔案中，8 位元圖塊使用的是工作階段調色盤中的色彩，而快取並未保存該調色盤。Juntei 使用
  Windows 預設調色盤，因此這些圖塊的色彩可能不正確。壓縮的圖塊採用遠端桌面通訊協定的交錯式 RLE
  （MS-RDPBCGR 3.1.9），Juntei 會依規格的定義將其解壓縮。
- 比 64 × 64 槽位小的 `.bmc` 圖塊之後，會留下較早圖塊的尾端。Juntei 會將這些殘留的列另存為獨立的影像
  （`RemnantImage`）。它們屬於較舊的圖塊，而非該槽位中的圖塊。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-file rdp-cache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Cache????.bin and bcache*.bmc files
  -f, --file <FILE>      Parse one cache file, whatever its name

Output:
  -o, --output <FILE>       Save one record per tile
  -t, --output-type <TYPE>  csv, json, or jsonl
  -x, --extract <DIR>       Save each tile as a BMP and a PNG collage of each cache file
      --collage-only        With --extract, save only the collages
      --collage-width <N>   Tiles per collage row (default 64)
```
```bash
juntei parse-file rdp-cache -d evidence/ -o rdp-cache.csv --extract rdp-cache/
juntei parse-file rdp-cache -f Cache0000.bin --extract rdp-cache/ --collage-only
juntei parse-file rdp-cache -d evidence/ -t jsonl | jq 'select(.SingleColor == false)'
```
使用 `-d` 時，Juntei 會讀取名稱為 `Cache` 加上數字與 `.bin`，以及 `bcache` 加上數字與 `.bmc` 的檔案。
開頭不是 `RDP8bmp` 簽章的 `Cache????.bin` 檔案屬於其他程式：會計入摘要並略過。使用 `-f` 時，無論檔名
為何，Juntei 都會讀取該檔案。

### 影像 { #images }

`--extract` 會為每個快取檔案建立一個資料夾 `<DIR>/<user>/<cache file>/`。當兩個快取檔案會共用同一個
資料夾時，例如某個快取檔案與其來自磁碟區陰影複製的副本，第二個資料夾會加上編號（`Cache0001.bin_2`）。
資料夾中包含：

- `<cache file>_<index>.bmp`：每個圖塊，保持其本身的大小。這些名稱與 bmc-tools 使用的名稱相同，因此
  RdpCacheStitcher 可以開啟此資料夾。
- `<cache file>_old_<index>.bmp`：僅限 `.bmc`，即較小圖塊之後殘留的較早圖塊尾端。
- `<cache file>_collage.png`：依檔案中的順序排列所有圖塊，每列 64 個（`--collage-width`），每個圖塊
  從左上角起放在一個 64 × 64 的格子中。索引為 *i* 的圖塊位於第 *i* / 64 列、第 *i* % 64 欄，皆從 0
  起算。格子中的空白部分以洋紅色填滿：較窄圖塊的旁邊、較矮圖塊的下方，以及無法讀取之圖塊的位置。

`--collage-only` 只會儲存拼貼圖。若未指定 `-C`，已存在的影像不會被取代，並會在該圖塊的 `Problems`
中註明。快取檔案本身永遠不會被取代，即使指定 `-C` 也一樣。

## CSV 輸出欄位 { #csv-output-fields }

| 欄位 | 意義 |
|---|---|
| `SourceFile` | 快取檔案。 |
| `User` | 依路徑判斷，快取檔案所在設定檔資料夾的使用者。 |
| `TileIndex` | 圖塊在檔案中的編號，從 0 起算，與其影像名稱中的編號相同。 |
| `Width`, `Height` | 圖塊的大小，以像素為單位。 |
| `SingleColor` | 圖塊的每個像素都是相同色彩時為 `true`。 |
| `TileImage` | 使用 `--extract` 時：圖塊的 BMP 檔案。 |
| `Collage` | 使用 `--extract` 時：快取檔案的拼貼圖。 |
| `RemnantImage` | `.bmc` 且使用 `--extract` 時：殘留在槽位中的較早圖塊尾端。 |
| `Key` | 圖塊在遠端桌面通訊協定中的 64 位元金鑰，以十六進位表示。 |
| `Format`, `FormatVersion` | `bin` 與其簽章之後的版本（Windows 10 及更新版本為 6），或 `bmc`。 |
| `BitsPerPixel` | `.bin` 為 32；`.bmc` 為 8、16、24 或 32。 |
| `Compressed`, `Flags` | `.bmc`：圖塊是否經過壓縮，以及其標頭的旗標。 |
| `Offset` | 圖塊標頭在檔案中的起始位置。 |
| `Problems` | 損毀情形：檔案在圖塊中途結束、任何圖塊都不可能有的圖塊標頭、與圖塊大小不符的資料、無法解壓縮的壓縮資料、無法儲存的影像。 |

JSON 與 JSONL 具有相同的欄位，但數字與布林值帶有型別，圖塊沒有的值為 `null`，`Problems` 則為清單。
不屬於任何單一圖塊的損毀，例如最後一個圖塊之後的資料，會在該次執行的訊息中回報。

## 自動剖析 { #automatic-parsing }

`auto-parse` 會寫入 `LateralMovement/RdpCache.csv`（使用 `-t json` 或 `-t jsonl` 時則為
`RdpCache.json` 或 `RdpCache.jsonl`），並附上 `LateralMovement/RdpCache-logs.txt`，且會將每個快取
檔案的拼貼圖儲存在 `LateralMovement/RdpCache/` 資料夾中。加上 `--extract-rdp-tiles` 即可同時儲存每個
圖塊，單一使用者就可能多達數萬個檔案。從磁碟映像檔剖析時，會複製快取檔案。合併的 `-j` 時間軸包含每一筆
資料列，並帶有 `Artifact` 識別碼 `rdp-cache`。由於圖塊沒有時間，
[`super-timeline`](../commands/super-timeline.md) 不會從中產生任何事件。

## 調查策略 { #investigation-strategy }

1. 開啟拼貼圖並尋找文字：電腦與使用者名稱、檔案路徑、命令、主控台視窗、瀏覽器網址列與對話方塊。
2. 使用 RdpCacheStitcher，從以 `--extract` 儲存的圖塊資料夾重建畫面。
3. 從用戶端的登錄（`parse-registry remote-access`）、`.rdp` 檔案與 `mstsc.exe` 的 Jump Lists，找出
   使用者連線過的電腦。在那些電腦上，尋找相符的 RDP 登入，以及使用者存取記錄（UAL）中的對應資料。
4. 利用快取檔案及其在磁碟區陰影複製中之副本的 `$MFT` 時間，縮小圖塊寫入時間的範圍。

## 延伸資訊 { #further-details }

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017：RDP 點陣圖快取（法文）](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — RDP 快取外掛程式](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9：Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — 持續性點陣圖快取](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
