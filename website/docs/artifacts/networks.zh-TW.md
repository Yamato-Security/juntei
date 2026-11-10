# 網路 { #networks }

## 這些是什麼 { #what-they-are }

Windows 會記住系統曾加入的網路，以及其網路的設定方式。

- **`SOFTWARE`：**
  - **網路設定檔**（`NetworkList\Profiles`）：系統每連線過一個網路就有一個設定檔，
    包含其名稱（Wi-Fi 網路的 SSID）、類別（公用、私人、網域）、連線類型
    （有線、無線、VPN、行動寬頻），以及系統第一次與最後一次連線到該網路的時間。
  - **簽章**（`NetworkList\Signatures`）：用來識別每個網路的資訊，例如
    閘道的 MAC 位址與 DNS 尾碼。
  - **網路卡**（`NetworkCards`）：介面卡。
- **`SYSTEM`**（每個控制集）：
  - **TCP/IP 介面：** 位址、DHCP 伺服器與租約、DNS 伺服器、網域。
  - 系統所提供的**共用**（`LanmanServer\Shares`）。
  - **連接埠 Proxy**（`PortProxy`）：由 `netsh interface portproxy` 建立的通道，
    攻擊者會利用它進行跳板，以及透過其他連接埠存取 RDP。
  - **持續性路由。**
  - **時區**，用來將設定檔的當地時間轉換為 UTC。

<span class="evidence-strength">最佳結論：</span>系統曾連線到具有此名稱與閘道的網路，
第一次與最後一次連線分別在這些時間。它曾從此 DHCP 伺服器取得此位址，
並在登錄區儲存時提供這些共用與通道。僅憑網路名稱無法識別其所在位置：
許多網路都叫做「Network」，或使用相同的 SSID。

## 位置 { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\NetworkList\Profiles\{GUID}
      ProfileName, Description, Category, NameType, Managed, DateCreated, DateLastConnected
  Microsoft\Windows NT\CurrentVersion\NetworkList\Signatures\Managed|Unmanaged\<signature>
      ProfileGuid, DefaultGatewayMac, DnsSuffix, FirstNetwork
  Microsoft\Windows NT\CurrentVersion\NetworkCards\<n>          ServiceName, Description
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Services\Tcpip\Parameters\Interfaces\{GUID}                    IP, DHCP, DNS, lease times
  Services\LanmanServer\Shares                                   one value per share
  Services\PortProxy\v4tov4|v4tov6|v6tov4|v6tov6\tcp             listen = connect
  Services\Tcpip\Parameters\PersistentRoutes                     destination,mask,gateway,metric
  Control\TimeZoneInformation
```

請將兩個登錄區連同其交易記錄一併收集。將設定檔時間轉換為 UTC 時需要 SYSTEM 登錄區。

## 分析注意事項 { #analyst-cautions }

- **設定檔時間為當地時間。** `DateCreated` 與 `DateLastConnected` 是以系統的
  當地時間寫入。
  - `FirstConnectedLocal` 與 `LastConnectedLocal` 保留其原始值。
  - `FirstConnectedUTC` 與 `LastConnectedUTC` 則以系統 SYSTEM 登錄區的時區（`TimeZone`）
    及其日光節約時間規則進行轉換。
  - 時區變更之前的時間，或日光節約時間規則不同之年份的時間
    （2007 年之前的美國日期），可能會有相應的誤差。
  - 若沒有可讀取的 SYSTEM 登錄區與有效的目前控制集，則只會提供當地時間。日光節約時間的跳空時段與重複出現的模稜兩可時段，並沒有單一可證實的 UTC 時間戳記：此時 UTC 會保留空白，並由 `Problems` 說明此限制。無效的時差或日期同樣不會被轉換。
- **只有第一次與最後一次。** 每個設定檔只有一次第一次連線與一次最後一次連線，而非每一次
  連線。設定檔機碼的最後寫入時間通常與最後一次連線相符。
- **名稱會重複。** Windows 會將未受管理的有線網路命名為「Network」、「Network 2」等。
  Wi-Fi SSID 並非唯一。請使用 `GatewayMac` 與 `DnsSuffix` 區分網路，並與 DHCP 記錄
  及資產清冊交叉比對。閘道 MAC 不一定是 Wi-Fi 存取點的 BSSID，單憑它也無法確立實體位置。
- **共用、Proxy 與路由**屬於設定：共用或連接埠 Proxy 可能被設定後
  從未使用。每個值本身都沒有時間；機碼的最後寫入時間是其中任何一個值
  變更的時間。
- **介面：** GUID 之下沒有位址的介面機碼，是尚未設定的介面卡。
  使用 DHCP 時，位址為最後一次租約的位址。在改用 DHCP 之前留下的靜態位址
  不會顯示為使用中。
- **控制集：** 預設會讀取 Windows 最後使用的控制集。`--all-control-sets`
  會讀取所有控制集，可能會顯示已從目前控制集中移除的共用或 Proxy。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry networks <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and SYSTEM hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry networks -d evidence/ -o networks.csv
juntei parse-registry networks -d evidence/ -t jsonl \
  | jq 'select(.Source == "port_proxy" or .Source == "share")'
```

