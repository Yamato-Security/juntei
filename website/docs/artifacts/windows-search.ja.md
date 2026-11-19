# Windows Search のインデックス { #windows-search-index }

## インデックスの内容 { #what-the-index-records }

Windows Search (`WSearch` サービス) は、ユーザー プロファイル、スタート メニュー、Outlook のメールボックス、Internet Explorer の履歴、オフライン ファイルなど、スコープに含まれるものをクロールし、読み取った内容を ESE データベースである `Windows.edb` に保持します。調査で重要なのは、次の 2 つの部分です。

- **プロパティ ストア** (`SystemIndex_PropertyStore`。Windows 7 では `SystemIndex_0A`): インデックス化された項目ごとに 1 行があり、パス、名前、種類、サイズ、所有者、インデクサーがファイルからコピーした作成・更新・アクセス日時、インデクサーが読み取った日時、ドキュメントのタイトルと作成者、本文の冒頭、コンピューター名、その他数百のプロパティを保持します。
- **ギャザラー** (gatherer) テーブル (`SystemIndex_Gthr`、`SystemIndex_GthrPth`): クローラー自身による各項目の記録で、URL パス、キューに入れられた時点の更新日時、クロール番号とフラグ、削除カウンターを保持します。

項目は、ファイル、メッセージ、ページがなくなった後も、クローラーがその削除に気付くまでインデックスに残ります。そのため、インデックスにはすでに存在しないファイルが、インデックス化された時点のメタデータと本文の抜粋とともに記載されています。

<span class="evidence-strength">この証拠から判断できること:</span> インデクサーが、記録されたパスにある項目を、記録されたメタデータで、`GatherTime` に、記載のコンピューターで読み取りました。インデックスは、ファイルが存在したことと、その内容を示します。ユーザーがそのファイルを開いたことは示しません。

## 保存場所と収集 { #location-and-collection }

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

Windows 7 から Windows 11 21H2 まで、および同時期の Windows Server リリースは、インデックスを `Windows.edb` に保持します。Windows 11 22H2 以降は、同じフォルダー内の SQLite データベース (`Windows.db`、`Windows-gather.db`、`Windows-usn.db`) に保持します。Juntei はこれらを見つけて名前を示し、未対応の新しい形式として報告します。

`Applications\Windows` フォルダー全体を、ディスク イメージまたはボリューム シャドウ コピーから収集するか、`WSearch` サービスを停止してから収集してください。サービスの実行中、データベースは開かれたままで、最新の変更は `MSS*.log` トランザクション ログにしか存在しません。Juntei はこのログを再生しません。稼働中のシステムからコピーしたインデックスはダーティな状態のままです。出力にはその旨が示され、ディスク上にある内容を読み取ります。ディレクトリ入力と `auto-parse` は、任意の深さにあるすべての `Windows.edb` を読み取ります。1 つのファイルには `-f` を使用してください。

## アナリストへの注意事項 { #analyst-cautions }

- 変更されない証拠コピーを解析する。効率的に読み取るためにファイルをメモリマップするので、解析中にデータベースを変更したり切り詰めたりしない。

- `GatherTime` は、インデクサーが項目を読み取った日時であり、ユーザーが作成、変更、または開いた日時ではありません。インデックス作成はバックグラウンドで実行され、変更から数分～数時間後になることも多く、コンピューターがビジー状態のときやバッテリ駆動時には一時停止します。
- `DateCreated`、`DateModified`、`DateAccessed` は、インデクサーがファイルからコピーした値です。項目が再度インデックス化されると更新されるため、`GatherTime` 時点のファイルを表します。
- `ItemPath` は表示用のパスです。`ItemURL` と `GatherPath` はクローラーの URL で、ファイルは `file:` (Windows 8 以降ではボリュームが `[GUID]` で表されます)、Internet Explorer の履歴は `iehistory://{SID}/`、Outlook のアイテムは `mapi://`、オフライン ファイルは `csc://{SID}/`、Windows アプリのコンテンツは `winrt://{SID}/` です。
- `User` は、項目のパスに含まれるプロファイル名です (`C:\Users\<name>\…`)。項目があった場所を示すものであり、作成者を示すものではありません。`FileOwner` は、インデクサーが読み取った時点のファイルの所有者です。
- `AutoSummary` は項目の本文の冒頭です。その後削除または暗号化されたドキュメントの内容を引用していることがあります。また、切り詰められた抜粋であり、ドキュメントそのものではありません。
- ダーティなデータベースには、ブランチ ページからまだ到達できないページが含まれていることがあります。Juntei はリーフ ページ自身のチェーンをたどり、その方法で読み取ったページ数を概要に示します。最新の変更はトランザクション ログに残ったままです。
- ギャザラー テーブルにあってプロパティ ストアに行がない項目 (`InTables` が `Gthr`) は、キューに入れられたかクロールされたものの、プロパティがまだ書き込まれていないか削除されたものです。パス、名前、ギャザラーの日時は残っています。
- Windows 7 は、文字列を独自の圧縮され難読化されたエンコーディングで保存します。Juntei はこれをデコードします。デコードできない値は 16 進数のまま残り、`Problems` に注記されます。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

