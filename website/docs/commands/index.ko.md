# 명령어 목록 { #command-list }

개별 파서는 소스 유형별로 그룹화되어 있습니다. `juntei --help`,
`juntei <SOURCE> --help` 또는 `juntei <SOURCE> <ARTIFACT> --help`를 실행하여 살펴보십시오.

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

여러 컴퓨터의 Windows 증거 수집 데이터에는
[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md)를 사용하십시오. 이 명령은 같은 그룹화된
명령어를 호출하고, 결과와 로그를 컴퓨터 및 조사 범주별로 정리합니다.

## [디스크 이미지](mount.md) { #disk-images }

| 명령어 | 입력 | 상세 가이드 |
|---|---|---|
| `mount` | VHDX 디스크 이미지 | [디스크 이미지 마운트](mount.md) |

## [ESE 명령어](user-activity.md) { #ese-commands }

| 명령어 | 입력 아티팩트 | 상세 가이드 |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ESE `Current.mdb`, `{GUID}.mdb`, `SystemIdentity.mdb` | [사용자 액세스 로깅 (UAL)](../artifacts/ual.md) |

## [파일 명령어](execution.md) { #file-commands }

| 명령어 | 입력 아티팩트 | 상세 가이드 |
|---|---|---|
| `parse-file defender-quarantine` | Defender 격리 트리 또는 항목 | [Defender 격리 데이터](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [LNK 바로 가기](../artifacts/lnk.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |

## [NTFS 명령어](filesystem.md) { #ntfs-commands }

| 명령어 | 입력 아티팩트 | 상세 가이드 |
|---|---|---|
| `parse-ntfs boot` | `$Boot` 또는 NTFS 볼륨 이미지 | [부트 섹터 ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30`(`$INDEX_ALLOCATION`) 또는 디스크 이미지 | [디렉터리 인덱스 ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | NTFS `$LogFile` | [NTFS 트랜잭션 로그](../artifacts/logfile.md) |
| `parse-ntfs mft` | NTFS `$MFT` | [마스터 파일 테이블](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS`(`$Secure_$SDS`) | [보안 설명자 ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [USN 저널](../artifacts/usn-journal.md) |

## [레지스트리 명령어](security-registry.md) { #registry-commands }

| 명령어 | 입력 아티팩트 | 상세 가이드 |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`, `SYSTEM` | [네트워크](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [LSA 비밀 정보](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD`(및 `BCD-Template`) | [부팅 구성 데이터 (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`, `NTUSER.DAT` | [애플리케이션 호환성](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [감사 정책](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft Office](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage, RecentApps 및 작업 표시줄 고정 항목](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [애플리케이션 기록](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [MRU(최근 사용) 목록](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM`(사용자 이름 확인용 `SAM`, `SOFTWARE` 포함) | [BAM / DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat`(Windows XP에서는 `NTUSER.DAT`) | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`, `NTUSER.DAT` | [설치된 프로그램](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [열기 및 다른 이름으로 저장 대화 상자](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat` | [레지스트리 자동 실행](../artifacts/persistence.md) |
| `parse-registry read` | 모든 기본 레지스트리 하이브 | [레지스트리 하이브](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT`(사용자 SID 확인용 `SOFTWARE` 포함) | [원격 액세스 (사용자 측)](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [SAM 로컬 계정](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` | [보안 설정](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [Windows 서비스](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` 또는 내보낸 AppCompatCache 값 | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | 한 시스템의 `SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` | [시스템 정보](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [작업 스케줄러 캐시](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM`(`SOFTWARE` 및 `NTUSER.DAT`와 함께) | [USB 장치](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`, `NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## 소스 유형과 형식 변경 { #source-types-and-format-changes }

그룹은 Juntei가 현재 지원하는 형식을 나타냅니다. `parse-registry amcache`는 `Amcache.hve`를 읽으며,
SQLite Amcache 인벤토리 데이터베이스를 지원한다는 의미는 아닙니다. 아티팩트 파서 크레이트는 분리되어
있으므로 다른 저장 형식에 대한 지원을 독립적으로 추가할 수 있습니다.

이전의 평면적인 파서 이름은 위의 소스 및 아티팩트 경로로 대체되었습니다. 기존 파서 옵션은 아티팩트 이름
뒤에 그대로 지정하십시오. `auto-parse`와 기존 레지스트리 하위 명령어는 이름이 유지됩니다.

## [Persistence Hunter](persistence-hunter.md) { #persistence-hunter }

`juntei persistence-hunter`는 여러 소스 형식에 걸쳐 헌팅을 수행하므로 최상위 분석 명령어입니다.
`parse-registry persistence`의 전체 구성 인벤토리를 보완하여 131개의 검사, 허용/차단 분류, 페이로드
메타데이터, 적용 범위 및 타임스탬프 상관 분석을 제공합니다.

## 최상위 사용법 { #top-level-usage }

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

소스 페이지에는 명령어 사용법과 간단한 예가 수록되어 있습니다. 아티팩트 가이드는 증거, 출력 필드, 제한
사항 및 조사 전략을 설명합니다.
