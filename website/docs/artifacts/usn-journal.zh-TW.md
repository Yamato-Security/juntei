# USN 日誌 (`$UsnJrnl:$J`) { #usn-journal-usnjrnlj }

## 什麼是 USN 日誌 { #what-the-usn-journal-is }

NTFS 會在 Update Sequence Number（USN）變更日誌中記錄檔案與目錄的變更。一筆記錄可以描述建立、
資料延伸／覆寫、屬性或安全性變更、重新命名、關閉以及刪除。它會儲存檔案參照、父參照、名稱、原因、屬性、
時間戳記，以及單調遞增的 USN。

此日誌非常適合用來重建攻擊者短時間內大量的檔案操作，特別是在檔案已遭刪除時。它記錄的是**中繼資料／內容發生了變更**，
而非變更的位元組。

<span class="evidence-strength">最佳結論：</span>NTFS 在該時間戳記為所參照的檔案記錄了所述的變更原因。
原因通常會持續累積，直到出現 `CLOSE` 記錄為止。

## 位置與收集 { #location-and-collection }

```text
C:\$Extend\$UsnJrnl:$J
```

`$J` 是 NTFS 中繼資料的具名資料流。請使用支援原始存取的鑑識工具收集。同時請收集同一磁碟區的 `$MFT`，
因為 USN 記錄儲存的是父檔案參照，而非完整路徑。

## 分析注意事項 { #analyst-cautions }

- 此日誌為循環式且大小有限，舊記錄會被覆寫。
- `FILE_DELETE` 表示 NTFS 已處理刪除；內容仍可能在其他地方可供復原。
- 像 `DATA_OVERWRITE` 這樣的原因，並不會揭露相關的位元組、程序或使用者。
- 一筆記錄中可能合併多個原因；`CLOSE` 通常標示累積變更的結束。
- 路徑是重建出來的。父目錄可能已被重新命名、刪除或重複使用。
- 檔案參照的序號很重要。僅憑項目編號可能會連結到錯誤的世代。
- 擷取（carving）可以復原有效記錄，但不含其原始日誌脈絡；請在報告中說明此區別。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-ntfs usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $J files
  -f, --file <FILE>      Parse one $J, or any file with --carve
  -m, --mft <FILE>       Use the matching volume's $MFT for paths
      --carve            Search a disk image/unallocated file for records

Output:
  -o, --output <PATH>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-ntfs usn-journal -d evidence/ -o usn-jsonl -t jsonl
juntei parse-ntfs usn-journal -f unallocated.bin --carve -o carved-usn.csv
```

## CSV 輸出欄位 { #csv-output-fields }

| 欄位 | 判讀 |
|---|---|
| `Name` | 此 USN 記錄中儲存的檔案／目錄名稱，並非完整路徑。 |
| `Extension` | 為方便起見從 `Name` 剖析出的副檔名；請勿將其視為可信的內容類型。 |
| `EntryNumber`, `SequenceNumber` | 檔案的 NTFS 參照。請同時使用兩者來識別記錄的世代。 |
| `ParentEntryNumber`, `ParentSequenceNumber` | 變更當時父目錄的 NTFS 參照。 |
| `ParentPath` | Juntei 考量時間因素所重建的父路徑。出現未知標記表示無法解析其上層路徑。 |
| `UpdateSequenceNumber` | USN 排序值。可用於穩定排序以及在 Amcache／MFT 之間交叉比對。 |
| `UpdateTimestamp` | V2/V3 記錄中儲存的時間戳記。對於擷取所得脈絡中的時間戳記，請與其他記錄一樣進行驗證。 |
| `UpdateReasons` | 一個或多個具名的變更旗標，以 `|` 連接。請將組合與 `CLOSE` 一併判讀。 |
| `FileAttributes` | 變更當時記錄的 NTFS 屬性；未知的位元會維持十六進位表示。 |
| `OffsetToData` | 在輸入中的位元組位移。對驗證與擷取所得的記錄至關重要。 |
| `SourceFile` | 輸入的日誌／映像檔；用以區分主機與磁碟區。 |
| `SourceInfo` | 描述變更如何進入日誌的來源旗標。通常屬於脈絡資訊，而非歸屬依據。 |
| `SecurityID` | 該記錄的 NTFS 安全性識別碼索引，而非使用者 SID。 |
| `Version` | USN 記錄版本。Juntei 會將 V2/V3 寫入 CSV；JSON 則保留 V4 範圍追蹤的 extent。 |

JSON/JSONL 能正確表示 128 位元的 V3 參照，並保留 V4 範圍追蹤記錄及其 extent。
當低階、與版本相關的欄位很重要時，請使用 JSON/JSONL。

## 調查策略 { #investigation-strategy }

1. 依時間、名稱、副檔名、父路徑或檔案參照進行篩選。
2. 除了時間之外，也依 `UpdateSequenceNumber` 排序，以查看確切的日誌順序。
3. 將 `RENAME_OLD_NAME` 與 `RENAME_NEW_NAME` 配對，以找出移動／重新命名。
4. 使用 `$MFT` 驗證記錄狀態、路徑、大小與時間戳記。
5. 從事件記錄、EDR、Prefetch、服務／工作或 Defender 交叉比對程序歸屬。

## 延伸資訊 { #further-details }

- [Microsoft：Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft：USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft：Change Journal 原因旗標](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2#members)
- [ys-ntfs-parser 的 usn crate](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/usn)
- [Sootmark usn](https://github.com/Sootmark/usn)
