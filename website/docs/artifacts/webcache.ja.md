# Internet Explorer と従来の Edge { #internet-explorer-and-legacy-edge }

## WebCache データベースとは { #what-the-webcache-database-is }

`WebCacheV01.dat` は、各ユーザーのプロファイル内にある Windows のインターネット機能 (WinINet) のデータベースで、ESE データベースです。Internet Explorer 10 と 11 は閲覧履歴、ダウンロード、キャッシュ、Cookie をここに保存します。従来の Edge (Edge が Chromium ベースになる前の Windows 10 の Edge) も同様です。レコードの種類ごとに、それぞれ独立したテーブルであるコンテナーがあります。

- **履歴** (`History`、および 1 日または 1 週間ごとの `MSHist01…`): 閲覧した URL。それぞれに、ページのタイトル、記録の対象となったアカウント、最後に閲覧された時刻、閲覧された回数が記録されます。エクスプローラーが開いたローカル ファイルも、`file:///` URL としてここに記録されます。
- **ダウンロード** (`iedownload`): ファイルの取得元 URL、保存先とダウンロード中に書き込まれていた場所、サイズ、ダウンロードの開始時刻、参照元のページ、MIME タイプ。
- **Cookie**: Windows 10 以降では Cookie そのもの (`CookieEntryEx` テーブル) で、ホスト、名前、値、時刻が含まれます。それより前のバージョンでは、Cookie を格納していた Cookie ファイル。
- **キャッシュ** (`Content`): ページを表示するためにダウンロードされたファイル。それぞれに、URL、ディスク上のコピーの場所、HTTP 応答ヘッダーが記録されます。WinINet を使用する他のプログラムも、ダウンロードしたものをここにキャッシュします。
- Web ストレージ (`DOMStore`)、互換性リスト (`iecompat`) など。

このデータベースはブラウザよりも長く残ります。Internet Explorer は廃止され、Chromium ベースの Edge は独自の SQLite データベースを使用します ([Web ブラウザ](browser.md)を参照) が、このデータベースは今もすべてのプロファイルに存在し、以前に記録された内容と、エクスプローラーや他のプログラムが今も記録している内容を保持しています。

<span class="evidence-strength">この証拠から判断できること:</span> ユーザーのプロファイルが、記録された時刻にその URL、ダウンロード、またはキャッシュされたファイルを記録しました。誰がキーボードの前にいたかも、キャッシュされたファイルが開かれたかどうかも、データベースからはわかりません。

## 保存場所と収集 { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

コンテナーが参照するファイル (キャッシュされたファイル、Cookie ファイル) は、それぞれ専用のフォルダーにあります。`%LOCALAPPDATA%\Microsoft\Windows\INetCache\`、`INetCookies\`、`History\`、およびアプリの場合は `%LOCALAPPDATA%\Packages\` の下です。

`WebCache` フォルダー全体を収集してください。Windows はユーザーがログオンしている間データベースを開いたままにするため、稼働中のシステムから取得したコピーは通常ダーティな状態です。最新の変更は、`V01*.log` トランザクション ログにしか存在しません。

!!! warning "トランザクション ログ"
    Juntei は ESE のトランザクション ログを再生しません。正常にシャットダウンされなかったデータベースは、概要ではダーティなままのデータベースとして数えられ、問題として報告されます。ログにしか存在しない変更は含まれません。それらを含めるには、まずフォルダーのコピーを作成し、そのコピーの中で Windows の `esentutl /r V01 /d` を実行してログを再生してください。

## 分析時の注意点 { #analyst-cautions }

- 時刻は UTC で、100 ナノ秒単位です。ただし、1 日または 1 週間の履歴コンテナーの `ModifiedTime` は例外で、システムのローカル時刻がオフセットなしで書き込まれており、`-Z` もそのままにします。元のシステムの当時のタイム ゾーンで変換してください。`AccessedTime` との差はそのタイム ゾーンのオフセットであることが多いものの、常にそうとは限りません。2 つが異なる時点を示している場合もあります。
- 履歴エントリは閲覧ではなく URL です。`AccessedTime` はその最後の閲覧、`AccessCount` は閲覧された回数です。`History` コンテナーは URL ごとに 1 つのエントリを保持します。1 日または 1 週間のコンテナーはその期間のエントリを保持し、その名前は期間の初日と最終日の翌日を示します (`MSHist012026110120261102`: 2026 年 11 月 1 日)。
- 履歴内の `file:///` URL は、エクスプローラーや他のプログラムから開かれたローカル ファイルであり、Web ページではありません。
- `Account` はエントリの記録の対象となったアカウント (`Visited: alice@…`) で、`User` はデータベースのパスが含まれるプロファイル フォルダーです。両者は異なる場合があります。プロファイル フォルダーの名前が常にアカウントの名前と同じとは限らず、また別の場所にコピーされたデータベースのパスにはプロファイル フォルダーが含まれません。
- `Package` は、エントリが誰のものかを示します。従来の Edge (`microsoft.microsoftedge_8wekyb3d8bbwe`)、拡張保護モードの Internet Explorer (`windows_ie_ac_001`)、または他のアプリのものです。空の場合は、ユーザー自身のもの (Internet Explorer、エクスプローラー、その他のプログラム) です。
- Microsoft はダウンロード レコードを文書化していません。Juntei は、開始時刻とサイズはヘッダーがレコード内に収まる場合にのみ読み取り、パスとその他の文字列は最後の 2 つが URL と Windows のパスである場合にのみ読み取ります。それ以外の場合、行にはエントリ自体の URL と、開始時刻を読み取れないときはエントリのアクセス時刻が入り、問題が報告されます。`Publisher` はブラウザが記録した名前であり、ファイルの署名を確認したものではありません。
- キャッシュ エントリは、ファイルがダウンロードされてキャッシュされたことを示します。ディスク上のコピーは、削除されている、置き換えられている、または一度も開かれていない可能性があります。
- 削除されたエントリは、現時点では復元されません。ESE は削除されたレコードを、ページをクリーンアップするまでページ内に保持することがありますが、Juntei は現存するレコードだけを読み取ります。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

