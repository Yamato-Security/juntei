# スクリーンショット { #screenshots }

Juntei のコマンドライン、自動解析、調査結果のフォルダ構成を紹介します。
画像をクリックすると、元のサイズで表示できます。

## 1. 起動とコマンド { #1-startup-and-commands }

<figure markdown>

[![Juntei の起動画面。CODE BLUE リリースと利用可能なコマンド](assets/screenshots/01-Startup.png)](assets/screenshots/01-Startup.png)

<figcaption>起動画面には、自動解析、データ形式別のパーサー、Persistence Hunter が表示されます。</figcaption>

</figure>

## 2. 20 秒未満で自動解析 { #2-automatic-parsing-in-under-20-seconds }

以下は、**MacBook Pro M4** で、**6 台のシステム**から収集した **8.7 GiB の Windows データ**を解析し、
**4.4 GiB の出力を 20 秒未満で保存**した例です。[auto-parse](commands/auto-parse.md) コマンドが、
16 個の並列プロセスで 240 個の解析タスクを実行し、482 個の結果ファイルを作成しました。
**210 タスクが成功し、30 タスクは対象アーティファクトが未収集のためスキップされ、失敗は 0 件でした。**

<figure markdown>

[![6 台のシステムと 8.7 GiB の入力を解析し、4.4 GiB の結果を 20 秒未満で出力した Juntei auto-parse](assets/screenshots/02-AutoParse.png)](assets/screenshots/02-AutoParse.png)

<figcaption>ホストの自動検出と並列解析。完了時には、入力・出力サイズ、タスクの結果、所要時間が表示されます。</figcaption>

</figure>

## 3. 調査しやすい結果の構成 { #3-results-organized-for-investigation }

<figure markdown>

[![AZUCHI の結果フォルダ。DFIR カテゴリ別に整理され、アカウントの CSV と解析ログが並ぶ](assets/screenshots/03-OutputFolders.png)](assets/screenshots/03-OutputFolders.png)

<figcaption>結果はコンピュータごとに DFIR カテゴリ別に整理されます。アーティファクトの CSV と対応する <code>-logs.txt</code> が並んでいるので、レコードと解析の詳細をすぐに確認できます。</figcaption>

</figure>
