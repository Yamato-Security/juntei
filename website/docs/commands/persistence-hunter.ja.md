# Persistence Hunter { #persistence-hunter }

永続化により、再起動、サインイン、スケジュールされたトリガー、または別のイベントの後にプログラムを再度実行できます。攻撃者は、この目的で通常の Windows 構成を使用できます。多くの正規プログラムはまったく同じメカニズムを使用しているため、永続化エントリを見つけることが分析の開始点となります。

`juntei persistence-hunter` は、[Hexastrike の PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper) のネイティブ Rust ポートであり、131 チェック: 130 チェック PyrsistenceSniper カタログとネイティブ スタート メニューを備えています。 COM チェック。

すべてのメカニズムの説明、その証拠の場所、および結果の調査方法については、[131 チェック永続性ガイド](../persistence-hunter/index.md) を参照してください。 [永続確認](../persistence-hunter/acknowledgements.md)は、カタログの背後にあるPythonおよびPowerShellプロジェクトを説明します。

## 使用法 { #usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei persistence-hunter -d <EVIDENCE> [OPTIONS]

Input:
  -d, --directory <DIR>  Evidence directory (a mounted Windows volume or a collection of hosts)
  -f, --file <FILE>      One registry hive (filesystem checks need a directory)
      --no-logs          Read dirty hives without replaying transaction logs

Filtering:
      --profile <FILE>               Replace the bundled allow/block profile with a YAML profile
      --min-severity <MIN_SEVERITY>  Minimum classification to include: info, low, medium, high [default: medium] [possible values: info, low, medium, high]
      --check <ID>                   Run only these check or ATT&CK IDs (comma separated; repeatable) [alias: --technique]

General Options:
  -h, --help         Show the help menu
      --list-checks  Show every available check and its ATT&CK technique

Analysis:
      --no-timeline  Skip MFT and event-log timestamp correlation
      --mft <FILE>   MFT to use for timestamp correlation (one host only)

Performance:
  -T, --threads <N>  Worker threads (default: logical CPU count)

Output:
  -o, --output <FILE>              Save findings to a file
  -t, --output-type <OUTPUT_TYPE>  Output type: csv (default), json, jsonl; without -o, write data to stdout [possible values: csv, json, jsonl]
      --coverage <FILE>            Save per-check coverage and errors to CSV
  -C, --clobber                    Overwrite output files

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## 最初のスキャン { #first-scan }

```bash
juntei persistence-hunter -d evidence/ -o PersistenceHunter.csv --coverage Coverage.csv
```

証拠としては、マウントされた Windows ボリューム、Velociraptor コレクション、または収集された複数のコンピューターを含むディレクトリなどが考えられます。分析中はコンピューターは別々に保たれ、すべての結果には `Comp` が含まれます。収集されたファイル名のエスケープは、Windows パスを解決するときにデコードされます。

1 つの巣箱の場合:

```bash
juntei persistence-hunter -f SOFTWARE -o software-persistence.csv --no-timeline
```

単一のハイブは、別のハイブまたはファイルシステムの証拠を必要とするチェックを提供できません。空の結果を意味のあるものとして扱う前に、カバレッジ レポートをお読みください。

