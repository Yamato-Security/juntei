# YARAスキャン { #yara-scan }

Rust 製のクロスプラットフォームエンジンで収集済みファイルをスキャンします。Windows、macOS、Linux に対応しています。ファイルはデータとして読み取るだけで、実行することはありません。アーティファクトの自動解析とは独立したコマンドです。

## ルールを選ぶ { #choose-rules }

既定の Core は精度と性能を重視しています。Extended はハンティングの対象を広げます。Full は最も広い範囲を扱い、誤検知が増える可能性があります。

```bash
juntei yara-scan -d evidence/ -o yara-core/
juntei yara-scan -d evidence/ --ruleset extended -o yara-extended/
juntei yara-scan -d evidence/ --ruleset full -o yara-full/ -t jsonl
juntei yara-scan -d evidence/ --rules local-rules/ -o yara-offline/
```

`--rules <FILE_OR_DIRECTORY>`: UTF-8 のローカルルールファイルまたはディレクトリを指定します。ダウンロードは行いません。ディレクトリは再帰的に探索します。include は指定したルールディレクトリ内で一度だけ展開し、依存順序を維持します。不正・未対応のルールは黙って除外せず、コンパイルを停止します。

## 制限と対象範囲 { #limits-and-scope }

復元した隔離ペイロードを含む通常ファイルを対象とします。シンボリックリンクと特殊ファイルはスキップします。アーカイブやディスクイメージの展開、ライブメモリのスキャンは行いません。安定した抽出済みファイルを使い、証拠の外側に新規の出力ディレクトリを指定してください。

- `-T, --threads <N>`: 並列スキャン数です。既定は CPU 数で、最大 8 です。
- `--timeout <SECONDS>`: ファイルごとの評価上限秒数です。既定は 60 です。
- `--max-file-mib <MIB>`: 任意のファイルサイズ上限（MiB）です。既定は無制限です。個別指定がなければ MFT にも適用します。明示した上限を超えるファイルはスキップとして報告します。
- `--rules-release YYYYMMDD`: Forge のリリース日、または既定の latest です。再現用にソースとハッシュを保存します。

## 結果 { #results }

- `YaraDetections.csv` / `YaraDetections.jsonl`: ルールとファイルの一致です。相対パス、証拠ホスト名、ハッシュ、メタデータ、限定した一致位置を含みます。一致した内容自体は出力しません。
- `YaraFiles.csv` / `YaraFiles.jsonl`: スキップ、読み取りエラー、タイムアウト、途中結果を含む、ファイルごとの対象範囲です。
- `Summary.json`: 完了状態、設定、件数、エンジンの版、ソースのハッシュです。
- `Rules.yar`: ダウンロードした元のルール、またはローカルルールを展開したスナップショットです。
- `CompilationWarnings.txt` / `CompilationError.txt`: コンパイル警告、またはコンパイル失敗時のエラーです。

終了コードは、0 が完了、3 が対象範囲の未完了、2 が準備・出力エラー、130 がキャンセルです。一致は調査の手がかりであり、侵害の証明ではありません。一致がないことも安全の証明にはなりません。ルール名とアーティファクトの文脈を確認してください。隔離されたペイロードを識別するには、事前に復号しておく必要があります。

## MFTの一致と復元 { #mft-recovery }

MFT は一つずつ、既定のサイズ上限なしで、2 段階で処理します。まず生のバイト列を他のファイルと同じようにマップしてスキャンし、次にすべての FILE レコードをファイルから 1 レコードずつ読み取って、各レジデント DATA ストリームを独立したファイルとしてスキャンします。この段階は生スキャンの結果に依存しません。ストリーム単独で一致した内容だけを復元します。非レジデントの内容は MFT にバイト列がないため、出力せずに数えます。破損レコードは報告し、対象範囲を未完了にします。

`--max-file-mib <MIB>` → `--max-mft-mib <MIB>` · `--max-mft-recoveries <N>`

復元数の既定上限は、実行ごとに 1000 ストリームです。制限への到達、破損レコード、タイムアウトは明示します。保存名は隔離用の拡張子で終わり、証拠側のディレクトリ構成は再作成しません。

`YaraMFTResults.csv` / `YaraMFTResults.jsonl` · `RecoveredMFT/` · `.quarantined`

## 詳細情報 { #further-details }

- [エンジン: Boreal](https://github.com/vthib/boreal)
- [ルールパッケージ: YARA Forge](https://github.com/YARAHQ/yara-forge)
- [詳しい使い方とフィールド解説（英語）](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.md)
- [詳しい使い方とフィールド解説（日本語）](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.ja.md)
