# Windows 服務 { #windows-services }

## 服務設定記錄了什麼 { #what-service-configuration-records }

Windows 服務與核心驅動程式設定於每個 `SYSTEM` 控制集之下。記錄中列出
執行檔／驅動程式或服務 DLL、啟動模式、服務類型、帳戶、載入順序群組、延遲
啟動、錯誤處理行為，以及失敗後 Windows 所採取的動作。

攻擊者會建立或修改服務，以達成持續存取、權限提升、載入驅動程式與
橫向移動。合法軟體使用的也是完全相同的機制，因此路徑、帳戶、簽章、
建立脈絡與執行期證據都很重要。

<span class="evidence-strength">最佳結論：</span>所解析的控制集曾將此
服務／驅動程式設定為如上所示。單憑設定並不能證明該服務曾成功啟動。

## 位置 { #location }

```text
SYSTEM\ControlSet00N\Services\<ServiceName>
SYSTEM\Select\Current   (which control set Windows used)
```

請收集 `SYSTEM` 及其交易記錄檔。System 7045 與 Security 4697 等事件 ID 可能
記錄安裝行為；服務控制事件與處理程序事件則可佐證實際的啟動／執行。

## 分析注意事項 { #analyst-cautions }

- `ImagePath` 可能包含環境變數、引號、參數、裝置路徑，或含有空格但未加引號的
  路徑。在正規化之前，請先完整保留其原始內容。
- 對於由 `svchost.exe` 承載的服務，`ServiceDll` 至關重要；單看 `ImagePath` 可能只會看到承載程序。
- `StartType=automatic` 並不能證明該服務曾經啟動或一直保持啟用。
- `ObjectName` 是所設定的帳戶，不一定是安裝該服務的使用者。
- 驅動程式與使用者模式服務使用不同的 `ServiceType` 旗標與路徑慣例。
- 失敗動作可以在服務失敗後，以服務帳戶身分執行命令；這是一個容易被忽略的
  持續存取攻擊面。
- 其他控制集可能保留了先前的設定；請使用 `IsCurrentControlSet`。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --service <NAME>    Only this service name
      --all-control-sets  Parse every ControlSet00N

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --all-control-sets -t jsonl \
  | jq 'select(.FailureCommand != null or (.ImagePath? | test("temp|public"; "i")))'
```

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`SourcePath` | 系統識別與輸入的登錄區。跨多次收集時請保留。 |
| `ControlSet`、`IsCurrentControlSet` | 來源控制集，以及 `Select\Current` 是否指向它。主張目前作用中狀態時，請優先採用目前的控制集。 |
| `ServiceName` | 登錄機碼／內部服務名稱。是連結到事件與服務控制管理員命令的有力樞紐。 |
| `DisplayName`、`Description` | 面向使用者的文字；可能是資源參照，也可能由攻擊者控制。 |
| `ImagePath` | 所設定的執行檔／驅動程式命令。請解析引號、參數、環境變數與裝置路徑。 |
| `ServiceDLL` | 由承載服務載入的 DLL。對於共用處理程序的服務，這通常才是實際的酬載。 |
| `StartType`、`StartValue` | 易讀／原始的啟動模式：boot (0)、system (1)、automatic (2)、manual (3)、disabled (4)。 |
| `DelayedAutostart` | 自動服務在初始開機階段之後才啟動；仍然可用於持續存取。 |
| `ServiceType`、`TypeValue` | 解碼後／原始旗標：核心驅動程式、檔案系統驅動程式、獨立／共用處理程序、使用者服務等。 |
| `ErrorControl` | 開機／啟動錯誤處理層級。主要屬於運作層面的脈絡。 |
| `ObjectName` | 所設定的服務帳戶或驅動程式物件名稱。並非安裝者身分。 |
| `Group` | 載入順序群組，對 boot／system 驅動程式特別重要。 |
| `FailureResetPeriod` | 失敗次數重設前的秒數；`4294967295` 表示永不重設。 |
| `FailureCommand` | 為 `run_command` 失敗動作所設定的命令。高價值的持續存取欄位。 |
| `FailureActions` | 完整解碼後的動作清單（`restart`、`reboot`、`run_command`、`none`）與延遲時間。 |
| `FailureActionType`、`FailureActionDelayMs` | 為方便篩選而攤平的第一個動作；也請檢視完整清單。 |
| `FailureActionsRaw` | 解碼不完整／損毀時的原始位元組。 |
| `KeyLastWrittenUTC` | 服務機碼的最後變更時間，不一定是安裝／啟動時間。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄記錄重播脈絡。 |
| `Problems` | 類型不符、失敗動作結構格式錯誤，或欄位遺失／無法讀取。 |

## 調查策略 { #investigation-strategy }

1. 檢視非標準路徑、可寫入的目錄、不尋常的帳戶、新的機碼時間與失敗命令。
2. 使用受影響主機的設定（而非分析人員的機器）展開環境變數。
3. 若已收集實際的執行檔或 DLL，請計算其雜湊值並檢查簽章。
4. 將服務名稱／路徑與事件 7045/4697、處理程序建立、Prefetch、`$MFT` 及 USN 交叉比對。
5. 比較目前與非目前的控制集，以找出已變更／已刪除的持續存取機制。

## 延伸資訊 { #further-details }

- [Microsoft：裝置與驅動程式服務的登錄樹狀結構](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft：SERVICE_STATUS／服務類型與狀態](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_status)
- [Microsoft：ChangeServiceConfig2 失敗動作](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
