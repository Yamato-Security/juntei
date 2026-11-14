# 自動コレクション解析 { #automatic-collection-parsing }

`auto-parse` は、複数のコンピューター Windows トリアージ コレクションを構造化分析ディレクトリに変換する最も簡単な方法です。証拠のルートと新しい出力ディレクトリを指定します。収集されたシステムを検出し、そのコンピュータ名を識別し、互換性のあるすべての Juntei パーサーを並行して実行します。

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

デフォルトの出力は CSV です。 CSV の結果を維持しながら、`jq` または SIEM インポート用の JSONL タイムラインを追加します。

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "出力ツリーと証拠ツリーを分離しておきます。"
    Juntei は、`<EVIDENCE>` 内にある出力ディレクトリ、または `<EVIDENCE>` を含む出力ディレクトリを拒否します。これにより、生成されたデータが証拠として再発見されるのを防ぎ、`--clobber` がソース コレクション内のファイルにアクセスするのを防ぎます。

## VHDX ディスクイメージ { #vhdx-disk-images }

`--vhdx` を指定すると、`auto-parse` は [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) を使用して
**Linux、macOS、Windows** で VHDX をネイティブに読み取ります。OS へのマウント、FUSE ドライバー、管理者権限は不要です。

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

各パーサーはファイルパスを必要とするため、リーダーが公開する NTFS ボリュームから、パーサーが読み取るファイル（レジストリハイブとそのトランザクションログ、NTFS メタデータファイル、データベース、Prefetch、ショートカット、Jump List、Outlook ファイル、ごみ箱のインデックス ファイル、Defender の検疫フォルダー）を専用の一時作業領域にコピーしてから通常の解析を行います。
それ以外のファイルは、元の更新時刻を持つ空のプレースホルダーになるため、ディレクトリとパスはディスク上と同じに見えます。
persistence-hunter は、永続化エントリが参照する実行ファイルなど、必要になったプレースホルダーの内容だけをイメージから読み取ります。
**出力結果に加えて、コピーするファイルを保存する一時ディスク容量が必要です。**
読み取りバッファーは固定サイズです。ホストのファイルシステムが対応している場合、ゼロ領域はスパース領域として書き込みます。
元のファイル名、ディレクトリ構造、ハードリンク、ファイル更新時刻を保持します。
LNK と Jump List のソース作成・更新・アクセス時刻は、一時ファイルではなく元の NTFS メタデータのスナップショットから取得します。
一時ディレクトリは、パーサーエラーが報告された場合を含めて正常な終了処理で削除されますが、強制終了時には残る場合があります。
元のイメージは読み取り専用で開き、未反映の VHDX ログもメモリ上でのみ適用します。

ホスト名の検出、ホスト別の出力、CSV/JSON/JSONL、`-j` は通常の証拠ディレクトリと同じ解析処理を使用します。
`manifest.json` の `Image` は元のイメージを、`EvidenceRoot` はイメージ内の相対パスを示します。
読み取れないイメージ、ボリューム、ファイル、安全に表現できないファイル名は、`ImageErrors` と
`manifest.csv` の失敗行に記録します。読み取れる証拠の解析は継続し、不完全な入力があれば終了コードは 1 になります。
危険な名前や曖昧な名前は勝手に変更せず、エラーとして報告します。名前付き NTFS ストリームを追加ファイルとして抽出する機能はありません。
トリアージイメージ内に通常のファイルとして収集済みのアーティファクトは利用できます。

他のプログラムにファイルシステムを公開する [`mount` コマンド](mount.md) は、引き続き Linux の FUSE を必要とします。
`auto-parse --vhdx` はこのコマンドを使用しません。

ステージングは既定で出力先のファイルシステム上の専用ディレクトリを使用します。 `--temp-dir` で別ディスク上の既存のディレクトリを指定できます。元のメタデータは出力側に別途保持するため、ステージング先が満杯でも、読み取れたファイルの解析を続行できます。コピーと結果の両方に必要な空き容量を確保してください。出力先自体が満杯の場合は結果を保存できません。Ctrl+C はパーサープロセスを停止し、専用作業領域を削除して終了コード 130 を返します。SIGKILL や電源断ではコピーが残る場合があります。イメージ検索では読み取れないサブディレクトリをスキップしますが、入力ルートが読めない場合はエラーです。

ステージング先は出力ディレクトリの外に指定してください。Linux と macOS では、SIGTERM と SIGHUP でもパーサープロセスを停止し、作業領域を削除します。

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## E01 と Ex01 ディスクイメージ { #e01-and-ex01-disk-images }

`--e01` は、物理 EWF1（`.E01`）および EWF2（`.Ex01`）イメージを **Linux、macOS、Windows** でネイティブに読み取ります。FUSE、OS へのマウント、管理者権限は不要です。[ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) と、上記と同じ NTFS 解析処理を使用します。

分割イメージは最初のセグメントを指定します。`.E02` などの関連ファイルを元の名前のまま同じディレクトリに置いてください。ディスク全体（MBR/GPT）と単一の NTFS ボリュームに対応します。両形式が混在するディレクトリには `--vhdx --e01` を使用します。

