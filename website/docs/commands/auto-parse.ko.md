# 수집 데이터 자동 분석 { #automatic-collection-parsing }

`auto-parse`는 여러 컴퓨터의 Windows 트리아지 수집물을 구조화된 분석 디렉터리로 변환하는 가장 빠른
방법입니다. 증거 루트와 새 출력 디렉터리를 지정하십시오. 수집된 시스템을 찾아 컴퓨터 이름을 식별하고, 호환되는
모든 Juntei 파서를 병렬로 실행합니다.

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

기본 출력은 CSV입니다. CSV 결과를 유지하면서 `jq`나 SIEM 가져오기를 위한 JSONL 타임라인을 추가하려면 다음과
같이 합니다.

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "출력 트리와 증거 트리를 분리하십시오"
    Juntei는 `<EVIDENCE>` 안에 있거나 `<EVIDENCE>`를 포함하는 출력 디렉터리를 거부합니다. 이를 통해
    생성된 데이터가 증거로 다시 발견되는 것을 방지하고, `--clobber`가 원본 수집물의 파일을 건드리지 않도록
    합니다.

## VHDX 디스크 이미지 { #vhdx-disk-images }

`--vhdx`를 지정하면 `auto-parse`는 증거 디렉터리의 모든 VHDX 디스크 이미지 안에 있는 시스템을 파싱합니다. 각 이미지는 파싱 중 FUSE로 읽기 전용 마운트되며 파일을 추출하지 않습니다. [디스크 이미지 마운트](mount.md)를 참조하십시오.

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

NTFS 볼륨을 열 수 없으면 Juntei는 문제를 보고하고 읽을 수 있는 볼륨은 유지합니다. 이미지의 일부만 읽을 수 있으면 `auto-parse --vhdx`는 종료 코드 1을 반환하며, `mount`도 마운트 해제 후 1을 반환합니다.

## 출력 표준 시간대 { #output-time-zone }

파싱된 타임스탬프는 분석용 컴퓨터의 로컬 표준 시간대와 관계없이 기본적으로 UTC입니다. UTC는 Windows 이벤트
로그 형식인 `2026-10-20T08:46:38.2001668Z`를 사용합니다. 즉, ISO 8601 날짜/시간이며, 날짜와 시간 사이에
`T`, UTC를 나타내는 `Z`가 붙습니다. 사용 가능한 소수 자릿수는 보존됩니다. `manifest.json`의 `GeneratedUTC`도
이 형식을 사용합니다. 콘솔의 `Start time`과 `End time`은 출력 표준 시간대와 관계없이 분석용 컴퓨터의 로컬
시간(예: `2026-11-04 09:15:02`)입니다.

`-Z, --timezone`으로 고정 표시 오프셋을 선택합니다.

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

오프셋은 부호가 있는 두 자리 `HH:MM` 값입니다(시 `00–23`, 분 `00–59`). `JST` 같은 이름, `+9` 같은 축약된 값,
잘못된 범위는 예시 구문과 함께 오류를 발생시킵니다. UTC를 사용하려면 옵션을 생략하거나
`--timezone "+00:00"`을 사용하십시오. 고정 오프셋에는 일광 절약 시간 규칙이 적용되지 않습니다.

이 옵션은 아티팩트별 CSV, JSON, JSONL 결과와 추가 `-j` JSONL 내보내기의 파싱된 타임스탬프에 적용됩니다.
예를 들어 `2026-10-20T08:46:38.2001668Z`는 `2026-10-20T17:46:38.2001668+09:00`이 되며, 둘 다 같은 순간을
나타냅니다.

필드 이름은 `UTC`로 끝나는 기존 이름을 포함하여 그대로 유지됩니다. 변환된 값은 선택한 오프셋을 명시적으로
포함합니다. 두 매니페스트의 `OutputTimeZone`은 요청된 표시 오프셋을 기록하며, `GeneratedUTC`는 UTC 실행
메타데이터로 유지됩니다. 파서별 로그는 파서 고유의 UTC 요약을 유지합니다.

