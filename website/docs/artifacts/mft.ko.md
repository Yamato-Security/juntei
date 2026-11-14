# 마스터 파일 테이블 (`$MFT`) { #master-file-table-mft }

## MFT란 { #what-the-mft-is }

NTFS는 모든 파일과 디렉터리를 마스터 파일 테이블(Master File Table)의 레코드로 나타냅니다. 레코드에는
이름, 상위 참조, 타임스탬프, 크기, 플래그, 보안 식별자, 리파스(reparse) 정보 및 데이터 스트림 메타데이터
같은 속성이 들어 있습니다. 작은 파일의 콘텐츠와 대체 데이터 스트림(ADS)은 **상주**(resident)할 수 있으며,
이는 해당 바이트가 MFT 레코드 자체 안에 저장된다는 뜻입니다.

MFT는 볼륨 전체에 걸쳐 할당된 레코드와 삭제된 레코드를 모두 포함하므로 기초가 되는 아티팩트입니다. 일반적인
디렉터리 탐색으로는 더 이상 보이지 않는 경우에도, 파일이 존재했다는 사실, 어디에 링크되어 있었는지,
삭제되었는지 여부, 그리고 메타데이터가 어떻게 변경되었는지를 보여 줄 수 있습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 표시된 메타데이터를 가진 NTFS 파일 레코드가 수집
시점에 존재했거나 할당되지 않은 레코드에 남아 있었습니다. MFT만으로는 실행이나 사용자 의도를 증명하지
못합니다.

## 위치와 수집 { #location-and-collection }

```text
C:\$MFT
```

`$MFT`는 보호된 NTFS 메타데이터입니다. 원시 NTFS 접근이 가능한 포렌식 수집 도구나 트리아지 도구로
수집하십시오. 일반적인 사용자 모드 복사로는 보통 읽을 수 없습니다. 가능하면 `$J`, `$LogFile` 및 볼륨
메타데이터와 함께 수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- 삭제된 레코드는 부분적으로 덮어써졌거나 레코드 번호가 재사용되었을 수 있습니다.
- `SN`는 같은 엔트리 번호의 세대를 구별합니다. 엔트리와 시퀀스를 **모두** 대조하십시오.
- `$STANDARD_INFORMATION`(`0x10`)과 `$FILE_NAME`(`0x30`) 타임스탬프는 업데이트 동작이 다릅니다. 어느
  쪽도 조작으로부터 안전하지 않습니다.
- `SI<FN`, `uSecZeros` 및 `Copied`는 단서일 뿐이며, 자동으로 타임스톰핑(timestomping)이라고 판정하는
  것이 아닙니다.
- 하드 링크는 하나의 파일 레코드에 대해 여러 `$FILE_NAME` 속성과 경로를 만듭니다.
- NTFS는 여러 이름(DOS 8.3 포함), 대체 데이터 스트림 및 확장 레코드를 저장할 수 있습니다.
- 액세스 시간 업데이트는 비활성화되거나 지연되거나 병합될 수 있습니다.
- 전체 경로는 상위 레코드로부터 재구성됩니다. `[Orphaned]`, `[Unknown]` 또는 `PathUnknown`은 누락되었거나
  재사용되었거나 손상된 상위 계층을 나타내며, 문자 그대로의 디렉터리 이름이 아닙니다.
- FILE 레코드의 **슬랙(slack)**(사용 중인 크기를 넘어선 바이트)에는 이전에 가지고 있던 이름이 남아 있을 수
  있습니다. 인덱스가 인덱스 할당(index allocation)으로 이동한 작은 디렉터리의 인덱스 항목(대부분은 그곳에서
  여전히 사용 중), 이전 이름, 또는 재사용된 레코드의 이전 점유자가 그 예입니다. 복구된 이름에는
  `$STANDARD_INFORMATION`이 없으며, 이름의 엔트리 헤더가 덮어써졌을 수 있습니다. 레코드의 사용 중 크기나
  속성으로 슬랙의 시작 위치를 알 수 없으면 아무것도 복구되지 않으며, 해당 행의 `ParseErrors`에
  `slack not read`와 그 이유가 표시됩니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $MFT files
  -f, --file <FILE>      Parse one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse selected entry numbers (example: 0-15,30)

