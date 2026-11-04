# Persistence Hunter { #persistence-hunter }

持續存取讓程式能在重新開機、登入、排程觸發或其他事件之後再次執行。攻擊者可以利用一般的 Windows
設定達成此目的。許多合法程式使用的正是完全相同的機制，因此找到持續存取項目只是分析的起點。

`juntei persistence-hunter` 是
[Hexastrike 的 PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper) 的原生 Rust 移植版本，
提供 131 項檢查：PyrsistenceSniper 的 130 項檢查目錄，加上原生的「開始」功能表 COM 檢查。

關於每個機制、其證據位置，以及如何調查發現項目的說明，請參閱
[131 項檢查的持續存取指南](../persistence-hunter/index.md)。
[持續存取致謝](../persistence-hunter/acknowledgements.md)說明了此檢查目錄背後的 Python 與
PowerShell 專案。

## 用法 { #usage }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei persistence-hunter -d <EVIDENCE> [OPTIONS]

Input:
  -d, --directory <DIR>  Evidence directory (a mounted Windows volume or a collection of hosts)
  -f, --file <FILE>      One registry hive (filesystem checks need a directory)
      --no-logs          Read dirty hives without replaying transaction logs

Filtering:
      --profile <FILE>               Replace the bundled allow/block profile with a YAML profile
      --min-severity <MIN_SEVERITY>  Minimum classification to include: info, low, medium, high [default: medium] [possible values: info, low, medium, high]
      --check <ID>                   Run only these check or ATT&CK IDs (comma separated; repeatable) [alias: --technique]

General Options:
  -h, --help         Show the help menu
      --list-checks  Show every available check and its ATT&CK technique

Analysis:
      --no-timeline  Skip MFT and event-log timestamp correlation
      --mft <FILE>   MFT to use for timestamp correlation (one host only)

Performance:
  -T, --threads <N>  Worker threads (default: logical CPU count)

Output:
  -o, --output <FILE>              Save findings to a file
  -t, --output-type <OUTPUT_TYPE>  Output type: csv (default), json, jsonl; without -o, write data to stdout [possible values: csv, json, jsonl]
      --coverage <FILE>            Save per-check coverage and errors to CSV
  -C, --clobber                    Overwrite output files

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## 第一次掃描 { #first-scan }

```bash
juntei persistence-hunter -d evidence/ -o PersistenceHunter.csv --coverage Coverage.csv
```

證據可以是已掛載的 Windows 磁碟區、Velociraptor 收集資料，或包含多台已收集電腦的目錄。分析期間
各電腦會分開處理，且每個結果都包含 `Comp`。解析 Windows 路徑時，會解碼收集過程中經跳脫處理的檔案名稱。

針對單一登錄區：

```bash
juntei persistence-hunter -f SOFTWARE -o software-persistence.csv --no-timeline
```

單一登錄區無法提供需要其他登錄區或檔案系統證據的檢查。在將空白結果視為有意義之前，請先閱讀涵蓋範圍報告。

