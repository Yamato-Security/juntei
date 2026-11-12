# Outlook { #outlook }

## 什麼是 Outlook 的檔案 { #what-outlooks-files-are }

Windows 版 Outlook 將郵件、連絡人、行事曆與工作保存在其專屬的檔案中：

- **OST**（`.ost`）：信箱的離線複本，Outlook 會為 Exchange 與 Microsoft 365 帳戶保留這個檔案，自
  Outlook 2013 起也會為 IMAP 帳戶保留。Outlook 2013 及更新版本的 OST 使用 4 KiB 的頁面與壓縮的區塊。
- **PST**（`.pst`）：個人存放區：POP3 帳戶、封存（`archive.pst`）、匯出的檔案，以及使用者自行建立的
  存放區。ANSI PST 屬於 Outlook 97 至 2002；Unicode PST 則屬於 Outlook 2003 及更新版本。
- **MSG**（`.msg`）：儲存為檔案的單一項目，於項目被拖曳出 Outlook、以**另存新檔**儲存，或由其他
  程式匯出時產生。附加的郵件也是 MSG 檔案。
- **OFT**（`.oft`）：Outlook 範本，也就是用來建立新項目的 MSG 檔案。

每個存放區保存資料夾，每個資料夾保存其項目：電子郵件、會議邀請與回覆、連絡人與通訊群組清單、
行事曆項目、工作、記事、日誌項目與報告。每個資料夾也保存隱藏的項目，即其資料夾關聯資訊（FAI）：
檢視、規則、表單與 Outlook 的設定（`IPM.Configuration.*` 項目，例如 Exchange 信箱的自動完成清單）。
項目的附件可以是檔案、檔案的參照、OLE 物件或其他項目（以附件形式轉寄的郵件、週期性約會的例外），
而這些項目也可以有自己的附件。

<span class="evidence-strength">最佳結論：</span>收集當時，該存放區或檔案保存了該項目，以及其所記錄的
寄件者、收件者、時間與附件；這並不表示使用者曾閱讀、撰寫或寄出該項目，也不表示其寄件者確實是
項目所顯示的人。

## 位置與收集 { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