チェック ID とその ATT&CK マッピングをリストします。

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,windows_service_image_path -o focused.csv
```

`--list-checks`、または ATT&CK ID (`T1547.001` など) によって印刷された小切手 ID を使用します。 `T1547` などの親 ID は、そのサブテクニックを選択します。 `--technique` は、`--check` のエイリアスです。各チェックには、その手法、説明、および結果内の参照リンクが含まれています。

## 並列スキャン { #parallel-scans }

このコマンドは共有ワーカー プールを使用し、デフォルトでは論理 CPU の数が設定されます。 `-T, --threads <N>` を設定してワーカー数を選択するか、`-T 1` を設定してシングルスレッド スキャンを選択します。

```bash
juntei persistence-hunter -d evidence/ --min-severity info -T 4 -o PersistenceHunter.csv
```

証拠の発見、独立したチェック、ペイロード メタデータ、および MFT/EVTX タイムライン ソースは並行して実行されます。ホストに読み込まれたレジストリ ハイブは読み取り専用で共有され、同じペイロードのキャッシュされたメタデータは 1 回初期化されます。コレクション全体でハイブ メモリが増大するのを避けるために、ホストは一度に 1 つずつロードされます。検出結果、カバレッジ、タイムスタンプの選択、および警告の順序は、スレッド数が異なっても同じままです。

`auto-parse` はすでにパーサー プロセスを並行して実行しているため、各 `persistence-hunter` の子に 1 つのワーカーが与えられます。独自の `-T` オプションは、同時パーサー プロセスの数を制御します。

## 収集する証拠 { #evidence-to-collect }

| ソース | チェックの例 |
|---|---|
| `SYSTEM` および隣接するトランザクション ログ | サービス、サービス回復コマンド、LSA パッケージ、ブート設定、認証プロバイダー |
| `SOFTWARE` および隣接するトランザクション ログ | 実行キー、Winlogon、IFEO、COM 登録、TaskCache、Office 設定 |
| 各ユーザーの `NTUSER.DAT` および `UsrClass.dat` | ユーザー自動起動、環境変数、COM オーバーライド、Office 設定 |
| `SAM` および `SOFTWARE` | RID の異常、非表示のアカウント、異常なローカル管理者メンバーシップ |
| `Windows/System32/Tasks` | スケジュールされたタスクのコマンドと COM ハンドラー。 TaskCacheとの比較 |
| ユーザーごとのスタート メニュー `.library-ms` ファイルおよび CLSID という名前のフォルダー | COM アクティベーション トリガー候補、一致するユーザー/マシン クラス登録に結合 |
| スタートアップフォルダー、PowerShell プロファイル、Office テンプレート、ターミナル設定 | ユーザーまたはシステムトリガーによって実行されるファイルとコマンド |
| WMI `OBJECTS.DATA` | イベント コンシューマ インスタンスとフィルタとコンシューマのバインディング |
| BITS `qmgr.db` または `qmgr*.dat` | 回復された通知コマンドと所有者 SID |
| `$MFT` および関連する EVTX ファイル | 独立したタイムスタンプ相関関係 |
| 参照される実行可能ファイル、DLL、およびカタログ ファイル | SHA256 ハッシュと説明的な PE/証明書メタデータ |

レジストリ トランザクション ログは、利用可能な場合に再生されます。 `--no-logs` は、収集されたハイブを読み取ります。再生されていないダーティ ハイブは最近の変更を省略する可能性があり、カバレッジで報告されます。現在の制御セットは、アクティブ システムのチェックに使用されます。 DSRM バックドア チェックでは、`ControlSet001`、`ControlSet002`、および `CurrentControlSet` が存在する場合はさらに検査します。その設定をアクティブとして説明する前に、報告されたパスを `SYSTEM\\Select` と照合して確認してください。他の履歴コントロール セットは、個別のレジストリ パーサー コマンドを使用して調べることができます。

Hive 読み取りは、コンピューターごとに 1 GiB の入力バジェットを共有します。マシンのハイブは、ユーザーのハイブの前に試行されます。適合しない入力またはトランザクション ログには、部分的なカバレッジで名前が付けられます。他の読み取り可能なハイブとファイルにバックアップされたチェックは続行されます。正規のハイブの場所または明示的に提供されたエクスポートのみがハイブの ID を確立します。ユーザーの一時ディレクトリ内のコピーは、そのユーザーの実際のハイブを置き換えることはできません。 SOFTWARE に記録された元のシステム ドライブは、キャプチャされたシステム ボリュームにマッピングされるため、元の `C:` ボリュームを別の証拠ドライブ ラベルにマウントしても、ペイロード メタデータは破棄されません。

`auto-parse` もハンターを実行し、`persistence/PersistenceHunter.csv` (または `.json`/`.jsonl`) を各コンピューター ディレクトリ内に保存し、その横に `PersistenceHunter-logs.txt` を保存します。その結果は、オプションの `-j <FILE>` SIEM タイムラインにも入ります。自動解析では、`--min-severity info` を使用して、許可ルールに一致するエントリを含む完全な構成インベントリを保持します。

## COM の登録とアクティベーション トリガー { #com-registrations-and-activation-triggers }

`scheduled_task_files` チェックでは、タスク XML の `Actions Context` と一致する `Principal id` を使用して、各アクションのプリンシパルを選択します。グループ プリンシパルはグループとして保持されます。どの個々のユーザーのクラス登録が使用されるかを確立することはできません。

COM-ハンドラー候補はネイティブと 32-bit クラスビュー。一致したユーザーのみがキャプチャされます `UsrClass.dat`/`NTUSER.DAT` そして機械 `SOFTWARE` が考慮されます。 SID プリンシパルはキャプチャされたものを通じて参加します `ProfileList` パス。アカウント名（以下を含む） `HOST\user` そして `DOMAIN\user`、マッチングを好む `user.DOMAIN` 平野の上の横顔 `user` プロフィール。ローカルホストのエイリアスと `.\user` キャプチャされたホスト名をサフィックス マッチングに使用します。正確なサフィックスは、他のホスト エイリアスよりも優先されます。プレーン プロファイルは、一致する修飾プロファイルがない場合にのみ考慮されます。このようなすべての一致には明示的にラベルが付けられます。 **フォルダー名のヒューリスティック** で `Description`;アカウントの所有権やドメインのメンバーシップを証明するものではありません。あいまいな一致は未解決のままであり、結果の説明とカバレッジの両方で報告されます。修飾されたプロファイル間のあいまいさによって、単純なプロファイルへのフォールバックが引き起こされることはありません。これは依存しません `Volatile Environment`、通常はメモリ専用であり、レジストリ ハイブには保存されません。重複したユーザー CLSID マシンの影を落とす CLSIDの子供たち。グループのプリンシパルがマシンを使用する COM 候補を特定し、所見の説明でその制限を保持します。通常のグループ プリンシパル自体は、カバレッジを部分的にしたり、エラー概要エントリを消費したりしません。

リゾルバーは `InprocServer32` および `LocalServer32` に従い、管理対象 `CodeBase` パス (バージョン サブキーを含む) を使用し、AppID `LocalService` を収集された SYSTEM サービス `ImagePath` に結合します。 AppID の実行可能ファイル名の関連付けは保持されますが、ベース名は実行可能ファイルの完全なパスを確立しません。ローカル `file:` CodeBase URL はデコードされます。リモート URL はアクセスせずに記録されます。登録が存在しない場合は、CLSID と `ResolutionStatus: unresolved_com_class`、空のパス/ハッシュ フィールド、および不明な `Exists` が保持されます。

`start_menu_com` チェックは、ユーザーごとのスタート メニュー `.library-ms` XML URL を読み取り、空のディレクトリを含む CLSID という名前のフォルダーを収集します。トリガー パス、CLSID、候補サーバー、およびレジストリの出所をレポートします。ファイルのみの取得では空のディレクトリを保存できず、ユーザー ハイブが欠落していると、ユーザーごとの登録の完全な検索ができなくなります。シンボリックリンクはたどられません。これらは構成の候補です。アーキテクチャ、昇格、UAC、ポリシー、およびアクティブ化コンテキストは、登録が実際に使用されるかどうかに影響を与える可能性があります。プロセス、モジュールロード、またはその他の独立した証拠によって実行を裏付けます。

## 分類とプロフィール { #classification-and-profiles }

デフォルトの最小分類は `medium` です。バンドルされた許可リストに一致するエントリを含む、完全なインベントリを検査するには、次の手順を実行します。

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o all-persistence.csv
```

