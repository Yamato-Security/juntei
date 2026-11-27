# 事件記錄擷取 { #carve-evtx }

從磁碟映像檔、未配置空間、分頁檔或記憶體映像檔中，擷取（carving）記錄檔遭清除或刪除後仍殘留的事件記錄區塊。每個 64 KiB 的 EVTX 區塊都帶有自己的字串表與範本表，因此即使沒有記錄檔的其餘部分，也能讀出其中的記錄。Juntei 會將還原的區塊寫成一般的 `.evtx` 檔案，每個通道一個檔案，可直接以 Hayabusa 或事件檢視器開啟。

```bash
juntei carve-evtx -f host.E01 -o carved/
juntei carve-evtx -f host.vhdx --free-space-only -o carved-free/
juntei carve-evtx -f pagefile.sys --live-logs evidence/C/Windows/System32/winevt/Logs/ -o carved-pagefile/
juntei carve-evtx -f memory.raw -o carved-memory/ -t jsonl
```

未指定 `-o` 時，只會顯示摘要。

## 尋找區塊 { #finding-chunks }

- 會在每個位元組位移搜尋區塊簽章，且每個標頭都必須合理（標頭大小、記錄編號、位移）。總和檢查碼只會回報，不作為必要條件：部分遭覆寫的區塊仍可能保有可讀取的記錄。
- NTFS 壓縮記錄檔的區塊會先解壓縮（LZNT1，以 16 個叢集為單位）。在磁碟映像檔上會使用該磁碟區的叢集大小；其他檔案則使用 `--cluster-size <BYTES>`（預設為 4096）。
- 記錄會依各自的大小逐筆讀取，直到標頭所記載的最後一筆記錄位移為止，不會因記錄 ID 重複而停止。遇到損壞的記錄時，會從下一筆有效記錄繼續讀取。區塊 slack 中的舊記錄也會一併列出。

## 磁碟映像檔與 NTFS 磁碟區 { #disk-images }

對於 E01 或 VHDX 映像檔，或是含有 NTFS 的原始磁碟或磁碟區，每個區塊都會依其所在的叢集分類：

| 位置 | 意義 |
|---|---|
| `FreeSpace` | 沒有任何檔案使用的叢集。 |
| `DeletedLog` | 已刪除記錄檔的 `$MFT` 記錄仍列有的未配置叢集，或由這些叢集重建的記錄檔。 |
| `LiveLog` | 使用中事件記錄檔的叢集。除非指定 `--keep-live`，否則不會輸出。 |
| `AllocatedFile` | 其他檔案的叢集，例如 `pagefile.sys` 或磁碟區陰影複製的存放區檔案。 |
| `AllocatedUnknown` | 沒有任何 `$MFT` 記錄列出的已配置叢集。 |
| `OutsideVolumes` | 位於磁碟上所有 NTFS 磁碟區之外。 |
| `Unclassified` | 輸入中沒有 NTFS 磁碟區（分頁檔或記憶體映像檔）。 |

若已刪除記錄檔的 `$MFT` 記錄只列出未配置叢集，該記錄檔就會整份重建（包括壓縮的記錄檔），並儲存在 `Recovered/` 下。`--free-space-only` 只搜尋未配置叢集：速度較快，但會略過分頁檔、磁碟區陰影複製及其他檔案。

## 重複資料刪除 { #deduplication }

每個區塊只會寫入一次。若區塊屬於使用中的記錄檔、與另一個區塊完全相同，或其所有記錄都已存在於使用中的記錄檔或已寫入的區塊中，就不會輸出。記錄較多的區塊會優先處理，因此同一區塊較舊、較短的副本不會再帶來任何新內容。對於沒有 NTFS 的輸入，請以 `--live-logs <DIR>` 指定使用中的記錄檔。

## 結果 { #results }

- `Carved/<CHANNEL>.evtx`: 每個通道一份記錄檔，命名方式與 Windows 相同（`Microsoft-Windows-Sysmon%4Operational.evtx`）。區塊標頭中的記錄編號會重新編號，總和檢查碼也會重新計算。損壞的記錄及其後的所有內容都會成為 slack。除此之外，每個區塊都會完整保留（包括其 slack），因此仍可對其進行記錄還原，例如 Hayabusa 的 `-x`。 Hayabusa 的 `-x` 會搜尋標記為空的區塊。若要查看部分使用中的區塊在殘餘空間內留下的記錄，請查閱 Juntei 的 `records.csv`。
- `Recovered/Volume<N>/...`: 根據 `$MFT` 記錄重建的已刪除記錄檔。
- `chunks.csv` / `chunks.jsonl`: 找到的每個區塊，包括所在位置、處理結果（`Status`）、通道、記錄 ID、時間與總和檢查碼狀態。
- `records.csv` / `records.jsonl`: 已寫入區塊中的每筆記錄。`OnlyInCarvedData` 會標示使用中的記錄檔所沒有的記錄。

```bash
hayabusa csv-timeline -d carved/Carved -a -o carved-timeline.csv
```

擷取只能找到連續的區塊。跨越不連續叢集，或跨越記憶體中不連續實體分頁的區塊並不完整，其後段的記錄可能損壞或遺失。請先將 `hiberfil.sys` 解壓縮。搜尋完成時結束代碼為 0，發生錯誤時為 1。

## 進一步資訊 { #further-details }

- [詳細用法與欄位參考（英文）](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.md)
- [詳細用法與欄位參考（日文）](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.ja.md)
- [EVTX 格式（libevtx 說明文件）](https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc)
