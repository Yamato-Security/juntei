# Jump Lists { #jump-lists }

## Jump Lists란 { #what-jump-lists-are }

Jump Lists는 작업 표시줄과 시작 메뉴의 애플리케이션 항목에서 표시되는 메뉴의 기반이 됩니다. 자동(Automatic)
Jump Lists는 애플리케이션이 사용한 항목에 대해 Windows가 관리하는 기록입니다. 사용자 지정(Custom) Jump
Lists는 애플리케이션이 정의한 범주와 작업입니다. 둘 다 대상 경로, 시간, 볼륨/네트워크 컨텍스트, MFT 참조
및 실행 인수를 담은 LNK 구조를 포함할 수 있습니다.

자동 레코드는 애플리케이션이 연 파일이나 폴더, 마지막 사용 순서/시간, 상호 작용 횟수 및 고정 상태를
드러낼 수 있습니다. 사용자 지정 목록은 "최근에 닫은" 페이지나 애플리케이션 작업(악성 명령 인수 포함)을
드러낼 수 있습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 해당 애플리케이션이 그 항목/작업을 Jump List에
기록하거나 표시했습니다. 동작은 애플리케이션마다 다르므로 실제 실행/액세스는 교차 검증하십시오.

## 위치 및 수집 { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

16진수 파일 이름 접두사는 AppID입니다. 두 디렉터리를 모두 수집하고 사용자 프로필 경로를 보존하십시오.
자동 목록은 `DestList` 스트림과 번호가 매겨진 LNK 스트림을 가진 OLE Compound File 컨테이너이며, 사용자
지정 목록은 고유한 범주/링크 형식을 사용합니다.

## 분석 시 주의 사항 { #analyst-cautions }

- AppID 설명은 알려진 매핑에서 가져옵니다. 알 수 없는 AppID 자체는 의심스러운 것이 아닙니다.
- `LastModified` / 마지막 사용 필드는 DestList 항목에 속하며, 포함된 대상 시간과는 다릅니다.
- 상호 작용 횟수는 지원되는 버전에만 존재하며, 열기 전체 횟수가 아닙니다.
- 고정하면 일반적인 최근 사용 범위를 넘어 항목이 보존됩니다. 정리 작업이나 애플리케이션 설정으로 다른
  항목이 제거될 수 있습니다.
- 목록에는 오래되었거나 연결되지 않은 스트림, 또는 이전의 더 긴 사용자 지정 목록에서 남은 바이트가 포함될
  수 있습니다.
- 포함된 LNK 인수와 경로는 공격자가 제어할 수 있습니다. 애플리케이션과의 연관은 컨텍스트일 뿐 신뢰의
  근거가 아닙니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-list-csv
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

`auto-parse`를 사용하면 CSV 테이블은 각 컴퓨터 바로 아래의
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv`와
`FileAndFolderOpening/JumpLists-CustomDestinations.csv`입니다. 공유 로그는 해당 범주의
`JumpLists-logs.txt`입니다. 매니페스트의 `OutputFiles`에 정확한 경로가 나열됩니다. 단독 CSV 내보내기는
선택한 디렉터리에 `AutomaticDestinations.csv`와 `CustomDestinations.csv`를 유지합니다. JSON 및 JSONL
자동 내보내기는 단일 `JumpLists.json` 또는 `JumpLists.jsonl` 파일이며, 레코드 구조는 동일합니다.

## AutomaticDestinations CSV 필드 { #automaticdestinations-csv-fields }

| 필드 그룹 | 필드 및 해석 |
|---|---|
| 원본 | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: Jump List 컨테이너와 그 파일 시스템 시간입니다. |
| 애플리케이션 | `AppId`, `AppIdDescription`: 파일 이름의 AppID와 알려진 애플리케이션 매핑입니다. 알 수 없는 해시로부터 애플리케이션을 추정하지 마십시오. |
| 컨테이너 | `HasSps`, `DestListVersion`, `LastUsedEntryNumber`: DestList 구조/버전과 헤더 컨텍스트입니다. |
| 순서 | `MRU`, `EntryNumber`: 최근 사용 위치와 16진수 항목/스트림 식별자입니다. |
| 사용 메타데이터 | `CreationTime`, `LastModified`, `InteractionCount`, `PinStatus`, `PinPosition`: DestList의 수명 주기/사용/고정 상태입니다. 버전에 따라 해석하십시오. |
| 호스트/경로 | `Hostname`, `MacAddress`, `Path`, `DisplayPath`: 대상 위치와 추적 컨텍스트입니다. 호스트/MAC만으로 행위 주체를 판단하지 마십시오. |
| 추적 ID | `FileBirthDroid`, `FileDroid`, `VolumeBirthDroid`, `VolumeDroid`: 이동된 대상과 포함된 LNK 추적 정보를 상관 분석합니다. |
| 비고 | `Notes`: 연결되지 않은 스트림, 복구된 잔여 데이터 또는 기타 파서 컨텍스트를 알려 줍니다. |
| 포함된 LNK | `TargetCreated`, `TargetModified`, `TargetAccessed`, `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`, `DriveType`, `VolumeSerialNumber`, `VolumeLabel`, `LocalPath`, `CommonPath`, `TargetIDAbsolutePath`, `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`, `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `ExtraBlocksPresent`, `Args`: [LNK 바로 가기](lnk.md)에 설명된 대로 해석하십시오. |
| 추가 LNK | `NetworkPath`, `Name`, `IconLocation`: 포함된 바로 가기의 네트워크 및 표시 필드입니다. |
| 검증 | `Problems`: 누락된 스트림, 잘림, 개수 불일치 또는 링크 손상입니다. |

## CustomDestinations CSV 필드 { #customdestinations-csv-fields }

사용자 지정 출력은 `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`, `AppId`,
`AppIdDescription`, `EntryName`(범주 이름 또는 `Tasks`)으로 시작하고, 위에 나열된 모든 포함된 LNK 필드가
이어지며, 그다음 `Notes`, `NetworkPath`, `Name`, `IconLocation`, `Problems`가 옵니다.

JSON/JSONL은 여기에 더해 DestList 헤더, 항목별 점수/체크섬/속성 저장소, `DestListPropertyStore`,
복합 파일 스트림, 사용자 지정 범주, 마지막 범주 이후의 바이트, 그리고 포함된 모든 LNK의 중첩 필드를
보존합니다.

## 조사 전략 { #investigation-strategy }

1. 사용자와 원본 프로필을 식별한 다음, AppID를 신중하게 애플리케이션에 매핑하십시오.
2. 자동 항목을 `LastModified`와 `MRU`로 정렬하고, 고정된 항목을 분리하십시오.
3. 경로, 네트워크 공유, 이동식 볼륨 일련번호, MFT 참조 및 인수를 검토하십시오.
4. `Notes`와 연결되지 않은/잔여 링크를 검사하십시오. 삭제되었거나 이전의 항목이 그곳에 남아 있을 수
   있습니다.
5. RecentDocs, 단독 Recent LNK, Prefetch, SRUM, `$MFT` 및 이벤트 로그와 상관 분석하십시오.

## 추가 정보 { #further-details }

- [Microsoft: Compound Binary File 형식 (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft: Shell Link 형식 (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [Eric Zimmerman의 JLECmd](https://github.com/EricZimmerman/JLECmd)
- [JumpList 파서 테스트/참조 프로젝트](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
