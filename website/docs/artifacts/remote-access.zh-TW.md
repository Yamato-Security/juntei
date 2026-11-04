# 遠端存取（使用者端） { #remote-access-user-side }

## 這是什麼 { #what-it-is }

使用者的 `NTUSER.DAT` 會保留該使用者從這台電腦建立之遠端連線的痕跡。這些是橫向移動的來源端：

- **遠端桌面（`mstsc`）：**
  - 輸入到用戶端中的電腦，由新到舊；
  - 使用者曾登入的每台電腦各有一個機碼，其中記錄所使用的帳戶名稱，以及使用者接受之伺服器
    憑證的指紋；
  - 使用者允許使用本機裝置的電腦。
- **PuTTY：**
  - 已儲存的工作階段：主機、連接埠、帳戶、通訊協定、金鑰檔、Proxy 與連接埠轉送；
  - 最後啟動的工作階段；
  - 使用者接受的每個 SSH 伺服器金鑰。
- **WinSCP：**已儲存的工作階段與已接受的 SSH 伺服器金鑰，形式與 PuTTY 相同。
- **對應的磁碟機：**每個設定為登入時還原的對應磁碟機，以及輸入到 *連線網路磁碟機* 中的路徑。
- **共用資料夾：**Explorer 掛載過的共用資料夾（`MountPoints2`）。

即使遠端電腦本身的記錄已經消失，它們仍能顯示某個帳戶接下來前往何處，以及使用了哪些認證資訊。

<span class="evidence-strength">最佳結論：</span>此使用者的設定檔曾連線至此目標，或曾儲存
連至此目標的連線。單憑已儲存的工作階段或輸入的名稱，並不能證明連線成功。已接受的主機金鑰，
或具有 `UsernameHint` 的 `Servers` 機碼則是較有力的證據：用戶端曾連上該伺服器。

## 位置 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Terminal Server Client\Default            MRU0 … MRU9
Software\Microsoft\Terminal Server Client\Servers\<host>     UsernameHint, CertHash
Software\Microsoft\Terminal Server Client\LocalDevices       <host> = devices
Software\SimonTatham\PuTTY\Sessions\<session>
Software\SimonTatham\PuTTY\Jumplist                          Recent sessions
Software\SimonTatham\PuTTY\SshHostKeys                       <type>@<port>:<host> = key
Software\Martin Prikryl\WinSCP 2\Sessions\<session>
Software\Martin Prikryl\WinSCP 2\SshHostKeys
Network\<letter>                                             RemotePath, UserName, ProviderName
Software\Microsoft\Windows\CurrentVersion\Explorer\Map Network Drive MRU
Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2\##server#share
```

請收集每個設定檔的 `NTUSER.DAT` 及其交易記錄檔。也請收集系統的 `SOFTWARE`：它提供每位使用者的 SID。

## 分析注意事項 { #analyst-cautions }

- **機碼時間是清單的時間。** `Default`、`SshHostKeys` 與 `Map Network Drive MRU` 保存的是清單，
  其機碼的最後寫入時間是清單最後一次變更的時間。該時間屬於第一個項目（`MruPosition` 0）。
  對其他項目而言，它只是一個上限。`Servers\<host>`、工作階段、`Network\<letter>` 或共用資料夾
  機碼則有其自己的時間。
- **已儲存不代表已使用。** PuTTY 或 WinSCP 工作階段可以在從未連線的情況下被儲存。PuTTY 會在
  啟動已儲存的工作階段時重新寫入 `Recent sessions`，並在第一次連上伺服器時新增主機金鑰。
- **密碼：**
  - WinSCP 可以儲存工作階段、通道與 Proxy 的密碼。
  - PuTTY 只儲存 Proxy 的密碼，且為純文字。
  - `PasswordStored` 與 `Details` 的 `StoredPasswords` 會指出設定了哪些值。這些值絕不會被解碼或寫出。
- **WinSCP 會省略預設值。** 沒有 `FSProtocol` 的工作階段為 SFTP，而沒有 `PortNumber` 的工作階段
  使用該通訊協定的預設連接埠。此時 `Protocol` 與 `Port` 會保持空白，而不會進行猜測。
- **主機金鑰指紋**採用 OpenSSH 的 `SHA256:…` 格式，與 `ssh-keygen -l`、`ssh` 以及 PuTTY 0.75
  及之後版本顯示的形式相同。Juntei 會從 PuTTY 與 WinSCP 保存的金鑰計算它們。請將它們與伺服器
  本身的金鑰、`known_hosts` 檔案及記錄進行比較，以識別名稱或 IP 位址背後的伺服器。SSH-1 金鑰
  （`rsa`）沒有指紋。
- **對應的磁碟機：**只有在以其他帳戶對應磁碟機時，才會設定 `RemoteUsername`；若使用的是
  使用者自己的帳戶，Windows 會寫入 `REG_DWORD` 0。
- **共用資料夾：**`MountPoints2` 也有磁碟區與磁碟機代號的機碼。此處只回報網路共用資料夾
  （`##server#share`）；`\\server@SSL@443\DavWWWRoot` 是 WebDAV 共用資料夾。
