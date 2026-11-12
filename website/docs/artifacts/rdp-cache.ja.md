# RDP ビットマップキャッシュ { #rdp-bitmap-cache }

## RDP ビットマップキャッシュに記録される情報 { #what-the-rdp-bitmap-cache-records }

リモート デスクトップ クライアント (`mstsc.exe`) は、永続ビットマップキャッシュが有効な状態 (既定) で接続すると、以降のセッションで同じ部分を再び受信しなくて済むように、リモート画面の断片をディスクに保存します。各断片は最大 64 × 64 ピクセルのタイルです。タイルを集めると、ユーザーがリモート コンピューターで見ていたもの (ウィンドウ、テキスト、ファイル名、コマンド、デスクトップ) の小さな断片になります。

| クライアント | キャッシュ ファイル | タイル |
|---|---|---|
| Windows 7 以降 | `Cache0000.bin`、`Cache0001.bin`、…、各最大 100 MB | 32 ビット、連続して格納 |
| Windows XP、Server 2003 以前 | `bcache2.bmc` (1 ピクセルあたり 8 ビット)、`bcache22.bmc` (16)、`bcache23.bmc` (24)、`bcache24.bmc` (32)、各最大 10 MB | 64 × 64 のスロットごとに 1 つ。一部は圧縮 |

Windows 10 以降では、`.bin` ファイルの横に空の `bcache24.bmc` も残ります。

<span class="evidence-strength">この証拠から判断できること:</span> キャッシュがあるプロファイルのユーザーが、リモート デスクトップ クライアントで、キャッシュされた内容を表示していたコンピューターに接続しました。それがどのコンピューターだったのか、いつだったのかは、キャッシュからはわかりません。

## 保存場所と収集 { #locations-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```
`HKCU\Software\Microsoft\Terminal Server Client` の `BitmapPersistCacheLocation` 値で、キャッシュの場所が変更されていることがあります。すべてのプロファイルの `Cache` フォルダー全体を、ボリューム シャドウ コピーからも収集してください。キャッシュ ファイルの古いコピーには、現在のファイルにはもうないタイルが残っていることがあります。

## 分析時の注意点 { #analyst-cautions }

- タイルには日時がなく、キャッシュにはタイルの送信元のコンピューター名も記録されていません。タイルは、ほかの証拠から RDP 接続と関連付けてください。クライアントの `Terminal Server Client` レジストリ キー (`parse-registry remote-access`)、`Default.rdp` などの `.rdp` ファイル、`mstsc.exe` の Jump Lists、`$MFT` にあるキャッシュ ファイルの日時、リモート コンピューターのログオン イベントなどです。
- 1 つのキャッシュ ファイルには、多数のセッションと複数のリモート コンピューターのタイルが混在しています。ファイルがいっぱいになるとクライアントは古いタイルを置き換えるため、数日前や数か月前のタイルが残っていることもあります。
- タイルは画面の断片であり、画面全体ではありません。画面上で隣り合っていたタイルは、ファイル内でも隣り合っていることが多いものの、常にそうとは限りません。画面は [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher) を使って、またはコラージュを目で見て再構成してください。
- ほとんどのタイルは 64 × 64 ピクセルです。画面の右端と下端のタイルは、解像度に応じて幅が狭いか高さが低くなります (たとえば幅 58 または 62 ピクセル、高さ 28 または 35 ピクセル)。Juntei はすべてのタイルをそれぞれのサイズで読み取ります。すべてのタイルを幅 64 ピクセルとして読み取るツールでは、幅の狭いタイルが歪んで表示されます。
- 単色のタイル (`SingleColor`) のほとんどは背景です。内容を探すときは除外してください。
- `.bmc` ファイルの 8 ビットのタイルは、セッションのパレットの色で表されますが、キャッシュにはそのパレットが保存されていません。Juntei は Windows の既定のパレットを使うため、色が正しくない場合があります。圧縮されたタイルにはリモート デスクトップ プロトコルのインターリーブ RLE (MS-RDPBCGR 3.1.9) が使われており、Juntei は仕様の定義どおりに展開します。
- 64 × 64 のスロットより小さい `.bmc` タイルの後には、以前のタイルの末尾が残ります。Juntei はこの残りの行を別の画像 (`RemnantImage`) として保存します。これはスロット内のタイルではなく、より古いタイルの一部です。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-file rdp-cache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Cache????.bin and bcache*.bmc files
  -f, --file <FILE>      Parse one cache file, whatever its name

Output:
  -o, --output <FILE>       Save one record per tile
  -t, --output-type <TYPE>  csv, json, or jsonl
  -x, --extract <DIR>       Save each tile as a BMP and a PNG collage of each cache file
      --collage-only        With --extract, save only the collages
      --collage-width <N>   Tiles per collage row (default 64)