PST 可能位於任何地方：使用者會將封存檔移到其他資料夾、磁碟機與檔案共用。Outlook 會將每個設定檔中
各存放區的路徑保存在使用者的登錄檔中。使用者從 Outlook 開啟的附件，會被複製到
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\` 下的某個資料夾（Windows 8.1 之前為
`Temporary Internet Files\Content.Outlook`），並可能一直留在那裡。

Outlook 執行期間會持續開啟其存放區，而一個 OST 可能達數十 GB。請在 Outlook 關閉時收集，或從磁碟
映像或磁碟區陰影複製中收集。Juntei 以串流方式讀取每個存放區，從不將整個存放區載入記憶體。

## 分析注意事項 { #analyst-cautions }

- Juntei 以寬鬆的方式讀取：它會越過損毀之處，讀取可讀取的部分，並將其餘部分回報為問題。這些問題會
  顯示在畫面上（每個檔案的前 10 個），計入摘要與 Files 資料表，並列在相關項目與附件的 `Problems`
  欄位中。完全無法讀取的項目，在 Items 資料表中仍有一筆資料列，帶有其 `ItemID` 與原因。
- 時間皆為 UTC，精確到 100 奈秒。`SubmitTime` 是寄件者的用戶端送出郵件的時間，依據的是該電腦的
  時鐘；`DeliveryTime` 是郵件抵達此信箱的時間。`CreationTime` 是項目的這份複本建立的時間：傳遞到
  信箱或在信箱中建立、同步到 OST、複製或匯入 PST，或儲存為 MSG 檔案的時間；它並不是郵件撰寫的
  時間。`LastModifiedTime` 是項目在存放區中最後一次變更的時間。
- 工作的 `StartTime` 與 `DueTime` 是使用者時區中的日期，Outlook 將其儲存為午夜。Juntei 寫入時
  不附時差（`2017-02-19T00:00:00`），`-Z` 也會保持原樣。
- 在某些日期中，例如工作的到期日或提醒的時間，Outlook 會寫入 4501 年 1 月 1 日午夜來表示**無**。
  Juntei 依原樣寫入：`4501-01-01T00:00:00Z`；在工作的 `StartTime` 與 `DueTime` 中則不帶 `Z`。
- `Read` 是收集存放區時的已讀旗標：表示有人或某條規則將該項目標示為已讀，但並未說明是誰或何時。
- 寄件者與收件者的名稱和地址，是寄件程式與郵件伺服器所記錄的內容，可能遭到偽造。`EX` 地址是
  Exchange 的辨別名稱（`/o=…/ou=…/cn=Recipients/cn=…`）；若項目另外儲存了 SMTP 地址，該地址會
  位於 `SenderSMTP-Address` 與 `SMTP-Address` 中。若要確認郵件的來源，請查看其網際網路標頭：
  Properties 資料表中的 `0x007D001F` 或 `0x007D001E`（`PidTagTransportMessageHeaders`）。
- 在 Outlook 中刪除的項目，在清空之前會位於**刪除的郵件**（Deleted Items）中，並與其他項目一樣
  被讀取。已不在任何資料夾資料表中的項目，存放區可能會保留到重新使用其空間為止，但目前尚不會讀取。
- `ItemID` 是郵件在其 PST 或 OST 中的節點 ID：同一個項目在存放區的另一份複本中可能有不同的 ID。
  內嵌項目的 ID 是其父項目的 ID 加上附件的索引（`2097348/0`、`0/0/1`）；`.msg` 或 `.oft` 檔案的
  項目為 `0`。
- `User` 僅取自路徑（`Users` 之後的資料夾），檔案被複製到他處後便會遺失。PST 可能屬於任何使用者。
- 非 Unicode 的文字會以項目的字碼頁讀取（項目本身的、其地區設定的，或其網際網路字碼頁的字碼頁）。
  `--code-page` 指定未提供字碼頁之項目的字碼頁：也就是寫入這些項目之系統的 ANSI 字碼頁，日文系統
  為 `932`。
- PST 的密碼並不會加密檔案：Juntei 讀取受密碼保護的 PST 與讀取其他 PST 無異。存放區的
  `Encryption`（`Permute`，即 Outlook 的「可壓縮加密」，或 `Cyclic`）只會混淆其區塊。以 S/MIME
  加密的郵件會將其內文與附件保存在一個加密的附件（`smime.p7m`）中，Juntei 不會將其解密。
- 附件經常是惡意程式。`--extract-attachments` 會將其資料寫入以 SHA-256 命名的檔案（Unix 上的新檔案使用不具執行權限的 `0600` 模式）；
  請勿執行這些檔案，並將其保存在隔離的分析系統上。

### 尚未讀取的內容 { #not-read-yet }

離線通訊錄（OAB）、自動完成資料流（`Stream_Autocomplete_*.dat`）與 `.nk2` 檔案、傳送/接收設定
（`.srs`）、Outlook 的設定快取（RoamCache、Autodiscover）、Outlook 的 SQLite 資料庫，以及已不在
任何資料夾資料表中的郵件。

## 使用 Juntei 剖析 { #parse-with-juntei }

```text
Usage: juntei parse-file outlook <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pst, *.ost, *.msg and *.oft
  -f, --file <FILE>      Parse one PST, OST, MSG or OFT file, whatever its name
      --code-page <CP>   Code page of items that don't give theirs (default 1252)

Output:
  -o, --output <PATH>              Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>         csv, json, or jsonl
      --properties                 Add the Properties table: every property as stored
      --extract-attachments <DIR>  Write each attachment's data once, named by its SHA-256
  -C, --clobber                    Overwrite files when saving or extracting
```

```bash
juntei parse-file outlook -d Users -o outlook-csv
juntei parse-file outlook -f mailbox.ost -t jsonl \
  | jq 'select(.Table == "Items" and .Hidden == false) | {DeliveryTime, SenderAddress, Subject}'
