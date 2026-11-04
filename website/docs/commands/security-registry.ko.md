# 레지스트리 명령어 { #registry-commands }

```text
Version: 1.0.0 (CODE BLUE Release)
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

정식 명령어는 `app-compat`, `feature-usage`, `open-save-mru`, `recent-docs`,
`user-assist`, `shell-bags`입니다. 이전 철자(`appcompat`, `featureusage`,
`opensavemru`, `recentdocs`, `userassist`, `shellbags`)도 기존 스크립트를 위해 숨겨진 별칭으로
계속 사용할 수 있습니다. 자동 분석은 정식 이름을 사용하며, 아티팩트 식별자, 결과 스키마 및 출력 파일 이름은
변경되지 않습니다.

레지스트리 명령어는 `-f <HIVE>` 또는 `-d <COLLECTION>`과 구조화된 출력 옵션을 받습니다.
`amcache`는 현재 트랜잭션 로그를 재생하지 않고 Amcache 레지스트리 하이브를 읽습니다. `shimcache`는
기본적으로 인접한 SYSTEM 로그를 재생하며, 내보낸 원시 AppCompatCache 값도 받습니다.
그 밖의 레지스트리 명령어는 인접한 로그를 자동으로 재생합니다. 로그를 명시적으로 지정하려면 `--log1` / `--log2`를,
재생 없이 기본 하이브를 검사하려면 `--no-logs`를 사용하십시오.

## `parse-registry amcache` { #parse-registry-amcache }

```text
Version: 1.0.0 (CODE BLUE Release)
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

[Amcache](../artifacts/amcache.md)를 참조하십시오.

## `parse-registry bam` { #parse-registry-bam }

```text
Version: 1.0.0 (CODE BLUE Release)
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

사용자 이름은 호스트와 같은 방식으로 찾은 시스템의 `SAM` 및 `SOFTWARE`(`ProfileList`)에서 가져옵니다.
[BAM / DAM](../artifacts/bam.md)을 참조하십시오.

## `parse-registry installed-programs` { #parse-registry-installed-programs }

```text
Version: 1.0.0 (CODE BLUE Release)
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

`Uninstall` 엔트리, Windows Installer 제품, App Paths, WSL 배포판 및 패키지 앱을 다룹니다.
사용자 하이브의 사용자는 프로필 폴더와 시스템의 `ProfileList`에서 가져옵니다.
[설치된 프로그램](../artifacts/installed-programs.md)을 참조하십시오.

## `parse-registry muicache` { #parse-registry-muicache }

```text
Version: 1.0.0 (CODE BLUE Release)
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

사용자 이름은 수집본 내 하이브의 위치에서, SID는 `UsrClass.dat`의
루트 키 또는 시스템의 `SOFTWARE`(`ProfileList`)에서 가져옵니다. [MuiCache](../artifacts/muicache.md)를 참조하십시오.

## `parse-registry open-save-mru` { #parse-registry-open-save-mru }

```text
Version: 1.0.0 (CODE BLUE Release)
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

각 셸 항목 목록은 ys-lnk-parser로 디코딩되어 하나의 경로로 연결됩니다.
[열기 및 다른 이름으로 저장 대화 상자](../artifacts/opensavemru.md)를 참조하십시오.

## `parse-registry remote-access` { #parse-registry-remote-access }

```text
Version: 1.0.0 (CODE BLUE Release)
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

사용자의 SID는 호스트와 같은 방식으로 찾은 시스템의 `SOFTWARE`(`ProfileList`)에서 가져옵니다. 저장된
암호는 존재 여부만 보고되며 절대 디코딩되지 않습니다.
[원격 액세스 (사용자 측)](../artifacts/remote-access.md)를 참조하십시오.


## `parse-registry system-info` { #parse-registry-system-info }

```text
Version: 1.0.0 (CODE BLUE Release)
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

시스템당 하나의 레코드로, 이름, 도메인 또는 작업 그룹, Windows 버전과 설치 날짜, 표준 시간대,
마지막 종료, 마지막 사용자, 프로필 및 SID를 담습니다. 하나의 디렉터리에 있는 하이브는 한 시스템의 것으로
취급되며, `-f`를 사용하면 지정한 하이브와 그 옆에서 발견된 네 하이브 중 나머지가 사용됩니다.
[시스템 정보](../artifacts/system-info.md)를 참조하십시오.


## `parse-registry mru` { #parse-registry-mru }

```text
Version: 1.0.0 (CODE BLUE Release)
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

[MRU(최근 사용) 목록](../artifacts/mru.md)을 참조하십시오.


## `parse-registry app-history` { #parse-registry-app-history }

```text
Version: 1.0.0 (CODE BLUE Release)
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

Sysinternals 도구, WinRAR, 7-Zip 및 WinZip 기록, OneDrive 계정, Adobe 최근 파일,
TeamViewer 및 CCleaner를 다룹니다. [애플리케이션 기록](../artifacts/app-history.md)을 참조하십시오.


## `parse-registry feature-usage` { #parse-registry-feature-usage }

```text
Version: 1.0.0 (CODE BLUE Release)
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

`FeatureUsage`, `RecentApps`, `Taskband` 및 `ProgramsCache`를 읽습니다.
[FeatureUsage, RecentApps 및 작업 표시줄 고정 항목](../artifacts/featureusage.md)을 참조하십시오.


## `parse-registry security-settings` { #parse-registry-security-settings }

```text
Version: 1.0.0 (CODE BLUE Release)
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

