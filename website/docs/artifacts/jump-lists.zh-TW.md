# Jump Lists { #jump-lists }

## 什麼是 Jump Lists { #what-jump-lists-are }

Jump Lists 是從工作列與「開始」功能表的應用程式項目所顯示之選單背後的資料。自動 Jump
Lists 是由 Windows 管理、記錄應用程式所使用項目的歷程。自訂 Jump Lists 則是由應用程式
定義的類別與工作。兩者都可以內嵌 LNK 結構，其中包含目標路徑、時間、磁碟區／網路脈絡、
MFT 參照與啟動參數。

自動記錄可以揭露應用程式開啟過的檔案或資料夾、最後使用的順序／時間、互動次數與釘選狀態。
自訂清單則可能揭露「最近關閉」的頁面或應用程式工作——包括惡意的命令參數。

<span class="evidence-strength">最佳結論：</span>擁有該清單的應用程式曾在其 Jump List 中記錄或
顯示此項目／工作。行為因應用程式而異；請佐證實際的執行／存取。

## 位置與收集 { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

十六進位的檔案名稱前綴是 AppID。請收集這兩個目錄，並保留使用者設定檔路徑。自動清單是
OLE 複合檔案容器，內含一個 `DestList` 資料流與編號的 LNK 資料流；自訂清單則使用其自己的
類別／連結格式。

## 分析注意事項 { #analyst-cautions }

- AppID 的說明來自已知的對應表；未知的 AppID 本身並不可疑。
- `LastModified`／最後使用欄位屬於 DestList 項目；內嵌的目標時間則是不同的時間。
- 互動次數只存在於支援的版本中，且並非完整的開啟次數。
- 釘選會讓項目保留得比一般的最近使用期間更久；清理作業或應用程式設定可能移除其他項目。
- 清單可能包含過時／未連結的資料流，或來自先前較長之自訂清單的位元組。
- 內嵌的 LNK 參數與路徑可由攻擊者控制。與應用程式的關聯只是脈絡，並不代表可信任。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-list-csv
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

使用 `auto-parse` 時，CSV 表格為 `FileAndFolderOpening/JumpLists-AutomaticDestinations.csv`
與 `FileAndFolderOpening/JumpLists-CustomDestinations.csv`，直接位於每台電腦之下。
共用的記錄為該類別中的 `JumpLists-logs.txt`。資訊清單會在 `OutputFiles` 中列出確切路徑。
獨立的 CSV 匯出會在你選擇的目錄中保留 `AutomaticDestinations.csv` 與
`CustomDestinations.csv`。JSON 與 JSONL 的自動匯出為單一的 `JumpLists.json` 或
`JumpLists.jsonl` 檔案；記錄結構不變。

## AutomaticDestinations CSV 欄位 { #automaticdestinations-csv-fields }

| 欄位群組 | 欄位與判讀 |
|---|---|
| 來源 | `SourceFile`、`SourceCreated`、`SourceModified`、`SourceAccessed`：Jump List 容器及其檔案系統時間。 |
| 應用程式 | `AppId`、`AppIdDescription`：檔案名稱中的 AppID 與已知的應用程式對應。不要從未知的雜湊推斷身分。 |
| 容器 | `HasSps`、`DestListVersion`、`LastUsedEntryNumber`：DestList 結構／版本與標頭脈絡。 |
| 排序 | `MRU`、`EntryNumber`：最近使用的位置，以及十六進位的項目／資料流識別碼。 |
| 使用中繼資料 | `CreationTime`、`LastModified`、`InteractionCount`、`PinStatus`、`PinPosition`：DestList 的生命週期／使用／釘選狀態。請依版本判讀。 |
| 主機／路徑 | `Hostname`、`MacAddress`、`Path`、`DisplayPath`：目標位置與追蹤脈絡。主機／MAC 不可作為唯一的歸因依據。 |
| 追蹤 ID | `FileBirthDroid`、`FileDroid`、`VolumeBirthDroid`、`VolumeDroid`：用於關聯被移動的目標與內嵌 LNK 的追蹤資訊。 |
| 備註 | `Notes`：說明未連結的資料流、復原的殘留資料，或其他剖析器脈絡。 |
| 內嵌 LNK | `TargetCreated`、`TargetModified`、`TargetAccessed`、`FileSize`、`RelativePath`、`WorkingDirectory`、`FileAttributes`、`HeaderFlags`、`DriveType`、`VolumeSerialNumber`、`VolumeLabel`、`LocalPath`、`CommonPath`、`TargetIDAbsolutePath`、`TargetMFTEntryNumber`、`TargetMFTSequenceNumber`、`MachineID`、`MachineMACAddress`、`TrackerCreatedOn`、`ExtraBlocksPresent`、`Args`：請依 [LNK 捷徑](lnk.md)中的說明判讀。 |
| 其他 LNK | `NetworkPath`、`Name`、`IconLocation`：來自內嵌捷徑的網路與顯示欄位。 |
| 驗證 | `Problems`：缺少的資料流、截斷、數量不一致或連結損毀。 |

## CustomDestinations CSV 欄位 { #customdestinations-csv-fields }

自訂清單的輸出以 `SourceFile`、`SourceCreated`、`SourceModified`、`SourceAccessed`、`AppId`、
`AppIdDescription` 與 `EntryName`（類別名稱或 `Tasks`）開頭，接著是上方列出的所有內嵌 LNK 欄位，
然後是 `Notes`、`NetworkPath`、`Name`、`IconLocation` 與 `Problems`。

JSON/JSONL 另外保留 DestList 標頭、每個項目的分數／檢查碼／屬性存放區、
`DestListPropertyStore`、複合檔案資料流、自訂類別、最後一個類別之後的位元組，
以及每個內嵌 LNK 的巢狀欄位。

## 調查策略 { #investigation-strategy }

1. 識別使用者與來源設定檔，然後審慎地將 AppID 對應到應用程式。
2. 依 `LastModified` 與 `MRU` 排序自動項目；將釘選的項目分開檢視。
3. 檢視路徑、網路共用資料夾、卸除式磁碟區序號、MFT 參照與參數。
4. 檢查 `Notes` 以及未連結／殘留的連結——已刪除或較早的項目可能殘存在那裡。
5. 與 RecentDocs、獨立的 Recent LNK、Prefetch、SRUM、`$MFT` 與事件記錄進行關聯分析。

## 延伸資訊 { #further-details }

- [Microsoft：Compound Binary File 格式（MS-CFB）](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft：Shell Link 格式（MS-SHLLINK）](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [Eric Zimmerman 的 JLECmd](https://github.com/EricZimmerman/JLECmd)
- [JumpList 剖析器測試／參考專案](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