원시 FILETIME/에포크 숫자, 레지스트리 값 딕셔너리, 원본의 표준 시간대 구성, 텍스트 메타데이터, 날짜만 있는
값, 알려진 오프셋이 없는 로컬 시계 값은 보존됩니다. Juntei는 알 수 없는 로컬 시간에 `Z`를 추가하지 않습니다.
FAT에서 유래한 셸 항목 시계(ShellBags/OpenSaveMRU의 `Item*UTC` 및 LNK/Jump List의 `TargetItems`)에는 기록된
오프셋이 없습니다. 기존의 UTC 레이블이나 표시된 `Z` 접미사는 UTC임을 증명하지 않으므로, 오프셋 옵션은 이를
보존합니다. 자유 텍스트 요약을 담은 CSV 열은 기록된 텍스트를 유지하며, JSON/JSONL은 중첩된 파싱 타임스탬프를
구조화된 필드로 보존합니다.

UTC는 기존 파서 출력을 그대로 사용합니다. 명시적인 0이 아닌 오프셋은 각 파서 작업자 내에 스트리밍 서식 지정
단계를 추가하며, 증거를 변경하지 않고 표시 값만 변경합니다.

오프셋으로 인해 타임스탬프가 `0000–9999`년 범위를 벗어나게 되면, 해당 값은 원래의 명시적 오프셋을 유지합니다.
소수 자릿수가 9자리를 넘는 값도 반올림되지 않고 그대로 유지됩니다. Juntei는 파서의 `*-logs.txt` 파일에 경고를
기록하고 나머지 타임스탬프를 변환합니다. 잘못된 형식의 CSV/JSON이나 출력 오류가 발생하면 여전히 작업이
실패하며, 영향을 받은 원래 결과 파일은 변경되지 않습니다.

## 발견 대상 { #what-it-discovers }

