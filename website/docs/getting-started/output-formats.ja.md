# 出力フォーマット { #output-formats }

## ターミナルサマリー { #terminal-summaries }

`-o` または構造化された `-t` がない場合、コマンドは通常、カウント、時間範囲、ダメージ、および高価値の結果を表示します。概要は、大規模な出力を作成する前に範囲を理解するのに役立ちます。

## CSV { #csv }

CSV はデフォルトの保存形式です。表計算ツールやSQLインポート、タイムラインビューアなどに便利です。一部のパーサーは、アーティファクトに異なるレコード タイプ (Amcache、Jump Lists、および SRUM) が含まれるため、複数のテーブルを作成します。

!!! warning "証拠列をテキストとして扱う"
    パスまたはコマンド ラインは、`=`、`+`、`-`、または `@` で始めることができます。表計算ソフトでは数式として解釈される場合があります。すべてのフィールドをテキストとしてインポートするか、フォレンジック対応の CSV ビューアを使用します。 Juntei は証拠文字列を変更せずに保存します。

## フィールド名 { #field-names }

CSV 列ヘッダーと JSON/JSONL フィールド名は、Amcache および `auto-parse` マニフェストを含む PascalCase を使用します。頭字語は大文字のままです: `ProcessID`、`TimestampUTC`、`UserSID`、`SHA256`。頭字語に続く単語はハイフンで区切られます: `IP-Address`、`MFT-Entry`、`USN-Journal`。 `jq` の場合は、ハイフンで区切られたフィールドを引用符で囲みます: `.["IP-Address"]`。

デフォルトの MFT CSV では、`EntryNum`、`Ext`、`Dir`、`HasADS`、`Size`、`Created (SI)`、`Created (FN)` などの短くて読みやすい見出しが使用されます。完全な CSV スキーマについては、[MFT フィールド リファレンス ](../artifacts/mft.md) を参照してください。タイムスタンプのペアは MACB の順序に従います: DataModified、アクセス済み、MetadataChanged、作成済み。SI が FN の前にあります。 CSV ツールでスペースまたは句読点を含む見出しを引用符で囲みます。

証拠の値は元のスペルを維持します。 `Details`/`DataRaw` の下のレジストリ値名や `ZoneIdentifier` の下の `Zone.Identifier` 名など、ソースが提供する辞書キーも変更されません。 SRUM の `ValueState` および `Raw` ディクショナリは、データ列として同じラベルを使用します。正規化衝突は、フィールドを黙って削除するのではなく報告されます。

これにより、以前のエクスポート スキーマが変更されます。既存の結果を再生成し、保存されたクエリを更新します (たとえば、`.record_type` は `.RecordType` になり、Amcache `full_path` は `FullPath` になります)。人間が判読できるテキスト レポートと入力/構成形式は、既存の規則を維持しています。

## 省略されたフィールド名 { #abbreviated-field-names }

次のラベルは、自動解析とマニフェストを含む、CSV、JSON、および JSONL エクスポートに適用されます。古い結果を再生成するときに、保存されている `jq`、`xsv`、およびスプレッドシート フィルターを更新します。出力ラベルのみが変更されます。証拠の値と生のレジストリ値の名前は、元のスペルを維持します。

| 前のラベル | 現在のラベル |
|---|---|
| `ADS` | `HasADS` |
| `Application` | `App` |
| `ApplicationCompany` | `AppCompany` |
| `ApplicationCompanyRaw` | `AppCompanyRaw` |
| `ApplicationPath` | `AppPath` |
| `Arguments` | `Args` |
| `Attributes` | `Attr` |
| `AttributeType` | `AttrType` |
| `BytesRecvd` | `BytesReceived` |
| `Computer` | `Comp` |
| `ComputerName` | `CompName` |
| `ComputerNameLastWrittenUTC` | `CompNameLastWrittenUTC` |
| `Directories` | `Dir` |
| `FileName` | `Filename` |
| `MicrosecondZeros` | `ZeroU-Secs` |
| `SI-BeforeFN` | `SI<FN` |
| `UserName` | `User` |
| `Username` | `User` |
| `UserNameSource` | `UserSource` |
| `Users` | `User` |

