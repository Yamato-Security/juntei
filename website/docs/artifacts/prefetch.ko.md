# Prefetch { #prefetch }

## Prefetch란 { #what-prefetch-is }

Windows Prefetch 파일은 애플리케이션 시작을 최적화하는 데 사용되는 정보를 기록합니다. 프로그램별 `.pf`
파일에는 일반적으로 실행 파일 이름과 해시, 실행 횟수, 최근 실행 시간, 볼륨 정보, 디렉터리, 그리고 시작 중에
참조된 파일이 저장됩니다.

지원되는 Windows 버전에서 유효한 애플리케이션 Prefetch 레코드는 보통 실행 파일이 실행되었다는 강력한
증거입니다. 이벤트 로그 이외의 실행 아티팩트 중 가장 유용한 것 중 하나입니다.

<span class="evidence-strength">가장 타당한 결론:</span> 운영 체제의 동작과 파서의 유효성을 전제로, 지정된
실행 파일이 기록된 최근 실행 시간에 시작되었습니다.

## 위치와 수집 { #location-and-collection }

```text
C:\Windows\Prefetch\*.pf
```

Juntei는 압축된 Windows 8 이상의 파일을 포함하여 형식 버전 17부터 31(Windows XP부터 Windows 11)까지
지원합니다. 실행 파일 이름 해시와 인접한 레코드를 비교할 수 있도록 디렉터리 전체를 수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- Windows는 이전 버전에서는 과거 실행 시간을 하나만, 최신 버전에서는 최대 8개까지 저장합니다.
  `RunCount`는 사용 가능한 타임스탬프 수를 초과할 수 있습니다.
- Prefetch는 일부 시스템/역할에서 비활성화되거나 정리되거나 제외되거나 사용할 수 없을 수 있습니다. 부재가
  프로그램이 실행되지 않았다는 증거는 아닙니다.
- 로드된 파일 목록은 시작 시의 종속성과 액세스를 나타냅니다. 나열된 모든 파일이 악성이었다거나, 의도적으로
  열렸다거나, 전부 읽혔다는 것을 증명하지는 않습니다.
- Prefetch 헤더의 실행 파일 이름은 길이가 제한되어 있습니다. Juntei는 SCCA 경로 해시로 이름이 같은 로드 경로를 구분합니다. 여러 후보를 구분할 수 없으면 헤더
  이름을 유지하고 PathAmbiguous를 설정하며, 로드된 파일 목록은 그대로 확인할 수 있습니다. 해시가 일치하지 않는 후보가 하나뿐이면 그 경로를 유지하고
  PathValidated를 false로 설정합니다. 호스팅 프로세스는 명령줄 데이터를 해시에 포함할 수 있으므로, 검증되지 않은 경로가 반드시 틀린 것은 아닙니다.
- `.pf` 원본 파일의 파일 시스템 타임스탬프는 포함된 실행 시간과 별개입니다.
- `Findings`는 헌팅을 위한 관찰 결과입니다. 설치 프로그램과 정상적인 관리 작업도 스테이징 경로를 자주
  사용합니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-file prefetch <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pf files
  -f, --file <FILE>      Parse one Prefetch file

Records:
  -T, --timeline  Write one row per retained execution time, sorted by time

Output:
  -o, --output <FILE>       Save every record
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file prefetch -d Prefetch/ -o prefetch.csv
juntei parse-file prefetch -d Prefetch/ -T -o prefetch-timeline.csv
juntei parse-file prefetch -d Prefetch/ -t jsonl \
  | jq 'select(.ExecutableName? | test("powershell|rundll32"; "i"))'