列出檢查 ID 及其 ATT&CK 對應：

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,windows_service_image_path -o focused.csv
```

請使用 `--list-checks` 印出的檢查 ID，或 `T1547.001` 之類的 ATT&CK ID。
`T1547` 之類的上層 ID 會選取其子技術。`--technique` 是 `--check` 的別名。
每項檢查都會在發現項目中附上其技術、說明與參考連結。

## 平行掃描 { #parallel-scans }

此命令使用共用的工作執行緒集區，預設數量為邏輯 CPU 數。設定
`-T, --threads <N>` 可選擇工作執行緒數量，或使用 `-T 1` 進行單執行緒掃描：

```bash
juntei persistence-hunter -d evidence/ --min-severity info -T 4 -o PersistenceHunter.csv
```

證據探索、獨立的檢查、酬載中繼資料，以及 MFT/EVTX 時間軸來源會平行執行。一台主機已載入的登錄區
會以唯讀方式共用，同一酬載的快取中繼資料只會初始化一次。主機會一次載入一台，以避免登錄區記憶體
隨收集資料規模倍增。在不同的執行緒數量下，發現項目、涵蓋範圍、時間戳記的選擇與警告順序都保持相同。

`auto-parse` 本身已經會平行執行剖析器處理程序，因此它為每個 `persistence-hunter` 子處理程序
分配一個工作執行緒。它自己的 `-T` 選項控制同時執行的剖析器處理程序數量。

## 需要收集的證據 { #evidence-to-collect }

| 來源 | 檢查範例 |
|---|---|
| `SYSTEM` 及相鄰的交易記錄檔 | 服務、服務復原命令、LSA 套件、開機設定、驗證提供者 |
| `SOFTWARE` 及相鄰的交易記錄檔 | Run 機碼、Winlogon、IFEO、COM 註冊、TaskCache、Office 設定 |
| 每位使用者的 `NTUSER.DAT` 與 `UsrClass.dat` | 使用者自動啟動項目、環境變數、COM 覆寫、Office 設定 |
| `SAM` 與 `SOFTWARE` | RID 異常、隱藏帳戶、不尋常的本機 Administrators 成員資格 |
| `Windows/System32/Tasks` | 排程工作命令與 COM 處理常式；與 TaskCache 的比較 |
| 各使用者「開始」功能表中的 `.library-ms` 檔案與以 CLSID 命名的資料夾 | COM 啟用觸發條件的候選項目，與相符的使用者／電腦類別註冊結合 |
| 「啟動」資料夾、PowerShell 設定檔、Office 範本、Terminal 設定 | 由使用者或系統觸發條件執行的檔案與命令 |
| WMI `OBJECTS.DATA` | 事件取用者執行個體，以及篩選器與取用者的繫結 |
| BITS `qmgr.db` 或 `qmgr*.dat` | 復原的通知命令與擁有者 SID |
| `$MFT` 與相關的 EVTX 檔案 | 獨立的時間戳記關聯 |
| 被參照的執行檔、DLL 與目錄（catalog）檔案 | SHA256 雜湊值，以及描述性的 PE／憑證中繼資料 |

有交易記錄檔時會加以重播。`--no-logs` 會依收集時的原樣讀取登錄區；未重播的 dirty 登錄區可能遺漏
近期的變更，並會在涵蓋範圍中回報。現役系統的檢查使用目前的控制集。DSRM 後門檢查另外會在存在時檢查
`ControlSet001`、`ControlSet002` 與 `CurrentControlSet`；在將該設定描述為生效之前，請將其回報的
路徑與 `SYSTEM\\Select` 進行比對。其他歷史控制集可以使用個別的登錄剖析器命令檢查。

登錄區讀取共用每台電腦 1 GiB 的輸入預算。電腦登錄區會先於使用者登錄區嘗試讀取；無法納入預算的
輸入或交易記錄檔會在部分涵蓋範圍中列出。其他可讀取的登錄區與以檔案為基礎的檢查會繼續進行。只有標準
的登錄區位置或明確提供的匯出檔案才能確立登錄區身分；位於使用者暫存目錄中的副本，不能取代該使用者
真正的登錄區。SOFTWARE 中記錄的原始系統磁碟機會對應到所擷取的系統磁碟區，因此即使將原始的 `C:`
磁碟區以不同的證據磁碟機代號掛載，也不會捨棄酬載中繼資料。

`auto-parse` 也會執行此獵捕工具，並將 `persistence/PersistenceHunter.csv`（或 `.json`/`.jsonl`）
儲存在每台電腦的目錄中，旁邊附有 `PersistenceHunter-logs.txt`。其發現項目也會進入選用的
`-j <FILE>` SIEM 時間軸。自動解析使用 `--min-severity info`，以保留完整的設定清查，包括符合
允許規則的項目。

## COM 註冊與啟用觸發條件 { #com-registrations-and-activation-triggers }

`scheduled_task_files` 檢查會使用工作 XML 的 `Actions Context` 與相符的 `Principal id`，選取
每個動作的主體。群組主體會保留為群組；它無法確立會使用哪一位個別使用者的類別註冊。

COM 處理常式的候選項目會在原生與 32 位元類別檢視中分別解析。只會考慮相符使用者所擷取的
`UsrClass.dat`/`NTUSER.DAT` 以及電腦的 `SOFTWARE`。SID 主體會透過所擷取的 `ProfileList`
路徑進行結合。帳戶名稱（包括 `HOST\user` 與 `DOMAIN\user`）會優先選擇相符的 `user.DOMAIN`
設定檔，而非單純的 `user` 設定檔。本機主機別名與 `.\user` 會使用所擷取的主機名稱進行字尾比對；
完全相符的字尾優先於其他主機別名。只有在沒有任何限定設定檔相符時，才會考慮單純的設定檔。每個這類
比對結果都會在 `Description` 中明確標示為**資料夾名稱啟發式判斷**；它並不能證明帳戶的擁有權或
網域成員資格。模稜兩可的比對結果會維持未解析，並同時在發現項目的說明與涵蓋範圍中回報；限定設定檔
之間的歧義不會觸發改用單純設定檔的備援做法。這並不依賴 `Volatile Environment`，該機碼通常只存在於
記憶體中，不會與登錄區一起儲存。重複的使用者 CLSID 會遮蔽電腦 CLSID 的子項目。群組主體使用電腦的
COM 候選項目，並在發現項目的說明中保留此限制；一般的群組主體本身不會讓涵蓋範圍變為部分，也不會
佔用錯誤摘要的項目。

解析器會追蹤 `InprocServer32` 與 `LocalServer32`，使用受控程式碼的 `CodeBase` 路徑（包括版本
子機碼），並將 AppID 的 `LocalService` 與所收集的 SYSTEM 服務 `ImagePath` 進行結合。AppID 的
執行檔名稱關聯會被保留，但基本名稱無法確立執行檔的完整路徑。本機的 `file:` CodeBase URL 會被
解碼；遠端 URL 只會被記錄，不會實際存取。遺失的註冊會保留其 CLSID，並帶有
`ResolutionStatus: unresolved_com_class`、空白的路徑／雜湊欄位，以及未知的 `Exists`。

`start_menu_com` 檢查會讀取各使用者「開始」功能表中 `.library-ms` XML 的 URL，並收集以 CLSID
命名的資料夾，包括空目錄。它會回報觸發路徑、CLSID、候選伺服器與登錄來源資訊。僅收集檔案的擷取方式
無法保留空目錄，而遺失的使用者登錄區會讓各使用者的註冊查詢無法完整進行。不會追蹤符號連結。這些是
設定上的候選項目：架構、權限提升、UAC、原則與啟用脈絡都可能影響某個註冊是否實際被使用。請以處理程序、
模組載入或其他獨立證據佐證執行。

## 分類與設定檔 { #classification-and-profiles }

預設的最低分類為 `medium`。若要檢視完整的清查，包括符合內建允許清單的項目：

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o all-persistence.csv
```

