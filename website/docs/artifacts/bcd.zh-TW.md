# 開機設定資料 (BCD) { #boot-configuration-data-bcd }

## 什麼是 BCD 存放區 { #what-the-bcd-store-is }

Windows Vista 及更新版本將開機設定保存在一個登錄區中，即 BCD 存放區：
BIOS 系統為 `\Boot\BCD`，UEFI 系統則為 EFI 系統磁碟分割上的 `\EFI\Microsoft\Boot\BCD`。
`bcdedit` 與 Windows 安裝程式會編輯它；開機管理程式則在每次開機時讀取它。

存放區中保存著多個物件，每個物件皆以 GUID 命名：

- **Windows 開機管理程式**（`{bootmgr}`），包含預設項目、顯示順序與
  選單逾時；在 UEFI 系統上還有**韌體開機管理程式**（`{fwbootmgr}`），包含
  韌體的開機順序；
- 每個 Windows 安裝（包括修復環境）各有一個 **Windows 開機載入器**，另有一個
  **繼續執行應用程式**（休眠）與**記憶體測試程式**；
- **繼承設定**物件（`{globalsettings}`、`{bootloadersettings}`、…），其設定會被
  其他物件繼承；
- **裝置**物件（修復環境的 ramdisk 選項）。

每個物件都有元素（element）：描述、裝置與路徑，以及
`recoveryenabled`、`bootstatuspolicy`、`testsigning`、`nointegritychecks` 與 `safeboot` 等設定。

有兩類攻擊會變更這些設定：

- **勒索軟體**會關閉修復功能，使系統無法修復：
  `bcdedit /set {default} recoveryenabled No` 與
  `bcdedit /set {default} bootstatuspolicy ignoreallfailures`。部分家族會設定 `safeboot`，
  讓 Windows 以安全模式重新啟動，而資安產品在安全模式下不會執行。
- **Rootkit 與未簽署的驅動程式**可能使用 `testsigning on` 或 `nointegritychecks on` 來削弱開機時的檢查。單憑這些設定並不能證明有 rootkit 曾經執行。

<span class="evidence-strength">最佳結論：</span>在收集當時，此開機項目具有此設定。
元素機碼的最後寫入時間，即為該設定最後一次被寫入的時間。

## 位置 { #location }

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

元素的類型為 8 位十六進位數字：其類別（`1` 所有應用程式、`2` 某一種
應用程式、`3` 裝置）、其格式（`1` 裝置、`2` 字串、`3` 物件、`4` 物件清單、`5`
整數、`6` 布林值、`7` 整數清單）以及其編號。此命令會依照 Microsoft 文件中的名稱
為元素命名（`BcdLibraryBoolean_AutoRecoveryEnabled`）。

收集工具通常不會複製此存放區：EFI 系統磁碟分割未被掛載，而 `\Boot`
是隱藏的。請明確地收集它，或掛載該磁碟分割。`-d` 也會讀取 `BCD-Template`
（由 `HiveType` 標示是哪一種）；範本是 Windows 建立存放區的起點，而非實際用來開機的存放區。

## 分析注意事項 { #analyst-cautions }

- **設定會被繼承。** 開機載入器會從其
  `BcdLibraryObjectList_InheritedObjects` 中的物件繼承設定（通常為 `{bootloadersettings}`，而它又繼承
  `{globalsettings}`）。`RecoveryEnabled`、`TestSigning`、`NoIntegrityChecks`、`SafeBoot` 與
  `BootStatusPolicy` 是解析後的儲存值（自身的值，否則為繼承的值），而 `Inherited` 會指出哪些
  來自其他物件。繼承最多追蹤八層，且循環會受到限制。在不完整的存放區中，被參照的物件可能不存在。空白欄位並不能確立 Windows 實際生效的預設值。韌體與 Secure Boot 也可能使儲存的選項無法生效。
- **未設定不等於關閉。** 空白的 `RecoveryEnabled` 表示該元素不存在；此時適用 Windows 的
  預設值。
- **時間以元素為單位。** 每個元素的機碼時間（`Elements` 中的 `key_last_written`）是其
  最後一次被寫入的時間。Windows 在更新與升級時會重寫部分元素；請與其他
  元素的時間比較。
- **裝置僅部分解碼。** GPT 磁碟分割以其磁碟分割與磁碟的 GUID 表示；
  ramdisk 則以其選項物件表示。其他裝置僅保留其位元組（`data_raw`）。
- **未記載的元素**（`0x15000065`、`0x4…` 類別）沒有名稱，並保留其位元組。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.TestSigning == true or .NoIntegrityChecks == true or .RecoveryEnabled == false)'
```

摘要會列出每個存放區的預設項目，以及所有測試簽章、完整性檢查、修復、
開機狀態原則（任何 `Ignore…`）與安全模式設定，並附上其設定時間或繼承來源物件。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`, `SourcePath`, `HiveType` | 主機、登錄區，以及 `BCD` 或 `BCD-Template`。 |
| `ObjectGUID`, `ObjectType`, `ObjectTypeName` | 物件的 GUID、其類型（`0x10200003`）以及其為何物（`Windows boot loader`、`inherited settings for Windows boot loader`、…）。 |
| `Description`, `Path`, `Device` | 描述、應用程式的路徑及其裝置（GPT 磁碟分割與磁碟，或 ramdisk 的選項物件）。 |
| `IsDefault`, `DisplayOrder` | Windows 開機管理程式是否預設以它開機，以及它在開機選單中的位置。 |
| `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks` | 從存放區解析出的 `recoveryenabled`、`testsigning`、`nointegritychecks`。 |
| `SafeBoot`, `BootStatusPolicy` | `safeboot`（`Minimal`、`Network`、`DsRepair`）與 `bootstatuspolicy`（`IgnoreAllFailures`、…），並附上數值。 |
| `Inherited` | 上述哪些設定來自其他物件，以及來自哪個物件。 |
| `Elements` | 所有元素：其類型、名稱、值、位元組（針對裝置、未記載的類型與無法解碼的資料）以及機碼時間。 |
| `KeyLastWrittenUTC` | 物件機碼的最後寫入時間。 |
| `TransactionLogStatus`, `LogsApplied` | 登錄檔復原的脈絡。 |
| `Problems` | 沒有類型的物件、資料不符合其格式的元素。 |

## 調查策略 { #investigation-strategy }

1. 檢查預設項目與每個開機載入器是否有 `TestSigning`、`NoIntegrityChecks`、
   `RecoveryEnabled = false`、`Ignore…` 類的 `BootStatusPolicy` 以及 `SafeBoot`。
2. 使用元素時間推定變更的日期，並與 Prefetch、
   Amcache、程序建立事件（4688、Sysmon 1）以及 System 事件記錄中的 `bcdedit.exe` 交叉比對。
3. 尋找不應存在的開機項目或裝置（其他的載入器路徑、ramdisk），以及
   遭變更的預設項目或顯示順序。
4. 將存放區與 `BCD-Template` 以及相同組建的其他系統進行比較。

## 延伸資訊 { #further-details }

- [Microsoft：BCD 開機選項參考](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft：BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft：BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft：BcdObject（物件類型）](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
