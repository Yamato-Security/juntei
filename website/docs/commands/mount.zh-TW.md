# 掛載磁碟映像 { #mounting-disk-images }

`juntei mount` 在執行期間將 VHDX / E01 / Ex01 磁碟映像內的檔案以唯讀方式呈現在目錄中。Hayabusa 和 Juntei 等工具可直接掃描，不必擷取檔案或取得 root 權限。此功能使用 FUSE，適用於 Linux。讀取器與掛載函式庫由 [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) 維護。

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

在另一個終端機中：

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

在第一個終端機中按下 Ctrl+C 即可卸載，或執行 `fusermount3 -u /mnt/host1`。

## 掛載後顯示的內容 { #what-the-mount-shows }

- 掛載點顯示 NTFS 磁碟區內的檔案。收集的映像可能包含 `C`、`LongFileNames` 等目錄，以及 `$MFT` 等磁碟區中繼資料。多個 NTFS 磁碟區會顯示為 `Volume1`、`Volume2` 等。
- 每個檔案都是唯讀的，且屬於掛載點的擁有者。檔案時間為 NTFS 的
  `$STANDARD_INFORMATION` 時間。
- 名稱依 NTFS 儲存的原樣列出，並以完全相符的方式查找。不會顯示 DOS 8.3 名稱與替代資料流。
- 壓縮的檔案會被解壓縮（每個檔案最多 1 GiB）。以 EFS 加密的檔案無法讀取。

## 證據不會被變更 { #the-evidence-is-not-changed }

映像以唯讀方式開啟，掛載拒絕寫入。未正常關閉的 VHDX 會在記憶體中套用待處理記錄，絕不寫回映像。部分 DiscUtils 寫入器會將 NTFS 開機磁區的跳躍指令和 `55 AA` 簽章設為零；Juntei 可直接讀取這些磁碟區。

## 用法 { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX or E01; a split E01 image from its first segment)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## 原生映像分析 { #parsing-every-image-with-auto-parse }

使用 `--vhdx` 時，`auto-parse` 透過 [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) 在 **Linux、macOS、Windows** 原生讀取 VHDX，無需 OS 掛載、FUSE 或系統管理員權限。

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

解析器需要檔案路徑，因此會將讀取器公開的 NTFS 檔案中解析器會讀取的檔案（登錄 Hive 及其交易記錄、NTFS 中繼資料檔、資料庫、Prefetch 檔、捷徑、Jump List、Outlook 檔案與 Defender 隔離資料夾）串流複製至專用暫存目錄。其他檔案則成為保留原始更新時刻的空白預留位置，因此目錄與路徑看起來與磁碟上相同。persistence-hunter 僅在需要時才從映像檔讀取預留位置的內容，例如持續性項目所指向的執行檔。**除結果外，請預留足夠空間容納複製的檔案。**讀取緩衝區有大小限制；若主機支援，零區域會保留為稀疏區域。保留名稱、目錄、硬連結和更新時刻；LNK 與 Jump List 的來源建立、更新、存取時刻使用原始 NTFS 中繼資料快照，絕不採用暫存副本的時刻。正常結束（包含解析器錯誤）會刪除副本，強制終止可能留下檔案。來源維持唯讀，未反映的 VHDX 日誌僅套用至記憶體。

主機偵測、CSV/JSON/JSONL 和 `-j` 使用一般證據目錄的解析流程。`manifest.json` 中的 `Image` 識別原始映像；`EvidenceRoot` 識別內部相對路徑。讀取失敗或不安全的名稱會記錄在 `ImageErrors` 與 `manifest.csv` 的失敗列。保留可讀取資料的結果；輸入不完整時結束代碼為 1。不會擅自重新命名不安全或曖昧的名稱，也不會將具名 NTFS 串流抽取為額外檔案；初步蒐證映像中已收集為一般檔案的證據仍可使用。

獨立的 [`mount` 命令](mount.md) 將檔案系統公開給其他程式，仍需要 Linux FUSE；`auto-parse --vhdx` 不使用此命令。

暫存作業預設使用輸出檔案系統上的專用目錄。可用 `--temp-dir` 指定另一磁碟上已存在的目錄。原始中繼資料另存於輸出檔案系統，因此暫存磁碟滿載時仍可解析已讀取的檔案。請為暫存檔與結果保留空間；若輸出檔案系統本身已滿，結果將無法儲存。Ctrl+C 會停止解析程序、清除專用工作區，並以代碼 130 結束。SIGKILL 或斷電仍可能留下副本。搜尋映像檔時會略過無法讀取的子目錄，但輸入根目錄無法讀取仍屬錯誤。

暫存目錄必須位於輸出目錄之外。在 Linux 和 macOS 上，SIGTERM 和 SIGHUP 也會停止解析程序並清理工作區。

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## E01 與 Ex01 磁碟映像檔 { #e01-and-ex01-disk-images }

`--e01` 可在 **Linux、macOS 和 Windows** 原生讀取實體 EWF1（`.E01`）與 EWF2（`.Ex01`）映像檔，不需要 FUSE、作業系統掛載或管理員權限。它使用 [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) 及上述相同的 NTFS 解析流程。

分割映像檔請指定第一個區段。將 `.E02` 等所有相關檔案以原始名稱保留在相同目錄。支援完整磁碟（MBR/GPT）與單一 NTFS 磁碟區。混合兩種格式的目錄可使用 `--vhdx --e01`。

暫存空間與來源中繼資料的要求相同。E01 映像檔通常包含整顆磁碟，且所有映像檔都會在解析開始前完成暫存，但只會複製解析器讀取的檔案，通常僅占已用空間的一小部分。映像檔以唯讀方式開啟。取得時的缺損及開啟或讀取時發現的損壞會列入 `ImageErrors` 與 `manifest.csv`；保留可讀取的結果並以狀態碼 1 結束。這只檢查實際存取的資料，不是所有區塊或完整媒體雜湊。拒絕邏輯格式 `.L01` 與 `.Lx01`，且沒有密碼選項。

獨立的 `mount` 命令仍需要 Linux FUSE；原生自動解析則不需要。

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## OS 可見掛載的需求 { #os-visible-mount-requirements }

`juntei mount` 需要 Linux FUSE 3（`fusermount3`）和 `/dev/fuse` 存取權。Windows 和 macOS 請使用上述原生解析方式。
