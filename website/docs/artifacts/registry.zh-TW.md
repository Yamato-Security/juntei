# Windows 登錄區 { #windows-registry-hives }

## 什麼是登錄區 { #what-registry-hives-are }

Windows 登錄檔將系統與使用者設定儲存在二進位的登錄區（hive）檔案中。機碼構成樹狀結構；
機碼之下的值包含具有型別的資料。登錄檔跡證可描述持續存取、已安裝的軟體、
服務、帳戶、網路設定、使用者活動、安全性設定，以及 Windows 狀態的幾乎每個層面。

Juntei 的通用讀取器可解碼任何主要登錄區、重播交易記錄、回報損毀，並能
復原已刪除／可用 cell、cell slack、殘留資料、重播前以及僅存在於交易記錄中的記錄。

<span class="evidence-strength">最佳結論：</span>登錄區包含或以可復原的形式保留了
該機碼／值的狀態。機碼的最後寫入時間顯示機碼發生過變更，但不會顯示是哪個值變更或變更的原因。

## 常見的登錄區與位置 { #common-hives-and-locations }

| 登錄區 | 一般路徑 | 證據範例 |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | 服務、驅動程式、控制集、主機名稱、已掛載的裝置、ShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | 已安裝的軟體、TaskCache、設定檔、Windows 設定 |
| `SAM` | `C:\Windows\System32\config\SAM` | 本機帳戶與帳戶中繼資料 |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | 本機安全性原則／機密資訊中繼資料（取決於存取權限） |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | 預設系統設定檔的設定 |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | 使用者偏好設定、Run 機碼、RecentDocs、應用程式活動 |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | 殼層類別、COM／使用者殼層活動 |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | 應用程式清冊（如需語意化的輸出，請使用 `parse-registry amcache`） |

請收集相鄰的 `.LOG`、`.LOG1` 與 `.LOG2` 檔案。在 Windows 執行期間複製的登錄區可能處於
dirty 狀態；交易記錄中包含主要檔案本身尚未反映的已認可狀態。

## 分析注意事項 { #analyst-cautions }

- 機碼最後寫入時間戳記適用於**機碼**，而非個別的值。
- 登錄值通常沒有獨立的時間戳記。
- `CurrentControlSet` 是透過 `SYSTEM\Select` 解析的分析人員／Windows 別名；它並不是
  實際儲存的機碼。
- 已刪除的記錄可能不完整、過時、在多個復原來源間重複，或已被覆寫。
- 僅存在於交易記錄中的記錄呈現的是歷史／中間狀態。請說明 `recovered_from`。
- `absent_value` 結果表示所查詢的機碼存在，但在剖析出的狀態中沒有該值；
  它並不能證明該值從未存在。
- 字串、路徑與命令列屬於設定，並不能證明所設定的動作曾經執行。
- 沒有可套用記錄檔的 dirty 登錄區，可能會遺漏狀態或呈現過時的狀態。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host` | 從可用的系統脈絡推導出的主機名稱；若 Juntei 無法安全地識別則為空白。 |
| `HiveType` | 偵測到的登錄區類別（`SYSTEM`、`SOFTWARE`、`NTUSER.DAT` 等）。用以判讀路徑。 |
| `SourcePath` | 確切的輸入登錄區。在目錄模式合併多台主機／多位使用者時不可或缺。 |
| `RecordType` | `key`、`value`、`default_value` 或 `absent_value`。「不存在」僅針對該查詢／狀態而言。 |
| `KeyPath` | 登錄區根目錄之下的路徑。`[unknown]` 表示無法解析的已刪除上層路徑。 |
| `KeyLastWrittenUTC` | 機碼整體的最後修改時間。無法藉此識別是哪個值發生變更。 |
| `ValueName` | 具名的值；預設／未命名的值則為空白。 |
| `ValueType` | 登錄檔型別（`REG_SZ`、`REG_DWORD`、`REG_BINARY` 等）。型別會影響判讀。 |
| `ValueData` | 依型別呈現的值。多字串在 JSON 中維持為清單。 |
| `DataSize` | 值所記錄的大小。請與原始位元組及 `Problems` 比較。 |
| `ValueDataRaw` | 解碼失敗或使用 `--raw-data` 時的確切位元組。請保留以供驗證。 |
| `ValueU64` | DWORD/QWORD 的完整數值表示，不會有 JavaScript 精度損失。 |
| `ValueFiletimeUTC` | 選用的 QWORD 作為 FILETIME 的呈現。僅在該機碼的結構描述定義為 FILETIME 時才有意義。 |
| `AllocationStatus` | `allocated`、`deleted_complete`、`deleted_partial`、`residual_value_only` 或 `transaction_log_recovered`。決定可信度。 |
| `RecoveredFrom` | `free_cell`、`cell_slack`、`remnant_data`、`before_replay`，或如 `SYSTEM.LOG1#1234` 的記錄檔項目。 |
| `TransactionLogStatus` | 此記錄／登錄區脈絡的 `clean`、`replayed`、`unchanged` 或 `not_applied`。 |
| `LogsApplied` | Juntei 所重播之交易記錄的檔案名稱。請將其記載於報告中。 |
| `Problems` | 損毀、缺漏的 cell、型別不符、無法解析的清單／父項目或其他限制。 |

## 復原狀態的判讀 { #recovery-status-interpretation }

| 狀態 | 意義 |
|---|---|
| `deleted_complete` | 已刪除的機碼／值已以完整的已知路徑／資料復原。但仍屬歷史狀態，而非目前作用中的狀態。 |
| `deleted_partial` | 部分路徑、值清單、值或資料已被覆寫／無法讀取。請勿用重複使用之 cell 的內容來填補缺口。 |
| `residual_value_only` | 值雖然留存，但沒有足夠的機碼脈絡；`KeyPath` 可能不存在。 |
| `transaction_log_recovered` | 該狀態僅存在於重播前／記錄檔版本中，而不在目前已配置／可用的記錄中。 |

## 調查策略 { #investigation-strategy }

1. 若已知明確的語意機碼，請從該機碼著手；探索時則使用 regex／資料搜尋。
2. 在檢視內容之前，先檢查 `TransactionLogStatus`、`AllocationStatus`、`RecoveredFrom` 與 `Problems`。
3. 依照機碼文件所記載的結構描述判讀資料；`REG_BINARY` 本身並不具自我描述性。
4. 將設定與執行時期的證據（事件記錄、Prefetch、SRUM、服務／工作）交叉比對。
5. 對於新穎或有爭議的欄位，請保留原始資料。

## 延伸資訊 { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Maxim Suhanov 的登錄檔格式規格](https://github.com/msuhanov/regf)
- [yarp 登錄檔剖析器](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [Microsoft：登錄值類型](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
