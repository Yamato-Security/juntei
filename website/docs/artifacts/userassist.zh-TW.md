# UserAssist { #userassist }

## 什麼是 UserAssist { #what-userassist-is }

Windows Explorer 會計算使用者從 Explorer 殼層啟動的程式與捷徑：「開始」功能表、工作列、
桌面，以及在資料夾中按兩下。它將這些計數保存在使用者的 `NTUSER.DAT` 中。每個項目具有：

- 執行次數；
- 在 Windows 7 及之後的版本中，取得焦點的次數與時間長度（以毫秒計）；
- 最後一次執行的時間。

值名稱以 ROT13 編碼。程式名稱通常以已知資料夾的 GUID 開頭
（`{1AC14E77-…}` 即 `System32`）。

UserAssist 顯示特定使用者以互動方式啟動了什麼，而大多數其他執行證據並不會將執行歸屬到某位使用者。

<span class="evidence-strength">最佳結論：</span>使用者曾透過 Explorer 啟動此程式或捷徑，
共計此次數，最後一次在此時間。以其他方式（命令列、服務、排程工作、其他程式）啟動的程式不會被計入。

## 位置 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | 保存的內容 |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | 程式（`executable`），Windows 7 及之後版本 |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | 捷徑（`shortcut`），Windows 7 及之後版本 |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | 程式與捷徑（`active_desktop`），Windows 2000 至 Vista |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | 網際網路工具列（`internet_toolbar`），Windows 2000 至 Vista |

其他 GUID 會回報為 `other`。請收集每個設定檔的 `NTUSER.DAT` 及其交易記錄檔，以及提供使用者
SID 的系統 `SOFTWARE` 登錄區。

## 分析注意事項 { #analyst-cautions }

- **新設定檔會被預先植入項目。** 在 Windows 7 及之後的版本中，新設定檔一開始就有預設項目
  （Getting Started、小算盤、小畫家、便利貼……）。它們的執行次數從 14 開始遞減，且全部共用同一個
  最後執行時間，大約在設定檔建立的時候。它們並非使用者活動。
- **版本 3 從 5 開始計數**（Windows 2000 至 Vista）。所儲存的計數從 5 開始，因此會從中減去 5。
  所儲存的計數小於 5 時，會在 `Problems` 中回報。
- **路徑使用已知資料夾的預設路徑**（`%windir%\system32`、`%ProgramFiles%`、
  `%APPDATA%\…`）。被重新導向至其他位置的資料夾仍會顯示其預設路徑。應用程式 ID
  （`Microsoft.Windows.Explorer`、`Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`）與控制台
  項目（`::{…}`）沒有路徑。
- **追蹤功能可以被關閉**，方法是 `Explorer\Advanced` `Start_TrackProgs` = 0、
  `Policies\Explorer` `NoInstrumentation` = 1，或 `UserAssist\Settings` `NoLog` = 1。摘要會
  列出追蹤功能已關閉的登錄區。在活躍的設定檔上，空白或缺少的 UserAssist 可能代表反鑑識行為。
- **`NoEncrypt` = 1** 會讓 Windows 不經 ROT13 寫入名稱。這類名稱會被錯誤解碼，因此該登錄區的
  每個項目都會在 `Problems` 中註明（當記錄輸出至 stdout 時，也會顯示在 stderr 上）。
- **最後執行時間與計數是依項目保存的，**但機碼的最後寫入時間（`KeyLastWrittenUTC`）只是任一項目
  最近一次更新的時間。
- 對於只取得過焦點的項目，**執行次數可能為 0**。焦點次數與時間只存在於版本 5（Windows 7 及之後版本）。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry user-assist <INPUT> [OPTIONS]

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
juntei parse-registry user-assist -d evidence/ -o userassist.csv
juntei parse-registry user-assist -f NTUSER.DAT -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

摘要會列出最後執行的項目（由新到舊）及其使用者，以及 UserAssist 已關閉的登錄區。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`User`、`UserSID`、`SourcePath` | 主機、設定檔資料夾的名稱、取自系統 `ProfileList` 的使用者 SID（需有系統的 `SOFTWARE` 登錄區），以及登錄區。 |
| `GUID`、`Kind` | `{GUID}` 機碼及其保存的內容（`executable`、`shortcut`、`active_desktop`、`internet_toolbar`、`other`）。 |
| `NameRot13`、`Name` | 所儲存的值名稱，以及解碼後的名稱。 |
| `Path` | 以路徑表示的解碼名稱，已知資料夾的 GUID 已替換為其預設路徑；應用程式 ID 與 Shell 項目則為空。 |
| `RunCount` | 從 Explorer 啟動的次數（版本 3：所儲存的計數減 5）。 |
| `FocusCount`、`FocusTimeMs` | 取得焦點的次數，以及以毫秒計的時間長度（版本 5）。 |
| `LastRunUTC` | 最後一次啟動的時間。Windows 儲存為 0 時，或所儲存的時間超過 9999 年時為空（此時 `Problems` 會提供該值，`DataRaw` 會保留其位元組）。 |
| `Format`、`SessionID` | 資料的配置（5：72 位元組，3：16 位元組），以及最後一次寫入時的工作階段編號。 |
| `KeyLastWrittenUTC` | `Count` 機碼的最後寫入時間：其任一項目最近一次更新的時間。 |
| `DataRaw` | 無法解碼時，該值的位元組（十六進位）。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡。 |
| `Problems` | 大小未知的資料、與機碼 `Version` 不符的大小、小於 5 的版本 3 計數、超過 9999 年的最後執行時間、`NoEncrypt`。 |

## 調查策略 { #investigation-strategy }

1. 依 `LastRunUTC` 排序，並將預先植入的項目排除：它們共用同一個時間，大約在設定檔建立的時候。
2. 尋找位於使用者可寫入位置（`%USERPROFILE%`、`Downloads`、`Temp`、
   `ProgramData`）、卸除式磁碟機與網路路徑上的程式，以及系統管理與遠端存取工具。
3. 與 Prefetch（含執行時間的執行紀錄）、Amcache 與 ShimCache（存在跡證）、
   Jump Lists 與 LNK 檔案（以它們開啟了什麼），以及程式檔案的 `$MFT` 進行關聯分析。
4. 檢查追蹤功能已關閉的登錄區，以及 UserAssist 為空的設定檔，確認是否遭到清除。
5. 使用 `parse-registry read --recover-deleted` 找出已刪除的 UserAssist 值。

## 延伸資訊 { #further-details }

- [Microsoft：KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb：User Assist 機碼](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
