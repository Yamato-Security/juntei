# 開機磁區 (`$Boot`) { #boot-sector-boot }

## 什麼是 `$Boot` { #what-boot-is }

`$Boot` 是每個 NTFS 磁碟區的 MFT 項目 7：即磁碟區最前面的 8 KiB。其第一個磁區，也就是
**磁碟區開機記錄**（volume boot record），保存了描述檔案系統的 BIOS 參數區塊：

- 磁區與叢集大小、磁碟區中的磁區數量，以及**隱藏磁區**
  （磁碟上位於該磁碟區之前的磁區，也就是其分割區的起始位置）；
- `$MFT` 與 `$MFTMirr` 的起始位置，以及 FILE 與 INDX 記錄的大小；
- 64 位元的**磁碟區序號**，於格式化磁碟區時設定。

第一個磁區的其餘部分以及其後的 15 個磁區保存了載入 Windows 的**開機程式碼**。

序號的低 32 位元就是 Windows 以 `XXXX-XXXX` 形式顯示的值（`vol`、`dir`），也是
[LNK 捷徑](lnk.md)、[Jump Lists](jump-lists.md) 與 [Prefetch](prefetch.md) 用來記錄
檔案所在磁碟區的值。`$Boot` 能將這些記錄連結到所收集主機的某個磁碟區。

<span class="evidence-strength">最佳結論：</span>與某個 `$Boot` 的 `VolumeSerialNumber`
相同的 LNK、Jump List 或 Prefetch 磁碟區序號，指向該磁碟區，或指向被設定為（或連同複製而得到）
相同序號的另一個磁碟區。序號可以被變更，且在不同主機之間並非唯一。

## 位置與收集 { #location-and-collection }

```text
C:\$Boot          (MFT entry 7; the first 8 KiB of the volume)
```

請使用支援原始存取的鑑識工具收集它，或讀取磁碟區映像檔的第一個磁區：兩者是相同的位元組。
Juntei 會讀取任何以 NTFS 開機磁區開頭的檔案，因此磁碟區映像檔也可以作為輸入。磁碟映像檔
則是以分割表開頭；請將分割區的磁碟區提供給 Juntei。

## 分析注意事項 { #analyst-cautions }

- 磁碟區被格式化時，序號會改變。序號與目前任何磁碟區都不相符的 LNK 檔案，可能指向
  卸除式媒體、網路共用資料夾的伺服器，或之後才被格式化的磁碟區。
- 由相同 Windows 格式化的磁碟區，其開機程式碼是相同的：DFIR Artifact Museum 中
  Windows 10、11 與 Server 2012 R2 的 `$Boot` 檔案，其開機程式碼完全一致。磁碟區開機記錄
  bootkit 會變更開機程式碼，或變更開機程式碼用來尋找 `$Boot` 其餘部分的隱藏磁區欄位。
  在下任何結論之前，請將 `BootCodeSHA256`、`JumpInstruction` 與 `HiddenSectors` 與其他主機
  以及分割表進行比較：開機管理程式或加密軟體也可能變更開機程式碼。
- BitLocker 加密磁碟區的映像檔是以 BitLocker 自己的開機磁區開頭（OEM ID
  `-FVE-FS-`）。以 `-f` 提供時，Juntei 會回報它，並在 `Problems` 中說明它不是 NTFS；NTFS
  開機磁區位於加密資料中。從已解鎖磁碟區收集的 `$Boot` 則是 NTFS 的開機磁區。
- `Problems` 列出已辨識的結構不一致與截斷，而非提供完整的 Windows 可掛載性測試。請用它來
  評估擷取出的磁區與副本，而不要單獨將其視為攻擊發現。開機程式碼雜湊不包含開頭的跳躍指令、
  BIOS 參數區塊與結尾特徵碼；請一併檢查這些欄位。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      Parse one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

