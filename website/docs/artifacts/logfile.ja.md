# NTFS トランザクション ログ (`$LogFile`) { #ntfs-transaction-log-logfile }

## それは何か、そしてなぜそれが重要なのか { #what-it-is-and-why-it-matters }

NTFS は、Windows がシステム ボリュームに通常使用するファイル システムです。クラッシュから回復するために、ファイル システムのメタデータに加える変更を `$LogFile` という名前の隠しボリューム ファイルに記録します。各更新は、**redo** バイト (新しい状態) と **undo** バイト (変更をロールバックするために必要な以前の状態) を保持できます。これは、Windows イベント ログ、レジストリ `.LOG1` ファイル、および [USN ジャーナル](usn-journal.md) とは別のものです。

調査者にとって、以前の状態は特に役立ちます。元に戻すレコードには、名前変更前のファイル名、削除されたディレクトリ エントリ、またはメタデータ更新前のタイムスタンプを保存できます。一部のレコードには、収集された `$J` に存在しないレコードなど、NTFS が USN ジャーナルに書き込んだバイトも含まれています。ただし、トランザクションの多くはオペレーティング システムの日常的なアクティビティに関係しており、攻撃を説明するものではありません。

**例:** 削除されたインデックス エントリには `report.txt` が記載されています。後で追加されたエントリでは `report.txt.encrypted` について言及しています。完全なファイル参照、親、トランザクション チェーン、およびその他のアーティファクトの一致により、名前の変更がサポートされます。これら 2 つの名前だけでは、ランサムウェア、そのオペレーター、または暗号化の正確な時間を特定することはできません。

## 保存場所と収集 { #location-and-collection }

```text
C:\$LogFile
D:\$LogFile
```

すべての NTFS ボリュームには独自のログがあります。 NTFS メタデータを読み取るツールを使用して取得するか、フォレンジック イメージまたはスナップショットからエクスポートします。通常の Windows ファイルのコピーではアクセスできない可能性があります。同じボリュームの `$MFT` と `$Extend\$UsnJrnl:$J` をできるだけ近いタイミングで収集します。元のファイルとハッシュを保存します。その後の取得には、異なる範囲のアクティビティが含まれる場合があります。

`$LogFile` は **循環ログ** です。新しいレコードは古いレコードを上書きします。保存期間はログのサイズと作業負荷によって異なり、数分から数時間かかる場合もあります。ファイル名や操作が存在しないことは、それがまったく起こらなかったという証拠にはなりません。

## 記録が証明するものと警告 { #what-a-record-proves-and-cautions }

- 更新では、リカバリのためにログに記録されたメタデータ NTFS が記録されます。単一の REDO エントリは、上位レベルのアクション全体が完了したこと、または特定のユーザーがアクションを開始したことを証明するものではありません。関連するレコード、トランザクションの完了、結果として得られるファイル システム、および独立した証拠を関連付けます。
- **一般的なトランザクション タイムスタンプはありません。** 埋め込みタイムスタンプは、ファイルのメタデータを記述します。ファイルのメタデータは、古いもの、コピーされたもの、または攻撃者によって変更されたものである場合があります。 `CreatedOn` を名前変更またはタイムスタンプ変更の時刻としてインシデント タイムラインに配置しないでください。 USN レコードは別のタイムスタンプを提供する場合があります。
- LSN (ログ シーケンス番号) 順序レコード **1 つのボリュームのログ内**。これらは時刻ではないため、ホストまたはボリューム間で比較してはなりません。古い循環バッファ パスのレコードは、新しいアクティビティと一緒に存続できます。
- トランザクション テーブル スロットは再利用されます。 `TransactionId` 単独ではなく、`TransactionFirstLsn`、`TransactionId`、クライアント ID、およびソースを含むグループ。チェーンは、保持されたデータの外側を指すことができます。
- ファイル参照には、エントリ番号 **とシーケンス番号** が含まれます。 MFT エントリを再利用すると、無関係なファイルが同じファイルのように見える可能性があります。
- `$MFT` によって提供されるパスは、再構築された履歴パスではなく、**collected MFT state** を記述します。 Juntei はシーケンス番号と一致し、不明なコンポーネントは明示的に残されます。親の名前はトランザクション後に変更された可能性があり、ハード リンクでは複数の現在の名前が提供される場合があります。
- `TargetInferred` は、オープン属性マッピングがログの別の部分から取得されたことを意味します。このマッピングは特に注意して扱ってください。未解決のインデックス エントリは、そのレイアウトだけでデコードされる場合があります。生のバイトやその他のソースを使用して名前を検証します。
- 破れたページはスキップされます。読み取り可能な周囲の記録は引き続き利用可能です。 `Truncated` は、継続が欠落しているレコードをマークします。出力を完了したものとして扱う前に、パーサーの診断を確認してください。長さゼロの入力はエラーです。書き込まれていないすべて `FF` ログには、正当には行がありません。
- このパーサーは、過去の MFT を再構築するためにトランザクションを**再生するものではありません**。任意の非レジデントファイルの内容の復元や、収集したデータの実行も行いません。DAX の CRC 保護ページには対応していません。実証拠による検証は主に 4 KiB のログページを対象としています。

