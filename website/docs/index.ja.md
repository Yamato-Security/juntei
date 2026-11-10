---
hide:
  - navigation
  - toc
---

<div class="jt-hero" markdown>

![デジタルフォレンジックの女神 Juntei](assets/juntei-goddess.png){ .jt-hero-art width="1448" height="1086" }

<p class="tagline">
<strong>Juntei</strong> は、Yamato Security が開発する、イベントログ<em>以外</em>の Windows フォレンジックアーティファクトを解析するクロスプラットフォームツールです。収集した証拠を一貫した CSV・JSON・JSONL レコードに変換し、調査員が誤った結論を避けるために必要な情報を保持します。
</p>

<div class="jt-cta" markdown>
[はじめる :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[アーティファクトを学ぶ :material-archive-search:](artifacts/index.md){ .md-button }
[コマンドリファレンス :material-console:](commands/index.md){ .md-button }
[GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/juntei){ .md-button }
</div>

</div>

## 生のアーティファクトから根拠のある結論へ { #from-raw-artifact-to-defensible-conclusion }

Juntei はアーティファクトに何が記録されているかを示します。このサイトでは、その記録から**何を証明でき、何を証明できないか**を解説します。Windows DFIR の経験がなくても、まずアーティファクトのページから読んでみてください。各ガイドでは、用途、保存場所、記録の保持や収集時の注意点、解析コマンド、出力フィールド、解釈、他の証拠との関連付け、参考資料を説明します。

<div class="grid cards" markdown>

-   :material-file-tree:{ .lg .middle } __ファイルシステムの履歴__

    ---

    `$MFT` と USN ジャーナルから、ファイルの存在、名前、移動、削除、メタデータの変更、代替データストリームを調べます。

-   :material-play-circle-outline:{ .lg .middle } __プログラムに関する証拠__

    ---

    Prefetch は実行を裏付ける強い証拠になり得ます。Amcache と ShimCache は、インベントリ、パス、ハッシュ、互換性、過去の状況を補いますが、慎重な解釈が必要です。

-   :material-account-clock:{ .lg .middle } __ユーザーの活動__

    ---

    LNK、Jump Lists、RecentDocs、SRUM から、開いた項目、アプリの活動、リムーバブルドライブやネットワーク上の場所、リソース使用、過去のユーザーの状況を調べます。

-   :material-shield-bug:{ .lg .middle } __セキュリティに関する証拠__

    ---

    Defender の隔離データには、検出名、元のパス、ダウンロード元、ハッシュ、所有者が残る場合があります。隔離された内容を安全に復元し、隔離環境で調べることもできます。

-   :material-database-search:{ .lg .middle } __レジストリの状態__

    ---

    使用中・復元可能なレジストリデータを読み、トランザクションログを再生し、サービス、スケジュールタスク、ローカルアカウント、最近使った文書の状態を調べます。

-   :material-link-variant:{ .lg .middle } __推測ではなく、証拠を関連付ける__

    ---

    単一のアーティファクトだけで全容は分かりません。このガイドでは、独立した情報源を組み合わせ、証拠に基づくタイムラインを作る方法を紹介します。

</div>

!!! warning "フォレンジックの出力は証拠であり、判定結果ではありません"
    不審なパスは、調査の手掛かりです。タイムスタンプは、実行ではなくファイルのメタデータを示す場合があります。記録がないことは、操作が行われなかった証明にはなりません。元の証拠を保存し、ツールのバージョンとコマンドを記録して、重要な結論は別の証拠でも裏付けてください。
