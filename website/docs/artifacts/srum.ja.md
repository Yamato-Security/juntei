# SRUM { #srum }

## SRUMとは { #what-srum-is }

システム リソース使用量モニター (SRUM) は、アプリケーションとサービス リソースの使用量の ESE データベースを保存します。 Windows のバージョンに応じて、ネットワーク バイト、インターフェイス/プロファイル識別子、接続、CPU 時間、フォアグラウンド/バックグラウンド アクティビティ、ディスク アクティビティ、エネルギー使用量、プッシュ通知、およびアプリケーション タイムライン データが記録されます。エントリは、`SruDbIdMapTable` を通じてアプリケーション ID およびユーザー SID に関連付けられます。

SRUM は、実行可能ファイルが削除された場合に特に役立ちます。そのアプリケーション ID、ユーザー、およびリソース使用行は、ファイルよりも存続する可能性があります。ネットワーク合計は漏洩仮説を裏付ける可能性がありますが、SRUM はパケット キャプチャではなく、一般にすべてのリモート エンドポイントや転送されたファイルを識別するわけではありません。

<span class="evidence-strength">この証拠から判断できること:</span> Windows が、テーブルの計測期間におけるリソース使用を、このアプリケーションまたはアカウントに関連付けて記録しました。

## 保存場所と収集 { #location-and-collection }

```text
C:\Windows\System32\sru\SRUDB.dat
C:\Windows\System32\config\SOFTWARE   (context for account and Wi-Fi names)
```

SRUM は Windows 8 以降に存在します。 `SRUDB.dat` を一貫してコピーします。理想的にはスナップショットから、またはフォレンジック コレクターを使用します。また、`sru` ディレクトリ内の `SOFTWARE` および ESE トランザクション/ログ ファイルも収集します。

対応する SOFTWARE ハイブは ESE データベースとは別のものです。ダーティ状態の場合、Juntei は隣接する `SOFTWARE.LOG`、`.LOG1`、`.LOG2` のレジストリトランザクションログをメモリ上で自動再生するため、これらも収集してください。復元の詳細と警告が表示されます。ログが欠落している場合、アカウント名や Wi-Fi 名が不完全または古い可能性があります。この処理では SRUM データベースのログは再生しません。

!!! warning "ダーティなデータベース"
    Juntei は、収集した `SRUDB.dat` がダーティ状態であっても読み取り、その状態を報告します。現在は `SRU*.log` を再生しないため、ログにしか存在しない変更は含まれません。ダーティ状態のデータベースを、完全な記録であるかのように説明しないでください。

## 分析時の注意点 { #analyst-cautions }

- 多くの行は、正確なアクション時間ではなく、一般的に 1 時間ごとの境界付近での定期的な集計を表します。
- バイト合計は、それ自体でコンテンツ、意図、プロトコル、または宛先を識別するものではありません。
- アプリケーション ID は、パス、サービス、パッケージ化されたアプリ、またはオブジェクトにすることができます。 `AppType` を調べます。
- ユーザー名と Wi-Fi 名は、一致する `SOFTWARE` ハイブからエンリッチメントされます。名前が欠落していても、基礎となる ID が存在しないことを意味するわけではありません。
- `null`、ゼロ、列が存在しない、サポートされていないタイプ、および破損した値は、さまざまな状態です。
- プロバイダーのスキーマは、Windows バージョンによって異なります。不明なテーブルと元の列名を保持します。
- バックグラウンド Windows サービスは、正当に大量のリソース合計を生成します。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-ese srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SRUDB.dat
  -f, --file <FILE>      Parse one SRUDB.dat
  -r, --software <FILE>  Matching SOFTWARE hive for names

Output:
  -o, --output <PATH>        Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>   csv, json, or jsonl
      --include-value-state  Add per-value read state and raw bytes to JSON/JSONL
```

```bash
juntei parse-ese srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-ese srum -f SRUDB.dat -t jsonl \
  | jq 'select(.Provider == "NetworkUsages" and (.BytesSent // 0) > 100000000)'
juntei parse-ese srum -f SRUDB.dat -t jsonl --include-value-state \
  | jq '{Provider, Timestamp, ValueState, Raw}'
