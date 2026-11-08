# Web ブラウザ { #web-browsers }

## ブラウザのデータベースとは { #what-the-browser-databases-are }

ブラウザは、閲覧履歴、ダウンロード、Cookie、フォーム入力を、各ユーザーのプロファイル内にある SQLite データベースに保存します。Chromium 系ブラウザ (Chrome、Edge、Brave、Opera、Vivaldi) と Firefox 系ブラウザはそれぞれ独自のファイルとテーブルを使用しますが、記録するアクティビティの種類は同じです。

- **閲覧履歴**: 閲覧したページ。各閲覧の時刻、ブラウザがそのページに至った経緯 (入力、リンク、リダイレクト、ブックマーク)、および Chromium ではページが前面に表示されていた時間。
- **ダウンロード**: ファイルの取得元 URL (および途中のリダイレクト)、保存先、サイズ、ダウンロードの開始と終了の時刻、完了したかどうか。
- **Cookie**: Cookie を設定したサイトと、設定、最終送信、有効期限の各時刻。
- **フォーム入力履歴**: Web フォームや検索ボックスに入力された値と、その入力回数、最初と最後の入力時刻。

ブラウザのアクティビティは、ユーザーが何をしたかを最も直接的に示す記録であることが少なくありません。たどったフィッシング ページ、ダウンロードしたツール、使用した Web メール、入力した検索語などです。

<span class="evidence-strength">この証拠から判断できること:</span> ブラウザのプロファイルが、記録された時刻にその URL への閲覧を記録した、または記録されたパスにダウンロードを保存しました。誰がキーボードの前にいたかは、データベースからはわかりません。

## 保存場所と収集 { #location-and-collection }

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

`<Profile>` は `Default`、`Profile 1`、`Profile 2` などです。Chromium はバージョン 96 で `Cookies` を `Network` に移動しました。それより前のバージョンでは `History` と同じフォルダーにあります。Firefox はバージョン 25 まではダウンロードを `downloads.sqlite` に保存していましたが、それ以降のバージョンでは `places.sqlite` に保存します。

プロファイルのファイルはすべて収集してください。また、各データベースは、隣にある同じ名前のファイル (`places.sqlite-wal`、`cookies.sqlite-wal`、`formhistory.sqlite-wal`、`History-journal`) と一緒に収集してください。

!!! warning "ライトアヘッドログ"
    Firefox はデータベースをライトアヘッドログ モードで使用します。最新の変更は、ブラウザがそれをデータベースに書き込む (チェックポイント) までは `-wal` ファイルにしか存在しません。Juntei は、ブラウザと同じように、各データベースをその `-wal` ファイルとともに読み取ります。このファイルがないと、最近の閲覧、Cookie、フォーム入力が欠落し、ログ内で削除されたエントリが削除されていないものとして読み取られます。

## 削除されたエントリ { #deleted-entries }

ブラウザが履歴を削除すると、行はテーブルから消えますが、そのレコードはデータベースの空き領域、フリーリスト、またはデータベースのページの古いバージョンに残ることがあります。古いバージョンのページは、`-wal` ファイルの中、または `-wal` 内のページに置き換えられたデータベース ファイルの中にあります。Juntei は、これらから削除された閲覧、ページ、ダウンロード、フォーム入力を、`Status` が `Deleted` の行として復元します。

- `Confidence` は、レコードとテーブルの対応付けの確かさを示します。すべての値がその列の型に合う完全なレコードは `High`、損傷したレコードや複数のテーブルに当てはまるレコードは `Medium` または `Low` です。
- `RecoveredFrom` は、レコードがあった場所を示します。ページ内の領域 (セル、フリーブロック、未割り当て領域、空きページ)、どのページのどのバージョンか、およびページ内でのレコードのオフセットです。WAL フレームは 0 から数えます。
- `LostColumns` は、損傷したレコードに残っていない列を列挙します。それらのフィールドは空になります。

ブラウザは SQLite の `secure_delete` を使用して削除するため、データベース内で解放された領域はゼロで埋められます。そのため、復元できるデータの大半は古いバージョンのページにあります。すなわち、`-wal` ファイルにまだ残っているページと、`-wal` によって置き換えられたページについてデータベース ファイル自体が保持しているコピーで、いずれも次のチェックポイントまでしか残りません。`-wal` ファイルはデータベースと一緒に、ブラウザが再び実行される前にコピーしてください。

## 分析時の注意点 { #analyst-cautions }