Output:
  -o, --output <PATH>                   Save records
  -t, --output-type <TYPE>              csv, ys-ntfs-parser-csv, json, jsonl
  -e, --extract-resident-streams <DIR>  Extract resident $DATA streams
  -R, --reconstruct-filesystem <DIR>  Recreate a tree; recovered content gets .dontrun
      --reconstruction-i30 <FILE>      Supplement the tree with an I30 file (repeatable)

CSV Output:
  -a, --all-times      Include every $FILE_NAME time
  -s, --short-names    Include DOS 8.3-name rows
      --cutoff <DATE>  Keep recent SI-created or SI-modified records

Records:
  -x, --recover-slack  Recover file names from FILE record slack

Resident Data:
  -i, --include-resident-data      Put resident $DATA in the CSV
  -E, --resident-extensions <EXT>  Only these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  Only data up to this size

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail
  -L, --list-directory <ENTRY|PATH>  List a directory's contents
```

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff '2026-10-20T00:00:00Z' -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 32000-32100 | jq .
juntei parse-ntfs mft -f '$MFT' -x -o mft-with-slack.csv
juntei parse-ntfs mft -f '$MFT' -i -E ps1,bat,vbs,js -o scripts.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Users\alice\Downloads'
```

`-e`로 추출한 상주 스트림은 절대 실행하지 마십시오. Juntei가 `.dontrun` 이름을 붙이는 데에는 이유가
있습니다. `-i`의 상주 데이터 열도 같은 바이트이므로 신뢰할 수 없는 콘텐츠로 취급하십시오.

`-D`는 하나의 엔트리를 출력합니다. 헤더, 모든 속성(데이터 런 포함), 그리고 `-t json -x`가 기록하는 형식의
슬랙 내 이름을 출력한 다음 레코드의 바이트를 출력합니다. `-L`은 디렉터리의 모든 이름(사용 중인 것과 삭제된
것)을 엔트리 및 시퀀스 번호, 유형, 크기, 수정 시간과 함께 나열합니다. 디렉터리는 엔트리(`30`, `0x1E`,
`30-1`) 또는 경로(`\Windows\System32`, `C:/Windows`, 대소문자 구분 없음)로 지정합니다.

