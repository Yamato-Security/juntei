# Persistence Hunter { #persistence-hunter }

지속성(persistence)은 재부팅, 로그인, 예약된 트리거 또는 기타 이벤트 이후에 프로그램이 다시 실행되도록
합니다. 공격자는 이 목적을 위해 일반적인 Windows 구성을 사용할 수 있습니다. 많은 정상 프로그램도 완전히
동일한 메커니즘을 사용하므로, 지속성 항목을 발견하는 것은 분석의 출발점입니다.

`juntei persistence-hunter`는
[Hexastrike의 PyrsistenceSniper](https://github.com/hexastrike/PyrsistenceSniper)를 네이티브 Rust로
이식한 것으로, 131개의 검사를 제공합니다. 130개 검사로 이루어진 PyrsistenceSniper 카탈로그에 네이티브
시작 메뉴 COM 검사가 추가되었습니다.

각 메커니즘, 증거 위치 및 탐지 결과의 조사 방법에 대한 설명은
[131개 검사 지속성 가이드](../persistence-hunter/index.md)를 참조하십시오.
[지속성 감사의 말](../persistence-hunter/acknowledgements.md)에서는 카탈로그의 기반이 된 Python 및
PowerShell 프로젝트를 소개합니다.

## 사용법 { #usage }

```text
Version: 1.0.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei persistence-hunter -d <EVIDENCE> [OPTIONS]

Input:
  -d, --directory <DIR>  Evidence directory (a mounted Windows volume or a collection of hosts)
  -f, --file <FILE>      One registry hive (filesystem checks need a directory)
      --no-logs          Read dirty hives without replaying transaction logs

Filtering:
      --profile <FILE>               Replace the bundled allow/block profile with a YAML profile
      --min-severity <MIN_SEVERITY>  Minimum classification to include: info, low, medium, high [default: medium] [possible values: info, low, medium, high]
      --check <ID>                   Run only these check or ATT&CK IDs (comma separated; repeatable) [alias: --technique]

General Options:
  -h, --help         Show the help menu
      --list-checks  Show every available check and its ATT&CK technique

Analysis:
      --no-timeline  Skip MFT and event-log timestamp correlation
      --mft <FILE>   MFT to use for timestamp correlation (one host only)

Performance:
  -T, --threads <N>  Worker threads (default: logical CPU count)

Output:
  -o, --output <FILE>              Save findings to a file
  -t, --output-type <OUTPUT_TYPE>  Output type: csv (default), json, jsonl; without -o, write data to stdout [possible values: csv, json, jsonl]
      --coverage <FILE>            Save per-check coverage and errors to CSV
  -C, --clobber                    Overwrite output files

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## 첫 번째 스캔 { #first-scan }

```bash
juntei persistence-hunter -d evidence/ -o PersistenceHunter.csv --coverage Coverage.csv
```

증거는 마운트된 Windows 볼륨, Velociraptor 수집 데이터, 또는 여러 컴퓨터의 수집 데이터가 들어 있는
디렉터리일 수 있습니다. 분석 중에 컴퓨터는 서로 분리되어 처리되며, 모든 결과에 `Comp`가 포함됩니다. Windows
경로를 확인할 때 수집 시 이스케이프된 파일 이름은 디코딩됩니다.

하이브 하나만 지정하는 경우:

```bash
juntei persistence-hunter -f SOFTWARE -o software-persistence.csv --no-timeline
```

하이브 하나만으로는 다른 하이브나 파일 시스템 증거가 필요한 검사를 수행할 수 없습니다. 빈 결과를 의미 있는
것으로 취급하기 전에 커버리지 보고서를 읽으십시오.

검사 ID와 ATT&CK 매핑을 나열하려면 다음을 실행합니다.

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,windows_service_image_path -o focused.csv
```

`--list-checks`가 출력하는 검사 ID나 `T1547.001` 같은 ATT&CK ID를 사용하십시오.
`T1547` 같은 상위 ID를 지정하면 그 하위 기법이 선택됩니다. `--technique`는 `--check`의 별칭입니다.
탐지 결과에는 각 검사의 기법, 설명 및 참조 링크가 포함됩니다.

## 병렬 스캔 { #parallel-scans }

이 명령어는 공유 작업자 풀을 사용하며, 기본값은 논리 CPU 수입니다. 작업자 수를 지정하려면
`-T, --threads <N>`을, 단일 스레드로 스캔하려면 `-T 1`을 설정하십시오.

```bash
juntei persistence-hunter -d evidence/ --min-severity info -T 4 -o PersistenceHunter.csv
```

증거 탐색, 독립적인 검사, 페이로드 메타데이터 및 MFT/EVTX 타임라인 소스는 병렬로 실행됩니다. 호스트의
로드된 레지스트리 하이브는 읽기 전용으로 공유되며, 같은 페이로드에 대한 캐시된 메타데이터는 한 번만
초기화됩니다. 수집 데이터 전체에서 하이브 메모리가 배로 늘어나지 않도록 호스트는 한 번에 하나씩 로드됩니다.
탐지 결과, 커버리지, 타임스탬프 선택 및 경고 순서는 스레드 수가 달라도 동일합니다.

`auto-parse`는 이미 파서 프로세스를 병렬로 실행하므로, 각 `persistence-hunter` 하위 프로세스에 작업자를 하나씩
할당합니다. 이 명령어 자체의 `-T` 옵션은 동시에 실행되는 파서 프로세스 수를 제어합니다.

## 수집할 증거 { #evidence-to-collect }

| 출처 | 검사 예 |
|---|---|
| `SYSTEM` 및 인접한 트랜잭션 로그 | 서비스, 서비스 복구 명령, LSA 패키지, 부팅 설정, 인증 공급자 |
| `SOFTWARE` 및 인접한 트랜잭션 로그 | Run 키, Winlogon, IFEO, COM 등록, TaskCache, Office 설정 |
| 각 사용자의 `NTUSER.DAT` 및 `UsrClass.dat` | 사용자 자동 실행, 환경 변수, COM 재정의, Office 설정 |
| `SAM` 및 `SOFTWARE` | RID 이상, 숨겨진 계정, 비정상적인 로컬 Administrators 구성원 |
| `Windows/System32/Tasks` | 예약된 작업의 명령과 COM 처리기, TaskCache와의 비교 |
| 사용자별 시작 메뉴 `.library-ms` 파일 및 CLSID 이름의 폴더 | 일치하는 사용자/머신 클래스 등록과 결합된 COM 활성화 트리거 후보 |
| 시작 프로그램 폴더, PowerShell 프로필, Office 템플릿, Terminal 설정 | 사용자 또는 시스템 트리거로 실행되는 파일 및 명령 |
| WMI `OBJECTS.DATA` | 이벤트 소비자 인스턴스 및 필터-소비자 바인딩 |
| BITS `qmgr.db` 또는 `qmgr*.dat` | 복구된 알림 명령 및 소유자 SID |
| `$MFT` 및 관련 EVTX 파일 | 독립적인 타임스탬프 상관 분석 |
| 참조된 실행 파일, DLL 및 카탈로그 파일 | SHA256 해시 및 설명용 PE/인증서 메타데이터 |

레지스트리 트랜잭션 로그는 사용 가능한 경우 재생됩니다. `--no-logs`는 하이브를 수집된 그대로 읽습니다.
재생되지 않은 더티 하이브에는 최근 변경 사항이 누락될 수 있으며, 이는 커버리지에 보고됩니다. 활성 시스템
검사에는 현재 제어 집합이 사용됩니다. DSRM 백도어 검사는 추가로 `ControlSet001`, `ControlSet002` 및
`CurrentControlSet`이 있는 경우 이를 검사합니다. 해당 설정을 활성 상태라고 기술하기 전에 보고된 경로를
`SYSTEM\\Select`와 대조하십시오. 그 밖의 과거 제어 집합은 개별 레지스트리 파서 명령어로 검사할 수 있습니다.

하이브 읽기는 컴퓨터당 1GiB의 입력 예산을 공유합니다. 머신 하이브를 사용자 하이브보다 먼저 시도하며, 예산에
맞지 않는 입력이나 트랜잭션 로그는 부분 커버리지에 이름이 기록됩니다. 다른 읽을 수 있는 하이브와 파일 기반
검사는 계속 진행됩니다. 하이브의 식별은 표준 하이브 위치나 명시적으로 제공된 내보내기 파일로만 확정되며,
사용자의 임시 디렉터리 안에 있는 사본이 해당 사용자의 실제 하이브를 대체할 수는 없습니다. SOFTWARE에
기록된 원래 시스템 드라이브는 캡처된 시스템 볼륨에 매핑되므로, 원래의 `C:` 볼륨을 다른 증거 드라이브
레이블로 마운트하더라도 페이로드 메타데이터가 버려지지 않습니다.

`auto-parse`도 헌터를 실행하여 각 컴퓨터 디렉터리 안에 `persistence/PersistenceHunter.csv`(또는
`.json`/`.jsonl`)를 저장하고, 그 옆에 `PersistenceHunter-logs.txt`를 둡니다. 탐지 결과는 선택 사항인
`-j <FILE>` SIEM 타임라인에도 포함됩니다. 자동 파싱은 허용 규칙과 일치하는 항목을 포함한 전체 구성
인벤토리를 유지하기 위해 `--min-severity info`를 사용합니다.

## COM 등록과 활성화 트리거 { #com-registrations-and-activation-triggers }

`scheduled_task_files` 검사는 작업 XML의 `Actions Context`와 일치하는 `Principal id`를 사용하여 각 동작의
주체를 선택합니다. 그룹 주체는 그룹으로 유지되며, 어느 개별 사용자의 클래스 등록이 사용될지를 확정할 수
없습니다.

COM 처리기 후보는 네이티브 클래스 보기와 32비트 클래스 보기에서 각각 별도로 확인됩니다. 일치하는 사용자의
캡처된 `UsrClass.dat`/`NTUSER.DAT`와 머신의 `SOFTWARE`만 고려됩니다. SID 주체는 캡처된 `ProfileList`
경로를 통해 결합됩니다. `HOST\user` 및 `DOMAIN\user`를 포함한 계정 이름은 일반 `user` 프로필보다 일치하는
`user.DOMAIN` 프로필을 우선합니다. 로컬 호스트 별칭과 `.\user`는 접미사 매칭에 캡처된 호스트 이름을
사용하며, 정확한 접미사가 다른 호스트 별칭보다 우선합니다. 일반 프로필은 한정된 프로필이 하나도 일치하지 않을
때만 고려됩니다. 이러한 모든 일치는 `Description`에 **폴더 이름 휴리스틱**으로 명시적으로 표시되며, 계정
소유권이나 도메인 구성원 자격을 증명하지 않습니다. 모호한 일치는 확인되지 않은 상태로 남으며 탐지 결과 설명과
커버리지 모두에 보고됩니다. 한정된 프로필 간의 모호성으로 인해 일반 프로필로 대체되지는 않습니다. 이 처리는
보통 메모리에만 존재하고 레지스트리 하이브와 함께 저장되지 않는 `Volatile Environment`에 의존하지 않습니다.
중복된 사용자 CLSID는 머신 CLSID의 하위 항목을 가립니다. 그룹 주체는 머신 COM 후보를 사용하며 탐지 결과
설명에 그 제한 사항을 유지합니다. 일반적인 그룹 주체 자체로 인해 커버리지가 부분적으로 처리되거나 오류 요약
항목을 소비하지는 않습니다.

리졸버는 `InprocServer32`와 `LocalServer32`를 따라가고, 관리 코드의 `CodeBase` 경로(버전 하위 키 포함)를
사용하며, AppID의 `LocalService`를 수집된 SYSTEM 서비스의 `ImagePath`와 결합합니다. AppID의 실행 파일 이름
연결은 유지되지만, 기본 이름만으로 실행 파일의 전체 경로가 확정되지는 않습니다. 로컬 `file:` CodeBase URL은
디코딩되며, 원격 URL은 액세스하지 않고 기록만 합니다. 누락된 등록은
`ResolutionStatus: unresolved_com_class`, 빈 경로/해시 필드, 알 수 없는 `Exists`와 함께 CLSID를 유지합니다.

`start_menu_com` 검사는 사용자별 시작 메뉴 `.library-ms` XML URL을 읽고, 빈 디렉터리를 포함하여 CLSID
이름을 가진 폴더를 수집합니다. 트리거 경로, CLSID, 후보 서버 및 레지스트리 출처 정보를 보고합니다. 파일만
수집하는 방식으로는 빈 디렉터리를 보존할 수 없으며, 사용자 하이브가 없으면 사용자별 등록을 완전히 조회할 수
없습니다. 심볼릭 링크는 따라가지 않습니다. 이는 구성 후보입니다. 아키텍처, 권한 상승, UAC, 정책 및 활성화
컨텍스트에 따라 등록이 실제로 사용되는지가 달라질 수 있습니다. 실행 여부는 프로세스, 모듈 로드 또는 기타
독립적인 증거로 교차 검증하십시오.

## 분류와 프로필 { #classification-and-profiles }

기본 최소 분류는 `medium`입니다. 번들 허용 목록과 일치하는 항목을 포함한 전체 인벤토리를 검사하려면 다음을
실행합니다.

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o all-persistence.csv
```

!!! warning "불완전한 수집 데이터를 검토할 때는 모든 심각도를 사용하십시오"
    광범위한 허용 규칙은 서명자가 없거나 다른 경우에도 공격자의 명령과 부분적으로 일치할 수 있습니다.
    업스트림은 부분 일치를 `LOW`로 분류하며, 수동 명령어의 기본 임계값인 `medium`에서는 이것이 숨겨집니다.
    이는 Windows 디렉터리 아래의 악성 예약된 작업에 영향을 줄 수 있습니다. 완전한 조사를 위해서는
    `--min-severity info`를 사용하고, `PolicyReason`, `SignerStatus`, 명령 및 원본 증거를 함께
    검토하십시오. `auto-parse`는 이미 이러한 항목을 유지합니다.

| 분류 | 의미 |
|---|---|
| `HIGH` | 구성된 차단 규칙이 완전히 일치함 |
| `MEDIUM` | 허용 규칙이나 차단 규칙이 하나도 일치하지 않음 |
| `LOW` | 허용 규칙의 필수 조건은 일치했지만 서명자 메타데이터가 일치하지 않았거나 사용할 수 없었음 |
| `INFO` | 허용 규칙이 완전히 일치함 |

이는 정책상의 분류이며, 측정된 침해 확률이 아닙니다. 기본 `MEDIUM` 항목이 악성 코드의 증거는 아닙니다.
`INFO`는 파일에 유효한 서명이 있다거나 공격자가 그것을 악용하지 않았다는 것을 확정하지 않습니다. 부분적인
허용 일치는 기본 임계값보다 낮으므로, 부족한 수집 데이터를 검토하거나 허용 목록을 조정할 때는
`--min-severity low` 또는 `info`를 포함하십시오.

`--profile profile.yaml`로 YAML 프로필을 제공하십시오. 이 프로필은 번들 프로필을 **대체**합니다. 선택한 파일
안의 전역 규칙과 검사별 규칙은 결합되며, 완전히 일치하는 차단 규칙이 하나라도 있으면 그것이 우선합니다.
예를 들면 다음과 같습니다.

```yaml
allow: []
block:
  - reason: Example known-bad command fragment
    value_matches: 'example-malicious-loader\.exe'
checks:
  windows_service_image_path:
    enabled: true
    allow:
      - reason: Organization's expected service path
        value_matches: '^C:\\Program Files\\ExampleCorp\\Agent\\agent\.exe$'
        signer: ExampleCorp
```

지원되는 규칙 조건은 `path_matches`, `value_matches`, `hash`(64자리 16진수 SHA256), `not_lolbin`,
`signer`(대소문자를 구분하지 않는 부분 문자열)입니다. 명시된 모든 필수 조건이 일치해야 합니다. 서명자
메타데이터는 위에서 설명한 선택 조건입니다. 정규식은 Rust의 정규 표현식 구문을 사용하며, 전후방 탐색
(lookaround)과 역참조는 지원되지 않습니다. 알 수 없는 검사 ID, 잘못된 표현식 및 알 수 없는 프로필 필드는
정책을 조용히 약화시키지 않고 오류를 발생시킵니다.

## 출력 필드 { #output-fields }

CSV가 기본 파일 형식입니다. 읽기 쉬운 JSON 객체에는 `-t json`을, 한 줄에 하나의 객체가 들어가는 간결한
출력에는 `-t jsonl`을 사용하십시오. `-o` 없이 `-t`를 명시하면 데이터만 stdout에 기록되며, 커버리지 경고는
stderr로 출력됩니다. 두 옵션이 모두 없으면 Juntei는 요약과 최대 30개의 후보를 출력합니다.

| 필드 | 해석 방법 |
|---|---|
| `Comp`, `CheckID`, `Technique`, `MitreID` | 해당 행이 나타내는 컴퓨터와 지속성 메커니즘 |
| `Path`, `Value` | 레지스트리 위치, 파일, 작업 또는 복구된 레코드와 그 구성 내용 |
| `Description`, `References` | 해당 메커니즘이 중요한 이유와 추가 정보를 읽을 수 있는 곳 |
| `AccessGained` | 메커니즘의 일반적인 범위(`USER` 또는 `SYSTEM`, BITS는 `LOCAL_SERVICE`/`NETWORK_SERVICE`도 식별 가능)이며, 특정 명령이 해당 권한을 얻었다는 증거가 아님 |
| `Severity`, `PolicyReason` | 프로필 분류, 일치한 정책에 대한 설명, 그리고 부분 일치 또는 확인되지 않은 서명자에 대한 단서 |
| `SourceFile`, `RecordStatus` | 정확한 수집 원본(입력으로 지정된 경로로 시작하는 경로이며, auto-parse에서는 `./HOST/...`)과 할당/재생/복구 컨텍스트. 이를 따라 원래 증거로 거슬러 올라가십시오 |
| `KeyLastWrite` | 레지스트리 **키의** 타임스탬프. 개별 값의 날짜를 나타내지 않음 |
| `LastChange`, `ChangeSource`, `ChangeEvidence` | 독립적으로 상관 분석된 변경 시간, 출처 증거, 그리고 상관 분석의 성공 여부 |
| `ResolvedPath`, `ResolutionStatus`, `Exists` | 페이로드 경로와 오프라인 확인 결과. 수집되지 않은 증거는 `false`가 아니라 알 수 없음 |
| `SHA256` | 읽을 수 있는 경우 수집된 페이로드 바이트의 SHA256 |
| `Signer`, `SignerStatus` | 설명용으로 포함된 인증서/카탈로그 메타데이터와 그 사용 가능 여부. 인증서 신뢰와 서명 유효성은 검증되지 않음 |
| `IsLolbin`, `Launcher` | 구성된 실행기 또는 페이로드가 번들 LOLBAS 목록에 있는지 여부와, 프록시된 페이로드에 사용된 실행기 |
| `IsBuiltin`, `IsInOS-Directory` | 일반적인 Windows 명령과 시스템 디렉터리에 대한 컨텍스트. 어느 필드도 정당성을 확정하지 않음 |

`rundll32.exe` 명령은 DLL 페이로드를 지정할 수 있습니다. Juntei는 페이로드를 확인하고 해시를 계산하면서
실행기도 보존합니다. 수집된 컴퓨터 밖을 가리키는 참조, 누락된 드라이브, 대소문자를 구분하지 않는 모호한 일치,
또는 확인되지 않은 환경 변수는 분석가 머신의 파일로 대체되지 않습니다.

## 타임스탬프와 제한 사항 { #timestamps-and-limitations }

헌터는 분석가 VM에 복사된 파일의 수정 시간을 공격 타임스탬프로 사용하지 않습니다. 사용 가능한 경우 레지스트리
쓰기는 Sysmon 이벤트 13과, 작업 설치는 Security 4698 또는 Task Scheduler 106과, 서비스 설치는 System 7045와,
파일 기반 구성은 원래의 `$MFT` 타임스탬프와 상관 분석합니다. 그 출처는 `ChangeSource`에 표시됩니다.
이벤트는 구성 변경의 날짜를 알려 줄 수 있지만, 구성된 페이로드가 실행되었음을 증명하지는 않습니다.
로컬 System 이벤트 6011은 이전 컴퓨터 이름을 증명할 수 있습니다. 헌터는 수집된 컴퓨터를 기준으로 검증된 이름
변경 체인을 따라가며, 현재 `Comp` 값을 유지하면서 원래 이벤트의 컴퓨터 이름과 이름 변경 레코드 증거를
`ChangeSource`에 기록합니다. 증명되지 않은 외부 이름, 보관된 로그, 전달된 로그, 그리고 다른 Windows 설치는
이 규칙으로 인정되지 않습니다.

캡처된 ComputerName/ActiveComputerName, TCP/IP Hostname/NV Hostname, 그리고 client-info의 hostname/FQDN이
호스트 별칭을 제공하며, DNS/NetBIOS 차이와 보류 중인 이름 변경을 포괄합니다. 캡처된 식별 정보가 없으면,
기본 로그에 있는 단일 이름이 확인되지 않은 식별 정보라는 명시적 출처 표시 및 커버리지 경고와 함께 날짜 판단을
뒷받침할 수 있습니다. 확인되지 않은 이름이 여러 개이면 호스트에 귀속되지 않습니다. 거부된 Computer 이름은
상관 분석 커버리지의 제한 사항으로 보고되며, 해당 채널은 여전히 수집된 것으로 표시됩니다.

단일 컴퓨터에 대해 외부에서 수집한 MFT를 선택하려면 `--mft <FILE>`을 사용하십시오. 타임스탬프 상관 분석을
건너뛰려면 `--no-timeline`을 사용하십시오. `NOT_RUN`, `NOT_APPLICABLE`, `NO_ARTIFACT`, `NO_MATCH` 및
`REJECTED`는 `LastChange`가 비어 있는 이유를 설명합니다. 빈 필드가 해당 항목이 무해하다는 증거는 아닙니다.

WMI 및 BITS 복구는 완전한 데이터베이스 트랜잭션 파서가 아니라 범위가 제한된 카빙입니다. 복구된 문자열이나
바인딩에는 오래되었거나 삭제된 데이터가 포함될 수 있으므로, 이러한 레코드는 반드시 교차 검증해야 합니다.
WMI 클래스 스키마만으로는 구독으로 집계되지 않습니다. 수집된 작업 XML 파일이 없는 TaskCache 항목은 검토가
필요한 불일치이지만, 트리아지 수집에서 해당 파일이 단순히 누락되었을 수도 있습니다. 카탈로그 및 PE
메타데이터는 검증되지 않으며 수집 데이터에 없을 수 있습니다. 접근성 파일 교체 검사는 적절한 참조 서명
메타데이터에 의존하므로, 카탈로그 증거가 없다고 해서 모든 접근성 실행 파일이 표시되어서는 안 됩니다.

커버리지는 탐지 결과가 없는 검사를 포함하여 모든 검사에 대해 기록됩니다. `complete`는 해당하는 수집 원본을
읽었다는 뜻이며, 수집이 완전하다거나 컴퓨터에 다른 지속성이 없다는 것을 증명하지는 않습니다. `partial`,
`not_collected`, `failed` 및 `disabled`는 한계를 설명합니다. 탐지 결과와 함께 `--coverage Coverage.csv`와
파서 로그를 보관하십시오.

이 이식판은 Juntei의 CSV/JSON/JSONL 형식을 사용합니다. 업스트림 Python 프로그램을 호출하거나, 온라인 보강
서비스를 조회하거나, 업스트림의 HTML/XLSX UI를 재현하지 않습니다.

## 추가 정보 { #further-details }

- [PyrsistenceSniper 구현 및 프로필](https://github.com/hexastrike/PyrsistenceSniper)
- [MITRE ATT&CK 지속성 기법](https://attack.mitre.org/tactics/TA0003/)
- [Microsoft: Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
- [레지스트리 자동 실행 인벤토리](../artifacts/persistence.md)
- [서비스](../artifacts/services.md) 및 [TaskCache](../artifacts/taskcache.md)
