# 第一次調查 { #first-investigation }

此流程假設你已在 `evidence/` 中取得一份離線的分類收集資料（triage collection）。

## 1. 保全來源 { #1-preserve-the-source }

請在經過驗證的副本上作業。記錄雜湊值、收集時間、時區、主機、磁碟區與收集工具。Juntei 只會讀取
證據，但常駐資料流擷取或隔離資料擷取等選項會建立新的輸出檔案；請將它們導向獨立的分析目錄。

## 2. 清點手邊的資料 { #2-inventory-what-you-have }

請尋找[收集證據](collection.md)中列出的路徑。缺少部分跡證是正常的。不要因為收集資料中含有
登錄區，就假設其中也包含交易記錄檔。

## 3. 先執行摘要 { #3-run-summaries-first }

許多命令在未要求輸出檔案時，會印出簡潔的摘要：

```bash
juntei parse-file prefetch -d evidence/
juntei parse-registry shimcache -d evidence/
juntei parse-file defender-quarantine -d evidence/
```

在信任記錄之前，請先閱讀損毀、dirty 資料庫與未套用記錄檔的警告。

若要在一次作業中解析多台電腦之收集資料中的所有支援跡證，請使用：

```bash
juntei auto-parse -d evidence/ -o analysis/
```

關於主機探索、平行處理、輸出配置，以及供 SIEM 匯入的整合 JSONL，請參閱[自動解析收集資料](../commands/auto-parse.md)。

## 4. 儲存結構化輸出 { #4-save-structured-output }

```bash
mkdir -p analysis
juntei parse-file prefetch -d evidence/ -T -o analysis/prefetch.csv
juntei parse-ntfs mft -d evidence/ -o analysis/mft
juntei parse-ntfs usn-journal -d evidence/ -o analysis/usn
juntei parse-registry services -d evidence/ -o analysis/services.jsonl -t jsonl
```

撰寫指令碼與處理巢狀值時建議使用 JSONL；使用試算表或時間軸檢視器時則建議使用 CSV。

## 5. 從可靠的識別碼開始樞紐分析 { #5-pivot-from-a-reliable-identifier }

從檔案名稱、完整路徑、SHA-1/SHA-256、使用者 SID、MFT 項目／序號、工作 GUID、服務名稱或時間範圍
開始。只對 Windows 不區分大小寫處理的欄位進行大小寫正規化。

```bash
juntei parse-file prefetch -d evidence/ -t jsonl \
  | jq 'select(.ExecutableName? | ascii_downcase == "suspicious.exe")'
```

## 6. 進行關聯分析 { #6-correlate }

請使用[跨跡證調查流程](../resources/workflows.md)。一個實用的順序是：

1. Prefetch：執行紀錄與最近的執行時間。
2. `$MFT` 與 USN Journal：建立、路徑、重新命名與刪除。
3. Amcache 與 ShimCache：歷史上的存在跡證與中繼資料。
4. LNK、Jump Lists、RecentDocs 與 SRUM：使用者／應用程式脈絡。
5. 登錄中的服務與 TaskCache：持續存取。
6. Defender 隔離資料：偵測、移除、來源與雜湊值。

## 7. 記錄可重現性 { #7-document-reproducibility }

記錄來源路徑、來源雜湊值、Juntei 版本、確切的命令、輸出雜湊值、系統時區，以及之後進行的任何篩選。
絕對不要在未說明的情況下編輯所產生的證據輸出。
