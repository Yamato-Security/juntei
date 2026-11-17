# PowerShell 履歴 { #powershell-history }

## 履歴に記録される内容 { #what-the-history-records }

PSReadLine は、対話型 PowerShell ホストに入力されたコマンドラインを保存します。ファイルには保存順のテキストが含まれ、コマンドごとのタイムスタンプはありません。

## 保存場所と収集 { #locations-and-collection }

Windows PowerShell 5.1 と PowerShell 7+ の Windows 上の既定の保存場所は次のとおりです。

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

他のホストでは、同じディレクトリ内で `Visual Studio Code Host_history.txt` などの名前を使用します。Linux と macOS の PowerShell は通常、`~/.local/share/powershell/PSReadLine/<Host>_history.txt` を使用します。PSReadLine の `HistorySavePath` で保存先を変更できます。これらの既定パスは検出時の想定であり、すべての保存場所を網羅しているわけではありません。

ディレクトリ入力と `auto-parse` は、`PSReadLine` フォルダの直下にある `<Host>_history.txt` を再帰的に検出します。ASCII の大文字と小文字は区別しません。それ以外の場所にある任意の `*_history.txt` は選択しません。名前を変更したファイルや `PSReadLine` ディレクトリ外のコピーには、明示的に `-f` を指定してください。ユーザーを特定できるよう、元のプロファイル構造をできる限り保持してください。

## 分析時の注意点 { #analyst-cautions }

- どの行にも日時はありません。`FileLastWriteTime` はファイルの更新日時です。PSReadLine は既定でコマンドごとに履歴を保存する（`HistorySaveStyle` が `SaveIncrementally`）ため、この日時は通常、最後の行が追加された時刻に近くなります。ただし、それより前の行の日時については何もわかりません。
- 保存された行は、コマンドが実行されたことや正常に終了したことを証明するものではありません。実行の有無と時刻は、PowerShell イベントログ、プロセステレメトリ、その他の実行アーティファクトと照合してください。
- 履歴は編集、削除、無効化（`HistorySaveStyle` が `SaveNothing`）が可能です。また、PSReadLine は `password`、`token`、`apikey` を含む行など、機密情報を含むと思われる行を保存しません。Windows PowerShell ISE は PSReadLine を使用しません。スクリプトや、`-Command` または `-EncodedCommand` で実行したコマンドは入力されたものではないため、保存されません。
- 複数行のコマンドは、最後の行を除く各行の末尾にバッククォートを付けて保存されます。Juntei はそれらの行を別々のレコードとして保持し、`EntryStartLine` で関連付けます。ユーザーが入力したバッククォートで終わる行も同じように見えるため、1 つのエントリの行はまとめて読んでください。

## Juntei で解析する { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

既定のファイル形式は CSV です。`-t csv`、`-t json`、`-t jsonl` のいずれかを指定して `-o` を省略すると、レコードを標準出力に書き込みます。両方とも指定しなければ、概要のみを表示します。JSON は個別に整形されたオブジェクト、JSONL は 1 行に 1 オブジェクトです。既存の出力を上書きするには `-C` を追加します。ソースファイルを指す別名のパスは置換から保護されます。

## 文字エンコーディングと行の処理 { #encoding-and-line-handling }

- ファイル先頭の UTF-8、UTF-16LE、UTF-16BE のバイトオーダーマークでエンコーディングを選択し、そのマークを除去します。BOM がなければ常に UTF-8 として処理します。BOM のない UTF-16 は推測しません。NUL 文字は保持し、エンコーディング破損の可能性を報告します。
- 不正な UTF-8 シーケンスや対になっていない UTF-16 サロゲートは U+FFFD に置換し、`Problems` に破損を報告します。UTF-16 の末尾の余った 1 バイトや読み取りエラーも報告し、読み取れる行は保持します。完全性を判断する前に `Problems` とパーサーログの両方を確認してください。
- ファイルはストリーミング処理します。物理行ごとに保持する入力は最大 1,048,576 バイトで、BOM と LF 区切りは除外します（LF の前の CR は上限に含みます）。長い行は文字の境界で切り詰め、残りのバイトも読み進めます。`Problems` には省略したバイト数を記録します。この上限は文字数ではなく、エンコードされた入力のバイト数に適用します。
- LF と CRLF の改行を除去します。改行なしで終わる行の末尾の CR も除去します。他の空白とタブは保持します。空行はレコードを生成しませんが、行番号には数え、継続行の連鎖を終了します。空白のみの行はレコードとして残します。読み取りエラー時には、読み取りが止まった位置を示す空の診断行を生成することがあります。
- 行は分離したまま、バッククォートとインデントを保持します。`EntryStartLine` は、直前の行がバッククォートで終わる連続した行の連鎖の先頭を指します。シェル構文の解釈やコマンドの再構成は行いません。切り詰めは継続行の対応関係に影響する場合があります。このような行をまとめる前に `Problems` を確認してください。

