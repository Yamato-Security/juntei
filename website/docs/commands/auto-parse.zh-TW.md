# 自動解析收集資料 { #automatic-collection-parsing }

`auto-parse` 是將包含多台電腦的 Windows 分流收集資料，轉換為結構化分析目錄最快的方式。
只需提供證據根目錄與一個新的輸出目錄。它會找出
所收集的系統、識別其電腦名稱，並以平行方式執行每一個相容的 Juntei 剖析器。

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

預設輸出為 CSV。若要在保留這些 CSV 結果的同時，另外產生可供 `jq` 或 SIEM 匯入的 JSONL 時間軸：

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "請將輸出與證據的目錄樹分開"
    Juntei 會拒絕位於 `<EVIDENCE>` 之內或包含 `<EVIDENCE>` 的輸出目錄。這
    可防止產生的資料被當成證據再次被找到，也可防止 `--clobber`
    動到來源收集資料中的檔案。

## 輸出時區 { #output-time-zone }

剖析出的時間戳記預設為 UTC，與分析人員電腦的當地時區無關。UTC
採用 Windows 事件記錄的格式 `2026-10-20T08:46:38.2001668Z`：ISO 8601 日期／時間，日期與時間之間為 `T`，
並以 `Z` 表示 UTC。可用的小數位數會被保留。
`manifest.json` 的 `GeneratedUTC` 也使用此格式。主控台的 `Start time` 與 `End time`
則為分析人員電腦的當地時間，例如 `2026-11-04 09:15:02`，不受輸出時區
影響。

使用 `-Z, --timezone` 選擇固定的顯示時差：

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

時差是帶正負號、兩位數的 `HH:MM` 值（小時 `00–23`，分鐘 `00–59`）。像是
`JST` 這樣的名稱、`+9` 這樣的簡寫值，以及無效的範圍都會產生錯誤，並附上語法範例。
若要使用 UTC，請省略此選項或使用 `--timezone "+00:00"`。固定時差不會套用日光
節約時間規則。

此選項適用於各跡證 CSV、JSON 與 JSONL 結果中剖析出的時間戳記，以及
額外的 `-j` JSONL 匯出。例如，`2026-10-20T08:46:38.2001668Z` 會變為
`2026-10-20T17:46:38.2001668+09:00`；兩者描述的是同一個時間點。

欄位名稱維持不變，包括既有以 `UTC` 結尾的名稱。轉換後的值會明確地
帶有所選的時差。兩份清單檔中的 `OutputTimeZone` 會記錄所要求的顯示
時差，而 `GeneratedUTC` 仍為 UTC 的執行中繼資料。各剖析器的記錄檔則保留剖析器原生的 UTC 摘要。

原始的 FILETIME／epoch 數值、登錄值字典、來源的時區設定、文字
中繼資料、僅有日期的值，以及時差未知的當地時鐘值都會被保留。Juntei
不會為未知的當地時間加上 `Z`。源自 FAT 的殼層項目時鐘（ShellBags／OpenSaveMRU 中的 `Item*UTC`，
以及 LNK／Jump List 的 `TargetItems`）沒有記錄時差；其沿用的
UTC 標籤或呈現的 `Z` 後綴並不能證明其為 UTC，因此時差選項會保留它們的原值。
包含自由文字摘要的 CSV 欄位會保留
其記錄的文字；JSON/JSONL 則將巢狀的剖析時間戳記保留為結構化欄位。

UTC 會直接使用既有的剖析器輸出。明確指定的非零時差，會在每個剖析器工作程序中
增加一次串流格式化處理；它只會變更顯示值，而不會變更證據。

若時差會使時間戳記超出 `0000–9999` 年的範圍，該值會保留其原本
明確的時差。小數位數超過九位的值也會維持不變，而不會
被四捨五入。Juntei 會在該剖析器的 `*-logs.txt` 檔案中記錄警告，並轉換其他的
時間戳記。格式錯誤的 CSV／JSON 或輸出錯誤仍會使該工作失敗，並保持受影響的
原始結果檔案不變。

## 會找到哪些內容 { #what-it-discovers }