摘要會列出每個開機磁區的磁碟區序號（以 Windows 顯示的形式，以及 LNK 與 Prefetch 欄位
顯示的形式）、大小、叢集大小與 `$MFT` 叢集。解析多台主機時，它會依開機程式碼雜湊將它們分組，
讓開機程式碼與其他磁碟區不同的磁碟區能被突顯出來。

## CSV 輸出欄位 { #csv-output-fields }

| 欄位 | 判讀 |
|---|---|
| `SourcePath` | 輸入檔案。 |
| `VolumeSerialNumber` | 32 位元序號，以 LNK、Jump List 與 Prefetch 輸出顯示的形式呈現（`180B1E67`）。Windows 將其顯示為 `180B-1E67`。 |
| `VolumeSerialNumber64` | 完整的 64 位元序號。 |
| `OEM-ID` | 在 NTFS 磁碟區上為 `NTFS`。 |
| `BytesPerSector`、`SectorsPerCluster`、`ClusterSize` | 磁碟區的幾何配置。每叢集磁區數超過 128 時會以 2 的次方記錄，此處會加以解碼。無效時為空。 |
| `TotalSectors`、`VolumeSize` | 磁碟區中的磁區數及其位元組大小，不含位於其末端的開機磁區副本。 |
| `HiddenSectors` | 磁碟上位於該磁碟區之前的磁區：即其分割區的起始位置。 |
| `MFT-Cluster`、`MFT-Offset` | `$MFT` 的起始位置，以叢集及自磁碟區起點算起的位元組表示。 |
| `MFT-MirrorCluster`、`MFT-MirrorOffset` | `$MFTMirr` 的起始位置。 |
| `FileRecordSize`、`IndexRecordSize` | 每個 FILE 記錄（通常為 1,024）與每個 INDX 記錄（通常為 4,096）的位元組數。 |
| `MediaDescriptor` | 硬碟為 `F8`。 |
| `JumpInstruction` | 最前面三個位元組（`EB 52 90`）：越過 BIOS 參數區塊跳至開機程式碼的跳躍指令。 |
| `EndSignature` | 有效開機磁區末端的 `55 AA`。 |
| `BootCodeSHA256` | 開機程式碼的 SHA-256：第一個磁區的位元組 0x54 至 0x1FD，以及 8 KiB 中的其餘部分。 |
| `Problems` | Windows 不會掛載的情況：`NTFS` 以外的 OEM ID、沒有 `55 AA`、無效的大小、NTFS 保持為零的 FAT 欄位被設定、`$MFT` 超出磁碟區末端；以及短於 8 KiB 的檔案。 |

JSON/JSONL 具有相同的欄位與名稱，數字以數值表示，未知的值為 `null`，`Problems` 則為清單。

## 調查策略 { #investigation-strategy }

1. 將每台主機的 `VolumeSerialNumber` 與其 LNK 檔案、Jump Lists 與 Prefetch 檔案的
   磁碟區序號並列比對：與任何已收集磁碟區都不相符的，就是其他磁碟區（USB 隨身碟、
   共用資料夾、之後才格式化的磁碟區）。
2. 在 [USB 裝置](usb.md)的輸出中查詢其他序號，該輸出會列出已掛載之卸除式磁碟區的磁碟區序號。
3. 比較各主機的 `BootCodeSHA256` 與 `HiddenSectors`。對於不同的開機程式碼，請以反組譯工具
   與磁碟的分割表進行調查，並尋找與開機相關的事件記錄項目。
4. 使用 `MFT-Offset` 與 `ClusterSize` 在磁碟區映像檔中找出 `$MFT`。

## 延伸資訊 { #further-details }

- [Microsoft：NTFS 概觀](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
- [libfsntfs：NTFS 格式文件](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc)（磁碟區標頭）
- [The Sleuth Kit：fsstat](https://www.sleuthkit.org/sleuthkit/man/fsstat.html)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)（其 `boot` crate）