一時ディスク容量と元のメタデータに関する要件も同じです。E01 は通常ディスク全体を収録しており、すべてのイメージを解析開始前にステージングしますが、コピーするのはパーサーが読み取るファイルだけで、通常は使用容量のごく一部です。イメージは読み取り専用です。取得時の欠損や、開始時・読み取り中に見つかった破損は `ImageErrors` と `manifest.csv` に記録され、読み取れる結果を保持して終了コード 1 を返します。検査対象はアクセスしたデータであり、全チャンクやメディア全体のハッシュではありません。論理形式の `.L01`、`.Lx01` は拒否します。パスワード指定には対応しません。

別機能の `mount` コマンドには引き続き Linux FUSE が必要ですが、ネイティブ自動解析には不要です。

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## 出力タイムゾーン { #output-time-zone }

解析されたタイムスタンプは、アナリスト コンピュータのローカル タイム ゾーンに関係なく、デフォルトで UTC になります。 UTC は、`2026-10-20T08:46:38.2001668Z` 形式の Windows イベント ログを使用します。ISO 8601 日付/時刻、日付と時刻の間の `T`、および UTC の `Z`。使用可能な小数桁は保持されます。`manifest.json` の `GeneratedUTC` もこの形式を使用します。コンソールの `Start time` と `End time` は、出力タイム ゾーンに関係なく、アナリスト コンピュータのローカル時刻 (例: `2026-11-04 09:15:02`) です。

`-Z, --timezone` で固定ディスプレイ オフセットを選択します。

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

オフセットは、符号付きの 2 桁の `HH:MM` 値 (時間 `00–23`、分 `00–59`) です。 `JST` などの名前、`+9` などの短縮値、および無効な範囲では、構文例でエラーが発生します。オプションを省略するか、UTC には `--timezone "+00:00"` を使用します。固定オフセットには夏時間ルールは適用されません。

このオプションは、アーティファクトごとの CSV、JSON、および JSONL の結果と、追加の `-j` JSONL エクスポートの解析されたタイムスタンプに適用されます。たとえば、`2026-10-20T08:46:38.2001668Z` は `2026-10-20T17:46:38.2001668+09:00` になります。どちらも同じ瞬間を表しています。

フィールド名は、`UTC` で終わる既存の名前を含め、安定したままになります。変換された値には、選択されたオフセットが明示的に伝えられます。両方のマニフェストの `OutputTimeZone` は要求された表示オフセットを記録しますが、`GeneratedUTC` は UTC 実行メタデータのままです。パーサーごとのログには、パーサーのネイティブ UTC 概要が保持されます。

生の FILETIME/エポック番号、レジストリ値ディクショナリ、ソース タイムゾーン構成、テキスト メタデータ、日付のみの値、および既知のオフセットのないローカル クロック値が保存されます。 Juntei は、不明な現地時間に `Z` を追加しません。 FAT から派生したシェル項目クロック (ShellBags/OpenSaveMRU および LNK/ジャンプ リスト `TargetItems` の `Item*UTC`) には記録されたオフセットがありません。従来の UTC ラベルまたはレンダリングされた `Z` サフィックスは UTC であることを証明しないため、オフセット オプションによりそれらが保持されます。フリーテキストの概要を含む CSV 列には、記録されたテキストが保持されます。 JSON/JSONL は、ネストされた解析されたタイムスタンプを構造化フィールドとして保存します。

UTC は、既存のパーサー出力を直接使用します。明示的なゼロ以外のオフセットにより、各パーサー ワーカー内にストリーミング フォーマット パスが追加されます。証拠を変更せずに表示値を変更します。

オフセットによってタイムスタンプが `0000–9999` 年を超えて移動する場合、その値は元の明示的なオフセットを維持します。小数点以下の桁が 9 桁を超える値も、四捨五入されずに変更されません。 Juntei は、パーサーの `*-logs.txt` ファイルに警告を記録し、他のタイムスタンプを変換します。不正な形式の CSV/JSON または出力エラーでもタスクは失敗し、影響を受ける元の結果ファイルは変更されないままになります。

## それが発見するもの { #what-it-discovers }

Juntei は、`System32`、`Prefetch`、または `AppCompat` ディレクトリを含む `Windows` という名前のディレクトリを大文字と小文字を区別せずに再帰的に検索します。後者の 2 つを認識すると、Prefetch または Amcache のみを含むターゲット コレクションもサポートされます。これには、`AppData\Roaming\Microsoft\Windows` などのユーザー プロファイル ディレクトリは除外されます。 `C:\Windows.old\Windows` などのネストされたレガシー インストールは、重複コンピューターになるのではなく、外側の収集されたボリュームに割り当てられます。デフォルトの最大深さは 10 です。これは、次のような一般的な Velociraptor パスをカバーします。