!!! warning "檢視不完整的收集資料時，請使用所有嚴重性等級"
    即使攻擊者命令的簽署者缺失或不同，範圍寬鬆的允許規則仍可能與其部分相符。上游專案將部分相符
    歸類為 `LOW`，而手動執行命令時預設的 `medium` 門檻會將其隱藏。這可能影響位於 Windows 目錄下的
    惡意排程工作。進行完整調查時請使用 `--min-severity info`，並一併檢視 `PolicyReason`、
    `SignerStatus`、命令與來源證據。`auto-parse` 已經會保留這些項目。

| 分類 | 意義 |
|---|---|
| `HIGH` | 某條已設定的封鎖規則完全相符 |
| `MEDIUM` | 沒有任何允許或封鎖規則相符 |
| `LOW` | 某條允許規則的硬性條件相符，但簽署者中繼資料不相符或無法取得 |
| `INFO` | 某條允許規則完全相符 |

這些是原則分類，而非經過量測的遭入侵機率。預設的 `MEDIUM` 項目並不能證明是惡意程式。`INFO`
並不能確立檔案具有有效的簽章，也不能確立攻擊者未曾濫用它。部分符合允許規則的項目低於預設門檻，
因此在檢視稀疏的收集資料或調整允許清單時，請加上 `--min-severity low` 或 `info`。

