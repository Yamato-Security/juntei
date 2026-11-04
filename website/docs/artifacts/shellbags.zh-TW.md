# ShellBags { #shellbags }

## 什麼是 ShellBags { #what-shellbags-are }

Explorer 會記住使用者開啟的每個資料夾如何顯示：其檢視方式、圖示大小、排序順序與視窗位置。
為了再次找到資料夾的設定，它會將資料夾的身分以 Shell 項目樹狀結構保存在使用者的登錄中，這些項目
與連結的目標 ID 清單所保存的項目相同：先是磁碟機，接著逐層往下直到所開啟的資料夾。這些樹狀
結構就是 ShellBags。

它們涵蓋在 Explorer 以及開啟與儲存對話方塊中開啟的資料夾，包括卸除式磁碟機、網路共用資料夾、
可攜式裝置（手機、相機）、ZIP 壓縮檔與 FTP 伺服器上的資料夾，以及控制台頁面。即使資料夾、
磁碟機或網路共用資料夾已不存在，它們仍會保留。

<span class="evidence-strength">最佳結論：</span>此使用者的 Explorer 曾顯示此資料夾（或其下的
某個資料夾）。ShellBag 不會顯示在資料夾中做了什麼，也不會為每一次瀏覽標示日期：請參閱下方的時間說明。

## 位置 { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

`BagMRU` 機碼保存編號的值（每個值為一個 Shell 項目），以及一個表示其順序的 `MRUListEx` 值，
最近開啟的排在最前面。每個值都有一個同名的子機碼，用於其下的項目，因此路徑就是從 `BagMRU`
往下到某個值的各個項目。子機碼的 `NodeSlot` 是其 `Bags\<N>` 機碼的編號，資料夾的檢視設定就
保存在該處。

請收集每個設定檔的 `UsrClass.dat` 與 `NTUSER.DAT` 及其交易記錄檔（`.LOG1`、`.LOG2`），以及
系統的 `SOFTWARE` 登錄區，它提供 `NTUSER.DAT` 使用者的 SID。

## 各時間代表什麼 { #which-times-say-what }

- **Shell 項目的時間**（`ItemModifiedUTC`、`ItemCreatedUTC`、`ItemAccessedUTC`）是 Explorer
  寫入該項目時，資料夾本身的檔案系統時間（FAT 精度：2 秒）。它們標示的是資料夾的時間，而非
  瀏覽的時間，且在之後的瀏覽中不會更新。
  FAT 時鐘沒有儲存 UTC 時差。沿用的 `UTC` 標題與呈現的 `Z` 字尾只是顯示慣例，並不能證明來源
  時區。`auto-parse --timezone` 會保留這些欄位原樣；請依據來源系統的脈絡判讀它們。
- **機碼的最後寫入時間**會在其 `MRUListEx` 被重新寫入時改變，也就是當其中某個項目被開啟並移到
  最前面時。因此，項目的值所在機碼的時間（`ParentKeyLastWrittenUTC`）**只有在該項目是該機碼中
  最新的項目時**（`IsMostRecentInParent` 為 true、`MruPosition` 為 0），才是該項目最後一次被開啟的
  時間。對其他項目而言，它是較晚的時間，也就是某個同層項目被開啟的時間。
- 項目本身的機碼（`KeyLastWrittenUTC`）以相同方式標示其最新子項目被開啟的時間。

## 分析注意事項 { #analyst-cautions }

- **不存在幾乎無法證明什麼。** Explorer 會在儲存資料夾的檢視時寫入 ShellBags；只被短暫瀏覽過，
  或以其他方式（命令列、指令碼、程式）開啟的資料夾，可能沒有 ShellBag。使用者也可能刪除這些機碼。
- **NTFS 參照屬於資料夾**（`MftEntry`、`MftSequence`），為寫入該項目時的值。請在 `$MFT` 中查詢它，
  以確認相同的資料夾是否仍存在，或是否有其他檔案重複使用了該項目。
- **卸除式磁碟機**以其磁碟機代號命名。該代號是磁碟機當時所使用的代號；請將其與 USB 及
  已掛載裝置的歷程比對，以找出裝置。
- **名稱來自項目本身。** 之後被重新命名的資料夾在此處仍保留其舊名稱。根資料夾與已知資料夾的
  命名方式與鑑識工具常用的名稱相同（`This PC`、`Quick Access`、`Downloads`）；`Guid` 提供其識別碼。
- **搜尋結果、媒體櫃與控制台頁面**是檢視，而非磁碟上的資料夾
  （`property_view`、`control_panel_item`、`control_panel_category`）。
