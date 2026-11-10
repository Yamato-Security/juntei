# ディスクイメージのマウント { #mounting-disk-images }

`juntei mount` は、実行中の間、VHDX / E01 / Ex01 ディスクイメージ内のファイルをディレクトリから読み取り専用で読めるようにします。ファイルの抽出や root 権限なしに、Hayabusa や Juntei などのツールで調査できます。FUSE を使用し、Linux で動作します。リーダーとマウントライブラリは [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) で管理しています。

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

別のターミナルで次を実行します。

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

アンマウントするには、最初のターミナルで Ctrl+C を押すか、`fusermount3 -u /mnt/host1` を実行します。

## マウントで表示されるもの { #what-the-mount-shows }

- マウントポイントには NTFS ボリュームのファイルが表示されます。収集済みイメージには、`C` や `LongFileNames` などのディレクトリと、`$MFT` などのボリュームメタデータが含まれる場合があります。複数の NTFS ボリュームは `Volume1`、`Volume2` のように表示されます。
- すべてのファイルは読み取り専用で、マウントポイントの所有者に属します。ファイルの時刻は NTFS の `$STANDARD_INFORMATION` の時刻です。
- 名前は NTFS に格納されているとおりに一覧表示され、完全一致で検索されます。DOS 8.3 形式の名前と代替データ ストリームは表示されません。
- 圧縮されたファイルは展開されます (1 ファイルあたり最大 1 GiB)。EFS で暗号化されたファイルは読み取れません。

## 証拠は変更されない { #the-evidence-is-not-changed }

イメージは読み取り専用で開かれ、マウントは書き込みを拒否します。正常に閉じられなかった VHDX の保留中のログはメモリ上で適用され、イメージに書き戻されません。一部の DiscUtils ライターが作成する NTFS ブートセクターはジャンプ命令と `55 AA` シグネチャがゼロになっていますが、Juntei はそのまま読み取ります。

## 使用法 { #usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX or E01; a split E01 image from its first segment)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## auto-parse でイメージを解析する { #parsing-every-image-with-auto-parse }

`--vhdx` を指定すると、`auto-parse` は [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) を使用して
**Linux、macOS、Windows** で VHDX をネイティブに読み取ります。OS へのマウント、FUSE ドライバー、管理者権限は不要です。

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

各パーサーはファイルパスを必要とするため、リーダーが公開する NTFS ボリュームから、パーサーが読み取るファイル（レジストリハイブとそのトランザクションログ、NTFS メタデータファイル、データベース、Prefetch、ショートカット、Jump List、Outlook ファイル、Defender の検疫フォルダー）を専用の一時作業領域にコピーしてから通常の解析を行います。
それ以外のファイルは、元の更新時刻を持つ空のプレースホルダーになるため、ディレクトリとパスはディスク上と同じに見えます。
persistence-hunter は、永続化エントリが参照する実行ファイルなど、必要になったプレースホルダーの内容だけをイメージから読み取ります。
**出力結果に加えて、コピーするファイルを保存する一時ディスク容量が必要です。**
読み取りバッファーは固定サイズです。ホストのファイルシステムが対応している場合、ゼロ領域はスパース領域として書き込みます。
元のファイル名、ディレクトリ構造、ハードリンク、ファイル更新時刻を保持します。
LNK と Jump List のソース作成・更新・アクセス時刻は、一時ファイルではなく元の NTFS メタデータのスナップショットから取得します。
一時ディレクトリは、パーサーエラーが報告された場合を含めて正常な終了処理で削除されますが、強制終了時には残る場合があります。
元のイメージは読み取り専用で開き、未反映の VHDX ログもメモリ上でのみ適用します。

ホスト名の検出、ホスト別の出力、CSV/JSON/JSONL、`-j` は通常の証拠ディレクトリと同じ解析処理を使用します。
`manifest.json` の `Image` は元のイメージを、`EvidenceRoot` はイメージ内の相対パスを示します。
読み取れないイメージ、ボリューム、ファイル、安全に表現できないファイル名は、`ImageErrors` と
`manifest.csv` の失敗行に記録します。読み取れる証拠の解析は継続し、不完全な入力があれば終了コードは 1 になります。
危険な名前や曖昧な名前は勝手に変更せず、エラーとして報告します。名前付き NTFS ストリームを追加ファイルとして抽出する機能はありません。
トリアージイメージ内に通常のファイルとして収集済みのアーティファクトは利用できます。

他のプログラムにファイルシステムを公開する [`mount` コマンド](mount.md) は、引き続き Linux の FUSE を必要とします。
`auto-parse --vhdx` はこのコマンドを使用しません。

ステージングは既定で出力先のファイルシステム上の専用ディレクトリを使用します。 `--temp-dir` で別ディスク上の既存のディレクトリを指定できます。元のメタデータは出力側に別途保持するため、ステージング先が満杯でも、読み取れたファイルの解析を続行できます。コピーと結果の両方に必要な空き容量を確保してください。出力先自体が満杯の場合は結果を保存できません。Ctrl+C はパーサープロセスを停止し、専用作業領域を削除して終了コード 130 を返します。SIGKILL や電源断ではコピーが残る場合があります。イメージ検索では読み取れないサブディレクトリをスキップしますが、入力ルートが読めない場合はエラーです。

ステージング先は出力ディレクトリの外に指定してください。Linux と macOS では、SIGTERM と SIGHUP でもパーサープロセスを停止し、作業領域を削除します。

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## E01 と Ex01 ディスクイメージ { #e01-and-ex01-disk-images }

`--e01` は、物理 EWF1（`.E01`）および EWF2（`.Ex01`）イメージを **Linux、macOS、Windows** でネイティブに読み取ります。FUSE、OS へのマウント、管理者権限は不要です。[ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) と、上記と同じ NTFS 解析処理を使用します。

分割イメージは最初のセグメントを指定します。`.E02` などの関連ファイルを元の名前のまま同じディレクトリに置いてください。ディスク全体（MBR/GPT）と単一の NTFS ボリュームに対応します。両形式が混在するディレクトリには `--vhdx --e01` を使用します。

一時ディスク容量と元のメタデータに関する要件も同じです。E01 は通常ディスク全体を収録しており、すべてのイメージを解析開始前にステージングしますが、コピーするのはパーサーが読み取るファイルだけで、通常は使用容量のごく一部です。イメージは読み取り専用です。取得時の欠損や、開始時・読み取り中に見つかった破損は `ImageErrors` と `manifest.csv` に記録され、読み取れる結果を保持して終了コード 1 を返します。検査対象はアクセスしたデータであり、全チャンクやメディア全体のハッシュではありません。論理形式の `.L01`、`.Lx01` は拒否します。パスワード指定には対応しません。

別機能の `mount` コマンドには引き続き Linux FUSE が必要ですが、ネイティブ自動解析には不要です。

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## OS へのマウント要件 { #os-visible-mount-requirements }

`juntei mount` は Linux の FUSE 3 (`fusermount3`) と `/dev/fuse` へのアクセスを必要とします。Windows と macOS では、上記のネイティブ解析を使用してください。
