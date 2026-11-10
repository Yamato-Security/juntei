# Persistence Hunter 致謝 { #persistence-hunter-acknowledgements }

Juntei 的 `persistence-hunter` 命令建立在兩代社群持續存取獵捕成果的基礎之上。
感謝讓調查人員得以運用其技術、實作與研究的作者與貢獻者。

## PyrsistenceSniper：Python 專案 { #pyrsistencesniper-the-python-project }

由 [Hexastrike](https://github.com/Hexastrike) 開發的
[**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper) 是本功能的直接參考。Juntei
將其包含 130 項檢查的目錄移植為原生 Rust，並一併移植了預設允許清單、LOLBAS 快照與分類行為。參考版本固定於
[commit `b99c117`](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f)。

該 Python 專案採用 MIT 授權。Juntei 將其著作權與授權聲明保留於
[`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt)。
Juntei 的實作可在無 Python 直譯器的環境下離線執行。其收集資料的處理方式、涵蓋範圍回報，以及對不完整證據的保守判讀，
均記載於[命令指南](../commands/persistence-hunter.md)。

## PersistenceSniper：原始的 PowerShell 工具 { #persistencesniper-the-original-powershell-tool }

[**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper) 是啟發 PyrsistenceSniper
的原始 PowerShell 持續存取獵捕專案。PyrsistenceSniper 將功勞歸於
Federico Lagrasta；原始儲存庫也列出了 Riccardo Ancarani 及其他
貢獻者。感謝原始專案及其貢獻者讓 Windows 持續存取機制更易於調查。

Juntei 直接移植的參考對象是該 Python 專案，而原始的 PowerShell 工具則因此源流而列名致謝。Juntei 也獨立實作了
[hongson11698](https://github.com/hongson11698) 在原始專案中提出的
[排程工作 COM 解析 PR #31](https://github.com/last-byte/PersistenceSniper/pull/31)
與 [開始功能表 library／CLSID 資料夾 PR #32](https://github.com/last-byte/PersistenceSniper/pull/32)
所描述的機制。這些項目是對 Python 參考目錄的補充；但這並不表示兩個上游專案中所有針對執行中系統的功能
都已由 Juntei 的離線命令實作。

## 輔助研究與工具 { #supporting-research-and-tools }

- [MITRE ATT&CK](https://attack.mitre.org/) 提供了目錄中所載的技術識別碼。技術對應描述的是某個機制
  可能的用途；相符的登錄值或檔案本身並不構成經 ATT&CK 確認的攻擊。
- [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
  也被 PyrsistenceSniper 列為檢查 Windows 自動啟動項目的靈感來源。
- [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS) 記錄了可能遭濫用的合法 Windows 二進位檔。
  辨識出這類二進位檔有助於判讀命令，但並不會使該命令自動成為惡意命令。

關於 Juntei 全體所使用的登錄檔、NTFS、Shell Link、事件記錄及其他剖析器基礎，
請參閱[專案整體致謝](../acknowledgements.md)。

返回[持續存取機制指南](index.md)。
