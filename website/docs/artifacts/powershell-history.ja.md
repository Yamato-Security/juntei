# PowerShell 履歴 { #powershell-history }

## 履歴に記録される内容 { #what-the-history-records }

PSReadLine は、対話型 PowerShell ホストに入力されたコマンドラインを保存します。ファイルには保存順のテキストが含まれ、コマンドごとのタイムスタンプはありません。履歴は編集、消去、無効化、フィルタリングが可能で、コマンドの正常終了を証明するものではありません。実行の有無と時刻は、PowerShell イベントログ、プロセステレメトリ、その他の実行アーティファクトと照合してください。

## 保存場所と収集 { #location-and-collection }

Windows PowerShell 5.1 と PowerShell 7+ の Windows 上の既定の保存場所は次のとおりです。

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

他のホストでは、同じディレクトリ内で `Visual Studio Code Host_history.txt` などの名前を使用します。Linux と macOS の PowerShell は通常、`~/.local/share/powershell/PSReadLine/<Host>_history.txt` を使用します。PSReadLine の `HistorySavePath` で保存先を変更できます。これらの既定パスは検出時の想定であり、すべての保存場所を網羅しているわけではありません。

ディレクトリ入力と `auto-parse` は、`PSReadLine` フォルダの直下にある `<Host>_history.txt` を再帰的に検出します。ASCII の大文字と小文字は区別しません。それ以外の場所にある任意の `*_history.txt` は選択しません。名前を変更したファイルや `PSReadLine` ディレクトリ外のコピーには、明示的に `-f` を指定してください。ユーザーを特定できるよう、元のプロファイル構造をできる限り保持してください。

## Juntei で解析する { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

既定のファイル形式は CSV です。`-t csv`、`-t json`、`-t jsonl` のいずれかを指定して `-o` を省略すると、レコードを標準出力に書き込みます。両方とも指定しなければ、概要のみを表示します。JSON は個別に整形されたオブジェクト、JSONL は 1 行に 1 オブジェクトです。既存の出力を上書きするには `-C` を追加します。ソースファイルを指す別名のパスは置換から保護されます。

`auto-parse` は、VHDX/E01 イメージの場合も `ShellHistory/PowerShellHistory.csv`（または選択した JSON 形式）に保存します。履歴は `auto-parse -j` や `super-timeline` のイベントにはなりません。

## 文字エンコーディングと行の処理 { #encoding-and-line-handling }

- ファイル先頭の UTF-8、UTF-16LE、UTF-16BE のバイトオーダーマークでエンコーディングを選択し、そのマークを除去します。BOM がなければ常に UTF-8 として処理します。BOM のない UTF-16 は推測しません。NUL 文字は保持し、エンコーディング破損の可能性を報告します。
- 不正な UTF-8 シーケンスや対になっていない UTF-16 サロゲートは U+FFFD に置換し、`Problems` に破損を報告します。UTF-16 の末尾の余った 1 バイトや読み取りエラーも報告し、読み取れる行は保持します。完全性を判断する前に `Problems` とパーサーログの両方を確認してください。
- ファイルはストリーミング処理します。物理行ごとに保持する入力は最大 1,048,576 バイトで、BOM と LF 区切りは除外します（LF の前の CR は上限に含みます）。長い行は文字の境界で切り詰め、残りのバイトも読み進めます。`Problems` には省略したバイト数を記録します。この上限は文字数ではなく、エンコードされた入力のバイト数に適用します。
- LF と CRLF の改行を除去します。改行なしで終わる行の末尾の CR も除去します。他の空白とタブは保持します。空行はレコードを生成しませんが、行番号には数え、継続行の連鎖を終了します。空白のみの行はレコードとして残します。読み取りエラー時には、読み取りが止まった位置を示す空の診断行を生成することがあります。
- PSReadLine は複数行の入力を、継続する各物理行の末尾にバッククォートを付けて保存します。行は分離したまま、バッククォートとインデントを保持します。`EntryStartLine` は、直前の行がバッククォートで終わる連続した行の連鎖の先頭を指します。シェル構文の解釈やコマンドの再構成は行いません。切り詰めは継続行の対応関係に影響する場合があります。このような行をまとめる前に `Problems` を確認してください。

## 出力フィールド { #output-fields }

| フィールド | 解釈 |
|---|---|
| `SourceFile` | ディレクトリ入力からの相対パスで、区切りは `/`。`-f` の場合はファイル名のみです。絶対ソースパスは出力しません。 |
| `User` | 保持されたパスから推定したプロファイル名。ディレクトリ入力では入力ルート配下の `Users`/`Documents and Settings` 構造も認識します。`-f` では通常の PSReadLine プロファイルパスの末尾構造が必要です。不明な値は CSV では空、JSON では `null` です。 |
| `ShellHost` | `_history.txt` より前のファイル名部分。例は `ConsoleHost`。名前を変更してこの接尾辞がないファイルでは不明です。 |
| `LineNumber` | 1 から始まる物理行番号。省略された空行も含みます。 |
| `EntryStartLine` | 継続行の連鎖の開始物理行番号、またはこの行自身の番号。 |
| `Command` | 改行を除去したデコード済みの物理行。文書化された上限と CSV 保護が適用されます。 |
| `FileLastWriteTime` | 各行に繰り返し記録するソースファイルの更新時刻。**ファイルのメタデータであり、コマンド実行時刻ではありません。** イメージ解析ではステージング時刻ではなく、元のソースメタデータを使用します。 |
| `FileSize` | ソースファイルのサイズ（バイト）。 |
| `Encoding` | `UTF-8`、`UTF-8-BOM`、`UTF-16LE`、`UTF-16BE`。 |
| `Problems` | 行の破損や切り詰めの診断。CSV ではセミコロン区切り、JSON/JSONL では配列です。ファイル単位の診断はログにも記録します。 |

`FileLastWriteTime` は既定で UTC の ISO 8601 形式です。例は `2026-10-20T08:46:38.5Z`。表示するオフセットを変更する `-Z, --timezone` は `auto-parse` のみで使用できます。単独の `parse-file powershell-history` コマンドにはタイムゾーンオプションがありません。

CSV の各テキストセルで、表計算ソフトの数式として扱われる接頭文字（`=`、`+`、`-`、`@`、それらの全角文字、および空白や特定の不可視文字・制御文字の後にある接頭文字）を確認します。該当するセルと制御文字で始まるセルには、先頭にアポストロフィを付けます。そのため、CSV のコマンドやパスのテキストは変わる場合があります。JSON と JSONL は CSV 用のアポストロフィを付けず、デコードしたテキストをそのまま保持します。ただし、デコード時の置換と切り詰めは適用されます。保存されたコマンドのテキスト比較には JSONL を使用し、元の履歴ファイルは必ず保存してください。
