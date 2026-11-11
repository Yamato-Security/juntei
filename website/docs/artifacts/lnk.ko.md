# LNK 바로 가기 { #lnk-shortcuts }

## LNK 파일이란 { #what-lnk-files-are }

Windows Shell Link(`.lnk`) 파일은 대상과 함께 실행 인수, 작업 디렉터리, 아이콘, 창 상태, 대상
메타데이터, 볼륨/네트워크 정보, 셸 네임스페이스 항목, 그리고 선택적인 분산 링크 추적(distributed
link tracking) 식별자를 기술합니다. Windows와 애플리케이션은 최근 항목(Recent Items), 바탕 화면,
시작 메뉴 및 기타 위치에 이 파일을 생성하며, 공격자가 악성 바로 가기를 직접 전달하기도 합니다.

Recent 폴더의 LNK는 더 이상 존재하지 않는 대상(이동식 드라이브나 네트워크 공유 포함)과의 사용자
상호 작용을 뒷받침할 수 있습니다. 임의의 LNK는 한 번도 클릭되지 않은 채 존재할 수도 있습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 해당 바로 가기는 기록된 대상 컨텍스트로
생성/업데이트되었습니다. 사용 여부를 뒷받침하는지는 그 위치와 주변 아티팩트에 따라 결정됩니다.

## 위치 및 수집 { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\*.lnk
%USERPROFILE%\Desktop\*.lnk
%APPDATA%\Microsoft\Windows\Start Menu\Programs\...
%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\...
```

`Recent`만이 아니라 사용자 프로필 전체와 수집된 아카이브/이메일 콘텐츠까지 검색하십시오. 위치가
달라지면 해석도 달라지므로 원본 파일 자체의 시간과 경로를 보존하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- 대상의 시간, 크기, MFT 참조 및 볼륨 데이터는 LNK에 저장된 스냅샷입니다. 바로 가기 파일 자체의
  메타데이터가 아니며 오래된 정보일 수 있습니다.
- `MachineID`, MAC 및 tracker 생성 시간은 분산 링크 추적 컨텍스트를 나타내며, 일반적으로 링크를
  생성/업데이트한 머신을 가리킵니다. 반드시 해당 링크를 발견한 머신인 것은 아닙니다.
- MAC처럼 보이는 UUID 노드 값은 가상화, 무작위화 또는 네트워크와 무관한 식별자에서 비롯될 수 있습니다.
- `Args`로 인해 무해한 실행 파일(`powershell.exe`, `rundll32.exe`, `mshta.exe`)이 위험해질 수 있습니다.
- ANSI 문자열을 해석하려면 원본 시스템의 코드 페이지가 필요합니다. 일본어 링크의 상당수는 `--code-page 932`를 사용하십시오.
- 바로 가기는 조작되거나 복사될 수 있습니다. Prefetch, Jump List, RecentDocs, 이벤트 로그 또는 기타
  근거 없이 해당 필드를 실행과 동일시하지 마십시오.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.lnk
  -f, --file <FILE>      Parse one shortcut
      --code-page <CP>   ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -d evidence/ --code-page 932 -o lnk-ja.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

## CSV 출력 필드 { #csv-output-fields }

| 필드 그룹 | 필드 및 해석 |
|---|---|
| 원본 | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: 바로 가기 경로와 호스트 파일 시스템의 시간입니다. 대상 시간과는 별개입니다. |
| 대상 시간 | `TargetCreated`, `TargetModified`, `TargetAccessed`: LNK에 기록된 스냅샷입니다. 유용한 과거 단서이지만 현재 대상의 메타데이터는 아닙니다. |
| 대상 기본 정보 | `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`: 기록된 대상/실행 컨텍스트입니다. 인수에 따라 의미가 달라질 수 있습니다. |
| 볼륨 | `DriveType`, `VolumeSerialNumber`, `VolumeLabel`: 고정/이동식/네트워크 컨텍스트를 식별하고 다른 바로 가기와 상관 분석합니다. |
| 경로 | `LocalPath`, `NetworkPath`, `CommonPath`, `TargetIDAbsolutePath`: 서로 보완적인 경로 표현입니다. 불일치 여부와 셸 네임스페이스 경로를 검토하십시오. |
| 실행 동작 | `Args`, `ShowWindow`, `HotKey`, `ShimLayer`, `DarwinId`: 셸이 대상을 실행하거나 구성하는 방식입니다. `Args`가 특히 중요합니다. |
| NTFS 식별 정보 | `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`: 링크 생성 시점의 대상 참조입니다. 두 값을 모두 `$MFT`와 대조하십시오. |
| 추적 | `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `VolumeDroid`, `FileDroid`, `VolumeDroidBirth`, `FileDroidBirth`: 분산 링크 추적 정보입니다. 상관 분석에 사용하되 이것만으로 행위 주체를 판단하지 마십시오. |
| 추가 블록 | `ExtraBlocksPresent`: 존재하는 선택적 구조의 목록입니다. 다른 어떤 필드가 뒷받침되는지 설명합니다. |
| 표시 정보 | `Name`, `IconLocation`, `IconIndex`, `IconEnvironment`, `EnvironmentVariables`: 표시 정보와 환경 변수가 확장된 대상 데이터입니다. 공격자는 오해를 유발하는 이름/아이콘을 사용할 수 있습니다. |
| 셸 폴더 | `KnownFolder`, `SpecialFolder`: 디코딩된 Windows 셸 폴더 식별자입니다. |
| 네트워크 | `NetworkDevice`, `NetworkProvider`: 네트워크 공유/장치 및 공급자 컨텍스트입니다. |
| 인코딩 | `ConsoleCodePage`: 존재하는 경우 기록된 콘솔 코드 페이지입니다. 파서의 `--code-page`와는 별개입니다. |
| 후행 데이터 | `AppendedBytes`: 논리적 끝 이후의 바이트입니다. 예상치 못한 데이터는 스머글링이나 손상된 파일을 나타낼 수 있습니다. |
| 검증 | `Problems`: 손상, 잘림, 일관성 없는 블록 또는 디코딩 제한 사항입니다. 검토 대상에 포함하십시오. |