Juntei 會以不區分大小寫的方式遞迴搜尋名為 `Windows`、且包含 `System32`、`Prefetch`
或 `AppCompat` 目錄的目錄。辨識後兩者，也能支援僅包含 Prefetch 或 Amcache 的
針對性收集資料。這會排除像
`AppData\Roaming\Microsoft\Windows` 這樣的使用者設定檔目錄。像
`C:\Windows.old\Windows` 這類巢狀的舊版安裝，會歸屬於外層所收集的磁碟區，而不會成為重複的
電腦。預設的最大深度為 10。這涵蓋了常見的 Velociraptor 路徑，例如：

```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

只有在收集資料的巢狀層級更深時，才需要變更此限制：

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

對於找到的每個系統，電腦名稱依下列順序決定：

1. 最近的上層 `client_info.json` 中的 `hostname`（其值會以 `FQDN` 保留於清單檔中）。
2. 所收集之 `SYSTEM` 登錄區中的 `ComputerName`。
3. 最接近且有意義的收集目錄名稱，會略過磁碟機與 Velociraptor 的 `uploads/auto`
   架構目錄，並移除如 `_F.…` 的後綴。

名稱會被轉換為可安全作為目錄名稱使用的形式。重複的名稱會在其輸出目錄與 `Comp` 值中
加上數字後綴，以便在 JSON 中仍能區分不同的系統。
清單檔會在另一個 `Hostname` 欄位中，完整保留所找到的主機名稱。

列出系統之後，`auto-parse` 會在
`Parser tasks` 正上方顯示一行 `Timezone:`。它顯示所選的輸出時區：預設為 `UTC`，或在使用 `--timezone "+09:00"` 時
顯示如 `UTC+09:00` 的值。標籤為綠色，值則與其他摘要值一樣使用
終端機的預設文字顏色。`-K, --no-color` 會讓此行以純文字顯示。
所收集系統的時區設定可在
[系統資訊](../artifacts/system-info.md)結果中取得。

## 執行的剖析器 { #parsers-it-runs }

某個剖析器失敗並不會中止其他剖析器。`auto-parse` 會為每台找到的
電腦執行下列工作。這些工作使用與手動剖析相同、依來源分組的命令，例如
`parse-ntfs mft`、`parse-registry amcache`、`parse-ese srum` 與 `parse-file prefetch`：

- Amcache
- `$MFT`
- `$LogFile` NTFS redo/undo 記錄
- Prefetch 記錄，以及另外一份 Prefetch 執行時間軸
- USN Journal
- `$I30` 目錄索引
- `$Boot` 開機磁區與 `$Secure:$SDS` 安全性描述元
- SRUM
- LNK 捷徑
- Jump Lists
- ShimCache
- Persistence Hunter（131 項跨來源的持續存取檢查，含允許／封鎖設定檔）
- Defender 隔離中繼資料
- 開啟與另存新檔對話方塊歷程
- 所有登錄機碼與值，包括可復原的已刪除資料
- 服務、RecentDocs、TaskCache、SAM 帳戶、登錄自動啟動位置、UserAssist、BAM、
  已安裝程式、MuiCache、ShellBags 與 USB 裝置
- 系統資訊：名稱、Windows 版本、時區、最後一次關機、設定檔與 SID
- 稽核原則：系統被設定為記錄哪些安全性事件
- 開機設定（BCD 與 BCD-Template）
- 網路設定檔、介面、共用、連接埠 Proxy 與路由
- LSA 機密資訊的名稱與時間，以及使用中的快取網域登入（不會解密任何內容）

Persistence Hunter 會保留所有嚴重性等級（`--min-severity info`），包括部分符合允許清單的項目。
`LOW` 是一種需要調查的分類；缺少簽章中繼資料時，惡意命令也可能被歸為此類。
自動清冊不會以手動命令預設的 medium 門檻捨棄這些項目。

獵捕器在每個自動剖析器程序中只使用一個工作執行緒。這可避免 `auto-parse -T <N>`
的程序預算在每個子程序中再乘上一個與 CPU 數量相當的執行緒池。獨立執行的
`persistence-hunter -T <N>` 則控制獵捕器自身的工作執行緒。

CSV 的 `$MFT` 輸出包含所有時間戳記與短檔名。登錄檔輸出包含原始資料，以及
`REG_QWORD` 值的 FILETIME 解讀。JSON 與 JSONL 的 SRUM 輸出包含值狀態
中繼資料。Defender 檔案與常駐的 `$MFT` 資料流**不會**自動**擷取**。

未被收集的跡證屬於正常的 `skipped` 工作，而非失敗。

## 輸出配置 { #output-layout }

結果依電腦與調查類別分類整理。類別目錄使用
PascalCase，例如 `Accounts` 與 `FileAndFolderOpening`。Amcache 與 Jump Lists 的資料表會
直接放在其類別中，並加上檔名前綴。電腦目錄名稱保留所找到時的拼寫。既有的輸出
目錄不會被遷移；從舊的命名方式切換時，請選擇新的輸出目錄。每個剖析器都會在其結果旁寫入一個
`<artifact>-logs.txt` 檔案，即使該跡證未被收集也是如此。
由於每個跡證的欄位不同，CSV 會保留各自獨立的資料表：

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   └── Registry-RemoteAccess-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```

