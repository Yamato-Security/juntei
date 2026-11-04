# 安全性描述元 (`$Secure:$SDS`) { #security-descriptors-securesds }

## 什麼是 `$SDS` { #what-sds-is }

NTFS 上的每個檔案與目錄都有一個**安全性描述元**：其擁有者、主要群組、
**DACL**（誰可以對它做什麼）以及 **SACL**（哪些存取會被稽核，以及其完整性標籤）。
NTFS 將每個不同的描述元只保存一次，放在 `$Secure` 檔案（MFT 項目 9）的 `$SDS` 資料流中，
並賦予它一個**安全性 ID**。檔案的 `$STANDARD_INFORMATION` 只記錄該安全性 ID：即
[`parse-ntfs mft`](mft.md) 的 `SecurityID` 欄位。`$SDS` 則將它轉換為擁有者與 ACL。

`$SDS` 以 256 KiB 的區塊組成，每個區塊後面接著一份其複本（鏡像）。每個項目都有
其描述元的雜湊值，NTFS 用它來尋找已存在的描述元（`$SDH`），另外還有
安全性 ID（由 `$SII` 建立索引）。

<span class="evidence-strength">最佳結論：</span>`SecurityID` 指向某描述元的檔案，在其
`$MFT` 記錄最後一次寫入時，具有該擁有者與這些 ACL。檔案的
擁有者通常是建立它的帳戶，但系統管理員、安裝程式或取得擁有權的程式
都可以變更它。

## 位置與收集 { #location-and-collection }

```text
C:\$Secure:$SDS   (MFT entry 9, the $SDS data stream)
```

請使用能讀取原始資料的鑑識工具收集它；由於檔名中不能包含 `:`，收集結果通常將它命名為 `$Secure_$SDS`。
請一併收集同一磁碟區的 `$MFT`。

## 分析注意事項 { #analyst-cautions }

- 擁有者是一個 SID。只有知名 SID（`NT AUTHORITY\SYSTEM`、`BUILTIN\Administrators`、
  `NT SERVICE\TrustedInstaller`……）以及電腦或網域的內建帳戶（`-500`
  `Administrator`……）會顯示名稱。其他 SID 請使用主機的 `SAM`
  （[SAM 帳戶](sam.md)）、其設定檔清單（[系統資訊](system-info.md)）或網域加以解析。
- 描述元由所有具有相同擁有者與 ACL 的檔案共用，通常在這些檔案被刪除後
  仍會留在 `$SDS` 中。在 `$MFT` 中沒有對應檔案的描述元，本身並不代表
  已刪除的檔案。
- ACE 並非存取歷程。DACL 說明誰*可以*存取檔案；事件記錄與其他
  跡證才能說明誰實際存取過。SACL 的稽核項目描述的是設定，而非實際記錄了什麼。
- ACE 依其在 ACL 中的順序列出。Juntei 不會計算有效存取權：拒絕
  項目、繼承、群組成員資格、特殊權限以及擁有者的隱含權限都會產生影響。
- `DACL` 與 `SACL` 以 `icacls` 的方式寫出每個 ACE。`SDDL` 表示所支援的權限，遮罩以十六進位表示，帳戶以 SID 表示。它並不是二進位描述元的逐位元組複本；無法表示的條件式文字會讓它保持為空，並產生診斷訊息。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      Parse one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

摘要會計算描述元的數量，並列出其擁有者及各自擁有的描述元數量、
具有 NULL DACL（完全沒有限制）的描述元，以及任何損毀。

## CSV 輸出欄位 { #csv-output-fields }

| 欄位 | 判讀 |
|---|---|
| `SourcePath` | 輸入檔案。 |
| `SecurityID` | 安全性 ID：與同一磁碟區之 [`parse-ntfs mft`](mft.md) 的 `SecurityID` 結合。 |
| `Hash` | 與描述元一同記錄的雜湊值。 |
| `EntryOffset` | 項目在資料流中的位置。 |
| `FromMirror` | 若為從鏡像區塊讀取的複本則為 `True`：與其項目不同的複本（列在該項目之後），或損毀項目的複本。 |
| `Size` | 項目的大小，包含其 20 位元組的標頭。 |
| `Owner`、`OwnerName` | 擁有者的 SID，若為知名 SID 則附上其名稱。 |
| `Group`、`GroupName` | 主要群組的 SID，若為知名 SID 則附上其名稱。Windows 不會將其用於存取檢查。 |
| `Control` | 描述元的控制旗標：`SE_DACL_PRESENT`、`SE_DACL_PROTECTED`（已封鎖繼承）、`SE_DACL_AUTO_INHERITED`……以 `|` 連接。 |
| `DACL-ACE-Count`、`DACL` | DACL 的項目，以 `; ` 分隔。若沒有 DACL 則為空；若描述元授予所有人所有權限，則為 `NULL DACL: no restriction`。 |
| `SACL-ACE-Count`、`SACL` | SACL 的項目：稽核項目、強制完整性標籤、處理程序信任標籤與資源屬性。 |
| `SDDL` | 以 SDDL 表示的整個描述元。只有在其中某部分未被解碼時才會為空。 |
| `Problems` | 與描述元不符的雜湊值、無法解碼的部分、類型不明的 ACE。 |

