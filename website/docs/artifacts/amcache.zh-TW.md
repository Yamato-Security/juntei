# Amcache { #amcache }

## 什麼是 Amcache { #what-amcache-is }

Amcache 是儲存在登錄區中的 Windows 應用程式相容性清單。新版系統會將執行檔記錄在
`InventoryApplicationFile` 下，將隨插即用裝置記錄在 `InventoryDevicePnp` 下；較舊的
Windows 版本則使用舊式的 `Root\File` 配置。

在調查執行檔時，Amcache 之所以有價值，是因為即使原始檔案已經消失，它仍可能保留完整路徑、SHA-1、
檔案大小、PE 中繼資料、版本資訊、發行者文字、連結日期與登錄時間戳記。裝置記錄則可識別
曾連接的硬體、驅動程式、安裝日期與識別碼。

<span class="evidence-strength">最佳結論：</span>Windows 曾將所記錄的檔案或裝置納入清單。
執行檔記錄是**存在的證據，而非執行的證明**。

## 位置與收集 { #location-and-collection }

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

請從受影響的安裝環境收集此登錄區。`parse-registry amcache` 目前會剖析主要登錄區，
但**不會**重播相鄰的 Amcache 交易記錄，因此僅存在於這些記錄中的項目，在即時收集或
不一致的收集中可能會遺漏。請在報告中註明此限制。

## 分析注意事項 { #analyst-cautions }

- Amcache 的執行檔項目並不能證明使用者曾啟動該檔案。
- `Publisher` 是 Windows 所記錄的 PE／版本資源中繼資料，並不能證明該檔案在收集時
  具有有效的 Authenticode 簽章。
- 雜湊值可能不存在，而且舊式與新式結構描述所提供的值並不相同。
- `LinkDate` 來自 PE 標頭，可能被偽造、複製，或由可重現建置產生。
- 機碼的最後寫入時間描述的是登錄記錄的更新，不一定是下載、建立、
  首次執行或最後執行的時間。
