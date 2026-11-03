# 記事と参考文献 { #articles-and-references }

このページには、アーティファクト ガイドのさらに先を進みたい読者のための、質の高い出発点が集められています。アーティファクトごとの **「詳細」** セクションには、最もターゲットを絞った低レベルのリンクが含まれています。

## 初心者向け DFIR オリエンテーション { #beginner-dfir-orientation }

- [SANS Windows フォレンジック分析ポスター](https://www.sans.org/posters/windows-forensic-analysis/) — 一般的な Windows アーティファクトと場所のビジュアル マップ。
- [SANS Hunt Evil ポスター](https://www.sans.org/posters/hunt-evil/) — 実用的な狩猟の要点と Windows の調査リマインダー。
- [DFIR アーティファクト ミュージアム](https://github.com/AndrewRathbun/DFIRArtifactMuseum) — 学習およびパーサー検証用の公開アーティファクト サンプル。
- [Velociraptor アーティファクト リファレンス](https://docs.velociraptor.app/artifact_references/) — 多くの Windows アーティファクトのコレクション指向の説明と VQL 実装。
- [Plaso ドキュメント](https://plaso.readthedocs.io/) — クロスアーティファクト分析のためのタイムラインの概念とパーサー。

## ファイルシステムのアーティファクト { #file-system-artifacts }

- [Microsoft: マスター ファイル テーブル](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft オープン仕様: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft: 変更仕訳帳](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: セキュリティ記述子](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)および[SDDL](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft 公開仕様: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) — セキュリティ記述子、ACE、および条件式。
- [libfsntfs 形式のドキュメント](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [Omer Ben-Amram の mft パーサー](https://github.com/omerbenamram/mft)
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) — INDX レコードとインデックス スラック カービングを備えた NTFS パーサー。
- [INDXRipper](https://github.com/harelsegev/INDXRipper) — ファイル名に `$I30` のスラックを刻みます。
- [LogFileParser](https://github.com/jschicht/LogFileParser) — 詳細な NTFS トランザクション ログの調査と解析。
- [ntfs-linker](https://github.com/strozfriedberg/ntfs-linker) — `$LogFile`、`$MFT`、および USN の履歴を関連付けます。

## 実行およびインベントリのアーティファクト { #execution-and-inventory-artifacts }

- [エリック・ジマーマンの AmcacheParser](https://github.com/EricZimmerman/AmcacheParser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [libyal libscca](https://github.com/libyal/libscca) — Prefetch フォーマット/ライブラリ作品。
- [エリック・ジマーマンの PECmd](https://github.com/EricZimmerman/PECmd)
- [Mandiant: フォレンジック調査におけるアプリケーション互換性キャッシュの活用](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [エリック・ジマーマンの AppCompatCacheParser](https://github.com/EricZimmerman/AppCompatCacheParser)

## ユーザーアクティビティのアーティファクト { #user-activity-artifacts }

- [Microsoft オープン仕様: シェルリンク (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [Microsoft オープン仕様: 複合ファイル バイナリ (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [エリック・ジマーマンのLECmd](https://github.com/EricZimmerman/LECmd)
- [エリック・ジマーマンの JLECmd](https://github.com/EricZimmerman/JLECmd)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [エリック・ジマーマンのスラムECmd](https://github.com/EricZimmerman/Srum)
- [Microsoft: 拡張ストレージ エンジン](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## レジストリと永続性 { #registry-and-persistence }

- [Microsoft: レジストリ値のタイプ](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [Maxim Suhanov の Windows レジストリ形式仕様](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) および [libregf](https://github.com/libyal/libregf) — 低レベルのハイブ解析およびリカバリ参照。
- [Eric Zimmerman のレジストリ エクスプローラー / RECmd ドキュメント](https://ericzimmerman.github.io/)
- [Microsoft: サービス レジストリ ツリー](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: タスク スケジューラ スキーマ](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK: スケジュールされたタスク/ジョブ](https://attack.mitre.org/techniques/T1053/)
- [Microsoft オープン仕様: SAM リモート プロトコル](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender とマルウェアの処理 { #defender-and-malware-handling }

- [Microsoft: Defender](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus) の隔離されたファイルを復元します
- [dissect.target Defender プラグイン](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump) — 個別の調査/抽出実装。
- [FIRST マルウェア分析の基礎](https://www.first.org/global/sigs/malware/analysis/) — 処理と分析のリソース。

## Juntei パーサー プロジェクト { #juntei-parser-projects }

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`$MFT`、`$I30`、USN ジャーナル、`$LogFile`、`$Boot`、`$Secure:$SDS`)
- [ys-プリフェッチ-パーサー](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-ジャンプリスト-パーサー](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-レジストリ-パーサー](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "外部材質の変更"
    影響を受ける Windows バージョンと収集したデータに対する申し立てを確認します。有用な記事では、普遍的な規則ではなく、特定のオペレーティング システムの世代、パーサーのバージョン、または実験について説明できます。
