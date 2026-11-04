# 常見輸出方式 { #common-output-patterns }

## 單一跡證或整批收集資料 { #one-artifact-or-a-collection }

```bash
juntei parse-file prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-file prefetch -d evidence/
```

`-d` 會遞迴搜尋適用於該命令的檔案。來源路徑欄位可在合併的輸出中區分主機與跡證。

## 儲存 CSV { #save-csv }

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

有些剖析器會輸出多個 CSV 表格，因此需要指定目錄：

```bash
juntei parse-registry amcache -f Amcache.hve -o amcache-tables
juntei parse-ese srum -f SRUDB.dat -o srum-tables
juntei parse-file jump-lists -d evidence/ -o jump-list-tables
```

## 以串流輸出 JSONL { #stream-jsonl }

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.ImagePath? | test("temp|public"; "i"))'
```

訊息與進度會輸出至標準錯誤；標準輸出則維持可供剖析的格式。

## 刻意覆寫 { #overwrite-intentionally }

Juntei 預設會拒絕覆寫既有的輸出。請確認目標、保留任何重要的先前結果，然後加上 `-C` / `--clobber`：

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv --clobber
```

## 安靜模式與無色彩模式 { #quiet-and-no-color-modes }

自動化作業請使用 `-q`；不適合使用終端機色彩碼時請使用 `-K`：

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv -q -K
```

## 大小寫與路徑 { #case-and-paths }

含有空格、`$`、Shell 中繼字元或反斜線的 Windows 路徑，請依你的 Shell 需要加上引號。在類 Bash
的 Shell 中，單引號可避免 `$MFT` 與 `$J` 被當成變數：

```bash
juntei parse-ntfs mft -f '/evidence/C/$MFT' -o mft.csv
```
