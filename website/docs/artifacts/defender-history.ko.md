# Microsoft Defender 탐지 기록 { #microsoft-defender-detection-history }

## 기록에 저장되는 내용 { #what-the-history-records }

Microsoft Defender는 탐지마다 탐지 기록 파일을 하나씩 기록합니다. 각 파일에는 다음 내용이 기록됩니다.

- 위협의 식별자, 이름, 심각도 및 범주
- Defender가 수행한 조치와 그 성공 여부
- 위협이 실행 중이었는지 여부
- 관련된 사용자와 프로세스, 그리고 무엇이 위협을 탐지했는지
- 격리 엔트리(있는 경우)
- 위협이 발견된 모든 리소스: 파일, 아카이브 멤버, 다운로드, 프로세스, 레지스트리 값, 서비스, 명령줄 및 동작

Defender는 격리하지 않은 위협에 대해서도 이 파일을 기록합니다. 탐지만 한 위협, 허용된 위협, 제거에 실패한 위협이 여기에 해당합니다. Windows 보안의 보호 기록과 WMI의 `MSFT_MpThreatDetection` 클래스(`Get-MpThreatDetection`)가 이 레코드를 보여 줍니다. Orange Cyberdefense가 파일의 값을 변경하자 WMI는 변경된 값을 보고했습니다.

<span class="evidence-strength">가장 타당한 결론:</span> Defender가 나열된 리소스에서 지정된 위협을 탐지했다고 기록된 상태 및 시간과 함께 기록했습니다. 위협 이름은 제품의 판정입니다. 탐지 레코드만으로는 위협이 실행되었다는 사실도, 완전히 제거되었다는 사실도 보여 주지 못합니다.

## 위치와 수집 { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

각 파일의 이름은 해당 탐지의 `DetectionID`입니다. `DetectionHistory` 폴더 전체를 수집하십시오. 디렉터리 입력과 `auto-parse`는 `DetectionHistory` 폴더 안에서 깊이와 관계없이 탐지 기록 파일처럼 시작하는 모든 파일을 읽습니다. 그러한 폴더 밖에 보관된 사본에는 `-f`를 사용하십시오. Defender는 설정 가능한 일수가 지나면 검사 기록에서 항목을 제거하므로(`Set-MpPreference`의 `ScanPurgeItemsAfterDelay`) 오래된 탐지는 이미 사라졌을 수 있습니다.

격리 데이터는 별도의 아티팩트입니다. [Defender 격리 데이터](defender-quarantine.md)를 참조하십시오. 탐지의 `QuarantineID`는 격리 출력의 `EntryID`와 같은 표기(대문자, 중괄호 없음)를 사용하므로 두 값을 조인할 수 있습니다.

## 분석 시 주의 사항 { #analyst-cautions }

- 탐지, 치료 시도, 성공적인 제거는 서로 다른 상태입니다. `ThreatStatus`(`Detected`, `Quarantined`, `Removed`, `Allowed`, `RemoveFailed`, …)를 `ThreatStatusErrorCode`와 함께 읽으십시오. 후자는 마지막 상태 변경이 성공했을 때 0입니다.
- `ExecutionStatus`는 탐지 시점의 Defender 관점입니다(`Blocked`, `Allowed`, `Executing`, `NotExecuting`, `Unknown`). 위협이 실행되었다는 증거도, 실행된 적이 없다는 증거도 아닙니다. Prefetch, Amcache, 이벤트 로그 및 프로세스 원격 분석으로 확인하십시오.
- `ThreatName`은 당시의 시그니처 이름입니다. 오탐이나 이후의 재분류가 있을 수 있습니다. EICAR 같은 테스트 파일도 실제 위협과 똑같이 보고됩니다.
- `User`는 Defender가 기록한 `DomainUser` 값입니다(`UserSource`는 `DomainUser`). 비어 있거나, 서비스 계정이거나, 파일을 생성한 사용자와 다른 사용자일 수 있습니다.
- Defender는 하나의 위협에 대해 여러 파일을 기록할 수 있으며, Juntei는 그 파일을 모두 유지합니다. 복사되거나 복원된 파일처럼 두 파일이 같은 `DetectionID`를 가질 수도 있습니다. `DetectionID`와 `SourceFile`을 함께 기준으로 그룹화하십시오.
- 의미를 알 수 없는 값은 해석하지 않고 그대로 보존합니다. 해당 값의 섹션과 위치는 `UnknownValues`에 있습니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

