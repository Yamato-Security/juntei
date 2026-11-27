# イベントログのカービング { #carve-evtx }

ログの消去や削除のあとに残ったイベントログのチャンクを、ディスクイメージ、未割り当て領域、ページファイル、メモリイメージから復元します。EVTX の 64 KiB のチャンクはそれぞれ独自の文字列テーブルとテンプレートテーブルを持つため、ログの残りの部分がなくてもレコードを読めます。Juntei は復元したチャンクをチャネルごとに通常の `.evtx` ファイルとして書き出し、Hayabusa やイベント ビューアーで開けるようにします。

```bash
juntei carve-evtx -f host.E01 -o carved/
juntei carve-evtx -f host.vhdx --free-space-only -o carved-free/
juntei carve-evtx -f pagefile.sys --live-logs evidence/C/Windows/System32/winevt/Logs/ -o carved-pagefile/
juntei carve-evtx -f memory.raw -o carved-memory/ -t jsonl
```

`-o` を指定しない場合は、サマリーだけを表示します。

## チャンクの見つけ方 { #finding-chunks }

- チャンクのシグネチャをすべてのバイトオフセットで検索し、ヘッダーがもっともらしいこと（ヘッダーサイズ、レコード番号、オフセット）を確認します。チェックサムは記録しますが、一致を条件にはしません。一部が上書きされたチャンクにも、読めるレコードが残っているためです。
- NTFS 圧縮されたログのチャンクは展開します（LZNT1、16 クラスターの単位）。ディスクイメージではボリュームのクラスターサイズを使い、その他のファイルでは `--cluster-size <BYTES>`（既定値 4096）を使います。
- レコードは、ヘッダーにある最後のレコードのオフセットまでサイズをたどって読み、同じレコード ID が繰り返されても止めません。壊れたレコードのあとは、次の有効なレコードから読み続けます。チャンクのスラックにある古いレコードも一覧にします。

## ディスクイメージと NTFS ボリューム { #disk-images }

E01 または VHDX のイメージ、あるいは NTFS を持つ RAW のディスクまたはボリュームでは、各チャンクをそのクラスターによって分類します。

| 場所 | 意味 |
|---|---|
| `FreeSpace` | どのファイルも使っていないクラスターです。 |
| `DeletedLog` | 削除されたログの `$MFT` レコードがまだ指している空きクラスター、またはそこから復元したログです。 |
| `LiveLog` | 使用中のイベントログのクラスターです。`--keep-live` を指定しない限り書き出しません。 |
| `AllocatedFile` | `pagefile.sys` やシャドウコピーのストアファイルなど、他のファイルのクラスターです。 |
| `AllocatedUnknown` | どの `$MFT` レコードも指していない、割り当て済みのクラスターです。 |
| `OutsideVolumes` | ディスク上のどの NTFS ボリュームの外側でもあります。 |
| `Unclassified` | 入力に NTFS ボリュームがありません（ページファイルやメモリイメージ）。 |

`$MFT` レコードが空きクラスターだけを指している削除済みのログは、圧縮されたものも含めてまるごと復元し、`Recovered/` に保存します。`--free-space-only` は空きクラスターだけを検索します。速くなりますが、ページファイル、シャドウコピー、その他のファイルは検索しません。

## 重複排除 { #deduplication }

各チャンクは一度だけ書き出します。現行ログのチャンク、他のチャンクと同一のチャンク、すべてのレコードが現行ログかすでに書き出したチャンクにあるチャンクは書き出しません。レコードの多いチャンクから順に判断するため、古くて短いチャンクのコピーは何も追加しません。NTFS のない入力では、`--live-logs <DIR>` で現行ログを指定します。

## 出力 { #results }

- `Carved/<CHANNEL>.evtx`: チャネルごとに一つのログで、Windows のログと同じ名前の付け方です（`Microsoft-Windows-Sysmon%4Operational.evtx`）。チャンクヘッダーのレコード番号を振り直し、チェックサムを計算し直します。壊れたレコードとその後ろのすべてのレコードはスラックになります。それ以外の部分はスラックも含めてそのまま残します。Hayabusa の `-x` は空とマークされたチャンクを検索します。使用中のチャンクのスラックに残るレコードは、Juntei の `records.csv` で確認できます。
- `Recovered/Volume<N>/...`: `$MFT` レコードから復元した削除済みのログです。
- `chunks.csv` / `chunks.jsonl`: 見つかったすべてのチャンクです。場所、扱い（`Status`）、チャネル、レコード ID、時刻、チェックサムの状態を記録します。
- `records.csv` / `records.jsonl`: 書き出したチャンクのすべてのレコードです。`OnlyInCarvedData` は現行ログにないレコードを示します。

```bash
hayabusa csv-timeline -d carved/Carved -a -o carved-timeline.csv
```

見つけられるのは連続したチャンクだけです。連続していないクラスターや、メモリで連続していない物理ページにまたがるチャンクは不完全で、後ろのレコードが壊れているか欠けていることがあります。`hiberfil.sys` は先に展開してください。検索が完了した場合の終了コードは 0、エラーの場合は 1 です。

## 詳細情報 { #further-details }

- [詳しい使い方とフィールドの説明（英語）](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.md)
- [詳しい使い方とフィールドの説明（日本語）](https://github.com/Yamato-Security/juntei/blob/main/docs/carve-evtx.ja.md)
- [EVTX 形式（libevtx のドキュメント）](https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc)