使用 `--profile profile.yaml` 提供 YAML 設定檔。它會**取代**內建的設定檔。所選檔案中的全域規則與
各檢查規則會被合併；任何完全相符的封鎖規則都會優先。例如：

```yaml
allow: []
block:
  - reason: Example known-bad command fragment
    value_matches: 'example-malicious-loader\.exe'
checks:
  windows_service_image_path:
    enabled: true
    allow:
      - reason: Organization's expected service path
        value_matches: '^C:\\Program Files\\ExampleCorp\\Agent\\agent\.exe$'
        signer: ExampleCorp
```

支援的規則條件為 `path_matches`、`value_matches`、`hash`（64 位十六進位的 SHA256 數字）、
`not_lolbin` 與 `signer`（不區分大小寫的子字串）。所有列出的硬性條件都必須相符。簽署者中繼資料
則是上方所述的軟性條件。規則運算式使用 Rust 的規則運算式語法：不支援環顧（lookaround）與反向參照。
未知的檢查 ID、無效的運算式與未知的設定檔欄位會產生錯誤，而不會在未告知的情況下削弱原則。

## 輸出欄位 { #output-fields }

CSV 是預設的檔案格式。使用 `-t json` 可取得易讀的 JSON 物件，使用 `-t jsonl` 則可取得精簡的
每行一個物件的輸出。未指定 `-o` 時，明確指定 `-t` 只會將資料寫入 stdout；涵蓋範圍警告會輸出至
stderr。兩個選項都未指定時，Juntei 會印出摘要以及最多 30 個候選項目。

| 欄位 | 如何判讀 |
|---|---|
| `Comp`、`CheckID`、`Technique`、`MitreID` | 該列描述的是哪台電腦與哪種持續存取機制 |
| `Path`、`Value` | 登錄位置、檔案、工作或復原的記錄，以及其設定的內容 |
| `Description`、`References` | 此機制為何重要，以及可在何處閱讀更多資訊 |
| `AccessGained` | 此機制通常的範圍（`USER` 或 `SYSTEM`；BITS 也可能識別出 `LOCAL_SERVICE`/`NETWORK_SERVICE`）；並不能證明特定命令取得了該權限 |
| `Severity`、`PolicyReason` | 設定檔分類、相符原則的說明，以及任何部分相符或簽署者未驗證的限定說明 |
| `SourceFile`、`RecordStatus` | 確切的收集來源（其路徑以所提供的輸入開頭，auto-parse 則為 `./HOST/...`），以及配置／重播／復原脈絡；請據此追溯至原始證據 |
| `KeyLastWrite` | 登錄**機碼**的時間戳記。它並不標示個別值的日期 |
| `LastChange`、`ChangeSource`、`ChangeEvidence` | 獨立關聯得出的變更時間、來源證明，以及關聯是否成功 |
| `ResolvedPath`、`ResolutionStatus`、`Exists` | 酬載路徑與離線解析結果；未收集的證據為未知，而非 `false` |
| `SHA256` | 可讀取時，所收集之酬載位元組的 SHA256 |
| `Signer`、`SignerStatus` | 描述性的內嵌憑證／目錄中繼資料及其可用性；不會驗證憑證信任與簽章有效性 |
| `IsLolbin`、`Launcher` | 所設定的啟動程式或酬載是否位於內建的 LOLBAS 清單中，以及代理執行酬載時所使用的啟動程式 |
| `IsBuiltin`、`IsInOS-Directory` | 常見 Windows 命令與系統目錄的脈絡；兩個欄位都無法確立合法性 |

