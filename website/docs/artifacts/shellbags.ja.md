# ShellBags { #shellbags }

## ShellBagsとは { #what-shellbags-are }

エクスプローラーは、ユーザーが開いた各フォルダーの表示方法、ビュー、アイコン サイズ、並べ替え順序、ウィンドウの位置を記憶します。フォルダーの設定を再度検索するために、フォルダーの ID がシェル項目のツリーとしてユーザーのレジストリに保持されます。シェル項目は、リンクのターゲット ID リストに保持されているのと同じ項目 (ドライブ、次に開かれているフォルダーまでの各フォルダー) です。これらのツリーは ShellBags です。

エクスプローラーで開かれたフォルダー、および開くダイアログと保存ダイアログで開かれたフォルダー (リムーバブル ドライブ、ネットワーク共有、ポータブル デバイス (電話、カメラ)、ZIP アーカイブおよび FTP サーバー上のフォルダー、およびコントロール パネル ページを含む) が対象になります。これらは、フォルダー、ドライブ、またはネットワーク共有がなくなった後も残ります。

<span class="evidence-strength">この証拠から判断できること:</span> このユーザーの Explorer が、このフォルダ、または配下のフォルダを表示しました。ShellBags はフォルダ内で何をしたかを示さず、すべての訪問を時刻付けするものでもありません。下記の時刻の説明を参照してください。

## 場所 { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

`BagMRU` キーには、番号付きの値 (それぞれ 1 つのシェル項目) と、その順序 (最近開かれたものが最初) を示す `MRUListEx` 値が保持されます。各値にはその下の項目と同じ名前のサブキーがあるため、パスは `BagMRU` から値までの項目です。サブキーの `NodeSlot` は、フォルダーの表示設定が存在する `Bags\<N>` キーの番号です。

すべてのプロファイルのトランザクション ログ (`.LOG1`、`.LOG2`) と `UsrClass.dat` および `NTUSER.DAT` を収集し、`NTUSER.DAT` ユーザーの SID を提供するシステムの `SOFTWARE` ハイブを収集します。

## どのタイミングで何を言うか { #which-times-say-what }

- **シェル アイテムの時間** (`ItemModifiedUTC`、`ItemCreatedUTC`、`ItemAccessedUTC`) は、エクスプローラーがアイテムを書き込んだときのフォルダー自体のファイル システム時間 (FAT 精度: 2 秒) です。これらは、訪問時ではなくフォルダの日付を示し、その後の訪問では更新されません。 FAT クロックには UTC オフセットが格納されていません。従来の `UTC` 見出しと表示される `Z` 接尾辞は表示規則であり、ソース タイム ゾーンの証明ではありません。 `auto-parse --timezone` はこれらのフィールドを保持します。それらをソースシステムのコンテキストで解釈します。
- **A キーの最終書き込み時刻** は、`MRUListEx` が書き換えられるとき、つまり、その項目の 1 つが開かれて先頭に移動されるときに変更されます。したがって、項目の値が含まれるキーの時刻 (`ParentKeyLastWrittenUTC`) は、項目が最後に開かれたとき **は、そのキー内の最新の項目である場合のみ** (`IsMostRecentInParent` true、`MruPosition` 0) になります。他の項目については、兄弟が開かれた後の時期です。
- 項目自体のキー (`KeyLastWrittenUTC`) は、その最新の子が開かれた日付を同じように日付付けします。

## 分析時の注意点 { #analyst-cautions }

- **Absence はほとんど証明されません。** エクスプローラーは、フォルダーのビューが保存されるときに ShellBags を書き込みます。ほんの一瞬だけ表示されたフォルダー、または別の方法 (コマンド ライン、スクリプト、プログラム) で開かれたフォルダーには何もない可能性があります。ユーザーはキーを削除できます。
- **NTFS 参照は、項目が書き込まれたときのフォルダーの** (`MftEntry`、`MftSequence`) です。 `$MFT` でそれを検索し、同じフォルダーがまだ存在しているかどうか、または別のファイルがそのエントリを再利用しているかどうかを確認します。
- **リムーバブル ドライブ** は、ドライブ文字によって名前が付けられます。この手紙は当時ドライブにあったものです。これを USB およびデバイスのマウントされたデバイス履歴と照合します。
- **Names は item.** から取得されます。後で名前を変更したフォルダーは、ここに古い名前を保持します。ルート フォルダーと既知のフォルダーには、フォレンジック ツールの一般的な名前 (`This PC`、`Quick Access`、`Downloads`) と同じ名前が付けられます。 `Guid` はその識別子を示します。
- **検索結果、ライブラリ、およびコントロール パネル ページ** はビューであり、ディスク上のフォルダーではありません (`property_view`、`control_panel_item`、`control_panel_category`)。
- **パーサーが知らない項目**は、そのバイトを`DataRaw`に保持し、`ItemType` `unknown`とそのクラスのバイトを`ItemClass`に保持します。破損したアイテムや異常なアイテムは `Problems` で報告され、それらのバイトも保持されます。
- **A ダーティ ハイブ** には、ログがないと最新のアイテムが不足している可能性があります。 Juntei は、`--no-logs` でない限り、ハイブの隣のログを再生します。 `TransactionLogStatus` は、レコードが彼らからのものであるかどうかを示します。