!!! warning "不完全なコレクションをレビューするときはすべての重大度を使用する"
    広範な許可ルールは、署名者が見つからない場合や署名者が異なる場合でも、攻撃者のコマンドと部分的に一致する可能性があります。アップストリームは部分一致を `LOW` として分類しますが、手動コマンドのデフォルトの `medium` しきい値はこれを非表示にします。これは、Windows ディレクトリの下にある悪意のあるスケジュールされたタスクに影響を与える可能性があります。完全な調査には `--min-severity info` を使用し、`PolicyReason`、`SignerStatus`、コマンド、およびソース証拠を一緒に確認します。 `auto-parse` はこれらのエントリをすでに保持しています。

| 分類 | 意味 |
|---|---|
| `HIGH` | 構成されたブロック ルールが完全に一致しました |
| `MEDIUM` | 一致する許可ルールまたはブロック ルールがありません |
| `LOW` | 許可ルールのハード条件は一致しましたが、署名者のメタデータが一致しないか、利用できませんでした |
| `INFO` | 許可ルールが完全に一致しました |

これらはポリシーの分類であり、侵害の確率を測定したものではありません。デフォルトの `MEDIUM` エントリはマルウェアの証拠ではありません。 `INFO` は、ファイルに有効な署名があること、または攻撃者がその署名を悪用していないことを証明しません。部分的な許可一致はデフォルトのしきい値を下回っているため、スパース コレクションを確認するとき、または許可リストを調整するときに `--min-severity low` または `info` を含めます。

