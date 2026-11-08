# Outlook { #outlook }

## Outlook のファイルとは { #what-outlooks-files-are }

Windows 版 Outlook は、メール、連絡先、予定表、タスクを独自のファイルに保存します。

- **OST** (`.ost`): メールボックスのオフライン コピーです。Outlook は Exchange と Microsoft 365 のアカウントについてこれを保持し、Outlook 2013 以降では IMAP アカウントについても保持します。Outlook 2013 以降の OST は、4 KiB のページと圧縮されたブロックを使用します。
- **PST** (`.pst`): 個人用のストアです。POP3 アカウント、アーカイブ (`archive.pst`)、エクスポート、およびユーザーが作成したストアがこれにあたります。ANSI PST は Outlook 97 ～ 2002 のもの、Unicode PST は Outlook 2003 以降のものです。
- **MSG** (`.msg`): ファイルとして保存された 1 つのアイテムです。Outlook の外にドラッグされたとき、**名前を付けて保存** で保存されたとき、または他のプログラムがエクスポートしたときに作成されます。添付されたメッセージも MSG ファイルです。
- **OFT** (`.oft`): Outlook のテンプレートです。新しいアイテムの作成元となる MSG ファイルです。

各ストアはフォルダーを保持し、各フォルダーはそのアイテムを保持します。電子メール、会議出席依頼とその返信、連絡先と配布リスト、予定表アイテム、タスク、メモ、履歴 (Journal) のエントリ、レポートです。各フォルダーは非表示のアイテム、すなわちフォルダー関連情報 (FAI) も保持しています。ビュー、ルール、フォーム、および Outlook の設定 (Exchange メールボックスのオートコンプリート リストなどの `IPM.Configuration.*` アイテム) です。アイテムの添付ファイルは、ファイル、ファイルへの参照、OLE オブジェクト、または他のアイテム (添付ファイルとして転送されたメッセージ、定期的な予定の例外) の場合があり、そうしたアイテムもそれぞれ独自の添付ファイルを持ちます。

<span class="evidence-strength">この証拠から判断できること:</span> 収集された時点で、ストアまたはファイルがそのアイテムを、記録された送信者、受信者、時刻、添付ファイルとともに保持していました。ユーザーがそれを読んだ、書いた、または送信したことも、送信者がアイテムに示されているとおりの人物であることも、この証拠からはわかりません。

## 保存場所と収集 { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

