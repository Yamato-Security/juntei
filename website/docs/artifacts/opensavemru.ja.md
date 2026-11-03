# 「開く」および「名前を付けて保存」ダイアログ (OpenSaveMRU) { #open-and-save-as-dialogs-opensavemru }

## 対話履歴とは { #what-the-dialog-history-is }

プログラムが一般的な「開く」ダイアログまたは「名前を付けて保存」ダイアログを表示すると、Windows はユーザーの `NTUSER.DAT` に次のように記録します。

- **開かれたまたは保存されたファイル:** `OpenSavePidlMRU` には拡張子ごとに 1 つのサブキー (`docx`、`ps1`、…) があり、`*` には拡張子ごとに、それぞれ最近使用されたリストがあります。
- **ダイアログを使用したプログラムとそのダイアログが含まれていたフォルダー:** `LastVisitedPidlMRU` (古いダイアログを使用するプログラムの場合は `LastVisitedPidlMRULegacy`)。
- **ダイアログを表示したプログラム:** `CIDSizeMRU` (プログラムごとのダイアログのサイズ)、および `FirstFolder` (プログラムのパスとフォルダー)。
- Windows 2000 および XP、`OpenSaveMRU` (拡張子ごとのパス) および `LastVisitedMRU` (パスとしてのプログラムとそのフォルダー)。

Windows Vista 以降、ファイルとフォルダーはシェル項目リストであり、ショートカットのターゲットと同じ構造です。各項目はパス (この PC、ドライブ、フォルダー、ファイル) の一部であり、ファイルまたはフォルダー項目には独自の時刻と NTFS ファイル参照を含めることができます。

<span class="evidence-strength">この証拠から判断できること:</span> このユーザーのセッションで、プログラムが「ファイルを開く」または「名前を付けて保存」ダイアログを使って、このファイル、あるいはこのフォルダ内のファイルを開く・保存する操作を行いました。各リストのキーの最終書き込み時刻で日時を判断できるのは、直近の項目だけです。

## 場所 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

プロファイルごとに `NTUSER.DAT` とそのトランザクション ログを収集します。

## 分析時の注意点 { #analyst-cautions }

- **リストごとに 1 回。** キーの最終書き込み時刻は、そのエントリの 1 つがそのリストの先頭に移動すると変更されるため、0 の位置にあるエントリのみに日付が付けられます (`IsMostRecent`、`LastUsedUTC`)。他のエントリはリストの中で新しいものから古いものへと順番に並んでいますが、日付はありません。
- **2 つのリストに同じファイルがあります。** ファイルはその拡張子のリストと `*` にあります。 2回の使用としてカウントしないでください。 `*` は内線全体の順序を示します。
- **開いた、または保存しました。** リストにはどちらが記載されていません。ファイルの `$MFT` および USN ジャーナル レコード、LNK ファイル、およびジャンプ リストと関連付けます。
- **Item の時間は、ダイアログではなくファイルの時間です。** `ItemModifiedUTC`、`ItemCreatedUTC`、および `ItemAccessedUTC` は、項目が書き込まれたときのファイルの時間です (2 秒 (FAT))。既知のフォルダー (`CLSID_ThisPCDocumentsRegFolder`、`Downloads` など) の項目には、多くの場合何もありません。これらの FAT タイムスタンプにはタイムゾーン オフセットがありません。 `UTC` 列のサフィックスは表示規則であり、元のクロックが UTC であることを示すものではありません。レジストリの FILETIME と比較する前に、ソース システムのタイムゾーンおよびその他のタイムスタンプと関連付けます。 Microsoft の [ ファイル時間ガイダンス ](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times) を参照してください。
- **Path はシェル パスです。** これらはデスクトップ、`This PC\C:\Users\…`、既知のフォルダー、ライブラリ (`UsersLibraries\Documents\…`)、またはネットワークの場所から始まります。フォルダーまたはファイルで始まるパスがデスクトップ上にあります。空のパスはデスクトップそのものです。
- **Items not decode.** シェル項目パーサーが認識できないタイプの項目は、そのクラス (`[class 0x00]`) によって表示されます。そのタイプは `unknown` で、リストのバイトは `DataRaw` にあります。
- **プログラムは名前です。** `LastVisitedPidlMRU` および `CIDSizeMRU` はパスではなくファイル名 (`WINWORD.EXE`) を保持します。 `FirstFolder` はパスを保持します。
- **Lリストは短く、クリーンアップされています。** 各リストは限られた数のエントリを保持し、クリーンアップ ツールはキーを削除します。削除された値には `parse-registry read --recover-deleted` を使用します。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry open-save-mru -d evidence/ -o opensavemru.csv
juntei parse-registry open-save-mru -f NTUSER.DAT -t jsonl \
  | jq 'select(.List == "OpenSavePidlMRU" and .Extension != "*")'
