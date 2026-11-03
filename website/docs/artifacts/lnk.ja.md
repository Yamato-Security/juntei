# LNK のショートカット { #lnk-shortcuts }

## LNK ファイルとは { #what-lnk-files-are }

Windows シェル リンク (`.lnk`) ファイルには、ターゲットと起動引数、作業ディレクトリ、アイコン、ウィンドウ状態、ターゲット メタデータ、ボリューム/ネットワーク情報、シェル名前空間項目、およびオプションの分散リンク追跡識別子が記述されます。 Windows とアプリケーションは、最近使ったアイテム、デスクトップ、スタート メニュー、その他の場所にそれらを作成します。攻撃者は悪意のあるショートカットを直接配布することもあります。

最近使ったフォルダー LNK は、リムーバブル ドライブやネットワーク共有など、存在しないターゲットとのユーザー操作をサポートできます。任意の LNK は、クリックされなくても存在できます。

<span class="evidence-strength">この証拠から判断できること:</span> 記録されたリンク先の情報を含むショートカットが作成、または更新されました。保存場所や周辺のアーティファクトを考慮して、実際の使用を裏付けるか判断してください。

## 保存場所と収集 { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\*.lnk
%USERPROFILE%\Desktop\*.lnk
%APPDATA%\Microsoft\Windows\Start Menu\Programs\...
%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\...
```

`Recent` だけでなく、ユーザー プロファイル全体と収集されたアーカイブ/電子メール コンテンツを検索します。場所によって解釈が変わるため、ソース ファイル自体の時間とパスを保存してください。

## 分析時の注意点 { #analyst-cautions }

- ターゲット時間、サイズ、MFT 参照、およびボリューム データは、LNK に保存されるスナップショットです。これらはショートカット ファイル自体のメタデータではないため、古い可能性があります。
- `MachineID`、MAC、およびトラッカーの作成時間は、分散リンク追跡コンテキストを表します。これは通常、リンクを作成/更新したマシンであり、リンクを見つけたマシンである必要はありません。
- MAC のような UUID ノードは、仮想化、ランダム化、または非ネットワーク ID から取得される可能性があります。
- `Args` は、無害な実行可能ファイル (`powershell.exe`、`rundll32.exe`、`mshta.exe`) を危険なものにする可能性があります。
- ANSI 文字列にはソース システムのコード ページが必要です。多くの日本語リンクには `--code-page 932` を使用してください。
- ショートカットは作成またはコピーできます。そのフィールドを、Prefetch、ジャンプ リスト、RecentDocs、イベント ログ、またはその他のサポートのない実行と同一視しないでください。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.lnk
  -f, --file <FILE>      Parse one shortcut
      --code-page <CP>   ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -d evidence/ --code-page 932 -o lnk-ja.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

## CSV 出力フィールド { #csv-output-fields }

| フィールドグループ | フィールドと解釈 |
|---|---|
| ソース | `SourceFile`、`SourceCreated`、`SourceModified`、`SourceAccessed`: ショートカット パスとホスト ファイル システムの時間。これらは目標タイムとは別のものです。 |
| 目標タイム | `TargetCreated`、`TargetModified`、`TargetAccessed`: LNK に記録されたスナップショット。現在のターゲットのメタデータではなく、有用な歴史的手がかり。 |
| ターゲットの基本 | `FileSize`、`RelativePath`、`WorkingDirectory`、`FileAttributes`、`HeaderFlags`: 記録されたターゲット/起動コンテキスト。引数によって意味が変わる可能性があります。 |
| ボリューム | `DriveType`、`VolumeSerialNumber`、`VolumeLabel`: 固定/削除可能/ネットワーク コンテキストを識別し、他のショートカットを関連付けます。 |
| パス | `LocalPath`、`NetworkPath`、`CommonPath`、`TargetIDAbsolutePath`: 相補的なパス表現。不一致とシェル名前空間パスを確認します。 |
| 起動動作 | `Args`、`ShowWindow`、`HotKey`、`ShimLayer`、`DarwinId`: シェルがターゲットを起動または構成する方法。 `Args` は重要です。 |
| NTFS ID | `TargetMFTEntryNumber`、`TargetMFTSequenceNumber`: リンク作成時のターゲット参照。両方とも `$MFT` に一致します。 |
| 追跡 | `MachineID`、`MachineMACAddress`、`TrackerCreatedOn`、`VolumeDroid`、`FileDroid`、`VolumeDroidBirth`、`FileDroidBirth`: 分散リンク トラッキング。帰属のみではなく、相関関係として使用してください。 |
| 追加ブロック | `ExtraBlocksPresent`: 利用可能なオプションの構造のリスト。他のどのフィールドがサポートされているかについて説明します。 |
| プレゼンテーション | `Name`、`IconLocation`、`IconIndex`、`IconEnvironment`、`EnvironmentVariables`: 表示および環境拡張されたターゲット データ。攻撃者は誤解を招く名前やアイコンを使用する可能性があります。 |
| シェルフォルダー | `KnownFolder`、`SpecialFolder`: デコードされた Windows シェル フォルダー識別子。 |
| ネットワーク | `NetworkDevice`、`NetworkProvider`: ネットワーク共有/デバイスおよびプロバイダーのコンテキスト。 |
| エンコーディング | `ConsoleCodePage`: コンソール コード ページが存在する場合は記録されます。パーサー `--code-page` とは別のものです。 |
| 追跡データ | `AppendedBytes`: 論理終了後のバイト数。予期しないデータは、密輸または不正な形式のファイルを示している可能性があります。 |
| 検証 | `Problems`: 損傷、切り捨て、不整合なブロック、またはデコードの制限。レビューに含めます。 |

JSON/JSONL は、すべてのターゲット ID 項目 (種類、名前、GUID、FAT 時間および MFT 参照)、すべての追加データ ブロック、プロパティ ストア プロパティ、コンソール設定、および生の構造関係をさらに保持します。フラットな CSV 行だけに依存するのではなく、疑わしいショートカットに使用してください。ネストされた `TargetItems` 時間は、格納されたオフセットのない FAT シェル項目クロックから取得されます。従来の `Z` レンダリングは UTC を証明しません。 `auto-parse --timezone` はそれらを保存します。 LNK ヘッダーのターゲット FILETIME とトラッカー時間は、変換可能な絶対タイムスタンプのままです。

## 調査戦略 { #investigation-strategy }

1. ターゲットフィールドを解釈する前に、ソースの場所とソース時間を調べてください。
2. ターゲット パス、`Args`、`WorkingDirectory`、および環境ブロックからの効果的な起動を再構築します。
3. ターゲット MFT リファレンス、ボリューム シリアル、ネットワーク パス、ドロイド、MachineID、および MAC を証拠全体にピボットします。
4. 実行については、Prefetch とイベント ログを確認してください。ユーザー操作については、Jump Lists/RecentDocs を確認してください。
5. 悪意のある添付ファイルの場合は、追加されたデータを保存し、LNK 自体をハッシュ/分析します。

## さらに詳しい情報 { #further-details }

- [Microsoft オープン仕様: MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [Sootmark シェル](https://github.com/Sootmark/shell)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [エリック・ジマーマンのLECmd](https://github.com/EricZimmerman/LECmd)
