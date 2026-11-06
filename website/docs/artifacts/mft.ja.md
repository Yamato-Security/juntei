# マスター ファイル テーブル (`$MFT`) { #master-file-table-mft }

## MFT とは { #what-the-mft-is }

NTFS は、マスター ファイル テーブル内のレコードを持つすべてのファイルとディレクトリを表します。レコードには、名前、親参照、タイムスタンプ、サイズ、フラグ、セキュリティ識別子、再解析情報、データ ストリーム メタデータなどの属性が含まれます。小さなファイル コンテンツと代替データ ストリームは **resident** になる可能性があります。これは、そのバイトが MFT レコード自体の内部に格納されることを意味します。

MFT は、ボリューム全体にわたって割り当てられたレコードと削除されたレコードをカバーするため、基礎となります。通常のディレクトリの参照ではファイルが表示されなくなった場合でも、ファイルの存在、リンクの場所、削除されたかどうか、メタデータがどのように変更されたかを表示できます。

<span class="evidence-strength">この証拠から判断できること:</span> 表示されたメタデータを持つ NTFS ファイルレコードが収集時点で存在した、または未割り当てレコード内に残っていました。MFT 単独では、実行やユーザーの意図を証明できません。

## 保存場所と収集 { #location-and-collection }

```text
C:\$MFT
```

`$MFT` は保護された NTFS メタデータです。生の NTFS アクセスが可能なフォレンジック取得またはトリアージ ツールを使用して収集します。通常、通常のユーザーモードのコピーではそれを読み取ることはできません。可能な場合は、`$J`、`$LogFile`、およびボリューム メタデータと組み合わせます。

## 分析時の注意点 { #analyst-cautions }

- 削除されたレコードは部分的に上書きしたり、レコード番号を再利用したりできます。
- `SN`は、同じエントリ番号の世代を区別します。エントリ **および** シーケンスと一致します。
- `$STANDARD_INFORMATION` (`0x10`) および `$FILE_NAME` (`0x30`) タイムスタンプの更新動作は異なります。どちらも操作の影響を受けないわけではありません。
- `SI<FN`、`uSecZeros`、および `Copied` はリードであり、自動タイムスタンプ判定ではありません。
- ハード リンクは、1 つのファイル レコードに対して複数の `$FILE_NAME` 属性とパスを生成します。
- NTFS は、複数の名前 (DOS 8.3 を含む)、代替データ ストリーム、および拡張レコードを保存できます。
- アクセス時の更新は無効になったり、遅延したり、統合されたりする場合があります。
- フルパスは親レコードから再構築されます。 `[Orphaned]`、`[Unknown]`、または `PathUnknown` は、リテラルのディレクトリ名ではなく、祖先の欠落/再利用/破損を示します。
- FILE レコードの **slack** (使用中のサイズを超えたバイト数) は、以前に保持していた名前、つまり、インデックスがインデックス割り当てに移動された小さなディレクトリのインデックス エントリ (ほとんどはまだそこで使用されています)、以前の名前、または再利用されたレコードの前の占有者を保持できます。回復された名前には `$STANDARD_INFORMATION` がなく、名前のエントリ ヘッダーが上書きされている可能性があります。使用中のレコードのサイズまたは属性がそのスラックがどこから始まるかを示していない場合、レコードからは何も回復されず、その行の `ParseErrors` は `slack not read` と表示され、その理由も示されます。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $MFT files
  -f, --file <FILE>      Parse one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse selected entry numbers (example: 0-15,30)

Output:
  -o, --output <PATH>                   Save records
  -t, --output-type <TYPE>              csv, ys-ntfs-parser-csv, json, jsonl
  -e, --extract-resident-streams <DIR>  Extract resident $DATA streams

CSV Output:
  -a, --all-times      Include every $FILE_NAME time
  -s, --short-names    Include DOS 8.3-name rows
      --cutoff <DATE>  Keep recent SI-created or SI-modified records

Records:
  -x, --recover-slack  Recover file names from FILE record slack

Resident Data:
  -i, --include-resident-data      Put resident $DATA in the CSV
  -E, --resident-extensions <EXT>  Only these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  Only data up to this size

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail
  -L, --list-directory <ENTRY|PATH>  List a directory's contents
```

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff '2026-10-20T00:00:00Z' -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 32000-32100 | jq .
juntei parse-ntfs mft -f '$MFT' -x -o mft-with-slack.csv
juntei parse-ntfs mft -f '$MFT' -i -E ps1,bat,vbs,js -o scripts.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Users\alice\Downloads'
```

`-e` で抽出された常駐ストリームを決して実行しないでください。 Juntei が `.dontrun` という名前を付けているのには理由があります。 `-i` の常駐データ列は同じバイトであるため、信頼できないコンテンツとして扱います。

