# NTFS 트랜잭션 로그 (`$LogFile`) { #ntfs-transaction-log-logfile }

## 개요와 중요성 { #what-it-is-and-why-it-matters }

NTFS는 Windows가 일반적으로 시스템 볼륨에 사용하는 파일 시스템입니다. NTFS는 충돌로부터 복구하기 위해
파일 시스템 메타데이터에 적용하려는 변경 사항을 `$LogFile`이라는 숨겨진 볼륨 파일에 기록합니다. 각
업데이트에는 **redo** 바이트(새 상태)와 **undo** 바이트(변경을 롤백하는 데 필요한 이전 상태)가 포함될 수
있습니다. 이는 Windows 이벤트 로그, 레지스트리 `.LOG1` 파일, [USN 저널](usn-journal.md)과는 별개입니다.

조사관에게는 이전 상태가 특히 유용합니다. undo 레코드에는 이름 변경 전의 파일 이름, 삭제된 디렉터리
엔트리, 또는 메타데이터 업데이트 이전의 타임스탬프가 보존되어 있을 수 있습니다. 일부 레코드에는 NTFS가
USN 저널에 기록한 바이트도 들어 있으며, 여기에는 수집된 `$J`에 더 이상 존재하지 않는 레코드도 포함됩니다.
다만 많은 트랜잭션은 운영 체제의 일상적인 활동과 관련된 것이며 공격을 나타내지 않습니다.

**예:** 제거된 인덱스 엔트리에 `report.txt`가 언급되고, 이후에 추가된 엔트리에
`report.txt.encrypted`가 언급됩니다. 완전한 파일 참조, 상위 항목, 트랜잭션 체인 및 다른 아티팩트를
매칭하면 이름 변경을 뒷받침할 수 있습니다. 이 두 이름만으로는 랜섬웨어, 그 운영자, 또는 정확한 암호화
시각을 식별할 수 없습니다.

## 위치와 수집 { #location-and-collection }

```text
C:\$LogFile
D:\$LogFile
```

모든 NTFS 볼륨에는 각자의 로그가 있습니다. NTFS 메타데이터를 읽는 도구를 사용하거나 포렌식 이미지 또는
스냅샷에서 내보내는 방식으로 획득하십시오. 일반적인 Windows 파일 복사로는 접근하지 못할 수 있습니다. 같은
볼륨의 `$MFT`와 `$Extend\$UsnJrnl:$J`를 가능한 한 가까운 시점에 수집하십시오. 원본 파일과 해시를
보존하십시오. 나중에 획득한 데이터에는 다른 구간의 활동이 들어 있을 수 있습니다.

`$LogFile`은 **순환 로그**이며, 새 레코드가 오래된 레코드를 덮어씁니다. 보존 기간은 로그 크기와 작업량에
따라 달라지며 몇 분이나 몇 시간에 불과할 수도 있습니다. 파일 이름이나 작업이 없다고 해서 그것이 발생하지
않았다는 증거가 되지는 않습니다.

## 레코드가 증명하는 것과 주의 사항 { #what-a-record-proves-and-cautions }

- 업데이트는 NTFS가 복구를 위해 로깅한 메타데이터를 기록합니다. 하나의 redo 엔트리만으로는 상위 수준의
  작업 전체가 완료되었음이나 특정 인물이 이를 시작했음을 증명하지 못합니다. 관련 레코드, 트랜잭션 완료
  여부, 그 결과로 나타난 파일 시스템 상태, 그리고 독립적인 증거를 상관 분석하십시오.
- **일반적인 트랜잭션 타임스탬프는 없습니다.** 포함된 타임스탬프는 파일 메타데이터를 나타내며, 오래되었거나
  복사되었거나 공격자가 수정한 값일 수 있습니다. `CreatedOn`을 이름 변경이나 타임스탬프 변경 시각으로
  사고 타임라인에 올리지 마십시오. USN 레코드가 별도의 타임스탬프를 제공할 수 있습니다.
- LSN(로그 시퀀스 번호)은 **하나의 볼륨 로그 내에서** 레코드의 순서를 정합니다. LSN은 시각이 아니며
  호스트나 볼륨 간에 비교해서는 안 됩니다. 순환 버퍼의 이전 회차 레코드가 더 새로운 활동과 함께 남아
  있을 수 있습니다.
