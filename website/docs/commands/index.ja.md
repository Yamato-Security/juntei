# コマンド一覧 { #command-list }

個々のパーサーはソース タイプごとにグループ化されています。 `juntei --help`、`juntei <SOURCE> --help`、または `juntei <SOURCE> <ARTIFACT> --help` を実行してそれらを調べます。

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

マルチコンピューター Windows 証拠収集の場合は、[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md) を使用します。同じグループ化されたコマンドを呼び出し、結果とログをコンピュータおよび調査カテゴリごとに整理します。

[`juntei super-timeline -d <PARSED> -o SuperTimeline.csv`](super-timeline.md) は、解析済みアーティファクトの各時刻を意味付きイベントに展開します。デフォルトでは並べ替えず、`--sort` を指定すると時系列に並べ替えて重複を統合します。`auto-parse -j` は引き続きネイティブレコードを保持します。

## [ディスクイメージ](mount.md) { #disk-images }

| コマンド | 入力 | 詳細ガイド |
|---|---|---|
| `mount` | VHDX / E01 / Ex01 ディスクイメージ | [ディスクイメージのマウント](mount.md) |

## [ESE コマンド](user-activity.md) { #ese-commands }

| コマンド | 入力アーティファクト | 詳細ガイド |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ESE `Current.mdb`、`{GUID}.mdb`、`SystemIdentity.mdb` | [ユーザーアクセスログ (UAL)](../artifacts/ual.md) |
| `parse-ese webcache` | ESE `WebCacheV01.dat` | [Internet Explorer と従来の Edge](../artifacts/webcache.md) |

## [ファイルコマンド](execution.md) { #file-commands }

| コマンド | 入力アーティファクト | 詳細ガイド |
|---|---|---|
| `parse-file defender-quarantine` | Defender 隔離ツリーまたはエントリ | [Defender 隔離](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [LNK ショートカット](../artifacts/lnk.md) |
| `parse-file outlook` | `*.pst`, `*.ost`, `*.msg`, `*.oft` | [Outlook](../artifacts/outlook.md) |
| `parse-file powershell-history` | `PSReadLine\*_history.txt` | [PowerShell 履歴](../artifacts/powershell-history.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |
| `parse-file recycle-bin` | `$Recycle.Bin\<SID>\$I*`、`RECYCLER\<SID>\INFO2`、`RECYCLED\INFO2` | [ごみ箱](../artifacts/recycle-bin.md) |

## [NTFS コマンド](filesystem.md) { #ntfs-commands }

| コマンド | 入力アーティファクト | 詳細ガイド |
|---|---|---|
| `parse-ntfs boot` | `$Boot`、または NTFS ボリューム イメージ | [ブート セクター ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30` (`$INDEX_ALLOCATION`)、またはディスク イメージ | [ディレクトリインデックス ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | NTFS `$LogFile` | [NTFS トランザクション ログ](../artifacts/logfile.md) |
| `parse-ntfs mft` | NTFS `$MFT` | [マスター ファイル テーブル](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS` (`$Secure_$SDS`) | [セキュリティ記述子 ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [USN ジャーナル](../artifacts/usn-journal.md) |

## [レジストリコマンド](security-registry.md) { #registry-commands }

| コマンド | 入力アーティファクト | 詳細ガイド |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`, `SYSTEM` | [ネットワーク](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [LSA 秘密](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD` (および `BCD-Template`) | [ブート構成データ (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`, `NTUSER.DAT` | [アプリケーションの互換性](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [監査ポリシー](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft オフィス](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage、RecentApps およびタスクバー ピン](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [アプリケーション履歴](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [最近使用したリスト](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM` (ユーザー名には `SAM`、`SOFTWARE` を使用) | [BAM および DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat` (Windows XP 上の `NTUSER.DAT`) | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`, `NTUSER.DAT` | [インストールされたプログラム](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [ダイアログを開いて名前を付けて保存](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat` | [レジストリの自動起動](../artifacts/persistence.md) |
| `parse-registry read` | 任意のプライマリ レジストリ ハイブ | [レジストリ ハイブ](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT` (ユーザー SID には `SOFTWARE`) | [リモート アクセス (ユーザー側)](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [SAM ローカル アカウント](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` | [セキュリティ設定](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [Windows サービス](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` またはエクスポートされた AppCompatCache 値 | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | 1 つのシステムの `SYSTEM`、`SOFTWARE`、`SAM`、`SECURITY` | [システム情報](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [Tタスク スケジューラ キャッシュ](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM`、`SOFTWARE` および `NTUSER.DAT` を含む | [USB デバイス](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`, `NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## [SQLite コマンド](sqlite.md) { #sqlite-commands }

| コマンド | 入力アーティファクト | 詳細ガイド |
|---|---|---|
| `parse-sqlite browser` | `History`, `Cookies`, `Web Data`, `places.sqlite`, `cookies.sqlite`, `formhistory.sqlite` | [Web ブラウザ](../artifacts/browser.md) |

## ソースの種類とフォーマットの変更 { #source-types-and-format-changes }

グループは、Juntei が現在サポートしている形式を説明します。 `parse-registry amcache` は `Amcache.hve` を読み取ります。 SQLite Amcache インベントリ データベースのサポートを意味するものではありません。アーティファクト パーサー クレートは分離されたままであるため、別のストレージ形式のサポートを個別に追加できます。

以前のフラット パーサー名は、上記のソースとアーティファクトのパスに置き換えられました。アーティファクト名の後に既存のパーサー オプションを保持します。 `auto-parse` および既存のレジストリ サブコマンドの名前は維持されます。

## [Persistence Hunter](persistence-hunter.md) { #persistence-hunter }

`juntei persistence-hunter` は複数のソース形式をハントするため、トップレベルの分析コマンドです。これは、`parse-registry persistence` の完全な構成インベントリを 131 チェック、許可/ブロック分類、ペイロード メタデータ、カバレッジ、およびタイムスタンプ相関で補完します。

## トップレベルの使用法 { #top-level-usage }

```text
Usage: juntei <COMMAND>

Commands:
  auto-parse          Discover and parse Windows evidence
  help                Print this message or the help of the given subcommand(s)
  mount               Mount a VHDX or E01 disk image read-only (FUSE)
  parse-ese           Parse ESE databases
  parse-file          Parse file artifacts
  parse-ntfs          Parse NTFS artifacts
  parse-registry      Parse registry hives
  parse-sqlite        Parse SQLite databases
  persistence-hunter  Hunt offline Windows persistence

Options:
  -h, --help     Print help
  -V, --version  Print version
```

ソース ページではコマンドの使用法を再現し、短い例を示しています。アーティファクト ガイドでは、証拠、出力フィールド、制限、および調査戦略について説明します。