```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

コレクションがより深くネストされている場合にのみ、制限を変更します。

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

検出されたシステムごとに、コンピューター名が次の順序で選択されます。

1. 最も近い祖先 `client_info.json` 内の `hostname` (その値はマニフェスト内で `FQDN` として保持されます)。
2. 収集された `SYSTEM` レジストリ ハイブ内の `ComputerName`。
3. 最も近い有用なコレクション ディレクトリ名。ドライブと Velociraptor `uploads/auto` スキャフォールディングがスキップされ、`_F.…` などのサフィックスが削除されます。

名前はディレクトリ名として安全に使用できるようになります。重複する名前には、出力ディレクトリと `Comp` 値の両方に数値接尾辞が付けられるため、JSON 内で個別のシステムを区別できるようになります。マニフェストでは、別の `Hostname` フィールドで検出されたホスト名を正確に保持します。

システムをリストした後、`auto-parse` は `Parser tasks` のすぐ上に `Timezone:` の 1 行を出力します。選択された出力タイムゾーンが表示されます。デフォルトでは `UTC`、`--timezone "+09:00"` が使用されている場合は `UTC+09:00` などの値が表示されます。ラベルは緑色で、値には他の概要値と同様に、端末のデフォルトのテキスト色が使用されます。 `-K, --no-color` はラインをプレーンに保ちます。収集されたシステムのタイムゾーン構成は、[システム情報](../artifacts/system-info.md)の結果で確認できます。

## 実行されるパーサー { #parsers-it-runs }

1 つのパーサーが失敗しても、他のパーサーは停止しません。 `auto-parse` は、検出されたすべてのコンピューターに対してこれらのジョブを実行します。ジョブは、手動解析と同じソース グループ化されたコマンド (`parse-ntfs mft`、`parse-registry amcache`、`parse-ese srum`、`parse-file prefetch` など) を使用します。

- Amcache
- `$MFT`
- `$LogFile` NTFS やり直し/元に戻すレコード
- Prefetch レコードと別の Prefetch 実行タイムライン
- USN ジャーナル
- `$I30` ディレクトリ インデックス
- `$Boot` ブート セクターと `$Secure:$SDS` セキュリティ記述子
- SRUM
- ユーザーアクセスログ (UAL)
- Web ブラウザの閲覧履歴、ダウンロード、Cookie、フォーム入力履歴 (Chromium 系および Firefox 系ブラウザ)
- Internet Explorer と従来の Edge の WebCache: 閲覧履歴、ダウンロード、Cookie、キャッシュ エントリ
- Outlook の PST、OST、MSG、OFT ファイル: フォルダー、アイテム、受信者、添付ファイル、連絡先、予定表アイテム、タスク
- LNK のショートカット
- Jump Lists
- ごみ箱: 削除された項目の元のパス、サイズ、削除日時、ユーザー
- ShimCache
- Persistence Hunter (許可/ブロック プロファイルを使用した 131 クロスソース永続性チェック)
- Defender 隔離メタデータ
- 「開いて名前を付けて保存」ダイアログの履歴
- 回復可能な削除されたデータを含む、すべてのレジストリ キーと値
- サービス、RecentDocs、TaskCache、SAM アカウント、レジストリの自動起動の場所、UserAssist、BAM、インストールされたプログラム、MuiCache、ShellBags、および USB デバイス
- システム情報: 名前、Windows バージョン、タイムゾーン、最終シャットダウン、プロファイル、SID
- 監査ポリシー: システムがログに記録するように設定されているセキュリティ イベント
- ブート構成 (BCD および BCD-Template)
- ネットワーク プロファイル、インターフェイス、共有、ポート プロキシ、およびルート
- LSA シークレットの名前と時刻、および使用中のキャッシュされたドメイン ログオン (何も復号化されません)

Persistence Hunter は、部分的な許可一致を含むすべての重大度 (`--min-severity info`) を保持します。 `LOW` は調査すべき分類です。署名メタデータが欠落していると、悪意のあるコマンドに対しても署名メタデータが生成される可能性があります。自動インベントリは、手動コマンドのデフォルトの中しきい値を使用して、これらのエントリを破棄しません。

ハンターは、各自動パーサー プロセス内で 1 人のワーカーを使用します。これにより、`auto-parse -T <N>` のプロセス バジェットがすべての子内の CPU サイズの別のプールで増加することがなくなります。スタンドアロン `persistence-hunter -T <N>` は、ハンター自身のワーカー スレッドを制御します。

CSV の `$MFT` 出力には、すべての時刻と短いファイル名が含まれます。レジストリの出力には、生データと `REG_QWORD` 値の FILETIME としての解釈が含まれます。SRUM の JSON・JSONL 出力には、値の状態を示すメタデータが含まれます。Defender の隔離ファイル抽出は引き続き明示的に有効にする。

`--mft-slack` を指定すると、`$MFT` の出力には FILE レコードのスラックに残ったファイル名も、[`parse-ntfs mft -x`](../artifacts/mft.md) と同じ形で含まれます。CSV では `FromSlack` が付いた行、JSON・JSONL では `SlackFileNames` です。`--code-page` は LNK ファイル、ジャンプ リスト、ShellBags、OpenSaveMRU に適用されます。

収集されなかったアーティファクトは通常の `skipped` タスクであり、失敗ではありません。


## ファイルシステムの再構成 { #filesystem-reconstruction }

収集した MFT ごとに、フォルダーをたどれる再構成ツリーも作成する。既存の解析テーブルの名前と内容は変わらず、CSV・JSON・JSONL のどのモードでも追加の出力になる。

収集パスからドライブを推定できる場合は `FileSystem/C` のようなディレクトリを使う。MFT 自体にはドライブ文字は記録されない。推定できなければ `UnknownVolume` を使い、同じ文字の MFT が複数あれば `C-MFT-2` のように分離する。

MFT で未割り当てのファイルとディレクトリには `-deleted` を付ける。削除済みの親ディレクトリも子孫のパスに同じ接尾辞で表示する。残存する無名のレジデント DATA は復元し、非レジデントまたは内容を取得できないファイルは空のプレースホルダーにする。元の論理サイズと復元したバイト数は別の情報だ。

`-i30` は削除を断定できない追加のディレクトリインデックス名を示す。MFT 内のレジデントインデックスと、同じボリュームから別途収集した I30 の両方を使う。スラックには削除だけでなくリネームやインデックス再編成のコピーも残る。古い名前に、再利用された MFT レコードや別の名前の内容を流用しない。

`_Juntei-Unresolved` には、親参照の欠落・再利用・循環や過度に深い階層のため位置を安全に決められない項目を置く。保存先で危険な名前はエスケープし、衝突した名前は別々に保存する。元の名前は一覧に残す。長い名前がある場合は DOS の別名を省き、ファイルのハードリンクは独立したコピーとして作る。

`_Juntei-Reconstruction.csv` が再構成の一覧だ。内容とパスの状態を `DataStatus`、`LogicalSize`、`RecoveredBytes`、`OriginalPath`、`OutputPath` で区別する。出所は `EntryNum`、`SN`、`ParentEntryNum`、`ParentSN`、`Source`、`FromSlack`、`Offset` で追跡できる。推定ドライブ、削除状態、ディレクトリかどうかも記録する。途中の一覧は .partial.csv で終わり、再構成が正常終了した場合だけ最終名にする。

`manifest.json` / `manifest.csv`: `ReconstructionRoot`, `ReconstructionInventory`.

復元ファイルには元の拡張子が付き、マルウェアが含まれる可能性がある。絶対に実行しない。リパースポイント、ACL、元のタイムスタンプ、実行権限は復元しない。名前付きの代替データストリームは通常のファイルに変換しない。必要なら MFT のレジデントストリーム抽出を別途使う。

再構成では数十万個のファイルを作る場合があり、空のプレースホルダーでも inode とディレクトリのメタデータ領域を使う。不要なら次のように無効にする。

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> --no-reconstruct-filesystem
```

