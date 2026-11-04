# Microsoft Office（最近使用的檔案、信任的文件、巨集設定） { #microsoft-office-recent-files-trusted-documents-macro-settings }

## Office 記錄的內容 { #what-office-records }

Microsoft Office 會依使用者在 `NTUSER.DAT` 中，並依版本與應用程式分別保存：

- **最近使用的檔案與位置：** `File MRU` 是應用程式開啟過的文件，`Place MRU`
  則是開啟這些文件時所在的資料夾，每個項目都附有最後一次開啟的時間。Microsoft 365
  也會為每個已登入的帳戶保存一份清單（`User MRU\<account>`）。
- **信任的文件：** 當使用者在文件上按下 *Enable Editing*（啟用編輯）或 *Enable Content*（啟用內容）時，
  Office 會將其記錄在 `TrustRecords` 中：包括文件的路徑、被信任的時間，以及其
  巨集是否被啟用。
- **巨集設定：** `VBAWarnings`（哪些巨集會執行）、`BlockContentExecutionFromInternet`
  （是否封鎖來自網際網路之檔案中的巨集）、`AccessVBOM`（程式是否可以
  變更 VBA 專案）以及**信任位置**，即其中文件的巨集無需詢問即可執行的
  資料夾。群組原則可在 `Software\Policies` 之下設定相同的項目。

惡意文件通常需要使用者啟用其巨集：一筆巨集已被啟用的信任記錄，
會將該文件、使用者與時間連結在一起。

<span class="evidence-strength">最佳結論：</span>使用者曾在此應用程式中開啟此文件
（最後一次在此時間）；使用者在此時間啟用了此文件的編輯或巨集；
Office 被設定為以此方式執行巨集。

## 位置 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

版本：`16.0` 為 Office 2016 及更新版本（2019、2021、2024、Microsoft 365），`15.0` 為 Office 2013，
`14.0` 為 Office 2010，`12.0` 為 Office 2007。

項目的形式如 `[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx`：`T`
是最後開啟時間（十六進位的 FILETIME），路徑則接在 `*` 之後。信任記錄為 24
位元組：先是文件被信任時的 FILETIME，接著最後 4 個位元組為其旗標；巨集已被啟用時為 `0x7FFFFFFF`，
僅啟用編輯時則為 `0x00000001`。

## 分析注意事項 { #analyst-cautions }

- **時間。** `LastOpenedUTC` 是每個項目各自的時間。`TrustedAtUTC` 是使用者信任
  該文件的時間，也就是按下 *Enable Editing* 或 *Enable Content* 的那一刻，可能早於
  最近一次的開啟。
- **巨集已啟用，不等於已執行。** `MacrosEnabled` 表示使用者啟用了該文件的巨集；它
  並不能證明巨集曾經執行或其做了什麼。請與程序建立、Prefetch、PowerShell
  記錄以及文件本身交叉比對。
- **路徑依 Office 儲存的形式呈現。** 信任記錄以路徑來指稱文件，這些路徑可能使用
  `%USERPROFILE%`、正斜線、`file:///` URL 以及 URL 編碼（`%20`）。雲端文件的 MRU 項目
  則是 URL。
- **預設值與實際生效的原則。** Office 會建立預設的信任位置（`Templates`、
  `XLSTART`、`STARTUP`、…）。位置的 `Date` 是有用的線索，但不能證明是誰建立了它：
  系統管理員、指令碼、原則以及複製來的登錄檔設定也都可能產生它。單憑缺少
  `VBAWarnings` 並不能確立實際生效的巨集原則。Office 版本、網際網路
  檔案標記以及其他管理設定也有影響。
  [Microsoft 說明網際網路巨集封鎖](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked)。
- **已刪除的文件。** 清單的存在時間比文件更長；已不存在的檔案仍可能顯示於
  此處。清單很短，且會被 Office 的 *Clear unpinned items*（清除未釘選的項目）與清理工具清除。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

摘要會列出巨集已被啟用的文件、讓巨集能更自由執行的設定
（`VBAWarnings` 1、`BlockContentExecutionFromInternet` 0、`AccessVBOM` 1、
`AllowNetworkLocations` 1）、使用者新增或由原則設定的信任位置，以及最後開啟的
文件。

## 輸出欄位 { #output-fields }

| 欄位 | 判讀 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | 主機、設定檔擁有者（取自路徑或殼層資料夾）、取自系統 `ProfileList` 的 SID，以及登錄區。 |
| `OfficeVersion`, `App` | 版本機碼（`16.0`）與應用程式（`Word`、`Excel`、…）。 |
| `List` | `File MRU`、`Place MRU`、`TrustRecords`、`Security` 或 `Trusted Locations`。 |
| `Account` | `User MRU` 清單所屬的 Microsoft 365 帳戶。 |
| `Policy` | 該設定或位置是否由原則所設定。 |
| `MRU-Position`, `IsMostRecent` | 項目在清單中的位置（`Item 1` 為 0，即最近使用）。 |
| `Path` | 文件或資料夾；信任文件依儲存形式表示的路徑；信任位置的 `Path`。 |
| `LastOpenedUTC` | 項目最後一次被開啟的時間。 |
| `TrustedAtUTC`, `MacrosEnabled`, `TrustFlags` | 文件被信任的時間、其巨集是否被啟用，以及記錄的旗標。未知的旗標會使 `MacrosEnabled` 保留空白，並產生診斷訊息。 |
| `Setting`, `Value` | 巨集設定及其值與意義（`1 (all macros enabled)`）；信任位置的機碼（`Location3`）及其其他值（`AllowSubFolders=1; Date=…; Description=…`）。 |
| `KeyPath`, `KeyLastWrittenUTC` | 機碼及其最後寫入時間。 |
| `DataRaw` | 無法解碼時，該值的位元組（十六進位）。 |
| `TransactionLogStatus`, `LogsApplied` | 登錄檔復原的脈絡。 |
| `Problems` | 沒有 `*` 或時間無效的項目、不是 24 位元組的信任記錄、沒有 `Path` 的位置、錯誤的類型。 |

## 調查策略 { #investigation-strategy }

1. 從 `MacrosEnabled` 為 true 的項目著手：哪些文件、何時、由誰。找出這些檔案（`$MFT`、
   USN Journal、`Zone.Identifier`）、其來源（瀏覽器與郵件歷程）以及接下來執行了什麼。
2. 檢查巨集設定與信任位置：啟用所有巨集、未封鎖網際網路巨集、
   VBA 專案存取，或使用者新增的位置（位於 `Downloads`、`Temp`、網路共用）
   都會削弱預設的防護。
3. 將 `LastOpenedUTC` 與 LNK 檔案、Jump Lists 以及 RecentDocs 一同放入時間軸，以掌握
   文件的路徑，包括卸除式與網路位置。

## 延伸資訊 { #further-details }

- [Microsoft：在 Microsoft 365 檔案中啟用或停用巨集](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6)
- [Microsoft：信任的文件](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft：新增、移除或變更信任位置](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