- 트랜잭션 테이블 슬롯은 재사용됩니다. `TransactionId`만이 아니라 `TransactionFirstLsn`,
  `TransactionId`, 클라이언트 식별 정보 및 원본을 함께 사용하여 그룹화하십시오. 체인이 보존된 데이터
  바깥을 가리킬 수 있습니다.
- 파일 참조에는 엔트리 번호 **와 시퀀스 번호**가 포함됩니다. 그렇지 않으면 MFT 엔트리의 재사용으로 인해
  서로 무관한 파일이 같은 파일처럼 보일 수 있습니다.
- `$MFT`에서 제공하는 경로는 재구성된 과거 경로가 아니라 **수집된 MFT 상태**를 나타냅니다. Juntei는
  시퀀스 번호를 매칭하며, 알 수 없는 구성 요소는 명시적으로 남겨 둡니다. 상위 항목은 트랜잭션 이후에
  이름이 변경되었을 수 있으며, 하드 링크로 인해 현재 이름이 둘 이상일 수 있습니다.
- `TargetInferred`는 열린 속성(open attribute) 매핑이 로그의 다른 구간에서 얻어졌음을 의미합니다. 해당
  매핑은 더욱 주의해서 다루십시오. 해석되지 않은 인덱스 엔트리는 레이아웃만으로 디코딩될 수 있으므로,
  그 이름은 원시 바이트와 다른 출처로 검증하십시오.
- 찢어진 페이지(torn page)는 건너뛰며, 읽을 수 있는 주변 레코드는 계속 사용할 수 있습니다. `Truncated`는
  후속 부분이 누락된 레코드를 표시합니다. 출력을 완전한 것으로 취급하기 전에 파서 진단 메시지를
  검토하십시오. 길이가 0인 입력은 오류이며, 한 번도 기록되지 않은 모두 `FF`인 로그는 정상적으로 행이
  없습니다.
- 파서는 과거의 MFT를 재구성하기 위해 트랜잭션을 **재생하지 않습니다**. 임의의 비상주(non-resident) 파일
  콘텐츠를 복구하거나 수집된 데이터를 실행하지도 않습니다. DAX CRC 보호 페이지는 지원되지 않습니다. 실제
  증거를 사용한 검증은 주로 4 KiB 로그 페이지를 대상으로 합니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```bash
# Single log, current paths from an adjacent $MFT when present
juntei parse-ntfs logfile -f '$LogFile' -o logfile.csv

# Explicit MFT from the SAME volume
juntei parse-ntfs logfile -f '$LogFile' -m '$MFT' -t jsonl -o logfile.jsonl

# Multiple collected logs; SourceFile distinguishes every input
juntei parse-ntfs logfile -d evidence -o collected-logs.csv

# Preserve the underlying payloads for low-level validation
juntei parse-ntfs logfile -f '$LogFile' --raw-data -t jsonl -o logfile-raw.jsonl

# Automatic per-computer output
juntei auto-parse -d evidence -o parsed
```