- **移除：**清除清單或使用可攜式用戶端，都不會在此處留下痕跡。請與 Jump Lists、Prefetch、
  `.rdp` 檔案、RDP 點陣圖快取及目標的記錄進行比較。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry remote-access <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry remote-access -d evidence/ -o remote.csv
juntei parse-registry remote-access -f NTUSER.DAT -t jsonl \
  | jq 'select(.Client == "rdp" or .HostKeyFingerprint != null)'
```

摘要會依用戶端計算連線數量。它會列出機碼最後寫入的連線（由新到舊），以及保存密碼的工作階段。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`User`、`UserSID`、`SourcePath` | 主機、設定檔資料夾的名稱、取自系統 `ProfileList` 的使用者 SID（需有系統的 `SOFTWARE` 登錄區），以及登錄區。 |
| `Client` | `rdp`、`putty`、`winscp`、`mapped_drive` 或 `network_share`。 |
| `Kind` | 記錄的來源：`rdp_mru`、`rdp_server`、`rdp_local_devices`、`putty_session`、`putty_host_key`、`winscp_session`、`winscp_host_key`、`network_drive`、`map_network_drive_mru`、`mount_point`。 |
| `Target`、`Port` | 電腦（主機名稱或 IP 位址）與連接埠，或共用資料夾（`\\server\share`）。只有在 Windows 或用戶端寫入連接埠時才會提供。 |
| `RemoteUsername` | 在目標上使用的帳戶：`UsernameHint`、工作階段的 `UserName`（或 PuTTY `HostName` 中的 `user@`）、對應磁碟機的 `UserName`。 |
| `Protocol` | PuTTY 的 `Protocol`（`ssh`、`telnet`、`raw`……）、WinSCP 的 `FSProtocol`（`sftp`、`scp`、`ftp`、`webdav`、`s3`）、對應磁碟機的提供者（`Microsoft Windows Network`）。 |
| `MRU-Position` | 在其清單中的位置（0 為最新）：`MRU<n>`、`MRUList`、PuTTY 的 `Recent sessions`。 |
| `SessionName` | 已儲存工作階段的名稱，`%20` 與其他跳脫字元已解碼。 |
| `HostKeyType`、`HostKeyFingerprint`、`HostKey` | 已接受之 SSH 主機金鑰的類型（`rsa2`、`ssh-ed25519`、`ecdsa-sha2-nistp256`……）、其 OpenSSH 指紋，以及所儲存的數值。 |
| `DriveLetter` | 對應磁碟機的代號。 |
| `PasswordStored` | 工作階段是否保存密碼；絕不會解碼。 |
| `Details` | 其他設定，以 `Name=Value; …`（CSV）或物件（JSON）表示：RDP 憑證的 `CertHash`、`LocalDevices`、工作階段的金鑰檔、Proxy、SSH 通道、連接埠轉送、遠端命令與目錄、`StoredPasswords`、共用資料夾的 `Label`、磁碟機的 `ConnectionType`。 |
| `KeyPath`、`ValueName` | 機碼，以及對於由單一值構成的記錄而言的值。 |
| `KeyLastWrittenUTC` | 機碼的最後寫入時間（清單請參閱注意事項）。 |
| `DataRaw` | 當值不是字串或無法解碼時，該值的位元組（十六進位）。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡。 |
| `Problems` | 類型錯誤的值（連同其位元組）、無法讀取的主機金鑰、不存在的 `MRUList` 字母，以及未列出的值。 |

## 調查策略 { #investigation-strategy }

1. 依用戶端列出每位使用者的目標，並與該使用者應該連線的系統進行核對。請尋找系統管理共用資料夾
   （`C$`、`ADMIN$`）與 IP 位址。
2. 記下所使用的帳戶（`RemoteUsername`）：網域系統管理員以及其他電腦上的本機帳戶。
3. 將 `HostKeyFingerprint` 與伺服器的金鑰比對，並將 `CertHash` 與 RDP 伺服器的憑證比對。
4. 與目標的登入事件（4624 類型 10 與 3、1149）、Jump Lists 與 LNK 檔案（`mstsc.exe`、
   `putty.exe`、從共用資料夾開啟的檔案）、Prefetch 以及 RDP 點陣圖快取進行關聯分析。
5. 使用 `parse-registry read --recover-deleted` 找出已刪除的項目，並使用該設定檔 `NTUSER.DAT`
   的較舊副本（磁碟區陰影複製、備份）找出已清除的清單。

## 延伸資訊 { #further-details }

- [Microsoft：從遠端桌面連線的「電腦」方塊中移除項目](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [PuTTY FAQ：PuTTY 將其資料儲存在哪裡？](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP：設定的儲存位置](https://winscp.net/eng/docs/config)
- [libyal winreg-kb：Terminal Server Client](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb：掛載點](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