- 時刻は UTC です。Chromium と Firefox はマイクロ秒単位で記録しますが、Chromium のフォーム入力履歴は秒単位です。
- 閲覧の記録があっても、ユーザーが入力またはクリックしたとは限りません。リダイレクト、フレーム、他のプログラムが開いたページも閲覧として記録されます。`Transition`、`Typed`、`Hidden` を確認してください。
- Chromium は Cookie の値を暗号化するため、それらの `Value` は空です。Juntei は復号しません。
- `Removed` のフォーム入力履歴の行は、Firefox の `moz_deleted_formhistory` に由来します。このテーブルには Android 版 Firefox が Sync のために書き込みます。デスクトップ版 Firefox では通常は空です。
- 信頼度が `Low` の削除済みエントリは、別のレコードの断片である可能性があります。現存する行や他の証拠と照合してください。
- 削除済みの行がないことは、何も削除されなかったことを意味しません。ゼロで埋められた領域やチェックポイントの後には、復元できるものは残りません。削除された Cookie は、現時点では復元されません。
- データベースの内容からブラウザの系統を判定します。対応するプロファイルのパスがあれば `Browser` を Chrome や Firefox などに絞り込みますが、内容と矛盾するパスでは系統を変更しません。それ以外は `Chromium-based`、`Firefox-based`、`Safari`（判定不能なら `Unknown`）です。`User` と `Profile` はパスから取得するため、コピーすると情報が失われる場合があります。
- ロールバック ジャーナル (`-journal`) は読み取りません。Internet Explorer と従来の Edge の `WebCacheV01.dat` も読み取りません。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

`-d` はファイルを名前で検索し、それぞれをテーブルによって判別します。他のプログラムの `History` ファイルはスキップされます。`-f` は名前に関係なくデータベースを読み取ります。

## 出力テーブル { #output-tables }

CSV では 4 つのファイルが作成されます。行がない場合は、ヘッダーだけのファイルになります。JSON と JSONL の各行には、`SourceFile` の後に `Table` (`History`、`Downloads`、`Cookies`、または `FormHistory`) が含まれます。

| CSV ファイル | 内容 |
|---|---|
| `Browser-History.csv` | 閲覧 (現存および削除済み)。URL が現存するどのページにもない、削除されたページ。 |
| `Browser-Downloads.csv` | ダウンロード (現存および削除済み)。 |
| `Browser-Cookies.csv` | Cookie。 |
| `Browser-FormHistory.csv` | フォーム入力 (現存および削除済み)。Firefox の削除記録。 |

## フィールド { #fields }

すべてのテーブルは、次のフィールドで始まります。

| フィールド | 解釈 |
|---|---|
| `SourceFile` | 行の読み取り元のデータベース。 |
| `User` | パス上でプロファイルが属する Windows ユーザー (`Users` の次のフォルダー)。 |
| `Browser` | 内容から判定したブラウザの系統。対応するプロファイルのパスがあれば製品名に絞り込みます。 |
| `Profile` | ブラウザのプロファイル。データベースのあるフォルダーです。 |
| `Status` | `Live`、`Deleted` (復元されたもの)、または `Removed` (Firefox の削除記録)。 |

### 閲覧履歴 { #history }

