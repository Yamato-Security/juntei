# 致謝 { #acknowledgements }

Juntei 之所以能夠存在，是因為 DFIR 與開放原始碼社群公開了剖析器、格式研究、參考輸出、規格以及測試證據。
以下專案為 Juntei 的實作、驗證、欄位命名或分析人員預期的輸出提供了依據。列於此處並不代表其程式碼已內嵌於
Juntei 中；授權條款請參閱各專案。

## 由 Yamato Security 維護的剖析器基礎 { #parser-foundations-maintained-by-yamato-security }

| Juntei 領域 | 維護的剖析器 | 上游／基礎 |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)（`mft` crate） | [Omer Ben-Amram mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| USN Journal | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)（`usn` crate） | [Sootmark usn](https://github.com/Sootmark/usn) |
| NTFS `$LogFile` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)（`logfile` crate） | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| 目錄索引（`$I30`） | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)（`mft` crate，`i30` 模組） | 下方致謝的索引 slack 研究 |
| NTFS `$Boot`、`$Secure:$SDS` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)（`boot` 與 `secure` crate） | Microsoft 的 MS-DTYP 以及下方致謝的 NTFS 研究 |
| SRUM、UAL / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) 與 [ese](https://github.com/Sootmark/ese) |
| 網頁瀏覽器／SQLite | [ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) | [Sootmark browser](https://github.com/Sootmark/browser)、[sqlite](https://github.com/Sootmark/sqlite) 與 [plist](https://github.com/Sootmark/plist) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark shell](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) | Sootmark shell 的複合檔案與 Jump List 相關成果 |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [Sootmark defender](https://github.com/Sootmark/defender) |
| VHDX 磁碟映像檔（`mount`、`auto-parse --vhdx`） | [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) (Security Ronin `vhdx-core`) 與 [ntfs-core](https://github.com/SecurityRonin/ntfs-forensic)，透過 [fuser](https://github.com/cberner/fuser) 提供 | Microsoft 的 MS-VHDX |
| E01 / Ex01 (`mount`, `auto-parse --e01`) | [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) | [ewf-image](https://github.com/ebrig/ewf-image) (Apache-2.0); [libewf](https://github.com/libyal/libewf) (EWF1/EWF2) |
| Registry | [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) | 下方致謝的登錄檔格式與復原研究 |

## 參考工具與獨立驗證 { #reference-tools-and-independent-validation }

- Hexastrike 的 [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper)
  為原生 Rust 實作的 `persistence-hunter` 命令提供了持續存取檢查項目目錄、預設允許清單、LOLBAS 快照以及參考行為。
  其 MIT 授權與著作權聲明保留於 `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`。原始的
  [PersistenceSniper](https://github.com/last-byte/PersistenceSniper) 啟發了這個 Python 專案；
  完整的源流請參閱[持續存取相關致謝](persistence-hunter/acknowledgements.md)。

- Maxim Suhanov 的 **regf specification** 與 **yarp** 為登錄檔結構、交易記錄重播以及已刪除記錄的復原提供了依據；
  他的 **dfir_ntfs** 則是 INDX 記錄與索引 slack 擷取（carving）的參考。
- Harel Segev 的 **INDXRipper** 是從索引 slack 擷取檔案名稱的另一項參考。
- **dfir_ntfs**、Joakim Schicht 的 **LogFileParser** 以及 Stroz Friedberg 的 **ntfs-linker** 為
  `$LogFile` 的驗證與判讀提供了依據；Velocidex 的 **go-ntfs** 提供了公開的 NTFS 測試證據。
- Joachim Metz／libyal 專案的 **libregf**、**libfsntfs**、**libscca**、**libfwsi** 與
  **libesedb** 提供了詳盡的格式研究與獨立的剖析參考。
- Fox-IT 的 **dissect.esedb**、**dissect.ntfs** 與 **dissect.target** 提供了驗證時所用的 SRUM、UAL、
  `$Secure:$SDS`、SAM 與 Defender 的獨立判讀結果。
- **The Sleuth Kit** 的 `fsstat` 是以
  `mkntfs` 建立之磁碟區 NTFS 開機磁區的參考。
- Eric Zimmerman 的部分工具確立了比較時所使用的欄位名稱與鑑識人員的預期。
- **regipy**、**python-registry**、**Notatin** 與 **artemis** 提供了登錄檔實作、
  測試或範例判讀結果。
- **LnkParse3**、AbdulRhmanAlfaifi 的 **lnk_parser** 與 **jumplist_parser**，以及 JLECmd 提供了
  獨立的 LNK／Jump List 行為與公開測試案例。
- Security Ronin 的 **prefetch-forensic** 與 **forensicnomicon** 專案為 Juntei
  高價值的 Prefetch／Amcache／ShimCache 觀察結果提供了依據。
- **defender-dump** 與 dissect.target 提供了各自獨立的實作，有助於理解 Defender
  隔離資料結構與安全的擷取行為。
- **Plaso**、**Chainsaw** 與 **Velociraptor** 在多種跡證上提供了剖析器比較、測試證據，
  或以收集為導向的參考行為。
- Obsidian Forensics 的 **Hindsight** 與 Velocidex 的 **SQLiteHunter** 定義，是瀏覽歷程、下載、
  Cookie 與表單歷程的參考依據。
- Abel Cheung 的 **rifiuti2** 是資源回收筒 `$I` 檔案與 INFO2 的參考依據，libyal 的 **dtformats**
  文件則說明了這兩種格式。

## 規格與研究 { #specifications-and-research }

感謝 Microsoft 公開 Shell Links、Compound Binary Files、NTFS／SAM 協定、登錄檔／服務／工作結構的
Open Specifications 以及 API 文件。同時也感謝 AppCompatCache、Amcache、SRUM、Prefetch、登錄檔與 NTFS
鑑識研究的作者們，讓實務人員得以理解這些跡證。

## 測試證據 { #test-evidence }

Andrew Rathbun 的
[DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum)、NIST CFReDS、Plaso、
rifiuti2、Regipy、Artemis、各剖析器測試儲存庫以及其他授權妥當的資料集所提供的公開證據，使得以真實 Windows
資料進行回歸測試成為可能。Juntei 會在所收錄的測試檔案旁保留出處與授權說明。

## 貢獻者 { #contributors }

感謝每一位 Juntei 與 Yamato Security 剖析器的貢獻者、審查者、問題回報者，以及提供邊界案例的使用者。
目前的貢獻者列於 GitHub 的
[Juntei 貢獻者頁面](https://github.com/Yamato-Security/juntei/graphs/contributors)。

## 授權 { #license }

Juntei 以
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
（RPL-1.5）發布，著作權屬於 Shirofune Security Co., Ltd.。其原始碼在 1.0.0 版之前以
AGPLv3 發布。建置時所使用的 crate 及其他第三方程式碼與資料之授權條款，請參閱
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES)。