MFT の CSV 出力には専用の列名 `File` (Filename)、`SN` (Sequence Number)、`ParentEntryNum` (ParentEntryNumber)、`ParentSN` (Parent Sequence Number)、`USN` (Update Sequence Number)、`LogfileSN` (Logfile Sequence Number)、`SID` (Security ID)、`Type` (Name Type)、`RefCnt` (Reference Count) を使用します。`auto-parse` の MFT CSV も同じ列名です。[MFT の出力フィールド](../artifacts/mft.md#csv-output-fields)を参照してください。

ネイティブ システム情報 JSON 時間は `CompNameLastWritten` (CSV: `CompNameLastWrittenUTC`) です。 `HasADS` はブール指標です。 Prefetch の `Dir` には、記録されたディレクトリのリストが含まれます。 `User` は、アーティファクト固有のタイプ (アカウント名の文字列、または USB 出力内の関連ユーザーのリスト) を保持します。実行中のアイデンティティを意味するものではありません。クエリ内の句読点を引用符で囲みます: `."SI<FN"` および `.["ZeroU-Secs"]`。

## JSON { #json }

JSON は、完全な結果を 1 つのドキュメントとして保存します。フラットな CSV では常に表現できない、配列、入れ子構造、NULL、大きな整数、および区別が保持されます。

```bash
juntei parse-file lnk -f sample.lnk -o lnk.json -t json
```

## JSON 行 { #json-lines }

JSONL は、1 行に 1 つの独立した JSON オブジェクトを書き込みます。ストリーミングも良好で、`jq` と自然に動作します。

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.StartType == "automatic") | {Host, ServiceName, ImagePath}'
```

構造化出力が標準出力に送られると、Juntei は進行状況とメッセージを標準エラーに送信するため、パイプはデータのみを受信します。

`auto-parse -j timeline.jsonl` は、分類された CSV の結果を保持しながら、完全なマルチホスト コレクション用の追加の JSONL データセットを作成します。個々の結果の形式を変更するには、`-t json` または `-t jsonl` を追加します。各レコードには `Comp` フィールドと `Artifact` フィールドがあるため、SIEM の取り込み後もソースは明確なままになります。

## UTC と精度 { #utc-and-precision }

Juntei は、形式が許可する場合、RFC 3339 を使用して、UTC で解析時間をフォーマットします。一部のアーティファクトは 100 ナノ秒の精度を持っています。その他には、秒、FAT 時間、集計間隔、または時間をまったく保存しないものもあります。数字は正確さではありません。アーティファクト ページに記載されているソース フィールドを常に理解してください。

UTC は、`2026-10-20T08:46:38.2001668Z` 形式の Windows イベント ログを使用し、利用可能な小数精度を保持します。 `auto-parse -Z "+09:00"` は、すべての結果形式とオプションの `-j` エクスポートで、明示的な固定オフセットを使用して解析されたインスタントを表示します。 Juntei を実行しているコンピューターに関係なく、デフォルトは UTC です。生の値とゾーン化されていない壁時計は記録されたままになります。 [出力タイムゾーン](../commands/auto-parse.md#output-time-zone)を参照してください。

## ダメージと欠損値 { #damage-and-missing-values }

空、`null`、`NA`、`Problems`、およびパーサー固有の状態フィールドは意味があります。これらは、アーティファクトに値が保存されていない、レコードが破損している、形式がサポートしていない、またはパーサーが解決できなかったことを意味する可能性があります。すべての空の値をゼロに置き換えないでください。

## 正規化されたタイムライン { #normalized-timeline }

[`super-timeline`](../commands/super-timeline.md) は、時刻の意味を示すラベルとソース参照を備えた共通の CSV/JSONL イベントスキーマを作成します。デフォルトでは並べ替えず、`--sort` を指定すると時系列への並べ替えと重複統合を行います。Timesketch のインポート用プロファイルにも対応しています。各パーサーのネイティブ出力と `auto-parse -j` は、既存のスキーマを保ちます。