`-D` は 1 つのエントリを出力します。ヘッダー、すべての属性 (データ実行を含む)、および `-t json -x` が書き込む際のスラック内の名前、そしてレコードのバイトです。 `-L` は、ディレクトリ内の使用中および削除されたすべての名前を、そのエントリとシーケンス番号、タイプ、サイズ、および変更時刻とともにリストします。ディレクトリは、エントリ (`30`、`0x1E`、`30-1`) またはパス (`\Windows\System32`、`C:/Windows`、大文字と小文字は区別されません) によって指定されます。

## CSV 出力フィールド { #csv-output-fields }

| フィールド | 解釈 |
|---|---|
| `EntryNum` (Entry Number) | MFT レコード番号。 `SN`と組み合わせてください。レコード番号は再利用されます。 |
| `SN` (Sequence Number) | NTFS 参照によって使用される生成カウンター。不一致があると、関係が古くなったり削除されたりする可能性があります。 |
| `InUse` | 収集時の割り当てフラグ。 `false` は、レコードが解放/削除されていることを意味しますが、すべてのバイトがそのままであることを意味するわけではありません。 |
| `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number) | `$FILE_NAME` からの親ディレクトリ参照。両方の部分を検証します。 |
| `ParentPath` | 親パスを再構築しました。不明または孤立したマーカーは、未解決の祖先を明らかにします。 |
| `File` (Filename) | 現在の行の `$FILE_NAME` またはストリームからの名前。 1 つのレコードに複数の行を含めることができます。 |
| `Ext` (Extension) | 名前から解析された便利な拡張子。信頼できるファイルタイプの決定ではありません。 |
| `Size` (File Size) | 行/ストリームに関連付けられた論理サイズ。スパース/圧縮された割り当ては異なる場合があります。 |
| `RefCnt` (Reference Count) | NTFS ハードリンク参照カウント。生き残っている名前の数と慎重に比較してください。 |
| `ReparseTarget` | ジャンクション/シンボリックリンクなど、サポートされている再解析ポイントのデコードされたターゲット。パスはリダイレクトされる場合があります。 |
| `Dir` (Directory) | レコードにはディレクトリフラグがあります。 |
| `HasADS` (Has Alternate Data Streams) | レコードには名前付きの代替データ ストリームが含まれています。これは悪意のあるものではありません。 |
| `IsADS` (Is Alternate Data Stream) | この行は、名前のないファイル データではなく、名前付きの代替データ ストリームを表します。 |
| `SI<FN` (Standard Information Before File Name) | SI の作成は、別の FN 作成時間よりも前に行われます。コピー/タイムスタンプと正当な原因を調査します。 |
| `uSecZeros` (Microsecond Zeros) | SI の作成/変更時刻は、正確にミリ秒境界に達します。弱いタイムストンピング ヒューリスティックのみ。 |
| `Copied` | 変更された時刻は作成された時刻より前になります。多くの場合、ファイル コピーと一致します。攻撃者の行動の証拠ではありません。 |
| `SI-Flags` (Standard Information Flags) | `$STANDARD_INFORMATION` ファイルの属性/フラグ。非表示/システム フラグは便利なピボットになります。 |
| `Type` (Name Type) | `$FILE_NAME` 名前空間 (Win32、DOS、POSIX、またはその組み合わせ)。名前の二重カウントを避けるために使用します。 |
| `DataModified (SI)`, `Accessed (SI)`, `MetadataChanged (SI)`, `Created (SI)` | SI 回。一般的なタイムライン フィールドですが、ユーザー モード ツールはそれらのいくつかを変更できます。 |
| `DataModified (FN)`, `Accessed (FN)`, `MetadataChanged (FN)`, `Created (FN)` | FN 回。更新方法が異なり、通常の API では変更が困難です。まだ不変ではありません。 |
| `USN` (Update Sequence Number) | レコードの SI 属性に関連付けられた最後の USN。 USN が残っている場合は、`$J` にピボットします。 |
| `LogfileSN` (Logfile Sequence Number) | NTFS `$LogFile` FILE レコード ヘッダーのシーケンス番号。低レベルのトランザクション分析に役立ちます。 |
| `SID` (Security ID) | NTFS `$Secure` へのインデックス。それ自体は Windows SID ではありません。ファイルの所有者および ACL として、[`parse-ntfs secure-sds`](secure-sds.md) の `SecurityID` と結合します。 |
| `ObjectID-FileDroid` | オブジェクト ID / 分散リンク追跡識別子 (存在する場合)。 LNK/ジャンプ リスト ドロイドと相関します。 |
| `LoggedUtilStream` | サポートされている場合は、デコードされた `$LOGGED_UTILITY_STREAM` コンテキスト。馴染みのない価値観を見直してみましょう。 |
| `ZoneID-Contents` | ZoneId やダウンロード URL などの常駐 `Zone.Identifier` コンテンツ。不在、削除、または非居住の可能性があります。 |
| `SourceFile` | 入力 `$MFT`。ホストまたはボリュームを結合する場合に必須です。 |
| `ResidentDataBase64`, `ResidentDataHex`, `ResidentDataASCII` | `-i` の場合: 行に常駐する `$DATA` (ファイルまたは代替データ ストリーム) の内容、Base64、バイト (`41-42`)、および印刷可能な場合はテキスト ASCII。データが非常駐である場合、または `-E`/`-M` によってフィルタリングされた場合は空になります。 |
| `ParseErrors` | この行に影響を与える損傷またはサポートされていない関係。フィルタリング中にこのカラムを決して破棄しないでください。 |
| `FromSlack` | `-x` の場合: FILE レコードのスラックから回復された名前の `True`。その `EntryNum` および `SN` は、名前が指すファイル (エントリ ヘッダーが上書きされたときは空) であり、その回復されたタイムスタンプは `(FN)` 列にあり、`InUse` は `False` です。 |
| `SlackRecord`, `SlackOffset` | `-x` の場合: スラックに名前が保持されている FILE レコード。内部に書き込まれたレコード番号ではなく、`$MFT` 内の位置によって (名前は基本レコードとともにリストされているにもかかわらず、拡張レコード自体のものです)、`$MFT` 内の名前のバイト オフセットによって決まります。スラック行の `ParseErrors` は、`fixup values do not match` などのレコードの問題です。回復されたバイトが影響を受ける可能性があります。 |
| `CopyOfEntryInUse` | `-x` の場合: 使用中のファイルは同じディレクトリにこの名前を持ち続けます (参照が残っている場合は同じファイルです)。使用中のファイルにはない名前が興味深いものです。削除された名前、名前が変更された名前、またはレコードの前の占有者が再利用されたものです。 |

タイムスタンプ列は MACB の順序 (変更、アクセス、変更、誕生/作成) に従います: `DataModified` SI/FN、`Accessed` SI/FN、`MetadataChanged` SI/FN、次に `Created` SI/FN。

`(SI)` は `$STANDARD_INFORMATION`、`(FN)` は `$FILE_NAME` を意味します。完全なレコードでは、これらが各時刻の通常の出所です。SI 属性がない通常のファイル名行では、既存の代替処理により、FN の作成時刻が `Created (FN)` に入り、FN の変更・レコード変更・アクセス時刻が、それぞれ `DataModified (SI)`、`MetadataChanged (SI)`、`Accessed (SI)` に入ります。復元されたスラック行では、4 種類すべての時刻が `(FN)` 列に入り、SI 列は空です。通常行の代替値は、**独立した SI の証拠ではありません**。時刻の出所を判断する際は、元の JSON 属性とスラックの出所情報を確認してください。

`-t ys-ntfs-parser-csv` (以前は `ys-mft-parser-csv`、現在も受け入れられています) は、維持されているパーサーのエントリ指向のスキーマを使用します。これらの読み取り可能な列名は、`auto-parse` の `MFT.csv` を含む、デフォルトの `-t csv` 出力に適用されます。 JSON/JSONL は、すべての属性 (常駐 `$DATA` を含む、16 進数) および `FullPath` を保持します。フラット調査 CSV が低レベルの構造を省略する場合に使用します。 `-x` の場合、各 JSON エントリには `SlackFileNames` も含まれます。

## 調査戦略 { #investigation-strategy }

1. 時間枠をフィルタリングしますが、削除されたレコードと両方のタイムスタンプ セットは保持します。
2. フルパス、疑わしい二重拡張子、ADS、ステージング ディレクトリ、既知のハッシュ/名前を検索します。 `-x` には、使用中のファイルにないスラック名 (`CopyOfEntryInUse` `False`) を含めます。 `-i -E ps1,bat,vbs,js` を使用すると、CSV から小さなスクリプトを直接読み取ります。
3. エントリ/シーケンスと `USN` を USN ジャーナルにピボットします。
4. Prefetch/LNK/ジャンプ リスト MFT 参照をターゲット レコードと比較します。
5. 削除を Defender 隔離または USN `FILE_DELETE`/名前変更アクティビティと関連付けます。

## さらに詳しい情報 { #further-details }

- [Microsoft: マスター ファイル テーブル](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft オープン仕様: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [ys-ntfs-parser の mft クレート](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/mft)
- [Omer Ben-Amram の mft パーサー](https://github.com/omerbenamram/mft)
- [libfsntfs 形式のドキュメント](https://github.com/libyal/libfsntfs/tree/main/documentation)
