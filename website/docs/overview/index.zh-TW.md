# 關於 Juntei { #about-juntei }

Juntei 是以記憶體安全的 Rust 撰寫的 Windows 鑑識跡證剖析器。它與
[Hayabusa](https://github.com/Yamato-Security/hayabusa) 相輔相成：Hayabusa 分析 Windows 事件記錄，
而 Juntei 則剖析持久保存的檔案系統、登錄、執行、使用者活動、資源使用
以及資安產品跡證。

同一個執行檔可在 Windows、macOS 與 Linux 上執行。當分析人員收到 KAPE、
Velociraptor、磁碟映像檔或其他離線收集的資料時，這點非常實用：只需要一個一致的介面，
而不必為每種跡證各準備一套程式。

## 設計目標 { #design-goals }

- **可經得起檢驗的剖析：**保留原始值並回報損毀情形，而不是默默地捏造資料。
- **人人適用：**預設用法對初學者而言容易上手，同時對資深人員而言也可自訂且
  功能完整。
- **獨立執行檔：**執行時不需要任何執行階段函式庫。下載即可直接
  使用。
- **極速：**目標是成為最快的剖析器，同時不需要大量
  記憶體。
- **跨平台使用：**不需要 Windows 工作站即可分析 Windows 證據。
- **自動化：**CSV 適用於試算表與時間軸工具；JSON/JSONL 適用於 `jq`、指令碼與資料湖。
- **安全地處理失敗：**盡可能回報並略過格式錯誤的記錄。
- **透明的來源脈絡：**Juntei 會標明每個實作背後所依據的開放原始碼剖析器與參考工具。

## Juntei 不做的事 { #what-juntei-does-not-do }

Juntei 不會擷取線上運作中的磁碟，也無法取代證據保全；它唯一會掛載的磁碟映像檔是 VHDX 映像檔，
且僅以唯讀方式掛載。它
不會自動將所有跡證整合成單一的事件敘述。它會剖析你所提供的
證據，並產生適合調查與交叉比對的記錄。

Juntei 也不會替你進行分析。對證據進行分析需要由 DFIR 專家執行。

## 授權 { #license }

Juntei 以
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
（RPL-1.5）授權發布，著作權屬於 Shirofune Security Co., Ltd.。在 1.0.0 版之前，其原始碼是以
AGPLv3 授權發布。建置時所使用的 crate 及其他第三方程式碼與資料的授權條款，請見
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES)。
Juntei 所依據以及用來比對驗證的專案，列於
[致謝](../acknowledgements.md)。
