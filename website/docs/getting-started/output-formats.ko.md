# 출력 형식 { #output-formats }

## 터미널 요약 { #terminal-summaries }

`-o`나 구조화된 `-t` 없이 실행하면, 명령어는 일반적으로 개수, 시간 범위, 손상, 고가치 발견 사항을 표시합니다.
요약은 대용량 출력을 생성하기 전에 범위를 파악하는 데 도움이 됩니다.

## CSV { #csv }

CSV는 기본 저장 형식입니다. 스프레드시트 도구, SQL 가져오기, 타임라인 뷰어에 편리합니다. 일부 파서는
아티팩트에 서로 다른 레코드 유형이 들어 있기 때문에 여러 테이블을 생성합니다(Amcache, Jump Lists, SRUM).

!!! warning "증거 열은 텍스트로 취급하십시오"
    경로나 명령줄은 `=`, `+`, `-` 또는 `@`로 시작할 수 있습니다. 스프레드시트 소프트웨어는 이를 수식으로
    해석할 수 있습니다. 모든 필드를 텍스트로 가져오거나 포렌식에 안전한 CSV 뷰어를 사용하십시오. Juntei는
    증거 문자열을 변경하지 않고 보존합니다.

## 필드 이름 { #field-names }

CSV 열 머리글과 JSON/JSONL 필드 이름은 Amcache 및 `auto-parse` 매니페스트를 포함하여 PascalCase를 사용합니다.
약어는 대문자로 유지됩니다: `ProcessID`, `TimestampUTC`, `UserSID`, `SHA256`. 약어 뒤에 오는 단어는 하이픈으로
구분합니다: `IP-Address`, `MFT-Entry`, `USN-Journal`. `jq`에서는 하이픈이 있는 필드를 따옴표로 묶으십시오:
`.["IP-Address"]`.

기본 MFT CSV는 `EntryNum`, `Ext`, `Dir`, `HasADS`, `Size`, `Created (SI)`, `Created (FN)`처럼 더 짧고 읽기
쉬운 머리글을 사용합니다. 전체 CSV 스키마는 [MFT 필드 참조](../artifacts/mft.md)를 참조하십시오. 타임스탬프
쌍은 MACB 순서(DataModified, Accessed, MetadataChanged, Created)를 따르며, SI가 FN보다 앞에 옵니다. CSV
도구에서는 공백이나 문장 부호가 포함된 머리글을 따옴표로 묶으십시오.

증거 값은 원래 철자를 유지합니다. `Details`/`DataRaw` 아래의 레지스트리 값 이름이나 `ZoneIdentifier` 아래의
`Zone.Identifier` 이름처럼 원본이 제공하는 딕셔너리 키도 변경되지 않습니다. SRUM의 `ValueState`와 `Raw`
딕셔너리는 데이터 열과 같은 레이블을 사용합니다. 정규화 충돌이 발생하면 필드를 조용히 버리지 않고 보고합니다.

이는 이전 내보내기 스키마를 변경합니다. 기존 결과를 다시 생성하고 저장된 쿼리를 업데이트하십시오(예를 들어
`.record_type`은 `.RecordType`이 되고, Amcache의 `full_path`는 `FullPath`가 됩니다). 사람이 읽는 텍스트
보고서와 입력/구성 형식은 기존 관례를 유지합니다.

## 축약된 필드 이름 { #abbreviated-field-names }

다음 레이블은 자동 분석과 매니페스트를 포함하여 CSV, JSON, JSONL 내보내기에 적용됩니다. 이전 결과를 다시
생성할 때는 저장된 `jq`, `xsv` 및 스프레드시트 필터를 업데이트하십시오. 출력 레이블만 변경되며, 증거 값과
원시 레지스트리 값 이름은 원래 철자를 유지합니다.

