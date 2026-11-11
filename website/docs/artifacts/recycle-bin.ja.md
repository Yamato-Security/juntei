# ごみ箱 { #recycle-bin }

## ごみ箱に記録される情報 { #what-the-recycle-bin-records }

ユーザーがエクスプローラーでファイルやフォルダーを削除すると、Windows はそれを同じボリュームのごみ箱に移動し、その横に小さなインデックス レコードを書き込みます。レコードには、元のパス、サイズ、削除日時が記録されます。ごみ箱のフォルダーはボリュームごと・ユーザーごとに作られ、ユーザーの SID が名前になっているため、どのアカウントが削除したかもわかります。

| Windows のバージョン | インデックス | 削除された内容 |
|---|---|---|
| Vista 以降 | 項目ごとに 1 つの `$I…` ファイル (Vista から 8.1 はフォーマット バージョン 1、Windows 10 以降はバージョン 2) | `$R…` (`$R` の後ろは同じ名前): ファイル、またはフォルダーとその中身 |
| NT 4.0、2000、XP、Server 2003 | ごみ箱ごとに 1 つの `INFO2` ファイル: ANSI パスと Unicode パスを持つ 800 バイトのレコード | `D<drive letter><index><extension>` (例: `Dc1.txt`) |
| 95、98、ME | 1 つの `INFO2` ファイル (Windows 95 では `INFO`): ANSI パスのみの 280 バイトのレコード | 同じ `D…` 形式の名前 |

<span class="evidence-strength">この証拠から判断できること:</span> そのレコードがあるごみ箱のアカウントが、記録された日時に、元のパスにあった項目をごみ箱に移動しました。Shift+Delete、コマンドライン、またはほとんどのプログラムによる削除はごみ箱を経由せず、レコードは残りません。

## 保存場所と収集 { #locations-and-collection }

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

すべてのボリュームのごみ箱から `$I` ファイルまたは INFO2 を収集してください。`$R` ファイルや `D…` ファイルも収集すると、削除された項目のうちどれがまだ残っているかがわかります。収集しない場合、すべての項目で `RecycledFileFound` は `false` になります。削除された内容はそれ自体が証拠であり、マルウェアの場合もあるため、そのつもりで取り扱ってください。

## 分析時の注意点 { #analyst-cautions }

- `DeletedTime` は、項目がごみ箱に移動された日時で、システムの時計による UTC の FILETIME です。ごみ箱が空にされた日時ではありません。項目自体の作成日時や更新日時は `$MFT` と `$R` ファイルにあります。
- SID は項目を削除したアカウントであり、必ずしもファイルの所有者ではありません。SOFTWARE ハイブの `ProfileList` にプロファイルがない SID (削除されたアカウントや、一度もログオンしていないドメイン アカウントなど) には `User` がありません。
- ごみ箱が空にされたとき、または項目が元に戻されたりごみ箱から削除されたりしたとき、Windows Vista 以降はその `$I` ファイルと `$R` ファイルを削除します。それらの名前は USN ジャーナル、`$MFT`、`$I30` のスラックに残ることがありますが、Juntei はそこから削除された `$I` レコードを復元しません。INFO2 はこうしたレコードを残し、ANSI パスの先頭 1 バイトを消去します。この場合 `RemovedFromBin` が `true` になり、Juntei はその文字をドライブ番号から復元します。
- `$I` ファイルの `FileSize` は項目のサイズで、フォルダーの場合は中身の合計サイズです。INFO2 では、項目がディスク上で占めていた領域をクラスター単位で表します (小さなファイルなら 4,096 バイト)。
- INFO2 の ANSI パスは 8.3 形式であることが多く、システムのコード ページが必要です。日本語版 Windows には `--code-page 932`、中国語版には 936 または 950 を使用してください。Unicode パスがある場合は、それが `OriginalPath` になります。
- 一部の Windows Vista システムは、サイズ フィールドが 1 バイト足りない 543 バイトの `$I` ファイルを書き込みます。Juntei はその日時とパスを読み取り、`FileSize` は空にして問題として報告します。
- `RecycledFileFound` はエンドポイントではなく収集データについての値です。`false` は、収集ツールが `$R` ファイルを取得しなかったこと、または項目がもうごみ箱にないことを意味する場合があります。
- システム以外のボリュームのごみ箱では、収集メタデータや既知のドライブ／ボリュームのフォルダー構成から同じコンピューターと確認できる場合だけ、そのプロファイルを使用します。対応が曖昧な場合、ユーザー名は不明のままにします。別のホストのプロファイルを流用することはありません。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

