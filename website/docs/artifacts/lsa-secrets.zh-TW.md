# LSA 機密資訊 { #lsa-secrets }

## 這些是什麼 { #what-they-are }

本機安全性授權（LSA）將機密資訊保存在 `SECURITY` 登錄區中。這些機密資訊包括
自動登入密碼、服務執行時所用帳戶的密碼、電腦帳戶的網域
密碼、DPAPI 系統金鑰，以及用來加密快取網域登入資訊的金鑰。它
也會保存快取的網域登入資訊，讓網域使用者在沒有網域控制站的情況下也能登入。

`parse-registry lsa-secrets` 會列出系統持有哪些機密資訊及其設定時間，以及
保存與使用了多少個快取登入。**它只讀取名稱與時間：不會解密任何內容，也
不會讀取或寫入任何機密資訊的位元組。**Juntei 讀取的是證據；它不會擷取認證資訊。

<span class="evidence-strength">最佳結論：</span>系統曾持有此名稱的機密資訊，且於
此時間設定。服務的機密資訊（`_SC_<service>`）表示 LSA 儲存了該服務執行時
所用帳戶的密碼。

## 位置 { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| 名稱 | 類型 | 內容 |
|---|---|---|
| `DefaultPassword` | `autologon` | Winlogon 自動登入時所使用的密碼。 |
| `_SC_<service>` | `service_account` | `<service>` 執行時所用帳戶的密碼。 |
| `$MACHINE.ACC` | `machine_account` | 電腦帳戶在其網域中的密碼。 |
| `DPAPI_SYSTEM` | `dpapi_system` | 系統的 DPAPI 金鑰。 |
| `NL$KM` | `cached_logon_key` | 用來加密快取登入資訊的金鑰。 |
| `L$_SQSA_<SID>` | `security_questions` | 本機帳戶的安全性問題（Windows 10 及更新版本）。 |
| `RasDialParams!<SID>#n`、`L$_Ras…` | `ras_credentials` | 撥號與 VPN 認證資訊。 |
| `L$…`、`G$…`、`M$…` | `local`、`global`、`machine` | 其他私人資料，依 `LsaStorePrivateData` 所給予的前綴區分。 |

請收集 `SECURITY` 及其交易記錄，以及提供主機名稱的系統 `SYSTEM`
登錄區。

## 分析注意事項 { #analyst-cautions }

- **只有名稱與時間，沒有值。**值是否包含資料是依其大小判斷。其內容
  不會被讀取。
- **`DefaultPassword` 不代表已啟用自動登入。**大多數系統上都存在此機密資訊，包括
  從未自動登入的系統。`SOFTWARE` 的
  `Microsoft\Windows NT\CurrentVersion\Winlogon`（`AutoAdminLogon`、`DefaultUserName`）才能說明
  是否已設定自動登入。最近的 `CurrentSetUTC` 是值得釐清的變更。
- **服務的機密資訊：**當服務被設定為以有密碼的帳戶執行時，就會出現 `_SC_<service>`。
  請將其與 `parse-registry services`（`ObjectName`）比較，並尋找
  最近安裝或不明的服務。服務被移除後，該機密資訊仍可能留存。
- **時間：**`CurrentSetUTC` 與 `OldSetUTC` 是 Windows 設定目前值與先前
  值的時間。`$MACHINE.ACC` 的密碼預設每 30 天變更一次，因此其時間會變動。
  `KeyLastWrittenUTC` 以及 `CurrVal`／`OldVal` 機碼的時間，大約是機密資訊最後一次
  變更的時間。
- **快取登入：**當槽位的記錄中有使用者名稱時，該槽位即為使用中。記錄的標頭並未
  加密；只會讀取其使用者名稱長度。`CachedLogonsKept` 是 `NL$Control` 的計數，
  它依循 `CachedLogonsCount`（預設為 10）。使用中的槽位表示有網域使用者曾以互動方式
  登入，而且其認證資訊可以從此登錄區離線破解。使用者名稱
  經過加密，不會被解碼。只有標頭完整且使用者名稱長度一致、經過驗證的二進位記錄才會計為使用中。若 `Problems` 回報有損毀的槽位，使用中的計數只涵蓋經過驗證的槽位；請勿將損毀的槽位解讀為空槽位。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

對於每個登錄區，摘要會列出每項機密資訊及其類型與設定時間。接著列出
LSA 儲存了其帳戶密碼的服務，並計算使用中的快取登入槽位數。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`SourcePath` | 主機（取自系統的 `SYSTEM` 登錄區），以及 `SECURITY` 登錄區。 |
| `Secret`、`Kind` | 機密資訊的名稱及其內容（請參閱上表）；快取登入記錄則為 `Cache` 與 `cached_logons`。 |
| `ServiceName`、`UserSID` | `_SC_` 機密資訊所屬的服務，以及名稱中所含的使用者 SID（`L$_SQSA_`、`RasDialParams`）。 |
| `CurrentSetUTC`、`OldSetUTC` | `CupdTime` 與 `OupdTime`：目前值與先前值的設定時間。 |
| `HasCurrentValue`、`HasOldValue` | `CurrVal` 與 `OldVal` 是否包含資料。 |
| `CurrentValueLastWrittenUTC`、`OldValueLastWrittenUTC`、`KeyLastWrittenUTC` | `CurrVal`、`OldVal` 與機密資訊機碼的最後寫入時間。 |
| `CachedLogons`、`CachedLogonsInUse`、`CachedLogonsKept` | 快取登入記錄：槽位數、使用中的槽位數，以及 `NL$Control` 的計數。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡。 |
| `Problems` | 格式錯誤或超出範圍的時間（僅顯示時間中繼資料）、損毀的快取標頭、錯誤的快取值類型，以及無法讀取的機碼。 |

## 調查策略 { #investigation-strategy }

1. 尋找不明或最近安裝之服務的 `_SC_` 機密資訊，並與
   `parse-registry services` 及事件 7045（已安裝服務）比對。
2. 將 `DefaultPassword` 的時間與 `Winlogon` 的自動登入設定互相核對。
3. 在網域成員上，記下使用中的快取登入槽位，以及 `Cache` 最後一次寫入的時間。
4. 將機密資訊的時間與時間軸比較：在入侵期間設定的機密資訊，可能
   標示著新的服務帳戶、自動登入或電腦帳戶的變更。

## 延伸資訊 { #further-details }

- [Microsoft：私人資料物件](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft：快取的網域登入資訊](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [libyal winreg-kb：快取認證資訊](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