`-d` は `WebCacheV01.dat` を名前で検索します。`-f` は名前に関係なくデータベースを読み取ります。

## 出力テーブル { #output-tables }

CSV では 5 つのファイルが作成されます。行がない場合は、ヘッダーだけのファイルになります。JSON と JSONL の各行には、`SourceFile` の後に `Table` (`History`、`Downloads`、`Cookies`、`Entries`、または `Containers`) が含まれます。

| CSV ファイル | 内容 |
|---|---|
| `WebCache-History.csv` | 履歴コンテナーのエントリ。 |
| `WebCache-Downloads.csv` | ダウンロード。 |
| `WebCache-Cookies.csv` | Cookie (Windows 10 以降)。 |
| `WebCache-Entries.csv` | その他すべてのコンテナーのエントリ: キャッシュされたファイル、Cookie ファイル、Web ストレージ、互換性リストなど。 |
| `WebCache-Containers.csv` | コンテナー。 |

## フィールド { #fields }

すべてのテーブルは、次のフィールドで始まります。

| フィールド | 解釈 |
|---|---|
| `SourceFile` | 行の読み取り元のデータベース。 |
| `User` | パス上でプロファイルが属する Windows ユーザー (`Users` の次のフォルダー)。 |
| `Package` | 行のデータが属するアプリ: コンテナーのディレクトリ (または Cookie テーブルのディレクトリ) の、`Packages` の下のフォルダー。ユーザー自身のデータでは空です。 |

### 履歴 { #history }

| フィールド | 解釈 |
|---|---|
| `Container` | コンテナー: `History`、または `MSHist01` の後に期間の初日と最終日の翌日が続くもの。 |
| `AccessedTime` | URL の最後の閲覧 (UTC)。1 日または 1 週間のコンテナーでは、その期間内の最後の閲覧。 |
| `ModifiedTime` | エントリが最後に変更された時刻 (UTC)。1 日または 1 週間のコンテナーでは、オフセットなしのシステムのローカル時刻。 |
| `CreationTime` | エントリが作成された時刻 (UTC)。 |
| `ExpiryTime`, `SyncTime` | エントリが履歴から外れる予定の時刻と、最後に同期された時刻 (UTC)。 |
| `Account` | エントリの記録の対象となったアカウント。 |
| `URL` | ページまたはファイル。`Visited: account@` プレフィックスは除かれます。 |
| `Title` | エントリに記録されたページのタイトル。1 日または 1 週間のコンテナーでは空です。 |
| `AccessCount`, `SyncCount` | URL が閲覧された回数と、エントリが同期された回数。 |
| `EntryType`, `Flags` | 保存されているままのエントリの種類とフラグ。 |
| `EntryID`, `ContainerID` | コンテナー内でのエントリの ID と、コンテナーの ID。 |

### ダウンロード { #downloads }