| 類別 | 跡證 |
|---|---|
| `Accounts` | SAM 本機帳戶、LSA 機密資訊的名稱與時間 |
| `Applications` | 應用程式歷程（第三方工具的痕跡） |
| `Applications` | 已安裝程式 |
| `ExternalDevices` | USB 裝置與磁碟區 |
| `FileAndFolderOpening` | Jump Lists、LNK 捷徑、Microsoft Office、MRU 清單、開啟與另存新檔對話方塊、RecentDocs、ShellBags |
| `FileExistence` | Amcache、ShimCache |
| `FileSystem` | `$Boot`、`$I30`、`$LogFile`、`$MFT`、`$Secure:$SDS`、USN Journal |
| `NetworkActivity` | 網路設定檔、介面、共用、連接埠 Proxy 與路由；SRUM 連線、流量與推播通知 |
| `LateralMovement` | 使用者對外的遠端桌面、PuTTY 與 WinSCP 連線、對應的磁碟機與共用 |
| `Persistence` | 開機設定（BCD）、登錄自動啟動項目、服務、TaskCache |
| `ProcessExecution` | 應用程式相容性、BAM 與 DAM、FeatureUsage 與釘選項目、MuiCache、Prefetch、Prefetch 時間軸、SRUM 應用程式／CPU 活動、UserAssist |
| `Registry` | 一般的登錄區機碼與值，包括復原的記錄 |
| `SecurityProducts` | Defender 隔離中繼資料與選擇性擷取的內容；安全性設定（Defender、UAC、認證保護、RDP、防火牆、記錄） |
| `SystemInformation` | 名稱、Windows 版本、時區、最後一次關機、設定檔、SID 與稽核原則；SRUM 能源／容量、身分對應表與未知提供者 |

類別有助於整理調查工作；但並不能證明特定活動。例如，
單憑 Amcache 與 ShimCache 項目並不能確立有執行過。SRUM 記錄的是資源使用情形；
在對程式是否曾執行做出結論之前，請先參閱跡證指南。

CSV、JSON 與 JSONL 使用相同的類別。每個剖析器工作各有一個 `<artifact>-logs.txt` 檔案，其中包含
確切的命令與訊息；例如 `HOST1/FileSystem/MFT-logs.txt` 與
`HOST1/ProcessExecution/Prefetch-logs.txt`。JSON 與 JSONL 使用相同的記錄檔名。
Amcache 與 Jump Lists 各有一個共用的記錄檔，放在其帶前綴的資料表旁。SRUM 提供者資料表會直接放在
`NetworkActivity`、`ProcessExecution` 與 `SystemInformation` 中，並共用一個
`SystemInformation/SRUM-logs.txt` 檔案。不會建立 `logs/` 子目錄。

