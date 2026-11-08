# アーティファクトガイド { #artifact-guide }

Windows は、ファイル システム、レジストリ、互換性データベース、ユーザー インターフェイス履歴、リソース アカウンティング、およびセキュリティ製品に重複するトレースを保存します。 Juntei はこれらのソースを解析しますが、調査者は依然として各ソースの意味を解釈する必要があります。

## 質問によってアーティファクトを選択してください { #choose-an-artifact-by-question }

| 調査質問 | ここから始めましょう | と裏付ける |
|---|---|---|
| この実行可能ファイルは実行されましたか? | [Prefetch](prefetch.md) | [ShimCache](shimcache.md)、[SRUM](srum.md)、イベント ログ |
| このファイルは存在したことがありますか? | [Amcache](amcache.md), [$MFT](mft.md) | [USN ジャーナル](usn-journal.md)、[$I30](i30.md)、ShimCache |
| ファイルが作成、名前変更、または削除されたのはいつですか? | [USN ジャーナル](usn-journal.md) | [$MFT](mft.md)、[$I30](i30.md)、Defender 隔離 |
| メタデータが変更される前に存在していた名前またはタイムスタンプは何ですか? | [$LogFile](logfile.md) | [$MFT](mft.md)、[USN ジャーナル](usn-journal.md)、[$I30](i30.md) |
| ファイルの所有者は誰ですか?また、ファイルを変更できるのは誰ですか? | [$Secure:$SDS](secure-sds.md) と [$MFT](mft.md) | [SAM](sam.md)、セキュリティ イベント ログ |
| ショートカットまたは Prefetch ファイルのターゲットはどのボリュームにありましたか? | [$Boot](boot.md) | [LNK](lnk.md)、[Prefetch](prefetch.md)、[USB デバイス](usb.md) |
| ユーザーは何を開きましたか? | [Jump Lists](jump-lists.md), [LNK](lnk.md), [RecentDocs](recentdocs.md) | SRUM, Prefetch |
| プログラムが大量のデータを転送しましたか? | [SRUM](srum.md) | ファイアウォール、プロキシ、DNS、パケットまたは EDR テレメトリ |
| ユーザーはどの Web サイトを閲覧し、何をダウンロードしましたか? | [Web ブラウザ](browser.md) | プロキシと DNS のログ、`$MFT` と USN ジャーナル、Defender、SRUM |
| このサーバーを利用したのはどのクライアントとアカウントで、それはどの日ですか? | [ユーザーアクセスログ (UAL)](ual.md) | セキュリティ イベント ログ、SMB と RDP のログ、ファイアウォール ログ |
| 永続性が設定されましたか? | [サービス](services.md)、[TaskCache](taskcache.md)、[レジストリ](registry.md) | イベントログ、Prefetch、`$MFT` |
| Defender は何を削除しましたか? | [Defender 隔離](defender-quarantine.md) | イベント ログ、`$MFT`、USN ジャーナル |
| ローカルアカウントは追加または変更されましたか? | [SAM](sam.md) | セキュリティ イベント ログとレジストリ トランザクション履歴 |

## フィールドの前に警告を読む { #read-warnings-before-fields }

すべてのアーティファクト ページは同じシーケンスに従います。

1. **それが何であるか**、またどのような質問に答えられるか。
2. **どこで見つかるか**、また収集がどのように影響するか。
3. **証明されるものと証明されないもの**
4. Juntei で**解析する方法**。
5. **すべての出力フィールドを読み取る方法。**
6. 独立した証拠と**関連付ける方法**。
7. **さらに詳細な**は低レベルの研究用です。

!!! danger "パーサーの検出結果を悪意のあるものと同一視しないでください"
    Juntei の検出結果では、`System32` の外にあるシステム バイナリ名など、確認する価値のあるパターンが特定されます。インストーラー、管理者、互換性レイヤー、およびソフトウェア更新により、異常なレコードが生成される場合があります。コンテキストと独立した証拠に基づいて検証します。