既定のファイル形式は CSV です。`-t csv`、`-t json`、`-t jsonl` を指定して `-o` を指定しない場合、レコードは標準出力に書き込まれます。どちらも指定しない場合は、概要だけが表示されます。既存の出力を上書きするには `-C` を追加してください。`-C` を指定しても、入力ファイルの上に出力が書き込まれることはありません。

Juntei は、インデックス化された項目ごとに 1 つのレコードを書き込みます。プロパティ ストアとギャザラー テーブルは、項目の識別子で結合します。概要には、スキーマごとのデータベース数、ダーティなデータベース数、両方のテーブルにある項目数、ギャザラーだけにある項目数、プロパティ ストアだけにある項目数、フォルダー数、削除済みとしてマークされた項目数、ストアごとの項目数、保存された日時のない項目数、問題の数が示されます。

破損があっても処理は止まりません。途中で切れたデータベースは、残っているページに含まれる項目を保持し、読み取れなかった内容を概要に示します。`Windows.edb` という名前でも Windows Search のインデックスではないファイルは、報告したうえでスキップします。ほかのデータベースは引き続き読み取ります。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `DocumentID` | インデックス内の項目の識別子。プロパティ ストアの `WorkID` (Windows 7 では `DocID`) と、ギャザラーの行の `DocumentID` です。 |
| `InTables` | `Gthr+PropertyStore`、`Gthr` (プロパティのない、キューに入れられたかクロールされた項目)、または `PropertyStore`。 |
| `ItemPath` | `System_ItemPathDisplay`: Windows が表示するとおりのパス。プロパティ ストアに行がない場合は空です。 |
| `ItemURL` | `System_ItemUrl`: 項目に対するクローラーの URL (`file:`、`iehistory://`、`mapi://`、`csc://`、`winrt://`)。 |
| `GatherPath` | ギャザラーのパス。スコープ ツリーと行のファイル名を結合したものです。 |
| `Name` | `System_ItemName`、またはギャザラーの行のファイル名。 |
| `ItemType`, `ItemTypeText` | `System_ItemType` (拡張子、または `Directory`) と、その表示名。 |
| `Kind` | `System_Kind`: `document`、`folder`、`link`、`program`、`email`、`picture` など。複数ある場合は `\|` で結合します。 |
| `MimeType` | `System_MIMEType`。 |
| `Store` | `System_Search_Store`: `file`、`mapi` (Outlook)、`iehistory`、`csc` (オフライン ファイル)、または `winrt` (Windows アプリ)。 |
| `IsFolder`, `IsDeleted` | 保存されているとおりの `System_IsFolder` と `System_IsDeleted`。 |
| `DeletedCount` | ギャザラーの行の削除カウンター。 |
| `Size` | `System_Size` (バイト単位)。プレースホルダーが保存されているフォルダーでは空です。 |
| `FileAttributes` | 数値で表した `System_FileAttributes` (16 はディレクトリ、32 はアーカイブ、2 は隠しファイルなど)。 |
| `FileOwner` | インデクサーが読み取った時点の `System_FileOwner` (`DOMAIN\user`)。 |
| `DateCreated`, `DateModified`, `DateAccessed` | インデクサーがコピーした時点のファイルの日時 (UTC)。 |
| `ItemDate` | `System_ItemDate`: 項目自身の日付。メッセージの送信時刻や写真の撮影日時などで、ファイルの場合は通常、秒単位の更新日時です。 |
| `GatherTime` | `System_Search_GatherTime`: インデクサーが項目を読み取った日時 (UTC)。 |
| `GatherLastModified` | ギャザラーの行の `LastModified`: クローラーが項目をキューに入れた時点の更新日時 (UTC)。 |
| `Title`, `Author` | `System_Title` と `System_Author`。 |
| `AutoSummary` | `System_Search_AutoSummary`: 項目の本文の冒頭。 |
| `CompName` | `System_ComputerName`: インデックスが属するコンピューター。 |
| `TargetURL`, `DateVisited` | Internet Explorer の履歴項目の URL と閲覧日時 (`System_Link_TargetUrl`、`System_Link_DateVisited`)。 |
| `ActivityApp`, `ActivityText`, `ActivityContentURI`, `ActivityStartTime`, `ActivityEndTime` | Windows 10 のアクティビティ履歴の項目 (`System_Activity_*`、`System_ActivityHistory_*`)。 |
| `User`, `UserSource` | 項目のパスに含まれるプロファイル名と、それをどのパスから読み取ったか (`ItemPath`、`ItemURL`、または `GatherPath`)。プロファイルの外にある場合は空です。 |
| `ScopeID`, `SDID` | ギャザラーの行のスコープ識別子とセキュリティ記述子識別子。 |
| `CrawlNumber` | ギャザラーの行の `CrawlNumberCrawled`。 |
| `GatherFlags` | ギャザラーの行の `TransactionFlags`。 |
| `Priority` | まだキューにある項目のクロール優先度。ESE は項目がクロールされると NULL にするため、ほとんどの項目では空です。 |
| `PropertyCount` | プロパティ ストアがその項目について保持するプロパティの数。 |
| `OtherProperties` | そのほかのすべてのプロパティ。`; ` で区切った `Name=Value` の形式です。 |
| `SourceFile` | 読み取ったデータベース。入力ディレクトリからの相対パス、`-f` の場合はファイル名です。 |
| `Problems` | デコードできなかった値、時刻ではない時刻、重複する行、破損。省略しないでください。 |