JSON/JSONL은 여기에 더해 모든 target-ID 항목(종류, 이름, GUID, FAT 시간 및 MFT 참조), 모든
추가 데이터 블록, 속성 저장소(property store) 속성, 콘솔 설정 및 원시 구조 관계를 보존합니다.
의심스러운 바로 가기는 평면적인 CSV 행에만 의존하지 말고 이 출력을 사용하십시오.
중첩된 `TargetItems`의 시간은 오프셋이 저장되지 않은 FAT 셸 항목 시계에서 비롯됩니다.
레거시 `Z` 표기는 UTC임을 증명하지 않으며, `auto-parse --timezone`은 이 값을 그대로 보존합니다.
LNK 헤더의 대상 FILETIME과 tracker 시간은 변환 가능한 절대 타임스탬프로 유지됩니다.

## 조사 전략 { #investigation-strategy }

1. 대상 필드를 해석하기 전에 원본의 위치와 원본 시간을 확인하십시오.
2. 대상 경로, `Args`, `WorkingDirectory` 및 환경 블록으로부터 실제 실행 내용을 재구성하십시오.
3. 대상 MFT 참조, 볼륨 일련번호, 네트워크 경로, droid, MachineID 및 MAC을 기준으로 여러 증거를 피벗하십시오.
4. 실행 여부는 Prefetch와 이벤트 로그로, 사용자 상호 작용은 Jump Lists/RecentDocs로 확인하십시오.
5. 악성 첨부 파일의 경우 추가된 데이터를 보존하고 LNK 자체의 해시를 계산하여 분석하십시오.

## 추가 정보 { #further-details }

- [Microsoft Open Specifications: MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [Sootmark shell](https://github.com/Sootmark/shell)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