- 路徑可能指向卸除式磁碟區或已不存在的磁碟區。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.RecordType == "file" and .SHA1 == "0123456789abcdef0123456789abcdef01234567")'
```

CSV 輸出包含摘要、發現項目、檔案與裝置資料表。使用 `auto-parse` 時，這些資料表為
`Registry-Amcache-Summary.csv`、`Registry-Amcache-Findings.csv`、`Registry-Amcache-Files.csv` 與
`Registry-Amcache-Devices.csv`，直接位於每台電腦的 `FileExistence/` 目錄下，
`Registry-Amcache-logs.txt` 則位於資料表旁。若有多個登錄區，會在副檔名前加上 `-Source1`、`-Source2` 等；
清單檔中的 `OutputFiles` 會列出確切的路徑。獨立匯出的 CSV
會保留其原生資料表名稱與所指定的輸出位置。JSONL 記錄使用
`RecordType` 值 `summary`、`finding`、`file` 與 `device`。

## 檔案輸出欄位 { #file-output-fields }

| 欄位 | 分析人員應如何使用 |
|---|---|
| `KeyName`、`SourceKeyPath` | 識別確切的來源登錄記錄與結構描述分支。請保留以確保可重現性。 |
| `KeyLastWrittenUTC`、`KeyLastWrittenFiletime` | 登錄記錄的更新時間，分別為易讀與原始格式。請勿將其標示為執行時間。 |
| `SourceVolumeKeyLastWrittenUTC`、`SourceVolumeKeyLastWrittenFiletime` | 舊式結構描述中上層磁碟區機碼的時間；屬於脈絡資訊，並非檔案操作時間。 |
| `FileID` | Amcache 所記錄的檔案識別碼。完整保留；不要假設它一定是 SHA-1。 |
| `SHA1`、`SHA256` | 可樞紐至 Defender、威脅情資、EDR 與檔案儲存庫的有力線索。Null 表示未記錄。 |
| `Name`、`OriginalFileName`、`Description` | 所記錄的 PE／版本名稱。與實際路徑比較以偵測偽裝；這些中繼資料可由攻擊者控制。 |
| `FullPath` | 最有用的位置樞紐。正規化大小寫時請謹慎，並保留原始的證據字串。 |
| `LongPathHash` | Windows 與路徑相關的識別碼；適合用於比對記錄，並非內容雜湊值。 |
| `ProgramID` | 在可用時，將檔案連結至應用程式清單記錄。 |
| `Publisher` | 所記錄的發行者文字。請視為中繼資料，而非簽章驗證。 |
| `Version`、`BinFileVersion`、`ProductName`、`ProductVersion`、`BinProductVersion` | 版本資源脈絡。差異或不合理的值可支持偽裝分析。 |
| `BinaryType` | 所記錄的架構／類型脈絡；復原檔案後，請與 PE 分析結果比較。 |
| `LinkDate` | 所記錄的 PE 編譯／連結時間戳記。僅在抱持懷疑並取得佐證時才有用。 |
| `Size` | 所記錄的大小；請與 `$MFT`、Defender 內容或復原的檔案比較。 |
| `Language` | PE 語言識別碼；是薄弱的脈絡線索，不能作為歸因依據。 |
| `IsPE-File`、`IsOS-Component` | Windows 所記錄的分類旗標。值為 false 並不能證明無害。 |
| `USN` | Amcache 所記錄的 NTFS 更新序號；有助於樞紐至留存的 USN 記錄。 |
| `APPX-PackageFullName`、`APPX-PackageRelativeID` | 將封裝的應用程式／元件與該檔案關聯起來。 |
| `RegistryValues` | 每一個原始登錄值，包含類型、解碼後的值與原始十六進位資料。當型別化欄位不存在，或用於未來／舊式欄位時使用。 |

## 裝置輸出欄位 { #device-output-fields }

| 欄位群組 | 欄位與判讀 |
|---|---|
| 來源 | `KeyName`、`SourceKeyPath`、`KeyLastWrittenUTC`、`KeyLastWrittenFiletime`：確切的裝置記錄與登錄更新時間。 |
| 識別 | `Description`、`BusDescription`、`Manufacturer`、`Model`：面向使用者的裝置描述；廠商字串可能很籠統。 |
| 硬體識別碼 | `HWID`、`CompID`、`Class`、`ClassGUID`、`ContainerID`、`DeviceInterfaceClasses`：最有力的裝置系列與執行個體樞紐。 |
| 狀態 | `DeviceState`、`ProblemCode`、`ParentID`、`MatchingID`、`Enumerator`：Windows 列舉與關聯脈絡。 |
| 驅動程式 | `DriverID`、`DriverName`、`DriverPackageStrongName`、`DriverVerDate`、`DriverVerVersion`、`Provider`、`Service`、`INF`：驅動程式套件與服務的關聯；請與已安裝的檔案及服務交叉比對。 |
| 安裝 | `InstallDate`、`FirstInstallDate`、`InstallState`、`ExtendedInfs`：所記錄的安裝生命週期。缺少日期並不能證明該裝置從未連接過。 |
| 篩選器與堆疊 | `LowerClassFilters`、`UpperClassFilters`、`LowerFilters`、`UpperFilters`、`StackID`、`LocationPaths`：進階的驅動程式堆疊脈絡；非預期的篩選驅動程式值得檢視。 |
| 原始資料保留 | `RegistryValues`：完整的型別化／原始來源值，用於驗證及不支援的欄位。 |

CSV 中的陣列與原始值欄位使用 `JSON` 後綴（例如 `HWID-JSON` 與
`RegistryValuesJSON`）；其儲存格內容為 JSON，而 JSON／JSONL 則保留原生陣列。

## 調查策略 { #investigation-strategy }

1. 以 `FullPath`、`SHA1`、`Name` 與 `USN` 為樞紐。
2. 尋找相符的 Prefetch 以佐證執行。
3. 使用 `$MFT` 與 USN 日誌確認檔案的建立、重新命名、刪除與大小。
4. 將發行者／版本中繼資料與獨立復原的 PE 及簽章驗證結果比較。
5. 檢查 Defender 隔離資料與事件記錄，以確認偵測或移除行為。

## 延伸資訊 { #further-details }

- [Juntei 的 ys-amcache-parser 輸出結構描述](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [Eric Zimmerman 的 AmcacheParser](https://github.com/EricZimmerman/AmcacheParser)
- [Security Ronin 的 amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [Regipy Amcache 外掛程式](https://github.com/mkorman90/regipy)
