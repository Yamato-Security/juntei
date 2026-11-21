# YARAスキャン

`yara-scan` は、収集済みファイルの内容をYARAルールで検索するコマンド。
Rust製の [Boreal](https://github.com/vthib/boreal) を使用し、Windows、macOS、Linuxで動作する。
ファイルはデータとして読み取り、実行しない。Defenderから復元した `.quarantined` ファイルも対象になる。
`auto-parse` とは独立したコマンドで、通常のアーティファクト解析時には自動実行しない。

## ルールの選択

`--rules` を指定しない場合、[YARA Forge](https://github.com/YARAHQ/yara-forge) の
GitHubリリースからHTTPSでパッケージをダウンロードし、コンパイルしてスキャンする。

| パッケージ | 指定方法 | 用途 |
|---|---|---|
| Core | 既定値、または `--ruleset core` | 精度と性能を重視した比較的絞られたルール。まずはこちらから開始する。 |
| Extended | `--ruleset extended` | 検索対象を広げたハンティング。追加の検知を文脈と照合する。 |
| Full | `--ruleset full` | 最も広い対象範囲。リソース消費と誤検知の可能性が増える。 |

```bash
juntei yara-scan -d evidence/ -o yara-core/
juntei yara-scan -d evidence/ --ruleset extended -o yara-extended/
juntei yara-scan -d evidence/ --ruleset full -o yara-full/ -t jsonl
```

オンライン実行では毎回、既定で最新パッケージをダウンロードする。
特定のリリースに固定するには `--rules-release YYYYMMDD` を使う。
例：そのリリースが公開されていれば `--rules-release 20261115`。
`Rules.yar` と `Summary.json` を保管すれば、`latest` の内容が変わっても、
保存されたソースとSHA256で使用ルールを特定できる。

オフラインではルールファイル、または `.yar` / `.yara` を含むディレクトリを指定する。

```bash
juntei yara-scan -d evidence/ --rules local-rules/ -o yara-offline/
juntei yara-scan -f recovered/agent.exe.quarantined --rules local.yar -o yara-file/
```

`--rules` を使うとネットワーク通信を行わない。明示的な `--ruleset` や `--rules-release` との併用はできない。
ディレクトリは再帰的に探索し、同じ名前空間内でパス順に読み込む。依存関係には `include` を使う。
`include` は記述ファイルを基準に解決し、指定したディレクトリ内
（ファイルを指定した場合はその親ディレクトリ内）に限定する。同じファイルは一度だけ展開する。
循環参照、範囲外の参照、ルール名の重複、未対応モジュールはエラーになる。
不正なルールを黙って除外しない。ローカルルールはUTF-8で保存する。

コンパイル警告は `CompilationWarnings.txt` に保存する。
コンパイル段階で失敗した場合は `CompilationError.txt` に内容を記録する。
`Rules.yar` は、ローカルルールを展開したスナップショット、またはダウンロードした元のソース。
ルールのライセンスは各作者のものが適用され、ダウンロードによって変更されない。

## 対象範囲と制限

拡張子にかかわらず通常ファイルを対象とする。シンボリックリンクや特殊ファイルは追跡せず、
スキップとして記録する。証拠ディレクトリ内にある使用中のルールソースも自己検知を防ぐため除外する。
出力先は証拠の外側にある**新しいディレクトリ**を指定し、ローカルルールを上書きしない。
単一ファイルのスキャンでは、そのファイルと同じ親ディレクトリ内に新しい結果ディレクトリを作成できる。
証拠の内容は変更しない。

ファイル全体を通常の**ファイル用セマンティクス**で評価する。プロセスメモリ用の評価ではない。
ZIPやディスクイメージの展開、非レジデントデータの復元、実行中プロセスのメモリスキャンは行わない。
VHDX/E01をそのまま指定して内部のファイルがスキャンされると考えず、取り出したファイルを指定する。
MFTメタデータから作った空のプレースホルダーには、マルウェア本体のバイト列は含まれない。

| オプション | 既定値 | 内容 |
|---|---|---|
| `-d, --directory <DIR>` | `-f` とどちらか必須 | ディレクトリを再帰スキャン。 |
| `-f, --file <FILE>` | `-d` とどちらか必須 | 単一ファイルをスキャン。 |
| `-o, --output <DIR>` | 必須 | 新規出力ディレクトリ。 |
| `-r, --rules <PATH>` | なし | オフライン用のルールファイル／ディレクトリ。 |
| `--ruleset core\|extended\|full` | `core` | ダウンロードするパッケージ。 |
| `--rules-release <RELEASE>` | `latest` | `latest` または `YYYYMMDD`。 |
| `-t, --output-type csv\|jsonl` | `csv` | 検知と対象範囲レポートの形式。 |
| `-T, --threads <N>` | CPU数、最大8 | 並列ファイルスキャン数。指定範囲は1～256。 |
| `--timeout <SECONDS>` | 60 | ファイルごとのYARA評価上限秒数。読み取り時間は含まない。 |
| `--max-file-mib <MIB>` | 64 | 超過ファイルをスキップ。指定範囲は1～4096 MiB。 |
| `--computer <NAME>` | 近傍の `client_info.json`、不明なら空欄 | 証拠ホスト名を明示。 |
| `-q, --quiet` | 無効 | ロゴ、進捗、通常の準備メッセージを省略。 |
| `-K, --no-color` | 無効 | 色を無効化。 |

コンパイル済みルールはワーカー間で共有する。各ワーカーは上限内のファイルを1つ保持する。
既定の8ワーカーではファイル内容だけで約512 MiBを保持し得る。
これにルール、マッチ情報、その他のメモリが加わるため、プロセス全体のメモリ上限ではない。
メモリが少ない環境では `-T` または `--max-file-mib` を下げる。
ハッシュとスキャンは同じメモリ上のバイト列に対して行う。読み取り中にサイズや更新時刻の変化を検出した場合は
読み取りエラーにする。安定した証拠コピーを使う。

Ctrl+Cはファイル間でキャンセルを要求する。実行中の評価は設定したタイムアウトまで続く場合がある。
キャンセル対象は `Cancelled` として記録する。

## 出力

| ファイル | 内容 |
|---|---|
| `YaraDetections.csv` / `.jsonl` | ルールとファイルの組み合わせごとの検知。未完了スキャンの途中結果も含む。 |
| `YaraFiles.csv` / `.jsonl` | 発見したファイル、スキップ対象、ディレクトリ探索エラーごとの対象範囲。MFT候補ストリームの行も含む。 |
| `Summary.json` | 完了状態、件数、設定、ルール／パッケージのハッシュ、ソース一覧、エンジンの版。 |
| `Rules.yar` | 再現用の使用ルール。 |
| `CompilationWarnings.txt` | コンパイル警告。空の場合もある。 |
| `RunStarted.json` | 開始時の記録。`Summary.json` がなければ実行は正常完了していない。 |

証拠パスは入力ディレクトリからの相対パス。単一ファイルでは親ディレクトリからの相対パスになる。
`Computer` は近傍の `client_info.json` または `--computer` から取得し、不明なら空欄にする。
解析用コンピューターの名前を証拠ホスト名として使わない。

### 検知フィールド

| フィールド | 解釈 |
|---|---|
| `Computer` | 判明した場合の証拠ホスト名。 |
| `FilePath` | ソースの相対パス。`.quarantined` も保持する。 |
| `Size` | スキャンしたバイト数。 |
| `SHA256` | スキャンした同じバイト列のハッシュ。 |
| `ScanStatus` | 評価完了は `Scanned`、未完了の途中結果は `ScanError`。 |
| `Rule`, `Namespace`, `Tags` | ルール名、名前空間、タグ。 |
| `Metadata` | 作者、説明、参照、重要度などルールに定義されたメタデータのJSON文字列。 |
| `MatchLocations` | 一致パターンごとの最初の位置を最大16パターン記録したJSON文字列。`Offset` はファイル内のバイト位置。`Length` と `XorKey` も含む。一致した内容自体は出力しない。 |

### 対象範囲フィールド

`Computer`、`FilePath`、`Size`、`SHA256` で対象を特定する。未読・スキップ対象のハッシュは空欄。
`Matches` は一致ルール数、`ElapsedMs` は読み取り・ハッシュ・評価の所要時間、`Note` は取得できたエラー説明。

`Status` は `Scanned`、`SkippedTooLarge`、`SkippedSymlink`、`SkippedSpecialFile`、
`SkippedRuleSource`、`DiscoveryError`、`ReadError`、`ScanError`、`Cancelled` を区別する。
スキップやタイムアウトは**安全を確認した状態ではない**。途中で検知があっても `ScanError` になる場合がある。
検知件数はルールとファイルの組み合わせ数であり、マルウェアファミリー数や侵害ホスト数ではない。

終了コードは対象の評価完了が0、対象範囲の未完了が3、準備・出力エラーが2、キャンセルが130。
検知そのものはエラーではない。`SkippedRuleSource` だけなら未完了扱いにしない。
まず `Summary.json` を確認し、実行痕跡、ファイルシステム、イベントログと照合して判断する。
YARAの一致は調査の手がかりであり、一致しないことは安全の証明にならない。

## Further Details

- [YARA Forgeのパッケージとリリース](https://github.com/YARAHQ/yara-forge)
- [Borealの互換性と実装](https://github.com/vthib/boreal)
- [YARAルール記述リファレンス](https://yara.readthedocs.io/en/stable/writingrules.html)

## MFTの生データスキャンと一致したレジデントファイルの復元

一般的な名前（`$MFT`、末尾が `$MFT`、`%24MFT`、`MFT`、`.mft`）、または妥当な先頭FILEレコードヘッダーからMFTを認識する。
MFTには通常ファイルとは別の `--max-mft-mib` を適用する。既定は **512 MiB**、指定範囲は1～4096。
通常ファイルの並列スキャン後に、コンパイル済みルールを共有して**MFTを一つずつ**スキャンする。
複数の巨大MFTバッファーを並列に保持しない。

生データで一致したパターンの位置から候補のFILEレコードを特定する。レコードを検証し、NTFS更新シーケンスの
fixupを適用して、そのレコードの各レジデント `$DATA` を独立したファイルとして再スキャンする。
これにより、再評価時のファイルサイズ、オフセット、複合条件はMFT全体ではなく各ストリームを対象とする。
**単独でルールに一致したレジデントストリームだけ**を `RecoveredMFT/` に `.quarantined` 拡張子で保存する。
保存名には元のファイル名、レコード番号、シーケンス番号、属性識別情報を含めるが、証拠側のディレクトリ構成は再作成しない。
元の証拠は変更しない。名前付き代替ストリームと削除レコードも対象になる。
拡張レコードの所有者は、世代番号とベースレコードの属性リストの両方を検証できた場合にだけ記録する。

これは**生データの一致を起点とした復元**であり、全レジデントストリームの網羅的スキャンではない。
小さいファイルだけを対象にするルールなど、MFTの生データには一致しないルールだけでは候補を選べない。
追加の対象範囲が必要なら、レジデントストリームを別途抽出してスキャンする。
メタデータだけの一致、FILEレコードをまたぐパターン、ストリーム単独では成立しない複合条件、位置を持たないルールからは
ペイロードとして出力しない。非レジデントの内容は元のボリュームのクラスタが必要で、単独のMFTだけからは復元できない。
その場合に空の代替ファイルは作らない。破損レコードは報告する。

`--max-mft-recoveries <N>` は実行全体の復元上限。既定は **1000**、指定範囲は1～100000。
復元上限や生データの位置保存上限に達した場合は未完了として明示する。
画面・検知行に表示する16パターンの上限では復元を制限しない。MFTごとに最大100000位置、
エンジンの各パターン最大1000出現まで保持した位置を使用する。どちらの上限に達しても報告する。

```bash
juntei yara-scan -d evidence/ --ruleset full -o yara-mft/
```

MFTが見つかると `YaraMFTResults.csv` または `YaraMFTResults.jsonl` を追加する。

| フィールド | 意味 |
|---|---|
| `Computer`, `MFTPath` | 証拠ホスト名と相対MFTパス。 |
| `EntryNum`, `SN` | 物理FILEレコードの番号と世代番号。 |
| `BaseEntryNum`, `BaseSN` | 検証できた拡張レコードのベース所有者。不明なら空欄。 |
| `OriginalPath`, `Stream` | 解決できたWindowsパスと名前付きストリーム。未解決のパスは明示する。 |
| `AttributeID`, `AttributeOffset` | 属性のインスタンス番号とレコード内オフセット。 |
| `Deleted` | 物理FILEレコードが未割り当てかどうか。 |
| `RawRules`, `StreamRules` | 生のMFTで一致したルール名と、ストリーム単独で一致したルール名。 |
| `Status`, `Note` | 復元状態と説明。 |
| `Size`, `SHA256` | fixup適用後の正確なレジデント内容のサイズとハッシュ。非レジデント行には内容のハッシュを付けない。 |
| `RecoveredFile` | 出力ディレクトリからの相対復元パス。未保存なら空欄。 |

状態は `Recovered`、`ResidentNotConfirmed`、`NonResidentUnavailable`、`MetadataOnly`、
`NoMatchOffsets`、`CrossRecordSpan`、`InvalidMFT`、`InvalidRecord`、`AttributeError`、
`TruncatedRecord`、`StreamScanError`、`RawLocationLimit`、`RecoveryLimitReached`、`Cancelled` を区別する。
復元エラーや上限到達は未完了扱いにし、非レジデント内容がないという制約も明示する。

通常の検知・対象範囲テーブルには `SourceType` を追加し、`File`、`MFTRaw`、`MFTResident` を区別する。
レジデント行の `FilePath` はレコードとストリームを指す仮想参照で、追加の入力ファイルではない。
検知行の `RecoveredFile` が実際の復元先を示す。生データの一致オフセットはMFTを基準とし、
レジデント行はそのストリームを基準とする。両方の一致が同じ証拠を指す場合もあり、検知数はマルウェアの一意数ではない。
`Summary.json` にMFT上限と復元統計を追加する。

保存されたファイルは**YARAに一致した証拠**であり、マルウェア判定や実行の証明ではない。
