# Jump Lists { #jump-lists }

## Jump Listsとは { #what-jump-lists-are }

Jump Lists は、タスクバーとスタート メニューのアプリケーション エントリから表示されるメニューに戻ります。自動 Jump Lists は、アプリケーションによって使用されるアイテムの Windows 管理の履歴です。カスタム Jump Lists は、アプリケーション定義のカテゴリとタスクです。どちらも、ターゲット パス、時刻、ボリューム/ネットワーク コンテキスト、MFT 参照、起動引数を含む LNK 構造を埋め込むことができます。

自動記録により、アプリケーションによって開かれたファイルやフォルダー、最終使用順序/時間、インタラクション数、固定状態が明らかになります。カスタム リストは、悪意のあるコマンド引数を含む、「最近閉じた」ページやアプリケーション タスクを公開する可能性があります。

<span class="evidence-strength">この証拠から判断できること:</span> 対象のアプリケーションが、この項目やタスクを Jump Lists に記録、または表示しました。挙動はアプリケーションによって異なるため、実際の実行やアクセスは別の証拠で裏付けてください。

## 保存場所と収集 { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

16 進数のファイル名のプレフィックスが AppID です。両方のディレクトリを収集し、ユーザー プロファイル パスを保持します。自動リストは、`DestList` ストリームと番号付き LNK ストリームを含む OLE 複合ファイル コンテナーです。カスタム リストは独自のカテゴリ/リンク形式を使用します。

## 分析時の注意点 { #analyst-cautions }

- AppID の説明は既知のマッピングから取得されます。不明な AppID 自体は疑わしいものではありません。
- `LastModified` / 最後に使用されたフィールドは DestList エントリに属します。埋め込まれたターゲット時間は異なります。
- インタラクション数はサポートされているバージョンにのみ存在し、完全なオープン数ではありません。
- 固定すると、通常の最新性を超えてアイテムが保持されます。クリーンアップまたはアプリケーション設定により、他の設定が削除される可能性があります。
- リストには、古い/未接続のストリームや、以前の長いカスタム リストのバイトが含まれる場合があります。
- 埋め込まれた LNK 引数とパスは攻撃者によって制御されます。アプリケーションの関連付けはコンテキストであり、信頼ではありません。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-list-csv
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

`auto-parse` の場合、CSV テーブルは各コンピュータの直下に `FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` および `FileAndFolderOpening/JumpLists-CustomDestinations.csv` になります。そのカテゴリの共有ログは `JumpLists-logs.txt` です。マニフェストには、`OutputFiles` の正確なパスがリストされています。スタンドアロン CSV エクスポートでは、選択したディレクトリに `AutomaticDestinations.csv` と `CustomDestinations.csv` が保持されます。 JSON および JSONL 自動エクスポートは、単一の `JumpLists.json` または `JumpLists.jsonl` ファイルです。レコード構造は変更されません。

## AutomaticDestinations CSV フィールド { #automaticdestinations-csv-fields }

| フィールドグループ | フィールドと解釈 |
|---|---|
| ソース | `SourceFile`、`SourceCreated`、`SourceModified`、`SourceAccessed`: ジャンプ リスト コンテナーとそのファイル システム時間。 |
| アプリケーション | `AppId`、`AppIdDescription`: ファイル名 AppID と既知のアプリケーション マッピング。未知のハッシュから ID を推測しないでください。 |
| コンテナ | `HasSps`、`DestListVersion`、`LastUsedEntryNumber`: DestList の構造/バージョンとヘッダー コンテキスト。 |
| 注文 | `MRU`、`EntryNumber`: 最後に使用された位置と 16 進数のエントリ/ストリーム識別子。 |
| メタデータを使用する | `CreationTime`、`LastModified`、`InteractionCount`、`PinStatus`、`PinPosition`: DestList のライフサイクル/使用/ピンの状態。バージョンごとに解釈します。 |
| ホスト/パス | `Hostname`、`MacAddress`、`Path`、`DisplayPath`: ターゲットの場所と追跡コンテキスト。 Host/MAC は唯一の帰属ではありません。 |
| トラッキングID | `FileBirthDroid`、`FileDroid`、`VolumeBirthDroid`、`VolumeDroid`: 移動されたターゲットと埋め込まれた LNK 追跡を関連付けます。 |
| 注意事項 | `Notes`: 接続されていないストリーム、復元された残り物、またはその他のパーサー コンテキストについて通知します。 |
| 埋め込み型 LNK | `TargetCreated`、`TargetModified`、`TargetAccessed`、`FileSize`、`RelativePath`、`WorkingDirectory`、`FileAttributes`、`HeaderFlags`、`DriveType`、 `VolumeSerialNumber`、`VolumeLabel`、`LocalPath`、`CommonPath`、`TargetIDAbsolutePath`、`TargetMFTEntryNumber`、`TargetMFTSequenceNumber`、`MachineID`、`MachineMACAddress`、 `TrackerCreatedOn`、`ExtraBlocksPresent`、`Args`: [LNK ショートカット](lnk.md) の説明に従って解釈します。 |
| 追加のLNK | `NetworkPath`、`Name`、`IconLocation`: 埋め込みショートカットのネットワークおよびプレゼンテーション フィールド。 |
| 検証 | `Problems`: ストリームの欠落、切り捨て、カウントの不一致、またはリンクの損傷。 |

## CustomDestinations CSV フィールド { #customdestinations-csv-fields }

カスタム出力は `SourceFile`、`SourceCreated`、`SourceModified`、`SourceAccessed`、`AppId`、`AppIdDescription`、および `EntryName` (カテゴリ名または `Tasks`) で始まり、その後に埋め込まれたすべての出力が続きます上記の LNK フィールド、次に `Notes`、`NetworkPath`、`Name`、`IconLocation`、および `Problems`。

JSON/JSONL は、DestList ヘッダー、エントリごとのスコア/チェックサム/プロパティ ストア、`DestListPropertyStore`、複合ファイル ストリーム、カスタム カテゴリ、最後のカテゴリの後のバイト、および埋め込まれたすべての LNK のネストされたフィールドをさらに保持します。

## 調査戦略 { #investigation-strategy }

1. ユーザーとソースのプロファイルを特定し、AppID をアプリケーションに慎重にマッピングします。
2. 自動エントリを `LastModified` および `MRU` で並べ替えます。個別の固定アイテム。
3. パス、ネットワーク共有、リムーバブル ボリューム シリアル、MFT 参照、および引数を確認します。
4. `Notes` および未接続/残されたリンクを検査します。削除されたアイテムや以前のアイテムはそこに残っている可能性があります。
5. RecentDocs、スタンドアロンの最近の LNK、Prefetch、SRUM、`$MFT`、およびイベント ログと関連付けます。

## さらに詳しい情報 { #further-details }

- [Microsoft: 複合バイナリ ファイル形式 (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft: シェルリンク形式 (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-ジャンプリスト-パーサー](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [JumpList パーサー テスト/リファレンス プロジェクト](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