발견된 각 설정은 값, 의미, 그리고 Windows의 기본 동작을 약화시키는지 여부를 담은 하나의 레코드가
됩니다. `--weakened-only`는 약화시키는 설정만 남깁니다.
[보안 설정](../artifacts/security-settings.md)을 참조하십시오.


## `parse-registry office` { #parse-registry-office }

```text
Version: 1.0.0 (CODE BLUE Release)
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

[Microsoft Office](../artifacts/office.md)를 참조하십시오.

### 예제 { #examples }

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

전체 옵션과 필드의 의미는 [레지스트리 하이브](../artifacts/registry.md),
[Windows 서비스](../artifacts/services.md), [RecentDocs](../artifacts/recentdocs.md),
[Microsoft Office](../artifacts/office.md),
[작업 스케줄러 캐시](../artifacts/taskcache.md), [SAM 로컬 계정](../artifacts/sam.md),
[레지스트리 자동 실행](../artifacts/persistence.md), [UserAssist](../artifacts/userassist.md)에 있습니다.


## `parse-registry auditpol` { #parse-registry-auditpol }

```text
Version: 1.0.0 (CODE BLUE Release)
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

`Policy\PolAdtEv`에 있는 감사 정책의 하위 범주(Windows Vista 이상) 또는 범주(XP 및 2003)당 하나의
레코드로, 무엇을 기록하는지, GUID와 메시지 ID, 그리고 주요 이벤트를 담습니다.
[감사 정책](../artifacts/auditpol.md)을 참조하십시오.


## `parse-registry app-compat` { #parse-registry-app-compat }

```text
Version: 1.0.0 (CODE BLUE Release)
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

호환성 레이어, 프로그램 호환성 관리자의 `Store`와 `Persisted`, CIT 및 사용자 지정
shim 데이터베이스를 다룹니다. ShimCache는 `parse-registry shimcache`가 담당합니다.
[애플리케이션 호환성](../artifacts/appcompat.md)을 참조하십시오.


## `parse-registry bcd` { #parse-registry-bcd }

```text
Version: 1.0.0 (CODE BLUE Release)
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

저장소의 개체당 하나의 레코드로, 테스트 서명, 무결성 검사, 복구, 부팅 상태 정책 및 안전 모드를
상속을 통해 해석한 결과를 담습니다.
[부팅 구성 데이터(BCD)](../artifacts/bcd.md)를 참조하십시오.


## `parse-registry lsa-secrets` { #parse-registry-lsa-secrets }

```text
Version: 1.0.0 (CODE BLUE Release)
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

LSA 비밀 정보(`Policy\Secrets\<name>`)당 하나의 레코드로, 그 종류, 이름에 포함된 서비스나 사용자,
값이 설정된 시점 및 데이터 보유 여부를 담으며, 캐시된 로그온(`Cache`)에 대해서도 하나의 레코드가 있습니다.
이름과 시간만 제공하며 아무것도 복호화하지 않습니다. [LSA 비밀 정보](../artifacts/lsa-secrets.md)를 참조하십시오.


## `parse-registry networks` { #parse-registry-networks }

```text
Version: 1.0.0 (CODE BLUE Release)
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

프로필 시간은 호스트와 같은 방식으로 찾은 시스템의 `SYSTEM` 하이브의 표준 시간대로 UTC로 변환되며,
발견된 원래 값도 함께 보존됩니다. [네트워크](../artifacts/networks.md)를 참조하십시오.


## `parse-registry persistence` { #parse-registry-persistence }

```text
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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

[ShimCache](../artifacts/shimcache.md)를 참조하십시오.

## `parse-registry taskcache` { #parse-registry-taskcache }

```text
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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
Version: 1.0.0 (CODE BLUE Release)
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

### 예제 { #examples_1 }

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

전체 옵션과 필드의 의미는 [레지스트리 하이브](../artifacts/registry.md),
[Windows 서비스](../artifacts/services.md), [RecentDocs](../artifacts/recentdocs.md),
[작업 스케줄러 캐시](../artifacts/taskcache.md), [SAM 로컬 계정](../artifacts/sam.md),
[레지스트리 자동 실행](../artifacts/persistence.md), [UserAssist](../artifacts/userassist.md),
[BAM / DAM](../artifacts/bam.md), [설치된 프로그램](../artifacts/installed-programs.md),
[MuiCache](../artifacts/muicache.md),
[ShellBags](../artifacts/shellbags.md),
[USB 장치](../artifacts/usb.md)에 있습니다.

[열기 및 다른 이름으로 저장 대화 상자](../artifacts/opensavemru.md)도 참조하십시오.

[remote-access](../artifacts/remote-access.md)도 참조하십시오.

[system-info](../artifacts/system-info.md)도 참조하십시오.

[mru](../artifacts/mru.md)도 참조하십시오.

[app-history](../artifacts/app-history.md)도 참조하십시오.

[featureusage](../artifacts/featureusage.md)도 참조하십시오.

[security-settings](../artifacts/security-settings.md)도 참조하십시오.

[office](../artifacts/office.md)도 참조하십시오.

[auditpol](../artifacts/auditpol.md)도 참조하십시오.

[appcompat](../artifacts/appcompat.md)도 참조하십시오.

[bcd](../artifacts/bcd.md)도 참조하십시오.

[lsa-secrets](../artifacts/lsa-secrets.md)도 참조하십시오.

[networks](../artifacts/networks.md)도 참조하십시오.