다음 예제는 탐색 가능한 파일 경로를 재구성한다. 비상주 내용은 빈 파일로 만든다. 자세한 내용: [파일 시스템 재구성](../commands/auto-parse.md#filesystem-reconstruction).

```bash
juntei parse-ntfs mft -f '$MFT' -R reconstructed
```

## CSV 출력 필드 { #csv-output-fields }

| 필드 | 해석 |
|---|---|
| `EntryNum` (Entry Number) | MFT 레코드 번호입니다. 레코드 번호는 재사용되므로 `SN`와 함께 사용하십시오. |
| `SN` (Sequence Number) | NTFS 참조에서 사용하는 세대 카운터입니다. 불일치는 오래되었거나 삭제된 관계를 드러낼 수 있습니다. |
| `InUse` | 수집 시점의 할당 플래그입니다. `false`는 레코드가 비어 있거나 삭제되었다는 뜻이며, 모든 바이트가 온전하다는 뜻은 아닙니다. |
| `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number) | `$FILE_NAME`의 상위 디렉터리 참조입니다. 두 부분을 모두 검증하십시오. |
| `ParentPath` | 재구성된 상위 경로입니다. 알 수 없음/고아 표시는 해결되지 않은 상위 계층을 나타냅니다. |
| `File` (Filename) | 현재 행의 `$FILE_NAME` 또는 스트림의 이름입니다. 하나의 레코드에 여러 행이 있을 수 있습니다. |
| `Ext` (Extension) | 이름에서 파싱한 편의상의 확장자이며, 신뢰할 수 있는 파일 유형 판정이 아닙니다. |
| `Size` (File Size) | 행/스트림에 연결된 논리적 크기입니다. 스파스/압축 할당 크기와 다를 수 있습니다. |
| `RefCnt` (Reference Count) | NTFS 하드 링크 참조 수입니다. 남아 있는 이름 수와는 신중하게 비교하십시오. |
| `ReparseTarget` | 정션/심볼릭 링크 같은 지원되는 리파스 포인트의 디코딩된 대상입니다. 경로가 리디렉션될 수 있습니다. |
| `Dir` (Directory) | 레코드에 디렉터리 플래그가 있습니다. |
| `HasADS` (Has Alternate Data Streams) | 레코드에 이름이 있는 대체 데이터 스트림이 포함되어 있습니다. 이것만으로 악성이 되지는 않습니다. |
| `IsADS` (Is Alternate Data Stream) | 이 행이 이름 없는 파일 데이터가 아니라 이름이 있는 대체 데이터 스트림을 나타냅니다. |
| `SI<FN` (Standard Information Before File Name) | SI 생성 시간이 다른 FN 생성 시간보다 앞섭니다. 복사/타임스톰핑과 정당한 원인을 조사하십시오. |
| `uSecZeros` (Microsecond Zeros) | SI 생성/수정 시간이 정확히 밀리초 경계에 위치합니다. 약한 타임스톰핑 휴리스틱일 뿐입니다. |
| `Copied` | 수정 시간이 생성 시간보다 앞섭니다. 파일 복사와 일치하는 경우가 많으며, 공격자 행위의 증거가 아닙니다. |
| `SI-Flags` (Standard Information Flags) | `$STANDARD_INFORMATION` 파일 속성/플래그입니다. 숨김/시스템 플래그는 유용한 피벗이 될 수 있습니다. |
| `Type` (Name Type) | `$FILE_NAME` 네임스페이스(Win32, DOS, POSIX 또는 조합)입니다. 이름을 중복 집계하지 않도록 사용하십시오. |
| `DataModified (SI)`, `Accessed (SI)`, `MetadataChanged (SI)`, `Created (SI)` | SI 시간입니다. 일반적인 타임라인 필드이지만, 사용자 모드 도구로 그중 여러 개를 변경할 수 있습니다. |
| `DataModified (FN)`, `Accessed (FN)`, `MetadataChanged (FN)`, `Created (FN)` | FN 시간입니다. 업데이트 방식이 다르며 일반적인 API로 변경하기가 대체로 더 어렵지만, 여전히 변경 불가능한 것은 아닙니다. |
| `USN` (Update Sequence Number) | 레코드의 SI 속성과 연결된 마지막 USN입니다. 해당 USN이 남아 있으면 `$J`로 피벗하십시오. |
| `LogfileSN` (Logfile Sequence Number) | FILE 레코드 헤더의 NTFS `$LogFile` 시퀀스 번호로, 저수준 트랜잭션 분석에 유용합니다. |
| `SID` (Security ID) | NTFS `$Secure`에 대한 인덱스이며, 그 자체로 Windows SID는 아닙니다. 파일의 소유자와 ACL을 확인하려면 [`parse-ntfs secure-sds`](secure-sds.md)의 `SecurityID`와 결합하십시오. |
| `ObjectID-FileDroid` | 존재하는 경우 객체 ID / 분산 링크 추적 식별자입니다. LNK/Jump List의 droid와 상관 분석하십시오. |
| `LoggedUtilStream` | 지원되는 경우 디코딩된 `$LOGGED_UTILITY_STREAM` 컨텍스트입니다. 익숙하지 않은 값은 검토하십시오. |
| `ZoneID-Contents` | ZoneId와 다운로드 URL 같은 상주 `Zone.Identifier` 콘텐츠입니다. 없거나 제거되었거나 비상주일 수 있습니다. |
| `SourceFile` | 입력 `$MFT`로, 여러 호스트나 볼륨을 결합할 때 필수적입니다. |
| `ResidentDataBase64`, `ResidentDataHex`, `ResidentDataASCII` | `-i` 사용 시: 행의 상주 `$DATA`(파일 또는 대체 데이터 스트림의 데이터) 콘텐츠를 Base64, 바이트(`41-42`), 그리고 출력 가능한 ASCII인 경우 텍스트로 나타냅니다. 데이터가 비상주이거나 `-E`/`-M`으로 필터링된 경우 비어 있습니다. |
| `ParseErrors` | 이 행에 영향을 주는 손상 또는 지원되지 않는 관계입니다. 필터링 중에 이 열을 절대 버리지 마십시오. |
| `FromSlack` | `-x` 사용 시: FILE 레코드의 슬랙에서 복구된 이름이면 `True`입니다. 해당 `EntryNum`과 `SN`는 그 이름이 가리키던 파일이며(엔트리 헤더가 덮어써진 경우 비어 있음), 복구된 타임스탬프는 `(FN)` 열에 있고 `InUse`는 `False`입니다. |
| `SlackRecord`, `SlackOffset` | `-x` 사용 시: 이름이 들어 있던 슬랙을 가진 FILE 레코드(레코드 내부에 기록된 레코드 번호가 아니라 `$MFT` 내 위치 기준이며, 확장 레코드의 이름이 기본 레코드와 함께 나열되더라도 확장 레코드 자체의 위치)와, `$MFT` 내 이름의 바이트 오프셋입니다. 슬랙 행의 `ParseErrors`는 `fixup values do not match` 같은 해당 레코드의 문제이며, 복구된 바이트가 영향을 받았을 수 있습니다. |
| `CopyOfEntryInUse` | `-x` 사용 시: 사용 중인 파일이 같은 디렉터리에 여전히 이 이름을 가지고 있습니다(참조가 남아 있는 경우 같은 파일임). 사용 중인 어떤 파일도 가지고 있지 않은 이름이 주목할 만한 대상입니다. 삭제되었거나 이름이 변경되었거나 재사용된 레코드의 이전 점유자입니다. |

타임스탬프 열은 MACB 순서(Modified, Accessed, Changed, Birth/creation)를 따릅니다.
`DataModified` SI/FN, `Accessed` SI/FN, `MetadataChanged` SI/FN, 그다음 `Created` SI/FN입니다.

`(SI)`는 `$STANDARD_INFORMATION`, `(FN)`은 `$FILE_NAME`을 의미합니다. 이 이름은 완전한 레코드의 일반적인
출처를 나타냅니다. SI 속성이 없는 일반 파일 이름 행의 경우, 기존 대체 처리에 따라 FN 생성 시간은
`Created (FN)`에, FN 수정/레코드 변경/액세스 값은 각각 `DataModified (SI)`, `MetadataChanged (SI)`,
`Accessed (SI)`에 배치됩니다. 복구된 슬랙 행은 네 가지 시간을 모두 `(FN)` 열에 배치하고 SI 열은 비워
둡니다. 일반 행의 대체 값은 독립적인 SI 증거가 **아닙니다**. 타임스탬프 출처를 판단할 때는 원래 JSON
속성과 슬랙 출처 정보를 사용하십시오.

`-t ys-ntfs-parser-csv`(이전 명칭 `ys-mft-parser-csv`도 계속 허용됨)는 유지 관리되는 파서의 엔트리 중심
스키마를 사용합니다. 이 읽기 쉬운 열 이름은 `auto-parse`의 `MFT.csv`를 포함한 기본 `-t csv` 출력에
적용됩니다. JSON/JSONL은 모든 속성(상주 `$DATA` 포함, 16진수)과 `FullPath`를 보존하므로, 평면적인 조사용
CSV에서 저수준 구조가 생략되는 경우 이를 사용하십시오. `-x`를 사용하면 각 JSON 엔트리에
`SlackFileNames`도 포함됩니다.

## 조사 전략 { #investigation-strategy }

1. 시간 범위로 필터링하되, 삭제된 레코드와 두 타임스탬프 세트를 모두 유지하십시오.
2. 전체 경로, 의심스러운 이중 확장자, ADS, 스테이징 디렉터리 및 알려진 해시/이름을 검색하십시오.
   `-x`를 사용할 때는 사용 중인 어떤 파일도 가지고 있지 않은 슬랙 이름(`CopyOfEntryInUse` `False`)을
   포함하고, `-i -E ps1,bat,vbs,js`를 사용하면 작은 스크립트를 CSV에서 바로 읽을 수 있습니다.
3. 엔트리/시퀀스와 `USN`를 USN 저널로 피벗하십시오.
4. Prefetch/LNK/Jump List의 MFT 참조를 대상 레코드와 비교하십시오.
5. 삭제를 Defender 격리 데이터 또는 USN `FILE_DELETE`/이름 변경 활동과 상관 분석하십시오.

## 추가 정보 { #further-details }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [ys-ntfs-parser의 mft 크레이트](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/mft)
- [Omer Ben-Amram의 mft 파서](https://github.com/omerbenamram/mft)
- [libfsntfs 형식 문서](https://github.com/libyal/libfsntfs/tree/main/documentation)
