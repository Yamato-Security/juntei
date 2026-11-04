# 작업 스케줄러 캐시 { #task-scheduler-cache }

## TaskCache란 { #what-taskcache-is }

Windows 작업 스케줄러는 `SOFTWARE` 아래에 작업 경로를 GUID, 트리거/목록, 보안 주체, 동작, 런타임
메타데이터 및 보안 설명자와 연결하는 레지스트리 캐시를 보관합니다. Juntei는 실행 및 COM 처리기 동작과
레거시 이메일/메시지 동작, 작업 타임스탬프, 그리고 일반적인 작업 스케줄러 인터페이스에서 숨겨진 작업의
징후를 디코딩합니다.

작업은 흔히 사용되는 지속성 및 원격 실행 메커니즘입니다. 레지스트리 캐시는 공격자가 작업의 XML 파일을
삭제하거나 변조한 경우에도 가치가 있습니다.

<span class="evidence-strength">가장 타당한 결론:</span> TaskCache가 해당 작업/동작을 구성했거나
보존했습니다. 특정 실행을 증명하려면 런타임 필드와 독립적인 로그가 필요합니다.

## 위치 { #locations }

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

`SOFTWARE`를 트랜잭션 로그와 함께 수집하고, `System32\Tasks` 트리 전체도 수집하십시오. 작업 스케줄러
Operational 및 Security 이벤트 로그는 생성/업데이트/런타임 증거를 추가로 제공합니다.

## 분석 시 주의 사항 { #analyst-cautions }

- 변조, 삭제, 불완전한 등록 또는 손상 이후에는 레지스트리 캐시와 작업 XML의 내용이 일치하지 않을 수 있습니다.
- Juntei의 `Hidden`은 구조적인 숨김(관련 조건에서 Tree 연결/보안 설명자 누락)을 식별하며, 단순히 일반적인
  “숨김” UI 플래그로 구성된 작업을 뜻하지 않습니다.
- Windows 7은 작업 동작을 캐시의 `Actions` 값이 아니라 XML에 저장합니다. 따라서 그 환경에서는 동작이
  비어 있다고 해서 자동으로 의심스러운 것은 아닙니다.
- `LastRun`과 `LastResult`는 유용하지만 완전하거나 변조 불가능한 실행 기록이 아닙니다.
- `Context`는 구성된 보안 주체/컨텍스트를 나타내며, 작업을 등록한 사용자가 아닙니다.
- COM 처리기 CLSID는 별도의 레지스트리/파일 해석이 필요합니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.Hidden == true or (.Executable? | test("powershell|cmd|mshta"; "i")))'
```

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `SourcePath` | 호스트 컨텍스트와 원본 `SOFTWARE` 하이브입니다. |
| `TaskName` | 스케줄러의 전체 경로입니다. XML 및 이벤트 로그로 피벗하기 위한 주요 기준입니다. |
| `TaskGUID` | Tree 및 Tasks 캐시 엔트리를 연결하는 GUID입니다. 강력한 상관 분석 키입니다. |
| `TaskIndex` | 스케줄러가 사용하는 Tree/인덱스 메타데이터입니다. 컨텍스트 정보이며 실행 횟수가 아닙니다. |
| `TaskLists` | 트리거/처리 방식을 분류하는 캐시 목록(`Boot`, `Logon`, `Plain`, `Maintenance`)입니다. |
| `Hidden` | 구조적으로 숨겨진 작업의 지표입니다. 변조라고 결론 내리기 전에 `InTree`, `InTasks` 및 설명자 상태를 검토하십시오. |
| `InTree`, `InTasks` | 캐시의 양쪽이 모두 존재하는지 여부입니다. 연결 누락은 삭제/변조/불완전한 등록을 나타낼 수 있습니다. |
| `HasSecurityDescriptor` | Tree 엔트리에 `SD`가 있는지 여부입니다. 해당 버전에서는 이것이 없으면 작업이 숨겨지는 동작의 원인이 될 수 있습니다. |
| `TreeKeyLastWrittenUTC`, `TaskKeyLastWrittenUTC` | 각 캐시 측의 키 수준 변경 시간입니다. 정확한 생성/실행 시간이 아닙니다. |
| `ActionIndex` | 작업 내의 0부터 시작하는 동작 번호입니다. 하나의 작업이 여러 행을 생성할 수 있습니다. |
| `ActionType` | `exec`, `com_handler`, `send_email` 또는 `show_message`입니다. |
| `Executable`, `Args`, `WorkingDirectory` | 실제로 적용되는 실행 동작입니다. 따옴표, 변수, 상대 경로 및 LOLBin을 해석하십시오. |
| `COM-ClassID`, `ActionData` | COM 처리기 CLSID와 데이터, 또는 레거시 이메일/메시지 페이로드입니다. CLSID는 별도로 해석하십시오. |
| `Context` | 동작과 함께 인코딩된 보안 주체 컨텍스트(`LocalSystem`, `Author` 등)입니다. 구성된 ID이며 등록자가 아닙니다. |
| `Author`, `URI`, `Description` | 작업 메타데이터입니다. 공격자가 쉽게 제어할 수 있으며 신뢰의 증거가 아닙니다. |
| `LastRegisteredUTC` | 존재하는 경우 DynamicInfo의 등록/업데이트 시간입니다. |
| `LastRunUTC`, `LastSuccessfulRunUTC`, `LastResult` | 캐시된 런타임/결과 메타데이터입니다. Operational 로그 및 프로세스 증거와 상관 분석하십시오. |
| `ActionsRaw` | 디코딩이 불완전하거나 검증이 필요한 경우의 원래 동작 바이트입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 경로/GUID 불일치, 잘못된 값 유형, 잘린 동작 구조 또는 누락된 부분입니다. |

## 조사 전략 { #investigation-strategy }

1. 숨겨졌거나 일관성이 없는 작업과 Microsoft가 아닌 경로를 먼저 검토하십시오.
2. 인수, 작업 디렉터리, 환경 변수 확장 및 COM 해석을 포함하여 실제로 적용되는 명령을 재구성하십시오.
3. 레지스트리 레코드를 작업 XML과 비교하고, 참조된 실행 파일의 해시와 타임스탬프를 확인하십시오.
4. `TaskName`/GUID/시간을 작업 스케줄러 이벤트, Security 4698/4702, 프로세스 로그, Prefetch 및 MFT/USN과 상관 분석하십시오.
5. 공격자가 작업을 제거한 경우 삭제/복구된 레지스트리 데이터를 검색하십시오.

## 추가 정보 { #further-details }

- [Microsoft: 작업 스케줄러 스키마](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft: 작업 스케줄러 이벤트 로그](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 Scheduled Task](https://attack.mitre.org/techniques/T1053/005/)