모든 옵션은 [명령어 레퍼런스](../commands/filesystem.md#parse-ntfs-logfile)를 참조하십시오. 디렉터리
탐색은 대소문자를 구분하지 않고 `$LogFile`, `LogFile`, 그 `.bin` 형식, 그리고 `C_$LogFile`이나
`C_$LogFile.bin` 같은 이름을 인식합니다. `-f`로 전달하는 단일 파일은 이름에 제한이 없습니다.

자동 MFT 페어링은 내보내기의 볼륨 접두사를 따릅니다. `C_$LogFile`은 `C_$MFT`를 사용하고,
`D_$LogFile.bin`은 먼저 `D_$MFT.bin`을, 그다음 `D_$MFT`를 시도합니다. 접두사가 있는 로그는 접두사 없는
`$MFT`를 대체 수단으로 사용하지 않습니다. 접두사가 없는 `LogFile`은 `MFT` 또는 `$MFT`도 인식합니다.
내보내기의 명명 규칙이 다르다면 `-f`와 함께 `-m`을 사용하여 일치하는 MFT를 명시적으로 선택하십시오.

출력은 업데이트, 체크포인트 및 알 수 없는 클라이언트 레코드를 포함하여 LSN당 하나의 레코드입니다. JSON은
보기 좋게 서식화된 객체의 스트림이고, JSONL은 한 줄에 객체 하나입니다. CSV는 같은 필드 이름을 사용하며,
배열과 구조화된 필드는 CSV 셀 안에 압축된 JSON으로 들어갑니다. 알 수 없는 스칼라 값은 CSV에서는 비어
있고 JSON에서는 `null`입니다. 빈 배열은 디코딩된 항목이 없다는 뜻이며, 항목이 존재하지 않았다는 증명이
아닙니다.

한 번에 하나의 로그 전체와 그 파싱된 레코드가 메모리에 유지됩니다. MFT 경로를 해석하면 메모리 사용량과
작업량이 늘어납니다. `--raw-data`는 출력 크기를 크게 늘릴 수 있습니다. `auto-parse`에서 메모리가 제한된
머신이라면 `-T`를 사용하여 동시에 실행되는 파서 프로세스 수를 제한하십시오.

인접한 MFT를 읽을 수 없으면 Juntei는 경고를 표시하고 경로 없이 로그를 내보내며, 이때 `MftSource`는 null로
남습니다. 이러한 보강 실패로 읽을 수 있는 1차 증거가 버려지지는 않습니다. MFT 매칭이 없거나 모호한 경우에도
경고가 표시되고 요약에 `Logs without a matching MFT` 개수가 표시되지만, 파싱 실패로 집계되지는 않습니다.

## 출력 필드 { #output-fields }

### 레코드 식별, 무결성 및 출처 { #record-identity-integrity-and-provenance }

| 필드 | 의미와 해석 |
|---|---|
| `SourceFile`, `FileOffset` | 입력 경로와 이 레코드의 헤더를 읽은 바이트 오프셋입니다. 바이트를 검증하는 데 사용하십시오. |
| `LSN` | 레코드의 로그 시퀀스 번호입니다. 큰 값은 정수이며 타임스탬프가 아닙니다. |
| `PreviousLSN`, `UndoNextLSN` | 이전 레코드와 롤백 레코드로의 링크입니다. 0은 링크가 없음을 의미하며, 0이 아닌 링크는 보존된 데이터 바깥을 가리킬 수 있습니다. |
| `TransactionID`, `TransactionFirstLSN` | 트랜잭션 테이블 슬롯과 이 트랜잭션에 연결된 가장 이른 LSN입니다. 슬롯만으로는 고유하지 않습니다. |
| `RecordType` | `ClientRecord`(업데이트), `ClientRestart`(체크포인트) 또는 `Other`입니다. |
| `ClientIndex`, `ClientSequence` | 로그 클라이언트의 식별 정보로, 일반적으로 NTFS입니다. 사용자 계정이나 프로세스 ID가 아닙니다. |
| `Flags`, `SpansPages` | 원시 레코드 플래그와 클라이언트 데이터가 다른 페이지로 이어지는지 여부입니다. 페이지에 걸치는 것은 정상입니다. |
| `Truncated` | 파서가 이 LSN의 누락된 후속 부분을 발견한 경우 true입니다. 파일 전체의 모든 손상을 요약하는 것은 아닙니다. |
| `ClusterSize`, `ClusterSizeSource` | 대상 오프셋 계산에 사용된 클러스터당 바이트 수입니다. 출처는 `Caller`, `Checkpoint` 또는 `Inferred`입니다. 실제 값을 알고 있다면 `--cluster-size`로 지정하십시오. |
| `LogVersion` | 읽을 수 있는 최신 재시작 페이지의 버전으로, 일반적으로 `1.1` 또는 `2.0`입니다. 읽을 수 있는 재시작 페이지가 남아 있지 않으면 null입니다. |
| `MFT-Source` | 현재 경로 보강에 사용된 MFT입니다(있는 경우). 반드시 같은 볼륨의 것이어야 합니다. |

`auto-parse` JSON/JSONL에서는 다른 파서와 마찬가지로 `Comp`와 `Artifact`가 추가됩니다. 작업의 `.txt`
로그에는 진단 메시지가 기록되며, 매니페스트에는 작업의 성공, 실패 또는 건너뜀 여부가 기록됩니다. 복구
가능한 손상이 있으면 작업은 성공하되 로그에 0이 아닌 `Parse problems` 개수가 표시될 수 있습니다.

### 작업과 대상 { #operations-and-target }

| 필드 | 의미와 해석 |
|---|---|
| `RedoOperation`, `UndoOperation` | NTFS 작업 이름이며, 알 수 없는 코드는 16진수로 표시됩니다. redo는 의도된 업데이트를, undo는 그 역연산을 나타냅니다. |
| `TargetEntryNumber`, `TargetSequenceNumber` | 파일 레코드에 대한 작업인 경우: 영향을 받은 MFT 레코드 번호이며, 시퀀스 번호는 추론하지 않습니다. 그 밖의 경우: 해석된 열린 속성을 소유한 파일 참조입니다. 인라인 디렉터리 인덱스 업데이트의 대상은 디렉터리이며, 엔트리가 설명하는 하위 항목이 아닙니다. |
| `TargetAttributeType`, `TargetAttributeName` | 작업에 사용된 열린 속성의 숫자 유형과 이름입니다. 파일 레코드 작업에서는 일반적으로 `$MFT`의 `$DATA`입니다. 일반적인 유형: 16(`$STANDARD_INFORMATION`), 48(`$FILE_NAME`), 128(`$DATA`), 144(`$INDEX_ROOT`), 160(`$INDEX_ALLOCATION`), 176(`$BITMAP`). |
| `TargetPath` | 대상의 전체 참조를 알 수 있는 경우의 현재 MFT 경로, 또는 알 수 없는 구성 요소입니다. 과거 세대를 알 수 없는 파일 레코드 작업에서는 비어 있습니다. 레코드 번호만으로 현재 그 번호를 점유한 파일로 해석해서는 안 됩니다. |
| `OpenAttributeEntryNumber`, `OpenAttributeSequenceNumber`, `OpenAttributePath` | 작업이 기록할 때 거치는 열린 속성을 소유한 파일과 그 현재 MFT 경로입니다. 파일 레코드 작업에서는 영향을 받은 레코드와 별도로 `$MFT` 컨텍스트를 보존합니다. |
| `TargetAttribute` | 로그의 열린 속성 테이블 내 오프셋입니다. 속성의 유형이나 MFT 식별자와는 다릅니다. |
| `TargetInferred` | 대상 매핑이 로그의 다른 구간에서 얻어진 경우 true입니다. 이 플래그가 false여도 대상이 null이면 해석되지 않은 상태입니다. |
| `TargetVCN`, `Lcns` | 대상 내의 가상 클러스터와 물리 클러스터 번호입니다. 시간 분석보다는 저수준 검증에 유용합니다. |
| `RecordOffset`, `AttributeOffset` | 속성/엔트리의 오프셋과 그 안에서의 업데이트 오프셋입니다. |
| `ClusterBlockOffset`, `TargetBlockSize` | 512바이트 단위의 오프셋과 대상 크기입니다. |
| `OffsetInTarget` | 영향을 받은 파일 레코드 또는 비상주 스트림 내에서 계산된 바이트 오프셋입니다. 클러스터 및 MFT 레코드 크기에 따라 달라집니다. |
| `RedoLength`, `UndoLength` | 선언된 데이터 길이입니다. 0으로 채우는 작업은 그만큼의 페이로드 바이트 없이 길이를 선언할 수 있습니다. |
| `RedoBytesAvailable`, `UndoBytesAvailable` | 실제로 디코딩에 사용할 수 있는 페이로드 바이트입니다. 선언된 길이 및 작업의 의미와 비교하십시오. |

### 편리한 파일 메타데이터 열 { #convenient-file-metadata-columns }

이 열들은 redo 측에서 우선 선택한 DOS가 아닌 파일 이름 하나를 표시하며, 없으면 undo 측으로 대체합니다.
어느 쪽에도 파일 이름이 없으면 타임스탬프 업데이트 필드는 redo 타임스탬프를 우선하고, 그다음 undo를
사용합니다. 디렉터리 엔트리 업데이트도 시간, 크기 및 플래그를 직접 표시하며, undo보다 redo를 우선합니다.
디코딩된 `$DATA` 속성은 파일 이름이 없어도 크기를 표시합니다. 디코딩된 파일 레코드의 이름 없는 `$DATA`
크기는 오래되었을 수 있는 파일 이름 크기보다 우선합니다. 이 열들은 탐색을 돕기 위한 것이며, 변경 전후를
완전히 비교하려면 각 측의 개별 필드를 사용하십시오.

| 필드 | 의미와 해석 |
|---|---|
| `MetadataSide` | `Redo` 또는 `Undo`로, 편리한 파일 이름/타임스탬프 열을 제공하는 측을 식별합니다. |
| `EntryNumber`, `SequenceNumber` | 디코딩된 파일 이름/파일 레코드/속성 메타데이터 또는 부분적인 `$STANDARD_INFORMATION` 타임스탬프 업데이트가 설명하는 파일이며, 알 수 있는 경우 시퀀스도 포함됩니다. 하위 항목을 식별하지 않는 디렉터리 엔트리 업데이트와 원시 크기/런 리스트 업데이트에서는 비어 있습니다. 영향을 받은 레코드는 `TargetEntryNumber`를 사용하십시오. |
| `Filename` | 우선 선택된 디코딩된 이름입니다. DOS 별칭과 하드 링크를 포함한 모든 이름은 각 측의 배열에 남아 있습니다. |
| `ParentEntryNumber`, `ParentSequenceNumber`, `ParentPath` | 상위 참조와 현재 MFT 경로입니다. 과거 경로를 재구성한 것이 아닙니다. |
| `CreatedOn`, `ContentModifiedOn`, `RecordModifiedOn`, `LastAccessedOn` | 100 ns 단위의 UTC 메타데이터 시간입니다. 트랜잭션 타임스탬프가 아닙니다. 누락되었거나 유효하지 않은 시간은 비어 있거나 null로 남습니다. |
| `FileSize`, `FileAttributes` | 선택된 메타데이터의 크기와 숫자 Windows 플래그입니다. 디렉터리 엔트리 값은 오래되었을 수 있습니다. `$DATA` 속성과 디코딩된 파일 레코드가 더 직접적인 크기를 제공할 수 있습니다. |
| `AttrType` | 디코딩된 속성 레코드의 숫자 유형입니다. 디코딩된 디렉터리 엔트리 메타데이터 업데이트에서는 48입니다. 작업이 대상으로 하는 열린 속성과는 별개입니다. |

### 완전한 redo/undo 메타데이터와 페이로드 { #complete-redoundo-metadata-and-payloads }

| 필드 | 의미와 해석 |
|---|---|
| `RedoFileNames`, `UndoFileNames` | 해당 측에서 디코딩된 모든 `$FILE_NAME`의 배열로, 추가된/새 이름과 제거된/이전 이름을 구분하여 보존합니다. |
| `RedoTimes`, `UndoTimes` | 각 측이 기록한 `$STANDARD_INFORMATION` 타임스탬프입니다. 부분 업데이트에서는 변경되지 않은 타임스탬프를 추측하지 않고 비워 둡니다. |
| `RedoDecoded`, `UndoDecoded` | `Kind`로 식별되는 구조화된 파일 레코드, 속성, 인덱스 엔트리 또는 디렉터리 엔트리 정보입니다. null은 지원되지 않는 작업이거나 바이트가 부족함을 의미할 수 있습니다. |
| `JournalWriteOffset`, `JournalWriteLength` | `$DATA:$J`에 기록하는 redo 데이터의 오프셋과 바이트 수입니다. 이는 다른 방법으로는 손실되었을 USN 레코드의 출처가 될 수 있는 위치를 식별하며, 재구성된 저널을 내보내지는 않습니다. |
| `Checkpoint` | 체크포인트 버전, 시작 LSN, 클러스터 크기, 그리고 열린 속성, 속성 이름, 더티 페이지 및 트랜잭션 테이블 덤프의 LSN/길이입니다. |
| `OtherRecordType` | 인식되지 않거나 디코딩할 수 없는 본문의 원시 클라이언트 레코드 유형입니다. |
| `RedoData`, `UndoData`, `OtherData` | `--raw-data`를 지정한 경우에만 출력되는 16진수 페이로드이며, 그렇지 않으면 null/비어 있음입니다. 지원되지 않거나 손상된 구조를 검증하기 위해 보존하십시오. |

각 파일 이름 객체에는 파일 및 상위 참조(`Raw`, `EntryNumber`, `SequenceNumber`), `Name`,
`NameEscaped`, `Namespace`, 현재 `ParentPath`, 네 개의 타임스탬프, `Size`, `AllocatedSize`, `Attr`,
`ReparseValue`가 포함됩니다. `NameEscaped`는 이스케이프로 표현된 유효하지 않은 UTF-16을 식별합니다.
리파스 값은 리파스 태그이거나 확장 특성(extended attribute) 크기일 수 있습니다.

타임스탬프 객체는 `Utc`와 정수 `Filetime`을 모두 보존합니다. 센티널 값이나 유효하지 않은 값은 원래의
정수를 유지하면서 UTC 표현이 null일 수 있습니다. `RedoTimes`와 `UndoTimes`에는 `Created`, `Modified`,
`Changed`, `Accessed`가 포함되며, 누락된 멤버의 값은 null입니다.

디코딩된 `FileRecord` 객체에는 번호, 시퀀스, 하드 링크 수, 플래그, 기본 참조, 헤더만 남아 있는지 여부,
그리고 존재하는 속성이 포함됩니다. 디코딩된 각 속성은 유형, 이름, 상주 상태, 플래그, ID, 크기 및 할당
크기를 보존합니다. `$STANDARD_INFORMATION`에는 네 개의 시간, 속성, 소유자/보안 ID, 할당량 및 USN이
추가됩니다. 이 ID들은 별도로 해석해야 하며, 파일을 변경한 사용자의 증명이 아닙니다. 인덱스 엔트리에는
플래그, 키 길이 및 선택적인 파일 이름이 포함됩니다. 디렉터리 엔트리 업데이트는 이름 자체가 로깅되지 않은
경우에도 네 개의 시간, 크기, 속성 및 리파스 값을 보존합니다.

## 조사 워크플로 { #investigation-workflow }

1. 누락된 재시작 페이지, 찢어진 페이지 및 잘린 레코드에 대한 진단 메시지를 검토하십시오.
2. 의심스러운 파일 이름이나 확장자를 `RedoFileNames`와 `UndoFileNames` 양쪽에서 검색하십시오.
3. 엔트리 **및 시퀀스** 번호를 매칭한 다음, 트랜잭션의 LSN 체인과 작업을 조사하십시오.
4. redo와 undo 타임스탬프를 비교하여 변경 사항을 확인하십시오. 일상적인 복사, 압축 해제 및 복원도
   타임스탬프를 바꿀 수 있으므로, 차이는 단서일 뿐 자동으로 타임스톰핑이라고 판정할 근거가 아닙니다.
5. [USN 저널](usn-journal.md), [$MFT](mft.md), [$I30](i30.md)과 상관 분석하십시오. 실행이나 행위자를
   확정하려면 이벤트 로그, [Prefetch](prefetch.md) 및 사용자 활동 아티팩트를 사용하십시오.

예를 들어 어느 한쪽에 암호화된 파일 이름이 포함된 모든 레코드를 찾으려면 다음과 같이 합니다.

```bash
jq 'select(any(.RedoFileNames[]?, .UndoFileNames[]?;
  .Name | endswith(".encrypted"))) |
  {Lsn, TransactionId, TransactionFirstLsn, RedoOperation, UndoOperation,
   RedoFileNames, UndoFileNames}' logfile.jsonl
```

이는 파일 이름 검색이며 완전한 랜섬웨어 탐지기가 아닙니다. 저널 페이로드를 조사하려면 `--raw-data`로
내보낸 뒤 `JournalWriteOffset`이 null이 아닌 레코드를 선택하십시오. `RedoData` 16진수는 해당 오프셋에
기록된 바이트를 나타냅니다. 이렇게 포함된 USN 레코드를 파싱하고 중복을 제거하는 것은 별도의 단계입니다.
Juntei는 아직 이를 `parse-ntfs usn-journal` 출력과 병합하지 않습니다.

## 추가 정보 { #further-details }

- [ys-ntfs-parser의 logfile 크레이트](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/logfile): 지원되는 구조, 검증 코퍼스 및 알려진 제한 사항.
- [Sootmark ntfslog](https://github.com/Sootmark/ntfslog): 업스트림 Rust 구현.
- [libfsntfs NTFS 형식 문서](https://github.com/libyal/libfsntfs/tree/main/documentation): 로그 페이지, 픽스업 및 메타데이터 구조.
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs): 독립적인 NTFS 파싱 연구 및 구현.
- [Joakim Schicht의 LogFileParser](https://github.com/jschicht/LogFileParser): 상세한 트랜잭션 로그 분석.
- [Stroz Friedberg의 ntfs-linker](https://github.com/strozfriedberg/ntfs-linker): NTFS 출처들을 상관 분석하여 파일 기록으로 구성.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum): 실제 Windows NTFS 샘플.
- [Velocidex go-ntfs](https://github.com/Velocidex/go-ntfs): NTFS 구현 및 공개 테스트 볼륨.
