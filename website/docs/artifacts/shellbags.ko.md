# ShellBags { #shellbags }

## ShellBags란 { #what-shellbags-are }

Explorer는 사용자가 연 각 폴더가 어떻게 표시되는지, 즉 보기 방식, 아이콘 크기, 정렬 순서 및 창 위치를
기억합니다. 폴더의 설정을 다시 찾기 위해 Explorer는 폴더의 식별 정보를 셸 항목 트리 형태로 사용자의
레지스트리에 보관합니다. 이 항목들은 링크의 대상 ID 목록이 담고 있는 것과 같은 항목으로, 드라이브에서
시작하여 연 폴더까지 이어지는 각 폴더입니다. 이 트리가 ShellBags입니다.

ShellBags는 Explorer와 열기 및 저장 대화 상자에서 연 폴더를 다루며, 여기에는 이동식 드라이브, 네트워크
공유, 휴대용 장치(휴대전화, 카메라), ZIP 압축 파일 및 FTP 서버의 폴더, 그리고 제어판 페이지가 포함됩니다.
폴더, 드라이브 또는 네트워크 공유가 사라진 후에도 남아 있습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 이 사용자의 Explorer가 이 폴더(또는 그 아래의
폴더)를 표시했습니다. ShellBag은 폴더에서 무엇을 했는지 보여 주지 않으며, 모든 방문의 날짜를 알려 주지도
않습니다. 아래의 시간 설명을 참조하십시오.

## 위치 { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

`BagMRU` 키에는 각각 하나의 셸 항목인 번호가 매겨진 값들과, 그 순서(가장 최근에 연 것이 먼저)를 나타내는
`MRUListEx` 값이 있습니다. 각 값에는 그 아래 항목을 위한 같은 이름의 하위 키가 있으므로, 경로는 `BagMRU`에서
해당 값까지 내려가는 항목들입니다. 하위 키의 `NodeSlot`은 폴더의 보기 설정이 들어 있는 `Bags\<N>` 키의
번호입니다.

모든 프로필의 `UsrClass.dat`와 `NTUSER.DAT`를 트랜잭션 로그(`.LOG1`, `.LOG2`)와 함께 수집하고,
`NTUSER.DAT` 사용자의 SID를 제공하는 시스템의 `SOFTWARE` 하이브도 수집하십시오.

## 각 시간이 나타내는 것 { #which-times-say-what }

- **셸 항목의 시간**(`ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC`)은 Explorer가 항목을 기록한
  시점의 폴더 자체 파일 시스템 시간(FAT 정밀도: 2초)입니다. 방문이 아니라 폴더의 날짜를 나타내며, 이후 방문
  시에는 업데이트되지 않습니다.
  FAT 시계에는 저장된 UTC 오프셋이 없습니다. 레거시 `UTC` 제목과 표시되는 `Z` 접미사는 표시 규칙일 뿐이며,
  원본 시간대의 증거가 아닙니다. `auto-parse --timezone`은 이 필드들을 그대로 보존하므로, 원본 시스템의
  컨텍스트와 함께 해석하십시오.
- **키의 마지막 쓰기 시간**은 `MRUListEx`가 다시 기록될 때, 즉 그 항목 중 하나가 열려 맨 앞으로 이동할 때
  바뀝니다. 따라서 항목의 값이 들어 있는 키의 시간(`ParentKeyLastWrittenUTC`)은 **해당 항목이 그 키에서 가장
  최근 항목인 경우에만**(`IsMostRecentInParent` true, `MruPosition` 0) 그 항목이 마지막으로 열린 시간입니다.
  나머지 항목에 대해서는 형제 항목이 열린 더 나중의 시간입니다.
- 항목 자체의 키(`KeyLastWrittenUTC`)는 같은 방식으로 가장 최근 하위 항목이 열린 날짜를 나타냅니다.

## 분석 시 주의 사항 { #analyst-cautions }

- **부재는 거의 아무것도 증명하지 않습니다.** Explorer는 폴더의 보기가 저장될 때 ShellBags를 기록합니다.
  잠깐만 본 폴더나 다른 방법(명령줄, 스크립트, 프로그램)으로 연 폴더에는 ShellBag이 없을 수 있습니다.
  사용자가 키를 삭제할 수도 있습니다.
- **NTFS 참조는 항목이 기록된 시점의 폴더 참조입니다**(`MftEntry`, `MftSequence`). `$MFT`에서 조회하여 같은
  폴더가 여전히 존재하는지, 또는 다른 파일이 해당 엔트리를 재사용했는지 확인하십시오.
- **이동식 드라이브**는 드라이브 문자로 이름이 지정됩니다. 그 문자는 당시 드라이브가 가졌던 문자이므로, USB
  및 마운트된 장치 기록과 대조하여 장치를 확인하십시오.
- **이름은 항목에서 가져옵니다.** 나중에 이름이 바뀐 폴더도 여기에는 이전 이름이 남습니다. 루트 폴더와 알려진
  폴더는 포렌식 도구에서 일반적으로 사용하는 이름(`This PC`, `Quick Access`, `Downloads`)으로 표시되며,
  `Guid`가 그 식별자를 제공합니다.
- **검색 결과, 라이브러리 및 제어판 페이지**는 디스크상의 폴더가 아니라 보기입니다(`property_view`,
  `control_panel_item`, `control_panel_category`).
