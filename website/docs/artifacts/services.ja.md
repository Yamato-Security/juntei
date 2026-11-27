# Windows サービス { #windows-services }

## サービス構成記録の内容 { #what-service-configuration-records }

Windows サービスとカーネル ドライバーは、各 `SYSTEM` コントロール セットの下で構成されます。レコードには、実行可能ファイル/ドライバーまたはサービス DLL、開始モード、サービス タイプ、アカウント、ロード順序グループ​​、遅延開始、エラー動作、および失敗後に Windows が実行するアクションの名前が付けられます。

攻撃者は、永続化、権限昇格、ドライバーの読み込み、横方向の移動のためにサービスを作成または変更します。正規のソフトウェアはまったく同じメカニズムを使用するため、パス、アカウント、署名、作成コンテキスト、および実行時の証拠が重要になります。

<span class="evidence-strength">この証拠から判断できること:</span> 解析したコントロールセットには、表示されたサービスやドライバの設定がありました。設定だけでは、そのサービスが正常に開始したとは証明できません。

## 場所 { #location }

```text
SYSTEM\ControlSet00N\Services\<ServiceName>
SYSTEM\Select\Current   (which control set Windows used)
```

`SYSTEM` とそのトランザクション ログを収集します。システム 7045 やセキュリティ 4697 などのイベント ID はインストールを記録できます。サービス制御イベントとプロセス イベントは、実際の開始/実行をサポートできます。

## 分析時の注意点 { #analyst-cautions }

- `ImagePath` には、環境変数、引用符、引数、デバイス パス、またはスペースを含む引用符で囲まれていないパスを含めることができます。正規化する前に正確に保存してください。
- `ServiceDll` は、`svchost.exe` がホストするサービスにとって重要です。 `ImagePath` 単独ではホストのみに名前を付けることができます。
- `StartType=automatic` は、サービスが開始されたか、有効なままであることを証明しません。
- `ObjectName` は構成されたアカウントであり、サービスをインストールしたユーザーである必要はありません。
- ドライバーとユーザー モード サービスは、異なる `ServiceType` フラグとパス規則を使用します。
- 失敗アクションでは、失敗後にサービス アカウントとしてコマンドを実行できます。これは見落としがちな永続サーフェスです。
- 他の制御セットは以前の構成を保存する場合があります。 `IsCurrentControlSet`を使用してください。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --service <NAME>    Only this service name
      --all-control-sets  Parse every ControlSet00N

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --all-control-sets -t jsonl \
  | jq 'select(.FailureCommand != null or (.ImagePath? | test("temp|public"; "i")))'