```

## 出力テーブル { #output-tables }

CSV は、`SruDbIdMapTable` に対して `SRUM-IdentityMap.csv` を作成し、プロバイダーごとに 1 つのテーブルを作成します。既知の出力名には次のものがあります。

| プロバイダー/テーブル | 調査での用途 |
|---|---|
| `NetworkUsages` | アプリケーション、ユーザー、インターフェイス、プロファイルごとに送受信されたバイト数。 |
| `NetworkConnections` | 接続数/期間スタイルのアクティビティと接続の開始時刻。 |
| `AppResourceUseInfo` | フォアグラウンド/バックグラウンド CPU、ディスク、ネットワーク、および関連アプリケーションの使用状況。 |
| `AppTimelineProvider` | 終了時間フィールドと使用状況フィールドを含むアプリケーション タイムライン アクティビティ。 |
| `EnergyUsage` | 短期および長期のエネルギー/プロバイダーの測定。 `IsLt` で区別されます。 |
| `PushNotifications` | プッシュ通知アクティビティ。 |
| `VfuProvider` | `vfuprov` プロバイダー。その詳細な目的は十分に文書化されていないため、そのフィールドは慎重に解釈してください。 |
| `EnergyEstimation`, `TaggedEnergy` | バージョン固有のエネルギーおよび使用量プロバイダー。 |
| `ProcessorTime`, `SizeInBytes`, `TotalUsed`, `NetworkBytes` | Windows サーバープロバイダーテーブル。 |
| 不明なテーブル名 | Juntei は、不明なプロバイダーを GUID/テーブル名で保存し、すべてのカタログ列を出力します。 |

なぜなら SRUM カタログ主導型であり、 Windows プロバイダー列を追加するため、プロバイダー固有のフィールドの固定された有限リストは意図的に存在しません。 Juntei 発する **データベース内に存在するすべての列 Juntei の PascalCase フィールドラベル**。次のフィールドは安定/共通出力コントラクトです。

## 分類された自動解析出力 { #categorized-auto-parse-output }

`auto-parse` は、各 SRUM テーブルをその調査カテゴリに直接配置します。 CSV、JSON、および JSONL は、同じ `SRUM-` ファイル名プレフィックスを使用します。それを囲む `SRUM/` ディレクトリはありません。

| カテゴリ | CSV ファイル |
|---|---|
| `NetworkActivity` | `SRUM-NetworkConnections.csv`, `SRUM-NetworkUsages.csv`, `SRUM-NetworkBytes.csv`, `SRUM-PushNotifications.csv` |
| `ProcessExecution` | `SRUM-AppResourceUseInfo.csv`, `SRUM-AppTimelineProvider.csv`, `SRUM-ProcessorTime.csv` |
| `SystemInformation` | `SRUM-EnergyUsage.csv`、`SRUM-EnergyEstimation.csv`、`SRUM-TaggedEnergy.csv`、`SRUM-SizeInBytes.csv`、`SRUM-TotalUsed.csv`、`SRUM-IdentityMap.csv`、`SRUM-VfuProvider.csv`、および不明なプロバイダー テーブル |

キャプチャに存在するテーブルのみがエクスポートされます。ネットワークカウンターと通知記録は、主な調査用途によって配置されます。これは、すべての行がリモート接続を記述しているという主張ではありません。 ID マップはいくつかのカテゴリをサポートします。 VFU および不明なプロバイダーは、存在するだけでは実行が確立されないため、`SystemInformation` に残ります。

`SystemInformation/SRUM-logs.txt` には、パーサー診断と元の出力から最終出力へのマッピングが含まれています。どちらのマニフェストにも、`OutputFiles` 内の実際のテーブル パスがリストされています。 SRUM タスクの `Output` はコンピュータ ディレクトリであり、`Category` はその共有ログ/メタデータの場所を示します。

複数のデータベースがある場合、ファイル名には拡張子の前に `-Source1`、`-Source2` などが付けられます。ログには元のキャプチャ エイリアスが保持され、JSON/JSONL 行には `SourceFile`、`Table`、`Provider`、`Comp`、および `Artifact` の来歴が保持されます。テーブルが複数のカテゴリにコピーされることはありません。結合された `-j` SIEM タイムラインには、保持されているすべての SRUM レコードが 1 回含まれます。

スタンドアロン `parse-ese srum` は、要求された出力場所を保持します。その CSV テーブル ファイルにも `SRUM-` プレフィックスが付いています。明示的な JSON/JSONL 出力ファイル名はアナリストによって選択されたままになります。

## 共通の出力フィールド { #common-output-fields }

| フィールド | 解釈 |
|---|---|
| `SourceFile` | 入力データベース。 JSON/JSONL に存在し、ホスト間で役立ちます。 |
| `Table` | 元の ESE テーブル名/GUID。バージョンまたは不明なプロバイダーを比較するときに保持します。 |
| `Provider` | わかりやすいプロバイダー/出力名、または不明な場合は元のテーブル名。 |
| `ID` | 行の `AutoIncId`;プロセス ID ではなく、データベース行識別子です。 |
| `Timestamp` | SRUM が数値を記録/集計したとき、ミリ秒に丸められます。通常、正確なイベント時間ではありません。 |
| `EXE-Info` | `AppID` からのアプリケーション/サービス/パッケージ ID を解決しました。メインプログラムのピボット。 |
| `EXE-InfoDescription` | 複合 SRUM アプリケーション名から抽出された説明。コンテキストのみ。 |
| `EXE-Timestamp` | 複合アプリケーション ID が存在する場合、その中に組み込まれた時間。列の使用時間ではありません。 |
| `AppType` | `NormalApp`、`Service`、`ModernApp`、または `Object`。 `EXE-Info` をどのように解釈するかを制御します。 |
| `SID` | アカウント SID を `UserID` から解決しました。強力なユーザー/サービス ID ピボット。 |
| `User` | `SOFTWARE` から強化されたわかりやすいプロファイル名。 `SID` が有効な間は null になる可能性があります。 |
| `UserID`, `AppID` | `SruDbIdMapTable` への生の数値参照。未解決の名前または重複した名前のために保存します。 |
| `IsLt` | 長期の場合は `true`、EnergyUsage テーブル、短期の場合は `false`。 |
| `InterfaceLUID` | プロバイダ行からの生のネットワーク インターフェイス LUID。 |
| `InterfaceType` | デコードされたインターフェースの種類（例: `IF_TYPE_IEEE80211`）。種類を示す値であり、特定のアダプターを一意に識別する値ではありません。 |
| `L2ProfileID` | 数値ネットワーク プロファイル参照。 |
| `ProfileName` | `SOFTWARE` から強化された Wi-Fi/プロファイル名。機密性が高い可能性があり、現在の接続を証明するものではありません。 |
| `ConnectStartTime`, `EventTimestamp`, `EndTime`, `StartTime` | 既知のプロバイダー FILETIME 列が UTC にデコードされました。それらの正確な意味は、普遍的な実行時間ではなく、プロバイダーに属します。 |
| プロバイダー固有の列 | `BytesSent`、`BytesReceived` (データベース列 `BytesRecvd`)、CPU/ディスク カウンター、期間、カウント、フラグ、または推定値などのエクスポートされたプロバイダー列。プロバイダーに従ってユニットを解釈し、同じスキーマ内の行を比較します。 |

## 診断フィールド (`--include-value-state`) { #diagnostic-fields-include-value-state }

| フィールド | 解釈 |
|---|---|
| `ValueState` | データベース フィールドの `parsed`、`null`、`absent`、`unsupported`、または `damaged` へのマップ。これにより、証拠の欠如とパーサーの失敗が区別されます。 |
| `Raw` | バイトを含む値の 16 進数の元のバイト。検証と将来のデコードに使用します。 |

`SRUM-IdentityMap.csv` (`SruDbIdMapTable` から) には、`IdIndex`、`IdType`、および `IdBlob`: プロバイダ行で使用される生のインデックス、アイデンティティ カテゴリ、およびデコードされたプログラム/SID/オブジェクト テキストが含まれます。

## 調査戦略 { #investigation-strategy }

1. プログラムは `EXE-Info`、`AppType`、および `AppID` で識別されます。 `Id` と PID を混同しないでください。
2. ネットワーク行をプログラム、ユーザー、プロファイル、インターフェイス、および時間間隔ごとにグループ化します。
3. 送受信されたバイト数を通常のベースラインおよび他のホストと比較します。
4. 疑わしい間隔を Prefetch、イベント ログ、DNS/プロキシ/ファイアウォール データ、およびファイル アクセスと関連付けます。
5. 収集の制限として、ダーティ データベース ステータスと欠落しているトランザクション ログ再生を報告します。

## さらに詳しい情報 { #further-details }

- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [dissect.esedb](https://github.com/fox-it/dissect.esedb)
- [libyal libesedb](https://github.com/libyal/libesedb)
- [Microsoft: 拡張ストレージ エンジン](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