摘要會列出最後連線的網路（由新到舊），以及所有共用、連接埠 Proxy 與
持續性路由。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`, `SourcePath` | 主機與登錄區。 |
| `ControlSet`, `IsCurrentControlSet` | 針對 SYSTEM 記錄：控制集，以及它是否為 Windows 最後使用的控制集。 |
| `Source` | `network_profile`、`network_signature`（沒有任何設定檔對應的簽章）、`network_card`、`interface`、`share`、`port_proxy`、`route`。 |
| `ProfileName` | 設定檔的名稱（Wi-Fi 網路的 SSID）；簽章則為其描述。 |
| `Category`, `ConnectionType` | `public`、`private`、`domain`；`wired`、`wireless`、`vpn`、`mobile_broadband`（否則為數值）。 |
| `FirstConnectedUTC`, `LastConnectedUTC` | 以 `TimeZone` 轉換為 UTC 的 `DateCreated` 與 `DateLastConnected`。 |
| `FirstConnectedLocal`, `LastConnectedLocal` | 同上，但為原始值（當地時間，精確到毫秒）。 |
| `TimeZone` | 轉換時所使用的時區。 |
| `GatewayMAC`, `DNS-Suffix` | 設定檔簽章中的閘道 MAC 位址與 DNS 尾碼（多個時以 `|` 分隔）。 |
| `InterfaceGUID` | 介面的 GUID；網路卡則為其 `ServiceName`。 |
| `IP-Address`, `DhcpServer`, `LeaseObtainedUTC`, `LeaseExpiresUTC` | 介面使用中的位址、DHCP 伺服器與租約。 |
| `ShareName`, `SharePath` | 共用及其所提供的資料夾。 |
| `ListenAddress`, `ListenPort`, `ConnectAddress`, `ConnectPort` | 連接埠 Proxy：它在何處接聽以及轉送至何處。 |
| `Route` | 依儲存格式表示的持續性路由：`destination,netmask,gateway,metric`。 |
| `Details` | 其他值，以 `Name=Value; …`（CSV）或物件（JSON）表示：設定檔的描述、`Managed`、首次名稱與簽章機碼；介面的介面卡（來自 `NetworkCards`）、遮罩、閘道、DNS 伺服器與網域；共用的類型、備註與其他欄位；Proxy 的種類；路由的各個部分。 |
| `KeyPath`, `ValueName`, `KeyLastWrittenUTC` | 機碼、共用／Proxy／路由所對應的值，以及機碼的最後寫入時間。 |
| `DataRaw` | 無法解碼的位元組：不是有效 `SYSTEMTIME` 的時間、不是字串清單的共用。 |
| `TransactionLogStatus`, `LogsApplied` | 登錄檔復原的脈絡。 |
| `Problems` | 類型或大小錯誤的值、不可能的日期，以及不符合其格式的名稱。 |

## 調查策略 { #investigation-strategy }

1. 依 `LastConnectedUTC` 排序設定檔，並檢查事件前後的網路：未知的
   Wi-Fi 網路、行動熱點、VPN。
2. 使用 `GatewayMac` 與 `DnsSuffix` 區分同名的網路，並在
   不同主機之間比對同一個網路。
3. 檢查每一個連接埠 Proxy 與共用。指向內部主機 3389 或 445 的連接埠 Proxy，或
   分享暫存資料夾（`C:\ProgramData`、`C:\Users\Public`）的共用，都是調查線索。請與事件
   記錄（`netsh` 程序建立、5142/5144 共用事件）以及機碼時間交叉比對。
4. 使用介面的位址與租約時間確認主機在網路上的位置，並與
   防火牆、DHCP 及 Proxy 記錄比對。
5. 使用 `--all-control-sets` 與 `parse-registry read --recover-deleted` 找出已移除的共用、
   Proxy 與設定檔。

## 延伸資訊 { #further-details }

- [Microsoft：netsh interface portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft：TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001：Proxy: Internal Proxy](https://attack.mitre.org/techniques/T1090/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
