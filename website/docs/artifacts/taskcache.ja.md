# タスクスケジューラキャッシュ { #task-scheduler-cache }

## TaskCacheとは { #what-taskcache-is }

Windows タスク スケジューラは、タスク パスを GUID、トリガー/リスト、プリンシパル、アクション、ランタイム メタデータ、セキュリティ記述子にリンクするレジストリ キャッシュを `SOFTWARE` の下に保持します。 Juntei は、実行および COM ハンドラー アクションに加えて、従来の電子メール/メッセージ アクション、タスクのタイムスタンプ、および通常のタスク スケジューラ インターフェイスから隠されたタスクの兆候をデコードします。

タスクは、一般的な永続化およびリモート実行メカニズムです。レジストリ キャッシュは、攻撃者がタスクの XML ファイルを削除または改ざんする場合にも役立ちます。

<span class="evidence-strength">この証拠から判断できること:</span> TaskCache に、このタスクやアクションの設定があった、または残っていました。特定の実行を証明するには、実行時のフィールドと独立したログが必要です。

## 所在地 { #locations }

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

トランザクション ログを含む `SOFTWARE` と完全な `System32\Tasks` ツリーを収集します。タスク スケジューラの操作およびセキュリティ イベント ログにより、作成/更新/実行時の証拠が追加されます。

## 分析時の注意点 { #analyst-cautions }

- レジストリ キャッシュとタスク XML は、改ざん、削除、不完全な登録、または破損の後、不一致になる可能性があります。
- Juntei の `Hidden` は、通常の「Hidden」UI フラグで構成された単純なタスクではなく、構造的な隠蔽 (関連する条件下でのツリー リンケージ/セキュリティ記述子の欠落) を識別します。
- Windows 7 は、タスク アクションをキャッシュの `Actions` 値ではなく XML に保存します。そこでは空のアクションは自動的には疑われません。
- `LastRun` と `LastResult` は便利ですが、完全な改ざん防止実行履歴ではありません。
- `Context` は、タスクを登録したユーザーではなく、構成されたプリンシパル/コンテキストに名前を付けます。
- COM ハンドラー CLSID には、個別のレジストリ/ファイル解決が必要です。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.Hidden == true or (.Executable? | test("powershell|cmd|mshta"; "i")))'
```

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `SourcePath` | ホスト コンテキストとソース `SOFTWARE` ハイブ。 |
| `TaskName` | スケジューラの完全なパス。 XML およびイベント ログへのプライマリ ピボット。 |
| `TaskGUID` | GUID 結合ツリーとタスクのキャッシュ エントリ。強力な相関キー。 |
| `TaskIndex` | スケジューラによって使用されるツリー/インデックスのメタデータ。実行数ではなく、コンテキストに基づいたものです。 |
| `TaskLists` | トリガー/処理を分類するキャッシュ リスト (`Boot`、`Logon`、`Plain`、`Maintenance`)。 |
| `Hidden` | 構造的な隠れタスクインジケーター。改ざんを結論付ける前に、`InTree`、`InTasks`、および記述子の状態を確認してください。 |
| `InTree`, `InTasks` | 両方のキャッシュ側が存在するかどうか。リンクが欠落している場合は、削除/改ざん/不完全な登録が示されている可能性があります。 |
| `HasSecurityDescriptor` | ツリーエントリに `SD` があるかどうか。存在しないと、関連するバージョンでの非表示タスクの動作が引き起こされる可能性があります。 |
| `TreeKeyLastWrittenUTC`, `TaskKeyLastWrittenUTC` | 各キャッシュ側のキーレベルの変更。正確な作成時間/実行時間ではありません。 |
| `ActionIndex` | タスク内のゼロベースのアクション。 1 つのタスクで複数の行が生成される場合があります。 |
| `ActionType` | `exec`、`com_handler`、`send_email`、または `show_message`。 |
| `Executable`, `Args`, `WorkingDirectory` | 効果的な実行アクション。引用符、変数、相対パス、LOLBin を解釈します。 |
| `COM-ClassID`, `ActionData` | COM ハンドラー CLSID とデータ、または従来の電子メール/メッセージ ペイロード。 CLSID を個別に解決します。 |
| `Context` | アクションでエンコードされた主要コンテキスト (`LocalSystem`、`Author` など)。レジストラではなく、構成された ID。 |
| `Author`, `URI`, `Description` | タスクのメタデータ。攻撃者に簡単に制御され、証拠が信頼できなくなります。 |
| `LastRegisteredUTC` | DynamicInfo の登録/更新時刻 (存在する場合)。 |
| `LastRunUTC`, `LastSuccessfulRunUTC`, `LastResult` | キャッシュされた実行時/結果メタデータ。操作ログとプロセス証拠を関連付けます。 |
| `ActionsRaw` | デコードが不完全な場合、または検証が必要な場合は、元のアクション バイト。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | パス/GUID の不一致、間違った値のタイプ、切り捨てられたアクション構造、または欠落している部分があります。 |

## 調査戦略 { #investigation-strategy }

1. 最初に、非表示/矛盾したタスクと非 Microsoft パスを確認してください。
2. 引数、作業ディレクトリ、環境拡張、COM 解決など、有効なコマンドを再構築します。
3. レジストリ レコードをタスク XML と比較し、参照された実行可能ファイルにハッシュ/タイムスタンプを付けます。
4. `TaskName`/GUID/時間をタスク スケジューラ イベント、セキュリティ 4698/4702、プロセス ログ、Prefetch、および MFT/USN と関連付けます。
5. 攻撃者がタスクを削除したときに削除/回復されたレジストリ データを検索します。

## さらに詳しい情報 { #further-details }

- [Microsoft: タスク スケジューラ スキーマ](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft: タスク スケジューラ イベント ログ](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 スケジュールされたタスク](https://attack.mitre.org/techniques/T1053/005/)