YAML プロファイルを `--profile profile.yaml` とともに指定します。 **はバンドルされたプロファイル** を置き換えます。選択したファイル内のグローバル ルールとチェックごとのルールが結合されます。完全に一致したブロック ルールが優先されます。たとえば:

```yaml
allow: []
block:
  - reason: Example known-bad command fragment
    value_matches: 'example-malicious-loader\.exe'
checks:
  windows_service_image_path:
    enabled: true
    allow:
      - reason: Organization's expected service path
        value_matches: '^C:\\Program Files\\ExampleCorp\\Agent\\agent\.exe$'
        signer: ExampleCorp
```

サポートされるルール条件は、`path_matches`、`value_matches`、`hash` (64 16 進数 SHA256 数字)、`not_lolbin`、および `signer` (大文字と小文字を区別しない部分文字列) です。記載されたすべてのハード条件が一致する必要があります。署名者のメタデータは、上記で説明したソフト条件です。正規表現は Rust の正規表現構文を使用します。ルックアラウンドと後方参照はサポートされていません。不明なチェック ID、無効な式、不明なプロファイル フィールドは、ポリシーを暗黙的に弱めるのではなく、エラーを生成します。

## 出力フィールド { #output-fields }

CSV がデフォルトのファイル形式です。読み取り可能な JSON オブジェクトには `-t json` を使用し、コンパクトな 1 行に 1 オブジェクトの出力には `-t jsonl` を使用します。 `-o` を使用しない場合、明示的な `-t` はデータのみを stdout に書き込みます。カバレッジ警告は標準エラー出力に送られます。どちらのオプションも指定しないと、Juntei は概要と最大 30 の候補を出力します。

| フィールド | それらをどう解釈するか |
|---|---|
| `Comp`, `CheckID`, `Technique`, `MitreID` | 行が説明するコンピュータと永続化メカニズム |
| `Path`, `Value` | レジストリの場所、ファイル、タスク、または回復されたレコードとその構成内容 |
| `Description`, `References` | メカニズムが重要な理由と詳細はどこで読むか |
| `AccessGained` | このメカニズムの通常のスコープ (`USER` または `SYSTEM`。BITS は `LOCAL_SERVICE`/`NETWORK_SERVICE` も識別できます)。特定のコマンドがその特権を達成したことを証明するものではありません |
| `Severity`, `PolicyReason` | プロファイル分類、一致ポリシーの説明、および部分一致または未検証の署名者の資格 |
| `SourceFile`, `RecordStatus` | 収集された正確なソース、指定された入力 (自動解析からの `./HOST/...`) で始まるそのパス、および割り当て/再生/リカバリ コンテキスト。これを元の証拠に遡って追跡します |
| `KeyLastWrite` | レジストリ **key の** タイムスタンプ。個々の値を特定するものではありません |
| `LastChange`, `ChangeSource`, `ChangeEvidence` | 独立して関連付けられた変更時間、ソース証明、関連付けが成功したかどうか |
| `ResolvedPath`, `ResolutionStatus`, `Exists` | ペイロード パスとオフライン解決の結果。未収集の証拠は不明ですが、`false` ではありません |
| `SHA256` | 収集されたペイロード バイトの SHA256 (読み取り可能な場合) |
| `Signer`, `SignerStatus` | 説明的な埋め込み証明書/カタログメタデータとその可用性。証明書の信頼性と署名の有効性が検証されていません |
| `IsLolbin`, `Launcher` | 構成されたランチャーまたはペイロードがバンドルされた LOLBAS リストに含まれているかどうか、およびプロキシされたペイロードに使用されるランチャー |
| `IsBuiltin`, `IsInOS-Directory` | 一般的な Windows コマンドとシステム ディレクトリのコンテキスト。どちらの分野も正当性を確立していません |

