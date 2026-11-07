# 使用者存取記錄 (UAL) { #user-access-logging-ual }

## 什麼是 UAL { #what-ual-is }

使用者存取記錄（User Access Logging，UAL）是 Windows Server 的功能，自 Windows Server 2012 起
預設即已安裝並啟用。它會將用戶端對伺服器角色所提出的要求彙總後儲存於 ESE 資料庫中。針對每個角色、
用戶端位址與帳戶的組合，它會保存該用戶端在當年度首次與最後一次出現的時間、要求總數，以及當年度
每一天的要求次數。角色包括 File Server（SMB）、Active Directory Domain Services、Active Directory
Certificate Services、DHCP Server、Print and Document Services、Remote Access、Web Server（IIS）
與 Windows Server Update Services。

UAL 對橫向移動調查很有價值：其用戶端資料列會顯示哪些位址與帳戶在哪幾天使用過伺服器，而且往往在
事件記錄早已被覆寫之後仍然保留。

<span class="evidence-strength">最佳結論：</span>位於所記錄位址的用戶端，以所記錄的帳戶通過驗證，
並在每個有計數的日子使用了該角色。

## 位置與收集 { #location-and-collection }

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

`Current.mdb` 保存最近的活動。`SystemIdentity.mdb` 描述伺服器本身、提供各角色的名稱，並列出保存各年度
資料的 `{GUID}.mdb` 檔案（`CHAINED_DATABASES`）。UAL 服務會持續開啟這些資料庫：請使用鑑識收集工具
或從磁碟區快照收集整個 `Sum` 目錄。

!!! warning "Dirty 資料庫"
    從執行中的伺服器收集到的 `Current.mdb` 經常處於 dirty 狀態。Juntei 會依收集時的狀態讀取並加以回報，
    但不會重播 `Svc*.log`：僅存在於交易記錄檔中的活動將會缺漏。

## 分析注意事項 { #analyst-cautions }

- UAL 以天為單位計算要求次數，而不會逐筆記錄每個要求。用戶端資料列僅有的時間是
  `InsertDate` 與 `LastAccess`。
- `DailyAccesses` 的日期是以 UTC 計算的日子。
- `Address` 是伺服器所看到的位址：伺服器與真正的用戶端之間可能隔著 NAT 閘道、Proxy 或 VPN。
  `::1` 與 `127.0.0.1` 代表伺服器本身。
- 請剖析 `Sum` 目錄中的每一個資料庫：同一個用戶端可能同時在 `Current.mdb` 與其所屬年度的
  `{GUID}.mdb` 中都有資料列。
- 沒有資料列並不能證明沒有任何連線：UAL 可以被關閉（`Disable-Ual`，或停止 User Access Logging
  Service），而 dirty 資料庫會缺少最新的變更。
- 若沒有 `SystemIdentity.mdb`，`RoleName` 會採用 Windows Server 為其角色所定的名稱。
- 若 `LastAccess` 為零或無效，Juntei 不會以此判定年份，而會依序使用有效的 `InsertDate` 和年度資料庫中繼資料；若仍無法判定年份，則保留 `DayOfYear`。
- 摘要總數會加總所有來源資料列。`Current.mdb`、年度資料庫及快照可能重疊，因此這些總數不是去除重複後的連線或登入次數。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-ese ual <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Current.mdb, {GUID}.mdb and SystemIdentity.mdb
  -f, --file <FILE>      Parse one UAL database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese ual -d Sum -o ual-csv
juntei parse-ese ual -d Sum -t jsonl \
  | jq 'select(.Table == "CLIENTS") | {Address, AuthenticatedUserName, RoleName, LastAccess}'
