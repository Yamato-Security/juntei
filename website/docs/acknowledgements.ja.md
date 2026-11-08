# 謝辞 { #acknowledgements }

Juntei が存在するのは、DFIR およびオープンソース コミュニティがパーサー、フォーマット調査、参照出力、仕様、およびテスト証拠を公開しているためです。以下のプロジェクトでは、実装、検証、フィールドの命名、または予想されるアナリストの出力について説明します。リストに掲載されていることは、そのコードが Juntei に埋め込まれていることを意味するものではありません。ライセンスについては各プロジェクトを参照してください。

## Yamato Security によって維持されるパーサーの基盤 { #parser-foundations-maintained-by-yamato-security }

| Junteiエリア | 保守されたパーサー | 上流・基礎 |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`mft` クレート) | [オマー・ベン・アムラム mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-プリフェッチ-パーサー](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| USN ジャーナル | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`usn` クレート) | [Sootmark usn](https://github.com/Sootmark/usn) |
| NTFS `$LogFile` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`logfile` クレート) | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| ディレクトリインデックス (`$I30`) | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`mft` クレート、`i30` モジュール) | 指数スラックに関する研究のクレジットは以下の通り |
| NTFS `$Boot`, `$Secure:$SDS` | [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`boot` および `secure` クレート) | Microsoft の MS-DTYP および NTFS の研究は以下にクレジットされています |
| SRUM、UAL / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) および [ese](https://github.com/Sootmark/ese) |
| Web ブラウザ / SQLite | [ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) | [Sootmark browser](https://github.com/Sootmark/browser)、[sqlite](https://github.com/Sootmark/sqlite) および [plist](https://github.com/Sootmark/plist) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark シェル](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-ジャンプリスト-パーサー](https://github.com/Yamato-Security/ys-jump-lists-parser) | Sootmark シェル複合ファイルとジャンプ リストが機能する |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [セキュリティ Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [コートマークディフェンダー](https://github.com/Sootmark/defender) |
| VHDX ディスクイメージ (`mount`) | [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) (Security Ronin `vhdx-core`) と [ntfs-core](https://github.com/SecurityRonin/ntfs-forensic)、[fuser](https://github.com/cberner/fuser) で提供 | Microsoft の MS-VHDX |
| レジストリ | [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser) | レジストリ形式と回復に関する研究のクレジットは以下に記載されています |

## 参照ツールと独立した検証 { #reference-tools-and-independent-validation }

- Hexastrike による [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper) は、永続性チェック カタログ、デフォルトのホワイトリスト、LOLBAS スナップショット、およびネイティブ Rust `persistence-hunter` コマンドの参照動作を提供しました。 MIT ライセンスと著作権表示は `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt` に保持されます。オリジナルの [PersistenceSniper](https://github.com/last-byte/PersistenceSniper) は、Python プロジェクトにインスピレーションを与えました。完全な系統については、[永続性確認](persistence-hunter/acknowledgements.md)を参照してください。

- Maxim Suhanov の **regf 仕様** および **yarp** は、レジストリ構造、トランザクション ログの再生、および削除されたレコードの回復を通知しました。彼の **dfir_ntfs** は、INDX レコードとインデックス スラック カービングの参照でした。
- Harel Segev の **INDXRipper** は、インデックス スラックからファイル名を切り出すためのさらなる参考資料です。
- **dfir_ntfs**、Joakim Schicht の **LogFileParser**、および Stroz Friedberg の **ntfs-linker** が通知`$LogFile` の検証と解釈。 Velocidex の **go-ntfs** は、公開された NTFS テスト証拠を提供しました。
- ヨアヒム・メッツ / リビア プロジェクト **libregf**、**libfsntfs**、**libscca**、 **libfwsi** および **libesedb** は、詳細な形式調査と独立した解析リファレンスを提供します。
- Fox-IT の **dissect.esedb**、**dissect.ntfs**、**dissect.target** は、検証で使用した SRUM、UAL、`$Secure:$SDS`、SAM、Defender の独立した解釈を提供しました。
- **Sleuth Kit** の `fsstat` は、`mkntfs` で作成されたボリュームの NTFS ブート セクターのリファレンスでした。
- Eric Zimmerman のツールの一部は、比較に使用されるフィールド名と検査者の期待値を確立しました。
- **regipy**、**python-registry**、**Notatin**、および **artemis** が提供するレジストリ実装、テスト、またはサンプル解釈。
- **LnkParse3**、AbdulRhmanAlfaifi の **lnk_parser** および **jumplist_parser**、および JLECmd は独立して提供されますLNK/ジャンプ リストの動作と公開テスト ケース。
- Security Ronin の **prefetch-forensic** および **forensicnomicon** プロジェクトは、Juntei の高価値 Prefetch/Amcache/ShimCache を通知しました。観察。
- **defender-dump** および dissect.target は、Defender 隔離構造と安全な抽出動作を理解するための個別の実装を提供します。
- **Plaso**、**Chainsaw**、および **Velociraptor** は、複数のアーティファクトにわたるパーサー比較、テスト証拠、またはコレクション指向の参照動作を提供しました。
- Obsidian Forensics の **Hindsight** と Velocidex の **SQLiteHunter** の定義は、ブラウザの閲覧履歴、ダウンロード、Cookie、フォーム入力履歴の参考資料となりました。

## 仕様と研究 { #specifications-and-research }

シェル リンク、複合バイナリ ファイル、NTFS/SAM プロトコル、レジストリ/サービス/タスク構造、および API ドキュメントのオープン仕様を公開してくださった Microsoft に感謝します。また、これらのアーティファクトを実務者に理解できるようにした AppCompatCache、Amcache、SRUM、Prefetch、レジストリ、および NTFS フォレンジック研究の作成者にも感謝します。

## テスト証拠 { #test-evidence }

Andrew Rathbun の [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum)、NIST CFReDS、Plaso、Regipy、Artemis、パーサー テスト リポジトリ、およびその他の適切にライセンスされたコレクションからの公開証拠により、実際の Windows データに対する回帰テストが可能になります。 Juntei は、付属のテスト ファイルの横に帰属/ライセンス ノートを保持します。

## 貢献者 { #contributors }

Juntei および Yamato Security パーサーの寄稿者、レビュー担当者、問題報告者、およびエッジ ケースを提供したユーザーの皆様に感謝します。現在の貢献者は、[Juntei 貢献者ページ](https://github.com/Yamato-Security/juntei/graphs/contributors) に GitHub によってリストされています。

## ライセンス { #license }

Juntei は、[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE) (RPL-1.5) および著作権白船セキュリティ株式会社に基づいてリリースされています。そのソース コードは、バージョン 1.0.0 よりも前に AGPLv3 で公開されていました。クレートおよびそれを使用して構築されたその他のサードパーティ コードおよびデータのライセンスは、[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES) にあります。