`rundll32.exe` コマンドは、DLL ペイロードに名前を付けることができます。 Juntei は、ペイロードの解決とハッシュ中にランチャーを保持します。収集されたコンピューター外の参照、ドライブの欠落、大文字と小文字を区別しないあいまいな一致、または未解決の環境変数は、分析者のコンピューターのファイルで置き換えられません。

## タイムスタンプと制限事項 { #timestamps-and-limitations }

ハンターは、アナリスト VM のコピーされたファイルの変更時刻を攻撃タイムスタンプとして使用しません。可能な場合は、レジストリの書き込みと Sysmon イベント 13、タスクのインストールとセキュリティ 4698 またはタスク スケジューラ 106、サービスのインストールとシステム 7045、およびファイルにバックアップされた構成と元の `$MFT` タイムスタンプが関連付けられます。出所は`ChangeSource`に記載されています。イベントは、構成されたペイロードが実行されたことを証明しなくても、構成変更の日付を示すことができます。ローカル システム イベント 6011 は、以前のコンピューター名を証明できます。ハンターは、収集されたコンピュータにアンカーされている検証済みの名前変更チェーンに従い、元のイベント コンピュータと名前変更レコードの証拠を `ChangeSource` に記録しながら、現在の `Comp` 値を保存します。証明されていない外部名、アーカイブされたログ、転送されたログ、および別の Windows インストールは、このルールでは受け入れられません。

キャプチャされた ComputerName/ActiveComputerName、TCP/IP ホスト名/NV ホスト名、およびクライアント情報ホスト名/FQDN は、DNS/NetBIOS の相違点と保留中の名前変更をカバーするホスト エイリアスを提供します。識別情報が取得されていない場合、プライマリ ログ内の単一の名前で、明示的な未検証の識別情報の出所と適用範囲の警告を伴う年代特定がサポートされます。複数の未検証の名前はホストに帰属されません。拒否されたコンピュータ名は、相関範囲の制限として報告されます。彼らのチャンネルはまだ収集済みとしてマークされています。

`--mft <FILE>` を使用して、単一のコンピューターに対して外部で収集された MFT を選択します。タイムスタンプ相関をスキップするには、`--no-timeline` を使用します。 `NOT_RUN`、`NOT_APPLICABLE`、`NO_ARTIFACT`、`NO_MATCH`、および `REJECTED` は空の `LastChange` を説明します。フィールドが空であっても、そのエントリが無害であるという証拠にはなりません。

WMI および BITS のリカバリは、完全なデータベース トランザクション パーサーではなく、制限されたカービングです。回復された文字列またはバインディングには、古いデータまたは削除されたデータが含まれる可能性があります。これらの記録は裏付けられる必要があります。 WMI クラス スキーマのみはサブスクリプションとしてカウントされません。収集されたタスク XML ファイルのない TaskCache エントリはレビュー対象の不一致です。トリアージ収集では単にそのファイルが省略されている可能性があります。カタログおよび PE メタデータは検証されていないため、コレクションに存在しない可能性があります。アクセシビリティ ファイルの置換チェックは、適切な参照署名メタデータに依存します。カタログ証拠が欠落しているために、すべてのアクセシビリティ実行可能ファイルにフラグが立てられることがあってはなりません。

カバレッジは、検出結果がゼロの小切手を含む、すべての小切手に対して記録されます。 `complete` は、該当する収集されたソースが読み取られたことを意味します。収集が完了したこと、またはコンピューターに他の永続性がないことを証明するものではありません。 `partial`、`not_collected`、`failed`、および `disabled` で制限について説明します。 `--coverage Coverage.csv` とパーサー ログを調査結果とともに保管してください。

ポートは Juntei の CSV/JSON/JSONL 形式を使用します。アップストリーム Python プログラムの呼び出し、オンライン エンリッチメント サービスのクエリ、アップストリーム HTML/XLSX UI の再現は行いません。

## さらに詳しく { #further-details }

- [PyrsistenceSniper の実装とプロファイル](https://github.com/hexastrike/PyrsistenceSniper)
- [MITRE ATT&CK 永続化テクニック](https://attack.mitre.org/tactics/TA0003/)
- [Microsoft: 自動実行](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
- [レジストリ自動開始インベントリ](../artifacts/persistence.md)
- [サービス](../artifacts/services.md)および[TaskCache](../artifacts/taskcache.md)