```

## 輸出資料表 { #output-tables }

CSV 會為每個資料表各建立一個檔案；資料表為空時，檔案只含標題列。JSON 與 JSONL 資料列帶有
`SourceFile`（資料庫）與 `Table`（該資料表在資料庫中的名稱）。

| CSV 檔案 | 資料表 | 內容 |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | 各角色的用戶端：位址、帳戶、當年度首次與最後一次存取、要求總數與每日要求次數。 |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | 各角色首次與最後一次被使用的時間。 |
| `UAL-DNS.csv` | `DNS` | 位址與主機名稱的配對，以及各配對最後一次出現的時間。 |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | Hyper-V 虛擬機器：識別碼、序號、建立時間與最後作用時間。 |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | 伺服器本身：名稱、網域、Windows 版本、產品類型、硬體、時區與最後一次開機；新的資料列可能代表重新命名、加入網域或升級為網域控制站。 |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | 各年度的 `{GUID}.mdb` 檔案。 |
| `UAL-Roles.csv` | `ROLE_IDS` | 角色 GUID 及其名稱。 |
| `UAL-<table>.csv` | 其他資料表 | Juntei 無法辨識之資料表的所有欄位。 |

## 欄位 { #fields }

| 欄位 | 判讀 |
|---|---|
| `SourceFile` | 讀取該資料列的資料庫。 |
| `RoleGUID`、`RoleName` | 角色，以及取自資料庫旁 `SystemIdentity.mdb` 的角色名稱（若無該檔案則為內建名稱）。 |
| `TenantID` | 用戶端所屬租用戶的 GUID（當角色有回報時）。 |
| `TotalAccesses` | 該用戶端在資料庫所屬年度的要求次數；即 `DailyAccesses` 的總和。 |
| `InsertDate` | 資料列建立的時間：也就是該用戶端當年度在此資料庫中的第一個要求（UTC）。 |
| `LastAccess` | 該用戶端最近一次的要求（UTC）。 |
| `Address` | 用戶端的 IPv4 或 IPv6 位址。 |
| `AuthenticatedUserName` | 用戶端驗證時所用的帳戶（`DOMAIN\user`；電腦帳戶以 `$` 結尾）。 |
| `ClientName` | 用戶端的名稱（當角色有回報時）。 |
| `DailyAccesses` | 有要求的日子及其次數，取自 `Day1` … `Day366` 欄位，並以 `LastAccess` 的年份標註日期：CSV 中為 `2023-01-12: 5, 2023-01-13: 2`，JSON 中為帶有 `Date` 與 `Accesses` 的物件。年份不明的日子會以 `DayOfYear` 取代 `Date`。 |
| `FirstSeen`、`LastSeen` | 角色的首次與最後一次使用（`ROLE_ACCESS`）、DNS 配對最後一次出現的時間（`DNS`），皆為 UTC。 |
| `CreationTime`、`LastSeenActive` | 虛擬機器或 `SYSTEM_IDENTITY` 資料列的建立時間，以及虛擬機器最後一次作用的時間，皆為 UTC。 |
| `OS-ProductType`、`OS-ProductTypeName` | 1 `Workstation`、2 `Domain Controller`、3 `Server`。 |
| `OS-CurrentTimeZone` | 伺服器相對於 UTC 的時差，以分鐘為單位（`-420` 即 UTC-07:00）。 |
| `OS-LastBootUpTime` | 伺服器最後一次開機的時間（UTC）；無法讀取時則為原樣儲存的 WMI 文字。 |
| 其他欄位 | 資料表的其他所有欄位，使用 Juntei 的 PascalCase 欄位標籤，例如 `SystemDNS-HostName`、`SystemDomainName`、`OS-BuildNumber` 與 `SystemSerialNumber`。 |

## 自動剖析 { #automatic-parsing }

`auto-parse` 會將這些資料表以 `UAL-Clients.csv`、`UAL-RoleAccess.csv` 等名稱寫入 `LateralMovement`
（使用 `-t json` 或 `-t jsonl` 時則為單一的 `UAL.json` 或 `UAL.jsonl` 檔案），並附上
`LateralMovement/UAL-logs.txt`。合併的 `-j` 時間軸包含每一筆資料列，並帶有 `Comp` 以及 `Artifact`
識別碼 `ual`。`-Z` 會轉換 `InsertDate`、`LastAccess`、`FirstSeen`、`LastSeen`、
`CreationTime`、`LastSeenActive` 與 `OS-LastBootUpTime`。

## 延伸資訊 { #further-details }

- [Microsoft — User Access Logging](https://learn.microsoft.com/en-us/windows-server/administration/user-access-logging/get-started-with-user-access-logging)
- [Plaso — User Access Logging parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/user_access_logging.py)
- [Velociraptor — Windows.Forensics.UserAccessLogs](https://docs.velociraptor.app/artifact_references/pages/windows.forensics.useraccesslogs/)
- [CrowdStrike — How to Leverage User Access Logging for Forensic Investigations](https://www.crowdstrike.com/en-us/blog/user-access-logging-ual-overview/)
