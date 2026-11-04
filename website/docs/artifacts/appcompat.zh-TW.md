# 應用程式相容性 { #application-compatibility }

## Windows 保存了哪些資料 { #what-windows-keeps }

除了 ShimCache（`parse-registry shimcache`）與 Amcache（`parse-registry amcache`）之外，
Windows 的應用程式相容性功能還會在
`Microsoft\Windows NT\CurrentVersion\AppCompatFlags` 下保存以下資料：

| 來源 | 登錄區與機碼 | 顯示的內容 |
|---|---|---|
| `layers` | `SOFTWARE`（及 `Wow6432Node`）與 `NTUSER.DAT` 的 `Layers` | 設定為以相容性層執行的程式：`RUNASADMIN`（一律以系統管理員身分執行）、`WIN7RTM`、`HIGHDPIAWARE`…… |
| `pca_store` | `NTUSER.DAT` 的 `Compatibility Assistant\Store`（Windows 8 及更新版本） | 程式相容性助理（Program Compatibility Assistant）觀察到使用者執行過的程式 |
| `pca_persisted` | `NTUSER.DAT` 的 `Compatibility Assistant\Persisted`（Windows 7） | 同上，適用於 Windows 7 |
| `cit_module` | `SOFTWARE` 的 `CIT\Module\<module>` | 載入過受追蹤模組的程式，以及其最後一次載入的時間 |
| `cit_system` | `SOFTWARE` 的 `CIT\System`（Windows 7 至早期 Windows 10） | 壓縮的 CIT 資料庫中的程式路徑與命令列 |
| `installed_sdb` | `SOFTWARE` 的 `InstalledSDB\{GUID}` | 以 `sdbinst` 安裝的自訂 shim 資料庫：檔案、描述、安裝時間 |
| `custom_sdb` | `SOFTWARE` 的 `Custom\<program>` | 套用至各程式的自訂 shim 資料庫，以及其安裝時間 |

<span class="evidence-strength">最佳結論：</span>相容性層與 shim 資料庫是會改變程式執行方式的
設定；程式相容性助理與 CIT 項目則顯示某個程式曾以此使用者身分或在此系統上執行（或載入過模組）。

自訂 shim 資料庫是一種持續存取與注入技術（MITRE ATT&CK T1546.011）：每當程式執行時，
shim 都可以將 DLL 載入該程式，或啟動另一個程式。

## 位置 { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

請收集 `SOFTWARE`、每個 `NTUSER.DAT` 及其交易記錄，以及 shim 資料庫路徑所指向的
`.sdb` 檔案（通常位於 `C:\Windows\AppPatch\Custom\`）。

## 分析注意事項 { #analyst-cautions }

- **CIT 的解壓縮有上限。**宣告解壓縮後超過 64 MiB 的值，會以原始位元組形式保留並附上
  診斷訊息。格式錯誤的壓縮資料與大小不符的情況都會被回報。


- **程式相容性助理項目沒有執行時間。**`Store` 資料（`SACP`，保存在
  `DataRaw` 中）帶有一個時間，但在 Windows 8.1、10 與 11 上，同一登錄區中每個程式的這個時間都相同，
  在其他系統上亦然：它並不是程式執行的時間。只有 `Store` 機碼的最後寫入時間
  能標示最近一次變更的時間。
- **卸除式媒體上的程式**會被命名為 `SIGN.MEDIA=<id> <path>`：`Path` 是該媒體上的路徑，
  `Details` 則包含媒體 ID。
- **CIT 資料庫**採用 LZNT1 壓縮，且其結構並未公開文件化：Juntei
  會列出在每個資料庫中找到的程式路徑與命令列，每個資料庫各列一次，不含次數或
  時間。命令列是獨立的記錄，並未與其程式綁定。Windows 10 及更新版本在此處保存的
  資料很少。
- **Shim 資料庫。**請檢查每一筆 `custom_sdb` 與 `installed_sdb` 記錄：合法軟體
  很少會安裝 shim 資料庫。`.sdb` 檔案本身會說明該 shim 的作用。
- 使用者設定的**相容性層**只套用於該使用者；`SOFTWARE` 中的則套用於所有人。`RUNASADMIN` 會讓
  程式每次都要求提升權限。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry app-compat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-compat -d evidence/ -o appcompat.csv
juntei parse-registry app-compat -d evidence/ -t jsonl \
  | jq 'select(.Source == "installed_sdb" or .Source == "custom_sdb")'
```

摘要會列出自訂 shim 資料庫、設有相容性層的程式，以及各 CIT 模組最後一次被使用的
時間。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`SourcePath`、`HiveType` | 主機、登錄區檔案，以及 `SOFTWARE` 或 `NTUSER.DAT`。 |
| `User`、`UserSID` | 使用者登錄區的設定檔資料夾名稱，以及取自系統 `ProfileList` 的 SID。 |
| `Source`、`Scope` | `layers`、`pca_store`、`pca_persisted`、`cit_module`、`cit_system`、`installed_sdb`、`custom_sdb`；`machine` 或 `user`。 |
| `Path` | 程式（值的名稱）、CIT 資料庫中的路徑或命令列，或 shim 資料庫所套用的程式。 |
| `Layers` | 相容性層，依原始寫法呈現（`~ RUNASADMIN`）。 |
| `LastSeenUTC` | 程式最後一次載入 CIT 模組的時間。 |
| `SdbGUID`、`SdbPath`、`SdbInstalledUTC` | shim 資料庫的 GUID、檔案與安裝時間。 |
| `Details` | CIT 模組名稱、CIT 資料庫的值名稱、程式的卸除式媒體 ID（`SignMedia`）、shim 資料庫的 `DatabaseDescription` 與 `DatabaseType`。 |
| `KeyPath`、`KeyLastWrittenUTC`、`ValueName`、`WOW64` | 記錄的來源位置、該機碼最後變更的時間，以及是否位於 `Wow6432Node` 之下。 |
| `DataRaw` | `Store` 值的 `SACP` 資料，或內容不符預期的資料（十六進位）。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡。 |
| `Problems` | 無法解碼或讀取的資料。 |

## 調查策略 { #investigation-strategy }

1. 檢視每一個自訂 shim 資料庫：其 `.sdb` 檔案、安裝時間以及所套用的程式；
   並與 Prefetch、Amcache 及處理程序建立事件中的 `sdbinst.exe` 交叉比對。
2. 在程式相容性助理項目中，尋找位於使用者可寫入資料夾中的程式，以及
   來自卸除式媒體的程式。
3. 檢查不尋常程式上的 `RUNASADMIN` 及其他相容性層。
4. 與 ShimCache、Amcache、BAM、Prefetch 及 UserAssist 交叉比對以取得執行時間。

## 延伸資訊 { #further-details }

- [Microsoft：應用程式相容性資料庫](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011：Application Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft：MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
