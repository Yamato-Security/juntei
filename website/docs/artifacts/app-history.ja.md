# アプリケーション履歴 { #application-history }

## ツールが後に残すもの { #what-tools-leave-behind }

多くのサードパーティ ツールは、ユーザーの `NTUSER.DAT` に設定と履歴を保存します。 Sysinternals ツール、アーカイバー、クラウド同期クライアント、リモート アクセス ツールなど、攻撃者がデータを使用する、またはデータを取得する前にステージングするために使用するツールがいくつかあります。 Juntei は次のように読み取ります。

| アプリケーション | `HKCU\Software` の下のキー | それが示すもの |
|---|---|---|
| シスインターナル | `Sysinternals\<tool>` | ユーザーがライセンスを受け入れた各ツール (`EulaAccepted`): PsExec、ProcDump、Sysmon など |
| WinRAR | `WinRAR\ArcHistory`, `DialogEditHistory\ArcName`, `DialogEditHistory\ExtrPath` | アーカイブがオープンしました。ダイアログに入力されたアーカイブ名と抽出フォルダー |
| 7-Zip | `7-Zip\Compression\ArcHistory`, `Extraction\PathHistory`, `FM\FolderHistory`, `FM\CopyHistory`, `FM\PanelPath0/1` | アーカイブが作成されました。抽出フォルダー。ファイルマネージャーで参照およびコピーされたフォルダー |
| WinZip | `Nico Mak Computing\WinZip\filemenu`, `extract`, `mru\archives` | 「ファイル」メニューのアーカイブ、抽出フォルダー、最近のアーカイブ |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | 各アカウントの電子メール アドレス、ローカル フォルダーおよび名前。同期されたライブラリ (`ScopeIdToMountPointPathCache`) とフォルダー (`Tenants\<tenant>`) |
| Adobe Acrobat および Reader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`, `cRecentFolders` | 最後に開いた PDF ファイルとフォルダー (ファイル名、サイズ、日付) |
| TeamViewer | `TeamViewer` (および古いバージョンの `Version<N>`) | すべての値: アカウントとユーザー名、ファイル転送開始フォルダー (`<partner ID>?<local>|<remote>`) など |
| CCクリーナー | `Piriform\CCleaner` | すべての値: クリーンに設定されているもの、`UpdateKey`、… |

<span class="evidence-strength">この証拠から判断できること:</span> このユーザーがツールを実行した、またはライセンスに同意したこと、ツールがこれらのアーカイブやフォルダを開いた・作成した・展開したこと、あるいはこのユーザー向けにクラウドアカウントが設定されたことを示します。多くの履歴には項目ごとの時刻がなく、リストのキーの最終書き込み時刻で日時を判断できるのは、直近の項目だけです。

## 場所 { #location }

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

すべてのユーザーの `NTUSER.DAT` とそのトランザクション ログを収集し、ユーザーの SID としてシステムの `SOFTWARE` を収集します。

## 分析時の注意点 { #analyst-cautions }

- **Sysinternals EULA accept.** `FirstUsedUTC` は、唯一の値が `1` に設定された読み取り可能な `EulaAccepted` である場合、キーの最終書き込み時刻からの初回使用推定値です。拒否または破損した EULA はこの見積もりを受け取りません。レジストリ設定を事前に構成、コピー、または編集することもでき、他のアーティファクトとの実行とその時間を裏付けます。設定を保持するツール (プロセス エクスプローラー、自動実行など) は後でキーを書き換えます。`FirstUsedUTC` は空であり、`KeyLastWrittenUTC` は最後の設定変更にすぎません。 `-accepteula` で実行するか、別のユーザー (PsExec のリモート側は SYSTEM として実行) として実行されるツールは、そのユーザーのキーを書き込みます。
- **Positions (times ではありません)。** リスト エントリは順序付けされます (`MruPosition` 0 が最新です)。ただし、リスト キーの最後の書き込み日付のみが最新のものになります。古いエントリは、不明な時期に以前に使用されていました。
- **型付けされた名前.** WinRAR の `ArcName` は、アーカイブを作成するときに入力されるアーカイブ名です。多くの場合、フォルダーのないファイル名です。
- **Adobe パス** は、Adobe が書き込んだまま保持されます (`/C/Users/alice/a.pdf` は `C:\Users\alice\a.pdf`)。その `sDate` (`Details` 内) は、書かれたままの PDF 日付文字列です。
- **CCleaner の `UpdateKey`** はゾーンなしの現地時間です。書かれたままに保管されています。
- **TeamViewer.** 接続ログはファイル (`Connections_incoming.txt`、`TeamViewer*_Logfile.log`) であり、レジストリ値ではありません。これらも収集してください。
- **Absence.** ユーザーとツールはこれらの履歴をクリアできます (CCleaner 自体もクリアできます)。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.App == "Sysinternals" and .FirstUsed != null)'
```

この概要には、最新の Sysinternals の初回使用推定値、各リストで最後に使用されたアーカイブとフォルダー、および OneDrive アカウントがリストされます。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `SourcePath` | ホストと `NTUSER.DAT`。 |
| `User`, `UserSID` | プロファイル フォルダーの名前と、システムの `ProfileList` からの SID。 |
| `App` | `Sysinternals`, `WinRAR`, `7-Zip`, `WinZip`, `OneDrive`, `Adobe Acrobat Reader`, `Adobe Acrobat`, `TeamViewer`, `CCleaner`. |
| `Item` | Sysinternals ツール、リスト (`ArcHistory`、`ExtrPath`、`FolderHistory`、`filemenu`、`cRecentFiles` など)、OneDrive アカウントまたはその同期フォルダーのキー、または値の名前。 |
| `MRU-Position` | リスト内のエントリの位置、0 が最新です。 |
| `Path` | アーカイブ、フォルダー、またはドキュメント。 |
| `Value` | `EulaAccepted`、OneDrive アカウントの電子メール アドレス、または同期されたライブラリの ID、TeamViewer、または CCleaner の値。 |
| `FirstUsedUTC` | EULA の受け入れからの初回使用の推定 (上記の注意を参照)。 |
| `Details` | OneDrive アカウントの `DisplayName`、`UserCID`、`ConfiguredTenantId`、…; Adobe ドキュメントの `Version`、`tFileName`、`uFileSize`、`uPageCount`、`sDate`。 |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName` | レコードの出所と、そのキーが最後に変更されたとき。 |
| `DataRaw` | デコードされない場合の値のバイト数 (16 進数)。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | デコードまたは読み取りができなかったデータ。 |

## 調査戦略 { #investigation-strategy }

1. 一般のユーザーが実行しない Sysinternals ツール (PsExec、ProcDump、PsLoggedOn、SDelete) とその EULA タイムスタンプを探します。
2. インシデントに関するアーカイブ名と抽出フォルダー (データがシステムから流出する前に、`ProgramData`、`Temp`、またはユーザーのフォルダーに作成されたアーカイブ) を確認します。
3. 組織のものではない OneDrive アカウント、TeamViewer アカウント名とファイル転送フォルダーを確認してください。
4. Prefetch、Amcache、BAM、および UserAssist (実行中のツール)、`$MFT` および USN ジャーナル (アーカイブのファイル)、LNK ファイルとジャンプ リスト (開いているドキュメント)、およびイベント ログと関連付けます。

## さらに詳しい情報 { #further-details }

- [Microsoft: システム内部](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