| フィールド | 解釈 |
|---|---|
| `StartTime` | ダウンロードが開始された時刻 (UTC)。 |
| `URL` | ファイルの取得元。 |
| `TargetPath`, `CurrentPath` | ファイルの保存先と、ダウンロード中に書き込まれていた場所。従来の Edge では `.partial` ファイル、Internet Explorer ではキャッシュのコピーです。 |
| `TotalBytes` | ファイルのサイズ。 |
| `Referrer` | 参照元のページ。 |
| `MimeType` | 配信時のファイルの種類 (Internet Explorer)。 |
| `Publisher` | 従来の Edge がファイルの発行元として記録した名前。 |
| `AccessedTime` | エントリ自体のアクセス時刻 (UTC)。ほとんどの場合、ダウンロードの開始時刻です。 |
| `DownloadID`, `ContainerID` | コンテナー内でのエントリの ID と、コンテナーの ID。 |

### Cookie { #cookies }

| フィールド | 解釈 |
|---|---|
| `LastModified`, `Expires` | Cookie が設定された時刻と、有効期限 (UTC)。 |
| `Host`, `Name`, `Path` | Cookie の対象サイト、Cookie の名前とパス。 |
| `Value` | 保存されている値。 |
| `Flags` | 保存されているままの Cookie のフラグ。 |
| `Directory` | データベースが Cookie テーブルに割り当てているディレクトリ。誰の Cookie かを示します (`…\INetCookies\ESE\`、従来の Edge の `…\MicrosoftEdge\Cookies\ESE\`、アプリのディレクトリ)。 |
| `CookieTable`, `EntryID` | テーブル (`CookieEntryEx_<n>`) と、その中での Cookie の ID。 |

### エントリ { #entries }

| フィールド | 解釈 |
|---|---|
| `Container` | コンテナー: `Content` (キャッシュされたファイル)、`Cookies` (Cookie ファイル)、`DOMStore` (Web ストレージ)、`iecompat`、…。 |
| `AccessedTime` | エントリが最後に使用された時刻 (UTC)。 |
| `ModifiedTime` | サーバーが示した、リソースの最終更新時刻 (UTC)。 |
| `CreationTime` | エントリが作成された時刻 (UTC)。 |
| `ExpiryTime`, `SyncTime` | エントリの有効期限と、最後にサーバーに確認された時刻 (UTC)。 |
| `URL` | 保存されている URL: キャッシュされたファイルでは URL、Cookie ファイルでは `Cookie:alice@example.com/`、そのほか `DOMStore:…`、`iecompat:…`。 |
| `Account` | Cookie ファイルのアカウント。 |
| `Filename`, `FilePath` | エントリのファイルとその場所。場所は、コンテナーのディレクトリと、エントリが指定するサブフォルダーです。 |
| `FileSize` | ファイルのサイズ。 |
| `AccessCount`, `SyncCount` | エントリが使用された回数と、サーバーに確認された回数。 |
| `EntryType`, `Flags` | 保存されているままのエントリの種類とフラグ。 |
| `RedirectURL` | URL のリダイレクト先。 |
| `ResponseHeaders`, `RequestHeaders` | HTTP 応答ヘッダー (`HTTP/1.1 200 OK`、`Content-Type`、`Server`、…) と要求ヘッダー。改行も含め、保存されているままです。 |
| `EntryID`, `ContainerID` | コンテナー内でのエントリの ID と、コンテナーの ID。 |

### コンテナー { #containers }

| フィールド | 解釈 |
|---|---|
| `ContainerID`, `Name` | コンテナーの ID と名前。 |
| `Directory` | コンテナーのファイルがあるディスク上の場所。 |
| `SecureDirectories` | ファイルが置かれている `Directory` のサブフォルダー (CSV ではコンマ区切り)。 |
| `PartitionID` | ユーザー自身のコンテナーは `M`、Internet Explorer の保護モードのコンテナー (`Low` フォルダー) は `L`、アプリのコンテナーはそのアプリ コンテナーの SID。 |
| `LastAccessTime`, `LastScavengeTime` | コンテナーが最後に使用された時刻と、最後にクリーンアップされた時刻 (UTC)。 |

## 自動解析 { #automatic-parsing }

`auto-parse` は、テーブルを `BrowserActivity` に `WebCache-History.csv`、`WebCache-Downloads.csv`、`WebCache-Cookies.csv`、`WebCache-Entries.csv`、`WebCache-Containers.csv` として書き出し (`-t json` または `-t jsonl` の場合は 1 つの `WebCache.json` または `WebCache.jsonl` ファイル)、あわせて `BrowserActivity/WebCache-logs.txt` を作成します。結合された `-j` タイムラインにはすべての行が含まれ、`Comp` と `Artifact` 識別子 `webcache` が付きます。`-Z` は、1 日または 1 週間の履歴コンテナーのローカル時刻である `ModifiedTime` を除き、すべての時刻、すなわち `Time` で終わるフィールド、`LastModified`、`Expires` を変換します。

## さらに詳しい情報 { #further-details }

- [Microsoft — WinINet について](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE データベースのファイル形式](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — MSIE WebCache パーサー](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
