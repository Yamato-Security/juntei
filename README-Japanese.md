<p align="center">
  <img src="./art/Juntei_Goddess%20of%20Digital%20Forensics_small.png" alt="デジタルフォレンジックの女神 Juntei" width="65%">
</p>

<h1 align="center">Juntei</h1>

<p align="center">
  <a href="README.md">English</a> | <strong>日本語</strong>
</p>

Juntei は、[Yamato Security](https://github.com/Yamato-Security) が開発する Windows の
フォレンジックアーティファクト解析ツールです。[Hayabusa](https://github.com/Yamato-Security/hayabusa)
の姉妹ツールとして、イベントログ以外のアーティファクトを解析します。

Juntei という名前は、深い慈悲の心を持つ仏教の尊格「准胝観音」に由来します。
英語では「Pure Avalokitesvara」とも呼ばれます。その慈悲の心で調査を支え、たくさんの腕を使って、
必要な Windows DFIR アーティファクトを解析してくれます。

## [Juntei ドキュメントはこちら →](https://yamato-security.github.io/juntei/ja/)

**Juntei のドキュメントでは、インストールと使い方、証拠の収集、対応する各アーティファクトの解析方法、
出力フィールドの解釈、フォレンジック調査で陥りやすい誤りについて解説しています。**

## 対応アーティファクト

Juntei は現在、広く利用されている Windows フォレンジックアーティファクト 42 種類に対応しており、今後も対応範囲を拡大していきます。

`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>` を実行すると、収集済みの複数の Windows システムを
自動検出し、対応するすべてのアーティファクトを並列に解析します。結果はコンピュータと証拠のカテゴリ別に
整理され、各結果の隣に `<artifact>-logs.txt` が保存されます。`-j <FILE>` を追加すると、CSV や JSON の
結果を残したまま、コンピュータ名を付与した SIEM 向けの JSONL ファイルも作成できます。

個別に解析する場合は、`parse-ese`、`parse-file`、`parse-ntfs`、`parse-registry`、`parse-sqlite` を使用します。
例: `juntei parse-file prefetch -d <EVIDENCE> -o prefetch.csv`。

`juntei auto-parse --vhdx -d <IMAGE_OR_DIR> -o <OUTPUT>` は、**Linux、macOS、Windows** で
OS へのマウントや管理者権限なしに VHDX を解析します。パーサー用の専用一時コピーを作成するため、
十分な一時ディスク容量が必要です。`juntei mount <IMAGE> <MOUNTPOINT>` は、Linux の FUSE を使って
他のプログラムにイメージを読み取り専用で公開する用途で引き続き利用できます。
一時コピーは既定で出力先のファイルシステムに作成されます。コピーと結果の両方に必要な空き容量を確保し、
別のディスクを使う場合は `--temp-dir <DIR>` を指定してください。Ctrl+C はパーサーを停止して専用コピーを削除します。
どちらも [Yamato Security/ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) を使用します。

[`juntei persistence-hunter`](https://yamato-security.github.io/juntei/ja/commands/persistence-hunter/)
では、レジストリハイブ、スケジュールタスク、スタートアップファイル、WMI、BITS を横断して永続化の痕跡を調べます。
[PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper) を Rust に移植したもので、
131 種類のチェック、許可・ブロックのプロファイル設定、根拠となる証拠、CSV・JSON・JSONL 出力に対応します。
検出される候補は永続化に関する設定を示すもので、それだけで侵害や実行が証明されるわけではありません。
各チェックのカバレッジを確認すると、未収集の証拠や、一部しか読み取れなかった証拠を把握できます。

### プロセスの実行

| アーティファクト | 種類 |
| --- | --- |
| **BAM / DAM** (`SYSTEM`) — ユーザーごとの各プログラムの最終実行時刻。SAM と ProfileList からユーザー名を補完します。 | Registry |
| **アプリケーション互換性** (`SOFTWARE`、`NTUSER.DAT`) — 互換性レイヤー、Program Compatibility Assistant が実行を認識したプログラム、CIT のプログラム・モジュール使用情報、カスタム互換性データベース。 | Registry |
| **Prefetch** (`*.pf`) — プログラムの実行回数、直近の実行時刻、参照ファイル。 | File |
| **SRUM** (`SRUDB.dat`) — アプリケーションとユーザーのリソース使用情報。ネットワーク送受信バイト数などを含みます。 | ESE |
| **UserAssist** (`NTUSER.DAT`) — ユーザーが Explorer から起動したプログラムとショートカット。実行回数、フォーカス時間、最終実行時刻を含みます。 | Registry |
| **FeatureUsage とピン留め** (`NTUSER.DAT`) — アプリの切り替え・起動・クリックの回数、最近使ったアプリとファイル、タスクバーのピン留め、スタートメニューのプログラム。 | Registry |
| **MuiCache** (`UsrClass.dat`) — ユーザーのシェルプロファイルにキャッシュされたプログラムのパス、説明、会社名。 | Registry |

### ファイルシステムとファイルの存在

| アーティファクト | 種類 |
| --- | --- |
| **Master File Table** (`$MFT`) — ファイルとディレクトリのレコード、パス、タイムスタンプ、代替データストリーム、削除済みエントリ。 | NTFS |
| **トランザクションログ** (`$LogFile`) — 以前のファイル名、タイムスタンプ、ディレクトリエントリが残ることのある Redo / Undo メタデータ。 | NTFS |
| **USN ジャーナル** (`$UsnJrnl:$J`) — ファイルの作成、変更、名前変更、削除に関する記録。 | NTFS |
| **ディレクトリインデックス** (`$I30`) — ディレクトリエントリ、およびインデックススラックに残る可能性がある削除前・名前変更前のファイル名。 | NTFS |
| **セキュリティ記述子** (`$Secure:$SDS`) — `$MFT` のセキュリティ ID に対応するファイルの所有者、DACL、SACL。可読形式と SDDL で出力します。 | NTFS |
| **ブートセクタ** (`$Boot`) — ボリュームシリアル番号（LNK と Prefetch に記録されるもの）、ディスク構成、`$MFT` の位置、ホスト間で比較できるブートコードのハッシュ。 | NTFS |
| **Amcache** (`Amcache.hve`) — 実行可能ファイルのパス、ハッシュ、発行元メタデータ、デバイスのインベントリ。エントリが存在するだけでは実行を証明できません。 | Registry |
| **ShimCache / AppCompatCache** (`SYSTEM` ハイブ) — キャッシュされたアプリケーションのパス、タイムスタンプ、互換性フラグ。エントリが存在するだけでは実行を証明できません。 | Registry |

### ファイルとフォルダのオープン

| アーティファクト | 種類 |
| --- | --- |
| **ショートカット** (`*.lnk`) — リンク先のパス、引数、タイムスタンプ、ボリューム、ネットワーク共有、作成元コンピュータの情報。 | File |
| **Microsoft Office** (`NTUSER.DAT` の `Software\Microsoft\Office`) — 各アプリが開いた文書・フォルダとその時刻、ユーザーがマクロを許可した文書（Trusted Documents）、マクロ設定と信頼できる場所。 | Registry |
| **Jump Lists** (`*.automaticDestinations-ms`、`*.customDestinations-ms`) — アプリごとの最近使った項目とピン留め項目、使用状況メタデータ、埋め込まれたショートカット。 | File |
| **MRU リスト** (`NTUSER.DAT` の `RunMRU`、`TypedPaths`、`WordWheelQuery`、`TypedURLs` など) — 「ファイル名を指定して実行」のコマンド、Explorer に入力したパスと検索、Internet Explorer に入力した URL と時刻、ペイントとワードパッドの最近使ったファイル。 | Registry |
| **最近使った文書** (`NTUSER.DAT` の `RecentDocs`) — 最近アクセスしたファイルとフォルダ。直近に使った順に記録されます。 | Registry |
| **ファイルを開く・名前を付けて保存ダイアログ** (`NTUSER.DAT` の `ComDlg32`) — 共通ダイアログで開いた・保存したファイルを拡張子別に記録し、使用したプログラムとフォルダも示します。 | Registry |
| **ShellBags** (`UsrClass.dat`、`NTUSER.DAT` の `BagMRU`) — ユーザーが Explorer で開いたフォルダのフルパス。リムーバブルドライブ、ネットワーク共有、ポータブルデバイス、ZIP、FTP を含みます。 | Registry |

### 外部デバイス

| アーティファクト | 種類 |
| --- | --- |
| **USB デバイス** (`SYSTEM`、`SOFTWARE`、`NTUSER.DAT`) — USB ストレージ、USB・ポータブルデバイスのシリアル番号とインストール・接続・取り外し時刻、ボリュームのドライブ文字・名前・シリアル番号、マウントしたユーザー。 | Registry |

### レジストリ

| アーティファクト | 種類 |
| --- | --- |
| **レジストリハイブ** (`SYSTEM`、`SOFTWARE`、`NTUSER.DAT` など) — キー、値、設定、トランザクションログの再生、復元可能な削除済みレコード。 | Registry |

### 自動起動と永続化

| アーティファクト | 種類 |
| --- | --- |
| **サービス** (`SYSTEM` ハイブ) — サービスとドライバのパス、自動起動設定、実行アカウント、障害時の動作。 | Registry |
| **スケジュールタスク** (`SOFTWARE` の `TaskCache`) — キャッシュされたタスク名、実行アクション、実行メタデータ、隠されたタスクの兆候。 | Registry |
| **レジストリの自動起動設定** (`SOFTWARE`、`SYSTEM`、`NTUSER.DAT`、`UsrClass.dat`) — Run キー、Winlogon、Image File Execution Options、AppInit / AppCert DLL、LSA パッケージ、COM ハイジャックなどの自動起動箇所と、対応する ATT&CK テクニック。 | Registry |
| **ブート構成** (`BCD`) — ブートエントリと設定。回復機能の無効化、起動エラーの無視、テスト署名、整合性チェックの無効化、セーフモードなど。 | Registry |

### アカウントの使用

| アーティファクト | 種類 |
| --- | --- |
| **ローカルアカウント** (`SAM`) — ローカルユーザー名、RID、アカウントフラグ、記録されたログオンとパスワードのメタデータ。 | Registry |
| **LSA シークレット** (`SECURITY`) — LSA が保持するシークレット（自動ログオン、サービスアカウント、コンピュータアカウントのパスワードなど）の種類と設定時刻、使用中のキャッシュ済みドメインログオン。名前と時刻のみを扱い、復号は行いません。 | Registry |

### アプリケーション

| アーティファクト | 種類 |
| --- | --- |
| **インストール済みプログラム** (`SOFTWARE`、`NTUSER.DAT`) — コンピュータ全体・ユーザー別のインストール情報、Windows Installer 製品、App Paths、WSL ディストリビューション、パッケージアプリ。インストール日とキーの時刻を含みます。 | Registry |

### 横展開

| アーティファクト | 種類 |
| --- | --- |
| **リモートアクセス（ユーザー側）** (`NTUSER.DAT`) — ユーザーが Remote Desktop、PuTTY、WinSCP で接続したコンピュータ、使用アカウント、承認した SSH ホスト鍵とフィンガープリント、割り当てたドライブと共有。 | Registry |
| **ユーザーアクセスログ (UAL)** (`Current.mdb`、`{GUID}.mdb`、`SystemIdentity.mdb`) — Windows Server の役割（ファイル共有、Active Directory、DHCP、IIS など）を使用したクライアントのアドレスとアカウント、日ごとの要求回数。 | ESE |

### システム情報

| アーティファクト | 種類 |
| --- | --- |
| **システム情報** (`SYSTEM`、`SOFTWARE`、`SAM`、`SECURITY`) — コンピュータ名・ドメイン名、Windows のバージョンとインストール日、タイムゾーンと記録された現地時刻、最終シャットダウン、最後のユーザー、プロファイル、SID。 | Registry |

### アプリケーション

| アーティファクト | 種類 |
| --- | --- |
| **アプリケーション履歴** (`NTUSER.DAT`) — ユーザーが実行した Sysinternals ツール、WinRAR・7-Zip・WinZip のアーカイブとフォルダの履歴、OneDrive アカウント、Adobe の最近使ったファイル、TeamViewer と CCleaner の値。 | Registry |

### システム情報

| アーティファクト | 種類 |
| --- | --- |
| **監査ポリシー** (`SECURITY` の `PolAdtEv`) — セキュリティイベントの記録設定をサブカテゴリ別に示します。記録がないことを判断材料にできない場合の把握に役立ちます。 | Registry |

### ブラウザの活動

| アーティファクト | 種類 |
| --- | --- |
| **Web ブラウザ** (`History`、`Cookies`、`Web Data`、`places.sqlite`、`cookies.sqlite`、`formhistory.sqlite`) — Chrome、Edge、Brave、Opera、Vivaldi、Firefox など Chromium 系・Firefox 系ブラウザの閲覧履歴、ダウンロード、Cookie、フォーム入力履歴。ライトアヘッドログ（`-wal`）も含めて読み、削除された閲覧履歴、ページ、ダウンロード、フォーム入力を復元します。 | SQLite |
| **Internet Explorer と従来の Edge** (`WebCacheV01.dat`) — Internet Explorer 10・11、従来の Edge、Windows のインターネット機能 (WinINet) を使うその他のプログラムの閲覧履歴（ページタイトル付き。エクスプローラーで開いたローカルファイルを含む）、ダウンロード、Cookie、キャッシュファイル。キャッシュファイルごとにディスク上のパスと HTTP ヘッダーも示します。 | ESE |

### コミュニケーション

| アーティファクト | 種類 |
| --- | --- |
| **Outlook** (`*.pst`、`*.ost`、`*.msg`、`*.oft`) — PST・OST ストア（Outlook 2013 以降の OST を含む）のフォルダーとアイテム、保存されたメッセージとテンプレート。差出人、受信者、日時、添付ファイル（SHA-256、抽出も可能）付きのメール、添付ファイルに埋め込まれたアイテム、連絡先、予定表アイテム、タスク、フォルダーの非表示アイテム、保存されているすべてのプロパティ。 | File |

### ネットワークの活動

| アーティファクト | 種類 |
| --- | --- |
| **ネットワーク** (`SOFTWARE`、`SYSTEM`) — 接続したネットワークと UTC の初回・最終接続時刻、ゲートウェイの MAC アドレス、DNS サフィックス、インターフェースと DHCP リース、共有、ポートプロキシ、永続ルート。 | Registry |

### セキュリティ製品

| アーティファクト | 種類 |
| --- | --- |
| **Microsoft Defender の隔離データ** (`Quarantine\Entries`、`ResourceData`、`Resources`) — 脅威の記録、元のパス、ハッシュ、ダウンロード元、復元可能な隔離コンテンツ。 | File |
| **セキュリティ設定** (`SOFTWARE`、`SYSTEM`、`NTUSER.DAT`) — Defender と除外設定、UAC、WDigest・LSA の保護、RDP、ファイアウォール、SMBv1、PowerShell ログ、AMSI、イベントログ設定。既定の保護を弱める設定にフラグを付けます。 | Registry |

## ライセンス

Juntei は [Reciprocal Public License 1.5](LICENSE)（RPL-1.5）で公開されており、著作権は
Shirofune Security Co., Ltd. が保有しています。
使用するクレート、サードパーティのコードとデータのライセンスは [`THIRD_PARTY_NOTICES`](THIRD_PARTY_NOTICES/) を、
テストファイルのライセンスは [`test_files`](test_files/) 内の各ファイルに付属する文書を参照してください。