上書きが有効なら、すべてのファイルとディレクトリが完成済みの一覧と一致する場合だけ、以前の再構成を置き換える。未記録のファイル、欠落、リンク、ジャンクション、未完成の一覧があれば、解析結果を書き換える前に拒否する。その場合は新しい出力先を使う。


## 出力レイアウト { #output-layout }

結果はコンピュータおよび調査のカテゴリ別に整理されます。カテゴリ ディレクトリでは、`Accounts` や `FileAndFolderOpening` など、PascalCase を使用します。 Amcache テーブルと Jump Lists テーブルは、ファイル名プレフィックスが付いたカテゴリに直接配置されます。コンピュータのディレクトリ名は、検出されたスペルを保持します。既存の出力ディレクトリは移行されません。古い名前から切り替える場合は、新しい出力ディレクトリを選択してください。各パーサーは、アーティファクトが収集されなかった場合も含め、その結果の横に `<artifact>-logs.txt` ファイルを書き込みます。各アーティファクトには異なる列があるため、CSV は個別のテーブルを保持します。

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── BrowserActivity/
│   │   ├── Browser-Cookies.csv
│   │   ├── Browser-Downloads.csv
│   │   ├── Browser-FormHistory.csv
│   │   ├── Browser-History.csv
│   │   ├── Browser-logs.txt
│   │   ├── WebCache-Containers.csv
│   │   ├── WebCache-Cookies.csv
│   │   ├── WebCache-Downloads.csv
│   │   ├── WebCache-Entries.csv
│   │   ├── WebCache-History.csv
│   │   └── WebCache-logs.txt
│   ├── Communication/
│   │   ├── Outlook-Attachments.csv
│   │   ├── Outlook-Calendar.csv
│   │   ├── Outlook-Contacts.csv
│   │   ├── Outlook-Files.csv
│   │   ├── Outlook-Folders.csv
│   │   ├── Outlook-Items.csv
│   │   ├── Outlook-Recipients.csv
│   │   ├── Outlook-Tasks.csv
│   │   └── Outlook-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── RecycleBin.csv
│   │   ├── RecycleBin-logs.txt
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   ├── Registry-RemoteAccess-logs.txt
│   │   ├── UAL-Clients.csv
│   │   ├── UAL-RoleAccess.csv
│   │   ├── UAL-SystemIdentity.csv
│   │   └── UAL-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```

| カテゴリ | アーティファクト |
|---|---|
| `Accounts` | SAM ローカル アカウント、LSA シークレットの名前と時刻 |
| `Applications` | アプリケーション履歴 (サードパーティツールのトレース) |
| `Applications` | インストールされているプログラム |
| `BrowserActivity` | Web ブラウザの閲覧履歴、ダウンロード、Cookie、フォーム入力履歴 (復元された削除済みエントリを含む)、Internet Explorer と従来の Edge の WebCache |
| `Communication` | Outlook の PST、OST、MSG、OFT ファイル: フォルダー、アイテム、受信者、添付ファイル、連絡先、予定表アイテム、タスク |
| `ExternalDevices` | USB デバイスとボリューム |
| `FileAndFolderOpening` | Jump Lists、LNK ショートカット、Microsoft Office、MRU リスト、開くダイアログと名前を付けて保存ダイアログ、RecentDocs、ShellBags |
| `FileExistence` | Amcache、ごみ箱、ShimCache |
| `FileSystem` | `$Boot`、`$I30`、`$LogFile`、`$MFT`、`$Secure:$SDS`、USN ジャーナル |
| `NetworkActivity` | ネットワーク プロファイル、インターフェイス、共有、ポート プロキシ、およびルート。 SRUM 接続、トラフィック、プッシュ通知 |
| `LateralMovement` | ユーザーの送信リモート デスクトップ、PuTTY および WinSCP 接続、マップされたドライブと共有。ユーザーアクセスログ (サーバーの役割のクライアント) |
| `Persistence` | ブート構成 (BCD)、レジストリ自動起動、サービス、TaskCache |
| `ProcessExecution` | アプリケーションの互換性、BAM および DAM、FeatureUsage とピン、MuiCache、Prefetch、Prefetch タイムライン、SRUM アプリケーション/CPU アクティビティ、UserAssist |
| `Registry` | 一般的なハイブキーと値 (回復されたレコードを含む) |
| `SecurityProducts` | Defender 隔離メタデータとオプションの抽出コンテンツ。セキュリティ設定 (Defender、UAC、資格情報の保護、RDP、ファイアウォール、ログ) |
| `SystemInformation` | 名前、Windows バージョン、タイムゾーン、最終シャットダウン、プロファイル、SID、監査ポリシー。 SRUM エネルギー/容量、ID マップ、および不明なプロバイダー |

カテゴリは調査を整理するのに役立ちます。それらは特定の活動の証拠ではありません。たとえば、Amcache および ShimCache エントリだけでは実行は確立されません。 SRUM はリソースの使用状況を記録します。実行中のプログラムについて結論を出す前に、アーティファクト ガイドを参照してください。

CSV、JSON、および JSONL は同じカテゴリを使用します。パーサー ジョブごとに 1 つの `<artifact>-logs.txt` ファイルには、その正確なコマンドとメッセージが含まれます。たとえば、`HOST1/FileSystem/MFT-logs.txt` や `HOST1/ProcessExecution/Prefetch-logs.txt` などです。 JSON と JSONL は同じログ ファイル名を使用します。 Amcache と Jump Lists にはそれぞれ、プレフィックス付きテーブルの横に 1 つの共有ログがあります。 SRUM プロバイダー テーブルは、共有 `SystemInformation/SRUM-logs.txt` ファイルを使用して、`NetworkActivity`、`ProcessExecution`、および `SystemInformation` に直接配置されます。 `logs/` サブディレクトリは作成されません。

デフォルトの CSV ファイル名は、`Registry-SAM.csv`、`LNK.csv`、`RecycleBin.csv`、`Registry-RecentDocs.csv`、`Registry-ShimCache.csv`、`LogFile.csv`、`I30.csv`、`MFT.csv`、`UsnJournal.csv`、 `Boot.csv`、`SecureSDS.csv`、`Registry-Networks.csv`、`Registry-Persistence.csv`、`Registry-Services.csv`、`Registry-TaskCache.csv`、`Registry-BCD.csv`、`Registry-SecuritySettings.csv`、`PrefetchTimeline.csv`、 `Prefetch.csv`、`Registry-UserAssist.csv`、`Registry-AppCompat.csv`、`Registry-Office.csv`、`Registry-FeatureUsage.csv`、`Registry-AppHistory.csv`、`Registry-MRU.csv`、`Registry-SystemInfo.csv`、`Registry-RemoteAccess.csv`、 `Registry-OpenSaveMRU.csv`、`Registry-InstalledPrograms.csv`、`Registry-BAM.csv`、`Registry-MuiCache.csv`、`Registry-ShellBags.csv`、`Registry-USB.csv`、`Registry-SecuritySettings.csv`、`Registry-AuditPolicy.csv`、`Registry-Dump.csv`、および`DefenderQuarantine.csv`。レジストリ派生テーブルでは、`Registry-ShimCache.csv`、`Registry-UserAssist.csv`、`Registry-BAM.csv`、`Registry-SystemInfo.csv` など、`Registry-` プレフィックスが使用されます。一般的なハイブのキーと値は `Registry-Dump.csv` にあります。 Amcache テーブルは、`FileExistence/Registry-Amcache-Devices.csv`、`Registry-Amcache-Files.csv`、`Registry-Amcache-Findings.csv`、および `Registry-Amcache-Summary.csv` です。 Jump Lists は `FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` と `JumpLists-CustomDestinations.csv` を使用します。 Amcache または Jump Lists サブディレクトリはありません。 SRUM テーブルは、そのカテゴリの直下で `SRUM-` ファイル名を使用します (例: `NetworkActivity/SRUM-NetworkConnections.csv`)。ユーザーアクセスログのテーブルは `LateralMovement` にあり、`UAL-Clients.csv` や `UAL-SystemIdentity.csv` のようにテーブルごとに 1 つのファイルとなり、`UAL-logs.txt` も作成されます。[ユーザーアクセスログ (UAL)](../artifacts/ual.md#automatic-parsing) を参照してください。Web ブラウザのテーブルは `BrowserActivity` にあり、`Browser-History.csv`、`Browser-Downloads.csv`、`Browser-Cookies.csv`、`Browser-FormHistory.csv` と `Browser-logs.txt` が作成されます。[Web ブラウザ](../artifacts/browser.md#automatic-parsing) を参照してください。Internet Explorer と従来の Edge の WebCache のテーブルも同じ場所にあり、`WebCache-History.csv`、`WebCache-Downloads.csv`、`WebCache-Cookies.csv`、`WebCache-Entries.csv`、`WebCache-Containers.csv` と `WebCache-logs.txt` が作成されます。[Internet Explorer と従来の Edge](../artifacts/webcache.md#automatic-parsing) を参照してください。 Outlook のテーブルは `Communication` にあり、`Outlook-Files.csv`、`Outlook-Folders.csv`、`Outlook-Items.csv`、`Outlook-Recipients.csv`、`Outlook-Attachments.csv`、`Outlook-Contacts.csv`、`Outlook-Calendar.csv`、`Outlook-Tasks.csv` と `Outlook-logs.txt` が作成されます。[Outlook](../artifacts/outlook.md#automatic-parsing) を参照してください。 [完全な SRUM 配置テーブル](../artifacts/srum.md#categorized-auto-parse-output) を参照してください。

ログは、`Registry-SAM-logs.txt`、`PrefetchTimeline-logs.txt`、`Registry-UserAssist-logs.txt` などの大文字のアーティファクト名と一致します。 JSON/JSONL ファイルは同じステムを使用します (例: `MFT.jsonl`)。レコードの `Artifact` 識別子は、クエリの場合は `mft` です。マルチテーブル出力では、テーブルの横に `Registry-Amcache-logs.txt` または `JumpLists-logs.txt` が表示されます。共有 SRUM ログは `SystemInformation` にあります。


`manifest.json` は、検出された元の `Hostname`、一意の出力/レコード `Comp` ID、ホスト名の来歴、証拠パス、アーティファクト `Category`、コマンド ライン、期間、終了コード、結果パス、ログ パス、要求された JSONL タイムライン、およびエクスポート エラー (Defender 抽出かどうか) を記録します。が要求されたか、各タスクが成功したか、スキップされたか、失敗したか。

自動解析では絶対パスが書き込まれないため、証拠、結果、または Juntei が分析システム上のどこにあるかを示さずに結果を共有できます。

- 証拠のパスは証拠のディレクトリに対する相対パスです。ターミナルは各コンピュータを証拠 (`HOST1: ./HOST1_F`) 内のディレクトリとともにリストし、パーサーは証拠ディレクトリ内で実行され、そのようなパスが与えられます。結果のソース パス列 (`SourceFile`、`SourcePath`、…)、ログ、マニフェストの `EvidenceRoot` および `WindowsDirectories` は、証拠 (`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`) 内の各ファイルの場所を示します。
- マニフェストとログの結果パス (`Output`、`OutputFiles`、`Log`、`JSON-Timeline`、`Command` の出力パス) と Defender の `ExtractedFile` は出力ディレクトリに対する相対パスです。 (その横のタイムラインは `./HOST1/ProcessExecution/Prefetch.csv`、`../siem.jsonl`)。 `Command` はプログラム名 (`juntei`) で始まります。その証拠パスは証拠ディレクトリに対する相対パスであり、その出力パスは出力ディレクトリに対する相対パスです。
- `EvidenceDirectory` と `OutputDirectory` は 2 つのディレクトリの名前です。 `ManifestVersion` 2 は、出力ディレクトリからの結果パスを記録するマニフェストをマークします。以前のマニフェストには `ManifestVersion` がなく、`-o` に与えられたものとして記録されます。 `-C` を再実行すると、両方が読み取られます。
- ターミナルには、相対パスの場合は指定された出力パスまたはタイムライン パスが表示され、絶対パス (`../results/`) の場合は現在のディレクトリからのパスが表示されます。

並列プロセスの数の後に、端末には、見つかった Windows システムのファイルの合計サイズ (`Total filesize: 26.1 MiB`) が表示されます。

進行状況メッセージには、アクティブなコンピューターとアーティファクトに続いて、おおよその入力サイズ (`HOST1 / prefetch (~2.0 MiB)` など) が表示されます。ファイルのグループは、それらを組み合わせたサイズを使用します。各 MFT、LogFile、または USN ジャーナル タスクには、個別の入力サイズが表示されます。推定には補助トランザクション ログとエンリッチメント ファイルは含まれておらず、解析された出力のサイズは予測されません。

`manifest.csv` は、スプレッドシート対応テーブルと同じタスク インデックスを提供し、パーサー タスクごとに 1 行が含まれます。これには、`Comp`、オリジナル `Hostname`、`FQDN`、`HostnameSource`、`EvidenceRoot`、`Artifact`、`Category`、`OutputType`、`Purpose`、が含まれます。 `Status`、`DurationMs`、`ExitCode`、`Output`、`OutputFiles`、`Log`、`Error`、および `Command`。 `Purpose` は、分類された結果の場合は `artifact`、CSV で使用される追加のネイティブ JSONL パスの場合は `json_timeline` です。 `OutputType` は、実際のパーサー出力形式です。 `Command` は、スペースと引用符を保持した、実行可能ファイルとその引数の JSON 配列です。 SRUM の場合、`Output` はコンピュータ ディレクトリ、`OutputFiles` は正確なプロバイダ出力パスの JSON 配列、`Category` は共有ログの場所 (`SystemInformation`) の名前です。ファイルは個別に分類されます。 `OutputFiles` は JSON マニフェストにも表示されます。

`Status` を `skipped` または `failed` にフィルタリングして、レビューが必要なタスクを見つけます。スキップされたタスクの `Error` は空である場合があり、その説明は `Log` ファイルにあります。

ターミナルの概要には、マニフェスト、パーサー ログ、抽出されたファイルの両方を含む、現在出力ディレクトリの下にある通常のファイルの合計数とサイズが含まれます。シンボリックリンクはたどられません。サイズは、`Total filesize` と同じフォーマッタを使用して、`KiB`、`MiB`、`GiB` などの単位で自動的にスケールされます。

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

複数のソース データベースが収集された場合、Amcache テーブルと SRUM テーブルは、`-Source1`、`-Source2` などを使用します。共有パーサー ログには元のパスから最終的なパスまでが記録され、JSON/JSONL レコードにはソースの出所が保持されます。 CSV Amcache/Jump Lists タスクは、マニフェスト `OutputFiles` に正確なテーブル パスをリストします。彼らの `Output` は調査カテゴリのディレクトリです。 JSON および JSONL Amcache および Jump Lists エクスポートは、すでに `Registry-Amcache.json[l]` および `JumpLists.json[l]` という名前の単一ファイルになっています。結合された `-j` タイムラインは、同じレコード内容と `Artifact` 値を保持します。 `--clobber` は、前述のマニフェストを使用して、無関係なカテゴリ ファイルを保持しながら、以前の名前を含む記録された出力をクリアします。通常の単一ボリューム コレクションは、`$MFT` および USN ジャーナル レコードを `FileSystem/MFT.csv` および `FileSystem/UsnJournal.csv` に直接書き込みます。追加の収集ボリュームは、`FileSystem/MFT-2.csv` や `FileSystem/MFT-2-logs.txt` などの番号付き兄弟ファイルを使用します。

## JSON および SIEM の取り込み { #json-and-siem-ingestion }

すべての JSON または JSONL レコードは、次の 2 つのトップレベル フィールドを受け取ります。

- `Comp`: ソース システム。
- `Artifact`: レコードを生成した Juntei パーサー (`mft` や `prefetch-timeline` など)。

`-j, --json-timeline <FILE>` は、個々の結果の形式に関係なく、追加の JSONL ファイルを 1 つ作成します。すべての分類された出力とそのログが保持されます。

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

ファイル名は指定されたとおりに使用されます。相対パスは、現在の作業ディレクトリからの相対パスです。これは、出力ルート、予約されていない別の出力サブディレクトリ、または出力ディレクトリの外部にある可能性があります。証拠、マニフェスト、コンピューターごとの結果/ログ、および一時パーサー ディレクトリを宛先として使用することはできません。既存のファイルには `-C` が必要です。エクスポートが失敗しても、そのまま残ります。

**CSV モード**、Juntei では、追加のネイティブ JSONL パスが実行されるため、SIEM ファイルは、CSV テーブルが省略できる数値型とブール型、ネストされたデータ、生のフィールド、およびメタデータを保持します。追加のジョブは同じ `-T` 同時実行制限を共有し、隔離コンテンツを再度抽出せず、別個の `*-json-timeline-logs.txt` サイドカー ログを持ちます。どちらのマニフェストも、その目的と出力形式を区別します。一時 JSONL ファイルは、実行が成功すると削除されます。パーサーまたはエクスポートが失敗した場合でも、それらは保持されます。追加の処理時間と一時ディスク容量を計画してください。

**JSON モード** では、Juntei は証拠を再解析せずに、既存の結果をコンパクトな JSONL にストリーミングします。 Amcache レポートは、ネイティブ JSONL と同じ個別の概要、検出結果、ファイル、およびデバイス レコードに拡張されます。 **JSONL モード** では、既存の注釈付きレコードが直接結合されます。すべてのモードで同じ SIEM レコード構造が使用されます。データセットはネイティブのタイムスタンプとアーティファクト フィールドを保持します。共通のイベント タイムスタンプを作成したり、異種アーティファクトをグローバルに並べ替えたりすることはありません。

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

`manifest.json` は、要求された `JSON-Timeline` パスとすべての `JSON-TimelineError` を記録します。そのタスク レコードと `manifest.csv` は、依然として保持されたアーティファクト出力を指しています。成功した CSV タイムラインのみのジョブは、削除された一時ファイルではなく、最終的な JSONL ファイルを参照します。いずれかのパーサーが失敗した場合、実行は失敗して終了し、タイムラインには成功した結果のみが含まれます。完了したものとして扱う前に、マニフェストとカテゴリのログを検査してください。ファイル数/サイズの概要には、出力ディレクトリ内のファイルが含まれます。外部タイムラインは `JSONL timeline: <FILE>` として個別に報告されます。

## パフォーマンス { #performance }

パーサー ジョブは別個のプロセスであるため、CPU の負荷が高いアーティファクトを同時に実行することができ、1 つのパーサーの損傷によって完全な実行が終了することはありません。デフォルトでは、Juntei は論理 CPU カウントを使用します。証拠が遅いディスク上にある場合、またはシステムのメモリが少ない場合は、同時実行を制限します。

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## 使用法 { #usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]
      --vhdx                  Parse VHDX images natively on Linux, macOS and Windows (read-only; temporary disk space required)
      --e01                   Parse E01/Ex01 images natively (split images from their first segment; temporary disk space required)
      --temp-dir <DIR>        Directory for private image staging (default: output filesystem)

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -Z, --timezone <OFFSET>             Display parsed timestamps using a fixed UTC offset (UTC by default) [default: +00:00]
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK files, Jump Lists, ShellBags and OpenSaveMRU [default: 1252]
      --mft-slack           Recover the file names left in the slack of $MFT records
      --no-reconstruct-filesystem  Skip the browsable MFT/I30 filesystem reconstruction

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

実際には新しい出力ディレクトリを使用してください。 `-C` は、この実行に対して生成された正確なターゲットのみを削除または上書きするため、現在欠落しているアーティファクトによって古い結果が残ることはありません。無関係なファイルや完全な出力ツリーは削除されません。また、新しい実行で `-x` を使用するかどうかに関係なく、解析前に、各コンピューターの `SecurityProducts/DefenderQuarantined/` ディレクトリと予約された `defender-quarantined/` ディレクトリが以前のフラット レイアウトから削除されます。

古いフラット レイアウトから切り替える場合は、新しい出力ディレクトリを使用します。以前の CSV/JSON ファイルは、自動的に再配置または削除されません。

!!! danger "抽出された Defender コンテンツをマルウェアとして扱う"
    `-x, --extract-defender-quarantined` は、隔離されたコンテンツを各コンピューターの `SecurityProducts/DefenderQuarantined/` ディレクトリの下のディスクに書き込みます。回復されたバイナリ ストリームは `.quarantined` で終わり、Juntei は実行権限を削除します。各バイナリ セキュリティ記述子は、所有権、権限、および SDDL を含む読み取り可能な `.security_descriptor.txt` レポートも取得します。復元されたペイロードは決して実行しないでください。分離された分析システムを使用し、ウイルス対策ソフトウェアまたは EDR ソフトウェアが回復されたコンテンツについて警告するか、削除することを期待してください。

レポート フィールドとそのフォレンジック制限については、[回復されたセキュリティ記述子の読み取り](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)を参照してください。

自動解析は、ユーザーのリモート接続 (RDP、PuTTY、WinSCP、およびマップされたドライブ) も読み取ります。

## 時系列を統合したタイムラインの作成 { #build-a-chronological-super-timeline }

解析後、[`super-timeline`](super-timeline.md) を使うと、記録された各時刻を意味付きのイベントに展開できます。`--sort` を指定すると、ホストやアーティファクトを横断して時系列に並べられます。

```bash
juntei super-timeline -d output/ -o SuperTimeline.csv --sort
```

既存の `-j, --json-timeline` は、各パーサーのスキーマを保ったネイティブレコードを JSONL にまとめる機能です。時刻の展開や並べ替えは行いません。詳細な証拠確認のため、このファイルと各パーサーの結果も保管してください。
