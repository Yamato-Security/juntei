# ShimCache (AppCompatCache) { #shimcache-appcompatcache }

## ShimCache란 { #what-shimcache-is }

Windows의 애플리케이션 호환성 하위 시스템은 호환성/shim 동작을 평가하는 동안 실행 파일 경로와 메타데이터를
캐시합니다. 이 캐시는 파일의 이름이 바뀌거나, 제거되거나, 이동식 볼륨과 함께 연결이 끊긴 뒤에도 남아
있습니다. 그 구조와 사용 가능한 필드는 Windows XP부터 Windows 11까지 버전마다 다릅니다.

ShimCache는 과거 경로를 발견하는 데 가치가 있으며, 형식에 신뢰할 수 있는 실행 플래그가 있는 경우 실행을
추가로 뒷받침합니다. 캐시된 시간은 일반적으로 **파일의 마지막 수정 시간**이며, Windows가 파일을 확인하거나
실행한 시간이 아닙니다.

<span class="evidence-strength">가장 타당한 결론:</span> Windows가 호환성 처리를 위해 해당 경로를
확인했습니다. 엔트리만으로는 존재/확인의 증거이며, `Executed`는 지원되는 경우에만 사용하십시오.

## 위치와 수집 { #location-and-collection }

바이트는 `SYSTEM` 레지스트리 하이브에 있으며, 일반적으로 다음 위치에 있습니다.

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

`SYSTEM`, `SYSTEM.LOG1` 및 `SYSTEM.LOG2`를 수집하십시오. Juntei는 모든 제어 집합을 읽고 가능한 경우
`Select\Current`를 식별합니다.

!!! warning "가장 최근의 라이브 엔트리는 하이브에 없을 수 있습니다"
    Windows는 ShimCache 업데이트를 메모리에 보관했다가 종료/재부팅 시 플러시할 수 있습니다. 실행 중인
    시스템에서 복사한 하이브에는 가장 최근의 확인 기록이 들어 있지 않을 수 있습니다. 캐시의 레지스트리 마지막
    쓰기 시간은 캐시가 저장된 시점을 설명하는 데 도움이 되지만, 엔트리별 실행 시간은 아닙니다.

## 분석 시 주의 사항 { #analyst-cautions }

- 캐시 엔트리 자체는 **실행의 증명이 아닙니다**.
- `Executed`는 `Yes`, `No` 또는 `NA`일 수 있으며, 그 의미는 형식/버전에 따라 다릅니다.
- `LastModifiedTimeUTC`는 참조된 파일의 시간이며, 확인이나 실행의 시간이 아닙니다.
- 순서는 유용하지만 보편적으로 정확한 타임라인은 아닙니다. 제어 집합/슬랙에 걸쳐 중복이 발생합니다.
- `FromSlack` 엔트리는 활성 캐시 밖에서 복구된 것이므로 추가적인 주의가 필요합니다.
- 사용 가능한 로그가 없는 더티(dirty) `SYSTEM`에서는 현재 레코드가 누락될 수 있습니다. 복구 요약을
  확인하십시오.
- 발견 사항은 의심스러운 이름/경로 패턴을 나타낼 뿐 맬웨어 판정이 아닙니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shimcache -f SYSTEM -o shimcache.csv
juntei parse-registry shimcache -d evidence/ -t jsonl \
  | jq 'select(.Executed == true or (.Findings | length > 0))'