| 이전 레이블 | 현재 레이블 |
|---|---|
| `ADS` | `HasADS` |
| `Application` | `App` |
| `ApplicationCompany` | `AppCompany` |
| `ApplicationCompanyRaw` | `AppCompanyRaw` |
| `ApplicationPath` | `AppPath` |
| `Arguments` | `Args` |
| `Attributes` | `Attr` |
| `AttributeType` | `AttrType` |
| `BytesRecvd` | `BytesReceived` |
| `Computer` | `Comp` |
| `ComputerName` | `CompName` |
| `ComputerNameLastWrittenUTC` | `CompNameLastWrittenUTC` |
| `Directories` | `Dir` |
| `FileName` | `Filename` |
| `MicrosecondZeros` | `ZeroU-Secs` |
| `SI-BeforeFN` | `SI<FN` |
| `UserName` | `User` |
| `Username` | `User` |
| `UserNameSource` | `UserSource` |
| `Users` | `User` |

네이티브 시스템 정보 JSON 시간은 `CompNameLastWritten`(CSV: `CompNameLastWrittenUTC`)입니다. `HasADS`는 부울
표시기이며, Prefetch의 `Dir`에는 기록된 디렉터리 목록이 들어 있습니다. `User`는 아티팩트별 유형을 유지합니다.
계정 이름이면 문자열이고, USB 출력에서는 연관된 사용자 목록입니다. 이는 실행 주체를 의미하지 않습니다. 쿼리에서
문장 부호는 따옴표로 묶으십시오: `."SI<FN"` 및 `.["ZeroU-Secs"]`.

## JSON { #json }

JSON은 전체 결과를 하나의 문서로 저장합니다. 배열, 중첩 구조, null, 큰 정수, 그리고 평면 CSV로는 항상 표현할
수 없는 구별을 보존합니다.

```bash
juntei parse-file lnk -f sample.lnk -o lnk.json -t json
```

## JSON Lines { #json-lines }

JSONL은 한 줄에 하나의 독립적인 JSON 개체를 기록합니다. 스트리밍에 적합하며 `jq`와 자연스럽게 함께
동작합니다.

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.StartType == "automatic") | {Host, ServiceName, ImagePath}'
```

구조화된 출력이 표준 출력으로 보내질 때, Juntei는 진행 상황과 메시지를 표준 오류로 보내므로 파이프는 데이터만
받습니다.

`auto-parse -j timeline.jsonl`은 범주별 CSV 결과를 유지하면서 여러 호스트의 전체 수집물에 대한 추가 JSONL
데이터 세트를 생성합니다. 개별 결과 형식을 변경하려면 `-t json` 또는 `-t jsonl`을 추가하십시오. 각 레코드에는
`Comp`와 `Artifact` 필드가 있으므로 SIEM에 가져온 후에도 출처가 명확하게 유지됩니다.

## UTC와 정밀도 { #utc-and-precision }

Juntei는 형식이 허용하는 경우 RFC 3339를 사용하여 파싱된 시간을 UTC로 표시합니다. 어떤 아티팩트는 100나노초
정밀도를 가지며, 다른 아티팩트는 초, FAT 시간, 집계 간격을 저장하거나 시간을 전혀 저장하지 않습니다. 자릿수가
정확도를 의미하지는 않습니다. 항상 각 아티팩트 페이지에 설명된 원본 필드를 이해하십시오.

UTC는 Windows 이벤트 로그 형식인 `2026-10-20T08:46:38.2001668Z`를 사용하며, 사용 가능한 소수 정밀도를
유지합니다. `auto-parse -Z "+09:00"`은 모든 결과 형식과 선택적인 `-j` 내보내기에서 파싱된 시각을 명시적인 고정
오프셋으로 표시합니다. 기본값은 Juntei를 실행하는 컴퓨터와 관계없이 UTC입니다. 원시 값과 표준 시간대가 없는
벽시계 시간은 기록된 그대로 유지됩니다. [출력 표준 시간대](../commands/auto-parse.md#output-time-zone)를
참조하십시오.

## 손상과 누락된 값 { #damage-and-missing-values }

빈 값, `null`, `NA`, `Problems` 및 파서별 상태 필드는 의미가 있습니다. 이는 아티팩트가 값을 저장하지 않았거나,
레코드가 손상되었거나, 형식이 이를 지원하지 않거나, 파서가 이를 해석할 수 없었음을 의미할 수 있습니다. 모든
빈 값을 0으로 대체하지 마십시오.