## Juntei で解析する { #parse-with-juntei }

```bash
# Single log, current paths from an adjacent $MFT when present
juntei parse-ntfs logfile -f '$LogFile' -o logfile.csv

# Explicit MFT from the SAME volume
juntei parse-ntfs logfile -f '$LogFile' -m '$MFT' -t jsonl -o logfile.jsonl

# Multiple collected logs; SourceFile distinguishes every input
juntei parse-ntfs logfile -d evidence -o collected-logs.csv

# Preserve the underlying payloads for low-level validation
juntei parse-ntfs logfile -f '$LogFile' --raw-data -t jsonl -o logfile-raw.jsonl

# Automatic per-computer output
juntei auto-parse -d evidence -o parsed
```

すべてのオプションについては、[ コマンド リファレンス ](../commands/filesystem.md#parse-ntfs-logfile) を参照してください。ディレクトリ検出では、`$LogFile`、`LogFile`、それらの `.bin` 形式、および `C_$LogFile` や `C_$LogFile.bin` などの名前を大文字と小文字を区別せずに受け入れます。 `-f` で渡される 1 つのファイルには、任意の名前を付けることができます。

MFT の自動ペアリングでは、エクスポートのボリューム プレフィックスが考慮されます。`C_$LogFile` は `C_$MFT` を使用し、`D_$LogFile.bin` は最初に `D_$MFT.bin` を試行し、次に `D_$MFT` を試行します。プレフィックス付きログは、フォールバックとしてプレーン `$MFT` を使用することはありません。プレフィックスのない `LogFile` は、`MFT` または `$MFT` も受け入れます。エクスポートに別の命名スキームがある場合、`-m` を `-f` とともに使用して、一致する MFT を明示的に選択します。

出力は、更新、チェックポイント、不明なクライアント レコードを含む、LSN ごとに 1 つのレコードです。 JSON は、きれいに印刷されたオブジェクトのストリームです。 JSONL には 1 行に 1 つのオブジェクトがあります。 CSV には同じフィールド名があります。配列と構造化フィールドは、CSV セル内のコンパクトな JSON です。 JSON の CSV および `null` では不明なスカラーが空です。空の配列は、デコードされた項目が存在しないことを意味するものであり、何も存在しなかったことを意味するものではありません。

1 つのログ全体とその解析されたレコードが一度にメモリに保持されます。 MFT パスを解決すると、メモリと作業が追加されます。 `--raw-data` では出力サイズが大幅に増加する可能性があります。 `auto-parse` では、`-T` を使用して、メモリに制約のあるマシン上の同時パーサー プロセスを制限します。

隣接する MFT を読み取れない場合、Juntei は警告を発し、パスなしでログをエクスポートします。 `MftSource` は null のままになります。この強化の失敗によって、読み取り可能な一次証拠が破棄されるわけではありません。 MFT の一致が見つからない、またはあいまいな場合も、概要に警告と `Logs without a matching MFT` カウントが生成されます。これらは解析失敗としてカウントされません。

## 出力フィールド { #output-fields }

### 身元、完全性、出所を記録する { #record-identity-integrity-and-provenance }

| フィールド | 意味と解釈 |
|---|---|
| `SourceFile`, `FileOffset` | 入力パスと、このレコードのヘッダーが読み取られたバイト オフセット。それらを使用してバイトを検証します。 |
| `LSN` | レコードのログのシーケンス番号。大きな値はタイムスタンプではなく整数です。 |
| `PreviousLSN`, `UndoNextLSN` | 以前のレコードとロールバック レコードへのリンク。ゼロはリンクがないことを意味します。ゼロ以外のリンクは、保持されたデータの外側を指している可能性があります。 |
| `TransactionID`, `TransactionFirstLSN` | トランザクション テーブル スロットと、このトランザクションにリンクされている最も古い LSN。スロットだけが一意ではありません。 |
| `RecordType` | `ClientRecord` (更新)、`ClientRestart` (チェックポイント)、または `Other`。 |
| `ClientIndex`, `ClientSequence` | ログクライアントの ID。通常は NTFS です。これらはユーザー アカウントまたはプロセス ID ではありません。 |
| `Flags`, `SpansPages` | 生レコード フラグと、クライアント データが別のページに続くかどうか。ページをまたぐのは正常です。 |
| `Truncated` | パーサーがこの LSN に欠落している継続を見つけた場合は True。ファイル全体のすべての損傷を要約するものではありません。 |
| `ClusterSize`, `ClusterSizeSource` | ターゲット オフセットに使用されるクラスターあたりのバイト数。ソースは `Caller`、`Checkpoint`、または `Inferred` です。既知の場合は、`--cluster-size` で実際の値を指定します。 |
| `LogVersion` | 最新の読み取り可能な再起動ページのバージョン (通常は `1.1` または `2.0`)。読み取り可能な再開ページが残っていない場合は null。 |
| `MFT-Source` | 現在のパスの強化に使用される MFT (存在する場合)。同じボリュームに属している必要があります。 |

`auto-parse` には、他のパーサーと同様に JSON/JSONL、`Comp`、`Artifact` が追加されます。タスクの `.txt` ログには診断が保存されます。マニフェストには、タスクが成功したか、失敗したか、スキップされたかが示されます。修復可能なダメージがあると、タスクが成功し、ログにゼロ以外の `Parse problems` カウントが記録されることがあります。

### 作戦と目標 { #operations-and-target }

| フィールド | 意味と解釈 |
|---|---|
| `RedoOperation`, `UndoOperation` | NTFS 操作名、または不明なコードの場合は 16 進数。 Redo は、意図した更新を記述します。 undo はその逆を表します。 |
| `TargetEntryNumber`, `TargetSequenceNumber` | ファイル レコードに対する操作の場合: 影響を受ける MFT レコード番号。シーケンス番号は推測されません。それ以外の場合: 解決されたオープン属性を所有するファイル参照。インライン ディレクトリ インデックス更新は、そのエントリで記述された子ではなく、ディレクトリをターゲットにします。 |
| `TargetAttributeType`, `TargetAttributeName` | 操作で使用されるオープン属性の数値タイプと名前。ファイル レコード操作の場合、これは通常 `$MFT` の `$DATA` です。一般的なタイプ: 16 (`$STANDARD_INFORMATION`)、48 (`$FILE_NAME`)、128 (`$DATA`)、144 (`$INDEX_ROOT`)、160 (`$INDEX_ALLOCATION`)、176 (`$BITMAP`)。 |
| `TargetPath` | ターゲットの完全な参照がわかっている場合、または不明なコンポーネントの場合の現在の MFT パス。履歴世代が不明なファイル レコード操作の場合は空です。レコード番号だけで現在の占有者を解決することはできません。 |
| `OpenAttributeEntryNumber`, `OpenAttributeSequenceNumber`, `OpenAttributePath` | 操作の書き込みに使用されるオープン属性を所有するファイル (現在の MFT パス)。ファイル レコード操作の場合、これらは影響を受けるレコードとは別に `$MFT` コンテキストを保存します。 |
| `TargetAttribute` | ログのオープン属性テーブル内のオフセット。これは、属性のタイプまたは MFT 識別子とは異なります。 |
| `TargetInferred` | ターゲット マッピングがログの別の部分から取得される場合は True。このフラグが false の場合でも、null ターゲットは解決されないままになります。 |
| `TargetVCN`, `Lcns` | ターゲット内の仮想クラスターと物理クラスターの番号。タイミングではなく低レベルの検証に役立ちます。 |
| `RecordOffset`, `AttributeOffset` | 属性/エントリとその中の更新のオフセット。 |
| `ClusterBlockOffset`, `TargetBlockSize` | オフセットとターゲットのサイズ (512 バイト単位)。 |
| `OffsetInTarget` | 影響を受けるファイル レコードまたは非常駐ストリーム内の計算されたバイト オフセット。クラスターと MFT レコードのサイズによって異なります。 |
| `RedoLength`, `UndoLength` | 宣言されたデータ長。ゼロ化操作では、それほど多くのペイロード バイトを運ばずに長さを宣言できます。 |
| `RedoBytesAvailable`, `UndoBytesAvailable` | 実際にデコードに使用できるペイロード バイト。宣言された長さと操作のセマンティクスを比較します。 |

### 便利なファイルメタデータ列 { #convenient-file-metadata-columns }

これらの列には、やり直し側から優先される 1 つの非 DOS ファイル名が表示され、元に戻すことができます。どちらの側にもファイル名がない場合、タイムスタンプ更新フィールドはやり直しタイムスタンプを優先し、次に元に戻します。ディレクトリ エントリの更新では、時間、サイズ、フラグも直接公開され、元に戻すよりもやり直しが優先されます。デコードされた `$DATA` 属性は、ファイル名が存在しない場合でもそのサイズを公開します。デコードされたファイル レコードの名前のない `$DATA` サイズは、古い可能性のあるファイル名のサイズよりも優先されます。これらは閲覧補助です。完全な前後比較には、別のサイドフィールドを使用してください。

| フィールド | 意味と解釈 |
|---|---|
| `MetadataSide` | `Redo` または `Undo`。便利なファイル名/タイムスタンプ列を提供する側を識別します。 |
| `EntryNumber`, `SequenceNumber` | デコードされたファイル名/ファイル レコード/属性メタデータ、または部分的な `$STANDARD_INFORMATION` タイムスタンプ更新によって記述されるファイル (既知の場合はシーケンスも含む)。子を識別しないディレクトリ エントリの更新、および生のサイズ/実行リストの更新の場合は空です。影響を受けるレコードには `TargetEntryNumber` を使用します。 |
| `Filename` | 優先されるデコードされた名前。 DOS エイリアスやハード リンクを含むすべての名前はサイド配列に残ります。 |
| `ParentEntryNumber`, `ParentSequenceNumber`, `ParentPath` | 親参照と現在の MFT パス。歴史的経路の再構築ではありません。 |
| `CreatedOn`, `ContentModifiedOn`, `RecordModifiedOn`, `LastAccessedOn` | UTC メタデータ時間から 100 ns。これらはトランザクションのタイムスタンプではありません。欠落または無効な時刻は空またはヌルのままになります。 |
| `FileSize`, `FileAttributes` | 選択したメタデータからの Windows フラグのサイズと数値。ディレクトリ エントリの値が古くなっている可能性があります。 `$DATA` 属性とデコードされたファイル レコードは、より直接的なサイズを提供できます。 |
| `AttrType` | デコードされた属性レコードの数値タイプ。デコードされたディレクトリ エントリ メタデータ更新の場合は 48。操作の対象となるオープン属性とは別のものです。 |

### 完全なやり直し/元に戻すメタデータとペイロード { #complete-redoundo-metadata-and-payloads }

| フィールド | 意味と解釈 |
|---|---|
| `RedoFileNames`, `UndoFileNames` | その側でデコードされたすべての `$FILE_NAME` の配列。追加/新規名と削除/古い名前が保持されます。 |
| `RedoTimes`, `UndoTimes` | 両側によって書き込まれた `$STANDARD_INFORMATION` タイムスタンプ。部分的な更新では、未変更のタイムスタンプが推測されるのではなく、存在しないままになります。 |
| `RedoDecoded`, `UndoDecoded` | `Kind` によって識別される、構造化されたファイル レコード、属性、インデックス エントリ、またはディレクトリ エントリ情報。 Null は、サポートされていない操作またはバイト数が不足していることを意味する可能性があります。 |
| `JournalWriteOffset`, `JournalWriteLength` | `$DATA:$J` を書き込む REDO データのオフセットとバイト数。これにより、USN レコードが失われた可能性のあるソースが特定されます。再構築されたジャーナルはエクスポートされません。 |
| `Checkpoint` | チェックポイントのバージョン、開始 LSN、クラスター サイズ、およびオープン属性、属性名、ダーティ ページ、およびトランザクション テーブル ダンプの LSN/長さ。 |
| `OtherRecordType` | 認識されない、またはデコードできない本体の生のクライアント レコード タイプ。 |
| `RedoData`, `UndoData`, `OtherData` | 16 進ペイロードは `--raw-data` のみ。それ以外の場合は null/空。これらは、サポートされていない構造または損傷した構造を検証するために保管してください。 |

各ファイル名オブジェクトには、ファイルと親の参照 (`Raw`、`EntryNumber`、`SequenceNumber`)、`Name`、`NameEscaped`、`Namespace`、現在の `ParentPath`、4 つのタイムスタンプ、`Size`、 `AllocatedSize`、`Attr`、`ReparseValue`。 `NameEscaped` は、エスケープで表された無効な UTF-16 を識別します。再解析値には、再解析タグまたは拡張属性サイズを指定できます。

タイムスタンプ オブジェクトは、`Utc` と整数 `Filetime` の両方を保持します。センチネル値または無効な値は、元の整数を保持しながら、NULL の UTC 表現を持つことができます。 `RedoTimes` および `UndoTimes` には、`Created`、`Modified`、`Changed`、および `Accessed` が含まれます。欠落しているメンバーの値は null です。

デコードされた `FileRecord` オブジェクトには、番号、シーケンス、ハードリンク数、フラグ、ベース参照、ヘッダーのみが存続するかどうか、および存在する属性が含まれます。デコードされた各属性は、タイプ、名前、常駐状態、フラグ、ID、サイズ、および割り当てられたサイズを保持します。 `$STANDARD_INFORMATION` は、属性、所有者/セキュリティ ID、クォータ、および USN の 4 つの時間を追加します。これらの ID は個別の解決が必要であり、ファイルを変更したユーザーの証拠ではありません。インデックス エントリには、フラグ、キーの長さ、およびオプションのファイル名が含まれます。ディレクトリ エントリの更新では、名前自体が記録されなかった場合でも、4 つの時刻、サイズ、属性、および再解析値が保持されます。

## 調査ワークフロー { #investigation-workflow }

1. 欠落している再開ページ、破損したページ、切り捨てられたレコードの診断を確認します。
2. `RedoFileNames` と `UndoFileNames` の両方で、疑わしいファイル名または拡張子を検索します。
3. エントリ **とシーケンス** の番号を照合し、トランザクションの LSN チェーンと操作を調べます。
4. 変更のやり直しおよび元に戻すタイムスタンプを比較します。定期的なコピー、抽出、復元によってもタイムスタンプが変更される可能性があります。違いはリードであり、自動タイムストンピングの判定ではありません。
5. [USN ジャーナル](usn-journal.md)、[$MFT](mft.md)、および [$I30](i30.md) と相関させます。イベント ログ [Prefetch](prefetch.md) およびユーザー アクティビティ アーティファクトを使用して、実行または帰属を確立します。

たとえば、どちらかの側に暗号化されたファイル名を持つすべてのレコードを検索します。

```bash
jq 'select(any(.RedoFileNames[]?, .UndoFileNames[]?;
  .Name | endswith(".encrypted"))) |
  {Lsn, TransactionId, TransactionFirstLsn, RedoOperation, UndoOperation,
   RedoFileNames, UndoFileNames}' logfile.jsonl
```

これはファイル名検索であり、完全なランサムウェア検出器ではありません。ジャーナル ペイロードを検査するには、`--raw-data` を使用してエクスポートし、null 以外の `JournalWriteOffset` を使用してレコードを選択します。 `RedoData` 16 進数は、そのオフセットに書き込まれたバイトを表します。これらの埋め込まれた USN レコードの解析と重複排除は別の手順です。 Juntei はまだそれらを `parse-ntfs usn-journal` 出力とマージしていません。

## さらに詳しい情報 { #further-details }

- [ys-ntfs-parser のログファイル crate](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/logfile): サポートされている構造、検証コーパス、および既知の制限事項。
- [Sootmark ntfslog](https://github.com/Sootmark/ntfslog): アップストリーム Rust 実装。
- [libfsntfs NTFS 形式のドキュメント](https://github.com/libyal/libfsntfs/tree/main/documentation): ログ ページ、修正、およびメタデータ構造。
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs): 独立した NTFS 解析の研究と実装。
- [Joakim Schicht の LogFileParser](https://github.com/jschicht/LogFileParser): 詳細なトランザクション ログ分析。
- [Stroz Friedberg の ntfs-linker](https://github.com/strozfriedberg/ntfs-linker): NTFS ソースをファイル履歴に関連付けます。
- [DFIR アーティファクト ミュージアム](https://github.com/AndrewRathbun/DFIRArtifactMuseum): 本物の Windows NTFS サンプル。
- [Velocidex go-ntfs](https://github.com/Velocidex/go-ntfs): NTFS の実装と公開テスト ボリューム。
