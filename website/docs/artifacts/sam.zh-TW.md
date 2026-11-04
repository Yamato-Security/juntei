# SAM 本機帳戶 { #sam-local-accounts }

## SAM 包含的內容 { #what-the-sam-contains }

Security Account Manager（`SAM`）登錄區儲存本機帳戶記錄。Juntei 會解碼帳戶名稱、RID、
設定檔／註解欄位、帳戶控制旗標、最後登入／密碼時間戳記、計數器、密碼提示、Microsoft 帳戶連結、
設定檔路徑，以及 Windows 是否在登入介面中隱藏該帳戶。

這有助於找出新建立的本機系統管理員、被啟用的休眠帳戶、密碼變更、隱藏帳戶，以及為橫向移動預先準備的帳戶。

<span class="evidence-strength">最佳結論：</span>本機 SAM 以所記錄的狀態呈現該帳戶。
它並非完整的驗證歷程，也不描述網域帳戶。

## 位置與收集 { #location-and-collection }

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

請以足夠的權限／原始存取收集登錄區與記錄檔。主機身分資訊可透過收集資料中對應的 `SYSTEM` 登錄區加以補充。

## 分析注意事項 { #analyst-cautions }

- 此命令剖析的是**本機**帳戶，而非 Active Directory 網域使用者。
- Juntei 刻意不擷取密碼雜湊。
- RID 500 即使被重新命名，仍是內建的 Administrator 帳戶；僅憑名稱並不足夠。
- `LastLogon` 與計數器是登錄檔狀態，而非完整的逐工作階段稽核軌跡。
- `HiddenFromLogonUI` 會將帳戶從登入畫面隱藏；它並不會停用該帳戶。
- 內建的 `DefaultAccount` 與 `WDAGUtilityAccount` 可能是合理地被隱藏。
- 因 `F` 值損毀／缺漏而得到的 null 帳戶旗標，不可解讀為已啟用。
- `NamesKeyLastWrittenUTC` 可近似帳戶名稱的註冊脈絡，但並不保證是建立時間。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.HiddenFromLogonUi == true or (.AccountFlags | index("disabled") | not))'
```

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`, `SourcePath` | 主機脈絡與確切的 SAM 登錄區。 |
| `RID` | 相對識別碼。強而有力的帳戶身分交叉比對依據；500 為內建的 Administrator。 |
| `User` | 目前的本機帳戶名稱。可能被重新命名；請依 RID 交叉比對。 |
| `FullName`, `Comment`, `UserComment` | 描述性文字；可由攻擊者控制，有助於檢查偽裝行為。 |
| `AccountFlags` | 解碼後的狀態，例如已停用、已鎖定、不需要密碼、一般帳戶、密碼永不過期。 |
| `AccountFlagsValue` | 原始數值旗標，供驗證與未支援的位元使用。 |
| `CountryCode`, `CodePage` | 帳戶的地區設定／字碼頁中繼資料；通常屬於脈絡資訊，而非歸屬依據。 |
| `LastLogonUTC`, `LastLogonWholeSecondsUTC` | 相同的最後登入狀態，分別以完整精度與相容用的整秒表示。並非完整歷程。 |
| `PasswordLastSetUTC` | 所記錄的密碼變更時間。請調查非預期的近期／過舊值。 |
| `AccountExpirationUTC` | 有限期時所設定的到期時間。null 可能代表永不到期／未設定。 |
| `LastFailedLogonUTC` | SAM 狀態中最後一次記錄的本機登入失敗；請以 Security 事件佐證。 |
| `FailedLogonCount`, `LogonCount` | 累計／狀態計數器，其更新行為依 Windows 而定；請作為脈絡使用，而非精確的事件數量。 |
| `PasswordHint` | 使用者提供的敏感提示；可能暴露密碼模式。請妥善處理。 |
| `InternetUserName` | 若存在，為所連結的 Microsoft 帳戶身分。 |
| `HiddenFromLogonUI` | 介面隱藏狀態。請檢查高 RID／非預期的自訂帳戶；並不代表已停用。 |
| `HomeDirectory`, `HomeDrive`, `ProfilePath`, `LogonScript` | 本機帳戶的環境／登入設定；不尋常的指令碼／UNC 路徑可能表示持續存取。 |
| `KeyLastWrittenUTC` | RID 帳戶機碼的變更時間。屬於機碼層級，並非特定欄位。 |
| `NamesKeyLastWrittenUTC` | 使用者名稱對應機碼的變更時間；可用來近似帳戶的生命週期。 |
| `LastLogonFiletime`, `PasswordLastSetFiletime`, `AccountExpirationFiletime`, `LastFailedLogonFiletime` | 為驗證與特殊哨兵值處理而保留的原始 64 位元 FILETIME 值。 |
| `TransactionLogStatus`, `LogsApplied` | 登錄檔重播的脈絡。 |
| `Problems` | F/V/Names 之間不一致、項目缺漏、位移格式錯誤或解碼損毀。 |

## 調查策略 { #investigation-strategy }

1. 將 RID、名稱、旗標、設定檔路徑與 `NamesKeyLastWrittenUTC` 與主機基準線比較。
2. 檢查已啟用的 RID 1000 以上帳戶、不需要密碼、永不過期、隱藏或近期變更的帳戶。
3. 與 Security 4720/4722/4724/4732 事件、登入事件、設定檔建立以及遠端活動交叉比對。
4. 若帳戶在收集前已被移除，請搜尋登錄檔復原歷程。
5. 區分合法的佈建／預設帳戶與攻擊者以互動方式建立的帳戶。

## 延伸資訊 { #further-details }

- [Microsoft Open Specifications：SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [dissect.target SAM 外掛程式](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
- [Eric Zimmerman 的 Registry Explorer](https://ericzimmerman.github.io/#!index.md)