기본 파일 형식은 CSV입니다. `-t csv`, `-t json` 또는 `-t jsonl`을 지정하고 `-o`를 생략하면 레코드를 stdout으로 출력합니다. 두 옵션 모두 없으면 요약만 표시합니다. 기존 출력을 덮어쓰려면 `-C`를 추가하십시오. `-C`를 지정하더라도 출력이 입력 파일 위에 덮어쓰이는 일은 없습니다.

Juntei는 각 탐지의 리소스마다 레코드를 하나씩 기록합니다. 리소스가 없는 탐지는 리소스 필드가 비어 있는 레코드 하나가 됩니다. 한 탐지의 레코드는 해당 `DetectionID`를 공유하며, 저장된 순서대로 `ResourceIndex`로 번호가 매겨집니다.

손상이 있어도 실행은 중단되지 않습니다. 중간에 잘린 파일은 손상 지점 전까지 읽은 값을 유지하며, `Problems`에 무엇이 누락되었는지 표시됩니다. `DetectionHistory` 폴더에 있지만 탐지 기록 파일이 아닌 파일은 보고한 뒤 건너뜁니다. 다른 파일은 계속 읽습니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Record` | `resource`이며, 리소스가 없는 탐지는 `detection`입니다. |
| `InitialDetectionTime` | Defender가 위협을 처음 탐지한 시간(UTC)입니다. 파일에 0이 들어 있으면 비어 있으며, 시간이 아닌 값은 `UnknownValues`에 보존됩니다. |
| `LastThreatStatusChangeTime` | 위협의 상태가 마지막으로 변경된 시간(UTC)입니다. |
| `RemediationTime` | Defender가 위협을 치료한 시간(UTC)입니다. 시도한 시간이며 제거되었다는 증거가 아닙니다. `ThreatStatus`를 확인하십시오. |
| `ThreatName`, `ThreatID` | Defender의 시그니처 이름과 숫자 식별자(`Get-MpThreatCatalog`)입니다. |
| `Severity`, `SeverityID` | `Low`, `Moderate`, `High`, `Severe` 또는 `Unknown`과 저장된 숫자입니다. |
| `Category`, `CategoryID` | `TROJAN`, `TOOL`, `BEHAVIOR` 같은 위협 범주와 저장된 숫자입니다. |
| `ThreatStatus`, `ThreatStatusID` | 위협에 대한 Defender의 상태입니다. 문서화된 이름이 없는 숫자는 `ThreatStatus`를 비운 채 보존됩니다. |
| `ThreatStatusErrorCode` | 마지막 상태 변경의 오류 코드입니다. 0은 성공을 의미합니다. |
| `ExecutionStatus`, `ExecutionStatusID` | 탐지 당시 Defender가 위협이 실행 중인 것을 보았는지 여부입니다. |
| `DetectionSource`, `DetectionSourceTypeID` | 무엇이 탐지했는지를 나타냅니다. `Real-time`, `IOAV`(다운로드 및 첨부 파일), `User`, `System`, `AMSI` 등이 있습니다. |
| `User`, `UserSource` | Defender가 기록한 `DomainUser` 값과 그 근거(`DomainUser`)입니다. 파일에 사용자가 지정되어 있지 않으면 비어 있습니다. |
| `Process` | 관련된 프로세스(`ProcessName`)입니다. `Unknown`일 수 있습니다. |
| `ResourceIndex`, `ResourceCount` | 이 리소스의 위치와 해당 탐지에 포함된 리소스 수입니다. |
| `ResourceType` | Defender가 부르는 이름 그대로의 `file`, `containerfile`, `webfile`, `process`, `regkeyvalue`, `service`, `CmdLine`, `behavior`, `amsi` 등입니다. |
| `Path` | 리소스 자체로, 경로, 아카이브 멤버(`archive.zip->member`), 레지스트리 값, 명령줄 또는 프로세스입니다. `webfile`은 URL과 다운로드한 프로세스(`pid:…,ProcessStart:…`)도 나타냅니다. 철자를 정확히 그대로 보존하십시오. |
| `ResourceFlags` | 16진수 리소스 플래그입니다. 그 의미는 문서화되어 있지 않습니다. |
| `TrackingStartTime` | 리소스의 `ThreatTrackingStartTime`으로, plaso가 탐지의 날짜를 정하는 방식과 같이 FILETIME(UTC)으로 읽은 정수입니다. |
| `SHA256`, `SHA1`, `MD5`, `TrackingSize` | 리소스의 추적 데이터에 있는 해시와 크기입니다. Defender가 검사했을 때의 파일을 나타냅니다. |
| `TrackingID` | 리소스의 `ThreatTrackingId`입니다. |
| `OtherTracking` | 그 밖의 모든 추적 값을 저장된 순서대로 `Key=Value` 형식으로 나타냅니다. 예를 들어 동작 탐지의 `ThreatTrackingScanSource` 또는 `ThreatTrackingBMContextRich`가 있습니다. |
| `QuarantineID` | 격리 엔트리(`Quarantine\Entries\{GUID}`)입니다. 파일에 0으로 채워진 GUID가 들어 있으면 비어 있습니다. |
| `DetectionID` | 탐지의 GUID이며 파일 이름이기도 합니다. 불일치는 `Problems`에 보고됩니다. |
| `MagicVersion` | 헤더의 `Magic.Version:1.2`입니다. 일부 Windows Server 2016 파일처럼 이 값 없이 기록된 파일에서는 비어 있습니다. |
| `UnknownValues` | 의미를 알 수 없는 값을 `position=value` 형식(예: `6.13=NT AUTHORITY\SYSTEM`)으로 나타내며, 상태 뒤에 오는 개수 지정 목록은 `2.list`로 나타냅니다. |
| `SourceFile` | 읽은 파일로, 입력 디렉터리 기준 상대 경로이며 `-f`에서는 파일 이름입니다. |
| `Problems` | 손상, 누락된 값, 예상치 못한 타입 또는 일치하지 않는 파일 이름입니다. 숨기지 마십시오. |

JSON과 JSONL은 같은 필드를 타입이 지정된 값으로 유지합니다. 숫자는 숫자 그대로이고 누락된 값은 `null`입니다. `OtherTracking` 대신 `Tracking`이 있으며, 여기에는 모든 추적 값이 Defender의 키, 타입, 값과 함께 들어 있습니다. `UnknownValues`는 타입이 포함된 위치 목록이 됩니다. `CountedList`에는 개수 지정 목록이 들어 있습니다. 저장된 FILETIME은 해석된 시간 옆에 원래의 틱 수를 유지합니다.

## 타임라인 동작 { #timeline-behavior }

`auto-parse`는 레코드를 `SecurityProducts/DefenderHistory.csv`(또는 `.json`/`.jsonl`)로 저장하고 결합된 `-j` 타임라인에 추가합니다. `super-timeline`은 각 리소스 레코드의 `InitialDetectionTime`, `LastThreatStatusChangeTime`, `RemediationTime`으로 이벤트를 만들고, 상태, 오류 코드, 실행 상태 및 탐지 소스를 이벤트 세부 정보에 넣습니다. `TrackingStartTime`은 정수로 저장되므로 `--include-interpreted-times`를 지정한 경우에만 이벤트가 됩니다. 파일 자체의 시간은 절대 사용하지 않습니다. 사용할 수 있는 시간이 없는 레코드는 파싱된 출력에 남으며 시간이 없는 레코드로 집계됩니다.

## 조사 전략 { #investigation-strategy }

1. `ThreatName`, `Path`, `SHA256`, `InitialDetectionTime` 및 `ThreatStatus`에서 시작하십시오.
2. 위협이 제거되었다고 결론 내리기 전에 `ThreatStatusErrorCode`와 `Problems`를 확인하십시오.
3. `QuarantineID`를 격리 데이터의 `EntryID`와 조인하여 격리된 바이트를 복구하십시오.
4. 해시와 경로를 기준으로 `$MFT`, USN 저널, Prefetch, Amcache, 브라우저 기록 및 이벤트 로그(Defender 운영 로그: 이벤트 ID 1116, 1117, 1118, 1119)로 피벗하십시오.
5. `webfile` 리소스는 URL과 프로세스 시작 정보를 사용하여 초기 침투를 조사하되, 브라우저, 프록시, DNS 증거로 검증하십시오.

## 테스트한 버전 { #tested-versions }

테스트에는 다른 도구가 독립적으로 읽은 결과가 있는 공개 파일을 사용합니다.

- Orange Cyberdefense의 Windows 10 샘플(Defender 4.18.24090.11): WMI가 보고한 내용과 대조했습니다.
- plaso의 Windows 테스트 파일 2개: plaso의 `windefender_history` 파서와 대조했습니다.
- DFIRArtifactMuseum의 APT 시뮬레이터 VM에서 얻은 파일 14개.
- 일부 Windows Server 2016 파일처럼 헤더의 버전 없이 기록된 사본.
- 중간에 잘린 파일, 빈 파일 또는 다른 종류의 파일.

## 추가 정보 { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [Orange Cyberdefense wddh-parser](https://github.com/cert-orangecyberdefense/wddh-parser)
- [plaso Defender 기록 파서](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Velociraptor Windows.Applications.DefenderHistory](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft: MSFT_MpThreatDetection 클래스](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft: Microsoft Defender 바이러스 백신 이벤트 ID](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
