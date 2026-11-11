# RecentDocs { #recentdocs }

## 什麼是 RecentDocs { #what-recentdocs-is }

Windows 檔案總管會在每位使用者的 `NTUSER.DAT` 中儲存檔案與資料夾的最近使用（MRU）清單。
根清單涵蓋各種類型的最近使用項目；子機碼則依副檔名（例如 `.docx`）分組，另外還有
`Folder` 清單。`MRUListEx` 指定編號值的順序。

RecentDocs 有助於識別使用者曾互動過的文件，包括之後被刪除或移動的
檔案。值中也可能包含 Windows 在使用者「最近使用的項目」資料夾中所使用的捷徑名稱，
可與 LNK 證據交叉比對。

<span class="evidence-strength">最佳結論：</span>檔案總管曾在該使用者的
RecentDocs 清單中記錄此項目。機碼時間只能標示該特定清單中最近一個項目的時間。

## 位置 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

請收集每個相關設定檔的 `NTUSER.DAT` 及其交易記錄，以及該使用者的「最近使用的項目」
LNK 目錄。

## 分析注意事項 { #analyst-cautions }

- `MRUListEx` 提供的是順序，而非每個項目的時間戳記。
- 機碼的最後寫入時間對應最近一次的更新；只有 MRU 位置 0 能從中推導出
  站得住腳的 `LastOpenedUTC`。
- 同一個項目可能同時出現在根清單與副檔名清單中。請勿重複計算為兩次開啟。
- 應用程式與檔案總管的行為可能不同；某些存取方式可能不會更新 RecentDocs。
- 清理／隱私工具與清單大小限制會移除較舊的證據。
- 檔名可能受攻擊者控制，不能證明內容類型。
- 使用者推斷來自設定檔路徑或 Shell 資料夾資料；請檢視空白或不明確的使用者名稱。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry recent-docs <INPUT> [OPTIONS]

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
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recent-docs -d evidence/ -t jsonl \
  | jq 'select(.Extension == ".docx" or .Extension == ".pdf")'
```

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host` | 在可用時，從收集／系統脈絡推導出的主機。 |
| `User` | 從路徑或 Shell 資料夾路徑推斷出的設定檔擁有者；為空時請保留不確定性。 |
| `SourcePath` | 輸入的 `NTUSER.DAT`。合併多位使用者的資料時不可或缺。 |
| `Extension` | 子清單（`.docx`、`Folder` 等）；根 RecentDocs 清單則為 null。 |
| `MRU-Position` | 零表示在此清單中最近使用。這是相對順序，而非經過時間。 |
| `ValueIndex` | `MRUListEx` 所參照的編號登錄值。僅作為結構上的樞紐。 |
| `Filename` | 解碼後的項目名稱。請透過 LNK、Jump List、MFT 或應用程式資料交叉比對完整路徑。 |
| `LinkName` | 編碼在 Shell 項目中的捷徑名稱，通常可在使用者的「最近使用的項目」目錄中找到。 |
| `KeyLastWrittenUTC` | 清單機碼最後變更的時間。適用於整份清單，而非每一列。 |
| `LastOpenedUTC` | 僅在 MRU 位置 0 時填入，因為此時機碼時間支持這種判讀。 |
| `ValueDataRaw` | 解碼失敗或證據需要驗證時的原始位元組。 |
| `TransactionLogStatus`、`LogsApplied` | 是否重播了登錄交易記錄，以及重播了哪些。 |
| `Problems` | 遺失的 MRU 值、未列入清單的值、錯誤的類型或解碼損毀。 |

## 調查策略 { #investigation-strategy }

1. 分開每位使用者的根清單與副檔名清單；謹慎地去除相同項目的重複。
2. 使用 MRU 位置排序，並且只以 `LastOpenedUTC` 作為帶時間戳記的主張。
3. 將 `LinkName`／`Filename` 與「最近使用的項目」LNK、Jump List、`$MFT` 及 USN 資料比對。
4. 檢視誘餌名稱、雙重副檔名、網路／卸除式目標，以及初始存取前後的文件。
5. 當執行行為很重要時，與應用程式的 Prefetch／SRUM 及事件記錄交叉比對。

## 延伸資訊 { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [libyal libfwsi](https://github.com/libyal/libfwsi)
- [SANS Windows 鑑識分析海報](https://www.sans.org/posters/windows-forensic-analysis/)
