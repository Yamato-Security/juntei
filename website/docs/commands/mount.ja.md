# ディスクイメージのマウント { #mounting-disk-images }

`juntei mount` は、実行中の間、ディスクイメージ (VHDX または E01 イメージ) 内のファイルをディレクトリから読み取り専用で読めるようにします。ファイルの抽出や root 権限なしに、Hayabusa や Juntei などのツールで調査できます。FUSE を使用し、Linux で動作します。VHDX リーダーとマウントライブラリは [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) で、E01 リーダーは [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) で管理しています。

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

E01 イメージ (EnCase の Expert Witness Format。`.Ex01` も含む) も同じ方法でマウントします。分割イメージは最初のセグメントで指定します。残りのセグメント (`host1.E02`、`host1.E03`、…) は、Juntei が同じ場所から見つけます。

```bash
juntei mount host1.E01 /mnt/host1
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
- パーティション テーブルのない単一ボリュームのイメージは、そのボリュームとして扱われます。
- 名前は NTFS に格納されているとおりに一覧表示され、完全一致で検索されます。DOS 8.3 形式の名前と代替データ ストリームは表示されません。
- 圧縮されたファイルは展開されます (1 ファイルあたり最大 1 GiB)。EFS で暗号化されたファイルは読み取れません。

## 証拠は変更されない { #the-evidence-is-not-changed }

イメージは読み取り専用で開かれ、マウントは書き込みを拒否します。正常に閉じられなかった VHDX の保留中のログはメモリ上で適用され、イメージに書き戻されません。一部の DiscUtils ライターが作成する NTFS ブートセクターはジャンプ命令と `55 AA` シグネチャがゼロになっていますが、Juntei はそのまま読み取ります。

E01 イメージのセグメントも読み取り専用で開かれます。各チャンクは読み取り時にチェックサムで検証されます。破損したチャンクは読み取れませんが、イメージのそれ以外の部分は読み取れます。取得時に読み取れなかったセクターと、完了しなかった取得は、警告として報告されます。論理証拠ファイル (`.L01`、`.Lx01`) はディスクではなくファイルを格納するものであり、マウントされません。

## 使用法 { #usage }

```text
Version: 1.1.0 (Dev Build)
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

## auto-parse ですべてのイメージを解析する { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` は、証拠ディレクトリ内のすべての VHDX ディスクイメージ (または `-d` で指定した 1 つのイメージ) に含まれる Windows システムを、`auto-parse --e01` は、すべての E01 イメージに含まれる Windows システムを、別途マウントすることなく解析します。2 つのオプションは同時に指定できます。

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d /cases/e01 --e01 -o parsed
```

- 各イメージは、新しい一時ディレクトリ内の、イメージ名に由来するフォルダー (`host1-triage`) に読み取り専用でマウントされ、そこで見つかったシステムは、ディスク上のコレクションと同じように解析されます。イメージは実行の最後と、Ctrl+C を押したときにアンマウントされます。
- ターミナルには、イメージの一覧と、各システムがどのイメージのものかが表示されます: `HOST1: ./host1-triage/C (./host1-triage.vhdx)`。
- `--e01` は各 E01 イメージの最初のセグメント (`*.E01`、`*.Ex01`) を対象とし、残りのセグメントはそれとあわせて読み取られます。
- マニフェストでは、`EvidenceDirectory` は指定されたディレクトリで、各システムの `Image` はそのイメージを示します。`EvidenceRoot` とパーサー コマンドはマウント フォルダー (`./host1-triage/C`) を使用します。
- Volume Shadow Copies から取得したファイルが `VSS32` などのフォルダーに含まれる場合、それぞれを別のシステム (`HOST1_2`、…) として検出し、解析します。
- マウントできないイメージは報告されます。その場合も他のイメージは解析されますが、実行は終了ステータス 1 で終了します。

開けない NTFS ボリュームがある場合、その問題を報告し、読み取り可能なボリュームの処理を続けます。イメージの一部を読み取れなかった場合、`auto-parse --vhdx` と `--e01` は終了コード 1 を返し、`mount` もアンマウント後に 1 を返します。

## 要件 { #requirements }

- Linux: FUSE 3 (`fusermount3`、`fuse3` パッケージ) と `/dev/fuse` へのアクセス権。通常のユーザーは、たいていこのアクセス権を持っています。root 権限は不要です。
- Windows と macOS はサポートされていません。Windows では代わりに、管理者として実行した PowerShell で VHDX イメージを読み取り専用でアタッチし、そのドライブ文字をツールに指定します。Windows は E01 イメージをアタッチできないため、E01 イメージをマウントできるツールで読み取り専用でマウントし、そのドライブをツールに指定します。

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