預設的 CSV 檔名為 `Registry-SAM.csv`、`LNK.csv`、`Registry-RecentDocs.csv`、`Registry-ShimCache.csv`、`LogFile.csv`、
`I30.csv`、`MFT.csv`、`UsnJournal.csv`、`Boot.csv`、`SecureSDS.csv`、`Registry-Networks.csv`、`Registry-Persistence.csv`、`Registry-Services.csv`、`Registry-TaskCache.csv`、`Registry-BCD.csv`、
`Registry-SecuritySettings.csv`、
`PrefetchTimeline.csv`、`Prefetch.csv`、`Registry-UserAssist.csv`、`Registry-AppCompat.csv`、`Registry-Office.csv`、`Registry-FeatureUsage.csv`、`Registry-AppHistory.csv`、`Registry-MRU.csv`、`Registry-SystemInfo.csv`、`Registry-RemoteAccess.csv`、`Registry-OpenSaveMRU.csv`、`Registry-InstalledPrograms.csv`、`Registry-BAM.csv`、`Registry-MuiCache.csv`、`Registry-ShellBags.csv`、`Registry-USB.csv`、`Registry-SecuritySettings.csv`、`Registry-AuditPolicy.csv`、`Registry-Dump.csv` 以及
`DefenderQuarantine.csv`。從登錄檔衍生的資料表使用 `Registry-` 前綴，包括
`Registry-ShimCache.csv`、`Registry-UserAssist.csv`、`Registry-BAM.csv` 與 `Registry-SystemInfo.csv`。
一般的登錄區機碼與值位於 `Registry-Dump.csv`。Amcache 資料表為
`FileExistence/Registry-Amcache-Devices.csv`、`Registry-Amcache-Files.csv`、
`Registry-Amcache-Findings.csv` 與 `Registry-Amcache-Summary.csv`。Jump Lists 使用
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` 與
`JumpLists-CustomDestinations.csv`。不會有 Amcache 或 Jump Lists 子目錄。
SRUM 資料表以 `SRUM-` 檔名直接放在其類別之下，例如
`NetworkActivity/SRUM-NetworkConnections.csv`。
請參閱 [SRUM 完整配置表](../artifacts/srum.md#categorized-auto-parse-output)。

記錄檔名與首字母大寫的跡證名稱一致，例如 `Registry-SAM-logs.txt`、`PrefetchTimeline-logs.txt`
與 `Registry-UserAssist-logs.txt`。JSON/JSONL 檔案使用相同的主檔名，例如 `MFT.jsonl`；
記錄的 `Artifact` 識別碼在查詢時為 `mft`。多資料表的輸出會在其資料表旁放置
`Registry-Amcache-logs.txt` 或 `JumpLists-logs.txt`。共用的 SRUM 記錄檔
位於 `SystemInformation`。


`manifest.json` 會記錄所找到的原始 `Hostname`、唯一的輸出／記錄 `Comp`
身分、主機名稱的出處、證據路徑、跡證 `Category`、命令列、執行時間、結束
代碼、結果路徑、記錄檔路徑、所要求的 JSONL 時間軸與任何匯出錯誤、是否要求擷取 Defender，
以及每個工作是
成功、略過還是失敗。

Auto-parse 不會寫入任何絕對路徑，因此分享結果時不會暴露證據、
結果或 Juntei 在分析系統上的位置：

- 證據路徑是相對於證據目錄的路徑。終端機會列出每台電腦及其在
  證據中的目錄（`HOST1: ./HOST1_F`），剖析器則在證據目錄中執行，並
  接收這類路徑。結果中的來源路徑欄位（`SourceFile`、`SourcePath`、…）、記錄檔，
  以及清單檔中的 `EvidenceRoot` 與 `WindowsDirectories`，會顯示每個檔案在
  證據中的位置（`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`）。
- 清單檔與記錄檔中的結果路徑（`Output`、`OutputFiles`、`Log`、`JSON-Timeline`、
  `Command` 中的輸出路徑）以及 Defender 的 `ExtractedFile`，均相對於輸出目錄
  （`./HOST1/ProcessExecution/Prefetch.csv`；若時間軸位於輸出目錄旁則為 `../siem.jsonl`）。`Command`
  以程式名稱（`juntei`）開頭；其中的證據路徑相對於證據
  目錄，輸出路徑則相對於輸出目錄。
- `EvidenceDirectory` 與 `OutputDirectory` 是這兩個目錄的名稱。`ManifestVersion` 2
  標示以輸出目錄為基準記錄結果路徑的清單檔；較早的清單檔沒有
  `ManifestVersion`，且依提供給 `-o` 的原樣記錄路徑。使用 `-C` 重新執行時兩者皆可讀取。
- 終端機顯示輸出或時間軸路徑時，若為相對路徑則依原樣顯示；若為絕對路徑，則顯示
  從目前目錄起算的路徑（`../results/`）。

在平行程序數量之後，終端機會顯示所找到之 Windows 系統的檔案總大小
（`Total filesize: 26.1 MiB`）。

進度訊息會顯示目前處理中的電腦與跡證，後接大約的輸入
大小，例如 `HOST1 / prefetch (~2.0 MiB)`。一組檔案會使用其合計大小；每個
MFT、LogFile 或 USN Journal 工作則顯示其個別的輸入大小。估計值不包括輔助的交易
記錄與補充資訊檔案，也無法預測剖析後輸出的大小。

`manifest.csv` 以適合試算表的資料表形式提供相同的工作索引，每個
剖析器工作一列。它包含 `Comp`、原始 `Hostname`、`FQDN`、`HostnameSource`、`EvidenceRoot`、
`Artifact`、`Category`、`OutputType`、`Purpose`、`Status`、`DurationMs`、`ExitCode`、`Output`、`OutputFiles`、`Log`、
`Error` 與 `Command`。`Purpose` 對分類後的結果為 `artifact`，對搭配 CSV 使用的額外原生 JSONL 處理則為
`json_timeline`。`OutputType` 是剖析器實際的輸出格式。
`Command` 是執行檔及其引數所組成的 JSON 陣列，保留空白與引號。
對於 SRUM，`Output` 是電腦目錄，`OutputFiles` 是其確切
提供者輸出路徑的 JSON 陣列，而 `Category` 則指出共用記錄檔的位置（`SystemInformation`）。
檔案會個別分類；`OutputFiles` 也會出現在 JSON 清單檔中。

將 `Status` 篩選為 `skipped` 或 `failed`，即可找出需要檢查的工作；略過之工作的 `Error`
可能為空白，說明則記載於其 `Log` 檔案中。

終端機摘要包含目前輸出目錄下一般檔案的總數與總大小，
包括兩份清單檔、剖析器記錄檔以及擷取出的檔案。不會追蹤符號連結。
大小會使用與 `Total filesize` 相同的格式化方式自動調整單位，
例如 `KiB`、`MiB` 與 `GiB`：

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

當收集到多個來源資料庫時，Amcache 與 SRUM 資料表會使用 `-Source1`、`-Source2` 等。
共用的剖析器記錄檔會記錄原始路徑與最終路徑的對應，而 JSON/JSONL
記錄則保留來源出處。CSV 的 Amcache／Jump Lists 工作會在
清單檔的 `OutputFiles` 中列出確切的資料表路徑；其 `Output` 則為調查類別目錄。JSON 與 JSONL 的
Amcache 與 Jump Lists 匯出本來就是單一檔案，分別命名為 `Registry-Amcache.json[l]` 與
`JumpLists.json[l]`。合併的 `-j` 時間軸保留相同的記錄內容與 `Artifact` 值。
`--clobber` 會使用先前的清單檔清除已記錄的輸出（包括較早的名稱），
同時保留不相關的類別檔案。一般的單一磁碟區收集資料會將 `$MFT` 與 USN Journal 記錄直接寫入
`FileSystem/MFT.csv` 與 `FileSystem/UsnJournal.csv`。額外收集的磁碟區則使用編號的
同層檔案，例如 `FileSystem/MFT-2.csv` 與 `FileSystem/MFT-2-logs.txt`。

## JSON 與 SIEM 匯入 { #json-and-siem-ingestion }

每筆 JSON 或 JSONL 記錄都會加上兩個最上層欄位：

- `Comp`：來源系統。
- `Artifact`：產生該記錄的 Juntei 剖析器，例如 `mft` 或 `prefetch-timeline`。

`-j, --json-timeline <FILE>` 會額外建立一個 JSONL 檔案，不受個別
結果格式影響。所有分類後的輸出及其記錄檔均會保留：

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

檔名會依提供的原樣使用；相對路徑以目前的工作目錄為基準。
它可以位於輸出根目錄、其他非保留的輸出子目錄，或輸出
目錄之外。證據、清單檔、各電腦的結果／記錄檔以及剖析器的暫存目錄都不能
作為目的地。既有的檔案需要 `-C`；匯出失敗時會保持其原狀。

在 **CSV 模式**下，Juntei 會額外執行一次原生 JSONL 處理，讓 SIEM 檔案保留
數值與布林型別、巢狀資料、原始欄位，以及 CSV 資料表可能省略的中繼資料。這些
額外工作共用相同的 `-T` 並行上限，不會再次擷取隔離內容，並且
有各自的 `*-json-timeline-logs.txt` 附屬記錄檔。兩份清單檔都會區分其用途與
輸出格式。暫存的 JSONL 檔案會在成功執行後移除；若剖析器或匯出失敗則會保留。
請預留額外的處理時間與暫存磁碟空間。

在 **JSON 模式**下，Juntei 會將既有的結果串流轉換為精簡的 JSONL，而不會重新剖析
證據。Amcache 報告會展開為與原生 JSONL 相同的個別摘要、發現、檔案與裝置
記錄。在 **JSONL 模式**下，既有的帶註記記錄會被直接合併。
所有模式都使用相同的 SIEM 記錄結構。此資料集保留原生的時間戳記與跡證
欄位；它不會自行建立共通的事件時間戳記，也不會對異質的跡證進行全域排序。

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

`manifest.json` 會記錄所要求的 `JSON-Timeline` 路徑以及任何 `JSON-TimelineError`。其工作
記錄與 `manifest.csv` 仍指向所保留的跡證輸出。成功完成、僅用於 CSV 時間軸的
工作會參照最終的 JSONL 檔案，而非已刪除的暫存檔案。若有任何剖析器失敗，
執行會以失敗狀態結束，且時間軸只包含成功的結果；在將其視為完整之前，請先檢查清單檔
與類別記錄檔。檔案數量／大小摘要涵蓋輸出目錄內的檔案；
位於外部的時間軸則另外以 `JSONL timeline: <FILE>` 回報。

## 效能 { #performance }

剖析器工作是各自獨立的程序，因此 CPU 負載重的跡證可以同時執行，而某個
剖析器中的損毀也不會終止整個執行。Juntei 預設使用邏輯 CPU 數量。當證據位於慢速磁碟，
或系統記憶體不多時，請限制並行數量：

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## 用法 { #usage }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK and Jump List files [default: 1252]

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

可行時請使用新的輸出目錄。`-C` 只會移除或覆寫此次執行所產生的確切
目標，因此現已不存在的跡證不會留下過時的結果。它不會刪除
不相關的檔案或整個輸出樹狀結構。它也會在剖析之前，移除每台電腦的
`SecurityProducts/DefenderQuarantined/` 目錄，以及先前扁平配置中保留的 `defender-quarantined/`
目錄，無論新的執行是否使用 `-x`。

從較舊的扁平配置切換時，請使用全新的輸出目錄。先前的 CSV/JSON 檔案
不會被自動搬移或刪除。

!!! danger "請將擷取出的 Defender 內容視為惡意程式"
    `-x, --extract-defender-quarantined` 會將隔離內容寫入磁碟，放在每台電腦的
    `SecurityProducts/DefenderQuarantined/` 目錄之下。復原出的二進位資料流以 `.quarantined` 結尾，且 Juntei
    會移除其執行權限。每個二進位安全性描述元也會另外產生一份可讀的 `.security_descriptor.txt`
    報告，內含擁有權、權限與 SDDL。切勿執行復原出的酬載。請使用隔離的
    分析系統，並預期防毒或 EDR 軟體會對復原出的內容發出警示或將其移除。

報告欄位及其鑑識上的限制，請參閱[解讀復原的安全性描述元](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)。

Auto-parse 也會讀取使用者的遠端連線（RDP、PuTTY、WinSCP 與對應的磁碟機）。
