# 應用程式歷程 { #application-history }

## 工具會留下什麼 { #what-tools-leave-behind }

許多第三方工具會將設定與歷程保存在使用者的 `NTUSER.DAT` 中。其中有幾種正是攻擊者
會使用的工具，或是用來在帶走資料之前先行整備資料的工具：Sysinternals 工具、壓縮程式、
雲端同步用戶端與遠端存取工具。Juntei 會讀取以下項目：

| 應用程式 | `HKCU\Software` 之下的機碼 | 顯示的內容 |
|---|---|---|
| Sysinternals | `Sysinternals\<tool>` | 使用者已接受授權（`EulaAccepted`）的每個工具：PsExec、ProcDump、Sysmon…… |
| WinRAR | `WinRAR\ArcHistory`、`DialogEditHistory\ArcName`、`DialogEditHistory\ExtrPath` | 開啟過的壓縮檔；在其對話方塊中輸入的壓縮檔名稱與解壓縮資料夾 |
| 7-Zip | `7-Zip\Compression\ArcHistory`、`Extraction\PathHistory`、`FM\FolderHistory`、`FM\CopyHistory`、`FM\PanelPath0/1` | 建立的壓縮檔；解壓縮資料夾；在其檔案管理員中瀏覽過及複製目的地的資料夾 |
| WinZip | `Nico Mak Computing\WinZip\filemenu`、`extract`、`mru\archives` | 其「檔案」功能表上的壓縮檔、解壓縮資料夾、最近使用的壓縮檔 |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | 每個帳戶的電子郵件地址、本機資料夾與名稱；同步的媒體櫃（`ScopeIdToMountPointPathCache`）與資料夾（`Tenants\<tenant>`） |
| Adobe Acrobat 與 Reader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`、`cRecentFolders` | 最近開啟的 PDF 檔案與資料夾，包含檔案名稱、大小與日期 |
| TeamViewer | `TeamViewer`（以及舊版的 `Version<N>`） | 所有值：帳戶與使用者名稱、檔案傳輸的起始資料夾（`<partner ID>?<local>|<remote>`）…… |
| CCleaner | `Piriform\CCleaner` | 所有值：設定要清除的項目、`UpdateKey`…… |

<span class="evidence-strength">最佳結論：</span>此使用者曾執行該工具（或接受其授權），
或該工具曾開啟、建立或解壓縮這些壓縮檔與資料夾，或此雲端帳戶曾為該使用者設定。
大多數歷程的項目都沒有各自的時間：只有最新的項目能取得其清單機碼的最後寫入時間。

## 位置 { #location }

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

請收集每位使用者的 `NTUSER.DAT` 及其交易記錄檔，以及系統的 `SOFTWARE`（用於取得使用者的 SID）。

## 分析注意事項 { #analyst-cautions }

- **Sysinternals EULA 接受紀錄。** 當機碼唯一的值是可讀取且設為 `1` 的 `EulaAccepted` 時，
  `FirstUsedUTC` 是依據該機碼最後寫入時間所推估的首次使用時間。被拒絕或已損毀的 EULA
  不會得到此推估值。登錄設定也可能被預先設定、複製或編輯：請以其他跡證佐證執行行為及其時間。
  會在該處保存設定的工具（Process Explorer、Autoruns……）之後會重新寫入該機碼：
  此時 `FirstUsedUTC` 為空，而 `KeyLastWrittenUTC` 只代表最後一次設定變更。以 `-accepteula`
  執行的工具，或以其他使用者身分執行的工具（PsExec 的遠端端以 SYSTEM 執行），會寫入該使用者的機碼。
- **是順序，不是時間。** 清單項目具有順序（`MruPosition` 0 為最新），但只有清單機碼的
  最後寫入時間能標示最新項目的日期。較舊的項目是在更早、時間未知的時點使用的。
- **手動輸入的名稱。** WinRAR 的 `ArcName` 是建立壓縮檔時所輸入的名稱：通常是不含資料夾的檔案名稱。
- **Adobe 路徑**會依 Adobe 寫入的形式保留（`/C/Users/alice/a.pdf` 即
  `C:\Users\alice\a.pdf`）。其 `sDate`（位於 `Details` 中）是 PDF 日期字串，依原樣保留。
- **CCleaner 的 `UpdateKey`** 是不含時區的當地時間；依原樣保留。
- **TeamViewer。** 連線記錄是檔案（`Connections_incoming.txt`、`TeamViewer*_Logfile.log`），
  而非登錄值：請一併收集。
- **不存在。** 使用者與工具都可以清除這些歷程（CCleaner 本身就能做到）。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.App == "Sysinternals" and .FirstUsed != null)'
```

摘要會列出最新的 Sysinternals 首次使用推估時間、各清單最後使用的壓縮檔與資料夾，
以及 OneDrive 帳戶。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`SourcePath` | 主機與 `NTUSER.DAT`。 |
| `User`、`UserSID` | 設定檔資料夾的名稱，以及取自系統 `ProfileList` 的 SID。 |
| `App` | `Sysinternals`、`WinRAR`、`7-Zip`、`WinZip`、`OneDrive`、`Adobe Acrobat Reader`、`Adobe Acrobat`、`TeamViewer`、`CCleaner`。 |
| `Item` | Sysinternals 工具、清單（`ArcHistory`、`ExtrPath`、`FolderHistory`、`filemenu`、`cRecentFiles`……）、OneDrive 帳戶或其同步資料夾的機碼，或值的名稱。 |
| `MRU-Position` | 該項目在清單中的位置，0 為最新。 |
| `Path` | 壓縮檔、資料夾或文件。 |
| `Value` | `EulaAccepted`、OneDrive 帳戶的電子郵件地址或同步媒體櫃的 ID、TeamViewer 或 CCleaner 的值。 |
| `FirstUsedUTC` | 依據 EULA 接受紀錄推估的首次使用時間（請參閱上方注意事項）。 |
| `Details` | OneDrive 帳戶的 `DisplayName`、`UserCID`、`ConfiguredTenantId`……；Adobe 文件的 `Version`、`tFileName`、`uFileSize`、`uPageCount`、`sDate`。 |
| `KeyPath`、`KeyLastWrittenUTC`、`ValueName` | 記錄的來源，以及該機碼最後變更的時間。 |
| `DataRaw` | 無法解碼時，該值的位元組（十六進位）。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡。 |
| `Problems` | 無法解碼或讀取的資料。 |

## 調查策略 { #investigation-strategy }

1. 尋找一般使用者不會執行的 Sysinternals 工具（PsExec、ProcDump、PsLoggedOn、
   SDelete）及其 EULA 時間戳記。
2. 檢視事件前後的壓縮檔名稱與解壓縮資料夾：在資料離開系統之前，於 `ProgramData`、
   `Temp` 或使用者資料夾中建立的壓縮檔。
3. 檢查不屬於組織的 OneDrive 帳戶，以及 TeamViewer 帳戶名稱與檔案傳輸資料夾。
4. 與 Prefetch、Amcache、BAM 與 UserAssist（工具的執行）、`$MFT` 與 USN
   Journal（壓縮檔本身）、LNK 檔案與 Jump Lists（開啟過的文件），以及事件記錄進行關聯分析。

## 延伸資訊 { #further-details }

- [Microsoft：Sysinternals](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
