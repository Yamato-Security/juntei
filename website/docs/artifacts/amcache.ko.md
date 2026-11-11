# Amcache { #amcache }

## Amcache란 { #what-amcache-is }

Amcache는 레지스트리 하이브에 저장되는 Windows 애플리케이션 호환성 인벤토리입니다. 최신 시스템은
실행 파일을 `InventoryApplicationFile` 아래에, 플러그 앤 플레이(PnP) 장치를 `InventoryDevicePnp`
아래에 기록하며, 이전 Windows 버전은 레거시 `Root\File` 구조를 사용합니다.

실행 파일 조사에서 Amcache가 가치 있는 이유는 원본 파일이 사라진 뒤에도 전체 경로, SHA-1, 파일 크기,
PE 메타데이터, 버전 정보, 게시자 텍스트, 링크 날짜 및 레지스트리 타임스탬프를 보존할 수 있기
때문입니다. 장치 레코드로는 연결된 하드웨어, 드라이버, 설치 날짜 및 식별자를 확인할 수 있습니다.

<span class="evidence-strength">가장 타당한 결론:</span> Windows가 기록된 파일 또는 장치를
인벤토리에 등록했습니다. 실행 파일 레코드는 **존재의 증거이지 실행의 증명이 아닙니다**.

## 위치와 수집 { #location-and-collection }

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

하이브와 인접한 `Amcache.hve.LOG`, `.LOG1`, `.LOG2` 파일을 함께 수집하십시오. 하이브가 더티 상태이면 `parse-registry amcache`는 분석 전에 사용 가능한 트랜잭션 로그를 메모리에서 자동 재생하며 증거 파일은 변경하지 않습니다. 복구 세부 정보는 파서 로그나 터미널에 기록되고, 레코드를 stdout으로 출력할 때 진단 메시지는 stderr로 전송됩니다. 로그가 없거나 사용할 수 없으면 레코드가 불완전하거나 일관성이 없는 하이브를 읽지 못할 수 있습니다.

## 분석 시 주의 사항 { #analyst-cautions }

- Amcache의 실행 파일 엔트리는 사용자가 해당 파일을 실행했음을 증명하지 않습니다.
- `Publisher`는 Windows가 기록한 PE/버전 리소스 메타데이터입니다. 수집 시점에 파일에 유효한
  Authenticode 서명이 있었음을 증명하지 않습니다.
- 해시가 없을 수 있으며, 레거시 스키마와 최신 스키마가 동일한 값을 제공하지는 않습니다.
- `LinkDate`는 PE 헤더에서 가져온 값으로, 위조되거나 복사되거나 재현 가능한 빌드(reproducible
  build)에 의해 생성될 수 있습니다.
- 키의 마지막 쓰기 시간은 레지스트리 레코드의 업데이트를 나타내며, 반드시 다운로드, 생성, 최초 실행
  또는 마지막 실행 시간인 것은 아닙니다.
