# YARA 掃描 { #yara-scan }

使用可攜式 Rust 引擎掃描已收集的檔案，支援 Windows、macOS 和 Linux。檔案只會作為資料讀取，不會執行。此命令獨立於自動鑑識資料剖析。

## 選擇規則 { #choose-rules }

預設的 Core 重視準確度與效能。Extended 擴大威脅獵捕範圍。Full 的範圍最廣，也可能產生更多誤報。

```bash
juntei yara-scan -d evidence/ -o yara-core/
juntei yara-scan -d evidence/ --ruleset extended -o yara-extended/
juntei yara-scan -d evidence/ --ruleset full -o yara-full/ -t jsonl
juntei yara-scan -d evidence/ --rules local-rules/ -o yara-offline/
```

`--rules <FILE_OR_DIRECTORY>`: 使用本機 UTF-8 規則檔或目錄，不會下載。目錄會遞迴搜尋。include 僅能引用所選規則目錄內的檔案，依相依順序展開一次。無效或不支援的規則會中止編譯，不會被默默捨棄。

## 限制與範圍 { #limits-and-scope }

所有一般檔案均可掃描，包括還原的隔離檔案。符號連結與特殊檔案會略過。不會解開壓縮檔或磁碟映像，也不掃描即時記憶體。請使用穩定的已擷取檔案，並在證據目錄之外建立新的輸出目錄。

- `-T, --threads <N>`: 平行掃描數；預設為 CPU 數，上限為 8。
- `--timeout <SECONDS>`: 每個檔案的評估逾時秒數；預設為 60。
- `--max-file-mib <MIB>`: 最大檔案大小（MiB），預設為 64。超過限制的檔案會記錄為略過。增加工作執行緒與緩衝區會增加記憶體用量。
- `--rules-release YYYYMMDD`: Forge 發行日期，或預設的 latest。會儲存下載的原始規則及雜湊以利重現。

## 結果 { #results }

- `YaraDetections.csv` / `YaraDetections.jsonl`: 規則與檔案的符合項目，包括相對路徑、證據電腦名稱、雜湊、中繼資料及有限的符合位移。不會匯出符合的位元組。
- `YaraFiles.csv` / `YaraFiles.jsonl`: 逐檔覆蓋狀態，包括略過、讀取錯誤、逾時及部分結果。
- `Summary.json`: 完成狀態、設定、數量、引擎版本與原始規則雜湊。
- `Rules.yar`: 完整的下載規則，或展開後的本機規則快照。
- `CompilationWarnings.txt` / `CompilationError.txt`: 編譯警告，或編譯失敗時的錯誤。

結束代碼：0 表示完成，3 表示覆蓋不完整，2 表示設定或輸出失敗，130 表示取消。符合規則只是調查線索，不代表已確認入侵；沒有符合也不能證明安全。應檢查規則標題及鑑識資料的脈絡。隔離檔案必須先解密才能辨識內容。

## MFT符合與復原 { #mft-recovery }

MFT以原始位元組逐一掃描，使用獨立大小限制（預設512 MiB）。剖析符合位置的記錄，並獨立重新掃描其中的常駐DATA串流。只有串流本身符合規則時才復原。僅涉及中繼資料、跨記錄或非常駐內容的符合會記錄，但不匯出檔案內容。

`--max-mft-mib <MIB>` · `--max-mft-recoveries <N>`

每次執行預設最多復原1000個串流。限制、損壞記錄及逾時均會明確呈現。儲存名稱使用隔離副檔名，不會重建證據的目錄結構。

`YaraMFTResults.csv` / `YaraMFTResults.jsonl` · `RecoveredMFT/` · `.quarantined`

## 進一步資訊 { #further-details }

- [引擎: Boreal](https://github.com/vthib/boreal)
- [規則套件: YARA Forge](https://github.com/YARAHQ/yara-forge)
- [詳細用法與欄位參考（英文）](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.md)
- [詳細用法與欄位參考（日文）](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.ja.md)