## Juntei で解析する { #parse-with-juntei }

```text
Usage: juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shell-bags -d evidence/ -o shellbags.csv
juntei parse-registry shell-bags -f UsrClass.dat -t jsonl \
  | jq 'select(.ItemType == "volume" and .GUID == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

シェル項目は、`parse-file lnk` のシェル項目パーサーである ys-lnk-parser でデコードされます。概要には、親キーの最終書き込み時刻から、ユーザーが最後に開いたフォルダー (新しい順) がリストされます。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath`, `HiveType` | ホスト、プロファイル フォルダーの名前、ユーザーの SID (`UsrClass.dat` のルート キーによって名前が付けられます。`NTUSER.DAT` の場合はシステムの `ProfileList`)、ハイブとそのタイプ。 |
| `BagKey` | ハイブのルート (`…\BagMRU\4\3\0`) の下にある項目のキー: その値はその上のキーにあります。 |
| `MRU-Position`, `IsMostRecentInParent` | 親キーの `MRUListEx` (0: 最後に開かれたキー) 内のその場所。リストにない場合は空になります。 |
| `NodeSlot` | `Bags\<N>` キーの番号 (フォルダーの表示設定)。 |
| `Path` | ツリーの最上部からそのツリーまでの項目が `\` で結合されます。 `[unknown]` は、値が欠落している項目を表します。 |
| `ItemType`, `ItemClass` | アイテムとは (`root_folder`、`volume`、`directory`、`file`、`network_location`、`uri`、`ftp_folder`、`compressed_folder`、`portable_device`、 `portable_device_folder`、`users_files_folder`、`property_view`、`control_panel_item`、`control_panel_category`、`cd_burn`、`unknown`) とそのクラス バイト。 |
| `ItemName`, `GUID` | その名前 (フォルダーの長い名前)、およびルート フォルダー、コントロール パネル項目、または代理フォルダーのクラス識別子。 |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | **シェル項目から:** 項目が書き込まれたときのフォルダーの時刻 (FAT、2 秒の精度)。 |
| `MFT-Entry`, `MFT-Sequence` | **シェル項目から:** フォルダーの NTFS ファイル参照。 |
| `ExtensionBlocks` | アイテムにある拡張ブロックの数。 |
| `KeyLastWrittenUTC` | **レジストリから:** 項目自体のキーの最後の書き込み: 最新の子が開かれたとき。 |
| `ParentKeyLastWrittenUTC` | **レジストリから:** その値が含まれるキーの最後の書き込み: 最後に開かれたとき、`IsMostRecentInParent` が true の場合。 |
| `DataRaw` | 完全に理解されていない項目、または問題がある項目の値のバイト数 (16 進数)。 |
| `TransactionLogStatus`, `LogsApplied` | レジストリ回復コンテキスト。 |
| `Problems` | キーの `MRUListEx` に名前が付けられているがキーに名前が付けられていない値、リストに名前が付けられていない値、値のないサブキー、バイナリではない値、間違ったタイプの `NodeSlot`、破損したシェル項目リスト。 |

## 調査戦略 { #investigation-strategy }

1. リムーバブル ドライブ (`volume` とリムーバブル ドライブ `Guid`)、ネットワークの場所、FTP フォルダとポータブル デバイス、およびその下のフォルダを一覧表示します。
2. 関心のあるフォルダーを探します。ユーザーが書き込み可能なステージング場所 (`Temp`、`ProgramData`、`Public`)、ツール フォルダーとアーカイブ フォルダー、他のユーザーのプロファイル、および存在しなくなったフォルダー (`$MFT` の NTFS 参照を確認してください)。
3. `IsMostRecentInParent` が true の項目の `ParentKeyLastWrittenUTC` でのみ訪問する日付。残りを、ファイルを開いた日付である LNK ファイル、Jump Lists および RecentDocs と関連付けます。
4. 削除された `BagMRU` キーと値には `parse-registry read --recover-deleted` を使用します。

## さらに詳しい情報 { #further-details }

- [libyal libfwsi: Windows シェル項目形式](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb: 最近使用された (BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

`ProfileList` からの SID ルックアップには、構文的に有効な一意の SID が 1 つ必要です。 `S-1-...bak` などのプロファイルとフォルダーのマッピングとバックアップ キーが曖昧な場合、`UserSid` は不明のままになります。
