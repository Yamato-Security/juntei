# 열기 및 다른 이름으로 저장 대화 상자 (OpenSaveMRU) { #open-and-save-as-dialogs-opensavemru }

## 대화 상자 기록이란 { #what-the-dialog-history-is }

프로그램이 공용 열기 또는 다른 이름으로 저장 대화 상자를 표시하면, Windows는 사용자의 `NTUSER.DAT`에 다음을
기록합니다.

- **어떤 파일이 열리거나 저장되었는지:** `OpenSavePidlMRU`에는 확장자(`docx`, `ps1`, …)마다 하나의 하위 키와
  모든 확장자를 위한 `*`가 있으며, 각각은 최근 사용(MRU) 목록입니다.
- **어떤 프로그램이 대화 상자를 사용했고 어느 폴더에 있었는지:** `LastVisitedPidlMRU`(이전 대화 상자를
  사용하는 프로그램의 경우 `LastVisitedPidlMRULegacy`).
- **대화 상자를 표시한 프로그램:** `CIDSizeMRU`(프로그램별 대화 상자 크기)와 `FirstFolder`(프로그램의 경로와
  폴더).
- Windows 2000과 XP에서는 `OpenSaveMRU`(확장자별 경로)와 `LastVisitedMRU`(프로그램과 그 폴더를 경로로 표시).

Windows Vista부터 파일과 폴더는 바로 가기의 대상과 같은 구조인 셸 항목 목록으로 저장됩니다. 각 항목은 경로의
한 부분(내 PC, 드라이브, 폴더, 파일)이며, 파일이나 폴더 항목은 자체 시간과 NTFS 파일 참조를 가질 수 있습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 이 사용자의 세션에서 어떤 프로그램이 열기 또는 다른
이름으로 저장 대화 상자를 사용하여 이 파일을 열거나 저장했거나, 이 폴더에서 대화 상자를 사용했습니다. 각
목록에서 가장 최근 항목만 날짜가 있으며, 이는 키의 마지막 쓰기 시간입니다.

## 위치 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

모든 프로필에 대해 `NTUSER.DAT`를 트랜잭션 로그와 함께 수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- **목록당 하나의 시간.** 키의 마지막 쓰기 시간은 항목 중 하나가 목록의 맨 앞으로 이동할 때 변경되므로,
  위치 0의 항목(`IsMostRecent`, `LastUsedUTC`)만 날짜를 알 수 있습니다. 나머지 항목은 목록 아래로 갈수록
  최신에서 오래된 순서로 정렬되어 있지만 날짜는 없습니다.
- **두 목록에 있는 같은 파일.** 파일은 해당 확장자의 목록과 `*`에 모두 있습니다. 두 번 사용된 것으로 세지
  마십시오. `*`는 확장자 전체에 걸친 순서를 알려 줍니다.
- **열었는지 저장했는지.** 목록은 어느 쪽인지 알려 주지 않습니다. 파일의 `$MFT` 및 USN 저널 레코드, LNK 파일,
  Jump Lists와 상관 분석하십시오.
- **항목 시간은 대화 상자가 아니라 파일의 시간입니다.** `ItemModifiedUTC`, `ItemCreatedUTC`,
  `ItemAccessedUTC`는 항목이 기록될 당시 파일의 시간이며, 2초 단위(FAT)입니다. 알려진 폴더
  (`CLSID_ThisPCDocumentsRegFolder`, `Downloads`, …)의 항목에는 시간이 없는 경우가 많습니다. 이 FAT
  타임스탬프에는 표준 시간대 오프셋이 없습니다. `UTC` 열 접미사는 표시 관례일 뿐 원래 시계가 UTC였다는 증거가
  아닙니다. 레지스트리 FILETIME과 비교하기 전에 원본 시스템의 표준 시간대 및 다른 타임스탬프와 상관 분석하십시오.
  Microsoft의 [파일 시간 안내](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times)를 참조하십시오.
- **경로는 셸 경로입니다.** 경로는 바탕 화면에서 시작합니다: `This PC\C:\Users\…`, 알려진 폴더, 라이브러리
  (`UsersLibraries\Documents\…`) 또는 네트워크 위치. 폴더나 파일로 시작하는 경로는 바탕 화면에 있는 것입니다.
  빈 경로는 바탕 화면 자체입니다.
