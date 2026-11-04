# Microsoft Defender 隔離資料 { #microsoft-defender-quarantine }

## 隔離資料包含哪些內容 { #what-the-quarantine-contains }

Microsoft Defender 的隔離區會保存偵測相關的中繼資料，以及其所隔離資源的加密內容。
Juntei 會將 Entries、ResourceData 與 Resources 記錄彼此關聯，以回報威脅標籤、隔離時間、
原始邏輯路徑與實體路徑、檔案中繼資料、擁有者、表示下載來源的 `Zone.Identifier`、雜湊值，
以及完整性問題。

此跡證可能比原本記錄偵測／修復行為的事件記錄保存得更久。它也可能保存
可疑內容的確切位元組，供在隔離環境中進行逆向工程。

<span class="evidence-strength">最佳結論：</span>Defender 曾在所記錄的時間，針對所列的
威脅／資源建立隔離記錄。產品標籤並非惡意軟體的獨立證明，
而單憑隔離也不能證明該檔案先前曾被執行。

## 位置與收集 { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Quarantine\
  Entries\{GUID}
  ResourceData\<prefix>\<SHA-1>
  Resources\<prefix>\<SHA-1>
```

請收集整個 `Quarantine` 目錄。缺少相對應內容／記錄檔案的項目仍是
有用的中繼資料，但無法取得檔案位元組或計算出雜湊值。

## 安全性與分析注意事項 { #safety-and-analyst-cautions }

- 擷取出的內容很可能是惡意的。請使用隔離的分析系統，且絕對不要執行它。
- `ThreatName` 是 Defender 當時的特徵碼／分類。可能有誤判，之後也可能
  被重新分類。
- 隔離時間是修復／隔離的時間，不一定是下載、建立或執行的時間。
- 被隔離的檔案可能是在執行前被偵測，也可能是在執行後才被偵測。
- 原始路徑與 Zone.Identifier 很有價值，但可能不存在，或受攻擊者控制。
- 孤立記錄可能源自保留期限／清理作業、收集不完整或資料損毀。
- Juntei 會驗證大小、CRC、ID 與雜湊值；在信任復原出的位元組之前，請先檢視 `Problems`。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find quarantine folders
  -f, --file <FILE>      Parse one Entries\{GUID} file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
      --extract <DIR>       Extract dangerous content with a .quarantined extension
  -C, --clobber             Overwrite confirmed output targets
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -t jsonl \
  | jq '{QuarantineTime, ThreatName, Path, SHA256: .Content.SHA256, Problems}'

# Only in an isolated malware-analysis environment
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract extracted/
```

