# Persistence Hunter 確認応答 { #persistence-hunter-acknowledgements }

Juntei の `persistence-hunter` コマンドは、2 世代にわたるコミュニティ永続性ハンティングに基づいて構築されています。研究者が自分たちの技術、実装、研究を利用できるようにしてくれた著者と寄稿者に感謝します。

## PyrsistenceSniper: Python プロジェクト { #pyrsistencesniper-the-python-project }

[**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper)、 による [ヘキサストライク](https://github.com/Hexastrike)は、この機能の直接の参照です。 Juntei そのポート 130- カタログをネイティブにチェックする Rust、デフォルトの許可リストとともに、 LOLBAS スナップショットと分類動作。参照バージョンは次の場所に固定されています [専念 `b99c117`](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f).

Python プロジェクトは MIT ライセンスを取得しています。 Juntei は、その著作権およびライセンス通知を [`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt) に保持します。 Juntei の実装は、Python インタープリターなしでオフラインで実行されます。収集の処理、対象範囲のレポート、不完全な証拠の保守的な解釈については、[ コマンド ガイド](../commands/persistence-hunter.md) に文書化されています。

## PersistenceSniper: オリジナルの PowerShell ツール { #persistencesniper-the-original-powershell-tool }

[**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper) は、PyrsistenceSniper に影響を与えたオリジナルの PowerShell 永続性ハンティング プロジェクトです。 PyrsistenceSniper のクレジットは Federico Lagrasta です。元のリポジトリには、Riccardo Ancarani と他の貢献者の功績も認められています。 Windows 永続化メカニズムを調査しやすくしてくれた元のプロジェクトとその貢献者に感謝します。

Junteiの直接移植リファレンスは、 Python プロジェクト。オリジナル PowerShell ツールはその系譜の功績であると考えられています。 Juntei によって提案されたメカニズムも独自に実装します。 [ホンソン11698](https://github.com/hongson11698) 元のプロジェクトでは [スケジュールされたタスク COM 解決 PR #31](https://github.com/last-byte/PersistenceSniper/pull/31) そして [スタートメニューライブラリ/CLSID-フォルダ PR #32](https://github.com/last-byte/PersistenceSniper/pull/32)。これらに加えて、 Python 参考カタログ。これは、いずれかのアップストリーム プロジェクトのすべてのライブ システム機能が、 Junteiのオフライン コマンド。

## 研究とツールのサポート { #supporting-research-and-tools }

- [MITRE ATT&CK](https://attack.mitre.org/) は、カタログに掲載されている技術識別子を提供します。テクニック マッピングは、メカニズムの可能な使用法を記述します。一致するレジストリ値またはファイル自体は、ATT&CK で確認された攻撃ではありません。
- [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns) は、Windows 自動起動を調べるためのインスピレーションとして PyrsistenceSniper によっても認められています。
- [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS) には、悪用される可能性のある正規の Windows バイナリが文書化されています。これらのバイナリのいずれかを認識すると、コマンドの解釈に役立ちますが、そのコマンドが自動的に悪意のあるものになるわけではありません。

Juntei 全体で使用されるレジストリ、NTFS、シェル リンク、イベント ログ、およびその他のパーサー基盤については、[プロジェクト全体の確認応答](../acknowledgements.md)を参照してください。

[永続化メカニズムのガイド](index.md)に戻ります。