- 경로가 이동식 볼륨이나 더 이상 존재하지 않는 볼륨을 가리킬 수 있습니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.RecordType == "file" and .SHA1 == "0123456789abcdef0123456789abcdef01234567")'
```

CSV 출력에는 요약, 발견 사항, 파일, 장치 테이블이 포함됩니다. `auto-parse`를 사용하면 이 테이블은
각 컴퓨터의 `FileExistence/` 디렉터리 바로 아래에 `Registry-Amcache-Summary.csv`,
`Registry-Amcache-Findings.csv`, `Registry-Amcache-Files.csv`, `Registry-Amcache-Devices.csv`로
생성되며, 테이블 옆에 `Registry-Amcache-logs.txt`가 함께 생성됩니다. 하이브가 여러 개이면 확장자 앞에
`-Source1`, `-Source2` 등이 추가되며, 매니페스트의 `OutputFiles`에 정확한 경로가 나열됩니다. 단독
실행 CSV 내보내기는 고유한 테이블 이름과 요청한 출력 위치를 그대로 유지합니다. JSONL 레코드는
`RecordType` 값으로 `summary`, `finding`, `file`, `device`를 사용합니다.

## 파일 출력 필드 { #file-output-fields }

| 필드 | 분석가의 활용 방법 |
|---|---|
| `KeyName`, `SourceKeyPath` | 정확한 원본 레지스트리 레코드와 스키마 분기를 식별합니다. 재현성을 위해 보존하십시오. |
| `KeyLastWrittenUTC`, `KeyLastWrittenFiletime` | 사람이 읽기 쉬운 형태와 원시 형태의 레지스트리 레코드 업데이트 시간입니다. 실행 시간이라고 표기하지 마십시오. |
| `SourceVolumeKeyLastWrittenUTC`, `SourceVolumeKeyLastWrittenFiletime` | 레거시 스키마에서 상위 볼륨 키의 시간입니다. 컨텍스트 정보이며 파일 동작 시간이 아닙니다. |
| `FileID` | Amcache가 기록한 파일 식별자입니다. 그대로 유지되며, 항상 SHA-1이라고 가정하지 마십시오. |
| `SHA1`, `SHA256` | Defender, 위협 인텔리전스, EDR 및 파일 저장소로 피벗하기 좋은 강력한 기준입니다. null은 기록되지 않았음을 의미합니다. |
| `Name`, `OriginalFileName`, `Description` | 기록된 PE/버전 이름입니다. 실제 경로와 비교하여 위장(masquerading)을 탐지하십시오. 메타데이터는 공격자가 제어할 수 있습니다. |
| `FullPath` | 가장 유용한 위치 피벗입니다. 대소문자는 신중하게 정규화하되 원래 증거 문자열은 보존하십시오. |
| `LongPathHash` | Windows의 경로 관련 식별자입니다. 레코드 매칭에 유용하지만 내용 해시는 아닙니다. |
| `ProgramID` | 가능한 경우 파일을 애플리케이션 인벤토리 레코드와 연결합니다. |
| `Publisher` | 기록된 게시자 텍스트입니다. 서명 검증이 아니라 메타데이터로 취급하십시오. |
| `Version`, `BinFileVersion`, `ProductName`, `ProductVersion`, `BinProductVersion` | 버전 리소스 컨텍스트입니다. 값의 차이나 부자연스러운 값은 위장 분석을 뒷받침할 수 있습니다. |
| `BinaryType` | 기록된 아키텍처/유형 컨텍스트입니다. 파일이 복구된 경우 PE 분석 결과와 비교하십시오. |
| `LinkDate` | 기록된 PE 컴파일/링크 타임스탬프입니다. 회의적인 시각과 교차 검증이 있을 때만 유용합니다. |
| `Size` | 기록된 크기입니다. `$MFT`, Defender 콘텐츠 또는 복구된 파일과 비교하십시오. |
| `Language` | PE 언어 식별자입니다. 약한 컨텍스트 단서일 뿐이며 공격 주체 귀속(attribution)의 근거가 아닙니다. |
| `IsPE-File`, `IsOS-Component` | Windows가 기록한 분류 플래그입니다. false 값이 무해함을 증명하지는 않습니다. |
| `USN` | Amcache가 기록한 NTFS 업데이트 시퀀스 값입니다. 남아 있는 USN 레코드로 피벗하는 데 도움이 됩니다. |
| `APPX-PackageFullName`, `APPX-PackageRelativeID` | 패키지된 애플리케이션/구성 요소를 파일과 연결합니다. |
| `RegistryValues` | 유형, 디코딩된 값, 원시 16진수를 포함한 모든 원본 레지스트리 값입니다. 형식화된 필드가 없거나 향후/레거시 필드를 확인할 때 사용하십시오. |

## 장치 출력 필드 { #device-output-fields }

| 필드 그룹 | 필드와 해석 |
|---|---|
| 원본 | `KeyName`, `SourceKeyPath`, `KeyLastWrittenUTC`, `KeyLastWrittenFiletime`: 정확한 장치 레코드와 레지스트리 업데이트 시간입니다. |
| 식별 정보 | `Description`, `BusDescription`, `Manufacturer`, `Model`: 사용자에게 표시되는 장치 설명입니다. 공급업체 문자열은 일반적인 값일 수 있습니다. |
| 하드웨어 식별자 | `HWID`, `CompID`, `Class`, `ClassGUID`, `ContainerID`, `DeviceInterfaceClasses`: 장치 계열 및 인스턴스로 피벗하기 위한 가장 강력한 기준입니다. |
| 상태 | `DeviceState`, `ProblemCode`, `ParentID`, `MatchingID`, `Enumerator`: Windows 열거 및 관계 컨텍스트입니다. |
| 드라이버 | `DriverID`, `DriverName`, `DriverPackageStrongName`, `DriverVerDate`, `DriverVerVersion`, `Provider`, `Service`, `INF`: 드라이버 패키지 및 서비스 연관 정보입니다. 설치된 파일 및 서비스와 상관 분석하십시오. |
| 설치 | `InstallDate`, `FirstInstallDate`, `InstallState`, `ExtendedInfs`: 기록된 설치 수명 주기입니다. 날짜가 없다고 해서 장치가 한 번도 연결되지 않았다는 증명이 되지는 않습니다. |
| 필터 및 스택 | `LowerClassFilters`, `UpperClassFilters`, `LowerFilters`, `UpperFilters`, `StackID`, `LocationPaths`: 고급 드라이버 스택 컨텍스트입니다. 예상치 못한 필터 드라이버는 검토할 가치가 있습니다. |
| 원시 데이터 보존 | `RegistryValues`: 검증 및 지원되지 않는 필드를 위한 완전한 형식화/원시 원본 값입니다. |

CSV의 배열 및 원시 값 열에는 `JSON` 접미사가 붙으며(예: `HWID-JSON`, `RegistryValuesJSON`),
해당 셀에는 JSON이 들어 있습니다. JSON/JSONL에서는 기본 배열 형태가 그대로 유지됩니다.

## 조사 전략 { #investigation-strategy }

1. `FullPath`, `SHA1`, `Name`, `USN`을 기준으로 피벗하십시오.
2. 실행을 뒷받침하기 위해 일치하는 Prefetch를 찾으십시오.
3. `$MFT`와 USN 저널을 사용하여 파일 생성, 이름 변경, 삭제 및 크기를 확정하십시오.
4. 게시자/버전 메타데이터를 독립적으로 복구한 PE 및 서명 검증 결과와 비교하십시오.
5. 탐지 또는 제거 여부를 확인하기 위해 Defender 격리 데이터와 이벤트 로그를 확인하십시오.

## 추가 정보 { #further-details }

- [Juntei의 ys-amcache-parser 출력 스키마](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [Eric Zimmerman의 AmcacheParser](https://github.com/EricZimmerman/AmcacheParser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [Regipy Amcache 플러그인](https://github.com/mkorman90/regipy)