juntei parse-file outlook -f suspicious.msg -o msg-csv --properties --extract-attachments isolated/
```

`-d` 會尋找 `.pst`、`.ost`、`.msg` 與 `.oft` 名稱（不區分大小寫），再檢查標頭：PST 或 OST 為 `!BDN`，MSG 或 OFT 為複合檔案簽章。同樣使用這些副檔名的無關檔案（例如 Tcl 訊息目錄）會略過。無法讀取的檔案，以及簽章可辨識但已損壞的容器，仍會回報錯誤。`-f` 一律驗證指定檔案，不受名稱影響，無效輸入會回報錯誤。若沒有任何檔案能夠讀取，結束狀態為 1。

`--extract-attachments` 會將每個附件的資料只寫入一次，檔名為 `<SHA-256>`，若附件的副檔名由 1 至 10 個
ASCII 字母與數字組成，則附加該副檔名（`0c7d…1311.jpg`）；路徑中不會使用任何來自項目的名稱。除非
指定 `-C`，否則該目錄必須是空的；指定 `-C` 時，名稱相同但內容不同的檔案會被取代。

## 輸出資料表 { #output-tables }

CSV 會建立八個檔案，使用 `--properties` 時為九個；沒有資料列的檔案只含標題列。JSON 與 JSONL
資料列會在 `SourceFile` 之後帶有 `Table`（`Files`、`Folders`、`Items`、`Recipients`、`Attachments`、
`Contacts`、`Calendar`、`Tasks` 或 `Properties`）。項目的資料列之後接著屬於該項目的資料列，再接著
其內嵌項目的資料列；檔案的 Files 資料列則位於該檔案所有其他資料列之後。

| CSV 檔案 | 內容 |
|---|---|
| `Outlook-Files.csv` | 每個讀取的檔案一筆資料列：存放區或檔案，以及其中保存的內容。 |
| `Outlook-Folders.csv` | PST 與 OST 的資料夾。 |
| `Outlook-Items.csv` | 每一個項目：資料夾中的郵件、資料夾的隱藏項目、`.msg` 與 `.oft` 檔案的項目，以及內嵌於其附件中的項目。 |
| `Outlook-Recipients.csv` | 項目的收件者。 |
| `Outlook-Attachments.csv` | 項目的附件。 |
| `Outlook-Contacts.csv` | 連絡人與通訊群組清單的欄位。 |
| `Outlook-Calendar.csv` | 行事曆項目與會議郵件的欄位。 |
| `Outlook-Tasks.csv` | 工作與工作要求的欄位。 |
| `Outlook-Properties.csv` | 使用 `--properties` 時：存放區、資料夾、項目、收件者與附件的每一個屬性，依原樣儲存。 |

## 欄位 { #fields }

每個資料表都以下列欄位開頭：

| 欄位 | 判讀 |
|---|---|
| `SourceFile` | 讀取該資料列的檔案。 |
| `User` | 路徑所在設定檔的 Windows 使用者（`Users` 之後的資料夾）。 |

### 檔案 { #files }

| 欄位 | 判讀 |
|---|---|
| `FileType` | `PST`、`OST`、`MSG`、`OFT`，或 `CFB`（以項目方式讀取、但其類別 ID 既非郵件亦非範本的複合檔案；許多 Outlook 以外的程式會將其留空）。 |
| `Format` | 存放區的配置：`ANSI`（Outlook 97 至 2002）、`Unicode`（Outlook 2003 及更新版本）或 `Unicode 4K`（Outlook 2013 及更新版本的 OST）；MSG 或 OFT 檔案則為其複合檔案版本（`CFB v3`、`CFB v4`）。 |
| `Version`、`ClientVersion` | 存放區標頭的 `wVer` 與 `wVerClient`。 |
| `Encryption` | 存放區區塊的編碼方式：`None`、`Permute` 或 `Cyclic`。 |
| `DisplayName` | 存放區的名稱（`Personal Folders`、某個電子郵件地址）。 |
| `RecordKey` | 存放區的識別碼（十六進位），其項目的 entry ID 中都含有此值。 |
| `Folders` | 存放區的資料夾數。 |
| `Items` | 其資料夾中的郵件數；MSG 或 OFT 檔案為 1。 |
| `HiddenItems` | 其資料夾的隱藏項目（資料夾關聯資訊）數。 |
| `EmbeddedItems` | 內嵌於附件中的項目數，涵蓋所有層級。 |
| `Attachments` | 上述所有項目的附件數。 |
| `Problems` | 在該檔案中發現的問題數。 |

### 資料夾 { #folders }

| 欄位 | 判讀 |
|---|---|
| `FolderID`、`ParentID` | 資料夾的節點 ID，以及其父資料夾的節點 ID（根資料夾為空白）。 |
| `FolderPath` | 從根目錄往下的各資料夾名稱：`/Top of Personal Folders/Inbox`、`/Root - Mailbox/IPM_SUBTREE/Sent Items`。 |
| `Name` | 資料夾的名稱。 |
| `ContainerClass` | 資料夾保存的內容：`IPF.Note`（郵件）、`IPF.Appointment`、`IPF.Contact`、`IPF.Task`、`IPF.Imap` 等。 |
| `ContentCount`、`UnreadCount` | 資料夾自身所記載的郵件數與未讀郵件數。 |
| `Messages`、`HiddenItems`、`Subfolders` | 其內容資料表、關聯內容資料表與階層資料表的資料列數：也就是 Juntei 所讀取的內容。若與 `ContentCount` 有差異，值得深入查看。 |
| `CreationTime`、`LastModifiedTime` | 資料夾的建立時間與最後一次變更的時間（UTC），僅在存放區保有這些時間時提供。 |
| `Problem` | 資料夾無法開啟的原因；此時其子資料夾不會被讀取。 |

### 項目 { #items }

| 欄位 | 判讀 |
|---|---|
| `FolderPath` | 項目所在的資料夾；在 MSG 或 OFT 檔案中為空白。 |
| `ItemID` | 項目的 ID：在 PST 或 OST 中為其節點 ID，在 MSG 或 OFT 檔案中為 `0`，內嵌項目則為其父項目的 ID 加上附件的索引（`2097348/0`）。 |
| `Hidden` | 項目是否為資料夾關聯資訊，即隱藏的項目。 |
| `EmbeddedIn` | 該項目所附加之項目的 ID；其他項目為空白。 |
| `ItemType` | 依郵件類別判斷的項目種類：`Mail`、`MeetingMessage`、`Appointment`、`Contact`、`DistributionList`、`Task`、`TaskRequest`、`StickyNote`、`Journal`、`Post`、`Report`、`Configuration` 或 `Other`。 |
| `MessageClass` | 依原樣儲存的郵件類別（`IPM.Note`、`IPM.Note.SMIME`、`IPM.Schedule.Meeting.Request`、`REPORT.IPM.Note.NDR` 等）。 |
| `SubmitTime` | 寄件者的用戶端送出郵件的時間（UTC，依寄件者的時鐘）。 |
| `DeliveryTime` | 郵件傳遞到此信箱的時間（UTC）。 |
| `CreationTime`、`LastModifiedTime` | 這份複本在存放區或檔案中建立的時間，以及最後一次變更的時間（UTC）。 |
| `Subject` | 主旨，包含其前置詞（`RE: `）。 |
| `SenderName`、`SenderAddressType`、`SenderAddress` | 寄件者：名稱、地址類型（`SMTP`、`EX`），以及該類型的地址。 |
| `SenderSMTP-Address` | 寄件者的 SMTP 地址，僅在除了其他類型的地址之外另有儲存時提供。 |
| `SentRepresentingName`、`SentRepresentingAddress` | 代表誰寄出（代理人代替主管寄送；共用信箱），若有儲存則為其 SMTP 地址，否則為該類型的地址。 |
| `To`、`Cc`、`Bcc` | 項目所顯示的收件者名稱（其地址請參閱 Recipients）。`Bcc` 只存在於寄件者的複本中。 |
| `InternetMessageID`、`InReplyTo` | 郵件的 `Message-ID`，以及其所回覆之郵件的 ID。 |
| `ConversationTopic` | 不含前置詞的主旨，同一交談中的郵件共用此值。 |
| `Importance`、`Sensitivity` | 重要性為 `Low`、`Normal` 或 `High`；敏感度為 `Normal`、`Personal`、`Private` 或 `Confidential`。 |
| `Read` | 項目是否標示為已讀。 |
| `Flags` | 郵件旗標（`PidTagMessageFlags`，十六進位）：`0x1` 已讀、`0x2` 未修改、`0x4` 已提交、`0x8` 未寄出（草稿）、`0x10` 有附件、`0x20` 來自本人、`0x40` 隱藏、`0x80` 重新寄送、`0x400` 曾經已讀、`0x2000` 來自網際網路。 |
| `Size` | 項目在存放區中的大小，包含其附件。 |
| `Recipients`、`Attachments` | 收件者數與附件數。 |
| `Categories` | 項目的類別，以 `;` 分隔。 |
| `BodyPreview` | 純文字內文的開頭部分，連續空白會合併為一個，最多 256 個字元；若只有 HTML 或 RTF 內文則為空白。 |
| `Problems` | 項目中無法讀取的部分（最多 10 個，其後註明還有多少個）。 |

### 收件者 { #recipients }

| 欄位 | 判讀 |
|---|---|
| `ItemID` | 項目的 ID。 |
| `Index` | 收件者在項目收件者資料表中的資料列，從 0 起算。 |
| `RecipientType` | `To`、`Cc`、`Bcc`、`Originator`，或依原樣儲存的值（十六進位）。 |
| `Name`、`AddressType`、`Address` | 收件者的名稱、地址類型與地址。 |
| `SMTP-Address` | SMTP 地址，僅在除了其他類型的地址之外另有儲存時提供。 |

### 附件 { #attachments }

| 欄位 | 判讀 |
|---|---|
| `ItemID`、`Index` | 項目的 ID，以及附件在其附件資料表中的資料列，從 0 起算。 |
| `Method` | `ByValue`（檔案）、`ByReference`、`ByReferenceResolve`、`ByReferenceOnly`（檔案的路徑）、`EmbeddedMessage`（項目）、`OLE`（OLE 物件）、`ByWebReference`，或其數值。 |
| `Filename` | 附件的名稱：其長檔名、短檔名或顯示名稱。 |
| `Extension` | 依原樣儲存的副檔名（`.pdf`），若無則取自其名稱。 |
| `MimeType`、`ContentID` | 其 MIME 類型，以及 HTML 內文用來參照它的內容 ID。 |
| `Inline` | 附件是否顯示在內文中（例如 HTML 內文中的圖片），而非列於附件清單中。 |
| `Hidden` | Outlook 是否在附件清單中隱藏它。 |
| `Size` | 存放區所記載的大小，包含其屬性。 |
| `DataSize`、`SHA256` | 其資料的大小與 SHA-256：資料即檔案本身，或在 PST 或 OST 中 OLE 物件的複合檔案。內嵌項目與以參照方式附加的附件為空白。 |
| `Pathname` | 以參照方式附加之附件的路徑。 |
| `CreationTime`、`LastModifiedTime` | 附件的時間（UTC）：通常是其來源檔案的時間。 |
| `EmbeddedSubject` | 內嵌項目的主旨；該項目自身的資料列帶有 `ItemID` `<ItemID>/<Index>`。 |
| `ExtractedFile` | 使用 `--extract-attachments` 時，其資料寫入該目錄中的哪個檔案。 |
| `Problems` | 附件中無法讀取或無法寫入的部分。 |

### 連絡人 { #contacts }

| 欄位 | 判讀 |
|---|---|
| `FolderPath`、`ItemID`、`ItemType` | 與 Items 資料表相同：`Contact` 或 `DistributionList`。 |
| `DisplayName`、`GivenName`、`Surname`、`Nickname` | 連絡人的名稱。 |
| `FileUnder` | 連絡人的歸檔名稱。 |
| `Company`、`Title`、`Department` | 連絡人的工作單位：公司、職稱與部門。 |
| `Email1Name`、`Email1Address`、`Email2Name`、`Email2Address`、`Email3Name`、`Email3Address` | 連絡人的三個電子郵件地址，各附其顯示名稱。 |
| `BusinessPhone`、`HomePhone`、`MobilePhone` | 連絡人的電話號碼。 |
| `InstantMessaging` | 連絡人的即時訊息地址。 |
| `CreationTime`、`LastModifiedTime` | 與 Items 資料表相同。 |

### 行事曆 { #calendar }

| 欄位 | 判讀 |
|---|---|
| `FolderPath`、`ItemID`、`ItemType`、`Subject` | 與 Items 資料表相同：`Appointment` 或 `MeetingMessage`。 |
| `StartTime`、`EndTime` | 約會的開始與結束時間（UTC）；週期性約會則為第一次發生的時間。 |
| `DurationMinutes` | 其長度，以分鐘計。 |
| `Location` | 地點。 |
| `AllDay`、`Recurring` | 是否為全天活動，以及是否為週期性（其例外為內嵌項目）。 |
| `BusyStatus` | `Free`、`Tentative`、`Busy`、`OutOfOffice` 或 `WorkingElsewhere`。 |
| `ResponseStatus` | 使用者的回應：`None`、`Organized`、`Tentative`、`Accepted`、`Declined` 或 `NotResponded`。 |
| `Organizer` | 項目寄件者的名稱。 |
| `AllAttendees`、`ToAttendees`、`CcAttendees` | 項目所列出的與會者。 |
| `GlobalObjectID` | 會議跨信箱的識別碼（十六進位）：召集人與與會者行事曆中的同一場會議共用此值。 |
| `ReminderSet`、`ReminderTime` | 是否設定提醒，以及提醒所針對的時間（UTC）。 |
| `CreationTime`、`LastModifiedTime` | 與 Items 資料表相同。 |

### 工作 { #tasks }

| 欄位 | 判讀 |
|---|---|
| `FolderPath`、`ItemID`、`ItemType`、`Subject` | 與 Items 資料表相同：`Task` 或 `TaskRequest`。 |
| `Status` | `NotStarted`、`InProgress`、`Complete`、`Waiting` 或 `Deferred`。 |
| `PercentComplete` | 完成的程度，從 0 到 100。 |
| `StartTime`、`DueTime` | 開始日期與到期日：使用者時區中的日期，不附時差。 |
| `CompletedTime` | 完成的時間（UTC）。 |
| `Complete` | 是否已完成。 |
| `Owner`、`Assigner` | 工作的擁有者，以及指派工作的人。 |
| `ReminderSet`、`ReminderTime` | 與 Calendar 資料表相同。 |
| `CreationTime`、`LastModifiedTime` | 與 Items 資料表相同。 |

### 屬性 { #properties }

| 欄位 | 判讀 |
|---|---|
| `ItemID` | 項目的 ID；存放區本身的屬性為 `store`；資料夾的屬性為 `folder:<FolderID>`。 |
| `Scope` | 屬性的所屬對象：`Store`、`Folder`、`Item`、`Recipient` 或 `Attachment`。 |
| `Index` | 收件者或附件的索引；其他為空白。 |
| `PropertyTag` | 屬性的 ID 與類型，格式如 `0x0037001F`（`PidTagSubject`，字串）。 |
| `PropertyName` | 具名屬性（ID 自 `0x8000` 起）的屬性集與名稱：`PSETID_Address:0x8083`、`PS_PUBLIC_STRINGS:Keywords`，若非知名的屬性集則為該屬性集的 GUID。 |
| `Type` | 屬性類型：`PtypString`、`PtypInteger32`、`PtypTime`、`PtypBinary`、`PtypMultipleString` 等。 |
| `Value` | 值：字串為文字；時間為 ISO 8601 UTC（`-Z` 不會轉換）；二進位值為十六進位（前 4,096 個位元組，其後為 `…(N bytes)`）；多值屬性的各值以清單呈現（在 CSV 中以 `, ` 分隔）；內嵌項目或附件的資料為 `(object)`；未解碼類型的值為附註說明的十六進位位元組；損毀的值為可復原的內容，其後接含有十六進位位元組的附註（無法復原任何內容時，僅有位元組與附註）。 |

## 自動剖析 { #automatic-parsing }

`auto-parse` 會將這些資料表以 `Outlook-Files.csv`、`Outlook-Folders.csv`、`Outlook-Items.csv`、
`Outlook-Recipients.csv`、`Outlook-Attachments.csv`、`Outlook-Contacts.csv`、`Outlook-Calendar.csv`
與 `Outlook-Tasks.csv` 寫入新的 `Communication` 類別（使用 `-t json` 或 `-t jsonl` 時則為單一的
`Outlook.json` 或 `Outlook.jsonl` 檔案），並附上 `Communication/Outlook-logs.txt`。它會將其
`--code-page` 傳給剖析器，但既不會加入 Properties 資料表，也不會擷取附件：如需這些內容，請執行
`parse-file outlook`。合併的 `-j` 時間軸包含每一筆資料列，並帶有 `Comp` 以及 `Artifact` 識別碼
`outlook`。`-Z` 會轉換名稱以 `Time` 結尾的每一個欄位，但工作的 `StartTime` 與 `DueTime` 除外。

## 延伸資訊 { #further-details }

- [Microsoft — \[MS-PST\]：Outlook 個人資料夾（.pst）檔案格式](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]：Outlook 項目（.msg）檔案格式](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]：郵件與附件物件通訊協定](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]：Exchange Server 通訊協定主要屬性清單](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — Personal Folder File（PFF）格式](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