`rundll32.exe` 命令可以指名 DLL 酬載。Juntei 在解析酬載並計算其雜湊值的同時，也會保留啟動程式。
位於所收集電腦之外的參照、遺失的磁碟機、不區分大小寫時模稜兩可的比對，或未解析的環境變數，都不會
以分析人員機器上的檔案替代。

## 時間戳記與限制 { #timestamps-and-limitations }

此獵捕工具不會將分析用虛擬機器上複製檔案的修改時間當作攻擊時間戳記。在可用時，它會將登錄寫入與
Sysmon 事件 13、工作安裝與 Security 4698 或 Task Scheduler 106、服務安裝與 System 7045，以及以
檔案為基礎的設定與原始的 `$MFT` 時間戳記進行關聯。來源資訊會顯示在 `ChangeSource` 中。
事件可以標示設定變更的日期，但不能證明所設定的酬載曾經執行。本機 System 事件 6011 可以證明較早的
電腦名稱。此獵捕工具會追蹤以所收集電腦為錨點、經過驗證的重新命名鏈，保留其目前的 `Comp` 值，同時在
`ChangeSource` 中記錄原始事件的電腦與重新命名記錄的證明。未經證明的外來名稱、封存的記錄、轉送的記錄，
以及其他 Windows 安裝，都不會透過此規則被接受。

所擷取的 ComputerName/ActiveComputerName、TCP/IP Hostname/NV Hostname，以及用戶端資訊中的
主機名稱／FQDN 會提供主機別名，涵蓋 DNS/NetBIOS 的差異與擱置中的重新命名。若沒有擷取到任何身分，
主要記錄中的單一名稱可以支援日期標示，並附上明確的「身分未驗證」來源資訊與涵蓋範圍警告。多個未驗證的
名稱不會被歸屬到該主機。被拒絕的 Computer 名稱會以關聯涵蓋範圍限制的形式回報；其通道仍會被標示為已收集。

使用 `--mft <FILE>` 為單一電腦選擇外部收集的 MFT。使用 `--no-timeline` 可略過時間戳記關聯。
`NOT_RUN`、`NOT_APPLICABLE`、`NO_ARTIFACT`、`NO_MATCH` 與 `REJECTED` 說明 `LastChange` 為空的原因；
空白欄位並不是該項目無害的證據。

WMI 與 BITS 的復原是有範圍限制的擷取（carving），而非完整的資料庫交易剖析器。復原的字串或繫結可能
包含舊的／已刪除的資料；這些記錄必須加以佐證。單憑 WMI 類別結構描述不會被計為訂閱。沒有對應已收集
工作 XML 檔案的 TaskCache 項目是有待檢視的不一致之處；分類收集資料可能只是遺漏了該檔案。
目錄（catalog）與 PE 中繼資料未經驗證，且可能不存在於收集資料中。協助工具檔案替換檢查依賴合適的參考
簽章中繼資料；缺少目錄證據不得導致每個協助工具執行檔都被標記。

每項檢查都會記錄涵蓋範圍，包括沒有任何發現項目的檢查。`complete` 表示已讀取適用的已收集來源；
它並不能證明收集資料是完整的，也不能證明該電腦沒有其他持續存取機制。`partial`、`not_collected`、
`failed` 與 `disabled` 說明其限制。請將 `--coverage Coverage.csv` 與剖析器記錄連同你的發現項目一起保存。

此移植版本使用 Juntei 的 CSV/JSON/JSONL 格式。它不會呼叫上游的 Python 程式、不會查詢線上擴充資訊
服務，也不會重現上游的 HTML/XLSX 使用者介面。

## 延伸資訊 { #further-details }

- [PyrsistenceSniper 的實作與設定檔](https://github.com/hexastrike/PyrsistenceSniper)
- [MITRE ATT&CK 持續存取技術](https://attack.mitre.org/tactics/TA0003/)
- [Microsoft：Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
- [登錄自動啟動項目清查](../artifacts/persistence.md)
- [服務](../artifacts/services.md)與 [TaskCache](../artifacts/taskcache.md)