```

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `ControlSet` | 캐시가 들어 있는 제어 집합의 번호입니다. 하나를 선택하기 전에 `IsCurrent`를 비교하십시오. |
| `CacheEntryPosition` | 디코딩된 캐시 내 위치/순서입니다. 상대적인 순서 파악에 유용하지만 시간이 아닙니다. |
| `Path` | 캐시된 실행 파일/패키지 경로입니다. 주요 헌팅 피벗입니다. |
| `LastModifiedTimeUTC` | 엔트리에 저장된 파일 수정 시간이며, 실행/확인 시간이 아닙니다. |
| `Executed` | 지원되는 형식 플래그에 따라 `Yes`, `No` 또는 `NA`입니다. 버전 컨텍스트와 함께만 사용하십시오. |
| `Duplicate` | 같은 엔트리가 이전 제어 집합이나 슬랙에서 이미 확인되었음을 나타냅니다. 중복 집계를 방지하는 데 도움이 됩니다. |
| `SourceFile` | 입력 하이브/원시 값입니다. 여러 호스트를 다룰 때 필수적입니다. |
| `FromSlack` | 활성 목록이 아니라 캐시 값의 슬랙에서 복구된 엔트리입니다. 신뢰도가 낮고 더 오래된 것일 수 있습니다. |
| `CacheFormat` | 디코딩된 Windows/캐시 구조입니다. 필드의 가용성과 의미를 결정합니다. |
| `CacheLastWrittenUTC` | 레지스트리 키/캐시 저장 시간입니다. 공유 컨텍스트이며, 각 엔트리의 실행 시간이 아닙니다. |
| `InsertFlags`, `ShimFlags` | 원시 호환성 플래그입니다. `Executed`는 지원되는 경우 Juntei가 해석한 결과입니다. |
| `DataSize` | 이 엔트리 데이터의 크기로, 구조 검증에 유용합니다. |
| `Machine` | 최신 형식에 기록된 아키텍처(`x64`, `x86`, `ARM64`)입니다. |
| `FileSize`, `LastUpdateTimeUTC` | 존재하는 경우 Windows XP 고유의 캐시된 메타데이터입니다. |
| `PackageName`, `PackageVersion`, `PackagePublisherID` | 지원되는 엔트리의 패키지 애플리케이션 식별 정보입니다. 게시자 ID는 컨텍스트이며 서명의 증명이 아닙니다. |
| `Findings` | 위치가 바뀐 시스템 바이너리나 의심스러운 스테이징 경로 같은 Juntei 이상 징후 코드입니다. 단서로만 사용하십시오. |
| `RegistryPath`, `RegistryValueName` | 정확한 하이브 키/값 출처입니다. 검증을 위해 보존하십시오. |
| `CacheValueSize` | 레지스트리 값 전체의 크기이며, 개별 엔트리의 `DataSize`가 아닙니다. |
| `IsCurrent` | 이 엔트리의 제어 집합이 `SYSTEM\Select\Current`와 같은지 여부입니다. 확실히 알 수 없으면 비어 있습니다. |
| `SelectCurrent`, `SelectDefault`, `SelectLastKnownGood`, `SelectFailed` | 컨텍스트 및 복구 분석을 위한 원시 `SYSTEM\Select` 제어 집합 번호입니다. |

JSON/JSONL은 추가로 엔트리 데이터, 태그, 패키지 앱 하위 레코드, 구조화된 발견 사항, 캐시 문제 및 복구 세부
정보를 보존합니다.

## 조사 전략 { #investigation-strategy }

1. 현재 제어 집합을 우선하되, 다른 제어 집합과 슬랙도 과거 증거로 보존하십시오.
2. 경로와 발견 사항을 검색하십시오. `LastModifiedTimeUTC`를 실행 시간으로 사용하지 마십시오.
3. `Executed == Yes`인 경우 Prefetch, SRUM, 이벤트 로그 또는 기타 런타임 흔적으로 뒷받침하십시오.
4. 경로를 `$MFT`/USN과 매칭하여 생성, 삭제 및 메타데이터 이력을 확인하십시오.
5. 트랜잭션 로그가 재생되었는지, 그리고 라이브 캐시가 플러시되지 않았을 가능성이 있는지 보고하십시오.

## 추가 정보 { #further-details }

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [Eric Zimmerman의 AppCompatCacheParser](https://github.com/EricZimmerman/AppCompatCacheParser)
- [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant: 포렌식 조사에서 애플리케이션 호환성 캐시 활용하기](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Maxim Suhanov의 Windows 레지스트리 형식 사양](https://github.com/msuhanov/regf)