```
```bash
juntei parse-file rdp-cache -d evidence/ -o rdp-cache.csv --extract rdp-cache/
juntei parse-file rdp-cache -f Cache0000.bin --extract rdp-cache/ --collage-only
juntei parse-file rdp-cache -d evidence/ -t jsonl | jq 'select(.SingleColor == false)'
```
`-d` では、`Cache` の後に数字と `.bin` が続く名前のファイルと、`bcache` の後に数字と `.bmc` が続く名前のファイルを読み取ります。`RDP8bmp` シグネチャで始まらない `Cache????.bin` ファイルは別のプログラムのものなので、サマリーで件数に数えたうえでスキップします。`-f` では、名前に関係なくそのファイルを読み取ります。

### 画像 { #images }

`--extract` は、キャッシュ ファイルごとに `<DIR>/<user>/<cache file>/` というフォルダーを作成します。キャッシュ ファイルとそのボリューム シャドウ コピー内のコピーのように、2 つのキャッシュ ファイルのフォルダーが重なる場合は、2 つ目のフォルダーに番号が付きます (`Cache0001.bin_2`)。フォルダーには次のファイルが保存されます。

- `<cache file>_<index>.bmp`: 各タイル (それぞれのサイズのまま)。名前は bmc-tools が使うものと同じなので、RdpCacheStitcher でこのフォルダーを開けます。
- `<cache file>_old_<index>.bmp`: `.bmc` のみ。小さいタイルの後に残った、以前のタイルの末尾。
- `<cache file>_collage.png`: すべてのタイルをファイル内の順序で 1 行に 64 個 (`--collage-width`) ずつ並べたもの。各タイルは 64 × 64 のセルに左上から配置されます。インデックス *i* のタイルは、0 から数えて *i* / 64 行目、*i* % 64 列目にあります。セルの空いた部分 (幅の狭いタイルの横、高さの低いタイルの下、読み取れなかったタイルの位置) はマゼンタで塗りつぶされます。

`--collage-only` を指定すると、コラージュだけを保存します。既存の画像は `-C` なしでは置き換えられず、そのタイルの `Problems` にその旨が記録されます。キャッシュ ファイル自体は、`-C` を指定しても置き換えられることはありません。

## CSV 出力フィールド { #csv-output-fields }

| フィールド | 意味 |
|---|---|
| `SourceFile` | キャッシュ ファイル。 |
| `User` | キャッシュ ファイルがあるプロファイル フォルダーのユーザー (パスから判断)。 |
| `TileIndex` | ファイル内でのタイルの番号。0 から数え、画像の名前に含まれる番号と同じです。 |
| `Width`、`Height` | タイルのサイズ (ピクセル単位)。 |
| `SingleColor` | タイルのすべてのピクセルが同じ色の場合に `true`。 |
| `TileImage` | `--extract` 指定時: タイルの BMP ファイル。 |
| `Collage` | `--extract` 指定時: キャッシュ ファイルのコラージュ。 |
| `RemnantImage` | `.bmc` で `--extract` 指定時: スロットに残っていた以前のタイルの末尾。 |
| `Key` | リモート デスクトップ プロトコルにおけるタイルの 64 ビットのキー (16 進数)。 |
| `Format`、`FormatVersion` | `bin` とシグネチャの後にあるバージョン (Windows 10 以降は 6)、または `bmc`。 |
| `BitsPerPixel` | `.bin` は 32。`.bmc` は 8、16、24、32 のいずれか。 |
| `Compressed`、`Flags` | `.bmc`: タイルが圧縮されていたかどうかと、そのヘッダーのフラグ。 |
| `Offset` | ファイル内でタイルのヘッダーが始まる位置。 |
| `Problems` | 破損: タイルの途中で終わるファイル、どのタイルにもあり得ないタイル ヘッダー、タイルのサイズに合わないデータ、展開できない圧縮データ、保存できなかった画像。 |

JSON と JSONL のフィールドは同じで、数値と真偽値には型が付き、タイルにない値は `null`、`Problems` はリストになります。最後のタイルの後にあるデータなど、どのタイルにも属さない破損は、実行時のメッセージで報告されます。

## 自動解析 { #automatic-parsing }

`auto-parse` は `LateralMovement/RdpCache.csv` (`-t json` または `-t jsonl` の場合は `RdpCache.json` または `RdpCache.jsonl`) と `LateralMovement/RdpCache-logs.txt` を作成し、各キャッシュ ファイルのコラージュを `LateralMovement/RdpCache/` フォルダーに保存します。`--extract-rdp-tiles` を追加するとすべてのタイルも保存されますが、1 人のユーザーで数万ファイルになることがあります。ディスク イメージからは、キャッシュ ファイルがコピーされます。結合された `-j` タイムラインにはすべての行が `Artifact` 識別子 `rdp-cache` 付きで含まれます。タイルには日時がないため、[`super-timeline`](../commands/super-timeline.md) はタイルからイベントを作成しません。

## 調査戦略 { #investigation-strategy }

1. コラージュを開き、テキストを探します。コンピューター名やユーザー名、ファイル パス、コマンド、コンソール ウィンドウ、ブラウザーのアドレス バー、ダイアログ ボックスなどです。
2. `--extract` で保存したタイルのフォルダーを使い、RdpCacheStitcher で画面を再構成します。
3. クライアントのレジストリ (`parse-registry remote-access`)、`.rdp` ファイル、`mstsc.exe` の Jump Lists から、ユーザーが接続したコンピューターを特定します。それらのコンピューターで、対応する RDP ログオンとユーザーアクセスログ (UAL) のレコードを探します。
4. キャッシュ ファイルと、ボリューム シャドウ コピー内のそのコピーの `$MFT` の日時を使って、タイルが書き込まれた時期を絞り込みます。

## さらに詳しい情報 { #further-details }

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017: RDP ビットマップキャッシュ (フランス語)](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — RDP キャッシュ プラグイン](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9: Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — 永続ビットマップキャッシュ](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
