# Windows 레지스트리 하이브 { #windows-registry-hives }

## 레지스트리 하이브란 { #what-registry-hives-are }

Windows 레지스트리는 시스템 및 사용자 구성을 바이너리 하이브 파일에 저장합니다. 키는 트리를 이루며, 키 아래의
값에는 유형이 지정된 데이터가 들어 있습니다. 레지스트리 아티팩트는 지속성, 설치된 소프트웨어, 서비스, 계정,
네트워크 구성, 사용자 활동, 보안 설정 등 Windows 상태의 거의 모든 부분을 설명합니다.

Juntei의 범용 리더는 모든 기본 하이브를 디코딩하고, 트랜잭션 로그를 재생하며, 손상을 보고하고, 삭제된 레코드
/빈 셀, 셀 슬랙, 잔여 데이터, 재생 전 상태, 트랜잭션 로그에만 있는 레코드를 복구할 수 있습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 하이브에 해당 키/값 상태가 들어 있었거나 복구 가능한
형태로 보존되어 있었습니다. 키의 마지막 쓰기 시간은 키가 변경되었음을 보여 줄 뿐, 어떤 값이 왜 변경되었는지는
보여 주지 않습니다.

## 일반적인 하이브와 위치 { #common-hives-and-locations }

| 하이브 | 일반적인 경로 | 증거의 예 |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | 서비스, 드라이버, 제어 집합, 호스트 이름, 마운트된 장치, ShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | 설치된 소프트웨어, TaskCache, 프로필, Windows 구성 |
| `SAM` | `C:\Windows\System32\config\SAM` | 로컬 계정과 계정 메타데이터 |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | 로컬 보안 정책/비밀 정보 메타데이터(접근 권한에 따라 다름) |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | 기본 시스템 프로필 구성 |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | 사용자 기본 설정, Run 키, RecentDocs, 애플리케이션 활동 |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | 셸 클래스, COM/사용자 셸 활동 |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | 애플리케이션 인벤토리(의미 해석된 출력은 `parse-registry amcache` 사용) |

인접한 `.LOG`, `.LOG1`, `.LOG2` 파일도 수집하십시오. Windows가 실행 중일 때 복사한 하이브는 더티 상태일 수
있습니다. 트랜잭션 로그에는 기본 파일만으로는 아직 반영되지 않은 커밋된 상태가 들어 있습니다.

## 분석 시 주의 사항 { #analyst-cautions }

- 키의 마지막 쓰기 타임스탬프는 개별 값이 아니라 **키**에 적용됩니다.
- 레지스트리 값에는 일반적으로 독립적인 타임스탬프가 없습니다.
- `CurrentControlSet`은 `SYSTEM\Select`를 통해 해석되는 분석가/Windows용 별칭이며, 실제로 저장된 키가
  아닙니다.
- 삭제된 레코드는 불완전하거나, 오래되었거나, 여러 복구 소스에 중복되어 있거나, 덮어쓰였을 수 있습니다.
- 트랜잭션 로그에만 있는 레코드는 과거/중간 상태를 보여 줍니다. `recovered_from`을 설명하십시오.
- `absent_value` 결과는 조회한 키가 존재하지만 파싱된 상태에 해당 값이 없다는 뜻이며, 그 값이 한 번도 존재하지
  않았다는 것을 증명하지 않습니다.