JSON と JSONL は、同じフィールドを型付きの値で保持します。数値は数値のままで、ない値は `null` です。`OtherProperties` の代わりに `Properties` があり、項目のすべてのプロパティを名前、数値識別子、列の型、値とともに保持します。時刻については、読み取った時刻と並べて元の FILETIME のティック数を保持します。`Gather` はギャザラーの行の列を保持します。

## タイムラインでの扱い { #timeline-behavior }

`auto-parse` は、レコードを `FileExistence/WindowsSearch.csv` (または `.json`/`.jsonl`) に保存し、統合された `-j` タイムラインに追加します。`super-timeline` は、`DateCreated`、`DateModified`、`DateAccessed` (Windows Search が記録したファイルの日時)、`GatherTime` (項目がインデックス化された日時)、Internet Explorer の履歴項目の `DateVisited`、アクティビティの `ActivityStartTime` と `ActivityEndTime` からイベントを作成します。`GatherLastModified` は、プロパティ ストアに更新日時がない項目についてだけイベントになります。これらの日時がいずれもない項目は解析済みの出力に残り、時刻なしとして数えられます。

## 調査の進め方 { #investigation-strategy }

1. `Store` を `file` で絞り込み、`ItemPath`、`Name`、`AutoSummary` から、事案に関係するファイル、名前、語句を検索します。インデックスには、その後削除されたファイルも記載されています。
2. `GatherTime` は、記録されたメタデータでファイルが存在していた最新の日時として、`DateModified` はその時点の更新日時として読み取ります。
3. `FileOwner` と `User` を使ってファイルをアカウントに結び付け、`$MFT`、USN ジャーナル、`$Secure:$SDS` で確認します。
4. 電子メール (`Store` が `mapi`) と Internet Explorer の履歴 (`iehistory`) については、完全なレコードを得るために Outlook のファイルと WebCache に展開します。
5. ホスト間でインデックスを比較します。`CompName` はインデックスが属するコンピューターを示し、イメージから収集したインデックスはそのイメージ上のファイルを示します。

## テストしたバージョン { #tested-versions }

テストには、ほかのツールが独立して読み取った結果のある公開インデックスを使用しています。

- plaso の Windows 7 のインデックス (`SystemIndex_0A`)。Windows 7 の文字列エンコーディングを含め、libyal の `esedbexport` と照合しています。
- DFIRArtifactMuseum の Windows 10 のインデックス (ダーティなまま)。Stroz Friedberg の SIDR と照合しています。同じく DFIRArtifactMuseum の Windows 11 のインデックス (ESE 形式リビジョン 200) は、`esedbexport` と照合しています。
- DFIRArtifactMuseum の Windows 11 22H2 の SQLite インデックス。未対応の新しい形式として報告されます。
- 途中で切れたデータベース、空のデータベース、別の種類のデータベース。

## 詳細情報 { #further-details }

- [Microsoft — Windows Search の概要](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE データベース ファイル形式](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb — Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg — SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft — Extensible Storage Engine のソース](https://github.com/microsoft/Extensible-Storage-Engine)
