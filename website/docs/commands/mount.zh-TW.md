# 掛載磁碟映像 { #mounting-disk-images }

`juntei mount` 在執行期間將 VHDX 磁碟映像內的檔案以唯讀方式呈現在目錄中。Hayabusa 和 Juntei 等工具可直接掃描，不必擷取檔案或取得 root 權限。此功能使用 FUSE，適用於 Linux。讀取器與掛載函式庫由 [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) 維護。

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
  <IMAGE>       The disk image (VHDX)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## 原生映像分析 { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` 可在 **Linux、macOS 和 Windows** 原生讀取 VHDX，無需 OS 掛載、FUSE 或系統管理員權限。

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

解析器需要檔案路徑，因此 Juntei 會將 NTFS 讀取器公開的檔案複製至專用暫存目錄。**請預留足夠的暫存磁碟空間以容納這些檔案。** 原始映像維持唯讀，正常結束時會刪除暫存副本。

讀取失敗及不安全的檔名會記錄於 `ImageErrors` 與 `manifest.csv` 的失敗列。仍會保留可讀取資料的結果；輸入不完整時結束代碼為 1。

供其他程式存取檔案的獨立 `mount` 命令仍需要 Linux、FUSE 3 及 `/dev/fuse` 存取權。

[English: implementation details and limitations](https://yamato-security.github.io/juntei/commands/auto-parse/#vhdx-disk-images).
