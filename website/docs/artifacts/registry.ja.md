# Windows レジストリ ハイブ { #windows-registry-hives }

## レジストリ ハイブとは { #what-registry-hives-are }

Windows レジストリは、システムとユーザーの構成をバイナリ ハイブ ファイルに保存します。キーはツリーを形成します。キーの下の値には型指定されたデータが含まれます。レジストリ アーティファクトは、永続性、インストールされているソフトウェア、サービス、アカウント、ネットワーク構成、ユーザー アクティビティ、セキュリティ設定、および Windows 状態のほぼすべての部分を記述します。

Juntei の汎用リーダーは、プライマリ ハイブをデコードし、トランザクション ログを再生し、損傷を報告し、削除/フリーセル、セル スラック、レムナント、プレリプレイ、およびトランザクション ログのみのレコードを回復できます。

<span class="evidence-strength">この証拠から判断できること:</span> ハイブ内に、そのキーや値の状態が存在した、または復元可能な形で残っていました。キーの最終書き込み時刻が示すのは、キーが変更されたことです。どの値を変更したか、理由は何かまでは分かりません。

## 一般的な巣箱と場所 { #common-hives-and-locations }

| ハイブ | 一般的なパス | 証拠の例 |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | サービス、ドライバー、コントロール セット、ホスト名、マウントされたデバイス、ShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | インストールされているソフトウェア、TaskCache、プロファイル、Windows 構成 |
| `SAM` | `C:\Windows\System32\config\SAM` | ローカルアカウントとアカウントメタデータ |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | ローカル セキュリティ ポリシー/シークレット メタデータ (アクセス依存) |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | デフォルトのシステムプロファイル構成 |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | ユーザー設定、実行キー、RecentDocs、アプリケーションアクティビティ |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | シェル クラス、COM/ユーザー シェル アクティビティ |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | アプリケーション インベントリ (セマンティック出力には `parse-registry amcache` を使用) |

隣接する `.LOG`、`.LOG1`、および `.LOG2` ファイルを収集します。 Windows の実行中にコピーされたハイブはダーティである可能性があります。トランザクション ログには、プライマリ ファイルだけではまだ反映されていないコミットされた状態が含まれています。

## 分析時の注意点 { #analyst-cautions }

- キーの最終書き込みタイムスタンプは、個々の値ではなく、**key** に適用されます。
- 通常、レジストリ値には独立したタイムスタンプがありません。
- `CurrentControlSet` は、`SYSTEM\Select` を通じて解決されるアナリスト/Windows エイリアスです。これは文字通りの保存されたキーではありません。
- 削除されたレコードは、部分的であったり、古くなったり、回復ソース間で重複したり、上書きされたりする可能性があります。
- トランザクション ログのみのレコードには、履歴/中間状態が表示されます。 `recovered_from` について説明します。
- `absent_value` の結果は、クエリされたキーは存在するが、解析された状態ではその値が欠落していることを意味します。値が存在しなかったことを証明するものではありません。
- 文字列、パス、コマンド ラインは構成であり、構成されたアクションが実行されたことを証明するものではありません。
- 該当するログのないダーティなハイブでは、古い状態が省略されたり、公開されたりする可能性があります。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host` | 利用可能なシステムコンテキストから派生したホスト名。 Juntei が安全に識別できない場合は空です。 |
| `HiveType` | ハイブ カテゴリ (`SYSTEM`、`SOFTWARE`、`NTUSER.DAT` など) が検出されました。パスを解釈するために使用します。 |
| `SourcePath` | 正確な入力ハイブ。ディレクトリ モードでホストとユーザーを組み合わせる場合は必須です。 |
| `RecordType` | `key`、`value`、`default_value`、または `absent_value`。不在はクエリ/状態にのみ適用されます。 |
| `KeyPath` | ハイブのルートの下のパス。 `[unknown]` は、未解決の削除された祖先をマークします。 |
| `KeyLastWrittenUTC` | キー全体の最後の変更。どの値が変更されたのかを特定できません。 |
| `ValueName` | 名前付きの値。デフォルト/名前のない値の場合は空です。 |
| `ValueType` | レジストリ タイプ (`REG_SZ`、`REG_DWORD`、`REG_BINARY` など)。タイプは解釈に影響を与えます。 |
| `ValueData` | 型を認識したレンダリング値。複数の文字列は JSON のリストに残ります。 |
| `DataSize` | サイズは値によって記録されます。生のバイトと `Problems` と比較します。 |
| `ValueDataRaw` | デコードが失敗した場合、または `--raw-data` が使用された場合の正確なバイト数。検証のために保存します。 |
| `ValueU64` | JavaScript の精度を損なうことのない、DWORD/QWORD の整数表現。 |
| `ValueFiletimeUTC` | オプションの QWORD-as-FILETIME レンダリング。キーのスキーマが FILETIME を定義している場合にのみ意味があります。 |
| `AllocationStatus` | `allocated`、`deleted_complete`、`deleted_partial`、`residual_value_only`、または `transaction_log_recovered`。自信をコントロールします。 |
| `RecoveredFrom` | `free_cell`、`cell_slack`、`remnant_data`、`before_replay`、または `SYSTEM.LOG1#1234` などのログ エントリ。 |
| `TransactionLogStatus` | このレコード/ハイブ コンテキストの場合は、`clean`、`replayed`、`unchanged`、または `not_applied`。 |
| `LogsApplied` | トランザクション ログ ファイル名 Juntei が再生されました。これらをレポートに記録します。 |
| `Problems` | 損傷、セルの欠落、型の不一致、未解決のリスト/親、またはその他の制限。 |

## 回復状態の解釈 { #recovery-status-interpretation }

| ステータス | 意味 |
|---|---|
| `deleted_complete` | 削除されたキー/値は、完全な既知のパス/データで復元されました。まだ歴史的であり、アクティブな状態ではありません。 |
| `deleted_partial` | 一部のパス、値リスト、値、またはデータが上書きされたか、読み取り不能になりました。再利用されたセルの隙間を埋めないでください。 |
| `residual_value_only` | 十分なキーコンテキストがなくても値は存続しました。 `KeyPath` が存在しない可能性があります。 |
| `transaction_log_recovered` | 状態は再生前/ログ バージョンにのみ存在し、現在割り当てられている/空きレコードには存在しません。 |

## 調査戦略 { #investigation-strategy }

1. わかっている場合は、正確なセマンティック キーから始めます。検出には正規表現/データ検索を使用します。
2. コンテンツの前に `TransactionLogStatus`、`AllocationStatus`、`RecoveredFrom`、および `Problems` を確認してください。
3. 文書化されたキーのスキーマに従ってデータを解釈します。 `REG_BINARY` は自己記述型ではありません。
4. 構成を実行時の証拠 (イベント ログ、Prefetch、SRUM、サービス/タスク) と関連付けます。
5. 新規または係争中の分野の生データを保存します。

## さらに詳しい情報 { #further-details }

- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
- [マキシム・スハノフのレジストリ形式仕様](https://github.com/msuhanov/regf)
- [yarp レジストリ パーサー](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [Microsoft: レジストリ値のタイプ](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