```

## 표준 CSV 필드 { #standard-csv-fields }

| 필드 | 해석 |
|---|---|
| `SourceFile` | 입력 `.pf`입니다. 호스트와 중복된 실행 파일 이름을 구별하기 위해 보존하십시오. |
| `ExecutableName` | 헤더의 실행 파일 이름입니다. 대부분의 헌팅에서는 대소문자를 구분하지 않지만, 원래 표기를 보존하십시오. |
| `ExecutablePath` | 가능한 경우 Juntei가 확인한 로드 경로입니다. `Files` 및 다른 아티팩트와 대조하여 확인하십시오. |
| `PathValidated` | 선택한 경로가 SCCA 경로 해시와 일치합니다. 경로 식별의 검증이며 파일 내용이나 신뢰성을 보장하지 않습니다. |
| `PathAmbiguous` | 여러 로드 경로 후보를 구분할 수 없어 `ExecutablePath`에 헤더 이름을 유지합니다. `Files`와 다른 아티팩트로 확인하세요. |
| `PrefetchHash` | Windows가 `.pf` 파일 이름에 사용하는 해시이며, 파일 콘텐츠의 암호화 해시가 아닙니다. |
| `FormatVersion` | Prefetch 구조 버전으로, 사용 가능한 필드와 타임스탬프 수를 결정합니다. |
| `RunCount` | Windows가 기록한 누적 카운터입니다. 빈도 파악에는 유용하지만 정확한 기록은 아닙니다. |
| `LastRun` | 보존된 가장 최근의 실행 시간입니다. |
| `PreviousRun1` … `PreviousRun7` | 보존된 이전 실행 시간으로, 최신 순입니다. 형식/기록에 없으면 비어 있습니다. |
| `VolumeCount` | 기록된 원본 볼륨 수입니다. |
| `VolumeDevicePaths` | 시작 중에 사용된 볼륨의 NT 장치 경로입니다. 일련번호와 상관 분석하십시오. |
| `VolumeSerialNumbers` | 볼륨 식별자로, 이동식 드라이브나 다시 포맷된 드라이브의 증거를 연결하는 데 유용합니다. |
| `VolumeCreated` | 기록된 볼륨 생성 시간이며, 실행 파일의 생성 시간이 아닙니다. |
| `DirectoryCount` | 기록된 디렉터리 문자열 수입니다. |
| `FileCount` | 기록된 로드 파일 참조 수입니다. |
| `Findings` | 위치가 변경된 시스템 바이너리나 의심스러운 실행 경로 같은 Juntei의 관찰 결과입니다. 직접 검토하십시오. |
| `Files` | 사용 가능한 NTFS 참조와 함께 로드/참조된 파일입니다. DLL/경로 피벗으로 매우 유용하지만, 파일에 대한 의도를 증명하지는 않습니다. |
| `Dir` | 시작과 연관된 디렉터리 문자열입니다. 개별 로드 파일 경로가 불완전할 때 유용합니다. |

## 타임라인 CSV 필드 (`-T`) { #timeline-csv-fields-t }

| 필드 | 해석 |
|---|---|
| `RunTime` | 보존된 하나의 Prefetch 실행 시간입니다. |
| `ExecutableName`, `ExecutablePath` | 프로그램 식별 정보/경로 컨텍스트입니다. |
| `PathValidated` | 선택한 경로가 SCCA 경로 해시와 일치합니다. 경로 식별의 검증이며 파일 내용이나 신뢰성을 보장하지 않습니다. |
| `PathAmbiguous` | 여러 로드 경로 후보를 구분할 수 없어 `ExecutablePath`에 헤더 이름을 유지합니다. `Files`와 다른 아티팩트로 확인하세요. |
| `PrefetchHash` | 실행 파일 이름은 같지만 변형/경로가 다른 것을 구분합니다. |
| `RunCount` | Prefetch 파일의 카운터이며, 각 타임라인 행에 반복됩니다. |
| `SourceFile` | 검증용 원본 `.pf`입니다. |

JSON/JSONL은 볼륨, 디렉터리, 로드된 파일 및 NTFS 파일 참조를 쉼표로 연결된 CSV 셀 대신 구조화된 배열로
보존합니다.

## 조사 전략 { #investigation-strategy }

1. `-T`를 사용하여 보존된 실행을 타임라인에 추가하십시오.
2. 실행 파일 이름과 Prefetch 해시를 함께 기준으로 그룹화하십시오. 같은 이름이라도 경로/해시가 다를 수
   있습니다.
3. `ExecutablePath`, 스테이징 경로, 위치가 변경된 시스템 바이너리에 대한 탐지 결과, 로드된 DLL을
   검토하십시오.
4. 대상 MFT 참조와 파일 경로를 `$MFT`/USN 레코드와 대조하십시오.
5. 실행 시간을 이벤트 로그, SRUM, 네트워크 텔레메트리, 서비스, 작업 및 사용자 아티팩트와 상관 분석하십시오.

## 추가 정보 { #further-details }

- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch)
- [libyal libscca](https://github.com/libyal/libscca)
- [Security Ronin prefetch-forensic](https://github.com/SecurityRonin/prefetch-forensic)
