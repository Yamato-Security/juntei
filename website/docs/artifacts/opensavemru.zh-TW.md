# 開啟與另存新檔對話方塊 (OpenSaveMRU) { #open-and-save-as-dialogs-opensavemru }

## 什麼是對話方塊歷程 { #what-the-dialog-history-is }

當程式顯示通用的「開啟」或「另存新檔」對話方塊時，Windows 會在使用者的
`NTUSER.DAT` 中記錄：

- **開啟或儲存了哪個檔案：** `OpenSavePidlMRU` 為每個副檔名（`docx`、
  `ps1`、…）各有一個子機碼，另有涵蓋所有副檔名的 `*`，每個都是一份最近使用清單；
- **哪個程式使用了對話方塊，以及當時所在的資料夾：** `LastVisitedPidlMRU`（以及
  供使用舊式對話方塊之程式使用的 `LastVisitedPidlMRULegacy`）；
- **曾顯示對話方塊的程式：** `CIDSizeMRU`（每個程式的對話方塊大小），以及
  `FirstFolder`（程式的路徑與一個資料夾）；
- 在 Windows 2000 與 XP 上，則為 `OpenSaveMRU`（每個副檔名的路徑）與 `LastVisitedMRU`（以路徑表示的程式及
  其資料夾）。

自 Windows Vista 起，檔案與資料夾以殼層項目清單（shell item list）表示，與
捷徑目標的結構相同：每個項目都是路徑的一部分（本機、磁碟機、資料夾、檔案），而
檔案或資料夾項目可以帶有自己的時間與 NTFS 檔案參照。

<span class="evidence-strength">最佳結論：</span>某個程式在此使用者的工作階段中，使用「開啟」或「另存新檔」對話方塊
開啟或儲存了此檔案，或曾位於此資料夾。只有每份清單中最新的項目
具有日期：即其機碼的最後寫入時間。

## 位置 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

請為每個設定檔收集 `NTUSER.DAT` 及其交易記錄。

## 分析注意事項 { #analyst-cautions }

- **每份清單只有一個時間。** 當某個項目移到清單最前面時，機碼的最後寫入時間就會改變，
  因此它只能為位置 0 的項目標定日期（`IsMostRecent`、`LastUsedUTC`）。
  其他項目依序由新到舊排列在清單中，但沒有日期。
- **同一個檔案出現在兩份清單中。** 檔案會同時出現在其副檔名的清單與 `*` 中；請勿將其計為
  兩次使用。`*` 提供跨副檔名的順序。
- **開啟或儲存。** 清單並不會說明是哪一種。請與該檔案的 `$MFT` 與 USN
  Journal 記錄、LNK 檔案以及 Jump Lists 交叉比對。
- **項目時間屬於檔案，而非對話方塊。** `ItemModifiedUTC`、`ItemCreatedUTC` 與
  `ItemAccessedUTC` 是寫入該項目時檔案的時間，精度為 2 秒（FAT）。已知資料夾
  （`CLSID_ThisPCDocumentsRegFolder`、`Downloads`、…）的項目通常沒有時間。這些 FAT
  時間戳記沒有時區位移。欄位名稱中的 `UTC` 後綴只是顯示慣例，並不能
  證明原始時鐘為 UTC；在與登錄檔的 FILETIME 比較之前，請先與來源系統的時區及
  其他時間戳記交叉比對。請參閱 Microsoft 的
  [檔案時間說明](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times)。
- **路徑是殼層路徑。** 它們從桌面開始：`This PC\C:\Users\…`、已知資料夾、
  媒體櫃（`UsersLibraries\Documents\…`）或網路位置。以資料夾或檔案開頭的路徑
  位於桌面上。空白的路徑就是桌面本身。
- **未解碼的項目。** 殼層項目剖析器不認得的類型，會以其
  類別（`[class 0x00]`）顯示，類型為 `unknown`，而清單的位元組則放在 `DataRaw` 中。
- **程式以名稱表示。** `LastVisitedPidlMRU` 與 `CIDSizeMRU` 保存的是檔案名稱
  （`WINWORD.EXE`），而非路徑。`FirstFolder` 則保存路徑。
- **清單很短且會被清除。** 每份清單只保存有限數量的項目，清理工具也會
  刪除這些機碼。對於已刪除的值，請使用 `parse-registry read --recover-deleted`。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry open-save-mru -d evidence/ -o opensavemru.csv
juntei parse-registry open-save-mru -f NTUSER.DAT -t jsonl \
  | jq 'select(.List == "OpenSavePidlMRU" and .Extension != "*")'
```

摘要會列出每份清單的項目、使用過對話方塊的程式，以及每個登錄區中最新的
檔案（`*` 的第一個項目）。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | 主機、設定檔擁有者（取自路徑或殼層資料夾）、取自系統 `ProfileList` 的 SID，以及登錄區。 |
| `List` | `OpenSavePidlMRU`、`LastVisitedPidlMRU`、`LastVisitedPidlMRULegacy`、`CIDSizeMRU`、`FirstFolder`、`OpenSaveMRU` 或 `LastVisitedMRU`。 |
| `Extension` | 副檔名的子機碼（`docx`、`*`）；其他清單則為空白。 |
| `MRU-Position`, `IsMostRecent` | 在機碼清單中的位置（0 為最新）；若清單未指出該值則為空白。 |
| `ValueName` | 值的編號（`MRUListEx`）或字母（`MRUList`）。 |
| `Program` | 顯示對話方塊的程式。 |
| `Path` | 檔案或資料夾：串接後的殼層項目名稱，或以文字儲存的路徑。 |
| `ItemType` | 最後一個殼層項目的類型：`file`、`directory`、`root_folder`、`volume`、`network_location`、`property_view`、`unknown`、… |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | 殼層項目所記錄的檔案或資料夾時間。 |
| `MFT-Entry`, `MFT-Sequence` | 殼層項目所記錄的 NTFS 檔案參照。 |
| `LastUsedUTC` | 機碼的最後寫入時間，僅適用於最新的項目。 |
| `KeyPath`, `KeyLastWrittenUTC` | 機碼及其最後寫入時間：這是其最新項目的時間，而非每一列的時間。 |
| `DataRaw` | 無法解碼時該值的位元組（十六進位），或項目類型未知時的殼層項目清單。 |
| `TransactionLogStatus`, `LogsApplied` | 登錄檔復原的脈絡。 |
| `Problems` | 清單有列出但機碼中不存在的值、清單未列出的值、錯誤的類型、被截斷的名稱或清單。 |

## 調查策略 { #investigation-strategy }

1. 依使用者按順序閱讀 `*`，並以 `LastUsedUTC` 確認最新的檔案。
2. 尋找位於使用者可寫入、卸除式或網路位置的指令碼、封存檔、執行檔與文件，
   以及名稱看起來像憑證或匯出資料的檔案。
3. 使用 `LastVisitedPidlMRU` 將程式與其最後使用的資料夾連結起來，並以 `CIDSizeMRU` 與
   `FirstFolder` 找出曾使用過對話方塊的程式。
4. 將路徑與 `MftEntry` 與 `$MFT`、USN Journal、LNK 檔案、Jump Lists、
   ShellBags 以及 RecentDocs 交叉比對。

## 延伸資訊 { #further-details }

- [libyal winreg-kb：Most recently used (MRU) keys](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi：Windows Shell Item format](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
