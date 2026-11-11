# SRUM { #srum }

## SRUM이란 { #what-srum-is }

시스템 리소스 사용 모니터(SRUM)는 애플리케이션과 서비스의 리소스 사용량을 ESE 데이터베이스에 저장합니다.
Windows 버전에 따라 네트워크 바이트, 인터페이스/프로필 식별자, 연결, CPU 시간, 포그라운드/백그라운드 활동,
디스크 활동, 에너지 사용, 푸시 알림, 애플리케이션 타임라인 데이터를 기록합니다. 항목은 `SruDbIdMapTable`을
통해 애플리케이션 ID 및 사용자 SID와 연결됩니다.

SRUM은 실행 파일이 삭제된 경우에 특히 유용합니다. 애플리케이션 ID, 사용자, 리소스 사용 행이 파일보다 오래
남을 수 있기 때문입니다. 네트워크 합계는 데이터 유출 가설을 뒷받침할 수 있지만, SRUM은 패킷 캡처가 아니며
일반적으로 모든 원격 엔드포인트나 전송된 파일을 식별하지 않습니다.

<span class="evidence-strength">가장 타당한 결론:</span> Windows는 테이블의 측정 기간 동안 기록된 리소스
사용량을 해당 애플리케이션/계정에 귀속시켰습니다.

## 위치와 수집 { #location-and-collection }

```text
C:\Windows\System32\sru\SRUDB.dat
C:\Windows\System32\config\SOFTWARE   (context for account and Wi-Fi names)
```

SRUM은 Windows 8 이상에 존재합니다. `SRUDB.dat`는 일관된 상태로, 가능하면 스냅샷에서 또는 포렌식 수집 도구로
복사하십시오. `SOFTWARE`와 `sru` 디렉터리의 ESE 트랜잭션/로그 파일도 수집하십시오.

대응하는 SOFTWARE 하이브는 ESE 데이터베이스와 별개입니다. 하이브가 더티 상태이면 Juntei는 인접한 레지스트리 트랜잭션 로그 `SOFTWARE.LOG`, `.LOG1`, `.LOG2`를 메모리에서 자동 재생하므로 함께 수집하십시오. 복구 세부 정보와 경고가 표시되며, 로그가 없으면 계정 또는 Wi-Fi 이름이 불완전하거나 오래된 값일 수 있습니다. 이 처리는 SRUM 데이터베이스 로그를 재생하지 않습니다.

!!! warning "더티 데이터베이스"
    Juntei는 더티 상태의 `SRUDB.dat`를 수집된 그대로 읽고 이를 보고합니다. 현재는 `SRU*.log`를 재생하지
    않으므로, 해당 로그에만 존재하는 변경 사항은 누락됩니다. 더티 데이터베이스를 아무 언급 없이 완전한 것으로
    설명하지 마십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- 많은 행은 정확한 동작 시간이 아니라 주기적인 집계(일반적으로 정시 단위 경계 부근)를 나타냅니다.
- 바이트 합계만으로는 내용, 의도, 프로토콜 또는 대상을 식별할 수 없습니다.
- 애플리케이션 ID는 경로, 서비스, 패키지 앱 또는 개체일 수 있으므로 `AppType`을 확인하십시오.
- 사용자 이름과 Wi-Fi 이름은 일치하는 `SOFTWARE` 하이브로부터 보강한 정보입니다. 이름이 없다고 해서 기반이
  되는 ID가 없다는 뜻은 아닙니다.
- `null`, 0, 존재하지 않는 열, 지원되지 않는 유형, 손상된 값은 서로 다른 상태입니다.
- 공급자 스키마는 Windows 버전에 따라 다릅니다. 알 수 없는 테이블과 원래 열 이름을 보존하십시오.
- 백그라운드 Windows 서비스는 정상적으로도 큰 리소스 합계를 생성합니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-ese srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SRUDB.dat
  -f, --file <FILE>      Parse one SRUDB.dat
  -r, --software <FILE>  Matching SOFTWARE hive for names

Output:
  -o, --output <PATH>        Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>   csv, json, or jsonl
      --include-value-state  Add per-value read state and raw bytes to JSON/JSONL
```

```bash
juntei parse-ese srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-ese srum -f SRUDB.dat -t jsonl \
  | jq 'select(.Provider == "NetworkUsages" and (.BytesSent // 0) > 100000000)'
juntei parse-ese srum -f SRUDB.dat -t jsonl --include-value-state \
  | jq '{Provider, Timestamp, ValueState, Raw}'