- **어떤 파서도 알지 못하는 항목**은 `ItemType` `unknown`과 함께 바이트를 `DataRaw`에 보존하고, 클래스
  바이트를 `ItemClass`에 기록합니다. 손상되었거나 특이한 항목은 `Problems`에 보고되며, 그 바이트도
  보존됩니다.
- **더티(dirty) 하이브**는 로그가 없으면 최신 항목이 누락될 수 있습니다. Juntei는 `--no-logs`가 지정되지
  않는 한 하이브 옆의 로그를 재생하며, `TransactionLogStatus`는 레코드가 로그에서 왔는지 여부를 나타냅니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shell-bags -d evidence/ -o shellbags.csv
juntei parse-registry shell-bags -f UsrClass.dat -t jsonl \
  | jq 'select(.ItemType == "volume" and .GUID == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

셸 항목은 `parse-file lnk`의 셸 항목 파서인 ys-lnk-parser로 디코딩됩니다. 요약에는 상위 키의 마지막 쓰기
시간을 기준으로 마지막에 연 폴더가 최신 순으로 사용자와 함께 나열됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath`, `HiveType` | 호스트, 프로필 폴더 이름, 사용자의 SID(`UsrClass.dat`는 루트 키 이름에, `NTUSER.DAT`는 시스템의 `ProfileList`에 있음), 하이브 및 그 유형입니다. |
| `BagKey` | 하이브 루트 아래의 항목 키(`…\BagMRU\4\3\0`)이며, 그 값은 바로 위의 키에 있습니다. |
| `MRU-Position`, `IsMostRecentInParent` | 상위 키의 `MRUListEx` 내 위치(0: 가장 최근에 연 것)이며, 목록에 없으면 비어 있습니다. |
| `NodeSlot` | 해당 `Bags\<N>` 키(폴더의 보기 설정)의 번호입니다. |
| `Path` | 트리의 맨 위부터 해당 항목까지의 항목을 `\`로 연결한 것입니다. `[unknown]`은 값이 누락된 항목을 나타냅니다. |
| `ItemType`, `ItemClass` | 항목의 종류(`root_folder`, `volume`, `directory`, `file`, `network_location`, `uri`, `ftp_folder`, `compressed_folder`, `portable_device`, `portable_device_folder`, `users_files_folder`, `property_view`, `control_panel_item`, `control_panel_category`, `cd_burn`, `unknown`)와 클래스 바이트입니다. |
| `ItemName`, `GUID` | 이름(폴더의 긴 이름)과, 루트 폴더, 제어판 항목 또는 위임 폴더의 클래스 식별자입니다. |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | **셸 항목에서 얻음:** 항목이 기록된 시점의 폴더 시간입니다(FAT, 2초 정밀도). |
| `MFT-Entry`, `MFT-Sequence` | **셸 항목에서 얻음:** 폴더의 NTFS 파일 참조입니다. |
| `ExtensionBlocks` | 항목이 가진 확장 블록 수입니다. |
| `KeyLastWrittenUTC` | **레지스트리에서 얻음:** 항목 자체 키의 마지막 쓰기 시간으로, 가장 최근 하위 항목이 열린 시간입니다. |
| `ParentKeyLastWrittenUTC` | **레지스트리에서 얻음:** 항목의 값이 들어 있는 키의 마지막 쓰기 시간으로, `IsMostRecentInParent`가 true이면 해당 항목이 마지막으로 열린 시간입니다. |
| `DataRaw` | 완전히 해석되지 않았거나 문제가 있는 항목의 경우 값의 바이트(16진수)입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 키의 `MRUListEx`가 지정하지만 키에 없는 값, 목록이 지정하지 않는 값, 값이 없는 하위 키, 바이너리가 아닌 값, 잘못된 유형의 `NodeSlot`, 손상된 셸 항목 목록입니다. |

## 조사 전략 { #investigation-strategy }

1. 이동식 드라이브(Removable Drives `Guid`를 가진 `volume`), 네트워크 위치, FTP 폴더, 휴대용 장치와 그 아래의
   폴더를 나열하십시오.
2. 관심 있는 폴더를 찾으십시오. 사용자가 쓰기 가능한 스테이징 위치(`Temp`, `ProgramData`, `Public`), 도구 및
   압축 파일 폴더, 다른 사용자의 프로필, 그리고 더 이상 존재하지 않는 폴더(`$MFT`에서 NTFS 참조 확인)가
   해당됩니다.
3. 방문 날짜는 `IsMostRecentInParent`가 true인 항목의 `ParentKeyLastWrittenUTC`로만 판단하십시오. 나머지는
   파일을 연 날짜를 나타내는 LNK 파일, Jump Lists 및 RecentDocs와 상관 분석하십시오.
4. 삭제된 `BagMRU` 키와 값을 찾으려면 `parse-registry read --recover-deleted`를 사용하십시오.

## 추가 정보 { #further-details }

- [libyal libfwsi: Windows Shell Item 형식](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb: Most recently used (BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

`ProfileList`에서 SID를 조회하려면 고유하고 구문적으로 유효한 SID가 하나 있어야 합니다. 모호한 프로필 폴더
매핑과 `S-1-...bak` 같은 백업 키가 있으면 `UserSid`는 알 수 없는 상태로 남습니다.
