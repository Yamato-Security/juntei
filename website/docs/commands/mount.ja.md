# ディスクイメージのマウント { #mounting-disk-images }

`juntei mount` は、実行中の間、VHDX ディスクイメージ内のファイルをディレクトリから読み取り専用で読めるようにします。ファイルの抽出や root 権限なしに、Hayabusa や Juntei などのツールで調査できます。FUSE を使用し、Linux で動作します。リーダーとマウントライブラリは [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) で管理しています。

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
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX)
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

各パーサーはファイルパスを必要とするため、リーダーが公開する NTFS ボリューム内のファイルを、
システムの専用一時ディレクトリにストリーミングでコピーしてから通常の解析を行います。
**出力結果に加えて、これらのファイルを保存する一時ディスク容量が必要です。**
解析対象になったアーティファクトだけではなく、公開されたファイルシステム全体をコピーします。
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

ステージング先は、容量の小さい RAM ディスクの場合があるシステムの一時ディレクトリではなく、出力先と同じファイルシステムが既定になりました。`--temp-dir` で別ディスク上の既存のディレクトリを指定できます。元のメタデータは出力側に別途保持するため、ステージング先が満杯でも、読み取れたファイルの解析を続行できます。コピーと結果の両方に必要な空き容量を確保してください。出力先自体が満杯の場合は結果を保存できません。Ctrl+C はパーサープロセスを停止し、専用作業領域を削除して終了コード 130 を返します。SIGKILL や電源断ではコピーが残る場合があります。イメージ検索では読み取れないサブディレクトリをスキップしますが、入力ルートが読めない場合はエラーです。

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## OS へのマウント要件 { #os-visible-mount-requirements }

`juntei mount` は Linux の FUSE 3 (`fusermount3`) と `/dev/fuse` へのアクセスを必要とします。Windows と macOS では、上記のネイティブ解析を使用してください。