Juntei는 대소문자를 구분하지 않고 `System32`, `Prefetch` 또는 `AppCompat` 디렉터리를 포함하는 `Windows`라는
이름의 디렉터리를 재귀적으로 검색합니다. 뒤의 두 가지도 인식하므로 Prefetch나 Amcache만 포함된 대상 지정
수집물도 지원합니다. 이로써 `AppData\Roaming\Microsoft\Windows` 같은 사용자 프로필 디렉터리는 제외됩니다.
`C:\Windows.old\Windows` 같은 중첩된 이전 설치는 중복 컴퓨터가 되지 않고 바깥쪽의 수집된 볼륨에 할당됩니다.
기본 최대 깊이는 10입니다. 이는 다음과 같은 일반적인 Velociraptor 경로를 포함합니다.
```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

수집물이 더 깊이 중첩된 경우에만 제한을 변경하십시오.

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

발견된 각 시스템의 컴퓨터 이름은 다음 순서로 선택됩니다.

1. 가장 가까운 상위의 `client_info.json`에 있는 `hostname`(그 값은 매니페스트에 `FQDN`으로 보존됨).
2. 수집된 `SYSTEM` 레지스트리 하이브의 `ComputerName`.
3. 가장 가까운 유용한 수집 디렉터리 이름(드라이브와 Velociraptor `uploads/auto` 구조는 건너뛰고, `_F.…` 같은
   접미사는 제거).

이름은 디렉터리 이름으로 사용하기에 안전하도록 변환됩니다. 중복된 이름에는 출력 디렉터리와 `Comp` 값 모두에
숫자 접미사가 붙으므로, JSON에서 서로 다른 시스템을 구별할 수 있습니다. 매니페스트는 발견된 그대로의 호스트
이름을 별도의 `Hostname` 필드에 보관합니다.

시스템을 나열한 후, `auto-parse`는 `Parser tasks` 바로 위에 `Timezone:` 줄 하나를 출력합니다. 이 줄은 선택된
출력 표준 시간대를 보여 줍니다. 기본값은 `UTC`이며, `--timezone "+09:00"`을 사용하면 `UTC+09:00` 같은 값이
됩니다. 레이블은 녹색이고 값은 다른 요약 값과 마찬가지로 터미널의 기본 텍스트 색상을 사용합니다.
`-K, --no-color`를 사용하면 이 줄은 색 없이 표시됩니다. 수집된 시스템의 표준 시간대 구성은
[시스템 정보](../artifacts/system-info.md) 결과에서 확인할 수 있습니다.

## 실행되는 파서 { #parsers-it-runs }

하나의 파서가 실패해도 다른 파서는 중단되지 않습니다. `auto-parse`는 발견된 모든 컴퓨터에 대해 다음 작업을
실행합니다. 작업은 `parse-ntfs mft`, `parse-registry amcache`, `parse-ese srum`, `parse-file prefetch` 등
수동 파싱과 동일한 소스별 그룹 명령어를 사용합니다.

- Amcache
- `$MFT`
- `$LogFile` NTFS 재실행/실행 취소(redo/undo) 레코드
- Prefetch 레코드와 별도의 Prefetch 실행 타임라인
- USN 저널
- `$I30` 디렉터리 인덱스
- `$Boot` 부트 섹터와 `$Secure:$SDS` 보안 설명자
- SRUM
- 사용자 액세스 로깅 (UAL)
- 웹 브라우저 방문 기록, 다운로드, 쿠키 및 양식 기록(Chromium 및 Firefox 기반 브라우저)
- Internet Explorer와 레거시 Edge의 WebCache: 방문 기록, 다운로드, 쿠키 및 캐시 항목
- LNK 바로 가기
- Jump Lists
- ShimCache
- Persistence Hunter(허용/차단 프로필을 갖춘 131개의 교차 소스 지속성 검사)
- Defender 격리 메타데이터
- 열기 및 다른 이름으로 저장 대화 상자 기록
- 복구 가능한 삭제된 데이터를 포함한 모든 레지스트리 키와 값
- 서비스, RecentDocs, TaskCache, SAM 계정, 레지스트리 자동 실행 위치, UserAssist, BAM, 설치된 프로그램,
  MuiCache, ShellBags, USB 장치
- 시스템 정보: 이름, Windows 버전, 표준 시간대, 마지막 종료, 프로필 및 SID
- 감사 정책: 시스템이 기록하도록 설정된 보안 이벤트
- 부팅 구성(BCD 및 BCD-Template)
- 네트워크 프로필, 인터페이스, 공유, 포트 프록시 및 경로
- LSA 비밀 정보의 이름과 시간, 사용 중인 캐시된 도메인 로그온(아무것도 복호화하지 않음)

Persistence Hunter는 부분 허용 일치를 포함하여 모든 심각도(`--min-severity info`)를 유지합니다. `LOW`는 조사할
대상이라는 분류이며, 서명 메타데이터가 없으면 악성 명령에도 이 분류가 나올 수 있습니다. 자동 인벤토리는 수동
명령어의 기본 medium 임계값으로 이러한 항목을 버리지 않습니다.

헌터는 각 자동 파서 프로세스 내에서 하나의 작업자를 사용합니다. 이를 통해 `auto-parse -T <N>`이 모든 하위
프로세스에서 CPU 수만큼의 풀을 추가로 만들어 프로세스 예산을 배가시키는 것을 방지합니다. 단독 실행하는
`persistence-hunter -T <N>`은 헌터 자체의 작업자 스레드를 제어합니다.

CSV `$MFT` 출력에는 모든 타임스탬프와 짧은 이름이 포함됩니다. 레지스트리 출력에는 원시 데이터와 `REG_QWORD`
값의 FILETIME 해석이 포함됩니다. JSON 및 JSONL SRUM 출력에는 값 상태 메타데이터가 포함됩니다. Defender 파일과
상주 `$MFT` 스트림은 자동으로 **추출되지 않습니다**.

`--mft-slack`를 사용하면 `$MFT` 출력에 FILE 레코드의 슬랙에 남은 파일 이름도 [`parse-ntfs mft -x`](../artifacts/mft.md)와 같은 형식으로 포함됩니다. CSV에서는 `FromSlack`로 표시된 행, JSON 및 JSONL에서는 `SlackFileNames`입니다. `--code-page`는 LNK 파일, Jump List, ShellBags 및 OpenSaveMRU에 적용됩니다.

수집되지 않은 아티팩트는 실패가 아니라 정상적인 `skipped` 작업입니다.

## 출력 레이아웃 { #output-layout }

결과는 컴퓨터와 조사 범주별로 정리됩니다. 범주 디렉터리는 `Accounts`, `FileAndFolderOpening`처럼 PascalCase를
사용합니다. Amcache와 Jump Lists 테이블은 파일 이름 접두사와 함께 해당 범주에 직접 배치됩니다. 컴퓨터
디렉터리 이름은 발견된 철자를 유지합니다. 기존 출력 디렉터리는 마이그레이션되지 않으므로, 이전 명명 방식에서
전환할 때는 새 출력 디렉터리를 선택하십시오. 각 파서는 아티팩트가 수집되지 않은 경우를 포함하여 결과 옆에
`<artifact>-logs.txt` 파일을 기록합니다. 아티팩트마다 열이 다르기 때문에 CSV는 별도의 테이블을 유지합니다.

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── BrowserActivity/
│   │   ├── Browser-Cookies.csv
│   │   ├── Browser-Downloads.csv
│   │   ├── Browser-FormHistory.csv
│   │   ├── Browser-History.csv
│   │   ├── Browser-logs.txt
│   │   ├── WebCache-Containers.csv
│   │   ├── WebCache-Cookies.csv
│   │   ├── WebCache-Downloads.csv
│   │   ├── WebCache-Entries.csv
│   │   ├── WebCache-History.csv
│   │   └── WebCache-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   ├── Registry-RemoteAccess-logs.txt
│   │   ├── UAL-Clients.csv
│   │   ├── UAL-RoleAccess.csv
│   │   ├── UAL-SystemIdentity.csv
│   │   └── UAL-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```

