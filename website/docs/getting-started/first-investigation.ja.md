# 最初の調査 { #first-investigation }

このワークフローは、`evidence/` でオフライン優先順位付けコレクションを受信したことを前提としています。

## 1。ソースを保存する { #1-preserve-the-source }

検証済みのコピーから作業します。ハッシュ、収集時間、タイムゾーン、ホスト、ボリューム、および収集ツールを記録します。 Juntei は証拠を読み取りますが、常駐ストリームや隔離抽出などのオプションにより新しい出力ファイルが作成されます。これらを別の分析ディレクトリに誘導します。

## 2。持っているものを棚卸しする { #2-inventory-what-you-have }

[Collecting Evidence](collection.md) でパスを探します。欠落しているアーティファクトは正常です。コレクションにレジストリ ハイブが含まれているからといって、コレクションにトランザクション ログが含まれていると想定しないでください。

## 3。最初にサマリーを実行する { #3-run-summaries-first }

多くのコマンドは、出力ファイルが要求されない場合に簡潔な概要を出力します。

```bash
juntei parse-file prefetch -d evidence/
juntei parse-registry shimcache -d evidence/
juntei parse-file defender-quarantine -d evidence/
```

レコードを信頼する前に、損傷、データベースのダーティ、および未適用のログの警告を読み取ります。

複数コンピューターのコレクション内のサポートされているすべてのアーティファクトを 1 回の操作で解析するには、次を使用します。

```bash
juntei auto-parse -d evidence/ -o analysis/
```

ホスト検出、並列処理、出力レイアウト、および SIEM 取り込みの統合 JSONL については、「[自動コレクション解析」](../commands/auto-parse.md) を参照してください。

## 4。構造化された出力を保存する { #4-save-structured-output }

```bash
mkdir -p analysis
juntei parse-file prefetch -d evidence/ -T -o analysis/prefetch.csv
juntei parse-ntfs mft -d evidence/ -o analysis/mft
juntei parse-ntfs usn-journal -d evidence/ -o analysis/usn
juntei parse-registry services -d evidence/ -o analysis/services.jsonl -t jsonl
```

スクリプトおよびネストされた値には JSONL を、スプレッドシートまたはタイムライン ビューアには CSV を推奨します。

## 5。信頼できる識別子からピボットする { #5-pivot-from-a-reliable-identifier }

ファイル名、完全なパス、SHA-1/SHA-256、ユーザー SID、MFT エントリ/シーケンス、タスク GUID、サービス名、またはタイムスタンプ ウィンドウで開始します。フィールドのみ大文字と小文字を正規化します。 Windows は大文字と小文字を区別せずに処理します。

```bash
juntei parse-file prefetch -d evidence/ -t jsonl \
  | jq 'select(.ExecutableName? | ascii_downcase == "suspicious.exe")'
```

## 6。相関させる { #6-correlate }

[Cクロスアーティファクトワークフロー](../resources/workflows.md)を使用します。実際のシーケンスは次のとおりです。

1. 実行時間と最近の実行時間は Prefetch。
2. `$MFT` および USN 作成、パス、名前変更、および削除のジャーナル。
3. 過去のプレゼンスとメタデータの場合は Amcache および ShimCache。
4. ユーザー/アプリケーション コンテキストの LNK、Jump Lists、RecentDocs、および SRUM。
5. 永続化のためのレジストリ サービスと TaskCache。
6. 検出、削除、発信元、ハッシュのための Defender 隔離。

## 7。文書の再現性 { #7-document-reproducibility }

ソース パス、ソース ハッシュ、Juntei バージョン、正確なコマンド、出力ハッシュ、システム タイム ゾーン、およびその後のフィルタリングを記録します。生成された証拠出力をサイレントに編集しないでください。