```

概要には、リストごとのエントリ、ダイアログを使用したプログラム、および各ハイブの最新のファイル (`*` の最初のエントリ) がリストされます。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | ホスト、プロファイル所有者 (パスまたはシェル フォルダーから)、システムの `ProfileList` からの SID、およびハイブ。 |
| `List` | `OpenSavePidlMRU`、`LastVisitedPidlMRU`、`LastVisitedPidlMRULegacy`、`CIDSizeMRU`、`FirstFolder`、`OpenSaveMRU`、または `LastVisitedMRU`。 |
| `Extension` | 拡張機能のサブキー (`docx`、`*`)。他のリストの場合は空です。 |
| `MRU-Position`, `IsMostRecent` | キーのリスト内の場所 (0 が最新です)。リストに値の名前がない場合は空です。 |
| `ValueName` | 値の数値 (`MRUListEx`) または文字 (`MRUList`)。 |
| `Program` | ダイアログを表示したプログラム。 |
| `Path` | ファイルまたはフォルダー: 結合されたシェル項目の名前、またはテキストとして保存されたパス。 |
| `ItemType` | 最後のシェル項目のタイプ: `file`、`directory`、`root_folder`、`volume`、`network_location`、`property_view`、`unknown`、… |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | ファイルまたはフォルダーの時間。そのシェル項目が記録したものです。 |
| `MFT-Entry`, `MFT-Sequence` | NTFS ファイルは、記録されたシェル項目を参照します。 |
| `LastUsedUTC` | 最新のエントリのみのキーの最終書き込み時刻。 |
| `KeyPath`, `KeyLastWrittenUTC` | キーとその最後の書き込み: すべての行ではなく、その最新のエントリの時刻。 |
| `DataRaw` | デコードされない場合は値のバイト (16 進数)、項目のタイプが不明な場合はシェル項目リスト。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | リストには名前が付いているがキーには付いていない値、名前が付けられていない値、間違った型、名前またはリストが短縮されています。 |

## 調査戦略 { #investigation-strategy }

1. `*` をユーザーごとに順番に読み取り、最新のファイルの場合は `LastUsedUTC` を読み取ります。
2. ユーザーが書き込み可能な場所、リムーバブルな場所、ネットワーク上の場所にあるスクリプト、アーカイブ、実行可能ファイル、ドキュメント、および認証情報やエクスポートなどの名前のファイルを探します。
3. プログラムを最後に使用したフォルダーに結び付けるには `LastVisitedPidlMRU` を使用し、ダイアログを使用したプログラムには `CIDSizeMRU` および `FirstFolder` を使用します。
4. パスと `MftEntry` を、`$MFT`、USN ジャーナル、LNK ファイル、ジャンプ リスト、ShellBags および RecentDocs と関連付けます。

## さらに詳しい情報 { #further-details }

- [libyal winreg-kb: 最近使用した (MRU) キー](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi: Windows シェル項目形式](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