PST はどこにでも置かれている可能性があります。ユーザーはアーカイブを他のフォルダー、ドライブ、ファイル共有に移動します。Outlook は、各プロファイルのストアのパスをユーザーのレジストリに保存しています。ユーザーが Outlook から開いた添付ファイルは、`%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\` 内のフォルダー (Windows 8.1 より前は `Temporary Internet Files\Content.Outlook`) にコピーされ、そこに残っていることがあります。

Outlook は実行中ストアを開いたままにし、OST は数十 GB に達することがあります。Outlook を終了した状態で収集するか、ディスク イメージまたはボリューム シャドウ コピーから収集してください。Juntei は各ストアをストリームとして読み取り、ストア全体をメモリに保持することはありません。

## 分析時の注意点 { #analyst-cautions }

- Juntei は寛容に読み取ります。損傷した箇所を越えて読み進め、読み取れるものを読み取り、残りを問題として報告します。問題は画面に表示され (ファイルごとに最初の 10 件)、概要と Files テーブルで数えられ、該当するアイテムと添付ファイルの `Problems` フィールドに記載されます。まったく読み取れないアイテムにも、Items テーブルに `ItemID` とその理由を含む行があります。
- 時刻は UTC で、100 ナノ秒単位です。`SubmitTime` は送信者のクライアントがメッセージを送信した時刻で、そのコンピューターの時計によるものです。`DeliveryTime` はメッセージがこのメールボックスに届いた時刻です。`CreationTime` はアイテムのこのコピーが作成された時刻です。すなわち、メールボックスに配信または作成された時刻、OST に同期された時刻、PST にコピーまたはインポートされた時刻、あるいは MSG ファイルとして保存された時刻です。メッセージが書かれた時刻ではありません。`LastModifiedTime` は、アイテムがストア内で最後に変更された時刻です。
- タスクの `StartTime` と `DueTime` はユーザーのタイム ゾーンでの日付で、Outlook はこれを午前 0 時として保存します。Juntei はこれらをオフセットなしで書き込み (`2017-02-19T00:00:00`)、`-Z` もそのままにします。
- Outlook は、タスクの期限やアラームの時刻など一部の日付で、**なし** を表すために 4501 年 1 月 1 日の午前 0 時を書き込みます。Juntei はこれを保存されているとおりに `4501-01-01T00:00:00Z` と書き込みます。タスクの `StartTime` と `DueTime` では `Z` は付きません。
- `Read` は、ストアが収集された時点の既読フラグです。誰か、またはルールがアイテムを既読にしたことを示しますが、それが誰で、いつだったかは示しません。
- 送信者と受信者の名前とアドレスは、送信側のプログラムとメール サーバーが記録したものであり、偽装されている可能性があります。`EX` アドレスは Exchange の識別名 (`/o=…/ou=…/cn=Recipients/cn=…`) です。アイテムがそれに加えて SMTP アドレスも保存している場合、SMTP アドレスは `SenderSMTP-Address` と `SMTP-Address` に入ります。メッセージの送信元を確認するには、そのインターネット ヘッダー、すなわち Properties テーブルの `0x007D001F` または `0x007D001E` (`PidTagTransportMessageHeaders`) を読んでください。
- Outlook で削除されたアイテムは、空にされるまで **削除済みアイテム** (Deleted Items) にあり、他のアイテムと同様に読み取られます。どのフォルダーのテーブルにも含まれなくなったアイテムは、ストアがその領域を再利用するまで残っていることがありますが、現時点では読み取られません。
- `ItemID` は、PST または OST 内でのメッセージのノード ID です。ストアの別のコピーでは、同じアイテムが別の ID を持つことがあります。埋め込まれたアイテムの ID は、親の ID と添付ファイルのインデックスです (`2097348/0`、`0/0/1`)。`.msg` または `.oft` ファイルのアイテムは `0` です。
- `User` はパスのみから取得され (`Users` の次のフォルダー)、ファイルが別の場所にコピーされると失われます。PST はどのユーザーのものである可能性もあります。
- Unicode でないテキストは、アイテムのコード ページ (アイテム自身のもの、そのロケールのもの、またはそのインターネット コード ページのもの) で読み取られます。`--code-page` は、コード ページを示さないアイテムのコード ページを指定します。それらを書き込んだシステムの ANSI コード ページで、日本語のシステムでは `932` です。
- PST のパスワードはファイルを暗号化しません。Juntei は、パスワードで保護された PST も他の PST と同様に読み取ります。ストアの `Encryption` (`Permute`、すなわち Outlook の「圧縮可能な暗号化」、または `Cyclic`) は、ブロックを難読化するだけです。S/MIME で暗号化されたメッセージは、本文と添付ファイルを暗号化された添付ファイル (`smime.p7m`) に格納しており、これは復号されません。
- 添付ファイルはしばしばマルウェアです。`--extract-attachments` は、そのデータを SHA-256 で名前を付けたファイルに、実行権限なしで書き込みます。決して実行せず、隔離された分析用システムで保管してください。

### まだ読み取らないもの { #not-read-yet }

オフライン アドレス帳 (OAB)、オートコンプリート ストリーム (`Stream_Autocomplete_*.dat`) と `.nk2` ファイル、送受信の設定 (`.srs`)、Outlook の設定キャッシュ (RoamCache、Autodiscover)、Outlook の SQLite データベース、およびどのフォルダーのテーブルにも含まれなくなったメッセージ。

## Juntei で解析する { #parse-with-juntei }

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

`-d` は、大文字と小文字を区別せずに、名前が `.pst`、`.ost`、`.msg`、`.oft` で終わるファイルを検索します。どの方法で指定されたファイルでも、その種類は先頭のバイトで判別されます。PST または OST は `!BDN`、MSG または OFT は複合ファイルのシグネチャです。それ以外のものは、解析できなかったファイルとして報告されます。どのファイルも読み取れなかった場合、終了ステータスは 1 になります。

`--extract-attachments` は、各添付ファイルのデータを 1 回だけ書き込みます。ファイル名は `<SHA-256>` で、添付ファイルの拡張子が 1 ～ 10 文字の ASCII 英数字である場合はその拡張子が付きます (`0c7d…1311.jpg`)。アイテムに含まれる名前がパスに使われることはありません。`-C` を指定しない限り、ディレクトリは空である必要があります。`-C` を指定すると、同じ名前で内容の異なるファイルは置き換えられます。

## 出力テーブル { #output-tables }

CSV では 8 つのファイル (`--properties` を指定すると 9 つ) が作成されます。行がない場合は、ヘッダーだけのファイルになります。JSON と JSONL の各行には、`SourceFile` の後に `Table` (`Files`、`Folders`、`Items`、`Recipients`、`Attachments`、`Contacts`、`Calendar`、`Tasks`、または `Properties`) が含まれます。アイテムの行の後にはそのアイテムに属する行が続き、その後にその埋め込みアイテムの行が続きます。ファイルの Files 行は、そのファイルの他のすべての行の後に来ます。

| CSV ファイル | 内容 |
|---|---|
| `Outlook-Files.csv` | 読み取ったファイルごとに 1 行: ストアまたはファイルと、その中身。 |
| `Outlook-Folders.csv` | PST と OST のフォルダー。 |
| `Outlook-Items.csv` | すべてのアイテム: フォルダー内のメッセージ、フォルダーの非表示アイテム、`.msg` と `.oft` ファイルのアイテム、およびそれらの添付ファイルに埋め込まれたアイテム。 |
| `Outlook-Recipients.csv` | アイテムの受信者。 |
| `Outlook-Attachments.csv` | アイテムの添付ファイル。 |
| `Outlook-Contacts.csv` | 連絡先と配布リストのフィールド。 |
| `Outlook-Calendar.csv` | 予定表アイテムと会議メッセージのフィールド。 |
| `Outlook-Tasks.csv` | タスクとタスクの依頼のフィールド。 |
| `Outlook-Properties.csv` | `--properties` を指定した場合: ストア、フォルダー、アイテム、受信者、添付ファイルのすべてのプロパティ (保存されているまま)。 |

## フィールド { #fields }

すべてのテーブルは、次のフィールドで始まります。

| フィールド | 解釈 |
|---|---|
| `SourceFile` | 行の読み取り元のファイル。 |
| `User` | パス上でプロファイルが属する Windows ユーザー (`Users` の次のフォルダー)。 |

### ファイル { #files }

| フィールド | 解釈 |
|---|---|
| `FileType` | `PST`、`OST`、`MSG`、`OFT`、または `CFB` (クラス ID がメッセージのものでもテンプレートのものでもない、アイテムとして読み取られた複合ファイル。Outlook 以外の多くのプログラムはクラス ID を空のままにします)。 |
| `Format` | ストアのレイアウト: `ANSI` (Outlook 97 ～ 2002)、`Unicode` (Outlook 2003 以降)、または `Unicode 4K` (Outlook 2013 以降の OST)。MSG または OFT ファイルでは、複合ファイルのバージョン (`CFB v3`、`CFB v4`)。 |
| `Version`, `ClientVersion` | ストアのヘッダーの `wVer` と `wVerClient`。 |
| `Encryption` | ストアのブロックのエンコード方式: `None`、`Permute`、または `Cyclic`。 |
| `DisplayName` | ストアの名前 (`Personal Folders`、電子メール アドレス)。 |
| `RecordKey` | ストアの識別子 (16 進数)。ストア内のアイテムのエントリ ID に含まれます。 |
| `Folders` | ストアのフォルダーの数。 |
| `Items` | フォルダー内のメッセージの数。MSG または OFT ファイルでは 1。 |
| `HiddenItems` | フォルダーの非表示アイテム (フォルダー関連情報) の数。 |
| `EmbeddedItems` | 添付ファイルに埋め込まれたアイテムの数 (すべての階層)。 |
| `Attachments` | それらすべてのアイテムの添付ファイルの数。 |
| `Problems` | ファイル内で見つかった問題の数。 |

### フォルダー { #folders }

| フィールド | 解釈 |
|---|---|
| `FolderID`, `ParentID` | フォルダーのノード ID と、親フォルダーのノード ID (ルート フォルダーでは空)。 |
| `FolderPath` | ルートから順にたどったフォルダーの名前: `/Top of Personal Folders/Inbox`、`/Root - Mailbox/IPM_SUBTREE/Sent Items`。 |
| `Name` | フォルダーの名前。 |
| `ContainerClass` | フォルダーが保持するものの種類: `IPF.Note` (メール)、`IPF.Appointment`、`IPF.Contact`、`IPF.Task`、`IPF.Imap`、…。 |
| `ContentCount`, `UnreadCount` | フォルダー自身が示すメッセージの数と未読メッセージの数。 |
| `Messages`, `HiddenItems`, `Subfolders` | フォルダーの内容テーブル、関連内容テーブル、階層テーブルの行の数。Juntei が読み取るのはこれらの行です。`ContentCount` との差異は確認する価値があります。 |
| `CreationTime`, `LastModifiedTime` | フォルダーが作成された時刻と最後に変更された時刻 (UTC)。ストアが保持している場合のみ。 |
| `Problem` | フォルダーを開けなかった理由。その場合、サブフォルダーは読み取られません。 |

### アイテム { #items }

| フィールド | 解釈 |
|---|---|
| `FolderPath` | アイテムのフォルダー。MSG または OFT ファイルでは空です。 |
| `ItemID` | アイテムの ID: PST または OST ではノード ID、MSG または OFT ファイルでは `0`、埋め込まれたアイテムでは親の ID と添付ファイルのインデックス (`2097348/0`)。 |
| `Hidden` | アイテムがフォルダー関連情報、すなわち非表示アイテムであるかどうか。 |
| `EmbeddedIn` | アイテムが添付されている親アイテムの ID。それ以外のアイテムでは空です。 |
| `ItemType` | メッセージ クラスが示すアイテムの種類: `Mail`、`MeetingMessage`、`Appointment`、`Contact`、`DistributionList`、`Task`、`TaskRequest`、`StickyNote`、`Journal`、`Post`、`Report`、`Configuration`、または `Other`。 |
| `MessageClass` | 保存されているままのメッセージ クラス (`IPM.Note`、`IPM.Note.SMIME`、`IPM.Schedule.Meeting.Request`、`REPORT.IPM.Note.NDR`、…)。 |
| `SubmitTime` | 送信者のクライアントがメッセージを送信した時刻 (UTC、送信者の時計による)。 |
| `DeliveryTime` | このメールボックスに配信された時刻 (UTC)。 |
| `CreationTime`, `LastModifiedTime` | このコピーがストアまたはファイル内に作成された時刻と、最後に変更された時刻 (UTC)。 |
| `Subject` | プレフィックス (`RE: `) を含む件名。 |
| `SenderName`, `SenderAddressType`, `SenderAddress` | 送信者: 名前、アドレスの種類 (`SMTP`、`EX`)、およびその種類のアドレス。 |
| `SenderSMTP-Address` | 送信者の SMTP アドレス。別の種類のアドレスに加えて保存されている場合のみ。 |
| `SentRepresentingName`, `SentRepresentingAddress` | 誰の代わりに送信されたか (代理人が上司の代わりに送信する場合、共有メールボックス)。アドレスは、保存されていれば SMTP アドレス、なければその種類のアドレスです。 |
| `To`, `Cc`, `Bcc` | アイテムに表示されるとおりの受信者の名前 (アドレスは Recipients を参照)。`Bcc` は送信者のコピーにのみ存在します。 |
| `InternetMessageID`, `InReplyTo` | メッセージの `Message-ID` と、返信元のメッセージの ID。 |
| `ConversationTopic` | プレフィックスを除いた件名。同じ会話のメッセージで共通です。 |
| `Importance`, `Sensitivity` | 重要度は `Low`、`Normal`、または `High`。秘密度は `Normal`、`Personal`、`Private`、または `Confidential`。 |
| `Read` | アイテムが既読としてマークされているかどうか。 |
| `Flags` | メッセージ フラグ (`PidTagMessageFlags`、16 進数): `0x1` 既読、`0x2` 未変更、`0x4` 送信処理済み、`0x8` 未送信 (下書き)、`0x10` 添付ファイルあり、`0x20` 自分から、`0x40` 非表示、`0x80` 再送信、`0x400` 一度でも既読になった、`0x2000` インターネットから。 |
| `Size` | 添付ファイルを含む、ストア内でのアイテムのサイズ。 |
| `Recipients`, `Attachments` | 受信者の数と添付ファイルの数。 |
| `Categories` | アイテムの分類項目 (`;` 区切り)。 |
| `BodyPreview` | プレーン テキスト本文の先頭部分。空白文字はまとめられ、最大 256 文字です。HTML または RTF の本文しかない場合は空です。 |
| `Problems` | アイテム内で読み取れなかったもの (最大 10 件。それを超える場合は残りの件数)。 |

### 受信者 { #recipients }

| フィールド | 解釈 |
|---|---|
| `ItemID` | アイテムの ID。 |
| `Index` | アイテムの受信者テーブルにおける受信者の行 (0 から)。 |
| `RecipientType` | `To`、`Cc`、`Bcc`、`Originator`、または保存されているままの値 (16 進数)。 |
| `Name`, `AddressType`, `Address` | 受信者の名前、アドレスの種類、アドレス。 |
| `SMTP-Address` | SMTP アドレス。別の種類のアドレスに加えて保存されている場合のみ。 |

### 添付ファイル { #attachments }

| フィールド | 解釈 |
|---|---|
| `ItemID`, `Index` | アイテムの ID と、アイテムの添付ファイル テーブルにおける添付ファイルの行 (0 から)。 |
| `Method` | `ByValue` (ファイル)、`ByReference`、`ByReferenceResolve`、`ByReferenceOnly` (ファイルへのパス)、`EmbeddedMessage` (アイテム)、`OLE` (OLE オブジェクト)、`ByWebReference`、または数値。 |
| `Filename` | 添付ファイルの名前: 長いファイル名、短いファイル名、または表示名。 |
| `Extension` | 保存されているままの拡張子 (`.pdf`)。なければ名前の拡張子。 |
| `MimeType`, `ContentID` | MIME タイプと、HTML 本文が添付ファイルを参照するときのコンテンツ ID。 |
| `Inline` | 添付ファイルが一覧に表示されるのではなく、本文中に表示されるかどうか (HTML 本文の画像)。 |
| `Hidden` | Outlook が添付ファイルの一覧で非表示にするかどうか。 |
| `Size` | ストアが示すサイズ (プロパティを含む)。 |
| `DataSize`, `SHA256` | データのサイズと SHA-256。データとは、ファイル、または PST や OST 内の OLE オブジェクトの複合ファイルです。埋め込まれたアイテムと参照による添付ファイルでは空です。 |
| `Pathname` | 参照による添付ファイルのパス。 |
| `CreationTime`, `LastModifiedTime` | 添付ファイルの時刻 (UTC)。多くの場合、作成元のファイルの時刻です。 |
| `EmbeddedSubject` | 埋め込まれたアイテムの件名。そのアイテム自身の行の `ItemID` は `<ItemID>/<Index>` です。 |
| `ExtractedFile` | `--extract-attachments` を指定した場合、データが書き込まれた、そのディレクトリ内のファイル。 |
| `Problems` | 添付ファイル内で読み取れなかったもの、または書き込めなかったもの。 |

### 連絡先 { #contacts }

| フィールド | 解釈 |
|---|---|
| `FolderPath`, `ItemID`, `ItemType` | Items テーブルと同じです。種類は `Contact` または `DistributionList`。 |
| `DisplayName`, `GivenName`, `Surname`, `Nickname` | 連絡先の名前。 |
| `FileUnder` | 連絡先の表題 (連絡先が分類される名前)。 |
| `Company`, `Title`, `Department` | 連絡先の勤務先 (会社名、役職、部署)。 |
| `Email1Name`, `Email1Address`, `Email2Name`, `Email2Address`, `Email3Name`, `Email3Address` | 連絡先の 3 つの電子メール アドレスと、それぞれの表示名。 |
| `BusinessPhone`, `HomePhone`, `MobilePhone` | 連絡先の電話番号。 |
| `InstantMessaging` | 連絡先のインスタント メッセージングのアドレス。 |
| `CreationTime`, `LastModifiedTime` | Items テーブルと同じです。 |

### 予定表 { #calendar }

| フィールド | 解釈 |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Items テーブルと同じです。種類は `Appointment` または `MeetingMessage`。 |
| `StartTime`, `EndTime` | 予定の開始時刻と終了時刻 (UTC)。定期的な予定では、最初の回の時刻です。 |
| `DurationMinutes` | 予定の長さ (分)。 |
| `Location` | 予定の場所。 |
| `AllDay`, `Recurring` | 終日のイベントかどうかと、定期的な予定かどうか (その例外は埋め込まれたアイテムです)。 |
| `BusyStatus` | `Free`、`Tentative`、`Busy`、`OutOfOffice`、または `WorkingElsewhere`。 |
| `ResponseStatus` | ユーザーの応答: `None`、`Organized`、`Tentative`、`Accepted`、`Declined`、または `NotResponded`。 |
| `Organizer` | アイテムの送信者の名前。 |
| `AllAttendees`, `ToAttendees`, `CcAttendees` | アイテムに記載されているとおりの出席者。 |
| `GlobalObjectID` | メールボックスをまたいで会議を識別する ID (16 進数)。開催者の予定表と出席者の予定表にある同じ会議で共通です。 |
| `ReminderSet`, `ReminderTime` | アラームが設定されているかどうかと、アラームの対象の時刻 (UTC)。 |
| `CreationTime`, `LastModifiedTime` | Items テーブルと同じです。 |

### タスク { #tasks }

| フィールド | 解釈 |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Items テーブルと同じです。種類は `Task` または `TaskRequest`。 |
| `Status` | `NotStarted`、`InProgress`、`Complete`、`Waiting`、または `Deferred`。 |
| `PercentComplete` | 進捗度 (0 ～ 100)。 |
| `StartTime`, `DueTime` | 開始日と期限: ユーザーのタイム ゾーンでの日付で、オフセットはありません。 |
| `CompletedTime` | 完了した時刻 (UTC)。 |
| `Complete` | 完了しているかどうか。 |
| `Owner`, `Assigner` | タスクの所有者と、タスクを割り当てた人。 |
| `ReminderSet`, `ReminderTime` | Calendar テーブルと同じです。 |
| `CreationTime`, `LastModifiedTime` | Items テーブルと同じです。 |

### プロパティ { #properties }

| フィールド | 解釈 |
|---|---|
| `ItemID` | アイテムの ID。ストア自体のプロパティでは `store`、フォルダーのプロパティでは `folder:<FolderID>`。 |
| `Scope` | プロパティの持ち主: `Store`、`Folder`、`Item`、`Recipient`、または `Attachment`。 |
| `Index` | 受信者または添付ファイルのインデックス。それ以外では空です。 |
| `PropertyTag` | プロパティの ID と型。`0x0037001F` の形式です (`PidTagSubject`、文字列)。 |
| `PropertyName` | 名前付きプロパティ (ID が `0x8000` 以上) では、そのプロパティ セットと名前: `PSETID_Address:0x8083`、`PS_PUBLIC_STRINGS:Keywords`、またはよく知られたセットでない場合はセットの GUID。 |
| `Type` | プロパティの型: `PtypString`、`PtypInteger32`、`PtypTime`、`PtypBinary`、`PtypMultipleString`、…。 |
| `Value` | 値: 文字列はテキスト、時刻は ISO 8601 の UTC (`-Z` は変換しません)、バイナリ値は 16 進数 (最初の 4,096 バイトの後に `…(N bytes)`)、複数値プロパティは値のリスト (CSV では `, ` 区切り)、埋め込まれたアイテムや添付ファイルのデータは `(object)`、デコードされない型の値は注記付きの 16 進数のバイト列、損傷した値は復元できた値の後に 16 進数のバイト列を含む注記 (何も復元できなかった場合はバイト列と注記のみ)。 |

## 自動解析 { #automatic-parsing }

`auto-parse` は、テーブルを新しい `Communication` カテゴリに `Outlook-Files.csv`、`Outlook-Folders.csv`、`Outlook-Items.csv`、`Outlook-Recipients.csv`、`Outlook-Attachments.csv`、`Outlook-Contacts.csv`、`Outlook-Calendar.csv`、`Outlook-Tasks.csv` として書き出し (`-t json` または `-t jsonl` の場合は 1 つの `Outlook.json` または `Outlook.jsonl` ファイル)、あわせて `Communication/Outlook-logs.txt` を作成します。パーサーには `--code-page` を渡しますが、Properties テーブルの追加も添付ファイルの抽出も行いません。それらが必要な場合は `parse-file outlook` を実行してください。結合された `-j` タイムラインにはすべての行が含まれ、`Comp` と `Artifact` 識別子 `outlook` が付きます。`-Z` は、タスクの `StartTime` と `DueTime` を除き、`Time` で終わるすべてのフィールドを変換します。

## さらに詳しい情報 { #further-details }

- [Microsoft — \[MS-PST\]: Outlook 個人用フォルダー (.pst) ファイル形式](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]: Outlook アイテム (.msg) ファイル形式](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]: メッセージおよび添付ファイル オブジェクト プロトコル](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]: Exchange Server プロトコル マスター プロパティ リスト](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — Personal Folder File (PFF) 形式](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
