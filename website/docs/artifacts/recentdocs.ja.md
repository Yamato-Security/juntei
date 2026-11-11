# RecentDocs { #recentdocs }

## RecentDocsとは { #what-recentdocs-is }

Windows エクスプローラーは、各ユーザーの `NTUSER.DAT` に最近使用したファイルとフォルダーのリストを保存します。ルート リストには、さまざまなタイプの最近のアイテムが含まれます。サブキー グループ拡張子 (`.docx` など) と `Folder` リスト。 `MRUListEx` は、番号付きの値の順序を指定します。

RecentDocs は、後で削除または移動されたファイルなど、ユーザーが操作したドキュメントを識別するのに役立ちます。値には、ユーザーの最近使ったフォルダーで使用されるショートカット名 Windows を含めることもでき、これを LNK 証拠と関連付けることができます。

<span class="evidence-strength">この証拠から判断できること:</span> Explorer が、そのユーザーの RecentDocs リストにこの項目を記録しました。キーの時刻が示すのは、その特定のリスト内で最も新しい項目だけです。

## 場所 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

すべての関連プロファイルの `NTUSER.DAT` とそのトランザクション ログ、およびユーザーの最近の LNK ディレクトリを収集します。

## 分析時の注意点 { #analyst-cautions }

- `MRUListEx` は、すべてのアイテムのタイムスタンプではなく、順序を与えます。
- キーの最終書き込み時刻は、最新の更新に対応します。 MRU 位置 0 のみが、そこから派生した防御可能な `LastOpenedUTC` を取得します。
- 項目はルート リストと拡張子リストに表示されます。 2 つのオープンとして二重にカウントしないでください。
- アプリケーションとエクスプローラーの動作は異なる場合があります。一部のアクセス方法では RecentDocs を更新できない場合があります。
- クリーンアップ/プライバシー ツールとリスト サイズ制限により、古い証拠が削除されます。
- ファイル名は攻撃者によって制御される可能性があり、コンテンツ タイプの証拠にはなりません。
- ユーザーの推論は、プロファイル パスまたはシェル フォルダー データから得られます。空のユーザー名または曖昧なユーザー名を確認します。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry recent-docs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recent-docs -d evidence/ -t jsonl \
  | jq 'select(.Extension == ".docx" or .Extension == ".pdf")'
```

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host` | 利用可能な場合、コレクション/システム コンテキストから派生したホスト。 |
| `User` | プロファイル所有者はパスまたはシェルフォルダーのパスから推測されます。空のときは不確実性を保ちます。 |
| `SourcePath` | `NTUSER.DAT`を入力します。ユーザーを結合する場合に必須です。 |
| `Extension` | サブリスト (`.docx`、`Folder` など);ルート RecentDocs リストの場合は null。 |
| `MRU-Position` | ゼロはこのリスト内で最新のものです。これは相対的な順序であり、経過時間ではありません。 |
| `ValueIndex` | `MRUListEx` によって参照される番号付きのレジストリ値。構造ピボットのみ。 |
| `Filename` | デコードされたアイテム名。 LNK、ジャンプ リスト、MFT、またはアプリケーション データを介してフル パスを関連付けます。 |
| `LinkName` | シェル項目にエンコードされたショートカット名。多くの場合、ユーザーの最近使ったディレクトリにあります。 |
| `KeyLastWrittenUTC` | リストキーが最後に変更された時刻。すべての行ではなく、リストに適用されます。 |
| `LastOpenedUTC` | MRU 位置 0 にのみ設定されます。キー時間はその解釈をサポートします。 |
| `ValueDataRaw` | デコードが失敗した場合、または証拠の検証が必要な場合の生のバイト。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ ログが再生されたかどうか、およびどのログが再生されたか。 |
| `Problems` | MRU 値の欠落、リストにない値、間違ったタイプ、またはデコードの損傷。 |

## 調査戦略 { #investigation-strategy }

1. 各ユーザーのルートと拡張子のリストを分離します。同じアイテムの重複を慎重に削除してください。
2. 順序付けには MRU 位置を使用し、タイムスタンプ付きの要求には `LastOpenedUTC` のみを使用します。
3. `LinkName`/`Filename` を最近の LNK、ジャンプ リスト、`$MFT`、および USN データと照合します。
4. おとり名、二重拡張子、ネットワーク/削除可能なターゲット、および最初のアクセスに近いドキュメントを確認します。
5. 実行が重要な場合は、アプリケーション Prefetch/SRUM とイベント ログを関連付けます。

## さらに詳しい情報 { #further-details }

- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
- [libyal libfwsi](https://github.com/libyal/libfwsi)
- [SANS Windows フォレンジック分析ポスター](https://www.sans.org/posters/windows-forensic-analysis/)