| フィールド | 解釈 |
|---|---|
| `RecordType` | `Visit`。閲覧の記録も失われている可能性がある削除されたページは `Page`。 |
| `VisitTime` | 閲覧が始まった時刻 (UTC)。 |
| `URL`, `Title` | ページとそのタイトル (最後に確認されたもの)。 |
| `Transition` | ブラウザがページに至った経緯。Chromium: コア タイプとそれに続く修飾子 (`LINK`、`TYPED`、`AUTO_BOOKMARK`、`FORM_SUBMIT`、`RELOAD` … に `CHAIN_START`、`SERVER_REDIRECT`、`FROM_ADDRESS_BAR` … が続く)。Firefox: `LINK`、`TYPED`、`BOOKMARK`、`EMBED`、`REDIRECT_PERMANENT`、`REDIRECT_TEMPORARY`、`DOWNLOAD`、`FRAMED_LINK`、`RELOAD`。 |
| `VisitDuration` | ページが前面に表示されていた秒数 (Chromium。計測されなかった場合は 0)。 |
| `VisitCount` | そのページへの閲覧の総数。 |
| `Typed`, `TypedCount` | URL が入力されたことがあるかどうかと、その回数 (Chromium)。 |
| `Hidden` | 履歴の一覧に表示されないもの: サブフレーム、リダイレクト元、エラー。 |
| `Frecency` | アドレス バーの候補表示のための、Firefox によるページのランク付け。 |
| `VisitID`, `FromVisitID` | 閲覧の行 ID と、遷移元の閲覧の行 ID (参照元のページ、リダイレクト元)。 |
| `LastVisitTime` | 削除されたページの最後の閲覧 (UTC)。 |
| `PageSource` | 削除された閲覧の URL とタイトルの取得元: `Live` (現存するページ)、`Recovered` (削除されたページ)、または `NotFound`。 |
| `Confidence`, `RecoveredFrom`, `LostColumns` | 削除済みエントリがどのように復元されたか。[削除されたエントリ](#deleted-entries)を参照してください。 |

### ダウンロード { #downloads }

| フィールド | 解釈 |
|---|---|
| `StartTime`, `EndTime` | ダウンロードの開始時刻と終了時刻 (UTC)。 |
| `URL`, `RedirectChain` | ファイルの取得元と、途中のすべての URL (Chromium。CSV では URL の間に ` -> ` が入ります)。 |
| `TargetPath`, `CurrentPath` | ファイルの保存先 (Firefox では `file://` URI) と、ダウンロード中に書き込まれていた場所。 |
| `ReceivedBytes`, `TotalBytes` | 受信したバイト数と、ファイルのサイズ。 |
| `State` | `Complete`、`InProgress`、`Cancelled`、`Interrupted`、`Paused`、`Blocked`、または `Dirty` (マルウェアとしてブロック)。 |
| `DangerType` | Chromium によるファイルの判定: 0 危険なし、1 危険なファイルの種類、2 危険な URL、3 危険なコンテンツ、4 危険の可能性あり、5 一般的でないファイル、6 ユーザーが保持、7 危険なホスト、8 望ましくない可能性あり、… |
| `InterruptReason` | 中断されたダウンロードが停止した理由 (Chromium。理由がない場合は 0)。 |
| `Referrer`, `TabURL` | 参照元のページと、ダウンロードを開始したタブに表示されていたページ。 |
| `MimeType`, `Opened` | 配信時のファイルの種類と、ブラウザから開かれたかどうか (Chromium)。 |
| `FileDeletedByUser` | ユーザーがブラウザのダウンロード一覧から、ダウンロードしたファイルをディスク上で削除したかどうか (Firefox)。 |
| `DownloadID` | ダウンロードの行 ID。 |

### Cookie { #cookies }

| フィールド | 解釈 |
|---|---|
| `Created`, `LastAccessed`, `Expires` | Cookie が設定された時刻、最後に送信された時刻、有効期限 (UTC。セッション Cookie では `Expires` は空)。 |
| `Host`, `Name`, `Path` | Cookie の対象サイト、Cookie の名前とパス。 |
| `Value` | 保存されている値。Chromium が暗号化した場合は空です。 |
| `Secure`, `HttpOnly`, `Persistent` | HTTPS でのみ送信、スクリプトから隠蔽、セッションをまたいで保持 (Chromium)。 |
| `GoogleAnalytics` | Google Analytics の Cookie (`__utma`、`__utmb`、`__utmt`、`__utmz`) をデコードしたフィールド: 訪問時刻、セッション、キャンペーンのソースと検索語。 |
| `RowID` | Cookie の行 ID。 |

### フォーム入力履歴 { #form-history }

| フィールド | 解釈 |
|---|---|
| `FirstUsed`, `LastUsed` | 値が最初と最後に入力された時刻 (UTC)。 |
| `RemovedTime` | Firefox の削除記録が示す、エントリが削除された時刻 (UTC)。 |
| `FieldName`, `Value` | フォームのフィールド (`email`、`q`、Firefox の検索バーは `searchbar-history`) と、入力された内容。 |
| `Count` | 値が入力された回数。 |
| `GUID` | Firefox におけるエントリの識別子。削除記録はこの識別子でエントリを示します。 |
| `Sources` | Firefox がエントリをどこから保存したか (これを保持するバージョンのみ。CSV では縦棒で区切られます)。 |
| `RowID` | エントリーの行 ID。復元時に失われた場合は空欄/null。実際に記録されたゼロは `0` のままです。 |

## 自動解析 { #automatic-parsing }

`auto-parse` は、テーブルを `BrowserActivity` に `Browser-History.csv`、`Browser-Downloads.csv`、`Browser-Cookies.csv`、`Browser-FormHistory.csv` として書き出し (`-t json` または `-t jsonl` の場合は 1 つの `Browser.json` または `Browser.jsonl` ファイル)、あわせて `BrowserActivity/Browser-logs.txt` を作成します。結合された `-j` タイムラインにはすべての行が含まれ、`Comp` と `Artifact` 識別子 `browser` が付きます。`-Z` は `VisitTime`、`LastVisitTime`、`StartTime`、`EndTime`、`Created`、`LastAccessed`、`Expires`、`FirstUsed`、`LastUsed`、`RemovedTime` を変換します。

## さらに詳しい情報 { #further-details }

- [Chromium — ページ遷移の種類](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — ダウンロードの危険度の種類](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — Places データベース](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — ライトアヘッドログ](https://www.sqlite.org/wal.html)
- [Plaso — ブラウザの SQLite パーサー](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