- **沒有任何剖析器認得的項目**會將其位元組保留在 `DataRaw` 中，`ItemType` 為 `unknown`，其類別
  位元組則位於 `ItemClass` 中。損毀或不尋常的項目會在 `Problems` 中回報，其位元組也會被保留。
- 若沒有交易記錄檔，**dirty 登錄區**可能缺少其最新的項目。除非指定 `--no-logs`，Juntei 會重播
  登錄區旁的記錄檔；`TransactionLogStatus` 會指出記錄是否來自這些記錄檔。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shell-bags -d evidence/ -o shellbags.csv
juntei parse-registry shell-bags -f UsrClass.dat -t jsonl \
  | jq 'select(.ItemType == "volume" and .GUID == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

Shell 項目以 `parse-file lnk` 的 Shell 項目剖析器 ys-lnk-parser 解碼。摘要會依據父機碼的
最後寫入時間，列出最後開啟的資料夾（由新到舊）及其使用者。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`User`、`UserSID`、`SourcePath`、`HiveType` | 主機、設定檔資料夾的名稱、使用者的 SID（`UsrClass.dat` 的根機碼會記錄它；`NTUSER.DAT` 則取自系統的 `ProfileList`）、登錄區及其類型。 |
| `BagKey` | 項目的機碼，位於登錄區根目錄之下（`…\BagMRU\4\3\0`）：其值位於上一層機碼中。 |
| `MRU-Position`、`IsMostRecentInParent` | 在其父機碼 `MRUListEx` 中的位置（0：最近開啟）；清單中沒有它時為空。 |
| `NodeSlot` | 其 `Bags\<N>` 機碼的編號（資料夾的檢視設定）。 |
| `Path` | 從其樹狀結構頂端往下到它的各個項目，以 `\` 串接。`[unknown]` 代表值遺失的項目。 |
| `ItemType`、`ItemClass` | 項目的種類（`root_folder`、`volume`、`directory`、`file`、`network_location`、`uri`、`ftp_folder`、`compressed_folder`、`portable_device`、`portable_device_folder`、`users_files_folder`、`property_view`、`control_panel_item`、`control_panel_category`、`cd_burn`、`unknown`），以及其類別位元組。 |
| `ItemName`、`GUID` | 其名稱（資料夾的長名稱），以及根資料夾、控制台項目或委派資料夾的類別識別碼。 |
| `ItemModifiedUTC`、`ItemCreatedUTC`、`ItemAccessedUTC` | **取自 Shell 項目：**寫入該項目時資料夾的時間（FAT，2 秒精度）。 |
| `MFT-Entry`、`MFT-Sequence` | **取自 Shell 項目：**資料夾的 NTFS 檔案參照。 |
| `ExtensionBlocks` | 項目具有的擴充區塊數量。 |
| `KeyLastWrittenUTC` | **取自登錄：**項目本身機碼的最後寫入時間：其最新子項目被開啟的時間。 |
| `ParentKeyLastWrittenUTC` | **取自登錄：**其值所在機碼的最後寫入時間：若 `IsMostRecentInParent` 為 true，即為它最後一次被開啟的時間。 |
| `DataRaw` | 對於未完全理解或有問題的項目，該值的位元組（十六進位）。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡。 |
| `Problems` | 機碼的 `MRUListEx` 列出但機碼中沒有的值、清單未列出的值、沒有對應值的子機碼、非二進位的值、類型錯誤的 `NodeSlot`、損毀的 Shell 項目清單。 |

## 調查策略 { #investigation-strategy }

1. 列出卸除式磁碟機（具有 Removable Drives `Guid` 的 `volume`）、網路位置、FTP 資料夾與
   可攜式裝置，以及其下的資料夾。
2. 尋找值得關注的資料夾：使用者可寫入的資料整備位置（`Temp`、`ProgramData`、`Public`）、
   工具與壓縮檔資料夾、其他使用者的設定檔，以及已不存在的資料夾（請在 `$MFT` 中檢查 NTFS 參照）。
3. 只使用 `IsMostRecentInParent` 為 true 之項目的 `ParentKeyLastWrittenUTC` 來標示瀏覽日期；
   其餘項目請與標示檔案開啟日期的 LNK 檔案、Jump Lists 與 RecentDocs 進行關聯分析。
4. 使用 `parse-registry read --recover-deleted` 找出已刪除的 `BagMRU` 機碼與值。

## 延伸資訊 { #further-details }

- [libyal libfwsi：Windows Shell Item 格式](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb：Most recently used (BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

從 `ProfileList` 查詢 SID 時，需要一個唯一且語法有效的 SID。模稜兩可的設定檔資料夾對應，以及 `S-1-...bak` 之類的備份機碼，會讓 `UserSid` 維持未知。
