# ブート セクタ (`$Boot`) { #boot-sector-boot }

## `$Boot`とは { #what-boot-is }

`$Boot` は、すべての NTFS ボリュームの MFT エントリ 7 です。ボリュームの最初の 8 KiB です。その最初のセクターである **ボリューム ブート レコード** には、ファイル システムを記述する BIOS パラメーター ブロックが保持されます。

- セクターとクラスターのサイズ、ボリューム内のセクターの数、および **隠しセクター** (ボリュームの前のディスク上のセクター、つまりボリュームのパーティションが開始される場所)。
- ここで、`$MFT` と `$MFTMirr` は始まり、FILE と INDX レコードのサイズです。
- 64 ビット **ボリュームのシリアル番号**。ボリュームのフォーマット時に設定されます。

最初のセクターの残りとその後の 15 セクターには、Windows をロードする **ブート コード** が保持されます。

シリアル番号の下位 32 ビットは、Windows では `XXXX-XXXX` (`vol`、`dir`) として表示され、[LNK のショートカットとして ](lnk.md) が表示されます。ファイルが存在していたボリュームの [Jump Lists](jump-lists.md) および [Prefetch](prefetch.md) レコード。 `$Boot` は、これらのレコードを収集されたホストのボリュームに関連付けます。

<span class="evidence-strength">この証拠から判断できること:</span> LNK、Jump Lists、Prefetch のボリュームシリアル番号が `$Boot` の `VolumeSerialNumber` と一致する場合、そのボリューム、または同じ番号を付与された・コピーされた別のボリュームを指します。シリアル番号は変更可能で、ホスト間で一意ではありません。

## 保存場所と収集 { #location-and-collection }

```text
C:\$Boot          (MFT entry 7; the first 8 KiB of the volume)
```

raw 対応のフォレンジック ツールを使用して収集するか、ボリューム イメージの最初のセクターを読み取ります。どちらも同じバイトです。 Juntei は、NTFS ブート セクターで始まるファイルを読み取るため、ボリューム イメージが入力として機能します。ディスク イメージは、代わりにパーティション テーブルから始まります。 Juntei にパーティションのボリュームを与えます。

## 分析時の注意点 { #analyst-cautions }

- シリアル番号はボリュームをフォーマットすると変わります。シリアル番号が現在のボリュームに一致しない LNK ファイルは、リムーバブル メディア、ネットワーク共有のサーバー、またはそれ以降にフォーマットされたボリュームを指している可能性があります。
- ブート コードは、同じ Windows でフォーマットされたボリュームでは同じです。DFIR アーティファクトの Windows、10、11、およびサーバー 2012 R2 `$Boot` ファイルで同じです。博物館。ボリューム ブート レコード ブートキットは、このフィールド、またはブート コードが `$Boot` の残りの部分を見つけるために使用する隠しセクター フィールドを変更します。結論を下す前に、`BootCodeSHA256`、`JumpInstruction`、および `HiddenSectors` を他のホストおよびパーティション テーブルと比較してください。ブート マネージャーや暗号化ソフトウェアによってもブート コードが変更される可能性があります。
- BitLocker 暗号化ボリュームのイメージは、BitLocker 自体のブート セクター (OEM ID `-FVE-FS-`) で始まります。 `-f` を指定すると、Juntei はそれを報告しますが、`Problems` は NTFS ではないと言います。 NTFS ブート セクターは暗号化されたデータ内にあります。ロック解除されたボリュームから収集された `$Boot` は、NTFS です。
- `Problems` は、完全な Windows 実装性テストを提供するのではなく、認識された構造的不一致と切り捨てをリストします。それ自体を攻撃の発見としてではなく、彫刻されたセクターとコピーを評価するために使用します。ブートコード ハッシュには、最初のジャンプ、BIOS パラメーター ブロック、および終了署名が含まれません。それらのフィールドも調べてください。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      Parse one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

