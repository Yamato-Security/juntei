# BAM / DAM { #bam-and-dam }

## 什麼是 BAM 與 DAM { #what-bam-and-dam-are }

Background Activity Moderator（BAM）與 Desktop Activity Moderator（DAM）是 Windows
服務（Windows 10 1709 及之後的版本），用來管理應用程式在背景中可以執行的動作。
它們會在 `SYSTEM` 登錄區中，依使用者保存每個程式最後一次執行的時間：

- 每位使用者一個機碼，以該使用者的 SID 命名；
- 每個程式一個值：其名稱為程式在其裝置上的路徑
  （`\Device\HarddiskVolume3\Windows\System32\cmd.exe`）或封裝應用程式的 ID
  （`Microsoft.WindowsCalculator_8wekyb3d8bbwe`）；
- 值資料的前 8 個位元組是其最後一次執行的時間（FILETIME）。

BAM 不需要使用者本身的登錄區，就能將程式與使用者及時間連結起來，而且它也會列出從卸除式
裝置與網路裝置執行的程式。

<span class="evidence-strength">最佳結論：</span>此程式的處理程序曾在此使用者的工作階段中
執行；它最後一次執行是在此時間。BAM 不會計算執行次數，而且在程式被刪除之後，項目仍可能保留。

## 位置 { #location }

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

每位使用者的機碼還有 `Version` 與 `SequenceNumber` 值；它們不是程式，因此會被略過。
從 1709 或 1803 升級的系統可能同時具有兩種配置。

請收集 `SYSTEM` 及其交易記錄檔，以及系統的 `SAM` 與 `SOFTWARE`：它們提供使用者名稱。

## 分析注意事項 { #analyst-cautions }

- **每個程式與使用者只有一個時間。** 只會保留最近一次的執行。Prefetch、Amcache 與
  事件記錄可提供較早的執行紀錄。
- **近期活動。** 據報導，一段時間未執行之程式的項目會在 Windows 啟動時被移除，因此 BAM
  主要顯示近期活動：BAM 中沒有的程式，仍可能在很久以前執行過。
- **是裝置路徑，而非磁碟機代號。** `\Device\HarddiskVolume3` 是 Windows 編號的第三個
  磁碟區，而不是 `C:`。`DevicePath` 與 `Path` 會將裝置與其上的路徑分開；在寫出磁碟機代號
  之前，請先以 `MountedDevices`、`$MFT` 或其他跡證比對該磁碟區。
- **並非所有程式。** BAM 與 DAM 追蹤的是使用者工作階段中的程式；服務大多不會出現。
  `S-1-5-18` 項目通常是安裝程式與登入畫面的處理程序（`FirstLogonAnim.exe`），而
  `S-1-5-90-0-<n>` 項目則是 Desktop Window Manager（`DWM-<n>`）。
- **控制集。** 預設會讀取 Windows 最後使用的控制集。`--all-control-sets` 會讀取所有控制集：
  較舊的控制集可能保留目前控制集已不再有的項目。
- **使用者名稱。** 本機帳戶的名稱取自 `SAM`（電腦的 SID 與帳戶的 RID）；網域帳戶的名稱
  則取自其在 `ProfileList` 中的設定檔資料夾。`UserSource` 指出來源為何（`sam`、
  `profile_list`、`well_known`）。若沒有這些資料，就只會提供 SID。`SAM` 中已不存在的
  RID（已刪除的帳戶）則維持未命名。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

摘要會列出每位使用者的 SID、名稱與項目數量，以及最後執行的程式（由新到舊）。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`SourcePath` | 主機，以及 `SYSTEM` 登錄區。 |
| `ControlSet`、`IsCurrentControlSet` | 項目所在的控制集，以及它是否為 Windows 最後使用的控制集。 |
| `Moderator` | `bam` 或 `dam`。 |
| `UserSID`、`User`、`UserSource` | 機碼所依據命名的 SID、使用者名稱，以及名稱的來源（`sam`、`profile_list`、`well_known`）。 |
| `Program` | Windows 所寫入的值名稱：裝置路徑或封裝應用程式的 ID。 |
| `DevicePath`、`Path` | 對於裝置路徑，為裝置（`\Device\HarddiskVolume3`）及其上的路徑（`\Windows\System32\cmd.exe`）；封裝應用程式則為空。 |
| `LastRunUTC` | 程式最後一次執行的時間（精確至 100 奈秒）。 |
| `KeyPath`、`KeyLastWrittenUTC` | 使用者的機碼及其最後寫入時間：即其任一項目最近一次更新的時間。 |
| `DataRaw` | 當值不含時間時（非 `REG_BINARY`，或少於 8 個位元組），該值的位元組（十六進位）。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡。 |
| `Problems` | 不含時間的資料，以及無法讀取的值。 |

## 調查策略 { #investigation-strategy }

1. 依 `LastRunUTC` 排序，並查看位於使用者可寫入位置（`\Users\`、`Downloads`、
   `Temp`、`ProgramData`）、系統磁碟區以外的裝置上的程式，以及管理工具與遠端存取工具。
2. 確認是哪位使用者執行了它們，以及與目前設定檔不相符的帳戶。
3. 與 Prefetch（執行次數與較早的執行）、UserAssist（從 Explorer 啟動）、Amcache
   與 ShimCache（存在跡證）、`$MFT`（程式的檔案）以及事件記錄進行關聯分析。
4. 使用 `--all-control-sets`，並以 `parse-registry read --recover-deleted` 找出已刪除的 BAM 值。

## 延伸資訊 { #further-details }

- [libyal winreg-kb：Background activity moderator (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft：安全性識別碼（常見的 SID）](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

超出範圍的最後執行 FILETIME 會讓 `LastRunUTC` 保持空白，並將原始值保留在 `DataRaw` 中，同時在 `Problems` 中附上說明。近期活動摘要會標示每台主機，並將不同登錄區／使用者的觀察結果分開保存。