| 범주 | 아티팩트 |
|---|---|
| `Accounts` | SAM 로컬 계정, LSA 비밀 정보의 이름과 시간 |
| `Applications` | 애플리케이션 기록(서드 파티 도구의 흔적) |
| `Applications` | 설치된 프로그램 |
| `BrowserActivity` | 복구된 삭제 항목을 포함한 웹 브라우저 방문 기록, 다운로드, 쿠키 및 양식 기록, 그리고 Internet Explorer와 레거시 Edge의 WebCache |
| `ExternalDevices` | USB 장치와 볼륨 |
| `FileAndFolderOpening` | Jump Lists, LNK 바로 가기, Microsoft Office, MRU 목록, 열기 및 다른 이름으로 저장 대화 상자, RecentDocs, ShellBags |
| `FileExistence` | Amcache, ShimCache |
| `FileSystem` | `$Boot`, `$I30`, `$LogFile`, `$MFT`, `$Secure:$SDS`, USN 저널 |
| `NetworkActivity` | 네트워크 프로필, 인터페이스, 공유, 포트 프록시 및 경로. SRUM 연결, 트래픽 및 푸시 알림 |
| `LateralMovement` | 사용자의 아웃바운드 원격 데스크톱, PuTTY 및 WinSCP 연결, 매핑된 드라이브 및 공유. 사용자 액세스 로깅(서버 역할의 클라이언트) |
| `Persistence` | 부팅 구성(BCD), 레지스트리 자동 실행, 서비스, TaskCache |
| `ProcessExecution` | 애플리케이션 호환성, BAM 및 DAM, FeatureUsage 및 고정 항목, MuiCache, Prefetch, Prefetch 타임라인, SRUM 애플리케이션/CPU 활동, UserAssist |
| `Registry` | 복구된 레코드를 포함한 일반 하이브 키와 값 |
| `SecurityProducts` | Defender 격리 메타데이터 및 선택적으로 추출된 콘텐츠. 보안 설정(Defender, UAC, 자격 증명 보호, RDP, 방화벽, 로깅) |
| `SystemInformation` | 이름, Windows 버전, 표준 시간대, 마지막 종료, 프로필, SID 및 감사 정책. SRUM 에너지/용량, ID 맵 및 알 수 없는 공급자 |

범주는 조사를 정리하는 데 도움을 주지만, 특정 활동의 증거는 아닙니다. 예를 들어 Amcache와 ShimCache 항목만으로는
실행이 입증되지 않습니다. SRUM은 리소스 사용량을 기록합니다. 프로그램이 실행되었다는 결론을 내리기 전에
아티팩트 가이드를 참조하십시오.