- **디코딩되지 않은 항목.** 셸 항목 파서가 알지 못하는 유형의 항목은 클래스(`[class 0x00]`)로 표시되고,
  유형은 `unknown`이며, 목록의 바이트는 `DataRaw`에 들어 있습니다.
- **프로그램은 이름입니다.** `LastVisitedPidlMRU`와 `CIDSizeMRU`는 경로가 아니라 파일 이름(`WINWORD.EXE`)을
  보관합니다. `FirstFolder`는 경로를 보관합니다.
- **목록은 짧고 정리됩니다.** 각 목록은 제한된 수의 항목만 보관하며, 정리 도구는 키를 삭제합니다. 삭제된 값은
  `parse-registry read --recover-deleted`를 사용하십시오.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry open-save-mru -d evidence/ -o opensavemru.csv
juntei parse-registry open-save-mru -f NTUSER.DAT -t jsonl \
  | jq 'select(.List == "OpenSavePidlMRU" and .Extension != "*")'
```

요약에는 목록별 항목, 대화 상자를 사용한 프로그램, 그리고 각 하이브의 가장 최근 파일(`*`의 첫 번째 항목)이
표시됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | 호스트, 프로필 소유자(경로 또는 셸 폴더에서), 시스템 `ProfileList`의 SID, 그리고 하이브. |
| `List` | `OpenSavePidlMRU`, `LastVisitedPidlMRU`, `LastVisitedPidlMRULegacy`, `CIDSizeMRU`, `FirstFolder`, `OpenSaveMRU` 또는 `LastVisitedMRU`. |
| `Extension` | 확장자의 하위 키(`docx`, `*`). 다른 목록에서는 비어 있습니다. |
| `MRU-Position`, `IsMostRecent` | 키의 목록 내 위치(0이 가장 최근). 목록이 해당 값을 지칭하지 않으면 비어 있습니다. |
| `ValueName` | 값의 번호(`MRUListEx`) 또는 문자(`MRUList`). |
| `Program` | 대화 상자를 표시한 프로그램. |
| `Path` | 파일 또는 폴더: 셸 항목 이름을 연결한 것, 또는 텍스트로 저장된 경로. |
| `ItemType` | 마지막 셸 항목의 유형: `file`, `directory`, `root_folder`, `volume`, `network_location`, `property_view`, `unknown`, … |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | 셸 항목이 기록한 파일 또는 폴더의 시간. |
| `MFT-Entry`, `MFT-Sequence` | 셸 항목이 기록한 NTFS 파일 참조. |
| `LastUsedUTC` | 키의 마지막 쓰기 시간. 가장 최근 항목에만 해당합니다. |
| `KeyPath`, `KeyLastWrittenUTC` | 키와 키의 마지막 쓰기 시간: 모든 행이 아니라 가장 최근 항목의 시간입니다. |
| `DataRaw` | 디코딩되지 않을 때 값의 바이트(16진수), 또는 항목 유형을 알 수 없을 때의 셸 항목 목록. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트. |
| `Problems` | 목록이 지칭하지만 키에 없는 값, 목록이 지칭하지 않는 값, 잘못된 유형, 잘린 이름이나 목록. |

## 조사 전략 { #investigation-strategy }

1. 사용자별로 `*`를 순서대로 읽고, 가장 최근 파일은 `LastUsedUTC`로 확인합니다.
2. 사용자가 쓸 수 있는 위치, 이동식 위치 또는 네트워크 위치에 있는 스크립트, 압축 파일, 실행 파일, 문서와,
   자격 증명이나 내보내기 파일처럼 보이는 이름의 파일을 찾습니다.
3. `LastVisitedPidlMRU`를 사용하여 프로그램과 그 프로그램이 마지막으로 사용한 폴더를 연결하고, `CIDSizeMRU`와
   `FirstFolder`로 대화 상자를 한 번이라도 사용한 프로그램을 확인합니다.
4. 경로와 `MftEntry`를 `$MFT`, USN 저널, LNK 파일, Jump Lists, ShellBags, RecentDocs와 상관 분석합니다.

## 추가 정보 { #further-details }

- [libyal winreg-kb: 최근 사용(MRU) 키](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi: Windows Shell Item 형식](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