- 문자열, 경로, 명령줄은 구성 정보이며, 구성된 동작이 실행되었다는 증거가 아닙니다.
- 적용 가능한 로그가 없는 더티 하이브는 상태를 누락하거나 오래된 상태를 드러낼 수 있습니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host` | 사용 가능한 시스템 컨텍스트에서 도출한 호스트 이름. Juntei가 안전하게 식별할 수 없으면 비어 있습니다. |
| `HiveType` | 감지된 하이브 범주(`SYSTEM`, `SOFTWARE`, `NTUSER.DAT` 등). 경로를 해석할 때 사용합니다. |
| `SourcePath` | 정확한 입력 하이브. 디렉터리 모드에서 여러 호스트/사용자를 결합할 때 필수적입니다. |
| `RecordType` | `key`, `value`, `default_value` 또는 `absent_value`. 부재는 조회/상태에만 적용됩니다. |
| `KeyPath` | 하이브 루트 아래의 경로. `[unknown]`은 해석되지 않은 삭제된 상위 경로를 나타냅니다. |
| `KeyLastWrittenUTC` | 키 전체의 마지막 수정 시간. 어떤 값이 변경되었는지는 식별할 수 없습니다. |
| `ValueName` | 이름이 있는 값. 기본/이름 없는 값의 경우 비어 있습니다. |
| `ValueType` | 레지스트리 유형(`REG_SZ`, `REG_DWORD`, `REG_BINARY` 등). 유형은 해석에 영향을 줍니다. |
| `ValueData` | 유형에 맞게 렌더링된 값. 다중 문자열은 JSON에서 목록으로 유지됩니다. |
| `DataSize` | 값이 기록한 크기. 원시 바이트 및 `Problems`와 비교하십시오. |
| `ValueDataRaw` | 디코딩이 실패하거나 `--raw-data`를 사용할 때의 정확한 바이트. 검증을 위해 보존하십시오. |
| `ValueU64` | JavaScript 정밀도 손실 없는 DWORD/QWORD의 전체 숫자 표현. |
| `ValueFiletimeUTC` | 선택적인 QWORD의 FILETIME 렌더링. 키의 스키마가 FILETIME을 정의하는 경우에만 의미가 있습니다. |
| `AllocationStatus` | `allocated`, `deleted_complete`, `deleted_partial`, `residual_value_only` 또는 `transaction_log_recovered`. 신뢰도를 결정합니다. |
| `RecoveredFrom` | `free_cell`, `cell_slack`, `remnant_data`, `before_replay` 또는 `SYSTEM.LOG1#1234` 같은 로그 항목. |
| `TransactionLogStatus` | 이 레코드/하이브 컨텍스트에 대한 `clean`, `replayed`, `unchanged` 또는 `not_applied`. |
| `LogsApplied` | Juntei가 재생한 트랜잭션 로그 파일 이름. 보고서에 기록하십시오. |
| `Problems` | 손상, 누락된 셀, 유형 불일치, 해석되지 않은 목록/상위 항목 또는 기타 제한 사항. |

## 복구 상태의 해석 { #recovery-status-interpretation }

| 상태 | 의미 |
|---|---|
| `deleted_complete` | 삭제된 키/값이 알려진 경로/데이터 전체와 함께 복구되었습니다. 그래도 과거 상태이며, 활성 상태가 아닙니다. |
| `deleted_partial` | 경로, 값 목록, 값 또는 데이터의 일부가 덮어쓰였거나 읽을 수 없습니다. 재사용된 셀로 빈 부분을 채우지 마십시오. |
| `residual_value_only` | 충분한 키 컨텍스트 없이 값만 남았습니다. `KeyPath`가 없을 수 있습니다. |
| `transaction_log_recovered` | 상태가 재생 전/로그 버전에만 존재하며 현재의 할당된/빈 레코드에는 없습니다. |

## 조사 전략 { #investigation-strategy }

1. 알고 있다면 의미가 명확한 정확한 키부터 시작하고, 탐색에는 정규식/데이터 검색을 사용합니다.
2. 내용을 보기 전에 `TransactionLogStatus`, `AllocationStatus`, `RecoveredFrom`, `Problems`를 확인합니다.
3. 키의 문서화된 스키마에 따라 데이터를 해석합니다. `REG_BINARY`는 자기 서술적이지 않습니다.
4. 구성 정보를 런타임 증거(이벤트 로그, Prefetch, SRUM, 서비스/작업)와 상관 분석합니다.
5. 새롭거나 논란이 있는 필드는 원시 데이터를 보존합니다.

## 추가 정보 { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Maxim Suhanov의 레지스트리 형식 사양](https://github.com/msuhanov/regf)
- [yarp 레지스트리 파서](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [Eric Zimmerman의 Registry](https://github.com/EricZimmerman/Registry)
- [Microsoft: 레지스트리 값 유형](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
