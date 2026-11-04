# 稽核原則 { #audit-policy }

## 這是什麼 { #what-it-is }

稽核原則決定 Windows 會將哪些安全性事件記錄到 Security 事件記錄中。在 Windows Vista
及之後的版本中，它依子類別設定（Windows 10 與 11 上有 59 個）；在 Windows XP 與
Server 2003 中則依類別設定。每一項都會記錄成功、失敗、兩者，或都不記錄。

無論原則來自 `auditpol`、本機安全性原則或群組原則，LSA 都會將現行生效的原則保存在
`SECURITY` 登錄區中。

<span class="evidence-strength">最佳結論：</span>在登錄區最後一次寫入時，系統被設定為
記錄（或不記錄）這些事件。當某個子類別未被稽核時，事件不存在並不能說明該活動是否曾經發生。

## 位置 { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

Windows Vista 及之後的版本會保存一個 12 位元組的標頭，接著依類別逐一保存每個子類別的
一項設定，最後是每個類別所含的子類別數量。各版本之間的差異只在於這些數量：每個版本都會在
類別的末尾新增子類別（在讀取過的 Windows 8.x 與 Server 2012 R2 登錄區中為 56 個，
Windows 10、11 與 Server 2022 中為 59 個）。Windows XP 與 Server 2003 則保存稽核是否
開啟（一個位元組，之後是 Windows 不會清除的填補位元組）以及九個類別設定。此配置已在
NIST CFReDS Hacking Case 的公開 Windows XP 映像檔上驗證過，該映像檔不稽核任何項目；
其類別設定的值則依循 Microsoft 的文件。

請收集 `SECURITY` 及其交易記錄檔，以及記錄主機名稱的系統 `SYSTEM` 登錄區。

## 分析注意事項 { #analyst-cautions }

- **未知的資料仍維持未知。** 不支援的修訂版本會連同診斷資訊一起保留。未知的設定旗標
  會讓 `Success` 與 `Failure` 保持空白，且不計入已稽核設定的數量。損毀的數量表無法將
  保留字提升為另一項稽核設定。擷取的資料可能包含名稱／GUID 目錄以外的額外欄位。
  這些欄位的設定與數值 `SubcategoryId` 會被保留，但名稱／GUID 為空；未對應的欄位本身
  並非損毀的證據。


- **順序是 Windows 自己的順序。** 在一個類別內，設定是依子類別的訊息 ID 排序，而非依其
  GUID。Special Logon（`%%12548`，Logon/Logoff 的索引 4）排在 IPsec Quick Mode 之前。
  依 GUID 順序讀取的工具會將 Windows 10 的預設值回報為 Special Logon「No Auditing」與
  IPsec Quick Mode「Success」。`SubcategoryId` 與事件 4719（System audit policy was
  changed）所提供的 ID 相同，因此兩者可以相互比對。
- **是現行生效的原則，而非其來源。** 群組原則與本機原則會寫入相同的值。群組原則的
  `audit.csv`（位於 SYSVOL 與 `C:\Windows\System32\GroupPolicy`）顯示的是原本預計套用的
  內容（某個原則的 `Machine\Microsoft\Windows NT\Audit\audit.csv`）。
- **隨時間的變更：**`KeyLastWrittenUTC` 大約是原則最後一次變更的時間。事件 4719 會記錄
  每一次變更（只要 Security 記錄仍保留該事件）。攻擊者可以關閉稽核，之後再重新開啟。
- **子類別與類別。** 在 Windows Vista 及之後的版本中，Windows 使用的是子類別（除非
  `SCENoApplyLegacyAuditPolicy` 安全性選項已關閉，且由舊版類別原則覆寫它們）。
- **Windows XP 與 2003：**當稽核關閉時（`AuditingEnabled` 為 False），無論類別的設定為何，
  都不會稽核任何類別。
- **事件：**`Events` 列出每個子類別的主要事件，取自 Microsoft 關於進階稽核原則設定的文件。
  這些是調查時通常會尋找的事件，而非該子類別所記錄的所有事件。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.Setting == "none") | .Subcategory'
```

對於每個登錄區，摘要會提供其配置、已稽核的設定數量，以及原則最後寫入的時間。它會依序列出
同時稽核成功與失敗、僅稽核成功、僅稽核失敗的項目，接著列出未稽核的子類別及其主要事件：
這些事件在擷取到的原則中是停用的；但這並不能確立較早活動期間的原則狀態。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`、`SourcePath` | 主機（取自系統的 `SYSTEM` 登錄區），以及 `SECURITY` 登錄區。 |
| `Layout` | `subcategories`（Windows Vista 及之後版本）或 `categories`（XP 與 2003）；值無法解碼時為空。 |
| `Category`、`Subcategory` | 依 `auditpol` 的命名。 |
| `SubcategoryGUID`、`SubcategoryID` | 子類別的 GUID，以及事件 4719 所提供的訊息 ID（`%%12548`）。 |
| `Setting` | `none`、`success`、`failure` 或 `success_and_failure`。 |
| `Success`、`Failure` | 是否記錄成功與失敗。 |
| `AuditingEnabled` | Windows XP 與 2003：稽核是否有開啟。 |
| `Events` | 該子類別的主要事件 ID。 |
| `KeyLastWrittenUTC` | `PolAdtEv` 的最後寫入時間：大約是原則最後一次變更的時間。 |
| `DataRaw` | 無法完整解碼時，該值的位元組（十六進位）。 |
| `TransactionLogStatus`、`LogsApplied` | 登錄復原脈絡。 |
| `Problems` | 未知的配置或大小、加總不符的數量，以及大於 3 的值。 |

## 調查策略 { #investigation-strategy }

1. 在斷定某件事沒有發生之前，請先確認其子類別是否有被稽核：
   Process Creation（4688）、Logon（4624、4625）、Special Logon（4672）、Other Object Access Events
   （排程工作，4698）、Security System Extension（服務安裝，4697）、User Account and
   Security Group Management（4720、4732）、File Share（5140）、Kerberos（4768、4769）。
2. 將原則與 Security 記錄比較：若出現目前未稽核之子類別的事件，代表原則曾經變更。
   請尋找事件 4719 以及近期的 `KeyLastWrittenUTC`。
3. 與群組原則的 `audit.csv` 比較，以區分整個網域的原則與本機變更。
4. 使用 `parse-registry read --recover-deleted` 與交易記錄檔來取得較早版本的
   `PolAdtEv`。

## 延伸資訊 { #further-details }

- [Microsoft：進階安全性稽核原則設定](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft：稽核常數（子類別 GUID）](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft：4719(S) System audit policy was changed](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