概要には、各ブート セクターのボリューム シリアル番号 (Windows が示すように、また LNK および Prefetch フィールドが示すように)、サイズ、クラスター サイズ、および `$MFT` クラスターがリストされます。複数のホストが解析されると、ブート コード ハッシュによってホストがグループ化され、ブート コードが他のボリュームと異なるボリュームが目立つようになります。

## CSV 出力フィールド { #csv-output-fields }

| フィールド | 解釈 |
|---|---|
| `SourcePath` | 入力ファイル。 |
| `VolumeSerialNumber` | 32 ビットのシリアル番号 (LNK、ジャンプ リスト、および Prefetch 出力) にそれが表示されます (`180B1E67`)。 Windows は `180B-1E67` と表示されます。 |
| `VolumeSerialNumber64` | 64 ビットのシリアル番号全体。 |
| `OEM-ID` | NTFS ボリューム上の `NTFS`。 |
| `BytesPerSector`, `SectorsPerCluster`, `ClusterSize` | ボリュームのジオメトリ。 128 上のクラスターごとのセクターは 2 の累乗として記録され、ここでデコードされます。有効でない場合は空です。 |
| `TotalSectors`, `VolumeSize` | ボリューム内のセクターとそのサイズ (バイト単位)。最後にブート セクターのコピーは含まれません。 |
| `HiddenSectors` | ボリュームの前のディスク上のセクタ: そのパーティションが始まる場所。 |
| `MFT-Cluster`, `MFT-Offset` | `$MFT` が始まる場所 (クラスター単位、ボリュームの先頭からのバイト単位)。 |
| `MFT-MirrorCluster`, `MFT-MirrorOffset` | `$MFTMirr` が始まる場所。 |
| `FileRecordSize`, `IndexRecordSize` | FILE レコード (通常は 1,024) および INDX レコード (通常は 4,096) ごとのバイト数。 |
| `MediaDescriptor` | ハードディスクの場合は `F8`。 |
| `JumpInstruction` | 最初の 3 バイト (`EB 52 90`): BIOS パラメータ ブロックを越えてブート コードにジャンプします。 |
| `EndSignature` | 有効なブート セクターの末尾にある `55 AA`。 |
| `BootCodeSHA256` | ブート コードの SHA-256: 最初のセクターのバイト 0x54 ～ 0x1FD と残りの 8 KiB。 |
| `Problems` | Windows がマウントしないもの: `NTFS` 以外の OEM ID、`55 AA` なし、無効なサイズ、FAT フィールド NTFS はゼロセットのまま、`$MFT` はボリュームの終わりを超えます。 8 KiB より短いファイル。 |

JSON/JSONL には同じフィールドと名前があり、数値は数値、`null` は不明な値、`Problems` はリストとなります。

## 調査戦略 { #investigation-strategy }

1. 各ホストの `VolumeSerialNumber` を、その LNK ファイル、Jump Lists および Prefetch ファイルのボリューム シリアル番号の横に配置します。収集されたボリュームに一致しないものは、他のボリュームです (USB ドライブ、共有、それ以降にフォーマットされたボリューム)。
2. [USB devices](usb.md) 出力で他のシリアル番号を調べます。この出力には、マウントされたリムーバブル ボリュームのボリューム シリアル番号がリストされています。
3. ホスト間で `BootCodeSHA256` と `HiddenSectors` を比較します。逆アセンブラとディスクのパーティション テーブルを使用して異なるブート コードを調査し、ブート関連のイベント ログ エントリを探します。
4. `MFT-Offset` および `ClusterSize` を使用して、ボリューム イメージ内の `$MFT` を検索します。

## さらに詳しい情報 { #further-details }

- [Microsoft: NTFS 概要](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
- [libfsntfs: NTFS 形式のドキュメント](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (ボリューム ヘッダー)
- [探偵キット: fsstat](https://www.sleuthkit.org/sleuthkit/man/fsstat.html)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`boot` クレート)