例如，`tool.exe` 會變成 `tool.exe.quarantined`。輔助資料流的名稱中會包含其類型，
例如 `tool.exe.Zone.Identifier.quarantined`。Juntei 也會移除執行權限。
副檔名與權限設定可降低意外執行的機會，但復原出的位元組仍必須
視為惡意軟體處理。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Record` | `resource`、沒有資源的項目、`orphan-content` 或 `orphan-record`。孤立記錄代表關聯／收集不完整。 |
| `QuarantineTime` | Defender 的隔離／修復時間（UTC），並非執行時間。 |
| `ThreatName`、`ThreatID` | Defender 的分類與數值 ID。請視為廠商的判定／脈絡。 |
| `ResourceType` | 檔案、登錄機碼、服務、啟動項目、排程工作或其他資源類別。 |
| `Path`、`PhysicalPath` | 原始的邏輯／實體位置。主要的交叉比對樞紐；請保留確切的拼寫。 |
| `FileSize` | 資源中繼資料所記錄的大小。請與 `ContentSize` 及 MFT 比較。 |
| `CreationTime`、`LastAccessTime`、`LastWriteTime` | 為被隔離資源擷取的中繼資料；與隔離時間不同，且可能受到時間戳記竄改（timestomping）影響。 |
| `Attr` | 所記錄的檔案屬性。隱藏／系統狀態可提供有用的脈絡。 |
| `Owner` | 取自復原之安全性描述元的 SID；擁有者不一定是執行的使用者。 |
| `ZoneID`、`ReferrerURL`、`HostURL` | 來自 `Zone.Identifier` 的下載區域與 URL 值。存在時是有力的來源線索；ADS 可能不存在或已被移除。 |
| `ContentSize` | 實際復原出的主要內容大小。請與所記錄的 `FileSize` 比較。 |
| `MD5`、`SHA1`、`SHA256` | 依據復原出的位元組計算。現代的內容識別請使用 SHA-256；為了互通性，請保留全部雜湊值。 |
| `Streams` | 復原出的資料流種類／名稱／大小；可能包含 ADS、安全性資料或登錄區。 |
| `ResourceID` | 連結資源中繼資料與內容記錄的識別碼。 |
| `EntryID` | 隔離項目的 GUID；也可連結至 Defender 歷程／事件證據。 |
| `ScanID` | 用於將資源／動作分組的 Defender 掃描識別碼。 |
| `QuarantineFolder` | 根來源資料夾／主機區分。 |
| `EntryFile`、`ContentFile`、`ResourceRecordFile` | 用來建立該列的確切來源檔案。對驗證至關重要。 |
| `ExtractedFile` | 由 `--extract` 寫出的主要隔離檔案；未擷取主要檔案時為空。輔助資料流會寫在其旁邊。`auto-parse -x` 會以相對於其輸出目錄的路徑提供（`./HOST1/SecurityProducts/DefenderQuarantined/…`）。 |
| `Problems` | 完整性不符、內容遺失、ID 不一致、損毀或孤立記錄的脈絡。請勿隱藏。 |

JSON／JSONL 會保留結構化的資源、資料流清單、記錄項目及所有關聯細節，
而不是 CSV 中合併後的文字。

## 擷取行為 { #extraction-behavior }

當隔離項目記錄了原始檔名時，Juntei 會使用該檔名並在最後加上 `.quarantined` 副檔名。
對於孤立資源或非檔案資源，則使用內容識別碼，並在檔名衝突時加上
識別碼以作區分。在 Windows 中無效的檔名字元會被取代。安全性資料流
與替代資料流會寫在主要檔案旁邊。每個安全性資料流也會另外產生一份易讀的
文字報告。Juntei 會移除執行權限；
若已存在內容不同的檔案，未指定 `-C` 時不會覆寫。這些控制措施可減少意外，但
並不會讓這些位元組變得安全。

### 讀取復原的安全性描述元 { #reading-the-recovered-security-descriptor }

`.security.quarantined` 檔案包含二進位的 Windows 安全性中繼資料，因此無法在
文字編輯器中閱讀。它是獨立的備份資料流，並不是執行檔的另一份複本。Juntei 在擷取時會自動
解碼它，無論使用 `parse-file defender-quarantine --extract` 或 `auto-parse -x` 皆然：

```text
tool.exe.quarantined              Original file content; treat as malware
tool.exe.security.quarantined     Original binary security descriptor, preserved unchanged
tool.exe.security_descriptor.txt  Readable ownership and permission report
```

文字檔名會跟隨實際的二進位檔名，包括任何衝突識別碼。產生的
文字是一般的 UTF-8，並採用與其他擷取檔案相同的覆寫保護。其中包含
二進位檔名、位元組數與 SHA-256，以便將報告與其來源對應。

| 報告欄位 | 如何判讀 |
|---|---|
| 擁有者與主要群組 | 描述元中所記錄的 SID。知名 SID 會顯示名稱，例如 `BUILTIN\Administrators` 或 `NT AUTHORITY\SYSTEM`。其他 SID 則維持未解析；單憑熟悉的 RID 不足以斷定本機／網域帳戶的名稱。擁有者並不能證明是誰建立或執行了檔案。 |
| 控制旗標 | 原始數值旗標及其名稱，包括 ACL 是否存在、繼承與保護。 |
| DACL | 依序排列的存取控制項目（ACE）：允許／拒絕類型、帳戶 SID、權限遮罩、權限與繼承旗標。這些是所記錄的項目，並非針對某個使用者權杖計算出的有效存取權。 |
| SACL | 所記錄的稽核或強制完整性項目。稽核成功／失敗旗標描述的是設定，而非存取歷程。SACL 不存在並不代表所有稽核都已停用。 |
| 物件 GUID | 物件專用 ACE 的選用 GUID；物件專用權限會以數值保留，而不會被誤標為檔案權限。 |
| SDDL | 擁有者、群組與所支援之 ACL 項目的安全性描述元定義語言（Security Descriptor Definition Language）表示法。控制旗標請另外查看。若任何部分損毀或不受支援，Juntei 不會輸出不完整的 SDDL 字串。 |
| 解碼狀態與問題 | `complete` 或 `incomplete`，並附上原因。不受支援的 ACE 會在報告中保留其原始十六進位資料。擷取成功時一律保留原始二進位檔。解碼問題也會出現在隔離記錄的 `Problems` 欄位中。 |

不存在的 DACL、NULL DACL 與空的 DACL 會分別回報。不存在或 NULL 的 DACL 不提供
任何任意存取限制；空的 DACL 則沒有任何授予存取權的 ACE。其他 Windows 存取
檢查與特殊權限仍然有影響。繼承而來的 ACE 與僅供繼承的 ACE 會維持明確標示。

例如，一筆授予 Administrators 繼承而來之完全控制權的項目可能顯示為：

```text
Entry 1: ALLOW (type 0x00)
  Flags: 0x10 (INHERITED_ACE)
  Account: S-1-5-32-544 (BUILTIN\Administrators)
  Access mask: 0x001F01FF
  Rights: Full control (FILE_ALL_ACCESS)
```

此解碼器可在 Windows、macOS 與 Linux 上離線執行，不需要 Windows API、Python 或帳戶查詢。
它會解碼基本的允許／拒絕／稽核／警報 ACE、物件 ACE 與強制完整性標籤。條件式、
回呼、資源屬性及其他不受支援的 ACE 會被明確識別，並以原始形式
保留；其存取條件不會被評估。損毀的描述元不會中斷其他資料流的
擷取。

## 調查策略 { #investigation-strategy }

1. 從 `ThreatName`、`Path`、`QuarantineTime`、`EntryId` 與雜湊值著手。
2. 驗證 `Problems`，並比較所記錄的大小與復原出的內容。
3. 以 SHA-256 為樞紐，查詢其他主機、EDR、沙箱與威脅情資資料。
4. 將原始路徑／時間與 `$MFT`、USN 日誌、Prefetch、Amcache 及事件記錄交叉比對。
5. 使用 URL／區域欄位調查初始存取，但請以瀏覽器／Proxy／DNS 證據加以驗證。

## 延伸資訊 { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark 的 defender](https://github.com/Sootmark/defender)
- [dissect.target Defender 外掛程式](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [Microsoft：Microsoft Defender 防毒軟體的修復與隔離](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [defender-dump](https://github.com/knez/defender-dump)
- [Microsoft：安全性描述元](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft：安全性描述元字串格式](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft：ACE 字串](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings)
