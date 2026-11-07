# ディスクイメージのマウント { #mounting-disk-images }

`juntei mount` は、VHDX ディスクイメージ内のファイルを、実行している間、あるディレクトリから読み取り専用で読めるようにします。これにより、KAPE の `--vhdx` コレクションや Hyper-V のディスクを、Hayabusa や Juntei 自身のコマンドなど任意のツールでスキャンできます。何かを抽出する必要はなく、root 権限も不要で、オペレーティングシステムが VHDX や NTFS に対応している必要もありません。FUSE を使用し、Linux で動作します。リーダーとマウントライブラリは [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) で管理しています。

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

- マウントポイントには NTFS ボリュームのファイルが表示されます。KAPE のコレクションであれば、収集されたファイルを格納する `C` フォルダー、KAPE の `LongFileNames` フォルダー、そして `$MFT` などのボリューム自身のメタデータ ファイルです。複数の NTFS ボリュームを含むイメージでは、それらが `Volume1`、`Volume2` のように表示されます。
- すべてのファイルは読み取り専用で、マウントポイントの所有者に属します。ファイルの時刻は NTFS の `$STANDARD_INFORMATION` の時刻です。
- 名前は NTFS に格納されているとおりに一覧表示され、完全一致で検索されます。DOS 8.3 形式の名前と代替データ ストリームは表示されません。
- 圧縮されたファイルは展開されます (1 ファイルあたり最大 1 GiB)。EFS で暗号化されたファイルは読み取れません。

## 証拠は変更されない { #the-evidence-is-not-changed }

イメージは読み取り専用で開かれ、マウントは書き込みを拒否します。正常に閉じられなかった VHDX は、保留中のログをメモリ上で適用した状態で読み取られ、イメージに書き戻されることはありません。KAPE は VHDX ファイルを DiscUtils で書き込みますが、DiscUtils が書き込んだ NTFS ブート セクターにはジャンプ命令と `55 AA` シグネチャがありません。Juntei はこれらのボリュームをそのまま読み取ります。

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

## auto-parse ですべてのイメージを解析する { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` は、証拠ディレクトリ内のすべての VHDX ディスクイメージ (または `-d` で指定した 1 つのイメージ) に含まれる Windows システムを、別途マウントすることなく解析します。

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

- 各イメージは、新しい一時ディレクトリ内の、イメージ名に由来するフォルダー (`host1-triage`) に読み取り専用でマウントされ、そこで見つかったシステムは、ディスク上のコレクションと同じように解析されます。イメージは実行の最後と、Ctrl+C を押したときにアンマウントされます。
- ターミナルには、イメージの一覧と、各システムがどのイメージのものかが表示されます: `HOST1: ./host1-triage/C (./host1-triage.vhdx)`。
- マニフェストでは、`EvidenceDirectory` は指定されたディレクトリで、各システムの `Image` はそのイメージを示します。`EvidenceRoot` とパーサー コマンドはマウント フォルダー (`./host1-triage/C`) を使用します。
- KAPE は、Volume Shadow Copies からコピーしたファイルを `VSS32` などのフォルダーに保存します。これらのフォルダーもそれぞれ検出され、別のシステム (`HOST1_2`、…) として解析されます。
- マウントできないイメージは報告されます。その場合も他のイメージは解析されますが、実行は終了ステータス 1 で終了します。

開けない NTFS ボリュームがある場合、その問題を報告し、読み取り可能なボリュームの処理を続けます。画像の一部を読み取れなかった場合、`auto-parse --vhdx` は終了コード 1 を返し、`mount` もアンマウント後に 1 を返します。

## 要件 { #requirements }

- Linux: FUSE 3 (`fusermount3`、`fuse3` パッケージ) と `/dev/fuse` へのアクセス権。通常のユーザーは、たいていこのアクセス権を持っています。root 権限は不要です。
- Windows と macOS はサポートされていません。Windows では代わりに、管理者として実行した PowerShell でイメージを読み取り専用でアタッチし、そのドライブ文字をツールに指定します。

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
