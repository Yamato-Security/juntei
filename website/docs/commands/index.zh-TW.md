# 命令清單 { #command-list }

個別的剖析器依其來源類型分組。執行 `juntei --help`、
`juntei <SOURCE> --help` 或 `juntei <SOURCE> <ARTIFACT> --help` 來瀏覽它們：

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

對於包含多台電腦的 Windows 證據收集資料，請使用
[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md)。它會呼叫相同的分組命令，
並依電腦與調查類別整理結果與記錄。

## [磁碟映像檔](mount.md) { #disk-images }

| 命令 | 輸入 | 詳細指南 |
|---|---|---|
| `mount` | VHDX 磁碟映像檔（KAPE `--vhdx`、Hyper-V） | [掛載磁碟映像](mount.md) |

## [ESE 命令](user-activity.md) { #ese-commands }

| 命令 | 輸入跡證 | 詳細指南 |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ESE `Current.mdb`、`{GUID}.mdb`、`SystemIdentity.mdb` | [使用者存取記錄 (UAL)](../artifacts/ual.md) |

## [檔案命令](execution.md) { #file-commands }

| 命令 | 輸入跡證 | 詳細指南 |
|---|---|---|
| `parse-file defender-quarantine` | Defender 隔離資料樹狀結構或項目 | [Defender 隔離資料](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`、`*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [LNK 捷徑](../artifacts/lnk.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |

## [NTFS 命令](filesystem.md) { #ntfs-commands }

| 命令 | 輸入跡證 | 詳細指南 |
|---|---|---|
| `parse-ntfs boot` | `$Boot`，或 NTFS 磁碟區映像檔 | [開機磁區 ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30`（`$INDEX_ALLOCATION`），或磁碟映像檔 | [目錄索引 ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | NTFS `$LogFile` | [NTFS 交易記錄](../artifacts/logfile.md) |
| `parse-ntfs mft` | NTFS `$MFT` | [主檔案表](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS`（`$Secure_$SDS`） | [安全性描述元 ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [USN 日誌](../artifacts/usn-journal.md) |

## [登錄命令](security-registry.md) { #registry-commands }

| 命令 | 輸入跡證 | 詳細指南 |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`、`SYSTEM` | [網路](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [LSA 機密資訊](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD`（以及 `BCD-Template`） | [開機設定資料 (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`、`NTUSER.DAT` | [應用程式相容性](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [稽核原則](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft Office](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage、RecentApps 與工作列釘選項目](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [應用程式歷程](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [最近使用（MRU）清單](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM`（搭配 `SAM`、`SOFTWARE` 以取得使用者名稱） | [BAM / DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat`（Windows XP 上為 `NTUSER.DAT`） | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`、`NTUSER.DAT` | [已安裝程式](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [開啟與另存新檔對話方塊](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`、`SYSTEM`、`NTUSER.DAT`、`UsrClass.dat` | [登錄自動啟動項目](../artifacts/persistence.md) |
| `parse-registry read` | 任何主要登錄區 | [登錄區](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT`（搭配 `SOFTWARE` 以取得使用者 SID） | [遠端存取（使用者端）](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [SAM 本機帳戶](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`、`SYSTEM`、`NTUSER.DAT` | [安全性設定](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [Windows 服務](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` 或匯出的 AppCompatCache 值 | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | 同一系統的 `SYSTEM`、`SOFTWARE`、`SAM`、`SECURITY` | [系統資訊](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [工作排程器快取](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM`，搭配 `SOFTWARE` 與 `NTUSER.DAT` | [USB 裝置](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`、`NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## 來源類型與格式變更 { #source-types-and-format-changes }

分組描述的是 Juntei 目前支援的格式。`parse-registry amcache` 讀取的是 `Amcache.hve`；這並不表示
它支援 SQLite 格式的 Amcache 清查資料庫。各跡證剖析器 crate 保持獨立，因此可以個別新增對其他
儲存格式的支援。

先前扁平的剖析器名稱已被上方的「來源與跡證」路徑取代。既有的剖析器選項請放在跡證名稱之後。
`auto-parse` 與既有的登錄子命令則維持其名稱。

## [Persistence Hunter](persistence-hunter.md) { #persistence-hunter }

`juntei persistence-hunter` 會橫跨多種來源格式進行獵捕，因此它是頂層的分析命令。它以 131 項檢查、
允許／封鎖分類、承載資料中繼資料、涵蓋範圍與時間戳記關聯，補足 `parse-registry persistence`
所提供的完整設定清查。

## 頂層用法 { #top-level-usage }

```text
Usage: juntei <COMMAND>

Commands:
  auto-parse          Discover and parse Windows evidence
  help                Print this message or the help of the given subcommand(s)
  mount               Mount a VHDX disk image read-only (FUSE)
  parse-ese           Parse ESE databases
  parse-file          Parse file artifacts
  parse-ntfs          Parse NTFS artifacts
  parse-registry      Parse registry hives
  persistence-hunter  Hunt offline Windows persistence

Options:
  -h, --help     Print help
  -V, --version  Print version
```

各來源頁面會重現命令用法並提供簡短的範例。跡證指南則說明證據、輸出欄位、限制與調查策略。