CSV, JSON, JSONL은 같은 범주를 사용합니다. 파서 작업마다 하나씩 있는 `<artifact>-logs.txt` 파일에는 정확한
명령어와 메시지가 들어 있습니다. 예를 들어 `HOST1/FileSystem/MFT-logs.txt`와
`HOST1/ProcessExecution/Prefetch-logs.txt`입니다. JSON과 JSONL도 같은 로그 파일 이름을 사용합니다. Amcache와
Jump Lists는 각각 접두사가 붙은 테이블 옆에 하나의 공유 로그를 가집니다. SRUM 공급자 테이블은
`NetworkActivity`, `ProcessExecution`, `SystemInformation`에 직접 배치되며, 공유 파일
`SystemInformation/SRUM-logs.txt`가 있습니다. `logs/` 하위 디렉터리는 생성되지 않습니다.

기본 CSV 파일 이름은 `Registry-SAM.csv`, `LNK.csv`, `Registry-RecentDocs.csv`, `Registry-ShimCache.csv`, `LogFile.csv`,
`I30.csv`, `MFT.csv`, `UsnJournal.csv`, `Boot.csv`, `SecureSDS.csv`, `Registry-Networks.csv`, `Registry-Persistence.csv`, `Registry-Services.csv`, `Registry-TaskCache.csv`, `Registry-BCD.csv`,
`Registry-SecuritySettings.csv`,
`PrefetchTimeline.csv`, `Prefetch.csv`, `Registry-UserAssist.csv`, `Registry-AppCompat.csv`, `Registry-Office.csv`, `Registry-FeatureUsage.csv`, `Registry-AppHistory.csv`, `Registry-MRU.csv`, `Registry-SystemInfo.csv`, `Registry-RemoteAccess.csv`, `Registry-OpenSaveMRU.csv`, `Registry-InstalledPrograms.csv`, `Registry-BAM.csv`, `Registry-MuiCache.csv`, `Registry-ShellBags.csv`, `Registry-USB.csv`, `Registry-SecuritySettings.csv`, `Registry-AuditPolicy.csv`, `Registry-Dump.csv`,
`DefenderQuarantine.csv`입니다. 레지스트리에서 파생된 테이블은 `Registry-ShimCache.csv`, `Registry-UserAssist.csv`,
`Registry-BAM.csv`, `Registry-SystemInfo.csv`를 포함하여 `Registry-` 접두사를 사용합니다. 일반 하이브 키와 값은
`Registry-Dump.csv`에 있습니다. Amcache 테이블은 `FileExistence/Registry-Amcache-Devices.csv`,
`Registry-Amcache-Files.csv`, `Registry-Amcache-Findings.csv`, `Registry-Amcache-Summary.csv`입니다. Jump Lists는
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv`와 `JumpLists-CustomDestinations.csv`를 사용합니다.
Amcache나 Jump Lists의 하위 디렉터리는 없습니다. SRUM 테이블은 범주 바로 아래에 `SRUM-` 파일 이름을 사용합니다.
예를 들어 `NetworkActivity/SRUM-NetworkConnections.csv`입니다.
사용자 액세스 로깅 테이블은 `LateralMovement`에 테이블마다 하나의 파일로 있으며(예: `UAL-Clients.csv`,
`UAL-SystemIdentity.csv`), `UAL-logs.txt`도 함께 있습니다.
[사용자 액세스 로깅 (UAL)](../artifacts/ual.md#automatic-parsing)을 참조하십시오.
웹 브라우저 테이블은 `BrowserActivity`에 있으며(`Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv`, `Browser-FormHistory.csv`), `Browser-logs.txt`도 함께 있습니다.
[웹 브라우저](../artifacts/browser.md#automatic-parsing)를 참조하십시오.
Internet Explorer와 레거시 Edge의 WebCache 테이블도 이곳에 있으며(`WebCache-History.csv`, `WebCache-Downloads.csv`,
`WebCache-Cookies.csv`, `WebCache-Entries.csv`, `WebCache-Containers.csv`), `WebCache-logs.txt`도 함께 있습니다.
[Internet Explorer와 레거시 Edge](../artifacts/webcache.md#automatic-parsing)를 참조하십시오.
[SRUM 배치 표 전체](../artifacts/srum.md#categorized-auto-parse-output)를 참조하십시오.

로그는 `Registry-SAM-logs.txt`, `PrefetchTimeline-logs.txt`, `Registry-UserAssist-logs.txt`처럼 대문자로 시작하는
아티팩트 이름과 일치합니다. JSON/JSONL 파일은 같은 이름 부분을 사용하며(예: `MFT.jsonl`), 쿼리용 레코드의
`Artifact` 식별자는 `mft`입니다. 여러 테이블을 출력하는 경우 테이블 옆에 `Registry-Amcache-logs.txt` 또는
`JumpLists-logs.txt`가 있습니다. 공유 SRUM 로그는 `SystemInformation`에 있습니다.


`manifest.json`은 발견된 원래 `Hostname`, 고유한 출력/레코드 `Comp` ID, 호스트 이름의 출처, 증거 경로,
아티팩트 `Category`, 명령줄, 소요 시간, 종료 코드, 결과 경로, 로그 경로, 요청된 JSONL 타임라인과 내보내기 오류,
Defender 추출 요청 여부, 그리고 각 작업이 성공했는지, 건너뛰었는지, 실패했는지를 기록합니다.

자동 분석은 절대 경로를 전혀 기록하지 않으므로, 분석 시스템에서 증거, 결과, Juntei가 어디에 있는지 드러내지
않고 결과를 공유할 수 있습니다.

- 증거 경로는 증거 디렉터리 기준의 상대 경로입니다. 터미널은 각 컴퓨터를 증거 내 디렉터리와 함께 나열하며
  (`HOST1: ./HOST1_F`), 파서는 증거 디렉터리에서 실행되고 이러한 경로를 전달받습니다. 결과의 원본 경로 열
  (`SourceFile`, `SourcePath`, …), 로그, 매니페스트의 `EvidenceRoot` 및 `WindowsDirectories`는 각 파일이 증거
  내 어디에 있는지 보여 줍니다(`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`).
- 매니페스트와 로그의 결과 경로(`Output`, `OutputFiles`, `Log`, `JSON-Timeline`, `Command` 내 출력 경로)와
  Defender의 `ExtractedFile`은 출력 디렉터리 기준의 상대 경로입니다(`./HOST1/ProcessExecution/Prefetch.csv`,
  출력 디렉터리 옆의 타임라인이면 `../siem.jsonl`). `Command`는 프로그램 이름(`juntei`)으로 시작하며, 그
  증거 경로는 증거 디렉터리 기준, 출력 경로는 출력 디렉터리 기준의 상대 경로입니다.
- `EvidenceDirectory`와 `OutputDirectory`는 두 디렉터리의 이름입니다. `ManifestVersion` 2는 출력 디렉터리
  기준으로 결과 경로를 기록하는 매니페스트를 나타냅니다. 이전 매니페스트에는 `ManifestVersion`이 없으며 경로를
  `-o`에 지정된 그대로 기록합니다. `-C` 재실행은 둘 다 읽습니다.
- 터미널은 출력 또는 타임라인 경로가 상대 경로이면 지정된 그대로, 절대 경로이면 현재 디렉터리 기준으로
  표시합니다(`../results/`).

병렬 프로세스 수 다음에 터미널은 발견된 Windows 시스템 파일의 총 크기를 표시합니다
(`Total filesize: 26.1 MiB`).

진행 메시지는 현재 처리 중인 컴퓨터와 아티팩트, 그리고 대략적인 입력 크기를 표시합니다. 예를 들어
`HOST1 / prefetch (~2.0 MiB)`입니다. 파일 그룹은 합계 크기를 사용하며, 각 MFT, LogFile 또는 USN 저널 작업은
개별 입력 크기를 표시합니다. 추정치는 보조 트랜잭션 로그와 보강용 파일을 제외하며, 파싱된 출력의 크기를
예측하지 않습니다.

`manifest.csv`는 같은 작업 색인을 스프레드시트에 적합한 테이블로 제공하며, 파서 작업당 한 행입니다. `Comp`,
원래 `Hostname`, `FQDN`, `HostnameSource`, `EvidenceRoot`, `Artifact`, `Category`, `OutputType`, `Purpose`,
`Status`, `DurationMs`, `ExitCode`, `Output`, `OutputFiles`, `Log`, `Error`, `Command`가 포함됩니다.
`Purpose`는 범주별 결과이면 `artifact`, CSV와 함께 사용되는 추가 네이티브 JSONL 처리이면 `json_timeline`입니다.
`OutputType`은 실제 파서 출력 형식입니다. `Command`는 실행 파일과 인수의 JSON 배열이며, 공백과 따옴표를
보존합니다. SRUM의 경우 `Output`은 컴퓨터 디렉터리, `OutputFiles`는 정확한 공급자 출력 경로의 JSON 배열이며,
`Category`는 공유 로그 위치(`SystemInformation`)를 나타냅니다. 파일은 개별적으로 분류되며, `OutputFiles`는 JSON
매니페스트에도 나타납니다.

검토가 필요한 작업을 찾으려면 `Status`를 `skipped` 또는 `failed`로 필터링하십시오. 건너뛴 작업의 `Error`는
비어 있을 수 있으며, 설명은 해당 `Log` 파일에 있습니다.

터미널 요약에는 두 매니페스트, 파서 로그, 추출된 파일을 포함하여 현재 출력 디렉터리 아래에 있는 일반 파일의
총 개수와 크기가 포함됩니다. 심볼릭 링크는 따라가지 않습니다. 크기는 `Total filesize`와 같은 포맷터를 사용하여
`KiB`, `MiB`, `GiB` 같은 단위로 자동 조정됩니다.

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

여러 원본 데이터베이스가 수집된 경우 Amcache와 SRUM 테이블에는 `-Source1`, `-Source2` 등이 붙습니다. 공유 파서
로그는 원래 경로에서 최종 경로로의 매핑을 기록하며, JSON/JSONL 레코드는 원본 출처 정보를 유지합니다. CSV
Amcache/Jump Lists 작업은 매니페스트 `OutputFiles`에 정확한 테이블 경로를 나열하며, `Output`은 조사 범주
디렉터리입니다. JSON 및 JSONL Amcache와 Jump Lists 내보내기는 이미 `Registry-Amcache.json[l]` 및
`JumpLists.json[l]`이라는 단일 파일입니다. 결합된 `-j` 타임라인은 같은 레코드 내용과 `Artifact` 값을 유지합니다.
`--clobber`는 이전 매니페스트를 사용하여 이전 이름을 포함한 기록된 출력을 지우며, 관련 없는 범주 파일은
보존합니다. 일반적인 단일 볼륨 수집물은 `$MFT`와 USN 저널 레코드를 `FileSystem/MFT.csv`와
`FileSystem/UsnJournal.csv`에 직접 기록합니다. 추가로 수집된 볼륨은 `FileSystem/MFT-2.csv`와
`FileSystem/MFT-2-logs.txt` 같은 번호가 붙은 형제 파일을 사용합니다.

## JSON 및 SIEM 가져오기 { #json-and-siem-ingestion }

모든 JSON 또는 JSONL 레코드에는 두 개의 최상위 필드가 붙습니다.

- `Comp`: 원본 시스템.
- `Artifact`: 레코드를 생성한 Juntei 파서(예: `mft` 또는 `prefetch-timeline`).

`-j, --json-timeline <FILE>`은 개별 결과 형식과 관계없이 하나의 추가 JSONL 파일을 생성합니다. 범주별 출력과
그 로그는 모두 유지됩니다.

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

파일 이름은 지정된 그대로 사용되며, 상대 경로는 현재 작업 디렉터리 기준입니다. 출력 루트, 예약되지 않은 다른
출력 하위 디렉터리, 또는 출력 디렉터리 외부에 둘 수 있습니다. 증거, 매니페스트, 컴퓨터별 결과/로그, 임시 파서
디렉터리는 대상으로 사용할 수 없습니다. 기존 파일이 있으면 `-C`가 필요하며, 내보내기가 실패하면 기존 파일은
그대로 유지됩니다.

**CSV 모드**에서 Juntei는 추가로 네이티브 JSONL 처리를 수행하므로, SIEM 파일은 CSV 테이블에서 생략될 수 있는
숫자 및 부울 유형, 중첩 데이터, 원시 필드, 메타데이터를 보존합니다. 추가 작업은 같은 `-T` 동시 실행 제한을
공유하고, 격리된 콘텐츠를 다시 추출하지 않으며, 별도의 `*-json-timeline-logs.txt` 부속 로그를 가집니다. 두
매니페스트는 그 목적과 출력 형식을 구별합니다. 임시 JSONL 파일은 실행이 성공하면 제거되며, 파서나 내보내기가
실패하면 유지됩니다. 추가 처리 시간과 임시 디스크 공간을 고려하십시오.

**JSON 모드**에서 Juntei는 증거를 다시 파싱하지 않고 기존 결과를 간결한 JSONL로 스트리밍합니다. Amcache
보고서는 네이티브 JSONL과 동일한 개별 요약, 발견 사항, 파일, 장치 레코드로 확장됩니다. **JSONL 모드**에서는
기존의 주석이 추가된 레코드가 그대로 결합됩니다. 모든 모드는 같은 SIEM 레코드 구조를 사용합니다. 데이터 세트는
네이티브 타임스탬프와 아티팩트 필드를 유지하며, 공통 이벤트 타임스탬프를 만들어 내거나 서로 다른 아티팩트를
전역적으로 정렬하지 않습니다.

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

`manifest.json`은 요청된 `JSON-Timeline` 경로와 `JSON-TimelineError`를 기록합니다. 작업 레코드와
`manifest.csv`는 여전히 유지된 아티팩트 출력을 가리킵니다. 성공한 CSV 타임라인 전용 작업은 삭제된 임시 파일
대신 최종 JSONL 파일을 참조합니다. 파서가 하나라도 실패하면 실행은 실패로 종료되며 타임라인에는 성공한 결과만
들어 있습니다. 완전한 것으로 취급하기 전에 매니페스트와 범주 로그를 확인하십시오. 파일 수/크기 요약은 출력
디렉터리 내부의 파일을 대상으로 하며, 외부 타임라인은 `JSONL timeline: <FILE>`로 별도 보고됩니다.

## 성능 { #performance }

파서 작업은 별도의 프로세스이므로 CPU 부하가 큰 아티팩트를 동시에 실행할 수 있으며, 한 파서의 손상이 전체
실행을 종료시키지 않습니다. 기본적으로 Juntei는 논리 CPU 수를 사용합니다. 증거가 느린 디스크에 있거나 시스템의
메모리가 적은 경우 동시 실행 수를 제한하십시오.

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## 사용법 { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]
      --vhdx                  Parse the files inside every VHDX disk image in the evidence directory, each mounted read-only with FUSE while it is parsed

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -Z, --timezone <OFFSET>             Display parsed timestamps using a fixed UTC offset (UTC by default) [default: +00:00]
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK files, Jump Lists, ShellBags and OpenSaveMRU [default: 1252]
      --mft-slack           Recover the file names left in the slack of $MFT records

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

가능하면 새 출력 디렉터리를 사용하십시오. `-C`는 이번 실행에서 생성되는 정확한 대상만 제거하거나 덮어쓰므로,
이제는 없는 아티팩트가 오래된 결과를 남기지 않습니다. 관련 없는 파일이나 출력 트리 전체를 삭제하지는 않습니다.
또한 새 실행에서 `-x`를 사용하는지와 관계없이, 파싱 전에 각 컴퓨터의 `SecurityProducts/DefenderQuarantined/`
디렉터리와 이전 평면 레이아웃의 예약된 `defender-quarantined/` 디렉터리를 제거합니다.

이전 평면 레이아웃에서 전환할 때는 새 출력 디렉터리를 사용하십시오. 이전 CSV/JSON 파일은 자동으로 이동되거나
삭제되지 않습니다.

!!! danger "추출된 Defender 콘텐츠는 악성코드로 취급하십시오"
    `-x, --extract-defender-quarantined`는 격리된 콘텐츠를 각 컴퓨터의
    `SecurityProducts/DefenderQuarantined/` 디렉터리 아래 디스크에 기록합니다. 복구된 바이너리 스트림은
    `.quarantined`로 끝나며, Juntei는 실행 권한을 제거합니다. 각 바이너리 보안 설명자에는 소유권, 권한, SDDL을
    담은 읽기 쉬운 `.security_descriptor.txt` 보고서도 생성됩니다. 복구된 페이로드를 절대 실행하지 마십시오.
    격리된 분석 시스템을 사용하고, 바이러스 백신이나 EDR 소프트웨어가 복구된 콘텐츠에 대해 경고하거나 이를
    제거할 수 있음을 예상하십시오.

보고서 필드와 그 포렌식상의 한계는
[복구된 보안 설명자 읽기](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)를
참조하십시오.

자동 분석은 사용자의 원격 연결(RDP, PuTTY, WinSCP 및 매핑된 드라이브)도 읽습니다.