`-d` では、`$I` の後に英数字 6 文字が続く名前 (拡張子は任意) のファイル、`INFO2`、および `RECYCLED` フォルダー内の `INFO` を読み取ります。`-f` では、名前に関係なくそのファイルを読み取ります。

## CSV 出力フィールド { #csv-output-fields }

| フィールド | 意味 |
|---|---|
| `SourceFile` | `$I` ファイルまたは INFO2 ファイル。 |
| `DeletedTime` | 項目がごみ箱に移動された日時 (UTC)。 |
| `OriginalPath` | 項目があった場所。INFO2 では、レコードに Unicode パスがあればそれを、なければ ANSI パスを使います。 |
| `FileSize` | `$I`: 項目のサイズ (バイト単位、フォルダーは合計)。INFO2: ディスク上の領域 (クラスター単位)。読み取れない場合は空です。 |
| `User`、`SID` | ごみ箱フォルダーの SID と、その `ProfileList` プロファイルの名前になっているユーザー。 |
| `RecycledFile`、`RecycledFileFound` | インデックスの横にある削除された内容のパス (`$R…`、`D<drive><index><extension>`) と、それが収集データに含まれているかどうか。 |
| `Format`、`FormatVersion` | `$I` の 1 (Vista から 8.1) または 2 (Windows 10 以降)、`INFO2` の 0 (95)、2 (NT 4.0)、4 (98)、5 (ME、2000 から Server 2003)。 |
| `RecordIndex`、`Drive`、`RecordOffset` | INFO2: レコードの番号 (`D…` ファイル名に含まれる番号)、ドライブ、INFO2 内でのレコードの開始位置。UNC パスにはドライブがありません。 |
| `RemovedFromBin` | INFO2: 項目が元に戻されたかごみ箱から削除された場合に `true`。 |
| `ShortPath` | INFO2: `OriginalPath` と異なる場合の ANSI パス (多くは 8.3 形式の名前)。 |
| `Problems` | 破損: 切り詰められたファイルや長すぎるファイル、無効なドライブ番号、有効な UTF-16 ではないパス、日時として解釈できない削除日時。 |

JSON と JSONL のフィールドは同じで、数値と真偽値には型が付き、レコードにない値は `null`、`Problems` はリストになります。

ごみ箱ディレクトリを列挙できない場合や、対応するファイル名が曖昧な場合、`RecycledFileFound` は不明（CSV では空欄、JSON では `null`）となり、原因を `Problems` に記録します。ハードリンクを含めて、ごみ箱の内容を出力で上書きすることや、削除されたフォルダー内への出力を拒否します。インデックスを一つも読み取れない場合、既存の結果を保持します。

## 自動解析 { #automatic-parsing }

`auto-parse` は `FileExistence/RecycleBin.csv` (`-t json` または `-t jsonl` の場合は `RecycleBin.json` または `RecycleBin.jsonl`) と `FileExistence/RecycleBin-logs.txt` を作成します。指定した `--code-page` は INFO2 に適用されます。ディスク イメージからは `$I` ファイルと INFO2 がコピーされ、`$R` ファイルと `D…` ファイルは空のプレースホルダーになるため、`RecycledFileFound` も埋まります。結合された `-j` タイムラインにはすべての行が `Artifact` 識別子 `recycle-bin` 付きで含まれ、`-Z` は `DeletedTime` を変換します。[`super-timeline`](../commands/super-timeline.md) は項目ごとに `Recycle Bin deletion (moved to the bin)` イベントを 1 つ作成します。

## 調査戦略 { #investigation-strategy }

1. ユーザーごとに `DeletedTime` で並べ替え、調査対象の活動の日時と比較します。
2. `OriginalPath` を LNK、ジャンプ リスト、ShellBags、RecentDocs で検索し、削除前にその項目が使われていたかを確認します。
3. USN ジャーナルでは、削除は `DeletedTime` ごろの `$Recycle.Bin` への名前変更として現れます。項目自体の日時は `$MFT` からわかります。
4. `RecycledFileFound` が `true` の場合は、`$R` または `D…` の内容のハッシュを取り、調べます。
5. ごみ箱が空にされていた場合は、USN ジャーナルと `$MFT` で `$I` と `$R` の名前を検索します。

## さらに詳しい情報 { #further-details }

- [libyal dtformats — Windows Recycle.Bin file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — Recycle Bin and INFO2 parsers](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
