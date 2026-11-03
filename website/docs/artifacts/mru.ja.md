# 最近使用したリスト (RunMRU、TypedPaths、WordWheelQuery、TypedURL) { #most-recently-used-lists-runmru-typedpaths-wordwheelquery-typedurls }

## リストとは何ですか { #what-the-lists-are }

Windows とそのプログラムは、ユーザーが最後に入力または開いた内容の短いリストをユーザーの `NTUSER.DAT` に保存します。

- **RunMRU:** [ファイル名を指定して実行] ダイアログに入力されたコマンド (Win+R): `cmd`、`powershell -ep bypass`、`\\server\share`、`regedit`。
- **TypedPaths:** パスがエクスプローラーのアドレス バーに入力されます。
- Explorer の検索ボックスで検索された **WordWheelQuery:** 用語。
- **TypedURLs:** アドレスが Internet Explorer のアドレス バーに入力され、Windows 8 `TypedURLsTime` 以降、それぞれが入力されたときのアドレス。
- **Paint および WordPad:** それぞれ最後に開いたファイル (`Recent File List`)。
- **ACMru (Windows XP):** 検索の種類ごとの検索アシスタントの用語。

<span class="evidence-strength">この証拠から判断できること:</span> ユーザーがこのコマンド、パス、検索語、アドレスを入力した、またはプログラムがこのファイルを開きました。入力したアドレスを `TypedURLsTime` で時刻付けできる場合を除き、各リストのキーの最終書き込み時刻が示すのは、そのリストの直近の項目だけです。

## 場所 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

`ACMru` のサブキーは検索の種類です: `5603` ファイル名とフォルダー名、`5604` ファイル内の単語またはフレーズ、`5647` コンピューターと人物、`5001` インターネット。

プロファイルごとに `NTUSER.DAT` とそのトランザクション ログを収集します。

## 分析時の注意点 { #analyst-cautions }

- **リストごとに 1 回。** リストが変更されるとキーの最終書き込み時刻も変わるため、日付は最新のエントリのみ (`IsMostRecent`、`LastUsedUTC`) になります。他のものは、日付なしで、順番にあります。 `TypedURLsTime` は、入力された各アドレス (`TypedTimeUTC`) に日付を付けます。
- **Typeed, not run.** RunMRU は、正常に実行されたかどうかに関係なく、[実行] ダイアログに入力された内容を記録します。 Windows は、各コマンドの末尾に `\1` を付けて保存します。 Juntei はそれを削除します。 Prefetch、BAM、UserAssist と関連付けて、作成イベントを処理します。
- **Lists は短いです。** それぞれは最新のエントリのみを保持し、プライバシー設定とクリーンアップ ツールによってそれらのエントリが消去されます。削除された値には `parse-registry read --recover-deleted` を使用します。
- **Terms は入力された内容です。** WordWheelQuery と ACMru は、見つかった内容ではなく、入力されたテキストを保持します。
- **Order はリストから取得されます。** RunMRU の `MRUList` と WordWheelQuery の `MRUListEx` が順序を与えます。他のものについては、値の番号が当てはまります (`url1`、`File1`、および `000` が最新です)。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

概要には、リストごとのエントリと、最後の日付のエントリ (入力されたアドレスと各リストの最新のエントリ) が新しい順にリストされます。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | ホスト、プロファイル所有者 (パスまたはシェル フォルダーから)、システムの `ProfileList` からの SID、およびハイブ。 |
| `List` | `RunMRU`、`TypedPaths`、`WordWheelQuery`、`TypedURLs`、`Paint`、`Wordpad`、または `ACMru`。 |
| `Subkey` | `ACMru` のサブキー: 検索の種類 (`5603`、`5604`、`5647`、`5001`)。 |
| `MRU-Position`, `IsMostRecent` | リスト内の場所 (0 が最新)。リストに値の名前がない場合は空です。 |
| `ValueName` | `a`, `0`, `url1`, `File1`, `000`, … |
| `Value` | コマンド (`\1` なし)、パス、検索語、アドレス、またはファイル。 |
| `TypedTimeUTC` | アドレスが入力されたとき、`TypedURLsTime` から。 |
| `LastUsedUTC` | 最新のエントリのみのキーの最終書き込み時刻。 |
| `KeyPath`, `KeyLastWrittenUTC` | キーとその最後の書き込み: すべての行ではなく、その最新のエントリの時刻。 |
| `DataRaw` | デコードされない場合の値のバイト数 (16 進数)。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | リスト名はあるがキーには付いていない値、名前が付けられていない値、名前が間違っている値、間違った型、用語の短縮、FILETIME ではない時間。 |

## 調査戦略 { #investigation-strategy }

1. ユーザーが入力したコマンド、スクリプト、ネットワーク共有、および管理ツールについては、「RunMRU」を参照してください。
2. TypedPaths および WordWheelQuery を読み取り、ユーザーが探しているフォルダーとファイル (認証情報、エクスポート、アーカイブ、リムーバブルおよびネットワークの場所) を確認します。
3. `TypedTimeUTC` を使用して、入力したアドレスをタイムラインに配置します。
4. Prefetch、BAM、UserAssist、ShellBags、LNK ファイル、ジャンプ リスト、ブラウザ履歴と関連付けます。

## さらに詳しい情報 { #further-details }

- [libyal winreg-kb: 最近使用した (MRU) キー](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
