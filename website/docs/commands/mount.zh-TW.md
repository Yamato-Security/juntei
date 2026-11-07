# 掛載磁碟映像 { #mounting-disk-images }

`juntei mount` 會在執行期間，讓 VHDX 磁碟映像檔內的檔案能在某個目錄下以唯讀方式讀取。如此一來，
KAPE 的 `--vhdx` 收集資料與 Hyper-V 磁碟就能以任何工具掃描，例如 Hayabusa 或 Juntei 本身的命令，
完全不需要擷取任何內容、不需要 root 權限，作業系統也不必支援 VHDX 或 NTFS。它使用 FUSE，
可在 Linux 上運作。

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

- 掛載點下會顯示 NTFS 磁碟區的檔案：若為 KAPE 收集資料，會有存放所收集檔案的 `C` 資料夾、
  KAPE 的 `LongFileNames` 資料夾，以及磁碟區本身的中繼資料檔案，例如 `$MFT`。包含多個 NTFS
  磁碟區的映像檔，會將它們顯示為 `Volume1`、`Volume2`，依此類推。
- 每個檔案都是唯讀的，且屬於掛載點的擁有者。檔案時間為 NTFS 的
  `$STANDARD_INFORMATION` 時間。
- 名稱依 NTFS 儲存的原樣列出，並以完全相符的方式查找。不會顯示 DOS 8.3 名稱與替代資料流。
- 壓縮的檔案會被解壓縮（每個檔案最多 1 GiB）。以 EFS 加密的檔案無法讀取。

## 證據不會被變更 { #the-evidence-is-not-changed }

映像檔以唯讀方式開啟，掛載也會拒絕寫入。未正常關閉的 VHDX 在讀取時，會在記憶體中套用其待處理的
記錄，絕不會寫回映像檔。KAPE 使用 DiscUtils 寫入其 VHDX 檔案，而 DiscUtils 寫出的 NTFS 開機磁區
缺少跳躍指令與 `55 AA` 簽章；Juntei 會照原樣讀取這些磁碟區。

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

## 使用 auto-parse 剖析每個映像檔 { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` 會剖析證據目錄中每個 VHDX 磁碟映像檔（或以 `-d` 指定的單一映像檔）內的
Windows 系統，不需要另外掛載：

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

- 每個映像檔會以唯讀方式掛載於一個新的暫存目錄中、以該映像檔命名的資料夾（`host1-triage`），
  在其中找到的系統會如同磁碟上的收集資料一樣被剖析。映像檔會在執行結束時卸載，按下 Ctrl+C 時
  也會卸載。
- 終端機會列出各映像檔，以及每個系統所屬的映像檔：
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`。
- 在清單檔中，`EvidenceDirectory` 是所提供的目錄，每個系統的 `Image` 則標示其所屬的映像檔。
  `EvidenceRoot` 與剖析器命令使用掛載資料夾（`./host1-triage/C`）。
- KAPE 會將從 Volume Shadow Copies 複製出的檔案存放在 `VSS32` 等資料夾中。每個這類資料夾都會
  被找到，並作為另一個系統（`HOST1_2`、…）進行剖析。
- 無法掛載的映像檔會被回報；其他映像檔仍會被剖析，而這次執行會以結束狀態 1 結束。

## 需求 { #requirements }

- Linux：FUSE 3（`fusermount3`，即 `fuse3` 套件），以及對 `/dev/fuse` 的存取權限，一般使用者
  通常都具備。不需要 root。
- 不支援 Windows 與 macOS。在 Windows 上，請改為在以系統管理員身分執行的 PowerShell 中，以唯讀方式
  連結映像檔，並將其磁碟機代號提供給工具：

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
