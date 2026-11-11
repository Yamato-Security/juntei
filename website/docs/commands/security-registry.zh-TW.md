# 登錄命令 { #registry-commands }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry <COMMAND> [OPTIONS]

Commands:
  amcache             Parse Amcache inventories
  app-compat          Parse application compatibility data
  app-history         Parse third-party tools' traces
  auditpol            Parse the audit policy
  bam                 Parse BAM and DAM last runs
  bcd                 Parse boot configuration
  feature-usage       Parse Explorer use counts and pins
  help                Print this message or the help of the given subcommand(s)
  installed-programs  Parse installed programs
  lsa-secrets         Parse LSA secrets' names and times
  mru                 Parse most-recently-used lists
  muicache            Parse MuiCache program names
  networks            Parse network profiles, interfaces, shares, port proxies and routes
  office              Parse Office recent files, trusted documents and macro settings
  open-save-mru       Parse Open and Save As dialog history
  persistence         Parse autostart locations
  read                Read keys and values
  recent-docs         Parse recent documents
  remote-access       Parse outbound RDP, PuTTY, WinSCP and mapped drive connections
  sam                 Parse local accounts
  security-settings   Parse security settings: Defender, UAC, credentials, RDP, firewall, logging
  services            Parse services
  shell-bags          Parse ShellBags (folders opened in Explorer)
  shimcache           Parse ShimCache entries
  system-info         Parse system information
  taskcache           Parse scheduled tasks
  usb                 Parse USB devices and volumes
  user-assist         Parse UserAssist entries

Options:
  -h, --help  Print help
```

標準命令名稱為 `app-compat`、`feature-usage`、`open-save-mru`、`recent-docs`、
`user-assist` 與 `shell-bags`。為了相容既有的指令碼，其舊有寫法（`appcompat`、`featureusage`、
`opensavemru`、`recentdocs`、`userassist` 與 `shellbags`）仍作為隱藏別名而被接受。
自動解析使用標準名稱；其跡證識別碼、
結果結構描述與輸出檔名皆維持不變。

登錄命令接受 `-f <HIVE>` 或 `-d <COLLECTION>` 以及結構化輸出選項。
`amcache` 會在登錄區處於 dirty 狀態時自動重播相鄰的 Amcache 交易記錄，但沒有指定交易記錄或略過重播的選項。`shimcache`
預設會重播相鄰的 SYSTEM 交易記錄，也接受匯出的原始 AppCompatCache 值。
其他登錄命令會自動重播相鄰的交易記錄；請使用 `--log1` / `--log2` 明確指定
交易記錄，或使用 `--no-logs` 在不重播的情況下檢視主要登錄區。

## `parse-registry amcache` { #parse-registry-amcache }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Amcache hives (Amcache.hve)
  -f, --file <FILE>      File path to one Amcache.hve

Records:
      --files    Include every inventoried executable (InventoryApplicationFile)
      --devices  Include every inventoried PnP device (InventoryDevicePnp)
  -a, --all      Include executables and devices (default for -o and for json, jsonl, csv)

Output:
  -o, --output <PATH>       Save the results to a file (text, json, jsonl) or a directory (csv)
  -t, --output-type <TYPE>  Output type: csv (default with -o), json, jsonl, text (default on the terminal)
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-registry amcache -f Amcache.hve --files
juntei parse-registry amcache -d evidence/ -o amcache-csv
juntei parse-registry amcache -f Amcache.hve -t jsonl | jq 'select(.RecordType == "file")'
```

請參閱 [Amcache](../artifacts/amcache.md)。