```

## 出力フィールド { #output-fields }

CSV では、`DependOnService`、`DependOnGroup`、`RequiredPrivileges` をレジストリに記録された順序の JSON 配列で出力します。`[]` は値が存在し、リストが明示的に空であることを示します。値が存在しない場合や読み取れない場合は空欄になります。読み取りエラーは `Problems` で確認できます。

| フィールド | 解釈 |
|---|---|
| `Host`, `SourcePath` | システム ID と入力ハイブ。コレクション間で保存します。 |
| `ControlSet`, `IsCurrentControlSet` | ソース管理セットと、`Select\Current` がそれに名前を付けるかどうか。アクティブ状態の主張には電流を優先します。 |
| `ServiceName` | レジストリ キー/内部サービス名。イベントと Service Control Manager コマンドへの強力なピボット。 |
| `DisplayName`, `Description` | ユーザー向けのテキスト。リソース参照または攻撃者によって制御される可能性があります。 |
| `ImagePath` | 構成された実行可能ファイル/ドライバー コマンド。引用符、引数、環境変数、デバイス パスを解析します。 |
| `ServiceDLL` | DLL はホスト サービスによってロードされました。多くの場合、共有プロセス サービスの実際のペイロード。 |
| `StartType`, `StartValue` | フレンドリー/raw スタート モード: ブート (0)、システム (1)、自動 (2)、手動 (3)、無効 (4)。 |
| `DelayedAutostart` | 自動サービスは初期ブートフェーズ後に開始されます。まだ永続化可能です。 |
| `ServiceType`, `TypeValue` | デコードされた/生のフラグ: カーネル ドライバー、ファイル システム ドライバー、独自/共有プロセス、ユーザー サービスなど。 |
| `ErrorControl` | ブート/スタートのエラー処理レベル。主に運用コンテキスト。 |
| `ObjectName` | 構成されたサービス アカウントまたはドライバー オブジェクト名。設置者の ID ではありません。 |
| `Group` | ロード順序グループ。ブート/システム ドライバーにとって特に重要です。 |
| `DependOnService`, `DependOnGroup` | 起動時に依存するサービスとロード順序グループです。指定されたサービスは先に起動する必要があります。グループの場合、Windows は各メンバーの起動を試み、少なくとも一つが実行中であれば依存条件を満たします。 |
| `RequiredPrivileges` | サービスに設定された特権です。設定がない場合、サービス コントロール マネージャーはプロセス トークンの既定の特権を使用します。プロセスを共有する場合は、各サービスが要求する特権の和集合を使用します。トークンに `SeChangeNotifyPrivilege` が既にあれば、リストに記載がなくても保持されます。 |
| `ServiceSidType` | `none`、`unrestricted`、`restricted` のいずれかです。トークンにサービス固有の SID (`NT SERVICE\<name>`) が含まれるかどうかを示し、ACL ではこの SID にアクセスを許可できます。`restricted` ではトークンが書き込み制限付きにもなります。 |
| `UserServiceFlags` | ユーザーごとのサービスのフラグを、保存されている数値のまま出力します。ユーザーごとのサービスのテンプレートとそのインスタンスにあります。 |
| `FailureResetPeriod` | 失敗カウントがリセットされるまでの秒数。 `4294967295` は決してしないことを意味します。 |
| `FailureCommand` | `run_command` 失敗アクション用に設定されたコマンド。高価値の永続フィールド。 |
| `RebootMessage` | `reboot` 失敗アクションの前にユーザーへ送信されるメッセージです。テキストまたはリソース参照です。 |
| `FailureActionsOnNonCrashFailures` | `True` の場合、プロセスが停止を報告せずに終了したときだけでなく、サービスが 0 以外の終了コードで停止したときにも失敗アクションを実行します。空の場合、このキーには保存されていません (ユーザーごとのサービスのインスタンスはテンプレートの設定を使うことがあります)。 |
| `FailureActions` | 保存されているすべてのアクション (`restart`、`reboot`、`run_command`、`own_restart`、`none`) と遅延を順に出力します。最後のアクションはそれ以降のすべての失敗に適用されます。[回復] タブには 1 回目、2 回目、その後の失敗しか表示されません。 |
| `FailureActionType`, `FailureActionDelayMs` | 最初のアクションはフィルタリングのために平坦化されています。完全なリストも調べてください。 |
| `FailureActionsRaw` | デコードに失敗した場合、または宣言されたアクションの後にバイトがある場合の元のバイトです。 |
| `KeyLastWrittenUTC` | サービス キーへの最後の変更。必ずしもインストール/開始時刻ではありません。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ再生コンテキスト。 |
| `Problems` | 型の不一致、不正な形式の障害構造、または欠落しているか読み取り不可能なフィールド。 |

## 調査戦略 { #investigation-strategy }

1. 標準以外のパス、書き込み可能なディレクトリ、異常なアカウント、新しいキーの時刻、および失敗したコマンドを確認します。
2. アナリスト マシンではなく、影響を受けるホストの構成を使用して環境変数を展開します。
3. 実際の実行可能ファイル、または収集された場合は DLL のハッシュ/署名をチェックします。
4. サービス名/パスをイベント 7045/4697、プロセス作成、Prefetch、`$MFT`、および USN に関連付けます。
5. 現在のコントロール セットと非現行のコントロール セットを比較して、変更/削除された永続性を確認します。

## さらに詳しい情報 { #further-details }

- [Microsoft: デバイスおよびドライバー サービスのレジストリ ツリー](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: SERVICE_STATUS / サービスの種類と状態](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_status)
- [Microsoft: ChangeServiceConfig2 失敗アクション](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w)
- [Microsoft: SERVICE_REQUIRED_PRIVILEGES_INFO](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_required_privileges_infow#remarks)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