```

## 출력 테이블 { #output-tables }

CSV는 `SruDbIdMapTable`에 대해 `SRUM-IdentityMap.csv`를, 공급자별로 하나의 테이블을 생성합니다. 알려진 출력
이름은 다음과 같습니다.

| 공급자/테이블 | 조사 목적 |
|---|---|
| `NetworkUsages` | 애플리케이션, 사용자, 인터페이스, 프로필별 송수신 바이트. |
| `NetworkConnections` | 연결 횟수/지속 시간 형태의 활동과 연결 시작 시간. |
| `AppResourceUseInfo` | 포그라운드/백그라운드 CPU, 디스크, 네트워크 및 관련 애플리케이션 사용량. |
| `AppTimelineProvider` | 종료 시간 및 사용 상태 필드를 포함한 애플리케이션 타임라인 활동. |
| `EnergyUsage` | 단기 및 장기 에너지/공급자 측정값. `IsLt`가 이를 구분합니다. |
| `PushNotifications` | 푸시 알림 활동. |
| `VfuProvider` | `vfuprov` 공급자. 구체적인 목적이 잘 문서화되어 있지 않으므로 필드를 신중하게 해석하십시오. |
| `EnergyEstimation`, `TaggedEnergy` | 버전별 에너지 및 사용량 공급자. |
| `ProcessorTime`, `SizeInBytes`, `TotalUsed`, `NetworkBytes` | Windows Server 공급자 테이블. |
| 알 수 없는 테이블 이름 | Juntei는 알 수 없는 공급자를 GUID/테이블 이름으로 보존하고 모든 카탈로그 열을 출력합니다. |

SRUM은 카탈로그 기반이며 Windows가 공급자 열을 추가하기 때문에, 공급자별 필드의 고정된 목록은 의도적으로
두지 않습니다. Juntei는 **데이터베이스에 존재하는 모든 열을 Juntei의 PascalCase 필드 레이블로** 출력합니다.
다음 필드는 안정적인 공통 출력 규약입니다.

## 범주별 자동 분석 출력 { #categorized-auto-parse-output }

`auto-parse`는 각 SRUM 테이블을 해당 조사 범주에 직접 배치합니다. CSV, JSON, JSONL은 같은 `SRUM-` 파일 이름
접두사를 사용하며, 이를 감싸는 `SRUM/` 디렉터리는 없습니다.

| 범주 | CSV 파일 |
|---|---|
| `NetworkActivity` | `SRUM-NetworkConnections.csv`, `SRUM-NetworkUsages.csv`, `SRUM-NetworkBytes.csv`, `SRUM-PushNotifications.csv` |
| `ProcessExecution` | `SRUM-AppResourceUseInfo.csv`, `SRUM-AppTimelineProvider.csv`, `SRUM-ProcessorTime.csv` |
| `SystemInformation` | `SRUM-EnergyUsage.csv`, `SRUM-EnergyEstimation.csv`, `SRUM-TaggedEnergy.csv`, `SRUM-SizeInBytes.csv`, `SRUM-TotalUsed.csv`, `SRUM-IdentityMap.csv`, `SRUM-VfuProvider.csv` 및 알 수 없는 공급자 테이블 |

수집물에 존재하는 테이블만 내보냅니다. 네트워크 카운터와 알림 레코드는 주된 조사 용도에 따라 배치된 것이며,
모든 행이 원격 연결을 설명한다는 의미는 아닙니다. ID 맵은 여러 범주를 지원합니다. VFU와 알 수 없는 공급자는
존재만으로 실행을 입증하지 않으므로 `SystemInformation`에 남습니다.

`SystemInformation/SRUM-logs.txt`에는 파서 진단과 원래 출력에서 최종 출력으로의 매핑이 들어 있습니다. 두
매니페스트 모두 `OutputFiles`에 실제 테이블 경로를 나열합니다. SRUM 작업의 `Output`은 컴퓨터 디렉터리이며,
`Category`는 공유 로그/메타데이터 위치를 나타냅니다.

데이터베이스가 여러 개이면 파일 이름의 확장자 앞에 `-Source1`, `-Source2` 등이 붙습니다. 로그는 원래 수집물
별칭을 보존하며, JSON/JSONL 행은 `SourceFile`, `Table`, `Provider`, `Comp`, `Artifact` 출처 정보를 유지합니다.
어떤 테이블도 여러 범주에 복사되지 않습니다. 결합된 `-j` SIEM 타임라인에는 보존된 모든 SRUM 레코드가 한 번씩
포함됩니다.

단독 실행하는 `parse-ese srum`은 요청된 출력 위치를 유지합니다. 그 CSV 테이블 파일에도 `SRUM-` 접두사가
붙으며, 명시적인 JSON/JSONL 출력 파일 이름은 분석가가 선택한 그대로입니다.

## 공통 출력 필드 { #common-output-fields }

| 필드 | 해석 |
|---|---|
| `SourceFile` | 입력 데이터베이스. JSON/JSONL에 있으며 여러 호스트를 다룰 때 유용합니다. |
| `Table` | 원래 ESE 테이블 이름/GUID. 버전이나 알 수 없는 공급자를 비교할 때 보존하십시오. |
| `Provider` | 알기 쉬운 공급자/출력 이름, 알 수 없는 경우 원래 테이블 이름. |
| `ID` | 행의 `AutoIncId`. 데이터베이스 행 식별자이며 프로세스 ID가 아닙니다. |
| `Timestamp` | SRUM이 수치를 기록/집계한 시간(밀리초 단위로 반올림). 대개 정확한 이벤트 시간이 아닙니다. |
| `EXE-Info` | `AppID`에서 해석한 애플리케이션/서비스/패키지 ID. 주요 프로그램 조사 기준점입니다. |
| `EXE-InfoDescription` | 복합 SRUM 애플리케이션 이름에서 추출한 설명. 컨텍스트로만 사용하십시오. |
| `EXE-Timestamp` | 존재하는 경우 복합 애플리케이션 ID에 포함된 시간. 행의 사용 시간이 아닙니다. |
| `AppType` | `NormalApp`, `Service`, `ModernApp` 또는 `Object`. `EXE-Info`를 해석하는 방식을 결정합니다. |
| `SID` | `UserID`에서 해석한 계정 SID. 강력한 사용자/서비스 식별 기준점입니다. |
| `User` | `SOFTWARE`로 보강한 알기 쉬운 프로필 이름. `SID`가 유효해도 null일 수 있습니다. |
| `UserID`, `AppID` | `SruDbIdMapTable`에 대한 원시 숫자 참조. 해석되지 않거나 중복된 이름을 위해 보존하십시오. |
| `IsLt` | 장기 EnergyUsage 테이블이면 `true`, 단기이면 `false`. |
| `InterfaceLUID` | 공급자 행의 원시 네트워크 인터페이스 LUID. |
| `InterfaceType` | 디코딩된 인터페이스 클래스(예: `IF_TYPE_IEEE80211`). 고유한 어댑터가 아니라 클래스입니다. |
| `L2ProfileID` | 숫자 네트워크 프로필 참조. |
| `ProfileName` | `SOFTWARE`로 보강한 Wi-Fi/프로필 이름. 민감할 수 있으며 현재 연결의 증거가 아닙니다. |
| `ConnectStartTime`, `EventTimestamp`, `EndTime`, `StartTime` | UTC로 디코딩된 알려진 공급자 FILETIME 열. 정확한 의미는 공급자에 따라 다르며, 보편적인 실행 시간이 아닙니다. |
| 공급자별 열 | `BytesSent`, `BytesReceived`(데이터베이스 열 `BytesRecvd`), CPU/디스크 카운터, 지속 시간, 횟수, 플래그, 추정치 등 내보낸 공급자 열. 단위는 공급자에 따라 해석하고, 같은 스키마 내의 행끼리 비교하십시오. |

## 진단 필드 (`--include-value-state`) { #diagnostic-fields-include-value-state }

| 필드 | 해석 |
|---|---|
| `ValueState` | 데이터베이스 필드에서 `parsed`, `null`, `absent`, `unsupported`, `damaged`로의 맵. 증거의 부재와 파서 실패를 구별합니다. |
| `Raw` | 바이트가 있는 값의 원래 바이트(16진수). 검증과 향후 디코딩에 사용하십시오. |

`SRUM-IdentityMap.csv`(`SruDbIdMapTable`에서 생성)에는 `IdIndex`, `IdType`, `IdBlob`이 들어 있습니다. 이는
공급자 행에서 사용하는 원시 인덱스, ID 범주, 디코딩된 프로그램/SID/개체 텍스트입니다.

## 조사 전략 { #investigation-strategy }

1. `EXE-Info`, `AppType`, `AppID`로 프로그램을 식별합니다. `Id`를 PID와 혼동하지 마십시오.
2. 네트워크 행을 프로그램, 사용자, 프로필, 인터페이스, 시간 간격별로 그룹화합니다.
3. 송수신 바이트를 정상 기준선 및 다른 호스트와 비교합니다.
4. 의심스러운 시간 간격을 Prefetch, 이벤트 로그, DNS/프록시/방화벽 데이터, 파일 접근과 상관 분석합니다.
5. 더티 데이터베이스 상태와 트랜잭션 로그 재생 누락을 수집상의 제한 사항으로 보고합니다.

## 추가 정보 { #further-details }

- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [dissect.esedb](https://github.com/fox-it/dissect.esedb)
- [libyal libesedb](https://github.com/libyal/libesedb)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