## `parse-registry bam` { #parse-registry-bam }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM hives
  -f, --file <FILE>      File path to one SYSTEM hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --all-control-sets  Every control set (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

使用者名稱取自系統的 `SAM` 與 `SOFTWARE`（`ProfileList`），其尋找方式與主機相同。
請參閱 [BAM / DAM](../artifacts/bam.md)。

## `parse-registry installed-programs` { #parse-registry-installed-programs }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      File path to one SOFTWARE or NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

`Uninstall` 項目、Windows Installer 產品、App Paths、WSL 發行版與封裝應用程式。
使用者登錄區的使用者取自其設定檔資料夾與系統的 `ProfileList`。請參閱
[已安裝程式](../artifacts/installed-programs.md)。

## `parse-registry muicache` { #parse-registry-muicache }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      File path to one UsrClass.dat (Windows Vista and later) or NTUSER.DAT (Windows XP) hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

使用者名稱取自登錄區在收集資料中的位置，其 SID 則取自 `UsrClass.dat` 的
根機碼或系統的 `SOFTWARE`（`ProfileList`）。請參閱 [MuiCache](../artifacts/muicache.md)。

## `parse-registry open-save-mru` { #parse-registry-open-save-mru }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (8.3 names, network shares): the system's ANSI code page (932 for Japanese) [default: 1252]

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

每個 Shell 項目清單都會以 ys-lnk-parser 解碼並組合成路徑。請參閱
[開啟與另存新檔對話方塊](../artifacts/opensavemru.md)。

## `parse-registry remote-access` { #parse-registry-remote-access }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry remote-access <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

使用者的 SID 取自系統的 `SOFTWARE`（`ProfileList`），其尋找方式與主機相同。儲存的
密碼只會回報為存在，絕不會被解碼。請參閱
[遠端存取（使用者端）](../artifacts/remote-access.md)。


## `parse-registry system-info` { #parse-registry-system-info }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM, SOFTWARE, SAM and SECURITY hives (those in one directory are one system's)
  -f, --file <FILE>      File path to a SYSTEM, SOFTWARE, SAM or SECURITY hive of one system; repeat for each (the others are looked for next to them)
      --no-logs          Read dirty hives as they are, without their transaction logs (default: <hive>.LOG, .LOG1 and .LOG2 next to each are replayed)

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

每個系統一筆記錄：其名稱、網域或工作群組、Windows 版本與安裝日期、時區、
上次關機、上次登入的使用者、設定檔與 SID。同一目錄中的登錄區屬於同一個系統；
使用 `-f` 時，則為所指定的登錄區，加上在其旁邊找到的、這四個登錄區中的其餘幾個。
請參閱[系統資訊](../artifacts/system-info.md)。


## `parse-registry mru` { #parse-registry-mru }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --list <LISTS>  Only these lists (comma separated): runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

請參閱 [MRU 清單](../artifacts/mru.md)。


## `parse-registry app-history` { #parse-registry-app-history }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

Sysinternals 工具、WinRAR、7-Zip 與 WinZip 的歷程、OneDrive 帳戶、Adobe 最近使用的檔案、
TeamViewer 與 CCleaner。請參閱[應用程式歷程](../artifacts/app-history.md)。


## `parse-registry feature-usage` { #parse-registry-feature-usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry feature-usage <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

讀取 `FeatureUsage`、`RecentApps`、`Taskband` 與 `ProgramsCache`。請參閱
[FeatureUsage、RecentApps 與工作列釘選項目](../artifacts/featureusage.md)。


## `parse-registry security-settings` { #parse-registry-security-settings }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      File path to one SOFTWARE, SYSTEM or NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive (ControlSet001, ControlSet002, ...), not only the one Windows used last
      --weakened-only     Only settings whose value weakens what Windows does by default

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

找到的每個設定都是一筆記錄，包含其值、意義，以及它是否削弱了 Windows
的預設行為；`--weakened-only` 只保留會削弱預設行為的設定。請參閱
[安全性設定](../artifacts/security-settings.md)。


## `parse-registry office` { #parse-registry-office }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry office <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

請參閱 [Microsoft Office](../artifacts/office.md)。

### 範例 { #examples }

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --service ExampleSvc -t jsonl
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry office -d collection/ -o office.csv
juntei parse-registry taskcache -f SOFTWARE --task ScheduledDefrag -t jsonl
juntei parse-registry sam -f SAM --rid 500 -t jsonl
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry user-assist -d collection/ -o userassist.csv
```

完整的選項與欄位語意請參閱[登錄區](../artifacts/registry.md)、
[Windows 服務](../artifacts/services.md)、[RecentDocs](../artifacts/recentdocs.md)、
[Microsoft Office](../artifacts/office.md)、
[工作排程器快取](../artifacts/taskcache.md)、[SAM 本機帳戶](../artifacts/sam.md)、
[登錄自動啟動項目](../artifacts/persistence.md)與 [UserAssist](../artifacts/userassist.md)。


## `parse-registry auditpol` { #parse-registry-auditpol }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SECURITY hives
  -f, --file <FILE>      File path to one SECURITY hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

稽核原則 `Policy\PolAdtEv` 中每個子類別（Windows Vista 及更新版本）或類別（XP 與 2003）
各一筆記錄：其記錄的內容、GUID 與訊息 ID，以及其主要事件。請參閱
[稽核原則](../artifacts/auditpol.md)。


## `parse-registry app-compat` { #parse-registry-app-compat }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry app-compat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      File path to one SOFTWARE or NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

相容性層、程式相容性助理的 `Store` 與 `Persisted`、CIT 以及自訂
shim 資料庫。ShimCache 則由 `parse-registry shimcache` 處理。請參閱
[應用程式相容性](../artifacts/appcompat.md)。


## `parse-registry bcd` { #parse-registry-bcd }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for BCD hives (and BCD-Template)
  -f, --file <FILE>      File path to one BCD hive (\Boot\BCD, \EFI\Microsoft\Boot\BCD)
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

存放區中每個物件一筆記錄，並透過繼承解析出測試簽署、完整性檢查、復原、開機狀態
原則與安全模式。請參閱
[開機設定資料 (BCD)](../artifacts/bcd.md)。


## `parse-registry lsa-secrets` { #parse-registry-lsa-secrets }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SECURITY hives
  -f, --file <FILE>      File path to one SECURITY hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

每個 LSA 機密資訊（`Policy\Secrets\<name>`）一筆記錄，包含其類型、其所指的服務或
使用者、其值的設定時間以及是否包含資料；快取登入（`Cache`）另有一筆
記錄。只有名稱與時間：不會解密任何內容。請參閱 [LSA 機密資訊](../artifacts/lsa-secrets.md)。


## `parse-registry networks` { #parse-registry-networks }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry networks <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE and SYSTEM hives
  -f, --file <FILE>      File path to one SOFTWARE or SYSTEM hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

設定檔時間會以系統 `SYSTEM` 登錄區（其尋找方式與主機相同）的時區轉換為 UTC，
同時也保留原始值。請參閱[網路](../artifacts/networks.md)。


## `parse-registry persistence` { #parse-registry-persistence }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      File path to one SOFTWARE, SYSTEM, NTUSER.DAT or UsrClass.dat hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry read` { #parse-registry-read }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for registry hives (every file that is one, logs aside)
  -f, --file <FILE>      File path to one registry hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
  -k, --key <PATH>             Only the key at this path (in any case; ex: Microsoft\Windows\CurrentVersion\Run). It may start with HKLM\ and the hive's name, and CurrentControlSet is the control set Windows used last
      --key-regex <REGEX>      Only keys whose path matches REGEX (ignoring case)
      --value <NAME>           Only the value named NAME (in any case; "" is the key's default value). With --key or --key-regex, a key without it gives an absent_value record
      --value-regex <REGEX>    Only values whose name matches REGEX (ignoring case; the default value's name is empty)
      --data-regex <REGEX>     Only values whose data, as written, matches REGEX (ignoring case, so a GUID matches in any case; binary data in hex)
      --timeline-start <DATE>  Only keys last written on or after DATE, in UTC unless an offset is given (ex: 2025-10-19, 2025-10-19T13:45:30Z)
      --timeline-end <DATE>    Only keys last written on or before DATE
      --recover-deleted        Also deleted keys and values: in free cells, the slack of cells in use and the data past the hive bins, and what only the transaction logs kept (allocation_status says how complete; recovered_from where found)

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
      --raw-data            Write every value's bytes in hex (value_data_raw), not only those that don't decode as their type
      --qword-as-filetime   Also write REG_QWORD values as FILETIMEs (value_filetime_utc), keeping the number (value_data, value_u64)
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-registry read -f SYSTEM --key 'CurrentControlSet\Services' -o services-raw.csv
juntei parse-registry read -f NTUSER.DAT --recover-deleted -t jsonl | jq .
```

## `parse-registry recent-docs` { #parse-registry-recent-docs }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry recent-docs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry sam` { #parse-registry-sam }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SAM hives
  -f, --file <FILE>      File path to one SAM hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --rid <RID>  Only the account with this RID (ex: 500, the built-in Administrator)

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry services` { #parse-registry-services }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM hives
  -f, --file <FILE>      File path to one SYSTEM hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --service <NAME>    Only the service of this name (its key's name, in any case)
      --all-control-sets  Every control set (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry shell-bags` { #parse-registry-shell-bags }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      File path to one UsrClass.dat or NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Parsing:
      --code-page <NUMBER>  Windows code page for the ANSI names of shell items (8.3 names, network shares) [default: 1252]

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry shimcache` { #parse-registry-shimcache }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM hives
  -f, --file <FILE>      File path to one SYSTEM hive, or an AppCompatCache value saved to a file
      --no-logs          Read a dirty hive as it is, without its transaction logs (SYSTEM.LOG1, SYSTEM.LOG2)

Output:
  -o, --output <FILE>       Save every entry to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. Without -o, the entries are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

```bash
juntei parse-registry shimcache -f SYSTEM
juntei parse-registry shimcache -d evidence/ -o shimcache.csv
```

請參閱 [ShimCache](../artifacts/shimcache.md)。

## `parse-registry taskcache` { #parse-registry-taskcache }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SOFTWARE hives
  -f, --file <FILE>      File path to one SOFTWARE hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Filtering:
      --task <NAME>  Only the task of this name, in any case: its whole name (\Microsoft\Windows\Defrag\ScheduledDefrag) or its last part (ScheduledDefrag)

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry usb` { #parse-registry-usb }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for SYSTEM hives (each system's SOFTWARE and users' NTUSER.DAT hives are read with it)
  -f, --file <FILE>      File path to one SYSTEM hive (its system's SOFTWARE and users' NTUSER.DAT hives are read with it)
      --log1 <FILE>      The SYSTEM hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The SYSTEM hive's second transaction log (.LOG2), for -f
      --no-logs          Read dirty hives as they are, without their transaction logs

Filtering:
      --all-control-sets  Every control set (ControlSet001, ControlSet002, ...), not only the one Windows used last

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## `parse-registry user-assist` { #parse-registry-user-assist }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei parse-registry user-assist <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for NTUSER.DAT hives
  -f, --file <FILE>      File path to one NTUSER.DAT hive
      --log1 <FILE>      The hive's first transaction log (.LOG1), for -f. With --log1 or --log2, only the logs given are replayed (default: <hive>.LOG, .LOG1 and .LOG2 next to it)
      --log2 <FILE>      The hive's second transaction log (.LOG2), for -f
      --no-logs          Read a dirty hive as it is, without its transaction logs

Output:
  -o, --output <FILE>       Save every record to a file
  -t, --output-type <TYPE>  Output type: csv (default), json, jsonl. With -t and without -o, the records are written to stdout
  -C, --clobber             Overwrite files when saving

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

### 範例 { #examples_1 }

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --service ExampleSvc -t jsonl
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry taskcache -f SOFTWARE --task ScheduledDefrag -t jsonl
juntei parse-registry sam -f SAM --rid 500 -t jsonl
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry usb -d collection/ -o usb.csv
juntei parse-registry user-assist -d collection/ -o userassist.csv
juntei parse-registry open-save-mru -d collection/ -o opensavemru.csv
juntei parse-registry bam -d collection/ --all-control-sets -o bam.csv
juntei parse-registry installed-programs -d collection/ -o installed.csv
juntei parse-registry muicache -d collection/ -o muicache.csv
juntei parse-registry shell-bags -d collection/ -o shellbags.csv
```

完整的選項與欄位語意請參閱[登錄區](../artifacts/registry.md)、
[Windows 服務](../artifacts/services.md)、[RecentDocs](../artifacts/recentdocs.md)、
[工作排程器快取](../artifacts/taskcache.md)、[SAM 本機帳戶](../artifacts/sam.md)、
[登錄自動啟動項目](../artifacts/persistence.md)、[UserAssist](../artifacts/userassist.md)、
[BAM / DAM](../artifacts/bam.md)、[已安裝程式](../artifacts/installed-programs.md)、
[MuiCache](../artifacts/muicache.md)、
[ShellBags](../artifacts/shellbags.md)與
[USB 裝置](../artifacts/usb.md)。

另請參閱[開啟與另存新檔對話方塊](../artifacts/opensavemru.md)。

另請參閱 [remote-access](../artifacts/remote-access.md)。

另請參閱 [system-info](../artifacts/system-info.md)。

另請參閱 [mru](../artifacts/mru.md)。

另請參閱 [app-history](../artifacts/app-history.md)。

另請參閱 [featureusage](../artifacts/featureusage.md)。

另請參閱 [security-settings](../artifacts/security-settings.md)。

另請參閱 [office](../artifacts/office.md)。

另請參閱 [auditpol](../artifacts/auditpol.md)。

另請參閱 [appcompat](../artifacts/appcompat.md)。

另請參閱 [bcd](../artifacts/bcd.md)。

另請參閱 [lsa-secrets](../artifacts/lsa-secrets.md)。

另請參閱 [networks](../artifacts/networks.md)。