`DACL` 與 `SACL` 的每個項目都以 `icacls` 的寫法呈現：帳戶、類型（若不是
允許項目）、繼承旗標以及權限。

| 部分 | 意義 |
|---|---|
| 帳戶 | 知名名稱或 SID。 |
| `(DENY)`、`(AUDIT)`、`(ACCESS_ALLOWED_CALLBACK)`…… | ACE 類型（若不是允許項目）。強制標籤不顯示類型。 |
| `(I)` | 繼承自上層目錄。 |
| `(OI)`、`(CI)`、`(IO)`、`(NP)` | 由檔案繼承（物件繼承）、由目錄繼承（容器繼承）、僅由它們繼承而不套用於此物件（僅繼承），以及僅向下繼承一層（不傳播）。 |
| `(SA)`、`(FA)` | 針對成功或失敗存取的稽核項目。 |
| `(F)`、`(M)`、`(RX)`、`(R)`、`(W)` | 完全控制、修改、讀取和執行、讀取、寫入。 |
| `(RD,WD,AD,…)` | 其他情況下的特定權限：讀取資料、寫入資料、附加資料、讀取與寫入延伸屬性（`REA`、`WEA`）、執行（`X`）、刪除子項目（`DC`）、讀取與寫入屬性（`RA`、`WA`）、刪除（`DE`）、讀取控制（`RC`）、寫入 DAC（`WDAC`）、寫入擁有者（`WO`）、同步處理（`S`），以及一般權限 `GA`、`GR`、`GW`、`GE`。`N` 表示無存取權。 |
| `(NW)`、`(NR)`、`(NX)` | 強制標籤的原則：禁止向上寫入、禁止向上讀取、禁止向上執行。 |
| `if (…)` | 回呼 ACE 的條件，例如 `(WIN://SYSAPPID Contains "…")`：該項目只套用於條件為真的處理程序。Windows 用它來賦予 App Container 套件存取權。 |

處理程序信任標籤與資源屬性會以十六進位顯示其遮罩，資源
屬性還會顯示其宣告（`("IMAGELOAD",TU,0x0,1)`）。

JSON／JSONL 具有相同的欄位與名稱，數字以數字表示，空值為 `null`，
`Problems` 則為清單。

## 調查策略 { #investigation-strategy }

1. 剖析同一磁碟區的 `$MFT`，並將其 `SecurityID` 與此輸出中的同名欄位結合：每個檔案都能取得其
   擁有者與 ACL。
2. 檢視屬於使用者帳戶（`S-1-5-21-…`）的擁有者：可疑帳戶擁有哪些檔案，
   以及哪些帳戶擁有系統目錄中的檔案。
3. 在程式執行或載入來源的目錄上尋找寬鬆的 ACL：具有 `(F)`、`(M)` 或 `(W)` 的
   `Everyone`、`Authenticated Users` 或 `User`，以及 NULL DACL。
4. 僅由單一檔案或由一組不尋常檔案使用的描述元，可能顯示在
   正常安裝程序之外建立或變更的檔案。
5. 以 Security 事件記錄項目（4670、搭配 SACL 的 4663）及
   [USN 日誌](usn-journal.md)的 `SECURITY_CHANGE` 記錄，佐證擁有權與權限的變更。

## 延伸資訊 { #further-details }

- [Microsoft：安全性描述元](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft Open Specifications：MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/)（安全性描述元、ACE、條件運算式）
- [Microsoft：安全性描述元字串格式（SDDL）](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft：icacls](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls)
- [libfsntfs：NTFS 格式文件](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc)（`$Secure`）
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser)（其 `secure` crate）

當結構化輸出寫至 stdout（使用 `-t` 但未使用 `-o`）時，資料流層級的復原警告會寫至 stderr，因此管線中仍能保有可讀的 CSV／JSON／JSONL，且不會隱藏損毀或已復原的區域。
