# Windows 서비스 { #windows-services }

## 서비스 구성이 기록하는 내용 { #what-service-configuration-records }

Windows 서비스와 커널 드라이버는 각 `SYSTEM` 제어 집합(control set) 아래에 구성됩니다. 레코드에는
실행 파일/드라이버 또는 서비스 DLL, 시작 모드, 서비스 유형, 계정, 로드 순서 그룹, 지연된 시작,
오류 처리 방식, 그리고 실패 후 Windows가 수행하는 동작이 기록됩니다.

공격자는 지속성 확보, 권한 상승, 드라이버 로드 및 측면 이동을 위해 서비스를 생성하거나 수정합니다.
정상 소프트웨어도 완전히 동일한 메커니즘을 사용하므로 경로, 계정, 서명, 생성 컨텍스트 및 런타임
증거가 중요합니다.

<span class="evidence-strength">가장 타당한 결론:</span> 분석한 제어 집합에 서비스/드라이버가 표시된
대로 구성되어 있었습니다. 구성만으로는 서비스가 성공적으로 시작되었음을 증명하지 못합니다.

## 위치 { #location }

```text
SYSTEM\ControlSet00N\Services\<ServiceName>
SYSTEM\Select\Current   (which control set Windows used)
```

`SYSTEM`을 트랜잭션 로그와 함께 수집하십시오. System 7045 및 Security 4697 같은 이벤트 ID는 설치를
기록할 수 있으며, 서비스 제어 및 프로세스 이벤트는 실제 시작/실행을 뒷받침할 수 있습니다.

## 분석 시 주의 사항 { #analyst-cautions }

- `ImagePath`에는 환경 변수, 따옴표, 인수, 장치 경로 또는 공백이 포함된 따옴표 없는 경로가 들어 있을 수
  있습니다. 정규화하기 전에 원래 값을 그대로 보존하십시오.
- `svchost.exe`에서 호스팅되는 서비스에서는 `ServiceDll`이 매우 중요합니다. `ImagePath`만으로는 호스트만 표시될 수 있습니다.
- `StartType=automatic`은 서비스가 시작되었거나 계속 활성화되어 있었음을 증명하지 않습니다.
- `ObjectName`은 구성된 계정이며, 반드시 서비스를 설치한 사용자인 것은 아닙니다.
- 드라이버와 사용자 모드 서비스는 서로 다른 `ServiceType` 플래그와 경로 규칙을 사용합니다.
- 실패 시 동작은 실패 후 서비스 계정으로 명령을 실행할 수 있습니다. 이는 놓치기 쉬운 지속성
  공격 표면입니다.
- 다른 제어 집합에 이전 구성이 보존되어 있을 수 있습니다. `IsCurrentControlSet`을 사용하십시오.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --service <NAME>    Only this service name
      --all-control-sets  Parse every ControlSet00N

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --all-control-sets -t jsonl \
  | jq 'select(.FailureCommand != null or (.ImagePath? | test("temp|public"; "i")))'
```

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `SourcePath` | 시스템 식별 정보와 입력 하이브입니다. 여러 수집본에 걸쳐 보존하십시오. |
| `ControlSet`, `IsCurrentControlSet` | 원본 제어 집합과 `Select\Current`가 이를 가리키는지 여부입니다. 활성 상태를 주장할 때는 현재 제어 집합을 우선하십시오. |
| `ServiceName` | 레지스트리 키/내부 서비스 이름입니다. 이벤트 및 서비스 제어 관리자(Service Control Manager) 명령으로 피벗하기 좋은 강력한 기준입니다. |
| `DisplayName`, `Description` | 사용자에게 표시되는 텍스트입니다. 리소스 참조이거나 공격자가 제어하는 값일 수 있습니다. |
| `ImagePath` | 구성된 그대로의 실행 파일/드라이버 명령입니다. 따옴표, 인수, 환경 변수 및 장치 경로를 분석하십시오. |
| `ServiceDLL` | 호스트 서비스가 로드하는 DLL입니다. 공유 프로세스 서비스에서는 이것이 실제 페이로드인 경우가 많습니다. |
| `StartType`, `StartValue` | 사람이 읽기 쉬운 형태/원시 값의 시작 모드입니다: boot (0), system (1), automatic (2), manual (3), disabled (4). |
| `DelayedAutostart` | 초기 부팅 단계 이후에 시작되는 자동 서비스입니다. 여전히 지속성에 이용될 수 있습니다. |
| `ServiceType`, `TypeValue` | 디코딩된/원시 플래그입니다: 커널 드라이버, 파일 시스템 드라이버, 단독/공유 프로세스, 사용자 서비스 등. |
| `ErrorControl` | 부팅/시작 시 오류 처리 수준입니다. 주로 운영상의 컨텍스트입니다. |
| `ObjectName` | 구성된 서비스 계정 또는 드라이버 개체 이름입니다. 설치자의 신원이 아닙니다. |
| `Group` | 로드 순서 그룹으로, 특히 boot/system 드라이버에서 중요합니다. |
| `FailureResetPeriod` | 실패 횟수가 초기화되기까지의 초 단위 시간입니다. `4294967295`는 초기화되지 않음을 의미합니다. |
| `FailureCommand` | `run_command` 실패 동작에 구성된 명령입니다. 지속성 분석에서 가치가 높은 필드입니다. |
| `FailureActions` | 디코딩된 전체 동작 목록(`restart`, `reboot`, `run_command`, `none`)과 지연 시간입니다. |
| `FailureActionType`, `FailureActionDelayMs` | 필터링을 위해 평탄화한 첫 번째 동작입니다. 전체 목록도 확인하십시오. |
| `FailureActionsRaw` | 디코딩이 불완전하거나 손상된 경우의 원본 바이트입니다. |
| `KeyLastWrittenUTC` | 서비스 키의 마지막 변경 시간이며, 반드시 설치/시작 시간인 것은 아닙니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 재생(replay) 컨텍스트입니다. |
| `Problems` | 형식 불일치, 잘못된 실패 구조, 누락되었거나 읽을 수 없는 필드입니다. |

## 조사 전략 { #investigation-strategy }

1. 비표준 경로, 쓰기 가능한 디렉터리, 비정상적인 계정, 새로운 키 시간 및 실패 명령을 검토하십시오.
2. 환경 변수는 분석가의 머신이 아니라 대상 호스트의 구성을 기준으로 확장하십시오.
3. 실제 실행 파일이나 DLL이 수집되었다면 해시와 서명을 확인하십시오.
4. 서비스 이름/경로를 이벤트 7045/4697, 프로세스 생성, Prefetch, `$MFT` 및 USN과 상관 분석하십시오.
5. 현재 제어 집합과 현재가 아닌 제어 집합을 비교하여 변경/삭제된 지속성을 찾으십시오.

## 추가 정보 { #further-details }

- [Microsoft: 장치 및 드라이버 서비스용 레지스트리 트리](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: SERVICE_STATUS / 서비스 유형 및 상태](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_status)
- [Microsoft: ChangeServiceConfig2 실패 시 동작](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
