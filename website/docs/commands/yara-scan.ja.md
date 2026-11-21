# YARAスキャン { #yara-scan }

Rust製のクロスプラットフォームエンジンで収集済みファイルをスキャンする。Windows、macOS、Linuxに対応する。ファイルはデータとして読み取り、実行しない。アーティファクトの自動解析とは独立したコマンド。

## ルールを選ぶ { #choose-rules }

既定のCoreは精度と性能を重視する。Extendedはハンティングの対象を広げる。Fullは最も広い範囲を扱い、誤検知が増える可能性がある。

```bash
juntei yara-scan -d evidence/ -o yara-core/
juntei yara-scan -d evidence/ --ruleset extended -o yara-extended/
juntei yara-scan -d evidence/ --ruleset full -o yara-full/ -t jsonl
juntei yara-scan -d evidence/ --rules local-rules/ -o yara-offline/
```

`--rules <FILE_OR_DIRECTORY>`: UTF-8のローカルルールファイルまたはディレクトリを指定する。ダウンロードは行わない。ディレクトリは再帰的に探索する。includeは指定したルールディレクトリ内で一度だけ展開し、依存順序を維持する。不正・未対応のルールを黙って除外せず、コンパイルを停止する。

## 制限と対象範囲 { #limits-and-scope }

復元した隔離ペイロードを含む通常ファイルを対象とする。シンボリックリンクと特殊ファイルはスキップする。アーカイブやディスクイメージの展開、ライブメモリのスキャンは行わない。安定した抽出済みファイルを使い、証拠の外側に新規出力ディレクトリを指定する。

- `-T, --threads <N>`: 並列スキャン数。既定はCPU数、最大8。
- `--timeout <SECONDS>`: ファイルごとの評価上限秒数。既定は60。
- `--max-file-mib <MIB>`: ファイルの最大サイズ（MiB）。既定は64。超過ファイルをスキップとして記録する。ワーカー数やバッファーを増やすとメモリ消費も増える。
- `--rules-release YYYYMMDD`: Forgeのリリース日、または既定のlatest。再現用にソースとハッシュを保存する。

## 結果 { #results }

- `YaraDetections.csv` / `YaraDetections.jsonl`: ルールとファイルの一致。相対パス、証拠ホスト名、ハッシュ、メタデータ、限定した一致位置を含む。一致した内容自体は出力しない。
- `YaraFiles.csv` / `YaraFiles.jsonl`: スキップ、読み取りエラー、タイムアウト、途中結果を含むファイルごとの対象範囲。
- `Summary.json`: 完了状態、設定、件数、エンジンの版、ソースのハッシュ。
- `Rules.yar`: ダウンロードした元のルール、またはローカルルールを展開したスナップショット。
- `CompilationWarnings.txt` / `CompilationError.txt`: コンパイル警告、またはコンパイル失敗時のエラー。

終了コードは、0が完了、3が対象範囲の未完了、2が準備・出力エラー、130がキャンセル。一致は調査の手がかりであり、侵害の証明ではない。一致がないことも安全の証明にはならない。ルール名とアーティファクトの文脈を確認する。隔離されたペイロードの識別には事前の復号が必要。

## MFTの一致と復元 { #mft-recovery }

MFTは専用のサイズ上限（既定512 MiB）で、一度に1つずつ生のバイト列をスキャンする。一致位置のレコードを解析し、レジデントDATAストリームを個別に再スキャンする。ストリーム単独で一致した内容だけを復元する。メタデータのみ、レコードをまたぐ一致、非レジデントの内容はファイルを出力せず報告する。

`--max-mft-mib <MIB>` · `--max-mft-recoveries <N>`

復元数の既定上限は実行ごとに1000ストリーム。制限到達、破損レコード、タイムアウトは明示する。保存名は隔離用拡張子で終わり、証拠側のディレクトリ構成は再作成しない。

`YaraMFTResults.csv` / `YaraMFTResults.jsonl` · `RecoveredMFT/` · `.quarantined`

## 詳細情報 { #further-details }

- [エンジン: Boreal](https://github.com/vthib/boreal)
- [ルールパッケージ: YARA Forge](https://github.com/YARAHQ/yara-forge)
- [詳しい使い方とフィールド解説（英語）](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.md)
- [詳しい使い方とフィールド解説（日本語）](https://github.com/Yamato-Security/juntei/blob/main/docs/yara-scan.ja.md)
