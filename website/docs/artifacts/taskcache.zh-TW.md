# 工作排程器快取 { #task-scheduler-cache }

## 什麼是 TaskCache { #what-taskcache-is }

Windows 工作排程器在 `SOFTWARE` 下保存一份登錄快取，將工作路徑連結至 GUID、
觸發程序／清單、主體、動作、執行期中繼資料與安全性描述元。Juntei 會解碼
執行動作與 COM 處理常式動作、舊式的電子郵件／訊息動作、工作時間戳記，以及
工作被從一般工作排程器介面隱藏的跡象。

排程工作是常見的持續存取與遠端執行機制。當攻擊者刪除或竄改工作的 XML 檔案時，
登錄快取也特別有價值。

<span class="evidence-strength">最佳結論：</span>TaskCache 曾設定或保存此
工作／動作。若要證明某次特定的執行，需要執行期欄位與獨立的記錄。

## 位置 { #locations }

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

請收集 `SOFTWARE` 及其交易記錄，以及完整的 `System32\Tasks` 樹狀結構。工作排程器的
Operational 與 Security 事件記錄可提供建立／更新／執行期的額外證據。

## 分析注意事項 { #analyst-cautions }

- 在遭竄改、刪除、註冊不完整或損毀之後，登錄快取與工作 XML 可能不一致。
- Juntei 的 `Hidden` 識別的是結構上的隱藏（在相關條件下缺少 Tree 連結／安全性描述元），
  而不只是設定了一般「隱藏」UI 旗標的工作。
- Windows 7 將工作動作儲存在 XML 中，而非快取的 `Actions` 值；因此在該版本上，
  空的動作並不自動代表可疑。
- `LastRun` 與 `LastResult` 很有用，但並非完整、無法竄改的執行歷程。
- `Context` 指出的是所設定的主體／內容，而非註冊該工作的使用者。
- COM 處理常式的 CLSID 需要另外透過登錄／檔案解析。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.Hidden == true or (.Executable? | test("powershell|cmd|mshta"; "i")))'
```

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`SourcePath` | 主機脈絡與來源 `SOFTWARE` 登錄區。 |
| `TaskName` | 完整的排程器路徑。連結至 XML 與事件記錄的主要樞紐。 |
| `TaskGUID` | 連結 Tree 與 Tasks 快取項目的 GUID。有力的交叉比對鍵值。 |
| `TaskIndex` | 排程器使用的 Tree／索引中繼資料；屬於脈絡資訊，並非執行次數。 |
| `TaskLists` | 將觸發／處理方式分類的快取清單（`Boot`、`Logon`、`Plain`、`Maintenance`）。 |
| `Hidden` | 結構上的隱藏工作指標。在斷定遭竄改之前，請先檢視 `InTree`、`InTasks` 與描述元狀態。 |
| `InTree`、`InTasks` | 快取的兩端是否都存在。缺少連結可能代表遭刪除／竄改／註冊不完整。 |
| `HasSecurityDescriptor` | Tree 項目是否具有 `SD`；在相關版本上，其缺席可能促成工作被隱藏的行為。 |
| `TreeKeyLastWrittenUTC`、`TaskKeyLastWrittenUTC` | 快取各端的機碼層級變更；並非確切的建立／執行時間。 |
| `ActionIndex` | 工作中從零起算的動作編號。一個工作可能產生多列資料。 |
| `ActionType` | `exec`、`com_handler`、`send_email` 或 `show_message`。 |
| `Executable`、`Args`、`WorkingDirectory` | 實際生效的執行動作。請判讀引號、變數、相對路徑與 LOLBin。 |
| `COM-ClassID`、`ActionData` | COM 處理常式的 CLSID 與資料，或舊式電子郵件／訊息的酬載。請另外解析 CLSID。 |
| `Context` | 與動作一同編碼的主體內容（`LocalSystem`、`Author` 等）；是所設定的身分，而非註冊者。 |
| `Author`、`URI`、`Description` | 工作中繼資料。很容易被攻擊者控制，不能作為可信的證據。 |
| `LastRegisteredUTC` | DynamicInfo 中的註冊／更新時間（若存在）。 |
| `LastRunUTC`、`LastSuccessfulRunUTC`、`LastResult` | 快取的執行期／結果中繼資料。請與 Operational 記錄及處理程序證據交叉比對。 |
| `ActionsRaw` | 解碼不完整或需要驗證時的原始動作位元組。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡。 |
| `Problems` | 路徑／GUID 不符、值類型錯誤、動作結構遭截斷或缺少部分內容。 |

## 調查策略 { #investigation-strategy }

1. 優先檢視被隱藏／不一致的工作，以及非 Microsoft 的路徑。
2. 重建實際生效的命令，包括參數、工作目錄、環境變數展開與 COM 解析。
3. 將登錄記錄與工作 XML 比較，並對所參照的執行檔計算雜湊值、檢查時間戳記。
4. 將 `TaskName`／GUID／時間與工作排程器事件、Security 4698/4702、處理程序記錄、Prefetch 及 MFT／USN 交叉比對。
5. 當攻擊者移除了工作時，請搜尋已刪除／已復原的登錄資料。

## 延伸資訊 { #further-details }

- [Microsoft：工作排程器結構描述](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft：工作排程器事件記錄](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 Scheduled Task](https://attack.mitre.org/techniques/T1053/005/)