## CSV 出力フィールド { #csv-output-fields }

| フィールド | 解釈 |
|---|---|
| `SourceFile` | ディレクトリ入力からの相対パスで、区切りは `/`。`-f` の場合はファイル名のみです。絶対ソースパスは出力しません。 |
| `User` | パスから判断したプロファイル名。`AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine`（または `.local/share/powershell/PSReadLine`）の 1 つ上のフォルダ名をユーザー名とします。入力がそのプロファイルや `PSReadLine` フォルダ自体の場合も同様です。ディレクトリ入力では入力ルート配下の `Users`/`Documents and Settings` 構造も認識します。不明な値は CSV では空、JSON では `null` です。 |
| `ShellHost` | `_history.txt` より前のファイル名部分。例は `ConsoleHost`。名前を変更してこの接尾辞がないファイルでは不明です。 |
| `LineNumber` | 1 から始まる物理行番号。省略された空行も含みます。 |
| `EntryStartLine` | 継続行の連鎖の開始物理行番号、またはこの行自身の番号。 |
| `Command` | 改行を除去したデコード済みの物理行。文書化された上限が適用されます。 |
| `FileLastWriteTime` | 各行に繰り返し記録するソースファイルの更新時刻。**ファイルのメタデータであり、コマンド実行時刻ではありません。** イメージ解析ではステージング時刻ではなく、元のソースメタデータを使用します。 |
| `FileSize` | ソースファイルのサイズ（バイト）。 |
| `Encoding` | `UTF-8`、`UTF-8-BOM`、`UTF-16LE`、`UTF-16BE`。 |
| `Problems` | 行の破損や切り詰めの診断。CSV ではセミコロン区切り、JSON/JSONL では配列です。ファイル単位の診断はログにも記録します。 |

JSON と JSONL のフィールドは同じで、数値には型が付き、不明な値は `null`、`Problems` はリストになります。どの形式でも、コマンドのテキストは保存されたとおりに保持します。

`FileLastWriteTime` は既定で UTC の ISO 8601 形式です。例は `2026-10-20T08:46:38.5Z`。表示するオフセットを変更する `-Z, --timezone` は `auto-parse` のみで使用できます。単独の `parse-file powershell-history` コマンドにはタイムゾーンオプションがありません。

## 自動解析 { #automatic-parsing }

`auto-parse` は `ShellHistory/PowerShellHistory.csv`（`-t json` または `-t jsonl` の場合は `PowerShellHistory.json` または `PowerShellHistory.jsonl`）と `ShellHistory/PowerShellHistory-logs.txt` を作成します。VHDX と E01 イメージからは履歴ファイルがコピーされ、`FileLastWriteTime` はイメージに記録された日時になります。結合された `-j` タイムラインにはすべての行が `Artifact` 識別子 `powershell-history` 付きで含まれます。これらの行には日時がないため、[`super-timeline`](../commands/super-timeline.md) はそこからイベントを作成せず、時刻なしレコードとして数えます。

## 調査戦略 { #investigation-strategy }

1. `Windows\System32\config\systemprofile` や `Windows\ServiceProfiles` 配下のプロファイルも含め、すべてのプロファイルの履歴を読みます。これらのプロファイルには、それぞれのアカウントとして実行されたシェルで入力されたコマンドが記録されています。
2. 複数行のエントリはまとめて読み、ダウンロード、エンコードされたコマンド、セキュリティ設定の変更、アカウントや資格情報に関するコマンド、リモート接続を探します。
3. PowerShell イベントログ（スクリプトブロックログ、イベント 4104）、プロセス作成イベント、`powershell.exe` と `pwsh.exe` の Prefetch ファイルと照合します。
4. `FileLastWriteTime` を履歴ファイルの `$MFT` と USN ジャーナルの日時、およびログオンと比較して、最後の行が追加された時期を絞り込みます。

## さらに詳しい情報 { #further-details }

- [Microsoft — about_PSReadLine: コマンド履歴](https://learn.microsoft.com/en-us/powershell/module/psreadline/about/about_psreadline)
- [Microsoft — Set-PSReadLineOption: HistorySavePath と HistorySaveStyle](https://learn.microsoft.com/en-us/powershell/module/psreadline/set-psreadlineoption)
- [Microsoft — about_Logging_Windows: スクリプトブロックログ](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [PSReadLine — 履歴ファイルの保存方法と読み込み方法](https://github.com/PowerShell/PSReadLine/blob/master/PSReadLine/History.cs)
