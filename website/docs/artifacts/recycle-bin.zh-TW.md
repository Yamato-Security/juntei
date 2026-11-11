# 資源回收筒 { #recycle-bin }

## 資源回收筒記錄了什麼 { #what-the-recycle-bin-records }

當使用者在檔案總管中刪除檔案或資料夾時，Windows 會將它移至所在磁碟區的資源回收筒，並在旁邊寫入
一筆小型索引記錄：它原本的路徑、大小，以及刪除的時間。每位使用者在每個磁碟區上都有一個以其 SID
命名的回收筒資料夾，因此這個資料夾也能說明是哪個帳戶刪除了該項目。

| Windows 版本 | 索引 | 已刪除的內容 |
|---|---|---|
| Vista 及更新版本 | 每個項目一個 `$I…` 檔案（Vista 至 8.1 為格式版本 1，Windows 10 及更新版本為版本 2） | `$R…`，`$R` 之後的名稱與索引檔相同：即該檔案，或連同內容的資料夾 |
| NT 4.0、2000、XP、Server 2003 | 每個回收筒一個 `INFO2` 檔案：800 位元組的記錄，含 ANSI 與 Unicode 兩種路徑 | `D<drive letter><index><extension>`，例如 `Dc1.txt` |
| 95、98、ME | 一個 `INFO2` 檔案（Windows 95 為 `INFO`）：280 位元組的記錄，只有 ANSI 路徑 | 同樣的 `D…` 名稱 |

<span class="evidence-strength">最佳結論：</span>保存此記錄之回收筒所屬的帳戶，曾在所記錄的時間將
位於原始路徑的項目移至資源回收筒。使用 Shift+Delete、從命令列或從大多數程式刪除時，都會略過
資源回收筒，不留下任何記錄。

## 位置與收集 { #locations-and-collection }

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

請收集每個磁碟區回收筒中的 `$I` 檔案或 INFO2。若同時收集 `$R` 與 `D…` 檔案，即可看出哪些已刪除的
項目仍在回收筒中；若未收集，每個項目的 `RecycledFileFound` 都會是 `false`。已刪除的內容本身就是
證據，也可能是惡意程式：請以相應的方式處理。

## 分析注意事項 { #analyst-cautions }

- `DeletedTime` 是項目被移至回收筒的時間，依系統時鐘記錄，格式為 UTC FILETIME。它並不是清空
  回收筒的時間。項目本身的建立與修改時間位於 `$MFT` 與 `$R` 檔案中。
- SID 代表刪除該項目的帳戶，不一定是檔案的擁有者。若某個 SID 在 SOFTWARE 登錄區的 `ProfileList`
  中沒有設定檔（已刪除的帳戶，或從未登入過的網域帳戶），就不會有 `User`。
- 清空回收筒，或從回收筒還原或刪除某個項目時，Windows Vista 及更新版本會刪除其 `$I` 與 `$R` 檔案。
  它們的名稱可能仍留在 USN Journal、`$MFT` 與 `$I30` slack 中；Juntei 不會從這些來源復原已刪除的
  `$I` 記錄。INFO2 則會保留這類記錄，並清除其 ANSI 路徑的第一個位元組：此時 `RemovedFromBin` 為
  `true`，而 Juntei 會依磁碟機編號補回該字元。
- `$I` 檔案中的 `FileSize` 是項目的大小；若為資料夾，則是其內容的總大小。在 INFO2 中，它是項目在
  磁碟上所占用的空間，以完整叢集計算（小型檔案為 4,096 位元組）。
- INFO2 的 ANSI 路徑通常是 8.3 格式，且需要系統的字碼頁：日文 Windows 請使用 `--code-page 932`，
  中文則使用 936 或 950。若有 Unicode 路徑，`OriginalPath` 即為該路徑。
- 部分 Windows Vista 系統寫入的是 543 位元組的 `$I` 檔案，其大小欄位少了一個位元組。Juntei 會讀取
  其時間與路徑，並將 `FileSize` 留空，同時回報一項問題。
- `RecycledFileFound` 描述的是收集內容，而非端點：`false` 可能表示收集工具未取得 `$R` 檔案，或該
  項目已不在回收筒中。
- 未安裝 Windows 的磁碟區上的回收筒，若收集內容中恰好只有一個 Windows 安裝，就會從該安裝取得
  使用者名稱。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

使用 `-d` 時，Juntei 會讀取名稱為 `$I` 後接六個英文字母或數字（副檔名不限）的檔案、`INFO2`，以及
位於 `RECYCLED` 資料夾中的 `INFO`。使用 `-f` 時，無論檔名為何，都會讀取該檔案。

## CSV 輸出欄位 { #csv-output-fields }

| 欄位 | 意義 |
|---|---|
| `SourceFile` | `$I` 或 INFO2 檔案。 |
| `DeletedTime` | 項目被移至回收筒的時間（UTC）。 |
| `OriginalPath` | 項目原本所在的位置。對於 INFO2，若記錄含有 Unicode 路徑則採用該路徑，否則採用 ANSI 路徑。 |
| `FileSize` | `$I`：項目的大小，以位元組為單位（資料夾為總大小）。INFO2：它在磁碟上所占用的空間，以完整叢集計算。無法讀取時為空白。 |
| `User`, `SID` | 回收筒資料夾的 SID，以及取自其 `ProfileList` 設定檔名稱的使用者。 |
| `RecycledFile`, `RecycledFileFound` | 位於索引旁之已刪除內容的路徑（`$R…`、`D<drive><index><extension>`），以及它是否在收集內容中。 |
| `Format`, `FormatVersion` | `$I` 1（Vista 至 8.1）或 2（Windows 10 及更新版本）；`INFO2` 0（95）、2（NT 4.0）、4（98）或 5（ME、2000 至 Server 2003）。 |
| `RecordIndex`, `Drive`, `RecordOffset` | INFO2：記錄的編號（即其 `D…` 檔名中的數字）、磁碟機，以及記錄在 INFO2 中的起始位置。UNC 路徑沒有磁碟機。 |
| `RemovedFromBin` | INFO2：項目已從回收筒還原或刪除時為 `true`。 |
| `ShortPath` | INFO2：與 `OriginalPath` 不同時的 ANSI 路徑（通常為 8.3 名稱）。 |
| `Problems` | 損毀情形：檔案遭截斷或過長、磁碟機編號無效、路徑不是有效的 UTF-16、刪除時間不是有效的時間值。 |

JSON 與 JSONL 具有相同的欄位，但數字與布林值帶有型別，記錄中沒有的值為 `null`，`Problems` 則為
清單。

## 自動剖析 { #automatic-parsing }

`auto-parse` 會寫入 `FileExistence/RecycleBin.csv`（使用 `-t json` 或 `-t jsonl` 時則為
`RecycleBin.json` 或 `RecycleBin.jsonl`），並附上 `FileExistence/RecycleBin-logs.txt`。其
`--code-page` 適用於 INFO2。從磁碟映像檔剖析時，會複製 `$I` 檔案與 INFO2；`$R` 與 `D…` 檔案則為
空白的預留位置，因此 `RecycledFileFound` 仍會有值。合併的 `-j` 時間軸包含每一筆資料列，並帶有
`Artifact` 識別碼 `recycle-bin`；`-Z` 會轉換 `DeletedTime`；
[`super-timeline`](../commands/super-timeline.md) 則會為每個項目產生一個
`Recycle Bin deletion (moved to the bin)` 事件。

## 調查策略 { #investigation-strategy }

1. 依使用者分別以 `DeletedTime` 排序，並與你所調查之活動的時間相比較。
2. 在 LNK、Jump Lists、ShellBags 與 RecentDocs 中查找 `OriginalPath`，確認該項目在刪除前是否曾被使用。
3. 在 USN Journal 中，刪除會在 `DeletedTime` 前後顯示為重新命名並移入 `$Recycle.Bin`；`$MFT` 則提供
   項目本身的時間。
4. 當 `RecycledFileFound` 為 `true` 時，請對 `$R` 或 `D…` 內容計算雜湊值並加以檢查。
5. 若回收筒已被清空，請在 USN Journal 與 `$MFT` 中搜尋 `$I` 與 `$R` 名稱。

## 延伸資訊 { #further-details }

- [libyal dtformats — Windows Recycle.Bin 檔案格式](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler 檔案格式](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — 資源回收筒與 INFO2 剖析器](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
